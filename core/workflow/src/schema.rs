//! Workflow planner schema (WF1).
//!
//! This module defines typed workflow planner units over existing DAG primitives.

use gunbc_ir::{Dag, Port, PortName, TypeId};

use crate::process_registry::ProcessUnitRef;

/// Canonical workflow ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkflowId(pub String);

impl WorkflowId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<&str> for WorkflowId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// Minimum typed workflow spec for planner execution.
#[derive(Debug, Clone)]
pub struct WorkflowSpec {
    pub id: WorkflowId,
    pub dag: Dag<WorkflowUnit>,
    pub policy_version: u32,
}

impl WorkflowSpec {
    pub fn new(id: impl Into<WorkflowId>, dag: Dag<WorkflowUnit>, policy_version: u32) -> Self {
        Self {
            id: id.into(),
            dag,
            policy_version,
        }
    }
}

/// Typed unit carried by planner DAG nodes.
#[derive(Debug, Clone)]
pub struct WorkflowUnit {
    pub op: WorkflowOp,
}

impl WorkflowUnit {
    pub fn new(op: WorkflowOp) -> Self {
        Self { op }
    }
}

/// Workflow operation (closed typed set, no shell-string fallback).
#[derive(Debug, Clone)]
pub enum WorkflowOp {
    InvokeProcessUnit(ProcessUnitRef),
    Aggregate(AggregateSpec),
    Report(ReportSpec),
}

/// Aggregate planner operation.
#[derive(Debug, Clone)]
pub struct AggregateSpec {
    pub label: String,
}

impl AggregateSpec {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

/// Report planner operation.
#[derive(Debug, Clone)]
pub struct ReportSpec {
    pub label: String,
}

impl ReportSpec {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

/// Required control input for workflow units.
pub const PORT_AFTER: &str = "after";
/// Required control output for workflow units.
pub const PORT_COMMIT: &str = "commit";
/// Required data output for workflow units.
pub const PORT_RESULT: &str = "result";
/// Type ID for workflow result payload.
pub const TYPE_WORKFLOW_RESULT: &str = "WorkflowResult";

/// Required input contract for planner units.
pub fn required_input_contract() -> Vec<Port> {
    vec![Port::optional(PORT_AFTER, "Bool")]
}

/// Required output contract for planner units.
pub fn required_output_contract() -> Vec<Port> {
    vec![
        Port::scalar(PORT_COMMIT, "Bool"),
        Port::scalar(PORT_RESULT, TYPE_WORKFLOW_RESULT),
    ]
}

/// Verify a node carries the required workflow unit I/O contract.
pub fn has_required_unit_contract(inputs: &[Port], outputs: &[Port]) -> bool {
    let has_after = inputs.iter().any(|port| {
        port.name == PortName::from(PORT_AFTER) && port.type_id == TypeId::from("Bool")
    });
    let has_commit = outputs.iter().any(|port| {
        port.name == PortName::from(PORT_COMMIT) && port.type_id == TypeId::from("Bool")
    });
    let has_result = outputs.iter().any(|port| {
        port.name == PortName::from(PORT_RESULT)
            && port.type_id == TypeId::from(TYPE_WORKFLOW_RESULT)
    });
    has_after && has_commit && has_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::{Dag, Node};

    #[test]
    fn required_contract_helpers_include_after_commit_result() {
        let inputs = required_input_contract();
        let outputs = required_output_contract();
        assert!(has_required_unit_contract(&inputs, &outputs));
    }

    #[test]
    fn workflow_spec_constructor_round_trips_id() {
        let dag: Dag<WorkflowUnit> = Dag::new();
        let spec = WorkflowSpec::new("ci", dag, 7);
        assert_eq!(spec.id.0, "ci");
        assert_eq!(spec.policy_version, 7);
    }

    #[test]
    fn contract_rejects_missing_result_output() {
        let inputs = required_input_contract();
        let outputs = vec![Port::scalar(PORT_COMMIT, "Bool")];
        assert!(!has_required_unit_contract(&inputs, &outputs));
    }

    #[test]
    fn unit_contract_matches_node_ports() {
        let node = Node::opaque(
            "ci.codegen",
            required_input_contract(),
            required_output_contract(),
            WorkflowUnit::new(WorkflowOp::Aggregate(AggregateSpec::new("agg"))),
        );
        assert!(has_required_unit_contract(&node.inputs, &node.outputs));
    }
}
