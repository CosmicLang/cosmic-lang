# Grammar Production Evidence Map (117/117)

| | |
|---|---|
| **Status** | **Machine-checked.** Verified by `tools/spec-consistency.py` (check G13). |
| **Directive** | ENGINEERING DIRECTIVE 002-R2 (post-merge) §3 |
| **Subject** | every production of `docs/spec/grammar.ebnf` — 117 of 117 mapped |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |
| **Related** | `docs/spec/grammar-consistency-report.md`, `docs/audits/phase-1-legacy-probes.md`, `docs/spec/implementation-gates.md`, `docs/spec/decisions/` |

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy
> repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this
> repository. Unprefixed paths are relative to this repository.

---

## 1. How to read this map

The grammar states evidence **per production block**; this document makes that
traceability **per production**. Every one of the 117 productions has exactly
one row. The checker fails if a production is missing, unknown, out of order,
or carries an empty or malformed cell — so the table cannot silently drift from
the grammar.

**Evidence classes** (closed set; combinations joined with `+`):

| Class | Meaning |
|---|---|
| `E1-source` | Read from the legacy source at the frozen SHA (path in the cell). |
| `E1-case` | Witnessed by an executed golden case (`tests/golden/<case>/`, 222/222 executed checks for jvm/js/script). |
| `E1-probe` | Reproduced by a read-only executed probe recorded in `docs/audits/phase-1-legacy-probes.md` (section cited in the cell). |
| `E2` | Inferred from E1 material; the basis is stated in the cell. |

**Recognition status** (closed set). This column is about **recognition only**;
open semantics do **not** make a recognisable production unrecognisable
(`docs/spec/implementation-gates.md` §3.1–3.2):

| Status | Meaning |
|---|---|
| `RECOGNISED` | The parser may recognise the production and preserve it in the CST. Any open semantics are gated at the semantic operation named in the residual column. |
| `UNSUPPORTED` | The parser must emit a **PARSE refusal** for this construct until its recognition is accepted — it is one of the unsupported form-families of `docs/spec/grammar-consistency-report.md` §4.2. |
| `REJECTED` | Refused **by design** (reserved-invalid tokens); never accepted as identifiers. |

**Decision record** — the record that owns the production's open question, or
`—`. **Residual ambiguity / unsupported condition** — what remains open or
unproven, including any gated operation. `—` means: nothing beyond what the
evidence class states.

**No evidence is invented here.** Source citations, case names and probe
section numbers are the ones already present in the grammar, the consistency
report and the probe record. Where a production has no specific citation in
those documents, the cell says exactly that rather than guessing.

---

## 2. The map

| # | Production | Evidence (source or executed case) | Class | Decision record | Recognition status | Residual ambiguity / unsupported condition |
|---|---|---|---|---|---|---|
| 1 | `source` | `cosmcc:parser/Lexer.java` tokenize | E1-source | — | RECOGNISED | The lexical root; no surface form of its own. |
| 2 | `bom` | `cosmcc:parser/Lexer.java` tokenize (BOM skip) | E1-source | — | RECOGNISED | Mid-file BOM behaviour not separately probed. |
| 3 | `line-terminator` | `cosmcc:parser/Lexer.java` tokenize | E1-source | — | RECOGNISED | `\r\n` consumed as one line break. |
| 4 | `whitespace` | `cosmcc:parser/Lexer.java` tokenize | E1-source | — | RECOGNISED | NBSP and ideographic space are not whitespace. |
| 5 | `comment` | `cosmcc:parser/Lexer.java` skipLineComment, skipBlockComment | E1-source | S14 | RECOGNISED | Normalisation policy open (S14). |
| 6 | `line-comment` | `cosmcc:parser/Lexer.java` skipLineComment | E1-source | S14 | RECOGNISED | Preservation policy open (S14). |
| 7 | `block-comment` | `cosmcc:parser/Lexer.java` skipBlockComment | E1-source | S14 | RECOGNISED | Non-nesting recorded as legacy behaviour; nesting policy open (S14). |
| 8 | `lexical-element` | Structural composition of layout and tokens (grammar §2.1) | E2 | — | RECOGNISED | Introduced so layout is reachable from `source` (report §6, B1). |
| 9 | `token` | Structural composition of the evidenced token classes (grammar §2.1) | E2 | — | RECOGNISED | — |
| 10 | `identifier` | `cosmcc:parser/Lexer.java` readIdentifier; `cosmcc:lang/KofKeywords.java` (keyword exclusion) | E1-source | S13 | RECOGNISED | Exact class set and normalisation open (S13). |
| 11 | `identifier-name` | `cosmcc:parser/Lexer.java` readIdentifier | E1-source | — | RECOGNISED | — |
| 12 | `identifier-start` | `cosmcc:parser/Lexer.java` tokenize; probe §5.2 (`_`, `$`) | E1-source+E1-probe | S13 | RECOGNISED | Digits are not admissible at the start. |
| 13 | `identifier-part` | `cosmcc:parser/Lexer.java` readIdentifier; probe §5.2 | E1-source+E1-probe | S13 | RECOGNISED | Combining marks open (S13). |
| 14 | `unicode-letter` | `cosmcc:parser/Lexer.java` (letter category test); probe §5.1 | E1-source+E1-probe | S13 | RECOGNISED | Exact category set open (S13). |
| 15 | `unicode-digit` | `cosmcc:parser/Lexer.java` (letter-or-digit test) | E1-source | S13 | RECOGNISED | Digit classes beyond the letter probes NOT VERIFIED (S13). |
| 16 | `any-char` | Formal placeholder (grammar §2.2) | E2 | — | RECOGNISED | A placeholder, not an enumerated class. |
| 17 | `keyword` | `cosmcc:lang/KofKeywords.java` | E1-source | — | RECOGNISED | — |
| 18 | `reserved-invalid` | `cosmcc:lang/KofKeywords.java` (SG-001); `cosmcc:parser/Parser.java` parse rejects | E1-source | — | REJECTED | Must never be accepted as identifiers. |
| 19 | `literal` | Alternation of the evidenced literal classes (grammar §2.4) | E2 | — | RECOGNISED | — |
| 20 | `int-literal` | `cosmcc:parser/Lexer.java` readNumber | E1-source | — | RECOGNISED | Binary, octal and digit separators absent (negative evidence). |
| 21 | `long-literal` | `cosmcc:parser/Lexer.java` readNumber | E1-source | — | RECOGNISED | — |
| 22 | `decimal-int` | `cosmcc:parser/Lexer.java` readNumber | E1-source | — | RECOGNISED | — |
| 23 | `hex-int` | `cosmcc:parser/Lexer.java` readNumber, isHexDigit | E1-source | — | RECOGNISED | — |
| 24 | `digit` | `cosmcc:parser/Lexer.java` readNumber | E1-source | — | RECOGNISED | — |
| 25 | `hex-digit` | `cosmcc:parser/Lexer.java` isHexDigit | E1-source | — | RECOGNISED | — |
| 26 | `float-literal` | `cosmcc:parser/Lexer.java` readNumber | E1-source | — | RECOGNISED | — |
| 27 | `double-literal` | `cosmcc:parser/Lexer.java` readNumber | E1-source | — | RECOGNISED | Optional suffix form. |
| 28 | `number-body` | `cosmcc:parser/Lexer.java` readNumber | E1-source | S7 | RECOGNISED | Fraction needs a digit after `.`; promotion and overflow semantics open (S7). |
| 29 | `string-literal` | `cosmcc:parser/Lexer.java` readString; probe §4.2 | E1-source+E1-probe | S8 | RECOGNISED | Triple-quote rejected by the legacy lexer (LEX008); policy open (S8). |
| 30 | `string-char` | `cosmcc:parser/Lexer.java` readString; probe §4.3 | E1-source+E1-probe | S8 | RECOGNISED | Raw newline accepted in legacy (verified); its normativity open (S8). |
| 31 | `char-literal` | `cosmcc:parser/Lexer.java` readChar; case `char-values` | E1-source+E1-case | — | RECOGNISED | — |
| 32 | `char-value` | `cosmcc:parser/Lexer.java` readChar | E1-source | — | RECOGNISED | Excludes line terminators. |
| 33 | `escape` | `cosmcc:parser/Lexer.java` readEscape; probe §4.1 | E1-source+E1-probe | S8 | RECOGNISED | Unknown escape silently yields the character (verified legacy defect); strict rejection proposed, pending S8. |
| 34 | `unicode-escape` | `cosmcc:parser/Lexer.java` readUnicodeEscape | E1-source | — | RECOGNISED | Exact-width `uXXXX` form; multi-`u` forms not evidenced. |
| 35 | `boolean-literal` | `cosmcc:lang/KofKeywords.java` | E1-source | — | RECOGNISED | — |
| 36 | `null-literal` | `cosmcc:lang/KofKeywords.java` | E1-source | — | RECOGNISED | — |
| 37 | `operator` | `cosmcc:parser/Lexer.java` readOperatorOrDelimiter; probe §6.3 | E1-source+E1-probe | — | RECOGNISED | Longest-match ordering; bitwise complement absent (S7). |
| 38 | `delimiter` | `cosmcc:parser/Lexer.java` readOperatorOrDelimiter | E1-source | — | RECOGNISED | Reference-only vocabulary table. |
| 39 | `separator` | `cosmcc:parser/Lexer.java` readOperatorOrDelimiter | E1-source | — | RECOGNISED | Reference-only vocabulary table. |
| 40 | `compilation-unit` | `cosmcc:parser/Parser.java` parse; case `hello` | E1-source+E1-case | S9 | RECOGNISED | `package`/`import` optionality and placement open (S9). |
| 41 | `package-decl` | `cosmcc:parser/Parser.java` parsePackage | E1-source | S9 | RECOGNISED | Optionality and placement unprobed (S9). |
| 42 | `import-decl` | `cosmcc:parser/Parser.java` parseImports | E1-source | S9 | RECOGNISED | Wildcards, aliases and static imports not evidenced (S9). |
| 43 | `qualified-name` | `cosmcc:parser/Parser.java` parsePackage, parseImports | E1-source | — | RECOGNISED | — |
| 44 | `top-level-decl` | `cosmcc:parser/Parser.java` parse; case `hello` | E1-source+E1-case | — | RECOGNISED | Dispatch order is significant and evidence-backed. |
| 45 | `top-level-body` | `cosmcc:parser/Parser.java` parse | E1-source | — | RECOGNISED | Contextual keywords require word plus lookahead. |
| 46 | `test-decl` | `cosmcc:parser/Parser.java` parseTestDeclaration | E1-source | S10 | UNSUPPORTED | Outside the corpus; PARSE refusal until recognition accepted (report §4.2). |
| 47 | `application-decl` | `cosmcc:parser/Parser.java` parseApplicationDeclaration | E1-source | S10 | UNSUPPORTED | Outside the corpus; PARSE refusal until recognition accepted (report §4.2). |
| 48 | `infra-decl` | `cosmcc:parser/Parser.java` parseInfraDeclaration | E1-source | S10 | UNSUPPORTED | Outside the corpus; PARSE refusal until recognition accepted (report §4.2). |
| 49 | `extern-decl` | `cosmcc:parser/Parser.java` parseExternDeclaration | E1-source | S10 | UNSUPPORTED | FFI is security-sensitive; refuse until specified (report §4.2). |
| 50 | `function-signature` | `cosmcc:parser/Parser.java` parseExternDeclaration | E1-source | S10 | UNSUPPORTED | Reachable only through `extern-decl`; refused with that family. |
| 51 | `foreign-module-decl` | `cosmcc:parser/Parser.java` parseForeignModule | E1-source | S10 | UNSUPPORTED | Desugars to extern declarations; refuse until specified (report §4.2). |
| 52 | `function-decl` | `cosmcc:parser/Parser.java` parseFunctionDeclaration; cases `hello`, `functions` | E1-source+E1-case | S1 | RECOGNISED | Inference boundary open (S1). Finding F4 affects parameter-list derivations. |
| 53 | `return-type` | `cosmcc:parser/Parser.java` parseFunctionDeclaration; case `hello` (absence) | E1-source+E1-case | S1 | RECOGNISED | Default when absent not evidenced (S1). |
| 54 | `parameter-list` | `cosmcc:parser/Parser.java` parseFunctionDeclaration; case `functions` | E1-source+E1-case | — | RECOGNISED | Finding F4: corpus parameter lists use `Type name [= default]`. |
| 55 | `parameter` | `cosmcc:parser/TypeParser.java` parseFormalParameter (both orders, default values); cases `functions`, `string-switch-defaults` | E1-source+E1-case | S1 | RECOGNISED | Finding F4: the merged production encodes only the `name : Type` order and omits `= default`; the executed corpus uses `Type name [= default]`. Type omission not evidenced (S1). |
| 56 | `function-body` | `cosmcc:parser/Parser.java` parseFunctionDeclaration; cases `hello` (block), `functions` (`=`) | E1-source+E1-case | — | RECOGNISED | — |
| 57 | `nested-function-decl` | `cosmcc:parser/StatementParser.java` parseNestedFunction | E1-source | S11 | RECOGNISED | Capture rejected with SEM011 (verified); intended closure design open (S11). |
| 58 | `type-ref` | `cosmcc:parser/TypeParser.java`; cases `nullable-suffix-leading-return-type`, `nested-generic-leading-return-type`, `nullable-soft-cast` | E1-source+E1-case | S1, S2 | RECOGNISED | Nested type arguments are a verified defect (probe §6.1–6.2) and must be gated, not guessed (S1); conversions beyond `T → T?` open (S2). |
| 59 | `nominal-type` | `cosmcc:parser/TypeParser.java` | E1-source | — | RECOGNISED | — |
| 60 | `type-arguments` | `cosmcc:parser/TypeParser.java`; case `nested-generic-leading-return-type`; probe §6.1–6.2 | E1-source+E1-case+E1-probe | S1 | RECOGNISED | Depth greater than one must be gated, not guessed (S1). |
| 61 | `array-suffix` | `cosmcc:parser/TypeParser.java` | E1-source | S9 | RECOGNISED | Corpus exercise of array types not established (S9). |
| 62 | `primitive-type` | `cosmcc:lang/KofKeywords.java` | E1-source | — | RECOGNISED | — |
| 63 | `type-decl` | `cosmcc:parser/TypeDeclarations.java`; cases `classes`, `interfaces`, `enum`, `records` | E1-source+E1-case | — | RECOGNISED | `entity` is handled by its own row. |
| 64 | `class-decl` | `cosmcc:parser/TypeDeclarations.java`; case `classes` | E1-source+E1-case | — | RECOGNISED | — |
| 65 | `interface-decl` | `cosmcc:parser/TypeDeclarations.java`; case `interfaces` | E1-source+E1-case | — | RECOGNISED | — |
| 66 | `entity-decl` | `cosmcc:lang/KofKeywords.java` (reserved keyword); declaration shape by class analogy | E1-source+E2 | S10 | UNSUPPORTED | Keyword reserved; semantics unknown; refuse until S10 is accepted. |
| 67 | `supertypes` | `cosmcc:parser/TypeDeclarations.java`; case `classes` (`extends`, `implements`) | E1-source+E1-case | — | RECOGNISED | Multiple `extends` not evidenced. |
| 68 | `record-decl` | `cosmcc:parser/TypeDeclarations.java`; case `records` | E1-source+E1-case | — | RECOGNISED | Positional form has no braces. |
| 69 | `positional-record` | case `records` | E1-case | — | RECOGNISED | Accessors are called as `value.x()`. |
| 70 | `record-body` | `cosmcc:parser/TypeDeclarations.java` | E1-source | — | RECOGNISED | Brace form source-evidenced; corpus exercise not separately established. |
| 71 | `record-component` | case `records` | E1-case | — | RECOGNISED | — |
| 72 | `enum-decl` | `cosmcc:parser/TypeDeclarations.java`; cases `enum`, `enum-methods`, `enum-switch-expr` | E1-source+E1-case | — | RECOGNISED | — |
| 73 | `enum-constant` | cases `enum`, `enum-methods` | E1-case | — | RECOGNISED | Constructor arguments evidenced. |
| 74 | `class-body` | `cosmcc:parser/ClassMemberParser.java` | E1-source | — | RECOGNISED | — |
| 75 | `member` | `cosmcc:parser/ClassMemberParser.java`; case `classes` | E1-source+E1-case | — | RECOGNISED | Annotation and modifier prefixes are rows 78 and 79. |
| 76 | `constructor-decl` | `cosmcc:parser/ClassMemberParser.java`; case `classes` | E1-source+E1-case | — | RECOGNISED | `super(...)` as first statement. Finding F4 affects class method and constructor parameter lists. |
| 77 | `field-decl` | `cosmcc:parser/ClassMemberParser.java`; case `classes` | E1-source+E1-case | — | RECOGNISED | Fields mutable. |
| 78 | `modifier` | `cosmcc:lang/KofKeywords.java`; `cosmcc:parser/ClassMemberParser.java` | E1-source | S10 | RECOGNISED | Accepted as tokens; legal combinations unknown (S10). |
| 79 | `annotation` | `cosmcc:parser/AnnotationParser.java` | E1-source | S10 | UNSUPPORTED | Vocabulary and semantics unknown; an unknown annotation is an error. |
| 80 | `annotation-args` | `cosmcc:parser/AnnotationParser.java` | E1-source | S10 | UNSUPPORTED | Unreachable while `annotation` is refused; same family. |
| 81 | `statement` | `cosmcc:parser/StatementParser.java` parseStatement; case `control-flow` | E1-source+E1-case | — | RECOGNISED | Finding F1: `break`/`continue` are evidenced but have no production; Finding F2 affects statement-switch arms. |
| 82 | `block` | `cosmcc:parser/StatementParser.java` parseBlock; case `hello` | E1-source+E1-case | — | RECOGNISED | — |
| 83 | `return-stmt` | `cosmcc:parser/StatementParser.java` | E1-source | — | RECOGNISED | — |
| 84 | `throw-stmt` | `cosmcc:parser/StatementParser.java`; case `exceptions` | E1-source+E1-case | — | RECOGNISED | String throwables. |
| 85 | `if-stmt` | `cosmcc:parser/StatementParser.java` | E1-source | — | RECOGNISED | — |
| 86 | `while-stmt` | `cosmcc:parser/StatementParser.java` | E1-source | — | RECOGNISED | — |
| 87 | `do-while-stmt` | `cosmcc:parser/StatementParser.java`; case `loops-control` | E1-source+E1-case | — | RECOGNISED | — |
| 88 | `for-stmt` | `cosmcc:parser/StatementParser.java`; cases `interfaces` (for-in), `loops-control` (three-clause) | E1-source+E1-case | — | RECOGNISED | Finding F3: the merged annotation claimed the three-clause form was corpus-unevidenced; corrected — `loops-control` exercises it. |
| 89 | `for-init` | `cosmcc:parser/StatementParser.java`; case `loops-control` | E1-source+E1-case | — | RECOGNISED | Optional type annotation. |
| 90 | `try-stmt` | `cosmcc:parser/StatementParser.java`; cases `exceptions`, `finally-control-flow`, `finally-return` | E1-source+E1-case | — | RECOGNISED | — |
| 91 | `catch-clause` | `cosmcc:parser/StatementParser.java`; case `exceptions` | E1-source+E1-case | — | RECOGNISED | Typed binding. |
| 92 | `finally-clause` | cases `finally-control-flow`, `finally-return` | E1-case | — | RECOGNISED | — |
| 93 | `switch-stmt` | `cosmcc:parser/StatementParser.java` parseSwitchStatement; cases `switch-expr`, `switch-break-in-loop` | E1-source+E1-case | S5 | RECOGNISED | Finding F2: legacy statement arms are `case X :` (colon); the merged grammar models `->`. Production split awaits owner authorization. |
| 94 | `switch-arm` | `cosmcc:parser/StatementParser.java` parseSwitchCasePatternOrValue; cases `switch-expr`, `enum-switch-expr`, `sealed-switch`, `switch-break-in-loop` | E1-source+E1-case | S5 | RECOGNISED | Finding F2; exhaustiveness, fallthrough and labelled forms open (S5). |
| 95 | `switch-label` | `cosmcc:parser/StatementParser.java` parseSwitchCasePatternOrValue; cases `switch-expr`, `enum-switch-expr` | E1-source+E1-case | S5 | RECOGNISED | Pattern versus value versus `default` disambiguation is semantic (S5). |
| 96 | `guard` | `cosmcc:parser/StatementParser.java` parseGuardIfPresent | E1-source | S5 | RECOGNISED | Recognition only; no normative guard rule (S5). |
| 97 | `switch-body` | `cosmcc:parser/StatementParser.java` parseSwitchStatement | E1-source | S5 | RECOGNISED | Finding F2: the statement form carries a statement list, not a single body. |
| 98 | `var-decl-stmt` | `cosmcc:parser/StatementParser.java` parseVarDecl; case `concurrency-spawn-await` | E1-source+E1-case | — | RECOGNISED | Optional type annotation. |
| 99 | `expression-stmt` | `cosmcc:parser/StatementParser.java` | E1-source | — | RECOGNISED | — |
| 100 | `expression` | `cosmcc:parser/ExpressionParser.java`; probes §2.5, §6.3 | E1-source+E1-probe | S11 | RECOGNISED | Sampled JVM order is left-to-right; unsampled forms and targets NOT VERIFIED (S11, gates M-6). |
| 101 | `assignment-expr` | `cosmcc:parser/ExpressionParser.java` parseAssignment; probe §2.5 | E1-source+E1-probe | S11 | RECOGNISED | Assignment is a statement (SEM027); `a = b = 1` rejected; associativity question dissolved (S11). |
| 102 | `assignment-op` | `cosmcc:parser/ExpressionParser.java`; case `compound-assign` | E1-source+E1-case | — | RECOGNISED | Compound operators exercised; simple `=` exercised in `loops-control`. |
| 103 | `binary-expr` | `cosmcc:parser/ExpressionParser.java` parseBinary; probe §6.3 | E1-source+E1-probe | S11 | RECOGNISED | Left-associative, verified for `<` and `>`; general order NOT VERIFIED. |
| 104 | `binary-op` | `cosmcc:parser/ExpressionParser.java` precedence table | E1-source | S7, S12 | RECOGNISED | Numeric operations open (S7); equality measured for several kinds, enum and map-key contracts NOT VERIFIED (S12). |
| 105 | `binary-rhs` | `cosmcc:parser/ExpressionParser.java` parseBinary | E1-source | — | RECOGNISED | `type-ref` right-hand side only for `as` and `instanceof`. |
| 106 | `unary-expr` | `cosmcc:parser/ExpressionParser.java` parseUnary; case `concurrency-spawn-await` | E1-source+E1-case | S6 | RECOGNISED | `spawn`/`await` typing gated (S6). |
| 107 | `postfix-expr` | `cosmcc:parser/ExpressionParser.java` parsePostfix, parseCallTypeArguments | E1-source | S1, S9 | RECOGNISED | Generic-call position evidenced; ambiguous overloads gated (S1); indexing details open (S9). |
| 108 | `postfix-op` | `cosmcc:parser/ExpressionParser.java` parsePostfix | E1-source | S1 | RECOGNISED | Ambiguous overload selection gated (S1); overload tie-break NOT VERIFIED. |
| 109 | `argument-list` | `cosmcc:parser/ExpressionParser.java` parsePostfix | E1-source | S11 | RECOGNISED | Sampled argument order left-to-right; cross-target NOT VERIFIED (S11). |
| 110 | `primary` | `cosmcc:parser/ExpressionParser.java` parsePrimary | E1-source | — | RECOGNISED | Includes both a parenthesised expression and the lambda position; lookahead ambiguity unresolved (report §5.3, Phase 2 parser-design requirement). |
| 111 | `if-expr` | `cosmcc:parser/ExpressionParser.java` parseIfExprBranch | E1-source | S5 | RECOGNISED | Parser-evidenced; no corpus exercise; surface provisional (S5). |
| 112 | `array-literal` | No evidence located (grammar §3.7) | E2 | S9 | UNSUPPORTED | Brace form is a placeholder; refuse until evidenced or accepted (report §4.2). |
| 113 | `lambda` | `cosmcc:parser/LambdaParser.java`, `ExpressionParser.parseTrailingLambdaBlock`; case `functions` | E1-source+E1-case | — | RECOGNISED | Parenthesis-versus-lambda lookahead unresolved (report §5.3); trailing-lambda same-line legacy rule is a proposed normative rule. |
| 114 | `lambda-params` | `cosmcc:parser/LambdaParser.java`; case `functions` | E1-source+E1-case | — | RECOGNISED | Typed parameters. |
| 115 | `lambda-body` | `cosmcc:parser/LambdaParser.java`; case `functions` | E1-source+E1-case | — | RECOGNISED | Expression or block. |
| 116 | `switch-expr` | `cosmcc:parser/ExpressionParser.java` parseSwitchExpression; cases `enum-switch-expr`, `switch-expr`, `sealed-switch` | E1-source+E1-case | S5 | RECOGNISED | Arrow arms; exhaustiveness not evidenced (S5). |
| 117 | `new-expr` | `cosmcc:parser/ExpressionParser.java` parseNewExpression | E1-source | — | RECOGNISED | `new` keyword parser-evidenced; corpus exercise not established; no case is cited for it. |

---

## 3. Findings surfaced by the mapping

The mapping exercise checked every production against the executed corpus and the
frozen source. Four statements in the merged Phase 1 documents do not survive
that check. **F1, F2 and F4 change the derivable language and are therefore
reported, not applied** — changing them is a grammar correction that the owner
must authorize. **F3 plus the minor corrections are annotation-level and are
applied in this branch.**

### F1 — `break`/`continue` statements are evidenced but have no production

**Evidence (all read-only):**

* `cosmcc:TokenType.java` defines `BREAK` and `CONTINUE`; `cosmcc:lang/KofKeywords.java` maps the words; `cosmcc:parser/StatementParser.java` parses them as `BreakStmt`/`ContinueStmt` with **no label** (advance, then expect separator); `cosmcc:KofFormatter.java` emits both; `cosmcc:lang/PortuKofVocabulary.java` maps `sair`/`segue`.
* Executed cases: `loops-control` uses `break` and `continue` inside three-clause `for` loops; `switch-break-in-loop` uses `break` inside switch arms (and shows that `break` exits the **switch**, not the enclosing loop — both cases pass 222/222).

**Defect:** the merged `statement` production has no alternative that derives
`break` or `continue`. A parser built strictly from the grammar would refuse
valid, executed legacy programs.

**Proposed correction (owner authorization required; changes the production
count):** add `break-stmt = "break" ;` and `continue-stmt = "continue" ;` (no
labels — labels are NOT VERIFIED and stay refused), referenced from
`statement`. The evidence above is sufficient for recognition; semantics
(loop versus switch exit) are the executed `switch-break-in-loop` behaviour.

### F2 — statement-`switch` arm form is mis-modelled

**Evidence:** `cosmcc:parser/StatementParser.java` `parseSwitchStatement`
requires `case X :` (COLON) with a **statement list** per arm and `default :`
(codes PARSE073/PARSE074); the expression form uses `case X -> value` arms.
Executed cases: `switch-expr` and `switch-break-in-loop` use the colon form in
statement position (including `default :`); `enum-switch-expr`, `switch-expr`
and `sealed-switch` use arrow arms in expression position. No fallthrough was
observed (`switch-expr` output stops at the matched arm).

**Defect:** the merged grammar defines a single `switch-arm` with `->` armed
bodies and shares it between `switch-stmt` and `switch-expr`, so colon-form
statement switches are not derivable.

**Proposed correction (owner authorization required; changes the production
set):** split the arm forms — `switch-stmt` takes colon arms with embedded
statement lists; `switch-expr` keeps arrow arms. Exhaustiveness, fallthrough
and labels remain open under S5.

### F3 — three-clause `for` was reported corpus-unevidenced; it is exercised

The merged annotation (and the register row and report row derived from it)
claimed the three-clause `for` form was "E1 from the parser but NOT exercised by
the corpus". Golden case `loops-control` (executed, 222/222) contains
`for (var i = 0; i < 10; i = i + 1)`. **Corrected in this branch** at the
annotation level: the grammar comment, the provisional register row and the
consistency-report row now cite the executed evidence. No production changed.

### F4 — function `parameter` models only the lambda order and omits defaults

**Evidence:** `cosmcc:parser/TypeParser.java` `parseFormalParameter` accepts
**both** orders — `name : Type` (the form idiomatic for lambdas, chosen when an
identifier is directly followed by `:`) and `Type name` — and, in both, an
optional `= default` value. The executed corpus uses the **`Type name`** order
everywhere a function declares parameters (`functions`: `String saudacao(String
nome)`, `Int soma(Int a, Int b)`), including constructors (`classes`:
`public constructor(String name)`), and exercises defaults five times
(`string-switch-defaults`: `String fallback = "unknown"`, `String prefix = "Hi"`,
`String suffix = "!"`, plus two call-site omissions).

**Defect:** the merged `parameter` production is
`parameter = identifier , ":" , type-ref ;` — the lambda order, with no default
value. No executed golden case with a parameterised declaration is derivable
under the merged grammar as written, and `function-decl`, `constructor-decl` and
every member method inherit the gap.

**Proposed correction (owner authorization required; changes a production body):**

```ebnf
parameter = ( identifier , ":" , type-ref | type-ref , identifier )
          , [ "=" , expression ] ;
```

The two orders are distinguishable by one-token lookahead, exactly as
`parseFormalParameter` does it (identifier followed by `:` chooses `name : Type`).
Default values are recognition-evidenced (source and execution); their semantic
evaluation rule is a separate matter.

### Minor corrections applied in this branch

* `docs/spec/language-spec.md` §5: the switch bullet described both switch forms
  as using `->` arms; corrected to colon arms (statement) and arrow arms
  (expression) per the evidence above.
* `docs/spec/language-spec.md` §5: case citation `switch-defaults` corrected to
  the existing case `string-switch-defaults`.
* `docs/spec/language-spec.md` §0.1: duplicate heading removed.

---

## 4. What this map does not claim

* **Not a correctness proof.** The map proves *traceability*: every production
  has a named evidence basis and a recognition disposition. It cannot prove the
  grammar describes the language — the differential harness does that (Phase 2).
* **Not an acceptance act.** `RECOGNISED` is a recognition disposition, not an
  owner decision. No rule is `ACCEPTED NORMATIVE RULE`; every open question
  remains owned by its decision record.
* **Not completeness of evidence.** Where the cell says "not established" or
  "NOT VERIFIED", the evidence is genuinely missing, and the consumer must gate
  rather than guess.
* **Not a substitute for the block markers.** The grammar's per-block citations
  remain; this map is the per-production projection the consistency report
  §3.2 previously identified as missing.
