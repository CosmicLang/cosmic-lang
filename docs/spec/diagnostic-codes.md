# Diagnostic Code Registry and Stability Policy

| | |
|---|---|
| **Status** | **Proposed — Phase 1 draft.** The namespaces below are allocations, not yet used by any code. |
| **Phase** | 1 (registry); used from Phase 2 |
| **Directive** | ENGINEERING DIRECTIVE 002, deliverable D9 |
| **Related** | `docs/spec/decisions/S15-feature-gates.md`, `docs/spec/decisions/S18-mir-invariants.md`, `crates/cosmic-diagnostics/src/code.rs`, `docs/spec/compatibility-matrix.md` §4 |

---

## 1. What is already implemented (E1)

Phase 0 shipped the mechanism. Verified in
`docs/audits/phase-0-source-review.md` §4 and `crates/cosmic-diagnostics`:

| Element | State |
|---|---|
| Format | `[A-Z]{2,8}[0-9]{3}` — 2–8 uppercase ASCII letters followed by exactly 3 digits |
| Construction | `DiagnosticCode::new` is a `const fn` that validates at compile time for embedded literals; `try_new` is the fallible runtime path and never panics |
| Severities | `Error`, `Warning`, `Note`, `Help` |
| Labels | exactly one primary label with a validated byte span, plus secondary labels |
| Suggestions | span + replacement + message + `Applicability` (`machine-applicable`, `maybe-incorrect`, `has-placeholders`, `unspecified`) |
| Canonical order | `DiagnosticBag::sorted` — file id, then start, then end, insertion order for ties |
| Machine format | deterministic single-line JSON, all keys always present, `null` for unresolvable locations |
| Totality | rendering never panics; unresolvable spans degrade to a `path:bytes` fallback |

**What Phase 0 did *not* do:** it never allocated a namespace, and it never
wrote the stability policy down as a registry. The `DiagnosticCode`
documentation states the policy in prose; this document makes it a registry with
allocations and a versioning rule.

---

## 2. Stability policy

### 2.1 The core rule

> **A published code's meaning is frozen forever. A code is retired, never
> repurposed.**

Formally:

1. Once a code appears in a **released** version of the toolchain, its meaning
   (the condition it reports) **must not change**, and it **must not** be moved
   to another condition.
2. To stop using a code, **retire** it: mark it retired in this registry and
   stop emitting it. Never reuse the number for a new meaning.
3. New conditions get **new** codes, appended inside the owning namespace.
4. Retired codes are **never** re-issued, even after a long interval. The
   registry keeps them so that old logs and bug reports stay interpretable.

**Why this rule exists.** Codes are part of the observable interface. Users,
documentation, CI scripts, editor integrations and machine consumers key on
them. A silently redefined code turns an old bug report into a lie, and it is
the diagnostic-layer form of the silent reinterpretation the language rules
forbid (`docs/spec/language-spec.md` §9).

### 2.2 Pre-release freedom

Before the first release, codes **may** be renumbered and reworded freely — the
freeze begins at the first released version. This registry records the
**allocation** so that a renumbering is a visible, reviewable change rather than
an accident.

### 2.3 Legacy codes are not inherited

The legacy compiler has its own code families (`LEX###`, `PARSE###`, `SEM###`,
`PKG###`, `COMP###`, `WASM###`, `FFI###`, `MEM###`, `L###`, `CONC###`). Verified
examples: `LEX001` unterminated block comment, `LEX002` unterminated string,
`LEX008` triple-quoted string rejected, `WASM002` a WASM refusal.

**Rule: no legacy number is inherited.** The new toolchain mints its own codes
with its own meanings. A shared number does **not** imply a shared meaning, and
the two must never be compared by number.

**Consequence for the differential harness.** Because codes are not shared, the
differential test layer (D8 L4) must compare **the condition**, not the code
number, or it must maintain an explicit mapping. `docs/development/test-strategy.md`
§3 L4 already restricts comparison to "code and position" — that must be read as
*the new toolchain's* code, with a condition-level mapping for legacy
comparison.

---

## 3. Namespace allocation

Two to eight letters, so the namespace is the letter run before the digits.

| Namespace | Owner | Scope | Status |
|---|---|---|---|
| `SRC` | `cosmic-source` | File registry, spans, offsets, line/column mapping, size limits, UTF-8 | **Allocated** |
| `LEX` | `cosmic-lexer` (Phase 2) | Lexical errors: malformed tokens, unterminated literals, invalid escapes, unexpected characters | **Allocated** |
| `PARSE` | `cosmic-syntax` (Phase 2) | Syntax errors and error recovery | **Allocated** |
| `RES` | `cosmic-sema` (Phase 2) | Name resolution: unresolved names, duplicate declarations, scope errors | **Allocated** |
| `TYPE` | `cosmic-sema` (Phase 2) | Typing: mismatch, arity, nullability, inference failure, ambiguity | **Allocated** |
| `OVL` | `cosmic-sema` (Phase 2) | Overload resolution specifically (ambiguity, no applicable candidate) — separated from `TYPE` because S1 treats overload ambiguity as its own contract | **Allocated** |
| `PAT` | `cosmic-sema` (Phase 2) | Pattern matching: non-exhaustive switch, unreachable arm, invalid pattern | **Allocated** |
| `CONC` | `cosmic-sema` (Phase 2) | Concurrency: spawn capture, await misuse, unsupported concurrency on a target | **Allocated** |
| `NUM` | `cosmic-sema` (Phase 2) | Numeric: literal out of range, division by zero, narrowing without a cast | **Allocated** |
| `UNI` | `cosmic-sema` (Phase 2) | Unicode: confusable identifier, lone surrogate escape, invalid identifier character | **Allocated** |
| `GATE` | driver (Phase 4) | Feature gates: `refused` on this target, construct not yet specified (S15) | **Allocated** |
| `MIR` | `cosmic-mir` (Phase 3) | MIR construction and verification failures | **Allocated** |
| `OPT` | `cosmic-opt` (Phase 3) | Optimization-pass verification failures | **Allocated** |
| `ICE` | any | Internal compiler error: an invariant the compiler itself must uphold was violated | **Allocated** |
| `CLI` | `cosmic-cli` | Usage errors, unknown commands, unimplemented commands | **Allocated** |
| `CG` | `cosmic-codegen-*` (Phase 4) | Code-generation errors that are not refusals | **Allocated** |
| `LNK` | linker/driver (Phase 4) | Link and packaging failures | **Allocated** |
| `PKG` | package manager (Phase 5) | Manifest and dependency errors | **Allocated** |

**Numbering within a namespace:** `000`–`099` are reserved for conditions that
exist at the namespace's first release; `100`–`899` are general use; `900`–`999`
are reserved for internal/diagnostic-of-diagnostic conditions (for example a
`SRC900` "span failed to resolve against the source map", which the renderers
currently degrade rather than report).

**Rule:** a code belongs to exactly one namespace, and the namespace belongs to
exactly one component. A component may not emit a code from another component's
namespace.

---

## 4. Severity and applicability conventions

| Severity | Convention |
|---|---|
| `Error` | The program's meaning is undefined, or the construct is unsupported/unspecified. Compilation must not proceed to code generation |
| `Warning` | The program is well-defined but suspicious. **A warning must never change the meaning of a program**, and no program that compiled before may fail because of a new warning (S15 Q2/Q4) |
| `Note` | Context attached to another diagnostic |
| `Help` | A suggestion, with an optional machine-applicable fix |

| `Applicability` | Convention |
|---|---|
| `machine-applicable` | The fix is guaranteed to be the intended change; safe for an editor to apply automatically |
| `maybe-incorrect` | The fix is plausible but may change behaviour |
| `has-placeholders` | The replacement contains placeholder text that must be edited |
| `unspecified` | Not declared. **This is the default and it means "do not auto-apply"** |

**Rule:** a suggestion marked `machine-applicable` must be verified by a test
that applies it and checks the result. An unverified auto-applicable fix is a
code-corrupting tool.

---

## 5. The reserved ICE namespace

`ICE###` is allocated to **internal compiler errors** — a violated invariant the
compiler itself must uphold (S18 Q4). Rules:

1. An ICE is **never** a user error. If a condition can be triggered by user
   input, it is a diagnostic in the owning namespace, not an ICE.
2. An ICE must be **reported, never panicked**. Phase 0's rule is that `panic!`
   may only encode a programming error that tests catch; an ICE that reaches a
   user is reported through the diagnostic path with a stable code and a
   non-zero exit.
3. Every ICE must carry enough context to reproduce: the failing component, and
   for a verifier failure, the function/block/operation index (S18 Q3).
4. **`ICE` is the metric surface for M8** (`measurement-plan.md`): the ICE rate
   is counted by these codes, which is what makes the design report's "zero known
   ICE on the corpus" SLO measurable at all.

---

## 6. Registry format

This section defines the table that each namespace maintains. **No codes are
listed yet**, because no component emits any — listing invented codes would
create a stability obligation for a meaning nobody has implemented.

| Column | Meaning |
|---|---|
| Code | The `[A-Z]{2,8}[0-9]{3}` identifier |
| Condition | The precise condition reported. This is the frozen meaning |
| Severity | `Error` / `Warning` / `Note` / `Help` |
| Introduced | The release in which it first appeared |
| Retired | The release in which it stopped being emitted, or `—` |
| Decision record | The `S<n>` that governs the condition, where one applies |
| Evidence | A test name that exercises it |

**Rule:** a code may not be added to the registry without an `Evidence` entry.
An untested diagnostic is a diagnostic that may not exist.

### 6.1 Conditions that will need codes, by decision record

Recorded here so that the registry has a planned shape rather than being
invented code by code in Phase 2:

| Decision record | Conditions needing codes |
|---|---|
| S1 | Nested-generic resolution failure; inference failure; no applicable overload; ambiguous overload; arity mismatch |
| S2 | Nullability mismatch; implicit unwrap not permitted; "soft cast" undefined (see the record — this condition is itself unresolved) |
| S5 | Non-exhaustive switch; unreachable switch arm; invalid pattern; guard not boolean |
| S6 | `await` on a non-handle; spawn capture violation; concurrency unsupported on this target |
| S7 | Integer literal out of range; division by zero (behaviour OPEN); narrowing without a cast |
| S8 | Unknown escape sequence; lone surrogate escape; unterminated literal; invalid `\u` |
| S9 | Unknown library member; property called as a method or vice versa |
| S10 | Reserved keyword not supported in this version; unknown annotation |
| S13 | Confusable identifier (warning); invalid identifier character |
| S14 | Interior `/*` in a block comment (warning) |
| S15 | Feature `refused` on this target; construct **not yet specified** (its decision record is OPEN) |
| S18 | MIR verification failure (ICE); CFG invariant violation (ICE); dominance violation (ICE) |

**Note the S15 row.** The `GATE` namespace carries the "construct not yet
specified" error, which is the mechanism that turns every OPEN decision record
into a hard error instead of an accidental language choice. It is the most
important code family in the registry, because it is what keeps Phase 2 honest.

---

## 7. Versioning of the registry itself

| Change | Classification |
|---|---|
| Adding a code inside an existing namespace | **Minor.** Additive; no compatibility impact |
| Retiring a code | **Minor**, but it is an observable change: the condition is no longer reported. Must be documented |
| Changing a code's severity | **Major.** A warning becoming an error breaks programs. Requires a decision record |
| Changing a code's condition | **Prohibited.** Retire and mint a new code instead |
| Renumbering a code | **Prohibited** after the first release |
| Adding a namespace | **Minor.** The namespace must be registered here first |
| Removing a namespace | **Major**, and effectively prohibited, because it would orphan retired codes |

---

## 8. What this document does not do

* **It does not list codes.** §6 defines the format and §6.1 the planned
  conditions; the codes themselves are minted when the component that emits them
  is written, with a test as evidence.
* **It does not adopt legacy numbers.** Explicitly prohibited by §2.3.
* **It does not decide severities for unresolved conditions.** A condition whose
  decision record is OPEN cannot have a severity, because its severity depends on
  the decision — which is why the `GATE` "not yet specified" error exists.
