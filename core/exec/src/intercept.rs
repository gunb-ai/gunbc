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
//! but they apply to transport execution nodes, not boundary nodes.

use gunbc_ir::{NodeId, PortName, Value};
use std::collections::HashMap;

/// Mock behavior for a single boundary port.
#[derive(Debug, Clone)]
pub struct BoundaryMock {
    /// The mock value to return for this port
    pub value: Value,
}

impl BoundaryMock {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

impl Default for BoundaryMock {
    fn default() -> Self {
        Self {
            value: Value::Str("<DRY-RUN>".to_string()),
        }
    }
}

/// Collection of mocks for boundary ports and DAG entry inputs.
#[derive(Debug, Clone, Default)]
pub struct BoundaryMocks {
    /// Map from (node_id, port_name) to mock behavior for outputs
    mocks: HashMap<(String, String), BoundaryMock>,
    /// Map from (node_id, port_name) to mock value for inputs (DAG entry points)
    input_mocks: HashMap<(String, String), Value>,
    /// Default mock to use when no specific mock is defined
    default_mock: BoundaryMock,
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
        self.input_mocks.insert((node_id.into(), port_name.into()), value);
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

    /// Get the mock for a boundary port, using the default if not set.
    pub fn get_mock(&self, node_id: &NodeId, port_name: &PortName) -> &BoundaryMock {
        let key = (node_id.0.clone(), port_name.0.clone());
        self.mocks.get(&key).unwrap_or(&self.default_mock)
    }

    /// Check if a specific mock is defined for a boundary port.
    pub fn has_mock(&self, node_id: &NodeId, port_name: &PortName) -> bool {
        let key = (node_id.0.clone(), port_name.0.clone());
        self.mocks.contains_key(&key)
    }

    /// Set the default mock to use for unspecified boundary ports.
    pub fn set_default(&mut self, mock: BoundaryMock) {
        self.default_mock = mock;
    }

    /// Set the default mock value.
    pub fn set_default_value(&mut self, value: Value) {
        self.default_mock = BoundaryMock::new(value);
    }

    /// Create mocks with a custom default value.
    pub fn with_default(value: Value) -> Self {
        let mut mocks = Self::new();
        mocks.set_default_value(value);
        mocks
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mock() {
        let mocks = BoundaryMocks::new();
        let mock = mocks.get_mock(&"node".into(), &"port".into());
        
        match &mock.value {
            Value::Str(s) => assert_eq!(s, "<DRY-RUN>"),
            _ => panic!("expected string value"),
        }
    }

    #[test]
    fn test_specific_mock() {
        let mut mocks = BoundaryMocks::new();
        mocks.set_value("gist", "url", Value::Str("https://mock.gist".to_string()));

        let mock = mocks.get_mock(&"gist".into(), &"url".into());
        match &mock.value {
            Value::Str(s) => assert_eq!(s, "https://mock.gist"),
            _ => panic!("expected string value"),
        }

        // Other ports still get the default
        let other = mocks.get_mock(&"other".into(), &"port".into());
        match &other.value {
            Value::Str(s) => assert_eq!(s, "<DRY-RUN>"),
            _ => panic!("expected string value"),
        }
    }
}
