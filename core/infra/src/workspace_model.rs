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
            name: "core/resolve",
            tier: CrateTier::Core,
            description: "Service operation resolution",
            depends_on: &["core/ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "core/workflow",
            tier: CrateTier::Core,
            description: "Workflow planner and executor",
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
            depends_on: &[
                "core/daglang/daglang-syntax",
                "core/daglang/daglang-resolve",
            ],
            is_producer: false,
        },
        CrateSpec {
            name: "core/daglang/daglang-lower",
            tier: CrateTier::Core,
            description: "DSL lowering to IR",
            depends_on: &[
                "core/daglang/daglang-syntax",
                "core/daglang/daglang-typecheck",
                "core/ir",
            ],
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
            depends_on: &[
                "core/daglang/daglang-syntax",
                "core/daglang/daglang-typecheck",
            ],
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
        // ── Application (10) ──────────────────────────────────────────
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

// ── Generator Graph ──────────────────────────────────────────────────

/// A producer→consumer edge in the generation graph.
///
/// Represents a tool whose output artifacts are consumed by another tool
/// or crate during the build/bootstrap process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorEdge {
    /// Tool that produces the artifact.
    pub producer: &'static str,
    /// Tool or crate that consumes the artifact.
    pub consumer: &'static str,
    /// The artifact path pattern connecting them.
    pub artifact: &'static str,
}

/// Known generator edges in the workspace.
///
/// These are the canonical producer→consumer relationships derived from
/// the tool registry's `provides`/`consumes` fields and build ordering.
pub fn known_generator_edges() -> Vec<GeneratorEdge> {
    vec![
        GeneratorEdge {
            producer: "codegen",
            consumer: "bootstrap",
            artifact: "target/codegen/.stamp",
        },
        GeneratorEdge {
            producer: "codegen",
            consumer: "makegen",
            artifact: "target/codegen/.stamp",
        },
        GeneratorEdge {
            producer: "pragma",
            consumer: "clippy",
            artifact: "clippy.toml",
        },
        GeneratorEdge {
            producer: "testgen",
            consumer: "cargo-test",
            artifact: "**/generated_tests*.rs",
        },
        GeneratorEdge {
            producer: "makegen",
            consumer: "make",
            artifact: "Makefile",
        },
        GeneratorEdge {
            producer: "bootstrap",
            consumer: "make",
            artifact: ".gitignore",
        },
    ]
}

/// Check for cycles in a generator edge graph.
///
/// Returns `Some(cycle_path)` if a cycle is found, `None` if the graph is acyclic.
pub fn check_generator_cycles(edges: &[GeneratorEdge]) -> Option<Vec<&str>> {
    let mut nodes: BTreeSet<&str> = BTreeSet::new();
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in edges {
        nodes.insert(edge.producer);
        nodes.insert(edge.consumer);
        adj.entry(edge.producer).or_default().push(edge.consumer);
    }

    // DFS-based cycle detection
    let mut visited: BTreeSet<&str> = BTreeSet::new();
    let mut on_stack: BTreeSet<&str> = BTreeSet::new();
    let mut path: Vec<&str> = Vec::new();

    fn dfs<'a>(
        node: &'a str,
        adj: &BTreeMap<&'a str, Vec<&'a str>>,
        visited: &mut BTreeSet<&'a str>,
        on_stack: &mut BTreeSet<&'a str>,
        path: &mut Vec<&'a str>,
    ) -> Option<Vec<&'a str>> {
        visited.insert(node);
        on_stack.insert(node);
        path.push(node);

        if let Some(neighbors) = adj.get(node) {
            for &next in neighbors {
                if !visited.contains(next) {
                    if let Some(cycle) = dfs(next, adj, visited, on_stack, path) {
                        return Some(cycle);
                    }
                } else if on_stack.contains(next) {
                    // Found cycle — extract cycle path
                    let start = path.iter().position(|&n| n == next).unwrap();
                    let mut cycle: Vec<&str> = path[start..].to_vec();
                    cycle.push(next);
                    return Some(cycle);
                }
            }
        }

        on_stack.remove(node);
        path.pop();
        None
    }

    for &node in &nodes {
        if !visited.contains(node) {
            if let Some(cycle) = dfs(node, &adj, &mut visited, &mut on_stack, &mut path) {
                return Some(cycle);
            }
        }
    }
    None
}

/// Derive execution order from a generator edge graph (topological sort).
///
/// Returns tool names in dependency order. Tools with no dependencies come first.
/// Returns `Err` with a cycle path if the graph has cycles.
pub fn generator_execution_order(edges: &[GeneratorEdge]) -> Result<Vec<&str>, Vec<&str>> {
    if let Some(cycle) = check_generator_cycles(edges) {
        return Err(cycle);
    }

    let mut nodes: BTreeSet<&str> = BTreeSet::new();
    let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for edge in edges {
        nodes.insert(edge.producer);
        nodes.insert(edge.consumer);
        adj.entry(edge.producer).or_default().push(edge.consumer);
        *in_degree.entry(edge.consumer).or_default() += 1;
        in_degree.entry(edge.producer).or_default();
    }

    let mut queue: std::collections::VecDeque<&str> = nodes
        .iter()
        .filter(|&&n| *in_degree.get(n).unwrap_or(&0) == 0)
        .copied()
        .collect();

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        if let Some(neighbors) = adj.get(node) {
            for &next in neighbors {
                let deg = in_degree.get_mut(next).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(next);
                }
            }
        }
    }

    Ok(order)
}

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
        CommitPolicy {
            pattern: "Makefile",
            reason: CommitReason::Generated,
            producer: Some("makegen"),
        },
        // Bootstrap seed files — generated but committed
        CommitPolicy {
            pattern: ".gitignore",
            reason: CommitReason::BootstrapSeed,
            producer: Some("bootstrap"),
        },
        CommitPolicy {
            pattern: "clippy.toml",
            reason: CommitReason::BootstrapSeed,
            producer: Some("pragma"),
        },
        CommitPolicy {
            pattern: "deps.toml",
            reason: CommitReason::BootstrapSeed,
            producer: Some("deps-config"),
        },
        CommitPolicy {
            pattern: "docs/ab-writing-workflows.md",
            reason: CommitReason::BootstrapSeed,
            producer: Some("docgen"),
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

    // ── Generator graph tests ───────────────────────────────────────

    #[test]
    fn generator_graph_is_acyclic() {
        let edges = known_generator_edges();
        assert!(
            check_generator_cycles(&edges).is_none(),
            "generator graph must be acyclic"
        );
    }

    #[test]
    fn generator_execution_order_is_valid() {
        let edges = known_generator_edges();
        let order = generator_execution_order(&edges).expect("should produce valid order");
        // Producers must appear before consumers
        for edge in &edges {
            let prod_pos = order.iter().position(|&n| n == edge.producer);
            let cons_pos = order.iter().position(|&n| n == edge.consumer);
            if let (Some(p), Some(c)) = (prod_pos, cons_pos) {
                assert!(
                    p < c,
                    "producer '{}' must come before consumer '{}' in execution order",
                    edge.producer,
                    edge.consumer
                );
            }
        }
    }

    #[test]
    fn cycle_detection_finds_cycles() {
        let edges = vec![
            GeneratorEdge {
                producer: "a",
                consumer: "b",
                artifact: "x",
            },
            GeneratorEdge {
                producer: "b",
                consumer: "c",
                artifact: "y",
            },
            GeneratorEdge {
                producer: "c",
                consumer: "a",
                artifact: "z",
            },
        ];
        let cycle = check_generator_cycles(&edges);
        assert!(cycle.is_some(), "should detect cycle a→b→c→a");
    }

    #[test]
    fn cycle_detection_returns_none_for_dag() {
        let edges = vec![
            GeneratorEdge {
                producer: "a",
                consumer: "b",
                artifact: "x",
            },
            GeneratorEdge {
                producer: "a",
                consumer: "c",
                artifact: "y",
            },
            GeneratorEdge {
                producer: "b",
                consumer: "c",
                artifact: "z",
            },
        ];
        assert!(check_generator_cycles(&edges).is_none());
    }

    // ── Commit policy tests ─────────────────────────────────────────

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
        // Seeds should NOT appear
        assert!(
            !gitignore.contains(".gitignore"),
            ".gitignore is a seed and should not be in derived gitignore"
        );
        // Build output should appear
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

    // ── Toolchain requirement tests ─────────────────────────────────

    #[test]
    fn toolchain_requirements_include_rustc() {
        let reqs = toolchain_requirements();
        assert!(reqs.iter().any(|r| r.tool == "rustc"), "must require rustc");
    }

    #[test]
    fn producer_crates_exist_in_workspace() {
        let crates = workspace_crates();
        let producers: Vec<_> = crates.iter().filter(|c| c.is_producer).collect();
        assert!(
            !producers.is_empty(),
            "workspace should have at least one producer crate"
        );
        for p in &producers {
            assert!(
                crates.iter().any(|c| c.name == p.name),
                "producer '{}' must be in workspace_crates()",
                p.name
            );
        }
    }
}
