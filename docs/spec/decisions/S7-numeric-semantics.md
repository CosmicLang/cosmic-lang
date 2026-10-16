# S7 — Numeric Semantics

| | |
|---|---|

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

| **Status** | **OPEN** (one sub-question is **PROPOSED**) |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; per-target validation in Phase 4 |
| **Related** | `docs/spec/language-spec.md` §2.4, `docs/spec/typing-contracts.md` §7, S1 |

## Question

1. **Q1** — What is the **overflow** rule for `Int` and `Long` arithmetic:
   wrap, trap, or saturate?
2. **Q2** — Is the legacy **implicit int → long promotion** of an out-of-range
   decimal literal kept?
3. **Q3** — What is the **division** rule for signed integers, and what happens
   on division by zero?
4. **Q4** — What is the exact **float/double formatting** contract?
5. **Q5** — Which numeric conversions are implicit versus explicit?
6. **Q6** — Should a **bitwise complement operator** (`~`) be added? The legacy
   language has none.

## Evidence

* **E1 (executed)** — Integer division **truncates toward zero**:
  `tests/golden/signed-division-overflow/expected.txt` contains the results
  `-3`, `-1`, `-3`, `1`, `0`, `-1`, `-4`, `-2`, `2`, `-2`, `-1` for the
  division/modulo expressions in that case. Executed on jvm/js/script,
  222/222.
* **E1 (executed)** — `Int` arithmetic **wraps** at 32 bits: the same case
  yields `-2147483648`. `Long` arithmetic wraps at 64 bits: it yields
  `-4500000000`, `-36000000000`, `-8999999999`. So overflow is two's-complement
  wrapping at both widths, not trapping and not saturating.
* **E1 (executed)** — Mixed-precision arithmetic yields values consistent with
  evaluation at the wider type (`-1285714285`, `-5`).
* **E1 (executed)** — Float→int conversion **truncates**: in
  `tests/golden/numeric-casts/expected.txt` the results are `3`, `3`, `1`, `14`,
  `20`, `7.5`, `2.5`, i.e. `7.5` and `2.5` do not round up.
* **E1 (executed)** — The float/double formatting contract is **specific and
  non-default**. `tests/golden/double-formatting/expected.txt` contains:
  * whole values print with a trailing `.0` — `1.0`, `1000.0`, `6.0`;
  * full precision is printed — `0.3333333333333333` (17 significant digits);
  * binary artefacts are **not** hidden — `0.30000000000000004`;
  * positive infinity prints as the literal token **`Infinity`** (capital `I`,
    no sign), and `-0.5` prints with a leading minus.
  These are observed outputs, i.e. the compatibility contract for formatting.
* **E1** — The legacy lexer **promotes** a decimal literal that does not fit a
  32-bit signed integer to a long literal rather than reporting an error
  (`parser/Lexer.java` `readNumber` + `isIntegerLiteral`).
* **E1** — The legacy token set has **no bitwise complement operator**: `~` is
  not a token and falls to the unexpected-character branch (`LEX005`).
* **E1** — The corpus also exercises `bitwise-ops`, `byte-arith`,
  `long-arithmetic`, `math-namespace`, `math-pow-constants` and
  `math-parse-namespace`, so the numeric surface is broadly covered by executed
  tests — with the caveat below.
* **E1** — The legacy ledger records a **measured cross-target divergence** in
  shift semantics: on riscv64/aarch64 and JS, `Int` shifts did not match the JVM
  (`cosmcc:docs/status.md` §698 at the frozen SHA). The golden corpus executes on
  jvm/js/script, and that divergence was found *after* those cases passed, which
  shows the corpus does **not** fully pin numeric semantics per target.

## Options

| # | Option | Assessment |
|---|---|---|
| G1 | **Keep wrapping arithmetic** (two's complement) | Matches the executed evidence exactly. Cost: keeps a classic source of silent bugs |
| G2 | **Trap or saturate on overflow** | Safer, but a **C2 divergence** from every executed case in `signed-division-overflow`, which the project treats as the compatibility contract |
| G3 | **Make overflow a compile-time error where statically provable, and keep wrapping at runtime** | Hybrid: catches constant-foldable cases without diverging at runtime. Adds a diagnostic surface that has no legacy counterpart |
| G4 | **Defer** | Leaves the numeric contract to Phase 2, where the type checker must already know the result types |

## Recommendation (PROPOSED, not in force)

* **Q1** — **Keep wrapping** (G1). The executed evidence is unambiguous and
  wrapping is the compatibility contract. Rationale: this project's rule is that
  the legacy behaviour is the contract unless a decision record says otherwise;
  changing the most-exercised numeric behaviour in the language would invalidate
  the corpus as an oracle. **Option G3 is attractive but must not be adopted in
  Phase 1** — it introduces diagnostics with no legacy counterpart and would
  make the corpus ambiguous as an oracle. It may be proposed later, on its own
  record, with a corpus impact analysis.
* **Q2** — **Remove the silent promotion.** This is a **PROPOSED divergence**:
  a decimal literal that does not fit its target type should be an error, not a
  silent widening. Rationale: it is a silent reinterpretation of the program's
  intent, and it is the same class of behaviour the project's rules forbid
  (`docs/spec/language-spec.md` §9). Consequence: it is a **C2 divergence** for
  any legacy program relying on it. The divergence must be measured against the
  corpus before adoption.
* **Q3** — Division **truncates toward zero**, matching the executed evidence.
  Division by zero is **OPEN**: no case exercises it, and the correct behaviour
  (trap, `Infinity`, or an exception) is not evidenced. Phase 2 must test it.
* **Q4** — The formatting contract is the observed one (Q1 evidence above),
  including `Infinity` as the printed spelling and the trailing `.0`. It must be
  specified as a **per-target conformance requirement**, because formatting is
  exactly where cross-target divergence hides.
* **Q5** — Only the conversions evidenced in `numeric-casts` are implicit;
  everything narrowing is explicit. Any widening rule must be listed before
  Phase 2 implements it (`typing-contracts.md` §7).
* **Q6** — **Do not add `~`.** No evidence, and adding an operator is a language
  extension, not a specification of the existing language. If wanted, it needs
  its own record and an owner decision.

## Compatibility consequence

* Q1 (wrapping) is **C2-compatible** with the corpus.
* Q2 (no silent promotion) is a **C2 divergence** and must be measured.
* Q3 division-by-zero, once decided, must be checked for **per-target
  divergence** before being declared normative.
* Q4 must be enforced by the Phase 4 differential matrix; the shift-semantics
  precedent shows that passing the golden corpus is **not** sufficient evidence
  of per-target numeric conformance.

## Open dependencies

* Division-by-zero behaviour requires new evidence (Phase 2).
* The corpus impact of Q2 requires measurement against the 74 cases.
* S6 owns whether numeric divergence between targets is permitted to be
  observable.
