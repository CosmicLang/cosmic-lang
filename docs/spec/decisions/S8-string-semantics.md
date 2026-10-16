# S8 — String and Escape Semantics

| | |
|---|---|
| **Status** | **OPEN**, with one part **PROPOSED** |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; Phase 2 implementation |
| **Related** | `docs/spec/language-spec.md` §2.4, `docs/spec/grammar.ebnf` §2.4, S13 |

## Question

1. **Q1** — Are **triple-quoted / raw / multiline** string literals part of the
   language?
2. **Q2** — Should an **unrecognised escape** keep being silently reinterpreted
   as the escaped character?
3. **Q3** — Is there **string interpolation**?
4. **Q4** — May a **raw newline** appear inside a `"…"` literal?
5. **Q5** — What is the exact escape set, and how are `\u` escapes handled for
   astral characters?
6. **Q6** — What is the string concatenation and equality contract?

## Evidence

* **E1 — the key correction.** The legacy lexer recognises the three-quote
  sequence `"""` **only in order to reject it**. `parser/Lexer.java`
  `skipTripleQuoted` consumes the block, advances past it, and emits one
  diagnostic (`LEX008`) whose message states that Kof has no triple-quoted
  (raw/multiline) string literal, that there is no interpolation, and that the
  user should use `"...\n..."` or concatenate with `+`.
  The in-source comment records that the sequence **used to fold silently to an
  empty string** and that this was fixed as a rule violation — which is exactly
  the silent-degradation class this project forbids.
  * **This contradicts a Phase 0 claim.** `docs/spec/compatibility-matrix.md`
    §3 item 8 recorded "triple-quoted form (lexer supports it)". That row is
    **wrong** and is corrected in this change.
* **E1** — Escape set: `\n`, `\t`, `\r`, `\\`, `\'`, `\"`, `\0`, `\uXXXX`.
  Evidence: `parser/Lexer.java` `readEscape`.
* **E1** — An **unrecognised escape is silent**: `readEscape`'s `default -> c`
  returns the escaped character itself and emits **no diagnostic**. So a source
  containing an unrecognised escape compiles and behaves as if the backslash
  were not there.
* **E1** — `\uXXXX` is a **single UTF-16 code unit** (`readUnicodeEscape`
  returns a `char`). Astral characters therefore require a surrogate pair of
  escapes. An incomplete escape is `LEX006`; an invalid hex digit is `LEX007`.
* **E1** — A **raw newline inside a `"…"` literal is consumed** by `readString`
  (it advances the line counter and appends the character), so the literal is
  not terminated by a newline. There is no diagnostic for a multi-line string.
  This is not exercised by the corpus.
* **E1 (executed)** — String behaviour is broadly covered: `strings`,
  `strings-escape-indent`, `strings-methods`, `strings-namespace`,
  `string-methods-extended`, `string-parts-valueof`, `string-switch-defaults`,
  `json-escaping`, all passing 222/222 on jvm/js/script.
* **E1 (negative)** — No executed case contains a triple-quoted literal, an
  unrecognised escape, or a raw newline inside a literal. The corpus therefore
  does **not** pin Q1–Q4.

## Options

| # | Option | Assessment |
|---|---|---|
| H1 | **Keep the rejection** of triple-quoted strings, keep the silent unknown escape | Maximum fidelity to the legacy lexer. But it keeps a silent reinterpretation, which contradicts the project's own rules |
| H2 | **Keep the triple-quote rejection** (it is a deliberate, documented, tested legacy decision) and **make an unknown escape a diagnostic** | Fixes the silent-degradation defect without adding a language feature. Risk: a legacy source relying on an unknown escape newly fails |
| H3 | **Add triple-quoted / raw multiline strings** as a new feature | A language *extension*. It has no legacy evidence, it would break the `LEX008` contract, and it is out of scope for a specification whose job is to describe the existing language |
| H4 | **Add interpolation** | Same as H3, and the `LEX008` message explicitly states interpolation does not exist |

## Recommendation (PROPOSED, not in force)

**H2.**

* **Q1** — **No triple-quoted, raw or multiline literals.** The legacy rejection
  is deliberate, documented in-source, and carries its own diagnostic code.
  Keeping it is fidelity, not a limitation. If the owner wants multiline
  strings, that is a **language extension** requiring its own record and a
  compatibility analysis — not a Phase 1 specification decision.
* **Q2** — **An unrecognised escape becomes an error.** This is the one place in
  the string surface where the legacy lexer silently reinterprets the program.
  The new toolchain emits a diagnostic naming the escape and offering the
  supported set. **This is a PROPOSED C2 divergence**, and it must be measured:
  if any corpus case contains an unrecognised escape, the divergence is real and
  must be recorded explicitly.
  Rationale: it is the same defect class as the triple-quote silent folding that
  the legacy project itself fixed, and the project's stated principle is that
  nothing fails silently.
* **Q3** — **No interpolation.** Stated explicitly by the legacy diagnostic and
  confirmed by the absence of any interpolation machinery in the lexer.
  Concatenation is `+`.
* **Q4** — A raw newline inside a `"…"` literal is **OPEN**. The lexer permits
  it silently; nothing tests it. Two readings are possible (permitted
  multi-line string, or an oversight). Phase 2 must produce a case and the owner
  must choose. **No rule is proposed now.**
* **Q5** — The escape set is the evidenced one. `\u` escapes stay
  **code-unit-based** for compatibility, but the new frontend must **diagnose a
  lone surrogate** rather than accepting an invalid scalar value — an
  ill-formed `\u` pair currently produces a broken UTF-16 sequence with no
  diagnostic. This is a **PROPOSED** tightening and belongs with S13.
* **Q6** — Concatenation is `+`; string equality is **structural** (value
  equality), evidenced by the executed string cases. The exact equality rule for
  strings versus objects is owned by **S12** (equality semantics).

## Compatibility consequence

* Q2 is a **C2 divergence** for sources with unrecognised escapes. Expected to
  be rare; must be measured against the corpus.
* Q5's lone-surrogate diagnostic is a **tightening** — programs that previously
  produced invalid text now fail. This is desirable but is still a divergence.
* Q1 preserves **C1 + C2** exactly.

## Open dependencies

* Q4 requires a new case and an owner decision.
* S13 owns the Unicode/surrogate interaction.
* S12 owns string equality as part of the general equality question.
