//! Stage 2b idempotency lens — `src/v3/lenses/idempotency.dag` names the API.
//!
//! The v3 emitter cannot yet lower `match` on user-defined sums like
//! `std.effects::WorkflowEffect` inside lens modules; the algebraic walk is
//! implemented in [`crate::workflow_idempotency`]. Only [`analyze_workflow`] is
//! exported at the crate root — composition helpers stay `pub(crate)` there.
//!
//! **Emit receipt:** `tests/m2_lens_idempotency_emit_test.rs` compiles this `.dag`
//! and asserts `emit_rust_module` / `emit_go_module` / `emit_python_module`
//! succeed with the expected surface (`analyze_workflow`, host read). A rustc
//! link-and-run equivalence check against this re-export awaits emitter fixes for
//! imported `std.effects` helpers (see that test module’s header).

use crate::dag::{Dag, NodeId, WorkflowIdempotencyReport};

pub fn analyze_workflow(d: &Dag, workflow_root: NodeId) -> WorkflowIdempotencyReport {
    crate::workflow_idempotency::analyze_workflow(d, workflow_root)
}
