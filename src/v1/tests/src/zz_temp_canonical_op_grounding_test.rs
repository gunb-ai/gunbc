use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::compile_to_resolved;
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};
use v1_compiler::v1_std_core::diagnostic_to_message;

use crate::helpers::{v2_layer_roots, workspace_root};

const TEST_FNS: &[&str] = &[
    "arith_ops_round_trip",
    "boolean_ops_round_trip",
    "comparison_ops_round_trip",
    "distinct_algebra_fields_discriminate",
    "shared_compare_field_relations_discriminate",
    "cross_family_ops_discriminate",
    "same_op_is_equal",
    "catalog_lookup_accepts_grounded_op",
    "catalog_lookup_misses_absent_op",
];

#[test]
fn canonical_operation_grounding_witnesses_run_green() {
    let roots: Vec<String> = v2_layer_roots()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    let entry = workspace_root()
        .join("src/v2/compiler/manual/target_model_canonical_operation_grounding_test.dag");
    let entry = entry.to_string_lossy().to_string();
    let sources = cli_run::load_sources_for_entry(&roots, &entry)
        .unwrap_or_else(|e| panic!("failed to load {entry}: {e}"));
    let resolved = compile_to_resolved(Rc::new(sources));
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(msgs.is_empty(), "witness corpus should resolve: {msgs:?}");
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    for f in TEST_FNS {
        match v1_interpreter::run_in_context(&ctx, f, false) {
            Ok(Value::Bool(true)) => {}
            other => panic!("{f} expected Bool(true), got {other:?}"),
        }
    }
}
