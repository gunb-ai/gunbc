//! v2 R2 generic instantiation — fold_list/fold_list_right cons|snoc callbacks bind T
//! from xs: FreeMonoid<T> (dep-graph-2026-06-12 §4a / tidy-stag-172).
//!
//! Executes the v4 witness through the real v2 compile+interpret pipeline (same path as
//! `gunbc run --source-root src/v4` claim runners).

use std::rc::Rc;
use std::sync::OnceLock;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const CERT_ENTRY: &str = "src/v4/test/claim/manual/fold_list_generic_instantiation.dag";
const WITNESS_FN: &str = "fold_list_generic_instantiation_holds";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

fn cert_source_pairs() -> &'static Vec<(String, String)> {
    static CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let entry_content = std::fs::read_to_string(workspace_root().join(CERT_ENTRY))
            .unwrap_or_else(|e| panic!("read {CERT_ENTRY}: {e}"));
        resolve_imports_transitively_with_source_roots(
            CERT_ENTRY,
            &entry_content,
            &v4_source_roots(),
        )
        .iter()
        .map(|s| (s.path.clone(), s.content.clone()))
        .collect()
    })
}

fn cert_sources() -> Vec<Rc<SourceFile>> {
    cert_source_pairs()
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
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
        "expected resolved graph for {CERT_ENTRY}, got diagnostics {msgs:?} (graph present: {})",
        resolved.graph.is_some()
    );
}

#[test]
fn v4_fold_list_generic_instantiation_holds() {
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), WITNESS_FN) {
        Ok(Value::Bool(true)) => {}
        other => panic!(
            "expected Bool(true) from {WITNESS_FN} (fold_list cons/snoc bind T from xs), got {other:?}"
        ),
    }
}

#[test]
fn v4_name_resolve_compiles_with_fold_list_dissolution() {
    const ENTRY: &str = "src/v4/compiler/03_name_resolve.dag";
    let entry_content = std::fs::read_to_string(workspace_root().join(ENTRY))
        .unwrap_or_else(|e| panic!("read {ENTRY}: {e}"));
    let sources = resolve_imports_transitively_with_source_roots(
        ENTRY,
        &entry_content,
        &v4_source_roots(),
    );
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_ok(&resolved);
}
