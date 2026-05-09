//! Materialize [`CostBasisDeclaration`](crate::lens_cost_symbolic::CostBasisDeclaration) from
//! lowered [`Dag`](crate::dag::Dag) facts — interim **consumer boundary** for gate **#93**
//! (`crdt_cost_basis_demonstrated`) and the user-authored cost-basis audit.
//!
//! Persisted `.dag` `data` rows and folding declarations into [`compute_symbolic_costs`](crate::lens_cost_symbolic::compute_symbolic_costs) remain follow-on; embedders MUST NOT
//! hand-assemble `CostBasisDeclaration` beside this helper when deriving the same evidence
//! from a compiled DAG (INVARIANTS.md P2).

use std::fmt;

use crate::dag::{
    ArrowBody, BindNode, Dag, DeclarationId, Lookup, PortId, SizeVariable, SymbolicCost,
    TypeConnective,
};
use crate::lens_cost_symbolic::{compute_symbolic_costs, CostBasisDeclaration, CostBasisKind};

/// [`try_build_per_write_log_cost_basis_declaration`] stops on missing DAG facts instead of
/// panicking (fail-closed consumer boundary; codex gate on PR #2299).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostBasisDeclarationBuildError {
    DeclarationNotFound {
        name: String,
    },
    MergeStepMissingParams {
        merge_step_declaration_name: String,
    },
    /// Declaration must be a function (`TypeConnective::Arrow`) so the body bind is
    /// reachable via [`ArrowBody::UserDefined`] (single authority per P2).
    WorkflowDeclarationNotArrow {
        name: String,
    },
    /// Arrow declaration body is not yet lowered to `UserDefined(BindNodeId)` — cannot read
    /// the authoritative body [`BindNode`] for parameters / span / structural parity.
    WorkflowBodyNotUserDefinedBind {
        name: String,
    },
    /// `compute_symbolic_costs` has no `Hit(LogCost)` row keyed to the merge step’s first
    /// parameter port — cannot fabricate a per-write log basis without lens evidence of a
    /// divide on that port (P2 / fail-closed).
    MergeStepLacksLogCostWitness {
        merge_step_declaration_name: String,
    },
}

impl fmt::Display for CostBasisDeclarationBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CostBasisDeclarationBuildError::DeclarationNotFound { name } => {
                write!(f, "declaration `{name}` not found")
            }
            CostBasisDeclarationBuildError::MergeStepMissingParams {
                merge_step_declaration_name,
            } => write!(
                f,
                "declaration `{merge_step_declaration_name}` should declare at least one parameter on its body bind"
            ),
            CostBasisDeclarationBuildError::WorkflowDeclarationNotArrow { name } => write!(
                f,
                "declaration `{name}` must have Arrow connective (callable)"
            ),
            CostBasisDeclarationBuildError::WorkflowBodyNotUserDefinedBind { name } => write!(
                f,
                "declaration `{name}` body is not ArrowBody::UserDefined(BindNodeId); cannot resolve authoritative bind node"
            ),
            CostBasisDeclarationBuildError::MergeStepLacksLogCostWitness {
                merge_step_declaration_name,
            } => write!(
                f,
                "declaration `{merge_step_declaration_name}` first param port has no LogCost witness in `compute_symbolic_costs` (expected divide-on-dividend lowering)"
            ),
        }
    }
}

impl std::error::Error for CostBasisDeclarationBuildError {}

fn user_defined_arrow_body_bind<'a>(
    dag: &'a Dag,
    declaration_name: &str,
) -> Result<(&'a BindNode, DeclarationId), CostBasisDeclarationBuildError> {
    let decl = dag.declaration_by_name(declaration_name).ok_or_else(|| {
        CostBasisDeclarationBuildError::DeclarationNotFound {
            name: declaration_name.to_string(),
        }
    })?;
    let subject = decl.id;
    let bind = match &decl.connective {
        TypeConnective::Arrow { body, .. } => match body {
            ArrowBody::UserDefined(bind_id) => bind_id.bind_opt(dag).ok_or_else(|| {
                CostBasisDeclarationBuildError::WorkflowBodyNotUserDefinedBind {
                    name: declaration_name.to_string(),
                }
            })?,
            _ => {
                return Err(
                    CostBasisDeclarationBuildError::WorkflowBodyNotUserDefinedBind {
                        name: declaration_name.to_string(),
                    },
                );
            }
        },
        _ => {
            return Err(
                CostBasisDeclarationBuildError::WorkflowDeclarationNotArrow {
                    name: declaration_name.to_string(),
                },
            );
        }
    };
    Ok((bind, subject))
}

fn symbolic_table_includes_log_on_port(dag: &Dag, port: PortId) -> bool {
    compute_symbolic_costs(dag).iter().any(|e| {
        matches!(
            &e.cost,
            Lookup::Hit(SymbolicCost::LogCost { _0: sv }) if sv.source_port == port
        )
    })
}

/// **`LogCost` witness:** returns [`MergeStepLacksLogCostWitness`] unless
/// [`compute_symbolic_costs`] already **`Hit`s `LogCost(merge_param_port)`** (divide-on-dividend
/// lowering); does not fabricate basis without lens table evidence.
/// `merge_step_declaration_name` is the **`fn` declaration id** (same string the author uses for
/// the merge helper, e.g. `crdt_merge_step`); its body [`BindNode`] is read only via
/// [`TypeConnective::Arrow`] → [`ArrowBody::UserDefined`] like the workflow (P2 single authority
/// for both callables).
pub fn try_build_per_write_log_cost_basis_declaration(
    dag: &Dag,
    workflow_declaration_name: &str,
    merge_step_declaration_name: &str,
) -> Result<CostBasisDeclaration, CostBasisDeclarationBuildError> {
    let (workflow_bind, subject) = user_defined_arrow_body_bind(dag, workflow_declaration_name)?;
    let (merge_bind, _) = user_defined_arrow_body_bind(dag, merge_step_declaration_name)?;
    let merge_replicas_port = *merge_bind.params.first().ok_or_else(|| {
        CostBasisDeclarationBuildError::MergeStepMissingParams {
            merge_step_declaration_name: merge_step_declaration_name.to_string(),
        }
    })?;
    if !symbolic_table_includes_log_on_port(dag, merge_replicas_port) {
        return Err(
            CostBasisDeclarationBuildError::MergeStepLacksLogCostWitness {
                merge_step_declaration_name: merge_step_declaration_name.to_string(),
            },
        );
    }
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
