//! The `PARSE` namespace registry evidence (`docs/spec/diagnostic-codes.md`
//! §6.3). The five test names below are frozen with the registry: one per
//! minted code, plus the namespace-stability test.

mod common;

use common::{all_codes, parse_codes, parse_text};
use cosmic_diagnostics::Severity;
use cosmic_syntax::ParseErrorCode;

/// `PARSE001` — unsupported syntax family refused: one negative fixture per
/// refused family, each exactly one refusal diagnostic.
#[test]
fn parse001_unsupported_families_are_refused() {
    let text = "\
test \"suite\" { check() }\n\
application { run() }\n\
infra \"prod\" { deploy() }\n\
extern Int nativeAdd(Int a, Int b)\n\
foreign module \"std\" { call() }\n\
entity User extends Base { }\n\
@Marker\n\
main() { val xs = {1, 2, 3} }\n\
";
    let (_, parsed) = parse_text(text);
    assert_eq!(parse_codes(&parsed), vec!["PARSE001"; 8]);
    for diagnostic in parsed.diagnostics().iter() {
        let message = diagnostic.message();
        assert!(message.contains("unsupported syntax family"), "{message}");
        assert!(
            message.contains("(S9") || message.contains("(S10"),
            "{message}"
        );
        assert_eq!(diagnostic.severity(), Severity::Error);
    }
}

/// `PARSE002` — unexpected token.
#[test]
fn parse002_unexpected_tokens_are_reported() {
    // A malformed region at statement start.
    let (_, parsed) = parse_text("main() { ? }\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE002"]);

    // A missing initialiser: one diagnostic, the offending token preserved.
    let (_, parsed) = parse_text("main() { val a = ? }\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE002"]);

    // The reserved-invalid words share the code class with the SG-001
    // message.
    for keyword in ["fun", "fn", "func"] {
        let (_, parsed) = parse_text(&format!("main() {{ {keyword} }}"));
        assert_eq!(parse_codes(&parsed), vec!["PARSE002"], "{keyword}");
        let message = parsed
            .diagnostics()
            .iter()
            .next()
            .expect("the refusal diagnostic")
            .message()
            .to_string();
        assert!(message.contains("SG-001"), "{keyword}: {message}");
    }

    for diagnostic in parsed.diagnostics().iter() {
        assert_eq!(diagnostic.severity(), Severity::Error);
    }
}

/// `PARSE003` — unexpected end of input (unclosed construct).
#[test]
fn parse003_unexpected_end_of_input_is_reported() {
    let cases = [
        "main() {\n",
        "main() {\n  call(2\n",
        "main() {\n  val x = (2\n",
        "main() {\n  val f = (x: Int) -> \n",
        "main() {\n  switch (x) {\n    case 1: call()\n",
    ];
    for text in cases {
        let (_, parsed) = parse_text(text);
        assert_eq!(parse_codes(&parsed), vec!["PARSE003"], "{text:?}");
        let diagnostic = parsed.diagnostics().iter().next().expect("one diagnostic");
        assert_eq!(diagnostic.severity(), Severity::Error);
        assert_eq!(diagnostic.primary().span().start(), text.len() as u32);
    }
}

/// `PARSE004` — labelled `break`/`continue` is not supported.
#[test]
fn parse004_labelled_jumps_are_refused() {
    let (_, parsed) =
        parse_text("main() {\n  while (ready) {\n    break outer\n  }\n  continue outer\n}\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE004", "PARSE004"]);
    for diagnostic in parsed.diagnostics().iter() {
        assert_eq!(diagnostic.severity(), Severity::Error);
        assert!(
            diagnostic.message().contains("S5"),
            "{}",
            diagnostic.message()
        );
    }
}

/// The `PARSE` namespace is contiguous, frozen for review, and nothing else
/// is emitted.
#[test]
fn parse_codes_are_stable() {
    let expected = ["PARSE001", "PARSE002", "PARSE003", "PARSE004"];
    for (index, code) in ParseErrorCode::ALL.into_iter().enumerate() {
        assert_eq!(code.as_str(), expected[index]);
        assert!(!code.condition().is_empty());
    }

    let fixtures = [
        "test \"t\" {\n}\n",
        "main() { ? }\n",
        "main() {\n",
        "main() { break outer }\n",
    ];
    for text in fixtures {
        let (_, parsed) = parse_text(text);
        let codes = all_codes(&parsed);
        assert!(
            codes.iter().all(|code| expected.contains(code)),
            "{text:?}: {codes:?}"
        );
        assert!(!codes.is_empty(), "{text:?}");
    }
}
