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
//! For simple cases, operations can be executed directly:
//!
//! ```ignore
//! use gunbc_ir::transport::cli::{CliToolOp, CLIPPY};
//!
//! let check = CliToolOp::check(&CLIPPY);
//! let result = check.execute()?;
//! ```

use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::process::Command;

use crate::resource::{AccessMode, ResourceId};

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
///
/// # Example
///
/// ```ignore
/// // In operation implementation - receives handle from inputs
/// fn execute_lint(inputs: HashMap<String, Value>) -> Result<...> {
///     let clippy: ToolHandle = inputs.get("tool:clippy").unwrap();
///     let result = clippy.run(&["--all-targets"]).execute()?;
///     // ...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ToolHandle {
    tool: &'static CliToolDef,
    /// Resolved path to the tool binary (e.g., "/usr/bin/cargo").
    /// For mocked handles, this may be a placeholder path.
    path: PathBuf,
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
        Self {
            tool,
            path: path.into(),
            _acquired: PhantomData,
        }
    }

    /// Create a mock ToolHandle for testing/DryRun mode.
    ///
    /// The path will be `/mock/{tool_id}` to make it obvious this is not real.
    pub fn mock(tool: &'static CliToolDef) -> Self {
        Self {
            tool,
            path: PathBuf::from(format!("/mock/{}", tool.id)),
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

    /// Run the tool with the given arguments.
    /// This is the ONLY way to execute a tool - you need the handle.
    pub fn run(&self, args: &[&str]) -> CliToolOp {
        CliToolOp::run(self.tool, args)
    }

    /// Get the resource ID for this tool handle.
    pub fn resource_id(&self) -> ResourceId {
        self.tool.resource_id()
    }
}

/// Convert a ToolHandle to a Value for passing through DAG edges.
impl From<ToolHandle> for crate::Value {
    fn from(handle: ToolHandle) -> Self {
        // Encode as "tool_handle:{id}:{path}" for reconstruction
        crate::Value::Str(format!(
            "tool_handle:{}:{}",
            handle.tool.id,
            handle.path.display()
        ))
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
/// The Value must be a string in the format "tool_handle:{id}:{path}".
/// The tool ID must match a known static tool definition.
impl TryFrom<&crate::Value> for ToolHandle {
    type Error = ToolHandleParseError;

    fn try_from(value: &crate::Value) -> Result<Self, Self::Error> {
        let s = match value {
            crate::Value::Str(s) => s,
            _ => {
                return Err(ToolHandleParseError {
                    message: "Expected string value".to_string(),
                })
            }
        };

        let parts: Vec<&str> = s.splitn(3, ':').collect();
        if parts.len() != 3 || parts[0] != "tool_handle" {
            return Err(ToolHandleParseError {
                message: format!("Invalid format: expected 'tool_handle:id:path', got '{}'", s),
            });
        }

        let tool_id = parts[1];
        let path = PathBuf::from(parts[2]);

        // Look up the static tool definition
        let tool = get_tool_by_id(tool_id).ok_or_else(|| ToolHandleParseError {
            message: format!("Unknown tool ID: {}", tool_id),
        })?;

        Ok(ToolHandle {
            tool,
            path,
            _acquired: PhantomData,
        })
    }
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
/// on Unix, `where` on Windows, or a mock for testing). This trait allows
/// the tool acquisition pipeline to be tested without shelling out.
pub trait ToolPathResolver {
    /// Resolve the absolute path to a tool binary.
    ///
    /// Returns `Ok(path)` if the binary is found, or an error if resolution
    /// fails or the binary is not on PATH.
    fn resolve(&self, tool: &'static CliToolDef) -> Result<PathBuf, CliToolError>;
}

/// Default resolver that uses the `which` command to find binaries on PATH.
pub struct WhichResolver;

#[allow(clippy::disallowed_methods)]
impl ToolPathResolver for WhichResolver {
    fn resolve(&self, tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
        let binary = tool.binary_name().ok_or_else(|| {
            CliToolError::new(tool, "resolve", "No binary name defined")
        })?;

        let output = Command::new("which")
            .arg(binary)
            .output()
            .map_err(|e| CliToolError::new(tool, "resolve", format!("Failed to run which: {}", e)))?;

        if !output.status.success() {
            return Err(CliToolError::new(
                tool,
                "resolve",
                format!("Binary '{}' not found on PATH", binary),
            ));
        }

        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(path_str))
    }
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
        Self { paths: HashMap::new() }
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

/// Resolve the path to a tool binary using `which`.
///
/// Convenience wrapper that uses [`WhichResolver`]. For injectable usage,
/// call [`resolve_tool_path_with`] instead.
pub fn resolve_tool_path(tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
    resolve_tool_path_with(tool, &WhichResolver)
}

/// Resolve the path to a tool binary using an injected resolver.
pub fn resolve_tool_path_with(
    tool: &'static CliToolDef,
    resolver: &dyn ToolPathResolver,
) -> Result<PathBuf, CliToolError> {
    resolver.resolve(tool)
}

/// Upsert a tool: check if installed, install if needed, return resolved path.
///
/// Convenience wrapper that uses [`WhichResolver`]. For injectable usage,
/// call [`upsert_tool_with`] instead.
#[allow(clippy::disallowed_methods)]
pub fn upsert_tool(tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
    upsert_tool_with(tool, &WhichResolver)
}

/// Upsert a tool using an injected path resolver.
///
/// This is the injectable variant — callers can pass a [`MockResolver`] for
/// testing or a [`WhichResolver`] for production use.
#[allow(clippy::disallowed_methods)]
pub fn upsert_tool_with(
    tool: &'static CliToolDef,
    resolver: &dyn ToolPathResolver,
) -> Result<PathBuf, CliToolError> {
    // Step 1: Check if tool exists
    let check_result = execute_check(tool)?;
    let exists = check_result
        .get("exists")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Step 2: Install if needed
    if !exists {
        execute_install(tool)?;
    }

    // Step 3: Resolve and return the path
    resolve_tool_path_with(tool, resolver)
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
    
    /// Execute the operation, returning outputs as a HashMap.
    ///
    /// This is the core execution logic for CLI tools.
    pub fn execute(&self) -> Result<HashMap<String, crate::Value>, CliToolError> {
        match self {
            Self::Check { tool } => execute_check(tool),
            Self::Install { tool } => execute_install(tool),
            Self::Run { tool, args } => execute_run(tool, args),
        }
    }

    /// Execute the operation using a ToolHandle (for Run ops).
    ///
    /// For `Run`, this uses the resolved path from the handle instead of the
    /// tool's configured binary name. For `Check`/`Install`, it falls back to
    /// the standard execution logic.
    pub fn execute_with_handle(
        &self,
        handle: &ToolHandle,
    ) -> Result<HashMap<String, crate::Value>, CliToolError> {
        match self {
            Self::Run { tool, args } => {
                if handle.id() != tool.id {
                    return Err(CliToolError::new(
                        tool,
                        "run",
                        format!(
                            "Tool handle '{}' does not match expected '{}'",
                            handle.id(),
                            tool.id
                        ),
                    ));
                }
                execute_run_with_path(tool, args, handle.path())
            }
            _ => self.execute(),
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
    pub fn new(tool: &'static CliToolDef, operation: &'static str, message: impl Into<String>) -> Self {
        Self {
            tool_id: tool.id,
            operation,
            message: message.into(),
        }
    }
}

// ============================================================================
// Execution Functions
// ============================================================================

// These functions are the tool acquisition implementation - they're allowed
// to use Command::new because they ARE the abstraction that other code uses.
#[allow(clippy::disallowed_methods)]
fn execute_check(tool: &'static CliToolDef) -> Result<HashMap<String, crate::Value>, CliToolError> {
    if tool.check_cmd.is_empty() {
        return Err(CliToolError::new(tool, "check", "No check command defined"));
    }
    
    let (cmd, args) = tool.check_cmd.split_first().unwrap();
    
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| CliToolError::new(tool, "check", format!("Failed to execute: {}", e)))?;
    
    let exists = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    
    let mut out = HashMap::new();
    out.insert("exists".to_string(), crate::Value::Bool(exists));
    out.insert("output".to_string(), crate::Value::Str(stdout));
    Ok(out)
}

#[allow(clippy::disallowed_methods)]
fn execute_install(tool: &'static CliToolDef) -> Result<HashMap<String, crate::Value>, CliToolError> {
    let install_cmd = tool.install_cmd.ok_or_else(|| {
        CliToolError::new(
            tool,
            "install",
            format!(
                "{} does not support automatic installation. Please install manually.",
                tool.id
            ),
        )
    })?;
    
    if install_cmd.is_empty() {
        return Err(CliToolError::new(tool, "install", "Empty install command"));
    }
    
    println!("Installing {}...", tool.id);
    
    let (cmd, args) = install_cmd.split_first().unwrap();
    
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| CliToolError::new(tool, "install", format!("Failed to execute: {}", e)))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CliToolError::new(tool, "install", stderr.to_string()));
    }
    
    println!("{} installed successfully", tool.id);
    
    let mut out = HashMap::new();
    out.insert("success".to_string(), crate::Value::Bool(true));
    Ok(out)
}

#[allow(clippy::disallowed_methods)]
fn execute_run(
    tool: &'static CliToolDef,
    args: &[String],
) -> Result<HashMap<String, crate::Value>, CliToolError> {
    if tool.run_cmd.is_empty() {
        return Err(CliToolError::new(tool, "run", "No run command defined"));
    }
    
    let (cmd, base_args) = tool.run_cmd.split_first().unwrap();
    
    let mut full_args: Vec<&str> = base_args.to_vec();
    full_args.extend(args.iter().map(|s| s.as_str()));
    
    println!("Running: {} {}", cmd, full_args.join(" "));
    
    let output = Command::new(cmd)
        .args(&full_args)
        .output()
        .map_err(|e| CliToolError::new(tool, "run", format!("Failed to execute: {}", e)))?;
    
    let success = output.status.success();
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    let mut out = HashMap::new();
    out.insert("success".to_string(), crate::Value::Bool(success));
    out.insert("exit_code".to_string(), crate::Value::Int(exit_code as i64));
    out.insert("stdout".to_string(), crate::Value::Str(stdout));
    out.insert("stderr".to_string(), crate::Value::Str(stderr));
    Ok(out)
}

/// Execute a tool run using a resolved binary path.
#[allow(clippy::disallowed_methods)]
pub fn execute_run_with_path(
    tool: &'static CliToolDef,
    args: &[String],
    path: &PathBuf,
) -> Result<HashMap<String, crate::Value>, CliToolError> {
    if tool.run_cmd.is_empty() {
        return Err(CliToolError::new(tool, "run", "No run command defined"));
    }

    let base_args = &tool.run_cmd[1..];
    let mut full_args: Vec<String> = base_args.iter().map(|s| s.to_string()).collect();
    full_args.extend_from_slice(args);

    let output = Command::new(path)
        .args(&full_args)
        .output()
        .map_err(|e| {
            CliToolError::new(
                tool,
                "run",
                format!("Failed to execute '{}': {}", path.display(), e),
            )
        })?;

    let success = output.status.success();
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut out = HashMap::new();
    out.insert("success".to_string(), crate::Value::Bool(success));
    out.insert("exit_code".to_string(), crate::Value::Int(exit_code as i64));
    out.insert("stdout".to_string(), crate::Value::Str(stdout));
    out.insert("stderr".to_string(), crate::Value::Str(stderr));
    Ok(out)
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
}
