# G1-S Syntax Entry Decision Package

| | |
|---|---|
| **Status** | **READY FOR OWNER DECISION — G1-S is NOT entered.** This document accepts nothing; it proposes the parser-recognition contract whose owner acceptance is the G1-S entry condition. |
| **Directive** | ENGINEERING DIRECTIVE 002-R4 §6 |
| **Basis** | `docs/spec/grammar.ebnf` (119 productions, corrected F1/F2/F4) and `docs/spec/grammar-evidence-map.md` (119/119, machine-checked) |
| **Scope** | Recognition only: the recognized syntax subset, unsupported families, disambiguation, recovery and CST obligations for `cosmic-syntax`. Semantic adoption is explicitly out of scope (see §9). |
| **Related** | `docs/spec/implementation-gates.md` §5 (G1-S) and §7 · `docs/spec/grammar-consistency-report.md` · `docs/spec/decisions/S15-feature-gates.md` · `docs/spec/decisions/S5-pattern-matching.md` · `docs/spec/g1-l-lexical-decision-package.md` · `docs/spec/phase-2-entry-readiness.md` |

> **Path convention.** A path prefixed `cosmcc:` refers to the legacy repository `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

---

## 1. What G1-S entry requires

`docs/spec/implementation-gates.md` §5 defines the entry condition:

> The owner accepts the recognized syntax subset; each unsupported syntax production is identified, each ambiguous construct has a documented disambiguation or a PARSE diagnostic; CST byte round-trip and recovery obligations are specified.

This package supplies each element:

1. the **recognized subset** from the corrected grammar and its machine-checked evidence map (§3);
2. the **unsupported families** and their refusal contract (§4);
3. the **ambiguous constructs** with disambiguation or PARSE behavior (§5) — including one finding that must be resolved before the subset can be accepted in full (§3.3);
4. the **recovery obligations** (§6) and the **CST round-trip invariants** (§7);
5. the proposed PARSE diagnostic conditions (§8).

## 2. The recognition model

The governing principle is `docs/spec/implementation-gates.md` §3.1–3.2: **the parser recognises the evidenced grammar; open semantics never cause parser rejection.** Gates fire at the semantic operation that consumes a rule — not at syntax. The evidence map's status column is about recognition only:

| Status | Count | Meaning |
|---|---|---|
| `RECOGNISED` | 108 | The parser may recognise and preserve the production; open semantics are gated later (Tier T), not here. |
| `UNSUPPORTED` | 10 | The parser must emit a PARSE refusal for this construct until its recognition is accepted (§4). |
| `REJECTED` | 1 | Refused by design: the reserved-invalid tokens (`fun`, `fn`, `func`). |
| **Total** | **119** | One row per production; machine-checked by G13. |

## 3. The evidenced recognition subset

### 3.1 What the subset is

**The parser recognises all 108 `RECOGNISED` productions of the corrected grammar** — including constructs whose *semantics* remain OPEN (S1, S2, S5, S6, S7, S9, S10, S11, S12, S13, S14). Recognition is not adoption: a program using only settled operations must parse and (later) type-check normally; a program that reaches an open *operation* fails at the semantic gate that names it.

### 3.2 Corrections already authorized (F1/F2/F4)

The subset is defined by the corrected grammar; the following corrections were authorized by the owner (ENGINEERING DIRECTIVE 002-R3 §1) and are applied, taking the production count from 117 to **119**:

| Finding | Correction applied | Recognition examples |
|---|---|---|
| F1 | `break-stmt`, `continue-stmt` added (unlabelled only; labelled remain refused) | `docs/spec/grammar-evidence-map.md` §4, example 1 |
| F2 | `switch-stmt-arm` (colon, statement list) and `switch-expr-arm` (arrow, single expression) separated; `switch-arm`/`switch-body` removed; `default` is its own arm form | same, example 2 |
| F4 | `parameter = ( identifier ":" type-ref \| type-ref identifier ) [ "=" expression ]` — both evidenced orders and `= default` | same, example 3 |
| F3 | annotation-level correction (three-clause `for` is executed via `loops-control`) | report §4.1, F3 |

### 3.3 Finding S-G1S-1 — attested pattern header forms are not derivable

**This is the one item that must be resolved before the recognized subset can be accepted in full.**

* **Evidence.** The executed corpus contains two arm-header forms beyond value and bare-type labels: a **destructuring pattern** — `switch-break-in-loop` exercises `case Point(var x, var y):` — and a **type-and-binding pattern** — `sealed-switch` exercises `case Circle c -> …` (S5 E1, read and recorded in 002-R3; the parser entry point is `parseSwitchCasePatternOrValue`, `cosmcc:parser/StatementParser.java`).
* **Observation.** The current production is `switch-label = expression | type-ref` (grammar §3.6). It derives value labels (`case 1:`, `case "two":`) and bare type labels, but **neither attested header form**: `Circle c` is not an expression or a type-ref, and `Point(var x, var y)` is not an expression under the corrected grammar. The evidence map's `switch-label` row records the pattern/value disambiguation as semantic (S5) but does not model the attested header surface.
* **Classification.** Same defect class as F1/F2/F4: an executed, evidenced form that the grammar as written cannot derive. It was **not** part of the authorized corrections, and Directive 002-R4 authorizes no grammar change — so it is **reported here, not corrected**.
* **Why it blocks full acceptance.** G1-S asks the owner to accept a *recognized syntax subset*. Accepting a subset that silently omits two executed forms would make the summary in §3.1 false for `sealed-switch` and `switch-break-in-loop`.
* **Options.**
  1. **Authorize a bounded recognition correction (recommended).** A new F-class correction derived from `parseSwitchCasePatternOrValue` plus the two executed cases, exactly as F1/F2/F4 were; it must first pin the exact header forms (a bounded re-read of the two case files is owed, since the current record attests but does not exhibit the source text), then update the grammar, the evidence map (G13) and the count, and re-run the checker. An illustrative shape — **not a specification and not to be implemented as written** — is `switch-label = expression | type-ref | binding-pattern | destructuring-pattern` with `binding-pattern = type-ref , identifier` and a parenthesised pattern-argument list for destructuring; the disambiguation is one-token lookahead after the type/name, matching the parser method.
  2. **Declare the two header forms refused** with PARSE diagnostics until evidenced again. Not recommended: it contradicts executed evidence and S5's classification, and it would refuse programs the legacy toolchain accepts.
* **What this does not change.** S5 remains OPEN; pattern *semantics* (binding rules, exhaustiveness, fallthrough) are untouched and stay gated (§9).

### 3.4 Recognition notes and latent items

| Item | Recognition position |
|---|---|
| `if-expr` | **Recognised** — parser-evidenced (`parseIfExprBranch`); no corpus exercise; surface provisional (S5). Not a refusal: recognition follows the parser, semantics stay gated. |
| `new-expr` | **Recognised** — `parseNewExpression`; no executed case is cited for it; recognition stands, no semantics adopted. |
| `for-stmt` three-clause form | **Recognised** — executed (`loops-control`, F3). |
| `sealed` modifier, contextual keywords (`test`, `application`, `infra`, `foreign`, `module`) | **Recognised** by word + one-token lookahead; remain valid identifiers elsewhere (grammar §3.2). |
| Parameter-type omission, return-type default | **Not derivable** — not evidenced (S1). A parameter without a type is a syntax error until evidence and a rule exist; no fallback is invented. |
| `fun` / `fn` / `func` | **REJECTED** — reserved-invalid; the parser refuses them explicitly and they are never identifiers (`cosmcc:lang/KofKeywords.java` header `SG-001`; `rejectFunctionKeyword`). |

## 4. Unsupported syntax families — the refusal contract

The nine families of `docs/spec/grammar-consistency-report.md` §4.2, their productions in the map, and the required behavior. "Refused" means: recognise the construct's start precisely enough to name it, emit **one** refusal diagnostic, consume its region if it can be delimited (else synchronise at the next statement boundary), preserve the consumed bytes in an error node, and produce no artifact.

| # | Family | Map productions | Refusal notes |
|---|---|---|---|
| 1 | `test` declaration | `test-decl` | word + string-literal lookahead identifies it; refuse whole declaration |
| 2 | `application` declaration | `application-decl` | word + `{` lookahead |
| 3 | `infra` declaration | `infra-decl` | word + string-literal lookahead |
| 4 | `extern` / foreign function interface | `extern-decl`, `function-signature` | security-sensitive; refuse with the `extern` token |
| 5 | `foreign module` declaration | `foreign-module-decl` | refuse whole declaration |
| 6 | `entity` declaration | `entity-decl` | keyword reserved; semantics unknown |
| 7 | annotations | `annotation`, `annotation-args` | all annotation names refused at entry (vocabulary unknown; the report's "unknown names" phrasing becomes name-level granularity only after a vocabulary is accepted) |
| 8 | labelled `break` / `continue` | (no production — refused by absence) | the grammar derives unlabelled forms only; a label after the keyword receives a dedicated diagnostic |
| 9 | `array-literal` | `array-literal` | no evidence located; the brace form is a placeholder and must not be implemented as if settled |

Everything else in the provisional classification parses and is semantically gated where its rule is open. The distinction is the report's: *"we do not know whether this exists"* (refuse) versus *"we do not know what this means"* (parse, gate later).

## 5. Ambiguities and disambiguation

### 5.1 Lambda versus parenthesised expression (the residual lookahead)

`primary` includes both a parenthesised expression and a lambda that may begin with `(`; they are **not disjoint on the first terminal**. The legacy parser resolves this with unbounded lookahead (`ExpressionParser.parsePrimary`, `LambdaParser`); the grammar does not encode it (report §5.3, "the single most important parser-design constraint this report surfaces").

**Required disambiguation (PROPOSED).**

1. The parser must **not** assume LL(1) at `(`.
2. On `(`, scan the parenthesised region; if the matching `)` is followed by `->`, parse the region as lambda parameters; otherwise as a parenthesised expression. The scan is deterministic and bounded by the region.
3. The trailing-lambda same-line rule is applied as recorded: a `{` starting on the same line as the preceding expression attaches as a trailing lambda block (grammar §6.1 note; PROPOSED NORMATIVE RULE — its adoption is part of this acceptance).
4. Differential cases owed before the rule is treated as settled: `(x) -> x` (untyped single parameter), `(x: Int) -> x` (executed form), `()` empty parameters, `(a + b)` expression, nested parentheses, and `(` immediately before a line boundary.

### 5.2 Generics versus comparison

Resolved in the grammar: `type-arguments` is reachable only from `type-ref` and the evidenced generic-call postfix — never as a fallback for a `<` in expression position; `a < b > c` is a left-associative comparison (probe §6.3; report §5.1). Recognition follows the grammar; accepting this subset accepts that consequence.

### 5.3 Switch label: pattern versus value

The disambiguation is **semantic** (S5, gated): label *syntax* is recognised under `switch-label`, both forms of arm are separate productions (F2), `default` is its own arm form, and guards are recognised on `case` arms only (no guard rule is adopted). The header-surface finding in §3.3 applies here.

## 6. Parser recovery obligations

From S15 Q3 and the staged proposal's E3 contract, as binding entry requirements for `cosmic-syntax`:

1. **Multiple independent errors**: a single run reports them (test fixtures must demonstrate at least three).
2. **Termination**: no infinite loop on any input, including malformed input; fuzz-tested.
3. **One diagnostic per construct**: no cascades (the legacy project's own discipline).
4. **No panic, ever**, on any input.
5. **Deterministic**: same input, same diagnostics, same canonical order.
6. **Bounded work per input** (the 4 GiB pre-read cap remains in force).
7. **No fabrication**: recovery may not invent tokens or bytes; every consumed region is preserved in an error node.
8. **Refusals are not failures of recovery**: a refused family yields exactly one diagnostic and the parse continues.
9. **Semantics-blindness**: an OPEN semantic rule never causes a parse rejection.

**Testable expected outcomes.** three independent errors in one file → three diagnostics in canonical order; unclosed delimiter → one diagnostic and a lossless error region; a refused `extern` declaration between valid functions → one refusal, valid neighbors parsed; random byte/character fuzz streams → termination with a well-formed diagnostic set or none.

## 7. CST round-trip invariants

1. **Byte-for-byte reprint**: source → CST → source is the identity, for valid and malformed inputs, including all trivia, the BOM prefix (G1-L D-L1/§4) and error regions.
2. The parser consumes the **lossless** lexer stream without discarding anything (G1-L D-L10); trivia attachment is a tree decision, never a deletion.
3. Token leaves carry exact byte spans; tree construction never synthesises bytes absent from the source (zero-width structural nodes, if any, are flagged as such and contribute no text).
4. A CST exists for **every** input, including refused constructs and malformed regions.
5. Reprint is deterministic (no map-order effects, no normalisation).
6. The round-trip property is exercised by the fuzz/property suite (deliverable E8) over corpus fixtures and generated malformed inputs.

## 8. PARSE diagnostic behavior for unsupported syntax

The `PARSE` namespace is allocated to `cosmic-syntax` (`docs/spec/diagnostic-codes.md` §3); no code exists yet, so the following is an **allocation plan** under the same minting rules as G1-L (§6: minted with a test as `Evidence`; pre-release renumbering free; legacy numbers not inherited).

| Proposed | Condition | Severity | Closing evidence |
|---|---|---|---|
| `PARSE001` | unsupported syntax family refused (names the family and its record) | Error | one negative test per family |
| `PARSE002` | unexpected token | Error | recovery fixtures |
| `PARSE003` | unexpected end of input (unclosed construct) | Error | recovery fixtures |
| `PARSE004` | labelled `break`/`continue` not supported | Error | dedicated fixture |

Requirements: each refusal names the construct, the family and the decision record; produces **no artifact**; exits through the compilation-failure path (S15 Q5); and does not cascade. The reserved-invalid keywords (`fun`/`fn`/`func`) are refused at parse entry with a specific message (condition class `PARSE002` unless the owner directs a dedicated code).

## 9. What this package does not do

* It does **not** change the grammar. Finding S-G1S-1 is reported for decision, not applied.
* It does **not** adopt semantics: exhaustiveness, fallthrough, guards, destructuring/binding meaning, evaluation order, overloads, equality, concurrency — every Tier-T/M question stays exactly where S1–S18 put it (OPEN or GATED).
* It does **not** enter G1-S, authorize Stage A, or authorize implementations. **Recognition acceptance is a separate owner action from semantic acceptance**, per `docs/spec/implementation-gates.md` §3.1 and the acceptance record in §5 of that document.

## 10. Owner action checklist

| # | Decision | ☐ Accept as proposed | ☐ Amend (state) | ☐ Decline (keep gated) |
|---|---|---|---|---|
| 1 | Recognized subset: all 108 `RECOGNISED` productions parse; semantics gated downstream (§3.1–3.2) | ☐ | ☐ | ☐ |
| 2 | S-G1S-1: authorize a bounded F-class correction for the attested pattern header forms (§3.3, recommended) *or* declare them refused | ☐ | ☐ | ☐ |
| 3 | Refusal contract for the nine unsupported families incl. labelled break/continue (§4) | ☐ | ☐ | ☐ |
| 4 | Lambda lookahead rule + trailing-lambda same-line rule (§5.1) | ☐ | ☐ | ☐ |
| 5 | Recovery obligations (§6) and CST round-trip invariants (§7) | ☐ | ☐ | ☐ |
| 6 | PARSE allocation plan (§8) | ☐ | ☐ | ☐ |

**G1-S is entered only when rows 1–6 are decided and S-G1S-1 is resolved (authorized correction applied, or explicit refusal recorded).** This document does not enter it; Directive 002-R4 does not authorize any compiler implementation.
