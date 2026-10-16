# ADR-0005 — Memory Strategy

| | |
|---|---|
| **Status** | **Proposed — DECISION WITHHELD.** This ADR records the options and the constraints. **It must not be marked Accepted until the owner chooses.** |
| | The design report explicitly requires a *study* before this choice, and the decision is an architectural one with no reversible cheap path. |
| **Date** | 2026-10-16 |
| **Deciders** | Project owner (the decision); engineering (the study and the constraints) |
| **Phase** | 1 (study and decision); implementation in Phase 3/4 |
| **Related** | `docs/spec/decisions/S16-memory-strategy.md` (the full analysis), S11 Q7 (closures), S17 (ABI), ADR-0004 |

---

## 1. Context

### 1.1 The design report refuses to pre-decide

Page 17 states the requirement directly: choose the strategy **after a study** —
ownership+borrowing, GC, or ARC — and **do not claim that Rust makes compiled
code automatically safe**.

That instruction is not a formality. It is the report's own guard against the
most common error in this decision: conflating the memory posture of the
**compiler** (which is Rust, and which is `unsafe_code = "forbid"` with zero
dependencies in Phase 0) with the memory posture of the **compiled program**
(which is whatever the runtime model chooses). They are unrelated, and the
report says so.

### 1.2 The legacy language has no memory syntax to be compatible with

**E1.** The legacy surface has **no** ownership, lifetime, borrow, move or
`drop` annotations anywhere in its token set (`TokenType.java`) or its grammar.
Consequence: adding a language-visible ownership model would be a **language
extension**, not a specification of the existing language. It would break
**every** existing program at the C1 level, because each would need annotations
it does not have.

### 1.3 The legacy model is per-target and hybrid

**E1.** From the Phase 0 baseline audit (source-verified):

* JVM and JS targets rely on the **host GC**;
* the **native** target implements a **mark-sweep collector with a free list**;
* the compiler additionally runs **compile-time** resource analyses — an
  ownership pass, a resource-leak analysis and a spawn-capture scanner — with
  their own diagnostic families (`MEM###`, `L-###`).

So the legacy model is **compile-time leak diagnostics layered on runtime
reclamation**, and reclamation differs per target.

### 1.4 The observable-effect constraint

Reclamation timing is **unobservable** in the evidenced surface: there are no
finalizers, no explicit free/destroy, and no memory introspection in the
language. This is what makes per-target reclamation compatible at the C2 level —
and it is a property the choice must **preserve**, not a property to be assumed.

---

## 2. The study the report requires — what it must produce

Directive 002 does **not** complete this study. It is recorded here as the
required input to the decision:

| # | Study question | Why it decides |
|---|---|---|
| ST-1 | How many of the 74 executed golden cases would an ownership+borrowing analysis **reject**? | If the answer is non-zero, an implicit ownership model is a **C1 divergence** and its cost is measurable |
| ST-2 | Do closures capture by value or by reference, and can a capture escape its defining scope? (S11 Q7) | Escaping by-reference capture forces heap allocation and pushes toward a managed model |
| ST-3 | Does the language need cycles that are unreachable but referenced? | Cycles eliminate naive ARC without a cycle collector |
| ST-4 | What reclamation latency is acceptable per target? | Determines whether a tracing collector is needed on native |
| ST-5 | What is the implementation cost of each option inside the compiler (analysis vs runtime)? | Determines when Phase 3/4 can deliver |
| ST-6 | Does any evidenced construct expose memory behaviour (size, identity, address, ordering of destruction)? | If yes, per-target models are **not** C2-compatible |

**ST-1 and ST-6 are the decisive ones.** ST-1 bounds the cost of an implicit
ownership model; ST-6 determines whether the "reclamation is unobservable"
assumption is even true.

---

## 3. Options

| Option | Language-visible? | Reclamation | Uniform across targets? | Compatibility |
|---|---|---|---|---|
| **P1** Implicit ownership + borrowing | No (inferred) | Static, deterministic | Only if every target can honour it | **C1 risk**: rejects programs an inference cannot prove |
| **P2** Visible ownership + borrowing | **Yes** | Static | Yes | **Total C1/C2 break** — every existing program needs annotations |
| **P3** Tracing GC | No | Non-deterministic | Yes if implemented per target | C2-compatible if reclamation stays unobservable |
| **P4** Reference counting (ARC) | No | Deterministic, immediate | Yes | C2-compatible; cycles leak without a cycle collector |
| **P5** Per-target models behind a normative observable contract | No | Per target | No (by design) | Matches the legacy reality; compatible **iff** ST-6 shows nothing observable |
| **P6** Defer | — | — | — | Blocks S17 and the closure design |

**P2 is excluded** unless the owner explicitly authorises a language extension,
because it breaks every existing program and has no legacy syntax to be
compatible with.

---

## 4. Decision

**WITHHELD.** No option is selected. The owner must choose, and the study in §2
must exist first (at minimum ST-1, ST-2 and ST-6).

Engineering's contribution is the **constraint set** the choice must satisfy:

1. **The source language must not change** (excludes P2 absent an explicit
   language-extension decision).
2. **Reclamation timing must remain unobservable** — no finalizers, no explicit
   free, no memory introspection. This is what keeps per-target models
   C2-compatible.
3. **The choice must not be justified by "Rust makes it safe."** The compiled
   program's safety follows from the chosen model, not from the compiler's
   implementation language.
4. **Compile-time leak diagnostics are worth keeping** under any model: they are
   cheap relative to the runtime mechanism and catch errors a collector cannot.
5. **Closure capture (S11 Q7) constrains the choice** and must be answered, or
   assumed explicitly in this ADR.
6. **The choice must be compatible with the S17 ABI rules.** Until the choice is
   made, the ABI **must not expose allocation or reclamation entry points** —
   which is achievable today because no backend exists. **This is why S16 must
   precede Phase 4.**

---

## 5. Consequences of withholding the decision

* **Nothing in Phase 2 is blocked.** The lexer, parser, CST and typed frontend
  are memory-model-agnostic.
* **S17's ABI is partially blocked**: the resource-management entry points
  cannot be designed. Everything else about the ABI can be.
* **The runtime cannot be implemented** until the choice is made — which is
  correct, because implementing a runtime before choosing its model is how the
  legacy two-runtime defect happened.
* **The risk of delay is low and the cost of a wrong choice is high.** A
  language-visible model chosen wrongly (P2) is a break at C1; a per-target
  model chosen when something *is* observable (P5) is a break at C2. Both are
  expensive. Withholding is the correct posture.

---

## 6. Status

* **Proposed, decision withheld.** Not Accepted, and must not be marked Accepted
  by engineering.
* **Prerequisite for:** `cosmic-runtime` implementation, the S17 resource ABI,
  and any Phase 4 backend work.
* **Not a prerequisite for:** Phase 2.
