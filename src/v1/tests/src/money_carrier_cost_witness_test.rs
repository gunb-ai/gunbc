use std::rc::Rc;

use v1_compiler::cli_run;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};
use v1_compiler::v1_std_core::diagnostic_to_message;

use crate::helpers::{source_roots, workspace_root};

fn blocking_diagnostics(resolved: &ResolvedPipelineResult) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect()
}

fn source_root_strings() -> Vec<String> {
    source_roots()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

#[test]
fn v1_std_currency_dag_resolves() {
    let roots = source_root_strings();
    let entry = workspace_root().join("dsl/std/currency.dag");
    let entry = entry.to_string_lossy().to_string();
    let sources = cli_run::load_sources_for_entry(&roots, &entry)
        .unwrap_or_else(|e| panic!("failed to load {entry}: {e}"));
    let resolved = compile_to_resolved(Rc::new(sources));
    let msgs = blocking_diagnostics(resolved.as_ref());
    assert!(
        msgs.is_empty(),
        "currency.dag should resolve on v2: {msgs:?}"
    );
}

#[test]
fn cost_projection_float_witness_evaluates_true() {
    let roots = source_root_strings();
    let entry = workspace_root().join("dsl/examples/cost_estimate/cost_estimate.dag");
    let entry = entry.to_string_lossy().to_string();
    let sources = cli_run::load_sources_for_entry(&roots, &entry)
        .unwrap_or_else(|e| panic!("failed to load {entry}: {e}"));
    let resolved = compile_to_resolved(Rc::new(sources));
    assert!(
        blocking_diagnostics(resolved.as_ref()).is_empty(),
        "cost_estimate witness should resolve on v2: {:?}",
        blocking_diagnostics(resolved.as_ref())
    );
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx =
        cli_run::make_eval_context(graph, resolved.source_indices.clone(), ExecutionMode::Wet);
    match v1_interpreter::run_in_context(&ctx, "cost_projection_float_witness", false) {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected Bool(true) Float-in-v2 witness, got {other:?}"),
    }
}
