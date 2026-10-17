//! Deterministic property tests over malformed input (`S15` Q3; G1-S §6):
//! parsing terminates, never panics, never loses or fabricates bytes, and is
//! a deterministic function of the token stream.
//!
//! No external dependency is used: cases come from a fixed-seed xorshift
//! generator over an adversarial chunk alphabet, and every assertion reports
//! the seed, case index and input, so a counterexample is reproducible and
//! can be turned into a regression fixture.

mod common;

use common::{parse_text, source_of};
use cosmic_syntax::{NodeKind, ParsedFile};

/// Fixed seed: the suite must be reproducible from a clean checkout.
const SEED: u64 = 0x00A2_5A2C_5EED_2026;

/// Adversarial chunks: keywords, separators, brackets, literals, trivia and
/// the parser's decision surface (lambdas, generics, switch labels).
const CHUNKS: &[&str] = &[
    "main",
    "f",
    "x",
    "a",
    "Item",
    "val",
    "var",
    "Int",
    "String",
    "void",
    "fun",
    "fn",
    "func",
    "test",
    "application",
    "infra",
    "extern",
    "foreign",
    "module",
    "entity",
    "sealed",
    "class",
    "record",
    "enum",
    "interface",
    "constructor",
    "if",
    "else",
    "while",
    "do",
    "for",
    "in",
    "return",
    "throw",
    "break",
    "continue",
    "switch",
    "case",
    "default",
    "try",
    "catch",
    "finally",
    "new",
    "spawn",
    "await",
    "as",
    "instanceof",
    "true",
    "false",
    "null",
    "this",
    "super",
    "1",
    "2.5",
    "\"s\"",
    "'c'",
    "\u{feff}",
    "(",
    ")",
    "{",
    "}",
    "[",
    "]",
    "<",
    ">",
    "<<",
    ">>",
    ">>>",
    "=",
    "==",
    "!",
    "!=",
    "+",
    "-",
    "*",
    "/",
    "%",
    "&",
    "&&",
    "|",
    "||",
    "^",
    "?",
    ":",
    ",",
    ";",
    ".",
    "@",
    "->",
    "=>",
    "\n",
    "\r\n",
    " ",
    "\t",
    "//c\n",
    "/*c*/",
    "é",
    "日",
    "\"🚀\"",
    "'",
    "\"",
    "/*",
    "*/",
];

/// xorshift64*: tiny, deterministic, dependency-free.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

fn generate(rng: &mut Rng, max_chunks: usize) -> String {
    let count = rng.below(max_chunks + 1);
    let mut text = String::new();
    for _ in 0..count {
        text.push_str(CHUNKS[rng.below(CHUNKS.len())]);
    }
    text
}

/// The tree shape as a comparable projection (kind and span, in pre-order).
fn shape(parsed: &ParsedFile) -> Vec<(NodeKind, u32, u32)> {
    parsed
        .nodes()
        .map(|node| (node.kind(), node.span().start(), node.span().end()))
        .collect()
}

/// The full diagnostic surface as a comparable projection.
fn diagnostics(parsed: &ParsedFile) -> Vec<(String, u32, u32)> {
    parsed
        .all_diagnostics()
        .iter()
        .map(|diagnostic| {
            let span = diagnostic.primary().span();
            (
                diagnostic.code().as_str().to_string(),
                span.start(),
                span.end(),
            )
        })
        .collect()
}

fn check(case: usize, text: &str) {
    let (sources, first) = parse_text(text);
    let source = source_of(&sources, &first);

    assert!(
        first.is_lossless(source),
        "not lossless (seed={SEED:#x} case={case}): {text:?}"
    );
    assert_eq!(
        first.reprint(source).as_deref(),
        Some(text),
        "reprint mismatch (seed={SEED:#x} case={case}): {text:?}"
    );
    // No cascades: every diagnostic needs a distinct consumed byte.
    assert!(
        first.diagnostics().len() <= text.len() + 1,
        "diagnostic cascade (seed={SEED:#x} case={case}): {text:?}"
    );

    // Determinism: a second parse of the same bytes is identical.
    let second = cosmic_syntax::parse(source);
    assert_eq!(
        shape(&first),
        shape(&second),
        "nondeterministic tree (seed={SEED:#x} case={case}): {text:?}"
    );
    assert_eq!(
        diagnostics(&first),
        diagnostics(&second),
        "nondeterministic diagnostics (seed={SEED:#x} case={case}): {text:?}"
    );
}

#[test]
fn random_malformed_inputs_are_lossless_deterministic_and_terminate() {
    let mut rng = Rng::new(SEED);
    for case in 0..2000 {
        let text = generate(&mut rng, 8);
        check(case, &text);
    }
}

#[test]
fn every_char_boundary_prefix_is_lossless_and_reprints() {
    let mut rng = Rng::new(SEED ^ 0x2026);
    for case in 0..300 {
        let text = generate(&mut rng, 6);
        let mut boundaries: Vec<usize> = text.char_indices().map(|(index, _)| index).collect();
        boundaries.push(text.len());
        for end in boundaries {
            let prefix = &text[..end];
            let (sources, parsed) = parse_text(prefix);
            let source = source_of(&sources, &parsed);
            assert!(
                parsed.is_lossless(source),
                "prefix not lossless (seed={SEED:#x} case={case} end={end}): {text:?}"
            );
            assert_eq!(
                parsed.reprint(source).as_deref(),
                Some(prefix),
                "prefix reprint (seed={SEED:#x} case={case} end={end}): {text:?}"
            );
        }
    }
}

#[test]
fn adversarial_keyword_salad_never_panics_and_stays_lossless() {
    let inputs = [
        "fun fn func",
        "(",
        ")",
        "{(<[",
        ">}])",
        "switch (x) { case",
        "switch (x) { case 1 ->",
        "class",
        "class A",
        "class A {",
        "package",
        "package a.",
        "@",
        "@a",
        "@a(",
        "val",
        "val x",
        "val x =",
        "(x) -> x",
        "{ x -> x }",
        "(x: Int) ->",
        "() ->",
        "a < b > c",
        "a >> b >>> c",
        "Map<String, List<Int>>",
        "'",
        "\"",
        "/*",
        "//",
        "\u{feff}",
        "\u{feff}\u{feff}",
        "for (;;) {}",
        "for (var i = 0; i < 10; i = i + 1) {}",
        "break outer",
        "continue outer",
        "x = ",
        "= x",
        "a +",
        "+ a",
        "case 1:",
        "default:",
        "catch (Error e)",
        "finally",
        "while",
        "extern",
        "test \"t\" {",
    ];
    for (case, text) in inputs.into_iter().enumerate() {
        check(case, text);
    }
}
