//! Gitignore generation from repo layout.
//!
//! This module derives .gitignore patterns from the repo's `BuildConfig`,
//! following the pattern from the-gunbai:
//! - `crates/gunbai-integrations-contracts/src/understanding/gitignore.rs`
//! - `.gitignore` (section comments with provenance)
//!
//! Each category has:
//! - `name`: Human-readable category name
//! - `source`: Which tool/concept generates these patterns
//! - `items`: The actual .gitignore patterns
//! - `rationale`: Why these files are ignored

use std::collections::HashSet;

use crate::dsl_registry::discover_tool_defs_from_dsl;
use crate::makegen::registry::{BuildConfig, BuildSystem};
use crate::WorkspaceBinary;
use gunbc_infra::workspace_model::{baseline_commit_policies, CommitReason};
use gunbc_ir::cargo::{CargoCommand, Subcommand};
use gunbc_ir::render_ir::{Category, FileHeader, PlainText, StructuredRenderer};
use gunbc_ir::symbols::{Tier, STANDARD};
use gunbc_ir::MakefileStructuredRenderer;

// ============================================================================
// Derive Categories from BuildConfig
// ============================================================================

/// Derive ignore categories from BuildConfig.
///
/// Categories are derived from `build_system` to ensure the gitignore
/// matches what's actually in the repo.
pub fn derive_categories(config: &BuildConfig) -> Vec<Category> {
    let mut categories = Vec::new();

    // From build_system - determines which build artifacts to ignore
    match config.build_system {
        BuildSystem::Cargo => {
            categories.push(Category {
                name: "Cargo build artifacts".into(),
                source: Some("cargo".into()),
                items: vec!["/target/".into(), "/target-codex/".into()],
                rationale: Some("Reproducible from source via cargo build".into()),
            });
            categories.push(Category {
                name: "Cargo cache (local CARGO_HOME)".into(),
                source: Some("tool/rust".into()),
                items: vec!["/.cargo-home/".into()],
                rationale: Some(
                    "Local Cargo cache may include git checkouts; not project state".into(),
                ),
            });
            categories.push(Category {
                name: "Codegen bin symlink".into(),
                source: Some("codegen".into()),
                items: vec!["/bin/".into()],
                rationale: Some("Symlink to target/release created by codegen".into()),
            });
        }
        BuildSystem::Buck2 => {
            // Buck2 uses both cargo (for codegen) and buck2 (for build)
            categories.push(Category {
                name: "Cargo build artifacts".into(),
                source: Some("cargo".into()),
                items: vec!["/target/".into(), "/target-codex/".into()],
                rationale: Some("Reproducible from source via cargo build".into()),
            });
            categories.push(Category {
                name: "Cargo cache (local CARGO_HOME)".into(),
                source: Some("tool/rust".into()),
                items: vec!["/.cargo-home/".into()],
                rationale: Some(
                    "Local Cargo cache may include git checkouts; not project state".into(),
                ),
            });
            categories.push(Category {
                name: "Codegen bin symlink".into(),
                source: Some("codegen".into()),
                items: vec!["/bin/".into()],
                rationale: Some("Symlink to target/release created by codegen".into()),
            });
            categories.push(Category {
                name: "Buck2 build artifacts".into(),
                source: Some("buck2".into()),
                items: vec!["/buck-out/".into()],
                rationale: Some("Reproducible from source via buck2 build".into()),
            });
            categories.push(Category {
                name: "Vendored dependencies".into(),
                source: Some("buck2".into()),
                items: vec![
                    "/third-party/rust/vendor/".into(),
                    "/third-party/rust/.cargo/".into(),
                ],
                rationale: Some("Reproducible from lockfile via reindeer vendor".into()),
            });
        }
    }

    // Coverage is always included (cargo-tarpaulin)
    categories.push(Category {
        name: "Coverage reports".into(),
        source: Some("cargo-tarpaulin".into()),
        items: vec![
            "tarpaulin-report.html".into(),
            "tarpaulin-report.json".into(),
            "cobertura.xml".into(),
            "lcov.info".into(),
            "coverage/".into(),
        ],
        rationale: Some("Generated, often large, reproducible".into()),
    });

    // Universal categories (always included)
    categories.push(Category {
        name: "Editor/IDE state".into(),
        source: Some("editor".into()),
        items: vec![
            ".idea/".into(),
            ".vscode/".into(),
            "*.swp".into(),
            "*.swo".into(),
            "*~".into(),
        ],
        rationale: Some("Per-developer configuration, not project state".into()),
    });
    categories.push(Category {
        name: "OS metadata".into(),
        source: Some("os".into()),
        items: vec![".DS_Store".into(), "Thumbs.db".into()],
        rationale: Some("OS-generated, not project state".into()),
    });
    categories.push(Category {
        name: "Secrets and local config".into(),
        source: Some("secrets".into()),
        items: vec![".env".into(), ".env.local".into(), ".env.*.local".into()],
        rationale: Some("Environment-specific, may contain secrets".into()),
    });
    categories.push(Category {
        name: "Generator stamp files".into(),
        source: Some("generators".into()),
        items: vec![".*-stamp".into()],
        rationale: Some("Producer-centric model stamps; regenerated by make targets".into()),
    });
    // Tool outputs — auto-derived from DSL entrypoint inference registry.
    // Bootstrap seed files are filtered out: they are generated but committed.
    let seed_patterns: HashSet<&str> = baseline_commit_policies()
        .iter()
        .filter(|p| p.reason == CommitReason::BootstrapSeed)
        .map(|p| p.pattern)
        .collect();

    for tool in discover_tool_defs_from_dsl() {
        if !tool.outputs.is_empty() {
            let items: Vec<_> = tool
                .outputs
                .iter()
                .filter(|p| !seed_patterns.contains(p.as_str()))
                .map(|p| {
                    if p.contains("**") || p.starts_with('*') {
                        p.to_string().into()
                    } else {
                        format!("/{p}").into()
                    }
                })
                .collect();
            if !items.is_empty() {
                categories.push(Category {
                    name: format!("{} outputs", tool.meta.tool_name).into(),
                    source: Some(tool.meta.tool_name.to_string().into()),
                    items,
                    rationale: Some(
                        format!("Generated by {}; not source-of-truth", tool.meta.tool_name).into(),
                    ),
                });
            }
        }
    }

    categories.push(Category {
        name: "Workflow runtime state".into(),
        source: Some("workflow".into()),
        items: vec!["/.gunbc/".into()],
        rationale: Some("Local execution ledger and config; not source-of-truth".into()),
    });

    categories
}

// ============================================================================
// GitignoreRenderer
// ============================================================================

/// Renderer for .gitignore files.
///
/// Produces a .gitignore with section comments showing provenance,
/// following the pattern from the-gunbai.
pub struct GitignoreRenderer<'a> {
    /// Categories to render
    pub categories: Vec<Category>,
    /// BuildConfig reference (for metadata)
    pub config: &'a BuildConfig,
}

impl<'a> GitignoreRenderer<'a> {
    /// Create a renderer from BuildConfig.
    pub fn from_config(config: &'a BuildConfig) -> Self {
        Self {
            categories: derive_categories(config),
            config,
        }
    }

    /// Create with custom categories.
    pub fn with_categories(config: &'a BuildConfig, categories: Vec<Category>) -> Self {
        Self { categories, config }
    }

    /// Render the complete .gitignore with header.
    pub fn render(&self) -> String {
        let regenerate_cmd =
            CargoCommand::new(Subcommand::Run(WorkspaceBinary::Bootstrap.invocation()));
        let header = FileHeader {
            generator_name: "gunbc-bootstrap".into(),
            regenerate_command: regenerate_cmd.to_shell().into(),
            comment_prefix: "#".into(),
        };
        format!("{}\n\n{}", header.render(), self.render_content())
    }

    /// Render just the content without header.
    pub fn render_content(&self) -> String {
        let renderer = MakefileStructuredRenderer::new(PlainText {
            tier: Tier::Ascii,
            symbol_set: &STANDARD,
        });

        let mut output = String::new();

        // Rule comment at top
        output.push_str(
            "# Rule: Ignore anything that is (a) reproducible from committed source-of-truth\n",
        );
        output
            .push_str("# files, or (b) environment-specific and not shared across developers.\n\n");

        // Render each category with provenance
        for category in &self.categories {
            output.push_str(&renderer.render_category(category));
        }

        output
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Render a .gitignore from BuildConfig.
pub fn render_gitignore(config: &BuildConfig) -> String {
    GitignoreRenderer::from_config(config).render()
}

/// Render .gitignore content only (without header).
pub fn render_gitignore_content(config: &BuildConfig) -> String {
    GitignoreRenderer::from_config(config).render_content()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_categories_cargo() {
        let config = BuildConfig::cargo();
        let categories = derive_categories(&config);

        // Should have cargo artifacts
        assert!(categories
            .iter()
            .any(|c| c.source.as_deref() == Some("cargo")));
        // Should have universal categories
        assert!(categories
            .iter()
            .any(|c| c.source.as_deref() == Some("editor")));
        assert!(categories.iter().any(|c| c.source.as_deref() == Some("os")));
        assert!(categories
            .iter()
            .any(|c| c.source.as_deref() == Some("secrets")));
    }

    #[test]
    fn test_derive_categories_buck2() {
        let config = BuildConfig::buck2();
        let categories = derive_categories(&config);

        // Should have both cargo and buck2
        assert!(categories
            .iter()
            .any(|c| c.source.as_deref() == Some("cargo")));
        assert!(categories
            .iter()
            .any(|c| c.source.as_deref() == Some("buck2")));
    }

    #[test]
    fn test_render_gitignore_has_header() {
        let config = BuildConfig::cargo();
        let content = render_gitignore(&config);

        assert!(content.contains("Generated by gunbc-bootstrap"));
        assert!(content.contains("DO NOT EDIT"));
    }

    #[test]
    fn test_render_gitignore_has_sections() {
        let config = BuildConfig::cargo();
        let content = render_gitignore(&config);

        // Should have section headers with provenance
        assert!(content.contains("# --- Cargo build artifacts (from cargo) ---"));
        assert!(content.contains("# --- Editor/IDE state (from editor) ---"));
    }

    #[test]
    fn test_render_gitignore_has_patterns() {
        let config = BuildConfig::cargo();
        let content = render_gitignore(&config);

        // Should have actual patterns
        assert!(content.contains("/target/"));
        assert!(content.contains("/Makefile"));
        assert!(content.contains(".DS_Store"));
        assert!(content.contains(".env"));
    }

    #[test]
    fn test_render_gitignore_has_rationale() {
        let config = BuildConfig::cargo();
        let content = render_gitignore(&config);

        // Should have rationale comments
        assert!(content.contains("Reproducible from source"));
        assert!(content.contains("Per-developer configuration"));
    }

    #[test]
    fn seed_files_excluded_from_rendered_gitignore() {
        let config = BuildConfig::cargo();
        let content = render_gitignore(&config);

        // Bootstrap seed files are generated but committed — they must NOT
        // appear as gitignore patterns in the tool output sections.
        assert!(
            !content.contains("/.gitignore"),
            ".gitignore is a bootstrap seed and should not be gitignored"
        );
        assert!(
            !content.contains("/clippy.toml"),
            "clippy.toml is a bootstrap seed and should not be gitignored"
        );
        assert!(
            !content.contains("/deps.toml"),
            "deps.toml is a bootstrap seed and should not be gitignored"
        );
    }
}
