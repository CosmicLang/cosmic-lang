//! The child-process oracle runner: explicit configuration, timeout,
//! captured output, and execution-failure classification.
//!
//! The oracle program is supplied by the operator. This module never looks for
//! one implicitly, never downloads anything, and never assumes that a
//! historical probe result implies a present executable.

use std::io::Read as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How the configured oracle behaved on one input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OracleStatus {
    /// No oracle was configured; every comparison is an explicit skip.
    NotConfigured,
    /// The oracle exited successfully (acceptance for this run).
    Accepted,
    /// The oracle exited non-zero (the raw code, or `None` on a signal).
    Rejected(Option<i32>),
    /// The oracle exceeded the configured timeout and was killed.
    TimedOut,
    /// The oracle could not be spawned or waited on.
    SpawnFailed(String),
}

impl OracleStatus {
    /// The stable label used in summaries.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            OracleStatus::NotConfigured => "not-configured",
            OracleStatus::Accepted => "accepted",
            OracleStatus::Rejected(_) => "rejected",
            OracleStatus::TimedOut => "timed-out",
            OracleStatus::SpawnFailed(_) => "spawn-failed",
        }
    }

    /// The exit code, when the process produced one.
    #[must_use]
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            OracleStatus::Accepted => Some(0),
            OracleStatus::Rejected(code) => *code,
            OracleStatus::NotConfigured | OracleStatus::TimedOut | OracleStatus::SpawnFailed(_) => {
                None
            }
        }
    }
}

/// An explicitly configured oracle program.
#[derive(Clone, Debug)]
pub struct OracleConfig {
    /// The program to execute (also settable through `COSMIC_LEGACY_ORACLE`).
    pub program: String,
    /// Fixed arguments; an argument containing `{input}` receives the fixture
    /// path substituted in place. When no argument contains `{input}`, the
    /// fixture path is appended as the final argument.
    pub args: Vec<String>,
    /// Wall-clock limit for one invocation.
    pub timeout: Duration,
    /// An operator-provided pin assertion (for example the legacy commit
    /// SHA). Recorded verbatim; the harness cannot verify it.
    pub pin: Option<String>,
}

/// The oracle's observable result for one input.
#[derive(Clone, Debug)]
pub struct OracleOutcome {
    /// The classified status.
    pub status: OracleStatus,
    /// Wall-clock duration of the invocation.
    pub duration_ms: u64,
    /// Captured standard output, capped at 4096 bytes.
    pub stdout: String,
    /// Captured standard error, capped at 4096 bytes.
    pub stderr: String,
}

/// How many bytes of oracle output are retained per stream.
pub const CAPTURE_LIMIT: u64 = 4096;

/// Runs the configured oracle on `input`.
#[must_use]
pub fn run_oracle(config: &OracleConfig, input: &Path) -> OracleOutcome {
    let mut command = Command::new(&config.program);
    let input_text = input.to_string_lossy();
    let mut substituted = false;
    for argument in &config.args {
        if argument.contains("{input}") {
            command.arg(argument.replace("{input}", input_text.as_ref()));
            substituted = true;
        } else {
            command.arg(argument);
        }
    }
    if !substituted {
        command.arg(input);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let start = Instant::now();
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return OracleOutcome {
                status: OracleStatus::SpawnFailed(error.to_string()),
                duration_ms: elapsed_ms(start),
                stdout: String::new(),
                stderr: String::new(),
            };
        }
    };
    let stdout_thread = child.stdout.take().map(read_capped);
    let stderr_thread = child.stderr.take().map(read_capped);

    let mut wait_error = None;
    let mut exit_status = None;
    let mut finished = false;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                exit_status = Some(status);
                finished = true;
                break;
            }
            Ok(None) => {
                if start.elapsed() >= config.timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(error) => {
                wait_error = Some(error.to_string());
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
        }
    }

    let stdout = finish_reader(stdout_thread);
    let stderr = finish_reader(stderr_thread);
    let status = if let Some(error) = wait_error {
        OracleStatus::SpawnFailed(error)
    } else if !finished {
        OracleStatus::TimedOut
    } else {
        match exit_status {
            Some(status) if status.success() => OracleStatus::Accepted,
            Some(status) => OracleStatus::Rejected(status.code()),
            None => OracleStatus::SpawnFailed("the oracle exited without a status".to_owned()),
        }
    };
    OracleOutcome {
        status,
        duration_ms: elapsed_ms(start),
        stdout,
        stderr,
    }
}

fn elapsed_ms(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Reads at most [`CAPTURE_LIMIT`] bytes into a string, then drains the rest so
/// a chatty oracle can never block on a full pipe.
fn read_capped(mut stream: impl std::io::Read + Send + 'static) -> std::thread::JoinHandle<String> {
    std::thread::spawn(move || {
        let mut buffer = Vec::new();
        {
            let mut limited = (&mut stream).take(CAPTURE_LIMIT);
            let _ = limited.read_to_end(&mut buffer);
        }
        let _ = std::io::copy(&mut stream, &mut std::io::sink());
        String::from_utf8_lossy(&buffer).into_owned()
    })
}

/// Collects a reader thread with a bounded grace period.
///
/// A killed oracle can leave descendants holding the pipe write ends, so the
/// readers would block far past the configured timeout; after the grace
/// period the thread is detached and the captured output is empty.
fn finish_reader(handle: Option<std::thread::JoinHandle<String>>) -> String {
    let Some(handle) = handle else {
        return String::new();
    };
    let deadline = Instant::now() + READER_GRACE;
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            drop(handle);
            return String::new();
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    handle.join().unwrap_or_default()
}

/// How long a stream reader may keep waiting after the child is gone.
const READER_GRACE: Duration = Duration::from_secs(1);
