//! Comparison semantics between the Rust frontend and the configured oracle.
//!
//! The classification deliberately keeps *unexplained* differences as
//! unconfirmed rather than folding them into agreement:
//!
//! * a legacy rejection of input the Rust frontend recognised may be a
//!   semantic or target-scope rejection and is never counted as a syntax
//!   incompatibility automatically ([`Classification::LegacyRejectionUnconfirmed`]);
//! * a Rust refusal of input the oracle accepts is a recognition-mismatch
//!   candidate and is reported with the fixture id for a minimal
//!   reproduction ([`Classification::RustRejectionUnconfirmed`]);
//! * only an explicit declaration ([`DeclaredDivergence`]) or an
//!   unsupported-family refusal turns a disagreement into a documented class.

use std::path::Path;

use crate::HarnessError;
use crate::corpus::{Fixture, FixtureCategory};
use crate::oracle::OracleStatus;

/// The outcome class of one fixture comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Classification {
    /// Both sides accept the input.
    AgreeAccept,
    /// Both sides refuse/reject the input.
    AgreeReject,
    /// A disagreement covered by an explicit declaration.
    DeclaredDivergence,
    /// The oracle accepts a syntax family this frontend refuses by design.
    UnsupportedFamily,
    /// The Rust frontend refused input the oracle accepts: a
    /// recognition-mismatch candidate needing triage.
    RustRejectionUnconfirmed,
    /// The oracle rejected input the Rust frontend recognised: possibly a
    /// semantic or target-scope rejection; needs triage.
    LegacyRejectionUnconfirmed,
    /// No comparison happened (no oracle configured, timeout or execution
    /// failure): an explicit skip with a recorded reason.
    OracleUnavailable,
}

impl Classification {
    /// The stable label used in summaries and reports.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Classification::AgreeAccept => "agree-accept",
            Classification::AgreeReject => "agree-reject",
            Classification::DeclaredDivergence => "declared-divergence",
            Classification::UnsupportedFamily => "unsupported-family",
            Classification::RustRejectionUnconfirmed => "rust-rejection-unconfirmed",
            Classification::LegacyRejectionUnconfirmed => "legacy-rejection-unconfirmed",
            Classification::OracleUnavailable => "oracle-unavailable",
        }
    }

    /// Whether this class is an unconfirmed difference requiring triage.
    #[must_use]
    pub fn needs_triage(self) -> bool {
        matches!(
            self,
            Classification::RustRejectionUnconfirmed | Classification::LegacyRejectionUnconfirmed
        )
    }
}

/// One declared, intentional divergence or unsupported-family note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredDivergence {
    /// The fixture id this declaration covers.
    pub id: String,
    /// Why the difference is intentional.
    pub reason: String,
    /// The accepted record that authorises it.
    pub record: String,
}

impl DeclaredDivergence {
    /// The declarations for the embedded corpus, each citing its record.
    #[must_use]
    pub fn defaults() -> Vec<DeclaredDivergence> {
        let entry = |id: &str, reason: &str, record: &str| DeclaredDivergence {
            id: id.to_owned(),
            reason: reason.to_owned(),
            record: record.to_owned(),
        };
        vec![
            entry(
                "lex_strict_escape",
                "unknown escapes are refused (LEX008); the accepted C2 divergence from the legacy silent-escape behaviour",
                "docs/spec/g1-l-lexical-decision-package.md D-L8",
            ),
            entry(
                "lex_triple_quoted",
                "triple-quoted syntax is refused as one region (LEX009) by the accepted lexical contract",
                "docs/spec/g1-l-lexical-decision-package.md D-L6",
            ),
            entry(
                "syn_untyped_lambda",
                "untyped lambda parameters are refused as a documented Stage A boundary",
                "docs/spec/g1-s-syntax-decision-package.md",
            ),
            entry(
                "mal_nesting_limit",
                "the 512-level implementation guard (PARSE005) may reject inputs a legacy compiler accepts; it is an implementation resource limit, not a syntax rule",
                "docs/spec/diagnostic-codes.md §6.3 and owner decision 003-A2-R1-FINAL",
            ),
        ]
    }

    /// Loads an optional `<root>/declared.tsv` with `id<TAB>reason<TAB>record`
    /// rows. A missing file yields an empty list.
    pub fn from_tsv(root: &Path) -> Result<Vec<DeclaredDivergence>, HarnessError> {
        let path = root.join("declared.tsv");
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(HarnessError::Corpus(format!("{}: {error}", path.display())));
            }
        };
        let mut declarations = Vec::new();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim_end_matches('\r');
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != 3 {
                return Err(HarnessError::Corpus(format!(
                    "declared.tsv row {}: expected 3 tab-separated fields, found {}",
                    index + 1,
                    fields.len()
                )));
            }
            declarations.push(DeclaredDivergence {
                id: fields[0].to_owned(),
                reason: fields[1].to_owned(),
                record: fields[2].to_owned(),
            });
        }
        Ok(declarations)
    }

    /// Finds the declaration covering `id`.
    #[must_use]
    pub fn find<'a>(
        declared: &'a [DeclaredDivergence],
        id: &str,
    ) -> Option<&'a DeclaredDivergence> {
        declared.iter().find(|declaration| declaration.id == id)
    }
}

/// Classifies one comparison and returns the note for the report.
#[must_use]
pub fn classify(
    fixture: &Fixture,
    rust_accepts: bool,
    oracle: &OracleStatus,
    declared: &[DeclaredDivergence],
) -> (Classification, String) {
    match oracle {
        OracleStatus::NotConfigured => (
            Classification::OracleUnavailable,
            "no oracle configured; comparison explicitly skipped".to_owned(),
        ),
        OracleStatus::TimedOut => (
            Classification::OracleUnavailable,
            "the oracle exceeded the timeout and was killed; this is an environmental failure, not a comparison".to_owned(),
        ),
        OracleStatus::SpawnFailed(error) => (
            Classification::OracleUnavailable,
            format!("the oracle could not run: {error}"),
        ),
        OracleStatus::Accepted => {
            if rust_accepts {
                (Classification::AgreeAccept, String::new())
            } else if let Some(declaration) = DeclaredDivergence::find(declared, &fixture.id) {
                (
                    Classification::DeclaredDivergence,
                    format!("{} (record: {})", declaration.reason, declaration.record),
                )
            } else if fixture.category == FixtureCategory::Refusals {
                (
                    Classification::UnsupportedFamily,
                    "the oracle accepts an unsupported family this frontend refuses by design (outside the accepted scope)"
                        .to_owned(),
                )
            } else {
                (
                    Classification::RustRejectionUnconfirmed,
                    "the Rust frontend refuses input the oracle accepts: recognition-mismatch candidate needing triage"
                        .to_owned(),
                )
            }
        }
        OracleStatus::Rejected(code) => {
            if !rust_accepts {
                (
                    Classification::AgreeReject,
                    format!("both sides refuse (oracle exit code {code:?})"),
                )
            } else if let Some(declaration) = DeclaredDivergence::find(declared, &fixture.id) {
                (
                    Classification::DeclaredDivergence,
                    format!("{} (record: {})", declaration.reason, declaration.record),
                )
            } else {
                (
                    Classification::LegacyRejectionUnconfirmed,
                    format!(
                        "the oracle exited with code {code:?}; this may be a semantic or target-scope rejection and must be triaged, not counted as a syntax incompatibility"
                    ),
                )
            }
        }
    }
}
