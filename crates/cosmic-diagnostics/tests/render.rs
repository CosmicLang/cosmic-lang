//! Integration tests for the `cosmic-diagnostics` public API:
//! exact human rendering, exact JSON rendering, and total behavior for
//! spans that cannot be resolved against the rendering source map.

use cosmic_diagnostics::{
    Applicability, Diagnostic, DiagnosticCode, Label, Severity, Suggestion, render,
};
use cosmic_source::SourceMap;

fn demo() -> (SourceMap, cosmic_source::FileId) {
    let mut sources = SourceMap::new();
    let file = sources
        .add("demo.csmx", "let x =\nlet y = 2\n")
        .expect("small text");
    (sources, file)
}

#[test]
fn human_rendering_is_exact_for_a_typical_error() {
    let (sources, file) = demo();
    let source = sources.get(file).unwrap();
    let at_end_of_line = source.span(7, 7).unwrap(); // after "let x ="

    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM001"),
        "expected expression",
        Label::new(at_end_of_line, "expected an expression here"),
    )
    .with_note("the right-hand side of `=` must be an expression")
    .with_suggestion(
        Suggestion::new(at_end_of_line, "0", "write `0`")
            .with_applicability(Applicability::MachineApplicable),
    );

    let expected = "\
error[CSM001]: expected expression
 --> demo.csmx:1:8
  |
1 | let x =
  |        ^ expected an expression here
  |
   = note: the right-hand side of `=` must be an expression
   = help: write `0`
";
    assert_eq!(render::human(&diagnostic, &sources), expected);
}

#[test]
fn json_rendering_is_exact_for_a_typical_error() {
    let (sources, file) = demo();
    let source = sources.get(file).unwrap();
    let at_end_of_line = source.span(7, 7).unwrap();

    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM001"),
        "expected expression",
        Label::new(at_end_of_line, "expected an expression here"),
    )
    .with_note("the right-hand side of `=` must be an expression")
    .with_suggestion(
        Suggestion::new(at_end_of_line, "0", "write `0`")
            .with_applicability(Applicability::MachineApplicable),
    );

    let expected = concat!(
        r#"{"severity":"error","code":"CSM001","message":"expected expression","#,
        r#""labels":[{"primary":true,"file":"demo.csmx","file_id":1,"start":7,"end":7,"#,
        r#""line":1,"column":8,"message":"expected an expression here"}],"#,
        r#""notes":["the right-hand side of `=` must be an expression"],"#,
        r#""suggestions":[{"file":"demo.csmx","file_id":1,"start":7,"end":7,"#,
        r#""replacement":"0","message":"write `0`","applicability":"machine-applicable"}]}"#,
    );
    assert_eq!(render::json(&diagnostic, &sources), expected);
}

#[test]
fn multiple_labels_render_primary_first_then_secondaries() {
    let mut sources = SourceMap::new();
    let file = sources.add("demo.csmx", "alpha\nbeta\ngamma\n").unwrap();
    let source = sources.get(file).unwrap();

    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM010"),
        "something broke",
        Label::new(source.span(0, 5).unwrap(), "first problem"),
    )
    .with_secondary(Label::new(source.span(11, 16).unwrap(), "related to this"));

    let expected = "\
error[CSM010]: something broke
 --> demo.csmx:1:1
  |
1 | alpha
  | ^^^^^ first problem
  |
 --> demo.csmx:3:1
  |
3 | gamma
  | ----- related to this
  |
";
    assert_eq!(render::human(&diagnostic, &sources), expected);
}

#[test]
fn gutter_width_adapts_to_the_largest_line_number() {
    let mut text = String::new();
    for i in 1..=100 {
        text.push_str(&format!("line {i}\n"));
    }
    let mut sources = SourceMap::new();
    let file = sources.add("big.csmx", text).unwrap();
    let source = sources.get(file).unwrap();

    // Lines 1-9 are 7 bytes ("line N\n" = 63 total), lines 10-99 are 8 bytes
    // (720 total), so "line 100" starts at byte 783 and is 8 bytes long.
    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM011"),
        "hit the last line",
        Label::new(source.span(783, 791).unwrap(), "here"),
    );

    let rendered = render::human(&diagnostic, &sources);
    assert!(
        rendered.contains("   --> big.csmx:100:1"),
        "got:\n{rendered}"
    );
    assert!(rendered.contains("100 | line 100"), "got:\n{rendered}");
    assert!(rendered.contains("    | ^^^^^^^^ here"), "got:\n{rendered}");
}

#[test]
fn tabs_are_normalised_in_snippets_so_markers_stay_aligned() {
    let mut sources = SourceMap::new();
    let file = sources.add("tabs.csmx", "a\tbc\n").unwrap();
    let source = sources.get(file).unwrap();

    // "bc" sits at byte offsets 2..4; visually at column 3 (tab counts once).
    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM012"),
        "tab test",
        Label::new(source.span(2, 4).unwrap(), "these two"),
    );

    let expected = "\
error[CSM012]: tab test
 --> tabs.csmx:1:3
  |
1 | a bc
  |   ^^ these two
  |
";
    assert_eq!(render::human(&diagnostic, &sources), expected);
}

#[test]
fn multi_line_spans_underline_the_first_line_to_its_end() {
    let mut sources = SourceMap::new();
    let file = sources.add("ml.csmx", "alpha\nbeta\n").unwrap();
    let source = sources.get(file).unwrap();

    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM013"),
        "spans two lines",
        Label::new(source.span(0, 8).unwrap(), "starts here"),
    );

    let rendered = render::human(&diagnostic, &sources);
    assert!(rendered.contains("--> ml.csmx:1:1"), "got:\n{rendered}");
    assert!(rendered.contains("1 | alpha"), "got:\n{rendered}");
    assert!(
        rendered.contains("  | ^^^^^ starts here"),
        "got:\n{rendered}"
    );
}

#[test]
fn empty_label_messages_render_bare_markers() {
    let (sources, file) = demo();
    let source = sources.get(file).unwrap();

    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM014"),
        "no message",
        Label::new(source.span(0, 3).unwrap(), ""),
    );

    let expected = "\
error[CSM014]: no message
 --> demo.csmx:1:1
  |
1 | let x =
  | ^^^
  |
";
    assert_eq!(render::human(&diagnostic, &sources), expected);
}

#[test]
fn unknown_files_fall_back_to_byte_ranges_without_panicking() {
    // Build the span in one map, render with a different, empty map.
    let (built_in, file) = demo();
    let span = built_in.get(file).unwrap().span(0, 3).unwrap();
    let render_with = SourceMap::new();

    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM015"),
        "foreign span",
        Label::new(span, "cannot show this file"),
    );

    let rendered = render::human(&diagnostic, &render_with);
    assert!(
        rendered.contains("--> <unknown source file #1>:bytes 0..3 cannot show this file"),
        "got:\n{rendered}"
    );

    let json = render::json(&diagnostic, &render_with);
    assert!(
        json.contains(r#""file":null,"file_id":1,"start":0,"end":3,"line":null,"column":null"#),
        "got:\n{json}"
    );
}

#[test]
fn warning_notes_and_help_have_their_own_severities() {
    let (sources, file) = demo();
    let source = sources.get(file).unwrap();
    let span = source.span(0, 1).unwrap();

    let warning = Diagnostic::warning(
        DiagnosticCode::new("CSM020"),
        "unused binding",
        Label::new(span, "never read"),
    );
    assert_eq!(warning.severity(), Severity::Warning);
    assert!(render::human(&warning, &sources).starts_with("warning[CSM020]: unused binding\n"));

    let note = Diagnostic::note(
        DiagnosticCode::new("CSM021"),
        "context",
        Label::new(span, ""),
    );
    assert_eq!(note.severity(), Severity::Note);

    let help = Diagnostic::help(
        DiagnosticCode::new("CSM022"),
        "consider X",
        Label::new(span, ""),
    );
    assert_eq!(help.severity(), Severity::Help);
}

#[test]
fn json_escapes_user_visible_text() {
    let mut sources = SourceMap::new();
    let file = sources.add("esc.csmx", "x\n").unwrap();
    let source = sources.get(file).unwrap();

    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM030"),
        "quote \" backslash \\ newline \n tab \t",
        Label::new(source.span(0, 1).unwrap(), "control \u{1} char"),
    );

    let json = render::json(&diagnostic, &sources);
    assert!(
        json.contains(r#""message":"quote \" backslash \\ newline \n tab \t""#),
        "got:\n{json}"
    );
    assert!(
        json.contains(r#""message":"control \u0001 char""#),
        "got:\n{json}"
    );
}

#[test]
fn json_contains_all_fields_even_when_absent() {
    let (sources, file) = demo();
    let source = sources.get(file).unwrap();
    let diagnostic = Diagnostic::error(
        DiagnosticCode::new("CSM031"),
        "minimal",
        Label::new(source.span(0, 0).unwrap(), ""),
    );

    let json = render::json(&diagnostic, &sources);
    assert!(json.contains(r#""notes":[]"#), "got:\n{json}");
    assert!(json.contains(r#""suggestions":[]"#), "got:\n{json}");
    assert!(
        json.contains(r#""line":1,"column":1,"message":"""#),
        "got:\n{json}"
    );
}
