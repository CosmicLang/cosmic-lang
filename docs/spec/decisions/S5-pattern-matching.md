# S5 — Pattern Matching, Guards, Exhaustiveness and Control Transfer

| | |
|---|---|
| **Status** | **OPEN** |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; Phase 2 validation |
| **Related** | `docs/spec/language-spec.md` §5, `docs/spec/grammar.ebnf` §3.6, S1 |

## Question

1. **Q1** — What **patterns** may appear in a `switch` label: values, type
   patterns, constant patterns, destructuring, `null`?
2. **Q2** — Are **guards** (`case X if cond -> …`) part of the language, and
   what is their scope?
3. **Q3** — Is `switch` **exhaustive**? For sealed hierarchies? For enums? For
   booleans? What happens when it is not?
4. **Q4** — Is **fallthrough** possible, or does every arm terminate?
5. **Q5** — Do `break` and `continue` accept **labels**, and what does `break`
   mean inside a `switch` nested in a loop?
6. **Q6** — May a `for` statement use the three-clause form
   (`for (init; cond; step)`), or only `for (x in collection)`?

## Evidence

* **E1** — The parser has dedicated entry points for case patterns and guards:
  `parser/StatementParser.java` `parseSwitchCasePatternOrValue` and
  `parseGuardIfPresent`. So type patterns **and** guards exist.
* **E1** — `switch` exists in both statement and expression form:
  `StatementParser.parseSwitchStatement` and
  `ExpressionParser.parseSwitchExpression`.
* **E1** — Executed cases: `enum-switch-expr` (enum + `case X -> value`),
  `switch-expr`, `switch-break-in-loop`, `string-switch-defaults`,
  `sealed-switch`. All pass 222/222 on jvm/js/script. (The case is named
  `string-switch-defaults`; an earlier citation of `switch-defaults` was wrong —
  corrected in 002-R3.)
* **E1** — `sealed` is a **contextual** modifier parsed before a type
  declaration (`parser/Parser.java` `parse()`), and there is an executed
  `sealed-switch` case, so sealed hierarchies plus switch are evidenced.
* **E1** — `string-switch-defaults` executes `default :` in a statement switch
  and `default ->` in a switch expression, plus five parameter defaults; the
  executed `switch-expr` and `switch-break-in-loop` cases also carry
  `default :` arms.
* **E1** — `break` is a keyword; `switch-break-in-loop` exercises `break`
  inside a `switch` inside a loop. The case has now been **read** (002-R3): it
  prints 208/21, which is only derivable if `break` exits the **switch** and
  the enclosing loop continues — recorded as VERIFIED LEGACY BEHAVIOR in
  `docs/spec/language-spec.md` §5. Whether that becomes a normative rule is
  Q4's acceptance question, still open.
* **E1** — `continue` is a keyword; `loops-control` executes `continue` (next
  iteration) and `break` (loop exit) inside a three-clause `for`. The earlier
  "no golden case exercises it" annotation was wrong (F3, corrected in R2):
  `loops-control` exercises the three-clause form and is executed.
* **E1** — `switch-break-in-loop` also exercises a **destructuring pattern**
  (`case Point(var x, var y):`, executed; record `Point`) and `sealed-switch`
  a type-and-binding pattern (`case Circle c -> …`). Pattern semantics remain
  open (Q1).
* **E1 (negative)** — No case exercises: labelled `break`/`continue`, guard
  expressions, or a non-exhaustive `switch`.

## Options

| # | Option | Assessment |
|---|---|---|
| E1 | **Define exhaustiveness as mandatory for sealed and enum switches**, and a compile error otherwise | Sound, teachable, and consistent with `sealed` existing at all. Risk: legacy programs with a `default` arm covering the rest stay valid; programs relying on a non-exhaustive switch over an enum would newly fail |
| E2 | **No exhaustiveness requirement**; a non-exhaustive switch is a runtime concern | Maximum compatibility, but it makes `sealed` largely decorative and pushes a static error to runtime |
| E3 | **Defer Q1–Q6 entirely** to Phase 2 | Leaves statement semantics unspecified |

## Recommendation (PROPOSED, not in force)

* **Q1** — Patterns: value/constant patterns, **type patterns**, `null`, and
  `default`. Destructuring is **not** proposed (no evidence).
* **Q2** — Guards are part of the language (parser-evidenced). Scope: a guard
  may reference the pattern's binding and enclosing scope; it must be a boolean
  expression.
* **Q3** — Exhaustiveness is **required for `switch` over a sealed hierarchy or
  an enum**, and **checked at compile time**. A non-exhaustive switch over those
  is an error. Over other types a `default` arm is required unless the type is
  known finite. Rationale: `sealed` exists, and the design report asks for
  "exhaustiveness requirements for sealed hierarchies".
* **Q4** — **No fallthrough** proposed (none observed in any executed case).
  The `break` sub-question has since been **read** (002-R3): the executed
  `switch-break-in-loop` case shows `break` exits the switch, not the loop —
  the C/Java reading. Adopting it as a normative rule remains part of this
  proposal's acceptance.
* **Q5** — **Labelled `break`/`continue` are not proposed.** No evidence.
* **Q6** — Only the `for (x in collection)` form is **normative** in this
  proposal. The three-clause form is parser-evidenced *and* executed
  (`loops-control`, F3 correction) and now derived by the grammar (002-R3);
  its normative acceptance stays **OPEN** — Phase 2 must not treat it as an
  accepted rule or invent semantics beyond the executed behavior without an
  owner decision.

## Compatibility consequence

* Mandatory exhaustiveness is a **C2 divergence** for any legacy program with a
  non-exhaustive switch over an enum or sealed type that the legacy compiler
  accepted. The size of that set is unknown; Phase 2 must measure it against the
  corpus before the rule is finalised.
* Removing fallthrough is a **C2 divergence** only if fallthrough is reachable
  in legacy code — unverified.
* The `for` restriction (Q6) is a **C1 divergence** for the three-clause form.

## Open dependencies

* The reading of `switch-break-in-loop`, `string-switch-defaults` and
  `loops-control` is done (002-R3): the executed assertions are recorded in the
  Evidence section and in `docs/spec/language-spec.md` §5. Q3 still needs a
  reachability measurement (no golden case is non-exhaustive) before it closes.
* S1 owns whether type patterns interact with generics.

## 002-R3 update (2026-10-16)

* Wrong case citation `switch-defaults` corrected to `string-switch-defaults`;
  its default forms are executed, not merely named.
* The three-clause `for` and `continue`/`break` executions (F3) are corrected
  in the Evidence section; `loops-control` is executed.
* `switch-break-in-loop` read: `break` exits the switch; recorded as VERIFIED
  LEGACY BEHAVIOR in `docs/spec/language-spec.md` §5. Q4's factual
  sub-question is answered; its normative adoption remains open.
* Grammar corrections (002-R3): colon statement arms and arrow expression arms
  are now separate productions (`switch-stmt-arm` / `switch-expr-arm`);
  `break-stmt`/`continue-stmt` added. Labelled forms remain NOT VERIFIED and
  are refused.
* Status remains **OPEN**; the S1–S18 records remain individually
  authoritative.
