//! Materialize [`CostBasisDeclaration`](crate::lens_cost_symbolic::CostBasisDeclaration) from
//! lowered [`Dag`](crate::dag::Dag) facts — interim **consumer boundary** for gate **#93**
//! (`crdt_cost_basis_demonstrated`) and the user-authored cost-basis audit.
//!
//! Persisted `.dag` `data` rows and folding declarations into [`compute_symbolic_costs`](crate::lens_cost_symbolic::compute_symbolic_costs) remain follow-on; embedders MUST NOT
//! hand-assemble `CostBasisDeclaration` beside this helper when deriving the same evidence
//! from a compiled DAG (INVARIANTS.md P2).

use crate::dag::{Behavior, Dag, SizeVariable, SymbolicCost};
use crate::lens_cost_symbolic::{CostBasisDeclaration, CostBasisKind};

/// Builds a **PerWrite** basis declaration: **§4.2** `O(log replicas)` as [`SymbolicCost::LogCost`]
/// on the **first parameter port** of `merge_step_fn_bind_name` (dividend port for `replicas / k`
/// inside `crdt_merge_step` today), with **`subject`** the [`DeclarationId`](crate::dag::DeclarationId) of
/// `workflow_fn_bind_name` and **`span`** from that workflow bind.
pub fn build_per_write_log_cost_basis_declaration(
    dag: &Dag,
    workflow_fn_bind_name: &str,
    merge_step_fn_bind_name: &str,
) -> CostBasisDeclaration {
    let find_bind = |name: &str| {
        dag.nodes()
            .iter()
            .filter_map(Behavior::as_bind)
            .find(|b| b.name == name)
            .unwrap_or_else(|| panic!("bind `{name}` not found"))
    };
    let workflow_bind = find_bind(workflow_fn_bind_name);
    let merge_bind = find_bind(merge_step_fn_bind_name);
    let merge_replicas_port = *merge_bind.params.first().unwrap_or_else(|| {
        panic!("`{merge_step_fn_bind_name}` should declare at least one parameter")
    });
    let subject = dag
        .declaration_by_name(workflow_fn_bind_name)
        .unwrap_or_else(|| panic!("declaration `{workflow_fn_bind_name}` not found"))
        .id;
    CostBasisDeclaration {
        subject,
        kind: CostBasisKind::PerWrite,
        cost: SymbolicCost::LogCost {
            _0: SizeVariable {
                source_port: merge_replicas_port,
                display_name: None,
            },
        },
        span: workflow_bind.span.clone(),
    }
}
