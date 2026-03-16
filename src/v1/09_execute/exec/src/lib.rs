//! **Stage 9 — Execute**: Transforms a `Dag<DynOp>` + `ExecutionMode`
//! into an execution log and node outputs.
//!
//! # Pipeline position
//!
//! - **Before**: [`resolve`] has produced a `Dag<DynOp>` of executable ops
//! - **After**: caller consumes execution results (logs, outputs, progress)
//!
//! # Sequential steps
//!
//! 1. Flatten sub-DAGs into a single flat DAG (`lower`)
//! 2. Topologically sort nodes for execution order
//! 3. Execute each node by calling its `Executable::execute` impl
//! 4. In `DryRun` mode, intercept transport execution nodes with mocks
//! 5. Emit progress events and CI workflow commands (`CiContext`)
//! 6. Collect execution log entries and node outputs
//!
//! # Purity
//!
//! Delegates I/O to the transport layer. Pure nodes always execute;
//! transport executors perform real I/O (or are mocked in DryRun).
//!
//! # Failure
//!
//! Returns `ExecError` with layered diagnostics (transport, auth, shell,
//! HTTP, filesystem context).

#![deny(dead_code)]
pub mod box_draw;
pub mod ci_context;
pub mod diagnostic;
pub mod display;
pub mod env;
pub mod error;
pub mod execute;
pub mod frame_build;
pub mod frame_write;
pub mod freshness;
pub mod helpers;
pub mod intercept;
pub mod ledger;

pub mod lower;
pub mod pattern_op;
pub mod progress;
pub mod render;
pub mod terminal;
pub mod topo;

pub use box_draw::{error_box, info_box, preamble_box, BoxStyle, TermBox};
pub use ci_context::CiContext;
pub use diagnostic::{
    credential_as_key, rest_acquisition_diagnostic, rest_request_as_lock, AcquisitionDiagnostic,
    KeyIdentity, LockIdentity,
};
pub use display::{
    execute_and_display, execute_and_display_with_result, execute_and_display_with_result_config,
    print_attention, print_error_boxes, print_preamble, print_preamble_auto, print_value,
    AttentionLevel, DisplayConfig, DisplayMode, DisplayResult, DisplayVerbosity, Preamble,
};
pub use env::{single_output as env_single_output, EnvNode};
pub use error::{
    classify_layers, decorate_service_failure, AcquisitionErrorLayer, AuthContext, ErrorClass,
    ErrorLayer, ExecError, FailureDetail, FileErrorLayer, HttpErrorLayer, IntoExecResult, NodeRole,
    NodeTraceLayer, RestErrorLayer, ResultExt, ServiceCallMetadata, ServiceErrorLayer,
    ShellErrorLayer, TransportContext, TransportFailureKind, TransportFailureLayer,
};
pub use execute::{
    execute_dag, execute_single_node, DryRunStrictness, ExecuteConfig, ExecutionLog, ExecutionMode,
    LogEntry,
};
pub use freshness::{
    compose_with_freshness, run_freshness_step, run_freshness_steps, FreshnessStep, WithFreshness,
};
pub use helpers::{
    optional_bool, optional_bool_strict, optional_int, optional_int_strict, optional_json,
    optional_json_strict, optional_map_str_str, optional_map_str_str_strict,
    optional_response_strict, optional_str, optional_str_list, optional_str_list_strict,
    optional_str_strict, propagate_skipped, require_bool, require_int, require_json,
    require_map_str_str, require_request, require_response, require_str, require_str_list,
    require_value, InputsExt, OutputMap, TransportResponseExt,
};
pub use intercept::{BoundaryMock, BoundaryMocks};
pub use ledger::{ExecutionLedger, ExecutionRecord, RedundancyViolation};

pub use lower::{lower, LoopInfo, LowerError, LowerResult};
pub use progress::{
    ComposedObserver, DagPhase, DagProgress, DagSnapshot, EdgeProgress, EdgeState, ExecutionEvent,
    FieldKind, FieldSummary, GroupProgress, NodeProgress, NodeState, OutputSummary, ProgressEvent,
    ProgressObserver, RecordingObserver, StageGroup,
};
pub use render::{Animation, AnimationMode, RenderMode};
pub use topo::topo_sort;

pub use gunbc_ir::LogDetailLevel;
use gunbc_ir::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Trait that opaque node operations must implement.
pub trait Executable: fmt::Debug {
    /// Execute the operation with the given inputs.
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError>;
}

/// Type-erased executable operation.
///
/// Wraps any `Executable` impl for use in `Dag<DynOp>`, eliminating the need
/// for legacy per-module union enums in app crates.
///
/// Clone is cheap (Arc refcount bump). Satisfies `Executable + Clone + Send + 'static`.
#[derive(Clone)]
pub struct DynOp(Arc<dyn Executable + Send + Sync>);

impl DynOp {
    /// Wrap any `Executable` in a type-erased `DynOp`.
    pub fn new(op: impl Executable + Send + Sync + 'static) -> Self {
        Self(Arc::new(op))
    }
}

impl fmt::Debug for DynOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Executable for DynOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        self.0.execute(inputs)
    }
}

impl From<gunbc_ir::patterns::PatternOp> for DynOp {
    fn from(op: gunbc_ir::patterns::PatternOp) -> Self {
        DynOp::new(op)
    }
}
