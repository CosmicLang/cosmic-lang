# Cosmic-Lang

Rust compiler toolchain for **Cosmic-Lang**, the successor compiler
(`cosmcc-rs`, CLI `cosmic`) to the legacy `CosmicLang/cosmcc` Kof compiler.

This is the initial bootstrap commit. The repository is under active
development on the `phase/0-foundation` branch.

## Project identity (proposed — see `docs/adr/0001-language-identity.md`)

- **Language:** Cosmic-Lang
- **Compiler:** `cosmcc-rs`
- **Executable:** `cosmic`
- **Source extension:** `.csmx` (proposed)

## Status

Phase 0 — Foundation and Legacy Baseline. See `docs/audits/phase-0-baseline.md`
for the legacy audit and `docs/architecture/compiler-pipeline.md` for the
target architecture.

Nothing here is a finished product. An unsupported command must fail loudly;
no component reports success without executing its tests.
