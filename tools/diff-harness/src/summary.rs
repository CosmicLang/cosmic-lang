//! Deterministic machine-readable summaries and the human-readable report.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use crate::compare::Classification;

/// The JSON schema identifier of a differential summary.
pub const SCHEMA: &str = "cosmic-diff-harness/summary/1";

/// One fixture's complete differential result.
#[derive(Clone, Debug)]
pub struct FixtureResult {
    /// Fixture identifier.
    pub id: String,
    /// Comparison category.
    pub category: String,
    /// Corpus expectation spelling.
    pub expectation: String,
    /// Whether the Rust frontend satisfied the corpus expectation.
    pub expectation_ok: bool,
    /// One-line description.
    pub description: String,
    /// Whether the Rust frontend accepted the input (no diagnostics).
    pub rust_accepts: bool,
    /// Rust diagnostic codes such as `LEX008,PARSE002`, or `none`.
    pub rust_codes: String,
    /// CST node count.
    pub rust_nodes: usize,
    /// CST losslessness.
    pub rust_lossless: bool,
    /// Byte-for-byte reprint success.
    pub rust_reprint_ok: bool,
    /// Oracle status label (see `OracleStatus::label`).
    pub oracle_status: String,
    /// Oracle exit code when one exists.
    pub oracle_exit: Option<i32>,
    /// Oracle invocation duration in milliseconds.
    pub oracle_duration_ms: u64,
    /// First characters of captured oracle stderr, flattened to one line.
    pub oracle_stderr_head: String,
    /// The comparison classification.
    pub classification: Classification,
    /// Reason, triage hint or declaration record.
    pub note: String,
}

/// The complete result of one differential run.
#[derive(Clone, Debug)]
pub struct Summary {
    /// Schema identifier (always the fixed `cosmic-diff-harness/summary/1`).
    pub schema: &'static str,
    /// Whether an oracle was configured for this run.
    pub oracle_configured: bool,
    /// The configured oracle program (empty when none).
    pub oracle_program: String,
    /// The operator-provided pin assertion (empty when none). The harness
    /// records it verbatim but cannot verify it.
    pub oracle_pin: String,
    /// Digest of the compared corpus.
    pub corpus_digest: String,
    /// Selected target categories, comma-separated.
    pub targets: String,
    /// Number of compared fixtures.
    pub fixture_count: usize,
    /// Per-fixture results, sorted by fixture id.
    pub results: Vec<FixtureResult>,
    /// Classification counts by stable label.
    pub counts: BTreeMap<&'static str, usize>,
    /// Fixtures where the Rust side violated its corpus expectation.
    pub expectation_failures: usize,
    /// Differences needing triage (unconfirmed, both directions).
    pub unconfirmed_differences: usize,
    /// `compared`, `differences-need-triage`, `skipped-no-oracle`,
    /// `skipped-oracle-unavailable` or `corpus-expectation-failure`.
    pub verdict: &'static str,
}

impl Summary {
    /// Folds per-fixture results into the complete summary.
    #[must_use]
    pub fn from_results(
        oracle_configured: bool,
        oracle_program: String,
        oracle_pin: String,
        corpus_digest: String,
        targets: String,
        results: Vec<FixtureResult>,
    ) -> Summary {
        let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
        let mut expectation_failures = 0;
        let mut unconfirmed_differences = 0;
        let mut compared_any = false;
        for result in &results {
            *counts.entry(result.classification.label()).or_insert(0) += 1;
            if !result.expectation_ok {
                expectation_failures += 1;
            }
            if result.classification.needs_triage() {
                unconfirmed_differences += 1;
            }
            if result.oracle_status == "accepted" || result.oracle_status == "rejected" {
                compared_any = true;
            }
        }
        let verdict = if expectation_failures > 0 {
            "corpus-expectation-failure"
        } else if !oracle_configured {
            "skipped-no-oracle"
        } else if !compared_any {
            "skipped-oracle-unavailable"
        } else if unconfirmed_differences > 0 {
            "differences-need-triage"
        } else {
            "compared"
        };
        let fixture_count = results.len();
        Summary {
            schema: SCHEMA,
            oracle_configured,
            oracle_program,
            oracle_pin,
            corpus_digest,
            targets,
            fixture_count,
            results,
            counts,
            expectation_failures,
            unconfirmed_differences,
            verdict,
        }
    }

    /// The deterministic machine-readable summary.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        let _ = writeln!(out, "  \"schema\": {},", json_string(self.schema));
        out.push_str("  \"oracle\": {\n");
        let _ = writeln!(out, "    \"configured\": {},", self.oracle_configured);
        let _ = writeln!(
            out,
            "    \"program\": {},",
            json_string(&self.oracle_program)
        );
        let _ = writeln!(out, "    \"pin\": {},", json_string(&self.oracle_pin));
        let pin_assertion = if self.oracle_pin.is_empty() {
            "none"
        } else {
            "operator-provided-unverified"
        };
        let _ = writeln!(out, "    \"pin_assertion\": {}", json_string(pin_assertion));
        out.push_str("  },\n");
        out.push_str("  \"corpus\": {\n");
        let _ = writeln!(out, "    \"digest\": {},", json_string(&self.corpus_digest));
        let _ = writeln!(out, "    \"targets\": {},", json_string(&self.targets));
        let _ = writeln!(out, "    \"fixture_count\": {}", self.fixture_count);
        out.push_str("  },\n");
        out.push_str("  \"counts\": {\n");
        let count_rows: Vec<String> = self
            .counts
            .iter()
            .map(|(label, count)| format!("    {}: {count}", json_string(label)))
            .collect();
        if !count_rows.is_empty() {
            out.push_str(&count_rows.join(",\n"));
            out.push('\n');
        }
        out.push_str("  },\n");
        let _ = writeln!(
            out,
            "  \"expectation_failures\": {},",
            self.expectation_failures
        );
        let _ = writeln!(
            out,
            "  \"unconfirmed_differences\": {},",
            self.unconfirmed_differences
        );
        let _ = writeln!(out, "  \"verdict\": {},", json_string(self.verdict));
        out.push_str("  \"results\": [");
        if self.results.is_empty() {
            out.push_str("]\n}\n");
            return out;
        }
        out.push('\n');
        let objects: Vec<String> = self.results.iter().map(result_json).collect();
        for (index, object) in objects.iter().enumerate() {
            out.push_str(object);
            if index + 1 < objects.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]\n}\n");
        out
    }

    /// The human-readable report.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("# Cosmic frontend differential summary\n\n");
        let _ = writeln!(out, "- **Schema:** `{}`", self.schema);
        let _ = writeln!(
            out,
            "- **Corpus:** {} fixtures, digest `{}`, targets `{}`",
            self.fixture_count, self.corpus_digest, self.targets
        );
        if self.oracle_configured {
            let pin = if self.oracle_pin.is_empty() {
                "none given".to_owned()
            } else {
                format!(
                    "`{}` (operator-asserted; the harness cannot verify it)",
                    self.oracle_pin
                )
            };
            let _ = writeln!(out, "- **Oracle:** `{}`, pin {pin}", self.oracle_program);
        } else {
            out.push_str(
                "- **Oracle:** not configured — every comparison is an explicit skip with a reason\n",
            );
        }
        let _ = writeln!(out, "- **Verdict:** `{}`", self.verdict);
        let _ = writeln!(
            out,
            "- **Expectation failures:** {}",
            self.expectation_failures
        );
        let _ = writeln!(
            out,
            "- **Unconfirmed differences:** {}",
            self.unconfirmed_differences
        );
        out.push_str("\n## Classification counts\n\n| classification | fixtures |\n|---|---|\n");
        for (label, count) in &self.counts {
            let _ = writeln!(out, "| `{label}` | {count} |");
        }
        let triage: Vec<&FixtureResult> = self
            .results
            .iter()
            .filter(|result| result.classification.needs_triage())
            .collect();
        out.push_str("\n## Unconfirmed differences (triage required)\n\n");
        if triage.is_empty() {
            out.push_str("None.\n");
        } else {
            out.push_str("| id | category | rust | oracle | classification | note |\n|---|\n");
            for result in &triage {
                let _ = writeln!(
                    out,
                    "| `{}` | {} | {} | {} | `{}` | {} |",
                    result.id,
                    result.category,
                    md_cell(&result.rust_codes),
                    result.oracle_status,
                    result.classification.label(),
                    md_cell(&result.note)
                );
            }
        }
        let skipped: Vec<&FixtureResult> = self
            .results
            .iter()
            .filter(|result| result.classification == Classification::OracleUnavailable)
            .collect();
        out.push_str("\n## Skipped comparisons\n\n");
        if skipped.is_empty() {
            out.push_str("None.\n");
        } else {
            let reason = skipped
                .first()
                .map(|result| result.note.clone())
                .unwrap_or_default();
            let _ = writeln!(
                out,
                "{} fixture(s) were not compared (first recorded reason: {}).",
                skipped.len(),
                md_cell(&reason)
            );
        }
        out.push_str("\n## All results\n\n| id | category | expectation | rust | oracle | classification | note |\n|---|---|---|---|---|---|---|\n");
        for result in &self.results {
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} | {} | `{}` | {} |",
                result.id,
                result.category,
                result.expectation,
                md_cell(&result.rust_codes),
                result.oracle_status,
                result.classification.label(),
                md_cell(&result.note)
            );
        }
        out
    }
}

fn result_json(result: &FixtureResult) -> String {
    let mut out = String::new();
    out.push_str("    {\n");
    let _ = writeln!(out, "      \"id\": {},", json_string(&result.id));
    let _ = writeln!(
        out,
        "      \"category\": {},",
        json_string(&result.category)
    );
    let _ = writeln!(
        out,
        "      \"expectation\": {},",
        json_string(&result.expectation)
    );
    let _ = writeln!(out, "      \"expectation_ok\": {},", result.expectation_ok);
    let _ = writeln!(
        out,
        "      \"description\": {},",
        json_string(&result.description)
    );
    let _ = writeln!(
        out,
        "      \"rust\": {{\"accepts\": {}, \"codes\": {}, \"nodes\": {}, \"lossless\": {}, \"reprint_ok\": {}}},",
        result.rust_accepts,
        json_string(&result.rust_codes),
        result.rust_nodes,
        result.rust_lossless,
        result.rust_reprint_ok
    );
    let _ = writeln!(
        out,
        "      \"oracle\": {{\"status\": {}, \"exit_code\": {}, \"duration_ms\": {}, \"stderr_head\": {}}},",
        json_string(&result.oracle_status),
        json_optional_i32(result.oracle_exit),
        result.oracle_duration_ms,
        json_string(&result.oracle_stderr_head)
    );
    let _ = writeln!(
        out,
        "      \"classification\": {},",
        json_string(result.classification.label())
    );
    let _ = writeln!(out, "      \"note\": {}", json_string(&result.note));
    out.push_str("    }");
    out
}

fn json_optional_i32(value: Option<i32>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".to_owned(),
    }
}

/// Escapes a string for embedded JSON (shared with the probe report).
pub(crate) fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 || ch == '\u{7f}' => {
                let _ = write!(out, "\\u{:04x}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// Escapes a value for a Markdown table cell (shared with the probe report).
pub(crate) fn md_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}
