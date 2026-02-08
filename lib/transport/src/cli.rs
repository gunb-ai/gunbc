//! CLI tool execution and acquisition (transport boundary).
//!
//! This module hosts the ONLY implementations that directly execute CLI
//! commands (via std::process::Command). Higher-level crates should treat
//! these helpers as the I/O boundary for tool acquisition and execution.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use gunbc_ir::transport::cli::{CliToolDef, CliToolError, CliToolOp, ToolHandle, ToolPathResolver};
use gunbc_ir::Value;

/// Resolver that uses the `which` command to find binaries on PATH (Unix).
pub struct WhichResolver;

impl ToolPathResolver for WhichResolver {
    fn resolve(&self, tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
        let binary = tool
            .binary_name()
            .ok_or_else(|| CliToolError::new(tool, "resolve", "No binary name defined"))?;

        let output = Command::new("which").arg(binary).output().map_err(|e| {
            CliToolError::new(tool, "resolve", format!("Failed to run which: {}", e))
        })?;

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

/// Resolver that uses the `where` command to find binaries on PATH (Windows).
pub struct WhereResolver;

impl ToolPathResolver for WhereResolver {
    fn resolve(&self, tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
        let binary = tool
            .binary_name()
            .ok_or_else(|| CliToolError::new(tool, "resolve", "No binary name defined"))?;

        let output = Command::new("where").arg(binary).output().map_err(|e| {
            CliToolError::new(tool, "resolve", format!("Failed to run where: {}", e))
        })?;

        if !output.status.success() {
            return Err(CliToolError::new(
                tool,
                "resolve",
                format!("Binary '{}' not found on PATH", binary),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let path_str = stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .ok_or_else(|| {
                CliToolError::new(
                    tool,
                    "resolve",
                    format!("Binary '{}' not found on PATH", binary),
                )
            })?;

        Ok(PathBuf::from(path_str))
    }
}

/// Resolve a tool binary path using the platform default resolver.
pub fn resolve_tool_path(tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
    #[cfg(windows)]
    {
        let resolver = WhereResolver;
        resolve_tool_path_with(tool, &resolver)
    }
    #[cfg(not(windows))]
    {
        let resolver = WhichResolver;
        resolve_tool_path_with(tool, &resolver)
    }
}

/// Resolve a tool binary path with an injected resolver.
pub fn resolve_tool_path_with(
    tool: &'static CliToolDef,
    resolver: &dyn ToolPathResolver,
) -> Result<PathBuf, CliToolError> {
    resolver.resolve(tool)
}

/// Upsert a tool using the default resolver.
pub fn upsert_tool(tool: &'static CliToolDef) -> Result<PathBuf, CliToolError> {
    #[cfg(windows)]
    {
        let resolver = WhereResolver;
        upsert_tool_with(tool, &resolver)
    }
    #[cfg(not(windows))]
    {
        let resolver = WhichResolver;
        upsert_tool_with(tool, &resolver)
    }
}

/// Upsert a tool with an injected resolver.
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

/// Execute a CLI tool op using the tool's configured command.
pub fn execute_cli_tool_op(
    op: &CliToolOp,
) -> Result<HashMap<String, Value>, CliToolError> {
    match op {
        CliToolOp::Check { tool } => execute_check(tool),
        CliToolOp::Install { tool } => execute_install(tool),
        CliToolOp::Run { tool, args } => execute_run(tool, args),
        CliToolOp::ResourceGate { ports, .. } => execute_resource_gate(ports),
    }
}

/// Execute a CLI tool op, preferring the path from a ToolHandle when provided.
pub fn execute_cli_tool_op_with_handle(
    op: &CliToolOp,
    handle: &ToolHandle,
) -> Result<HashMap<String, Value>, CliToolError> {
    match op {
        CliToolOp::Run { tool, args } => {
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
        _ => execute_cli_tool_op(op),
    }
}

// ============================================================================
// Internal execution helpers (I/O boundary)
// ============================================================================

fn execute_check(tool: &'static CliToolDef) -> Result<HashMap<String, Value>, CliToolError> {
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
    out.insert("exists".to_string(), Value::Bool(exists));
    out.insert("output".to_string(), Value::Str(stdout));
    Ok(out)
}

fn execute_install(tool: &'static CliToolDef) -> Result<HashMap<String, Value>, CliToolError> {
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
    out.insert("success".to_string(), Value::Bool(true));
    out.insert("install_done".to_string(), Value::Bool(true));
    Ok(out)
}

fn execute_resource_gate(ports: &[String]) -> Result<HashMap<String, Value>, CliToolError> {
    let mut out = HashMap::new();
    for port in ports {
        out.insert(port.clone(), Value::Unit);
    }
    Ok(out)
}

fn execute_run(
    tool: &'static CliToolDef,
    args: &[String],
) -> Result<HashMap<String, Value>, CliToolError> {
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
    out.insert("success".to_string(), Value::Bool(success));
    out.insert("exit_code".to_string(), Value::Int(exit_code as i64));
    out.insert("stdout".to_string(), Value::Str(stdout));
    out.insert("stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

fn execute_run_with_path(
    tool: &'static CliToolDef,
    args: &[String],
    path: &PathBuf,
) -> Result<HashMap<String, Value>, CliToolError> {
    if tool.run_cmd.is_empty() {
        return Err(CliToolError::new(tool, "run", "No run command defined"));
    }

    let (_, base_args) = tool.run_cmd.split_first().unwrap();

    let mut full_args: Vec<&str> = base_args.to_vec();
    full_args.extend(args.iter().map(|s| s.as_str()));

    println!("Running: {} {}", path.display(), full_args.join(" "));

    let output = Command::new(path)
        .args(&full_args)
        .output()
        .map_err(|e| CliToolError::new(tool, "run", format!("Failed to execute: {}", e)))?;

    let success = output.status.success();
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut out = HashMap::new();
    out.insert("success".to_string(), Value::Bool(success));
    out.insert("exit_code".to_string(), Value::Int(exit_code as i64));
    out.insert("stdout".to_string(), Value::Str(stdout));
    out.insert("stderr".to_string(), Value::Str(stderr));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::resource::AccessMode;
    use gunbc_ir::transport::cli::MockResolver;

    /// Tool definition for testing - uses `git` which should exist everywhere.
    static TEST_TOOL_GIT: CliToolDef = CliToolDef {
        id: "git",
        check_cmd: &["git", "--version"],
        install_cmd: None,
        run_cmd: &["git"],
        description: "Test git tool",
        access_mode: AccessMode::Read,
    };

    /// Tool definition for a nonexistent tool.
    static TEST_TOOL_NONEXISTENT: CliToolDef = CliToolDef {
        id: "nonexistent_tool_xyz_12345",
        check_cmd: &["nonexistent_tool_xyz_12345", "--version"],
        install_cmd: None,
        run_cmd: &["nonexistent_tool_xyz_12345"],
        description: "Nonexistent tool for testing",
        access_mode: AccessMode::Read,
    };

    #[test]
    #[cfg(not(windows))]
    fn test_which_resolver_finds_git() {
        let resolver = WhichResolver;
        let result = resolver.resolve(&TEST_TOOL_GIT);

        assert!(result.is_ok(), "git should be found: {:?}", result);
        let path = result.unwrap();
        assert!(path.exists(), "resolved path should exist");
        assert!(
            path.to_string_lossy().contains("git"),
            "path should contain 'git'"
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn test_which_resolver_fails_for_nonexistent() {
        let resolver = WhichResolver;
        let result = resolver.resolve(&TEST_TOOL_NONEXISTENT);

        assert!(
            result.is_err(),
            "nonexistent tool should not resolve: {:?}",
            result
        );

        let err = result.unwrap_err();
        assert_eq!(err.tool_id, "nonexistent_tool_xyz_12345");
        assert!(err.message.contains("not found"));
    }

    #[test]
    #[cfg(windows)]
    fn test_where_resolver_finds_git() {
        let resolver = WhereResolver;
        let result = resolver.resolve(&TEST_TOOL_GIT);

        assert!(result.is_ok(), "git should be found: {:?}", result);
        let path = result.unwrap();
        assert!(path.exists(), "resolved path should exist");
        assert!(
            path.to_string_lossy().contains("git"),
            "path should contain 'git'"
        );
    }

    #[test]
    #[cfg(windows)]
    fn test_where_resolver_fails_for_nonexistent() {
        let resolver = WhereResolver;
        let result = resolver.resolve(&TEST_TOOL_NONEXISTENT);

        assert!(
            result.is_err(),
            "nonexistent tool should not resolve: {:?}",
            result
        );

        let err = result.unwrap_err();
        assert_eq!(err.tool_id, "nonexistent_tool_xyz_12345");
        assert!(err.message.contains("not found"));
    }

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
    #[cfg(not(windows))]
    fn test_resolve_tool_path_with_which() {
        let result = resolve_tool_path_with(&TEST_TOOL_GIT, &WhichResolver);
        assert!(result.is_ok(), "should resolve git: {:?}", result);
    }

    #[test]
    fn test_resolve_tool_path_with_mock() {
        let mock = MockResolver::new().with_path("git", "/test/git");
        let result = resolve_tool_path_with(&TEST_TOOL_GIT, &mock);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("/test/git"));
    }
}
