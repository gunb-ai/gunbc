//! Repo-specific pragma policy (clippy + allowlists).
//!
//! This models the repo layout and exceptions on top of gunbc-clippy's
//! crate-level policy types.

use gunbc_clippy::{ClippyConfig, ClippyConfigRenderer, CratePolicy, CrateRole, LintId};
use gunbc_ir::render_ir::FileHeader;

/// Regenerate command for pragma outputs.
pub const PRAGMA_REGENERATE_CMD: &str = "cargo run -p gunbc-dag --bin gunbc-pragma";

/// Scope for disallowed-methods allowances.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowScope {
    Crate,
    Function,
}

/// Allowlist entry for #[allow(clippy::disallowed_methods)] occurrences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisallowedMethodsAllowEntry {
    pub path: &'static str,
    pub count: usize,
    pub scope: AllowScope,
    pub rationale: &'static str,
}

/// Policy for pragma linting (non-clippy).
#[derive(Debug, Clone, Copy)]
pub struct PragmaLintPolicy {
    /// Files allowed to use #[allow(dead_code)].
    pub allow_dead_code: &'static [&'static str],
    /// Lints allowed in #[allow(...)] pragmas.
    pub allow_lints: &'static [LintId],
}

const CRATE_POLICIES: &[CratePolicy] = &[
    CratePolicy::allow_disallowed_methods(
        "gunbc-lib-transport",
        CrateRole::TransportBoundary,
        "IS the I/O boundary - the designated place for I/O",
    ),
    CratePolicy::allow_disallowed_methods(
        "gunbc-codegen",
        CrateRole::CodegenBootstrap,
        "Bootstrap code - can't use transport (chicken/egg)",
    ),
    CratePolicy::allow_disallowed_methods(
        "gunbc-infra",
        CrateRole::Infra,
        "Infra hub (fs helpers, test utilities)",
    ),
];

const DISALLOWED_METHODS_ALLOWLIST: &[DisallowedMethodsAllowEntry] = &[
    // Crate-level exemptions
    DisallowedMethodsAllowEntry {
        path: "core/codegen/src/lib.rs",
        count: 1,
        scope: AllowScope::Crate,
        rationale: "codegen crate-level allowance",
    },
    DisallowedMethodsAllowEntry {
        path: "lib/transport/src/lib.rs",
        count: 1,
        scope: AllowScope::Crate,
        rationale: "transport crate-level allowance",
    },
    DisallowedMethodsAllowEntry {
        path: "core/infra/src/lib.rs",
        count: 1,
        scope: AllowScope::Crate,
        rationale: "infra crate-level allowance",
    },
    // Function-level exemptions
    DisallowedMethodsAllowEntry {
        path: "core/codegen/src/main.rs",
        count: 4,
        scope: AllowScope::Function,
        rationale: "codegen bootstrap I/O",
    },
    DisallowedMethodsAllowEntry {
        path: "lib/transport/src/executor.rs",
        count: 1,
        scope: AllowScope::Function,
        rationale: "transport shell executor boundary",
    },
];

const PRAGMA_LINT_POLICY: PragmaLintPolicy = PragmaLintPolicy {
    allow_dead_code: &[],
    allow_lints: &[],
};

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
pub fn disallowed_methods_allowlist() -> &'static [DisallowedMethodsAllowEntry] {
    DISALLOWED_METHODS_ALLOWLIST
}

/// Policy for pragma lint rules (generated code, dead_code allowances).
pub fn pragma_lint_policy() -> PragmaLintPolicy {
    PRAGMA_LINT_POLICY
}

/// Render the disallowed-methods allowlist file.
pub fn render_disallowed_methods_allowlist() -> String {
    let header = FileHeader {
        generator_name: "gunbc-pragma".to_string(),
        regenerate_command: PRAGMA_REGENERATE_CMD.to_string(),
        comment_prefix: "#".to_string(),
    };
    let mut output = header.render();
    output.push('\n');
    output.push_str("# Paths and counts for #[allow(clippy::disallowed_methods)] occurrences.\n");
    output.push_str("# Format: path:count\n#\n");

    output.push_str("# Crate-level exemptions (entire crate allowed):\n");
    for entry in DISALLOWED_METHODS_ALLOWLIST.iter().filter(|e| e.scope == AllowScope::Crate) {
        output.push_str(&format!("{}:{}\n", entry.path, entry.count));
    }
    output.push_str("#\n");
    output.push_str("# Function-level exemptions (I/O boundaries):\n");
    for entry in DISALLOWED_METHODS_ALLOWLIST
        .iter()
        .filter(|e| e.scope == AllowScope::Function)
    {
        output.push_str(&format!("{}:{}\n", entry.path, entry.count));
    }

    output
}

/// Render the pragma lint policy file (dead_code allowances, allowlist).
pub fn render_pragma_lint_policy() -> String {
    let header = FileHeader {
        generator_name: "gunbc-pragma".to_string(),
        regenerate_command: PRAGMA_REGENERATE_CMD.to_string(),
        comment_prefix: "#".to_string(),
    };
    let mut output = header.render();
    output.push('\n');
    output.push('\n');

    output.push_str("[allow.dead_code]\n");
    if PRAGMA_LINT_POLICY.allow_dead_code.is_empty() {
        output.push_str("# (none)\n");
    } else {
        for path in PRAGMA_LINT_POLICY.allow_dead_code {
            output.push_str(path);
            output.push('\n');
        }
    }

    output.push('\n');
    output.push_str("[allow.lints]\n");
    if PRAGMA_LINT_POLICY.allow_lints.is_empty() {
        output.push_str("# (none)\n");
    } else {
        for lint in PRAGMA_LINT_POLICY.allow_lints {
            output.push_str(&lint.allow_name());
            output.push('\n');
        }
    }

    output
}
