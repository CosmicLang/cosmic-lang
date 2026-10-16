# S2 — Nullability and the Structural Fallback

| | |
|---|---|
| **Status** | **OPEN** |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision, with Phase 2 differential validation |
| **Related** | `docs/spec/typing-contracts.md` §2/§4, `docs/spec/language-spec.md` §6.1, `docs/spec/grammar.ebnf` §3.4 |

## Question

1. **Q1** — Which conversions between `T`, `T?` and `U?` are **implicit**?
2. **Q2** — What is a **"soft cast"**? The corpus has a `nullable-soft-cast`
   case but the language specification has never stated what it means.
3. **Q3** — How does nullability compose with **type parameters**
   (`T?` where `T` is a type variable) and with generic arguments
   (`List<Int?>` versus `List<Int>?`)?
4. **Q4** — Is the legacy **structural fallback** (assignability that consults
   runtime class relations before relaxing) part of the language, or a defect?

## Evidence

* **E1** — `NullableType` is one of the eight `Type` variants, and the `?`
  suffix is evidenced by the corpus (`nullable-suffix-leading-return-type`,
  `nullable-soft-cast`, `nullable-suffix…`, `std-math-nullable`), executed
  222/222 on jvm/js/script.
* **E1** — The corpus exercises nested generics *inside* a nullable type, so
  `List<List<Int>>?` is valid syntax and behaves.
* **E1** — `null` is a literal token (`KofKeywords.java`), and null comparisons
  are exercised.
* **E1** — The legacy analyzer has an older structural assignability path that
  consults runtime class relations before relaxing. The Phase 0 audit records it
  as E1 but does not characterise when it fires.
* **E1 (negative)** — No case exercises `T? → T`, `T → T?` where `T` is a type
  parameter, or the interaction of nullability with wildcards.

## Options

| # | Option | Assessment |
|---|---|---|
| B1 | **Keep the legacy rules exactly**, deriving them from source, and write them down | Maximum compatibility; but the structural fallback may be unsound, and "derive from source" risks documenting a bug as a rule |
| B2 | **Strict nullable types**: `T?` and `T` are distinct, the only implicit direction is `T → T?`, and every `T? → T` requires an explicit check or cast | Sound and teachable; a **C2 divergence** wherever the legacy compiler allowed an implicit unwrap |
| B3 | **Defer entirely** to Phase 2 | Leaves the type system without a nullability contract, which blocks the typed HIR |

## Recommendation (PROPOSED, not in force)

**B2 for Q1/Q3, with Q2 and Q4 explicitly deferred and measured.**

* **Q1** — The only implicit conversion is **`T → T?`** (widening into
  nullability). Every other direction requires an explicit construct. Rationale:
  a nullable-to-non-nullable implicit conversion is exactly the kind of silent
  reinterpretation the project's own rules forbid.
* **Q2** — **Deferred.** "Soft cast" is named in a case directory but never
  defined. Phase 2 must read the executed behaviour of that case and produce a
  definition, then this record is updated. No rule is invented now.
* **Q3** — `T?` and `T` are distinct for type parameters as well; generic
  arguments carry their own nullability (`List<Int?>` ≠ `List<Int>?`). This is
  the only reading consistent with the `?`-as-suffix grammar.
* **Q4** — **Deferred, and flagged as suspected defect.** The structural
  fallback is not a language feature anyone designed; it is a compatibility
  shim. Phase 2 must determine when it fires and whether dropping it changes any
  corpus result. If it changes nothing, drop it. If it changes something, it
  becomes an explicit decision with evidence.

## Compatibility consequence

* B2 is a **C2 divergence** wherever the legacy compiler allowed an implicit
  nullable unwrap. The magnitude is unknown and must be measured by running the
  legacy corpus through the new frontend in Phase 2. Until then, this
  recommendation must **not** be treated as settled.
* The grammar is unaffected: the `?` suffix production is unchanged.

## Open dependencies

* Phase 2 must define "soft cast" from the executed case before this record can
  close.
* S1 (Q3, variance) interacts: nullability plus wildcards is an unanalysed
  combination.
