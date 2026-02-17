//! daglang-derive: Derives ProgressManifest, TestObligations, and ToolMetadata.
//!
//! After lowering to GraphIR, the derive phase extracts higher-level
//! information needed by renderers, test generation, and tooling:
//!
//! - **ProgressManifest**: topology, waves, SubDag boundaries, parallel
//!   groups, scatter points, stage groups — used by all progress renderers
//! - **TestObligations**: 4-bucket test obligations derived from DAG structure
//!   and `@mock_response` / `@contract` annotations
//! - **ToolMetadata**: CLI entrypoints, Makefile targets, tool descriptions
//!
//! # Pipeline position
//!
//! ```text
//! Validated GraphIR → [daglang-derive] → ProgressManifest
//!                                      → TestObligations
//!                                      → ToolMetadata
//! ```

use std::collections::{BTreeMap, VecDeque};

use daglang_lower::{
    classify_obligation, classify_service_transport, LoweredOp, ObligationCategory,
    ServiceTransportClass,
};
use gunbc_ir::{detect_boundaries, detect_entrypoints, Dag, Node};
use serde::Serialize;

/// Derived artifacts produced from lowered GraphIR.
#[derive(Debug, Clone)]
pub struct DerivedArtifacts {
    pub manifest: ProgressManifest,
    pub obligations: TestObligations,
    pub tool_metadata: ToolMetadata,
}

/// Progress-manifest contract derived from lowered DAG topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProgressManifest {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub waves: Vec<Vec<String>>,
    pub entrypoint_nodes: Vec<String>,
    pub boundary_nodes: Vec<String>,
    pub topology: Vec<TopologyNode>,
    pub labels: BTreeMap<String, String>,
    pub subdag_boundaries: Vec<SubDagBoundary>,
    pub parallel_groups: Vec<ParallelGroup>,
    pub scatter_points: Vec<String>,
    pub interactive_nodes: Vec<String>,
    pub capture_modes: BTreeMap<String, CaptureMode>,
    pub stage_groups: Vec<StageGroup>,
    pub resources: BTreeMap<String, Vec<ResourceUsage>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TopologyNode {
    pub node_id: String,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubDagBoundary {
    pub node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParallelGroup {
    pub group_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode {
    #[default]
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StageGroup {
    pub stage: String,
    pub node_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResourceUsage {
    pub resource: String,
    pub usage: String,
}

/// Minimal obligation summary derived from graph structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestObligations {
    pub dry_run_completion_required: bool,
    pub transport_execution_targets: usize,
    pub pure_node_determinism_targets: usize,
    pub service_transport_prepare_targets: usize,
    pub service_transport_execute_targets: usize,
    pub service_transport_parse_targets: usize,
    pub service_transport_hermetic_targets: usize,
    pub service_transport_external_targets: usize,
    pub service_transport_idempotent_targets: usize,
    pub service_transport_readonly_targets: usize,
    pub service_transport_permission_scoped_targets: usize,
    pub service_param_source_targets: usize,
    pub resource_provide_targets: usize,
    pub resource_acquire_targets: usize,
    pub resource_release_targets: usize,
}

/// Metadata summary for lowered modules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolMetadata {
    pub modules: Vec<ModuleMetadata>,
}

/// Module-level metadata counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleMetadata {
    pub module: String,
    pub callable_count: usize,
    pub pipeline_count: usize,
}

/// Errors during derivation.
#[derive(Debug)]
pub enum DeriveError {
    /// The IR graph is not valid for manifest derivation.
    InvalidGraph(String),
}

impl std::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidGraph(reason) => write!(f, "invalid graph for derivation: {reason}"),
        }
    }
}

/// Derive manifest, obligations, and metadata from a lowered daglang graph.
pub fn derive_artifacts(dag: &Dag<LoweredOp>) -> Result<DerivedArtifacts, DeriveError> {
    if dag.nodes.is_empty() {
        return Err(DeriveError::InvalidGraph(
            "cannot derive artifacts for an empty graph".to_string(),
        ));
    }

    let waves = compute_waves(dag)?;
    let entrypoint_nodes = {
        let mut nodes = detect_entrypoints(dag)
            .entrypoint_nodes
            .into_iter()
            .map(|node_id| node_id.0)
            .collect::<Vec<_>>();
        nodes.sort();
        nodes
    };
    let boundary_nodes = {
        let mut nodes = detect_boundaries(dag)
            .boundary_nodes
            .into_iter()
            .map(|node_id| node_id.0)
            .collect::<Vec<_>>();
        nodes.sort();
        nodes
    };
    let manifest = ProgressManifest {
        total_nodes: dag.nodes.len(),
        total_edges: dag.edges.len(),
        topology: derive_topology_from_waves(&waves),
        labels: derive_node_labels(&dag.nodes),
        subdag_boundaries: boundary_nodes
            .iter()
            .cloned()
            .map(|node_id| SubDagBoundary { node_id })
            .collect(),
        parallel_groups: derive_parallel_groups(&waves),
        scatter_points: Vec::new(),
        interactive_nodes: Vec::new(),
        capture_modes: derive_capture_modes(&dag.nodes),
        stage_groups: Vec::new(),
        resources: BTreeMap::new(),
        waves,
        entrypoint_nodes,
        boundary_nodes,
    };

    let obligation_counts = derive_obligation_counts(&dag.nodes);
    let obligations = TestObligations {
        dry_run_completion_required: true,
        transport_execution_targets: obligation_counts.transport_execution_targets,
        pure_node_determinism_targets: obligation_counts.pure_node_determinism_targets,
        service_transport_prepare_targets: obligation_counts.service_transport_prepare_targets,
        service_transport_execute_targets: obligation_counts.service_transport_execute_targets,
        service_transport_parse_targets: obligation_counts.service_transport_parse_targets,
        service_transport_hermetic_targets: obligation_counts.service_transport_hermetic_targets,
        service_transport_external_targets: obligation_counts.service_transport_external_targets,
        service_transport_idempotent_targets: obligation_counts
            .service_transport_idempotent_targets,
        service_transport_readonly_targets: obligation_counts.service_transport_readonly_targets,
        service_transport_permission_scoped_targets: obligation_counts
            .service_transport_permission_scoped_targets,
        service_param_source_targets: obligation_counts.service_param_source_targets,
        resource_provide_targets: obligation_counts.resource_provide_targets,
        resource_acquire_targets: obligation_counts.resource_acquire_targets,
        resource_release_targets: obligation_counts.resource_release_targets,
    };

    let tool_metadata = ToolMetadata {
        modules: derive_module_metadata(&dag.nodes),
    };

    Ok(DerivedArtifacts {
        manifest,
        obligations,
        tool_metadata,
    })
}

fn compute_waves(dag: &Dag<LoweredOp>) -> Result<Vec<Vec<String>>, DeriveError> {
    let mut indegree = BTreeMap::<String, usize>::new();
    let mut outgoing = BTreeMap::<String, Vec<String>>::new();

    for node in &dag.nodes {
        indegree.insert(node.id.0.clone(), 0);
        outgoing.insert(node.id.0.clone(), Vec::new());
    }

    for edge in &dag.edges {
        let to = edge.to_node.0.clone();
        let from = edge.from_node.0.clone();
        let degree = indegree.get_mut(&to).ok_or_else(|| {
            DeriveError::InvalidGraph(format!("edge targets missing node `{to}`"))
        })?;
        *degree += 1;
        outgoing
            .get_mut(&from)
            .ok_or_else(|| DeriveError::InvalidGraph(format!("edge source missing node `{from}`")))?
            .push(to);
    }

    let mut current_wave = indegree
        .iter()
        .filter_map(|(node, degree)| (*degree == 0).then_some(node.clone()))
        .collect::<Vec<_>>();
    current_wave.sort();

    if current_wave.is_empty() {
        return Err(DeriveError::InvalidGraph(
            "graph has no entrypoint wave (likely cyclic)".to_string(),
        ));
    }

    let mut waves = Vec::new();
    let mut processed = 0usize;

    while !current_wave.is_empty() {
        waves.push(current_wave.clone());
        processed += current_wave.len();
        let mut next_wave = Vec::new();

        for node in &current_wave {
            let mut queue = outgoing
                .get(node)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect::<VecDeque<_>>();
            while let Some(target) = queue.pop_front() {
                let degree = indegree.get_mut(&target).ok_or_else(|| {
                    DeriveError::InvalidGraph(format!("missing indegree for `{target}`"))
                })?;
                *degree = degree.saturating_sub(1);
                if *degree == 0 {
                    next_wave.push(target);
                }
            }
        }

        next_wave.sort();
        next_wave.dedup();
        current_wave = next_wave;
    }

    if processed != dag.nodes.len() {
        return Err(DeriveError::InvalidGraph(
            "graph contains a cycle and cannot be wave-partitioned".to_string(),
        ));
    }

    Ok(waves)
}

fn derive_topology_from_waves(waves: &[Vec<String>]) -> Vec<TopologyNode> {
    let mut topology = Vec::new();
    for (depth, wave) in waves.iter().enumerate() {
        for node_id in wave {
            topology.push(TopologyNode {
                node_id: node_id.clone(),
                depth,
            });
        }
    }
    topology
}

fn derive_parallel_groups(waves: &[Vec<String>]) -> Vec<ParallelGroup> {
    waves
        .iter()
        .enumerate()
        .filter(|(_depth, wave)| !wave.is_empty())
        .map(|(depth, wave)| ParallelGroup {
            group_id: format!("wave_{depth}"),
            node_ids: wave.clone(),
        })
        .collect()
}

fn derive_node_labels(nodes: &[Node<LoweredOp>]) -> BTreeMap<String, String> {
    nodes
        .iter()
        .map(|node| {
            let label = match &node.body {
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Callable { module, name, .. }) => {
                    format!("{module}.{name}")
                }
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pipeline { module, name, .. }) => {
                    format!("{module}.{name}")
                }
                gunbc_ir::node::NodeBody::SubDag(_) => "subdag".to_string(),
            };
            (node.id.0.clone(), label)
        })
        .collect()
}

fn derive_capture_modes(nodes: &[Node<LoweredOp>]) -> BTreeMap<String, CaptureMode> {
    nodes
        .iter()
        .map(|node| (node.id.0.clone(), CaptureMode::Default))
        .collect()
}

fn derive_module_metadata(nodes: &[Node<LoweredOp>]) -> Vec<ModuleMetadata> {
    let mut by_module: BTreeMap<String, ModuleMetadata> = BTreeMap::new();
    for node in nodes {
        let Some(op) = node.body.as_opaque() else {
            continue;
        };
        match op {
            LoweredOp::Callable { module, .. } => {
                let entry = by_module
                    .entry(module.clone())
                    .or_insert_with(|| ModuleMetadata {
                        module: module.clone(),
                        callable_count: 0,
                        pipeline_count: 0,
                    });
                entry.callable_count += 1;
            }
            LoweredOp::Pipeline { module, .. } => {
                let entry = by_module
                    .entry(module.clone())
                    .or_insert_with(|| ModuleMetadata {
                        module: module.clone(),
                        callable_count: 0,
                        pipeline_count: 0,
                    });
                entry.pipeline_count += 1;
            }
        }
    }
    by_module.into_values().collect()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ObligationCounts {
    transport_execution_targets: usize,
    pure_node_determinism_targets: usize,
    service_transport_prepare_targets: usize,
    service_transport_execute_targets: usize,
    service_transport_parse_targets: usize,
    service_transport_hermetic_targets: usize,
    service_transport_external_targets: usize,
    service_transport_idempotent_targets: usize,
    service_transport_readonly_targets: usize,
    service_transport_permission_scoped_targets: usize,
    service_param_source_targets: usize,
    resource_provide_targets: usize,
    resource_acquire_targets: usize,
    resource_release_targets: usize,
}

fn derive_obligation_counts(nodes: &[Node<LoweredOp>]) -> ObligationCounts {
    let mut counts = ObligationCounts::default();
    for node in nodes {
        if node
            .inputs
            .iter()
            .any(|port| port.type_id.0 == "TransportRequest")
        {
            counts.transport_execution_targets += 1;
        } else {
            counts.pure_node_determinism_targets += 1;
        }
        let Some(op) = node.body.as_opaque() else {
            continue;
        };
        match classify_obligation(op) {
            ObligationCategory::ServiceTransportPrepare => {
                counts.service_transport_prepare_targets += 1;
            }
            ObligationCategory::ServiceTransportExecute => {
                counts.service_transport_execute_targets += 1;
                match classify_service_transport(op) {
                    Some(ServiceTransportClass::ShellLocal)
                        if op
                            .service_call_metadata()
                            .is_some_and(|metadata| metadata.permissions.is_empty()) =>
                    {
                        counts.service_transport_hermetic_targets += 1;
                    }
                    Some(ServiceTransportClass::ShellLocal)
                    | Some(ServiceTransportClass::RestNetwork)
                    | Some(ServiceTransportClass::FileBoundary)
                    | Some(ServiceTransportClass::Unknown)
                    | None => {
                        counts.service_transport_external_targets += 1;
                    }
                }
                if op
                    .service_call_metadata()
                    .is_some_and(|metadata| metadata.idempotent)
                {
                    counts.service_transport_idempotent_targets += 1;
                }
                if op
                    .service_call_metadata()
                    .is_some_and(|metadata| metadata.readonly)
                {
                    counts.service_transport_readonly_targets += 1;
                }
                if op
                    .service_call_metadata()
                    .is_some_and(|metadata| !metadata.permissions.is_empty())
                {
                    counts.service_transport_permission_scoped_targets += 1;
                }
            }
            ObligationCategory::ServiceTransportParse => {
                counts.service_transport_parse_targets += 1;
            }
            ObligationCategory::ServiceParamSource => {
                counts.service_param_source_targets += 1;
            }
            ObligationCategory::ResourceProvide => {
                counts.resource_provide_targets += 1;
            }
            ObligationCategory::ResourceAcquire => {
                counts.resource_acquire_targets += 1;
            }
            ObligationCategory::ResourceRelease => {
                counts.resource_release_targets += 1;
            }
            ObligationCategory::None => {}
        }
    }
    counts
}

trait NodeBodyExt {
    fn as_opaque(&self) -> Option<&LoweredOp>;
}

impl NodeBodyExt for gunbc_ir::node::NodeBody<LoweredOp> {
    fn as_opaque(&self) -> Option<&LoweredOp> {
        match self {
            gunbc_ir::node::NodeBody::Opaque(op) => Some(op),
            gunbc_ir::node::NodeBody::SubDag(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::{
        CallableKind, LoweredOp, ObligationCategory, ServiceCallMetadata, ServiceTransportClass,
    };
    use gunbc_ir::{Edge, Node, Port};

    fn callable_node(id: &str, module: &str, name: &str) -> Node<LoweredOp> {
        Node::opaque(
            id,
            vec![Port::scalar("input", "String")],
            vec![Port::scalar("output", "String")],
            LoweredOp::Callable {
                module: module.to_string(),
                kind: CallableKind::Func,
                name: name.to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        )
    }

    #[test]
    fn derive_manifest_builds_stable_waves() {
        let mut dag = Dag::new();
        dag.add_node(callable_node("a", "tools.makegen", "a"));
        dag.add_node(callable_node("b", "tools.makegen", "b"));
        dag.add_node(callable_node("c", "tools.makegen", "c"));
        dag.add_edge(Edge::new("a", "output", "c", "input"));
        dag.add_edge(Edge::new("b", "output", "c", "input"));

        let artifacts = derive_artifacts(&dag).expect("derivation should succeed");
        assert_eq!(
            artifacts.manifest.waves,
            vec![
                vec!["a".to_string(), "b".to_string()],
                vec!["c".to_string()]
            ]
        );
        assert_eq!(artifacts.manifest.total_nodes, 3);
        assert_eq!(artifacts.manifest.total_edges, 2);
        assert_eq!(
            artifacts.manifest.topology,
            vec![
                TopologyNode {
                    node_id: "a".to_string(),
                    depth: 0,
                },
                TopologyNode {
                    node_id: "b".to_string(),
                    depth: 0,
                },
                TopologyNode {
                    node_id: "c".to_string(),
                    depth: 1,
                },
            ]
        );
        assert_eq!(
            artifacts
                .manifest
                .labels
                .get("a")
                .expect("label should exist for node a"),
            "tools.makegen.a"
        );
        assert_eq!(
            artifacts.manifest.parallel_groups,
            vec![
                ParallelGroup {
                    group_id: "wave_0".to_string(),
                    node_ids: vec!["a".to_string(), "b".to_string()],
                },
                ParallelGroup {
                    group_id: "wave_1".to_string(),
                    node_ids: vec!["c".to_string()],
                },
            ]
        );
        assert_eq!(
            artifacts.manifest.capture_modes.len(),
            3,
            "capture modes should be derived for all nodes"
        );
        let expected_boundaries = artifacts
            .manifest
            .boundary_nodes
            .iter()
            .cloned()
            .map(|node_id| SubDagBoundary { node_id })
            .collect::<Vec<_>>();
        assert_eq!(artifacts.manifest.subdag_boundaries, expected_boundaries);
    }

    #[test]
    fn derive_module_metadata_counts_callable_and_pipeline_nodes() {
        let mut dag = Dag::new();
        dag.add_node(callable_node("makegen", "tools.makegen", "makegen"));
        dag.add_node(Node::opaque(
            "ci",
            vec![],
            vec![Port::scalar("stages", "Int")],
            LoweredOp::Pipeline {
                module: "pipelines.ci".to_string(),
                name: "ci".to_string(),
                stages: 12,
            },
        ));

        let artifacts = derive_artifacts(&dag).expect("derivation should succeed");
        assert!(artifacts
            .tool_metadata
            .modules
            .iter()
            .any(|module| module.module == "tools.makegen" && module.callable_count == 1));
        assert!(artifacts
            .tool_metadata
            .modules
            .iter()
            .any(|module| module.module == "pipelines.ci" && module.pipeline_count == 1));
    }

    #[test]
    fn derive_obligations_count_transport_and_lifecycle_targets() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "param_source",
            vec![],
            vec![Port::scalar("path", "String")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "call_param_source::run::path".to_string(),
                obligation: ObligationCategory::ServiceParamSource,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "prepare_transport",
            vec![Port::scalar("path", "String")],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::prepare::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportPrepare,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "execute_transport",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::execute::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportExecute,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "parse_transport",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("body", "String")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::parse::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportParse,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "provide_resource",
            vec![Port::scalar("trigger", "Any")],
            vec![Port::scalar("out", "Storage")],
            LoweredOp::Callable {
                module: "sample.resources".to_string(),
                kind: CallableKind::Pattern,
                name: "resource_provide::run::out".to_string(),
                obligation: ObligationCategory::ResourceProvide,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "acquire_resource",
            vec![],
            vec![Port::scalar("resource_handle", "ResourceHandle")],
            LoweredOp::Callable {
                module: "sample.resources".to_string(),
                kind: CallableKind::Pattern,
                name: "resource_lifecycle::acquire::TempFile".to_string(),
                obligation: ObligationCategory::ResourceAcquire,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "release_resource",
            vec![Port::scalar("resource_handle", "ResourceHandle")],
            vec![Port::scalar("released", "Bool")],
            LoweredOp::Callable {
                module: "sample.resources".to_string(),
                kind: CallableKind::Pattern,
                name: "resource_lifecycle::release::TempFile".to_string(),
                obligation: ObligationCategory::ResourceRelease,
                service_metadata: None,
            },
        ));
        dag.add_edge(Edge::new(
            "param_source",
            "path",
            "prepare_transport",
            "path",
        ));
        dag.add_edge(Edge::new(
            "prepare_transport",
            "request",
            "execute_transport",
            "request",
        ));
        dag.add_edge(Edge::new(
            "execute_transport",
            "response",
            "parse_transport",
            "response",
        ));
        dag.add_edge(Edge::new(
            "provide_resource",
            "out",
            "release_resource",
            "resource_handle",
        ));
        dag.add_edge(Edge::new(
            "acquire_resource",
            "resource_handle",
            "release_resource",
            "resource_handle",
        ));

        let artifacts = derive_artifacts(&dag).expect("derivation should succeed");
        assert_eq!(artifacts.obligations.transport_execution_targets, 1);
        assert_eq!(artifacts.obligations.pure_node_determinism_targets, 6);
        assert_eq!(artifacts.obligations.service_transport_prepare_targets, 1);
        assert_eq!(artifacts.obligations.service_transport_execute_targets, 1);
        assert_eq!(artifacts.obligations.service_transport_parse_targets, 1);
        assert_eq!(artifacts.obligations.service_transport_hermetic_targets, 0);
        assert_eq!(artifacts.obligations.service_transport_external_targets, 1);
        assert_eq!(
            artifacts.obligations.service_transport_idempotent_targets,
            0
        );
        assert_eq!(artifacts.obligations.service_transport_readonly_targets, 0);
        assert_eq!(
            artifacts
                .obligations
                .service_transport_permission_scoped_targets,
            0
        );
        assert_eq!(artifacts.obligations.service_param_source_targets, 1);
        assert_eq!(artifacts.obligations.resource_provide_targets, 1);
        assert_eq!(artifacts.obligations.resource_acquire_targets, 1);
        assert_eq!(artifacts.obligations.resource_release_targets, 1);
    }

    #[test]
    fn derive_obligations_tracks_service_transport_semantic_buckets() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "execute_transport_hermetic",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::execute::FsStorage::read".to_string(),
                obligation: ObligationCategory::ServiceTransportExecute,
                service_metadata: Some(ServiceCallMetadata {
                    service: "FsStorage".to_string(),
                    operation: "read".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: true,
                    readonly: true,
                    permissions: vec![],
                }),
            },
        ));
        dag.add_node(Node::opaque(
            "execute_transport_external",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::execute::GistApi::create".to_string(),
                obligation: ObligationCategory::ServiceTransportExecute,
                service_metadata: Some(ServiceCallMetadata {
                    service: "GistApi".to_string(),
                    operation: "create".to_string(),
                    transport: ServiceTransportClass::RestNetwork,
                    idempotent: false,
                    readonly: false,
                    permissions: vec!["gist.write".to_string()],
                }),
            },
        ));

        let artifacts = derive_artifacts(&dag).expect("derivation should succeed");
        assert_eq!(artifacts.obligations.service_transport_execute_targets, 2);
        assert_eq!(artifacts.obligations.service_transport_hermetic_targets, 1);
        assert_eq!(artifacts.obligations.service_transport_external_targets, 1);
        assert_eq!(
            artifacts.obligations.service_transport_idempotent_targets,
            1
        );
        assert_eq!(artifacts.obligations.service_transport_readonly_targets, 1);
        assert_eq!(
            artifacts
                .obligations
                .service_transport_permission_scoped_targets,
            1
        );
    }

    #[test]
    fn derive_obligations_uses_structural_category_not_callable_name_prefix() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "misleading_name",
            vec![],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "service_transport::prepare::Fake::op".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
            },
        ));
        dag.add_node(Node::opaque(
            "structural_category",
            vec![],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "sample.services".to_string(),
                kind: CallableKind::Pattern,
                name: "not_a_transport_name".to_string(),
                obligation: ObligationCategory::ServiceTransportPrepare,
                service_metadata: None,
            },
        ));

        let artifacts = derive_artifacts(&dag).expect("derivation should succeed");
        assert_eq!(
            artifacts.obligations.service_transport_prepare_targets, 1,
            "classification should follow structural obligation category"
        );
    }
}
