//! Repo-specific pragma policy (clippy + allowlists).
//!
//! This models the repo layout and exceptions on top of gunbc-clippy's
//! crate-level policy types.
//!
//! # Data flow
//!
//! Policy data is defined in DSL understanding + config files:
//!   - `dsl/understanding/*.dag` — API surface facts (Layer 2)
//!   - `dsl/config/arch_rules.dag` — invariants + exemptions (Layer 3)
//!
//! Phase 1 (current): Rust `default_*()` functions mirror the DSL data.
//! Phase 2 (future): extern impls receive data from DSL pipeline.

use gunbc_clippy::{ClippyConfig, ClippyConfigRenderer, CratePolicy, CrateRole, LintId};
use gunbc_ir::render_ir::{FileHeader, PlainText, StructuredBlock, StructuredRenderer};
use gunbc_ir::symbols::{Tier, STANDARD};
use gunbc_ir::{PlainStructuredRenderer, WorkspaceLayout};
use std::borrow::Cow;

/// Regenerate command for pragma outputs.
pub const PRAGMA_REGENERATE_CMD: &str = "cargo run -p gunbc-dag --bin gunbc-pragma";

/// Allowlist entry for #[allow(clippy::disallowed_methods)] occurrences.
///
/// These are path prefixes (repo-relative) that are exempted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisallowedMethodsAllowPattern {
    pub pattern: String,
    pub rationale: &'static str,
}

/// Policy for pragma linting (non-clippy).
#[derive(Debug, Clone)]
pub struct PragmaLintPolicy {
    /// Files allowed to use #[allow(dead_code)].
    pub allow_dead_code: Vec<String>,
    /// Lints allowed in #[allow(...)] pragmas.
    pub allow_lints: Vec<LintId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CrateSelector {
    Exact(&'static str),
    Prefix(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DisallowedMethodsAllowRule {
    pub(crate) selector: CrateSelector,
    pub(crate) suffix: &'static str,
    pub(crate) as_prefix: bool,
    pub(crate) rationale: &'static str,
    pub(crate) fallback_pattern: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DeadCodeAllowRule {
    pub(crate) crate_name: &'static str,
    pub(crate) relative_path: &'static str,
    pub(crate) fallback_path: &'static str,
}

// ============================================================================
// Default policy data (Phase 1: mirrors DSL config/arch_rules.dag)
//
// These functions return the same data as the DSL understanding + config files.
// In Phase 2, extern impls will receive this data from the DSL pipeline instead.
// ============================================================================

/// Default crate policies for clippy allowances.
/// Mirrors: `dsl/config/arch_rules.dag` exemptions with WholePackage scope.
pub(crate) fn default_crate_policies() -> Vec<CratePolicy> {
    vec![CratePolicy::allow_disallowed_methods(
        "gunbc-lib-transport",
        CrateRole::TransportBoundary,
        "IS the I/O boundary - the designated place for I/O",
    )]
}

/// Default allowlist rules for #[allow(clippy::disallowed_methods)].
/// Mirrors: `dsl/config/arch_rules.dag` exemptions.
pub(crate) fn default_allowlist_rules() -> Vec<DisallowedMethodsAllowRule> {
    vec![
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-lib-transport"),
            suffix: "",
            as_prefix: true,
            rationale: "transport boundary",
            fallback_pattern: "lib/transport/",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-exec"),
            suffix: "src/freshness.rs",
            as_prefix: false,
            rationale: "freshness steps run external tooling as child processes",
            fallback_pattern: "core/exec/src/freshness.rs",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-exec"),
            suffix: "src/display.rs",
            as_prefix: false,
            rationale: "CI secret masking at transport boundary",
            fallback_pattern: "core/exec/src/display.rs",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-exec"),
            suffix: "src/execute.rs",
            as_prefix: false,
            rationale: "CI secret masking at transport boundary",
            fallback_pattern: "core/exec/src/execute.rs",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Prefix("daglang-"),
            suffix: "",
            as_prefix: true,
            rationale: "compiler pipeline: filesystem discovery for .dag module resolution",
            fallback_pattern: "core/daglang/",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-dag"),
            suffix: "src/",
            as_prefix: true,
            rationale: "build-time DSL module discovery and workspace graph construction",
            fallback_pattern: "gunbc-dag/src/",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-ir"),
            suffix: "src/workspace_layout.rs",
            as_prefix: false,
            rationale: "workspace layout discovery uses cargo metadata subprocess",
            fallback_pattern: "core/ir/src/workspace_layout.rs",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-ir"),
            suffix: "src/transport/credential.rs",
            as_prefix: false,
            rationale: "transport boundary: credential applied to outbound requests",
            fallback_pattern: "core/ir/src/transport/credential.rs",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-ir"),
            suffix: "src/resource/",
            as_prefix: true,
            rationale: "capability-marker validation for resource handles",
            fallback_pattern: "core/ir/src/resource/",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-lib-cloud-ops"),
            suffix: "",
            as_prefix: true,
            rationale: "cloud ops: file-backed config, credential policy, secret cache",
            fallback_pattern: "lib/cloud-ops/",
        },
        DisallowedMethodsAllowRule {
            selector: CrateSelector::Exact("gunbc-lib-gcp-ops"),
            suffix: "",
            as_prefix: true,
            rationale: "GCP ops: secret extraction for GCP service requests",
            fallback_pattern: "lib/gcp-ops/",
        },
    ]
}

/// Default dead code allowance rules.
/// Mirrors: `dsl/config/arch_rules.dag` dead_code_allowances.
pub(crate) fn default_dead_code_rules() -> Vec<DeadCodeAllowRule> {
    vec![
        DeadCodeAllowRule {
            crate_name: "daglang-syntax",
            relative_path: "src/parser.rs",
            fallback_path: "core/daglang/daglang-syntax/src/parser.rs",
        },
        DeadCodeAllowRule {
            crate_name: "daglang-emit",
            relative_path: "src/lower_mips.rs",
            fallback_path: "core/daglang/daglang-emit/src/lower_mips.rs",
        },
        DeadCodeAllowRule {
            crate_name: "gunbc-dag",
            relative_path: "src/makegen/registry.rs",
            fallback_path: "gunbc-dag/src/makegen/registry.rs",
        },
        DeadCodeAllowRule {
            crate_name: "gunbc-dag",
            relative_path: "src/workspace/subdags/languages.rs",
            fallback_path: "gunbc-dag/src/workspace/subdags/languages.rs",
        },
        DeadCodeAllowRule {
            crate_name: "gunbc-lib-gcp-ops",
            relative_path: "src/graph.rs",
            fallback_path: "lib/gcp-ops/src/graph.rs",
        },
    ]
}

/// Default pragma-allowed lints.
/// Mirrors: `dsl/config/arch_rules.dag` pragma_allow_lints.
pub(crate) fn default_pragma_allow_lints() -> Vec<LintId> {
    vec![
        LintId::clippy("large_enum_variant"),
        LintId::clippy("too_many_arguments"),
        LintId::clippy("vec_init_then_push"),
        LintId::rustc("unused_variables"),
    ]
}

// ============================================================================
// Parameterized policy functions
// ============================================================================

/// Build clippy config from crate policies.
pub fn clippy_config_from(crates: &[CratePolicy]) -> ClippyConfig {
    ClippyConfig::transport_pattern_with_crates(crates)
}

/// Build a renderer for clippy.toml from config.
pub fn render_clippy_toml_from(config: ClippyConfig) -> String {
    clippy_renderer_from(config).render()
}

/// Build a renderer for clippy.toml.
pub fn clippy_renderer_from(config: ClippyConfig) -> ClippyConfigRenderer {
    ClippyConfigRenderer::with_regenerate_command(config, PRAGMA_REGENERATE_CMD)
}

/// Resolve allowlist entries from rules.
pub(crate) fn resolve_allowlist_from(
    rules: &[DisallowedMethodsAllowRule],
) -> Vec<DisallowedMethodsAllowPattern> {
    let layout = workspace_layout_or_none();
    let mut patterns = Vec::new();

    for rule in rules {
        let mut resolved = resolve_allowlist_rule(layout.as_ref(), rule);
        if resolved.is_empty() {
            resolved.push(rule.fallback_pattern.to_string());
        }
        resolved.sort();
        resolved.dedup();
        for pattern in resolved {
            patterns.push(DisallowedMethodsAllowPattern {
                pattern,
                rationale: rule.rationale,
            });
        }
    }

    patterns
}

/// Build pragma lint policy from rules.
pub(crate) fn lint_policy_from(
    dead_code_rules: &[DeadCodeAllowRule],
    allow_lints: Vec<LintId>,
) -> PragmaLintPolicy {
    PragmaLintPolicy {
        allow_dead_code: resolve_dead_code_from(dead_code_rules),
        allow_lints,
    }
}

/// Render the disallowed-methods allowlist file from resolved patterns.
pub fn render_allowlist_from(allowlist: &[DisallowedMethodsAllowPattern]) -> String {
    let header = FileHeader {
        generator_name: Cow::Borrowed("gunbc-pragma"),
        regenerate_command: Cow::Borrowed(PRAGMA_REGENERATE_CMD),
        comment_prefix: Cow::Borrowed("#"),
    };

    let mut blocks = Vec::new();
    blocks.push(StructuredBlock::Raw(
        "# Allowed path prefixes for #[allow(clippy::disallowed_methods)] and #[allow(clippy::disallowed_types)].\n\
         # Format: prefix (repo-relative)\n\
         # Note: any path containing \"/tests/\" is always allowed.\n#\n"
            .to_string(),
    ));
    for entry in allowlist {
        blocks.push(StructuredBlock::Raw(format!(
            "# {}\n{}\n",
            entry.rationale, entry.pattern
        )));
    }

    let renderer = PlainStructuredRenderer::new(PlainText {
        tier: Tier::Ascii,
        symbol_set: &STANDARD,
    });
    let mut output = header.render();
    output.push('\n');
    for block in &blocks {
        output.push_str(&renderer.render_block(block));
    }
    output
}

/// Render the pragma lint policy file from policy data.
pub fn render_lint_policy_from(policy: &PragmaLintPolicy) -> String {
    let header = FileHeader {
        generator_name: Cow::Borrowed("gunbc-pragma"),
        regenerate_command: Cow::Borrowed(PRAGMA_REGENERATE_CMD),
        comment_prefix: Cow::Borrowed("#"),
    };

    let mut blocks = Vec::new();

    // [allow.dead_code] section
    blocks.push(StructuredBlock::Raw("[allow.dead_code]\n".to_string()));
    if policy.allow_dead_code.is_empty() {
        blocks.push(StructuredBlock::Raw("# (none)\n".to_string()));
    } else {
        for path in &policy.allow_dead_code {
            blocks.push(StructuredBlock::Raw(format!("{}\n", path)));
        }
    }

    // [allow.lints] section
    blocks.push(StructuredBlock::Raw("\n[allow.lints]\n".to_string()));
    if policy.allow_lints.is_empty() {
        blocks.push(StructuredBlock::Raw("# (none)\n".to_string()));
    } else {
        for lint in &policy.allow_lints {
            blocks.push(StructuredBlock::Raw(format!("{}\n", lint.allow_name())));
        }
    }

    let renderer = PlainStructuredRenderer::new(PlainText {
        tier: Tier::Ascii,
        symbol_set: &STANDARD,
    });
    let mut output = header.render();
    output.push('\n');
    output.push('\n');
    for block in &blocks {
        output.push_str(&renderer.render_block(block));
    }
    output
}

// ============================================================================
// Convenience functions using defaults (Phase 1 wrappers)
// ============================================================================

/// Repo-specific crate policies (for clippy allowances).
pub fn crate_policies() -> Vec<CratePolicy> {
    default_crate_policies()
}

/// Build clippy config from default repo policy.
pub fn clippy_config() -> ClippyConfig {
    clippy_config_from(&default_crate_policies())
}

/// Build a renderer for clippy.toml using default repo policy.
pub fn clippy_renderer() -> ClippyConfigRenderer {
    clippy_renderer_from(clippy_config())
}

/// Allowlist entries for #[allow(clippy::disallowed_methods)] using defaults.
pub fn disallowed_methods_allowlist() -> Vec<DisallowedMethodsAllowPattern> {
    resolve_allowlist_from(&default_allowlist_rules())
}

/// Policy for pragma lint rules using defaults.
pub fn pragma_lint_policy() -> PragmaLintPolicy {
    lint_policy_from(&default_dead_code_rules(), default_pragma_allow_lints())
}

/// Render the disallowed-methods allowlist file using defaults.
pub fn render_disallowed_methods_allowlist() -> String {
    render_allowlist_from(&disallowed_methods_allowlist())
}

/// Render the pragma lint policy file using defaults.
pub fn render_pragma_lint_policy() -> String {
    render_lint_policy_from(&pragma_lint_policy())
}

// ============================================================================
// Internal resolution logic
// ============================================================================

fn workspace_layout_or_none() -> Option<WorkspaceLayout> {
    WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .ok()
}

fn resolve_allowlist_rule(
    layout: Option<&WorkspaceLayout>,
    rule: &DisallowedMethodsAllowRule,
) -> Vec<String> {
    let Some(layout) = layout else {
        return Vec::new();
    };

    let mut matches = Vec::new();
    match rule.selector {
        CrateSelector::Exact(crate_name) => {
            if let Some(crate_dir) = layout.crate_dir(crate_name) {
                matches.push(resolve_crate_rule_path(
                    layout,
                    crate_dir,
                    rule.suffix,
                    rule.as_prefix,
                ));
            }
        }
        CrateSelector::Prefix(prefix) => {
            for (crate_name, crate_dir) in &layout.crates {
                if crate_name.starts_with(prefix) {
                    matches.push(resolve_crate_rule_path(
                        layout,
                        crate_dir,
                        rule.suffix,
                        rule.as_prefix,
                    ));
                }
            }
        }
    }
    matches
}

fn resolve_crate_rule_path(
    layout: &WorkspaceLayout,
    crate_dir: &std::path::Path,
    suffix: &str,
    as_prefix: bool,
) -> String {
    let mut path = layout.relative_path(&layout.workspace_root, crate_dir);
    if !suffix.is_empty() {
        path = path.join(suffix);
    }
    let mut normalized = path.to_string_lossy().replace('\\', "/");
    if as_prefix && !normalized.ends_with('/') {
        normalized.push('/');
    }
    normalized
}

fn resolve_dead_code_from(rules: &[DeadCodeAllowRule]) -> Vec<String> {
    let layout = workspace_layout_or_none();
    let mut paths = Vec::new();
    for rule in rules {
        let resolved = layout
            .as_ref()
            .and_then(|layout| {
                layout
                    .crate_dir(rule.crate_name)
                    .map(|crate_dir| (layout, crate_dir))
            })
            .map(|(layout, crate_dir)| {
                let rel = layout.relative_path(&layout.workspace_root, crate_dir);
                rel.join(rule.relative_path)
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .unwrap_or_else(|| rule.fallback_path.to_string());
        paths.push(resolved);
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disallowed_methods_allowlist_resolves_workspace_paths() {
        let allowlist = disallowed_methods_allowlist();
        assert!(
            allowlist
                .iter()
                .any(|entry| entry.pattern == "lib/transport/"),
            "transport allowlist should resolve via crate path"
        );
        assert!(
            allowlist
                .iter()
                .any(|entry| entry.pattern == "core/exec/src/freshness.rs"),
            "freshness allowlist should resolve via crate path"
        );
        assert!(
            allowlist
                .iter()
                .any(|entry| entry.pattern.starts_with("core/daglang/")),
            "daglang crate-prefix allowlist should resolve from crate names"
        );
        assert!(
            allowlist
                .iter()
                .any(|entry| entry.pattern == "gunbc-dag/src/"),
            "gunbc-dag allowlist should resolve to crate src prefix"
        );
    }

    #[test]
    fn pragma_lint_policy_resolves_dead_code_paths_from_crate_locations() {
        let policy = pragma_lint_policy();
        assert!(
            policy
                .allow_dead_code
                .iter()
                .any(|path| path == "core/daglang/daglang-syntax/src/parser.rs"),
            "dead_code allowlist should resolve parser path via crate location"
        );
    }

    #[test]
    fn parameterized_clippy_config_matches_default() {
        let default = clippy_config();
        let parameterized = clippy_config_from(&default_crate_policies());
        assert_eq!(
            default.disallowed_methods.len(),
            parameterized.disallowed_methods.len()
        );
        assert_eq!(
            default.crate_allowances.len(),
            parameterized.crate_allowances.len()
        );
    }

    #[test]
    fn parameterized_allowlist_matches_default() {
        let default = disallowed_methods_allowlist();
        let parameterized = resolve_allowlist_from(&default_allowlist_rules());
        assert_eq!(default.len(), parameterized.len());
        for (d, p) in default.iter().zip(parameterized.iter()) {
            assert_eq!(d.pattern, p.pattern);
            assert_eq!(d.rationale, p.rationale);
        }
    }
}
