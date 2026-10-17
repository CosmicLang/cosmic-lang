//! Unicode behaviour tests: encoding, classification and position edges
//! (G1-L D-L1/D-L2/D-L3, with the S13 measurement boundaries made explicit).

mod common;

use common::{assert_codes, lex_text, source_of, token_kinds};
use cosmic_lexer::{ElementKind, LexErrorCode, TokenKind};

#[test]
fn multibyte_characters_in_identifiers_and_strings_are_single_tokens() {
    let (sources, lexed) = lex_text("var π = \"α😀β\"");
    assert!(lexed.diagnostics().is_empty());
    let texts: Vec<&str> = lexed
        .tokens()
        .map(|element| source_of(&sources, &lexed).text_of(element.span()).unwrap())
        .collect();
    assert_eq!(texts, ["var", "π", "=", "\"α😀β\""]);
}

#[test]
fn astral_letter_identifiers_are_single_tokens() {
    // U+10400 and U+10428: DESERET capital/small letters outside the BMP.
    let name = "𐐀𐐨";
    let (_, lexed) = lex_text(name);
    assert!(lexed.diagnostics().is_empty());
    assert_eq!(token_kinds(&lexed), [TokenKind::Identifier]);
}

#[test]
fn keyword_like_prefixes_continue_as_identifiers() {
    // The keyword table matches whole lexemes only.
    let (_, lexed) = lex_text("classé class_ class$ class1");
    assert!(lexed.diagnostics().is_empty());
    assert_eq!(
        token_kinds(&lexed),
        [
            TokenKind::Identifier,
            TokenKind::Identifier,
            TokenKind::Identifier,
            TokenKind::Identifier,
        ]
    );
}

#[test]
fn combining_marks_after_identifiers_are_unexpected_characters() {
    let (sources, lexed) = lex_text("e\u{301}x");
    assert_codes(&lexed, &[LexErrorCode::Lex005]);
    let source = source_of(&sources, &lexed);
    let region = lexed
        .elements()
        .iter()
        .find(|element| element.kind() == ElementKind::Error)
        .expect("combining mark region");
    assert_eq!(source.text_of(region.span()).unwrap(), "\u{301}");
    // `x` still lexes as its own identifier.
    let identifiers: Vec<&str> = lexed
        .tokens()
        .map(|element| source.text_of(element.span()).unwrap())
        .collect();
    assert_eq!(identifiers, ["e", "x"]);
}

#[test]
fn unicode_line_separators_are_not_line_terminators() {
    for separator in ['\u{2028}', '\u{2029}'] {
        let text = format!("a{separator}b");
        let (sources, lexed) = lex_text(&text);
        assert_codes(&lexed, &[LexErrorCode::Lex005]);
        // The separator does not open a new line: `b` is still on line 1.
        let source = source_of(&sources, &lexed);
        let b = lexed
            .tokens()
            .find(|element| source.text_of(element.span()) == Some("b"))
            .expect("b");
        assert_eq!(source.line_col(b.span().start()).unwrap().line, 1);
    }
}

#[test]
fn spaced_separators_are_not_whitespace() {
    for space in ['\u{a0}', '\u{3000}', '\u{2028}'] {
        let text = format!("a{space}b");
        let (_, lexed) = lex_text(&text);
        assert_codes(&lexed, &[LexErrorCode::Lex005]);
    }
}

#[test]
fn form_feed_is_whitespace() {
    let (_, lexed) = lex_text("a\u{c}b");
    assert!(lexed.diagnostics().is_empty());
    assert_eq!(
        token_kinds(&lexed),
        [TokenKind::Identifier, TokenKind::Identifier]
    );
}

#[test]
fn multi_byte_error_regions_cover_whole_scalar_values() {
    // An emoji outside any literal is one error region of four bytes.
    let (sources, lexed) = lex_text("a😀b");
    assert_codes(&lexed, &[LexErrorCode::Lex005]);
    let source = source_of(&sources, &lexed);
    let region = lexed
        .elements()
        .iter()
        .find(|element| element.kind() == ElementKind::Error)
        .expect("region");
    assert_eq!(source.text_of(region.span()).unwrap(), "😀");
    assert_eq!(region.span().end() - region.span().start(), 4);
}
