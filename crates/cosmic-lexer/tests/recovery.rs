//! Recovery tests: multiple independent errors, local recovery, and the
//! one-diagnostic-per-construct discipline (G1-L D-L4/D-L5/D-L6/D-L8).

mod common;

use common::{assert_codes, lex_text, report, scan_codes, source_of, token_kinds};
use cosmic_diagnostics::Severity;
use cosmic_lexer::{ElementKind, LexErrorCode, TokenKind};

#[test]
fn one_file_with_four_independent_errors_still_lexes_its_clean_regions() {
    let text = "~ \"a\\q\" 'ab' \"\"\"x\"\"\"\nvar ok = 1\n";
    let (sources, lexed) = lex_text(text);
    assert_codes(
        &lexed,
        &[
            LexErrorCode::Lex005,
            LexErrorCode::Lex008,
            LexErrorCode::Lex004,
            LexErrorCode::Lex009,
        ],
    );
    // The clean region after the last error still lexes completely.
    let kinds = token_kinds(&lexed);
    assert_eq!(
        &kinds[kinds.len() - 4..],
        [
            TokenKind::Var,
            TokenKind::Identifier,
            TokenKind::Eq,
            TokenKind::IntLiteral,
        ]
    );
    // Every diagnostic is error severity and has a resolving primary span.
    let source = source_of(&sources, &lexed);
    for diagnostic in report(&lexed) {
        assert_eq!(diagnostic.severity(), Severity::Error);
        assert!(source.text_of(diagnostic.primary().span()).is_some());
    }
}

#[test]
fn scan_resumes_after_each_local_recovery_construct() {
    // One fixture per local recovery path; the tokens after the construct
    // are still produced in full.
    let cases: &[(&str, LexErrorCode)] = &[
        ("~ var x", LexErrorCode::Lex005),
        ("'' var x", LexErrorCode::Lex003),
        ("'ab' var x", LexErrorCode::Lex004),
        ("'a\nvar x", LexErrorCode::Lex004),
        ("\"\"\"a\"\"\" var x", LexErrorCode::Lex009),
        ("\"a\\q\" var x", LexErrorCode::Lex008),
    ];
    for (text, code) in cases {
        let (_, lexed) = lex_text(text);
        assert_codes(&lexed, &[*code]);
        let kinds = token_kinds(&lexed);
        assert_eq!(
            &kinds[kinds.len() - 2..],
            [TokenKind::Var, TokenKind::Identifier],
            "{text:?}"
        );
    }
}

#[test]
fn end_of_file_errors_keep_the_tail_as_one_region() {
    for (text, code) in [
        ("/* a", LexErrorCode::Lex001),
        ("var s = \"a", LexErrorCode::Lex002),
        ("var s = \"\"\"a", LexErrorCode::Lex009),
    ] {
        let (sources, lexed) = lex_text(text);
        assert_codes(&lexed, &[code]);
        let source = source_of(&sources, &lexed);
        let region = lexed.elements().last().expect("region");
        assert_eq!(region.kind(), ElementKind::Error, "{text:?}");
        assert_eq!(region.span().end() as usize, text.len(), "{text:?}");
        assert!(source.text_of(region.span()).is_some());
    }
}

#[test]
fn unterminated_constructs_do_not_cascade() {
    // An unterminated string contains bytes that would report escape errors
    // if scanned twice: exactly one diagnostic is emitted (LEX002), never a
    // LEX008/LEX010 cascade.
    let (_, lexed) = lex_text("var s = \"\\q\\uD800");
    assert_codes(&lexed, &[LexErrorCode::Lex002]);

    let (_, lexed) = lex_text("var c = '\\q");
    assert_codes(&lexed, &[LexErrorCode::Lex004]);
}

#[test]
fn multiple_errors_inside_one_literal_are_reported_in_scan_order() {
    let (_, lexed) = lex_text(r#"var s = "\q\u12G4x""#);
    assert_eq!(scan_codes(&lexed), ["LEX008", "LEX007"]);
    let token = lexed.tokens().last().expect("string token");
    assert_eq!(token.errors(), [LexErrorCode::Lex008, LexErrorCode::Lex007]);
}

#[test]
fn lexing_is_deterministic_for_identical_input() {
    let text = "~ \"\\q\" 'ab' \"\"\"x\"\"\" /* y";
    let (sources, first) = lex_text(text);
    let (_, second) = lex_text(text);
    assert_eq!(first, second);
    assert_eq!(
        scan_codes(&first),
        ["LEX005", "LEX008", "LEX004", "LEX009", "LEX001"]
    );
    let source = source_of(&sources, &first);
    assert!(first.is_lossless(source));
    assert_eq!(first.reprint(source).as_deref(), Some(text));
}
