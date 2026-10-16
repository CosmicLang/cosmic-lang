# S11 — Evaluation Order and Binding Semantics

| | |
|---|---|

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

| **Status** | **VERIFIED LEGACY BEHAVIOR** — bounded JVM results for the tested evaluation and binding cases; **OPEN** for capture intent, untested forms and target parity. No normative approval. |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; bounded JVM evidence produced in Phase 1 (`002-R1 §4`); cross-target evidence a Phase 2 deliverable |
| **Related** | `docs/spec/language-spec.md` §7/§8, `docs/spec/grammar.ebnf` §3.7, S12 |

> **R1 status interpretation:** Evidence claims below are VERIFIED LEGACY BEHAVIOR only; any proposed semantics remain PROPOSED NORMATIVE RULE pending owner action. Older recommendations predate the experiments and may describe the prior evidence gap.

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
* **E1 (probe, added by Directive 002-R1 §4)** — Executed read-only JVM probes
  now observe the behaviour directly on minimal programs: arguments, receivers
  and binary operands evaluate left to right; `&&`/`||` short-circuit;
  assignment is a statement; blocks scope and may shadow; capture is rejected.
  Commands, programs, outputs and exit statuses: `docs/audits/phase-1-legacy-probes.md`
  §2. Cross-target parity and untested forms remain **NOT VERIFIED**.

## Options

| # | Option | Assessment |
|---|---|---|
| K1 | **Specify left-to-right evaluation** for arguments, receiver-then-arguments, and binary operands | The conventional choice, and it now **matches the executed JVM probes** (probe record §2.1–2.3) rather than being assumed. Adopted here as a **PROPOSED NORMATIVE RULE**; cross-target parity is still NOT VERIFIED |
| K2 | **Specify evaluation order as unspecified**, and forbid programs from depending on it | Sound in theory; unusable in practice, because a program with two side-effecting arguments is not detectable without an effect system |
| K3 | **Produce the evidence first** (differential tests that observe order), then decide | **Partly satisfied:** the Phase 1 read-only probes produced the evidence for the sampled JVM forms. Remaining evidence (other targets, untested forms) is a Phase 2 differential-harness deliverable |
| K4 | **Adopt left-to-right provisionally**, marked as provisional, and confirm with later evidence | Now **superseded by K1-with-evidence** for the sampled forms; the "provisional quietly becomes settled" risk it guards against is handled by the explicit PROPOSED status and the remaining NOT VERIFIED items |

## Recommendation (PROPOSED, not in force)

**K1 as a PROPOSED NORMATIVE RULE for Q1–Q3 and Q6, on sampled JVM evidence
(probe record §2.1–2.3, §2.6); Q4 evidenced and proposed; Q5 dissolved by
evidence; Q7's rejection verified and its intent still OPEN.**

* **Q1–Q3** — **VERIFIED LEGACY BEHAVIOR on sampled JVM forms:** arguments,
  receiver-before-arguments and binary operands are evaluated **left to right**
  (`docs/audits/phase-1-legacy-probes.md` §2.1–2.3, exit `0`). Unsampled forms
  and the `js`/`script` targets remain **NOT VERIFIED**.
  **PROPOSED NORMATIVE RULE:** left-to-right, matching the sampled legacy
  behaviour. The evaluation model must still be kept behind one component so a
  cross-target divergence, if later measured, is not expensive to accommodate.
* **Q4** — **Short-circuit is VERIFIED LEGACY BEHAVIOR on the sample** (§2.4)
  and **PROPOSED NORMATIVE RULE for `&&` and `||`.** The token structure, the
  precedence levels and the dedicated conditional-jump IR operations already
  pointed the same way; the probe confirms it in execution.
* **Q5** — **the question dissolved; it is not merely unanswered.** Assignment
  is a **statement**, not an expression: `a = b = 1` is rejected with `SEM027`
  (§2.5). There is no chained-assignment associativity in the tested forms.
  **PROPOSED NORMATIVE RULE:** assignment remains a statement. Untested forms
  elsewhere in the grammar remain NOT VERIFIED.
* **Q6** — **VERIFIED LEGACY BEHAVIOR on the sample:** blocks are scoped and an
  inner declaration shadows an outer one without error (§2.6).
  **PROPOSED NORMATIVE RULE:** block scoping with shadowing permitted.
* **Q7** — **VERIFIED LEGACY BEHAVIOR that capture is rejected** (`SEM011`,
  §2.7): a nested function cannot see the enclosing local at all.
  **PROPOSED NORMATIVE RULE:** nested functions are non-capturing until a
  capture design is accepted. The **intended** semantics are **NOT
  DISTINGUISHED** by the probe and remain OPEN. Note the interaction with the
  memory strategy (S16): whether a nested function captures by value or by
  reference determines whether closures need heap allocation, which is a design
  input to ADR-0005 — and the measured rejection substantially narrows that
  study. **Q7 therefore feeds S16.**

## Compatibility consequence

* Q1–Q3: the proposal (left-to-right) **matches the sampled legacy behaviour**,
  so it is **C0-compatible on the probed forms**. A program with observable side
  effects in argument position is where a divergence *would* show; the corpus
  cannot detect it, which is why it was probed directly. Unsampled forms and
  other targets remain NOT VERIFIED, so a blanket C0 claim is not made.
* Q4 is **C2-compatible** with the legacy structure and is now executed-evidenced.
* Q5: keeping assignment a statement is C0-compatible with the probed rejection
  of `a = b = 1` (`SEM027`).
* Q7 affects the memory strategy, so getting it wrong is expensive later.

## Open dependencies

* Cross-target parity for Q1–Q3 and Q6 is **NOT VERIFIED**; the Phase 2
  differential harness must repeat the probes on `--target js` and
  `--target script` before any compatibility claim rests on them.
* **Q7's intent requires an owner or maintainer statement**, not an experiment;
  the probe establishes only the implemented rejection.
* **Q7 is an input to ADR-0005** (memory strategy): capture semantics constrain
  whether closures can be stack-allocated.
* S12 (equality) shared the "no executed evidence" problem; it now has its own
  bounded JVM probes (`docs/audits/phase-1-legacy-probes.md` §3).

---

## Experimental evidence (Directive 002-R1 §4)

Executed read-only against the frozen legacy compiler on 2026-10-16,
`--target jvm`, OpenJDK 27. Full commands, programs, stdout, stderr and exit
statuses are in `docs/audits/phase-1-legacy-probes.md`.

This record's questions were **answered by
measurement**, which is why it now has VERIFIED LEGACY BEHAVIOR evidence; normative acceptance is pending.

| Q | Result | Observed |
|---|---|---|
| Q1 argument order | **left to right** | `sum(a(), b())` printed `A` then `B` |
| Q2 receiver vs arguments | **receiver first** | `recv().m(arg())` printed `RECV` then `ARG` |
| Q3 binary operands | **left to right** | `l() + r()` printed `LHS` then `RHS` |
| Q4 short-circuit | **yes, both operators** | `f() && g()` printed only `F`; `g() \|\| f()` printed only `G` |
| Q5 assignment associativity | **not applicable** | `a = b = 1` is `SEM027`: *assignment is a statement, not an expression* |
| Q6 scoping / shadowing | **block-scoped, shadowing allowed** | inner `var x = 2` printed `2`, outer printed `1` |
| Q7 nested-function capture | **rejected** | `SEM011: Undefined variable or type: 'n'` |

**Q5 dissolved rather than being answered.** There is no associativity to
determine, because assignment is not an expression. This is a better outcome
than an answer: the question was an artefact of assuming expression assignment.

**Q7 has a consequence beyond this record.** Nested functions are
**observed to reject this capture**, so this test does not require a closure environment to allocate. The
"escaping by-reference capture forces heap allocation" argument therefore **does
not apply** to this language, which substantially narrows the ADR-0005 memory
study. The *intent* behind the rejection is **not** established and remains OPEN.

**Proposed rules** (all PROPOSED NORMATIVE RULE, none accepted):
evaluation is left-to-right at every position; `&&` and `||` short-circuit;
assignment is a statement; blocks scope and may shadow.
