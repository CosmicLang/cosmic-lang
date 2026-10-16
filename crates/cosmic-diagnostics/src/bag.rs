use crate::Diagnostic;

/// An ordered collection of diagnostics.
///
/// Insertion order is preserved exactly. [`DiagnosticBag::sorted`] provides
/// the canonical *reporting* order (by source position, insertion order for
/// ties) without mutating the bag, so reporting is deterministic regardless
/// of how the compiler internals were scheduled.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiagnosticBag {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    /// Creates an empty bag.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one diagnostic.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Appends all diagnostics from an iterator.
    pub fn extend<I: IntoIterator<Item = Diagnostic>>(&mut self, diagnostics: I) {
        self.diagnostics.extend(diagnostics);
    }

    /// Number of diagnostics.
    #[must_use]
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Whether the bag is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Iterates in insertion order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    /// All diagnostics as a slice, in insertion order.
    #[must_use]
    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Consumes the bag, returning the diagnostics in insertion order.
    #[must_use]
    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    /// Number of [`Severity::Error`](crate::Severity::Error) diagnostics.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity().is_error())
            .count()
    }

    /// Number of [`Severity::Warning`](crate::Severity::Warning) diagnostics.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity() == crate::Severity::Warning)
            .count()
    }

    /// Whether any diagnostic is an error.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity().is_error())
    }

    /// The canonical reporting order: by primary file id, then start offset,
    /// then end offset; ties preserve insertion order (stable sort).
    #[must_use]
    pub fn sorted(&self) -> Vec<&Diagnostic> {
        let mut ordered: Vec<&Diagnostic> = self.diagnostics.iter().collect();
        ordered.sort_by_key(|diagnostic| {
            let span = diagnostic.primary().span();
            (span.file().to_raw(), span.start(), span.end())
        });
        ordered
    }
}

#[cfg(test)]
mod tests {
    use cosmic_source::SourceMap;

    use crate::{Diagnostic, DiagnosticCode, Label};

    use super::DiagnosticBag;

    fn diagnostic_at(sources: &SourceMap, id: cosmic_source::FileId, start: u32) -> Diagnostic {
        let span = sources.get(id).unwrap().span(start, start + 1).unwrap();
        Diagnostic::error(DiagnosticCode::new("CSM001"), "test", Label::new(span, ""))
    }

    #[test]
    fn counts_and_flags_track_severity() {
        let mut sources = SourceMap::new();
        let file = sources.add("a.csmx", "abcdef").unwrap();

        let mut bag = DiagnosticBag::new();
        assert!(bag.is_empty());
        assert!(!bag.has_errors());

        bag.push(diagnostic_at(&sources, file, 0));
        bag.push(Diagnostic::warning(
            DiagnosticCode::new("CSM002"),
            "w",
            Label::new(sources.get(file).unwrap().full_span(), ""),
        ));

        assert_eq!(bag.len(), 2);
        assert_eq!(bag.error_count(), 1);
        assert_eq!(bag.warning_count(), 1);
        assert!(bag.has_errors());
    }

    #[test]
    fn sorted_orders_by_file_then_offset_and_is_stable() {
        let mut sources = SourceMap::new();
        let first = sources.add("a.csmx", "xxx").unwrap();
        let second = sources.add("b.csmx", "yyy").unwrap();

        let d_first_late = diagnostic_at(&sources, first, 2);
        let d_second = diagnostic_at(&sources, second, 0);
        let d_first_early_a = diagnostic_at(&sources, first, 0);
        let d_first_early_b = diagnostic_at(&sources, first, 0);

        let mut bag = DiagnosticBag::new();
        bag.push(d_first_late.clone());
        bag.push(d_second.clone());
        bag.push(d_first_early_a.clone());
        bag.push(d_first_early_b.clone());

        let order = bag.sorted();
        assert_eq!(order.len(), 4);
        assert_eq!(order[0].primary().span().start(), 0);
        assert_eq!(order[1].primary().span().start(), 0);
        assert_eq!(order[2].primary().span().start(), 2);
        assert_eq!(order[3].primary().span().file(), second);

        // Stability: the two equal-key diagnostics keep insertion order.
        assert!(std::ptr::eq(order[0], &bag.as_slice()[2]));
        assert!(std::ptr::eq(order[1], &bag.as_slice()[3]));
    }
}
