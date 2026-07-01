use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const CONFORMANCE_ENTRY: &str =
    "src/v2/test/claim/manual/coproduct_reflection_conformance_test.dag";
const WITNESS_FN: &str = "coproduct_reflection_conformance_holds";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn cert_sources() -> Vec<Rc<SourceFile>> {
    let entry_content = std::fs::read_to_string(workspace_root().join(CONFORMANCE_ENTRY))
        .unwrap_or_else(|e| panic!("read {CONFORMANCE_ENTRY}: {e}"));
    resolve_imports_transitively_with_source_roots(
        CONFORMANCE_ENTRY,
        &entry_content,
        &v2_source_roots(),
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
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
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
    v1_interpreter::run(graph, resolved.source_indices.clone(), function)
        .unwrap_or_else(|e| panic!("run {function}: {e}"))
}

#[test]
fn coproduct_reflection_std_node_bridge_fns_are_intercept_wired() {
    let source = include_str!("../../stage0/src/v1_interpreter.rs");
    for name in v1_interpreter::std_node_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.node bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_node_query_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.node_query bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_concept_index_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.concept_index bridge `{name}`"
        );
    }
    for name in v1_interpreter::std_fn_index_bridge_fn_names() {
        assert!(
            source.contains(&format!("\"{name}\" =>")),
            "eval_call intercept must wire v2.std.fn_index bridge `{name}`"
        );
    }
    assert!(
        source.contains("is_v4_std_node_bridge_call"),
        "eval_call must gate v2.std.node bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_node_query_bridge_call"),
        "eval_call must gate v2.std.node_query bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_concept_index_bridge_call"),
        "eval_call must gate v2.std.concept_index bridge dispatch"
    );
    assert!(
        source.contains("is_v4_std_fn_index_bridge_call"),
        "eval_call must gate v2.std.fn_index bridge dispatch"
    );
}

#[test]
fn coproduct_reflection_conformance_holds() {
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    for (name, expect) in [
        ("witness_connective_behavior_arm_sets_are_distinct", true),
        (
            "witness_behavior_nullary_inhabitant_set_matches_arm_keys",
            true,
        ),
        ("witness_connective_nullary_inhabitants_fail_closed", true),
    ] {
        match run_witness(&resolved, name) {
            Value::Bool(v) if v == expect => {}
            other => panic!("witness {name}: expected Bool({expect}), got {other:?}"),
        }
    }
    match run_witness(&resolved, WITNESS_FN) {
        Value::Bool(true) => {}
        other => {
            panic!("expected Bool(true) from {WITNESS_FN} (conformance), got {other:?}")
        }
    }
}

#[test]
fn coproduct_reflection_connective_behavior_arm_sets_are_distinct() {
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

const CONCEPT_INDEX_ENTRY: &str = "src/v2/test/claim/concept_index_enumeration_test.dag";

#[test]
fn concept_index_parse_only_perturb_witnesses_hold() {
    let roots: Vec<String> = v2_source_roots()
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let entry = workspace_root()
        .join(CONCEPT_INDEX_ENTRY)
        .to_string_lossy()
        .into_owned();
    let (graph, si) = cli_run::resolve_entry_graph(&roots, &entry).expect("resolve entry");
    let ctx = cli_run::make_eval_context(&graph, si, ExecutionMode::Wet);
    let outcome = cli_run::run_claim(&ctx, "concept_index_enumeration_witnesses");
    assert_eq!(
        outcome,
        cli_run::ClaimOutcome::Pass,
        "concept_index_enumeration_witnesses must pass (includes parse-only perturb-RED)"
    );
}
