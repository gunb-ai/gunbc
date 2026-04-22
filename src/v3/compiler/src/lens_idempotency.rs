//! Stage 2b idempotency lens — `src/v3/lenses/idempotency.dag` names the API.
//!
//! The behavioral walk over `WorkflowEffect` is authored in
//! `src/v3/std/effects.dag` (`lane2_workflow_idempotency_report`); this module
//! re-exports [`analyze_workflow`] over native `Dag` for the same projection as
//! the emitted `lenses/idempotency.dag` surface (`workflow_idempotency`).
//! Only [`analyze_workflow`] is exported at the crate root — composition helpers
//! stay `pub(crate)` in [`crate::workflow_idempotency`].
//!
//! **Emit receipt:** `tests/m2_lens_idempotency_emit_test.rs` compiles this `.dag`
//! and asserts multi-target emission succeeds; `tests/m2_lens_idempotency_migration_test.rs`
//! rustc-links the emitted Rust module and checks `analyze_workflow` matches this
//! re-export (declared lens surface + `lane2_workflow_effect_at` path).

use crate::dag::{Dag, NodeId, WorkflowIdempotencyReport};

pub fn analyze_workflow(d: &Dag, workflow_root: NodeId) -> WorkflowIdempotencyReport {
    crate::workflow_idempotency::analyze_workflow(d, workflow_root)
}
