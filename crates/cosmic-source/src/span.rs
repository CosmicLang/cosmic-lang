use core::fmt;

use crate::FileId;

/// A byte range `[start, end)` inside one source file.
///
/// # Invariants
///
/// Every `Span` value produced by this crate is valid for the file identified
/// by [`Span::file`]:
///
/// * `0 <= start <= end <= text.len()` (byte offsets);
/// * both bounds lie on UTF-8 character boundaries of the file text.
///
/// The invariant is enforced by the checked constructors
/// [`SourceFile::span`](crate::SourceFile::span) and
/// [`SourceFile::full_span`](crate::SourceFile::full_span); `Span` cannot be
/// constructed outside this crate, so any `Span` value can be trusted.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span {
    file: FileId,
    start: u32,
    end: u32,
}

impl Span {
    /// Builds a span without validating it against the file.
    ///
    /// Crate-internal: callers must uphold the [`Span`] invariants.
    pub(crate) const fn new_unchecked(file: FileId, start: u32, end: u32) -> Self {
        Self { file, start, end }
    }

    /// The file this span refers to.
    #[must_use]
    pub const fn file(self) -> FileId {
        self.file
    }

    /// Byte offset of the first byte covered by this span.
    #[must_use]
    pub const fn start(self) -> u32 {
        self.start
    }

    /// Byte offset one past the last byte covered by this span.
    #[must_use]
    pub const fn end(self) -> u32 {
        self.end
    }

    /// Number of bytes covered by this span.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    /// Whether this span covers zero bytes (a caret position).
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Span({}:{}..{})", self.file, self.start, self.end)
    }
}

/// The ways a byte range can be rejected as invalid for a source file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SpanError {
    /// `start > end`.
    StartAfterEnd {
        /// The rejected start offset.
        start: u32,
        /// The rejected end offset.
        end: u32,
    },
    /// The offset is beyond the end of the file text.
    OutOfBounds {
        /// The rejected offset.
        offset: u32,
        /// Byte length of the file text.
        len: u32,
    },
    /// The offset falls inside a multibyte UTF-8 sequence.
    NotCharBoundary {
        /// The rejected offset.
        offset: u32,
    },
}

impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SpanError::StartAfterEnd { start, end } => {
                write!(f, "span start {start} is after span end {end}")
            }
            SpanError::OutOfBounds { offset, len } => {
                write!(f, "offset {offset} is out of bounds for a {len}-byte file")
            }
            SpanError::NotCharBoundary { offset } => {
                write!(f, "offset {offset} is not a UTF-8 character boundary")
            }
        }
    }
}

impl std::error::Error for SpanError {}
