//! Repo-specific pragma policy (clippy + allowlists).
//!
//! This models the repo layout and exceptions on top of gunbc-clippy's
//! crate-level policy types.

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
    pub allow_lints: &'static [LintId],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrateSelector {
    Exact(&'static str),
    Prefix(&'static str),
}

#[derive(Debug, Clone, Copy)]
struct DisallowedMethodsAllowRule {
    selector: CrateSelector,
    suffix: &'static str,
    as_prefix: bool,
    rationale: &'static str,
    fallback_pattern: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct DeadCodeAllowRule {
    crate_name: &'static str,
    relative_path: &'static str,
    fallback_path: &'static str,
}

const CRATE_POLICIES: &[CratePolicy] = &[CratePolicy::allow_disallowed_methods(
    "gunbc-lib-transport",
    CrateRole::TransportBoundary,
    "IS the I/O boundary - the designated place for I/O",
)];

const DISALLOWED_METHODS_ALLOWLIST_RULES: &[DisallowedMethodsAllowRule] = &[
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
        selector: CrateSelector::Exact("gunbc-lib-cloud-ops"),
        suffix: "",
        as_prefix: true,
        rationale: "cloud ops: file-backed config, credential policy, secret cache",
        fallback_pattern: "lib/cloud-ops/",
    },
];

const DEAD_CODE_ALLOW_RULES: &[DeadCodeAllowRule] = &[
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
];

const PRAGMA_ALLOW_LINTS: &[LintId] = &[
    LintId::clippy("too_many_arguments"),
    LintId::clippy("vec_init_then_push"),
    LintId::rustc("unused_variables"),
];

/// Repo-specific crate policies (for clippy allowances).
pub fn crate_policies() -> &'static [CratePolicy] {
    CRATE_POLICIES
}

/// Build clippy config from repo policy.
pub fn clippy_config() -> ClippyConfig {
    ClippyConfig::transport_pattern_with_crates(CRATE_POLICIES)
}

/// Build a renderer for clippy.toml using repo policy.
pub fn clippy_renderer() -> ClippyConfigRenderer {
    ClippyConfigRenderer::with_regenerate_command(clippy_config(), PRAGMA_REGENERATE_CMD)
}

/// Allowlist entries for #[allow(clippy::disallowed_methods)].
pub fn disallowed_methods_allowlist() -> Vec<DisallowedMethodsAllowPattern> {
    resolve_disallowed_methods_allowlist()
}

/// Policy for pragma lint rules (generated code, dead_code allowances).
pub fn pragma_lint_policy() -> PragmaLintPolicy {
    PragmaLintPolicy {
        allow_dead_code: resolve_dead_code_allow_paths(),
        allow_lints: PRAGMA_ALLOW_LINTS,
    }
}

/// Render the disallowed-methods allowlist file.
pub fn render_disallowed_methods_allowlist() -> String {
    let header = FileHeader {
        generator_name: Cow::Borrowed("gunbc-pragma"),
        regenerate_command: Cow::Borrowed(PRAGMA_REGENERATE_CMD),
        comment_prefix: Cow::Borrowed("#"),
    };

    let blocks = build_allowlist_blocks();
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

/// Build allowlist as structured blocks.
fn build_allowlist_blocks() -> Vec<StructuredBlock> {
    let mut blocks = Vec::new();
    let allowlist = disallowed_methods_allowlist();

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

    blocks
}

/// Render the pragma lint policy file (dead_code allowances, allowlist).
pub fn render_pragma_lint_policy() -> String {
    let header = FileHeader {
        generator_name: Cow::Borrowed("gunbc-pragma"),
        regenerate_command: Cow::Borrowed(PRAGMA_REGENERATE_CMD),
        comment_prefix: Cow::Borrowed("#"),
    };

    let blocks = build_lint_policy_blocks();
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

/// Build lint policy as structured blocks.
fn build_lint_policy_blocks() -> Vec<StructuredBlock> {
    let mut blocks = Vec::new();
    let policy = pragma_lint_policy();

    // [allow.dead_code] section
    blocks.push(StructuredBlock::Raw("[allow.dead_code]\n".to_string()));
    if policy.allow_dead_code.is_empty() {
        blocks.push(StructuredBlock::Raw("# (none)\n".to_string()));
    } else {
        for path in policy.allow_dead_code {
            blocks.push(StructuredBlock::Raw(format!("{}\n", path)));
        }
    }

    // [allow.lints] section
    blocks.push(StructuredBlock::Raw("\n[allow.lints]\n".to_string()));
    if policy.allow_lints.is_empty() {
        blocks.push(StructuredBlock::Raw("# (none)\n".to_string()));
    } else {
        for lint in policy.allow_lints {
            blocks.push(StructuredBlock::Raw(format!("{}\n", lint.allow_name())));
        }
    }

    blocks
}

fn workspace_layout_or_none() -> Option<WorkspaceLayout> {
    WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .ok()
}

fn resolve_disallowed_methods_allowlist() -> Vec<DisallowedMethodsAllowPattern> {
    let layout = workspace_layout_or_none();
    let mut patterns = Vec::new();

    for rule in DISALLOWED_METHODS_ALLOWLIST_RULES {
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

fn resolve_dead_code_allow_paths() -> Vec<String> {
    let layout = workspace_layout_or_none();
    let mut paths = Vec::new();
    for rule in DEAD_CODE_ALLOW_RULES {
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
}
