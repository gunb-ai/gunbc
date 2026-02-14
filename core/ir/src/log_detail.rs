//! Execution log detail policy primitives shared by IR and executor.

use serde::{Deserialize, Serialize};

/// Execution log detail level.
///
/// This is used by the executor to decide whether node inputs should be
/// captured in [`gunbc_exec::LogEntry`]-style records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LogDetailLevel {
    /// Record node outputs and interception state only.
    #[default]
    Basic,
    /// Record node outputs and effective node inputs.
    IncludeInputs,
}

impl LogDetailLevel {
    /// Whether this level captures node inputs in execution logs.
    pub fn includes_inputs(self) -> bool {
        matches!(self, Self::IncludeInputs)
    }
}
