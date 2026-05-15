//! PATH X receipt: `dsl/std/unicode.dag` `char_in_class` is `ArrowBody::UserDefined`
//! after expression-bodied authoring, and evaluates through the eager host evaluator.
//!
//! **Tokenizer bridge (ROADMAP parity row):** compares `char_in_class` (fixture wrappers) to
//! `tokenize_generated::byte_matches` on `0..=127` (`char_in_class` interpreter parity — PR #693).

use crate::common::cached_compile_to_dag;

use v3_compiler::dag::LiteralBits;
use v3_compiler::dag::{ArrowBody, Behavior, TypeConnective};
use v3_compiler::evaluator::{
    evaluate_body, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder, Value,
};
use v3_compiler::{byte_matches, ScannerCharClass};

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

fn eval_utf8_char_class_predicate(
    dag: &v3_compiler::dag::Dag,
    fn_name: &str,
    byte: u8,
    strategy: &EvalStrategy,
) -> bool {
    let nid = bind_node_for_user_fn(dag, fn_name);
    let Behavior::Bind(bind) = dag.node(nid) else {
        panic!("`{fn_name}` must lower to a Bind behavior");
    };
    assert_eq!(
        bind.params.len(),
        1,
        "`{fn_name}` must be unary (Char -> Bool) for parity probes"
    );

    let mut stack = EvalStateStack::with_root_frame(EvalFrame::empty());
    stack
        .bind_top(
            bind.params[0],
            Value::LiteralValue(LiteralBits::Int(format!("{}", i32::from(byte)))),
        )
        .expect("bind arity-1 Char literal Char port");

    let v = evaluate_body(dag, nid, &mut stack, strategy.clone())
        .unwrap_or_else(|e| panic!("eval {fn_name}: {e:?}"));
    match v {
        Value::LiteralValue(LiteralBits::Bool(b)) => b,
        other => panic!("`{fn_name}` must return Bool literal, got {other:?}"),
    }
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
        decl.span.file, "dsl/std/unicode.dag",
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
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());

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

/// ROADMAP: `char_in_class` interpreter parity (tokenizer bridge finish) — ASCII `0..=127`.
#[test]
fn path_b_brief3_char_in_class_matches_codegen_byte_matches_on_ascii() {
    let dag = cached_compile_to_dag(FIXTURE, FIXTURE_PATH);
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());

    let strategy = EvalStrategy::ApplicativeOrder {
        input_order: InputEvaluationOrder::LeftFirst,
    };

    for byte in 0_u8..=127 {
        for (scanner, fn_name) in [
            (ScannerCharClass::Whitespace, "brief3_membership_whitespace"),
            (ScannerCharClass::Digit, "brief3_membership_digit"),
            (
                ScannerCharClass::IdentStart,
                "brief3_membership_ident_start",
            ),
            (
                ScannerCharClass::IdentContinue,
                "brief3_membership_ident_continue",
            ),
        ] {
            let codegen = byte_matches(byte, scanner);
            let lowered = eval_utf8_char_class_predicate(&dag, fn_name, byte, &strategy);
            assert_eq!(
                codegen, lowered,
                "divergence byte {byte:#04x} scanner {scanner:?}; \
                 codegen(byte_matches) vs substrate char_in_class via `{fn_name}`"
            );
        }
    }
}
