# Phase 1 — Legacy Behavioural Probe Record

| | |
|---|---|
| **Status** | **Evidence record.** Read-only probes against the frozen legacy compiler. |
| **Directive** | ENGINEERING DIRECTIVE 002-R1 §4 |
| **Legacy reference** | `CosmicLang/cosmcc` @ `284358f43db885ccd8b52b11f897ac0defec10de` (v0.6.5) |
| **Probe date** | 2026-10-16 |
| **Host** | macOS (Apple silicon), OpenJDK 27 (`/opt/homebrew/opt/openjdk@27`), Maven 3.10.0 |
| **Isolation** | Fresh clone at `/tmp/cosmic-probe/legacy`; probes at `/tmp/cosmic-probe/probes`. **Nothing from this record was copied into this repository** — no `.kf` fixture, no legacy code, no legacy documentation |

> **Path convention.** A path prefixed `cosmcc:` refers to the **legacy repository** `CosmicLang/cosmcc` at the frozen SHA — it does not exist in this repository. Unprefixed paths are relative to this repository.

---

## 0. Method, and what this record does and does not establish

### 0.1 Method

1. The legacy project was cloned read-only into an isolated temporary directory
   at the frozen SHA and built with `mvn -B -ntp -DskipTests install`
   (BUILD SUCCESS, 6 modules, 1 m 32 s). The shaded CLI jar
   (`kof-cli/target/kof-cli-0.6.5.jar`, 251 MB) was used directly.
2. Probe programs were **written for this record** — small, minimal, and not
   derived from any legacy fixture. They exercise the constructs the decision
   records could not settle from the corpus.
3. Each probe was run with `kof run <file> --target jvm` (or `kof check` where
   diagnostics, not program output, were the object of study).
4. The command, the program, the stdout, the stderr and the exit status are
   recorded verbatim below.
5. **Independent reproduction.** Every probe was re-executed a second time, on
   the same day, from a fresh copy of the probe set (`/tmp/cosmic-probe/rerun`)
   against the same frozen jar. All stdout, stderr and exit statuses matched
   this record, with **one correction**: `kof check` emits diagnostics on
   **stdout**, not stderr — see §7.4. Stream attribution for the affected
   entries was corrected; no observed content changed.

### 0.2 Two rules this record enforces

**Rule 1 — a compiler defect is not a normative language rule.** Where the
legacy compiler crashes, mis-compiles, or silently drops information, the
observed behaviour is recorded as **VERIFIED LEGACY BEHAVIOUR (defect)** and is
**not** promoted to a language rule. The proposed rule, where one is proposed,
is marked separately.

**Rule 2 — do not claim a decision is closed when the probe does not
distinguish its alternatives.** Three questions in this record are explicitly
reported as **NOT DISTINGUISHED**: overload ambiguity between unrelated types,
`kof check`'s behaviour on a long run of malformed statements, and whether a
nested function could capture if it were allowed to escape. They remain open.

### 0.3 Notation

| Marker | Meaning |
|---|---|
| **OBSERVED** | The exact output, quoted |
| **VERIFIED** | The observation distinguishes the alternatives of the question |
| **NOT DISTINGUISHED** | The observation does not distinguish them; the question stays open |
| **DEFECT** | The observed behaviour is a compiler bug, not a language rule |
| **PROPOSED** | A rule proposed for the new toolchain, based on the observation |

---

## 1. Build and invocation baseline

```sh
export JAVA_HOME=/opt/homebrew/opt/openjdk@27
cd /tmp/cosmic-probe/legacy
mvn -B -ntp -DskipTests install          # BUILD SUCCESS, 01:32 min
java -jar kof-cli/target/kof-cli-0.6.5.jar version   # -> kof 0.6.5
```

Every probe below ran on `--target jvm`. The JVM target was chosen because it is
the target the legacy project treats as the oracle, and because the native
target cannot be exercised on this host (GNU-ELF assembly versus the macOS
assembler — the same platform limitation recorded in
`docs/audits/phase-0-baseline.md` §5.3).

**Probes were not run on `--target js` or `--target script`.** Cross-target
divergence is therefore **not** covered by this record, and the
cross-target-divergence risk recorded in the Phase 0 audit is unaffected.

---

## 2. S11 — evaluation order and binding

### 2.1 Argument order

```sh
java -jar $JAR run probes/S11-arg-order/main.kf --target jvm
```

```kof
Int a() { println("A"); return 1 }
Int b() { println("B"); return 2 }
Int sum(Int x, Int y) { return x + y }
main() { println(sum(a(), b())) }
```

* exit `0`
* **OBSERVED stdout:** `A`, `B`, `3`
* **VERIFIED:** function arguments are evaluated **left to right**.

### 2.2 Receiver versus arguments

```kf
class T {
    Int v
    public constructor(Int v) { this.v = v }
    m(Int x): Int { return this.v + x }
}
T recv() { println("RECV"); return T(10) }
Int arg() { println("ARG"); return 5 }
main() { println(recv().m(arg())) }
```

* exit `0`
* **OBSERVED stdout:** `RECV`, `ARG`, `15`
* **VERIFIED:** the **receiver is evaluated before the arguments**.

### 2.3 Binary operand order

```kf
Int l() { println("LHS"); return 1 }
Int r() { println("RHS"); return 2 }
main() { println(l() + r()) }
```

* exit `0`
* **OBSERVED stdout:** `LHS`, `RHS`, `3`
* **VERIFIED:** binary operands are evaluated **left to right**.

### 2.4 Short-circuiting

```kf
Bool f() { println("F"); return false }
Bool g() { println("G"); return true }
main() {
    if (f() && g()) { println("AND-TRUE") } else { println("AND-FALSE") }
    if (g() || f()) { println("OR-TRUE") } else { println("OR-FALSE") }
}
```

* exit `0`
* **OBSERVED stdout:** `F`, `AND-FALSE`, `G`, `OR-TRUE`
* **VERIFIED:** `&&` and `||` **short-circuit**. In the `&&` case `g` was never
  evaluated (no `G` before `AND-FALSE`); in the `||` case `f` was never
  evaluated (no `F` before `OR-TRUE`).

### 2.5 Assignment associativity — the question dissolves

```kf
main() {
    var a = 0
    var b = 0
    a = b = 1
    println(a)
    println(b)
}
```

* exit `1`
* **OBSERVED stderr:**
  `main.kf:5:5: error: assignment is a statement, not an expression (use '=' on its own line) [SEM027]`
* **VERIFIED:** assignment is a **statement**, not an expression. Chained
  assignment is a compile error with a dedicated diagnostic code.
* **Consequence:** S11 Q5 ("is assignment right-associative?") is **not
  applicable**. There is no associativity to determine. This is a better outcome
  than an answer: the question was an artefact of assuming expression
  assignment.

### 2.6 Scoping and shadowing

```kf
main() {
    var x = 1
    {
        var x = 2
        println(x)
    }
    println(x)
}
```

* exit `0`
* **OBSERVED stdout:** `2`, `1`
* **VERIFIED:** blocks are **scoped**, and an inner declaration **shadows** an
  outer one without error. The outer binding is unaffected after the block.

### 2.7 Nested-function capture — the finding with the widest consequences

```kf
main() {
    var n = 10
    Int add(Int k) { return n + k }
    println(add(5))
}
```

* exit `1`
* **OBSERVED stderr:**
  `main.kf:3:29: error: Undefined variable or type: 'n' [SEM011]`
* **VERIFIED: a nested function does NOT capture enclosing locals.** The nested
  function's body cannot see the enclosing local `n` at all — it is reported as
  an *undefined variable*, which is the same diagnostic a truly undefined name
  produces.
* **Consequence for the memory strategy (S16 / ADR-0005):** the argument that
  "escaping by-reference capture forces heap allocation" **does not apply** to
  this language as evidenced. Nested functions are non-capturing local
  declarations, so there is no closure environment to allocate. This
  substantially narrows the ADR-0005 study.
* **NOT DISTINGUISHED:** whether a nested function *could* capture if it were
  permitted to escape, or what the intended semantics were. The probe shows
  only that capture is **rejected today**. Whether that is the designed rule or
  an unimplemented feature is not established, and the record does not claim it.

### 2.8 S11 summary

| Q | Question | Result |
|---|---|---|
| Q1 | Argument order | **VERIFIED** left to right |
| Q2 | Receiver versus arguments | **VERIFIED** receiver first |
| Q3 | Binary operand order | **VERIFIED** left to right |
| Q4 | Short-circuit | **VERIFIED** both operators |
| Q5 | Assignment associativity | **VERIFIED** — not applicable; assignment is a statement (`SEM027`) |
| Q6 | Scoping / shadowing | **VERIFIED** block-scoped, shadowing allowed |
| Q7 | Nested-function capture | **VERIFIED** rejected (`SEM011`); *intent* NOT DISTINGUISHED |

**S11 now has VERIFIED LEGACY BEHAVIOR for the sampled JVM cases.** The
corresponding Cosmic-Lang rules remain PROPOSED NORMATIVE RULE, and Q7's intent
and untested cases remain OPEN. The measurement is sufficient for a scoped
parser gate analysis, not a universal semantic contract.

---

## 3. S12 — equality semantics

```kf
record Point(Int x, Int y)
class C {
    Int v
    public constructor(Int v) { this.v = v }
}
main() {
    var s1 = "ab"
    var s2 = "a" + "b"
    println(s1 == s2)
    var p1 = Point(1, 2)
    var p2 = Point(1, 2)
    println(p1 == p2)
    var c1 = C(1)
    var c2 = C(1)
    println(c1 == c2)
    println(c1 == c1)
}
```

* exit `0`
* **OBSERVED stdout:** `true`, `true`, `false`, `true`
* **VERIFIED:**
  * **strings: value equality** — `"ab" == "a" + "b"` is `true`, so two
    independently constructed strings with equal content compare equal;
  * **records: component-wise structural equality** — two distinct `Point(1,2)`
    instances compare equal;
  * **classes: reference equality** — two distinct `C(1)` instances compare
    `false`, while the same instance compares `true`.

### 3.1 Floating-point corner cases

```kf
main() {
    var z = 0.0
    var n = z / z
    println(n)
    println(n == n)
    var neg = -0.0
    println(neg == 0.0)
    println(1.0 / neg)
}
```

* exit `0`
* **OBSERVED stdout:** `NaN`, `false`, `true`, `-Infinity`
* **VERIFIED:**
  * `0.0 / 0.0` yields `NaN` and **`NaN == NaN` is `false`** — IEEE-754
    behaviour, not a total-equality reading;
  * **`-0.0 == 0.0` is `true`**, and negative zero is nevertheless **preserved**
    as a value (`1.0 / -0.0` is `-Infinity`, so the sign bit survived);
  * the printed spellings are **`NaN`** and **`-Infinity`** — capital `I`, and
    the sign is printed. This extends the formatting contract recorded in
    `docs/spec/decisions/S7-numeric-semantics.md`.

### 3.2 S12 summary

| Q | Question | Result |
|---|---|---|
| Q1 | Reference vs structural per type kind | **VERIFIED** — strings value, records component-wise, classes reference |
| Q4 | String equality | **VERIFIED** value equality |
| Q5 | `NaN` / `-0.0` | **VERIFIED** IEEE-754 (`NaN != NaN`, `-0.0 == 0.0`, sign preserved) |
| Q6 | Records and enums | **VERIFIED** records component-wise; enums not probed |
| Q2/Q3 | Other operators; `equals`/`hashCode` | Unchanged — no evidence of such members exists |
| Q7 | Map-key / set contract | **Still open** — not probed |

**S12 can move from OPEN to PROPOSED**, with Q7 (map-key/set contract) and
enums explicitly recorded as not probed.

---

## 4. S8 — string and escape semantics

### 4.1 Unknown escape — confirmed silent

```kf
main() { println("a\qb") }
```

* exit `0`
* **OBSERVED stdout:** `aqb`
* **OBSERVED stderr:** *(empty)*
* **VERIFIED:** an unrecognised escape `\q` is **silently dropped**: the string
  becomes `aqb`, compilation succeeds, and **no diagnostic is emitted**. This
  empirically confirms the source-level finding and upgrades it from E1-by-
  inspection to E1-by-execution.
* **Assessment:** a **silent reinterpretation of the program text**. It is the
  same defect class as the historical silent folding of `"""…"""` to an empty
  string that the legacy project itself fixed (which is why `LEX008` exists).
  The proposed rule for the new toolchain is that an unrecognised escape is an
  **error** (S8 Q2, PROPOSED).

### 4.2 Triple-quoted string — confirmed rejected

```kf
main() { println("""hello""") }
```

* exit `1`
* **OBSERVED stderr:**
  `main.kf:1:18: error: Kof has no triple-quoted (raw/multiline) string literal: use "..." with \n (no interpolation either — concatenate with +) [LEX008]`
* **VERIFIED:** triple-quoted strings are **rejected** with a dedicated code.
  This is the empirical confirmation of the correction made in Phase 1 to
  `docs/spec/compatibility-matrix.md` §3 item 8.

### 4.3 Raw newline inside a string — permitted

```kf
main() {
    println("line1
line2")
}
```

* exit `0`
* **OBSERVED stdout:** `line1`, `line2`
* **VERIFIED:** a **raw newline inside a `"…"` literal is accepted** and becomes
  part of the string value. S8 Q4 is therefore resolved: multi-line string
  literals are permitted, they are simply not spelled with `"""`.
* **Interpretation:** this is *coherent*, not an oversight — the language
  rejects the `"""` **syntax** while allowing the **capability** via a raw
  newline. The two facts together explain the `LEX008` message, which directs
  the user to `"...\n..."`.

### 4.4 S8 summary

| Q | Question | Result |
|---|---|---|
| Q1 | Triple-quoted / raw / multiline literals | **VERIFIED** — `"""` rejected (`LEX008`); raw newline **permitted** |
| Q2 | Silent unknown escape | **VERIFIED** — silent, no diagnostic (defect) |
| Q3 | Interpolation | Unchanged — stated absent by the `LEX008` message |
| Q4 | Raw newline | **VERIFIED** permitted |
| Q5/Q6 | Escape set; concatenation/equality | Escape set unchanged; string equality resolved under S12 |

---

## 5. S13 — Unicode behaviour

### 5.1 Non-ASCII identifiers are valid

```kf
Int café(Int x) = x + 1
main() { println(café(41)) }
```

* exit `0`, **OBSERVED stdout:** `42`
* **VERIFIED:** a non-ASCII **letter** is a valid identifier character. This
  resolves S13 Q1's non-ASCII half, which the golden corpus did not exercise.

### 5.2 `_` and `$` identifier starts

```kf
Int _under(Int x) = x + 1
main() { println(_under(1)) }        // exit 0, stdout: 2
```

```kf
Int $dollar(Int x) = x + 1
main() { println($dollar(1)) }       // exit 0, stdout: 2
```

* **VERIFIED:** both `_`-leading and `$`-leading identifiers are accepted, and
  both are usable as function names.

### 5.3 Diagnostic columns count UTF-16 code units — measured

```kf
main() { println("😀" +) }
```

* exit `1`
* **OBSERVED stderr:**
  ```
  main.kf:1:24: error: Unexpected token in expression: ) [PARSE041]
  main.kf:1:26: error: Expected ')' [PARSE043]
  ```

**Column arithmetic.** The line is `main() { println("😀" +) }`. Counting in
**UTF-16 code units** (the emoji occupies two):

| Unit | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Char | `m` | `a` | `i` | `n` | `(` | `)` | ` ` | `{` | ` ` | `p` | `r` | `i` | `n` | `t` | `l` | `n` | `(` | `"` | 😀 | 😀 | `"` | ` ` | `+` | `)` |

The reported column **24** is exactly where `)` sits **in code units**. Counting
in **Unicode scalar values** the same `)` would be at column **23**.

* **VERIFIED:** the legacy compiler reports **UTF-16 code-unit columns**, and
  the divergence from scalar-value counting is **exactly +1 per astral character
  preceding the position on that line**.
* **This is the first *quantified* measurement of the S13 Q4 divergence.** The
  Phase 1 record asserted it from source; this record measures it.
* **PROPOSED:** the new toolchain counts **scalar values** (unchanged from the
  Phase 1 proposal), and the differential harness must normalise diagnostic
  columns on any line containing an astral character.
* **Incidental observation:** this run emitted **two** diagnostics
  (`PARSE041`, `PARSE043`) for one malformed expression — evidence for
  multi-error recovery, recorded under S15 below.

### 5.4 S13 summary

| Q | Question | Result |
|---|---|---|
| Q1 | Identifier character class | **VERIFIED** for non-ASCII letters, `_` and `$`; digit-leading and other classes not probed |
| Q2 | Normalisation | Unchanged — no normalisation exists in the lexer |
| Q4 | Column counting | **VERIFIED** code units, divergence quantified as +1 per astral character |
| Q3/Q5/Q6 | Confusables, surrogates, encoding | Unchanged — not probed |

---

## 6. S1 — generics and overload resolution

### 6.1 Nested generics — the defect reproduced

```kf
class Box<T>(T value) {
    get(): T { return value }
}
main() {
    var bb: Box<Box<Int>> = Box(Box(7))
    println(bb.get().get())
}
```

* **OBSERVED:** the program **compiles successfully**, then **crashes at
  runtime**:
  ```
  Exception in thread "main" java.lang.NoSuchMethodError: 'int Box.get()'
      at Default.Main.main(main.kf:6)
  ```
* exit `1`
* **VERIFIED (DEFECT):** the compiler accepts `Box<Box<Int>>` and generates a
  call to `int Box.get()` — an erasure that does not exist. This is the legacy
  ledger item `cosmcc:docs/status.md` §706, **now reproduced independently**.
* **Not a language rule.** There is no rule to read off: the program is
  well-formed and the compiler mis-compiles it. Per Rule 1, this is recorded as
  a **defect**, and the new toolchain must define the *correct* resolution rule
  (S1 Q1), not reproduce this.

### 6.2 The boundary of the defect — inference inside a generic call

```kf
class Box<T>(T value) {
    get(): T { return value }
}
main() {
    var a: Box<List<Int>> = Box(listOf(1, 2))
    println(a.get().size)
    var b: List<Box<Int>> = listOf(Box(3))
    println(b.get(0).get())
    var c: Box<Box<Int>> = Box(Box(7))
    println(c.get().get())
}
```

* exit `1`
* **OBSERVED stderr:**
  `main.kf:7:5: error: type mismatch: cannot assign List<Box> to 'b: List<Box<Int>>' [SEM021]`
* **VERIFIED (DEFECT), and this is the sharper finding:** the diagnostic renders
  the inferred type as **`List<Box>`** — the **nested type argument was dropped
  during inference**. The construction `Box(3)` nested inside the generic call
  `listOf(...)` was inferred as a **raw** `Box` rather than `Box<Int>`, so the
  assignment to `List<Box<Int>>` failed.
* Note the contrast: the standalone form `var b: Box<Int> = Box(42)` works in
  the legacy golden corpus. So the defect is specifically that **the expected
  type does not propagate into arguments of an enclosing generic call**.
* **Interpretation:** this is a *type-inference* defect, distinct from the
  erasure defect in 6.1, and it is narrower and more reproducible than the
  ledger item describes.

### 6.3 `<` and `>` — resolved as a comparison chain, not generics

```kf
main() {
    println(true > 0)
    println(true > 1)
    println(1 < 2 > 0)
    println(3 < 2 > 0)
    println(1 < 2 > 1)
}
```

* exit `0`
* **OBSERVED stdout:** `true`, `false`, `true`, `false`, `false`

**Why `1 < 2 > 0` is decisive.** Two readings are possible:

| Reading | Evaluation | Result |
|---|---|---|
| **Left-associative** | `(1 < 2) > 0` = `true > 0` = `1 > 0` | **true** |
| Right-associative | `1 < (2 > 0)` = `1 < true` = `1 < 1` | false |

The observed **`true`** matches the **left-associative** reading. The
corroborating cases agree: `3 < 2 > 0` → `(3<2)>0` = `false>0` = `0>0` = `false`
✓, and `1 < 2 > 1` → `(1<2)>1` = `true>1` = `1>1` = `false` ✓.

* **VERIFIED:** `a < b > c` is parsed as a **left-associative comparison
  chain**, **not** as a generic type-argument list. The classic
  generics-versus-comparison ambiguity is resolved by context: a generic
  application requires a *type* on the left, and `a` is a value.
* **Consequence for the grammar:** the `type-arguments` production in
  `docs/spec/grammar.ebnf` must be reachable **only** in type position and in
  the explicitly-evidenced generic-call position, never as a fallback for a
  `<` in expression position. This is a grammar constraint derived from
  experiment, and it is recorded in the consistency report.

### 6.4 Overload resolution — specificity works, ambiguity is silent

**Specificity.** `m(Int)` versus `m(Long)` called with `1`:

* exit `0`, **OBSERVED stdout:** `int` → the **most specific** applicable
  overload is chosen.

**Ambiguity.** A class implementing two interfaces, with one overload per
interface:

```kf
interface A { a(): String }
interface B { b(): String }
class C implements A, B {
    a(): String = "a"
    b(): String = "b"
}
class O {
    public constructor() {}
    m(A x): String = "A"
    m(B x): String = "B"
}
main() {
    var c = C()
    println(O().m(c))
}
```

* exit `0`, **OBSERVED stdout:** `A`
* **VERIFIED (and this is a significant finding):** where the argument is
  **equally applicable to two unrelated parameter types**, the compiler does
  **not** report an ambiguity. It silently selects the **first declared**
  overload.
* **Consequence:** the "declaration order is the tie-break" behaviour that
  `docs/spec/typing-contracts.md` §6 *predicted* is now **confirmed by
  execution**. The Phase 1 proposal — "ambiguity is an error, never a
  declaration-order tie-break" — is therefore a **C2 divergence**, and the
  record now states that with evidence rather than as a guess.
* **NOT DISTINGUISHED:** whether the tie-break is *strictly* declaration order
  in all cases (only one ordering was probed), and whether a reversed
  declaration order would select `B`. The record does **not** claim the general
  rule; it claims the observed instance.

### 6.5 Incidental: Bool is comparable with Int but not addable

```kf
main() {
    println(1 + true)
    println(true + true)
    var b: Bool = true
    println(b > 0)
}
```

* exit `1`
* **OBSERVED stderr:**
  `main.kf:3:24: error: Cannot apply '+' to boolean types. Use == or != for comparison. [SEM002]`
* Combined with §6.3 (`true > 0` is `true`), this shows the legacy compiler
  **permits** Bool↔Int comparison but **rejects** Bool arithmetic.
* **Assessment (inconsistency, not a rule):** the diagnostic's own text says to
  use `==` or `!=` "for comparison", which contradicts the observed acceptance
  of `true > 0`. Recorded as an inconsistency for S7 to decide explicitly.

### 6.6 S1 summary

| Q | Question | Result |
|---|---|---|
| Q1 | Nested type-argument resolution | **VERIFIED as a DEFECT** — `Box<Box<Int>>` mis-compiles; inference drops a nested argument inside a generic call |
| Q5 | Overload applicability/specificity | **PARTIAL** — specificity verified; ambiguity silently resolved by declaration order (one instance) |
| — | `<`/`>` disambiguation | **VERIFIED** left-associative comparison chain |
| Q2/Q3/Q4 | Generic inference, bounds, variance | **NOT DISTINGUISHED** |

---

## 7. S15 — error recovery, exit codes and diagnostic quality (incidental)

Not a priority in the directive, but the probes produced evidence worth keeping.

### 7.1 Multi-error recovery works

```kf
Int f(Int x) = x + 1
main() {
    println(f())
    println(undefined1)
    println(undefined2)
}
```

```sh
java -jar $JAR check probes/S15-multierror2/main.kf
```

* **OBSERVED stdout:** three independent diagnostics in one run —
  ```
  :0:0: error: Wrong number of arguments for 'f': expected 1 but got 0 [SEM013]
  main.kf:4:13: error: Undefined variable or type: 'undefined1' [SEM011]
  main.kf:5:13: error: Undefined variable or type: 'undefined2' [SEM011]
  ```
* **exit code: `1`**
* **VERIFIED:** multi-error recovery is **real** (three independent errors from
  one run) and `check` exits **non-zero** on errors. The design report's page 5
  criterion ("recover multiple errors") is met in this case.

### 7.2 A diagnostic with no source position

The **first** diagnostic above carries the position **`:0:0`** — no file, no
line, no column. A machine consumer cannot locate it.

* **VERIFIED (DEFECT):** at least one diagnostic family is emitted without a
  source position.
* **Relevance:** this is precisely the class of defect Phase 0's
  `cosmic-diagnostics` prevents by making the **primary label with a validated
  span mandatory**. Recorded as supporting evidence for that design decision.

### 7.3 Recovery is not uniform

```kf
main() {
    var x = 
    var y = 
    println(undefinedName)
}
```

* **OBSERVED stdout:** a **single** diagnostic —
  `main.kf:3:5: error: Unexpected token in expression: var [PARSE041]`
* **exit code: `1`**
* **VERIFIED:** a run of three malformed statements produced **one** error, not
  three. So recovery quality **depends on the construct**.
* **NOT DISTINGUISHED:** whether a longer or differently-shaped input recovers
  better. The record claims only that multi-error recovery is **not uniform**,
  and that the design report's criterion is therefore **partially** met rather
  than met.

### 7.4 The diagnostic stream depends on the subcommand

| Subcommand | Stream | Evidence |
|---|---|---|
| `kof check` | **stdout** | §7.1 and §7.3: with stderr redirected to `/dev/null`, the full diagnostic set is still printed; with stdout redirected, stderr is empty |
| `kof run` | **stderr** | §2.5, §4.2, §5.3, §6.1–6.2: the diagnostics and the runtime exception appear on stderr with stdout empty |

* **VERIFIED:** the legacy CLI is **internally inconsistent** about where it
  writes diagnostics.
* **Relevance:** for a machine consumer this matters — a harness that captures
  only stderr sees *nothing* from `check`. The new `cosmic check` should fix the
  stream contract deliberately (one stated stream, machine-readable mode
  separate), which is the same class of correction as the mandatory span in
  §7.2.

---

## 8. S2 — bonus probe: the "soft cast" is defined

The Phase 1 record left S2 Q2 open ("what is a soft cast?"). One probe settles it.

```kf
class Animal { }
class Dog extends Animal { }
main() {
    var a: Animal = Animal()
    var s = a as Dog?
    println(s == null)
    var d: Animal = Dog()
    println((d as Dog?) != null)
    var o: Object = 42
    println((o as String?) == null)
}
```

* exit `0`
* **OBSERVED stdout:** `true`, `true`, `true`
* **VERIFIED:** `expr as T?` is a **soft cast**: it yields **`null`** when the
  runtime type is not `T`, instead of throwing. A failed cast to a nullable type
  is therefore an ordinary null result, not an exception.
* **Consequence:** S2 Q2 moves from OPEN to **VERIFIED LEGACY BEHAVIOUR**, and
  the term "soft cast" in the compatibility matrix now has a definition.
* This also corroborates the design report's page 7 point that `as` takes a
  **type** on the right and that the `?` suffix participates in the cast.

---

## 9. Cross-cutting results

### 9.1 Questions moved out of the blocked state

| Record | Was | Now | Basis |
|---|---|---|---|
| S11 | BLOCKED-ON-MEASUREMENT (historical) | **VERIFIED LEGACY BEHAVIOR** on sampled JVM forms; **OPEN** capture intent | §2 — partial evidence; no acceptance |
| S12 | OPEN | **VERIFIED LEGACY BEHAVIOR** for four type kinds; **OPEN / NOT VERIFIED** for enum and collections | §3 |
| S13 | PROPOSED (historical) | **VERIFIED LEGACY BEHAVIOR** on samples; **PROPOSED NORMATIVE RULE** scalar columns | §5 |
| S8 | OPEN | **VERIFIED LEGACY BEHAVIOR** on three string cases; **PROPOSED NORMATIVE RULE** strict unknown escapes | §4 |
| S2 | OPEN | **OPEN** with Q2 VERIFIED LEGACY BEHAVIOR | §8 |
| S1 | OPEN / BLOCKED-ON-MEASUREMENT (historical) | **OPEN** with measured defects and partial overload evidence | §6 |

**These probes alone do not approve G1b's replacement.** They demonstrate that
S1/S11 measurements are not prerequisites to lexical/syntactic recognition.
Only an explicit owner acceptance of the component-scoped gates in
`docs/spec/implementation-gates.md` can resolve the Phase 2 entry cycle.

### 9.2 Defects confirmed (not rules)

1. `Box<Box<Int>>` mis-compiles → `NoSuchMethodError` at runtime (§6.1).
2. Type inference drops a nested type argument inside an enclosing generic call
   (§6.2).
3. An overload ambiguity selected the first declaration in one ordering; the
   general tie-break is NOT VERIFIED (§6.4).
4. An unknown string escape is silently dropped with no diagnostic (§4.1).
5. Bool↔Int comparison is allowed while Bool arithmetic is rejected (§6.5).
6. At least one diagnostic is emitted with no source position `:0:0` (§7.2).
7. Multi-error recovery is not uniform (§7.3).
8. The diagnostic stream differs between subcommands: `check` uses stdout,
   `run` uses stderr (§7.4).

### 9.3 Limitations of this record

* **One target.** All probes ran on `--target jvm`. Cross-target divergence is
  **not** covered. The Phase 0 finding that the golden corpus passed while
  cross-target shift semantics diverged means this record cannot be used to
  claim cross-target behaviour.
* **One ordering probed** for the overload tie-break (§6.4).
* **Enums not probed** for equality (S12 Q6), and the map-key/set contract
  (S12 Q7) not probed.
* **Q7's intent** for nested functions is not established (§2.7).
* **Single-execution evidence, independently reproduced once.** Each probe was
  executed twice (original run and the reproduction of §0.1 step 5) with
  identical results, which is a reproducibility check, not a determinism proof,
  and not a repetition budget for statistical claims.
* **No legacy fixture was copied or committed.** All probe programs were written
  for this record; they live only in `/tmp/cosmic-probe/probes` and are recorded
  here as text because they are the experiment's input, which is required to
  reproduce it. They are **not** vendored into the repository as fixtures.
