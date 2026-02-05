# Design: Unified Resource Model

> **Consolidates:** URGENT_codegen_upsert.md, design-build-resource-chain.md,
> design-resource-acquisition.md (Phases 4-5), and related TODO_hacks items.
>
> **Principle:** All acquirable things—tools, build artifacts, filesystem handles,
> auth tokens—are **resources** with the same upsert semantics: Check → Create → Resolve.
> The only difference is how "freshness" is determined.

## Ownership
- [ ] Unassigned

## Status
**URGENT** — This design addresses multiple interconnected issues:
- CI codegen check is brittle (file existence hack)
- Manual dependency wiring in Makefile (extra_deps, fix_deps, PrepLevel)
- testgen/testgen-check duality, fmt/fmt-fix duality
- No staleness detection for build artifacts

## 1. Problem Statement

We have multiple resource acquisition patterns that are **identical in structure**
but implemented separately:

| Resource Type | Check | Create | Resolve | Current State |
|--------------|-------|--------|---------|---------------|
| **ToolHandle** | `which {tool}` | `cargo install` | Return path | ✅ Done (EnvOp) |
| **GeneratedCli** | File exists? | Run codegen | Return manifest | ❌ Brittle hack |
| **GeneratedTests** | `--check` mode | Run testgen | Return success | ❌ Two targets |
| **FormattedCode** | `cargo fmt --check` | `cargo fmt` | Return success | ❌ Two targets |
| **AuthToken** | Env var set? | Error | Return secret | ✅ Done (AuthEnv) |

The tool acquisition pattern (EnvOp → ToolHandle) **works perfectly**. We should
generalize it to all resources, not reinvent it for each resource type.

### Current Hacks (All Same Root Cause)

| Hack | Location | Root Cause |
|------|----------|------------|
| `target/codegen/bin/deps/main.rs` existence check | ci/ops.rs:151 | No content hash |
| `extra_deps: ["testgen-check"]` | registry.rs:783 | No resource model |
| `fix_deps: ["fmt-fix"]` | registry.rs:784 | Verify vs Ensure not unified |
| `PrepLevel::Full → "codegen"` | render.rs:297 | Manual dep routing |
| `ensure-codegen` vs `codegen` | Makefile:15-25 | Bootstrap special-cased |
| `testgen` vs `testgen-check` | testgen.rs:20 | Two targets simulate mode |

## 2. Design Principles

1. **One trait, one pattern.** The existing `Resource` trait and `UpsertBuilder`
   pattern apply to ALL acquirable things.

2. **Freshness is a key computation.** Tools use "does binary exist on PATH?"
   Build artifacts use "does output exist AND input_hash == stored_hash?"

3. **Mode flows through edges.** `ExecMode::Verify` fails on stale.
   `ExecMode::Ensure` regenerates. Same as DryRun/Real for transport.

4. **Declarative dependencies.** DAGs declare `provides`/`needs`, the framework
   resolves the dependency chain. No manual `extra_deps`.

## 3. Unified Resource Abstraction

### 3.1 Extending the Existing Resource Trait

The `Resource` trait already exists in `core/ir/src/resource.rs`:

```rust
pub trait Resource: Into<Value> + TryFrom<Value> {
    fn resource_id(&self) -> ResourceId;
    fn access_mode(&self) -> AccessMode;
    fn kind(&self) -> ResourceKind;
}
```

We extend it with freshness semantics:

```rust
/// Extended resource trait with freshness checking.
pub trait ManagedResource: Resource {
    /// The key type for this resource (determines freshness).
    type Key: Eq + Hash + Serialize + Deserialize;

    /// Compute the current key from inputs.
    /// For tools: binary path existence.
    /// For build artifacts: content hash of input files.
    fn compute_key(&self) -> Self::Key;

    /// Check if the resource is fresh given a stored key.
    fn is_fresh(&self, stored: &Self::Key) -> bool {
        self.compute_key() == *stored
    }
}
```

### 3.2 Resource State (Unified)

```rust
/// State of any managed resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState {
    /// Resource doesn't exist.
    Missing,
    /// Resource exists but key doesn't match (stale).
    Stale { reason: String },
    /// Resource exists and key matches (fresh).
    Fresh,
}

/// Execution mode for resource acquisition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecMode {
    /// Check that resources are fresh, fail if stale (CI mode).
    #[default]
    Verify,
    /// Ensure resources are fresh, regenerate if stale (dev mode).
    Ensure,
}
```

### 3.3 Build Resource Enum

```rust
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

impl ManagedResource for BuildResource {
    type Key = ContentHash;  // SHA-256 of input files

    fn compute_key(&self) -> ContentHash {
        match self {
            Self::GeneratedCli => hash_codegen_inputs(),
            Self::GeneratedTests => hash_testgen_inputs(),
            Self::FormattedCode => hash_source_files(),
            // ...
        }
    }
}
```

## 4. Manifest: The Upsert Key Storage

### 4.1 Manifest Schema

The manifest stores computed keys for each resource:

```rust
/// Manifest for tracking resource freshness.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceManifest {
    /// Version of the manifest format.
    pub version: u32,
    /// When the manifest was last updated.
    pub updated_at: Timestamp,
    /// Resource keys by resource ID.
    pub resources: HashMap<ResourceId, ResourceEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceEntry {
    /// The computed key when this resource was last created.
    pub key: String,  // Hex-encoded hash
    /// When this resource was last created.
    pub created_at: Timestamp,
    /// Files this resource produced (for cleanup).
    pub outputs: Vec<PathBuf>,
}
```

Location: `target/.resource-manifest.json`

### 4.2 Key Computation

For build artifacts, the key is a content hash of all inputs:

```rust
fn hash_codegen_inputs() -> ContentHash {
    let mut hasher = Sha256::new();

    // Hash codegen tool version
    hasher.update(env!("CARGO_PKG_VERSION").as_bytes());

    // Hash all registry source files
    for path in glob("core/codegen/src/**/*.rs") {
        hasher.update(&std::fs::read(path)?);
    }

    // Hash all tool definitions
    for path in glob("gunbc-dag/src/*/registry.rs") {
        hasher.update(&std::fs::read(path)?);
    }

    ContentHash(hasher.finalize())
}
```

## 5. Unified Upsert Pattern

### 5.1 ResourceEnvOp (Generalized EnvOp)

```rust
/// Environment operation that acquires any managed resource.
///
/// This generalizes EnvOp (tools) to all resource types.
pub struct ResourceEnvOp<R: ManagedResource> {
    pub resource: R,
    pub mode: ExecMode,
}

impl<R: ManagedResource> Executable for ResourceEnvOp<R> {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<OutputMap, ExecError> {
        let manifest = load_manifest()?;
        let current_key = self.resource.compute_key();

        let state = match manifest.get(&self.resource.resource_id()) {
            None => ResourceState::Missing,
            Some(entry) if entry.key != current_key.to_string() => {
                ResourceState::Stale { reason: "inputs changed".into() }
            }
            Some(_) => ResourceState::Fresh,
        };

        match (state, self.mode) {
            // Fresh in any mode: return existing
            (ResourceState::Fresh, _) => {
                Ok(OutputMap::new().state("state", ResourceState::Fresh))
            }

            // Missing/Stale in Ensure mode: create
            (_, ExecMode::Ensure) => {
                self.resource.create()?;
                save_manifest_entry(&self.resource, &current_key)?;
                Ok(OutputMap::new().state("state", ResourceState::Fresh))
            }

            // Missing/Stale in Verify mode: fail
            (ResourceState::Missing, ExecMode::Verify) => {
                Err(ExecError::new(format!(
                    "Resource {} is missing (run with --mode=ensure)",
                    self.resource.resource_id()
                )))
            }
            (ResourceState::Stale { reason }, ExecMode::Verify) => {
                Err(ExecError::new(format!(
                    "Resource {} is stale: {} (run with --mode=ensure)",
                    self.resource.resource_id(), reason
                )))
            }
        }
    }
}
```

### 5.2 Comparison: Tool vs Build Resource

| Aspect | ToolHandle | BuildResource |
|--------|------------|---------------|
| Check | `which {binary}` | manifest.key == hash(inputs) |
| Create | `cargo install` | Run provider DAG |
| Resolve | Return path | Return success + update manifest |
| Key | Binary path | Content hash |
| Storage | None (stateless) | `.resource-manifest.json` |

The pattern is identical—only the key computation differs.

## 6. Dependency Declaration

### 6.1 DAG Resource Contract

Each DAG declares what it provides and needs:

```rust
/// A DAG's resource contract.
#[derive(Debug, Clone, Default)]
pub struct ResourceContract {
    /// Resource this DAG provides (if any).
    pub provides: Option<BuildResource>,
    /// Resources this DAG needs before it can run.
    pub needs: Vec<BuildResource>,
}
```

### 6.2 Tool Registry Integration

```rust
// In all_tools()
ToolDef::new("codegen", "Generate CLI entrypoints")
    .provides(BuildResource::GeneratedCli)
    .needs(vec![])  // Bootstrap, no deps

ToolDef::new("testgen", "Generate tests from DAGs")
    .provides(BuildResource::GeneratedTests)
    .needs(vec![BuildResource::GeneratedCli])

ToolDef::new("test", "Run all tests")
    .provides(BuildResource::TestedCode)
    .needs(vec![
        BuildResource::GeneratedCli,
        BuildResource::GeneratedTests,
        BuildResource::CompiledCode,
    ])
```

### 6.3 Automatic Dependency Resolution

```rust
impl ResourceRegistry {
    /// Resolve the execution order for acquiring a target resource.
    pub fn resolve(&self, target: BuildResource) -> Vec<BuildResource> {
        // Topological sort of the dependency graph
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        self.visit(target, &mut visited, &mut result);
        result
    }

    fn visit(&self, r: BuildResource, visited: &mut HashSet<BuildResource>, order: &mut Vec<BuildResource>) {
        if visited.contains(&r) { return; }
        visited.insert(r);

        for dep in self.get_contract(r).needs {
            self.visit(dep, visited, order);
        }
        order.push(r);
    }
}
```

## 7. Makefile Simplification

### 7.1 Before (Current)

```makefile
# Complex manual wiring
test: codegen testgen-check
	@cargo run -p gunbc-dag --bin gunbc-build --release

test-fix: fmt-fix lint-fix codegen testgen-check
	@cargo run -p gunbc-dag --bin gunbc-build --release
```

### 7.2 After (With Resource Model)

```makefile
# Simple mode flags
test:
	@cargo run -p gunbc-dag --bin gunbc-build --release -- --mode=verify

test-fix:
	@cargo run -p gunbc-dag --bin gunbc-build --release -- --mode=ensure
```

The build DAG internally:
1. Declares `needs: [GeneratedCli, GeneratedTests, CompiledCode]`
2. Mode flag propagates to all dependency checks
3. Verify mode fails fast on stale resources
4. Ensure mode regenerates as needed

### 7.3 Elimination of Manual Wiring

| Removed | Replaced By |
|---------|-------------|
| `extra_deps: Vec<String>` | `ResourceContract.needs` |
| `fix_deps: Vec<String>` | `ExecMode::Ensure` |
| `PrepLevel` enum | `ResourceContract.needs` |
| `prep_dep_name()` | Automatic resolution |
| `testgen` vs `testgen-check` | Single target + mode |
| `fmt` vs `fmt-fix` | Single target + mode |

## 8. CI Integration (Codegen Upsert Fix)

### 8.1 Current (Brittle)

```rust
// ci/ops.rs:151
fn execute_prepare_codegen_exists_check(_inputs: ...) -> ... {
    // HACK: Check arbitrary file
    let request = TransportRequest::File(
        FileRequest::exists("target/codegen/bin/deps/main.rs")
    );
    ...
}
```

### 8.2 New (Content Hash)

```rust
fn execute_prepare_codegen_check(_inputs: ...) -> ... {
    let manifest = load_manifest()?;
    let current_hash = hash_codegen_inputs();

    let fresh = manifest
        .get(&ResourceId::new("build:generated_cli"))
        .map(|e| e.key == current_hash.to_string())
        .unwrap_or(false);

    OutputMap::new()
        .bool("codegen_fresh", fresh)
        .str("current_hash", current_hash.to_string())
        .ok()
}
```

## 9. Implementation Plan

### Phase 1: Core Infrastructure (Non-Breaking)

**Files to create:**
- `core/ir/src/managed_resource.rs` — `ManagedResource` trait, `ResourceState`, `ExecMode`
- `core/ir/src/manifest.rs` — `ResourceManifest`, load/save functions

**Files to modify:**
- `core/ir/src/lib.rs` — Export new types
- `core/ir/src/resource.rs` — Add `ResourceId::build()` constructor

### Phase 2: Build Resource Types

**Files to create:**
- `core/ir/src/build_resource.rs` — `BuildResource` enum, `ResourceContract`

**Files to modify:**
- `core/codegen/src/registry.rs` — Add `provides`/`needs` to `ToolDef`

### Phase 3: Content Hash Implementation

**Files to create:**
- `gunbc-dag/src/resource_hash.rs` — Hash computation for each `BuildResource`

**Files to modify:**
- `core/codegen/src/main.rs` — Write manifest after successful codegen

### Phase 4: CI Integration

**Files to modify:**
- `gunbc-dag/src/ci/ops.rs` — Replace file existence check with manifest check
- `gunbc-dag/src/ci/graph.rs` — Update graph to use new ops

### Phase 5: Makefile Simplification

**Files to modify:**
- `gunbc-dag/src/makegen/registry.rs` — Remove `extra_deps`, `fix_deps`, `PrepLevel`
- `gunbc-dag/src/makegen/render.rs` — Emit `--mode` flags instead of dep chains
- `gunbc-dag/src/bin/*.rs` — Add `--mode` flag parsing

### Phase 6: Cleanup

**Files to remove/archive:**
- `TODO/URGENT_codegen_upsert.md` → TODONE
- `TODO/design-build-resource-chain.md` → TODONE (consolidated here)
- `TODO/design-resource-acquisition.md` Phases 4-5 → mark complete

**Files to update:**
- `TODO_hacks` — Remove resolved items (PrepLevel, extra_deps, fix_deps)

## 10. Consolidated TODO Items

### From URGENT_codegen_upsert.md
- [x] Problem identified: file existence check is brittle
- [ ] Implement content hash manifest
- [ ] Update CI ops to use manifest

### From design-build-resource-chain.md
- [ ] `BuildResource` enum
- [ ] `ResourceState` enum
- [ ] `ExecMode` enum
- [ ] `ResourceContract` struct
- [ ] Resource registry and resolver
- [ ] `--mode` flag in build tools
- [ ] Makefile simplification

### From design-resource-acquisition.md (Phases 4-5)
- [ ] Sub-DAG zero-based delegation
- [ ] Resource accounting integration

### From TODO_hacks
- [ ] PrepLevel→deps mapping hardcoded (§32-45) — replaced by ResourceContract
- [ ] Meta-target dependency strings not verified (§65-80) — replaced by typed refs
- [ ] Tool targets blanket-depend on ensure-codegen (§49-61) — replaced by needs

## 11. Checklist

### Phase 1
- [ ] Create `ManagedResource` trait
- [ ] Create `ResourceState` enum
- [ ] Create `ExecMode` enum
- [ ] Create `ResourceManifest` struct
- [ ] Implement manifest load/save

### Phase 2
- [ ] Create `BuildResource` enum
- [ ] Create `ResourceContract` struct
- [ ] Add `provides`/`needs` to `ToolDef`
- [ ] Implement `ResourceRegistry`

### Phase 3
- [ ] Implement `hash_codegen_inputs()`
- [ ] Implement `hash_testgen_inputs()`
- [ ] Update codegen to write manifest

### Phase 4
- [ ] Update `PrepareCodegenExistsCheck` to use manifest
- [ ] Update `ParseCodegenExists` to use hash comparison
- [ ] Test CI with new check

### Phase 5
- [ ] Add `--mode` flag to build tools
- [ ] Remove `extra_deps` from `MetaTarget`
- [ ] Remove `fix_deps` from `MetaTarget`
- [ ] Remove `PrepLevel` enum
- [ ] Update Makefile renderer

### Phase 6
- [ ] Move obsolete TODOs to TODONE
- [ ] Update TODO_hacks
- [ ] Update tests
