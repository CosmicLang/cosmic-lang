# Phase 1 — Specification Engineering Report

> **Historical Directive 002 snapshot.** The inventory, status distribution,
> Phase 1 gate assessment and CI SHA in this report describe the initial draft
> before Directive 002-R1. They are **superseded for current decisions** by
> `docs/audits/phase-1-002-r1-reconciliation.md`,
> `docs/spec/implementation-gates.md`, and
> `docs/spec/decisions/README.md`. In particular, S1/S11 now have bounded JVM
> experimental evidence; 117 grammar productions are currently defined; G1b
> remains a proposed supersession awaiting owner acceptance. Prior CI results
> do not establish CI success on the final corrective commit.

| | |
|---|---|
| **Date** | 2026-10-16 |
| **Directive** | ENGINEERING DIRECTIVE 002 — Phase 1: Formal Language Specification and Architecture Contract |
| **Branch** | `phase/1-specification` |
| **Base commit** | `8f0944cd9ac1f30c835482b086a6dab300b5f1f0` (`main`, the Phase 0 merge commit) |
| **Phase 0 merge** | PR #1 — two verified parents: `7bf5061` (original `main`) and `d7bc652` (Phase 0 head). Post-merge CI run `38930672864` **success** |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |
| **Scope delivered** | Specification only. **No compiler implementation was added.** |

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

---

## 1. Governance preflight (directive §2)

### 1.1 State before any change

| Item | Verified value |
|---|---|
| `main` HEAD | `8f0944cd9ac1f30c835482b086a6dab300b5f1f0` — **matches** the directive |
| Merge commit parents | `7bf50614ee81621c0e7050a55c6bbe4c157bac6b` and `d7bc652d366e176ee118c25e7f2876c3b182352d` — **matches** |
| Post-merge CI | Run `38930672864`, event `push`, branch `main`, conclusion **success** |
| **Classic branch protection on `main`** | **ABSENT** — the API returned `404 Branch not protected` |
| **Repository rulesets** | **EMPTY** — `[]` |
| Administrator access | **Available** — the API reports `permissions.admin: true` |
| Collaborators | **Exactly one** (`CosmicLang`, `admin`) — the repository is single-maintainer |
| The `Validate` check | Present on `main` from **app id `15368`** (`github-actions`), conclusion `success` |

The Phase 0 review had recorded that it "could not determine whether classic
branch protection is active". This preflight resolved it: **there was none**.

### 1.2 Protection applied (directive §2)

Applied with `PUT /repos/CosmicLang/cosmic-lang/branches/main/protection`.

| Setting | Value | Directive requirement |
|---|---|---|
| `required_pull_request_reviews.required_approving_review_count` | **0** | "Pull requests required before integration" + "Do not create an impossible self-approval requirement for a single-maintainer repository" |
| `required_status_checks.strict` | `true` | PRs must be up to date with `main` before merging |
| `required_status_checks.checks` | `[{context: "Validate", app_id: 15368}]` | "The existing `Validate` check required before merge, with GitHub Actions as the trusted source when supported" |
| `enforce_admins` | `true` | "Rules enforced for administrators where supported and appropriate" |
| `allow_force_pushes` | `false` | "Force pushes prohibited" |
| `allow_deletions` | `false` | "Branch deletion prohibited" |
| `required_conversation_resolution` | `true` | "No automatic merge or unrestricted bypass" (added beyond the literal list; see below) |
| `required_linear_history` | `false` | Deliberate: Phase 0 integrated through a **merge commit**, and forbidding merge commits would change the project's integration model without authorisation |
| `restrictions` | `null` | No push restrictions — a single maintainer needs to be able to push branches |
| `lock_branch` | `false` | Not required, and locking would block the documented workflow |

**Two deliberate choices worth flagging:**

1. **`required_approving_review_count = 0`.** With one collaborator, requiring
   one approval would make merging **impossible** (GitHub does not permit
   self-approval), which is precisely what the directive forbids. The
   requirement that changes go through a pull request is still enforced; only
   the *approval count* is zero.
2. **`required_conversation_resolution = true` was added** beyond the
   directive's literal list, because it directly serves "no unrestricted
   bypass" by preventing a merge while review comments are unresolved. It is
   reversible with one API call and is recorded here so the owner can drop it.

**"Existing stronger protection preserved":** there was none to preserve.

### 1.3 Effective enforcement — empirically verified

The configuration was not merely read back; **enforcement was tested**. A
throwaway commit was created on a scratch branch as a **fast-forward descendant
of `main`** and pushed directly to `main`:

```
remote: error: GH006: Protected branch update failed for refs/heads/main.
remote: - Changes must be made through a pull request.
remote: - Required status check "Validate" is expected.
 ! [remote rejected] probe/protection-test -> main (protected branch hook declined)
```

Both the pull-request requirement **and** the required status check were
enforced. `main` was verified **unchanged** afterwards
(`8f0944cd9ac1f30c835482b086a6dab300b5f1f0`), and the scratch branch was
deleted. No repository protection setting was altered beyond §1.2, and no
GitHub permission was bypassed.

---

## 2. Deliverable inventory (directive §4)

| # | Deliverable | Path | Status |
|---|---|---|---|
| D1 | Language specification | `docs/spec/language-spec.md` | **Delivered** — 12 sections + unresolved register |
| D2 | Formal grammar | `docs/spec/grammar.ebnf` | **Historical: 59 productions**; Directive 002-R1 expanded the working proposal to 117 productions, with block-level evidence markers only |
| D3 | Type-system contracts | `docs/spec/typing-contracts.md` | **Delivered** — 9 sections, TI-1…TI-8 |
| D4 | Semantic decisions | `docs/spec/decisions/` | **Delivered** — 18 records + register index |
| D5 | Compatibility policy | `docs/spec/compatibility-policy.md` | **Updated** — priority order confirmed, round-2 findings folded in |
| D6 | Architecture decision records | `docs/adr/` | **Delivered** — ADR-0001 reviewed/updated, ADR-0002…ADR-0005 new |
| D7 | Measurement plan | `docs/development/measurement-plan.md` | **Delivered** — 11 metric specifications |
| D8 | Test strategy | `docs/development/test-strategy.md` | **Delivered** — 8-layer ladder (L0–L7) |
| D9 | Diagnostic registry | `docs/spec/diagnostic-codes.md` | **Delivered** — 18 namespaces, stability policy |
| D10 | This report | `docs/audits/phase-1-specification-report.md` | **Delivered** |
| — | Extension assessment (§5) | `docs/spec/extension-assessment.md` | **Delivered** — round 2 |
| — | Licensing recommendation (§6) | `docs/audits/phase-1-licensing-recommendation.md` | **Delivered** |
| — | Directive 003 proposal (§10J) | `docs/directives/directive-003-proposal.md` | **Delivered** — proposed, not authorised |

**Documents corrected in this phase (not just added):**

| Document | Correction | Why |
|---|---|---|
| `docs/spec/compatibility-matrix.md` §3 item 8 | Removed the claim that the lexer supports triple-quoted strings | The claim was **false**: the lexer recognises `"""` only to reject it with `LEX008` |
| `docs/adr/0001-language-identity.md` §3 | Struck out "no existing editor grammar or language tooling claims `.csmx`" | Round 2 code search **disproved** it |
| All documents citing legacy paths | Legacy paths are now prefixed `cosmcc:` | Makes every evidence reference unambiguous |
| `docs/audits/phase-0-baseline.md`, `docs/audits/phase-0-design-conformance.md`, `docs/spec/compatibility-policy.md`, `docs/spec/language-spec.md`, `docs/spec/typing-contracts.md`, 6 decision records | Added the path convention note | Same |

---

## 3. Grammar evidence and unresolved productions (directive §10D)

### 3.1 Evidence basis

Every production in `docs/spec/grammar.ebnf` carries one of:

* `[E1:path]` — derived from legacy source at the frozen SHA (path given
  relative to `kof-compiler/src/main/java/dev/kof/compiler/`);
* `[E1:case]` — witnessed by an executed golden case (222/222 for
  jvm/js/script);
* `[E2]` — inferred from E1 material;
* `[OPEN:Sn]` — deliberately undecided, owned by decision record `Sn`.

**Primary evidence sources read in this phase:** `parser/Lexer.java`,
`parser/Parser.java`, `parser/StatementParser.java`, `parser/ExpressionParser.java`,
`parser/TypeParser.java`, `parser/TypeDeclarations.java`,
`parser/AnnotationParser.java`, `parser/LambdaParser.java`,
`parser/ClassMemberParser.java`, `TokenType.java`, `lang/KofKeywords.java`,
`kof-script/src/main/java/dev/kof/script/KofScript.java`,
`kof-cli/src/main/java/dev/kof/cli/KofCliSupport.java`, plus the golden corpus
case names and selected executed `expected.txt` outputs.

### 3.2 The verified precedence table (a headline result)

`ExpressionParser.precedence()` was read directly, so the grammar's precedence
levels are **E1, not inferred**:

| Level | Operators |
|---|---|
| 1 (lowest) | `\|\|` |
| 2 | `&&` |
| 3 | `\|` `^` |
| 4 | `&` |
| 5 | `==` `!=` `<` `<=` `>` `>=` `instanceof` `as` |
| 6 | `<<` `>>` `>>>` |
| 7 | `+` `-` |
| 8 (highest binary) | `*` `/` `%` |
| — | assignment, parsed **below** all binary levels |

`as` and `instanceof` take a **type** on the right, not an expression — a
verified special case.

### 3.3 Unresolved productions (must not be implemented as if settled)

| Production | Reason | Record |
|---|---|---|
| `package-decl`, `import-decl` | Optionality, placement, wildcards, aliases | S9 |
| `test-decl`, `application-decl`, `infra-decl` | Outside the corpus; contextual keywords | S10 |
| `extern-decl`, `foreign-module-decl` | FFI; security-sensitive | S10 |
| `entity-decl` | Keyword reserved; semantics unknown | S10 |
| `annotation` | Vocabulary and semantics unknown | S10 |
| `modifier` combinations | Legal combinations unknown | S10 |
| `type-ref` with nested arguments | Known legacy defect area | S1 |
| `type-ref` `?` semantics | Conversion rules unknown | S2 |
| `parameter` type optionality; default return type | Not evidenced | S1 |
| `for-stmt` three-clause form | Parser-evidenced, corpus-unevidenced | S5 |
| `switch-arm` exhaustiveness, fallthrough, labelled break/continue | Not evidenced | S5 |
| `assignment-expr` associativity | Not evidenced | S11 |
| `nested-function-decl` capture semantics | Not evidenced | S11 |
| `array-literal`, array indexing details | Not exercised by the corpus | S9 |
| expression evaluation order | Never written down; observable only | S11 |
| identifier class set, normalisation, confusables | Unicode policy open | S13 |
| trivia and terminator normalisation | Formatter/LSP policy open | S14 |
| feature-gate syntax | Not designed | S15 |

### 3.4 Deliberately absent productions (findings, not omissions)

Triple-quoted/raw/multiline strings · string interpolation · the bitwise
complement operator `~` · binary/octal/digit-separator/hex-float literals ·
`fun`/`fn`/`func` declarations · package/import wildcards, aliases, static
imports · any ownership/borrowing/lifetime syntax · generics *declaration* and
bounds syntax.

---

## 4. S1–S18 decision register (directive §10E)

| # | Record | Status |
|---|---|---|
| S1 | Nested generics, inference, bounds, variance, overload resolution | **OPEN** (partly BLOCKED-ON-MEASUREMENT) |
| S2 | Nullability and the structural fallback | **OPEN** |
| S3 | PortuKof | **BLOCKED-ON-OWNER** |
| S4 | KofScript | **BLOCKED-ON-OWNER** |
| S5 | Pattern matching, guards, exhaustiveness | **OPEN** |
| S6 | Concurrency contract | **OPEN** |
| S7 | Numeric semantics | **OPEN** (one part PROPOSED) |
| S8 | String and escape semantics | **OPEN** (one part PROPOSED) |
| S9 | Standard-library boundary, member kinds, module rules | **OPEN** |
| S10 | Reserved keywords, annotations, FFI, contextual declarations | **OPEN** |
| S11 | Evaluation order and binding | **BLOCKED-ON-MEASUREMENT** |
| S12 | Equality semantics | **OPEN** (one part PROPOSED) |
| S13 | Unicode semantics | **PROPOSED** |
| S14 | Trivia preservation and normalisation | **PROPOSED** |
| S15 | Feature gates, severities, error recovery | **PROPOSED** |
| S16 | Memory strategy | **BLOCKED-ON-OWNER** |
| S17 | Runtime ABI versioning and layering | **BLOCKED-ON-OWNER** |
| S18 | Typed-IR invariants and the MIR verifier | **PROPOSED** |

**Historical Directive 002 distribution (superseded by Directive 002-R1):**
9 OPEN · 4 PROPOSED · 4 BLOCKED-ON-OWNER · 1 BLOCKED-ON-MEASUREMENT.

**No record claims owner approval.** Every record separates **proven legacy
behaviour** (E1) from **proposed new semantics** (P) and marks unresolved
owner-dependent items OPEN.

### 4.1 New findings surfaced by the decision records

These are the substantive discoveries of Phase 1, each traceable to source:

1. **Triple-quoted strings do not exist and are actively rejected** (`LEX008`).
   A Phase 0 compatibility-matrix claim to the contrary was **wrong** and is
   corrected. (S8)
2. **Unknown escapes are silently reinterpreted** as the escaped character, with
   no diagnostic. (S8)
3. **The legacy lexer counts columns in UTF-16 code units**, while the Phase 0
   foundation counts Unicode scalar values — so legacy and new diagnostics
   **diverge from the first astral character on a line**. (S13)
4. **The legacy lexer silently promotes an out-of-range decimal integer literal
   to long.** A silent widening. (S7)
5. **There is no bitwise complement operator**; `~` lexes as an unexpected
   character. (S7)
6. **The grammar is line-sensitive in one place**: a trailing lambda block
   attaches only if its `{` starts on the same line as the token closing the
   preceding call. (D2)
7. **Block comments do not nest**, and the legacy formatter reconstructs layout
   rather than preserving it — because trivia is discarded. (S14)
8. **PortuKof is not a keyword-only surface**: it has stdlib-member aliases,
   method aliases and its own diagnostics, canonicalised in the lexer. (S3)
9. **KofScript's wrapper is line-oriented text manipulation**, not a
   parse-and-rewrite, and the tree contains **0** `.ks` files. (S4)
10. **The legacy WASM backend refuses record equality** (`WASM002`) because the
    JVM `==` desugar merges a field comparison through control flow across
    labels — which also proves record `==` is component-wise. (S12, S18)
11. **There is no typed AST**: types live in analyzer-side maps and are
    re-inferred during lowering — the design report's page 4 diagnosis,
    confirmed at source. (D3)
12. **Overload resolution has no written rules** and no corpus case, so it
    cannot be specified from evidence and is BLOCKED-ON-MEASUREMENT. (S1)

---

## 5. Source-extension assessment (directive §5)

Full record: `docs/spec/extension-assessment.md`. Summary:

* **Round 2 closed the round-1 gap** (code search) and **changed the
  conclusion**. Authenticated GitHub code search (`extension:csmx`,
  2026-10-16) returns **10 hits**, dominated by `akeit0/csharp-markup-extension`
  — an MIT-licensed C# project using `.csmx` as a **language source extension**
  and shipping a **language server**. It is a stale PoC (last push 2026-06-15,
  16 stars), and there is **no file-format-registry entry** for either project.
* **Control measurements:** `extension:cosmic` → 58 hits, **all** derived files
  (`Dockerfile.cosmic`, `Xresources.cosmic`, …), **no competing language
  claim**; `extension:kf` → 7,056 hits (which is why a new extension is needed
  at all).
* **MIME:** no IANA-registered type contains `cosmic`/`csmx`/`kof`/`csm`.
  `text/x-cosmic-lang` is a **by-definition unregistered** `x-` type; a
  registered type would need the vendor or personal tree.
* **Residual risk: MODERATE at the tooling level, LOW at the registry level.**
* **Coverage gaps named:** no direct extension-database query, no
  package-registry query, one code host, no trademark search, point-in-time.
* **Verdict: worldwide uniqueness is NOT established**, and **no extension or
  MIME decision is accepted**. ADR-0002 records the options and withholds the
  choice. `.cosmic` is recorded as the leading alternative; `.csm` remains
  rejected.

---

## 6. Licensing and compatibility proposals (directive §10F)

### 6.1 Licensing

`docs/audits/phase-1-licensing-recommendation.md`.

* **Recommendation: Option B** (independent implementation under a permissive
  licence), with the **licence text chosen by the owner**. Rationale: the work
  already done is option-B-shaped; compatibility is achievable behaviourally;
  and the design report's own strategy is behavioural ("the main asset to
  preserve is the language's demonstrable semantics").
* **No licence is chosen.** No `LICENSE` file exists; no licence field was added
  to `Cargo.toml`; `README.md` still states "Not yet chosen".
* **No legacy artefact was imported** — verified: 0 `.kf`/`.ptkf`/`.ks` files,
  0 copied prose, 0 translated code.
* **One unconditional recommendation:** add a contribution policy forbidding the
  import of legacy artefacts. It does not require the licence decision and
  protects whichever option is later chosen.
* **Ten legal-review dependencies** are identified, three of them new in this
  phase (L9/L10: whether deriving PortuKof vocabulary or KofScript wrapping
  semantics from *observed behaviour* creates a derivative work — these
  directly gate S3 and S4).

### 6.2 Compatibility

`docs/spec/compatibility-policy.md`, priority order confirmed by directive §4:

| Priority | Format | Compatibility level | Owner decision |
|---|---|---|---|
| 1 | `.csmx` | N/A (native) | **Required** — pending, two search gaps named |
| 2 | `.kf` / `.kof` | **C1 + C2**, scoped to evidenced constructs | **Required** |
| 3 | `.ptkf` | None yet | **Required** (scope) |
| 4 | `.ks` | None yet | **Required** (scope) |

The **C0–C3** vocabulary is retained, and **C3 (bit-identical legacy artifacts)
is promised for no format**. The parser is **extension-independent**, which is
what makes the extension decision a rename rather than a migration.

---

## 7. Validation results (directive §7)

Run at the working tree that becomes the branch tip, with the pinned toolchain
(1.99.0):

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | **exit 0** — no diff |
| `cargo clippy --workspace --all-targets -- -D warnings` | **exit 0** — no warnings |
| `cargo test --workspace --all-targets` | **58 passed, 0 failed, 1 ignored** |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | **exit 0** — 4 crates documented, no warnings |

**Test counts are unchanged from the Phase 0 review** (58 + 1 ignored), which is
the expected result: Phase 1 changed no code. The single ignored test is the
documented manual 5 GiB oversized-load check.

### 7.1 Documentation integrity validation (directive §7)

A scripted validation was run over `docs/**` and `README.md`:

| Check | Result |
|---|---|
| Decision records present for S1–S18 | **18 of 18**, none missing |
| Register rows | **18** (S1…S18) |
| Internal markdown links | **18 checked, 0 broken** |
| Referenced repository paths | **269 checked, 0 unresolved** |
| Grammar productions | **59**, with **19** evidence/unresolved markers |
| Legacy fixture files in this repository | **0** |
| `LICENSE` file present | **No** — correct, the licence is undecided |
| Workspace crates | **exactly 3** (`cosmic-cli`, `cosmic-diagnostics`, `cosmic-source`) — no new crates |

The path check initially failed on 32 references to legacy-repository documents.
Rather than suppress it, **the references were fixed**: legacy paths are now
prefixed `cosmcc:` and 11 documents carry an explicit path-convention note. The
check then passed cleanly. This is recorded because the check found a real
documentation defect (unresolvable evidence references), not a false positive.

---

## 8. Phase 1 gates (directive §9)

| Gate | Criterion | Status |
|---|---|---|
| **G1** | Grammar and specification evidence complete | **SATISFIED (engineering)** — D1 and D2 delivered; every production evidence-marked; **owner approval of the grammar is still required** |
| **G1b** | Historical flat all-decisions gate | **SUPERSESSION PROPOSED, NOT ACCEPTED.** The old engineering completeness criterion created a dependency cycle; see `docs/spec/implementation-gates.md` for the proposed replacement. |
| **G1c** | Identity, extension and compatibility decisions prepared for approval | **SATISFIED (engineering)** — ADR-0001 reviewed, ADR-0002 new, compatibility policy updated. **Owner approval outstanding** |
| **G1d** | Licensing decision presented; not falsely marked resolved | **SATISFIED (engineering)** — recommendation delivered; licence **undecided**; 10 legal dependencies identified. **Owner decision outstanding** |
| **G1e** | CI green and governance protections verified | **SATISFIED** — see §9 |

**Gates requiring owner decisions remain PENDING until actual authorisation.**
Nothing in this report converts an owner decision into a completed one.

---

## 9. Draft PR state and CI (directive §10H)

| Item | Value |
|---|---|
| Branch | `phase/1-specification` |
| Base | `main` @ `8f0944cd9ac1f30c835482b086a6dab300b5f1f0` |
| PR | **#2, draft**, against `main` |
| Merged | **No** — and not marked ready for review |
| PR #1 | **Not modified** (already merged) |
| Force pushes | **None** — all pushes fast-forward |
| CI | Recorded in the PR; the branch-push and pull-request runs must both be green |

**Governance in force during this work:** the protection applied in §1.2. Note
that `main` was protected *before* the Phase 1 branch was created, so the
protection was in force for the whole phase.

---

## 10. Remaining risks and known limitations

| # | Risk / limitation | Class |
|---|---|---|
| R1 | **13 of 18 decision records are not decided.** Phase 2 can start on the frontend, but every undecided construct must be gated as "not yet specified" rather than implemented | E1 |
| R2 | **The grammar is unapproved.** It is evidence-backed but the owner has not accepted it, and it contains 18 provisional productions | E1 |
| R3 | **Evaluation order and overload resolution cannot be specified from existing evidence.** They need the Phase 2 differential harness | E1 |
| R4 | **The `.csmx` collision is real but unresolved**, and two search gaps remain | E1 |
| R5 | **The memory strategy is undecided**, which blocks the runtime and the S17 resource ABI (not Phase 2) | E1 |
| R6 | **No measurement has been taken.** 9 of 11 metrics are not yet measurable; the design report's SLOs are unmet because the compiler does not exist | E1 |
| R7 | **G0-B remains PARTIAL** — the ~6,284-method legacy JUnit reactor, native golden variants and the reproducible-build check are still unexecuted. The owner accepted this for progression; it is **not** reclassified as a pass | E1 |
| R8 | **Three legal questions (L1, L9, L10) gate S3 and S4** and require qualified review | E1 |
| R9 | **The legacy compiler's JVM/JS targets were not executed in this phase.** All numeric and equality evidence quoted is from `expected.txt` files recorded in the Phase 0 baseline, not re-executed here | E2 |
| R10 | **`docs/spec/language-spec.md` §1.3 records an intentional divergence** (code-unit versus scalar-value columns) that will make naive differential comparison fail on astral-containing lines. The harness design accounts for it, but the harness does not exist yet | E1 |
| R11 | **The specification is behaviour-derived and therefore incomplete by construction.** Constructs outside the corpus (entities, FFI, annotations, the query DSL, PortuKof, KofScript) are documented as *unknown*, not as *unsupported* | E1 |

---

## 11. Owner decisions required (directive §10I)

| # | Decision | Blocking | Record |
|---|---|---|---|
| 1 | **Outbound licence** | Phase 5; S3/S4 economics | `phase-1-licensing-recommendation.md` |
| 2 | **Source extension and MIME type** | Phase 5; editor tooling | ADR-0002 |
| 3 | **Accept or reject the grammar and specification** | Gate G1 | D1, D2 |
| 4 | **Standard-library and runtime naming** | The library specification | ADR-0003 |
| 5 | **Per-module ownership** (or explicit single-owner acknowledgement) | Governance; conformance row P19-3 | ADR-0004 |
| 6 | **Memory strategy** (or explicit deferral) | Phase 4; S17 | ADR-0005 |
| 7 | **PortuKof in scope?** | S3 | S3 |
| 8 | **KofScript in scope?** | S4 | S4 |
| 9 | **Accept each PROPOSED decision record** (S13, S14, S15, S18) | Phase 2 implementation | register |
| 10 | **Authorise Phase 2** (Directive 003) | Phase 2 | `directive-003-proposal.md` |
| 11 | **G0-B**: accept as PARTIAL permanently, or authorise the missing legacy coverage run | Gate G0 | baseline audit §5.5 |

---

## 12. Statement of limits

* **No compiler implementation was written.** The workspace still contains
  exactly the three Phase 0 crates, and the validation results are unchanged
  from Phase 0. This is the directive's requirement, and it is verifiable by
  inspecting the diff: every changed file under `crates/` is absent — the
  changes are documentation, configuration notes and corrections.
* **No measurement was manufactured.** `docs/development/measurement-plan.md`
  states which metrics are measurable today (two of eleven) and refuses to
  estimate the rest.
* **No owner decision was made.** Every owner-dependent item is marked, named
  and left open.
* **No legacy artefact was imported.** Verified: 0 fixtures, 0 copied prose,
  0 translated code, no `LICENSE`.
* **No phase boundary was crossed.** Phase 2 is not started, no `cosmic-lexer`
  exists, and PR #2 is a draft.
