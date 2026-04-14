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
    ArrowBody, AtomPayload, Dag, DeclarationId, Field, TypeConnective,
};
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
    let dag = Dag::new();

    // Int must be present and shaped as Instantiation(OrderedRing, [T := Word64]).
    let int_id = find_named(&dag, "Int");
    let word64_id = find_named(&dag, "Word64");
    let ordered_ring_id = find_named(&dag, "OrderedRing");

    let (template, arguments) = match &dag.declaration(int_id).connective {
        TypeConnective::Instantiation {
            template,
            arguments,
        } => (*template, arguments.clone()),
        other => panic!("expected Int to be Instantiation, got {other:?}"),
    };
    assert_eq!(
        template, ordered_ring_id,
        "Int's template must be OrderedRing"
    );
    assert_eq!(
        arguments.len(),
        1,
        "Int's template has one parameter (T := Word64)"
    );
    assert_eq!(
        arguments[0].value, word64_id,
        "Int's T must bind to Word64"
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

    // Build the substitution map [T := Word64] from Int's Instantiation.
    let mut subst: HashMap<DeclarationId, DeclarationId> = HashMap::new();
    for arg in &arguments {
        subst.insert(arg.parameter, arg.value);
    }

    // Substitute the Arrow's inputs and output. A TypeParam resolves to
    // itself (as a DeclarationId) and the subst lookup maps T to Word64.
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
fn smoke_int_add_external_realization() {
    // M1_DESIGN.md §6.5 — the ExternalRealization substrate path walks
    // cleanly through inference. Bootstrap constructs the stub chain in
    // `bootstrap::inject_realization_stub`:
    //   - `Realization` meta-type (empty Conj)
    //   - `Int64_add_rust` realization instance with `meta_tag` pointing
    //     at the meta-type
    //   - `Int64_add` Arrow whose body is `ExternalRealization(instance)`
    //     rather than `Pending`
    //
    // The test asserts the chain is walkable, the Arrow's body is the
    // ExternalRealization variant (not Pending), and that compile_to_dag
    // completes without panicking on a user program that references the
    // realized Arrow's signature.
    let dag = Dag::new();

    let arrow_id = find_named(&dag, "Int64_add");
    let arrow_decl = dag.declaration(arrow_id);
    let (inputs, output, body) = match &arrow_decl.connective {
        TypeConnective::Arrow {
            inputs,
            output,
            body,
        } => (inputs.clone(), *output, body.clone()),
        other => panic!("expected Int64_add to be Arrow, got {other:?}"),
    };
    assert_eq!(inputs.len(), 2, "Int64_add takes two arguments");
    assert_eq!(inputs[0], output, "input[0] and output are the same type");

    // The key assertion: the Arrow body is ExternalRealization, not Pending.
    // This validates that the bootstrap path from §6.5 actually populates the
    // ExternalRealization variant and inference will later walk it as a valid
    // realization.
    let realization_id = match body {
        ArrowBody::ExternalRealization(id) => id,
        other => {
            panic!("expected Int64_add body to be ExternalRealization, got {other:?}")
        }
    };
    let realization_decl = dag.declaration(realization_id);
    assert_eq!(
        realization_decl.name.as_deref(),
        Some("Int64_add_rust"),
        "realization name should be Int64_add_rust"
    );

    // meta_tag points at the Realization meta-type; this is the Q0 split
    // (meta_tag ≠ inhabits) from PR #444.
    let meta_type_id = realization_decl
        .meta_tag
        .expect("realization instance must carry a meta_tag");
    let meta_type_decl = dag.declaration(meta_type_id);
    assert_eq!(
        meta_type_decl.name.as_deref(),
        Some("Realization"),
        "meta_tag points at the Realization meta-type"
    );
    assert!(
        realization_decl.inhabits.is_none(),
        "realization instance uses meta_tag only, not inhabits"
    );
    assert!(
        matches!(
            meta_type_decl.connective,
            TypeConnective::Conj { .. }
        ),
        "Realization meta-type is a Conj"
    );

    // Smoke check on inference: compile a small user program that invokes
    // the realized arrow via its declared name. `Int64_add` is a named
    // declaration in the bootstrap set, so lowering resolves the call
    // target to `arrow_id` and inference walks its Arrow signature. A clean
    // compile proves the ExternalRealization body does not panic inference.
    let result = compile_to_dag("let x: Int = Int64_add(1, 2)", "smoke.v3");
    match result {
        Ok(_) => {}
        Err(CompileError::Semantic(dag)) => panic!(
            "compile produced semantic diagnostics: {:?}",
            dag.diagnostics()
        ),
        Err(other) => panic!("unexpected structural error: {other:?}"),
    }
}
