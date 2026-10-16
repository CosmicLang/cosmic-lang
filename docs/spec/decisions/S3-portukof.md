# S3 — PortuKof (`.ptkf`) Compatibility

| | |
|---|---|
| **Status** | **OPEN** — owner-dependent PortuKof scope decision |
| **Owner** | Project owner (scope); engineering (analysis) |
| **Phase** | Phase 1 decision; specification and implementation only if authorised |
| **Related** | `docs/spec/compatibility-policy.md` §2.3, `docs/spec/language-spec.md` §2.3, ADR-0002 |

## Question

1. **Q1** — Is PortuKof in scope for the new toolchain at all?
2. **Q2** — If yes, what exactly differs from the Kof surface: keywords only, or
   also standard-library member names, method names and diagnostics?
3. **Q3** — Can a single file mix Portuguese and English surfaces?
4. **Q4** — Is PortuKof a *translation* (surface → canonical) or a *parallel
   language* with its own semantics?

## Evidence

* **E1** — The extension `.ptkf` is accepted by the legacy CLI's source
  discovery and validation filters
  (`kof-cli/src/main/java/dev/kof/cli/KofCliSupport.java`).
* **E1** — There are exactly **4** `.ptkf` files in the tree, all under
  `examples/portukof/`: a hello-world, a collections example, and an
  import-pair (`main.ptkf` plus `imports/util/Mat.ptkf`).
* **E1** — The surface is **not keyword-only**. The profile machinery includes
  keyword mapping, lexical canonicalisation
  (`parser/Lexer.java` applies `profile.keywords()` and `profile.lexicalCanon()`
  to every identifier), and additional components for stdlib members, method
  aliases, method splicing, diagnostics and a parity checker:
  `lang/LanguageProfile.java`, `lang/PortuKofVocabulary.java`,
  `lang/SurfaceNames.java`, `lang/PortuKofParity.java`,
  `lang/PortuKofStdlibMembers.java`, `lang/PortuKofDiagnostics.java`,
  `lang/PortuKofMethodAliases.java`, `PortuKofMethodSplicer.java`.
* **E1** — The hello-world example uses a translated entry-point name and a
  translated print member, which confirms that **stdlib member names are part of
  the surface**, not just keywords.
* **E1** — Because canonicalisation happens **in the lexer**, a translated
  surface token becomes the canonical token before the parser sees it. So at the
  parser level there is only one language.
* **E1** — `kof-cli/src/main/java/dev/kof/cli/Fmt.java` documents that the
  **file name is the authority** for the profile (`.ptkf` ⇒ PortuKof), and
  `LspHover.java` documents that the internal symbol stays **canonical** while
  the label is localised.
* **E1 (negative)** — There is **no** executed golden case for `.ptkf`. The
  golden corpus is `.kf` only. The 222/222 baseline therefore says **nothing**
  about PortuKof behaviour.
* **E1 (negative)** — There is no evidence of a mixed-surface program.

## Options

| # | Option | Assessment |
|---|---|---|
| C1 | **Out of scope.** Support `.csmx` and `.kf` only; `.ptkf` produces a clear "unsupported format" diagnostic | Smallest surface, honest, and consistent with the design report's "prioritise the core language" instruction. Breaks the 4 example programs |
| C2 | **In scope as a lexical translation layer** — a profile table mapping surface tokens/members to canonical ones, with no separate grammar | Matches the evidenced architecture exactly (canonicalisation in the lexer). Cost is the profile tables, which must be derived from the vocabulary components, not guessed |
| C3 | **In scope as a separately specified language** with its own grammar | Contradicts the evidence: the parser never sees the surface form |
| C4 | **Defer the decision** to a later phase | Acceptable only if `.ptkf` files are *rejected loudly* meanwhile, never silently mis-parsed |

## Recommendation (PROPOSED, not in force)

**C4 as the interim behaviour, then C2 if the owner wants PortuKof.**

* **Interim (normative if accepted):** the new toolchain must **reject `.ptkf`
  loudly** — a clear "PortuKof is not yet supported" diagnostic and a non-zero
  exit — rather than attempting to parse it as Kof. Silently mis-parsing a
  Portuguese file as Kof would produce a cascade of confusing errors, which is
  the failure mode the project's "never silently degrade" rule exists to
  prevent.
* **If in scope:** implement as **C2** — a lexical translation layer, with the
  profile tables derived from the legacy vocabulary components at the frozen SHA
  and validated by a new fixture set. **No new grammar.**
* **Q3** — Assume **not supported** (no mixed surfaces) until evidence says
  otherwise. If mixed surfaces exist, canonicalisation in the lexer means they
  would already work, but this is unverified and must not be claimed.
* **Q4** — It is a **translation**, not a parallel language. The evidence
  (lexer-level canonicalisation) is unambiguous.

## Compatibility consequence

* C2 would be **C1 (syntactic acceptance)** plus **C2 (behaviour)** for the
  translated surface, scoped to the derived tables.
* **C3 is not promised** — see `compatibility-policy.md` §1.
* PortuKof is **not** part of the `.kf` compatibility promise; it is a separate
  format with a separate decision, as `compatibility-policy.md` §2.3 requires.

## Open dependencies

* **Owner scope decision** (Q1). This record cannot close without it.
* Deriving the profile tables requires reading the vocabulary components at the
  frozen SHA. The tables are **facts about the surface**, not protected
  expression, but the derivation must be recorded so the provenance is auditable
  (`docs/audits/phase-0-licensing-matrix.md` §4).
* No `.ptkf` fixture set exists; one must be authored from observed behaviour,
  not copied.
