# S12 — Equality Semantics

| | |
|---|---|
| **Status** | **OPEN** (one part **PROPOSED**) |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; Phase 2 measurement |
| **Related** | `docs/spec/language-spec.md` §6.1/§9, `docs/spec/typing-contracts.md` §1.1/§7, S9, S11 |

## Question

1. **Q1** — Is `==` **reference** or **structural** equality, per type kind?
2. **Q2** — Are `==` and `!=` the *only* equality operators, and is there a
   separate identity comparison?
3. **Q3** — Are `equals`/`hashCode`-style members part of the language, and does
   overriding one oblige overriding the other?
4. **Q4** — How does equality behave for `string`?
5. **Q5** — How does equality behave for `Int`, `Long`, `Float` and `Double`,
   including `NaN` and `-0.0`?
6. **Q6** — How does equality behave for a `record` (component-wise?) and for an
   `enum` (identity?)?
7. **Q7** — What is the contract for using values as **map keys** and in
   **sets**?

## Evidence

* **E1** — `==` and `!=` are distinct tokens at precedence level 5, alongside
  the relational operators (`ExpressionParser.precedence`). There is **no**
  separate identity-comparison operator in the token set.
* **E1** — The legacy compiler has a **record-equality lowerer**, i.e. record
  equality is a dedicated lowering concern rather than a library call. So record
  equality is not plain reference equality.
* **E1** — The legacy WASM backend has an explicit, documented **refusal** for
  record equality (`WASM002`) with the stated reason that the JVM `==` desugar
  merges the field comparison through control flow across labels, which the
  linearized loop model does not represent. This is direct evidence that record
  `==` is **component-wise** on the JVM (the desugar compares fields) and that
  the semantics were **not** portable to at least one target.
* **E1 (executed)** — The corpus exercises equality in several cases:
  `records` (`p.x() == 3 && p.y() == 7`), `record-values`, `abstract-records`,
  `enum`, `map-set`, `map-methods`, `collections`, `json-records`,
  `string-methods-extended`, `strings-methods`. All pass 222/222 on
  jvm/js/script.
* **E1 (executed)** — `double-formatting` prints `Infinity` and a negative
  value, which shows the numeric surface includes infinities; it does **not**
  exercise `NaN` equality or `-0.0` equality.
* **E1 (negative)** — **No** golden case distinguishes reference equality from
  structural equality for objects. No case compares `NaN`, `-0.0`, or two
  structurally-equal-but-distinct records that are *not* compared via a record
  type known statically.
* **E1** — The design report (page 7) lists **equality** among the semantics
  that must be made normative, precisely because it affects observable
  behaviour and can differ per target.

## Options

| # | Option | Assessment |
|---|---|---|
| L1 | **`==` is structural everywhere** (value equality for all types) | Simple to state and teach. But it makes `==` potentially expensive and silently recursive, and it would be a C2 divergence for any legacy program that relies on reference equality for objects |
| L2 | **`==` is reference equality for reference types and value equality for primitives, with records structural by lowering** — i.e. describe the legacy behaviour exactly | Matches the strongest evidence (record-equality lowerer, JVM field-compare desugar, WASM refusal). Keeps `==` cheap. Cost: the rule is "structural for records, reference for classes", which is inconsistent-looking and needs to be stated carefully |
| L3 | **Structural for records and enums, reference for classes and interfaces, value for primitives and strings** | A refinement of L2 that makes the record/enum cases uniform and keeps strings value-like. Still needs the string decision (Q4) confirmed |
| L4 | **Defer** | Equality is needed by the type checker (it decides whether `==` is legal between two types) |

## Recommendation (PROPOSED, not in force)

**L3, with Q4 and Q5 explicitly OPEN until measured.**

* **Q1/Q2** — `==` and `!=` are the only equality operators (no identity
  operator exists in the token set, and adding one is a language extension).
  Semantics by type kind:
  * **primitives** (`bool`, `byte`, `short`, `int`, `long`, `float`, `double`,
    `char`): **value** equality;
  * **`string`**: **value** equality (this is the model implied by treating
    `string` as a language type with value semantics — see S9 Q3, which must be
    confirmed);
  * **`record`**: **component-wise structural** equality (evidenced by the
    record-equality lowerer and the JVM field-compare desugar);
  * **`enum`**: **identity** equality (enum constants are singletons);
  * **class/interface/other reference types**: **reference** equality.
* **Q3** — No `equals`/`hashCode` override contract is proposed. The evidenced
  language has no such members, and inventing them would be a language
  extension.
* **Q4** — **OPEN.** The string decision depends on S9 Q3 (`string` as a
  language-level value type). Recorded as value equality as a **proposal**, to
  be confirmed with S9.
* **Q5** — **OPEN.** `NaN != NaN` and `-0.0 == 0.0` (IEEE-754 behaviour) versus
  a total-equality reading is **not** evidenced. This matters because the WASM
  refusal shows equality is already a portability hazard. Phase 2 must test it
  and Phase 4 must confirm it per target.
* **Q6** — Records component-wise; enums by identity. Stated above.
* **Q7** — **OPEN.** The map-key/set contract depends on Q3 and Q5: if equality
  is reference-based for classes, then two structurally-equal class instances are
  distinct keys. This is observable and must be specified before the collection
  library is (S9 Q5).

## Compatibility consequence

* L3 is **C2-compatible** with the evidenced record behaviour and with the
  primitive cases.
* Declaring **`string` value-equal** is a **C2 divergence** if the legacy
  compiler used reference equality for strings. That must be measured, not
  assumed — a string-equality case is cheap to add to the Phase 2 harness.
* The **WASM refusal** is a precedent for the honest-refusal policy: where a
  target cannot honour a normative equality rule, it must refuse rather than
  silently differ. That is the `compatibility-policy.md` §3 rule 4 applied to
  equality.
* **C3 is not promised** — the JVM desugar shape is explicitly not part of the
  contract.

## Open dependencies

* **S9 Q3** (`string` modelling) must be confirmed.
* **S11** shares the measurement harness; equality and evaluation order can be
  measured in the same Phase 2 pass.
* **S1** owns whether equality is defined for type parameters.
* The collection library (S9 Q5) depends on Q7.
