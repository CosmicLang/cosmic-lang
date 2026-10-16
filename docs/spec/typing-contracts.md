# Cosmic-Lang Type-System Contracts

| | |
|---|---|
| **Status** | **Proposed — Phase 1 draft.** Not approved. |
| **Phase** | 1 (formal specification) |
| **Directive** | ENGINEERING DIRECTIVE 002, deliverable D3 |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |
| **Companion documents** | `docs/spec/language-spec.md` (D1), `docs/spec/grammar.ebnf` (D2), `docs/spec/decisions/` (D4) |

Evidence tags and normative markers are as defined in
`docs/spec/language-spec.md` §0. **No ownership or borrowing model is specified
here**: the memory strategy is owned by decision record S16 and ADR-0005, and
this document deliberately constrains itself to the type system that the
evidenced language already has.

---

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.


## 1. Type representation

### 1.1 Verified legacy representation (E1)

The legacy analyzer's `Type` is a sealed interface with **eight** variants:

| Variant | Role |
|---|---|
| `PrimitiveType` | `bool`, `byte`, `short`, `int`, `long`, `float`, `double`, `char` |
| `ClassType` | nominal class/interface/record/enum/entity type, with type arguments |
| `TypeVariable` | a generic parameter |
| `FunctionType` | a function/lambda type (evidenced by `() -> Int` type syntax) |
| `ArrayType` | array of an element type |
| `WildcardType` | use-site wildcard; **constructed in practice** (the legacy docs claimed it never was — see `docs/audits/phase-0-baseline.md` §7 item 3) |
| `UnknownType` | the error type |
| `NullableType` | the `T?` wrapper |

Evidence: `docs/audits/phase-0-baseline.md` §3.5, itself E1 from the legacy
source at the frozen SHA.

* `string` is a **primitive type keyword** in the legacy lexer
  (`KofKeywords.java`), while the JVM backend maps it to `java/lang/String`.
  `[E1]` `[E2]` — the surface treats it as primitive; the runtime treats it as a
  reference type. This dual status is a real modelling question and is owned by
  decision record S9.

### 1.2 Proposed new representation [P]

* **[P]** Types are **immutable, interned values** addressed by a stable
  `TypeId`. Interning makes type equality structural-by-construction and cheap,
  and it is what the design report page 13 asks for ("arenas indexed by IDs").
* **[P]** The eight-variant split above is retained as the *semantic* model,
  because it is evidenced. The new representation may reorganise the encoding
  (for example folding `NullableType` into a flag) **only** if a decision record
  records the change and its compatibility consequence.
* **[P]** `UnknownType` is the error type and must never be used as an
  assignability wildcard: an expression whose type is `UnknownType` suppresses
  cascading errors but does not type-check. `[E2]` — this is the standard
  poison-type discipline and the legacy analyzer behaves this way in practice.
* **[UNRESOLVED:S1]** The representation of nested type arguments and their
  identity.

---

## 2. Nominal typing

* **[LEGACY]** Assignability is **nominal**: a breadth-first search over
  ordinary supertypes, with declaration-site variance and use-site wildcards.
  An older structural path consults runtime class relations before relaxing.
  Evidence: `docs/audits/phase-0-baseline.md` §3.5 (E1).
* **[NORMATIVE]** Assignability is nominal in the new toolchain: two class
  types are related only through `extends`/`implements`, never through
  structural similarity.
* **[UNRESOLVED:S2]** Whether the legacy structural fallback is a *bug* to drop
  or a *feature* to keep. Dropping it silently would break programs; keeping it
  silently would make the type system unsound-looking. A decision record must
  choose, with differential evidence.
* **[UNRESOLVED:S1]** Variance: the legacy analyzer has both declaration-site
  variance and use-site wildcards. Which is normative, and how they interact, is
  not settled.

---

## 3. Generics

* **[LEGACY]** Generic type arguments are written `<…>` and may nest; generic
  functions are declared (corpus case `generic-toplevel-function`); generic
  calls may carry explicit type arguments
  (`ExpressionParser.parseCallTypeArguments`).
* **[LEGACY]** Erasure is the runtime model (generics are not reified).
* **[E2]** The legacy ledger records an **open defect** in nested generic
  resolution: `Box<Box<Int>>.get()` resolving at the wrong nesting level
  (`cosmcc:docs/status.md` §706 at the frozen SHA). The new compiler must define the
  *correct* rule and decide explicitly that the legacy behaviour is a bug.
* **[UNRESOLVED:S1]** The resolution rule for nested type arguments, the
  inference rule for omitted type arguments, and the bounds syntax (no
  declaration-site bound syntax is evidenced at all).
* **[NORMATIVE]** No generic *declaration* syntax is specified by this document,
  because none is evidenced. Phase 2 must not implement a guessed syntax.

---

## 4. Nullability

* **[LEGACY]** `T?` is a nullable wrapper type. The corpus exercises nullable
  suffix types (`nullable-suffix-leading-return-type`), nested generics inside a
  nullable type, `null` literals and null comparisons, and a
  `nullable-soft-cast` case, plus `std-math-nullable`.
* **[UNRESOLVED:S2]** The conversion rules — which of `T → T?`, `T? → T`,
  `T? → U?` are implicit, what a "soft cast" is, and how nullability composes
  with type parameters and with generic arguments — are **not** specified. The
  corpus exercises common paths only.
* **[NORMATIVE]** Until S2 is decided, no nullability conversion is
  normative, and the new frontend must not infer one.

---

## 5. Inference

* **[LEGACY]** The legacy architecture has **no typed AST**: expression types
  live in analyzer-side maps and are re-inferred during lowering. The design
  report calls this out (page 4) as a structural risk.
* **[P]** The new toolchain uses a **typed HIR with stable node IDs** and
  records the resolved type on the node, so that later phases never re-infer.
  This is what the design report page 4 asks for and what the Phase 0
  conformance audit classified as DEFERRED to Phase 1/2.
* **[P]** Inference is **local and bounded**: it is driven by the declared types
  at the boundaries (parameters, return types, field types) and by local
  unification inside an expression. No whole-program inference, no inference
  that escapes a function boundary.
* **[UNRESOLVED:S1]** The exact inference boundary — which expressions may omit
  a type, whether a lambda parameter may omit its type, whether `var` without an
  initialiser is legal, and whether the return type may be inferred from the
  body. (S1 owns generics *and* the inference boundary; see its Q3.)
* **[NORMATIVE]** Inference must never *silently* pick a widening type. Where
  the evidence shows a silent widening (integer-literal overflow promotion,
  `docs/spec/language-spec.md` §2.4), the decision record S7 owns whether to
  keep it.

---

## 6. Overload resolution

* **[LEGACY]** Overload resolution exists and lives in dedicated components
  (`MethodCallTyper` / `OverloadSelector`); the analyzer's own
  `resolveMethodCalls` phase is a documented no-op vestige.
  Evidence: `docs/audits/phase-0-baseline.md` §3.5 (E1).
* **[UNRESOLVED:S1]** The applicability and specificity rules are not written
  down anywhere in the legacy tree and are not derivable from the golden corpus,
  which contains no overload-ambiguity case. This is a genuine specification
  gap: it must be closed by targeted differential tests in Phase 2, not by
  guessing now.
* **[NORMATIVE]** Where the language is ambiguous, the new compiler must emit an
  ambiguity diagnostic rather than choosing by declaration order. The legacy
  behaviour (declaration order as a tie-break) is not evidence of a language
  rule.

---

## 7. Coercions

* **[LEGACY]** The corpus evidences explicit conversions via `as`
  (`type-check-casts`, `numeric-casts`, `bitwise-ops`) and `instanceof`.
* **[LEGACY]** Numeric narrowing/widening and saturation behaviour is exercised
  (`numeric-casts`, `signed-division-overflow`, `byte-arith`,
  `long-arithmetic`, `double-formatting`).
* **[UNRESOLVED:S7]** Which numeric conversions are implicit, which are
  explicit, the overflow rule for each, and the float/double formatting
  contract. The Phase 0 baseline audit notes that a legacy ledger item records
  *cross-target divergence* in shift semantics (riscv64/aarch64/JS versus the
  JVM), so the numeric contract cannot be assumed to be uniform across targets.
* **[NORMATIVE]** No implicit narrowing coercion is proposed. Any implicit
  conversion must be listed explicitly in decision record S7 before Phase 2 may
  implement it.

---

## 8. Typed IR invariants

* **[LEGACY]** The legacy IR is a typed linear stack machine: most operations
  carry a `Type`, and that type has a **functional** role in consumption (for
  example a conditional-jump operation's operand type selects the JVM
  comparison instruction). It has **no formal semantics of its own** — no
  dependency graph, no value domain. Evidence: the legacy architecture document
  as re-verified in `docs/audits/phase-0-design-conformance.md` §3.
* **[P]** The new toolchain makes the typed representation carry its own
  semantics, so that a verifier can check it. The invariants that must hold when
  a typed representation crosses into MIR:

| # | Invariant | Rationale |
|---|---|---|
| TI-1 | Every expression node has a resolved type, and that type is not `UnknownType` on the path to code generation | Prevents poison types leaking into MIR |
| TI-2 | Every type reference resolves to a declared type, or the compilation fails with a diagnostic | No implicit or guessed types |
| TI-3 | Every type argument arity matches its declaration's parameter count | Prevents the nested-generic class of defect (S1) |
| TI-4 | Every name binding is resolved to exactly one declaration, or the compilation fails | No ambiguous bindings |
| TI-5 | Every `var`/`val` binding has a declared or inferred type fixed **before** MIR construction; no inference occurs after this point | Makes MIR construction deterministic |
| TI-6 | Every function type's arity matches its use site | Lambda/call arity safety |
| TI-7 | Nullability is represented in the type, never in a side table | Prevents the legacy side-map drift the design report warns about |
| TI-8 | An `as`/`instanceof` operand's type is a declared type, not an expression-derived one | Matches the verified grammar (`§6.1`) |

* **[UNRESOLVED:S18]** These are **proposals**. S18 owns the final invariant
  list, its enforcement points, and the MIR verifier's obligations.
* **[NORMATIVE]** Whatever S18 decides must be checkable mechanically after
  lowering and after each optimization pass, per the design report page 15.

---

## 9. What this document deliberately does not specify

| Not specified | Owner | Reason |
|---|---|---|
| Ownership, borrowing, lifetimes | S16 + ADR-0005 | The design report requires a *study* before a choice; the legacy surface has no ownership syntax to be compatible with |
| Reference counting, GC or arena allocation | S16 + ADR-0005 | Same |
| Escape analysis and its type-level effects | S16 | Depends on S16 |
| Effect typing (I/O, exceptions) | S6 | The design report asks for effect checks, but the evidenced surface has only `spawn`/`await` and exceptions |
| Trait/interface default-method resolution details | S1 | Overlaps overload resolution, which is itself unresolved |
| Variance declaration syntax | S1 | Not evidenced |

**Rationale for the restraint.** The design report page 17 explicitly warns:
"Do not claim that Rust automatically makes compiled code safe", and it requires
a study before choosing a memory strategy. Inventing an ownership model here
would be exactly the kind of unevidenced decision Phase 0's review discipline
forbids, and would pre-empt ADR-0005.
