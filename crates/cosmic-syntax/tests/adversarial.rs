//! A3 adversarial regressions: pathological statement workloads and repeated
//! implementation-guard trips stay bounded, deterministic and lossless.
//!
//! These inputs exercise characteristics accepted in owner decision
//! 003-A2-R1-FINAL (C): per-trip `PARSE005` reporting with repeated
//! diagnostics, and the bounded `types_lines`-style workload that the A3
//! measurement script exercises out-of-CI. Ordinary CI keeps the sizes
//! deliberately small; the extreme workloads are measured separately.

mod common;

use common::{all_codes, parse_text, source_of};
use cosmic_syntax::{NodeKind, ParsedFile};

/// `(kind, span)` for every node: the structural signature used for
/// determinism checks.
fn signature(parsed: &ParsedFile) -> Vec<(NodeKind, u32, u32)> {
    parsed
        .nodes()
        .map(|node| (node.kind(), node.span().start(), node.span().end()))
        .collect()
}

#[test]
fn pathological_if_lines_stay_bounded_deterministic_and_lossless() {
    // 5_000 unterminated `if (c)` lines: the per-trip PARSE005 workload.
    let mut text = String::from("main() {\n");
    for _ in 0..5_000 {
        text.push_str("    if (c)\n");
    }
    text.push_str("}\n");

    let (sources, parsed) = parse_text(&text);
    let source = source_of(&sources, &parsed);
    assert!(
        parsed.is_lossless(source),
        "not lossless: {:?}",
        common::describe(&parsed)
    );
    assert_eq!(parsed.reprint(source).as_deref(), Some(text.as_str()));
    let codes = all_codes(&parsed);
    let trips = codes.iter().filter(|code| **code == "PARSE005").count();
    assert!(
        trips >= 1,
        "the implementation guard must trip at least once: {codes:?}"
    );
    assert!(
        trips <= 500,
        "per-trip reporting must stay bounded; got {trips}"
    );
    assert!(
        codes.iter().all(|code| code.starts_with("PARSE")),
        "unexpected lexical diagnostics: {codes:?}"
    );

    // Determinism: an identical parse produces the identical diagnostic
    // sequence and the identical structural signature.
    let (_, again) = parse_text(&text);
    assert_eq!(all_codes(&again), codes);
    assert_eq!(signature(&again), signature(&parsed));
}

#[test]
fn adversarial_statement_soup_is_deterministic_and_lossless() {
    let mut text = String::from("main() {\n");
    for index in 0..2_000 {
        match index % 5 {
            0 => text.push_str("    val x = 1\n"),
            1 => text.push_str("    ~\n"),
            2 => text.push_str("    if (c) {\n"),
            3 => text.push_str("    }\n"),
            _ => text.push_str("    call()\n"),
        }
    }
    text.push_str("}\n");

    let (sources, parsed) = parse_text(&text);
    let source = source_of(&sources, &parsed);
    assert!(parsed.is_lossless(source));
    assert_eq!(parsed.reprint(source).as_deref(), Some(text.as_str()));
    let codes = all_codes(&parsed);
    assert!(
        codes.iter().any(|code| *code == "LEX005"),
        "the `~` lines must produce unexpected-character regions"
    );
    assert!(
        codes.len() <= 2_000,
        "diagnostics must stay bounded per input element; got {}",
        codes.len()
    );

    let (_, again) = parse_text(&text);
    assert_eq!(all_codes(&again), codes);
    assert_eq!(signature(&again), signature(&parsed));
}
