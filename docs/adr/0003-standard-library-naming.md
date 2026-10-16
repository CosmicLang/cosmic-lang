# ADR-0003 — Standard Library and Runtime Naming

| | |
|---|---|
| **Status** | **Proposed** — awaiting owner approval. No name is decided by this ADR. |
| **Date** | 2026-10-16 |
| **Deciders** | Project owner (final authority); engineering (author of the proposal) |
| **Phase** | 1 |
| **Related** | ADR-0001, ADR-0004, `docs/spec/decisions/S9-stdlib-boundary.md`, `docs/spec/decisions/S17-abi-versioning.md` |

---

## 1. Context

The design report uses **two different namings** for the library layer, in two
different places, and never reconciles them:

| Report page | Wording |
|---|---|
| 3 | The identity action names **`cosmic-std`** alongside `cosmcc`, `cosmic` and `.csmx` |
| 17 | "cosmic.core, cosmic.io, cosmic.time and cosmic.net minimal, with per-target capability and safety contracts" |

So the report proposes a **single** library named `cosmic-std` *and* a **split**
of four namespaces. Both cannot be the whole story, and the Phase 0 review
recorded the gap: the conformance audit's row **P3-1** classifies the identity
action as **PARTIAL** precisely because "`cosmic-std` naming is not covered" by
ADR-0001.

Additionally, the design report's page 8 requires the runtime to be separated
from the standard library, and page 17 requires a tooling set
(`cosmic fmt`, `check`, `build`, `test`, `run`, `doc`, `lsp`), a manifest
(`Cosmic.toml`) and a lock file (`Cosmic.lock`).

Naming is **not** a technical decision that engineering may take: it is part of
the project's public identity, it appears in user-visible diagnostics and
documentation, and changing it after release is expensive. It is therefore
owner-blocked.

---

## 2. The naming surfaces that must be decided together

Naming these independently produces exactly the divergence the design report
warns about on page 3 ("namespaces, executables, Maven artifacts, docs,
telemetry and packages diverge"). The surfaces are:

| # | Surface | Example | Notes |
|---|---|---|---|
| N1 | The **library** as a whole | `cosmic-std`? | The report's page 3 name |
| N2 | The **library namespaces** used in source | `cosmic.core`, `cosmic.io`, `cosmic.time`, `cosmic.net`? | The report's page 17 names. These appear **in user programs**, so they are the most expensive to change |
| N3 | The **runtime** (not user-visible in source, but in the ABI and artifacts) | `cosmic-runtime`? | Separated from N1/N2 by page 8 |
| N4 | The **project manifest** | `Cosmic.toml`? | Report page 17 |
| N5 | The **lock file** | `Cosmic.lock`? | Report page 17. Legacy precedent: the lock was generated and gitignored, and `kof.lock` did not exist |
| N6 | The **tooling commands** | `cosmic fmt`, `cosmic check`, … | Report page 17. Phase 0 already ships `cosmic --help` / `--version` and *declares* the seven planned commands as not implemented |
| N7 | The **Rust crate names** | `cosmic-source`, `cosmic-diagnostics`, … | Phase 0 already uses the `cosmic-` prefix for all three crates |

**Constraint (E1):** N6 is already **partially settled in practice** — Phase 0
ships an executable named `cosmic` and declares the seven command words in
`crates/cosmic-cli/src/lib.rs`. N7 is likewise already in use for three crates.
So this ADR cannot choose freely; it must either confirm the existing usage or
explicitly authorise a rename (which is cheap now and expensive later — the
same argument ADR-0001 §6 makes).

---

## 3. Options

| Option | Shape | Assessment |
|---|---|---|
| **B1** | **One library, one namespace**: `cosmic-std`, with modules inside it (`cosmic-std::io`) rather than dotted source namespaces | Simplest to name and version; matches report page 3. But report page 17's dotted names (`cosmic.io`) are what appear *in source*, and a single crate name does not provide them |
| **B2** | **Split namespaces**: `cosmic.core`, `cosmic.io`, `cosmic.time`, `cosmic.net` as source-visible namespaces, backed by one or more crates | Matches report page 17 exactly and matches the legacy shape, where the corpus calls dotted namespaces (`math.*`, `json.*`, `time.*` — 24 of 74 golden cases are namespace cases). Cost: more names to stabilise and version |
| **B3** | **Hybrid**: a library *product* named `cosmic-std`, exposing the four dotted namespaces from report page 17 | Reconciles the two report pages instead of choosing between them. Cost: two names for one thing, which needs a clear rule about which appears where |
| **B4** | **Defer entirely** | Acceptable for N3–N7 in principle, but N2 is needed as soon as a single program calls `println`, and N6/N7 are already in use |

---

## 4. Decision

**Not taken.** Engineering's recommendation:

**Recommend B3 (hybrid), with the following allocation:**

| Surface | Proposed name | Rationale |
|---|---|---|
| N1 library product | `cosmic-std` | The report's page 3 name; it is the *distribution* identity, used in manifests, release notes and version strings |
| N2 source namespaces | `cosmic.core`, `cosmic.io`, `cosmic.time`, `cosmic.net` | The report's page 17 names; these are what user programs write, so they follow the report's most concrete statement. The corpus evidence (24 namespace cases) shows a dotted-namespace surface is the language's actual shape |
| N3 runtime | `cosmic-runtime` | Matches the Phase 0 crate prefix and the design report's module list (page 13) |
| N4 manifest | `Cosmic.toml` | Report page 17; capitalised to match the project name |
| N5 lock file | `Cosmic.lock` | Report page 17. **Explicitly not** the legacy behaviour: the legacy lock was generated *and gitignored*; whether the new lock is committed is a separate decision (see §5) |
| N6 tooling | `cosmic fmt\|check\|build\|test\|run\|doc\|lsp` | Already declared by Phase 0; confirming is cheaper than renaming |
| N7 crates | `cosmic-<component>` | Already in use for `cosmic-source`, `cosmic-diagnostics`, `cosmic-cli` |

**Rationale for B3 over B1/B2:** the report states both names, and the two
statements are not in conflict once the *layer* is distinguished — `cosmic-std`
is the product, the dotted names are its source-visible surface. Choosing B1
would discard the source-visible names that 24 of the 74 executed cases depend
on the shape of; choosing B2 would discard the distribution identity.

---

## 5. Consequences and open sub-questions

* **Consequence:** a program's source contains `cosmic.io.…`-style names, so N2
  becomes part of the **compatibility contract** the moment the library ships.
  It must therefore be decided **before** the library is implemented, not after.
* **Consequence:** `println` and the other evidenced builtins must be assigned
  to one of the four namespaces. `docs/spec/decisions/S9-stdlib-boundary.md` Q1
  establishes that *everything callable is library*; this ADR must then say
  **which** library namespace owns the entry-point builtins. That is an open
  sub-question and is flagged, not answered.
* **Open sub-question:** is `Cosmic.lock` committed to version control? The
  legacy lock was gitignored. Phase 0's toolchain policy commits `Cargo.lock`
  for the *compiler*. These are different locks and the choice must be made
  deliberately.
* **Open sub-question:** does `cosmic-std` version independently of the
  compiler? If yes, the ABI versioning rules of S17 apply to it; if no, the
  release cadence is coupled.
* **Not decided here:** the tooling command *set* beyond the seven names, the
  manifest schema, and the package/dependency naming inside the manifest.

---

## 6. Status

* **Proposed.** Owner approval required.
* **Blocking:** the standard-library specification (S9 Q5) cannot be written
  without N2; therefore the library work is blocked on this ADR.
* **Not blocking:** Phase 2's lexer, parser and typed frontend. Those do not
  reference library names.
* **Cost of delay:** low now, high after the first release, because N2 appears
  in user source.
