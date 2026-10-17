//! Shared helpers for the `cosmic-lexer` integration tests.
//!
//! Each test file compiles as its own crate and includes this module with
//! `mod common;`. The `allow(dead_code)` covers helpers a particular test
//! crate does not use.
#![allow(dead_code)]

use cosmic_diagnostics::Diagnostic;
use cosmic_lexer::{ElementKind, LexErrorCode, LexedFile, TokenKind};
use cosmic_source::{SourceFile, SourceMap};

/// Lexes `text` and returns the map (kept alive for span resolution).
pub fn lex_text(text: &str) -> (SourceMap, LexedFile) {
    let mut sources = SourceMap::new();
    let id = sources
        .add("test.cmic", text)
        .expect("tests use small texts");
    let lexed = cosmic_lexer::lex(sources.get(id).expect("just registered"));
    (sources, lexed)
}

/// The registered file behind a lexed result.
pub fn source_of<'a>(sources: &'a SourceMap, lexed: &LexedFile) -> &'a SourceFile {
    sources
        .get(lexed.file())
        .expect("the lexed file is registered")
}

/// Diagnostic code strings in scan order.
pub fn scan_codes(lexed: &LexedFile) -> Vec<&'static str> {
    lexed
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code().as_str())
        .collect()
}

/// Diagnostics in canonical reporting order (by source position).
pub fn report(lexed: &LexedFile) -> Vec<&Diagnostic> {
    lexed.diagnostics().sorted()
}

/// Asserts that the diagnostics are exactly `expected`, in scan order.
pub fn assert_codes(lexed: &LexedFile, expected: &[LexErrorCode]) {
    let want: Vec<&str> = expected.iter().map(|code| code.as_str()).collect();
    assert_eq!(scan_codes(lexed), want, "diagnostic codes");
}

/// Token kinds in scan order.
pub fn token_kinds(lexed: &LexedFile) -> Vec<TokenKind> {
    lexed
        .tokens()
        .map(|element| match element.kind() {
            ElementKind::Token(kind) => kind,
            other => panic!("tokens() yields token elements, found {other:?}"),
        })
        .collect()
}

/// `(kind, lexeme)` pairs for every token, in scan order.
pub fn token_texts<'a>(sources: &'a SourceMap, lexed: &LexedFile) -> Vec<(TokenKind, &'a str)> {
    let source = source_of(sources, lexed);
    lexed
        .tokens()
        .map(|element| {
            let kind = match element.kind() {
                ElementKind::Token(kind) => kind,
                other => panic!("tokens() yields token elements, found {other:?}"),
            };
            (
                kind,
                source
                    .text_of(element.span())
                    .expect("element spans resolve"),
            )
        })
        .collect()
}

/// Lexes `text` and returns `(kind, lexeme)` pairs for every token.
pub fn lex_to_tokens(text: &str) -> Vec<(TokenKind, String)> {
    let (sources, lexed) = lex_text(text);
    token_texts(&sources, &lexed)
        .into_iter()
        .map(|(kind, text)| (kind, text.to_owned()))
        .collect()
}

/// `(kind, lexeme)` pairs for every element, in scan order.
pub fn element_texts(text: &str) -> Vec<(ElementKind, String)> {
    let (sources, lexed) = lex_text(text);
    let source = source_of(&sources, &lexed);
    lexed
        .elements()
        .iter()
        .map(|element| {
            (
                element.kind(),
                source
                    .text_of(element.span())
                    .expect("element spans resolve")
                    .to_owned(),
            )
        })
        .collect()
}
