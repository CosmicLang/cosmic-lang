use core::fmt;

/// A stable diagnostic code such as `CSM001`.
///
/// # Format
///
/// Two to eight uppercase ASCII letters (the component or family) followed by
/// exactly three ASCII digits, e.g. `SRC004`, `TYPE012`, `WSM001`.
///
/// # Stability policy
///
/// Codes are part of the observable interface: users, documentation, tests
/// and machine consumers (LSP, CI scripts) key on them. Once a code has been
/// used in a released version, its meaning must never be redefined or moved
/// to another condition — retire the code instead. New codes are appended
/// inside the owning component's namespace.
///
/// # Construction
///
/// * [`DiagnosticCode::new`] is a `const fn` for codes embedded in the
///   compiler; it validates at compile time when used in const context and
///   panics on a malformed literal (a programming error, caught by tests).
/// * [`DiagnosticCode::try_new`] is the fallible form for any non-const
///   situation and never panics.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiagnosticCode(&'static str);

impl DiagnosticCode {
    /// Creates a code from a `'static` string.
    ///
    /// The format is validated; when this function is used to initialize a
    /// `const`/`static`, an invalid format is a compile-time error.
    ///
    /// # Panics
    ///
    /// Panics if `code` does not match `[A-Z]{2,8}[0-9]{3}`. Compiler code
    /// should only ever pass string literals, so this cannot be triggered by
    /// user input; use [`DiagnosticCode::try_new`] where that cannot be
    /// guaranteed.
    #[must_use]
    pub const fn new(code: &'static str) -> Self {
        if !is_valid_code(code) {
            panic!("invalid diagnostic code: expected 2-8 uppercase letters followed by 3 digits");
        }
        Self(code)
    }

    /// Fallible constructor for codes that are not compile-time constants.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCode`] when `code` does not match `[A-Z]{2,8}[0-9]{3}`.
    pub fn try_new(code: &'static str) -> Result<Self, InvalidCode> {
        if is_valid_code(code) {
            Ok(Self(code))
        } else {
            Err(InvalidCode { code })
        }
    }

    /// The code as a string, e.g. `"CSM001"`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl fmt::Debug for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DiagnosticCode({})", self.0)
    }
}

/// Error returned by [`DiagnosticCode::try_new`] for a malformed code.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InvalidCode {
    /// The rejected code.
    pub code: &'static str,
}

impl fmt::Display for InvalidCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid diagnostic code {:?}: expected 2-8 uppercase letters followed by 3 digits",
            self.code
        )
    }
}

impl std::error::Error for InvalidCode {}

/// `[A-Z]{2,8}[0-9]{3}`, without regex machinery.
const fn is_valid_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    while i < len {
        let byte = bytes[i];
        if byte >= b'A' && byte <= b'Z' {
            i += 1;
        } else {
            break;
        }
    }

    let letters = i;
    if letters < 2 || letters > 8 {
        return false;
    }
    if len - i != 3 {
        return false;
    }
    while i < len {
        let byte = bytes[i];
        if byte < b'0' || byte > b'9' {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticCode, InvalidCode, is_valid_code};

    #[test]
    fn accepts_well_formed_codes() {
        for code in ["AB123", "SRC004", "TYPE012", "PORTUKOF777"] {
            assert!(
                DiagnosticCode::try_new(code).is_ok(),
                "{code} must be valid"
            );
        }
    }

    #[test]
    fn rejects_malformed_codes() {
        for code in [
            "",
            "A123",
            "AC01234",
            "ABCDEFGHI123",
            "SRC12",
            "SRC0001",
            "src123",
            "SRC12a",
            "123ABC",
            "SRC 123",
            "SRC1234",
            "SRC-123",
        ] {
            let error = DiagnosticCode::try_new(code).unwrap_err();
            assert_eq!(error, InvalidCode { code });
        }
    }

    #[test]
    fn const_constructor_matches_try_constructor() {
        const CODE: DiagnosticCode = DiagnosticCode::new("CSM001");
        assert_eq!(CODE.as_str(), "CSM001");
        assert_eq!(Some(CODE), DiagnosticCode::try_new("CSM001").ok());
    }

    #[test]
    fn public_constants_are_formatted_correctly() {
        assert!(is_valid_code("CSM001"));
        assert!(!is_valid_code("csm001"));
    }
}
