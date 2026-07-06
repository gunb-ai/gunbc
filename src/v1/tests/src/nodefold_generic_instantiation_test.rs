use std::rc::Rc;
use std::sync::OnceLock;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const NODEFOLD_CERT: &str = "src/v2/test/claim/manual/nodefold_topdown_inline_algebra.dag";
const CHAINED_CERT: &str = "src/v2/test/claim/manual/chained_generic_field_access.dag";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn cert_sources(entry: &str) -> Vec<Rc<SourceFile>> {
    static NODEFOLD_CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    static CHAINED_CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    let cache = if entry == NODEFOLD_CERT {
        NODEFOLD_CACHE.get_or_init(|| load_cert_pairs(entry))
    } else {
        CHAINED_CACHE.get_or_init(|| load_cert_pairs(entry))
    };
    cache
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect()
}

fn load_cert_pairs(entry: &str) -> Vec<(String, String)> {
    let entry_content = std::fs::read_to_string(workspace_root().join(entry))
        .unwrap_or_else(|e| panic!("read {entry}: {e}"));
    resolve_imports_transitively_with_source_roots(entry, &entry_content, &v2_source_roots())
        .iter()
        .map(|s| (s.path.clone(), s.content.clone()))
        .collect()
}

fn assert_resolved_ok(result: &ResolvedPipelineResult, label: &str) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "{label}: expected resolved graph, got diagnostics {msgs:?} (graph present: {})",
        result.graph.is_some()
    );
}

#[test]
fn v2_nodefold_topdown_inline_algebra_compiles_and_runs() {
    let resolved = compile_to_resolved(Rc::new(cert_sources(NODEFOLD_CERT).into()));
    assert_resolved_ok(&resolved, NODEFOLD_CERT);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v1_interpreter::run(graph, resolved.source_indices.clone(), "use_inline_topdown") {
        Ok(Value::Int(8)) => {}
        other => panic!(
            "expected Int(8) (A=R=MyFold field access in inline algebra init), got {other:?}"
        ),
    }
}

#[test]
fn v2_chained_generic_field_access_compiles() {
    let resolved = compile_to_resolved(Rc::new(cert_sources(CHAINED_CERT).into()));
    assert_resolved_ok(&resolved, CHAINED_CERT);
}

#[test]
fn v2_chained_generic_field_access_runs() {
    let resolved = compile_to_resolved(Rc::new(cert_sources(CHAINED_CERT).into()));
    assert_resolved_ok(&resolved, CHAINED_CERT);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v1_interpreter::run(graph, resolved.source_indices.clone(), "use_chained") {
        Ok(Value::Int(9)) => {}
        other => panic!(
            "expected Int(9) from chained generic field access o.middle.inner.value, got {other:?}"
        ),
    }
}
