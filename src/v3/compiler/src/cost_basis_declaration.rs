//! Materialize [`CostBasisDeclaration`](crate::lens_cost_symbolic::CostBasisDeclaration) from
//! lowered [`Dag`](crate::dag::Dag) facts — interim **consumer boundary** for gate **#93**
//! (`crdt_cost_basis_demonstrated`) and the user-authored cost-basis audit.
//!
//! Persisted `.dag` `data` rows and folding declarations into [`compute_symbolic_costs`](crate::lens_cost_symbolic::compute_symbolic_costs) remain follow-on; embedders MUST NOT
//! hand-assemble `CostBasisDeclaration` beside this helper when deriving the same evidence
//! from a compiled DAG (INVARIANTS.md P2).

use std::fmt;

use crate::dag::{Behavior, BindNode, Dag, SizeVariable, SymbolicCost};
use crate::lens_cost_symbolic::{CostBasisDeclaration, CostBasisKind};

/// [`try_build_per_write_log_cost_basis_declaration`] stops on missing DAG facts instead of
/// panicking (fail-closed consumer boundary; codex gate on PR #2299).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostBasisDeclarationBuildError {
    BindNotFound { name: String },
    DeclarationNotFound { name: String },
    MergeStepMissingParams { merge_step_fn_bind_name: String },
}

impl fmt::Display for CostBasisDeclarationBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CostBasisDeclarationBuildError::BindNotFound { name } => {
                write!(f, "bind `{name}` not found")
            }
            CostBasisDeclarationBuildError::DeclarationNotFound { name } => {
                write!(f, "declaration `{name}` not found")
            }
            CostBasisDeclarationBuildError::MergeStepMissingParams {
                merge_step_fn_bind_name,
            } => write!(
                f,
                "`{merge_step_fn_bind_name}` should declare at least one parameter"
            ),
        }
    }
}

impl std::error::Error for CostBasisDeclarationBuildError {}

/// Builds a **PerWrite** basis declaration: **§4.2** `O(log replicas)` as [`SymbolicCost::LogCost`]
/// on the **first parameter port** of `merge_step_fn_bind_name` (dividend port for `replicas / k`
/// inside `crdt_merge_step` today), with **`subject`** the [`DeclarationId`](crate::dag::DeclarationId) of
/// `workflow_fn_bind_name` and **`span`** from that workflow bind.
pub fn try_build_per_write_log_cost_basis_declaration(
    dag: &Dag,
    workflow_fn_bind_name: &str,
    merge_step_fn_bind_name: &str,
) -> Result<CostBasisDeclaration, CostBasisDeclarationBuildError> {
    let find_bind = |name: &str| -> Result<&BindNode, CostBasisDeclarationBuildError> {
        dag.nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .find(|b| b.name == name)
            .ok_or_else(|| CostBasisDeclarationBuildError::BindNotFound {
                name: name.to_string(),
            })
    };
    let workflow_bind = find_bind(workflow_fn_bind_name)?;
    let merge_bind = find_bind(merge_step_fn_bind_name)?;
    let merge_replicas_port = *merge_bind.params.first().ok_or_else(|| {
        CostBasisDeclarationBuildError::MergeStepMissingParams {
            merge_step_fn_bind_name: merge_step_fn_bind_name.to_string(),
        }
    })?;
    let subject = dag
        .declaration_by_name(workflow_fn_bind_name)
        .ok_or_else(|| CostBasisDeclarationBuildError::DeclarationNotFound {
            name: workflow_fn_bind_name.to_string(),
        })?
        .id;
    Ok(CostBasisDeclaration {
        subject,
        kind: CostBasisKind::PerWrite,
        cost: SymbolicCost::LogCost {
            _0: SizeVariable {
                source_port: merge_replicas_port,
                display_name: None,
            },
        },
        span: workflow_bind.span.clone(),
    })
}
