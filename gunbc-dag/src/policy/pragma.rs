//! Repo-specific pragma policy (clippy + allowlists).
//!
//! This models the repo layout and exceptions on top of gunbc-clippy's
//! crate-level policy types.

use gunbc_clippy::{ClippyConfig, ClippyConfigRenderer, CratePolicy, CrateRole, LintId};
use gunbc_ir::render_ir::{FileHeader, PlainText, StructuredBlock, StructuredRenderer};
use gunbc_ir::symbols::{Tier, STANDARD};
use gunbc_ir::PlainStructuredRenderer;

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
    DisallowedMethodsAllowEntry {
        path: "gunbc-dag/tests/mock_spec_registration.rs",
        count: 1,
        scope: AllowScope::Function,
        rationale: "test reads source files to validate registration",
    },
    DisallowedMethodsAllowEntry {
        path: "gunbc-dag/tests/tool_registration.rs",
        count: 3,
        scope: AllowScope::Function,
        rationale: "tests read source files to validate registration",
    },
];

const PRAGMA_LINT_POLICY: PragmaLintPolicy = PragmaLintPolicy {
    allow_dead_code: &[],
    allow_lints: &[
        LintId::clippy("too_many_arguments"),
        LintId::rustc("unused_variables"),
    ],
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

    blocks.push(StructuredBlock::Raw(
        "# Paths and counts for #[allow(clippy::disallowed_methods)] occurrences.\n\
         # Format: path:count\n#\n"
            .to_string(),
    ));

    // Crate-level exemptions
    blocks.push(StructuredBlock::Raw(
        "# Crate-level exemptions (entire crate allowed):\n".to_string(),
    ));
    for entry in DISALLOWED_METHODS_ALLOWLIST
        .iter()
        .filter(|e| e.scope == AllowScope::Crate)
    {
        blocks.push(StructuredBlock::Raw(format!(
            "{}:{}\n",
            entry.path, entry.count
        )));
    }

    // Function-level exemptions
    blocks.push(StructuredBlock::Raw(
        "#\n# Function-level exemptions (I/O boundaries):\n".to_string(),
    ));
    for entry in DISALLOWED_METHODS_ALLOWLIST
        .iter()
        .filter(|e| e.scope == AllowScope::Function)
    {
        blocks.push(StructuredBlock::Raw(format!(
            "{}:{}\n",
            entry.path, entry.count
        )));
    }

    blocks
}

/// Render the pragma lint policy file (dead_code allowances, allowlist).
pub fn render_pragma_lint_policy() -> String {
    let header = FileHeader {
        generator_name: "gunbc-pragma".to_string(),
        regenerate_command: PRAGMA_REGENERATE_CMD.to_string(),
        comment_prefix: "#".to_string(),
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

    // [allow.dead_code] section
    blocks.push(StructuredBlock::Raw("[allow.dead_code]\n".to_string()));
    if PRAGMA_LINT_POLICY.allow_dead_code.is_empty() {
        blocks.push(StructuredBlock::Raw("# (none)\n".to_string()));
    } else {
        for path in PRAGMA_LINT_POLICY.allow_dead_code {
            blocks.push(StructuredBlock::Raw(format!("{}\n", path)));
        }
    }

    // [allow.lints] section
    blocks.push(StructuredBlock::Raw("\n[allow.lints]\n".to_string()));
    if PRAGMA_LINT_POLICY.allow_lints.is_empty() {
        blocks.push(StructuredBlock::Raw("# (none)\n".to_string()));
    } else {
        for lint in PRAGMA_LINT_POLICY.allow_lints {
            blocks.push(StructuredBlock::Raw(format!("{}\n", lint.allow_name())));
        }
    }

    blocks
}
