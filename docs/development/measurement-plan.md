# Measurement Plan

| | |
|---|---|
| **Status** | **Proposed — Phase 1 draft.** No measurement in this document has been taken. |
| **Phase** | 1 (plan); executed from Phase 2 onward |
| **Directive** | ENGINEERING DIRECTIVE 002, deliverable D7 |
| **Related** | `docs/audits/phase-0-design-conformance.md` (row P2-3, P19-4), `docs/development/toolchain-policy.md` §5 |

---

## 1. Purpose and the rule that governs it

The design report (page 2) requires measurement of coverage, dependencies,
duplication, cyclomatic complexity, cold-start, throughput and artifact size.
Its governance section (page 19) lists a metric set: ICE rate, incremental build
time, per-feature parity, conformance, post-release regressions, memory
consumption and P0 bug time-to-resolution.

The Phase 0 review recorded all of this as **not measured** — conformance-audit
row **P2-3** (PARTIAL) and row **P19-4** (DEFERRED) — and the toolchain policy
states plainly that no measurement infrastructure exists.

**The governing rule of this plan: do not manufacture measurements.** Every
number below is marked with how it will be obtained and *when it can first be
obtained*. No figure is estimated, and no figure is presented as if it existed.
This mirrors the project's evidence discipline (E1/E2/H/P) applied to numbers.

**Why this matters here specifically.** The legacy repository's own architecture
documentation had drifted from its code in six independent figures, including
line counts and an operation count. A measurement plan whose numbers are
guesses would reproduce exactly that failure.

---

## 2. Measurement status today

| Metric group | Status | Why |
|---|---|---|
| Compilation throughput | **NOT MEASURABLE** | No compiler frontend exists. There is nothing to compile |
| p50/p95 latency | **NOT MEASURABLE** | Same |
| Memory usage of the compiler | **PARTIALLY MEASURABLE** | Phase 0's crates can be measured, but they do no compilation work, so the number would be meaningless |
| Artifact size | **NOT MEASURABLE** | No artifacts are produced |
| Test coverage | **MEASURABLE NOW** | Phase 0 has tests and a workspace; line coverage can be taken |
| Dependency exposure | **MEASURABLE NOW** | Zero dependencies; the number is known and trivially verifiable |
| ICE rate | **NOT MEASURABLE** | No compiler |
| Incremental build time | **PARTIALLY MEASURABLE** | The Phase 0 workspace builds; the number is small and uninteresting until the workspace grows |

**Only two metric groups are measurable today.** That is the honest starting
point, and it is why this document is a *plan*.

---

## 3. Reproducibility protocol (applies to every measurement)

A measurement that cannot be reproduced is not evidence. Every measurement
recorded under this plan must state:

| Field | Requirement |
|---|---|
| `sha` | The exact commit measured |
| `host` | OS, OS version, CPU model, core count, RAM, storage type |
| `toolchain` | `rustc --version`, `cargo --version`; the pinned toolchain from `rust-toolchain.toml` |
| `profile` | The Cargo profile (`dev` / `release`) and any non-default flags |
| `command` | The exact command line |
| `runs` | Number of runs, and the reported statistic (median / p50 / p95 / min) |
| `date` | Date of measurement |
| `caveats` | Anything that could invalidate comparison with another run |

**Fixed machine requirement.** The design report asks for p50/p95 "on a fixed
machine". Measurements intended for trend comparison must be taken on **one
declared machine** and recorded with the fields above. Measurements from
different machines are **not** comparable and must be labelled as such.

**Warm-up.** Latency and throughput runs must state whether they include a cold
cache. Cold-start and warm-start are **different metrics** and must never be
averaged together.

---

## 4. Metric specifications

Each specification states what is measured, how, and the earliest phase in which
the number is meaningful.

### M1 — Dependency exposure

| Field | Value |
|---|---|
| Definition | Number of direct and transitive third-party dependencies, per crate and workspace-wide |
| Method | `cargo tree` for the graph; `cargo metadata` for the count; manual licence inventory per crate |
| Reported as | Direct count, transitive count, and a **licence histogram** |
| Gate | A new dependency requires the justification the toolchain policy §3 already mandates: (a) the problem, (b) why std cannot solve it, (c) an exact version requirement, (d) a maintenance/licence note |
| Earliest meaningful | **Now** (the answer is 0 direct, 0 transitive) |
| Owner | `docs/development/toolchain-policy.md` §3 |

### M2 — Test coverage

| Field | Value |
|---|---|
| Definition | Line coverage of the workspace by `cargo test --workspace --all-targets` |
| Method | `cargo llvm-cov` (requires a dev-dependency decision) or `grcov` |
| Reported as | Per-crate line coverage, plus **uncovered public API surface** (a hand-checked list, because line coverage hides untested paths that are never executed) |
| Caveat | Line coverage is a *floor*, not a quality measure. The Phase 0 source review found real defects (CRLF columns, load allocation) in a crate with 100%-passing tests, which is direct evidence that coverage and correctness are different things |
| Gate | No numeric gate is proposed. Coverage is reported, not enforced, until the codebase is large enough for a floor to be meaningful |
| Earliest meaningful | **Now** for the Phase 0 crates; re-taken each phase |
| Note | `cargo llvm-cov` is an external tool; adding it is a dependency decision under M1 |

### M3 — Cyclomatic complexity and duplication

| Field | Value |
|---|---|
| Definition | Per-function cyclomatic complexity; duplicated code blocks |
| Method | `clippy` has no complexity lint of this kind; use a dedicated tool (`cargo-geiger` is unrelated; the realistic options are `rust-code-analysis` or `clippy::cognitive_complexity`) |
| Reported as | Distribution (p50/p95/max) per crate, plus a list of outliers above a declared threshold |
| Threshold | **Not yet declared.** The legacy project used a 500-line file gate with a 500–599 tolerance band; a complexity threshold should be chosen from measured data, not in advance |
| Earliest meaningful | Phase 2 (when there is real logic to measure) |

### M4 — Compilation throughput

| Field | Value |
|---|---|
| Definition | Source bytes or lines compiled per second by the **frontend** (lex → parse → type-check), on a fixed corpus |
| Method | A benchmark harness over the conformance corpus plus synthetic large inputs (generated, not copied from legacy fixtures) |
| Corpus | The new fixture set from D8. **Legacy fixtures are not vendored** |
| Reported as | Median and p95 over N runs, with cold/warm cache stated |
| Earliest meaningful | Phase 2 (needs the frontend) |

### M5 — p50 / p95 latency

| Field | Value |
|---|---|
| Definition | Wall-clock latency of a full compiler invocation on a fixed corpus, reported as a distribution |
| Method | Same harness as M4, measuring the whole process including startup |
| Separately reported | **Cold start** (first invocation after a cache purge) versus **warm start** |
| Reported as | p50, p95, and the sample count |
| Earliest meaningful | Phase 2 |

### M6 — Compiler memory usage

| Field | Value |
|---|---|
| Definition | Peak resident set size of a compiler invocation |
| Method | `/usr/bin/time -l` on macOS (the audit host) and `/usr/bin/time -v` on Linux (the CI host); or a `getrusage`-based harness |
| **Precedent** | The Phase 0 source review measured this successfully: loading a 5 GiB sparse file cost ~798 MB peak RSS before the fix and ~2 MB after. So this method is **already proven on this project** |
| Reported as | Peak RSS per corpus input, plus the maximum across the corpus |
| Note | This metric is also a **defect detector**: an unexpected peak RSS is how the Phase 0 allocation defect was found |
| Earliest meaningful | Now for `cosmic-source`/`cosmic-diagnostics`; Phase 2 for the frontend |

### M7 — Artifact size

| Field | Value |
|---|---|
| Definition | Size of the produced artifact per target (executable, `.wasm`, `.class` set) |
| Method | `du`/`stat` on the artifact, with the build profile and target triple recorded |
| Reported as | Total size and a breakdown by section for native targets |
| Earliest meaningful | Phase 4 (no artifacts exist before then) |

### M8 — ICE rate

| Field | Value |
|---|---|
| Definition | Number of internal compiler errors (verification failures, unexpected panics) per unit of corpus |
| Method | Count ICE diagnostics emitted across the conformance corpus and the fuzz corpus; the diagnostic code namespace for ICE is owned by D9 |
| Target | The design report's SLO: **zero known ICE on the corpus** |
| Note | S18 establishes that a verifier failure must be an ICE with a stable code, never a panic — which is what makes this metric countable |
| Earliest meaningful | Phase 2 (needs a compiler that can ICE) |

### M9 — Incremental build time

| Field | Value |
|---|---|
| Definition | Wall-clock time for an incremental `cargo build` after a one-line edit, per crate |
| Method | Scripted edit-then-build, repeated, on the fixed machine |
| Caveat | Incremental build time is dominated by the dependency graph shape, so it is a **design signal** (is the crate graph too coarse?) rather than a quality metric |
| Earliest meaningful | Now (the workspace builds); meaningful when the workspace has more than three crates |

### M10 — Conformance and per-feature parity

| Field | Value |
|---|---|
| Definition | Fraction of conformance cases passing, per target and per language feature |
| Method | The differential harness from D8, run per target; a SKIP must carry a stated reason and is **not** counted as a pass |
| Reported as | A matrix: feature × target → pass / fail / skipped-with-reason |
| Precedent | The legacy project's own corpus reports `222 passed, 0 failed, 0 skipped` for jvm/js/script and explicitly records native as NOT EXECUTED. That reporting shape is the model |
| Earliest meaningful | Phase 2 (frontend), Phase 4 (per target) |

### M11 — Post-release regressions and P0 time-to-resolution

| Field | Value |
|---|---|
| Definition | Regressions found after a release; time from P0 report to fix |
| Method | Requires a release process and an issue tracker convention |
| Earliest meaningful | Phase 5 |
| Note | Recorded for completeness because the design report lists it; **not actionable before Phase 5** and deliberately not designed here |

---

## 5. What this plan does not do

* **It does not produce numbers.** Every metric above is a specification, and
  §2 states which are measurable today (two of eleven).
* **It does not set gates** for the metrics that have none (coverage,
  complexity). Gates should be derived from measured data, not chosen in
  advance — the legacy project's own file-size gate (500 lines, with a measured
  tolerance band and a ratchet file) is a good example of a gate derived from
  measurement rather than asserted.
* **It does not vendor legacy fixtures** to build a corpus. The corpus is
  authored from observed behaviour under the policy in
  `docs/audits/phase-0-licensing-matrix.md` §4.
* **It does not claim any of the design report's SLOs are met.** The SLOs are
  recorded as targets in M8/M10 and are unmet because the compiler does not
  exist.
* **It does not add tooling dependencies.** Several methods above (`cargo
  llvm-cov`, a complexity analyzer) require a dependency decision under M1.
  That decision is **not** taken here.

---

## 6. When to revisit

| Trigger | Action |
|---|---|
| Phase 2 completes | Take M1–M6, M8, M9, M10 for the first time; derive complexity and coverage gates from the data |
| Phase 3 completes | Re-take M4–M6 with MIR in the pipeline; M8 becomes meaningful |
| Phase 4 completes | Take M7; complete the M10 matrix per target |
| Any phase boundary | Re-run the reproducibility protocol on the fixed machine and record the trend |
| A defect found by measurement | Record it with the reproduction, as the Phase 0 allocation defect was recorded |
