//! Runs the Rust frontend on one input and checks the corpus expectation.

use cosmic_source::SourceMap;

use crate::{Expectation, HarnessError};

/// The Rust frontend's observable result for one input.
#[derive(Clone, Debug)]
pub struct FrontendOutcome {
    /// Lexical diagnostic codes, in emission order.
    pub lex_codes: Vec<String>,
    /// Syntactic diagnostic codes, in emission order.
    pub parse_codes: Vec<String>,
    /// Number of CST nodes (including leaves).
    pub nodes: usize,
    /// Whether the CST losslessly mirrors every lexer element.
    pub lossless: bool,
    /// Whether reprinting the CST reproduces the source byte for byte.
    pub reprint_ok: bool,
}

impl FrontendOutcome {
    /// No diagnostic of any namespace was emitted.
    #[must_use]
    pub fn accepted(&self) -> bool {
        self.lex_codes.is_empty() && self.parse_codes.is_empty()
    }

    /// Whether the outcome satisfies the corpus expectation.
    #[must_use]
    pub fn matches(&self, expectation: Expectation) -> bool {
        match expectation {
            Expectation::Recognised => self.accepted(),
            Expectation::Refused => self
                .parse_codes
                .iter()
                .any(|code| code == "PARSE001" || code == "PARSE004"),
            Expectation::Error => !self.accepted(),
        }
    }

    /// A short human-readable code list, e.g. `LEX008,PARSE002`.
    #[must_use]
    pub fn codes_label(&self) -> String {
        if self.accepted() {
            return "none".to_owned();
        }
        let mut codes: Vec<&str> = self
            .lex_codes
            .iter()
            .chain(self.parse_codes.iter())
            .map(String::as_str)
            .collect();
        codes.sort_unstable();
        codes.dedup();
        codes.join(",")
    }
}

/// Runs `cosmic-lexer` + `cosmic-syntax` over `text` and reports the result.
pub fn run_frontend(text: &str) -> Result<FrontendOutcome, HarnessError> {
    let mut sources = SourceMap::new();
    let id = sources
        .add("fixture.csmx", text)
        .map_err(|error| HarnessError::Corpus(format!("fixture registration failed: {error}")))?;
    let source = sources.get(id).ok_or_else(|| {
        HarnessError::Corpus("fixture disappeared from the source map".to_owned())
    })?;
    let parsed = cosmic_syntax::parse(source);
    let mut lex_codes = Vec::new();
    let mut parse_codes = Vec::new();
    for diagnostic in parsed.all_diagnostics() {
        let code = diagnostic.code().as_str().to_owned();
        if code.starts_with("PARSE") {
            parse_codes.push(code);
        } else {
            lex_codes.push(code);
        }
    }
    Ok(FrontendOutcome {
        lex_codes,
        parse_codes,
        nodes: parsed.nodes().count(),
        lossless: parsed.is_lossless(source),
        reprint_ok: parsed.reprint(source).as_deref() == Some(text),
    })
}
