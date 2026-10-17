//! Executed lexical probe measurements for the accepted contracts.
//!
//! These probes are the executable M-7 / escape-contract / known-divergence
//! measurements required by the A3 directive. Each case runs the accepted
//! Rust classification authority (the `cosmic-lexer` scanner) over one
//! authored fragment and records the observation. The expectations are pinned
//! by a unit test, so a behavioural change in the accepted contract fails the
//! suite instead of silently drifting.
//!
//! Every probe report states explicitly that the legacy-oracle cross-check is
//! **NOT MEASURED** unless an oracle is configured separately: these probes
//! measure the Rust side, not legacy compatibility.

use std::fmt::Write as _;

use cosmic_lexer::ElementKind;
use cosmic_source::SourceMap;

use crate::summary::{json_string, md_cell};

/// The JSON schema identifier of a probe report.
pub const PROBE_SCHEMA: &str = "cosmic-diff-harness/probes/1";

/// The oracle-status note printed in every probe report.
pub const ORACLE_NOTE: &str = "NOT MEASURED: no legacy oracle was configured in this environment; these probes measure the accepted Rust classification authority and make no legacy-compatibility claim";

/// One executable probe case.
#[derive(Clone, Debug)]
pub struct ProbeCase {
    /// Stable probe id.
    pub id: &'static str,
    /// Probe group.
    pub group: &'static str,
    /// The exact input fragment.
    pub input: &'static str,
    /// The expected observation.
    pub expected: &'static str,
    /// The decision record / accepted authority this probe evidences.
    pub evidence: &'static str,
}

/// One evaluated probe case.
#[derive(Clone, Debug)]
pub struct ProbeOutcome {
    /// Stable probe id.
    pub case_id: &'static str,
    /// Probe group.
    pub group: &'static str,
    /// Escaped input fragment.
    pub input: String,
    /// What the scanner produced.
    pub observed: String,
    /// What the accepted contract requires.
    pub expected: String,
    /// Whether `observed == expected`.
    pub pass: bool,
    /// The decision record / accepted authority.
    pub evidence: &'static str,
}

/// The complete probe measurement report.
#[derive(Clone, Debug)]
pub struct ProbeReport {
    /// Schema identifier (always [`PROBE_SCHEMA`]).
    pub schema: &'static str,
    /// The oracle cross-check note (always [`ORACLE_NOTE`]).
    pub oracle_note: &'static str,
    /// Evaluated outcomes, in case order.
    pub outcomes: Vec<ProbeOutcome>,
    /// Number of passing cases.
    pub passed: usize,
    /// Number of failing cases.
    pub failed: usize,
}

impl ProbeReport {
    /// The machine-readable probe report.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        let _ = writeln!(out, "  \"schema\": {},", json_string(self.schema));
        let _ = writeln!(out, "  \"oracle_note\": {},", json_string(self.oracle_note));
        let _ = writeln!(out, "  \"passed\": {},", self.passed);
        let _ = writeln!(out, "  \"failed\": {},", self.failed);
        out.push_str("  \"cases\": [");
        if self.outcomes.is_empty() {
            out.push_str("]\n}\n");
            return out;
        }
        out.push('\n');
        for (index, outcome) in self.outcomes.iter().enumerate() {
            out.push_str("    {\n");
            let _ = writeln!(out, "      \"id\": {},", json_string(outcome.case_id));
            let _ = writeln!(out, "      \"group\": {},", json_string(outcome.group));
            let _ = writeln!(out, "      \"input\": {},", json_string(&outcome.input));
            let _ = writeln!(
                out,
                "      \"observed\": {},",
                json_string(&outcome.observed)
            );
            let _ = writeln!(
                out,
                "      \"expected\": {},",
                json_string(&outcome.expected)
            );
            let _ = writeln!(out, "      \"pass\": {},", outcome.pass);
            let _ = writeln!(out, "      \"evidence\": {}", json_string(outcome.evidence));
            out.push_str("    }");
            if index + 1 < self.outcomes.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]\n}\n");
        out
    }

    /// The human-readable probe report.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Lexical probe measurements (Increment A3)\n\n");
        let _ = writeln!(out, "- **Schema:** `{}`", self.schema);
        let _ = writeln!(
            out,
            "- **Cases:** {} (passed {}, failed {})",
            self.outcomes.len(),
            self.passed,
            self.failed
        );
        let _ = writeln!(out, "- **Oracle cross-check:** {}", self.oracle_note);
        out.push_str(
            "\nRegenerated with `cargo run -p cosmic-diff-harness -- --emit-probes <path>`.\n\
             The expectations are pinned by the `cosmic-diff-harness` probe test, so this table\n\
             is a measurement of the accepted frontend, not a snapshot.\n\n",
        );
        out.push_str("| id | group | input | observed | expected | result | evidence |\n|---|---|---|---|---|---|---|\n");
        for outcome in &self.outcomes {
            let result = if outcome.pass { "PASS" } else { "FAIL" };
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} | {} | {} | {} |",
                outcome.case_id,
                md_cell(outcome.group),
                md_cell(&outcome.input),
                md_cell(&outcome.observed),
                md_cell(&outcome.expected),
                result,
                md_cell(outcome.evidence)
            );
        }
        out
    }
}

/// Evaluates every probe case and returns the report.
#[must_use]
pub fn probe_report() -> ProbeReport {
    let outcomes: Vec<ProbeOutcome> = CASES
        .iter()
        .map(|case| {
            let observed = observe(case);
            ProbeOutcome {
                case_id: case.id,
                group: case.group,
                input: escape_text(case.input),
                pass: observed == case.expected,
                observed,
                expected: case.expected.to_owned(),
                evidence: case.evidence,
            }
        })
        .collect();
    let passed = outcomes.iter().filter(|outcome| outcome.pass).count();
    let failed = outcomes.len() - passed;
    ProbeReport {
        schema: PROBE_SCHEMA,
        oracle_note: ORACLE_NOTE,
        outcomes,
        passed,
        failed,
    }
}

/// Runs the scanner over the case input and renders the observation.
fn observe(case: &ProbeCase) -> String {
    if case.id == "div_columns_scalar_values" {
        return observe_columns(case.input);
    }
    let mut sources = SourceMap::new();
    let Ok(id) = sources.add("probe.csmx", case.input) else {
        return "registration-failed".to_owned();
    };
    let Some(source) = sources.get(id) else {
        return "registration-failed".to_owned();
    };
    let lexed = cosmic_lexer::lex(source);
    let mut parts = Vec::new();
    for element in lexed.elements() {
        let text = source.text_of(element.span()).unwrap_or("");
        let mut part = match element.kind() {
            ElementKind::Bom => "bom".to_owned(),
            ElementKind::Whitespace => "ws".to_owned(),
            ElementKind::LineTerminator(kind) => format!("eol:{kind:?}"),
            ElementKind::LineComment => "line-comment".to_owned(),
            ElementKind::BlockComment => "block-comment".to_owned(),
            ElementKind::Token(kind) => format!("{}:{}", kind.name(), escape_text(text)),
            ElementKind::Error => format!("error:{}", escape_text(text)),
        };
        if !element.errors().is_empty() {
            let codes: Vec<&str> = element.errors().iter().map(|code| code.as_str()).collect();
            part.push('[');
            part.push_str(&codes.join(","));
            part.push(']');
        }
        parts.push(part);
    }
    if parts.is_empty() {
        return "(no elements)".to_owned();
    }
    parts.join(" ")
}

/// The column-convention probe: where the `x` of `😀x` lands.
fn observe_columns(input: &str) -> String {
    let Some(offset) = input.find('x') else {
        return "missing-x".to_owned();
    };
    let Ok(offset) = u32::try_from(offset) else {
        return "offset-overflow".to_owned();
    };
    let mut sources = SourceMap::new();
    let Ok(id) = sources.add("probe.csmx", input) else {
        return "registration-failed".to_owned();
    };
    let Some(source) = sources.get(id) else {
        return "registration-failed".to_owned();
    };
    let Some(position) = source.line_col(offset) else {
        return "no-position".to_owned();
    };
    format!(
        "line_col(x)={}:{} (1-based; scalar-value columns)",
        position.line, position.column
    )
}

/// Escapes a fragment for single-line display.
fn escape_text(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out
}

/// The probe cases, in report order.
const CASES: &[ProbeCase] = &[
    // --- identifier classification (M-7) ------------------------------------
    ProbeCase {
        id: "m7_ascii_word",
        group: "identifier-classification (M-7)",
        input: "camelCase",
        expected: "identifier:camelCase",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L3",
    },
    ProbeCase {
        id: "m7_underscore",
        group: "identifier-classification (M-7)",
        input: "_",
        expected: "identifier:_",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L3",
    },
    ProbeCase {
        id: "m7_dollar",
        group: "identifier-classification (M-7)",
        input: "$",
        expected: "identifier:$",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L3",
    },
    ProbeCase {
        id: "m7_digits_continue",
        group: "identifier-classification (M-7)",
        input: "a1b",
        expected: "identifier:a1b",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L3",
    },
    ProbeCase {
        id: "m7_digit_start",
        group: "identifier-classification (M-7)",
        input: "1abc",
        expected: "integer literal:1 identifier:abc",
        evidence: "M-7 digit-leading names; longest-valid-prefix numeric recognition",
    },
    ProbeCase {
        id: "m7_precomposed_letter",
        group: "identifier-classification (M-7)",
        input: "café",
        expected: "identifier:café",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L3",
    },
    ProbeCase {
        id: "m7_non_ascii_letters",
        group: "identifier-classification (M-7)",
        input: "Ωmega",
        expected: "identifier:Ωmega",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L3",
    },
    ProbeCase {
        id: "m7_non_bmp_letter",
        group: "identifier-classification (M-7)",
        input: "\u{10400}\u{10428}",
        expected: "identifier:\u{10400}\u{10428}",
        evidence: "M-7 non-BMP letters (Deseret)",
    },
    ProbeCase {
        id: "m7_combining_mark_continue",
        group: "identifier-classification (M-7)",
        input: "e\u{0301}",
        expected: "identifier:e error:\u{0301}[LEX005]",
        evidence: "M-7 combining marks; measured boundary of the accepted authority",
    },
    ProbeCase {
        id: "m7_leading_combining",
        group: "identifier-classification (M-7)",
        input: "\u{0301}name",
        expected: "error:\u{0301}[LEX005] identifier:name",
        evidence: "M-7 combining marks in start position",
    },
    ProbeCase {
        id: "m7_emoji",
        group: "identifier-classification (M-7)",
        input: "😀",
        expected: "error:😀[LEX005]",
        evidence: "M-7 Unicode edge categories (symbols are not identifiers)",
    },
    ProbeCase {
        id: "m7_normalization_not_applied",
        group: "identifier-classification (M-7)",
        input: "café cafe\u{0301}",
        expected: "identifier:café ws identifier:cafe error:\u{0301}[LEX005]",
        evidence: "M-7 normalization-sensitive spellings stay distinct; no normalization",
    },
    // --- escape contracts (D-L5, D-L8, D-L9) --------------------------------
    ProbeCase {
        id: "esc_unknown",
        group: "escape-contracts (D-L5/D-L8/D-L9)",
        input: r#""a\qb""#,
        expected: r#"string literal:"a\\qb"[LEX008]"#,
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L8 (strict; declared divergence)",
    },
    ProbeCase {
        id: "esc_unicode_escape_valid",
        group: "escape-contracts (D-L5/D-L8/D-L9)",
        input: r#""\u0041""#,
        expected: r#"string literal:"\\u0041""#,
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L5 (four-hex-digit form)",
    },
    ProbeCase {
        id: "esc_lone_high_string",
        group: "escape-contracts (D-L5/D-L8/D-L9)",
        input: r#""\uD800""#,
        expected: r#"string literal:"\\uD800"[LEX010]"#,
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L9",
    },
    ProbeCase {
        id: "esc_lone_low_string",
        group: "escape-contracts (D-L5/D-L8/D-L9)",
        input: r#""\uDC00""#,
        expected: r#"string literal:"\\uDC00"[LEX010]"#,
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L9",
    },
    ProbeCase {
        id: "esc_pair_string",
        group: "escape-contracts (D-L5/D-L8/D-L9)",
        input: r#""\uD83D\uDE00""#,
        expected: r#"string literal:"\\uD83D\\uDE00""#,
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L9 (pair decodes to one scalar)",
    },
    ProbeCase {
        id: "esc_lone_high_char",
        group: "escape-contracts (D-L5/D-L8/D-L9)",
        input: r#"'\uD800'"#,
        expected: r#"character literal:'\\uD800'[LEX010]"#,
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L9 (each half alone)",
    },
    ProbeCase {
        id: "esc_pair_char",
        group: "escape-contracts (D-L5/D-L8/D-L9)",
        input: r#"'\uD83D\uDE00'"#,
        expected: r#"character literal:'\\uD83D\\uDE00'[LEX004]"#,
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L5 item 4 (char literal is exactly one char-value)",
    },
    ProbeCase {
        id: "esc_empty_char",
        group: "escape-contracts (D-L5/D-L8/D-L9)",
        input: "''",
        expected: "error:''[LEX003]",
        evidence: "LEX003 empty character literal",
    },
    // --- other known divergences -------------------------------------------
    ProbeCase {
        id: "div_line_comment_cr",
        group: "other-known-divergences",
        input: "x // note\ry",
        expected: "identifier:x ws line-comment eol:Cr identifier:y",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L4 (bare CR terminates)",
    },
    ProbeCase {
        id: "div_line_comment_crlf",
        group: "other-known-divergences",
        input: "x // note\r\ny",
        expected: "identifier:x ws line-comment eol:CrLf identifier:y",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L4",
    },
    ProbeCase {
        id: "div_triple_quote",
        group: "other-known-divergences",
        input: r#""""x""""#,
        expected: r#"error:"""x"""[LEX009]"#,
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L6 (declared divergence)",
    },
    ProbeCase {
        id: "div_bom_prefix",
        group: "other-known-divergences",
        input: "\u{FEFF}x",
        expected: "bom identifier:x",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L1",
    },
    ProbeCase {
        id: "div_bom_midfile",
        group: "other-known-divergences",
        input: "x\u{FEFF}y",
        expected: "identifier:x error:\u{FEFF}[LEX005] identifier:y",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L1 (only at byte 0)",
    },
    ProbeCase {
        id: "div_number_longest_prefix",
        group: "other-known-divergences",
        input: "0x1F3G",
        expected: "integer literal:0x1F3 identifier:G",
        evidence: "longest-valid-prefix numeric recognition (A1-R1 contract)",
    },
    ProbeCase {
        id: "div_number_long_int_suffix",
        group: "other-known-divergences",
        input: "10L",
        expected: "long literal:10L",
        evidence: "long suffix on integer forms (A1-R1 contract)",
    },
    ProbeCase {
        id: "div_number_double_long_suffix",
        group: "other-known-divergences",
        input: "1.0L",
        expected: "double literal:1.0 identifier:L",
        evidence: "long suffix restricted to integer forms (A1-R1 contract)",
    },
    ProbeCase {
        id: "div_raw_crlf_string",
        group: "other-known-divergences",
        input: "\"a\r\nb\"",
        expected: r#"string literal:"a\r\nb""#,
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L7 (raw newlines are content; decoded-value question stays open)",
    },
    ProbeCase {
        id: "div_columns_scalar_values",
        group: "other-known-divergences",
        input: "😀x",
        expected: "line_col(x)=1:2 (1-based; scalar-value columns)",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L2 (UTF-16 vs scalar-value convention)",
    },
    ProbeCase {
        id: "div_char_recovery_multi",
        group: "other-known-divergences",
        input: "'ab'",
        expected: "character literal:'ab'[LEX004]",
        evidence: "LEX004 multi-character literal recovery",
    },
    ProbeCase {
        id: "div_char_recovery_unterminated",
        group: "other-known-divergences",
        input: "'a",
        expected: "error:'a[LEX004]",
        evidence: "LEX004 unterminated literal recovery",
    },
    ProbeCase {
        id: "div_nbsp",
        group: "other-known-divergences",
        input: "x\u{00A0}y",
        expected: "identifier:x error:\u{00A0}[LEX005] identifier:y",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L4 (unexpected character)",
    },
    ProbeCase {
        id: "div_line_separator",
        group: "other-known-divergences",
        input: "x\u{2028}y",
        expected: "identifier:x error:\u{2028}[LEX005] identifier:y",
        evidence: "docs/spec/g1-l-lexical-decision-package.md D-L4 (U+2028 is not a line terminator)",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_expectations_match_observations() {
        let report = probe_report();
        let mismatches: Vec<String> = report
            .outcomes
            .iter()
            .filter(|outcome| !outcome.pass)
            .map(|outcome| {
                format!(
                    "{}: observed {:?}, expected {:?}",
                    outcome.case_id, outcome.observed, outcome.expected
                )
            })
            .collect();
        assert!(
            mismatches.is_empty(),
            "probe mismatches:\n{}",
            mismatches.join("\n")
        );
        assert_eq!(report.failed, 0);
        assert!(report.passed >= 30, "unexpectedly small probe set");
    }
}
