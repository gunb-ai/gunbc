//! Stage 2e parallelism lens — `src/v3/lenses/parallelism.dag` names the API.
//!
//! The Stage 2e walk is still implemented in [`crate::workflow_parallelism`]
//! while `src/v3/lenses/parallelism.dag` remains a `LensSurfacePending` stub.
//! `emit_rust_module` can lower `match` on imported user sums such as
//! `WorkflowEffect` in lens modules; follow-on work is to port this analysis
//! into `.dag` / `std.effects` and rewire the lens like `idempotency.dag`.
//! Only [`analyze_parallelism`] is exported at the crate root.

use crate::dag::{Dag, NodeId, WorkflowParallelismReport};

pub fn analyze_parallelism(d: &Dag, workflow_root: NodeId) -> WorkflowParallelismReport {
    crate::workflow_parallelism::analyze_parallelism(d, workflow_root)
}
