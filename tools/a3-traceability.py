#!/usr/bin/env python3
"""Generate the A3 production-family test traceability matrix.

The source of truth is the machine-checked grammar evidence map
(``docs/spec/grammar-evidence-map.md``; verified by
``tools/spec-consistency.py`` check G13). This generator maps every
production to a named family with independently authored test evidence in
this repository, verifies that every referenced integration test, unit test,
diff-harness corpus fixture and probe actually exists, and writes
``docs/audits/phase-2-a3-traceability.md``.

The generator exits non-zero when the map shape or an evidence reference
does not hold, so the generated document can never cite a test that does not
exist. No network access and no third-party packages are used.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MAP_PATH = ROOT / "docs" / "spec" / "grammar-evidence-map.md"
OUT_PATH = ROOT / "docs" / "audits" / "phase-2-a3-traceability.md"

ROW_RE = re.compile(r"^\| (\d+) \| `([^`]+)` \|")


def fail(message: str) -> "None":
    print(f"a3-traceability: {message}", file=sys.stderr)
    sys.exit(1)


def scan_test_fns(tests_dir: Path) -> set[str]:
    names: set[str] = set()
    if not tests_dir.is_dir():
        return names
    for path in sorted(tests_dir.glob("*.rs")):
        text = path.read_text(encoding="utf-8")
        names.update(re.findall(r"^fn ([a-z0-9_]+)", text, re.MULTILINE))
    return names


def scan_src_fns(src_root: Path) -> set[str]:
    names: set[str] = set()
    for path in sorted(src_root.glob("**/*.rs")):
        text = path.read_text(encoding="utf-8")
        names.update(re.findall(r"\bfn ([a-z0-9_]+)", text))
    return names


def parse_map() -> list[dict]:
    rows = []
    for line in MAP_PATH.read_text(encoding="utf-8").splitlines():
        match = ROW_RE.match(line)
        if not match:
            continue
        cells = [cell.strip() for cell in line.split("|")]
        # cells[0] and cells[-1] are empty (leading/trailing pipes).
        number = int(match.group(1))
        name = match.group(2)
        classification = cells[6]
        rows.append({"number": number, "name": name, "classification": classification})
    return rows


# Family definitions. ``numbers`` covers the full 1..=122 range exactly once
# across all families; ``scope`` must agree with every member's classification.
FAMILIES = [
    {
        "label": "F01 lexical root and trivia",
        "numbers": list(range(1, 9)),
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "lexer:dl1_bom_is_a_prefix_leaf_and_does_not_change_the_token_stream",
            "lexer:dl1_bom_only_file_is_one_prefix_leaf",
            "lexer:dl1_bom_after_the_prefix_is_unexpected",
            "lexer:dl1_empty_file_lexes_to_nothing",
            "lexer:dl2_crlf_lf_and_cr_are_distinct_terminators_and_one_break_each",
            "lexer:dl2_terminator_bytes_are_preserved_in_their_exact_form",
            "lexer:dl4_whitespace_run_is_a_single_trivia_element",
            "lexer:dl4_line_comment_excludes_every_terminator_form",
            "lexer:dl4_block_comments_do_not_nest",
            "lexer:elements_partition_the_file_without_gaps_or_overlaps",
            "syntax:trivia_and_bom_leaves_are_preserved_in_order",
            "corpus:lex_bom_prefix",
            "corpus:lex_line_comment_cr",
            "corpus:lex_line_comment_crlf",
            "probe:div_bom_prefix",
            "probe:div_bom_midfile",
            "probe:div_line_comment_cr",
            "probe:div_line_comment_crlf",
        ],
        "note": "Each family row is evidence at family granularity; see the granularity caveat below.",
    },
    {
        "label": "F02 identifier vocabulary",
        "numbers": list(range(9, 18)),
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "lexer:dl3_unicode_letters_are_identifier_parts",
            "lexer:dl3_underscore_and_dollar_are_identifier_characters",
            "lexer:dl3_digit_starts_a_number_not_an_identifier",
            "lexer:dl3_contextual_words_stay_identifiers",
            "lexer:keyword_like_prefixes_continue_as_identifiers",
            "lexer:keyword_tokens_and_reserved_invalid_words",
            "lexer:astral_letter_identifiers_are_single_tokens",
            "lexer:combining_marks_after_identifiers_are_unexpected_characters",
            "corpus:lex_identifier_ascii",
            "corpus:lex_identifier_underscore",
            "corpus:lex_identifier_dollar",
            "corpus:lex_identifier_leading_digit",
            "corpus:lex_identifier_unicode_letter",
            "corpus:lex_identifier_non_bmp_letter",
            "corpus:lex_identifier_combining_continue",
            "corpus:lex_identifier_leading_combining",
            "corpus:lex_identifier_emoji",
            "probe:m7_ascii_word",
            "probe:m7_underscore",
            "probe:m7_dollar",
            "probe:m7_digits_continue",
            "probe:m7_digit_start",
            "probe:m7_precomposed_letter",
            "probe:m7_non_ascii_letters",
            "probe:m7_non_bmp_letter",
            "probe:m7_combining_mark_continue",
            "probe:m7_leading_combining",
            "probe:m7_emoji",
            "probe:m7_normalization_not_applied",
        ],
        "note": "The M-7 boundary probes measure refusal boundaries (combining marks, emoji) as part of the vocabulary contract.",
    },
    {
        "label": "F03 reserved-invalid vocabulary",
        "numbers": [18],
        "scope": "rejected",
        "result": "verified",
        "evidence": [
            "lexer:keyword_tokens_and_reserved_invalid_words",
            "syntax:reserved_invalid_function_words_are_refused_with_the_specific_message",
            "syntax:reserved_invalid_words_in_expression_position_are_reported",
            "corpus:syn_reserved_invalid_function",
        ],
        "note": "Production 18 is the single REJECTED row of the source map; the rejection is verified by name-specific tests.",
    },
    {
        "label": "F04 numeric literals",
        "numbers": list(range(19, 29)),
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "lexer:numeric_literals_follow_the_grammar_suffix_rules",
            "lexer:long_suffix_is_restricted_to_integer_forms",
            "lexer:long_suffix_boundary_preserves_spans_and_source_bytes",
            "lexer:malformed_number_tails_lex_as_separate_valid_tokens",
            "corpus:lex_number_longest_prefix",
            "probe:div_number_longest_prefix",
            "probe:div_number_long_int_suffix",
            "probe:div_number_double_long_suffix",
        ],
        "note": "The long-suffix restriction is the A1-R1 correction; the double-literal spelling measured by probe div_number_double_long_suffix.",
    },
    {
        "label": "F05 string and char literals, escapes",
        "numbers": list(range(29, 35)),
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "lexer:dl5_every_accepted_escape_lexes_clean",
            "lexer:dl5_both_literal_grammars_recognise_the_same_escape_forms",
            "lexer:dl5_char_and_string_share_the_escape_set",
            "lexer:dl6_triple_quote_is_refused_as_one_error_region",
            "lexer:dl7_raw_line_breaks_are_string_content",
            "lexer:dl7_crlf_inside_a_literal_is_preserved_byte_for_byte",
            "lexer:dl8_literal_with_unknown_escape_is_preserved_as_an_error_marked_token",
            "lexer:dl9_surrogate_pair_is_one_clean_literal",
            "lexer:dl9_pairing_is_greedy_and_local",
            "lexer:lex009_triple_quoted_strings_are_refused",
            "lexer:lex010_lone_surrogate_escapes_are_errors",
            "corpus:lex_strict_escape",
            "corpus:lex_unicode_escape_valid",
            "corpus:lex_triple_quoted",
            "corpus:lex_lone_surrogate_string",
            "corpus:lex_lone_surrogate_char",
            "corpus:lex_surrogate_pair_string",
            "corpus:lex_raw_crlf_in_string",
            "corpus:lex_char_multi",
            "corpus:lex_char_unterminated",
            "probe:esc_unknown",
            "probe:esc_unicode_escape_valid",
            "probe:esc_lone_high_string",
            "probe:esc_lone_low_string",
            "probe:esc_pair_string",
            "probe:esc_lone_high_char",
            "probe:esc_pair_char",
            "probe:esc_empty_char",
            "probe:div_triple_quote",
            "probe:div_raw_crlf_string",
            "probe:div_char_recovery_multi",
            "probe:div_char_recovery_unterminated",
        ],
        "note": "Strict refusal of unknown escapes is an intentional divergence from the legacy silent-escape behaviour (D-L8); triple-quoted forms are refused (D-L6).",
    },
    {
        "label": "F06 boolean and null literals",
        "numbers": [35, 36],
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "src:keyword_kind",
            "corpus:syn_boolean_null_literals",
        ],
        "note": "Lexer keyword classification is unit-tested (src unit test keyword_kind); expression-position acceptance is pinned by the A3 corpus fixture.",
    },
    {
        "label": "F07 operators, delimiters, separators",
        "numbers": list(range(37, 40)),
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "lexer:operators_delimiters_and_separators_match_the_grammar_tables",
            "lexer:spaced_separators_are_not_whitespace",
            "syntax:operator_tokens",
            "syntax:split_shift_leaves_share_one_lexer_element",
            "syntax:shift_operators_rejoin_presplit_tokens",
        ],
        "note": "The shift rejoin is the A2 correction; the tables against the grammar remain the accepted authority.",
    },
    {
        "label": "F08 compilation unit and top-level declarations",
        "numbers": list(range(40, 46)),
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "syntax:package_and_imports_are_recognised",
            "syntax:empty_and_trivia_only_files_are_recognised",
            "syntax:bom_only_file_is_valid_and_preserved",
            "corpus:syn_switch_statement",
        ],
        "note": "Every syntax fixture is a whole compilation unit, so top-level structure is exercised by the entire syntax corpus.",
    },
    {
        "label": "F09 unsupported declaration families",
        "numbers": list(range(46, 52)),
        "scope": "refused by design",
        "result": "verified",
        "evidence": [
            "syntax:family1_test_declaration_is_refused",
            "syntax:family2_application_declaration_is_refused",
            "syntax:family3_infra_declaration_is_refused",
            "syntax:family4_extern_declaration_is_refused_and_neighbours_parse",
            "syntax:family5_foreign_module_declaration_is_refused",
            "syntax:every_refusal_is_exactly_one_parse001_and_keeps_lex_codes_untouched",
            "corpus:ref_test_family",
            "corpus:ref_application_family",
            "corpus:ref_infra_family",
            "corpus:ref_extern_family",
            "corpus:ref_foreign_module_family",
        ],
        "note": "Productions 46–51 are UNSUPPORTED in the source map; the refusal (PARSE001) is the accepted contract until recognition is accepted.",
    },
    {
        "label": "F10 function declarations and bodies",
        "numbers": list(range(52, 58)),
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "syntax:function_declarations_cover_both_parameter_orders_and_both_bodies",
            "syntax:parameter_defaults_are_recognised",
            "syntax:same_line_brace_after_a_call_is_a_nested_declaration",
            "syntax:next_line_brace_after_a_call_is_a_block_statement",
            "syntax:a_nested_function_body_does_not_attach_across_a_multiline_comment",
            "syntax:a_nested_function_body_attaches_across_an_inline_block_comment",
            "corpus:syn_nested_function_inline",
            "corpus:syn_nested_function_multiline",
        ],
        "note": "Nested-function attachment follows the A2-R1 physical-line rules.",
    },
    {
        "label": "F11 type references",
        "numbers": list(range(58, 63)),
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "syntax:nested_type_arguments_close_with_shared_shift_tokens",
            "syntax:generic_call_and_comparison_disambiguation",
            "syntax:deep_type_arguments_are_bounded",
            "corpus:syn_generic_call",
        ],
        "note": "Type-argument nesting is guarded by the PARSE005 implementation limit; deep chains are bounded in an isolated process.",
    },
    {
        "label": "F12 type declarations",
        "numbers": [63, 64, 65, 67, 68, 69, 70, 71, 72, 73],
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "syntax:type_declarations_cover_class_interface_record_enum_and_sealed",
        ],
        "note": "One family-level test covers class/interface/record/enum shapes including supertypes and sealed modifiers.",
    },
    {
        "label": "F13 class members, constructors, fields, modifiers",
        "numbers": [74, 75, 76, 77, 78],
        "scope": "accepted",
        "result": "partially verified",
        "evidence": [
            "syntax:type_declarations_cover_class_interface_record_enum_and_sealed",
            "syntax:family7_annotations_are_refused_at_top_level_and_in_members",
        ],
        "note": "No per-member assertions: members/constructors/fields/modifiers are exercised only inside family-level declaration tests.",
    },
    {
        "label": "F14 unsupported entity declaration",
        "numbers": [66],
        "scope": "refused by design",
        "result": "verified",
        "evidence": [
            "syntax:family6_entity_declaration_is_refused",
            "corpus:ref_entity_family",
        ],
        "note": "Production 66 is UNSUPPORTED in the source map (refused with the entity family).",
    },
    {
        "label": "F15 unsupported annotations",
        "numbers": [79, 80],
        "scope": "refused by design",
        "result": "verified",
        "evidence": [
            "syntax:family7_annotations_are_refused_at_top_level_and_in_members",
            "corpus:ref_annotation_family",
        ],
        "note": "Productions 79–80 are UNSUPPORTED in the source map.",
    },
    {
        "label": "F16 statements",
        "numbers": list(range(81, 104)),
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "syntax:statement_families_are_recognised",
            "syntax:zero_width_structural_nodes_contribute_no_text",
            "syntax:return_does_not_cross_lines",
            "syntax:assignment_does_not_cross_lines",
            "syntax:unlabelled_break_and_continue_are_recognised",
            "syntax:else_catch_finally_and_while_bind_across_lines",
            "syntax:switch_labels_cover_expression_binding_destructuring_and_type_forms",
            "syntax:stray_closers_are_reported_and_preserved",
            "syntax:pathological_if_lines_stay_bounded_deterministic_and_lossless",
            "syntax:adversarial_statement_soup_is_deterministic_and_lossless",
            "corpus:syn_switch_statement",
            "corpus:mal_unclosed_brace",
        ],
        "note": "Includes the A3 adversarial regressions for pathological statement sequences.",
    },
    {
        "label": "F17 expressions",
        "numbers": list(range(104, 116)) + [122],
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "syntax:expression_families_are_recognised",
            "syntax:addition_is_left_associative",
            "syntax:multiplication_binds_tighter_than_addition",
            "syntax:shift_operators_rejoin_presplit_tokens",
            "syntax:nested_type_arguments_close_with_shared_shift_tokens",
            "syntax:parenthesised_regions_lift_the_line_gate_across_a_multiline_comment",
            "corpus:syn_shift_expression",
            "corpus:syn_generic_call",
        ],
        "note": "Production 122 (new-expr) is exercised in the same primary-expression test matrix.",
    },
    {
        "label": "F18 unsupported array literal",
        "numbers": [116],
        "scope": "refused by design",
        "result": "verified",
        "evidence": [
            "syntax:family9_array_literal_brace_form_is_refused",
            "syntax:family9_arrow_arm_block_bodies_fall_to_the_array_refusal",
            "corpus:ref_array_literal_family",
        ],
        "note": "Production 116 is UNSUPPORTED in the source map; the brace form must not be broadened.",
    },
    {
        "label": "F19 lambdas",
        "numbers": [117, 118, 119],
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "syntax:lambda_forms_are_recognised",
            "syntax:trailing_lambda_attaches_only_on_the_same_line",
            "syntax:a_trailing_lambda_does_not_attach_across_a_multiline_comment",
            "syntax:a_trailing_lambda_attaches_across_an_inline_block_comment",
            "syntax:untyped_lambda_parameters_are_syntax_errors",
            "corpus:syn_trailing_lambda_inline",
            "corpus:syn_trailing_lambda_multiline",
            "corpus:syn_untyped_lambda",
        ],
        "note": "Untyped lambda parameters are a documented Stage A boundary (refused, not accepted).",
    },
    {
        "label": "F20 switch expressions",
        "numbers": [120, 121],
        "scope": "accepted",
        "result": "verified",
        "evidence": [
            "syntax:switch_labels_cover_expression_binding_destructuring_and_type_forms",
        ],
        "note": "The same labels test covers the expression position (`val j = switch (a) { ... }`).",
    },
]

RESULT_VOCABULARY = {
    "verified": "at least one direct, independently authored test exercises the family (or its refusal) and passes in the workspace suite",
    "partially verified": "the family is exercised only at family level, without per-production assertions",
    "failed": "an identified defect kept the construct from being verified",
    "intentionally divergent": "the accepted contract intentionally differs from the legacy behaviour",
    "not measured": "no independent repository test exists and the reason is recorded",
}


def main() -> int:
    rows = parse_map()
    if len(rows) != 122:
        fail(f"expected 122 rows in the evidence map, found {len(rows)}")
    numbers = [row["number"] for row in rows]
    if numbers != list(range(1, 123)):
        fail("evidence-map rows are not numbered 1..=122 in order")

    counts = {}
    for row in rows:
        counts[row["classification"]] = counts.get(row["classification"], 0) + 1
    expected_counts = {"RECOGNISED": 111, "UNSUPPORTED": 10, "REJECTED": 1}
    if counts != expected_counts:
        fail(f"classification counts changed: {counts} != {expected_counts}")

    # Evidence pools.
    lexer_tests = scan_test_fns(ROOT / "crates" / "cosmic-lexer" / "tests")
    syntax_tests = scan_test_fns(ROOT / "crates" / "cosmic-syntax" / "tests")
    src_fns = scan_src_fns(ROOT / "crates")
    corpus_text = (ROOT / "tools" / "diff-harness" / "src" / "corpus.rs").read_text(
        encoding="utf-8"
    )
    corpus_ids = set(re.findall(r'\n    \(\n        "([a-z0-9_]+)",', corpus_text))
    probes_text = (ROOT / "tools" / "diff-harness" / "src" / "probes.rs").read_text(
        encoding="utf-8"
    )
    probe_ids = set(re.findall(r'id: "([a-z0-9_]+)"', probes_text))
    pools = {
        "lexer": lexer_tests,
        "syntax": syntax_tests,
        "src": src_fns,
        "corpus": corpus_ids,
        "probe": probe_ids,
    }

    # Coverage and scope validation.
    covered: dict[int, str] = {}
    for family in FAMILIES:
        for number in family["numbers"]:
            if number in covered:
                fail(f"production {number} covered by two families")
            covered[number] = family["label"]
        scope = family["scope"]
        expected_class = {
            "accepted": "RECOGNISED",
            "refused by design": "UNSUPPORTED",
            "rejected": "REJECTED",
        }[scope]
        for number in family["numbers"]:
            row = rows[number - 1]
            if row["classification"] != expected_class:
                fail(
                    f"family {family['label']}: production {number} is "
                    f"{row['classification']}, expected {expected_class}"
                )
        if family["result"] not in RESULT_VOCABULARY:
            fail(f"family {family['label']}: unknown result {family['result']!r}")
        for reference in family["evidence"]:
            kind, _, name = reference.partition(":")
            if kind not in pools:
                fail(f"family {family['label']}: unknown evidence kind {kind!r}")
            if name not in pools[kind]:
                fail(
                    f"family {family['label']}: evidence {reference} does not exist in this repository"
                )
    missing = [number for number in range(1, 123) if number not in covered]
    if missing:
        fail(f"productions not covered by families: {missing}")

    # Render.
    out = []
    out.append("# Production-family test traceability (Increment A3)")
    out.append("")
    out.append("| | |")
    out.append("|---|---|")
    out.append(
        "| **Status** | Generated by `tools/a3-traceability.py`; every evidence "
        "reference is machine-checked to exist in this repository. |"
    )
    out.append(
        "| **Source of truth** | `docs/spec/grammar-evidence-map.md` — 122/122 "
        "productions in grammar definition order, machine-checked by "
        "`tools/spec-consistency.py` (check G13). |"
    )
    out.append(
        "| **Scope** | Increment A3 (Stage A): mapping independently authored "
        "tests to the accepted production families. It records measurement; it "
        "adds no acceptance beyond the G1-L/G1-S decision packages. |"
    )
    out.append("")
    out.append("## Classification of the source map")
    out.append("")
    out.append("| classification | productions |")
    out.append("|---|---|")
    for label in ("RECOGNISED", "UNSUPPORTED", "REJECTED"):
        out.append(f"| `{label}` | {counts[label]} |")
    out.append("")
    out.append("## Result vocabulary")
    out.append("")
    for label, description in RESULT_VOCABULARY.items():
        out.append(f"- **{label}** — {description}.")
    out.append(
        "- **refused by design** / **rejected by design** — the accepted contract "
        "is the refusal (PARSE001) or the rejection; the refusal/rejection itself "
        "is verified."
    )
    out.append("")
    out.append("## Family summary")
    out.append("")
    out.append("| family | productions | scope | result | evidence |")
    out.append("|---|---|---|---|---|")
    for family in FAMILIES:
        count = len(family["numbers"])
        evidence = "; ".join(f"`{reference}`" for reference in family["evidence"])
        out.append(
            f"| {family['label']} | {count} | {family['scope']} | "
            f"**{family['result']}** | {evidence} |"
        )
    out.append("")
    out.append("## Full matrix")
    out.append("")
    out.append("| # | production | class | family | result |")
    out.append("|---|---|---|---|---|")
    for row in rows:
        number = row["number"]
        family = next(f for f in FAMILIES if number in f["numbers"])
        out.append(
            f"| {number} | `{row['name']}` | `{row['classification']}` | "
            f"{family['label']} | **{family['result']}** |"
        )
    out.append("")
    out.append("## Notes")
    out.append("")
    out.append(
        "- **Granularity caveat.** Mapping is at family granularity. A row's "
        "result is the family result; families F06 and F13 are the only ones not "
        "holding a direct per-family integration test, and they are marked "
        "accordingly (F06 receives A3 corpus coverage, F13 stays partially "
        "verified)."
    )
    out.append(
        "- **Intentional divergences.** F05 covers the strict escape refusal "
        "(LEX008 / D-L8), the triple-quote refusal (LEX009 / D-L6) and the "
        "surrogate rules (LEX010 / D-L9); those divergences are declared in the "
        "decision packages and measured by the A3 probes."
    )
    out.append(
        "- **Refusals.** The ten UNSUPPORTED productions (F09, F14, F15, F18) "
        "must be refused (PARSE001) until recognition is separately accepted; "
        "the refusal tests listed above are the verification."
    )
    out.append(
        "- **No new acceptance.** This document introduces no acceptance: every "
        "recognised family was already accepted by the G1-L and G1-S decision "
        "packages and implemented in increments A1/A2."
    )
    out.append(
        "- **Regeneration.** `python3 tools/a3-traceability.py` (stdlib only, "
        "offline). The generator fails when the map shape, the classification "
        "counts, or any evidence reference is invalid."
    )
    out.append("")

    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUT_PATH.write_text("\n".join(out), encoding="utf-8")
    print(
        f"a3-traceability: wrote {OUT_PATH.relative_to(ROOT)} "
        f"({len(rows)} productions, {len(FAMILIES)} families, "
        f"{counts['RECOGNISED']}/{counts['UNSUPPORTED']}/{counts['REJECTED']})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
