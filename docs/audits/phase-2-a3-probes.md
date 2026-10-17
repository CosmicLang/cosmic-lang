# Lexical probe measurements (Increment A3)

- **Schema:** `cosmic-diff-harness/probes/1`
- **Cases:** 34 (passed 34, failed 0)
- **Oracle cross-check:** NOT MEASURED: no legacy oracle was configured in this environment; these probes measure the accepted Rust classification authority and make no legacy-compatibility claim

Regenerated with `cargo run -p cosmic-diff-harness -- --emit-probes <path>`.
The expectations are pinned by the `cosmic-diff-harness` probe test, so this table
is a measurement of the accepted frontend, not a snapshot.

| id | group | input | observed | expected | result | evidence |
|---|---|---|---|---|---|---|
| `m7_ascii_word` | identifier-classification (M-7) | camelCase | identifier:camelCase | identifier:camelCase | PASS | docs/spec/g1-l-lexical-decision-package.md D-L3 |
| `m7_underscore` | identifier-classification (M-7) | _ | identifier:_ | identifier:_ | PASS | docs/spec/g1-l-lexical-decision-package.md D-L3 |
| `m7_dollar` | identifier-classification (M-7) | $ | identifier:$ | identifier:$ | PASS | docs/spec/g1-l-lexical-decision-package.md D-L3 |
| `m7_digits_continue` | identifier-classification (M-7) | a1b | identifier:a1b | identifier:a1b | PASS | docs/spec/g1-l-lexical-decision-package.md D-L3 |
| `m7_digit_start` | identifier-classification (M-7) | 1abc | integer literal:1 identifier:abc | integer literal:1 identifier:abc | PASS | M-7 digit-leading names; longest-valid-prefix numeric recognition |
| `m7_precomposed_letter` | identifier-classification (M-7) | café | identifier:café | identifier:café | PASS | docs/spec/g1-l-lexical-decision-package.md D-L3 |
| `m7_non_ascii_letters` | identifier-classification (M-7) | Ωmega | identifier:Ωmega | identifier:Ωmega | PASS | docs/spec/g1-l-lexical-decision-package.md D-L3 |
| `m7_non_bmp_letter` | identifier-classification (M-7) | 𐐀𐐨 | identifier:𐐀𐐨 | identifier:𐐀𐐨 | PASS | M-7 non-BMP letters (Deseret) |
| `m7_combining_mark_continue` | identifier-classification (M-7) | é | identifier:e error:́[LEX005] | identifier:e error:́[LEX005] | PASS | M-7 combining marks; measured boundary of the accepted authority |
| `m7_leading_combining` | identifier-classification (M-7) | ́name | error:́[LEX005] identifier:name | error:́[LEX005] identifier:name | PASS | M-7 combining marks in start position |
| `m7_emoji` | identifier-classification (M-7) | 😀 | error:😀[LEX005] | error:😀[LEX005] | PASS | M-7 Unicode edge categories (symbols are not identifiers) |
| `m7_normalization_not_applied` | identifier-classification (M-7) | café café | identifier:café ws identifier:cafe error:́[LEX005] | identifier:café ws identifier:cafe error:́[LEX005] | PASS | M-7 normalization-sensitive spellings stay distinct; no normalization |
| `esc_unknown` | escape-contracts (D-L5/D-L8/D-L9) | "a\\qb" | string literal:"a\\qb"[LEX008] | string literal:"a\\qb"[LEX008] | PASS | docs/spec/g1-l-lexical-decision-package.md D-L8 (strict; declared divergence) |
| `esc_unicode_escape_valid` | escape-contracts (D-L5/D-L8/D-L9) | "\\u0041" | string literal:"\\u0041" | string literal:"\\u0041" | PASS | docs/spec/g1-l-lexical-decision-package.md D-L5 (four-hex-digit form) |
| `esc_lone_high_string` | escape-contracts (D-L5/D-L8/D-L9) | "\\uD800" | string literal:"\\uD800"[LEX010] | string literal:"\\uD800"[LEX010] | PASS | docs/spec/g1-l-lexical-decision-package.md D-L9 |
| `esc_lone_low_string` | escape-contracts (D-L5/D-L8/D-L9) | "\\uDC00" | string literal:"\\uDC00"[LEX010] | string literal:"\\uDC00"[LEX010] | PASS | docs/spec/g1-l-lexical-decision-package.md D-L9 |
| `esc_pair_string` | escape-contracts (D-L5/D-L8/D-L9) | "\\uD83D\\uDE00" | string literal:"\\uD83D\\uDE00" | string literal:"\\uD83D\\uDE00" | PASS | docs/spec/g1-l-lexical-decision-package.md D-L9 (pair decodes to one scalar) |
| `esc_lone_high_char` | escape-contracts (D-L5/D-L8/D-L9) | '\\uD800' | character literal:'\\uD800'[LEX010] | character literal:'\\uD800'[LEX010] | PASS | docs/spec/g1-l-lexical-decision-package.md D-L9 (each half alone) |
| `esc_pair_char` | escape-contracts (D-L5/D-L8/D-L9) | '\\uD83D\\uDE00' | character literal:'\\uD83D\\uDE00'[LEX004] | character literal:'\\uD83D\\uDE00'[LEX004] | PASS | docs/spec/g1-l-lexical-decision-package.md D-L5 item 4 (char literal is exactly one char-value) |
| `esc_empty_char` | escape-contracts (D-L5/D-L8/D-L9) | '' | error:''[LEX003] | error:''[LEX003] | PASS | LEX003 empty character literal |
| `div_line_comment_cr` | other-known-divergences | x // note\ry | identifier:x ws line-comment eol:Cr identifier:y | identifier:x ws line-comment eol:Cr identifier:y | PASS | docs/spec/g1-l-lexical-decision-package.md D-L4 (bare CR terminates) |
| `div_line_comment_crlf` | other-known-divergences | x // note\r\ny | identifier:x ws line-comment eol:CrLf identifier:y | identifier:x ws line-comment eol:CrLf identifier:y | PASS | docs/spec/g1-l-lexical-decision-package.md D-L4 |
| `div_triple_quote` | other-known-divergences | """x""" | error:"""x"""[LEX009] | error:"""x"""[LEX009] | PASS | docs/spec/g1-l-lexical-decision-package.md D-L6 (declared divergence) |
| `div_bom_prefix` | other-known-divergences | ﻿x | bom identifier:x | bom identifier:x | PASS | docs/spec/g1-l-lexical-decision-package.md D-L1 |
| `div_bom_midfile` | other-known-divergences | x﻿y | identifier:x error:﻿[LEX005] identifier:y | identifier:x error:﻿[LEX005] identifier:y | PASS | docs/spec/g1-l-lexical-decision-package.md D-L1 (only at byte 0) |
| `div_number_longest_prefix` | other-known-divergences | 0x1F3G | integer literal:0x1F3 identifier:G | integer literal:0x1F3 identifier:G | PASS | longest-valid-prefix numeric recognition (A1-R1 contract) |
| `div_number_long_int_suffix` | other-known-divergences | 10L | long literal:10L | long literal:10L | PASS | long suffix on integer forms (A1-R1 contract) |
| `div_number_double_long_suffix` | other-known-divergences | 1.0L | double literal:1.0 identifier:L | double literal:1.0 identifier:L | PASS | long suffix restricted to integer forms (A1-R1 contract) |
| `div_raw_crlf_string` | other-known-divergences | "a\r\nb" | string literal:"a\r\nb" | string literal:"a\r\nb" | PASS | docs/spec/g1-l-lexical-decision-package.md D-L7 (raw newlines are content; decoded-value question stays open) |
| `div_columns_scalar_values` | other-known-divergences | 😀x | line_col(x)=1:2 (1-based; scalar-value columns) | line_col(x)=1:2 (1-based; scalar-value columns) | PASS | docs/spec/g1-l-lexical-decision-package.md D-L2 (UTF-16 vs scalar-value convention) |
| `div_char_recovery_multi` | other-known-divergences | 'ab' | character literal:'ab'[LEX004] | character literal:'ab'[LEX004] | PASS | LEX004 multi-character literal recovery |
| `div_char_recovery_unterminated` | other-known-divergences | 'a | error:'a[LEX004] | error:'a[LEX004] | PASS | LEX004 unterminated literal recovery |
| `div_nbsp` | other-known-divergences | x y | identifier:x error: [LEX005] identifier:y | identifier:x error: [LEX005] identifier:y | PASS | docs/spec/g1-l-lexical-decision-package.md D-L4 (unexpected character) |
| `div_line_separator` | other-known-divergences | x y | identifier:x error: [LEX005] identifier:y | identifier:x error: [LEX005] identifier:y | PASS | docs/spec/g1-l-lexical-decision-package.md D-L4 (U+2028 is not a line terminator) |
