//! Justfile rendering.
//!
//! This renderer is a second workflow-format consumer for the shared
//! `WorkflowSpec`/registry model. It intentionally mirrors the Makefile target
//! graph (target names + dependencies) while emitting Just syntax.

use std::collections::BTreeMap;
use std::path::PathBuf;

use daglang_driver::{compile_from_context, DriverContext};
use serde::Deserialize;

use crate::makegen::registry::{BuildConfig, EntrypointParam, ToolInfo, ToolRegistry};
use gunbc_ir::CargoInvocation;
use gunbc_ir::cargo::{CargoCommand, Subcommand, Warnings};
use gunbc_ir::render_ir::FileHeader;
use super::shared::render_makefile;

/// Renderer for Justfiles with standardized header generation.
pub struct JustfileRenderer<'a> {
    pub registry: &'a ToolRegistry,
    pub config: BuildConfig,
}

impl<'a> JustfileRenderer<'a> {
    /// Create a new renderer with the default cargo build config.
    pub fn new(registry: &'a ToolRegistry) -> Self {
        Self {
            registry,
            config: BuildConfig::cargo(),
        }
    }

    /// Create a new renderer with a specific build config.
    pub fn with_config(registry: &'a ToolRegistry, config: BuildConfig) -> Self {
        Self { registry, config }
    }

    /// Render the complete Justfile with header.
    pub fn render(&self) -> String {
        let regenerate_cmd =
            CargoCommand::new(Subcommand::Run(CargoInvocation::composed("makegen", "dag")));
        let header = FileHeader {
            generator_name: "gunbc-makegen".into(),
            regenerate_command: format!("{} --format just", regenerate_cmd.to_shell()).into(),
            comment_prefix: "#".into(),
        };
        format!(
            "{}\n\n{}",
            header.render(),
            render_justfile_content(self.registry, &self.config)
        )
    }
}

/// Render a complete Justfile from the tool registry.
pub fn render_justfile(registry: &ToolRegistry) -> String {
    JustfileRenderer::new(registry).render()
}

/// Render a complete Justfile with a specific build config.
pub fn render_justfile_with_config(registry: &ToolRegistry, config: &BuildConfig) -> String {
    JustfileRenderer::with_config(registry, config.clone()).render()
}

#[derive(Debug, Clone)]
struct Recipe {
    name: String,
    deps: Vec<String>,
    body: Vec<String>,
    comment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CoreWorkflowData {
    name: String,
    description: String,
    deps: Vec<String>,
    body: Vec<String>,
    comment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ResourceNeedData {
    resource: String,
    mode: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MetaTargetData {
    name: String,
    description: String,
    has_check: bool,
    has_fix: bool,
    command: String,
    check_command: Option<String>,
    fix_command: Option<String>,
    command_prefix: Option<String>,
    resource_needs: Vec<ResourceNeedData>,
    fix_prerequisites: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ResourceTargetEntryData {
    resource: String,
    ensure_target: String,
    verify_target: String,
}

fn render_justfile_content(registry: &ToolRegistry, config: &BuildConfig) -> String {
    let mut out = String::new();
    out.push_str("# NOTE: This Justfile mirrors Makefile target topology.\n");
    out.push_str("set shell := [\"bash\", \"-cu\"]\n\n");

    let vars = collect_entrypoint_vars(registry);
    for (var, default) in vars {
        out.push_str(&format!("{var} := \"{}\"\n", escape_just_string(&default)));
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
    }

    for recipe in build_recipes(registry, config) {
        if let Some(comment) = recipe.comment {
            out.push_str(&format!("# {comment}\n"));
        }
        out.push_str(&recipe.name);
        out.push(':');
        if !recipe.deps.is_empty() {
            out.push(' ');
            out.push_str(&recipe.deps.join(" "));
        }
        out.push('\n');
        for line in recipe.body {
            out.push_str("    ");
            out.push_str(&line);
            out.push('\n');
        }
        out.push('\n');
    }

    out
}

fn build_recipes(registry: &ToolRegistry, config: &BuildConfig) -> Vec<Recipe> {
    let mut recipes = Vec::new();
    let (core_workflows, meta_targets, resource_targets) = load_build_targets_data();

    recipes.push(Recipe {
        name: "help".to_string(),
        deps: Vec::new(),
        body: vec![
            "@echo \"gunbc tools - generated Justfile\"".to_string(),
            "@echo \"Use 'just <target>' for workflow execution.\"".to_string(),
        ],
        comment: Some("Help target".to_string()),
    });

    for workflow in core_workflows {
        let body = workflow
            .body
            .into_iter()
            .map(|line| normalize_make_command_for_just(&line))
            .collect();
        recipes.push(Recipe {
            name: workflow.name,
            deps: workflow.deps,
            body,
            comment: Some(workflow.comment.unwrap_or(workflow.description)),
        });
    }

    for meta in &meta_targets {
        recipes.push(build_meta_recipe(meta, &resource_targets));
        if meta.has_check {
            recipes.push(build_meta_check_recipe(meta, &resource_targets));
        }
        if meta.has_fix {
            recipes.push(build_meta_fix_recipe(meta, &resource_targets));
        }
    }

    for tool in &registry.tools {
        recipes.push(build_tool_recipe(tool, config, false));
        recipes.push(build_tool_recipe(tool, config, true));
        for extra in &tool.extra_targets {
            recipes.push(Recipe {
                name: format!("{}-{}", tool.short_name, extra.suffix),
                deps: vec![tool.short_name.clone()],
                body: extra.post_commands.clone(),
                comment: Some(format!(
                    "{}-{}: {}",
                    tool.short_name, extra.suffix, extra.description
                )),
            });
        }
    }

    recipes
}

fn resolve_resource_target(
    resource: &str,
    mode: &str,
    res_targets: &[ResourceTargetEntryData],
) -> Option<String> {
    let entry = res_targets.iter().find(|entry| entry.resource == resource)?;
    Some(match mode {
        "ensure" => entry.ensure_target.clone(),
        _ => entry.verify_target.clone(),
    })
}

fn resolve_meta_base_deps(
    meta: &MetaTargetData,
    res_targets: &[ResourceTargetEntryData],
) -> Vec<String> {
    meta.resource_needs
        .iter()
        .filter_map(|need| resolve_resource_target(&need.resource, &need.mode, res_targets))
        .collect()
}

fn resolve_meta_fix_deps(
    meta: &MetaTargetData,
    res_targets: &[ResourceTargetEntryData],
) -> Vec<String> {
    meta.resource_needs
        .iter()
        .filter_map(|need| resolve_resource_target(&need.resource, "ensure", res_targets))
        .collect()
}

fn apply_prefix(prefix: &Option<String>, command: &str) -> String {
    match prefix {
        Some(p) if !p.trim().is_empty() => {
            let base = command.strip_prefix('@').unwrap_or(command);
            format!("@{} {}", p.trim(), base)
        }
        _ => command.to_string(),
    }
}

fn build_meta_recipe(meta: &MetaTargetData, res_targets: &[ResourceTargetEntryData]) -> Recipe {
    let deps = resolve_meta_base_deps(meta, res_targets);
    Recipe {
        name: meta.name.clone(),
        deps,
        body: vec![apply_prefix(&meta.command_prefix, &meta.command)],
        comment: Some(format!("{}: {}", meta.name, meta.description)),
    }
}

fn build_meta_check_recipe(
    meta: &MetaTargetData,
    res_targets: &[ResourceTargetEntryData],
) -> Recipe {
    let deps = resolve_meta_base_deps(meta, res_targets);
    let command = meta
        .check_command
        .clone()
        .unwrap_or_else(|| meta.command.clone());
    Recipe {
        name: format!("{}-check", meta.name),
        deps,
        body: vec![command],
        comment: Some(format!("{}-check: {}", meta.name, meta.description)),
    }
}

fn build_meta_fix_recipe(
    meta: &MetaTargetData,
    res_targets: &[ResourceTargetEntryData],
) -> Recipe {
    let mut deps = Vec::new();
    deps.extend(meta.fix_prerequisites.iter().cloned());
    deps.extend(resolve_meta_fix_deps(meta, res_targets));
    let command = meta.fix_command.clone().unwrap_or_else(|| meta.command.clone());
    let command = apply_prefix(&meta.command_prefix, &command);
    Recipe {
        name: format!("{}-fix", meta.name),
        deps,
        body: vec![command],
        comment: Some(format!("{}-fix: auto-fix then verify", meta.name)),
    }
}

fn build_tool_recipe(tool: &ToolInfo, config: &BuildConfig, dry_run: bool) -> Recipe {
    let _ = config;
    let deps = tool_target_deps(tool);
    let name = if dry_run {
        format!("{}-dry", tool.short_name)
    } else {
        tool.short_name.clone()
    };
    let port_list = tool
        .entrypoints
        .iter()
        .map(|p| format!("{} ({})", p.port_name, p.type_hint))
        .collect::<Vec<_>>()
        .join(", ");

    Recipe {
        name,
        deps,
        body: vec![tool_command_for_just(tool, config, dry_run)],
        comment: Some(format!("{} entrypoints: {}", tool.binary_name(), port_list)),
    }
}

fn tool_target_deps(tool: &ToolInfo) -> Vec<String> {
    if tool.needs_generated_cli {
        vec!["ensure-codegen".to_string()]
    } else {
        Vec::new()
    }
}

#[allow(clippy::disallowed_methods)] // Build-time DSL data loading.
fn load_build_targets_data() -> (
    Vec<CoreWorkflowData>,
    Vec<MetaTargetData>,
    Vec<ResourceTargetEntryData>,
) {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let dag_file = dsl_root.join("config/build_targets.dag");
    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(dag_file),
    };
    let output = compile_from_context(&context).expect("build_targets.dag should compile");
    let workflows = serde_json::from_value::<Vec<CoreWorkflowData>>(
        output
            .data_values
            .get("core_workflows")
            .cloned()
            .expect("core_workflows data should exist"),
    )
    .expect("core_workflows should deserialize");
    let metas = serde_json::from_value::<Vec<MetaTargetData>>(
        output
            .data_values
            .get("meta_targets")
            .cloned()
            .expect("meta_targets data should exist"),
    )
    .expect("meta_targets should deserialize");
    let resource_targets = serde_json::from_value::<Vec<ResourceTargetEntryData>>(
        output
            .data_values
            .get("resource_targets")
            .cloned()
            .expect("resource_targets data should exist"),
    )
    .expect("resource_targets should deserialize");
    (workflows, metas, resource_targets)
}

fn normalize_make_command_for_just(command: &str) -> String {
    command
        .replace("@$(MAKE) ", "@just ")
        .replace("$(MAKE) ", "just ")
}

fn tool_command_for_just(tool: &ToolInfo, config: &BuildConfig, dry_run: bool) -> String {
    let cli_args = render_shell_cli_args(&tool.entrypoints);
    let warning_prefix = if config.warnings == Warnings::Deny {
        "RUSTFLAGS=\"-D warnings\" "
    } else {
        ""
    };
    let env_prefix = "";

    if dry_run {
        format!(
            "@{}{}{} -- --dry-run{}",
            env_prefix,
            warning_prefix,
            tool.invocation.command(),
            cli_args
        )
    } else {
        format!(
            "@{}{}{} --{}",
            env_prefix,
            warning_prefix,
            tool.invocation.command(),
            cli_args
        )
    }
}

fn render_shell_cli_args(params: &[EntrypointParam]) -> String {
    if params.is_empty() {
        return String::new();
    }

    params
        .iter()
        .map(|p| {
            if p.repeatable {
                format!(
                    " $(for v in ${{{}}}; do printf ' {} %s' \"$v\"; done)",
                    p.make_var, p.cli_flag
                )
            } else {
                format!(
                    " ${{{}:+{} \"${{{}}}\"}}",
                    p.make_var, p.cli_flag, p.make_var
                )
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn collect_entrypoint_vars(registry: &ToolRegistry) -> BTreeMap<String, String> {
    let mut vars = BTreeMap::new();
    for tool in &registry.tools {
        for param in &tool.entrypoints {
            vars.entry(param.make_var.clone())
                .or_insert_with(|| param.default.clone().unwrap_or_default());
        }
    }
    vars
}

fn escape_just_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::makegen::shared::render_makefile;
    use std::collections::BTreeSet;

    #[test]
    fn test_render_justfile_has_header_and_help() {
        let registry = ToolRegistry::default_registry();
        let justfile = render_justfile(&registry);
        assert!(justfile.contains("Generated by gunbc-makegen"));
        assert!(justfile.contains("help:"));
        assert!(justfile.contains("set shell := [\"bash\", \"-cu\"]"));
    }

    #[test]
    fn test_justfile_target_graph_matches_makefile() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);
        let justfile = render_justfile(&registry);

        let make_graph = parse_target_graph(&makefile);
        let just_graph = parse_target_graph(&justfile);
        assert_eq!(make_graph, just_graph);
    }

    fn parse_target_graph(content: &str) -> BTreeMap<String, BTreeSet<String>> {
        let mut graph = BTreeMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with('.') || trimmed.contains(":=") {
                continue;
            }
            if line.starts_with(' ') || line.starts_with('\t') {
                continue;
            }

            let Some((name, deps)) = trimmed.split_once(':') else {
                continue;
            };
            let target = name.trim();
            if target.is_empty() {
                continue;
            }
            let deps = deps
                .split_whitespace()
                .filter(|dep| !dep.is_empty())
                .map(|dep| dep.to_string())
                .collect::<BTreeSet<_>>();
            graph.insert(target.to_string(), deps);
        }

        graph
    }
}
