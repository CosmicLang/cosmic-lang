# S18 — Typed-IR Invariants and the MIR Verifier

| | |
|---|---|

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

| **Status** | **PROPOSED NORMATIVE RULE** — owner acceptance outstanding |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; enforcement in Phase 2/3 |
| **Related** | `docs/spec/typing-contracts.md` §8, `docs/spec/language-spec.md` §12, S1, S11, S15 |

## Question

1. **Q1** — Which invariants must hold when a typed representation crosses into
   MIR?
2. **Q2** — At which points must they be **verified**?
3. **Q3** — What are the MIR verifier's obligations, and is it always on?
4. **Q4** — What happens when verification fails — a panic, an internal error, or
   a user-visible diagnostic?
5. **Q5** — How does interpreted-MIR execution relate to generated code?

## Evidence

* **E1 — the legacy typed IR carries no semantics of its own.** The Phase 0
  design-conformance audit re-verified this from source: the legacy IR is a
  typed linear stack machine where most operations carry a `Type`, and that type
  has a **functional** role in consumption (for example, a conditional-jump
  operation's operand type selects the JVM comparison instruction), but the
  representation has **no formal semantics** — no dependency graph, no value
  domain, no verifier. Source: `docs/audits/phase-0-design-conformance.md` §3.
* **E1 — the legacy IR is not actually block-structured.** The legacy
  architecture document states that lowering emits **exactly one basic block per
  method** and that the optimizer flattens and repacks; the real unit is the
  flat operation list with labels. Re-verified at source in
  `docs/audits/phase-0-design-conformance.md` §3. So the legacy "basic blocks"
  are nominal, which is why flow analyses are hard — exactly the design report's
  page 6 diagnosis.
* **E1 — the design report requires a verifier after lowering and after each
  pass**, in test builds, and requires comparing interpreted-MIR execution with
  generated output (page 15). The report also requires "SSA with a dominance
  verifier" (page 13).
* **E1 — the legacy project has a precedent for catching this class of defect.**
  The WASM backend **refuses** record equality because the JVM desugar merges a
  field comparison through control flow across labels, which the linearized
  model cannot represent. That is a defect that a MIR verifier with explicit
  terminators and a proper CFG would have made visible *before* code generation.
* **E1 — the legacy project's own open defect list contains a semantics bug**
  (cross-target shift divergence, `cosmcc:docs/status.md` §698) found *after* the
  golden corpus passed. This is evidence that "the corpus is green" is not a
  sufficient correctness argument, and it is the strongest available argument
  for an always-on verifier in test builds.
* **E1** — Phase 0 established two relevant precedents: `unsafe_code` is
  `forbid` workspace-wide, and the diagnostics layer is **total** (never panics
  on input). A verifier that panics on a compiler bug would be a *programming*
  error, which is the one class of panic Phase 0 permits; a verifier that panics
  on *user input* is forbidden.

## Options

| # | Option | Assessment |
|---|---|---|
| R1 | **Verifier always on, in every build**, including release | Maximum safety. Cost: compilation throughput — the design report's own metrics list build time, so this has a measurable price |
| R2 | **Verifier on in test/debug builds only**, as the design report specifies | Matches the report, keeps release builds fast. Cost: a verifier-clean debug build plus an optimizer bug that only manifests in release is still possible — but that is a compiler bug, not a language-level hazard |
| R3 | **Verifier on behind a flag**, default off | Cheapest, and the least useful: a verifier nobody runs catches nothing |
| R4 | **No verifier**; rely on the differential corpus | Directly contradicted by the shift-semantics precedent: the corpus passed and the semantics were still wrong |

## Recommendation (PROPOSED, not in force)

**R2, with R1's guarantee for a defined subset.**

* **Q1 — The invariant set.** Adopt TI-1…TI-8 from
  `docs/spec/typing-contracts.md` §8 as the *minimum*, and add the MIR-specific
  invariants:
  | # | Invariant | Rationale |
  |---|---|---|
  | MI-1 | Every basic block has **exactly one** terminator, and the terminator is explicit | The legacy single-block IR has none; the design report requires them |
  | MI-2 | Every branch target names an existing block, and the CFG is **reachable-consistent** (no block is referenced but unreachable, and vice versa unless deliberately dead) | Catches the label-jump class of defect |
  | MI-3 | Every place/rvalue is **type-consistent** with its declared type | Catches the operand-type/instruction mismatch class |
  | MI-4 | Every local is **definitely assigned** before use on every path | The legacy model has no such check |
  | MI-5 | After SSA construction, **every use is dominated by its definition** | The design report's "dominance verifier" |
  | MI-6 | Every function's exit paths agree with its declared result type | Prevents a miscompiled return |
  | MI-7 | No operation carries an unresolved or `UnknownType` type | TI-1 restated at the MIR boundary |
  | MI-8 | An optimization pass **preserves the observable effect** of the blocks it rewrites; a pass that cannot prove it must not apply | This is the invariant the shift-semantics defect violated |
* **Q2 — Verification points.** After lowering, and after **every** optimization
  pass, in test and debug builds. Additionally, **always** (including release)
  for: the CFG structural invariants (MI-1, MI-2) and the dominance invariant
  (MI-5) after SSA construction. Rationale: those three are cheap, and they are
  the ones whose violation produces silently wrong code rather than a crash.
* **Q3 — Verifier obligations.** The verifier must be **total** (never panic on
  compiler-generated IR), must report the *first* violation with enough context
  to reproduce it (function, block, operation index), and must be usable as a
  library so tests can call it directly rather than only through a build.
* **Q4 — On failure.** A verification failure in a *test* build is an **ICE**
  (internal compiler error) with a stable diagnostic code and a non-zero exit —
  never a panic, never a silent continue. In a *release* build, the always-on
  subset (Q2) reports the same way. **The verifier must never "repair" the IR**:
  silent repair is the compiler-internal form of the silent degradation the
  language rules forbid.
* **Q5 — Interpreted MIR is the oracle.** The MIR interpreter is the semantic
  reference: for any program, interpreting the MIR and running the generated
  artifact must agree. A disagreement is a **backend** defect, and it is the
  only way to attribute a failure to a backend rather than to the frontend. This
  is the design report's own ordering (page 16: the interpreter is first, as the
  oracle).

## Compatibility consequence

* **C0–C3 neutral.** The verifier is an internal correctness mechanism; it does
  not change the language or the observable behaviour of a correct program.
* It does change the *failure mode* of an incorrect compiler: a miscompile
  becomes an ICE instead of wrong output. That is strictly better and is the
  point.
* The **ICE rate** is one of the design report's own metrics (page 19). This
  record is what makes that metric measurable.

## Open dependencies

* **S11** (evaluation order) — MI-8 cannot be checked precisely until the
  observable effect of each operation is defined, which needs the evaluation
  order.
* **S1** (generics) — MI-3's type consistency depends on the type
  representation being settled.
* **S15** — the ICE diagnostic code and the refusal code must share one
  namespace (D9).
* The MIR *representation* itself (blocks, statements, terminators, places,
  rvalues) is the design report's page 15 list and is not specified by
  Directive 002; this record specifies only the invariants a future MIR must
  satisfy.
