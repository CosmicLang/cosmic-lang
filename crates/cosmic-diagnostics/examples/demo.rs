//! Demo: renders one structured diagnostic in both supported formats.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p cosmic-diagnostics --example demo
//! ```

use cosmic_diagnostics::{Applicability, Diagnostic, DiagnosticCode, Label, Suggestion, render};
use cosmic_source::SourceMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sources = SourceMap::new();
    let file = sources.add("examples/hello.csmx", "main() {\n    println(\"hi\"\n}\n")?;
    let source = sources.get(file).ok_or("just registered")?;

    // The `"hi"` string opens at byte 21 (the `"` character).
    let opening_quote = source.span(21, 22)?;
    // The `\n` after `"hi"` is byte 25.
    let line_end = source.span(25, 25)?;

    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("LEX001"),
        "unterminated string literal",
        Label::new(opening_quote, "string starts here and is never closed"),
    )
    .with_note("string literals must be closed before the end of the line")
    .with_suggestion(
        Suggestion::new(line_end, "\"", "close the string literal here")
            .with_applicability(Applicability::MaybeIncorrect),
    );

    println!("{}", render::human(&diagnostic, &sources));
    println!("{}", render::json(&diagnostic, &sources));
    Ok(())
}
