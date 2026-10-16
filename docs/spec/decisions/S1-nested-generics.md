# S1 — Nested Generics, Generic Inference, Bounds, Variance and Overload Resolution

| | |
|---|---|

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

| **Status** | **OPEN** — Q1 is a **verified defect** (not a missing rule); Q5 is **PARTIAL**. Both are **GATED**, not blocking. |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Decision needed in Phase 1; differential evidence produced in Phase 2 |
| **Related** | `docs/spec/typing-contracts.md` §3/§6, `docs/spec/language-spec.md` §6.1, `docs/spec/grammar.ebnf` §3.4 |

> **R1 status interpretation:** Evidence claims below are VERIFIED LEGACY BEHAVIOR only; any proposed semantics remain PROPOSED NORMATIVE RULE pending owner action. Older recommendations predate the experiments and may describe the prior evidence gap.

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

---

## Experimental evidence (Directive 002-R1 §4)

Executed read-only against the frozen legacy compiler on 2026-10-16,
`--target jvm`, OpenJDK 27. Full commands, programs, stdout, stderr and exit
statuses are in `docs/audits/phase-1-legacy-probes.md`.

The probe **changed the character of Q1**: it is
not a missing rule, it is a **confirmed defect**.

| Probe | Result |
|---|---|
| `Box<Box<Int>>` | **compiles, then crashes at run time** — `NoSuchMethodError: 'int Box.get()'` |
| `List<Box<Int>> = listOf(Box(3))` | `SEM021`, and the message renders the inferred type as **`List<Box>`** — the nested type argument was **dropped during inference** |
| `Box<List<Int>>` (assignment) | accepted |
| `m(Int)` vs `m(Long)` with `1` | **`int`** — the most specific applicable overload wins |
| `m(A)` vs `m(B)`, argument implements both | **`A`** — **no ambiguity reported**; the first declared overload is selected silently |
| `a < b > c` | a **left-associative comparison chain**, not a generic application |

**Two consequences that matter.**

1. **Q1 cannot be specified from the legacy behaviour**, because the legacy
   behaviour is wrong. There is no correct rule to read off. It must be
   **designed**, and the legacy divergence recorded as a defect. This is why the
   record stays OPEN rather than moving to PROPOSED.
2. **Q5's tie-break is confirmed.** `docs/spec/typing-contracts.md` §6 *predicted*
   that declaration order was the tie-break. That prediction is now backed by
   execution, which makes the proposed rule ("ambiguity is an error") a
   **measured C2 divergence** rather than a guess.

**Still NOT DISTINGUISHED:** whether the tie-break is *strictly* declaration
order in all cases (one ordering was probed); generic bounds and variance syntax
(no evidence at all).

**Gating.** Nested type arguments (depth ≥ 2) and ambiguous overloads are
**GATED**: `Box<Int>` and unambiguous calls compile normally; only the
unsupported operation is refused. See `docs/spec/implementation-gates.md` §3.2.
