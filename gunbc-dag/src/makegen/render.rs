//! Makefile rendering.
//!
//! Uses `BuildConfig` as the single source of truth for all build commands.
//! Uses `FileHeader` for standardized header generation.
//!
//! Uses `MAKEFILE.indent` from the language module for tab indentation.
//! Uses `MakefileStructuredRenderer` to render `StructuredBlock` IR.

use crate::makegen::registry::{
    BuildConfig, EntrypointParam, ExtraTarget, MetaTarget, ToolInfo, ToolRegistry,
};
use gunbc_ir::cargo::Warnings;
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
            regenerate_command: "make makegen".to_string(),
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
         # Dev workflow:     make test-fix  (fix everything, then test)\n\
         # CI verification:  make test      (verify fmt + lint + test)\n\n"
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
        ".PHONY: help ensure-codegen codegen build clean testgen testgen-check pragma-check verify fmt-fix lint-fix",
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

    // ensure-codegen
    blocks.push(StructuredBlock::Raw(
        "# Ensure CLI entrypoints exist (bootstrap-safe)\n".to_string(),
    ));
    blocks.push(StructuredBlock::Target(Target {
        name: "ensure-codegen".to_string(),
        deps: vec![],
        body: vec![config.ensure_codegen_shell()],
    }));

    // codegen
    blocks.push(StructuredBlock::Raw(
        "# Generate CLI entrypoints (DAG upsert)\n".to_string(),
    ));
    blocks.push(StructuredBlock::Target(Target {
        name: "codegen".to_string(),
        deps: vec!["ensure-codegen".to_string()],
        body: vec![config.codegen_shell()],
    }));

    // build
    let build_desc = if config.use_dag_entrypoints {
        "codegen \u{2192} testgen \u{2192} gunbc-build"
    } else {
        "codegen \u{2192} testgen \u{2192} cargo build"
    };
    blocks.push(StructuredBlock::Raw(format!(
        "# Full build transaction: {build_desc}\n"
    )));
    blocks.push(StructuredBlock::Target(Target {
        name: "build".to_string(),
        deps: vec!["codegen".to_string(), "testgen".to_string()],
        body: vec![config.build_shell()],
    }));

    // clean
    blocks.push(StructuredBlock::Raw(
        "# Clean build artifacts\n".to_string(),
    ));
    blocks.push(StructuredBlock::Target(Target {
        name: "clean".to_string(),
        deps: vec![],
        body: vec!["@cargo clean".to_string()],
    }));

    // testgen
    blocks.push(StructuredBlock::Raw(
        "# Regenerate tests from DAG structures and MockSpecs\n".to_string(),
    ));
    blocks.push(StructuredBlock::Target(Target {
        name: "testgen".to_string(),
        deps: vec!["ensure-codegen".to_string()],
        body: vec![config.testgen_shell()],
    }));

    // testgen-check
    blocks.push(StructuredBlock::Raw(
        "# Check if generated tests are stale (fails if regeneration needed)\n".to_string(),
    ));
    blocks.push(StructuredBlock::Target(Target {
        name: "testgen-check".to_string(),
        deps: vec!["ensure-codegen".to_string()],
        body: vec![config.testgen_check_shell()],
    }));

    // pragma-check
    blocks.push(StructuredBlock::Raw(
        "# Check if pragma artifacts are stale (fails if regeneration needed)\n".to_string(),
    ));
    blocks.push(StructuredBlock::Target(Target {
        name: "pragma-check".to_string(),
        deps: vec!["ensure-codegen".to_string()],
        body: vec![format!(
            "@{}cargo run -p gunbc-dag --bin gunbc-pragma --release -- --check",
            warning_prefix
        )],
    }));

    // verify
    blocks.push(StructuredBlock::Raw(
        "# Verify generated artifacts match their generators\n".to_string(),
    ));
    blocks.push(StructuredBlock::Target(Target {
        name: "verify".to_string(),
        deps: vec!["ensure-codegen".to_string()],
        body: vec![
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-makegen --release -- --check",
                warning_prefix
            ),
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-bootstrap --release -- --check",
                warning_prefix
            ),
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-testgen --release -- --check",
                warning_prefix
            ),
            format!(
                "@{}cargo run -p gunbc-dag --bin gunbc-pragma --release -- --check",
                warning_prefix
            ),
        ],
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
    lines.push("@echo \"  clean    - Remove build artifacts\"".to_string());
    lines.push("@echo \"  testgen  - Regenerate tests from DAG structures\"".to_string());
    lines.push("@echo \"  testgen-check  - Check if generated tests are stale\"".to_string());
    lines.push(
        "@echo \"  pragma-check  - Check if pragma artifacts are stale\"".to_string(),
    );
    lines.push(
        "@echo \"  verify   - Verify generated artifacts match their generators\"".to_string(),
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
            let deps = if meta.fix_deps.is_empty() {
                "auto-fix".to_string()
            } else {
                format!("{} first", meta.fix_deps.join(" + "))
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
    })
}

/// Build meta targets section as structured blocks.
fn build_meta_targets(registry: &ToolRegistry, config: &BuildConfig) -> Vec<StructuredBlock> {
    let mut blocks = Vec::new();

    blocks.push(StructuredBlock::Raw(
        "# ============================================================================\n\
         # Meta Targets - Development workflow commands\n\
         # ============================================================================\n\n"
            .to_string(),
    ));

    // Fix alias targets
    blocks.extend(build_fix_alias_targets(config));

    for meta in &registry.meta_targets {
        blocks.push(build_meta_target(meta, config));
        if meta.has_fix_variant {
            blocks.extend(build_meta_fix_variant(meta, config));
        }
        if meta.has_check_variant {
            blocks.extend(build_meta_check_variant(meta, config));
        }
    }

    blocks
}

/// Build the dependency list for a meta target.
fn meta_target_deps(meta: &MetaTarget, config: &BuildConfig) -> Vec<String> {
    let mut deps: Vec<String> = Vec::new();

    if let Some(prep) = meta.prep_level.dep_name(config.use_dag_entrypoints) {
        deps.push(prep.to_string());
    }

    for dep in &meta.extra_deps {
        deps.push((*dep).to_string());
    }

    deps
}

/// Build fix alias targets.
fn build_fix_alias_targets(config: &BuildConfig) -> Vec<StructuredBlock> {
    vec![
        StructuredBlock::Raw("# fmt-fix: apply formatting (alias for fmt)\n".to_string()),
        StructuredBlock::Target(Target {
            name: "fmt-fix".to_string(),
            deps: vec![],
            body: vec![config.fmt_shell()],
        }),
        StructuredBlock::Raw(
            "# lint-fix: auto-fix lint issues where possible\n".to_string(),
        ),
        StructuredBlock::Target(Target {
            name: "lint-fix".to_string(),
            deps: vec!["pragma".to_string()],
            body: vec![config.lint_fix_shell()],
        }),
    ]
}

/// Build a single meta target.
fn build_meta_target(meta: &MetaTarget, config: &BuildConfig) -> StructuredBlock {
    let deps = meta_target_deps(meta, config);
    let command = meta.get_command(config);

    StructuredBlock::Raw(format!(
        "# {}: {}\n{}:{}\n\t{}\n\n",
        meta.name,
        meta.description,
        meta.name,
        if deps.is_empty() {
            String::new()
        } else {
            format!(" {}", deps.join(" "))
        },
        command
    ))
}

/// Build a check variant for a meta target.
fn build_meta_check_variant(
    meta: &MetaTarget,
    config: &BuildConfig,
) -> Vec<StructuredBlock> {
    if let Some(check_cmd) = meta.get_check_command(config) {
        let deps = meta_target_deps(meta, config);

        vec![StructuredBlock::Raw(format!(
            "{}-check:{}\n\t{}\n\n",
            meta.name,
            if deps.is_empty() {
                String::new()
            } else {
                format!(" {}", deps.join(" "))
            },
            check_cmd
        ))]
    } else {
        vec![]
    }
}

/// Build a fix variant for a meta target.
fn build_meta_fix_variant(
    meta: &MetaTarget,
    config: &BuildConfig,
) -> Vec<StructuredBlock> {
    if !meta.has_fix_variant {
        return vec![];
    }

    let mut deps = Vec::new();

    for dep in meta.get_fix_deps() {
        deps.push(dep.to_string());
    }

    if let Some(prep) = meta.prep_level.dep_name(config.use_dag_entrypoints) {
        deps.push(prep.to_string());
    }

    for dep in &meta.extra_deps {
        let fix_dep = if dep.ends_with("-check") {
            dep.trim_end_matches("-check").to_string()
        } else {
            dep.to_string()
        };
        deps.push(fix_dep);
    }

    let fix_cmd = meta
        .get_fix_command(config)
        .unwrap_or_else(|| meta.get_command(config));

    vec![StructuredBlock::Raw(format!(
        "# {}-fix: auto-fix then verify\n{}-fix:{}\n\t{}\n\n",
        meta.name,
        meta.name,
        if deps.is_empty() {
            String::new()
        } else {
            format!(" {}", deps.join(" "))
        },
        fix_cmd
    ))]
}

/// Render help text for parameters.
fn render_help_params(params: &[EntrypointParam]) -> String {
    params
        .iter()
        .map(|p| {
            if let Some(ref default) = p.default {
                format!("[{}={}]", p.make_var, default)
            } else {
                format!("[{}=...]", p.make_var)
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

    // Comment + Target as raw (comment is separate from the target name line)
    StructuredBlock::Raw(format!(
        "# {} entrypoints: {}\n{}: ensure-codegen\n\t@{}{} --{}\n\n",
        tool.binary_name(),
        port_list,
        tool.short_name,
        warning_prefix,
        tool.invocation.command(),
        cli_args
    ))
}

/// Build a dry-run target.
fn build_dry_run_target(tool: &ToolInfo, config: &BuildConfig) -> StructuredBlock {
    let warning_prefix = if config.warnings == Warnings::Deny {
        "RUSTFLAGS=\"-D warnings\" "
    } else {
        ""
    };

    let cli_args = render_cli_args(&tool.entrypoints);

    StructuredBlock::Raw(format!(
        "{}-dry: ensure-codegen\n\t@{}{} -- --dry-run{}\n\n",
        tool.short_name, warning_prefix, tool.invocation.command(), cli_args
    ))
}

/// Build an extra composite target.
fn build_extra_target(tool: &ToolInfo, extra: &ExtraTarget) -> StructuredBlock {
    let mut body = Vec::new();
    for cmd in &extra.post_commands {
        body.push(cmd.clone());
    }

    StructuredBlock::Raw(format!(
        "# {}-{}: {}\n{}-{}: {}\n{}\n",
        tool.short_name,
        extra.suffix,
        extra.description,
        tool.short_name,
        extra.suffix,
        tool.short_name,
        body.iter()
            .map(|cmd| format!("\t{}", cmd))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

/// Render CLI arguments from entrypoint parameters.
fn render_cli_args(params: &[EntrypointParam]) -> String {
    if params.is_empty() {
        return String::new();
    }

    let args: Vec<String> = params
        .iter()
        .map(|p| {
            // $(if $(VAR),--flag $(VAR))
            format!(" $(if $({}),{} $({}))", p.make_var, p.cli_flag, p.make_var)
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
    fn test_render_makefile_has_core_targets() {
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        assert!(makefile.contains("ensure-codegen:"));
        assert!(makefile.contains("codegen: ensure-codegen"));
        assert!(makefile.contains("build: codegen testgen"));
        assert!(makefile.contains("testgen: ensure-codegen"));
        assert!(makefile.contains("testgen-check: ensure-codegen"));
        assert!(makefile.contains("pragma-check: ensure-codegen"));
        assert!(makefile.contains("gunbc-pragma --release -- --check"));
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
            makefile.contains("test: build testgen verify"),
            "test should depend on build, testgen, and verify"
        );
        assert!(makefile.contains("cargo test"));

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
        let registry = ToolRegistry::default_registry();
        let makefile = render_makefile(&registry);

        assert!(makefile.contains("Build commands:"));
        assert!(makefile.contains("codegen  - Generate CLI entrypoints"));
        assert!(makefile.contains("ensure-codegen  - Bootstrap CLI entrypoints"));
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
        assert!(output.contains("# DO NOT EDIT - regenerate with: make makegen"));

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
}
