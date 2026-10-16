# Language Compatibility Matrix (seed)

| | |
|---|---|
| **Status** | Working document — Phase 0 seed. Every row cites evidence; absence of a row means *not yet analyzed*, never "unsupported". |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |
| **Executed baseline** | 2026-10-16, macOS host, isolated checkout — see `docs/audits/phase-0-baseline.md` §5 |
| **Rule** | This document is **not a grammar**. Constructs are recorded only with a source-example path; semantics are recorded only when witnessed by an executed check or explicit legacy source. |

Legend for **Legacy evidence** — `G` golden corpus case (executed for the
listed targets), `S` legacy source code inspection (read-only). Executed
targets in the Phase 0 baseline: **jvm ✓, js ✓, script ✓** (all 74 cases);
**native NOT EXECUTED** (GNU-ELF assembly vs macOS assembler — platform
limitation, not a defect claim).

## 1. Verified constructs

| # | Construct (as written) | Example source | Legacy evidence | Notes |
|---|---|---|---|---|
| 1 | Top-level `main()` entry, no function keyword | `hello`: `main() { println("Hello, Kof!") }` | G jvm/js/script | print builtin `println` |
| 2 | Expression-bodied function with explicit return type | `enum-switch-expr`: `String label(Color c) = switch (c) { … }` | G jvm/js/script | |
| 3 | Block-bodied function, leading return type | `nullable-suffix…`: `List<Int>? maybeList(Bool ok) { … }` | G jvm/js/script | |
| 4 | `record` positional declaration + accessor calls | `records`: `record Point(Int x, Int y)`; `p.x()` | G jvm/js/script | accessors are method calls |
| 5 | `class` with fields and `public constructor` | `classes`: `class Animal { String name; public constructor(String name) { this.name = name } }` | G jvm/js/script | `this` assignment; fields mutable (`d.name = "rex2"`) |
| 6 | Inheritance `extends` + `super(...)` | `classes`: `class Dog extends Animal { public constructor(String name) { super(name) } }` | G jvm/js/script | |
| 7 | Method override with expression body | `classes`: `speak(): String = "dog"` | G jvm/js/script | virtual dispatch verified (prints "dog") |
| 8 | `interface` + implementation | `interfaces`: `interface Speaker { speak(): String }`; `class Cat implements Speaker` | G jvm/js/script | `List<Speaker>` polymorphic call |
| 9 | `enum` + `switch` expression with `case X -> value` | `enum-switch-expr`: `enum Color { RED, GREEN, BLUE }` | G jvm/js/script | no fallthrough observed |
| 10 | Nullable suffix types + `null` literal + comparisons | `nullable-suffix…`: `List<List<Int>>? maybeGrid(Bool ok)` | G jvm/js/script | nested generics inside nullable |
| 11 | Generic types, nested type arguments | `nullable-suffix…` / `nested-generic…` cases | G jvm/js/script | see §3 open questions |
| 12 | `var` / `val` bindings | `concurrency-spawn-await`: `var va = await a`; `val a = spawn twice(21)` | G jvm/js/script | |
| 13 | `for (var x in collection)` | `interfaces`: `for (var a in animals) { … }` | G jvm/js/script | |
| 14 | Exceptions: `try` / `catch (Type e)` / `finally`, `throw` of strings | `exceptions`: `throw "boom"`; `catch (String e)` | G jvm/js/script | string throwables |
| 15 | Concurrency: `spawn` → handle, `await` | `concurrency-spawn-await`: `val a = spawn twice(21)` | G jvm/js/script | handle typed by call result |
| 16 | String concatenation `+`, interpolation-free | `exceptions`: `"caught: " + e` | G jvm/js/script | |
| 17 | Collections stdlib style (`listOf`, `get`, higher-order helpers) | `list-predicates`, `list-map-mutation`, `map-set`, `zip-pairs`, `pipelines` cases | G jvm/js/script | full stdlib surface NOT analyzed |
| 18 | Namespaced intrinsics (`math.*`, `json.*`, `encoding.*`, `rng.*`, `time.*`, `config.*`, `observability.*`, …) | 20+ golden cases (`math-namespace`, `json-maps`, `encoding-base64url`, `time-calendar-namespace`, …) | G jvm/js/script | these are runtime/stdlib namespaces, not syntax |
| 19 | No statement-terminating semicolons required | every golden case | G all | legacy lexer treats newline as separator |
| 20 | `fun`/`fn`/`func` are reserved and invalid | `lang/KofKeywords.java` | S | deliberate reserved-but-invalid tokens |

## 2. Source and project file formats (verified by inspection + execution)

| Format | Meaning | Evidence | Status in new toolchain |
|---|---|---|---|
| `.kf` | Kof source (330 files in legacy tree) | S, G builds | Compatibility analysis required (ADR-0001 §4) |
| `.kof` | accepted alias by CLI discovery | S (`KofCliSupport.isKofSource`) | recorded; decision pending |
| `.ptkf` | PortuKof — Portuguese keyword surface (4 files) | S (`D-PORTUKOF` profile) | recorded; decision pending |
| `.ks` | KofScript — statement scripts wrapped into `main()` for the pure-Kof frontend | S (`KofScript.wrapPureKof`) | recorded; semantic analysis required |
| `.csmx` | **proposed** new primary extension | ADR-0001 §3 (investigated, not yet approved) | proposed only |
| `kof.toml` | project config; forgiving INI-subset parser; unknown keys → warning | S (`KofProjectConfig`) + `examples/fullstack/kof.toml` | `Cosmic.toml` proposed, **not implemented** |
| `kofdeps.lock` | generated dependency lock (gitignored); `kof.lock` does not exist | S | `Cosmic.lock` proposed, **not implemented** |

## 3. Unresolved semantic questions (must be answered by tests/spec, not guessing)

1. **Nested generics resolution** — legacy `status.md` §706 records
   `Box<Box<Int>>.get()` resolving at the wrong nesting level (open at this
   SHA). The new compiler must define the correct rule and decide when the
   legacy behavior is a bug (audit §7).
2. **Nullable assignment/extension rules** — which conversions are allowed
   between `T`, `T?` and inferred types (golden cases exercise common paths
   only).
3. **PortuKof aliasing semantics** — which tokens/idioms differ, and whether
   mixed-surface programs exist (only 4 `.ptkf` examples in-tree).
4. **`.ks` wrap semantics** — exact transformations applied to script files
   before compilation (documented only as "materialize as Kof" in source).
5. **Pattern matching breadth** — type patterns, guards, exhaustiveness
   requirements for sealed hierarchies.
6. **Concurrency model contract** — `spawn`/`await` typing, failure
   propagation, and scheduling guarantees per backend.
7. **Numeric semantics** — overflow behavior (`signed-division-overflow`
   case exists), float formatting (`double-formatting` case exists); the
   normative reference must be pinned per operation.
8. **Strings** — escape and indent semantics (`strings-escape-indent` case),
   triple-quoted form (lexer supports it; golden corpus not yet analyzed for
   it).
9. **Standard library boundary** — what is language vs runtime (`math.*`,
   `json.*`, `listOf`, …); the legacy project itself ships boundary checks
   (`check_stdlib_boundary.sh`), i.e. this split was already recognized as
   delicate.
10. **`entity`/`extern`/`generated`/`audited`/`unique` keywords** — reserved
    in the keyword table; semantics not yet analyzed (ORM/FFI/codegen
    surfaces outside the golden corpus).

## 4. Legacy diagnostics vocabulary observed (reference for code design)

Families seen in legacy source: `SEM###` (semantics), `PKG###` (packages),
`COMP###` (driver), `WASM###`, `FFI###`, `MEM###` (memory rules), `L-###`
(leaks), `CONC###`. The new `cosmic-diagnostics` crate adopts the same
`[A-Z]{2,8}[0-9]{3}` shape (see architecture doc §2.2). Numbers are **not**
inherited blindly; new codes are minted with new meanings and the stability
policy of the new crate.

## 5. How rows graduate

A row may claim *compatibility verified* only when a `cosmcc-rs` component
reproduces the behavior under a test that encodes the legacy evidence (case
file + expected output/behavior). Until then rows stay at "legacy evidence"
level, and this matrix never claims more than that.
