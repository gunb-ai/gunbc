//! Makefile rendering.
//!
//! Uses `BuildConfig` as the single source of truth for all build commands.
//! Uses `FileHeader` for standardized header generation.
//!
//! Uses `MAKEFILE.indent` from the language module for tab indentation.
//! Uses `MakefileStructuredRenderer` to render `StructuredBlock` IR.

use crate::makegen::registry::{
    BuildConfig, EntrypointParam, ExtraTarget, MetaTarget, ResourceTargetMap, ToolInfo,
    ToolRegistry,
};
use gunbc_ir::cargo::Warnings;
use gunbc_ir::resource::ExecMode;
use gunbc_ir::render_ir::{FileHeader, PlainText, StructuredBlock, StructuredRenderer, Target};
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
        let header = FileHeader {
            generator_name: MAKEGEN_NAME.to_string(),
            regenerate_command: "cargo run -p gunbc-dag --bin gunbc-makegen --release".to_string(),
            comment_prefix: "#".to_string(),
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

/// Render a complete Makefile from the tool registry.
pub fn render_makefile(registry: &ToolRegistry) -> String {
    MakefileRenderer::new(registry).render()
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
            .to_string(),
    ));

    // Default goal
    blocks.push(StructuredBlock::Raw(
        ".DEFAULT_GOAL := help\n\n".to_string(),
    ));

    // .PHONY
    blocks.push(StructuredBlock::Raw(build_phony_line(registry)));

    // Core build system targets
    blocks.extend(build_core_targets(config));

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
    let mut phony = String::from(
        ".PHONY: help preflight-fix lint-upsert ensure-codegen codegen build clean testgen testgen-check pragma-check verify verify-fix fmt-fix lint-fix",
    );

    for meta in &registry.meta_targets {
        phony.push_str(&format!(" {}", meta.name));
        if meta.has_check_variant {
            phony.push_str(&format!(" {}-check", meta.name));
        }
        if meta.has_fix_variant {
            phony.push_str(&format!(" {}-fix", meta.name));
        }
    }

    for tool in &registry.tools {
        phony.push_str(&format!(" {} {}-dry", tool.short_name, tool.short_name));
        for extra in &tool.extra_targets {
            phony.push_str(&format!(" {}-{}", tool.short_name, extra.suffix));
        }
    }
    phony.push_str("\n\n");
    phony
}

/// Build core targets as structured blocks.
fn build_core_targets(config: &BuildConfig) -> Vec<StructuredBlock> {
    let mut blocks = Vec::new();
    let warning_prefix = if config.warnings == Warnings::Deny {
        "RUSTFLAGS=\"-D warnings\" "
    } else {
        ""
    };

    // preflight-fix
    blocks.push(StructuredBlock::Target(Target {
        name: "preflight-fix".to_string(),
        deps: vec![],
        body: vec![
            "@cargo fix --workspace --all-targets --allow-dirty --allow-staged".to_string(),
        ],
        comment: Some(
            "Preflight: auto-fix rustc warnings before running generators".to_string(),
        ),
    }));

    // ensure-codegen
    blocks.push(StructuredBlock::Target(Target {
        name: "ensure-codegen".to_string(),
        deps: vec!["preflight-fix".to_string()],
        body: vec![config.ensure_codegen_shell()],
        comment: Some("Ensure CLI entrypoints exist (bootstrap-safe)".to_string()),
    }));

    // lint-upsert
    let lint_cmd = config.lint.to_shell();
    let lint_fix_cmd = config.lint_fix.to_shell();
    let lint_upsert = format!("@{} || ({} && {})", lint_cmd, lint_fix_cmd, lint_cmd);
    blocks.push(StructuredBlock::Target(Target {
        name: "lint-upsert".to_string(),
        deps: vec!["ensure-codegen".to_string(), "pragma".to_string()],
        body: vec![lint_upsert],
        comment: Some("Lint upsert: fix if needed, then verify".to_string()),
    }));

    // codegen
    blocks.push(StructuredBlock::Target(Target {
        name: "codegen".to_string(),
        deps: vec!["lint-upsert".to_string()],
        body: vec![config.codegen_shell()],
        comment: Some("Generate CLI entrypoints (DAG upsert)".to_string()),
    }));

    // build
    let build_desc = if config.use_dag_entrypoints {
        "codegen \u{2192} testgen \u{2192} gunbc-build"
    } else {
        "codegen \u{2192} testgen \u{2192} cargo build"
    };
    blocks.push(StructuredBlock::Target(Target {
        name: "build".to_string(),
        deps: vec!["codegen".to_string(), "testgen".to_string()],
        body: vec![config.build_shell()],
        comment: Some(format!("Full build transaction: {build_desc}")),
    }));

    // clean
    blocks.push(StructuredBlock::Target(Target {
        name: "clean".to_string(),
        deps: vec![],
        body: vec!["@cargo clean".to_string()],
        comment: Some("Clean build artifacts".to_string()),
    }));

    // testgen
    blocks.push(StructuredBlock::Target(Target {
        name: "testgen".to_string(),
        deps: vec!["lint-upsert".to_string()],
        body: vec![config.testgen_shell()],
        comment: Some("Regenerate tests from DAG structures and MockSpecs".to_string()),
    }));

    // testgen-check
    blocks.push(StructuredBlock::Target(Target {
        name: "testgen-check".to_string(),
        deps: vec!["lint-upsert".to_string()],
        body: vec![config.testgen_check_shell()],
        comment: Some(
            "Check if generated tests are stale (fails if regeneration needed)".to_string(),
        ),
    }));

    // pragma-check
    blocks.push(StructuredBlock::Target(Target {
        name: "pragma-check".to_string(),
        deps: vec!["lint-upsert".to_string()],
        body: vec![format!(
            "@{}cargo run -p gunbc-dag --bin gunbc-pragma --release -- --mode=verify",
            warning_prefix
        )],
        comment: Some(
            "Check if pragma artifacts are stale (fails if regeneration needed)".to_string(),
        ),
    }));

    // verify
    blocks.push(StructuredBlock::Target(Target {
        name: "verify".to_string(),
        deps: vec!["lint-upsert".to_string()],
        body: vec![
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-makegen --release -- --mode=verify",
                warning_prefix
            ),
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-bootstrap --release -- --mode=verify",
                warning_prefix
            ),
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-testgen --release -- --mode=verify",
                warning_prefix
            ),
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-pragma --release -- --mode=verify",
                warning_prefix
            ),
        ],
        comment: Some(
            "Verify generated artifacts match their generators".to_string(),
        ),
    }));

    // verify-fix
    blocks.push(StructuredBlock::Target(Target {
        name: "verify-fix".to_string(),
        deps: vec!["lint-upsert".to_string()],
        body: vec![
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-makegen --release -- --mode=ensure",
                warning_prefix
            ),
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-bootstrap --release -- --mode=ensure",
                warning_prefix
            ),
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-testgen --release -- --mode=ensure",
                warning_prefix
            ),
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-pragma --release -- --mode=ensure",
                warning_prefix
            ),
        ],
        comment: Some("Ensure generated artifacts are up to date".to_string()),
    }));

    blocks
}

/// Build the help target as a raw block (complex echo formatting).
fn build_help_target(registry: &ToolRegistry, config: &BuildConfig) -> StructuredBlock {
    let mut lines: Vec<String> = vec![
        "@echo \"gunbc tools - generated Makefile\"".to_string(),
        "@echo \"\"".to_string(),
        "@echo \"Naming convention:\"".to_string(),
        "@echo \"  make <target>      - verify only (CI-safe)\"".to_string(),
        "@echo \"  make <target>-fix  - auto-fix then verify (for dev)\"".to_string(),
        "@echo \"\"".to_string(),
        // Build transactions section
        "@echo \"Build commands:\"".to_string(),
    ];
    let build_desc = if config.use_dag_entrypoints {
        "codegen \u{2192} testgen \u{2192} gunbc-build"
    } else {
        "codegen \u{2192} testgen \u{2192} cargo build"
    };
    lines.push(format!("@echo \"  build    - {build_desc}\""));
    lines.push("@echo \"  codegen  - Generate CLI entrypoints\"".to_string());
    lines.push(
        "@echo \"  ensure-codegen  - Bootstrap CLI entrypoints (safe on clean)\"".to_string(),
    );
    lines.push(
        "@echo \"  preflight-fix  - Auto-fix rustc warnings (workspace)\"".to_string(),
    );
    lines.push(
        "@echo \"  lint-upsert  - Auto-fix lint issues then verify\"".to_string(),
    );
    lines.push("@echo \"  clean    - Remove build artifacts\"".to_string());
    lines.push("@echo \"  testgen  - Regenerate tests from DAG structures\"".to_string());
    lines.push("@echo \"  testgen-check  - Check if generated tests are stale\"".to_string());
    lines.push(
        "@echo \"  pragma-check  - Check if pragma artifacts are stale\"".to_string(),
    );
    lines.push(
        "@echo \"  verify   - Verify generated artifacts match their generators\"".to_string(),
    );
    lines.push(
        "@echo \"  verify-fix  - Ensure generated artifacts are up to date\"".to_string(),
    );
    lines.push("@echo \"\"".to_string());

    // Meta targets section
    lines.push("@echo \"Development:\"".to_string());
    for meta in &registry.meta_targets {
        lines.push(format!(
            "@echo \"  {}  - {}\"",
            meta.name, meta.description
        ));
        if meta.has_fix_variant {
            let deps = if meta.fix_prerequisites.is_empty() {
                "auto-fix".to_string()
            } else {
                let names: Vec<&str> = meta.fix_prerequisites.iter().map(|f| f.target_name()).collect();
                format!("{} first", names.join(" + "))
            };
            lines.push(format!(
                "@echo \"  {}-fix  - {} ({})\"",
                meta.name, meta.description, deps
            ));
        }
        if meta.has_check_variant {
            lines.push(format!(
                "@echo \"  {}-check  - {} (check only)\"",
                meta.name, meta.description
            ));
        }
    }
    lines.push("@echo \"\"".to_string());

    // Tools section
    lines.push("@echo \"Tools:\"".to_string());
    for tool in &registry.tools {
        let params = render_help_params(&tool.entrypoints);
        lines.push(format!(
            "@echo \"  {} {}  - {}\"",
            tool.short_name, params, tool.description
        ));
        for extra in &tool.extra_targets {
            lines.push(format!(
                "@echo \"  {}-{}  - {}\"",
                tool.short_name, extra.suffix, extra.description
            ));
        }
    }
    lines.push("@echo \"\"".to_string());
    lines.push("@echo \"Add -dry suffix for dry-run (e.g., make gist-dry)\"".to_string());

    StructuredBlock::Target(Target {
        name: "help".to_string(),
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
            .to_string(),
    ));

    // Fix alias targets
    blocks.extend(build_fix_alias_targets(config));

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
fn meta_target_deps(
    meta: &MetaTarget,
    res_map: &ResourceTargetMap,
) -> Vec<String> {
    let mut deps: Vec<String> = Vec::new();

    for need in &meta.resources {
        if let Some(target) = res_map.resolve(&need.id, need.base_mode) {
            deps.push(target.to_string());
        }
    }

    deps
}

/// Build fix alias targets.
fn build_fix_alias_targets(config: &BuildConfig) -> Vec<StructuredBlock> {
    vec![
        StructuredBlock::Target(Target {
            name: "fmt-fix".to_string(),
            deps: vec![],
            body: vec![config.fmt_shell()],
            comment: Some("fmt-fix: apply formatting (alias for fmt)".to_string()),
        }),
        StructuredBlock::Target(Target {
            name: "lint-fix".to_string(),
            deps: vec!["pragma".to_string()],
            body: vec![config.lint_fix_shell()],
            comment: Some("lint-fix: auto-fix lint issues where possible".to_string()),
        }),
    ]
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
        name: meta.name.clone(),
        deps,
        body: vec![command],
        comment: Some(format!("{}: {}", meta.name, meta.description)),
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
            name: format!("{}-check", meta.name),
            deps,
            body: vec![check_cmd],
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

    let mut deps = Vec::new();

    // Fix prerequisites first (e.g., FmtFix, LintFix)
    for dep in &meta.fix_prerequisites {
        deps.push(dep.target_name().to_string());
    }

    // All resources resolved in Ensure mode
    for need in &meta.resources {
        if let Some(target) = res_map.resolve(&need.id, ExecMode::Ensure) {
            deps.push(target.to_string());
        }
    }

    let fix_cmd = meta
        .get_fix_command(config)
        .unwrap_or_else(|| meta.get_command(config));

    vec![StructuredBlock::Target(Target {
        name: format!("{}-fix", meta.name),
        deps,
        body: vec![fix_cmd],
        comment: Some(format!("{}-fix: auto-fix then verify", meta.name)),
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
    let warning_prefix = if config.warnings == Warnings::Deny {
        "RUSTFLAGS=\"-D warnings\" "
    } else {
        ""
    };

    let port_list = tool
        .entrypoints
        .iter()
        .map(|p| format!("{} ({})", p.port_name, p.type_hint))
        .collect::<Vec<_>>()
        .join(", ");

    let cli_args = render_cli_args(&tool.entrypoints);

    let deps = if tool.short_name == "pragma" {
        vec!["preflight-fix".to_string()]
    } else if tool.needs_generated_cli {
        vec!["lint-upsert".to_string()]
    } else {
        vec!["preflight-fix".to_string()]
    };

    StructuredBlock::Target(Target {
        name: tool.short_name.clone(),
        deps,
        body: vec![format!("@{}{} --{}", warning_prefix, tool.invocation.command(), cli_args)],
        comment: Some(format!("{} entrypoints: {}", tool.binary_name(), port_list)),
    })
}

/// Build a dry-run target.
fn build_dry_run_target(tool: &ToolInfo, config: &BuildConfig) -> StructuredBlock {
    let warning_prefix = if config.warnings == Warnings::Deny {
        "RUSTFLAGS=\"-D warnings\" "
    } else {
        ""
    };

    let cli_args = render_cli_args(&tool.entrypoints);

    let deps = if tool.short_name == "pragma" {
        vec!["preflight-fix".to_string()]
    } else if tool.needs_generated_cli {
        vec!["lint-upsert".to_string()]
    } else {
        vec!["preflight-fix".to_string()]
    };

    StructuredBlock::Target(Target {
        name: format!("{}-dry", tool.short_name),
        deps,
        body: vec![format!(
            "@{}{} -- --dry-run{}",
            warning_prefix,
            tool.invocation.command(),
            cli_args
        )],
        comment: None,
    })
}

/// Build an extra composite target.
fn build_extra_target(tool: &ToolInfo, extra: &ExtraTarget) -> StructuredBlock {
    StructuredBlock::Target(Target {
        name: format!("{}-{}", tool.short_name, extra.suffix),
        deps: vec![tool.short_name.clone()],
        body: extra.post_commands.clone(),
        comment: Some(format!(
            "{}-{}: {}",
            tool.short_name, extra.suffix, extra.description
        )),
    })
}

/// Render CLI arguments from entrypoint parameters.
fn render_cli_args(params: &[EntrypointParam]) -> String {
    if params.is_empty() {
        return String::new();
    }

    let args: Vec<String> = params
        .iter()
        .map(|p| {
            if p.repeatable {
                // $(if $(VAR),$(foreach v,$(VAR),--flag $(v)))
                format!(
                    " $(if $({}),$(foreach v,$({}),{} $(v)))",
                    p.make_var, p.make_var, p.cli_flag
                )
            } else {
                // $(if $(VAR),--flag $(VAR))
                format!(
                    " $(if $({}),{} $({}))",
                    p.make_var, p.cli_flag, p.make_var
                )
            }
        })
        .collect();

    args.join("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::cargo;

    #[test]
    fn test_render_makefile_has_header() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        assert!(makefile.contains("Generated by gunbc-makegen"));
        assert!(makefile.contains(".PHONY:"));
        assert!(makefile.contains("help:"));
    }

    #[test]
    fn test_render_makefile_has_targets() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        assert!(makefile.contains("gist:"));
        assert!(makefile.contains("gist-dry:"));
        assert!(makefile.contains("pragma:"));
        assert!(makefile.contains("pragma-dry:"));
    }

    #[test]
    fn test_render_makefile_has_cli_args() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        // Should have conditional variable expansion
        assert!(makefile.contains("$(if $(REPO)"));
        assert!(makefile.contains("--repo"));
    }

    #[test]
    fn test_render_makefile_repeatable_cli_args() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        assert!(
            makefile.contains("$(foreach v,$(EXT),--extensions $(v))"),
            "repeatable vars should expand into repeated flags"
        );
    }

    #[test]
    fn test_render_help_params_repeatable() {
        let params = vec![EntrypointParam {
            port_name: "extensions".to_string(),
            make_var: "EXT".to_string(),
            cli_flag: "--extensions".to_string(),
            type_hint: "String".to_string(),
            default: None,
            repeatable: true,
        }];

        let rendered = render_help_params(&params);
        assert_eq!(rendered, "[EXT=... ...]");
    }

    #[test]
    fn test_render_makefile_has_core_targets() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        assert!(makefile.contains("ensure-codegen: preflight-fix"));
        assert!(makefile.contains("codegen: lint-upsert"));
        assert!(makefile.contains("build: codegen testgen"));
        assert!(makefile.contains("testgen: lint-upsert"));
        assert!(makefile.contains("testgen-check: lint-upsert"));
        assert!(makefile.contains("pragma-check: lint-upsert"));
        assert!(makefile.contains("gunbc-pragma --release -- --mode=verify"));
        assert!(makefile.contains("clean:"));
    }

    #[test]
    fn test_render_makefile_has_meta_targets() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        // Meta targets section
        assert!(makefile.contains("# Meta Targets"));

        // Individual meta targets
        assert!(
            makefile.contains("test: build verify-fix"),
            "test should depend on build and verify-fix (testgen is included in build)"
        );
        assert!(makefile.contains("cargo test"));

        assert!(makefile.contains("check: lint-upsert"));
        assert!(makefile.contains("cargo check --all-targets"));

        assert!(makefile.contains("clippy: lint-upsert"));
        assert!(makefile.contains("cargo clippy --all-targets"));

        assert!(makefile.contains("fmt:"));
        assert!(makefile.contains("@cargo fmt"));
        assert!(makefile.contains("fmt-check:"));
    }

    #[test]
    fn test_render_makefile_help_has_sections() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        assert!(makefile.contains("Build commands:"));
        assert!(makefile.contains("codegen  - Generate CLI entrypoints"));
        assert!(makefile.contains("ensure-codegen  - Bootstrap CLI entrypoints"));
        assert!(makefile.contains("preflight-fix  - Auto-fix rustc warnings"));
        assert!(makefile.contains("lint-upsert  - Auto-fix lint issues"));
        assert!(makefile.contains("Development:"));
        assert!(makefile.contains("Tools:"));
    }

    #[test]
    fn test_render_makefile_has_testgen_targets() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        // Testgen targets in core section
        assert!(makefile.contains("testgen:"), "should have testgen target");
        assert!(
            makefile.contains("testgen-check:"),
            "should have testgen-check target"
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
            makefile.contains("testgen testgen-check pragma-check"),
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

        assert!(
            makefile.contains("RUSTFLAGS=\"-D warnings\" cargo run -p gunbc-gist --bin gunbc-gist")
        );
        assert!(makefile.contains(
            "RUSTFLAGS=\"-D warnings\" cargo run -p gunbc-gist --bin gunbc-gist -- --dry-run"
        ));
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
            "# DO NOT EDIT - regenerate with: cargo run -p gunbc-dag --bin gunbc-makegen --release"
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
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        assert!(makefile.contains("# Naming convention:"));
        assert!(makefile.contains("make <target>      - verify only"));
        assert!(makefile.contains("make <target>-fix  - auto-fix then verify"));
    }

    #[test]
    fn test_render_makefile_has_fix_alias_targets() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        // Should have fmt-fix and lint-fix as dependency targets
        assert!(makefile.contains("fmt-fix:"));
        assert!(makefile.contains("lint-fix: pragma"));
        assert!(makefile.contains("@cargo fmt")); // fmt-fix uses fmt command
        assert!(makefile.contains("--fix")); // lint-fix uses clippy --fix
    }

    #[test]
    fn test_render_makefile_has_fix_variants() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

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
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

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
                raw.starts_with('#')
                    || raw.starts_with('.')
                    || raw.contains("============"),
                "unexpected Raw block that looks like a target definition: {:?}",
                &raw[..raw.len().min(80)]
            );
        }
    }
}
