//! Git operations for gunbc DAGs.
//!
//! Pure operations that build [`TransportRequest`] values or parse
//! [`TransportResponse`] values for git commands. All I/O happens through
//! `TransportOps::Execute` nodes — these ops are PURE.
//!
//! Uses [`GitRequest`] from `gunbc_ir::transport::git` to enforce
//! deterministic, environment-independent git output.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_lib_git_ops::GitOps;
//!
//! // In a DAG graph op enum:
//! GistGraphOp::Git(GitOps::PrepareLsFiles)
//! GistGraphOp::Git(GitOps::ParseLsFiles)
//! ```

use gunbc_exec::{
    optional_str, require_response, ExecError, Executable, OutputMap, TransportResponseExt,
};
use gunbc_ir::transport::git::{self, GitRequest};
use gunbc_ir::Value;
use std::collections::HashMap;

/// Git operations for use in DAG nodes.
///
/// All operations are PURE — no I/O. They build [`TransportRequest`] values
/// or parse [`TransportResponse`] values. Actual I/O happens at
/// `TransportOps::Execute` boundary nodes.
#[derive(Debug, Clone)]
pub enum GitOps {
    // ========================================================================
    // ls-files chain
    // ========================================================================
    /// Build a `git ls-files` request (PURE)
    PrepareLsFiles {
        /// File extensions to filter by (empty = all files).
        /// Converted to pathspec globs: `".rs"` → `":(glob)**/*.rs"`.
        extensions: Vec<String>,
    },
    /// Parse ls-files response into file list (PURE)
    ParseLsFiles,

    // ========================================================================
    // diff chain
    // ========================================================================
    /// Build a `git diff <base_ref>...HEAD` request (PURE)
    PrepareDiff {
        /// Default base ref (can be overridden at runtime via `base_ref` input)
        base_ref: String,
        /// File extensions to filter by (empty = all files).
        extensions: Vec<String>,
    },
    /// Parse unified diff into per-file chunks (PURE)
    ParseDiff,

    // ========================================================================
    // diff --name-only chain
    // ========================================================================
    /// Build a `git diff --name-only <base_ref>...HEAD` request (PURE)
    PrepareDiffNameOnly {
        /// Default base ref (can be overridden at runtime via `base_ref` input)
        base_ref: String,
        /// File extensions to filter by (empty = all files).
        extensions: Vec<String>,
    },
    /// Parse diff name-only response into file list (PURE)
    ParseDiffNameOnly,

    // ========================================================================
    // Utility operations
    // ========================================================================
    /// Build a `git rev-parse --abbrev-ref HEAD` request (PURE)
    PrepareCurrentBranch,
    /// Parse current branch name (PURE)
    ParseCurrentBranch,
}

impl Executable for GitOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            // ================================================================
            // ls-files
            // ================================================================
            GitOps::PrepareLsFiles { extensions } => {
                let repo_path = optional_str(&inputs, "repo_path").unwrap_or(".");

                let mut req = GitRequest::ls_files();
                if !extensions.is_empty() {
                    req = req.extensions(extensions.iter().map(|s| s.as_str()));
                }
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new().request("request", request).ok()
            }
            GitOps::ParseLsFiles => {
                let response = require_response(&inputs, "response")?;
                let shell = response.require_shell()?;

                let files = if shell.exit_code == 0 {
                    git::parse_ls_files(&shell.stdout)
                } else {
                    // Return empty list on failure (could be non-git repo)
                    Vec::new()
                };

                OutputMap::new().str_list("files", files).ok()
            }

            // ================================================================
            // diff
            // ================================================================
            GitOps::PrepareDiff { base_ref, extensions } => {
                let repo_path = optional_str(&inputs, "repo_path").unwrap_or(".");

                // Allow runtime override of base_ref
                let effective_ref =
                    optional_str(&inputs, "base_ref").unwrap_or(base_ref.as_str());

                let mut req = GitRequest::diff(effective_ref);
                if !extensions.is_empty() {
                    req = req.extensions(extensions.iter().map(|s| s.as_str()));
                }
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new().request("request", request).ok()
            }
            GitOps::ParseDiff => {
                let response = require_response(&inputs, "response")?;
                let shell = response.require_shell()?;

                let chunks = git::parse_diff_chunks(&shell.stdout);
                let (adds, dels, count) = git::diff_stats(&chunks);

                OutputMap::new()
                    .map_str_str("diff_files", chunks)
                    .str("stats", format!("+{} -{} across {} files", adds, dels, count))
                    .ok()
            }

            // ================================================================
            // diff --name-only
            // ================================================================
            GitOps::PrepareDiffNameOnly { base_ref, extensions } => {
                let repo_path = optional_str(&inputs, "repo_path").unwrap_or(".");

                let effective_ref =
                    optional_str(&inputs, "base_ref").unwrap_or(base_ref.as_str());

                let mut req = GitRequest::diff_name_only(effective_ref);
                if !extensions.is_empty() {
                    req = req.extensions(extensions.iter().map(|s| s.as_str()));
                }
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new().request("request", request).ok()
            }
            GitOps::ParseDiffNameOnly => {
                let response = require_response(&inputs, "response")?;
                let shell = response.require_shell()?;

                let files = if shell.exit_code == 0 {
                    git::parse_diff_name_only(&shell.stdout)
                } else {
                    Vec::new()
                };

                OutputMap::new().str_list("files", files).ok()
            }

            // ================================================================
            // Utilities
            // ================================================================
            GitOps::PrepareCurrentBranch => {
                let repo_path = optional_str(&inputs, "repo_path").unwrap_or(".");

                let mut req = GitRequest::current_branch();
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new().request("request", request).ok()
            }
            GitOps::ParseCurrentBranch => {
                let response = require_response(&inputs, "response")?;
                let shell = response.require_shell()?;

                let branch = git::parse_current_branch(&shell.stdout);

                OutputMap::new().str("branch", branch).ok()
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{ShellResponse, TransportRequest, TransportResponse};

    fn shell_response(stdout: &str) -> TransportResponse {
        TransportResponse::Shell(ShellResponse {
            exit_code: 0,
            stdout: stdout.to_string(),
            stderr: String::new(),
        })
    }

    fn failed_shell_response() -> TransportResponse {
        TransportResponse::Shell(ShellResponse {
            exit_code: 1,
            stdout: String::new(),
            stderr: "error".to_string(),
        })
    }

    // ========================================================================
    // PrepareLsFiles
    // ========================================================================

    #[test]
    fn test_prepare_ls_files_default() {
        let inputs = HashMap::new();
        let result = GitOps::PrepareLsFiles { extensions: vec![] }.execute(inputs).unwrap();

        let request = result.get("request").unwrap();
        match request {
            Value::Request(TransportRequest::Shell(req)) => {
                assert_eq!(req.command, "git");
                assert!(req.args.contains(&"ls-files".to_string()));
                assert_eq!(req.cwd, None); // "." doesn't set cwd
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_prepare_ls_files_with_path() {
        let mut inputs = HashMap::new();
        inputs.insert("repo_path".to_string(), Value::Str("/my/repo".to_string()));

        let result = GitOps::PrepareLsFiles { extensions: vec![] }.execute(inputs).unwrap();
        let request = result.get("request").unwrap();
        match request {
            Value::Request(TransportRequest::Shell(req)) => {
                assert_eq!(req.cwd, Some("/my/repo".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    // ========================================================================
    // ParseLsFiles
    // ========================================================================

    #[test]
    fn test_parse_ls_files() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(shell_response("src/main.rs\nREADME.md\n")),
        );

        let result = GitOps::ParseLsFiles.execute(inputs).unwrap();
        let files = result.get("files").unwrap().as_str_list().unwrap();
        assert_eq!(files, vec!["src/main.rs", "README.md"]);
    }

    #[test]
    fn test_parse_ls_files_failure() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(failed_shell_response()),
        );

        let result = GitOps::ParseLsFiles.execute(inputs).unwrap();
        let files = result.get("files").unwrap().as_str_list().unwrap();
        assert!(files.is_empty());
    }

    // ========================================================================
    // PrepareDiff
    // ========================================================================

    #[test]
    fn test_prepare_diff_default_ref() {
        let inputs = HashMap::new();
        let op = GitOps::PrepareDiff {
            base_ref: "main".to_string(),
            extensions: vec![],
        };
        let result = op.execute(inputs).unwrap();

        match result.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(req)) => {
                assert!(req.args.contains(&"main...HEAD".to_string()));
                assert!(req.args.contains(&"--no-ext-diff".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_prepare_diff_runtime_override() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "base_ref".to_string(),
            Value::Str("develop".to_string()),
        );

        let op = GitOps::PrepareDiff {
            base_ref: "main".to_string(),
            extensions: vec![],
        };
        let result = op.execute(inputs).unwrap();

        match result.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(req)) => {
                // Should use runtime override, not build-time default
                assert!(req.args.contains(&"develop...HEAD".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    // ========================================================================
    // ParseDiff
    // ========================================================================

    #[test]
    fn test_parse_diff() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!(\"hello\");
 }";

        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(shell_response(diff)),
        );

        let result = GitOps::ParseDiff.execute(inputs).unwrap();

        let diff_files = result.get("diff_files").unwrap().as_map_str_str().unwrap();
        assert_eq!(diff_files.len(), 1);
        assert!(diff_files.contains_key("src/main.rs"));

        let stats = result.get("stats").unwrap().as_str().unwrap();
        assert!(stats.contains("+1"));
    }

    // ========================================================================
    // PrepareCurrentBranch
    // ========================================================================

    #[test]
    fn test_prepare_current_branch() {
        let inputs = HashMap::new();
        let result = GitOps::PrepareCurrentBranch.execute(inputs).unwrap();

        match result.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(req)) => {
                assert!(req.args.contains(&"rev-parse".to_string()));
                assert!(req.args.contains(&"--abbrev-ref".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_parse_current_branch() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(shell_response("feature/my-branch\n")),
        );

        let result = GitOps::ParseCurrentBranch.execute(inputs).unwrap();
        let branch = result.get("branch").unwrap().as_str().unwrap();
        assert_eq!(branch, "feature/my-branch");
    }
}
