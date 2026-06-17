//! §5 discriminating witness: body_producer closure resolve is bounded AND inference stays fail-closed.
//! Run: cargo test -p v1-compiler-tests body_producer_infer_perf_witness -- --nocapture

use std::rc::Rc;
use std::time::{Duration, Instant};

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const ENTRY: &str = "src/v2/compiler/manual/pbp_body_producer_perf_repro.dag";
const WRONG_TYPE_ENTRY: &str = "src/v2/compiler/manual/pbp_body_producer_wrong_type_repro.dag";
/// Cold resolve budget after target_model VEP shard (was ~244s pre-fix).
const RESOLVE_BUDGET: Duration = Duration::from_secs(60);

fn sources_for(entry: &str) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let content = std::fs::read_to_string(ws.join(entry)).expect("read entry");
    resolve_imports_transitively_with_source_roots(entry, &content, &[ws.join("src/v2"), ws.join("dsl")])
}

fn non_complexity_errors(resolved: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect()
}

#[test]
fn body_producer_infer_perf_witness_resolves_within_budget() {
    let sources = sources_for(ENTRY);
    let t0 = Instant::now();
    let resolved = compile_to_resolved(Rc::new(sources));
    let elapsed = t0.elapsed();
    let errs = non_complexity_errors(&resolved);
    assert!(
        errs.is_empty() && resolved.graph.is_some(),
        "body_producer closure must resolve cleanly, got errors {errs:?}"
    );
    assert!(
        elapsed < RESOLVE_BUDGET,
        "body_producer closure resolve took {:?}, budget {:?}",
        elapsed,
        RESOLVE_BUDGET
    );
}

#[test]
fn body_producer_infer_perf_witness_wrong_type_still_rejects() {
    let sources = sources_for(WRONG_TYPE_ENTRY);
    let resolved = compile_to_resolved(Rc::new(sources));
    let errs = non_complexity_errors(&resolved);
    assert!(
        !errs.is_empty() || resolved.graph.is_none(),
        "wrong-type repro must fail inference/resolve (fail-closed), got clean graph"
    );
}
