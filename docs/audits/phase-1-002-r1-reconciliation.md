# Phase 1 — Directive 002-R1 Reconciliation and Owner Decision Package

| | |
|---|---|
| **Status** | **Engineering reconciliation — no owner acceptance is granted by this document.** |
| **Directive** | ENGINEERING DIRECTIVE 002-R1 |
| **Repository** | `CosmicLang/cosmic-lang` |
| **Branch** | `phase/1-specification` |
| **Base** | `main` @ `8f0944cd9ac1f30c835482b086a6dab300b5f1f0` |
| **PR** | #2 — open, draft (unchanged) |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |
| **Related** | `docs/spec/implementation-gates.md`, `docs/audits/phase-1-legacy-probes.md`, `docs/spec/grammar-consistency-report.md`, `docs/audits/phase-1-provenance-controls.md`, `docs/directives/directive-003-proposal.md` |

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

This document is the reconciliation record that `docs/audits/phase-1-specification-report.md` forwards to: that report describes the initial Directive 002 draft; it is **historical and superseded for current decisions**. This record states the corrected state, the evidence, and the explicit owner decisions that remain.

**Nothing in Phase 1 has been approved by the owner.** The gate changes, the normative-status corrections, the grammar, the decision register and every proposed rule are **PROPOSED** until the owner acts on §10.

---

## A. Verified repository and governance state

* **HEAD** of `phase/1-specification` is `18a9255835ab904d762577f9a496512f21e808e8` — matches the directive's expected HEAD.
* **Base** `main` is `8f0944cd9ac1f30c835482b086a6dab300b5f1f0` — matches.
* **PR #2** is open and in **draft**, head `phase/1-specification`, base `main`, mergeable. It remains open, draft and **unmerged**.
* **Classic `main` protection is active and unchanged** (verified via the GraphQL/`repos/{owner}/{repo}/branches/main/protection` endpoint): pull requests required; required status check `Validate` (GitHub Actions, app id 15368) with `strict: true`; `enforce_admins: true`; **force-pushes prohibited**; **deletions prohibited**; required conversation resolution. No protection change was made or authorised by this directive; none is needed.
* **Direct push to `main` remains restricted** by that protection (a direct fast-forward push was rejected empirically during Phase 0 preflight, and the protection has not been altered since).
* **CI** on the current SHA `18a9255` passes (push + PR runs, conclusion `success`) — see §H for the post-correction SHA.

## B. Root cause and resolution of the G1/G1b/Phase 2 cycle

### B.1 Root cause

The original `G1b` gate read **"all 18 questions decided or explicitly owner-dependent"**, and Phase 2 was gated on G1–G1e. Two records — **S1** (generics/overloads) and **S11** (evaluation order) — were **BLOCKED-ON-MEASUREMENT**, and the measurement instrument (the differential harness) was assigned to Phase 2. That is a closed dependency cycle:

```
G1b requires S1 and S11 decided
        ^                            S1/S11 require measurement
        |                            |
        |                            v
Phase 2 cannot start  <---------  measurement happens in Phase 2
```

The cycle is real and fatal under the original wording, and it is **an artefact of how the gate was written**, not of any genuine technical dependency. Three modelling errors, each independently sufficient:

1. **One gate over questions with different consumers** — a single `G1b` could not distinguish lexical, semantic, middle-end, runtime and owner questions, so every open question blocked every component.
2. **Measurement scheduled in the phase it was blocking** — a measurement is a *deliverable* of the phase that can produce it, not a *prerequisite* of that phase. The prerequisite is the ability to measure.
3. **"Decided" conflated with "settled"** — an implementation can be correct while a question stays open, **provided the compiler refuses the affected operation instead of guessing**. Gating on "decided" is exactly the pressure that would make an engineer invent a rule.

### B.2 Resolution — without lowering any standard

`docs/spec/implementation-gates.md` replaces the flat `G1b` with **component-scoped entry gates**:

| Gate | Unlocks | Gating principle |
|---|---|---|
| G1-L | `cosmic-lexer` | lexical contract accepted; lexical open cases gated |
| G1-S | `cosmic-syntax` (parser/CST) | recognition subset accepted; un-evidenced syntax refused with PARSE diagnostics |
| G1-T | `cosmic-sema` (typing/HIR) | per-operation accepted rules; the rest gated per operation |
| G1-M | `cosmic-mir` / `cosmic-opt` | every middle-end operation has an accepted rule, else deferred |
| G1-R | `cosmic-runtime` / codegen | S16/S17 decided; capability-table schema exists |

Three principles resolve the cycle:

* **Recognition is separable from acceptance** — the lexer/parser recognise the full evidenced grammar; gating happens where semantics are consumed. A parser may recognise a construct whose semantics are open, provided semantic compilation does not invent behaviour.
* **Gates name a `(construct, decision record, unsupported operation)` triple** — feature gates apply to the specific unsupported semantic operation, never to an entire syntax category.
* **Measurements are deliverables, not prerequisites** — the *ability* to measure gates a phase; the *result* does not.

**Notably, most of the measurement debt is now moot:** Directive 002-R1 §4 authorised read-only probes and `docs/audits/phase-1-legacy-probes.md` records them (see §C). S11 and most of S1 are no longer measurement-blocked; the residual debt is small and named (`implementation-gates.md` §6).

**What has not changed:** every open question is still open unless a probe closed it; no `GATED` record is decided (a gate is a refusal, not a rule); owner-blocked decisions (S3, S4, S16, S17, licence, extension, naming) are still owner-blocked; Phase 3/4 remain gated on G1-M/G1-R; **Phase 2 is not authorised**.

## C. Experimental evidence — S1, S8, S11, S12, S13

All probes executed **read-only** against the frozen legacy compiler (`--target jvm`, OpenJDK 27), in an isolated clone at the pinned SHA, on 2026-10-16. Full commands, inputs, outputs, exit statuses and interpretations are in `docs/audits/phase-1-legacy-probes.md`. **Defects are recorded as defects, never promoted to rules.** Probe programs were written for the experiment and were **not** copied into this repository.

| Record | Result | Evidence class |
|---|---|---|
| **S11** | Arguments left-to-right; receiver before arguments; binary operands left-to-right; `&&`/`\|\|` short-circuit; **assignment is a statement** (`SEM027`, so chained-assignment associativity does not exist); block-scoping with shadowing; nested functions **do not capture** enclosing locals (`SEM011`) | **VERIFIED LEGACY BEHAVIOR** on sampled JVM forms; capture *intent* and target parity stay OPEN |
| **S12** | Strings value-equal; records component-wise; classes reference-equal; `NaN != NaN`; `-0.0 == 0.0` with sign preserved (`-Infinity` prints); printing contract extended with `NaN`/`-Infinity` | **VERIFIED LEGACY BEHAVIOR** for four type kinds + float corners; enums and map/set contract NOT VERIFIED |
| **S8** | Triple-quote rejected (`LEX008`); **raw newline inside `"…"` permitted**; unknown escape **silently dropped with no diagnostic** (recorded defect; strict-rejection is PROPOSED, a C2 divergence) | **VERIFIED LEGACY BEHAVIOR** for three string cases; Q2's fix is PROPOSED NORMATIVE RULE |
| **S13** | Non-ASCII letters valid identifier chars; `_`/`$` starts valid; diagnostic columns count **UTF-16 code units** — divergence from scalar counting measured and quantised as +1 per astral character | **VERIFIED LEGACY BEHAVIOR**; scalar columns remain PROPOSED (unchanged), a documented intentional divergence |
| **S1** | `Box<Box<Int>>` **compiles then crashes** (`NoSuchMethodError`); inference inside an enclosing generic call **drops a nested type argument** (`List<Box>`); `<` is a **left-associative comparison chain**, never a generic application; specificity picks the most specific overload; **ambiguity is silently resolved by declaration order** in the one probed ordering | **VERIFIED LEGACY BEHAVIOR** — defects recorded, never promoted to rules; tie-break generality, bounds, variance and inference design stay OPEN |
| **S2** (bonus) | `expr as T?` is a **soft cast**: a failed cast yields `null`, it does not throw | **VERIFIED LEGACY BEHAVIOR**; conversion rules beyond this stay OPEN |

**Independent reproduction.** Every probe in that record was re-executed before this reconciliation was committed — same frozen jar, same pinned SHA, a fresh copy of the probe set. All stdout, stderr and exit statuses reproduced. **One correction was applied:** `kof check` writes diagnostics to **stdout**, not stderr; the affected entries were fixed and the subcommand stream difference is now recorded as its own finding (probe record §7.4). No observed behaviour changed, and the correction is disclosed rather than silently edited.

**What the probes do not establish.** One target only (JVM); one overload ordering; enums and map/set equality unprobed; the capture *intent* is not established; cross-target (js/script) parity is **NOT VERIFIED**. The remaining measurement debt is named, bounded and itemised in `docs/spec/implementation-gates.md` §6 (M-1…M-7). **No probe result is an accepted rule.**

---

## D. Grammar consistency and provisional-production classification

**Instrument.** `tools/spec-consistency.py` — documentation tooling, not compiler code — runs 13 checks (G0–G12): production parseability, no undefined nonterminals, no unreachable productions, no duplicate definitions, block-level evidence markers, open-marker traceability, register completeness, status vocabulary, gate definitions, internal links, repository paths and the normative vocabulary. It exits non-zero on any failure. **Current result: all checks pass** (raw output in §H).

### D.1 The directive's "59 productions" — measured, corrected

The directive and the historical Phase 1 report both describe the delivered grammar as **"59 productions"**. A mechanical parse of the file at HEAD `18a9255` finds **97 production statements** and **37 findings**: one unparseable statement, **24 undefined nonterminals** (`digit`, `hex-digit`, `unicode-letter`, `any-char`, `char-literal`, `if-expr`, `array-literal`, `record-body`, `for-init`, `function-signature`, …) and **12 unreachable productions** (`unary-expr`, `binary-op`, `whitespace`, `comment`, `line-terminator`, …). A lexer cannot be derived from that file and a parser cannot reach those productions. The reproduction command and the full finding list are in `docs/spec/grammar-consistency-report.md` §3.1.

The grammar was completed and unified to **117 productions**. The historical "59" is superseded; the ratio does not indicate new language surface, it indicates **missing definitions and lexical plumbing** that were cited but never written. The statement corruption had a concrete root cause: a **malformed backslash terminal** (`"\"` where ISO EBNF requires `"\\"`) plus mixed `"`/`'` delimiters, which unbalanced the quote-aware splitter, merged seven productions into one statement, and exposed fragments (`true`, `false`, `null`, `n`, `t`, `r`, `u`) as spurious symbols. The corrected grammar uses one quoting convention and the checker now parses every statement.

### D.2 Classification of the provisional productions

Per `docs/spec/grammar-consistency-report.md` §4, every provisional production is in exactly one of three classes:

1. **Recognised now; semantics gated per operation** (the parser accepts, the semantic stage refuses only the unsupported operation): nested type arguments at depth ≥ 2 (S1), `?` conversions beyond `T → T?` (S2), `switch` exhaustiveness/fallthrough (S5), literal promotion and division by zero (S7), enum and map/set equality (S12), confusable-identifier policy (S13). `Box<Int>` and unambiguous overloads compile; only the contested operation is gated.
2. **Syntax marked unsupported — refused until recognised** (nine form families): `test`/`application`/`infra`/`extern`/`foreign module`/`entity` declarations, unknown annotations, labelled `break`/`continue`, array literals. These lack sufficient evidence for independent recognition and receive **PARSE refusals** naming the construct.
3. **Questions that dissolved rather than being answered** (from the probes): assignment "associativity" (assignment is a statement, `SEM027`); evaluation order (a semantic question, now sampled); nested-function capture (a semantic question; recognition is independent).

**This three-way split is the directive's "recognition ≠ acceptance" principle made operational**, and it is what allows the lexer and parser to be specified while S1/S11 remain technically open.

### D.3 Ambiguity and boundary findings

* **Generics versus comparison is resolved by evidence:** `a < b > c` is a left-associative comparison chain (probe §6.3); the grammar now reaches `type-arguments` only from `type-ref` and the evidenced generic-call postfix, never as a `<` fallback in expression position.
* **One residual lookahead requirement, recorded not resolved:** `primary` admits both `(expression)` and `lambda` with no disjoint first terminal; the legacy parser uses unbounded lookahead (`ExpressionParser.parsePrimary`, `LambdaParser`). The corrected grammar does **not** encode that lookahead, and the report says so (§5.3). **This is the single most important parser-design constraint handed to Phase 2.**
* **Four lexical/grammar boundary defects fixed** (report §6): layout productions now reachable from `source`; `identifier` excludes keywords; `type-ref` admits `primitive-type`; the raw-newline description matches the probed legacy behaviour (normative policy remains S8's).
* **Deliberate weakening, disclosed:** evidence traceability is **block-level** (10/10 production blocks carry a citation) and **not per-production** (0/117 carry their own inline marker). The owner may require per-production markers; today's convention is recorded rather than hidden.

---

## E. Source-extension candidate comparison (`.csmx` and alternatives)

**Status: pending formal acceptance. No extension or MIME type is approved.**

**The collision is confirmed and material.** The repository `akeit0/csharp-markup-extension` (C# with JSX-like syntax, MIT, 16 stars, created 2026-06-11, last push 2026-06-15) contains **nine `.csmx` source files, including language-server test fixtures** and a language-server component — an active competing **programming-language** claim in the same extension namespace. It was found by authenticated code search, which is the channel that distinguishes real format use from web-page strings; the round-1 assessment missed it. Its status was independently re-verified during this directive (repository metadata and file list). It remains a PoC-scale project, but it is a tooling-level collision, not noise.

**Candidate matrix** (`docs/spec/extension-assessment.md` §8; GitHub counts are indexed public-file matches at the stated instant, **not a census**):

| Rank | Candidate | Competing language-source use | Registry/editor evidence | Residual uncertainty | Migration cost |
|---|---|---|---|---|---|
| 1 | `.cosmic` | None found in sampled hits (58 indexed files are config/data suffixes: `Dockerfile.cosmic`, `pdb.cosmic`, …) | Exact registry entry NOT VERIFIED; marketplace association NOT VERIFIED | Unindexed tooling; no exclusivity | Medium |
| 2 | `.cmic` | None located (0 indexed hits; absence ≠ proof) | NOT VERIFIED | Short-suffix unfamiliarity | Medium |
| 3 | `.ccos` | None located (0 indexed hits) | NOT VERIFIED | Confusion with `.cos` | Medium |
| 4 | `.csmx` | **Confirmed competitor** (`akeit0/csharp-markup-extension` + language server) | No registry entry confirmed; no exclusive claim | PoC uptake; LSP activation untested | Low now, high after release |
| 5 | `.cos` | **Confirmed** Creatures CAOS scripting source; ~7,600 indexed matches | Direct FileInfo entry (Capture One Settings + Creatures CAOS) | Mixed unrelated formats | High |
| 6 | `.csm` | CASL/other established formats; ~6,736 indexed matches | Not exhaustively validated | Multiple mature associations | High |

**Priority criterion:** competing **language-source and editor/LSP ownership** outranks raw file counts — which is why `.cosmic` is ranked above `.csmx`. **Worldwide uniqueness is not established for any candidate**, and no candidate may be pronounced available on this evidence.

**MIME:** no IANA-registered type contains `cosmic`, `csmx`, `kof` or `csm`. `text/x-cosmic-lang` is `x-`-prefixed — usable immediately, **unregistrable and permanently provisional**; a real registration would need the vendor (`application/vnd.cosmic-lang`) or personal tree. **The MIME type is not approved.**

**Recommendation (PROPOSED):** keep `.csmx` unaccepted; if the owner wants a decision now, `.cosmic` is the leading proposal with `.cmic` as the concise fallback and `.ccos` as the conservative fallback; the parser must never depend on the extension (it does not — the grammar has no extension coupling). A change of extension later is a migration (renames, tooling, LSP association, docs), not a parser change. Remaining cheap follow-ups: direct editor-marketplace and package-registry queries, and a trademark search (§5 of the assessment; the last is a legal question).

---

## F. Corrected normative decision register

Authoritative statuses live in `docs/spec/decisions/README.md` and each record. **No record is ACCEPTED.** The six statuses are per-question; a record may carry several.

| # | Record | Status after 002-R1 |
|---|---|---|
| S1 | Nested generics / inference / overloads | **OPEN** · VERIFIED LEGACY BEHAVIOR (defects) · **GATED** (depth ≥ 2, ambiguous overloads); tie-break NOT VERIFIED |
| S2 | Nullability and conversions | **OPEN** · VERIFIED LEGACY BEHAVIOR (soft cast defined) · conversions **GATED** |
| S3 | PortuKof | **OPEN** (owner scope/legal choice) · `.ptkf` refused loudly meanwhile |
| S4 | KofScript | **OPEN** (owner scope/legal choice) · `.ks` refused loudly meanwhile |
| S5 | Pattern matching | **OPEN** · NOT VERIFIED (exhaustiveness, fallthrough) · **GATED** per check; labelled `break` refused as unsupported syntax |
| S6 | Concurrency | **OPEN** (typing) · **DEFERRED** (runtime capability) |
| S7 | Numeric semantics | **OPEN** · VERIFIED (wrapping, formatting) · NOT VERIFIED (division by zero, promotion) · **GATED** |
| S8 | String semantics | **VERIFIED LEGACY BEHAVIOR** (Q1/Q4; silent escape defect) · **PROPOSED NORMATIVE RULE** (strict unknown escapes) · **OPEN** (policy acceptance) |
| S9 | Stdlib boundary | **OPEN** · NOT VERIFIED (member boundary) · module forms parser-evidenced |
| S10 | Reserved words / annotations / FFI | **OPEN** (owner scope) · recognition evidenced · unsupported constructs refused specifically |
| S11 | Evaluation order and binding | **VERIFIED LEGACY BEHAVIOR** (seven sampled questions) · **PROPOSED NORMATIVE RULE** (left-to-right, short-circuit, statement assignment, block scoping) · **OPEN** (capture intent, target parity) |
| S12 | Equality | **VERIFIED LEGACY BEHAVIOR** (strings, records, classes, float corners) · **NOT VERIFIED** (enum, map/set) · **OPEN** |
| S13 | Unicode | **VERIFIED LEGACY BEHAVIOR** (identifier classes, UTF-16 columns) · **PROPOSED NORMATIVE RULE** (scalar columns, documented divergence) · **OPEN** (confusables policy) |
| S14 | Trivia | **PROPOSED NORMATIVE RULE** (lossless CST) |
| S15 | Feature gates | **PROPOSED NORMATIVE RULE** (gate vocabulary and observability contract — `implementation-gates.md` §7) |
| S16 | Memory strategy | **DEFERRED** · **OPEN** (owner choice) |
| S17 | ABI versioning | **DEFERRED** · **OPEN** (owner choice) |
| S18 | Typed-IR invariants | **PROPOSED NORMATIVE RULE** (TI-1…TI-8, MI-1…MI-8) |

Gate mapping per record: `docs/spec/implementation-gates.md` §4. Tier assignment: L (lexer): S8, S13, S14 · S (parser): recognition subsets of S2, S5, S9, S10, S15 · T (sema): S1, S2, S5, S6, S7, S11, S12, S18 · M (MIR): S11, S18 · R (runtime): S6, S16, S17 · O (owner): S3, S4.

---

## G. Licensing and provenance

**The licence remains undecided; no LICENSE file exists; no licence field was added to any manifest.** This document makes no choice.

**Provenance correction — the term "clean-room" was misleading.** The specification was built **after direct human and agent inspection of the legacy Java source**, plus executed probes against the legacy compiler. Reading the original implementation and deriving a specification from it does **not** establish a formally isolated clean-room process; a clean-room claim would require documented researcher/implementor separation and an information-transfer protocol that does not exist here. The accurate description is **independently authored, behaviour-guided reimplementation** — and even that phrase is a factual engineering description, **not** a copyright or licence conclusion (see `docs/audits/phase-1-provenance-controls.md`). The provenance-control language in the ADRs, the licensing recommendation and the README was corrected accordingly.

**Engineering provenance controls now in force (proposed normative):**

1. **`CONTRIBUTING.md`** states the import prohibition: no legacy code, fixtures, generated outputs or protected documentation may be pasted, mechanically translated, bundled or redistributed. The legacy executable may be consulted only as an external read-only oracle.
2. **Evidence discipline:** every semantics-changing change cites the immutable source path + SHA, the executed target, observed output and exit status, and the affected decision record.
3. **The differential harness** (Phase 2 design) runs newly authored programs against a separately obtained legacy executable as a black box and must not vendor legacy fixtures; when the toolchain is absent it skips with a stated reason.
4. **Dependency licences** must be recorded when the first dependency is added (toolchain policy §3).
5. **Uncertain provenance is escalated** for human review before commit.

**Licensing recommendation (engineering, non-binding):** keep the decision open until **qualified counsel** rules on the ten legal dependencies (L1–L10 in `docs/audits/phase-1-licensing-recommendation.md` §5) — especially **L1** (is an independently written reimplementation of the same language a derivative work?), **L9/L10** (do behaviourally derived PortuKof/KofScript specifications create derivative works?) and **L6/L8** (black-box execution and CI logs). Among the options: **A (GPL-3.0)** is the cheapest path *if* the owner intends to reuse legacy-derived vocabulary/semantics and accepts copyleft; **B (permissive, e.g. Apache-2.0 or MIT)** preserves adoption and matches the project's independent-authorship posture, but rests on L1; **C (other copyleft)** requires the same review first; **D (stay unlicensed)** keeps all rights reserved and blocks distribution and external contribution, but does not make the already-public repository private. Engineering's stated preference for a published project is **B with an express patent grant**, contingent on L1 review — **the owner decides, and nothing is selected automatically.**

---

## H. Executed tests and CI evidence

### H.1 Rust validation (local, this directive)

| Check | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all -- --check` | **clean** |
| Lints | `cargo clippy --workspace --all-targets -- -D warnings` | **clean** (exit 0) |
| Tests | `cargo test --workspace --all-targets` | **58 passed, 1 ignored, 0 failed** |
| Documentation | `cargo doc --workspace --no-deps` | **clean** |

Toolchain: Rust 1.99.0 (pinned by `rust-toolchain.toml`, matched by CI). No compiler source changed in this directive; the workspace remains the three Phase 0 crates.

### H.2 Specification consistency (the §10 link/evidence/gate validation)

`python3 tools/spec-consistency.py` — all 13 checks pass. The captured run is in `docs/spec/grammar-consistency-report.md` §3 and covers exactly what the directive requires: decision-register links (G6/G7), grammar evidence references (G4/G5), legacy source references (via the `cosmcc:` prefix rule, G11) and gate dependencies (G9). The checker is deterministic; anyone can re-run it.

### H.3 Legacy probe reproduction

Every probe in `docs/audits/phase-1-legacy-probes.md` was executed twice (original + independent reproduction) against `CosmicLang/cosmcc` @ `284358f4…`, `--target jvm`, OpenJDK 27, with identical outputs and exit statuses; one stream-attribution correction (§C above). Probe inputs live only in isolated temporary directories and are recorded in the probe document as text — **no legacy code or fixture was copied into this repository**.

### H.4 CI on the pushed SHA

CI (`Validate`: fmt + clippy + test + doc, pinned actions, run on push and PR) is verified on the exact final push of this directive; run IDs and conclusions are recorded on PR #2. The pre-directive SHA `18a9255` was green (push run `38934038793`, PR run `38934066056`); the final-SHA result is appended to the PR conversation, not self-referenced here.

---

## I. Final PR #2 status

PR #2 (`phase/1-specification` → `main`) remains **OPEN** and **DRAFT** — unchanged in state by this directive. Base `main` @ `8f0944c`; branch protection verified unchanged (PRs required; `Validate` required, strict; admins enforced; force-pushes and deletions prohibited; conversation resolution required). No force-push was used; all commits are ordinary additions on top of `18a9255`. **Nothing was merged; nothing was marked ready for review; no protection setting was modified.**

---

## J. Owner decision package

Each item states what engineering has done, what only the owner can do, and the options. **None of these is approved by this document.**

### J-1. The gate model (unblocks or perpetuates the cycle)

* **Question:** replace the flat `G1b` ("all 18 decisions settled") with component-scoped entry gates **G1-L/G1-S/G1-T/G1-M/G1-R**, keeping G1, G1c, G1d, G1e?
* **Approve:** ☐ **Accept as proposed** (`implementation-gates.md`) — a component-scoped Phase 2 authorization becomes possible. ☐ Accept with modifications (name them). ☐ Reject — the cycle returns: S1/S11 (or any open question) blocks everything, while their remaining measurements are Phase 2 deliverables. ☐ Defer.
* **Consequence of acceptance:** none of the open questions becomes decided; each gated record still refuses precisely one operation until its rule is accepted.

### J-2. Lexical and syntactic recognition contract (gates G1-L / G1-S)

* **Approve:** (a) the evidenced lexical contract (tokens, BOM, terminators, whitespace, identifiers as probed, comments, literals), with (b) the strict unknown-escape refusal **or** legacy-silent behaviour, and (c) the recognition subset: every evidenced production parses; the **nine unsupported syntax families** are refused with PARSE diagnostics; the **lambda-lookahead position** is implemented per Phase-2 parser design.
* ☐ Accept (b) as strict-rejection PROPOSED rule (recommended; C2 divergence from silent legacy behaviour). ☐ Keep legacy silence (compatibility-first). ☐ Defer.

### J-3. Proposed normative rules to accept (or defer) individually

Left-to-right evaluation, receiver-first, short-circuit, assignment-as-statement, block scoping with shadowing, non-capturing nested functions (pending capture design), value equality for strings/records/reference for classes, IEEE float equality, scalar-value diagnostic columns, lossless CST trivia, the gate observability contract, the typed-IR invariants, triple-quote continued rejection, raw newlines permitted. Each is separately labelled in the decision register; **acceptance can be per-record**. ☐ Accept all as listed ☐ Accept selected (list) ☐ Defer all (gates stay active) ☐ Reject selected (list).

### J-4. Source extension and MIME (gates G1c)

* Options: ☐ `.cosmic` (rank 1) ☐ `.cmic` ☐ `.ccos` ☐ keep `.csmx` with the acknowledged C# collision ☐ other. MIME: ☐ `text/x-cosmic-lang` (provisional, unregistrable) ☐ defer ☐ plan `vnd.` registration.
* Authorises the two cheap follow-up searches (marketplace/package registry; trademark is legal) if wanted.

### J-5. Licence (gate G1d)

* Options A–D per `docs/audits/phase-1-licensing-recommendation.md`; engineering recommends deciding **after** counsel review of L1/L6/L8/L9/L10, with a permissive-with-patent-grant preference if publication is intended. ☐ A ☐ B ☐ C ☐ D ☐ defer. **Until then: no release and no external contribution promises — and no "internal" development in this public repository, where a pushed branch or PR is itself publication; the public/private choice is now explicit (Options L-A/L-B in `docs/spec/phase-2-entry-readiness.md` §3).**

### J-6. Scope decisions S3 (PortuKof) and S4 (KofScript)

☐ In scope ☐ Out of scope ☐ Defer (both refused loudly meanwhile). Legal-review items L9/L10 attach to "in scope".

### J-7. Memory strategy (S16) and ABI (S17)

☐ Defer explicitly (recommended; no component before Phase 4 needs them) ☐ Decide now. Note: the probe showing nested functions are non-capturing **substantially narrows** the ADR-0005 study.

### J-8. Language identity and naming (ADR-0001/0003)

Owner confirmation of language name, compiler and binary names; `fun`/`fn`/`func` remain reserved-but-invalid.

### J-9. Grammar evidence granularity (optional)

☐ Keep block-level evidence citations (current proposal) ☐ Require per-production inline markers before G1 acceptance (stricter; the checker already counts 0/117).

### J-10. Phase 2 authorization (Directive 003)

☐ Authorise as proposed (component-scoped: lexer, CST/parser, resolver/typed HIR, `cosmic check` — **no MIR/backend/runtime**) ☐ Authorise narrower ☐ Defer ☐ Reject. See §K.

### J-11. CI extension (optional)

☐ Add `python3 tools/spec-consistency.py` as a CI step so document consistency cannot regress (documentation tooling; no runtime role). Currently run manually.

---

## K. Revised Directive 003 proposal (not executed)

`docs/directives/directive-003-proposal.md` has been rewritten for the new gate model:

* **Entry conditions** are the component gates (G1, G1-L, G1-S, G1-T, G1c, G1d, G1e; G1b only as archival text unless replaced per J-1). It states explicitly that **lexer/parser recognition and semantic acceptance are separate approvals**.
* **Authorised scope:** `cosmic-lexer`, lossless CST, recovering parser, name resolution + typed HIR, `cosmic check`, feature gates, differential harness, fuzz/property tests, measurement run. **No MIR, no optimizer, no backend, no runtime, no standard library.**
* **Unsupported-syntax handling:** the nine syntax families are refused with PARSE diagnostics; open *semantics* never reject a parse — they gate the operation.
* **Completion gates G2–G2e** are defined in the proposal itself (component contracts, per-operation gate negative tests, differential harness, measurements, CI) and gate Phase 3 behind a further owner authorization.
* **Prohibitions preserved:** no legacy import, no extension hard-coding, no licence choice, no PR merge, no release, no protection changes.

**This proposal is not executed and must not be executed until the owner acts on §J.**

---

*End of reconciliation record. Statuses in this document follow the six-word normative vocabulary; nothing here is owner-approved.*