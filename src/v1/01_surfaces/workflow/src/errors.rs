//! Workflow planner admission/schema errors.

use gunbc_ir::{AccessMode, NodeId, ResourceAccessError};
use std::fmt;

use crate::process_registry::{ClaimId, ProcessUnitRef, UnitClaim};

/// Fail-closed validation errors for workflow planner admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowAdmissionError {
    UnknownProcessUnit {
        node_id: NodeId,
        process_unit: ProcessUnitRef,
    },
    ResourceAccess(ResourceAccessError),
    MissingRequiredClaims {
        node_id: NodeId,
        process_unit: ProcessUnitRef,
        missing_claims: Vec<UnitClaim>,
    },
    UndeclaredEffectfulIo {
        node_id: NodeId,
        process_unit: ProcessUnitRef,
        missing_claim_ports: Vec<ClaimId>,
    },
    ConflictingClaims {
        left_node: NodeId,
        right_node: NodeId,
        left_claim: ClaimId,
        right_claim: ClaimId,
        left_mode: AccessMode,
        right_mode: AccessMode,
    },
}

impl fmt::Display for WorkflowAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowAdmissionError::UnknownProcessUnit {
                node_id,
                process_unit,
            } => write!(
                f,
                "node '{}' references unknown process unit '{}::{}'",
                node_id.0, process_unit.process_id.0, process_unit.unit_id.0
            ),
            WorkflowAdmissionError::ResourceAccess(error) => write!(f, "{error}"),
            WorkflowAdmissionError::MissingRequiredClaims {
                node_id,
                process_unit,
                missing_claims,
            } => {
                let claims = missing_claims
                    .iter()
                    .map(|claim| format!("{}:{:?}", claim.claim_id.0, claim.access_mode))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(
                    f,
                    "node '{}' missing required claims for process unit '{}::{}': {}",
                    node_id.0, process_unit.process_id.0, process_unit.unit_id.0, claims
                )
            }
            WorkflowAdmissionError::UndeclaredEffectfulIo {
                node_id,
                process_unit,
                missing_claim_ports,
            } => {
                let claims = missing_claim_ports
                    .iter()
                    .map(|claim| claim.0.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(
                    f,
                    "node '{}' missing declared resource claim ports for effectful process unit '{}::{}': {}",
                    node_id.0, process_unit.process_id.0, process_unit.unit_id.0, claims
                )
            }
            WorkflowAdmissionError::ConflictingClaims {
                left_node,
                right_node,
                left_claim,
                right_claim,
                left_mode,
                right_mode,
            } => write!(
                f,
                "unordered conflicting claims: node '{}' [{}:{:?}] vs node '{}' [{}:{:?}]",
                left_node.0, left_claim.0, left_mode, right_node.0, right_claim.0, right_mode
            ),
        }
    }
}

impl std::error::Error for WorkflowAdmissionError {}
