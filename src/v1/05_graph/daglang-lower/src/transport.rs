//! Transport manifest: pure-data types for transport triplet derivation.
//!
//! This module defines `TransportManifest` — the pure-data result of transport
//! derivation. The manifest contains all nodes, edges, and registry entries
//! needed to represent transport triplets in the DAG.
//!
//! It also provides `TransportTripletSpec` and `build_transport_triplet` for
//! constructing the prepare/execute/parse node triplet in a single place,
//! eliminating the 4-way duplication across service, interface stub, loop body,
//! and branch body transport creation sites.

use gunbc_ir::{
    Dag, Edge, Node, NodeOrigin, OperationKey, Port, Predicate, PredicateValue,
};

use crate::{
    CallableKind, LoweredOp, ServiceCallMetadata, ServiceEndpointRegistry, TransportObligation,
};

/// A deferred edge to be applied to the DagBuilder.
pub(crate) struct ManifestEdge {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

/// Pure-data result of transport derivation.
///
/// Contains all the nodes, edges, and registry entries needed to represent
/// transport triplets in the DAG, without mutating any builder directly.
pub(crate) struct TransportManifest {
    pub nodes: Vec<Node<LoweredOp>>,
    pub edges: Vec<ManifestEdge>,
    pub registry: ServiceEndpointRegistry,
}

impl TransportManifest {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            registry: ServiceEndpointRegistry::default(),
        }
    }

    pub fn add_node(&mut self, node: Node<LoweredOp>) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) {
        self.edges.push(ManifestEdge {
            from_node: from_node.to_string(),
            from_port: from_port.to_string(),
            to_node: to_node.to_string(),
            to_port: to_port.to_string(),
        });
    }
}

// ============================================================================
// Transport triplet construction (S16: Transport Dispatch)
// ============================================================================

/// Configuration for building a transport triplet (prepare/execute/parse).
///
/// Captures all variation across the 4 call sites: service transport, interface
/// stub, loop body, and branch body. A single `build_transport_triplet` function
/// consumes this spec and produces the 3 nodes + edges.
pub(crate) struct TransportTripletSpec {
    /// Module name for the `LoweredOp::Transport` payload.
    pub module: String,
    /// Service name (for the `service_transport::*` name prefix).
    pub service: String,
    /// Operation name (for the `service_transport::*` name prefix).
    pub operation: String,
    /// Full service call metadata.
    pub metadata: ServiceCallMetadata,
    /// Node IDs for the triplet.
    pub prepare_id: String,
    pub execute_id: String,
    pub parse_id: String,
    /// Input ports for the prepare node.
    pub prepare_inputs: Vec<Port>,
    /// Extra input ports for the execute node beyond `request: TransportRequest`.
    /// For example, `res:credential: Credential` for REST auth.
    pub execute_extra_inputs: Vec<Port>,
    /// Output ports for the parse node.
    pub parse_outputs: Vec<Port>,
    /// How execute→parse wiring works.
    pub execute_parse_wiring: ExecuteParseWiring,
    /// Optional origin for all 3 nodes.
    pub origin: Option<NodeOrigin>,
    /// Optional operation key for the execute node.
    pub operation_key: Option<OperationKey>,
}

/// How the execute node connects to the parse node.
pub(crate) enum ExecuteParseWiring {
    /// Standard wiring: execute.response → parse.response.
    /// The execute node outputs `[response: TransportResponse]` and the parse
    /// node inputs `[response: TransportResponse]`.
    Response,
    /// Per-field wiring: execute outputs typed fields, parse inputs the same
    /// typed fields. Used by interface stubs where the execute node produces
    /// typed capability outputs directly (not a raw TransportResponse).
    PerField {
        /// The typed output ports on the execute node (same as parse inputs).
        fields: Vec<Port>,
    },
}

/// The 3 nodes + edges produced by `build_transport_triplet`.
pub(crate) struct TransportTripletNodes {
    pub prepare: Node<LoweredOp>,
    pub execute: Node<LoweredOp>,
    pub parse: Node<LoweredOp>,
    pub edges: Vec<TripletEdge>,
}

/// An edge within a transport triplet.
pub(crate) struct TripletEdge {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

/// Build the 3 transport nodes (prepare/execute/parse) and their internal edges.
///
/// This is the single source of truth for transport triplet construction,
/// replacing the duplicated patterns in `derive_service_transport_triplets`,
/// `derive_interface_stub_transport_triplets`, `make_loop_body_dag`, and
/// `make_branch_body_dag`.
pub(crate) fn build_transport_triplet(spec: TransportTripletSpec) -> TransportTripletNodes {
    // -- Prepare node: inputs → [request: TransportRequest] -------------------
    let prepare_node = {
        let node = Node::opaque(
            spec.prepare_id.clone(),
            spec.prepare_inputs,
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Transport {
                module: spec.module.clone(),
                kind: CallableKind::Pattern,
                name: format!(
                    "service_transport::prepare::{}::{}",
                    spec.service, spec.operation
                ),
                obligation: TransportObligation::Prepare,
                service_metadata: Box::new(spec.metadata.clone()),
                is_interactive: false,
                resource_target: None,
            },
        );
        let mut node = match &spec.origin {
            Some(origin) => node.with_origin(origin.clone()),
            None => node,
        };
        if let Some(key) = &spec.operation_key {
            node = node.with_operation_key(key.clone());
        }
        node
    };

    // -- Execute node: [request + extras] → [response or typed fields] --------
    let execute_node = {
        let mut execute_inputs = vec![Port::scalar("request", "TransportRequest")];
        execute_inputs.extend(spec.execute_extra_inputs);

        let execute_outputs = match &spec.execute_parse_wiring {
            ExecuteParseWiring::Response => {
                vec![Port::scalar("response", "TransportResponse")]
            }
            ExecuteParseWiring::PerField { fields } => fields.clone(),
        };

        let mut node = Node::opaque(
            spec.execute_id.clone(),
            execute_inputs,
            execute_outputs,
            LoweredOp::Transport {
                module: spec.module.clone(),
                kind: CallableKind::Pattern,
                name: format!(
                    "service_transport::execute::{}::{}",
                    spec.service, spec.operation
                ),
                obligation: TransportObligation::Execute,
                service_metadata: Box::new(spec.metadata.clone()),
                is_interactive: false,
                resource_target: None,
            },
        )
        .with_input_guard(
            "request",
            Predicate::Not(Box::new(Predicate::Equals(PredicateValue::Skipped))),
        );

        if let Some(key) = &spec.operation_key {
            node = node.with_operation_key(key.clone());
        }
        match &spec.origin {
            Some(origin) => node.with_origin(origin.clone()),
            None => node,
        }
    };

    // -- Parse node: [response or typed fields] → parse_outputs ---------------
    let parse_node = {
        let parse_inputs = match &spec.execute_parse_wiring {
            ExecuteParseWiring::Response => {
                vec![Port::scalar("response", "TransportResponse")]
            }
            ExecuteParseWiring::PerField { fields } => fields.clone(),
        };

        let node = Node::opaque(
            spec.parse_id.clone(),
            parse_inputs,
            spec.parse_outputs,
            LoweredOp::Transport {
                module: spec.module.clone(),
                kind: CallableKind::Pattern,
                name: format!(
                    "service_transport::parse::{}::{}",
                    spec.service, spec.operation
                ),
                obligation: TransportObligation::Parse,
                service_metadata: Box::new(spec.metadata.clone()),
                is_interactive: false,
                resource_target: None,
            },
        );
        let mut node = match &spec.origin {
            Some(origin) => node.with_origin(origin.clone()),
            None => node,
        };
        if let Some(key) = &spec.operation_key {
            node = node.with_operation_key(key.clone());
        }
        node
    };

    // -- Internal edges: prepare → execute, execute → parse -------------------
    let mut edges = vec![TripletEdge {
        from_node: spec.prepare_id,
        from_port: "request".to_string(),
        to_node: spec.execute_id.clone(),
        to_port: "request".to_string(),
    }];

    match &spec.execute_parse_wiring {
        ExecuteParseWiring::Response => {
            edges.push(TripletEdge {
                from_node: spec.execute_id,
                from_port: "response".to_string(),
                to_node: spec.parse_id,
                to_port: "response".to_string(),
            });
        }
        ExecuteParseWiring::PerField { fields } => {
            for field in fields {
                edges.push(TripletEdge {
                    from_node: spec.execute_id.clone(),
                    from_port: field.name.0.clone(),
                    to_node: spec.parse_id.clone(),
                    to_port: field.name.0.clone(),
                });
            }
        }
    }

    TransportTripletNodes {
        prepare: prepare_node,
        execute: execute_node,
        parse: parse_node,
        edges,
    }
}

/// Emit a transport triplet into a `TransportManifest`.
pub(crate) fn emit_triplet_to_manifest(manifest: &mut TransportManifest, triplet: TransportTripletNodes) {
    manifest.add_node(triplet.prepare);
    manifest.add_node(triplet.execute);
    manifest.add_node(triplet.parse);
    for edge in triplet.edges {
        manifest.add_edge(&edge.from_node, &edge.from_port, &edge.to_node, &edge.to_port);
    }
}

/// Emit a transport triplet into a `Dag<LoweredOp>` directly.
pub(crate) fn emit_triplet_to_dag(dag: &mut Dag<LoweredOp>, triplet: TransportTripletNodes) {
    dag.add_node(triplet.prepare);
    dag.add_node(triplet.execute);
    dag.add_node(triplet.parse);
    for edge in triplet.edges {
        dag.add_edge(Edge::new(
            edge.from_node.as_str(),
            edge.from_port.as_str(),
            edge.to_node.as_str(),
            edge.to_port.as_str(),
        ));
    }
}
