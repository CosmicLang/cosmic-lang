# Phase 1 Semantic Decision Register

| | |
|---|---|
| **Status** | **Proposed — Phase 1 draft.** Not approved. |
| **Phase** | 1 (formal specification) |
| **Directive** | ENGINEERING DIRECTIVE 002, deliverable D4 |
| **Scope** | One record per semantic question S1–S18 |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |

## Status vocabulary

| Status | Meaning |
|---|---|
| **OPEN** | The question is identified, evidence has been gathered, **no decision has been made**. Phase 2 must not implement a guessed answer. |
| **PROPOSED** | This register recommends an answer. **It is not in force** until the owner accepts it. |
| **EVIDENCE-SETTLED** | The legacy behaviour is established by E1 evidence and no choice is needed to describe it. Whether to *keep* it may still be a separate question. |
| **BLOCKED-ON-OWNER** | The decision requires an owner act that cannot be taken by engineering: a licence choice, a naming choice, a scope choice, or legal review. |
| **BLOCKED-ON-MEASUREMENT** | The decision needs executed differential evidence that does not exist yet. Phase 2 produces it. |

**No record in this register claims owner approval.** Records whose outcome is a
product, legal or naming choice are **BLOCKED-ON-OWNER** and stay open.

## The register

| # | Record | Question | Status |
|---|---|---|---|
| S1 | [S1-nested-generics.md](S1-nested-generics.md) | Nested type-argument resolution, generic inference (including the parameter/return-type inference boundary), bounds and variance syntax, overload resolution | **OPEN** (partly BLOCKED-ON-MEASUREMENT) |
| S2 | [S2-nullability.md](S2-nullability.md) | `T` / `T?` conversions, soft casts, nullability of type parameters, the legacy structural fallback | **OPEN** |
| S3 | [S3-portukof.md](S3-portukof.md) | PortuKof surface: which tokens differ, mixed-surface programs, whether it is in scope at all | **BLOCKED-ON-OWNER** |
| S4 | [S4-kofscript.md](S4-kofscript.md) | KofScript `.ks`: the wrapping transformation and its execution semantics | **BLOCKED-ON-OWNER** |
| S5 | [S5-pattern-matching.md](S5-pattern-matching.md) | Pattern breadth, guards, exhaustiveness, fallthrough, labelled `break`/`continue` | **OPEN** |
| S6 | [S6-concurrency.md](S6-concurrency.md) | `spawn`/`await` typing, failure propagation, cancellation, per-target scheduling guarantees | **OPEN** |
| S7 | [S7-numeric-semantics.md](S7-numeric-semantics.md) | Overflow, literal promotion, division, float formatting, bitwise complement | **OPEN** |
| S8 | [S8-string-semantics.md](S8-string-semantics.md) | Escapes, silent unknown escapes, triple-quote policy, interpolation | **OPEN** (one part PROPOSED) |
| S9 | [S9-stdlib-boundary.md](S9-stdlib-boundary.md) | Language versus runtime, properties versus methods, `package`/`import` rules, `string` dual status | **OPEN** |
| S10 | [S10-reserved-keywords.md](S10-reserved-keywords.md) | `entity`/`extern`/`generated`/`audited`/`unique`, annotations, FFI, entity/query DSL, contextual declarations | **OPEN** |
| S11 | [S11-evaluation-order.md](S11-evaluation-order.md) | Evaluation order (arguments, receiver, operands, short-circuit), assignment associativity, scoping, shadowing, nested-function capture | **BLOCKED-ON-MEASUREMENT** |
| S12 | [S12-equality.md](S12-equality.md) | Equality: reference versus structural per type kind, `NaN`/`-0.0`, records, enums, strings, map-key and set contracts | **OPEN** (one part PROPOSED) |
| S13 | [S13-unicode.md](S13-unicode.md) | Identifier classes, normalisation, confusables, and the legacy/new column divergence | **PROPOSED** |
| S14 | [S14-trivia.md](S14-trivia.md) | Trivia preservation, terminator normalisation, block-comment nesting | **PROPOSED** |
| S15 | [S15-feature-gates.md](S15-feature-gates.md) | Feature-gate vocabulary, severities, error-recovery contract | **PROPOSED** |
| S16 | [S16-memory-strategy.md](S16-memory-strategy.md) | Ownership+borrowing, GC or ARC | **BLOCKED-ON-OWNER** (see ADR-0005) |
| S17 | [S17-abi-versioning.md](S17-abi-versioning.md) | Runtime ABI versioning and the runtime → frontend dependency ban | **BLOCKED-ON-OWNER** (see ADR-0003, ADR-0004, ADR-0005) |
| S18 | [S18-mir-invariants.md](S18-mir-invariants.md) | Typed-IR invariants verified on entry to MIR, and the MIR verifier's obligations | **PROPOSED** |

## How a record is decided

1. Engineering writes the record with evidence, options and a recommendation.
2. If the outcome is technical and evidenced, the owner accepts or rejects it.
3. If the outcome is a product, legal or naming choice, the record stays
   **BLOCKED-ON-OWNER** until the owner acts. Engineering must not fill the gap.
4. A record that would change observable behaviour of existing `.kf` programs
   must state the compatibility consequence under the C0–C3 vocabulary of
   `docs/spec/compatibility-policy.md`.

## Gate mapping

| Gate | Requirement | Covered by |
|---|---|---|
| **G1** | Grammar and specification evidence complete | D1, D2 |
| **G1b** | All 18 questions decided or explicitly owner-dependent | this register |
| **G1c** | Language identity, extension and compatibility decisions prepared for approval | ADR-0001, ADR-0002, D5 |
| **G1d** | Licensing decision presented, not falsely resolved | `docs/audits/phase-0-licensing-matrix.md`, `docs/audits/phase-1-licensing-recommendation.md` |
| **G1e** | CI green and governance protections verified | `docs/audits/phase-1-specification-report.md` |
