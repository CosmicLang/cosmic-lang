# S11 — Evaluation Order and Binding Semantics

| | |
|---|---|

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

| **Status** | **BLOCKED-ON-MEASUREMENT** for the ordering questions; **PROPOSED** for short-circuit |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; evidence produced in Phase 2 |
| **Related** | `docs/spec/language-spec.md` §7/§8, `docs/spec/grammar.ebnf` §3.7, S12 |

## Question

1. **Q1** — In what order are **function arguments** evaluated?
2. **Q2** — In what order are a **receiver** and its **arguments** evaluated in a
   method call?
3. **Q3** — In what order are the **operands** of a binary operator evaluated?
4. **Q4** — Do `&&` and `||` **short-circuit** for all operand types?
5. **Q5** — Is **assignment** right-associative, and what does a chained
   assignment evaluate to?
6. **Q6** — What are the **scoping and shadowing** rules?
7. **Q7** — What does a **nested function** capture, and when?

## Evidence

* **E1** — `&&` and `||` are distinct tokens with their own precedence levels
  (levels 2 and 1, `ExpressionParser.precedence`), and the legacy IR has
  dedicated conditional-jump operations. So short-circuiting is **structurally
  supported**.
* **E1** — The legacy compiler lowers expressions to a **typed linear stack
  machine**. In such a machine the evaluation order is the emission order, so
  the order is *determined* by the lowerer — but it was never written down as a
  language rule, and it is not visible in the golden corpus (whose cases print
  values, not evaluation traces).
* **E1** — Assignment is parsed by a **separate** function
  (`ExpressionParser.parseAssignment`) that sits *above* `parseBinary`, i.e.
  assignment has the lowest precedence. Its associativity is not evident from
  the code without reading the recursion shape, and is not tested.
* **E1** — Nested function declarations exist
  (`StatementParser.parseNestedFunction`), but nothing in the corpus exercises
  capture.
* **E1 (negative)** — **No** golden case observes evaluation order. There is no
  case with two side-effecting arguments, no case with a side-effecting receiver,
  and no case that distinguishes left-to-right from right-to-left. The corpus is
  therefore **silent** on Q1–Q3 and Q5–Q7.
* **E1** — The legacy project's own ledger contains a *semantics* defect in a
  neighbouring area (shift behaviour differing per target, `cosmcc:docs/status.md`
  §698), which shows that "the lowerer decides" has produced observable
  divergences before.

## Options

| # | Option | Assessment |
|---|---|---|
| K1 | **Specify left-to-right evaluation** for arguments, receiver-then-arguments, and binary operands | The conventional choice and the one most likely to match the legacy lowerer. But it is **not evidenced**, so adopting it means asserting a rule the project cannot currently check |
| K2 | **Specify evaluation order as unspecified**, and forbid programs from depending on it | Sound in theory; unusable in practice, because a program with two side-effecting arguments is not detectable without an effect system |
| K3 | **Produce the evidence first** (Phase 2 differential tests that observe order), then decide | Correct method. Cost: Phase 2 cannot build the typed HIR's evaluation model until the order is known — but it *can* build the frontend, because order is a middle-end concern |
| K4 | **Adopt left-to-right provisionally**, marked as provisional, and confirm with Phase 2 evidence | Pragmatic; the risk is that "provisional" quietly becomes "settled" |

## Recommendation (PROPOSED, not in force)

**K3 for Q1–Q3 and Q5–Q7; K1 marked PROVISIONAL as the working assumption;
short-circuit (Q4) decided now.**

* **Q1–Q3** — **BLOCKED-ON-MEASUREMENT.** The order must be established by
  targeted differential tests in Phase 2: pairs of side-effecting arguments,
  side-effecting receivers, and side-effecting binary operands. Until then the
  specification records the order as **unspecified** and Phase 2 must not encode
  an order in a way that is expensive to change (i.e. keep the evaluation model
  behind one component).
  **Working assumption, explicitly provisional: left-to-right**, which is what
  the conventional lowering produces. It must not be documented as settled.
* **Q4** — **Short-circuit is normative for `&&` and `||`.** The token
  structure, the precedence levels and the dedicated conditional-jump IR
  operations are sufficient evidence that the operators are control-flow
  operators rather than eager boolean functions. This is the one part of this
  record that can be decided from the evidence at hand.
* **Q5** — **OPEN.** Associativity and the value of a chained assignment are not
  evidenced. Phase 2 must test them. The provisional assumption is
  right-associative with the assigned value as the result, matching the
  conventional reading — again explicitly provisional.
* **Q6** — **OPEN.** Scoping and shadowing rules are not evidenced. The
  specification states only the structural facts: resolution is scope-based with
  a symbol table and a two-pass shape (`docs/spec/language-spec.md` §7). The
  shadowing rule must be decided with evidence.
* **Q7** — **OPEN.** Capture semantics for nested functions are not evidenced.
  Note the interaction with the memory strategy (S16): whether a nested function
  captures by value or by reference determines whether closures need heap
  allocation, which is a design input to ADR-0005. **Q7 therefore feeds S16.**

## Compatibility consequence

* Deciding Q1–Q3 contrary to the legacy order would be a **C2 divergence** on
  any program with observable side effects in argument position. Such programs
  exist in principle but not in the corpus, so the corpus cannot detect the
  divergence — which is precisely why the evidence must be produced separately.
* Q4 is **C2-compatible** with the legacy structure.
* Q7 affects the memory strategy, so getting it wrong is expensive later.

## Open dependencies

* Phase 2 must build the differential harness before Q1–Q3, Q5–Q7 can close.
* **Q7 is an input to ADR-0005** (memory strategy): capture semantics constrain
  whether closures can be stack-allocated.
* S12 (equality) shares the "no observable evidence" problem and must be
  measured with the same harness.
