//! gunbc-exec: Execution engine for gunbc DAGs.
//!
//! This crate provides:
//! - [`Executable`]: Trait for operations that can be executed
//! - [`execute`]: Execute a DAG in real mode
//! - [`execute_with_mode`]: Execute with dry-run interception at transport nodes
//! - [`lower`]: Flatten sub-DAGs into a single flat DAG
//! - [`CiContext`]: Runtime CI context for emitting workflow commands
//!
//! # Dry-Run via Transport Interception
//!
//! Dry-run is not a flag threaded through operations. It's an execution mode
//! that intercepts **transport execution nodes** - nodes that consume
//! `TransportRequest` values. This follows the design principle:
//!
//! > "World I/O is performed only by transport executor nodes"
//! > "DryRun intercepts transport execution nodes, not boundary outputs"
//!
//! This ensures:
//! - Pure nodes always execute (they can't do I/O)
//! - Transport executors are replaced with mocks
//! - Boundaries are just interface definitions, not interception points
//!
//! # CI Context
//!
//! When executing in a CI environment (GitHub Actions, GitLab CI, etc.),
//! [`CiContext`] automatically emits workflow commands to create collapsible
//! log groups around DAG nodes, emit annotations for errors, etc.

pub mod ci_context;
pub mod error;
pub mod execute;
pub mod helpers;
pub mod intercept;
pub mod lower;
pub mod pattern_op;
pub mod progress;
pub mod render;
pub mod topo;

pub use ci_context::CiContext;
pub use error::{ExecError, ResultExt};
pub use execute::{
    execute, execute_single_node, execute_with_ci, execute_with_mode, execute_with_mode_and_ci,
    execute_with_progress, execute_with_progress_and_mode, execute_with_all,
    ExecutionLog, ExecutionMode, LogEntry,
};
pub use helpers::{
    optional_bool, optional_json, optional_map_str_str, optional_str, optional_str_list,
    propagate_skipped, require_bool, require_int, require_json, require_map_str_str,
    require_request, require_response, require_str, require_str_list, require_value,
    OutputMap, TransportResponseExt,
};
pub use intercept::{BoundaryMock, BoundaryMocks};
pub use lower::{lower, LowerError};
pub use progress::{
    DagProgress, DagSnapshot, EdgeProgress, EdgeState, FieldKind, FieldSummary, NodeProgress,
    NodeState, OutputSummary, ProgressEvent, ProgressObserver, RecordingObserver, DagPhase,
};
pub use render::{
    Animation, AnimationMode, FrameLoop, FramePolicy, RenderMode, TerminalRenderer,
};
pub use topo::topo_sort;

use gunbc_ir::Value;
use std::collections::HashMap;
use std::fmt;

/// Trait that opaque node operations must implement.
pub trait Executable: fmt::Debug {
    /// Execute the operation with the given inputs.
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError>;
}
