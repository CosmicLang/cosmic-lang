# Phase 2 Stage A — final verification report (Increment A3)

| | |
|---|---|
| **Increment** | A3 — final Stage A verification increment |
| **Directive** | ENGINEERING DIRECTIVE 003-A3 |
| **Branch** | `phase/2-a3-verification` |
| **Pull request** | [#10](https://github.com/CosmicLang/cosmic-lang/pull/10) — **DRAFT, do not merge** (owner review required) |
| **Stage** | Stage A only (lexical and syntactic frontend). **Stage B is not authorized and was not started.** |
| **Outcome** | Harness, probes, measurements, traceability and robustness evidence delivered as authorized. Oracle-dependent acceptance conditions are **NOT MEASURED** (no legacy oracle available). **No full Stage A acceptance is claimed**; see §K. |

All A3 commits carry DCO `Signed-off-by:` trailers. No legacy GPL-3.0
material was copied, imported or mechanically translated, and the legacy
oracle was never downloaded, vendored or cloned by any tooling or CI.

---

## A. Baseline

| condition | evidence | result |
|---|---|---|
| A2 merge on `main` | PR #9 merged at `3af9692dc49097fff84b769d90f3525daa26d9d0` | PASS |
| Post-merge CI on the exact SHA | CI run `39074513579`, success | PASS |
| Branch base | `phase/2-a3-verification` created from the merge commit | PASS |
| `PARSE005` and `NESTING_LIMIT = 512` | `crates/cosmic-syntax/src/parser/mod.rs` (`NESTING_LIMIT: u32 = 512`); unchanged by A3 | PASS |
| Provenance, DCO, Apache-2.0, G1d records | licensing package and gate records intact | PASS |
| Discrepancy report | none found | PASS |

The A2 report is baseline context, not A3 evidence. The A2 probe generators
were not preserved in the repository, so the previously reported durations
(≈18 s for a ~150 KB type-argument workload; 291× `PARSE005` for 50 000
`if_lines` lines) are carried as *baseline context only* and were **not**
reused as measurements (§H).

## B. Branch history

| commit | subject | verification |
|---|---|---|
| `5d786c3` | Phase 2 A3: oracle-optional differential harness (cosmic-diff-harness) | CI run `39076398542` failed on the doc gate (private intra-doc links); fixed later in this branch |
| `37ecab0` | Phase 2 A3: adversarial statement regressions and boolean/null corpus fixture | the SHA recorded in the measurement record |
| `71b740f` | Phase 2 A3: reproducible measurement runner, measurements record, probe report | measurement/probe evidence |
| `3210e4b` | Phase 2 A3: production-family traceability and M-6 cross-target context | traceability and M-6 evidence |
| `5e43f96` | Phase 2 A3: use slice contains in the adversarial soup regression | clippy `-D warnings` fix |
| `e918420` | Phase 2 A3: fix rustdoc private intra-doc links in the harness | CI run `39077394704`, success (push) |
| report commit | this report plus the register updates (docs-only) | CI run recorded in the verification addendum |

Six reviewable code/evidence commits plus the report commit. Nothing was
merged, no protection setting was changed, and no Stage B work exists on the
branch.

## C. Harness design

The deliverable is the workspace member `tools/diff-harness` (crate
`cosmic-diff-harness`, `publish = false`, four path dependencies on the
workspace crates only, **zero external dependencies**). Design points:

- **Corpus.** 49 authored fixtures embedded in the harness (lexical, syntax,
  refusals, malformed), each with an expected verdict and a pinned
  classification. Identity digest: `fnv1a64:68637c1715b1f4cb`; self-check
  reports `0` expectation failures. No legacy fixture is embedded, copied or
  translated. An external corpus directory may be supplied with `--corpus`;
  `--keep-temp` preserves the oracle workspace for inspection.
- **Oracle protocol.** The oracle is **optional and external**: program,
  whitespace-separated arguments (`{input}` is substituted) and an
  operator-asserted pin (`--oracle-pin`). It is executed read-only in an
  isolated temporary directory. Nothing in the repository or CI downloads,
  clones or vendors it.
- **Skip-with-reason.** Without an oracle every comparison is an explicit
  `skipped-no-oracle` / `oracle-unavailable` record and the process exits `3`.
  No result is fabricated, and `--oracle-pin` is recorded as an operator
  assertion only (the harness cannot verify a pin).
- **Classifications.** `AgreeAccept`, `AgreeReject`, `DeclaredDivergence`,
  `UnsupportedFamily`, `RustRejectionUnconfirmed`,
  `LegacyRejectionUnconfirmed`, `OracleUnavailable`.
- **Resource hygiene.** Oracle stdout/stderr are capped at 4096 bytes each
  (`CAPTURE_LIMIT`), the run is bounded by a timeout and killed on overrun, and
  reader threads get a bounded grace (`READER_GRACE`, 1 s) before being
  detached so a descendant holding the pipe cannot hang the harness (a real
  hang found and fixed during A3; see §G).
- **Determinism.** The JSON summary is deterministic (no timestamps, stable
  ordering) and pinned by tests; exit codes are `0` compared cleanly, `1`
  differences need triage, `2` harness/usage or corpus-integrity failure, `3`
  skipped (no oracle).
- **Modes.** `--list`, `--self-check`, `--emit-probes <path>`,
  `--emit-probes-json <path>`, `--measure <file> [--repeat n]`, and the
  differential run (`--corpus`, `--targets`, `--oracle`, `--json`, `--report`).
- **Tests.** One unit test evaluates and pins the probe expectations
  (34 cases); `tools/diff-harness/tests/harness_pipeline.rs` adds five
  pipeline tests (embedded-corpus self-check, synthetic-oracle agreement,
  declared divergence and triage, timeout kill, spawn failure, JSON
  determinism/escaping) plus one `#[ignore]`d real-oracle test gated on
  `COSMIC_LEGACY_ORACLE`. Synthetic `sh` oracles validate the *machinery*
  only and are never claimed as legacy evidence.

## D. Oracle availability

| check | result |
|---|---|
| `cosmcc` on `PATH` | not found |
| `COSMIC_LEGACY_ORACLE` environment variable | unset |
| local legacy checkout available to run | none |
| oracle obtained by tooling/CI | none — prohibited by the directive and by policy |

Consequence: every oracle-dependent condition in the A3 directive is reported
as **NOT MEASURED**, with the skip reason recorded by the harness and by the
probes (exit `3` for the differential run). The named completion condition is
an externally obtained, frozen oracle supplied by the owner: program, optional
arguments, and a pin assertion. With it, the ignored real-oracle pipeline test
(`cargo test -p cosmic-diff-harness --test harness_pipeline -- --ignored`) and
the differential CLI become executable and would produce the first pinned
comparison record. That condition was **not** available for this increment.

## E. Comparisons

| comparison | status | evidence |
|---|---|---|
| Legacy corpus differential (E7/G2c) | **NOT MEASURED** | no oracle; harness exits `3` with explicit skips |
| Legacy probe cross-check | **NOT MEASURED** | probe report states the oracle cross-check explicitly |
| Internal corpus self-consistency | measured | 49/49 fixtures match their pinned expectations; digest `fnv1a64:68637c1715b1f4cb` |
| Cross-authority identifier classification | measured as **context only** | `docs/audits/phase-2-a3-m6-cross-target.md`: CPython 3.9.6 and Node v26.12.0 classify the M-7 fragments; JDK stubs are non-functional on this host (reported `n/a`) |
| M-6 legacy target re-runs (`--target js` / `--target script`) | **NOT MEASURED** | the instrument needs the legacy compiler; unavailable |

The cross-authority table is **not** the M-6 instrument and supports no
legacy-compatibility claim. Its measured boundary differences are recorded in
§F and §G.

## F. M-7 and escape measurements

`docs/audits/phase-2-a3-probes.md` records **34 executed probes, 34 passed,
0 failed** (12 identifier-classification, 8 escape-contract, 14
known-divergence cases). The failures counter would have blocked the branch;
it is `0`.

**M-7 identifier classes (accepted authority).** One identifier token:
`camelCase`, `_`, `$`, `a1b`, `café`, `Ωmega`, and the non-BMP Deseret pair
`𐐀𐐨`. Boundary behaviour measured and pinned:

- digit-leading `1abc` splits into integer literal `1` plus identifier `abc`
  (longest valid prefix, no diagnostic);
- a combining mark after an identifier (`e` + U+0301) is **not** an
  identifier part — `e` is an identifier and the mark is an error region with
  `LEX005`;
- a leading combining mark is `LEX005` then an identifier;
- emoji are refused with `LEX005`;
- normalization is not applied: precomposed `café` and decomposed
  `cafe` + U+0301 are classified differently;
- `line_col` columns are 1-based scalar-value columns (`😀x` puts `x` at
  `1:2`).

**Escape and literal contracts.** `\uXXXX` (four hex digits) is accepted;
braced `\u{...}` is refused (`LEX007`); an unknown escape keeps the literal
as an error-marked `StringLiteral` (`LEX008`); triple-quoted forms are refused
(`LEX009`); lone surrogate escapes are errors (`LEX010`) while a surrogate
pair is one clean literal; empty and unterminated char literals produce
`Error` regions (`LEX003`/`LEX004`); multi-character char literals and
surrogate-pair escapes in char position remain `CharLiteral` tokens carrying
`[LEX004]`.

**Declared divergences pinned by the probes.** `?` is a valid nullable token;
`~` is unexpected (`LEX005`); `1.0` is a `double literal`; `10L` is a
`long literal`; `1.0L` splits into a double literal plus identifier `L`;
`0x1F3G` parses as a declaration plus expression statement `G` with no
diagnostic; a nested function body after a multiline block comment does not
attach but produces no diagnostic (it is a separate block statement).

## G. Unexpected mismatches and corrections

No frontend defect was found. Four corpus expectations were corrected to the
measured, contract-consistent behaviour (all documented in the harness and in
§F), and one harness defect was found and fixed:

1. `?` — initially treated as an unexpected character; measured as a valid
   token. The two unexpected-character fixtures now use `~`.
2. Nested function body across a multiline comment — expected to attach with a
   diagnostic; measured as a separate block statement with **no** diagnostic.
   Expectation corrected to `recognised`.
3. `val x = 0x1F3G` — expected malformed; measured as a declaration plus
   expression statement `G` with no diagnostic (longest-prefix numeric
   lexing). Expectation corrected.
4. Empty/unterminated char literals — measured as `Error` regions with
   `LEX003`/`LEX004` rather than char-literal diagnostics. Fixtures aligned.

Harness defect: after the timeout kill, reader threads could stay blocked on
pipes held by killed descendants (a real hang observed while building the
pipeline tests). Fixed by bounded reader grace (1 s) and detaching the
readers; the synthetic timeout oracle now uses `exec sleep 30` so the kill
closes the pipe. The timeout-kill path is covered by a pipeline test.

No grammar weakening was accepted: `PARSE005` semantics and
`NESTING_LIMIT = 512` are untouched, and no diagnostic code or meaning was
changed. No legacy mismatch was observable (no oracle).

## H. Robustness and performance

Method: `target/release/cosmic-diff-harness --measure` on generated,
authored inputs; warm latency is the per-run sample set inside one process,
cold invocation is a separate process measured by `/usr/bin/time -l`; cold and
warm results are reported separately and never averaged. Host: Apple M2, 8
cores, 8 GiB RAM, macOS 27.0.1 (`arm64`); rustc 1.99.1, cargo 1.99.1; release
profile; `sha` `37ecab0` with the A3 tooling uncommitted at measurement time
(listed in the measurement caveat). Full record:
`docs/audits/phase-2-a3-measurements.md` and the JSON next to it.

Warm throughput/latency (p50, per-input sample set):

| input | bytes | runs | p50 | p95 | p50 throughput |
|---|---|---|---|---|---|
| `valid_small` | 1.0 KiB | 30 | 0.07 ms | 0.14 ms | 15.1 MiB/s |
| `valid_medium` | 256.0 KiB | 30 | 15.63 ms | 19.38 ms | 16.0 MiB/s |
| `valid_large` | 4.0 MiB | 5 | 194.68 ms | 223.37 ms | 20.5 MiB/s |

All valid inputs: accepted, lossless and reprinting byte-for-byte.

Adversarial workloads (bounded; `accepts=False` means the input legitimately
carries diagnostics/refusals, `lossless=True` means the CST still reproduces
the source byte-for-byte):

| workload | bytes | p50 | max | peak RSS | cold wall | note |
|---|---|---|---|---|---|---|
| `if_lines_50k` | 537.1 KiB | 20.00 ms | 21.37 ms | 84.4 MiB | 0.02 s | per-trip `PARSE005` workload (A2 shape) |
| `deep_type_arguments_50k` | 146.5 KiB | 5.67 ms | 6.26 ms | 30.6 MiB | 0.00 s | 50 000 nested type arguments; guard workload |
| `type_decl_lines_50k` | 1.5 MiB | 68.76 ms | 73.24 ms | — | — | 1 250 013 nodes |
| `parens_1m` | 1.9 MiB | 352.90 ms | 355.77 ms | 382.5 MiB | 0.33 s | 1 000 000 nested parentheses; guard workload |
| `statement_soup_10k` | 97.7 KiB | 4.03 ms | 4.89 ms | — | — | mixed malformed soup |

- **Zero panics and zero timeouts** across all measured runs; timeouts were
  bounded per run and would have been recorded as such.
- **Dependency exposure:** 0 external packages (workspace packages: 6).
- **Memory observation, not a gate:** the 4 MiB recognised input peaks at
  585.9 MiB (≈146× input size) with the full CST retained. Stage A declares no
  threshold; this is recorded for Stage B attention.
- **A2 comparison:** the A2 figures (≈18 s; 291× `PARSE005`) are baseline
  context only. Their generator was not preserved; the A3 generators are new
  and the numbers are not comparable. All A3 adversarial runs are bounded and
  deterministic, and the adversarial regressions in
  `crates/cosmic-syntax/tests/adversarial.rs` assert bounded diagnostics,
  determinism and lossless reprint; the recursion suite runs the extreme case
  in an isolated child process so an abort cannot take the suite with it.

## I. Traceability

`docs/audits/phase-2-a3-traceability.md` maps **all 122** productions of the
machine-checked evidence map (`docs/spec/grammar-evidence-map.md`, G13):
**111 recognised, 10 unsupported, 1 rejected**, grouped into 20 families.
Every referenced integration test, source unit test, harness fixture and probe
is machine-checked to exist by `tools/a3-traceability.py`; the generator fails
if the map shape, the 111/10/1 counts, the family coverage or an evidence
reference is invalid. Refusal families are verified as refusals; F13
(class members, constructors, fields, modifiers) is the single family marked
**partially verified** (family-level evidence only). The document adds no
acceptance beyond the G1-L/G1-S decision packages.

## J. Measured versus unmeasured

| item | status | reason / evidence |
|---|---|---|
| M-7 identifier-class probes | **measured** | 34/34 probes pass; boundaries listed in §F |
| Escape and literal contracts | **measured** | probes + lexer conformance tests |
| Frontend throughput/latency (plan M4/M5) | **measured** | host-specific figures in §H |
| Peak RSS (plan M6) | **measured** | `/usr/bin/time -l`, per input |
| Dependency exposure (plan M1) | **measured** | 0 external packages |
| Panic/ICE incidence (plan M8) | **measured as 0 observed** | measured corpus only; not a claim over all inputs |
| Legacy differential / per-target conformance (plan M10, E7, G2c) | **NOT MEASURED** | no oracle available; harness skips with reason and exits `3` |
| M-6 legacy target re-runs | **NOT MEASURED** | no legacy compiler; cross-authority context only |
| Test coverage (plan M2) | **NOT MEASURED** | no coverage tool installed; adding one is a dependency decision the plan defers; per-suite test totals are the substitute evidence |
| Complexity/duplication (plan M3) | **NOT MEASURED** | no analyzer installed and no threshold declared |
| Artifact size (plan M7) | **NOT MEASURED** | no artifact exists in Stage A |
| Incremental build time (plan M9) | **NOT MEASURED** | out of A3 scope; no edit-build tooling added |
| Cross-machine comparability | **NOT MEASURED** | single declared host; the protocol forbids comparing across machines |
| A2 duration reproduction | **NOT MEASURED** | A2 generators were not preserved; A3 uses new, documented generators |

No unmeasured result is claimed settled anywhere in this report.

## K. Gate assessment (Stage A scope)

The G2-family gates are assessed **per authorized stage scope** only. Stage A
covers the lexical and syntactic frontend; Stage B components have not been
authorized, are excluded from these verdicts, and were not implemented.

| gate | assessment | basis |
|---|---|---|
| **G2** | **PARTIALLY MET — not claimable as satisfied** | E1 (lexer), E2 (lossless CST), E3 (recovering parser) and E8 (properties/fuzz budget) are evidenced by the passing suites, the probes and the measurements. E7 (differential harness) is delivered and its machinery is tested, but the gate also requires the harness be runnable against a separately obtained frozen oracle; that end-to-end condition is **NOT MEASURED**. |
| **G2b** | **OUT OF SCOPE — pending** | E6 is the semantic feature-gate requirement; Stage B is not authorized and no semantic component exists. |
| **G2c** | **PENDING — oracle unavailable** | The harness records target, exit status and reproducible input, and records missing targets as explicit skips (NOT VERIFIED). The condition requiring a run against a separately obtained frozen oracle was not met: no oracle exists in this environment and none may be downloaded. |
| **G2d** | **MET within the measurable Stage A scope** | E9 evidence: actual measurements with reproducible commands, recorded protocol fields, and an explicit list of open evidence gaps; no unmeasured result claimed settled. |
| **G2e** | **Evidence complete; CI verdict recorded in §M** | E8 and E10 evidence (this report, the artifacts, the register updates) are present; CI on the exact final SHA is verified after this report lands and recorded in the verification addendum. |

Semantic-only conditions (E4/E5/E6, G1-T, and the open S1/S5/S13-era semantic
questions) remain **outside this scope and pending**. Stage A completion does
not satisfy Stage B conditions, and vice versa.

## L. Test totals

The five prescribed commands were run on the exact report tree (Rust 1.99.1,
zero external dependencies); all passed:

| command | result |
|---|---|
| `python3 tools/spec-consistency.py` | all checks pass (G0–G13) |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace --all-targets` | **243 passed, 0 failed, 2 ignored** across 28 test binaries |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | clean |

`cargo test --workspace` (without `--all-targets`) additionally runs the four
doc-tests: **247 passed, 0 failed, 2 ignored** in total.

| crate / suite | passed | ignored |
|---|---|---|
| `cosmic-cli` (integration) | 10 | 0 |
| `cosmic-diagnostics` (unit + render) | 19 | 0 |
| `cosmic-diff-harness` (probe unit test + pipeline) | 6 | 1 |
| `cosmic-lexer` (unit + 7 integration suites) | 93 | 0 |
| `cosmic-source` (unit + source map) | 29 | 1 |
| `cosmic-syntax` (unit + 10 integration suites) | 86 | 0 |
| doc-tests (separate local run) | 4 | 0 |
| **total** | **247** | **2** |

The two ignored tests are ignored **by design**, with recorded reasons: the
harness real-oracle pipeline test (requires an externally supplied oracle;
ordinary CI never runs it) and the source-map oversized-file test (creates a
5 GiB sparse file; manual run only). The `cosmic-syntax` recursion suite
re-executes its extreme case in an isolated child process; it is counted once
in the totals above.

## M. Final SHA and CI

| item | record |
|---|---|
| Pre-report head | `e9184204af6fb70b23cdee8fed505d77aa96d977` — CI run `39077394704`, **success** (push); the PR-triggered suite is part of the same checks page |
| Report commit | the commit carrying this report (docs-only) — its exact SHA and CI run id are recorded in the verification addendum appended immediately after CI completes |
| Intermediate failure | `5d786c3` CI run `39076398542` failed on the rustdoc doc gate (private intra-doc links); fixed in `e918420` — evidence that the prescribed CI gate is effective |
| Final verification | CI must be green on the exact head of PR #10; the PR checks page is the live record, and the addendum pins the run id observed at verification time |

The branch was pushed before this report and CI was observed on the pushed
SHA; the report and register updates are the final change set.

## N. Pull request

- URL: https://github.com/CosmicLang/cosmic-lang/pull/10
- State: **DRAFT**; base `main`; not merged; no auto-merge; no branch
  protection change; no review approval requested implicitly.
- The owner decides whether to merge, request changes, or return the
  increment. Nothing in this report authorizes a merge.

## O. Remaining owner decisions

1. **PR #10 disposition** — review and either approve a future merge or return
   the increment. Engineering does not merge it.
2. **Oracle provisioning** — provide a separately obtained, frozen legacy
   oracle (program, arguments, pin assertion) to execute E7/G2c end-to-end and
   the M-1…M-7 legacy cross-checks. Until then those remain NOT MEASURED.
3. **Gate reading** — accept or reject §K: G2 partially met (E7 pending),
   G2b out of scope, G2c pending, G2d met, G2e evidence complete with the CI
   verdict recorded.
4. **M-7 / D-L3 settlement** — engineering position: the identifier boundary
   is measured against the accepted authority only; D-L3 should **not** be
   called settled against legacy behaviour until an oracle comparison exists.
5. **Coverage and complexity tooling (plan M2/M3)** — decide whether a future
   increment may add measurement tooling, or keep both NOT MEASURED.
6. **Memory observation** — the 4 MiB recognised input peaks at 585.9 MiB
   (≈146× input size). Decide whether Stage B must schedule memory work or
   whether the observation is accepted as recorded.
7. **Stage B** — remains **NOT AUTHORIZED**; G1-T entry and a separate owner
   directive are required. No semantic component, typed HIR/MIR, codegen,
   runtime or standard library work was started.
