# `cosmic-diff-harness`

The **Increment A3** differential and measurement harness for the Cosmic
frontend (Phase 2 Stage A). It exists to compare the Rust frontend against a
**separately obtained, externally provided** legacy oracle and to make the
Stage A measurement obligations reproducibly executable.

## Oracle policy

* The legacy oracle is **never** downloaded, vendored, copied, translated or
  redistributed by this repository. The harness only executes a program the
  operator points it at.
* The oracle is **optional**: ordinary `cargo test --workspace --all-targets`
  runs the embedded pipeline tests and the probe measurements without any
  oracle; oracle-dependent runs are the `#[ignore]`d
  `committed_corpus_against_the_configured_legacy_oracle` test and the binary
  run described below.
* When no oracle is configured, every result is an **explicit skip**
  (`oracle-unavailable`, reason recorded) — the harness never fabricates a
  comparison.
* The harness cannot independently verify the oracle's provenance; a pin can
  only be recorded as an operator assertion (see `--oracle-pin`).

## Commands

```console
# Differential run over the embedded corpus (skips without an oracle):
cargo run -p cosmic-diff-harness -- --report target/diff-report.md --json target/diff.json

# Electronically compare against an externally provided oracle:
COSMIC_LEGACY_ORACLE=/path/to/oracle \
cargo run -p cosmic-diff-harness -- \
    --oracle-arg '{input}' --oracle-pin 'legacy@284358f43db885ccd8b52b11f897ac0defec10de' \
    --json target/diff.json --report target/diff-report.md

# Corpus expectations only (no oracle), probe measurement report,
# and the in-process frontend timing helper:
cargo run -p cosmic-diff-harness -- --self-check
cargo run -p cosmic-diff-harness -- --emit-probes target/probes.md
cargo run -p cosmic-diff-harness --release -- --measure file.csmx --repeat 30
```

## Exit codes (binary)

| Code | Meaning |
|---|---|
| 0 | Compared; no unconfirmed differences. |
| 1 | Compared; unconfirmed differences need triage. |
| 2 | Harness usage, corpus-integrity or probe-expectation failure. |
| 3 | Oracle not configured or not runnable: explicit skip (Rust side still validated). |

## Layout

* `src/corpus.rs` — the independently authored corpus (embedded) and the
  external-corpus loader (`manifest.tsv` format).
* `src/frontend.rs` — runs the Rust frontend on one input and checks the
  corpus expectation.
* `src/oracle.rs` — the child-process oracle runner (timeout, capture,
  execution-failure classification).
* `src/compare.rs` — the comparison semantics (agreement, declared
  divergences, unsupported families, unconfirmed differences, skips).
* `src/probes.rs` — the executed lexical probe measurements (M-7 identifier
  classes, escape contracts, other known divergences).
* `src/summary.rs` — deterministic machine-readable JSON and the
  human-readable report.
* `tools/` (repository root) — the A3 measurement and traceability scripts.

License: Apache-2.0 (with the rest of the repository). No legacy GPL-3.0
material is included in this crate or its corpus.
