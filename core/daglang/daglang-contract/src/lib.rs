use std::collections::BTreeMap;

use serde::Serialize;

/// Progress-manifest contract derived from lowered DAG topology.
///
/// The stable JSON contract includes: `total_nodes`, `topology`, `labels`,
/// `subdag_boundaries`, `parallel_groups`, `scatter_points`, `interactive_nodes`,
/// `capture_modes`, `stage_groups`, `resources`.
///
/// Fields marked `skip_serializing` (`total_edges`, `waves`, `entrypoint_nodes`,
/// `boundary_nodes`) are used internally by text renderers and emit but are not
/// part of the stable JSON contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProgressManifest {
    pub total_nodes: usize,
    #[serde(skip_serializing)]
    pub total_edges: usize,
    #[serde(skip_serializing)]
    pub waves: Vec<Vec<String>>,
    #[serde(skip_serializing)]
    pub entrypoint_nodes: Vec<String>,
    #[serde(skip_serializing)]
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
    pub id: String,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubDagBoundary {
    pub node_id: String,
    pub label: String,
    pub inner_nodes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParallelGroup {
    pub nodes: Vec<String>,
    pub depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_subdag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode {
    #[default]
    Captured,
    Passthrough,
    Streamed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StageGroup {
    pub stage_id: String,
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResourceUsage {
    pub resource: String,
    pub usage: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestObligations {
    pub dry_run_completion_required: bool,
    pub total_obligations: usize,
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
    pub interface_contract_verification_targets: usize,
}
