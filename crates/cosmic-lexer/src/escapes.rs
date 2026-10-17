//! Escape scanning for string and character literals (D-L5, D-L8, D-L9).
//!
//! One scanner serves both literal forms, so the accepted escape set, the
//! `\u` window rules and the surrogate-pairing rules cannot drift apart. The
//! scanner decodes the literal value **and** collects every problem, so it is
//! also the single source of truth for the unit tests of the D-L5/D-L8/D-L9
//! tables.

use core::ops::Range;

use crate::codes::LexErrorCode;

/// One problem found while scanning a literal interior.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct EscapeIssue {
    pub(crate) code: LexErrorCode,
    /// Byte range of the issue **within the literal interior**.
    pub(crate) range: Range<usize>,
}

/// The outcome of scanning a literal interior.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct EscapeScan {
    /// The decoded value. An escape that is refused (an issue) contributes
    /// nothing; the value of a refused literal is not meaningful, because
    /// compilation fails on the diagnostic.
    ///
    /// Read by the unit tests that pin the D-L5/D-L9 decoded-value tables;
    /// the scanner itself consumes only `issues`.
    #[allow(dead_code)]
    pub(crate) decoded: String,
    /// Issues in scan order (deterministic for any interior).
    pub(crate) issues: Vec<EscapeIssue>,
}

/// Scans `interior` — the raw text between a literal's delimiters — decoding
/// the accepted escapes and collecting every escape problem.
///
/// The scanner is total: any `&str` produces a result, and every issue has a
/// byte range inside `interior`.
pub(crate) fn scan_escapes(interior: &str) -> EscapeScan {
    let mut scan = EscapeScan {
        decoded: String::new(),
        issues: Vec::new(),
    };
    let mut i = 0_usize;
    while i < interior.len() {
        let c = char_at(interior, i);
        if c != '\\' {
            // Raw content, including raw line break bytes inside strings
            // (D-L7): value bytes are preserved exactly, without decoding.
            scan.decoded.push(c);
            i += c.len_utf8();
            continue;
        }
        let escape_start = i;
        i += 1;
        let Some(escaped) = interior.get(i..).and_then(|rest| rest.chars().next()) else {
            // Unreachable through `lex`: the delimiter scan never treats a
            // quote as closing when it is escaped, so a string interior
            // never ends with a lone backslash. Kept total regardless.
            break;
        };
        match escaped {
            'n' => {
                scan.decoded.push('\n');
                i += 1;
            }
            't' => {
                scan.decoded.push('\t');
                i += 1;
            }
            'r' => {
                scan.decoded.push('\r');
                i += 1;
            }
            '\\' => {
                scan.decoded.push('\\');
                i += 1;
            }
            '\'' => {
                scan.decoded.push('\'');
                i += 1;
            }
            '"' => {
                scan.decoded.push('"');
                i += 1;
            }
            '0' => {
                scan.decoded.push('\0');
                i += 1;
            }
            'u' => {
                i += 1;
                scan_unicode_escape(&mut scan, interior, escape_start, &mut i);
            }
            other => {
                // D-L8: strict rule. The refused escape contributes no value.
                let end = escape_start + 1 + other.len_utf8();
                scan.issues.push(EscapeIssue {
                    code: LexErrorCode::Lex008,
                    range: escape_start..end,
                });
                i = end;
            }
        }
    }
    scan
}

/// Consumes one `\u` escape at `*cursor` (just past the `u`), pushing the
/// issue or the decoded scalar. The cursor always advances to the end of the
/// four-character window (or the interior end for an incomplete window).
fn scan_unicode_escape(
    scan: &mut EscapeScan,
    interior: &str,
    escape_start: usize,
    cursor: &mut usize,
) {
    match hex_window(interior, *cursor) {
        HexWindow::Incomplete => {
            scan.issues.push(EscapeIssue {
                code: LexErrorCode::Lex006,
                range: escape_start..interior.len(),
            });
            *cursor = interior.len();
        }
        HexWindow::Invalid { end } => {
            scan.issues.push(EscapeIssue {
                code: LexErrorCode::Lex007,
                range: escape_start..end,
            });
            *cursor = end;
        }
        HexWindow::Value { unit, end } => {
            *cursor = end;
            if (0xD800..=0xDBFF).contains(&unit) {
                // A high surrogate must pair with an immediately following
                // low-surrogate `\u` escape (D-L9 item 2).
                if let Some(low) = low_surrogate_escape(interior, end) {
                    let scalar = 0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00);
                    if let Some(c) = char::from_u32(scalar) {
                        scan.decoded.push(c);
                    }
                    *cursor = end + 6;
                } else {
                    scan.issues.push(EscapeIssue {
                        code: LexErrorCode::Lex010,
                        range: escape_start..end,
                    });
                }
            } else if (0xDC00..=0xDFFF).contains(&unit) {
                scan.issues.push(EscapeIssue {
                    code: LexErrorCode::Lex010,
                    range: escape_start..end,
                });
            } else if let Some(c) = char::from_u32(unit) {
                scan.decoded.push(c);
            }
        }
    }
}

/// The outcome of reading the four-character `\u` window.
enum HexWindow {
    /// Fewer than four characters remain in the interior.
    Incomplete,
    /// Four characters are present but at least one is not a hexadecimal
    /// digit; `end` is the byte offset just past the fourth window character.
    Invalid { end: usize },
    /// Four hexadecimal digits; `end` is the byte offset just past them.
    Value { unit: u32, end: usize },
}

/// Reads up to four characters starting at `start` and interprets them as a
/// `\u` window. A window that runs out of interior is `Incomplete`
/// ([`LexErrorCode::Lex006`]); a full window with a non-hexadecimal
/// character is `Invalid` ([`LexErrorCode::Lex007`]).
fn hex_window(interior: &str, start: usize) -> HexWindow {
    let Some(tail) = interior.get(start..) else {
        return HexWindow::Incomplete;
    };
    let mut chars = tail.chars();
    let mut unit = 0_u32;
    let mut count = 0_usize;
    let mut bytes = 0_usize;
    let mut numeric = true;
    while count < 4 {
        let Some(c) = chars.next() else {
            return HexWindow::Incomplete;
        };
        bytes += c.len_utf8();
        match hex_value(c) {
            Some(digit) => unit = unit * 16 + digit,
            None => numeric = false,
        }
        count += 1;
    }
    let end = start + bytes;
    if numeric {
        HexWindow::Value { unit, end }
    } else {
        HexWindow::Invalid { end }
    }
}

/// If a low-surrogate `\u` escape starts exactly at `start`, returns its code
/// unit.
fn low_surrogate_escape(interior: &str, start: usize) -> Option<u32> {
    let mut chars = interior.get(start..)?.chars();
    if chars.next()? != '\\' || chars.next()? != 'u' {
        return None;
    }
    match hex_window(interior, start + 2) {
        HexWindow::Value { unit, .. } if (0xDC00..=0xDFFF).contains(&unit) => Some(unit),
        _ => None,
    }
}

/// The value of one hexadecimal digit, or `None`.
fn hex_value(c: char) -> Option<u32> {
    Some(match c {
        '0'..='9' => (c as u32) - ('0' as u32),
        'a'..='f' => (c as u32) - ('a' as u32) + 10,
        'A'..='F' => (c as u32) - ('A' as u32) + 10,
        _ => return None,
    })
}

/// Decodes one escape character to its value; `None` for an unknown escape.
#[cfg(test)]
fn known_escape(c: char) -> Option<char> {
    Some(match c {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        '\\' => '\\',
        '\'' => '\'',
        '"' => '"',
        '0' => '\0',
        _ => return None,
    })
}

fn char_at(text: &str, offset: usize) -> char {
    text[offset..]
        .chars()
        .next()
        .expect("scanner offsets stay on character boundaries")
}

#[cfg(test)]
mod tests {
    use super::{LexErrorCode, scan_escapes};

    /// D-L5 T1 — each of the eight accepted escapes decodes to the
    /// documented character.
    #[test]
    fn accepted_escapes_decode() {
        for (raw, expected) in [
            ("\\n", '\n'),
            ("\\t", '\t'),
            ("\\r", '\r'),
            ("\\\\", '\\'),
            ("\\'", '\''),
            ("\\\"", '"'),
            ("\\0", '\0'),
        ] {
            let scan = scan_escapes(raw);
            assert!(scan.issues.is_empty(), "{raw} must be accepted");
            assert_eq!(scan.decoded, expected.to_string(), "{raw}");
            assert_eq!(
                super::known_escape(raw.chars().nth(1).unwrap()),
                Some(expected)
            );
        }
        let scan = scan_escapes("\\u0041");
        assert!(scan.issues.is_empty());
        assert_eq!(scan.decoded, "A");
    }

    /// D-L9 T1 — a surrogate pair decodes to the same scalar as the direct
    /// character.
    #[test]
    fn surrogate_pairs_decode_to_one_scalar() {
        let scan = scan_escapes("\\uD83D\\uDE00");
        assert!(scan.issues.is_empty());
        assert_eq!(scan.decoded, "\u{1F600}");
        assert_eq!(scan.decoded.chars().count(), 1);
    }

    /// D-L9 T2 — a lone low surrogate is an error.
    #[test]
    fn lone_low_surrogate_reports_lex010() {
        let scan = scan_escapes("\\uDC00");
        assert_eq!(scan.issues.len(), 1);
        assert_eq!(scan.issues[0].code, LexErrorCode::Lex010);
        assert_eq!(scan.issues[0].range, 0..6);
    }

    /// D-L9 T3 — a high surrogate not followed by a low-surrogate escape is
    /// one error per lone surrogate.
    #[test]
    fn unpaired_high_surrogates_report_lex010() {
        let scan = scan_escapes("\\uD800x");
        assert_eq!(scan.issues.len(), 1);
        assert_eq!(scan.issues[0].code, LexErrorCode::Lex010);
        assert_eq!(scan.issues[0].range, 0..6);

        let scan = scan_escapes("\\uD800\\uDBFF");
        assert_eq!(scan.issues.len(), 2);
        assert!(scan.issues.iter().all(|i| i.code == LexErrorCode::Lex010));
        assert_eq!(scan.issues[0].range, 0..6);
        assert_eq!(scan.issues[1].range, 6..12);
    }

    /// A non-low-surrogate escape after a high surrogate reports the high as
    /// lone; the following escape is then scanned normally.
    #[test]
    fn high_surrogate_followed_by_plain_scalar() {
        let scan = scan_escapes("\\uD800\\u0041");
        assert_eq!(scan.issues.len(), 1);
        assert_eq!(scan.issues[0].code, LexErrorCode::Lex010);
        assert_eq!(scan.issues[0].range, 0..6);
        assert!(scan.decoded.contains('A'));
    }

    /// D-L9 T4 — an incomplete window reports `LEX006`.
    #[test]
    fn incomplete_windows_report_lex006() {
        for raw in ["\\u", "\\u1", "\\u12", "\\u123"] {
            let scan = scan_escapes(raw);
            assert_eq!(scan.issues.len(), 1, "{raw}");
            assert_eq!(scan.issues[0].code, LexErrorCode::Lex006, "{raw}");
        }
    }

    /// D-L9 T5 — a non-hexadecimal digit reports `LEX007`.
    #[test]
    fn non_hex_digit_reports_lex007() {
        let scan = scan_escapes("\\u12G4");
        assert_eq!(scan.issues.len(), 1);
        assert_eq!(scan.issues[0].code, LexErrorCode::Lex007);
        assert_eq!(scan.issues[0].range, 0..6);
    }

    /// D-L8 — unknown escapes are strict errors; each occurrence reports one.
    #[test]
    fn unknown_escapes_report_lex008() {
        let scan = scan_escapes("\\q");
        assert_eq!(scan.issues.len(), 1);
        assert_eq!(scan.issues[0].code, LexErrorCode::Lex008);
        assert_eq!(scan.issues[0].range, 0..2);

        let scan = scan_escapes("a\\q\\qb");
        assert_eq!(scan.issues.len(), 2);
        assert_eq!(scan.issues[0].range, 1..3);
        assert_eq!(scan.issues[1].range, 3..5);
    }

    #[test]
    fn decoded_values_match_direct_characters() {
        // The decoded scalar and the direct character a program could write
        // instead are the same string.
        assert_eq!(scan_escapes("\\uD83D\\uDE00").decoded, "😀");
        assert_eq!(scan_escapes("a\\nb").decoded, "a\nb");
    }
}
