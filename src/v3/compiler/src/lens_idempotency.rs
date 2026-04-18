//! Stage 2b idempotency lens — `src/v3/lenses/idempotency.dag` names the API.
//!
//! The v3 emitter cannot yet lower `match` on user-defined sums like
//! `std.effects::WorkflowEffect` inside lens modules; the algebraic walk is
//! implemented in [`crate::workflow_idempotency`] and re-exported here.

use crate::dag::{Dag, NodeId, WorkflowIdempotencyReport};

pub fn analyze_workflow(d: &Dag, workflow_root: NodeId) -> WorkflowIdempotencyReport {
    crate::workflow_idempotency::analyze_workflow(d, workflow_root)
}
