# S13 — Unicode Semantics

| | |
|---|---|
| **Status** | **ACCEPTED NORMATIVE RULE** for the lexical decisions Q1 (D-L3), Q4 (D-L2), Q5 (D-L9) and Q6 (D-L1) — owner acceptance 003-G1 (2026-10-16, G1-L); **VERIFIED LEGACY BEHAVIOR** for the executed probes; Q3 remains a deferred proposal (UNI namespace, off by default); M-7 classification cases and the strict-rule corpus impact remain measurement obligations |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; Phase 2 implementation |
| **Related** | `docs/spec/language-spec.md` §1.3/§2.2, `docs/spec/typing-contracts.md` §1.2, S8 |

> **R1 status interpretation:** Evidence claims below are VERIFIED LEGACY BEHAVIOR only; any proposed semantics remain PROPOSED NORMATIVE RULE pending owner action. Older recommendations predate the experiments and may describe the prior evidence gap.

## Question

1. **Q1** — What is the exact **identifier character class**?
2. **Q2** — Is there any **normalisation** (NFC/NFD) of identifiers or literals?
3. **Q3** — Are **confusable** identifiers a diagnostic concern?
4. **Q4** — **Column counting**: the legacy lexer counts UTF-16 code units while
   the new foundation counts Unicode scalar values. Which is normative?
5. **Q5** — How are **lone surrogates** from `\uXXXX` escapes handled?
6. **Q6** — What is the **source encoding** contract, beyond UTF-8 plus BOM
   tolerance?

## Evidence

* **E1 — Q4, the material finding.** The legacy lexer advances its column
  counter once per **UTF-16 code unit**: `advance()` increments `column` by one
  and `source.charAt(pos)` addresses code units
  (`parser/Lexer.java`). An astral character (for example an emoji) therefore
  occupies **two** legacy columns.
  The Phase 0 foundation counts columns in Unicode **scalar values**
  (`crates/cosmic-source/src/file.rs`, documented and tested). The two
  therefore **disagree from the first astral character on a line**.
  This divergence is already recorded in `docs/spec/language-spec.md` §1.3 and
  is a direct consequence of a deliberate Phase 0 decision, not a bug in either
  implementation.
* **E1** — Identifiers start with a Unicode letter, `_` or `$`, and continue
  with Unicode letters/digits, `_` or `$`
  (`parser/Lexer.java` `tokenize`, `readIdentifier`). The predicate is
  Unicode-aware, so non-ASCII letters are legal identifier characters.
* **E1** — The corpus exercises **ASCII identifiers only**; the only non-ASCII
  content in the corpus is inside string literals (for example the hello-world
  text). So Q1's non-ASCII half is **unevidenced**.
* **E1** — `\uXXXX` is a single UTF-16 code unit
  (`parser/Lexer.java` `readUnicodeEscape` returns a `char`). A `\u` escape
  naming a high surrogate is therefore accepted as a standalone value with **no
  diagnostic**, producing an ill-formed scalar sequence when the resulting
  string is used.
* **E1** — There is **no** normalisation anywhere in the legacy lexer: it copies
  characters verbatim. So `é` (U+00E9) and `e` + U+0301 are distinct
  identifiers and distinct string contents.
* **E1** — A leading UTF-8 BOM is tolerated and is **not** a code character
  (`parser/Lexer.java`, comment `OBS-008`).
* **E1** — Phase 0 already rejects invalid UTF-8 at load time and reports the
  first invalid byte offset (`cosmic-source::SourceMap::load`), verified in
  `docs/audits/phase-0-source-review.md`.

## Options

| # | Option | Assessment |
|---|---|---|
| M1 | **Keep legacy code-unit columns** (revert the Phase 0 decision) | Maximum diagnostic-position compatibility. Cost: it makes column numbers wrong for editors, which display scalar values or grapheme clusters, and it would undo a tested Phase 0 behaviour |
| M2 | **Keep scalar-value columns** (Phase 0 behaviour) and record the divergence explicitly | Editors agree; diagnostics from the new toolchain differ from the legacy ones only on lines containing astral characters. Requires differential tests to normalise for it |
| M3 | **Count grapheme clusters** (display columns) | Best match to what a user sees, but it needs Unicode segmentation tables, which the Phase 0 zero-dependency policy forbids and which are not stabilised in the Rust standard library |
| M4 | **Add normalisation** (NFC) to identifiers | Prevents a class of confusable-identifier bugs, but silently changes program text, which contradicts the "no silent normalisation" rule |

## Recommendation (PROPOSED, not in force)

**M2, with Q2 answered as "no normalisation" and Q3 answered as "diagnose, do
not normalise".**

* **Q1** — Identifiers are Unicode **letters** and **digits** plus `_` and `$`,
  with the same start restriction as the legacy lexer. Because the non-ASCII
  half is unevidenced, this is recorded as **PROPOSED** and Phase 2 must add
  cases with non-ASCII identifiers before it is treated as settled.
* **Q2** — **No normalisation.** Source text is used verbatim; a decomposed and
  a precomposed spelling are different identifiers. Rationale: normalising would
  silently change the meaning of program text, and the project forbids silent
  reinterpretation.
* **Q3** — **Do not normalise; do diagnose.** A diagnostic is emitted when an
  identifier contains characters that are visually confusable with ASCII or with
  another identifier in the same scope. This is a **warning**, not an error, and
  is a **new diagnostic** with no legacy counterpart — so it must not change the
  meaning of any existing program. Rationale: confusable identifiers are a
  security concern, and the design report's supply-chain section treats
  adversarial input as in scope; a warning is the intervention that does not
  break compatibility.
* **Q4** — **Scalar values are normative (M2).** The Phase 0 decision stands. The
  legacy divergence is recorded, is **C0/C1-neutral** (it affects diagnostic
  positions, not acceptance) and is **C2-relevant only for tooling that compares
  diagnostic text**. Differential tests must normalise positions or restrict to
  the Basic Multilingual Plane. Explicitly: the new toolchain does **not** adopt
  the legacy code-unit counting.
* **Q5** — A `\uXXXX` escape naming a **lone surrogate is an error**. The legacy
  lexer accepts it silently and produces ill-formed text; that is the same
  silent-degradation class the project forbids (compare the triple-quote
  history in S8). This is a **PROPOSED tightening** and a C2 divergence for
  sources that rely on it — expected to be zero, but to be measured.
* **Q6** — The encoding contract is UTF-8 with a single optional leading BOM
  which is not a code character. Invalid UTF-8 is rejected at load with the byte
  offset of the first invalid byte. Already implemented and tested in Phase 0.

## Compatibility consequence

* **Q4 is an intentional, documented divergence.** It must appear in the
  "known differences" list that Phase 2's differential harness consults, or the
  harness will report false failures on any astral-containing case.
* **Q3 (confusable warning)** adds diagnostics. It must be off by default in any
  comparison against legacy output, and it must never be an error.
* **Q5** is a tightening; measure it against the corpus.

## Open dependencies

* **S8** shares the escape/surrogate handling; Q5 must be implemented once, in
  the lexer, and referenced from both records.
* Phase 2 must add non-ASCII identifier cases before Q1 can be treated as
  settled.
* The differential harness design (D8) must include a position-normalisation
  mode; this record is a hard requirement on it.

---

## Experimental evidence (Directive 002-R1 §4)

Executed read-only against the frozen legacy compiler on 2026-10-16,
`--target jvm`, OpenJDK 27. Full commands, programs, stdout, stderr and exit
statuses are in `docs/audits/phase-1-legacy-probes.md`.

The probe **resolved Q1's non-ASCII half and measured Q4**.

| Question | Result | Observed |
|---|---|---|
| Q1 non-ASCII letters | **valid identifier characters** | `Int café(Int x) = x + 1` compiled; `café(41)` printed `42` |
| Q1 `_` and `$` starts | **valid** | `_under(1)` → `2`; `$dollar(1)` → `2` |
| Q4 column counting | **UTF-16 code units, measured** | see below |

**Q4 is now quantified, not merely asserted.** For the source line
`main() { println("😀" +) }` the legacy compiler reported the `)` at **column
24**. Counting in UTF-16 code units, the emoji occupies columns 19–20 and `)`
falls at 24. Counting in **Unicode scalar values**, `)` would be at **23**.

**The divergence is therefore exactly +1 per astral character preceding the
position on that line.** The Phase 1 record asserted this from source; the probe
measures it. The proposed rule (scalar values) is unchanged, and the differential
harness must normalise diagnostic columns on any line containing an astral
character.

**Still OPEN:** identifier classes beyond letters/`_`/`$` (combining marks,
digit-leading names), and lone-surrogate escapes.

---

## 003-G1 update (2026-10-16)

The owner accepted the G1-L lexical contract (ENGINEERING DIRECTIVE 003-G1 §3),
which decides this record's Q1, Q4, Q5 and Q6 at individual-rule granularity:

* **Q1 → D-L3 — accepted as proposed.** Identifier start is a Unicode letter,
  `_` or `$`; continuation adds Unicode digits; no normalisation; `$` retained.
  The classification-authority caveat stands: the Rust standard-library
  predicates are accepted, not verified, and the M-7 differential cases remain
  owed before the rule is called settled.
* **Q4 → D-L2 — accepted as proposed.** Byte spans are authoritative; columns
  are scalar-value counts; the legacy UTF-16 divergence is declared and must be
  normalised by the differential harness. The corrected tab-column example is
  preserved.
* **Q5 → D-L9 — accepted as proposed.** A lone-surrogate escape is an error
  (LEX010); the corpus impact must still be measured.
* **Q6 → D-L1 — accepted as proposed.** UTF-8 only; invalid bytes rejected at
  load with the first invalid byte offset; one leading BOM consumed as a code
  character and preserved as a three-byte prefix leaf contributing no language
  token. The input boundary is preserved: no token stream or CST is owed for a
  file that fails UTF-8 validation.
* **Unchanged.** Q2's no-normalisation position is retained within the accepted
  D-L3 contract (package item 3); Q3's confusable-identifier warning remains a
  deferred proposal in the `UNI` namespace, off by default.

No other status changes.
