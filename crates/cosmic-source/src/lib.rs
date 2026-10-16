//! Source management for the Cosmic-Lang compiler (`cosmcc-rs`).
//!
//! This crate owns the single source of truth for three things:
//!
//! * **Registered source files** — [`SourceMap`] assigns a stable [`FileId`]
//!   to every file, in registration order.
//! * **Byte spans** — [`Span`] is a validated `[start, end)` byte range of a
//!   UTF-8 source file. Spans can only be built through checked constructors
//!   ([`SourceFile::span`], [`SourceFile::full_span`]), so every `Span` value
//!   in the system is guaranteed to lie on character boundaries and inside
//!   the file.
//! * **Line/column mapping** — [`SourceFile::line_col`] and friends map byte
//!   offsets to human-friendly 1-based line/column positions, handling `\n`,
//!   `\r\n` and lone `\r` line terminators and multibyte UTF-8 text.
//!
//! # Guarantees and non-goals
//!
//! * No function in this crate panics on malformed *input data*: invalid
//!   offsets, out-of-range lines and invalid UTF-8 are reported through
//!   `Option`/`Result`.
//! * Offsets are **byte** offsets into UTF-8 text. Columns are counted in
//!   Unicode scalar values (`char`s), which is what humans see.
//! * Files larger than the `u32` address space (4 GiB) are rejected at
//!   registration time; all internal offsets are `u32` by design.
//!
//! # Example
//!
//! ```
//! use cosmic_source::SourceMap;
//!
//! let mut sources = SourceMap::new();
//! let file = sources.add("demo.csmx", "let x = 1\nlet y = 2\n")?;
//! let source = sources.get(file).expect("just registered");
//!
//! let offset = source.text().find("y = 2").expect("present") as u32;
//! let loc = source.line_col(offset).expect("valid offset");
//! assert_eq!((loc.line, loc.column), (2, 5));
//! # Ok::<(), cosmic_source::SourceMapError>(())
//! ```
#![forbid(unsafe_code)]

mod file;
mod map;
mod span;

pub use file::{LineCol, SourceFile};
pub use map::{FileId, LoadError, SourceMap, SourceMapError};
pub use span::{Span, SpanError};
