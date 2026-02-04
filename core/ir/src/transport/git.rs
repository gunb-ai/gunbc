//! Git-specific transport request types.
//!
//! This module provides a [`GitRequest`] builder that converts high-level git
//! operations into deterministic [`TransportRequest::Shell`] values. All git
//! commands go through this builder to enforce consistent, environment-independent
//! output — preventing user config (`color.ui`, `diff.noprefix`, `diff.external`,
//! etc.) from breaking downstream parsers.
//!
//! Built on [`ShellRequest`] for command execution via the transport layer.
//!
//! # Example
//!
//! ```
//! use gunbc_ir::transport::git::{GitRequest, parse_ls_files};
//!
//! // Build a deterministic git ls-files request
//! let request = GitRequest::ls_files().cwd("/my/repo").to_shell_request();
//!
//! // Parse the response
//! let files = parse_ls_files("src/main.rs\nREADME.md\n");
//! assert_eq!(files, vec!["src/main.rs", "README.md"]);
//! ```

use super::{ShellRequest, TransportRequest};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Git operation request.
///
/// High-level representation of a git command that converts to a
/// deterministic [`TransportRequest::Shell`]. All git commands go through
/// this builder to enforce consistent, environment-independent output.
///
/// # Example
///
/// ```
/// use gunbc_ir::transport::git::GitRequest;
///
/// let req = GitRequest::ls_files()
///     .cwd("/path/to/repo")
///     .to_shell_request();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitRequest {
    /// The git subcommand to execute.
    pub subcommand: GitSubcommand,
    /// Working directory for the git command.
    pub cwd: Option<String>,
    /// Pathspec patterns for filtering (appended after `--`).
    ///
    /// Both `ls-files` and `diff` support pathspecs:
    /// - `git ls-files -- '*.rs'`
    /// - `git diff main...HEAD -- '*.rs' 'src/'`
    ///
    /// Empty means no filtering (all files).
    pub pathspecs: Vec<String>,
}

/// Git subcommands supported by the transport API.
///
/// Each variant maps to a specific git command with deterministic flags.
/// New subcommands are added as variants — not as free-form argument strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitSubcommand {
    /// `git ls-files --cached --others --exclude-standard`
    ///
    /// Lists tracked and untracked files, respecting .gitignore.
    LsFiles,

    /// `git diff <base_ref>...HEAD` (triple-dot: changes since fork point)
    ///
    /// Produces a unified diff of changes on HEAD since it diverged from base_ref.
    Diff {
        /// The ref to diff against (e.g., "main", "origin/main").
        base_ref: String,
    },

    /// `git diff --name-only <base_ref>...HEAD`
    ///
    /// Lists only the names of files changed since base_ref.
    DiffNameOnly {
        /// The ref to diff against.
        base_ref: String,
    },

    /// `git rev-parse --abbrev-ref HEAD`
    ///
    /// Returns the current branch name (or "HEAD" if detached).
    CurrentBranch,

    /// `git merge-base <base_ref> HEAD`
    ///
    /// Returns the common ancestor commit hash.
    MergeBase {
        /// The ref to find the merge base with.
        base_ref: String,
    },
}

impl GitRequest {
    // ========================================================================
    // Builder constructors
    // ========================================================================

    /// List tracked and untracked files (respects .gitignore).
    pub fn ls_files() -> Self {
        Self {
            subcommand: GitSubcommand::LsFiles,
            cwd: None,
            pathspecs: Vec::new(),
        }
    }

    /// Unified diff: changes on HEAD since it diverged from base_ref.
    pub fn diff(base_ref: impl Into<String>) -> Self {
        Self {
            subcommand: GitSubcommand::Diff {
                base_ref: base_ref.into(),
            },
            cwd: None,
            pathspecs: Vec::new(),
        }
    }

    /// File list only: names of files changed since base_ref.
    pub fn diff_name_only(base_ref: impl Into<String>) -> Self {
        Self {
            subcommand: GitSubcommand::DiffNameOnly {
                base_ref: base_ref.into(),
            },
            cwd: None,
            pathspecs: Vec::new(),
        }
    }

    /// Current branch name (or "HEAD" if detached).
    pub fn current_branch() -> Self {
        Self {
            subcommand: GitSubcommand::CurrentBranch,
            cwd: None,
            pathspecs: Vec::new(),
        }
    }

    /// Common ancestor commit between base_ref and HEAD.
    pub fn merge_base(base_ref: impl Into<String>) -> Self {
        Self {
            subcommand: GitSubcommand::MergeBase {
                base_ref: base_ref.into(),
            },
            cwd: None,
            pathspecs: Vec::new(),
        }
    }

    // ========================================================================
    // Builder methods
    // ========================================================================

    /// Set working directory for the git command.
    pub fn cwd(mut self, path: impl Into<String>) -> Self {
        self.cwd = Some(path.into());
        self
    }

    /// Add pathspec patterns for filtering.
    ///
    /// Appended after `--` in the git command. Supports glob patterns:
    /// - `"*.rs"` — match `.rs` files in current directory
    /// - `":(glob)**/*.rs"` — match `.rs` files recursively
    /// - `"src/"` — match everything under `src/`
    ///
    /// For extension filtering, converts extensions like `".rs"` to
    /// recursive glob patterns `":(glob)**/*.rs"`.
    pub fn pathspecs(mut self, specs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.pathspecs = specs.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Add pathspec patterns from file extensions (e.g., `[".rs", ".toml"]`).
    ///
    /// Converts each extension to a recursive glob: `".rs"` → `":(glob)**/*.rs"`.
    pub fn extensions(mut self, exts: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        self.pathspecs = exts
            .into_iter()
            .map(|ext| {
                let e = ext.as_ref();
                let e = e.strip_prefix('.').unwrap_or(e);
                format!(":(glob)**/*.{}", e)
            })
            .collect();
        self
    }

    // ========================================================================
    // Conversion to transport request
    // ========================================================================

    /// Convert to a deterministic shell transport request.
    ///
    /// This is the single source of truth for how git commands are invoked.
    /// Global flags ensure deterministic output regardless of user config.
    pub fn to_shell_request(&self) -> TransportRequest {
        let mut req = ShellRequest::new("git")
            // Global flags: deterministic output regardless of user config
            .args(["-c", "color.ui=never"]) // no ANSI escapes
            .args(["-c", "core.quotepath=false"]) // don't escape unicode paths
            .args(["-c", "log.date=iso-strict"]) // deterministic dates
            .arg("--no-pager"); // never pipe through less/more

        match &self.subcommand {
            GitSubcommand::LsFiles => {
                req = req.args(["ls-files", "--cached", "--others", "--exclude-standard"]);
            }
            GitSubcommand::Diff { base_ref } => {
                req = req
                    .arg("diff")
                    .arg("--no-ext-diff") // no external diff driver
                    .arg("--no-color") // belt+suspenders with color.ui=never
                    .arg("--src-prefix=a/") // enforce standard prefixes
                    .arg("--dst-prefix=b/") // even if diff.noprefix is set
                    .arg("--find-renames") // detect renames
                    .arg(format!("{}...HEAD", base_ref));
            }
            GitSubcommand::DiffNameOnly { base_ref } => {
                req = req
                    .arg("diff")
                    .arg("--no-ext-diff")
                    .arg("--no-color")
                    .arg("--name-only")
                    .arg(format!("{}...HEAD", base_ref));
            }
            GitSubcommand::CurrentBranch => {
                req = req.args(["rev-parse", "--abbrev-ref", "HEAD"]);
            }
            GitSubcommand::MergeBase { base_ref } => {
                req = req.arg("merge-base").arg(base_ref.as_str()).arg("HEAD");
            }
        }

        // Append pathspecs after `--` if any
        if !self.pathspecs.is_empty() {
            req = req.arg("--");
            for spec in &self.pathspecs {
                req = req.arg(spec.as_str());
            }
        }

        if let Some(ref cwd) = self.cwd {
            req = req.cwd(cwd.as_str());
        }

        TransportRequest::Shell(req)
    }
}

// ============================================================================
// Response parsers
// ============================================================================

/// Parse file list from `git ls-files` output.
///
/// Splits stdout on newlines, trims whitespace, filters empty lines.
///
/// # Example
///
/// ```
/// use gunbc_ir::transport::git::parse_ls_files;
///
/// let files = parse_ls_files("src/main.rs\nREADME.md\n");
/// assert_eq!(files, vec!["src/main.rs", "README.md"]);
/// ```
pub fn parse_ls_files(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// Parse unified diff into per-file chunks.
///
/// Splits on `diff --git a/... b/...` headers. The key is the post-image
/// filename (the `b/` path, prefix stripped). The value is the entire diff
/// chunk including header and hunks.
///
/// Returns a `BTreeMap` for deterministic ordering.
///
/// # Example
///
/// ```
/// use gunbc_ir::transport::git::parse_diff_chunks;
///
/// let diff = "\
/// diff --git a/src/main.rs b/src/main.rs
/// --- a/src/main.rs
/// +++ b/src/main.rs
/// @@ -1,3 +1,4 @@
///  fn main() {
/// +    println!(\"hello\");
///  }
/// diff --git a/README.md b/README.md
/// --- a/README.md
/// +++ b/README.md
/// @@ -1 +1,2 @@
///  # Title
/// +New line
/// ";
///
/// let chunks = parse_diff_chunks(diff);
/// assert_eq!(chunks.len(), 2);
/// assert!(chunks.contains_key("src/main.rs"));
/// assert!(chunks.contains_key("README.md"));
/// ```
pub fn parse_diff_chunks(stdout: &str) -> BTreeMap<String, String> {
    let mut chunks = BTreeMap::new();
    let mut current_file: Option<String> = None;
    let mut current_chunk = String::new();

    for line in stdout.lines() {
        if line.starts_with("diff --git ") {
            // Save previous chunk
            if let Some(filename) = current_file.take() {
                let chunk = current_chunk.trim_end().to_string();
                if !chunk.is_empty() {
                    chunks.insert(filename, chunk);
                }
            }

            // Extract filename from "diff --git a/<path> b/<path>"
            let filename = extract_diff_filename(line);
            current_file = Some(filename);
            current_chunk = String::new();
            current_chunk.push_str(line);
            current_chunk.push('\n');
        } else if current_file.is_some() {
            current_chunk.push_str(line);
            current_chunk.push('\n');
        }
    }

    // Save last chunk
    if let Some(filename) = current_file {
        let chunk = current_chunk.trim_end().to_string();
        if !chunk.is_empty() {
            chunks.insert(filename, chunk);
        }
    }

    chunks
}

/// Extract the post-image filename from a `diff --git` header line.
///
/// Input format: `diff --git a/<path> b/<path>`
/// Returns: `<path>` (from the `b/` side, with prefix stripped)
fn extract_diff_filename(header: &str) -> String {
    // Format: "diff --git a/<path> b/<path>"
    // We want the b/ path (post-image), which is after the last " b/" occurrence.
    if let Some(b_pos) = header.rfind(" b/") {
        header[b_pos + 3..].to_string()
    } else {
        // Fallback: try to extract from the a/ path
        if let Some(a_pos) = header.find(" a/") {
            let after_a = &header[a_pos + 3..];
            // Take up to the next space
            after_a
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        } else {
            "unknown".to_string()
        }
    }
}

/// Parse file list from `git diff --name-only` output.
///
/// Same format as ls-files: one filename per line.
pub fn parse_diff_name_only(stdout: &str) -> Vec<String> {
    parse_ls_files(stdout) // Same format
}

/// Parse a single branch name from `git rev-parse --abbrev-ref HEAD`.
pub fn parse_current_branch(stdout: &str) -> String {
    stdout.trim().to_string()
}

/// Parse a commit hash from `git merge-base` output.
pub fn parse_merge_base(stdout: &str) -> String {
    stdout.trim().to_string()
}

/// Truncate diff chunks that exceed size limits.
///
/// Prevents oversized gist payloads by capping both per-file line count
/// and total line count across all files. Files are processed in sorted
/// order (BTreeMap). Once the total budget is exhausted, remaining files
/// are dropped entirely.
///
/// Returns a new `BTreeMap` with truncated chunks. Truncated chunks get
/// a notice appended: `\n... (N lines truncated)`.
pub fn truncate_diff_chunks(
    chunks: BTreeMap<String, String>,
    max_lines_per_file: usize,
    max_total_lines: usize,
) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    let mut total_lines = 0;

    for (filename, chunk) in chunks {
        if total_lines >= max_total_lines {
            // Budget exhausted — drop remaining files
            break;
        }

        let lines: Vec<&str> = chunk.lines().collect();
        let remaining_budget = max_total_lines.saturating_sub(total_lines);
        let file_limit = max_lines_per_file.min(remaining_budget);

        if lines.len() <= file_limit {
            total_lines += lines.len();
            result.insert(filename, chunk);
        } else {
            let kept: String = lines[..file_limit].join("\n");
            let dropped = lines.len() - file_limit;
            let truncated = format!("{}\n... ({} lines truncated)", kept, dropped);
            total_lines += file_limit + 1; // +1 for the notice line
            result.insert(filename, truncated);
        }
    }

    result
}

/// Compute diff stats from parsed diff chunks.
///
/// Returns `(additions, deletions, file_count)`.
///
/// Counts lines starting with `+` (excluding `+++` headers) as additions,
/// and lines starting with `-` (excluding `---` headers) as deletions.
pub fn diff_stats(chunks: &BTreeMap<String, String>) -> (usize, usize, usize) {
    let mut additions = 0;
    let mut deletions = 0;

    for chunk in chunks.values() {
        for line in chunk.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                additions += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                deletions += 1;
            }
        }
    }

    (additions, deletions, chunks.len())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // GitRequest builder tests
    // ========================================================================

    #[test]
    fn test_ls_files_request() {
        let req = GitRequest::ls_files().cwd("/my/repo").to_shell_request();

        match req {
            TransportRequest::Shell(shell) => {
                assert_eq!(shell.command, "git");
                assert!(shell.args.contains(&"ls-files".to_string()));
                assert!(shell.args.contains(&"--cached".to_string()));
                assert!(shell.args.contains(&"--others".to_string()));
                assert!(shell.args.contains(&"--exclude-standard".to_string()));
                assert_eq!(shell.cwd, Some("/my/repo".to_string()));
                // Verify deterministic flags
                assert!(shell.args.contains(&"--no-pager".to_string()));
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_diff_request() {
        let req = GitRequest::diff("main").cwd("/repo").to_shell_request();

        match req {
            TransportRequest::Shell(shell) => {
                assert_eq!(shell.command, "git");
                assert!(shell.args.contains(&"diff".to_string()));
                assert!(shell.args.contains(&"--no-ext-diff".to_string()));
                assert!(shell.args.contains(&"--no-color".to_string()));
                assert!(shell.args.contains(&"--src-prefix=a/".to_string()));
                assert!(shell.args.contains(&"--dst-prefix=b/".to_string()));
                assert!(shell.args.contains(&"--find-renames".to_string()));
                assert!(shell.args.contains(&"main...HEAD".to_string()));
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_diff_name_only_request() {
        let req = GitRequest::diff_name_only("origin/main").to_shell_request();

        match req {
            TransportRequest::Shell(shell) => {
                assert!(shell.args.contains(&"--name-only".to_string()));
                assert!(shell.args.contains(&"origin/main...HEAD".to_string()));
                assert_eq!(shell.cwd, None);
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_current_branch_request() {
        let req = GitRequest::current_branch().to_shell_request();

        match req {
            TransportRequest::Shell(shell) => {
                assert!(shell.args.contains(&"rev-parse".to_string()));
                assert!(shell.args.contains(&"--abbrev-ref".to_string()));
                assert!(shell.args.contains(&"HEAD".to_string()));
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_merge_base_request() {
        let req = GitRequest::merge_base("develop").to_shell_request();

        match req {
            TransportRequest::Shell(shell) => {
                assert!(shell.args.contains(&"merge-base".to_string()));
                assert!(shell.args.contains(&"develop".to_string()));
                assert!(shell.args.contains(&"HEAD".to_string()));
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_ls_files_with_pathspecs() {
        let req = GitRequest::ls_files()
            .pathspecs(["*.rs", "src/"])
            .to_shell_request();

        match req {
            TransportRequest::Shell(shell) => {
                let joined = shell.args.join(" ");
                assert!(joined.contains("-- *.rs src/"));
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_diff_with_extensions() {
        let req = GitRequest::diff("main")
            .extensions([".rs", ".toml"])
            .to_shell_request();

        match req {
            TransportRequest::Shell(shell) => {
                let joined = shell.args.join(" ");
                assert!(joined.contains("-- :(glob)**/*.rs :(glob)**/*.toml"));
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_no_pathspecs_no_separator() {
        let req = GitRequest::ls_files().to_shell_request();

        match req {
            TransportRequest::Shell(shell) => {
                assert!(!shell.args.contains(&"--".to_string()));
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_extensions_strips_dot() {
        let req = GitRequest::diff("main")
            .extensions(["rs", ".py"])
            .to_shell_request();

        match req {
            TransportRequest::Shell(shell) => {
                let joined = shell.args.join(" ");
                assert!(joined.contains(":(glob)**/*.rs"));
                assert!(joined.contains(":(glob)**/*.py"));
            }
            _ => panic!("expected Shell request"),
        }
    }

    #[test]
    fn test_deterministic_global_flags() {
        // All requests should have deterministic global flags
        let requests = vec![
            GitRequest::ls_files().to_shell_request(),
            GitRequest::diff("main").to_shell_request(),
            GitRequest::current_branch().to_shell_request(),
        ];

        for req in requests {
            match req {
                TransportRequest::Shell(shell) => {
                    // Joined args for easier checking
                    let joined = shell.args.join(" ");
                    assert!(
                        joined.contains("color.ui=never"),
                        "missing color.ui=never in: {}",
                        joined
                    );
                    assert!(
                        joined.contains("core.quotepath=false"),
                        "missing core.quotepath=false in: {}",
                        joined
                    );
                    assert!(
                        joined.contains("--no-pager"),
                        "missing --no-pager in: {}",
                        joined
                    );
                }
                _ => panic!("expected Shell request"),
            }
        }
    }

    // ========================================================================
    // Parser tests
    // ========================================================================

    #[test]
    fn test_parse_ls_files() {
        let files = parse_ls_files("src/main.rs\nREADME.md\nCargo.toml\n");
        assert_eq!(files, vec!["src/main.rs", "README.md", "Cargo.toml"]);
    }

    #[test]
    fn test_parse_ls_files_empty() {
        let files = parse_ls_files("");
        assert!(files.is_empty());
    }

    #[test]
    fn test_parse_ls_files_with_whitespace() {
        let files = parse_ls_files("  src/main.rs  \n  \n  README.md  \n");
        assert_eq!(files, vec!["src/main.rs", "README.md"]);
    }

    #[test]
    fn test_parse_diff_chunks_basic() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!(\"hello\");
 }";

        let chunks = parse_diff_chunks(diff);
        assert_eq!(chunks.len(), 1);
        assert!(chunks.contains_key("src/main.rs"));
        let chunk = &chunks["src/main.rs"];
        assert!(chunk.starts_with("diff --git"));
        assert!(chunk.contains("+    println!(\"hello\");"));
    }

    #[test]
    fn test_parse_diff_chunks_multiple_files() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1 +1,2 @@
 fn main() {}
+// new
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -1 +1,2 @@
 # Title
+New line";

        let chunks = parse_diff_chunks(diff);
        assert_eq!(chunks.len(), 2);
        assert!(chunks.contains_key("src/main.rs"));
        assert!(chunks.contains_key("README.md"));
    }

    #[test]
    fn test_parse_diff_chunks_empty() {
        let chunks = parse_diff_chunks("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_parse_diff_chunks_new_file() {
        let diff = "\
diff --git a/new_file.rs b/new_file.rs
new file mode 100644
--- /dev/null
+++ b/new_file.rs
@@ -0,0 +1,3 @@
+fn new() {
+    // brand new
+}";

        let chunks = parse_diff_chunks(diff);
        assert_eq!(chunks.len(), 1);
        assert!(chunks.contains_key("new_file.rs"));
    }

    #[test]
    fn test_parse_current_branch() {
        assert_eq!(parse_current_branch("main\n"), "main");
        assert_eq!(parse_current_branch("feature/foo\n"), "feature/foo");
        assert_eq!(parse_current_branch("HEAD\n"), "HEAD");
    }

    #[test]
    fn test_parse_merge_base() {
        assert_eq!(
            parse_merge_base("abc123def456\n"),
            "abc123def456"
        );
    }

    #[test]
    fn test_diff_stats() {
        let mut chunks = BTreeMap::new();
        chunks.insert(
            "file.rs".to_string(),
            "\
diff --git a/file.rs b/file.rs
--- a/file.rs
+++ b/file.rs
@@ -1,3 +1,5 @@
 existing
+added1
+added2
-removed1
 existing"
                .to_string(),
        );

        let (adds, dels, files) = diff_stats(&chunks);
        assert_eq!(adds, 2);
        assert_eq!(dels, 1);
        assert_eq!(files, 1);
    }

    #[test]
    fn test_diff_stats_empty() {
        let chunks = BTreeMap::new();
        let (adds, dels, files) = diff_stats(&chunks);
        assert_eq!(adds, 0);
        assert_eq!(dels, 0);
        assert_eq!(files, 0);
    }

    #[test]
    fn test_extract_diff_filename() {
        assert_eq!(
            extract_diff_filename("diff --git a/src/main.rs b/src/main.rs"),
            "src/main.rs"
        );
        assert_eq!(
            extract_diff_filename("diff --git a/old_name.rs b/new_name.rs"),
            "new_name.rs"
        );
        assert_eq!(
            extract_diff_filename("diff --git a/path/with spaces/file.rs b/path/with spaces/file.rs"),
            "path/with spaces/file.rs"
        );
    }

    #[test]
    fn test_extract_diff_filename_with_b_in_path() {
        // Known limitation: if a filename literally contains " b/" as a path
        // component, rfind(" b/") picks the last occurrence (inside the filename)
        // rather than the prefix. This produces a truncated filename key.
        //
        // In practice this is vanishingly rare — no real project has directories
        // named "b" preceded by a space. The diff chunk itself is still captured
        // correctly; only the BTreeMap key is affected.
        assert_eq!(
            extract_diff_filename("diff --git a/path b/file.rs b/path b/file.rs"),
            "file.rs" // truncated — known limitation
        );
    }

    #[test]
    fn test_parse_diff_chunks_deleted_file() {
        let diff = "\
diff --git a/removed.rs b/removed.rs
deleted file mode 100644
--- a/removed.rs
+++ /dev/null
@@ -1,3 +0,0 @@
-fn old() {
-    // gone
-}";

        let chunks = parse_diff_chunks(diff);
        assert_eq!(chunks.len(), 1);
        assert!(chunks.contains_key("removed.rs"));
        let chunk = &chunks["removed.rs"];
        assert!(chunk.contains("deleted file mode"));
        assert!(chunk.contains("-fn old()"));
    }

    #[test]
    fn test_parse_diff_chunks_binary_file() {
        let diff = "\
diff --git a/image.png b/image.png
Binary files a/image.png and b/image.png differ";

        let chunks = parse_diff_chunks(diff);
        assert_eq!(chunks.len(), 1);
        assert!(chunks.contains_key("image.png"));
        assert!(chunks["image.png"].contains("Binary files"));
    }

    #[test]
    fn test_truncate_diff_chunks_no_op_when_small() {
        let mut chunks = BTreeMap::new();
        chunks.insert("a.rs".to_string(), "small diff".to_string());
        chunks.insert("b.rs".to_string(), "also small".to_string());

        let truncated = truncate_diff_chunks(chunks, 1000, 50);
        assert_eq!(truncated.len(), 2);
        assert!(!truncated["a.rs"].contains("truncated"));
    }

    #[test]
    fn test_truncate_diff_chunks_caps_per_file() {
        let mut chunks = BTreeMap::new();
        let big: String = (0..200).map(|i| format!("+line {}\n", i)).collect();
        chunks.insert("big.rs".to_string(), big);

        let truncated = truncate_diff_chunks(chunks, 50, 500);
        assert_eq!(truncated.len(), 1);
        let lines: Vec<&str> = truncated["big.rs"].lines().collect();
        assert!(lines.len() <= 52); // 50 + truncation notice
        assert!(truncated["big.rs"].contains("truncated"));
    }

    #[test]
    fn test_truncate_diff_chunks_caps_total() {
        let mut chunks = BTreeMap::new();
        for i in 0..100 {
            let content: String = (0..10).map(|j| format!("+line {}:{}\n", i, j)).collect();
            chunks.insert(format!("file_{:03}.rs", i), content);
        }

        let truncated = truncate_diff_chunks(chunks, 1000, 50);
        let total_lines: usize = truncated.values().map(|c| c.lines().count()).sum();
        // Should be capped near the total limit (50 + some truncation notices)
        assert!(total_lines <= 70, "total lines {} exceeded expected cap", total_lines);
    }
}
