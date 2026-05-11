//! R3 Class 2 gap representative: top-level function-valued `data`
//! executes through the public evaluator as a callable declaration.

use crate::common::{cached_compile_any, cached_compile_to_dag};
use v3_compiler::dag::{
    literal_bits_int, ArrowBody, Behavior, PortState, TransformTarget, TypeConnective, ValueBody,
};
use v3_compiler::evaluator::{
    evaluate_body, EvalFrame, EvalStateStack, EvalStrategy, InputEvaluationOrder, Value,
};

const SOURCE: &str = include_str!("../fixtures/r3_class_2_function_valued_data.dag");
const FILE: &str = "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data.dag";
const RECURSIVE_SOURCE: &str = r#"
data countdown: fn(Int) -> Int = |n| countdown(n)

fn use_countdown() -> Int = countdown(1)
"#;
const DATA_DATA_CYCLE_SOURCE: &str = r#"
data evenish: fn(Int) -> Int = |n| oddish(n)
data oddish: fn(Int) -> Int = |n| evenish(n)

fn use_evenish() -> Int = evenish(1)
"#;
const DATA_FN_CYCLE_SOURCE: &str = r#"
data entry: fn(Int) -> Int = |n| helper(n)

fn helper(n: Int) -> Int = entry(n)
fn use_entry() -> Int = entry(1)
"#;

fn bind_node_id_for_fn(dag: &v3_compiler::dag::Dag, name: &str) -> v3_compiler::dag::NodeId {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("missing `{name}`"));
    let TypeConnective::Arrow { body, .. } = &decl.connective else {
        panic!("`{name}` must lower as an Arrow");
    };
    let ArrowBody::UserDefined(bind_id) = body else {
        panic!("`{name}` must have an executable UserDefined body, got {body:?}");
    };
    bind_id.node_id()
}

fn assert_rejected_data_lambda(dag: &v3_compiler::dag::Dag, name: &str) {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("function-valued data declaration `{name}`"));
    assert!(
        matches!(decl.value_body, Some(ValueBody::Unparsed(_))),
        "failed data lambda `{name}` must retain an Unparsed body marker"
    );
    let TypeConnective::Arrow {
        body: ArrowBody::UserDefined(bind_id),
        ..
    } = &decl.connective
    else {
        panic!("rejected data lambda `{name}` must keep a poisoned executable Arrow");
    };
    assert!(
        matches!(
            dag.port(bind_id.bind(dag).value).state(),
            PortState::Unresolved
        ),
        "rejected data lambda `{name}` must poison callers through an unresolved body"
    );
}

#[test]
fn substrate_gap_function_valued_data_executes_through_evaluator() {
    let dag = cached_compile_to_dag(SOURCE, FILE);
    assert!(
        dag.diagnostics().is_empty(),
        "representative must compile without diagnostics: {:?}",
        dag.diagnostics()
    );

    let add_one = dag
        .declaration_by_name("add_one")
        .expect("function-valued data declaration")
        .id;
    let add_one_decl = dag.declaration(add_one);
    assert!(
        add_one_decl.meta_tag.is_some(),
        "`add_one` must retain the data declaration's type-annotation edge"
    );
    assert!(
        add_one_decl.value_body.is_none(),
        "executable function-valued data must not keep an opaque ValueBody scaffold"
    );
    assert!(
        matches!(
            &add_one_decl.connective,
            TypeConnective::Arrow {
                body: ArrowBody::UserDefined(_),
                ..
            }
        ),
        "function-valued data must carry an executable Arrow body"
    );
    let user_defined_arrow_names: Vec<&str> = dag
        .declarations()
        .iter()
        .filter(|decl| decl.span.file == FILE)
        .filter_map(|decl| {
            matches!(
                &decl.connective,
                TypeConnective::Arrow {
                    body: ArrowBody::UserDefined(_),
                    ..
                }
            )
            .then_some(decl.name.as_deref())
            .flatten()
        })
        .collect();
    assert_eq!(
        user_defined_arrow_names,
        vec!["add_one", "test_function_valued_data"],
        "only the named data callable and named caller should carry executable Arrow bodies"
    );
    assert!(
        dag.nodes().iter().any(|node| {
            matches!(
                node,
                Behavior::Transform(t)
                    if matches!(&t.target, TransformTarget::Callable(target) if *target == add_one)
            )
        }),
        "`add_one(41)` must lower to TransformTarget::Callable(add_one), not a Rust-side bypass"
    );

    let entry = bind_node_id_for_fn(&dag, "test_function_valued_data");
    let mut state = EvalStateStack::with_root_frame(EvalFrame::empty());
    let strategy = EvalStrategy::ApplicativeOrder {
        input_order: InputEvaluationOrder::LeftFirst,
    };
    let value = evaluate_body(&dag, entry, &mut state, strategy)
        .expect("function-valued data should execute through evaluator");

    assert_eq!(value, Value::LiteralValue(literal_bits_int(42)));
}

#[test]
fn function_valued_data_recursion_fails_closed() {
    let dag = cached_compile_any(
        RECURSIVE_SOURCE,
        "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_recursive.dag",
    );
    assert_rejected_data_lambda(&dag, "countdown");
}

#[test]
fn function_valued_data_cycles_fail_closed() {
    for (source, file, data_name) in [
        (
            DATA_DATA_CYCLE_SOURCE,
            "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_data_cycle.dag",
            "evenish",
        ),
        (
            DATA_FN_CYCLE_SOURCE,
            "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_fn_cycle.dag",
            "entry",
        ),
    ] {
        let dag = cached_compile_any(source, file);
        assert_rejected_data_lambda(&dag, data_name);
    }
}
