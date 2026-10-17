//! The recovering recursive-descent parser over the lossless lexer stream.
//!
//! [`parse`] consumes a `cosmic-lexer` [`LexedFile`](cosmic_lexer::LexedFile)
//! and produces a [`ParsedFile`]: the same lossless element stream plus the
//! CST arena and the `PARSE` diagnostics.
//!
//! # What is recognised
//!
//! The parser recognises the accepted recognition subset of
//! `docs/spec/g1-s-syntax-decision-package.md` §3: every `RECOGNISED`
//! production of `docs/spec/grammar.ebnf` is parsed; the nine unsupported
//! families of §4 are **refused** with exactly one `PARSE001` diagnostic per
//! construct, the reserved-invalid words `fun`/`fn`/`func` are refused with a
//! `PARSE002` diagnostic (SG-001), and a label after `break`/`continue` is
//! refused with `PARSE004`. Refusal nodes and generic `Error` regions preserve
//! the consumed bytes; nothing is fabricated.
//!
//! # Line sensitivity
//!
//! Two accepted line rules are implemented:
//!
//! * **Statement separation** — a statement or declaration ends where the
//!   next statement begins; an expression continuation (a binary operator, a
//!   postfix operator, `as`/`instanceof`) is taken only when its token starts
//!   on the same line as the completed expression, except inside `( … )` and
//!   `[ … ]` regions. `else`, `catch`, `finally`, `while` (of `do`) and
//!   `case`/`default` are committed continuations of their construct and are
//!   not line-gated.
//! * **Trailing-lambda same-line rule** (grammar §6.1 note, accepted by
//!   003-G1 §4): a `{` that starts a *lambda* (`"{" [ lambda-params ] "->" …`)
//!   is attached as a trailing lambda only when it starts on the same line as
//!   the preceding expression; a `{` on a later line is a block statement (in
//!   statement position) or an array-literal refusal (in other expression
//!   positions).
//!
//! # Determinism, recovery, termination
//!
//! Every disambiguation uses bounded speculative parsing over the token
//! vector with full rollback (position, events, diagnostics); the chosen
//! reading is therefore a deterministic function of the token stream. Every
//! loop enforces progress, so no input can loop forever, and no accessor
//! panics on malformed input. Each failure path reports at most one
//! diagnostic per construct; recovery preserves the skipped bytes in `Error`
//! nodes.

mod decl;
mod expr;
mod stmt;

use cosmic_diagnostics::{Diagnostic, DiagnosticBag, Label};
use cosmic_lexer::{ElementKind, LexedFile, TokenKind};
use cosmic_source::{SourceFile, Span};

use crate::codes::ParseErrorCode;
use crate::kind::NodeKind;
use crate::tree::{Event, ParsedFile};

/// Parses one registered source file into a lossless [`ParsedFile`].
///
/// Lexing runs first ([`cosmic_lexer::lex`]); the parser consumes the
/// resulting element stream. `LEX` diagnostics are kept in
/// [`ParsedFile::lexed`]; `PARSE` diagnostics are in
/// [`ParsedFile::diagnostics`]. No filename extension or file-system access
/// is involved — the unit of parsing is the [`SourceFile`] text.
#[must_use]
pub fn parse(source: &SourceFile) -> ParsedFile {
    let lexed = cosmic_lexer::lex(source);
    let (events, diagnostics) = {
        let mut parser = Parser::new(source, &lexed);
        parser.parse_file();
        parser.into_parts()
    };
    ParsedFile::build(source, lexed, events, diagnostics)
}

/// One significant token of the stream.
///
/// A `>>` (`Shr`) or `>>>` (`Ushr`) lexer token is **pre-split** into two or
/// three `>` tokens whose spans partition the lexer token, so that nested
/// type arguments can close with them; the parser re-joins adjacent parts of
/// the same element into a shift operator where the grammar needs one.
#[derive(Clone, Copy, Debug)]
struct Tok {
    kind: TokenKind,
    span: Span,
    /// Index of the lexer element this token (part) mirrors.
    elem: u32,
    /// Whether a line terminator element separates this token from the
    /// previous significant token.
    preceded_by_lt: bool,
    /// Whether this is the first part of its lexer element (the part that
    /// carries the element's lexical errors).
    is_first_part: bool,
}

/// A full parser checkpoint for speculative parsing and rollback.
#[derive(Clone, Copy, Debug)]
struct Checkpoint {
    pos: usize,
    events: usize,
    diagnostics: usize,
    next_element: usize,
    depth: u32,
}

/// The parser state: token stream, output events, diagnostics.
struct Parser<'s> {
    source: &'s SourceFile,
    lexed: &'s LexedFile,
    tokens: Vec<Tok>,
    pos: usize,
    events: Vec<Event>,
    diagnostics: Vec<Diagnostic>,
    /// The first lexer element not yet emitted as a leaf.
    next_element: usize,
    /// Depth of enclosing `( … )` / `[ … ]` regions; lifts the line gate for
    /// expression continuations inside them.
    depth: u32,
}

impl<'s> Parser<'s> {
    fn new(source: &'s SourceFile, lexed: &'s LexedFile) -> Self {
        let mut tokens = Vec::new();
        let mut pending_lt = false;
        for (index, element) in lexed.elements().iter().enumerate() {
            let elem = index as u32;
            match element.kind() {
                ElementKind::LineTerminator(_) => {
                    pending_lt = true;
                }
                ElementKind::Token(TokenKind::Shr) => {
                    let span = element.span();
                    let split = source.span(span.start(), span.start() + 1).unwrap_or(span);
                    let tail = source.span(span.start() + 1, span.end()).unwrap_or(span);
                    tokens.push(Tok {
                        kind: TokenKind::Gt,
                        span: split,
                        elem,
                        preceded_by_lt: pending_lt,
                        is_first_part: true,
                    });
                    tokens.push(Tok {
                        kind: TokenKind::Gt,
                        span: tail,
                        elem,
                        preceded_by_lt: false,
                        is_first_part: false,
                    });
                    pending_lt = false;
                }
                ElementKind::Token(TokenKind::Ushr) => {
                    let span = element.span();
                    let a = source.span(span.start(), span.start() + 1).unwrap_or(span);
                    let b = source
                        .span(span.start() + 1, span.start() + 2)
                        .unwrap_or(span);
                    let c = source.span(span.start() + 2, span.end()).unwrap_or(span);
                    tokens.push(Tok {
                        kind: TokenKind::Gt,
                        span: a,
                        elem,
                        preceded_by_lt: pending_lt,
                        is_first_part: true,
                    });
                    tokens.push(Tok {
                        kind: TokenKind::Gt,
                        span: b,
                        elem,
                        preceded_by_lt: false,
                        is_first_part: false,
                    });
                    tokens.push(Tok {
                        kind: TokenKind::Gt,
                        span: c,
                        elem,
                        preceded_by_lt: false,
                        is_first_part: false,
                    });
                    pending_lt = false;
                }
                ElementKind::Token(kind) => {
                    tokens.push(Tok {
                        kind,
                        span: element.span(),
                        elem,
                        preceded_by_lt: pending_lt,
                        is_first_part: true,
                    });
                    pending_lt = false;
                }
                ElementKind::Bom
                | ElementKind::Whitespace
                | ElementKind::LineComment
                | ElementKind::BlockComment
                | ElementKind::Error => {}
            }
        }
        Self {
            source,
            lexed,
            tokens,
            pos: 0,
            events: Vec::new(),
            diagnostics: Vec::new(),
            next_element: 0,
            depth: 0,
        }
    }

    fn into_parts(self) -> (Vec<Event>, DiagnosticBag) {
        let mut bag = DiagnosticBag::new();
        bag.extend(self.diagnostics);
        (self.events, bag)
    }

    // --- Token stream access -------------------------------------------

    fn current(&self) -> Option<Tok> {
        self.tokens.get(self.pos).copied()
    }

    fn cur_kind(&self) -> Option<TokenKind> {
        self.current().map(|tok| tok.kind)
    }

    fn nth(&self, offset: usize) -> Option<TokenKind> {
        self.tokens.get(self.pos + offset).map(|tok| tok.kind)
    }

    fn nth_tok(&self, offset: usize) -> Option<Tok> {
        self.tokens.get(self.pos + offset).copied()
    }

    fn eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.cur_kind() == Some(kind)
    }

    /// Whether the current token starts on the same line as the previous
    /// significant token.
    fn same_line(&self) -> bool {
        self.current().is_some_and(|tok| !tok.preceded_by_lt)
    }

    fn preceded_by_lt(&self) -> bool {
        self.current().is_some_and(|tok| tok.preceded_by_lt)
    }

    /// The line gate for expression continuations: continuation tokens must
    /// be on the same line, except inside `( … )` / `[ … ]` regions.
    fn line_ok(&self) -> bool {
        !self.preceded_by_lt() || self.depth > 0
    }

    /// An adjacent `>` part of the same lexer element (for `>>`/`>>>`
    /// re-joining), if the current token is one.
    fn next_gt_part(&self) -> bool {
        match (self.nth_tok(0), self.nth_tok(1)) {
            (Some(a), Some(b)) => {
                a.kind == TokenKind::Gt
                    && b.kind == TokenKind::Gt
                    && a.elem == b.elem
                    && a.span.end() == b.span.start()
                    && !b.preceded_by_lt
            }
            _ => false,
        }
    }

    /// The exact source text of the current token, if any.
    fn current_text(&self) -> Option<&'s str> {
        let tok = self.current()?;
        self.source.text_of(tok.span)
    }

    /// Whether the current token is the contextual keyword `name`
    /// (a plain identifier in the lexer that this parser recognises by word
    /// plus lookahead, grammar §3.2).
    fn is_contextual(&self, name: &str) -> bool {
        self.cur_kind() == Some(TokenKind::Identifier) && self.current_text() == Some(name)
    }

    fn nth_is_contextual(&self, offset: usize, name: &str) -> bool {
        match (self.nth(offset), self.nth_tok(offset)) {
            (Some(TokenKind::Identifier), Some(tok)) => self.source.text_of(tok.span) == Some(name),
            _ => false,
        }
    }

    /// A short, printable preview of the current token for diagnostics.
    fn preview_current(&self) -> String {
        match self.current_text() {
            Some(text) => preview(text),
            None => "end of input".to_string(),
        }
    }

    // --- Consumption and leaf emission ---------------------------------

    /// Consumes the current token, first flushing any lexer elements between
    /// the last consumed one and this token into the currently open node.
    /// Every flushed element becomes exactly one leaf; the token becomes a
    /// leaf for its (part of its) element's span.
    fn bump(&mut self) {
        let Some(tok) = self.current() else {
            return;
        };
        let elements = self.lexed.elements();
        while self.next_element < tok.elem as usize {
            let element = &elements[self.next_element];
            self.events.push(Event::Leaf {
                kind: element_leaf_kind(element.kind()),
                span: element.span(),
                elem: self.next_element as u32,
                errors: element.errors().to_vec(),
            });
            self.next_element += 1;
        }
        let errors = if tok.is_first_part {
            elements
                .get(tok.elem as usize)
                .map(|element| element.errors().to_vec())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        self.events.push(Event::Leaf {
            kind: NodeKind::Token(tok.kind),
            span: tok.span,
            elem: tok.elem,
            errors,
        });
        if elements
            .get(tok.elem as usize)
            .is_some_and(|element| element.span().end() == tok.span.end())
        {
            self.next_element = tok.elem as usize + 1;
        }
        match tok.kind {
            TokenKind::LParen | TokenKind::LBracket => self.depth += 1,
            TokenKind::RParen | TokenKind::RBracket => {
                self.depth = self.depth.saturating_sub(1);
            }
            _ => {}
        }
        self.pos += 1;
    }

    /// Emits all remaining unconsumed lexer elements (trailing trivia) into
    /// the currently open node. Called after every token has been consumed.
    fn flush_trailing_elements(&mut self) {
        let elements = self.lexed.elements();
        while self.next_element < elements.len() {
            let element = &elements[self.next_element];
            self.events.push(Event::Leaf {
                kind: element_leaf_kind(element.kind()),
                span: element.span(),
                elem: self.next_element as u32,
                errors: element.errors().to_vec(),
            });
            self.next_element += 1;
        }
    }

    // --- Tree events ----------------------------------------------------

    fn start(&mut self, kind: NodeKind) {
        let hint = self
            .current()
            .map(|tok| tok.span.start())
            .unwrap_or_else(|| self.source.len());
        self.events.push(Event::Start { kind, hint });
    }

    fn finish(&mut self) {
        self.events.push(Event::Finish);
    }

    /// Starts a regenerated-node helper: `start(kind)` … `finish()` around a
    /// closure is spelled out at every call site (events must stay ordered).
    fn wrap(&mut self, kind: NodeKind, inner: impl FnOnce(&mut Self)) {
        self.start(kind);
        inner(self);
        self.finish();
    }

    // --- Diagnostics ----------------------------------------------------

    /// The span used for diagnostics that point at the current token (or at
    /// the end of input).
    fn cur_span(&self) -> Span {
        match self.current() {
            Some(tok) => tok.span,
            None => self.eof_span(),
        }
    }

    fn eof_span(&self) -> Span {
        self.source
            .span(self.source.len(), self.source.len())
            .unwrap_or_else(|_| self.source.full_span())
    }

    fn diag(&mut self, code: ParseErrorCode, span: Span, message: String, label: &'static str) {
        self.diagnostics.push(Diagnostic::error(
            code.code(),
            message,
            Label::new(span, label),
        ));
    }

    /// Reports `PARSE003` at the end of input.
    fn diag_eof(&mut self, message: String) {
        let span = self.eof_span();
        self.diag(
            ParseErrorCode::Parse003,
            span,
            format!("unexpected end of input: {message}"),
            "unexpected end of input",
        );
    }

    /// Reports `PARSE002` at the current token. A reserved-invalid word gets
    /// the specific SG-001 message (condition class `PARSE002`).
    fn diag_unexpected(&mut self, what: &str) {
        let span = self.cur_span();
        let text = self.preview_current();
        let message = match self.cur_kind() {
            Some(TokenKind::ReservedFun | TokenKind::ReservedFn | TokenKind::ReservedFunc) => {
                reserved_message(&text)
            }
            _ => format!("expected {what}, found `{text}`"),
        };
        self.diag(ParseErrorCode::Parse002, span, message, "unexpected token");
    }

    // --- Checkpoints and speculation ------------------------------------

    fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            pos: self.pos,
            events: self.events.len(),
            diagnostics: self.diagnostics.len(),
            next_element: self.next_element,
            depth: self.depth,
        }
    }

    fn rollback(&mut self, cp: Checkpoint) {
        self.pos = cp.pos;
        self.events.truncate(cp.events);
        self.diagnostics.truncate(cp.diagnostics);
        self.next_element = cp.next_element;
        self.depth = cp.depth;
    }

    /// Runs `probe` speculatively; commits its events **iff** it succeeds and
    /// adds no diagnostic, otherwise rolls everything back. Deterministic:
    /// the committed reading is a pure function of the token stream.
    fn speculate(&mut self, probe: impl FnOnce(&mut Self) -> bool) -> bool {
        let cp = self.checkpoint();
        let before = self.diagnostics.len();
        if probe(self) && self.diagnostics.len() == before {
            true
        } else {
            self.rollback(cp);
            false
        }
    }

    // --- Expectations and recovery --------------------------------------

    /// Consumes the current token when it has `kind`; otherwise reports one
    /// `PARSE002`/`PARSE003` and leaves the token in place.
    fn expect(&mut self, kind: TokenKind, what: &str) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            if self.eof() {
                self.diag_eof(format!("{what} is missing"));
            } else {
                self.diag_unexpected(what);
            }
            false
        }
    }

    /// Consumes an identifier, or reports one diagnostic.
    fn expect_ident(&mut self, what: &str) -> bool {
        self.expect(TokenKind::Identifier, what)
    }

    /// Guarantees progress in a list loop. When the last iteration consumed
    /// nothing: if it also reported nothing, report one `PARSE002`; then
    /// consume the offending token into an `Error` node (no second
    /// diagnostic).
    fn progress_guard(&mut self, before_pos: usize, before_diags: usize) -> bool {
        if self.pos > before_pos {
            return true;
        }
        if self.eof() {
            return false;
        }
        if self.diagnostics.len() == before_diags {
            let span = self.cur_span();
            let text = self.preview_current();
            let message = match self.cur_kind() {
                Some(TokenKind::ReservedFun | TokenKind::ReservedFn | TokenKind::ReservedFunc) => {
                    reserved_message(&text)
                }
                _ => format!("unexpected token `{text}`"),
            };
            self.diag(ParseErrorCode::Parse002, span, message, "unexpected token");
        }
        self.wrap(NodeKind::Error, |p| p.bump());
        true
    }

    /// Consumes a malformed region into an `Error` node with exactly one
    /// diagnostic: `PARSE002` at the first token (or `PARSE003` when the
    /// input already ended). `stop` decides where the region ends; the first
    /// token is always consumed when present.
    fn recover_region(&mut self, stop: impl Fn(&Self) -> bool, message: &str) {
        self.start(NodeKind::Error);
        if self.eof() {
            self.diag_eof(message.to_string());
        } else {
            let text = self.preview_current();
            let full = match self.cur_kind() {
                Some(TokenKind::ReservedFun | TokenKind::ReservedFn | TokenKind::ReservedFunc) => {
                    reserved_message(&text)
                }
                _ => format!("{message} (found `{text}`)"),
            };
            let span = self.cur_span();
            self.diag(ParseErrorCode::Parse002, span, full, "unexpected token");
            self.bump();
            while !self.eof() && !stop(self) {
                self.bump();
            }
        }
        self.finish();
    }

    /// Consumes the closer of a delimited construct with at most one
    /// diagnostic per construct: nothing is reported when the construct was
    /// already diagnosed since `diags_at_start`; an end of input gets
    /// `PARSE003`, any other mismatch `PARSE002`.
    fn close_token(&mut self, closer: TokenKind, what: &str, diags_at_start: usize) -> bool {
        if self.at(closer) {
            self.bump();
            return true;
        }
        if self.diagnostics.len() > diags_at_start {
            return false;
        }
        if self.eof() {
            self.diag_eof(format!("{what} is never closed"));
        } else {
            self.diag_unexpected(what);
        }
        false
    }

    /// Consumes `open … close` as a balanced region (counting only this pair)
    /// or up to the end of input. Used by refusals to preserve their
    /// delimited region. Assumes the current token is `open`.
    fn consume_balanced(&mut self, open: TokenKind, close: TokenKind) {
        let mut depth: u32 = 1;
        self.bump();
        while !self.eof() && depth > 0 {
            if self.at(open) {
                depth += 1;
            } else if self.at(close) {
                depth -= 1;
            }
            self.bump();
        }
    }

    // --- Token-class predicates -----------------------------------------

    fn is_primitive_type(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Bool
                | TokenKind::Byte
                | TokenKind::Short
                | TokenKind::Int
                | TokenKind::Long
                | TokenKind::Float
                | TokenKind::Double
                | TokenKind::Char
                | TokenKind::String
        )
    }

    fn is_reserved_invalid(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::ReservedFun | TokenKind::ReservedFn | TokenKind::ReservedFunc
        )
    }

    fn is_modifier(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Public
                | TokenKind::Private
                | TokenKind::Protected
                | TokenKind::Static
                | TokenKind::Final
                | TokenKind::Abstract
                | TokenKind::Transient
                | TokenKind::Volatile
                | TokenKind::Synchronized
                | TokenKind::Native
                | TokenKind::Default
                | TokenKind::Override
        )
    }

    fn is_type_keyword(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Class
                | TokenKind::Interface
                | TokenKind::Record
                | TokenKind::Enum
                | TokenKind::Entity
        )
    }

    fn can_start_type(kind: TokenKind) -> bool {
        kind == TokenKind::Identifier || Self::is_primitive_type(kind)
    }

    fn can_start_expression(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::IntLiteral
                | TokenKind::LongLiteral
                | TokenKind::FloatLiteral
                | TokenKind::DoubleLiteral
                | TokenKind::StringLiteral
                | TokenKind::CharLiteral
                | TokenKind::BooleanLiteral
                | TokenKind::NullLiteral
                | TokenKind::Identifier
                | TokenKind::This
                | TokenKind::Super
                | TokenKind::LParen
                | TokenKind::LBrace
                | TokenKind::Switch
                | TokenKind::If
                | TokenKind::New
                | TokenKind::Bang
                | TokenKind::Minus
                | TokenKind::PlusPlus
                | TokenKind::MinusMinus
                | TokenKind::Spawn
                | TokenKind::Await
                | TokenKind::ReservedFun
                | TokenKind::ReservedFn
                | TokenKind::ReservedFunc
        )
    }

    fn can_start_statement(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::LBrace
                | TokenKind::Return
                | TokenKind::Throw
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Do
                | TokenKind::For
                | TokenKind::Try
                | TokenKind::Switch
                | TokenKind::Var
                | TokenKind::Val
                | TokenKind::Semicolon
        ) || Self::can_start_expression(kind)
    }

    fn can_start_top_level(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Package
                | TokenKind::Import
                | TokenKind::Extern
                | TokenKind::Class
                | TokenKind::Interface
                | TokenKind::Record
                | TokenKind::Enum
                | TokenKind::Entity
                | TokenKind::At
                | TokenKind::Void
        ) || Self::can_start_type(kind)
            || Self::is_reserved_invalid(kind)
    }

    // --- Compilation unit dispatch --------------------------------------

    fn parse_file(&mut self) {
        self.start(NodeKind::File);
        while !self.eof() {
            let before_pos = self.pos;
            let before_diags = self.diagnostics.len();
            self.parse_top_level_entry();
            self.progress_guard(before_pos, before_diags);
        }
        while !self.eof() {
            self.bump();
        }
        self.flush_trailing_elements();
        self.finish();
    }
}

/// Maps a lexer element kind to the CST leaf kind.
fn element_leaf_kind(kind: ElementKind) -> NodeKind {
    match kind {
        ElementKind::Token(kind) => NodeKind::Token(kind),
        ElementKind::Bom => NodeKind::Bom,
        ElementKind::Whitespace => NodeKind::Whitespace,
        ElementKind::LineTerminator(kind) => NodeKind::LineTerminator(kind),
        ElementKind::LineComment => NodeKind::LineComment,
        ElementKind::BlockComment => NodeKind::BlockComment,
        ElementKind::Error => NodeKind::LexError,
    }
}

/// The specific SG-001 refusal message for `fun`/`fn`/`func`.
fn reserved_message(text: &str) -> String {
    format!(
        "`{text}` is reserved-invalid (SG-001): it is refused at parse entry and is never a declaration keyword"
    )
}

/// A short printable preview of a token's text for diagnostics.
fn preview(text: &str) -> String {
    let mut out = String::new();
    for (count, ch) in text.chars().enumerate() {
        if count == 12 {
            out.push('…');
            break;
        }
        if ch.is_control() {
            out.push('·');
        } else {
            out.push(ch);
        }
    }
    out
}
