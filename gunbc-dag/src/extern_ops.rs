//! Explicit extern operation implementations for DSL `extern func` declarations.

use std::collections::{BTreeMap, HashMap};

use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::Value;

/// Resolve an extern symbol to a concrete runtime operation.
///
/// All app-specific DSL symbols are registered here. This is the single
/// dispatch table for `extern func` declarations and domain-specific
/// callable implementations (e.g., `tools.infra::infra`).
pub fn resolve_extern_symbol(module: &str, name: &str) -> Option<DynOp> {
    match (module, name) {
        ("std.markdown", "render_tree") => Some(DynOp::new(RenderTreeOp)),
        ("tools.gist", "build_snapshot_content") => Some(DynOp::new(BuildSnapshotContentOp)),
        ("tools.makegen", "discover_tools") => Some(DynOp::new(DiscoverToolsOp)),
        ("tools.bootstrap", "render_bootstrap_makefile") => {
            Some(DynOp::new(GenerateBootstrapMakefileOp))
        }
        ("tools.bootstrap", "render_bootstrap_gitignore") => {
            Some(DynOp::new(GenerateBootstrapGitignoreOp))
        }
        ("tools.cigen", "discover_ci_config") => Some(DynOp::new(DiscoverCiConfigOp)),
        ("tools.infra", "infra") => Some(DynOp::new(InfraDispatchOp)),
        _ => None,
    }
}

// ============================================================================
// std.markdown extern impls
// ============================================================================

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

        let mut content =
            format!("# Workspace Snapshot\n\nBranch: `{branch}`\n\n## Directory Tree\n\n{tree}\n");

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
                content.push_str(&format!("\n### {path}\n\n```{lang}\n{file_content}\n```\n"));
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

#[derive(Debug, Clone)]
struct DiscoverToolsOp;

impl Executable for DiscoverToolsOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
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
                map.insert(
                    "short_name".to_string(),
                    Value::Str(tool.short_name.clone()),
                );
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

        OutputMap::new().value("return", Value::List(tools)).ok()
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
        use crate::makegen::{registry::ToolRegistry, shared::render_makefile};
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
// tools.cigen extern impls
// ============================================================================

#[derive(Debug, Clone)]
struct DiscoverCiConfigOp;

impl Executable for DiscoverCiConfigOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let secrets: Vec<Value> = crate::ci::ci_live_test_secrets()
            .into_iter()
            .map(|s| Value::Str(s.to_string()))
            .collect();

        let tool = gunbc_ir::CargoInvocation::composed("ci", "dag");
        let tool_command = tool.command();

        let bootstrap_script = concat!(
            "rm -rf target/codegen\n",
            "# Cargo validates all [[bin]] paths even with --bin filter.\n",
            "# Create minimal stubs so the manifest parses, then check only bootstrap binaries.\n",
            "for dir in $(grep 'path = \"../target/codegen/' gunbc-dag/Cargo.toml | ",
            "sed 's|.*\"../\\(.*\\)/main.rs\"|\\1|'); do\n",
            "  mkdir -p \"$dir\" && echo 'fn main() {}' > \"$dir/main.rs\"\n",
            "done\n",
            "cargo check -p gunbc-dag --bin gunbc-codegen --bin gunbc-ci",
        );

        let mut result = BTreeMap::new();
        result.insert("secrets".to_string(), Value::List(secrets));
        result.insert("tool_command".to_string(), Value::Str(tool_command));
        result.insert(
            "bootstrap_script".to_string(),
            Value::Str(bootstrap_script.to_string()),
        );

        Ok(result.into_iter().collect())
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
// tools.infra domain op
// ============================================================================

#[derive(Debug, Clone)]
struct InfraDispatchOp;

impl Executable for InfraDispatchOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let environment = inputs
            .get("environment")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExecError::new("missing required 'environment' input for tools.infra::infra")
            })?
            .to_string();
        let runtime = inputs
            .get("runtime")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExecError::new("missing required 'runtime' input for tools.infra::infra")
            })?
            .to_string();
        let spec_targets = inputs
            .get("spec_targets")
            .and_then(Value::as_str_list)
            .ok_or_else(|| {
                ExecError::new("missing required 'spec_targets' input for tools.infra::infra")
            })?;
        let target = inputs
            .get("target")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        let skip = inputs
            .get("skip")
            .and_then(Value::as_str_list)
            .unwrap_or_default();
        let execute = inputs
            .get("execute")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let targeted = if target.is_empty() {
            spec_targets.clone()
        } else {
            spec_targets
                .iter()
                .filter(|item| target.iter().any(|selected| selected == *item))
                .cloned()
                .collect::<Vec<_>>()
        };
        let planned_targets = targeted
            .into_iter()
            .filter(|item| !skip.iter().any(|excluded| excluded == item))
            .collect::<Vec<_>>();
        let target_count = planned_targets.len() as i64;
        let mode = if execute { "apply" } else { "plan" };
        let applied_count = if execute { target_count } else { 0 };
        let report = format!(
            "infra {mode} (env={environment}, runtime={runtime}): {target_count} target(s)"
        );

        OutputMap::new()
            .str("environment", environment)
            .str("runtime", runtime)
            .str("mode", mode)
            .str_list("planned_targets", planned_targets)
            .int("target_count", target_count)
            .int("applied_count", applied_count)
            .str("report", report)
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_tree_extern_produces_tree_characters() {
        let inputs = HashMap::from([(
            "paths".to_string(),
            Value::str_list(vec!["src/main.rs".to_string(), "Cargo.toml".to_string()]),
        )]);
        let out = RenderTreeOp.execute(inputs).unwrap();
        let rendered = out["return"].as_str().unwrap();
        assert!(rendered.starts_with("```\n."));
        assert!(rendered.ends_with("```"));
        assert!(rendered.contains("Cargo.toml"));
    }

    #[test]
    fn discover_tools_returns_non_empty_list() {
        let out = DiscoverToolsOp.execute(HashMap::new()).unwrap();
        let tools = out.get("return").expect("return key");
        match tools {
            Value::List(items) => assert!(!items.is_empty(), "tool list should not be empty"),
            _ => panic!("discover_tools should return a list"),
        }
    }
}
