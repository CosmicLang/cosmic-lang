# Cosmic-Lang

Rust compiler toolchain for **Cosmic-Lang** — the successor toolchain
(compiler `cosmcc-rs`, executable `cosmic`) to the legacy `CosmicLang/cosmcc`
Kof compiler.

> **Status: Phase 0 — Foundation.** This repository does not compile programs
> yet, and it says so everywhere it matters: the `cosmic` CLI implements only
> `--help` and `--version`; every planned command fails with a clear error and
> exit code 2. No component reports success without performing its documented
> operation.

## What exists today

| Component | State | Evidence |
|---|---|---|
| `cosmic-source` | Implemented: source registry, checked byte spans, line/column mapping (Unicode, CRLF, lone CR) | `crates/cosmic-source` + tests |
| `cosmic-diagnostics` | Implemented: structured diagnostics, human + JSON renderers, no panics on input | `crates/cosmic-diagnostics` + tests |
| `cosmic` CLI | Bootstrap: `--help`, `--version`; planned commands fail loudly | `crates/cosmic-cli` + tests |
| CI | `cargo fmt` / `clippy -D warnings` / `test` / `doc`, plus the specification consistency checker (`python3 tools/spec-consistency.py`), on every pull request and on every branch push, including `main`; actions pinned to commit SHAs | `.github/workflows/ci.yml` |

Everything else (lexer, parser, semantics, MIR, backends, runtime, package
manager) is **not implemented**. See
`docs/architecture/compiler-pipeline.md` for the target pipeline and the
honest current state.

## Quick start

```sh
cargo test --workspace --all-targets     # run all tests
cargo run -p cosmic-cli -- --help        # the cosmic CLI
cargo run -p cosmic-cli -- --version
cargo run -p cosmic-diagnostics --example demo   # render a demo diagnostic
```

## Repository layout

```
crates/
  cosmic-source/        source registry, spans, line/column mapping
  cosmic-diagnostics/   structured diagnostics + renderers
  cosmic-cli/           the `cosmic` executable
docs/
  audits/phase-0-baseline.md                 legacy audit (cosmcc @ 284358f4, v0.6.5)
  audits/phase-0-design-conformance.md       the 20-page design report vs. the implementation
  audits/phase-0-source-review.md            source-level review of the Rust foundation
  audits/phase-0-licensing-matrix.md         licensing and provenance decision matrix (no licence chosen)
  audits/phase-1-licensing-recommendation.md Phase 1 licensing recommendation + legal dependencies
  audits/phase-1-specification-report.md     Phase 1 engineering report
  architecture/compiler-pipeline.md          target architecture + current state
  adr/0001-language-identity.md              naming / .cmic / compatibility
  adr/0002-source-extension.md               extension decided (.cmic); MIME selection (PROPOSED)
  adr/0003-standard-library-naming.md        stdlib and runtime naming (PROPOSED)
  adr/0004-module-ownership.md               crate layering + ownership (PROPOSED)
  adr/0005-memory-strategy.md                memory strategy (DECISION WITHHELD)
  spec/language-spec.md                      Phase 1 language specification (PROPOSED)
  spec/grammar.ebnf                          formal grammar (block-level evidence)
  spec/grammar-consistency-report.md         machine-checked consistency report (G0-G13)
  spec/grammar-evidence-map.md               per-production evidence map (122/122, machine-checked)
  spec/g1-l-lexical-decision-package.md      G1-L lexical entry decision package (ACCEPTED — 003-G1)
  spec/g1-s-syntax-decision-package.md       G1-S syntax entry decision package (ACCEPTED — 003-G1)
  spec/phase-2-entry-readiness.md            Phase 2 entry readiness + owner decision matrix
  spec/typing-contracts.md                   type-system contracts
  spec/diagnostic-codes.md                   diagnostic code registry + stability policy
  spec/compatibility-matrix.md               language-construct evidence ledger
  spec/compatibility-policy.md               per-format compatibility proposal
  spec/extension-assessment.md               extension collision investigations + owner selection (.cmic)
  spec/decisions/                            S1-S18 semantic decision register
  development/toolchain-policy.md            Rust pin, dependency and lint policy
  development/measurement-plan.md            reproducible measurement specifications
  development/test-strategy.md               test ladder + differential harness design
  directives/directive-002-proposal.md       proposed Phase 1 (specification-only)
  directives/directive-003-proposal.md       proposed Phase 2 frontend, staged (NOT AUTHORISED; revised 002-R4)
```

## Phase 1 status (specification)

Phase 1 produced the **language specification, grammar, typing contracts, 18
semantic decision records and five ADRs** — all **Proposed**. No compiler
implementation was added: the workspace still contains exactly the three Phase 0
crates.

The authoritative register is `docs/spec/decisions/README.md`; the component
entry gates are `docs/spec/implementation-gates.md`. The Stage A gates G1, G1-L
and G1-S were accepted by the owner and the G1c Stage A-limited deferral
approved (ENGINEERING DIRECTIVE 003-G1, 2026-10-16); **G1d remains PENDING** (no
licence applied, no legal clearance). The six normative statuses distinguish **VERIFIED LEGACY BEHAVIOR**,
**PROPOSED NORMATIVE RULE**, **ACCEPTED NORMATIVE RULE**, **OPEN**, **DEFERRED**
and **NOT VERIFIED**. Their use is per question: a record can contain several.
The Directive 002-R1 read-only JVM probes (`docs/audits/phase-1-legacy-probes.md`)
resolved some evidence gaps but do not establish cross-target compatibility or
approve semantics. Unsupported semantic operations must be refused by their
specific `GATE` diagnostic; unrelated syntax remains recognizable when evidenced.
Owner acceptances are recorded at individual-rule granularity in the affected
records (S5, S8, S13, S14 — 003-G1 updates).

**Phase 2 is not authorised.** The proposed Directive 003 is
`docs/directives/directive-003-proposal.md` (revised, staged, still NOT
AUTHORIZED); the entry ledger is `docs/spec/phase-2-entry-readiness.md`, with
`docs/spec/g1-l-lexical-decision-package.md` and
`docs/spec/g1-s-syntax-decision-package.md` now recording the accepted Stage A
gates. **Stage A implementation remains unauthorised until G1d is cleared and a
separate directive authorizes it.**

## Phase 0 review status

Phase 0 was reviewed against the original 20-page design report
(`docs/audits/phase-0-design-conformance.md`). The review found that the
report's factual figures were drawn from the legacy repository's own
documentation, which is measurably stale in six places — Java release, ASM
version, IR operation count and the Lexer/Parser/Sema line counts. The
inspected code is authoritative: the legacy build baseline is **Java 25**, and
the IR has **37** unsealed operation records.

Two defects were found in the foundation and fixed narrowly (a CRLF column
inconsistency and an allocation-limit bypass in `SourceMap::load`); see
`docs/audits/phase-0-source-review.md`.

## Project identity (extension decided; see ADR-0001 and ADR-0002)

* **Language:** Cosmic-Lang
* **Compiler:** `cosmcc-rs`
* **Executable:** `cosmic`
* **Source extension:** `.cmic` — **selected by the project owner** on
  2026-10-16. GitHub code search returned zero indexed matches for the extension
  at selection time; the round-2 `.csmx`/`.cosmic` collision investigations are
  retained as history in `docs/spec/extension-assessment.md`. **Worldwide
  uniqueness is not claimed**, and the **MIME type and editor integration remain
  separate, undecided matters.**
* **Standard library:** naming proposed in ADR-0003, **pending owner
  acceptance**.

## Legacy baseline

The legacy Kof compiler was baselined at `CosmicLang/cosmcc`
`284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5): build reproduces, and
74/74 golden cases passed on jvm, js and script targets (222/222 checks) on
2026-10-16. Native e2e was not executed on that host (GNU-ELF assembly vs the
macOS assembler). Full details, including licensing observations, are in
`docs/audits/phase-0-baseline.md`.

## License

**Intended: Apache-2.0 (conditional — not applied).** The owner selected Apache
License 2.0 as the intended outbound licence (2026-10-16), conditional on
copyright-ownership verification and a qualified legal review (G1d, still
PENDING). **No LICENSE file has been added and no grant is in effect**; until
the licence is formally authorised and applied, treat all content as "all
rights reserved". The legacy `cosmcc` is GPL-3.0; this repository contains no
code translated or copied from it (ADR-0001 §2.5).
