# S15 — Feature Gates, Severities and the Error-Recovery Contract

| | |
|---|---|
| **Status** | **PROPOSED NORMATIVE RULE** — owner acceptance outstanding |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; Phase 2 implementation |
| **Related** | `docs/spec/language-spec.md` §9, `docs/spec/compatibility-policy.md` §3, `docs/spec/diagnostic-codes.md` (D9), S6, S14 |

## Question

1. **Q1** — What is the **feature-gate** vocabulary for "this construct is not
   supported on this target"?
2. **Q2** — Which conditions are **errors**, which are **warnings**, and which
   are **notes/help**?
3. **Q3** — What is the **error-recovery contract** — how many errors must a
   single run report, and what must never happen?
4. **Q4** — Are warnings **fatal** by default, or opt-in?
5. **Q5** — What must a **refusal** look like (as opposed to a failure)?

## Evidence

* **E1** — The design report (page 16) requires "declarative feature gates in
  the manifest; `unsupported` = a clear diagnostic, never silently degraded
  generation".
* **E1** — The legacy compiler already practices **honest refusal** in at least
  one backend: the WASM backend emits dedicated refusal codes with **no
  artifacts** when it cannot represent a construct (recorded in
  `docs/audits/phase-0-baseline.md` §3.6 and corroborated by the record-equality
  refusal discussed in S12). So "refuse rather than degrade" has precedent, not
  just aspiration.
* **E1** — The legacy lexer produces **one** diagnostic for an unterminated
  block comment rather than cascading (`LEX001`), and the triple-quote
  rejection deliberately skips the block so that one diagnostic lands instead of
  a cascade (`LEX008`). The legacy project has therefore **already adopted** the
  "one diagnostic per construct, no cascade" discipline.
* **E1** — The design report (page 5) sets a parser criterion: the parser "must
  recover multiple errors, never enter an infinite loop, and pass structured
  fuzzing". So multi-error recovery is a **requirement**, not a preference.
* **E1** — Phase 0's `cosmic-diagnostics` already implements the severity
  vocabulary `Error` / `Warning` / `Note` / `Help`, a stable code format, a
  canonical reporting order, and total (never-panicking) rendering. Verified in
  `docs/audits/phase-0-source-review.md` §4.
* **E1** — Phase 0's CLI already implements the **loud-failure** contract: a
  planned-but-unimplemented command exits with code 2 and says so; nothing
  reports success without doing its job (`crates/cosmic-cli/src/lib.rs`,
  verified by tests). This is the precedent for how a refusal must look.
* **E1** — The legacy diagnostics vocabulary is code-prefixed by family
  (`LEX###`, `PARSE###`, `SEM###`, `PKG###`, `COMP###`, `WASM###`, `FFI###`,
  `MEM###`, `L###`, `CONC###`).

## Options

| # | Option | Assessment |
|---|---|---|
| O1 | **A declarative capability manifest per target**, checked before code generation; an unsupported construct is a compile error with a dedicated code, never a degraded artifact | Matches the design report exactly and the legacy WASM precedent. Cost: every target must declare its capabilities, which is real work |
| O2 | **Runtime feature detection**: generate the best artifact the target allows and warn | This is precisely "silently degraded generation", which the design report forbids and which the project's principles forbid |
| O3 | **No gates**: a target either supports the whole language or is not offered | Simple, but it makes partial targets (like WASM) impossible, and the legacy project already ships one |
| O4 | **Defer** | The gate schema is needed before any backend work, which is Phase 4 — so deferring is *possible*, but the **vocabulary** must be fixed now because diagnostics reference it |

## Recommendation (PROPOSED, not in force)

**O1.**

* **Q1 — Feature-gate vocabulary.** Three states, declared per target in a
  manifest:
  | State | Meaning | Consequence |
  |---|---|---|
  | `supported` | The target honours the construct's normative semantics | Normal compilation |
  | `refused` | The target **cannot** honour the semantics | **Compile error** with a dedicated code, naming the construct and the target. No artifact is produced |
  | `unspecified` | The construct's semantics are not yet normative (an open decision record) | **Compile error**: "this construct is not yet specified". Never a silent choice |

  The third state is what makes the Phase 1 work safe: a construct whose
  decision record is OPEN is *refused*, not guessed. This is how
  `docs/spec/decisions/` connects to the compiler.
* **Q2 — Severity policy.**
  * **Error**: anything that makes the program's meaning undefined or
    unsupported — type errors, unresolved names, a `refused` feature gate, a
    construct with an OPEN decision record, malformed syntax.
  * **Warning**: a condition where the program is well-defined but suspicious —
    unused bindings, a confusable identifier (S13 Q3), an interior `/*` in a
    block comment (S14 Q3), a deprecated form.
  * **Note / Help**: context and suggestions attached to another diagnostic.
  * A **warning never changes the meaning** of a program, and no legacy program
    may become an error because of a warning.
* **Q3 — Error-recovery contract.** Normative requirements:
  1. A single run reports **multiple independent errors** (not just the first).
  2. Recovery **must terminate**: no infinite loop on any input, including
     malformed input. This is a hard requirement from the design report page 5.
  3. **One diagnostic per construct** where a cascade would otherwise occur —
     the legacy lexer's own discipline (`LEX001`, `LEX008`).
  4. No panic, ever, on any input. Phase 0 already satisfies this for the
     diagnostics layer; it is a standing requirement.
  5. Recovery must be **deterministic**: the same input yields the same
     diagnostics in the same order. Phase 0's canonical ordering already
     provides the mechanism.
  6. **Bounded work per input**: the design report page 14 requires anti-DoS
     limits. Phase 0 already enforces a 4 GiB source cap *before* reading.
* **Q4 — Warnings are not fatal by default.** A `-D warnings`-style strict mode
  may be offered by the CLI, but the default must accept a program with
  warnings, because otherwise a new warning would break existing programs — a
  compatibility hazard with no upside.
* **Q5 — A refusal is not a failure.** A refusal must: name the construct, name
  the target, name the feature gate state, state which decision record (if any)
  is open, produce **no artifact**, and exit non-zero with a code distinct from
  a plain usage error. This mirrors the Phase 0 CLI contract (exit 2 for usage
  and unimplemented; exit 1 reserved for compilation failures). **Proposed:
  `refused` uses the compilation-failure exit path**, because the program was
  submitted for compilation and was rejected on its merits.

## Compatibility consequence

* The gate schema is **C0–C3 neutral** for the source extension (currently
  unapproved; the leading proposal is `.cosmic`, per `docs/spec/extension-assessment.md`). For legacy formats it
  matters only in that a `refused` gate is a **C1 divergence** on the affected
  target — but only for targets that never fully supported the construct anyway
  (the legacy WASM backend already refuses).
* The `unspecified` state is the mechanism that keeps Phase 2 honest: it turns
  every OPEN decision record into a hard error instead of an accidental choice.
  **This is the single most important compatibility safeguard in Phase 1.**

## Open dependencies

* **D9** (`docs/spec/diagnostic-codes.md`) owns the code namespace that the gate
  diagnostics will use.
* **S6** depends on this record for the per-target capability table schema.
* The manifest file format itself is a Phase 4 concern (and `Cosmic.toml`'s
  naming is ADR-0003's).
