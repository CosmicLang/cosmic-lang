//! The A3 differential and measurement harness for the Cosmic frontend.
//!
//! This crate implements the Stage A verification infrastructure required by
//! Increment A3: an **oracle-optional** differential harness over an
//! independently authored corpus, executed lexical probe measurements, and a
//! deterministic machine-readable summary with a human-readable report.
//!
//! # What this harness is, and is not
//!
//! * It compares the Rust frontend (`cosmic-lexer` + `cosmic-syntax`) against
//!   a legacy compiler **only when the operator explicitly configures one**
//!   (`--oracle`, or `COSMIC_LEGACY_ORACLE`). The legacy program stays
//!   external, read-only and unmodified; no legacy source, binary or output is
//!   vendored, copied or translated into this repository.
//! * Without an oracle every comparison is an explicit skip with a reason.
//!   The harness never fabricates a comparison result and never claims a
//!   pinned comparison it cannot support: a `--oracle-pin` value is recorded
//!   as an **operator assertion**, not independently verified.
//! * A legacy rejection that the corpus does not declare as a divergence is
//!   reported as an **unconfirmed difference** (it may be semantic or
//!   target-scope related), never silently reclassified as an agreement.
//!
//! # Pipeline
//!
//! 1. Load the corpus: the embedded, independently authored fixtures
//!    ([`Corpus::embedded`]) or an external directory layout
//!    ([`Corpus::from_dir`]).
//! 2. Run the Rust frontend per fixture ([`run_frontend`]) and check the
//!    corpus expectation.
//! 3. When configured, run the oracle per fixture through
//!    [`oracle::run_oracle`] with a timeout and captured output.
//! 4. Classify the pair ([`classify`]) and fold everything into a
//!    [`Summary`] (deterministic JSON plus a Markdown report).
//!
//! The binary's CLI, exit-code contract and commands are documented in the
//! crate `README.md`.

use std::fmt;

mod compare;
mod corpus;
mod frontend;
mod oracle;
mod probes;
mod run;
mod summary;

pub use compare::{Classification, DeclaredDivergence, classify};
pub use corpus::{Corpus, Expectation, Fixture, FixtureCategory};
pub use frontend::{FrontendOutcome, run_frontend};
pub use oracle::{OracleConfig, OracleOutcome, OracleStatus, run_oracle};
pub use probes::{ProbeCase, ProbeOutcome, ProbeReport, probe_report};
pub use run::{DiffOptions, run_diff};
pub use summary::{FixtureResult, Summary};

/// An error produced by the harness.
#[derive(Debug)]
pub enum HarnessError {
    /// The underlying file system or process operation failed.
    Io(std::io::Error),
    /// The corpus is malformed (bad manifest row, missing fixture file, …).
    Corpus(String),
    /// The command line or option combination is invalid.
    Usage(String),
}

impl fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HarnessError::Io(error) => write!(formatter, "i/o error: {error}"),
            HarnessError::Corpus(message) => write!(formatter, "corpus error: {message}"),
            HarnessError::Usage(message) => write!(formatter, "usage error: {message}"),
        }
    }
}

impl std::error::Error for HarnessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HarnessError::Io(error) => Some(error),
            HarnessError::Corpus(_) | HarnessError::Usage(_) => None,
        }
    }
}

impl From<std::io::Error> for HarnessError {
    fn from(error: std::io::Error) -> Self {
        HarnessError::Io(error)
    }
}
