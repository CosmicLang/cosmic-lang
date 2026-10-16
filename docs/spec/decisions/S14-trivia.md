# S14 — Trivia Preservation and Normalisation

| | |
|---|---|
| **Status** | **PROPOSED NORMATIVE RULE** — owner acceptance outstanding |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; Phase 2 implementation |
| **Related** | `docs/spec/language-spec.md` §3, `docs/spec/grammar.ebnf` §2.1, S13, S15 |

## Question

1. **Q1** — Must the frontend preserve **trivia** (whitespace, line breaks,
   comments) losslessly?
2. **Q2** — Is any normalisation permitted beyond the leading BOM?
3. **Q3** — Do **block comments nest**?
4. **Q4** — How are **line terminators** represented in the preserved form?
5. **Q5** — What is the **formatter** contract, given the preserved form?

## Evidence

* **E1** — The legacy lexer **discards** trivia: it skips whitespace and
  comments and emits no token for them (`parser/Lexer.java` `tokenize`,
  `skipLineComment`, `skipBlockComment`). There is no lossless stream.
* **E1 — consequence.** Because trivia is discarded, the legacy tooling cannot
  round-trip formatting. The legacy repository nonetheless ships a formatter
  (`kof-cli/src/main/java/dev/kof/cli/Fmt.java`), which therefore must
  reconstruct formatting from the token stream plus its own layout rules. That
  is a **reconstruction**, not a preservation, and it cannot preserve
  user-chosen layout.
* **E1** — The legacy lexer's `skipBlockComment` scans to the **first** `*/`, so
  block comments **do not nest**.
* **E1** — An unterminated block comment produces exactly one diagnostic
  (`LEX001`). The lexer deliberately does not cascade.
* **E1** — Line terminators are `\n`, `\r\n` and lone `\r`, and in the CRLF case
  the pair is consumed as one break (`parser/Lexer.java` `tokenize`). The
  legacy lexer **does not record which terminator form was used**.
* **E1** — The design report (page 14) requires a **lossless CST** that
  "preserves comments and errors for formatter and LSP". So losslessness is a
  stated design requirement of the new toolchain, not an optional nicety.
* **E1** — Phase 0's `cosmic-source` treats all three terminator forms as
  equivalent for line/column mapping and **does not preserve the original form**
  either; it exposes line *text* with the terminator stripped.

## Options

| # | Option | Assessment |
|---|---|---|
| N1 | **Lossless CST preserving every byte of trivia, including the exact terminator form** | The strongest guarantee: a formatter becomes a pure function over the tree, and an LSP can do surgical edits. Cost: the CST is larger, and every consumer must handle trivia |
| N2 | **Preserve comments and whitespace, but canonicalise terminator form** (record `\n` internally, remember "this file used CRLF") | Smaller tree, round-trip still possible via a per-file flag. Cost: mixed-terminator files cannot round-trip exactly |
| N3 | **Preserve trivia but not comments** | Rejected: the design report explicitly requires comments to be preserved for the formatter and LSP |
| N4 | **Discard trivia, reconstruct in the formatter** (the legacy approach) | Rejected: it is the behaviour the design report exists to replace, and it destroys user layout |

## Recommendation (PROPOSED, not in force)

**N1.**

* **Q1** — **Yes, lossless.** The CST preserves every byte of trivia, including
  whitespace runs, comment text, and the exact terminator form. This is the
  design report's requirement and it is the only option that makes the formatter
  a pure function and the LSP safe.
* **Q2** — **Normalisation is permitted only for the leading BOM**, which is
  consumed and not represented as trivia. Nothing else is normalised. Rationale:
  any other normalisation silently changes program text (see S13 Q2 for the same
  rule applied to identifiers).
* **Q3** — **Block comments do not nest**, matching the legacy behaviour.
  Rationale: nesting would change the meaning of existing files — a comment
  containing the text `*/` inside another comment currently terminates the outer
  comment, and nesting would silently re-interpret it. If nesting is wanted it
  is a language extension requiring its own record.
  **Proposed diagnostic:** a block comment containing an interior `/*` may be
  worth a *warning*, because it is a common source of confusion, but it must not
  be an error.
* **Q4** — The exact terminator form is preserved as trivia. `\n`, `\r\n` and
  `\r` are distinct trivia tokens. The *line structure* used by the grammar is
  terminator-agnostic (all three terminate a line), so the grammar is unaffected;
  only the formatter and LSP see the difference.
* **Q5** — The formatter contract that follows from N1: **the formatter is a
  pure function of the CST**, and formatting an already-formatted file is a
  no-op (idempotence). Trivia that the formatter chooses to rewrite must be
  rewritten deterministically. The formatter itself is **not** specified by
  Directive 002 — only the guarantee it needs from the CST.

## Compatibility consequence

* N1 is **C0–C3 neutral** for program meaning: trivia has no semantics, so
  preserving it cannot change behaviour. It is purely additive capability.
* Q3 (non-nesting) preserves the legacy reading of existing files. Choosing
  nesting would be a **C1 divergence** for any file with a comment containing
  `*/` or an interior `/*`.
* **Note for the differential harness:** because the legacy tooling discards
  trivia and the new tooling preserves it, a byte-for-byte comparison of
  *source* is meaningless. Comparisons must be made on **behaviour** (program
  output), which is what C2 means. This is already the policy in
  `docs/spec/compatibility-policy.md` §1.

## Open dependencies

* **S15** owns the error-recovery contract, which determines how the CST
  represents malformed regions alongside preserved trivia.
* The formatter's own specification (layout rules, line width, indentation) is
  **not** in Directive 002's scope and requires a later directive.
* Phase 0's `cosmic-source` does not yet preserve terminator form; that is a
  Phase 2 concern for the CST, not a defect in `cosmic-source` (which serves
  line/column mapping, not round-tripping).
