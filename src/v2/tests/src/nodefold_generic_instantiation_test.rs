//! v2 R2 generic instantiation — items (b) NodeFold A,R binding and (c) chained field access.
//! dep-graph-2026-06-12 §4a / tidy-stag-172.

use std::rc::Rc;
use std::sync::OnceLock;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v2_compiler::v2_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const NODEFOLD_CERT: &str = "src/v4/test/claim/manual/nodefold_topdown_inline_algebra.dag";
const CHAINED_CERT: &str = "src/v4/test/claim/manual/chained_generic_field_access.dag";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
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
    resolve_imports_transitively_with_source_roots(entry, &entry_content, &v4_source_roots())
        .iter()
        .map(|s| (s.path.clone(), s.content.clone()))
        .collect()
}

fn assert_resolved_ok(result: &ResolvedPipelineResult, label: &str) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "{label}: expected resolved graph, got diagnostics {msgs:?} (graph present: {})",
        result.graph.is_some()
    );
}

#[test]
fn v4_nodefold_topdown_inline_algebra_compiles_and_runs() {
    let resolved = compile_to_resolved(Rc::new(cert_sources(NODEFOLD_CERT)));
    assert_resolved_ok(&resolved, NODEFOLD_CERT);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "use_inline_topdown") {
        Ok(Value::Int(8)) => {}
        other => panic!(
            "expected Int(8) (A=R=MyFold field access in inline algebra init), got {other:?}"
        ),
    }
}

#[test]
fn v4_chained_generic_field_access_compiles_and_runs() {
    let resolved = compile_to_resolved(Rc::new(cert_sources(CHAINED_CERT)));
    assert_resolved_ok(&resolved, CHAINED_CERT);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    match v2_interpreter::run(graph, resolved.source_indices.clone(), "use_chained") {
        Ok(Value::Int(9)) => {}
        other => panic!(
            "expected Int(9) from chained generic field access o.middle.inner.value, got {other:?}"
        ),
    }
}

#[test]
fn debug_nodefold_expr_errors_in_graph() {
    use v2_compiler::v2_std_core::{authored_name_at, ExprData, Node};
    fn walk(
        n: &Rc<Node>,
        path: &str,
        si: &std::collections::HashMap<String, Rc<v2_compiler::v2_std_core::NewlineIndex>>,
        out: &mut Vec<String>,
    ) {
        if let ExprData::ExprError { message, .. } = &*n.expr_data {
            out.push(format!("{path}: {message}"));
        }
        if let Some(body) = &n.body {
            walk(body, &format!("{path}.body"), si, out);
        }
        for (i, c) in n.children.iter().enumerate() {
            walk(c, &format!("{path}.children[{i}]"), si, out);
        }
    }
    let resolved = compile_to_resolved(Rc::new(cert_sources(NODEFOLD_CERT)));
    let graph = resolved.graph.as_ref().expect("graph");
    let si = &*resolved.source_indices;
    let mut errors = Vec::new();
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            let name = authored_name_at(Rc::new(si.clone()), item.clone());
            if let Some(body) = &item.body {
                walk(body, &name, si, &mut errors);
            }
        }
    }
    assert!(errors.is_empty(), "ExprError nodes in graph: {errors:?}");
}
