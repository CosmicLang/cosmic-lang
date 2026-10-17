//! End-to-end tests of the differential harness pipeline.
//!
//! The synthetic oracle used here is authored by this repository purely to
//! exercise the harness machinery. It is not a legacy compiler, and its
//! results are never legacy-compatibility evidence.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cosmic_diff_harness::{
    Classification, Corpus, DeclaredDivergence, DiffOptions, FixtureCategory, OracleConfig,
    run_diff,
};

fn all_categories() -> BTreeSet<FixtureCategory> {
    FixtureCategory::ALL.into_iter().collect()
}

fn unique_dir(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after the epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "cosmic-diff-harness-test-{label}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn clean(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

/// Writes the synthetic self-test oracle and returns `(program, args)`.
///
/// The script is authored for this repository: it accepts files containing
/// `ORACLE_ACCEPT`, sleeps forever on `ORACLE_TIMEOUT`, and rejects
/// everything else.
fn synthetic_oracle(dir: &Path) -> (String, Vec<String>) {
    let script = dir.join("oracle.sh");
    fs::write(
        &script,
        "#!/bin/sh\n\
         # Synthetic harness self-test oracle (authored for this repository).\n\
         input=\"$1\"\n\
         if grep -q 'ORACLE_ACCEPT' \"$input\"; then exit 0; fi\n\
         if grep -q 'ORACLE_TIMEOUT' \"$input\"; then exec sleep 30; fi\n\
         exit 1\n",
    )
    .expect("write oracle script");
    (
        "sh".to_owned(),
        vec![script.to_string_lossy().into_owned(), "{input}".to_owned()],
    )
}

#[test]
fn embedded_corpus_runs_without_an_oracle_and_matches_expectations() {
    let options = DiffOptions {
        corpus: Corpus::embedded(),
        categories: all_categories(),
        oracle: None,
        declared: DeclaredDivergence::defaults(),
        keep_temp: false,
    };
    let summary = run_diff(&options).expect("harness run");
    let failures: Vec<&str> = summary
        .results
        .iter()
        .filter(|result| !result.expectation_ok)
        .map(|result| result.id.as_str())
        .collect();
    assert!(
        failures.is_empty(),
        "Rust-side expectation failures: {failures:?}"
    );
    assert_eq!(summary.verdict, "skipped-no-oracle");
    assert_eq!(summary.unconfirmed_differences, 0);
    assert!(
        summary
            .results
            .iter()
            .all(|result| result.classification == Classification::OracleUnavailable)
    );
    assert!(
        summary
            .results
            .iter()
            .all(|result| result.rust_lossless && result.rust_reprint_ok)
    );
    assert!(summary.results.len() >= 45, "corpus unexpectedly small");
}

#[test]
fn synthetic_oracle_classifications_cover_agreement_declaration_and_triage() {
    let dir = unique_dir("pipeline");
    let corpus_dir = dir.join("corpus");
    for category in ["lexical", "syntax", "refusals", "malformed"] {
        fs::create_dir_all(corpus_dir.join(category)).expect("create category dir");
    }
    let write = |category: &str, id: &str, text: &str| {
        fs::write(corpus_dir.join(category).join(format!("{id}.csmx")), text)
            .expect("write fixture");
    };
    write(
        "syntax",
        "agree_accept",
        "main() {\n    // ORACLE_ACCEPT\n    x()\n}\n",
    );
    write("malformed", "agree_reject", "main() {\n");
    write("syntax", "legacy_reject", "main() {\n    x()\n}\n");
    write(
        "lexical",
        "declared_case",
        "main() {\n    val s = \"a\\qb\" // ORACLE_ACCEPT\n}\n",
    );
    let manifest = "agree_accept\tsyntax\trecognised\taccepted by both\n\
                    agree_reject\tmalformed\terror\trefused by both\n\
                    legacy_reject\tsyntax\trecognised\toracle rejection needing triage\n\
                    declared_case\tlexical\terror\tdeclared divergence with \"quotes\" and \\ backslash\n";
    fs::write(corpus_dir.join("manifest.tsv"), manifest).expect("write manifest");
    fs::write(
        corpus_dir.join("declared.tsv"),
        "declared_case\tself-test declaration\tcosmic-diff-harness pipeline test\n",
    )
    .expect("write declared tsv");

    let corpus = Corpus::from_dir(&corpus_dir).expect("load external corpus");
    let declared = DeclaredDivergence::from_tsv(&corpus_dir).expect("load declarations");
    let (program, args) = synthetic_oracle(&dir);
    let options = DiffOptions {
        corpus,
        categories: all_categories(),
        oracle: Some(OracleConfig {
            program,
            args,
            timeout: Duration::from_secs(10),
            pin: Some("self-test-pin".to_owned()),
        }),
        declared,
        keep_temp: false,
    };
    let summary = run_diff(&options).expect("harness run");
    assert_eq!(summary.expectation_failures, 0);
    assert_eq!(summary.counts.get("agree-accept"), Some(&1));
    assert_eq!(summary.counts.get("agree-reject"), Some(&1));
    assert_eq!(summary.counts.get("legacy-rejection-unconfirmed"), Some(&1));
    assert_eq!(summary.counts.get("declared-divergence"), Some(&1));
    assert_eq!(summary.unconfirmed_differences, 1);
    assert_eq!(summary.verdict, "differences-need-triage");
    assert_eq!(summary.oracle_pin, "self-test-pin");
    let json = summary.to_json();
    assert!(
        json.contains(r#""description": "declared divergence with \"quotes\" and \\ backslash""#),
        "JSON escaping of quotes/backslashes"
    );
    assert!(summary.to_markdown().contains("## Unconfirmed differences"));
    clean(&dir);
}

#[test]
fn oracle_timeout_is_killed_and_reported_without_hanging() {
    let dir = unique_dir("timeout");
    let corpus_dir = dir.join("corpus");
    fs::create_dir_all(corpus_dir.join("syntax")).expect("create category dir");
    fs::write(
        corpus_dir.join("syntax").join("slow.csmx"),
        "main() {\n    // ORACLE_TIMEOUT\n    x()\n}\n",
    )
    .expect("write fixture");
    fs::write(
        corpus_dir.join("manifest.tsv"),
        "slow\tsyntax\trecognised\tslow oracle\n",
    )
    .expect("write manifest");
    let (program, args) = synthetic_oracle(&dir);
    let start = Instant::now();
    let summary = run_diff(&DiffOptions {
        corpus: Corpus::from_dir(&corpus_dir).expect("load corpus"),
        categories: all_categories(),
        oracle: Some(OracleConfig {
            program,
            args,
            timeout: Duration::from_millis(300),
            pin: None,
        }),
        declared: Vec::new(),
        keep_temp: false,
    })
    .expect("harness run");
    let wall = start.elapsed();
    assert!(wall < Duration::from_secs(10), "harness hung: {wall:?}");
    let result = &summary.results[0];
    assert_eq!(result.oracle_status, "timed-out");
    assert_eq!(result.classification, Classification::OracleUnavailable);
    assert!(result.note.contains("timeout"));
    clean(&dir);
}

#[test]
fn unavailable_oracle_program_is_an_explicit_skip() {
    let summary = run_diff(&DiffOptions {
        corpus: Corpus::embedded(),
        categories: all_categories(),
        oracle: Some(OracleConfig {
            program: "/nonexistent-cosmic-legacy-oracle".to_owned(),
            args: Vec::new(),
            timeout: Duration::from_secs(5),
            pin: None,
        }),
        declared: DeclaredDivergence::defaults(),
        keep_temp: false,
    })
    .expect("harness run");
    assert!(
        summary
            .results
            .iter()
            .all(|result| result.classification == Classification::OracleUnavailable)
    );
    assert!(
        summary
            .results
            .iter()
            .all(|result| result.oracle_status == "spawn-failed")
    );
    assert_eq!(summary.verdict, "skipped-oracle-unavailable");
    assert_eq!(summary.expectation_failures, 0);
}

#[test]
fn json_summary_is_deterministic() {
    let run = || {
        run_diff(&DiffOptions {
            corpus: Corpus::embedded(),
            categories: all_categories(),
            oracle: None,
            declared: DeclaredDivergence::defaults(),
            keep_temp: false,
        })
        .expect("harness run")
        .to_json()
    };
    let first = run();
    let second = run();
    assert_eq!(first, second);
    assert!(first.starts_with("{\n"));
    assert!(first.trim_end().ends_with('}'));
    assert!(first.contains("\"schema\": \"cosmic-diff-harness/summary/1\""));
    assert!(first.contains("\"verdict\": \"skipped-no-oracle\""));
    assert!(first.contains("\"pin_assertion\": \"none\""));
}

#[test]
#[ignore = "requires an externally provided legacy oracle: set COSMIC_LEGACY_ORACLE (program) and optionally COSMIC_LEGACY_ORACLE_ARGS (whitespace-separated arguments; {input} is substituted) and COSMIC_LEGACY_ORACLE_PIN; ordinary CI never runs this"]
fn committed_corpus_against_the_configured_legacy_oracle() {
    let program = std::env::var("COSMIC_LEGACY_ORACLE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .expect("set COSMIC_LEGACY_ORACLE to the externally obtained legacy oracle executable");
    let args: Vec<String> = std::env::var("COSMIC_LEGACY_ORACLE_ARGS")
        .map(|value| value.split_whitespace().map(str::to_owned).collect())
        .unwrap_or_default();
    let pin = std::env::var("COSMIC_LEGACY_ORACLE_PIN")
        .ok()
        .filter(|value| !value.trim().is_empty());
    let options = DiffOptions {
        corpus: Corpus::embedded(),
        categories: all_categories(),
        oracle: Some(OracleConfig {
            program,
            args,
            timeout: Duration::from_secs(30),
            pin,
        }),
        declared: DeclaredDivergence::defaults(),
        keep_temp: false,
    };
    let summary = run_diff(&options).expect("harness run");
    println!("{}", summary.to_markdown());
    let artifacts = Path::new("target").join("diff-harness");
    let _ = fs::create_dir_all(&artifacts);
    let _ = fs::write(artifacts.join("summary.json"), summary.to_json());
    let _ = fs::write(artifacts.join("summary.md"), summary.to_markdown());
    assert_eq!(
        summary.expectation_failures, 0,
        "the Rust side must satisfy the corpus even when the oracle disagrees"
    );
}
