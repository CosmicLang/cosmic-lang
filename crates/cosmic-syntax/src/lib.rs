//! Lossless concrete syntax trees and recovering parsing for the Cosmic-Lang
//! compiler (`cosmcc-rs`).
//!
//! Parsing a [`SourceFile`](cosmic_source::SourceFile) produces a
//! [`ParsedFile`]: the lossless [cosmic-lexer](cosmic_lexer) element stream,
//! a **concrete** syntax tree over exactly those elements, and the `PARSE`
//! diagnostics. The crate recognises the accepted recognition subset of
//! `docs/spec/g1-s-syntax-decision-package.md` §3 over the accepted grammar
//! `docs/spec/grammar.ebnf`:
//!
//! * Every `RECOGNISED` production is parsed into structural [`NodeKind`]s.
//! * The nine `UNSUPPORTED` families are **refused**: exactly one `PARSE001`
//!   diagnostic per construct, the delimited region preserved in a refusal
//!   node (never fabricated bytes, never silent acceptance).
//! * `fun`/`fn`/`func` are reserved-invalid (`SG-001`) and refused with a
//!   `PARSE002` diagnostic; a label after `break`/`continue` is refused with
//!   `PARSE004`.
//! * Malformed input is preserved: an unexpected region lands in an `Error`
//!   node with one `PARSE002`/`PARSE003` diagnostic. Parsing never panics,
//!   never loops and never drops a byte: every lexer element — tokens,
//!   trivia, the BOM and lexical error regions — becomes exactly one leaf,
//!   in order ([`ParsedFile::is_lossless`]).
//!
//! # Reconstruction
//!
//! [`ParsedFile::reprint`] concatenates the leaf bytes and reproduces the
//! input byte-for-byte, malformed or not:
//!
//! ```
//! use cosmic_source::SourceMap;
//!
//! let mut sources = SourceMap::new();
//! let file = sources.add("demo.csmx", "main() {\n  val answer = 40 + 2\n}\n")?;
//! let source = sources.get(file).expect("just registered");
//!
//! let parsed = cosmic_syntax::parse(source);
//! assert!(!parsed.has_errors());
//! assert!(parsed.is_lossless(source));
//! assert_eq!(
//!     parsed.reprint(source).as_deref(),
//!     Some("main() {\n  val answer = 40 + 2\n}\n"),
//! );
//! # Ok::<(), cosmic_source::SourceMapError>(())
//! ```
//!
//! # Determinism and diagnostics
//!
//! Disambiguation (lambda vs parenthesised expression, generic call vs
//! comparison, brace lambda vs array-literal refusal, nested function
//! declaration vs expression statement, switch-label forms) uses **bounded
//! speculative parsing with full rollback**, so the result is a deterministic
//! function of the token stream: parsing the same file twice yields identical
//! trees and diagnostics. `LEX` diagnostics are never remapped or rewritten;
//! [`ParsedFile::all_diagnostics`] merges both layers in a single
//! deterministic order.
//!
//! Code stability: the four `PARSE` codes are frozen in
//! `docs/spec/diagnostic-codes.md` §6.3 and exposed as [`ParseErrorCode`].
#![forbid(unsafe_code)]

mod codes;
mod kind;
mod parser;
mod tree;

pub use codes::ParseErrorCode;
pub use kind::NodeKind;
pub use parser::parse;
pub use tree::{Descendants, Node, NodeId, ParsedFile};
