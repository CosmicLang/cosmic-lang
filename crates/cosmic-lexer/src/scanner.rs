//! The scanning core: `lex` and the element-producing loop.
//!
//! The scanner walks the text once, in byte order, emitting exactly one
//! element per step and always advancing. Recovery is local: an unterminated
//! block comment or string literal becomes an error region reaching the end
//! of the file (one diagnostic, no cascade); a refused triple-quoted string
//! resumes after its closing `"""`; an unexpected character consumes exactly
//! one scalar value.

use core::ops::Range;

use cosmic_diagnostics::{Diagnostic, DiagnosticBag, Label};
use cosmic_source::{SourceFile, Span};

use crate::codes::LexErrorCode;
use crate::element::{ElementKind, LexElement, LexedFile, TerminatorKind};
use crate::escapes::scan_escapes;
use crate::token::{TokenKind, keyword_kind};

/// The leading UTF-8 byte-order mark, `U+FEFF` (`EF BB BF` in UTF-8).
const BOM: char = '\u{FEFF}';
const BOM_LEN: usize = BOM.len_utf8();

/// Lexes one registered source file into a lossless [`LexedFile`].
///
/// The function is total: any `SourceFile` — including malformed source text
/// — produces a result. Lexical errors become [`LexErrorCode`] diagnostics
/// on the returned [`LexedFile`]; they never abort the scan, and no byte of
/// the file is dropped (see [`LexedFile::is_lossless`]).
#[must_use]
pub fn lex(source: &SourceFile) -> LexedFile {
    Scanner::new(source).run()
}

struct Scanner<'a> {
    source: &'a SourceFile,
    text: &'a str,
    bytes: &'a [u8],
    pos: usize,
    elements: Vec<LexElement>,
    diagnostics: DiagnosticBag,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a SourceFile) -> Self {
        let text = source.text();
        Self {
            source,
            text,
            bytes: text.as_bytes(),
            pos: 0,
            elements: Vec::new(),
            diagnostics: DiagnosticBag::new(),
        }
    }

    fn run(mut self) -> LexedFile {
        if self.text.starts_with(BOM) {
            // D-L1 §4: the BOM is a prefix leaf, not trivia and not part of
            // the first token; it is recognised only at byte 0.
            self.push_element(ElementKind::Bom, 0, BOM_LEN, Vec::new());
            self.pos = BOM_LEN;
        }
        while self.pos < self.text.len() {
            self.step();
        }
        LexedFile::new(self.source.id(), self.elements, self.diagnostics)
    }

    /// Emits exactly one element and advances. Every branch must advance by
    /// at least one byte, so the scan terminates on every input.
    fn step(&mut self) {
        match self.bytes[self.pos] {
            b'\n' => self.terminator_lf(),
            b'\r' => self.terminator_cr(),
            b' ' | b'\t' | b'\x0C' => self.whitespace(),
            b'/' if self.bytes.get(self.pos + 1) == Some(&b'/') => self.line_comment(),
            b'/' if self.bytes.get(self.pos + 1) == Some(&b'*') => self.block_comment(),
            b'"' => self.string_or_triple(),
            b'\'' => self.char_literal(),
            b'0'..=b'9' => self.number(),
            _ => {
                let c = self.char_at(self.pos);
                if c.is_alphabetic() || c == '_' || c == '$' {
                    self.identifier();
                } else {
                    self.operator_or_unexpected();
                }
            }
        }
    }

    fn terminator_lf(&mut self) {
        let start = self.pos;
        self.push_element(
            ElementKind::LineTerminator(TerminatorKind::Lf),
            start,
            start + 1,
            Vec::new(),
        );
        self.pos += 1;
    }

    fn terminator_cr(&mut self) {
        let start = self.pos;
        let (kind, len) = if self.bytes.get(start + 1) == Some(&b'\n') {
            (TerminatorKind::CrLf, 2)
        } else {
            (TerminatorKind::Cr, 1)
        };
        self.push_element(
            ElementKind::LineTerminator(kind),
            start,
            start + len,
            Vec::new(),
        );
        self.pos += len;
    }

    fn whitespace(&mut self) {
        let start = self.pos;
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\t' | b'\x0C')) {
            self.pos += 1;
        }
        self.push_element(ElementKind::Whitespace, start, self.pos, Vec::new());
    }

    /// A `//` comment. Per the accepted grammar §2.1 the comment excludes the
    /// line terminator, and `LF`, `CRLF` and `CR` all terminate the line; the
    /// terminator is the next element, in its exact form.
    fn line_comment(&mut self) {
        let start = self.pos;
        self.pos += 2;
        while let Some(c) = self.char_at_opt(self.pos) {
            if c == '\n' || c == '\r' {
                break;
            }
            self.pos += c.len_utf8();
        }
        self.push_element(ElementKind::LineComment, start, self.pos, Vec::new());
    }

    /// A `/* … */` comment. Block comments do **not** nest (grammar §2.1):
    /// the first `*/` closes the comment. An unterminated comment produces
    /// exactly one diagnostic and one error region reaching end of file.
    fn block_comment(&mut self) {
        let start = self.pos;
        self.pos += 2;
        loop {
            if self.pos >= self.text.len() {
                self.push_element(
                    ElementKind::Error,
                    start,
                    self.text.len(),
                    vec![LexErrorCode::Lex001],
                );
                self.report_with_note(
                    LexErrorCode::Lex001,
                    self.span(start, start + 2),
                    "unterminated comment",
                    "unterminated block comment",
                    "the comment reaches the end of the file without a closing `*/` and is preserved as an error region",
                );
                self.pos = self.text.len();
                return;
            }
            if self.bytes[self.pos] == b'*' && self.bytes.get(self.pos + 1) == Some(&b'/') {
                self.pos += 2;
                break;
            }
            self.pos += self.char_at(self.pos).len_utf8();
        }
        self.push_element(ElementKind::BlockComment, start, self.pos, Vec::new());
    }

    fn string_or_triple(&mut self) {
        // D-L6 item 4: detection is positional — a quote immediately followed
        // by two more quotes begins the refused sequence. `""` stays a legal
        // empty string.
        if self.bytes.get(self.pos + 1) == Some(&b'"')
            && self.bytes.get(self.pos + 2) == Some(&b'"')
        {
            self.triple_quoted_string();
        } else {
            self.string();
        }
    }

    fn triple_quoted_string(&mut self) {
        let start = self.pos;
        self.pos += 3;
        loop {
            if self.pos >= self.text.len() {
                break;
            }
            if self.bytes.get(self.pos) == Some(&b'"')
                && self.bytes.get(self.pos + 1) == Some(&b'"')
                && self.bytes.get(self.pos + 2) == Some(&b'"')
            {
                self.pos += 3;
                break;
            }
            self.pos += self.char_at(self.pos).len_utf8();
        }
        self.push_element(
            ElementKind::Error,
            start,
            self.pos,
            vec![LexErrorCode::Lex009],
        );
        self.report_with_note(
            LexErrorCode::Lex009,
            self.span(start, start + 3),
            "no triple-quoted string literal",
            "triple-quoted string literal",
            "Cosmic-Lang has no raw, multiline or interpolated string literals; write line breaks inside a `\"...\"` literal or concatenate with `+`",
        );
    }

    /// A `"…"` string literal. Raw line breaks are content (D-L7), so the
    /// literal closes only at its `"`, and an unterminated literal reaches
    /// end of file: one diagnostic (`LEX002`), one error region, no cascade.
    fn string(&mut self) {
        let start = self.pos;
        let mut p = start + 1;
        loop {
            if p >= self.text.len() {
                self.push_element(
                    ElementKind::Error,
                    start,
                    self.text.len(),
                    vec![LexErrorCode::Lex002],
                );
                self.report_with_note(
                    LexErrorCode::Lex002,
                    self.span(start, start + 1),
                    "string is not closed",
                    "unterminated string literal",
                    "a string literal must close with `\"` before the end of the file; raw line breaks inside the literal are permitted",
                );
                self.pos = self.text.len();
                return;
            }
            match self.bytes[p] {
                b'"' => {
                    p += 1;
                    break;
                }
                b'\\' => {
                    // Skip the escaped character so `\"` never closes the
                    // literal. A trailing backslash before end of file is
                    // handled by the unterminated branch above.
                    p += 1;
                    if p < self.text.len() {
                        p += self.char_at(p).len_utf8();
                    }
                }
                _ => p += self.char_at(p).len_utf8(),
            }
        }
        let interior_start = start + 1;
        let interior = &self.text[interior_start..p - 1];
        let scan = scan_escapes(interior);
        let mut errors = Vec::new();
        for issue in &scan.issues {
            let issue_start = interior_start + issue.range.start;
            let issue_end = interior_start + issue.range.end;
            self.report_escape(issue.code, issue_start, issue_end);
            errors.push(issue.code);
        }
        self.push_element(
            ElementKind::Token(TokenKind::StringLiteral),
            start,
            p,
            errors,
        );
        self.pos = p;
    }

    /// A `'…'` character literal.
    ///
    /// The grammar admits exactly one `char-value` between the quotes, and
    /// `char-value` excludes line terminators. Consequences of the accepted
    /// contract (D-L5):
    ///
    /// * `''` is an empty literal — `LEX003`;
    /// * a literal that does not close before a line break or end of file is
    ///   unterminated — `LEX004`;
    /// * a literal carrying more than one character value is refused with
    ///   `LEX004` and preserved as one error-marked token (when it closes on
    ///   the same line) or one error region — never silently truncated.
    fn char_literal(&mut self) {
        let start = self.pos;
        let len = self.text.len();
        let value_start = start + 1;
        if value_start >= len {
            // A lone `'` at end of file: there is no closing quote, so this
            // is unterminated, not empty.
            self.unterminated_char(start, len);
            return;
        }
        if self.bytes[value_start] == b'\'' {
            self.push_element(
                ElementKind::Error,
                start,
                value_start + 1,
                vec![LexErrorCode::Lex003],
            );
            self.report_with_note(
                LexErrorCode::Lex003,
                self.span(start, value_start + 1),
                "no character between the quotes",
                "empty character literal",
                "a character literal must contain exactly one character",
            );
            self.pos = value_start + 1;
            return;
        }
        let mut p = value_start;
        let mut escape: Option<Range<usize>> = None;
        if self.bytes[p] == b'\\' {
            p += 1;
            if p >= len {
                self.unterminated_char(start, len);
                return;
            }
            let escaped = self.char_at(p);
            if escaped == '\n' || escaped == '\r' {
                // The escape has no value before the line break: the literal
                // cannot close on this line.
                self.unterminated_char(start, p);
                return;
            }
            p += escaped.len_utf8();
            if escaped == 'u' {
                // The `\u` window never crosses the closing quote or a line
                // terminator, so an incomplete window stays inside the
                // literal and reports `LEX006` instead of swallowing the
                // rest of the line.
                for _ in 0..4 {
                    match self.bytes.get(p) {
                        None | Some(b'\'' | b'\n' | b'\r') => break,
                        _ => p += self.char_at(p).len_utf8(),
                    }
                }
            }
            escape = Some(value_start..p);
        } else {
            let c = self.char_at(p);
            if c == '\n' || c == '\r' {
                self.unterminated_char(start, p);
                return;
            }
            p += c.len_utf8();
        }
        if self.bytes.get(p) == Some(&b'\'') {
            let mut errors = Vec::new();
            if let Some(range) = escape {
                let scan = scan_escapes(&self.text[range.clone()]);
                for issue in &scan.issues {
                    self.report_escape(
                        issue.code,
                        range.start + issue.range.start,
                        range.start + issue.range.end,
                    );
                    errors.push(issue.code);
                }
            }
            self.push_element(
                ElementKind::Token(TokenKind::CharLiteral),
                start,
                p + 1,
                errors,
            );
            self.pos = p + 1;
            return;
        }
        if p >= len || matches!(self.bytes.get(p), Some(b'\n' | b'\r')) {
            self.unterminated_char(start, p);
            return;
        }
        // Multi-character: consume through the closing quote when it sits on
        // the same line, otherwise up to the line break, and report exactly
        // one diagnostic for the whole literal.
        let mut q = p;
        loop {
            match self.bytes.get(q) {
                None | Some(b'\n' | b'\r') => break,
                Some(b'\'') => {
                    q += 1;
                    break;
                }
                Some(b'\\') => {
                    q += 1;
                    if let Some(c) = self.char_at_opt(q)
                        && c != '\n'
                        && c != '\r'
                    {
                        q += c.len_utf8();
                    }
                }
                Some(_) => q += self.char_at(q).len_utf8(),
            }
        }
        if self.bytes.get(q - 1) == Some(&b'\'') {
            self.push_element(
                ElementKind::Token(TokenKind::CharLiteral),
                start,
                q,
                vec![LexErrorCode::Lex004],
            );
            self.report_with_note(
                LexErrorCode::Lex004,
                self.span(start, q),
                "more than one character value",
                "multi-character character literal",
                "a character literal must contain exactly one character value",
            );
        } else {
            self.push_element(ElementKind::Error, start, q, vec![LexErrorCode::Lex004]);
            self.report_with_note(
                LexErrorCode::Lex004,
                self.span(start, start + 1),
                "literal is not exactly one character",
                "unterminated character literal",
                "a character literal must close with `'` on the same line",
            );
        }
        self.pos = q;
    }

    /// Pushes the shared `LEX004` diagnostic and the error region for a
    /// character literal with no closing quote before `end`.
    fn unterminated_char(&mut self, start: usize, end: usize) {
        self.push_element(ElementKind::Error, start, end, vec![LexErrorCode::Lex004]);
        self.report_with_note(
            LexErrorCode::Lex004,
            self.span(start, start + 1),
            "literal is not exactly one character",
            "unterminated character literal",
            "a character literal must close with `'` on the same line",
        );
        self.pos = end;
    }

    /// A numeric literal, exactly as the accepted grammar derives it
    /// (grammar §2.4): decimal or `0x` hex integers, an optional fraction
    /// introduced only when a digit follows the `.`, an optional exponent
    /// introduced only when a digit follows `e`/`E` (with an optional sign),
    /// the `f`/`F` and `d`/`D` suffixes on any number form, and the `l`/`L`
    /// suffix only on the integer forms (`long-literal = ( decimal-int |
    /// hex-int ) , ( "l" | "L" )`).
    ///
    /// A malformed or misplaced suffix tail is **not** consumed: `0x` without
    /// a hexadecimal digit lexes as `0` then an identifier, `1e` without
    /// exponent digits lexes as `1` then an identifier, and `1.2L` lexes as a
    /// double then an identifier — the longest valid tokenisation under the
    /// accepted grammar, with no invented diagnostic and no silently accepted
    /// malformed form. Value-based promotion is a semantic question (S7) and
    /// is deliberately not performed.
    fn number(&mut self) {
        let start = self.pos;
        let mut p = start;
        let mut kind = TokenKind::IntLiteral;
        if self.bytes[p] == b'0'
            && matches!(self.bytes.get(p + 1), Some(b'x' | b'X'))
            && self
                .bytes
                .get(p + 2)
                .is_some_and(|byte| byte.is_ascii_hexdigit())
        {
            p += 2;
            while self
                .bytes
                .get(p)
                .is_some_and(|byte| byte.is_ascii_hexdigit())
            {
                p += 1;
            }
            if matches!(self.bytes.get(p), Some(b'l' | b'L')) {
                kind = TokenKind::LongLiteral;
                p += 1;
            }
        } else {
            while self.bytes.get(p).is_some_and(|byte| byte.is_ascii_digit()) {
                p += 1;
            }
            let mut is_double = false;
            if self.bytes.get(p) == Some(&b'.')
                && self
                    .bytes
                    .get(p + 1)
                    .is_some_and(|byte| byte.is_ascii_digit())
            {
                p += 1;
                is_double = true;
                while self.bytes.get(p).is_some_and(|byte| byte.is_ascii_digit()) {
                    p += 1;
                }
            }
            if matches!(self.bytes.get(p), Some(b'e' | b'E')) {
                let mut q = p + 1;
                if matches!(self.bytes.get(q), Some(b'+' | b'-')) {
                    q += 1;
                }
                if self.bytes.get(q).is_some_and(|byte| byte.is_ascii_digit()) {
                    is_double = true;
                    p = q;
                    while self.bytes.get(p).is_some_and(|byte| byte.is_ascii_digit()) {
                        p += 1;
                    }
                }
            }
            match self.bytes.get(p) {
                Some(b'f' | b'F') => {
                    kind = TokenKind::FloatLiteral;
                    p += 1;
                }
                Some(b'd' | b'D') => {
                    kind = TokenKind::DoubleLiteral;
                    p += 1;
                }
                // `long-literal` admits `l`/`L` only after `decimal-int` or
                // `hex-int`, never after a fraction or exponent: the suffix
                // is not consumed, so the byte starts an identifier instead.
                Some(b'l' | b'L') if !is_double => {
                    kind = TokenKind::LongLiteral;
                    p += 1;
                }
                _ if is_double => kind = TokenKind::DoubleLiteral,
                _ => {}
            }
        }
        self.push_element(ElementKind::Token(kind), start, p, Vec::new());
        self.pos = p;
    }

    /// An identifier-shaped token: `identifier-start` then
    /// `identifier-part`*, classified by the keyword table (grammar §2.2–§2.3).
    ///
    /// The character classes are the Rust standard-library predicates
    /// `char::is_alphabetic` and `char::is_alphanumeric` (D-L3), plus `_` and
    /// `$` at every position. The Unicode-category edges (combining marks,
    /// letters outside the legacy `Character.isLetter` classification) are
    /// owned by the S13 measurement obligations; see the crate docs.
    fn identifier(&mut self) {
        let start = self.pos;
        let mut p = start;
        while let Some(c) = self.char_at_opt(p) {
            if c.is_alphanumeric() || c == '_' || c == '$' {
                p += c.len_utf8();
            } else {
                break;
            }
        }
        let name = &self.text[start..p];
        let kind = keyword_kind(name).unwrap_or(TokenKind::Identifier);
        self.push_element(ElementKind::Token(kind), start, p, Vec::new());
        self.pos = p;
    }

    /// One operator, delimiter or separator (longest match), or
    /// [`LexErrorCode::Lex005`] for a character the token grammar does not
    /// recognise. Exactly one scalar value is consumed in the error case, so
    /// recovery never loses bytes or stalls.
    fn operator_or_unexpected(&mut self) {
        let start = self.pos;
        if let Some((kind, len)) = self.operator() {
            self.push_element(ElementKind::Token(kind), start, start + len, Vec::new());
            self.pos = start + len;
            return;
        }
        let c = self.char_at(start);
        self.pos = start + c.len_utf8();
        self.push_element(
            ElementKind::Error,
            start,
            self.pos,
            vec![LexErrorCode::Lex005],
        );
        if c == BOM {
            self.report_with_note(
                LexErrorCode::Lex005,
                self.span(start, self.pos),
                "unexpected character",
                format!("unexpected character `{}`", c.escape_debug()),
                "a byte-order mark is recognised only at the start of a file",
            );
        } else {
            self.report(
                LexErrorCode::Lex005,
                self.span(start, self.pos),
                "unexpected character",
                format!("unexpected character `{}`", c.escape_debug()),
            );
        }
    }

    /// The operator/delimiter/separator tables of grammar §2.5, longest
    /// match first (`>>>=` before `>>>` before `>>`, `<<=` before `<<`, and
    /// so on). `//` and `/*` never reach this function — the dispatcher
    /// handles them as comments first.
    fn operator(&self) -> Option<(TokenKind, usize)> {
        use TokenKind::*;
        let b = self.bytes[self.pos];
        let n1 = self.bytes.get(self.pos + 1).copied();
        let n2 = self.bytes.get(self.pos + 2).copied();
        let n3 = self.bytes.get(self.pos + 3).copied();
        Some(match b {
            b'+' => match n1 {
                Some(b'+') => (PlusPlus, 2),
                Some(b'=') => (PlusEq, 2),
                _ => (Plus, 1),
            },
            b'-' => match n1 {
                Some(b'-') => (MinusMinus, 2),
                Some(b'=') => (MinusEq, 2),
                Some(b'>') => (Arrow, 2),
                _ => (Minus, 1),
            },
            b'*' => match n1 {
                Some(b'=') => (StarEq, 2),
                _ => (Star, 1),
            },
            b'/' => match n1 {
                Some(b'=') => (SlashEq, 2),
                _ => (Slash, 1),
            },
            b'%' => match n1 {
                Some(b'=') => (PercentEq, 2),
                _ => (Percent, 1),
            },
            b'!' => match n1 {
                Some(b'=') => (BangEq, 2),
                _ => (Bang, 1),
            },
            b'=' => match n1 {
                Some(b'=') => (EqEq, 2),
                _ => (Eq, 1),
            },
            b'&' => match n1 {
                Some(b'&') => (AmpAmp, 2),
                Some(b'=') => (AmpEq, 2),
                _ => (Amp, 1),
            },
            b'|' => match n1 {
                Some(b'|') => (PipePipe, 2),
                Some(b'=') => (PipeEq, 2),
                _ => (Pipe, 1),
            },
            b'^' => match n1 {
                Some(b'=') => (CaretEq, 2),
                _ => (Caret, 1),
            },
            b'<' => match n1 {
                Some(b'=') => (LtEq, 2),
                Some(b'<') => match n2 {
                    Some(b'=') => (ShlEq, 3),
                    _ => (Shl, 2),
                },
                _ => (Lt, 1),
            },
            b'>' => match n1 {
                Some(b'=') => (GtEq, 2),
                Some(b'>') => match n2 {
                    Some(b'>') => match n3 {
                        Some(b'=') => (UshrEq, 4),
                        _ => (Ushr, 3),
                    },
                    Some(b'=') => (ShrEq, 3),
                    _ => (Shr, 2),
                },
                _ => (Gt, 1),
            },
            b'(' => (LParen, 1),
            b')' => (RParen, 1),
            b'{' => (LBrace, 1),
            b'}' => (RBrace, 1),
            b'[' => (LBracket, 1),
            b']' => (RBracket, 1),
            b';' => (Semicolon, 1),
            b',' => (Comma, 1),
            b'.' => (Dot, 1),
            b':' => (Colon, 1),
            b'?' => (Question, 1),
            b'@' => (At, 1),
            _ => return None,
        })
    }

    fn report_escape(&mut self, code: LexErrorCode, start: usize, end: usize) {
        let span = self.span(start, end);
        let text = &self.text[start..end];
        match code {
            LexErrorCode::Lex006 => self.report_with_note(
                code,
                span,
                "escape is incomplete",
                "incomplete `\\u` escape",
                "`\\u` must be followed by exactly four hexadecimal digits inside the literal",
            ),
            LexErrorCode::Lex007 => self.report_with_note(
                code,
                span,
                "not a hexadecimal digit",
                format!("invalid `\\u` escape `{text}`"),
                "the four characters after `\\u` must be hexadecimal digits",
            ),
            LexErrorCode::Lex008 => self.report_with_note(
                code,
                span,
                "unsupported escape",
                format!("unknown escape sequence `{text}`"),
                "supported escapes: `\\n`, `\\t`, `\\r`, `\\\\`, `\\'`, `\\\"`, `\\0` and `\\uXXXX`",
            ),
            LexErrorCode::Lex010 => self.report_with_note(
                code,
                span,
                "surrogate is not paired",
                "lone surrogate in `\\u` escape",
                "a `\\u` escape naming a UTF-16 surrogate must pair with an immediately following low-surrogate `\\u` escape to form one character",
            ),
            _ => self.report(code, span, "invalid escape", "invalid escape"),
        }
    }

    fn report(
        &mut self,
        code: LexErrorCode,
        label_span: Span,
        label: &'static str,
        message: impl Into<String>,
    ) {
        self.diagnostics.push(Diagnostic::error(
            code.code(),
            message,
            Label::new(label_span, label),
        ));
    }

    fn report_with_note(
        &mut self,
        code: LexErrorCode,
        label_span: Span,
        label: &'static str,
        message: impl Into<String>,
        note: impl Into<String>,
    ) {
        self.diagnostics.push(
            Diagnostic::error(code.code(), message, Label::new(label_span, label)).with_note(note),
        );
    }

    fn push_element(
        &mut self,
        kind: ElementKind,
        start: usize,
        end: usize,
        errors: Vec<LexErrorCode>,
    ) {
        let span = self.span(start, end);
        self.elements.push(LexElement::new(kind, span, errors));
    }

    fn span(&self, start: usize, end: usize) -> Span {
        let start = u32::try_from(start).expect("source length is below u32::MAX");
        let end = u32::try_from(end).expect("source length is below u32::MAX");
        self.source
            .span(start, end)
            .expect("scanner bounds stay on character boundaries")
    }

    fn char_at(&self, offset: usize) -> char {
        self.char_at_opt(offset)
            .expect("scanner offsets stay on character boundaries")
    }

    fn char_at_opt(&self, offset: usize) -> Option<char> {
        self.text.get(offset..)?.chars().next()
    }
}

#[cfg(test)]
mod tests {
    use super::{Scanner, TokenKind};
    use cosmic_source::SourceMap;

    fn operator_tokens(text: &str) -> Vec<(TokenKind, String)> {
        let mut sources = SourceMap::new();
        let id = sources.add("operators.cmic", text).unwrap();
        let source = sources.get(id).unwrap();
        let lexed = super::lex(source);
        lexed
            .tokens()
            .map(|element| {
                let kind = match element.kind() {
                    crate::ElementKind::Token(kind) => kind,
                    other => panic!("expected a token, found {other:?}"),
                };
                let text = source.text_of(element.span()).unwrap().to_owned();
                (kind, text)
            })
            .collect()
    }

    #[test]
    fn longest_match_prefers_the_longest_operator() {
        for (text, expected) in [
            (">>>=", TokenKind::UshrEq),
            (">>>", TokenKind::Ushr),
            (">>=", TokenKind::ShrEq),
            (">>", TokenKind::Shr),
            (">=", TokenKind::GtEq),
            (">", TokenKind::Gt),
            ("<<=", TokenKind::ShlEq),
            ("<<", TokenKind::Shl),
            ("<=", TokenKind::LtEq),
            ("<", TokenKind::Lt),
            ("--", TokenKind::MinusMinus),
            ("->", TokenKind::Arrow),
            ("-=", TokenKind::MinusEq),
            ("-", TokenKind::Minus),
            ("++", TokenKind::PlusPlus),
            ("+=", TokenKind::PlusEq),
        ] {
            let tokens = operator_tokens(text);
            assert_eq!(tokens.len(), 1, "{text}");
            assert_eq!(tokens[0], (expected, text.to_owned()), "{text}");
        }
    }

    #[test]
    fn scan_terminates_on_every_byte_prefix() {
        // Truncation at any char boundary must still terminate and consume
        // every byte: a smoke test for the advance invariants.
        let text = r#"var s = "a\u12" /* /* venue */ 'x' 0x1e5 """ "#;
        let mut sources = SourceMap::new();
        for (offset, _) in text.char_indices() {
            let prefix = &text[..offset];
            let id = sources
                .add(format!("prefix-{offset}.cmic"), prefix)
                .unwrap();
            let source = sources.get(id).unwrap();
            let lexed = super::lex(source);
            assert!(
                lexed.is_lossless(source),
                "prefix {offset} must be lossless"
            );
        }
    }

    #[test]
    fn scanner_dispatch_advances_on_unknown_bytes() {
        // `Scanner::step` covers the unexpected-character branch; verify the
        // advance by lexing a lone unknown byte directly.
        let mut sources = SourceMap::new();
        let id = sources.add("tilde.cmic", "~").unwrap();
        let source = sources.get(id).unwrap();
        let mut scanner = Scanner::new(source);
        scanner.step();
        assert_eq!(scanner.pos, 1);
    }
}
