# Cosmic-Lang Language Specification

| | |
|---|---|
| **Status** | **Proposed — Phase 1 draft.** Not approved. Normative content is marked as such; everything else is explicitly non-normative. |
| **Phase** | 1 (formal specification) |
| **Directive** | ENGINEERING DIRECTIVE 002, deliverable D1 |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |
| **Evidence basis** | Legacy source inspection at the frozen SHA (read-only) and the executed golden corpus recorded in `docs/audits/phase-0-baseline.md` §5 |
| **Companion documents** | `docs/spec/grammar.ebnf` (D2), `docs/spec/typing-contracts.md` (D3), `docs/spec/decisions/` (D4), `docs/spec/compatibility-policy.md` (D5), `docs/spec/diagnostic-codes.md` (D9) |
| **Provenance rule** | No legacy implementation code, documentation prose or test fixture is reproduced here. Behaviour and facts are cited by file path and SHA; every example is written for this document. See `docs/audits/phase-0-licensing-matrix.md` §4. |

---

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.


## 0. How to read this document

### 0.1 Evidence tags

Every claim carries two orthogonal tags: an **evidence** tag (how we know) and a
**status** tag (what the claim is worth as a rule).

### 0.1 Evidence tags

| Tag | Meaning |
|---|---|
| **E1** | Directly verified — read in the legacy source at the frozen SHA, witnessed by an executed golden case, or reproduced by an executed probe (`docs/audits/phase-1-legacy-probes.md`) |
| **E2** | Evidence-supported inference — reasoned from E1 material, not itself executed |
| **H** | Hypothesis — plausible but unverified; **never normative** |

### 0.2 Normative status

The six statuses below are the **only** permitted status words. They are the
vocabulary the directive requires, and they are the vocabulary the decision
register uses.

| Status | Meaning |
|---|---|
| **[VERIFIED LEGACY BEHAVIOR]** | The statement describes behaviour **observed in the legacy compiler**. It is evidence. It is **not** automatically a requirement, and it is **not** a rule until a decision record says so |
| **[PROPOSED NORMATIVE RULE]** | The statement proposes required behaviour for the new toolchain. **No rule in this document has been accepted by the owner**, so nothing here is an accepted contract |
| **[ACCEPTED NORMATIVE RULE]** | An owner-accepted rule. **This document currently contains none** — the status is defined so that the distinction is explicit, and so that a rule cannot drift from PROPOSED to ACCEPTED by being read repeatedly |
| **[OPEN]** | Deliberately not decided. A decision record owns it. An open construct must be **refused** by the implementation, never guessed |
| **[DEFERRED]** | Not needed by the component that consumes this part of the specification. A later phase owns it |
| **[NOT VERIFIED]** | Not established either way. Absence of evidence is recorded as absence, never as support |

### 0.3 How to read a statement

* A statement marked **[VERIFIED LEGACY BEHAVIOR]** alone describes the old
  language. It may be changed by a decision record.
* A statement marked **[PROPOSED NORMATIVE RULE]** is a *proposal*. It becomes
  binding only when the owner accepts it — at which point it would be marked
  **[ACCEPTED NORMATIVE RULE]**, which no statement in this document currently
  is.
* Where a rule and the observed behaviour **differ**, both are stated and the
  difference is named as a **C2 divergence** under
  `docs/spec/compatibility-policy.md`.

**This distinction is the point of the document.** The Phase 0 conformance audit
found that the legacy repository's own documentation had drifted from its code
in six independent places, so behaviour is quoted from source and marked as
behaviour, never silently promoted to requirement. Equally, a *proposal* must
never be read as an *accepted contract*: **this entire specification is
Proposed.**

**Gate status.** Which of these statuses a component needs before it can be
built is defined in `docs/spec/implementation-gates.md` — an `OPEN` question
does not block a component that does not consume it, and where it does block,
the construct is **gated** (refused) rather than guessed.

### 0.4 Scope

This specification describes **the language the legacy compiler implements**,
as evidenced, and the boundaries the new toolchain must respect. It does **not**
describe the compiler's internal architecture (that is
`docs/architecture/compiler-pipeline.md`) and does **not** authorise
implementation (Phase 2 is not authorised).

---

## 1. Source model

### 1.1 Encoding and BOM

* **[VERIFIED LEGACY BEHAVIOR]** Source text is UTF-8. A single leading UTF-8 BOM (`EF BB BF`,
  U+FEFF) is tolerated and **is not a code character**: the lexer starts after
  it. Evidence: `parser/Lexer.java` (`tokenize`, the `startsWith("\uFEFF")`
  guard, comment `OBS-008`).
* **[PROPOSED NORMATIVE RULE]** The new frontend accepts the same BOM tolerance, and must
  report invalid UTF-8 with the byte offset of the first invalid byte
  (already implemented in `cosmic-source::SourceMap::load`).

### 1.2 Line terminators

* **[VERIFIED LEGACY BEHAVIOR]** `\n`, `\r\n` and a lone `\r` each terminate a line. In the
  CRLF case the pair is consumed as **one** line break. Evidence:
  `parser/Lexer.java` `tokenize` (the `'\n' || '\r'` branch).
* **[PROPOSED NORMATIVE RULE]** The same universal-terminator rule applies, and a trailing
  terminator opens one final empty line (`"a\n"` has two lines). Implemented
  and tested in `cosmic-source`.

### 1.3 Columns — **known legacy/new divergence**

* **[VERIFIED LEGACY BEHAVIOR]** The legacy lexer advances its column counter once per UTF-16
  **code unit** (`Lexer.java` `advance()` increments `column` by one, and
  `source.charAt` addresses code units). An astral character therefore
  occupies **two** legacy columns.
* **[PROPOSED NORMATIVE RULE]** The new toolchain counts columns in Unicode **scalar values**
  (one per `char`), as documented in `docs/architecture/compiler-pipeline.md`
  §2.1 and implemented in `cosmic-source`.
* **Consequence (E2):** for any source containing astral characters, legacy
  and new column numbers differ from the first astral character on a line.
  Because diagnostics are keyed to line/column, a byte-for-byte comparison of
  legacy and new diagnostics will differ on such lines even when both are
  correct. **This is recorded as an intentional divergence**, owned by decision
  record S13 (`docs/spec/decisions/S13-unicode.md`). Differential tests must
  normalise for it or restrict to the Basic Multilingual Plane.
* **[VERIFIED LEGACY BEHAVIOR]** The legacy lexer reports positions as `file, line, column`
  plus an offset/length pair, not as byte spans. **[PROPOSED NORMATIVE RULE]** the new
  toolchain uses validated byte spans (`cosmic_source::Span`) as the
  authoritative coordinates and derives line/column for presentation.

### 1.4 Whitespace

* **[VERIFIED LEGACY BEHAVIOR]** Space (`U+0020`), tab (`U+0009`) and form feed (`U+000C`) are
  whitespace. No other character is. Evidence: `Lexer.java` `tokenize`.
* **[PROPOSED NORMATIVE RULE]** The same three characters. Note the practical consequence:
  a non-breaking space or an ideographic space is **not** whitespace and will
  be an unexpected-character diagnostic.

---

## 2. Lexical structure

### 2.1 Comments

* **[VERIFIED LEGACY BEHAVIOR]** `//` begins a line comment, terminated by end of line. `/*`
  begins a block comment, terminated by `*/`. A block comment may span lines.
  An unterminated block comment produces one diagnostic (`LEX001`). Evidence:
  `Lexer.java` `skipLineComment`, `skipBlockComment`.
* **[OPEN]** Whether block comments **nest**. The legacy lexer's
  `skipBlockComment` scans to the first `*/`, so they do not nest; whether the
  new language should nest is a decision (record S14).

### 2.2 Identifiers

* **[VERIFIED LEGACY BEHAVIOR]** An identifier starts with a Unicode letter, `_` or `$`, and
  continues with Unicode letters/digits, `_` or `$`. Evidence:
  `Lexer.java` `tokenize` (start test `Character.isLetter(c) || '_' || '$'`)
  and `readIdentifier` (continue test `isLetterOrDigit || '_' || '$'`).
* **[E2]** Because `Character.isLetter` is Unicode-aware, non-ASCII letters are
  valid identifier characters. This is not exercised by the golden corpus.
* **[PROPOSED NORMATIVE RULE]** Identifiers are Unicode letters/digits plus `_` and `$`, with
  the same start restriction, **subject to** decision record S13 (which owns
  normalisation and the treatment of confusable characters).
* **[OPEN]** Whether `$` should remain permitted in new code
  (record S13).

### 2.3 Keywords

* **[VERIFIED LEGACY BEHAVIOR]** The keyword table is a single canonical map shared by the Kof
  lexer profile and used as the base vocabulary for the PortuKof profile.
  Evidence: `lang/KofKeywords.java`.

The verified keyword set (E1) is:

| Group | Keywords |
|---|---|
| Type declarations | `class`, `interface`, `record`, `enum`, `entity` |
| Declaration modifiers / markers | `extern`, `generated`, `audited`, `unique` |
| Inheritance | `extends`, `implements` |
| **Reserved and invalid** | `fun`, `fn`, `func` |
| Packages | `package`, `import` |
| Access / member modifiers | `public`, `private`, `protected`, `static`, `final`, `abstract`, `transient`, `volatile`, `synchronized`, `native`, `default`, `override` |
| Values and objects | `void`, `new`, `this`, `super` |
| Control | `return`, `throw`, `if`, `else`, `for`, `while`, `do`, `switch`, `case`, `break`, `continue` |
| Exceptions | `try`, `catch`, `finally` |
| Concurrency | `spawn`, `await` |
| Other operators/keywords | `assert`, `instanceof`, `var`, `val`, `as` |
| Primitive types | `bool`, `byte`, `short`, `int`, `long`, `float`, `double`, `char`, `string` |
| Literals | `true`, `false` (both tokenise as a boolean literal), `null` |

* **[VERIFIED LEGACY BEHAVIOR]** `fun`/`fn`/`func` are **reserved but invalid**: they tokenise to
  dedicated token types and the parser rejects them explicitly rather than
  letting them fall back to identifiers. Evidence: `KofKeywords.java` header
  comment (`SG-001`) and `Parser.java` `parse()` (`rejectFunctionKeyword`).
* **[PROPOSED NORMATIVE RULE]** The new language keeps the same reserved-but-invalid set and
  must never silently accept them as identifiers. This is the legacy project's
  own "fail loudly, never silently" rule, and Phase 0's CLI already follows it.
* **[OPEN]** The semantics of `entity`, `extern`, `generated`, `audited`,
  `unique` are not established by the golden corpus (record S10).

### 2.4 Literals

#### Integer and floating point

* **[VERIFIED LEGACY BEHAVIOR]** Decimal integer literals; hexadecimal literals prefixed `0x` or
  `0X`; optional suffixes `l`/`L` (long), `f`/`F` (float), `d`/`D` (double); a
  fraction only when `.` is immediately followed by a digit; an exponent
  `e`/`E` with an optional sign. Evidence: `Lexer.java` `readNumber`.
* **[VERIFIED LEGACY BEHAVIOR]** A decimal literal that does not fit in a 32-bit signed integer
  is **promoted** to a long literal rather than being an error; a hexadecimal
  literal is int unless suffixed. Evidence: `readNumber` +
  `isIntegerLiteral` (`Integer.parseInt` failure ⇒ `LONG_LITERAL`).
* **[VERIFIED LEGACY BEHAVIOR]** Not supported: binary or octal literals, digit separators
  (`1_000`), a trailing `.` with no fraction digits, hex floats, and any
  literal starting with `.`. Evidence: `readNumber` structure.
* **[OPEN]** Whether the implicit int→long promotion on overflow is
  desirable in the new language (record S7). It is a silent widening, which
  sits awkwardly with the "never silently degrade" principle.
* **[OPEN]** The exact overflow/rounding behaviour of float and double
  literals (record S7).

#### Strings

* **[VERIFIED LEGACY BEHAVIOR]** A string is `"…"`, single-line in practice: a raw newline inside
  the literal is consumed and the line counter is advanced, so a literal may
  span lines in the lexer, but this is not exercised by the corpus.
  Unterminated ⇒ `LEX002`. Evidence: `Lexer.java` `readString`.
* **[VERIFIED LEGACY BEHAVIOR] Triple-quoted strings do not exist and are actively rejected.**
  `"""` is recognised and produces `LEX008` with the message that Kof has no
  raw/multiline string literal and no interpolation, and that the user should
  use `"...\n..."` or concatenate with `+`. Evidence: `Lexer.java`
  `skipTripleQuoted`.
  * **Correction to a Phase 0 claim.** `docs/spec/compatibility-matrix.md` §3
    item 8 recorded "triple-quoted form (lexer supports it)". That is **wrong**:
    the lexer recognises the sequence only to reject it. This specification is
    authoritative over that matrix row; the matrix is corrected in the same
    change.
* **[VERIFIED LEGACY BEHAVIOR]** No string interpolation. Concatenation is `+`. Evidence: the
  `LEX008` message and the corpus.
* **[OPEN]** Whether the new language keeps the rejection (a proposal is
  in record S8).

#### Escape sequences

* **[VERIFIED LEGACY BEHAVIOR]** `\n`, `\t`, `\r`, `\\`, `\'`, `\"`, `\0`, and `\uXXXX`.
  Evidence: `Lexer.java` `readEscape`.
* **[VERIFIED LEGACY BEHAVIOR]** An **unrecognised** escape yields the escaped character itself
  (`default -> c`), with **no diagnostic**. So `"\q"` lexes as `q`.
* **[VERIFIED LEGACY BEHAVIOR]** `\uXXXX` is a single UTF-16 code unit; an incomplete escape is
  `LEX006` and an invalid hex digit is `LEX007`. Astral characters therefore
  require a surrogate pair of escapes. Evidence: `readUnicodeEscape`.
* **[OPEN]** Whether an unknown escape should keep being silent. A silent
  reinterpretation is exactly the class of behaviour the project's own rules
  forbid; a proposal is in record S8.

#### Characters

* **[VERIFIED LEGACY BEHAVIOR]** `'x'` with the same escape set; empty (`''`) ⇒ `LEX003`;
  unterminated ⇒ `LEX004`. Evidence: `Lexer.java` `readChar`.

#### Booleans and null

* **[VERIFIED LEGACY BEHAVIOR]** `true` and `false` both tokenise to a boolean literal token;
  `null` to a null literal token. Evidence: `KofKeywords.java`.

### 2.5 Operators and delimiters

* **[VERIFIED LEGACY BEHAVIOR]** The complete verified set (E1, `Lexer.java`
  `readOperatorOrDelimiter`, cross-checked against `TokenType.java`):

| Class | Tokens |
|---|---|
| Arithmetic | `+` `-` `*` `/` `%` |
| Comparison | `==` `!=` `<` `<=` `>` `>=` |
| Logical | `&&` `\|\|` `!` |
| Bitwise | `&` `\|` `^` `<<` `>>` `>>>` |
| Assignment | `=` `+=` `-=` `*=` `/=` `%=` `&=` `\|=` `^=` `<<=` `>>=` `>>>=` |
| Increment/decrement | `++` `--` |
| Lambda / switch arrow | `->` |
| Delimiters | `(` `)` `{` `}` `[` `]` |
| Separators | `;` `,` `.` `:` `?` |
| Annotation | `@` |

* **[VERIFIED LEGACY BEHAVIOR] There is no bitwise complement operator.** `~` is not in the
  token set and falls through to the unexpected-character branch (`LEX005`).
  Evidence: absence from `TokenType.java` and the `default` arm of
  `readOperatorOrDelimiter`. **[OPEN]** whether to add one (record S7).
* **[VERIFIED LEGACY BEHAVIOR] Semicolons are not required** as statement terminators; statements
  are separated by line structure and grammar. `;` exists as a token and is
  accepted. Evidence: every golden case omits them; the token exists.
* **[VERIFIED LEGACY BEHAVIOR]** Any other character is `LEX005` "unexpected character" and
  produces an error token rather than a panic. Evidence: the `default` arm.

### 2.6 Lexical diagnostics observed (E1)

`LEX001` unterminated block comment · `LEX002` unterminated string ·
`LEX003` empty character literal · `LEX004` unterminated character literal ·
`LEX005` unexpected character · `LEX006` incomplete `\u` escape ·
`LEX007` invalid `\u` escape · `LEX008` triple-quoted string rejected.

**[PROPOSED NORMATIVE RULE]** These are legacy *codes*. The new toolchain mints its own
`LEX###` codes with its own meanings under the stability policy in
`docs/spec/diagnostic-codes.md`; legacy numbers are **not** inherited.

---

## 3. Trivia and normalisation policy

* **[VERIFIED LEGACY BEHAVIOR]** Comments and whitespace are discarded by the lexer; there is no
  lossless token stream, so the legacy tooling cannot round-trip formatting.
* **[PROPOSED NORMATIVE RULE]** The new frontend keeps a **lossless CST** that preserves
  trivia (whitespace, line breaks, comments) exactly, so a formatter and an LSP
  can round-trip source. This follows the design report page 14.
* **[PROPOSED NORMATIVE RULE]** Normalisation is permitted **only** for the leading BOM
  (§1.1). No other character is normalised, reordered or dropped.
* **[OPEN]** Whether the CST should also preserve a canonical form of
  line terminators (record S14).

---

## 4. Declarations

### 4.1 Compilation unit

* **[VERIFIED LEGACY BEHAVIOR]** A compilation unit is: an optional `package` declaration,
  then zero or more `import` declarations, then zero or more top-level
  declarations. Evidence: `Parser.java` `parse()`.
* **[OPEN]** Whether `package` is truly optional and whether it may
  appear after imports (record S9).

### 4.2 Top-level declarations

* **[VERIFIED LEGACY BEHAVIOR]** The parser dispatches, in order: annotations, then
  * `fun`/`fn`/`func` → **rejected** (reserved-invalid, §2.3);
  * `test "<string>"` → a test declaration whose body is a statement block;
  * `application { … }` → an application declaration;
  * `infra "<string>"` → an infrastructure declaration;
  * an annotated `class`/`interface`/`record`/`entity` → a type declaration;
  * `extern` → an external (FFI) declaration;
  * `foreign module …` → sugar that desugars to `extern` declarations sharing
    one library;
  * `sealed` before a type declaration → a sealed type;
  * an identifier, `void`, a primitive type name, or `(` → a **function
    declaration**;
  * otherwise → a type declaration.

  Evidence: `Parser.java` `parse()`.

* **[E1] `test`, `application`, `infra`, `foreign module` and `sealed` are
  contextual keywords**, recognised by word + following-token lookahead, not by
  the keyword table. Consequence (E2): they remain usable as identifiers
  elsewhere, and a grammar must encode the lookahead rather than reserving the
  words.
* **[OPEN]** `test`, `application` and `infra` declarations are **outside
  the golden corpus**. Their semantics are not established here and are not
  normative. Records S9 and S10 own them.
* **[OPEN]** The FFI surface (`extern`, `foreign module`) is outside the
  golden corpus (record S10).

### 4.3 Functions

* **[VERIFIED LEGACY BEHAVIOR]** A function has an **optional leading return type**, a name, a
  parenthesised parameter list, and either a block body or `= expression`
  (expression body). A function with no return type is the idiomatic
  top-level `main()`. Evidence: `Parser.java`
  `parseFunctionDeclaration`; corpus cases `hello` (no return type, block body)
  and `functions` (both block and `=` bodies).
* **[VERIFIED LEGACY BEHAVIOR]** Functions may be **nested** inside other bodies. Evidence:
  `StatementParser.java` `parseNestedFunction`.
* **[OPEN]** Whether nested functions capture enclosing locals, and how
  (record S11).
* **[VERIFIED LEGACY BEHAVIOR]** Parameters carry a type: `name: Type`. Evidence: the lambda
  parameter form `(x: Int) -> …` in corpus case `functions`, and
  `LambdaParser.parseLambdaBlockParams`.
* **[OPEN]** Whether parameter types may be omitted in a function
  declaration, and what the return type defaults to (record S1, which owns the
  inference boundary).

### 4.4 Type declarations

* **[VERIFIED LEGACY BEHAVIOR]** `class`, `interface`, `record`, `enum` and `entity` are the
  type-declaration keywords; `extends` and `implements` attach supertypes;
  `sealed` is a contextual modifier. Evidence: `Parser.java` `parse()`,
  `TypeDeclarations.java`.
* **[VERIFIED LEGACY BEHAVIOR]** A `record` may be **positional**: `record Name(Type a, Type b)`
  with no braces, generating accessors callable as `value.a()`. Evidence:
  corpus case `records`.
* **[VERIFIED LEGACY BEHAVIOR]** Record/type members are declared in a braced body; fields are
  written `Type name`; constructors are written `public constructor(params)`
  with `super(...)` for the parent. Evidence: corpus cases `classes`,
  `interfaces`, `abstract-records`.
* **[VERIFIED LEGACY BEHAVIOR]** `enum Name { A, B, C }` declares members accessed as
  `Name.A`; enums have methods including `name()` and `values()`. Evidence:
  corpus cases `enum`, `enum-methods`, `enum-switch-expr`.
* **[VERIFIED LEGACY BEHAVIOR]** Annotations (`@`) precede declarations. Evidence:
  `AnnotationParser.java`, `Parser.java` `parse()`.
* **[OPEN]** The annotation vocabulary and its semantics (record S10).

### 4.5 Modifiers

* **[VERIFIED LEGACY BEHAVIOR]** `public`, `private`, `protected`, `static`, `final`,
  `abstract`, `transient`, `volatile`, `synchronized`, `native`, `default`,
  `override` are keywords. Which combinations are legal is **not** established
  by the corpus. **[OPEN]** — record S10.

---

## 5. Statements

* **[VERIFIED LEGACY BEHAVIOR]** Verified statement forms (E1, `StatementParser.java`):
  block `{ … }`; `return [expr]`; `if`/`else`; `while`; `do`/`while`; `for`;
  `throw`; `try`/`catch`/`finally`; `switch` with `case` labels; `var`/`val`
  declarations; nested function declarations; expression statements.
* **[VERIFIED LEGACY BEHAVIOR]** `for` supports at least the `for (var x in collection)` form.
  Evidence: corpus cases `interfaces`, `list-higher-order`.
* **[VERIFIED LEGACY BEHAVIOR]** `catch` binds a typed variable: `catch (String e)`. Evidence:
  corpus case `exceptions`; `throw "boom"` throws a **string**, so throwables
  are not restricted to a class hierarchy.
* **[VERIFIED LEGACY BEHAVIOR]** `switch` supports both statement and expression forms with
  `case <pattern-or-value> -> <result>` arms, and arms may carry guards.
  Evidence: `StatementParser.parseSwitchCasePatternOrValue`,
  `parseGuardIfPresent`, `ExpressionParser.parseSwitchExpression`; corpus cases
  `enum-switch-expr`, `switch-expr`, `switch-break-in-loop`,
  `switch-defaults`, `sealed-switch`.
* **[OPEN]** Exhaustiveness requirements for `switch` over sealed types
  (record S5).
* **[OPEN]** Whether `break`/`continue` may carry labels (record S5).

---

## 6. Expressions

### 6.1 Verified precedence table (E1)

From `ExpressionParser.java` `precedence()` — this is a directly verified
table, not an inference:

| Level | Operators | Associativity |
|---|---|---|
| 1 (lowest) | `\|\|` | left |
| 2 | `&&` | left |
| 3 | `\|` `^` | left |
| 4 | `&` | left |
| 5 | `==` `!=` `<` `<=` `>` `>=` `instanceof` `as` | left |
| 6 | `<<` `>>` `>>>` | left |
| 7 | `+` `-` | left |
| 8 (highest binary) | `*` `/` `%` | left |
| — | assignment (`=` and compound forms) | parsed **below** all binary levels |

* **[VERIFIED LEGACY BEHAVIOR]** `as` and `instanceof` take a **type** on the right-hand side,
  not an expression; the type reference stops at the end of the type. Evidence:
  `parseBinary` special-cases both operators and calls `TypeParser.parseTypeRef`.
  **[OPEN]** whether assignment is right-associative (record S11).
* **[VERIFIED LEGACY BEHAVIOR]** Unary prefixes: `!`, `-`, `++`, `--`, and the expression forms
  of `spawn` and `await`. `spawn <expr>` and `await <expr>` are lowered as
  synthetic calls, i.e. they are prefix operators, not statements. Evidence:
  `ExpressionParser.parseUnary`.
* **[VERIFIED LEGACY BEHAVIOR]** Postfix forms (calls, member access, indexing, `++`/`--`) are
  parsed by `parsePostfix`.
* **[VERIFIED LEGACY BEHAVIOR] The grammar is line-sensitive in one place:** a trailing lambda
  block `{ … }` is only attached when its `{` starts on the **same line** as
  the token that closes the preceding call. Evidence:
  `ExpressionParser.trailingLambdaSameLine` (documented as fixing silent
  mis-attachment of a following block statement). **[PROPOSED NORMATIVE RULE]** the new
  frontend must reproduce this rule or explicitly decide otherwise; a
  line-insensitive grammar would silently change the meaning of existing code.
* **[VERIFIED LEGACY BEHAVIOR]** Lambdas may be written with a typed parameter list and `->`, as
  in `(x: Int) -> x * 2`, or as a trailing block with parameters
  (`{ a, b -> … }`) or without. Evidence: corpus case `functions`,
  `LambdaParser.java`, `parseTrailingLambdaBlock`.
* **[VERIFIED LEGACY BEHAVIOR]** Member access includes **property-style accessors without
  parentheses** for collections, e.g. `Color.values().size` and
  `list.size`. Evidence: corpus cases `enum`, `collections`. **[OPEN]**
  which members are properties and which are methods (record S9).
* **[VERIFIED LEGACY BEHAVIOR]** Generic call type arguments are supported:
  `parseCallTypeArguments`. **[OPEN]** their inference rules (record S1).
* **[OPEN]** The entity/query DSL parsed by `parseQueryDsl` is outside
  the corpus and is **not** specified here (record S10).

---

## 7. Scoping and name resolution

* **[VERIFIED LEGACY BEHAVIOR]** Resolution is scope-based with a symbol table; the legacy
  analyzer's phase order is `preDeclareType → defineMembers →
  analyzeDeclaration (fixpoint ≤ 4) → resolveMethodCalls`, where the last step
  is a documented no-op vestige and real overload resolution lives elsewhere.
  Evidence: `docs/audits/phase-0-baseline.md` §3.5 (itself E1 from source).
* **[E2]** A two-pass shape (declare types, then resolve members) is required
  because declarations may refer to each other in either order.
* **[OPEN]** Shadowing rules, the exact lookup order for a bare
  identifier, and whether a type may be declared after use (record S11).

---

## 8. Evaluation order

* **[OPEN]** The language specification does not yet state the
  evaluation order of: function arguments, the receiver and arguments of a
  method call, operands of a binary operator, and the initialiser expressions
  of a `var` list. **Decision record S11 owns this.** The legacy compiler
  lowers to a stack machine whose order is implicit in the lowerer, so the
  order is *observable* but was never written down; re-deriving it requires
  targeted differential tests, which is Phase 2 work.
* **[VERIFIED LEGACY BEHAVIOR]** `&&` and `||` short-circuit (they are distinct tokens with their
  own precedence levels and dedicated IR operations). **[OPEN]** whether
  this holds for all operand types (record S11).

---

## 9. Error semantics

* **[VERIFIED LEGACY BEHAVIOR]** Diagnostics carry a stable code, a severity, a source position
  and a human message. The observed legacy families are `LEX###`, `PARSE###`,
  `SEM###`, `PKG###`, `COMP###`, `WASM###`, `FFI###`, `MEM###`, `L###`,
  `CONC###`. Evidence: `docs/spec/compatibility-matrix.md` §4 and the lexer
  above.
* **[PROPOSED NORMATIVE RULE]** Every new diagnostic carries a stable code, a validated byte
  span, a severity and an optional suggestion, and rendering never panics on
  malformed input — already implemented in `cosmic-diagnostics` and verified in
  `docs/audits/phase-0-source-review.md` §4.4.
* **[PROPOSED NORMATIVE RULE]** No construct may fail silently. The legacy lexer's silent
  unknown-escape reinterpretation (§2.4) and its historical silent folding of
  `"""…"""` to an empty string (fixed in the legacy tree as `#364`, which is why
  `LEX008` exists) are both counter-examples the new toolchain must not repeat.
* **[OPEN]** The severity policy (which conditions are errors versus
  warnings) and the error-recovery contract (record S15).

---

## 10. Concurrency

* **[VERIFIED LEGACY BEHAVIOR]** `spawn` produces a handle for an expression; `await` consumes
  one. Both are usable in expression position. Evidence:
  `ExpressionParser.parseUnary`; corpus case `concurrency-spawn-await`.
* **[VERIFIED LEGACY BEHAVIOR]** The design report (page 7) records that the JS target announces
  **sequential** `spawn`/`await` while other targets use genuinely concurrent
  mechanisms, and the legacy ledger records a measured cross-target divergence
  in shift semantics (`cosmcc:docs/status.md` §698, cited in the Phase 0 conformance
  audit §3). **Cross-target observable divergence is therefore not
  hypothetical.**
* **[OPEN]** The normative concurrency contract — `spawn`/`await` typing,
  failure propagation, cancellation, and per-target scheduling guarantees — is
  owned by **decision record S6**. Until it is decided, no concurrency
  guarantee is claimed by this specification.

---

## 11. Normative compatibility boundaries

* **[PROPOSED NORMATIVE RULE]** The compatibility vocabulary is C0–C3 as defined in
  `docs/spec/compatibility-policy.md` §1. Only **C1 (syntactic acceptance) and
  C2 (observable behaviour)** are ever promised, and only for constructs the
  evidence covers.
* **[PROPOSED NORMATIVE RULE]** **C3 (bit-identical legacy artifacts) is not promised for any
  format.** The design report's own backend plan contemplates not rebuilding the
  JVM backend, so byte-identical output is not a coherent goal.
* **[PROPOSED NORMATIVE RULE]** A construct absent from the evidence is **not** claimed
  supported and **not** claimed unsupported. The compatibility matrix's rule
  stands: absence means "not yet analysed".
* **[PROPOSED NORMATIVE RULE]** Legacy **defects** are not part of the contract. Each known
  open legacy item (nested-generic mis-resolution, JS loop/return miscompiles,
  the shift-semantics divergence, the `Box<Box<Int>>` resolution bug) is an
  input to a decision record, not a behaviour to reproduce.
* **[PROPOSED NORMATIVE RULE]** No artefact is copied. Fixtures and documentation are
  re-derived from observed behaviour with the case name and SHA recorded as
  provenance (`docs/audits/phase-0-licensing-matrix.md` §4).

---

## 12. Unresolved register

Every **[OPEN]** above is owned by exactly one decision record:

| Record | Question |
|---|---|
| S1 | Generics: nested type-argument resolution, generic inference (including the parameter/return-type inference boundary), bounds and variance, overload resolution |
| S2 | Nullability: `T` / `T?` conversions, nullable access, nullability of type parameters |
| S3 | PortuKof: which surface tokens differ, and whether mixed surfaces exist |
| S4 | KofScript: the exact wrapping transformation and its execution semantics |
| S5 | Pattern matching: breadth, guards, exhaustiveness, labelled break/continue |
| S6 | Concurrency contract |
| S7 | Numeric semantics: overflow, promotion, division, float formatting, bitwise complement |
| S8 | String semantics: escapes, unknown escapes, triple-quote policy, interpolation |
| S9 | Standard-library boundary; properties versus methods; `package`/`import` rules |
| S10 | Reserved keywords without evidenced semantics; annotations; FFI; entity/query DSL |
| S11 | Evaluation order; assignment associativity; scoping and shadowing; nested-function capture |
| S12 | Equality: reference versus structural per type kind; `NaN`/`-0.0`; records, enums and strings; map-key and set contracts |
| S13 | Unicode: identifier classes, normalisation, column counting divergence |
| S14 | Trivia preservation and normalisation policy |
| S15 | Feature gates, severities and the error-recovery contract |
| S16 | Memory strategy |
| S17 | Runtime ABI versioning |
| S18 | Typed-IR invariants verified on entry to MIR |

The register, its statuses and its owner dependencies are tracked in
`docs/spec/decisions/README.md`.
