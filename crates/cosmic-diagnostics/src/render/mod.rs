//! Rendering of diagnostics.
//!
//! Rendering is a pure function of a [`Diagnostic`](crate::Diagnostic) plus
//! the [`SourceMap`](cosmic_source::SourceMap) it points into. Two formats
//! are provided from the same data:
//!
//! * [`human`] — the rustc-style text format shown to users on a terminal;
//! * [`json`] — a deterministic single-line JSON document for machine
//!   consumers (CI tooling, editors, test harnesses).
//!
//! Both renderers are total: they never panic, and spans that cannot be
//! resolved against the given source map degrade to a byte-range fallback.

mod human;
mod json;

pub use human::render as human;
pub use json::render as json;
