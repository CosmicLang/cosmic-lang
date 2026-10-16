# S17 — Runtime ABI Versioning and Layering

| | |
|---|---|
| **Status** | **DEFERRED** — owner-dependent ABI choice; versioning mechanics are PROPOSED NORMATIVE RULE, not accepted |
| **Owner** | Project owner (naming via ADR-0003, layering via ADR-0004); engineering (mechanics) |
| **Phase** | Decision in Phase 1; implementation in Phase 3/4 |
| **Related** | ADR-0003, ADR-0004, ADR-0005, `docs/spec/decisions/S16-memory-strategy.md`, S9, S15 |

## Question

1. **Q1** — What is the **runtime's** boundary: what belongs to a
   semantically-specified runtime, to the standard library, and to per-target
   bindings?
2. **Q2** — How is the **ABI versioned**, and what is the compatibility promise
   between an ABI version and a compiled artifact?
3. **Q3** — Is the **runtime → frontend dependency ban** absolute?
4. **Q4** — What is the **naming** of the runtime and library crates (this is
   the part that is owner-blocked)?
5. **Q5** — How are **resource-management** entry points represented in the ABI,
   given that S16 is undecided?

## Evidence

* **E1 — the legacy project has a documented runtime-layering problem.** The
  design report (page 8) records that the legacy repository has a `kof-runtime`
  module with Java helper classes, **but the JVM backend does not use it in the
  expected form**: it generates its own runtime class and compiles it in place
  with `javac`. The Phase 0 audit verified this independently
  (`docs/audits/phase-0-baseline.md` §3.2, §3.6). So the legacy project has
  **two runtimes**, and the report names the consequence: divergence of
  contracts, duplication, and maintenance/distribution/observability confusion.
* **E1 — consequence of the above.** Running JVM-target output requires a full
  JDK, not just a JRE, because the generated runtime is compiled in place. That
  is a deployment consequence of the layering defect, and it is a concrete
  argument for specifying the boundary before writing a backend.
* **E1** — The design report (page 8) prescribes the fix: separate a
  **semantically specified runtime**, a **standard library**, **per-target
  bindings**, and a **versioned ABI**, and **forbid reverse dependencies from
  the runtime to the frontend**. The report's own criterion is "conformance
  tests per runtime function and an ABI/API compatibility table".
* **E1** — The legacy native target documents its runtime ABI in a dedicated
  document and exposes an FFI surface with per-backend status codes
  (`docs/audits/phase-0-baseline.md` §3.6). So a versioned, documented ABI is
  **already a legacy practice**, at least on one target.
* **E1** — The legacy `kof-script` module compiles and executes in-process
  (`KofScript.eval`) and returns a structured result
  (`RunResult(exitCode, stdout, stderr, success)`). That is an ABI-shaped
  surface between the driver and a target, and it is *not* the same shape as the
  native ABI — another instance of the layering problem.
* **E1** — Phase 0's workspace has exactly three crates and an explicit rule
  that a crate is added only with a concrete contract, tests and a justified
  responsibility (`docs/architecture/compiler-pipeline.md` §1). So the
  layering must be justified crate by crate, not drawn as a diagram and
  populated.
* **E1 (negative)** — No runtime exists in the new repository, no ABI is
  defined, and no ABI compatibility table exists. There is nothing to version
  yet.

## Options

| # | Option | Assessment |
|---|---|---|
| Q1 | **One runtime crate + one stdlib crate + per-target binding crates**, ABI versioned as a single integer, with the reverse-dependency ban enforced by the crate graph | Matches the design report's prescription exactly, and the crate graph *is* the enforcement mechanism — a reverse dependency becomes a build error, not a policy |
| Q2 | **Per-target runtime crates with no shared specification**, each versioned independently | Mirrors the legacy defect the report exists to fix |
| Q3 | **Semver per crate** rather than a single ABI version | More expressive for library consumers, but an ABI is a *binary* interface: a semver range is the wrong tool, and it makes "which ABI does this artifact need" ambiguous |
| Q4 | **Defer the whole question** to Phase 4 | Acceptable for the mechanics, but the **layering** decision must precede backend work, and the **naming** decision blocks ADR-0003 |

## Recommendation (PROPOSED, not in force)

* **Q1 — Layering.** Four layers, in dependency order:
  1. **`cosmic-runtime`** — the semantically specified runtime: strings,
     collections, allocation, errors, the scheduler, and resource management.
     **Normative**: its behaviour is part of the language's observable
     semantics, so it needs its own conformance suite.
  2. **Standard library** — everything callable from source, per S9 Q1. Built
     on the runtime.
  3. **Per-target bindings** — the platform-specific implementations of the
     runtime's operations (JVM/JS/native/WASM). Each declares its capability
     state per S15.
  4. **The driver / frontend** — depends on none of the above.

  **Enforcement:** the dependency direction is a **crate-graph invariant**.
  `cosmic-runtime` must not depend on the frontend crates, and the compiler
  must not depend on a binding. This makes the design report's ban a build
  error rather than a convention, which is the only way such bans survive.
* **Q2 — Versioning mechanics.** The ABI has a **single monotonic integer
  version**, declared by the runtime and recorded in every compiled artifact.
  Rules:
  * a compiled artifact declares the ABI version it requires;
  * a runtime **accepts** an artifact whose ABI version is ≤ its own, and
    **refuses** an artifact with a newer ABI version, with a clear diagnostic;
  * an ABI change that alters an existing entry point's meaning is a
    **major** increment and requires a decision record;
  * additive entry points are a **minor** increment;
  * the ABI version is **independent** of the language version and of the
    compiler's semver.
  Rationale: an ABI is a binary contract, and an integer with explicit accept/
  refuse rules is auditable in a way a semver range is not.
* **Q3 — The ban is absolute**, with one documented exception: a target binding
  may depend on the *diagnostic data types* (not on the frontend's analysis
  crates), because a runtime error must be reportable as a diagnostic. That
  exception is a **leaf data dependency**, not a reverse dependency on analysis.
  If the owner prefers no exception at all, the runtime defines its own error
  type and the driver converts — which is the cleaner design and is the one
  proposed here: **no exception; the runtime owns its error type.**
* **Q4 — Naming is OPEN (owner decision required).** ADR-0003 must decide between the two
  names the design report uses: `cosmic-std` (page 3) and the split
  `cosmic.core` / `cosmic.io` / `cosmic.time` / `cosmic.net` (page 17). This
  record does not choose, and the crate names above are **placeholders**.
* **Q5 — Resource-management entry points are deliberately left unversioned.**
  Their shape depends on S16 (memory strategy), which is itself
  **DEFERRED / owner decision required**. Until S16 is decided, the ABI **must not** expose
  allocation or reclamation entry points at all — which is achievable, because
  no backend exists. **This is the concrete reason S16 must precede Phase 4.**

## Compatibility consequence

* The layering decision is **C0–C3 neutral** for source compatibility: it is an
  internal architecture choice. Its observable effect is on *deployment* (the
  legacy JDK-required-at-runtime consequence) and on error reporting.
* The ABI versioning rules are **neutral** because no artifact exists yet;
  adopting them now costs nothing and prevents the legacy situation where two
  runtimes disagreed.
* **C3 is not promised**, so a different runtime implementation per target is
  explicitly allowed as long as it honours the normative runtime semantics.

## Open dependencies

* **ADR-0003** (naming) — Q4 is blocked on it.
* **S16** (memory strategy) — Q5 is blocked on it.
* **S9** (library boundary) — determines what lands in layer 2.
* **S15** (feature gates) — determines how a binding declares its capability
  state.
* The ABI compatibility table the design report requires is a Phase 3/4
  deliverable, not a Phase 1 one; this record specifies its *schema*, not its
  contents.
