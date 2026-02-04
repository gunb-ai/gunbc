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

use gunbc_exec::{optional_str, require_response, require_str, ExecError, Executable, OutputMap};
use gunbc_ir::transport::gist::GistRequest;
use gunbc_ir::transport::{ShellResponse, TransportRequest, TransportResponse};
use gunbc_ir::Value;
use std::collections::HashMap;
use std::time::SystemTime;

/// Gist operations for use in DAG nodes.
///
/// All operations are PURE - no I/O. Use TransportOps::Execute for actual I/O.
#[derive(Debug, Clone)]
pub enum GistOps {
    /// Prepare a gist creation request (PURE - no I/O)
    PrepareRequest { public: bool },
    /// Parse gist response to extract URL (PURE - no I/O)
    ParseGistResponse,
}

impl Executable for GistOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistOps::PrepareRequest { public } => {
                let markdown = require_str(&inputs, "markdown")?;
                let branch = optional_str(&inputs, "branch");

                let filename = generate_gist_filename(branch.unwrap_or("snapshot"));
                let description = match branch {
                    Some(b) => format!("Code snapshot of {} created by gunbc-gist", b),
                    None => "Code snapshot created by gunbc-gist".to_string(),
                };

                let request =
                    prepare_gist_request(markdown, *public, &description, &filename);

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

/// Sanitize a branch name for use in a filename.
///
/// Branch names like `claude/branch-name` or `feature/foo bar` contain
/// characters that are problematic in filenames across platforms. This
/// function replaces them with hyphens and normalizes the result.
///
/// Rules:
/// - Replace `/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`, spaces with `-`
/// - Collapse consecutive hyphens into one
/// - Trim leading/trailing hyphens
/// - Fall back to `"snapshot"` if the result is empty
///
/// # Examples
///
/// ```ignore
/// assert_eq!(sanitize_branch_for_filename("main"), "main");
/// assert_eq!(sanitize_branch_for_filename("claude/branch-name"), "claude-branch-name");
/// assert_eq!(sanitize_branch_for_filename("feature/foo bar"), "feature-foo-bar");
/// ```
pub fn sanitize_branch_for_filename(branch: &str) -> String {
    let sanitized: String = branch
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | ' ' => '-',
            _ => c,
        })
        .collect();

    // Collapse consecutive hyphens
    let mut result = String::with_capacity(sanitized.len());
    let mut prev_hyphen = false;
    for c in sanitized.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push(c);
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    // Trim leading/trailing hyphens
    let trimmed = result.trim_matches('-');

    if trimmed.is_empty() {
        "snapshot".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Generate a gist filename from a branch name.
///
/// Produces a filename like `main_2024-01-15_14-30-00.md` by sanitizing
/// the branch name and appending a human-readable UTC timestamp.
///
/// # Examples
///
/// ```ignore
/// // With branch "claude/my-feature":
/// // => "claude-my-feature_2024-01-15_14-30-00.md"
/// let filename = generate_gist_filename("claude/my-feature");
/// assert!(filename.starts_with("claude-my-feature_"));
/// assert!(filename.ends_with(".md"));
/// ```
pub fn generate_gist_filename(branch: &str) -> String {
    let sanitized = sanitize_branch_for_filename(branch);
    let timestamp = format_utc_timestamp(SystemTime::now());
    format!("{}_{}.md", sanitized, timestamp)
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
        TransportResponse::Rest(r) => {
            gunbc_ir::transport::gist::parse_gist_url_from_rest(&r.body)
                .unwrap_or_else(|| "unknown".to_string())
        }
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
        let filename = generate_gist_filename("claude/my-feature");
        let request = prepare_gist_request("# Test", false, "Test gist", &filename);

        match request {
            TransportRequest::Shell(req) => {
                // Filename should start with sanitized branch name
                let f_arg = req.args.iter().find(|a| a.starts_with("claude-my-feature_"));
                assert!(f_arg.is_some(), "expected filename with sanitized branch name, got args: {:?}", req.args);
                assert!(f_arg.unwrap().ends_with(".md"));
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_without_branch() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        assert!(result.contains_key("request"));
        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(req))) => {
                // Without branch, filename should start with "snapshot_"
                let f_arg = req.args.iter().find(|a| a.starts_with("snapshot_"));
                assert!(f_arg.is_some(), "expected snapshot filename, got args: {:?}", req.args);
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_gist_ops_prepare_with_branch() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));
        inputs.insert("branch".to_string(), Value::Str("feature/cool-thing".to_string()));

        let op = GistOps::PrepareRequest { public: false };
        let result = op.execute(inputs).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(req))) => {
                let f_arg = req.args.iter().find(|a| a.starts_with("feature-cool-thing_"));
                assert!(f_arg.is_some(), "expected branch-based filename, got args: {:?}", req.args);
                // Description should include the branch name
                let desc_idx = req.args.iter().position(|a| a == "--desc").unwrap();
                let desc = &req.args[desc_idx + 1];
                assert!(desc.contains("feature/cool-thing"), "description should include original branch name");
            }
            _ => panic!("expected shell request"),
        }
    }

    // ========================================================================
    // Filename sanitization tests
    // ========================================================================

    #[test]
    fn test_sanitize_simple_branch() {
        assert_eq!(sanitize_branch_for_filename("main"), "main");
        assert_eq!(sanitize_branch_for_filename("develop"), "develop");
        assert_eq!(sanitize_branch_for_filename("my-branch"), "my-branch");
    }

    #[test]
    fn test_sanitize_branch_with_slashes() {
        assert_eq!(sanitize_branch_for_filename("claude/branch-name"), "claude-branch-name");
        assert_eq!(sanitize_branch_for_filename("feature/foo/bar"), "feature-foo-bar");
        assert_eq!(sanitize_branch_for_filename("refs/heads/main"), "refs-heads-main");
    }

    #[test]
    fn test_sanitize_branch_with_spaces() {
        assert_eq!(sanitize_branch_for_filename("my branch"), "my-branch");
        assert_eq!(sanitize_branch_for_filename("feature/foo bar"), "feature-foo-bar");
    }

    #[test]
    fn test_sanitize_branch_windows_unsafe_chars() {
        assert_eq!(sanitize_branch_for_filename("branch:name"), "branch-name");
        assert_eq!(sanitize_branch_for_filename("branch*name"), "branch-name");
        assert_eq!(sanitize_branch_for_filename("branch?name"), "branch-name");
        assert_eq!(sanitize_branch_for_filename("branch<name>"), "branch-name");
        assert_eq!(sanitize_branch_for_filename("branch|name"), "branch-name");
        assert_eq!(sanitize_branch_for_filename("branch\"name"), "branch-name");
        assert_eq!(sanitize_branch_for_filename("branch\\name"), "branch-name");
    }

    #[test]
    fn test_sanitize_collapses_consecutive_hyphens() {
        assert_eq!(sanitize_branch_for_filename("a//b"), "a-b");
        assert_eq!(sanitize_branch_for_filename("a///b"), "a-b");
        assert_eq!(sanitize_branch_for_filename("a/ /b"), "a-b");
    }

    #[test]
    fn test_sanitize_trims_leading_trailing_hyphens() {
        assert_eq!(sanitize_branch_for_filename("/branch"), "branch");
        assert_eq!(sanitize_branch_for_filename("branch/"), "branch");
        assert_eq!(sanitize_branch_for_filename("/branch/"), "branch");
    }

    #[test]
    fn test_sanitize_empty_and_degenerate() {
        assert_eq!(sanitize_branch_for_filename(""), "snapshot");
        assert_eq!(sanitize_branch_for_filename("/"), "snapshot");
        assert_eq!(sanitize_branch_for_filename("///"), "snapshot");
        assert_eq!(sanitize_branch_for_filename("   "), "snapshot");
    }

    #[test]
    fn test_sanitize_preserves_dots_and_underscores() {
        assert_eq!(sanitize_branch_for_filename("v1.0.0"), "v1.0.0");
        assert_eq!(sanitize_branch_for_filename("my_branch"), "my_branch");
        assert_eq!(sanitize_branch_for_filename("release/v2.0_rc1"), "release-v2.0_rc1");
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
        let filename = generate_gist_filename("main");
        assert!(filename.starts_with("main_"), "expected 'main_' prefix, got: {}", filename);
        assert!(filename.ends_with(".md"), "expected '.md' suffix, got: {}", filename);
        // Should match pattern: main_YYYY-MM-DD_HH-MM-SS.md
        assert_eq!(filename.len(), "main_".len() + "YYYY-MM-DD_HH-MM-SS".len() + ".md".len());
    }

    #[test]
    fn test_generate_gist_filename_sanitizes_branch() {
        let filename = generate_gist_filename("claude/improve-gist-filename");
        assert!(filename.starts_with("claude-improve-gist-filename_"), "got: {}", filename);
        assert!(filename.ends_with(".md"));
    }
}
