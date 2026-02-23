//! Workspace structure model — crate tiers, dependency invariants.
//!
//! Provides a compile-time description of the workspace layout so that
//! downstream validation and codegen can reason about crate roles without
//! parsing `Cargo.toml` at runtime.

use std::collections::{BTreeMap, BTreeSet};

/// Crate role within the workspace layering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrateTier {
    /// Leaf crates with no internal dependencies (infra, registries).
    Foundation,
    /// Core IR, compiler, execution, and shared library crates.
    Core,
    /// Top-level binaries and integration glue (cloud providers, tools, gunbc-dag).
    Application,
}

impl CrateTier {
    fn rank(self) -> u8 {
        match self {
            CrateTier::Foundation => 0,
            CrateTier::Core => 1,
            CrateTier::Application => 2,
        }
    }
}

impl PartialOrd for CrateTier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CrateTier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

/// Descriptor for one workspace member crate.
#[derive(Debug, Clone)]
pub struct CrateSpec {
    /// Crate name as it appears in `Cargo.toml` members (directory path).
    pub name: &'static str,
    /// Tier in the layering hierarchy.
    pub tier: CrateTier,
    /// Short human description.
    pub description: &'static str,
    /// Crate names this depends on (within the workspace).
    pub depends_on: &'static [&'static str],
    /// Whether this crate produces artifacts consumed by other crates.
    pub is_producer: bool,
}

/// All workspace member crates with their tier classification.
///
/// Mirrors the `[workspace] members` list in the root `Cargo.toml`.
pub fn workspace_crates() -> Vec<CrateSpec> {
    vec![
        // ── Foundation (5) ──────────────────────────────────────────────
        CrateSpec {
            name: "core/infra",
            tier: CrateTier::Foundation,
            description: "Hashing, manifests, resource IDs",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "core/tool-registry",
            tier: CrateTier::Foundation,
            description: "Tool auto-discovery (inventory)",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "core/tool-registry-macros",
            tier: CrateTier::Foundation,
            description: "Proc-macros for tool-registry",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "core/testgen-registry",
            tier: CrateTier::Foundation,
            description: "Testgen target registry",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "core/testgen-registry-macros",
            tier: CrateTier::Foundation,
            description: "Proc-macros for testgen-registry",
            depends_on: &[],
            is_producer: false,
        },
        // ── Core (17) ──────────────────────────────────────────────────
        CrateSpec {
            name: "core/ir",
            tier: CrateTier::Core,
            description: "IR types, DAG, TypeOp, resources",
            depends_on: &["core/infra"],
            is_producer: false,
        },
        CrateSpec {
            name: "core/exec",
            tier: CrateTier::Core,
            description: "DAG executor and loop runtime",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "core/cli",
            tier: CrateTier::Core,
            description: "CLI argument parsing",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "core/codegen",
            tier: CrateTier::Core,
            description: "Code generation framework",
            depends_on: &["core/ir", "core/exec", "core/infra"],
            is_producer: true,
        },
        CrateSpec {
            name: "core/test",
            tier: CrateTier::Core,
            description: "Test infrastructure",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "core/delegate-macros",
            tier: CrateTier::Core,
            description: "Delegation proc-macros",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "core/daglang/daglang-syntax",
            tier: CrateTier::Core,
            description: "DSL lexer and parser",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "core/daglang/daglang-contract",
            tier: CrateTier::Core,
            description: "DSL contract types",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "core/daglang/daglang-resolve",
            tier: CrateTier::Core,
            description: "DSL module resolution",
            depends_on: &["core/daglang/daglang-syntax"],
            is_producer: false,
        },
        CrateSpec {
            name: "core/daglang/daglang-typecheck",
            tier: CrateTier::Core,
            description: "DSL type checking",
            depends_on: &["core/daglang/daglang-syntax", "core/daglang/daglang-resolve"],
            is_producer: false,
        },
        CrateSpec {
            name: "core/daglang/daglang-lower",
            tier: CrateTier::Core,
            description: "DSL lowering to IR",
            depends_on: &["core/daglang/daglang-syntax", "core/daglang/daglang-typecheck", "core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "core/daglang/daglang-derive",
            tier: CrateTier::Core,
            description: "DSL derive macros",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "core/daglang/daglang-emit",
            tier: CrateTier::Core,
            description: "DSL code emission",
            depends_on: &["core/daglang/daglang-syntax", "core/daglang/daglang-typecheck"],
            is_producer: true,
        },
        CrateSpec {
            name: "core/daglang/daglang-driver",
            tier: CrateTier::Core,
            description: "DSL compiler driver",
            depends_on: &[
                "core/daglang/daglang-syntax",
                "core/daglang/daglang-resolve",
                "core/daglang/daglang-typecheck",
                "core/daglang/daglang-lower",
                "core/daglang/daglang-emit",
            ],
            is_producer: false,
        },
        CrateSpec {
            name: "core/daglang/daglang-cli",
            tier: CrateTier::Core,
            description: "DSL CLI binary",
            depends_on: &["core/daglang/daglang-driver"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/primitives",
            tier: CrateTier::Core,
            description: "Shared primitive ops (StableHashOp)",
            depends_on: &["core/infra"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/blob",
            tier: CrateTier::Core,
            description: "Blob metadata (BlobMeta)",
            depends_on: &["core/infra"],
            is_producer: false,
        },
        // ── Application (15) ──────────────────────────────────────────
        CrateSpec {
            name: "lib/git-ops",
            tier: CrateTier::Application,
            description: "Git operation library",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/gist-ops",
            tier: CrateTier::Application,
            description: "Gist operation library",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/cloud-ops",
            tier: CrateTier::Application,
            description: "Cloud operations abstraction",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/aws-ops",
            tier: CrateTier::Application,
            description: "AWS provider operations",
            depends_on: &["core/ir", "lib/cloud-ops"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/azure-ops",
            tier: CrateTier::Application,
            description: "Azure provider operations",
            depends_on: &["core/ir", "lib/cloud-ops"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/llm-ops",
            tier: CrateTier::Application,
            description: "LLM operation library",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/review",
            tier: CrateTier::Application,
            description: "Code review operations",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/design-ops",
            tier: CrateTier::Application,
            description: "Design operations",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/markdown",
            tier: CrateTier::Application,
            description: "Markdown processing",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/transport",
            tier: CrateTier::Application,
            description: "Transport operations (file, shell, HTTP)",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/gcp-ops",
            tier: CrateTier::Application,
            description: "GCP provider operations",
            depends_on: &["core/ir", "lib/cloud-ops"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/tools/gist",
            tier: CrateTier::Application,
            description: "Gist tool definition",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/tools/deps",
            tier: CrateTier::Application,
            description: "Deps tool definition",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "lib/tools/clippy",
            tier: CrateTier::Application,
            description: "Clippy tool definition",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "gunbc-dag",
            tier: CrateTier::Application,
            description: "Repo-specific DAGs and binaries",
            depends_on: &["core/ir", "core/exec", "core/codegen", "core/infra"],
            is_producer: true,
        },
    ]
}

/// Workspace invariant violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceViolation {
    /// A crate depends on a higher-tier crate (layering breach).
    LayeringBreach {
        crate_name: &'static str,
        crate_tier: CrateTier,
        dep_name: &'static str,
        dep_tier: CrateTier,
    },
    /// A dependency references a crate not in the workspace.
    UnknownDependency {
        crate_name: &'static str,
        dep_name: &'static str,
    },
}

/// Validate workspace structural invariants.
///
/// Checks:
/// - No crate depends on a higher-tier crate (Foundation < Core < Application).
/// - All declared dependencies reference known workspace members.
pub fn validate_workspace_invariants() -> Vec<WorkspaceViolation> {
    let crates = workspace_crates();
    let tier_map: BTreeMap<&str, CrateTier> = crates.iter().map(|c| (c.name, c.tier)).collect();
    let known: BTreeSet<&str> = crates.iter().map(|c| c.name).collect();

    let mut violations = Vec::new();
    for spec in &crates {
        for dep in spec.depends_on {
            if !known.contains(dep) {
                violations.push(WorkspaceViolation::UnknownDependency {
                    crate_name: spec.name,
                    dep_name: dep,
                });
                continue;
            }
            let dep_tier = tier_map[dep];
            if dep_tier > spec.tier {
                violations.push(WorkspaceViolation::LayeringBreach {
                    crate_name: spec.name,
                    crate_tier: spec.tier,
                    dep_name: dep,
                    dep_tier,
                });
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layering_invariants_hold() {
        let violations = validate_workspace_invariants();
        assert!(
            violations.is_empty(),
            "workspace layering violations: {violations:?}"
        );
    }

    #[test]
    fn workspace_crates_count_matches_cargo_toml() {
        let crates = workspace_crates();
        // Count must match [workspace] members in root Cargo.toml.
        // Read the actual file to verify.
        let cargo_toml =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml"))
                .expect("should read root Cargo.toml");

        let member_count = cargo_toml
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                // Match lines like `"core/ir",` or `"gunbc-dag",`
                trimmed.starts_with('"') && (trimmed.ends_with(',') || trimmed.ends_with('"'))
            })
            .filter(|line| {
                let stripped = line.trim().trim_end_matches(',').trim_matches('"');
                stripped.contains('/') || stripped == "gunbc-dag"
            })
            .count();

        assert_eq!(
            crates.len(),
            member_count,
            "workspace_crates() has {} entries but Cargo.toml has {} members",
            crates.len(),
            member_count,
        );
    }

    #[test]
    fn foundation_crates_have_no_internal_deps() {
        for spec in workspace_crates() {
            if spec.tier == CrateTier::Foundation {
                assert!(
                    spec.depends_on.is_empty(),
                    "Foundation crate '{}' must not have internal dependencies, found: {:?}",
                    spec.name,
                    spec.depends_on
                );
            }
        }
    }

    #[test]
    fn tier_ordering_is_consistent() {
        assert!(CrateTier::Foundation < CrateTier::Core);
        assert!(CrateTier::Core < CrateTier::Application);
        assert!(CrateTier::Foundation < CrateTier::Application);
    }

    #[test]
    fn no_duplicate_crate_names() {
        let crates = workspace_crates();
        let mut seen = BTreeSet::new();
        for spec in &crates {
            assert!(
                seen.insert(spec.name),
                "duplicate crate name in workspace_crates(): {}",
                spec.name
            );
        }
    }
}
