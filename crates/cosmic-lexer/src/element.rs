//! The lossless element model: what lexing one file produces.

use cosmic_diagnostics::DiagnosticBag;
use cosmic_source::{FileId, SourceFile, Span};

use crate::{LexErrorCode, TokenKind};

/// The exact byte form of one line terminator (G1-L D-L4 item 4, S14 Q4).
///
/// A `CRLF` pair is **one** element of kind [`TerminatorKind::CrLf`], never a
/// `CR` element followed by an `LF` element; a lone `CR` is
/// [`TerminatorKind::Cr`]. The exact form is preserved so a reprint is
/// byte-for-byte and a later normalisation step can make its own decision.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TerminatorKind {
    /// `\n` — one byte.
    Lf,
    /// `\r\n` — two bytes, one line break.
    CrLf,
    /// `\r` alone — one byte, one line break.
    Cr,
}

impl TerminatorKind {
    /// The exact byte form of this terminator.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            TerminatorKind::Lf => "\n",
            TerminatorKind::CrLf => "\r\n",
            TerminatorKind::Cr => "\r",
        }
    }
}

/// What one element of the lossless stream is.
///
/// The elements of a [`LexedFile`] partition the whole file: tokens, trivia,
/// the optional leading BOM and error regions. Error regions hold the bytes
/// consumed by a recovery step (an unterminated comment or literal, a
/// refused triple-quoted string, or an unexpected character); no byte of the
/// file is ever dropped (G1-L D-L10).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ElementKind {
    /// The leading UTF-8 BOM (`U+FEFF`), preserved as a prefix leaf (D-L1 §4).
    ///
    /// It contributes no language token and is recognised only at byte 0.
    Bom,
    /// A run of one or more whitespace characters (`U+0020`, `U+0009`,
    /// `U+000C`; grammar §2.1).
    Whitespace,
    /// One line terminator in its exact form (D-L4; grammar §1.2).
    LineTerminator(TerminatorKind),
    /// A `//` comment, **excluding** the terminating line break (grammar §2.1).
    LineComment,
    /// A `/* … */` comment. Block comments do not nest (grammar §2.1).
    ///
    /// A closed comment is clean trivia; an unterminated one never appears as
    /// this kind — its bytes become an [`ElementKind::Error`] region carrying
    /// [`LexErrorCode::Lex001`].
    BlockComment,
    /// One language token of the given kind.
    Token(TokenKind),
    /// A region consumed by lexical-error recovery (D-L10).
    ///
    /// The bytes are preserved exactly; the attached
    /// [`LexErrorCode`]s (also in the [`LexedFile::diagnostics`]) say why.
    Error,
}

/// One element of the lossless stream.
///
/// `span` is the exact byte range of the element in its file; the boundary
/// of every span lies on a UTF-8 character boundary of the file text, so
/// [`SourceFile::text_of`] always resolves.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LexElement {
    kind: ElementKind,
    span: Span,
    errors: Vec<LexErrorCode>,
}

impl LexElement {
    /// Builds an element. Crate-internal: spans come from the scanner.
    pub(crate) fn new(kind: ElementKind, span: Span, errors: Vec<LexErrorCode>) -> Self {
        Self { kind, span, errors }
    }

    /// What this element is.
    #[must_use]
    pub fn kind(&self) -> ElementKind {
        self.kind
    }

    /// The exact byte range of this element in its file.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    /// The lexical errors attached to this element, in scan order.
    ///
    /// Empty for a clean element. Non-empty for error regions and for
    /// well-delimited but refused lexemes preserved as error-marked tokens
    /// (for example a string literal with an unknown escape).
    #[must_use]
    pub fn errors(&self) -> &[LexErrorCode] {
        &self.errors
    }

    /// Whether this element carries at least one lexical error.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// The complete lossless lexical analysis of one registered source file.
///
/// [`LexedFile::elements`] is ordered by byte position and partitions the
/// file: every element starts where the previous one ends, and the elements
/// together cover `0..len` exactly.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LexedFile {
    file: FileId,
    elements: Vec<LexElement>,
    diagnostics: DiagnosticBag,
}

impl LexedFile {
    /// Builds a lexed file. Crate-internal: produced by [`lex`](crate::lex).
    pub(crate) fn new(file: FileId, elements: Vec<LexElement>, diagnostics: DiagnosticBag) -> Self {
        Self {
            file,
            elements,
            diagnostics,
        }
    }

    /// The file this analysis belongs to.
    #[must_use]
    pub fn file(&self) -> FileId {
        self.file
    }

    /// Every element in scan order: tokens, trivia, BOM and error regions.
    #[must_use]
    pub fn elements(&self) -> &[LexElement] {
        &self.elements
    }

    /// The token elements in scan order, skipping trivia and error regions.
    pub fn tokens(&self) -> impl Iterator<Item = &LexElement> + '_ {
        self.elements
            .iter()
            .filter(|element| matches!(element.kind(), ElementKind::Token(_)))
    }

    /// The diagnostics reported by the scan, in scan order.
    ///
    /// [`DiagnosticBag::sorted`](cosmic_diagnostics::DiagnosticBag::sorted)
    /// defines the canonical reporting order.
    #[must_use]
    pub fn diagnostics(&self) -> &DiagnosticBag {
        &self.diagnostics
    }

    /// Whether the scan reported at least one error.
    ///
    /// Every lexer diagnostic is error severity, so this is equivalent to a
    /// non-empty diagnostics bag.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.error_count() > 0
    }

    /// Whether `source` is the file this analysis was produced from, and the
    /// elements partition it exactly: contiguous, gapless, no overlap.
    #[must_use]
    pub fn is_lossless(&self, source: &SourceFile) -> bool {
        if source.id() != self.file {
            return false;
        }
        let mut expected = 0_u32;
        for element in &self.elements {
            let span = element.span();
            if span.file() != self.file || span.start() != expected {
                return false;
            }
            expected = span.end();
        }
        expected == source.len()
    }

    /// Rebuilds the source text by concatenating the element bytes.
    ///
    /// Returns `None` when `source` is not the analysed file, or when an
    /// element span does not resolve in it. For the analysed file this
    /// always succeeds and reproduces the text byte-for-byte (D-L10).
    #[must_use]
    pub fn reprint(&self, source: &SourceFile) -> Option<String> {
        if source.id() != self.file {
            return None;
        }
        let mut out = String::with_capacity(source.len() as usize);
        for element in &self.elements {
            out.push_str(source.text_of(element.span())?);
        }
        Some(out)
    }
}
