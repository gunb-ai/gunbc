use std::rc::Rc;
use std::time::{Duration, Instant};

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const BISECT_ENTRY: &str = "src/v2/test/claim/manual/validate_ingest_staging_stage_bisect_test.dag";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn bisect_sources() -> Vec<Rc<SourceFile>> {
    let entry_content = std::fs::read_to_string(workspace_root().join(BISECT_ENTRY))
        .unwrap_or_else(|e| panic!("read {BISECT_ENTRY}: {e}"));
    resolve_imports_transitively_with_source_roots(BISECT_ENTRY, &entry_content, &v2_source_roots())
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
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected resolved graph for {BISECT_ENTRY}, got {msgs:?}"
    );
}

fn assert_witness_terminates(function: &str, budget: Duration) {
    let resolved = compile_to_resolved(Rc::new(bisect_sources().into()));
    assert_resolved_ok(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        v1_interpreter::ExecutionMode::Wet,
    );
    let start = Instant::now();
    match v1_interpreter::run_in_context(&ctx, function, false) {
        Ok(Value::Bool(true)) => {}
        other => panic!("expected Bool(true) from {function}, got {other:?}"),
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed <= budget,
        "{function} exceeded budget {:?} (elapsed {:?})",
        budget,
        elapsed
    );
}

#[test]
fn interpreted_parse_bisect_tokenize_terminates() {
    assert_witness_terminates("bisect_tokenize_terminates", Duration::from_secs(30));
}

#[test]
fn interpreted_parse_bisect_parse_terminates() {
    assert_witness_terminates("bisect_parse_terminates", Duration::from_secs(60));
}

#[test]
fn interpreted_parse_bisect_wave1_add_correctness_holds() {
    assert_witness_terminates(
        "witness_bisect_wave1_parse_module_add_correctness_holds",
        Duration::from_secs(90),
    );
}

#[test]
fn interpreted_parse_bisect_normalize_terminates() {
    assert_witness_terminates("bisect_normalize_terminates", Duration::from_secs(60));
}
