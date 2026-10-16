# ADR-0001 — Language Identity, Source Extension and Compatibility Policy

| | |
|---|---|
| **Status** | **Proposed** — awaiting owner approval. Nothing in this ADR is final except where marked *(decided)*; the primary source extension is one such owner decision. |
| **Date** | 2026-10-16 (round 1) · 2026-10-16 (round 2, Directive 002) · 2026-10-16 (post-merge Directive 002-R2: owner extension decision recorded) |
| **Deciders** | Project owner (final authority); engineering (author of the proposal) |
| **Related** | `docs/audits/phase-0-baseline.md` §3.7/§6, `docs/audits/phase-0-design-conformance.md` §3, `docs/audits/phase-0-licensing-matrix.md`, `docs/spec/compatibility-matrix.md`, `docs/spec/compatibility-policy.md`, `docs/spec/extension-assessment.md`, ADR-0002, ADR-0003, ADR-0004, ADR-0005 |
| **Superseded in part** | §2.2 (extension) is developed further by **ADR-0002**; §2.4 (project files) and the `cosmic-std` gap are developed by **ADR-0003**. This ADR remains the parent identity record. |

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
2. **Primary source extension:** `.cmic` — *(decided)* selected by the project
   owner on 2026-10-16 (post-merge Directive 002-R2) and recorded in **ADR-0002**
   §4 with the collision evidence. The `.csmx`/`.cosmic` investigations in §3 and
   `docs/spec/extension-assessment.md` are retained **as historical evidence**.
   The MIME type and editor integration remain separate, undecided matters.
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

## 3. `.csmx` collision investigation (rounds 1–2, 2026-10-16 — historical; owner selected `.cmic`)

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
* ~~No existing editor grammar or language tooling claims `.csmx`.~~
  **CORRECTED in round 2 — this finding was wrong.**

> **Round-2 correction (Directive 002, 2026-10-16).** The limitation above
> ("exhaustive GitHub code search … not performed") has now been closed, and it
> **changed the conclusion**. Authenticated GitHub code search
> (`extension:csmx`) returns 10 hits, dominated by
> **`akeit0/csharp-markup-extension`** — an MIT-licensed C# project using
> `.csmx` as a **programming-language source extension**, shipping a
> **language server**. It is a stale PoC (last push 2026-06-15, 16 stars) and
> there is no file-format-registry entry for either project, but the claim that
> "no existing language tooling claims `.csmx`" is **false**. The full record,
> including the search coverage, the control measurements (`.cosmic`: 58 hits,
> none a language claim; `.kf`: 7,056 hits) and the MIME findings, is in
> `docs/spec/extension-assessment.md`; the decision is developed in
> **ADR-0002**.

**Conclusion (revised):** `.csmx` **worldwide uniqueness is not established**,
and round 2 found a real competing language claim. Residual risk is **low for
the format-registry view** and **moderate for the tooling view** (editor and
language-server association) — not "low" as round 1 assessed. Round 1's own
recommendation to evaluate alternatives at approval time stands and is now
material: **`.cosmic`** has more raw file occupation but **no competing
language claim**, which is the property that actually matters for tooling.

**Recommendation:** keep `.csmx` **proposed**; close the two named search gaps
(direct extension-database query, package-registry query) before acceptance;
choose between `.csmx` and `.cosmic` on a **stated criterion**; `.csm` remains
rejected. Because the parser is **extension-independent**
(`docs/spec/compatibility-policy.md` §3 rule 1), a wrong choice costs a rename,
not a migration — so the investigation should be time-boxed. See ADR-0002.

> **Owner decision (post-merge Directive 002-R2, 2026-10-16).** The owner
> selected **`.cmic`** as the primary source extension; the decision and its
> evidence record are in **ADR-0002** §4. The round-1 and round-2 investigations
> above remain the historical record of why `.csmx` was not accepted and why
> `.cosmic` was the leading alternative. The selection carries the limitations
> the investigations recorded: it rests on indexed code search and public
> sources, **not** on a worldwide-uniqueness claim, and it does not publish
> anything — no MIME registration, no editor association. Those remain separate
> matters.

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
  legacy suite (the matrix now contains the seed; the evidenced construct set
  is recorded as a PROPOSED NORMATIVE RULE in `docs/spec/language-spec.md` and `docs/spec/grammar.ebnf`; no owner acceptance is implied).
* A decision on `.ptkf` and `.ks` handling in the new toolchain — the proposed
  per-format framework is in `docs/spec/compatibility-policy.md` §2, with
  decision records `S3-portukof.md` and `S4-kofscript.md`.
* **Approval of the extension** — **decided by the owner: `.cmic`**
  (2026-10-16, ADR-0002 §4). A direct extension-database check was performed
  during R2 (FileInfo: no entry for `.cmic`); a package-registry query remains
  open, and MIME type and editor/registry publication are separate matters that
  this ADR does not decide.
* **A `cosmic-std` / stdlib naming decision** — now owned by **ADR-0003**,
  which reconciles the design report's two conflicting namings (page 3's
  `cosmic-std` versus page 17's `cosmic.core`/`cosmic.io`/`cosmic.time`/
  `cosmic.net`). This closes conformance-audit row P3-1.
* **Per-module ownership** — now owned by **ADR-0004**, which also proposes the
  crate layering and closes conformance-audit rows P13-2 and P19-3.
* **The memory strategy** — owned by **ADR-0005**, whose decision is
  deliberately **withheld** pending the study the design report requires
  (page 17).

**Of these, three remain owner acts that engineering cannot perform:** the
licence, the names, and the ownership/memory decisions. The extension was an
owner act and **has been performed** (`.cmic`, 2026-10-16); its MIME and
publication follow-ups remain open. The remaining items are engineering
deliverables that are now complete.

## 6. Consequences

* Phase 0 code and docs already use the proposed names (`cosmic`,
  `cosmcc-rs`, `Cosmic-Lang`) — renaming is cheap now and expensive later;
  hence this ADR is requested at the earliest possible gate.
* The owner selected `.cmic` as the primary source extension (2026-10-16).
  Current normative documents and examples use it; historical `.csmx`/`.cosmic`
  references are retained only where they document previous proposals and the
  collision investigations. **The Rust crates are deliberately not modified**:
  43 `.csmx` filename strings remain in test fixtures and doc/runtime examples
  across `crates/cosmic-source`, `crates/cosmic-diagnostics` and
  `crates/cosmic-cli`; updating them is Phase 2 implementation work, recorded
  here so the extension-migration inventory is complete.
* No compatibility claims are made anywhere until the matrix rows carrying
  evidence graduate from "verified in legacy run" to "verified in cosmcc-rs".
* The workspace contains no `LICENSE` file and no legacy-derived code; this
  is intentional and visible, not an omission.
