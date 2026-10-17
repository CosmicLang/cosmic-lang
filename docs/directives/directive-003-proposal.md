# ENGINEERING DIRECTIVE 003 — Proposed Phase 2 Frontend

> **Status: PROPOSED NORMATIVE RULE / NOT AUTHORIZED.** Draft for owner
> consideration under Directives 002-R1, 002-R2 (post-merge), 002-R3,
> **revised for staged sequencing and the public-repository reconciliation in
> 002-R4, reconciled with the verified S-G1S-1 pattern-header correction
> (002-R5), updated for the owner-accepted G1d risk-acceptance amendment
> (003-L3), and updated for the authorized Apache-2.0 licensing-package
> application (003-L4)**. No implementation may begin from this document alone. Historic
> Directive 002 gate language (G1/G1b/G1c/G1d/G1e) remains an archival record;
> the revised component gates in `docs/spec/implementation-gates.md` were
> **accepted by the owner in Directive 002-R3**, which still does not authorize
> this directive or any Phase 2 work. **Stage A approval must not silently
> authorize Stage B** (§2).

**Role:** Principal Rust Compiler Engineer
**Repository:** `CosmicLang/cosmic-lang`
**Proposed implementation vehicle:** a branch `phase/2-frontend` in this
repository **only under owner Option L-A** (public development, see
`docs/spec/phase-2-entry-readiness.md` §3); under Option L-B the work proceeds
in a genuinely private environment and no branch is created here. Based on
`main` at the Phase 1 integration (PR #3, merge commit `c436a9f`; the earlier
`c52c56d` basis is superseded) and contingent on an explicit owner
authorization of this directive.
**Pinned legacy reference:** `CosmicLang/cosmcc` @
`284358f43db885ccd8b52b11f897ac0defec10de` (read-only oracle).

## 1. Mission and component entry

Build a total Rust lexer, lossless CST, recovering parser, resolver, typed HIR
and `cosmic check` without defining MIR, backend, runtime, or standard-library
semantics. The work is allowed only after the owner has expressly authorized
Directive 003 with the following scoped conditions:

| Gate | Prerequisite to the named component |
|---|---|
| **G1** | Evidence-backed specification/grammar approved by owner **for the precise recognition subset**, with provisional and unsupported syntax listed. No global freeze presumed. |
| **G1-L** | Owner accepts the lexical contract for emitted tokens and error cases; the decision package (`docs/spec/g1-l-lexical-decision-package.md`) presents ten decisions with testable expected outcomes and a LEX allocation plan. **Accepted by the owner (003-G1 §3, 2026-10-16); G1-L entered.** |
| **G1-S** | Owner approves the concrete parser recognition subset and ambiguous cases (especially lambda lookahead); finding S-G1S-1 is resolved (verified pattern-header correction applied, 002-R5 — `docs/spec/g1-s-syntax-decision-package.md` §3.3). Syntax with no evidenced form is explicitly rejected with PARSE diagnostics, while evidenced syntax with open semantics remains parsable. **Accepted by the owner (003-G1 §4, 2026-10-16); G1-S entered.** |
| **G1-T** | Before type checker / typed HIR: every implemented semantic operation must have an ACCEPTED NORMATIVE RULE or owner-authorized narrow provisional contract. All others have operation-specific, tested GATE diagnostics. |
| **G1c** | Owner accepts the identity decision **or explicitly defers** it for frontend-only work — the source extension is already selected (`.cmic`, 2026-10-16, ADR-0002 §4.1). Never hardcode `.cmic` or any extension in the grammar. The Stage A-limited deferral was approved by the owner (003-G1 §5). |
| **G1d** | Owner licensing decision **and** the public/private development choice: **Option L-A** (public development under an established licensing and contribution policy after legal/provenance review) or **Option L-B** (implementation exclusively in a genuinely private repository or local environment) — see `docs/spec/phase-2-entry-readiness.md` §3. **This repository is public: a branch or PR pushed here publishes source even without a release.** No legacy artifacts imported. Owner selected **L-A** and the outbound licence **Apache-2.0**; the internal G1d procedure was amended by owner risk acceptance (003-L3 §1–§3) and the licensing package was authorized and applied on the draft licensing branch (003-L4 §1–§3) — the internal G1d CLEARED transition is recorded only after integration and validation (`docs/spec/phase-2-entry-readiness.md` §9). |
| **G1e** | CI green for exact branch SHA, governance/protection revalidated. |
| **G0-B** | Remains PARTIAL; documented Phase 0 exception is not reported as a pass. |

**G1b is replaced by the owner-accepted component gate model**
(002-R3, 2026-10-16): G1-L/G1-S/G1-T/G1-M/G1-R in
`docs/spec/implementation-gates.md`. The
underlying unresolved semantic questions remain OPEN. Approval of lexer/parser
recognition does not imply approval of the language's semantics, product
identity, extension, license, or runtime architecture.

The owner-reviewable decision packages are
`docs/spec/g1-l-lexical-decision-package.md` and
`docs/spec/g1-s-syntax-decision-package.md`; the consolidated owner decision
matrix is `docs/spec/phase-2-entry-readiness.md` §5.

Memory management (S16), ABI (S17), and per-target runtime guarantees are
DEFERRED to their consumers; they do not block purely syntactic recognition.
Captures remain OPEN and specifically gated in typed HIR until decided.

## 2. Staged deliverables, conditional on authorization

**Two stages. Stage A approval must not silently authorize Stage B; Stage A
completion does not satisfy G1-T.**

### 2.1 Stage A — lexical and syntactic frontend

Entry requires G1, G1-L and G1-S acceptance and resolution of the applicable
G1c/G1d conditions (see §1 and `docs/spec/phase-2-entry-readiness.md` §5).

| # | Component | Required evidence |
|---|---|---|
| E1 | `cosmic-lexer` | Total tokenization over arbitrary bytes, no panic, byte spans and diagnosed invalid lexical forms; S8/S13/S14 fixtures authored afresh; contract per `docs/spec/g1-l-lexical-decision-package.md`. |
| E2 | `cosmic-syntax` lossless CST | Source-to-CST-to-source byte-for-byte property, including errors, the BOM prefix and all trivia (G1-L D-L10; G1-S §7). |
| E3 | Recovering parser | Tested recognized subset from the corrected grammar; explicit PARSE refusals for the nine unsupported families; bounded lambda/parenthesis disambiguation without an LL(1) assumption; stable recovery with >=3 independent errors (S15 Q3); OPEN semantics never cause parser rejection; finding S-G1S-1 resolved (verified and applied, 002-R5: `docs/spec/g1-s-syntax-decision-package.md` §3.3). |
| E7 | Differential harness | Isolated fresh test inputs, external pinned legacy oracle, explicit target, normalized intentional divergences (including S13 scalar columns and the S8 strict-escape rules), skip-with-reason if unavailable. Never vendor legacy fixtures. |
| E8 | Fuzz / properties | Parser termination, CST roundtrip, well-formed diagnostics and no panic under documented budget. |

### 2.2 Stage B — semantic frontend

Requires Stage A acceptance, **G1-T entry** and a separate owner authorization
of its scope.

| # | Component | Required evidence |
|---|---|---|
| E4 | Name resolution and `cosmic-sema` typed HIR | Stable typed IDs and TI-1..TI-8 verification; G1-T prerequisites met; no guessed conversions, capture, overload tie-break, equality or pattern semantics. |
| E5 | `cosmic check` | Exit 0 success, 1 semantic/syntax error, 2 CLI usage; no executable emitted. |
| E6 | Semantic feature gates | Negative tests for exact (construct, decision, operation) triples with a GATE code, error location, and no artifact; unaffected uses of the same construct continue to work. |

### 2.3 Measurement and audit, per authorized stage

| # | Component | Required evidence |
|---|---|---|
| E9 | Reproducible measurements | Execute `docs/development/measurement-plan.md` where measurable; report unmeasurable metrics as such, repeat relevant probes across JVM/JS/script. |
| E10 | Phase 2 audit | Actual artifacts, metrics, command logs, decision-gate register and CI evidence for final SHA in `docs/audits/phase-2-report.md`. |

Scope limits: no MIR, optimizer, backend, runtime or standard-library
implementation is authorized by this proposal. `.cmic` remains the primary
source extension and the parser stays independent of filename extensions
(ADR-0002 §4.1). The documented `.csmx` literal migration inventory
(`docs/adr/0001-language-identity.md` §6) is preserved for any future authorized
implementation.

## 3. Semantics not automatically approved

| Record | Evidence now available | Outstanding requirement |
|---|---|---|
| S1 | JVM nested-generic failure and limited overload samples | Reject only unresolved nested-generic *resolution* and ambiguous overload operations until an explicit rule is accepted. Reverse-order overload tie-break measurement still needed. |
| S2 | JVM soft-cast failure-to-null observation | Conversion table and structural fallback not accepted. |
| S5 | Parser-recognized switch/pattern forms; binding/destructuring arm-header syntax is executed-evidenced (002-R4) and derived by the corrected grammar (F5, 002-R5) | Normative pattern semantics (including destructuring meaning) remain unadopted; define exhaustiveness, fallthrough and labelled-break treatment; gate those checks individually. S-G1S-1 (pattern-header recognition completeness) is resolved (`docs/spec/g1-s-syntax-decision-package.md` §3.3). |
| S7 | Numeric operations partly observed | Division by zero, promotion and Bool/Int discrepancy unresolved. |
| S8 | JVM raw newline allowed, unknown escape silently consumed, triple quote rejected | Decide whether to prohibit invalid escapes as a correct divergence; lexer must fail safely on unaccepted forms. |
| S11 | JVM evaluation order, short circuit, assignment statement, scope, capture rejection observed | Proposed normative rules require acceptance; intended closure semantics still OPEN. |
| S12 | JVM string/record/class/float equality observed | Enum equality and map/set key contract NOT VERIFIED; no invented universal equality rule. |
| S13 | JVM letter identifiers and UTF-16 diagnostic columns observed | Scalar-column divergence is PROPOSED NORMATIVE RULE; combining marks and normalization OPEN. |
| S14/S15/S18 | Lossless CST, gating policy and IR invariants proposed | Obtain relevant owner acceptance; invariant-bearing code cannot rely solely on a proposal. |

S3/S4 (PortuKof/KofScript) are **OPEN owner scope/legal choices**. S16/S17
are **DEFERRED owner runtime decisions**. These decisions never silently become
accepted because a different component passes its readiness gate.

## 4. Acceptance conditions and scope limits

The following **proposed completion gates** are defined here, independently of
the Phase 1 component-entry gates. None is satisfied by this proposal:

| Gate | Measurable completion condition |
|---|---|
| **G2** | All owner-authorized frontend components satisfy their E1–E5 contracts, including CST round-trip, error recovery and `cosmic check` exit statuses. Components not authorized are explicitly excluded from this gate's scope. |
| **G2b** | Per-operation feature-gate negative tests (E6) emit stable diagnostics and produce no artifacts, while supported neighboring operations remain usable. |
| **G2c** | E7 differential harness is runnable against a separately obtained frozen oracle and records target, exit status and reproducible input; missing targets are recorded as NOT VERIFIED. |
| **G2d** | E9 results include actual measurements, reproducible commands and open evidence gaps; no unmeasured result is claimed settled. |
| **G2e** | E8 and E10 evidence plus all prescribed CI jobs succeed at the exact Phase 2 final SHA. |

The G2-family gates are assessed **per authorized stage scope**: Stage A against
its own contracts; later stages only if separately authorized. Stage A
acceptance does not satisfy Stage B conditions, and vice versa.

Phase 3 requires a further, explicit owner authorization after these applicable
completion gates are assessed. A gate can be marked satisfied only for the
component scope expressly authorized by the owner.

No compiler code, dependencies, PR merge, release, approved ADR, repository
protection change or license choice is authorized by this **proposal** itself.
Do not mechanically translate or copy legacy code, fixtures or documentation.

**END — PROPOSAL ONLY.**
