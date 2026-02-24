//! Compiled fn bridge: DSL `fn` items with real Executable implementations.
//!
//! DSL pure functions (`fn` items) normally resolve to `PassthroughOp` which
//! cannot compute — it just forwards inputs to outputs. This module provides
//! compiled implementations for specific fn items that need actual computation.
//!
//! # Registry
//!
//! `lookup_compiled_fn(module, name)` returns `Some(DynOp)` for fn items with
//! compiled implementations, `None` for everything else (which falls through
//! to PassthroughOp in the resolver).
//!
//! # Supported modules
//!
//! - `std.markdown` — Markdown rendering functions
//! - `tools.gist_snapshot` — Gist snapshot content building
//! - `tools.gist` — Gist diff/recent content rendering

use std::collections::HashMap;

use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::Value;

// ============================================================================
// Registry
// ============================================================================

/// Look up a compiled fn implementation by module and name.
///
/// Returns `Some(DynOp)` if a compiled implementation exists, `None` otherwise.
pub fn lookup_compiled_fn(module: &str, name: &str) -> Option<DynOp> {
    match (module, name) {
        // std.markdown rendering fns
        ("std.markdown", "render_heading") => Some(DynOp::new(RenderHeadingOp)),
        ("std.markdown", "render_code_block") => Some(DynOp::new(RenderCodeBlockOp)),
        ("std.markdown", "render_bullet_list") => Some(DynOp::new(RenderBulletListOp)),
        ("std.markdown", "render_numbered_list") => Some(DynOp::new(RenderNumberedListOp)),
        ("std.markdown", "render_tree") => Some(DynOp::new(RenderTreeOp)),
        ("std.markdown", "render_node") => Some(DynOp::new(RenderNodeOp)),
        ("std.markdown", "render_markdown") => Some(DynOp::new(RenderMarkdownOp)),

        // tools.gist_snapshot compiled fns
        ("tools.gist_snapshot", "build_snapshot_content") => {
            Some(DynOp::new(BuildSnapshotContentOp))
        }

        // tools.gist compiled fns
        ("tools.gist", "render_diff_markdown") => Some(DynOp::new(RenderDiffMarkdownOp)),

        _ => None,
    }
}

// ============================================================================
// std.markdown compiled fns
// ============================================================================

/// `render_heading(level: Int, text: String) -> String`
///
/// Produces: `# text` (level number of `#` characters).
#[derive(Debug, Clone)]
struct RenderHeadingOp;

impl Executable for RenderHeadingOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let level = inputs
            .get("level")
            .and_then(Value::as_int)
            .unwrap_or(1)
            .clamp(1, 6) as usize;
        let text = inputs
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let prefix = "#".repeat(level);
        OutputMap::new().str("return", format!("{prefix} {text}")).ok()
    }
}

/// `render_code_block(code: String, language: String?) -> String`
///
/// Produces a fenced code block: `` ```lang\ncode\n``` ``
#[derive(Debug, Clone)]
struct RenderCodeBlockOp;

impl Executable for RenderCodeBlockOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let code = inputs
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let language = inputs
            .get("language")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let result = format!("```{language}\n{code}\n```");
        OutputMap::new().str("return", result).ok()
    }
}

/// `render_bullet_list(items: List<String>) -> String`
///
/// Produces: `- item1\n- item2\n...`
#[derive(Debug, Clone)]
struct RenderBulletListOp;

impl Executable for RenderBulletListOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let items = inputs
            .get("items")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        let result = items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n");
        OutputMap::new().str("return", result).ok()
    }
}

/// `render_numbered_list(items: List<String>) -> String`
///
/// Produces: `1. item1\n1. item2\n...`
#[derive(Debug, Clone)]
struct RenderNumberedListOp;

impl Executable for RenderNumberedListOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let items = inputs
            .get("items")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        let result = items
            .iter()
            .enumerate()
            .map(|(i, item)| format!("{}. {item}", i + 1))
            .collect::<Vec<_>>()
            .join("\n");
        OutputMap::new().str("return", result).ok()
    }
}

/// `render_tree(paths: List<String>) -> String`
///
/// Produces sorted paths joined by newlines.
#[derive(Debug, Clone)]
struct RenderTreeOp;

impl Executable for RenderTreeOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let mut paths = inputs
            .get("paths")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        paths.sort();
        let result = paths.join("\n");
        OutputMap::new().str("return", result).ok()
    }
}

/// `render_node(node: MarkdownNode) -> String`
///
/// Renders a single MarkdownNode (passed as JSON) to a markdown string.
#[derive(Debug, Clone)]
struct RenderNodeOp;

impl Executable for RenderNodeOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let node = inputs
            .get("node")
            .ok_or_else(|| ExecError::new("render_node: missing 'node' input"))?;
        let rendered = render_markdown_node(node);
        OutputMap::new().str("return", rendered).ok()
    }
}

/// `render_markdown(doc: MarkdownDoc) -> String`
///
/// Renders a full MarkdownDoc (passed as JSON `{ nodes: [...] }`) to markdown.
#[derive(Debug, Clone)]
struct RenderMarkdownOp;

impl Executable for RenderMarkdownOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let doc = inputs
            .get("doc")
            .ok_or_else(|| ExecError::new("render_markdown: missing 'doc' input"))?;

        let rendered = match doc {
            Value::Json(json) => render_markdown_doc_json(json),
            Value::Map(map) => {
                // Try to find a "nodes" key with a list of node values
                if let Some(nodes_value) = map.get("nodes") {
                    render_markdown_value_nodes(nodes_value)
                } else {
                    String::new()
                }
            }
            // If it's already a string, pass through
            Value::Str(s) => s.clone(),
            _ => String::new(),
        };

        OutputMap::new().str("return", rendered).ok()
    }
}

// ============================================================================
// tools.gist_snapshot compiled fns
// ============================================================================

/// `build_snapshot_content(branch: String, files: List<String>) -> String`
///
/// Renders a full markdown document for the workspace snapshot:
/// heading, branch info, and sorted directory tree.
#[derive(Debug, Clone)]
struct BuildSnapshotContentOp;

impl Executable for BuildSnapshotContentOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let branch = inputs
            .get("branch")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let files = inputs
            .get("files")
            .and_then(Value::as_str_list)
            .unwrap_or_default();

        let mut sorted_files = files;
        sorted_files.sort();
        let tree = sorted_files.join("\n");

        let content = format!(
            "# Workspace Snapshot\n\nBranch: {branch}\n\n## Directory Tree\n\n{tree}\n"
        );

        OutputMap::new().str("return", content).ok()
    }
}

// ============================================================================
// tools.gist compiled fns
// ============================================================================

/// `render_diff_markdown(diff: String, branch: String, base_ref: CommitSha) -> String`
///
/// Renders a diff as a markdown document with a heading and fenced code block.
#[derive(Debug, Clone)]
struct RenderDiffMarkdownOp;

impl Executable for RenderDiffMarkdownOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let diff = inputs
            .get("diff")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let branch = inputs
            .get("branch")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let base_ref = inputs
            .get("base_ref")
            .and_then(Value::as_str)
            .unwrap_or("HEAD~1");

        let result = format!("# Diff: {branch} vs {base_ref}\n\n```diff\n{diff}\n```\n");
        OutputMap::new().str("return", result).ok()
    }
}

// ============================================================================
// Internal rendering helpers
// ============================================================================

/// Render a single MarkdownNode from a Value.
fn render_markdown_node(node: &Value) -> String {
    match node {
        Value::Json(json) => render_markdown_node_json(json),
        Value::Map(map) => render_markdown_node_map(map),
        Value::Str(s) => s.clone(),
        _ => String::new(),
    }
}

/// Render a MarkdownNode from a serde_json::Value.
fn render_markdown_node_json(json: &serde_json::Value) -> String {
    let node_type = json
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    match node_type {
        "Heading" => {
            let level = json.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
            let text = json.get("text").and_then(|v| v.as_str()).unwrap_or_default();
            let prefix = "#".repeat(level.clamp(1, 6));
            format!("{prefix} {text}")
        }
        "CodeBlock" => {
            let language = json
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let code = json.get("code").and_then(|v| v.as_str()).unwrap_or_default();
            format!("```{language}\n{code}\n```")
        }
        "Paragraph" => json
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        "BulletList" => {
            let items = json
                .get("items")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("- {s}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            items
        }
        "NumberedList" => {
            let items = json
                .get("items")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .enumerate()
                        .map(|(i, s)| format!("{}. {s}", i + 1))
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            items
        }
        "Tree" => {
            let mut paths: Vec<String> = json
                .get("paths")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            paths.sort();
            paths.join("\n")
        }
        "ThematicBreak" => "---".to_string(),
        "BlockQuote" => {
            let text = json
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            format!("> {text}")
        }
        _ => String::new(),
    }
}

/// Render a MarkdownNode from a BTreeMap (Value::Map).
fn render_markdown_node_map(map: &std::collections::BTreeMap<String, Value>) -> String {
    let node_type = map
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    match node_type {
        "Heading" => {
            let level = map
                .get("level")
                .and_then(Value::as_int)
                .unwrap_or(1)
                .clamp(1, 6) as usize;
            let text = map.get("text").and_then(Value::as_str).unwrap_or_default();
            let prefix = "#".repeat(level);
            format!("{prefix} {text}")
        }
        "Paragraph" => map
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        "Tree" => {
            let mut paths = map
                .get("paths")
                .and_then(Value::as_str_list)
                .unwrap_or_default();
            paths.sort();
            paths.join("\n")
        }
        _ => String::new(),
    }
}

/// Render a full MarkdownDoc from JSON `{ "nodes": [...] }`.
fn render_markdown_doc_json(json: &serde_json::Value) -> String {
    let nodes = match json.get("nodes").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return String::new(),
    };

    nodes
        .iter()
        .map(render_markdown_node_json)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Render markdown from a Value that contains a list of nodes.
fn render_markdown_value_nodes(nodes_value: &Value) -> String {
    match nodes_value {
        Value::List(items) => items
            .iter()
            .map(render_markdown_node)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
        _ => String::new(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_heading_level_1() {
        let inputs = HashMap::from([
            ("level".to_string(), Value::Int(1)),
            ("text".to_string(), Value::Str("Hello".to_string())),
        ]);
        let out = RenderHeadingOp.execute(inputs).unwrap();
        assert_eq!(out["return"], Value::Str("# Hello".to_string()));
    }

    #[test]
    fn render_heading_level_3() {
        let inputs = HashMap::from([
            ("level".to_string(), Value::Int(3)),
            ("text".to_string(), Value::Str("Section".to_string())),
        ]);
        let out = RenderHeadingOp.execute(inputs).unwrap();
        assert_eq!(out["return"], Value::Str("### Section".to_string()));
    }

    #[test]
    fn render_code_block_with_language() {
        let inputs = HashMap::from([
            ("code".to_string(), Value::Str("fn main() {}".to_string())),
            ("language".to_string(), Value::Str("rust".to_string())),
        ]);
        let out = RenderCodeBlockOp.execute(inputs).unwrap();
        assert_eq!(
            out["return"],
            Value::Str("```rust\nfn main() {}\n```".to_string())
        );
    }

    #[test]
    fn render_code_block_no_language() {
        let inputs = HashMap::from([("code".to_string(), Value::Str("hello".to_string()))]);
        let out = RenderCodeBlockOp.execute(inputs).unwrap();
        assert_eq!(out["return"], Value::Str("```\nhello\n```".to_string()));
    }

    #[test]
    fn render_bullet_list_items() {
        let inputs = HashMap::from([(
            "items".to_string(),
            Value::str_list(vec!["foo".to_string(), "bar".to_string()]),
        )]);
        let out = RenderBulletListOp.execute(inputs).unwrap();
        assert_eq!(out["return"], Value::Str("- foo\n- bar".to_string()));
    }

    #[test]
    fn render_tree_sorted() {
        let inputs = HashMap::from([(
            "paths".to_string(),
            Value::str_list(vec![
                "src/main.rs".to_string(),
                "Cargo.toml".to_string(),
                "README.md".to_string(),
            ]),
        )]);
        let out = RenderTreeOp.execute(inputs).unwrap();
        assert_eq!(
            out["return"],
            Value::Str("Cargo.toml\nREADME.md\nsrc/main.rs".to_string())
        );
    }

    #[test]
    fn render_markdown_doc_from_json() {
        let doc = serde_json::json!({
            "nodes": [
                { "type": "Heading", "level": 1, "text": "Title" },
                { "type": "Paragraph", "text": "Some text." },
                { "type": "CodeBlock", "language": "rust", "code": "let x = 1;" },
            ]
        });
        let inputs = HashMap::from([("doc".to_string(), Value::Json(doc))]);
        let out = RenderMarkdownOp.execute(inputs).unwrap();
        let rendered = out["return"].as_str().unwrap();
        assert!(rendered.contains("# Title"));
        assert!(rendered.contains("Some text."));
        assert!(rendered.contains("```rust\nlet x = 1;\n```"));
    }

    #[test]
    fn build_snapshot_content_produces_markdown() {
        let inputs = HashMap::from([
            ("branch".to_string(), Value::Str("main".to_string())),
            (
                "files".to_string(),
                Value::str_list(vec![
                    "src/main.rs".to_string(),
                    "Cargo.toml".to_string(),
                ]),
            ),
        ]);
        let out = BuildSnapshotContentOp.execute(inputs).unwrap();
        let rendered = out["return"].as_str().unwrap();
        assert!(rendered.contains("# Workspace Snapshot"));
        assert!(rendered.contains("Branch: main"));
        assert!(rendered.contains("## Directory Tree"));
        // Files should be sorted
        assert!(rendered.contains("Cargo.toml\nsrc/main.rs"));
    }

    #[test]
    fn render_diff_markdown_produces_content() {
        let inputs = HashMap::from([
            (
                "diff".to_string(),
                Value::Str("+added line".to_string()),
            ),
            ("branch".to_string(), Value::Str("feature".to_string())),
            (
                "base_ref".to_string(),
                Value::Str("HEAD~1".to_string()),
            ),
        ]);
        let out = RenderDiffMarkdownOp.execute(inputs).unwrap();
        let rendered = out["return"].as_str().unwrap();
        assert!(rendered.contains("# Diff: feature vs HEAD~1"));
        assert!(rendered.contains("```diff\n+added line\n```"));
    }

    #[test]
    fn build_snapshot_content_sorts_files() {
        // Verify files are sorted in the output.
        let inputs = HashMap::from([
            ("branch".to_string(), Value::Str("feature".to_string())),
            (
                "files".to_string(),
                Value::str_list(vec![
                    "src/lib.rs".to_string(),
                    "Cargo.toml".to_string(),
                    "README.md".to_string(),
                ]),
            ),
        ]);
        let out = BuildSnapshotContentOp.execute(inputs).unwrap();
        let rendered = out["return"].as_str().unwrap();

        assert!(rendered.contains("# Workspace Snapshot"));
        assert!(rendered.contains("Branch: feature"));
        // Files sorted: Cargo.toml < README.md < src/lib.rs
        let tree_start = rendered.find("## Directory Tree").unwrap();
        let tree_section = &rendered[tree_start..];
        assert!(tree_section.contains("Cargo.toml\nREADME.md\nsrc/lib.rs"));
    }
}
