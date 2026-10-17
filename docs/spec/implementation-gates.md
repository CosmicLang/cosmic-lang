# Implementation Gates — Resolution of the Phase 1 Approval Cycle

| | |
|---|---|
| **Status** | **Accepted by the owner** (ENGINEERING DIRECTIVE 002-R3, 2026-10-16). The component-scoped model replaces the flat `G1b` and is the project's implementation-readiness model. Acceptance covers the **framework**, not individual language rules (S1–S18 remain individually authoritative) and is **not** by itself Phase 2 authorization. (`.cmic` selection: ADR-0002 §4.1.) **Stage A entry gates:** G1, G1-L and G1-S accepted and the G1c Stage A-limited deferral approved (ENGINEERING DIRECTIVE 003-G1, 2026-10-16); **G1d — CLEARED UNDER OWNER-ACCEPTED RISK PROCEDURE (NO QUALIFIED-COUNSEL REVIEW)** — the licensing package was integrated and validated (PR #7, merged 2026-10-17T02:23:07Z; post-merge CI green; 003-L3; owner authorization 003-L4, 2026-10-16); an internal risk-based status, **not** professional legal clearance. **Stage A implementation is authorized by ENGINEERING DIRECTIVE 003-A (2026-10-17); Stage B remains NOT AUTHORIZED.** |
| **Directive** | ENGINEERING DIRECTIVE 002-R1 §3 · 002-R2 (post-merge) §5 · 002-R3 §7 · 003-G1 §2–§6 · 003-L3 §1–§5 · 003-L4 §1–§7 · 003-A (G1d transition and Stage A authorization, 2026-10-17) |
| **Supersedes** | the single `G1b` gate defined in `docs/spec/decisions/README.md` and `docs/directives/directive-003-proposal.md` |
| **Related** | `docs/spec/decisions/` (S1–S18), `docs/audits/phase-1-legacy-probes.md`, `docs/spec/grammar-consistency-report.md`, `docs/directives/directive-003-proposal.md` |

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

---

## 1. The defect

Phase 1 defined five gates. One of them, **G1b**, was:

> "All 18 questions decided or explicitly owner-dependent."

and Phase 2 was gated on G1–G1e. Two questions — **S1** (generics and overload
resolution) and **S11** (evaluation order) — were marked
**BLOCKED-ON-MEASUREMENT**, and the measurement was assigned to Phase 2's
differential harness. That produces a closed cycle:

```
G1b requires S1 and S11 to be decided
        ^                                S1 and S11 require measurement
        |                                |
        |                                v
Phase 2 cannot start  <----------  measurement happens in Phase 2
```

The cycle is real and it is fatal: no authorised action can break it. It is also
**entirely an artefact of how the gate was written**, not of any genuine
technical dependency. The rest of this document removes the artefact without
lowering any technical standard.

---

## 2. Root cause

Three distinct modelling errors, each independently sufficient to cause the
cycle.

### 2.1 A single gate over questions with different consumers

G1b treated the 18 questions as one undifferentiated set. They are not. They
differ in **which component consumes the answer**:

* Whether an escape sequence is valid is a **lexical** question. Nothing else
  needs it.
* Whether `switch` must be exhaustive is a **type-checking** question. The lexer
  and the parser do not need it.
* Whether reclamation is by tracing GC or by ownership is a **runtime**
  question. Neither the frontend nor the type checker needs it.
* Whether PortuKof is in scope is a **product** question. No component needs it
  until PortuKof support is attempted.

A question that no component needs yet cannot be a gate on any component.

### 2.2 The measurement was scheduled in the phase it was blocking

S1 and S11 needed **executed evidence**. The only instrument that produces it —
the differential harness — was assigned to Phase 2, and Phase 2 was gated on
S1 and S11 being decided. The dependency was ordered backwards.

**Correction:** a measurement is a *deliverable* of the phase that can produce
it, not a *prerequisite* of that phase. The prerequisite is the **ability to
measure**, not the result.

### 2.3 "Decided" was conflated with "settled"

G1b required every question to be *decided*. But an implementation can be
correctly built while a question remains open, **provided the compiler refuses
the affected operation instead of guessing**. Gating on "decided" forces every
decision to be made before any code is written, which is neither achievable nor
desirable — and it is precisely the pressure that would make an engineer invent
a rule to unblock progress. That is the failure mode the whole evidence
discipline exists to prevent.

**Correction:** the gate requires each question to have a **disposition** —
`EVIDENCED`, `GATED`, `DEFERRED` or `OWNER` — and requires that a `GATED`
disposition be **enforced by a feature gate**, not merely recorded.

---

## 3. Resolution

### 3.1 Principle 1 — separate syntax recognition from semantic acceptance

> **The lexer and parser recognise the full grammar. Gating happens where
> semantics are consumed.**

A parser may recognise a construct whose semantics are not yet approved. It
must not invent behaviour, and the *semantic* stage must refuse the operation it
cannot perform.

Concretely:

| Stage | Behaviour with an open decision |
|---|---|
| **Lexer** | Produces tokens. Gates only on **lexical** questions (e.g. an undecided escape rule). |
| **Parser / CST** | Parses the whole grammar. Emits **no** gate for a syntactic construct whose *semantics* are open. Produces a CST/AST that preserves the construct. |
| **Name resolution / typing / HIR** | Gates **per operation**. A construct whose semantics are open produces a `GATE` error naming the construct and the decision record — but only when the open operation is actually required. |
| **MIR / backend / runtime** | Gates on middle-end and runtime decisions. |

This is the rule the directive requires: *"A parser may recognize a construct
whose semantics are not yet approved, provided that semantic compilation does
not invent behavior."*

### 3.2 Principle 2 — gate the semantic operation, not the syntax category

> **A gate names a `(construct, decision record, unsupported semantic
> operation)` triple.** It must never reject an entire category of syntax.

The distinction is what makes gating usable. Worked examples using Phase 1's
actual open questions:

| Open question | Wrong (category-level gate) | Right (operation-level gate) |
|---|---|---|
| S1 Q1 — nested type-argument resolution | Reject all generic types | Accept `Box<Int>`, `List<Box<Int>>`; gate **only** a type argument that is itself a parameterised type at depth ≥ 2 |
| S5 Q3 — exhaustiveness | Reject `switch` entirely | Accept every `switch`; gate **only** the exhaustiveness check, and only where exhaustiveness is required |
| S5 Q4 — fallthrough / `break` resolution | Reject `switch` in a loop | Accept `switch` anywhere; `break` inside a switch arm is executed behavior that exits the switch (recorded, 002-R3); gate **only** the normative adoption of the no-fallthrough / `break` rules if it is needed |
| S12 Q7 — map-key/set contract | Reject all collections | Accept all collections; gate **only** hashing a key whose equality rule is open |
| S7 — integer-literal promotion | Reject integer literals | Accept all literals; gate **only** the out-of-range decimal case |
| S6 — per-target capability | Reject all concurrency | Accept `spawn`/`await`; gate **only** on a target that cannot honour the required guarantee |
| S18 — MIR verifier obligations | Block the middle-end | Gate **only** the pass whose postcondition cannot be checked |

**Consequence:** a program that avoids the open operation compiles and behaves
normally. The gate is a scalpel, not a switch.

### 3.3 Principle 3 — measurements are deliverables, not prerequisites

The **ability to measure** gates a phase; the **result** does not.

* Phase 2 must contain the differential harness. That is a Phase 2
  **deliverable**.
* S1's and S11's measurements are produced by it. They therefore **cannot**
  gate Phase 2's entry — only its completion.

And in fact most of this is now moot: **the measurements have been taken**.
Directive 002-R1 §4 authorised read-only probes, and
`docs/audits/phase-1-legacy-probes.md` records them. S11 has measured JVM outcomes (the capture-design intent remains OPEN), and
S1's questions are answered or narrowed to precise, reproducible defects. The
remaining measurement debt is small and named in §6.

### 3.4 Principle 4 — progress disposition versus normative status

Only the six normative statuses in `language-spec.md` §0.2 describe the
authority of a rule: **VERIFIED LEGACY BEHAVIOR**, **PROPOSED NORMATIVE RULE**,
**ACCEPTED NORMATIVE RULE**, **OPEN**, **DEFERRED**, **NOT VERIFIED**.
In particular, an executed experiment establishes **VERIFIED LEGACY BEHAVIOR**,
not **ACCEPTED NORMATIVE RULE**. Approval is a separate owner action.

The table below uses *implementation dispositions*: **EVIDENCED** (an observed
legacy behavior and/or a separately labelled proposal), **GATED** (a specifically
identified operation that must issue a blocking diagnostic until its rule is
accepted), **DEFERRED** (a later-stage dependency), and **OWNER** (a product,
identity or legal choice). These are planning labels, not normative statuses.
An EVIDENCED disposition does not authorize semantic code generation by itself.
A proposal may guide parser recognition; compiling the contested semantic
operation requires acceptance or a narrowly defined owner-approved provisional
contract. No unspecified behavior may be silently chosen.

A GATED operation is a correct *non-implementation*, provided that a gate hit
names the precise operation, emits a stable diagnostic and produces no artifact.
Recognition of the same syntactic category for other operations remains valid.
Once the owner accepts a rule, the affected row of §4.1 is marked **ACCEPTED**
with the accepting directive — an owner-action marker, not another planning
label; the rule's normative status lives in its decision record.

---

## 4. Tiering the 18 decisions

Each record is assigned to the tier of the component that first consumes it. A
record may appear in more than one tier when its sub-questions have different
consumers; those are split explicitly.

| Tier | Consumer | Records |
|---|---|---|
| **L — Lexical** | `cosmic-lexer` | S8, S13, S14 |
| **S — Syntactic** | `cosmic-syntax` (parser/CST) | S5 (patterns, guards — *recognition only*), S9 (module forms), S10 (contextual declarations, annotation syntax), S2 (`?` suffix syntax), S15 (gate vocabulary) |
| **T — Semantic** | `cosmic-sema` (resolution, typing, typed HIR) | S1, S2 (conversions), S5 (exhaustiveness, fallthrough), S6 (typing), S7, S11 (scoping/shadowing), S12, S18 |
| **M — Middle-end** | `cosmic-mir`, `cosmic-opt` | S11 (evaluation order), S18 (verifier obligations) |
| **R — Runtime / backend** | `cosmic-runtime`, `cosmic-codegen-*` | S6 (per-target capability table), S16, S17 |
| **O — Owner** | none (scope decisions) | S3, S4 |

### 4.1 Disposition table

| Record | Tier(s) | Disposition | Basis |
|---|---|---|---|
| S1 nested generics / inference / overload | T | **GATED** | Q1 exhibited a **probable compiler defect** (`docs/audits/phase-1-legacy-probes.md` §6.1–6.2); Q5 partial (§6.4). Nested arguments (depth ≥ 2) and ambiguous overloads are gated |
| S2 nullability | S (syntax), T (conversions) | **S: EVIDENCED · T: GATED** | `?` suffix syntax verified; "soft cast" now defined (§8 of the probe record); conversion rules beyond `T → T?` gated |
| S3 PortuKof | O | **OWNER** | Scope decision. `.ptkf` is refused loudly meanwhile |
| S4 KofScript | O | **OWNER** | Scope decision. `.ks` is refused loudly meanwhile |
| S5 pattern matching | S (patterns/guards), T (exhaustiveness/fallthrough) | **S: ACCEPTED (recognition, 003-G1) · T: GATED** | Pattern and guard **syntax** is parser-evidenced and now inside the accepted recognition subset; exhaustiveness and fallthrough remain gated; recognition is not semantic adoption |
| S6 concurrency | T (typing), R (capability) | **T: GATED · R: DEFERRED** | `spawn`/`await` typing gated; the capability table belongs to Phase 4 |
| S7 numeric semantics | T | **GATED** | Wrapping and formatting are verified; literal promotion, division-by-zero and the Bool/Int inconsistency are gated |
| S8 string semantics | L | **ACCEPTED (G1-L: Q1, Q2, Q4, Q5 — 003-G1)** | Triple-quote rejection, raw newlines and the strict unknown-escape error verified/probed; unknown escapes are refused with LEX008 (corpus impact measurement owed) |
| S9 stdlib boundary | S (module forms), T (member kinds) | **S: EVIDENCED · T: GATED** | `package`/`import` forms are parser-evidenced; property-versus-method classification is gated |
| S10 reserved keywords / annotations / FFI | S | **EVIDENCED (recognition) · GATED (semantics)** | All reserved words and contextual declarations are **parsed**; every unsupported construct is refused with a specific diagnostic |
| S11 evaluation order and binding | T (scope), M (order) | **EVIDENCED** | JVM behavior observed for seven questions; closure-capture intent remains OPEN (§2 of the probe record) |
| S12 equality | T | **EVIDENCED (strings, records, classes, float corners) · OPEN (enum Q6, Q7)** | JVM string/record/class/float probes are measured; enum identity and map-key/set contract remain NOT VERIFIED; gate only unsupported equality operations |
| S13 Unicode | L | **ACCEPTED (G1-L: Q1, Q4, Q5, Q6 — 003-G1) · Q3 deferred** | Identifier classes verified; the column divergence is measured and documented; M-7 classification cases owed; the confusable-identifier warning is off by default |
| S14 trivia | L | **ACCEPTED (G1-L: Q1, Q2, Q4 — 003-G1)** | Lossless preservation, exact terminator forms and the BOM-prefix refinement are evidence-backed and accepted |
| S15 feature gates | S | **EVIDENCED** | The vocabulary in this document *is* the answer |
| S16 memory strategy | R | **DEFERRED** | Owner decision; no component before Phase 4 needs it |
| S17 ABI versioning | R | **DEFERRED** | Owner decision; blocked only on S16 |
| S18 MIR invariants | T, M | **EVIDENCED (proposed set) · GATED (per-pass)** | TI-1…TI-8 and MI-1…MI-8 proposed; a pass whose postcondition cannot be checked is gated |

**EVIDENCED indicates documented observations or proposals only; it does not
establish ACCEPTED NORMATIVE RULE.** See the authoritative statuses in each record.
Do not infer cross-target parity from JVM-only probes.

---

## 5. Revised gate structure

The single `G1b` is replaced by **five component-scoped gates** (`G1-L`,
`G1-S`, `G1-T`, `G1-M`, `G1-R`), plus the non-cyclical Phase 1 gates that
already existed.

> **Owner acceptance record (Directive 002-R3 §7, 2026-10-16).** The owner has
> adopted this component-scoped model as the project's implementation-readiness
> model, superseding the circular flat `G1b`. Five consequences are explicit:
>
> 1. **Component scoping.** G1-L gates lexical implementation, G1-S gates the
>    parser/CST implementation, G1-T gates semantic analysis and typed HIR,
>    G1-M gates MIR, and G1-R gates runtime/backend implementation.
> 2. **No automatic satisfaction.** A passing component gate does **not**
>    automatically satisfy any other component gate.
> 3. **Framework, not rules.** The owner accepted the gate framework — not every
>    normative language rule. The S1–S18 records remain individually
>    authoritative.
> 4. **No silent semantics.** No component may silently implement semantics
>    whose required rule is still OPEN; the exact operation must be gated
>    (§3.2, §7).
> 5. **Entry conditions still apply.** Each component gate keeps its own
>    measurable entry condition (table below). Accepting the model is not entry
>    into any component gate and not Phase 2 authorization.

| Gate | Scope | Measurable entry condition |
|---|---|---|
| **G1** | Specification evidence | Grammar/specification/typing contracts and consistency audit delivered; owner explicitly approves their stated recognition subset and provisional exclusions; per-production traceability limitations disclosed |
| **G1b** | Historical flat semantic-decision gate | **SUPERSEDED.** The owner accepted G1-L/G1-S/G1-T/G1-M/G1-R as the replacement (002-R3, 2026-10-16); the flat gate is an archival record only. |
| **G1-L** | **Lexical readiness** — unlocks `cosmic-lexer` | The owner accepts the lexical contract for emitted tokens and error cases (including invalid escapes, Unicode boundaries and newline handling); GATE/LEX diagnostic codes allocated for open cases |
| **G1-S** | **Syntactic readiness** — unlocks `cosmic-syntax` (parser/CST) | The owner accepts the recognized syntax subset; each unsupported syntax production is identified, each ambiguous construct has a documented disambiguation or a PARSE diagnostic; CST byte round-trip and recovery obligations are specified |
| **G1-T** | **Semantic readiness** (name resolution, typing, typed HIR) — unlocks `cosmic-sema` | Accepted rules for each implemented semantic operation; per-operation (construct, decision, operation, diagnostic, closing evidence) entries for each unaccepted operation; negative tests require no output artifact |
| **G1-M** | **Middle-end readiness** — unlocks `cosmic-mir` / `cosmic-opt` | Every Tier-M operation has an ACCEPTED NORMATIVE RULE and mechanically specified verifier obligations; otherwise defer that operation |
| **G1-R** | **Backend readiness** — unlocks `cosmic-runtime` / codegen | S16 and S17 decided; the per-target capability table schema exists |
| **G1c** | Owner: identity, compatibility | **Extension decided: `.cmic`** (2026-10-16, ADR-0002 §4.1). Identity naming and compatibility still require owner approval or explicit component-limited deferral; the parser is independent of filename extension. No release, announcement or MIME registration is authorized; source publication in this public repository is governed by G1d |
| **G1d** | Owner: licence and publication | Owner licensing decision **and** the public/private development choice: **L-A** (public development under an established licensing and contribution policy; the review requirement is governed by the amended G1d procedure — mandatory qualified-counsel review replaced by owner risk acceptance, 003-L3) or **L-B** (development exclusively in a genuinely private repository or local environment) — see `docs/spec/phase-2-entry-readiness.md` §3. No legacy artifacts imported. A branch or PR in this public repository publishes source even without a release |
| **G1e** | CI and governance | CI green on the branch; protection verified |

> **002-R4 correction (2026-10-16).** The G1c and G1d rows previously said
> "no publication" and offered an "internal-development-only deferral" while
> Directive 003 directed implementation onto a public branch. That framing was
> factually inconsistent: in a public repository, a pushed branch or pull
> request publishes the source, with or without a binary release. The rows above
> now state the two owner options (L-A/L-B); the licence itself remains
> undecided.

> **003-G1 owner acceptance record (2026-10-16).** The owner accepted the Stage A entry gates and recorded the G1d hold:
>
> 1. **G1 — ACCEPTED**, limited to the corrected Phase 1 recognition basis: the 122-production grammar and the machine-checked 122/122 evidence map; 111 `RECOGNISED` / 10 `UNSUPPORTED` / 1 `REJECTED`; corrections F1/F2/F4/F5 recognized. Unsupported syntax must produce explicit diagnostics; an OPEN semantic rule is not silently adopted by recognition; the language specification is not globally frozen.
> 2. **G1-L — ACCEPTED, ENTERED** for Stage A lexical readiness: D-L1…D-L10 and the LEX001–LEX010 allocation plan (`docs/spec/g1-l-lexical-decision-package.md`). The strict unknown-escape and lone-surrogate rules are accepted intentional compatibility divergences whose corpus impact must still be measured; Unicode classification edge cases (M-7) remain measurement obligations.
> 3. **G1-S — ACCEPTED, ENTERED** for Stage A syntactic readiness: the recognition subset, the nine-family refusal contract, the lambda/parenthesis and trailing-lambda rules, the recovery obligations and CST round-trip invariants, and the PARSE allocation plan (`docs/spec/g1-s-syntax-decision-package.md`). Pattern recognition is not pattern-semantic approval; typed HIR, overload resolution, conversions, evaluation order and runtime behaviour are not approved.
> 4. **G1c — SATISFIED** through an explicit Stage A-limited deferral: `.cmic` remains the binding extension decision; the parser stays extension-independent; MIME registration, editor/LSP publication and release-level naming/compatibility decisions are deferred; releases and distribution-policy changes are not authorized.
> 5. **G1d — remains PENDING**: the owner selected **L-A** (public development) and the intended outbound licence **Apache-2.0**, conditional on rights verification and qualified legal review; the licence has not been applied or granted and no legal clearance has been obtained (conditions: `docs/spec/phase-2-entry-readiness.md` §5 item 6).
>
> **None of these acceptances authorizes implementation.** Stage A additionally requires this gate-acceptance record on `main` (merged via its reviewed PR, post-merge CI green), **G1d clearance**, and a separate owner authorization of the Stage A scope; Stage B remains separate and requires G1-T.

> **003-L3 owner risk-acceptance record (2026-10-16).** The owner accepted Option B of Directive 003-L2 §F and amended the internal G1d governance procedure: mandatory qualified-counsel review is replaced by the recorded owner risk acceptance — (1) the owner's documented copyright ownership self-declaration; (2) acceptance of the verified technical provenance findings (003-L1/003-L2); (3) explicit acknowledgment of unresolved legal uncertainty; (4) express acceptance of the documented residual risks; (5) explicit acknowledgment that no qualified lawyer has reviewed or cleared the project; (6) retention of the GPL-3.0-material and external-contribution restrictions. The full record is `docs/spec/phase-2-entry-readiness.md` §9. This is an owner governance decision — **not** a legal opinion, copyright determination or waiver of third-party rights.
>
> **G1d status: AMENDED PROCEDURE APPROVED; LICENSING IMPLEMENTATION PENDING.** Reserved future result (not assigned): **G1d — CLEARED UNDER OWNER-ACCEPTED RISK PROCEDURE (NO QUALIFIED-COUNSEL REVIEW)** — the transition requires the owner-authorized Apache-2.0 licensing and contribution package to be applied and validated, with any unresolved technical or ownership blocker reported. The original qualified-counsel requirement remains preserved in the dated 003-G1 record above and in the 003-L1/003-L2 reports; nothing here claims a legal clearance.

> **003-L4 owner authorization and licensing-package record (2026-10-16).** The owner expressly authorized publication of the Apache-2.0 licensing package on branch `legal/apache-2.0-adoption` (committed, pushed and validated in the public repository; **merging requires a separate owner approval**), approved the **Developer Certificate of Origin (`Signed-off-by`)** as the inbound contribution policy, and approved the public copyright attribution **Copyright 2026 CosmicLang**; the 003-L3 risk acceptance (Option B) was reaffirmed and the provenance restrictions retained (no GPL-3.0 legacy imports or mechanical translation; independently authored implementation and tests; rights to contribute; escalation of suspected provenance violations). The package is applied on the draft licensing PR: root `LICENSE` (complete, unmodified Apache License 2.0 text), `license = "Apache-2.0"` in `[workspace.package]` inherited by the three crates, the README licensing declaration, the `CONTRIBUTING.md` DCO contribution policy with dependency-licence review, and the governance-record updates.
>
> **G1d does not transition yet.** The internal result changes to **G1d — CLEARED UNDER OWNER-ACCEPTED RISK PROCEDURE (NO QUALIFIED-COUNSEL REVIEW)** only when the package is integrated and validated, with no unresolved technical or ownership blocker reported. That result certifies compliance with the project's amended internal governance procedure only — it is **not** legal advice, **not** professional legal clearance and does **not** eliminate third-party rights, and it must never be represented as any of those. Stage A and every Phase 2 component remain **NOT AUTHORIZED** (003-L4 §4, §6).

> **003-A G1d transition and Stage A authorization record (2026-10-17).** The
> owner-authorized Apache-2.0 licensing and contribution package was integrated
> by the owner (PR #7, merge commit `f03d70485fdb196c58c4d9eec62240337af8884f`,
> merged 2026-10-17T02:23:07Z) with post-merge `Validate` run `38994709806`
> green on the merge commit and no unresolved technical or ownership blocker
> reported. The reserved internal result is therefore assigned:
> **G1d — CLEARED UNDER OWNER-ACCEPTED RISK PROCEDURE (NO QUALIFIED-COUNSEL
> REVIEW)**. It certifies compliance with the project's amended internal
> governance procedure only — it is **not** legal advice, **not** professional
> legal clearance, does **not** eliminate third-party rights and must never be
> represented as any of those. On that basis the owner authorized **ENGINEERING
> DIRECTIVE 003-A** (Phase 2, Stage A — lexical and syntactic frontend),
> starting with increment A1 (`cosmic-lexer`) on `phase/2-frontend`; **Stage B
> remains NOT AUTHORIZED** and still requires G1-T entry plus a separate owner
> authorization.

### 5.1 The cycle, resolved

| Gate | Status after Directive 002-R3 |
|---|---|
| G1 | **ACCEPTED** (owner, 003-G1 §2, 2026-10-16) — corrected 122-production grammar + machine-checked 122/122 map; 111/10/1 classification; F1/F2/F4/F5 recognized; unsupported syntax refused with explicit diagnostics; recognition does not adopt open semantics |
| **G1-L** | **ACCEPTED · ENTERED** (owner, 003-G1 §3, 2026-10-16) — D-L1…D-L10 and the LEX001–LEX010 allocation accepted; corpus-impact and M-7 measurements remain owed |
| **G1-S** | **ACCEPTED · ENTERED** (owner, 003-G1 §4, 2026-10-16) — recognition subset, nine-family refusals, lambda/trailing-lambda rules, recovery and CST invariants, PARSE allocation; S-G1S-1 resolved (F5; `docs/spec/g1-s-syntax-decision-package.md` §3.3); semantics stay per the S records |
| G1-T | **FRAMEWORK ACCEPTED · ENTRY PENDING** — every Tier-T record is closed or gated; the per-operation gates are specified in §4.1; acceptance outstanding |
| G1-M | **FRAMEWORK ACCEPTED · ENTRY PENDING** — S11 JVM behavior is measured; S18 invariant proposals still require acceptance |
| G1-R | **FRAMEWORK ACCEPTED · ENTRY PENDING (OWNER)** — S16/S17 undecided. Does not gate Phase 2 or Phase 3 |
| G1c / G1d | **G1c SATISFIED via Stage A-limited deferral** (owner, 003-G1 §5) — extension `.cmic`; parser extension-independent; MIME/editor/release-level decisions deferred. **G1d — CLEARED UNDER OWNER-ACCEPTED RISK PROCEDURE (NO QUALIFIED-COUNSEL REVIEW)** — the licensing package (owner authorization 003-L4, 2026-10-16) was integrated and validated (PR #7, merged 2026-10-17T02:23:07Z; post-merge CI green): L-A selected; the Apache-2.0 licence text, Cargo metadata and inheritance, README declaration, DCO contribution policy and attribution are in force; the status is an internal risk-accepted governance result, **not professional legal clearance** (`docs/spec/phase-2-entry-readiness.md` §5 item 6, §9) |
| G1e | **PRIOR SHA PASSED** (PR #7 post-merge CI green at `f03d704`) — verify protection details and CI on the Stage A draft-PR head and on each subsequent increment |

**Therefore: the gate framework is accepted (002-R3); the Stage A entry gates
G1, G1-L and G1-S are accepted with the G1c deferral approved (003-G1); the
internal G1d procedure is amended under the owner's risk acceptance (003-L3);
the authorized licensing package was integrated and validated (PR #7; post-merge
CI green) and the internal G1d result is **CLEARED UNDER OWNER-ACCEPTED RISK
PROCEDURE (NO QUALIFIED-COUNSEL REVIEW)** — an internal governance status, not
professional legal clearance — and the owner authorized Directive 003-A for the
Stage A frontend (increment A1: `cosmic-lexer`; 2026-10-17). Stage B still
requires G1-T entry and a separate owner authorization.** The dependency chain
that made authorization impossible is gone, and it was removed by (a) measuring what
could be measured, and (b) scoping each gate to the component that consumes the
decision — not by relaxing any standard.

### 5.2 What has *not* changed

* **Every open question is still open** unless the probe record closed it. The
  gate change moves no decision.
* **No `GATED` record is decided.** A gate is a refusal, not a rule.
* **No S1–S18 semantic question moved.** The 003-G1 acceptances are lexical and
  recognition-layer decisions; S5's pattern semantics, every Tier-T/M/R
  disposition and every measurement debt keep their recorded state.
* **The owner-blocked decisions are still owner-blocked** (S3, S4, S16, S17,
  the MIME type, the naming) — with completions now recorded: the source
  extension is `.cmic` and the G1c Stage A-limited deferral is approved
  (003-G1 §5); the licence/publication hold is resolved under the amended G1d
  procedure (owner risk acceptance 003-L3; owner authorization 003-L4) — L-A
  selected; Apache-2.0 applied, integrated and validated (PR #7); the internal
  G1d CLEARED transition is recorded (003-A, 2026-10-17) as an internal
  risk-accepted result, not professional legal clearance.
* **Stage A is authorized; nothing semantic moved.** Directive 003-A
  (2026-10-17) authorizes the Stage A lexical/syntactic frontend only
  (increment A1: `cosmic-lexer`); Stage B still requires G1-T entry and a
  separate owner authorization.
* **Phase 3 and Phase 4 remain blocked** on G1-M and G1-R respectively.
* **G0-B remains PARTIAL.**

---

## 6. Remaining measurement debt

Small, named, and no longer blocking anything it should block.

| # | Debt | Blocks | Instrument |
|---|---|---|---|
| M-1 | Overload tie-break: is it *strictly* declaration order? (one ordering probed) | S1 Q5 | A reversed-declaration probe — one run |
| M-2 | Map-key / set contract for a key type whose equality is reference-based | S12 Q7 | A probe using a class key in a `Set`/`Map` |
| M-3 | Enum equality and `values()` identity | S12 Q6 | A probe comparing enum constants |
| M-4 | Nested-function capture *intent* (the probe shows rejection, not design) | S11 Q7 | An owner or maintainer statement; **not** an experiment |
| M-5 | Division by zero (integer and floating) | S7 Q3 | A probe per numeric type |
| M-6 | Cross-target behaviour of every probe in this record | cross-target risk | Re-running the probes on `--target js` and `--target script` |
| M-7 | Identifier class boundaries beyond letters, `_`, `$` | S13 Q1 | Probes with combining marks and digits-in-start |

**M-6 is the most important debt in this table.** Every probe ran on the JVM
target only, and the Phase 0 audit already established that a passing corpus
does not guarantee cross-target agreement (the shift-semantics divergence). No
compatibility claim may rest on JVM-only evidence.

**A3 measurement update (2026-10-17).** The Stage A verification increment
executed the M-7 identifier-class probes against the accepted frontend (34/34
pass; `docs/audits/phase-2-a3-probes.md`) and recorded cross-authority context
for M-6 (`docs/audits/phase-2-a3-m6-cross-target.md`) plus reproducible
frontend measurements (`docs/audits/phase-2-a3-measurements.md`). The legacy
oracle remained unavailable and was never downloaded or vendored: the M-6
target re-runs and every legacy-side cross-check of M-1…M-7 are **NOT
MEASURED**, so M-7 is measured against the accepted authority only and D-L3
is **not** settled against legacy behaviour. These debt items remain open.

---

## 7. Gate observability requirement

A gate that is not observable cannot be audited. Every gate hit must:

1. name the **construct** (e.g. "nested type argument"),
2. name the **decision record** (e.g. `S1`),
3. name the **unsupported operation** (e.g. "resolve a parameterised type
   argument whose own argument is parameterised"),
4. state what would close it ("requires S1 Q1 to be decided"),
5. carry a stable diagnostic code in the `GATE` namespace
   (`docs/spec/diagnostic-codes.md` §3),
6. produce **no artifact**.

Requirement 6 is what distinguishes a gate from a warning: a program that hits a
gate does not compile.

---

## 8. Summary

| Before | After |
|---|---|
| One gate (`G1b`) over all 18 questions | Six component-scoped gates (G1, G1-L, G1-S, G1-T, G1-M, G1-R) |
| "Decided or owner-dependent" | "EVIDENCED, GATED, DEFERRED or OWNER" — a `GATED` disposition is complete |
| Measurement required before the phase that produces it | Measurement is a deliverable of the phase; only the *ability* to measure is a prerequisite |
| A parser that rejects syntax whose semantics are open | A parser that accepts the whole grammar; gates fire at the semantic operation |
| Category-level gating | `(construct, record, operation)` gating |
| Phase 2 blocked | **Component readiness can be approved separately**; parser recognition is distinct from acceptance; MIR and runtime remain dependent on their own normative gates |

**No technical standard was lowered.** The two questions that caused the cycle
were answered by experiment, and the remaining open questions are now gated
rather than blocking.
