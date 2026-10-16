# Compatibility Policy — Proposal for Owner Decision

| | |
|---|---|
| **Date** | 2026-10-16 |
| **Directive** | ENGINEERING DIRECTIVE 001-R1 §8 |
| **Status** | **Proposed.** Nothing here is a decision, a grammar, or a compatibility promise. |
| **Related** | `docs/adr/0001-language-identity.md`, `docs/spec/compatibility-matrix.md`, `docs/audits/phase-0-design-conformance.md`, `docs/audits/phase-0-licensing-matrix.md` |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |

This document proposes a *decision framework* for source-format compatibility.
It deliberately contains **no grammar**, because the design report's own
migration plan assigns grammar freezing to Phase 1 and this review is not
authorised to perform Phase 1 work.

---

## 1. What "compatibility" can mean here

Four distinct claims are routinely conflated. Each is separately decidable,
and this policy keeps them apart:

| Level | Claim | Decidable by |
|---|---|---|
| **C0 — Extension acceptance** | The toolchain recognises the extension and reads the bytes as text | File-extension policy; no semantics involved |
| **C1 — Lexical/syntactic acceptance** | The file parses without error under the new frontend | Grammar (Phase 1) + lexer/parser (Phase 2) |
| **C2 — Semantic equivalence** | The program produces the same observable behaviour as under the legacy compiler | Executed differential tests against the legacy oracle |
| **C3 — Bit-identical output** | Generated artifacts match the legacy backend byte for byte | Per-target backend work (Phase 4) |

**Proposal:** the compatibility promise for any legacy format is stated at
**C1 + C2** only, per format, and only after evidence exists. C3 is **not**
promised for any format — the design report's backend plan (page 16) proposes a
different target order and explicitly contemplates not redoing the JVM backend.
C0 alone is never presented as compatibility.

---

## 2. Per-format decision proposal

### 2.1 `.csmx` — proposed primary Cosmic-Lang source format

| Item | Proposal |
|---|---|
| Role | **Primary** source format for the new toolchain |
| Current state | Proposed in ADR-0001 §2.2 and already used consistently by Phase 0 code, documentation and test fixtures |
| Uniqueness | **Not established.** The design report (page 12) requires registry/GitHub/editor searches before formalising; ADR-0001 §3 records two web queries and states explicitly that exhaustive GitHub code search and trademark/registry queries were **not** performed |
| MIME type | `text/x-cosmic-lang` is proposed by the report (page 12) but **not registered**; `x-` prefixed types are non-standard by construction |
| Editor association | Not created. ADR-0001 §3 recommends a tree-sitter grammar plus editor associations in the phase that introduces the lexer |
| Compatibility level | N/A — this is the native format, not a legacy format |
| Owner decision needed | Approve the extension after the exhaustive search; approve the MIME type and editor-association plan |
| Target phase | Phase 1 (search + approval) → Phase 2 (tooling) |

**Non-promise:** `.csmx` is not claimed to be globally unique. The design
report itself says this cannot be demonstrated, and this review found no
evidence to contradict it.

### 2.2 `.kf` / `.kof` — high-priority legacy syntax compatibility candidates

| Item | Proposal |
|---|---|
| Role | **High-priority compatibility candidates.** These are the legacy language's real sources: 330 `.kf` files in the legacy tree; `.kof` is accepted as an alias by the legacy CLI |
| Evidence | `.kf`/`.kof` are accepted by `KofCliSupport.isKofSource` (verified at source level); the golden corpus is written in this syntax; 222/222 executed checks passed for jvm/js/script |
| Compatibility level | **Target C1 + C2**, scoped to the constructs the compatibility matrix has evidence for |
| Priority | Highest of the four — it is the format that carries the language's actual behaviour |
| What is *not* proposed | No promise of C2 for constructs outside the evidenced set; no promise of C3; no promise that legacy bugs are reproduced (see §4) |
| Owner decision needed | Confirm `.kf` as the first compatibility target, or defer it in favour of `.csmx`-only support |
| Target phase | Phase 1 (freeze the evidenced construct set) → Phase 2 (frontend) → differential validation |

**Non-promise:** no claim is made that every `.kf` program will compile. The
compatibility matrix's rule applies unchanged: *absence of a row means "not yet
analyzed", never "unsupported"* — and equally never "supported".

### 2.3 `.ptkf` — PortuKof, separately specified

| Item | Proposal |
|---|---|
| Role | **Separately specified** compatibility surface. Not a priority-1 target |
| Evidence | 4 `.ptkf` files in the legacy tree; accepted by the legacy CLI through the same extension filter; a `D-PORTUKOF` profile and `Fmt.java`'s "`.ptkf` → PortuKof" profile rule were observed in source |
| What is verified | That the extension is accepted and that a Portuguese keyword surface exists |
| What is **not** verified | Which tokens or idioms differ from `.kf`; whether mixed-surface programs exist; whether the mapping is one-to-one; how identifiers and diagnostics behave |
| Compatibility level | **Not stated yet.** Requires its own specification |
| Owner decision needed | Decide whether PortuKof is in scope at all; if yes, authorise a dedicated specification task |
| Target phase | Phase 1 (specification, if authorised) → Phase 2+ |

**Non-promise:** no PortuKof behaviour is claimed. The design report lists
PortuKof as a distinct surface requiring its own analysis, and this review
confirms the legacy evidence is too thin to specify it.

### 2.4 `.ks` — KofScript, separately specified wrapping and execution semantics

| Item | Proposal |
|---|---|
| Role | **Separately specified.** Not a syntax variant but a *transformation* |
| Evidence | Accepted by the legacy CLI's validation filter (`KofCliSupport`, verified); the legacy tree contains **0** `.ks` files at the reference SHA; the wrapping is documented in source only as materialising the script into a Kof `main()` |
| Compatibility level | **Not stated.** Depends on the exact transformation |
| Unresolved | The precise rewrite applied to a script before it reaches the pure-Kof frontend; whether a `main()` wrapper is synthesised, what the wrapper's signature and return handling are, how top-level statements and imports are treated, and what the execution semantics are for the `script`/`repl` path |
| Owner decision needed | Decide whether KofScript is in scope; if yes, authorise a dedicated specification task that reconstructs the transformation from the legacy source at a frozen SHA |
| Target phase | Phase 1 (specification, if authorised) → Phase 2+ |

**Non-promise:** `.ks` support is not promised, and the zero-in-tree fixture
count means it cannot be validated against the golden corpus.

---

## 3. Cross-cutting rules proposed

1. **The parser is extension-independent.** Extension determines only *which
   surface profile* applies; it must never change the core grammar. The design
   report (page 12) proposes the same rule.
2. **Autodetection is by extension**, with an explicit override; the report's
   migration policy is the reference.
3. **A legacy format is never silently dropped.** Either it is specified, or it
   is explicitly out of scope by owner decision. Silence is not a decision.
4. **Unsupported constructs produce a clear diagnostic**, never a silently
   degraded compilation — the same rule the Phase 0 CLI already follows by
   failing loudly with exit code 2 rather than reporting success.
5. **Compatibility claims require evidence.** A row may claim *compatibility
   verified* only when a `cosmcc-rs` component reproduces the behaviour under a
   test that encodes the legacy evidence (`docs/spec/compatibility-matrix.md`
   §5). Until then the claim stays at "legacy evidence" level.
6. **Legacy defects are not part of the contract.** The maintainers' own open
   items (e.g. nested-generic mis-resolution, JS loop/return miscompiles, the
   cross-target shift divergence recorded in `docs/status.md` §698) are inputs
   to the specification, not behaviours to reproduce. The specification must
   decide each one explicitly.
7. **No artefact copying.** Legacy fixtures and documentation are not vendored
   (`docs/audits/phase-0-licensing-matrix.md` §4).

---

## 4. Specification questions carried forward (not answered here)

These are the design report's own open items plus the ones this review adds.
They are Phase 1 inputs and must be answered by tests or by explicit owner
decisions — never by guessing.

**Inherited from `docs/spec/compatibility-matrix.md` §3:** nested-generic
resolution rules; nullable conversion rules; PortuKof aliasing semantics; `.ks`
wrap semantics; pattern-matching breadth; the concurrency contract; numeric
semantics (overflow, float formatting); string escape and triple-quote
semantics; the stdlib/language boundary; the reserved keywords `entity`,
`extern`, `generated`, `audited`, `unique`.

**Added by this review:**

| # | Question | Why it matters |
|---|---|---|
| Q1 | Is `.kf` compatibility a **Phase 2 gate**, or may `.csmx`-only support ship first? | Determines whether the Phase 2 frontend must accept the legacy surface on day one |
| Q2 | Which of the 74 golden cases constitute the minimum C2 contract for the first target? | The corpus is the only executed oracle; the gate needs a defined subset |
| Q3 | How is a *legacy defect* recorded when the new compiler intentionally differs? | Without a mechanism, differential tests will report false failures |
| Q4 | Does the new toolchain need to read `kof.toml` / `kofdeps.lock`? | The report proposes `Cosmic.toml`/`Cosmic.lock`; no migration story exists for existing projects |
| Q5 | Is the `Kof` language name itself subject to `TRADEMARK.md` constraints? | Affects naming, CLI strings and diagnostics text (see licensing matrix Q3/Q4) |
| Q6 | What is the acceptance criterion for "compatible enough" for `.ptkf`? | 4 in-tree files cannot define a contract |
| Q7 | Must the `.ks` transformation be byte-exact, or behaviour-equivalent? | Changes whether the specification must reconstruct the rewrite verbatim |
| Q8 | Who owns each compatibility row? | The report's governance mitigation requires per-module owners; none are assigned |

---

## 5. Summary table for the owner

| Format | Proposed role | Compatibility level promised | Priority | Phase 1 deliverable | Owner decision required |
|---|---|---|---|---|---|
| `.csmx` | Primary Cosmic-Lang source | N/A (native) | — | Exhaustive uniqueness search + approval | **Yes** |
| `.kf` / `.kof` | Legacy syntax compatibility | **C1 + C2**, scoped to evidenced constructs | **Highest** | Freeze the evidenced construct set | **Yes** |
| `.ptkf` | PortuKof, separately specified | None yet | Separate | Dedicated specification (if in scope) | **Yes** |
| `.ks` | KofScript wrapping/execution, separately specified | None yet | Separate | Dedicated specification (if in scope) | **Yes** |

**No full compatibility is promised for any format**, and no grammar is
invented by this document.
