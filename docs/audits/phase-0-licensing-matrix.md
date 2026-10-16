# Phase 0 — Licensing and Provenance Decision Matrix

| | |
|---|---|
| **Date** | 2026-10-16 |
| **Directive** | ENGINEERING DIRECTIVE 001-R1 §7 |
| **Status** | **Proposal for owner decision. No licence is chosen, implied or recommended as a decision by this document.** |
| **Established facts** | Every row tagged **E1** was verified first-hand at `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` |
| **Interpretation** | Every row tagged **E2** is engineering reasoning, not legal advice |
| **Legal questions** | §5 lists the items that require qualified legal review |

---

## 1. Established facts (E1)

| # | Fact | How verified |
|---|---|---|
| F1 | The legacy repository `CosmicLang/cosmcc` carries a root `LICENSE` file containing the full text of the **GNU General Public License, version 3, 29 June 2007** | File read at the reference SHA |
| F2 | No additional terms file was observed (no `COPYING`, no exception/waiver document) | Tree inspection |
| F3 | The Maven POMs declare **no `<licenses>` section**; the licence is expressed only through the root `LICENSE` file | `grep "<licenses>"` over all `pom.xml` → no matches |
| F4 | `TRADEMARK.md` and `SECURITY.md` exist at the legacy root, alongside contribution and code-of-conduct documents | Directory listing |
| F5 | The new repository `CosmicLang/cosmic-lang` contains **no `LICENSE` file** and no licence field in `Cargo.toml` | Repository inspection; `README.md` §License says "Not yet chosen" |
| F6 | The new repository contains **no code, test fixture or documentation text translated or copied from the legacy implementation** | Reviewed the whole diff `main..phase/0-foundation`: 3 Rust crates written from scratch, 5 documentation files written from scratch, workspace configuration. No legacy file path appears as a source; no legacy identifier (`Kof*`, `IRModule`, `SemanticAnalyzer`, …) appears in any new source file |
| F7 | The legacy reference is used **read-only**: an isolated clone at a frozen SHA, for inspection and for executing its own test suite | `docs/audits/phase-0-baseline.md` §1 |
| F8 | The legacy golden corpus (74 cases: `Main.kf` + `expected.txt`) was **executed, not copied**: no fixture was vendored into the new repository | New repository tree contains no `.kf`/`.ptkf`/`.ks` file and no `tests/golden` directory |
| F9 | Phase 0 uses **zero external dependencies**, so no third-party licence is currently implicated by the new code | `Cargo.lock` lists only the three workspace packages |
| F10 | The legacy version is `0.6.5` (`VERSION`), and the tree contains 330 `.kf`, 4 `.ptkf` and 0 `.ks` source files | `wc`/`find` at the reference SHA |

---

## 2. Decision matrix

Options are evaluated against the obligations each would create. "Effort" is
relative engineering effort, not calendar time.

| Criterion | **Option A** — Distribute `cosmcc-rs` under GPL-3.0 | **Option B** — Independent implementation under a permissive licence (e.g. MIT/Apache-2.0) | **Option C** — Independent implementation under a copyleft licence other than GPL-3.0 (e.g. MPL-2.0, LGPL-3.0) | **Option D** — No licence yet (current state) |
|---|---|---|---|---|
| **Legal basis** | Treats `cosmcc-rs` as a derivative/combined work of the legacy compiler and accepts GPL-3.0 for the whole work | Relies on the *idea/expression* distinction: language semantics and observable behaviour are not protectable expression, so a clean reimplementation is an independent work | Same basis as B, with a different outbound grant | Unlicensed content is "all rights reserved" by default |
| **Source-disclosure obligation** | **Yes** — distributing binaries requires making complete corresponding source available under GPL-3.0 | No | Only for files derived from copyleft-licensed sources (none today) | N/A |
| **Copyleft reach** | Whole-program copyleft: any distribution of a linked/combined work must be GPL-3.0 | None | File-level (MPL) or library-level (LGPL) | N/A |
| **Compatibility with the legacy ecosystem** | Maximum: code and fixtures could be reused directly, and a fork/merge path stays open | Behavioural compatibility only; **no code or fixture reuse** | Behavioural compatibility only; no code reuse | None |
| **Effect on adopters** | Restrictive for proprietary tooling and IDE integrations built on the compiler | Most permissive; attracts tooling and embedders | Middle ground; some corporate policies forbid GPL but allow MPL | Legally unusable: nobody may lawfully copy or distribute the repository |
| **Consistency with directive principle 2** ("do not port Java source mechanically to Rust") | Neutral — the principle already forbids mechanical translation, so A's main advantage (reuse) is not being used | **Aligned** | Aligned | Neutral |
| **Consistency with the design report** (rebuild behaviour from evidence; "the asset to preserve is the demonstrable semantics") | Neutral | **Aligned** — the report's whole strategy is behavioural re-derivation | Aligned | Neutral |
| **Residual provenance risk** | Low: obligations are accepted explicitly | **Medium**: rests on the reimplementation being genuinely independent (§3) | Medium: same as B | Low today, but the repository is unusable |
| **Effort** | Lowest (reuse becomes legal) | Highest (every artefact must be re-derived) | Highest | None |
| **Reversibility** | Low — copyleft grants cannot be revoked for released versions | Medium — a permissive grant cannot be revoked for released versions either | Medium | **High** — the decision is still open, which is the point of this document |
| **Blocks Phase 1?** | No | No | No | **Yes, for anything that would copy legacy artefacts** |

**Engineering observation (E2, not a recommendation).** Options A and B are
both internally consistent. The Phase 0 workspace was built under the
assumption that reuse is *not* available: zero legacy code, zero legacy
fixtures, behaviour re-derived from executed evidence. That makes Option B the
cheapest option to *adopt* given the work already done, and Option A the option
that would require the least work *going forward*. The choice is the owner's;
this document deliberately stops there.

---

## 3. Provenance and derivative-work risk register

| # | Risk | Likelihood if Option B/C is chosen | Mitigation already in place | Mitigation still required |
|---|---|---|---|---|
| R1 | A future contributor translates legacy Java code into Rust, creating a derivative work that must be GPL-3.0 | **Medium** — the legacy source is public and familiar | Directive principle 2 forbids mechanical translation; the baseline audit §6 records the rule; ADR-0001 §4.3 restates it | A written contribution policy ("do not paste or translate legacy source"), a pre-commit/review check, and a `CONTRIBUTING` note once a licence exists |
| R2 | Legacy **test fixtures** are copied in, which would drag the corpus under GPL-3.0 | **Medium** — the corpus is the obvious oracle and is convenient to vendor | The corpus is **not** vendored (F8); `docs/spec/compatibility-matrix.md` §5 requires a new test that *encodes* the legacy evidence rather than copying it | Formalise "encode, do not copy": new fixtures written from observed behaviour, with the legacy case name cited as provenance |
| R3 | Legacy **documentation prose** is copied into `docs/` | Medium | All five Phase 0 documents were written from scratch; legacy docs are cited, not reproduced | Same contribution policy as R1 |
| R4 | A dependency is added later that is itself GPL-3.0, forcing the whole work to GPL-3.0 | Low | Zero dependencies today; every new dependency requires a recorded licence note (`docs/development/toolchain-policy.md` §3) | Add an explicit "GPL-incompatible dependency" check to the dependency review step |
| R5 | The repository stays unlicensed and someone assumes a permissive grant | **High while Option D persists** | `README.md` §License states "Not yet chosen … treat all content as 'all rights reserved'" | Decide the licence; until then keep the statement prominent |
| R6 | A published binary inherits the legacy `kof` name or `dev.kof` namespace and creates trademark confusion | Low | ADR-0001 proposes `cosmcc-rs`/`cosmic`; the new executable is `cosmic` | Confirm the naming decision and check `TRADEMARK.md` obligations before the first release |

---

## 4. Legacy fixture and documentation reuse policy (proposed)

| Artefact class | Reuse permitted? | Rule |
|---|---|---|
| Legacy implementation source (Java) | **No** | Never copied, never mechanically translated. Applies regardless of the chosen licence, per directive principle 2 |
| Legacy test fixtures (`.kf`/`.ptkf`/`.ks`, `expected.txt`) | **No** (proposed) | Re-derive as new fixtures whose provenance is the *observed behaviour*, citing the legacy case name and SHA. If the owner ever chooses GPL-3.0, this rule may be revisited |
| Legacy documentation prose | **No** (proposed) | Cite and summarise; do not reproduce |
| Legacy build configuration (`pom.xml`, CI) | **No** | Independent Cargo workspace |
| Legacy **facts** (module list, counts, file extensions, version strings) | **Yes** | Facts are not protectable expression; they are recorded in the audits with the SHA as provenance |
| Legacy **executed behaviour** (golden corpus results) | **Yes** | Behaviour is the compatibility contract; recorded as evidence, not vendored as data |

---

## 5. Legal questions requiring qualified review

These are explicitly **not** answered here.

1. **Scope of derivative work.** Does an independent Rust reimplementation of
   the *same language*, written from behavioural evidence and documentation
   without access to implementation source, constitute a derivative work of
   the GPL-3.0 compiler? (Engineering position: no, per the idea/expression
   distinction. This is the single assumption Option B rests on and it needs
   legal confirmation.)
2. **Corpus as a derivative.** If any part of the legacy golden corpus were
   ever vendored, would that make the new test suite a derivative work of the
   GPL-3.0 repository?
3. **Language-name and CLI-name compatibility.** Does providing an
   independently implemented compiler that accepts `.kf`/`.kof`/`.ptkf`/`.ks`
   inputs and is invoked as a compatible `cosmic` CLI raise a trademark or
   unfair-competition question beyond `TRADEMARK.md`? Is `Kof` a mark that
   constrains naming?
4. **Trademark obligations.** `TRADEMARK.md` exists in the legacy repository;
   its terms have not been analysed here.
5. **Contributor provenance.** What representation should contributors make
   (e.g. a DCO or a CLA) to keep the independence claim credible, and what
   record should be kept of the review that enforced it?
6. **Outbound licence choice interactions.** If the owner selects a
   GPL-3.0-incompatible outbound licence, what is the effect on the ability to
   use GPL-3.0 tooling (e.g. legacy build scripts) during differential testing?
7. **Patent exposure.** The legacy repository carries no patent grant beyond
   what GPL-3.0 §11 implies. Is a patent non-assertion grant advisable in the
   new outbound licence?
8. **Redistribution of legacy documentation in CI logs.** If differential
   testing runs the legacy compiler in CI and publishes its output, does that
   constitute distribution of GPL-covered material?

---

## 6. Current state and what this document does not do

* **No licence is chosen.** No `LICENSE` file was added, no licence field was
  added to `Cargo.toml`, and no licence text appears anywhere in the new
  repository.
* **No GPL-covered code or fixture was imported.** Verified by reviewing the
  entire diff (F6, F8).
* **Nothing in Phase 0 required the licence decision.** The three crates are
  original work, and the compatibility evidence is behavioural.
* **Phase 1 does not require it either**, unless the owner decides to allow
  legacy artefact reuse. The specification work in Directive 002 can proceed
  under the current "all rights reserved" state.
* **Phase 5 (releases) does require it.** The repository cannot be published or
  distributed in any form until the owner decides.
