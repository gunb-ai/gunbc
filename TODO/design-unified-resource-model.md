# Design: Unified Resource Model

> **Consolidates:** URGENT_codegen_upsert.md, design-build-resource-chain.md,
> design-resource-acquisition.md (Phases 4-5), and related TODO_hacks items.
>
> **Principle:** All acquirable things—tools, build artifacts, filesystem handles,
> auth tokens—are **resources** with the same upsert semantics: Check → Create → Resolve.
> Freshness keys are **derived from declared inputs**, not configured.

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

1. **Deduce over configure.** If something can be computed from the model, compute it.
   Hash scope, staleness propagation, and dependencies should all be derived from
   proper modeling, not hardcoded or configured.

2. **Unify patterns.** `ToolHandle` and build resource handles are the **same abstraction**.
   Both are `ResourceHandle<R>` — proof of acquisition that flows through DAG edges.
   The only difference is the key computation (path existence vs content hash).

3. **Safe/slow first.** Be conservative now, optimize with proper understanding later.
   If we can't deduce something, be safe (assume stale). No premature optimization.

4. **Leverage DAG infrastructure.** Transactions, DryRun, hermeticity come from the
   DAG execution model, not special-cased code. The DAG is pure; the executor handles I/O.

5. **Mode is executor context.** Like DryRun, `ExecMode` (Verify/Ensure) is ambient
   context from the executor, not modeled in DAG structure. The DAG declares dependencies;
   the executor decides how to handle staleness.

## 3. Key Design Decisions

### 3.1 Hash Scope: Derived from Declared Inputs

**Decision:** Hash scope is **not a configuration choice**. It's derived from the
resource's declared inputs, similar to Bazel's action graph.

```rust
/// A resource definition declares its inputs and outputs explicitly.
pub struct ResourceDef {
    pub id: ResourceId,
    /// Input patterns — these determine the freshness key
    pub inputs: Vec<InputPattern>,
    /// Output patterns — these are produced when the resource is created
    pub outputs: Vec<PathBuf>,
    /// The DAG that creates this resource (if creatable)
    pub provider: Option<DagRef>,
}

pub enum InputPattern {
    /// Glob pattern for source files
    Glob(String),
    /// Another resource's outputs (transitive dependency)
    Resource(ResourceId),
    /// Environment value (e.g., toolchain version)
    Env(String),
}
```

Then hash computation is simply:

```rust
fn compute_key(def: &ResourceDef) -> ContentHash {
    let mut hasher = Sha256::new();
    for input in &def.inputs {
        match input {
            InputPattern::Glob(pattern) => {
                for path in glob(pattern) {
                    hasher.update(&fs::read(path)?);
                }
            }
            InputPattern::Resource(id) => {
                // Include dependency's key (transitive freshness)
                hasher.update(manifest.get(id)?.key.as_bytes());
            }
            InputPattern::Env(var) => {
                hasher.update(env::var(var)?.as_bytes());
            }
        }
    }
    ContentHash(hasher.finalize())
}
```

**Rationale:** No "narrow vs wide" choice needed. The inputs are the inputs.
If you want to invalidate on toolchain changes, declare `Env("RUSTC_VERSION")`.

### 3.2 Staleness Propagation: Falls Out of DAG Edges

**Decision:** Each resource checks its own inputs. Staleness propagates naturally
through `InputPattern::Resource` dependencies.

If resource B depends on resource A:
```rust
ResourceDef {
    id: "build:generated_tests",
    inputs: vec![
        Glob("gunbc-dag/src/**/*.rs"),
        Resource("build:generated_cli"),  // <-- includes A's key
    ],
    ...
}
```

When A becomes stale:
1. A's manifest key changes
2. B's hash computation includes A's key
3. B's computed key differs from stored key
4. B is stale

**No special staleness propagation logic needed** — it falls out of the model.

### 3.3 Granularity: Flexible ResourceScope

**Decision:** Resources can be as granular as needed. A resource might be:
- A single file
- A glob pattern
- A named logical resource

```rust
pub enum ResourceScope {
    /// Single file (finest granularity)
    File(PathBuf),
    /// Glob pattern (e.g., "target/codegen/bin/**/*.rs")
    Pattern(String),
    /// Named logical resource (e.g., "generated_cli")
    Named(String),
}
```

Codegen can generate resource definitions if there are many. We don't need to
manually enumerate every file.

### 3.4 Bootstrap: Special-Cased (For Now)

**Decision:** The bootstrap codegen (`ensure-codegen`) remains special-cased.
It uses a simple cargo run, not the manifest system.

**Rationale:** The codebase isn't mature enough for self-referential resource
management. Bootstrap is already working. Don't overcomplicate.

**Future:** Once the resource model is stable, bootstrap could become the first
resource in the chain with `inputs: []` (no dependencies).

### 3.5 Unified ResourceHandle

**Decision:** `ToolHandle` and build resource handles are unified under
`ResourceHandle<R>`. Both are **proof of acquisition** that flows through edges.

```rust
/// A handle proving a resource has been acquired and is fresh.
/// This is the ONLY way to depend on a resource in a DAG.
pub struct ResourceHandle<R: ManagedResource> {
    /// The resource this handle refers to
    resource: R,
    /// The key at time of acquisition (proves freshness)
    key: ContentHash,
    /// Capability marker (prevents forgery)
    _acquired: PhantomData<()>,
}

impl<R: ManagedResource> ResourceHandle<R> {
    /// Framework use only — creates a handle after successful acquisition.
    pub(crate) fn acquire(resource: R, key: ContentHash) -> Self {
        Self { resource, key, _acquired: PhantomData }
    }

    /// Get the resource this handle refers to.
    pub fn resource(&self) -> &R { &self.resource }

    /// Get the freshness key at time of acquisition.
    pub fn key(&self) -> &ContentHash { &self.key }
}
```

For tools, the handle also carries the resolved path:
```rust
impl ResourceHandle<ToolResource> {
    pub fn path(&self) -> &Path { &self.resource.path }
}
```

For build resources, the handle carries proof of freshness (the key).
The "payload" may be Unit, but **the handle itself is the proof**.

```rust
// Both are ResourceHandle — same pattern
let tool: ResourceHandle<ToolResource> = inputs.get("res:clippy")?;
let codegen: ResourceHandle<BuildResource> = inputs.get("res:generated_cli")?;

// Tool has path
tool.path();

// Build has proof (handle existence = resource is fresh)
codegen.key();  // Can verify if needed, but having the handle is the proof
```

### 3.6 Mode as Executor Context

**Decision:** `ExecMode` (Verify/Ensure) is **executor context**, not DAG structure.
Similar to `DryRun`.

```rust
pub struct ExecutorContext {
    pub dry_run: bool,
    pub mode: ExecMode,
    // ...
}

pub enum ExecMode {
    /// Assert resources are fresh, fail if stale (CI mode)
    Verify,
    /// Make resources fresh if stale (dev mode)
    Ensure,
}
```

The DAG declares what resources it needs. The executor decides:
- **Verify:** Check freshness, fail if stale
- **Ensure:** Check freshness, regenerate if stale

**Rationale:** Mode is about "what to do when stale" — that's execution policy,
not graph structure. The same DAG runs in both CI (Verify) and dev (Ensure).

### 3.7 Transactions: DAG is Pure, Executor Handles I/O

**Decision:** The DAG is pure. Manifest updates are **outputs**, not side effects.

```rust
// DAG node output includes manifest update instruction
pub struct ResourceAcquisitionOutput {
    pub handle: ResourceHandle<R>,
    pub manifest_update: Option<ManifestEntry>,
}

// Executor applies manifest updates atomically
impl Executor {
    fn apply_manifest_updates(&self, updates: Vec<ManifestEntry>) -> Result<()> {
        if self.context.dry_run {
            return Ok(());  // Don't write in DryRun
        }
        // Write to .manifest.tmp, rename on success (atomic)
        let tmp = manifest_path().with_extension("tmp");
        write_manifest(&tmp, updates)?;
        fs::rename(tmp, manifest_path())?;
        Ok(())
    }
}
```

**Rationale:** Keeps DAG execution hermetic. The DAG computes what should happen;
the executor makes it happen. Same pattern as DryRun.

## 4. Unified Resource Abstraction

### 4.1 The ManagedResource Trait

```rust
/// A resource that can be acquired with freshness checking.
///
/// This trait unifies tools, build artifacts, and other acquirable things.
/// All follow the same pattern: Check → Create → Resolve.
pub trait ManagedResource: Resource + Sized {
    /// The definition of this resource (inputs, outputs, provider).
    fn definition(&self) -> &ResourceDef;

    /// Compute the current freshness key from declared inputs.
    fn compute_key(&self) -> Result<ContentHash, ResourceError> {
        compute_key_from_def(self.definition())
    }

    /// Check current state against manifest.
    fn check_state(&self, manifest: &ResourceManifest) -> ResourceState {
        let current_key = match self.compute_key() {
            Ok(k) => k,
            Err(e) => return ResourceState::Error(e.to_string()),
        };

        match manifest.get(&self.definition().id) {
            None => ResourceState::Missing,
            Some(entry) if entry.key != current_key => {
                ResourceState::Stale {
                    reason: "inputs changed".into(),
                    stored_key: entry.key.clone(),
                    current_key,
                }
            }
            Some(_) => ResourceState::Fresh,
        }
    }

    /// Create/regenerate this resource.
    /// Returns the manifest entry to store.
    fn create(&self, ctx: &ExecutorContext) -> Result<ManifestEntry, ResourceError>;

    /// Acquire a handle to this resource.
    /// Checks freshness, creates if needed (based on mode), returns handle.
    fn acquire(&self, ctx: &ExecutorContext, manifest: &ResourceManifest)
        -> Result<ResourceHandle<Self>, ResourceError>
    {
        let state = self.check_state(manifest);

        match (state, ctx.mode) {
            (ResourceState::Fresh, _) => {
                let key = self.compute_key()?;
                Ok(ResourceHandle::acquire(self.clone(), key))
            }
            (_, ExecMode::Ensure) => {
                let entry = self.create(ctx)?;
                Ok(ResourceHandle::acquire(self.clone(), entry.key.clone()))
            }
            (ResourceState::Missing, ExecMode::Verify) => {
                Err(ResourceError::Missing(self.definition().id.clone()))
            }
            (ResourceState::Stale { reason, .. }, ExecMode::Verify) => {
                Err(ResourceError::Stale {
                    id: self.definition().id.clone(),
                    reason,
                })
            }
            (ResourceState::Error(e), _) => {
                Err(ResourceError::CheckFailed(e))
            }
        }
    }
}
```

### 4.2 ResourceState

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState {
    /// Resource doesn't exist in manifest.
    Missing,
    /// Resource exists but key doesn't match (inputs changed).
    Stale {
        reason: String,
        stored_key: ContentHash,
        current_key: ContentHash,
    },
    /// Resource exists and key matches.
    Fresh,
    /// Error computing state (e.g., can't read input file).
    Error(String),
}
```

### 4.3 Concrete Resource Types

```rust
/// A tool resource (binary on PATH).
pub struct ToolResource {
    pub def: &'static CliToolDef,
    pub path: PathBuf,
}

impl ManagedResource for ToolResource {
    fn definition(&self) -> &ResourceDef {
        // Tools use path existence as the "input"
        ResourceDef {
            id: ResourceId::tool(self.def.id),
            inputs: vec![],  // No file inputs
            outputs: vec![self.path.clone()],
            provider: None,  // Created externally
        }
    }

    fn compute_key(&self) -> Result<ContentHash, ResourceError> {
        // For tools, key is just "does the binary exist at this path"
        if self.path.exists() {
            Ok(ContentHash::from_path(&self.path))
        } else {
            Err(ResourceError::Missing(self.definition().id.clone()))
        }
    }

    fn create(&self, ctx: &ExecutorContext) -> Result<ManifestEntry, ResourceError> {
        // Run install command
        execute_install(self.def)?;
        let path = resolve_tool_path(self.def)?;
        Ok(ManifestEntry {
            key: ContentHash::from_path(&path),
            created_at: ctx.now(),
            outputs: vec![path],
        })
    }
}

/// A build resource (generated artifact).
pub struct BuildResource {
    pub kind: BuildResourceKind,
    pub def: ResourceDef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildResourceKind {
    GeneratedCli,
    GeneratedTests,
    FormattedCode,
    LintedCode,
    CompiledCode,
    TypeChecked,
    TestedCode,
}

impl ManagedResource for BuildResource {
    fn definition(&self) -> &ResourceDef {
        &self.def
    }

    fn create(&self, ctx: &ExecutorContext) -> Result<ManifestEntry, ResourceError> {
        // Run the provider DAG
        let provider = self.def.provider.as_ref()
            .ok_or(ResourceError::NoProvider(self.def.id.clone()))?;

        execute_dag(provider, ctx)?;

        Ok(ManifestEntry {
            key: self.compute_key()?,
            created_at: ctx.now(),
            outputs: self.def.outputs.clone(),
        })
    }
}
```

## 5. Resource Registry

### 5.1 Declaring Resources

Resources are declared in the tool registry with explicit inputs/outputs:

```rust
pub fn build_resources() -> Vec<ResourceDef> {
    vec![
        ResourceDef {
            id: ResourceId::build("generated_cli"),
            inputs: vec![
                InputPattern::Glob("core/codegen/src/**/*.rs"),
                InputPattern::Glob("core/codegen/Cargo.toml"),
                InputPattern::Env("CARGO_PKG_VERSION"),
            ],
            outputs: vec![PathBuf::from("target/codegen/bin")],
            provider: Some(DagRef::new("codegen")),
        },
        ResourceDef {
            id: ResourceId::build("generated_tests"),
            inputs: vec![
                InputPattern::Glob("gunbc-dag/src/**/*.rs"),
                InputPattern::Resource(ResourceId::build("generated_cli")),
            ],
            outputs: vec![PathBuf::from("target/codegen/lib/*/tests.rs")],
            provider: Some(DagRef::new("testgen")),
        },
        // ... more resources
    ]
}
```

### 5.2 Automatic Resolution

```rust
impl ResourceRegistry {
    /// Resolve all resources needed to acquire a target.
    /// Returns topologically sorted list (dependencies first).
    pub fn resolve(&self, target: &ResourceId) -> Result<Vec<ResourceId>, CycleError> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();  // For cycle detection

        self.visit(target, &mut visited, &mut stack, &mut result)?;
        Ok(result)
    }

    fn visit(
        &self,
        id: &ResourceId,
        visited: &mut HashSet<ResourceId>,
        stack: &mut HashSet<ResourceId>,
        order: &mut Vec<ResourceId>,
    ) -> Result<(), CycleError> {
        if visited.contains(id) {
            return Ok(());
        }
        if stack.contains(id) {
            return Err(CycleError::new(id.clone()));
        }

        stack.insert(id.clone());

        let def = self.get(id)?;
        for input in &def.inputs {
            if let InputPattern::Resource(dep_id) = input {
                self.visit(dep_id, visited, stack, order)?;
            }
        }

        stack.remove(id);
        visited.insert(id.clone());
        order.push(id.clone());

        Ok(())
    }
}
```

## 6. Manifest

### 6.1 Schema

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceManifest {
    /// Schema version for forward compatibility.
    pub version: u32,
    /// Resources by ID.
    pub resources: HashMap<ResourceId, ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Content hash of inputs when resource was created.
    pub key: ContentHash,
    /// When this entry was created.
    pub created_at: Timestamp,
    /// Files this resource produced.
    pub outputs: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(String);  // Hex-encoded SHA-256
```

### 6.2 Location

```
target/.resource-manifest.json
```

### 6.3 Atomic Updates

```rust
impl ResourceManifest {
    pub fn save(&self) -> Result<(), io::Error> {
        let path = Path::new("target/.resource-manifest.json");
        let tmp = path.with_extension("json.tmp");

        // Write to temp file
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&tmp, content)?;

        // Atomic rename
        fs::rename(tmp, path)?;

        Ok(())
    }
}
```

## 7. Makefile Simplification

### 7.1 Before (Current)

```makefile
# Complex manual wiring
test: codegen testgen-check
	@cargo run -p gunbc-dag --bin gunbc-test --release

test-fix: fmt-fix lint-fix codegen testgen
	@cargo run -p gunbc-dag --bin gunbc-test --release
```

### 7.2 After (With Resource Model)

```makefile
# Mode flag only — dependencies resolved automatically
test:
	@cargo run -p gunbc-dag --bin gunbc-test --release -- --mode=verify

test-fix:
	@cargo run -p gunbc-dag --bin gunbc-test --release -- --mode=ensure
```

The test binary internally:
1. Declares `needs: [GeneratedCli, GeneratedTests, CompiledCode]`
2. Calls `registry.resolve("build:tested_code")`
3. Acquires each resource in order (checks freshness, creates if needed per mode)
4. Runs the actual tests

### 7.3 Elimination of Manual Wiring

| Removed | Replaced By |
|---------|-------------|
| `extra_deps: Vec<String>` | `ResourceDef.inputs` with `InputPattern::Resource` |
| `fix_deps: Vec<String>` | `ExecMode::Ensure` (single flag) |
| `PrepLevel` enum | Automatic resolution from declared dependencies |
| `prep_dep_name()` function | `ResourceRegistry.resolve()` |
| `testgen` vs `testgen-check` | Single target + `--mode` flag |
| `fmt` vs `fmt-fix` | Single target + `--mode` flag |

## 8. Implementation Plan

### Phase 1: Core Types (Foundation)

**Goal:** Establish the type system without changing behavior.

**New files:**
- `core/ir/src/resource/managed.rs` — `ManagedResource` trait
- `core/ir/src/resource/handle.rs` — `ResourceHandle<R>` (unified)
- `core/ir/src/resource/state.rs` — `ResourceState` enum
- `core/ir/src/resource/manifest.rs` — `ResourceManifest`, `ManifestEntry`
- `core/ir/src/resource/def.rs` — `ResourceDef`, `InputPattern`
- `core/ir/src/resource/hash.rs` — `ContentHash`, hash computation

**Modify:**
- `core/ir/src/resource.rs` — Re-export new types, keep existing `Resource` trait
- `core/ir/src/lib.rs` — Export `resource` module

**Tests:**
- Unit tests for `ContentHash` computation
- Unit tests for `ResourceState` transitions
- Unit tests for manifest serialization

**Estimated scope:** ~500 LOC new, ~50 LOC modified

---

### Phase 2: Resource Registry

**Goal:** Declare resources with explicit inputs/outputs.

**New files:**
- `core/ir/src/resource/registry.rs` — `ResourceRegistry`, resolution logic

**Modify:**
- `core/codegen/src/registry.rs` — Add `ResourceDef` to `ToolDef`

**Codegen changes:**
- Generate `build_resources()` function from tool registry
- Include input patterns derived from existing knowledge

**Tests:**
- Test dependency resolution (topological sort)
- Test cycle detection
- Test transitive dependency inclusion

**Estimated scope:** ~300 LOC new, ~100 LOC modified

---

### Phase 3: Manifest Integration

**Goal:** Write and read manifest during codegen.

**Modify:**
- `core/codegen/src/main.rs` — Write manifest after successful codegen
- `gunbc-dag/src/bin/testgen.rs` — Write manifest after successful testgen

**New behavior:**
- After `cargo run -p gunbc-codegen`, `target/.resource-manifest.json` exists
- Manifest contains entry for `build:generated_cli` with computed hash

**Tests:**
- Integration test: run codegen, verify manifest written
- Integration test: modify source, verify hash changes
- Integration test: run codegen again, verify manifest updated

**Estimated scope:** ~100 LOC new, ~50 LOC modified

---

### Phase 4: CI Integration (Fix Brittle Check)

**Goal:** Replace file existence check with manifest check.

**Modify:**
- `gunbc-dag/src/ci/ops.rs` — Replace `PrepareCodegenExistsCheck` with manifest-based check
- `gunbc-dag/src/ci/graph.rs` — Update graph if needed

**Old behavior:**
```rust
FileRequest::exists("target/codegen/bin/deps/main.rs")
```

**New behavior:**
```rust
let manifest = load_manifest()?;
let fresh = manifest.check_fresh(&ResourceId::build("generated_cli"), &compute_hash());
```

**Tests:**
- Test CI graph with fresh manifest (skips codegen)
- Test CI graph with stale manifest (runs codegen)
- Test CI graph with missing manifest (runs codegen)

**Estimated scope:** ~50 LOC modified

---

### Phase 5: ExecMode Integration

**Goal:** Add `--mode=verify|ensure` flag to build tools.

**Modify:**
- `gunbc-dag/src/bin/testgen.rs` — Replace `--check` with `--mode=verify`
- `core/exec/src/context.rs` — Add `ExecMode` to `ExecutorContext`
- Other bin files as needed

**New CLI:**
```
gunbc-testgen                    # default: --mode=ensure
gunbc-testgen --mode=verify      # CI mode (fail if stale)
gunbc-testgen --mode=ensure      # Dev mode (regenerate if stale)
```

**Deprecate:**
- `--check` flag (replaced by `--mode=verify`)

**Tests:**
- Test `--mode=verify` fails on stale
- Test `--mode=ensure` regenerates on stale
- Test backward compat for `--check` → `--mode=verify`

**Estimated scope:** ~100 LOC modified

---

### Phase 6: Makefile Simplification

**Goal:** Remove manual dependency wiring from makegen.

**Modify:**
- `gunbc-dag/src/makegen/registry.rs`:
  - Remove `extra_deps` field from `MetaTarget`
  - Remove `fix_deps` field from `MetaTarget`
  - Remove `PrepLevel` enum (or deprecate)
  - Add `resources: Vec<ResourceId>` to `MetaTarget`

- `gunbc-dag/src/makegen/render.rs`:
  - Remove `prep_dep_name()` function
  - Remove fix variant dependency transformation
  - Emit `--mode=verify` for base targets
  - Emit `--mode=ensure` for `-fix` targets

**Before:**
```rust
MetaTarget::new("test", ...)
    .with_extra_deps(vec!["testgen-check"])
    .with_fix_variant(vec!["fmt-fix", "lint-fix"])
```

**After:**
```rust
MetaTarget::new("test", ...)
    .with_resources(vec![
        ResourceId::build("generated_cli"),
        ResourceId::build("generated_tests"),
    ])
// Fix variant automatically uses --mode=ensure
```

**Tests:**
- Verify generated Makefile has correct targets
- Verify `make test` uses `--mode=verify`
- Verify `make test-fix` uses `--mode=ensure`

**Estimated scope:** ~200 LOC modified, ~100 LOC deleted

---

### Phase 7: ToolHandle Unification

**Goal:** Migrate `ToolHandle` to use `ResourceHandle<ToolResource>`.

**Modify:**
- `core/ir/src/transport/cli.rs`:
  - `ToolHandle` becomes type alias for `ResourceHandle<ToolResource>`
  - Keep backward-compatible API

**This is optional/deferred** — can be done incrementally since the pattern
is already correct for tools.

**Estimated scope:** ~100 LOC modified

---

### Phase 8: Cleanup

**Goal:** Remove obsolete code and update docs.

**Remove/archive:**
- Already done: `URGENT_codegen_upsert.md` → TODONE
- Already done: `design-build-resource-chain.md` → TODONE

**Update:**
- `TODO_hacks` — Mark resolved items
- `design-resource-acquisition.md` — Mark phases 4-5 complete
- `README.md` — Document `--mode` flag

**Estimated scope:** Documentation only

---

## 9. Summary: What Changes Where

| File | Change |
|------|--------|
| `core/ir/src/resource/*.rs` | New: managed.rs, handle.rs, state.rs, manifest.rs, def.rs, hash.rs, registry.rs |
| `core/ir/src/resource.rs` | Re-export new types |
| `core/codegen/src/main.rs` | Write manifest after codegen |
| `core/codegen/src/registry.rs` | Add `ResourceDef` to `ToolDef` |
| `gunbc-dag/src/bin/testgen.rs` | Write manifest, replace `--check` with `--mode` |
| `gunbc-dag/src/ci/ops.rs` | Replace file check with manifest check |
| `gunbc-dag/src/makegen/registry.rs` | Remove extra_deps, fix_deps, PrepLevel |
| `gunbc-dag/src/makegen/render.rs` | Emit `--mode` flags instead of dep chains |
| `core/exec/src/context.rs` | Add `ExecMode` to context |

## 10. Checklist

### Phase 1: Core Types
- [ ] `ManagedResource` trait
- [ ] `ResourceHandle<R>` struct
- [ ] `ResourceState` enum
- [ ] `ResourceManifest` struct
- [ ] `ManifestEntry` struct
- [ ] `ResourceDef` struct
- [ ] `InputPattern` enum
- [ ] `ContentHash` struct and computation
- [ ] Unit tests for all types

### Phase 2: Resource Registry
- [ ] `ResourceRegistry` struct
- [ ] `resolve()` with topological sort
- [ ] Cycle detection
- [ ] `ResourceDef` in `ToolDef`
- [ ] `build_resources()` function
- [ ] Unit tests for resolution

### Phase 3: Manifest Integration ✓
- [x] Manifest write in codegen
- [x] Manifest write in testgen
- [x] Integration tests

### Phase 4: CI Integration ✓
- [x] Replace file check in ci/ops.rs
- [x] Test with fresh/stale/missing manifest

### Phase 5: ExecMode Integration ✓
- [x] `ExecMode` enum (in state.rs)
- [x] `--mode` flag in CI tool
- [ ] `--mode` flag in testgen (deferred)
- [ ] `--mode` flag in other bins (deferred)
- [ ] Deprecate `--check` flag (deferred)

### Phase 6: Makefile Simplification (Deferred)
- [ ] Remove `extra_deps` from `MetaTarget`
- [ ] Remove `fix_deps` from `MetaTarget`
- [ ] Remove/deprecate `PrepLevel`
- [ ] Remove `prep_dep_name()`
- [ ] Emit `--mode` in renderer
- [ ] Test generated Makefile

Note: Phase 6 is now enabled by the infrastructure from phases 1-5.
The current manual wiring continues to work; simplification is a future optimization.

### Phase 7: ToolHandle Unification (Optional)
- [ ] `ToolHandle` as `ResourceHandle<ToolResource>`

### Phase 8: Cleanup ✓
- [x] Update TODO_hacks
- [x] Mark resolved TODOs in TODONE
- [ ] Update README (deferred)
