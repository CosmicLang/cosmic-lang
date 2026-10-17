//! Declaration parsing: the compilation unit (`§3.2`), functions (`§3.3`),
//! types (`§3.4`–`§3.5`), members and the refusal entry points of the nine
//! unsupported families (`docs/spec/g1-s-syntax-decision-package.md` §4).
//!
//! Each refusal recognises its construct precisely enough to name it,
//! reports exactly one `PARSE001` diagnostic naming the family and its
//! decision record, consumes the delimited region when it can be delimited
//! and preserves it in a refusal node. Refusal nodes are error nodes and
//! carry no semantics.

use cosmic_lexer::TokenKind;

use crate::codes::ParseErrorCode;
use crate::kind::NodeKind;

use super::Parser;

impl Parser<'_> {
    // --- Compilation unit -------------------------------------------------

    /// One entry of the compilation unit: `package`/`import` directly, or a
    /// `top-level-decl` (optional annotations plus a top-level body).
    pub(super) fn parse_top_level_entry(&mut self) {
        if self.at(TokenKind::Package) {
            self.parse_package_decl();
            return;
        }
        if self.at(TokenKind::Import) {
            self.parse_import_decl();
            return;
        }
        self.start(NodeKind::TopLevelDecl);
        self.parse_annotations();
        self.parse_top_level_body();
        self.finish();
    }

    fn parse_package_decl(&mut self) {
        self.start(NodeKind::PackageDecl);
        self.bump();
        self.parse_qualified_name("a name after `package`");
        self.finish();
    }

    fn parse_import_decl(&mut self) {
        self.start(NodeKind::ImportDecl);
        self.bump();
        self.parse_qualified_name("a name after `import`");
        self.finish();
    }

    /// `qualified-name = identifier { "." identifier }`.
    fn parse_qualified_name(&mut self, what: &str) -> bool {
        if self.cur_kind() != Some(TokenKind::Identifier) {
            self.diag_unexpected(what);
            return false;
        }
        self.start(NodeKind::QualifiedName);
        self.bump();
        while self.at(TokenKind::Dot) && self.nth(1) == Some(TokenKind::Identifier) {
            self.bump();
            self.bump();
        }
        self.finish();
        true
    }

    /// Consumes `{ annotation }` prefixes. Annotations are refused (family
    /// 7): one `PARSE001` per annotation, then parsing continues with the
    /// annotated declaration.
    pub(super) fn parse_annotations(&mut self) {
        while self.at(TokenKind::At) {
            self.parse_annotation();
        }
    }

    fn parse_annotation(&mut self) {
        let span = self.cur_span();
        self.start(NodeKind::Annotation);
        self.diag(
            ParseErrorCode::Parse001,
            span,
            "unsupported syntax family: annotations are refused while their name vocabulary is unknown (S10)"
                .to_string(),
            "refused here",
        );
        self.bump();
        if self.at(TokenKind::Identifier) {
            self.bump();
            while self.at(TokenKind::Dot) && self.nth(1) == Some(TokenKind::Identifier) {
                self.bump();
                self.bump();
            }
        }
        if self.at(TokenKind::LParen) {
            self.consume_balanced(TokenKind::LParen, TokenKind::RParen);
        }
        self.finish();
    }

    fn parse_top_level_body(&mut self) {
        // Contextual keyword lookaheads (grammar §3.2): `test`/`infra` need
        // a string literal, `application` a `{`, `foreign` a `module`.
        if self.is_contextual("test") && self.nth(1) == Some(TokenKind::StringLiteral) {
            self.refuse_test_decl();
            return;
        }
        if self.is_contextual("application") && self.nth(1) == Some(TokenKind::LBrace) {
            self.refuse_application_decl();
            return;
        }
        if self.is_contextual("infra") && self.nth(1) == Some(TokenKind::StringLiteral) {
            self.refuse_infra_decl();
            return;
        }
        if self.at(TokenKind::Extern) {
            self.refuse_extern_decl();
            return;
        }
        if self.is_contextual("foreign") && self.nth_is_contextual(1, "module") {
            self.refuse_foreign_module_decl();
            return;
        }
        if self.at(TokenKind::Entity) {
            self.refuse_entity_decl();
            return;
        }
        if self.cur_kind().is_some_and(Self::is_type_keyword) {
            self.parse_type_decl();
            return;
        }
        if self.is_contextual("sealed") && self.nth(1).is_some_and(Self::is_type_keyword) {
            self.parse_type_decl();
            return;
        }
        match self.cur_kind() {
            Some(kind)
                if kind == TokenKind::Identifier
                    || kind == TokenKind::Void
                    || Self::is_primitive_type(kind) =>
            {
                self.parse_function_decl(false);
            }
            _ => self.recover_top_level_region(),
        }
    }

    // --- Refused families -------------------------------------------------

    fn refuse_test_decl(&mut self) {
        let span = self.cur_span();
        self.start(NodeKind::TestDecl);
        self.diag(
            ParseErrorCode::Parse001,
            span,
            "unsupported syntax family: `test` declarations are refused (S10; the declaration form is not accepted)".to_string(),
            "refused here",
        );
        self.bump();
        if self.at(TokenKind::StringLiteral) {
            self.bump();
        }
        if self.at(TokenKind::LBrace) {
            self.consume_balanced(TokenKind::LBrace, TokenKind::RBrace);
        }
        self.finish();
    }

    fn refuse_application_decl(&mut self) {
        let span = self.cur_span();
        self.start(NodeKind::ApplicationDecl);
        self.diag(
            ParseErrorCode::Parse001,
            span,
            "unsupported syntax family: `application` declarations are refused (S10)".to_string(),
            "refused here",
        );
        self.bump();
        if self.at(TokenKind::LBrace) {
            self.consume_balanced(TokenKind::LBrace, TokenKind::RBrace);
        }
        self.finish();
    }

    fn refuse_infra_decl(&mut self) {
        let span = self.cur_span();
        self.start(NodeKind::InfraDecl);
        self.diag(
            ParseErrorCode::Parse001,
            span,
            "unsupported syntax family: `infra` declarations are refused (S10)".to_string(),
            "refused here",
        );
        self.bump();
        if self.at(TokenKind::StringLiteral) {
            self.bump();
        }
        if self.at(TokenKind::LBrace) {
            self.consume_balanced(TokenKind::LBrace, TokenKind::RBrace);
        }
        self.finish();
    }

    fn refuse_extern_decl(&mut self) {
        let span = self.cur_span();
        self.start(NodeKind::ExternDecl);
        self.diag(
            ParseErrorCode::Parse001,
            span,
            "unsupported syntax family: `extern` declarations and `function-signature` are refused: the foreign-function interface is security-sensitive and not implemented (S10)"
                .to_string(),
            "refused here",
        );
        self.bump();
        self.consume_signature_region();
        self.finish();
    }

    /// Consumes `[ return-type ] identifier parameter-list` after `extern`,
    /// stopping after the first closed parameter list, before any block, or
    /// at the start of a new line before the list opened.
    fn consume_signature_region(&mut self) {
        let mut depth: u32 = 0;
        let mut saw_open = false;
        while let Some(tok) = self.current() {
            if !saw_open && tok.preceded_by_lt && depth == 0 {
                break;
            }
            match tok.kind {
                TokenKind::LParen => {
                    depth += 1;
                    saw_open = true;
                    self.bump();
                }
                TokenKind::RParen => {
                    self.bump();
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::LBrace => break,
                _ => self.bump(),
            }
        }
    }

    fn refuse_foreign_module_decl(&mut self) {
        let span = self.cur_span();
        self.start(NodeKind::ForeignModuleDecl);
        self.diag(
            ParseErrorCode::Parse001,
            span,
            "unsupported syntax family: `foreign module` declarations are refused (S10; they desugar to `extern` declarations)".to_string(),
            "refused here",
        );
        self.bump();
        self.bump();
        if self.at(TokenKind::StringLiteral) {
            self.bump();
        }
        if self.at(TokenKind::LBrace) {
            self.consume_balanced(TokenKind::LBrace, TokenKind::RBrace);
        }
        self.finish();
    }

    fn refuse_entity_decl(&mut self) {
        let span = self.cur_span();
        self.start(NodeKind::EntityDecl);
        self.diag(
            ParseErrorCode::Parse001,
            span,
            "unsupported syntax family: `entity` declarations are refused (S10; the semantics are unknown)".to_string(),
            "refused here",
        );
        self.bump();
        if self.at(TokenKind::Identifier) {
            self.bump();
        }
        if self.at(TokenKind::Extends) {
            self.bump();
            self.parse_type_ref();
        }
        if self.at(TokenKind::Implements) {
            self.bump();
            self.parse_type_ref();
            while self.at(TokenKind::Comma) {
                self.bump();
                self.parse_type_ref();
            }
        }
        if self.at(TokenKind::LBrace) {
            self.consume_balanced(TokenKind::LBrace, TokenKind::RBrace);
        }
        self.finish();
    }

    // --- Type declarations ------------------------------------------------

    /// `type-decl = [ "sealed" ] ( class-decl | interface-decl | record-decl
    /// | enum-decl | entity-decl )`.
    pub(super) fn parse_type_decl(&mut self) {
        self.start(NodeKind::TypeDecl);
        if self.is_contextual("sealed") {
            self.bump();
        }
        match self.cur_kind() {
            Some(TokenKind::Class) => self.parse_class_decl(),
            Some(TokenKind::Interface) => self.parse_interface_decl(),
            Some(TokenKind::Record) => self.parse_record_decl(),
            Some(TokenKind::Enum) => self.parse_enum_decl(),
            Some(TokenKind::Entity) => self.refuse_entity_decl(),
            _ => self.diag_unexpected("`class`, `interface`, `record`, `enum` or `entity`"),
        }
        self.finish();
    }

    fn parse_class_decl(&mut self) {
        self.start(NodeKind::ClassDecl);
        self.bump();
        self.expect_ident("the class name");
        self.parse_supertypes();
        self.parse_member_body(NodeKind::ClassBody);
        self.finish();
    }

    fn parse_interface_decl(&mut self) {
        self.start(NodeKind::InterfaceDecl);
        self.bump();
        self.expect_ident("the interface name");
        self.parse_supertypes();
        self.parse_member_body(NodeKind::ClassBody);
        self.finish();
    }

    fn parse_record_decl(&mut self) {
        self.start(NodeKind::RecordDecl);
        self.bump();
        self.expect_ident("the record name");
        if self.at(TokenKind::LParen) {
            self.parse_positional_record();
        } else if self.at(TokenKind::LBrace) {
            self.parse_member_body(NodeKind::RecordBody);
        } else {
            self.diag_unexpected("`(` (positional record) or `{` (record body)");
        }
        self.finish();
    }

    fn parse_positional_record(&mut self) {
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::PositionalRecord);
        self.bump();
        loop {
            if self.eof() || self.at(TokenKind::RParen) {
                break;
            }
            let before = self.pos;
            let before_diags = self.diagnostics.len();
            self.parse_record_component();
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
        self.close_token(
            TokenKind::RParen,
            "`)` closing the positional record",
            diags_at_start,
        );
        self.finish();
    }

    /// `record-component = type-ref identifier`.
    fn parse_record_component(&mut self) {
        self.start(NodeKind::RecordComponent);
        self.parse_type_ref();
        self.expect_ident("the component name");
        self.finish();
    }

    fn parse_enum_decl(&mut self) {
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::EnumDecl);
        self.bump();
        self.expect_ident("the enum name");
        if self.expect(TokenKind::LBrace, "`{` to open the enum body") {
            loop {
                if self.eof() || self.at(TokenKind::RBrace) {
                    break;
                }
                let before = self.pos;
                let before_diags = self.diagnostics.len();
                self.parse_enum_constant();
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
        }
        self.close_token(
            TokenKind::RBrace,
            "`}` closing the enum body",
            diags_at_start,
        );
        self.finish();
    }

    /// `enum-constant = identifier [ "(" [ expression { "," expression } ] ")" ]`.
    fn parse_enum_constant(&mut self) {
        self.start(NodeKind::EnumConstant);
        if !self.expect_ident("an enum constant name") {
            self.finish();
            return;
        }
        if self.at(TokenKind::LParen) {
            self.parse_call_arguments();
        }
        self.finish();
    }

    /// `supertypes = [ "extends" type-ref ] [ "implements" type-ref { "," type-ref } ]`.
    fn parse_supertypes(&mut self) {
        if !self.at(TokenKind::Extends) && !self.at(TokenKind::Implements) {
            return;
        }
        self.start(NodeKind::Supertypes);
        if self.at(TokenKind::Extends) {
            self.bump();
            self.parse_type_ref();
        }
        if self.at(TokenKind::Implements) {
            self.bump();
            self.parse_type_ref();
            while self.at(TokenKind::Comma) {
                self.bump();
                self.parse_type_ref();
            }
        }
        self.finish();
    }

    // --- Members ----------------------------------------------------------

    /// `{ member }` of a class, interface or record body.
    fn parse_member_body(&mut self, kind: NodeKind) {
        let diags_at_start = self.diagnostics.len();
        self.start(kind);
        if self.expect(TokenKind::LBrace, "`{` to open the type body") {
            while !self.eof() && !self.at(TokenKind::RBrace) {
                let before = self.pos;
                let before_diags = self.diagnostics.len();
                self.parse_member();
                self.progress_guard(before, before_diags);
            }
        }
        self.close_token(
            TokenKind::RBrace,
            "`}` closing the type body",
            diags_at_start,
        );
        self.finish();
    }

    /// `member = { annotation } { modifier } ( constructor-decl | field-decl
    /// | function-decl | type-decl )`.
    fn parse_member(&mut self) {
        if !self.enter_nesting() {
            self.over_nesting_limit();
            return;
        }
        self.start(NodeKind::Member);
        self.parse_annotations();
        while self.cur_kind().is_some_and(Self::is_modifier) {
            self.bump();
        }
        match self.cur_kind() {
            Some(TokenKind::Identifier)
                if self.current_text() == Some("constructor")
                    && self.nth(1) == Some(TokenKind::LParen) =>
            {
                self.parse_constructor_decl();
            }
            Some(kind) if Self::is_type_keyword(kind) => self.parse_type_decl(),
            Some(kind)
                if kind == TokenKind::Identifier
                    || kind == TokenKind::Void
                    || Self::is_primitive_type(kind) =>
            {
                if !self.try_function_decl(false) {
                    self.parse_field_decl();
                }
            }
            _ => {
                self.recover_region(
                    |p| p.at(TokenKind::RBrace) || (p.preceded_by_lt() && p.can_start_member()),
                    "unexpected token in the type body",
                );
            }
        }
        self.finish();
        self.exit_nesting();
    }

    fn can_start_member(&self) -> bool {
        let Some(kind) = self.cur_kind() else {
            return false;
        };
        kind == TokenKind::At
            || kind == TokenKind::Identifier
            || kind == TokenKind::Void
            || Self::is_modifier(kind)
            || Self::is_type_keyword(kind)
            || Self::is_primitive_type(kind)
    }

    /// `constructor-decl = [ "public" ] "constructor" parameter-list block`
    /// (the leading `public` is consumed by the modifier loop).
    fn parse_constructor_decl(&mut self) {
        self.start(NodeKind::ConstructorDecl);
        self.bump();
        self.parse_parameter_list();
        self.parse_block();
        self.finish();
    }

    /// `field-decl = type-ref identifier [ "=" expression ]`.
    fn parse_field_decl(&mut self) {
        self.start(NodeKind::FieldDecl);
        self.parse_type_ref();
        self.expect_ident("the field name");
        if self.at(TokenKind::Eq) {
            self.bump();
            self.parse_expr_required("the field initialiser expression");
        }
        self.finish();
    }

    // --- Functions and parameters ----------------------------------------

    /// Speculative attempt: commits the parsed `FunctionDecl` iff the whole
    /// declaration parses without a diagnostic and its body is present.
    pub(super) fn try_function_decl(&mut self, same_line_body: bool) -> bool {
        self.speculate(|p| p.parse_function_decl(same_line_body))
    }

    /// `function-decl = [ return-type ] identifier parameter-list
    /// function-body`. Returns whether the declaration (with body) was
    /// parsed; the caller decides what to do otherwise. A nested declaration
    /// requires its body to start on the same line (`same_line_body`), since
    /// a `{` on a later line is a following block statement under statement
    /// separation; top-level and member declarations are not line-gated.
    pub(super) fn parse_function_decl(&mut self, same_line_body: bool) -> bool {
        self.start(NodeKind::FunctionDecl);
        let mut signature_ok = false;
        match self.cur_kind() {
            Some(TokenKind::Identifier) => {
                // `name ( …` (shorter derivation) first, else `Type name ( …`.
                let cp = self.checkpoint();
                self.bump();
                if self.at(TokenKind::LParen) {
                    signature_ok = self.parse_parameter_list();
                } else {
                    self.rollback(cp);
                    self.parse_type_ref();
                    signature_ok = self.expect_ident("the function name");
                    if signature_ok {
                        signature_ok = self.parse_parameter_list();
                    }
                }
            }
            Some(TokenKind::Void) => {
                self.bump();
                signature_ok = self.expect_ident("the function name");
                if signature_ok {
                    signature_ok = self.parse_parameter_list();
                }
            }
            Some(kind) if Self::is_primitive_type(kind) => {
                self.parse_type_ref();
                signature_ok = self.expect_ident("the function name");
                if signature_ok {
                    signature_ok = self.parse_parameter_list();
                }
            }
            _ => {}
        }
        let mut body = false;
        if signature_ok {
            let body_starts_here = (self.at(TokenKind::LBrace) || self.at(TokenKind::Eq))
                && (!same_line_body || self.same_line());
            if body_starts_here {
                if self.at(TokenKind::LBrace) {
                    self.parse_block();
                } else {
                    self.bump();
                    self.parse_expr_required("the `= expression` function body");
                }
                body = true;
            } else {
                self.diag_unexpected("a function body (a block or `= expression`)");
            }
        }
        self.finish();
        signature_ok && body
    }

    /// `parameter-list = "(" [ parameter { "," parameter } ] ")"`.
    pub(super) fn parse_parameter_list(&mut self) -> bool {
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::ParameterList);
        let opened = self.expect(TokenKind::LParen, "`(` opening the parameter list");
        if opened {
            loop {
                if self.eof() || self.at(TokenKind::RParen) {
                    break;
                }
                let before = self.pos;
                let before_diags = self.diagnostics.len();
                self.parse_parameter();
                if self.pos == before {
                    if self.diagnostics.len() == before_diags {
                        self.diag_unexpected("a parameter");
                    }
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
                "`)` closing the parameter list",
                diags_at_start,
            );
        }
        self.finish();
        opened
    }

    /// `parameter = ( identifier ":" type-ref | type-ref identifier )
    /// [ "=" expression ]`. The one-token lookahead is the accepted rule: an
    /// identifier directly followed by `:` selects the `name : Type` order;
    /// otherwise the `Type name` order is parsed. A parameter without a type
    /// is not derivable (S1 open) and is a syntax error.
    pub(super) fn parse_parameter(&mut self) {
        self.start(NodeKind::Parameter);
        if self.cur_kind() == Some(TokenKind::Identifier) && self.nth(1) == Some(TokenKind::Colon) {
            self.bump();
            self.bump();
            self.parse_type_ref();
        } else if self.cur_kind().is_some_and(Self::can_start_type) {
            self.parse_type_ref();
            self.expect_ident(
                "the parameter name after its type (a parameter without a type is not derivable; S1)",
            );
        } else {
            self.diag_unexpected("a parameter");
        }
        if self.at(TokenKind::Eq) {
            self.bump();
            self.parse_expr_required("the default value expression");
        }
        self.finish();
    }

    // --- Types ------------------------------------------------------------

    /// `type-ref = ( primitive-type | nominal-type ) [ "?" ] { array-suffix }`.
    pub(super) fn parse_type_ref(&mut self) {
        if !self.cur_kind().is_some_and(Self::can_start_type) {
            self.diag_unexpected("a type");
            return;
        }
        self.start(NodeKind::TypeRef);
        if self.cur_kind().is_some_and(Self::is_primitive_type) {
            self.bump();
        } else {
            self.parse_nominal_type();
        }
        if self.at(TokenKind::Question) {
            self.bump();
        }
        while self.at(TokenKind::LBracket) {
            self.start(NodeKind::ArraySuffix);
            self.bump();
            self.expect(TokenKind::RBracket, "`]` closing the array suffix");
            self.finish();
        }
        self.finish();
    }

    /// `nominal-type = qualified-name [ type-arguments ]`.
    fn parse_nominal_type(&mut self) {
        self.start(NodeKind::NominalType);
        self.parse_qualified_name("a type name");
        if self.at(TokenKind::Lt) {
            self.parse_type_arguments();
        }
        self.finish();
    }

    /// `type-arguments = "<" type-ref { "," type-ref } ">"`. The closing `>`
    /// characters come from the pre-split `>>`/`>>>` tokens when nested
    /// arguments close together.
    pub(super) fn parse_type_arguments(&mut self) {
        if !self.enter_nesting() {
            self.over_nesting_limit();
            return;
        }
        let diags_at_start = self.diagnostics.len();
        self.start(NodeKind::TypeArguments);
        if self.expect(TokenKind::Lt, "`<` opening the type arguments") {
            self.parse_type_ref();
            while self.at(TokenKind::Comma) {
                self.bump();
                if self.cur_kind().is_some_and(Self::can_start_type) {
                    self.parse_type_ref();
                } else {
                    self.diag_unexpected("a type argument after `,`");
                    break;
                }
            }
            self.close_token(
                TokenKind::Gt,
                "`>` closing the type arguments",
                diags_at_start,
            );
        }
        self.finish();
        self.exit_nesting();
    }

    // --- Recovery ---------------------------------------------------------

    fn recover_top_level_region(&mut self) {
        self.recover_region(
            |p| p.preceded_by_lt() && p.cur_kind().is_some_and(Self::can_start_top_level),
            "unexpected token at the top level (a declaration is expected)",
        );
    }
}
