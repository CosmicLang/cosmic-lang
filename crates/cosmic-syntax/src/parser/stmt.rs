//! Statement parsing: the `§3.6` productions, switch labels and patterns.
//!
//! The dispatch follows the accepted production order (block, `return`,
//! `throw`, `break`/`continue`, `if`, `while`, `do`, `for`, `try`, `switch`,
//! `var`/`val`, nested function declaration, expression statement, `;`), with
//! the contextual-keyword lookaheads of the grammar. Labelled
//! `break`/`continue` is the refusal of family 8: the keyword and its
//! same-line label are preserved in a generic `Error` node with one
//! `PARSE004` diagnostic.

use cosmic_lexer::TokenKind;

use crate::codes::ParseErrorCode;
use crate::kind::NodeKind;

use super::Parser;

impl Parser<'_> {
    /// Parses statements until `stop` holds or the input ends. Every
    /// iteration guarantees progress; a statement that fails without
    /// consuming or diagnosing gets one `PARSE002` and its first token
    /// preserved in an `Error` node.
    pub(super) fn parse_statement_list(&mut self, stop: impl Fn(&Self) -> bool) {
        while !self.eof() && !stop(self) {
            let before = self.pos;
            let before_diags = self.diagnostics.len();
            self.parse_statement();
            self.progress_guard(before, before_diags);
        }
    }

    /// `block = "{" { statement } "}"`, with at most one diagnostic for the
    /// whole construct.
    pub(super) fn parse_block(&mut self) {
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::Block);
        if self.expect(TokenKind::LBrace, "`{` to open the block") {
            self.parse_statement_list(|p| p.at(TokenKind::RBrace));
        }
        self.close_token(TokenKind::RBrace, "`}` closing the block", diags_at_start);
        self.finish();
    }

    fn parse_statement(&mut self) {
        match self.cur_kind() {
            Some(TokenKind::LBrace) => self.parse_block(),
            Some(TokenKind::Return) => self.parse_return(),
            Some(TokenKind::Throw) => self.parse_throw(),
            Some(TokenKind::Break | TokenKind::Continue) => self.parse_jump(),
            Some(TokenKind::If) => self.parse_if_stmt(),
            Some(TokenKind::While) => self.parse_while_stmt(),
            Some(TokenKind::Do) => self.parse_do_while_stmt(),
            Some(TokenKind::For) => self.parse_for_stmt(),
            Some(TokenKind::Try) => self.parse_try_stmt(),
            Some(TokenKind::Switch) => self.parse_switch_stmt(),
            Some(TokenKind::Var | TokenKind::Val) => self.parse_var_decl_stmt(),
            Some(TokenKind::Semicolon) => self.parse_empty_stmt(),
            // `nested-function-decl | expression-stmt` for a leading name or
            // type: the function declaration is tried first (grammar order),
            // committed only when its body exists on the same line — a `{`
            // on a later line is a following block statement, not a body.
            Some(kind)
                if kind == TokenKind::Identifier
                    || kind == TokenKind::Void
                    || Self::is_primitive_type(kind) =>
            {
                if !self.try_function_decl(true) {
                    if Self::can_start_expression(kind) {
                        self.parse_expression_statement();
                    } else {
                        self.recover_statement_region();
                    }
                }
            }
            Some(kind) if Self::can_start_expression(kind) => self.parse_expression_statement(),
            // The caller's list owns these tokens (`}` closes the block,
            // `case`/`default` end the arm); a statement cannot start here.
            Some(TokenKind::RBrace | TokenKind::Case | TokenKind::Default) => {
                self.diag_unexpected("a statement");
            }
            _ => self.recover_statement_region(),
        }
    }

    fn parse_return(&mut self) {
        self.start(NodeKind::ReturnStmt);
        self.bump();
        // `return [ expression ]`: an expression is taken only when it
        // starts on the same line (statement separation); `return` alone and
        // `return` at the end of the line are complete.
        if !self.eof()
            && self.same_line()
            && self.cur_kind().is_some_and(Self::can_start_expression)
        {
            self.parse_expr();
        }
        self.finish();
    }

    fn parse_throw(&mut self) {
        self.start(NodeKind::ThrowStmt);
        self.bump();
        self.parse_expr_required("an expression after `throw`");
        self.finish();
    }

    fn parse_jump(&mut self) {
        let is_break = self.at(TokenKind::Break);
        let word = if is_break { "break" } else { "continue" };
        // Family 8: a same-line identifier after the keyword is a label.
        let labelled = self
            .nth_tok(1)
            .is_some_and(|tok| tok.kind == TokenKind::Identifier && !tok.preceded_by_lt);
        if labelled {
            let span = self
                .nth_tok(1)
                .map(|tok| tok.span)
                .unwrap_or_else(|| self.cur_span());
            self.start(NodeKind::Error);
            self.bump();
            self.diag(
                ParseErrorCode::Parse004,
                span,
                format!(
                    "labelled `{word}` is not supported: the accepted grammar derives unlabelled forms only (S5)"
                ),
                "label",
            );
            self.bump();
            self.finish();
        } else {
            self.start(if is_break {
                NodeKind::BreakStmt
            } else {
                NodeKind::ContinueStmt
            });
            self.bump();
            self.finish();
        }
    }

    fn parse_if_stmt(&mut self) {
        self.start(NodeKind::IfStmt);
        self.bump();
        self.expect(TokenKind::LParen, "`(` after `if`");
        self.parse_expr_required("the `if` condition");
        self.expect(TokenKind::RParen, "`)` after the `if` condition");
        self.parse_statement();
        if self.at(TokenKind::Else) {
            self.bump();
            self.parse_statement();
        }
        self.finish();
    }

    fn parse_while_stmt(&mut self) {
        self.start(NodeKind::WhileStmt);
        self.bump();
        self.expect(TokenKind::LParen, "`(` after `while`");
        self.parse_expr_required("the `while` condition");
        self.expect(TokenKind::RParen, "`)` after the `while` condition");
        self.parse_statement();
        self.finish();
    }

    fn parse_do_while_stmt(&mut self) {
        self.start(NodeKind::DoWhileStmt);
        self.bump();
        self.parse_statement();
        if self.at(TokenKind::While) {
            self.bump();
            self.expect(TokenKind::LParen, "`(` after `while`");
            self.parse_expr_required("the `do`-`while` condition");
            self.expect(TokenKind::RParen, "`)` after the `do`-`while` condition");
        } else {
            self.diag_unexpected("`while` after the `do` body");
        }
        self.finish();
    }

    /// Whether the current `for` header is the `for (… in …)` form
    /// (contextual `in`, contextual lookahead per grammar §3.2).
    fn for_in_ahead(&self) -> bool {
        if self.at(TokenKind::Var) || self.at(TokenKind::Val) {
            self.nth(1) == Some(TokenKind::Identifier) && self.nth_is_contextual(2, "in")
        } else if self.at(TokenKind::Identifier) {
            self.nth_is_contextual(1, "in")
        } else {
            false
        }
    }

    fn parse_for_stmt(&mut self) {
        self.start(NodeKind::ForStmt);
        self.bump();
        self.expect(TokenKind::LParen, "`(` after `for`");
        if self.for_in_ahead() {
            if self.at(TokenKind::Var) || self.at(TokenKind::Val) {
                self.bump();
            }
            self.expect_ident("the `for … in` loop variable name");
            if self.nth_is_contextual(0, "in") {
                self.bump();
            } else {
                self.diag_unexpected("`in`");
            }
            self.parse_expr_required("the iterated expression");
            self.expect(TokenKind::RParen, "`)` after the `for … in` header");
            self.parse_statement();
            self.finish();
            return;
        }
        // Three-clause form (F3, executed by `loops-control`).
        if !self.at(TokenKind::Semicolon) {
            self.start(NodeKind::ForInit);
            if self.at(TokenKind::Var) || self.at(TokenKind::Val) {
                self.bump();
                self.expect_ident("the binding name");
                if self.at(TokenKind::Colon) {
                    self.bump();
                    self.parse_type_ref();
                }
                if self.expect(TokenKind::Eq, "`=` in the `for` initialiser") {
                    self.parse_statement_expr_required("the initialiser expression");
                }
            } else {
                self.parse_statement_expr_required("the `for` initialiser expression");
            }
            self.finish();
        }
        self.expect(TokenKind::Semicolon, "`;` after the `for` initialiser");
        if !self.at(TokenKind::Semicolon) {
            self.parse_expr_required("the `for` condition expression");
        }
        self.expect(TokenKind::Semicolon, "`;` after the `for` condition");
        if !self.at(TokenKind::RParen) {
            // The executed three-clause form assigns in the step slot
            // (`i = i + 1`; `loops-control`), so the slot is parsed like a
            // statement position.
            self.parse_statement_expr_required("the `for` step expression");
        }
        self.expect(TokenKind::RParen, "`)` after the `for` header");
        self.parse_statement();
        self.finish();
    }

    fn parse_try_stmt(&mut self) {
        self.start(NodeKind::TryStmt);
        self.bump();
        self.parse_block();
        while self.at(TokenKind::Catch) {
            self.parse_catch_clause();
        }
        if self.at(TokenKind::Finally) {
            self.parse_finally_clause();
        }
        self.finish();
    }

    fn parse_catch_clause(&mut self) {
        self.start(NodeKind::CatchClause);
        self.bump();
        self.expect(TokenKind::LParen, "`(` after `catch`");
        self.parse_type_ref();
        self.expect_ident("the `catch` binding name");
        self.expect(TokenKind::RParen, "`)` after the `catch` parameter");
        self.parse_block();
        self.finish();
    }

    fn parse_finally_clause(&mut self) {
        self.start(NodeKind::FinallyClause);
        self.bump();
        self.parse_block();
        self.finish();
    }

    fn parse_switch_stmt(&mut self) {
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::SwitchStmt);
        self.bump();
        self.expect(TokenKind::LParen, "`(` after `switch`");
        self.parse_expr_required("the switch subject");
        self.expect(TokenKind::RParen, "`)` after the switch subject");
        if !self.expect(TokenKind::LBrace, "`{` to open the switch body") {
            self.finish();
            return;
        }
        while !self.eof() && !self.at(TokenKind::RBrace) {
            let before = self.pos;
            let before_diags = self.diagnostics.len();
            if self.at(TokenKind::Case) || self.at(TokenKind::Default) {
                self.parse_switch_stmt_arm();
            } else {
                self.recover_region(
                    |p| {
                        p.at(TokenKind::Case)
                            || p.at(TokenKind::Default)
                            || p.at(TokenKind::RBrace)
                    },
                    "unexpected token in the switch body (statement arms start with `case` or `default`)",
                );
            }
            self.progress_guard(before, before_diags);
        }
        self.close_token(
            TokenKind::RBrace,
            "`}` closing the switch body",
            diags_at_start,
        );
        self.finish();
    }

    /// `switch-stmt-arm = "case" switch-label [guard] ":" {statement} |
    /// "default" ":" {statement}` (colon arms, F2).
    fn parse_switch_stmt_arm(&mut self) {
        self.start(NodeKind::SwitchStmtArm);
        if self.at(TokenKind::Case) {
            self.bump();
            self.parse_switch_label();
            self.parse_guard_opt();
            self.expect(TokenKind::Colon, "`:` in a switch statement arm");
        } else {
            self.bump();
            self.expect(TokenKind::Colon, "`:` after `default`");
        }
        self.parse_statement_list(|p| {
            p.at(TokenKind::Case) || p.at(TokenKind::Default) || p.at(TokenKind::RBrace)
        });
        self.finish();
    }

    /// The accepted `guard` production: `if expression` on a `case` arm.
    pub(super) fn parse_guard_opt(&mut self) {
        if self.at(TokenKind::If) {
            self.start(NodeKind::Guard);
            self.bump();
            self.parse_expr_required("the guard condition expression");
            self.finish();
        }
    }

    /// `switch-label = expression | type-ref | binding-pattern |
    /// destructuring-pattern` (grammar §3.6; disambiguation is semantic —
    /// S5 — so recognition follows the production order with bounded
    /// speculation).
    pub(super) fn parse_switch_label(&mut self) {
        let is_identifier = self.cur_kind() == Some(TokenKind::Identifier);
        if is_identifier && self.nth(1) == Some(TokenKind::Identifier) {
            self.start(NodeKind::BindingPattern);
            self.bump();
            self.bump();
            self.finish();
            return;
        }
        if is_identifier && self.nth(1) == Some(TokenKind::LParen) {
            let ok = self.speculate(|p| {
                p.parse_destructuring_pattern();
                p.at_label_end()
            });
            if ok {
                return;
            }
        }
        if self.cur_kind().is_some_and(Self::is_primitive_type) {
            self.parse_type_ref();
            return;
        }
        // `expression` first (production order); the `type-ref` reading is
        // reachable only when the expression reading leaves the label
        // unfinished (for example `List<Int>`, whose `<` is not a call).
        let ok = self.speculate(|p| {
            p.parse_expr();
            p.at_label_end()
        });
        if !ok {
            let typed = self.speculate(|p| {
                p.parse_type_ref();
                p.at_label_end()
            });
            if !typed {
                self.parse_expr_required("a switch label");
            }
        }
    }

    fn at_label_end(&self) -> bool {
        self.at(TokenKind::Colon) || self.at(TokenKind::Arrow) || self.at(TokenKind::If)
    }

    /// `destructuring-pattern = identifier "(" pattern-field { "," pattern-field } ")"` (F5).
    fn parse_destructuring_pattern(&mut self) {
        self.start(NodeKind::DestructuringPattern);
        self.bump();
        if self.expect(TokenKind::LParen, "`(` after the pattern name") {
            loop {
                let before = self.pos;
                let before_diags = self.diagnostics.len();
                self.parse_pattern_field();
                if self.pos == before {
                    self.progress_guard(before, before_diags);
                    break;
                }
                if self.at(TokenKind::Comma) {
                    self.bump();
                    continue;
                }
                break;
            }
            self.expect(TokenKind::RParen, "`)` closing the destructuring pattern");
        }
        self.finish();
    }

    /// `pattern-field = ( "var" | "val" ) identifier` (F5).
    fn parse_pattern_field(&mut self) {
        self.start(NodeKind::PatternField);
        if self.at(TokenKind::Var) || self.at(TokenKind::Val) {
            self.bump();
            self.expect_ident("the field binding name");
        } else {
            self.diag_unexpected("`var` or `val` in a destructuring pattern field");
        }
        self.finish();
    }

    fn parse_var_decl_stmt(&mut self) {
        self.start(NodeKind::VarDeclStmt);
        self.bump();
        self.expect_ident("the binding name");
        if self.at(TokenKind::Colon) {
            self.bump();
            self.parse_type_ref();
        }
        if self.expect(TokenKind::Eq, "`=` in the `var`/`val` declaration") {
            self.parse_statement_expr_required("the declaration's initialiser expression");
        }
        self.finish();
    }

    fn parse_expression_statement(&mut self) {
        self.start(NodeKind::ExprStmt);
        if self.cur_kind().is_some_and(Self::can_start_expression) {
            self.parse_statement_expr();
        }
        self.finish();
    }

    fn parse_empty_stmt(&mut self) {
        self.start(NodeKind::EmptyStmt);
        self.bump();
        self.finish();
    }

    /// The fallback for a token that cannot start a statement: one
    /// `PARSE002` (or the SG-001 message for a reserved word) and the region
    /// up to the next statement boundary preserved in an `Error` node.
    fn recover_statement_region(&mut self) {
        self.recover_region(
            |p| {
                p.at(TokenKind::RBrace)
                    || p.at(TokenKind::Case)
                    || p.at(TokenKind::Default)
                    || (p.preceded_by_lt() && p.cur_kind().is_some_and(Self::can_start_statement))
            },
            "unexpected token at the start of a statement",
        );
    }
}
