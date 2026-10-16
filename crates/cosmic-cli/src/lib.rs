//! Command-line surface of the `cosmic` executable (Cosmic-Lang compiler,
//! `cosmcc-rs`).
//!
//! # Phase 0 scope
//!
//! Exactly two capabilities exist in this build: `--help` and `--version`
//! (plus the equivalent `help` / `version` words). Everything else fails
//! loudly:
//!
//! * a *planned* command (`check`, `build`, `run`, `test`, `fmt`, `doc`,
//!   `lsp`) exits with code 2 and says it is not implemented yet;
//! * anything unknown is reported as unrecognized, also with exit code 2.
//!
//! No command reports success without performing its documented operation.
//!
//! The entry point is [`run`], a pure function over argument strings and
//! output streams, so the CLI behavior is unit-testable without spawning a
//! process.
#![forbid(unsafe_code)]

use std::io::Write;

/// Version of the `cosmic` executable.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit code for usage errors, unknown commands and unimplemented commands.
pub const EXIT_USAGE: u8 = 2;

/// Commands that are part of the planned CLI but are not implemented in this
/// Phase 0 build. Advertised only as *not implemented*.
pub const PLANNED_COMMANDS: &[&str] = &["check", "build", "run", "test", "fmt", "doc", "lsp"];

const HELP: &str = "\
cosmic — Cosmic-Lang compiler driver (cosmcc-rs)

Usage: cosmic [OPTIONS]

Options:
  -h, --help     Print this help and exit
  -V, --version  Print version information and exit

Commands:
  help           Print this help and exit
  version        Print version information and exit

Planned but NOT implemented in this Phase 0 build:
  check  build  run  test  fmt  doc  lsp
These commands fail with a clear error and exit code 2. No command reports
success without performing its documented operation.

Exit codes:
  0  success
  2  usage error, unknown command, or unimplemented command
  (1 is reserved for compilation failures once compilation exists)
";

/// Runs the CLI with the given arguments (without the program name) and
/// writes results to `stdout`/`stderr`. Returns the process exit code.
///
/// This function never panics and never exits the process itself.
pub fn run<I, S>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args
        .into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .collect();

    match args.as_slice() {
        [] => {
            let _ = writeln!(stderr, "error: no arguments given");
            let _ = writeln!(stderr, "hint: run `cosmic --help` for usage");
            EXIT_USAGE
        }
        [arg] => dispatch(arg, stdout, stderr),
        [first, rest @ ..] => {
            match first.as_str() {
                "-h" | "--help" | "help" | "-V" | "--version" | "version" => {
                    // The extra argument is `rest[0]`; `rest` is non-empty here.
                    let extra = rest.first().map(String::as_str).unwrap_or_default();
                    let _ = writeln!(
                        stderr,
                        "error: unexpected argument `{extra}` after `{first}`"
                    );
                    let _ = writeln!(stderr, "hint: run `cosmic --help` for usage");
                    EXIT_USAGE
                }
                command if command.starts_with('-') => {
                    let _ = writeln!(stderr, "error: unknown option `{command}`");
                    let _ = writeln!(stderr, "hint: run `cosmic --help` for usage");
                    EXIT_USAGE
                }
                command if PLANNED_COMMANDS.contains(&command) => {
                    report_not_implemented(command, stderr);
                    EXIT_USAGE
                }
                command => {
                    let _ = writeln!(stderr, "error: unrecognized command `{command}`");
                    let _ = writeln!(stderr, "hint: run `cosmic --help` for usage");
                    EXIT_USAGE
                }
            }
        }
    }
}

fn dispatch(arg: &str, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    match arg {
        "-h" | "--help" | "help" => {
            let _ = stdout.write_all(HELP.as_bytes());
            0
        }
        "-V" | "--version" | "version" => {
            let _ = writeln!(stdout, "cosmic {VERSION} (cosmcc-rs)");
            0
        }
        command if command.starts_with('-') => {
            let _ = writeln!(stderr, "error: unknown option `{command}`");
            let _ = writeln!(stderr, "hint: run `cosmic --help` for usage");
            EXIT_USAGE
        }
        command if PLANNED_COMMANDS.contains(&command) => {
            report_not_implemented(command, stderr);
            EXIT_USAGE
        }
        command => {
            let _ = writeln!(stderr, "error: unrecognized command `{command}`");
            let _ = writeln!(stderr, "hint: run `cosmic --help` for usage");
            EXIT_USAGE
        }
    }
}

fn report_not_implemented(command: &str, stderr: &mut dyn Write) {
    let _ = writeln!(stderr, "error: command `{command}` is not implemented yet");
    let _ = writeln!(
        stderr,
        "hint: this Phase 0 build implements only `--help` and `--version`; \
         run `cosmic --help` for the current command surface"
    );
}
