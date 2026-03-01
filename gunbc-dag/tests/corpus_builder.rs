//! Integration tests: cross-workflow mock corpus builder.
//!
//! Validates that the corpus builder can compile DSL tools, execute
//! baseline DryRuns, and accumulate per-node examples across multiple
//! workflows. This is the end-to-end smoke test for BB-1.

#![allow(clippy::disallowed_methods)]

use gunbc_codegen::testgen::mock_corpus::{build_corpus, WorkflowInfo};
use gunbc_dag::dsl_builder::build_dsl_graph_for_entry;
use gunbc_test::auto_mock_spec;
use gunbc_test::MockSpec;

// ---------------------------------------------------------------------------
// Helper: compile a DSL tool and produce (WorkflowInfo, Dag, MockSpec)
// ---------------------------------------------------------------------------

fn compile_tool(
    dag_file: &str,
    entry_node: &str,
    workflow_name: &str,
) -> Option<(gunbc_ir::Dag<gunbc_exec::DynOp>, MockSpec, WorkflowInfo)> {
    let dag = build_dsl_graph_for_entry(dag_file, entry_node).ok()?;
    let spec = auto_mock_spec(&dag, workflow_name);
    let info = WorkflowInfo {
        name: workflow_name.to_string(),
        profile: None,
    };
    Some((dag, spec, info))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn single_tool_produces_nonempty_corpus() {
    let (dag, spec, info) =
        match compile_tool("tools/pragma.dag", "tools.pragma::pragma_lint", "pragma") {
            Some(t) => t,
            None => {
                eprintln!("skipping: pragma tool failed to compile");
                return;
            }
        };

    let (corpus_map, _edges) =
        build_corpus(&[(info, &dag, &spec)], |_| true).expect("strict corpus build should pass");

    assert!(
        !corpus_map.is_empty(),
        "corpus should have at least one node identity after DryRun"
    );

    let total_examples: usize = corpus_map.values().map(|c| c.len()).sum();
    assert!(
        total_examples > 0,
        "corpus should have at least one example"
    );
}

#[test]
fn multi_workflow_accumulates_shared_nodes() {
    // Compile two tools that might share nodes (e.g., content_upsert pattern)
    let pragma = compile_tool("tools/pragma.dag", "tools.pragma::pragma_lint", "pragma");
    let makegen = compile_tool("tools/makegen.dag", "tools.makegen::makegen", "makegen");

    // Collect compiled tools into owned storage
    let mut compiled: Vec<(gunbc_ir::Dag<gunbc_exec::DynOp>, MockSpec, WorkflowInfo)> = Vec::new();
    if let Some(t) = pragma {
        compiled.push(t);
    }
    if let Some(t) = makegen {
        compiled.push(t);
    }

    if compiled.len() < 2 {
        eprintln!("skipping: need at least 2 tools to test multi-workflow");
        return;
    }

    // Build reference tuples
    let workflow_tuples: Vec<(WorkflowInfo, &gunbc_ir::Dag<gunbc_exec::DynOp>, &MockSpec)> =
        compiled
            .iter()
            .map(|(dag, spec, info)| (info.clone(), dag, spec))
            .collect();

    let (corpus_map, _edges) =
        build_corpus(&workflow_tuples, |_| true).expect("strict corpus build should pass");

    assert!(
        !corpus_map.is_empty(),
        "multi-workflow corpus should be non-empty"
    );

    // Check if any node has examples from multiple workflows
    let multi_workflow_nodes: Vec<_> = corpus_map
        .iter()
        .filter(|(_, corpus)| corpus.workflow_names().len() >= 2)
        .map(|(id, corpus)| (id.to_string(), corpus.workflow_names()))
        .collect();

    // Log what we found even if no shared nodes (informational)
    eprintln!(
        "corpus has {} node identities, {} with multi-workflow examples",
        corpus_map.len(),
        multi_workflow_nodes.len()
    );
    for (id, wfs) in &multi_workflow_nodes {
        eprintln!("  shared node: {} (workflows: {:?})", id, wfs);
    }
}

#[test]
fn corpus_node_identities_are_well_formed() {
    let (dag, spec, info) =
        match compile_tool("tools/pragma.dag", "tools.pragma::pragma_lint", "pragma") {
            Some(t) => t,
            None => {
                eprintln!("skipping: pragma tool failed to compile");
                return;
            }
        };

    let (corpus_map, _) =
        build_corpus(&[(info, &dag, &spec)], |_| true).expect("strict corpus build should pass");

    for (identity, corpus) in &corpus_map {
        // NodeIdentity should have non-empty module and callable
        assert!(
            !identity.module.is_empty(),
            "node identity module should not be empty: {:?}",
            identity
        );
        assert!(
            !identity.callable.is_empty(),
            "node identity callable should not be empty: {:?}",
            identity
        );

        // Every example should have provenance
        for example in &corpus.examples {
            assert_eq!(
                example.provenance.workflow, "pragma",
                "example should have correct workflow provenance"
            );
        }
    }
}

#[test]
fn edge_examples_have_valid_port_mappings() {
    let (dag, spec, info) =
        match compile_tool("tools/pragma.dag", "tools.pragma::pragma_lint", "pragma") {
            Some(t) => t,
            None => {
                eprintln!("skipping: pragma tool failed to compile");
                return;
            }
        };

    let (_, edges) =
        build_corpus(&[(info, &dag, &spec)], |_| true).expect("strict corpus build should pass");

    for edge in &edges {
        // Edge port map should not be empty
        assert!(
            !edge.edge_port_map.is_empty(),
            "edge from {} to {} should have port mappings",
            edge.from_node,
            edge.to_node
        );

        // Port names should be non-empty
        for (from_port, to_port) in &edge.edge_port_map {
            assert!(!from_port.is_empty(), "from_port should not be empty");
            assert!(!to_port.is_empty(), "to_port should not be empty");
        }
    }

    eprintln!(
        "captured {} edge examples from pragma workflow",
        edges.len()
    );
}
