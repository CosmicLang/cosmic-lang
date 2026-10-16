# Phase 0 — Design Conformance Audit (PDF ↔ Implementation)

| | |
|---|---|
| **Date** | 2026-10-16 |
| **Directive** | ENGINEERING DIRECTIVE 001-R1 — Phase 0 Final Review, Design Conformance and Approval Gate |
| **Reference document** | *Cosmic-Lang — Projeto do Compilador Rust*, 20 pages, 15 October 2026 (`Cosmic-Lang_Projeto_Compilador_Rust_20_Paginas.pdf`) |
| **Reviewed repository** | `https://github.com/CosmicLang/cosmic-lang.git` |
| **Reviewed HEAD** | `a522aeeca80361f1e9757e7663bf5f7e47f68409` (`phase/0-foundation`) — matches the directive's expected SHA |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) — re-verified read-only for this audit |
| **Companion documents** | `docs/audits/phase-0-baseline.md`, `docs/audits/phase-0-source-review.md`, `docs/architecture/compiler-pipeline.md`, `docs/adr/0001-language-identity.md`, `docs/spec/compatibility-matrix.md` |

Findings are tagged as in the baseline audit: **E1** directly verified fact ·
**E2** evidence-supported inference · **H** hypothesis requiring investigation ·
**P** proposed engineering decision.

Classification vocabulary for the matrix: **SATISFIED** · **PARTIAL** ·
**DEFERRED** · **CONFLICT** · **NOT VERIFIED**.

---

## 1. Scope and method

1. The 20-page design report was read in full (all 20 pages, text extracted and
   read page by page — this was the gap Directive 001 had to leave open).
2. The repository was inspected before any modification: HEAD, branch history,
   worktree cleanliness, PR #1 metadata and GitHub Actions results (§2).
3. The legacy repository was cloned read-only at the reference SHA and the
   design report's factual claims were re-derived from **source**, not from the
   legacy repository's own prose (§3). This is the step the design report
   itself declared it had not performed.
4. Every architectural requirement in the 20 pages was mapped to the current
   implementation or to the phase that owns it (§4).
5. The Rust implementation was reviewed at source level and probed
   adversarially (§5, and `docs/audits/phase-0-source-review.md`).
6. No Phase 1 work was started; no `cosmic-lexer` was created; PR #1 was not
   merged and not marked ready for review.

**Statement of limits.** The design report is a *documentary* diagnosis: it
states plainly that it was produced "sem clone ou execução de CI, testes,
análise estática ou binários" and that its line counts are "números do
documento, não recontados". Everything in §3 below is a *staleness* of the
legacy repository's published documentation relative to the legacy source at
the reference SHA. None of it is a defect of the new implementation, and none
of it changes a phase boundary.

---

## 2. Repository state as reviewed (E1)

| Item | Value |
|---|---|
| Local HEAD | `a522aeeca80361f1e9757e7663bf5f7e47f68409` |
| Expected SHA (directive) | `a522aeeca80361f1e9757e7663bf5f7e47f68409` → **match** |
| Branch | `phase/0-foundation`, tracking `origin/phase/0-foundation` |
| Worktree | clean before the review (no uncommitted changes, no untracked files) |
| `main` | `7bf5061` (`chore: bootstrap repository with project readme`) — one commit, no merge of `phase/0-foundation` |
| PR #1 | `OPEN`, `isDraft: true`, base `main`, head `phase/0-foundation` @ `a522aee…`, `mergeable: MERGEABLE`, `mergeStateStatus: CLEAN` |
| CI on `a522aee…` | run `38923756871` (`pull_request`) → **success**; run `38923737521` (`push` to `phase/0-foundation`) → **success** |
| History | 6 commits, linear, no force-push or history rewrite observed |
| Toolchain used locally | rustc/cargo 1.99.0 (Homebrew), matching `rust-toolchain.toml` |

The `push` run above is itself the evidence for the CI gap corrected in this
review: the workflow validated a branch push but excluded `main`, so a commit
integrated into the default branch would have produced no validation run.

---

## 3. Discrepancy resolution — documentary assessment vs verified source

This is the audit's most important section: the design report's figures were
re-derived from the legacy source at the reference SHA. The report's own source
of truth was the legacy repository's **published documentation**, which is
measurably stale in five places.

| # | Claim in the design report | PDF page | Where the report got it | Verified against source at `284358f4` (E1) | Verdict |
|---|---|---|---|---|---|
| 1 | "arquitetura publicada utiliza **Java 21**" | 1 | `docs/architecture/compiler-architecture.md:44` diagram label `Kof Compiler (Java 21 + ASM 9.8)` | `pom.xml`: `<maven.compiler.release>25</maven.compiler.release>` **and** `maven-compiler-plugin` `<release>25</release>`. `docs/development/DECISIONS.md` §D-BASELINE (2026-09-14): "The repository build baseline moves from Java 21 to Java 25." The surviving "Java 21" text is scoped to the **tooling API** baseline (`docs/tooling/README.md:52`), not the compiler build. | **PDF STALE** → correct: **Java 25** |
| 2 | ASM **9.8** | 1 | same diagram label | `pom.xml`: `<asm.version>9.10.1</asm.version>`, pinned with an in-file rationale (ASM 9.11 refuses large generated methods in `COMPUTE_FRAMES`) | **PDF STALE** → correct: **9.10.1** |
| 3 | IR with **30 operations** | 1, 6 | `compiler-architecture.md:65/92/307/314` — the doc's own table ("KofOperation (sealed, 30 records)") | `grep -c "implements KofOperation"` over `src/main` = **37** record declarations. `KofOperation.java` is an **empty marker interface, not sealed**. The doc's table omits `KofPop2`, `KofDup2`, `KofContinueLabel`, `KofNewMultiArray`, `KofExcUnlink`, `KofStatementIf`, `KofFunctionAddress` (7 records). | **PDF STALE** → correct: **37 ops, unsealed** |
| 4 | Lexer **477** lines | 5 | `compiler-architecture.md:49` `Lexer (477L)` | `wc -l parser/Lexer.java` = **449** | **PDF STALE** (off by 28) |
| 5 | Parser **1975** lines | 5 | `compiler-architecture.md:50`; also `docs/language-reference/grammar.md:5` | `wc -l parser/Parser.java` = **496**. `docs/architecture/PLAN-SOLID-500.md` row 7 records the cause: "Parser 1975 → ✅ DONE \| 456 + 7 classes (`7433c8e`)" — the figure predates a completed decomposition. | **PDF STALE** (pre-refactor figure) |
| 6 | Semantic Analysis **2293** lines | 5 | `compiler-architecture.md:54` | `wc -l SemanticAnalyzer.java` = **566**. `PLAN-SOLID-500.md` row 6: "SemanticAnalyzer 2293 → ✅ DONE \| 396 + 8 classes (`6e2ff77`)". | **PDF STALE** (pre-refactor figure) |
| 7 | AST **50 sealed nodes** | implied, p4 | `compiler-architecture.md:51` `AST (50 nós sealed)` | Baseline audit §3.3 (E1, Directive 001): ~55 record types implementing **non-sealed** marker interfaces; `AstNodes.java` contains only orphan Javadoc. | **PDF STALE** |
| 8 | Backends **JVM, nativo, JS** | 1, 7 | published architecture | Confirmed by module inspection: `jvm/JvmBackend` (60 files), `nat/NativeBackend` (226), `js/JsBackend` (66). The list is also **incomplete**: WASM/WASI, Android (JVM + `AndroidProjectWriter`) and a direct-IR script interpreter also exist. | **CONFIRMED, incomplete** |
| 9 | `kof-runtime` is not used by generated JVM code, which emits its own `KofRuntime` | 8 | architecture doc | Confirmed (baseline audit §3.2, E1). | **CONFIRMED** |
| 10 | JS announces **sequential** `spawn`/`await` while other targets use concurrent mechanisms | 7 | architecture doc | Corroborated: `docs/targets/KOFANDROID.md` records "KofJS do WebView: sequencial". Additionally, `docs/status.md` §698 documents a **real, measured cross-target divergence** (Int shift semantics differed on riscv64/aarch64/JS from the JVM), which is direct evidence for the report's cross-target risk. | **CONFIRMED** |
| 11 | Golden corpus: **74** cases; `tests/golden/` | — | baseline audit §5.2 | `ls tests/golden` → **74** case directories. | **CONFIRMED** |
| 12 | Legacy is **GPL-3.0**; POMs declare no `<licenses>` | 8 (implied) | baseline audit §3.1 | `LICENSE` = GNU GPL v3, 29 June 2007; `grep "<licenses>"` over all POMs → no matches; `SECURITY.md` and `TRADEMARK.md` present. | **CONFIRMED** |
| 13 | Source extensions `.kf` / `.kof` / `.ptkf` / `.ks`; `kof.toml`; `kofdeps.lock`; no `kof.lock` | 3, 12 | architecture doc | `KofCliSupport.isKofSource` → `.kf`/`.kof`/`.ptkf`; a second filter adds `.ks`. Tree counts: 330 `.kf`, 4 `.ptkf`, 0 `.ks`. `examples/fullstack/kof.toml` exists; no `kof.lock`/`kofdeps.lock` in the tree. | **CONFIRMED** |

### 3.1 Conclusion of the discrepancy resolution

* Every discrepancy is a **stale legacy document**, never a divergence between
  the design report and the new implementation's behaviour. The report was
  honest about its own limits and about not recounting; the numbers it quotes
  are simply older than the reference SHA.
* The new repository's Directive 001 audit already reported the **correct**
  values (Java 25; 37 unsealed operation records) because it inspected code.
  The directive's instruction — "do not treat outdated source documentation as
  more authoritative than the inspected code" — is therefore satisfied by the
  existing documentation, and the design report's figures must **not** be
  carried into Phase 1 as facts.
* **Phase 1 consequence (P):** the specification work must cite the *source at
  a frozen SHA*, not the legacy architecture prose. The legacy repository's own
  documentation is demonstrably unreliable on six independent figures (Java,
  ASM, IR op count, Lexer, Parser, Sema, AST) — a fact that strengthens the
  case for rebuilding the compatibility contract from executed behaviour and
  source inspection, as `docs/spec/compatibility-matrix.md` already requires.

---

## 4. Conformance matrix — design report → implementation

Row identifiers are `P<page>-<n>`. "Target phase" uses the phase boundaries
restored in §5.

### Page 1 — Executive summary and technical decision

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P1-1 | Do not perform a big-bang rewrite; freeze semantics and build conformance tests first, then a Rust frontend, then backends with parity evidence | Foundation only; no rewrite attempted; legacy behaviour baselined as an oracle | `README.md` "Status: Phase 0 — Foundation"; workspace = 3 crates; `docs/architecture/compiler-pipeline.md` §5 non-goals | **SATISFIED** | Phase 0 | None |
| P1-2 | Provenance: the fork declares origin in `KofLang/Kof4j` and keeps Kof naming, Maven modules and original docs | Recorded and verified | `docs/audits/phase-0-baseline.md` §3.1/§3.2; `docs/adr/0001-language-identity.md` §1 | **SATISFIED** | Phase 0 | None |
| P1-3 | Published architecture uses **Java 21** | Legacy build baseline is Java **25** | §3 row 1 | **CONFLICT** | Phase 1 | Cite Java 25 in the specification; treat the "Java 21" figure as stale |
| P1-4 | IR of a stack machine with **30 operations** | Legacy IR has **37** unsealed operation records | §3 row 3 | **CONFLICT** | Phase 1 | Use 37 in the specification; do not inherit the count |
| P1-5 | Backends JVM, native and JS | Confirmed (plus WASM/WASI, Android, script) | §3 row 8 | **SATISFIED** | Phase 0 (baseline) | Record the additional targets in the compatibility scope |
| P1-6 | Adopt Rust, but do not migrate the whole ecosystem at once | Only the foundation exists; nothing else was migrated | `README.md`; workspace members | **SATISFIED** | Phase 0 | None |

### Page 2 — Scope, method and evidence classification

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P2-1 | Adopt the E1/E2/H/P evidence vocabulary | Adopted verbatim and used consistently | `docs/audits/phase-0-baseline.md` §1; this document | **SATISFIED** | Phase 0 | None |
| P2-2 | Freeze SHA, inventory the tree, run tests in a clean environment | All three performed | §2 (SHA), §3 (tree), baseline audit §5 (isolated clone, golden corpus 222/222 for jvm/js/script) | **SATISFIED** | Phase 0 | None |
| P2-3 | Measure coverage, dependencies, duplication, cyclomatic complexity, cold-start, throughput and artifact size | Not measured | Toolchain policy §5 states no measurements exist; zero dependencies; no artifacts | **PARTIAL** | Phase 1 (measurement work), Phase 4 (cold-start/throughput/artifacts) | Add a measurement task to Directive 002; do not claim metrics before then |
| P2-4 | Make no claim of exploitable vulnerability, regression, security failure or definitive incompatibility without reproducible proof | Respected throughout | Baseline audit §7 lists only first-hand verified items and quotes maintainer ledger items as such | **SATISFIED** | Phase 0 | None |

### Page 3 — Fork state and Cosmic-Lang identity

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P3-1 | Identity ADR: `cosmcc`, `cosmic`, `cosmic-std`, `.csmx`, package names, `.kf`/`.ks` alias policy, compatibility matrix | ADR-0001 exists as **PROPOSED** covering compiler/executable names, `.csmx`, alias policy and a compatibility policy; a compatibility matrix seed exists. `cosmic-std` naming is **not** covered | `docs/adr/0001-language-identity.md`; `docs/spec/compatibility-matrix.md` | **PARTIAL** | Phase 1 (acceptance) | Add a `cosmic-std` / stdlib naming decision to Directive 002; obtain owner approval |
| P3-2 | Risk: a purely cosmetic rename leaves namespaces, executables, Maven artifacts, docs, telemetry and packages divergent | Recorded as context in the ADR | ADR-0001 §1, §6 | **PARTIAL** | Phase 1 | Convert the risk into explicit acceptance criteria per renamed surface |
| P3-3 | Organisation of the legacy tree (`kof-compiler`, `kof-cli`, `kof-runtime`, `kof-script`, `kof-c-compiler`, plus docs/training/editor/libs/scripts/tests) | Verified | §3; baseline audit §3.2 | **SATISFIED** | Phase 0 | None |

### Page 4 — Diagnostic 01: architectural boundaries

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P4-1 | Pipeline documented as Lexer → Parser → AST → desugar/imports → SemanticAnalyzer → lowering → IR → optimizer → backend | Documented as the legacy pipeline and mirrored in the target pipeline | `docs/architecture/compiler-pipeline.md` §1 | **SATISFIED** | Phase 0 | None |
| P4-2 | Legacy has side type maps and no own typed AST | Verified | Baseline audit §3.3 | **SATISFIED** | Phase 0 | None |
| P4-3 | Build an immutable typed HIR with stable IDs, symbol interning and invariants verified on entry to MIR | Not started (correctly out of Phase 0 scope) | `compiler-pipeline.md` §5 non-goals; target pipeline lists `cosmic-hir` as "not started" | **DEFERRED** | Phase 1 (contract) → Phase 2 (implementation) | Specify HIR invariants in Directive 002; implement in Phase 2 |

### Page 5 — Diagnostic 02: frontend scalability

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P5-1 | Published indicators: Lexer 477, Parser 1975, Semantic Analysis 2293 lines | Verified source gives 449 / 496 / 566; the published figures predate completed decompositions | §3 rows 4–6 | **CONFLICT** | Phase 1 | Use measured values; the "size risk" argument no longer applies to Parser/Sema at this SHA |
| P5-2 | Separate lexical, parser, recovery, resolution, inference, effect checking and diagnostics; define internal contracts | Partially embodied: `cosmic-source` and `cosmic-diagnostics` are separate crates with explicit contracts, and the workspace rule forbids empty mirror crates | `compiler-pipeline.md` §1 "Rule for introducing crates"; §2 | **PARTIAL** | Phase 1 (contracts) → Phase 2 (implementation) | Fix the remaining component contracts in Directive 002 |
| P5-3 | Criterion: the parser must recover multiple errors, never loop infinitely, and pass structured fuzzing | Not started | — | **DEFERRED** | Phase 2 (gate G2) | Make error-recovery and fuzz criteria explicit acceptance gates |

### Page 6 — Diagnostic 03: stack IR and optimizations

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P6-1 | IR is a typed linear stack machine with 30 operations; minimal optimization, no inlining/LICM | 37 operations (see P1-4); "minimal optimization, no inlining/LICM" verified | §3 row 3; baseline audit §3.4 (`Optimizer`: constant folding, constant/copy propagation, dead-effect elimination, reachability, jump-to-next removal; ≤ 8 iterations) | **CONFLICT** (count) / **SATISFIED** (optimization scope) | Phase 1 | Use 37; record the optimizer scope as verified |
| P6-2 | Adopt block-based MIR with CFG and SSA (or on-demand SSA) and verifiers after each pass | Recorded as the target; not implemented | `compiler-pipeline.md` §1/§3.4 | **DEFERRED** | Phase 1 (design) → Phase 3 (implementation) | Specify the MIR/CFG/SSA contract in Directive 002 |
| P6-3 | Criterion: differential semantics suite before/after constant folding, DCE and CFG simplification | Not started | — | **DEFERRED** | Phase 3 (gate G3) | Define the differential harness as a gate |

### Page 7 — Diagnostic 04: multi-backend parity

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P7-1 | Targets: JVM, JS, native and direct IR interpretation; per-target concurrency mechanisms differ | Verified, and divergence is documented first-hand in the legacy ledger | §3 row 10; baseline audit §3.6 (SCRIPT = `KofInterpreter`, no artifact) | **SATISFIED** | Phase 0 (baseline) | None |
| P7-2 | Define normative semantics for concurrency, overflow, Unicode, equality, exceptions and evaluation order; declare per-target capabilities | Not started; the open questions are enumerated instead | `docs/spec/compatibility-matrix.md` §3 items 6–7 | **DEFERRED** | Phase 1 | Make these the core of Directive 002 (see §7 below) |
| P7-3 | Criterion: automated matrix of golden tests per backend with a documented SKIP per feature | The legacy corpus provides the matrix (74 cases × 4 target classes; native SKIPPED with a stated reason). The new toolchain's matrix is a seed only | Baseline audit §5.2/§5.3; `compatibility-matrix.md` header legend | **PARTIAL** | Phase 1 (matrix) → Phase 4 (execution) | Keep SKIP reasons mandatory; the new matrix must not claim more than the legacy evidence |

### Page 8 — Diagnostic 05: runtime and coupling

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P8-1 | `kof-runtime` exists but generated JVM code emits its own `KofRuntime` | Verified | §3 row 9 | **SATISFIED** | Phase 0 (baseline) | None |
| P8-2 | Separate semantically specified runtime, stdlib, per-target bindings and a versioned ABI; forbid runtime → frontend reverse dependencies | Not started | `compiler-pipeline.md` §5 non-goals | **DEFERRED** | Phase 1 (design) → Phase 3/4 (implementation) | Specify the runtime/stdlib boundary and the ABI versioning rule |
| P8-3 | Criterion: conformance tests per runtime function and an ABI/API compatibility table | Not started | — | **DEFERRED** | Phase 4 | Define the runtime conformance gate |

### Page 9 — Diagnostic 06: product scope

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P9-1 | The legacy README announces JVM/native/JS, Android, REPL, UI, HTTP, DB, ORM, crypto, tooling and a C subset | Verified | Legacy tree: `kof-c-compiler`, `docs/bugs-and-gaps/KOFUI-AUDIT.md`, `docs/architecture/makealive-plan.md`; baseline audit §3.2 | **SATISFIED** | Phase 0 (baseline) | None |
| P9-2 | Prioritise core language, CLI, minimal stdlib and one reference target; treat Android/UI/C-subset as independent products | Embodied in the Phase 0 workspace but not yet a ratified scope decision | `compiler-pipeline.md` §1 crate rule; §5 non-goals | **PARTIAL** | Phase 1 | Record the scope decision explicitly for owner approval |
| P9-3 | Criterion: no feature declared stable without specification, conformance suite and a defined owner | Adopted in spirit (invariant 4; "one crate per concrete contract"); not formalised per feature | `compiler-pipeline.md` §4 invariant 4 | **PARTIAL** | Phase 1 | Add a per-feature stability/ownership table to the specification |

### Page 10 — Diagnostic 07: security and supply chain

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P10-1 | Presence of `SECURITY.md` and automation does not prove efficacy of controls | Verified as fact, no efficacy claim made | §3 row 12 | **SATISFIED** | Phase 0 | None |
| P10-2 | Audit Maven/transitive dependencies, versioned executables, link subprocesses, adversarial inputs, escaping and generated JS | Not performed (explicitly a hypothesis list) | — | **NOT VERIFIED** | Phase 1 | Keep as hypotheses; do not promote to findings without reproduction |
| P10-3 | `cargo audit`/`cargo deny`, CycloneDX SBOM, locked dependencies, reproducible build, lexer/parser fuzzing, ASAN for native support components | Partially: `Cargo.lock` committed, **zero** external dependencies, pinned toolchain, CI actions pinned to commit SHAs by this review. No `cargo audit`/`cargo deny`, no SBOM, no fuzzing yet | `docs/development/toolchain-policy.md` §2/§3; `.github/workflows/ci.yml` | **PARTIAL** | Phase 1 (tooling) → Phase 2 (fuzzing) | Add `cargo audit`/`cargo deny` + SBOM to Directive 002 |
| P10-4 | Gate: block release on an applicable critical advisory, detected secrets, missing provenance, failing sandbox tests | Not applicable — no releases exist | Toolchain policy §5 | **DEFERRED** | Phase 5 (gate G5) | Instantiate the gate when the first artifact exists |

### Page 11 — Diagnostic 08: testability and observability

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P11-1 | Published test counts do not correspond to verified local execution | The baseline audit separates executed evidence from published counts | Baseline audit §5.4/§5.5 | **SATISFIED** | Phase 0 | None |
| P11-2 | Introduce snapshot tests, property-based tests, selective mutation tests, fuzz corpus, golden diagnostics, differential compiler tests and regression suites | Golden diagnostics exist (exact-string human **and** JSON render tests); differential testing uses the legacy golden corpus as oracle. Property-based, mutation and fuzz: not started | `crates/cosmic-diagnostics/tests/render.rs`; `docs/spec/compatibility-matrix.md` §5 | **PARTIAL** | Phase 1 (property-based/mutation plan) → Phase 2 (fuzzing) | Specify the test-strategy ladder in Directive 002 |
| P11-3 | SLO: zero known ICE in the corpus; 100% critical regressions; publish p50/p95 performance on a fixed machine | Not measurable yet — there is no compiler to crash or measure | — | **DEFERRED** | Phase 2+ | Adopt the SLO as a gate, not as a current claim |

### Page 12 — Proposed `.csmx` extension

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P12-1 | `.csmx` as the Cosmic-Lang source extension, provisional MIME `text/x-cosmic-lang`, editor association | `.csmx` proposed in ADR-0001 and already used consistently in Phase 0 code, docs and tests. MIME type and editor association not registered | `docs/adr/0001-language-identity.md` §2.2/§3; `crates/cosmic-source/tests/source_map.rs` uses `*.csmx` fixtures | **PARTIAL** | Phase 1 (approval) → Phase 2 (tooling) | Register the MIME type and editor association in the lexer phase; obtain owner approval |
| P12-2 | Uniqueness of `.csmx` cannot be proven; search extension registries, GitHub and the editor ecosystem before formalising | Two independent web queries performed; exhaustive GitHub code search and trademark/registry queries explicitly **not** performed | ADR-0001 §3 | **PARTIAL** | Phase 1 | Perform the exhaustive search as an explicit Phase 1 task; keep the extension *proposed* until then |
| P12-3 | Migration policy: autodetection by extension, parser independent of extension, opt-in codemod migration, preserve old sources until the compatibility gate | Principle recorded; nothing implemented | ADR-0001 §2.3/§4 | **DEFERRED** | Phase 2 (autodetect) / Phase 5 (codemod) | Carry the policy into Directive 002 as a constraint |

### Page 13 — Rust reference architecture

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P13-1 | Pipeline: SourceMap → Lexer → Lossless CST → Parser AST → Name Resolution → Typed HIR → MIR/CFG → SSA → Optimizer → Codegen → Link/Package | Mirrored exactly, with per-stage "implemented / not started" markers | `compiler-pipeline.md` §1 | **SATISFIED** | Phase 0 (documentation) | None |
| P13-2 | Modules: `cosmic-source`, `cosmic-lexer`, `cosmic-syntax`, `cosmic-hir`, `cosmic-sema`, `cosmic-mir`, `cosmic-opt`, `cosmic-codegen-*`, `cosmic-runtime`, `cosmic-cli` | Target list matches; only `cosmic-source` and `cosmic-cli` exist, plus `cosmic-diagnostics` (a cross-cutting service the report's list omits) | `Cargo.toml` workspace members; `compiler-pipeline.md` §2.2 | **PARTIAL** | Phase 1+ | Record `cosmic-diagnostics` as a justified addition to the module list |
| P13-3 | Invariants: valid spans, recoverable AST, typed HIR, MIR with explicit terminators, SSA with a dominance verifier, no undefined instructions | Only "valid spans" has code today — enforced by checked constructors and tested exhaustively | `crates/cosmic-source/src/span.rs`; `crates/cosmic-source/tests/source_map.rs` | **PARTIAL** | Phase 1 (contracts) → Phase 2/3 (enforcement) | State each invariant as a Phase 1 contract with an enforcement point |
| P13-4 | Design: stable per-crate internal APIs, immutable structures where possible, ID-indexed arenas, structured tracing | Immutability and per-crate APIs honoured (`Span`/`FileId` are `Copy`, no interior mutability, no `unsafe`). Arenas and tracing not started | `crates/cosmic-source/src/*`; `Cargo.toml` `unsafe_code = "forbid"` | **PARTIAL** | Phase 1 (design) → Phase 2 (arenas) | Specify arena/ID and tracing conventions |

### Page 14 — Frontend: lexer, parser and typed HIR

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P14-1 | Lexer: byte-based UTF-8 spans, trivia preserved, normalisation only where specified, comments/docstrings, anti-DoS limits | Span half implemented: `Span` is a validated byte range on UTF-8 boundaries with checked constructors. Anti-DoS partly implemented: a 4 GiB `u32` cap, now enforced **from metadata before reading** (this review). Lexer itself not started | `crates/cosmic-source/src/span.rs`, `src/map.rs`; `docs/audits/phase-0-source-review.md` §3 | **PARTIAL** | Phase 1 (trivia/normalisation policy) → Phase 2 (lexer) | Specify the normalisation policy; implement the lexer in Phase 2 |
| P14-2 | Parser: Pratt expressions + predictive declarations; lossless CST preserving comments and errors for formatter/LSP | Not started | — | **DEFERRED** | Phase 2 | Keep out of Phase 1 (directive §4) |
| P14-3 | Sema: per-scope symbol table, interning, binding and overload resolution; restricted inference with unification and local checks | Not started | — | **DEFERRED** | Phase 2 | Keep out of Phase 1 |
| P14-4 | Contract: every diagnostic carries code, span, optional suggestion and severity; **no panic on invalid input** | **Fully implemented and tested.** `Diagnostic` = `Severity` + stable `DiagnosticCode` + message + exactly one primary `Label`(span) + secondary labels + notes + structured `Suggestion`(span, replacement, message, applicability). Both renderers are total; unresolvable spans degrade to a byte-range fallback | `crates/cosmic-diagnostics/src/diagnostic.rs`, `code.rs`, `render/human.rs`, `render/json.rs`; `tests/render.rs` | **SATISFIED** | Phase 0 | None |

### Page 15 — Middle-end: MIR, SSA and optimization

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P15-1 | MIR: `BasicBlock`, `Statement`, `Terminator`, `Place`, `Rvalue`, `TypeId`; explicit memory/effects | Not started | `compiler-pipeline.md` §5 | **DEFERRED** | Phase 3 (design in Phase 1) | Specify in Directive 002, implement in Phase 3 |
| P15-2 | Passes v1: const propagation, unreachable-block elimination, DCE, CFG simplification, conservative CSE | Not started | — | **DEFERRED** | Phase 3 | Define the pass contract and verifier ordering |
| P15-3 | Later passes (inlining with budget, LICM, strength reduction, escape analysis, PGO) only after measurement infrastructure | Not started; the dependency on measurement is recorded | `compiler-pipeline.md` §5; toolchain policy §5 | **DEFERRED** | Phase 3+ | Keep the measurement precondition explicit |
| P15-4 | Verification: run the verifier after lowering and after each pass in test builds; compare interpreted MIR execution with generated output | Not started | — | **DEFERRED** | Phase 3 (gate G3) | Make the verifier a gate, not a convention |

### Page 16 — Backends and execution strategy

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P16-1 | Order: (1) MIR interpreter as oracle, (2) WASM/WASI, (3) native via Cranelift or LLVM, (4) JVM only on measured demand | Recorded in the target pipeline | `compiler-pipeline.md` §3.5 ("WASM/WASI first, then native (Cranelift vs LLVM evaluated later)") | **DEFERRED** | Phase 3/4 | Carry the order into the Phase 1 specification |
| P16-2 | Cranelift vs LLVM decision | Not taken (correctly) | — | **DEFERRED** | Phase 4 | Require a measured decision record |
| P16-3 | JS backend may preserve browser integration; JVM only redone if real cases justify ABI/library maintenance | Not started | — | **DEFERRED** | Phase 4 | Keep as an explicit non-goal until justified |
| P16-4 | Compatibility: declarative feature gates in the manifest; `unsupported` = a clear diagnostic, never silently degraded generation | The **principle** is already implemented: planned commands fail loudly with exit 2, and no command reports success without performing its operation. Manifest feature gates not implemented | `crates/cosmic-cli/src/lib.rs`; `compiler-pipeline.md` §4 invariant 4; `crates/cosmic-cli/tests/cli.rs` | **PARTIAL** | Phase 1 (manifest) → Phase 4 (gates) | Keep the loud-failure rule as the precedent for feature gates |

### Page 17 — Runtime, stdlib and toolchain

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P17-1 | Runtime contract: explicit ABI, UTF-8 strings, collections, allocation, errors, scheduler, resource management | Not started | `compiler-pipeline.md` §5 | **DEFERRED** | Phase 1 (spec) → Phase 3/4 | Specify the runtime contract |
| P17-2 | Memory strategy chosen after study (ownership+borrowing, GC or ARC); do not claim Rust makes compiled code automatically safe | Not chosen; no safety claim made anywhere | `README.md`; toolchain policy §5 | **DEFERRED** | Phase 1 (study) | Make the study an explicit Phase 1 deliverable |
| P17-3 | Stdlib: `cosmic.core`, `cosmic.io`, `cosmic.time`, `cosmic.net` minimal, with per-target capability and safety contracts | Not started; names not yet decided | `compiler-pipeline.md` §5 | **DEFERRED** | Phase 1 (naming/spec) | Cover in Directive 002 (see also P3-1, `cosmic-std`) |
| P17-4 | Tooling: `cosmic fmt`, `check`, `build`, `test`, `run`, `doc`, `lsp`; `Cosmic.toml` + `Cosmic.lock`; hermetic artifacts where viable | The seven commands are declared and **fail loudly** (exit 2, "not implemented yet"); `Cosmic.toml`/`Cosmic.lock` recorded as future names only; hermetic artifacts not started | `crates/cosmic-cli/src/lib.rs` (`PLANNED_COMMANDS`); ADR-0001 §2.4 | **PARTIAL** | Phase 1 (manifest) → Phase 2+ (commands) | Implement `check` first, wired to the real libraries |

### Page 18 — Phased migration plan (the roadmap)

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P18-0 | **Fase 0 — baseline:** freeze SHA, licences and architecture; run CI; create the fork's conformance corpus **without altering semantics** | SHA frozen and verified ✓; architecture documented ✓; CI created, green on PR and on branch push, `main` gap fixed by this review ✓; conformance corpus baselined **read-only** (74 cases, 222/222 executed for jvm/js/script) ✓. **Licences not frozen** — the new repository deliberately has no LICENSE and the decision is escalated to the owner. The corpus is deliberately **not vendored** (GPL fixture risk) and was not re-executed in this review | §2; baseline audit §5/§6; `docs/audits/phase-0-licensing-matrix.md`; `README.md` §License | **PARTIAL** | Phase 0 | Keep **G0-B PARTIAL** (§6). Obtain the owner's licence decision; record the corpus-not-vendored decision as a documented exception |
| P18-1 | **Fase 1 — specification:** freeze syntax, types and ambiguous cases; record ADRs and the `.csmx` extension design | **Not started** — blocked pending owner approval. Deviation: a *proposed* ADR-0001 and a compatibility-matrix seed were produced during Phase 0 | `docs/adr/0001-language-identity.md` (Status: Proposed); `docs/spec/compatibility-matrix.md` | **DEFERRED** | Phase 1 | Keep Phase 1 blocked; treat ADR-0001 as a Phase 0 *input* to Phase 1, not as an approved Phase 1 output |
| P18-2 | **Fase 2 — Rust frontend:** lexer, CST, parser, diagnostics, HIR; differential validation against the fork | Not started | — | **DEFERRED** | Phase 2 | Do not create `cosmic-lexer` in Phase 1 (directive §4) |
| P18-3 | **Fase 3 — MIR:** operational semantics, reference interpreter, minimal optimizer | Not started | — | **DEFERRED** | Phase 3 | — |
| P18-4 | **Fase 4 — targets:** one production target + minimal stdlib parity; further targets only after gates | Not started | — | **DEFERRED** | Phase 4 | — |
| P18-5 | **Fase 5 — adoption:** codemods, documentation, signed releases, controlled compatibility deprecation | Not started | — | **DEFERRED** | Phase 5 | — |

### Page 19 — Acceptance criteria, risks and governance

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P19-1 | Gates G0–G5 (corpus/SHAs; grammar; fuzz; MIR verifier; e2e target; reproducible release) | G0 partially satisfied; G1–G5 not started | §6 below; baseline audit §8 | **PARTIAL** | Phase 0 (G0) → later phases | Retain G0-B as PARTIAL; a documented exception may be proposed, never disguised as PASS |
| P19-2 | Major risks: inherited semantic ambiguity, cross-target compatibility, cost of JVM+JS+native, memory design, no dedicated team | Documented as a risk register, and cross-target divergence is now corroborated by a first-hand legacy bug record | Baseline audit §8; `compatibility-matrix.md` §3; legacy `docs/status.md` §698 | **SATISFIED** | Phase 0 | None |
| P19-3 | Mitigations: RFC/ADR, differential tests, gradual rollout, fixed budget per target, primary target, CI matrix, per-module owners | Partially present: ADR mechanism started; differential testing = legacy golden oracle; CI exists on one OS; no per-module owners | ADR-0001; `.github/workflows/ci.yml`; baseline audit §5 | **PARTIAL** | Phase 1 | Add a multi-OS matrix and per-module ownership to Directive 002 |
| P19-4 | Metrics: ICE rate, incremental build time, per-feature parity, conformance, post-release regressions, memory consumption, P0 TTR | Not measured; absence is explicitly documented | Toolchain policy §5 | **DEFERRED** | Phase 1 (instrument) → Phase 4 (report) | Define the metric set and the fixed measurement machine |

### Page 20 — Conclusion, backlog and sources

| ID | Requirement (report) | Implemented state | Evidence | Classification | Target phase | Required action |
|---|---|---|---|---|---|---|
| P20-1 | Adopt Rust without migrating the whole ecosystem at once; the asset to preserve is the language's demonstrable semantics | Exactly what Phase 0 did | `README.md`; baseline audit §9 | **SATISFIED** | Phase 0 | None |
| P20-2 | Backlog P0: freeze SHA ✓; audit licence and dependencies ✓ (legacy GPL-3.0 verified; new repo intentionally unlicensed, escalated; zero dependencies + committed lockfile + SHA-pinned CI actions); execute baseline ✓ (222/222 read-only); minimal specification ✗; golden corpus ✓ as oracle; ADR of naming/extension ✓ proposed; **lexer and parser prototype ✗** | Mixed — see cells | §2, §3; `docs/audits/phase-0-licensing-matrix.md`; ADR-0001 | **PARTIAL** | Phase 0 / Phase 1 | **CONFLICT resolved:** the P0 "lexer and parser prototype" contradicts the report's own Fase 2 and directive §4 — the frontend belongs to Phase 2. The phase plan and the directive win; the prototype is **not** authorised |
| P20-3 | Backlog P1: typed HIR, MIR verifier, interpreter, fuzzing, `check` tool, initial target, migration CLI | Not started | — | **DEFERRED** | Phase 2–5 | Carry into Directive 002's backlog |
| P20-4 | Primary sources: the report's five cited artefacts exist and were read | Verified: `README.md`, `docs/architecture/compiler-architecture.md`, `docs/audits/complexity-audit.md`, `docs/bugs-and-gaps/ecosystem-coverage.md`, `pom.xml` | Read-only clone at `284358f4` | **SATISFIED** | Phase 0 | None |
| P20-5 | Methodological warning: a complete audit of all bugs/risks requires source analysis, build and execution; the report is a diagnosis plus an investigation plan | This review closes the gap: the legacy tree was cloned and inspected at source level, and the executed baseline from Directive 001 is on record | §1, §3; baseline audit §5 | **SATISFIED** | Phase 0 | None |

---

## 5. Restored roadmap and phase boundaries

The design report's phase plan (page 18) and the directive's phase list agree.
The boundaries are:

| Phase | Owns | Status after this review |
|---|---|---|
| **0** | Foundation, provenance, source-management infrastructure, diagnostics, baseline | **Substantially complete, one documented gap** (G0-B, §6) |
| **1** | Formal specification, grammar, typing contracts, semantic ambiguities, approved ADRs | **Not started — blocked pending owner approval** |
| **2** | Lexer, lossless CST, parser, AST, typed frontend | Not started |
| **3** | MIR, interpreter, verification, initial optimizer | Not started |
| **4** | Backend implementation, runtime conformance | Not started |
| **5** | Migration, releases, adoption | Not started |

**Boundary deviations to record (E1):**

1. **ADR-0001 was authored during Phase 0** although page 18 assigns ADR
   recording to Fase 1. This is acceptable *only* because the ADR is marked
   **Proposed** and its §5 lists the owner decisions required before it can be
   accepted. It must be treated as a Phase 0 input to Phase 1, not as an
   approved Phase 1 deliverable.
2. **The compatibility matrix seed was authored during Phase 0**, although the
   specification is Fase 1 work. It is explicitly a *seed* that cites evidence
   and claims nothing beyond "legacy evidence"; it is not a grammar.
3. **`docs/spec/` exists in Phase 0** as the home of that seed. No grammar,
   typing contract or semantic decision was frozen.
4. **Page 20's P0 backlog asks for a "lexer and parser prototype"**, which
   contradicts the report's own Fase 2 and directive §4. **Resolution: the
   phase plan and the directive win. `cosmic-lexer` is NOT created in
   Phase 1 without additional owner authorisation.**
5. **Page 18 Fase 0 says "freeze licences"**; the new repository deliberately
   has no LICENSE and the decision is escalated (see
   `docs/audits/phase-0-licensing-matrix.md`). This is a documented deviation
   requiring owner acceptance, not an omission.

---

## 6. Gate G0 status (evidence-based)

The directive requires that G0-B be retained as **PARTIAL** unless the missing
legacy test coverage has genuinely been executed and verified.

| Gate item | Status | Evidence |
|---|---|---|
| G0-A — SHAs frozen and published | **SATISFIED** | New repo HEAD `a522aee…`; legacy reference `284358f4…`; both recorded in `docs/audits/phase-0-baseline.md` and `README.md` |
| G0-B — conformance corpus executed and verified | **PARTIAL** | Executed: legacy golden corpus **222/222** checks (74 cases × jvm/js/script) in an isolated clone on 2026-10-16. Not executed: the ~6,284-method JUnit reactor, native/Android/browser golden variants, and the reproducible-build check. Not re-executed in this review |
| G0-C — architecture documented | **SATISFIED** | `docs/architecture/compiler-pipeline.md`; this audit |
| G0-D — CI operational | **SATISFIED** (gap fixed) | Run `38923756871` (PR) and `38923737521` (branch push) both **success**; `main`-push validation added by this review |

**G0-B remains PARTIAL.** No exception is proposed here; the missing coverage
is genuinely missing.

---

## 7. Classification summary

| Classification | Count | Rows |
|---|---|---|
| **SATISFIED** | 20 | P1-1, P1-2, P1-5, P1-6, P2-1, P2-2, P2-4, P3-3, P4-1, P4-2, P7-1, P8-1, P9-1, P10-1, P11-1, P13-1, P14-4, P19-2, P20-1, P20-4, P20-5 (P20-5 counted once) |
| **PARTIAL** | 19 | P2-3, P3-1, P3-2, P5-2, P7-3, P9-2, P9-3, P10-3, P11-2, P12-1, P12-2, P13-2, P13-3, P13-4, P14-1, P16-4, P17-4, P18-0, P19-1, P19-3, P20-2 |
| **DEFERRED** | 26 | P4-3, P5-3, P6-2, P6-3, P7-2, P8-2, P8-3, P10-4, P11-3, P12-3, P14-2, P14-3, P15-1…P15-4, P16-1…P16-3, P17-1…P17-3, P18-1…P18-5, P19-4, P20-3 |
| **CONFLICT** | 4 | P1-3, P1-4, P5-1, P6-1 |
| **NOT VERIFIED** | 1 | P10-2 |

The four **CONFLICT** rows are all resolved in §3: they are stale legacy
documentation, and the inspected code is authoritative. None of them affects
the phase plan.

---

## 8. Residual limitations of this audit

* **Corpus not re-executed.** The legacy golden corpus was verified read-only
  (case count, structure, extensions) but not re-run in this review; the
  executed evidence remains Directive 001's 222/222 result. A host with the
  legacy JVM/Node toolchain would be needed to reproduce it, and re-running it
  was not required by this directive's scope.
* **Native target.** Not executed, for the platform reason already recorded
  (GNU-ELF assembly vs the macOS assembler). Unchanged.
* **Documentary staleness is systemic.** Six independent figures in the legacy
  architecture documentation disagree with the legacy source at the reference
  SHA. Phase 1 must therefore treat every legacy prose claim as unverified
  until re-derived from source or from executed behaviour.
* **No independent legal advice.** The licensing matrix separates established
  facts from interpretation and flags the questions that need qualified
  review.
