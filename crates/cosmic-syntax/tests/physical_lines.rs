//! Physical-line regressions (A2-R1-01).
//!
//! The accepted line rules — statement separation, expression
//! continuations, the trailing-lambda same-line rule (grammar §6.1 note,
//! accepted by 003-G1 §4) and the same-line nested-function body — are
//! judged by the **physical** line breaks of the source. A `LF`, `CR` or
//! `CRLF` separating two significant tokens counts wherever it occurs
//! between them, including inside a block comment or a recoverable
//! lexical-error region. A line break inside a string-literal *token* is
//! part of that token, not something between two tokens.
//!
//! Every case pins concrete node kinds, token spans, diagnostic codes and
//! byte-for-byte reconstruction — never only "the parse succeeded".

mod common;

use common::{
    all_codes, count_of_kind, describe, first_of_kind, nodes_of_kind, parse_codes, parse_text,
    recognised, source_of, text_of,
};
use cosmic_syntax::{NodeKind, ParsedFile};

/// The three accepted physical line terminators (grammar §1.2).
const BREAKS: [&str; 3] = ["\n", "\r", "\r\n"];

/// `(kind, span)` for every node, in pre-order: the structural signature.
fn signature(parsed: &ParsedFile) -> Vec<(NodeKind, u32, u32)> {
    parsed
        .nodes()
        .map(|node| (node.kind(), node.span().start(), node.span().end()))
        .collect()
}

#[test]
fn a_same_line_continuation_is_kept_across_an_inline_block_comment() {
    let (sources, parsed) = recognised("main() {\n    val x = a /* inline */ + b\n}\n");
    let source = source_of(&sources, &parsed);
    assert_eq!(count_of_kind(&parsed, NodeKind::BinaryExpr), 1);
    let val = first_of_kind(&parsed, NodeKind::VarDeclStmt).expect("val statement");
    // Node spans include the preceding trivia leaves (lossless forward
    // attachment); the content after that is exact.
    assert_eq!(
        text_of(val, source).trim_start(),
        "val x = a /* inline */ + b"
    );
}

#[test]
fn a_physical_break_inside_a_block_comment_ends_the_continuation() {
    for break_ in BREAKS {
        let text = format!("main() {{\n    val x = a /*{break_}*/ + b\n}}\n");
        let context = format!("break {break_:?}");
        let (sources, parsed) = parse_text(&text);
        let source = source_of(&sources, &parsed);
        assert_eq!(
            parse_codes(&parsed),
            vec!["PARSE002"],
            "{context}: {:?}",
            describe(&parsed)
        );
        assert_eq!(count_of_kind(&parsed, NodeKind::BinaryExpr), 0, "{context}");
        // The `val` statement ends at the comment; the abandoned `+ b`
        // continuation is preserved as one error region (which absorbs the
        // trivia before it, including the comment).
        let val = first_of_kind(&parsed, NodeKind::VarDeclStmt).expect("val statement");
        assert_eq!(text_of(val, source).trim_start(), "val x = a", "{context}");
        let error = first_of_kind(&parsed, NodeKind::Error).expect("error region");
        assert_eq!(
            text_of(error, source).trim_start(),
            format!("/*{break_}*/ + b"),
            "{context}"
        );
        // The comment and the `+` token keep their exact spans.
        let comment = first_of_kind(&parsed, NodeKind::BlockComment).expect("comment leaf");
        assert_eq!(
            text_of(comment, source),
            format!("/*{break_}*/"),
            "{context}"
        );
        let plus_at = u32::try_from(text.find('+').expect("the `+`")).unwrap();
        let plus = parsed
            .tokens()
            .find(|node| node.text(source) == Some("+"))
            .expect("the `+` token leaf");
        assert_eq!(plus.span().start(), plus_at, "{context}");
        assert_eq!(plus.span().end(), plus_at + 1, "{context}");
        assert!(parsed.is_lossless(source), "{context}");
        assert_eq!(
            parsed.reprint(source).as_deref(),
            Some(text.as_str()),
            "{context}"
        );
    }
}

#[test]
fn parenthesised_regions_lift_the_line_gate_across_a_multiline_comment() {
    // Inside `( … )` and `[ … ]` an expression continuation is accepted
    // even when the operator token sits on a later physical line.
    for text in [
        "main() {\n    call(a /*\n*/ + b)\n}\n",
        "main() {\n    val x = items[a /*\n*/ + b]\n}\n",
    ] {
        let (_, parsed) = recognised(text);
        assert_eq!(count_of_kind(&parsed, NodeKind::BinaryExpr), 1, "{text:?}");
    }
}

#[test]
fn a_trailing_lambda_does_not_attach_across_a_multiline_comment() {
    for break_ in BREAKS {
        let text = format!("main() {{\n    items.each /*{break_}*/ {{ x: Int -> call(x) }}\n}}\n");
        let context = format!("break {break_:?}");
        let (sources, parsed) = parse_text(&text);
        let source = source_of(&sources, &parsed);
        assert_eq!(
            parse_codes(&parsed),
            vec!["PARSE002"],
            "{context}: {:?}",
            describe(&parsed)
        );
        assert_eq!(count_of_kind(&parsed, NodeKind::LambdaExpr), 0, "{context}");
        // The braces are a block statement; its body cannot start with a
        // type ascription, so the region after `x` is preserved once.
        assert_eq!(count_of_kind(&parsed, NodeKind::Block), 2, "{context}");
        let error = first_of_kind(&parsed, NodeKind::Error).expect("error region");
        assert_eq!(
            text_of(error, source).trim_start(),
            ": Int -> call(x)",
            "{context}"
        );
        assert!(parsed.is_lossless(source), "{context}");
        assert_eq!(
            parsed.reprint(source).as_deref(),
            Some(text.as_str()),
            "{context}"
        );
    }
}

#[test]
fn a_trailing_lambda_attaches_across_an_inline_block_comment() {
    let (sources, parsed) =
        recognised("main() {\n    items.each /* c */ { x: Int -> call(x) }\n}\n");
    let source = source_of(&sources, &parsed);
    assert_eq!(count_of_kind(&parsed, NodeKind::LambdaExpr), 1);
    let stmt = first_of_kind(&parsed, NodeKind::ExprStmt).expect("expression statement");
    assert_eq!(
        text_of(stmt, source).trim_start(),
        "items.each /* c */ { x: Int -> call(x) }"
    );
}

#[test]
fn a_nested_function_body_does_not_attach_across_a_multiline_comment() {
    for break_ in BREAKS {
        let text = format!("main() {{\n    inner() /*{break_}*/ {{ call() }}\n}}\n");
        let context = format!("break {break_:?}");
        let (sources, parsed) = parse_text(&text);
        let source = source_of(&sources, &parsed);
        // The declaration probe is speculative: the failed same-line body
        // leaves no diagnostic, and the braces become a block statement.
        assert_eq!(
            parse_codes(&parsed),
            Vec::<&str>::new(),
            "{context}: {:?}",
            describe(&parsed)
        );
        assert_eq!(
            count_of_kind(&parsed, NodeKind::FunctionDecl),
            1,
            "{context}"
        );
        assert_eq!(count_of_kind(&parsed, NodeKind::Block), 2, "{context}");
        // `inner()` is an expression statement and the braces a block
        // statement; the block's body statement `call()` is the second.
        assert_eq!(count_of_kind(&parsed, NodeKind::ExprStmt), 2, "{context}");
        let stmt = first_of_kind(&parsed, NodeKind::ExprStmt).expect("expression statement");
        assert_eq!(text_of(stmt, source).trim_start(), "inner()", "{context}");
        let blocks = nodes_of_kind(&parsed, NodeKind::Block);
        assert_eq!(
            text_of(blocks[1], source).trim_start(),
            format!("/*{break_}*/ {{ call() }}"),
            "{context}"
        );
        assert!(parsed.is_lossless(source), "{context}");
        assert_eq!(
            parsed.reprint(source).as_deref(),
            Some(text.as_str()),
            "{context}"
        );
    }
}

#[test]
fn a_nested_function_body_attaches_across_an_inline_block_comment() {
    let (sources, parsed) = recognised("main() {\n    inner() /* c */ { call() }\n}\n");
    let source = source_of(&sources, &parsed);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::Block), 2);
    // Only the body statement `call()` is an expression statement.
    let statements = nodes_of_kind(&parsed, NodeKind::ExprStmt);
    assert_eq!(statements.len(), 1);
    assert_eq!(text_of(statements[0], source).trim_start(), "call()");
}

#[test]
fn statement_separation_uses_physical_breaks() {
    // `return` takes its optional expression only on the same line.
    let (sources, parsed) = parse_text("main() {\n    return /*\n*/ value\n}\n");
    let source = source_of(&sources, &parsed);
    assert_eq!(
        parse_codes(&parsed),
        Vec::<&str>::new(),
        "{:?}",
        describe(&parsed)
    );
    let ret = first_of_kind(&parsed, NodeKind::ReturnStmt).expect("return statement");
    assert_eq!(text_of(ret, source).trim_start(), "return");
    let stmt = first_of_kind(&parsed, NodeKind::ExprStmt).expect("expression statement");
    assert_eq!(text_of(stmt, source).trim_start(), "/*\n*/ value");

    // A `val` right-hand side does not continue across the comment.
    let (sources, parsed) = parse_text("main() {\n    val x = 1 /*\n*/ + 2\n}\n");
    let source = source_of(&sources, &parsed);
    assert_eq!(
        parse_codes(&parsed),
        vec!["PARSE002"],
        "{:?}",
        describe(&parsed)
    );
    let val = first_of_kind(&parsed, NodeKind::VarDeclStmt).expect("val statement");
    assert_eq!(text_of(val, source).trim_start(), "val x = 1");
    let error = first_of_kind(&parsed, NodeKind::Error).expect("error region");
    assert_eq!(text_of(error, source).trim_start(), "/*\n*/ + 2");

    // The assignment operator itself is line-gated at statement level.
    let (sources, parsed) = parse_text("main() {\n    x /*\n*/ = b\n}\n");
    let source = source_of(&sources, &parsed);
    assert_eq!(
        parse_codes(&parsed),
        vec!["PARSE002"],
        "{:?}",
        describe(&parsed)
    );
    assert_eq!(count_of_kind(&parsed, NodeKind::AssignExpr), 0);
    let error = first_of_kind(&parsed, NodeKind::Error).expect("error region");
    assert_eq!(text_of(error, source).trim_start(), "/*\n*/ = b");

    // The right-hand side of an assignment must start on the same line.
    let (sources, parsed) = parse_text("main() {\n    x = /*\n*/ b\n}\n");
    let source = source_of(&sources, &parsed);
    assert_eq!(
        parse_codes(&parsed),
        vec!["PARSE002"],
        "{:?}",
        describe(&parsed)
    );
    assert_eq!(count_of_kind(&parsed, NodeKind::AssignExpr), 1);
    let texts: Vec<&str> = nodes_of_kind(&parsed, NodeKind::ExprStmt)
        .into_iter()
        .map(|node| text_of(node, source).trim_start())
        .collect();
    assert_eq!(texts, vec!["x =", "/*\n*/ b"]);
}

#[test]
fn committed_keyword_continuations_ignore_the_physical_break() {
    // `else` is a committed continuation of its `if` and is deliberately
    // not line-gated, so the multiline comment does not detach it.
    let (_, parsed) = recognised("main() {\n    if (c) x /*\n*/ else y\n}\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::IfStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ExprStmt), 2);
}

#[test]
fn unicode_line_lookalikes_are_not_physical_terminators() {
    // U+00A0 (no-break space), U+2028 (line separator) and U+2029
    // (paragraph separator) are not accepted physical terminators: each is
    // an unexpected character (LEX005) and the continuation after it stays
    // on the same physical line.
    for lookalike in ['\u{00A0}', '\u{2028}', '\u{2029}'] {
        let text = format!("main() {{\n    val x = a {lookalike} + b\n}}\n");
        let (_, parsed) = parse_text(&text);
        assert_eq!(
            all_codes(&parsed),
            vec!["LEX005"],
            "{lookalike:?}: {:?}",
            describe(&parsed)
        );
        assert_eq!(
            count_of_kind(&parsed, NodeKind::BinaryExpr),
            1,
            "{lookalike:?}"
        );
    }
}

#[test]
fn a_line_break_inside_an_error_region_separates_significant_tokens() {
    // A refused triple-quoted string is one recoverable lexical-error
    // region; a raw line break inside it still separates the surrounding
    // significant tokens.
    for break_ in BREAKS {
        let text = format!("main() {{\n    a \"\"\"x{break_}y\"\"\" + b\n}}\n");
        let context = format!("break {break_:?}");
        let (sources, parsed) = parse_text(&text);
        let source = source_of(&sources, &parsed);
        assert_eq!(
            all_codes(&parsed),
            vec!["LEX009", "PARSE002"],
            "{context}: {:?}",
            describe(&parsed)
        );
        assert_eq!(parse_codes(&parsed), vec!["PARSE002"], "{context}");
        assert_eq!(count_of_kind(&parsed, NodeKind::BinaryExpr), 0, "{context}");
        let lex = first_of_kind(&parsed, NodeKind::LexError).expect("lexer error region");
        assert_eq!(
            text_of(lex, source),
            format!("\"\"\"x{break_}y\"\"\""),
            "{context}"
        );
        let error = first_of_kind(&parsed, NodeKind::Error).expect("parser error region");
        assert_eq!(
            text_of(error, source).trim_start(),
            format!("\"\"\"x{break_}y\"\"\" + b"),
            "{context}"
        );
        assert!(parsed.is_lossless(source), "{context}");
        assert_eq!(
            parsed.reprint(source).as_deref(),
            Some(text.as_str()),
            "{context}"
        );
    }

    // Control: with no line break inside the region, the `+` is on the same
    // physical line and the continuation is taken.
    let (_, parsed) = parse_text("main() {\n    a \"\"\"x y\"\"\" + b\n}\n");
    assert_eq!(
        all_codes(&parsed),
        vec!["LEX009"],
        "{:?}",
        describe(&parsed)
    );
    assert_eq!(count_of_kind(&parsed, NodeKind::BinaryExpr), 1);
}

#[test]
fn every_regression_input_round_trips_and_parses_deterministically() {
    let mut inputs = vec![
        "main() {\n    val x = a /* inline */ + b\n}\n".to_owned(),
        "main() {\n    items.each /* c */ { x: Int -> call(x) }\n}\n".to_owned(),
        "main() {\n    inner() /* c */ { call() }\n}\n".to_owned(),
        "main() {\n    if (c) x /*\n*/ else y\n}\n".to_owned(),
        "main() {\n    return /*\n*/ value\n}\n".to_owned(),
        "main() {\n    a \"\"\"x y\"\"\" + b\n}\n".to_owned(),
    ];
    for break_ in BREAKS {
        inputs.push(format!("main() {{\n    val x = a /*{break_}*/ + b\n}}\n"));
        inputs.push(format!(
            "main() {{\n    items.each /*{break_}*/ {{ x: Int -> call(x) }}\n}}\n"
        ));
        inputs.push(format!(
            "main() {{\n    inner() /*{break_}*/ {{ call() }}\n}}\n"
        ));
        inputs.push(format!("main() {{\n    a \"\"\"x{break_}y\"\"\" + b\n}}\n"));
        inputs.push(format!("main() {{\n    call(a /*{break_}*/ + b)\n}}\n"));
        inputs.push(format!(
            "main() {{\n    val x = items[a /*{break_}*/ + b]\n}}\n"
        ));
    }
    for text in &inputs {
        let (first_sources, first) = parse_text(text);
        let (second_sources, second) = parse_text(text);
        let first_source = source_of(&first_sources, &first);
        let second_source = source_of(&second_sources, &second);
        assert!(first.is_lossless(first_source), "not lossless: {text:?}");
        assert_eq!(
            first.reprint(first_source).as_deref(),
            Some(text.as_str()),
            "reprint: {text:?}"
        );
        assert_eq!(
            second.reprint(second_source).as_deref(),
            Some(text.as_str()),
            "second reprint: {text:?}"
        );
        assert_eq!(
            all_codes(&first),
            all_codes(&second),
            "codes differ: {text:?}"
        );
        assert_eq!(
            signature(&first),
            signature(&second),
            "structure differs: {text:?}"
        );
    }
}
