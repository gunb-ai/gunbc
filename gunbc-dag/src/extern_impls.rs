//! Explicit extern implementations for DSL `extern func` declarations.
//!
//! Every entry here corresponds to an `extern func` in a `.dag` file. If a
//! DSL function has a body, it must NOT appear here — the DSL body is what
//! runs. This module provides Rust implementations for operations that
//! cannot be expressed in pure DSL (registry access, recursive algorithms,
//! complex content assembly).
//!
//! # Invariant
//!
//! `all_extern_symbols()` returns the complete set of `(module, name)` pairs.
//! A test snapshots this list to prevent silent additions.

use std::collections::{BTreeMap, HashMap};

use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::Value;

// ============================================================================
// Registry
// ============================================================================

/// All extern symbols backed by this module.
///
/// Used by tests to snapshot the full key set and prevent silent additions.
pub fn all_extern_symbols() -> &'static [(&'static str, &'static str)] {
    &[
        ("config.tool_discovery", "discover_tools"),
        ("std.markdown", "render_tree"),
        ("tools.bootstrap", "render_bootstrap_gitignore"),
        ("tools.bootstrap", "render_bootstrap_makefile"),
        ("tools.gist", "build_snapshot_content"),
    ]
}

/// Look up an extern implementation by module and name.
///
/// Returns `Some(DynOp)` if an extern implementation exists, `None` otherwise.
/// No fallback — unresolvable extern symbols are hard errors in the resolver.
pub fn lookup_extern_impl(module: &str, name: &str) -> Option<DynOp> {
    match (module, name) {
        ("std.markdown", "render_tree") => Some(DynOp::new(RenderTreeOp)),

        ("tools.gist", "build_snapshot_content") => Some(DynOp::new(BuildSnapshotContentOp)),

        ("config.tool_discovery", "discover_tools") => Some(DynOp::new(DiscoverToolsOp)),

        ("tools.bootstrap", "render_bootstrap_makefile") => {
            Some(DynOp::new(GenerateBootstrapMakefileOp))
        }
        ("tools.bootstrap", "render_bootstrap_gitignore") => {
            Some(DynOp::new(GenerateBootstrapGitignoreOp))
        }

        _ => None,
    }
}

// ============================================================================
// std.markdown extern impls
// ============================================================================

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

// ============================================================================
// tools.gist extern impls
// ============================================================================

/// `build_snapshot_content(branch, files, file_contents, skipped) -> String`
///
/// Renders a full markdown document for the workspace snapshot.
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

        let file_contents = extract_file_contents(&inputs)?;

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
fn extract_file_contents(inputs: &HashMap<String, Value>) -> Result<Vec<String>, ExecError> {
    let Some(value) = inputs.get("file_contents") else {
        return Ok(Vec::new());
    };
    match value {
        Value::List(items) => {
            let mut result = Vec::with_capacity(items.len());
            for (idx, item) in items.iter().enumerate() {
                match item {
                    Value::Str(s) => result.push(s.clone()),
                    Value::Map(map) => match map.get("content").and_then(|v| v.as_str()) {
                        Some(content) => result.push(content.to_string()),
                        None => {
                            return Err(ExecError::new(format!(
                                "extract_file_contents: item[{idx}]: Map missing 'content' key (keys: {:?})",
                                map.keys().collect::<Vec<_>>()
                            )));
                        }
                    },
                    other => {
                        return Err(ExecError::new(format!(
                            "extract_file_contents: item[{idx}]: unexpected value type {:?}",
                            std::mem::discriminant(other)
                        )));
                    }
                }
            }
            Ok(result)
        }
        other => Err(ExecError::new(format!(
            "extract_file_contents: expected List, got {:?}",
            std::mem::discriminant(other)
        ))),
    }
}

// ============================================================================
// tools.makegen extern impls
// ============================================================================

/// `discover_tools() -> List<DiscoveredTool>`
///
/// Returns tool information from the Rust ToolRegistry as structured data
/// that the DSL rendering functions can consume.
#[derive(Debug, Clone)]
struct DiscoverToolsOp;

impl Executable for DiscoverToolsOp {
    fn execute(&self, _inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        use crate::makegen::registry::{BuildConfig, ToolRegistry};
        use gunbc_ir::cargo::{CargoCommand, Subcommand};

        let registry = ToolRegistry::default_registry();
        let config = BuildConfig::cargo();

        let tools: Vec<Value> = registry
            .tools
            .iter()
            .map(|tool| {
                let cmd = CargoCommand::new(Subcommand::Run(tool.invocation.clone()))
                    .quiet()
                    .release()
                    .warnings(config.warnings);
                let command = format!("@{}", cmd.to_shell_with_env());
                let dry_run_command = format!("{} -- --dry-run strict", command);

                let deps: Vec<Value> = if tool.needs_generated_cli {
                    vec![Value::Str("ensure-codegen".to_string())]
                } else {
                    vec![]
                };

                let entrypoints: Vec<Value> = tool
                    .entrypoints
                    .iter()
                    .map(|ep| {
                        let mut map = BTreeMap::new();
                        map.insert("port_name".to_string(), Value::Str(ep.port_name.clone()));
                        map.insert("make_var".to_string(), Value::Str(ep.make_var.clone()));
                        map.insert("cli_flag".to_string(), Value::Str(ep.cli_flag.clone()));
                        map.insert("type_hint".to_string(), Value::Str(ep.type_hint.clone()));
                        map.insert(
                            "default".to_string(),
                            match &ep.default {
                                Some(d) => Value::Str(d.clone()),
                                None => Value::Unit,
                            },
                        );
                        map.insert("repeatable".to_string(), Value::Bool(ep.repeatable));
                        Value::Map(map)
                    })
                    .collect();

                let extra_targets: Vec<Value> = tool
                    .extra_targets
                    .iter()
                    .map(|extra| {
                        let mut map = BTreeMap::new();
                        map.insert("suffix".to_string(), Value::Str(extra.suffix.clone()));
                        map.insert(
                            "description".to_string(),
                            Value::Str(extra.description.clone()),
                        );
                        map.insert(
                            "post_commands".to_string(),
                            Value::List(
                                extra
                                    .post_commands
                                    .iter()
                                    .map(|c| Value::Str(c.clone()))
                                    .collect(),
                            ),
                        );
                        Value::Map(map)
                    })
                    .collect();

                let live_secrets: Vec<Value> = tool
                    .live_secrets
                    .iter()
                    .map(|s| Value::Str(s.clone()))
                    .collect();

                let mut map = BTreeMap::new();
                map.insert("short_name".to_string(), Value::Str(tool.short_name.clone()));
                map.insert(
                    "description".to_string(),
                    Value::Str(tool.description.clone()),
                );
                map.insert(
                    "binary_name".to_string(),
                    Value::Str(tool.binary_name().to_string()),
                );
                map.insert("command".to_string(), Value::Str(command));
                map.insert("dry_run_command".to_string(), Value::Str(dry_run_command));
                map.insert("deps".to_string(), Value::List(deps));
                map.insert("entrypoints".to_string(), Value::List(entrypoints));
                map.insert("extra_targets".to_string(), Value::List(extra_targets));
                map.insert("live_secrets".to_string(), Value::List(live_secrets));
                Value::Map(map)
            })
            .collect();

        OutputMap::new()
            .value("return", Value::List(tools))
            .ok()
    }
}

// ============================================================================
// tools.bootstrap extern impls
// ============================================================================

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
// Tree rendering + language hints (shared helpers)
// ============================================================================

fn render_path_tree(paths: &[&str]) -> String {
    let mut sorted: Vec<&str> = paths.to_vec();
    sorted.sort();
    let mut result = String::from("```\n.\n");
    let tree = build_tree(&sorted);
    render_tree_node(&tree, &mut result, "");
    result.push_str("```");
    result
}

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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_extern_symbols_matches_lookup() {
        for &(module, name) in all_extern_symbols() {
            assert!(
                lookup_extern_impl(module, name).is_some(),
                "all_extern_symbols lists ({module}, {name}) but lookup_extern_impl returns None"
            );
        }
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
    }

    #[test]
    fn render_path_tree_basic() {
        let tree = render_path_tree(&["src/main.rs", "Cargo.toml", "src/lib.rs"]);
        assert!(tree.starts_with("```\n."));
        assert!(tree.ends_with("```"));
        assert!(tree.contains("├──") || tree.contains("└──"));
    }

    #[test]
    fn lang_for_path_common_extensions() {
        assert_eq!(lang_for_path("src/main.rs"), "rust");
        assert_eq!(lang_for_path("Cargo.toml"), "toml");
        assert_eq!(lang_for_path("package.json"), "json");
        assert_eq!(lang_for_path("rules.dag"), "dag");
    }

    #[test]
    fn test_discover_tools_returns_list() {
        let result = DiscoverToolsOp.execute(HashMap::new()).unwrap();
        let tools = result.get("return").expect("expected return key");
        match tools {
            Value::List(items) => {
                assert!(items.len() >= 5, "should discover at least 5 tools, got {}", items.len());
                // Check first tool has expected fields
                if let Value::Map(tool) = &items[0] {
                    assert!(tool.contains_key("short_name"), "tool missing short_name");
                    assert!(tool.contains_key("command"), "tool missing command");
                    assert!(tool.contains_key("dry_run_command"), "tool missing dry_run_command");
                    assert!(tool.contains_key("deps"), "tool missing deps");
                    assert!(tool.contains_key("entrypoints"), "tool missing entrypoints");
                }
            }
            _ => panic!("expected List, got {:?}", std::mem::discriminant(tools)),
        }
    }

    #[test]
    fn test_generate_makefile() {
        let result = GenerateBootstrapMakefileOp.execute(HashMap::new()).unwrap();
        let content = result.get("makefile_content").and_then(Value::as_str).expect("expected makefile content");
        assert!(content.contains("Generated by gunbc-makegen"));
        assert!(content.contains("build:"));
    }

    #[test]
    fn test_generate_gitignore() {
        let result = GenerateBootstrapGitignoreOp.execute(HashMap::new()).unwrap();
        let content = result.get("gitignore_content").and_then(Value::as_str).expect("expected gitignore content");
        assert!(content.contains("Generated by gunbc-bootstrap"));
        assert!(content.contains("/target/"));
    }

}
