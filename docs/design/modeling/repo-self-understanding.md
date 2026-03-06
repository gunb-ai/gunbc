# Design: Repository Self-Understanding Model

**Status**: Draft
**Date**: 2026-02-23
**Related tasks**: M20, M14, M18
**Reference**: `the-gunbai` `understanding/repo.rs`, `understanding/workspace.rs`,
`understanding/generator.rs`, `understanding/codegen_layering.rs`, `understanding/recipe.rs`

## Problem

gunbc's self-knowledge is **scattered across code, memory files, and human context**.
The repo already has strong self-modeling primitives — `#[tool_target]` inventory,
`DagSpecDef` registry, `@outputs` annotations, content_upsert chains, freshness
manifests — but they're independent systems with no unifying model. The repo doesn't
model its own structure as data the way it models external systems.

### What's scattered today

| Self-knowledge | Where it lives | Problem |
|----------------|----------------|---------|
| Crate dependency graph | Cargo.toml files (16+ crates) | Implicit; no layering enforcement |
| Bootstrap ordering | Human knowledge (codegen → bootstrap → makegen) | Not modeled; not testable |
| Tool→artifact→tool edges | `#[tool_target]` outputs + ad-hoc knowledge | No producer/consumer graph |
| Commit policies | `.gitignore` (handwritten) + seed file exceptions | No canonical source |
| Toolchain requirements | GNUmakefile + CI scripts | Duplicated; can drift |
| Build commands | `BuildConfig` in makegen/registry.rs | String-based; not structured |
| Crate tier classification | MEMORY.md (agent memory!) | Not in code at all |

### the-gunbai's solution

the-gunbai solves this with a `understanding/` module that captures meta-knowledge
about the repo itself. The philosophy:

> "Instead of hardcoding rules in .gitignore, deps.toml, CI configs, etc., we capture
> our understanding of 'what this repo is' in one place. Then we generate/validate
> everything else from that understanding."

Key components:
- **`repo.rs`**: `ToolDependency`, `CommitPolicy`, `MakeTargetSpec` — all derived from
  one canonical model
- **`workspace.rs`**: `CrateTier` (Foundation/Bootstrap/Generated), bootstrap validation,
  tier derivation from dependency graph
- **`generator.rs`**: `GeneratorUnderstanding`, `GeneratorProducer` with DG0 dedup model —
  all_generators() / all_producers() central functions
- **`codegen_layering.rs`**: `BuildLayer` (Contracts < Codegen < Full), layer enforcement,
  producer/consumer edges derived from generator targets
- **`recipe.rs`**: `RecipeStep` structured enum replacing raw shell strings

## Design: gunbc Self-Understanding (DSL-First)

### Principle: Model the repo with the same tools used to model external systems

gunbc's compositional modeling philosophy says external systems are compositions of
layered concerns, each imposing invariants on generated code. The repo itself is an
external system from the compiler's perspective — it has structure, constraints,
dependencies, and generated artifacts. Model it the same way.

### Component 1: Workspace Model

A typed model of the crate graph with tier classification. Adapted from
gunbai's `workspace.rs` and `codegen_layering.rs`.

```rust
/// Crate tier in the bootstrap hierarchy.
///
/// Determines what a crate can depend on and whether it may use
/// generated code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrateTier {
    /// Leaf infrastructure — no internal dependencies, no generated code.
    /// Examples: gunbc-infra, gunbc-tool-registry, gunbc-testgen-registry
    Foundation,

    /// Depends on Foundation crates; may be a generator producer.
    /// Must have committed targets (no generated main.rs).
    /// Examples: gunbc-ir, gunbc-exec, gunbc-codegen
    Core,

    /// Depends on Foundation + Core; consumes generated artifacts.
    /// Examples: gunbc-app (consumes codegen output, testgen output)
    Application,
}

/// Per-crate specification in the workspace model.
pub struct CrateSpec {
    pub name: &'static str,
    pub tier: CrateTier,
    pub description: &'static str,
    /// Crates this one directly depends on (workspace deps only).
    pub depends_on: &'static [&'static str],
    /// Whether this crate is a generator producer (runs codegen).
    pub is_producer: bool,
}
```

**Canonical workspace registry:**

```rust
pub fn workspace_crates() -> Vec<CrateSpec> {
    vec![
        // Foundation tier — leaf crates, no internal deps
        CrateSpec {
            name: "gunbc-infra",
            tier: CrateTier::Foundation,
            description: "ResourceId, hash, manifest, freshness",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "gunbc-tool-registry",
            tier: CrateTier::Foundation,
            description: "inventory-based tool auto-discovery",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "gunbc-tool-registry-macros",
            tier: CrateTier::Foundation,
            description: "#[tool_target] proc macro",
            depends_on: &[],
            is_producer: false,
        },
        CrateSpec {
            name: "gunbc-testgen-registry",
            tier: CrateTier::Foundation,
            description: "inventory-based testgen target auto-discovery",
            depends_on: &[],
            is_producer: false,
        },
        // ...

        // Core tier — depends on Foundation, may be producers
        CrateSpec {
            name: "gunbc-ir",
            tier: CrateTier::Core,
            description: "Graph IR, type system, patterns, transport model",
            depends_on: &["gunbc-infra"],
            is_producer: false,
        },
        CrateSpec {
            name: "gunbc-exec",
            tier: CrateTier::Core,
            description: "Execution engine, DryRun, simulation",
            depends_on: &["gunbc-ir"],
            is_producer: false,
        },
        CrateSpec {
            name: "gunbc-codegen",
            tier: CrateTier::Core,
            description: "CLI and test generation from tool registry",
            depends_on: &["gunbc-ir", "gunbc-exec", "gunbc-infra"],
            is_producer: true,  // produces CLI entrypoints
        },
        // ...

        // Application tier — consumes generated artifacts
        CrateSpec {
            name: "gunbc-app",
            tier: CrateTier::Application,
            description: "Repo-specific DAGs, CLI entrypoints, tool workflows",
            depends_on: &["gunbc-ir", "gunbc-exec", "gunbc-codegen", "gunbc-infra",
                          "gunbc-tool-registry", "gunbc-testgen-registry",
                          "gunbc-lib-transport", "gunbc-lib-tools-gist",
                          "gunbc-lib-tools-deps", "gunbc-lib-tools-review",
                          "gunbc-lib-primitives", "gunbc-lib-blob"],
            is_producer: false,
        },
    ]
}
```

**Invariant enforcement:**

```rust
/// Validate workspace tier invariants.
pub fn validate_workspace_invariants(specs: &[CrateSpec]) -> Vec<WorkspaceViolation> {
    let mut violations = Vec::new();

    for spec in specs {
        for dep_name in spec.depends_on {
            if let Some(dep) = specs.iter().find(|s| s.name == *dep_name) {
                // Core cannot depend on Application
                // Foundation cannot depend on Core or Application
                if dep.tier > spec.tier {
                    violations.push(WorkspaceViolation::LayerViolation {
                        crate_name: spec.name,
                        depends_on: dep.name,
                        crate_tier: spec.tier,
                        dep_tier: dep.tier,
                    });
                }
            }
        }
    }

    violations
}
```

**Test: workspace model matches Cargo.toml:**

```rust
#[test]
fn workspace_model_matches_cargo_toml() {
    let model = workspace_crates();
    let cargo_members = parse_workspace_members(); // read root Cargo.toml

    // Every workspace member must have a CrateSpec
    for member in &cargo_members {
        assert!(
            model.iter().any(|s| s.name == member),
            "workspace member {} missing from workspace_crates()",
            member
        );
    }

    // Every CrateSpec must be a workspace member
    for spec in &model {
        assert!(
            cargo_members.contains(&spec.name.to_string()),
            "CrateSpec {} not in Cargo.toml workspace members",
            spec.name
        );
    }
}
```

### Component 2: Generator Graph

A model of which tools produce which artifacts and which tools consume them.
Adapted from gunbai's `generator.rs` and DG0 producer-centric model.

gunbc already has `#[tool_target]` with `outputs` fields. The missing piece is
**producer→consumer edges** — which tools' outputs feed into other tools' inputs.

```rust
/// A producer→consumer edge in the generation graph.
///
/// Derived from tool registrations: the producer's declared outputs
/// are matched against the consumer's declared inputs (or Cargo source
/// directories).
pub struct GeneratorEdge {
    /// Tool that produces the artifact.
    pub producer: &'static str,
    /// Tool or crate that consumes the artifact.
    pub consumer: &'static str,
    /// The artifact path pattern connecting them.
    pub artifact: &'static str,
}

/// Derive producer→consumer edges from tool registrations.
pub fn derive_generator_edges() -> Vec<GeneratorEdge> {
    let tools: Vec<_> = iter_tool_targets().collect();
    let mut edges = Vec::new();

    for producer in &tools {
        for output in &producer.outputs {
            for consumer in &tools {
                if consumer.name != producer.name
                    && consumer_needs_artifact(consumer, output)
                {
                    edges.push(GeneratorEdge {
                        producer: producer.name,
                        consumer: consumer.name,
                        artifact: output,
                    });
                }
            }
        }
    }

    edges
}
```

**Known edges in gunbc today:**

```
codegen → CLI entrypoints → bootstrap (consumes CLI binaries)
codegen → CLI entrypoints → makegen (consumes tool registry)
pragma → clippy.toml → all crates (consumed by cargo clippy)
testgen → generated_tests*.rs → test crates (consumed by cargo test)
bootstrap → Makefile, .gitignore → developer workflow (consumed by make)
makegen → Makefile → developer workflow (consumed by make)
```

**Cycle detection:**

```rust
/// Check for cycles in the generator graph.
pub fn check_generator_cycles(edges: &[GeneratorEdge]) -> Option<Vec<&str>> {
    // Topological sort; return cycle path if found.
    // A cycle means the build system cannot bootstrap.
}
```

**Execution ordering:**

```rust
/// Derive execution order from generator graph (topological sort).
pub fn generator_execution_order(edges: &[GeneratorEdge]) -> Vec<&str> {
    // Returns tool names in dependency order.
    // Independent tools can be parallelized.
}
```

### Component 3: Commit Policy Model

A canonical source for what should/shouldn't be in git. Adapted from
gunbai's `CommitPolicy` in `repo.rs`.

```rust
/// Commit policy for a file pattern.
pub struct CommitPolicy {
    /// Glob pattern (gitignore-style).
    pub pattern: &'static str,
    /// Why this pattern exists.
    pub reason: CommitReason,
    /// The tool or system that generates this artifact.
    pub producer: Option<&'static str>,
}

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

/// All commit policies, derived from tool registrations + conventions.
pub fn commit_policies() -> Vec<CommitPolicy> {
    let mut policies = Vec::new();

    // From tool registrations
    for tool in iter_tool_targets() {
        for output in &tool.outputs {
            policies.push(CommitPolicy {
                pattern: output,
                reason: CommitReason::Generated,
                producer: Some(tool.name),
            });
        }
    }

    // Build artifacts
    policies.push(CommitPolicy {
        pattern: "target/",
        reason: CommitReason::BuildOutput,
        producer: None,
    });

    // Bootstrap seeds (generated but committed)
    for seed in COMMITTED_SEED_FILES {
        policies.push(CommitPolicy {
            pattern: seed,
            reason: CommitReason::BootstrapSeed,
            producer: Some("bootstrap"),
        });
    }

    policies
}
```

**.gitignore derivation:**

```rust
/// Generate .gitignore content from commit policies.
pub fn derive_gitignore(policies: &[CommitPolicy]) -> String {
    let mut lines = vec!["# Auto-generated from commit policies. Do not edit.".to_string()];

    for policy in policies {
        if matches!(policy.reason, CommitReason::Generated | CommitReason::BuildOutput) {
            lines.push(policy.pattern.to_string());
        }
    }

    lines.join("\n")
}
```

This replaces the handwritten `.gitignore` with a derived one, and the
`all_tool_outputs_gitignored` test becomes a policy-model validation instead
of an ad-hoc check.

### Component 4: Toolchain Requirements

Canonical declaration of toolchain requirements, replacing scattered GNUmakefile
and CI script checks.

```rust
/// A toolchain requirement for building/running the repo.
pub struct ToolchainRequirement {
    pub tool: &'static str,
    pub min_version: Option<&'static str>,
    pub purpose: &'static str,
    pub install_hint: &'static str,
}

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
```

### Where it lives

Two options, each with trade-offs:

**Option A: DSL module (`dsl/meta/workspace.dag`)**

Model the workspace in the DSL itself. This is the "eat your own dog food"
approach — the repo's self-model is a DAG like everything else.

```dag
// dsl/meta/workspace.dag

type CrateTier = Foundation | Core | Application

type CrateSpec {
    name: String
    tier: CrateTier
    description: String
    depends_on: List<String>
    is_producer: Bool
}

// Canonical workspace model
const WORKSPACE: List<CrateSpec> = [
    { name: "gunbc-infra", tier: Foundation, ... },
    { name: "gunbc-ir", tier: Core, ... },
    ...
]
```

Pro: Fully compositional, uses the DSL type system, validates via the same
pipeline. Con: The DSL compiler must be able to build before the workspace
model exists (bootstrap chicken-and-egg).

**Option B: Rust module (`core/infra/src/workspace_model.rs`)**

Model in Foundation-tier Rust code. This is simpler and avoids the bootstrap
problem.

Pro: No bootstrap issue, can be used by codegen/testgen directly.
Con: Not authored in the DSL (but the DSL can import it as a typed input).

**Recommendation**: Option B for the initial implementation. The workspace model
is Foundation-tier data that must exist before the DSL compiler runs. Once the
model is stable, a DSL surface can be added on top for documentation and
validation purposes.

## Scope

### In scope
- Workspace crate model with tier classification and layering validation
- Generator graph (producer→consumer edges, cycle detection, execution ordering)
- Commit policy model replacing handwritten .gitignore
- Toolchain requirements model
- Tests proving workspace model matches Cargo.toml
- Tests proving generator graph is acyclic

### Not in scope (future work)
- Structured recipe steps (gunbai's `RecipeStep`) — separate task, builds on this
- DSL surface for the workspace model — after Rust model is stable
- CI pipeline modeling as data — after generator graph is proven
- Cross-repo modeling (dependencies on the-gunbai) — different concern

## Relationship to Other Tasks

- **M14** (single inventory authority): The workspace model + generator graph IS the
  single inventory authority. M14 can derive from it instead of building a separate
  registration model.
- **M18** (single semantic authority / projection-only): The workspace model is the
  canonical source; Makefile, .gitignore, CI scripts are projections derived from it.
- **M13** (registry→CLI→Make contract tests): Contract tests can validate that the
  generator graph's derived execution order matches actual Make target dependencies.
- **Foundation Close-Out**: This is both Lane A (one model, not scattered knowledge)
  and Lane B (invariants machine-enforced — layering violations fail tests).

## Migration Path

1. **Phase 0**: Add `workspace_model.rs` to `core/infra` with `CrateSpec`,
   `CrateTier`, `workspace_crates()`. Add layering validation tests. No behavioral
   change — purely additive.

2. **Phase 1**: Add generator edge derivation from `iter_tool_targets()`. Add cycle
   detection and execution ordering. Wire into existing `tool_registration.rs` tests.

3. **Phase 2**: Add commit policy model. Derive `.gitignore` from policies. Replace
   `all_tool_outputs_gitignored` test with policy-model validation.

4. **Phase 3**: Add toolchain requirements model. Wire into `make install` / bootstrap
   validation. Replace scattered version checks with canonical source.
