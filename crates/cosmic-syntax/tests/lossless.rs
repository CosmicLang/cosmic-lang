//! Losslessness: every `cosmic-lexer` element — token, trivia, BOM or lexical
//! error region — becomes exactly one CST leaf, in order, and the reprint is
//! byte-for-byte, for recognised and malformed input alike. No byte is
//! fabricated and no byte is dropped.

mod common;

use common::{all_codes, parse_codes, parse_text, source_of};
use cosmic_lexer::TokenKind;
use cosmic_syntax::NodeKind;

/// Representative inputs: empty and trivial files, BOM handling, Unicode,
/// trivia, and every malformed shape the parser must survive.
const CASES: &[&str] = &[
    "",
    "\n",
    "\r\n",
    "\t \t",
    "\u{feff}",
    "\u{feff}\u{feff}",
    "\u{feff}main() {}\n",
    "main()  {}  // trailing comment\n",
    "main() {\n\tval x = 1 // 日本\n}\n",
    "main() { val café = 1 }\n",
    "main() { \"héllo ☕\" }\n",
    "main() { \"🚀\" }\n",
    "main() { val x = \"a\r\nb\" }\n",
    "main() {\r\n  call()\r\n}\r\n",
    "main() { /* block\n comment */ }\n",
    "main() {",
    "main() { val x = ?\n",
    "@Bad\nmain() {}\n",
    "main() { val xs = {1, 2} }\n",
    "fun main() {}\n",
    "main() { break outer }\n",
    "main() { val x = a >> b >>> c }\n",
    "main() { val f = (x) -> x }\n",
    "main() { switch (x) { case",
    "main() { val x = 1.2L }\n",
    "main() { ' }\n",
    "main() { \"unterminated\n",
    "/* unterminated\n",
    "main() { \0 }\n",
];

#[test]
fn byte_for_byte_round_trip_over_representative_inputs() {
    for text in CASES {
        let (sources, parsed) = parse_text(text);
        let source = source_of(&sources, &parsed);
        assert!(parsed.is_lossless(source), "not lossless: {text:?}");
        assert_eq!(
            parsed.reprint(source).as_deref(),
            Some(*text),
            "reprint mismatch: {text:?}"
        );
    }
}

#[test]
fn leaf_spans_tile_the_whole_file_in_order() {
    for text in CASES {
        let (sources, parsed) = parse_text(text);
        let source = source_of(&sources, &parsed);
        let mut cursor = 0u32;
        for leaf in parsed.leaves() {
            assert_eq!(leaf.span().start(), cursor, "gap before a leaf: {text:?}");
            assert!(
                leaf.span().end() > leaf.span().start(),
                "zero-width leaf: {text:?}"
            );
            cursor = leaf.span().end();
        }
        assert_eq!(
            cursor,
            source.len(),
            "tiling does not reach the end: {text:?}"
        );
    }
}

#[test]
fn trivia_and_bom_leaves_are_preserved_in_order() {
    let text = "\u{feff}main()  {}  // tail\n/* block */\n";
    let (sources, parsed) = parse_text(text);
    let source = source_of(&sources, &parsed);
    let leaves: Vec<_> = parsed.leaves().collect();

    let bom = leaves.first().expect("a first leaf");
    assert_eq!(bom.kind(), NodeKind::Bom);
    assert_eq!(bom.span().start(), 0);
    assert_eq!(bom.span().end(), 3);
    assert!(bom.token_kind().is_none());

    let texts: Vec<&str> = leaves
        .iter()
        .map(|leaf| leaf.text(source).expect("same file"))
        .collect();
    assert!(texts.contains(&"  "));
    assert!(texts.contains(&"// tail"));
    assert!(texts.contains(&"/* block */"));
    assert!(texts.iter().any(|text| text.contains('\n')));

    // Tokens exclude trivia; the leaf stream includes everything.
    assert!(parsed.tokens().count() < leaves.len());
    assert!(parsed.is_lossless(source));
}

#[test]
fn lexer_error_regions_are_leaves_and_lex_diagnostics_survive() {
    let text = "main() { \"unterminated\n";
    let (sources, parsed) = parse_text(text);
    let source = source_of(&sources, &parsed);
    assert!(parsed.is_lossless(source));

    let lex_error_leaves = parsed
        .leaves()
        .filter(|leaf| leaf.kind() == NodeKind::LexError)
        .count();
    assert_eq!(lex_error_leaves, 1);

    let codes = all_codes(&parsed);
    assert!(codes.contains(&"LEX002"), "{codes:?}");
    // The parser adds its own diagnostic for the unclosed block; it never
    // rewrites the lexer's.
    assert_eq!(parse_codes(&parsed), vec!["PARSE003"]);
}

#[test]
fn split_shift_leaves_share_one_lexer_element() {
    let (sources, parsed) = parse_text("main() { val x = a >> b }\n");
    let source = source_of(&sources, &parsed);
    let gt_leaves: Vec<_> = parsed
        .leaves()
        .filter(|leaf| leaf.kind() == NodeKind::Token(TokenKind::Gt))
        .collect();
    assert_eq!(gt_leaves.len(), 2);
    assert_eq!(gt_leaves[0].span().end(), gt_leaves[1].span().start());

    let first = gt_leaves[0].element().expect("leaf mirrors an element");
    let second = gt_leaves[1].element().expect("leaf mirrors an element");
    assert!(std::ptr::eq(first, second));
    assert_eq!(first.span().start(), gt_leaves[0].span().start());
    assert_eq!(first.span().end(), gt_leaves[1].span().end());
    assert!(parsed.is_lossless(source));
}

#[test]
fn bom_only_file_is_valid_and_preserved() {
    let (sources, parsed) = parse_text("\u{feff}");
    assert!(parsed.diagnostics().is_empty());
    let source = source_of(&sources, &parsed);
    assert!(parsed.is_lossless(source));
    assert_eq!(parsed.reprint(source).as_deref(), Some("\u{feff}"));
    assert_eq!(parsed.leaves().count(), 1);
}

#[test]
fn zero_width_structural_nodes_contribute_no_text() {
    // A missing expression leaves an empty structural node behind; it must
    // carry an empty span and no text, and must not disturb the tiling.
    let text = "main() { val x = }\n";
    let (sources, parsed) = parse_text(text);
    let source = source_of(&sources, &parsed);
    for node in parsed.nodes() {
        if node.is_zero_width() {
            assert_eq!(node.span().start(), node.span().end());
            assert_eq!(node.text(source), Some(""));
        }
    }
    assert!(parsed.is_lossless(source));
}
