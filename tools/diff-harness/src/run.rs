//! Orchestration: per-fixture execution with temporary-directory isolation.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::HarnessError;
use crate::compare::{DeclaredDivergence, classify};
use crate::corpus::{Corpus, Fixture, FixtureCategory};
use crate::frontend::run_frontend;
use crate::oracle::{OracleConfig, OracleOutcome, OracleStatus, run_oracle};
use crate::summary::{FixtureResult, Summary};

/// Options for one differential run.
#[derive(Clone, Debug)]
pub struct DiffOptions {
    /// The corpus to compare.
    pub corpus: Corpus,
    /// The selected fixture categories.
    pub categories: BTreeSet<FixtureCategory>,
    /// The configured oracle; `None` records explicit skips.
    pub oracle: Option<OracleConfig>,
    /// Declarations that classify a disagreement as intentional.
    pub declared: Vec<DeclaredDivergence>,
    /// Keep the temporary directory for inspection.
    pub keep_temp: bool,
}

/// Runs the corpus against the Rust frontend and, when configured, the oracle.
///
/// Each fixture is written to a private temporary file before the oracle sees
/// it; the directory is removed afterwards unless `keep_temp` is set.
pub fn run_diff(options: &DiffOptions) -> Result<Summary, HarnessError> {
    let temp_root = unique_temp_dir();
    std::fs::create_dir_all(&temp_root)?;
    let mut results = Vec::new();
    for fixture in options.corpus.select(&options.categories) {
        results.push(run_one(fixture, options, &temp_root)?);
    }
    if !options.keep_temp {
        let _ = std::fs::remove_dir_all(&temp_root);
    }
    let targets = options
        .categories
        .iter()
        .map(|category| category.name())
        .collect::<Vec<_>>()
        .join(",");
    let (configured, program, pin) = match &options.oracle {
        Some(config) => (
            true,
            config.program.clone(),
            config.pin.clone().unwrap_or_default(),
        ),
        None => (false, String::new(), String::new()),
    };
    Ok(Summary::from_results(
        configured,
        program,
        pin,
        options.corpus.digest(),
        targets,
        results,
    ))
}

fn run_one(
    fixture: &Fixture,
    options: &DiffOptions,
    temp_root: &std::path::Path,
) -> Result<FixtureResult, HarnessError> {
    let fixture_path = temp_root.join(format!("{}.csmx", fixture.id));
    std::fs::write(&fixture_path, &fixture.text)?;
    let frontend = run_frontend(&fixture.text)?;
    let oracle_outcome = match &options.oracle {
        Some(config) => run_oracle(config, &fixture_path),
        None => OracleOutcome {
            status: OracleStatus::NotConfigured,
            duration_ms: 0,
            stdout: String::new(),
            stderr: String::new(),
        },
    };
    let (classification, note) = classify(
        fixture,
        frontend.accepted(),
        &oracle_outcome.status,
        &options.declared,
    );
    Ok(FixtureResult {
        id: fixture.id.clone(),
        category: fixture.category.name().to_owned(),
        expectation: fixture.expectation.name().to_owned(),
        expectation_ok: frontend.matches(fixture.expectation),
        description: fixture.description.clone(),
        rust_accepts: frontend.accepted(),
        rust_codes: frontend.codes_label(),
        rust_nodes: frontend.nodes,
        rust_lossless: frontend.lossless,
        rust_reprint_ok: frontend.reprint_ok,
        oracle_status: oracle_outcome.status.label().to_owned(),
        oracle_exit: oracle_outcome.status.exit_code(),
        oracle_duration_ms: oracle_outcome.duration_ms,
        oracle_stderr_head: head(&oracle_outcome.stderr),
        classification,
        note,
    })
}

/// A unique per-run temporary directory (identity only; never in reports).
fn unique_temp_dir() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "cosmic-diff-harness-{}-{stamp}",
        std::process::id()
    ))
}

/// The first line of oracle stderr, capped at 200 characters.
fn head(text: &str) -> String {
    let line = text.lines().next().unwrap_or("");
    let mut out = String::new();
    for (index, ch) in line.chars().enumerate() {
        if index >= 200 {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}
