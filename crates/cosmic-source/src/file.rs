use core::fmt;
use std::path::{Path, PathBuf};

use crate::FileId;
use crate::span::{Span, SpanError};

/// A 1-based line/column location inside a source file.
///
/// `line` and `column` are 1-based for direct human consumption in
/// diagnostics. `column` counts Unicode scalar values (`char`s) from the
/// start of the line: a tab or an emoji counts as one column. Byte offsets
/// remain the canonical coordinates internally; see [`SourceFile::line_col`].
///
/// Line terminators are handled uniformly: `\n`, `\r\n` and lone `\r` all
/// terminate a line (matching common editor behavior and the legacy Kof
/// lexer).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct LineCol {
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number, counted in Unicode scalar values.
    pub column: u32,
}

/// A registered source file: UTF-8 text with a stable identity and a
/// precomputed line index.
///
/// Constructed only through [`SourceMap`](crate::SourceMap), which owns all
/// `SourceFile` values and hands out [`FileId`]s.
#[derive(Debug)]
pub struct SourceFile {
    id: FileId,
    path: PathBuf,
    text: String,
    /// Byte offset of the first byte of every line. Always non-empty, with
    /// `line_starts[0] == 0`; sorted ascending. See [`compute_line_starts`].
    line_starts: Vec<u32>,
}

impl SourceFile {
    /// Creates a file record. Crate-internal: only [`SourceMap`](crate::SourceMap)
    /// may allocate [`FileId`]s, keeping the registry consistent.
    ///
    /// The caller must guarantee `text.len() < u32::MAX` so that every byte
    /// offset fits in `u32` (see [`SourceMap::add`](crate::SourceMap::add)).
    pub(crate) fn new(id: FileId, path: PathBuf, text: String) -> Self {
        let line_starts = compute_line_starts(&text);
        Self {
            id,
            path,
            text,
            line_starts,
        }
    }

    /// Stable identity of this file inside its [`SourceMap`](crate::SourceMap).
    #[must_use]
    pub fn id(&self) -> FileId {
        self.id
    }

    /// Path this file was registered under (not canonicalized).
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Full UTF-8 text of the file.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Byte length of the file text.
    #[must_use]
    pub fn len(&self) -> u32 {
        // Infallible by construction: `SourceMap::add` rejects texts whose
        // byte length does not fit in `u32`.
        u32::try_from(self.text.len()).unwrap_or(u32::MAX)
    }

    /// Whether the file text is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Number of lines. A file always has at least one line; a trailing line
    /// terminator opens one final (empty) line, so `"a\n"` has two lines.
    #[must_use]
    pub fn line_count(&self) -> u32 {
        // Infallible by construction: `compute_line_starts` pushes at most
        // one start per byte, and the text is at most `u32::MAX - 1` bytes.
        u32::try_from(self.line_starts.len()).unwrap_or(u32::MAX)
    }

    /// Zero-based index of the line containing `offset`.
    ///
    /// Returns `None` only when `offset` is beyond the end of the text. An
    /// offset inside a multibyte character still maps to its line: only
    /// [`Self::line_col`] requires character boundaries.
    #[must_use]
    pub fn line_index(&self, offset: u32) -> Option<u32> {
        if offset > self.len() {
            return None;
        }
        // Greatest index `i` with `line_starts[i] <= offset`; `line_starts[0]`
        // is always 0, so the subtraction never underflows.
        let index = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(insert) => insert.checked_sub(1)?,
        };
        u32::try_from(index).ok()
    }

    /// Maps a byte offset to a 1-based line/column position.
    ///
    /// Returns `None` when `offset` is beyond the end of the text or does not
    /// lie on a UTF-8 character boundary.
    #[must_use]
    pub fn line_col(&self, offset: u32) -> Option<LineCol> {
        if offset > self.len() || !self.text.is_char_boundary(offset as usize) {
            return None;
        }
        let line = self.line_index(offset)?;
        let start = *self.line_starts.get(line as usize)?;
        let column = self
            .text
            .get(start as usize..offset as usize)?
            .chars()
            .count();
        Some(LineCol {
            line: line + 1,
            column: u32::try_from(column).ok()?.saturating_add(1),
        })
    }

    /// Text of one line, **zero-based**, without its line terminator.
    ///
    /// `line == line_count() - 1` is the last valid line. Returns `None` for
    /// out-of-range indices.
    #[must_use]
    pub fn line_text(&self, line: u32) -> Option<&str> {
        let start = *self.line_starts.get(line as usize)? as usize;
        let end = match self.line_starts.get(line as usize + 1) {
            Some(&next) => next as usize,
            None => self.text.len(),
        };
        let mut text = self.text.get(start..end)?;
        if let Some(stripped) = text.strip_suffix("\r\n") {
            text = stripped;
        } else if let Some(stripped) = text.strip_suffix(['\n', '\r']) {
            text = stripped;
        }
        Some(text)
    }

    /// Builds a checked span `[start, end)` for this file.
    ///
    /// # Errors
    ///
    /// * [`SpanError::StartAfterEnd`] if `start > end`;
    /// * [`SpanError::OutOfBounds`] if `end` (or `start`) is beyond the text;
    /// * [`SpanError::NotCharBoundary`] if a bound splits a UTF-8 character.
    pub fn span(&self, start: u32, end: u32) -> Result<Span, SpanError> {
        if start > end {
            return Err(SpanError::StartAfterEnd { start, end });
        }
        self.check_offset(start)?;
        self.check_offset(end)?;
        Ok(Span::new_unchecked(self.id, start, end))
    }

    /// The span covering the whole file (always valid, possibly empty).
    #[must_use]
    pub fn full_span(&self) -> Span {
        Span::new_unchecked(self.id, 0, self.len())
    }

    /// Text covered by `span`, or `None` if the span belongs to a different
    /// file. (Bounds were already validated when the span was built.)
    #[must_use]
    pub fn text_of(&self, span: Span) -> Option<&str> {
        if span.file() != self.id {
            return None;
        }
        self.text.get(span.start() as usize..span.end() as usize)
    }

    fn check_offset(&self, offset: u32) -> Result<(), SpanError> {
        if offset > self.len() {
            return Err(SpanError::OutOfBounds {
                offset,
                len: self.len(),
            });
        }
        if !self.text.is_char_boundary(offset as usize) {
            return Err(SpanError::NotCharBoundary { offset });
        }
        Ok(())
    }
}

impl fmt::Display for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

/// Byte offsets of the start of every line.
///
/// A line starts after a `\n`, after a `\r\n` pair, or after a lone `\r`.
/// The result always contains at least `0` (the start of the first line) and
/// is sorted ascending.
fn compute_line_starts(text: &str) -> Vec<u32> {
    let bytes = text.as_bytes();
    let mut starts = Vec::with_capacity(1 + bytes.len() / 32);
    starts.push(0_u32);
    let mut i = 0_usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                i += 1;
                push_offset(&mut starts, i);
            }
            b'\r' => {
                i += if bytes.get(i + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
                push_offset(&mut starts, i);
            }
            _ => i += 1,
        }
    }
    starts
}

/// Pushes `offset` when it fits in `u32`. Cannot drop a value in practice:
/// the text length is bounded by `u32::MAX - 1`, so `i <= len` always fits.
fn push_offset(starts: &mut Vec<u32>, offset: usize) {
    if let Ok(offset) = u32::try_from(offset) {
        starts.push(offset);
    }
}
