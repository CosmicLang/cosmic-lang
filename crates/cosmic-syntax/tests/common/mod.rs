//! Shared helpers for the `cosmic-syntax` integration tests.
//!
//! Each test file compiles as its own crate and includes this module with
//! `mod common;`. The `allow(dead_code)` covers helpers a particular test
//! crate does not use.
#![allow(dead_code)]

use cosmic_lexer::TokenKind;
use cosmic_source::{SourceFile, SourceMap};
use cosmic_syntax::{Node, NodeKind, ParsedFile};

/// Parses `text` and returns the map (kept alive for span resolution).
pub fn parse_text(text: &str) -> (SourceMap, ParsedFile) {
    let mut sources = SourceMap::new();
    let id = sources
        .add("test.csmx", text)
        .expect("tests use small texts");
    let parsed = cosmic_syntax::parse(sources.get(id).expect("just registered"));
    (sources, parsed)
}

/// The registered file behind a parsed result.
pub fn source_of<'a>(sources: &'a SourceMap, parsed: &ParsedFile) -> &'a SourceFile {
    sources
        .get(parsed.file())
        .expect("the parsed file is registered")
}

/// Parses `text` and asserts full recognition: no `LEX`/`PARSE` errors, a
/// lossless tree (leaves mirror the lexer elements exactly) and a
/// byte-for-byte reprint.
pub fn recognised(text: &str) -> (SourceMap, ParsedFile) {
    let (sources, parsed) = parse_text(text);
    let source = source_of(&sources, &parsed);
    assert!(
        !parsed.has_errors(),
        "expected a fully recognised parse of {text:?}, got {:?}",
        describe(&parsed)
    );
    assert!(parsed.is_lossless(source), "not lossless: {text:?}");
    assert_eq!(parsed.reprint(source).as_deref(), Some(text), "reprint");
    (sources, parsed)
}

/// `PARSE` code strings in discovery order.
pub fn parse_codes(parsed: &ParsedFile) -> Vec<&'static str> {
    parsed
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code().as_str())
        .collect()
}

/// All diagnostic code strings (`LEX` then `PARSE`) in canonical order.
pub fn all_codes(parsed: &ParsedFile) -> Vec<&'static str> {
    parsed
        .all_diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code().as_str())
        .collect()
}

/// `CODE: message` for every diagnostic, for assertion messages.
pub fn describe(parsed: &ParsedFile) -> Vec<String> {
    parsed
        .all_diagnostics()
        .iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.code().as_str(), diagnostic.message()))
        .collect()
}

/// Every node in pre-order.
pub fn nodes(parsed: &ParsedFile) -> Vec<Node<'_>> {
    parsed.nodes().collect()
}

/// Every node of one kind, in pre-order.
pub fn nodes_of_kind(parsed: &ParsedFile, kind: NodeKind) -> Vec<Node<'_>> {
    parsed.nodes().filter(|node| node.kind() == kind).collect()
}

/// The first node of one kind, in pre-order.
pub fn first_of_kind(parsed: &ParsedFile, kind: NodeKind) -> Option<Node<'_>> {
    parsed.nodes().find(|node| node.kind() == kind)
}

/// How many nodes of one kind exist.
pub fn count_of_kind(parsed: &ParsedFile, kind: NodeKind) -> usize {
    parsed.nodes().filter(|node| node.kind() == kind).count()
}

/// `(kind, lexeme)` pairs for every token leaf, in source order.
pub fn token_texts<'a>(
    sources: &'a SourceMap,
    parsed: &'a ParsedFile,
) -> Vec<(TokenKind, &'a str)> {
    let source = source_of(sources, parsed);
    parsed
        .tokens()
        .map(|node| {
            (
                node.token_kind().expect("tokens() yields token leaves"),
                node.text(source).expect("leaf spans resolve"),
            )
        })
        .collect()
}

/// Token kinds in source order.
pub fn token_kinds(parsed: &ParsedFile) -> Vec<TokenKind> {
    parsed
        .tokens()
        .map(|node| node.token_kind().expect("tokens() yields token leaves"))
        .collect()
}

/// The source text of a node.
pub fn text_of<'a>(node: Node<'_>, source: &'a SourceFile) -> &'a str {
    node.text(source).expect("node spans the parsed file")
}
