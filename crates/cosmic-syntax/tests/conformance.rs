//! Conformance: every recognised production family of the accepted grammar
//! (`docs/spec/grammar.ebnf` §3) parses into structure without diagnostics,
//! and the disambiguation and line rules of
//! `docs/spec/g1-s-syntax-decision-package.md` §5 hold.
//!
//! Every fixture is hand-written for this suite; no fixture is mechanically
//! translated from another implementation. The `recognised` helper asserts
//! three things at once for every fixture: no `LEX`/`PARSE` diagnostics, a
//! lossless leaf stream, and a byte-for-byte reprint.

mod common;

use common::{
    count_of_kind, first_of_kind, nodes_of_kind, parse_codes, parse_text, recognised, source_of,
};
use cosmic_lexer::TokenKind;
use cosmic_syntax::{Node, NodeKind};

/// The token kinds among a node's direct children, in order. For a
/// `BinaryExpr` this is exactly its operator token(s): `+`, or the two/three
/// `>` parts of a re-joined `>>`/`>>>`.
fn operator_tokens(node: Node<'_>) -> Vec<TokenKind> {
    node.children()
        .filter_map(|child| child.token_kind())
        .collect()
}

// --- Units, top level, functions ----------------------------------------

#[test]
fn empty_and_trivia_only_files_are_recognised() {
    for text in [
        "",
        "\n",
        "  \n\t// line comment\n",
        "/* block */\n\n",
        "\u{feff}",
        "\u{feff}\n\n/* after bom */",
    ] {
        let (sources, parsed) = recognised(text);
        let source = source_of(&sources, &parsed);
        let root = parsed.root();
        assert_eq!(root.kind(), NodeKind::File, "{text:?}");
        assert_eq!(root.span().start(), 0, "{text:?}");
        assert_eq!(root.span().end(), text.len() as u32, "{text:?}");
        assert!(root.parent().is_none(), "{text:?}");
        assert!(source.len() == text.len() as u32);
    }
}

#[test]
fn package_and_imports_are_recognised() {
    let text = "package demo.app\nimport demo.one.Two\nimport demo.three\nmain() {}\n";
    let (_, parsed) = recognised(text);
    assert_eq!(count_of_kind(&parsed, NodeKind::PackageDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ImportDecl), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::QualifiedName), 3);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::TopLevelDecl), 1);
}

#[test]
fn function_declarations_cover_both_parameter_orders_and_both_bodies() {
    let text = "\
main() {}\n\
Int add(Int a, Int b) { return a + b }\n\
addOne(Int a) = a + 1\n\
addTwo(a: Int) = a + 2\n\
void log(String message) { call(message) }\n\
withDefault(Int a = 1) = a\n\
";
    let (_, parsed) = recognised(text);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 6);
    assert_eq!(count_of_kind(&parsed, NodeKind::ParameterList), 6);
    assert_eq!(count_of_kind(&parsed, NodeKind::Parameter), 6);
    assert_eq!(count_of_kind(&parsed, NodeKind::ReturnStmt), 1);
    // Block bodies: `main`, `add` and `log`; the other three use the
    // expression body (`= expression`).
    assert_eq!(count_of_kind(&parsed, NodeKind::Block), 3);
}

#[test]
fn type_declarations_cover_class_interface_record_enum_and_sealed() {
    let text = "\
class Circle extends Shape implements Drawable, Named {\n\
  Int radius = 0\n\
  Int bare\n\
  constructor(Int radius) { call() }\n\
  area() = 3\n\
  static Int count = 0\n\
}\n\
interface Drawable { draw() { call() } }\n\
record Point(Int x, Int y)\n\
record Box { Int value }\n\
enum Color { RED, GREEN(1, 2), BLUE }\n\
sealed class Leaf extends Node { }\n\
";
    let (_, parsed) = recognised(text);
    assert_eq!(count_of_kind(&parsed, NodeKind::ClassDecl), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::InterfaceDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::RecordDecl), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::PositionalRecord), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::RecordBody), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::EnumDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::EnumConstant), 3);
    assert_eq!(count_of_kind(&parsed, NodeKind::ConstructorDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FieldDecl), 4);
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::Supertypes), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::ClassBody), 3);
    assert_eq!(count_of_kind(&parsed, NodeKind::Member), 7);
}

// --- Statements ----------------------------------------------------------

#[test]
fn statement_families_are_recognised() {
    let text = "\
run() {\n\
  val a = 1\n\
  var b: Int = 2\n\
  b = b + a\n\
  if (a < b) { call() } else call2()\n\
  while (a < 10) a++\n\
  do a-- while (a > 0)\n\
  for (var i = 0; i < 10; i = i + 1) { call(i) }\n\
  for (item in items) call(item)\n\
  for (val x in xs) call2(x)\n\
  try { call() } catch (Error e) { call3() } finally { call4() }\n\
  switch (a) {\n\
    case 1: call()\n\
    case Circle c if c.radius > 0: call2()\n\
    case Point(var x, var y): call3()\n\
    case Int: call4()\n\
    default: call5()\n\
  }\n\
  break\n\
  continue\n\
  throw boom()\n\
  return\n\
  ;\n\
}\n\
";
    let (_, parsed) = recognised(text);
    assert_eq!(count_of_kind(&parsed, NodeKind::VarDeclStmt), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::AssignExpr), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::IfStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::WhileStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::DoWhileStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ForStmt), 3);
    assert_eq!(count_of_kind(&parsed, NodeKind::ForInit), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::TryStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::CatchClause), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FinallyClause), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::SwitchStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::SwitchStmtArm), 5);
    assert_eq!(count_of_kind(&parsed, NodeKind::Guard), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::BindingPattern), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::DestructuringPattern), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::PatternField), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::BreakStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ContinueStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ThrowStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ReturnStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::EmptyStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::Block), 6);
}

#[test]
fn unlabelled_break_and_continue_are_recognised() {
    let (_, parsed) =
        recognised("main() {\n  while (ready) {\n    if (done) break\n    continue\n  }\n}\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::BreakStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ContinueStmt), 1);
}

#[test]
fn else_catch_finally_and_while_bind_across_lines() {
    let text = "\
main() {\n\
  if (a)\n\
    call()\n\
  else\n\
    call2()\n\
  do\n\
    call3()\n\
  while (a)\n\
  try\n\
    { call4() }\n\
  catch (Error e)\n\
    { call5() }\n\
  finally\n\
    { call6() }\n\
}\n\
";
    let (_, parsed) = recognised(text);
    assert_eq!(count_of_kind(&parsed, NodeKind::IfStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::DoWhileStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::CatchClause), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::FinallyClause), 1);
}

#[test]
fn return_does_not_cross_lines() {
    let (_, parsed) = recognised("main() {\n  return\n  call()\n}\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::ReturnStmt), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ExprStmt), 1);
}

#[test]
fn switch_labels_cover_expression_binding_destructuring_and_type_forms() {
    let text = "\
run() {\n\
  switch (shape) {\n\
    case 1 + 2: call()\n\
    case Circle c if c.radius > 0: call2()\n\
    case Point(var x, var y): call3()\n\
    case List<Int>: call4()\n\
    case Int: call5()\n\
    default: call6()\n\
  }\n\
}\n\
";
    let (_, parsed) = recognised(text);
    assert_eq!(count_of_kind(&parsed, NodeKind::SwitchStmtArm), 6);
    assert_eq!(count_of_kind(&parsed, NodeKind::BindingPattern), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::DestructuringPattern), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::PatternField), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::Guard), 1);
    // `List<Int>` is a type reference, never a generic call.
    assert_eq!(count_of_kind(&parsed, NodeKind::TypeArguments), 1);
}

#[test]
fn parameter_defaults_are_recognised() {
    let (_, parsed) = recognised("f(Int a = 1, b: Int = 2) = a + b\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::Parameter), 2);
    assert_eq!(
        common::token_kinds(&parsed)
            .into_iter()
            .filter(|kind| *kind == TokenKind::IntLiteral)
            .count(),
        2
    );
}

// --- Expressions ---------------------------------------------------------

#[test]
fn expression_families_are_recognised() {
    let text = "\
expressions() {\n\
  val a = 1 + 2 * 3 - 4 / 5 % 6\n\
  val b = a << 2 >> 3 >>> 4\n\
  val c = a == b && a != 2 && a < 3 && a <= 4 && a > 5 && a >= 6\n\
  val d = a && b || !c\n\
  val e = a & b | c ^ a\n\
  val f = -a + -b\n\
  val g = a instanceof Int && b as Long\n\
  val h = a.b.c(1, 2).d[3].e++\n\
  val i = if (a) 1 else 2\n\
  val j = switch (a) { case 1 -> 10 default -> 20 }\n\
  val k = new Box(1, 2)\n\
  val l = (a + b) * c\n\
  val m = ++a + b--\n\
  val n = spawn task()\n\
  val o = await task()\n\
  val p = identity<Int>(1)\n\
  val q = a < b\n\
  val r = a < b > c\n\
  val s = items.map { x: Int -> x + 1 }\n\
}\n\
";
    let (_, parsed) = recognised(text);
    assert_eq!(count_of_kind(&parsed, NodeKind::VarDeclStmt), 19);
    assert!(first_of_kind(&parsed, NodeKind::BinaryExpr).is_some());
    assert!(first_of_kind(&parsed, NodeKind::UnaryExpr).is_some());
    assert!(first_of_kind(&parsed, NodeKind::PostfixExpr).is_some());
    assert_eq!(count_of_kind(&parsed, NodeKind::IfExpr), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::SwitchExpr), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::SwitchExprArm), 2);
    assert_eq!(count_of_kind(&parsed, NodeKind::NewExpr), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::ParenExpr), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::TypeArguments), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::LambdaExpr), 1);
    // `c(1, 2)`, `task()` (spawn), `task()` (await), `identity<Int>(1)` and
    // `new Box(1, 2)`.
    assert_eq!(count_of_kind(&parsed, NodeKind::ArgumentList), 5);
}

#[test]
fn multiplication_binds_tighter_than_addition() {
    let (_, parsed) = recognised("f() { val x = a + b * c }\n");
    let binaries = nodes_of_kind(&parsed, NodeKind::BinaryExpr);
    assert_eq!(binaries.len(), 2);
    let plus = binaries
        .iter()
        .copied()
        .find(|node| operator_tokens(*node) == vec![TokenKind::Plus])
        .expect("a `+` node");
    let star = binaries
        .iter()
        .copied()
        .find(|node| operator_tokens(*node) == vec![TokenKind::Star])
        .expect("a `*` node");
    // `a + (b * c)`: the multiplication is nested inside the addition.
    assert!(star.span().start() > plus.span().start());
    assert!(star.span().end() == plus.span().end());
}

#[test]
fn addition_is_left_associative() {
    let (_, parsed) = recognised("f() { val x = a - b - c }\n");
    let mut binaries = nodes_of_kind(&parsed, NodeKind::BinaryExpr);
    assert_eq!(binaries.len(), 2);
    binaries.sort_by_key(|node| node.span().end());
    let (inner, outer) = (binaries[0], binaries[1]);
    // `(a - b) - c`: both start at `a`; the outer one extends past the inner.
    assert_eq!(inner.span().start(), outer.span().start());
    assert!(inner.span().end() < outer.span().end());
}

#[test]
fn shift_operators_rejoin_presplit_tokens() {
    let (_, parsed) = recognised("f() { val x = a >> b >>> c }\n");
    let binaries = nodes_of_kind(&parsed, NodeKind::BinaryExpr);
    assert_eq!(binaries.len(), 2);
    let shapes: Vec<Vec<TokenKind>> = binaries.iter().map(|node| operator_tokens(*node)).collect();
    assert!(shapes.contains(&vec![TokenKind::Gt, TokenKind::Gt]));
    assert!(shapes.contains(&vec![TokenKind::Gt, TokenKind::Gt, TokenKind::Gt]));
}

#[test]
fn generic_call_and_comparison_disambiguation() {
    // `identity<Int>(1)`: type arguments followed by the call.
    let (_, parsed) = recognised("f() { val x = identity<Int>(1) }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::TypeArguments), 1);

    // `<` without a following call is a comparison, never type arguments.
    let (_, parsed) = recognised("f() { val x = a < b }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::TypeArguments), 0);
    let binaries = nodes_of_kind(&parsed, NodeKind::BinaryExpr);
    assert_eq!(binaries.len(), 1);
    assert_eq!(operator_tokens(binaries[0]), vec![TokenKind::Lt]);

    // `a < b > c` is a left-associative comparison, not `a<b>` arguments.
    let (_, parsed) = recognised("f() { val x = a < b > c }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::TypeArguments), 0);
    let mut binaries = nodes_of_kind(&parsed, NodeKind::BinaryExpr);
    assert_eq!(binaries.len(), 2);
    binaries.sort_by_key(|node| node.span().end());
    assert_eq!(operator_tokens(binaries[0]), vec![TokenKind::Lt]);
    assert_eq!(operator_tokens(binaries[1]), vec![TokenKind::Gt]);
}

#[test]
fn nested_type_arguments_close_with_shared_shift_tokens() {
    let (_, parsed) = recognised("f() { val x: Map<String, List<Int>> = m }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::TypeArguments), 2);

    let (_, parsed) = recognised("f() { val x: A<B<C<D>>> = m }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::TypeArguments), 3);
}

#[test]
fn lambda_forms_are_recognised() {
    // Parenthesised lambda with a typed parameter and an expression body.
    let (_, parsed) = recognised("f() { val twice = (x: Int) -> x * 2 }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::LambdaExpr), 1);
    // One parameter list for `f`, one for the lambda.
    assert_eq!(count_of_kind(&parsed, NodeKind::ParameterList), 2);

    // Empty parameters and a block body.
    let (_, parsed) = recognised("f() { val run = () -> { call() } }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::LambdaExpr), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::Block), 2);

    // Brace lambda with typed parameters, in expression position.
    let (_, parsed) = recognised("f() { val add = { a: Int, b: Int -> a + b } }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::LambdaExpr), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::LambdaParams), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::Parameter), 2);
}

#[test]
fn trailing_lambda_attaches_only_on_the_same_line() {
    // Same line: the brace lambda attaches to the call.
    let (_, parsed) = recognised("f() { items.each { x: Int -> call(x) } }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::LambdaExpr), 1);

    // Next line: the `{` starts a block statement, not a trailing lambda.
    let (_, parsed) = recognised("f() { items.each\n  { call(x) } }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::LambdaExpr), 0);
    assert_eq!(count_of_kind(&parsed, NodeKind::Block), 2);
}

#[test]
fn untyped_lambda_parameters_are_syntax_errors() {
    // `(x) -> x` is owed by G1-S §5.1 item 4: the region is scanned as
    // lambda parameters, and the untyped parameter (S1) is a syntax error.
    let (_, parsed) = parse_text("f() { val g = (x) -> x }\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE002"]);

    // The lambda arrow may sit on the next line; the scan is not line-gated.
    let (_, parsed) = recognised("f() { val g = (x: Int)\n  -> x }\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::LambdaExpr), 1);
}

// --- Statement-versus-declaration and line rules -------------------------

#[test]
fn same_line_brace_after_a_call_is_a_nested_declaration() {
    // Grammar order: `nested-function-decl` precedes `expression-stmt`, and
    // the body `{` starts on the same line.
    let (_, parsed) = recognised("outer() {\n  inner() { call() }\n}\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 2);
}

#[test]
fn next_line_brace_after_a_call_is_a_block_statement() {
    let (_, parsed) = recognised("outer() {\n  init()\n  { call() }\n}\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::FunctionDecl), 1);
    assert_eq!(count_of_kind(&parsed, NodeKind::Block), 2);
}

#[test]
fn continuation_operators_do_not_cross_lines() {
    let (_, parsed) = parse_text("main() {\n  val a = b\n  + c\n}\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE002"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::BinaryExpr), 0);
}

#[test]
fn assignment_does_not_cross_lines() {
    let (_, parsed) = parse_text("main() {\n  x\n  = 1\n}\n");
    assert_eq!(parse_codes(&parsed), vec!["PARSE002"]);
    assert_eq!(count_of_kind(&parsed, NodeKind::AssignExpr), 0);
}

#[test]
fn continuations_inside_parentheses_may_cross_lines() {
    let (_, parsed) = recognised("main() {\n  call(\n    a\n    + b\n  )\n}\n");
    assert_eq!(count_of_kind(&parsed, NodeKind::BinaryExpr), 1);
}
