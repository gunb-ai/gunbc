//! Shared helpers for DSL-backed graph builders (T3).

use daglang_driver::{compile_from_context, DriverContext};
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag, WorkspaceLayout};
use std::collections::HashSet;

use crate::resolve_lowered_dag;

fn workspace_layout() -> Result<WorkspaceLayout, BuilderError> {
    WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|error| {
            BuilderError::InternalInvariant(format!(
                "failed to resolve workspace layout for DSL builder: {error}"
            ))
        })
}

fn compile_lowered(relative_module: &str) -> Result<Dag<daglang_lower::LoweredOp>, BuilderError> {
    let layout = workspace_layout()?;
    let dsl_root = layout.workspace_root.join("dsl");
    let target_file = dsl_root.join(relative_module);

    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(target_file),
    };

    let output = compile_from_context(&context).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to compile DSL module `{relative_module}`: {error}"
        ))
    })?;
    Ok(output.lowered_dag)
}

fn strip_pipeline_nodes(mut dag: Dag<daglang_lower::LoweredOp>) -> Dag<daglang_lower::LoweredOp> {
    let pipeline_ids: HashSet<String> = dag
        .nodes
        .iter()
        .filter_map(|node| match &node.body {
            gunbc_ir::node::NodeBody::Opaque(daglang_lower::LoweredOp::Pipeline { .. }) => {
                Some(node.id.0.clone())
            }
            _ => None,
        })
        .collect();

    if pipeline_ids.is_empty() {
        return dag;
    }

    dag.nodes.retain(|node| !pipeline_ids.contains(&node.id.0));
    dag.edges.retain(|edge| {
        !pipeline_ids.contains(&edge.from_node.0) && !pipeline_ids.contains(&edge.to_node.0)
    });
    dag
}

fn slice_dag_from_entry<T>(mut dag: Dag<T>, entry_node_id: &str) -> Result<Dag<T>, BuilderError> {
    if !dag.nodes.iter().any(|node| node.id.0 == entry_node_id) {
        return Err(BuilderError::InternalInvariant(format!(
            "entry node `{entry_node_id}` not found in compiled DAG"
        )));
    }

    let mut include = HashSet::<String>::new();
    let mut backward_stack = vec![entry_node_id.to_string()];
    while let Some(node_id) = backward_stack.pop() {
        if !include.insert(node_id.clone()) {
            continue;
        }
        for edge in &dag.edges {
            if edge.to_node.0 == node_id {
                backward_stack.push(edge.from_node.0.clone());
            }
        }
    }

    let mut forward_seen = HashSet::<String>::new();
    let mut forward_stack = vec![entry_node_id.to_string()];
    while let Some(node_id) = forward_stack.pop() {
        if !forward_seen.insert(node_id.clone()) {
            continue;
        }
        include.insert(node_id.clone());
        for edge in &dag.edges {
            if edge.from_node.0 == node_id {
                forward_stack.push(edge.to_node.0.clone());
            }
        }
    }

    dag.nodes.retain(|node| include.contains(&node.id.0));
    dag.edges.retain(|edge| {
        include.contains(&edge.from_node.0) && include.contains(&edge.to_node.0)
    });
    Ok(dag)
}

/// Compile a DSL module and resolve lowered ops into `Dag<DynOp>`.
pub(crate) fn build_dsl_graph(relative_module: &str) -> Result<Dag<DynOp>, BuilderError> {
    let lowered = strip_pipeline_nodes(compile_lowered(relative_module)?);
    resolve_lowered_dag(&lowered).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to resolve lowered DAG for `{relative_module}`: {error}"
        ))
    })
}

fn build_dsl_graph_for_entry(
    relative_module: &str,
    entry_node_id: &str,
) -> Result<Dag<DynOp>, BuilderError> {
    let lowered = strip_pipeline_nodes(compile_lowered(relative_module)?);
    let lowered = slice_dag_from_entry(lowered, entry_node_id)?;
    resolve_lowered_dag(&lowered).map_err(|error| {
        BuilderError::InternalInvariant(format!(
            "failed to resolve lowered DAG for `{relative_module}` entry `{entry_node_id}`: {error}"
        ))
    })
}

pub(crate) fn build_bootstrap_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/bootstrap.dag")
}

pub(crate) fn build_build_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/build.dag")
}

pub(crate) fn build_ci_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("pipelines/ci.dag")
}

pub(crate) fn build_codegen_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/codegen.dag")
}

pub(crate) fn build_docgen_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/docgen.dag")
}

pub(crate) fn build_infra_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/infra.dag")
}

pub(crate) fn build_makegen_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph_for_entry("tools/makegen.dag", "tools.makegen::makegen")
}

pub(crate) fn build_pragma_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph_for_entry("tools/pragma.dag", "tools.pragma::pragma")
}

pub(crate) fn build_deps_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/deps.dag")
}

pub fn build_clippy_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/clippy.dag")
}

pub fn build_aws_credential_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("cloud/aws/credential.dag")
}

pub fn build_azure_credential_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("cloud/azure/credential.dag")
}

pub fn build_review_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("tools/review.dag")
}

pub fn build_dimension_review_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph("funcs/review_pipeline.dag")
}

pub fn build_gist_snapshot_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph_for_entry("tools/gist.dag", "tools.gist::gist_snapshot")
}

pub fn build_gist_diff_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph_for_entry("tools/gist.dag", "tools.gist::gist_diff")
}

pub fn build_gist_recent_graph_dsl() -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph_for_entry("tools/gist.dag", "tools.gist::gist_recent")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_makegen_dsl_graph() {
        let dag = build_makegen_graph_dsl().expect("makegen DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_pragma_dsl_graph() {
        let dag = build_pragma_graph_dsl().expect("pragma DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_bootstrap_dsl_graph() {
        let dag = build_bootstrap_graph_dsl().expect("bootstrap DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_build_dsl_graph() {
        let dag = build_build_graph_dsl().expect("build DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_codegen_dsl_graph() {
        let dag = build_codegen_graph_dsl().expect("codegen DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_docgen_dsl_graph() {
        let dag = build_docgen_graph_dsl().expect("docgen DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_infra_dsl_graph() {
        let dag = build_infra_graph_dsl().expect("infra DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_clippy_dsl_graph() {
        let dag = build_dsl_graph("tools/clippy.dag").expect("clippy DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_deps_dsl_graph() {
        let dag = build_deps_graph_dsl().expect("deps DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_review_dsl_graph() {
        let dag = build_dsl_graph("tools/review.dag").expect("review DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_dimension_review_dsl_graph() {
        let dag = build_dimension_review_graph_dsl()
            .expect("dimension review DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn builds_gist_dsl_graph() {
        let dag = build_gist_snapshot_graph_dsl().expect("gist DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
    }

    #[test]
    fn diagnostic_gist_dag_structure() {
        let dag = build_gist_snapshot_graph_dsl().expect("gist DSL graph should resolve");
        let boundaries = gunbc_ir::detect_boundaries(&dag);

        println!("\n=== GIST DAG NODES ({}) ===", dag.nodes.len());
        for node in &dag.nodes {
            let outputs: Vec<String> = node
                .outputs
                .iter()
                .map(|p| format!("{}:{}", p.name.0, &p.type_id.0))
                .collect();
            println!("  {} → [{}]", node.id.0, outputs.join(", "));
        }

        println!("\n=== BOUNDARY PORTS ===");
        for (node_id, port_name) in &boundaries.boundary_ports {
            println!("  {}.{}", node_id.0, port_name.0);
        }

        println!("\n=== EDGES TO/FROM PARSE NODE ===");
        for edge in &dag.edges {
            if edge.from_node.0.contains("parse_transport_services_github_gist")
                || edge.to_node.0.contains("parse_transport_services_github_gist")
            {
                println!(
                    "  {}.{} → {}.{}",
                    edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
                );
            }
        }

        // Verify parse node has url output
        let parse_node = dag
            .nodes
            .iter()
            .find(|n| n.id.0.contains("parse_transport_services_github_gist"));
        assert!(parse_node.is_some(), "parse node should exist");
        let parse_node = parse_node.unwrap();
        let url_port = parse_node.outputs.iter().find(|p| p.name.0 == "url");
        assert!(url_port.is_some(), "parse node should have url output port");

        // Inspect the DynOp on the parse node to see the RestOperationSpec
        println!("\n=== PARSE NODE OP DEBUG ===");
        println!("  {:?}", parse_node.body);

        // Check edges TO the prepare node
        println!("\n=== EDGES TO PREPARE NODE ===");
        for edge in &dag.edges {
            if edge.to_node.0.contains("prepare_transport_services_github_gist") {
                println!(
                    "  {}.{} → {}.{}",
                    edge.from_node.0, edge.from_port.0, edge.to_node.0, edge.to_port.0
                );
            }
        }
    }

    #[test]
    fn builds_ci_dsl_graph() {
        let dag = build_ci_graph_dsl().expect("ci DSL graph should resolve");
        assert!(!dag.nodes.is_empty());
        assert!(
            !dag.nodes.iter().any(|node| node.id.0 == "pipelines.ci::ci"),
            "runtime CI graph should not include pipeline metadata nodes"
        );
    }
}
