# Phase 2 Entry Readiness — Owner Decision Package

| | |
|---|---|
| **Status** | **PARTIALLY DECIDED — items 1–5 are OWNER APPROVED (ENGINEERING DIRECTIVE 003-G1, 2026-10-16); items 2 and the Stage A gate-acceptance record were merged (PR #4, PR #5; post-merge CI green) and the owner risk-acceptance record was merged (PR #6; post-merge CI green); item 6 (G1d) — the Apache-2.0 licensing package is authorized and applied on the draft licensing branch; the internal CLEARED transition is recorded only after integration and validation (003-L3; owner authorization 003-L4, 2026-10-16); item 7 (Stage A authorization) remains NOT AUTHORIZED.** Directives: ENGINEERING DIRECTIVE 002-R4 (entry-gate preparation and public-repository licensing reconciliation) · 002-R5 (S-G1S-1 correction and lexical-contract corrections) · 003-G1 (Stage A gate acceptances) · 003-L3 (owner risk acceptance and G1d governance amendment) · 003-L4 (Apache-2.0 licensing authorization, DCO approval and attribution). |
| **Baseline** | `main` @ `920e6f2065ab1e10136e05651ac52322f249c63c` — PR #6 (the owner risk-acceptance record) merged by the owner (2026-10-17T01:41Z); post-merge `Validate` run `38991925696` green. (PR #5 merged `001ff543fe7851df3fbec2a60f4eedb6725c58c9` and PR #4 merged `e3451e45d914491aedeaa1a76e84478c84c992cc` earlier; §1 below is the historical 002-R4 preflight record.) |
| **Related** | `docs/spec/g1-l-lexical-decision-package.md` · `docs/spec/g1-s-syntax-decision-package.md` · `docs/spec/implementation-gates.md` · `docs/directives/directive-003-proposal.md` (revised, still NOT AUTHORIZED) · `docs/audits/phase-1-002-r1-reconciliation.md` §J · `docs/audits/phase-1-licensing-recommendation.md` · `docs/adr/0001-language-identity.md` · `docs/adr/0002-source-extension.md` |

> **Path convention.** A path prefixed `cosmcc:` refers to the legacy repository `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

---

## 1. Verified Phase 1 integration baseline

Preflight for Directive 002-R4, re-verified before any change:

| Item | Verified state |
|---|---|
| `main` HEAD | `c436a9fa456e021eabdd43e0bfbba36228358117` |
| PR #3 | **MERGED** by the owner account (`CosmicLang`); standard merge, no squash/rebase |
| Merge parents | `c52c56d` (prior main) + `1e9c574` (reviewed head) — exactly as reviewed |
| Merged tree | identical to the reviewed head (`git diff c436a9f 1e9c574` empty) |
| Post-merge CI | `Validate` run `38965867091` @ `c436a9f`: **success** (specification consistency + fmt/lint/test/doc) |

## 2. Bounded documentation corrections applied

| Correction | Where | Change |
|---|---|---|
| A — stale count | `.github/workflows/ci.yml` (Specification consistency step comment) | "117-production evidence map" → "119" (002-R4) → re-stamped to "122-production evidence map" by the 002-R5 production-count reconciliation. No change to workflow behavior. |
| B — S5 wording | `docs/spec/decisions/S5-pattern-matching.md` | The stale "destructuring lacks evidence" claim is reconciled with the executed cases, split into four layers: verified syntax recognition / observed legacy behavior / proposed normative semantics / unresolved decisions. **No status changes; no semantics adopted; OPEN/GATED dispositions preserved.** Cross-referenced to the G1-S finding in §3.3 of the syntax package. |
| C — S-G1S-1 corrected (002-R5 §3–§4) | `docs/spec/grammar.ebnf`, `docs/spec/grammar-evidence-map.md`, `docs/spec/g1-s-syntax-decision-package.md` §3.3, `docs/spec/decisions/S5-pattern-matching.md`, the consistency report | The two attested pattern-header forms are now derived (`binding-pattern`, `destructuring-pattern`, `pattern-field`); production count 119 → 122. Evidence verified by direct read of the frozen legacy source and both executed cases. **No semantics adopted; S5 remains OPEN.** |
| D — lexical-contract corrections (002-R5 §5–§6) | `docs/spec/g1-l-lexical-decision-package.md` | D-L2 tab-column example corrected to one-based column 3 for `a\tb`; the lossless-input boundary stated explicitly (valid UTF-8 → byte-preserving pipeline with round-trip, including malformed language syntax; invalid UTF-8 → load-time rejection with the first invalid byte offset, no CST owed); the BOM prefix leaf is a byte span over the original three bytes contributing no language token. |

## 3. G1d — public-repository reconciliation

### 3.1 The fact

**This repository is public.** A branch or pull request pushed here **publishes the source code**, whether or not any binary release exists. "Internal-only development without publication" is therefore impossible **in this repository by construction**.

### 3.2 The audited statements (before) and their corrections (applied)

| Document | Statement as found (abridged) | Correction applied |
|---|---|---|
| `docs/directives/directive-003-proposal.md` | G1d row: "…explicitly approves **internal-only** independent work **without legacy imports or publication**…" while the proposal directed implementation onto a public branch | Row replaced: the repository is public; the owner must choose Option L-A or L-B (§3.3). The proposed `phase/2-frontend` vehicle is explicitly conditional on L-A. |
| `docs/spec/implementation-gates.md` | G1d row: "explicit **internal-development-only deferral**; …**no release/distribution**"; G1c row trailing "…**no publication**" | Rows rewritten: publication fact stated; L-A/L-B options named; "no publication" clarified as "no release, announcement or MIME registration is authorized". A dated correction note was added under the gate table. |
| `docs/audits/phase-1-002-r1-reconciliation.md` | J-5: "**Until then: internal development only**, no release, no external contribution promises." | Corrected: no release and no external contribution promises; work in this public repository is itself public; L-A/L-B pointer added. §G's Option-D gloss ("keeps the project internal-only") corrected likewise. |
| `docs/audits/phase-1-licensing-recommendation.md` | Option D consequence: "**Phase 2 can still proceed** (it is internal work)" | Corrected: under Option D, Phase 2 work in this public repository would itself be public; it can proceed only in a genuinely private environment (L-B), pending the owner's choice. |

**Preserved unchanged:** no licence file, no licensing metadata, no legal representation was added; the provenance controls and the no-legacy-code-import rule stand as recorded in `docs/audits/phase-1-002-r1-reconciliation.md` §G; no licence option is chosen; no release is authorized.

### 3.3 The two owner options (choose exactly one when licensing is addressed)

**Option L-A — Public development.** The owner obtains appropriate legal/provenance review (the L1/L6/L8/L9/L10 items of `docs/audits/phase-1-licensing-recommendation.md` §5), establishes a licensing and contribution policy (options A–D of that document, chosen by the owner), and then **expressly authorizes public implementation**. Consequences: every Stage A/B artifact is published as it lands; the licensing decision precedes or accompanies implementation; contributions are covered by the established policy.

**Option L-B — Private development.** Implementation remains **exclusively in a genuinely private repository or local working environment** until the owner resolves publication and licensing. Consequences: no implementation branches or PRs in this public repository; the specification and decision documents already public here remain as they are; the work returns to public view only by a later explicit owner action.

**Engineering takes no position between L-A and L-B** — the choice is the owner's. **Owner decision (2026-10-16): ☑ L-A** — public development selected; the intended outbound licence is Apache-2.0. By explicit owner decision the internal G1d procedure was amended (003-L3, 2026-10-16): mandatory qualified-counsel review is replaced by the owner risk-acceptance record in §9; the Apache-2.0 licensing package was subsequently authorized and applied on the draft licensing branch (003-L4, 2026-10-16), and the internal G1d CLEARED transition is recorded only after integration and validation (§5 item 6, §9). ☐ L-B ☐ defer.

## 4. Staged Phase 2 proposal (summary)

`docs/directives/directive-003-proposal.md` has been revised (still **NOT AUTHORIZED**) to a two-stage structure:

* **Stage A — lexical and syntactic frontend:** `cosmic-lexer`, lossless CST, recovering parser, recognition and diagnostic tests, property/fuzz suite, differential recognition harness. Entry requires G1, G1-L and G1-S acceptance and resolution of the applicable G1c/G1d conditions. G1, G1-L and G1-S are accepted and the G1c Stage A-limited deferral is approved (003-G1); **G1d — licensing package authorized and applied; the internal CLEARED transition is recorded only after integration and validation** (003-L3; owner authorization 003-L4).
* **Stage B — semantic frontend:** name resolution, type checking, typed HIR, per-operation feature gates, `cosmic check`. Requires Stage A **and** G1-T entry **and** a separate owner authorization of its scope. **Stage A approval must not silently authorize Stage B.**

Prohibited by the proposal: MIR, optimizer, backend, runtime and standard-library implementation. `.cmic` remains the primary source extension and the parser stays independent of filename extensions; the documented Rust `.csmx` literal migration inventory (`docs/adr/0001-language-identity.md` §6) is preserved for future authorized implementation.

## 5. Owner decision matrix

**Legend.**

| Label | Meaning |
|---|---|
| **READY FOR OWNER DECISION** | The engineering package is complete; the owner has not acted. |
| **OWNER APPROVED** | The owner has acted; this does **not** by itself authorize any implementation. |
| **AUTHORIZED TO IMPLEMENT** | The owner approved **and** authorized the scoped implementation work. |
| **DEFERRED (recommended)** | Engineering recommends deferral now; the owner may still act. |

**Current ledger.** *OWNER APPROVED:* the component gate framework (002-R3); the `.cmic` source extension (2026-10-16); the F1/F2/F4 recognition corrections (authorized and applied); the S-G1S-1 pattern-header correction (002-R5 §3, conditional on direct verification — performed; applied as F5); **G1 (003-G1 §2), G1-L (003-G1 §3), G1-S (003-G1 §4) and the G1c Stage A-limited deferral (003-G1 §5)**; **L-A and the outbound licence Apache-2.0 with the licensing package — authorized and applied on the draft licensing branch (owner authorization 003-L4 §1–§3, 2026-10-16)**; **the G1d governance amendment — owner risk acceptance replacing mandatory qualified-counsel review (003-L3 §1–§3, 2026-10-16)**. *MERGED:* PR #4, PR #5 and PR #6 (owner merges; post-merge CI green). *AUTHORIZED TO IMPLEMENT:* **nothing** — no Phase 2 component is authorized; the licensing package awaits owner merge approval, and the Stage A authorization remains outstanding.

| # | Owner decision | Engineering recommendation | Key evidence | Consequence if deferred | Component blocked | Status |
|---|---|---|---|---|---|---|
| 1 | **G1** — accept the specification recognition subset (corrected grammar + evidence map as the G1 basis) | Accept as corrected (F1/F2/F4/F5 applied; per-production traceability machine-checked) | 122/122 evidence map; checker G0–G13; grammar correction record in the report | Nothing downstream can be authorized | all Phase 2 | **OWNER APPROVED** (003-G1 §2) |
| 2 | **S-G1S-1** — attested binding/destructuring arm headers (`case Circle c`, `case Point(var x, var y)`) — **resolved**: corrected so `switch-label` derives both forms | No further engineering action; G1-S acceptance remains item 4 | Direct verification of the frozen legacy source (`parseSwitchCasePatternOrValue`) and both executed cases; grammar §3.6; map rows 98–100; `docs/spec/g1-s-syntax-decision-package.md` §3.3 | — | G1-S / parser | **OWNER APPROVED** · APPLIED (002-R5 §3, conditional on verification — performed) |
| 3 | **G1-L** — accept the lexical contract (D-L1…D-L10) and the LEX allocation plan | Accept as proposed in the package | `docs/spec/g1-l-lexical-decision-package.md`; S8/S13/S14 records + probes; Phase 0 `crates/cosmic-source` tests | `cosmic-lexer` cannot start | `cosmic-lexer` | **OWNER APPROVED — G1-L ENTERED** (003-G1 §3) |
| 4 | **G1-S** — accept the recognized subset, refusal contract, lambda rule, recovery and CST obligations | Accept (item 2 is resolved; no further engineering action) | `docs/spec/g1-s-syntax-decision-package.md`; report §4/§5; map statuses | `cosmic-syntax` cannot start | `cosmic-syntax` | **OWNER APPROVED — G1-S ENTERED** (003-G1 §4) |
| 5 | **G1c** — identity/compatibility handling | Grant the component-limited deferral for Stage A: extension already `.cmic`; parser extension-independent; **no** MIME registration, naming or release decision now | ADR-0001, ADR-0002 §4.1; grammar carries no extension | Without deferral, Stage A stays blocked on release-level decisions it does not need | none for Stage A if deferred; identity work otherwise | **OWNER APPROVED — Stage A-limited deferral** (003-G1 §5) |
| 6 | **G1d** — public vs private development and licensing | Choose **L-A** or **L-B** (no engineering preference); licensing options A–D per the licensing recommendation, with the review requirement now governed by the 003-L3 amendment | §3 and §9 of this document; `docs/audits/phase-1-licensing-recommendation.md` §5 (L1–L10) | No implementation in this repository | Stage A (this repository) | **L-A SELECTED · Apache-2.0 AUTHORIZED AND APPLIED ON THE DRAFT LICENSING BRANCH · G1d — the internal CLEARED transition is recorded only after integration and validation** — mandatory qualified-counsel review replaced by the owner risk-acceptance record (§9; 003-L3 §1–§3); licensing package per 003-L4 §1–§3; the internal CLEARED status is **not** professional legal clearance |
| 7 | **Stage A authorization** — authorize the lexical/syntactic frontend per the revised Directive 003 | Authorize explicitly after items 1–6 are decided and the licensing package is applied | `docs/directives/directive-003-proposal.md` §2.1 | No implementation | `cosmic-lexer`, `cosmic-syntax`, CST | **NOT AUTHORIZED** — the gate-acceptance record is merged (PR #5); requires the amended G1d transition (licensing package integrated and validated; 003-L3 §3; 003-L4 §4, §6) and a separate directive (003-G1 §7–§8) |
| 8 | **G1-T and Stage B** — semantic frontend (resolution, typing, typed HIR, `cosmic check`) | **Defer**: requires Stage A plus its own acceptance and a separate authorization | `docs/spec/implementation-gates.md` §5 (G1-T row); directive-003 §2.2 | No semantic work; semantic gates remain the enforcement | `cosmic-sema` | DEFERRED (recommended) |
| 9 | **G1-M / G1-R** — middle-end and backend readiness | **Defer** to Phase 3/4 consumers | gates §5; S16/S17 undecided | No MIR/backend work | `cosmic-mir`, `cosmic-opt`, codegen | DEFERRED (recommended) |
| 10 | **S5 normative adoptions** (pattern breadth incl. destructuring *semantics*, guards, exhaustiveness, fallthrough, labels, three-clause acceptance) | Decide per S5's own options; **not a Stage A blocker** — recognition is separate and semantics stay gated meanwhile | `docs/spec/decisions/S5-pattern-matching.md` incl. the 002-R4 update | Semantic operations stay gated (correct non-implementation) | Tier-T operations only | READY FOR OWNER DECISION (can be deferred) |
| 11 | **S8 / S13 / S14 policy questions** | Decided through the G1-L items: S8 Q1→D-L6, Q2→D-L8, Q4→D-L7, Q5→D-L9; S13 Q4→D-L2, Q5→D-L9, Q6→D-L1; S14 Q1→D-L10, Q2→D-L1, Q3 noted, Q4→D-L4 | G1-L package §§2–3 | Lexer contract incomplete | `cosmic-lexer` | **OWNER APPROVED via G1-L** (003-G1 §3; S8/S13/S14 records updated at individual-rule granularity) |
| 12 | **S3 / S4** — PortuKof / KofScript scope | **Defer** (no Stage A consumer; both refused loudly meanwhile) | gates §4.1; S3/S4 records | None for Stage A | none | DEFERRED (recommended) |
| 13 | **S16 / S17** — memory strategy / ABI | **Defer** (no Stage A consumer) | gates §5; ADR-0005 withheld | None for Stage A | `cosmic-runtime` (Phase 4) | DEFERRED (recommended) |
| 14 | **MIME type** | **Defer** (no Stage A consumer) | ADR-0002; compatibility-policy | None for Stage A | release/tooling only | DEFERRED (recommended) |
| 15 | **Editor integration / publication** | **Defer** (no Stage A consumer) | ADR-0002; compatibility-policy §2.2 | None for Stage A | release/tooling only | DEFERRED (recommended) |
| 16 | **Measurement debt M-1…M-7** (incl. M-7 identifier-class probes) | Stage A harness deliverable, per `docs/spec/implementation-gates.md` §6; M-7 runs before D-L3 is called settled | gates §6; G1-L D-L3 | Compatibility claims stay limited to measured scope | none (owed evidence) | READY (schedule as Stage A evidence) |

**003-G1 update (2026-10-16).** Items 1, 3, 4 and 5 were decided as accepted (G1; G1-L entered; G1-S entered; G1c Stage A-limited deferral); item 2 was merged by the owner (PR #4; post-merge CI green); item 6 records L-A selected with the intended licence Apache-2.0 (conditional, **not applied**) and **remains PENDING**; item 7 therefore remains **NOT AUTHORIZED**. Item 10 is untouched — no S5 semantics were decided. The acceptance records live in `docs/spec/implementation-gates.md` §5.1, the two entry packages and the affected S records (003-G1 updates).

**003-L3 update (2026-10-16).** The owner accepted Option B of Directive 003-L2 §F and amended the internal G1d governance procedure: mandatory qualified-counsel review is replaced by the owner risk-acceptance record in §9. Current result: **G1d — AMENDED PROCEDURE APPROVED; LICENSING IMPLEMENTATION PENDING**; reserved, not assigned: **G1d — CLEARED UNDER OWNER-ACCEPTED RISK PROCEDURE (NO QUALIFIED-COUNSEL REVIEW)**, reachable only when the owner-authorized Apache-2.0 licensing and contribution package is applied and validated, with any unresolved technical or ownership blocker reported. Item 7 remains **NOT AUTHORIZED**; items 1–5, the merge records and every deferral are unchanged; no licence, metadata or repository setting was changed by this record. The identified minimal scope of the next licensing PR is in §9.

**003-L4 update (2026-10-16).** The owner expressly authorized publication of the Apache-2.0 licensing package on branch `legal/apache-2.0-adoption` (merge requires a separate owner approval), approved the Developer Certificate of Origin (`Signed-off-by`) as the inbound contribution policy, and approved the public attribution **Copyright 2026 CosmicLang**; Option B and the 003-L3 risk acceptance were reaffirmed. The package is applied on the draft licensing PR (root `LICENSE`, Cargo metadata and inheritance, README declaration, `CONTRIBUTING.md` contribution policy, dependency-licence review, governance records). Current result: **G1d — licensing package authorized and applied; the internal CLEARED transition is recorded only after integration and validation**; reserved, not assigned until then: **G1d — CLEARED UNDER OWNER-ACCEPTED RISK PROCEDURE (NO QUALIFIED-COUNSEL REVIEW)** — recorded only when the package is integrated and validated with no unresolved blocker; that result is an internal risk-based clearance and must never be represented as professional legal clearance. Item 7 remains **NOT AUTHORIZED**; items 1–5, the merge records and every deferral are unchanged.

## 6. Staged authorization semantics

* **Stage A approval is one decision** (matrix item 7) that becomes available when items 1–6 are decided — items 1–6 are now decided (item 6 under the amended owner risk-acceptance procedure, with the licensing package applied on the draft licensing branch and awaiting integration and validation); item 7 additionally requires that transition and remains **NOT AUTHORIZED**. It would authorize exactly the Stage A components and no others.
* **Stage B approval is a separate future decision** (matrix item 8): it additionally requires G1-T entry (per-operation rules or gates accepted) and its own authorization; Stage A artifacts do not imply it.
* **No stage authorizes** MIR, optimizer, backend, runtime or standard library; those remain Phase 3/4 and gate on G1-M/G1-R and S16/S17.
* If the owner chooses **L-B**, items 1–6 and the packages remain valid decisions for the private workstream; only the *venue* changes (no branches or PRs in this public repository).

## 7. Validation

This change is documentation-only: no crate, dependency or compiler behavior is touched. The branch is required to pass the established validation before review:

```sh
python3 tools/spec-consistency.py          # G0–G13, exit 0
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo doc --workspace --no-deps
```

The captured checker output is refreshed in `docs/spec/grammar-consistency-report.md` §3; CI (`Validate`) records the exact-SHA result on the draft pull request.

## 8. What this document is not

It is not itself an authorization to implement, a licensing grant, a merge, or a
release. It records the owner's decisions where the owner acted; the open
checkboxes above mark decisions not yet taken (the licensing implementation
under item 6 and the deferrals). The 003-G1 acceptances record G1/G1-L/G1-S/G1c;
the 003-L3 decision amended the internal G1d procedure (owner risk acceptance,
no qualified-counsel review), and the 003-L4 authorization applied the
Apache-2.0 licensing package on the draft licensing branch; the internal G1d
CLEARED transition and the Directive 003-A
authorization remain outstanding — **no Phase 2 implementation is authorized**.

## 9. Owner risk-acceptance record (003-L3, 2026-10-16)

The owner expressly accepted **Option B** of Directive 003-L2 §F and amended the project's internal G1d governance procedure: mandatory qualified-counsel review is replaced by this record. The following is recorded accurately. This is an owner governance decision — **not** a legal opinion, copyright determination or waiver of third-party rights — and it creates no signature, certification, identity proof or professional clearance.

* **Ownership.** The owner declares holding the necessary rights over the Cosmic-Lang code and documentation and no known employment or contractual obligations transferring those rights. **This remains a self-declaration, not an independently proven chain of title.**
* **Provenance.** The owner accepts the verified technical findings of 003-L1/003-L2: no imported legacy implementation or fixtures were identified, while acknowledging that **a complete similarity comparison was not performed**.
* **Legacy inspection.** The owner acknowledges that the legacy Java source was directly inspected and that **the project cannot claim formal clean-room separation**.
* **Legal uncertainty.** The owner accepts the documented residual risks relating to derivative works, GPL-3.0, AI-assisted authorship, copyright ownership, trademarks, patents and future third-party claims.
* **Professional advice.** The owner acknowledges that neither the Claude preliminary report nor the engineering audits constitute qualified legal advice; **no qualified lawyer has reviewed or cleared the project**.
* **Continuing safeguards.** The ban on importing or mechanically translating GPL-3.0 legacy material is retained; **external contributions remain restricted** until an approved licensing and contribution policy is in force.

**Current result: G1d — licensing package authorized and applied on the draft licensing branch (003-L4, 2026-10-16); the internal CLEARED transition is recorded only after integration and validation.** Reserved, not assigned until then: **G1d — CLEARED UNDER OWNER-ACCEPTED RISK PROCEDURE (NO QUALIFIED-COUNSEL REVIEW)** — the transition requires the owner-authorized Apache-2.0 licensing and contribution package to be integrated and validated, with any unresolved technical or ownership blocker reported; the result certifies compliance with the amended internal governance procedure only and is **not professional legal clearance**. For the historical record: the original G1d procedure required a dated written qualified-counsel assessment (003-G1 §6; matrix item 6 as recorded 2026-10-16); that requirement was replaced by this explicit owner decision and is preserved in the dated gate-register records, not falsified.

**Licensing PR — identified scope (003-L3), authorized and applied (003-L4 §1).** The scope identified by 003-L3 was expressly authorized by the owner and applied on branch `legal/apache-2.0-adoption` (draft PR; **merge requires a separate owner approval**):

* official, unmodified Apache License 2.0 text as `LICENSE` — applied (source `https://www.apache.org/licenses/LICENSE-2.0.txt`, 11,358 bytes, SHA-256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`);
* copyright-holder attribution — the owner approved the public attribution **Copyright 2026 CosmicLang** (003-L4 §3); it appears in the README licensing declaration and remains a declaration selected by the owner, **not independently verified legal proof of ownership**;
* `license = "Apache-2.0"` in `[workspace.package]`, inherited by the three crates via `license.workspace = true` — applied;
* README licensing declaration — applied;
* `CONTRIBUTING.md` inbound-contribution policy — the owner approved the **DCO (`Signed-off-by`)**, not a CLA (003-L4 §2); the provenance controls are retained and the DCO 1.1 text is included;
* `NOTICE` — **not added**: no applicable third-party attribution notices exist (zero external dependencies; no bundled third-party material);
* dependency-review requirement — recorded (`CONTRIBUTING.md` §3; `docs/development/toolchain-policy.md` §3); licence-audit tooling is introduced when the first dependency is added.

### 003-L4 authorization and licensing-package record (2026-10-16)

The owner expressly authorized publication of the Apache-2.0 licensing package on branch `legal/apache-2.0-adoption` — committed, pushed and validated in the public repository; **merging requires a separate owner approval** — approved the **Developer Certificate of Origin (`Signed-off-by`)** as the inbound contribution policy, and approved the public copyright attribution **Copyright 2026 CosmicLang**. The owner reaffirmed the 003-L3 risk acceptance (Option B) and retained the provenance restrictions: no importing or mechanically translating GPL-3.0 legacy material; independently authored implementation and tests; rights to contribute; escalation of suspected provenance violations. Stage A and every Phase 2 component remain **NOT AUTHORIZED** (003-L4 §5–§6: no auto-merge, no `phase/2-frontend` branch, no lexer/parser/CST code, no Stage B work, no legacy material imported, no clean-room or professional-clearance claim), and no professional legal clearance is claimed or implied.

The package applied on the draft licensing PR: root `LICENSE` (complete, unmodified Apache License 2.0 text — source `https://www.apache.org/licenses/LICENSE-2.0.txt`, 11,358 bytes, SHA-256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`); `license = "Apache-2.0"` in `[workspace.package]`, inherited by the three crates via `license.workspace = true`; the README licensing declaration with the approved attribution; `CONTRIBUTING.md` (DCO 1.1 with required `Signed-off-by`, provenance rules, dependency-licence review); the dependency-licence note in `docs/development/toolchain-policy.md` §3; and the governance-record updates in this document and `docs/spec/implementation-gates.md`. No `NOTICE` file is added — no applicable third-party attribution notices exist. No compiler, dependency or repository-setting change is included.

**Transition condition.** G1d changes to **G1d — CLEARED UNDER OWNER-ACCEPTED RISK PROCEDURE (NO QUALIFIED-COUNSEL REVIEW)** only when this package is integrated (merged) and validated with no newly identified blocker left unresolved. That status is an internal, risk-accepted governance result: it is **not** legal advice, **not** professional legal clearance and does **not** eliminate third-party rights — and it must never be represented as any of those. After merge and post-merge validation, the final G1d transition and the separate Directive 003-A authorization will be requested (003-L4 §4, §6).
