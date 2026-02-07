//! Generic CLI tool abstraction.
//!
//! This module provides a universal pattern for CLI tool upsert:
//! 1. **Check**: Verify the tool is installed
//! 2. **Install**: Install the tool if needed
//! 3. **Run**: Execute the tool with arguments
//!
//! Tools are defined declaratively via `CliToolDef`, and operations
//! are uniform via `CliToolOp`. This eliminates boilerplate when
//! adding new CLI tools to the system.
//!
//! # Fractal DAG Pattern
//!
//! Use `build_cli_upsert()` to create a sub-DAG node that implements
//! the upsert pattern. This integrates with the codebase's fractal DAG
//! approach where DAGs can contain sub-DAGs.
//!
//! ```ignore
//! use gunbc_ir::transport::cli::{build_cli_upsert, CLIPPY};
//!
//! // Build a sub-DAG node for clippy upsert
//! let clippy_node = build_cli_upsert(&CLIPPY, &["--all-targets"]);
//!
//! // This node can be composed into larger DAGs
//! ```
//!
//! # Direct Operations
//!
//! Direct execution lives in the transport layer. Use `build_cli_upsert()` and
//! execute via DAG nodes, or call transport-layer helpers when you need an
//! imperative check/install/run at the I/O boundary.

use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::resource::{
    capability_marker, ensure_capability_marker, AccessMode, ContentHash, Resource, ResourceHandle,
    ResourceId, ResourceKind,
};

/// Definition of a CLI tool for the upsert pattern.
///
/// Tools are defined as static data, making them easy to declare
/// and compose. The three commands map to the upsert phases:
/// - `check_cmd`: Verify existence (typically `tool --version`)
/// - `install_cmd`: Install if missing (optional - some tools require manual install)
/// - `run_cmd`: Base command to execute (args appended)
///
/// Tools can also define their resource access patterns (exclusivity),
/// which consumers don't need to know about.
#[derive(Debug, Clone, Copy)]
pub struct CliToolDef {
    /// Unique identifier for this tool
    pub id: &'static str,

    /// Command to check if tool is installed.
    /// Typically `["tool", "--version"]` or similar.
    pub check_cmd: &'static [&'static str],

    /// Command to install the tool, if automatic installation is supported.
    /// None means the tool must be installed manually.
    pub install_cmd: Option<&'static [&'static str]>,

    /// Base command to run the tool. Arguments are appended to this.
    pub run_cmd: &'static [&'static str],

    /// Human-readable description
    pub description: &'static str,

    /// Resource access mode for this tool (default: Read = parallel-safe).
    /// Tools that modify global state should use Exclusive.
    /// Consumers don't need to know this - the framework handles scheduling.
    pub access_mode: AccessMode,
}

impl CliToolDef {
    /// Get the resource ID for this tool.
    pub fn resource_id(&self) -> ResourceId {
        ResourceId::tool(self.id)
    }
}

// ============================================================================
// ToolHandle - Capability-Based Access
// ============================================================================

/// Marker type for tool resources (used with `ResourceHandle`).
#[derive(Debug, Clone, Copy)]
pub struct ToolResource;

/// A handle to an acquired tool. This is the ONLY way to run tool commands.
///
/// You cannot construct this directly - it only comes from tool acquisition
/// via the framework. This ensures the upsert pattern is always followed.
///
/// # Design
///
/// The capability pattern enforces that:
/// 1. You cannot use a tool without acquiring it first
/// 2. Acquisition happens via an environment node that provides resources
/// 3. The handle carries the resolved path to the tool binary
/// 4. The handle wraps a `ResourceHandle<ToolResource>` for unified resource tracking
///
/// # Example
///
/// ```ignore
/// // In operation implementation - receives handle from inputs
/// fn execute_lint(inputs: HashMap<String, Value>) -> Result<...> {
///     let clippy: ToolHandle = inputs.get("tool:clippy").unwrap();
///     // Execution happens at the transport boundary (e.g., via a DAG node)
///     // ...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ToolHandle {
    tool: &'static CliToolDef,
    /// Resolved path to the tool binary (e.g., "/usr/bin/cargo").
    /// For mocked handles, this may be a placeholder path.
    path: PathBuf,
    /// Unified resource handle (freshness proof + capability marker).
    resource_handle: ResourceHandle<ToolResource>,
    /// Private field prevents direct construction outside this module.
    _acquired: PhantomData<()>,
}

impl ToolHandle {
    /// Create a new ToolHandle with a resolved path.
    ///
    /// **Framework use only.** This should only be called by the execution
    /// framework (specifically, the environment node) after successfully
    /// acquiring a tool. User operation code should receive ToolHandle
    /// values through DAG inputs, not construct them directly.
    pub fn acquire(tool: &'static CliToolDef, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let resource_handle = tool_resource_handle(tool, &path);
        Self {
            tool,
            path,
            resource_handle,
            _acquired: PhantomData,
        }
    }

    /// Create a mock ToolHandle for testing/DryRun mode.
    ///
    /// The path will be `/mock/{tool_id}` to make it obvious this is not real.
    pub fn mock(tool: &'static CliToolDef) -> Self {
        let path = PathBuf::from(format!("/mock/{}", tool.id));
        let resource_handle = tool_resource_handle(tool, &path);
        Self {
            tool,
            path,
            resource_handle,
            _acquired: PhantomData,
        }
    }

    /// Get the tool definition this handle refers to.
    pub fn tool(&self) -> &'static CliToolDef {
        self.tool
    }

    /// Get the tool ID.
    pub fn id(&self) -> &'static str {
        self.tool.id
    }

    /// Get the resolved path to the tool binary.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Access the underlying unified resource handle.
    pub fn resource_handle(&self) -> &ResourceHandle<ToolResource> {
        &self.resource_handle
    }

    /// Get the freshness key for this tool handle.
    pub fn key(&self) -> &ContentHash {
        self.resource_handle.key()
    }

    /// Run the tool with the given arguments.
    /// This is the ONLY way to execute a tool - you need the handle.
    pub fn run(&self, args: &[&str]) -> CliToolOp {
        CliToolOp::run(self.tool, args)
    }

    /// Get the resource ID for this tool handle.
    pub fn resource_id(&self) -> ResourceId {
        self.resource_handle.resource_id().clone()
    }
}

/// Convert a ToolHandle to a Value for passing through DAG edges.
impl From<ToolHandle> for crate::Value {
    fn from(handle: ToolHandle) -> Self {
        let mut map = BTreeMap::new();
        map.insert(
            "type".to_string(),
            crate::Value::Str("tool_handle".to_string()),
        );
        map.insert(
            "id".to_string(),
            crate::Value::Str(handle.tool.id.to_string()),
        );
        map.insert(
            "path".to_string(),
            crate::Value::Str(handle.path.display().to_string()),
        );
        map.insert("cap".to_string(), crate::Value::Secret(capability_marker()));
        crate::Value::Map(map)
    }
}

/// Allow reconstructing a ToolHandle from an owned Value.
impl TryFrom<crate::Value> for ToolHandle {
    type Error = ToolHandleParseError;

    fn try_from(value: crate::Value) -> Result<Self, Self::Error> {
        ToolHandle::try_from(&value)
    }
}

/// Error when parsing a ToolHandle from a Value.
#[derive(Debug)]
pub struct ToolHandleParseError {
    pub message: String,
}

impl std::fmt::Display for ToolHandleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ToolHandle parse error: {}", self.message)
    }
}

impl std::error::Error for ToolHandleParseError {}

/// Try to reconstruct a ToolHandle from a Value.
///
/// The Value must be a map with a capability marker:
/// { type = "tool_handle", id = "...", path = "...", cap = <secret> }.
/// The tool ID must match a known static tool definition.
impl TryFrom<&crate::Value> for ToolHandle {
    type Error = ToolHandleParseError;

    fn try_from(value: &crate::Value) -> Result<Self, Self::Error> {
        let map = match value {
            crate::Value::Map(m) => m,
            _ => {
                return Err(ToolHandleParseError {
                    message: "Expected map value".to_string(),
                })
            }
        };

        if let Err(e) = ensure_capability_marker(map, "ToolHandle") {
            return Err(ToolHandleParseError { message: e });
        }

        let type_field = map.get("type").and_then(crate::Value::as_str).unwrap_or("");
        if type_field != "tool_handle" {
            return Err(ToolHandleParseError {
                message: format!("Invalid type: expected 'tool_handle', got '{}'", type_field),
            });
        }

        let tool_id = map
            .get("id")
            .and_then(crate::Value::as_str)
            .ok_or_else(|| ToolHandleParseError {
                message: "ToolHandle missing 'id'".to_string(),
            })?;
        let path_str = map
            .get("path")
            .and_then(crate::Value::as_str)
            .ok_or_else(|| ToolHandleParseError {
                message: "ToolHandle missing 'path'".to_string(),
            })?;
        let path = PathBuf::from(path_str);

        // Look up the static tool definition
        let tool = get_tool_by_id(tool_id).ok_or_else(|| ToolHandleParseError {
            message: format!("Unknown tool ID: {}", tool_id),
        })?;

        let resource_handle = tool_resource_handle(tool, &path);

        Ok(ToolHandle {
            tool,
            path,
            resource_handle,
            _acquired: PhantomData,
        })
    }
}

impl Resource for ToolHandle {
    fn resource_id(&self) -> ResourceId {
        self.resource_handle.resource_id().clone()
    }

    fn access_mode(&self) -> AccessMode {
        self.tool.access_mode
    }

    fn kind(&self) -> ResourceKind {
        ResourceKind::Capability
    }
}

fn tool_resource_handle(
    tool: &'static CliToolDef,
    path: &PathBuf,
) -> ResourceHandle<ToolResource> {
    let key = ContentHash::from_path(path);
    ResourceHandle::acquire(tool.resource_id(), key)
}

/// Look up a tool definition by ID.
///
/// Returns the static tool definition if found.
pub fn get_tool_by_id(id: &str) -> Option<&'static CliToolDef> {
    match id {
        "clippy" => Some(&CLIPPY),
        "rustfmt" => Some(&RUSTFMT),
        "cargo" => Some(&CARGO),
        "git" => Some(&GIT),
        "gh" => Some(&GH),
        _ => None,
    }
}

// ============================================================================
// Tool Path Resolution — Trait-Based Abstraction
// ============================================================================

/// Abstraction for resolving tool binary paths on the system.
///
/// Implementations provide the mechanism for locating binaries (e.g., `which`
/// on Unix, `where` on Windows, or a mock for testing). Concrete resolvers that
/// shell out live in the transport layer.
pub trait ToolPathResolver {
    /// Resolve the absolute path to a tool binary.
    ///
    /// Returns `Ok(path)` if the binary is found, or an error if resolution
    /// fails or the binary is not on PATH.
    fn resolve(&self, tool: &'static CliToolDef) -> Result<PathBuf, CliToolError>;
}

/// Mock resolver for testing — returns pre-configured paths for tools.
///
/// # Example
///
/// ```ignore
/// let resolver = MockResolver::new()
///     .with_path("cargo", "/usr/bin/cargo")
///     .with_path("clippy", "/usr/bin/clippy");
/// let path = resolver.resolve(&CARGO).unwrap();
/// ```
pub struct MockResolver {
    paths: HashMap<String, PathBuf>,
}

impl Default for MockResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl MockResolver {
    /// Create an empty mock resolver (all resolutions will fail).
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
        }
    }

    /// Register a resolved path for a tool ID.
    pub fn with_path(mut self, tool_id: &str, path: impl Into<PathBuf>) -> Self {
        self.paths.insert(tool_id.to_string(), path.into());
        self
    }
}

impl ToolPathResolver for MockResolver {
    fn resolve(&self, tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
        self.paths.get(tool.id).cloned().ok_or_else(|| {
            CliToolError::new(
                tool,
                "resolve",
                format!("MockResolver: no path configured for '{}'", tool.id),
            )
        })
    }
}

impl CliToolDef {
    /// Create a Check operation for this tool.
    pub fn check(&'static self) -> CliToolOp {
        CliToolOp::Check { tool: self }
    }

    /// Create an Install operation for this tool.
    pub fn install(&'static self) -> CliToolOp {
        CliToolOp::Install { tool: self }
    }

    /// Create a Run operation for this tool with the given arguments.
    pub fn run(&'static self, args: &[&str]) -> CliToolOp {
        CliToolOp::Run {
            tool: self,
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Create a ToolHandle for this tool with a resolved path.
    ///
    /// **Framework use only.** See `ToolHandle::acquire` for details.
    pub fn acquire(&'static self, path: impl Into<PathBuf>) -> ToolHandle {
        ToolHandle::acquire(self, path)
    }

    /// Create a mock ToolHandle for this tool.
    ///
    /// **Testing/DryRun use only.**
    pub fn mock(&'static self) -> ToolHandle {
        ToolHandle::mock(self)
    }

    /// Get the binary name for this tool (first element of run_cmd).
    pub fn binary_name(&self) -> Option<&'static str> {
        self.run_cmd.first().copied()
    }
}

/// Generic operation for any CLI tool.
///
/// This enum represents the three phases of the upsert pattern:
/// - Check: Does the tool exist?
/// - Install: Create/install the tool
/// - Run: Execute the tool
#[derive(Debug, Clone)]
pub enum CliToolOp {
    /// Check if the tool is installed.
    /// Outputs: exists (Bool), output (String)
    Check { tool: &'static CliToolDef },

    /// Install the tool.
    /// Outputs: success (Bool), error (String if failed)
    Install { tool: &'static CliToolDef },

    /// Run the tool with arguments.
    /// Outputs: success (Bool), stdout (String), stderr (String), exit_code (Int)
    Run {
        tool: &'static CliToolDef,
        args: Vec<String>,
    },
}

impl CliToolOp {
    /// Create a Check operation.
    pub fn check(tool: &'static CliToolDef) -> Self {
        Self::Check { tool }
    }

    /// Create an Install operation.
    pub fn install(tool: &'static CliToolDef) -> Self {
        Self::Install { tool }
    }

    /// Create a Run operation.
    pub fn run(tool: &'static CliToolDef, args: &[&str]) -> Self {
        Self::Run {
            tool,
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Get the tool this operation is for.
    pub fn tool(&self) -> &'static CliToolDef {
        match self {
            Self::Check { tool } | Self::Install { tool } | Self::Run { tool, .. } => tool,
        }
    }
}

// ============================================================================
// Fractal DAG Builder
// ============================================================================

use crate::node::Node;
use crate::patterns::UpsertBuilder;

/// Build a CLI tool upsert sub-DAG node.
///
/// This creates a `Node<CliToolOp>` containing a sub-DAG that implements
/// the check → install → run pattern. The node can be composed into
/// larger DAGs using the fractal DAG approach.
///
/// # Arguments
///
/// * `tool` - The CLI tool definition
/// * `args` - Arguments to pass when running the tool
///
/// # Example
///
/// ```ignore
/// use gunbc_ir::transport::cli::{build_cli_upsert, CLIPPY};
///
/// // Build upsert sub-DAG for clippy
/// let node = build_cli_upsert(&CLIPPY, &["--all-targets", "--", "-D", "warnings"]);
///
/// // The node can be added to a larger DAG
/// builder.add_node(node);
/// ```
pub fn build_cli_upsert(tool: &'static CliToolDef, args: &[&str]) -> Node<CliToolOp> {
    UpsertBuilder::new(tool.id)
        .with_check(CliToolOp::check(tool))
        .with_create(CliToolOp::install(tool))
        .with_resolve(CliToolOp::run(tool, args))
        .with_input_port("trigger", "Unit")
        .with_output_port("result", "CliResult")
        .build()
}

/// Build a CLI tool upsert sub-DAG for just check + install (no run).
///
/// This is useful when you want to ensure a tool is installed but
/// will run it separately with different arguments.
pub fn build_cli_ensure(tool: &'static CliToolDef) -> Node<CliToolOp> {
    UpsertBuilder::new(format!("{}_ensure", tool.id))
        .with_check(CliToolOp::check(tool))
        .with_create(CliToolOp::install(tool))
        .with_resolve(CliToolOp::check(tool)) // Re-check as resolve
        .with_input_port("trigger", "Unit")
        .with_output_port("exists", "Bool")
        .build()
}

/// Error type for CLI tool operations.
#[derive(Debug)]
pub struct CliToolError {
    pub tool_id: &'static str,
    pub operation: &'static str,
    pub message: String,
}

impl std::fmt::Display for CliToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}: {}", self.tool_id, self.operation, self.message)
    }
}

impl std::error::Error for CliToolError {}

impl CliToolError {
    pub fn new(
        tool: &'static CliToolDef,
        operation: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            tool_id: tool.id,
            operation,
            message: message.into(),
        }
    }
}

// ============================================================================
// Common Tool Definitions
// ============================================================================

/// Clippy - Rust linter
/// Read access mode - can run in parallel with other read-only tools
pub static CLIPPY: CliToolDef = CliToolDef {
    id: "clippy",
    check_cmd: &["cargo", "clippy", "--version"],
    install_cmd: Some(&["rustup", "component", "add", "clippy"]),
    run_cmd: &["cargo", "clippy"],
    description: "Rust linter",
    access_mode: AccessMode::Read,
};

/// Rustfmt - Rust formatter
/// Write access mode - modifies files, should not run in parallel with other writers
pub static RUSTFMT: CliToolDef = CliToolDef {
    id: "rustfmt",
    check_cmd: &["rustfmt", "--version"],
    install_cmd: Some(&["rustup", "component", "add", "rustfmt"]),
    run_cmd: &["cargo", "fmt"],
    description: "Rust formatter",
    access_mode: AccessMode::Write,
};

/// Cargo - Rust package manager (no auto-install, requires rustup)
/// Read access mode by default (depends on subcommand)
pub static CARGO: CliToolDef = CliToolDef {
    id: "cargo",
    check_cmd: &["cargo", "--version"],
    install_cmd: None, // Requires rustup.rs
    run_cmd: &["cargo"],
    description: "Rust package manager",
    access_mode: AccessMode::Read,
};

/// Git - Version control
/// Read access mode by default (depends on subcommand)
pub static GIT: CliToolDef = CliToolDef {
    id: "git",
    check_cmd: &["git", "--version"],
    install_cmd: None, // System-dependent
    run_cmd: &["git"],
    description: "Version control system",
    access_mode: AccessMode::Read,
};

/// GitHub CLI
pub static GH: CliToolDef = CliToolDef {
    id: "gh",
    check_cmd: &["gh", "--version"],
    install_cmd: None, // System-dependent (brew, apt, etc.)
    run_cmd: &["gh"],
    description: "GitHub CLI",
    access_mode: AccessMode::Read,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clippy_def() {
        assert_eq!(CLIPPY.id, "clippy");
        assert!(CLIPPY.install_cmd.is_some());
    }

    #[test]
    fn test_cargo_no_auto_install() {
        assert!(CARGO.install_cmd.is_none());
    }

    #[test]
    fn test_tool_check_op() {
        let op = CLIPPY.check();
        assert!(matches!(op, CliToolOp::Check { .. }));
        assert_eq!(op.tool().id, "clippy");
    }

    #[test]
    fn test_tool_run_op() {
        let op = CLIPPY.run(&["--", "-D", "warnings"]);
        if let CliToolOp::Run { args, .. } = op {
            assert_eq!(args, vec!["--", "-D", "warnings"]);
        } else {
            panic!("Expected Run variant");
        }
    }

    // ========================================================================
    // Integration tests for tool path resolution
    // ========================================================================

    use crate::resource::AccessMode;

    /// Tool definition for testing - uses `git` which should exist everywhere.
    static TEST_TOOL_GIT: CliToolDef = CliToolDef {
        id: "git",
        check_cmd: &["git", "--version"],
        install_cmd: None,
        run_cmd: &["git"],
        description: "Test git tool",
        access_mode: AccessMode::Read,
    };

    #[test]
    fn test_mock_resolver_returns_configured_path() {
        let resolver = MockResolver::new().with_path("git", "/mock/path/to/git");
        let result = resolver.resolve(&TEST_TOOL_GIT);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/mock/path/to/git"));
    }

    #[test]
    fn test_mock_resolver_fails_for_unconfigured() {
        let resolver = MockResolver::new(); // no paths configured
        let result = resolver.resolve(&TEST_TOOL_GIT);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("MockResolver"));
        assert!(err.message.contains("no path configured"));
    }

    #[test]
    fn test_cli_tool_error_display() {
        let err = CliToolError::new(&TEST_TOOL_GIT, "resolve", "tool not found");

        let display = format!("{}", err);
        assert!(display.contains("git"));
        assert!(display.contains("resolve"));
        assert!(display.contains("tool not found"));
    }

    #[test]
    fn test_tool_handle_acquire() {
        let handle = TEST_TOOL_GIT.acquire("/path/to/git");

        assert_eq!(handle.id(), "git");
        assert_eq!(handle.path(), &PathBuf::from("/path/to/git"));
        // A real acquired handle has a real path
        assert!(!handle.path().to_string_lossy().starts_with("/mock/"));
    }

    #[test]
    fn test_tool_handle_mock() {
        let handle = TEST_TOOL_GIT.mock();

        assert_eq!(handle.id(), "git");
        // Mock handles have paths starting with /mock/
        assert!(handle.path().to_string_lossy().starts_with("/mock/"));
        assert!(handle.path().to_string_lossy().contains("git"));
    }

    #[test]
    fn test_get_tool_by_id() {
        let tool = get_tool_by_id("git");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().id, "git");

        let unknown = get_tool_by_id("unknown_tool_xyz");
        assert!(unknown.is_none());
    }
}
