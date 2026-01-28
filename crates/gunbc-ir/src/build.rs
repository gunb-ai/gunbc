//! Builder helpers for constructing DAG elements.
//!
//! These are the canonical constructors — use them instead of
//! writing out struct literals with `.into()` everywhere.

use crate::dag::{Edge, Port};
use crate::metadata::NodeMetadata;
use crate::types::{BehaviorKind, NodeId, PortName, ToolId, TypeId};

/// Create a port with no guard.
pub fn port(name: &str, ty: &str) -> Port {
    Port {
        name: PortName(name.into()),
        type_id: TypeId(ty.into()),
        guard: None,
    }
}

/// Create a port with a guard expression.
pub fn guarded_port(name: &str, ty: &str, guard: &str) -> Port {
    Port {
        name: PortName(name.into()),
        type_id: TypeId(ty.into()),
        guard: Some(guard.into()),
    }
}

/// Create an edge between two nodes.
pub fn edge(from: &str, from_port: &str, to: &str, to_port: &str) -> Edge {
    Edge {
        from_node: NodeId(from.into()),
        from_port: PortName(from_port.into()),
        to_node: NodeId(to.into()),
        to_port: PortName(to_port.into()),
    }
}

/// Create node metadata.
pub fn node_meta(tool: &str, behavior: BehaviorKind) -> NodeMetadata {
    NodeMetadata {
        tool: ToolId(tool.into()),
        behavior,
    }
}
