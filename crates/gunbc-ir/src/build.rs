//! Builder helpers for constructing DAG elements.
//!
//! These are the canonical constructors — use them instead of
//! writing out struct literals with `.into()` everywhere.

use crate::algebra::{Predicate, Value};
use crate::dag::{Edge, Port};
use crate::types::{NodeId, PortName, TypeId};

/// Create a port with no guard.
pub fn port(name: &str, ty: &str) -> Port {
    Port {
        name: PortName(name.into()),
        type_id: TypeId(ty.into()),
        guard: None,
    }
}

/// Create a port with a typed predicate guard.
pub fn guarded_port(name: &str, ty: &str, guard: Predicate) -> Port {
    Port {
        name: PortName(name.into()),
        type_id: TypeId(ty.into()),
        guard: Some(guard),
    }
}

/// Create a port with an equality guard (value must equal expected).
pub fn eq_guarded_port(name: &str, ty: &str, expected: Value) -> Port {
    guarded_port(name, ty, Predicate::Eq(expected))
}

/// Create a port with a not-equal guard (value must not equal expected).
pub fn neq_guarded_port(name: &str, ty: &str, expected: Value) -> Port {
    guarded_port(name, ty, Predicate::NotEq(expected))
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
