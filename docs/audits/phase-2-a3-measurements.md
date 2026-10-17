# A3 frontend measurements (Stage A)

| | |
|---|---|
| **Status** | Measured by `tools/measure-a3.py` (stdlib only). Every figure below was produced by the recorded command on the recorded host; nothing is estimated. |
| **Protocol** | `docs/development/measurement-plan.md` §3 — sha, host, toolchain, profile, command, runs, date and caveats; cold and warm runs are reported separately and never averaged. |
| **Subject** | The accepted frontend (`cosmic-lexer` → `cosmic-syntax`) through `target/release/cosmic-diff-harness --measure`; no semantic analysis or code generation exists in Stage A. |

## Record fields

| field | value |
|---|---|
| `sha` | `37ecab05d3baa1c3900627ee1e5283dec06b39b4` |
| `date` | 2026-10-17 |
| `host` | arm64, macOS-27.0.1-arm64-arm-64bit, macOS 27.0.1; CPU: Apple M2; 8 cores; RAM: 8192.0 MiB; storage: solid state |
| `toolchain` | rustc 1.99.1 (aarch64-apple-darwin); cargo 1.99.1 (5f94df478 2026-08-27) (Homebrew) |
| `profile` | `release` (`cargo build --release -p cosmic-diff-harness`); no non-default flags |
| `command` | `target/release/cosmic-diff-harness --measure <input> --repeat <n>`; peak RSS: `/usr/bin/time -l target/release/cosmic-diff-harness --measure <input> --repeat 1` |
| `runs` | per table row (`p50` = lower median of the per-run sample set) |
| `caveats` | the working tree was **not clean** at measurement time; uncommitted paths: `docs/audits/phase-2-a3-measurements.json`, `docs/audits/phase-2-a3-measurements.md`, `docs/audits/phase-2-a3-traceability.md`, `tools/a3-traceability.py`, `tools/m6-cross-target.py`, `tools/measure-a3.py` |

## Method

- Inputs are generated (authored) files, never legacy fixtures: `valid_*` are repetitions of a small recognised program; the adversarial inputs reproduce the workloads named by the A2 record (`if_lines`, deep type arguments, deep parentheses) and the A3 statement soup.
- **Warm latency** is the per-run sample set inside one process (`--repeat n`); **cold invocation** is a separate process measured by `/usr/bin/time -l` (reported in the peak-RSS table). They are different metrics and are not averaged.
- Every run records `accepts`/`lossless`/`reprint_ok`; a measurement whose input was not accepted would be measuring recovery, not recognition (the `valid_*` corpus is sanity-checked before measuring).
- Adversarial runs are **bounded**: no run exceeded its per-run timeout.

## Throughput and latency (warm)

| input | bytes | runs | min | p50 | p95 | max | p50 throughput | accepted / lossless / reprints |
|---|---|---|---|---|---|---|---|
| `valid_small` | 1.0 KiB | 30 | 0.06 ms | 0.07 ms | 0.14 ms | 0.14 ms | 15.1 MiB/s | True / True / True |
| `valid_medium` | 256.0 KiB | 30 | 12.77 ms | 15.63 ms | 19.38 ms | 25.42 ms | 16.0 MiB/s | True / True / True |
| `valid_large` | 4.0 MiB | 5 | 170.43 ms | 194.68 ms | 223.37 ms | 223.37 ms | 20.5 MiB/s | True / True / True |

## Adversarial workloads (bounded)

| input | bytes | runs | min | p50 | max | result flags | notes |
|---|---|---|---|---|---|---|---|
| `if_lines_50k` | 537.1 KiB | 3 | 19.82 ms | 20.00 ms | 21.37 ms | accepts=False, lossless=True | reproduces the A2 per-trip PARSE005 workload |
| `deep_type_arguments_50k` | 146.5 KiB | 3 | 5.62 ms | 5.67 ms | 6.26 ms | accepts=False, lossless=True | 50 000 nested type arguments; implementation guard workload |
| `type_decl_lines_50k` | 1.5 MiB | 2 | 68.76 ms | 68.76 ms | 73.24 ms | accepts=False, lossless=True | nodes 1250013 |
| `parens_1m` | 1.9 MiB | 3 | 332.20 ms | 352.90 ms | 355.77 ms | accepts=False, lossless=True | 1 000 000 nested parentheses; implementation guard workload |
| `statement_soup_10k` | 97.7 KiB | 3 | 3.91 ms | 4.03 ms | 4.89 ms | accepts=False, lossless=True | mixed malformed soup; determinism covered by tests |

### Previous reported durations (baseline context)

- The A2 record reported ≈18 s (bounded) for a ~150 KB type-argument workload and 291×`PARSE005` for `if_lines` at 50 000 lines on the previous development machine; the A2 probe generator was not preserved.
- This run reproduces the *shape* with A3-authored generators: `deep_type_arguments_50k` (146.5 KiB) measures 5.67 ms p50 and `if_lines_50k` reproduces the per-trip guard workload at 20.00 ms p50. Different generator, machine and toolchain: these are **not a comparison**; the A2 figures are baseline context only, exactly as the directive states.

## Peak RSS and cold invocation

| input | peak RSS | cold wall time (`real`) | note |
|---|---|---|---|
| `valid_medium` | 42.0 MiB | 0.01 s | includes process startup |
| `valid_large` | 585.9 MiB | 0.20 s | includes process startup |
| `if_lines_50k` | 84.4 MiB | 0.02 s | includes process startup |
| `deep_type_arguments_50k` | 30.6 MiB | 0.00 s | includes process startup |
| `parens_1m` | 382.5 MiB | 0.33 s | includes process startup |

- **Observation, not a gate.** The 4 MiB recognised input peaks at 585.9 MiB (≈146× input size) with the full CST retained. Stage A declares no memory threshold; this is recorded for Stage B attention, consistent with the plan's note that peak RSS is a defect detector.

## Dependency exposure (plan M1)

- Workspace packages: 6 (`cosmic-cli`, `cosmic-diagnostics`, `cosmic-diff-harness`, `cosmic-lexer`, `cosmic-source`, `cosmic-syntax`).
- External (non-workspace) packages, direct and transitive, normal edges: **0** — the zero-dependency invariant holds.

## Panic / ICE incidence (plan M8)

- Panic events observed across all A3 measurement runs: **0**. No run panicked.
- This is a count over the measured corpus, not a claim over all inputs; the property and adversarial suites (`crates/cosmic-syntax/tests/properties.rs`, `adversarial.rs`) are the systematic complement.

## Plan-metric mapping for this increment

| plan metric | A3 status |
|---|---|
| M1 dependency exposure | **measured** (above; zero external packages) |
| M2 test coverage | **NOT MEASURED** — no coverage tooling is installed and adding one (`cargo llvm-cov`) is a dependency decision that the measurement plan §5 explicitly does not take; substitute evidence: per-suite test totals in the A3 report |
| M3 complexity/duplication | **NOT MEASURED** — no analyzer installed; no threshold declared (plan: derive from measured data, not in advance) |
| M4 compilation throughput | **measured** (frontend only; no semantic phase exists) |
| M5 p50/p95 latency | **measured** in-process; cold invocation wall time recorded separately |
| M6 compiler memory usage | **measured** (peak RSS per input, `/usr/bin/time -l`) |
| M7 artifact size | **NOT MEASURED** — no artifact exists before Phase 4 |
| M8 ICE rate | **measured as zero observed** on the A3 corpus; not a claim over all inputs |
| M9 incremental build time | **NOT MEASURED** — out of A3 scope; no edit-build tooling was added |
| M10 conformance and per-feature parity | **NOT MEASURED for legacy targets** — the differential harness is delivered and its machinery is tested, but no legacy oracle is available in this environment, so per-target conformance cannot be claimed; harness skips are recorded with reasons (see the report and the harness summary) |
| M11 post-release regressions | not applicable before Phase 5 |

## What is deliberately not claimed

- **No legacy comparison.** The legacy compiler was not present; every oracle-dependent condition is reported as NOT MEASURED, not as agreement.
- **No cross-machine claim.** These numbers are one declared host (above); the protocol states that different machines are not comparable.
- **No tuned inputs.** The adversarial generators are the workloads named by the A2 record and the A3 directive; they were not tuned for favourable numbers, and every run is bounded by the recorded timeout.
- **No coverage claim.** Test totals demonstrate activity, not coverage of the state space; M2 stays NOT MEASURED until tooling is approved.

## Regeneration

`python3 tools/measure-a3.py` (stdlib only, offline; rebuilds the release harness first). The JSON record next to this document carries every raw sample, command and flag for independent re-derivation.
