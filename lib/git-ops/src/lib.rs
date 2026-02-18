//! Git operations for gunbc DAGs.
//!
//! Pure operations that build [`TransportRequest`] values or parse
//! [`TransportResponse`] values for git commands. All I/O happens through
//! `TransportOps::Execute` nodes — these ops are PURE.
//!
//! Uses [`GitRequest`] from `gunbc_ir::transport::git` to enforce
//! deterministic, environment-independent git output.
//!
//! # SubDag builders
//!
//! - [`build_branch_resolution_subdag`]: Encapsulates `current_branch` +
//!   `remote_branches` as a single SubDag, giving consumers correct
//!   detached-HEAD handling for free.
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

#![deny(dead_code)]
use gunbc_exec::{
    optional_str_strict, propagate_skipped, require_response, require_str, ExecError, Executable,
    OutputMap, TransportResponseExt,
};
use gunbc_ir::build::{optional, port, resource, AccessMode};
use gunbc_ir::dag::{Dag, Edge};
use gunbc_ir::node::Node;
use gunbc_ir::transport::git::{self, GitRequest};
use gunbc_ir::transport::ShellRequest;
use gunbc_ir::Value;
use gunbc_lib_transport::TransportOps;
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

    /// Build a `git branch -r --points-at HEAD` request (PURE)
    ///
    /// Queries which remote tracking branches point at the current commit.
    /// This is a separate question from "what branch are we on?" — it
    /// resolves the remote ref when HEAD is detached.
    PrepareRemoteBranchesAtHead,
    /// Parse remote branches response, extracting the branch name
    /// with remote prefix stripped (PURE)
    ParseRemoteBranchesAtHead,

    // ========================================================================
    // rev-list chain
    // ========================================================================
    /// Build a `git rev-list -1 --before="<before>" HEAD` request (PURE)
    PrepareRevListBefore {
        /// Date expression (e.g., "7 days ago").
        before: String,
    },
    /// Parse rev-list response into optional commit SHA (PURE)
    ///
    /// Emits `base_ref` output only if a commit was found. Empty output
    /// (repo younger than the requested period) produces no output.
    ParseRevListBefore,

    // ========================================================================
    // git show chain
    // ========================================================================
    /// Build a `git show <ref>:<path>` request (PURE).
    ///
    /// Retrieves the contents of a file at a specific ref.
    ///
    /// Inputs: `base_ref` (OptionalString — falls back to `default_ref`)
    /// Outputs: `request` (TransportRequest), `skip` (Bool)
    PrepareGitShow {
        /// Default ref to use when `base_ref` input is absent.
        default_ref: String,
        /// Path within the repo (e.g., ".dag-snapshots/workspace.json").
        path: String,
    },
    /// Parse `git show` response into file content (PURE).
    ///
    /// On success: emits the raw file content as `content`.
    /// On failure: emits an empty string (file not found at ref).
    ///
    /// Inputs: `response` (TransportResponse)
    /// Outputs: `content` (String)
    ParseGitShow,
}

impl Executable for GitOps {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            // ================================================================
            // ls-files
            // ================================================================
            GitOps::PrepareLsFiles { extensions } => {
                let repo_path = require_str(&inputs, "repo_path")?;

                let mut req = GitRequest::ls_files();
                if !extensions.is_empty() {
                    req = req.extensions(extensions.iter().map(|s| s.as_str()));
                }
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .ok()
            }
            GitOps::ParseLsFiles => {
                if let Some(result) = propagate_skipped(&inputs, "response", &["files"]) {
                    return result;
                }
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
            GitOps::PrepareDiff {
                base_ref,
                extensions,
            } => {
                let repo_path = require_str(&inputs, "repo_path")?;

                // Allow runtime override of base_ref
                let effective_ref =
                    optional_str_strict(&inputs, "base_ref")?.unwrap_or(base_ref.as_str());

                let mut req = GitRequest::diff(effective_ref);
                if !extensions.is_empty() {
                    req = req.extensions(extensions.iter().map(|s| s.as_str()));
                }
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .ok()
            }
            GitOps::ParseDiff => {
                if let Some(result) =
                    propagate_skipped(&inputs, "response", &["diff_files", "stats"])
                {
                    return result;
                }
                let response = require_response(&inputs, "response")?;
                let shell = response.require_shell()?;

                let chunks = git::parse_diff_chunks(&shell.stdout);
                let (adds, dels, count) = git::diff_stats(&chunks);

                // Truncate oversized diffs to prevent GitHub API payload rejection.
                // Stats are computed before truncation to reflect the real diff.
                let chunks = git::truncate_diff_chunks(chunks, 500, 5000);

                OutputMap::new()
                    .map_str_str("diff_files", chunks)
                    .str(
                        "stats",
                        format!("+{} -{} across {} files", adds, dels, count),
                    )
                    .ok()
            }

            // ================================================================
            // diff --name-only
            // ================================================================
            GitOps::PrepareDiffNameOnly {
                base_ref,
                extensions,
            } => {
                let repo_path = require_str(&inputs, "repo_path")?;

                let effective_ref =
                    optional_str_strict(&inputs, "base_ref")?.unwrap_or(base_ref.as_str());

                let mut req = GitRequest::diff_name_only(effective_ref);
                if !extensions.is_empty() {
                    req = req.extensions(extensions.iter().map(|s| s.as_str()));
                }
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .ok()
            }
            GitOps::ParseDiffNameOnly => {
                if let Some(result) = propagate_skipped(&inputs, "response", &["files"]) {
                    return result;
                }
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
                let repo_path = require_str(&inputs, "repo_path")?;

                let mut req = GitRequest::current_branch();
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .ok()
            }
            GitOps::ParseCurrentBranch => {
                if let Some(result) = propagate_skipped(&inputs, "response", &["branch"]) {
                    return result;
                }
                let response = require_response(&inputs, "response")?;
                let shell = response.require_shell()?;

                let branch = git::parse_current_branch(&shell.stdout);

                // Treat empty, whitespace-only, and "HEAD" (detached) as unknown.
                // When detached, the parallel RemoteBranchesAtHead chain handles
                // resolution — this node only reports local branch state.
                let mut out = OutputMap::new();
                if !branch.is_empty() && branch != "HEAD" {
                    out = out.str("branch", branch);
                }
                out.ok()
            }

            // ================================================================
            // Remote branches at HEAD
            // ================================================================
            GitOps::PrepareRemoteBranchesAtHead => {
                let repo_path = require_str(&inputs, "repo_path")?;

                let mut req = GitRequest::remote_branches_at_head();
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .ok()
            }
            GitOps::ParseRemoteBranchesAtHead => {
                if let Some(result) = propagate_skipped(&inputs, "response", &["remote_branch"]) {
                    return result;
                }
                let response = require_response(&inputs, "response")?;
                let shell = response.require_shell()?;

                let remote_branch = if shell.exit_code == 0 {
                    git::parse_remote_branches_at_head(&shell.stdout)
                } else {
                    String::new()
                };

                // Only emit if a remote branch was found
                let mut out = OutputMap::new();
                if !remote_branch.is_empty() {
                    out = out.str("remote_branch", remote_branch);
                }
                out.ok()
            }

            // ================================================================
            // rev-list
            // ================================================================
            GitOps::PrepareRevListBefore { before } => {
                let repo_path = require_str(&inputs, "repo_path")?;

                let mut req = GitRequest::rev_list_before(before.as_str());
                if repo_path != "." {
                    req = req.cwd(repo_path);
                }
                let request = req.to_shell_request();

                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .ok()
            }
            GitOps::ParseRevListBefore => {
                if let Some(result) = propagate_skipped(&inputs, "response", &["base_ref"]) {
                    return result;
                }
                let response = require_response(&inputs, "response")?;
                let shell = response.require_shell()?;

                let sha = git::parse_rev_list_before(&shell.stdout);

                // Only emit base_ref if a commit was found
                let mut out = OutputMap::new();
                if !sha.is_empty() {
                    out = out.str("base_ref", sha);
                }
                out.ok()
            }

            // ================================================================
            // git show
            // ================================================================
            GitOps::PrepareGitShow { default_ref, path } => {
                let effective_ref =
                    optional_str_strict(&inputs, "base_ref")?.unwrap_or(default_ref.as_str());

                let git_path = format!("{}:{}", effective_ref, path);
                let request = ShellRequest::new("git")
                    .args(["show", &git_path])
                    .into_transport_request();

                OutputMap::new()
                    .request("request", request)
                    .bool("skip", false)
                    .ok()
            }
            GitOps::ParseGitShow => {
                if let Some(result) = propagate_skipped(&inputs, "response", &["content"]) {
                    return result;
                }
                let response = require_response(&inputs, "response")?;
                let shell = response.require_shell()?;

                let content = if shell.exit_code == 0 {
                    shell.stdout.clone()
                } else {
                    // File not found at ref — return empty string
                    String::new()
                };

                OutputMap::new().str("content", content).ok()
            }
        }
    }
}

// ============================================================================
// SubDag builders
// ============================================================================

/// Operation type for the branch resolution SubDag.
///
/// Wraps `GitOps` (pure) and `TransportOps` (boundary) so the SubDag is
/// self-contained. Consumers `map_ops` this into their own graph-op enum.
#[derive(Debug, Clone)]
pub enum BranchResolutionOp {
    /// Pure git operations (prepare/parse).
    Git(GitOps),
    /// Transport boundary (execute).
    Transport(TransportOps),
}

impl Executable for BranchResolutionOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            BranchResolutionOp::Git(op) => op.execute(inputs),
            BranchResolutionOp::Transport(op) => op.execute(inputs),
        }
    }
}

/// Build a self-contained branch-resolution SubDag.
///
/// The SubDag encapsulates two parallel transport triplets:
///
/// ```text
///   current_branch: PrepareCurrentBranch → Execute → ParseCurrentBranch
///   remote_branches: PrepareRemoteBranchesAtHead → Execute → ParseRemoteBranchesAtHead
/// ```
///
/// **Interface (auto-inferred from entrypoints/boundaries):**
/// - Inputs: `repo_path: String`, `res:file: FilesystemHandle`
/// - Outputs: `branch: OptionalString`, `remote_branch: OptionalString`
///
/// The two triplets run in parallel. `branch` is the local branch name
/// (`None` when HEAD is detached). `remote_branch` is the remote tracking
/// branch that points at HEAD (`None` when not on any remote branch).
///
/// Consumers get correct detached-HEAD handling for free by wiring both
/// outputs to their gist/filename logic.
pub fn build_branch_resolution_subdag() -> Dag<BranchResolutionOp> {
    let mut dag = Dag::new();

    // ========================================================================
    // current_branch triplet
    // ========================================================================
    dag.add_node(Node::opaque(
        "prepare_current_branch",
        vec![port("repo_path", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        BranchResolutionOp::Git(GitOps::PrepareCurrentBranch),
    ));
    dag.add_node(Node::opaque(
        "execute_current_branch",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("file", "FilesystemHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        BranchResolutionOp::Transport(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_current_branch",
        vec![port("response", "TransportResponse")],
        vec![optional("branch", "OptionalString")],
        BranchResolutionOp::Git(GitOps::ParseCurrentBranch),
    ));
    dag.add_edge(Edge::new(
        "prepare_current_branch",
        "request",
        "execute_current_branch",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_current_branch",
        "skip",
        "execute_current_branch",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "execute_current_branch",
        "response",
        "parse_current_branch",
        "response",
    ));

    // ========================================================================
    // remote_branches triplet
    // ========================================================================
    dag.add_node(Node::opaque(
        "prepare_remote_branches",
        vec![port("repo_path", "String")],
        vec![port("request", "TransportRequest"), port("skip", "Bool")],
        BranchResolutionOp::Git(GitOps::PrepareRemoteBranchesAtHead),
    ));
    dag.add_node(Node::opaque(
        "execute_remote_branches",
        vec![
            port("request", "TransportRequest"),
            port("skip", "Bool"),
            resource("file", "FilesystemHandle", AccessMode::Read),
        ],
        vec![port("response", "TransportResponse")],
        BranchResolutionOp::Transport(TransportOps::Execute),
    ));
    dag.add_node(Node::opaque(
        "parse_remote_branches",
        vec![port("response", "TransportResponse")],
        vec![optional("remote_branch", "OptionalString")],
        BranchResolutionOp::Git(GitOps::ParseRemoteBranchesAtHead),
    ));
    dag.add_edge(Edge::new(
        "prepare_remote_branches",
        "request",
        "execute_remote_branches",
        "request",
    ));
    dag.add_edge(Edge::new(
        "prepare_remote_branches",
        "skip",
        "execute_remote_branches",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "execute_remote_branches",
        "response",
        "parse_remote_branches",
        "response",
    ));

    dag
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::{ShellResponse, TransportRequest, TransportResponse};

    fn shell_response(stdout: &str) -> TransportResponse {
        ShellResponse::ok(stdout).into()
    }

    fn failed_shell_response() -> TransportResponse {
        ShellResponse::failed(1, "error").into()
    }

    // ========================================================================
    // PrepareLsFiles
    // ========================================================================

    #[test]
    fn test_prepare_ls_files_default() {
        let mut inputs = HashMap::new();
        inputs.insert("repo_path".to_string(), Value::Str(".".to_string()));
        let result = GitOps::PrepareLsFiles { extensions: vec![] }
            .execute(inputs)
            .unwrap();

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

        let result = GitOps::PrepareLsFiles { extensions: vec![] }
            .execute(inputs)
            .unwrap();
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
        let mut inputs = HashMap::new();
        inputs.insert("repo_path".to_string(), Value::Str(".".to_string()));
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
        inputs.insert("repo_path".to_string(), Value::Str(".".to_string()));
        inputs.insert("base_ref".to_string(), Value::Str("develop".to_string()));

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
        let mut inputs = HashMap::new();
        inputs.insert("repo_path".to_string(), Value::Str(".".to_string()));
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

    // ========================================================================
    // PrepareRemoteBranchesAtHead
    // ========================================================================

    #[test]
    fn test_prepare_remote_branches_at_head() {
        let mut inputs = HashMap::new();
        inputs.insert("repo_path".to_string(), Value::Str(".".to_string()));
        let result = GitOps::PrepareRemoteBranchesAtHead.execute(inputs).unwrap();

        match result.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(req)) => {
                assert!(req.args.contains(&"branch".to_string()));
                assert!(req.args.contains(&"-r".to_string()));
                assert!(req.args.contains(&"--points-at".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_parse_remote_branches_at_head_found() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(shell_response("  origin/main\n")),
        );

        let result = GitOps::ParseRemoteBranchesAtHead.execute(inputs).unwrap();
        let remote = result.get("remote_branch").unwrap().as_str().unwrap();
        assert_eq!(remote, "main");
    }

    #[test]
    fn test_parse_remote_branches_at_head_empty() {
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Response(shell_response("")));

        let result = GitOps::ParseRemoteBranchesAtHead.execute(inputs).unwrap();
        // No remote branch found — output should be absent
        assert!(!result.contains_key("remote_branch"));
    }

    #[test]
    fn test_parse_remote_branches_at_head_failure() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(failed_shell_response()),
        );

        let result = GitOps::ParseRemoteBranchesAtHead.execute(inputs).unwrap();
        assert!(!result.contains_key("remote_branch"));
    }

    // ========================================================================
    // PrepareRevListBefore
    // ========================================================================

    #[test]
    fn test_prepare_rev_list_before() {
        let mut inputs = HashMap::new();
        inputs.insert("repo_path".to_string(), Value::Str(".".to_string()));
        let op = GitOps::PrepareRevListBefore {
            before: "7 days ago".to_string(),
        };
        let result = op.execute(inputs).unwrap();

        match result.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(req)) => {
                assert!(req.args.contains(&"rev-list".to_string()));
                assert!(req.args.contains(&"-1".to_string()));
                assert!(req.args.contains(&"--before=7 days ago".to_string()));
                assert_eq!(req.cwd, None);
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_prepare_rev_list_before_with_path() {
        let mut inputs = HashMap::new();
        inputs.insert("repo_path".to_string(), Value::Str("/my/repo".to_string()));

        let op = GitOps::PrepareRevListBefore {
            before: "7 days ago".to_string(),
        };
        let result = op.execute(inputs).unwrap();

        match result.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(req)) => {
                assert_eq!(req.cwd, Some("/my/repo".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    // ========================================================================
    // ParseRevListBefore
    // ========================================================================

    #[test]
    fn test_parse_rev_list_before_found() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(shell_response("abc123def456\n")),
        );

        let result = GitOps::ParseRevListBefore.execute(inputs).unwrap();
        let base_ref = result.get("base_ref").unwrap().as_str().unwrap();
        assert_eq!(base_ref, "abc123def456");
    }

    #[test]
    fn test_parse_rev_list_before_empty() {
        let mut inputs = HashMap::new();
        inputs.insert("response".to_string(), Value::Response(shell_response("")));

        let result = GitOps::ParseRevListBefore.execute(inputs).unwrap();
        // No commit found — output should be absent
        assert!(!result.contains_key("base_ref"));
    }

    // ========================================================================
    // PrepareGitShow
    // ========================================================================

    #[test]
    fn test_prepare_git_show_default_ref() {
        let inputs = HashMap::new();
        let op = GitOps::PrepareGitShow {
            default_ref: "main".to_string(),
            path: ".dag-snapshots/workspace.json".to_string(),
        };
        let result = op.execute(inputs).unwrap();

        match result.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(req)) => {
                assert_eq!(req.command, "git");
                assert!(req.args.contains(&"show".to_string()));
                assert!(req
                    .args
                    .contains(&"main:.dag-snapshots/workspace.json".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    #[test]
    fn test_prepare_git_show_runtime_override() {
        let mut inputs = HashMap::new();
        inputs.insert("base_ref".to_string(), Value::Str("develop".to_string()));

        let op = GitOps::PrepareGitShow {
            default_ref: "main".to_string(),
            path: "some/file.json".to_string(),
        };
        let result = op.execute(inputs).unwrap();

        match result.get("request").unwrap() {
            Value::Request(TransportRequest::Shell(req)) => {
                assert!(req.args.contains(&"develop:some/file.json".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }

    // ========================================================================
    // ParseGitShow
    // ========================================================================

    #[test]
    fn test_parse_git_show_success() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(shell_response("{\"nodes\":[]}\n")),
        );

        let result = GitOps::ParseGitShow.execute(inputs).unwrap();
        let content = result.get("content").unwrap().as_str().unwrap();
        assert_eq!(content, "{\"nodes\":[]}\n");
    }

    #[test]
    fn test_parse_git_show_failure() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "response".to_string(),
            Value::Response(failed_shell_response()),
        );

        let result = GitOps::ParseGitShow.execute(inputs).unwrap();
        let content = result.get("content").unwrap().as_str().unwrap();
        assert_eq!(content, "");
    }

    // ========================================================================
    // build_branch_resolution_subdag
    // ========================================================================

    #[test]
    fn test_branch_resolution_subdag_structure() {
        let dag = build_branch_resolution_subdag();

        // Should have 6 nodes: 3 per triplet
        assert_eq!(
            dag.nodes.len(),
            6,
            "branch resolution SubDag should have 6 nodes"
        );

        // Check all expected nodes exist
        assert!(dag.get_node(&"prepare_current_branch".into()).is_some());
        assert!(dag.get_node(&"execute_current_branch".into()).is_some());
        assert!(dag.get_node(&"parse_current_branch".into()).is_some());
        assert!(dag.get_node(&"prepare_remote_branches".into()).is_some());
        assert!(dag.get_node(&"execute_remote_branches".into()).is_some());
        assert!(dag.get_node(&"parse_remote_branches".into()).is_some());

        // Check edges: 3 per triplet = 6 total
        assert_eq!(
            dag.edges.len(),
            6,
            "branch resolution SubDag should have 6 edges"
        );
    }

    #[test]
    fn test_branch_resolution_subdag_as_node() {
        // Wrap in Node::subdag and verify auto-inferred interface
        let inner = build_branch_resolution_subdag();
        let node = Node::subdag("branch_resolution", inner);

        // Inputs: repo_path (x2 deduplicated to 1), res:file (x2 deduplicated to 1)
        let input_names: Vec<&str> = node.inputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(
            input_names.contains(&"repo_path"),
            "should expose repo_path input, got {:?}",
            input_names
        );
        assert!(
            input_names.contains(&"res:file"),
            "should expose res:file input, got {:?}",
            input_names
        );

        // Outputs: branch, remote_branch
        let output_names: Vec<&str> = node.outputs.iter().map(|p| p.name.0.as_str()).collect();
        assert!(
            output_names.contains(&"branch"),
            "should expose branch output, got {:?}",
            output_names
        );
        assert!(
            output_names.contains(&"remote_branch"),
            "should expose remote_branch output, got {:?}",
            output_names
        );
    }
}
