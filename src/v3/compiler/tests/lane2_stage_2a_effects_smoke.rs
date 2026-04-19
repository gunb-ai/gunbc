// Lane 2 Stage 2a acceptance smoke: `src/v3/std/effects.dag` compiles
// cleanly under `Dag::new()` bootstrap and exposes 5 representative
// function signatures.
//
// The Stage 2a scope (per docs/lane2-compile-time-proofs.md:37) is
// pure structural carry-over of `dsl/std/effects.dag`. This smoke
// test encodes the acceptance gate: "compiles cleanly in v3; minimal
// smoke test asserts parse + 5 representative function signatures."
// Parse is covered in `real_stdlib_parse_smoke::effects_dag_parses`;
// here we prove bootstrap picks the file up without diagnostics and
// registers the function declarations downstream consumers need.

use v3_compiler::dag::{ArrowBody, Dag, TypeConnective};

fn arrow_body(dag: &Dag, name: &str) -> ArrowBody {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("expected `{name}` declaration after bootstrap"));
    match &decl.connective {
        TypeConnective::Arrow { body, .. } => body.clone(),
        other => panic!("expected `{name}` to be Arrow, got {other:?}"),
    }
}

fn assert_record_type(dag: &Dag, name: &str) {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("expected type `{name}` declaration after bootstrap"));
    match &decl.connective {
        TypeConnective::Conj { .. } => {}
        TypeConnective::Disj { .. } => {}
        TypeConnective::Atom(_) => {}
        other => panic!("expected `{name}` to be a record/enum/atom type, got {other:?}"),
    }
}

#[test]
fn effects_dag_bootstraps_without_diagnostics() {
    let dag = Dag::new();
    let effects_diags: Vec<_> = dag
        .diagnostics()
        .iter()
        .filter(|(_, diag)| format!("{diag:?}").contains("effects.dag"))
        .collect();
    assert!(
        effects_diags.is_empty(),
        "src/v3/std/effects.dag produced diagnostics during bootstrap: {effects_diags:?}"
    );
}

#[test]
fn effects_dag_exposes_five_representative_function_signatures() {
    let dag = Dag::new();

    // Five representative function signatures from the Stage 2a scope
    // — one per algebraic concern the downstream lens pipeline uses.
    // Per-arrow body-state is not asserted here because bootstrap
    // lowers bodies into whatever shape the current compiler can
    // handle (see `list.dag` staging note); the signature presence
    // is the load-bearing acceptance bit.
    for name in [
        "is_idempotent_effect",
        "compose_effects",
        "derive_effect_shape",
        "check_modifier_vs_derivation",
        "generate_idempotency_obligations",
    ] {
        let _body = arrow_body(&dag, name);
    }
}

#[test]
fn effects_dag_inlines_required_http_path_helpers() {
    let dag = Dag::new();

    for name in ["parse_path_template", "last_path_param"] {
        let _body = arrow_body(&dag, name);
    }
}

#[test]
fn effects_dag_exposes_core_effect_algebra_types() {
    let dag = Dag::new();

    // Types the downstream lens consumers query by name. Asserting
    // the connectives are record/enum/atom shapes (not left as
    // Pending placeholders) guards against regression into a
    // parse-only carry-over where the types fail to lower.
    for name in [
        "EffectShape",
        "IdempotentShape",
        "BreakingShape",
        "CreateCause",
        "KeySource",
        "IdempotencyEvidence",
        "CompositionVerdict",
        "OperationEffect",
        "BreakingOperation",
        "ModifierAgreement",
        "ModifierAxisCheck",
        "ModifierCheck",
        "WorkflowEffect",
        "BoolPortRef",
        "BranchArm",
        "WorkflowIdempotencyReport",
        "WorkflowParallelismReport",
        "ParallelismUnsupportedKind",
        "ParallelismUnsupportedDetail",
        "IdempotencyUnsupportedDetail",
    ] {
        assert_record_type(&dag, name);
    }
}
