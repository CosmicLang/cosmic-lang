//! Conformance tests: the testable outcomes of the G1-L decision package
//! (D-L1 … D-L10), exercised through the public API.
//!
//! Statements that live at other layers (rendering, loader, line/column
//! mapping) are tested where they belong and only referenced here.

mod common;

use common::{
    assert_codes, element_texts, lex_text, lex_to_tokens, source_of, token_kinds, token_texts,
};
use cosmic_lexer::{ElementKind, LexErrorCode, TerminatorKind, TokenKind};

// --- D-L1: BOM ---------------------------------------------------------

#[test]
fn dl1_bom_is_a_prefix_leaf_and_does_not_change_the_token_stream() {
    let (sources, without) = lex_text("var x = 1");
    let (with_sources, with) = lex_text("\u{feff}var x = 1");

    let bom = &with.elements()[0];
    assert_eq!(bom.kind(), ElementKind::Bom);
    assert_eq!((bom.span().start(), bom.span().end()), (0, 3));
    assert!(!bom.has_errors());

    // Same token kinds; every token span shifted by the three BOM bytes.
    assert_eq!(token_kinds(&with), token_kinds(&without));
    let shifted: Vec<u32> = with
        .tokens()
        .map(|element| element.span().start())
        .collect();
    let base: Vec<u32> = without
        .tokens()
        .map(|element| element.span().start() + 3)
        .collect();
    assert_eq!(shifted, base);
    let _ = (sources, with_sources);
}

#[test]
fn dl1_bom_only_file_is_one_prefix_leaf() {
    let (sources, lexed) = lex_text("\u{feff}");
    assert_eq!(lexed.elements().len(), 1);
    assert_eq!(lexed.elements()[0].kind(), ElementKind::Bom);
    assert!(lexed.diagnostics().is_empty());
    assert_eq!(lexed.tokens().count(), 0);
    let source = source_of(&sources, &lexed);
    assert_eq!(lexed.reprint(source).as_deref(), Some("\u{feff}"));
}

#[test]
fn dl1_bom_after_the_prefix_is_unexpected() {
    for text in ["\u{feff}\u{feff}var x = 1", "var \u{feff} = 1"] {
        let (_, lexed) = lex_text(text);
        assert_codes(&lexed, &[LexErrorCode::Lex005]);
    }
}

#[test]
fn dl1_empty_file_lexes_to_nothing() {
    let (sources, lexed) = lex_text("");
    assert!(lexed.elements().is_empty());
    assert!(lexed.diagnostics().is_empty());
    let source = source_of(&sources, &lexed);
    assert!(lexed.is_lossless(source));
    assert_eq!(lexed.reprint(source).as_deref(), Some(""));
}

// --- D-L2: Unicode, byte spans, line/column ----------------------------

#[test]
fn dl2_token_spans_cover_exactly_the_token_bytes() {
    let (sources, lexed) = lex_text("var π = \"α😀β\"");
    assert!(lexed.diagnostics().is_empty());
    let texts: Vec<&str> = token_texts(&sources, &lexed)
        .into_iter()
        .map(|(_, text)| text)
        .collect();
    assert_eq!(texts, ["var", "π", "=", "\"α😀β\""]);

    // No element boundary ever splits a multi-byte sequence.
    let source = source_of(&sources, &lexed);
    for element in lexed.elements() {
        assert!(
            source
                .text()
                .is_char_boundary(element.span().start() as usize)
        );
        assert!(
            source
                .text()
                .is_char_boundary(element.span().end() as usize)
        );
    }
}

#[test]
fn dl2_columns_count_scalar_values() {
    // `main() { println("😀" +) }` — the `)` after `+` is column 23 by
    // scalar values (the recorded legacy/new divergence: the legacy lexer
    // reported 24, counting UTF-16 code units).
    let (sources, lexed) = lex_text("main() { println(\"😀\" +) }");
    let source = source_of(&sources, &lexed);
    let tokens: Vec<_> = lexed.tokens().collect();
    let plus = tokens
        .iter()
        .position(|element| element.kind() == ElementKind::Token(TokenKind::Plus))
        .expect("plus token");
    let close = tokens[plus + 1];
    assert_eq!(close.kind(), ElementKind::Token(TokenKind::RParen));
    let position = source.line_col(close.span().start()).expect("inside file");
    assert_eq!((position.line, position.column), (1, 23));
}

#[test]
fn dl2_crlf_lf_and_cr_are_distinct_terminators_and_one_break_each() {
    let (sources, lexed) = lex_text("a\nb\r\nc\rd");
    let terminators: Vec<TerminatorKind> = lexed
        .elements()
        .iter()
        .filter_map(|element| match element.kind() {
            ElementKind::LineTerminator(kind) => Some(kind),
            _ => None,
        })
        .collect();
    assert_eq!(
        terminators,
        [TerminatorKind::Lf, TerminatorKind::CrLf, TerminatorKind::Cr]
    );

    // `b`, `c` and `d` open lines 2, 3 and 4 at column 1.
    let source = source_of(&sources, &lexed);
    for (name, line) in [("b", 2), ("c", 3), ("d", 4)] {
        let token = lexed
            .tokens()
            .find(|element| source.text_of(element.span()) == Some(name))
            .expect("identifier");
        let position = source.line_col(token.span().start()).expect("inside file");
        assert_eq!((position.line, position.column), (line, 1), "{name}");
    }
}

#[test]
fn dl2_terminator_bytes_are_preserved_in_their_exact_form() {
    let texts: Vec<String> = element_texts("a\r\nb\nc\rd")
        .into_iter()
        .filter(|(kind, _)| matches!(kind, ElementKind::LineTerminator(_)))
        .map(|(_, text)| text)
        .collect();
    assert_eq!(texts, ["\r\n", "\n", "\r"]);
}

#[test]
fn dl2_tab_advances_exactly_one_column() {
    let (sources, lexed) = lex_text("a\tb");
    let source = source_of(&sources, &lexed);
    let b = lexed.tokens().nth(1).expect("b");
    let position = source.line_col(b.span().start()).expect("inside file");
    assert_eq!(position.column, 3);
}

// --- D-L3: identifiers --------------------------------------------------

#[test]
fn dl3_unicode_letters_are_identifier_parts() {
    for name in ["café", "Ωmega", "𐐀"] {
        let (_, lexed) = lex_text(name);
        assert!(lexed.diagnostics().is_empty(), "{name}");
        assert_eq!(token_kinds(&lexed), [TokenKind::Identifier], "{name}");
        let tokens = lex_to_tokens(name);
        assert_eq!(tokens, [(TokenKind::Identifier, name.to_owned())], "{name}");
    }
}

#[test]
fn dl3_underscore_and_dollar_are_identifier_characters() {
    for name in ["_", "$", "_under", "$dollar", "__x", "x_1", "x$"] {
        let tokens = lex_to_tokens(name);
        assert_eq!(tokens, [(TokenKind::Identifier, name.to_owned())], "{name}");
    }
}

#[test]
fn dl3_digit_starts_a_number_not_an_identifier() {
    let tokens = lex_to_tokens("1abc");
    assert_eq!(
        tokens,
        [
            (TokenKind::IntLiteral, "1".to_owned()),
            (TokenKind::Identifier, "abc".to_owned()),
        ]
    );
}

#[test]
fn dl3_combining_mark_does_not_continue_an_identifier() {
    let (sources, lexed) = lex_text("e\u{301}");
    assert_codes(&lexed, &[LexErrorCode::Lex005]);
    assert_eq!(token_kinds(&lexed), [TokenKind::Identifier]);
    let source = source_of(&sources, &lexed);
    let region = lexed
        .elements()
        .iter()
        .find(|element| element.kind() == ElementKind::Error)
        .expect("combining mark region");
    assert_eq!(source.text_of(region.span()).unwrap(), "\u{301}");
}

#[test]
fn dl3_contextual_words_stay_identifiers() {
    let tokens = lex_to_tokens("test application infra foreign module sealed");
    assert_eq!(tokens.len(), 6);
    assert!(
        tokens
            .iter()
            .all(|(kind, _)| *kind == TokenKind::Identifier),
        "{tokens:?}"
    );
}

// --- D-L4: whitespace, comments, terminators ----------------------------

#[test]
fn dl4_whitespace_run_is_a_single_trivia_element() {
    let elements = element_texts("a \t\x0C b");
    assert_eq!(
        elements[0],
        (ElementKind::Token(TokenKind::Identifier), "a".to_owned())
    );
    assert_eq!(
        elements[1],
        (ElementKind::Whitespace, " \t\x0C ".to_owned())
    );
    assert_eq!(
        elements[2],
        (ElementKind::Token(TokenKind::Identifier), "b".to_owned())
    );
}

#[test]
fn dl4_nbsp_is_not_whitespace() {
    for space in ["\u{a0}", "\u{3000}"] {
        let text = format!("a{space}b");
        let (_, lexed) = lex_text(&text);
        assert_codes(&lexed, &[LexErrorCode::Lex005]);
    }
}

#[test]
fn dl4_block_comments_do_not_nest() {
    let tokens = lex_to_tokens("a /* x /* y */ b");
    assert_eq!(
        tokens,
        [
            (TokenKind::Identifier, "a".to_owned()),
            (TokenKind::Identifier, "b".to_owned()),
        ]
    );
    let elements = element_texts("a /* x /* y */ b");
    assert_eq!(
        elements[2],
        (ElementKind::BlockComment, "/* x /* y */".to_owned())
    );
}

#[test]
fn dl4_line_comment_excludes_every_terminator_form() {
    let elements = element_texts("// one\n// two\r\n// three\ra");
    let comments: Vec<&str> = elements
        .iter()
        .filter(|(kind, _)| *kind == ElementKind::LineComment)
        .map(|(_, text)| text.as_str())
        .collect();
    assert_eq!(comments, ["// one", "// two", "// three"]);
    let terminators: Vec<TerminatorKind> = elements
        .iter()
        .filter_map(|(kind, _)| match kind {
            ElementKind::LineTerminator(terminator) => Some(*terminator),
            _ => None,
        })
        .collect();
    assert_eq!(
        terminators,
        [TerminatorKind::Lf, TerminatorKind::CrLf, TerminatorKind::Cr]
    );
    assert_eq!(
        token_kinds(&lex_text("// one\n// two\r\n// three\ra").1),
        [TokenKind::Identifier]
    );
}

#[test]
fn dl4_comment_only_and_empty_tail_are_preserved() {
    // D-L4 T4: `//` at end of file — the comment runs to end of file with no
    // diagnostic and no trailing terminator element.
    let (sources, lexed) = lex_text("var x = 1 // tail");
    assert!(lexed.diagnostics().is_empty());
    let source = source_of(&sources, &lexed);
    let comment = lexed.elements().last().expect("comment");
    assert_eq!(comment.kind(), ElementKind::LineComment);
    assert_eq!(source.text_of(comment.span()).unwrap(), "// tail");
}

// --- D-L5: literals and escapes -----------------------------------------

#[test]
fn dl5_every_accepted_escape_lexes_clean() {
    let text = r#""\n\t\r\\\'\"\0\u0041""#;
    let (_, lexed) = lex_text(text);
    assert!(
        lexed.diagnostics().is_empty(),
        "{:?}",
        common::scan_codes(&lexed)
    );
    assert_eq!(token_kinds(&lexed), [TokenKind::StringLiteral]);
}

#[test]
fn dl5_char_and_string_share_the_escape_set() {
    let text = r#"'a' '\n' '\'' '\u0041'"#;
    let (_, lexed) = lex_text(text);
    assert!(lexed.diagnostics().is_empty());
    assert_eq!(token_kinds(&lexed), [TokenKind::CharLiteral; 4]);
}

#[test]
fn dl5_both_literal_grammars_recognise_the_same_escape_forms() {
    let string_with_all = r#""\n\t\r\\\'\"\0\u0041""#;
    let chars_with_all = r#"'\n' '\t' '\r' '\\' '\'' '\"' '\0' '\u0041'"#;
    for text in [string_with_all, chars_with_all] {
        let (_, lexed) = lex_text(text);
        assert!(lexed.diagnostics().is_empty(), "{text}");
    }
}

// --- D-L6: triple quotes -------------------------------------------------

#[test]
fn dl6_triple_quote_is_refused_as_one_error_region() {
    let (sources, lexed) = lex_text("var s = \"\"\"text\"\"\"");
    assert_codes(&lexed, &[LexErrorCode::Lex009]);
    let source = source_of(&sources, &lexed);
    let region = lexed
        .elements()
        .iter()
        .find(|element| element.kind() == ElementKind::Error)
        .expect("region");
    assert_eq!(source.text_of(region.span()).unwrap(), "\"\"\"text\"\"\"");
}

#[test]
fn dl6_empty_string_is_not_a_triple_quote() {
    let (_, lexed) = lex_text(r#"var s = """#);
    assert!(lexed.diagnostics().is_empty());
    assert_eq!(
        token_kinds(&lexed),
        [
            TokenKind::Var,
            TokenKind::Identifier,
            TokenKind::Eq,
            TokenKind::StringLiteral,
        ]
    );
}

#[test]
fn dl6_six_quotes_are_one_refused_sequence() {
    let (sources, lexed) = lex_text("var s = \"\"\"\"\"\"");
    assert_codes(&lexed, &[LexErrorCode::Lex009]);
    let source = source_of(&sources, &lexed);
    let region = lexed
        .elements()
        .iter()
        .find(|element| element.kind() == ElementKind::Error)
        .expect("region");
    assert_eq!(source.text_of(region.span()).unwrap(), "\"\"\"\"\"\"");
}

#[test]
fn dl6_unterminated_sequence_reaches_eof() {
    let (sources, lexed) = lex_text("var s = \"\"\"oops");
    assert_codes(&lexed, &[LexErrorCode::Lex009]);
    let source = source_of(&sources, &lexed);
    let region = lexed.elements().last().expect("region");
    assert_eq!(source.text_of(region.span()).unwrap(), "\"\"\"oops");
}

#[test]
fn dl6_lexing_resumes_after_the_declined_sequence() {
    let (_, lexed) = lex_text("\"\"\"x\"\"\"\nvar y = 2");
    assert_codes(&lexed, &[LexErrorCode::Lex009]);
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

// --- D-L7: raw line breaks inside strings --------------------------------

#[test]
fn dl7_raw_line_breaks_are_string_content() {
    let text = "var s = \"first\nsecond\"\nvar t = 2";
    let (sources, lexed) = lex_text(text);
    assert!(lexed.diagnostics().is_empty());
    let source = source_of(&sources, &lexed);
    let string = lexed
        .tokens()
        .find(|element| element.kind() == ElementKind::Token(TokenKind::StringLiteral))
        .expect("string");
    assert_eq!(source.text_of(string.span()).unwrap(), "\"first\nsecond\"");
    // The `t` of `var t` is on line 3, column 5 (its line starts at `var`).
    let t = lexed
        .tokens()
        .find(|element| source.text_of(element.span()) == Some("t"))
        .expect("t");
    let position = source.line_col(t.span().start()).expect("inside file");
    assert_eq!((position.line, position.column), (3, 5));
}

#[test]
fn dl7_crlf_inside_a_literal_is_preserved_byte_for_byte() {
    // D-L7 T4: the CRLF bytes are preserved exactly. Their decoded value
    // (one content character or two) is a required differential case before
    // the harness fixes it, so no value assertion is made here.
    let text = "var s = \"a\r\nb\"";
    let (sources, lexed) = lex_text(text);
    assert!(lexed.diagnostics().is_empty());
    let source = source_of(&sources, &lexed);
    assert!(lexed.is_lossless(source));
    assert_eq!(lexed.reprint(source).as_deref(), Some(text));
}

// --- D-L8: unknown escapes -----------------------------------------------

#[test]
fn dl8_literal_with_unknown_escape_is_preserved_as_an_error_marked_token() {
    let text = r#"var s = "a\qb""#;
    let (sources, lexed) = lex_text(text);
    assert_codes(&lexed, &[LexErrorCode::Lex008]);
    let source = source_of(&sources, &lexed);
    let token = lexed.tokens().last().expect("string");
    assert_eq!(token.errors(), [LexErrorCode::Lex008]);
    assert_eq!(source.text_of(token.span()).unwrap(), r#""a\qb""#);
    assert!(lexed.is_lossless(source));
}

// --- D-L9: surrogates ----------------------------------------------------

#[test]
fn dl9_surrogate_pair_is_one_clean_literal() {
    let (_, lexed) = lex_text(r#"var s = "\uD83D\uDE00""#);
    assert!(lexed.diagnostics().is_empty());
    assert_eq!(token_kinds(&lexed).last(), Some(&TokenKind::StringLiteral));
}

#[test]
fn dl9_pairing_is_greedy_and_local() {
    // First high is spurious (next escape is not low); second pair is valid:
    // exactly one diagnostic, and scanning continues inside the literal.
    let (_, lexed) = lex_text(r#"var s = "\uD800\uD83D\uDE00""#);
    assert_codes(&lexed, &[LexErrorCode::Lex010]);
}

#[test]
fn dl5_surrogate_pair_inside_a_char_literal_is_refused_as_multi_character() {
    // A1-R1-02 contract boundary: `char-literal = "'" , ( char-value ) , "'"`
    // (grammar §2.4) admits exactly one `char-value` occurrence, and D-L5
    // item 4 refuses a character literal carrying more than one character
    // value. A surrogate pair is two adjacent `escape` occurrences; D-L9
    // item 2 decodes them to one scalar in the accepted string spelling
    // (above), but a character literal carrying both is multi-character:
    // refused with exactly one LEX004, preserved byte for byte. The
    // character-literal grammar is deliberately not broadened.
    let text = r#"var c = '\uD83D\uDE00'"#;
    let (sources, lexed) = lex_text(text);
    assert_codes(&lexed, &[LexErrorCode::Lex004]);
    let source = source_of(&sources, &lexed);
    let token = lexed.tokens().last().expect("character literal token");
    assert_eq!(token.kind(), ElementKind::Token(TokenKind::CharLiteral));
    assert_eq!(token.errors(), [LexErrorCode::Lex004]);
    assert_eq!(source.text_of(token.span()).unwrap(), r#"'\uD83D\uDE00'"#);
    assert!(lexed.is_lossless(source));
    assert_eq!(lexed.reprint(source).as_deref(), Some(text));

    // The admitted single-`char-value` spelling of the same scalar is the
    // direct astral character.
    let (_, lexed) = lex_text("var c = '😀'");
    assert!(lexed.diagnostics().is_empty());
    assert_eq!(
        token_kinds(&lexed),
        [
            TokenKind::Var,
            TokenKind::Identifier,
            TokenKind::Eq,
            TokenKind::CharLiteral,
        ]
    );

    // Each half alone remains the lone-surrogate error, not a
    // multi-character refusal.
    for half in [r#"'\uD83D'"#, r#"'\uDC00'"#] {
        let (_, lexed) = lex_text(&format!("var c = {half}"));
        assert_codes(&lexed, &[LexErrorCode::Lex010]);
        assert_eq!(token_kinds(&lexed).last(), Some(&TokenKind::CharLiteral));
    }
}

// --- Vocabulary: keywords, operators, numbers ---------------------------

#[test]
fn operators_delimiters_and_separators_match_the_grammar_tables() {
    let cases: &[(&str, TokenKind)] = &[
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Star),
        ("/", TokenKind::Slash),
        ("%", TokenKind::Percent),
        ("==", TokenKind::EqEq),
        ("!=", TokenKind::BangEq),
        ("<", TokenKind::Lt),
        ("<=", TokenKind::LtEq),
        (">", TokenKind::Gt),
        (">=", TokenKind::GtEq),
        ("&&", TokenKind::AmpAmp),
        ("||", TokenKind::PipePipe),
        ("!", TokenKind::Bang),
        ("&", TokenKind::Amp),
        ("|", TokenKind::Pipe),
        ("^", TokenKind::Caret),
        ("<<", TokenKind::Shl),
        (">>", TokenKind::Shr),
        (">>>", TokenKind::Ushr),
        ("=", TokenKind::Eq),
        ("+=", TokenKind::PlusEq),
        ("-=", TokenKind::MinusEq),
        ("*=", TokenKind::StarEq),
        ("/=", TokenKind::SlashEq),
        ("%=", TokenKind::PercentEq),
        ("&=", TokenKind::AmpEq),
        ("|=", TokenKind::PipeEq),
        ("^=", TokenKind::CaretEq),
        ("<<=", TokenKind::ShlEq),
        (">>=", TokenKind::ShrEq),
        (">>>=", TokenKind::UshrEq),
        ("++", TokenKind::PlusPlus),
        ("--", TokenKind::MinusMinus),
        ("->", TokenKind::Arrow),
        ("(", TokenKind::LParen),
        (")", TokenKind::RParen),
        ("{", TokenKind::LBrace),
        ("}", TokenKind::RBrace),
        ("[", TokenKind::LBracket),
        ("]", TokenKind::RBracket),
        (";", TokenKind::Semicolon),
        (",", TokenKind::Comma),
        (".", TokenKind::Dot),
        (":", TokenKind::Colon),
        ("?", TokenKind::Question),
        ("@", TokenKind::At),
    ];
    for (text, kind) in cases {
        let tokens = lex_to_tokens(text);
        assert_eq!(tokens, [(*kind, (*text).to_owned())], "{text:?}");
    }
}

#[test]
fn keyword_tokens_and_reserved_invalid_words() {
    let tokens = lex_to_tokens("class fun fn func true false null");
    assert_eq!(
        tokens,
        [
            (TokenKind::Class, "class".to_owned()),
            (TokenKind::ReservedFun, "fun".to_owned()),
            (TokenKind::ReservedFn, "fn".to_owned()),
            (TokenKind::ReservedFunc, "func".to_owned()),
            (TokenKind::BooleanLiteral, "true".to_owned()),
            (TokenKind::BooleanLiteral, "false".to_owned()),
            (TokenKind::NullLiteral, "null".to_owned()),
        ]
    );
}

#[test]
fn numeric_literals_follow_the_grammar_suffix_rules() {
    let cases = [
        ("0", TokenKind::IntLiteral),
        ("123", TokenKind::IntLiteral),
        ("0x1F", TokenKind::IntLiteral),
        ("0Xab", TokenKind::IntLiteral),
        ("0x1f", TokenKind::IntLiteral), // `f` is a hex digit here, not a suffix
        ("1l", TokenKind::LongLiteral),
        ("2L", TokenKind::LongLiteral),
        ("0xAL", TokenKind::LongLiteral),
        ("1f", TokenKind::FloatLiteral),
        ("1F", TokenKind::FloatLiteral),
        ("1d", TokenKind::DoubleLiteral),
        ("1.5", TokenKind::DoubleLiteral),
        ("1e5", TokenKind::DoubleLiteral),
        ("1E+5", TokenKind::DoubleLiteral),
        ("1e-5", TokenKind::DoubleLiteral),
        ("1.5e2", TokenKind::DoubleLiteral),
        // `f`/`F` and `d`/`D` are valid on every number form (number-body).
        ("1.2f", TokenKind::FloatLiteral),
        ("1e3f", TokenKind::FloatLiteral),
        ("1.2e3f", TokenKind::FloatLiteral),
        ("1.2d", TokenKind::DoubleLiteral),
        ("1e3d", TokenKind::DoubleLiteral),
        ("1.2e3d", TokenKind::DoubleLiteral),
        // No value-based promotion: ranges belong to decision record S7.
        ("99999999999999999999", TokenKind::IntLiteral),
    ];
    for (text, kind) in cases {
        let tokens = lex_to_tokens(text);
        assert_eq!(tokens, [(kind, text.to_owned())], "{text}");
    }
}

#[test]
fn malformed_number_tails_lex_as_separate_valid_tokens() {
    // The grammar admits no `0x` without a hex digit and no dangling
    // exponent. The longest valid tokenisation is used; no invented
    // diagnostic, no silently accepted malformed literal.
    let cases: &[(&str, &[(TokenKind, &str)])] = &[
        (
            "0x",
            &[(TokenKind::IntLiteral, "0"), (TokenKind::Identifier, "x")],
        ),
        (
            "0xG",
            &[(TokenKind::IntLiteral, "0"), (TokenKind::Identifier, "xG")],
        ),
        (
            "0xl",
            &[(TokenKind::IntLiteral, "0"), (TokenKind::Identifier, "xl")],
        ),
        (
            "1e",
            &[(TokenKind::IntLiteral, "1"), (TokenKind::Identifier, "e")],
        ),
        (
            "1e+",
            &[
                (TokenKind::IntLiteral, "1"),
                (TokenKind::Identifier, "e"),
                (TokenKind::Plus, "+"),
            ],
        ),
        (
            "1e-",
            &[
                (TokenKind::IntLiteral, "1"),
                (TokenKind::Identifier, "e"),
                (TokenKind::Minus, "-"),
            ],
        ),
        ("1.", &[(TokenKind::IntLiteral, "1"), (TokenKind::Dot, ".")]),
        (".5", &[(TokenKind::Dot, "."), (TokenKind::IntLiteral, "5")]),
    ];
    for (text, expected) in cases {
        let (_, lexed) = lex_text(text);
        assert!(lexed.diagnostics().is_empty(), "{text:?}");
        let tokens = lex_to_tokens(text);
        let expected: Vec<(TokenKind, String)> = expected
            .iter()
            .map(|(kind, spelling)| (*kind, (*spelling).to_owned()))
            .collect();
        assert_eq!(tokens, expected, "{text:?}");
    }
}

#[test]
fn long_suffix_is_restricted_to_integer_forms() {
    // A1-R1-01: `long-literal = ( decimal-int | hex-int ) , ( "l" | "L" )`
    // (grammar §2.4) — a fraction or exponent makes the literal a double
    // form, so an `l`/`L` there is not part of the numeric token. The
    // longest valid prefix wins and the letter starts an identifier: no
    // invented diagnostic, no value-based promotion (S7) and no malformed
    // literal silently classified as `LongLiteral`.
    let cases: &[(&str, &[(TokenKind, &str)])] = &[
        (
            "1.2L",
            &[
                (TokenKind::DoubleLiteral, "1.2"),
                (TokenKind::Identifier, "L"),
            ],
        ),
        (
            "1e3L",
            &[
                (TokenKind::DoubleLiteral, "1e3"),
                (TokenKind::Identifier, "L"),
            ],
        ),
        (
            "1.2e3L",
            &[
                (TokenKind::DoubleLiteral, "1.2e3"),
                (TokenKind::Identifier, "L"),
            ],
        ),
        (
            "1.2l",
            &[
                (TokenKind::DoubleLiteral, "1.2"),
                (TokenKind::Identifier, "l"),
            ],
        ),
        // Integer forms — decimal and hex — keep the long suffix.
        ("123L", &[(TokenKind::LongLiteral, "123L")]),
        ("0x1FL", &[(TokenKind::LongLiteral, "0x1FL")]),
    ];
    for (text, expected) in cases {
        let (_, lexed) = lex_text(text);
        assert!(lexed.diagnostics().is_empty(), "{text:?}");
        let tokens = lex_to_tokens(text);
        let expected: Vec<(TokenKind, String)> = expected
            .iter()
            .map(|(kind, spelling)| (*kind, (*spelling).to_owned()))
            .collect();
        assert_eq!(tokens, expected, "{text:?}");
    }
}

#[test]
fn long_suffix_boundary_preserves_spans_and_source_bytes() {
    // Byte spans across the repaired boundary: the number owns exactly its
    // longest valid prefix and the rejected suffix byte owns the following
    // token. Source reconstruction stays byte-for-byte complete.
    let text = "var a = 1.2L\nvar b = 0x1FL\nvar c = 1e3L";
    let (sources, lexed) = lex_text(text);
    assert!(lexed.diagnostics().is_empty());
    let source = source_of(&sources, &lexed);

    let expected: &[(TokenKind, &str)] = &[
        (TokenKind::Var, "var"),
        (TokenKind::Identifier, "a"),
        (TokenKind::Eq, "="),
        (TokenKind::DoubleLiteral, "1.2"),
        (TokenKind::Identifier, "L"),
        (TokenKind::Var, "var"),
        (TokenKind::Identifier, "b"),
        (TokenKind::Eq, "="),
        (TokenKind::LongLiteral, "0x1FL"),
        (TokenKind::Var, "var"),
        (TokenKind::Identifier, "c"),
        (TokenKind::Eq, "="),
        (TokenKind::DoubleLiteral, "1e3"),
        (TokenKind::Identifier, "L"),
    ];
    let mut cursor = 0;
    for (element, &(kind, spelling)) in lexed.tokens().zip(expected) {
        let start = text[cursor..].find(spelling).expect("spelling occurs next") + cursor;
        assert_eq!(
            (element.span().start(), element.span().end()),
            (start as u32, (start + spelling.len()) as u32),
            "{spelling:?}"
        );
        assert_eq!(element.kind(), ElementKind::Token(kind), "{spelling:?}");
        assert!(!element.has_errors(), "{spelling:?}");
        assert_eq!(source.text_of(element.span()).unwrap(), spelling);
        cursor = start + spelling.len();
    }
    assert_eq!(lexed.tokens().count(), expected.len());
    assert!(lexed.is_lossless(source));
    assert_eq!(lexed.reprint(source).as_deref(), Some(text));
}

#[test]
fn negative_recognition_is_explicit() {
    // `~` and `#` are not part of the token set: unexpected characters.
    for text in ["~", "#"] {
        let (_, lexed) = lex_text(text);
        assert_codes(&lexed, &[LexErrorCode::Lex005]);
        assert_eq!(lexed.tokens().count(), 0, "{text:?}");
    }
    let (_, lexed) = lex_text("a # b");
    assert_codes(&lexed, &[LexErrorCode::Lex005]);
    assert_eq!(
        token_kinds(&lexed),
        [TokenKind::Identifier, TokenKind::Identifier]
    );
}
