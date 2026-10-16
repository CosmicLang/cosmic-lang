# S10 — Reserved Keywords, Annotations, FFI and Contextual Declarations

| | |
|---|---|

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

| **Status** | **OPEN** |
| **Owner** | Engineering (analysis); project owner (acceptance for scope) |
| **Phase** | Phase 1 decision; scope-dependent implementation later |
| **Related** | `docs/spec/language-spec.md` §4.2/§4.4/§4.5, `docs/spec/grammar.ebnf` §3.2, S9 |

## Question

1. **Q1** — What do `entity`, `extern`, `generated`, `audited` and `unique`
   mean? All five are keywords; none is exercised by the golden corpus.
2. **Q2** — What is the **annotation** vocabulary and its semantics?
3. **Q3** — What is the **FFI** surface (`extern`, `foreign module`)?
4. **Q4** — What are the **contextual top-level declarations** `test`,
   `application` and `infra`?
5. **Q5** — What is the **entity/query DSL** parsed by
   `ExpressionParser.parseQueryDsl`?
6. **Q6** — Which **modifier** combinations are legal?

## Evidence

* **E1** — `entity`, `extern`, `generated`, `audited` and `unique` are all in
  the canonical keyword table (`lang/KofKeywords.java`).
* **E1** — The parser has a dedicated branch for `extern` declarations
  (`Parser.parseExternDeclaration`) and for `foreign module` blocks, which
  desugar to `extern` declarations sharing one library
  (`Parser.parseForeignModule`).
* **E1** — The parser has dedicated entry points for `test "<string>" { … }`,
  `application { … }` and `infra "<string>" { … }`, all recognised by word plus
  one-token lookahead, i.e. as **contextual** keywords
  (`Parser.parse`, `parseTestDeclaration`, `parseApplicationDeclaration`,
  `parseInfraDeclaration`).
* **E1** — Annotations (`@`) are parsed by a dedicated component
  (`parser/AnnotationParser.java`) and are only accepted before **type**
  declarations in the top-level dispatch; a function with annotations falls
  through to the type-declaration branch.
* **E1** — The parser has an entity/query DSL entry point
  (`ExpressionParser.parseQueryDsl`), i.e. a dedicated expression form
  associated with an entity type.
* **E1** — The legacy tree documents ORM/"makealive" codegen as a large surface
  (`cosmcc:docs/architecture/makealive-plan.md`), and the ecosystem-coverage document
  lists further surfaces. Neither is executed by the corpus.
* **E1 (negative)** — **None** of these constructs appears in any of the 74
  golden cases. There is therefore **no executed evidence** for Q1–Q6. The
  Phase 0 audit recorded exactly this: these are "reserved in the keyword table;
  semantics not yet analyzed".
* **E1** — The design report (page 9) instructs that Android, UI and the C
  subset be treated as **independent products**, and that no feature be declared
  stable without a specification, a conformance suite and an owner.

## Options

| # | Option | Assessment |
|---|---|---|
| J1 | **Reserve but do not specify.** The keywords stay reserved (so they cannot become identifiers), and every construct is **rejected with a clear "not supported" diagnostic** | Honest, matches the evidence level exactly, and satisfies "no feature declared stable without a specification". Cost: programs using them cannot be compiled |
| J2 | **Specify them** from the legacy source | Would mean writing a specification from *implementation* inspection with no executed behaviour to check against — high risk of enshrining a bug, and it contradicts the project's evidence discipline |
| J3 | **Drop the keywords**, freeing them as identifiers | Breaks C1 for any legacy program using them, and contradicts the legacy design intent (they are reserved deliberately, like `fun`/`fn`/`func`) |
| J4 | **Defer** | The keywords must still be *decided* because they affect whether an identifier is legal |

## Recommendation (PROPOSED, not in force)

**J1.**

* **Q1** — All five keywords **stay reserved** (so they cannot silently become
  identifiers, consistent with the `fun`/`fn`/`func` precedent) and are
  **not specified**. Each produces a clear "reserved keyword not supported in
  this version" diagnostic. This is the only position consistent with having no
  executed evidence.
* **Q2** — Annotations: the **syntax** is specified (an `@`-prefixed qualified
  name with optional arguments, before a type declaration) because it is
  parser-evidenced. The **vocabulary and semantics** are **not** specified. An
  unknown annotation is an error, never silently ignored.
* **Q3** — FFI (`extern`, `foreign module`) is **not specified**. It is a
  security-sensitive surface (the design report page 10 explicitly lists
  "link subprocesses" and "adversarial inputs" as audit targets), and
  specifying an FFI ABI without a runtime design would be premature. Rejected
  loudly.
* **Q4** — `test`, `application` and `infra` are **contextual** and **not
  specified**. Note the consequence that *is* specified: because they are
  contextual, they remain legal identifiers elsewhere, and the grammar must
  encode the lookahead. The test declaration in particular is a *tooling*
  construct, not a language construct, and should be owned by the test
  framework specification rather than by the language.
* **Q5** — The entity/query DSL is **not specified**. It is the clearest
  example of the "diluted scope" risk the design report warns about (page 9):
  a query language embedded in an expression position. If it is wanted, it
  needs its own specification and its own owner.
* **Q6** — Legal modifier combinations are **not specified**. Phase 2 must
  accept the modifiers as tokens and apply a conservative rule (accept the
  evidenced ones, reject combinations without evidence) rather than inventing a
  matrix.

## Compatibility consequence

* J1 means the new toolchain **cannot compile** any legacy program that uses
  entities, annotations, FFI, `test`/`application`/`infra` blocks or the query
  DSL. Since **no golden case uses them**, the executed corpus is unaffected —
  but the *ecosystem* is not, and the owner must know that J1 excludes it.
* This is a **scope decision, not a technical one**, and therefore
  **BLOCKED-ON-OWNER** in substance: engineering recommends J1, but only the
  owner can decide that these surfaces are out of scope.
* No **C0–C3** claim is made for any of these constructs at any level.

## Open dependencies

* **Owner scope decision** for Q1, Q3, Q4, Q5.
* The design report's page 9 instruction ("treat Android, UI and C subset as
  independent products") must be ratified before these surfaces can be formally
  excluded, because excluding them is a scope act.
* S9 owns the library boundary that the FFI and entity surfaces would sit
  across.
