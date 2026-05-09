use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    ArrowBody, AtomPayload, Behavior, BranchPattern, Dag, DeclarationId, PortState,
    TransformTarget, TypeConnective,
};
use v3_compiler::operators::{ArithmeticOp, ComparisonOp, LogicalOp, OperatorKind};
use v3_compiler::Diagnostic;

use crate::common::substrate_receipts::{
    assert_bootstrap_float64_compose_ieee_machine_width,
    assert_bootstrap_int64_compose_int_machine_width, bind_named, bind_value_type_decl,
    callable_instantiation_arguments, field, find_named, transforms_in_source_file,
};
use crate::common::{cached_compile_any, cached_compile_to_dag};

fn compile_any(src: &str, file: &str) -> Dag {
    cached_compile_any(src, file)
}

#[test]
fn operator_helpers_round_trip_from_dag_authority() {
    for (symbol, op, field_name) in [
        ("+", OperatorKind::Arithmetic(ArithmeticOp::Add), "add"),
        ("-", OperatorKind::Arithmetic(ArithmeticOp::Sub), "sub"),
        ("*", OperatorKind::Arithmetic(ArithmeticOp::Mul), "mul"),
        ("/", OperatorKind::Arithmetic(ArithmeticOp::Div), "div"),
        ("==", OperatorKind::Comparison(ComparisonOp::Eq), "eq"),
        ("!=", OperatorKind::Comparison(ComparisonOp::Ne), "ne"),
        ("<", OperatorKind::Comparison(ComparisonOp::Lt), "lt"),
        ("<=", OperatorKind::Comparison(ComparisonOp::Le), "le"),
        (">", OperatorKind::Comparison(ComparisonOp::Gt), "gt"),
        (">=", OperatorKind::Comparison(ComparisonOp::Ge), "ge"),
        ("&&", OperatorKind::Logical(LogicalOp::And), "meet"),
        ("||", OperatorKind::Logical(LogicalOp::Or), "join"),
    ] {
        assert_eq!(v3_compiler::operators::from_symbol(symbol), Some(op));
        assert_eq!(v3_compiler::operators::symbol(op), symbol);
        assert_eq!(v3_compiler::operators::algebra_field_name(op), field_name);
    }
}

#[test]
fn bootstrap_int64_compose_int_machine_width_per_gate_19() {
    // R3 gate #19: fixed-width integers refine abstract `Int` via Compose × MachineWidth,
    // not parallel OrderedRing<Word*> substrate.
    assert_bootstrap_int64_compose_int_machine_width(&Dag::new());
}

#[test]
fn bootstrap_float64_compose_ieee_machine_width_per_gate_19() {
    assert_bootstrap_float64_compose_ieee_machine_width(&Dag::new());
}

#[test]
fn bootstrap_inventory_stays_typed_and_cached() {
    let dag = Dag::new();
    for (shape, name) in [
        (dag.int_shape().expect("Int cached"), "Int"),
        (dag.bool_shape().expect("Bool cached"), "Bool"),
        (dag.string_shape().expect("String cached"), "String"),
    ] {
        assert_eq!(
            shape,
            v3_compiler::types::TypeShape::new(find_named(&dag, name))
        );
    }

    for decl_id in [
        dag.bind_marker().expect("Bind marker"),
        dag.branch_marker().expect("Branch marker"),
        dag.loop_marker().expect("Loop marker"),
        dag.transform_marker().expect("Transform marker"),
        dag.value_marker().expect("ValueBehavior marker"),
        dag.main_marker().expect("Main marker"),
        dag.declaration_ref_marker().expect("DeclarationRef marker"),
        dag.type_realization_meta().expect("TypeRealization meta"),
        dag.type_instantiation_realization_meta()
            .expect("TypeInstantiationRealization meta"),
        dag.operator_realization_meta()
            .expect("OperatorRealization meta"),
        dag.behavior_realization_meta()
            .expect("BehaviorRealization meta"),
        dag.callable_realization_meta()
            .expect("CallableRealization meta"),
        dag.pattern_realization_meta()
            .expect("PatternRealization meta"),
    ] {
        assert!(
            matches!(
                dag.declaration(decl_id).connective,
                TypeConnective::Conj { .. }
            ),
            "bootstrap cache should point at Conj authorities"
        );
    }

    let rust_language = dag
        .rust_language_spec()
        .expect("rust_language syntax bundle should be cached");
    let rust_language_labels = match dag.declaration(rust_language).value_body.as_ref() {
        Some(v3_compiler::dag::ValueBody::Structural { fields }) => fields
            .iter()
            .map(|(label, _)| label.as_str())
            .collect::<Vec<_>>(),
        other => panic!("rust_language must lower structurally, got {other:?}"),
    };
    assert_eq!(
        rust_language_labels,
        vec![
            "statements",
            "expressions",
            "control_flow",
            "literals",
            "modules",
            "functions",
            "type_applications",
            "type_definitions",
            "record_derive_templates",
            "patterns",
            "collection_ops",
            "values",
        ]
    );

    let rust_rendering = dag
        .rust_rendering_spec()
        .expect("rust_rendering bundle should be cached");
    let rust_rendering_labels = match dag.declaration(rust_rendering).value_body.as_ref() {
        Some(v3_compiler::dag::ValueBody::Structural { fields }) => fields
            .iter()
            .map(|(label, _)| label.as_str())
            .collect::<Vec<_>>(),
        other => panic!("rust_rendering must lower structurally, got {other:?}"),
    };
    assert_eq!(rust_rendering_labels, vec!["read", "construct"]);

    assert!(matches!(
        dag.declaration(find_named(&dag, "RenderingModel"))
            .connective,
        TypeConnective::Conj { .. }
    ));
    assert!(matches!(
        dag.declaration(find_named(&dag, "ReadStrategy")).connective,
        TypeConnective::Disj { .. }
    ));
    assert!(matches!(
        dag.declaration(find_named(&dag, "ConstructStrategy"))
            .connective,
        TypeConnective::Disj { .. }
    ));
}

#[test]
fn bootstrap_child_declarations_stay_anonymous_and_structural() {
    let dag = Dag::new();
    for name in [
        "T",
        "True",
        "False",
        "Less",
        "Equal",
        "Greater",
        "Int64_add_rust",
        "Int64_add",
    ] {
        assert!(
            dag.declaration_by_name(name).is_none(),
            "child declaration `{name}` must not leak through declaration_by_name"
        );
    }

    assert!(dag.declaration_by_name("Int").is_some());
    assert!(dag.declaration_by_name("OrderedRing").is_some());
    assert!(dag.declaration_by_name("Classical").is_some());

    let ordered_ring_fields = match &dag.declaration(find_named(&dag, "OrderedRing")).connective {
        TypeConnective::Conj { children } => children,
        other => panic!("OrderedRing should be a Conj, got {other:?}"),
    };
    for label in [
        "add", "sub", "mul", "div", "eq", "ne", "lt", "le", "gt", "ge",
    ] {
        assert!(
            ordered_ring_fields.iter().any(|field| field.label == label),
            "OrderedRing missing direct operator field `{label}`"
        );
    }

    let magma_decl = dag.declaration(find_named(&dag, "Magma"));
    let magma_fields = match &magma_decl.connective {
        TypeConnective::Conj { children } => children,
        other => panic!("Magma should be a Conj, got {other:?}"),
    };
    assert_eq!(magma_fields.len(), 1);
    assert_eq!(magma_fields[0].label, "op");
    assert_eq!(magma_decl.type_params.len(), 1);
    assert!(matches!(
        dag.declaration(magma_decl.type_params[0]).connective,
        TypeConnective::Atom(AtomPayload::TypeParam(_))
    ));
}

#[test]
fn synthetic_service_compile_receipt_uses_nested_conjs() {
    let dag = compile_any(
        "\
type SyntheticService { }
type SyntheticOperation { }
type RunInput { }
type RunOutput { }
type RunArguments { }
type CmdExec_Run { input: RunInput output: RunOutput arguments: RunArguments }
type CmdExec_Operations { Run: CmdExec_Run }
type CmdExec { operations: CmdExec_Operations }
",
        "synthetic.v3",
    );
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());

    let cmd_exec_fields = match &dag.declaration(find_named(&dag, "CmdExec")).connective {
        TypeConnective::Conj { children } => children,
        other => panic!("CmdExec should lower to a Conj, got {other:?}"),
    };
    let operations = field(cmd_exec_fields, "operations").ty;
    let run = match &dag.declaration(operations).connective {
        TypeConnective::Conj { children } => field(children, "Run").ty,
        other => panic!("CmdExec_Operations should lower to a Conj, got {other:?}"),
    };
    let run_fields = match &dag.declaration(run).connective {
        TypeConnective::Conj { children } => children,
        other => panic!("CmdExec_Run should lower to a Conj, got {other:?}"),
    };
    for label in ["input", "output", "arguments"] {
        let child = field(run_fields, label).ty;
        assert!(
            matches!(
                dag.declaration(child).connective,
                TypeConnective::Conj { .. }
            ),
            "CmdExec_Run.{label} should point at a Conj carrier"
        );
    }

    for name in ["SyntheticService", "SyntheticOperation"] {
        assert!(matches!(
            dag.declaration(find_named(&dag, name)).connective,
            TypeConnective::Conj { .. }
        ));
    }
}

#[test]
fn m17_operator_lowers_to_structural_transform_target() {
    // Class 2: `let x = 1 + 2` lowers to a Transform whose
    // `target: TransformTarget::Operator(Arithmetic(Add))`. No
    // anonymous stub declaration is allocated for the operator
    // symbol; the dispatch fact lives on the Transform's variant.
    let dag = cached_compile_to_dag("let x = 1 + 2", "test.v3");
    let add_node = transforms_in_source_file(&dag, "test.v3")
        .next()
        .expect("Transform node exists");
    match &add_node.target {
        TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)) => {}
        other => panic!("expected TransformTarget::Operator(Arithmetic(Add)), got {other:?}"),
    }
}

#[test]
fn m17_comparison_operator_lowers_to_structural_transform_target() {
    // Class 2: comparison operators commit to the
    // `OperatorKind::Comparison` variant at parse time. The
    // arithmetic-vs-comparison split is structural, not a sibling
    // string match.
    let dag = cached_compile_to_dag("let y = 1 < 2", "test.v3");
    let cmp_node = transforms_in_source_file(&dag, "test.v3")
        .next()
        .expect("Transform node exists");
    match &cmp_node.target {
        TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Lt)) => {}
        other => panic!("expected TransformTarget::Operator(Comparison(Lt)), got {other:?}"),
    }
}

#[test]
fn m17_user_function_call_lowers_to_callable_target() {
    // Class 2: user function calls still resolve to
    // `TransformTarget::Callable(DeclarationId)`. The split between
    // operator and callable is discriminated structurally at
    // lowering time based on the `SurfaceExpr` variant the parser
    // committed to.
    let src = "fn f(x: Int) -> Int = x + 1\nlet y = f(5)";
    let dag = cached_compile_to_dag(src, "test.v3");
    let f_call = transforms_in_source_file(&dag, "test.v3")
        .find(|t| matches!(&t.target, TransformTarget::Callable(_)))
        .expect("Callable Transform exists");
    let target_id = match &f_call.target {
        TransformTarget::Callable(id) => *id,
        _ => unreachable!(),
    };
    let name = dag.declaration(target_id).name.as_deref();
    assert_eq!(
        name,
        Some("f"),
        "Callable target points at user function `f`"
    );
}

#[test]
fn m17_brace_bodied_fn_parses_single_expr_to_user_defined_arrow_body() {
    // Prereq-2 slice: `fn foo(x: Int) -> Int { x + 1 }` parses as
    // `SurfaceItem::Fn` with a real `SurfaceExpr` body and lowers to
    // `ArrowBody::UserDefined` (Bind), not `FnExternalBody` /
    // `ArrowBody::Unparsed`.
    let src = "fn foo(x: Int) -> Int { x + 1 }";
    let dag = compile_any(src, "brace_fn_user_defined.v3");

    // The `foo` declaration exists and carries the right Arrow
    // signature.
    let foo_id = find_named(&dag, "foo");
    let (inputs, output, body) = match &dag.declaration(foo_id).connective {
        TypeConnective::Arrow {
            inputs,
            output,
            body,
        } => (inputs.clone(), *output, body.clone()),
        other => panic!("expected Arrow, got {other:?}"),
    };
    assert_eq!(inputs.len(), 1);
    assert!(
        matches!(body, ArrowBody::UserDefined(_)),
        "brace-bodied fn with a single expression body must carry ArrowBody::UserDefined, got {body:?}"
    );

    // The signature types resolve to the cached primitives.
    let int_id = find_named(&dag, "Int");
    let input_decl = dag.declaration(inputs[0]);
    let resolved_input_id = match &input_decl.connective {
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(id))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(id)) => *id,
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name)) => {
            panic!("input still unresolved: {name}")
        }
        _ => inputs[0],
    };
    let resolved_output_id = match &dag.declaration(output).connective {
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(id))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(id)) => *id,
        _ => output,
    };
    assert_eq!(resolved_input_id, int_id);
    assert_eq!(resolved_output_id, int_id);
}

#[test]
fn m17_brace_fn_body_accepts_type_labeled_record_literal() {
    // `parse_field_label` accepts `type` (KwType) as a field name; brace-fn
    // record lookahead must match `parse_record_literal`, not only `{ ident:`.
    let src = "fn typed_field() -> Int { type: 1 }";
    let tokens = v3_compiler::tokenize_for_test(src, "brace_fn_type_field.v3").expect("tokenize");
    let module = v3_compiler::parse_for_test(&tokens, "brace_fn_type_field.v3").expect("parse");
    let item = module
        .items
        .iter()
        .find(|i| matches!(i, v3_compiler::parse_surface::SurfaceItem::Fn { name, .. } if name == "typed_field"))
        .expect("typed_field fn");
    let v3_compiler::parse_surface::SurfaceItem::Fn { body, .. } = item else {
        panic!("expected Fn, got {item:?}");
    };
    assert!(
        matches!(body, v3_compiler::parse_surface::SurfaceExpr::Record { .. }),
        "expected Record literal body, got {body:?}"
    );
}

#[test]
fn m17_brace_fn_body_accepts_string_keyed_map_literal() {
    // Same `{ "key": expr }` disambiguation as `parse_data_item` — map literals
    // must not fall through to `parse_expr` (record parser rejects string keys).
    let src = "fn map_body(x: Int) -> Int { \"k\": x }";
    let tokens = v3_compiler::tokenize_for_test(src, "brace_fn_map_body.v3").expect("tokenize");
    let module = v3_compiler::parse_for_test(&tokens, "brace_fn_map_body.v3").expect("parse");
    let item = module
        .items
        .iter()
        .find(|i| matches!(i, v3_compiler::parse_surface::SurfaceItem::Fn { name, .. } if name == "map_body"))
        .expect("map_body fn");
    let v3_compiler::parse_surface::SurfaceItem::Fn { body, .. } = item else {
        panic!("expected Fn, got {item:?}");
    };
    assert!(
        matches!(body, v3_compiler::parse_surface::SurfaceExpr::Map { .. }),
        "expected Map literal body, got {body:?}"
    );
}

#[test]
fn m17_brace_fn_body_accepts_empty_record_literal() {
    // `{}` is a valid zero-field `SurfaceExpr::Record`; brace-fn record lookahead
    // must match `looks_like_record_literal` / `parse_data_item` (`{` then `}`).
    let src = "fn empty_rec() -> Int {}";
    let tokens = v3_compiler::tokenize_for_test(src, "brace_fn_empty_record.v3").expect("tokenize");
    let module = v3_compiler::parse_for_test(&tokens, "brace_fn_empty_record.v3").expect("parse");
    let item = module
        .items
        .iter()
        .find(|i| matches!(i, v3_compiler::parse_surface::SurfaceItem::Fn { name, .. } if name == "empty_rec"))
        .expect("empty_rec fn");
    let v3_compiler::parse_surface::SurfaceItem::Fn { body, .. } = item else {
        panic!("expected Fn, got {item:?}");
    };
    match body {
        v3_compiler::parse_surface::SurfaceExpr::Record { fields, .. } => {
            assert!(
                fields.is_empty(),
                "expected empty record literal fields, got {fields:?}"
            );
        }
        other => panic!("expected Record body, got {other:?}"),
    }
}

#[test]
fn m17_dag_corpus_brace_fn_stays_fn_external_body_at_parse_time() {
    // Parser gate (`fn_brace_body_parse_as_expression`): staged `.dag`
    // sources keep the legacy `FnExternalBody` surface even when the
    // brace contents are a single expressible `match` — bootstrap byte
    // stability and `ArrowBody::Unparsed` contracts for std/ remain
    // unchanged until an explicit corpus opt-in regen.
    let src = "fn staged(x: Int) -> Int {\n  match x { A => 1 }\n}";
    let tokens = v3_compiler::tokenize_for_test(src, "src/v3/std/corpus.dag").expect("tokenize");
    let module = v3_compiler::parse_for_test(&tokens, "src/v3/std/corpus.dag").expect("parse");
    let item = module
        .items
        .iter()
        .find(|i| match i {
            v3_compiler::parse_surface::SurfaceItem::FnExternalBody { name, .. }
            | v3_compiler::parse_surface::SurfaceItem::Fn { name, .. } => name == "staged",
            _ => false,
        })
        .expect("staged fn");
    assert!(
        matches!(
            item,
            v3_compiler::parse_surface::SurfaceItem::FnExternalBody { .. }
        ),
        "authority .dag sources must keep brace-bodied fn on FnExternalBody at parse time, got {item:?}"
    );
}

#[test]
fn m17_multi_statement_brace_fn_in_v3_falls_back_to_unparsed_arrow() {
    // Multi-statement / unparseable-as-single-expr brace bodies in `.v3`
    // hit `parse_expr` failure or non-exhausting-`}` recovery →
    // `FnExternalBody` → `ArrowBody::Unparsed` (opaque scaffold ratchet).
    //
    // M1(2.8) R14: user-range `ArrowBody::Unparsed` must still fail the
    // `compile_to_dag` boundary (`Err(Semantic)` with diagnostics) — do not
    // use `compile_any` here, which collapses clean vs semantic outcomes.
    // See `m18_r14_user_block_bodied_fn_is_rejected`.
    let src = "fn staged(x: Int) -> Int {\n  let y = x + 1\n  y + 1\n}";
    let result = compile_to_dag(src, "multi_stmt_fn.v3");
    assert!(
        result.is_err(),
        "user-range opaque fn body must fail compile_to_dag (R14), not compile cleanly"
    );
    let dag = match result {
        Err(v3_compiler::CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    let foo_id = find_named(&dag, "staged");
    let body = match &dag.declaration(foo_id).connective {
        TypeConnective::Arrow { body, .. } => body.clone(),
        other => panic!("expected Arrow, got {other:?}"),
    };
    assert!(
        matches!(body, ArrowBody::Unparsed(_)),
        "multi-statement brace body in .v3 should lower to ArrowBody::Unparsed, got {body:?}"
    );
}

#[test]
fn m17_multi_statement_brace_fn_parse_surface_stays_fn_external_body() {
    // Parse-phase receipt: multi-stmt / non-single-expr `.v3` brace bodies must
    // surface as `FnExternalBody` (brace-skip), not `ParseError` — scheduled
    // review claim "parser rejects instead of preserving" is false for this
    // path (`parse_fn_item` `Err` / partial-expr branches).
    let src = "fn staged(x: Int) -> Int {\n  let y = x + 1\n  y + 1\n}";
    let tokens =
        v3_compiler::tokenize_for_test(src, "multi_stmt_parse_surface.v3").expect("tokenize");
    let module =
        v3_compiler::parse_for_test(&tokens, "multi_stmt_parse_surface.v3").expect("parse");
    let item = module
        .items
        .iter()
        .find(|i| match i {
            v3_compiler::parse_surface::SurfaceItem::FnExternalBody { name, .. }
            | v3_compiler::parse_surface::SurfaceItem::Fn { name, .. } => name == "staged",
            _ => false,
        })
        .expect("staged fn item");
    assert!(
        matches!(
            item,
            v3_compiler::parse_surface::SurfaceItem::FnExternalBody { .. }
        ),
        "multi-line brace fn body must parse as FnExternalBody (fallback), got {item:?}"
    );
}

#[test]
fn m17_malformed_brace_expr_probe_returns_parse_error() {
    // Incomplete `{ x + }` is a malformed single-expression probe: `parse_expr`
    // fails after consuming a prefix, and the parser **propagates** that
    // diagnostic. Only brace bodies whose first inner token is `let` use
    // `FnExternalBody` on `parse_expr` `Err` (multi-statement scaffold); see
    // `fn_brace_body_expr_err_falls_back_to_external` in `parse_parser_body.txt`.
    let src = "fn broken(x: Int) -> Int { x + }";
    let tokens = v3_compiler::tokenize_for_test(src, "malformed_brace_probe.v3").expect("tokenize");
    let err = v3_compiler::parse_for_test(&tokens, "malformed_brace_probe.v3")
        .expect_err("malformed brace-body expr must fail parse");
    assert!(
        matches!(err, Diagnostic::ParseError { .. }),
        "expected ParseError for malformed probe, got {err:?}"
    );
    let result = compile_to_dag(src, "malformed_brace_lower.v3");
    assert!(
        matches!(result, Err(v3_compiler::CompileError::Parse(_))),
        "malformed brace fn must fail compile_to_dag at parse boundary, got {result:?}"
    );
}

#[test]
fn m17_data_declaration_produces_typed_declaration() {
    // Class 3 (QW2): `data foo: Int = { body }` parses as
    // `SurfaceItem::Data` and lowers to a declaration whose
    // connective resolves through `type_to_connective`. The
    // declaration is nameable via `declaration_by_name` and its
    // type fact flows forward.
    let src = "data foo: Int = { anything goes here }";
    let dag = compile_any(src, "data.v3");

    let foo_id = find_named(&dag, "foo");
    // `type_to_connective` on a bare `Int` emits
    // `Instantiation { template: Int_id, arguments: [] }`, not a
    // direct resolved-identifier wrapper. Either shape is
    // acceptable — the fact is "foo's type is Int" and we verify
    // it by walking one level.
    let foo_conn = dag.declaration(foo_id).connective.clone();
    let resolved = match &foo_conn {
        TypeConnective::Instantiation { template, .. } => *template,
        TypeConnective::Atom(AtomPayload::ResolvedByStructure(id))
        | TypeConnective::Atom(AtomPayload::ResolvedByName(id)) => *id,
        other => panic!("expected Instantiation or resolved identifier, got {other:?}"),
    };
    let int_id = find_named(&dag, "Int");
    assert_eq!(resolved, int_id, "data foo's type resolves to Int");
}

#[test]
fn m17_module_and_import_preserve_parsed_facts() {
    // Class 3 (QW3): `module` and `import` items emit real
    // `SurfaceItem` variants (no longer parser-absorbed), and
    // lowering accepts them without error even though they are
    // no-ops. Parsed facts survive into the SurfaceModule so M2
    // module scoping can consume them.
    let src = "module foo.bar\nimport baz.quux { Zot, Wat }\nlet x = 1";
    let dag = cached_compile_to_dag(src, "imports.v3");
    // The `let x = 1` item still binds, confirming the module /
    // import items didn't break lowering.
    let bind_x = bind_named(&dag, "x");
    assert!(
        matches!(
            dag.port(bind_x.value).state(),
            v3_compiler::dag::PortState::Resolved(_)
        ),
        "let binding after module/import compiles successfully"
    );
}

#[test]
fn m17_r9_arithmetic_operator_walks_to_algebra_field() {
    // M1(2.7) review round 9: resolve_operator_arrow must walk the
    // LHS type's algebra chain and read the operator's signature
    // from the actual algebra field declaration in std/algebra.dag,
    // substituting the receiver type parameter to the source type.
    //
    // For `let x = 1 + 2`, the walk path uses abstract `Int` algebra facts
    // (width-specialized literals still resolve through the construction-chain
    // carrier per `dsl/std/integer.dag`; gate #19 aligns fixed-width names as
    // Compose<Int, MachineWidth<…>>, not parallel OrderedRing<Word*> substrate).
    //
    // If the walk is wrong or returns Word64 (the old failing
    // mode), the operator output port's type won't match the Int
    // ports of the operands and the compile will fail with a
    // TypeMismatch. This test asserts the happy path compiles
    // cleanly and the operator output is typed as Int.
    let dag = cached_compile_to_dag("let x = 1 + 2", "test.v3");
    let bind_x = bind_named(&dag, "x");
    let int_shape = dag.int_shape().expect("Int cached at bootstrap");
    match dag.port(bind_x.value).state() {
        v3_compiler::dag::PortState::Resolved(ty) if *ty == int_shape => {}
        other => panic!("expected Bind(x).value to be Resolved(Int), got {other:?}"),
    }
}

#[test]
fn m17_r9_comparison_operator_walks_to_algebra_field() {
    // Same as the arithmetic test but for `<`: the walk looks up
    // the "lt" field on OrderedRing (added in M1(2.7) R9) and
    // reads its Arrow signature. OrderedRing.lt: fn(T, T) -> Bool.
    // Substituting receiver T → Int gives (Int, Int) -> Bool.
    let dag = cached_compile_to_dag("let y = 1 < 2", "test.v3");
    let bind_y = bind_named(&dag, "y");
    let bool_shape = dag.bool_shape().expect("Bool cached at bootstrap");
    match dag.port(bind_y.value).state() {
        v3_compiler::dag::PortState::Resolved(ty) if *ty == bool_shape => {}
        other => panic!("expected Bind(y).value to be Resolved(Bool), got {other:?}"),
    }
}

#[test]
fn m1_3_prb_data_item_record_body_lowers_structurally() {
    // M1(3) PR-B: `data foo: T = { field: literal, ... }` with a
    // record literal body whose type annotation resolves to a Conj
    // produces a declaration whose value_body is
    // Some(Structural { fields: [...] }) — not Unparsed. This is
    // the shape `src/v3/spec/rust.dag` uses to ground
    // every Realization structurally.
    //
    // Uses `LocalMeta` / `test_local_item` (not `Realization` /
    // `rust_int`) to avoid colliding with the rust.dag names that
    // bootstrap now loads. `find_named` uses first-match semantics
    // and would return the bootstrap declaration otherwise.
    let src = "\
type LocalMeta { target_name: String, cost: Int }
data test_local_item: LocalMeta = { target_name: \"Int\", cost: 1 }
";
    let dag = compile_any(src, "structural_data.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "structural data item should compile cleanly, got: {:?}",
        dag.diagnostics()
    );
    let item_id = find_named(&dag, "test_local_item");
    let meta_id = find_named(&dag, "LocalMeta");
    let item = dag.declaration(item_id);
    assert_eq!(
        item.meta_tag,
        Some(meta_id),
        "data item's meta_tag should point at its type annotation"
    );
    let value_body = item
        .value_body
        .as_ref()
        .expect("data item should have value_body");
    let fields = match value_body {
        v3_compiler::dag::ValueBody::Structural { fields } => fields,
        v3_compiler::dag::ValueBody::Unparsed(_) => panic!(
            "expected Structural value_body, got Unparsed — inhabitance checking didn't run or failed"
        ),
        v3_compiler::dag::ValueBody::Scalar(_) => panic!(
            "expected Structural value_body, got Scalar — record-shape expected"
        ),
        v3_compiler::dag::ValueBody::List(_) => {
            panic!("expected Structural value_body, got List — record-shape expected")
        }
        v3_compiler::dag::ValueBody::Map(_) => {
            panic!("expected Structural value_body, got Map — record-shape expected")
        }
    };
    // Fields are emitted in the type's declared order. PR-B
    // unwind: each field value is a `FieldValue::Literal` (since
    // `LocalMeta`'s declared field types are `String` / `Int` —
    // not the `DeclarationRef` sentinel), so we match against the
    // wrapped LiteralBits.
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].0, "target_name");
    match &fields[0].1 {
        v3_compiler::dag::FieldValue::Literal(v3_compiler::dag::LiteralBits::String(s)) => {
            assert_eq!(s, "Int")
        }
        other => panic!("expected Literal(String) for target_name, got {other:?}"),
    }
    assert_eq!(fields[1].0, "cost");
    match &fields[1].1 {
        v3_compiler::dag::FieldValue::Literal(v3_compiler::dag::LiteralBits::Int(n)) => {
            assert_eq!(n, "1")
        }
        other => panic!("expected Literal(Int) for cost, got {other:?}"),
    }
}

#[test]
fn m1_3_prb_data_item_with_extra_field_is_rejected() {
    // PR-B inhabitance check: a record body with a field not in
    // the type definition fails fail-closed with a diagnostic
    // anchored to that field's span. Uses `LocalMeta` /
    // `test_local_extra` to avoid colliding with rust.dag's
    // bootstrap names (see
    // `m1_3_prb_data_item_record_body_lowers_structurally`).
    let src = "\
type LocalMeta { target_name: String }
data test_local_extra: LocalMeta = { target_name: \"Int\", bogus: 42 }
";
    let dag = compile_any(src, "extra_field.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "extra field should fail fail-closed"
    );
}

#[test]
fn m1_3_prb_data_item_with_wrong_field_type_is_rejected() {
    // PR-B inhabitance check: a field whose literal value doesn't
    // match the declared type fails fail-closed. Uses local names
    // to avoid bootstrap collisions.
    let src = "\
type LocalMeta2 { cost: Int }
data test_local_bad: LocalMeta2 = { cost: \"not a number\" }
";
    let dag = compile_any(src, "wrong_type.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "wrong field type should fail fail-closed"
    );
}

#[test]
fn m1_3_prb_unwind_r1_behavior_realization_with_primitive_target_is_rejected() {
    // PR-B-unwind R1 narrowing check: the `DeclarationRef` sentinel
    // is too permissive on its own (it accepts ANY declaration).
    // The lower-time narrowing fail-closes when a realization
    // field's resolved target violates the per-(category, field)
    // constraint. Here: BehaviorRealization.target must be a
    // v3_l1 substrate marker, NOT a primitive type. Wiring a
    // BehaviorRealization to `Int` is the exact bad state the
    // narrowing was added to catch.
    //
    // Uses imports from rust.dag's already-loaded BehaviorRealization
    // meta-type so we don't need to re-declare it locally.
    let src = "\
data test_bad_behavior: BehaviorRealization = { target: Int, carrier: \"oops\", cost: 0 }
";
    let dag = compile_any(src, "bad_behavior.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "BehaviorRealization with a primitive target should fail-closed at lower time"
    );
}

#[test]
fn m1_3_prb_unwind_r1_type_realization_with_behavior_target_is_rejected() {
    // PR-B-unwind R1 narrowing check, dual case:
    // TypeRealization.target must be a primitive type, NOT a
    // v3_l1 substrate marker. Wiring a TypeRealization to `Bind`
    // is the dual bad state the narrowing catches.
    let src = "\
data test_bad_type: TypeRealization = { target: Bind, carrier: \"oops\", cost: 0 }
";
    let dag = compile_any(src, "bad_type.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "TypeRealization with a behavior marker target should fail-closed at lower time"
    );
}

#[test]
fn m1_3_prb_unwind_r1_callable_realization_with_primitive_target_is_rejected() {
    let src = "\
data test_bad_callable: CallableRealization = { target: Int, strategy: ListEmpty, cost: 0 }
";
    let dag = compile_any(src, "bad_callable_realization.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "CallableRealization with a non-callable target should fail-closed at lower time"
    );
}

#[test]
fn m1_3_prb_unwind_r1_pattern_realization_with_primitive_target_is_rejected() {
    let src = "\
data test_bad_pattern: PatternRealization = {
  target: Int,
  strategy: VectorList,
  scrutinee: \"{expr}\",
  empty_pattern: \"[]\",
  cons_pattern: \"[{head}, {tail} @ ..]\",
  head_expr: \"{head}\",
  tail_expr: \"{tail}\",
  cost: 0
}
";
    let dag = compile_any(src, "bad_pattern_realization.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "PatternRealization with a non-Disj target should fail-closed at lower time"
    );
}

#[test]
fn m1_3_prb_unwind_r1_type_instantiation_realization_with_monomorphic_target_is_rejected() {
    let src = "\
data test_bad_type_instantiation: TypeInstantiationRealization = {
  target: Int,
  carrier: \"Vec<{element}>\",
  cost: 0
}
";
    let dag = compile_any(src, "bad_type_instantiation_realization.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "TypeInstantiationRealization with a monomorphic target should fail-closed at lower time"
    );
}

#[test]
fn m1_3_prb_rust_dag_bootstrap_loads_structurally() {
    // M1(3) PR-B-unwind: verify rust.dag loaded cleanly during
    // Dag::new() bootstrap and that `rust_int_add` lowered to a
    // structural data item with **typed declaration references**
    // for the `target` and `op` fields (not strings). This is the
    // end-to-end receipt that the unwind succeeded: every
    // realization now carries DeclarationId edges to the
    // declarations it realizes, and consumers (emit_rust) read
    // those edges directly without name-string dispatch.
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "rust.dag should bootstrap cleanly, got: {:?}",
        dag.diagnostics()
    );

    // The unwind splits Realization into three meta-types. The
    // `rust_int_add` declaration is an OperatorRealization.
    let op_realization_meta = find_named(&dag, "OperatorRealization");
    let rust_int_add_id = find_named(&dag, "rust_int_add");
    let rust_int_add = dag.declaration(rust_int_add_id);
    assert_eq!(
        rust_int_add.meta_tag,
        Some(op_realization_meta),
        "rust_int_add's meta_tag must point at OperatorRealization"
    );
    let fields = match rust_int_add
        .value_body
        .as_ref()
        .expect("rust_int_add must carry a structural value_body")
    {
        v3_compiler::dag::ValueBody::Structural { fields } => fields,
        v3_compiler::dag::ValueBody::Unparsed(_) => {
            panic!("rust_int_add's value_body must be Structural, not Unparsed")
        }
        v3_compiler::dag::ValueBody::Scalar(_) => {
            panic!("rust_int_add's value_body must be Structural, not Scalar")
        }
        v3_compiler::dag::ValueBody::List(_) => {
            panic!("rust_int_add's value_body must be Structural, not List")
        }
        v3_compiler::dag::ValueBody::Map(_) => {
            panic!("rust_int_add's value_body must be Structural, not Map")
        }
    };
    // Fields appear in OperatorRealization's declared order:
    // language, target, op, carrier, cost.
    assert_eq!(fields.len(), 5);

    // language → rust_language (typed Reference to the rust_language
    // data declaration). Replaces the prior TargetLanguage enum variant;
    // target ownership is now carried by a typed edge, not a compiler-
    // side variant roster (INVARIANTS.md E-6).
    assert_eq!(fields[0].0, "language");
    let rust_language_id = find_named(&dag, "rust_language");
    match &fields[0].1 {
        v3_compiler::dag::FieldValue::Reference(id) => assert_eq!(
            *id, rust_language_id,
            "rust_int_add's language should reference the rust_language declaration"
        ),
        other => panic!("expected Reference for language, got {other:?}"),
    }

    // target → Int (typed Reference)
    assert_eq!(fields[1].0, "target");
    let int_id = find_named(&dag, "Int");
    match &fields[1].1 {
        v3_compiler::dag::FieldValue::Reference(id) => assert_eq!(
            *id, int_id,
            "rust_int_add's target should reference the Int declaration"
        ),
        other => panic!("expected Reference for target, got {other:?}"),
    }

    // op → OrderedRing.add (typed Reference resolved via
    // dotted-path lowering). Walk OrderedRing to find its `add`
    // child declaration id and compare structurally.
    assert_eq!(fields[2].0, "op");
    let ordered_ring_id = find_named(&dag, "OrderedRing");
    let ordered_ring_add_id = match &dag.declaration(ordered_ring_id).connective {
        TypeConnective::Conj { children } => {
            children
                .iter()
                .find(|f| f.label == "add")
                .expect("OrderedRing has an add field")
                .ty
        }
        other => panic!("OrderedRing should be a Conj, got {other:?}"),
    };
    match &fields[2].1 {
        v3_compiler::dag::FieldValue::Reference(id) => assert_eq!(
            *id, ordered_ring_add_id,
            "rust_int_add's op should reference OrderedRing.add"
        ),
        other => panic!("expected Reference for op, got {other:?}"),
    }

    // carrier → "({lhs} + {rhs})" (full-expression template; PR #681 unified all
    // OperatorRealization carriers to self-describing templates, dissolving the
    // former binary_op / carrier dual-authority split)
    assert_eq!(fields[3].0, "carrier");
    assert!(matches!(
        &fields[3].1,
        v3_compiler::dag::FieldValue::Literal(v3_compiler::dag::LiteralBits::String(s)) if s == "({lhs} + {rhs})"
    ));

    // cost → 1 (Literal Int)
    assert_eq!(fields[4].0, "cost");
    assert!(matches!(
        &fields[4].1,
        v3_compiler::dag::FieldValue::Literal(v3_compiler::dag::LiteralBits::Int(s)) if s == "1"
    ));
}

#[test]
fn rust_clean_emission_verifier_declares_structural_output_policy() {
    let dag = Dag::new();
    let verifier_policy = find_named(&dag, "VerifierOutputPolicy");
    let ignore_output = match &dag.declaration(verifier_policy).connective {
        TypeConnective::Disj { variants } => {
            variants
                .iter()
                .find(|variant| variant.label == "IgnoreVerifierOutput")
                .expect("VerifierOutputPolicy.IgnoreVerifierOutput exists")
                .ty
        }
        other => panic!("VerifierOutputPolicy must be a Disj, got {other:?}"),
    };
    let post_emit_verifier = dag
        .declaration_by_name("PostEmitVerifier")
        .expect("PostEmitVerifier type exists");
    let verifier_fields = match &dag.declaration(post_emit_verifier.id).connective {
        TypeConnective::Conj { children } => children,
        other => panic!("PostEmitVerifier must be a Conj, got {other:?}"),
    };
    assert_eq!(
        field(verifier_fields, "output_policy").ty,
        verifier_policy,
        "PostEmitVerifier.output_policy must point at VerifierOutputPolicy"
    );

    let rust_clean_emission = dag
        .declaration_by_name("rust_clean_emission")
        .expect("rust_clean_emission exists");
    let clean_fields = match rust_clean_emission
        .value_body
        .as_ref()
        .expect("rust_clean_emission must carry a value_body")
    {
        v3_compiler::dag::ValueBody::Structural { fields } => fields,
        other => panic!("rust_clean_emission must be Structural, got {other:?}"),
    };
    let post_emit_value = clean_fields
        .iter()
        .find(|(label, _)| label == "post_emit_verifier")
        .expect("rust_clean_emission.post_emit_verifier field exists");
    let verifier_record = match &post_emit_value.1 {
        v3_compiler::dag::FieldValue::Record(fields) => fields,
        other => panic!("post_emit_verifier must be a Record, got {other:?}"),
    };
    let output_policy = verifier_record
        .iter()
        .find(|(label, _)| label == "output_policy")
        .expect("post_emit_verifier.output_policy field exists");
    match &output_policy.1 {
        v3_compiler::dag::FieldValue::Variant {
            constructor,
            payload,
        } => {
            assert_eq!(
                *constructor, ignore_output,
                "rust_clean_emission should declare IgnoreVerifierOutput for rustc"
            );
            assert!(
                payload.is_empty(),
                "IgnoreVerifierOutput must stay payload-free"
            );
        }
        other => panic!("output_policy must be a Variant, got {other:?}"),
    }
}

#[test]
fn instantiation_arguments_participate_in_type_shape_equivalence() {
    let src = "\
fn expect_ints(xs: List<Int>) -> Int = 0
let bad: Int = expect_ints(singleton(true))
";
    let dag = compile_any(src, "instantiation_shape_equivalence.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "List<Int> and List<Bool> should not compare equal during inference"
    );
}

#[test]
fn m17_r9_data_item_has_unparsed_value_body_scaffold() {
    // M1(2.7) review round 9, QW2 + structural scaffold honesty:
    // data items must be structurally distinguishable from type
    // aliases. `data foo: Int = {...}` lowers to a declaration
    // whose connective is the resolved type AND whose value_body
    // is Some(ValueBody::Unparsed(body_span)). A plain
    // `type foo = Int` has value_body = None.
    //
    // M1(3) PR-B note: `{ 42 }` is not a record literal shape
    // (the lookahead `{`, `Ident`, `:` fails because the second
    // token is an IntLit, not an Ident). The parser falls back
    // to brace-skip and the body lands as `Unparsed`. If this
    // test ever starts producing `Structural`, the parser's
    // record-literal lookahead has widened unexpectedly.
    let dag = compile_any("data foo: Int = { 42 }", "data.v3");
    let foo_id = find_named(&dag, "foo");
    let foo_decl = dag.declaration(foo_id);
    // Structural check: the value_body field carries an Unparsed
    // scaffold with the body's source span preserved.
    match &foo_decl.value_body {
        Some(v3_compiler::dag::ValueBody::Unparsed(_span)) => {}
        Some(v3_compiler::dag::ValueBody::Structural { .. }) => panic!(
            "`{{ 42 }}` should not parse as a record literal — the lookahead \
             requires `{{`, `Ident`, `:` and the second token here is an IntLit"
        ),
        Some(v3_compiler::dag::ValueBody::Scalar(_)) => panic!(
            "`{{ 42 }}` has leading `{{` and so must take the brace-skip path, \
             not the scalar-expression path; landed as Scalar unexpectedly"
        ),
        Some(v3_compiler::dag::ValueBody::List(_)) => panic!(
            "`{{ 42 }}` has leading `{{` and so must take the brace-skip path, \
             not the list-expression path; landed as List unexpectedly"
        ),
        Some(v3_compiler::dag::ValueBody::Map(_)) => panic!(
            "`{{ 42 }}` has leading `{{` and so must take the brace-skip path, \
             not the map-expression path; landed as Map unexpectedly"
        ),
        None => panic!(
            "data item must have value_body = Some(Unparsed), got None — \
             the declaration is structurally identical to a type alias"
        ),
    }
}

#[test]
fn m18_match_on_user_sum_type_compiles() {
    // M1(2.8): match lowers to a Branch node with one Path per arm,
    // each carrying a BranchPattern::ResolvedVariant(DeclarationId)
    // after inference. The scrutinee's declaration must be a Disj;
    // each arm's pattern name must match one of the Disj's variant
    // labels (scoped against the scrutinee type, not globally).
    //
    // This test uses literal RHS bodies so it doesn't hit the
    // variant-expression RHS class-5 gap (bare `True`/`False` as
    // expressions).
    //
    // The `Plus` in a call site like `always_zero(Plus)` is a variant
    // expression — class-5 gap. Compile only the fn body.
    let src = "\
type Sign = Plus | Minus
fn always_zero(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }
";
    let dag = compile_any(src, "match.v3");
    let bind = bind_named(&dag, "always_zero");
    // Walk the Bind's value port back to its producer Behavior.
    let body_node_id = dag
        .port(bind.value)
        .produced_by
        .expect("Bind value port has a producer");
    let branch = match dag.node(body_node_id) {
        Behavior::Branch(b) => b,
        other => panic!("expected Branch at match root, got {other:?}"),
    };
    assert_eq!(branch.paths.len(), 2, "two arms produce two paths");
    for path in &branch.paths {
        match &path.pattern {
            BranchPattern::ResolvedVariant(_) => {}
            other => panic!(
                "expected ResolvedVariant after infer, got {other:?} — \
                 pattern resolution pass did not run or failed"
            ),
        }
    }
}

#[test]
fn infer_variant_constructor_call_returns_parent_sum_type() {
    let src = "\
type Sign = Plus | Minus
fn classify(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }
let total: Int = classify(Plus)
";
    let dag = compile_any(src, "variant_constructor_parent_sum.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "variant constructor call should type-check as the parent sum, got {:?}",
        dag.diagnostics()
    );
    assert_eq!(bind_value_type_decl(&dag, "total"), find_named(&dag, "Int"));
}

#[test]
fn m18_match_on_non_disj_scrutinee_is_rejected() {
    // M1(2.8): the Branch input relaxation accepts Bool OR any
    // Disj. String is Instantiation(FreeMonoid<Char>), NOT Disj —
    // match on a String scrutinee must fail.
    let src = "\
type Sign = Plus | Minus
fn bad(s: String) -> Int = match s { Plus => 0, Minus => 1 }
";
    let dag = compile_any(src, "bad_match.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "match on String scrutinee should produce a diagnostic"
    );
}

#[test]
fn m18_r11_non_exhaustive_match_is_rejected() {
    // M1(2.8) R11: coproduct elimination must be exhaustive. A
    // match that omits a variant of the scrutinee's Disj fails
    // fail-closed.
    let src = "\
type Sign = Plus | Minus
fn bad(s: Sign) -> Int = match s { Plus => 0 }
";
    let dag = compile_any(src, "non_exhaustive.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "non-exhaustive match must emit a diagnostic (missing Minus arm)"
    );
}

#[test]
fn m18_r11_duplicate_match_arm_is_rejected() {
    // M1(2.8) R11: each variant of the scrutinee's Disj must
    // appear in at most one arm. Duplicated arms are fail-closed.
    let src = "\
type Sign = Plus | Minus
fn bad(s: Sign) -> Int = match s { Plus => 0, Plus => 1, Minus => 2 }
";
    let dag = compile_any(src, "duplicate_arm.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "duplicate match arm must emit a diagnostic"
    );
}

#[test]
fn m18_r15_match_on_aliased_sum_type_compiles() {
    // M1(2.8) R15: Branch gate and pattern resolution walk
    // through alias / instantiation edges to find the underlying
    // Disj. Before R15, both checks read the immediate
    // connective and rejected `type Hue = Color` because Hue's
    // connective is Instantiation { template: Color }, not Disj.
    //
    // Match arms resolve against Color's variants regardless of
    // whether the scrutinee port type is Color or Hue.
    let src = "\
type Color = Red | Green | Blue
type Hue = Color
fn classify(h: Hue) -> Int = match h { Red => 0, Green => 1, Blue => 2 }
";
    let dag = cached_compile_to_dag(src, "aliased_sum.v3");
    assert!(dag.diagnostics().is_empty());

    // The Branch exists and its pattern resolution ran through
    // the alias walk — each path should carry a ResolvedVariant.
    // Scope to the user-source branch: bootstrap modules (notably
    // `src/v3/std/algebra.dag`) now push Branch nodes for their own
    // if/else lowering, so first-match is no longer reliable.
    let branch = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_branch)
        .find(|b| b.span.file == "aliased_sum.v3")
        .expect("Branch exists");
    assert_eq!(branch.paths.len(), 3);
    for path in &branch.paths {
        match &path.pattern {
            BranchPattern::ResolvedVariant(_) => {}
            other => panic!("expected ResolvedVariant, got {other:?}"),
        }
    }
}

#[test]
fn m18_r15_non_exhaustive_match_on_aliased_sum_type_is_rejected() {
    // R15 negative: aliased sum types still go through the
    // exhaustiveness check. A match that covers only 2 of 3
    // Color variants via Hue should still fail fail-closed.
    let src = "\
type Color = Red | Green | Blue
type Hue = Color
fn classify(h: Hue) -> Int = match h { Red => 0, Green => 1 }
";
    let dag = compile_any(src, "aliased_non_exhaustive.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "non-exhaustive match via alias should fail"
    );
}

#[test]
fn m18_r14_user_block_bodied_fn_is_rejected() {
    // M1(2.8) R14: user-range `ArrowBody::Unparsed` scaffolds must fail-closed
    // (`compile_to_dag` → `Err(Semantic)`) with the scaffold-rejection diagnostic.
    //
    // `{ junk }` is no longer a reliable `FnExternalBody` boundary fixture: on
    // user `.v3`, a lone identifier parses as a real `SurfaceExpr` and lowers
    // through `UserDefined`, so the R14 unparsed-body sweep never runs. Use a
    // multi-statement brace body instead — it hits `FnExternalBody` →
    // `ArrowBody::Unparsed` like `m17_multi_statement_brace_fn_in_v3_falls_back_to_unparsed_arrow`.
    let src = "fn foo(x: Int) -> Int {\n  let y = x + 1\n  y\n}";
    let result = compile_to_dag(src, "user_r14_opaque_fn.v3");
    assert!(
        result.is_err(),
        "user-range fn with opaque body must fail compile_to_dag"
    );
    let dag = match result {
        Err(v3_compiler::CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| {
            matches!(
                diag,
                Diagnostic::ResolveError { ref name, .. } if name.contains("opaque block body")
            )
        }),
        "expected R14 opaque-fn-body scaffold rejection diagnostic, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn m18_r14_user_data_with_opaque_body_is_rejected() {
    // M1(2.8) R14: analogous to the fn case. User-range
    // `data foo: Int = { junk }` must fail-closed because the
    // ValueBody::Unparsed scaffold was designed for
    // std/bootstrap data tables (kernel_algebra_profile et al.),
    // not user code. User code should not ship opaque data
    // bodies the compiler cannot validate.
    let result = compile_to_dag("data foo: Int = { junk }", "user_data.v3");
    assert!(
        result.is_err(),
        "user-range data with opaque body must fail compile_to_dag"
    );
}

#[test]
fn m18_r13_mutual_recursion_poisons_callers() {
    // Mutual recursion now compiles when a shared descent witness
    // exists, so the fail-closed regression shape is an SCC where at
    // least one intra-cluster edge preserves the measure. Downstream
    // callers must still cascade to Unresolved.
    let src = "\
fn a(n: Int) -> Int = b(n)
fn b(n: Int) -> Int = a(n - 1)
let c = a(1)
";
    let dag = compile_any(src, "mutual_recursion_cascade.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "non-terminating mutual recursion should produce a diagnostic"
    );
    let bind_c = bind_named(&dag, "c");
    assert!(
        matches!(
            dag.port(bind_c.value).state(),
            v3_compiler::dag::PortState::Unresolved
        ),
        "caller of a mutually-recursive fn must cascade to Unresolved; got {:?}",
        dag.port(bind_c.value).state()
    );
    assert_eq!(
        dag.clusters().len(),
        0,
        "failed mutual recursion must not materialize a cluster witness"
    );
    // Scope the Loop check to this fixture — bootstrap std modules
    // (e.g. `src/v3/std/algebra.dag`) correctly materialize Loop
    // nodes for their own structural recursion, so a global
    // `filter_map(as_loop).next().is_none()` no longer reflects the
    // test's intent ("user's non-terminating SCC didn't fabricate a
    // Loop").
    assert!(
        dag.nodes()
            .iter()
            .filter_map(Behavior::as_loop)
            .all(|l| l.span.file != "mutual_recursion_cascade.v3"),
        "failed mutual recursion must not fabricate Loop materialization"
    );
}

#[test]
fn m18_r12_invalid_match_cascades_to_downstream_callers() {
    // M1(2.8) R12: pattern resolution and coverage checks run
    // inside the fixpoint loop so non-exhaustive / duplicate
    // match failures cascade to downstream consumers. Before
    // R12, resolve_branch_patterns ran after the main loop had
    // already typed every downstream port — the Branch's output
    // would flip to Unresolved AFTER consumers had locked in
    // Resolved types, leaking invalid types through the
    // compile boundary.
    //
    // Regression shape: non-exhaustive match in a fn body that
    // a let binding then consumes. After R12, compile_to_dag
    // returns Err because the let-port cascades to Unresolved.
    let src = "\
type Sign = Plus | Minus
fn always_zero(s: Sign) -> Int = match s { Plus => 0 }
";
    // The fn itself doesn't have a downstream caller here (to
    // avoid needing variant-expression parsing for `Plus`/`Minus`
    // as arguments at the call site — class-5 gap #4). Instead
    // we verify the structural shape: the Branch's output port
    // and the Bind's value port (both cascading targets) must
    // be Unresolved after compile_to_dag returns.
    let result = compile_to_dag(src, "cascade.v3");
    assert!(
        result.is_err(),
        "non-exhaustive match should fail the compile boundary"
    );
    // Pull the Dag out of the Err and walk structurally.
    let dag = match result {
        Err(v3_compiler::CompileError::Semantic(dag)) => dag,
        other => panic!("expected CompileError::Semantic, got {other:?}"),
    };
    let bind = bind_named(&dag, "always_zero");
    // The Bind's value port is the fn body's return port,
    // which is the Branch's output. It must be Unresolved —
    // the cascade from the coverage-check failure reached here.
    assert!(
        matches!(
            dag.port(bind.value).state(),
            v3_compiler::dag::PortState::Unresolved
        ),
        "non-exhaustive match body must cascade to Bind value port; got {:?}",
        dag.port(bind.value).state()
    );
}

#[test]
fn m18_r11_three_variant_exhaustive_match_compiles() {
    // R11 sanity: a match covering all three variants of a
    // three-constructor sum type compiles cleanly. This also
    // exercises the coverage check on a non-boolean sum size.
    let src = "\
type Ternary = Low | Mid | High
fn level(t: Ternary) -> Int = match t { Low => 0, Mid => 1, High => 2 }
";
    let dag = cached_compile_to_dag(src, "ternary.v3");
    assert!(dag.diagnostics().is_empty());
}

#[test]
fn m18_match_with_unknown_variant_is_rejected() {
    // M1(2.8): variant resolution scoped against the scrutinee's
    // Disj. An arm referencing a variant name that isn't declared
    // on the scrutinee's type fails fail-closed.
    let src = "\
type Sign = Plus | Minus
fn bad(s: Sign) -> Int = match s { Plus => 0, Zero => 1 }
";
    let dag = compile_any(src, "unknown_variant.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "unknown variant in match arm should produce a diagnostic"
    );
}

#[test]
fn m18_if_then_else_populates_branch_pattern() {
    // M1(2.8): `if`/`else` lowering no longer relies on positional
    // convention (paths[0]=then, paths[1]=else). Each path carries
    // an explicit BranchPattern — UnresolvedVariant{name:"True"}
    // for the then-branch and {name:"False"} for the else-branch,
    // resolved to Bool's True/False variant declarations after
    // inference.
    let dag = cached_compile_to_dag("let r = if 1 > 0 then 10 else 20", "if.v3");
    let branch = dag
        .nodes()
        .iter()
        .find_map(Behavior::as_branch)
        .expect("Branch exists");
    assert_eq!(branch.paths.len(), 2);
    // Both paths must be ResolvedVariant post-infer.
    for path in &branch.paths {
        match &path.pattern {
            BranchPattern::ResolvedVariant(_) => {}
            other => panic!("if/else paths should resolve to Bool variants, got {other:?}"),
        }
    }
}

#[test]
fn m18_bool_is_structurally_a_disj() {
    // Substrate sanity check: the Branch input relaxation hinges
    // on Bool being a Disj declaration. If types.dag ever changes
    // Bool to a non-Disj shape, `if`/`else` will break — this
    // test pins the invariant.
    let dag = Dag::new();
    let bool_id = find_named(&dag, "Bool");
    match &dag.declaration(bool_id).connective {
        TypeConnective::Disj { variants } => {
            assert!(variants.iter().any(|f| f.label == "True"));
            assert!(variants.iter().any(|f| f.label == "False"));
        }
        other => panic!("Bool must be a Disj for if/else to type-check, got {other:?}"),
    }
}

#[test]
fn m17_r9_fn_param_uses_single_declaration_id_for_arrow_and_port() {
    // M1(2.7) R9 double-lower fix: for a fn parameter like
    // `fn f(x: Foo) -> Foo`, the Arrow's input DeclarationId and
    // the param port's TypeShape must share the same underlying
    // id. Before R9, the param was lowered TWICE — once via
    // type_to_declaration_id for the Arrow, once again inside
    // lower_type_for_port for the port — producing two anonymous
    // declarations for compound types that wouldn't structurally
    // match later in infer.
    //
    // Guard: compile a fn with a user-declared record type as
    // param and return. The fn's Arrow declaration's input must
    // point at the same DeclarationId as its Bind's param port
    // TypeShape.
    let src = "\
type Point { x: Int y: Int }
fn identity(p: Point) -> Point = p
";
    let dag = cached_compile_to_dag(src, "test.v3");

    // Find `identity`'s declaration — its connective is Arrow.
    let identity_id = find_named(&dag, "identity");
    let (arrow_inputs, _) = match &dag.declaration(identity_id).connective {
        TypeConnective::Arrow { inputs, output, .. } => (inputs.clone(), *output),
        other => panic!("expected Arrow, got {other:?}"),
    };
    assert_eq!(arrow_inputs.len(), 1, "identity takes one param");
    let arrow_input_id = arrow_inputs[0];

    // Find the corresponding Bind's first param port and read its
    // TypeShape. The port's DeclarationId must equal the Arrow's
    // input DeclarationId.
    let bind = bind_named(&dag, "identity");
    let first_param_port = bind.params.first().expect("one param");
    let param_shape = match dag.port(*first_param_port).state() {
        v3_compiler::dag::PortState::Resolved(ty) => *ty,
        other => panic!("param port should be Resolved, got {other:?}"),
    };
    assert_eq!(
        param_shape.declaration, arrow_input_id,
        "Arrow input and param port TypeShape must share the same DeclarationId \
         — a split indicates the R9 double-lower regression"
    );
}

#[test]
fn m17_r9_type_alias_has_no_value_body() {
    // Converse of the data test: `type foo = Int` (a pure type
    // alias) must have value_body = None, so consumers can
    // discriminate on this field alone.
    let dag = cached_compile_to_dag("type Foo = Int", "alias.v3");
    let foo_id = find_named(&dag, "Foo");
    let foo_decl = dag.declaration(foo_id);
    assert!(
        foo_decl.value_body.is_none(),
        "type alias must have value_body = None, got {:?}",
        foo_decl.value_body
    );
}

#[test]
fn m17_template_argument_stub_branch_is_gone() {
    // Class 3 (QW4): `build_template_arguments` no longer produces
    // self-referential `TemplateArgument { parameter: value, value }`
    // entries for stub templates. When the template is an
    // unresolved stub, the arguments list is empty — the stub's own
    // diagnostic is the authoritative failure, and the instantiation
    // carries no parameter-binding records.
    //
    // Trigger: user code references an unknown parameterized type.
    // The `Unknown<Int>` instantiation's declaration should have
    // `arguments: []`, not a self-referential pair.
    let src = "type Foo = Unknown<Int>";
    let dag = compile_any(src, "stub.v3");

    // Walk every Instantiation declaration; any whose template
    // is an unresolved stub must have an empty arguments list.
    let mut found_stub_inst = false;
    for decl in dag.declarations() {
        if let TypeConnective::Instantiation {
            template,
            arguments,
        } = &decl.connective
        {
            let template_is_stub = matches!(
                &dag.declaration(*template).connective,
                TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(_))
            );
            if template_is_stub {
                found_stub_inst = true;
                assert!(
                    arguments.is_empty(),
                    "stub template Instantiation must carry no TemplateArguments, found {} entries",
                    arguments.len()
                );
            }
        }
    }
    // The test is only meaningful if we actually found a stub
    // instantiation to inspect — otherwise it's vacuous.
    assert!(
        found_stub_inst,
        "test did not observe a stub Instantiation; the negative invariant is vacuous"
    );
}

#[test]
fn prereq0_single_level_higher_order_call_binds_function_argument() {
    let src = "\
fn step(x: Int) -> Int = x
fn apply<T>(x: T, f: fn(T) -> T) -> T = f(x)
let y = apply(3, step)
";
    let dag = compile_any(src, "prereq0_single_level.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "single-level higher-order call should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let int_id = find_named(&dag, "Int");
    let step_id = find_named(&dag, "step");
    let apply_id = find_named(&dag, "apply");
    let apply_decl = dag.declaration(apply_id);
    let apply_t = apply_decl.type_params[0];
    let apply_f = match &apply_decl.connective {
        TypeConnective::Arrow { inputs, .. } => inputs[1],
        other => panic!("apply should be Arrow, got {other:?}"),
    };

    let mut saw_apply_instantiation = false;
    let mut saw_param_call = false;
    for node in dag.nodes() {
        let Behavior::Transform(transform) = node else {
            continue;
        };
        let TransformTarget::Callable(target) = transform.target else {
            continue;
        };
        let body_call_targets_param_slot = if target == apply_f {
            true
        } else {
            matches!(
                &dag.declaration(target).connective,
                TypeConnective::Instantiation { template, .. } if *template == apply_f
            )
        };
        if body_call_targets_param_slot {
            saw_param_call = true;
            assert_eq!(
                transform.inputs.len(),
                1,
                "calling the function-typed parameter should still pass exactly one runtime arg"
            );
            continue;
        }
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &dag.declaration(target).connective
        else {
            continue;
        };
        if *template != apply_id {
            continue;
        }
        saw_apply_instantiation = true;
        assert_eq!(
            transform.inputs.len(),
            1,
            "function-typed call arguments should bind through Instantiation, not runtime ports"
        );
        assert!(
            arguments
                .iter()
                .any(|arg| arg.parameter == apply_f && arg.value == step_id),
            "expected apply's function slot to bind to `step`, got {arguments:?}"
        );
        assert!(
            arguments
                .iter()
                .any(|arg| arg.parameter == apply_t && arg.value == int_id),
            "expected apply's type parameter T to bind to Int via function-signature matching, got {arguments:?}"
        );
    }

    assert!(
        saw_apply_instantiation,
        "did not observe the top-level `apply(3, step)` instantiation"
    );
    assert!(
        saw_param_call,
        "did not observe the body call `f(x)` lowered against apply's function-parameter slot"
    );
    assert_eq!(
        bind_value_type_decl(&dag, "y"),
        int_id,
        "the top-level binding should infer Int"
    );
}

#[test]
fn prereq0_nested_higher_order_call_threads_subststack_chain() {
    let src = "\
fn step(x: Int) -> Int = x
fn apply<T>(x: T, f: fn(T) -> T) -> T = f(x)
fn twice<T>(x: T, f: fn(T) -> T) -> T = apply(apply(x, f), f)
let y = twice(3, step)
";
    let dag = compile_any(src, "prereq0_nested.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "nested higher-order call should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let int_id = find_named(&dag, "Int");
    let step_id = find_named(&dag, "step");
    let apply_id = find_named(&dag, "apply");
    let twice_id = find_named(&dag, "twice");

    let apply_decl = dag.declaration(apply_id);
    let apply_t = apply_decl.type_params[0];
    let apply_f = match &apply_decl.connective {
        TypeConnective::Arrow { inputs, .. } => inputs[1],
        other => panic!("apply should be Arrow, got {other:?}"),
    };

    let twice_decl = dag.declaration(twice_id);
    let twice_t = twice_decl.type_params[0];
    let twice_f = match &twice_decl.connective {
        TypeConnective::Arrow { inputs, .. } => inputs[1],
        other => panic!("twice should be Arrow, got {other:?}"),
    };

    let mut apply_instantiations = 0usize;
    let mut saw_twice_instantiation = false;
    for node in dag.nodes() {
        let Behavior::Transform(transform) = node else {
            continue;
        };
        let TransformTarget::Callable(target) = transform.target else {
            continue;
        };
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &dag.declaration(target).connective
        else {
            continue;
        };
        if *template == apply_id {
            apply_instantiations += 1;
            assert_eq!(
                transform.inputs.len(),
                1,
                "each nested `apply(..., f)` call should keep only the value arg as a runtime input"
            );
            assert!(
                arguments
                    .iter()
                    .any(|arg| arg.parameter == apply_f && arg.value == twice_f),
                "expected apply's function slot to bind to twice's function slot, got {arguments:?}"
            );
            assert!(
                arguments
                    .iter()
                    .any(|arg| arg.parameter == apply_t && arg.value == twice_t),
                "expected apply's type parameter to bind to twice's type parameter, got {arguments:?}"
            );
        }
        if *template == twice_id {
            saw_twice_instantiation = true;
            assert_eq!(
                transform.inputs.len(),
                1,
                "the top-level `twice(3, step)` call should keep only the value arg as a runtime input"
            );
            assert!(
                arguments
                    .iter()
                    .any(|arg| arg.parameter == twice_f && arg.value == step_id),
                "expected twice's function slot to bind to `step`, got {arguments:?}"
            );
            assert!(
                arguments
                    .iter()
                    .any(|arg| arg.parameter == twice_t && arg.value == int_id),
                "expected twice's type parameter to bind to Int, got {arguments:?}"
            );
        }
    }

    assert!(
        apply_instantiations >= 2,
        "expected both nested `apply` calls to instantiate through SubstStack bindings"
    );
    assert!(
        saw_twice_instantiation,
        "did not observe the top-level `twice(3, step)` instantiation"
    );
    assert_eq!(
        bind_value_type_decl(&dag, "y"),
        int_id,
        "the top-level nested higher-order binding should infer Int"
    );
}

#[test]
fn prereq3_annotated_lambda_direct_call_uses_declared_params_only() {
    let src = "\
let f: fn(Int) -> Int = |x| x
let y = f(42)
";
    let dag = compile_any(src, "prereq3_direct_lambda.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "annotated direct lambda call should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let int_id = find_named(&dag, "Int");
    let mut saw_lambda_call = false;
    for node in dag.nodes() {
        let Behavior::Transform(transform) = node else {
            continue;
        };
        let TransformTarget::Callable(target) = transform.target else {
            continue;
        };
        let TypeConnective::Arrow { body, .. } = &dag.declaration(target).connective else {
            continue;
        };
        let ArrowBody::UserDefined(bind_id) = body else {
            continue;
        };
        let bind = (*bind_id).bind(&dag);
        if bind.name.starts_with("__anon_lambda_") {
            saw_lambda_call = true;
            assert_eq!(
                transform.inputs.len(),
                1,
                "direct lambda call should pass only the declared runtime argument"
            );
            assert_eq!(
                bind.params.len(),
                1,
                "non-capturing lambda bind should carry exactly one declared parameter"
            );
        }
    }

    assert!(
        saw_lambda_call,
        "did not observe the direct lambda call target"
    );
    assert_eq!(bind_value_type_decl(&dag, "y"), int_id);
}

#[test]
fn prereq0_5_fold_generic_call_synthesizes_implicit_template_bindings() {
    let src = "\
let total: Int = fold(singleton(1), 0, |acc, x| acc + x)
";
    let dag = compile_any(src, "prereq0_5_fold.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "implicit generic fold call should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let int_id = find_named(&dag, "Int");
    let singleton_id = find_named(&dag, "singleton");
    let fold_id = find_named(&dag, "fold");
    let fold_decl = dag.declaration(fold_id);
    let fold_t = fold_decl.type_params[0];
    let fold_u = fold_decl.type_params[1];
    let singleton_t = dag.declaration(singleton_id).type_params[0];

    let singleton_instantiations = callable_instantiation_arguments(&dag, singleton_id);
    assert!(
        singleton_instantiations.iter().any(|arguments| {
            arguments
                .iter()
                .any(|arg| arg.parameter == singleton_t && arg.value == int_id)
        }),
        "expected singleton(1) to instantiate element := Int"
    );

    let fold_instantiations = callable_instantiation_arguments(&dag, fold_id);
    assert!(
        fold_instantiations.iter().any(|arguments| {
            arguments
                .iter()
                .any(|arg| arg.parameter == fold_t && arg.value == int_id)
                && arguments
                    .iter()
                    .any(|arg| arg.parameter == fold_u && arg.value == int_id)
        }),
        "expected fold(singleton(1), 0, ...) to infer T := Int and U := Int"
    );

    let lambda_binds: Vec<_> = dag
        .nodes()
        .iter()
        .filter_map(|node| match node {
            Behavior::Bind(bind) if bind.name.starts_with("__anon_lambda_") => Some(bind),
            _ => None,
        })
        .collect();
    assert_eq!(lambda_binds.len(), 1, "expected one lambda bind");
    let bind = lambda_binds[0];
    let lambda_param_ports = &bind.params[bind.params.len() - 2..];
    for port in lambda_param_ports {
        match dag.port(*port).state() {
            PortState::Resolved(ty) => assert_eq!(
                ty.declaration, int_id,
                "fold lambda parameter should resolve to Int"
            ),
            other => panic!("fold lambda parameter did not resolve, got {other:?}"),
        }
    }

    assert_eq!(bind_value_type_decl(&dag, "total"), int_id);
}

#[test]
fn prereq0_5_map_generic_call_synthesizes_implicit_template_bindings() {
    let src = "\
let xs = map(singleton(1), |x| x + 1)
";
    let dag = compile_any(src, "prereq0_5_map.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "implicit generic map call should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let int_id = find_named(&dag, "Int");
    let map_id = find_named(&dag, "map");
    let map_decl = dag.declaration(map_id);
    let map_a = map_decl.type_params[0];
    let map_b = map_decl.type_params[1];
    let map_instantiations = callable_instantiation_arguments(&dag, map_id);
    assert!(
        map_instantiations.iter().any(|arguments| {
            arguments
                .iter()
                .any(|arg| arg.parameter == map_a && arg.value == int_id)
                && arguments
                    .iter()
                    .any(|arg| arg.parameter == map_b && arg.value == int_id)
        }),
        "expected map(singleton(1), |x| x + 1) to infer A := Int and B := Int"
    );

    let lambda_binds: Vec<_> = dag
        .nodes()
        .iter()
        .filter_map(|node| match node {
            Behavior::Bind(bind) if bind.name.starts_with("__anon_lambda_") => Some(bind),
            _ => None,
        })
        .collect();
    assert_eq!(lambda_binds.len(), 1, "expected one lambda bind");
    let bind = lambda_binds[0];
    let lambda_param_port = *bind
        .params
        .last()
        .expect("map lambda should expose one declared parameter");
    match dag.port(lambda_param_port).state() {
        PortState::Resolved(ty) => assert_eq!(
            ty.declaration, int_id,
            "map lambda parameter should resolve to Int"
        ),
        other => panic!("map lambda parameter did not resolve, got {other:?}"),
    }
}

#[test]
fn prereq0_5_conflicting_implicit_template_bindings_fail_closed() {
    let src = "\
fn keep<T>(a: T, b: T) -> T = a
let bad = keep(1, true)
";
    let dag = compile_any(src, "prereq0_5_conflict.v3");
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| {
            matches!(
                diag,
                Diagnostic::ResolveError { name, .. }
                    if name.contains("implicit template binding")
            )
        }),
        "expected implicit generic conflict diagnostic, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn prereq3_multi_param_lambda_parses_and_compiles() {
    let src = "\
let f: fn(Int, Int) -> Int = |x, y| x + y
let y = f(2, 3)
";
    let dag = compile_any(src, "prereq3_multi_param_lambda.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "multi-parameter lambda should compile cleanly: {:?}",
        dag.diagnostics()
    );
    assert_eq!(bind_value_type_decl(&dag, "y"), find_named(&dag, "Int"));
}

#[test]
fn prereq3_captured_lambda_direct_call_bakes_capture_into_bind() {
    let src = "\
let base: Int = 1
let f: fn(Int) -> Int = |x| base + x
let y = f(3)
";
    let dag = compile_any(src, "prereq3_captured_lambda.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "captured lambda call should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let int_id = find_named(&dag, "Int");
    let mut saw_lambda_call = false;
    for node in dag.nodes() {
        let Behavior::Transform(transform) = node else {
            continue;
        };
        let TransformTarget::Callable(target) = transform.target else {
            continue;
        };
        let TypeConnective::Arrow { body, .. } = &dag.declaration(target).connective else {
            continue;
        };
        let ArrowBody::UserDefined(bind_id) = body else {
            continue;
        };
        let bind = (*bind_id).bind(&dag);
        if bind.name.starts_with("__anon_lambda_") {
            saw_lambda_call = true;
            assert_eq!(
                transform.inputs.len(),
                1,
                "captured lambda call should still pass only the declared runtime argument"
            );
            assert_eq!(
                bind.params.len(),
                2,
                "captured lambda bind should expose [capture + declared] inputs structurally"
            );
        }
    }

    assert!(
        saw_lambda_call,
        "did not observe the captured lambda call target"
    );
    assert_eq!(bind_value_type_decl(&dag, "y"), int_id);
}

#[test]
fn prereq3_lambda_argument_to_higher_order_call_uses_expected_signature() {
    let src = "\
fn apply_to_three(f: fn(Int) -> Int) -> Int = f(3)
let base: Int = 1
let y = apply_to_three(|x| base + x)
";
    let dag = compile_any(src, "prereq3_lambda_hof.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "lambda argument to higher-order call should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let int_id = find_named(&dag, "Int");
    let apply_id = find_named(&dag, "apply_to_three");
    let apply_f = match &dag.declaration(apply_id).connective {
        TypeConnective::Arrow { inputs, .. } => inputs[0],
        other => panic!("apply_to_three should be Arrow, got {other:?}"),
    };
    let mut saw_instantiation = false;
    for node in dag.nodes() {
        let Behavior::Transform(transform) = node else {
            continue;
        };
        let TransformTarget::Callable(target) = transform.target else {
            continue;
        };
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &dag.declaration(target).connective
        else {
            continue;
        };
        if *template != apply_id {
            continue;
        }
        saw_instantiation = true;
        assert_eq!(
            transform.inputs.len(),
            0,
            "function-typed lambda argument should bind through Instantiation, not runtime ports"
        );
        let lambda_arg = arguments
            .iter()
            .find(|arg| arg.parameter == apply_f)
            .unwrap_or_else(|| panic!("expected function-slot binding, got {arguments:?}"));
        assert!(
            matches!(
                dag.declaration(lambda_arg.value).connective,
                TypeConnective::Arrow {
                    body: ArrowBody::UserDefined(_),
                    ..
                }
            ),
            "expected higher-order lambda argument to lower to a synthetic callable declaration"
        );
    }

    assert!(
        saw_instantiation,
        "did not observe the higher-order lambda instantiation"
    );
    assert_eq!(bind_value_type_decl(&dag, "y"), int_id);
}

#[test]
fn prereq3_lambda_can_capture_outer_callable_parameter() {
    let src = "\
fn apply_to_three(f: fn(Int) -> Int) -> Int = f(3)
fn use_callback(f: fn(Int) -> Int) -> Int = apply_to_three(|x| f(x))
let y = use_callback(|z| z + 1)
";
    let dag = compile_any(src, "prereq3_capture_outer_callable.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "lambda should preserve captured outer callable bindings: {:?}",
        dag.diagnostics()
    );
    assert_eq!(bind_value_type_decl(&dag, "y"), find_named(&dag, "Int"));
}

#[test]
fn infer_retries_higher_order_callable_binding_until_lambda_signature_resolves() {
    let src = "\
fn apply_to_three(f: fn(Int) -> Int) -> Int = f(3)
fn use_callback(f: fn(Int) -> Int) -> Int = apply_to_three(f)
let y = use_callback(|z| z + 1)
";
    let dag = compile_any(src, "callable_retry_lambda_signature.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "higher-order callable binding should retry until the lambda signature resolves: {:?}",
        dag.diagnostics()
    );
    assert_eq!(bind_value_type_decl(&dag, "y"), find_named(&dag, "Int"));
}

#[test]
fn prereq4_list_dag_bootstrap_loads_cleanly() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load staged std.list declarations cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    for name in [
        "List",
        "empty",
        "singleton",
        "cons",
        "fold",
        "map",
        "filter",
    ] {
        assert!(
            dag.declaration_by_name(name).is_some(),
            "bootstrap should register staged std.list declaration `{name}`"
        );
    }
    let list = dag
        .declaration_by_name("List")
        .expect("bootstrap should register List");
    let TypeConnective::Disj { variants } = &list.connective else {
        panic!(
            "staged std.list should shadow the v2 alias with a structural Disj, got {:?}",
            list.connective
        );
    };
    let labels: Vec<_> = variants.iter().map(|field| field.label.as_str()).collect();
    assert_eq!(labels, vec!["Empty", "Cons"]);
}

#[test]
fn prereq4_record_literal_in_expression_position_compiles_with_expected_type() {
    let src = "\
type Point { x: Int y: Int }
fn x_of(p: Point) -> Int = p.x
let total: Int = x_of({ x: 1, y: 2 })
";
    let dag = cached_compile_to_dag(src, "expr_record_literal.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "record literal in expression position should compile when an expected type is available, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn record_literal_duplicate_fields_fail_closed_at_repeated_field() {
    let src = "\
type Pair { a: Int b: Int }
fn first(p: Pair) -> Int = p.a
let duplicate: Int = first({ a: 1, a: 2, b: 3 })
";
    let dag = compile_any(src, "expr_record_literal_duplicate_fields.v3");
    let duplicate_span = u32::try_from(src.find("a: 2").expect("fixture includes repeated field"))
        .expect("fixture span fits in SourceSpan");

    assert!(
        dag.diagnostics().iter().any(|(_, diagnostic)| {
            matches!(
                diagnostic,
                Diagnostic::ResolveError { name, span, .. }
                    if name.contains("record literal repeats field `a`")
                        && span.file == "expr_record_literal_duplicate_fields.v3"
                        && span.byte_start <= duplicate_span
                        && duplicate_span < span.byte_end
            )
        }),
        "expected duplicate record literal diagnostic anchored to second `a`, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn prereq4_list_literal_in_expression_position_lowers_through_std_list_constructors() {
    let dag = cached_compile_to_dag("let xs: List<Int> = [1, 2, 3]", "expr_list_literal.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "got diagnostics: {:?}",
        dag.diagnostics()
    );
    let callable_names: Vec<String> = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_transform)
        .filter_map(|transform| match transform.target {
            TransformTarget::Callable(target) => Some(target),
            _ => None,
        })
        .map(|target| match &dag.declaration(target).connective {
            TypeConnective::Instantiation { template, .. } => *template,
            _ => target,
        })
        .filter_map(|target| dag.declaration(target).name.clone())
        .collect();
    assert!(
        callable_names.iter().any(|name| name == "singleton"),
        "list literal should lower through `singleton`, got {callable_names:?}"
    );
    assert!(
        callable_names.iter().filter(|name| *name == "cons").count() >= 2,
        "list literal should lower through repeated `cons`, got {callable_names:?}"
    );
}

#[test]
fn top_level_list_data_body_lowers_to_value_body_list() {
    let dag = cached_compile_to_dag("data xs: List<Int> = [1, 2, 3]", "data_list_body.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "top-level List<Int> data body should lower structurally, got {:?}",
        dag.diagnostics()
    );
    let decl = dag
        .declaration_by_name("xs")
        .expect("data declaration should exist");
    let Some(v3_compiler::dag::ValueBody::List(elements)) = &decl.value_body else {
        panic!(
            "data xs should lower to ValueBody::List, got {:?}",
            decl.value_body
        );
    };
    let values: Vec<i64> = elements
        .iter()
        .map(|element| match element {
            v3_compiler::dag::FieldValue::Literal(v3_compiler::dag::LiteralBits::Int(value)) => {
                value
                    .parse::<i64>()
                    .expect("fixture list element Int decimal")
            }
            other => panic!("expected literal int list element, got {other:?}"),
        })
        .collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn top_level_list_data_body_requires_list_declared_type() {
    let dag = compile_any(
        "data xs: Int = [1, 2, 3]",
        "data_list_body_non_list_type.v3",
    );
    let expected = "data `xs` has a list body but its declared type is not a List<_>";
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            v3_compiler::diagnostics::Diagnostic::ResolveError { name, .. }
                if name == expected
        )),
        "expected fail-closed non-List diagnostic, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn prereq0_conflicting_callable_template_bindings_fail_closed() {
    let src = "\
fn step_int(x: Int) -> Int = x
fn step_bool(x: Bool) -> Bool = x
fn use<T>(x: T, f: fn(T) -> T, g: fn(T) -> T) -> T = g(f(x))
let bad = use(1, step_int, step_bool)
";
    let dag = compile_any(src, "prereq0_conflicting_callable_bindings.v3");
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            v3_compiler::diagnostics::Diagnostic::ResolveError { name, .. }
                if name.contains("conflicts with earlier template bindings")
        )),
        "expected fail-closed callable-binding diagnostic, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn prereq3_unannotated_lambda_fails_closed() {
    let src = "\
let f = |x| x
";
    let dag = compile_any(src, "prereq3_unannotated_lambda.v3");
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            v3_compiler::diagnostics::Diagnostic::ResolveError { name, .. }
                if name.contains("lambda expressions currently require an expected function type")
        )),
        "expected fail-closed diagnostic for unannotated lambda, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn prereq1_field_access_lowers_to_field_project() {
    // Prereq 1: a local dotted path lowers to a TransformTarget::FieldProject,
    // not a synthesized accessor declaration. The input port is the authority
    // for the parent type; the target only needs the projected field label.
    let src = "\
type Point { x: Int y: Int }
fn get_x(point: Point) -> Int = point.x
";
    let dag = cached_compile_to_dag(src, "field_access.v3");
    let int_id = find_named(&dag, "Int");

    let bind = bind_named(&dag, "get_x");
    let body_node_id = dag
        .port(bind.value)
        .produced_by
        .expect("Bind value has a producer");
    let projection = match dag.node(body_node_id) {
        Behavior::Transform(t) => t,
        other => panic!("expected Transform field projection, got {other:?}"),
    };
    assert_eq!(projection.inputs, vec![bind.params[0]]);
    match &projection.target {
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            assert_eq!(field_label, "x");
            assert_eq!(*field_child, Some(int_id));
        }
        other => panic!("expected FieldProject target, got {other:?}"),
    }
    match dag.port(bind.value).state() {
        v3_compiler::dag::PortState::Resolved(ty) => {
            assert_eq!(*ty, dag.int_shape().expect("Int cached"))
        }
        other => panic!("field projection output should resolve to Int, got {other:?}"),
    }
}

#[test]
fn prereq1_multi_hop_field_access_lowers_to_chained_field_projects() {
    let src = "\
type Inner { x: Int }
type Outer { inner: Inner }
fn get_nested_x(outer: Outer) -> Int = outer.inner.x
";
    let dag = cached_compile_to_dag(src, "field_access_multi_hop.v3");

    let bind = bind_named(&dag, "get_nested_x");

    let final_projection = match dag.node(
        dag.port(bind.value)
            .produced_by
            .expect("Bind value has a producer"),
    ) {
        Behavior::Transform(t) => t,
        other => panic!("expected final Transform field projection, got {other:?}"),
    };
    match &final_projection.target {
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            assert_eq!(field_label, "x");
            assert_eq!(*field_child, Some(find_named(&dag, "Int")));
        }
        other => panic!("expected final FieldProject target, got {other:?}"),
    }

    let intermediate_projection = match dag.node(
        dag.port(final_projection.inputs[0])
            .produced_by
            .expect("final projection input has a producer"),
    ) {
        Behavior::Transform(t) => t,
        other => panic!("expected intermediate Transform field projection, got {other:?}"),
    };
    assert_eq!(intermediate_projection.inputs, vec![bind.params[0]]);
    assert_eq!(intermediate_projection.output, final_projection.inputs[0]);
    match &intermediate_projection.target {
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            assert_eq!(field_label, "inner");
            assert_eq!(*field_child, Some(find_named(&dag, "Inner")));
        }
        other => panic!("expected intermediate FieldProject target, got {other:?}"),
    }
}

#[test]
fn prereq1_field_access_on_instantiated_record_substitutes_template_args() {
    let src = "\
type Box<T> { value: T }
fn read(boxed: Box<Int>) -> Int = boxed.value
";
    let dag = cached_compile_to_dag(src, "field_access_generic.v3");
    let box_id = find_named(&dag, "Box");
    let box_t = dag.declaration(box_id).type_params[0];

    let bind = bind_named(&dag, "read");
    let projection = match dag.node(
        dag.port(bind.value)
            .produced_by
            .expect("Bind value has a producer"),
    ) {
        Behavior::Transform(t) => t,
        other => panic!("expected Transform field projection, got {other:?}"),
    };
    match &projection.target {
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            assert_eq!(field_label, "value");
            assert_eq!(*field_child, Some(box_t));
        }
        other => panic!("expected FieldProject target, got {other:?}"),
    }
    match dag.port(bind.value).state() {
        v3_compiler::dag::PortState::Resolved(ty) => {
            assert_eq!(*ty, dag.int_shape().expect("Int cached"))
        }
        other => panic!("generic field projection output should resolve to Int, got {other:?}"),
    }
}

#[test]
fn prereq1_field_access_on_non_conj_type_is_rejected() {
    let dag = compile_any("fn bad(flag: Bool) -> Int = flag.x", "bad_field_access.v3");
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            Diagnostic::ResolveError { name, .. }
                if name.contains("field `x`") && name.contains("Conj")
        )),
        "expected a non-Conj field-access diagnostic naming `x`, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn prereq1_nonexistent_field_is_rejected_with_field_name() {
    let src = "\
type Point { x: Int }
fn bad(point: Point) -> Int = point.y
";
    let dag = compile_any(src, "missing_field.v3");
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            Diagnostic::ResolveError { name, .. }
                if name.contains("field `y` does not exist")
        )),
        "expected a missing-field diagnostic naming `y`, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn prereq2_payload_binding_compiles_and_types_the_payload_port() {
    // Covers both the basic payload-capture case and the mixed bare +
    // with-payload arm shape in one source program.
    let src = "\
type BoxedInt = Boxed(Int) | Empty
fn unwrap_or_zero(b: BoxedInt) -> Int = match b { Boxed(value) => value, Empty => 0 }
";
    let dag = cached_compile_to_dag(src, "payload_binding.v3");

    let bind = bind_named(&dag, "unwrap_or_zero");
    let branch = match dag.node(
        dag.port(bind.value)
            .produced_by
            .expect("Bind value has a producer"),
    ) {
        Behavior::Branch(b) => b,
        other => panic!("expected Branch at match root, got {other:?}"),
    };
    let payload_path = branch
        .paths
        .iter()
        .find(|path| path.binding.is_some())
        .expect("payload-capturing path exists");
    assert!(
        branch.paths.iter().any(|path| path.binding.is_none()),
        "mixed bare + with-payload arms should preserve `binding: None` on the bare arm"
    );
    match &payload_path.pattern {
        BranchPattern::ResolvedVariant(_) => {}
        other => panic!("expected resolved match pattern, got {other:?}"),
    }
    let binding = payload_path
        .binding
        .as_ref()
        .expect("binding payload stored on Path");
    assert_eq!(binding.binding_name, "value");
    match dag.port(binding.payload_port).state() {
        v3_compiler::dag::PortState::Resolved(ty) => {
            assert_eq!(*ty, dag.int_shape().expect("Int cached"))
        }
        other => panic!("payload port should resolve to Int, got {other:?}"),
    }
}

#[test]
fn prereq2_payload_binding_integrates_with_field_access() {
    let src = "\
type Point { x: Int y: Int }
type MaybePoint = Some(Point) | None
fn id(m: MaybePoint) -> MaybePoint = m
fn get_or_zero(m: MaybePoint) -> Int = match id(m) { Some(point) => point.x, None => 0 }
";
    let dag = cached_compile_to_dag(src, "payload_field_access.v3");

    let bind = bind_named(&dag, "get_or_zero");
    let branch = match dag.node(
        dag.port(bind.value)
            .produced_by
            .expect("Bind value has a producer"),
    ) {
        Behavior::Branch(b) => b,
        other => panic!("expected Branch at match root, got {other:?}"),
    };
    let payload_path = branch
        .paths
        .iter()
        .find(|path| path.binding.is_some())
        .expect("payload-capturing path exists");
    let binding = payload_path
        .binding
        .as_ref()
        .expect("binding payload stored on Path");
    let body_node_id = dag
        .port(payload_path.output)
        .produced_by
        .expect("payload arm body should be the field projection");
    let projection = match dag.node(body_node_id) {
        Behavior::Transform(t) => t,
        other => panic!("expected Transform field projection, got {other:?}"),
    };
    assert_eq!(projection.inputs, vec![binding.payload_port]);
    match &projection.target {
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            assert_eq!(field_label, "x");
            assert_eq!(*field_child, Some(find_named(&dag, "Int")));
        }
        other => panic!("expected FieldProject target, got {other:?}"),
    }
}

#[test]
fn prereq2_payload_binding_on_inferred_scrutinee_compiles() {
    let src = "\
type BoxedInt = Boxed(Int) | Empty
fn id(b: BoxedInt) -> BoxedInt = b
fn unwrap_or_zero(b: BoxedInt) -> Int = match id(b) { Boxed(value) => value, Empty => 0 }
";
    let dag = cached_compile_to_dag(src, "payload_binding_inferred.v3");

    let bind = bind_named(&dag, "unwrap_or_zero");
    let branch = match dag.node(
        dag.port(bind.value)
            .produced_by
            .expect("Bind value has a producer"),
    ) {
        Behavior::Branch(b) => b,
        other => panic!("expected Branch at match root, got {other:?}"),
    };
    let payload_path = branch
        .paths
        .iter()
        .find(|path| path.binding.is_some())
        .expect("payload-capturing path exists");
    let binding = payload_path
        .binding
        .as_ref()
        .expect("binding payload stored on Path");
    match dag.port(binding.payload_port).state() {
        v3_compiler::dag::PortState::Resolved(ty) => {
            assert_eq!(*ty, dag.int_shape().expect("Int cached"))
        }
        other => panic!("payload port for inferred scrutinee should resolve to Int, got {other:?}"),
    }
}

#[test]
fn prereq2_payload_binding_on_instantiated_sum_substitutes_template_args() {
    let src = "\
type Maybe<T> = Some(T) | None
fn unwrap_or_zero(m: Maybe<Int>) -> Int = match m { Some(value) => value, None => 0 }
";
    let dag = cached_compile_to_dag(src, "payload_binding_generic.v3");

    let bind = bind_named(&dag, "unwrap_or_zero");
    let branch = match dag.node(
        dag.port(bind.value)
            .produced_by
            .expect("Bind value has a producer"),
    ) {
        Behavior::Branch(b) => b,
        other => panic!("expected Branch at match root, got {other:?}"),
    };
    let payload_path = branch
        .paths
        .iter()
        .find(|path| path.binding.is_some())
        .expect("payload-capturing path exists");
    let binding = payload_path
        .binding
        .as_ref()
        .expect("binding payload stored on Path");
    match dag.port(binding.payload_port).state() {
        v3_compiler::dag::PortState::Resolved(ty) => {
            assert_eq!(*ty, dag.int_shape().expect("Int cached"))
        }
        other => panic!("payload port for instantiated sum should resolve to Int, got {other:?}"),
    }
}

#[test]
fn reflected_optional_handle_field_projection_resolves() {
    let src = "\
import std.substrate { DagPort, NodeId }
fn producer_or_self(port: DagPort) -> NodeId = match port.produced_by { Some(node_id) => node_id, None => port.id }
";
    let dag = cached_compile_to_dag(src, "optional_handle_field_projection.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "optional reflected handle field projection should compile cleanly, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn prereq2_payload_binding_on_non_disj_scrutinee_fails_closed() {
    let dag = compile_any(
        "fn bad(i: Int) -> Int = match i { Nope(v) => v }",
        "payload_non_disj.v3",
    );
    assert!(
        !dag.diagnostics().is_empty(),
        "payload binding on a non-Disj scrutinee must fail closed"
    );
}

#[test]
fn prereq2_payload_binding_can_bind_record_payloads() {
    let src = "\
type Point { x: Int }
type Wrapped = Wrap { inner: Point } | Empty
fn unwrap_or_zero(w: Wrapped) -> Int = match w { Wrap(payload) => payload.inner.x, Empty => 0 }
";
    let dag = cached_compile_to_dag(src, "payload_record_variant.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "record payload binding should compile cleanly: {:?}",
        dag.diagnostics()
    );
    assert_eq!(
        bind_value_type_decl(&dag, "unwrap_or_zero"),
        find_named(&dag, "Int")
    );
}

#[test]
fn prereq2_named_variant_constructor_expression_compiles_against_expected_sum() {
    let src = "\
type Point { x: Int }
type Wrapped = Wrap { inner: Point } | Empty
fn wrap(point: Point) -> Wrapped = Wrap { inner: point }
";
    let dag = cached_compile_to_dag(src, "named_variant_constructor.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "named variant constructor expression should compile cleanly: {:?}",
        dag.diagnostics()
    );
    assert_eq!(
        bind_value_type_decl(&dag, "wrap"),
        find_named(&dag, "Wrapped")
    );
}

/// Prereq-2 (lens-fold): brace-bodied `fn` lowers a `match` whose arms
/// mix bare and record-shaped variant constructors against a sum whose
/// variant carries a Conj payload (`Cell { n: Int }`), matching the
/// class-5 payload path toward `Witness`-style constructors — without
/// `Witness` / `OptionalDiagnostic` special cases.
#[test]
fn prereq2_brace_fn_match_returns_bare_variant_constructors() {
    let src = "\
type Slot = Cell { n: Int } | Vacant
fn toggle(s: Slot) -> Slot {
  match s {
    Cell { n: k } => Vacant
    Vacant => Cell { n: 1 }
  }
}
";
    let dag = cached_compile_to_dag(src, "prereq2_brace_fn_variants.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "brace-bodied fn with bare variant match arms should compile cleanly: {:?}",
        dag.diagnostics()
    );
    assert_eq!(
        bind_value_type_decl(&dag, "toggle"),
        find_named(&dag, "Slot")
    );
}

#[test]
fn prereq2_brace_fn_optional_diagnostic_bare_variant() {
    let src = "\
import v3.std.dimensions { OptionalDiagnostic }
fn no_diag() -> OptionalDiagnostic {
  NoDiagnostic
}
";
    let dag = cached_compile_to_dag(src, "prereq2_optional_diag_brace_fn.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "OptionalDiagnostic brace-bodied bare variant should compile cleanly: {:?}",
        dag.diagnostics()
    );
    let out = bind_value_type_decl(&dag, "no_diag");
    let opt = find_named(&dag, "OptionalDiagnostic");
    assert_eq!(
        out, opt,
        "OptionalDiagnostic fn should resolve to the sum declaration"
    );
}

#[test]
fn class5_imported_expected_sum_positional_constructor_in_brace_body() {
    let src = "\
import v3.std.dimensions { Witness }
fn ok() -> Witness<Int> {
  Inhabits(1)
}
";
    let dag = cached_compile_to_dag(src, "class5_witness_inhabits_brace_fn.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "imported expected-type variant constructor should compile cleanly: {:?}",
        dag.diagnostics()
    );
    let out = bind_value_type_decl(&dag, "ok");
    let witness = find_named(&dag, "Witness");
    match &dag.declaration(out).connective {
        TypeConnective::Instantiation { template, .. } => assert_eq!(*template, witness),
        other => panic!("expected Witness<Int> instantiation, got {other:?}"),
    }
}

#[test]
fn class5_expected_sum_constructor_with_resolved_arg_does_not_nest_target_instantiation() {
    let src = "\
import v3.std.dimensions { Witness }
fn ok(x: Int) -> Witness<Int> {
  Inhabits(x)
}
";
    let dag = cached_compile_to_dag(src, "class5_witness_inhabits_resolved_arg.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "expected-type constructor with resolved argument should compile cleanly: {:?}",
        dag.diagnostics()
    );
    let out = bind_value_type_decl(&dag, "ok");
    let witness = find_named(&dag, "Witness");
    match &dag.declaration(out).connective {
        TypeConnective::Instantiation { template, .. } => assert_eq!(*template, witness),
        other => panic!("expected Witness<Int> instantiation, got {other:?}"),
    }

    let bind = bind_named(&dag, "ok");
    let transform = match dag.node(
        dag.port(bind.value)
            .produced_by
            .expect("Bind value has a producer"),
    ) {
        Behavior::Transform(t) => t,
        other => panic!("expected constructor Transform at function body, got {other:?}"),
    };
    let TransformTarget::Callable(target) = transform.target else {
        panic!("expected callable constructor target");
    };
    if let TypeConnective::Instantiation { template, .. } = &dag.declaration(target).connective {
        assert!(
            !matches!(
                dag.declaration(*template).connective,
                TypeConnective::Instantiation { .. }
            ),
            "constructor target must not nest an instantiated variant target"
        );
    }
}

#[test]
fn class5_expected_generic_sum_record_constructor_in_brace_body() {
    let src = "\
type Boxed<T> = Packed { value: T } | Empty
fn packed() -> Boxed<Int> {
  Packed { value: 1 }
}
";
    let dag = cached_compile_to_dag(src, "class5_generic_record_constructor_brace_fn.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "expected-type generic record constructor should compile cleanly: {:?}",
        dag.diagnostics()
    );
    let out = bind_value_type_decl(&dag, "packed");
    let boxed = find_named(&dag, "Boxed");
    match &dag.declaration(out).connective {
        TypeConnective::Instantiation { template, .. } => assert_eq!(*template, boxed),
        other => panic!("expected Boxed<Int> instantiation, got {other:?}"),
    }
}

#[test]
fn prereq2_named_payload_pattern_binds_field_projection_ports() {
    let src = "\
type Point { x: Int }
type Wrapped = Wrap { inner: Point } | Empty
fn unwrap_or_zero(w: Wrapped) -> Int = match w { Wrap { inner: point } => point.x, Empty => 0 }
";
    let dag = cached_compile_to_dag(src, "named_payload_pattern.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "named payload pattern should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let bind = bind_named(&dag, "unwrap_or_zero");
    let branch = match dag.node(
        dag.port(bind.value)
            .produced_by
            .expect("Bind value has a producer"),
    ) {
        Behavior::Branch(b) => b,
        other => panic!("expected Branch at match root, got {other:?}"),
    };
    let payload_path = branch
        .paths
        .iter()
        .find(|path| path.binding.is_some())
        .expect("payload-capturing path exists");
    let binding = payload_path
        .binding
        .as_ref()
        .expect("binding payload stored on Path");
    let body_projection = match dag.node(
        dag.port(payload_path.output)
            .produced_by
            .expect("payload arm body should be a field projection"),
    ) {
        Behavior::Transform(t) => t,
        other => panic!("expected Transform field projection, got {other:?}"),
    };
    let projected_point_port = body_projection.inputs[0];
    let point_projection = match dag.node(
        dag.port(projected_point_port)
            .produced_by
            .expect("named binding should be backed by a field projection"),
    ) {
        Behavior::Transform(t) => t,
        other => panic!("expected inner field projection, got {other:?}"),
    };
    assert_eq!(point_projection.inputs, vec![binding.payload_port]);
    match &point_projection.target {
        TransformTarget::FieldProject {
            field_label,
            field_child,
        } => {
            assert_eq!(field_label, "inner");
            assert_eq!(*field_child, Some(find_named(&dag, "Point")));
        }
        other => panic!("expected inner FieldProject target, got {other:?}"),
    }
}

#[test]
fn prereq2_named_payload_pattern_duplicate_binding_fails_closed() {
    let src = "\
type Pair { a: Int b: Int }
type Wrapped = Wrap { inner: Pair } | Empty
fn bad(w: Wrapped) -> Int = match w { Wrap { inner: pair } => match pair { Pair { a: x, b: x } => x }, Empty => 0 }
";
    let dag = compile_any(src, "named_payload_pattern_duplicate_binding.v3");
    assert!(
        dag.diagnostics().iter().any(|(_, diag)| matches!(
            diag,
            Diagnostic::ResolveError { name, .. }
                if name.contains("binds `x` more than once")
        )),
        "expected fail-closed duplicate payload-pattern binding diagnostic, got {:?}",
        dag.diagnostics()
    );
}

/// Regression for #565 / codex blocker: `VariantFields` duplicate-binding
/// detection incorrectly used the full outer scope, so a legitimate
/// shadow (fn parameter `x` + match-arm `Foo { x }` renaming a field
/// to `x`) got rejected as a duplicate. Match arms must be allowed to
/// shadow outer names exactly like `VariantWith { binding: x }` already does.
#[test]
fn prereq2_named_payload_pattern_shadows_outer_scope_binding() {
    // `inner` is in outer scope (fn parameter). The match arm's
    // `Wrap { inner: pair }` binds a new `pair` — not a shadow. But the
    // bug fires whenever an outer name collides with ANY pattern binding,
    // so introduce the conflict by also binding the field to `inner`:
    // `Wrap { inner: inner }` is the shadow case the bug prevented.
    let src = "\
type Pair { inner: Int }
type Wrapped = Wrap { inner: Pair } | Empty
fn read(w: Wrapped, inner: Int) -> Int = match w { Wrap { inner: inner } => inner.inner, Empty => inner }
";
    let dag = cached_compile_to_dag(src, "named_payload_pattern_shadowing.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "named-payload match arm must shadow the outer `inner` binding, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn recursion_accepts_structural_descent_on_recursive_payload_field() {
    let src = "\
type IntList = Empty | Cons { head: Int, tail: IntList }
fn count(list: IntList) -> Int = match list { Empty => 0, Cons(payload) => 1 + count(payload.tail) }
";
    let dag = cached_compile_to_dag(src, "structural_descent.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "structural descent on a recursive payload field should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let bind = bind_named(&dag, "count");
    let producer = dag
        .port(bind.value)
        .produced_by
        .expect("recursive function value should have a producer");
    match dag.node(producer) {
        Behavior::Loop(_) => {}
        other => panic!("expected bounded recursion to lower to Loop, got {other:?}"),
    }
}

#[test]
fn mutual_recursion_accepts_structural_descent_on_later_recursive_type() {
    let src = "\
fn even(list: IntList) -> Bool = match list { Empty => true, Cons(payload) => odd(payload.tail) }
fn odd(list: IntList) -> Bool = match list { Empty => false, Cons(payload) => even(payload.tail) }
type IntList = Empty | Cons { head: Int, tail: IntList }
";
    let dag = cached_compile_to_dag(src, "mutual_structural_descent_later_type.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "mutual structural descent through a later recursive type should compile cleanly: {:?}",
        dag.diagnostics()
    );
    assert_eq!(
        dag.clusters().len(),
        1,
        "later type lowering should not block mutual cluster planning"
    );
}

#[test]
fn mutual_recursion_planner_ignores_callable_parameter_shadowing() {
    let src = "\
fn even(n: Int, odd: fn(Int) -> Bool) -> Bool = odd(n)
fn odd(n: Int, even: fn(Int) -> Bool) -> Bool = even(n)
";
    let dag = cached_compile_to_dag(src, "mutual_shadowing_false_positive.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "callable-parameter shadowing should not fabricate a mutual-recursion diagnostic: {:?}",
        dag.diagnostics()
    );
    assert_eq!(
        dag.clusters().len(),
        0,
        "shadowed callable parameters are not top-level mutual recursion"
    );
    // Scope the Loop check to this fixture's source — bootstrap
    // std modules legitimately contribute Loop nodes for their own
    // structural recursion (e.g. `src/v3/std/algebra.dag::drop_zero`).
    assert!(
        dag.nodes()
            .iter()
            .filter_map(Behavior::as_loop)
            .all(|l| l.span.file != "mutual_shadowing_false_positive.v3"),
        "shadowed callable parameters must not introduce synthetic recursion loops"
    );
}

#[test]
fn mutual_recursion_planner_respects_is_first_on_duplicate_fn() {
    // Second `fn a` is a duplicate (fail-closed diagnostic). The mutual-
    // recursion planner must use only first-authority items — same as
    // lowering — so the duplicate cannot replace `a`'s body with a
    // non-recursive spine and destroy the a↔b cluster.
    let src = "\
fn even(n: Int) -> Bool = if n == 0 then true else odd(n - 1)
fn odd(n: Int) -> Bool = if n == 0 then false else even(n - 1)
fn odd(n: Int) -> Bool = false
";
    let dag = compile_any(src, "mutual_duplicate_is_first.v3");
    assert!(
        dag.diagnostics()
            .iter()
            .any(|d| format!("{d:?}").contains("duplicate declaration")),
        "expected duplicate `odd` diagnostic, got {:?}",
        dag.diagnostics()
    );
    assert_eq!(
        dag.clusters().len(),
        1,
        "planner must keep the first `fn odd` body (mutual with `even`); duplicate must not overwrite call graph"
    );
    assert!(
        dag.nodes().iter().filter_map(Behavior::as_loop).count() >= 1,
        "mutual even/odd should still lower through Loop despite duplicate `odd`"
    );
}

#[test]
fn mutual_recursion_checker_uses_shadow_aware_recursive_edges() {
    let src = "\
fn apply_once(f: fn(Int) -> Bool, x: Int) -> Bool = f(x)
fn even(n: Int) -> Bool = if n == 0 then true else odd(n - 1)
fn odd(n: Int) -> Bool = if n == 0 then false else if n == 1 then apply_once(|even| even(n), n) else even(n - 1)
";
    let dag = cached_compile_to_dag(src, "mutual_shadowing_inside_cluster.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "shadowed lambda parameters inside a real cluster must not poison descent checking: {:?}",
        dag.diagnostics()
    );
    assert_eq!(
        dag.clusters().len(),
        1,
        "real recursive edges still form one cluster"
    );
    let odd_bind = bind_named(&dag, "odd");
    let odd_producer = dag
        .port(odd_bind.value)
        .produced_by
        .expect("odd loop exists");
    assert!(
        matches!(dag.node(odd_producer), Behavior::Loop(_)),
        "accepted mutual recursion should still lower odd through Loop"
    );
}

#[test]
fn recursive_generic_sum_can_reference_itself_in_payload_types() {
    let src = "\
type MyList<T> = Empty | Cons { head: T, tail: MyList<T> }
";
    let dag = cached_compile_to_dag(src, "recursive_generic_sum.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "recursive generic sums should lower without self-resolution diagnostics: {:?}",
        dag.diagnostics()
    );

    let list = dag
        .declaration_by_name("MyList")
        .expect("MyList declaration exists");
    let TypeConnective::Disj { variants } = &list.connective else {
        panic!(
            "recursive generic sum should lower to Disj, got {:?}",
            list.connective
        );
    };
    let cons = variants
        .iter()
        .find(|field| field.label == "Cons")
        .expect("Cons variant exists");
    let cons_decl = dag.declaration(cons.ty);
    let TypeConnective::Conj { children } = &cons_decl.connective else {
        panic!(
            "Cons payload should be a Conj, got {:?}",
            cons_decl.connective
        );
    };
    let tail = children
        .iter()
        .find(|field| field.label == "tail")
        .expect("tail field exists");
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = &dag.declaration(tail.ty).connective
    else {
        panic!(
            "tail field should instantiate MyList<T>, got {:?}",
            dag.declaration(tail.ty).connective
        );
    };
    assert_eq!(
        *template, list.id,
        "tail field should point back to the enclosing recursive sum template"
    );
    assert_eq!(
        arguments.len(),
        1,
        "tail field should preserve the recursive type parameter binding"
    );
}

#[test]
fn std_list_supports_structural_match_and_recursive_descent() {
    let src = "\
fn count(list: List<Int>) -> Int = match list { Empty => 0, Cons(payload) => 1 + count(payload.tail) }
let n: Int = count([1, 2, 3])
";
    let dag = cached_compile_to_dag(src, "std_list_structural_recursion.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "std.list structural recursion should compile cleanly: {:?}",
        dag.diagnostics()
    );
    assert_eq!(bind_value_type_decl(&dag, "n"), find_named(&dag, "Int"));

    let bind = bind_named(&dag, "count");
    let producer = dag
        .port(bind.value)
        .produced_by
        .expect("recursive std.list function value should have a producer");
    match dag.node(producer) {
        Behavior::Loop(_) => {}
        other => panic!("expected std.list recursive descent to lower to Loop, got {other:?}"),
    }
}

#[test]
fn substrate_accessors_exist_in_bootstrap_dag() {
    let dag = v3_compiler::Dag::new();
    let port_decl = dag.declaration_by_name("port").expect("port exists");
    let node_decl = dag.declaration_by_name("node").expect("node exists");
    let resolve_decl = dag
        .declaration_by_name("resolve_producer")
        .expect("resolve_producer exists");
    // Each accessor must be an Arrow with the right arity. Post
    // review-round 1b.3, bodies stay `Unparsed` at bootstrap — the
    // per-target realization lookup happens at emission time against
    // the `SubstrateAccessorBinding` records (filtered by
    // `language: <active LanguageSpec>`). See DB-14 and
    // `emit_rust::build_substrate_accessor_index`. E-9-shaped bootstrap
    // rewrite deferred — see **Deferral: E-9 substrate accessor bootstrap
    // rewrite** in `ROADMAP.md`.
    use v3_compiler::dag::{ArrowBody, TypeConnective};
    match (
        &port_decl.connective,
        &node_decl.connective,
        &resolve_decl.connective,
    ) {
        (
            TypeConnective::Arrow {
                inputs: pi,
                body: pb,
                ..
            },
            TypeConnective::Arrow {
                inputs: ni,
                body: nb,
                ..
            },
            TypeConnective::Arrow {
                inputs: ri,
                body: rb,
                ..
            },
        ) => {
            assert_eq!(pi.len(), 2, "port arity");
            assert_eq!(ni.len(), 2, "node arity");
            assert_eq!(ri.len(), 2, "resolve_producer arity");
            // Bodies stay Unparsed at bootstrap. Upgrading them to a
            // specific ExternalRealization would silently drop target
            // selection the moment a second backend registers its own
            // binding — that was the review-round 1b.3 root cause.
            assert!(
                matches!(pb, ArrowBody::Unparsed(_)),
                "port body should stay Unparsed — target selection happens at emission time; got {pb:?}"
            );
            assert!(
                matches!(nb, ArrowBody::Unparsed(_)),
                "node body should stay Unparsed; got {nb:?}"
            );
            assert!(
                matches!(rb, ArrowBody::Unparsed(_)),
                "resolve_producer body should stay Unparsed; got {rb:?}"
            );
        }
        other => panic!("accessor shapes: {other:?}"),
    }
    // No bootstrap diagnostics.
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should be clean: {:?}",
        dag.diagnostics()
    );
}

#[test]
fn substrate_port_call_lowers_without_match() {
    let src = "\
fn test(d: Dag, pid: PortId) -> DagPort? = port(d, pid)
";
    let dag = compile_any(src, "probe_port_no_match.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "port call without match should resolve: {:?}",
        dag.diagnostics()
    );
}

#[test]
fn resolve_producer_opt_walks_through_bind_hops() {
    // DB-5 locks `resolve_producer` as recursive Bind-chain resolution.
    // Verify the Rust-side helper never returns a Bind — a single-hop
    // implementation would, which is the regression this test pins.
    let src = "\
let base: Int = 1 + 2
let alias: Int = base
let double_alias: Int = alias
let total: Int = double_alias + double_alias
";
    let dag = cached_compile_to_dag(src, "bind_hop.v3");
    // Walk every Bind in the Dag. For each, resolve_producer_opt on its
    // value port MUST land on a non-Bind (Value / Transform / etc.) —
    // never a Bind.
    let mut checked = 0;
    for node in dag.nodes() {
        let v3_compiler::dag::Behavior::Bind(bind) = node else {
            continue;
        };
        let resolved = dag.resolve_producer_opt(&bind.value);
        if let Some(behavior) = resolved {
            assert!(
                !matches!(behavior, v3_compiler::dag::Behavior::Bind(_)),
                "resolve_producer_opt returned a Bind for bind `{}` — Bind-hop \
                 traversal missing (DB-5 locks this as recursive)",
                bind.name
            );
            checked += 1;
        }
    }
    assert!(
        checked > 0,
        "no Binds in fixture — test cannot exercise Bind-hop traversal"
    );
}

#[test]
fn substrate_accessor_rust_binding_invariants() {
    // 1b.3: every `SubstrateAccessorBinding` row is Structural and
    // carries `language: Reference(rust_language)` so emitters filter
    // deterministically.
    // 1b.4: the accessor universe has no Rust-exposed hole (every
    // declared accessor has a rust_language binding).
    use std::collections::HashSet;

    use v3_compiler::dag::{FieldValue, ValueBody};

    let dag = v3_compiler::Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should be clean: {:?}",
        dag.diagnostics()
    );
    let binding_meta = dag
        .declaration_by_name("SubstrateAccessorBinding")
        .expect("SubstrateAccessorBinding type exists");
    let rust_language = dag
        .declaration_by_name("rust_language")
        .expect("rust_language exists");
    let mut universe: HashSet<DeclarationId> = HashSet::new();
    let mut rust_covered: HashSet<DeclarationId> = HashSet::new();
    let mut checked = 0;
    for decl in dag.declarations() {
        if decl.meta_tag != Some(binding_meta.id) {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            panic!("binding `{:?}` must have Structural value body", decl.name);
        };
        let language_field = fields.iter().find(|(label, _)| label == "language");
        assert!(
            language_field.is_some(),
            "binding `{:?}` missing required `language` field (review 1b.3)",
            decl.name
        );
        let (_, lang_val) = language_field.unwrap();
        match lang_val {
            FieldValue::Reference(id) => {
                assert_eq!(
                    *id, rust_language.id,
                    "binding `{:?}` should target rust_language",
                    decl.name
                );
            }
            other => panic!(
                "binding `{:?}` language field is not a Reference: {other:?}",
                decl.name
            ),
        }
        let mut accessor = None;
        let mut language = None;
        for (label, value) in fields {
            match (label.as_str(), value) {
                ("accessor", FieldValue::Reference(id)) => accessor = Some(*id),
                ("language", FieldValue::Reference(id)) => language = Some(*id),
                _ => {}
            }
        }
        let (Some(accessor), Some(language)) = (accessor, language) else {
            panic!(
                "binding `{:?}` missing accessor/language references",
                decl.name
            );
        };
        universe.insert(accessor);
        if language == rust_language.id {
            rust_covered.insert(accessor);
        }
        checked += 1;
    }
    assert_eq!(
        checked, 8,
        "expected 8 substrate accessor bindings (port, node, resolve_producer, lane2_workflow_at, declaration_by_id, workflow_root_port, declaration_by_name, per_call_pattern_at)"
    );
    let missing: Vec<_> = universe.difference(&rust_covered).copied().collect();
    assert!(
        missing.is_empty(),
        "substrate accessor universe not fully covered for rust_language — \
         {} accessor(s) without a Rust binding: {:?}. emit_rust would fail \
         closed on any program that calls them.",
        missing.len(),
        missing
    );
    assert!(
        !universe.is_empty(),
        "universe is empty — SubstrateAccessorBinding records missing entirely"
    );
}

#[test]
fn substrate_port_accessor_resolves_from_user_code() {
    let src = "\
import std.substrate { Dag, DagPort, PortId, port }

fn test(d: Dag, pid: PortId) -> Bool =
  match port(d, pid) {
    None => false
    Some(p) => true
  }
";
    let dag = compile_any(src, "probe_port.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "substrate port accessor should resolve from user code: {:?}",
        dag.diagnostics()
    );
}

#[test]
fn substrate_node_accessor_resolves_from_user_code() {
    let src = "\
import std.substrate { Dag, Behavior, NodeId, node }

fn test(d: Dag, nid: NodeId) -> Bool =
  match node(d, nid) {
    None => false
    Some(b) => true
  }
";
    let dag = compile_any(src, "probe_node.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "substrate node accessor should resolve from user code: {:?}",
        dag.diagnostics()
    );
}

#[test]
fn std_list_cons_accepts_user_record_element() {
    let src = "\
type FoundBind { name: String }
let xs: List<FoundBind> = cons({ name: \"x\" }, empty())
";
    let dag = compile_any(src, "std_list_user_record_cons.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "expected cons({{...}}, empty()) over a user record to compile cleanly, got diagnostics: {:?}",
        dag.diagnostics()
    );
}

#[test]
fn monomorphic_recursive_self_call_with_reflected_list_arg_compiles() {
    let src = "\
fn step(n: Int, d: Dag, x: PortId, ys: List<PortId>) -> List<PortId> =
  if n == 0 then ys else step(n - 1, d, x, cons(x, ys))
";
    let dag = compile_any(src, "monomorphic_recursive_reflected_list_arg.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "monomorphic recursive self-call with reflected list args should compile cleanly, got diagnostics: {:?}",
        dag.diagnostics()
    );
}

#[test]
fn monomorphic_recursive_self_call_with_helper_produced_reflected_list_args_compiles() {
    let src = "\
fn expand_frontier_list(d: Dag, frontier: List<PortId>, referenced: List<PortId>) -> List<PortId> = frontier
fn expand_referenced_list(frontier: List<PortId>, referenced: List<PortId>) -> List<PortId> = referenced
fn walk_steps(remaining: Int, d: Dag, frontier: List<PortId>, referenced: List<PortId>) -> List<PortId> =
  if remaining == 0 then
    referenced
  else
    if is_empty(frontier) then
      referenced
    else
      walk_steps(
        remaining - 1,
        d,
        expand_frontier_list(d, frontier, referenced),
        expand_referenced_list(frontier, referenced)
      )
";
    let dag = compile_any(src, "monomorphic_recursive_helper_reflected_list_args.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "monomorphic recursive self-call with helper-produced reflected list args should compile cleanly, got diagnostics: {:?}",
        dag.diagnostics()
    );
}

#[test]
fn referenced_port_walk_real_helper_stack_compiles() {
    let src = "\
fn referenced_ports(d: Dag, root: PortId) -> List<PortId> =
  walk_steps(length(d.ports), d, singleton(root), empty())

fn walk_steps(remaining: Int, d: Dag, frontier: List<PortId>, referenced: List<PortId>) -> List<PortId> =
  if remaining == 0 then
    referenced
  else
    if is_empty(frontier) then
      referenced
    else
      walk_steps(
        remaining - 1,
        d,
        expand_frontier_list(d, frontier, referenced),
        expand_referenced_list(frontier, referenced)
      )

fn expand_frontier_list(d: Dag, frontier: List<PortId>, referenced: List<PortId>) -> List<PortId> =
  fold(frontier, empty(), |next, port|
    if contains(referenced, port) then
      next
    else
      concat(inputs_for_port(d, port), next)
  )

fn expand_referenced_list(frontier: List<PortId>, referenced: List<PortId>) -> List<PortId> =
  fold(frontier, referenced, |acc, port|
    if contains(acc, port) then
      acc
    else
      cons(port, acc)
  )

fn inputs_for_port(d: Dag, port_id: PortId) -> List<PortId> =
  match find_producer(d.nodes, port_id) {
    MissingBehavior => empty()
    FoundBehavior(behavior) => inputs_for_behavior(d, behavior)
  }

type BehaviorLookup
  = MissingBehavior
  | FoundBehavior(Behavior)

type ResultPortLookup
  = MissingResultPort
  | FoundResultPort(PortId)

fn inputs_for_behavior(d: Dag, behavior: Behavior) -> List<PortId> =
  match behavior {
    Value(v) => empty()
    Transform(t) => t.inputs
    Branch(branch) => cons(branch.input, branch_path_outputs(branch.paths))
    Loop(loop_node) => loop_inputs(d, loop_node)
    Bind(bind) => singleton(bind.result_port)
  }

fn loop_inputs(d: Dag, loop_node: LoopNode) -> List<PortId> =
  concat(
    cons(loop_node.source, cons(loop_node.init, loop_bound_inputs(loop_node.bound))),
    match behavior_result_port(d.nodes, loop_node.body) {
      MissingResultPort => empty()
      FoundResultPort(port) => singleton(port)
    }
  )

fn loop_bound_inputs(bound: LoopBound) -> List<PortId> =
  match bound {
    Cardinality(payload) => singleton(payload.count)
    Descent(payload) => singleton(payload.measure)
  }

fn branch_path_outputs(paths: List<BranchPath>) -> List<PortId> =
  match paths {
    Empty => empty()
    Cons(payload) => cons(payload.head.result_port, branch_path_outputs(payload.tail))
  }

fn find_behavior(nodes: List<Behavior>, node_id: NodeId) -> BehaviorLookup =
  match nodes {
    Empty => MissingBehavior
    Cons(payload) =>
      if behavior_id(payload.head) == node_id then
        FoundBehavior(payload.head)
      else
        find_behavior(payload.tail, node_id)
  }

fn find_producer(nodes: List<Behavior>, port_id: PortId) -> BehaviorLookup =
  match nodes {
    Empty => MissingBehavior
    Cons(payload) =>
      if behavior_port(payload.head) == port_id then
        FoundBehavior(payload.head)
      else
        find_producer(payload.tail, port_id)
  }

fn behavior_result_port(nodes: List<Behavior>, node_id: NodeId) -> ResultPortLookup =
  match find_behavior(nodes, node_id) {
    MissingBehavior => MissingResultPort
    FoundBehavior(behavior) => FoundResultPort(behavior_port(behavior))
  }

fn behavior_id(behavior: Behavior) -> NodeId =
  match behavior {
    Value(v) => v.id
    Transform(t) => t.id
    Branch(b) => b.id
    Loop(l) => l.id
    Bind(bind) => bind.id
  }

fn behavior_port(behavior: Behavior) -> PortId =
  match behavior {
    Value(v) => v.result_port
    Transform(t) => t.result_port
    Branch(b) => b.result_port
    Loop(l) => l.result_port
    Bind(bind) => bind.result_port
  }
";
    let dag = compile_any(src, "referenced_port_walk_real_stack.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "referenced-port walk over the real helper stack should compile cleanly, got diagnostics: {:?}",
        dag.diagnostics()
    );
}
