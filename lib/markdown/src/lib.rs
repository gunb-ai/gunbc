//! Markdown operations.
//!
//! Operations for generating and working with markdown content.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ops::markdown::render_code_snapshot;
//! use std::collections::BTreeMap;
//!
//! let mut contents = BTreeMap::new();
//! contents.insert("src/lib.rs".to_string(), "fn main() {}".to_string());
//!
//! let markdown = render_code_snapshot(&contents);
//! println!("{}", markdown);
//! ```

#![deny(dead_code)]
use gunbc_exec::{
    optional_str, propagate_skipped, require_map_str_str, ExecError, Executable, OutputMap,
};
use gunbc_ir::language::markdown_language_id;
use gunbc_ir::Value;
use std::collections::{BTreeMap, HashMap};

/// Markdown operations for use in DAG nodes.
#[derive(Debug, Clone)]
pub enum MarkdownOp {
    /// Render files as a markdown code snapshot
    RenderCodeSnapshot,
    /// Render per-file diffs as a markdown diff snapshot
    RenderDiffSnapshot,
}

impl Executable for MarkdownOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            MarkdownOp::RenderCodeSnapshot => {
                if let Some(result) = propagate_skipped(&inputs, "contents", &["markdown"]) {
                    return result;
                }
                let contents = require_map_str_str(&inputs, "contents")?;

                let markdown = render_code_snapshot(&contents);

                OutputMap::new().str("markdown", markdown).ok()
            }
            MarkdownOp::RenderDiffSnapshot => {
                if let Some(result) = propagate_skipped(&inputs, "diff_files", &["markdown"]) {
                    return result;
                }
                let diff_files = require_map_str_str(&inputs, "diff_files")?;

                let stats = optional_str(&inputs, "stats").unwrap_or("");

                let markdown = render_diff_snapshot(&diff_files, stats);

                OutputMap::new().str("markdown", markdown).ok()
            }
        }
    }
}

// ============================================================================
// Standalone helper functions
// ============================================================================

/// Render file contents as a markdown code snapshot.
///
/// Creates a markdown document with each file in a fenced code block,
/// with language detection based on file extension.
///
/// # Example
///
/// ```ignore
/// let mut contents = BTreeMap::new();
/// contents.insert("lib.rs".to_string(), "fn main() {}".to_string());
///
/// let md = render_code_snapshot(&contents);
/// assert!(md.contains("```rust"));
/// ```
pub fn render_code_snapshot(contents: &BTreeMap<String, String>) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Code Snapshot\n\n");

    for (filename, content) in contents {
        // Use centralized language detection from the Languages DAG
        let lang = markdown_language_id(filename);

        markdown.push_str(&format!("## `{}`\n\n", filename));
        markdown.push_str(&format!("```{}\n", lang));
        markdown.push_str(content);
        if !content.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push_str("```\n\n");
    }

    markdown
}

/// Render per-file diffs as a markdown diff snapshot.
///
/// Creates a markdown document with each file's diff in a fenced code block
/// using the `diff` language identifier for syntax highlighting. Includes
/// an optional stats summary line.
///
/// # Example
///
/// ```ignore
/// let mut diffs = BTreeMap::new();
/// diffs.insert("src/main.rs".to_string(), "@@ -1 +1,2 @@\n fn main() {}\n+// new".to_string());
///
/// let md = render_diff_snapshot(&diffs, "+1 -0 across 1 files");
/// assert!(md.contains("```diff"));
/// ```
pub fn render_diff_snapshot(diff_files: &BTreeMap<String, String>, stats: &str) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Branch Diff\n\n");

    if !stats.is_empty() {
        markdown.push_str(&format!("> {}\n\n", stats));
    }

    if diff_files.is_empty() {
        markdown.push_str("No changes between base and HEAD.\n");
        return markdown;
    }

    for (filename, diff_chunk) in diff_files {
        markdown.push_str(&format!("## `{}`\n\n", filename));
        markdown.push_str("```diff\n");
        markdown.push_str(diff_chunk);
        if !diff_chunk.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push_str("```\n\n");
    }

    markdown
}

/// Detect programming language from file extension.
///
/// Returns the language identifier for use in markdown fenced code blocks.
///
/// **Note**: This function now delegates to `gunbc_ir::language::markdown_language_id`
/// which is the single source of truth for language detection in the codebase.
pub fn detect_language(filename: &str) -> &'static str {
    markdown_language_id(filename)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("foo.rs"), "rust");
        assert_eq!(detect_language("bar.py"), "python");
        assert_eq!(detect_language("baz.ts"), "typescript");
        assert_eq!(detect_language("unknown.xyz"), "");
    }

    #[test]
    fn test_render_code_snapshot() {
        let mut contents = BTreeMap::new();
        contents.insert("test.rs".to_string(), "fn main() {}".to_string());

        let md = render_code_snapshot(&contents);

        assert!(md.contains("# Code Snapshot"));
        assert!(md.contains("## `test.rs`"));
        assert!(md.contains("```rust"));
        assert!(md.contains("fn main() {}"));
    }

    #[test]
    fn test_markdown_op() {
        let mut contents = BTreeMap::new();
        contents.insert("lib.rs".to_string(), "// code".to_string());

        let mut inputs = HashMap::new();
        inputs.insert("contents".to_string(), Value::str_map(contents));

        let op = MarkdownOp::RenderCodeSnapshot;
        let result = op.execute(inputs).unwrap();

        match result.get("markdown") {
            Some(Value::Str(md)) => {
                assert!(md.contains("lib.rs"));
            }
            _ => panic!("expected markdown"),
        }
    }

    #[test]
    fn test_render_diff_snapshot() {
        let mut diffs = BTreeMap::new();
        diffs.insert(
            "src/main.rs".to_string(),
            "@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }".to_string(),
        );

        let md = render_diff_snapshot(&diffs, "+1 -0 across 1 files");

        assert!(md.contains("# Branch Diff"));
        assert!(md.contains("> +1 -0 across 1 files"));
        assert!(md.contains("## `src/main.rs`"));
        assert!(md.contains("```diff"));
        assert!(md.contains("+    println!(\"hello\");"));
    }

    #[test]
    fn test_render_diff_snapshot_empty() {
        let diffs = BTreeMap::new();
        let md = render_diff_snapshot(&diffs, "");

        assert!(md.contains("# Branch Diff"));
        assert!(md.contains("No changes between base and HEAD."));
        assert!(!md.contains("```diff"));
    }

    #[test]
    fn test_render_diff_snapshot_no_stats() {
        let mut diffs = BTreeMap::new();
        diffs.insert("file.rs".to_string(), "+new line".to_string());

        let md = render_diff_snapshot(&diffs, "");

        assert!(md.contains("# Branch Diff"));
        assert!(!md.contains(">"));
        assert!(md.contains("```diff"));
    }

    #[test]
    fn test_render_diff_snapshot_multiple_files() {
        let mut diffs = BTreeMap::new();
        diffs.insert("a.rs".to_string(), "+line1".to_string());
        diffs.insert("b.rs".to_string(), "-line2".to_string());

        let md = render_diff_snapshot(&diffs, "+1 -1 across 2 files");

        assert!(md.contains("## `a.rs`"));
        assert!(md.contains("## `b.rs`"));
        // BTreeMap is sorted, so a.rs should come before b.rs
        let a_pos = md.find("## `a.rs`").unwrap();
        let b_pos = md.find("## `b.rs`").unwrap();
        assert!(a_pos < b_pos);
    }

    #[test]
    fn test_markdown_op_diff_snapshot() {
        let mut diffs = BTreeMap::new();
        diffs.insert("lib.rs".to_string(), "+// new code".to_string());

        let mut inputs = HashMap::new();
        inputs.insert("diff_files".to_string(), Value::str_map(diffs));
        inputs.insert(
            "stats".to_string(),
            Value::Str("+1 -0 across 1 files".to_string()),
        );

        let op = MarkdownOp::RenderDiffSnapshot;
        let result = op.execute(inputs).unwrap();

        match result.get("markdown") {
            Some(Value::Str(md)) => {
                assert!(md.contains("# Branch Diff"));
                assert!(md.contains("lib.rs"));
            }
            _ => panic!("expected markdown"),
        }
    }
}
