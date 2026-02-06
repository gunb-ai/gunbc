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

use crate::makegen::registry::{BuildConfig, BuildSystem};
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
                name: "Cargo build artifacts".to_string(),
                source: Some("cargo".to_string()),
                items: vec!["/target/".to_string(), "/target-codex/".to_string()],
                rationale: Some("Reproducible from source via cargo build".to_string()),
            });
            categories.push(Category {
                name: "Cargo cache (local CARGO_HOME)".to_string(),
                source: Some("tool/rust".to_string()),
                items: vec!["/.cargo-home/".to_string()],
                rationale: Some(
                    "Local Cargo cache may include git checkouts; not project state".to_string(),
                ),
            });
            categories.push(Category {
                name: "Codegen bin symlink".to_string(),
                source: Some("codegen".to_string()),
                items: vec!["/bin/".to_string()],
                rationale: Some("Symlink to target/release created by codegen".to_string()),
            });
        }
        BuildSystem::Buck2 => {
            // Buck2 uses both cargo (for codegen) and buck2 (for build)
            categories.push(Category {
                name: "Cargo build artifacts".to_string(),
                source: Some("cargo".to_string()),
                items: vec!["/target/".to_string(), "/target-codex/".to_string()],
                rationale: Some("Reproducible from source via cargo build".to_string()),
            });
            categories.push(Category {
                name: "Cargo cache (local CARGO_HOME)".to_string(),
                source: Some("tool/rust".to_string()),
                items: vec!["/.cargo-home/".to_string()],
                rationale: Some(
                    "Local Cargo cache may include git checkouts; not project state".to_string(),
                ),
            });
            categories.push(Category {
                name: "Codegen bin symlink".to_string(),
                source: Some("codegen".to_string()),
                items: vec!["/bin/".to_string()],
                rationale: Some("Symlink to target/release created by codegen".to_string()),
            });
            categories.push(Category {
                name: "Buck2 build artifacts".to_string(),
                source: Some("buck2".to_string()),
                items: vec!["/buck-out/".to_string()],
                rationale: Some("Reproducible from source via buck2 build".to_string()),
            });
            categories.push(Category {
                name: "Vendored dependencies".to_string(),
                source: Some("buck2".to_string()),
                items: vec![
                    "/third-party/rust/vendor/".to_string(),
                    "/third-party/rust/.cargo/".to_string(),
                ],
                rationale: Some("Reproducible from lockfile via reindeer vendor".to_string()),
            });
        }
    }

    // Coverage is always included (cargo-tarpaulin)
    categories.push(Category {
        name: "Coverage reports".to_string(),
        source: Some("cargo-tarpaulin".to_string()),
        items: vec![
            "tarpaulin-report.html".to_string(),
            "tarpaulin-report.json".to_string(),
            "cobertura.xml".to_string(),
            "lcov.info".to_string(),
            "coverage/".to_string(),
        ],
        rationale: Some("Generated, often large, reproducible".to_string()),
    });

    // Universal categories (always included)
    categories.push(Category {
        name: "Editor/IDE state".to_string(),
        source: Some("editor".to_string()),
        items: vec![
            ".idea/".to_string(),
            ".vscode/".to_string(),
            "*.swp".to_string(),
            "*.swo".to_string(),
            "*~".to_string(),
        ],
        rationale: Some("Per-developer configuration, not project state".to_string()),
    });
    categories.push(Category {
        name: "OS metadata".to_string(),
        source: Some("os".to_string()),
        items: vec![".DS_Store".to_string(), "Thumbs.db".to_string()],
        rationale: Some("OS-generated, not project state".to_string()),
    });
    categories.push(Category {
        name: "Secrets and local config".to_string(),
        source: Some("secrets".to_string()),
        items: vec![
            ".env".to_string(),
            ".env.local".to_string(),
            ".env.*.local".to_string(),
        ],
        rationale: Some("Environment-specific, may contain secrets".to_string()),
    });
    categories.push(Category {
        name: "Generator stamp files".to_string(),
        source: Some("generators".to_string()),
        items: vec![".*-stamp".to_string()],
        rationale: Some("Producer-centric model stamps; regenerated by make targets".to_string()),
    });
    categories.push(Category {
        name: "Generated test files".to_string(),
        source: Some("testgen".to_string()),
        items: vec!["**/generated_tests*.rs".to_string()],
        rationale: Some(
            "Regenerated by make testgen; staleness checked by make test".to_string(),
        ),
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
        let header = FileHeader {
            generator_name: "gunbc-bootstrap".to_string(),
            regenerate_command: "make bootstrap".to_string(),
            comment_prefix: "#".to_string(),
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
        assert!(categories
            .iter()
            .any(|c| c.source.as_deref() == Some("os")));
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
}
