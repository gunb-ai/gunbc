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
    ArrowBody, AtomPayload, Behavior, Dag, DeclarationId, Field, TransformTarget, TypeConnective,
};
use v3_compiler::operators::{ArithmeticOp, ComparisonOp, OperatorKind};
use v3_compiler::CompileError;

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
    let (arrow_inputs, arrow_output, arrow_body) = match &dag.declaration(add_field.ty).connective
    {
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

    let substitute = |id: DeclarationId| -> DeclarationId {
        *subst.get(&id).unwrap_or(&id)
    };
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
    // `Realization` is no longer a top-level bootstrap declaration
    // after M1(2.6) review round 4 — realization facts live in
    // `dsl/extdeps/languages/*` per the thesis, not in compiler code.
    // The §6.5 smoke test moved to a `#[cfg(test)]` module inside
    // `bootstrap.rs` where it can construct a local realization chain
    // without polluting `Dag::new()`.
    assert!(
        dag.declaration_by_name("Realization").is_none(),
        "Realization must not be a production bootstrap declaration"
    );
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
        other => panic!(
            "expected TransformTarget::Operator(Arithmetic(Add)), got {other:?}"
        ),
    }
}

#[test]
fn m17_comparison_operator_lowers_to_structural_transform_target() {
    // Class 2: comparison operators commit to the
    // `OperatorKind::Comparison` variant at parse time. The
    // arithmetic-vs-comparison split is structural, not a sibling
    // string match.
    let dag =
        compile_to_dag("let y = 1 < 2", "test.v3").expect("compiles");
    let cmp_node = dag
        .nodes()
        .iter()
        .find_map(Behavior::as_transform)
        .expect("Transform node exists");
    match &cmp_node.target {
        TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Lt)) => {}
        other => panic!(
            "expected TransformTarget::Operator(Comparison(Lt)), got {other:?}"
        ),
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
    assert_eq!(name, Some("f"), "Callable target points at user function `f`");
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
