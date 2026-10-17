//! Lossless lexical analysis for the Cosmic-Lang compiler (`cosmcc-rs`).
//!
//! This crate implements the accepted lexical contract of
//! `docs/spec/g1-l-lexical-decision-package.md` (decisions D-L1 … D-L10) on
//! top of the Phase 0 foundation crates. Lexing a
//! [`SourceFile`](cosmic_source::SourceFile) produces a [`LexedFile`]: a
//! **lossless stream** in which every byte of the UTF-8 source text lands in
//! exactly one element:
//!
//! * a **token** — see [`TokenKind`];
//! * a **trivia element** — a whitespace run, a line terminator of its exact
//!   byte form, a line comment or a block comment;
//! * an **error region** holding the bytes consumed by a recovery step.
//!
//! The leading UTF-8 BOM, when present, is a prefix leaf of its own and
//! contributes no language token ([`ElementKind::Bom`]; D-L1 §4). Every
//! element carries an exact byte [`Span`](cosmic_source::Span) validated by
//! [`cosmic_source`], and the elements partition the file, so a consumer can
//! reprint the source byte-for-byte with [`LexedFile::reprint`] and verify
//! the partitioning with [`LexedFile::is_lossless`].
//!
//! # Decision map (D-L1 … D-L10)
//!
//! | Decision | Implemented as |
//! |---|---|
//! | D-L1 BOM | a leading `U+FEFF` becomes one [`ElementKind::Bom`] leaf; any later `U+FEFF` is an unexpected character (`LEX005`) with a help note |
//! | D-L2 Unicode, byte spans | spans are byte ranges on character boundaries; the lexer contains no column logic |
//! | D-L3 identifiers | `char::is_alphabetic` / `char::is_alphanumeric` plus `_` and `$`; combining marks do not continue an identifier |
//! | D-L4 whitespace, terminators | whitespace runs and `LF`/`CRLF`/`CR` trivia elements, each in its exact form |
//! | D-L5 literals, escapes | strings and characters share one escape scanner ([`LexErrorCode::Lex006`] … [`LexErrorCode::Lex010`]) |
//! | D-L6 triple quotes | a `"""` sequence is consumed as an error region (`LEX009`) |
//! | D-L7 raw line breaks | raw terminators are string content with no diagnostic |
//! | D-L8 unknown escapes | strict rule: `LEX008`, with the literal preserved as an error-marked token |
//! | D-L9 surrogates | the escape scanner applies the pairing rules; lone surrogates are `LEX010` |
//! | D-L10 losslessness | [`LexedFile::is_lossless`], [`LexedFile::reprint`] and the property suite |
//!
//! # Diagnostics
//!
//! The ten `LEX` codes are all error severity, deterministic for a given
//! input and stable once released. Their registered conditions and evidence
//! tests live in `docs/spec/diagnostic-codes.md`. The lexer is **total** on
//! any registered file: malformed text produces diagnostics and error
//! regions — never a panic, never silently dropped bytes.
//!
//! # Deliberate non-goals
//!
//! * No semantic decision is taken here. Value-based integer promotion,
//!   numeric values, identifier normalisation and interning are owned by
//!   their own (gated or deferred) decision records; the lexer classifies
//!   numeric literals syntactically, by the grammar's suffix rules only.
//! * Unicode-category edges beyond the accepted classification authority
//!   (`char::is_alphabetic` / `char::is_alphanumeric`) are recorded
//!   measurement obligations (S13 M-7), not silent behaviour here.
//! * No external dependencies: `std` only.
//!
//! # Example
//!
//! ```
//! use cosmic_lexer::{ElementKind, TokenKind, lex};
//! use cosmic_source::SourceMap;
//!
//! let mut sources = SourceMap::new();
//! let file = sources.add("demo.cmic", "// note\nvar x = 1\n").unwrap();
//! let source = sources.get(file).unwrap();
//!
//! let lexed = lex(source);
//! assert!(lexed.diagnostics().is_empty());
//!
//! let kinds: Vec<TokenKind> = lexed
//!     .tokens()
//!     .map(|element| match element.kind() {
//!         ElementKind::Token(kind) => kind,
//!         other => unreachable!("tokens() yields tokens, found {other:?}"),
//!     })
//!     .collect();
//! assert_eq!(
//!     kinds,
//!     [
//!         TokenKind::Var,
//!         TokenKind::Identifier,
//!         TokenKind::Eq,
//!         TokenKind::IntLiteral,
//!     ]
//! );
//!
//! // Losslessness: the stream reprints the file byte-for-byte.
//! assert!(lexed.is_lossless(source));
//! assert_eq!(lexed.reprint(source).as_deref(), Some("// note\nvar x = 1\n"));
//! ```

#![forbid(unsafe_code)]

mod codes;
mod element;
mod escapes;
mod scanner;
mod token;

pub use codes::LexErrorCode;
pub use element::{ElementKind, LexElement, LexedFile, TerminatorKind};
pub use scanner::lex;
pub use token::TokenKind;
