//! CLI for the A3 differential and measurement harness.
//!
//! Commands and the exit-code contract are documented in the crate
//! `README.md`; run `--help` for a summary.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use cosmic_diff_harness::{
    Corpus, DeclaredDivergence, DiffOptions, FixtureCategory, HarnessError, OracleConfig,
    ProbeReport, probe_report, run_diff, run_frontend,
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("cosmic-diff-harness: {error}");
            ExitCode::from(2)
        }
    }
}

enum Mode {
    Diff,
    List,
    SelfCheck,
    EmitProbes(PathBuf),
    EmitProbesJson(PathBuf),
    Measure(PathBuf, usize),
}

struct Cli {
    mode: Mode,
    corpus_dir: Option<PathBuf>,
    targets: BTreeSet<FixtureCategory>,
    oracle_program: Option<String>,
    oracle_args: Vec<String>,
    oracle_pin: Option<String>,
    timeout_ms: u64,
    json_out: Option<PathBuf>,
    report_out: Option<PathBuf>,
    keep_temp: bool,
}

fn run() -> Result<ExitCode, HarnessError> {
    let cli = parse_args()?;
    match &cli.mode {
        Mode::EmitProbes(path) => {
            let report = probe_report();
            std::fs::write(path, report.to_markdown())?;
            println!(
                "wrote {} ({} cases, {} passed, {} failed)",
                path.display(),
                report.outcomes.len(),
                report.passed,
                report.failed
            );
            return Ok(exit_for_probes(&report));
        }
        Mode::EmitProbesJson(path) => {
            let report = probe_report();
            std::fs::write(path, report.to_json())?;
            println!(
                "wrote {} ({} cases, {} passed, {} failed)",
                path.display(),
                report.outcomes.len(),
                report.passed,
                report.failed
            );
            return Ok(exit_for_probes(&report));
        }
        Mode::Measure(file, repeat) => {
            measure(file, *repeat)?;
            return Ok(ExitCode::SUCCESS);
        }
        Mode::Diff | Mode::List | Mode::SelfCheck => {}
    }

    let corpus = match &cli.corpus_dir {
        Some(dir) => Corpus::from_dir(dir)?,
        None => Corpus::embedded(),
    };
    let declared = match &cli.corpus_dir {
        Some(dir) => {
            let loaded = DeclaredDivergence::from_tsv(dir)?;
            if loaded.is_empty() {
                DeclaredDivergence::defaults()
            } else {
                loaded
            }
        }
        None => DeclaredDivergence::defaults(),
    };

    match &cli.mode {
        Mode::List => {
            for fixture in corpus.select(&cli.targets) {
                println!(
                    "{}\t{}\t{}\t{}",
                    fixture.id,
                    fixture.category.name(),
                    fixture.expectation.name(),
                    fixture.description
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Mode::SelfCheck => {
            let options = DiffOptions {
                corpus,
                categories: cli.targets.clone(),
                oracle: None,
                declared,
                keep_temp: false,
            };
            let summary = run_diff(&options)?;
            println!(
                "corpus: {} fixtures, digest {}, expectation failures: {}",
                summary.fixture_count, summary.corpus_digest, summary.expectation_failures
            );
            for result in summary
                .results
                .iter()
                .filter(|result| !result.expectation_ok)
            {
                println!(
                    "FAIL {}: codes {} does not satisfy expectation {} ({})",
                    result.id, result.rust_codes, result.expectation, result.description
                );
            }
            Ok(if summary.expectation_failures == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(2)
            })
        }
        Mode::Diff => {
            if !cli.oracle_args.is_empty() && cli.oracle_program.is_none() {
                return Err(HarnessError::Usage(
                    "--oracle-arg requires --oracle or COSMIC_LEGACY_ORACLE".to_owned(),
                ));
            }
            let oracle = cli.oracle_program.as_ref().map(|program| OracleConfig {
                program: program.clone(),
                args: cli.oracle_args.clone(),
                timeout: Duration::from_millis(cli.timeout_ms),
                pin: cli.oracle_pin.clone(),
            });
            let options = DiffOptions {
                corpus,
                categories: cli.targets.clone(),
                oracle,
                declared,
                keep_temp: cli.keep_temp,
            };
            let summary = run_diff(&options)?;
            if let Some(path) = &cli.json_out {
                std::fs::write(path, summary.to_json())?;
            }
            let report = summary.to_markdown();
            match &cli.report_out {
                Some(path) => {
                    std::fs::write(path, &report)?;
                    println!("wrote {}", path.display());
                }
                None => print!("{report}"),
            }
            println!(
                "verdict: {} ({} fixtures, {} expectation failures, {} unconfirmed differences)",
                summary.verdict,
                summary.fixture_count,
                summary.expectation_failures,
                summary.unconfirmed_differences
            );
            let code = if summary.expectation_failures > 0 {
                2
            } else if summary.unconfirmed_differences > 0 {
                1
            } else if summary.verdict.starts_with("skipped") {
                3
            } else {
                0
            };
            Ok(ExitCode::from(code))
        }
        Mode::EmitProbes(_) | Mode::EmitProbesJson(_) | Mode::Measure(_, _) => {
            unreachable!("handled before the corpus is loaded")
        }
    }
}

fn exit_for_probes(report: &ProbeReport) -> ExitCode {
    if report.failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    }
}

fn measure(file: &Path, repeat: usize) -> Result<(), HarnessError> {
    let text = std::fs::read_to_string(file)?;
    let bytes = text.len();
    let mut times: Vec<u128> = Vec::with_capacity(repeat);
    for index in 0..repeat {
        let start = Instant::now();
        let outcome = run_frontend(&text)?;
        let elapsed_ns = start.elapsed().as_nanos();
        times.push(elapsed_ns);
        println!(
            "{{\"run\": {index}, \"bytes\": {bytes}, \"nodes\": {}, \"accepts\": {}, \"lossless\": {}, \"reprint_ok\": {}, \"elapsed_ns\": {elapsed_ns}}}",
            outcome.nodes,
            outcome.accepted(),
            outcome.lossless,
            outcome.reprint_ok
        );
    }
    let mut sorted = times.clone();
    sorted.sort_unstable();
    let min = sorted.first().copied().unwrap_or(0);
    let max = sorted.last().copied().unwrap_or(0);
    let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0);
    let mib_per_second = if median > 0 {
        (bytes as f64 / (median as f64 / 1_000_000_000.0)) / (1024.0 * 1024.0)
    } else {
        0.0
    };
    println!(
        "{{\"aggregate\": {{\"runs\": {}, \"min_ns\": {min}, \"median_ns\": {median}, \"max_ns\": {max}, \"median_mib_per_s\": {mib_per_second:.3}}}}}",
        times.len()
    );
    Ok(())
}

fn parse_args() -> Result<Cli, HarnessError> {
    let mut cli = Cli {
        mode: Mode::Diff,
        corpus_dir: None,
        targets: FixtureCategory::ALL.into_iter().collect(),
        oracle_program: std::env::var("COSMIC_LEGACY_ORACLE")
            .ok()
            .filter(|value| !value.trim().is_empty()),
        oracle_args: Vec::new(),
        oracle_pin: std::env::var("COSMIC_LEGACY_ORACLE_PIN")
            .ok()
            .filter(|value| !value.trim().is_empty()),
        timeout_ms: 10_000,
        json_out: None,
        report_out: None,
        keep_temp: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--corpus" => {
                cli.corpus_dir = Some(PathBuf::from(next_value(&mut args, "--corpus")?));
            }
            "--targets" => {
                let value = next_value(&mut args, "--targets")?;
                let mut targets = BTreeSet::new();
                for part in value.split(',') {
                    let category = FixtureCategory::parse(part.trim()).ok_or_else(|| {
                        HarnessError::Usage(format!("unknown --targets value {part:?}"))
                    })?;
                    targets.insert(category);
                }
                cli.targets = targets;
            }
            "--oracle" => {
                cli.oracle_program = Some(next_value(&mut args, "--oracle")?);
            }
            "--oracle-arg" => {
                cli.oracle_args.push(next_value(&mut args, "--oracle-arg")?);
            }
            "--oracle-pin" => {
                cli.oracle_pin = Some(next_value(&mut args, "--oracle-pin")?);
            }
            "--timeout-ms" => {
                let value = next_value(&mut args, "--timeout-ms")?;
                cli.timeout_ms = value.parse().map_err(|_| {
                    HarnessError::Usage(format!("invalid --timeout-ms value {value:?}"))
                })?;
            }
            "--json" => {
                cli.json_out = Some(PathBuf::from(next_value(&mut args, "--json")?));
            }
            "--report" => {
                cli.report_out = Some(PathBuf::from(next_value(&mut args, "--report")?));
            }
            "--keep-temp" => cli.keep_temp = true,
            "--list" => cli.mode = Mode::List,
            "--self-check" => cli.mode = Mode::SelfCheck,
            "--emit-probes" => {
                cli.mode = Mode::EmitProbes(PathBuf::from(next_value(&mut args, "--emit-probes")?));
            }
            "--emit-probes-json" => {
                cli.mode = Mode::EmitProbesJson(PathBuf::from(next_value(
                    &mut args,
                    "--emit-probes-json",
                )?));
            }
            "--measure" => {
                let file = PathBuf::from(next_value(&mut args, "--measure")?);
                cli.mode = Mode::Measure(file, 1);
            }
            "--repeat" => {
                let value = next_value(&mut args, "--repeat")?;
                let count: usize = value.parse().map_err(|_| {
                    HarnessError::Usage(format!("invalid --repeat value {value:?}"))
                })?;
                match &cli.mode {
                    Mode::Measure(file, _) => {
                        cli.mode = Mode::Measure(file.clone(), count);
                    }
                    _ => {
                        return Err(HarnessError::Usage(
                            "--repeat must follow --measure".to_owned(),
                        ));
                    }
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => {
                return Err(HarnessError::Usage(format!("unknown argument {other:?}")));
            }
        }
    }
    Ok(cli)
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, HarnessError> {
    args.next()
        .ok_or_else(|| HarnessError::Usage(format!("{flag} requires a value")))
}

fn print_help() {
    println!(
        "cosmic-diff-harness — A3 differential and measurement harness\n\n\
         Usage:\n\
         \x20 cosmic-diff-harness [--corpus <dir>] [--targets lexical,syntax,refusals,malformed]\n\
         \x20                     [--oracle <program>] [--oracle-arg <arg>]... [--oracle-pin <pin>]\n\
         \x20                     [--timeout-ms <n>] [--json <path>] [--report <path>] [--keep-temp]\n\
         \x20 cosmic-diff-harness --list\n\
         \x20 cosmic-diff-harness --self-check\n\
         \x20 cosmic-diff-harness --emit-probes <path> | --emit-probes-json <path>\n\
         \x20 cosmic-diff-harness --measure <file> [--repeat <n>]\n\n\
         Oracle policy: the legacy oracle is optional, externally obtained and never\n\
         downloaded or vendored. Without one, every comparison is an explicit skip\n\
         and the exit status is 3. A --oracle-pin value is recorded as an operator\n\
         assertion; the harness cannot verify it.\n\n\
         Exit codes: 0 compared cleanly; 1 differences need triage; 2 harness/usage\n\
         or corpus-integrity failure; 3 oracle not configured or not runnable."
    );
}
