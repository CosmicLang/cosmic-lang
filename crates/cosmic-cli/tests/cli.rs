//! Integration tests for the `cosmic` CLI surface.
//!
//! The CLI is exercised through [`cosmic_cli::run`], so these tests cover
//! the real dispatch logic without spawning a subprocess.

use cosmic_cli::{EXIT_USAGE, PLANNED_COMMANDS, VERSION, run};

fn run_cli(args: &[&str]) -> (u8, String, String) {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = run(args.iter().copied(), &mut stdout, &mut stderr);
    (
        code,
        String::from_utf8(stdout).expect("stdout is UTF-8"),
        String::from_utf8(stderr).expect("stderr is UTF-8"),
    )
}

#[test]
fn help_flags_print_usage_to_stdout_and_succeed() {
    for arg in ["--help", "-h", "help"] {
        let (code, stdout, stderr) = run_cli(&[arg]);
        assert_eq!(code, 0, "{arg} must succeed");
        assert!(stderr.is_empty(), "{arg} must not write to stderr");
        assert!(stdout.contains("Usage: cosmic [OPTIONS]"), "got:\n{stdout}");
        assert!(stdout.contains("--version"), "got:\n{stdout}");
    }
}

#[test]
fn help_documents_that_planned_commands_are_not_implemented() {
    let (_, stdout, _) = run_cli(&["--help"]);
    assert!(
        stdout.contains("Planned but NOT implemented"),
        "help must not advertise unimplemented commands as available; got:\n{stdout}"
    );
    for command in PLANNED_COMMANDS {
        assert!(stdout.contains(command), "help must mention `{command}`");
    }
}

#[test]
fn version_flags_print_version_to_stdout_and_succeed() {
    let expected = format!("cosmic {VERSION} (cosmcc-rs)\n");
    for arg in ["--version", "-V", "version"] {
        let (code, stdout, stderr) = run_cli(&[arg]);
        assert_eq!(code, 0, "{arg} must succeed");
        assert_eq!(stdout, expected, "{arg} must print exactly the version");
        assert!(stderr.is_empty(), "{arg} must not write to stderr");
    }
}

#[test]
fn no_arguments_is_a_usage_error() {
    let (code, stdout, stderr) = run_cli(&[]);
    assert_eq!(code, EXIT_USAGE);
    assert!(stdout.is_empty(), "nothing to report on stdout");
    assert!(stderr.contains("no arguments"), "got:\n{stderr}");
}

#[test]
fn planned_commands_fail_loudly_and_never_report_success() {
    for command in PLANNED_COMMANDS {
        let (code, stdout, stderr) = run_cli(&[command]);
        assert_eq!(code, EXIT_USAGE, "`{command}` must fail");
        assert!(stdout.is_empty(), "`{command}` must not claim success");
        assert!(
            stderr.contains(&format!("command `{command}` is not implemented yet")),
            "got:\n{stderr}"
        );
    }
}

#[test]
fn planned_commands_with_arguments_still_fail_loudly() {
    let (code, stdout, stderr) = run_cli(&["check", "--strict", "src/main.csmx"]);
    assert_eq!(code, EXIT_USAGE);
    assert!(stdout.is_empty());
    assert!(stderr.contains("not implemented yet"), "got:\n{stderr}");
}

#[test]
fn unknown_commands_are_reported_with_their_name() {
    let (code, stdout, stderr) = run_cli(&["frobnicate"]);
    assert_eq!(code, EXIT_USAGE);
    assert!(stdout.is_empty());
    assert!(
        stderr.contains("unrecognized command `frobnicate`"),
        "got:\n{stderr}"
    );
}

#[test]
fn unknown_options_are_reported_with_their_name() {
    for arg in ["--frobnicate", "-x"] {
        let (code, stdout, stderr) = run_cli(&[arg]);
        assert_eq!(code, EXIT_USAGE);
        assert!(stdout.is_empty());
        assert!(
            stderr.contains(&format!("unknown option `{arg}`")),
            "got:\n{stderr}"
        );
    }
}

#[test]
fn global_flags_reject_trailing_arguments() {
    let (code, stdout, stderr) = run_cli(&["--help", "extra"]);
    assert_eq!(code, EXIT_USAGE);
    assert!(stdout.is_empty());
    assert!(
        stderr.contains("unexpected argument `extra`"),
        "got:\n{stderr}"
    );

    let (code, _, stderr) = run_cli(&["--version", "--help"]);
    assert_eq!(code, EXIT_USAGE);
    assert!(
        stderr.contains("unexpected argument `--help`"),
        "got:\n{stderr}"
    );
}

#[test]
fn error_output_stays_on_stderr_for_every_failure_mode() {
    // A broad sweep: no failing invocation may write a "success-looking"
    // payload to stdout.
    for args in [
        vec![],
        vec!["nonsense"],
        vec!["--nonsense"],
        vec!["build"],
        vec!["lsp", "--stdio"],
    ] {
        let (code, stdout, _) = run_cli(&args);
        assert_eq!(code, EXIT_USAGE);
        assert!(stdout.is_empty(), "args {args:?} wrote to stdout");
    }
}
