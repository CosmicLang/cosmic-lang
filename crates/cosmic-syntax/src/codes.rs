//! The five stable `PARSE` diagnostic codes and their registry conditions.
//!
//! The `PARSE` namespace is owned by `cosmic-syntax`
//! (`docs/spec/diagnostic-codes.md` §3). The allocation was accepted with the
//! G1-S decision package (`docs/spec/g1-s-syntax-decision-package.md` §8) and
//! is minted here with the integration test named as its evidence.
//!
//! Like every code in the registry, a code's condition is frozen once
//! released; new conditions get new codes. All five codes are **error**
//! severity: parsing an input that triggers any of them cannot yield a
//! fully-recognised program.

use cosmic_diagnostics::DiagnosticCode;

/// A stable code from the `PARSE` namespace.
///
/// | Code | Condition |
/// |---|---|
/// | [`ParseErrorCode::Parse001`] | unsupported syntax family refused |
/// | [`ParseErrorCode::Parse002`] | unexpected token |
/// | [`ParseErrorCode::Parse003`] | unexpected end of input (unclosed construct) |
/// | [`ParseErrorCode::Parse004`] | labelled `break`/`continue` is not supported |
/// | [`ParseErrorCode::Parse005`] | nesting exceeds the parser's implementation limit |
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum ParseErrorCode {
    /// `PARSE001` — unsupported syntax family refused (the diagnostic names
    /// the construct, its family and the governing decision record).
    Parse001,
    /// `PARSE002` — unexpected token.
    Parse002,
    /// `PARSE003` — unexpected end of input (an unclosed construct).
    Parse003,
    /// `PARSE004` — labelled `break`/`continue` is not supported (the grammar
    /// derives unlabelled forms only; S5 owns the question).
    Parse004,
    /// `PARSE005` — nesting exceeds the parser's implementation limit (a
    /// stack-safety bound, not a syntax rule; the balanced region is
    /// preserved as an error region and parsing continues).
    Parse005,
}

impl ParseErrorCode {
    /// All five codes in numeric order.
    pub const ALL: [ParseErrorCode; 5] = [
        ParseErrorCode::Parse001,
        ParseErrorCode::Parse002,
        ParseErrorCode::Parse003,
        ParseErrorCode::Parse004,
        ParseErrorCode::Parse005,
    ];

    /// The stable code string, e.g. `"PARSE001"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ParseErrorCode::Parse001 => "PARSE001",
            ParseErrorCode::Parse002 => "PARSE002",
            ParseErrorCode::Parse003 => "PARSE003",
            ParseErrorCode::Parse004 => "PARSE004",
            ParseErrorCode::Parse005 => "PARSE005",
        }
    }

    /// The code as a validated [`DiagnosticCode`], built at compile time.
    #[must_use]
    pub const fn code(self) -> DiagnosticCode {
        DiagnosticCode::new(self.as_str())
    }

    /// The registered condition, verbatim from the code registry.
    #[must_use]
    pub const fn condition(self) -> &'static str {
        match self {
            ParseErrorCode::Parse001 => {
                "unsupported syntax family refused (the refused construct is named with its family and decision record)"
            }
            ParseErrorCode::Parse002 => "unexpected token",
            ParseErrorCode::Parse003 => {
                "unexpected end of input (a construct that must close was not closed)"
            }
            ParseErrorCode::Parse004 => {
                "labelled `break`/`continue` is not supported (the grammar derives unlabelled forms only)"
            }
            ParseErrorCode::Parse005 => {
                "nesting exceeds the parser's implementation limit (an implementation resource limit, not a syntax rule)"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ParseErrorCode;

    #[test]
    fn codes_are_stable_strings() {
        let expected = ["PARSE001", "PARSE002", "PARSE003", "PARSE004", "PARSE005"];
        for (code, want) in ParseErrorCode::ALL.into_iter().zip(expected) {
            assert_eq!(code.as_str(), want);
            assert_eq!(code.code().as_str(), want);
        }
    }

    #[test]
    fn every_condition_is_nonempty_and_distinct() {
        let mut conditions: Vec<&'static str> = ParseErrorCode::ALL
            .iter()
            .map(|code| code.condition())
            .collect();
        conditions.sort_unstable();
        let count = conditions.len();
        conditions.dedup();
        assert_eq!(conditions.len(), count, "conditions must be distinct");
        assert!(conditions.iter().all(|condition| !condition.is_empty()));
    }
}
