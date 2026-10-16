//! Entry point of the `cosmic` executable.
//!
//! All behavior lives in the `cosmic_cli` library so it can be tested
//! in-process; this binary only wires up the standard streams and the
//! process exit code.

use std::io::Write;

fn main() {
    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();

    let code = cosmic_cli::run(std::env::args().skip(1), &mut stdout, &mut stderr);

    // Explicit flush; write errors are ignored on purpose (e.g. a closed
    // pipe from `cosmic --help | head` must not turn into a spurious
    // failure exit code).
    let _ = stdout.flush();
    let _ = stderr.flush();

    std::process::exit(i32::from(code));
}
