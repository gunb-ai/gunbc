//! Stage 2b idempotency lens — `src/v3/lenses/idempotency.dag` names the API.
//!
//! The v3 emitter cannot yet lower `match` on user-defined sums like
//! `std.effects::WorkflowEffect` inside lens modules; the algebraic walk is
//! implemented in [`crate::workflow_idempotency`]. Only [`analyze_workflow`] is
//! exported at the crate root — composition helpers stay `pub(crate)` there.
//!
//! **Emit-and-run receipt:** `tests/m2_lens_idempotency_emit_test.rs` loads the
//! `.dag` through `emit_rust_module` and asserts the emitted `analyze_workflow`
//! matches this oracle on a `WorkflowEffect` fixture (class-5 gap closure).

use crate::dag::{Dag, NodeId, WorkflowIdempotencyReport};

pub fn analyze_workflow(d: &Dag, workflow_root: NodeId) -> WorkflowIdempotencyReport {
    crate::workflow_idempotency::analyze_workflow(d, workflow_root)
}
