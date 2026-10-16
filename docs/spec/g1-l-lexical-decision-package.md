# G1-L Lexical Entry Decision Package

| | |
|---|---|
| **Status** | **READY FOR OWNER DECISION — G1-L is NOT entered.** This document accepts nothing; it proposes the lexical contract whose owner acceptance is the G1-L entry condition. |
| **Directive** | ENGINEERING DIRECTIVE 002-R4 §5 |
| **Scope** | The lexical contract of `cosmic-lexer`: emitted tokens and trivia, error cases, and the diagnostics carrying them. Tier-L records only (S8, S13, S14). |
| **Related** | `docs/spec/implementation-gates.md` §5 (G1-L) and §7 · `docs/spec/language-spec.md` §1–§3 · `docs/spec/grammar.ebnf` §2 · `docs/spec/diagnostic-codes.md` · `docs/spec/decisions/S8-string-semantics.md` · `docs/spec/decisions/S13-unicode.md` · `docs/spec/decisions/S14-trivia.md` · `docs/spec/phase-2-entry-readiness.md` |

> **Path convention.** A path prefixed `cosmcc:` refers to the legacy repository `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

> **Acceptance is not given here.** Every item below is a proposal with testable expected outcomes. Declining an item leaves its question OPEN; in that case the affected construct must be enforced as a **lexical gate** (`docs/spec/implementation-gates.md` §3.1, §7) — refused with a named diagnostic — never resolved by silently following the legacy behaviour or by inventing a rule.

---

## 1. What G1-L entry requires

`docs/spec/implementation-gates.md` §5 defines the entry condition:

> The owner accepts the lexical contract for emitted tokens and error cases (including invalid escapes, Unicode boundaries and newline handling); GATE/LEX diagnostic codes allocated for open cases.

This package supplies exactly what that condition asks for:

1. **Ten decisions** (D-L1 … D-L10) covering the directive's ten contract items, each with its evidence, a recommended normative option, testable expected outcomes, and a divergence classification.
2. **A proposed LEX diagnostic allocation** (§3). The registry lists no codes yet (`docs/spec/diagnostic-codes.md` §6), so nothing here can conflict with an existing code.
3. **The BOM round-trip check** (§4) required by the directive.

Three labels are used, and only these:

| Label | Meaning |
|---|---|
| **E1-implemented** | The Phase 0 foundation already behaves this way and is tested (`crates/cosmic-source`). Still *not* an accepted language rule. |
| **PROPOSED** | A recommendation awaiting the owner's G1-L decision. |
| **LEGACY-UNVERIFIED** | The legacy behaviour for this exact case has not been probed; the recommendation is new-toolchain policy and a differential case is owed. |

### 1.1 Summary of the ten decisions

| ID | Topic | Record | Recommendation | Disposition of evidence |
|---|---|---|---|---|
| D-L1 | UTF-8 validation and leading BOM | S13 Q6 | UTF-8 only; one leading BOM consumed; preserved as a prefix (see §4) | E1 + E1-implemented |
| D-L2 | Byte spans and scalar-value columns | S13 Q4 | Byte spans authoritative; scalar-value columns; legacy UTF-16 divergence declared | E1 + probe; E1-implemented |
| D-L3 | Identifier classes | S13 Q1 | Unicode letters/digits + `_`/`$`; no normalisation | E1 + probes; M-7 debt named |
| D-L4 | Whitespace, comments, CR/LF/CRLF | S14 | Three whitespace characters; non-nesting comments; three terminators preserved distinctly | E1 |
| D-L5 | Strings and the evidenced escape set | S8 | The evidenced eight escapes; shared with character literals | E1 + executed corpus |
| D-L6 | Triple-quoted syntax | S8 Q1 | Keep the rejection; one diagnostic | E1 + probe |
| D-L7 | Raw newline in `"…"` | S8 Q4 | **Permit** (legacy parity) | E1 + probe |
| D-L8 | Unknown escapes | S8 Q2 | **Error** (strict), declared C2 divergence | E1 + probe; confirmed defect |
| D-L9 | Lone-surrogate escapes | S13 Q5, S8 Q5 | **Error**; valid pairs decode to one scalar | E1; proposed tightening |
| D-L10 | Lossless trivia and byte preservation | S14 Q1 | Full byte partition; reprint is the identity | E1; design requirement |

No item below changes S8, S13 or S14's status; their proposals mature only through the owner action recorded in §6.

---

## 2. The ten decisions

### D-L1 — UTF-8 input validation and leading-BOM handling

**Question.** What is the source encoding contract, and how is a leading BOM treated, so that a lossless CST can still reprint the exact input bytes?

**Evidence.**

* The legacy lexer tolerates a single leading UTF-8 BOM (`EF BB BF`, U+FEFF) and starts after it: `cosmcc:parser/Lexer.java` `tokenize`, the `startsWith("\uFEFF")` guard, comment `OBS-008` (language-spec §1.1; S13 E1).
* Phase 0 rejects invalid UTF-8 at load and reports the byte offset of the first invalid byte (`crates/cosmic-source` `LoadError::NotUtf8`, verified in `docs/audits/phase-0-source-review.md`).
* U+FEFF is not one of the three whitespace characters (language-spec §1.4), so a BOM anywhere but the start reaches the legacy unexpected-character branch. **LEGACY-UNVERIFIED:** mid-file BOM behaviour has not been probed (evidence-map row 2 residual).

**Recommended normative option (PROPOSED).**

1. Source is UTF-8. Ill-formed input (invalid byte, truncated sequence, overlong form) is rejected at load with the first invalid byte offset; no replacement characters are substituted.
2. A single leading U+FEFF is **consumed** — it is not a code character — and **preserved as a zero-width prefix leaf** so reprinting reproduces the original bytes (§4).
3. U+FEFF elsewhere is not whitespace: it is an unexpected character (see LEX005 in §3) with a help note that a byte-order mark is recognised only at the start of a file.
4. At most one BOM is consumed; a second one immediately after the first is an unexpected character.

**Testable expected outcomes.**

| # | Input | Expected |
|---|---|---|
| T1 | `EF BB BF` + `main() {}` | token stream identical to the same source without the BOM; prefix leaf records bytes 0–3 |
| T2 | BOM-only file | empty token stream; one prefix leaf; no diagnostics |
| T3 | two leading BOMs + `main` | first consumed; second reported (LEX005); reprint byte-identical |
| T4 | file whose byte 0 is `0x80` | load error with offset 0 (existing Phase 0 behaviour) |
| T5 | truncated multi-byte sequence at EOF | load error with the offset of the sequence start |
| T6 | BOM inserted mid-file | LEX005 at that byte (LEGACY-UNVERIFIED) |

**Divergence.** Encoding acceptance is C0/C1-neutral: the BOM was already tolerated, and rejecting invalid UTF-8 matches the legacy behaviour of failing on input it cannot read.

---

### D-L2 — Byte-accurate spans and scalar-value diagnostic columns

**Question.** What are the authoritative source coordinates, and in what unit are diagnostic columns counted?

**Evidence.**

* The legacy lexer advances its column counter once per UTF-16 **code unit** (`cosmcc:parser/Lexer.java` `advance`, `source.charAt`), so an astral character occupies two legacy columns (S13 E1).
* Phase 0 counts columns in Unicode **scalar values** and stores validated byte spans (language-spec §1.3; `crates/cosmic-source`).
* The divergence is **measured**: on the line `main() { println("😀" +) }` the legacy compiler reported column 24 for `)`; scalar-value counting yields 23 — exactly +1 per astral character preceding the position (S13, executed probe).

**Recommended normative option (PROPOSED, implemented in Phase 0).**

1. **Byte spans are authoritative**: every token, trivia element and diagnostic carries validated byte offsets into the original file. No position is derived from character counts downstream.
2. **Columns are scalar-value counts** for presentation, derived from the span; the legacy UTF-16 counting is not adopted.
3. Tabs count as one column (Phase 0 behaviour).
4. A terminator resets the column: the first character after `\n`, `\r\n` or a lone `\r` is column 1; a CRLF pair never splits a column position.
5. The legacy divergence is declared and must be normalised by the differential harness on any line containing an astral character (S13 compatibility consequence).

**Testable expected outcomes.**

| # | Input | Expected |
|---|---|---|
| T1 | token after a multi-byte character | span covers exactly the token bytes; no span boundary inside a multi-byte sequence (validated by the span constructor) |
| T2 | `main() { println("😀" +) }` | `)` column = 23 (legacy: 24, recorded); differential harness normalises or restricts to the BMP |
| T3 | two astral characters on one line | divergence +2, per the recorded rule |
| T4 | `a\r\nb` | `b` is line 2, column 1; the CRLF is one trivia element |
| T5 | `a\rb` | same as T4 for the lone-CR case |
| T6 | `a\tb` | `b` is column 2 (tab = one column) |

**Divergence.** C0/C1-neutral; C2-relevant only for tooling that compares diagnostic text — which the harness policy already addresses.

**Status.** E1-implemented in `crates/cosmic-source`; owner acceptance pending (S13 Q4).

---

### D-L3 — Identifier start and continuation classes

**Question.** Which characters start and continue an identifier?

**Evidence.**

* Identifiers start with a Unicode letter, `_` or `$`, and continue with Unicode letters/digits, `_` or `$` (`cosmcc:parser/Lexer.java` `tokenize`, `readIdentifier`; language-spec §2.2). A digit is never admissible at the start (evidence-map row 12).
* Probes resolved the non-ASCII half: `café(41)` compiled and printed `42`; `_under(1)` and `$dollar(1)` both compiled (S13 Q1 executed probes).
* **Still OPEN (M-7):** identifier classes beyond letters/`_`/`$` — combining marks and other categories, and digit-leading names. The corpus contains ASCII identifiers only.

**Recommended normative option (PROPOSED).**

1. `identifier-start = unicode-letter | "_" | "$"`.
2. `identifier-part = unicode-letter | unicode-digit | "_" | "$"`.
3. **No normalisation**: precomposed and decomposed spellings are different identifiers (S13 Q2, unchanged).
4. **`$` remains permitted** (recognition parity with the probed behaviour; S13 keeps "whether `$` should remain" as its own policy question — this package recommends *keep*, and the owner can amend).
5. Classification authority: the Rust standard-library predicates (`char::is_alphabetic`, `char::is_alphanumeric`) under the zero-dependency policy — with an explicit caveat: Java's `Character.isLetter`/`isLetterOrDigit` general categories and the Rust Unicode properties are **not proven identical** on edge categories. The M-7 differential cases are owed as a Stage A deliverable; until they run, the classification authority is accepted, not verified.

**Testable expected outcomes.**

| # | Input | Expected |
|---|---|---|
| T1 | `café` | one identifier (probe-parity fixture) |
| T2 | `_under`, `$dollar` | identifiers (probe-parity fixtures) |
| T3 | `1abc` | number literal `1` followed by identifier `abc` — never merged |
| T4 | `e` + U+0301 combining acute | identifier `e`, then U+0301 is an unexpected character (combining marks do not continue; M-7 case owed — LEGACY-UNVERIFIED for the exact mark) |
| T5 | `_`, `$`, `__x`, `x_1` | valid identifiers |
| T6 | astral letter (e.g. U+10400) | valid identifier character (new-toolchain policy; LEGACY-UNVERIFIED) |

**Divergence.** The evidenced subset keeps parity; classification edge cases may diverge and must be measured (M-7) before any compatibility claim.

---

### D-L4 — Whitespace, comments and CR/LF/CRLF boundaries

**Question.** What is whitespace, what terminates a line, how do comments behave, and are the three terminator forms distinguished?

**Evidence.**

* Whitespace is exactly space (U+0020), tab (U+0009) and form feed (U+000C); no other character is (`cosmcc:parser/Lexer.java`; language-spec §1.4). A non-breaking space or ideographic space is not whitespace.
* Line terminators are `\n`, `\r\n` and a lone `\r`; the CRLF pair is consumed as **one** break (language-spec §1.2; evidence-map row 3).
* `//` runs to the end of the line; `/* … */` scans to the **first** `*/` and does not nest; an unterminated block comment produces **exactly one** diagnostic without cascading (S14 E1; the legacy project's own discipline).
* Phase 0 treats the three terminator forms as equivalent for line/column mapping and does not preserve the form (S14 E1); Unicode line separators do not break lines (tested in `crates/cosmic-source`).

**Recommended normative option (PROPOSED).**

1. Whitespace = U+0020, U+0009, U+000C only.
2. Terminators = `\n`, `\r\n`, lone `\r`; CRLF is one break. U+2028/U+2029 are neither whitespace nor terminators (LEGACY-UNVERIFIED by probe; predicted by source).
3. `//` excludes the terminator; `/* … */` is non-nesting and ends at the first `*/`; an unterminated block comment yields one diagnostic and one error region reaching EOF.
4. The exact terminator form is preserved as distinct trivia (S14 N1/Q4, unchanged); the grammar remains terminator-agnostic.
5. S14 Q3's optional interior-`/*` warning is **not** required at G1-L entry; it remains a deferred S14 proposal.

**Testable expected outcomes.**

| # | Input | Expected |
|---|---|---|
| T1 | NBSP, U+3000, U+2028, U+2029 | unexpected-character diagnostic |
| T2 | `/* a /* b */ c */` | the comment ends at the first `*/`; no nesting |
| T3 | unterminated `/*` | exactly one diagnostic; error region to EOF; no cascade |
| T4 | `// x` at EOF with no terminator | legal; comment trivia to EOF |
| T5 | mixed `\n` / `\r\n` / `\r` file | reprints byte-identical; line mapping agrees with Phase 0 |
| T6 | `a\r\nb` | CRLF is one trivia element; `b` is line 2 column 1 |

**Divergence.** N1 is C0–C3-neutral; non-nesting preserves the legacy reading of existing files (S14 compatibility note).

---

### D-L5 — Ordinary strings and the evidenced escape set

**Question.** What is the exact escape set, and does it also govern character literals?

**Evidence.**

* Escape set: `\n`, `\t`, `\r`, `\\`, `\'`, `\"`, `\0`, `\uXXXX` (`cosmcc:parser/Lexer.java` `readEscape`; language-spec §2.4; S8 E1).
* Character literals `'x'` share the escape reader; empty (`''`) and unterminated literals each carry a dedicated legacy code (language-spec §2.4).
* Executed string coverage is broad and passing (S8 E1: `strings`, `json-escaping`, `strings-methods`, … 222/222).

**Recommended normative option (PROPOSED).**

1. A string literal is `"…"` with the eight evidenced escapes; `\'` inside a string and `\"` inside a character literal are legal (shared reader).
2. `\uXXXX` decodes per D-L9 (code-unit based, pairs required for astral values).
3. Unterminated string → one diagnostic (LEX002 below); unterminated character literal → LEX004; empty character literal `''` → LEX003.
4. Multi-character character literals are **not evidenced** and must not be invented: until a rule exists, a character literal carrying more than one character value is refused with a named diagnostic, not silently accepted.
5. Triple-quoted and interpolated forms are governed by D-L6.

**Testable expected outcomes.**

| # | Input | Value / result |
|---|---|---|
| T1 | each of the eight escapes | the documented decoded character |
| T2 | `"\u0041"` | `"A"` |
| T3 | `'\''` and `"\'"` | `'` in both |
| T4 | `` `"x` `` (unterminated) | one diagnostic; error region reaches the line end or EOF |
| T5 | `''` and `'ab'` | one diagnostic each (empty; multi-character refused) |
| T6 | literal containing `é` and astral text | bytes preserved; reprint exact |

**Divergence.** Preserves legacy C1/C2 for the evidenced set; item 4's refusal is a tightening only for an unevidenced form.

---

### D-L6 — Triple-quoted syntax is rejected

**Question.** Are triple-quoted / raw / multiline string literals part of the language?

**Evidence.**

* The legacy lexer recognises `"""` **only to reject it**: one diagnostic whose message states that Kof has no raw/multiline literal and no interpolation, and directs the user to `"...\n..."` or `+` concatenation; the in-source comment records that the sequence once folded silently to an empty string and that this was fixed as a rule violation (S8 E1; probe Q1: rejected).
* No corpus case contains the form (S8 E1 negative).
* The `LEX008` rejection is deliberate, documented and tested — keeping it is fidelity, not a limitation (S8 Q1 recommendation, unchanged).

**Recommended normative option (PROPOSED).**

1. A `"` immediately followed by two more quotes begins the refused triple-quote sequence.
2. Exactly **one** diagnostic per occurrence, naming the rule (no raw/multiline strings, no interpolation; use `"...\n..."` or concatenation).
3. The sequence is consumed to the closing `"""` if one exists, else to EOF — one diagnostic either way; the consumed bytes are preserved as an error region; lexing resumes after it.
4. `""` (two quotes) remains a legal **empty string**; the lexer must distinguish it from `"""`.

**Testable expected outcomes.**

| # | Input | Expected |
|---|---|---|
| T1 | `"""x"""` | one diagnostic; error region holds exactly those bytes; no silent empty string (the historical defect) |
| T2 | `""` | valid empty string, no diagnostic |
| T3 | unterminated `"""x` | one diagnostic; error region to EOF; no cascade |
| T4 | statement following a refused triple-quote on the next line | tokenises normally |

**Divergence.** Preserves the legacy rejection exactly (C1/C2); diagnostic text and code are new-toolchain.

---

### D-L7 — Raw newlines inside ordinary string literals

**Question.** May a raw line terminator appear inside a `"…"` literal?

**Evidence.**

* The legacy `readString` consumes a raw newline, advances the line counter, and appends the character; there is no diagnostic (S8 E1).
* The executed probe is decisive about the fact: a literal spanning two lines printed two lines, exit 0 (S8 Q4: **PERMITTED**).
* The corpus does not exercise it (S8 E1 negative), and S8 Q4's normative recommendation is currently "no rule is proposed now" — this package *is* the rule the record is waiting for.

**Recommended normative option (PROPOSED).**

1. **Permit**: a raw line terminator inside `"…"` is part of the literal's content; the line counter advances; no diagnostic.
2. This matches both the legacy lexer and the grammar, whose `string-char` excludes only the quote and the backslash (report §6, B4).
3. Exact value semantics for a `\r\n` inside a literal (one content character or two) is a **required differential case** before the harness treats it as fixed; the reprint guarantee (D-L10) is unaffected either way.

**Alternative recorded.** Forbidding the form would be a C1 divergence for legacy sources containing multiline literals — not recommended, because the `LEX008` message itself directs users to the multiline capability.

**Testable expected outcomes.**

| # | Input | Expected |
|---|---|---|
| T1 | two-line literal, printed | output matches the probe (two lines, exit 0 for the equivalent program) |
| T2 | literal token text | exact bytes; reprint identity |
| T3 | position after the literal | correct line/column per D-L2 |
| T4 | CRLF inside a literal | recorded as differential-pending; not asserted until measured |

---

### D-L8 — Unknown escape sequences

**Question.** Does an unrecognised escape keep being silently reinterpreted as the escaped character?

**Evidence.**

* Legacy: an unrecognised escape returns the escaped character itself (`default -> c`) with **no diagnostic** (`cosmcc:parser/Lexer.java` `readEscape`; language-spec §2.4).
* Probe: `"a\qb"` printed `aqb`, no diagnostic, exit 0 (S8 Q2: **silently dropped**).
* S8 classifies this as a **confirmed defect** — the same silent-reinterpretation class as the historical triple-quote folding — with the H2 fix proposed; the corpus contains no case that relies on it (negative evidence).

**Recommended normative option (PROPOSED — recommended explicitly per Directive 002-R4 §5).**

1. An unrecognised escape is an **error** (LEX008 below), one diagnostic per occurrence, naming the character and listing the supported set.
2. The backslash is **not** silently dropped; scanning continues to the closing quote so recovery is local; the literal is preserved in the CST as an error-marked leaf.
3. This is a **declared C2 divergence** from the legacy silent-dropping behaviour. Expected corpus impact is zero (negative evidence) and must be measured against the corpus by the differential harness before any compatibility statement.

**Testable expected outcomes.**

| # | Input | Expected |
|---|---|---|
| T1 | `"a\qb"` | one diagnostic (legacy: silent `aqb` — recorded divergence); no artifact is produced for the program |
| T2 | `"\x41"` | one diagnostic |
| T3 | `"a\\b"` | no diagnostic; value `a\b` |
| T4 | two unknown escapes in one literal | two diagnostics, deterministic order |
| T5 | reprint of the error-marked literal | byte-identical (D-L10) |

---

### D-L9 — Lone-surrogate escapes

**Question.** What happens when `\uXXXX` names a surrogate code unit without its pair?

**Evidence.**

* `readUnicodeEscape` returns a single UTF-16 `char`, so a lone high surrogate is accepted silently and produces an ill-formed sequence when used (S13 E1; S8 E1).
* S13 Q5 and S8 Q5 both propose an error — the same silent-degradation class the project forbids; the tightening is expected to affect zero corpus cases, to be measured.

**Recommended normative option (PROPOSED).**

1. `\uXXXX` is exactly four hex digits; the value is a UTF-16 code unit.
2. A high surrogate (U+D800–U+DBFF) must be **immediately** followed by a `\u` escape naming a low surrogate (U+DC00–U+DFFF); together they decode to one Unicode scalar value.
3. Any unpaired surrogate — high without a following low, or low without a preceding high — is an error (LEX010 below).
4. Fewer than four hex digits → LEX006; a non-hex digit → LEX007.
5. Direct astral characters in the source (valid UTF-8) need no escape and are unaffected.

**Testable expected outcomes.**

| # | Input | Expected |
|---|---|---|
| T1 | `"\uD83D\uDE00"` | value equals the direct `"😀"` (one scalar) |
| T2 | `"\uD800"` | one LEX010 diagnostic |
| T3 | `"\uDC00"` | one LEX010 diagnostic |
| T4 | `"\uD800x"` | one LEX010 diagnostic (a following non-escape does not pair); recovery continues |
| T5 | `"\u12"` | LEX006 |
| T6 | `"\u12G4"` | LEX007 |

**Divergence.** A tightening (C2) for sources relying on ill-formed text; expected zero reliance, to be measured.

---

### D-L10 — Lossless trivia and exact source-byte preservation

**Question.** Does the lexer preserve every byte of the file, including trivia and malformed regions?

**Evidence.**

* The legacy lexer discards trivia; there is no lossless stream, and formatting cannot round-trip (S14 E1).
* The design report (page 14) requires a lossless CST that "preserves comments and errors for formatter and LSP" — a stated design requirement, not a nicety (S14 E1).
* The grammar already routes layout through `lexical-element` (grammar §2.1), so a byte-complete stream is the derivable interpretation; permissive trivia policy is S14's N1 proposal.

**Recommended normative option (PROPOSED).**

1. The lexer emits a stream in which **every byte of the file is covered exactly once, in order**, by exactly one of: a token, a trivia element (whitespace run, comment, terminator with its exact form, BOM prefix), or an error region holding the bytes consumed by a recovery step.
2. No normalisation beyond the D-L1 prefix handling; no byte is dropped or duplicated.
3. **Reprint identity:** concatenating the leaves in byte order reproduces the input file byte-for-byte, for every input, valid or malformed. This makes S14 N1's guarantee testable at the lexer boundary before any parser exists.
4. Trivia *attachment* (which token a comment belongs to) is a tree-construction concern owned by G1-S; the lexer guarantees presence, order, exact bytes, and no gaps or overlaps.

**Testable expected outcomes.**

| # | Input | Expected |
|---|---|---|
| T1 | property test over corpus fixtures and fuzz inputs | `reprint(lex(src)) == src` byte-for-byte |
| T2 | any input | leaf byte ranges partition `[0, len)`; no gaps, no overlaps |
| T3 | unterminated comment or string | error leaf reaches EOF; partition holds |
| T4 | zero-byte file | empty stream; reprint = empty |
| T5 | BOM-only file | prefix leaf; reprint = the three bytes |

---

## 3. Proposed LEX diagnostic allocation

`docs/spec/diagnostic-codes.md` §3 allocates the `LEX` namespace to `cosmic-lexer` and reserves `000`–`099` for conditions present at first release. The registry lists no codes yet, so the following is an **allocation plan**, not registry entries: each code is minted into the registry when the component is implemented, with a test as its `Evidence` entry (§6). Before the first release, renumbering is free (§2.2); after it, meanings are frozen (§2.1). Legacy numbers are never inherited or compared (§2.3) — some proposed digits therefore denote conditions different from the legacy number of the same digits, and the differential harness maps **conditions**, never numbers.

| Proposed | Condition | Severity | Record | Legacy counterpart (informational only) |
|---|---|---|---|---|
| `LEX001` | unterminated block comment | Error | S14 | legacy `LEX001`, same condition |
| `LEX002` | unterminated string literal | Error | S8 | legacy `LEX002`, same condition |
| `LEX003` | empty character literal | Error | S8 | legacy `LEX003`, same condition |
| `LEX004` | unterminated character literal | Error | S8 | legacy `LEX004`, same condition |
| `LEX005` | unexpected character (misplaced U+FEFF, NBSP, U+3000, U+2028/U+2029, `~`, combining mark outside an identifier, any other) | Error | S13, S14 | legacy `LEX005`, same condition |
| `LEX006` | incomplete `\u` escape | Error | S8, S13 | legacy `LEX006`, same condition |
| `LEX007` | invalid `\u` escape (non-hex digit) | Error | S8, S13 | legacy `LEX007`, same condition |
| `LEX008` | unknown escape sequence (D-L8, new strict rule) | Error | S8 Q2 | **no legacy code** (legacy was silent) |
| `LEX009` | triple-quoted string rejected (D-L6) | Error | S8 Q1 | legacy `LEX008` condition, different digits |
| `LEX010` | lone-surrogate escape (D-L9) | Error | S13 Q5, S8 Q5 | **no legacy code** (legacy was silent) |

Notes.

* Severities are Error per S15's policy: these conditions make the token stream unusable as written, and S15 requires an explicit diagnostic rather than degradation.
* **Not allocated here:** the optional interior-`/*` warning (S14 Q3) and the confusable-identifier warning (S13 Q3) remain deferred proposals; the confusable warning belongs to the `UNI` namespace (owned by `cosmic-sema`), not `LEX`.
* Invalid UTF-8 is currently a *load error* (`crates/cosmic-source`), not a diagnostic code. If it is later surfaced as a diagnostic it belongs to the `SRC` namespace; recorded for completeness, not required by G1-L.
* A declined decision (see §5) does not mint a code here: the affected construct is refused by a lexical gate, and the `GATE` code is allocated when that gate is implemented.

---

## 4. The BOM round-trip check

The directive asks explicitly: *does consuming the leading BOM make the required byte-for-byte CST round-trip impossible?*

**Answer: yes, if the BOM is only consumed.** The three bytes `EF BB BF` would be absent from every leaf, and `reprint(cst)` could not reproduce the file. Consuming cannot be silently combined with D-L10's identity property.

**Proposed preservation mechanism (PROPOSED).** The BOM is consumed as a *code character* (it never influences parsing) but is represented as a **prefix leaf** of the CST (category: file prefix, not ordinary trivia). The prefix leaf:

* carries the exact three bytes and the byte range `0..3`;
* is emitted at most once, only at offset 0;
* participates in the D-L10 byte partition and the reprint identity;
* is what the grammar production `bom` in `source = [ bom ] , { lexical-element }` (grammar §2.1) corresponds to — **no grammar change is required**;
* refines S14 Q2's draft wording ("consumed and not represented as trivia"): consumed as a code character, preserved as a *prefix*, not represented as ordinary trivia. The owner accepts this refinement with D-L1.

No other preservation mechanism (for example storing the prefix outside the tree) is proposed; the prefix leaf keeps the reprint property local to the tree and survives formatter and LSP round-trips unchanged.

---

## 5. What declining an item means

If the owner declines rather than accepts an item, the question stays OPEN and the affected construct becomes a **lexical gate** (`docs/spec/implementation-gates.md` §3.1, §7): the compiler refuses it with a stable `GATE` diagnostic naming the construct, the decision record and the unsupported operation, and produces no artifact. Examples:

* **Decline D-L8** → an unknown escape is refused as "not yet specified" (the silent legacy drop is *not* adopted as the interim behaviour; it is the confirmed defect).
* **Decline D-L7** → a raw newline inside a string is refused as "not yet specified" (neither silently permitted nor silently forbidden).
* **Decline D-L3 items** → the authority for classification (letters beyond the probed set, `$` retention) is gated for non-ASCII/`$` identifiers until decided.

Refusing-with-a-name is the only non-choice this project permits; it is also what keeps a later acceptance from breaking working code silently.

---

## 6. Owner action checklist

| # | Decision | ☐ Accept as proposed | ☐ Accept with amendments (state) | ☐ Decline (lexical gate) |
|---|---|---|---|---|
| D-L1 | UTF-8 + BOM prefix (incl. §4 refinement) | ☐ | ☐ | ☐ |
| D-L2 | Byte spans + scalar columns | ☐ | ☐ | ☐ |
| D-L3 | Identifier classes | ☐ | ☐ | ☐ |
| D-L4 | Whitespace / comments / terminators | ☐ | ☐ | ☐ |
| D-L5 | Strings + escape set | ☐ | ☐ | ☐ |
| D-L6 | Triple-quote rejection kept | ☐ | ☐ | ☐ |
| D-L7 | Raw newline permitted | ☐ | ☐ | ☐ |
| D-L8 | Unknown escape = error | ☐ | ☐ | ☐ |
| D-L9 | Lone surrogate = error | ☐ | ☐ | ☐ |
| D-L10 | Lossless byte partition | ☐ | ☐ | ☐ |
| §3 | LEX allocation plan accepted | ☐ | ☐ | ☐ |

**G1-L is entered only when all eleven rows are decided and the LEX allocation is accepted.** This document does not enter it; Directive 002-R4 does not authorize any compiler implementation.
