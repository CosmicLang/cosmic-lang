# Cosmic-Lang Compiler Pipeline — Architecture

| | |
|---|---|
| **Status** | Living document — Phase 0 foundation; updated for Phase 2 Stage A (2026-10-17) |
| **Scope** | Target end-state pipeline, implemented components as of Phase 0, interfaces and invariants |
| **Related** | `docs/audits/phase-0-baseline.md`, `docs/adr/0001-language-identity.md`, `docs/spec/compatibility-matrix.md` |

---

## 1. Target pipeline (long-term)

```
Source Management          cosmic-source        (implemented in Phase 0)
  → Lexer                  cosmic-lexer         (Phase 2 Stage A: implemented — draft PR)
  → Lossless CST           cosmic-syntax        (not started)
  → AST                    cosmic-syntax        (not started)
  → Name Resolution        cosmic-sema          (not started)
  → Typed HIR              cosmic-hir           (not started)
  → MIR / CFG              cosmic-mir           (not started)
  → SSA + Optimization     cosmic-opt           (not started)
  → Code Generation        cosmic-codegen-wasm  (not started)
                           cosmic-codegen-native(not started)
  → Runtime & Packaging    cosmic-runtime       (not started)
  → Driver                 cosmic-cli           (Phase 0: CLI bootstrap only)
                           cosmic-diagnostics   (implemented in Phase 0)
```

**Rule for introducing crates:** a component is added to the workspace only
when it has a concrete contract, tests and a justified responsibility. Empty
crates are not created to mirror this diagram. As of Phase 0 the workspace
contains exactly three crates — `cosmic-source`, `cosmic-diagnostics` (which
sits beside the pipeline as a cross-cutting service) and `cosmic-cli`. Phase 2
Stage A added the first pipeline crate beyond that foundation, `cosmic-lexer`
(increment A1, §2.5); every component from `cosmic-syntax` onward remains
unstarted.

## 2. Implemented components (Phase 0 foundation; Stage A adds the lexer)

### 2.1 `cosmic-source` — source management

**Contract:** owns file identity, byte spans and line/column mapping for one
compilation session.

* `SourceMap` — registry assigning `FileId`s (1-based `NonZeroU32`, opaque,
  assigned in registration order; ids are session-local and not persisted).
  Files can be added from text or loaded from disk (`load` reports I/O
  errors, invalid UTF-8 with the first invalid byte, and oversized files).
* `SourceFile` — UTF-8 text + precomputed line index; exposes `text`,
  `line_col`, `line_index`, `line_text`, `span`, `full_span`, `text_of`.
* `Span` — validated `[start, end)` byte range of one file. **Invariant:**
  bounds are ordered, inside the file, and on UTF-8 character boundaries.
  Spans cannot be constructed outside the crate except through checked
  constructors (`SourceFile::span`, `SourceFile::full_span`), which return
  typed errors (`SpanError::{StartAfterEnd, OutOfBounds, NotCharBoundary}`).

**Design decisions:**

* All offsets are `u32`; files of ≥ 4 GiB − 1 bytes are rejected with a typed
  error instead of silently truncating. `load` enforces this from the file's
  **metadata before reading**, so an oversized file is never materialized in
  memory (the post-read check remains as defence in depth).
* Line terminators are universal: `\n`, `\r\n` and lone `\r`. A trailing
  terminator opens one final empty line (`"a\n"` has two lines).
* Columns in `LineCol` are **1-based and counted in Unicode scalar values**
  (`char`s), which is what editors show; byte offsets remain authoritative.
* A byte inside a line terminator maps to the column **one past the last
  visible character** — terminators are not columns of their own, so LF, CRLF
  and lone CR files report identical columns for the same logical position
  (`"a\r\nb"`: both terminator bytes report line 1, column 2). This keeps
  `line_col` consistent with `line_text`, which never prints the terminator,
  and guarantees the diagnostic renderer's caret never lands beyond the
  printed line.
* No panics on input data: every fallible operation returns `Option` or a
  typed error. No `unsafe`. No dependencies.

### 2.2 `cosmic-diagnostics` — structured diagnostics

**Contract:** diagnostics are *data*; rendering is a separate step.

* `Diagnostic` — severity (`Error`/`Warning`/`Note`/`Help`), stable
  `DiagnosticCode`, human message, exactly one primary `Label`, secondary
  labels, notes, structured `Suggestion`s (span + replacement + message +
  `Applicability`).
* `DiagnosticCode` — format `[A-Z]{2,8}[0-9]{3}` (same shape as the legacy
  code vocabulary, e.g. `SEM011`). **Stability policy:** a published code's
  meaning is frozen forever; codes are retired, never repurposed.
  `DiagnosticCode::new` is a `const fn` validated at compile time for
  compiler-embedded constants; `try_new` is the fallible runtime path.
* `DiagnosticBag` — insertion-ordered collection with `error_count`,
  `warning_count`, `has_errors`, and `sorted()` implementing the canonical
  reporting order (file id, start, end; stable).
* `render::human` — deterministic rustc-style text: adaptive gutter, `^`
  primary / `-` secondary markers, tab-normalized snippets, multi-line spans
  underlined on their first line, `= note:` / `= help:` trailers.
* `render::json` — deterministic single-line JSON (severity, code, message,
  labels with file/offset/line/column/primary, notes, suggestions with
  applicability). Keys always present; unresolvable locations are `null`.

**Design decisions:**

* Rendering never panics: a span that does not resolve against the rendering
  `SourceMap` degrades to a `path:bytes` location line without a snippet.
* Machine format is a small, unit-tested RFC 8259 writer inside the crate —
  no `serde` dependency in Phase 0. If structured output grows (LSP, SARIF),
  adopting `serde` is a planned review decision, not an accident.
* JSON output is validated in tests by exact-string assertions and, as
  manual evidence, by an independent parser (`python3 -m json.tool`).

### 2.3 `cosmic-cli` — `cosmic` executable (bootstrap)

**Contract:** exactly two capabilities exist in Phase 0 — `--help` and
`--version` (plus equivalent `help`/`version` words).

* Entry point `cosmic_cli::run(args, stdout, stderr) -> u8` — a pure
  function over argument strings and streams; `main` only wires stdio and
  exits with the returned code. This keeps CLI behavior testable in-process.
* Planned commands (`check build run test fmt doc lsp`) **fail loudly**:
  exit code 2 with "not implemented yet"; unknown commands/options also
  exit 2. Nothing ever reports success without doing its documented job.
* Exit-code contract: `0` success; `2` usage error / unknown / unimplemented;
  `1` is reserved for compilation failures once compilation exists.

### 2.4 `cosmic-cli` is not yet wired to the libraries

Deliberate: the CLI does not depend on `cosmic-source`/`cosmic-diagnostics`
until there is a command that actually uses them (the first will be `check`).
The crates are not connected just to decorate the dependency graph.

### 2.5 `cosmic-lexer` — lexical frontend (Phase 2 Stage A, increment A1)

**Contract:** total, lossless tokenization under the accepted G1-L lexical
contract (`docs/spec/g1-l-lexical-decision-package.md`, D-L1–D-L10).

* `lex(&SourceFile) -> LexedFile` — the element stream (`Bom`, `Whitespace`,
  `LineTerminator`, `LineComment`, `BlockComment`, `Token`, `Error`) partitions
  the file byte-for-byte; `is_lossless` and `reprint` reproduce the original
  bytes, and the property tests exercise arbitrary malformed input.
* Diagnostics are `Diagnostic`s with the minted `LEX001`–`LEX010` codes
  (`docs/spec/diagnostic-codes.md` §6.2); recovery is local, deterministic and
  never cascades (one diagnostic per construct).
* Unicode: scalar-value columns, `char::is_alphabetic`/`is_alphanumeric`
  identifier classification (the M-7 measurement debt remains recorded), exact
  byte spans including astral characters.
* No panics on any input, no `unsafe`, no dependencies.

## 3. Planned interfaces between phases (informative)

The boundaries below are fixed *semantically* now (typed, no stringly-typed
coupling), but their exact Rust signatures land with their components:

1. **Lexer → CST:** tokens carry `Span` values from `cosmic-source`; lexical
   errors are `Diagnostic`s with `LEX###` codes. The lexer must be total on
   arbitrary bytes: every byte sequence terminates with tokens or a
   diagnostic, never a panic.
2. **CST:** lossless, including trivia; parsing any token stream is total.
3. **HIR/sema:** name resolution and typing attach to stable ids; all errors
   are structured `Diagnostic`s with stable codes.
4. **MIR:** CFG with SSA later; the MIR interpreter is the semantic reference
   (directive §10) before any optimized backend is trusted.
5. **Backends:** WASM/WASI first, then native (Cranelift vs LLVM evaluated
   later). Directive 001 does not authorize any backend work.

## 4. Cross-cutting invariants

1. `unsafe_code = "forbid"` workspace-wide. If unsafe ever becomes necessary,
   it requires a written safety argument and dedicated tests.
2. No panics triggered by input data anywhere; `panic!` may only encode
   programming errors that tests catch (e.g. invalid diagnostic-code
   literals).
3. Determinism: identical inputs must produce byte-identical rendered
   diagnostics; registration order defines `FileId`s; `DiagnosticBag::sorted`
   defines one canonical order.
4. No component reports success without executing its documented operation.
5. Diagnostics never require a panic to *report* malformed input; malformed
   input flows data-first (Result/Option), and the CLI terminates with a
   defined exit code.
6. Dependency policy: standard library only in Phase 0; each new dependency
   needs a justification recorded in review (see toolchain policy).

## 5. Explicit non-goals of Phase 0

Lexer (still outside Phase 0; implemented in Phase 2 Stage A — §2.5), parser,
name resolution, typing, HIR/MIR, optimizer, code generation
(wasm/native/JVM/JS), runtime, package manager (`Cosmic.toml`/`Cosmic.lock`),
GC/ownership system, and any language-semantics decision not backed by legacy
evidence. Apart from the Stage A lexer, none of these are stubbed; they do not
exist yet.
