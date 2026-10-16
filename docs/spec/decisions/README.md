# Phase 1 Semantic Decision Register

| | |
|---|---|
| **Status** | **Proposed — Phase 1 draft.** Not approved. |
| **Phase** | 1 (formal specification) |
| **Directive** | ENGINEERING DIRECTIVE 002, deliverable D4 |
| **Scope** | One record per semantic question S1–S18 |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |

## Normative status vocabulary (authoritative)

| Status | Meaning |
|---|---|
| **VERIFIED LEGACY BEHAVIOR** | Demonstrated fact at the pinned legacy revision, with specified target and command; never automatically a rule for Cosmic-Lang. |
| **PROPOSED NORMATIVE RULE** | Engineering recommendation awaiting an explicit owner decision. |
| **ACCEPTED NORMATIVE RULE** | Explicitly accepted by the owner, with an acceptance record. **None is granted by Directive 002-R1.** |
| **OPEN** | An unresolved technical or owner-dependent question; a consumer must gate its exact unsupported operation. |
| **DEFERRED** | Outside the presently authorized component; must be resolved before its consumer. |
| **NOT VERIFIED** | Insufficient or non-discriminating evidence; no conclusion inferred. |

Old labels such as EVIDENCE-SETTLED, BLOCKED-ON-MEASUREMENT and CLOSED are
historical workflow labels. They carry **no normative approval**.
For component-specific gating see `docs/spec/implementation-gates.md`.
Verified legacy behavior is evidence; proposed correct behavior is a different
decision. JVM probes do not establish cross-target portability.

## The register

| # | Record | Question | Status |
|---|---|---|---|
| S1 | [S1-nested-generics.md](S1-nested-generics.md) | Nested type-argument resolution, generic inference (including the parameter/return-type inference boundary), bounds and variance syntax, overload resolution | **OPEN** · VERIFIED LEGACY BEHAVIOR (partial JVM probes) · NOT VERIFIED (tie-break) |
| S2 | [S2-nullability.md](S2-nullability.md) | `T` / `T?` conversions, soft casts, nullability of type parameters, the legacy structural fallback | **OPEN** · VERIFIED LEGACY BEHAVIOR (soft-cast JVM probe) · PROPOSED NORMATIVE RULE |
| S3 | [S3-portukof.md](S3-portukof.md) | PortuKof surface: which tokens differ, mixed-surface programs, whether it is in scope at all | **OPEN** (owner scope/legal choice) |
| S4 | [S4-kofscript.md](S4-kofscript.md) | KofScript `.ks`: the wrapping transformation and its execution semantics | **OPEN** (owner scope/legal choice) |
| S5 | [S5-pattern-matching.md](S5-pattern-matching.md) | Pattern breadth, guards, exhaustiveness, fallthrough, labelled `break`/`continue` | **OPEN** · NOT VERIFIED (exhaustiveness, fallthrough) |
| S6 | [S6-concurrency.md](S6-concurrency.md) | `spawn`/`await` typing, failure propagation, cancellation, per-target scheduling guarantees | **OPEN** (typing) · DEFERRED (runtime) |
| S7 | [S7-numeric-semantics.md](S7-numeric-semantics.md) | Overflow, literal promotion, division, float formatting, bitwise complement | **OPEN** · NOT VERIFIED (division by zero and promotion) |
| S8 | [S8-string-semantics.md](S8-string-semantics.md) | Escapes, silent unknown escapes, triple-quote policy, interpolation | **VERIFIED LEGACY BEHAVIOR** (raw newline, unknown escape, triple-quote JVM) · **PROPOSED NORMATIVE RULE** (invalid-escape rejection) · **OPEN** |
| S9 | [S9-stdlib-boundary.md](S9-stdlib-boundary.md) | Language versus runtime, properties versus methods, `package`/`import` rules, `string` dual status | **OPEN** · NOT VERIFIED (member boundary) |
| S10 | [S10-reserved-keywords.md](S10-reserved-keywords.md) | `entity`/`extern`/`generated`/`audited`/`unique`, annotations, FFI, entity/query DSL, contextual declarations | **OPEN** · NOT VERIFIED (contextual declarations, FFI) |
| S11 | [S11-evaluation-order.md](S11-evaluation-order.md) | Evaluation order (arguments, receiver, operands, short-circuit), assignment associativity, scoping, shadowing, nested-function capture | **VERIFIED LEGACY BEHAVIOR** (JVM order, scoping, capture rejection) · **PROPOSED NORMATIVE RULE** · **OPEN** (capture intent) |
| S12 | [S12-equality.md](S12-equality.md) | Equality: reference versus structural per type kind, `NaN`/`-0.0`, records, enums, strings, map-key and set contracts | **VERIFIED LEGACY BEHAVIOR** (JVM strings, records, classes, floats) · **NOT VERIFIED** (enum and map/set) · **OPEN** |
| S13 | [S13-unicode.md](S13-unicode.md) | Identifier classes, normalisation, confusables, and the legacy/new column divergence | **VERIFIED LEGACY BEHAVIOR** (JVM identifiers/UTF-16 columns) · **PROPOSED NORMATIVE RULE** (scalar columns) · **OPEN** |
| S14 | [S14-trivia.md](S14-trivia.md) | Trivia preservation, terminator normalisation, block-comment nesting | **PROPOSED NORMATIVE RULE** (lossless CST) |
| S15 | [S15-feature-gates.md](S15-feature-gates.md) | Feature-gate vocabulary, severities, error-recovery contract | **PROPOSED NORMATIVE RULE** (diagnostic gate contract) |
| S16 | [S16-memory-strategy.md](S16-memory-strategy.md) | Ownership+borrowing, GC or ARC | **DEFERRED** · **OPEN** (owner choice) |
| S17 | [S17-abi-versioning.md](S17-abi-versioning.md) | Runtime ABI versioning and the runtime → frontend dependency ban | **DEFERRED** · **OPEN** (owner choice) |
| S18 | [S18-mir-invariants.md](S18-mir-invariants.md) | Typed-IR invariants verified on entry to MIR, and the MIR verifier's obligations | **PROPOSED NORMATIVE RULE** (typed IR and verifier invariants) |

## How a record is decided

1. Engineering writes the record with evidence, options and a recommendation.
2. If the outcome is technical and evidenced, the owner accepts or rejects it.
3. If the outcome is a product, legal or naming choice, the record stays
   **OPEN (owner action)** until the owner acts. Engineering must not fill the gap.
4. A record that would change observable behaviour of existing `.kf` programs
   must state the compatibility consequence under the C0–C3 vocabulary of
   `docs/spec/compatibility-policy.md`.

## Gate mapping

The component gate framework (G1-L/G1-S/G1-T/G1-M/G1-R) was **accepted by the
owner** in Directive 002-R3 (2026-10-16) and replaces the flat `G1b`. Acceptance
of the framework approves no language rule and does not authorize Phase 2;
S1–S18 remain individually authoritative.


| Gate | Requirement | Covered by |
|---|---|---|
| **G1** | Grammar and specification evidence complete | D1, D2 |
| **G1b (superseded)** | Flat all-decisions prerequisite was cyclic; **superseded by the owner-accepted component entry gates** G1-L/G1-S/G1-T/G1-M/G1-R (002-R3) | `docs/spec/implementation-gates.md` |
| **G1c** | Language identity, extension and compatibility decisions prepared for approval | ADR-0001, ADR-0002, D5 |
| **G1d** | Licensing decision presented, not falsely resolved | `docs/audits/phase-0-licensing-matrix.md`, `docs/audits/phase-1-licensing-recommendation.md` |
| **G1e** | CI green and governance protections verified | `docs/audits/phase-1-specification-report.md` |
