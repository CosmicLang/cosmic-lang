# ADR-0002 — Source Extension and MIME Type Selection

| | |
|---|---|
| **Status** | **Proposed** — awaiting owner approval of the MIME type. **The extension is decided:** `.cmic` (owner, 2026-10-16, §4.1). |
| **Date** | 2026-10-16 (round 1: R1) · 2026-10-16 (post-merge Directive 002-R2: owner selection recorded) |
| **Deciders** | Project owner (final authority); engineering (author of the proposal) |
| **Phase** | 1 |
| **Related** | ADR-0001 §2.2/§3, `docs/spec/extension-assessment.md`, `docs/spec/compatibility-policy.md` §2.1 |

---

## 1. Context

The design report (page 12) proposes `.csmx` as the Cosmic-Lang source
extension with the provisional MIME type `text/x-cosmic-lang`, and explicitly
requires that collision risk be investigated before the choice is formalised —
while stating that worldwide uniqueness cannot be demonstrated.

ADR-0001 §2.2 adopted `.csmx` as a **proposal** and §3 recorded a first-round
investigation (two generic web queries) whose stated limitation was that
exhaustive code search had not been performed.

Directive 002 §5 required that investigation to continue across public code
repositories, extension databases, language tooling, editor associations and
MIME conventions. That work is recorded in full in
`docs/spec/extension-assessment.md`. **It changed the conclusion.**

---

## 2. The finding that drives this ADR

Authenticated GitHub code search (`extension:csmx`, 2026-10-16) returns 10 hits,
dominated by **`akeit0/csharp-markup-extension`** — an MIT-licensed C# project
that uses `.csmx` as a **programming-language source extension** and ships a
**language server**.

Round 1 concluded "residual risk is low for the format registry view and
moderate for tooling heuristics near `.csm`". Round 2 **sharpens that into a
named competitor in exactly the namespace that matters**: language identifiers
and editor associations, which is where an extension collision has real
consequences.

Mitigating facts: the project is a stale PoC (last push 2026-06-15, 16 stars,
single author), the domain is unrelated, and **no file-format registry entry
exists for either project**. So this is a tooling collision, not a format claim.

The name is also degraded by an unrelated Android application called "csmx",
which is what round 1's web searches surfaced.

---

## 3. Options

| Option | Extension | Evidence | Assessment |
|---|---|---|---|
| **A1** | `.csmx` | 10 code-search hits; 1 competing language project; no registry entry; name collision with an unrelated app | Keeps the design report's proposal and Phase 0's existing usage. Tooling collision is real but low-impact |
| **A2** | `.cosmic` | 58 code-search hits, **all** derived/variant files (`Dockerfile.cosmic`, `Xresources.cosmic`, …); **no competing language claim** | Self-describing, and the property that matters (no language claim) is stronger. But more raw occupation and a longer name |
| **A3** | `.csm` | Heavily used by unrelated formats (Steam backups, Autodesk 3ds Max, CASL, C++ precompiled headers) | **Rejected** — round 1 and round 2 agree |
| **A4** | `.cx` | Short; not investigated in either round | **Insufficient evidence** — would need its own investigation |
| **A5** | **No new extension**: keep `.kf` as primary | 7,056 code-search hits for `.kf` across many unrelated formats | **Rejected** — this is precisely why a new extension is needed |

---

### Directive 002-R1 update — ranked alternatives

The six-candidate, registry/editor/package-coverage matrix in
`docs/spec/extension-assessment.md` §8 supersedes this initial ranking:
`.cosmic` (1), `.cmic` (2), `.ccos` (3), `.csmx` (4),
`.cos` (5), `.csm` (6). At R1 the owner had approved **none**, registry and
package checks were incomplete, and zero GitHub hits did not establish
uniqueness. **The owner's subsequent selection of `.cmic` — the ranked-second
candidate — is recorded in §4.1.**

## 4. Decision

### 4.1 Owner decision (post-merge Directive 002-R2, 2026-10-16)

**Taken: the primary source extension is `.cmic`.** The project owner selected
it; the selection is recorded here with its evidence and its limits.

* **Code search.** Authenticated GitHub code search for `extension:cmic`
  returns **zero indexed matches** (2026-10-16), the only candidate in the
  round-2 matrix with zero *indexed* occupation at decision time.
* **Filename search.** `filename:.cmic` returns hits that are **substring
  artifacts** (`.cmic2`, `.cmico`, crawler-saved `*.cmic-holdings.co.jp.*`
  files), not files of a competing format — recorded as such, not as "zero
  occupation".
* **Registry view.** A direct extension-database query (FileInfo) returns no
  entry for `.cmic`.
* **Brand overlap.** The "CMiC" construction-ERP brand dominates web results
  for the string — a discoverability consideration, not a format claim.
* **Not claimed: worldwide uniqueness.** The evidence rests on indexed search
  and public sources; unindexed tooling, package registries and editor
  association stores were not exhaustively queried, and a package-registry
  query remains open.

**Consequences:**

* Current normative documents, examples and compatibility tables use `.cmic`
  as the primary extension.
* The parser remains **extension-independent** (§5): this is a surface
  decision, never a grammar change.
* The **MIME type is still not decided** (§6). Editor integration and any
  registry or package publication are separate matters and are **not**
  performed by this decision.
* The historical `.csmx`/`.cosmic` investigations are preserved in
  `docs/spec/extension-assessment.md` and ADR-0001 §3 — they document why
  `.csmx` was not accepted and why `.cosmic` was the leading alternative.

### 4.2 Historical: engineering recommendation (Directive 002-R1, retained)

**Not taken by engineering — the decision required the owner.** Engineering's
recommendation at R1 was:

1. **Do not accept `.csmx` yet.** The evidence does not support a confident
   acceptance, and the directive requires the extension decision to remain
   pending formal acceptance.
2. **Close the two named search gaps first** (`extension-assessment.md` §6):
   direct editor/extension registry inspection and package API checks for the
   language identifier; these are NOT VERIFIED.
3. **Then choose between A1 and A2.** The deciding criterion should be stated
   explicitly, because the two criteria point in opposite directions:
   * "fewest existing files" → `.csmx`
   * "no competing language claim" → `.cosmic`
   Engineering's view is that the second criterion matters more, because the
   cost of a collision is borne by **editor and language-server association**,
   not by file counts.
4. **Do not invest heavily in the search.** See §5.

---

## 5. Consequence that lowers the stakes

The verified grammar makes the parser **extension-independent**
(`docs/spec/compatibility-policy.md` §3 rule 1; `docs/spec/decisions/S14-trivia.md`
Q2): the extension selects a *surface profile*, never the core grammar. The
extension is therefore a **discovery convenience**, not a semantic marker.

An extension change requires **migration**: source-file renames, build tooling,
editor and LSP associations, documentation and user update instructions. This is an
argument for a time-boxed investigation and a prompt decision rather than an
exhaustive one — and it is the strongest reason not to over-engineer this
choice.

The owner's selection of `.cmic` (2026-10-16, §4.1) converts this into a
decision. The documentation part of the migration is performed in this
repository; the tooling and publication parts remain out of scope until
Phase 5.

---

## 6. MIME type

| Item | Finding | Recommendation |
|---|---|---|
| `text/x-cosmic-lang` (design report page 12) | `x-` prefixed types are **by definition unregistered**; IANA registers no `text/x-*` name and no `cosmic`/`csmx`/`kof`/`csm` subtype | Use it as a **local convention only**. Do not claim interoperability |
| A registered alternative | RFC 6838 registration is possible only in the Standards, Vendor (`vnd.`) or Personal (`prs.`) trees | If a registered type is ever needed, `application/vnd.cosmic-lang` is the realistic route. **Not proposed now** — registration has a cost and no current consumer |
| Coverage caveat | The IANA registry page is large and was truncated during retrieval, so the `text` tree and the late `application` range were not fully enumerated | The negative result is strong for `cosmic`/`csmx` (they sort early) and **weaker for `kof`**. Recorded, not glossed over |

**Decision: not taken.** Treat the MIME type as provisional and unregistered.
The `.cmic` extension selection changes none of this: no MIME registration is
performed or promised, and the extension decision is explicitly separate from
the MIME question.

---

## 7. Status of this ADR

* **Extension — decided.** The owner selected `.cmic` (2026-10-16, §4.1).
* **MIME type — proposed only.** `text/x-cosmic-lang` remains provisional and
  unregistered; owner approval of a MIME type is still required if one is ever
  published.
* **Not blocking Phase 2.** Because the parser is extension-independent, the
  lexer and parser can be built before any MIME or publication decision; they
  simply must not hard-code the extension anywhere.
* **Blocking Phase 5 (release).** The remaining open items — the MIME type and
  publication/editor association — are part of a released identity.
