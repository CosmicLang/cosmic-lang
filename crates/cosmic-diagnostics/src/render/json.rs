use core::fmt::Write as _;

use cosmic_source::SourceMap;

use crate::{Diagnostic, Label};

/// Renders a diagnostic as a deterministic single-line JSON document.
///
/// Schema (all fields always present; unknown values are `null`):
///
/// ```json
/// {
///   "severity": "error",
///   "code": "CSM001",
///   "message": "expected expression",
///   "labels": [
///     {
///       "primary": true,
///       "file": "demo.csmx",
///       "file_id": 1,
///       "start": 12,
///       "end": 13,
///       "line": 1,
///       "column": 13,
///       "message": "expected an expression here"
///     }
///   ],
///   "notes": ["..."],
///   "suggestions": [
///     {
///       "file": "demo.csmx",
///       "file_id": 1,
///       "start": 12,
///       "end": 13,
///       "replacement": "0",
///       "message": "replace with `0`",
///       "applicability": "machine-applicable"
///     }
///   ]
/// }
/// ```
///
/// `line`/`column` are derived from the rendering `SourceMap`; `null` when
/// the file is not registered there. `file` is `null` under the same
/// condition. Byte offsets (`start`/`end`) are always the authoritative
/// coordinates. Keys are emitted in schema order and control characters are
/// escaped per RFC 8259; the output is identical across runs for identical
/// inputs.
#[must_use]
pub fn render(diagnostic: &Diagnostic, sources: &SourceMap) -> String {
    let mut out = String::new();

    let mut root = Obj::new(&mut out);
    root.str("severity", diagnostic.severity().as_str());
    root.str("code", diagnostic.code().as_str());
    root.str("message", diagnostic.message());

    root.key("labels");
    {
        let mut labels = Arr::new(root.raw());
        render_label(&mut labels, diagnostic.primary(), true, sources);
        for label in diagnostic.secondary() {
            render_label(&mut labels, label, false, sources);
        }
        labels.finish();
    }

    root.key("notes");
    {
        let mut notes = Arr::new(root.raw());
        for note in diagnostic.notes() {
            notes.string(note);
        }
        notes.finish();
    }

    root.key("suggestions");
    {
        let mut suggestions = Arr::new(root.raw());
        for suggestion in diagnostic.suggestions() {
            let span = suggestion.span();
            let mut object = Obj::new(suggestions.raw());
            object.opt_str("file", file_path(sources, span.file().to_raw()));
            object.num("file_id", span.file().to_raw());
            object.num("start", span.start());
            object.num("end", span.end());
            object.str("replacement", suggestion.replacement());
            object.str("message", suggestion.message());
            object.str("applicability", suggestion.applicability().as_str());
            object.finish();
        }
        suggestions.finish();
    }

    root.finish();
    out
}

fn render_label(arr: &mut Arr<'_>, label: &Label, primary: bool, sources: &SourceMap) {
    let span = label.span();
    let file_id = span.file().to_raw();
    let location = sources
        .get(span.file())
        .and_then(|file| file.line_col(span.start()));

    let mut object = Obj::new(arr.raw());
    object.bool("primary", primary);
    object.opt_str("file", file_path(sources, file_id));
    object.num("file_id", file_id);
    object.num("start", span.start());
    object.num("end", span.end());
    object.opt_num("line", location.map(|loc| loc.line));
    object.opt_num("column", location.map(|loc| loc.column));
    object.str("message", label.message());
    object.finish();
}

fn file_path(sources: &SourceMap, file_id: u32) -> Option<String> {
    let id = cosmic_source::FileId::from_raw(file_id)?;
    Some(sources.get(id)?.path().display().to_string())
}

/// Minimal JSON object writer with deterministic key order.
struct Obj<'a> {
    out: &'a mut String,
    first: bool,
}

impl<'a> Obj<'a> {
    fn new(out: &'a mut String) -> Self {
        out.push('{');
        Self { out, first: true }
    }

    fn raw(&mut self) -> &mut String {
        self.out
    }

    fn key(&mut self, name: &str) {
        if !self.first {
            self.out.push(',');
        }
        self.first = false;
        push_string(self.out, name);
        self.out.push(':');
    }

    fn str(&mut self, name: &str, value: &str) {
        self.key(name);
        push_string(self.out, value);
    }

    fn opt_str(&mut self, name: &str, value: Option<String>) {
        self.key(name);
        match value {
            Some(value) => push_string(self.out, &value),
            None => self.out.push_str("null"),
        }
    }

    fn num(&mut self, name: &str, value: u32) {
        self.key(name);
        let _ = write!(self.out, "{value}");
    }

    fn opt_num(&mut self, name: &str, value: Option<u32>) {
        self.key(name);
        match value {
            Some(value) => {
                let _ = write!(self.out, "{value}");
            }
            None => self.out.push_str("null"),
        }
    }

    fn bool(&mut self, name: &str, value: bool) {
        self.key(name);
        self.out.push_str(if value { "true" } else { "false" });
    }

    fn finish(self) {
        self.out.push('}');
    }
}

/// Minimal JSON array writer sharing the parent output buffer.
struct Arr<'a> {
    out: &'a mut String,
    first: bool,
}

impl<'a> Arr<'a> {
    fn new(out: &'a mut String) -> Self {
        out.push('[');
        Self { out, first: true }
    }

    fn raw(&mut self) -> &mut String {
        if !self.first {
            self.out.push(',');
        }
        self.first = false;
        self.out
    }

    fn string(&mut self, value: &str) {
        if !self.first {
            self.out.push(',');
        }
        self.first = false;
        push_string(self.out, value);
    }

    fn finish(self) {
        self.out.push(']');
    }
}

/// Escapes and writes a JSON string per RFC 8259.
fn push_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::push_string;

    #[test]
    fn escaping_covers_quotes_backslashes_controls_and_unicode() {
        let mut out = String::new();
        push_string(&mut out, "a\"b\\c\nd\te\u{1}f\u{8}g\u{c}h 😀é");
        assert_eq!(out, "\"a\\\"b\\\\c\\nd\\te\\u0001f\\bg\\fh 😀é\"",);
    }

    #[test]
    fn plain_ascii_passes_through() {
        let mut out = String::new();
        push_string(&mut out, "plain text");
        assert_eq!(out, "\"plain text\"");
    }
}
