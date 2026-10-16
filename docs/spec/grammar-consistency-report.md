# Grammar and Specification Consistency Report

| | |
|---|---|
| **Status** | **Machine-checked.** Produced by `tools/spec-consistency.py`. |
| **Directive** | ENGINEERING DIRECTIVE 002-R1 §5 · 002-R2 (post-merge) §3 · 002-R3 §6 · 002-R5 §3–§4 |
| **Subject** | `docs/spec/grammar.ebnf` (122 productions) and the specification document set |
| **Checked** | 2026-10-16 |
| **Result** | **Proposed and machine-checkable** — run the checker for the current result; see §3 for its latest captured output |

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

---

## 1. Why a checker exists

The directive asks for a **machine-checkable** consistency report "where
feasible". It is feasible, so the report is generated rather than asserted:

```sh
python3 tools/spec-consistency.py
```

`tools/spec-consistency.py` is **documentation tooling**. It is not compiler
code, it is not shipped, and it has no runtime role. It exists so that the claims
below can be re-derived by anyone.

**This matters because the checker found real defects.** Run against the HEAD
revision, it reports **37 findings**: 24 undefined nonterminals, 12 unreachable
productions, and one production body corrupted by mixed terminal quoting
(§3.1). A report that only asserted consistency would have missed all of them.

---

## 2. The checks

| # | Check | What it proves |
|---|---|---|
| **G0** | No unparseable grammar statement | Every statement in the fenced grammar blocks is a well-formed production |
| **G1** | Every referenced nonterminal is defined | The grammar has no dangling symbol |
| **G2** | Every production is reachable from a root | No orphaned production (`source`, `compilation-unit` are the roots) |
| **G3** | No production is defined twice | No accidental redefinition |
| **G4** | Every production block carries an evidence or status marker | Evidence is stated where the production is |
| **G5** | Every `[OPEN:Sn]` names an existing decision record | The grammar's open questions are traceable |
| **G6** | Every `S<n>` reference in `docs/` resolves | No reference to a decision record that does not exist |
| **G7** | The register lists exactly S1–S18 and 18 record files exist | The register is complete |
| **G8** | Every record's status uses a permitted status word | No invented statuses |
| **G9** | Every gate id referenced in `docs/` is defined in a gate document | Gates are defined before they are cited |
| **G10** | Every internal markdown link resolves | No broken navigation |
| **G11** | Every repository path referenced in backticks resolves | No reference to a file that does not exist (legacy paths must be prefixed `cosmcc:`) |
| **G12** | The six normative statuses are defined in the specification | The status vocabulary exists and is authoritative |
| **G13** | The production evidence map covers every production exactly once | Per-production traceability: one well-formed entry per production, numbered and sequenced in grammar definition order, with resolvable decision references (`docs/spec/grammar-evidence-map.md`) |

---

## 3. Raw output

```
G-grammar: 122 productions parsed from docs/spec/grammar.ebnf
G1 undefined nonterminals: none
G2 unreachable productions: none (3 reference-only tables excluded: delimiter, operator, separator)
G4 evidence: 10 production blocks, all carry a marker; inline per-production markers: 0/122 (per-production traceability: docs/spec/grammar-evidence-map.md, G13)
G5 open productions reference 5 records, all present: S1, S13, S2, S5, S9

G7 decision records: 18 of 18 (S1..S18)
G7 register rows: 18
G8 status vocabulary checked on 18 records

G6 decision-record references: 951 occurrences, all resolve
G9 gate ids: 15 defined across 2 documents, 15 referenced, all known
G10 internal links: 19 checked, 0 broken
G11 repository paths: 475 checked, 0 unresolved, 2 legacy-context

G12 normative vocabulary: 6 terms defined

G13 evidence map: 122/122 productions mapped, rows in grammar definition order, cells well-formed

OK — all specification consistency checks passed
```

### 3.1 The HEAD baseline — measured, not asserted

The directive — following the Phase 1 report — describes the delivered grammar
as **"59 productions"**. A mechanical parse of the same file at the HEAD
revision finds **97 production statements**. The "59" was an undercount that no
tool produced; the grammar defined **117 productions** after the R1 corrections
supplied the missing definitions, **119** after the 002-R3 recognition
corrections (F1, F2), and **122** after the 002-R5 pattern-header correction
(F5 — finding S-G1S-1, verified directly against the frozen legacy evidence).

The baseline is reproducible without editing the repository:

```sh
git show 18a9255:docs/spec/grammar.ebnf > /tmp/head.ebnf
python3 tools/spec-consistency.py --grammar /tmp/head.ebnf
```

Output on 2026-10-16: **97 productions · 37 findings** — one unparseable
statement, **24 undefined nonterminals**, **12 unreachable productions**.

| Check | Finding at HEAD `18a9255` | Classification |
|---|---|---|
| G0 | 1 unparseable statement: `binary-expr(min) = …` — a Java-style parameterised name used as a production name | Convention defect |
| G1 | **24 undefined nonterminals.** Substantive: `any-char`, `array-literal`, `binary-expr`, `boolean-literal`, `character`, `char-literal`, `digit`, `for-init`, `function-signature`, `hex-digit`, `if-expr`, `level`, `null-literal`, `record-body`, `unicode-digit`, `unicode-escape`, `unicode-letter`; artifact: `true`, `false`, `null`, `t`, `n`, `r`, `u` (see root cause below) | **Substantive defect** |
| G2 | **12 unreachable productions**: `binary-op`, `binary-rhs`, `block-comment`, `comment`, `keyword`, `line-comment`, `line-terminator`, `primitive-type`, `reserved-invalid`, `string-char`, `unary-expr`, `whitespace` | **Substantive defect** |

**Root cause of the single-letter symbols.** The delivered grammar **mixed `"`
and `'` as terminal delimiters**, and the backslash terminal was written

```ebnf
string-char    = any-char - ( '"' | "\" ) ;
```

where the ISO-EBNF spelling of a backslash terminal is `"\\"`. The malformed
terminal `"\"` is an *unbalanced quote* to the quote-aware statement splitter,
which therefore read across the following lines: it merged **seven productions
into one statement** (`string-char`, `char-literal`, `char-value`, `escape`,
`unicode-escape`, `boolean-literal`, `null-literal`) and left several terminals
unmasked, so fragments of them (`true`, `false`, `null`, `n`, `t`, `r`, `u`)
were read as nonterminals. **The corrected grammar uses one terminal convention**
(`"…"` with `\"` for the quote character and `"\\"` for the backslash), and the
corruption is gone.

The other G1/G2 findings are not cosmetic: a lexer cannot be derived from a
grammar in which `digit`, `hex-digit` or `unicode-letter` do not exist, and a
parser cannot reach `unary-expr`/`binary-op` productions whose parent production
is not connected to any root.

**Workflow findings during the correction** (observed while repairing; not
reproducible with the single command above, and listed for completeness):
G4 block-marker convention, G5 marker renames (`UNRESOLVED` → `OPEN`), G7
register sort order, G9 gate definitions, G10/G11 broken references — each is
now defined and checked by the final run in §3.

**This is why the report is machine-checked.** A hand-written report asserted
"59 productions, all evidence-marked". The tool found 97 statements, 37
problems, and a corrupted production body that a reader would skim past.

### 3.2 The G4 convention

The grammar states evidence **per group of productions**: a fenced block cites
the source it was derived from, and the productions in that block inherit it.
That is the right granularity here — `int-literal`, `long-literal`,
`decimal-int` and `hex-int` are all evidenced by the same method
(`parser/Lexer.java readNumber`), and repeating the citation four times would be
noise.

So the checkable requirement was made **block-scoped**: every block that defines
productions must carry at least one evidence or status marker. **10 of 10
blocks do.** The inline per-production statistic is still reported for
transparency: **0 of 122 productions carry their own inline marker** — every
production is covered by a block-level citation, none by a citation that names
only it.

**This gap is closed by the evidence map.** `docs/spec/grammar-evidence-map.md`
projects the block-level evidence onto every single production, and check
**G13** fails if any production is missing, unknown, out of order or
malformed. The earlier "deliberate weakening" recorded here is therefore
retired: block markers remain the in-grammar convention, and per-production
traceability is enforced where it belongs — in the map.

---

## 4. Provisional productions — full classification

Directive §5 requires five things per provisional production. This table is that
classification.

**The governing principle** (`docs/spec/implementation-gates.md` §3.1):
*recognition is separable from acceptance*. Evidence-backed syntax may be
recognised while its semantics are gated; syntax not yet evidenced must receive
a precise PARSE refusal until the owner approves a recognition contract.

| Production | Exact unresolved dependency | Recognisable independently? | Unsupported **syntax** to mark? | Evidence that would close it | Record |
|---|---|---|---|---|---|
| `package-decl` | Optionality; must it precede imports? | **Yes** | No | A probe with `import` before `package`, and a file with no `package` | S9 |
| `import-decl` | Wildcards, aliases, static imports | **Yes** | No | Probes for `import x.*` and `import x as y` | S9 |
| `test-decl` | Semantics of `test "name" { … }`; is it a language or tooling construct? | **Yes** — word + string lookahead is parser-evidenced | **Yes** — the construct is outside the corpus and must be refused | A legacy source read plus an owner scope decision | S10 |
| `application-decl` | Same | **Yes** | **Yes** | Same | S10 |
| `infra-decl` | Same | **Yes** | **Yes** | Same | S10 |
| `extern-decl` | FFI semantics; ABI; safety | **Yes** | **Yes** — security-sensitive; refuse until specified | An FFI specification (later phase) | S10 |
| `foreign-module-decl` | Desugaring to `extern`; sharing semantics | **Yes** | **Yes** | Same | S10 |
| `entity-decl` | Semantics of `entity`; ORM surface | **Yes** | **Yes** | An owner scope decision | S10 |
| `annotation` | Vocabulary and semantics | **Yes** — syntax is parser-evidenced | **Yes** — an unknown annotation is an error | The annotation vocabulary from legacy source | S10 |
| `modifier` combinations | Which combinations are legal | **Yes** | No — modifiers are accepted as tokens; illegal combinations are a semantic check | Legacy source + probes per combination | S10 |
| `type-arguments` with a nested argument | **The correct rule does not exist** — the legacy behaviour is a defect (`NoSuchMethodError`) | **Yes** | No — `Box<Int>` is fine; only depth ≥ 2 is gated | A designed rule; the legacy behaviour cannot supply one | S1 |
| `type-ref` `?` | Conversions beyond `T → T?` | **Yes** | No | Probes per conversion direction | S2 |
| `parameter` type optionality | Not evidenced | **Yes** | No | A probe omitting a parameter type | S1 |
| `return-type` default | Not evidenced | **Yes** | No | A probe omitting a return type | S1 |
| `parameter` order and `= default` | **Corrected (002-R3)** — both evidenced orders and the optional `= default` are modelled | **Yes** | No — recognition is evidenced; default-expression evaluation semantics remain open | — correction applied | S1 |
| `for-stmt` three-clause form | **Corrected (002-R2)** — already exercised by the executed case `loops-control` | **Yes** | No | — evidence found | S5 |
| `switch-stmt-arm` / `switch-expr-arm` exhaustiveness | Not evidenced | **Yes** | No — gate the *exhaustiveness check*, not `switch` | A probe with a non-exhaustive switch | S5 |
| `switch-stmt-arm` / `switch-expr-arm` fallthrough | Not evidenced | **Yes** | No | A probe with two adjacent arms | S5 |
| labelled `break`/`continue` | Not evidenced | **Yes** | **Yes** — labels are syntax that may not exist | A probe using `break label` | S5 |
| `break`/`continue` statements | **Corrected (002-R3)** — `break-stmt`/`continue-stmt` added; unlabelled forms only | **Yes** | No — the syntax is evidenced and now derived | — correction applied | S5 |
| `switch-stmt` colon arms | **Corrected (002-R3)** — `switch-stmt-arm` (colon) and `switch-expr-arm` (arrow) are separate productions | **Yes** | No — the colon form is evidenced by executed cases | — correction applied | S5 |
| `switch-label` pattern headers | **Corrected (002-R5)** — `binding-pattern`, `destructuring-pattern` and `pattern-field` added after direct verification of the frozen legacy evidence (002-R5 §3) | **Yes** | No — recognition is evidenced by executed cases and verified source | — correction applied | S5 |
| `if-expr` | Parser-evidenced, corpus-unevidenced | **Yes** | No | A probe using `if` in expression position | S5 |
| `array-literal` | Not evidenced at all | **Yes** | **Yes** — the surface form is a guess and must not be implemented | A corpus or source example | S9 |
| Identifier class beyond letters/`_`/`$` | Combining marks, other categories | **Yes** | No | Probes with combining marks and digit-leading names | S13 |
| Normalisation / confusables | Policy choice | **Yes** | No | An owner policy decision | S13 |
| Trivia and terminator normalisation | Policy choice | **Yes** (CST layer) | No | An owner policy decision | S14 |
| Feature-gate syntax | — | — | — | **CLOSED** — `implementation-gates.md` §3.2 | S15 |

### 4.1 Productions whose question **dissolved** rather than being answered

Three questions were narrowed by the legacy probes. These are legacy
observations or question reclassifications, not normative acceptance.

| Question | Why it dissolved |
|---|---|
| `assignment-expr` associativity | Tested legacy forms treat assignment as a statement; `a = b = 1` is rejected with `SEM027`. Other forms/targets require verification before a universal contract. |
| `expression` evaluation order | A semantic question, not a syntax rule. Sampled JVM operand/receiver/argument order is left-to-right; unsampled forms and targets remain NOT VERIFIED. |
| `nested-function-decl` capture | Nested declaration recognition is independent of closure semantics. A sampled capture is rejected; intended closure design remains OPEN. |

### 4.2 Unsupportable syntax — what must be **marked as unsupported**, not implemented

The following **nine syntax forms or form-families** lack sufficient evidence
for independent recognition and require explicit PARSE refusals until their
recognition is accepted:

`test-decl` · `application-decl` · `infra-decl` · `extern-decl` ·
`foreign-module-decl` · `entity-decl` · `annotation` (unknown names) ·
labelled `break`/`continue` · `array-literal`

Everything else in §4 is recognised and then semantically gated. **This
distinction is the difference between "we do not know what this means" and "we
do not know whether this exists"**, and the two require different diagnostics.

---

## 5. Ambiguity analysis

### 5.1 The generics-versus-comparison ambiguity — evidence-backed proposal

The classic ambiguity in a language with both `<` and generic type arguments is
`a < b > c`. It was probed
(`docs/audits/phase-1-legacy-probes.md` §6.3); the JVM sample distinguishes
the following legacy parse, but does not approve a new normative rule:

* `a < b > c` parses as a **left-associative comparison chain**.
* A generic application requires a **type** on the left; `a` is a value.

**Proposed grammar consequence (owner acceptance pending):** the `type-arguments`
production must be reachable only from `type-ref` and from the explicitly
evidenced generic-call postfix position — **never as a fallback for a `<` in
expression position**. The grammar now satisfies this: `type-arguments` is
referenced only by `nominal-type` and by `postfix-op`.

### 5.2 Examined alternatives — additional ambiguity remains

The following selected pairs were reviewed. This is not a proof of
unambiguity or of deterministic lookahead for the entire grammar:

| Potential ambiguity | Why it is not ambiguous |
|---|---|
| `record-decl`: `positional-record` vs `record-body` | Disjoint on `(` versus `{` |
| `type-decl`: the five declaration kinds | Disjoint on the leading keyword |
| `top-level-body`: `function-decl` vs `type-decl` | The parser's dispatch order is explicit and evidence-backed (`Parser.java parse()`), and a function begins with an identifier, `void`, a primitive type name, or `(` |
| `statement`: `expression-stmt` vs the keyword-led forms | Disjoint on the leading keyword |
| `binary-expr`: `as`/`instanceof` taking a type | Not an ambiguity: the production *requires* a `type-ref` for exactly those two operators |
| `switch-label`: pattern vs value | Not an ambiguity at the grammar level; the disambiguation is semantic (S5); `default` is its own arm form since 002-R3; the pattern-header alternatives are derived since 002-R5 (F5) |
| `lambda`: `(params) -> body` vs a parenthesised expression | **UNRESOLVED** on `(` — see §5.3 |

### 5.3 One residual lookahead requirement — recorded, not resolved

`primary` includes both `"(" , expression , ")"` and `lambda`, where `lambda` may
begin with `(`. These are **not** disjoint on the first terminal. The legacy
parser handles this with unbounded lookahead in `ExpressionParser.parsePrimary`
and `LambdaParser`. **The grammar does not encode that lookahead**, and this
report does not invent one.

* **Classification:** a **lexical/grammar boundary issue**, recorded as
  `[OPEN:S11]`-adjacent and owned by the parser design in Phase 2.
* **What would close it:** a statement of the lookahead rule, derived from the
  legacy parser's actual behaviour on ambiguous inputs, plus probes.
* **Consequence for Phase 2:** the parser must implement whatever lookahead the
  evidence supports, and must not assume the grammar is LL(1) at this position.
  **This is the single most important parser-design constraint this report
  surfaces.**

---

## 6. Lexical / grammar boundary check

The directive asks for "incorrect lexical/grammar boundaries". Three were found
and one remains open.

| # | Boundary | Finding | Status |
|---|---|---|---|
| B1 | Layout productions were defined but **not reachable** from `source` | `whitespace`, `line-terminator`, `comment`, `operator`, `delimiter`, `separator` were orphaned — the lexer could not be derived from the grammar | **FIXED** — `source` now produces `lexical-element`, which covers layout and tokens |
| B2 | `identifier` did not exclude keywords | The grammar said an identifier was any letter sequence, which would make `class` an identifier. Keywords are distinguished by *not* being identifiers | **FIXED** — `identifier = identifier-name - ( keyword \| reserved-invalid )` |
| B3 | `type-ref` did not admit `primitive-type` | A primitive type name was reachable only as an identifier, conflating two different things | **FIXED** — `type-ref = ( primitive-type \| nominal-type ) , …` |
| B4 | Raw newline inside a string literal | The current `string-char = any-char - ( "\\\"" \| "\\\\" )` no longer excludes line terminators, matching the sampled JVM legacy behavior (§4.3 of the probe record). | **CORRECTED DESCRIPTION** — S8's normative newline policy remains OPEN until owner acceptance. |

**B4 was a grammar/description mismatch, now corrected at recognition level.**
The normative decision about permitting a raw newline in the new language is
still S8's responsibility. Grammar consistency alone is not approval.

---

## 7. What this report does not claim

* **It does not claim the grammar is complete.** It claims it is *internally
  consistent*: no dangling symbols, no orphans, no duplicates, every production
  evidence-cited, every open question traceable.
* **It does not claim the grammar is correct.** A production can be
  self-consistent and still mis-describe the language. Correctness is established
  by the differential harness (Phase 2), not by a consistency checker. The R2
  production mapping (`docs/spec/grammar-evidence-map.md` §3) found three
  recognition gaps (F1, F2, F4) and one annotation error (F3); the 002-R4
  entry-readiness pass surfaced one further recognition gap — finding S-G1S-1 —
  corrected as F5 in 002-R5 after direct verification; all were corrected under
  owner authorization — F3 in R2, F1/F2/F4 in 002-R3, F5 in 002-R5 — and the
  map's per-production coverage is machine-checked (G13). The corrections
  removed known mis-descriptions; they do not establish correctness.
* **It does not resolve any semantic question.** The provisional productions are
  classified, not decided.
* **It does not claim the grammar is LL(1).** §5.3 identifies a position where it
  is not.
* **The checker is not compiler code** and is not part of the toolchain.
