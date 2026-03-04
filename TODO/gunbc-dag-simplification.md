# Lane 2: Compiler Debt & Application Layer Cleanup

> We're at an early stage. Debt this early is bizarre — it's design via implementation.
> For each piece of debt: think ahead to the truly minimal final representation.
> Go a → f directly, not a → b → c → d → e → f.

> **Anti-patterns from retrospective**:
> - Don't add intermediate abstractions (DynOp type-dispatch: built 5,950 LOC, then deleted it all)
> - Don't build features without understanding invariants (FnBodyDelegate: built, broke on port naming, reverted)
> - Don't optimize what you're about to delete (Bridges 6-7: keep as-is, they get deleted not improved)
> - `@annotation("string")` is never the final state — go straight to structural blocks

---

## The 10 Accidental Bridges

Each bridge exists because the compiler doesn't do enough. For each: current state, final state, what to delete.

### Delete immediately (zero blockers)

**Bridge 4: `OutputPathMetadataOp`** — identity node carrying `@outputs` annotations through DAG.

- **Location**: `core/resolve/src/resolve.rs:359-365` (~7 LOC)
- **Current**: `@outputs("glob")` creates a passthrough node at resolve time
- **Final**: Already done. `extract_outputs_annotation()` in the lowerer extracts paths at compile time. The runtime node does literally nothing.
- **Acceptance**: `OutputPathMetadataOp` struct **deleted** from `resolve.rs`. `grep -r 'OutputPathMetadataOp' core/` returns 0. All tests pass.
- **Effort**: Trivial.

**Bridge 10: `PipelineDispatchOp`** — stage routing as a runtime node.

- **Location**: `core/resolve/src/resolve.rs:268-352` (~85 LOC)
- **Current**: Creates pipeline dispatch nodes with stage progression logic
- **Final**: Pipelines are either (a) inlined as SubDag chains with edge dependencies, or (b) metadata for static generation (Makefile targets). `strip_pipeline_nodes()` in `core/resolve/src/builder.rs:92-113` already strips these before resolution.
- **Acceptance**: If dead code: `PipelineDispatchOp` struct **deleted**. `strip_pipeline_nodes()` **deleted**. `grep -r 'PipelineDispatchOp\|strip_pipeline_nodes' core/` returns 0. If alive: design doc explaining first-class execution vs metadata-only.
- **Effort**: Trivial if dead. Design decision if alive.

**Bridge 5: `dsl_builder.rs`** — 8+ wrapper functions that each pass `GunbcExternResolver`.

- **Location**: `core/resolve/src/builder.rs` (335 LOC, 8 public functions)
- **Current**: `build_dsl_graph()`, `build_tool_graph()`, `build_dsl_graph_with_types()`, `build_dsl_graph_with_types_and_profile()`, `build_dsl_graph_with_profile()`, `build_dsl_graph_for_entry()`, `build_dsl_graph_for_entrypoint()`, `tool_signature()` — differ only in optional parameters
- **Final**: One function: `build_dsl_graph(module, opts: BuildOpts)` where `BuildOpts` has optional `profile`, `entry_func`, `include_types`, `tool_mode`. The 8 variants become callers setting different defaults.
- **Acceptance**: `builder.rs` has exactly 1 public `build_dsl_graph()` + `BuildOpts` struct + `tool_signature()` (if needed separately). Line count drops from 335 to ~150. `grep -c 'pub fn build_' core/resolve/src/builder.rs` returns 1.
- **Effort**: Small.

---

### Compiler fixes (medium effort, go directly to final state)

**Bridge 1: `DeclaredOutputCallableOp`** — SubDag wrapper that maps inner results to outer ports.

- **Location**: `core/resolve/src/resolve.rs:153-161` (~9 LOC struct, but creation sites + wiring elsewhere)
- **Current**: Lowerer creates `Callable` node with `fn_body`. Resolver wraps in `DeclaredOutputCallableOp` to passthrough `__out:field` → declared output ports.
- **Incremental trap**: Make the wrapper smarter, add port introspection, etc.
- **Final state**: Lowerer produces SubDag directly for fn/func/pattern items. No Callable wrapper. SubDag's internal unconnected ports map directly to parent DAG ports (I2.4 invariant).
- **Acceptance**: `DeclaredOutputCallableOp` struct **deleted**. `grep -r 'DeclaredOutputCallableOp' core/` returns 0. `lower_fn_item()` in `daglang-lower` returns `LoweredOp::SubDag`. Tests: `box_draw.dag`, `markdown_render.dag` compile and produce correct output.
- **Depends on**: Nothing.
- **Effort**: Medium.

**Bridge 2: `FnBodyCallableOp`** — evaluates fn bodies at runtime via `evaluate_fn_body()`.

- **Location**: `core/resolve/src/resolve.rs:173-258` (~86 LOC)
- **Current**: Pure DSL fn items are stored as `LoweredFnBody` in the IR. At runtime, resolver creates `FnBodyCallableOp` which calls `evaluate_fn_body()`.
- **Incremental trap**: Cache fn evaluations, add memoization, etc.
- **Final state**: All fn bodies are lowered to SubDag form at compile time. No `LoweredFnBody` in the runtime IR. `evaluate_fn_body()` exists only in the lowerer for compile-time evaluation (data declarations, config rendering). Runtime never evaluates fn bodies — it executes SubDag nodes.
- **Acceptance**: `FnBodyCallableOp` struct **deleted**. `fn_body` field **deleted** from `LoweredOp::Callable`. `grep -r 'FnBodyCallableOp\|LoweredFnBody' core/resolve/` returns 0. `evaluate_fn_body()` remains in `daglang-lower/src/eval.rs` but is only called at compile time.
- **Depends on**: Bridge 1 (SubDag direct lowering).
- **Enables**: RF-E5 test restoration (`makegen_runtime_differential`).
- **Effort**: Medium.

**Bridge 3: `CollectionDelegate`** — collection ops (map/filter/fold) delegate to evaluator.

- **Location**: `core/resolve/src/resolve.rs:456-472` (~17 LOC) + `daglang-lower/src/eval.rs` `evaluate_collection()` (~50 LOC at line 87)
- **Current**: `LoweredOp::Collection` nodes are resolved as `CollectionDelegate` which calls `evaluate_collection()` from the lowerer at runtime.
- **Incremental trap**: Move evaluator functions into resolver, create per-op structs.
- **Final state**: Collection operations are proper IR nodes (`MapNode`, `FilterNode`, `FoldNode` already exist in `core/ir/src/patterns/`). The lowerer produces these IR nodes. The executor handles them. No evaluator delegation.
- **Acceptance**: `CollectionDelegate` struct **deleted**. `LoweredOp::Collection` variant **deleted** from `daglang-lower/src/lib.rs`. `evaluate_collection()` **deleted** from `eval.rs`. `grep -r 'CollectionDelegate\|evaluate_collection\|LoweredOp::Collection' core/` returns 0. Lowerer emits `MapNode`/`FilterNode`/`FoldNode` from `core/ir/src/patterns/`.
- **Effort**: Medium.

**Bridge 8: `add_fs_env_root_node()`** — filesystem resource injected at resolver time.

- **Location**: `core/resolve/src/fs_env.rs:7-20` (~14 LOC) + `DslFsEnvOp` in `resolve.rs`
- **Current**: `add_fs_env_root_node()` called by DSL builders after lowering to inject filesystem capability.
- **Incremental trap**: Make injection configurable, add resource composition.
- **Final state**: Lowerer detects file I/O operations during lowering and auto-injects resource acquisition nodes. DSL `uses fs: Filesystem(mode: Write)` declarations drive insertion — the lowerer honors them, not the resolver.
- **Acceptance**: `add_fs_env_root_node()` **deleted** from `fs_env.rs`. Callers in `builder.rs` no longer call it. `grep -r 'add_fs_env_root_node' core/` returns 0. Resource acquisition nodes inserted by lowerer when `uses` declarations or file operations are present.
- **Effort**: Medium.

**Bridge 9: `GenericFilePrepareOp`/`GenericFileParseOp`** — generic adapters for file transport.

- **Location**: `core/resolve/src/service_ops/service_ops_impl.rs:1133-1230` (~100 LOC combined)
- **Current**: File I/O goes through generic adapters that interpret `FileOp::Read`/`Write`/etc.
- **Incremental trap**: Unify all file ops into one adapter, add caching.
- **Final state**: File transport handled by typed IR nodes, same as REST/Shell. `transport file { op: READ, path: "..." }` lowers to a `FileRequest` builder node + `FileResponse` parser node. No generic adapter indirection.
- **Acceptance**: `GenericFilePrepareOp` and `GenericFileParseOp` structs **deleted**. `grep -r 'GenericFilePrepareOp\|GenericFileParseOp' core/` returns 0. File transport uses same prepare/execute/parse triplet pattern as REST and Shell.
- **Effort**: Medium.

---

### Blocked on larger design work

**Bridge 6: `registry_tools_to_value()`** — manual Rust struct → `Value::Map` conversion for tool registry.
**Bridge 7: `DiscoverToolsOp`** — runtime tool registry discovery.

- **Locations**: `core/codegen/src/makegen/shared.rs:84-178` (~95 LOC) + `gunbc-dag/src/extern_ops.rs:151-178` (~28 LOC)
- **Current**: Tool registry lives as Rust structs built from `inventory`. Runtime converts to `Value::Map` for DSL consumption.
- **Final state**: Tool registry is a compile-time artifact. Compiler scans `dsl/tools/*.dag`, extracts entrypoints, emits `dsl/generated/tool_registry.dag`. Makegen/bootstrap import this artifact directly. No runtime discovery. No struct→Value conversion.
- **Acceptance**: `registry_tools_to_value()` **deleted** from `shared.rs`. `DiscoverToolsOp` **deleted** from `extern_ops.rs`. `dsl/generated/tool_registry.dag` exists and is consumed by `tools/makegen.dag` imports. `grep -r 'registry_tools_to_value\|DiscoverToolsOp' .` returns 0.
- **Cannot go a→f yet**: Requires compiler artifact emission. 2-3 items of work.
- **Interim**: Keep as-is. Don't optimize, don't cache, don't refactor. These will be **deleted**, not improved.

---

## Application Layer Cleanup

Once compiler fixes land, the application layer (currently gunbc-dag) becomes simple:

### Rename

`gunbc-dag` → `gunbc-app` (or `gunbc-bin`). It's the application entry point, not a DAG definition crate.

- **Acceptance**: `Cargo.toml` `[package] name = "gunbc-app"`. Directory renamed. All `Cargo.toml` dependency references updated. `grep -r 'gunbc-dag' Cargo.toml */Cargo.toml` returns 0.

### Output directory: single source of truth

Currently output paths are specified in three places:
1. `core/ir/src/workspace_layout.rs` — hardcoded `"target/codegen/bin"` constants
2. `codegen_cli.rs` — calls `WorkspaceLayout` with string fallback
3. `gunbc-dag/Cargo.toml` — `[[bin]]` entries: `path = "../target/codegen/bin/*/main.rs"`

Plus `dsl/config/codegen_paths.dag` which declares intent but isn't consumed.

- **Final state**: `dsl/config/codegen_paths.dag` is the source of truth. `WorkspaceLayout` derives from it. `Cargo.toml` `[[bin]]` entries are generated by codegen from the same source.
- **Acceptance**: `grep -r '"target/codegen/bin"' core/ir/` returns 0 (hardcoded constants deleted). `WorkspaceLayout::new()` reads from compiled DSL config. One declaration, three consumers.

### Testgen engine

Testgen is split between `core/codegen/src/testgen/` (14,400 LOC library) and `gunbc-dag/src/testgen_dag/` (1,500 LOC orchestration) due to a real circular dependency with testgen-registry.

**Question**: Is the meta-circular testgen (testgen is itself a DAG that generates tests) the right design? Or should `daglang compile --emit=tests` be a compiler mode? The latter would eliminate the testgen DAG, the circular dependency, and the 1,500 LOC orchestration layer entirely.

- **Acceptance (if compiler mode)**: `gunbc-dag/src/testgen_dag/` **deleted** (~1,500 LOC). `daglang compile --emit=tests` produces test files. Testgen-registry circular dependency eliminated.

### What remains after cleanup

| Component | LOC | Why it stays |
|-----------|-----|--------------|
| `bin/ci.rs` | 267 | Bootstrap (runs before codegen) |
| `bin/codegen_cli.rs` | 959 | The generator (can't generate itself) |
| `resolve.rs` | ~30 | `GunbcExternResolver` (app-specific extern dispatch) |
| `extern_ops.rs` | shrinking | Only operations blocked on missing compiler features |
| 17 generated `[[bin]]` entries | — | Templates pointing to generated main.rs |

Everything else either moves into the compiler (bridges 1-3, 8-9), gets deleted (bridges 4-5, 10), or waits for artifact emission (bridges 6-7).

---

## Execution Order

| Priority | Items | Effort | Deleted |
|----------|-------|--------|---------|
| **Now** | Delete bridges 4, 5, 10 | 1-2 days | `OutputPathMetadataOp` (7 LOC), `PipelineDispatchOp` (85 LOC), `strip_pipeline_nodes()` (22 LOC), 6 redundant builder functions (~185 LOC) |
| **Next** | Bridge 1 + Bridge 2 | 3-5 days | `DeclaredOutputCallableOp` (9 LOC), `FnBodyCallableOp` (86 LOC), `LoweredFnBody` type, `fn_body` field on `LoweredOp::Callable` |
| **Next** | Bridge 3 + Bridge 8 | 3-5 days | `CollectionDelegate` (17 LOC), `evaluate_collection()` (50 LOC), `LoweredOp::Collection` variant, `add_fs_env_root_node()` (14 LOC), `DslFsEnvOp` |
| **Next** | Bridge 9 | 2-3 days | `GenericFilePrepareOp` (44 LOC), `GenericFileParseOp` (50 LOC) |
| **Then** | Rename crate, output dir consolidation | 1-2 days | Hardcoded path constants in `workspace_layout.rs` |
| **Later** | Bridges 6-7 (artifact emission) | 1-2 weeks | `registry_tools_to_value()` (95 LOC), `DiscoverToolsOp` (28 LOC) |

## Deleted Tests (re-add when bridges eliminated)

| ID | Tests | Fixed by |
|----|-------|----------|
| RF-E5 | `makegen_runtime_differential_interpreter_vs_generated_rust_layer1` | Bridge 2 (FnBodyCallableOp elimination) |
| RF-E6 | `makegen_exec_runtime_e2e`, `pragma_exec_runtime_e2e` + generated variants | Bridge 1 + 2 (SubDag lowering) |

## Future (post-cleanup)

| ID | Task | Notes |
|----|------|-------|
| C28-P2 | Daggen cache manager (content-hash → `.dagbin` → skip recompilation) | Infrastructure ready |
| C28-P3 | Daggen codegen integration (serialize at `make codegen` time) | Eliminates runtime DSL parsing |

---

## Success Criteria

1. **Zero accidental bridges** — every remaining bridge is architecturally necessary
2. **Lowerer does the work** — fn bodies lowered to SubDag, resource nodes injected at lower time, collection ops as IR nodes
3. **Resolver is thin** — resolves extern symbols, that's it
4. **Output directory** specified once, derived everywhere
5. **Crate named correctly** — reflects that it's the application layer
6. **No intermediate refactors** — each change goes directly to final state
7. **Each step verified by grep** — deleted items confirmed absent from codebase
