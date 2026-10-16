//! Structured diagnostics for the Cosmic-Lang compiler (`cosmcc-rs`).
//!
//! A [`Diagnostic`] is *data*: severity, a stable [`DiagnosticCode`], a
//! human-readable message, one required primary label, optional secondary
//! labels, notes and machine-applicable suggestions. Rendering is a separate
//! step and lives in [`render`] — the human format and the machine-readable
//! JSON format are produced from the same data.
//!
//! # Design rules
//!
//! * **No panics on user input.** Constructing and rendering diagnostics
//!   never panics, even for spans that do not belong to the rendering
//!   [`SourceMap`](cosmic_source::SourceMap): rendering degrades to a
//!   byte-range fallback instead.
//! * **Stable codes.** Codes are part of the observable interface (docs,
//!   tests, tooling). See [`DiagnosticCode`] for the format and the
//!   stability policy.
//! * **Deterministic ordering.** [`DiagnosticBag::sorted`] defines one
//!   canonical order for reporting; insertion order is preserved for ties.
//!
//! # Example
//!
//! ```
//! use cosmic_diagnostics::{Diagnostic, DiagnosticCode, Label, render};
//! use cosmic_source::SourceMap;
//!
//! let mut sources = SourceMap::new();
//! let file = sources.add("demo.csmx", "let x = ")?;
//! let source = sources.get(file).expect("just registered");
//! let span = source.span(8, 8).expect("valid offset");
//!
//! let diagnostic = Diagnostic::error(
//!     DiagnosticCode::new("CSM001"),
//!     "expected expression",
//!     Label::new(span, "expected an expression here"),
//! )
//! .with_note("the right-hand side of `=` must be an expression");
//!
//! let text = render::human(&diagnostic, &sources);
//! assert!(text.contains("error[CSM001]: expected expression"));
//! assert!(text.contains("--> demo.csmx:1:9"));
//! # Ok::<(), cosmic_source::SourceMapError>(())
//! ```
#![forbid(unsafe_code)]

mod bag;
mod code;
mod diagnostic;

pub mod render;

pub use bag::DiagnosticBag;
pub use code::{DiagnosticCode, InvalidCode};
pub use diagnostic::{Applicability, Diagnostic, Label, Severity, Suggestion};
