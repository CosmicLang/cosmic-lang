use core::fmt::Write as _;

use cosmic_source::SourceMap;

use crate::{Diagnostic, Label};

/// Renders a diagnostic in the human-readable text format.
///
/// Layout (deterministic, one trailing newline at the end):
///
/// ```text
/// error[CSM001]: expected expression
///  --> demo.csmx:3:13
///   |
/// 3 |     let x =
///   |             ^ expected an expression here
///   |
///    = note: the right-hand side of `=` must be an expression
///    = help: write `0`
/// ```
///
/// Layout rules:
///
/// * the gutter width adapts to the largest displayed line number;
/// * the primary label is underlined with `^`, secondary labels with `-`;
/// * zero-width spans underline one character;
/// * spans covering several lines underline the first line up to its end;
/// * tabs in source lines are rendered as single spaces so that the marker
///   line stays aligned with the printed column (columns count `char`s);
/// * labels whose file is missing from `sources` fall back to a
///   `path:bytes` location line without a source snippet.
///
/// The output is identical across runs for identical inputs.
#[must_use]
pub fn render(diagnostic: &Diagnostic, sources: &SourceMap) -> String {
    let views: Vec<LabelView<'_>> = std::iter::once((diagnostic.primary(), true))
        .chain(diagnostic.secondary().iter().map(|label| (label, false)))
        .map(|(label, primary)| LabelView::resolve(label, primary, sources))
        .collect();

    let width = views
        .iter()
        .filter_map(|view| view.location.as_ref())
        .map(|location| digit_count(location.line))
        .max()
        .unwrap_or(1);

    let mut out = String::new();
    let _ = writeln!(
        out,
        "{}[{}]: {}",
        diagnostic.severity(),
        diagnostic.code(),
        diagnostic.message()
    );
    for view in &views {
        view.render_into(&mut out, width, sources);
    }
    for note in diagnostic.notes() {
        let _ = writeln!(out, "{:indent$}= note: {note}", "", indent = width + 2);
    }
    for suggestion in diagnostic.suggestions() {
        let _ = writeln!(
            out,
            "{:indent$}= help: {}",
            "",
            suggestion.message(),
            indent = width + 2
        );
    }
    out
}

/// One label with everything needed to print it.
struct LabelView<'a> {
    label: &'a Label,
    primary: bool,
    location: Option<Location>,
}

struct Location {
    path: String,
    line: u32,
    column: u32,
    line_text: String,
    marker_len: u32,
}

impl<'a> LabelView<'a> {
    fn resolve(label: &'a Label, primary: bool, sources: &SourceMap) -> Self {
        let span = label.span();
        let location = sources.get(span.file()).and_then(|file| {
            let start = file.line_col(span.start())?;
            let line_index = start.line.checked_sub(1)?;
            let line_text = file.line_text(line_index)?.replace('\t', " ");
            let marker_len = match file.line_col(span.end()) {
                Some(end) if end.line == start.line => end.column.saturating_sub(start.column),
                // Span ends on another line (or at an unmappable offset):
                // underline to the end of the first line.
                _ => u32::try_from(
                    line_text
                        .chars()
                        .skip(usize::try_from(start.column.saturating_sub(1)).ok()?)
                        .count(),
                )
                .ok()?,
            }
            .max(1);
            Some(Location {
                path: file.path().display().to_string(),
                line: start.line,
                column: start.column,
                line_text,
                marker_len,
            })
        });
        Self {
            label,
            primary,
            location,
        }
    }

    fn render_into(&self, out: &mut String, width: usize, sources: &SourceMap) {
        let indent = " ".repeat(width);
        match &self.location {
            Some(location) => {
                let _ = writeln!(
                    out,
                    "{indent}--> {}:{}:{}",
                    location.path, location.line, location.column
                );
                let _ = writeln!(out, "{indent} |");
                let _ = writeln!(
                    out,
                    "{:>width$} | {}",
                    location.line,
                    location.line_text,
                    width = width
                );
                let marker = if self.primary { '^' } else { '-' };
                let pad =
                    " ".repeat(usize::try_from(location.column.saturating_sub(1)).unwrap_or(0));
                let marks: String =
                    std::iter::repeat_n(marker, usize::try_from(location.marker_len).unwrap_or(1))
                        .collect();
                let message = self.label.message();
                if message.is_empty() {
                    let _ = writeln!(out, "{indent} | {pad}{marks}");
                } else {
                    let _ = writeln!(out, "{indent} | {pad}{marks} {message}");
                }
                let _ = writeln!(out, "{indent} |");
            }
            None => {
                let span = self.label.span();
                let sub = match sources.get(span.file()) {
                    Some(file) => file.path().display().to_string(),
                    None => format!("<unknown source file #{}>", span.file().to_raw()),
                };
                let message = self.label.message();
                if message.is_empty() {
                    let _ = writeln!(
                        out,
                        "{indent}--> {sub}:bytes {}..{}",
                        span.start(),
                        span.end()
                    );
                } else {
                    let _ = writeln!(
                        out,
                        "{indent}--> {sub}:bytes {}..{} {message}",
                        span.start(),
                        span.end()
                    );
                }
            }
        }
    }
}

fn digit_count(mut value: u32) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}
