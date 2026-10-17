# M-6 cross-target context: identifier classification (Increment A3)

| | |
|---|---|
| **M-6 status** | **NOT MEASURED.** The M-6 instrument is re-running the probe record on `--target js` and `--target script` of the legacy compiler (`docs/spec/implementation-gates.md` §6). No legacy compiler is available in this environment and vendoring or downloading one is prohibited, so the instrument cannot be executed. |
| **This document** | Cross-**authority** context only: the accepted Rust classification authority versus the host runtimes that happened to be available (CPython, Node, JDK) for the M-7 fragment set. It is not a target run and no legacy-compatibility claim may rest on it. |
| **Rust authority** | `cosmic-lexer` via the A3 probe measurements (`docs/audits/phase-2-a3-probes.md`); classification labels pinned by the `cosmic-diff-harness` probe test. |

## Authorities and their rules

| runtime | rule used by this table | version |
|---|---|---|
| Rust (`cosmic-lexer`, accepted authority) | `char::is_alphabetic` at start; `char::is_alphanumeric` or `_`/`$` in continuation; everything else refused (D-L3) | workspace toolchain |
| CPython `str.isidentifier()` | XID_Start/XID_Continue per UAX #31 | 3.9.6 |
| Node IdentifierName | `^[\p{ID_Start}_$][\p{ID_Continue}\u200C\u200D_$]*$` with `u` flag (language rule, also computed without the `$`/`_` extras) | v26.12.0 |
| JDK `Character.isJavaIdentifierStart/Part` | Java identifier rules (include `$`, `_` and combining marks) | javac/java stub present but non-functional (no Java Runtime installed) — n/a |

## Fragment table

| fragment | accepted Rust authority | CPython | Node (IdentifierName) | Node (property-only) | JDK | divergence from Rust |
|---|---|---|---|---|---|---|
| `camelCase` (`m7_ascii_word`) | identifier (accepted) | pass | pass | pass | n/a | none |
| `_` (`m7_underscore`) | identifier (accepted) | pass | pass | refuse | n/a | none |
| `$` (`m7_dollar`) | identifier (accepted) | refuse | pass | refuse | n/a | CPython |
| `a1b` (`m7_digits_continue`) | identifier (accepted) | pass | pass | pass | n/a | none |
| `1abc` (`m7_digit_start`) | not one identifier (number `1` + identifier `abc`) | refuse | refuse | refuse | n/a | none |
| `caf\u00E9` (`m7_precomposed_letter`) | identifier (accepted) | pass | pass | pass | n/a | none |
| `\u03A9mega` (`m7_non_ascii_letters`) | identifier (accepted) | pass | pass | pass | n/a | none |
| `\U00010400\U00010428` (`m7_non_bmp_letter`) | identifier (accepted) | pass | pass | pass | n/a | none |
| `e\u0301` (`m7_combining_mark_continue`) | identifier + refused combining mark (LEX005) | pass | pass | pass | n/a | CPython, Node |
| `\u0301name` (`m7_leading_combining`) | refused combining mark + identifier (LEX005) | refuse | refuse | refuse | n/a | none |
| `\U0001F600` (`m7_emoji`) | refused (LEX005) | refuse | refuse | refuse | n/a | none |
| `caf\u00E9 cafe\u0301` (`m7_normalization_not_applied`) | identifier, whitespace, identifier, refused mark (LEX005) | refuse | refuse | refuse | n/a | none |

## Reading the table

- `pass`/`refuse` are whole-fragment classifications by each runtime's own rule; they are not statements about the legacy compiler's targets.
- Rows with a divergence are the interesting boundary cases: they show where the accepted Rust authority draws the identifier class differently from the available host runtimes. Any legacy-target re-run must be run with the real M-6 instrument before compatibility may be claimed.
- The `$` fragment is a known authority difference: it is an accepted identifier character in the Rust authority and in Java/JS language rules, but not in XID_Start/XID_Continue.

## Record fields

| field | value |
|---|---|
| `command` | `target/release/cosmic-diff-harness --emit-probes-json <tmp>`; CPython in-process `str.isidentifier`; `node classify.js <fragments>`; `javac Classify.java && java Classify <fragments>` |
| `caveats` | runtimes were whatever the host provides (versions in the authority table); missing runtimes are `n/a`, never guessed; fragment classification by a runtime is context, not the M-6 instrument |

Regenerated with `python3 tools/m6-cross-target.py` (stdlib only, offline).
