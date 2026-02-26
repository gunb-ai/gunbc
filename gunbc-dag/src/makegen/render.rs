//! Makefile rendering.
//!
//! Uses `BuildConfig` as the single source of truth for all build commands.
//! Uses `FileHeader` for standardized header generation.
//!
//! Uses `MAKEFILE.indent` from the language module for tab indentation.
//! Uses `MakefileStructuredRenderer` to render `StructuredBlock` IR.

use std::borrow::Cow;

use crate::makegen::registry::{
    BuildConfig, EntrypointParam, ExtraTarget, MetaTarget, ResourceTargetMap, ToolInfo,
    ToolRegistry, WorkflowSpec,
};
use crate::WorkspaceBinary;
use gunbc_ir::cargo::{CargoCommand, CargoInvocation, Subcommand};
use gunbc_ir::render_ir::{FileHeader, PlainText, StructuredBlock, StructuredRenderer, Target};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::symbols::{Tier, STANDARD};
use gunbc_ir::MakefileStructuredRenderer;

// ============================================================================
// MakefileRenderer
// ============================================================================

/// Renderer for Makefiles with standardized header generation.
///
/// Wraps a `ToolRegistry` and `BuildConfig` to produce a complete Makefile
/// with standardized header format.
pub struct MakefileRenderer<'a> {
    pub registry: &'a ToolRegistry,
    pub config: BuildConfig,
}

impl<'a> MakefileRenderer<'a> {
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
}

/// Composed generator name for the Makefile/gitignore renderer.
/// This must match `cargo::name("makegen")` — verified by test.
pub(crate) const MAKEGEN_NAME: &str = "gunbc-makegen";

impl MakefileRenderer<'_> {
    /// Render the complete Makefile with header.
    pub fn render(&self) -> String {
        let regenerate_cmd =
            CargoCommand::new(Subcommand::Run(WorkspaceBinary::Makegen.invocation()));
        let header = FileHeader {
            generator_name: MAKEGEN_NAME.into(),
            regenerate_command: regenerate_cmd.to_shell().into(),
            comment_prefix: "#".into(),
        };
        format!(
            "{}\n\n{}",
            header.render(),
            render_makefile_content(self.registry, &self.config)
        )
    }

    /// Render just the content without header.
    pub fn render_content(&self) -> String {
        render_makefile_content(self.registry, &self.config)
    }
}

// ============================================================================
// Public API (backwards compatible)
// ============================================================================

/// Render a complete Makefile using the default tool registry.
pub fn render_makefile() -> String {
    let registry = ToolRegistry::default_registry();
    MakefileRenderer::new(&registry).render()
}

/// Render a complete Makefile with a specific build config.
pub fn render_makefile_with_config(registry: &ToolRegistry, config: &BuildConfig) -> String {
    MakefileRenderer::with_config(registry, config.clone()).render()
}

// ============================================================================
// Structured IR construction
// ============================================================================

fn plain_medium() -> PlainText {
    PlainText {
        tier: Tier::Ascii,
        symbol_set: &STANDARD,
    }
}

/// Render Makefile content without the header.
///
/// Builds `Vec<StructuredBlock>` IR, then renders via `MakefileStructuredRenderer`.
fn render_makefile_content(registry: &ToolRegistry, config: &BuildConfig) -> String {
    let blocks = build_makefile_blocks(registry, config);
    let renderer = MakefileStructuredRenderer::new(plain_medium());

    let mut output = String::new();
    for block in &blocks {
        output.push_str(&renderer.render_block(block));
    }
    output
}

/// Build the complete Makefile as structured blocks.
fn build_makefile_blocks(registry: &ToolRegistry, config: &BuildConfig) -> Vec<StructuredBlock> {
    let mut blocks = Vec::new();

    // Naming convention header
    blocks.push(StructuredBlock::Raw(
        "# Naming convention:\n\
         #   make <target>      - verify only (CI-safe, fails on issues)\n\
         #   make <target>-fix  - auto-fix then verify (for dev)\n\
         #\n\
         # Dev default:     make test      (ensure generated artifacts, then test)\n\
         # Dev workflow:    make test-fix  (fmt/lint fix + ensure generated artifacts, then test)\n\
         # CI verification: make verify    (check generated artifacts)\n\n"
            .into(),
    ));

    // Default goal
    blocks.push(StructuredBlock::Raw(".DEFAULT_GOAL := help\n\n".into()));

    // .PHONY
    blocks.push(StructuredBlock::Raw(build_phony_line(registry)));

    // Core build system targets
    blocks.extend(build_core_targets(registry, config));

    // Help target
    blocks.push(build_help_target(registry, config));

    // Meta targets
    blocks.extend(build_meta_targets(registry, config));

    // Tool targets
    for tool in &registry.tools {
        blocks.push(build_tool_target(tool, config));
        blocks.push(build_dry_run_target(tool, config));
        for extra in &tool.extra_targets {
            blocks.push(build_extra_target(tool, extra));
        }
    }

    blocks
}

/// Build the .PHONY line.
fn build_phony_line(registry: &ToolRegistry) -> String {
    let mut names = vec!["help".to_string()];
    names.extend(
        registry
            .core_workflows
            .iter()
            .map(|workflow| workflow.name.clone()),
    );

    for meta in &registry.meta_targets {
        names.push(meta.name.clone());
        if meta.has_check_variant {
            names.push(format!("{}-check", meta.name));
        }
        if meta.has_fix_variant {
            names.push(format!("{}-fix", meta.name));
        }
    }

    for tool in &registry.tools {
        names.push(tool.short_name.clone());
        names.push(format!("{}-dry", tool.short_name));
        for extra in &tool.extra_targets {
            names.push(format!("{}-{}", tool.short_name, extra.suffix));
        }
    }

    let mut seen = std::collections::BTreeSet::new();
    names.retain(|name| seen.insert(name.clone()));
    format!(".PHONY: {}\n\n", names.join(" "))
}

/// Build core targets from registry workflow specs.
fn build_core_targets(registry: &ToolRegistry, config: &BuildConfig) -> Vec<StructuredBlock> {
    registry
        .core_workflows
        .iter()
        .map(|workflow| {
            let deps = workflow
                .deps
                .iter()
                .cloned()
                .map(Cow::Owned)
                .collect::<Vec<_>>();
            StructuredBlock::Target(Target {
                name: workflow.name.clone().into(),
                deps,
                body: core_workflow_body(workflow, config),
                comment: Some(core_workflow_comment(workflow, config).into()),
            })
        })
        .collect()
}

pub(crate) fn core_workflow_comment(workflow: &WorkflowSpec, config: &BuildConfig) -> String {
    if workflow.name == "build" {
        let build_desc = if config.use_dag_entrypoints {
            "codegen \u{2192} testgen \u{2192} gunbc-build"
        } else {
            "codegen \u{2192} testgen \u{2192} cargo build"
        };
        return format!("Full build transaction: {build_desc}");
    }
    workflow.description.clone()
}

pub(crate) fn core_workflow_body(
    workflow: &WorkflowSpec,
    config: &BuildConfig,
) -> Vec<Cow<'static, str>> {
    match workflow.name.as_str() {
        "preflight-fix" => {
            vec!["@cargo fix --workspace --all-targets --allow-dirty --allow-staged".into()]
        }
        "ensure-codegen" => vec![config.ensure_codegen.shell().into()],
        "build-release-bins" => {
            vec!["@RUSTFLAGS=\"-D warnings\" cargo build --workspace --release --bins".into()]
        }
        "lint-upsert" => {
            let lint_cmd = config.lint.to_shell();
            let lint_fix_cmd = config.lint_fix.to_shell();
            let lint_upsert = format!("@{} || ({} && {})", lint_cmd, lint_fix_cmd, lint_cmd);
            vec![config.pragma.shell().into(), lint_upsert.into()]
        }
        "codegen" => vec![config.codegen.shell().into()],
        "build" => vec![config.build.shell().into()],
        "clean" => vec!["@cargo clean".into()],
        "testgen" => vec![config.testgen.shell().into()],
        "testgen-check" => vec![config.testgen.with_mode(ExecMode::Verify).shell().into()],
        "deps-config" => vec![format!(
            "@target/release/{} --mode=ensure",
            WorkspaceBinary::DepsConfig.invocation().binary
        )
        .into()],
        "deps-config-check" => vec![format!(
            "@target/release/{} --mode=verify",
            WorkspaceBinary::DepsConfig.invocation().binary
        )
        .into()],
        "makegen-check" => vec![config.makegen.with_mode(ExecMode::Verify).shell().into()],
        "bootstrap-check" => vec![config.bootstrap.with_mode(ExecMode::Verify).shell().into()],
        "pragma-check" => vec![config.pragma.with_mode(ExecMode::Verify).shell().into()],
        "verify" => vec![
            "@$(MAKE) deps-config-check".into(),
            "@$(MAKE) makegen-check".into(),
            "@$(MAKE) bootstrap-check".into(),
            "@$(MAKE) testgen-check".into(),
            "@$(MAKE) pragma-check".into(),
        ],
        "verify-fix" => vec![
            "@$(MAKE) deps-config".into(),
            config.makegen.with_mode(ExecMode::Ensure).shell().into(),
            config.bootstrap.with_mode(ExecMode::Ensure).shell().into(),
            config.testgen.with_mode(ExecMode::Ensure).shell().into(),
            config.pragma.with_mode(ExecMode::Ensure).shell().into(),
        ],
        "fmt-fix" => vec![config.fmt.shell().into()],
        "lint-fix" => vec![config.lint_fix.shell().into()],
        // WF8: CI and test-all are thin wrappers over workflow planner execution.
        "ci" => vec![workflow_planner_command("ci", config).into()],
        "test-all" => {
            vec![workflow_planner_command("test-all", config).into()]
        }
        _ => panic!(
            "missing core workflow body renderer for '{}'",
            workflow.name
        ),
    }
}

fn workflow_secret_rows(registry: &ToolRegistry, config: &BuildConfig) -> Vec<(String, String)> {
    let mut rows: Vec<(String, String)> = registry
        .workflow_specs(config)
        .into_iter()
        .filter(|workflow| !workflow.live_secrets.is_empty())
        .map(|workflow| (workflow.name, workflow.live_secrets.join(", ")))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    rows
}

/// Build the help target as a raw block (complex echo formatting).
fn build_help_target(registry: &ToolRegistry, config: &BuildConfig) -> StructuredBlock {
    let mut lines: Vec<Cow<'static, str>> = vec![
        "@echo \"gunbc tools - generated Makefile\"".into(),
        "@echo \"\"".into(),
        "@echo \"Naming convention:\"".into(),
        "@echo \"  make <target>      - verify only (CI-safe)\"".into(),
        "@echo \"  make <target>-fix  - auto-fix then verify (for dev)\"".into(),
        "@echo \"\"".into(),
        // Build transactions section
        "@echo \"Build commands:\"".into(),
    ];
    for workflow in &registry.core_workflows {
        let desc = core_workflow_comment(workflow, config);
        lines.push(format!("@echo \"  {}  - {}\"", workflow.name, desc).into());
    }
    lines.push("@echo \"\"".into());

    // Meta targets section
    lines.push("@echo \"Development:\"".into());
    for meta in &registry.meta_targets {
        lines.push(format!("@echo \"  {}  - {}\"", meta.name, meta.description).into());
        if meta.has_fix_variant {
            let deps = if meta.fix_prerequisites.is_empty() {
                "auto-fix".into()
            } else {
                let names: Vec<&str> = meta
                    .fix_prerequisites
                    .iter()
                    .map(|f| f.target_name())
                    .collect();
                format!("{} first", names.join(" + "))
            };
            lines.push(
                format!(
                    "@echo \"  {}-fix  - {} ({})\"",
                    meta.name, meta.description, deps
                )
                .into(),
            );
        }
        if meta.has_check_variant {
            lines.push(
                format!(
                    "@echo \"  {}-check  - {} (check only)\"",
                    meta.name, meta.description
                )
                .into(),
            );
        }
    }
    lines.push("@echo \"\"".into());

    // Tools section
    lines.push("@echo \"Tools:\"".into());
    for tool in &registry.tools {
        let params = render_help_params(&tool.entrypoints);
        lines.push(
            format!(
                "@echo \"  {} {}  - {}\"",
                tool.short_name, params, tool.description
            )
            .into(),
        );
        for extra in &tool.extra_targets {
            lines.push(
                format!(
                    "@echo \"  {}-{}  - {}\"",
                    tool.short_name, extra.suffix, extra.description
                )
                .into(),
            );
        }
    }
    lines.push("@echo \"\"".into());
    lines.push("@echo \"Add -dry suffix for dry-run (e.g., make deps-dry)\"".into());

    // Secrets section: show workflow metadata with live-secret requirements.
    let workflow_secrets = workflow_secret_rows(registry, config);
    if !workflow_secrets.is_empty() {
        lines.push("@echo \"\"".into());
        lines.push("@echo \"Required secrets (for live execution):\"".into());
        for (workflow_name, secrets) in workflow_secrets {
            lines.push(format!("@echo \"  {}: {}\"", workflow_name, secrets).into());
        }
    }

    StructuredBlock::Target(Target {
        name: "help".into(),
        deps: vec![],
        body: lines,
        comment: None,
    })
}

/// Build meta targets section as structured blocks.
fn build_meta_targets(registry: &ToolRegistry, config: &BuildConfig) -> Vec<StructuredBlock> {
    let mut blocks = Vec::new();
    let res_map = ResourceTargetMap::default_map(config);

    blocks.push(StructuredBlock::Raw(
        "# ============================================================================\n\
         # Meta Targets - Development workflow commands\n\
         # ============================================================================\n\n"
            .into(),
    ));

    for meta in &registry.meta_targets {
        blocks.push(build_meta_target(meta, config, &res_map));
        if meta.has_fix_variant {
            blocks.extend(build_meta_fix_variant(meta, config, &res_map));
        }
        if meta.has_check_variant {
            blocks.extend(build_meta_check_variant(meta, config, &res_map));
        }
    }

    blocks
}

/// Build the dependency list for a meta target (base/check variant).
///
/// Resolves each `ResourceNeed` using its `base_mode` via `ResourceTargetMap`.
pub(crate) fn meta_target_deps(
    meta: &MetaTarget,
    res_map: &ResourceTargetMap,
) -> Vec<Cow<'static, str>> {
    meta.workflow_spec(res_map)
        .deps
        .into_iter()
        .map(Cow::Owned)
        .collect()
}

/// Build a single meta target.
fn build_meta_target(
    meta: &MetaTarget,
    config: &BuildConfig,
    res_map: &ResourceTargetMap,
) -> StructuredBlock {
    let deps = meta_target_deps(meta, res_map);
    let command = meta.get_command(config);

    StructuredBlock::Target(Target {
        name: meta.name.clone().into(),
        deps,
        body: vec![command.into()],
        comment: Some(format!("{}: {}", meta.name, meta.description).into()),
    })
}

/// Build a check variant for a meta target.
fn build_meta_check_variant(
    meta: &MetaTarget,
    config: &BuildConfig,
    res_map: &ResourceTargetMap,
) -> Vec<StructuredBlock> {
    if let Some(check_cmd) = meta.get_check_command(config) {
        let deps = meta_target_deps(meta, res_map);

        vec![StructuredBlock::Target(Target {
            name: format!("{}-check", meta.name).into(),
            deps,
            body: vec![check_cmd.into()],
            comment: None,
        })]
    } else {
        vec![]
    }
}

/// Build a fix variant for a meta target.
///
/// Fix variants always resolve resources in Ensure mode (fix = ensure everything),
/// preceded by any fix_prerequisites (e.g., "fmt-fix", "lint-fix").
fn build_meta_fix_variant(
    meta: &MetaTarget,
    config: &BuildConfig,
    res_map: &ResourceTargetMap,
) -> Vec<StructuredBlock> {
    if !meta.has_fix_variant {
        return vec![];
    }

    let mut deps: Vec<Cow<'static, str>> = Vec::new();

    // Fix prerequisites first (e.g., FmtFix, LintFix)
    for dep in &meta.fix_prerequisites {
        deps.push(dep.target_name().into());
    }

    // All resources resolved in Ensure mode
    for need in &meta.resources {
        let target = res_map
            .resolve(&need.id, ExecMode::Ensure)
            .unwrap_or_else(|| {
                panic!(
                    "missing resource target mapping for {:?} ({:?}) in fix variant '{}-fix'",
                    need.id,
                    ExecMode::Ensure,
                    meta.name
                )
            });
        deps.push(Cow::Owned(target.to_string()));
    }

    let fix_cmd = meta
        .get_fix_command(config)
        .unwrap_or_else(|| meta.get_command(config));

    vec![StructuredBlock::Target(Target {
        name: format!("{}-fix", meta.name).into(),
        deps,
        body: vec![fix_cmd.into()],
        comment: Some(format!("{}-fix: auto-fix then verify", meta.name).into()),
    })]
}

/// Render help text for parameters.
fn render_help_params(params: &[EntrypointParam]) -> String {
    params
        .iter()
        .map(|p| {
            let repeat_suffix = if p.repeatable { " ..." } else { "" };
            if let Some(ref default) = p.default {
                format!("[{}={}{}]", p.make_var, default, repeat_suffix)
            } else {
                format!("[{}=...{}]", p.make_var, repeat_suffix)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Build a tool target.
fn build_tool_target(tool: &ToolInfo, config: &BuildConfig) -> StructuredBlock {
    let port_list = tool
        .entrypoints
        .iter()
        .map(|p| format!("{} ({})", p.port_name, p.type_hint))
        .collect::<Vec<_>>()
        .join(", ");

    let deps = tool_target_deps(tool, config);
    let cmd = tool_command(tool, config, false);

    StructuredBlock::Target(Target {
        name: tool.short_name.clone().into(),
        deps,
        body: vec![cmd.into()],
        comment: Some(format!("{} entrypoints: {}", tool.binary_name(), port_list).into()),
    })
}

/// Build a dry-run target.
fn build_dry_run_target(tool: &ToolInfo, config: &BuildConfig) -> StructuredBlock {
    let deps = tool_target_deps(tool, config);
    let cmd = tool_command(tool, config, true);

    StructuredBlock::Target(Target {
        name: format!("{}-dry", tool.short_name).into(),
        deps,
        body: vec![cmd.into()],
        comment: None,
    })
}

pub(crate) fn tool_target_deps(tool: &ToolInfo, config: &BuildConfig) -> Vec<Cow<'static, str>> {
    tool.workflow_spec(config)
        .deps
        .into_iter()
        .map(Cow::Owned)
        .collect()
}

fn tool_command(tool: &ToolInfo, config: &BuildConfig, dry_run: bool) -> String {
    workflow_tool_command(tool, dry_run, config)
}

/// Render a workflow planner command for core workflows (ci, test-all).
fn workflow_planner_command(name: &str, config: &BuildConfig) -> String {
    let workflow_inv = CargoInvocation::composed("workflow", "dag");
    let cmd = CargoCommand::new(Subcommand::Run(workflow_inv))
        .quiet()
        .release()
        .warnings(config.warnings);
    format!("@{} -- {name}", cmd.to_shell_with_env())
}

/// Render a workflow-dispatched tool command.
///
/// All tool targets dispatch through `gunbc-workflow` run mode via `cargo run`,
/// so cold-start clones and stale binaries are handled by Cargo freshness.
/// Uses `CargoCommand` from `BuildConfig` to inherit the repo's warning policy.
fn workflow_tool_command(tool: &ToolInfo, dry_run: bool, config: &BuildConfig) -> String {
    let workflow_inv = CargoInvocation::composed("workflow", "dag");
    let cmd = CargoCommand::new(Subcommand::Run(workflow_inv))
        .quiet()
        .release()
        .warnings(config.warnings);

    let mut shell = format!("@{}", cmd.to_shell_with_env());
    shell.push_str(&format!(" -- {}", tool.short_name));
    if dry_run {
        shell.push_str(" --dry-run strict");
    }
    shell
}

/// Build an extra composite target.
fn build_extra_target(tool: &ToolInfo, extra: &ExtraTarget) -> StructuredBlock {
    StructuredBlock::Target(Target {
        name: format!("{}-{}", tool.short_name, extra.suffix).into(),
        deps: vec![tool.short_name.clone().into()],
        body: extra
            .post_commands
            .iter()
            .map(|s| Cow::from(s.clone()))
            .collect(),
        comment: Some(
            format!(
                "{}-{}: {}",
                tool.short_name, extra.suffix, extra.description
            )
            .into(),
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::cargo;

    #[test]
    fn test_render_makefile_has_header() {
        let makefile = render_makefile();

        assert!(makefile.contains("Generated by gunbc-makegen"));
        assert!(makefile.contains(".PHONY:"));
        assert!(makefile.contains("help:"));
    }

    #[test]
    fn test_render_makefile_has_targets() {
        let makefile = render_makefile();

        assert!(makefile.contains("deps:"));
        assert!(makefile.contains("deps-dry:"));
        assert!(makefile.contains("pragma:"));
        assert!(makefile.contains("pragma-dry:"));
    }

    #[test]
    fn test_render_makefile_help_mentions_entrypoint_variables() {
        let makefile = render_makefile();

        // DSL convention: make_var = UPPER_SNAKE(param_name), so manifest_path → MANIFEST_PATH
        assert!(makefile.contains("[MANIFEST_PATH="));
        assert!(
            !makefile.contains("$(if $(MANIFEST_PATH)"),
            "tool entrypoint args should not be threaded through workflow wrapper commands"
        );
    }

    #[test]
    fn test_render_help_params_repeatable() {
        let params = vec![EntrypointParam {
            port_name: "extensions".into(),
            make_var: "EXT".into(),
            cli_flag: "--extensions".into(),
            type_hint: "String".into(),
            default: None,
            repeatable: true,
        }];

        let rendered = render_help_params(&params);
        assert_eq!(rendered, "[EXT=... ...]");
    }

    #[test]
    fn test_render_makefile_has_core_targets() {
        let makefile = render_makefile();

        // ensure-codegen has no prerequisites (minimal pipeline)
        assert!(
            makefile.contains("ensure-codegen:\n"),
            "ensure-codegen should have no prerequisites"
        );
        assert!(makefile.contains("codegen: lint-upsert"));
        assert!(makefile.contains("build: codegen testgen"));
        assert!(makefile.contains("build-release-bins: ensure-codegen"));
        assert!(makefile.contains("testgen: lint-upsert"));
        assert!(makefile.contains("testgen-check: lint-upsert"));
        assert!(makefile.contains("makegen-check: lint-upsert"));
        assert!(makefile.contains("bootstrap-check: lint-upsert"));
        assert!(makefile.contains("pragma-check: lint-upsert"));
        assert!(makefile.contains("gunbc-pragma -- --mode=verify"));
        assert!(makefile.contains("clean:"));
    }

    #[test]
    fn test_render_makefile_has_meta_targets() {
        let makefile = render_makefile();

        // Meta targets section
        assert!(makefile.contains("# Meta Targets"));

        // Individual meta targets
        assert!(
            makefile.contains("test: build verify-fix"),
            "test should depend on build and verify-fix (testgen is included in build)"
        );
        assert!(makefile.contains("cargo test"));
        assert!(makefile.contains("test-integration: build verify-fix"));
        assert!(makefile.contains("cargo test integration"));
        assert!(makefile.contains("test-external: build verify-fix"));
        assert!(makefile.contains("cargo test live_flow"));

        assert!(makefile.contains("check: ensure-codegen"));
        assert!(makefile.contains("cargo check --all-targets"));

        assert!(makefile.contains("clippy: ensure-codegen"));
        assert!(makefile.contains("cargo clippy --all-targets"));

        assert!(makefile.contains("fmt:"));
        assert!(makefile.contains("@cargo fmt"));
        assert!(makefile.contains("fmt-check:"));
    }

    #[test]
    fn test_render_makefile_help_has_sections() {
        let makefile = render_makefile();

        assert!(makefile.contains("Build commands:"));
        assert!(makefile.contains("codegen  - Generate CLI entrypoints (DAG upsert)"));
        assert!(
            makefile.contains("ensure-codegen  - Ensure CLI entrypoints exist (bootstrap-safe)")
        );
        assert!(makefile.contains(
            "preflight-fix  - Preflight: auto-fix rustc warnings before running generators"
        ));
        assert!(makefile.contains("lint-upsert  - Lint upsert: fix if needed, then verify"));
        assert!(makefile.contains("Development:"));
        assert!(makefile.contains("Tools:"));
    }

    #[test]
    fn test_render_help_includes_live_secrets_from_workflow_metadata() {
        let mut registry = ToolRegistry::new();
        registry.register_core_workflow(WorkflowSpec {
            name: "deploy".to_string(),
            description: "Deploy workflow".to_string(),
            kind: crate::makegen::registry::WorkflowKind::Core,
            entrypoints: Vec::new(),
            deps: vec!["secure-tool".to_string()],
            resources: Vec::new(),
            live_secrets: Vec::new(),
        });

        let mut secure_tool = ToolInfo::new("gunbc-secure-tool", "secure-tool", "Secure tool");
        secure_tool.live_secrets = vec!["SECURE_TOKEN".to_string()];
        registry.register(secure_tool);

        let help = build_help_target(&registry, &BuildConfig::cargo());
        let StructuredBlock::Target(target) = help else {
            panic!("help should be rendered as a target block");
        };
        let help_text = target
            .body
            .iter()
            .map(|line| line.as_ref())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(help_text.contains("Required secrets (for live execution):"));
        assert!(help_text.contains("deploy: SECURE_TOKEN"));
        assert!(help_text.contains("secure-tool: SECURE_TOKEN"));
    }

    #[test]
    fn test_render_makefile_has_testgen_targets() {
        let makefile = render_makefile();

        // Testgen targets in core section
        assert!(makefile.contains("testgen:"), "should have testgen target");
        assert!(
            makefile.contains("testgen-check:"),
            "should have testgen-check target"
        );
        assert!(
            makefile.contains("makegen-check:"),
            "should have makegen-check target"
        );
        assert!(
            makefile.contains("bootstrap-check:"),
            "should have bootstrap-check target"
        );
        assert!(
            makefile.contains("pragma-check:"),
            "should have pragma-check target"
        );

        // Testgen in help
        assert!(
            makefile.contains("testgen  - Regenerate tests"),
            "help should mention testgen"
        );
        assert!(
            makefile.contains("pragma-check  - Check if pragma artifacts are stale"),
            "help should mention pragma-check"
        );

        // Testgen in .PHONY
        assert!(
            makefile.contains(
                "testgen testgen-check deps-config deps-config-check makegen-check bootstrap-check pragma-check"
            ),
            "should be in .PHONY"
        );
        assert!(
            makefile.contains("ensure-codegen"),
            "should include ensure-codegen in .PHONY"
        );
        assert!(
            makefile.contains("lint-upsert"),
            "should include lint-upsert in .PHONY"
        );
        assert!(
            makefile.contains("codegen"),
            "should include codegen in .PHONY"
        );
    }

    #[test]
    fn test_render_makefile_with_build_config() {
        let registry = ToolRegistry::default_registry();
        let config = BuildConfig::cargo();
        let makefile = render_makefile_with_config(&registry, &config);

        // Should use BuildConfig commands
        assert!(makefile.contains("cargo build --all-targets"));
        assert!(makefile.contains("cargo test"));
        assert!(makefile.contains("cargo clippy"));
    }

    #[test]
    fn test_render_makefile_tool_targets_deny_warnings() {
        let registry = ToolRegistry::default_registry();
        let config = BuildConfig::cargo();
        let makefile = render_makefile_with_config(&registry, &config);

        assert!(makefile.contains("build-release-bins: ensure-codegen"));
        assert!(
            makefile.contains("RUSTFLAGS=\"-D warnings\" cargo build --workspace --release --bins")
        );
        // Tool targets dispatch through workflow binary via cargo run with RUSTFLAGS.
        assert!(
            makefile.contains("@RUSTFLAGS=\"-D warnings\" cargo run -p gunbc-dag --bin gunbc-workflow -q --release -- deps"),
            "tool targets should dispatch through gunbc-workflow with warning policy"
        );
        assert!(
            makefile.contains(
                "@RUSTFLAGS=\"-D warnings\" cargo run -p gunbc-dag --bin gunbc-workflow -q --release -- deps --dry-run strict"
            ),
            "dry-run targets should pass strict dry-run mode to gunbc-workflow"
        );
    }

    #[test]
    fn test_render_makefile_buck2_config() {
        let registry = ToolRegistry::default_registry();
        let config = BuildConfig::buck2();
        let makefile = render_makefile_with_config(&registry, &config);

        // Should use buck2 commands for build/test
        assert!(makefile.contains("buck2 build //..."));
        assert!(makefile.contains("buck2 test //..."));
    }

    // ========================================================================
    // FileHeader + Render Tests
    // ========================================================================

    #[test]
    fn test_makefile_renderer_generator_name() {
        // Verify the const matches the composed name
        assert_eq!(MAKEGEN_NAME, cargo::name("makegen"));
    }

    #[test]
    fn test_makefile_renderer_render() {
        let registry = ToolRegistry::default_registry();
        let renderer = MakefileRenderer::new(&registry);

        // Test that render() produces standardized header
        let output = renderer.render();
        assert!(output.contains("# Generated by gunbc-makegen"));
        assert!(output.contains(
            "# DO NOT EDIT - regenerate with: cargo run -p gunbc-dag --bin gunbc-makegen"
        ));

        // Test that render_content() produces just the body
        let content = renderer.render_content();
        assert!(!content.contains("Generated by"));
        assert!(content.contains(".DEFAULT_GOAL := help"));
    }

    // ========================================================================
    // Fix Variant Tests (the-gunbai dev UX convention)
    // ========================================================================

    #[test]
    fn test_render_makefile_has_naming_convention_header() {
        let makefile = render_makefile();

        assert!(makefile.contains("# Naming convention:"));
        assert!(makefile.contains("make <target>      - verify only"));
        assert!(makefile.contains("make <target>-fix  - auto-fix then verify"));
    }

    #[test]
    fn test_render_makefile_has_fix_alias_targets() {
        let makefile = render_makefile();

        // Should have fmt-fix and lint-fix as dependency targets
        assert!(makefile.contains("fmt-fix:"));
        assert!(makefile.contains("lint-fix: pragma"));
        assert!(makefile.contains("@cargo fmt")); // fmt-fix uses fmt command
        assert!(makefile.contains("--fix")); // lint-fix uses clippy --fix
    }

    #[test]
    fn test_render_makefile_has_fix_variants() {
        let makefile = render_makefile();

        // test-fix should depend on fmt-fix and lint-fix
        assert!(makefile.contains("test-fix:"));
        assert!(makefile.contains("fmt-fix"));
        assert!(makefile.contains("lint-fix"));

        // clippy-fix should exist
        assert!(makefile.contains("clippy-fix: ensure-codegen pragma"));

        // check-fix should exist
        assert!(makefile.contains("check-fix:"));
    }

    #[test]
    fn test_render_makefile_help_shows_fix_variants() {
        let makefile = render_makefile();

        // Help should mention fix variants
        assert!(makefile.contains("test-fix"));
        assert!(makefile.contains("clippy-fix"));
    }

    #[test]
    fn test_structured_blocks_use_target_ir() {
        let registry = ToolRegistry::default_registry();
        let config = BuildConfig::cargo();
        let blocks = build_makefile_blocks(&registry, &config);

        // Count Raw vs Target blocks — Raw should only be used for
        // non-target content (headers, directives, section banners).
        let mut target_count = 0;
        let mut raw_contents = Vec::new();

        for block in &blocks {
            match block {
                StructuredBlock::Raw(s) => {
                    raw_contents.push(s.clone());
                }
                StructuredBlock::Target(_) => {
                    target_count += 1;
                }
                _ => {}
            }
        }

        // All build targets should use Target IR, not Raw
        assert!(
            target_count > 20,
            "expected many Target blocks, got {}",
            target_count
        );

        // Raw blocks should only contain directives and section headers
        for raw in &raw_contents {
            assert!(
                raw.starts_with('#') || raw.starts_with('.') || raw.contains("============"),
                "unexpected Raw block that looks like a target definition: {:?}",
                &raw[..raw.len().min(80)]
            );
        }
    }

    #[test]
    fn tool_targets_use_minimal_prerequisites() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile();

        // Tool targets should have no Make prerequisites; freshness is planner-managed.
        for tool in &registry.tools {
            let expected = format!("{}:", tool.short_name);
            assert!(
                makefile.contains(&expected),
                "tool '{}' should exist as a make target",
                tool.short_name
            );
            assert!(
                !makefile.contains(&format!("{}: ensure-codegen", tool.short_name)),
                "tool '{}' should not depend on ensure-codegen",
                tool.short_name
            );
        }

        // Maintenance targets MUST still use lint-upsert
        for target in [
            "codegen",
            "testgen",
            "testgen-check",
            "makegen-check",
            "bootstrap-check",
            "pragma-check",
        ] {
            assert!(
                makefile.contains(&format!("{target}: lint-upsert")),
                "maintenance target '{target}' must depend on lint-upsert for full verification"
            );
        }
    }

    #[test]
    fn test_tool_targets_dispatch_via_workflow_binary() {
        let makefile = render_makefile();

        // Bootstrap should use workflow run dispatch, no ensure-codegen dep.
        assert!(
            makefile.contains("cargo run -p gunbc-dag --bin gunbc-workflow -q --release -- bootstrap"),
            "bootstrap should dispatch via gunbc-workflow"
        );
        assert!(
            !makefile.contains("bootstrap: ensure-codegen"),
            "bootstrap should not have ensure-codegen prerequisite"
        );

        // Pragma should use gunbc-workflow dispatch
        assert!(
            makefile.contains("cargo run -p gunbc-dag --bin gunbc-workflow -q --release -- pragma"),
            "pragma should dispatch via gunbc-workflow"
        );

        // Deps should also dispatch via workflow (no legacy cargo-run path).
        assert!(
            makefile.contains("cargo run -p gunbc-dag --bin gunbc-workflow -q --release -- deps"),
            "deps should dispatch via gunbc-workflow"
        );
        assert!(
            !makefile.contains("deps: ensure-codegen"),
            "deps should not have ensure-codegen prerequisite"
        );
    }

    #[test]
    fn test_tool_dry_run_uses_strict_mode() {
        let makefile = render_makefile();

        assert!(
            makefile.contains(
                "cargo run -p gunbc-dag --bin gunbc-workflow -q --release -- bootstrap --dry-run strict"
            ),
            "bootstrap-dry should dispatch via gunbc-workflow with --dry-run strict"
        );
    }

    #[test]
    fn test_no_tool_target_uses_cargo_run_directly() {
        let makefile = render_makefile();

        assert!(
            !makefile.contains("cargo run -p gunbc-deps --bin gunbc-deps"),
            "tool targets must dispatch through gunbc-workflow"
        );
    }
}
