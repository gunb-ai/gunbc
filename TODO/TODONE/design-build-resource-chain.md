# Design: Build Resource Chain

> Resolves: Manual dependency wiring in Makefile generation (extra_deps, fix_deps,
> PrepLevel), testgen-check vs testgen duality, and the lack of automatic
> "ensure" chains for build artifacts.
>
> Principle: **Build artifacts are resources acquired via DAG execution.**
> Each DAG declares what it provides and needs. The executor resolves the
> dependency graph and flows mode (verify/ensure) through the chain.

## Ownership
- [ ] Unassigned

## Priority
**URGENT** — This design addresses fundamental dependency wiring issues that cause
bugs like `test-fix` failing on stale tests instead of regenerating them.

## 1. Problem Statement

Today, build artifact dependencies are wired manually across multiple layers:

```
LAYER 1: PrepLevel enum (coarse)
  PrepLevel::Full → depends on "build" (or "codegen" if DAG entrypoints)
  PrepLevel::Codegen → depends on "ensure-codegen"
  PrepLevel::None → no dependency

LAYER 2: extra_deps (manual bolt-on)
  test → extra_deps: ["testgen-check"]

LAYER 3: fix_deps (separate list for -fix variants)
  test-fix → fix_deps: ["fmt-fix", "lint-fix"]

LAYER 4: Render-time routing
  if config.use_dag_entrypoints { "codegen" } else { "build" }
```

This causes real bugs:
- `make test-fix` inherits `testgen-check` (fails on stale) instead of `testgen` (regenerates)
- Adding a new build stage requires updating multiple places
- The verify vs ensure duality is encoded as separate targets, not a mode flag

The root cause: **build artifacts aren't modeled as resources.** There's no way
for a DAG to say "I need generated tests to exist" and have the system figure
out how to satisfy that need.

### Current Bandaids

| Location | Bandaid | Why It Exists |
|----------|---------|---------------|
| `ensure-codegen` vs `codegen` | Two targets | Bootstrap can't use DAG |
| `testgen` vs `testgen-check` | Two targets | Verify vs generate duality |
| `extra_deps: ["testgen-check"]` | Manual wiring | No resource model |
| `fix_deps: ["fmt-fix", "lint-fix"]` | Separate list | -fix needs different deps |
| `use_dag_entrypoints` routing | Config check | PrepLevel::Full means different things |

## 2. Design Principles

Extending the resource acquisition model:

- **Build artifacts are resources.** Generated CLI, generated tests, compiled
  code, formatted code — all are resources acquired via DAG execution.
- **DAGs declare provides/needs.** Each DAG explicitly states what resource it
  produces and what resources it requires.
- **Mode flows through the chain.** A single `ExecMode` (Verify/Ensure) flag
  propagates to all dependencies, determining check-only vs regenerate behavior.
- **Resolution is automatic.** The executor builds the dependency graph from
  provides/needs declarations — no manual wiring.
- **Makefile becomes a thin shell.** `make test` and `make test-fix` invoke
  the same DAG with different mode flags.

## 3. Build Resource Taxonomy

### 3.1 Build Resource Types

| Resource | Provider DAG | Staleness Check | Ensure Action |
|----------|--------------|-----------------|---------------|
| `GeneratedCli` | CodegenDAG | Files exist in target/codegen/bin/ | Run bootstrapper |
| `GeneratedTests` | TestgenDAG | Compare generated vs source | Regenerate test files |
| `FormattedCode` | FmtDAG | `cargo fmt --check` exit code | Run `cargo fmt` |
| `LintedCode` | ClippyDAG | `cargo clippy` exit code | Run `cargo clippy --fix` |
| `CompiledCode` | BuildDAG | `cargo build` exit code | Run `cargo build` |
| `TypeChecked` | CheckDAG | `cargo check` exit code | Run `cargo check` |
| `TestedCode` | TestDAG | `cargo test` exit code | Run `cargo test` |

### 3.2 Resource State

```rust
/// State of a build resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState {
    /// Resource doesn't exist or has never been generated.
    Missing,
    /// Resource exists but is out of date.
    Stale { reason: String },
    /// Resource exists and is up to date.
    Fresh,
}
```

### 3.3 Execution Mode

```rust
/// How to handle resource acquisition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecMode {
    /// Check that resources are fresh, fail if stale (CI mode).
    Verify,
    /// Ensure resources are fresh, regenerate if stale (dev mode).
    Ensure,
}
```

## 4. Resource Declaration

### 4.1 DAG Metadata

Each DAG declares its resource contract:

```rust
/// Metadata about a DAG's resource requirements.
pub struct DagResourceContract {
    /// Resource this DAG provides (if any).
    pub provides: Option<BuildResource>,
    /// Resources this DAG needs before it can run.
    pub needs: Vec<BuildResource>,
}

/// A build resource identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildResource {
    GeneratedCli,
    GeneratedTests,
    FormattedCode,
    LintedCode,
    CompiledCode,
    TypeChecked,
    TestedCode,
}
```

### 4.2 Registration in Tool Registry

```rust
// In core/codegen/src/registry.rs

ToolDef::new("testgen", "Generate tests from DAG structures")
    .provides(BuildResource::GeneratedTests)
    .needs(vec![BuildResource::GeneratedCli])
    // ...

ToolDef::new("test", "Run all tests")
    .provides(BuildResource::TestedCode)
    .needs(vec![
        BuildResource::GeneratedCli,
        BuildResource::GeneratedTests,
        BuildResource::CompiledCode,
    ])
    // ...
```

### 4.3 Dependency Graph Resolution

Given a target resource and mode, resolve the full dependency chain:

```rust
/// Resolve the execution order for acquiring a resource.
pub fn resolve_dependencies(
    target: BuildResource,
    mode: ExecMode,
    registry: &ResourceRegistry,
) -> Vec<(BuildResource, &DagBuilder)> {
    // Topological sort of the dependency graph
    // Returns providers in execution order
}
```

Example resolution for `BuildResource::TestedCode`:

```
TestedCode
  ├─ needs: CompiledCode
  │    └─ needs: GeneratedCli
  ├─ needs: GeneratedTests
  │    └─ needs: GeneratedCli (already resolved)
  └─ needs: GeneratedCli (already resolved)

Execution order: [GeneratedCli, GeneratedTests, CompiledCode, TestedCode]
```

## 5. Execution Flow

### 5.1 Resource-Aware Executor

```rust
impl ResourceExecutor {
    pub fn execute(
        &self,
        target: BuildResource,
        mode: ExecMode,
    ) -> Result<(), ResourceError> {
        let chain = resolve_dependencies(target, mode, &self.registry);

        for (resource, dag_builder) in chain {
            let state = self.check_resource_state(resource)?;

            match (state, mode) {
                // Fresh in any mode: skip
                (ResourceState::Fresh, _) => continue,

                // Missing/Stale in Ensure mode: regenerate
                (ResourceState::Missing | ResourceState::Stale { .. }, ExecMode::Ensure) => {
                    let dag = dag_builder();
                    self.executor.execute(&dag)?;
                }

                // Missing/Stale in Verify mode: fail
                (ResourceState::Missing, ExecMode::Verify) => {
                    return Err(ResourceError::Missing { resource });
                }
                (ResourceState::Stale { reason }, ExecMode::Verify) => {
                    return Err(ResourceError::Stale { resource, reason });
                }
            }
        }

        Ok(())
    }
}
```

### 5.2 Staleness Checking

Each resource type defines its staleness check:

```rust
impl BuildResource {
    pub fn check_state(&self, ctx: &CheckContext) -> ResourceState {
        match self {
            BuildResource::GeneratedCli => {
                // Check if all expected files exist
                let paths = expected_codegen_paths();
                if paths.iter().all(|p| p.exists()) {
                    ResourceState::Fresh
                } else {
                    ResourceState::Missing
                }
            }
            BuildResource::GeneratedTests => {
                // Run testgen --check, parse exit code
                match ctx.run_testgen_check() {
                    Ok(()) => ResourceState::Fresh,
                    Err(stale_files) => ResourceState::Stale {
                        reason: format!("Stale: {}", stale_files.join(", ")),
                    },
                }
            }
            BuildResource::FormattedCode => {
                // Run cargo fmt --check
                match ctx.run_fmt_check() {
                    Ok(()) => ResourceState::Fresh,
                    Err(_) => ResourceState::Stale {
                        reason: "Code not formatted".into(),
                    },
                }
            }
            // ... etc
        }
    }
}
```

## 6. Makefile Simplification

### 6.1 Before (Current State)

```makefile
# Complex dependency wiring
test: codegen testgen-check
	@cargo run -p gunbc-dag --bin gunbc-build --release

test-fix: fmt-fix lint-fix codegen testgen-check
	@cargo run -p gunbc-dag --bin gunbc-build --release
```

### 6.2 After (With Resource Model)

```makefile
# Simple mode flags
test:
	@cargo run -p gunbc-dag --bin gunbc-build --release -- --mode=verify

test-fix:
	@cargo run -p gunbc-dag --bin gunbc-build --release -- --mode=ensure
```

The build DAG internally:
1. Declares `needs: [GeneratedCli, GeneratedTests, CompiledCode, ...]`
2. Mode flag propagates to all dependency checks
3. Verify mode fails fast on stale resources
4. Ensure mode regenerates as needed

### 6.3 Meta Target Generation

```rust
fn render_meta_target(meta: &MetaTarget, config: &BuildConfig) -> String {
    // No more extra_deps, fix_deps, PrepLevel routing
    // Just emit the target with appropriate mode flag

    let base = format!(
        "{name}:\n\t@{command} --mode=verify\n\n",
        name = meta.name,
        command = meta.get_command(config),
    );

    let fix = if meta.has_fix_variant {
        format!(
            "{name}-fix:\n\t@{command} --mode=ensure\n\n",
            name = meta.name,
            command = meta.get_command(config),
        )
    } else {
        String::new()
    };

    format!("{}{}", base, fix)
}
```

## 7. Integration with Existing Resource Model

The build resource model **extends** the existing resource acquisition model
(design-resource-acquisition.md), not replaces it.

| Aspect | Runtime Resources | Build Resources |
|--------|-------------------|-----------------|
| Examples | ToolHandle, FsHandle, Platform | GeneratedCli, GeneratedTests |
| Acquired by | Environment nodes (EnvOp) | Provider DAGs (CodegenDAG, etc.) |
| Flows through | Edges as values | Implicit dependency chain |
| Staleness | N/A (acquired fresh) | Check via commands/file comparison |
| Mode | Real/DryRun | Verify/Ensure |

The key insight: runtime resources flow through **values on edges**, while
build resources flow through **implicit dependency resolution**. Both follow
the principle that dependencies are declared, not constructed inline.

## 8. Implementation Plan

### Phase 1: Core Infrastructure

**Files to create:**
- `core/ir/src/build_resource.rs` — BuildResource enum, ResourceState, ExecMode

**Files to modify:**
- `core/ir/src/lib.rs` — Export new types

```rust
// core/ir/src/build_resource.rs

/// A build-time resource (artifact from a build stage).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildResource {
    GeneratedCli,
    GeneratedTests,
    FormattedCode,
    LintedCode,
    CompiledCode,
    TypeChecked,
    TestedCode,
}

/// State of a build resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState {
    Missing,
    Stale { reason: String },
    Fresh,
}

/// Execution mode for resource acquisition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecMode {
    #[default]
    Verify,
    Ensure,
}

/// A DAG's resource contract.
#[derive(Debug, Clone, Default)]
pub struct DagResourceContract {
    pub provides: Option<BuildResource>,
    pub needs: Vec<BuildResource>,
}
```

### Phase 2: Tool Registry Integration

**Files to modify:**
- `core/codegen/src/registry.rs` — Add provides/needs to ToolDef

```rust
// Add to ToolDef
pub struct ToolDef {
    // ... existing fields
    pub resource_contract: DagResourceContract,
}

impl ToolDef {
    pub fn provides(mut self, resource: BuildResource) -> Self {
        self.resource_contract.provides = Some(resource);
        self
    }

    pub fn needs(mut self, resources: Vec<BuildResource>) -> Self {
        self.resource_contract.needs = resources;
        self
    }
}
```

**Update existing tool definitions:**
```rust
// In all_tools()

ToolDef::new("codegen", "Generate CLI entrypoints")
    .provides(BuildResource::GeneratedCli)
    .needs(vec![])  // bootstrap, no deps
    // ...

ToolDef::new("testgen", "Generate tests from DAGs")
    .provides(BuildResource::GeneratedTests)
    .needs(vec![BuildResource::GeneratedCli])
    // ...
```

### Phase 3: Resource Registry and Resolver

**Files to create:**
- `gunbc-dag/src/resource_registry.rs` — ResourceRegistry, dependency resolver

```rust
// gunbc-dag/src/resource_registry.rs

use gunbc_ir::{BuildResource, DagResourceContract};
use std::collections::HashMap;

pub struct ResourceRegistry {
    providers: HashMap<BuildResource, ProviderInfo>,
}

struct ProviderInfo {
    contract: DagResourceContract,
    dag_builder: fn() -> Box<dyn Fn() -> Dag<WorkspaceOp>>,
}

impl ResourceRegistry {
    /// Build registry from tool definitions.
    pub fn from_tools() -> Self { ... }

    /// Resolve dependency chain for a target resource.
    pub fn resolve(&self, target: BuildResource) -> Vec<BuildResource> {
        // Topological sort
    }
}
```

### Phase 4: Staleness Checking

**Files to modify:**
- `gunbc-dag/src/codegen/ops.rs` — Extract staleness check as reusable function
- `gunbc-dag/src/bin/testgen.rs` — Extract staleness check

**Files to create:**
- `gunbc-dag/src/staleness.rs` — Unified staleness checking

```rust
// gunbc-dag/src/staleness.rs

impl BuildResource {
    pub fn check_state(&self) -> ResourceState {
        match self {
            BuildResource::GeneratedCli => check_codegen_state(),
            BuildResource::GeneratedTests => check_testgen_state(),
            BuildResource::FormattedCode => check_fmt_state(),
            // ...
        }
    }

    pub fn ensure(&self) -> Result<(), EnsureError> {
        match self {
            BuildResource::GeneratedCli => run_codegen(),
            BuildResource::GeneratedTests => run_testgen(),
            BuildResource::FormattedCode => run_fmt(),
            // ...
        }
    }
}
```

### Phase 5: Resource-Aware Executor

**Files to modify:**
- `gunbc-dag/src/bin/build.rs` — Add --mode flag, use ResourceExecutor

```rust
// gunbc-dag/src/bin/build.rs

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "verify")]
    mode: ExecMode,

    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let registry = ResourceRegistry::from_tools();
    let executor = ResourceExecutor::new(registry, args.mode);

    // The build DAG's needs are resolved automatically
    executor.execute(BuildResource::TestedCode)?;

    Ok(())
}
```

### Phase 6: Makefile Simplification

**Files to modify:**
- `gunbc-dag/src/makegen/registry.rs` — Remove extra_deps, fix_deps, simplify PrepLevel
- `gunbc-dag/src/makegen/render.rs` — Emit mode flags instead of dependency chains

```rust
// Simplified MetaTarget (no more extra_deps, fix_deps)
pub struct MetaTarget {
    pub name: String,
    pub description: String,
    pub target_resource: BuildResource,  // What this target produces
    pub has_fix_variant: bool,
}

// Render becomes trivial
fn render_meta_target(meta: &MetaTarget, config: &BuildConfig) -> String {
    format!(
        "# {name}: {desc}\n{name}:\n\t@{cmd} --mode=verify\n\n",
        name = meta.name,
        desc = meta.description,
        cmd = meta.get_command(config),
    )
}
```

### Phase 7: Cleanup

**Files to modify:**
- `gunbc-dag/src/makegen/registry.rs` — Remove:
  - `PrepLevel` enum (replaced by resource needs)
  - `extra_deps` field
  - `fix_deps` field
  - `prep_dep_name()` function
  - `meta_target_deps()` function

- `gunbc-dag/src/makegen/render.rs` — Remove:
  - `render_fix_alias_targets()` (fmt-fix, lint-fix handled by mode)
  - Complex dependency wiring in `render_meta_fix_variant()`

## 9. Migration Path

### Step 1: Add Infrastructure (Non-Breaking)
- Add BuildResource, ResourceState, ExecMode to core/ir
- Add provides/needs to ToolDef (optional fields)
- No behavior change yet

### Step 2: Populate Resource Contracts
- Update all_tools() with provides/needs declarations
- Add ResourceRegistry
- Still no behavior change in Makefile

### Step 3: Add --mode Flag to Build
- gunbc-build accepts --mode=verify|ensure
- Default to verify (matches current `make test` behavior)
- Makefile unchanged, but can use new flag

### Step 4: Switch Makefile to Mode Flags
- Update render.rs to emit --mode flags
- Remove extra_deps/fix_deps wiring
- `make test` → `--mode=verify`
- `make test-fix` → `--mode=ensure`

### Step 5: Cleanup Legacy Code
- Remove PrepLevel, extra_deps, fix_deps
- Remove fix alias targets
- Simplify MetaTarget structure

## 10. Testing Strategy

### Unit Tests
- `ResourceRegistry::resolve()` returns correct topological order
- `BuildResource::check_state()` correctly identifies Fresh/Stale/Missing
- `ExecMode::Verify` fails on stale, `ExecMode::Ensure` regenerates

### Integration Tests
- `make test` with stale testgen → fails with clear message
- `make test-fix` with stale testgen → regenerates then runs tests
- Dependency chain: codegen → testgen → build → test works correctly

### Regression Tests
- Verify all existing Makefile targets still work
- CI pipeline passes with new implementation

## 11. Checklist

### Phase 1: Core Infrastructure
- [ ] Create `core/ir/src/build_resource.rs`
- [ ] Add `BuildResource` enum
- [ ] Add `ResourceState` enum
- [ ] Add `ExecMode` enum
- [ ] Add `DagResourceContract` struct
- [ ] Export from `core/ir/src/lib.rs`

### Phase 2: Tool Registry
- [ ] Add `resource_contract` field to `ToolDef`
- [ ] Add `provides()` and `needs()` builder methods
- [ ] Update `all_tools()` with resource contracts for all tools

### Phase 3: Resource Registry
- [ ] Create `gunbc-dag/src/resource_registry.rs`
- [ ] Implement `ResourceRegistry::from_tools()`
- [ ] Implement topological sort in `resolve()`
- [ ] Add cycle detection

### Phase 4: Staleness Checking
- [ ] Create `gunbc-dag/src/staleness.rs`
- [ ] Implement `check_state()` for each `BuildResource`
- [ ] Implement `ensure()` for each `BuildResource`
- [ ] Extract existing staleness logic from codegen/testgen

### Phase 5: Resource Executor
- [ ] Create `ResourceExecutor` struct
- [ ] Add `--mode` flag to `gunbc-build`
- [ ] Implement mode-aware execution loop
- [ ] Add clear error messages for stale resources

### Phase 6: Makefile Simplification
- [ ] Update `MetaTarget` to use `target_resource`
- [ ] Update `render_meta_target()` to emit `--mode` flags
- [ ] Remove `extra_deps` and `fix_deps` from `MetaTarget`
- [ ] Remove `PrepLevel` enum and related functions

### Phase 7: Cleanup
- [ ] Remove `prep_dep_name()`, `meta_target_deps()`
- [ ] Remove `render_fix_alias_targets()`
- [ ] Simplify `render_meta_fix_variant()`
- [ ] Update tests

## 12. Open Questions

1. **Should fmt/clippy be resources or just pre-flight checks?**
   - They don't produce artifacts, they validate code state
   - Could model as "code quality" resources that test/build implicitly need
   - Or keep as explicit `--mode=ensure` triggers fmt/clippy --fix

2. **How to handle parallel resource acquisition?**
   - If A needs [B, C] and B, C are independent, can acquire in parallel
   - ResourceExecutor could detect parallelizable branches
   - For now: sequential is fine, optimize later

3. **Should DryRun interact with ExecMode?**
   - DryRun + Verify = check without executing
   - DryRun + Ensure = show what would be regenerated
   - Orthogonal concerns, both should work together

4. **Bootstrap chicken-egg: codegen needs to run before codegen DAG exists**
   - Current solution: `ensure-codegen` is a simple cargo run, not a DAG
   - Keep this as a special case, or model as "bootstrap resource"?
