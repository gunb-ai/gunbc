//! Transport derivation: builds transport triplets as pure data.
//!
//! This module defines `TransportManifest` — the pure-data result of transport
//! derivation. The manifest contains all nodes, edges, and registry entries
//! needed to represent transport triplets in the DAG.
//!
//! Invariant: every service call site maps to exactly one transport triplet
//! (prepare → execute → parse).

use gunbc_ir::Node;

use crate::{LoweredOp, ServiceEndpointRegistry};

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
