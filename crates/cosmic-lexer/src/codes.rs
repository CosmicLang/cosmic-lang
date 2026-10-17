//! The ten stable `LEX` diagnostic codes and their registry conditions.

use cosmic_diagnostics::DiagnosticCode;

/// A stable code from the `LEX` namespace (`docs/spec/diagnostic-codes.md`).
///
/// Every `LEX` code is **error** severity: a lexical error makes the token
/// stream unusable as written. Codes are part of the observable interface —
/// once released, a code's condition must never be redefined; new conditions
/// are appended inside the namespace. Each code is minted into the registry
/// with the unit/integration test named as its evidence.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum LexErrorCode {
    /// `LEX001` — unterminated block comment.
    Lex001,
    /// `LEX002` — unterminated string literal.
    Lex002,
    /// `LEX003` — empty character literal.
    Lex003,
    /// `LEX004` — character literal that does not contain exactly one
    /// character value (unterminated or multi-character).
    Lex004,
    /// `LEX005` — unexpected character: anything the token grammar does not
    /// recognise.
    Lex005,
    /// `LEX006` — incomplete `\u` escape (fewer than four hexadecimal digits).
    Lex006,
    /// `LEX007` — invalid `\u` escape (a non-hexadecimal digit in the
    /// four-digit window).
    Lex007,
    /// `LEX008` — unknown escape sequence (the strict rule; the legacy lexer
    /// silently kept the escaped character).
    Lex008,
    /// `LEX009` — triple-quoted string literal (not part of the language).
    Lex009,
    /// `LEX010` — `\u` escape naming an unpaired UTF-16 surrogate.
    Lex010,
}

impl LexErrorCode {
    /// All ten codes in numeric order.
    pub const ALL: [LexErrorCode; 10] = [
        LexErrorCode::Lex001,
        LexErrorCode::Lex002,
        LexErrorCode::Lex003,
        LexErrorCode::Lex004,
        LexErrorCode::Lex005,
        LexErrorCode::Lex006,
        LexErrorCode::Lex007,
        LexErrorCode::Lex008,
        LexErrorCode::Lex009,
        LexErrorCode::Lex010,
    ];

    /// The stable code string, e.g. `"LEX001"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            LexErrorCode::Lex001 => "LEX001",
            LexErrorCode::Lex002 => "LEX002",
            LexErrorCode::Lex003 => "LEX003",
            LexErrorCode::Lex004 => "LEX004",
            LexErrorCode::Lex005 => "LEX005",
            LexErrorCode::Lex006 => "LEX006",
            LexErrorCode::Lex007 => "LEX007",
            LexErrorCode::Lex008 => "LEX008",
            LexErrorCode::Lex009 => "LEX009",
            LexErrorCode::Lex010 => "LEX010",
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
            LexErrorCode::Lex001 => "unterminated block comment",
            LexErrorCode::Lex002 => "unterminated string literal",
            LexErrorCode::Lex003 => "empty character literal",
            LexErrorCode::Lex004 => {
                "character literal that does not contain exactly one character value (unterminated or multi-character)"
            }
            LexErrorCode::Lex005 => {
                "unexpected character (anything the token grammar does not recognise)"
            }
            LexErrorCode::Lex006 => "incomplete `\\u` escape (fewer than four hexadecimal digits)",
            LexErrorCode::Lex007 => {
                "invalid `\\u` escape (a non-hexadecimal digit in the four-digit window)"
            }
            LexErrorCode::Lex008 => "unknown escape sequence",
            LexErrorCode::Lex009 => "triple-quoted string literal (not part of the language)",
            LexErrorCode::Lex010 => "`\\u` escape naming an unpaired UTF-16 surrogate",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LexErrorCode;

    #[test]
    fn codes_are_stable_strings() {
        let expected = [
            "LEX001", "LEX002", "LEX003", "LEX004", "LEX005", "LEX006", "LEX007", "LEX008",
            "LEX009", "LEX010",
        ];
        for (code, want) in LexErrorCode::ALL.into_iter().zip(expected) {
            assert_eq!(code.as_str(), want);
            assert_eq!(code.code().as_str(), want);
        }
    }

    #[test]
    fn every_condition_is_nonempty_and_distinct() {
        let mut conditions: Vec<&'static str> = LexErrorCode::ALL
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
