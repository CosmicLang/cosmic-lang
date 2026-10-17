//! The refusal contract of `docs/spec/g1-s-syntax-decision-package.md` §4.
//!
//! Each of the nine unsupported families is refused with **exactly one**
//! diagnostic, its consumed region is preserved in a named error node, and
//! parsing continues: valid neighbours on both sides still parse. Labelled
//! `break`/`continue` (family 8) gets its dedicated `PARSE004`; the other
//! eight families get `PARSE001`.

mod common;

use common::{all_codes, count_of_kind, parse_codes, parse_text, recognised};
use cosmic_syntax::NodeKind;

#[test]
fn family1_test_declaration_is_refused() {
    let text = "test \"adds\" {\n  check(1)\n}\nmain() {}\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::TestDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 1);
}

#[test]
fn family2_application_declaration_is_refused() {
    let text = "application {\n  run()\n}\nmain() {}\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::ApplicationDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 1);
}

#[test]
fn family3_infra_declaration_is_refused() {
    let text = "infra \"prod\" {\n  deploy()\n}\nmain() {}\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::InfraDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 1);
}

#[test]
fn family4_extern_declaration_is_refused_and_neighbours_parse() {
    let text = "first() { call() }\nextern Int nativeAdd(Int a, Int b)\nsecond() { call2() }\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::ExternDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 2);
}

#[test]
fn family5_foreign_module_declaration_is_refused() {
    let text = "foreign module \"std\" {\n  call()\n}\nmain() {}\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::ForeignModuleDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 1);
}

#[test]
fn family6_entity_declaration_is_refused() {
    let text = "entity User extends Base {\n  Int id = 0\n}\nmain() {}\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::EntityDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 1);
}

#[test]
fn family7_annotations_are_refused_at_top_level_and_in_members() {
    // Simple name, then a qualified name with argument list.
    let text = "@Deprecated\nmain() {}\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::Annotation), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 1);

    let text = "@a.b.Marker(1, \"x\")\nmain() {}\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::Annotation), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 1);

    // A member annotation: the field after it still parses.
    let text = "class A {\n  @Marker\n  Int x = 1\n}\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::Annotation), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FieldDecl), 1);
}

#[test]
fn family9_array_literal_brace_form_is_refused() {
    // Right-hand side of a declaration.
    let (_, parsed) = parse_text("main() { val xs = {1, 2, 3} }\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::ArrayLiteral), 1);

    // Argument position.
    let (_, parsed) = parse_text("main() { call({1, 2}) }\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::ArrayLiteral), 1);

    // Empty brace form.
    let (_, parsed) = parse_text("main() { val xs = {} }\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::ArrayLiteral), 1);
}

#[test]
fn family9_arrow_arm_block_bodies_fall_to_the_array_refusal() {
    // The brace form is not part of the accepted arrow-arm production; it is
    // not derivable, so it is refused as the family-9 brace form.
    let (_, parsed) = parse_text("main() { val y = switch (x) { case 1 -> { call() } } }\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::ArrayLiteral), 1);
}

#[test]
fn family8_labelled_jumps_are_refused_with_parse004() {
    let text = "main() {\n  while (ready) {\n    break outer\n  }\n  continue outer\n}\n";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE004", "PARSE004"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::Error), 2);
    for diagnostic in parsed.diagnostics().iter() {
        let message = diagnostic.message();
        assert!(message.contains("labelled"), "{message}");
        assert!(message.contains("unlabelled"), "{message}");
        assert!(message.contains("S5"), "{message}");
    }

    // Unlabelled forms stay recognised (`F1`).
    let (_, parsed) = recognised("main() {\n  while (ready) {\n    break\n  }\n  continue\n}\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::BreakStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ContinueStmt), 1);
}

#[test]
fn reserved_invalid_function_words_are_refused_with_the_specific_message() {
    for keyword in ["fun", "fn", "func"] {
        let text = format!("main() {{}}\n{keyword} other() {{}}\nnext() {{}}\n");
        let (_, parsed) = parse_text(&text);
        assert_eq!(parse_codes(&parsed), vec!["PARSE002"], "{keyword}");
        let message = parsed
            .diagnostics()
            .iter()
            .next()
            .expect("the refusal diagnostic")
            .message()
            .to_string();
        assert!(message.contains("reserved-invalid"), "{keyword}: {message}");
        assert!(message.contains("SG-001"), "{keyword}: {message}");
        // The refusal synchronises at the next statement boundary: the
        // following declarations still parse.
        assert_eq!(
            count_of_kind(&parsed, NodeKind::FunctionDecl),
            2,
            "{keyword}"
        );
    }
}

#[test]
fn every_refusal_is_exactly_one_parse001_and_keeps_lex_codes_untouched() {
    let fixtures = [
        "test \"t\" {\n}\n",
        "application {\n}\n",
        "infra \"i\" {\n}\n",
        "extern Int nativeAdd(Int a)\n",
        "foreign module \"std\" {\n}\n",
        "entity User {\n}\n",
        "@Marker\nmain() {}\n",
        "main() { val xs = {1} }\n",
    ];
    for text in fixtures {
        let (_, parsed) = parse_text(text);
        assert_eq!(parse_codes(&parsed), vec!["PARSE001"], "{text:?}");
        assert_eq!(all_codes(&parsed), vec!["PARSE001"], "{text:?}");
    }
}
