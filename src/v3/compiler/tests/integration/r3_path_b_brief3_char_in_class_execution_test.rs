//! PATH X receipt: `dsl/std/unicode.dag` `char_in_class` is `ArrowBody::UserDefined`
//! after expression-bodied authoring, and evaluates through the eager host evaluator.

use crate::common::cached_compile_to_dag;

use v3_compiler::dag::{ArrowBody, TypeConnective};
use v3_compiler::evaluator::{
    evaluate_body, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder, Value,
};
use v3_compiler::dag::LiteralBits;

const FIXTURE: &str = include_str!("../fixtures/r3_path_b_brief3_char_in_class_exec.dag");
const FIXTURE_PATH: &str = "src/v3/compiler/tests/fixtures/r3_path_b_brief3_char_in_class_exec.dag";

fn bind_node_for_user_fn(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::NodeId {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("missing fn `{name}`"));
    let TypeConnective::Arrow { body, .. } = &decl.connective else {
        panic!("`{name}` must be an Arrow declaration");
    };
    let ArrowBody::UserDefined(bind_id) = body else {
        panic!("`{name}` expected UserDefined Arrow body, got {body:?}");
    };
    bind_id.node_id()
}

#[test]
fn path_b_brief3_char_in_class_authority_is_user_defined() {
    let dag = cached_compile_to_dag(FIXTURE, FIXTURE_PATH);
    assert!(
        dag.diagnostics().is_empty(),
        "fixture should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let decl = dag
        .declaration_by_name("char_in_class")
        .expect("bootstrap must expose unicode.dag char_in_class");
    assert_eq!(
        decl.span.file,
        "dsl/std/unicode.dag",
        "witness must be the dsl/std authority declaration"
    );
    let TypeConnective::Arrow { body, .. } = &decl.connective else {
        panic!("char_in_class must lower as Arrow");
    };
    assert!(
        matches!(body, ArrowBody::UserDefined(_)),
        "PATH X expects executable body, got {body:?}"
    );
}

#[test]
fn path_b_brief3_char_in_class_executes_via_evaluator() {
    let dag = cached_compile_to_dag(FIXTURE, FIXTURE_PATH);
    assert!(
        dag.diagnostics().is_empty(),
        "{:?}",
        dag.diagnostics()
    );

    let strategy = EvalStrategy::ApplicativeOrder {
        input_order: InputEvaluationOrder::LeftFirst,
    };

    let mut state = EvalStateStack::with_root_frame(EvalFrame::empty());
    let v = evaluate_body(
        &dag,
        bind_node_for_user_fn(&dag, "receipt_letter_is_ident_start"),
        &mut state,
        strategy.clone(),
    )
    .expect("eval letter / IdentStart");
    assert_eq!(v, Value::LiteralValue(LiteralBits::Bool(true)));

    let mut state = EvalStateStack::with_root_frame(EvalFrame::empty());
    let v = evaluate_body(
        &dag,
        bind_node_for_user_fn(&dag, "receipt_digit_not_ident_start"),
        &mut state,
        strategy.clone(),
    )
    .expect("eval digit / IdentStart");
    assert_eq!(v, Value::LiteralValue(LiteralBits::Bool(false)));

    let mut state = EvalStateStack::with_root_frame(EvalFrame::empty());
    let v = evaluate_body(
        &dag,
        bind_node_for_user_fn(&dag, "receipt_space_not_ident_start"),
        &mut state,
        strategy.clone(),
    )
    .expect("eval space / IdentStart");
    assert_eq!(v, Value::LiteralValue(LiteralBits::Bool(false)));

    let mut state = EvalStateStack::with_root_frame(EvalFrame::empty());
    let v = evaluate_body(
        &dag,
        bind_node_for_user_fn(&dag, "receipt_digit_is_digit"),
        &mut state,
        strategy,
    )
    .expect("eval digit / Digit");
    assert_eq!(v, Value::LiteralValue(LiteralBits::Bool(true)));
}
