//! The independently authored comparison corpus and its manifest model.
//!
//! Every fixture in [`Corpus::embedded`] was authored for this repository to
//! exercise an accepted decision or a known divergence. Nothing is copied or
//! mechanically translated from the legacy implementation. An external corpus
//! with the same manifest shape can be loaded with [`Corpus::from_dir`].

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use crate::HarnessError;

/// The comparison category of a fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FixtureCategory {
    /// Lexical-contract inputs (identifier classes, escapes, literals, …).
    Lexical,
    /// Recognised-syntax inputs.
    Syntax,
    /// Inputs in an unsupported family that must be refused.
    Refusals,
    /// Malformed inputs that must recover.
    Malformed,
}

impl FixtureCategory {
    /// Every category, in a stable order.
    pub const ALL: [FixtureCategory; 4] = [
        FixtureCategory::Lexical,
        FixtureCategory::Syntax,
        FixtureCategory::Refusals,
        FixtureCategory::Malformed,
    ];

    /// The manifest spelling of this category.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            FixtureCategory::Lexical => "lexical",
            FixtureCategory::Syntax => "syntax",
            FixtureCategory::Refusals => "refusals",
            FixtureCategory::Malformed => "malformed",
        }
    }

    /// Parses a manifest spelling.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.name() == text)
    }
}

/// What the Rust frontend must do with a fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expectation {
    /// No diagnostic at all.
    Recognised,
    /// At least one refusal diagnostic (`PARSE001`/`PARSE004`).
    Refused,
    /// At least one diagnostic (lexical or syntactic) and recovery.
    Error,
}

impl Expectation {
    /// The manifest spelling of this expectation.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Expectation::Recognised => "recognised",
            Expectation::Refused => "refused",
            Expectation::Error => "error",
        }
    }

    /// Parses a manifest spelling.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        [
            Expectation::Recognised,
            Expectation::Refused,
            Expectation::Error,
        ]
        .into_iter()
        .find(|expectation| expectation.name() == text)
    }
}

/// One independently authored source input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fixture {
    /// Stable identifier, equal to the file stem in an external corpus.
    pub id: String,
    /// Comparison category.
    pub category: FixtureCategory,
    /// What the Rust frontend must do with the input.
    pub expectation: Expectation,
    /// One-line description used in reports.
    pub description: String,
    /// The exact source text.
    pub text: String,
}

/// A corpus of authored fixtures.
#[derive(Clone, Debug)]
pub struct Corpus {
    /// The fixtures, sorted by id.
    pub fixtures: Vec<Fixture>,
}

impl Corpus {
    /// Builds the embedded corpus shipped with the harness.
    #[must_use]
    pub fn embedded() -> Corpus {
        let mut fixtures: Vec<Fixture> = EMBEDDED
            .iter()
            .map(|(id, category, expectation, description, text)| Fixture {
                id: (*id).to_owned(),
                category: FixtureCategory::parse(category).expect("embedded category spelling"),
                expectation: Expectation::parse(expectation)
                    .expect("embedded expectation spelling"),
                description: (*description).to_owned(),
                text: (*text).to_owned(),
            })
            .collect();
        fixtures.push(Fixture {
            id: "mal_nesting_limit".to_owned(),
            category: FixtureCategory::Malformed,
            expectation: Expectation::Error,
            description: "300 nested parentheses cross the implementation nesting guard (PARSE005); the region stays lossless".to_owned(),
            text: format!(
                "main() {{\n    val x = {}1{}\n}}\n",
                "(".repeat(300),
                ")".repeat(300)
            ),
        });
        fixtures.sort_by(|left, right| left.id.cmp(&right.id));
        Corpus { fixtures }
    }

    /// Loads a corpus from an external directory.
    ///
    /// The layout is `<root>/manifest.tsv` plus `<root>/<category>/<id>.csmx`.
    /// A manifest row is `id<TAB>category<TAB>expectation<TAB>description`;
    /// blank lines and `#` comments are ignored.
    pub fn from_dir(root: &Path) -> Result<Corpus, HarnessError> {
        let manifest_path = root.join("manifest.tsv");
        let manifest = std::fs::read_to_string(&manifest_path).map_err(|error| {
            HarnessError::Corpus(format!("{}: {error}", manifest_path.display()))
        })?;
        let mut fixtures = Vec::new();
        for (index, line) in manifest.lines().enumerate() {
            let line = line.trim_end_matches('\r');
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != 4 {
                return Err(HarnessError::Corpus(format!(
                    "manifest.tsv row {}: expected 4 tab-separated fields, found {}",
                    index + 1,
                    fields.len()
                )));
            }
            let (id, category, expectation, description) =
                (fields[0], fields[1], fields[2], fields[3]);
            let category = FixtureCategory::parse(category).ok_or_else(|| {
                HarnessError::Corpus(format!("row {}: unknown category {category:?}", index + 1))
            })?;
            let expectation = Expectation::parse(expectation).ok_or_else(|| {
                HarnessError::Corpus(format!(
                    "row {}: unknown expectation {expectation:?}",
                    index + 1
                ))
            })?;
            let path = root.join(category.name()).join(format!("{id}.csmx"));
            let text = std::fs::read_to_string(&path)
                .map_err(|error| HarnessError::Corpus(format!("{}: {error}", path.display())))?;
            fixtures.push(Fixture {
                id: id.to_owned(),
                category,
                expectation,
                description: description.to_owned(),
                text,
            });
        }
        let corpus = Corpus { fixtures };
        corpus.validate()?;
        Ok(corpus)
    }

    /// Rejects duplicate fixture ids.
    pub fn validate(&self) -> Result<(), HarnessError> {
        let mut seen = BTreeSet::new();
        for fixture in &self.fixtures {
            if !seen.insert(fixture.id.as_str()) {
                return Err(HarnessError::Corpus(format!(
                    "duplicate fixture id {:?}",
                    fixture.id
                )));
            }
        }
        Ok(())
    }

    /// The fixtures whose category is selected, sorted by id.
    #[must_use]
    pub fn select(&self, categories: &BTreeSet<FixtureCategory>) -> Vec<&Fixture> {
        let mut selected: Vec<&Fixture> = self
            .fixtures
            .iter()
            .filter(|fixture| categories.contains(&fixture.category))
            .collect();
        selected.sort_by(|left, right| left.id.cmp(&right.id));
        selected
    }

    /// A stable digest of the corpus identity and contents (FNV-1a 64).
    #[must_use]
    pub fn digest(&self) -> String {
        let mut entries: Vec<(&str, &str)> = self
            .fixtures
            .iter()
            .map(|fixture| (fixture.id.as_str(), fixture.text.as_str()))
            .collect();
        entries.sort_unstable();
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for (id, text) in entries {
            for byte in id.bytes().chain(std::iter::once(0)).chain(text.bytes()) {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        let mut digest = String::new();
        let _ = write!(digest, "fnv1a64:{hash:016x}");
        digest
    }
}

/// The embedded fixture table: `(id, category, expectation, description, text)`.
const EMBEDDED: &[(&str, &str, &str, &str, &str)] = &[
    // --- lexical ------------------------------------------------------------
    (
        "lex_identifier_ascii",
        "lexical",
        "recognised",
        "ASCII letters, digits in continuation position and `_` form one identifier",
        "main() {\n    val camelCase_1 = 1\n}\n",
    ),
    (
        "lex_identifier_underscore",
        "lexical",
        "recognised",
        "`_` alone is an identifier (D-L3)",
        "main() {\n    val _ = 1\n}\n",
    ),
    (
        "lex_identifier_dollar",
        "lexical",
        "recognised",
        "`$` is an identifier character (D-L3)",
        "main() {\n    val $x = 1\n}\n",
    ),
    (
        "lex_identifier_unicode_letter",
        "lexical",
        "recognised",
        "precomposed non-ASCII letters are identifier characters (M-7 boundary)",
        "main() {\n    val café = 1\n}\n",
    ),
    (
        "lex_identifier_non_bmp_letter",
        "lexical",
        "recognised",
        "the non-BMP letter U+10400 starts an identifier (M-7 boundary)",
        "main() {\n    val \u{10400}name = 1\n}\n",
    ),
    (
        "lex_identifier_combining_continue",
        "lexical",
        "error",
        "a combining mark does not continue an identifier (M-7 measured boundary)",
        "main() {\n    val e\u{0301} = 1\n}\n",
    ),
    (
        "lex_identifier_leading_combining",
        "lexical",
        "error",
        "a combining mark does not start an identifier (M-7 measured boundary)",
        "main() {\n    val \u{0301}name = 1\n}\n",
    ),
    (
        "lex_identifier_leading_digit",
        "lexical",
        "error",
        "digit-leading name: longest-valid-prefix number, then an identifier",
        "main() {\n    val 1abc = 1\n}\n",
    ),
    (
        "lex_identifier_emoji",
        "lexical",
        "error",
        "an emoji is not an identifier character",
        "main() {\n    val 😀 = 1\n}\n",
    ),
    (
        "lex_unexpected_character",
        "lexical",
        "error",
        "an unexpected character (`~`) becomes one error region (LEX005)",
        "main() {\n    val a = b ~ c\n}\n",
    ),
    (
        "lex_strict_escape",
        "lexical",
        "error",
        "unknown escapes are refused (LEX008, declared divergence D-L8)",
        "main() {\n    val s = \"a\\qb\"\n}\n",
    ),
    (
        "lex_unicode_escape_valid",
        "lexical",
        "recognised",
        "a valid four-hex-digit `\\uXXXX` escape is accepted (D-L5)",
        "main() {\n    val s = \"\\u0041\"\n}\n",
    ),
    (
        "lex_lone_surrogate_string",
        "lexical",
        "error",
        "a lone high surrogate in a string is refused (LEX010, D-L9)",
        "main() {\n    val s = \"\\uD800\"\n}\n",
    ),
    (
        "lex_surrogate_pair_string",
        "lexical",
        "recognised",
        "a valid surrogate pair in a string is one clean literal (D-L9)",
        "main() {\n    val s = \"\\uD83D\\uDE00\"\n}\n",
    ),
    (
        "lex_lone_surrogate_char",
        "lexical",
        "error",
        "a lone surrogate escape in a character literal is refused (LEX010, D-L9)",
        "main() {\n    val c = '\\uD800'\n}\n",
    ),
    (
        "lex_triple_quoted",
        "lexical",
        "error",
        "triple-quoted strings are refused as one region (LEX009, D-L6)",
        "main() {\n    val s = \"\"\"x\"\"\"\n}\n",
    ),
    (
        "lex_char_multi",
        "lexical",
        "error",
        "a multi-character literal is refused (LEX004)",
        "main() {\n    val c = 'ab'\n}\n",
    ),
    (
        "lex_char_unterminated",
        "lexical",
        "error",
        "an unterminated character literal is refused (LEX004)",
        "main() {\n    val c = 'a\n}\n",
    ),
    (
        "lex_number_longest_prefix",
        "lexical",
        "recognised",
        "`0x1F3G` lexes the longest valid prefix `0x1F3` plus identifier `G`; the parser accepts the declaration followed by an expression statement with no diagnostic",
        "main() {\n    val x = 0x1F3G\n}\n",
    ),
    (
        "lex_bom_prefix",
        "lexical",
        "recognised",
        "the BOM at byte 0 is a preserved prefix leaf and changes no token (D-L1)",
        "\u{FEFF}main() {\n    x()\n}\n",
    ),
    (
        "lex_line_comment_crlf",
        "lexical",
        "recognised",
        "CRLF terminates a line comment and is excluded from it (D-L4)",
        "main() {\r\n    // note\r\n    x()\r\n}\r\n",
    ),
    (
        "lex_line_comment_cr",
        "lexical",
        "recognised",
        "bare CR terminates a line comment (D-L4)",
        "main() {\r    x() // note\ry()\r}\r",
    ),
    (
        "lex_raw_crlf_in_string",
        "lexical",
        "recognised",
        "raw CRLF inside a string literal is content (D-L7); the decoded-value question stays open",
        "main() {\n    val s = \"a\r\nb\"\n}\n",
    ),
    (
        "lex_nbsp_is_not_whitespace",
        "lexical",
        "error",
        "NBSP is not whitespace and becomes an error region (LEX005)",
        "main() {\n    val x = a\u{00A0}+ b\n}\n",
    ),
    // --- syntax -------------------------------------------------------------
    (
        "syn_paren_lift_across_comment",
        "syntax",
        "recognised",
        "the parenthesised continuation lift still holds across a multiline comment (F2)",
        "main() {\n    call(a /*\n*/ + b)\n}\n",
    ),
    (
        "syn_trailing_lambda_inline",
        "syntax",
        "recognised",
        "a trailing lambda attaches across an inline comment (same physical line)",
        "main() {\n    items.each /* c */ { x: Int -> call(x) }\n}\n",
    ),
    (
        "syn_trailing_lambda_multiline",
        "syntax",
        "error",
        "a trailing lambda does not attach across a physical line inside a comment (A2-R1-01)",
        "main() {\n    items.each /*\n*/ { x: Int -> call(x) }\n}\n",
    ),
    (
        "syn_generic_call",
        "syntax",
        "recognised",
        "generic-call versus comparison disambiguation accepts `listOf<Int>(1)` (§5.2)",
        "main() {\n    val xs = listOf<Int>(1)\n}\n",
    ),
    (
        "syn_shift_expression",
        "syntax",
        "recognised",
        "the pre-split `>>` token re-joins as one shift operator",
        "main() {\n    val y = a >> b\n}\n",
    ),
    (
        "syn_switch_statement",
        "syntax",
        "recognised",
        "switch statement with expression labels and a default arm",
        "main() {\n    switch (x) {\n        case 1: call()\n        default: call()\n    }\n}\n",
    ),
    (
        "syn_nested_function_inline",
        "syntax",
        "recognised",
        "a same-line nested function body attaches (F5 correction)",
        "main() {\n    inner() { call() }\n}\n",
    ),
    (
        "syn_nested_function_multiline",
        "syntax",
        "recognised",
        "a same-line nested body does not attach across a physical break inside a comment; the braces become a separate block statement with no diagnostic (A2-R1-01)",
        "main() {\n    inner() /*\n*/ { call() }\n}\n",
    ),
    (
        "syn_else_crosses_comment",
        "syntax",
        "recognised",
        "the committed keyword continuation `else` is not line-gated",
        "main() {\n    if (c) x /*\n*/ else y\n}\n",
    ),
    (
        "syn_boolean_null_literals",
        "syntax",
        "recognised",
        "boolean and null literals are accepted in expression position",
        "main() {\n    val a = true\n    val b = false\n    val c = null\n}\n",
    ),
    (
        "syn_untyped_lambda",
        "syntax",
        "error",
        "untyped lambda parameters are refused (documented boundary; S1 open)",
        "main() {\n    val f = (x) -> x\n}\n",
    ),
    (
        "syn_reserved_invalid_function",
        "syntax",
        "error",
        "`fun` produces the specific SG-001 PARSE002 message",
        "main() {\n    fun helper() {}\n}\n",
    ),
    // --- refusals -----------------------------------------------------------
    (
        "ref_test_family",
        "refusals",
        "refused",
        "family 1 (`test`) is refused with one PARSE001",
        "test \"suite\" { check() }\n",
    ),
    (
        "ref_application_family",
        "refusals",
        "refused",
        "family 2 (`application`) is refused with one PARSE001",
        "application { run() }\n",
    ),
    (
        "ref_infra_family",
        "refusals",
        "refused",
        "family 3 (`infra`) is refused with one PARSE001",
        "infra \"prod\" { deploy() }\n",
    ),
    (
        "ref_extern_family",
        "refusals",
        "refused",
        "family 4 (`extern`) is refused with one PARSE001",
        "extern Int nativeAdd(Int a, Int b)\n",
    ),
    (
        "ref_foreign_module_family",
        "refusals",
        "refused",
        "family 5 (`foreign module`) is refused with one PARSE001",
        "foreign module \"std\" { call() }\n",
    ),
    (
        "ref_entity_family",
        "refusals",
        "refused",
        "family 6 (`entity`) is refused with one PARSE001",
        "entity User extends Base { }\n",
    ),
    (
        "ref_annotation_family",
        "refusals",
        "refused",
        "family 7 (annotations) is refused with one PARSE001",
        "@Marker\nmain() {\n}\n",
    ),
    (
        "ref_array_literal_family",
        "refusals",
        "refused",
        "family 9 (brace array literal) is refused with one PARSE001",
        "main() {\n    val xs = {1, 2}\n}\n",
    ),
    (
        "ref_labelled_jump",
        "refusals",
        "refused",
        "family 8 (labelled `break`) is refused with one PARSE004",
        "main() {\n    break outer\n}\n",
    ),
    // --- malformed ----------------------------------------------------------
    (
        "mal_unclosed_brace",
        "malformed",
        "error",
        "an unclosed block reports PARSE003 once and keeps the region",
        "main() {\n",
    ),
    (
        "mal_excess_closer",
        "malformed",
        "error",
        "a stray closer is reported and preserved",
        "main() {}\n}\n",
    ),
    (
        "mal_multiple_errors",
        "malformed",
        "error",
        "three independent unexpected `~` tokens produce bounded diagnostics",
        "main() {\n    ~\n    ~\n    ~\n}\n",
    ),
];
