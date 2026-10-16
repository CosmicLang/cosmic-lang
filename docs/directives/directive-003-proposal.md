# ENGINEERING DIRECTIVE 003 — Proposed Phase 2 Frontend

> **Status: PROPOSED NORMATIVE RULE / NOT AUTHORIZED.** Draft for owner
> consideration under Directive 002-R1. No implementation may begin from this
> document alone. Historic Directive 002 gate language (G1/G1b/G1c/G1d/G1e)
> remains an archival record; this proposal requires an explicit owner decision
> on the revised component gates in `docs/spec/implementation-gates.md`.

**Role:** Principal Rust Compiler Engineer
**Repository:** `CosmicLang/cosmic-lang`
**Proposed branch:** `phase/2-frontend`, based on `main` only after an
authorized disposition of draft PR #2.
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
| **G1-L** | Owner accepts lexical recognition and invalid-token contract; byte spans, Unicode class boundaries, raw-newline policy, unknown-escape diagnostic and recovery behavior have testable expected outcomes. |
| **G1-S** | Owner approves concrete parser recognition subset and ambiguous cases (especially lambda lookahead); syntax with no evidenced form is explicitly rejected with PARSE diagnostics, while evidenced syntax with open semantics remains parsable. |
| **G1-T** | Before type checker / typed HIR: every implemented semantic operation must have an ACCEPTED NORMATIVE RULE or owner-authorized narrow provisional contract. All others have operation-specific, tested GATE diagnostics. |
| **G1c** | Owner accepts identity/extension decision **or explicitly defers** the choice for frontend-only work; never hardcode `.csmx`, `.cosmic` or any extension in the grammar. |
| **G1d** | Owner chooses licensing **or explicitly approves internal-only independent work without legacy imports or publication** while legal review is pending. |
| **G1e** | CI green for exact branch SHA, governance/protection revalidated. |
| **G0-B** | Remains PARTIAL; documented Phase 0 exception is not reported as a pass. |

**G1b is replaced only if the owner approves the revised gate model.** The
underlying unresolved semantic questions remain OPEN. Approval of lexer/parser
recognition does not imply approval of the language's semantics, product
identity, extension, license, or runtime architecture.

Memory management (S16), ABI (S17), and per-target runtime guarantees are
DEFERRED to their consumers; they do not block purely syntactic recognition.
Captures remain OPEN and specifically gated in typed HIR until decided.

## 2. Implementation deliverables, conditional on authorization

| # | Component | Required evidence |
|---|---|---|
| E1 | `cosmic-lexer` | Total tokenization over arbitrary bytes, no panic, byte spans and diagnosed invalid lexical forms; S8/S13/S14 fixtures authored afresh. |
| E2 | `cosmic-syntax` lossless CST | Source-to-CST-to-source byte-for-byte property, including errors and all trivia. |
| E3 | Recovering parser | Tested recognized subset from the grammar, explicit PARSE refusals for *unsupported syntax*, bounded lookahead resolution, stable recovery with >=3 independent errors. OPEN **semantics alone never cause parser rejection**. |
| E4 | Name resolution and `cosmic-sema` typed HIR | Stable typed IDs and TI-1..TI-8 verification; G1-T prerequisites met; no guessed conversions, capture, overload tie-break, equality or pattern semantics. |
| E5 | `cosmic check` | Exit 0 success, 1 semantic/syntax error, 2 CLI usage; no executable emitted. |
| E6 | Semantic feature gates | Negative tests for exact (construct, decision, operation) triples with a GATE code, error location, and no artifact; unaffected uses of the same construct continue to work. |
| E7 | Differential harness | Isolated fresh test inputs, external pinned legacy oracle, explicit target, normalized intentional divergences, skip-with-reason if unavailable. Never vendor legacy fixtures. |
| E8 | Fuzz / properties | Parser termination, CST roundtrip, well-formed diagnostics and no panic under documented budget. |
| E9 | Reproducible measurements | Execute `docs/development/measurement-plan.md` where measurable; report unmeasurable metrics as such, repeat relevant probes across JVM/JS/script. |
| E10 | Phase 2 audit | Actual artifacts, metrics, command logs, decision-gate register and CI evidence for final SHA in `docs/audits/phase-2-report.md`. |

## 3. Semantics not automatically approved

| Record | Evidence now available | Outstanding requirement |
|---|---|---|
| S1 | JVM nested-generic failure and limited overload samples | Reject only unresolved nested-generic *resolution* and ambiguous overload operations until an explicit rule is accepted. Reverse-order overload tie-break measurement still needed. |
| S2 | JVM soft-cast failure-to-null observation | Conversion table and structural fallback not accepted. |
| S5 | Parser-recognized switch/pattern forms | Define exhaustiveness, fallthrough and labeled-break treatment; gate those checks individually. |
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

Phase 3 requires a further, explicit owner authorization after these applicable
completion gates are assessed. A gate can be marked satisfied only for the
component scope expressly authorized by the owner.

No compiler code, dependencies, PR merge, release, approved ADR, repository
protection change or license choice is authorized by this **proposal** itself.
Do not mechanically translate or copy legacy code, fixtures or documentation.

**END — PROPOSAL ONLY.**
