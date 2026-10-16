# Source-Extension Assessment (`.csmx`) — Collision Investigation, Round 2

| | |
|---|---|
| **Status** | **Pending formal acceptance.** The extension remains **proposed**, not decided. |
| **Phase** | 1 (directive §5) |
| **Directive** | ENGINEERING DIRECTIVE 002 §5 |
| **Predecessor** | `docs/adr/0001-language-identity.md` §3 (round 1, two generic web queries) |
| **Search date** | 2026-10-16 |
| **Verdict** | **Worldwide uniqueness is NOT established — and round 2 found a real collision that round 1 missed.** |

---

## 1. Why a second round was needed

Round 1 (ADR-0001 §3) ran two generic web queries and concluded that the visible
`.csmx` hits were "names, not formats". That conclusion was reached **without
code search**, and it was recorded as an explicit limitation: "exhaustive GitHub
code search … not performed".

The directive requires the investigation to continue across "public code
repositories, extension databases, language tooling, editor associations and
MIME conventions". Round 2 therefore added **authenticated code search**, which
is the one channel that can actually distinguish "an extension in use" from "a
string that appears on a web page".

**Round 2 changed the conclusion.**

---

## 2. Search coverage (all performed 2026-10-16)

| # | Channel | Query / target | Tool | Result |
|---|---|---|---|---|
| 1 | GitHub code search (authenticated) | `extension:csmx` | `gh api search/code` | **10 hits** — see §3 |
| 2 | GitHub code search (authenticated) | `filename:.csmx` | `gh api search/code` | 11 hits (overlapping with #1) |
| 3 | GitHub code search (authenticated) | `extension:csmx repo:akeit0/csharp-markup-extension` | `gh api search/code` | 9 hits in one repository |
| 4 | GitHub repository metadata | `akeit0/csharp-markup-extension` | `gh api repos/…` | Active language project, MIT, C# — see §3.1 |
| 5 | GitHub code search (authenticated, control) | `extension:cosmic` | `gh api search/code` | 58 hits — see §4.1 |
| 6 | GitHub code search (authenticated, control) | `extension:kf` | `gh api search/code` | 7,056 hits — see §4.2 |
| 7 | IANA media-type registry | subtypes containing `cosmic`, `csmx`, `kof`, `csm`; and whether `text/x-*` is registrable | `WebFetch` on `iana.org/assignments/media-types` | No matches; `text/x-*` is **by definition unregistered** — see §5 |
| 8 | General web search | `.csmx` file extension | WebSearch | No format registry entry; a university course-management system named "CMSX" and an unrelated domain `csmx.online` |
| 9 | General web search | `CSMX CSharp Markup Extension .csmx language` | WebSearch | Surfaced unrelated noise (a Chinese Android game-modifier app also called "csmx"); **did not** surface the repository found by code search |
| 10 | General web search | `fileinfo.com .csmx file extension` | WebSearch | No entry; only extension-database index pages |
| 11 | General web search | `VS Code language id "csmx" extension marketplace` | WebSearch | Nothing relevant returned |
| 12 | File-extension databases | extension lookup pages | WebSearch (indirect) | No registered entry found. **Not** a direct database query — see §6 |

**Not covered:** GitLab/Bitbucket/self-hosted code search · sourcegraph · private
repositories · package registries (npm/crates.io/PyPI) · editor-marketplace
listings queried directly (only via web search) · any non-indexed or
access-restricted corpus.

---

## 3. Collisions found

### 3.1 MATERIAL — an active programming language using `.csmx`

| Field | Value |
|---|---|
| Repository | `akeit0/csharp-markup-extension` |
| Description | "C# Markup Extension PoC: C# with JSX-like Expression Syntax" |
| Language | C# |
| Licence | MIT |
| Stars | 16 |
| Created | 2026-06-11 |
| Last push | 2026-06-15 |
| Archived | No |
| Files | 9 `.csmx` files, including a **language-server test suite** (`Csmx.LanguageServer.Tests`) with fixtures such as `Components/CounterView.csmx` |

**Why this is material and not noise:**

1. `.csmx` is used here as a **programming-language source extension**, not as a
   configuration or data suffix. That is exactly the same *category* of use as
   Cosmic-Lang intends.
2. The project ships a **language server**, which means it plausibly declares a
   language identifier and editor associations. Language identifiers and
   extension associations are the two namespaces where a collision actually
   hurts (an editor or LSP client may resolve `.csmx` to the wrong language).
3. It is **publicly indexed by code search**, so a user or an AI assistant
   searching for "what is `.csmx`" will find it.

**Why it is not fatal:**

1. It is explicitly a **PoC**, last pushed 2026-06-15 — four months stale at the
   time of this assessment.
2. 16 stars and a single author: minimal ecosystem footprint.
3. MIT-licensed and unrelated in domain (C#/JSX-style markup versus a
   statically-typed general-purpose language).
4. There is **no file-format-registry entry**, so neither project has a formal
   claim.

**Residual risk: MODERATE.** It is a tooling-level collision, not a format-level
one. It is materially higher than round 1 assessed.

### 3.2 SYNTHETIC — not a collision

`GregTrevellick/FilesForEveryExtension` contains a file literally named
`GitIgnore.CSMX`. That repository exists to create one placeholder file per
extension letter; it is a *catalogue of extensions*, not a use of one. **Not a
collision.**

### 3.3 NAME-ONLY — not a format collision

A Chinese-language Android game-modifier application is also called "csmx".
It does not use the extension; the collision is on the *name*, which matters
only for search discoverability (and it matters: it is what round 1's web
searches surfaced). **Not a format collision**, but it degrades the
searchability of the name.

---

## 4. Control measurements (why the numbers mean something)

A hit count is only interpretable against a baseline.

### 4.1 `extension:cosmic` — 58 hits

Every sampled hit is a **derived or variant file**, not a language source:
`Dockerfile.cosmic`, `Xresources.cosmic`, `pdb.cosmic`, `blacklist.cosmic`,
`compilers.cosmic`. So `.cosmic` is used as a *suffix convention* for
configuration and generated data, and **no project claims it as a language
extension**.

**Implication:** `.cosmic` has **more raw occupation** (58) than `.csmx` (10) but
**no language claim**. If the criterion is "no competing language", `.cosmic`
is arguably *safer* than `.csmx`. If the criterion is "fewest existing files",
`.csmx` is safer. This is a genuine trade-off for the owner, not an obvious win
either way.

### 4.2 `extension:kf` — 7,056 hits

The legacy `.kf` extension is heavily occupied across many unrelated formats.
This is **why** the new toolchain needs its own extension: `.kf` cannot be a
primary Cosmic-Lang extension without ambiguity. It also confirms that the
legacy formats are compatibility surfaces, not extension candidates.

---

## 5. MIME conventions

* **No IANA-registered media type** contains `cosmic`, `csmx`, `kof` or `csm`.
* `text/x-cosmic-lang` — proposed by the design report (page 12) — is a
  **`x-` prefixed type**. The `x-` convention marks a type as **experimental and
  unregistered**; IANA does not register `text/x-*` names at all. RFC 6838
  registration is only possible in the **Standards**, **Vendor** (`vnd.`) or
  **Personal** (`prs.`) trees.
* **Consequence:** `text/x-cosmic-lang` can be used immediately and requires no
  registration, but it is a **de facto local convention**, not an interoperable
  identity. If a registered type is ever wanted, the realistic route is
  `application/vnd.cosmic-lang` (vendor tree) or a `prs.` registration, both of
  which require a registration procedure.
* **Caveat on coverage:** the IANA registry page is large and was truncated
  during retrieval; the `text` tree and the later alphabetical range of the
  `application` tree were not fully enumerated. The negative result is therefore
  **strong for `csmx`/`cosmic`** (they would sort early) and **weaker for `kof`**.
  This limitation is recorded rather than glossed over.

---

## 6. Methodological limitations of this round

1. **No direct file-extension-database query.** The extension databases
   (fileinfo.com and similar) were reached only through web search, not queried
   directly. A negative result from a search engine is weaker than a negative
   result from the database itself.
2. **One code host.** Only GitHub was searched. GitLab, Bitbucket, Sourcegraph
   and self-hosted forges are uncovered.
3. **Index coverage.** GitHub code search covers indexed public repositories
   only; private repositories, forks outside the index, and non-code corpora are
   uncovered.
4. **Point-in-time.** The search is a snapshot. A collision can appear at any
   time — including *after* this assessment.
5. **No trademark search.** No trademark registry was queried for "CSMX",
   "Cosmic-Lang" or "Kof". That is a **legal question** and is listed in
   `docs/audits/phase-0-licensing-matrix.md` §5 (Q3, Q4).
6. **No package-registry search.** npm/crates.io/PyPI were not queried for a
   package that claims the extension or the language identifier.

**None of these limitations can be closed by engineering alone.** Limitations 5
and 6 in particular require either owner action or a scope decision.

---

## 7. Conclusion and recommendation

**Worldwide uniqueness of `.csmx` is NOT established.** Round 1's conclusion
("appears usable, residual risk low for the registry view, moderate for tooling")
is **revised**: the registry view remains clean, but the **tooling view is now
confirmed to have a real, live competitor** (§3.1), and the name is degraded by
an unrelated application of the same name (§3.3).

**Recommendation to the owner (PROPOSED):**

1. **Keep `.csmx` proposed, do not accept it yet.** The decision requires the
   owner, and the evidence does not support a confident acceptance.
2. **Authorise the two missing searches** before acceptance: a direct
   file-extension-database query, and a package-registry query for the language
   identifier. Both are cheap.
3. **Evaluate `.cosmic` as the leading alternative.** It has more raw file
   occupation but **no competing language claim**, which is the property that
   actually matters for editor and language-server association. It is also
   self-describing.
4. **Do not choose `.csm`** — the round-1 assessment stands: it is heavily used
   by unrelated formats (Steam backups, Autodesk, CASL, precompiled headers).
5. **Regardless of the extension, do not rely on the extension for identity.**
   The verified grammar makes the parser **extension-independent**
   (`docs/spec/decisions/S14-trivia.md`, `compatibility-policy.md` §3 rule 1), so
   the extension is a *discovery convenience*, not a semantic marker. This
   materially reduces the cost of a wrong choice: a later change is a rename,
   not a migration.
6. **Treat the MIME type as provisional and unregistered.** Do not claim
   interoperability for `text/x-cosmic-lang`.

**No formal acceptance is recorded by this document.** Per the directive, the
extension and MIME decisions remain **pending formal acceptance**.

---

## 8. Directive 002-R1 candidate matrix (2026-10-16)

This supersedes the narrow two-candidate ranking in §7. Authenticated GitHub
code search was repeated with `GET /search/code?q=extension:EXT&per_page=6`.
Counts below are indexed public-file matches at this instant, **not a census**.
Repository samples, not raw hit counts, establish competing source use. For a
permanent decision, repeat queries and inspect full editor registrations.

| Rank | Candidate | GitHub code-search evidence | Competing language/source use | Editor/LSP collision | Extension registry evidence | Package registry evidence | Residual uncertainty | Migration cost |
|---|---|---|---|---|---|---|---|---|
| **1** | `.cosmic` | **58** indexed; sampled `Dockerfile.cosmic`, `Xresources.cosmic`, `pdb.cosmic`, `compilers.cosmic` | No competing **language source** found **in sampled hits**; suffix is used for config/data | Existing suffix file associations may cause incidental highlighting; direct VS Code/Open VSX enumeration NOT VERIFIED | Indexed FileInfo lookup did not establish an exact `.cosmic` entry; NOT VERIFIED elsewhere | npm/crates.io/PyPI indexed name searches inconclusive; no authoritative extension mapping verified | Unindexed language tooling and repositories | **Medium** if changing from proposed `.csmx`: docs, templates, OS associations, editor `fileExtensions`, LSP selectors, CLI examples and historical sample renames |
| **2** | `.cmic` | **0** indexed matches | None located; **absence cannot prove uniqueness** | Editor extension and LSP registration NOT VERIFIED | No directly confirmed format registry entry; NOT VERIFIED | Package mappings NOT VERIFIED | Search blindness/very short unfamiliar suffix | **Medium**: same surfaces as above, plus explanation/discoverability |
| **3** | `.ccos` | **0** indexed matches | None located in indexed sample; NOT VERIFIED globally | Editor/LSP registrations NOT VERIFIED | No directly confirmed extension registry entry | Package mappings NOT VERIFIED | Potential confusion with `.cos`, new suffix has no recognizable established meaning | **Medium**: docs, editor/OS mapping and source rename |
| **4** | `.csmx` | **10** indexed matches | **Confirmed** `akeit0/csharp-markup-extension`: C# JSX-like source and `.csmx` test fixtures | **Confirmed competing C# language server** in same file association namespace; actual client priority not tested | No entry confirmed by indirect FileInfo search; no exclusive claim | Package names/editor identifier ownership NOT VERIFIED | Existing PoC may gain uptake; editor activation details untested | **Low now** (existing proposal/documentation); **high later** after release, when files, highlighting and LSP setup migrate |
| **5** | `.cos` | **~7,600** indexed matches | **Confirmed** Creatures CAOS source scripts; indexed examples also include `uwdb/Cosette` query files | Material competing scripting source and existing application associations | **Direct FileInfo listing**: Capture One Settings File + Creatures CAOS File (`https://fileinfo.com/extension/cos`) | Package/editor mappings NOT VERIFIED | Count includes unrelated formats and archive noise; no exclusivity | **High**: editor, OS, user data and build-system coexistence conflicts |
| **6** | `.csm` | **~6,736** indexed matches | Used by CASL/other non-Cosmic formats; language-source subset not fully sampled | Collision risk across multiple established formats | FileInfo and other indexes not exhaustively validated; earlier round identified Steam/Autodesk usage | Package mappings NOT VERIFIED | Multiple mature associations; direct registry snapshot outstanding | **High**: confusing file associations and editor routing |

**Search scope:** GitHub authenticated code search + indexed public web search +
one directly viewed FileInfo page for `.cos`
(`https://fileinfo.com/extension/cos`) + the IANA media-type registry
(`https://www.iana.org/assignments/media-types`, dated 2026-10-09).
Marketplaces: indexed queries of VS Code listings for `csmx` and `cosmic`
did not establish either candidate's file association; the unrelated `Cosmic
Code` VS Code theme is name overlap only, **not a language extension**.
Queries for `cosmic-lang` packages on npm, crates.io and PyPI through indexed
web search returned no definitive, directly verified package-to-suffix mapping.
Package registry **direct API listings were not completed**; consequently all
package-registration cells are **NOT VERIFIED**. No extension can be pronounced
available based on these results.

**Priority criterion:** competing language-source and editor/LSP ownership is
more important than raw file counts. `.cosmic` is therefore the leading
**proposal**, with `.cmic` a concise fallback and `.ccos` a conservative
second fallback. `.csmx` should not be accepted without an explicit owner
choice acknowledging its confirmed C# markup conflict. Neither the extension
nor MIME type is approved. `text/x-cosmic-lang` remains only a provisional
unregistered convention. A later change is a **migration** (file renames,
tooling configuration, language-server activation, compatibility docs), even
when the parser grammar is extension-independent.

Remaining decision evidence: direct editor Marketplace/Open VSX extension
manifest searches; direct package registry API searches; OS file association
tests; another code forge; legal trademark review. These are bounded follow-ups,
not conditions for treating absence of evidence as proof of uniqueness.
