//! Stage 2e parallelism lens — `src/v3/lenses/parallelism.dag` names the API.
//!
//! The v3 emitter cannot yet lower `match` on user-defined sums like
//! `std.effects::WorkflowEffect` inside lens modules; the walk is implemented in
//! [`crate::workflow_parallelism`]. Only [`analyze_parallelism`] is exported at
//! the crate root.

use crate::dag::{Dag, NodeId, WorkflowParallelismReport};

pub fn analyze_parallelism(d: &Dag, workflow_root: NodeId) -> WorkflowParallelismReport {
    crate::workflow_parallelism::analyze_parallelism(d, workflow_root)
}
