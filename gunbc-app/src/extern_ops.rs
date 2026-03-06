//! Explicit extern operation implementations for DSL `extern func` declarations.

use std::collections::{BTreeMap, HashMap};
use std::sync::{LazyLock, OnceLock};

use gunbc_exec::{DynOp, ExecError, Executable, OutputMap};
use gunbc_ir::Value;

/// App-specific runtime bindings for gunbc DAG compilation.
///
/// This is the actual app binding point between generic `gunbc_resolve`
/// infrastructure and repo-local extern implementations.
pub fn gunbc_runtime_bindings() -> &'static gunbc_resolve::RuntimeBindings {
    static RUNTIME_BINDINGS: OnceLock<gunbc_resolve::RuntimeBindings> = OnceLock::new();
    RUNTIME_BINDINGS.get_or_init(|| {
        let mut bindings = gunbc_resolve::RuntimeBindings::new();
        bindings.register("std.markdown", "render_tree", DynOp::new(RenderTreeOp));
        bindings.register(
            "tools.gist",
            "build_snapshot_content",
            DynOp::new(BuildSnapshotContentOp),
        );
        bindings.register(
            "tools.makegen",
            "discover_tools",
            DynOp::new(DiscoverToolsOp),
        );
        bindings.register(
            "tools.bootstrap",
            "render_bootstrap_makefile",
            DynOp::new(GenerateBootstrapMakefileOp),
        );
        bindings.register(
            "tools.bootstrap",
            "render_bootstrap_gitignore",
            DynOp::new(GenerateBootstrapGitignoreOp),
        );
        bindings.register(
            "tools.pragma",
            "render_clippy_toml_content",
            DynOp::new(RenderPragmaClippyTomlContentOp),
        );
        bindings.register(
            "tools.pragma",
            "render_disallowed_methods_allowlist_content",
            DynOp::new(RenderPragmaDisallowedMethodsAllowlistContentOp),
        );
        bindings.register(
            "tools.pragma",
            "render_pragma_lint_policy_content",
            DynOp::new(RenderPragmaLintPolicyContentOp),
        );
        bindings.register(
            "tools.cigen",
            "discover_ci_config",
            DynOp::new(DiscoverCiConfigOp),
        );
        bindings.register(
            "tools.testgen",
            "discover_testgen_modules",
            DynOp::new(DiscoverTestgenModulesOp),
        );
        bindings.register(
            "tools.testgen",
            "render_testgen_module",
            DynOp::new(RenderTestgenModuleOp),
        );
        bindings.register("tools.infra", "infra", DynOp::new(InfraDispatchOp));
        bindings.register(
            "tools.readme",
            "discover_readme_tools",
            DynOp::new(DiscoverReadmeToolsOp),
        );
        bindings
    })
}

/// Build the app-specific runtime bindings table.
///
/// All extern symbols are registered with their concrete DynOp implementations.
pub fn cloned_gunbc_runtime_bindings() -> gunbc_resolve::RuntimeBindings {
    gunbc_runtime_bindings().clone()
}

#[allow(non_upper_case_globals)]
pub static GunbcExternResolver: LazyLock<gunbc_resolve::RuntimeBindings> =
    LazyLock::new(cloned_gunbc_runtime_bindings);

/// Resolve an extern symbol to a concrete runtime operation.
///
/// All app-specific DSL symbols are registered here. This is the single
/// dispatch table for `extern func` declarations and domain-specific
/// callable implementations (e.g., `tools.infra::infra`).
pub fn resolve_extern_symbol(module: &str, name: &str) -> Option<DynOp> {
    gunbc_runtime_bindings().resolve(module, name)
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
        use crate::makegen::model::{
            load_build_targets_data, reserved_target_names, validate_target_namespace_with_data,
        };
        use crate::makegen::tools::{
            discover_makegen_tools, filter_reserved_tools, tools_to_value,
        };
        use gunbc_ir::cargo::Warnings;

        let tools = discover_makegen_tools()
            .map_err(|e| ExecError::new(format!("failed to discover tools: {e}")))?;

        // Match the Rust-side render_makefile() pattern: filter collisions,
        // then validate — so both paths produce identical output.
        let build_targets = load_build_targets_data()
            .map_err(|e| ExecError::new(format!("failed to load build targets: {e}")))?;
        let reserved = reserved_target_names(&build_targets);
        let filtered = filter_reserved_tools(&tools, &reserved);
        validate_target_namespace_with_data(&filtered, &build_targets)
            .map_err(|e| ExecError::new(format!("invalid make target namespace: {e}")))?;

        let tools = tools_to_value(&filtered, Warnings::Deny);
        OutputMap::new().value("return", tools).ok()
    }
}

// ============================================================================
// tools.readme extern impls
// ============================================================================

#[derive(Debug, Clone)]
struct DiscoverReadmeToolsOp;

impl Executable for DiscoverReadmeToolsOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        use gunbc_codegen::tool_discovery::discover_tool_defs_from_dsl;

        let defs = discover_tool_defs_from_dsl()
            .map_err(|e| ExecError::new(format!("failed to discover tools: {e}")))?;

        let tools: Vec<Value> = defs
            .iter()
            .map(|def| {
                let mut map = BTreeMap::new();
                map.insert(
                    "name".to_string(),
                    Value::Str(def.meta.tool_name.to_string()),
                );
                map.insert(
                    "description".to_string(),
                    Value::Str(def.meta.description.to_string()),
                );
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
        use crate::makegen::shared::render_makefile_from_dsl_discovery;
        let makefile = render_makefile_from_dsl_discovery()
            .map_err(|e| ExecError::new(format!("failed to render makefile: {e}")))?;
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
        let gitignore = render_gitignore(&config)
            .map_err(|e| ExecError::new(format!("failed to render gitignore: {e}")))?;
        OutputMap::new()
            .str("gitignore_content", gitignore.clone())
            .str("return", gitignore)
            .ok()
    }
}

// ============================================================================
// tools.pragma extern impls
// ============================================================================

#[derive(Debug, Clone)]
struct RenderPragmaClippyTomlContentOp;

impl Executable for RenderPragmaClippyTomlContentOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let content = crate::pragma_dsl_render::render_clippy_toml_via_dsl();
        OutputMap::new().str("return", content).ok()
    }
}

#[derive(Debug, Clone)]
struct RenderPragmaDisallowedMethodsAllowlistContentOp;

impl Executable for RenderPragmaDisallowedMethodsAllowlistContentOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let content = crate::pragma_dsl_render::render_allowlist_via_dsl();
        OutputMap::new().str("return", content).ok()
    }
}

#[derive(Debug, Clone)]
struct RenderPragmaLintPolicyContentOp;

impl Executable for RenderPragmaLintPolicyContentOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let content = crate::pragma_dsl_render::render_lint_policy_via_dsl();
        OutputMap::new().str("return", content).ok()
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
            "for dir in $(grep 'path = \"../target/codegen/' gunbc-app/Cargo.toml | ",
            "sed 's|.*\"../\\(.*\\)/main.rs\"|\\1|'); do\n",
            "  mkdir -p \"$dir\" && echo 'fn main() {}' > \"$dir/main.rs\"\n",
            "done\n",
            "cargo check -p gunbc-app --bin gunbc-codegen --bin gunbc-ci",
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
// tools.testgen extern impls
// ============================================================================

#[derive(Debug, Clone)]
struct DiscoverTestgenModulesOp;

impl Executable for DiscoverTestgenModulesOp {
    fn execute(
        &self,
        _inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, ExecError> {
        let (dsl_root, _) = testgen_workspace_paths()?;
        let modules = crate::testgen_dag::discover_compilable_modules(&dsl_root)
            .into_iter()
            .map(|module| Value::Str(module.dsl_path))
            .collect::<Vec<_>>();
        OutputMap::new().value("return", Value::List(modules)).ok()
    }
}

#[derive(Debug, Clone)]
struct RenderTestgenModuleOp;

impl Executable for RenderTestgenModuleOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        let dsl_path = inputs
            .get("dsl_path")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ExecError::new(
                    "missing required 'dsl_path' input for tools.testgen::render_testgen_module",
                )
            })?;
        let (dsl_root, output_dir) = testgen_workspace_paths()?;
        let module =
            crate::testgen_dag::find_compilable_module(&dsl_root, dsl_path).ok_or_else(|| {
                ExecError::new(format!("testgen module is not compilable: {dsl_path}"))
            })?;
        let rendered = crate::testgen_dag::render_auto_testgen_for_module(&module, &output_dir);

        OutputMap::new()
            .str("content", rendered.content)
            .str("path", rendered.path)
            .ok()
    }
}

fn testgen_workspace_paths() -> Result<(std::path::PathBuf, std::path::PathBuf), ExecError> {
    let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
        .map_err(|e| ExecError::new(format!("workspace layout for testgen externs: {e}")))?;
    Ok((
        layout.workspace_root.join("dsl"),
        layout.workspace_root.join("gunbc-dag").join("src"),
    ))
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
        let out = DiscoverToolsOp
            .execute(HashMap::new())
            .expect("discover_tools should succeed after filtering reserved targets");
        let tools = out.get("return").expect("return key");
        match tools {
            Value::List(items) => assert!(!items.is_empty(), "tool list should not be empty"),
            _ => panic!("discover_tools should return a list"),
        }
    }

    #[test]
    fn discover_tools_matches_dsl_tool_projection() {
        use crate::makegen::model::{load_build_targets_data, reserved_target_names};
        use crate::makegen::tools::{
            discover_makegen_tools, filter_reserved_tools, tools_to_value,
        };
        use gunbc_ir::cargo::Warnings;

        let tools = discover_makegen_tools().expect("tool discovery should succeed");
        let build_targets = load_build_targets_data().expect("build targets should load");
        let reserved = reserved_target_names(&build_targets);
        let filtered = filter_reserved_tools(&tools, &reserved);
        let expected = tools_to_value(&filtered, Warnings::Deny);

        let out = DiscoverToolsOp
            .execute(HashMap::new())
            .expect("discover_tools should succeed");
        let actual = out.get("return").expect("return key").clone();
        assert_eq!(
            actual, expected,
            "DiscoverToolsOp output should match the DSL-discovered tool projection"
        );
    }

    #[test]
    fn discover_testgen_modules_includes_testgen_tool() {
        let out = DiscoverTestgenModulesOp
            .execute(HashMap::new())
            .expect("discover_testgen_modules should succeed");
        let modules = out
            .get("return")
            .and_then(Value::as_str_list)
            .expect("discover_testgen_modules should return List<String>");
        assert!(
            modules.iter().any(|module| module == "tools/testgen.dag"),
            "tools/testgen.dag should be part of the discovered testgen set",
        );
    }

    #[test]
    fn render_testgen_module_renders_generated_test_content() {
        let out = RenderTestgenModuleOp
            .execute(HashMap::from([(
                "dsl_path".to_string(),
                Value::Str("tools/makegen.dag".to_string()),
            )]))
            .expect("render_testgen_module should succeed");
        let path = out
            .get("path")
            .and_then(Value::as_str)
            .expect("path output");
        let content = out
            .get("content")
            .and_then(Value::as_str)
            .expect("content output");
        assert!(
            path.ends_with("generated_tests_tools_makegen.rs"),
            "unexpected generated path: {path}",
        );
        assert!(
            content.contains("#[test]"),
            "rendered testgen content should contain test functions",
        );
    }

    #[test]
    fn discover_readme_tools_returns_name_and_description() {
        let out = DiscoverReadmeToolsOp
            .execute(HashMap::new())
            .expect("discover_readme_tools should succeed");
        let tools = out.get("return").expect("return key");
        let Value::List(items) = tools else {
            panic!("discover_readme_tools should return a list");
        };
        assert!(!items.is_empty(), "tool list should not be empty");
        // Each tool should have name and description fields.
        let first = items[0].as_map().expect("tool should be a map");
        assert!(first.contains_key("name"), "tool should have 'name' field");
        assert!(
            first.contains_key("description"),
            "tool should have 'description' field"
        );
    }

    #[test]
    fn discover_readme_tools_includes_readme_itself() {
        let out = DiscoverReadmeToolsOp
            .execute(HashMap::new())
            .expect("discover_readme_tools should succeed");
        let tools = out
            .get("return")
            .and_then(|v| match v {
                Value::List(items) => Some(items.clone()),
                _ => None,
            })
            .expect("should return a list");
        let names: Vec<String> = tools
            .iter()
            .filter_map(|t| {
                t.as_map()
                    .and_then(|m| m.get("name"))
                    .and_then(Value::as_str)
                    .map(|s| s.to_string())
            })
            .collect();
        assert!(
            names.iter().any(|n| n == "readme"),
            "readme tool should be auto-discovered, found: {names:?}"
        );
    }

    #[test]
    fn render_readme_fn_produces_markdown() {
        use daglang_driver::compile_data_from_module;

        let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout");
        let dsl_root = layout.workspace_root.join("dsl");
        let output = compile_data_from_module(&dsl_root, "tools/readme.dag")
            .expect("readme.dag should compile");

        let body = output
            .fns
            .get("render_readme")
            .expect("render_readme fn body should exist");

        // Build mock tool list input.
        let mock_tools = Value::List(vec![{
            let mut m = BTreeMap::new();
            m.insert("name".to_string(), Value::Str("test-tool".to_string()));
            m.insert(
                "description".to_string(),
                Value::Str("A test tool".to_string()),
            );
            Value::Map(m)
        }]);

        let mut inputs = HashMap::new();
        inputs.insert("tools".to_string(), mock_tools);

        let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, &output.fns)
            .expect("render_readme should evaluate");
        let content = result
            .get("return")
            .and_then(Value::as_str)
            .expect("render_readme should return a string");

        assert!(content.starts_with("# gunbc"), "should start with title");
        assert!(
            content.contains("## Install"),
            "should have install section"
        );
        assert!(content.contains("## Tools"), "should have tools section");
        assert!(content.contains("test-tool"), "should include mock tool");
        assert!(
            content.contains("Generated by `gunbc-readme`"),
            "should have footer"
        );
    }
}
