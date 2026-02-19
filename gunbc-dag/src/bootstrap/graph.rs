//! DSL-backed graph builder for the bootstrap tool.

use crate::dsl_builder::build_bootstrap_graph_dsl;
use gunbc_exec::DynOp;
use gunbc_ir::{infer_signature, BuilderError, Dag, WorkflowSignature};

/// Runtime op type for bootstrap graphs.
pub type BootstrapGraphOp = DynOp;

/// Get the declared signature for the bootstrap workflow (auto-derived from DAG).
pub fn bootstrap_signature() -> WorkflowSignature {
    infer_signature(&build_bootstrap_graph().expect("bootstrap DAG should build for signature"))
}

/// Build bootstrap graph from the DSL source.
pub fn build_bootstrap_graph() -> Result<Dag<BootstrapGraphOp>, BuilderError> {
    build_bootstrap_graph_dsl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_bootstrap_graph_from_dsl() {
        let dag = build_bootstrap_graph().expect("bootstrap DSL graph should build");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn debug_bootstrap_node_structure() {
        let dag = build_bootstrap_graph().expect("bootstrap DSL graph should build");
        eprintln!("=== ORIGINAL DAG ===");
        for node in &dag.nodes {
            let is_subdag = matches!(&node.body, gunbc_ir::NodeBody::SubDag { .. });
            let outputs: Vec<_> = node.outputs.iter().map(|p| format!("{}:{}", p.name.0, p.type_id.0)).collect();
            eprintln!("  node={} subdag={} outputs={:?}", node.id.0, is_subdag, outputs);
        }
        let outgoing: std::collections::HashSet<&str> = dag.edges.iter().map(|e| e.from_node.0.as_str()).collect();
        eprintln!("=== NODES WITH OUTGOING EDGES (original: {} edges) ===", dag.edges.len());
        for n in &outgoing {
            eprintln!("  {n}");
        }
        let lowered = gunbc_exec::lower(&dag).expect("should lower");
        eprintln!("=== LOWERED EDGES: {} ===", lowered.dag.edges.len());
        for e in &lowered.dag.edges {
            eprintln!("  {} .{} -> {} .{}", e.from_node.0, e.from_port.0, e.to_node.0, e.to_port.0);
        }
        eprintln!("=== LOWERED DAG ===");
        for node in &lowered.dag.nodes {
            let outputs: Vec<_> = node.outputs.iter().map(|p| format!("{}:{}", p.name.0, p.type_id.0)).collect();
            eprintln!("  node={} outputs={:?}", node.id.0, outputs);
        }
        let lowered_outgoing: std::collections::HashSet<&str> = lowered.dag.edges.iter().map(|e| e.from_node.0.as_str()).collect();
        let lowered_nodes: std::collections::HashSet<&str> = lowered.dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
        eprintln!("=== LOWERED TERMINALS (no outgoing) ===");
        for node in &lowered.dag.nodes {
            if !lowered_outgoing.contains(node.id.0.as_str()) {
                let outputs: Vec<_> = node.outputs.iter().map(|p| format!("{}:{}", p.name.0, p.type_id.0)).collect();
                eprintln!("  node={} outputs={:?}", node.id.0, outputs);
            }
        }
        eprintln!("=== ORIGINAL-ONLY TERMINALS (not in lowered, no outgoing) ===");
        for node in &dag.nodes {
            if outgoing.contains(node.id.0.as_str()) { continue; }
            if lowered_nodes.contains(node.id.0.as_str()) { continue; }
            let outputs: Vec<_> = node.outputs.iter().map(|p| format!("{}:{}", p.name.0, p.type_id.0)).collect();
            eprintln!("  node={} outputs={:?}", node.id.0, outputs);
        }
    }
}
