# Phase 0 — Source-Level Review of the Rust Foundation

| | |
|---|---|
| **Date** | 2026-10-16 |
| **Directive** | ENGINEERING DIRECTIVE 001-R1 §5 (review the Rust implementation) and §9 (validation) |
| **Reviewed revision** | `a522aeeca80361f1e9757e7663bf5f7e47f68409` (pre-review HEAD) |
| **Crates reviewed** | `cosmic-source`, `cosmic-diagnostics`, `cosmic-cli` |
| **Method** | Full source read, plus an adversarial out-of-tree probe harness built against the crates as a path dependency (not committed) |

The probe harness exercised every byte offset of a set of pathological inputs,
checked renderer invariants exhaustively, emitted JSON for independent
validation, and loaded an oversized sparse file. It found two defects, both
fixed here.

---

## 1. Review areas covered

| Directive review area | Verdict |
|---|---|
| Data representation and public API contracts | Reviewed — no defect |
| Span invariants and `FileId` ownership | Reviewed — invariants hold; one column-mapping defect found and fixed (§2) |
| UTF-8 boundary handling | Reviewed — exhaustive offset scan found no boundary error |
| Integer overflow and allocation limits | Reviewed — one allocation-limit defect found and fixed (§3) |
| Invalid-input error handling | Reviewed — total; one contract deviation found and fixed (§3) |
| Diagnostic determinism | Reviewed — deterministic; verified by exact-output tests |
| JSON validity | Reviewed — validated with an independent parser (§4) |
| Test quality and meaningful coverage | Reviewed — two coverage gaps reported, not fixed (§6) |
| Potential runtime panics | Reviewed — none reachable from input data (§5) |
| Dependency and unsafe-code policy | Reviewed — verified compliant (§7) |

---

## 2. Defect 1 (FIXED) — CRLF line terminators were counted as a column of their own

**Severity:** low (cosmetic in output, but it broke an internal consistency
invariant between two public API methods).

**Location:** `crates/cosmic-source/src/file.rs`, `SourceFile::line_col`.

**Reproduction (pre-fix):**

```rust
let mut sources = SourceMap::new();
let id = sources.add("demo.csmx", "a\r\nb")?;
let file = sources.get(id).unwrap();

assert_eq!(file.line_text(0), Some("a"));          // the printed line is 1 char
assert_eq!(file.line_col(2), Some(LineCol { line: 1, column: 3 }));
//                                        ^ column 3 for a 1-character line
```

**Observed pre-fix behaviour.** `line_col` counted characters from the start of
the line **including the terminator bytes**, while `line_text` strips the
terminator. For `"a\r\nb"`, offset 2 (the `\n` of the CRLF pair) reported
line 1, column 3, although the printed line `"a"` is one character long. The
same offset in an LF file (`"a\nb"`, offset 1) reported column 2. So LF and
CRLF files disagreed by one column for the same logical position, and a
diagnostic whose span pointed at the `\n` of a CRLF pair made the human
renderer place its caret two columns past the end of the printed line.

The out-of-tree probe scanned every byte offset of ten pathological inputs
(`""`, `"abc"`, `"a\nb\nc"`, `"a\r\nb"`, `"a\rb"`, `"a\r\n"`, `"\r\n"`,
`"héllo"`, `"a😀b"`, `"let x =\r\nlet y = 2\r\n"`) and reported exactly the
CRLF cases:

```
[crlf] offset 2 -> line 1 col 3 exceeds visible line length 1 (line_text="a")
[crlf-code] offset 8 -> line 1 col 9 exceeds visible line length 7 (line_text="let x =")
[crlf-code] offset 19 -> line 2 col 11 exceeds visible line length 9 (line_text="let y = 2")
```

**Fix.** A new private helper `line_content_range` returns the line's byte
range without its terminator; `line_text` now uses it (removing duplicated
strip logic) and `line_col` clamps the counted range to it. A terminator byte
therefore maps to the column one past the last visible character — the same
position LF files already reported.

**Post-fix:** the probe reports zero findings across all ten inputs, and the
new invariant is enforced by a test that walks every offset of those inputs and
asserts `column <= visible_line_length + 1`
(`every_offset_maps_inside_its_printed_line`).

**Tests added:**
`line_terminator_columns_agree_across_lf_crlf_and_lone_cr`,
`every_offset_maps_inside_its_printed_line`.

**Documentation updated:** `docs/architecture/compiler-pipeline.md` §2.1 now
states the terminator column rule explicitly.

---

## 3. Defect 2 (FIXED) — `SourceMap::load` materialised oversized files before rejecting them

**Severity:** medium (unbounded allocation on invalid input; a documented error
contract could be replaced by a different error variant).

**Location:** `crates/cosmic-source/src/map.rs`, `SourceMap::load`.

**Reproduction.** Create a 5 GiB sparse file (no real disk usage) and call
`load`:

```sh
truncate -s 5G /tmp/huge.csmx        # apparent 5368709120 bytes, 0 B on disk
```

Measured **before** the fix, with the pre-review revision built out of tree:

```
load: TooLarge(5368709120) [documented]
        2.09 real         0.00 user         1.02 sys
           797605888  maximum resident set size
```

The 1.02 s of **kernel** time and the ~798 MB peak resident set prove the file
was read in full before the size check ran. `load` called `std::fs::read`
first and only then compared `bytes.len()` against the 4 GiB `u32` limit.

**Consequences.**

* *E1 (measured):* an oversized file is fully materialised before being
  rejected — 2.09 s and ~798 MB of allocation for a file that is rejected.
  The design report's page 14 explicitly requires "limites anti-DoS" on the
  source layer, and the crate documents the 4 GiB cap as a protection.
* *E2 (inferred, not reproduced on this host):* if the allocation fails,
  `std::fs::read` returns an I/O error and `map_err` produces
  `LoadError::Io` — **not** the documented `LoadError::TooLarge`. The public
  error contract for an oversized file therefore depended on available memory.
  This consequence could not be reproduced here: the sandbox rejects
  `setrlimit(RLIMIT_AS)` (`setrlimit` returned `-1`), so no memory-constrained
  run was possible. It is reported as inference, not as a measured fact.

**Fix.** `load` now checks the file's **declared length from metadata** before
reading, through a small pure helper `check_declared_len`. A metadata failure
is deliberately ignored: the subsequent read still produces the authoritative
I/O error, so behaviour for missing or unreadable files is unchanged. The
post-read check is retained as defence in depth (metadata can be stale).

**Measured after the fix:**

```
load: TooLarge(5368709120) [documented]
        0.00 real         0.00 user         0.00 sys
             1982464  maximum resident set size
```

2.09 s → 0.00 s; ~798 MB → ~1.98 MB (the process baseline). The documented
error variant is now produced without reading the file.

**Tests added:**

* `declared_length_limit_is_checked_before_reading` (unit) — covers 0, exactly
  the limit, limit + 1, 5 GiB and `u64::MAX`.
* `load_rejects_oversized_files_from_metadata_without_reading` (integration,
  `#[ignore]`d) — the end-to-end reproduction above. It is ignored by default
  on purpose: the fixture is a 5 GiB sparse file, and if the metadata
  pre-check were ever removed the test would try to read 5 GiB into memory. It
  is documented as a manual check to run after touching `load`.

---

## 4. Verified-correct areas (evidence, not assertions)

### 4.1 Span invariants and `FileId` ownership

* `Span` fields are private and `Span::new_unchecked` is `pub(crate)`; the only
  public constructors are `SourceFile::span` (checked) and
  `SourceFile::full_span` (always valid). Every `Span` in the system is
  therefore ordered, in bounds and on UTF-8 boundaries.
* `text_of` rejects spans from another file by comparing `FileId`s, verified by
  a test in both directions.
* `FileId` is a `NonZeroU32`, so `0` is unrepresentable; `from_raw(0)` returns
  `None`. Ids are assigned in registration order and never reused; the
  documented "session-local, resolved by numeric value" semantics is explicitly
  tested (`lookup_of_foreign_ids_is_out_of_range_or_numeric`).

### 4.2 UTF-8 boundary handling

The probe called `line_col` and `line_index` at **every** byte offset of ten
inputs including multibyte (`"héllo"`), astral (`"a😀b"`) and U+2028 text, and
asserted:

* `line_col` is `Some` exactly at character boundaries and `None` strictly
  inside a multibyte sequence;
* `line_index` is never `None` for an in-bounds offset.

No violation was found. The existing tests cover the same ground
(`astral_characters_are_single_boundaries`,
`span_constructor_rejects_offsets_inside_multibyte_characters`,
`columns_count_astral_characters_as_one`).

### 4.3 Integer overflow

* `Span::len()` is `end - start`; the ordering invariant makes underflow
  impossible.
* Offsets are `u32` throughout; the file cap (`u32::MAX - 1`) is enforced on
  both registration paths, and the arithmetic that could overflow
  (`u32::try_from(len)`, `count.checked_add(1)`) is explicitly checked.
* `line_col` builds its column with `u32::try_from(...).ok()?` then
  `saturating_add(1)` — no wrap, no panic.

### 4.4 JSON validity

The probe generated a diagnostic whose message, label, note and suggestion
contained **every** C0 control character, DEL, a quote, a backslash, an astral
emoji, U+2028 and a BOM, then validated the output with an independent parser
(`json.load`) and compared a re-serialisation with the raw bytes:

```
parsed OK; top-level keys: ['severity','code','message','labels','notes','suggestions']
labels[0] keys: ['primary','file','file_id','start','end','line','column','message']
roundtrip identical: True
```

The output is valid RFC 8259 JSON, all documented keys are present in schema
order, and escaping is correct. (The *human* renderer deliberately emits raw
control characters, matching rustc; that is a terminal format, not a machine
format.)

### 4.5 Determinism

* `DiagnosticBag::sorted` uses the stable `sort_by_key` over
  `(file_id, start, end)`; ties preserve insertion order — asserted with
  pointer identity in a test.
* Rendering is a pure function of the diagnostic plus the `SourceMap`; the
  exact-string tests for both formats pin the output byte-for-byte.
* `FileId` assignment is registration-order dependent by design and documented
  as such.

### 4.6 Panics reachable from input data

None found. The only `panic!` in the workspace is in
`DiagnosticCode::new`, a `const fn` for compiler-embedded literals; the
fallible path is `try_new`. The renderers never index unchecked: every lookup
is `get`/`?`/`unwrap_or`. `cosmic_cli::run` is total and never exits the
process itself; `main` performs the only `process::exit`.

### 4.7 Dependency and unsafe-code policy

* `unsafe_code = "forbid"` in `[workspace.lints.rust]` and repeated as
  `#![forbid(unsafe_code)]` in each library root; `grep -r unsafe` over
  `crates/` finds no `unsafe` block. Verified.
* Zero external dependencies in all three crates (`cosmic-diagnostics` depends
  only on `cosmic-source` by path). `Cargo.lock` is committed and lists only
  the three workspace packages. Verified.
* `missing_docs` / `missing_debug_implementations` warn, and CI promotes all
  warnings to errors; `clippy::dbg_macro`, `clippy::todo`,
  `clippy::unimplemented` are denied. Verified by a clean
  `cargo clippy --workspace --all-targets -- -D warnings`.

---

## 5. Findings reported but deliberately not changed

These are recorded for the owner and for Phase 1. None is a demonstrated
defect, and the directive asks for narrowly scoped fixes only.

| # | Finding | Class | Why not fixed now |
|---|---|---|---|
| N1 | `DiagnosticCode` requires `&'static str`, so codes cannot be built from runtime strings even via `try_new` | Design observation | Deliberate and documented; changing it is an API decision for the diagnostics specification (Phase 1) |
| N2 | The human renderer replaces tabs with a single space, so a caret under a tab is misaligned in a real terminal (tabs render as 1–8 columns) | Design observation | Documented trade-off; the alternative (display-width computation) needs the Unicode width tables the zero-dependency policy forbids in Phase 0 |
| N3 | Emoji and wide CJK characters count as one column but render as two, so markers can drift in the terminal | Design observation | Same reason as N2; inherent to char-counted columns, which is what the crate documents |
| N4 | `Severity` has no `Ord`, so callers cannot sort by severity | Design observation | Deliberate: `DiagnosticBag::sorted` defines source order as canonical, and the enum documents the choice |
| N5 | The renderer's `-->` gutter is `width` spaces, one narrower than rustc's layout in some cases | Cosmetic | Pinned by exact-output tests; changing it is a presentation decision, not a defect |

---

## 6. Test-coverage gaps (reported, not fixed)

| # | Gap | Impact | Recommended action |
|---|---|---|---|
| G1 | The `cosmic` **binary** is never executed by a test: `crates/cosmic-cli/tests/cli.rs` exercises `cosmic_cli::run` in-process, so the `main` wiring (stream locking, flush, `process::exit(i32::from(code))`) is untested | A regression in `main` would not be caught | Add one integration test using `env!("CARGO_BIN_EXE_cosmic")` in Phase 1, when the CLI gains a real command |
| G2 | JSON output is pinned by exact-string assertions only; no test parses it | Exact strings do pin the bytes, but a future schema change is not validated for *parseability* | Adopt `serde_json` as a dev-dependency when the dependency policy allows, or keep the current manual validation as documented evidence |
| G3 | `crates/cosmic-diagnostics/examples/demo.rs` is compiled but its output is not asserted | An example that silently breaks produces no failure | Compile-only coverage is accepted for Phase 0; assert its output when the CLI can print diagnostics |

---

## 7. Validation performed

Run from a clean worktree after the fixes, with the pinned toolchain
(1.99.0):

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

All four exit `0`.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean, no diff |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean, no warnings |
| `cargo test --workspace --all-targets` | **58 passed, 0 failed, 1 ignored** |
| `cargo test --workspace --doc` | **2 passed, 0 failed** |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | 4 crates documented, no warnings |

**Test-count delta.** The pre-review revision `a522aee…` reported **55 passed,
0 failed** (`cargo test --workspace --all-targets`, measured by building that
revision out of tree) plus 2 doc-tests. This review adds **3 passing tests**
(`declared_length_limit_is_checked_before_reading`,
`line_terminator_columns_agree_across_lf_crlf_and_lone_cr`,
`every_offset_maps_inside_its_printed_line`) and **1 ignored test**
(`load_rejects_oversized_files_from_metadata_without_reading`, a manual 5 GiB
fixture). The directive's "55 reported passing tests" is therefore confirmed
exactly, and the count is now 58 + 1 ignored + 2 doc-tests = 61.

**Continuous integration.** The branch tip that carries these fixes and the
audit documents was validated by two GitHub Actions runs, both **success**:

| Event | Run | Result |
|---|---|---|
| `push` (`phase/0-foundation`) | `38926681567` | success — Format, Lint, Test, Doc all green with the newly SHA-pinned actions |
| `pull_request` (#1) | `38926688309` | success |

The subsequent documentation commit that records this table is the branch tip
at the time of writing; its runs are visible in PR #1. No force-push was used
at any point, and published history was not rewritten.

**Reproducibility note.** No release artefacts exist in Phase 0, so no
reproducible-build claim is made (`docs/development/toolchain-policy.md` §5).
What is verifiable and verified: the pinned toolchain, the committed
`Cargo.lock`, zero external dependencies, and CI that re-runs the same four
commands on a different operating system (Linux) from the one used locally
(macOS).

