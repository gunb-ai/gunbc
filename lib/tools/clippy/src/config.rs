//! Clippy configuration modeling.
//!
//! This module provides structured configuration for clippy.toml generation.
//! The configuration is defined as Rust code (source of truth) and rendered
//! to TOML format for clippy to consume.
//!
//! # Pattern
//!
//! ```text
//! ClippyConfig (Rust)  -->  generate_clippy_toml()  -->  clippy.toml
//! ```
//!
//! # Usage
//!
//! ```text
//! use gunbc_clippy::config::{ClippyConfig, generate_clippy_toml};
//!
//! // Use a preset configuration
//! let config = ClippyConfig::transport_pattern();
//!
//! // Generate the TOML
//! let toml = generate_clippy_toml(&config);
//! ```

use gunbc_ir::render_ir::{FileHeader, PlainText, StructuredBlock, StructuredRenderer};
use gunbc_ir::symbols::{Tier, STANDARD};
use gunbc_ir::PlainStructuredRenderer;
use std::borrow::Cow;
use std::fmt::Write;

// ============================================================================
// Configuration Structs
// ============================================================================

/// A clippy disallowed method rule.
///
/// Maps to entries in the `disallowed-methods` array in clippy.toml.
#[derive(Debug, Clone)]
pub struct DisallowedMethod {
    /// Full path to the method (e.g., "std::fs::read")
    pub path: &'static str,
    /// Human-readable reason why this method is disallowed
    pub reason: &'static str,
}

impl DisallowedMethod {
    /// Create a new disallowed method rule.
    pub const fn new(path: &'static str, reason: &'static str) -> Self {
        Self { path, reason }
    }
}

/// A clippy disallowed type rule.
///
/// Maps to entries in the `disallowed-types` array in clippy.toml.
#[derive(Debug, Clone)]
pub struct DisallowedType {
    /// Full path to the type (e.g., "std::fs::File")
    pub path: &'static str,
    /// Human-readable reason why this type is disallowed
    pub reason: &'static str,
}

impl DisallowedType {
    /// Create a new disallowed type rule.
    pub const fn new(path: &'static str, reason: &'static str) -> Self {
        Self { path, reason }
    }
}

/// A crate-level allowance for bypassing disallowed methods.
///
/// Documents which crates have `#![allow(clippy::disallowed_methods)]`
/// and why they are permitted to bypass the rules.
#[derive(Debug, Clone)]
pub struct CrateAllowance {
    /// Crate name (e.g., "gunbc-lib-transport")
    pub crate_name: &'static str,
    /// Reason why this crate is allowed to bypass rules
    pub reason: &'static str,
}

impl CrateAllowance {
    /// Create a new crate allowance.
    pub const fn new(crate_name: &'static str, reason: &'static str) -> Self {
        Self { crate_name, reason }
    }
}

/// Clippy configuration.
///
/// This struct models what CAN be configured in clippy.toml.
/// Use preset functions like `transport_pattern()` for common configurations.
#[derive(Debug, Clone, Default)]
pub struct ClippyConfig {
    /// Methods that are disallowed (unless crate has an allowance).
    pub disallowed_methods: Vec<DisallowedMethod>,
    /// Types that are disallowed (unless crate has an allowance).
    pub disallowed_types: Vec<DisallowedType>,
    /// Crates that are allowed to bypass disallowed methods.
    pub crate_allowances: Vec<CrateAllowance>,
    /// Threshold for large error types (in bytes).
    /// Types larger than this in Result will trigger a warning.
    pub large_error_threshold: Option<u32>,
}

impl ClippyConfig {
    /// Create an empty configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a disallowed method.
    pub fn disallow(mut self, path: &'static str, reason: &'static str) -> Self {
        self.disallowed_methods
            .push(DisallowedMethod::new(path, reason));
        self
    }

    /// Add a disallowed type.
    pub fn disallow_type(mut self, path: &'static str, reason: &'static str) -> Self {
        self.disallowed_types
            .push(DisallowedType::new(path, reason));
        self
    }

    /// Add a crate allowance.
    pub fn allow_crate(mut self, crate_name: &'static str, reason: &'static str) -> Self {
        self.crate_allowances
            .push(CrateAllowance::new(crate_name, reason));
        self
    }

    /// Set the large error threshold.
    pub fn with_large_error_threshold(mut self, threshold: u32) -> Self {
        self.large_error_threshold = Some(threshold);
        self
    }

    // ========================================================================
    // Preset Configurations
    // ========================================================================

    /// Transport pattern enforcement configuration.
    ///
    /// This configuration enforces system invariants:
    ///
    /// ## I6. No Escape Hatches
    /// Direct I/O operations are disallowed. You cannot bypass the transport layer.
    /// If I/O must go through transport, there's no function to call to skip it.
    ///
    /// ## I7. No Fallbacks  
    /// Operations either succeed or fail. No silent degradation.
    /// Don't use `.unwrap_or_default()` to hide errors.
    ///
    /// ## I8. No Warnings
    /// All lints run with `-D warnings`. Warnings are errors.
    /// If something is wrong, the build fails — it doesn't print a warning and continue.
    ///
    /// ## Approved Exceptions (must have documented reason):
    /// - `gunbc-lib-transport` — IS the I/O boundary (the one place I/O is allowed)
    pub fn transport_pattern() -> Self {
        Self::transport_pattern_base()
            // Document approved crates (minimal exceptions)
            // Crate names follow the {PREFIX}-{component} pattern (see cargo::name)
            .allow_crate(
                "gunbc-lib-transport",
                "IS the I/O boundary - the designated place for I/O",
            )
    }

    /// Transport pattern without any crate allowances.
    fn transport_pattern_base() -> Self {
        Self::new()
            .with_large_error_threshold(256)
            // Filesystem operations - enforces I6 (no escape hatches)
            .disallow(
                "std::fs::read",
                "I6: No escape hatches. Use PrepareFileReadOp + TransportOps::Execute",
            )
            .disallow(
                "std::fs::read_to_string",
                "I6: No escape hatches. Use PrepareFileReadOp + TransportOps::Execute",
            )
            .disallow(
                "std::fs::write",
                "I6: No escape hatches. Use PrepareFileWriteOp + TransportOps::Execute",
            )
            .disallow(
                "std::fs::read_dir",
                "I6: No escape hatches. Use PrepareDirectoryListOp + TransportOps::Execute",
            )
            .disallow(
                "std::fs::read_link",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::canonicalize",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::metadata",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::symlink_metadata",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::copy",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::rename",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::hard_link",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::create_dir",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::remove_file",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::remove_dir",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::remove_dir_all",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::create_dir_all",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::set_permissions",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::File::open",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::File::create",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::File::options",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::OpenOptions::new",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::OpenOptions::open",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::DirBuilder::new",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            .disallow(
                "std::fs::DirBuilder::create",
                "I6: No escape hatches. Direct filesystem ops must be in transport layer",
            )
            // Process execution - enforces I6
            .disallow(
                "std::process::Command::new",
                "I6: No escape hatches. Use env nodes + tool handles. Command::new only in transport executor/cli.",
            )
            // HTTP clients - enforce transport-only network I/O
            .disallow(
                "ureq::request",
                "I6: No escape hatches. Network I/O must be in transport layer",
            )
            .disallow(
                "ureq::get",
                "I6: No escape hatches. Network I/O must be in transport layer",
            )
            .disallow(
                "ureq::post",
                "I6: No escape hatches. Network I/O must be in transport layer",
            )
            .disallow(
                "reqwest::get",
                "I6: No escape hatches. Network I/O must be in transport layer",
            )
            .disallow(
                "reqwest::blocking::get",
                "I6: No escape hatches. Network I/O must be in transport layer",
            )
            .disallow(
                "reqwest::Client::new",
                "I6: No escape hatches. Network I/O must be in transport layer",
            )
            .disallow(
                "reqwest::blocking::Client::new",
                "I6: No escape hatches. Network I/O must be in transport layer",
            )
            // git2 - enforce transport-only git I/O
            .disallow(
                "git2::Repository::open",
                "I6: No escape hatches. Git I/O must be in transport layer",
            )
            .disallow(
                "git2::Repository::open_bare",
                "I6: No escape hatches. Git I/O must be in transport layer",
            )
            .disallow(
                "git2::Repository::discover",
                "I6: No escape hatches. Git I/O must be in transport layer",
            )
            .disallow(
                "git2::Repository::init",
                "I6: No escape hatches. Git I/O must be in transport layer",
            )
            // Secret plaintext extraction aliases - enforce approved transport-boundary naming
            .disallow(
                "gunbc_ir::value::SecretString::expose_plaintext_for_transport",
                "M7: Use expose_plaintext_for_transport at approved transport boundaries only",
            )
            .disallow(
                "gunbc_ir::transport::credential::Secret::expose_plaintext_for_transport",
                "M7: Use expose_plaintext_for_transport at approved transport boundaries only",
            )
            // Filesystem types - disallow owning raw file handles outside transport
            .disallow_type(
                "std::fs::File",
                "I6: No escape hatches. Direct filesystem types must be in transport layer",
            )
            .disallow_type(
                "std::fs::OpenOptions",
                "I6: No escape hatches. Direct filesystem types must be in transport layer",
            )
            .disallow_type(
                "std::fs::DirBuilder",
                "I6: No escape hatches. Direct filesystem types must be in transport layer",
            )
            .disallow_type(
                "std::fs::DirEntry",
                "I6: No escape hatches. Direct filesystem types must be in transport layer",
            )
            .disallow_type(
                "std::fs::ReadDir",
                "I6: No escape hatches. Direct filesystem types must be in transport layer",
            )
            .disallow_type(
                "std::fs::Metadata",
                "I6: No escape hatches. Direct filesystem types must be in transport layer",
            )
            .disallow_type(
                "std::fs::Permissions",
                "I6: No escape hatches. Direct filesystem types must be in transport layer",
            )
            .disallow_type(
                "std::fs::FileType",
                "I6: No escape hatches. Direct filesystem types must be in transport layer",
            )
    }

    /// Transport pattern configured from crate policies.
    ///
    /// This is the repo-specific extension point: gunbc-dag can supply a
    /// policy list that includes infra and other hubs.
    pub fn transport_pattern_with_crates(crates: &[crate::policy::CratePolicy]) -> Self {
        let mut config = Self::transport_pattern_base();
        for policy in crates {
            if let Some(allowance) = policy.disallowed_methods_allowance() {
                config.crate_allowances.push(allowance);
            }
        }
        config
    }
}

// ============================================================================
// TOML Generation
// ============================================================================

/// Generate clippy.toml content from a configuration.
///
/// The output includes a header comment explaining the configuration.
pub fn generate_clippy_toml(config: &ClippyConfig) -> String {
    let blocks = build_clippy_toml_blocks(config);
    let renderer = PlainStructuredRenderer::new(PlainText {
        tier: Tier::Ascii,
        symbol_set: &STANDARD,
    });

    let mut output = String::new();
    for block in &blocks {
        output.push_str(&renderer.render_block(block));
    }
    output
}

/// Build clippy.toml as structured blocks.
fn build_clippy_toml_blocks(config: &ClippyConfig) -> Vec<StructuredBlock> {
    let mut blocks = Vec::new();

    // Header comments
    blocks.push(StructuredBlock::Raw(
        "# Clippy configuration for gunbc\n#\n".to_string(),
    ));

    // Large error threshold
    if config.large_error_threshold.is_some() {
        blocks.push(StructuredBlock::Raw(
            "# BuilderError is intentionally large (144 bytes) to contain diagnostic info.\n\
             # Increase the threshold to allow it in Result types.\n"
                .to_string(),
        ));
    }
    if let Some(threshold) = config.large_error_threshold {
        blocks.push(StructuredBlock::Raw(format!(
            "large-error-threshold = {}\n\n",
            threshold
        )));
    }

    // Invariants documentation
    if !config.disallowed_methods.is_empty() {
        let mut invariants = String::new();
        invariants.push_str("# This configuration enforces system invariants:\n");
        invariants.push_str("#\n");
        invariants.push_str("# I6. NO ESCAPE HATCHES\n");
        invariants
            .push_str("#     The system cannot be bypassed. If I/O must go through transport,\n");
        invariants.push_str("#     there's no function to call to skip it.\n");
        invariants.push_str("#\n");
        invariants.push_str("# I7. NO FALLBACKS\n");
        invariants.push_str("#     Operations either succeed or fail. No silent degradation.\n");
        invariants.push_str("#     Don't use .unwrap_or_default() to hide errors.\n");
        invariants.push_str("#\n");
        invariants.push_str("# I8. NO WARNINGS\n");
        invariants.push_str("#     Run with: cargo clippy --all-targets -- -D warnings\n");
        invariants.push_str("#     Warnings are errors. If something is wrong, the build fails.\n");
        invariants.push_str("#\n");
        invariants.push_str("# APPROVED EXCEPTIONS (must have documented reason):\n");
        for allowance in &config.crate_allowances {
            writeln!(
                invariants,
                "#   - {} ({})",
                allowance.crate_name, allowance.reason
            )
            .unwrap();
        }
        invariants.push_str("#\n");
        invariants.push_str(
            "# To add an exception: #[allow(clippy::disallowed_methods)] with comment.\n",
        );
        invariants.push('\n');
        blocks.push(StructuredBlock::Raw(invariants));
    }

    // Disallowed methods array (TOML-specific syntax → Raw blocks)
    if !config.disallowed_methods.is_empty() {
        let fs_methods: Vec<_> = config
            .disallowed_methods
            .iter()
            .filter(|m| m.path.starts_with("std::fs::"))
            .collect();
        let process_methods: Vec<_> = config
            .disallowed_methods
            .iter()
            .filter(|m| m.path.starts_with("std::process::"))
            .collect();
        let other_methods: Vec<_> = config
            .disallowed_methods
            .iter()
            .filter(|m| !m.path.starts_with("std::fs::") && !m.path.starts_with("std::process::"))
            .collect();
        let secret_plaintext_methods: Vec<_> = other_methods
            .iter()
            .filter(|method| {
                method.path == "gunbc_ir::value::SecretString::expose_plaintext_for_transport"
                    || method.path == "gunbc_ir::transport::credential::Secret::expose_plaintext_for_transport"
            })
            .copied()
            .collect();
        let generic_other_methods: Vec<_> = other_methods
            .iter()
            .filter(|method| {
                method.path != "gunbc_ir::value::SecretString::expose_plaintext_for_transport"
                    && method.path != "gunbc_ir::transport::credential::Secret::expose_plaintext_for_transport"
            })
            .copied()
            .collect();

        let mut array = String::from("disallowed-methods = [\n");

        if !fs_methods.is_empty() {
            array.push_str(
                "    # Filesystem operations - use PrepareFileReadOp/PrepareFileWriteOp instead\n",
            );
            for method in &fs_methods {
                writeln!(
                    array,
                    "    {{ path = \"{}\", reason = \"{}\" }},",
                    method.path, method.reason
                )
                .unwrap();
            }
            array.push_str("    \n");
        }

        if !process_methods.is_empty() {
            array.push_str(
                "    # Process execution - use node.requires(&cli::TOOL) for tool dependencies\n",
            );
            array.push_str(
                "    # Direct Command::new should only be in transport executor and cli.rs\n",
            );
            for method in &process_methods {
                writeln!(
                    array,
                    "    {{ path = \"{}\", reason = \"{}\" }},",
                    method.path, method.reason
                )
                .unwrap();
            }
        }

        if !generic_other_methods.is_empty() {
            array.push_str("    # Other disallowed methods\n");
            for method in &generic_other_methods {
                writeln!(
                    array,
                    "    {{ path = \"{}\", reason = \"{}\" }},",
                    method.path, method.reason
                )
                .unwrap();
            }
        }

        if !secret_plaintext_methods.is_empty() {
            array.push_str("    # Secret plaintext extraction aliases — force transport-boundary-only naming.\n");
            for method in &secret_plaintext_methods {
                writeln!(
                    array,
                    "    {{ path = \"{}\", reason = \"{}\" }},",
                    method.path, method.reason
                )
                .unwrap();
            }
        }

        array.push_str("]\n");
        blocks.push(StructuredBlock::Raw(array));
    }

    // Disallowed types array
    if !config.disallowed_types.is_empty() {
        let fs_types: Vec<_> = config
            .disallowed_types
            .iter()
            .filter(|t| t.path.starts_with("std::fs::"))
            .collect();
        let other_types: Vec<_> = config
            .disallowed_types
            .iter()
            .filter(|t| !t.path.starts_with("std::fs::"))
            .collect();

        let mut array = String::from("\ndisallowed-types = [\n");

        if !fs_types.is_empty() {
            array.push_str(
                "    # Filesystem types - use FilesystemHandle and transport ops instead\n",
            );
            for ty in &fs_types {
                writeln!(
                    array,
                    "    {{ path = \"{}\", reason = \"{}\" }},",
                    ty.path, ty.reason
                )
                .unwrap();
            }
        }

        if !other_types.is_empty() {
            array.push_str("    # Other disallowed types\n");
            for ty in &other_types {
                writeln!(
                    array,
                    "    {{ path = \"{}\", reason = \"{}\" }},",
                    ty.path, ty.reason
                )
                .unwrap();
            }
        }

        array.push_str("]\n");
        blocks.push(StructuredBlock::Raw(array));
    }

    blocks
}

// ============================================================================
// ClippyConfigRenderer
// ============================================================================

/// Wrapper for rendering ClippyConfig with the standard header.
pub struct ClippyConfigRenderer {
    config: ClippyConfig,
    regenerate_command: String,
}

impl ClippyConfigRenderer {
    /// Create a new renderer with the given config.
    pub fn new(config: ClippyConfig) -> Self {
        Self {
            config,
            regenerate_command: DEFAULT_REGENERATE_CMD.to_string(),
        }
    }

    /// Create a new renderer with a custom regenerate command.
    pub fn with_regenerate_command(
        config: ClippyConfig,
        regenerate_command: impl Into<String>,
    ) -> Self {
        Self {
            config,
            regenerate_command: regenerate_command.into(),
        }
    }

    /// Create a renderer with the transport pattern config.
    pub fn transport_pattern() -> Self {
        Self::new(ClippyConfig::transport_pattern())
    }
}

/// Composed generator name for ClippyConfigRenderer.
/// Must match `cargo::name("clippy")` — verified by test.
const CLIPPY_GENERATOR_NAME: &str = "gunbc-clippy";
/// Default regenerate command for clippy.toml.
const DEFAULT_REGENERATE_CMD: &str = "cargo run -p gunbc-dag --bin gunbc-pragma";

impl ClippyConfigRenderer {
    /// Render the complete clippy.toml with header.
    pub fn render(&self) -> String {
        let header = FileHeader {
            generator_name: Cow::Borrowed(CLIPPY_GENERATOR_NAME),
            regenerate_command: Cow::Owned(self.regenerate_command.clone()),
            comment_prefix: Cow::Borrowed("#"),
        };
        format!(
            "{}\n\n{}",
            header.render(),
            generate_clippy_toml(&self.config)
        )
    }

    /// Get the generator name.
    pub fn generator_name(&self) -> &str {
        CLIPPY_GENERATOR_NAME
    }

    /// Get the regenerate command.
    pub fn regenerate_command(&self) -> &str {
        &self.regenerate_command
    }

    /// Render just the content (without header).
    pub fn render_content(&self) -> String {
        generate_clippy_toml(&self.config)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disallowed_method_creation() {
        let method = DisallowedMethod::new("std::fs::read", "Use transport layer");
        assert_eq!(method.path, "std::fs::read");
        assert_eq!(method.reason, "Use transport layer");
    }

    #[test]
    fn test_disallowed_type_creation() {
        let ty = DisallowedType::new("std::fs::File", "Use transport layer");
        assert_eq!(ty.path, "std::fs::File");
        assert_eq!(ty.reason, "Use transport layer");
    }

    #[test]
    fn test_crate_allowance_creation() {
        let allowance = CrateAllowance::new("gunbc-lib-transport", "I/O boundary");
        assert_eq!(allowance.crate_name, "gunbc-lib-transport");
        assert_eq!(allowance.reason, "I/O boundary");
    }

    #[test]
    fn test_clippy_config_builder() {
        let config = ClippyConfig::new()
            .disallow("std::fs::read", "reason1")
            .disallow("std::fs::write", "reason2")
            .disallow_type("std::fs::File", "reason3")
            .allow_crate("my-crate", "special case");

        assert_eq!(config.disallowed_methods.len(), 2);
        assert_eq!(config.disallowed_types.len(), 1);
        assert_eq!(config.crate_allowances.len(), 1);
    }

    #[test]
    fn test_transport_pattern_preset() {
        let config = ClippyConfig::transport_pattern();

        // Should have filesystem methods disallowed
        assert!(config
            .disallowed_methods
            .iter()
            .any(|m| m.path == "std::fs::read"));
        assert!(config
            .disallowed_methods
            .iter()
            .any(|m| m.path == "std::fs::write"));

        // Should have Command::new disallowed
        assert!(config
            .disallowed_methods
            .iter()
            .any(|m| m.path == "std::process::Command::new"));

        // Should have filesystem types disallowed
        assert!(config
            .disallowed_types
            .iter()
            .any(|t| t.path == "std::fs::File"));

        // Should have approved crates
        assert!(config
            .crate_allowances
            .iter()
            .any(|c| c.crate_name == "gunbc-lib-transport"));
    }

    #[test]
    fn test_generate_clippy_toml() {
        let config = ClippyConfig::transport_pattern();
        let toml = generate_clippy_toml(&config);

        // Should contain header
        assert!(toml.contains("# Clippy configuration for gunbc"));

        // Should contain large-error-threshold
        assert!(toml.contains("large-error-threshold = 256"));

        // Should contain disallowed-methods array
        assert!(toml.contains("disallowed-methods = ["));
        assert!(toml.contains("disallowed-types = ["));

        // Should contain specific methods
        assert!(toml.contains("std::fs::read"));
        assert!(toml.contains("std::process::Command::new"));
        assert!(toml.contains(
            "# Secret plaintext extraction aliases — force transport-boundary-only naming."
        ));
        assert!(toml.contains("gunbc_ir::value::SecretString::expose"));
        assert!(toml.contains("gunbc_ir::transport::credential::Secret::expose"));
        assert!(toml.contains("std::fs::File"));
    }

    #[test]
    fn test_render_implementation() {
        let renderer = ClippyConfigRenderer::transport_pattern();

        let output = renderer.render();
        assert!(output.contains("# Generated by gunbc-clippy"));
        assert!(output.contains("gunbc-pragma"));
        assert!(output.contains("disallowed-methods"));
    }
}
