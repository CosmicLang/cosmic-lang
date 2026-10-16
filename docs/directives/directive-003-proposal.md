# ENGINEERING DIRECTIVE 003 — Proposal (Rust Lexer and Parser Foundation)

> **Status: PROPOSED — not authorised.** This is the recommended text for
> Directive 003, produced by Directive 002 §10J. Directive 002 §9 forbids
> starting Phase 2 until gates G1, G1b, G1c, G1d and G1e are satisfied.

---

## COSMIC-LANG — ENGINEERING DIRECTIVE 003
### Phase 2 — Lexer, Lossless CST, Parser and Typed Frontend

**Role:** Principal Rust Compiler Engineer
**Reasoning:** Maximum available
**Repository:** `https://github.com/CosmicLang/cosmic-lang.git`
**Expected base branch:** `main` (after PR #2 is accepted by the owner)
**Development branch:** `phase/2-frontend`
**Legacy reference:** `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de`
**Inputs from Phase 1:** `docs/spec/language-spec.md`,
`docs/spec/grammar.ebnf`, `docs/spec/typing-contracts.md`,
`docs/spec/decisions/` (S1–S18), `docs/spec/diagnostic-codes.md`,
`docs/development/test-strategy.md`, `docs/development/measurement-plan.md`

---

## 1. Mission

Phase 1 froze the language's evidenced surface as a specification. Phase 2
builds the frontend that implements it: a total lexer, a lossless CST, a
recovering parser, name resolution, and a typed HIR with stable IDs.

**Explicitly out of scope:** MIR, the optimizer, any backend, the runtime, the
standard library, and the package manager. Phase 2 produces a frontend that can
parse and type-check a program and report diagnostics — nothing that executes it.

---

## 2. Entry conditions (must be verified before any code)

| Gate | Requirement | If unmet |
|---|---|---|
| **G1** | Grammar and specification evidence complete | Cannot start: the grammar is the contract |
| **G1b** | All 18 decisions decided or explicitly owner-dependent | Any **OPEN** decision whose construct Phase 2 would implement becomes a `GATE` "not yet specified" **error**, per S15. Phase 2 may not guess |
| **G1c** | Identity/extension/compatibility prepared for approval | The extension must **not** be hard-coded anywhere; the parser must be extension-independent |
| **G1d** | Licensing decision presented | If unresolved, **no legacy artefact may be imported**; the differential harness runs in skip-with-reason mode |
| **G1e** | CI green and governance protections verified | Blocked |
| **G0-B** | Accepted as PARTIAL (owner accepted this for Phase 1) | Remains PARTIAL; Phase 2 may propose the missing legacy coverage run as a separate task, never as a pass |

**Additional entry condition:** the owner must decide the **memory strategy**
(ADR-0005) **or** explicitly defer it. Phase 2 does not need it — the frontend
is memory-model-agnostic — but closure capture (S11 Q7) is a frontend question
that feeds it, so the deferral must be explicit.

---

## 3. Authorised scope

1. `cosmic-lexer` — a total lexer producing tokens with byte spans from
   `cosmic-source`, plus trivia.
2. `cosmic-syntax` — a **lossless** CST preserving every byte of trivia
   (S14 Q1), and an AST derived from it.
3. Parser recovery — multiple errors per run, guaranteed termination, one
   diagnostic per construct (S15 Q3).
4. `cosmic-sema` — name resolution and a **typed HIR** with stable IDs.
5. The `cosmic check` command — the first real CLI command, wired to the
   libraries (closing the Phase 0 "CLI is not wired to the libraries" note).
6. The test ladder layers L2–L5 from `docs/development/test-strategy.md`.
7. The `GATE` "not yet specified" mechanism (S15), so that every OPEN decision
   is a hard error rather than an accident.

---

## 4. Constraints

1. **No construct without a decision.** A construct whose record is OPEN is
   **refused**, never guessed. This is the single most important constraint.
2. **No extension hard-coding.** The lexer/parser must not reference `.csmx`.
3. **Lossless trivia.** Whitespace, comments and the exact line-terminator form
   are preserved (S14).
4. **Byte spans are authoritative.** Positions come from `cosmic-source`;
   line/column is derived for presentation. Columns count Unicode scalar values
   (S13 Q4), and the intentional divergence from the legacy code-unit counting
   is recorded in the differential harness's normalisation list.
5. **No panics on input.** Every fallible path returns a typed error or a
   diagnostic. `unsafe_code` stays `forbid`.
6. **No new dependencies without the toolchain-policy justification** (§3 of
   `docs/development/toolchain-policy.md`).
7. **No legacy artefact.** No `.kf`/`.ptkf`/`.ks` file, no `expected.txt`, no
   legacy prose.
8. **No guessing at evaluation order.** S11 is BLOCKED-ON-MEASUREMENT; the
   frontend may build the AST/CST but must keep the evaluation model behind one
   component so the order can be set later.
9. **Diagnostics mint their own codes** per `docs/spec/diagnostic-codes.md`,
   with a test as evidence. Legacy numbers are not inherited.
10. **No backend, no runtime, no stdlib.** Not even stubs.

---

## 5. Required deliverables

| # | Deliverable | Path | Evidence required |
|---|---|---|---|
| E1 | `cosmic-lexer` | `crates/cosmic-lexer/` | Totality on arbitrary bytes; every token type in `grammar.ebnf` §2; the `LEX` code family |
| E2 | Lossless CST | `crates/cosmic-syntax/` | Round-trip property: parsing then printing any input reproduces it byte-for-byte |
| E3 | Parser | `crates/cosmic-syntax/` | Every production in `grammar.ebnf` §3 that is not `[UNRESOLVED]`; the `PARSE` code family |
| E4 | Error recovery | same | A case with ≥3 independent errors reported in one run; a fuzz corpus proving termination |
| E5 | Name resolution + typed HIR | `crates/cosmic-sema/` | TI-1…TI-8 from `typing-contracts.md` §8, each with a test; the `RES`/`TYPE`/`OVL` families |
| E6 | `cosmic check` | `crates/cosmic-cli/` | Wired to the libraries; exit code 1 on compilation failure, 0 on success, 2 on usage |
| E7 | Feature gates | driver | Every OPEN decision record yields a `GATE` error naming the record |
| E8 | Differential harness | `docs/development/test-strategy.md` §3 L4 | Runs in skip-with-reason mode when the legacy toolchain is absent |
| E9 | Property + fuzz tests | `crates/*/tests/`, `fuzz/` | Layers L3 and L5 |
| E10 | Measurement run | `docs/audits/phase-2-report.md` | M1–M6, M8–M10 from `measurement-plan.md`, on the fixed machine, with the full reproducibility protocol |
| E11 | Phase 2 engineering report | `docs/audits/phase-2-report.md` | Gate status with evidence |

---

## 6. Decisions Phase 2 must close (with evidence, not by guessing)

| Record | What Phase 2 must produce |
|---|---|
| **S1** Q1/Q5 | A differential harness for nested generics and overload ambiguity; then a rule, and a record of the legacy divergence as a defect |
| **S2** Q2/Q4 | A definition of "soft cast" from the executed case; a determination of when the structural fallback fires |
| **S5** Q3/Q4/Q6 | Read the executed behaviour of `switch-break-in-loop` and `switch-defaults`; then decide exhaustiveness, fallthrough and the `for` form |
| **S7** Q3 | Division-by-zero behaviour, tested |
| **S8** Q4 | Whether a raw newline inside a literal is permitted |
| **S11** Q1–Q3, Q5–Q7 | Evaluation order, assignment associativity, scoping, shadowing, capture — measured |
| **S12** Q4/Q5 | String equality; `NaN`/`-0.0` equality |
| **S13** Q1 | Non-ASCII identifier cases |
| **S14** Q1 | The round-trip property test |

**Records that stay owner-blocked in Phase 2** (Phase 2 must not fill them):
S3, S4, S16, S17, and the identity/extension/naming decisions.

---

## 7. Gates for Phase 2

| Gate | Criterion |
|---|---|
| **G2** | Frontend complete with fuzzing: the lexer and parser terminate on arbitrary bytes, never panic, and pass a structured fuzz run with a stated budget |
| **G2b** | Every construct the frontend accepts is either specified or explicitly gated as "not yet specified" |
| **G2c** | The differential harness runs and reports pass/fail/skip-with-reason, with the intentional-divergence list populated |
| **G2d** | Measurement run complete with the reproducibility protocol |
| **G2e** | CI green; no new dependency without the policy justification |

**Phase 3 (MIR) must not begin until G2–G2e are met.**

---

## 8. Explicitly not authorised by Directive 003

* Creating `cosmic-mir`, `cosmic-opt`, `cosmic-codegen-*` or `cosmic-runtime`.
* Any MIR, optimizer, backend, runtime or standard-library code.
* Copying, translating or vendoring any legacy code, fixture or prose.
* Choosing a licence.
* Approving any ADR or decision record on the owner's behalf.
* Merging PR #2 or marking it ready for review.
* Changing repository protection settings.

---

## 9. Completion criteria

Phase 2 is complete when: the lexer, CST, parser, resolver and typed HIR exist
and are tested; the `cosmic check` command works; every accepted construct is
specified or gated; the differential harness runs; the measurement run is
recorded; gates G2–G2e are met; and the owner has authorised Phase 3.

**END DIRECTIVE 003 (PROPOSED).**
