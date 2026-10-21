use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{run, Value};
use v1_compiler::v1_std_core::{diagnostic_to_message, diagnostic_to_span, source_text_at};

fn compile(
    path: &str,
    source: &str,
) -> Rc<v1_compiler::v1_compiler_compile::ResolvedPipelineResult> {
    compile_to_resolved(Rc::new(im::vector![Rc::new(SourceFile {
        path: path.to_string(),
        content: source.to_string(),
    })]))
}

fn evaluate(path: &str, source: &str) -> Value {
    let resolved = compile(path, source);
    let graph = resolved
        .graph
        .clone()
        .unwrap_or_else(|| panic!("explicit fixture must resolve: {:?}", resolved.diagnostics));
    run(&graph, resolved.source_indices.clone(), "f")
        .unwrap_or_else(|e| panic!("explicit fixture must execute: {e:?}"))
}

#[test]
fn explicit_subtraction_and_unary_statement_keep_their_values() {
    let explicit_infix = "module explicit_infix\nfn f() -> Int {\n  10 -\n    1\n}\n";
    assert!(
        matches!(
            evaluate("explicit_infix.dag", explicit_infix),
            Value::Int(9)
        ),
        "an infix '-' placed before the newline must still evaluate as subtraction"
    );

    let explicit_unary = "module explicit_unary\nfn f() -> Int {\n  let a = 10\n  (-1)\n  a\n}\n";
    assert!(
        matches!(evaluate("explicit_unary.dag", explicit_unary), Value::Int(10)),
        "a parenthesized unary-negation statement must remain admitted and leave the final value unchanged"
    );
}

#[test]
fn ambiguous_prefix_infix_newline_boundary_refuses_at_both_spans() {
    let ambiguous = "module ambiguous\nfn f() -> Int {\n  10\n  - 1\n}\n";
    let refused = compile("ambiguous.dag", ambiguous);
    let expected_message = "ambiguous newline before prefix/infix operator '-': move the infix operator before the newline or parenthesize the unary expression";
    let matching: Vec<_> = refused
        .diagnostics
        .iter()
        .filter(|d| diagnostic_to_message(d.diagnostic.clone()) == expected_message)
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "expected one exact ambiguity refusal, got: {:?}",
        refused.diagnostics
    );
    let span = diagnostic_to_span(matching[0].diagnostic.clone());
    let index = refused
        .source_indices
        .get("ambiguous.dag")
        .expect("fixture source index");
    assert_eq!(source_text_at(index.clone(), span), "10\n  -");
}
