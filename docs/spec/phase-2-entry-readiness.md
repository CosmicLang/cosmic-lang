# Phase 2 Entry Readiness — Owner Decision Package

| | |
|---|---|
| **Status** | **READY FOR OWNER DECISION — nothing in this document is approved or authorized except where an item records a completed owner authorization.** Directives: ENGINEERING DIRECTIVE 002-R4 (entry-gate preparation and public-repository licensing reconciliation) · 002-R5 (S-G1S-1 correction and lexical-contract corrections). |
| **Baseline** | `main` @ `c436a9fa456e021eabdd43e0bfbba36228358117` — PR #3 merged by the owner; post-merge `Validate` green. |
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

**Engineering takes no position between L-A and L-B** — the choice is the owner's, and this directive merely makes it explicit rather than implied. ☐ L-A ☐ L-B ☐ defer.

## 4. Staged Phase 2 proposal (summary)

`docs/directives/directive-003-proposal.md` has been revised (still **NOT AUTHORIZED**) to a two-stage structure:

* **Stage A — lexical and syntactic frontend:** `cosmic-lexer`, lossless CST, recovering parser, recognition and diagnostic tests, property/fuzz suite, differential recognition harness. Entry requires G1, G1-L and G1-S acceptance and resolution of the applicable G1c/G1d conditions.
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

**Current ledger.** *OWNER APPROVED:* the component gate framework (002-R3); the `.cmic` source extension (2026-10-16); the F1/F2/F4 recognition corrections (authorized and applied); the S-G1S-1 pattern-header correction (002-R5 §3, conditional on direct verification — performed; applied as F5). *AUTHORIZED TO IMPLEMENT:* **nothing** — no Phase 2 component is authorized.

| # | Owner decision | Engineering recommendation | Key evidence | Consequence if deferred | Component blocked | Status |
|---|---|---|---|---|---|---|
| 1 | **G1** — accept the specification recognition subset (corrected grammar + evidence map as the G1 basis) | Accept as corrected (F1/F2/F4/F5 applied; per-production traceability machine-checked) | 122/122 evidence map; checker G0–G13; grammar correction record in the report | Nothing downstream can be authorized | all Phase 2 | READY FOR OWNER DECISION |
| 2 | **S-G1S-1** — attested binding/destructuring arm headers (`case Circle c`, `case Point(var x, var y)`) — **resolved**: corrected so `switch-label` derives both forms | No further engineering action; G1-S acceptance remains item 4 | Direct verification of the frozen legacy source (`parseSwitchCasePatternOrValue`) and both executed cases; grammar §3.6; map rows 98–100; `docs/spec/g1-s-syntax-decision-package.md` §3.3 | — | G1-S / parser | **OWNER APPROVED** · APPLIED (002-R5 §3, conditional on verification — performed) |
| 3 | **G1-L** — accept the lexical contract (D-L1…D-L10) and the LEX allocation plan | Accept as proposed in the package | `docs/spec/g1-l-lexical-decision-package.md`; S8/S13/S14 records + probes; Phase 0 `crates/cosmic-source` tests | `cosmic-lexer` cannot start | `cosmic-lexer` | READY FOR OWNER DECISION |
| 4 | **G1-S** — accept the recognized subset, refusal contract, lambda rule, recovery and CST obligations | Accept (item 2 is resolved; no further engineering action) | `docs/spec/g1-s-syntax-decision-package.md`; report §4/§5; map statuses | `cosmic-syntax` cannot start | `cosmic-syntax` | READY FOR OWNER DECISION |
| 5 | **G1c** — identity/compatibility handling | Grant the component-limited deferral for Stage A: extension already `.cmic`; parser extension-independent; **no** MIME registration, naming or release decision now | ADR-0001, ADR-0002 §4.1; grammar carries no extension | Without deferral, Stage A stays blocked on release-level decisions it does not need | none for Stage A if deferred; identity work otherwise | READY FOR OWNER DECISION |
| 6 | **G1d** — public vs private development and licensing | Choose **L-A** or **L-B** (no engineering preference); licensing options A–D + counsel review for L-A per the licensing recommendation | §3 of this document; `docs/audits/phase-1-licensing-recommendation.md` §5 (L1–L10) | No implementation in this repository | Stage A (this repository) | READY FOR OWNER DECISION |
| 7 | **Stage A authorization** — authorize the lexical/syntactic frontend per the revised Directive 003 | Authorize explicitly after items 1–6 are decided | `docs/directives/directive-003-proposal.md` §2.1 | No implementation | `cosmic-lexer`, `cosmic-syntax`, CST | READY FOR OWNER DECISION |
| 8 | **G1-T and Stage B** — semantic frontend (resolution, typing, typed HIR, `cosmic check`) | **Defer**: requires Stage A plus its own acceptance and a separate authorization | `docs/spec/implementation-gates.md` §5 (G1-T row); directive-003 §2.2 | No semantic work; semantic gates remain the enforcement | `cosmic-sema` | DEFERRED (recommended) |
| 9 | **G1-M / G1-R** — middle-end and backend readiness | **Defer** to Phase 3/4 consumers | gates §5; S16/S17 undecided | No MIR/backend work | `cosmic-mir`, `cosmic-opt`, codegen | DEFERRED (recommended) |
| 10 | **S5 normative adoptions** (pattern breadth incl. destructuring *semantics*, guards, exhaustiveness, fallthrough, labels, three-clause acceptance) | Decide per S5's own options; **not a Stage A blocker** — recognition is separate and semantics stay gated meanwhile | `docs/spec/decisions/S5-pattern-matching.md` incl. the 002-R4 update | Semantic operations stay gated (correct non-implementation) | Tier-T operations only | READY FOR OWNER DECISION (can be deferred) |
| 11 | **S8 / S13 / S14 policy questions** | Decided through the G1-L items: S8 Q1→D-L6, Q2→D-L8, Q4→D-L7, Q5→D-L9; S13 Q4→D-L2, Q5→D-L9, Q6→D-L1; S14 Q1→D-L10, Q2→D-L1, Q3 noted, Q4→D-L4 | G1-L package §§2–3 | Lexer contract incomplete | `cosmic-lexer` | READY FOR OWNER DECISION (via item 3) |
| 12 | **S3 / S4** — PortuKof / KofScript scope | **Defer** (no Stage A consumer; both refused loudly meanwhile) | gates §4.1; S3/S4 records | None for Stage A | none | DEFERRED (recommended) |
| 13 | **S16 / S17** — memory strategy / ABI | **Defer** (no Stage A consumer) | gates §5; ADR-0005 withheld | None for Stage A | `cosmic-runtime` (Phase 4) | DEFERRED (recommended) |
| 14 | **MIME type** | **Defer** (no Stage A consumer) | ADR-0002; compatibility-policy | None for Stage A | release/tooling only | DEFERRED (recommended) |
| 15 | **Editor integration / publication** | **Defer** (no Stage A consumer) | ADR-0002; compatibility-policy §2.2 | None for Stage A | release/tooling only | DEFERRED (recommended) |
| 16 | **Measurement debt M-1…M-7** (incl. M-7 identifier-class probes) | Stage A harness deliverable, per `docs/spec/implementation-gates.md` §6; M-7 runs before D-L3 is called settled | gates §6; G1-L D-L3 | Compatibility claims stay limited to measured scope | none (owed evidence) | READY (schedule as Stage A evidence) |

## 6. Staged authorization semantics

* **Stage A approval is one decision** (matrix item 7) that becomes available when items 1–6 are decided. It authorizes exactly the Stage A components and no others.
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

It is not an approval, an authorization, a licence choice, a merge, or a release. It chooses none of the options above; every "☐" remains blank until the owner acts. Directive 002-R4 authorized documentation corrections, gate-entry analysis and a draft pull request — **no Phase 2 implementation**.
