//! Gist operations.
//!
//! Operations for working with GitHub Gists.
//!
//! All operations are PURE (no I/O). I/O happens through TransportOps::Execute nodes.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_lib_gist_ops::prepare_gist_request;
//!
//! let request = prepare_gist_request("# My Content", true, "My gist");
//! // request is now a TransportRequest ready to be executed via TransportOps::Execute
//! ```

#![forbid(dead_code)]
use gunbc_exec::{
    optional_str, require_response, require_str, ExecError, Executable, IntoExecResult, OutputMap,
};
use gunbc_ir::transport::gist::GistRequest;
use gunbc_ir::transport::{ShellResponse, TransportRequest, TransportResponse};
use gunbc_ir::{Timestamp, Value};
use gunbc_primitives::filename;
use std::collections::HashMap;
use std::time::SystemTime;

/// Gist operations for use in DAG nodes.
///
/// All operations are PURE - no I/O. Use TransportOps::Execute for actual I/O.
#[derive(Debug, Clone)]
pub enum GistOps {
    /// Filesystem environment (resource acquisition)
    FsEnv { scope: filename::Scope },
    /// Clock environment (timestamp snapshot)
    ClockEnv,
    /// Prepare a gist creation request (PURE - no I/O)
    PrepareRequest { public: bool },
    /// Parse gist response to extract URL (PURE - no I/O)
    ParseGistResponse,
}

impl Executable for GistOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistOps::FsEnv { scope } => {
                let fs = filename::FilesystemHandle::cross_platform(*scope);
                let port = match scope {
                    filename::Scope::Read => "fs:read",
                    filename::Scope::Write => "fs:write",
                };
                OutputMap::new().value(port, fs.into()).ok()
            }
            GistOps::ClockEnv => {
                let ts = Timestamp::now();
                OutputMap::new().value("clock", ts.into()).ok()
            }
            GistOps::PrepareRequest { public } => {
                let markdown = require_str(&inputs, "markdown")?;
                let branch = optional_str(&inputs, "branch");
                let remote_branch = optional_str(&inputs, "remote_branch");
                let base_ref = optional_str(&inputs, "base_ref");

                // Acquire system resources at the DAG boundary (not inline)
                let fs = require_filesystem_handle(&inputs, "res:fs")?;
                let now = require_timestamp(&inputs, "res:clock")?;

                // Explicit priority: local branch > remote branch > "snapshot".
                // - `branch` is set when HEAD points to a local branch
                // - `remote_branch` is set when HEAD is detached at a remote
                //   tracking branch (e.g., after `git checkout origin/main`)
                // - Neither is set for arbitrary detached commits
                let effective_branch = branch.or(remote_branch);

                let (filename, description) = if let Some(sha) = base_ref {
                    // Recent mode: base_ref is the SHA from rev-list
                    let short_sha = &sha[..sha.len().min(7)];
                    let branch_label = effective_branch.unwrap_or("snapshot");
                    let prefix = format!(
                        "{}_recent-3d_{}..HEAD",
                        sanitize_branch_for_filename(&fs, branch_label),
                        short_sha
                    );
                    let filename = generate_gist_filename_with_prefix(&fs, &prefix, now);
                    let description = match effective_branch {
                        Some(b) if !b.trim().is_empty() && b.trim() != "HEAD" => {
                            format!(
                                "Recent changes (3d) {}..HEAD on {} created by gunbc-gist",
                                short_sha, b
                            )
                        }
                        _ => format!(
                            "Recent changes (3d) {}..HEAD created by gunbc-gist",
                            short_sha
                        ),
                    };
                    (filename, description)
                } else {
                    // Snapshot/diff mode, or recent mode with young repo
                    // (young repo: parse_rev_list produces no output → absent from inputs)
                    let filename = generate_gist_filename(
                        &fs,
                        effective_branch.unwrap_or("snapshot"),
                        now,
                    );
                    let description = match effective_branch {
                        Some(b) if !b.trim().is_empty() && b.trim() != "HEAD" => {
                            format!("Code snapshot of {} created by gunbc-gist", b)
                        }
                        _ => "Code snapshot created by gunbc-gist".to_string(),
                    };
                    (filename, description)
                };

                let request = prepare_gist_request(markdown, *public, &description, &filename);

                OutputMap::new().request("request", request).ok()
            }
            GistOps::ParseGistResponse => {
                let response = require_response(&inputs, "response")?;

                let url = extract_gist_url(response);

                OutputMap::new().str("url", url).ok()
            }
        }
    }
}

// ============================================================================
// Standalone helper functions
// ============================================================================

/// Prepare a gist creation request.
///
/// Returns a `TransportRequest` that can be executed to create a gist.
/// This is PURE - it doesn't perform any I/O, just builds the request.
///
/// # Example
///
/// ```ignore
/// let request = prepare_gist_request("# Hello", true, "My public gist", "main_2024-01-15_14-30-00.md");
/// // Execute via TransportOps::Execute node in the DAG
/// ```
pub fn prepare_gist_request(
    content: &str,
    public: bool,
    description: &str,
    filename: &str,
) -> TransportRequest {
    GistRequest::new()
        .file(filename, content)
        .public(public)
        .description(description)
        .to_shell_request()
}

/// Sanitize a branch name for use as a filename component.
///
/// Falls back to `"snapshot"` if the branch is empty or entirely degenerate.
///
/// Replaces spaces with the replacement char (convention, not a FS rule),
/// then routes through the filesystem gateway. Falls back to `"snapshot"`
/// if the input is empty or sanitizes to the filesystem's default fallback.
///
/// # Examples
///
/// ```ignore
/// let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
/// assert_eq!(sanitize_branch_for_filename(&fs, "main"), "main");
/// assert_eq!(sanitize_branch_for_filename(&fs, "claude/branch-name"), "claude-branch-name");
/// ```
pub fn sanitize_branch_for_filename(fs: &filename::FilesystemHandle, branch: &str) -> String {
    // Replace spaces before filesystem gateway (convention, not a FS rule)
    let no_spaces: String = branch
        .chars()
        .map(|c| if c == ' ' { fs.replacement() } else { c })
        .collect();

    let outcome = fs.prepare_filename(&no_spaces, filename::WritePolicy::Sanitize);

    // Detect the sanitizer's degenerate-input fallback ("untitled") vs a real branch
    // literally named "untitled". A real "untitled" branch passes validation unchanged
    // (outcome is Valid), while a degenerate input produces Sanitized { sanitized: "untitled" }.
    if outcome.was_sanitized() && outcome.filename() == Some("untitled") {
        return "snapshot".to_string();
    }

    match outcome.filename() {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => "snapshot".to_string(),
    }
}

/// Generate a gist filename from a branch name, filesystem handle, and timestamp.
///
/// The branch prefix is sanitized and truncated to fit within the filesystem's
/// max component bytes after accounting for the suffix (`_YYYY-MM-DD_HH-MM-SS.md`).
///
/// # Examples
///
/// ```ignore
/// let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
/// let now = Timestamp::now();
/// let filename = generate_gist_filename(&fs, "claude/my-feature", now);
/// assert!(filename.starts_with("claude-my-feature_"));
/// assert!(filename.ends_with(".md"));
/// ```
pub fn generate_gist_filename(
    fs: &filename::FilesystemHandle,
    branch: &str,
    now: Timestamp,
) -> String {
    let sanitized = sanitize_branch_for_filename(fs, branch);
    let timestamp = format_utc_timestamp(now.to_system_time());
    let suffix = format!("_{}.md", timestamp); // e.g., "_2024-01-15_14-30-00.md" = 23 bytes

    // Ensure the full filename fits within the filesystem's component limit.
    let max_bytes = fs.max_component_bytes();
    let branch_budget = max_bytes.saturating_sub(suffix.len());

    let truncated = if sanitized.len() > branch_budget {
        // Truncate at UTF-8 boundary
        let mut end = branch_budget;
        while end > 0 && !sanitized.is_char_boundary(end) {
            end -= 1;
        }
        // Trim trailing replacement char from truncation point
        sanitized[..end].trim_end_matches(fs.replacement())
    } else {
        &sanitized
    };

    if truncated.is_empty() {
        format!("snapshot{}", suffix)
    } else {
        format!("{}{}", truncated, suffix)
    }
}

/// Generate a gist filename from a pre-built prefix and timestamp.
///
/// Unlike `generate_gist_filename`, the caller supplies the full prefix
/// (already sanitized). This is used when the prefix contains extra
/// metadata like a commit range (e.g., `main_recent-3d_abc123d..HEAD`).
pub fn generate_gist_filename_with_prefix(
    fs: &filename::FilesystemHandle,
    prefix: &str,
    now: Timestamp,
) -> String {
    let timestamp = format_utc_timestamp(now.to_system_time());
    let suffix = format!("_{}.md", timestamp);

    let max_bytes = fs.max_component_bytes();
    let prefix_budget = max_bytes.saturating_sub(suffix.len());

    let truncated = if prefix.len() > prefix_budget {
        let mut end = prefix_budget;
        while end > 0 && !prefix.is_char_boundary(end) {
            end -= 1;
        }
        prefix[..end].trim_end_matches(fs.replacement())
    } else {
        prefix
    };

    if truncated.is_empty() {
        format!("snapshot{}", suffix)
    } else {
        format!("{}{}", truncated, suffix)
    }
}

/// Format a SystemTime as a human-readable UTC timestamp for filenames.
///
/// Produces format: `YYYY-MM-DD_HH-MM-SS` (all filename-safe characters).
fn format_utc_timestamp(time: SystemTime) -> String {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Manual UTC breakdown (no chrono dependency)
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Convert days since epoch to Y-M-D using a civil calendar algorithm
    // Based on Howard Hinnant's algorithm (public domain)
    let (year, month, day) = days_to_civil(days as i64);

    format!(
        "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn require_filesystem_handle(
    inputs: &HashMap<String, Value>,
    key: &str,
) -> Result<filename::FilesystemHandle, ExecError> {
    let value = inputs
        .get(key)
        .ok_or_else(|| ExecError::new(format!("missing '{}' input", key)))?;
    filename::FilesystemHandle::try_from(value)
        .with_exec_context(|| format!("invalid '{}' input", key))
}

fn require_timestamp(inputs: &HashMap<String, Value>, key: &str) -> Result<Timestamp, ExecError> {
    let value = inputs
        .get(key)
        .ok_or_else(|| ExecError::new(format!("missing '{}' input", key)))?;
    Timestamp::try_from(value).with_exec_context(|| format!("invalid '{}' input", key))
}

/// Convert days since Unix epoch to (year, month, day).
///
/// Uses Howard Hinnant's civil_from_days algorithm.
fn days_to_civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Extract gist URL from a transport response.
pub fn extract_gist_url(response: &TransportResponse) -> String {
    match response {
        TransportResponse::Shell(ShellResponse { stdout, .. }) => {
            gunbc_ir::transport::gist::parse_gist_url_from_shell(stdout)
                .unwrap_or_else(|| stdout.trim().to_string())
        }
        TransportResponse::Rest(r) => gunbc_ir::transport::gist::parse_gist_url_from_rest(&r.body)
            .unwrap_or_else(|| "unknown".to_string()),
        _ => "unknown".to_string(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_gist_request() {
        let request = prepare_gist_request("# Test", false, "Test gist", "test.md");

        match request {
            TransportRequest::Shell(req) => {
                assert_eq!(req.command, "gh");
                assert!(req.args.contains(&"gist".to_string()));
                // Verify the filename is used
                assert!(req.args.contains(&"test.md".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_prepare_gist_request_with_branch_filename() {
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let fixed_time = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );
        let filename = generate_gist_filename(&fs, "claude/my-feature", fixed_time);
        let request = prepare_gist_request("# Test", false, "Test gist", &filename);

        match request {
            TransportRequest::Shell(req) => {
                // Filename should start with sanitized branch name
                let f_arg = req
                    .args
                    .iter()
                    .find(|a| a.starts_with("claude-my-feature_"));
                assert!(
                    f_arg.is_some(),
                    "expected filename with sanitized branch name, got args: {:?}",
                    req.args
                );
                assert!(f_arg.unwrap().ends_with(".md"));
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_without_branch() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:fs".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        assert!(result.contains_key("request"));
        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(req))) => {
                // Without branch, filename should start with "snapshot_"
                let f_arg = req.args.iter().find(|a| a.starts_with("snapshot_"));
                assert!(
                    f_arg.is_some(),
                    "expected snapshot filename, got args: {:?}",
                    req.args
                );
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_with_branch() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        inputs.insert(
            "branch".to_string(),
            Value::Str("feature/cool-thing".to_string()),
        );
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:fs".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(req))) => {
                let f_arg = req
                    .args
                    .iter()
                    .find(|a| a.starts_with("feature-cool-thing_"));
                assert!(
                    f_arg.is_some(),
                    "expected branch-based filename, got args: {:?}",
                    req.args
                );
                // Description should include the branch name
                let desc_idx = req.args.iter().position(|a| a == "--desc").unwrap();
                let desc = &req.args[desc_idx + 1];
                assert!(
                    desc.contains("feature/cool-thing"),
                    "description should include original branch name"
                );
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_with_remote_branch_when_detached() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        // No local "branch" — simulates detached HEAD
        inputs.insert(
            "remote_branch".to_string(),
            Value::Str("main".to_string()),
        );
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:fs".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(req))) => {
                // Should use remote_branch ("main") for filename, not "snapshot"
                let f_arg = req.args.iter().find(|a| a.starts_with("main_"));
                assert!(
                    f_arg.is_some(),
                    "expected remote-branch-based filename, got args: {:?}",
                    req.args
                );
                // Description should mention the branch
                let desc_idx = req.args.iter().position(|a| a == "--desc").unwrap();
                let desc = &req.args[desc_idx + 1];
                assert!(
                    desc.contains("main"),
                    "description should include remote branch name"
                );
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_gist_ops_local_branch_preferred_over_remote() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        // Both local and remote — local should win
        inputs.insert(
            "branch".to_string(),
            Value::Str("my-feature".to_string()),
        );
        inputs.insert(
            "remote_branch".to_string(),
            Value::Str("main".to_string()),
        );
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:fs".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(req))) => {
                // Should use local branch, not remote
                let f_arg = req.args.iter().find(|a| a.starts_with("my-feature_"));
                assert!(
                    f_arg.is_some(),
                    "local branch should take priority, got args: {:?}",
                    req.args
                );
            }
            _ => panic!("expected shell request"),
        }
    }

    // ========================================================================
    // Filename sanitization tests
    // ========================================================================

    fn test_fs() -> filename::FilesystemHandle {
        filename::FilesystemHandle::cross_platform(filename::Scope::Write)
    }

    #[test]
    fn test_sanitize_simple_branch() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "main"), "main");
        assert_eq!(sanitize_branch_for_filename(&fs, "develop"), "develop");
        assert_eq!(sanitize_branch_for_filename(&fs, "my-branch"), "my-branch");
    }

    #[test]
    fn test_sanitize_branch_with_slashes() {
        let fs = test_fs();
        assert_eq!(
            sanitize_branch_for_filename(&fs, "claude/branch-name"),
            "claude-branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "feature/foo/bar"),
            "feature-foo-bar"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "refs/heads/main"),
            "refs-heads-main"
        );
    }

    #[test]
    fn test_sanitize_branch_with_spaces() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "my branch"), "my-branch");
        assert_eq!(
            sanitize_branch_for_filename(&fs, "feature/foo bar"),
            "feature-foo-bar"
        );
    }

    #[test]
    fn test_sanitize_branch_windows_unsafe_chars() {
        let fs = test_fs();
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch:name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch*name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch?name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch<name>"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch|name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch\"name"),
            "branch-name"
        );
        assert_eq!(
            sanitize_branch_for_filename(&fs, "branch\\name"),
            "branch-name"
        );
    }

    #[test]
    fn test_sanitize_collapses_consecutive_hyphens() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "a//b"), "a-b");
        assert_eq!(sanitize_branch_for_filename(&fs, "a///b"), "a-b");
        assert_eq!(sanitize_branch_for_filename(&fs, "a/ /b"), "a-b");
    }

    #[test]
    fn test_sanitize_trims_leading_trailing_hyphens() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "/branch"), "branch");
        assert_eq!(sanitize_branch_for_filename(&fs, "branch/"), "branch");
        assert_eq!(sanitize_branch_for_filename(&fs, "/branch/"), "branch");
    }

    #[test]
    fn test_sanitize_empty_and_degenerate() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, ""), "snapshot");
        assert_eq!(sanitize_branch_for_filename(&fs, "/"), "snapshot");
        assert_eq!(sanitize_branch_for_filename(&fs, "///"), "snapshot");
        assert_eq!(sanitize_branch_for_filename(&fs, "   "), "snapshot");
    }

    #[test]
    fn test_sanitize_preserves_dots_and_underscores() {
        let fs = test_fs();
        assert_eq!(sanitize_branch_for_filename(&fs, "v1.0.0"), "v1.0.0");
        assert_eq!(sanitize_branch_for_filename(&fs, "my_branch"), "my_branch");
        assert_eq!(
            sanitize_branch_for_filename(&fs, "release/v2.0_rc1"),
            "release-v2.0_rc1"
        );
    }

    // ========================================================================
    // Timestamp and filename generation tests
    // ========================================================================

    #[test]
    fn test_format_utc_timestamp_known_value() {
        // 2024-01-15 13:30:00 UTC = 1705325400 seconds since epoch
        let time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400);
        let ts = format_utc_timestamp(time);
        assert_eq!(ts, "2024-01-15_13-30-00");
    }

    #[test]
    fn test_format_utc_timestamp_epoch() {
        let ts = format_utc_timestamp(SystemTime::UNIX_EPOCH);
        assert_eq!(ts, "1970-01-01_00-00-00");
    }

    #[test]
    fn test_generate_gist_filename_format() {
        let fs = test_fs();
        let fixed_time = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );
        let filename = generate_gist_filename(&fs, "main", fixed_time);
        assert_eq!(filename, "main_2024-01-15_13-30-00.md");
    }

    #[test]
    fn test_generate_gist_filename_sanitizes_branch() {
        let fs = test_fs();
        let fixed_time = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );
        let filename = generate_gist_filename(&fs, "claude/improve-gist-filename", fixed_time);
        assert_eq!(
            filename,
            "claude-improve-gist-filename_2024-01-15_13-30-00.md"
        );
    }

    // ========================================================================
    // "untitled" sentinel collision — real branch vs degenerate input
    // ========================================================================

    #[test]
    fn test_sanitize_real_untitled_branch_preserved() {
        let fs = test_fs();
        // A real branch named "untitled" must NOT be turned into "snapshot"
        assert_eq!(sanitize_branch_for_filename(&fs, "untitled"), "untitled");
    }

    // ========================================================================
    // Deterministic timestamp tests
    // ========================================================================

    #[test]
    fn test_generate_gist_filename_deterministic_timestamp() {
        let fs = test_fs();
        let t1 = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );
        let t2 = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );

        // Same inputs → same output (deterministic)
        let f1 = generate_gist_filename(&fs, "test", t1);
        let f2 = generate_gist_filename(&fs, "test", t2);
        assert_eq!(f1, f2);
    }

    // ========================================================================
    // Filename length capping
    // ========================================================================

    #[test]
    fn test_generate_gist_filename_caps_total_length() {
        let fs = test_fs();
        let fixed_time = Timestamp::from_system_time(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1705325400),
        );

        // Branch name that's 250 chars — after sanitization still 250 chars
        let long_branch = "a".repeat(250);
        let filename = generate_gist_filename(&fs, &long_branch, fixed_time);

        assert!(
            filename.len() <= 255,
            "filename {} bytes exceeds 255: {}",
            filename.len(),
            filename
        );
        assert!(filename.ends_with("_2024-01-15_13-30-00.md"));
    }

    // ========================================================================
    // Recent mode (base_ref) tests
    // ========================================================================

    #[test]
    fn test_gist_ops_prepare_with_base_ref() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Diff".to_string()));
        inputs.insert("branch".to_string(), Value::Str("main".to_string()));
        inputs.insert(
            "base_ref".to_string(),
            Value::Str("abc123def456".to_string()),
        );
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:fs".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(req))) => {
                // Filename should contain recent-3d and short SHA
                let f_arg = req
                    .args
                    .iter()
                    .find(|a| a.contains("recent-3d") && a.contains("abc123d..HEAD"));
                assert!(
                    f_arg.is_some(),
                    "expected recent-mode filename with commit range, got args: {:?}",
                    req.args
                );
                assert!(f_arg.unwrap().starts_with("main_recent-3d_abc123d..HEAD_"));
                assert!(f_arg.unwrap().ends_with(".md"));

                // Description should mention the commit range
                let desc_idx = req.args.iter().position(|a| a == "--desc").unwrap();
                let desc = &req.args[desc_idx + 1];
                assert!(
                    desc.contains("Recent changes (3d) abc123d..HEAD on main"),
                    "description should contain commit range and branch, got: {}",
                    desc
                );
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_without_base_ref_unchanged() {
        // Verify that snapshot/diff mode behavior is unchanged
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        inputs.insert("branch".to_string(), Value::Str("main".to_string()));
        // No base_ref — snapshot/diff mode
        let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
        let ts = Timestamp::from_system_time(SystemTime::UNIX_EPOCH);
        inputs.insert("res:fs".to_string(), fs.into());
        inputs.insert("res:clock".to_string(), ts.into());

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(req))) => {
                // Should NOT contain recent-3d
                let has_recent = req.args.iter().any(|a| a.contains("recent-3d"));
                assert!(
                    !has_recent,
                    "snapshot mode should not have recent-3d, got args: {:?}",
                    req.args
                );
                // Should start with branch name
                let f_arg = req.args.iter().find(|a| a.starts_with("main_"));
                assert!(f_arg.is_some());
            }
            _ => panic!("expected shell request"),
        }
    }
}
