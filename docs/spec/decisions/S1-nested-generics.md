# S1 — Nested Generics, Generic Inference, Bounds, Variance and Overload Resolution

| | |
|---|---|

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

| **Status** | **OPEN** (the overload-resolution and nested-argument sub-questions are **BLOCKED-ON-MEASUREMENT**) |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Decision needed in Phase 1; differential evidence produced in Phase 2 |
| **Related** | `docs/spec/typing-contracts.md` §3/§6, `docs/spec/language-spec.md` §6.1, `docs/spec/grammar.ebnf` §3.4 |

## Question

Five related sub-questions, all currently unanswered:

1. **Q1** — What is the resolution rule for **nested type arguments**, e.g.
   `Box<Box<Int>>`? The legacy ledger records an open defect where
   `Box<Box<Int>>.get()` resolved at the wrong nesting level
   (`cosmcc:docs/status.md` §706 at the frozen SHA).
2. **Q2** — When a generic call omits its type arguments, what is the
   **inference** rule?
3. **Q3** — What is the **bounds** syntax? No declaration-site bound syntax is
   evidenced at all.
4. **Q4** — Is **variance** declared at the declaration site, at the use site
   (wildcards), or both, and how do they interact?
5. **Q5** — What are the **overload applicability and specificity** rules?

## Evidence

* **E1** — `Type` is a sealed interface with eight variants including
  `TypeVariable` and `WildcardType`, and `WildcardType` **is** constructed in
  practice (the legacy docs claimed it was never constructed; the Phase 0 audit
  verified otherwise). Source: `docs/audits/phase-0-baseline.md` §3.5/§7.
* **E1** — Assignability is a nominal breadth-first search over supertypes with
  **both** declaration-site variance and use-site wildcards, plus an older
  structural fallback that consults runtime class relations.
* **E1** — Generic type arguments are written `<…>`, may nest, and generic
  *calls* may carry explicit type arguments
  (`parser/ExpressionParser.java parseCallTypeArguments`).
* **E1** — The golden corpus contains `generic-toplevel-function`,
  `generics-box`, `nested-generic-leading-return-type` and
  `abstract-records`, and those cases pass 222/222 on jvm/js/script. So the
  *simple* forms work.
* **E1 (negative)** — The golden corpus contains **no** overload-ambiguity case
  and **no** bounds-syntax case. There is therefore **no executed evidence** for
  Q3 or Q5.
* **E1** — Overload resolution lives in `MethodCallTyper` / `OverloadSelector`;
  the analyzer's `resolveMethodCalls` phase is a documented no-op vestige.

## Options

| # | Option | Assessment |
|---|---|---|
| A1 | **Defer all five sub-questions to Phase 2** and require differential evidence before any rule is fixed | Honest, and consistent with the project's evidence discipline. But it leaves Phase 2 without a contract, so Phase 2 would be partly discovery |
| A2 | **Fix Q1 and Q2 now** from the evidenced simple cases, and defer Q3/Q4/Q5 | Gets the highest-value rules in place; risk of fixing a rule that the complex cases contradict |
| A3 | **Adopt a specified third-party generic model** (e.g. Java's, given the legacy type model resembles it) | Fast, but it is an unevidenced import of a foreign semantics, and the design report explicitly wants the language's own semantics, not a Java clone |

## Recommendation (PROPOSED, not in force)

**A1 for Q1/Q5; A2 for Q2/Q4; Q3 deferred with no syntax permitted.**

* **Q1** — Cannot be decided from documentation, because the legacy behaviour is
  a *defect*: there is no correct rule to read off. Decide it in Phase 2 from a
  targeted differential harness that pins the *intended* rule, records the
  legacy divergence as a known defect, and adds it to the "legacy defects are not
  part of the contract" list. **Do not guess a rule now.**
* **Q2** — Inference is *local and bounded* (see `typing-contracts.md` §5):
  infer from declared boundary types and local unification only. A call with
  omitted type arguments infers from its argument types; if inference fails, it
  is an error, never a silent `UnknownType`.
* **Q3** — **No bounds syntax is specified.** Phase 2 must reject any
  declaration-site bound syntax rather than invent one. If bounds are needed,
  they require their own ADR.
* **Q4** — Both declaration-site and use-site variance exist in the legacy
  model; the new toolchain retains **use-site** wildcards as the primary
  mechanism (they are the one the corpus-adjacent code exercises) and treats
  declaration-site variance as **OPEN**.
* **Q5** — Adopt the discipline in `typing-contracts.md` §6: ambiguity is an
  **error**, not a declaration-order tie-break. The legacy tie-break is not
  evidence of a language rule.

## Compatibility consequence

* Fixing Q1 correctly will **intentionally differ** from the legacy compiler on
  nested generics. Under C0–C3, this is a C2 divergence on a construct that the
  legacy compiler gets wrong. It must be recorded in the legacy-defect list, not
  treated as a regression.
* Any change to overload resolution (Q5) is a **C2 divergence** on programs that
  the legacy compiler accepted by tie-break. Such programs are expected to be
  rare, but the risk is real and must be measured, not assumed.

## Open dependencies

* **Phase 2 differential harness** must exist before Q1 and Q5 can be closed.
* Owner acceptance of the "legacy defects are not part of the contract" policy
  (already proposed in `docs/spec/compatibility-policy.md` §3 rule 6).
