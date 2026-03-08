use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Serialize;

// ── Verdict: the one result type for all pipeline stages ──────────────

/// Every stage API returns `Verdict<T>`. PASS = `Ok(T)`, FAIL = `Err(Diagnostics)`.
/// No third state. No "continue with partial."
pub type Verdict<T> = Result<T, Diagnostics>;

/// A collection of diagnostics. Non-empty `errors` means FAIL.
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    pub errors: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn single(diag: Diagnostic) -> Self {
        Self { errors: vec![diag] }
    }

    pub fn push(&mut self, diag: Diagnostic) {
        self.errors.push(diag);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn merge(&mut self, other: Diagnostics) {
        self.errors.extend(other.errors);
    }

    /// Convert to Verdict: Ok(()) if no errors, Err(self) otherwise.
    pub fn into_result(self) -> Verdict<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl std::fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, diag) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{diag}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}

impl From<Diagnostic> for Diagnostics {
    fn from(diag: Diagnostic) -> Self {
        Self::single(diag)
    }
}

// ── Diagnostic: every error answers what, where, and how to fix ───────

/// A single diagnostic. Every user-facing error must carry a span (source location).
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Machine-readable error code (e.g. "PAR001", "TC014", "LOW003").
    pub code: &'static str,
    /// Human-readable message describing the contradiction.
    pub message: String,
    /// Source location. Not optional for user-facing errors.
    pub span: Option<Span>,
    /// Which `.dag` file.
    pub file: Option<PathBuf>,
    /// 1-based line number (derived from span + source text).
    pub line: Option<usize>,
    /// 1-based column number (derived from span + source text).
    pub column: Option<usize>,
    /// Structured context for programmatic handling.
    pub context: DiagnosticContext,
    /// Concrete suggestion for resolving the contradiction.
    pub help: Option<String>,
    /// Secondary source locations (e.g. "first defined here").
    pub related: Vec<RelatedSpan>,
}

impl Diagnostic {
    /// Create a located diagnostic — the preferred constructor.
    ///
    /// Source location is mandatory. Use this for all user-facing diagnostics.
    pub fn located(code: &'static str, message: impl Into<String>, primary: LocatedSpan) -> Self {
        Self {
            code,
            message: message.into(),
            span: Some(primary.span),
            file: None, // FileId-based resolution deferred to Phase 2 (FileTable)
            line: None,
            column: None,
            context: DiagnosticContext::Note(String::new()),
            help: None,
            related: Vec::new(),
        }
    }

    /// Create a diagnostic without source location.
    ///
    /// Prefer `Diagnostic::located()` which enforces source location.
    /// Call sites should be updated to provide spans.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            span: None,
            file: None,
            line: None,
            column: None,
            context: DiagnosticContext::Note(String::new()),
            help: None,
            related: Vec::new(),
        }
    }

    /// Set resolved line and column (1-based).
    pub fn with_line_col(mut self, line: usize, col: usize) -> Self {
        self.line = Some(line);
        self.column = Some(col);
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_file(mut self, file: PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_context(mut self, context: DiagnosticContext) -> Self {
        self.context = context;
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_related(mut self, related: Vec<RelatedSpan>) -> Self {
        self.related = related;
        self
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // [CODE] file:line:col: message
        write!(f, "[{}]", self.code)?;
        if let Some(file) = &self.file {
            write!(f, " {}", file.display())?;
            if let (Some(line), Some(col)) = (self.line, self.column) {
                write!(f, ":{line}:{col}")?;
            } else if let Some(span) = &self.span {
                write!(f, ":{}", span.start)?;
            }
        }
        write!(f, ": {}", self.message)?;
        if let Some(help) = &self.help {
            write!(f, "\n  help: {help}")?;
        }
        Ok(())
    }
}

/// Structured context so tooling can act on errors programmatically.
#[derive(Debug, Clone)]
pub enum DiagnosticContext {
    /// Type mismatch: expected X, got Y.
    TypeMismatch { expected: String, got: String },
    /// Missing required item (import, arg, port, field).
    Missing {
        kind: &'static str,
        name: String,
        available: Vec<String>,
    },
    /// Duplicate definition.
    Duplicate { name: String, first: Option<Span> },
    /// Unsupported feature (NYI diagnostic).
    Unsupported { feature: String },
    /// Generic note (escape hatch for rare cases).
    Note(String),
}

/// Role of a secondary span in a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanRole {
    /// "first defined here"
    Definition,
    /// "conflicts with this"
    Conflict,
    /// "referenced here"
    Related,
}

/// A span with mandatory file and label — the primary location of a diagnostic.
#[derive(Debug, Clone)]
pub struct LocatedSpan {
    pub file: FileId,
    pub span: Span,
    pub label: String,
}

/// A secondary span with role annotation.
#[derive(Debug, Clone)]
pub struct LabeledSpan {
    pub file: FileId,
    pub span: Span,
    pub label: String,
    pub role: SpanRole,
}

/// A secondary source location referenced by a diagnostic.
#[derive(Debug, Clone)]
pub struct RelatedSpan {
    pub span: Span,
    pub file: Option<PathBuf>,
    pub label: String,
    pub role: SpanRole,
}

// ── Span: source location (byte offset range) ────────────────────────

/// Byte offset range in a source file. This is the contract-level span
/// shared across all pipeline stages. Stage-specific crates may define
/// richer span types that convert to/from this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end && self.start == 0
    }
}

/// Convert a byte offset into a 1-based (line, column) pair.
///
/// Offsets beyond EOF are clamped to EOF. This is a pure function with no
/// dependencies, shared across all pipeline stages that need to resolve
/// byte spans to human-readable locations.
pub fn byte_to_line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let clamped = byte_offset.min(source.len());
    let mut line = 1usize;
    let mut col = 1usize;
    for (idx, ch) in source.char_indices() {
        if idx >= clamped {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

// ── Interned identity types ───────────────────────────────────────────
// Each stage introduces IDs for its domain. Later stages carry IDs, not strings.

macro_rules! interned_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub u32);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

interned_id!(
    FileId,
    "Stable file identity within one compilation (Stage 0: Ingest)."
);
interned_id!(
    ModuleId,
    "Index into ModuleGraph.modules (Stage 2: Module Resolve)."
);
interned_id!(DefId, "Any named definition (Stage 3: Typecheck).");
interned_id!(
    CallableId,
    "fn/func/pattern (Stage 3: Typecheck)."
);
interned_id!(ServiceOpId, "Service operation (Stage 3: Typecheck).");
interned_id!(TypeId, "Type arena entry (Stage 3: Typecheck).");
interned_id!(DataId, "Evaluated data declaration (Stage 4: Lower).");
interned_id!(
    PatternInstanceId,
    "Expanded pattern — triplet, chain, loop (Stage 4: Lower)."
);

// ── Manifest + obligation contract types ──────────────────────────────

/// Progress-manifest contract derived from lowered DAG topology.
///
/// The stable JSON contract includes: `schema_version`, `total_nodes`, `topology`,
/// `labels`, `subdag_boundaries`, `parallel_groups`, `scatter_points`,
/// `interactive_nodes`, `capture_modes`, `stage_groups`, `resources`.
///
/// Fields marked `skip_serializing` (`total_edges`, `waves`, `entrypoint_nodes`,
/// `boundary_nodes`) are used internally by text renderers and emit but are not
/// part of the stable JSON contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProgressManifest {
    /// Schema version for forward-compatible evolution. Bump when adding or
    /// removing serialized fields.
    pub schema_version: u32,
    pub total_nodes: usize,
    #[serde(skip_serializing)]
    pub total_edges: usize,
    #[serde(skip_serializing)]
    pub waves: Vec<Vec<String>>,
    #[serde(skip_serializing)]
    pub entrypoint_nodes: Vec<String>,
    #[serde(skip_serializing)]
    pub boundary_nodes: Vec<String>,
    pub topology: Vec<TopologyNode>,
    pub labels: BTreeMap<String, String>,
    pub subdag_boundaries: Vec<SubDagBoundary>,
    pub parallel_groups: Vec<ParallelGroup>,
    pub scatter_points: Vec<String>,
    pub interactive_nodes: Vec<String>,
    pub capture_modes: BTreeMap<String, CaptureMode>,
    pub stage_groups: Vec<StageGroup>,
    pub resources: BTreeMap<String, Vec<ResourceUsage>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TopologyNode {
    pub id: String,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubDagBoundary {
    pub node_id: String,
    pub label: String,
    pub inner_nodes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParallelGroup {
    pub nodes: Vec<String>,
    pub depth: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_subdag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode {
    #[default]
    Captured,
    Passthrough,
    Streamed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StageGroup {
    pub stage_id: String,
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResourceUsage {
    pub resource: String,
    pub usage: String,
}

/// Test obligation counters derived from DAG topology.
///
/// ## Counter model
///
/// **Top-level (disjoint node buckets — sum equals `total_obligations`):**
/// - `transport_execution_targets`: nodes that accept a `TransportRequest` input
/// - `pure_node_determinism_targets`: all other nodes
///
/// **Obligation-category counters (per `ObligationCategory` match):**
/// - `service_transport_prepare_targets`, `service_transport_execute_targets`,
///   `service_transport_parse_targets`, `service_param_source_targets`,
///   `resource_provide_targets`, `resource_acquire_targets`,
///   `resource_release_targets`, `interface_contract_verification_targets`
///
/// **Semantic attributes on `ServiceTransportExecute` nodes:**
/// - `hermetic` vs `external` — *mutually exclusive*
/// - `idempotent`, `readonly` — *independent overlays*
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestObligations {
    pub dry_run_completion_required: bool,
    /// Sum of `transport_execution_targets + pure_node_determinism_targets`.
    /// These two buckets are disjoint and together cover every node in the DAG.
    pub total_obligations: usize,
    pub transport_execution_targets: usize,
    pub pure_node_determinism_targets: usize,
    pub service_transport_prepare_targets: usize,
    pub service_transport_execute_targets: usize,
    pub service_transport_parse_targets: usize,
    /// Mutually exclusive with `service_transport_external_targets`.
    pub service_transport_hermetic_targets: usize,
    /// Mutually exclusive with `service_transport_hermetic_targets`.
    pub service_transport_external_targets: usize,
    /// Independent attribute overlay (can combine with hermetic/external).
    pub service_transport_idempotent_targets: usize,
    /// Independent attribute overlay (can combine with hermetic/external).
    pub service_transport_readonly_targets: usize,
    pub service_param_source_targets: usize,
    pub resource_provide_targets: usize,
    pub resource_acquire_targets: usize,
    pub resource_release_targets: usize,
    pub interface_contract_verification_targets: usize,
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_diagnostic(code: &'static str, message: &str) -> Diagnostic {
        Diagnostic::located(
            code,
            message,
            LocatedSpan {
                file: FileId(0),
                span: Span::new(0, 1),
                label: "test".to_string(),
            },
        )
    }

    #[test]
    fn verdict_ok_is_pass() {
        let v: Verdict<i32> = make_verdict_ok(42);
        assert!(v.is_ok());
    }

    fn make_verdict_ok(n: i32) -> Verdict<i32> {
        Ok(n)
    }

    #[test]
    fn verdict_err_is_fail() {
        let diags = Diagnostics::single(test_diagnostic("TEST001", "test error"));
        assert_eq!(diags.errors.len(), 1);
        assert_eq!(diags.errors[0].code, "TEST001");
        let v: Verdict<i32> = Err(diags);
        assert!(v.is_err());
    }

    #[test]
    fn diagnostics_into_result_empty_is_ok() {
        let d = Diagnostics::new();
        assert!(d.into_result().is_ok());
    }

    #[test]
    fn diagnostics_into_result_nonempty_is_err() {
        let mut d = Diagnostics::new();
        d.push(test_diagnostic("E001", "bad"));
        assert!(d.into_result().is_err());
    }

    #[test]
    fn diagnostic_display_with_file_and_span() {
        let d = test_diagnostic("TC014", "type mismatch: expected String, got Int")
            .with_file(PathBuf::from("tools/gist.dag"))
            .with_span(Span::new(42, 55))
            .with_help("change argument type to String");
        let s = d.to_string();
        assert!(s.contains("[TC014]"));
        assert!(s.contains("tools/gist.dag"));
        assert!(s.contains("type mismatch"));
        assert!(s.contains("help:"));
    }

    #[test]
    fn diagnostics_merge_combines_errors() {
        let mut a = Diagnostics::single(test_diagnostic("E001", "first"));
        let b = Diagnostics::single(test_diagnostic("E002", "second"));
        a.merge(b);
        assert_eq!(a.errors.len(), 2);
    }

    #[test]
    fn diagnostic_located_requires_span() {
        let primary = LocatedSpan {
            file: FileId(0),
            span: Span::new(10, 20),
            label: "here".to_string(),
        };
        let d = Diagnostic::located("TC001", "type mismatch", primary);
        assert_eq!(d.code, "TC001");
        assert!(d.span.is_some());
        assert_eq!(d.span.unwrap().start, 10);
    }

    #[test]
    fn labeled_span_has_role() {
        let ls = LabeledSpan {
            file: FileId(0),
            span: Span::new(5, 15),
            label: "first defined here".to_string(),
            role: SpanRole::Definition,
        };
        assert_eq!(ls.role, SpanRole::Definition);
    }

    #[test]
    fn interned_ids_are_distinct_types() {
        let f = FileId(0);
        let m = ModuleId(0);
        // These are different types even with the same inner value
        assert_eq!(format!("{f}"), "FileId(0)");
        assert_eq!(format!("{m}"), "ModuleId(0)");
    }

    #[test]
    fn span_empty_is_zero() {
        let s = Span::empty();
        assert!(s.is_empty());
        assert_eq!(s.start, 0);
        assert_eq!(s.end, 0);
    }
}
