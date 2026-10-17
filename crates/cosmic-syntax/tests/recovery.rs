//! Recovery obligations (`docs/spec/g1-s-syntax-decision-package.md` §6):
//! more than one independent error in a single run, unclosed delimiters,
//! bounded progress on malformed input, and no false successful parse.

mod common;

use common::{count_of_kind, parse_codes, parse_text, source_of};
use cosmic_syntax::NodeKind;

#[test]
fn three_independent_errors_are_reported_in_one_run() {
    let text = "main() {\n  val a = ?\n  throw ?\n  @Bad\n}\n";
    let (sources, parsed) = parse_text(text);
    assert_eq!(
        parse_codes(&parsed),
        vec!["PARSE002", "PARSE002", "PARSE002"]
    );
    assert_eq!(count_of_kind(&parsed, NodeKind::Error), 3);
    // Canonical order: strictly increasing diagnostic spans.
    let spans: Vec<u32> = parsed
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.primary().span().start())
        .collect();
    assert!(spans.windows(2).all(|window| window[0] < window[1]));
    // Bytes preserved, structure intact around the defects.
    let source = source_of(&sources, &parsed);
    assert!(parsed.is_lossless(source));
    assert_eq!(parsed.reprint(source).as_deref(), Some(text));
}

#[test]
fn unclosed_delimiters_report_parse003_once_per_construct() {
    let cases = [
        "main() {\n",
        "main() {\n  call(2\n",
        "main() {\n  val x = (2\n",
        "main() {\n  val f = (x: Int) -> \n",
        "main() {\n  switch (x) {\n    case 1: call()\n",
    ];
    for text in cases {
        let (sources, parsed) = parse_text(text);
        assert_eq!(parse_codes(&parsed), vec!["PARSE003"], "{text:?}");
        // The diagnostic points at the end of the input.
        assert_eq!(
            parsed
                .diagnostics()
                .iter()
                .next()
                .expect("one diagnostic")
                .primary()
                .span()
                .start(),
            text.len() as u32,
            "{text:?}"
        );
        let source = source_of(&sources, &parsed);
        assert!(parsed.is_lossless(source), "{text:?}");
        assert_eq!(parsed.reprint(source).as_deref(), Some(text), "{text:?}");
    }
}

#[test]
fn stray_closers_are_reported_and_preserved() {
    let text = "main() {\n  )\n}\n";
    let (sources, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE002"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::Error), 1);
    let source = source_of(&sources, &parsed);
    assert!(parsed.is_lossless(source));
    assert_eq!(parsed.reprint(source).as_deref(), Some(text));
}

#[test]
fn no_false_success_while_errors_remain() {
    let (_, parsed) = parse_text("main() { ] }\n");
    assert!(!parsed.diagnostics().is_empty());
    assert!(parsed.has_errors());

    let (_, clean) = parse_text("main() { }\n");
    assert!(clean.diagnostics().is_empty());
    assert!(!clean.has_errors());
}

#[test]
fn reserved_invalid_words_in_expression_position_are_reported() {
    let (_, parsed) = parse_text("main() { val x = fun }\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE002"]);
    let message = parsed
        .diagnostics()
        .iter()
        .next()
        .expect("the refusal diagnostic")
        .message()
        .to_string();
    assert!(message.contains("reserved-invalid"), "{message}");
    assert!(message.contains("SG-001"), "{message}");
}

#[test]
fn recovery_after_a_refusal_keeps_later_structure() {
    let text = "\
first() { call() }\n\
extern Int nativeAdd(Int a)\n\
second() {\n\
  val x = ?\n\
}\n\
third() { call2() }\n\
";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001", "PARSE002"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 3);
}

#[test]
fn pathological_nesting_terminates() {
    // Two hundred unclosed parentheses: exactly one missing expression.
    let text = format!("main() {{ val x = {}\n", "(".repeat(200));
    let (sources, parsed) = parse_text(&text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE003"]);
    let source = source_of(&sources, &parsed);
    assert!(parsed.is_lossless(source));
    assert_eq!(parsed.reprint(source).as_deref(), Some(text.as_str()));

    // Two hundred nested valid blocks: clean parse, no stack overflow.
    let text = format!("main() {{\n{}{}\n}}\n", "{".repeat(200), "}".repeat(200));
    let (sources, parsed) = parse_text(&text);
    assert!(parsed.diagnostics().is_empty());
    let source = source_of(&sources, &parsed);
    assert!(parsed.is_lossless(source));

    // Two hundred nested valid parentheses close cleanly.
    let text = format!(
        "main() {{ val x = {}1{}\n}}\n",
        "(".repeat(200),
        ")".repeat(200)
    );
    let (sources, parsed) = parse_text(&text);
    assert!(parsed.diagnostics().is_empty());
    let source = source_of(&sources, &parsed);
    assert!(parsed.is_lossless(source));
}
