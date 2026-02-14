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

#![deny(dead_code)]
pub mod box_draw;
pub mod ci_context;
pub mod display;
pub mod env;
pub mod error;
pub mod execute;
pub mod frame_build;
pub mod frame_write;
pub mod freshness;
pub mod helpers;
pub mod intercept;
pub mod lint_guard;
pub mod lower;
pub mod pattern_op;
pub mod progress;
pub mod render;
pub mod terminal;
pub mod topo;

pub use box_draw::{error_box, info_box, preamble_box, BoxStyle, TermBox};
pub use ci_context::CiContext;
pub use display::{
    execute_and_display, execute_and_display_with_result, print_attention, print_error_boxes,
    print_preamble, print_preamble_auto, print_value, AttentionLevel, DisplayResult, Preamble,
};
pub use env::{single_output as env_single_output, EnvNode};
pub use error::{ExecError, IntoExecResult, ResultExt};
pub use execute::{
    execute, execute_single_node, execute_with_mode, execute_with_mode_and_inputs,
    execute_with_mode_and_inputs_and_detail, execute_with_progress, execute_with_progress_and_mode,
    execute_with_progress_and_mode_and_detail, execute_with_progress_and_mode_and_inputs,
    execute_with_progress_and_mode_and_inputs_and_detail, ExecutionLog, ExecutionMode, LogEntry,
};
pub use freshness::{compose_with_freshness, FreshnessStep, WithFreshness};
pub use helpers::{
    optional_bool, optional_bool_strict, optional_int, optional_int_strict, optional_json,
    optional_json_strict, optional_map_str_str, optional_map_str_str_strict,
    optional_response_strict, optional_str, optional_str_list, optional_str_list_strict,
    optional_str_strict, propagate_skipped, require_bool, require_int, require_json,
    require_map_str_str, require_request, require_response, require_str, require_str_list,
    require_value, InputsExt, OutputMap, TransportResponseExt,
};
pub use intercept::{BoundaryMock, BoundaryMocks};
pub use lint_guard::inject_lint_guard;
pub use lower::{lower, LoopInfo, LowerError, LowerResult};
pub use progress::{
    ComposedObserver, DagPhase, DagProgress, DagSnapshot, EdgeProgress, EdgeState, FieldKind,
    FieldSummary, GroupProgress, NodeProgress, NodeState, OutputSummary, ProgressEvent,
    ProgressObserver, RecordingObserver, StageGroup,
};
pub use render::{Animation, AnimationMode, RenderMode};
pub use topo::topo_sort;

pub use gunbc_ir::LogDetailLevel;
use gunbc_ir::Value;
use std::collections::HashMap;
use std::fmt;

/// Trait that opaque node operations must implement.
pub trait Executable: fmt::Debug {
    /// Execute the operation with the given inputs.
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError>;
}
