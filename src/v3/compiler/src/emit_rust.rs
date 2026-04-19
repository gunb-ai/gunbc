pub use crate::emit::rust_target::{EmitError, RealizationCategory, SubstrateMarkerRole};
use crate::emit::{emit, emit_module, EmitDispatchError, EmitTarget};
use crate::Dag;

pub fn emit_rust(dag: &Dag) -> Result<String, EmitError> {
    match emit(dag, EmitTarget::Rust) {
        Ok(source) => Ok(source.text),
        Err(EmitDispatchError::Core(error)) => Err(error),
        Err(EmitDispatchError::Python(_)) => {
            unreachable!("EmitTarget::Rust cannot yield a Python emission error")
        }
    }
}

pub fn emit_rust_module(dag: &Dag) -> Result<String, EmitError> {
    match emit_module(dag, EmitTarget::Rust) {
        Ok(source) => Ok(source.text),
        Err(EmitDispatchError::Core(error)) => Err(error),
        Err(EmitDispatchError::Python(_)) => {
            unreachable!("EmitTarget::Rust cannot yield a Python emission error")
        }
    }
}
<<<<<<< HEAD
=======

fn variant_name_for_decl(
    dag: &Dag,
    disj_id: DeclarationId,
    variant_id: DeclarationId,
) -> Result<String, EmitError> {
    let TypeConnective::Disj { variants } = &dag.declaration(disj_id).connective else {
        unreachable!("variant_name_for_decl requires a Disj parent")
    };
    variants
        .iter()
        .find(|variant| variant.ty == variant_id)
        .map(|variant| variant.label.clone())
        .ok_or_else(|| {
            EmitError::UnsupportedBehavior(format!(
                "variant {variant_id:?} was not found under parent disjunction {disj_id:?}"
            ))
        })
}

fn variant_parent_info(dag: &Dag, variant_id: DeclarationId) -> Option<(String, String)> {
    dag.declarations().iter().find_map(|decl| {
        let enum_name = decl.name.as_ref()?;
        let TypeConnective::Disj { variants } = &decl.connective else {
            return None;
        };
        variants
            .iter()
            .find(|variant| variant.ty == variant_id)
            .map(|variant| (enum_name.clone(), variant.label.clone()))
    })
}

fn callable_template(target: DeclarationId, dag: &Dag) -> (DeclarationId, Vec<TemplateArgument>) {
    match &dag.declaration(target).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => (*template, arguments.clone()),
        _ => (target, Vec::new()),
    }
}

fn bound_callable_argument(
    dag: &Dag,
    template: DeclarationId,
    arguments: &[TemplateArgument],
    input_index: usize,
) -> Result<DeclarationId, EmitError> {
    let TypeConnective::Arrow { inputs, .. } = &dag.declaration(template).connective else {
        return Err(EmitError::UnsupportedBehavior(
            "realized callable template did not resolve to an Arrow declaration".to_string(),
        ));
    };
    let Some(param_decl) = inputs.get(input_index).copied() else {
        return Err(EmitError::UnsupportedBehavior(format!(
            "realized callable slot {} is missing from the template declaration",
            input_index
        )));
    };
    arguments
        .iter()
        .find(|arg| arg.parameter == param_decl)
        .map(|arg| arg.value)
        .ok_or_else(|| {
            EmitError::UnsupportedBehavior(
                "realized callable argument did not bind through template instantiation"
                    .to_string(),
            )
        })
}

fn rust_string_literal_body(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_ascii() && !c.is_control() => out.push(c),
            c => {
                use std::fmt::Write;
                let _ = write!(&mut out, "\\u{{{:X}}}", c as u32);
            }
        }
    }
    out
}

fn render_value(v: &ValueNode, literals: &LiteralSyntaxBinding) -> String {
    match &v.data {
        LiteralBits::Int(n) => n.to_string(),
        LiteralBits::Bool(true) => literals.true_keyword.clone(),
        LiteralBits::Bool(false) => literals.false_keyword.clone(),
        LiteralBits::String(s) => format!(
            "String::from({}{}{})",
            literals.string_delimiter,
            rust_string_literal_body(s),
            literals.string_delimiter
        ),
    }
}

fn behavior_result_port(behavior: &Behavior) -> PortId {
    match behavior {
        Behavior::Value(v) => v.result_port(),
        Behavior::Transform(t) => t.result_port(),
        Behavior::Branch(b) => b.result_port(),
        Behavior::Loop(l) => l.result_port(),
        Behavior::Bind(b) => b.result_port(),
    }
}

fn is_bootstrap_file(file: &str) -> bool {
    file.starts_with("dsl/std/")
        || file.starts_with("src/v3/std/")
        || file.starts_with("src/v3/spec/")
        || file.starts_with("src/v3/compiler/")
}

/// Walk a port's resolved TypeShape declaration through anonymous
/// aliases (`Atom(ResolvedIdentifier)`) and instantiations
/// (`TypeConnective::Instantiation`) until it lands on the first
/// **named** declaration. Returns that declaration's id.
///
/// **Why named-declaration stop.** The realization indexes are
/// keyed by the canonical declaration ids of the named primitives
/// declared in std/ (`Int`, `Bool`, `String`, etc.). When a port's
/// `TypeShape` points at an anonymous wrapper (e.g. an
/// `Instantiation { template: Int, .. }` allocated by
/// `type_to_declaration_id` for compound types), the walk steps
/// through the wrapper to the named declaration the realization
/// references. When the port's TypeShape is a named alias like
/// `type CommitSha = String`, the walk stops at `CommitSha` —
/// callers see the alias's id directly. If the realization index
/// has no entry for the alias, the lookup fails with
/// `MissingTypeRealization` carrying the alias id, which is the
/// honest signal: the realization spec needs to declare the alias
/// (or M2+ adds an alias-walking dispatch via meta_tag chains).
///
/// At PR-B scope the walk depth is bounded to 32 to catch any
/// runaway cycles; the std/ types we actually consume bottom out
/// in 1–2 hops.
fn primitive_type_id_for_port(dag: &Dag, port: PortId) -> Result<DeclarationId, EmitError> {
    let ts = dag
        .port(port)
        .value_type()
        .ok_or(EmitError::UntypedPort(port))?;
    let mut current = ts.declaration;
    for _ in 0..32 {
        let decl = dag.declaration(current);
        if decl.name.is_some() {
            return Ok(current);
        }
        match &decl.connective {
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => current = *next,
            _ => return Ok(current),
        }
    }
    Err(EmitError::UnsupportedBehavior(
        "port type walk exceeded depth 32 — likely a cycle".to_string(),
    ))
}

fn walk_to_conj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        let decl = dag.declaration(current);
        match &decl.connective {
            TypeConnective::Conj { .. } => return Some(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => current = *next,
            _ => return None,
        }
    }
    None
}

/// Walk a declaration through aliases / instantiations to a `Disj`.
/// Returns the Disj declaration's id, or None if the chain bottoms
/// out without hitting a Disj. Mirrors `walk_to_conj_decl` in
/// `lower.rs` for symmetry.
fn walk_to_disj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        match &dag.declaration(current).connective {
            TypeConnective::Disj { .. } => return Some(current),
            TypeConnective::Cardinality {
                bound: crate::dag::CardinalityBound::AtMostOne,
                ..
            } => return optional_match_disj_for_cardinality(dag, current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => current = *next,
            _ => return None,
        }
    }
    None
}

fn optional_match_disj_for_cardinality(
    dag: &Dag,
    cardinality_decl_id: DeclarationId,
) -> Option<DeclarationId> {
    dag.optional_match_disj(cardinality_decl_id)
}

fn is_optional_match_disj(dag: &Dag, disj_id: DeclarationId) -> bool {
    dag.declarations()
        .iter()
        .filter_map(|decl| dag.optional_match_disj(decl.id))
        .any(|optional_disj| optional_disj == disj_id)
}

/// Resolve the algebra-field declaration id for a given operand
/// type and `OperatorKind`. Walks the operand type's instantiation
/// chain to the algebra Conj (e.g. OrderedRing for Int), then finds
/// the field whose label matches the operator's algebra field name.
/// Returns the field's child declaration id, which the rust.dag
/// `op: OrderedRing.add` reference also resolves to via the
/// dotted-path lowering.
///
/// **Why this is acceptable as a thin bridge.** The
/// `OperatorKind::algebra_field_name()` lookup is the substrate's
/// existing operator → field mapping (already used by
/// `infer::resolve_operator_arrow`). It IS a name comparison, but
/// the name lives ONCE in `operators.rs` (tightly coupled to the
/// `OperatorKind` enum) and the resolved declaration id is what
/// flows downstream. The emitter doesn't repeat the comparison;
/// it asks this helper for the field id and uses it as a typed
/// index key.
fn algebra_field_for_operator(
    dag: &Dag,
    operand_type_id: DeclarationId,
    op: OperatorKind,
) -> Result<DeclarationId, EmitError> {
    // Walk the operand type to its algebra Conj. The same walk is
    // used by infer.rs's resolve_operator_arrow.
    let Some(algebra_conj_id) = walk_to_algebra_conj(dag, operand_type_id) else {
        return canonical_operator_field(dag, op);
    };
    let field_label = op.algebra_field_name();
    let children = match &dag.declaration(algebra_conj_id).connective {
        TypeConnective::Conj { children } => children,
        _ => unreachable!("walk_to_algebra_conj returned a non-Conj"),
    };
    if let Some(field) = children.iter().find(|f| f.label == field_label) {
        return Ok(field.ty);
    }
    canonical_operator_field(dag, op)
}

/// Walk a declaration through aliases / instantiations until it
/// reaches a Conj (the algebra declaration). Returns the Conj's id.
fn walk_to_algebra_conj(dag: &Dag, start: DeclarationId) -> Option<DeclarationId> {
    let mut current = start;
    for _ in 0..32 {
        match &dag.declaration(current).connective {
            TypeConnective::Conj { .. } => return Some(current),
            TypeConnective::Instantiation { template, .. } => current = *template,
            TypeConnective::Atom(AtomPayload::ResolvedByStructure(next))
            | TypeConnective::Atom(AtomPayload::ResolvedByName(next)) => current = *next,
            _ => return None,
        }
    }
    None
}

fn canonical_operator_field(dag: &Dag, op: OperatorKind) -> Result<DeclarationId, EmitError> {
    let ordered_ring = dag.declaration_by_name("OrderedRing").ok_or_else(|| {
        EmitError::UnsupportedBehavior(
            "bootstrap is missing the canonical `OrderedRing` declaration".to_string(),
        )
    })?;
    let TypeConnective::Conj { children } = &ordered_ring.connective else {
        return Err(EmitError::UnsupportedBehavior(
            "`OrderedRing` does not lower to a Conj declaration".to_string(),
        ));
    };
    let field_label = op.algebra_field_name();
    children
        .iter()
        .find(|field| field.label == field_label)
        .map(|field| field.ty)
        .ok_or_else(|| {
            EmitError::UnsupportedBehavior(format!(
                "`OrderedRing` has no canonical field labeled {field_label}"
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_to_dag;
    use crate::diagnostics::SourceSpan;

    #[test]
    fn render_field_project_reads_borrowed_nodes_without_cloning() {
        let mut dag = Dag::new();
        let parent_port = dag.alloc_port(None);
        let dag_type = dag
            .declaration_by_name("Dag")
            .expect("Dag type realization target exists")
            .id;
        let dag_nodes_type = match &dag.declaration(dag_type).connective {
            TypeConnective::Conj { children } => {
                children
                    .iter()
                    .find(|field| field.label == "nodes")
                    .expect("Dag.nodes field")
                    .ty
            }
            other => panic!("Dag must be a Conj, got {other:?}"),
        };
        dag.set_port_type(parent_port, crate::types::TypeShape::new(dag_type));
        let node_id = dag.alloc_node_id();
        let output = dag.alloc_port(Some(node_id));
        dag.push_node(Behavior::Transform(TransformNode {
            id: node_id,
            target: TransformTarget::FieldProject {
                field_label: "nodes".to_string(),
                field_child: Some(dag_nodes_type),
            },
            inputs: vec![parent_port],
            output,
            span: SourceSpan::new("<test>", 0, 0),
        }));
        dag.set_port_type(output, crate::types::TypeShape::new(dag_nodes_type));

        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let input_use_facts = InputUseFacts::build(&dag, &indexes);
        let mut bound_names = HashMap::new();
        bound_names.insert(parent_port, LocalBinding::Owned("parent".to_string()));
        let ctx = Ctx {
            dag: &dag,
            indexes: &indexes,
            bound_names: &bound_names,
            input_use_facts: &input_use_facts,
            mode: EmitRustMode::Program,
        };

        let rendered = match dag.node(node_id) {
            Behavior::Transform(t) => ctx
                .render_transform(t, &RenderLocals::default(), RenderMode::BorrowedRead)
                .expect("field project renders"),
            other => panic!("expected Transform node, got {other:?}"),
        };
        assert_eq!(rendered, "(parent).nodes()");
    }

    #[test]
    fn render_field_project_constructs_owned_list_from_borrowed_nodes() {
        let mut dag = Dag::new();
        let parent_port = dag.alloc_port(None);
        let dag_type = dag
            .declaration_by_name("Dag")
            .expect("Dag type realization target exists")
            .id;
        let dag_nodes_type = match &dag.declaration(dag_type).connective {
            TypeConnective::Conj { children } => {
                children
                    .iter()
                    .find(|field| field.label == "nodes")
                    .expect("Dag.nodes field")
                    .ty
            }
            other => panic!("Dag must be a Conj, got {other:?}"),
        };
        dag.set_port_type(parent_port, crate::types::TypeShape::new(dag_type));

        let mut test_node_ids = Vec::new();
        for _ in 0..2 {
            let node_id = dag.alloc_node_id();
            let output = dag.alloc_port(Some(node_id));
            dag.push_node(Behavior::Transform(TransformNode {
                id: node_id,
                target: TransformTarget::FieldProject {
                    field_label: "nodes".to_string(),
                    field_child: Some(dag_nodes_type),
                },
                inputs: vec![parent_port],
                output,
                span: SourceSpan::new("<test>", 0, 0),
            }));
            dag.set_port_type(output, crate::types::TypeShape::new(dag_nodes_type));
            test_node_ids.push(node_id);
        }

        // Query by the specific node we just pushed — earlier Transform
        // nodes in `dag.nodes()` belong to bootstrap-loaded std modules
        // and have no `parent` binding in `bound_names`, which renders
        // as the empty-list fallback rather than the expected projection.
        let first_transform = match dag.node(test_node_ids[0]) {
            Behavior::Transform(t) => t,
            other => panic!("pushed transform went missing, got {other:?}"),
        };
        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let input_use_facts = InputUseFacts::build(&dag, &indexes);
        let mut bound_names = HashMap::new();
        bound_names.insert(parent_port, LocalBinding::Owned("parent".to_string()));
        let ctx = Ctx {
            dag: &dag,
            indexes: &indexes,
            bound_names: &bound_names,
            input_use_facts: &input_use_facts,
            mode: EmitRustMode::Program,
        };

        let rendered = ctx
            .render_transform(
                first_transform,
                &RenderLocals::default(),
                RenderMode::OwnedConstruct,
            )
            .expect("field project renders");
        assert_eq!(rendered, "((parent).nodes()).to_vec()");
    }

    #[test]
    fn render_fold_iterates_named_list_input_by_borrow() {
        let dag = compile_to_dag(
            "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
            "test.v3",
        )
        .expect("compiles");
        let fold_template = dag.declaration_by_name("fold").expect("fold decl").id;
        let fold_transform = dag
            .nodes()
            .iter()
            .find_map(|node| match node {
                Behavior::Transform(t) => match &t.target {
                    TransformTarget::Callable(target) => {
                        let (template, _) = callable_template(*target, &dag);
                        (template == fold_template).then_some(t)
                    }
                    _ => None,
                },
                _ => None,
            })
            .expect("fold transform");

        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let input_use_facts = InputUseFacts::build(&dag, &indexes);
        let mut bound_names = HashMap::new();
        bound_names.insert(
            fold_transform.inputs[0],
            LocalBinding::Owned("xs".to_string()),
        );
        let ctx = Ctx {
            dag: &dag,
            indexes: &indexes,
            bound_names: &bound_names,
            input_use_facts: &input_use_facts,
            mode: EmitRustMode::Program,
        };

        let rendered = ctx
            .render_transform(
                fold_transform,
                &RenderLocals::default(),
                RenderMode::OwnedConstruct,
            )
            .expect("fold renders");
        assert!(
            rendered.contains("(xs).iter().fold("),
            "expected named list inputs to be iterated by borrow, got: {rendered}"
        );
    }

    #[test]
    fn rendering_model_read_strategy_controls_function_parameter_shape() {
        let mut dag = compile_to_dag(
            "type Sign = Plus | Minus
fn classify(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }",
            "test.v3",
        )
        .expect("compiles");
        let rendering_decl = dag.rust_rendering_spec().expect("rust_rendering cached");
        let pass_by_value = named_variant_id(&dag, "ReadStrategy", "PassByValue")
            .expect("ReadStrategy.PassByValue exists");
        let copy_or_clone = named_variant_id(&dag, "ConstructStrategy", "CopyOrClone")
            .expect("ConstructStrategy.CopyOrClone exists");
        dag.declaration_mut(rendering_decl).value_body = Some(ValueBody::Structural {
            fields: vec![
                (
                    "read".to_string(),
                    FieldValue::Variant {
                        constructor: pass_by_value,
                        payload: Vec::new(),
                    },
                ),
                (
                    "construct".to_string(),
                    FieldValue::Variant {
                        constructor: copy_or_clone,
                        payload: Vec::new(),
                    },
                ),
            ],
        });

        let rendered = emit_rust_with_mode(&dag, EmitRustMode::Module).expect("emits");
        assert!(
            rendered.contains("fn classify(p0: Sign) -> i64 {"),
            "expected PassByValue read strategy to render owned function params, got: {rendered}"
        );
    }

    #[test]
    fn clean_emission_rejects_rust_invalid_pattern_binding_variants() {
        let assert_rejected =
            |pick: fn(&crate::dag::PatternBindingRuleVariants) -> Option<DeclarationId>,
             expected_detail: &'static str| {
                let mut dag = compile_to_dag(
                    "type Sign = Plus | Minus
fn classify(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }",
                    "test.v3",
                )
                .expect("compiles");
                let clean_decl = dag
                    .rust_clean_emission_spec()
                    .expect("rust_clean_emission cached");
                let invalid_ctor = pick(dag.pattern_binding_rule_variants())
                    .expect("PatternBindingRule variant cached");
                dag.declaration_mut(clean_decl).value_body = Some(ValueBody::Structural {
                    fields: vec![(
                        "pattern_bindings".to_string(),
                        FieldValue::Variant {
                            constructor: invalid_ctor,
                            payload: Vec::new(),
                        },
                    )],
                });

                let err = emit_rust_with_mode(&dag, EmitRustMode::Module)
                    .expect_err("Rust-invalid pattern binding rule must fail closed");
                assert!(matches!(
                    err,
                    EmitError::MalformedTargetSyntax {
                        declaration,
                        detail,
                    } if declaration == clean_decl && detail == expected_detail
                ));
            };

        assert_rejected(
            |v| v.emit_prefixed,
            "rust_clean_emission.pattern_bindings cannot use PatternBindingRule.EmitPrefixedUnderscoreWhenUnused; Rust only supports EmitBindingAlways or EmitUnderscoreWhenUnused",
        );
        assert_rejected(
            |v| v.not_applicable,
            "rust_clean_emission.pattern_bindings cannot use PatternBindingRule.NotApplicablePatternBinding; Rust only supports EmitBindingAlways or EmitUnderscoreWhenUnused",
        );
    }

    #[test]
    fn callable_disposition_derives_direct_return_as_consumed() {
        let dag = compile_to_dag("fn id(x: Int) -> Int = x", "direct_return.v3").expect("compiles");
        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let id_decl = dag.declaration_by_name("id").expect("id decl").id;
        assert_eq!(
            indexes.callable_dispositions.get(&id_decl),
            Some(&vec![ParameterDispositionBinding::Consumed]),
        );
    }

    #[test]
    fn callable_disposition_keeps_match_scrutinee_borrowed() {
        let dag = compile_to_dag(
            "fn head_or_zero(list: List<Int>) -> Int = match list { Empty => 0, Cons(payload) => payload.head }",
            "match_payload.v3",
        )
        .expect("compiles");
        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let decl = dag
            .declaration_by_name("head_or_zero")
            .expect("head_or_zero decl")
            .id;
        assert_eq!(
            indexes.callable_dispositions.get(&decl),
            Some(&vec![ParameterDispositionBinding::Borrowed]),
        );
    }

    #[test]
    fn callable_disposition_keeps_nested_lambda_capture_borrowed() {
        let dag = compile_to_dag(
            "fn apply_to_three(f: fn(Int) -> Int) -> Int = f(3)
fn use_callback(base: Int) -> Int = apply_to_three(|x| base + x)",
            "nested_lambda.v3",
        )
        .expect("compiles");
        let indexes = RealizationIndexes::build(&dag).expect("indexes build");
        let decl = dag
            .declaration_by_name("use_callback")
            .expect("use_callback decl")
            .id;
        assert_eq!(
            indexes.callable_dispositions.get(&decl),
            Some(&vec![ParameterDispositionBinding::Borrowed]),
        );
    }

    /// B1 — `require_parameter_dispositions` is fail-closed against
    /// arity, slot duplication, and out-of-range slots, so a spec
    /// CallableRealization can't silently drift from the callable's
    /// declared Arrow input arity.
    #[test]
    fn parameter_dispositions_reject_arity_drift_and_slot_collisions() {
        let dag = compile_to_dag("fn id(x: Int) -> Int = x", "arity_drift.v3").expect("compiles");
        let bogus_decl = dag.declaration_by_name("id").expect("id decl").id;
        let borrowed =
            named_variant_id(&dag, "ParameterDisposition", "Borrowed").expect("Borrowed");
        let consumed =
            named_variant_id(&dag, "ParameterDisposition", "Consumed").expect("Consumed");
        let entry = |slot: i64, ctor: DeclarationId| {
            FieldValue::Record(vec![
                (
                    "slot".to_string(),
                    FieldValue::Literal(LiteralBits::Int(slot)),
                ),
                (
                    "disposition".to_string(),
                    FieldValue::Variant {
                        constructor: ctor,
                        payload: vec![],
                    },
                ),
            ])
        };
        let bind =
            |entries: Vec<FieldValue>| vec![("parameters".to_string(), FieldValue::List(entries))];

        // Arity too low: 1 entry expected, 0 supplied.
        let fields = bind(vec![]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 1),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Arity too high: 1 entry expected, 2 supplied.
        let fields = bind(vec![entry(0, borrowed), entry(1, borrowed)]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 1),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Slot duplication: both entries claim slot 0.
        let fields = bind(vec![entry(0, borrowed), entry(0, consumed)]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 2),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Out-of-range slot: arity is 1 but entry claims slot 5.
        let fields = bind(vec![entry(5, borrowed)]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 1),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Negative slot: rejected before the bound check.
        let fields = bind(vec![entry(-1, borrowed)]);
        assert!(matches!(
            require_parameter_dispositions(&dag, &fields, bogus_decl, 1),
            Err(EmitError::MalformedRealization { .. }),
        ));

        // Well-formed: each slot in [0, arity) exactly once. Returns a
        // Vec of length `expected_arity`, indexed by slot.
        let fields = bind(vec![entry(1, borrowed), entry(0, consumed)]);
        let result = require_parameter_dispositions(&dag, &fields, bogus_decl, 2)
            .expect("well-formed parameters parse");
        assert_eq!(
            result,
            vec![
                ParameterDispositionBinding::Consumed,
                ParameterDispositionBinding::Borrowed,
            ],
        );
    }

    /// B11 (post-refactor) — shared realizations are owned via a
    /// typed `language: DeclarationRef` pointing at the target's
    /// language-spec declaration, NOT a TargetLanguage enum variant.
    /// Each emitter compares the typed reference to its cached
    /// language-spec id at index-build time. A realization whose
    /// surface name is `rust_*` but whose `language` refers to
    /// `go_language` is owned by Go.
    #[test]
    fn target_language_is_typed_reference_to_language_spec() {
        let dag = compile_to_dag("fn id(x: Int) -> Int = x", "lang_ref.v3").expect("compiles");
        let rust_language_id = dag
            .rust_language_spec()
            .expect("rust_language cached after bootstrap");
        let go_language_id = dag
            .go_language_spec()
            .expect("go_language cached after bootstrap");
        // Rust and Go have distinct language-spec declaration ids, so
        // comparing a realization's `language` field to the cached id
        // partitions entries cleanly. This is the structural signal
        // that replaced the TargetLanguage enum roster.
        assert_ne!(rust_language_id, go_language_id);
    }
}
>>>>>>> 63d0099e7 (Fix structural resolution lens snapshot)
