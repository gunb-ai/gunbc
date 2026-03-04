//! daglang-derive: Derives ProgressManifest, TestObligations, TransportTriplets,
//! and ToolMetadata.
//!
//! After lowering to GraphIR, the derive phase extracts higher-level
//! information needed by renderers, test generation, and tooling:
//!
//! - **ProgressManifest**: topology, waves, SubDag boundaries, parallel
//!   groups, scatter points, stage groups — used by all progress renderers
//! - **TestObligations**: 4-bucket test obligations derived from DAG structure
//!   and `mock_response` / `contract` declarations
//! - **TransportTriplets**: prepare→execute→parse transport chains with metadata
//! - **ToolMetadata**: CLI entrypoints, Makefile targets, tool descriptions
//!
//! # Pipeline position
//!
//! ```text
//! Validated GraphIR → [daglang-derive] → ProgressManifest
//!                                      → TestObligations
//!                                      → ToolMetadata
//! ```

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

pub use daglang_contract::{
    CaptureMode, ParallelGroup, ProgressManifest, ResourceUsage, StageGroup, SubDagBoundary,
    TestObligations, TopologyNode,
};
use daglang_lower::{
    classify_obligation, classify_service_transport, CallableKind, CollectionOpKind, LoweredOp,
    ObligationCategory, ServiceCallMetadata, ServiceTransportClass,
};
use gunbc_ir::{detect_boundaries, detect_entrypoints, Dag, Node, NodeKind};

/// Derived artifacts produced from lowered GraphIR.
#[derive(Debug, Clone)]
pub struct DerivedArtifacts {
    pub manifest: ProgressManifest,
    pub obligations: TestObligations,
    pub transport_triplets: Vec<TransportTriplet>,
    pub tool_metadata: ToolMetadata,
    /// Per-callable structural properties derived from graph traversal.
    /// Key is callable name (e.g., "tools.makegen::render_makefile").
    pub callable_properties: BTreeMap<String, CallableProperties>,
}

/// Structural properties of a callable, extracted by graph walk.
///
/// These are raw facts about what transports, permissions, and behavioral
/// flags a callable transitively reaches. Classification into test tiers
/// is the DSL's job (via `config/test_policy.dag`), not ours.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CallableProperties {
    pub transport_classes: Vec<ServiceTransportClass>,
    pub permissions: Vec<String>,
    pub idempotent: bool,
    pub readonly: bool,
    pub service_operations: Vec<(String, String)>,
}

/// A discovered prepare→execute→parse transport triplet.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TransportTriplet {
    pub prepare_node: String,
    pub execute_node: String,
    pub parse_nodes: Vec<String>,
    pub service_metadata: Option<ServiceCallMetadata>,
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
        schema_version: 1,
        total_nodes: dag.nodes.len(),
        total_edges: dag.edges.len(),
        topology: derive_topology_from_waves(&waves),
        labels: derive_node_labels(&dag.nodes),
        subdag_boundaries: derive_subdag_boundaries(&dag.nodes),
        parallel_groups: derive_parallel_groups(&waves),
        scatter_points: derive_scatter_points(&dag.nodes),
        interactive_nodes: derive_interactive_nodes(&dag.nodes),
        capture_modes: derive_capture_modes(&dag.nodes),
        stage_groups: derive_stage_groups(&dag.nodes),
        resources: derive_resources(&dag.nodes),
        waves,
        entrypoint_nodes,
        boundary_nodes,
    };

    let obligation_counts = derive_obligation_counts(&dag.nodes);
    let obligations = TestObligations {
        dry_run_completion_required: true,
        total_obligations: obligation_counts.transport_execution_targets
            + obligation_counts.pure_node_determinism_targets,
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
        interface_contract_verification_targets: obligation_counts
            .interface_contract_verification_targets,
    };

    let tool_metadata = ToolMetadata {
        modules: derive_module_metadata(&dag.nodes),
    };
    let transport_triplets = derive_transport_triplets(dag);
    let callable_properties = derive_callable_properties(dag);

    Ok(DerivedArtifacts {
        manifest,
        obligations,
        transport_triplets,
        tool_metadata,
        callable_properties,
    })
}

/// Derive transport triplets by following TransportRequest/TransportResponse edges.
pub fn derive_transport_triplets(dag: &Dag<LoweredOp>) -> Vec<TransportTriplet> {
    let node_by_id = dag
        .nodes
        .iter()
        .map(|node| (node.id.0.as_str(), node))
        .collect::<HashMap<_, _>>();
    // Use Vec with manual dedup on (prepare, execute, parse) keys.
    // We avoid BTreeSet since ServiceCallMetadata doesn't implement Ord (TL-12).
    let mut triplets = Vec::<TransportTriplet>::new();
    let mut seen_keys = std::collections::HashSet::<(String, String, Vec<String>)>::new();

    for edge in &dag.edges {
        let Some(prepare_node) = node_by_id.get(edge.from_node.0.as_str()).copied() else {
            continue;
        };
        let Some(execute_node) = node_by_id.get(edge.to_node.0.as_str()).copied() else {
            continue;
        };
        if node_output_port_type(prepare_node, edge.from_port.0.as_str())
            != Some("TransportRequest")
            || node_input_port_type(execute_node, edge.to_port.0.as_str())
                != Some("TransportRequest")
        {
            continue;
        }

        let mut parse_nodes = dag
            .edges
            .iter()
            .filter(|next_edge| next_edge.from_node.0 == edge.to_node.0)
            .filter_map(|next_edge| {
                let parse_node = node_by_id.get(next_edge.to_node.0.as_str()).copied()?;
                (node_output_port_type(execute_node, next_edge.from_port.0.as_str())
                    == Some("TransportResponse")
                    && node_input_port_type(parse_node, next_edge.to_port.0.as_str())
                        == Some("TransportResponse"))
                .then_some(next_edge.to_node.0.clone())
            })
            .collect::<Vec<_>>();
        parse_nodes.sort();
        parse_nodes.dedup();
        let service_metadata = match &execute_node.body {
            gunbc_ir::node::NodeBody::Opaque(op) => op.service_call_metadata().cloned(),
            gunbc_ir::node::NodeBody::SubDag(_) => None,
        };

        let key = (
            edge.from_node.0.clone(),
            edge.to_node.0.clone(),
            parse_nodes.clone(),
        );
        if seen_keys.insert(key) {
            triplets.push(TransportTriplet {
                prepare_node: edge.from_node.0.clone(),
                execute_node: edge.to_node.0.clone(),
                parse_nodes,
                service_metadata,
            });
        }
    }

    triplets.sort_by(|a, b| {
        (&a.prepare_node, &a.execute_node, &a.parse_nodes).cmp(&(
            &b.prepare_node,
            &b.execute_node,
            &b.parse_nodes,
        ))
    });
    triplets
}

fn node_input_port_type<'a>(node: &'a Node<LoweredOp>, port_name: &str) -> Option<&'a str> {
    node.inputs
        .iter()
        .find(|port| port.name.0 == port_name)
        .map(|port| port.type_id.0.as_str())
}

fn node_output_port_type<'a>(node: &'a Node<LoweredOp>, port_name: &str) -> Option<&'a str> {
    node.outputs
        .iter()
        .find(|port| port.name.0 == port_name)
        .map(|port| port.type_id.0.as_str())
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
        for id in wave {
            topology.push(TopologyNode {
                id: id.clone(),
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
            nodes: wave.clone(),
            depth,
            parent_subdag: None, // nesting support arrives in Phase 3
        })
        .collect()
}

fn derive_subdag_boundaries(nodes: &[Node<LoweredOp>]) -> Vec<SubDagBoundary> {
    let mut boundaries = nodes
        .iter()
        .filter_map(|node| match &node.body {
            gunbc_ir::node::NodeBody::SubDag(subdag) => {
                let mut inner_nodes: Vec<String> =
                    subdag.nodes.iter().map(|n| n.id.0.clone()).collect();
                inner_nodes.sort();
                Some(SubDagBoundary {
                    node_id: node.id.0.clone(),
                    label: node.id.0.clone(),
                    inner_nodes,
                    parent: None, // nesting support arrives in Phase 3
                })
            }
            gunbc_ir::node::NodeBody::Opaque(_) => None,
        })
        .collect::<Vec<_>>();
    boundaries.sort_by(|lhs, rhs| lhs.node_id.cmp(&rhs.node_id));
    boundaries
}

fn derive_scatter_points(nodes: &[Node<LoweredOp>]) -> Vec<String> {
    let mut scatter = Vec::new();
    for node in nodes {
        let gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection {
            module, callable, ..
        }) = &node.body
        else {
            continue;
        };
        let scatter_id = if callable.contains("::") {
            callable.replace("::", ".")
        } else {
            format!("{module}.{callable}")
        };
        scatter.push(scatter_id);
    }
    scatter.sort();
    scatter
}

fn derive_stage_groups(nodes: &[Node<LoweredOp>]) -> Vec<StageGroup> {
    let mut staged = nodes
        .iter()
        .filter_map(|node| match node.body.as_opaque() {
            Some(LoweredOp::Pipeline {
                module,
                name,
                stages,
                stage_names,
            }) => Some((
                node.id.0.clone(),
                module.clone(),
                name.clone(),
                *stages,
                stage_names.clone(),
            )),
            _ => None,
        })
        .flat_map(|(node_id, module, name, stages, stage_names)| {
            if !stage_names.is_empty() {
                return stage_names
                    .into_iter()
                    .enumerate()
                    .map(|(index, stage_name)| {
                        (
                            module.clone(),
                            name.clone(),
                            index,
                            stage_name,
                            node_id.clone(),
                        )
                    })
                    .collect::<Vec<_>>();
            }
            if stages == 0 {
                return vec![(module, name, 0usize, "stage_0".to_string(), node_id)];
            }
            (1..=stages)
                .map(|stage_index| {
                    (
                        module.clone(),
                        name.clone(),
                        stage_index,
                        format!("stage_{stage_index}"),
                        node_id.clone(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    staged.sort_by(|lhs, rhs| {
        lhs.0
            .cmp(&rhs.0)
            .then_with(|| lhs.1.cmp(&rhs.1))
            .then_with(|| lhs.2.cmp(&rhs.2))
            .then_with(|| lhs.3.cmp(&rhs.3))
            .then_with(|| lhs.4.cmp(&rhs.4))
    });
    staged
        .into_iter()
        .map(
            |(module, name, _stage_order, stage_name, node_id)| StageGroup {
                stage_id: format!("{module}.{name}:{stage_name}"),
                nodes: vec![node_id],
            },
        )
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
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Primitive { module, name, .. }) => {
                    format!("{module}.{name}")
                }
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection {
                    module,
                    callable,
                    kind,
                }) => {
                    let callable_label = callable
                        .strip_prefix(&format!("{module}::"))
                        .unwrap_or(callable);
                    format!(
                        "{module}.{callable_label}::{}",
                        kind.node_label()
                    )
                }
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pipeline { module, name, .. }) => {
                    format!("{module}.{name}")
                }
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Pattern(_)) => {
                    "pattern_internal".to_string()
                }
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::UnsupportedPattern { name }) => {
                    format!("unsupported_pattern:{name}")
                }
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::ExternCall { symbol }) => {
                    format!("extern_call:{symbol}")
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
        .map(|node| {
            let capture_mode = match node.body.as_opaque() {
                Some(LoweredOp::Callable {
                    obligation,
                    is_interactive,
                    ..
                }) => {
                    if matches!(
                        obligation,
                        ObligationCategory::ServiceTransportPrepare
                            | ObligationCategory::ServiceTransportExecute
                            | ObligationCategory::ServiceTransportParse
                    ) {
                        CaptureMode::Captured
                    } else if *is_interactive {
                        CaptureMode::Passthrough
                    } else {
                        CaptureMode::Captured
                    }
                }
                _ => CaptureMode::Captured,
            };
            (node.id.0.clone(), capture_mode)
        })
        .collect()
}

fn derive_interactive_nodes(nodes: &[Node<LoweredOp>]) -> Vec<String> {
    let mut interactive = nodes
        .iter()
        .filter_map(|node| match node.body.as_opaque() {
            Some(LoweredOp::Callable {
                is_interactive: true,
                ..
            }) => Some(node.id.0.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    interactive.sort();
    interactive
}

fn derive_resources(nodes: &[Node<LoweredOp>]) -> BTreeMap<String, Vec<ResourceUsage>> {
    let mut resources = BTreeMap::<String, Vec<ResourceUsage>>::new();
    for node in nodes {
        let Some(LoweredOp::Callable {
            obligation,
            resource_target,
            ..
        }) = node.body.as_opaque()
        else {
            continue;
        };
        let Some(resource) = resource_target.as_ref() else {
            continue;
        };
        let usage = match obligation {
            ObligationCategory::ResourceAcquire => Some(ResourceUsage {
                resource: resource.clone(),
                usage: "acquire".to_string(),
            }),
            ObligationCategory::ResourceRelease => Some(ResourceUsage {
                resource: resource.clone(),
                usage: "release".to_string(),
            }),
            ObligationCategory::ResourceProvide => Some(ResourceUsage {
                resource: resource.clone(),
                usage: "provide".to_string(),
            }),
            _ => None,
        };
        if let Some(usage) = usage {
            resources.entry(node.id.0.clone()).or_default().push(usage);
        }
    }
    for usages in resources.values_mut() {
        usages.sort_by(|lhs, rhs| {
            lhs.resource
                .cmp(&rhs.resource)
                .then_with(|| lhs.usage.cmp(&rhs.usage))
        });
    }
    resources
}

fn derive_module_metadata(nodes: &[Node<LoweredOp>]) -> Vec<ModuleMetadata> {
    let mut by_module: BTreeMap<String, ModuleMetadata> = BTreeMap::new();
    for node in nodes {
        let Some(op) = node.body.as_opaque() else {
            continue;
        };
        let (module, is_pipeline) = match op {
            LoweredOp::Callable { module, .. }
            | LoweredOp::Primitive { module, .. }
            | LoweredOp::Collection { module, .. } => (module, false),
            LoweredOp::Pipeline { module, .. } => (module, true),
            LoweredOp::Pattern(_)
            | LoweredOp::UnsupportedPattern { .. }
            | LoweredOp::ExternCall { .. } => continue,
        };
        let entry = by_module
            .entry(module.clone())
            .or_insert_with(|| ModuleMetadata {
                module: module.clone(),
                callable_count: 0,
                pipeline_count: 0,
            });
        if is_pipeline {
            entry.pipeline_count += 1;
        } else {
            entry.callable_count += 1;
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
    interface_contract_verification_targets: usize,
}

fn derive_obligation_counts(nodes: &[Node<LoweredOp>]) -> ObligationCounts {
    let mut counts = ObligationCounts::default();
    for node in nodes {
        if node.kind == NodeKind::TransportExecute {
            counts.transport_execution_targets += 1;
        } else {
            counts.pure_node_determinism_targets += 1;
        }
        let Some(op) = node.body.as_opaque() else {
            continue;
        };
        // Guardrail: a callable with service_metadata set should always have a
        // non-None obligation category. If this fires, a lowering path forgot to
        // set the obligation field on a service transport node, causing obligation
        // counts to silently under-report.
        debug_assert!(
            !(op.service_call_metadata().is_some()
                && classify_obligation(op) == ObligationCategory::None),
            "node `{}` has service_metadata but ObligationCategory::None — \
             lowering must set an explicit obligation category",
            node.id.0,
        );
        match classify_obligation(op) {
            ObligationCategory::ServiceTransportPrepare => {
                counts.service_transport_prepare_targets += 1;
            }
            ObligationCategory::ServiceTransportExecute => {
                counts.service_transport_execute_targets += 1;
                match classify_service_transport(op) {
                    Some(ServiceTransportClass::LocalDirect) => {
                        counts.service_transport_hermetic_targets += 1;
                    }
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
                    // InterfaceStub: hermetic in DryRun (auto-mocked), but
                    // counts as external for live test classification.
                    Some(ServiceTransportClass::InterfaceStub) => {
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
            ObligationCategory::InterfaceContractVerification => {
                counts.interface_contract_verification_targets += 1;
            }
            ObligationCategory::PureRender
            | ObligationCategory::PureDataLoad
            | ObligationCategory::PureGeneric => {
                // Pure functions: no obligation counting needed
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

/// Derive per-callable structural properties via graph walk.
///
/// For each callable (Func/Fn) entrypoint, BFS through reachable nodes
/// and collect transport classes, permissions, idempotent/readonly flags,
/// and service operations. SubDag inner nodes are recursively traversed.
fn derive_callable_properties(dag: &Dag<LoweredOp>) -> BTreeMap<String, CallableProperties> {
    let node_by_id: HashMap<&str, &Node<LoweredOp>> =
        dag.nodes.iter().map(|n| (n.id.0.as_str(), n)).collect();

    // Build adjacency list from edges
    let mut successors: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &dag.edges {
        successors
            .entry(edge.from_node.0.as_str())
            .or_default()
            .push(edge.to_node.0.as_str());
    }

    // Identify callable entrypoints
    let callable_entries: Vec<(&str, &str, &str)> = dag
        .nodes
        .iter()
        .filter_map(|node| match node.body.as_opaque() {
            Some(LoweredOp::Callable {
                kind: CallableKind::Func,
                module,
                name,
                ..
            }) => Some((node.id.0.as_str(), module.as_str(), name.as_str())),
            _ => None,
        })
        .collect();

    let mut result = BTreeMap::new();

    for (entry_id, module, name) in callable_entries {
        let mut transport_classes = BTreeSet::new();
        let mut permissions = BTreeSet::new();
        let mut idempotent = true;
        let mut readonly = true;
        let mut service_operations = BTreeSet::new();
        let mut has_service_metadata = false;

        // BFS from the callable entrypoint
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(entry_id);

        while let Some(node_id) = queue.pop_front() {
            if !visited.insert(node_id) {
                continue;
            }

            if let Some(node) = node_by_id.get(node_id) {
                // Collect service metadata from this node
                collect_service_metadata_from_node(
                    node,
                    &mut transport_classes,
                    &mut permissions,
                    &mut idempotent,
                    &mut readonly,
                    &mut service_operations,
                    &mut has_service_metadata,
                );

                // Recurse into SubDag inner nodes
                if let gunbc_ir::node::NodeBody::SubDag(subdag) = &node.body {
                    for inner_node in &subdag.nodes {
                        collect_service_metadata_from_node(
                            inner_node,
                            &mut transport_classes,
                            &mut permissions,
                            &mut idempotent,
                            &mut readonly,
                            &mut service_operations,
                            &mut has_service_metadata,
                        );
                    }
                }
            }

            // Enqueue successors
            if let Some(nexts) = successors.get(node_id) {
                for next in nexts {
                    queue.push_back(next);
                }
            }
        }

        // If no service metadata found, default idempotent/readonly to true (pure)
        if !has_service_metadata {
            idempotent = true;
            readonly = true;
        }

        let key = format!("{module}::{name}");
        let mut transport_vec: Vec<ServiceTransportClass> = transport_classes.into_iter().collect();
        transport_vec.sort();
        let mut perms_vec: Vec<String> = permissions.into_iter().collect();
        perms_vec.sort();
        let mut ops_vec: Vec<(String, String)> = service_operations.into_iter().collect();
        ops_vec.sort();

        result.insert(
            key,
            CallableProperties {
                transport_classes: transport_vec,
                permissions: perms_vec,
                idempotent,
                readonly,
                service_operations: ops_vec,
            },
        );
    }

    result
}

fn collect_service_metadata_from_node(
    node: &Node<LoweredOp>,
    transport_classes: &mut BTreeSet<ServiceTransportClass>,
    permissions: &mut BTreeSet<String>,
    idempotent: &mut bool,
    readonly: &mut bool,
    service_operations: &mut BTreeSet<(String, String)>,
    has_service_metadata: &mut bool,
) {
    let Some(op) = node.body.as_opaque() else {
        return;
    };
    if let Some(metadata) = op.service_call_metadata() {
        *has_service_metadata = true;
        transport_classes.insert(metadata.transport);
        for perm in &metadata.permissions {
            permissions.insert(perm.clone());
        }
        if !metadata.idempotent {
            *idempotent = false;
        }
        if !metadata.readonly {
            *readonly = false;
        }
        service_operations.insert((metadata.service.clone(), metadata.operation.clone()));
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
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        )
        .with_kind(NodeKind::Pure)
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
                    id: "a".to_string(),
                    depth: 0,
                },
                TopologyNode {
                    id: "b".to_string(),
                    depth: 0,
                },
                TopologyNode {
                    id: "c".to_string(),
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
                    nodes: vec!["a".to_string(), "b".to_string()],
                    depth: 0,
                    parent_subdag: None,
                },
                ParallelGroup {
                    nodes: vec!["c".to_string()],
                    depth: 1,
                    parent_subdag: None,
                },
            ]
        );
        assert_eq!(
            artifacts.manifest.capture_modes.len(),
            3,
            "capture modes should be derived for all nodes"
        );
        assert!(
            artifacts.manifest.subdag_boundaries.is_empty(),
            "opaque-only DAG should not report subdag boundaries"
        );
    }

    #[test]
    fn derive_module_metadata_counts_callable_and_pipeline_nodes() {
        let mut dag = Dag::new();
        dag.add_node(callable_node("makegen", "tools.makegen", "makegen"));
        dag.add_node(
            Node::opaque(
                "ci",
                vec![],
                vec![Port::scalar("stages", "Int")],
                LoweredOp::Pipeline {
                    module: "pipelines.ci".to_string(),
                    name: "ci".to_string(),
                    stages: 12,
                    stage_names: (1..=12).map(|i| format!("stage_{i}")).collect(),
                },
            )
            .with_kind(NodeKind::Pure),
        );

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
        let expected_stage_groups = (1..=12)
            .map(|stage_index| StageGroup {
                stage_id: format!("pipelines.ci.ci:stage_{stage_index}"),
                nodes: vec!["ci".to_string()],
            })
            .collect::<Vec<_>>();
        assert_eq!(artifacts.manifest.stage_groups, expected_stage_groups);
    }

    #[test]
    fn derive_manifest_collects_collection_scatter_points() {
        let mut dag = Dag::new();
        dag.add_node(
            Node::opaque(
                "snapshot_map_0",
                vec![Port::scalar("items", "List<String>")],
                vec![Port::scalar("items", "List<String>")],
                LoweredOp::Collection {
                    module: "tools.gist".to_string(),
                    callable: "render_snapshot".to_string(),
                    kind: CollectionOpKind::Map,
                },
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "snapshot_join_1",
                vec![Port::scalar("items", "List<String>")],
                vec![Port::scalar("items", "String")],
                LoweredOp::Collection {
                    module: "tools.gist".to_string(),
                    callable: "render_snapshot".to_string(),
                    kind: CollectionOpKind::Join,
                },
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_node(callable_node(
            "gist_snapshot",
            "tools.gist",
            "gist_snapshot",
        ));

        let artifacts = derive_artifacts(&dag).expect("derivation should succeed");
        assert_eq!(
            artifacts.manifest.scatter_points,
            vec![
                "tools.gist.render_snapshot".to_string(),
                "tools.gist.render_snapshot".to_string()
            ]
        );
    }

    #[test]
    fn derive_obligations_count_transport_and_lifecycle_targets() {
        let mut dag = Dag::new();
        dag.add_node(
            Node::opaque(
                "param_source",
                vec![],
                vec![Port::scalar("path", "String")],
                LoweredOp::Callable {
                    module: "sample.services".to_string(),
                    kind: CallableKind::Pattern,
                    name: "call_param_source::run::path".to_string(),
                    obligation: ObligationCategory::ServiceParamSource,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "prepare_transport",
                vec![Port::scalar("path", "String")],
                vec![Port::scalar("request", "TransportRequest")],
                LoweredOp::Callable {
                    module: "sample.services".to_string(),
                    kind: CallableKind::Pattern,
                    name: "service_transport::prepare::FsStorage::read".to_string(),
                    obligation: ObligationCategory::ServiceTransportPrepare,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::TransportPrepare),
        );
        dag.add_node(
            Node::opaque(
                "execute_transport",
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                LoweredOp::Callable {
                    module: "sample.services".to_string(),
                    kind: CallableKind::Pattern,
                    name: "service_transport::execute::FsStorage::read".to_string(),
                    obligation: ObligationCategory::ServiceTransportExecute,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::TransportExecute),
        );
        dag.add_node(
            Node::opaque(
                "parse_transport",
                vec![Port::scalar("response", "TransportResponse")],
                vec![Port::scalar("body", "String")],
                LoweredOp::Callable {
                    module: "sample.services".to_string(),
                    kind: CallableKind::Pattern,
                    name: "service_transport::parse::FsStorage::read".to_string(),
                    obligation: ObligationCategory::ServiceTransportParse,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::TransportParse),
        );
        dag.add_node(
            Node::opaque(
                "provide_resource",
                vec![Port::scalar("trigger", "Any")],
                vec![Port::scalar("out", "Storage")],
                LoweredOp::Callable {
                    module: "sample.resources".to_string(),
                    kind: CallableKind::Pattern,
                    name: "resource_provide::run::out".to_string(),
                    obligation: ObligationCategory::ResourceProvide,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: Some("out".to_string()),
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::ResourceEnvironment),
        );
        dag.add_node(
            Node::opaque(
                "acquire_resource",
                vec![],
                vec![Port::scalar("resource_handle", "ResourceHandle")],
                LoweredOp::Callable {
                    module: "sample.resources".to_string(),
                    kind: CallableKind::Pattern,
                    name: "resource_lifecycle::acquire::TempFile".to_string(),
                    obligation: ObligationCategory::ResourceAcquire,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: Some("TempFile".to_string()),
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::ResourceAcquire),
        );
        dag.add_node(
            Node::opaque(
                "release_resource",
                vec![Port::scalar("resource_handle", "ResourceHandle")],
                vec![Port::scalar("released", "Bool")],
                LoweredOp::Callable {
                    module: "sample.resources".to_string(),
                    kind: CallableKind::Pattern,
                    name: "resource_lifecycle::release::TempFile".to_string(),
                    obligation: ObligationCategory::ResourceRelease,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: Some("TempFile".to_string()),
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::ResourceRelease),
        );
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
        assert_eq!(
            artifacts
                .obligations
                .interface_contract_verification_targets,
            0
        );
        assert_eq!(artifacts.transport_triplets.len(), 1);
        assert_eq!(
            artifacts.transport_triplets[0],
            TransportTriplet {
                prepare_node: "prepare_transport".to_string(),
                execute_node: "execute_transport".to_string(),
                parse_nodes: vec!["parse_transport".to_string()],
                service_metadata: None,
            }
        );
        assert_eq!(
            artifacts
                .manifest
                .resources
                .get("acquire_resource")
                .expect("acquire resource usage should be derived"),
            &vec![ResourceUsage {
                resource: "TempFile".to_string(),
                usage: "acquire".to_string(),
            }]
        );
        assert_eq!(
            artifacts
                .manifest
                .resources
                .get("release_resource")
                .expect("release resource usage should be derived"),
            &vec![ResourceUsage {
                resource: "TempFile".to_string(),
                usage: "release".to_string(),
            }]
        );
        assert!(
            artifacts.manifest.interactive_nodes.is_empty(),
            "fixture DAG has no interactive annotations"
        );
    }

    #[test]
    fn derive_obligations_tracks_service_transport_semantic_buckets() {
        let mut dag = Dag::new();
        dag.add_node(
            Node::opaque(
                "execute_transport_hermetic",
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                LoweredOp::Callable {
                    module: "sample.services".to_string(),
                    kind: CallableKind::Pattern,
                    name: "service_transport::execute::FsStorage::read".to_string(),
                    obligation: ObligationCategory::ServiceTransportExecute,
                    service_metadata: Some(Box::new(ServiceCallMetadata {
                        service: "FsStorage".to_string(),
                        operation: "read".to_string(),
                        transport: ServiceTransportClass::ShellLocal,
                        idempotent: true,
                        readonly: true,
                        permissions: vec![],
                        spec: None,
                    })),
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::TransportExecute),
        );
        dag.add_node(
            Node::opaque(
                "execute_transport_external",
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                LoweredOp::Callable {
                    module: "sample.services".to_string(),
                    kind: CallableKind::Pattern,
                    name: "service_transport::execute::GistApi::create".to_string(),
                    obligation: ObligationCategory::ServiceTransportExecute,
                    service_metadata: Some(Box::new(ServiceCallMetadata {
                        service: "GistApi".to_string(),
                        operation: "create".to_string(),
                        transport: ServiceTransportClass::RestNetwork,
                        idempotent: false,
                        readonly: false,
                        permissions: vec!["gist.write".to_string()],
                        spec: None,
                    })),
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::TransportExecute),
        );

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
        assert_eq!(
            artifacts
                .obligations
                .interface_contract_verification_targets,
            0
        );
    }

    #[test]
    fn derive_obligations_uses_structural_category_not_callable_name_prefix() {
        let mut dag = Dag::new();
        dag.add_node(
            Node::opaque(
                "misleading_name",
                vec![],
                vec![Port::scalar("out", "String")],
                LoweredOp::Callable {
                    module: "sample.services".to_string(),
                    kind: CallableKind::Pattern,
                    name: "service_transport::prepare::Fake::op".to_string(),
                    obligation: ObligationCategory::None,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "structural_category",
                vec![],
                vec![Port::scalar("out", "String")],
                LoweredOp::Callable {
                    module: "sample.services".to_string(),
                    kind: CallableKind::Pattern,
                    name: "not_a_transport_name".to_string(),
                    obligation: ObligationCategory::ServiceTransportPrepare,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::TransportPrepare),
        );

        let artifacts = derive_artifacts(&dag).expect("derivation should succeed");
        assert_eq!(
            artifacts.obligations.service_transport_prepare_targets, 1,
            "classification should follow structural obligation category"
        );
    }

    #[test]
    fn derive_manifest_uses_structural_interactive_metadata() {
        let mut dag = Dag::new();
        dag.add_node(
            Node::opaque(
                "interactive_node",
                vec![Port::scalar("input", "String")],
                vec![Port::scalar("output", "String")],
                LoweredOp::Callable {
                    module: "sample.app".to_string(),
                    kind: CallableKind::Func,
                    name: "prompt_user".to_string(),
                    obligation: ObligationCategory::None,
                    service_metadata: None,
                    is_interactive: true,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::Pure),
        );
        dag.add_node(
            Node::opaque(
                "transport_node",
                vec![Port::scalar("request", "TransportRequest")],
                vec![Port::scalar("response", "TransportResponse")],
                LoweredOp::Callable {
                    module: "sample.app".to_string(),
                    kind: CallableKind::Pattern,
                    name: "service_transport::execute::FsStorage::read".to_string(),
                    obligation: ObligationCategory::ServiceTransportExecute,
                    service_metadata: None,
                    is_interactive: false,
                    resource_target: None,
                    fn_body: None,
                },
            )
            .with_kind(NodeKind::TransportExecute),
        );

        let artifacts = derive_artifacts(&dag).expect("derivation should succeed");
        assert_eq!(
            artifacts.manifest.interactive_nodes,
            vec!["interactive_node".to_string()]
        );
        assert_eq!(
            artifacts
                .manifest
                .capture_modes
                .get("interactive_node")
                .cloned(),
            Some(CaptureMode::Passthrough)
        );
        assert_eq!(
            artifacts
                .manifest
                .capture_modes
                .get("transport_node")
                .cloned(),
            Some(CaptureMode::Captured)
        );
    }
}
