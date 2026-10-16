# ENGINEERING DIRECTIVE 002 — Proposal (Specification Only)

> **Status: PROPOSED — not authorised.** This document is the review's
> recommended text for Directive 002. It is produced by Directive 001-R1 §4/§11J
> and requires explicit owner authorisation before any of it is executed.
> Directive 001-R1 §10 forbids starting Directive 002.

---

## COSMIC-LANG — ENGINEERING DIRECTIVE 002
### Phase 1 — Formal Specification, Grammar, Typing Contracts and Semantic Ambiguities

**Role:** Principal Rust Compiler Engineer and Language Specification Author
**Reasoning:** Maximum available
**Repository:** `https://github.com/CosmicLang/cosmic-lang.git`
**Expected base branch:** `main` (after PR #1 is accepted by the owner)
**Legacy reference:** `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5)
**Inputs from Phase 0:** `docs/audits/phase-0-baseline.md`,
`docs/audits/phase-0-design-conformance.md`, `docs/audits/phase-0-source-review.md`,
`docs/audits/phase-0-licensing-matrix.md`, `docs/adr/0001-language-identity.md`,
`docs/spec/compatibility-matrix.md`, `docs/spec/compatibility-policy.md`

---

## 1. Mission

Phase 0 delivered the foundation, the legacy baseline and a verified
design-conformance audit. Phase 1 converts the **evidenced behaviour** of the
legacy language into a **written specification** that later phases can be held
to.

Directive 002 is a **specification-only** directive. It authorises documents,
decision records and test plans. It does **not** authorise implementing the
lexer, the parser, the CST, the HIR, the MIR or any backend.

**Explicitly out of scope:** `cosmic-lexer`, `cosmic-syntax`, `cosmic-hir`,
`cosmic-sema`, `cosmic-mir`, `cosmic-opt`, `cosmic-codegen-*`,
`cosmic-runtime`. Creating any of these crates requires a separate, later
directive. This resolves the conflict between the design report's page 20
backlog item ("protótipo lexer e parser") and its own page 18 phase plan: **the
phase plan wins.**

**Prerequisite:** the owner's licensing decision
(`docs/audits/phase-0-licensing-matrix.md`). Phase 1 may begin with the licence
decision open, but no Phase 1 deliverable may copy legacy code, fixtures or
prose.

---

## 2. Authorised scope

1. A formal specification of the language as evidenced by the legacy corpus and
   the legacy source at the frozen SHA.
2. A grammar (EBNF or equivalent) for the `.csmx` surface.
3. Typing contracts: what is typed where, the inference boundary, and the
   invariants of the typed representation.
4. Resolution of the enumerated semantic ambiguities, one decision record each.
5. Approval-ready ADRs, including ADR-0001.
6. A source-format compatibility decision per
   `docs/spec/compatibility-policy.md`.
7. A measurement plan for the metrics the design report requires (page 2, page 19).
8. The test-strategy ladder and the differential-testing plan.

---

## 3. Deliverables

| # | Deliverable | Path | Content |
|---|---|---|---|
| D1 | Language specification | `docs/spec/language-spec.md` | Normative description of the evidenced surface: lexical structure, declarations, statements, expressions, scoping, evaluation order, exceptions, concurrency, numeric semantics, string semantics |
| D2 | Grammar | `docs/spec/grammar.ebnf` | Formal grammar for `.csmx`, with a legacy-compatibility note per production |
| D3 | Typing contracts | `docs/spec/typing-contracts.md` | Type representation, inference boundary, coercion/nullable rules, generic instantiation, overload resolution, invariant list |
| D4 | Semantic ambiguity decisions | `docs/spec/decisions/` (one file per question) | One decision record per question in §5, each with evidence, options, decision and consequences |
| D5 | Compatibility decision | `docs/spec/compatibility-policy.md` (updated) | Approved per-format decision, replacing the proposal |
| D6 | ADR set | `docs/adr/` | ADR-0001 promoted to Accepted; new ADRs for memory strategy, stdlib naming (`cosmic-std`), module list including `cosmic-diagnostics`, and the `.csmx` extension |
| D7 | Measurement plan | `docs/development/measurement-plan.md` | Coverage, dependency, duplication, cyclomatic-complexity, cold-start, throughput and artifact-size measurement; the fixed machine for p50/p95 |
| D8 | Test strategy | `docs/development/test-strategy.md` | The ladder: unit → exact-output → golden diagnostics → property-based → differential → fuzz → regression; plus the differential harness design |
| D9 | Diagnostic code registry | `docs/spec/diagnostic-codes.md` | The `[A-Z]{2,8}[0-9]{3}` namespace allocation per component and the stability policy in writing |
| D10 | Phase 1 exit report | `docs/audits/phase-1-specification-report.md` | Evidence that G1 (grammar approved) is met |

**No deliverable may contain invented grammar.** Every production must cite
either a legacy source example (with path and SHA) or an executed golden case.

---

## 4. Constraints

1. **Evidence discipline.** Every normative statement carries an evidence tag
   (E1/E2/H/P) and, where it describes legacy behaviour, the case name or
   source path. Unsupported statements are marked **H** (hypothesis) and are
   not normative.
2. **No compatibility promises beyond evidence.** The C0–C3 levels of
   `docs/spec/compatibility-policy.md` §1 are the vocabulary; C3 is not
   promised for any format.
3. **No implementation.** No crate may be created; no lexer, parser or type
   checker may be written. Prototypes are not permitted even as throwaway code
   in the repository.
4. **No legacy artefact reuse.** No legacy code, fixture or prose is copied.
5. **No new dependencies.** Phase 1 produces documents; if tooling is needed
   (e.g. a grammar linter), it requires an explicit justification per
   `docs/development/toolchain-policy.md` §3.
6. **Retain unresolved semantics as explicit questions.** An unanswered
   question must appear as an open decision record, never as a silent
   assumption.
7. **Preserve Phase 0.** No refactor of `cosmic-source`, `cosmic-diagnostics`
   or `cosmic-cli` is authorised by this directive.

---

## 5. Semantic questions that must be decided

Each becomes one decision record under `docs/spec/decisions/`. A decision
record may conclude "legacy behaviour is a defect; the new rule is X", but it
must record the divergence and its compatibility consequence.

**Inherited from the Phase 0 compatibility matrix:**

| # | Question |
|---|---|
| S1 | Nested-generic resolution rules (the legacy ledger records an open mis-resolution) |
| S2 | Nullable conversion and extension rules between `T`, `T?` and inferred types |
| S3 | PortuKof aliasing semantics |
| S4 | `.ks` KofScript wrapping and execution semantics |
| S5 | Pattern-matching breadth: type patterns, guards, exhaustiveness for sealed hierarchies |
| S6 | The concurrency contract: `spawn`/`await` typing, failure propagation, per-target scheduling guarantees |
| S7 | Numeric semantics: integer overflow, division semantics, float formatting and comparison |
| S8 | String semantics: escapes, indentation, triple-quoted form |
| S9 | The stdlib/language boundary (`math.*`, `json.*`, `listOf`, …) |
| S10 | Reserved keywords without evidenced semantics: `entity`, `extern`, `generated`, `audited`, `unique` |

**Added by the conformance audit:**

| # | Question |
|---|---|
| S11 | Evaluation order, including argument and receiver order, and short-circuit rules |
| S12 | Equality semantics: reference vs structural, per type kind |
| S13 | Unicode semantics: identifier classes, normalisation, source-encoding policy (BOM tolerance was observed in the legacy lexer) |
| S14 | Trivia and normalisation policy: what the lexer may normalise, and where lossless CST fidelity is required |
| S15 | The exact feature-gate vocabulary for "unsupported on this target" |
| S16 | The memory model: ownership+borrowing, GC or ARC — the design report requires a study, not an assumption |
| S17 | The runtime ABI versioning rule and the runtime → frontend dependency ban |
| S18 | The HIR invariants verified on entry to MIR, and the MIR verifier's obligations |

---

## 6. Approval gates

| Gate | Criterion | Evidence required |
|---|---|---|
| **G1 — grammar approved** | D1 and D2 exist; every production is evidenced; the owner approves | D10 cites D1/D2 and records the approval |
| **G1b — ambiguities closed** | Every S1–S18 has a decision record, with none left as an assumption | D4 completeness check in D10 |
| **G1c — identity settled** | ADR-0001 Accepted; `.csmx` uniqueness search performed exhaustively; MIME and editor-association plan approved | D5, D6 |
| **G1d — licence decided** | The owner has chosen the outbound licence and the reuse policy is final | Updated `docs/audits/phase-0-licensing-matrix.md` §6 |

**Phase 2 must not begin until G1–G1d are met.** G0-B remains PARTIAL from
Phase 0; Directive 002 may propose a documented exception for owner acceptance,
but must not reclassify it as PASS.

---

## 7. Explicitly not authorised by Directive 002

* Creating `cosmic-lexer` or any other new crate.
* Writing any lexer, parser, CST, HIR, MIR or backend code.
* Porting or translating legacy Java source.
* Copying legacy test fixtures or documentation.
* Choosing a licence on the owner's behalf.
* Publishing, tagging or releasing anything.
* Merging PR #1.
* Changing repository protection settings.

---

## 8. Completion criteria

Phase 1 is complete when: the specification, grammar and typing contracts exist
and are evidenced; every enumerated semantic question has a decision record;
the identity ADRs are approved; the compatibility decision is approved; the
measurement and test-strategy plans exist; and the owner has authorised Phase 2.

**END DIRECTIVE 002 (PROPOSED).**
