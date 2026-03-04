# Lane 2: Compiler Debt & Application Layer Cleanup

> We're at an early stage. Debt this early is bizarre — it's design via implementation.
> For each piece of debt: think ahead to the truly minimal final representation.
> Go a → f directly, not a → b → c → d → e → f.

---

## The 10 Accidental Bridges

Each bridge exists because the compiler doesn't do enough. For each: current state, final state, and whether we can skip the intermediate steps.

### Delete immediately (zero blockers)

**Bridge 4: `OutputPathMetadataOp`** — identity node carrying `@outputs` annotations through DAG.

- **Current**: `@outputs("glob")` creates a passthrough node at resolve time
- **Final**: Already done. `extract_outputs_annotation()` in the lowerer extracts paths at compile time. The runtime node does literally nothing. Delete it.
- **Effort**: Trivial. Delete ~15 lines in `resolve.rs`, remove creation site.

**Bridge 10: `PipelineDispatchOp`** — stage routing as a runtime node.

- **Current**: Creates pipeline dispatch nodes with stage progression logic
- **Final**: Pipelines are either (a) inlined as SubDag chains with edge dependencies, or (b) metadata for static generation (Makefile targets). Current code already strips pipeline nodes before resolution (`strip_pipeline_nodes()`). This op may be dead code.
- **Action**: Verify if used. If dead, delete. If alive, decide: first-class execution or metadata-only.
- **Effort**: Trivial if dead. Design decision if alive.

**Bridge 5: `dsl_builder.rs`** — 7 wrapper functions that each pass `GunbcExternResolver`.

- **Current**: `build_dsl_graph()`, `build_tool_graph()`, `build_dsl_graph_with_types()`, etc. — 7 functions that differ only in optional parameters
- **Final**: One function: `build_dsl_graph(module, opts: BuildOpts)` where `BuildOpts` has optional `profile`, `entry_func`, `include_types`. The 7 variants are callers setting different defaults.
- **Effort**: Small. Refactor to single entry point + options struct.

---

### Compiler fixes (medium effort, go directly to final state)

**Bridge 1: `DeclaredOutputCallableOp`** — SubDag wrapper that maps inner results to outer ports.

- **Current**: Lowerer creates `Callable` node with `fn_body`. Resolver wraps in `DeclaredOutputCallableOp` to passthrough `__out:field` → declared output ports.
- **Incremental trap**: Make the wrapper smarter, add port introspection, etc.
- **Final state**: Lowerer produces SubDag directly for fn/func/pattern items. No Callable wrapper. SubDag's internal unconnected ports map directly to parent DAG ports (this is what SubDag already does for non-callable items). No resolver wrapper needed.
- **Why a→f works**: SubDag already has this port-mapping semantics (I2.4 invariant). The Callable→SubDag indirection exists because the lowerer historically didn't produce SubDags for callables.
- **Effort**: Medium. Modify `lower_fn_item()` in daglang-lower to return SubDag. Delete DeclaredOutputCallableOp (~70 lines). Test with box_draw.dag, markdown_render.dag.

**Bridge 2: `FnBodyCallableOp`** — evaluates fn bodies at runtime via `evaluate_fn_body()`.

- **Current**: Pure DSL fn items are stored as `LoweredFnBody` in the IR. At runtime, resolver creates `FnBodyCallableOp` which calls `evaluate_fn_body()`.
- **Incremental trap**: Cache fn evaluations, add memoization, etc.
- **Final state**: All fn bodies are lowered to SubDag form at compile time. No `LoweredFnBody` in the runtime IR. `evaluate_fn_body()` exists only in the lowerer for compile-time evaluation (data declarations, config rendering). Runtime never evaluates fn bodies — it executes SubDag nodes.
- **Depends on**: Bridge 1 (SubDag direct lowering).
- **Effort**: Medium. Remove `fn_body` field from `LoweredOp::Callable`. All fn call sites lower to SubDag invocation.

**Bridge 3: `CollectionDelegate`** — collection ops (map/filter/fold) delegate to evaluator.

- **Current**: `LoweredOp::Collection` nodes are resolved as `CollectionDelegate` which calls `evaluate_collection()` from the lowerer at runtime.
- **Incremental trap**: Move evaluator functions into resolver, create per-op structs.
- **Final state**: Collection operations are proper IR nodes (`MapNode`, `FilterNode`, `FoldNode` already exist in `core/ir/src/patterns/`). The lowerer produces these IR nodes. The executor handles them. No evaluator delegation.
- **Why a→f works**: The IR node types already exist. The lowerer just doesn't use them — it creates `Collection` variants that delegate instead.
- **Effort**: Medium. Wire lowerer to emit IR collection nodes instead of `LoweredOp::Collection`. Extend executor for any missing collection types.

**Bridge 8: `FsEnvRootOp`** — filesystem resource injected at resolver time.

- **Current**: `add_fs_env_root_node()` called by DSL builders after lowering to inject filesystem capability.
- **Incremental trap**: Make injection configurable, add resource composition.
- **Final state**: Lowerer detects file I/O operations during lowering and auto-injects resource acquisition nodes. DSL already has `uses fs: Filesystem(mode: Write)` declarations — the lowerer should honor them by inserting the resource nodes, not leave it to the resolver.
- **Effort**: Medium. Move `add_fs_env_root_node()` logic into lowerer when `uses` declarations or file operations are present. Delete resolver-time injection.

**Bridge 9: `FileReadPrepareOp`/`FileWritePrepareOp`** — generic adapters for file transport.

- **Current**: File I/O goes through `GenericFilePrepareOp`/`GenericFileParseOp` — generic adapters that interpret `FileOp::Read`/`Write`/etc.
- **Incremental trap**: Unify all file ops into one adapter, add caching.
- **Final state**: File transport is handled by typed IR nodes, same as REST/Shell. `transport file { op: READ, path: "..." }` lowers to a `FileRequest` builder node (simple data struct) + `FileResponse` parser node. No generic adapter indirection.
- **Effort**: Medium. Define file-specific IR node types (or extend service transport nodes). Modify lowerer to produce them.

---

### Blocked on larger design work

**Bridge 6: `registry_tools_to_value()`** — manual Rust struct → `Value::Map` conversion for tool registry.
**Bridge 7: `discover_tools_op`** — runtime tool registry discovery.

- **Current**: Tool registry lives as Rust structs built from `inventory`. Runtime converts to `Value::Map` for DSL consumption.
- **Final state**: Tool registry is a compile-time artifact. Compiler scans `dsl/tools/*.dag`, extracts entrypoints, emits `dsl/generated/tool_registry.dag`. Makegen/bootstrap import this artifact directly. No runtime discovery. No struct→Value conversion.
- **Cannot go a→f yet**: Requires compiler artifact emission (Phase 7b from extern bridge analysis). 2-3 items of work. But this IS the final state — don't build intermediate caching or optimization layers.
- **Interim**: Keep as-is, don't optimize. These will be deleted, not improved.

---

## Application Layer Cleanup

Once compiler fixes land, the application layer (currently gunbc-dag) becomes simple:

### Rename

`gunbc-dag` → `gunbc-app` (or `gunbc-bin`). It's the application entry point, not a DAG definition crate.

### Output directory: single source of truth

Currently output paths are specified in three places:
1. `core/ir/src/workspace_layout.rs` — hardcoded `"target/codegen/bin"` constants
2. `codegen_cli.rs` — calls `WorkspaceLayout` with string fallback
3. `gunbc-dag/Cargo.toml` — `[[bin]]` entries: `path = "../target/codegen/bin/*/main.rs"`

Plus `dsl/config/codegen_paths.dag` which declares intent but isn't consumed.

**Final state**: DSL config is the source of truth. `WorkspaceLayout` derives from it. `Cargo.toml` `[[bin]]` entries are generated by codegen from the same source. One declaration, three consumers.

### Testgen engine

Testgen is split between `core/codegen/src/testgen/` (14,400 LOC library) and `gunbc-dag/src/testgen_dag/` (1,500 LOC orchestration) due to a real circular dependency with testgen-registry.

**Question**: Is the meta-circular testgen (testgen is itself a DAG that generates tests) the right design? Or should `daglang compile --emit=tests` be a compiler mode? The latter would eliminate the testgen DAG, the circular dependency, and the 1,500 LOC orchestration layer entirely.

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

| Priority | Items | Effort | Effect |
|----------|-------|--------|--------|
| **Now** | Delete bridges 4, 5, 10 | 1-2 days | Remove ~100 LOC of dead/boilerplate code |
| **Next** | Bridge 1 (SubDag direct lowering) + Bridge 2 (compile-time fn bodies) | 3-5 days | Eliminate 2 resolver ops, simplify fn call pipeline |
| **Next** | Bridge 3 (IR collection nodes) + Bridge 8 (lowerer resource injection) | 3-5 days | Eliminate evaluator delegation, clean resource model |
| **Next** | Bridge 9 (typed file transport) | 2-3 days | Eliminate generic file adapters |
| **Then** | Rename crate, output dir consolidation | 1-2 days | Naming + single source of truth |
| **Later** | Bridges 6-7 (artifact emission) | 1-2 weeks | Eliminate runtime tool discovery entirely |

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
