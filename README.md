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
| CI | `cargo fmt` / `clippy -D warnings` / `test` / `doc` on every branch and PR | `.github/workflows/ci.yml` |

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
  audits/phase-0-baseline.md        legacy audit (cosmcc @ 284358f4, v0.6.5)
  architecture/compiler-pipeline.md target architecture + current state
  adr/0001-language-identity.md     naming / .csmx / compatibility (PROPOSED)
  spec/compatibility-matrix.md      language-construct evidence ledger
  development/toolchain-policy.md   Rust pin, dependency and lint policy
```

## Project identity (proposed — see ADR-0001)

* **Language:** Cosmic-Lang
* **Compiler:** `cosmcc-rs`
* **Executable:** `cosmic`
* **Source extension:** `.csmx` (proposed; collision investigation in
  ADR-0001 §3; final approval pending)

## Legacy baseline

The legacy Kof compiler was baselined at `CosmicLang/cosmcc`
`284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5): build reproduces, and
74/74 golden cases passed on jvm, js and script targets (222/222 checks) on
2026-10-16. Native e2e was not executed on that host (GNU-ELF assembly vs the
macOS assembler). Full details, including licensing observations, are in
`docs/audits/phase-0-baseline.md`.

## License

**Not yet chosen.** The legacy `cosmcc` is GPL-3.0; this repository contains
no code translated or copied from it, and no LICENSE file will be added until
the owner decides (ADR-0001 §2.5). Until then, treat all content as
"all rights reserved".
