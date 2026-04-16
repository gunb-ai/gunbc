use v3_compiler::compile_to_dag;
use v3_compiler::dag::{AtomPayload, Behavior, Dag, LiteralBits, TransformTarget, TypeConnective};
use v3_compiler::types::TypeShape;

fn primitive_shape(dag: &Dag, name: &str) -> TypeShape {
    TypeShape::new(
        dag.declaration_by_name(name)
            .unwrap_or_else(|| panic!("primitive `{name}` missing from bootstrap"))
            .id,
    )
}

fn assert_target_name(dag: &Dag, target: &TransformTarget, expected: &str) {
    let actual: Option<String> = match target {
        TransformTarget::Callable(id) => {
            let decl = dag.declaration(*id);
            match &decl.connective {
                TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name)) => {
                    Some(name.clone())
                }
                TypeConnective::Atom(AtomPayload::ResolvedIdentifier(next)) => {
                    dag.declaration(*next).name.clone()
                }
                _ => decl.name.clone(),
            }
        }
        TransformTarget::FieldProject { field_label } => Some(format!(".{field_label}")),
        TransformTarget::Operator(op_kind) => Some(op_kind.symbol().to_string()),
    };
    assert_eq!(
        actual.as_deref(),
        Some(expected),
        "Transform.target name mismatch"
    );
}

fn bind_named<'a>(dag: &'a Dag, name: &str) -> &'a v3_compiler::dag::BindNode {
    dag.nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == name)
        .unwrap_or_else(|| panic!("Bind({name}) must exist"))
}

fn literal_input(dag: &Dag, port: v3_compiler::dag::PortId) -> &LiteralBits {
    let producer_id = dag
        .port(port)
        .produced_by
        .expect("input port has a producer node");
    &dag.node(producer_id)
        .as_value()
        .expect("input producer is a Value node")
        .data
}

#[test]
fn pipe_desugars_unary_call_by_injecting_the_left_value() {
    let src = "\
fn negate(x: Int) -> Int = 0 - x
let y = 5 |> negate
";
    let dag = compile_to_dag(src, "pipe_unary.v3").expect("compiles");

    let bind_y = bind_named(&dag, "y");
    let call_id = dag
        .port(bind_y.value)
        .produced_by
        .expect("Bind(y) value has a producer");
    let call = dag
        .node(call_id)
        .as_transform()
        .expect("producer is a Transform");

    assert_target_name(&dag, &call.target, "negate");
    assert_eq!(call.inputs.len(), 1, "negate takes one injected argument");
    assert_eq!(literal_input(&dag, call.inputs[0]), &LiteralBits::Int(5));
    assert_eq!(
        dag.port(bind_y.value).value_type(),
        Some(&primitive_shape(&dag, "Int")),
        "pipe-desugared call preserves the callee's declared return type",
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn pipe_desugars_multi_arg_call_with_first_arg_injection() {
    let src = "\
fn keep_first(a: Int, b: Int) -> Int = a
let y = 5 |> keep_first(6)
";
    let dag = compile_to_dag(src, "pipe_multi_arg.v3").expect("compiles");

    let bind_y = bind_named(&dag, "y");
    let call_id = dag
        .port(bind_y.value)
        .produced_by
        .expect("Bind(y) value has a producer");
    let call = dag
        .node(call_id)
        .as_transform()
        .expect("producer is a Transform");

    assert_target_name(&dag, &call.target, "keep_first");
    assert_eq!(call.inputs.len(), 2, "keep_first takes the injected arg plus one explicit arg");
    assert_eq!(
        literal_input(&dag, call.inputs[0]),
        &LiteralBits::Int(5),
        "the piped value becomes argument 0",
    );
    assert_eq!(
        literal_input(&dag, call.inputs[1]),
        &LiteralBits::Int(6),
        "the original call arguments keep their order after the injected value",
    );
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn pipe_chains_left_to_right() {
    let src = "\
fn add1(x: Int) -> Int = x + 1
fn double(x: Int) -> Int = x + x
let y = 5 |> add1 |> double
";
    let dag = compile_to_dag(src, "pipe_chain.v3").expect("compiles");

    let bind_y = bind_named(&dag, "y");
    let outer_call_id = dag
        .port(bind_y.value)
        .produced_by
        .expect("Bind(y) value has a producer");
    let outer_call = dag
        .node(outer_call_id)
        .as_transform()
        .expect("outer producer is a Transform");
    assert_target_name(&dag, &outer_call.target, "double");
    assert_eq!(outer_call.inputs.len(), 1);

    let inner_call_id = dag
        .port(outer_call.inputs[0])
        .produced_by
        .expect("double input is produced by the previous pipe stage");
    let inner_call = dag
        .node(inner_call_id)
        .as_transform()
        .expect("previous stage is a Transform");
    assert_target_name(&dag, &inner_call.target, "add1");
    assert_eq!(inner_call.inputs.len(), 1);
    assert_eq!(literal_input(&dag, inner_call.inputs[0]), &LiteralBits::Int(5));
    assert!(dag.diagnostics().is_empty());
}
