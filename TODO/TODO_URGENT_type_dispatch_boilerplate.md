# URGENT: Type-Dispatch Boilerplate Elimination

**Status**: Active
**Date**: 2026-02-17
**Priority**: Critical — defeats the purpose of the DSL migration
**Track**: B — Migration Prerequisite
**Blocks**: All "Ready Now" items in `TODO_URGENT_dsl_migration.md`

---

## Problem

The codebase has **~1,350 lines of zero-logic type routing** — union enums, `From` impls,
converter functions, and `Executable` dispatch — all serving one purpose: satisfying
`Dag<T: Executable + Clone + Send + 'static>` type constraints.

Every graph needs a concrete `T`, so each module defines a union enum wrapping its domain
ops + infrastructure ops. Composing graphs (SubDag, workspace) requires converting between
these union types. Adding a new tool costs **~60 lines of boilerplate across 4-5 files**.

This is the exact drift the DSL was built to eliminate.

---

## Root Cause

`execute_with_mode_and_inputs<T: Executable + Clone + Send>` in `core/exec/src/execute.rs`
requires a monomorphic `T`. Each module satisfies this by defining a per-module union enum
(e.g., `PragmaGraphOp`, `CIGraphOp`). Workspace-level composition requires a master union
(`WorkspaceOp`) plus converter functions mapping inner → outer.

Every domain op (PragmaOp, CIOp, etc.) **already implements `Executable`**. The union enums
add no logic — they exist purely to bundle heterogeneous ops into one type.

---

## Fix: `DynOp`

A type-erased wrapper satisfies all executor constraints:

```rust
// In core/exec/src/lib.rs (~20 lines)
use std::sync::Arc;

#[derive(Clone)]
pub struct DynOp(Arc<dyn Executable + Send + Sync>);

impl DynOp {
    pub fn new(op: impl Executable + Send + Sync + 'static) -> Self {
        Self(Arc::new(op))
    }
}

impl fmt::Debug for DynOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Executable for DynOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        self.0.execute(inputs)
    }
}
```

- `Clone`: `Arc::clone` (refcount bump)
- `Send`: `Arc<T: Send + Sync>` is `Send`
- `'static`: `Arc` owns its data
- `Executable`: delegates to inner

One type replaces **all 15 union enums**, **all 16 From impls**, **all 10 converter functions**,
and **all mechanical Executable dispatch**.

---

## Full Inventory

### 15 GraphOp Union Enums (~400 lines)

Each wraps domain + infra ops with an `impl Executable` that just delegates `op.execute(inputs)`.

#### gunbc-dag crate

| Enum | File | Variants | Lines | Notes |
|------|------|----------|-------|-------|
| `WorkspaceOp` | `gunbc-dag/src/workspace/ops.rs:45-96` | 19 | 96 | Master union for workspace DAG |
| `CIGraphOp` | `gunbc-dag/src/ci/graph.rs:71-98` | 6 | 28 | CI + Codegen + CloudEnv + FsEnv + PrepareFileExists + Transport |
| `BuildGraphOp` | `gunbc-dag/src/build/graph.rs:28-47` | 3 | 20 | Build + FsEnv + Transport |
| `CodegenGraphOp` | `gunbc-dag/src/codegen/graph.rs:25-44` | 3 | 20 | Codegen + FsEnv + Transport |
| `DocgenGraphOp` | `gunbc-dag/src/docgen/graph.rs:21-49` | 6 | 29 | Docgen + FsEnv + PrepareFileRead/Write + Blob + Transport |
| `DagVizGraphOp` | `gunbc-dag/src/dag_viz/graph.rs:88-234` | 15+ | 150 | Largest — many visualization-specific ops |

*Note*: `PragmaGraphOp`, `TestgenGraphOp`, `MakegenGraphOp`, `BootstrapGraphOp` use the
`FileOpsGraph<T>` type alias — same pattern but at least DRY via the generic wrapper.

#### lib crates

| Enum | File | Variants | Lines |
|------|------|----------|-------|
| `DepsGraphOp` | `lib/tools/deps/src/graph.rs:29-53` | 5 | 25 |
| `ClippyGraphOp` | `lib/tools/clippy/src/graph.rs:26-49` | 2 | 24 |
| `GistGraphOp` | `lib/tools/gist/src/graph.rs:71-146` | 11 | 76 |
| `GcpSecretManagerGraphOp` | `lib/gcp-ops/src/graph.rs:11-26` | 3 | 16 |
| `GcpDiscoveryGraphOp` | `lib/gcp-ops/src/discovery_graph.rs:59-75` | 4 | 17 |
| `CloudSecretManagerGraphOp` | `lib/cloud-ops/src/graph.rs:25-42` | 4 | 18 |
| `GitHubCredentialGraphOp` | `lib/cloud-ops/src/github_credential_graph.rs:15-32` | 3 | 18 |
| `ReviewGraphOp` | `lib/review/src/graph.rs` | 7+ | ~40 |
| `LlmGraphOp` | `lib/llm-ops/src/graph.rs` | 3 | ~18 |

### 16 From<> Impls for WorkspaceOp (~95 lines)

All in `gunbc-dag/src/workspace/ops.rs:147-241`. Every one is the same pattern:

```rust
impl From<PragmaOp> for WorkspaceOp {
    fn from(op: PragmaOp) -> Self { WorkspaceOp::Pragma(op) }
}
```

### 10 Converter Functions (~80 lines)

All in `gunbc-dag/src/workspace/subdags/`. Each mechanically maps `GraphOp::X(op) => WorkspaceOp::X(op)`:

| Function | File | Lines |
|----------|------|-------|
| `convert_pragma_op` | `subdags/pragma.rs:11-20` | 10 |
| `convert_ci_op` | `subdags/ci.rs:11-22` | 12 |
| `convert_build_op` | `subdags/build.rs:10-16` | 7 |
| `convert_codegen_op` | `subdags/codegen.rs:10-16` | 7 |
| `convert_docgen_op` | `subdags/docgen.rs:11-20` | 10 |
| `convert_testgen_op` | `subdags/testgen.rs:13-26` | 14 |
| `convert_gist_op` | `subdags/gist.rs:19-29` | 11 |
| `convert_dag_viz_op` | `subdags/dag_viz.rs:10-12` | 3 |
| `convert_clippy_op` | `subdags/clippy.rs:11-13` | 3 |
| `convert_language_op` | `subdags/languages.rs` | 3 |

### 3-Hop Exec-Bridge Chain (~90 lines)

The daglang interpreter layer maps `LoweredOp` (string-based) to executable operations
through **three separate enum types** instead of delegating directly to existing ops:

1. **`RuntimeOpId`** — `daglang-lower/src/lib.rs:162-173` (12 lines)
   String name → enum variant classification

2. **`classify_runtime_op()`** — `daglang-lower/src/lib.rs:202-232` (31 lines)
   Match on module + name strings → `RuntimeOpId`

3. **`ResolvedOp`** — `daglang-exec-bridge/src/lib.rs:13-57` (45 lines)
   Separate enum + `Executable` impl with duplicated handler functions

4. **Duplicated handlers** — `daglang-exec-bridge/src/lib.rs:159-368` (~210 lines)
   `execute_load_registry()`, `execute_render_makefile()`, etc. — **reimplements** the
   same operations that already exist in `makegen/ops.rs` instead of delegating.

Currently only supports **makegen**. Extending to 6 more modules would multiply this boilerplate.

### Supporting Infrastructure (~80 lines)

| File | Purpose | Lines |
|------|---------|-------|
| `gunbc-dag/src/file_ops_graph.rs` | `FileOpsGraph<T>` generic wrapper | 42 |
| `gunbc-dag/src/workspace/convert.rs` | `convert_dag()` + `convert_node()` utilities | 37 |

### Manual Graph Builders (~4,000 lines)

7 `graph.rs` files with manual `DagBuilder` wiring, superseded by DSL `.dag` files:

| Module | File | Builder Lines |
|--------|------|---------------|
| pragma | `gunbc-dag/src/pragma/graph.rs` | ~200 |
| codegen | `gunbc-dag/src/codegen/graph.rs` | ~240 |
| makegen | `gunbc-dag/src/makegen/graph.rs` | ~180 |
| ci | `gunbc-dag/src/ci/graph.rs` | ~1,000 |
| bootstrap | `gunbc-dag/src/bootstrap/graph.rs` | ~230 |
| build | `gunbc-dag/src/build/graph.rs` | ~200 |
| docgen | `gunbc-dag/src/docgen/graph.rs` | ~270 |

DSL equivalents exist in `dsl/tools/*.dag` and `dsl/pipelines/ci.dag`.
Parity tests in `daglang-lower` confirm structural equivalence.

---

## Implementation Plan

### Step 1: Add `DynOp` to `core/exec/src/lib.rs`

~20 lines. See code sketch above. This is the foundation — everything else depends on it.

### Step 2: Add central resolver `gunbc-dag/src/resolve.rs`

~150 lines. One function maps `LoweredOp` string names → existing domain ops via `DynOp::new()`.
**No reimplementation** — delegates to `PragmaOp`, `CIOp`, `TransportOps`, `FsEnv`, etc.

```rust
pub fn resolve_lowered_op(op: &LoweredOp) -> Result<DynOp, ResolveError> {
    match (module.as_str(), name.as_str()) {
        // Domain ops — delegate to existing Executable impls
        ("tools.pragma", "render_clippy") => Ok(DynOp::new(PragmaOp::RenderClippy)),
        ("tools.pragma", "render_allowlist") => Ok(DynOp::new(PragmaOp::RenderAllowlist)),
        ("tools.pragma", "render_policy") => Ok(DynOp::new(PragmaOp::RenderLintPolicy)),
        // ... other modules ...

        // Infrastructure ops — shared across all modules
        (_, n) if n.contains("fs_env") => Ok(DynOp::new(FsEnv::new(Scope::Write))),
        (_, n) if n.contains("prepare_read") => Ok(DynOp::new(PrepareFileReadOp)),
        (_, n) if n.contains("prepare_write") => Ok(DynOp::new(PrepareFileWriteOp)),
        (_, n) if n.contains("compare") => Ok(DynOp::new(BlobOps::CompareContent)),
        (_, n) if is_transport(n) => Ok(DynOp::new(TransportOps::Execute)),

        _ => Err(ResolveError { ... }),
    }
}

pub fn resolve_lowered_dag(dag: &Dag<LoweredOp>) -> Result<Dag<DynOp>, ResolveError> {
    // Walk nodes, resolve each op, copy edges
}
```

**Dependency**: add `daglang-lower` to `gunbc-dag/Cargo.toml`

### Step 3: Add DSL builder per module

~15 lines each, ~105 total. Each module gets a builder that compiles its `.dag` file:

```rust
pub fn build_pragma_graph() -> Result<Dag<DynOp>, CompileError> {
    let output = compile_from_context(&DriverContext {
        roots: vec![PathBuf::from("dsl")],
        target_file: Some(PathBuf::from("dsl/tools/pragma.dag")),
    })?;
    resolve_lowered_dag(&output.lowered_dag)
}
```

**Dependency**: add `daglang-driver` to `gunbc-dag/Cargo.toml`

### Step 4: Delete manual graph.rs builders (7 files, ~4,000 lines)

Delete `DagBuilder` construction code from pragma, codegen, makegen, ci, bootstrap, build, docgen.

**Keep**: `ops.rs` files (domain ops + Executable impls) — these are real logic, not boilerplate.

**Move**: signature functions (`pragma_signature()`, etc.) and CI config helpers
(`ci_workflow_config()`, `ci_integrations()`) to `ops.rs` or a new `config.rs`.

**Update**: `graph_mock.rs` files — change `builder = "..."` expressions in testgen macros
to reference the new DSL-based builders.

### Step 5: Delete boilerplate layer (~600 lines)

- `gunbc-dag/src/workspace/ops.rs` — `WorkspaceOp` enum + 16 From impls + Executable dispatch
- `gunbc-dag/src/workspace/convert.rs` — `convert_dag()` + `convert_node()`
- `gunbc-dag/src/workspace/subdags/*.rs` — remove converter functions; simplify each to:
  compile `.dag` file → wrap result in `Node::subdag()`
- `gunbc-dag/src/file_ops_graph.rs` — `FileOpsGraph<T>` no longer needed
- Per-module GraphOp enums: `BuildGraphOp`, `CodegenGraphOp`, `CIGraphOp`, `DocgenGraphOp`

### Step 6: Delete/simplify `daglang-exec-bridge`

The central resolver (Step 2) replaces all of:
- `ResolvedOp` enum + `Executable` impl
- `RuntimeOpId` enum + `classify_runtime_op()`
- All duplicated handler functions (`execute_load_registry()`, etc.)

Keep mock helper constructors (`makegen_dry_run_transport_mocks()`, etc.) if still needed,
or move them to the module that uses them.

### Step 7: Update callers

- `gunbc-dag/src/bin/*.rs` — `build_*_graph()` now returns `Dag<DynOp>`.
  Executor functions are generic (`T: Executable + Clone + Send`), so this just works.
- `gunbc-dag/src/lib.rs` — remove GraphOp type re-exports
- `graph_mock.rs` files — update `builder = "..."` in `#[testgen_target]` macros
- `daglang-lower` parity tests — delete (no manual builder to compare against)
- `tests/workflow_acceptance.rs` — update `build_ci_graph_with_mode` references
- `tests/integration_fs.rs` — update `build_makegen_graph`, `build_bootstrap_graph`
- `tests/resource_registry_coverage.rs` — update builder references

### Step 8: Future wave — lib crates

Same `DynOp` pattern eliminates boilerplate in:
- `lib/tools/deps/src/graph.rs` (DepsGraphOp)
- `lib/tools/clippy/src/graph.rs` (ClippyGraphOp)
- `lib/tools/gist/src/graph.rs` (GistGraphOp)
- `lib/gcp-ops/src/graph.rs` (GcpSecretManagerGraphOp)
- `lib/gcp-ops/src/discovery_graph.rs` (GcpDiscoveryGraphOp)
- `lib/cloud-ops/src/graph.rs` (CloudSecretManagerGraphOp)
- `lib/cloud-ops/src/github_credential_graph.rs` (GitHubCredentialGraphOp)
- `lib/review/src/graph.rs` (ReviewGraphOp)
- `lib/llm-ops/src/graph.rs` (LlmGraphOp)

---

## Verification

1. `cargo test --workspace` — all tests pass
2. `cargo clippy --all-targets -- -D warnings` — 0 warnings
3. `cargo run -p gunbc-dag --bin gunbc-pragma -- --dry-run` — each binary works
4. `cargo run -p gunbc-dag --bin gunbc-workspace -- --dry-run` — workspace DAG works

## Impact

- **Deleted**: ~5,950 lines (builders + GraphOp enums + WorkspaceOp + converters + exec-bridge)
- **Added**: ~300 lines (DynOp + resolver + DSL builders)
- **Net**: ~5,650 lines removed
- **Future**: lib crate cleanup removes another ~300 lines
