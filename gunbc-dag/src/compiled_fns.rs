//! Compiled fn bridge: DSL `fn` items with real Executable implementations.
//!
//! DSL pure functions (`fn` items) normally resolve to `DeclaredOutputCallableOp` which
//! cannot compute — it just forwards inputs to outputs. This module provides
//! compiled implementations for specific fn items that need actual computation.
//!
//! # Registry
//!
//! `lookup_compiled_fn(module, name)` returns `Some(DynOp)` for fn items with
//! compiled implementations, `None` for everything else (which falls through
//! to DeclaredOutputCallableOp in the resolver).
//!
//! # Supported modules
//!
//! - `std.markdown` — Markdown rendering functions
//! - `tools.gist` — Gist snapshot content building + diff/recent content rendering
//! - `tools.pragma` — Clippy config, allowlist, and lint policy rendering
//! - `tools.bootstrap` — Workspace scan, Makefile/gitignore generation
//! - `tools.makegen` — Registry loading, Makefile rendering, entrypoint check

use std::collections::{BTreeMap, HashMap};

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

        // tools.gist compiled fns
        ("tools.gist", "build_snapshot_content") => Some(DynOp::new(BuildSnapshotContentOp)),
        ("tools.gist", "render_diff_markdown") => Some(DynOp::new(RenderDiffMarkdownOp)),

        // tools.pragma compiled fns
        ("tools.pragma", "render_clippy_toml") => Some(DynOp::new(RenderClippyTomlOp)),
        ("tools.pragma", "render_disallowed_methods_allowlist") => {
            Some(DynOp::new(RenderAllowlistOp))
        }
        ("tools.pragma", "render_pragma_lint_policy") => Some(DynOp::new(RenderLintPolicyOp)),

        // tools.bootstrap compiled fns
        ("tools.bootstrap", "prepare_scan_workspace") => {
            Some(DynOp::new(PrepareScanWorkspaceOp))
        }
        ("tools.bootstrap", "parse_scan_result") => Some(DynOp::new(ParseScanResultOp)),
        ("tools.bootstrap", "render_bootstrap_makefile") => {
            Some(DynOp::new(GenerateBootstrapMakefileOp))
        }
        ("tools.bootstrap", "render_bootstrap_gitignore") => {
            Some(DynOp::new(GenerateBootstrapGitignoreOp))
        }

        // tools.makegen compiled fns
        ("tools.makegen", "load_registry") => Some(DynOp::new(LoadRegistryOp)),
        ("tools.makegen", "render_makefile") => Some(DynOp::new(RenderMakefileCompiledOp)),
        ("tools.makegen", "makegen") => Some(DynOp::new(MakegenEntrypointOp)),

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
/// Produces a tree-character directory tree inside a fenced code block.
#[derive(Debug, Clone)]
struct RenderTreeOp;

impl Executable for RenderTreeOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let paths = inputs
            .get("paths")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        let result = render_path_tree(&paths.iter().map(String::as_str).collect::<Vec<_>>());
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
// tools.gist compiled fns
// ============================================================================

/// `build_snapshot_content(branch: String, files: List<String>, file_contents: List<String>, skipped: List<String>) -> String`
///
/// Renders a full markdown document for the workspace snapshot:
/// heading, branch info, tree-character directory tree, fenced code
/// blocks for every tracked file, and a list of skipped entries
/// (directories, symlinks, binary files, etc.).
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
        let skipped = inputs
            .get("skipped")
            .and_then(Value::as_str_list)
            .unwrap_or_default();

        // file_contents can be List<String> or List<Map{content: String}>.
        let file_contents = extract_file_contents(&inputs);

        let mut sorted_files = files.clone();
        sorted_files.sort();

        let tree = render_path_tree(&sorted_files.iter().map(String::as_str).collect::<Vec<_>>());

        let mut content = format!(
            "# Workspace Snapshot\n\nBranch: `{branch}`\n\n## Directory Tree\n\n{tree}\n"
        );

        if !skipped.is_empty() {
            content.push_str("\n## Skipped Entries\n\n");
            for path in &skipped {
                content.push_str(&format!("- {path}\n"));
            }
        }

        if !file_contents.is_empty() {
            content.push_str("\n## File Contents\n");
            // Zip paths and contents (maintain original order).
            for (path, file_content) in files.iter().zip(file_contents.iter()) {
                let lang = lang_for_path(path);
                content.push_str(&format!(
                    "\n### {path}\n\n```{lang}\n{file_content}\n```\n"
                ));
            }
        }

        OutputMap::new().str("return", content).ok()
    }
}

/// Extract file content strings from inputs. Handles both `List<String>` and
/// `List<Map{content: String}>` (the latter from transport parse outputs).
fn extract_file_contents(inputs: &HashMap<String, Value>) -> Vec<String> {
    let Some(value) = inputs.get("file_contents") else {
        return Vec::new();
    };
    match value {
        Value::List(items) => items
            .iter()
            .map(|item| match item {
                Value::Str(s) => s.clone(),
                Value::Map(map) => map
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                _ => String::new(),
            })
            .collect(),
        _ => Vec::new(),
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
// tools.pragma compiled fns
// ============================================================================

/// `render_clippy_toml() -> { content: String, return: String }`
#[derive(Debug, Clone)]
struct RenderClippyTomlOp;

impl Executable for RenderClippyTomlOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        use crate::policy::pragma::clippy_renderer;
        let content = clippy_renderer().render();
        OutputMap::new()
            .str("content", content.clone())
            .str("return", content)
            .ok()
    }
}

/// `render_disallowed_methods_allowlist() -> { content: String, return: String }`
#[derive(Debug, Clone)]
struct RenderAllowlistOp;

impl Executable for RenderAllowlistOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        use crate::policy::pragma::render_disallowed_methods_allowlist;
        let content = render_disallowed_methods_allowlist();
        OutputMap::new()
            .str("content", content.clone())
            .str("return", content)
            .ok()
    }
}

/// `render_pragma_lint_policy() -> { content: String, return: String }`
#[derive(Debug, Clone)]
struct RenderLintPolicyOp;

impl Executable for RenderLintPolicyOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        use crate::policy::pragma::render_pragma_lint_policy;
        let content = render_pragma_lint_policy();
        OutputMap::new()
            .str("content", content.clone())
            .str("return", content)
            .ok()
    }
}

// ============================================================================
// tools.bootstrap compiled fns
// ============================================================================

/// `prepare_scan_workspace() -> { request: TransportRequest, skip: Bool }`
#[derive(Debug, Clone)]
struct PrepareScanWorkspaceOp;

impl Executable for PrepareScanWorkspaceOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        use gunbc_ir::transport::ShellRequest;
        let request = ShellRequest::new("find")
            .args(["crates", "-maxdepth", "1", "-mindepth", "1", "-type", "d"])
            .into_transport_request();
        OutputMap::new()
            .request("request", request)
            .bool("skip", false)
            .ok()
    }
}

/// `parse_scan_result(response: TransportResponse) -> { crate_count: Int, crate_names: List<String> }`
#[derive(Debug, Clone)]
struct ParseScanResultOp;

impl Executable for ParseScanResultOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        use gunbc_exec::{propagate_skipped, require_response};
        use gunbc_ir::transport::TransportResponse;

        if let Some(result) =
            propagate_skipped(&inputs, "response", &["crate_count", "crate_names"])
        {
            return result;
        }

        let response = require_response(&inputs, "response")?;
        let mut crate_names = Vec::new();

        if let TransportResponse::Shell(shell) = response {
            if shell.success() {
                for line in shell.stdout.lines() {
                    let line = line.trim();
                    if !line.is_empty() {
                        if let Some(name) = line.strip_prefix("crates/") {
                            if !name.is_empty() && !name.contains('/') {
                                crate_names.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        crate_names.sort();

        OutputMap::new()
            .int("crate_count", crate_names.len() as i64)
            .str_list("crate_names", crate_names)
            .ok()
    }
}

/// `render_bootstrap_makefile(crate_names: List<String>?) -> { makefile_content: String, return: String }`
#[derive(Debug, Clone)]
struct GenerateBootstrapMakefileOp;

impl Executable for GenerateBootstrapMakefileOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        use gunbc_exec::optional_str_list_strict;
        let _ = optional_str_list_strict(&inputs, "crate_names")?;
        use crate::makegen::{registry::ToolRegistry, render::render_makefile};
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);
        OutputMap::new()
            .str("makefile_content", makefile.clone())
            .str("return", makefile)
            .ok()
    }
}

/// `render_bootstrap_gitignore(crate_names: List<String>?) -> { gitignore_content: String, return: String }`
#[derive(Debug, Clone)]
struct GenerateBootstrapGitignoreOp;

impl Executable for GenerateBootstrapGitignoreOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        use gunbc_exec::optional_str_list_strict;
        let _ = optional_str_list_strict(&inputs, "crate_names")?;
        use crate::makegen::{gitignore::render_gitignore, registry::default_build_config};
        let config = default_build_config();
        let gitignore = render_gitignore(&config);
        OutputMap::new()
            .str("gitignore_content", gitignore.clone())
            .str("return", gitignore)
            .ok()
    }
}

// ============================================================================
// tools.makegen compiled fns
// ============================================================================

/// `load_registry() -> { tool_count: Int, tool_names: List<String>, registry: Json }`
#[derive(Debug, Clone)]
struct LoadRegistryOp;

impl Executable for LoadRegistryOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        use crate::makegen::registry::ToolRegistry;
        use gunbc_testgen_registry::iter_dag_specs;

        let registry = ToolRegistry::default_registry();
        let tool_names: Vec<String> = registry
            .tools
            .iter()
            .map(|t| t.short_name.clone())
            .collect();

        let testgen_targets: Vec<serde_json::Value> = iter_dag_specs()
            .map(|spec| {
                serde_json::json!({
                    "name": spec.name,
                    "origin_crate": spec.origin_crate,
                    "output_path": spec.meta.output_path,
                    "module_name": spec.meta.module_name,
                    "tool_name": spec.meta.tool_name,
                })
            })
            .collect();

        let registry_json = serde_json::json!({
            "tools": registry.tools.iter().map(|t| {
                serde_json::json!({
                    "binary_name": t.binary_name(),
                    "short_name": t.short_name,
                    "description": t.description,
                    "entrypoints": t.entrypoints.iter().map(|e| {
                        serde_json::json!({
                            "port_name": e.port_name,
                            "make_var": e.make_var,
                            "cli_flag": e.cli_flag,
                            "type_hint": e.type_hint,
                            "default": e.default,
                            "repeatable": e.repeatable,
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>(),
            "testgen_targets": testgen_targets,
            "testgen_target_count": testgen_targets.len(),
        });

        OutputMap::new()
            .int("tool_count", registry.tools.len() as i64)
            .str_list("tool_names", tool_names)
            .json("registry", registry_json)
            .ok()
    }
}

/// `render_makefile() -> { return: String }`
#[derive(Debug, Clone)]
struct RenderMakefileCompiledOp;

impl Executable for RenderMakefileCompiledOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        use crate::makegen::registry::ToolRegistry;
        use crate::makegen::render::render_makefile;
        let registry = ToolRegistry::default_registry();
        let content = render_makefile(&registry);
        OutputMap::new().str("return", content).ok()
    }
}

/// `makegen(__deps: List<TransportResponse>) -> { written: Bool }`
#[derive(Debug, Clone)]
struct MakegenEntrypointOp;

impl Executable for MakegenEntrypointOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        use gunbc_ir::transport::{FileOp, TransportResponse};
        let written = inputs
            .get("__deps")
            .and_then(Value::as_list)
            .map(|deps| {
                deps.iter().any(|value| {
                    matches!(
                        value,
                        Value::Response(TransportResponse::File(response))
                            if response.operation == FileOp::Write && response.success
                    )
                })
            })
            .unwrap_or(false);
        OutputMap::new().bool("written", written).ok()
    }
}

// ============================================================================
// Tree rendering + language hints
// ============================================================================

/// Render sorted file paths as a tree-character directory tree in a fenced code block.
///
/// Example output:
/// ```text
/// .
/// ├── Cargo.toml
/// ├── src
/// │   ├── lib.rs
/// │   └── main.rs
/// └── tests
///     └── test.rs
/// ```
fn render_path_tree(paths: &[&str]) -> String {
    let mut sorted: Vec<&str> = paths.to_vec();
    sorted.sort();
    let mut result = String::from("```\n.\n");
    let tree = build_tree(&sorted);
    render_tree_node(&tree, &mut result, "");
    result.push_str("```");
    result
}

/// Internal tree node for directory structure.
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    fn new() -> Self {
        Self {
            children: BTreeMap::new(),
        }
    }
}

fn build_tree(paths: &[&str]) -> TreeNode {
    let mut root = TreeNode::new();
    for path in paths {
        let parts: Vec<&str> = path.split('/').collect();
        let mut current = &mut root;
        for part in parts {
            current = current
                .children
                .entry(part.to_string())
                .or_insert_with(TreeNode::new);
        }
    }
    root
}

fn render_tree_node(node: &TreeNode, out: &mut String, prefix: &str) {
    let count = node.children.len();
    for (i, (name, child)) in node.children.iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        out.push_str(prefix);
        out.push_str(connector);
        out.push_str(name);
        out.push('\n');
        let child_prefix = if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };
        render_tree_node(child, out, &child_prefix);
    }
}

/// Map file extension to language hint for fenced code blocks.
fn lang_for_path(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "rust",
        "toml" => "toml",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "md" => "markdown",
        "sh" | "bash" => "bash",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "go" => "go",
        "java" => "java",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        "xml" => "xml",
        "dag" => "dag",
        "txt" => "text",
        "lock" => "text",
        "gitignore" => "gitignore",
        "Makefile" | "makefile" => "makefile",
        _ => {
            // Handle files like "Makefile" where the filename IS the type.
            let filename = path.rsplit('/').next().unwrap_or(path);
            match filename {
                "Makefile" | "makefile" => "makefile",
                "Dockerfile" => "dockerfile",
                _ => "",
            }
        }
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
    fn render_tree_produces_tree_characters() {
        let inputs = HashMap::from([(
            "paths".to_string(),
            Value::str_list(vec![
                "src/main.rs".to_string(),
                "Cargo.toml".to_string(),
            ]),
        )]);
        let out = RenderTreeOp.execute(inputs).unwrap();
        let rendered = out["return"].as_str().unwrap();
        assert!(rendered.starts_with("```\n."));
        assert!(rendered.ends_with("```"));
        assert!(rendered.contains("Cargo.toml"));
        assert!(rendered.contains("src"));
        assert!(rendered.contains("main.rs"));
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
            (
                "file_contents".to_string(),
                Value::str_list(vec![
                    "fn main() {}".to_string(),
                    "[package]\nname = \"test\"".to_string(),
                ]),
            ),
        ]);
        let out = BuildSnapshotContentOp.execute(inputs).unwrap();
        let rendered = out["return"].as_str().unwrap();
        assert!(rendered.contains("# Workspace Snapshot"));
        assert!(rendered.contains("Branch: `main`"));
        assert!(rendered.contains("## Directory Tree"));
        assert!(rendered.contains("## File Contents"));
        // File contents should appear as fenced code blocks
        assert!(rendered.contains("```rust\nfn main() {}\n```"));
        assert!(rendered.contains("```toml\n[package]\nname = \"test\"\n```"));
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
    fn build_snapshot_content_sorts_files_in_tree() {
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
        assert!(rendered.contains("Branch: `feature`"));
        let tree_start = rendered.find("## Directory Tree").unwrap();
        let tree_section = &rendered[tree_start..];
        // Tree contains sorted entries with tree characters
        assert!(tree_section.contains("Cargo.toml"));
        assert!(tree_section.contains("README.md"));
        assert!(tree_section.contains("lib.rs"));
    }

    #[test]
    fn render_path_tree_basic() {
        let tree = render_path_tree(&["src/main.rs", "Cargo.toml", "src/lib.rs"]);
        assert!(tree.starts_with("```\n."));
        assert!(tree.ends_with("```"));
        // Tree should contain all entries
        assert!(tree.contains("Cargo.toml"));
        assert!(tree.contains("main.rs"));
        assert!(tree.contains("lib.rs"));
        // Should have tree characters
        assert!(tree.contains("├──") || tree.contains("└──"));
    }

    #[test]
    fn render_path_tree_nested() {
        let tree = render_path_tree(&["a/b/c.rs", "a/d.rs", "e.rs"]);
        assert!(tree.contains("a"));
        assert!(tree.contains("c.rs"));
        assert!(tree.contains("d.rs"));
        assert!(tree.contains("e.rs"));
        // Nested items should have indent
        assert!(tree.contains("│"));
    }

    #[test]
    fn lang_for_path_common_extensions() {
        assert_eq!(lang_for_path("src/main.rs"), "rust");
        assert_eq!(lang_for_path("Cargo.toml"), "toml");
        assert_eq!(lang_for_path("package.json"), "json");
        assert_eq!(lang_for_path("style.css"), "css");
        assert_eq!(lang_for_path("script.py"), "python");
        assert_eq!(lang_for_path("README.md"), "markdown");
        assert_eq!(lang_for_path("config.yaml"), "yaml");
        assert_eq!(lang_for_path("query.sql"), "sql");
        assert_eq!(lang_for_path("rules.dag"), "dag");
    }

    #[test]
    fn build_snapshot_with_file_contents() {
        let inputs = HashMap::from([
            ("branch".to_string(), Value::Str("main".to_string())),
            (
                "files".to_string(),
                Value::str_list(vec!["lib.rs".to_string(), "Cargo.toml".to_string()]),
            ),
            (
                "file_contents".to_string(),
                Value::str_list(vec![
                    "pub fn hello() {}".to_string(),
                    "[package]\nname = \"test\"".to_string(),
                ]),
            ),
        ]);
        let out = BuildSnapshotContentOp.execute(inputs).unwrap();
        let rendered = out["return"].as_str().unwrap();
        assert!(rendered.contains("## File Contents"));
        assert!(rendered.contains("### lib.rs"));
        assert!(rendered.contains("```rust\npub fn hello() {}\n```"));
        assert!(rendered.contains("### Cargo.toml"));
        assert!(rendered.contains("```toml\n[package]\nname = \"test\"\n```"));
    }

    #[test]
    fn build_snapshot_without_file_contents() {
        let inputs = HashMap::from([
            ("branch".to_string(), Value::Str("main".to_string())),
            (
                "files".to_string(),
                Value::str_list(vec!["file.rs".to_string()]),
            ),
        ]);
        let out = BuildSnapshotContentOp.execute(inputs).unwrap();
        let rendered = out["return"].as_str().unwrap();
        assert!(rendered.contains("## Directory Tree"));
        // No File Contents section when file_contents is absent
        assert!(!rendered.contains("## File Contents"));
    }

    // ========================================================================
    // Pragma compiled fn tests (migrated from pragma/ops.rs)
    // ========================================================================

    #[test]
    fn test_render_clippy_toml() {
        let result = RenderClippyTomlOp.execute(HashMap::new()).unwrap();
        match result.get("content") {
            Some(Value::Str(content)) => {
                assert!(content.contains("disallowed-methods"));
            }
            _ => panic!("expected clippy content"),
        }
    }

    #[test]
    fn test_render_allowlist() {
        let result = RenderAllowlistOp.execute(HashMap::new()).unwrap();
        match result.get("content") {
            Some(Value::Str(content)) => {
                assert!(content.contains("Generated by gunbc-pragma"));
            }
            _ => panic!("expected allowlist content"),
        }
    }

    #[test]
    fn test_render_lint_policy() {
        let result = RenderLintPolicyOp.execute(HashMap::new()).unwrap();
        match result.get("content") {
            Some(Value::Str(content)) => {
                assert!(content.contains("Generated by gunbc-pragma"));
            }
            _ => panic!("expected lint policy content"),
        }
    }

    // ========================================================================
    // Bootstrap compiled fn tests (migrated from bootstrap/ops.rs)
    // ========================================================================

    #[test]
    fn test_generate_makefile() {
        let result = GenerateBootstrapMakefileOp
            .execute(HashMap::new())
            .unwrap();
        match result.get("makefile_content") {
            Some(Value::Str(content)) => {
                assert!(content.contains("Generated by gunbc-makegen"));
                assert!(content.contains("Naming convention"));
                assert!(content.contains("<target>-fix"));
                assert!(content.contains("build:"));
                assert!(content.contains("test:"));
                assert!(content.contains("deps:"));
                assert!(content.contains("test-fix:"));
                assert!(content.contains("clippy-fix:"));
            }
            _ => panic!("expected makefile content"),
        }
    }

    #[test]
    fn test_generate_gitignore() {
        let result = GenerateBootstrapGitignoreOp
            .execute(HashMap::new())
            .unwrap();
        match result.get("gitignore_content") {
            Some(Value::Str(content)) => {
                assert!(content.contains("Generated by gunbc-bootstrap"));
                assert!(content.contains("(from cargo)"));
                assert!(content.contains("(from editor)"));
                assert!(content.contains("/target/"));
                assert!(content.contains(".DS_Store"));
                assert!(content.contains(".env"));
            }
            _ => panic!("expected gitignore content"),
        }
    }

    // ========================================================================
    // Makegen compiled fn tests (migrated from makegen/ops.rs)
    // ========================================================================

    #[test]
    fn test_load_registry() {
        let result = LoadRegistryOp.execute(HashMap::new()).unwrap();
        match result.get("tool_count") {
            Some(Value::Int(n)) => assert!(*n >= 2),
            _ => panic!("expected tool count"),
        }
        match result.get("tool_names").and_then(|v| v.as_str_list()) {
            Some(names) => {
                assert!(names.contains(&"deps".to_string()));
                assert!(names.contains(&"makegen".to_string()));
            }
            _ => panic!("expected tool names"),
        }
    }

    #[test]
    fn test_render_makefile_compiled() {
        let result = RenderMakefileCompiledOp.execute(HashMap::new()).unwrap();
        match result.get("return") {
            Some(Value::Str(content)) => {
                assert!(content.contains("deps:"));
                assert!(content.contains("makegen:"));
            }
            _ => panic!("expected makefile content"),
        }
    }
}
