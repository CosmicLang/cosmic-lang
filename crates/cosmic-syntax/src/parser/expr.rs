//! Expression parsing: the `§3.7` productions of the accepted grammar.
//!
//! Precedence climbing over the accepted `binary-op` table (levels 1–8,
//! all left-associative), the line gate for continuations, postfix
//! operations (calls, member access, indexing, `++`/`--`, generic calls and
//! the trailing-lambda same-line rule) and the primary forms (literals,
//! identifiers, parenthesised expressions, both lambda forms, `switch`,
//! `if`, `new` and the refused `{ … }` brace form).
//!
//! Assignment is parsed at statement level only
//! ([`Parser::parse_statement_expr`]): the accepted note to
//! `grammar.ebnf` §3.7 records that `assignment-expr` is reachable only from
//! a statement position (`SEM027` gates chains downstream). In expression
//! contexts (arguments, operands, defaults) `=` is not consumed.

use cosmic_lexer::TokenKind;

use crate::codes::ParseErrorCode;
use crate::kind::NodeKind;
use crate::tree::Event;

use super::Parser;

/// The token shape an operator consumes (a `>>`/`>>>` is consumed as its
/// pre-split `>` parts).
#[derive(Clone, Copy)]
enum OpShape {
    Single,
    Shift2,
    Shift3,
}

/// One binary operator occurrence: precedence level (1 lowest, 8 highest),
/// token shape and whether its right-hand side is a `type-ref` (only `as`
/// and `instanceof`; grammar §3.7).
#[derive(Clone, Copy)]
struct BinOp {
    level: u8,
    shape: OpShape,
    type_rhs: bool,
}

fn is_assign_op(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Eq
            | TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::PercentEq
            | TokenKind::AmpEq
            | TokenKind::PipeEq
            | TokenKind::CaretEq
            | TokenKind::ShlEq
            | TokenKind::ShrEq
            | TokenKind::UshrEq
    )
}

impl Parser<'_> {
    /// `expression` (binary level 0, no assignment).
    pub(super) fn parse_expr(&mut self) {
        self.parse_binary(0);
    }

    /// `expression`, reporting exactly one diagnostic when no expression can
    /// start at the current position.
    pub(super) fn parse_expr_required(&mut self, what: &str) {
        if self.cur_kind().is_some_and(Self::can_start_expression) {
            self.parse_expr();
        } else {
            self.missing_expression(what);
        }
    }

    /// `assignment-expr` in a statement position, reporting exactly one
    /// diagnostic when no expression can start at the current position.
    pub(super) fn parse_statement_expr_required(&mut self, what: &str) {
        if self.cur_kind().is_some_and(Self::can_start_expression) {
            self.parse_statement_expr();
        } else {
            self.missing_expression(what);
        }
    }

    /// A required expression is missing: one diagnostic, and the offending
    /// token is preserved in an `Error` node when it cannot belong to an
    /// enclosing construct. Closers (`)`, `]`, `}`), separators (`;`, `,`,
    /// `:`), the arm arrow, the end of input and the start of the next
    /// statement (a token on a new line that can start a statement) are left
    /// for the enclosing construct, so one defect yields one diagnostic and
    /// no enclosing closer is swallowed.
    fn missing_expression(&mut self, what: &str) {
        match self.cur_kind() {
            None => {
                self.diag_eof(format!("{what} is missing"));
                return;
            }
            Some(
                TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::Semicolon
                | TokenKind::Comma
                | TokenKind::Colon
                | TokenKind::Arrow,
            ) => {
                self.diag_unexpected(what);
                return;
            }
            _ => {}
        }
        if self.preceded_by_lt() && self.cur_kind().is_some_and(Self::can_start_statement) {
            // The next statement starts here; it is not part of this defect.
            self.diag_unexpected(what);
            return;
        }
        self.diag_unexpected(what);
        self.wrap(NodeKind::Error, |parser| parser.bump());
    }

    /// `assignment-expr` in a statement position: `binary-expr` with an
    /// optional `assignment-op assignment-expr` tail. Used by expression
    /// statements, `var`/`val` right-hand sides and `for`-header slots.
    pub(super) fn parse_statement_expr(&mut self) {
        if !self.enter_nesting() {
            self.over_nesting_limit();
            return;
        }
        let left_start = self.events.len();
        let hint = self
            .current()
            .map(|tok| tok.span.start())
            .unwrap_or_else(|| self.source.len());
        self.parse_binary(0);
        while let Some(kind) = self.cur_kind() {
            if !is_assign_op(kind) || !self.line_ok() {
                break;
            }
            self.events.insert(
                left_start,
                Event::Start {
                    kind: NodeKind::AssignExpr,
                    hint,
                },
            );
            self.bump();
            if self.preceded_by_lt() && self.depth == 0 {
                // The right-hand side does not continue across a line
                // terminator (statement separation).
                self.diag_unexpected("the right-hand side of the assignment");
            } else {
                self.parse_statement_expr_required("the right-hand side of the assignment");
            }
            self.finish();
        }
        self.exit_nesting();
    }

    fn parse_binary(&mut self, min_level: u8) {
        if !self.enter_nesting() {
            self.over_nesting_limit();
            return;
        }
        let left_start = self.events.len();
        let hint = self
            .current()
            .map(|tok| tok.span.start())
            .unwrap_or_else(|| self.source.len());
        self.parse_unary();
        while let Some(op) = self.peek_binary_op() {
            if op.level < min_level {
                break;
            }
            // Statement separation: a continuation operator on a new line is
            // not taken (outside `( … )` / `[ … ]` regions).
            if self.preceded_by_lt() && self.depth == 0 {
                break;
            }
            self.events.insert(
                left_start,
                Event::Start {
                    kind: NodeKind::BinaryExpr,
                    hint,
                },
            );
            self.consume_binary_op(op.shape);
            if op.type_rhs {
                self.parse_type_ref();
            } else if self.cur_kind().is_some_and(Self::can_start_expression) {
                self.parse_binary(op.level + 1);
            } else {
                // The operator has no right-hand operand: report once, so a
                // half binary expression can never be committed silently
                // (this is what makes a switch-label type reference such as
                // `case List<Int>:` fail expression speculation and fall
                // through to the type-reference reading).
                self.missing_expression("the right-hand operand of the operator");
            }
            self.finish();
        }
        self.exit_nesting();
    }

    fn peek_binary_op(&self) -> Option<BinOp> {
        let kind = self.cur_kind()?;
        let single = |level: u8| BinOp {
            level,
            shape: OpShape::Single,
            type_rhs: false,
        };
        let op = match kind {
            TokenKind::PipePipe => single(1),
            TokenKind::AmpAmp => single(2),
            TokenKind::Pipe | TokenKind::Caret => single(3),
            TokenKind::Amp => single(4),
            TokenKind::EqEq
            | TokenKind::BangEq
            | TokenKind::LtEq
            | TokenKind::GtEq
            | TokenKind::Lt => single(5),
            TokenKind::Gt => {
                if self.next_gt_part() {
                    // `>>` or `>>>` re-joined from the pre-split parts; the
                    // third part belongs to the same element and is adjacent.
                    let three = match (self.nth_tok(1), self.nth_tok(2)) {
                        (Some(b), Some(c)) => {
                            b.kind == TokenKind::Gt
                                && c.kind == TokenKind::Gt
                                && b.elem == c.elem
                                && b.span.end() == c.span.start()
                                && !c.preceded_by_lt
                        }
                        _ => false,
                    };
                    BinOp {
                        level: 6,
                        shape: if three {
                            OpShape::Shift3
                        } else {
                            OpShape::Shift2
                        },
                        type_rhs: false,
                    }
                } else {
                    single(5)
                }
            }
            TokenKind::Instanceof | TokenKind::As => BinOp {
                level: 5,
                shape: OpShape::Single,
                type_rhs: true,
            },
            // `Shl` is a single token; `Shr`/`Ushr` are pre-split and appear
            // only as `Gt` parts, but are handled defensively.
            TokenKind::Shl | TokenKind::Shr | TokenKind::Ushr => single(6),
            TokenKind::Plus | TokenKind::Minus => single(7),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => single(8),
            _ => return None,
        };
        Some(op)
    }

    fn consume_binary_op(&mut self, shape: OpShape) {
        match shape {
            OpShape::Single => self.bump(),
            OpShape::Shift2 => {
                self.bump();
                self.bump();
            }
            OpShape::Shift3 => {
                self.bump();
                self.bump();
                self.bump();
            }
        }
    }

    // --- Unary, postfix, primary ----------------------------------------

    fn parse_unary(&mut self) {
        if !self.enter_nesting() {
            self.over_nesting_limit();
            return;
        }
        if matches!(
            self.cur_kind(),
            Some(
                TokenKind::Bang
                    | TokenKind::Minus
                    | TokenKind::PlusPlus
                    | TokenKind::MinusMinus
                    | TokenKind::Spawn
                    | TokenKind::Await
            )
        ) {
            self.start(NodeKind::UnaryExpr);
            self.bump();
            self.parse_unary();
            self.finish();
        } else {
            self.parse_postfix();
        }
        self.exit_nesting();
    }

    fn parse_postfix(&mut self) {
        if !self.cur_kind().is_some_and(Self::can_start_expression) {
            return;
        }
        self.start(NodeKind::PostfixExpr);
        self.parse_primary();
        while let Some(kind) = self.cur_kind() {
            match kind {
                TokenKind::LParen if self.line_ok() => self.parse_call_arguments(),
                TokenKind::Dot if self.line_ok() => {
                    self.bump();
                    self.expect_ident("a member name after `.`");
                    if self.at(TokenKind::LParen) && self.line_ok() {
                        self.parse_call_arguments();
                    }
                }
                TokenKind::LBracket if self.line_ok() => {
                    let diags_at_start = self.diagnostics.len();
                    self.bump();
                    self.parse_expr_required("an index expression");
                    self.close_token(
                        TokenKind::RBracket,
                        "`]` closing the index expression",
                        diags_at_start,
                    );
                }
                TokenKind::PlusPlus | TokenKind::MinusMinus if self.line_ok() => self.bump(),
                TokenKind::Lt if self.line_ok() => {
                    // Generic call: `type-arguments "(" [argument-list] ")"`.
                    // Never a fallback for `<` (grammar §3.7, G1-S §5.2): a
                    // failed attempt rolls back and the `<` stays a
                    // comparison for the binary parser.
                    let ok = self.speculate(|p| {
                        p.parse_type_arguments();
                        if !p.at(TokenKind::LParen) {
                            return false;
                        }
                        p.parse_call_arguments();
                        true
                    });
                    if !ok {
                        break;
                    }
                }
                // Trailing-lambda same-line rule (grammar §6.1 note): only a
                // same-line `{` that starts a lambda attaches; otherwise the
                // `{` belongs to the enclosing context.
                TokenKind::LBrace if self.same_line() => {
                    if self.brace_lambda_ahead() {
                        self.parse_brace_lambda();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        self.finish();
    }

    fn parse_primary(&mut self) {
        let Some(kind) = self.cur_kind() else { return };
        match kind {
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
            | TokenKind::Super => self.bump(),
            TokenKind::LParen => {
                if self.paren_is_lambda() {
                    self.parse_paren_lambda();
                } else {
                    self.parse_paren_expr();
                }
            }
            TokenKind::LBrace => {
                if self.brace_lambda_ahead() {
                    self.parse_brace_lambda();
                } else {
                    self.refuse_array_literal();
                }
            }
            TokenKind::Switch => self.parse_switch_expr(),
            TokenKind::If => self.parse_if_expr(),
            TokenKind::New => self.parse_new_expr(),
            TokenKind::ReservedFun | TokenKind::ReservedFn | TokenKind::ReservedFunc => {
                let span = self.cur_span();
                let text = self.preview_current();
                self.start(NodeKind::Error);
                self.diag(
                    ParseErrorCode::Parse002,
                    span,
                    super::reserved_message(&text),
                    "reserved word",
                );
                self.bump();
                self.finish();
            }
            _ => {}
        }
    }

    /// `"(" [ argument-list ] ")"` of a call: one `ArgumentList` node.
    pub(super) fn parse_call_arguments(&mut self) {
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::ArgumentList);
        if !self.expect(TokenKind::LParen, "`(`") {
            self.finish();
            return;
        }
        loop {
            if self.eof() || self.at(TokenKind::RParen) {
                break;
            }
            let before = self.pos;
            let before_diags = self.diagnostics.len();
            self.parse_expr_required("an argument expression");
            if self.pos == before {
                // No argument consumed: its one diagnostic was reported by
                // `parse_expr_required`; preserve the token and stop.
                self.progress_guard(before, before_diags);
                break;
            }
            if self.at(TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        self.close_token(
            TokenKind::RParen,
            "`)` closing the argument list",
            diags_at_start,
        );
        self.finish();
    }

    /// Whether the `(` at the current position starts a lambda: the matching
    /// `)` is followed by `->` (G1-S §5.1, item 2). The scan is deterministic
    /// and bounded by the token stream.
    fn paren_is_lambda(&self) -> bool {
        let mut depth: i32 = 0;
        for index in self.pos..self.tokens.len() {
            match self.tokens[index].kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return self
                            .tokens
                            .get(index + 1)
                            .is_some_and(|tok| tok.kind == TokenKind::Arrow);
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn parse_paren_expr(&mut self) {
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::ParenExpr);
        self.bump();
        self.parse_expr_required("an expression inside the parentheses");
        self.close_token(
            TokenKind::RParen,
            "`)` closing the parenthesised expression",
            diags_at_start,
        );
        self.finish();
    }

    fn parse_paren_lambda(&mut self) {
        self.start(NodeKind::LambdaExpr);
        self.parse_parameter_list();
        if self.at(TokenKind::Arrow) {
            self.bump();
        } else {
            self.diag_unexpected("`->` after the lambda parameters");
        }
        if self.at(TokenKind::LBrace) {
            self.parse_block();
        } else {
            self.parse_expr_required("the lambda body expression");
        }
        self.finish();
    }

    /// Whether the `{` at the current position starts a lambda
    /// (`"{" [ lambda-params ] "->" …`). Pure lookahead with full rollback;
    /// a parameter list that is not clean (for example an untyped parameter,
    /// which is not derivable, S1) does not start a lambda.
    fn brace_lambda_ahead(&mut self) -> bool {
        if !self.at(TokenKind::LBrace) {
            return false;
        }
        let cp = self.checkpoint();
        let before = self.diagnostics.len();
        self.bump();
        let ok = if self.at(TokenKind::Arrow) {
            true
        } else if self.cur_kind().is_some_and(Self::can_start_type) {
            self.parse_lambda_params();
            self.at(TokenKind::Arrow) && self.diagnostics.len() == before
        } else {
            false
        };
        self.rollback(cp);
        ok
    }

    /// The bare parameter list of a brace lambda: `parameter { "," parameter }`
    /// wrapped in one `LambdaParams` node.
    fn parse_lambda_params(&mut self) -> bool {
        self.start(NodeKind::LambdaParams);
        let mut any = false;
        loop {
            let before = self.pos;
            let before_diags = self.diagnostics.len();
            self.parse_parameter();
            if self.pos > before {
                any = true;
            } else {
                self.progress_guard(before, before_diags);
                break;
            }
            if self.at(TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        self.finish();
        any
    }

    /// `"{" [ lambda-params ] "->" { statement } "}"`. The `LambdaExpr` node
    /// itself covers the braces; statements are its children.
    fn parse_brace_lambda(&mut self) {
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::LambdaExpr);
        self.bump();
        if self.at(TokenKind::Arrow) {
            self.bump();
        } else {
            if self.cur_kind().is_some_and(Self::can_start_type) {
                self.parse_lambda_params();
            }
            self.expect(TokenKind::Arrow, "`->` after the lambda parameters");
        }
        self.parse_statement_list(|p| p.at(TokenKind::RBrace));
        self.close_token(TokenKind::RBrace, "`}` closing the lambda", diags_at_start);
        self.finish();
    }

    fn parse_if_expr(&mut self) {
        self.start(NodeKind::IfExpr);
        self.bump();
        self.expect(TokenKind::LParen, "`(` after `if`");
        self.parse_expr_required("the `if` condition");
        self.expect(TokenKind::RParen, "`)` after the `if` condition");
        self.parse_expr_required("the `if` then-branch expression");
        if self.at(TokenKind::Else) {
            self.bump();
            self.parse_expr_required("the `else` branch expression");
        }
        self.finish();
    }

    fn parse_new_expr(&mut self) {
        self.start(NodeKind::NewExpr);
        self.bump();
        self.parse_type_ref();
        if self.at(TokenKind::LParen) {
            self.parse_call_arguments();
        } else {
            self.diag_unexpected("`(` after the type in a `new` expression");
        }
        self.finish();
    }

    fn parse_switch_expr(&mut self) {
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::SwitchExpr);
        self.bump();
        self.expect(TokenKind::LParen, "`(` after `switch`");
        self.parse_expr_required("the switch expression subject");
        self.expect(TokenKind::RParen, "`)` after the switch expression subject");
        if !self.expect(TokenKind::LBrace, "`{` to open the switch expression body") {
            self.finish();
            return;
        }
        while !self.eof() && !self.at(TokenKind::RBrace) {
            let before = self.pos;
            let before_diags = self.diagnostics.len();
            if self.at(TokenKind::Case) || self.at(TokenKind::Default) {
                self.parse_switch_expr_arm();
            } else {
                self.recover_region(
                    |p| {
                        p.at(TokenKind::Case) || p.at(TokenKind::Default) || p.at(TokenKind::RBrace)
                    },
                    "unexpected token in the switch expression body",
                );
            }
            self.progress_guard(before, before_diags);
        }
        self.close_token(
            TokenKind::RBrace,
            "`}` closing the switch expression body",
            diags_at_start,
        );
        self.finish();
    }

    fn parse_switch_expr_arm(&mut self) {
        self.start(NodeKind::SwitchExprArm);
        if self.at(TokenKind::Case) {
            self.bump();
            self.parse_switch_label();
            self.parse_guard_opt();
        } else {
            // `default` arms have no guard (F2; shared with the statement
            // form's `default :`).
            self.bump();
        }
        self.expect(TokenKind::Arrow, "`->` in a switch expression arm");
        self.parse_expr_required("the switch arm expression");
        self.finish();
    }

    /// Refused family 9: the brace form in expression position.
    fn refuse_array_literal(&mut self) {
        let span = self.cur_span();
        self.start(NodeKind::ArrayLiteral);
        self.diag(
            ParseErrorCode::Parse001,
            span,
            "unsupported syntax family: the brace array-literal form is a placeholder with no evidence and is not implemented (S9)"
                .to_string(),
            "refused here",
        );
        self.consume_balanced(TokenKind::LBrace, TokenKind::RBrace);
        self.finish();
    }
}
