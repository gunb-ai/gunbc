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
use std::borrow::Cow;

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
                name: Cow::Borrowed("Cargo build artifacts"),
                source: Some(Cow::Borrowed("cargo")),
                items: vec![Cow::Borrowed("/target/"), Cow::Borrowed("/target-codex/")],
                rationale: Some(Cow::Borrowed("Reproducible from source via cargo build")),
            });
            categories.push(Category {
                name: Cow::Borrowed("Cargo cache (local CARGO_HOME)"),
                source: Some(Cow::Borrowed("tool/rust")),
                items: vec![Cow::Borrowed("/.cargo-home/")],
                rationale: Some(Cow::Borrowed(
                    "Local Cargo cache may include git checkouts; not project state",
                )),
            });
            categories.push(Category {
                name: Cow::Borrowed("Codegen bin symlink"),
                source: Some(Cow::Borrowed("codegen")),
                items: vec![Cow::Borrowed("/bin/")],
                rationale: Some(Cow::Borrowed("Symlink to target/release created by codegen")),
            });
        }
        BuildSystem::Buck2 => {
            // Buck2 uses both cargo (for codegen) and buck2 (for build)
            categories.push(Category {
                name: Cow::Borrowed("Cargo build artifacts"),
                source: Some(Cow::Borrowed("cargo")),
                items: vec![Cow::Borrowed("/target/"), Cow::Borrowed("/target-codex/")],
                rationale: Some(Cow::Borrowed("Reproducible from source via cargo build")),
            });
            categories.push(Category {
                name: Cow::Borrowed("Cargo cache (local CARGO_HOME)"),
                source: Some(Cow::Borrowed("tool/rust")),
                items: vec![Cow::Borrowed("/.cargo-home/")],
                rationale: Some(Cow::Borrowed(
                    "Local Cargo cache may include git checkouts; not project state",
                )),
            });
            categories.push(Category {
                name: Cow::Borrowed("Codegen bin symlink"),
                source: Some(Cow::Borrowed("codegen")),
                items: vec![Cow::Borrowed("/bin/")],
                rationale: Some(Cow::Borrowed("Symlink to target/release created by codegen")),
            });
            categories.push(Category {
                name: Cow::Borrowed("Buck2 build artifacts"),
                source: Some(Cow::Borrowed("buck2")),
                items: vec![Cow::Borrowed("/buck-out/")],
                rationale: Some(Cow::Borrowed("Reproducible from source via buck2 build")),
            });
            categories.push(Category {
                name: Cow::Borrowed("Vendored dependencies"),
                source: Some(Cow::Borrowed("buck2")),
                items: vec![
                    Cow::Borrowed("/third-party/rust/vendor/"),
                    Cow::Borrowed("/third-party/rust/.cargo/"),
                ],
                rationale: Some(Cow::Borrowed("Reproducible from lockfile via reindeer vendor")),
            });
        }
    }

    // Coverage is always included (cargo-tarpaulin)
    categories.push(Category {
        name: Cow::Borrowed("Coverage reports"),
        source: Some(Cow::Borrowed("cargo-tarpaulin")),
        items: vec![
            Cow::Borrowed("tarpaulin-report.html"),
            Cow::Borrowed("tarpaulin-report.json"),
            Cow::Borrowed("cobertura.xml"),
            Cow::Borrowed("lcov.info"),
            Cow::Borrowed("coverage/"),
        ],
        rationale: Some(Cow::Borrowed("Generated, often large, reproducible")),
    });

    // Universal categories (always included)
    categories.push(Category {
        name: Cow::Borrowed("Editor/IDE state"),
        source: Some(Cow::Borrowed("editor")),
        items: vec![
            Cow::Borrowed(".idea/"),
            Cow::Borrowed(".vscode/"),
            Cow::Borrowed("*.swp"),
            Cow::Borrowed("*.swo"),
            Cow::Borrowed("*~"),
        ],
        rationale: Some(Cow::Borrowed("Per-developer configuration, not project state")),
    });
    categories.push(Category {
        name: Cow::Borrowed("OS metadata"),
        source: Some(Cow::Borrowed("os")),
        items: vec![Cow::Borrowed(".DS_Store"), Cow::Borrowed("Thumbs.db")],
        rationale: Some(Cow::Borrowed("OS-generated, not project state")),
    });
    categories.push(Category {
        name: Cow::Borrowed("Secrets and local config"),
        source: Some(Cow::Borrowed("secrets")),
        items: vec![
            Cow::Borrowed(".env"),
            Cow::Borrowed(".env.local"),
            Cow::Borrowed(".env.*.local"),
        ],
        rationale: Some(Cow::Borrowed("Environment-specific, may contain secrets")),
    });
    categories.push(Category {
        name: Cow::Borrowed("Generator stamp files"),
        source: Some(Cow::Borrowed("generators")),
        items: vec![Cow::Borrowed(".*-stamp")],
        rationale: Some(Cow::Borrowed(
            "Producer-centric model stamps; regenerated by make targets",
        )),
    });
    categories.push(Category {
        name: Cow::Borrowed("Bootstrap outputs"),
        source: Some(Cow::Borrowed("bootstrap")),
        items: vec![Cow::Borrowed("/Makefile"), Cow::Borrowed("/output")],
        rationale: Some(Cow::Borrowed(
            "Regenerated by gunbc-bootstrap/makegen; not source-of-truth",
        )),
    });
    categories.push(Category {
        name: Cow::Borrowed("Pragma outputs"),
        source: Some(Cow::Borrowed("pragma")),
        items: vec![
            Cow::Borrowed("/clippy.toml"),
            Cow::Borrowed("/tools/disallowed-methods-allowlist.txt"),
            Cow::Borrowed("/tools/pragma-lint-policy.txt"),
        ],
        rationale: Some(Cow::Borrowed(
            "Regenerated by gunbc-pragma; not source-of-truth",
        )),
    });
    categories.push(Category {
        name: Cow::Borrowed("Generated test files"),
        source: Some(Cow::Borrowed("testgen")),
        items: vec![Cow::Borrowed("**/generated_tests*.rs")],
        rationale: Some(Cow::Borrowed(
            "Regenerated by make testgen; staleness checked by make test",
        )),
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
            generator_name: Cow::Borrowed("gunbc-bootstrap"),
            regenerate_command: Cow::Borrowed("cargo run -p gunbc-dag --bin gunbc-bootstrap --release"),
            comment_prefix: Cow::Borrowed("#"),
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
        assert!(content.contains("/clippy.toml"));
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
