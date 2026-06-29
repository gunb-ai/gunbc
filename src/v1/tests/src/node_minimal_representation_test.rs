use std::rc::Rc;
use std::sync::OnceLock;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const CERT_ENTRY: &str = "src/v2/test/claim/manual/node_minimal_representation_test.dag";

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn cert_sources() -> Vec<Rc<SourceFile>> {
    static CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let entry_content = std::fs::read_to_string(workspace_root().join(CERT_ENTRY))
                .unwrap_or_else(|e| panic!("read {CERT_ENTRY}: {e}"));
            resolve_imports_transitively_with_source_roots(
                CERT_ENTRY,
                &entry_content,
                &v2_source_roots(),
            )
            .iter()
            .map(|s| (s.path.clone(), s.content.clone()))
            .collect()
        })
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
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

fn run_bool_test(
    graph: &v1_compiler::v1_compiler_infer_items::ResolvedGraph,
    source_indices: std::rc::Rc<
        std::collections::HashMap<String, std::rc::Rc<v1_compiler::v1_std_core::NewlineIndex>>,
    >,
    name: &str,
) -> bool {
    match v1_interpreter::run(graph, source_indices, name) {
        Ok(Value::Bool(b)) => b,
        other => panic!("{name}: expected Bool, got {other:?}"),
    }
}

#[test]
fn v2_node_minimal_representation_compiles_and_witnesses_hold() {
    let resolved = compile_to_resolved(Rc::new(cert_sources()));
    assert_resolved_ok(&resolved, CERT_ENTRY);
    let graph = resolved
        .graph
        .as_ref()
        .expect("graph after successful resolve");
    let source_indices = resolved.source_indices.clone();

    assert!(
        run_bool_test(
            graph,
            source_indices.clone(),
            "node_minimal_expr_leaf_field_count_below_superset_holds"
        ),
        "expr leaf field count should be below 18-field superset"
    );
    assert!(
        run_bool_test(
            graph,
            source_indices.clone(),
            "node_minimal_expr_leaf_drops_dead_fields_holds"
        ),
        "expr leaf should not carry connective/params/uses/body"
    );
    assert!(
        run_bool_test(
            graph,
            source_indices.clone(),
            "node_minimal_substrate_round_trips_v2_node_holds"
        ),
        "substrate round-trip from v2 Node should preserve kind/children/occurrence_id"
    );
    assert!(
        run_bool_test(
            graph,
            source_indices.clone(),
            "node_minimal_substrate_has_no_superset_fields_holds"
        ),
        "substrate graph node should not map to v1 superset fields"
    );
    assert!(
        run_bool_test(
            graph,
            source_indices,
            "node_minimal_kind_of_expr_leaf_holds"
        ),
        "minimal_node_kind_of should classify expr leaf example"
    );
}
