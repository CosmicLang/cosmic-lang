# Licensing Recommendation — Phase 1

| | |
|---|---|
| **Status** | **Recommendation only. No licence is chosen, and none may be chosen by engineering.** |
| **Phase** | 1 |
| **Directive** | ENGINEERING DIRECTIVE 002 §6 (and §9 gate G1d) |
| **Predecessor** | `docs/audits/phase-0-licensing-matrix.md` (the options and the fact base) |
| **Rule respected** | The legacy project is GPL-3.0; the new repository's outbound licence is **undecided** and stays undecided until the owner decides |

---

## 1. What changed since the Phase 0 matrix

The Phase 0 matrix laid out four options (A: GPL-3.0, B: permissive independent
implementation, C: other copyleft, D: no licence) and separated established
facts from engineering inference and from legal questions. It reached no
recommendation, deliberately.

Phase 1 adds **three new pieces of evidence** that make a recommendation
possible without deciding anything:

| # | New evidence | Source |
|---|---|---|
| N1 | **The proposed specification was independently authored after legacy inspection; provenance still requires review.** `docs/spec/language-spec.md`, `grammar.ebnf`, `typing-contracts.md`, the 18 decision records and the ADRs were written from **observed behaviour and source facts**, with no legacy artefact reproduced. The "independent implementation" claim in option B is therefore no longer hypothetical for the *documentation* half of the project | This phase |
| N2 | **The differential test strategy deliberately does not vendor legacy fixtures.** The harness is designed to use the legacy compiler as a **black box** on newly authored inputs, with the legacy toolchain as an optional external dependency that skips with a stated reason when absent | `docs/development/test-strategy.md` §3 L4, §5 |
| N3 | **A legal question about executing the legacy compiler was identified and is now explicit**, rather than being discovered later during Phase 2 | `docs/development/test-strategy.md` §5 |

N2 is the important one: it means **option B does not require any legacy artefact
in this repository at any point**, including for compatibility testing. That
removes the main practical pressure that would otherwise push toward option A.

---

## 2. Recommendation

**Engineering recommends Option B — an independent implementation under a
permissive licence — with the licence text itself chosen by the owner.**

This is a **recommendation**, not a decision. The owner may choose A or C; §4
states what changes if they do.

### 2.1 Why B

| Reason | Basis |
|---|---|
| **The work already done is option-B-shaped.** Three Rust crates, a full specification, a grammar, 18 decision records and five ADRs contain no legacy code, no legacy fixture and no legacy prose. Option A's main advantage — the legal ability to reuse — has not been used and is not needed | E1: Phase 0 licensing matrix F6/F8, extended by N1 |
| **Compatibility is achievable behaviourally.** The contract is the language's observable behaviour, verified through the legacy corpus used as an oracle. Nothing in that method requires copying | N2 |
| **The design report's own strategy is behavioural.** "The main asset to preserve is the language's demonstrable semantics" (page 20). Option B is the licensing expression of that strategy; option A would be the expression of a *code-reuse* strategy the report does not propose | Design report page 20, and its directive principle 2 ("do not port Java source mechanically to Rust") |
| **It maximises adopter reach.** A compiler for a new language benefits more from tooling, IDE and embedding adoption than from copyleft reach. Permissive licensing is what makes an editor extension, a language server, or an embedder viable without a licence audit | Engineering judgement (E2), stated as such |
| **It keeps the door open on compatibility scope.** S3 (PortuKof) and S4 (KofScript) are **OPEN (owner scope choice)** precisely because they might require legacy-derived vocabulary tables. Under option B, deriving those tables from *observed behaviour* remains the only route — which is a constraint the owner should know about before choosing | S3 §Evidence, S4 §Evidence |

### 2.2 The specific licence is the owner's choice

Engineering does not name a licence. The relevant properties to weigh:

| Property | Why it matters here |
|---|---|
| Patent grant | A compiler invites patent exposure; a permissive licence with an express patent grant is stronger than one without |
| Attribution requirement | The project already needs an attribution policy for its own documentation (see §3) |
| Compatibility with the *toolchain's* dependencies | Phase 0 has **zero** dependencies, so nothing constrains the choice today — but the measurement and test plans propose tooling (`cargo llvm-cov`, a JSON parser dev-dependency, a complexity analyser) whose licences must be checked against the chosen outbound licence **before** they are added |
| Compatibility with the legacy GPL-3.0 | If any legacy artefact is ever imported, a GPL-incompatible outbound licence becomes impossible. This is the one hard constraint, and N2 is what keeps it from binding |

### 2.3 The one action engineering recommends unconditionally

**Add a contribution policy that forbids importing legacy artefacts.** The
Phase 0 matrix listed this as mitigation R1 and it is still missing. Concretely:

1. A `CONTRIBUTING` note stating: **do not copy or mechanically translate legacy
   `cosmcc` implementation code, documentation prose, or test fixtures.**
2. A review checklist item for every pull request touching source or tests.
3. The "encode, do not copy" rule already in
   `docs/audits/phase-0-licensing-matrix.md` §4, restated where contributors
   will see it.

**This does not require the licence decision to be made first**, and it protects
whichever option the owner later chooses. It is the single highest-value action
in this document.

---

## 3. Attribution requirements (actionable now)

| Artefact | Attribution obligation |
|---|---|
| Legacy **facts** used in this repository (module lists, counts, file extensions, version strings, code families) | Facts are not protectable. They are already cited by path and SHA throughout `docs/audits/` and `docs/spec/`. **Keep citing them** — it is the project's evidence discipline, not a legal requirement |
| Legacy **observed behaviour** (corpus results) | Cited by case name and SHA. Recorded as evidence, never vendored |
| Legacy **code, documentation prose, fixtures** | **Not used, not attributed, not permitted.** Importing them would add legal obligations, but a legal determination of existing obligations is still pending, and §2.3 forbids it |
| Third-party **Rust dependencies** | Not applicable today (zero dependencies). When the first dependency is added, its licence must be recorded — the toolchain policy §3 already requires a licence note per dependency |
| The **project's own** name and identity | Naming is ADR-0001/ADR-0003's concern, and trademark is a separate legal question (see §5) |

**Whether attribution notices are legally required needs qualified review; the repository currently contains no vendored legacy code**,
because nothing from the legacy project is present in it. This is worth stating
explicitly: the absence of an attribution file is not an omission.

---

## 4. What changes if the owner chooses differently

| If the owner chooses… | Then… |
|---|---|
| **Option A (GPL-3.0)** | Certain legacy materials **may become reusable if legal rights and GPL conditions permit**, which would make S3 (PortuKof) and S4 (KofScript) substantially cheaper — their vocabulary tables and wrapping semantics could be derived directly instead of reverse-engineered. The specification work already done remains valid; nothing would need to be redone. **But** the project would then be copyleft, and the permissive-adoption argument in §2.1 is lost |
| **Option C (another copyleft)** | The same reuse becomes possible only if the chosen licence is GPL-3.0-compatible. MPL-2.0 and LGPL-3.0 are compatible in one direction but impose different file-level or library-level obligations. **This option requires legal advice before it can be evaluated**, because the compatibility direction matters |
| **Option D (stay unlicensed)** | The repository cannot be published, distributed, or contributed to in any normal sense, and the "all rights reserved" statement must stay prominent. **Phase 2 work in this public repository would itself be public** — a pushed branch or PR publishes source even without a release — so under this option Phase 2 can proceed only in a genuinely private environment (the public/private choice is documented in `docs/spec/phase-2-entry-readiness.md` §3); Phase 5 cannot proceed, and no external contributor can be accepted |

---

## 5. Legal-review dependencies (identified, not answered)

Carried forward from the Phase 0 matrix §5, with the Phase 1 additions marked:

| # | Question | Status after Phase 1 |
|---|---|---|
| L1 | Does an independently written Rust reimplementation of the same language constitute a derivative work of the GPL-3.0 compiler? | **Unchanged.** This remains the single assumption option B rests on |
| L2 | Would vendoring any legacy fixture make the new test suite derivative? | **Mitigated by design** (N2): the strategy does not vendor fixtures. The question remains open only as a contingency |
| L3 | Do language and CLI names raise trademark or unfair-competition issues beyond the legacy `TRADEMARK.md`? Is `Kof` a constraining mark? | **Unchanged.** Relevant to ADR-0001/ADR-0003 naming |
| L4 | What are the legacy `TRADEMARK.md` terms? | **Unchanged.** Not analysed in either phase |
| L5 | What contributor representation (DCO/CLA) keeps the independence claim credible? | **Sharpened**: §2.3's contribution policy is the operational half; the legal half is still open |
| L6 | Effect of an outbound licence incompatible with GPL-3.0 on using GPL-3.0 tooling during differential testing? | **NEW and now concrete**: `docs/development/test-strategy.md` §5 identifies executing the legacy compiler as a black box, and publishing its output as the open question |
| L7 | Is a patent non-assertion grant advisable? | **Unchanged**, and now more relevant because §2.2 raises the patent property of the licence choice |
| L8 | Does publishing legacy compiler output in CI logs constitute distribution? | **NEW**: the same question as L6, viewed from the CI angle |
| L9 | **NEW** — Does deriving a PortuKof vocabulary table by *observing* the legacy compiler's behaviour (rather than reading its source) create a derivative work? | **NEW**, raised by S3. It is the same class as L1 but narrower and more concrete |
| L10 | **NEW** — Does the KofScript wrapping semantics, if specified from observed input/output pairs, create a derivative work of the legacy implementation? | **NEW**, raised by S4. Same class as L9 |

**L1, L9 and L10 are the questions that actually gate engineering work**, because
they determine whether S3 and S4 can be specified behaviourally under option B.
They require **qualified legal review**, and engineering must not answer them.

---

## 6. Phase 1 gate G1d status

| Requirement | Status |
|---|---|
| Licensing decision **presented** to the owner | **SATISFIED** — this document presents it, with options, consequences and a recommendation |
| Not falsely marked resolved | **SATISFIED** — the licence is undecided, `README.md` still says "Not yet chosen", no `LICENSE` file exists, and no licence field was added to `Cargo.toml` |
| Legal-review dependencies identified | **SATISFIED** — §5, ten items, three of them new |
| Actionable recommendation produced | **SATISFIED** — §2, including one unconditional action (§2.3) that does not depend on the licence decision |

**G1d remains PENDING OWNER ACTION.** The engineering obligation is met; the
owner's decision is not made, and this document does not make it.

---

## Directive 002-R1 provenance correction

Earlier references to an "independent" or "clean-room" implementation
do **not** certify a formally isolated clean-room process. Legacy source was
inspected directly. The engineering controls and unresolved legal dependencies
are defined in `docs/audits/phase-1-provenance-controls.md`; qualified review
is required before making derivative-work or distribution claims.
