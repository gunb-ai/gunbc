//! Workspace structure model — commit policies and toolchain requirements.
//!
//! The hardcoded `workspace_crates()` (32 CrateSpec entries) and
//! `known_generator_edges()` (6 GeneratorEdge entries) have been removed.
//! Crate structure is derivable from `cargo metadata`. Generator edges
//! are modeled in DSL tool definitions and `CompileOutput.output_paths`.
//!
//! What remains: commit policies (used by .gitignore generation) and
//! toolchain requirements.

// ── Commit Policy ───────────────────────────────────────────────────

/// Reason a file pattern has a particular commit policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitReason {
    /// Generated artifact — never committed.
    Generated,
    /// Build artifact — never committed.
    BuildOutput,
    /// Seed file — generated but committed for bootstrap.
    BootstrapSeed,
    /// Sensitive — never committed.
    Secret,
}

/// Commit policy for a file pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitPolicy {
    /// Glob pattern (gitignore-style).
    pub pattern: &'static str,
    /// Why this pattern exists.
    pub reason: CommitReason,
    /// The tool or system that generates this artifact.
    pub producer: Option<&'static str>,
}

impl CommitPolicy {
    /// Whether this pattern should be in .gitignore.
    pub fn should_gitignore(&self) -> bool {
        matches!(
            self.reason,
            CommitReason::Generated | CommitReason::BuildOutput | CommitReason::Secret
        )
    }
}

/// Canonical commit policies for the workspace.
///
/// These are the baseline policies; tool-specific policies are derived
/// from the tool registry's `outputs` field at a higher layer.
pub fn baseline_commit_policies() -> Vec<CommitPolicy> {
    vec![
        CommitPolicy {
            pattern: "target/",
            reason: CommitReason::BuildOutput,
            producer: None,
        },
        CommitPolicy {
            pattern: ".env",
            reason: CommitReason::Secret,
            producer: None,
        },
        CommitPolicy {
            pattern: "*.pem",
            reason: CommitReason::Secret,
            producer: None,
        },
        CommitPolicy {
            pattern: "target/codegen/.stamp",
            reason: CommitReason::Generated,
            producer: Some("codegen"),
        },
        // Bootstrap seed files — generated but committed
        CommitPolicy {
            pattern: ".gitignore",
            reason: CommitReason::BootstrapSeed,
            producer: Some("bootstrap"),
        },
        CommitPolicy {
            pattern: "deps.toml",
            reason: CommitReason::BootstrapSeed,
            producer: Some("deps-config"),
        },
        CommitPolicy {
            pattern: "README.md",
            reason: CommitReason::BootstrapSeed,
            producer: Some("readme"),
        },
    ]
}

/// Generate .gitignore content from commit policies.
pub fn derive_gitignore(policies: &[CommitPolicy]) -> String {
    let mut lines = Vec::new();
    for policy in policies {
        if policy.should_gitignore() {
            lines.push(policy.pattern.to_string());
        }
    }
    lines.sort();
    lines.dedup();
    lines.join("\n")
}

// ── Toolchain Requirements ──────────────────────────────────────────

/// A toolchain requirement for building/running the repo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainRequirement {
    pub tool: &'static str,
    pub min_version: Option<&'static str>,
    pub purpose: &'static str,
    pub install_hint: &'static str,
}

/// Canonical toolchain requirements.
pub fn toolchain_requirements() -> Vec<ToolchainRequirement> {
    vec![
        ToolchainRequirement {
            tool: "rustc",
            min_version: Some("1.75.0"),
            purpose: "Rust compiler",
            install_hint: "rustup update stable",
        },
        ToolchainRequirement {
            tool: "cargo",
            min_version: Some("1.75.0"),
            purpose: "Rust package manager",
            install_hint: "rustup update stable",
        },
        ToolchainRequirement {
            tool: "make",
            min_version: None,
            purpose: "Build orchestration",
            install_hint: "apt install make",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_policies_include_build_output() {
        let policies = baseline_commit_policies();
        assert!(
            policies.iter().any(|p| p.pattern == "target/"),
            "baseline must include target/"
        );
    }

    #[test]
    fn bootstrap_seeds_are_not_gitignored() {
        let policies = baseline_commit_policies();
        for policy in &policies {
            if policy.reason == CommitReason::BootstrapSeed {
                assert!(
                    !policy.should_gitignore(),
                    "bootstrap seed '{}' should NOT be gitignored",
                    policy.pattern
                );
            }
        }
    }

    #[test]
    fn derive_gitignore_excludes_seeds() {
        let policies = baseline_commit_policies();
        let gitignore = derive_gitignore(&policies);
        assert!(
            !gitignore.contains(".gitignore"),
            ".gitignore is a seed and should not be in derived gitignore"
        );
        assert!(
            gitignore.contains("target/"),
            "target/ should be in derived gitignore"
        );
    }

    #[test]
    fn derive_gitignore_includes_secrets() {
        let policies = baseline_commit_policies();
        let gitignore = derive_gitignore(&policies);
        assert!(
            gitignore.contains(".env"),
            ".env should be in derived gitignore"
        );
    }

    #[test]
    fn toolchain_requirements_include_rustc() {
        let reqs = toolchain_requirements();
        assert!(reqs.iter().any(|r| r.tool == "rustc"), "must require rustc");
    }
}
