# ADR-0001 — Language Identity, Source Extension and Compatibility Policy

| | |
|---|---|
| **Status** | **Proposed** — awaiting owner approval. Nothing in this ADR is final except where marked *(decided)*. |
| **Date** | 2026-10-16 |
| **Deciders** | Project owner (final authority); engineering (author of the proposal) |
| **Related** | `docs/audits/phase-0-baseline.md` §3.7/§6, `docs/audits/phase-0-design-conformance.md` §3, `docs/audits/phase-0-licensing-matrix.md`, `docs/spec/compatibility-matrix.md`, `docs/spec/compatibility-policy.md` |

---

## 1. Context

The legacy compiler `CosmicLang/cosmcc` (verified at SHA `284358f4…`, version
`0.6.5`) implements a language called **Kof** with the compiler CLI `kof`,
accepting source files with the extensions **`.kf`**, **`.kof`**, **`.ptkf`**
(PortuKof, a Portuguese keyword surface) and **`.ks`** (KofScript, a
statement-script format that is materialized into a Kof `main()` before
compilation). Project configuration uses `kof.toml`; dependency locking uses a
generated `kofdeps.lock` (no `kof.lock` exists).

Directive 001 proposes a new identity for the successor Rust toolchain:
language **Cosmic-Lang**, compiler **`cosmcc-rs`**, executable **`cosmic`**,
source extension **`.csmx`** — and explicitly states that `.csmx` uniqueness
"has not been established" and must be investigated before final approval,
and that the legacy `.kf`/`.ks` formats may require compatibility support that
must not be abandoned without a compatibility analysis.

## 2. Proposed decision

1. **Language name:** Cosmic-Lang. **Compiler:** `cosmcc-rs`. **Executable:**
   `cosmic`. *(follows the directive; adopted by the Phase 0 workspace)*
2. **Primary source extension:** `.csmx`, **proposed** pending the collision
   investigation in §3 and owner approval.
3. **Legacy compatibility:** `.kf` (Kof), `.kof`, `.ptkf` (PortuKof) and
   `.ks` (KofScript) remain *recognized legacy formats* for the purpose of a
   future compatibility analysis. **No decision is taken now** on whether,
   when, or how the new compiler reads them; that requires the semantic
   compatibility work listed in §5.
4. **Project files:** the proposed names `Cosmic.toml` / `Cosmic.lock` are
   recorded as future names only. No package management is implemented in
   Phase 0, and none will be before the format and dependency-resolution
   semantics are specified. (The legacy lessons are recorded: `kof.toml` is
   a forgiving minimal-subset parser with warnings for unknown keys; the lock
   file is generated and gitignored — the new design need not inherit either
   choice.)
5. **License:** the new repository has **no LICENSE file** and none is added
   until the owner decides. The legacy project is GPL-3.0; Phase 0 introduces
   no code translated from it (see audit §6). This blocks any decision that
   would require copying legacy implementation code.

## 3. `.csmx` collision investigation (performed 2026-10-16)

**Method:** web searches against general file-extension databases and editor-
tooling results (two independent queries), plus direct reasoning from the
legacy facts. Not performed: exhaustive GitHub code search (requires
authenticated search beyond the session) and trademark/registry queries.

**Findings:**

* No **registered file-format entry** for `.csmx` was found in the queried
  extension databases. The visible `.csmx` hits are names, not formats: a
  university course-management system called "CMSX" and an unrelated domain
  `csmx.online`.
* The neighborhood is busy: **`.csm`** is a heavily-used extension (Steam
  backups, Autodesk 3ds Max, CASL compiled programs, C++ precompiled headers
  via Micro Focus). Editors and OS "open with" heuristics often fall back to
  `*.csm` handling; this is a **tooling risk, not a claimed collision**.
* No existing editor grammar or language tooling claims `.csmx`.

**Conclusion:** `.csmx` appears usable but **worldwide uniqueness is not
established** — matching the directive's own wording. Residual risk is low
for the format registry view and moderate for tooling heuristics near `.csm`.

**Recommendation:** keep `.csmx` proposed; commission a tree-sitter grammar
and editor associations in the phase that introduces the lexer (they also
mitigate the heuristic risk, since tooling creates the association). If the
owner prefers a stronger signal, alternatives to evaluate at approval time:
`.cosmic`, `.csm` (rejected: heavily used), `.cx`.

## 4. Compatibility policy (proposed)

1. The **behavioral contract** is the legacy language as evidenced by the
   legacy repository's tests and documentation — starting with the golden
   corpus baselined in the Phase 0 audit (222/222 executed checks passed for
   jvm/js/script on 2026-10-16).
2. **No breaking language changes** are proposed by this ADR. Divergences may
   only come from explicit owner decisions informed by the compatibility
   matrix's open questions.
3. Legacy *implementation* code is never translated mechanically (GPL-3.0
   and directive principle 2). Compatibility is achieved by rebuilding
   behavior against recorded evidence.
4. PortuKof (`.ptkf`) and KofScript (`.ks`) are part of the compatibility
   surface to be analyzed, not silently dropped.
5. Known legacy defects (e.g. the maintainers' open items on nested generics
   and JS miscompiles, audit §7) are **not** part of the compatibility
   contract; they are inputs to the eventual specification.

## 5. Work required before this ADR can move from Proposed to Accepted

* Owner decision on the repository license (independent of legacy GPL code
  vs GPL-3.0 inheritance) — the options, obligations and open legal questions
  are laid out in `docs/audits/phase-0-licensing-matrix.md`. **No licence is
  chosen by this ADR or by that document.**
* Owner confirmation of language/compiler/binary names.
* A per-construct compatibility analysis of `.kf` semantics against the
  legacy suite (the matrix now contains the seed).
* A decision on `.ptkf` and `.ks` handling in the new toolchain — the proposed
  per-format framework is in `docs/spec/compatibility-policy.md` §2.
* Approval of the extension after the lexer-phase tooling work in §3, and after
  the exhaustive uniqueness search the design report requires (page 12), which
  §3 records as **not yet performed**.
* A `cosmic-std` / stdlib naming decision: the design report (page 3) names
  `cosmic-std`, and this ADR does not yet cover it
  (`docs/audits/phase-0-design-conformance.md`, row P3-1).

## 6. Consequences

* Phase 0 code and docs already use the proposed names (`cosmic`,
  `cosmcc-rs`, `Cosmic-Lang`) — renaming is cheap now and expensive later;
  hence this ADR is requested at the earliest possible gate.
* No compatibility claims are made anywhere until the matrix rows carrying
  evidence graduate from "verified in legacy run" to "verified in cosmcc-rs".
* The workspace contains no `LICENSE` file and no legacy-derived code; this
  is intentional and visible, not an omission.
