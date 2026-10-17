//! Token kinds and the keyword table of the accepted language surface.

/// The kind of one language token.
///
/// The vocabulary mirrors the accepted grammar (`docs/spec/grammar.ebnf`
/// §2.3–§2.5) exactly:
///
/// * identifiers — the grammar's `identifier-name` minus the keyword and
///   reserved-invalid sets; contextual words such as `test`, `application`,
///   `infra`, `foreign`, `module` and `sealed` are **not** keywords and stay
///   [`TokenKind::Identifier`], so the parser can recognise them by
///   lookahead;
/// * the literal classes `int`/`long`/`float`/`double`/`string`/`char`, plus
///   `true`/`false` (both [`TokenKind::BooleanLiteral`]) and `null`
///   ([`TokenKind::NullLiteral`]);
/// * the 59 keywords, the three reserved-invalid words (`fun`, `fn`, `func`
///   — lexed, then rejected by the parser) and the operator, delimiter and
///   separator tables.
///
/// Numeric classification is purely **syntactic** and suffix-based, per the
/// grammar: a decimal literal without suffix is an [`TokenKind::IntLiteral`]
/// regardless of its value. Value-based integer promotion is a semantic
/// question owned by decision record S7 and is deliberately not performed
/// here, so the lexer never takes a rule that remains gated.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TokenKind {
    /// An identifier (`identifier-name` minus the keyword and reserved-invalid sets).
    Identifier,
    /// An integer literal: decimal digits (no suffix) or `0x`/`0X` hex digits.
    IntLiteral,
    /// A long literal: an integer literal with a trailing `l`/`L`.
    LongLiteral,
    /// A float literal: a number body with a trailing `f`/`F`.
    FloatLiteral,
    /// A double literal: a number body with a fraction or exponent, and an
    /// optional trailing `d`/`D`.
    DoubleLiteral,
    /// A string literal, delimited by `"`.
    StringLiteral,
    /// A character literal, delimited by `'`, carrying exactly one character value.
    CharLiteral,
    /// A boolean literal: `true` or `false`.
    BooleanLiteral,
    /// A null literal: `null`.
    NullLiteral,
    /// Reserved-invalid word `fun`; the parser rejects it.
    ReservedFun,
    /// Reserved-invalid word `fn`; the parser rejects it.
    ReservedFn,
    /// Reserved-invalid word `func`; the parser rejects it.
    ReservedFunc,
    /// Keyword `abstract`.
    Abstract,
    /// Keyword `as`.
    As,
    /// Keyword `assert`.
    Assert,
    /// Keyword `audited`.
    Audited,
    /// Keyword `await`.
    Await,
    /// Keyword `bool`.
    Bool,
    /// Keyword `break`.
    Break,
    /// Keyword `byte`.
    Byte,
    /// Keyword `case`.
    Case,
    /// Keyword `catch`.
    Catch,
    /// Keyword `char`.
    Char,
    /// Keyword `class`.
    Class,
    /// Keyword `continue`.
    Continue,
    /// Keyword `default`.
    Default,
    /// Keyword `do`.
    Do,
    /// Keyword `double`.
    Double,
    /// Keyword `else`.
    Else,
    /// Keyword `entity`.
    Entity,
    /// Keyword `enum`.
    Enum,
    /// Keyword `extern`.
    Extern,
    /// Keyword `extends`.
    Extends,
    /// Keyword `final`.
    Final,
    /// Keyword `finally`.
    Finally,
    /// Keyword `float`.
    Float,
    /// Keyword `for`.
    For,
    /// Keyword `generated`.
    Generated,
    /// Keyword `if`.
    If,
    /// Keyword `implements`.
    Implements,
    /// Keyword `import`.
    Import,
    /// Keyword `instanceof`.
    Instanceof,
    /// Keyword `int`.
    Int,
    /// Keyword `interface`.
    Interface,
    /// Keyword `long`.
    Long,
    /// Keyword `native`.
    Native,
    /// Keyword `new`.
    New,
    /// Keyword `override`.
    Override,
    /// Keyword `package`.
    Package,
    /// Keyword `private`.
    Private,
    /// Keyword `protected`.
    Protected,
    /// Keyword `public`.
    Public,
    /// Keyword `record`.
    Record,
    /// Keyword `return`.
    Return,
    /// Keyword `short`.
    Short,
    /// Keyword `spawn`.
    Spawn,
    /// Keyword `static`.
    Static,
    /// Keyword `string`.
    String,
    /// Keyword `super`.
    Super,
    /// Keyword `switch`.
    Switch,
    /// Keyword `synchronized`.
    Synchronized,
    /// Keyword `this`.
    This,
    /// Keyword `throw`.
    Throw,
    /// Keyword `transient`.
    Transient,
    /// Keyword `try`.
    Try,
    /// Keyword `unique`.
    Unique,
    /// Keyword `val`.
    Val,
    /// Keyword `var`.
    Var,
    /// Keyword `void`.
    Void,
    /// Keyword `volatile`.
    Volatile,
    /// Keyword `while`.
    While,
    /// Operator `+`.
    Plus,
    /// Operator `-`.
    Minus,
    /// Operator `*`.
    Star,
    /// Operator `/`.
    Slash,
    /// Operator `%`.
    Percent,
    /// Operator `==`.
    EqEq,
    /// Operator `!=`.
    BangEq,
    /// Operator `<`.
    Lt,
    /// Operator `<=`.
    LtEq,
    /// Operator `>`.
    Gt,
    /// Operator `>=`.
    GtEq,
    /// Operator `&&`.
    AmpAmp,
    /// Operator `||`.
    PipePipe,
    /// Operator `!`.
    Bang,
    /// Operator `&`.
    Amp,
    /// Operator `|`.
    Pipe,
    /// Operator `^`.
    Caret,
    /// Operator `<<`.
    Shl,
    /// Operator `>>`.
    Shr,
    /// Operator `>>>`.
    Ushr,
    /// Operator `=`.
    Eq,
    /// Operator `+=`.
    PlusEq,
    /// Operator `-=`.
    MinusEq,
    /// Operator `*=`.
    StarEq,
    /// Operator `/=`.
    SlashEq,
    /// Operator `%=`.
    PercentEq,
    /// Operator `&=`.
    AmpEq,
    /// Operator `|=`.
    PipeEq,
    /// Operator `^=`.
    CaretEq,
    /// Operator `<<=`.
    ShlEq,
    /// Operator `>>=`.
    ShrEq,
    /// Operator `>>>=`.
    UshrEq,
    /// Operator `++`.
    PlusPlus,
    /// Operator `--`.
    MinusMinus,
    /// Operator `->`.
    Arrow,
    /// Delimiter `(`.
    LParen,
    /// Delimiter `)`.
    RParen,
    /// Delimiter `{`.
    LBrace,
    /// Delimiter `}`.
    RBrace,
    /// Delimiter `[`.
    LBracket,
    /// Delimiter `]`.
    RBracket,
    /// Separator `;`.
    Semicolon,
    /// Separator `,`.
    Comma,
    /// Separator `.`.
    Dot,
    /// Separator `:`.
    Colon,
    /// Separator `?`.
    Question,
    /// Separator `@`.
    At,
}

impl TokenKind {
    /// A short human name for this kind, used in tests and messages.
    ///
    /// Fixed-spelling tokens (keywords, operators, delimiters, separators)
    /// return their source spelling; the remaining kinds return a category
    /// word such as `"identifier"` or `"string literal"`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        use TokenKind::*;
        match self {
            Identifier => "identifier",
            IntLiteral => "integer literal",
            LongLiteral => "long literal",
            FloatLiteral => "float literal",
            DoubleLiteral => "double literal",
            StringLiteral => "string literal",
            CharLiteral => "character literal",
            BooleanLiteral => "boolean literal",
            NullLiteral => "null literal",
            ReservedFun => "fun",
            ReservedFn => "fn",
            ReservedFunc => "func",
            Abstract => "abstract",
            As => "as",
            Assert => "assert",
            Audited => "audited",
            Await => "await",
            Bool => "bool",
            Break => "break",
            Byte => "byte",
            Case => "case",
            Catch => "catch",
            Char => "char",
            Class => "class",
            Continue => "continue",
            Default => "default",
            Do => "do",
            Double => "double",
            Else => "else",
            Entity => "entity",
            Enum => "enum",
            Extern => "extern",
            Extends => "extends",
            Final => "final",
            Finally => "finally",
            Float => "float",
            For => "for",
            Generated => "generated",
            If => "if",
            Implements => "implements",
            Import => "import",
            Instanceof => "instanceof",
            Int => "int",
            Interface => "interface",
            Long => "long",
            Native => "native",
            New => "new",
            Override => "override",
            Package => "package",
            Private => "private",
            Protected => "protected",
            Public => "public",
            Record => "record",
            Return => "return",
            Short => "short",
            Spawn => "spawn",
            Static => "static",
            String => "string",
            Super => "super",
            Switch => "switch",
            Synchronized => "synchronized",
            This => "this",
            Throw => "throw",
            Transient => "transient",
            Try => "try",
            Unique => "unique",
            Val => "val",
            Var => "var",
            Void => "void",
            Volatile => "volatile",
            While => "while",
            Plus => "+",
            Minus => "-",
            Star => "*",
            Slash => "/",
            Percent => "%",
            EqEq => "==",
            BangEq => "!=",
            Lt => "<",
            LtEq => "<=",
            Gt => ">",
            GtEq => ">=",
            AmpAmp => "&&",
            PipePipe => "||",
            Bang => "!",
            Amp => "&",
            Pipe => "|",
            Caret => "^",
            Shl => "<<",
            Shr => ">>",
            Ushr => ">>>",
            Eq => "=",
            PlusEq => "+=",
            MinusEq => "-=",
            StarEq => "*=",
            SlashEq => "/=",
            PercentEq => "%=",
            AmpEq => "&=",
            PipeEq => "|=",
            CaretEq => "^=",
            ShlEq => "<<=",
            ShrEq => ">>=",
            UshrEq => ">>>=",
            PlusPlus => "++",
            MinusMinus => "--",
            Arrow => "->",
            LParen => "(",
            RParen => ")",
            LBrace => "{",
            RBrace => "}",
            LBracket => "[",
            RBracket => "]",
            Semicolon => ";",
            Comma => ",",
            Dot => ".",
            Colon => ":",
            Question => "?",
            At => "@",
        }
    }

    /// Whether this kind is one of the 59 keywords (excluding the boolean and
    /// null literals and the reserved-invalid words).
    #[must_use]
    pub const fn is_keyword(self) -> bool {
        use TokenKind::*;
        matches!(
            self,
            Abstract
                | As
                | Assert
                | Audited
                | Await
                | Bool
                | Break
                | Byte
                | Case
                | Catch
                | Char
                | Class
                | Continue
                | Default
                | Do
                | Double
                | Else
                | Entity
                | Enum
                | Extern
                | Extends
                | Final
                | Finally
                | Float
                | For
                | Generated
                | If
                | Implements
                | Import
                | Instanceof
                | Int
                | Interface
                | Long
                | Native
                | New
                | Override
                | Package
                | Private
                | Protected
                | Public
                | Record
                | Return
                | Short
                | Spawn
                | Static
                | String
                | Super
                | Switch
                | Synchronized
                | This
                | Throw
                | Transient
                | Try
                | Unique
                | Val
                | Var
                | Void
                | Volatile
                | While
        )
    }
}

/// Looks up the fixed spelling of an identifier-shaped token.
///
/// Returns the keyword, reserved-invalid or boolean/null literal kind for
/// exact matches, and `None` for an ordinary identifier. Contextual words are
/// deliberately absent; see [`TokenKind`].
pub(crate) fn keyword_kind(name: &str) -> Option<TokenKind> {
    use TokenKind::*;
    Some(match name {
        "abstract" => Abstract,
        "as" => As,
        "assert" => Assert,
        "audited" => Audited,
        "await" => Await,
        "bool" => Bool,
        "break" => Break,
        "byte" => Byte,
        "case" => Case,
        "catch" => Catch,
        "char" => Char,
        "class" => Class,
        "continue" => Continue,
        "default" => Default,
        "do" => Do,
        "double" => Double,
        "else" => Else,
        "entity" => Entity,
        "enum" => Enum,
        "extern" => Extern,
        "extends" => Extends,
        "final" => Final,
        "finally" => Finally,
        "float" => Float,
        "for" => For,
        "generated" => Generated,
        "if" => If,
        "implements" => Implements,
        "import" => Import,
        "instanceof" => Instanceof,
        "int" => Int,
        "interface" => Interface,
        "long" => Long,
        "native" => Native,
        "new" => New,
        "override" => Override,
        "package" => Package,
        "private" => Private,
        "protected" => Protected,
        "public" => Public,
        "record" => Record,
        "return" => Return,
        "short" => Short,
        "spawn" => Spawn,
        "static" => Static,
        "string" => String,
        "super" => Super,
        "switch" => Switch,
        "synchronized" => Synchronized,
        "this" => This,
        "throw" => Throw,
        "transient" => Transient,
        "try" => Try,
        "unique" => Unique,
        "val" => Val,
        "var" => Var,
        "void" => Void,
        "volatile" => Volatile,
        "while" => While,
        "true" | "false" => BooleanLiteral,
        "null" => NullLiteral,
        "fun" => ReservedFun,
        "fn" => ReservedFn,
        "func" => ReservedFunc,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{TokenKind, keyword_kind};

    /// The full accepted keyword list (`docs/spec/grammar.ebnf` §2.3),
    /// minus `true`/`false`/`null` (literal kinds) and the reserved-invalid
    /// words (checked separately).
    const KEYWORDS: [&str; 59] = [
        "abstract",
        "as",
        "assert",
        "audited",
        "await",
        "bool",
        "break",
        "byte",
        "case",
        "catch",
        "char",
        "class",
        "continue",
        "default",
        "do",
        "double",
        "else",
        "entity",
        "enum",
        "extern",
        "extends",
        "final",
        "finally",
        "float",
        "for",
        "generated",
        "if",
        "implements",
        "import",
        "instanceof",
        "int",
        "interface",
        "long",
        "native",
        "new",
        "override",
        "package",
        "private",
        "protected",
        "public",
        "record",
        "return",
        "short",
        "spawn",
        "static",
        "string",
        "super",
        "switch",
        "synchronized",
        "this",
        "throw",
        "transient",
        "try",
        "unique",
        "val",
        "var",
        "void",
        "volatile",
        "while",
    ];

    #[test]
    fn keyword_table_covers_the_full_accepted_set() {
        for keyword in KEYWORDS {
            let kind = keyword_kind(keyword).unwrap_or_else(|| panic!("missing keyword {keyword}"));
            assert!(kind.is_keyword(), "{keyword} must classify as a keyword");
            assert_eq!(kind.name(), keyword, "spelling must round-trip");
        }
        assert_eq!(keyword_kind("true"), Some(TokenKind::BooleanLiteral));
        assert_eq!(keyword_kind("false"), Some(TokenKind::BooleanLiteral));
        assert_eq!(keyword_kind("null"), Some(TokenKind::NullLiteral));
        assert_eq!(keyword_kind("fun"), Some(TokenKind::ReservedFun));
        assert_eq!(keyword_kind("fn"), Some(TokenKind::ReservedFn));
        assert_eq!(keyword_kind("func"), Some(TokenKind::ReservedFunc));
    }

    #[test]
    fn contextual_words_are_not_keywords() {
        for word in [
            "test",
            "application",
            "infra",
            "foreign",
            "module",
            "sealed",
        ] {
            assert_eq!(keyword_kind(word), None, "{word} must stay an identifier");
        }
    }

    #[test]
    fn operator_spellings_round_trip() {
        let kinds = [
            (TokenKind::PlusPlus, "++"),
            (TokenKind::UshrEq, ">>>="),
            (TokenKind::ShlEq, "<<="),
            (TokenKind::Arrow, "->"),
            (TokenKind::LBracket, "["),
            (TokenKind::At, "@"),
        ];
        for (kind, spelling) in kinds {
            assert_eq!(kind.name(), spelling);
        }
    }
}
