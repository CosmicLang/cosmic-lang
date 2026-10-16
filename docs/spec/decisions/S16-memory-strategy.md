# S16 — Memory Strategy

| | |
|---|---|
| **Status** | **BLOCKED-ON-OWNER** — this is a major architectural choice and must not be made by engineering alone |
| **Owner** | Project owner (decision); engineering (study and options) |
| **Phase** | Decision in Phase 1; ADR-0005 carries the record |
| **Related** | ADR-0005, `docs/spec/typing-contracts.md` §9, S11 (Q7), S9 |

## Question

1. **Q1** — What memory model does a compiled Cosmic-Lang program use:
   ownership + borrowing (Rust-style), tracing GC, reference counting (ARC), or
   arena/region allocation?
2. **Q2** — Is the model **language-visible** (the programmer writes lifetimes
   or ownership annotations) or **entirely implicit**?
3. **Q3** — How do **closures** capture (feeds directly from S11 Q7)?
4. **Q4** — How are **resource leaks** handled — statically, or by the runtime?
5. **Q5** — Is the model **uniform across targets**, or per-target?

## Evidence

* **E1 — the design report requires a study, not a choice.** Page 17 states:
  "Escolher estratégia após estudo: ownership+borrowing, GC ou ARC. Não afirmar
  que Rust torna automaticamente o código compilado seguro." That is: *choose
  the strategy after a study, and do not claim Rust makes compiled code
  automatically safe.* The report explicitly refuses to pre-decide.
* **E1 — the legacy language has no memory syntax.** The legacy surface has no
  ownership, lifetime, borrow or move annotations anywhere in its token set or
  grammar (`TokenType.java` and `docs/spec/grammar.ebnf` §4). So there is **no
  legacy syntax to be compatible with** — which means adding a language-visible
  ownership model would be a **language extension**, not a specification of the
  existing language.
* **E1 — the legacy model is per-target, not uniform.** The Phase 0 baseline
  audit records: the JVM and JS targets rely on the **host GC**, while the
  native target implements a **mark-sweep collector with a free list**
  (`docs/audits/phase-0-baseline.md` §3.5, E1).
* **E1 — the legacy compiler performs *compile-time* resource diagnostics.**
  A dedicated ownership pass, a resource-leak analysis and a spawn-capture
  scanner run as part of statement analysis, with their own diagnostic codes
  (`MEM###`, `L-###`). So the legacy model is **hybrid**: compile-time leak
  *diagnostics* layered on runtime reclamation.
* **E1 — the WASM target refuses constructs it cannot represent** rather than
  degrading (recorded in S12/S15), which shows the project already accepts
  per-target capability differences.
* **E1** — Phase 0's own code is `unsafe_code = "forbid"` workspace-wide with
  zero dependencies, so the *compiler's* implementation has a known memory
  posture. That is a separate question from the *compiled programs'* posture,
  and conflating them is a known trap the design report warns about.

## Options

| # | Option | Assessment |
|---|---|---|
| P1 | **Ownership + borrowing, implicit at the language level** (inference-based, no user-written lifetimes) | Keeps the source language unchanged, gives deterministic reclamation, and needs no runtime collector. Cost: the inference problem is genuinely hard, and some programs cannot be expressed — which means a fallback is needed, and the fallback is usually a GC or an arena, so P1 rarely stands alone |
| P2 | **Ownership + borrowing, visible at the language level** (the programmer writes annotations) | Solves inference by exposing it. Cost: it is a **language extension** with no legacy syntax, so every existing `.kf` program would need annotations — a total C1/C2 break |
| P3 | **Tracing GC**, uniform across targets | Uniform semantics, no language change, and the legacy JVM/JS targets already behave this way. Cost: non-deterministic reclamation, and the native target currently does not have a tracing collector (it has mark-sweep, which is a form of tracing GC but with different pause characteristics) |
| P4 | **Reference counting (ARC)** | Deterministic reclamation without a tracing collector, and it works uniformly across targets. Cost: cycles leak unless a cycle collector is added; per-operation cost |
| P5 | **Per-target models with a normative *observable* contract** (reclamation timing is unobservable; only finalization, if any, is specified) | Matches the legacy reality exactly and keeps the door open per target. Cost: the "unobservable" claim is only true if the language exposes no finalizers and no memory introspection — which is currently the case |
| P6 | **Defer** | Blocks the runtime design (S17) and the closure design (S11 Q7) |

## Recommendation (PROPOSED, not in force)

**This record does not recommend a model.** It records the options, the
constraints and the decision criteria, and hands the choice to the owner
through **ADR-0005**. Choosing here would violate both the design report's
explicit instruction (study first) and this project's rule that engineering does
not make product decisions.

**What engineering does recommend — the constraints the choice must satisfy:**

1. **The source language must not change.** P2 is therefore excluded unless the
   owner explicitly authorises a language extension, because it would break
   every existing program (a total C1/C2 break) and it has no legacy syntax to
   be compatible with.
2. **Reclamation timing must be unobservable** for the choice to be
   per-target-compatible. Concretely: no finalizers, no explicit
   free/destroy, no memory-size introspection in the language. This is
   satisfiable today, because the legacy surface has none of these.
3. **The choice must not be justified by "Rust makes it safe."** The design
   report forbids that claim explicitly, and it is false: the memory posture of
   the *compiler* (which is Rust) and the memory posture of the *compiled
   program* (which is whatever the model chooses) are different questions.
4. **Compile-time leak diagnostics are worth keeping** regardless of the
   runtime model: the legacy compiler has them, they are cheap relative to the
   reclamation mechanism, and they catch errors a GC cannot.
5. **Closure capture (S11 Q7) constrains the choice.** If capture is by
   reference and escapes, closures need heap allocation, which pushes toward a
   managed model. S11 Q7 must be answered before ADR-0005 can be decided, or
   ADR-0005 must state its assumption explicitly.

**Decision criteria for ADR-0005** (what the study must produce):

| Criterion | Why it decides |
|---|---|
| Reclamation determinism | Determines whether native targets need a collector |
| Uniformity across targets | Determines whether per-target capability gates (S15) are needed for memory |
| Source-language impact | P2 breaks compatibility; the others do not |
| Closure and cycle handling | Cycles eliminate naive ARC; escaping closures eliminate stack allocation |
| Implementation cost in the compiler | Affects when Phase 3/4 can deliver |
| Observability | Only unobservable timing keeps C2 compatibility across targets |

## Compatibility consequence

* **P3/P4/P5 are C0–C3 neutral** for the source language: reclamation timing is
  not observable in the evidenced surface, so the same program behaves the same
  way.
* **P1 is neutral only if the ownership analysis never rejects a legacy
  program.** If it does, that is a **C1 divergence** — a program that compiled
  before now fails. This is the single largest compatibility risk of P1 and must
  be measured against the corpus.
* **P2 is a total break** and is not compatible at any level.
* **C3 is not promised**, so a per-target memory implementation is explicitly
  permitted.

## Open dependencies

* **ADR-0005** carries the decision and must not be marked Accepted without the
  owner's choice.
* **S11 Q7** (closure capture) must be answered, or explicitly assumed.
* **S17** (ABI versioning) depends on this: the ABI's resource-management
  functions differ per model.
* The study the design report requires has **not** been performed. It is a
  Phase 1 deliverable that Directive 002 authorises but does not itself
  complete — see `docs/audits/phase-1-specification-report.md` §"incomplete
  within this directive".
