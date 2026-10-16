# S6 — Concurrency Contract

| | |
|---|---|

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

| **Status** | **OPEN** |
| **Owner** | Engineering (analysis); project owner (acceptance) |
| **Phase** | Phase 1 decision; per-target validation in Phase 3/4 |
| **Related** | `docs/spec/language-spec.md` §10, `docs/spec/grammar.ebnf` §3.7 |

## Question

1. **Q1** — What is the **type** of `spawn <expr>`? Is it a distinct handle type,
   or the expression's result type wrapped?
2. **Q2** — What does `await <expr>` accept, and what does it return?
3. **Q3** — How do **failures** propagate from a spawned computation to the
   awaiter — exception, result value, or undefined behaviour?
4. **Q4** — Is there **cancellation**, timeout, or `select`-style waiting?
5. **Q5** — What **scheduling guarantees** does each target provide, and may a
   program observe the difference?
6. **Q6** — Is the concurrency contract **normative per target**, or uniform
   with documented per-target degradations?

## Evidence

* **E1** — `spawn` and `await` are **prefix expression operators**, not
  statements: `ExpressionParser.parseUnary` consumes them and lowers each to a
  synthetic call (`__kof_spawn_expr` / `__kof_await`). The parser creates no new
  AST node for them.
* **E1** — The corpus case `concurrency-spawn-await` is executed and passes
  222/222 on jvm/js/script. It exercises `val a = spawn …` and
  `var va = await a`, i.e. a spawned call bound to a value and later awaited.
* **E1** — The design report (page 7) states that the JS target announces
  **sequential** `spawn`/`await` while other targets use genuinely concurrent
  mechanisms. The legacy Android target documentation records that the JS path
  is sequential.
* **E1** — The legacy ledger records a **measured cross-target divergence** in
  a different area (integer shift semantics differing on riscv64/aarch64/JS from
  the JVM; `cosmcc:docs/status.md` §698 at the frozen SHA). This establishes that
  per-target observable divergence is a real, documented phenomenon in this
  project, not a theoretical risk.
* **E1** — The legacy compiler performs **compile-time** concurrency checks:
  a spawn-capture scanner runs as part of statement analysis, and the memory
  diagnostics family includes concurrency-related codes. So capture legality is
  a static concern, not only a runtime one.
* **E1 (negative)** — No case exercises: failure propagation, cancellation,
  timeouts, nested spawn, or a data race. There is **no** evidence for Q3/Q4.

## Options

| # | Option | Assessment |
|---|---|---|
| F1 | **Uniform normative semantics, with a declared per-target capability table**; a target that cannot honour a guarantee must refuse at compile time (feature gate), never silently degrade | Matches the design report's own rule ("unsupported = clear diagnostic, never silently degraded generation") and the project's "never silently degrade" principle. Cost: some targets lose features |
| F2 | **Per-target semantics**: the language says nothing normative; each target documents its own behaviour | Maximum target freedom, minimum promise. But then "the same program" means different things on different targets, which is precisely the risk the design report raises |
| F3 | **Sequential everywhere** (make the JS behaviour universal) | Simple and fully reproducible, and it removes the whole class of races. Cost: it discards real concurrency on the JVM/native targets, and it would be a C2 divergence for any legacy program that relies on parallelism for progress |
| F4 | **Defer** | Leaves the typed HIR unable to type `spawn`/`await` |

## Recommendation (PROPOSED, not in force)

**F1, with F3's determinism applied to the *observable ordering* guarantees.**

* **Q1/Q2** — `spawn <expr>` yields a **distinct handle type parameterised by
  the expression's result type**; `await` accepts a handle and yields that
  result type. Rationale: the legacy lowering turns them into calls and the
  corpus binds the handle with `val`/`var`, so the handle is a value; making it
  a distinct type is what lets the type checker reject `await` on a non-handle.
  The exact surface name of the handle type is **OPEN** (a naming decision).
* **Q3** — **Failures propagate to the awaiter**: `await` re-raises the failure
  in the awaiting computation. Rationale: the legacy `RunResult` model carries
  an exit code and stderr separately, and a spawn that fails silently would be a
  silent-degradation failure mode. **This is a proposal, not evidence** — no
  case exercises it.
* **Q4** — **No cancellation, timeout or select is proposed.** No evidence. If
  needed, it requires its own record.
* **Q5/Q6** — Adopt **F1**: the contract is uniform and normative; each target
  publishes a capability table; a target that cannot provide a guarantee
  **refuses the program with a clear diagnostic** rather than degrading. The
  capability table's schema is owned by S15 (feature gates).
* **Ordering** — The contract must **not** promise any interleaving order. A
  conforming program may not observe scheduling; only `await` creates an
  ordering point. This is the only rule that makes the JS-sequential and
  JVM-concurrent targets both conforming.

## Compatibility consequence

* F1 is a **C2 divergence** for any legacy program that relies on
  target-specific concurrency behaviour (for example, one that depends on JS's
  sequential execution for determinism). Such programs would either conform
  (if they only use `await` for ordering) or be refused.
* **C3 is not promised** for any target, so differing thread pools and
  schedulers are explicitly out of scope.
* The design report's page 7 criterion — "an automated matrix with golden tests
  run on each backend and a documented SKIP per feature" — is the mechanism that
  must produce the capability table. It belongs to Phase 4.

## Open dependencies

* The handle type's surface name is an identity decision (ADR-0001 family).
* S15 owns the feature-gate schema that Q5/Q6 depend on.
* Phase 3 must produce the differential evidence for Q3 before it becomes
  normative.
