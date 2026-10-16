# S9 — Standard-Library Boundary, Member Kinds and Module Rules

| | |
|---|---|
| **Status** | **OPEN** |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; ADR-0003 owns naming |
| **Related** | `docs/spec/language-spec.md` §4.1/§4.4/§6.1, `docs/spec/typing-contracts.md` §1.1, ADR-0003 |

## Question

1. **Q1** — Where is the line between **language** and **standard library**? Is
   `println` a language builtin or a library function?
2. **Q2** — Which members are **properties** (accessed without parentheses, e.g.
   `list.size`) and which are **methods** (called with parentheses)?
3. **Q3** — Is `string` a **primitive type** (it is a keyword) or a **reference
   type** (the JVM backend maps it to `java.lang.String`)?
4. **Q4** — What are the rules for `package` and `import`: optionality,
   ordering, wildcards, aliases, static imports?
5. **Q5** — What is the **collection** surface (`listOf`, `get`, `map`, `zip`,
   higher-order helpers) and is it language or library?

## Evidence

* **E1** — The legacy project ships a **dedicated boundary checker**:
  `scripts/check_stdlib_boundary.sh` (5827 bytes) exists at the frozen SHA. The
  project itself treated the language/library split as delicate enough to be
  enforced by tooling. That is strong evidence that the boundary is **not
  self-evident** and must be specified explicitly.
* **E1 (executed)** — **24** of the 74 golden cases are namespace cases:
  `cache-namespace`, `config-namespace`, `encoding-base64url`,
  `encoding-namespace`, `json`, `json-escaping`, `json-maps`, `json-records`,
  `jwt-namespace`, `math-namespace`, `math-parse-namespace`,
  `math-pow-constants`, `observability-namespace`, `passwords-namespace`,
  `random-namespace`, `rng-namespace`, `security-crypto-namespace`,
  `std-math-nullable`, `strings-namespace`, `time-calendar-namespace`,
  `time-namespace`, `uuid-namespace`, `validation-namespace`,
  `validation-predicates`. All pass 222/222 on jvm/js/script.
  So roughly **one third of the executed compatibility surface is library
  surface**, not syntax.
* **E1** — Namespace calls are written as dotted calls (`math.…`, `json.…`),
  i.e. they use the same member-access syntax as object methods. Nothing in the
  grammar distinguishes a namespace from a receiver.
* **E1** — `string` is a **primitive-type keyword** (`lang/KofKeywords.java`)
  while the JVM backend maps it to a reference type
  (`docs/audits/phase-0-baseline.md` §3.6, `JvmTypeMapper`). Its dual status is
  an evidenced fact, not an inference.
* **E1** — Property-style access **without parentheses** is evidenced:
  `Color.values().size` and collection `.size` appear in the corpus
  (`enum`, `collections`).
* **E1** — `package` and `import` are parsed as a plain qualified name; no
  wildcard, alias or static-import form is produced by the legacy parser
  (`parser/Parser.java` `parsePackage`, `parseImports`).
* **E1** — Collection helpers (`listOf`, `get`, `map`, `zip`, predicates) appear
  in the corpus as ordinary calls (`list-higher-order`, `list-map-mutation`,
  `list-predicates`, `collections`, `collection-concat`, `map-methods`,
  `map-set`, `zip-pairs`, `pipelines`).

## Options

| # | Option | Assessment |
|---|---|---|
| I1 | **Language = syntax + type system only.** Everything callable is library, including `println`. Namespaces are library modules | Clean, principled, and it makes the standard library versionable independently of the language. Cost: the language has no builtins at all, so a target must supply the library to run anything |
| I2 | **A small set of language builtins** (print/println, collection literals, `listOf`) plus everything else as library | Matches practice and the corpus, but hard-codes a boundary that the legacy project needed a script to police |
| I3 | **Language = syntax + type system + the evidenced namespace surface**, with namespaces treated as part of the compatibility contract | Maximum compatibility, but it makes the language spec enormous and couples it to 24 namespace cases |
| I4 | **Defer** | Leaves the frontend unable to resolve `println`, which the very first golden case needs |

## Recommendation (PROPOSED, not in force)

**I1 for the *boundary rule*, with I2's pragmatism for the *first target*.**

* **Q1** — The **boundary rule** is I1: the language specification defines
  syntax, types and semantics; **every callable name, including `println`, is
  provided by the standard library**. The library is specified separately with
  its own version and its own conformance suite. Rationale: the legacy project
  needed a script to police this boundary, which proves the boundary cannot be
  left implicit; making it a rule removes the ambiguity permanently. The first
  target still ships the library as part of the toolchain, so `hello` works.
* **Q2** — Members are **methods by default**; a property access without
  parentheses is allowed only for members the declaration marks as a property
  (an accessor with no parameters, or a field-like accessor such as a record
  component or `size`). Rationale: the corpus evidences both forms, and the
  distinction must be a declaration property, not a naming convention.
* **Q3** — `string` remains a **language-level type name** that is *modelled* as
  a reference type with value semantics. The specification states this
  explicitly rather than letting the keyword/backend mismatch stand unexplained.
  **This is a modelling decision the owner should confirm**, because it affects
  equality and identity (S12).
* **Q4** — `package` is **optional** and must precede imports; imports must
  precede declarations. **Wildcards, aliases and static imports are not
  specified** (no evidence). Phase 2 must reject them rather than invent them.
* **Q5** — Collections are **library**, per Q1, but the collection surface used
  by the corpus is part of the **compatibility contract** and therefore needs a
  library specification of its own. That library specification is **not**
  produced by Directive 002; it is proposed as a Directive 003+ deliverable.
  Until it exists, no collection compatibility is claimed.

## Compatibility consequence

* Q1 changes no syntax and no runtime behaviour, so it is **C1/C2 neutral**. It
  changes *where* the contract lives.
* Q3 must be confirmed before equality semantics can be fixed (S12), because
  value versus reference equality for strings is observable.
* Q4 is a **C1 divergence** only if a legacy program uses a wildcard or alias
  import. Since the legacy parser does not produce those forms, no such program
  can exist — the divergence risk is **zero**, and the rule is a clarification.

## Open dependencies

* **ADR-0003** owns the standard-library naming (`cosmic-std` versus
  `cosmic.core`/`cosmic.io`/…, which the design report names both ways).
* The collection and namespace surfaces need their own specification before any
  compatibility claim.
* Q3 needs owner confirmation.
