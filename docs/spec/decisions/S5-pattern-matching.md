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
  `switch-expr`, `switch-break-in-loop`, `switch-defaults`, `sealed-switch`.
  All pass 222/222 on jvm/js/script.
* **E1** — `sealed` is a **contextual** modifier parsed before a type
  declaration (`parser/Parser.java` `parse()`), and there is an executed
  `sealed-switch` case, so sealed hierarchies plus switch are evidenced.
* **E1** — `switch-defaults` exists as a case name, which shows a `default` arm
  is expected.
* **E1** — `break` is a keyword; `switch-break-in-loop` exercises `break`
  inside a `switch` inside a loop. **The case name implies the interaction is
  significant**, but the specification does not state the rule.
* **E1** — `continue` is a keyword. `StatementParser` has a `for` parser that
  handles more than the `in` form (a three-clause shape is present in the
  parser), but **no golden case exercises it**.
* **E1 (negative)** — No case exercises: labelled `break`/`continue`,
  destructuring patterns, guard expressions, or a non-exhaustive `switch`.

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
* **Q4** — **No fallthrough.** Every arm terminates. `switch-break-in-loop`
  must be re-examined in Phase 2 to confirm that `break` inside a switch arm
  means "exit the switch" (the C/Java reading) rather than "exit the loop".
  **This is an OPEN sub-question** and the case must be read before the rule is
  fixed; the name suggests the interaction was subtle enough to deserve a
  dedicated test.
* **Q5** — **Labelled `break`/`continue` are not proposed.** No evidence.
* **Q6** — Only the `for (x in collection)` form is **normative**. The
  three-clause form is parser-evidenced but corpus-unevidenced; it stays
  **OPEN** and Phase 2 must not implement it without a decision.

## Compatibility consequence

* Mandatory exhaustiveness is a **C2 divergence** for any legacy program with a
  non-exhaustive switch over an enum or sealed type that the legacy compiler
  accepted. The size of that set is unknown; Phase 2 must measure it against the
  corpus before the rule is finalised.
* Removing fallthrough is a **C2 divergence** only if fallthrough is reachable
  in legacy code — unverified.
* The `for` restriction (Q6) is a **C1 divergence** for the three-clause form.

## Open dependencies

* Phase 2 must read the executed behaviour of `switch-break-in-loop` and
  `switch-defaults` and report what they actually assert before Q4 and Q3 close.
* S1 owns whether type patterns interact with generics.
