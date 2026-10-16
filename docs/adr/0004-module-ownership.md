# ADR-0004 — Module Ownership and Crate Layering

| | |
|---|---|
| **Status** | **Proposed** — awaiting owner approval. The ownership assignments cannot be made by engineering. |
| **Date** | 2026-10-16 |
| **Deciders** | Project owner (owners); engineering (the layering proposal) |
| **Phase** | 1 |
| **Related** | ADR-0003, ADR-0005, `docs/architecture/compiler-pipeline.md`, `docs/spec/decisions/S17-abi-versioning.md`, `docs/audits/phase-0-design-conformance.md` (rows P13-2, P19-3) |

---

## 1. Context

Two separate gaps converge here.

**Gap 1 — no per-module owners.** The design report's governance section
(page 19) lists "proprietários por módulo" (per-module owners) as a mitigation
for the project's main organisational risk: the report names "ausência de
equipe dedicada" (absence of a dedicated team) as a top-tier risk. The Phase 0
review recorded this as **not done**: the conformance audit's row **P19-3**
classifies the governance mitigations as **PARTIAL** because "no per-module
owners" exist. Engineering cannot assign owners to itself.

**Gap 2 — the module list is incomplete and unowned.** The design report
(page 13) names the target modules:

`cosmic-source`, `cosmic-lexer`, `cosmic-syntax`, `cosmic-hir`, `cosmic-sema`,
`cosmic-mir`, `cosmic-opt`, `cosmic-codegen-*`, `cosmic-runtime`, `cosmic-cli`.

Phase 0 shipped **`cosmic-diagnostics`**, which is **not on that list** — a
justified addition (it is a cross-cutting service, not a pipeline stage), but
one the conformance audit records as a deviation (row **P13-2**). An unlisted
crate with no stated reason is exactly how a module list drifts.

There is also a structural constraint Phase 0 established and which this ADR
must preserve:

* **E1** — the workspace has an explicit rule that a crate is added **only**
  when it has a concrete contract, tests and a justified responsibility; empty
  crates are not created to mirror a diagram
  (`docs/architecture/compiler-pipeline.md` §1). Phase 0 therefore has three
  crates, not ten.
* **E1** — the legacy project's own layering defect is the cautionary tale: it
  has **two runtimes** (a `kof-runtime` module that the JVM backend does not
  use, plus a generated runtime compiled in place with `javac`), which the
  design report page 8 exists to fix, and which forces a full JDK at run time
  for JVM output.
* **E1** — the legacy architecture documentation drifted from its code in six
  independent figures (Phase 0 conformance audit §3). A module list with no
  owner is a list that drifts.

---

## 2. Decision 1 — Crate layering (engineering, proposed)

**Proposed layering**, in dependency order. A dependency may only point
**downward** in this table.

| Layer | Crate(s) | Responsibility | May depend on |
|---|---|---|---|
| L4 Driver | `cosmic-cli` | Command surface, argument handling, exit codes, wiring | L1–L3 |
| L3 Analysis | `cosmic-hir`, `cosmic-sema`, `cosmic-mir`, `cosmic-opt`, `cosmic-codegen-*` | Name resolution, typing, middle-end, code generation | L1, L2 |
| L2 Frontend | `cosmic-lexer`, `cosmic-syntax` | Tokens, lossless CST, AST | L1 |
| L1 Foundation | `cosmic-source`, `cosmic-diagnostics` | File registry, spans, line/column; diagnostic data + rendering | **nothing** |
| L0 Runtime | `cosmic-runtime` | The semantically specified runtime and the versioned ABI | **nothing** |

**Rules:**

1. **L0 depends on nothing.** `cosmic-runtime` must not depend on any frontend
   crate. This is the design report's page 8 ban ("proibir dependências
   reversas runtime → frontend"). S17 proposes enforcing it as a **crate-graph
   invariant** rather than a convention, because a convention does not survive.
2. **`cosmic-diagnostics` is a cross-cutting service, not a pipeline stage.**
   It sits at L1 beside `cosmic-source` and is depended on by every layer above.
   It is added to the module list with this justification — closing audit row
   P13-2.
3. **No empty crates.** A crate appears in the workspace only when it has a
   contract and tests (`compiler-pipeline.md` §1). The table above is a
   *target*, not an instruction to create ten crates now.
4. **`cosmic-codegen-*` is a family, not a crate.** Each backend gets its own
   crate only when it exists.
5. **No cycles, ever.** If a cycle appears, the layering is wrong, not the
   rule.

**Explicitly rejected:** mirroring the legacy module structure
(`kof-compiler` + `kof-cli` + `kof-runtime` + `kof-script` + `kof-c-compiler`).
That structure is what produced the two-runtime defect; reproducing it would
reproduce the defect.

---

## 3. Decision 2 — Module ownership (owner-blocked)

**Not taken.** Engineering cannot assign owners.

**Proposed ownership model** for the owner to fill in:

| Module | Owner | Escalation |
|---|---|---|
| `cosmic-source` | *(owner to assign)* | Project owner |
| `cosmic-diagnostics` | *(owner to assign)* | Project owner |
| `cosmic-lexer` | *(owner to assign)* | Project owner |
| `cosmic-syntax` | *(owner to assign)* | Project owner |
| `cosmic-hir` / `cosmic-sema` | *(owner to assign)* | Project owner |
| `cosmic-mir` / `cosmic-opt` | *(owner to assign)* | Project owner |
| `cosmic-codegen-*` | *(owner to assign)*, one per backend | Project owner |
| `cosmic-runtime` | *(owner to assign)* | Project owner |
| `cosmic-cli` | *(owner to assign)* | Project owner |
| Standard library | *(owner to assign)* | Project owner |
| Compatibility matrix | *(owner to assign)* | Project owner |
| Decision register (`docs/spec/decisions/`) | *(owner to assign)* | Project owner |

**Minimum viable ownership.** The design report's risk is the *absence* of a
dedicated team. A single-maintainer repository cannot have per-module owners in
a meaningful sense, so the honest options are:

| Option | Shape | Assessment |
|---|---|---|
| **C1** | **One owner for everything** (the project owner), with ownership recorded explicitly | Honest for a single-maintainer repository. It satisfies the *documentation* requirement and removes the pretence. It does **not** mitigate the bus-factor risk — and the ADR should say so rather than implying it does |
| **C2** | **Assign owners per module as contributors appear** | Realistic and incremental. Requires a rule for *when* an owner is required: proposed — before a module's first release, not before its first commit |
| **C3** | **Leave ownership unassigned** | This is the current state, and it is why audit row P19-3 is PARTIAL |

**Recommendation: C1 now, C2 as contributors appear.** Recording "the project
owner owns everything" is strictly better than recording nothing, because it
makes the bus-factor risk visible instead of implicit. A vacant ownership field
looks like an oversight; an explicit single-owner field looks like a stated
risk.

---

## 4. Decision 3 — The compatibility-matrix and decision-register owners

The compatibility matrix (`docs/spec/compatibility-matrix.md`) and the decision
register (`docs/spec/decisions/`) are **artefacts with a maintenance
obligation**, not one-off documents. The register's rules require that every
open question be closed by evidence or an owner act, and that every behavioural
divergence be recorded. Without an owner, both will drift — and the legacy
project's documentation drift (six figures in the Phase 0 audit §3) is the
precedent for exactly how that happens.

**Proposed:** the decision register's owner is responsible for ensuring that no
decision record stays **OPEN** without a stated dependency, and that every
**PROPOSED** record is either accepted or rejected by the owner. This is a
process rule, and it is the mechanism that keeps Phase 1 from becoming
decorative.

---

## 5. Consequences

* The layering gives `docs/architecture/compiler-pipeline.md` a dependency
  direction it currently only implies. That document should be updated to show
  the layers once this ADR is accepted.
* `cosmic-diagnostics`'s inclusion closes a Phase 0 audit deviation.
* Per-module ownership as a *documentation* act is cheap; as a *mitigation* it
  is not available to a single-maintainer repository, and the ADR says so
  plainly rather than claiming the risk is handled.
* The layering is **testable**: a dependency-direction violation is a build
  failure, which is the only kind of governance that does not rot.

---

## 6. Status

* **Proposed.** The layering is engineering's proposal; the ownership model
  requires the owner.
* **Blocking:** nothing in Phase 2 (the frontend crates are L1/L2 and their
  direction is already correct in Phase 0's three-crate workspace).
* **Blocking later:** Phase 3/4, where the L0 boundary and the backend family
  become real, and where a dependency-direction violation would be expensive to
  unwind.
