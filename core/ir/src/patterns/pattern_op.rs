//! Internal operations used by pattern builders.
//!
//! These ops are emitted by pattern builders (Branch/Loop/Repeat/etc.) to avoid
//! relying on `T::default()` for internal nodes. Consumers that want to use
//! these patterns should ensure their operation type can be constructed from
//! `PatternOp` (e.g., via `From<PatternOp>` in a composed enum).

use super::repeat::{FailureClassifier, RepeatPolicy};
use std::time::Duration;

/// Internal operation variants for pattern-generated nodes.
#[derive(Debug, Clone)]
pub enum PatternOp {
    /// Merge results from a Branch true/false subdag.
    BranchMerge { output_port: String },
    /// Unpack a list input for Loop patterns.
    LoopUnpack { input_port: String, element_port: String },
    /// Pack Loop results back into a list.
    LoopPack { output_port: String },
    /// Retry controller (single-pass semantics; carries policy/classifier).
    RetryController {
        input_port: String,
        policy: RepeatPolicy,
        classifier: FailureClassifier,
    },
    /// Retry result collector.
    RetryCollector { output_port: String },
    /// While init passthrough for loop-carried state.
    WhileInit { input_port: String },
    /// While controller (single-pass semantics; carries max_iterations).
    WhileController { max_iterations: Option<usize> },
    /// Poll timer (single-pass semantics; carries interval/timeout).
    PollTimer {
        input_port: String,
        interval: Duration,
        timeout: Duration,
    },
    /// Poll result collector.
    PollCollector { output_port: String },
}

impl From<PatternOp> for PatternOp {
    fn from(op: PatternOp) -> Self {
        op
    }
}
