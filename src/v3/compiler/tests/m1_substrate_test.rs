// M1(2.5) substrate tests — oracle from src/v3/M1_DESIGN.md §5 and §6.
//
// Test 1 (`parse_std_algebra_and_walk_int_add`) is the §5 canonical walk:
// starting from the Int declaration produced by bootstrap, follow the
// Instantiation template into OrderedRing, find the `add` field, substitute
// the template argument T := Word64, and assert the resulting Arrow's
// inputs and output both point at the Word64 declaration.
//
// Test 2 (`parse_synthetic_service_all_layers`) is the §6 five-level nested
// Conj shape: SyntheticService / SyntheticOperation meta-types and a
// CmdExec instance whose declaration tree mirrors the nesting.

use std::collections::HashMap;
use v3_compiler::compile_to_dag;
use v3_compiler::dag::{
    ArrowBody, AtomPayload, Behavior, BranchPattern, Dag, DeclarationId, Field, PortState,
    TransformTarget, TypeConnective,
};
use v3_compiler::operators::{ArithmeticOp, ComparisonOp, OperatorKind};
use v3_compiler::{CompileError, Diagnostic};

fn compile_any(src: &str, file: &str) -> Dag {
    match compile_to_dag(src, file) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => dag,
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}

fn find_named(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("declaration `{name}` not found"))
        .id
}

fn field<'a>(fields: &'a [Field], label: &str) -> &'a Field {
    fields
        .iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("field `{label}` not found"))
}

fn bind_value_type_decl(dag: &Dag, name: &str) -> DeclarationId {
    let value_port = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            Behavior::Bind(bind) if bind.name == name => Some(bind.value),
            _ => None,
        })
        .unwrap_or_else(|| panic!("bind `{name}` not found"));
    match dag.port(value_port).state() {
        PortState::Resolved(ty) => ty.declaration,
        other => panic!("bind `{name}` did not resolve, got {other:?}"),
    }
}

fn callable_instantiation_arguments(
    dag: &Dag,
    template: DeclarationId,
) -> Vec<&[v3_compiler::dag::TemplateArgument]> {
    dag.nodes()
        .iter()
        .filter_map(|node| {
            let Behavior::Transform(transform) = node else {
                return None;
            };
            let TransformTarget::Callable(target) = transform.target else {
                return None;
            };
            let TypeConnective::Instantiation {
                template: inst_template,
                arguments,
            } = &dag.declaration(target).connective
            else {
                return None;
            };
            (*inst_template == template).then_some(arguments.as_slice())
        })
        .collect()
}

#[test]
fn parse_std_algebra_and_walk_int_add() {
    // M1_DESIGN.md §5 canonical walk. The real `dsl/std/integer.dag`
    // declares `Int = Int64` and `Int64 = OrderedRing<Word64>`, so Int
    // reaches the OrderedRing algebra through two Instantiation hops
    // rather than one. The test walks that chain accumulating template
    // arguments on a SubstStack-equivalent HashMap, finds the `add`
    // field on OrderedRing, and asserts the substituted Arrow's inputs
    // and output both point at Word64.
    let dag = Dag::new();

    let int_id = find_named(&dag, "Int");
    let word64_id = find_named(&dag, "Word64");
    let ordered_ring_id = find_named(&dag, "OrderedRing");

    // Walk Int's Instantiation chain to the first Conj, accumulating
    // substitutions along the way. `subst` maps TypeParam DeclarationIds
    // to the concrete DeclarationIds they're bound to.
    let mut subst: HashMap<DeclarationId, DeclarationId> = HashMap::new();
    let algebra_id = walk_instantiation_chain(&dag, int_id, &mut subst);
    assert_eq!(
        algebra_id, ordered_ring_id,
        "Int's instantiation chain must root at OrderedRing"
    );

    // Walk OrderedRing's Conj to find the `add` field.
    let ordered_ring_children = match &dag.declaration(ordered_ring_id).connective {
        TypeConnective::Conj { children } => children.clone(),
        other => panic!("expected OrderedRing to be Conj, got {other:?}"),
    };
    let add_field = field(&ordered_ring_children, "add");

    // `add`'s ty is an Arrow [T, T] → T, body: Pending. Substituting
    // T := Word64 yields [Word64, Word64] → Word64.
    let (arrow_inputs, arrow_output, arrow_body) = match &dag.declaration(add_field.ty).connective {
        TypeConnective::Arrow {
            inputs,
            output,
            body,
        } => (inputs.clone(), *output, body.clone()),
        other => panic!("expected add field to be Arrow, got {other:?}"),
    };
    assert!(
        matches!(arrow_body, ArrowBody::Pending),
        "algebra arrow bodies are Pending at M1(2.5) — got {arrow_body:?}"
    );
    assert_eq!(arrow_inputs.len(), 2, "add takes two arguments");

    let substitute = |id: DeclarationId| -> DeclarationId { *subst.get(&id).unwrap_or(&id) };
    let sub_input0 = substitute(arrow_inputs[0]);
    let sub_input1 = substitute(arrow_inputs[1]);
    let sub_output = substitute(arrow_output);

    assert_eq!(sub_input0, word64_id);
    assert_eq!(sub_input1, word64_id);
    assert_eq!(sub_output, word64_id);
}

/// Walk a declaration's Instantiation chain, accumulating
/// `[parameter := value]` bindings into `subst`, until a non-
/// Instantiation declaration (typically a Conj algebra) is reached.
/// Returns the terminal declaration's id.
fn walk_instantiation_chain(
    dag: &Dag,
    start: DeclarationId,
    subst: &mut HashMap<DeclarationId, DeclarationId>,
) -> DeclarationId {
    let mut current = start;
    for _ in 0..16 {
        match &dag.declaration(current).connective {
            TypeConnective::Instantiation {
                template,
                arguments,
            } => {
                for arg in arguments {
                    subst.insert(arg.parameter, arg.value);
                }
                current = *template;
            }
            _ => return current,
        }
    }
    current
}

#[test]
fn parse_synthetic_service_all_layers() {
    // Synthetic nested-domain model. Five levels: SyntheticService meta
    // → operations container → SyntheticOperation Run meta → input /
    // output / arguments → scalar fields. The compiler needs to produce
    // the full Declaration tree with only the six committed connectives.
    let src = "\
type SyntheticService { }
type SyntheticOperation { }
type RunInput { }
type RunOutput { }
type RunArguments { }
type CmdExec_Run {
  input: RunInput
  output: RunOutput
  arguments: RunArguments
}
type CmdExec_Operations {
  Run: CmdExec_Run
}
type CmdExec {
  operations: CmdExec_Operations
}
";
    let dag = compile_any(src, "synthetic.v3");
    assert!(
        dag.diagnostics().is_empty(),
        "synthetic service should compile cleanly: {:?}",
        dag.diagnostics()
    );

    // Level 1: CmdExec is a Conj with one child, operations.
    let cmd_exec_id = find_named(&dag, "CmdExec");
    let cmd_exec_children = match &dag.declaration(cmd_exec_id).connective {
        TypeConnective::Conj { children } => children.clone(),
        other => panic!("expected CmdExec Conj, got {other:?}"),
    };
    let operations_field = field(&cmd_exec_children, "operations");

    // Level 2: operations field points at CmdExec_Operations which is a
    // Conj with a single `Run` child.
    let operations_id = operations_field.ty;
    let ops_children = match &dag.declaration(operations_id).connective {
        TypeConnective::Conj { children } => children.clone(),
        other => panic!("expected CmdExec_Operations Conj, got {other:?}"),
    };
    let run_field = field(&ops_children, "Run");

    // Level 3: Run field points at CmdExec_Run which is a Conj with
    // input, output, and arguments children.
    let run_id = run_field.ty;
    let run_children = match &dag.declaration(run_id).connective {
        TypeConnective::Conj { children } => children.clone(),
        other => panic!("expected CmdExec_Run Conj, got {other:?}"),
    };
    let input_field = field(&run_children, "input");
    let output_field = field(&run_children, "output");
    let arguments_field = field(&run_children, "arguments");

    // Level 4: each field points at a Conj (empty for this test, but the
    // structure is there). Assert the connective shape, not the children.
    for f in [input_field, output_field, arguments_field] {
        let child_connective = &dag.declaration(f.ty).connective;
        assert!(
            matches!(child_connective, TypeConnective::Conj { .. }),
            "expected level-4 Conj for `{}`, got {:?}",
            f.label,
            child_connective
        );
    }

    // Level 5: the synthetic service / operation meta-types exist as
    // bare Conj declarations. They're lookup anchors for inhabits tags
    // in a richer model; at M1(2.5) they're structurally present but
    // empty.
    for meta in ["SyntheticService", "SyntheticOperation"] {
        let meta_id = find_named(&dag, meta);
        assert!(
            matches!(
                &dag.declaration(meta_id).connective,
                TypeConnective::Conj { .. }
            ),
            "meta-type `{meta}` should be a Conj"
        );
    }

    // Spot-check: Magma<T>'s type parameter lives in the canonical
    // `Declaration.type_params` slot, not mixed into Conj.children.
    // Magma.children should contain only the `op` field; T is reachable
    // via dag.declaration(magma_id).type_params.
    let magma_id = find_named(&dag, "Magma");
    let magma_decl = dag.declaration(magma_id);
    let magma_children = match &magma_decl.connective {
        TypeConnective::Conj { children } => children,
        _ => panic!("Magma should be Conj"),
    };
    assert_eq!(
        magma_children.len(),
        1,
        "Magma's Conj children are pure record fields (just `op`), not TypeParams"
    );
    assert_eq!(magma_children[0].label, "op");
    assert_eq!(
        magma_decl.type_params.len(),
        1,
        "Magma has exactly one type parameter (T)"
    );
    let t_id = magma_decl.type_params[0];
    let is_type_param = matches!(
        &dag.declaration(t_id).connective,
        TypeConnective::Atom(AtomPayload::TypeParam(_))
    );
    assert!(
        is_type_param,
        "Magma.type_params[0] should be a TypeParam atom declaration"
    );
}

#[test]
fn child_declarations_are_anonymous() {
    // Named type parameters (`T`), sum variants (`True`, `False`,
    // `Less`, `Equal`, `Greater`), and realization scaffolds
    // (`Int64_add_rust`, the realization Arrow itself) are child
    // declarations of their parents. They are intentionally stored
    // with `name: None` so that `Dag::declaration_by_name` cannot find
    // them — user code that types `T` or `True` as a free identifier
    // gets a ResolveError rather than silently binding to a leaked
    // child declaration from the bootstrap set.
    let dag = Dag::new();
    let leaked_names = [
        "T",              // Monoid/Group/Ring/... type parameter binder
        "True",           // Classical variant
        "False",          // Classical variant
        "Less",           // Ordering variant
        "Equal",          // Ordering variant
        "Greater",        // Ordering variant
        "Int64_add_rust", // §6.5 realization scaffold (pre-anonymization)
        "Int64_add",      // §6.5 realization arrow (pre-anonymization)
    ];
    for name in leaked_names {
        assert!(
            dag.declaration_by_name(name).is_none(),
            "child declaration `{name}` must not be findable via declaration_by_name — it would silently shadow any user-code `{name}` reference"
        );
    }
    // Sanity check: genuine top-level declarations still resolve.
    assert!(dag.declaration_by_name("Int").is_some());
    assert!(dag.declaration_by_name("OrderedRing").is_some());
    assert!(dag.declaration_by_name("Classical").is_some());
    // Starting at M1(3)+reflection, the per-target realization
    // meta-types are production bootstrap declarations from
    // `src/v3/spec/rust.dag`:
    //
    //   - `TypeRealization`              — monomorphic type →
    //                                      target type
    //   - `TypeInstantiationRealization` — generic template →
    //                                      target instantiated carrier
    // are production bootstrap declarations from the first
    //   - `OperatorRealization` — (operand type, algebra field)
    //                             → target operator symbol
    //   - `BehaviorRealization` — substrate marker → target
    //                             template
    //   - `CallableRealization` — callable declaration → render
    //                             strategy
    //   - `PatternRealization`  — structural sum → carrier-
    //                             specific match lowering
    //
    // The v3 emitter reads `data rust_*: <RealizationKind> = {...}`
    // items through `meta_tag` filtering against each meta-type
    // declaration. This is the thesis-aligned end-state: the
    // realization meta-types live in extdeps where every
    // per-target-language fact lives, not in compiler code.
    //
    // The unwind also added marker types from `dsl/std/v3_l1.dag`
    // (Bind, Branch, Loop, Transform, Value, Main) plus the
    // `DeclarationRef` sentinel meta-type that lets target spec files
    // carry typed declaration references in record-literal field
    // values. Each is a Conj (empty body) by construction.
    for meta in [
        "TypeRealization",
        "TypeInstantiationRealization",
        "OperatorRealization",
        "BehaviorRealization",
        "CallableRealization",
        "PatternRealization",
    ] {
        let id = dag
            .declaration_by_name(meta)
            .unwrap_or_else(|| panic!("`{meta}` must be declared by src/v3/spec/rust.dag"))
            .id;
        assert!(
            matches!(dag.declaration(id).connective, TypeConnective::Conj { .. }),
            "`{meta}` must lower to a Conj"
        );
    }
    for marker in [
        "Bind",
        "Branch",
        "Loop",
        "Transform",
        "Value",
        "Main",
        "DeclarationRef",
    ] {
        let id = dag
            .declaration_by_name(marker)
            .unwrap_or_else(|| panic!("`{marker}` must be declared by dsl/std/v3_l1.dag"))
            .id;
        assert!(
            matches!(dag.declaration(id).connective, TypeConnective::Conj { .. }),
            "v3_l1 marker `{marker}` must lower to a Conj"
        );
    }
}

// ════════════════════════════════════════════════════════════════
// M1(2.7) — FACTS FLOW FORWARD + SINGLE AUTHORITY fixes
// ════════════════════════════════════════════════════════════════
//
// These tests verify the structural guarantees the M1(2.7) PR
// introduced in response to ChatGPT's review of PR #445:
//
//   Class 1 — primitive identity cache
//   Class 2 — TransformTarget + OperatorKind structural dispatch
//   Class 3 — scaffold honesty: FnExternalBody, Data, Module, Import
//   Class 4 — Realization DeclarationId cache (tested indirectly
//             via the existing §6.5 smoke test)

#[test]
fn m17_primitive_cache_is_populated_at_bootstrap() {
    // Class 1: `Dag::int_shape` / `bool_shape` / `string_shape`
    // return typed `TypeShape` pointers cached at bootstrap, not
    // `None`. Callers that used to run `primitive_shape(dag, "Int")`
    // name scans per call now read the cache in O(1) with no string
    // comparison.
    let dag = Dag::new();
    let int_shape = dag.int_shape().expect("Int cached at bootstrap");
    let bool_shape = dag.bool_shape().expect("Bool cached at bootstrap");
    let string_shape = dag.string_shape().expect("String cached at bootstrap");

    // The cached shapes must match the declaration-table lookups —
    // the cache is a typed index over the same table, not a
    // parallel authority.
    let int_by_name = dag.declaration_by_name("Int").unwrap().id;
    let bool_by_name = dag.declaration_by_name("Bool").unwrap().id;
    let string_by_name = dag.declaration_by_name("String").unwrap().id;
    assert_eq!(int_shape, v3_compiler::types::TypeShape::new(int_by_name));
    assert_eq!(bool_shape, v3_compiler::types::TypeShape::new(bool_by_name));
    assert_eq!(
        string_shape,
        v3_compiler::types::TypeShape::new(string_by_name)
    );
}

#[test]
fn m2_rust_language_syntax_bundle_is_cached_and_structural() {
    let dag = Dag::new();
    let rust_language = dag
        .rust_language_spec()
        .expect("rust_language syntax bundle should be cached at bootstrap");
    let decl = dag.declaration(rust_language);
    match decl.value_body.as_ref() {
        Some(v3_compiler::dag::ValueBody::Structural { fields }) => {
            let labels: Vec<_> = fields.iter().map(|(label, _)| label.as_str()).collect();
            assert_eq!(
                labels,
                vec![
                    "statements",
                    "expressions",
                    "control_flow",
                    "literals",
                    "modules",
                    "functions",
                    "type_applications",
                    "type_definitions",
                    "patterns",
                    "collection_ops",
                    "values",
                ]
            );
        }
        other => panic!("rust_language must lower structurally, got {other:?}"),
    }

    for syntax_type in [
        "StatementSyntax",
        "ExpressionSyntax",
        "ForEachSyntax",
        "ControlFlowSyntax",
        "LiteralSyntax",
        "FunctionSyntax",
        "TypeApplicationSyntax",
        "TypeDefinitionSyntax",
        "PatternMatchSyntax",
        "CollectionOps",
        "ValueConstructionSyntax",
        "LanguageSpec",
    ] {
        let id = dag
            .declaration_by_name(syntax_type)
            .unwrap_or_else(|| panic!("`{syntax_type}` must be declared in rust.dag"))
            .id;
        assert!(
            matches!(dag.declaration(id).connective, TypeConnective::Conj { .. }),
            "`{syntax_type}` must lower to a Conj"
        );
    }
}

#[test]
fn m2_rust_rendering_bundle_is_cached_and_structural() {
    let dag = Dag::new();
    let rust_rendering = dag
        .rust_rendering_spec()
        .expect("rust_rendering bundle should be cached at bootstrap");
    let decl = dag.declaration(rust_rendering);
    match decl.value_body.as_ref() {
        Some(v3_compiler::dag::ValueBody::Structural { fields }) => {
            let labels: Vec<_> = fields.iter().map(|(label, _)| label.as_str()).collect();
            assert_eq!(labels, vec!["read", "construct"]);
        }
        other => panic!("rust_rendering must lower structurally, got {other:?}"),
    }

    for type_name in ["RenderingModel", "ReadStrategy", "ConstructStrategy"] {
        let id = dag
            .declaration_by_name(type_name)
            .unwrap_or_else(|| panic!("`{type_name}` must be declared in rust.dag"))
            .id;
        match type_name {
            "RenderingModel" => assert!(
                matches!(dag.declaration(id).connective, TypeConnective::Conj { .. }),
                "`RenderingModel` must lower to a Conj"
            ),
            "ReadStrategy" | "ConstructStrategy" => assert!(
                matches!(dag.declaration(id).connective, TypeConnective::Disj { .. }),
                "`{type_name}` must lower to a Disj"
            ),
            _ => unreachable!("covered above"),
        }
    }
}

#[test]
fn m17_operator_lowers_to_structural_transform_target() {
    // Class 2: `let x = 1 + 2` lowers to a Transform whose
    // `target: TransformTarget::Operator(Arithmetic(Add))`. No
    // anonymous stub declaration is allocated for the operator
    // symbol; the dispatch fact lives on the Transform's variant.
    let dag = compile_to_dag("let x = 1 + 2", "test.v3").expect("compiles");
    let add_node = dag
        .nodes()
        .iter()
        .find_map(Behavior::as_transform)
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
    let dag = compile_to_dag("let y = 1 < 2", "test.v3").expect("compiles");
    let cmp_node = dag
        .nodes()
        .iter()
        .find_map(Behavior::as_transform)
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
    let dag = compile_to_dag(src, "test.v3").expect("compiles");
    // The outermost Transform is the f(5) call — find it by
    // looking for the one whose target resolves to "f".
    let f_call = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_transform)
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
fn m17_block_bodied_fn_produces_unparsed_scaffold_declaration() {
    // Class 3 (QW1): `fn foo(x: Int) -> Int { body }` in a module
    // context is parsed as `FnExternalBody`. Lowering produces a
    // declaration whose connective is an `Arrow` carrying
    // `ArrowBody::Unparsed(body_span)`. The signature flows forward
    // through the declaration table — callers can type-check
    // against it — and the body source span is preserved for M2+
    // parser extensions.
    //
    // Note: at M1(2.7) the parser's opaque body consumer handles
    // arbitrary token contents, so the body text below doesn't need
    // to be real v3 — it just needs matching braces.
    let src = "fn foo(x: Int) -> Int { some unparseable body }";
    let dag = compile_any(src, "scaffold.v3");

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
        matches!(body, ArrowBody::Unparsed(_)),
        "block-bodied fn must carry ArrowBody::Unparsed, got {body:?}"
    );

    // The signature types resolve to the cached primitives.
    let int_id = find_named(&dag, "Int");
    let input_decl = dag.declaration(inputs[0]);
    let resolved_input_id = match &input_decl.connective {
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(id)) => *id,
        TypeConnective::Atom(AtomPayload::UnresolvedIdentifier(name)) => {
            panic!("input still unresolved: {name}")
        }
        _ => inputs[0],
    };
    let resolved_output_id = match &dag.declaration(output).connective {
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(id)) => *id,
        _ => output,
    };
    assert_eq!(resolved_input_id, int_id);
    assert_eq!(resolved_output_id, int_id);
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
    // direct `ResolvedIdentifier` wrapper. Either shape is
    // acceptable — the fact is "foo's type is Int" and we verify
    // it by walking one level.
    let foo_conn = dag.declaration(foo_id).connective.clone();
    let resolved = match &foo_conn {
        TypeConnective::Instantiation { template, .. } => *template,
        TypeConnective::Atom(AtomPayload::ResolvedIdentifier(id)) => *id,
        other => panic!("expected Instantiation or ResolvedIdentifier, got {other:?}"),
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
    let dag = compile_to_dag(src, "imports.v3").expect("compiles cleanly");
    // The `let x = 1` item still binds, confirming the module /
    // import items didn't break lowering.
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
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
    // For `let x = 1 + 2`, the walk path is:
    //   Int (source) → Int64 → OrderedRing<Word64> (algebra Conj)
    //   → "add" field → Arrow(T, T) -> T
    //   → substitute T → Int → signature (Int, Int) -> Int
    //
    // If the walk is wrong or returns Word64 (the old failing
    // mode), the operator output port's type won't match the Int
    // ports of the operands and the compile will fail with a
    // TypeMismatch. This test asserts the happy path compiles
    // cleanly and the operator output is typed as Int.
    let dag = compile_to_dag("let x = 1 + 2", "test.v3").expect("compiles");
    let bind_x = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "x")
        .expect("Bind(x) exists");
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
    let dag = compile_to_dag("let y = 1 < 2", "test.v3").expect("compiles");
    let bind_y = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "y")
        .expect("Bind(y) exists");
    let bool_shape = dag.bool_shape().expect("Bool cached at bootstrap");
    match dag.port(bind_y.value).state() {
        v3_compiler::dag::PortState::Resolved(ty) if *ty == bool_shape => {}
        other => panic!("expected Bind(y).value to be Resolved(Bool), got {other:?}"),
    }
}

#[test]
fn m17_r9_ordered_ring_carries_direct_operator_fields() {
    // Verify algebra.dag was extended with the direct operator
    // fields the §8.9 walk looks up. If any is missing the walk
    // falls back to the Rust scaffold bridge, which would make
    // the two tests above silently pass via the wrong path.
    let dag = Dag::new();
    let ordered_ring = find_named(&dag, "OrderedRing");
    let fields = match &dag.declaration(ordered_ring).connective {
        TypeConnective::Conj { children } => children.clone(),
        other => panic!("expected OrderedRing Conj, got {other:?}"),
    };
    for required in [
        "add", "sub", "mul", "div", "eq", "ne", "lt", "le", "gt", "ge",
    ] {
        assert!(
            fields.iter().any(|f| f.label == required),
            "OrderedRing missing direct operator field `{required}` — the §8.9 walk will fall back to the Rust scaffold bridge"
        );
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
            assert_eq!(*n, 1)
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

    // carrier → "+" (Literal String)
    assert_eq!(fields[3].0, "carrier");
    assert!(matches!(
        &fields[3].1,
        v3_compiler::dag::FieldValue::Literal(v3_compiler::dag::LiteralBits::String(s)) if s == "+"
    ));

    // cost → 1 (Literal Int)
    assert_eq!(fields[4].0, "cost");
    assert!(matches!(
        &fields[4].1,
        v3_compiler::dag::FieldValue::Literal(v3_compiler::dag::LiteralBits::Int(1))
    ));
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
    let src = "\
type Sign = Plus | Minus
fn always_zero(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }
let answer = always_zero(Plus)
";
    // The `Plus` in the call site `always_zero(Plus)` IS a variant
    // expression — it falls into the class-5 gap. Use a simpler
    // harness that avoids that by compiling just the fn body.
    let src_without_call = "\
type Sign = Plus | Minus
fn always_zero(s: Sign) -> Int = match s { Plus => 0, Minus => 1 }
";
    let dag = compile_any(src_without_call, "match.v3");
    // Find the always_zero fn declaration; its connective should
    // be an Arrow, and the Bind's sub-DAG should contain a Branch
    // with exactly two paths, each pattern ResolvedVariant(...).
    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "always_zero")
        .expect("Bind(always_zero) exists");
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
    let _ = src;
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
    let dag = compile_to_dag(src, "aliased_sum.v3").expect("compiles");
    assert!(dag.diagnostics().is_empty());

    // The Branch exists and its pattern resolution ran through
    // the alias walk — each path should carry a ResolvedVariant.
    let branch = dag
        .nodes()
        .iter()
        .find_map(Behavior::as_branch)
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
    // M1(2.8) R14: user-range ArrowBody::Unparsed scaffolds must
    // fail-closed. The FnExternalBody surface form exists so
    // std/bootstrap files with match/record/pipe/lambda bodies
    // can parse cleanly (their bodies become scaffolds), but
    // ordinary user code has no business shipping an opaque
    // body that the compiler never validates.
    //
    // Before R14, `fn foo(x: Int) -> Int { junk }` compiled
    // cleanly because nothing rejected the user-range
    // ArrowBody::Unparsed. After R14, the lower-side sweep
    // rejects every user-range Unparsed scaffold.
    let result = compile_to_dag("fn foo(x: Int) -> Int { junk }", "user.v3");
    assert!(
        result.is_err(),
        "user-range fn with opaque body must fail compile_to_dag"
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
    // M1(2.8) R13: the mutually-recursive fn rejection path used to
    // store ArrowBody::Pending, which decide_transform accepts as
    // scaffold (signature type-checks, body walking skipped).
    // Downstream callers of a mutually-recursive fn got Resolved
    // types even though the callee's Bind value port was already
    // Unresolved — a FAIL-CLOSED leak.
    //
    // Fix: emit ArrowBody::UserDefined(bind_id) so decide_transform's
    // UserDefined arm reads the Bind's value port state and
    // cascades Decision::Fail to callers when it's Unresolved.
    //
    // Regression shape: define two mutually recursive fns, then a
    // let binding that calls one of them. After R13, the let port
    // must cascade to Unresolved.
    let src = "\
fn a(n: Int) -> Int = b(n)
fn b(n: Int) -> Int = a(n)
let c = a(1)
";
    let dag = compile_any(src, "mutual_recursion_cascade.v3");
    assert!(
        !dag.diagnostics().is_empty(),
        "mutual recursion should produce a diagnostic"
    );
    let bind_c = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "c")
        .expect("Bind(c) exists");
    assert!(
        matches!(
            dag.port(bind_c.value).state(),
            v3_compiler::dag::PortState::Unresolved
        ),
        "caller of a mutually-recursive fn must cascade to Unresolved; got {:?}",
        dag.port(bind_c.value).state()
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
    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "always_zero")
        .expect("Bind(always_zero) exists");
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
    let dag = compile_to_dag(src, "ternary.v3").expect("compiles");
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
    let dag = compile_to_dag("let r = if 1 > 0 then 10 else 20", "if.v3").expect("compiles");
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
    let dag = compile_to_dag(src, "test.v3").expect("compiles");

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
    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "identity")
        .expect("Bind(identity) exists");
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
    let dag = compile_to_dag("type Foo = Int", "alias.v3").expect("compiles");
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
        let bind = dag
            .node(*bind_id)
            .as_bind()
            .expect("lambda body bind should exist");
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
        let bind = dag
            .node(*bind_id)
            .as_bind()
            .expect("lambda body bind should exist");
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
    let dag = compile_to_dag(src, "expr_record_literal.v3").expect("compiles");
    assert!(
        dag.diagnostics().is_empty(),
        "record literal in expression position should compile when an expected type is available, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
}

#[test]
fn prereq4_list_literal_in_expression_position_lowers_through_std_list_constructors() {
    let dag =
        compile_to_dag("let xs: List<Int> = [1, 2, 3]", "expr_list_literal.v3").expect("compiles");
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
    let dag = compile_to_dag(src, "field_access.v3").expect("compiles");
    let int_id = find_named(&dag, "Int");

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "get_x")
        .expect("Bind(get_x) exists");
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
    let dag = compile_to_dag(src, "field_access_multi_hop.v3").expect("compiles");

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "get_nested_x")
        .expect("Bind(get_nested_x) exists");

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
    let dag = compile_to_dag(src, "field_access_generic.v3").expect("compiles");
    let box_id = find_named(&dag, "Box");
    let box_t = dag.declaration(box_id).type_params[0];

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "read")
        .expect("Bind(read) exists");
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
    let dag = compile_to_dag(src, "payload_binding.v3").expect("compiles");

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "unwrap_or_zero")
        .expect("Bind(unwrap_or_zero) exists");
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
    let dag = compile_to_dag(src, "payload_field_access.v3").expect("compiles");

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "get_or_zero")
        .expect("Bind(get_or_zero) exists");
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
    let dag = compile_to_dag(src, "payload_binding_inferred.v3").expect("compiles");

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "unwrap_or_zero")
        .expect("Bind(unwrap_or_zero) exists");
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
    let dag = compile_to_dag(src, "payload_binding_generic.v3").expect("compiles");

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "unwrap_or_zero")
        .expect("Bind(unwrap_or_zero) exists");
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
    let dag = compile_to_dag(src, "optional_handle_field_projection.v3").expect("compiles");
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
    let dag = compile_to_dag(src, "payload_record_variant.v3").expect("compiles");
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
fn recursion_accepts_structural_descent_on_recursive_payload_field() {
    let src = "\
type IntList = Empty | Cons { head: Int, tail: IntList }
fn count(list: IntList) -> Int = match list { Empty => 0, Cons(payload) => 1 + count(payload.tail) }
";
    let dag = compile_to_dag(src, "structural_descent.v3").expect("compiles");
    assert!(
        dag.diagnostics().is_empty(),
        "structural descent on a recursive payload field should compile cleanly: {:?}",
        dag.diagnostics()
    );

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "count")
        .expect("Bind(count) exists");
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
fn recursive_generic_sum_can_reference_itself_in_payload_types() {
    let src = "\
type MyList<T> = Empty | Cons { head: T, tail: MyList<T> }
";
    let dag = compile_to_dag(src, "recursive_generic_sum.v3").expect("compiles");
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
    let dag = compile_to_dag(src, "std_list_structural_recursion.v3").expect("compiles");
    assert!(
        dag.diagnostics().is_empty(),
        "std.list structural recursion should compile cleanly: {:?}",
        dag.diagnostics()
    );
    assert_eq!(bind_value_type_decl(&dag, "n"), find_named(&dag, "Int"));

    let bind = dag
        .nodes()
        .iter()
        .filter_map(Behavior::as_bind)
        .find(|b| b.name == "count")
        .expect("Bind(count) exists");
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
    cons(
      loop_node.source,
      cons(loop_node.init, singleton(loop_node.bound.count))
    ),
    match behavior_result_port(d.nodes, loop_node.body) {
      MissingResultPort => empty()
      FoundResultPort(port) => singleton(port)
    }
  )

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
