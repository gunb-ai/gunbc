//! Transport execution interception for dry-run mode.
//!
//! In dry-run mode, **transport execution nodes** (those that consume
//! `TransportRequest` values) and **tool environment nodes** (those that
//! emit `ToolHandle` outputs) have their execution intercepted and replaced
//! with mock behavior. This follows the design principle:
//!
//! > "World I/O is performed only by transport executor nodes"
//! > "DryRun intercepts transport execution nodes, not boundary outputs"
//!
//! Additionally, **DAG entry inputs** (input ports with no incoming edges)
//! can be mocked via `input_mocks`. This allows testing DAGs that expect
//! external inputs when run in isolation.
//!
//! Note: The mocks are still called "BoundaryMocks" for backwards compatibility,
//! but they apply to transport execution nodes, not boundary nodes. Missing
//! mocks are treated as errors by the executor (no default fallback).

use gunbc_ir::{NodeId, PortName, Value};
use std::cell::Cell;
use std::collections::HashMap;

/// Mock behavior for a single boundary port.
///
/// Supports both static values and ordered sequences. When a sequence is
/// provided, `next_value()` returns values in order; once exhausted it
/// falls back to the static `value`.
#[derive(Debug, Clone)]
pub struct BoundaryMock {
    /// The static fallback value
    pub value: Value,
    /// Ordered responses (returned before falling back to `value`)
    sequence: Vec<Value>,
    /// Call counter (Cell for interior mutability — DAG execution is single-threaded)
    call_count: Cell<usize>,
}

impl BoundaryMock {
    pub fn new(value: Value) -> Self {
        Self {
            value,
            sequence: Vec::new(),
            call_count: Cell::new(0),
        }
    }

    /// Create a mock with an ordered sequence of responses.
    ///
    /// `next_value()` returns `sequence[i]` for call `i`; once the
    /// sequence is exhausted, it returns the `default` value.
    pub fn with_sequence(default: Value, sequence: Vec<Value>) -> Self {
        Self {
            value: default,
            sequence,
            call_count: Cell::new(0),
        }
    }

    /// Return the next value in the sequence, or the static fallback.
    pub fn next_value(&self) -> Value {
        let idx = self.call_count.get();
        self.call_count.set(idx + 1);
        if idx < self.sequence.len() {
            self.sequence[idx].clone()
        } else {
            self.value.clone()
        }
    }

    /// Get the current call count.
    pub fn call_count(&self) -> usize {
        self.call_count.get()
    }
}

/// Collection of mocks for boundary ports and DAG entry inputs.
#[derive(Debug, Clone, Default)]
pub struct BoundaryMocks {
    /// Map from (node_id, port_name) to mock behavior for outputs
    mocks: HashMap<(String, String), BoundaryMock>,
    /// Map from (node_id, port_name) to mock value for inputs (DAG entry points)
    input_mocks: HashMap<(String, String), Value>,
}

impl BoundaryMocks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a specific mock for a boundary port (output interception).
    pub fn set_mock(
        &mut self,
        node_id: impl Into<String>,
        port_name: impl Into<String>,
        mock: BoundaryMock,
    ) {
        self.mocks.insert((node_id.into(), port_name.into()), mock);
    }

    /// Set a mock value directly for a boundary port (output interception).
    pub fn set_value(
        &mut self,
        node_id: impl Into<String>,
        port_name: impl Into<String>,
        value: Value,
    ) {
        self.set_mock(node_id, port_name, BoundaryMock::new(value));
    }

    /// Set a mock value for a DAG entry input (input injection).
    ///
    /// Use this when a node has an input port with no incoming edge.
    /// The mock value will be injected as if it came from an upstream node.
    pub fn set_input(
        &mut self,
        node_id: impl Into<String>,
        port_name: impl Into<String>,
        value: Value,
    ) {
        self.input_mocks
            .insert((node_id.into(), port_name.into()), value);
    }

    /// Get the mock value for a DAG entry input, if defined.
    pub fn get_input(&self, node_id: &str, port_name: &str) -> Option<&Value> {
        let key = (node_id.to_string(), port_name.to_string());
        self.input_mocks.get(&key)
    }

    /// Check if an input mock is defined for a specific port.
    pub fn has_input(&self, node_id: &str, port_name: &str) -> bool {
        let key = (node_id.to_string(), port_name.to_string());
        self.input_mocks.contains_key(&key)
    }

    /// Get the mock for a boundary port, if defined.
    pub fn get_mock(&self, node_id: &NodeId, port_name: &PortName) -> Option<&BoundaryMock> {
        let key = (node_id.0.clone(), port_name.0.clone());
        self.mocks.get(&key)
    }

    /// Set a sequenced mock for a boundary port (output interception).
    ///
    /// Returns values from `sequence` in order; once exhausted, falls back to `default`.
    pub fn set_sequence(
        &mut self,
        node_id: impl Into<String>,
        port_name: impl Into<String>,
        default: Value,
        sequence: Vec<Value>,
    ) {
        self.mocks.insert(
            (node_id.into(), port_name.into()),
            BoundaryMock::with_sequence(default, sequence),
        );
    }

    /// Check if a specific mock is defined for a boundary port.
    pub fn has_mock(&self, node_id: &NodeId, port_name: &PortName) -> bool {
        let key = (node_id.0.clone(), port_name.0.clone());
        self.mocks.contains_key(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_mock() {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value("gist", "url", Value::Str("https://mock.gist".to_string()));

        let mock = mocks.get_mock(&"gist".into(), &"url".into()).unwrap();
        match &mock.value {
            Value::Str(s) => assert_eq!(s, "https://mock.gist"),
            _ => panic!("expected string value"),
        }
        assert!(mocks.get_mock(&"other".into(), &"port".into()).is_none());
    }

    #[test]
    fn test_sequence_returns_in_order() {
        let mock = BoundaryMock::with_sequence(
            Value::Str("default".into()),
            vec![
                Value::Str("first".into()),
                Value::Str("second".into()),
            ],
        );

        assert_eq!(mock.next_value(), Value::Str("first".into()));
        assert_eq!(mock.next_value(), Value::Str("second".into()));
    }

    #[test]
    fn test_sequence_exhausted_falls_back_to_default() {
        let mock = BoundaryMock::with_sequence(
            Value::Str("default".into()),
            vec![Value::Str("first".into())],
        );

        assert_eq!(mock.next_value(), Value::Str("first".into()));
        assert_eq!(mock.next_value(), Value::Str("default".into()));
        assert_eq!(mock.next_value(), Value::Str("default".into()));
    }

    #[test]
    fn test_sequence_call_count() {
        let mock = BoundaryMock::with_sequence(
            Value::Str("default".into()),
            vec![Value::Str("a".into()), Value::Str("b".into())],
        );

        assert_eq!(mock.call_count(), 0);
        mock.next_value();
        assert_eq!(mock.call_count(), 1);
        mock.next_value();
        assert_eq!(mock.call_count(), 2);
        mock.next_value(); // falls back to default
        assert_eq!(mock.call_count(), 3);
    }

    #[test]
    fn test_static_mock_always_returns_same() {
        let mock = BoundaryMock::new(Value::Int(42));

        assert_eq!(mock.next_value(), Value::Int(42));
        assert_eq!(mock.next_value(), Value::Int(42));
        assert_eq!(mock.next_value(), Value::Int(42));
    }

    #[test]
    fn test_set_sequence_on_boundary_mocks() {
        let mut mocks = BoundaryMocks::new();
        mocks.set_sequence(
            "node",
            "port",
            Value::Str("default".into()),
            vec![Value::Str("first".into())],
        );

        let mock = mocks.get_mock(&"node".into(), &"port".into()).unwrap();
        assert_eq!(mock.next_value(), Value::Str("first".into()));
        assert_eq!(mock.next_value(), Value::Str("default".into()));
    }
}
