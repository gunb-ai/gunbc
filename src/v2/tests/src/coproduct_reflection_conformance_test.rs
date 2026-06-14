//! R-reflect Phase 2a: Path-3 key-set conformance on Connective/Behavior by execution.

use std::rc::Rc;

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

#[test]
fn coproduct_reflection_path3_connective_behavior_conformance_holds() {
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), WITNESS_FN) {
        Ok(Value::Bool(true)) => {}
        other => panic!(
            "expected Bool(true) from {WITNESS_FN} (Path-3 key-set conformance), got {other:?}"
        ),
    }
}
