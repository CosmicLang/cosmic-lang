# Test Strategy

| | |
|---|---|
| **Status** | **Proposed — Phase 1 draft.** |
| **Phase** | 1 (design); implemented from Phase 2 |
| **Directive** | ENGINEERING DIRECTIVE 002, deliverable D8 |
| **Related** | `docs/development/measurement-plan.md` (D7), `docs/spec/diagnostic-codes.md` (D9), `docs/spec/decisions/S15-feature-gates.md`, `docs/audits/phase-0-licensing-matrix.md` §4, `docs/audits/phase-0-source-review.md` §6 |

---

## 1. Principles

1. **Evidence over counts.** A test count is not evidence of correctness. This
   is not a slogan here: the Phase 0 source review found two real defects — a
   CRLF column inconsistency and an allocation-limit bypass — in a crate whose
   tests all passed. The design report makes the same point on page 11.
2. **Tests encode behaviour, they do not import it.** Legacy fixtures are **not**
   copied. A new test records the *observed behaviour* with the legacy case name
   and SHA as provenance (`docs/audits/phase-0-licensing-matrix.md` §4).
3. **A skip is not a pass.** Every skipped case carries a stated reason, and
   skipped cases are reported separately. The legacy corpus already does this
   (`222 passed, 0 failed, 0 skipped` for jvm/js/script; native explicitly
   NOT EXECUTED) and that reporting shape is adopted.
4. **A test that cannot fail is not a test.** Every layer below must be able to
   demonstrate a *failing* run before it is trusted. The Phase 0 review's
   precedent: the oversized-load test was written so that removing the fix makes
   it fail.
5. **No test may be dangerous.** A test whose failure mode is a 5 GiB
   allocation is `#[ignore]`d and documented as a manual check, as Phase 0
   already does. Tests must be safe to run on a laptop and in CI.
6. **Determinism.** Identical inputs produce identical outputs. Anything
   order-dependent (hash iteration, parallel scheduling) must be made
   deterministic in the test, not worked around.

---

## 2. The test ladder

Each layer answers a question the layers below cannot.

| # | Layer | Answers | Introduced in |
|---|---|---|---|
| L0 | **Unit** | Does this function do what its contract says? | Phase 0 (exists) |
| L1 | **Exact-output** | Does the rendered artifact match byte-for-byte? | Phase 0 (exists) |
| L2 | **Golden diagnostics** | Does the *diagnostic* match byte-for-byte for a malformed input? | Phase 1 (design) / Phase 2 |
| L3 | **Property-based** | Does the invariant hold for all inputs, not just the chosen ones? | Phase 2 |
| L4 | **Differential** | Does the new toolchain behave like the legacy one, where it should? | Phase 2 |
| L5 | **Fuzz** | Does the toolchain terminate on arbitrary bytes? | Phase 2 |
| L6 | **Regression** | Does a fixed defect stay fixed? | Continuous |
| L7 | **Conformance matrix** | Does each feature work on each target? | Phase 4 |

---

## 3. Layer specifications

### L0 — Unit tests

* **Scope:** crate-internal behaviour, exercised through the public API where
  possible.
* **Phase 0 state:** present and effective. `cosmic-source` has 28 integration
  tests plus 2 unit tests; `cosmic-diagnostics` has 8 unit tests plus 11
  integration tests; `cosmic-cli` has 10 integration tests. Verified counts in
  `docs/audits/phase-1-specification-report.md`.
* **Rule:** a unit test asserts **one** contract. A test asserting several
  independent facts makes a failure ambiguous.

### L1 — Exact-output tests

* **Scope:** renderers, formatters, and anything whose output is a contract.
* **Phase 0 state:** present. Both diagnostic renderers are pinned by
  exact-string assertions, including escaping, gutter width, tab normalisation
  and multi-line spans.
* **Known gap (from the Phase 0 review, G2):** exact-string assertions pin the
  bytes but do not prove *parseability*. The review verified JSON validity
  **manually** with an independent parser. **Action:** when the dependency
  policy permits, add a dev-dependency parser and assert round-trip
  parseability as part of L1. Until then, the manual check is recorded as
  evidence rather than implied.

### L2 — Golden diagnostics

* **Scope:** for a corpus of malformed inputs, assert the **complete diagnostic
  output** byte-for-byte, in both human and machine formats.
* **Why a separate layer:** the design report (page 11) lists "golden
  diagnostics" explicitly, and it is a different claim from L1: L1 tests a
  renderer with a hand-built diagnostic; L2 tests that the *frontend produces
  the right diagnostic* for a given source.
* **Design:** one directory per case containing the input and the expected
  diagnostic output; a runner that reports `passed / failed / skipped`.
* **Caveat:** golden diagnostics are brittle under intentional message changes.
  Mitigation: the expected file is regenerated only by an explicit command, and
  a change to an expected file requires a stated reason in the commit message.

### L3 — Property-based tests

* **Scope:** invariants that must hold for all inputs. Examples already known to
  be worth testing:
  * `Span` invariants: every span produced by a checked constructor is ordered,
    in bounds, and on a character boundary.
  * `line_col` is `Some` **exactly** at character boundaries.
  * A column never exceeds the printed line length plus one. **This invariant
    was violated before the Phase 0 CRLF fix** and is now enforced by an
    exhaustive test; it is a natural property to generalise to arbitrary text.
  * Rendering is total: never panics, for arbitrary diagnostics and arbitrary
    source maps.
  * JSON output is always parseable, for arbitrary message content.
* **Rule:** a property test must be able to shrink a counterexample to a minimal
  one; otherwise the failure is not actionable.

### L4 — Differential tests

* **Scope:** run the same program through the legacy compiler and the new
  toolchain and compare **observable behaviour**.
* **What is compared:** program stdout, exit status, and (for error cases) the
  diagnostic *code and position*, not the message text.
* **What is NOT compared:** source bytes, artifact bytes, formatting, or
  diagnostic message wording. C3 is not promised
  (`docs/spec/compatibility-policy.md` §1).
* **Mandatory normalisations.** The harness must support these, because without
  them it will report false failures:
  | Normalisation | Reason |
  |---|---|
  | Diagnostic column on lines containing astral characters | The legacy lexer counts UTF-16 code units, the new toolchain counts scalar values (S13 Q4). This is a **documented, intentional divergence** |
  | Diagnostic message text | Messages are not part of the compatibility contract |
  | Trivia and formatting | The legacy tooling discards trivia; the new tooling preserves it (S14) |
* **The known-divergence list.** The harness must consult an explicit list of
  intentional divergences and report a hit as "expected divergence", not as a
  failure. The list is populated from the decision records: nested generics
  (S1), nullability conversions (S2), exhaustiveness (S5), integer-literal
  promotion and unknown escapes (S7/S8), confusable-identifier warnings (S13),
  and the CRLF/astral column rule (S13).
* **Legacy defect handling.** Where the legacy compiler is wrong (nested
  generics, JS loop/return miscompiles, cross-target shift semantics), a
  divergence is **expected** and must be recorded as a legacy defect, not
  treated as a regression. This is the "legacy defects are not part of the
  contract" policy (`compatibility-policy.md` §3 rule 6).
* **Cost note:** the legacy compiler must be **executed** for this layer, which
  means a host with the legacy toolchain. That is a CI cost and a licensing
  consideration; it is analysed in §5.

### L5 — Fuzz

* **Scope:** the lexer and parser, per the design report's page 5 criterion
  ("must recover multiple errors, never enter an infinite loop, and pass
  structured fuzzing").
* **What it checks:** (a) **termination** on arbitrary bytes; (b) **no panic**;
  (c) the error count is bounded (no diagnostic cascade).
* **Corpus:** a seed corpus of **newly authored** valid and invalid inputs,
  plus structure-aware mutation. **No legacy fixture is used as a seed.**
* **Runtime budget:** a fuzz target must have a stated time budget so CI can run
  a short pass and a nightly job can run a long one.
* **Phase 0 relevance:** the `cosmic-source` crate is already total on arbitrary
  text (verified by an exhaustive offset scan across ten pathological inputs),
  so the first fuzz targets should cover it and the diagnostics renderers before
  the lexer exists.

### L6 — Regression tests

* **Rule:** every fixed defect gets a test that **fails without the fix**. Two
  Phase 0 examples set the standard:
  * `line_terminator_columns_agree_across_lf_crlf_and_lone_cr` and
    `every_offset_maps_inside_its_printed_line` for the CRLF column defect;
  * `declared_length_limit_is_checked_before_reading` plus an `#[ignore]`d
    end-to-end test for the allocation-limit defect.
* **Rule:** the commit message names the test and states the before/after
  measurement where one exists.

### L7 — Conformance matrix

* **Scope:** feature × target → pass / fail / **skipped with a reason**.
* **Rule:** a feature is claimed supported on a target only when the matrix has
  a passing entry. `unspecified` (per S15) is a distinct state from `refused`.
* **Precedent:** the legacy corpus's reporting shape (222/222 with native
  NOT EXECUTED and a stated platform reason) is adopted verbatim.

---

## 4. Coverage gaps carried from Phase 0

The Phase 0 source review recorded three gaps. Their disposition:

| # | Gap | Disposition |
|---|---|---|
| G1 | The `cosmic` **binary** is never executed by a test: `tests/cli.rs` exercises `cosmic_cli::run` in-process, so the `main` wiring (stream locking, flush, `process::exit`) is untested | **Close in Phase 2**, when the CLI gains a real command. Add one integration test using `env!("CARGO_BIN_EXE_cosmic")`. Not closed in Phase 1 because Phase 1 is specification-only and must not change code |
| G2 | JSON output is pinned by exact strings only, never parsed | **Close when the dependency policy permits a parser dev-dependency.** Until then the manual independent-parser check is recorded as evidence (`docs/audits/phase-1-specification-report.md`) |
| G3 | `examples/demo.rs` is compiled but its output is not asserted | **Accept** for now; assert its output when the CLI can print diagnostics |

---

## 5. The differential harness and the licence

L4 requires **executing** the legacy compiler. That raises a licensing question
that must be answered before the harness is built:

| Question | Analysis |
|---|---|
| Does executing the legacy compiler create a derivative work? | Engineering position: **no**. Running a program is not copying it. The legacy compiler is used as a black box, and only its **output** is observed |
| Does publishing the output create a distribution? | Engineering position: the output of a compiler on a test input is not the compiler. But this is a **legal question** and is listed in `docs/audits/phase-0-licensing-matrix.md` §5 (Q8) |
| Do legacy fixtures need to be present? | **No.** The new corpus is authored separately. The legacy compiler is pointed at *new* inputs; legacy fixtures are never copied or vendored |
| Is the legacy compiler committed or vendored? | **No.** It is cloned read-only at a frozen SHA into an isolated directory at run time, exactly as Phase 0's baseline audit did |
| Is this a blocker? | **No** for Phase 2's frontend work. It is a blocker for **closing** the differential layer, and therefore for the compatibility claims in `compatibility-policy.md` |

**Recommendation:** build the harness so that the legacy toolchain is an
**optional external dependency** that the test runner discovers and skips with a
stated reason when absent — the same skip-with-reason discipline as L7. That way
the harness exists, the new corpus is developed, and the legacy comparison is
enabled wherever a host can provide it, without making the licence question a
prerequisite for progress.

---

## 6. What this strategy does not do

* **It does not import legacy fixtures.** No `.kf`/`.ptkf`/`.ks` file and no
  `expected.txt` is copied into this repository.
* **It does not set coverage gates.** See `measurement-plan.md` M2.
* **It does not implement any layer.** Directive 002 is specification-only; this
  document designs the layers, and Phase 2 implements them.
* **It does not promise mutation testing.** The design report mentions
  "selective mutation tests"; that is recorded as an aspirational layer with no
  tool selected and no dependency decision taken. It is **not** part of the
  ladder above, because a layer that cannot be run is not a layer.
