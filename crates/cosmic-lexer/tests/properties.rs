//! Deterministic property tests for malformed input (G1-L D-L10).
//!
//! No external dependency is used: cases come from a fixed-seed xorshift
//! generator over an adversarial alphabet, and every assertion reports the
//! seed, case index and input, so a counterexample is reproducible and can
//! be made into a regression fixture. The properties are:
//!
//! * every input lexes, with a partition that is gapless and reprints the
//!   exact bytes (byte preservation);
//! * diagnostics do not cascade (each needs a distinct consumed byte);
//! * lexing is deterministic for identical input;
//! * every character-boundary prefix of an input is itself lossless
//!   (truncation robustness, including mid-literal and mid-escape).

mod common;

use common::{lex_text, source_of};

/// Fixed seed: the suite must be reproducible from a clean checkout.
const SEED: u64 = 0x00C0_51CC_0DE1_2026;

/// Adversarial alphabet: every construct edge the lexer has to survive.
const ALPHABET: &[char] = &[
    'a',
    'z',
    'A',
    'Z',
    '_',
    '$',
    '0',
    '9',
    'f',
    'F',
    'd',
    'D',
    'l',
    'L',
    'x',
    'X',
    'e',
    'E',
    'u',
    'b',
    'c',
    '\n',
    '\r',
    '\t',
    '\u{c}',
    ' ',
    '"',
    '\'',
    '\\',
    '/',
    '*',
    '#',
    '~',
    ':',
    '.',
    ';',
    ',',
    '(',
    ')',
    '{',
    '}',
    '[',
    ']',
    '<',
    '>',
    '=',
    '!',
    '&',
    '|',
    '^',
    '%',
    '@',
    '?',
    '+',
    '-',
    'é',
    'Ω',
    '😀',
    '\u{301}',
    '\u{feff}',
    '\u{2028}',
    '\u{a0}',
    '\u{3000}',
    '\u{10400}',
];

/// Escape-dominated alphabet for the surrogate/escape windows.
const ESCAPE_ALPHABET: &[char] = &[
    '\\', 'u', '"', '\'', 'n', 't', 'r', '0', 'D', 'd', '8', 'C', 'E', 'F', 'A', 'x', 'q', ' ',
    '\n', 'B', '9', 'e',
];

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

fn generate(rng: &mut Rng, alphabet: &[char], max_len: usize) -> String {
    let len = rng.below(max_len + 1);
    let mut out = String::new();
    for _ in 0..len {
        out.push(alphabet[rng.below(alphabet.len())]);
    }
    out
}

/// The shared property: lossless, byte-exact, bounded diagnostics, and
/// deterministic across two independent lexes.
fn check_properties(case: usize, text: &str) {
    let (sources, first) = lex_text(text);
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
    assert!(
        first.diagnostics().len() <= text.len() + 1,
        "diagnostic cascade (seed={SEED:#x} case={case}): {text:?}"
    );
    let (_, second) = lex_text(text);
    assert_eq!(
        first, second,
        "nondeterministic (seed={SEED:#x} case={case}): {text:?}"
    );
}

#[test]
fn random_malformed_inputs_are_lossless_and_deterministic() {
    let mut rng = Rng::new(SEED);
    for case in 0..600 {
        let text = generate(&mut rng, ALPHABET, 64);
        check_properties(case, &text);
    }
}

#[test]
fn escape_dominated_inputs_are_lossless_and_deterministic() {
    let mut rng = Rng::new(SEED ^ 0xE5CA_9E00_2026);
    for case in 0..300 {
        let text = generate(&mut rng, ESCAPE_ALPHABET, 40);
        check_properties(1_000 + case, &text);
    }
}

#[test]
fn every_character_boundary_prefix_is_lossless() {
    let mut rng = Rng::new(SEED ^ 0x9EFF_1A00_2026);
    for case in 0..150 {
        let text = generate(&mut rng, ALPHABET, 48);
        for (offset, _) in text.char_indices() {
            let prefix = &text[..offset];
            let (sources, lexed) = lex_text(prefix);
            let source = source_of(&sources, &lexed);
            assert!(
                lexed.is_lossless(source),
                "prefix not lossless (seed={SEED:#x} case={case} offset={offset}): {prefix:?}"
            );
            assert_eq!(
                lexed.reprint(source).as_deref(),
                Some(prefix),
                "prefix reprint mismatch (seed={SEED:#x} case={case} offset={offset})"
            );
        }
    }
}

#[test]
fn generated_literals_never_panic_the_escape_windows() {
    // Strings built around partial unicode escapes exercise every window
    // length and surrogate position deterministically.
    for body in [
        "",
        "u",
        "u0",
        "u00",
        "u000",
        "u0000",
        "uD800",
        "uDC00",
        "uD83D",
        "uDE00",
        "uD83D\\uDE00",
        "uD800\\uD800",
        "uD800\\uDC00",
        "u12G",
        "uG123",
        "u123G",
    ] {
        for quote in ['"', '\''] {
            let text = format!("var x = {quote}\\{body}{quote}");
            let (sources, lexed) = lex_text(&text);
            let source = source_of(&sources, &lexed);
            assert!(lexed.is_lossless(source), "{text:?}");
            assert_eq!(lexed.reprint(source).as_deref(), Some(text.as_str()));
        }
    }
}
