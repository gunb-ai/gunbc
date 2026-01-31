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
//! ```ignore
//! use gunbc_clippy::config::{ClippyConfig, generate_clippy_toml};
//!
//! // Use a preset configuration
//! let config = ClippyConfig::transport_pattern();
//!
//! // Generate the TOML
//! let toml = generate_clippy_toml(&config);
//! ```

use gunbc_ir::Renderable;

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

/// A crate-level allowance for bypassing disallowed methods.
///
/// Documents which crates have `#![allow(clippy::disallowed_methods)]`
/// and why they are permitted to bypass the rules.
#[derive(Debug, Clone)]
pub struct CrateAllowance {
    /// Crate name (e.g., "gunbc-transport")
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
        self.disallowed_methods.push(DisallowedMethod::new(path, reason));
        self
    }

    /// Add a crate allowance.
    pub fn allow_crate(mut self, crate_name: &'static str, reason: &'static str) -> Self {
        self.crate_allowances.push(CrateAllowance::new(crate_name, reason));
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
    /// - `gunbc-transport` — IS the I/O boundary (the one place I/O is allowed)
    /// - `gunbc-codegen` — Bootstrap code (chicken/egg problem with transport)
    pub fn transport_pattern() -> Self {
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
                "std::fs::remove_file",
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
            // Process execution - enforces I6
            .disallow(
                "std::process::Command::new",
                "I6: No escape hatches. Use node.requires(&cli::TOOL). Command::new only in transport executor.",
            )
            // Document approved crates (minimal exceptions)
            // Crate names follow the {PREFIX}-{component} pattern (see cargo::name)
            .allow_crate(
                "gunbc-transport",
                "IS the I/O boundary - the designated place for I/O",
            )
            .allow_crate(
                "gunbc-codegen",
                "Bootstrap code - can't use transport (chicken/egg)",
            )
    }
}

// ============================================================================
// TOML Generation
// ============================================================================

/// Generate clippy.toml content from a configuration.
///
/// The output includes a header comment explaining the configuration.
pub fn generate_clippy_toml(config: &ClippyConfig) -> String {
    let mut output = String::new();

    // Header comments
    output.push_str("# Clippy configuration for gunbc\n");
    output.push_str("#\n");

    // Large error threshold comment
    if config.large_error_threshold.is_some() {
        output.push_str("# BuilderError is intentionally large (144 bytes) to contain diagnostic info.\n");
        output.push_str("# Increase the threshold to allow it in Result types.\n");
    }

    // Large error threshold setting
    if let Some(threshold) = config.large_error_threshold {
        output.push_str(&format!("large-error-threshold = {}\n\n", threshold));
    }

    // Document the invariants being enforced
    if !config.disallowed_methods.is_empty() {
        output.push_str("# This configuration enforces system invariants:\n");
        output.push_str("#\n");
        output.push_str("# I6. NO ESCAPE HATCHES\n");
        output.push_str("#     The system cannot be bypassed. If I/O must go through transport,\n");
        output.push_str("#     there's no function to call to skip it.\n");
        output.push_str("#\n");
        output.push_str("# I7. NO FALLBACKS\n");
        output.push_str("#     Operations either succeed or fail. No silent degradation.\n");
        output.push_str("#     Don't use .unwrap_or_default() to hide errors.\n");
        output.push_str("#\n");
        output.push_str("# I8. NO WARNINGS\n");
        output.push_str("#     Run with: cargo clippy --all-targets -- -D warnings\n");
        output.push_str("#     Warnings are errors. If something is wrong, the build fails.\n");
        output.push_str("#\n");
        output.push_str("# APPROVED EXCEPTIONS (must have documented reason):\n");
        for allowance in &config.crate_allowances {
            output.push_str(&format!("#   - {} ({})\n", allowance.crate_name, allowance.reason));
        }
        output.push_str("#\n");
        output.push_str("# To add an exception: #[allow(clippy::disallowed_methods)] with comment.\n");
        output.push('\n');
    }

    // Disallowed methods array
    if !config.disallowed_methods.is_empty() {
        output.push_str("disallowed-methods = [\n");

        // Group by category (filesystem vs process)
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

        // Filesystem operations
        if !fs_methods.is_empty() {
            output.push_str("    # Filesystem operations - use PrepareFileReadOp/PrepareFileWriteOp instead\n");
            for method in &fs_methods {
                output.push_str(&format!(
                    "    {{ path = \"{}\", reason = \"{}\" }},\n",
                    method.path, method.reason
                ));
            }
            output.push_str("    \n");
        }

        // Process execution
        if !process_methods.is_empty() {
            output.push_str("    # Process execution - use node.requires(&cli::TOOL) for tool dependencies\n");
            output.push_str("    # Direct Command::new should only be in transport executor and cli.rs\n");
            for method in &process_methods {
                output.push_str(&format!(
                    "    {{ path = \"{}\", reason = \"{}\" }},\n",
                    method.path, method.reason
                ));
            }
        }

        // Other methods
        if !other_methods.is_empty() {
            output.push_str("    # Other disallowed methods\n");
            for method in &other_methods {
                output.push_str(&format!(
                    "    {{ path = \"{}\", reason = \"{}\" }},\n",
                    method.path, method.reason
                ));
            }
        }

        output.push_str("]\n");
    }

    output
}

// ============================================================================
// Renderable Implementation
// ============================================================================

/// Wrapper for rendering ClippyConfig with the standard header.
pub struct ClippyConfigRenderer {
    config: ClippyConfig,
}

impl ClippyConfigRenderer {
    /// Create a new renderer with the given config.
    pub fn new(config: ClippyConfig) -> Self {
        Self { config }
    }

    /// Create a renderer with the transport pattern config.
    pub fn transport_pattern() -> Self {
        Self::new(ClippyConfig::transport_pattern())
    }
}

/// Composed generator name for ClippyConfigRenderer.
/// Must match `cargo::name("clippy")` — verified by test.
const CLIPPY_GENERATOR_NAME: &str = "gunbc-clippy";

impl Renderable for ClippyConfigRenderer {
    fn generator_name(&self) -> &str {
        CLIPPY_GENERATOR_NAME
    }

    fn regenerate_command(&self) -> &str {
        "cargo run -p gunbc-codegen -- clippy-toml"
    }

    fn format_id(&self) -> &str {
        "toml"
    }

    fn render_content(&self) -> String {
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
    fn test_crate_allowance_creation() {
        let allowance = CrateAllowance::new("gunbc-transport", "I/O boundary");
        assert_eq!(allowance.crate_name, "gunbc-transport");
        assert_eq!(allowance.reason, "I/O boundary");
    }

    #[test]
    fn test_clippy_config_builder() {
        let config = ClippyConfig::new()
            .disallow("std::fs::read", "reason1")
            .disallow("std::fs::write", "reason2")
            .allow_crate("my-crate", "special case");

        assert_eq!(config.disallowed_methods.len(), 2);
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

        // Should have approved crates
        assert!(config
            .crate_allowances
            .iter()
            .any(|c| c.crate_name == "gunbc-transport"));
        assert!(config
            .crate_allowances
            .iter()
            .any(|c| c.crate_name == "gunbc-codegen"));
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

        // Should contain specific methods
        assert!(toml.contains("std::fs::read"));
        assert!(toml.contains("std::process::Command::new"));
    }

    #[test]
    fn test_renderable_implementation() {
        let renderer = ClippyConfigRenderer::transport_pattern();

        assert_eq!(renderer.generator_name(), "gunbc-clippy");
        assert!(renderer.regenerate_command().contains("clippy-toml"));

        let content = renderer.render_content();
        assert!(content.contains("disallowed-methods"));
    }
}
