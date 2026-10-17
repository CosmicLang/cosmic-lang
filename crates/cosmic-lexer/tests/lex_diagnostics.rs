//! One positive test per `LEX` code, plus the code-stability check.
//!
//! The evidence column of `docs/spec/diagnostic-codes.md` names these tests,
//! so their names are part of the registry: rename only together with the
//! registry entry.

mod common;

use common::{assert_codes, lex_text, report, scan_codes, source_of, token_kinds};
use cosmic_diagnostics::Severity;
use cosmic_lexer::{ElementKind, LexErrorCode, LexedFile, TokenKind};
use cosmic_source::SourceMap;

/// Lexes and asserts exactly one error-severity diagnostic of `code`.
fn one_diagnostic(text: &str, code: LexErrorCode) -> (SourceMap, LexedFile) {
    let (sources, lexed) = lex_text(text);
    assert_codes(&lexed, &[code]);
    let diagnostics = report(&lexed);
    assert_eq!(diagnostics.len(), 1, "{text:?}");
    assert_eq!(diagnostics[0].severity(), Severity::Error, "{text:?}");
    (sources, lexed)
}

#[test]
fn lex001_unterminated_block_comment_is_reported_once() {
    let text = "var a = 1\n/* never closed\nvar b = 2\n";
    let (sources, lexed) = one_diagnostic(text, LexErrorCode::Lex001);
    // One error region from the `/*` to end of file, no cascade.
    let region = lexed.elements().last().expect("non-empty");
    assert_eq!(region.kind(), ElementKind::Error);
    assert_eq!(region.errors(), [LexErrorCode::Lex001]);
    assert_eq!(region.span().start() as usize, text.find("/*").unwrap());
    assert_eq!(region.span().end() as usize, text.len());
    let source = source_of(&sources, &lexed);
    assert!(source.text_of(region.span()).is_some());
    // Tokens before the comment are unaffected.
    assert_eq!(
        token_kinds(&lexed),
        [
            TokenKind::Var,
            TokenKind::Identifier,
            TokenKind::Eq,
            TokenKind::IntLiteral,
        ]
    );
}

#[test]
fn lex002_unterminated_string_is_reported_once() {
    // Raw line breaks are legal inside strings (D-L7), so the literal only
    // ends at the closing quote or end of file.
    let text = "var s = \"abc\ndef";
    let (sources, lexed) = one_diagnostic(text, LexErrorCode::Lex002);
    let region = lexed.elements().last().expect("non-empty");
    assert_eq!(region.kind(), ElementKind::Error);
    assert_eq!(region.span().end() as usize, text.len());
    let source = source_of(&sources, &lexed);
    assert_eq!(source.text_of(region.span()).unwrap(), "\"abc\ndef");
}

#[test]
fn lex003_empty_character_literal() {
    let (sources, lexed) = one_diagnostic("var c = ''", LexErrorCode::Lex003);
    let region = lexed
        .elements()
        .iter()
        .find(|element| element.kind() == ElementKind::Error)
        .expect("error region");
    let source = source_of(&sources, &lexed);
    assert_eq!(source.text_of(region.span()).unwrap(), "''");
}

#[test]
fn lex004_unterminated_and_multi_character_literals() {
    // `'ab'` closes but carries two character values: one error-marked token.
    let (sources, lexed) = lex_text("var c = 'ab'");
    assert_codes(&lexed, &[LexErrorCode::Lex004]);
    let source = source_of(&sources, &lexed);
    let marked: Vec<_> = lexed
        .elements()
        .iter()
        .filter(|element| element.errors() == [LexErrorCode::Lex004])
        .collect();
    assert_eq!(marked.len(), 1);
    assert_eq!(marked[0].kind(), ElementKind::Token(TokenKind::CharLiteral));
    assert_eq!(source.text_of(marked[0].span()).unwrap(), "'ab'");

    // `'a` + line break: an unterminated region reaching the line break;
    // scanning resumes and the next line still lexes.
    let (sources, lexed) = lex_text("var c = 'a\nvar d = 2");
    assert_codes(&lexed, &[LexErrorCode::Lex004]);
    let source = source_of(&sources, &lexed);
    let region = lexed
        .elements()
        .iter()
        .find(|element| element.kind() == ElementKind::Error)
        .expect("error region");
    assert_eq!(source.text_of(region.span()).unwrap(), "'a");
    assert_eq!(
        token_kinds(&lexed),
        [
            TokenKind::Var,
            TokenKind::Identifier,
            TokenKind::Eq,
            TokenKind::Var,
            TokenKind::Identifier,
            TokenKind::Eq,
            TokenKind::IntLiteral,
        ]
    );

    // A lone `'` at end of file has no closing quote: unterminated.
    one_diagnostic("var c = '", LexErrorCode::Lex004);
}

#[test]
fn lex005_unexpected_characters_are_reported() {
    for (text, expected) in [
        ("var x = ~1", "~"),
        ("var x = a\u{a0}b", "\u{a0}"),
        ("var x = 'a' #", "#"),
    ] {
        let (sources, lexed) = one_diagnostic(text, LexErrorCode::Lex005);
        let source = source_of(&sources, &lexed);
        let region = lexed
            .elements()
            .iter()
            .find(|element| element.kind() == ElementKind::Error)
            .expect("error region");
        assert_eq!(source.text_of(region.span()).unwrap(), expected, "{text:?}");
    }

    // A byte-order mark after the prefix gets the documented help note.
    let (_, lexed) = lex_text("\u{feff}\u{feff}var x = 1");
    assert_codes(&lexed, &[LexErrorCode::Lex005]);
    let diagnostics = report(&lexed);
    assert_eq!(diagnostics[0].severity(), Severity::Error);
    assert!(
        diagnostics[0]
            .notes()
            .iter()
            .any(|note| note.contains("byte-order mark")),
        "the misplaced-BOM diagnostic carries the D-L1 help note"
    );
}

#[test]
fn lex006_incomplete_unicode_escape() {
    let text = "var s = \"\\u12\"";
    let (sources, lexed) = one_diagnostic(text, LexErrorCode::Lex006);
    let source = source_of(&sources, &lexed);
    let diagnostic = report(&lexed)[0];
    assert_eq!(
        source.text_of(diagnostic.primary().span()).unwrap(),
        "\\u12"
    );
}

#[test]
fn lex007_invalid_unicode_escape() {
    let text = "var s = \"\\u12G4\"";
    let (sources, lexed) = one_diagnostic(text, LexErrorCode::Lex007);
    let source = source_of(&sources, &lexed);
    let diagnostic = report(&lexed)[0];
    assert_eq!(
        source.text_of(diagnostic.primary().span()).unwrap(),
        "\\u12G4"
    );
}

#[test]
fn lex008_unknown_escape_is_an_error() {
    // One unknown escape: one diagnostic, the literal preserved as an
    // error-marked token (D-L8).
    let text = r#"var s = "a\qb""#;
    let (sources, lexed) = one_diagnostic(text, LexErrorCode::Lex008);
    let source = source_of(&sources, &lexed);
    let token = lexed.tokens().last().expect("string token");
    assert_eq!(token.errors(), [LexErrorCode::Lex008]);
    assert_eq!(source.text_of(token.span()).unwrap(), r#""a\qb""#);
    assert!(
        report(&lexed)[0]
            .message()
            .contains("unknown escape sequence"),
        "message carries the unknown escape"
    );

    // Two unknown escapes in one literal: two diagnostics, scan order.
    let (_, lexed) = lex_text(r#"var s = "a\q\qb""#);
    assert_codes(&lexed, &[LexErrorCode::Lex008, LexErrorCode::Lex008]);
    assert_eq!(
        report(&lexed)
            .iter()
            .map(|diagnostic| diagnostic.severity())
            .collect::<Vec<_>>(),
        [Severity::Error, Severity::Error]
    );
}

#[test]
fn lex009_triple_quoted_strings_are_refused() {
    let text = "var s = \"\"\"text\"\"\"\nvar y = 2";
    let (sources, lexed) = one_diagnostic(text, LexErrorCode::Lex009);
    let source = source_of(&sources, &lexed);
    let region = lexed
        .elements()
        .iter()
        .find(|element| element.kind() == ElementKind::Error)
        .expect("error region");
    assert_eq!(source.text_of(region.span()).unwrap(), "\"\"\"text\"\"\"");
    // Lexing resumes after the refused sequence.
    assert_eq!(
        token_kinds(&lexed),
        [
            TokenKind::Var,
            TokenKind::Identifier,
            TokenKind::Eq,
            TokenKind::Var,
            TokenKind::Identifier,
            TokenKind::Eq,
            TokenKind::IntLiteral,
        ]
    );
}

#[test]
fn lex010_lone_surrogate_escapes_are_errors() {
    // A lone high surrogate followed by an ordinary character.
    let (sources, lexed) = one_diagnostic("var s = \"\\uD800x\"", LexErrorCode::Lex010);
    let source = source_of(&sources, &lexed);
    let diagnostic = report(&lexed)[0];
    assert_eq!(
        source.text_of(diagnostic.primary().span()).unwrap(),
        "\\uD800"
    );

    // A lone low surrogate.
    one_diagnostic("var s = \"\\uDC00\"", LexErrorCode::Lex010);

    // A properly paired escape is clean.
    let (_, lexed) = lex_text("var s = \"\\uD83D\\uDE00\"");
    assert!(lexed.diagnostics().is_empty());
}

#[test]
fn lex_codes_are_stable() {
    // The ten allocations are frozen: code strings and the namespace format.
    assert_eq!(LexErrorCode::ALL.len(), 10);
    for (index, code) in LexErrorCode::ALL.into_iter().enumerate() {
        let expected = format!("LEX{:03}", index + 1);
        assert_eq!(code.as_str(), expected);
        assert_eq!(code.code().as_str(), expected);
        assert!(!code.condition().is_empty(), "{expected}");
    }
    // No fixture above may ever produce an unknown code.
    let (_, lexed) = lex_text("~ \"\\q\" 'ab' '' 'x\n/* \"\"\" ");
    let known: Vec<&str> = LexErrorCode::ALL.iter().map(|code| code.as_str()).collect();
    assert!(
        scan_codes(&lexed).iter().all(|code| known.contains(code)),
        "{:?}",
        scan_codes(&lexed)
    );
}
