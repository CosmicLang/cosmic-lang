//! The lossless concrete syntax tree: arena, cursor API, reconstruction.
//!
//! [`ParsedFile`] owns the whole result of parsing one file: the lexer's
//! lossless element stream, the CST arena built from it, and the `PARSE`
//! diagnostics. The tree is a *concrete* syntax tree:
//!
//! * Every lexer element becomes exactly one leaf — tokens, trivia, the BOM
//!   prefix and lexical error regions alike. Nothing is dropped and nothing
//!   is invented.
//! * Every node stores an exact byte [`Span`]; internal node spans are the
//!   union of their children, the root's span is the whole file, and empty
//!   structural nodes (if any) are explicitly zero-width.
//! * [`ParsedFile::reprint`] concatenates the leaf bytes and reproduces the
//!   source byte-for-byte; [`ParsedFile::is_lossless`] verifies the stronger
//!   invariant that the leaves *are* the lexer elements, in order, spanning
//!   each element exactly.
//!
//! Navigation is cursor-based: [`Node`] borrows the tree (it is `Copy`) and
//! walks with [`Node::children`], [`Node::parent`] and [`Node::descendants`].
//! A node identifier ([`NodeId`]) is stable for the lifetime of the tree and
//! can be stored in side tables.
//!
//! Token splitting: a `>>`/`>>>` lexer token that closes nested type
//! arguments is represented by two/three `>` leaves whose spans partition
//! the token's span. This is the only case where one lexer element contributes
//! more than one leaf, and it is checked by [`ParsedFile::is_lossless`].

use cosmic_diagnostics::{Diagnostic, DiagnosticBag};
use cosmic_lexer::{ElementKind, LexElement, LexErrorCode, LexedFile, TokenKind};
use cosmic_source::{FileId, SourceFile, Span};

use crate::NodeKind;

/// A stable identifier of one node in a [`ParsedFile`].
///
/// Identifiers are dense indexes into the tree arena; `NodeId(0)` is always
/// the root (`File`) node. They are values: cheap to copy, compare and store.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct NodeId(u32);

impl NodeId {
    /// The dense arena index of this node.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// One node of the arena.
#[derive(Clone, Debug)]
struct NodeData {
    kind: NodeKind,
    span: Span,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    /// `Some` for leaves: the index of the lexer element this leaf mirrors.
    elem: Option<u32>,
    /// Lexical errors attached to the mirrored lexer element (leaves only).
    errors: Vec<LexErrorCode>,
}

/// One crate-internal build event of the tree.
///
/// The parser emits events while it consumes the token stream (and any
/// speculation is rolled back by truncating the event list before the tree
/// is built). The tree builder [`ParsedFile::build`] folds them into the
/// arena: `Start` opens a node, `Leaf` adds one leaf, `Finish` closes the
/// innermost open node and computes its span.
#[derive(Clone, Debug)]
pub(crate) enum Event {
    /// Open a node. `hint` is the byte offset to use if the node ends empty
    /// (the start of the current token, or the end of the file).
    Start {
        /// The kind of the opened node.
        kind: NodeKind,
        /// Zero-width fallback position inside the file.
        hint: u32,
    },
    /// Close the innermost open node.
    Finish,
    /// Add one leaf mirroring (a part of) one lexer element.
    Leaf {
        /// The leaf kind.
        kind: NodeKind,
        /// The exact byte range of this leaf.
        span: Span,
        /// The index of the mirrored lexer element.
        elem: u32,
        /// Lexical errors of the mirrored element.
        errors: Vec<LexErrorCode>,
    },
}

/// The complete lossless syntax analysis of one registered source file.
#[derive(Clone, Debug)]
pub struct ParsedFile {
    file: FileId,
    lexed: LexedFile,
    nodes: Vec<NodeData>,
    diagnostics: DiagnosticBag,
}

impl ParsedFile {
    /// Folds parser events into the arena. Crate-internal: produced by
    /// [`parse`](crate::parse).
    pub(crate) fn build(
        source: &SourceFile,
        lexed: LexedFile,
        events: Vec<Event>,
        diagnostics: DiagnosticBag,
    ) -> Self {
        let file = source.id();
        let fallback = source.full_span();
        let mut nodes: Vec<NodeData> = Vec::with_capacity(events.len() / 2 + 1);
        let mut stack: Vec<(NodeId, u32)> = Vec::new();

        for event in events {
            match event {
                Event::Start { kind, hint } => {
                    let id = NodeId(nodes.len() as u32);
                    let parent = stack.last().map(|&(id, _)| id);
                    nodes.push(NodeData {
                        kind,
                        span: fallback,
                        parent,
                        children: Vec::new(),
                        elem: None,
                        errors: Vec::new(),
                    });
                    if let Some(parent) = parent {
                        nodes[parent.0 as usize].children.push(id);
                    }
                    stack.push((id, hint));
                }
                Event::Finish => {
                    if let Some((id, hint)) = stack.pop() {
                        let span = {
                            let data = &nodes[id.0 as usize];
                            match (data.children.first(), data.children.last()) {
                                (Some(&first), Some(&last)) => {
                                    let start = nodes[first.0 as usize].span.start();
                                    let end = nodes[last.0 as usize].span.end();
                                    source.span(start, end).unwrap_or(fallback)
                                }
                                _ => source.span(hint, hint).unwrap_or(fallback),
                            }
                        };
                        nodes[id.0 as usize].span = span;
                    }
                }
                Event::Leaf {
                    kind,
                    span,
                    elem,
                    errors,
                } => {
                    let id = NodeId(nodes.len() as u32);
                    let parent = stack.last().map(|&(id, _)| id);
                    nodes.push(NodeData {
                        kind,
                        span,
                        parent,
                        children: Vec::new(),
                        elem: Some(elem),
                        errors,
                    });
                    if let Some(parent) = parent {
                        nodes[parent.0 as usize].children.push(id);
                    }
                }
            }
        }

        if nodes.is_empty() {
            // Defensive: a well-formed event stream opens `File` first; an
            // empty arena still yields a valid (empty) tree rather than a
            // panic.
            nodes.push(NodeData {
                kind: NodeKind::File,
                span: fallback,
                parent: None,
                children: Vec::new(),
                elem: None,
                errors: Vec::new(),
            });
        }
        // The root always spans the whole file.
        nodes[0].span = fallback;

        Self {
            file,
            lexed,
            nodes,
            diagnostics,
        }
    }

    /// The file this analysis belongs to.
    #[must_use]
    pub fn file(&self) -> FileId {
        self.file
    }

    /// The lossless element stream this tree was built from, with the `LEX`
    /// diagnostics.
    #[must_use]
    pub fn lexed(&self) -> &LexedFile {
        &self.lexed
    }

    /// The `PARSE` diagnostics reported while building the tree, in
    /// discovery order.
    ///
    /// [`DiagnosticBag::sorted`] defines the canonical reporting order.
    #[must_use]
    pub fn diagnostics(&self) -> &DiagnosticBag {
        &self.diagnostics
    }

    /// Every diagnostic of the whole front-end run: the `LEX` diagnostics
    /// followed by the `PARSE` diagnostics, sorted by primary span
    /// (insertion order for ties: `LEX` first). Deterministic.
    #[must_use]
    pub fn all_diagnostics(&self) -> Vec<&Diagnostic> {
        let mut all: Vec<&Diagnostic> =
            Vec::with_capacity(self.lexed.diagnostics().len() + self.diagnostics.len());
        all.extend(self.lexed.diagnostics().iter());
        all.extend(self.diagnostics.iter());
        all.sort_by_key(|diagnostic| {
            let span = diagnostic.primary().span();
            (span.start(), span.end())
        });
        all
    }

    /// Whether the front-end run reported at least one error, lexing or
    /// parsing. When this is `false` the whole file is recognised: every
    /// lexical rule held and the parser produced a fully-recognised tree.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.lexed.has_errors() || self.diagnostics.has_errors()
    }

    /// The root node: always [`NodeKind::File`], spanning the whole file.
    #[must_use]
    pub fn root(&self) -> Node<'_> {
        Node {
            tree: self,
            id: NodeId(0),
        }
    }

    /// The number of nodes (internal nodes and leaves) in the arena.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Every node in pre-order (the root first, then its whole subtree).
    #[must_use]
    pub fn nodes(&self) -> Descendants<'_> {
        Descendants {
            tree: self,
            stack: vec![NodeId(0)],
        }
    }

    /// Every leaf in source order: one per lexer element, except that a
    /// `>>`/`>>>` closing token contributes one `>` leaf per character.
    pub fn leaves(&self) -> impl Iterator<Item = Node<'_>> {
        self.nodes().filter(|node| node.is_leaf())
    }

    /// Every token leaf in source order, skipping trivia, the BOM and
    /// lexical error regions.
    pub fn tokens(&self) -> impl Iterator<Item = Node<'_>> {
        self.leaves().filter(|node| node.token_kind().is_some())
    }

    /// Whether the tree is a lossless representation of `source`:
    ///
    /// * the leaves, in order, partition the file exactly (contiguous, no
    ///   gap, no overlap, no fabricated bytes);
    /// * the leaves mirror the lexer elements of [`ParsedFile::lexed`] in
    ///   order — each element is covered exactly by its leaf (or by its
    ///   `>` leaves when a `>>`/`>>>` token was split);
    /// * every leaf kind agrees with the element kind it mirrors.
    #[must_use]
    pub fn is_lossless(&self, source: &SourceFile) -> bool {
        if source.id() != self.file {
            return false;
        }
        let elements = self.lexed.elements();
        let mut expected = 0_u32;
        let mut elem_cursor = 0_usize;

        for node in self.leaves() {
            let data = &self.nodes[node.id.0 as usize];
            let Some(elem_index) = data.elem else {
                // A leaf always mirrors an element.
                return false;
            };
            let index = elem_index as usize;
            if index != elem_cursor {
                // Elements must be mirrored in order, each exactly once.
                return false;
            }
            let Some(element) = elements.get(index) else {
                return false;
            };
            let elem_span = element.span();
            if data.span.file() != self.file
                || data.span.start() != expected
                || data.span.end() > elem_span.end()
            {
                return false;
            }
            if !leaf_kind_matches(data.kind, element.kind()) {
                return false;
            }
            expected = data.span.end();
            if expected == elem_span.end() {
                elem_cursor += 1;
            }
        }

        expected == source.len() && elem_cursor == elements.len()
    }

    /// Rebuilds the source text by concatenating the leaf bytes in order.
    ///
    /// Returns `None` when `source` is not the parsed file. For the parsed
    /// file this always succeeds and reproduces the text byte-for-byte.
    #[must_use]
    pub fn reprint(&self, source: &SourceFile) -> Option<String> {
        if source.id() != self.file {
            return None;
        }
        let mut out = String::with_capacity(source.len() as usize);
        for node in self.leaves() {
            out.push_str(source.text_of(node.span())?);
        }
        Some(out)
    }
}

/// Whether a leaf kind agrees with the element kind it mirrors.
///
/// Everything agrees one-to-one, except that the parser splits a `>>`
/// (`Shr`) or `>>>` (`Ushr`) token into individual `>` leaves when those
/// characters close nested type arguments.
fn leaf_kind_matches(leaf: NodeKind, element: ElementKind) -> bool {
    match element {
        ElementKind::Token(kind) => match leaf {
            NodeKind::Token(inner) => {
                inner == kind
                    || (inner == TokenKind::Gt && matches!(kind, TokenKind::Shr | TokenKind::Ushr))
            }
            _ => false,
        },
        ElementKind::Bom => leaf == NodeKind::Bom,
        ElementKind::Whitespace => leaf == NodeKind::Whitespace,
        ElementKind::LineTerminator(kind) => leaf == NodeKind::LineTerminator(kind),
        ElementKind::LineComment => leaf == NodeKind::LineComment,
        ElementKind::BlockComment => leaf == NodeKind::BlockComment,
        ElementKind::Error => leaf == NodeKind::LexError,
    }
}

/// A cursor to one node of a [`ParsedFile`].
///
/// The cursor is `Copy` and borrows the tree; every accessor is a cheap
/// arena lookup.
#[derive(Clone, Copy)]
pub struct Node<'a> {
    tree: &'a ParsedFile,
    id: NodeId,
}

impl std::fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("kind", &self.data().kind)
            .field("span", &self.data().span)
            .finish()
    }
}

impl<'a> Node<'a> {
    fn data(self) -> &'a NodeData {
        &self.tree.nodes[self.id.0 as usize]
    }

    /// The stable identifier of this node.
    #[must_use]
    pub const fn id(self) -> NodeId {
        self.id
    }

    /// What this node is.
    #[must_use]
    pub fn kind(self) -> NodeKind {
        self.data().kind
    }

    /// The exact byte range of this node. Internal spans are the union of
    /// their children; the root spans the whole file.
    #[must_use]
    pub fn span(self) -> Span {
        self.data().span
    }

    /// The parent node, `None` for the root.
    #[must_use]
    pub fn parent(self) -> Option<Node<'a>> {
        self.data().parent.map(|id| Node {
            tree: self.tree,
            id,
        })
    }

    /// The direct children, in source order.
    pub fn children(self) -> impl ExactSizeIterator<Item = Node<'a>> + 'a {
        let tree = self.tree;
        self.data()
            .children
            .iter()
            .map(move |&id| Node { tree, id })
    }

    /// The direct children that are internal nodes (not leaves).
    pub fn child_nodes(self) -> impl Iterator<Item = Node<'a>> + 'a {
        self.children().filter(|node| !node.is_leaf())
    }

    /// The direct children that are leaves, in source order.
    pub fn child_leaves(self) -> impl Iterator<Item = Node<'a>> + 'a {
        self.children().filter(|node| node.is_leaf())
    }

    /// The number of direct children.
    #[must_use]
    pub fn child_count(self) -> usize {
        self.data().children.len()
    }

    /// Whether this node is a leaf (a token, trivia, the BOM or a lexical
    /// error region), i.e. has no children.
    #[must_use]
    pub fn is_leaf(self) -> bool {
        self.data().children.is_empty() && self.data().elem.is_some()
    }

    /// Whether this node is a **zero-width** structural node (no text of
    /// its own). Only the root of an empty file can be zero-width in
    /// practice; the flag exists so consumers never mistake such a node for
    /// a covering one.
    #[must_use]
    pub fn is_zero_width(self) -> bool {
        self.data().children.is_empty() && self.data().elem.is_none()
    }

    /// For a token leaf, its lexer token kind.
    #[must_use]
    pub fn token_kind(self) -> Option<TokenKind> {
        match self.data().kind {
            NodeKind::Token(kind) => Some(kind),
            _ => None,
        }
    }

    /// For a leaf, the lexer element it mirrors (or mirrors a part of, for a
    /// split `>` leaf).
    #[must_use]
    pub fn element(self) -> Option<&'a LexElement> {
        let index = self.data().elem? as usize;
        self.tree.lexed.elements().get(index)
    }

    /// The lexical errors attached to the mirrored element (empty for
    /// internal nodes and clean leaves).
    #[must_use]
    pub fn errors(self) -> &'a [LexErrorCode] {
        &self.data().errors
    }

    /// Whether the mirrored element carries at least one lexical error.
    #[must_use]
    pub fn has_lex_errors(self) -> bool {
        !self.data().errors.is_empty()
    }

    /// Whether this node is an error node: a refusal node of an unsupported
    /// syntax family, or a generic recovery [`NodeKind::Error`] region.
    /// Every error node carries at least one `PARSE` diagnostic.
    #[must_use]
    pub fn is_error_node(self) -> bool {
        self.data().kind.is_error_node()
    }

    /// Whether this leaf is trivia (whitespace, a line terminator or a
    /// comment).
    #[must_use]
    pub fn is_trivia(self) -> bool {
        self.data().kind.is_trivia()
    }

    /// The exact source text this node covers, resolved through `source`.
    ///
    /// Returns `None` when `source` is not the parsed file.
    #[must_use]
    pub fn text(self, source: &SourceFile) -> Option<&str> {
        if source.id() != self.tree.file {
            return None;
        }
        source.text_of(self.data().span)
    }

    /// This node and all its descendants in pre-order.
    #[must_use]
    pub fn descendants(self) -> Descendants<'a> {
        Descendants {
            tree: self.tree,
            stack: vec![self.id],
        }
    }
}

/// Pre-order iterator over a node's subtree; see [`Node::descendants`].
#[derive(Clone, Debug)]
pub struct Descendants<'a> {
    tree: &'a ParsedFile,
    stack: Vec<NodeId>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Node<'a>> {
        let id = self.stack.pop()?;
        let data = &self.tree.nodes[id.0 as usize];
        self.stack.extend(data.children.iter().rev().copied());
        Some(Node {
            tree: self.tree,
            id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_index_is_dense_from_zero() {
        assert_eq!(NodeId(0).index(), 0);
        assert_eq!(NodeId(7).index(), 7);
    }

    #[test]
    fn split_kinds_are_accepted() {
        assert!(leaf_kind_matches(
            NodeKind::Token(TokenKind::Gt),
            ElementKind::Token(TokenKind::Shr)
        ));
        assert!(leaf_kind_matches(
            NodeKind::Token(TokenKind::Gt),
            ElementKind::Token(TokenKind::Ushr)
        ));
        assert!(!leaf_kind_matches(
            NodeKind::Token(TokenKind::Gt),
            ElementKind::Token(TokenKind::GtEq)
        ));
    }
}
