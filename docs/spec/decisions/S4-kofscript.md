# S4 — KofScript (`.ks`) Wrapping and Execution Semantics

| | |
|---|---|
| **Status** | **BLOCKED-ON-OWNER** — whether KofScript is in scope is a product decision |
| **Owner** | Project owner (scope); engineering (analysis) |
| **Phase** | Phase 1 decision; specification and implementation only if authorised |
| **Related** | `docs/spec/compatibility-policy.md` §2.4, `docs/spec/language-spec.md` §4.3, ADR-0002 |

## Question

1. **Q1** — Is KofScript in scope for the new toolchain?
2. **Q2** — What is the **exact** transformation applied to a `.ks` file before
   it reaches the Kof frontend?
3. **Q3** — Must the transformation be byte-exact, or is behaviour-equivalence
   sufficient?
4. **Q4** — What are the execution semantics — in-process compilation, a
   persistent REPL state, and the lifetime of top-level bindings?
5. **Q5** — How do the REPL globals interact with the wrapping?

## Evidence

* **E1** — `.ks` is accepted by the legacy CLI's validation filter
  (`kof-cli/src/main/java/dev/kof/cli/KofCliSupport.java`).
* **E1** — The legacy tree contains **0** `.ks` files at the frozen SHA. There
  is therefore **no fixture and no executed evidence** for KofScript behaviour.
* **E1** — The wrapping entry point is
  `KofScript.wrapPureKof(String code)`
  (`kof-script/src/main/java/dev/kof/script/KofScript.java`).
* **E1** — The transformation is documented in-source as: KofScript is *pure
  Kof executed directly*; when no `main()` is declared, the **top-level
  statements become `main()`**. The wrapper's stated single service is the
  script execution model. No other language sugar is introduced.
* **E1** — The implementation is **line-oriented text manipulation**:
  `code.split("\n", -1)`, per-line stripping, splitting a line at a `;` to
  separate a first statement from the remainder, and accumulating lines into a
  block. This is a text transformation, not a parse-and-rewrite.
* **E1** — Execution is **in-process**: `KofScript.eval(code, target)` compiles
  and runs, returning a `RunResult(exitCode, stdout, stderr, success)`, and the
  result is cached keyed by a hash of the *wrapped* source and the target.
* **E1** — There is a REPL (`KofRepl`) with a persistent globals model
  (`ScriptGlobals`, `ScriptGlobalTypes`), plus an executor
  (`KofScriptExecutor`).
* **E1** — `kof-script` has 15 Java files and 9 test classes, including a
  syntax-coverage test and several interop end-to-end tests (Python, R). So the
  surface is real but its tests are **not** part of the executed Phase 0
  baseline (which covered the golden corpus and a JUnit slice only).

## Options

| # | Option | Assessment |
|---|---|---|
| D1 | **Out of scope.** `.ks` produces a clear "unsupported format" diagnostic | Honest and cheap. The format has no fixtures, so nothing can be validated anyway |
| D2 | **In scope, behaviour-equivalent wrapper** — specify the *observable* rule ("top-level statements execute as the body of the entry point") and implement it directly in the frontend, without reproducing the legacy text transformation | Cleaner, and avoids enshrining a line-oriented text hack. Risk: edge cases (a top-level declaration mixed with statements, a `;`-separated line, a trailing block) may diverge |
| D3 | **In scope, byte-exact transformation** — reproduce the text wrapper exactly | Maximum compatibility, but it forces the new frontend to reproduce a fragile line-based algorithm whose behaviour on edge cases is unknown and untested |
| D4 | **Defer**, while rejecting `.ks` loudly | Safe interim, same as C4 in S3 |

## Recommendation (PROPOSED, not in force)

**D4 as the interim behaviour, then D2 if the owner wants KofScript.**

* **Interim:** reject `.ks` loudly. Never parse it as Kof — a script file has no
  `main()`, so parsing it as Kof would produce a cascade of top-level errors.
* **If in scope:** implement **D2** (behaviour-equivalent), not D3.
  Rationale: the legacy transformation is line-oriented text manipulation with
  no fixtures and no executed evidence. Reproducing it byte-for-byte would mean
  freezing an algorithm that nobody has characterised, and the licensing matrix
  forbids deriving the new implementation from the legacy implementation code.
  A behaviour-level rule can be specified from observed *inputs and outputs*,
  which is exactly the evidence model the project uses.
* **Q2** — The behaviour-level rule to specify and validate: *a `.ks` file's
  top-level statements execute in order as the body of the program entry point;
  the file's observable effect equals the same statements placed inside an
  entry-point body*. Any deviation found by testing becomes a documented
  difference.
* **Q3** — Behaviour-equivalent, not byte-exact. C3 is not promised for any
  format.
* **Q4/Q5** — The REPL/globals model is a **tooling** concern, not a language
  concern. If KofScript is authorised, the REPL must be specified separately;
  this record does not attempt it, because a persistent-globals model has no
  language-level evidence at all.

## Compatibility consequence

* D2 would claim **C1 + C2** for the wrapping behaviour, scoped to what tests
  can validate.
* Because there are **no fixtures**, the claim would rest on newly authored
  fixtures derived from observed behaviour. That is weaker evidence than the
  `.kf` corpus and must be stated as such wherever compatibility is claimed.

## Open dependencies

* **Owner scope decision** (Q1).
* The legacy `kof-script` JUnit tests were **not executed** in the Phase 0
  baseline. If KofScript is authorised, executing them (read-only, on a
  compatible host) is the cheapest way to obtain evidence — and it must be
  recorded as a legacy-execution result, not as a new-repository test.
* The REPL/globals surface requires its own specification if it is in scope.
