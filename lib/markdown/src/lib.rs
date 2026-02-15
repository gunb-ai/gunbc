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
    optional_str_strict, propagate_skipped, require_map_str_str, ExecError, Executable, OutputMap,
};
use gunbc_ir::language::markdown_language_id;
use gunbc_ir::Value;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

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

                let stats = optional_str_strict(&inputs, "stats")?.unwrap_or("");

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

        write!(markdown, "## `{}`\n\n", filename).unwrap();
        writeln!(markdown, "```{}", lang).unwrap();
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
        write!(markdown, "> {}\n\n", stats).unwrap();
    }

    if diff_files.is_empty() {
        markdown.push_str("No changes between base and HEAD.\n");
        return markdown;
    }

    for (filename, diff_chunk) in diff_files {
        write!(markdown, "## `{}`\n\n", filename).unwrap();
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
// DAG visualization rendering
// ============================================================================

/// Render a DAG diff as a markdown document with embedded Mermaid diagrams.
///
/// The document has two tiers:
/// 1. **Workspace Overview**: Top-level nodes colored by diff status.
/// 2. **Per-Changed-Tool Detail**: Expanded diagrams for each changed tool.
///
/// Unchanged tools are listed at the bottom without diagrams.
pub fn render_dag_diff_snapshot(
    title: &str,
    new_topo: &gunbc_ir::DagTopology,
    diff: &gunbc_ir::DagDiffResult,
    old_topo: &gunbc_ir::DagTopology,
) -> String {
    use gunbc_ir::{dag_mermaid, dag_topology::NodeTopology};

    let mut md = String::new();

    // Header
    write!(md, "# {}\n\n", title).unwrap();

    // Stats summary
    let changed_tool_count = diff.added_nodes.len() + diff.removed_nodes.len() + diff.changed_nodes.len();
    let total_tools = new_topo.node_count()
        + diff
            .removed_nodes
            .iter()
            .filter(|id| old_topo.get_node(id).is_some())
            .count();
    write!(
        md,
        "> {} across {} of {} workflows\n\n",
        diff.stats_summary(),
        changed_tool_count,
        total_tools,
    )
    .unwrap();

    // Tier 1: Workspace Overview
    md.push_str("## Workspace Overview\n\n");

    let removed_nodes: Vec<&NodeTopology> = old_topo
        .nodes
        .iter()
        .filter(|n| diff.removed_nodes.contains(&n.id))
        .collect();

    let overview = dag_mermaid::to_mermaid_overview_diff(new_topo, diff, &removed_nodes);
    md.push_str("```mermaid\n");
    md.push_str(&overview);
    md.push_str("```\n\n");

    md.push_str("---\n\n");

    // Tier 2: Per-changed-tool detail
    // Added tools
    for id in &diff.added_nodes {
        if let Some(node) = new_topo.get_node(id) {
            write!(md, "## `{}` (new)\n\n", id.0).unwrap();

            if let Some(ref children) = node.children {
                let snapshot = dag_mermaid::to_mermaid_snapshot(&id.0, children);
                md.push_str("```mermaid\n");
                md.push_str(&snapshot);
                md.push_str("```\n\n");
            }

            md.push_str("---\n\n");
        }
    }

    // Changed tools
    for change in &diff.changed_nodes {
        write!(md, "## `{}` (changed)\n\n", change.id.0).unwrap();

        // Changelog
        md.push_str("### Changes\n\n");
        if let Some(ref child_diff) = change.child_diff {
            md.push_str(&dag_mermaid::render_changelog(child_diff));
        } else {
            // Port-level changes only (no SubDag internals)
            let single_node_diff = gunbc_ir::DagDiffResult {
                changed_nodes: vec![change.clone()],
                ..Default::default()
            };
            md.push_str(&dag_mermaid::render_changelog(&single_node_diff));
        }
        md.push_str("\n\n");

        // Expanded diagram
        if let Some(node) = new_topo.get_node(&change.id) {
            if let Some(ref children) = node.children {
                if let Some(ref child_diff) = change.child_diff {
                    let expanded = dag_mermaid::to_mermaid_expanded_diff(
                        &change.id.0,
                        children,
                        child_diff,
                        &[],
                    );
                    md.push_str("```mermaid\n");
                    md.push_str(&expanded);
                    md.push_str("```\n\n");
                }
            }
        }

        md.push_str("---\n\n");
    }

    // Removed tools
    for id in &diff.removed_nodes {
        write!(md, "## `{}` (removed)\n\n", id.0).unwrap();
        md.push_str("This workflow was removed.\n\n");
        md.push_str("---\n\n");
    }

    // Unchanged tools
    if !diff.unchanged_nodes.is_empty() {
        md.push_str("## Unchanged workflows\n\n");
        let names: Vec<&str> = diff.unchanged_nodes.iter().map(|id| id.0.as_str()).collect();
        md.push_str(&names.join(", "));
        md.push('\n');
    }

    md
}

/// Render a DAG snapshot (non-diff) as a markdown document with embedded Mermaid.
///
/// Shows all workspace tools as Mermaid diagrams. Used by `make dag-viz`.
/// Each diagram includes both a fenced mermaid block (for GitHub rendering)
/// and a mermaid.ink image link (for proper zoom/pan in browsers).
pub fn render_dag_snapshot(title: &str, topo: &gunbc_ir::DagTopology) -> String {
    let mut md = String::new();

    write!(md, "# {}\n\n", title).unwrap();
    write!(
        md,
        "> {} workflows, {} total nodes\n\n",
        topo.node_count(),
        topo.total_node_count(),
    )
    .unwrap();

    // Legend — matches SemanticColor palette used in dag_mermaid
    md.push_str("**Legend**: ");
    md.push_str("🔵 env/resource · ");
    md.push_str("🟠 execute/transport · ");
    md.push_str("⚪ logic · ");
    md.push_str("`-->` data flow · ");
    md.push_str("`-.->` resource flow\n\n");

    // Overview diagram
    md.push_str("## Overview\n\n");
    let overview = gunbc_ir::dag_mermaid::to_mermaid_snapshot("workspace", topo);
    embed_mermaid_with_image(&mut md, &overview, "overview");

    // Per-tool diagrams
    for node in &topo.nodes {
        write!(md, "## `{}`\n\n", node.id.0).unwrap();

        if let Some(ref children) = node.children {
            write!(md, "{} nodes\n\n", children.total_node_count()).unwrap();
            let diagram = gunbc_ir::dag_mermaid::to_mermaid_snapshot(&node.id.0, children);
            embed_mermaid_with_image(&mut md, &diagram, &node.id.0);
        } else {
            md.push_str("Opaque node (no internal structure).\n\n");
        }
    }

    md
}

/// Embed a Mermaid diagram with both fenced code block and an image link.
///
/// The fenced block renders inline on GitHub. The image link provides a
/// fallback with proper browser zoom/scroll/pan via mermaid.ink.
fn embed_mermaid_with_image(md: &mut String, mermaid_code: &str, label: &str) {
    // Fenced mermaid block (GitHub native rendering)
    md.push_str("```mermaid\n");
    md.push_str(mermaid_code);
    md.push_str("```\n\n");

    // mermaid.ink image link (proper zoom/pan) — only if diagram is small enough
    if let Some(url) = gunbc_ir::dag_mermaid::mermaid_ink_url(mermaid_code) {
        write!(
            md,
            "<details><summary>Open zoomable image ({})</summary>\n\n",
            label
        )
        .unwrap();
        write!(md, "![{}]({})\n\n", label, url).unwrap();
        md.push_str("</details>\n\n");
    }
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
