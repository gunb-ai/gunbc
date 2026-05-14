//! R3 Class 2 gap representative: top-level function-valued `data`
//! executes through the public evaluator as a callable declaration.

use crate::common::{cached_compile_any, cached_compile_to_dag};
use v3_compiler::dag::{
    literal_bits_int, ArrowBody, Behavior, BindEmitParticipation, PortState, TransformTarget,
    TypeConnective, ValueBody,
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
const NESTED_RECURSIVE_SOURCE: &str = r#"
data countdown: fn(Int) -> Int = |n| if true then countdown(n) else 0

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
const DATA_PATH_CYCLE_SOURCE: &str = r#"
type Fns { next: fn(Int) -> Int }

data fns: Fns = { next: entry }
data entry: fn(Int) -> Int = |n| fns.next(n)

fn use_entry() -> Int = entry(1)
"#;
const MALFORMED_LAMBDA_SOURCE: &str = r#"
data wrong_arity: fn(Int) -> Int = |x, y| x

fn use_wrong_arity() -> Int = wrong_arity(1)
"#;
const SHADOWED_DATA_NAME_SOURCE: &str = r#"
data apply_one: fn(fn(Int) -> Int) -> Int = |apply_one| apply_one(1)

fn inc(n: Int) -> Int = n + 1
fn use_apply_one() -> Int = apply_one(inc)
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
        inputs,
        body: ArrowBody::UserDefined(bind_id),
        ..
    } = &decl.connective
    else {
        panic!("rejected data lambda `{name}` must keep a poisoned executable Arrow");
    };
    assert_eq!(
        bind_id.bind(dag).params.len(),
        inputs.len(),
        "rejected data lambda `{name}` must preserve callable arity"
    );
    assert!(
        matches!(
            dag.port(bind_id.bind(dag).value).state(),
            PortState::Unresolved
        ),
        "rejected data lambda `{name}` must poison callers through an unresolved body"
    );
}

fn assert_rejected_data_lambda_in_file(dag: &v3_compiler::dag::Dag, file: &str, name: &str) {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("function-valued data declaration `{name}` in {file}"));
    assert!(
        matches!(decl.value_body, Some(ValueBody::Unparsed(_))),
        "failed data lambda `{name}` in {file} must retain an Unparsed body marker; got {:?}",
        decl.value_body
    );
    assert_rejected_data_lambda(dag, name);
}

fn assert_user_callable_bind_names(dag: &v3_compiler::dag::Dag, file: &str, expected: &[&str]) {
    let mut names: Vec<&str> = dag
        .nodes()
        .iter()
        .filter_map(|node| match node {
            Behavior::Bind(bind)
                if bind.span.file == file
                    && bind.emit_participation() == Some(BindEmitParticipation::UserCallable) =>
            {
                Some(bind.name.as_str())
            }
            _ => None,
        })
        .collect();
    names.sort_unstable();
    let mut expected_names = expected.to_vec();
    expected_names.sort_unstable();
    assert_eq!(
        names, expected_names,
        "rejected data-lambda cycles must not leave pre-rejection lambda binds"
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
        vec![
            "add_one",
            "gate61_empty_int_witnesses",
            "gate61_int_witnesses",
            "report_int",
            "test_function_valued_data",
            "test_function_valued_dimension_report",
        ],
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
fn substrate_gap_function_valued_data_produces_dimension_report() {
    let dag = cached_compile_to_dag(SOURCE, FILE);
    assert!(
        dag.diagnostics().is_empty(),
        "representative must compile without diagnostics: {:?}",
        dag.diagnostics()
    );

    let report_int = dag
        .declaration_by_name("report_int")
        .expect("function-valued DimensionReport data")
        .id;
    let report_int_decl = dag.declaration(report_int);
    assert!(
        report_int_decl.value_body.is_none(),
        "`report_int` must lower as executable function-valued data"
    );
    assert!(
        matches!(
            &report_int_decl.connective,
            TypeConnective::Arrow {
                body: ArrowBody::UserDefined(_),
                ..
            }
        ),
        "`report_int` must carry a UserDefined Arrow body"
    );
    assert!(
        dag.nodes().iter().any(|node| {
            matches!(
                node,
                Behavior::Transform(t)
                    if matches!(&t.target, TransformTarget::Callable(target) if *target == report_int)
            )
        }),
        "`report_int(42)` must lower to TransformTarget::Callable(report_int)"
    );

    let entry = bind_node_id_for_fn(&dag, "test_function_valued_dimension_report");
    let mut state = EvalStateStack::with_root_frame(EvalFrame::empty());
    let strategy = EvalStrategy::ApplicativeOrder {
        input_order: InputEvaluationOrder::LeftFirst,
    };
    let value = evaluate_body(&dag, entry, &mut state, strategy)
        .expect("function-valued data should produce DimensionReport through evaluator");

    let Value::VariantValue { tag, payload } = value else {
        panic!("expected DimensionReport::DimensionOk VariantValue");
    };
    let variant_label = dag.declarations().iter().find_map(|decl| {
        let TypeConnective::Disj { variants } = &decl.connective else {
            return None;
        };
        variants
            .iter()
            .find(|variant| variant.ty == tag)
            .map(|variant| variant.label.as_str())
    });
    assert_eq!(
        variant_label,
        Some("DimensionOk"),
        "E6-G0d must materialize the canonical DimensionOk variant constructor"
    );
    let Value::RecordValue(fields) = *payload else {
        panic!("DimensionOk payload must be a RecordValue");
    };
    let composed = fields
        .iter()
        .find(|field| field.label == "composed")
        .expect("DimensionOk.composed field");
    assert_eq!(composed.value, Value::LiteralValue(literal_bits_int(42)));
}

#[test]
fn function_valued_data_recursion_fails_closed() {
    let dag = cached_compile_any(
        RECURSIVE_SOURCE,
        "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_recursive.dag",
    );
    assert_rejected_data_lambda(&dag, "countdown");

    let dag = cached_compile_any(
        NESTED_RECURSIVE_SOURCE,
        "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_nested_recursive.dag",
    );
    assert_rejected_data_lambda(&dag, "countdown");
}

#[test]
fn function_valued_data_cycles_fail_closed() {
    for (source, file, data_name, expected_binds) in [
        (
            DATA_DATA_CYCLE_SOURCE,
            "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_data_cycle.dag",
            "evenish",
            &["evenish", "oddish", "use_evenish"][..],
        ),
        (
            DATA_FN_CYCLE_SOURCE,
            "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_fn_cycle.dag",
            "entry",
            &["entry", "helper", "use_entry"][..],
        ),
        (
            DATA_PATH_CYCLE_SOURCE,
            "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_path_cycle.dag",
            "entry",
            &["entry", "use_entry"][..],
        ),
    ] {
        let dag = cached_compile_any(source, file);
        assert_rejected_data_lambda_in_file(&dag, file, data_name);
        assert_user_callable_bind_names(&dag, file, expected_binds);
    }
}

#[test]
fn function_valued_data_lambda_errors_poison_callable() {
    let dag = cached_compile_any(
        MALFORMED_LAMBDA_SOURCE,
        "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_malformed_lambda.dag",
    );
    assert_rejected_data_lambda(&dag, "wrong_arity");
}

#[test]
fn function_valued_data_lambda_parameter_shadowing_is_not_recursive() {
    let dag = cached_compile_any(
        SHADOWED_DATA_NAME_SOURCE,
        "src/v3/compiler/tests/fixtures/r3_class_2_function_valued_data_shadowed_name.dag",
    );
    assert!(
        dag.diagnostics().is_empty(),
        "shadowed data name must resolve as the lambda parameter, not recursive data: {:?}",
        dag.diagnostics()
    );
    let apply_one = dag
        .declaration_by_name("apply_one")
        .expect("function-valued data declaration");
    assert!(
        apply_one.value_body.is_none(),
        "shadowed parameter call must not poison the data lambda"
    );
    assert!(
        matches!(
            &apply_one.connective,
            TypeConnective::Arrow {
                body: ArrowBody::UserDefined(_),
                ..
            }
        ),
        "shadowed parameter call must keep the data lambda executable"
    );

    assert!(
        dag.declaration_by_name("use_apply_one").is_some(),
        "caller should compile after shadow-aware data-lambda lowering"
    );
}
