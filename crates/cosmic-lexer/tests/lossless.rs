//! Losslessness tests (G1-L D-L10): partitioning, reprint identity,
//! byte preservation over the fixture corpus and over truncations.

mod common;

use common::{element_texts, lex_text, source_of};
use cosmic_lexer::ElementKind;
use cosmic_source::{LoadError, SourceMap};

/// Independently authored fixture corpus: clean files, malformed files and
/// every recovery construct of the lexer.
const CORPUS: &[&str] = &[
    "",
    "\u{feff}",
    "var x = 1\n",
    "\u{feff}var x = 1\r\nvar y = 2\rvar z = 3\n",
    "// comment only",
    "/* still open",
    "\"unterminated",
    "'a' 'ab' '' '",
    "\"\"\"raw\"\"\"",
    "a ~ b \u{a0} c",
    "var s = \"line\nbreak\"",
    "var s = \"\\uD800\"",
    "var s = \"\\q\\u12G4\"",
    "0x 1e 1e+ 1. .5 0x1F 1l 1f 1d",
    "π = \"😀\" // café\r\n",
    "var a = 0xAL;\n",
    "/* x /* y */ z */",
    "\u{feff}\u{feff}",
];

#[test]
fn reprint_identity_over_the_fixture_corpus() {
    for text in CORPUS {
        let (sources, lexed) = lex_text(text);
        let source = source_of(&sources, &lexed);
        assert!(lexed.is_lossless(source), "{text:?}");
        assert_eq!(lexed.reprint(source).as_deref(), Some(*text), "{text:?}");
    }
}

#[test]
fn elements_partition_the_file_without_gaps_or_overlaps() {
    for text in CORPUS {
        let (_, lexed) = lex_text(text);
        let mut expected = 0_u32;
        for element in lexed.elements() {
            assert_eq!(element.span().start(), expected, "{text:?}");
            assert!(element.span().end() >= element.span().start());
            expected = element.span().end();
        }
        assert_eq!(expected as usize, text.len(), "{text:?}");
    }
}

#[test]
fn trivia_is_preserved_exactly_as_elements() {
    let elements = element_texts(" \t// note\r\n/* block */\n");
    let kinds: Vec<_> = elements
        .iter()
        .map(|(kind, text)| (*kind, text.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            (ElementKind::Whitespace, " \t"),
            (ElementKind::LineComment, "// note"),
            (
                ElementKind::LineTerminator(cosmic_lexer::TerminatorKind::CrLf),
                "\r\n"
            ),
            (ElementKind::BlockComment, "/* block */"),
            (
                ElementKind::LineTerminator(cosmic_lexer::TerminatorKind::Lf),
                "\n"
            ),
        ]
    );
}

#[test]
fn every_prefix_of_a_malformed_file_is_lossless() {
    let text = "var s = \"a\\u12\" /* x /* y */ 'ab' \"\"\"q\"\"\" ~ 0x1e";
    let mut sources = SourceMap::new();
    for (offset, _) in text.char_indices() {
        let prefix = &text[..offset];
        let id = sources
            .add(format!("prefix-{offset}.cmic"), prefix)
            .expect("small text");
        let source = sources.get(id).expect("just registered");
        let lexed = cosmic_lexer::lex(source);
        assert!(lexed.is_lossless(source), "prefix at {offset}");
        assert_eq!(lexed.reprint(source).as_deref(), Some(prefix));
    }
}

#[test]
fn every_byte_reaches_exactly_one_element_category() {
    // Tokens + trivia + error regions + at most one BOM element cover the
    // file; the partition check above proves contiguity, this proves no
    // element is silently skipped by kinds alone.
    let text = "\u{feff}/* a */ var s = \"\\q\" 'x' ~\n";
    let (sources, lexed) = lex_text(text);
    let source = source_of(&sources, &lexed);
    let mut covered = 0_usize;
    for element in lexed.elements() {
        let part = source
            .text_of(element.span())
            .expect("element spans resolve");
        covered += part.len();
    }
    assert_eq!(covered, text.len());
}

#[test]
fn load_registers_and_lexes_a_utf8_file_from_disk() {
    let path = std::env::temp_dir().join(format!(
        "cosmic-lexer-roundtrip-{}.cmic",
        std::process::id()
    ));
    let text = "\u{feff}var s = \"café\"\r\n// note\n";
    std::fs::write(&path, text).expect("temp file");
    let mut sources = SourceMap::new();
    let id = sources.load(&path).expect("valid UTF-8");
    let source = sources.get(id).expect("just loaded");
    let lexed = cosmic_lexer::lex(source);
    assert!(lexed.diagnostics().is_empty());
    assert!(lexed.is_lossless(source));
    assert_eq!(lexed.reprint(source).as_deref(), Some(text));
    std::fs::remove_file(&path).ok();
}

#[test]
fn invalid_utf8_is_rejected_by_the_loader_before_lexing() {
    let path =
        std::env::temp_dir().join(format!("cosmic-lexer-invalid-{}.cmic", std::process::id()));
    std::fs::write(&path, [0x76, 0xFF, 0x76]).expect("temp file");
    let mut sources = SourceMap::new();
    match sources.load(&path) {
        Err(LoadError::NotUtf8 { valid_up_to, .. }) => assert_eq!(valid_up_to, 1),
        other => panic!("expected NotUtf8, got {other:?}"),
    }
    assert_eq!(sources.len(), 0, "a rejected load registers nothing");
    std::fs::remove_file(&path).ok();
}
