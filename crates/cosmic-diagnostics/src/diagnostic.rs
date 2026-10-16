use core::fmt;

use cosmic_source::Span;

use crate::DiagnosticCode;

/// Severity of a diagnostic.
///
/// Severities are presented in this order in reports: `Error`, `Warning`,
/// `Note`, `Help`. The enum deliberately does not implement `Ord`: sorting
/// diagnostics is defined by source position (see
/// [`DiagnosticBag::sorted`](crate::DiagnosticBag::sorted)), not by severity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Severity {
    /// The program is invalid; compilation cannot continue to code
    /// generation.
    Error,
    /// The program is valid but something is suspicious.
    Warning,
    /// Additional context attached to another diagnostic.
    Note,
    /// A suggestion for the user; never invalidates the program.
    Help,
}

impl Severity {
    /// Lowercase name used in rendered output and machine-readable formats.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }

    /// Whether this severity marks the program as invalid.
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(self, Severity::Error)
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A labelled source span inside a diagnostic.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Label {
    span: Span,
    message: String,
}

impl Label {
    /// Creates a label spanning `span` with a human-readable `message`.
    /// Empty messages are allowed: the label then only underlines the span.
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    /// The labelled span.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    /// The label message (possibly empty).
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// How reliably a [`Suggestion`] can be applied by tooling.
///
/// Mirrors the applicability vocabulary used by language servers so that
/// future LSP code actions can map suggestions directly.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Applicability {
    /// The suggestion is guaranteed to be the intended change.
    MachineApplicable,
    /// The suggestion is plausible but may change behavior.
    MaybeIncorrect,
    /// The replacement contains placeholder text and must be edited.
    HasPlaceholders,
    /// Applicability was not declared.
    #[default]
    Unspecified,
}

impl Applicability {
    /// Machine-readable name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Applicability::MachineApplicable => "machine-applicable",
            Applicability::MaybeIncorrect => "maybe-incorrect",
            Applicability::HasPlaceholders => "has-placeholders",
            Applicability::Unspecified => "unspecified",
        }
    }
}

impl fmt::Display for Applicability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A structured fix: replace the text of `span` with `replacement`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Suggestion {
    span: Span,
    replacement: String,
    message: String,
    applicability: Applicability,
}

impl Suggestion {
    /// Creates a suggestion to replace the text at `span` with
    /// `replacement`, described by `message`.
    ///
    /// `message` should be self-contained for human rendering (e.g.
    /// ``"replace `letx` with `let x`"``); the human renderer prints it
    /// verbatim, while the machine-readable renderer exposes the structured
    /// span/replacement pair.
    pub fn new(span: Span, replacement: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            span,
            replacement: replacement.into(),
            message: message.into(),
            applicability: Applicability::Unspecified,
        }
    }

    /// Declares [`Applicability`] (builder style).
    #[must_use]
    pub fn with_applicability(mut self, applicability: Applicability) -> Self {
        self.applicability = applicability;
        self
    }

    /// The span this suggestion replaces.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    /// The replacement text.
    #[must_use]
    pub fn replacement(&self) -> &str {
        &self.replacement
    }

    /// The human-readable description.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Declared applicability.
    #[must_use]
    pub fn applicability(&self) -> Applicability {
        self.applicability
    }
}

/// A structured diagnostic: severity, stable code, message, source labels,
/// notes and suggestions.
///
/// Build with [`Diagnostic::error`] and friends, then chain
/// [`Diagnostic::with_secondary`], [`Diagnostic::with_note`] and
/// [`Diagnostic::with_suggestion`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Diagnostic {
    severity: Severity,
    code: DiagnosticCode,
    message: String,
    primary: Label,
    secondary: Vec<Label>,
    notes: Vec<String>,
    suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    /// Creates a diagnostic. Prefer the severity-named constructors
    /// ([`Diagnostic::error`], [`Diagnostic::warning`], [`Diagnostic::note`],
    /// [`Diagnostic::help`]).
    pub fn new(
        severity: Severity,
        code: DiagnosticCode,
        message: impl Into<String>,
        primary: Label,
    ) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            primary,
            secondary: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// An [`Severity::Error`] diagnostic. Every diagnostic has exactly one
    /// primary label.
    pub fn error(code: DiagnosticCode, message: impl Into<String>, primary: Label) -> Self {
        Self::new(Severity::Error, code, message, primary)
    }

    /// A [`Severity::Warning`] diagnostic.
    pub fn warning(code: DiagnosticCode, message: impl Into<String>, primary: Label) -> Self {
        Self::new(Severity::Warning, code, message, primary)
    }

    /// A [`Severity::Note`] diagnostic.
    pub fn note(code: DiagnosticCode, message: impl Into<String>, primary: Label) -> Self {
        Self::new(Severity::Note, code, message, primary)
    }

    /// A [`Severity::Help`] diagnostic.
    pub fn help(code: DiagnosticCode, message: impl Into<String>, primary: Label) -> Self {
        Self::new(Severity::Help, code, message, primary)
    }

    /// Appends a secondary label (builder style).
    #[must_use]
    pub fn with_secondary(mut self, label: Label) -> Self {
        self.secondary.push(label);
        self
    }

    /// Appends a free-form note (builder style).
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Appends a suggestion (builder style).
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// The severity.
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// The stable code.
    #[must_use]
    pub fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// The main message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The required primary label.
    #[must_use]
    pub fn primary(&self) -> &Label {
        &self.primary
    }

    /// Secondary labels, in insertion order.
    #[must_use]
    pub fn secondary(&self) -> &[Label] {
        &self.secondary
    }

    /// Notes, in insertion order.
    #[must_use]
    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    /// Suggestions, in insertion order.
    #[must_use]
    pub fn suggestions(&self) -> &[Suggestion] {
        &self.suggestions
    }
}
