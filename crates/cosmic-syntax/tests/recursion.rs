//! Recursion-safety regressions (A2-R1-02).
//!
//! Before this repair a small valid input — a few thousand nested
//! parentheses — aborted the whole process with a stack overflow. Measured
//! on the development machine: the deepest recursive chain (a parenthesised
//! expression) uses ≈ 600 bytes of stack per level; a 1 MiB thread passed
//! 1 700 levels and overflowed by 1 736, a 2 MiB thread (the Rust std
//! default) passed 3 400 and overflowed by 3 461 — with or without closing
//! brackets, malformed input failed like valid input, and the probe aborted
//! the process rather than reporting.
//!
//! The guard bounds the recursive-descent paths deterministically: reaching
//! the limit reports one `PARSE005` and consumes the balanced region
//! iteratively, so no input can exhaust the stack. `PARSE005` is an
//! **implementation resource limit, not a syntax rule**.
//!
//! The extreme case is validated in an isolated child process
//! (re-executing this test binary) so that an (unexpected) abort cannot
//! take the rest of the suite with it.

mod common;

use common::{count_of_kind, describe, nodes_of_kind, parse_codes, parse_text, source_of, text_of};
use cosmic_syntax::{NodeKind, ParsedFile};

/// A `main` whose body binds the given expression to `val x`.
fn expression_body(expression: &str) -> String {
    format!("main() {{\n    val x = {expression}\n}}\n")
}

/// `(kind, span)` for every node, in pre-order: the structural signature.
fn signature(parsed: &ParsedFile) -> Vec<(NodeKind, u32, u32)> {
    parsed
        .nodes()
        .map(|node| (node.kind(), node.span().start(), node.span().end()))
        .collect()
}

/// One `PARSE005`, full losslessness, byte-for-byte reprint, and an
/// identical second parse (determinism).
fn assert_bounded_and_lossless(text: &str, shape: &str) {
    let (sources, parsed) = parse_text(text);
    let source = source_of(&sources, &parsed);
    assert_eq!(
        parse_codes(&parsed),
        vec!["PARSE005"],
        "{shape}: {:?}",
        describe(&parsed)
    );
    assert!(parsed.is_lossless(source), "{shape}: not lossless");
    assert_eq!(
        parsed.reprint(source).as_deref(),
        Some(text),
        "{shape}: reprint"
    );
    let (_, again) = parse_text(text);
    assert_eq!(
        parse_codes(&again),
        vec!["PARSE005"],
        "{shape}: second parse"
    );
    assert_eq!(
        signature(&parsed),
        signature(&again),
        "{shape}: structure differs between parses"
    );
}

#[test]
fn nesting_below_the_limit_is_fully_recognised() {
    let text = expression_body(&format!("{}1{}", "(".repeat(200), ")".repeat(200)));
    let (sources, parsed) = parse_text(&text);
    let source = source_of(&sources, &parsed);
    assert_eq!(
        parse_codes(&parsed),
        Vec::<&str>::new(),
        "{:?}",
        describe(&parsed)
    );
    assert_eq!(parsed.reprint(source).as_deref(), Some(text.as_str()));
}

#[test]
fn deep_parentheses_report_the_implementation_limit_once() {
    let text = expression_body(&format!("{}1{}", "(".repeat(400), ")".repeat(400)));
    assert_bounded_and_lossless(&text, "balanced parentheses");

    let (sources, parsed) = parse_text(&text);
    let source = source_of(&sources, &parsed);
    let diagnostic = parsed.diagnostics().iter().next().expect("one diagnostic");
    let message = diagnostic.message();
    assert!(message.contains("implementation limit"), "{message}");
    assert!(message.contains("not a syntax rule"), "{message}");
    // The diagnostic points at the parenthesis where the bound was met:
    // one guarded entry for the statement, plus two guarded entries per
    // level (binary operand + unary), so the trip is the 256th `(`.
    let span = diagnostic.primary().span();
    assert_eq!(source.text_of(span), Some("("));
    let run_start = u32::try_from(text.find("((((").expect("the open run")).unwrap();
    assert_eq!(span.start(), run_start + 255);
    assert_eq!(span.end(), run_start + 256);
    // The parens below the bound stay structured: 200 fully parsed levels
    // plus the error region, and nothing else.
    assert_eq!(count_of_kind(&parsed, NodeKind::Error), 1);
}

#[test]
fn deep_unary_chains_are_bounded() {
    let text = expression_body(&format!("{}1", "!".repeat(40_000)));
    assert_bounded_and_lossless(&text, "unary chain");
}

#[test]
fn deep_brace_statement_nesting_is_bounded() {
    let text = format!(
        "main() {{\n{}x{}\n}}\n",
        "{".repeat(10_000),
        "}".repeat(10_000)
    );
    assert_bounded_and_lossless(&text, "nested blocks");
}

#[test]
fn deep_type_arguments_are_bounded() {
    let text = format!(
        "main() {{\n    val x: {}Int{}\n}}\n",
        "A<".repeat(5_000),
        ">".repeat(5_000)
    );
    let (sources, parsed) = parse_text(&text);
    let source = source_of(&sources, &parsed);
    // The guard trips inside the type arguments; the declaration that owns
    // them then reports its missing `=` once — a consequence of the
    // resource-limit region, not a second defect.
    assert_eq!(
        parse_codes(&parsed),
        vec!["PARSE005", "PARSE002"],
        "{:?}",
        describe(&parsed)
    );
    assert!(parsed.is_lossless(source));
    assert_eq!(parsed.reprint(source).as_deref(), Some(text.as_str()));
}

#[test]
fn several_independent_deep_regions_all_recover() {
    let region = format!("{}1{}", "(".repeat(400), ")".repeat(400));
    let text = format!(
        "main() {{\n    val a = {region}\n    val b = {region}\n    val c = {region}\n    call(a)\n}}\n"
    );
    let (sources, parsed) = parse_text(&text);
    let source = source_of(&sources, &parsed);
    assert_eq!(
        parse_codes(&parsed),
        vec!["PARSE005"; 3],
        "{:?}",
        describe(&parsed)
    );
    assert_eq!(count_of_kind(&parsed, NodeKind::VarDeclStmt), 3);
    // Recovery continues: the final statement is fully recognised.
    let statements = nodes_of_kind(&parsed, NodeKind::ExprStmt);
    assert_eq!(statements.len(), 1);
    assert_eq!(text_of(statements[0], source).trim_start(), "call(a)");
    assert!(parsed.is_lossless(source));
    assert_eq!(parsed.reprint(source).as_deref(), Some(text.as_str()));
}

/// Set in the child process that re-executes this test binary.
const CHILD_ENV: &str = "COSMIC_SYNTAX_RECURSION_CHILD";

/// Two hundred thousand levels — far beyond any stack — must stay bounded
/// and lossless. Valid, malformed and unary shapes are all covered.
#[test]
fn extreme_nesting_is_bounded_in_an_isolated_process() {
    if std::env::var_os(CHILD_ENV).is_some() {
        const EXTREME: usize = 200_000;
        for (label, text) in [
            (
                "parentheses",
                expression_body(&format!("{}1{}", "(".repeat(EXTREME), ")".repeat(EXTREME))),
            ),
            (
                "unary",
                expression_body(&format!("{}1", "!".repeat(EXTREME))),
            ),
            ("unclosed", expression_body(&"(".repeat(EXTREME))),
        ] {
            assert_bounded_and_lossless(&text, label);
        }
        return;
    }
    let exe = std::env::current_exe().expect("the test binary path");
    let status = std::process::Command::new(exe)
        .args([
            "--exact",
            "extreme_nesting_is_bounded_in_an_isolated_process",
            "--nocapture",
        ])
        .env(CHILD_ENV, "1")
        .status()
        .expect("spawn the isolated child");
    assert!(status.success(), "the isolated child failed: {status}");
}
