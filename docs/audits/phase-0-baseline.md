# Phase 0 — Legacy Baseline Audit

| | |
|---|---|
| **Date** | 2026-10-16 |
| **Directive** | ENGINEERING DIRECTIVE 001 — Rust Compiler Foundation, Legacy Baseline and Repository Bootstrap |
| **Legacy repository** | `https://github.com/CosmicLang/cosmcc.git` (read-only) |
| **Reference SHA** | `284358f43db885ccd8b52b11f897ac0defec10de` (2026-10-13) — verified, matches the directive |
| **Audit checkout** | isolated clone at `/private/var/folders/.../opencode/cosmcc-audit` (original repository untouched) |
| **Legacy version** | `0.6.5` (`VERSION` file; git tip: "merge: promote prerelease → main — cut 0.6.5") |

Findings are tagged: **E1** directly verified fact · **E2** evidence-supported inference · **H** hypothesis requiring investigation · **P** proposed engineering decision.

---

## 1. Scope and method

1. Cloned `cosmcc` into an isolated temporary directory and checked out the reference SHA. The original repository was used read-only (`git clone` + `git checkout` only).
2. Audited by direct code inspection (targeted greps and file reads) and full-file analysis of module structure, lexer/parser, IR, semantics, backends, tests, CI and docs.
3. Attempted a reproducible build and test baseline on this machine (macOS). Commands and raw results are recorded in §5.
4. Two deep-dive surveys (compiler architecture; tests/CI/project files) were performed with the same read-only discipline and folded into this report.

**Note on the design report:** the 20-page document "Cosmic-Lang — Projeto do Compilador Rust" referenced by the directive was **not provided in the execution session**. The architecture planning in this repository therefore follows the pipeline and component list stated in directive §5, and this audit never cites the report as evidence. This limitation is restated in §8.

---

## 2. Execution environment (E1)

| Tool | Version / note |
|---|---|
| OS | macOS 27.0.1 (build 26A434), arm64 (Apple silicon) |
| Rust | rustc 1.99.0 (b940084d7 2026-09-28), cargo 1.99.0 (Homebrew) — see `docs/development/toolchain-policy.md` |
| Java | OpenJDK 27 (Homebrew); legacy targets `--release 25` |
| Maven | Apache Maven 3.10.0 |
| Node.js | v26.12.0 |
| Git | 2.54.0 (Apple Git-157) |
| GitHub CLI | gh 2.102.0 |
| Missing | `gradle`, `rustup`, `wasm-pack` |

---

## 3. Repository facts (E1)

### 3.1 License and attribution

* `LICENSE` at the repository root is the **GNU General Public License, version 3** (GPL-3.0), full text, no additional terms file observed.
* `TRADEMARK.md` and `SECURITY.md` exist; contributions/CoC docs exist.
* The Maven POMs (`pom.xml`) declare **no `<licenses>` section**; the license is expressed only through the root `LICENSE` file.

### 3.2 Module structure (Maven multi-module, group `dev.kof`, version `0.6.5`)

| Module | Main Java files | Responsibility (E1, by inspection) |
|---|---|---|
| `kof-compiler` | 1,797 | lexer → parser → AST → semantics → IR → optimizer → backends (JVM, native, JS, WASM, MCU, interpreter) |
| `kof-cli` | 202 | `kof` executable: 31 command words, decompiler, translator, LSP, benchmark, package manager |
| `kof-runtime` | 12 | GraalJS host/runner and bridge shims (`dev.kof.runtime`); **not** used by generated JVM code (E1) |
| `kof-script` | 15 | `KofScript` direct execution/REPL; `.ks` handling |
| `kof-c-compiler` | 19 | C-subset→native path driven by `kof c` |

Build facts (E1): `maven.compiler.release=25`; reproducible-jar property `project.build.outputTimestamp=2020-01-01T00:00:00Z`; ASM pinned to `9.10.1` with an in-file explanation; JUnit `6.1.3`; GraalJS `25.4.4.1.1`.

### 3.3 Frontend (E1)

* Hand-written lexer (`parser/Lexer.java`, 449 lines): `//` and `/* */` comments, triple-quoted strings, char literals, a leading UTF-8 BOM is tolerated, CRLF and lone CR count as line breaks, identifiers may start with `_` or `$`.
* Hand-written parser (`parser/Parser.java` + 11 cooperating sub-parsers). No parser generator.
* Keywords (`lang/KofKeywords.java`) are Java-like (`class`, `interface`, `record`, `enum`, `entity`, `extern`, `static`, `val`, `var`, `spawn`, `await`, …) **plus the reserved-but-invalid tokens `fun`/`fn`/`func`** — reserved deliberately so they cannot return as identifiers (documented in-source).
* AST: ~55 record types implementing non-sealed marker interfaces (`AstNode`, `ExpressionNode`, `StatementNode`, `MemberNode`, `TypeDeclarationNode`). `AstNodes.java` contains only orphan Javadoc — the file-name claim in `docs/architecture/compiler-architecture.md` is stale (E1).
* No typed AST: expression types are resolved in analyzer-side maps and re-inferred during lowering (E2, consistent with the doc's own §8 item 4).

### 3.4 IR and optimization (E1)

* IR: `IRModule` → `IRClass` → `IRMethod` → `IRBasicBlock` (index + operations). A **typed linear stack machine** with label jumps and exception-region markers.
* `KofOperation` is an empty marker interface (not sealed) with **37** record implementations (measured).
* Lowering: one basic block per method initially; dedicated lowerers (`StatementLowerer`, `ExpressionLowerer`, `SwitchExprLowerer`, collection lowerers, desugar steps).
* Optimizer (`backend/Optimizer.java`): constant folding, constant/copy propagation, dead-effect elimination, reachability, jump-to-next removal; ≤ 8 iterations. No inlining, no LICM.

### 3.5 Semantic analysis (E1)

* `SemanticAnalyzer` with phase contract `preDeclareType → defineMembers → analyzeDeclaration (fixpoint ≤ 4) → resolveMethodCalls`; the last is an intentional no-op vestige; real overload resolution lives in `MethodCallTyper`/`OverloadSelector`.
* `Type` is a sealed interface with 8 variants: `PrimitiveType`, `ClassType`, `TypeVariable`, `FunctionType`, `ArrayType`, `WildcardType`, `UnknownType`, `NullableType`.
* Nominal assignability BFS over ordinary supertypes with declaration-site variance and use-site wildcards (SG-009); an older structural path consults runtime class relations before relaxing.
* Verified feature families by test presence (E1): generics/erasure, nullability, pattern matching, records, interfaces with default methods, sealed types, variance, enums with switch expressions, exceptions, concurrency (`spawn`/`await`).
* Memory model: compile-time ownership/resource diagnostics (`memory/OwnershipPass`, `ResourceLeakAnalysis`, `SpawnCaptureScanner`; codes MEM001/…), executed as part of statement analysis. Runtime GC is per-target (host GC on JVM/JS; mark-sweep + free list on native).

### 3.6 Backends (E1 for structure / E2 where noted)

* **JVM** (`jvm/JvmBackend`, 60 files): ASM with `COMPUTE_FRAMES`; generated code includes a generated `KofRuntime` that is compiled **in place with `javac`** — a full JDK is required to run compiled output.
* **Native** (`nat/NativeBackend`, 226 files): string-assembly for x86-64, real lowering for RISC-V 64, an aarch64 translator (doc itself calls it "regex-based … fragile" — E2), ELF output through `as`/`ld`; MCU targets RV32I/Thumb-2 with qemu-based gates.
* **JS** (`js/JsBackend`, 66 files): stack IR → JS IR tree → ES2022 ESM + `kof-runtime.mjs` + source maps; executed in-process by a GraalJS host.
* **WASM/WASI** (`wasm/WasmBackend`, 14 files): binary wasm for a scalar subset with documented honest refusals (`WASM001`/`WASM002`). The Javadoc in `Target.java` still claims "SEM EMISSÃO AINDA" while tests assert emission since 15.2/15.3 — stale docs (E1).
* **Android**: JVM backend with `AndroidProjectWriter`; **SCRIPT**: direct IR interpretation (`KofInterpreter`), no artifact; **vk/GPU**: FFI stubs to `libvkchain` (x86-64) plus JVM FFN path — not a backend implementation (E1).
* ABI: JVM mapping to `java/lang/String`, `ArrayList`, `HashSet`, `Map`, `LinkedBlockingQueue`, `CompletableFuture` (E1, `JvmTypeMapper`); native runtime ABI documented in `docs/runtime/RUNTIME_ABI.md`; FFI surface in `docs/ffi-abi-structs.md` with per-backend status codes.

### 3.7 Language surface files and project files (E1)

* Extensions accepted by the CLI: **`.kf`** (Kof, 330 files in-tree), **`.kof`** (alias), **`.ptkf`** (PortuKof — Portuguese keyword surface, 4 files), **`.ks`** (KofScript statement scripts; materialized into a `main()` wrapper before reaching the pure-Kof frontend; zero `.ks` files in-tree at this SHA).
* `kof.toml`: parsed by a deliberately minimal INI-subset TOML reader (`KofProjectConfig`): `[project] name`, `[backend] target`, `[frontend] target`, `[server] port`, `[sources] app|test`. Unknown keys are warnings, never errors.
* `kof.lock` does **not** exist. Dependency locking is `kofdeps.lock`, generated by `kof deps`, gitignored, currently absent from the tree.

### 3.8 CLI surface (E1)

* 31 command words in `kof-cli/src/main/java/dev/kof/cli/Main.java`: `build, run, serve, check, md, test, bench, profile, inspect, decompile, translate, compare, migrate, debug, info, lsp, install, script/repl, init, new, connector, deps, deploy, workflow, makealive, c, fmt, editor, config, version` (+ help on unknown).
* **Verified wart (E1, executed):** `kof --version` is not handled; it prints `unknown: --version` plus the usage text and **exits 0**. `kof version` prints `kof 0.6.5` and exits 0. The new `cosmic` CLI intentionally does not reproduce the exit-0-on-unknown-flag behavior.

### 3.9 Test suite and CI (E1)

* 835 JUnit test classes (~6,284 `@Test` methods, ~208k lines of test code) across kof-compiler (741), kof-cli (80), kof-script (9), kof-c-compiler (5), kof-runtime (0 own tests).
* `tests/golden/`: **74 cases** (`Main.kf` + `expected.txt`), executed for jvm/native/js/script by `tests/run-golden.sh`, comparing stdout exactly and requiring exit code 0; targets whose runtime is absent are SKIPPED with a stated reason.
* Legacy CI: 16 workflows; PRs require structural quality checks, `mvn clean package` + golden + integration tests, a 3-OS matrix job, and a qemu cross-native suite; CodeQL/secret-scan/benchmark bots attach to PRs. The legacy repository additionally enforces PRs-to-`main` by code-owner gate (`pr-base-guard.yml`).
* Repo hygiene (E1): committed build artifacts at the repository root (`binutils-*.deb`, `build-probe/`, `_test_dir/`, `bench-io-data.txt`).

---

## 4. Language constructs verified from source (E1) — evidence seed for the compatibility matrix

Extracts from the golden corpus at the reference SHA (full table in `docs/spec/compatibility-matrix.md`):

* Top-level `main()` without any function keyword; functions declared as `Type? name(params) { … }` or with expression bodies `name(params): Type = expr`.
* `record Point(Int x, Int y)` with accessor call syntax `p.x()`.
* `class Dog extends Animal`, `public constructor(String name)`, `super(name)`, field assignment `d.name = "rex2"`.
* `interface Speaker { speak(): String }` and `class Cat implements Speaker { speak(): String = "meow" }`.
* `enum Color { RED, GREEN, BLUE }` + `switch` expressions with `case Color.RED -> "red"`.
* Nullability suffix `List<Int>?` with `null` comparisons; generics with nested type arguments.
* `for (var a in animals)`; `var`/`val`; exceptions `try/catch (String e)/finally` with `throw "boom"`; `spawn`/`await` handles.
* No statement-terminating semicolons are required (newline-separated); `fun`/`fn`/`func` are reserved and invalid.

---

## 5. Build and test baseline — what was executed (E1)

All commands run in the isolated clone with `JAVA_HOME` set to the Homebrew OpenJDK 27.

### 5.1 Build

```
mvn -B -ntp -DskipTests install
→ BUILD SUCCESS — 6 modules (parent, runtime, compiler, script, c-compiler, cli) in 01:40 min
```

### 5.2 Golden corpus (functional baseline)

```
tests/run-golden.sh --target jvm --target js --target script
→ GOLDEN TESTS: 222 passed, 0 failed, 0 skipped   (74 cases × 3 targets)
```

This is the strongest executed evidence: each case was compiled by the legacy compiler, executed on the JVM / Node.js / the script interpreter, and its stdout matched `expected.txt` exactly with exit code 0.

### 5.3 Native target on this host — NOT EXECUTED

```
tests/run-golden.sh --target native hello loops-control
→ FAIL [hello/native] compilation
```

Root cause verified by re-running the build verbosely: the native backend emits GNU/Linux ELF assembly (`.section .data`, `.type X, @object`) and the macOS `as` is a Mach-O assembler that rejects these directives:

```
Main.s:2:15: error: unexpected token in '.section' directive
Main.s:64:1: error: unknown directive
.type Default_Main_vtable, @object
```

This is a **platform limitation of this audit host**, not evidence of a compiler defect: the legacy CI runs native tests on Ubuntu with cross binutils and qemu. Native golden targets are marked NOT EXECUTED, with the reason above.

### 5.4 JUnit slices

```
mvn -B -ntp test -pl kof-compiler -Dtest='SemanticResolutionTest,MemoryModelTest,OperatorMethodParseTest' …
→ Tests run: 40, Failures: 0, Errors: 0, Skipped: 0

mvn -B -ntp test -pl kof-script …
→ Tests run: 69, Failures: 1, Errors: 0, Skipped: 1
   failing test: KofScriptTest.evalNativeTarget — same macOS `as` limitation (§5.3), not a compiler defect
   skipped test: an R-interop E2E test (R runtime not installed on this host)
```

### 5.5 Not executed (explicitly incomplete)

* **Full JUnit reactor** (~6,284 tests): not executed. The legacy suite requires Linux-specific facilities (MongoDB service, qemu, cross binutils, WebKitGTK, per-OS matrix) and hours of wall time; running it on this macOS host would not reproduce the legacy CI baseline. **The authoritative legacy baseline remains the legacy project's own CI** on Ubuntu; this audit records the executed subset above.
* **Native/Android/browser/multi-arch golden variants**: not executed (reasons above).
* **Reproducible-build check** (`scripts/check-reproducible-build.sh`, two clean rebuilds ≈ 6 min): not executed in this session.

---

## 6. Licensing and attribution (E1 facts, P implications)

* **E1:** The legacy compiler is **GPL-3.0**. Any derivative work of that source — including a line-by-line or mechanical translation of implementation code into Rust — must be distributed under GPL-3.0-compatible terms, and carries the GPL's licensing obligations.
* **E1:** The new repository `CosmicLang/cosmic-lang` currently contains **no LICENSE file**; the license of `cosmcc-rs` has not been decided by the owner.
* **P (proposed, requires owner decision):** keep `cosmcc-rs` a **clean reimplementation**. The language's *semantics and observable behavior* are the compatibility contract we build against (verified through the legacy test corpus like the golden suite), while *implementation code is not translated*. Directive principle 2 ("Do not port Java source mechanically to Rust") aligns with this. Until the owner chooses a license, no license file is added and no copied code is introduced. **This is an open licensing decision escalated to the owner (see ADR-0001).**

---

## 7. Verified defects and warts in the legacy snapshot (E1)

Only behaviors verified first-hand are listed; maintainer-documented open bugs are quoted as such and not re-verified.

1. `kof --version` prints `unknown: --version` + usage and **exits 0** (executed; §3.8). Unknown flags should not look like success.
2. Version stamps disagree: `VERSION` = `0.6.5` vs `README.md` "0.6.1" vs `docs/status.md`/`docs/backend-parity.md` "0.6.4".
3. Architecture docs drift from code: `compiler-architecture.md` claims a sealed `AstNodes.java` and "30 sealed IR ops" (reality: Javadoc-only file; 37 records, not sealed), claims `WildcardType` is never constructed (it is), and `Target.java` still says WASM emits nothing (it emits).
4. Committed build artifacts at the repository root (§3.9).
5. Known open defects in the maintainers' own ledger at this SHA (`docs/status.md`, 13/10/2026): §705 native x86 hang (AVIF slice, OPEN), §706 `Box<Box<Int>>` nested-generics mis-resolution (OPEN), §703/§704 JS loop/return miscompiles (reverted/OPEN). These are **not** independent findings of this audit; they are the project's documented state and matter to the compatibility baseline because the new compiler must eventually decide how to treat them.

---

## 8. Risks and limitations of this audit

* **Evidence limitation (H → mitigated):** the design report was not available in-session (§1).
* **Platform divergence (E2):** the legacy snapshot is developed/verified on Linux (CI). macOS-only executions here cover the JVM, JS, script targets and slices of JUnit; everything native/Android/browser remains Linux CI territory. Numbers from this audit must not be presented as the legacy project's full suite status.
* **Suite size (E1):** the full JUnit reactor was not executed (§5.5). G0-B is therefore reported as **partially complete**.
* **Semantics beyond the golden corpus (H):** PortuKof aliasing, `.ks` wrapping, entity/ORM "makealive" codegen, workflow/pipeline facilities and the `c` command are large surfaces that the golden corpus does not exercise; the compatibility matrix marks them as open questions rather than settled semantics.
* **Licensing uncertainty (P):** the new repository has no license yet; compatibility work must stay behavior-level until the owner decides (§6).

---

## 9. What the new compiler can safely build on

* The legacy **golden corpus (74 cases, 4 target classes)** is a ready-made behavioral oracle; 222/222 executed checks passed here for jvm/js/script.
* The legacy language is regular enough that a hand-written lexer/parser matches its needs (no parser generator backup needed as a requirement).
* Diagnostics vocabulary precedent: stable codes with an UPPERCASE family prefix + 3 digits (e.g. `SEM011`, `PKG004`, `WASM002`) — the new diagnostics crate adopts the same shape (`docs/architecture/compiler-pipeline.md` §4).
* Source formats `.kf`/`.kof`/`.ptkf`/`.ks` and the `kof.toml`/`kofdeps.lock` split (project file vs generated lock) are inputs to the identity decision in `docs/adr/0001-language-identity.md`.
