use core::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use crate::file::SourceFile;

/// Stable identifier of a source file registered in a [`SourceMap`].
///
/// `FileId`s are assigned in registration order starting at 1, are never
/// reused inside a map, and are stable for the lifetime of the map. They are
/// session-local handles, not persisted identifiers: the same file gets the
/// same `FileId` across runs only if it is registered in the same order.
///
/// The encoding is a `NonZeroU32`; `FileId` deliberately exposes no raw
/// constructor besides [`FileId::from_raw`] (which validates).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(NonZeroU32);

impl FileId {
    /// Rebuilds a `FileId` from its raw representation, or `None` for 0 /
    /// out-of-range values.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match NonZeroU32::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    /// Raw representation (always `>= 1`).
    #[must_use]
    pub const fn to_raw(self) -> u32 {
        self.0.get()
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file#{}", self.0)
    }
}

impl fmt::Debug for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileId({})", self.0)
    }
}

/// Largest byte length a source file may have so every offset fits in `u32`.
/// The text itself must stay below `u32::MAX` bytes because the offset one
/// past the end (`len`) is a valid position and must also fit.
const MAX_SOURCE_LEN: usize = u32::MAX as usize - 1;

/// Registry of source files for one compilation session.
///
/// Registration order is the identity: the first [`SourceMap::add`] call
/// returns `FileId` 1, the second `FileId` 2, and so on. Lookup by
/// [`FileId`] is O(1) and deterministic.
#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

/// Reasons a source file cannot be registered.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SourceMapError {
    /// The text cannot be addressed by `u32` offsets: it is longer than
    /// 4 GiB − 1 bytes (`u32::MAX - 1`).
    SourceTooLarge {
        /// Registered (or intended) path of the file.
        path: PathBuf,
        /// Byte length of the text.
        len: usize,
    },
    /// The map already holds the maximum number of files (`u32::MAX`).
    TooManyFiles,
}

impl fmt::Display for SourceMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceMapError::SourceTooLarge { path, len } => write!(
                f,
                "source file {path:?} is too large to index: {len} bytes exceeds the 4 GiB (u32) limit",
                path = path.display(),
            ),
            SourceMapError::TooManyFiles => write!(
                f,
                "source map is full: at most {} files can be registered",
                u32::MAX
            ),
        }
    }
}

impl std::error::Error for SourceMapError {}

/// Reasons [`SourceMap::load`] failed.
#[derive(Debug)]
pub enum LoadError {
    /// The file could not be read.
    Io {
        /// Path that was read.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// The file is not valid UTF-8.
    NotUtf8 {
        /// Path that was read.
        path: PathBuf,
        /// Offset of the first invalid byte.
        valid_up_to: usize,
    },
    /// The file is too large to index (see [`SourceMapError::SourceTooLarge`]).
    TooLarge {
        /// Path that was read.
        path: PathBuf,
        /// Byte length of the file.
        len: usize,
    },
    /// The map is full (see [`SourceMapError::TooManyFiles`]).
    TooManyFiles,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io { path, source } => {
                write!(f, "cannot read {}: {source}", path.display())
            }
            LoadError::NotUtf8 { path, valid_up_to } => write!(
                f,
                "{} is not valid UTF-8 (first invalid byte at offset {valid_up_to})",
                path.display()
            ),
            LoadError::TooLarge { path, len } => write!(
                f,
                "{} is too large to index: {len} bytes exceeds the 4 GiB (u32) limit",
                path.display()
            ),
            LoadError::TooManyFiles => write!(
                f,
                "source map is full: at most {} files can be registered",
                u32::MAX
            ),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl SourceMap {
    /// Creates an empty map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `text` under `path` and returns its [`FileId`].
    ///
    /// Registering the same path twice yields two distinct `FileId`s; the
    /// map deliberately performs no path-based deduplication, so identity is
    /// always explicit.
    ///
    /// # Errors
    ///
    /// Fails with [`SourceMapError::SourceTooLarge`] for texts of
    /// `u32::MAX` bytes or more, and with [`SourceMapError::TooManyFiles`]
    /// when the map cannot assign another id.
    pub fn add(
        &mut self,
        path: impl Into<PathBuf>,
        text: impl Into<String>,
    ) -> Result<FileId, SourceMapError> {
        let path = path.into();
        let text = text.into();
        check_size(text.len(), &path)?;
        let raw = u32::try_from(self.files.len())
            .ok()
            .and_then(|count| count.checked_add(1))
            .ok_or(SourceMapError::TooManyFiles)?;
        let id = FileId::from_raw(raw).ok_or(SourceMapError::TooManyFiles)?;
        self.files.push(SourceFile::new(id, path, text));
        Ok(id)
    }

    /// Reads `path` from disk and registers it.
    ///
    /// # Errors
    ///
    /// See [`LoadError`]: I/O failures, invalid UTF-8, oversized files and a
    /// full map are all reported explicitly.
    pub fn load(&mut self, path: &Path) -> Result<FileId, LoadError> {
        let bytes = std::fs::read(path).map_err(|source| LoadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if bytes.len() > MAX_SOURCE_LEN {
            return Err(LoadError::TooLarge {
                path: path.to_path_buf(),
                len: bytes.len(),
            });
        }
        let text = String::from_utf8(bytes).map_err(|error| LoadError::NotUtf8 {
            path: path.to_path_buf(),
            valid_up_to: error.utf8_error().valid_up_to(),
        })?;
        self.add(path, text).map_err(|error| match error {
            SourceMapError::SourceTooLarge { path, len } => LoadError::TooLarge { path, len },
            SourceMapError::TooManyFiles => LoadError::TooManyFiles,
        })
    }

    /// Looks up a file by id. Returns `None` when the numeric id is out of
    /// range for this map.
    ///
    /// Ids are session-local: a `FileId` produced by a *different* map is
    /// resolved by numeric value and may name an unrelated file. Keep exactly
    /// one `SourceMap` per compilation session and never mix ids across maps.
    #[must_use]
    pub fn get(&self, id: FileId) -> Option<&SourceFile> {
        let index = id.to_raw().checked_sub(1)?;
        self.files.get(index as usize)
    }

    /// Number of registered files.
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether no file is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Iterates over registered files in registration order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &SourceFile> {
        self.files.iter()
    }
}

/// Rejects texts that cannot be addressed with `u32` byte offsets.
fn check_size(len: usize, path: &Path) -> Result<(), SourceMapError> {
    if len > MAX_SOURCE_LEN {
        return Err(SourceMapError::SourceTooLarge {
            path: path.to_path_buf(),
            len,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MAX_SOURCE_LEN, check_size};
    use crate::SourceMapError;
    use std::path::Path;

    #[test]
    fn size_limit_accepts_maximum_and_rejects_the_rest() {
        let path = Path::new("huge.csmx");
        assert_eq!(check_size(0, path), Ok(()));
        assert_eq!(check_size(1024, path), Ok(()));
        assert_eq!(check_size(MAX_SOURCE_LEN, path), Ok(()));

        for len in [MAX_SOURCE_LEN + 1, u32::MAX as usize, usize::MAX] {
            match check_size(len, path) {
                Err(SourceMapError::SourceTooLarge { len: got, .. }) => assert_eq!(got, len),
                other => panic!("expected SourceTooLarge for len {len}, got {other:?}"),
            }
        }
    }
}
