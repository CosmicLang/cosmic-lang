//! Public-API tests for `cosmic-source`.
//!
//! Coverage: empty files, Unicode (multibyte and astral), CRLF and lone-CR
//! line terminators, invalid offsets, character-boundary violations, span
//! constructors, cross-file span rejection, file loading (valid, invalid
//! UTF-8, missing) and registry determinism.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use cosmic_source::{FileId, LineCol, LoadError, SourceMap, SpanError};

/// Registers `text` under a synthetic path.
fn file(text: &str) -> (SourceMap, cosmic_source::FileId) {
    let mut sources = SourceMap::new();
    let id = sources.add("test.csmx", text).expect("small text");
    (sources, id)
}

// ---------------------------------------------------------------------------
// Spans
// ---------------------------------------------------------------------------

#[test]
fn empty_file_has_an_empty_full_span() {
    let (sources, id) = file("");
    let source = sources.get(id).unwrap();
    assert_eq!(source.len(), 0);
    assert!(source.is_empty());

    let span = source.full_span();
    assert!(span.is_empty());
    assert_eq!(span.start(), 0);
    assert_eq!(span.end(), 0);
    assert_eq!(source.text_of(span), Some(""));

    assert_eq!(source.span(0, 0), Ok(span));
    assert_eq!(
        source.span(0, 1),
        Err(SpanError::OutOfBounds { offset: 1, len: 0 })
    );
}

#[test]
fn checked_span_constructors_accept_ascii_ranges() {
    let (sources, id) = file("abcdef");
    let source = sources.get(id).unwrap();

    let span = source.span(2, 5).unwrap();
    assert_eq!(span.file(), id);
    assert_eq!((span.start(), span.end(), span.len()), (2, 5, 3));
    assert!(!span.is_empty());
    assert_eq!(source.text_of(span), Some("cde"));

    assert_eq!(source.text_of(source.full_span()), Some("abcdef"));
}

#[test]
fn span_constructor_rejects_start_after_end_first() {
    let (sources, id) = file("abcdef");
    let source = sources.get(id).unwrap();
    assert_eq!(
        source.span(5, 2),
        Err(SpanError::StartAfterEnd { start: 5, end: 2 })
    );
}

#[test]
fn span_constructor_rejects_out_of_bounds_offsets() {
    let (sources, id) = file("abc");
    let source = sources.get(id).unwrap();
    assert_eq!(
        source.span(0, 4),
        Err(SpanError::OutOfBounds { offset: 4, len: 3 })
    );
    let eof = source.span(3, 3).unwrap();
    assert!(eof.is_empty(), "one past the end is a valid empty span");
    assert_eq!((eof.start(), eof.end()), (3, 3));
    assert_eq!(
        source.span(4, 4),
        Err(SpanError::OutOfBounds { offset: 4, len: 3 })
    );
}

#[test]
fn span_constructor_rejects_offsets_inside_multibyte_characters() {
    // "é" occupies bytes 1..3; offsets 1 and 3 are boundaries, 2 is not.
    let (sources, id) = file("héllo");
    let source = sources.get(id).unwrap();

    assert_eq!(source.text_of(source.span(1, 3).unwrap()), Some("é"));
    assert_eq!(
        source.span(0, 2),
        Err(SpanError::NotCharBoundary { offset: 2 })
    );
    assert_eq!(
        source.span(2, 4),
        Err(SpanError::NotCharBoundary { offset: 2 })
    );
}

#[test]
fn astral_characters_are_single_boundaries() {
    // "a😀b": 'a' = 0, '😀' = 1..5 (4 bytes), 'b' = 5.
    let (sources, id) = file("a😀b");
    let source = sources.get(id).unwrap();

    let emoji = source.span(1, 5).unwrap();
    assert_eq!(source.text_of(emoji), Some("😀"));
    for inside in 2..=4 {
        assert_eq!(
            source.span(0, inside),
            Err(SpanError::NotCharBoundary { offset: inside }),
        );
    }
    assert_eq!(source.text_of(source.span(5, 6).unwrap()), Some("b"));
}

#[test]
fn spans_cannot_be_applied_to_other_files() {
    let mut sources = SourceMap::new();
    let first = sources.add("a.csmx", "alpha").unwrap();
    let second = sources.add("b.csmx", "beta").unwrap();

    let span_in_first = sources.get(first).unwrap().span(0, 2).unwrap();
    let other = sources.get(second).unwrap();
    assert_eq!(other.text_of(span_in_first), None);

    let span_in_second = other.span(0, 4).unwrap();
    assert_eq!(sources.get(first).unwrap().text_of(span_in_second), None);
}

// ---------------------------------------------------------------------------
// Line/column mapping
// ---------------------------------------------------------------------------

fn loc(line: u32, column: u32) -> LineCol {
    LineCol { line, column }
}

#[test]
fn empty_file_maps_to_line_one_column_one() {
    let (sources, id) = file("");
    let source = sources.get(id).unwrap();

    assert_eq!(source.line_count(), 1);
    assert_eq!(source.line_index(0), Some(0));
    assert_eq!(source.line_col(0), Some(loc(1, 1)));
    assert_eq!(source.line_text(0), Some(""));
    assert_eq!(source.line_text(1), None);
    assert_eq!(source.line_index(1), None);
    assert_eq!(source.line_col(1), None);
}

#[test]
fn single_line_mapping_without_terminator() {
    let (sources, id) = file("abc");
    let source = sources.get(id).unwrap();

    assert_eq!(source.line_count(), 1);
    assert_eq!(source.line_col(0), Some(loc(1, 1)));
    assert_eq!(source.line_col(2), Some(loc(1, 3)));
    assert_eq!(
        source.line_col(3),
        Some(loc(1, 4)),
        "one past the last char"
    );
    assert_eq!(source.line_col(4), None);
    assert_eq!(source.line_index(3), Some(0));
    assert_eq!(source.line_text(0), Some("abc"));
}

#[test]
fn lf_lines_are_indexed_like_editors_show_them() {
    let (sources, id) = file("a\nb\nc");
    let source = sources.get(id).unwrap();

    assert_eq!(source.line_count(), 3);
    assert_eq!(source.line_text(0), Some("a"));
    assert_eq!(source.line_text(1), Some("b"));
    assert_eq!(source.line_text(2), Some("c"));
    assert_eq!(source.line_col(2), Some(loc(2, 1)));
    assert_eq!(source.line_col(4), Some(loc(3, 1)));
    assert_eq!(source.line_col(5), Some(loc(3, 2)));
    assert_eq!(
        source.line_index(1),
        Some(0),
        "the newline itself is line 1"
    );
}

#[test]
fn trailing_newline_opens_a_final_empty_line() {
    let (sources, id) = file("a\n");
    let source = sources.get(id).unwrap();

    assert_eq!(source.line_count(), 2);
    assert_eq!(source.line_text(0), Some("a"));
    assert_eq!(source.line_text(1), Some(""));
    assert_eq!(source.line_col(2), Some(loc(2, 1)));
}

#[test]
fn crlf_terminators_are_one_line_break_and_not_part_of_line_text() {
    // "a\r\nb": line 1 starts at byte 0, line 2 starts at byte 3.
    let (sources, id) = file("a\r\nb");
    let source = sources.get(id).unwrap();

    assert_eq!(source.line_count(), 2);
    assert_eq!(source.line_text(0), Some("a"));
    assert_eq!(source.line_text(1), Some("b"));
    assert_eq!(
        source.line_index(2),
        Some(0),
        "the \\n byte is still line 1"
    );
    assert_eq!(source.line_col(3), Some(loc(2, 1)));
}

#[test]
fn lone_cr_terminates_a_line() {
    let (sources, id) = file("a\rb");
    let source = sources.get(id).unwrap();

    assert_eq!(source.line_count(), 2);
    assert_eq!(source.line_text(0), Some("a"));
    assert_eq!(source.line_text(1), Some("b"));
    assert_eq!(source.line_col(2), Some(loc(2, 1)));
}

#[test]
fn crlf_only_file_has_two_empty_lines() {
    let (sources, id) = file("\r\n");
    let source = sources.get(id).unwrap();

    assert_eq!(source.line_count(), 2);
    assert_eq!(source.line_text(0), Some(""));
    assert_eq!(source.line_text(1), Some(""));
    assert_eq!(source.line_col(2), Some(loc(2, 1)));
}

#[test]
fn columns_count_characters_not_bytes() {
    // "héllo": h = 0, é = 1..3, l = 3, l = 4, o = 5.
    let (sources, id) = file("héllo");
    let source = sources.get(id).unwrap();

    assert_eq!(source.line_col(0), Some(loc(1, 1))); // h
    assert_eq!(source.line_col(1), Some(loc(1, 2))); // é
    assert_eq!(source.line_col(3), Some(loc(1, 3))); // l
    assert_eq!(source.line_col(5), Some(loc(1, 5))); // o
    assert_eq!(source.line_col(2), None, "inside é is not mappable");
}

#[test]
fn columns_count_astral_characters_as_one() {
    // "a😀b": 'a' = 0, '😀' = 1..5, 'b' = 5.
    let (sources, id) = file("a😀b");
    let source = sources.get(id).unwrap();

    assert_eq!(source.line_col(0), Some(loc(1, 1)));
    assert_eq!(source.line_col(1), Some(loc(1, 2)));
    assert_eq!(source.line_col(5), Some(loc(1, 3)), "b is column 3");
    assert_eq!(source.line_col(2), None);
}

#[test]
fn tab_counts_as_one_column() {
    let (sources, id) = file("\tx");
    let source = sources.get(id).unwrap();
    assert_eq!(source.line_col(1), Some(loc(1, 2)));
}

#[test]
fn unicode_line_terminators_do_not_break_lines() {
    // U+2028 LINE SEPARATOR is a scalar value, not a terminator for us.
    let (sources, id) = file("a\u{2028}b");
    let source = sources.get(id).unwrap();
    assert_eq!(source.line_count(), 1);
    assert_eq!(
        source.line_col(4),
        Some(loc(1, 3)),
        "U+2028 is 3 bytes, one column"
    );
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

#[test]
fn ids_are_assigned_in_registration_order_and_stay_stable() {
    let mut sources = SourceMap::new();
    assert!(sources.is_empty());

    let a = sources.add("a.csmx", "alpha").unwrap();
    let b = sources.add("b.csmx", "beta").unwrap();

    assert_eq!(a.to_raw(), 1);
    assert_eq!(b.to_raw(), 2);
    assert_eq!(sources.len(), 2);
    assert_eq!(a, FileId::from_raw(1).unwrap());

    assert_eq!(sources.get(a).unwrap().path(), PathBuf::from("a.csmx"));
    assert_eq!(sources.get(b).unwrap().text(), "beta");

    let paths: Vec<_> = sources.iter().map(|f| f.path().to_path_buf()).collect();
    assert_eq!(
        paths,
        vec![PathBuf::from("a.csmx"), PathBuf::from("b.csmx")]
    );
}

#[test]
fn registering_the_same_path_twice_yields_distinct_ids() {
    let mut sources = SourceMap::new();
    let first = sources.add("same.csmx", "one").unwrap();
    let second = sources.add("same.csmx", "two").unwrap();

    assert_ne!(first, second);
    assert_eq!(sources.get(first).unwrap().text(), "one");
    assert_eq!(sources.get(second).unwrap().text(), "two");
}

#[test]
fn file_id_raw_round_trip_rejects_zero() {
    assert_eq!(FileId::from_raw(0), None);
    let id = FileId::from_raw(7).unwrap();
    assert_eq!(id.to_raw(), 7);
}

#[test]
fn lookup_of_foreign_ids_is_out_of_range_or_numeric() {
    let mut sources = SourceMap::new();
    let only = sources.add("a.csmx", "alpha").unwrap();

    // Out of range for this map -> None.
    assert!(sources.get(FileId::from_raw(2).unwrap()).is_none());
    // Numerically in range: resolves by value (documented session-local semantics).
    assert_eq!(sources.get(only).unwrap().path(), PathBuf::from("a.csmx"));
}

// ---------------------------------------------------------------------------
// Loading from disk
// ---------------------------------------------------------------------------

static TEMP_COUNTER: AtomicU32 = AtomicU32::new(0);

fn temp_path(label: &str) -> PathBuf {
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("cosmic-source-{}-{n}-{label}", std::process::id()))
}

#[test]
fn load_reads_valid_utf8_files() {
    let path = temp_path("valid.csmx");
    std::fs::write(&path, "let x = 1\n").unwrap();

    let mut sources = SourceMap::new();
    let id = sources.load(&path).unwrap();
    let source = sources.get(id).unwrap();
    assert_eq!(source.text(), "let x = 1\n");
    assert_eq!(source.path(), path.as_path());

    std::fs::remove_file(&path).unwrap();
}

#[test]
fn load_reports_missing_files() {
    let path = temp_path("missing.csmx");
    let _ = std::fs::remove_file(&path);

    let mut sources = SourceMap::new();
    match sources.load(&path) {
        Err(LoadError::Io { path: got, source }) => {
            assert_eq!(got, path);
            assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        }
        other => panic!("expected Io error, got {other:?}"),
    }
    assert!(
        sources.is_empty(),
        "failed loads must not register anything"
    );
}

#[test]
fn load_reports_invalid_utf8_precisely() {
    let path = temp_path("invalid.csmx");
    // "ab" followed by a stray continuation byte.
    std::fs::write(&path, [b'a', b'b', 0x80, b'c']).unwrap();

    let mut sources = SourceMap::new();
    match sources.load(&path) {
        Err(LoadError::NotUtf8 {
            path: got,
            valid_up_to,
        }) => {
            assert_eq!(got, path);
            assert_eq!(valid_up_to, 2);
        }
        other => panic!("expected NotUtf8 error, got {other:?}"),
    }

    std::fs::remove_file(&path).unwrap();
}
