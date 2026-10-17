//! The CST node vocabulary: what one node of the concrete syntax tree is.
//!
//! Every value that appears in a [`crate::Node`] is one of these kinds. The
//! variants fall into four groups:
//!
//! * **Structural nodes** for the recognised productions of
//!   `docs/spec/grammar.ebnf` §3 (`File`, `FunctionDecl`, `BinaryExpr`, …).
//! * **Refusal nodes** for the ten `UNSUPPORTED` productions and the refused
//!   labelled-jump form (`docs/spec/g1-s-syntax-decision-package.md` §4).
//!   These nodes are **error nodes**: they preserve the consumed bytes of a
//!   refused construct and always carry a [`crate::ParseErrorCode`]
//!   diagnostic. No semantics are attached to them.
//! * **`Error`** — a generic error node holding a malformed region that does
//!   not name a refused family. It, too, is an error node and carries at
//!   least one `PARSE` diagnostic.
//! * **Leaves** — one per lexer element ([`cosmic_lexer::ElementKind`]):
//!   tokens, trivia, the BOM and lexical error regions. There is no silent
//!   token loss: every lexer element becomes exactly one leaf, in order.

use cosmic_lexer::{ElementKind, TerminatorKind, TokenKind};

/// What one CST node is.
///
/// See the [crate-level documentation](crate) for the four groups. The name
/// `NodeKind` is used for leaves as well: a leaf is a node with no children.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NodeKind {
    // --- Structure -----------------------------------------------------
    /// The root of every CST: one `File` per parsed file (`source`; `bom`
    /// when present). Its span is the whole file.
    File,
    /// One `top-level-decl`: refused annotations plus exactly one
    /// top-level body.
    TopLevelDecl,
    /// One `member` of a `class-body`: refused annotations, modifiers, then
    /// one member declaration.
    Member,

    // --- Compilation unit ----------------------------------------------
    /// `package-decl` — `package` plus a qualified name.
    PackageDecl,
    /// `import-decl` — `import` plus a qualified name.
    ImportDecl,
    /// `qualified-name` — `identifier { "." identifier }`.
    QualifiedName,

    // --- Declarations ---------------------------------------------------
    /// `function-decl` — optional return type, name, parameter list and
    /// body (recognition only; S1 owns typing).
    FunctionDecl,
    /// `parameter-list` — `"(" [ parameter { "," parameter } ] ")"`.
    ParameterList,
    /// `parameter` — `identifier ":" type-ref` or `type-ref identifier`,
    /// each with an optional `= default` (F4).
    Parameter,
    /// `type-decl` — the optional contextual `sealed` prefix and one of the
    /// type declaration forms.
    TypeDecl,
    /// `class-decl`.
    ClassDecl,
    /// `interface-decl`.
    InterfaceDecl,
    /// `record-decl`.
    RecordDecl,
    /// `positional-record` — `"(" [ record-component { "," record-component } ] ")"`.
    PositionalRecord,
    /// `record-body` — `"{" { member } "}"`.
    RecordBody,
    /// `record-component` — `type-ref identifier`.
    RecordComponent,
    /// `enum-decl`.
    EnumDecl,
    /// `enum-constant` — `identifier [ "(" [ expression { "," expression } ] ")" ]`.
    EnumConstant,
    /// `class-body` — `"{" { member } "}"`.
    ClassBody,
    /// `constructor-decl` — optional `public`, `constructor`, parameter list
    /// and block.
    ConstructorDecl,
    /// `field-decl` — `type-ref identifier [ "=" expression ]`.
    FieldDecl,
    /// `supertypes` — the optional `extends`/`implements` clauses.
    Supertypes,

    // --- Types ----------------------------------------------------------
    /// `type-ref` — primitive or nominal type, optional `?`, array suffixes
    /// (S1/S2 own the semantics; recognition only).
    TypeRef,
    /// `nominal-type` — `qualified-name [ type-arguments ]`.
    NominalType,
    /// `type-arguments` — `"<" type-ref { "," type-ref } ">"`.
    TypeArguments,
    /// `array-suffix` — `"[" "]"`.
    ArraySuffix,

    // --- Statements -----------------------------------------------------
    /// `block` — `"{" { statement } "}"`.
    Block,
    /// `return-stmt` — `return [ expression ]`.
    ReturnStmt,
    /// `throw-stmt` — `throw expression`.
    ThrowStmt,
    /// `break-stmt` — unlabelled `break` (F1). A same-line label is a
    /// refused construct and never lands in this node.
    BreakStmt,
    /// `continue-stmt` — unlabelled `continue` (F1).
    ContinueStmt,
    /// `if-stmt` — condition plus then-branch, optional `else`.
    IfStmt,
    /// `while-stmt`.
    WhileStmt,
    /// `do-while-stmt`.
    DoWhileStmt,
    /// `for-stmt` — the three-clause form (F3) or the `for (… in …)` form.
    ForStmt,
    /// `for-init` — a `var`/`val` binding or an expression.
    ForInit,
    /// `try-stmt` — block, catch clauses, optional `finally`.
    TryStmt,
    /// `catch-clause` — `catch ( type-ref identifier )` plus a block.
    CatchClause,
    /// `finally-clause` — `finally` plus a block.
    FinallyClause,
    /// `switch-stmt` — colon arms (F2).
    SwitchStmt,
    /// `switch-stmt-arm` — `case … :` / `default :` plus its statement list.
    SwitchStmtArm,
    /// `var-decl-stmt` — `( "var" | "val" ) identifier [ ":" type-ref ] "=" expression`.
    VarDeclStmt,
    /// `expression-stmt` — an expression in statement position (including the
    /// assignment form).
    ExprStmt,
    /// The empty statement: one `;`.
    EmptyStmt,

    // --- Patterns (switch labels) --------------------------------------
    /// `binding-pattern` — `identifier identifier` (F5).
    BindingPattern,
    /// `destructuring-pattern` — `identifier "(" pattern-field { "," pattern-field } ")"` (F5).
    DestructuringPattern,
    /// `pattern-field` — `( "var" | "val" ) identifier` (F5).
    PatternField,
    /// `guard` — `if expression` on a `case` arm.
    Guard,

    // --- Expressions ----------------------------------------------------
    /// `assignment-expr` — statement-position assignment (`=` and compound
    /// forms); semantic rejection of chains is SEM027's (S11).
    AssignExpr,
    /// `binary-expr` — one level of the accepted precedence table.
    BinaryExpr,
    /// `unary-expr` — `!`, `-`, `++`, `--`, `spawn`, `await`.
    UnaryExpr,
    /// `postfix-expr` — a primary plus its postfix operations (calls, member
    /// access, indexing, `++`/`--`, generic calls).
    PostfixExpr,
    /// `argument-list` — `"(" [ expression { "," expression } ] ")"` in a call.
    ArgumentList,
    /// A parenthesised expression: `"(" expression ")"` of `primary`.
    ParenExpr,
    /// `lambda` — the parenthesised form `"(" [params] ")" "->" body` or the
    /// brace form `"{" [params] "->" {statement} "}"`.
    LambdaExpr,
    /// `lambda-params` — the bare parameter list of a brace lambda
    /// (`{ a, b -> … }`). Parenthesised lambdas reuse [`NodeKind::ParameterList`].
    LambdaParams,
    /// `switch-expr` — arrow arms (F2).
    SwitchExpr,
    /// `switch-expr-arm` — `case … ->` / `default ->` plus one expression.
    SwitchExprArm,
    /// `if-expr` — the parser-evidenced expression form (S5 surface OPEN).
    IfExpr,
    /// `new-expr` — `new type-ref ( [arguments] )`.
    NewExpr,

    // --- Refusal nodes (error nodes by construction) --------------------
    /// Refused family 1: `test-decl` (S10).
    TestDecl,
    /// Refused family 2: `application-decl` (S10).
    ApplicationDecl,
    /// Refused family 3: `infra-decl` (S10).
    InfraDecl,
    /// Refused family 4: `extern-decl` / `function-signature` (S10).
    ExternDecl,
    /// Refused family 5: `foreign-module-decl` (S10).
    ForeignModuleDecl,
    /// Refused family 6: `entity-decl` (S10).
    EntityDecl,
    /// Refused family 7: `annotation` (S10).
    Annotation,
    /// Refused family 9: `array-literal` (S9).
    ArrayLiteral,

    // --- Generic recovery ----------------------------------------------
    /// A malformed region consumed by recovery. Always carries at least one
    /// `PARSE` diagnostic; its bytes are preserved exactly.
    Error,

    // --- Leaves (one per lexer element) ---------------------------------
    /// One language token of the given kind.
    Token(TokenKind),
    /// The leading UTF-8 BOM prefix leaf (D-L1 §4).
    Bom,
    /// A whitespace run.
    Whitespace,
    /// One line terminator in its exact byte form.
    LineTerminator(TerminatorKind),
    /// A `//` comment (excluding the line break).
    LineComment,
    /// A closed `/* … */` comment.
    BlockComment,
    /// A lexical error region (the bytes consumed by lexer recovery). The
    /// lexer diagnostics live in [`crate::ParsedFile::lexed`].
    LexError,
}

impl NodeKind {
    /// Whether this kind is a **leaf**: a node with no children that covers
    /// exactly one lexer element (a token, trivia, the BOM or a lexical
    /// error region).
    #[must_use]
    pub const fn is_leaf(self) -> bool {
        matches!(
            self,
            NodeKind::Token(_)
                | NodeKind::Bom
                | NodeKind::Whitespace
                | NodeKind::LineTerminator(_)
                | NodeKind::LineComment
                | NodeKind::BlockComment
                | NodeKind::LexError
        )
    }

    /// Whether this kind is trivia (whitespace, terminators or comments).
    #[must_use]
    pub const fn is_trivia(self) -> bool {
        matches!(
            self,
            NodeKind::Whitespace
                | NodeKind::LineTerminator(_)
                | NodeKind::LineComment
                | NodeKind::BlockComment
        )
    }

    /// Whether this kind is a token leaf of the given lexer kind.
    #[must_use]
    pub const fn is_token(self, kind: TokenKind) -> bool {
        matches!(self, NodeKind::Token(inner) if inner as u16 == kind as u16)
    }

    /// Whether this kind is an **error node**: a refusal node for an
    /// unsupported syntax family, or a generic recovery `Error`.
    ///
    /// Every error node carries at least one `PARSE` diagnostic. Error nodes
    /// never contain fabricated bytes: they only hold consumed elements.
    #[must_use]
    pub const fn is_error_node(self) -> bool {
        matches!(
            self,
            NodeKind::Error
                | NodeKind::TestDecl
                | NodeKind::ApplicationDecl
                | NodeKind::InfraDecl
                | NodeKind::ExternDecl
                | NodeKind::ForeignModuleDecl
                | NodeKind::EntityDecl
                | NodeKind::Annotation
                | NodeKind::ArrayLiteral
        )
    }

    /// Whether this kind is a refusal node for one of the accepted
    /// unsupported syntax families.
    #[must_use]
    pub const fn is_refusal_node(self) -> bool {
        matches!(
            self,
            NodeKind::TestDecl
                | NodeKind::ApplicationDecl
                | NodeKind::InfraDecl
                | NodeKind::ExternDecl
                | NodeKind::ForeignModuleDecl
                | NodeKind::EntityDecl
                | NodeKind::Annotation
                | NodeKind::ArrayLiteral
        )
    }

    /// The lexer element kind behind a leaf, when the leaf mirrors one.
    #[must_use]
    pub fn element_kind(self) -> Option<ElementKind> {
        match self {
            NodeKind::Token(kind) => Some(ElementKind::Token(kind)),
            NodeKind::Bom => Some(ElementKind::Bom),
            NodeKind::Whitespace => Some(ElementKind::Whitespace),
            NodeKind::LineTerminator(kind) => Some(ElementKind::LineTerminator(kind)),
            NodeKind::LineComment => Some(ElementKind::LineComment),
            NodeKind::BlockComment => Some(ElementKind::BlockComment),
            NodeKind::LexError => Some(ElementKind::Error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NodeKind;
    use cosmic_lexer::{TerminatorKind, TokenKind};

    #[test]
    fn leaf_and_trivia_classification_is_disjoint_and_total_for_leaves() {
        let leaves = [
            NodeKind::Token(TokenKind::Identifier),
            NodeKind::Bom,
            NodeKind::Whitespace,
            NodeKind::LineTerminator(TerminatorKind::Lf),
            NodeKind::LineComment,
            NodeKind::BlockComment,
            NodeKind::LexError,
        ];
        for leaf in leaves {
            assert!(leaf.is_leaf(), "{leaf:?}");
            assert!(!leaf.is_error_node(), "{leaf:?}");
        }
        assert!(!NodeKind::File.is_leaf());
        assert!(!NodeKind::File.is_error_node());
        assert!(NodeKind::Error.is_error_node());
        assert!(NodeKind::TestDecl.is_refusal_node());
        assert!(!NodeKind::Error.is_refusal_node());
    }

    #[test]
    fn element_kind_round_trips_tokens() {
        assert_eq!(
            NodeKind::Token(TokenKind::Var).element_kind(),
            Some(cosmic_lexer::ElementKind::Token(TokenKind::Var))
        );
        assert_eq!(NodeKind::File.element_kind(), None);
    }
}
