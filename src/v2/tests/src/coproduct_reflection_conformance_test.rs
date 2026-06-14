//! R-reflect Phase 2a: Path-3 key-set conformance on Connective/Behavior by execution.

use std::rc::Rc;

use v2_compiler::cli_run;
use v2_compiler::coproduct_reflection;
use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const CONFORMANCE_ENTRY: &str = "src/v4/test/claim/manual/coproduct_reflection_conformance.dag";
const WITNESS_FN: &str = "coproduct_reflection_conformance_holds";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

fn cert_sources() -> Vec<Rc<SourceFile>> {
    let entry_content = std::fs::read_to_string(workspace_root().join(CONFORMANCE_ENTRY))
        .unwrap_or_else(|e| panic!("read {CONFORMANCE_ENTRY}: {e}"));
    resolve_imports_transitively_with_source_roots(
        CONFORMANCE_ENTRY,
        &entry_content,
        &v4_source_roots(),
    )
    .iter()
    .map(|s| {
        Rc::new(SourceFile {
            path: s.path.clone(),
            content: s.content.clone(),
        })
    })
    .collect()
}

fn assert_resolved_ok(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected resolved graph for {CONFORMANCE_ENTRY}, got diagnostics {msgs:?}"
    );
}

fn run_witness(resolved: &ResolvedPipelineResult, function: &str) -> Value {
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    v2_interpreter::run(graph, resolved.source_indices.clone(), function)
        .unwrap_or_else(|e| panic!("run {function}: {e}"))
}

#[test]
fn coproduct_reflection_path3_connective_behavior_conformance_holds() {
    std::env::set_var("GUNBC_ROOT", workspace_root());
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    match run_witness(&resolved, WITNESS_FN) {
        Value::Bool(true) => {}
        other => panic!(
            "expected Bool(true) from {WITNESS_FN} (Path-3 key-set conformance), got {other:?}"
        ),
    }
}

#[test]
fn coproduct_reflection_connective_behavior_arm_sets_are_distinct() {
    std::env::set_var("GUNBC_ROOT", workspace_root());
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    match run_witness(
        &resolved,
        "witness_connective_behavior_arm_sets_are_distinct",
    ) {
        Value::Bool(true) => {}
        other => panic!("expected distinct arm sets, got {other:?}"),
    }
}

#[test]
fn coproduct_reflection_path3_witness_fails_on_dropped_disj_arm() {
    std::env::set_var("GUNBC_ROOT", workspace_root());
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    let ctx = cli_run::make_eval_context(graph, resolved.source_indices.clone());
    let connective = (None, Value::Str("Connective".to_string()));

    let reflection_keys =
        coproduct_reflection::eval_coproduct_arm_keys(&ctx, &[connective.clone()])
            .expect("reflection keys");
    let corrupted =
        coproduct_reflection::eval_coproduct_arm_keys_with_dropped_last_arm(&ctx, "Connective")
            .expect("corrupted keys");
    let syntactic = coproduct_reflection::eval_syntactic_coproduct_arm_keys(&ctx, &[connective])
        .expect("syntactic keys");

    assert_ne!(
        reflection_keys, corrupted,
        "dropped-arm corruption must change reflection output"
    );
    assert_ne!(
        corrupted, syntactic,
        "mechanism drift (dropped Disj arm) must break Path-3 bag_eq witness"
    );
}
