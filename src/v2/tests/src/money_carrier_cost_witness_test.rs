//! Float-in-v2 consumer + ISO 4217 currency model — v2 resolve + eval proof.
//!
//! Pairs with #4825 (Float-in-v2 parse/resolve). #4825 proved `dsl/std/float.dag`
//! RESOLVES on v2; these tests prove a real consumer of `Float` (a cost
//! projection) and the ISO 4217 `CurrencyCode` enum RESOLVE and EVALUATE on the
//! v2 interpreter end-to-end.
//!
//! Interim scope: the real `product.compute_fabric.CostEstimate` claim-run is gated
//! on #4831 (Option-in-v2) + #4826 (G1 identifier-variant type args for Measure
//! value eval); see `dsl/examples/cost_estimate/cost_estimate.dag`. This witness
//! is the interim Float-in-v2 proof until those land.

use std::rc::Rc;

use v2_compiler::cli_run;
use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v2_compiler::v2_interpreter::{self, Value};
use v2_compiler::v2_std_core::diagnostic_to_message;

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
fn v2_std_currency_dag_resolves() {
    // Use the scoped entry loader (transitive import closure only), matching the
    // witness test below — the whole-tree `resolve_imports_transitively_*`
    // helper walks every `.dag` under the roots and is far slower here.
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

/// The Float-in-v2 consumer witness (`dsl/examples/cost_estimate`) evaluates to
/// `true`: a `Float` field, a float literal, Float multiplication, Float
/// comparison, and the currency query all run on the v2 interpreter.
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
    let ctx = cli_run::make_eval_context(graph, resolved.source_indices.clone());
    match v2_interpreter::run_in_context(&ctx, "cost_projection_float_witness", false) {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected Bool(true) Float-in-v2 witness, got {other:?}"),
    }
}
