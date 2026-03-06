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

## Postmortem: `make gist` Credential Chain Failure (2026-03-04)

**Symptom**: `make gist` fails with DNS error trying to reach `metadata.google.internal` on a local WSL2 machine. The `audience` parameter arrives as the string `"true"` (a boolean) instead of `"sigstore"`.

**Root cause**: Two compounding compiler bugs, both caused by the dual execution model (Bridges 1+2).

### Bug 1: Unguarded transport nodes for non-matching match arms

The credential chain has `match runtime { LocalDev => local_auth(), Metadata => metadata_oidc(audience) }`. The fn body evaluator (`evaluate_fn_body()`) correctly picks `local_auth()`. But the DAG lowerer creates transport triplets for **all** service calls referenced in the module — including `gcp.Metadata.GetIdentityToken` from the `metadata_oidc` branch. These transport nodes are top-level DAG nodes with no guards, so they execute unconditionally regardless of which match arm was taken.

**Fix**: Bridge 1+2 elimination. When fn bodies are lowered to SubDags, match arms become guarded SubDag branches. Transport nodes inside the `Metadata` branch would be guarded and produce `Value::Skipped` when runtime ≠ Metadata.

### Bug 2: Default parameter values not injected into DAG

`acquire_gcp_secret(audience: NonEmptyStr = "sigstore")` is called without passing `audience`. The lowerer creates a port for `audience` but does NOT create a literal source node with the default `"sigstore"`. The port has no incoming edge, so the executor fills it with a mock value (`Bool(true)` → string `"true"` via `input_as_string`).

**Fix**: Lowerer must emit literal source nodes for parameters with defaults when callers omit them. This is a missing compiler feature, independent of but exacerbated by Bridges 1+2.

### Why this matters

This is exactly what the compiler is supposed to prevent. The DSL declarations are correct — types are right, match arms are right, defaults are right. But the compiler's dual execution model (interpreter for fn bodies + DAG executor for transport) means:
- Conditional logic doesn't suppress unreachable transport nodes
- Default values don't flow through to nested service calls
- The failure manifests as an opaque DNS error, not a compile-time diagnostic

**This postmortem is the case study for why Bridges 1+2 are the highest-priority compiler fixes.**

---

## The 10 Accidental Bridges

Each bridge exists because the compiler doesn't do enough. For each: current state, final state, what to delete.

### Delete immediately (zero blockers)

**Bridge 4: `OutputPathMetadataOp`** — **DONE** (already deleted from main).

**Bridge 10: `PipelineDispatchOp`** — **DONE**. `strip_pipeline_nodes()` deleted. Resolver silently skips Pipeline nodes (metadata-only, not executed). 22 LOC removed from builder.rs.

**Bridge 5: `dsl_builder.rs`** — **DONE**. Consolidated 6 pub fns to 1: `build_dsl_graph(module, resolver, opts: BuildOpts)`. Dead code deleted (`build_tool_graph`, `tool_signature`). 201 LOC (from ~335).

**Bridge 2b: Default parameter injection** — **DONE** (lowerer injects literal source nodes for omitted call args with defaults; daglang-lower/src/lib.rs:9164-9189).

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
- **Enables**: RF-E5 test restoration (`makegen_runtime_differential`). **Fixes `make gist` postmortem Bug 1** (unguarded transport nodes for non-matching match arms). **Also fixes P0-5 / P1-1** from `docs/review/gap-analysis-tasks.md` (`make install` fails on unresolvable compute nodes).
- **Effort**: Medium.

**Bridge 2b: Default parameter injection** — missing compiler feature.

- **Location**: `core/daglang/daglang-lower/src/lib.rs`, `wire_fn_call_arguments()` (~lines 8976-9108)
- **Current**: When a caller omits a parameter with a default value (e.g., `audience: NonEmptyStr = "sigstore"`), the lowerer creates a port but does NOT inject a literal source node with the default. The port has no incoming edge. At runtime, the executor fills it with a mock/garbage value.
- **Final state**: Lowerer reads `param.default` from the callee's `Param` definition. For any parameter not explicitly passed by the caller, creates a `PrimitiveLiteral` source node with the default expression's value and wires it to the port.
- **Acceptance**: Test: `func f(x: String = "hello") -> { out: String } { return { out: x } }` called as `f()` produces `"hello"`. `make gist` no longer sends `audience=true`. Default values flow through nested pattern/func calls.
- **Fixes**: `make gist` postmortem Bug 2 (default parameter values not injected).
- **Effort**: Small-Medium. The `Param.default` field already exists in the AST.

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

**Bridge 11: Shell hermeticity annotation** -- hermetic vs external classification erased at transport layer.

- **Location**: `core/ir/src/transport/mod.rs` (ShellRequest struct)
- **Design doc**: `docs/design/shell-hermeticity-annotation.md`
- **Current**: `TransportRequest::Shell(ShellRequest)` erases whether the producer is hermetic (local, deterministic, no external network/auth) or external. `git ls-files` and `gh gist create` become structurally identical after lowering.
- **Incremental trap**: Infer hermeticity from command string prefixes ("git" -> hermetic).
- **Final state**: `ShellProducerSemantics` struct with `Hermeticity` enum (Hermetic | External) as optional field on `ShellRequest`. Producers set it at construction time. Testgen categorization uses it. Strict mode rejects `None` for classified workflows.
- **Acceptance**: `ShellProducerSemantics` + `Hermeticity` types exist in `core/ir/src/transport/mod.rs`. `ShellRequest.semantics` field populated by git, cargo, gist, shell producers. Testgen reads `semantics.hermeticity`. `grep -r 'ShellProducerSemantics' core/ir/` returns 2+ hits (definition + usage). `grep -r 'Hermeticity' core/ir/` returns 2+ hits.
- **Depends on**: Nothing. Can run in parallel with all other bridges.
- **Effort**: Medium.
- **Status**: Types + field + builder method DONE. Git (Hermetic), GitHub CLI (External), Cargo (Hermetic), GCP auth (External), GitHub PR (External) producers annotated.

---

### Blocked on larger design work

**Bridge 6: `registry_tools_to_value()`** — manual Rust struct → `Value::Map` conversion for tool registry.
**Bridge 7: `DiscoverToolsOp`** — runtime tool registry discovery.

- **Locations**: `core/codegen/src/makegen/shared.rs:84-178` (~95 LOC) + `gunbc-app/src/extern_ops.rs:151-178` (~28 LOC)
- **Current**: Tool registry lives as Rust structs built from `inventory`. Runtime converts to `Value::Map` for DSL consumption.
- **Final state**: Tool registry is a compile-time artifact. Compiler scans `dsl/tools/*.dag`, extracts entrypoints, emits `dsl/generated/tool_registry.dag`. Makegen/bootstrap import this artifact directly. No runtime discovery. No struct→Value conversion.
- **Acceptance**: `registry_tools_to_value()` **deleted** from `shared.rs`. `DiscoverToolsOp` **deleted** from `extern_ops.rs`. `dsl/generated/tool_registry.dag` exists and is consumed by `tools/makegen.dag` imports. `grep -r 'registry_tools_to_value\|DiscoverToolsOp' .` returns 0.
- **Cannot go a→f yet**: Requires compiler artifact emission. 2-3 items of work.
- **Interim**: Keep as-is. Don't optimize, don't cache, don't refactor. These will be **deleted**, not improved.

---

## Application Layer Cleanup

Once compiler fixes land, the application layer (currently gunbc-app) becomes simple:

### Thin shims are compiler debt

Thin shims in `gunbc-dag` are not harmless convenience code. They are compiler debt:
- they create a visible hack point for new contributors
- they let missing compiler features leak back into app Rust
- they turn naming-only wrappers into public APIs that people start depending on

Rule: if a file only renames, re-exports, or lightly wraps compiler/runtime entrypoints without
enforcing a real invariant, delete it. Do not preserve shim layers just because they are small.

Update 2026-03-05:
- deleted `gunbc-dag/src/tool_graphs.rs`
- deleted `gunbc-dag/src/pragma/mod.rs`
- deleted `gunbc-dag/src/docgen/mod.rs`
- deleted `gunbc-dag/src/dsl_builder.rs`
- deleted `gunbc-dag/src/fs_env.rs`
- deleted `gunbc-dag/src/dry_run.rs`
- deleted `gunbc-dag/src/dsl_registry.rs`
- deleted `gunbc-dag/src/resolve.rs`
- generated callers now use direct `gunbc_resolve::builder::build_dsl_graph(...)` calls with `gunbc_dag::extern_ops::GunbcExternResolver`

Remaining shim inventory:
- none in `gunbc-dag/src`

Final state:
- generated CLIs/tests/macros call real compiler/runtime entrypoints directly
- no public app-layer wrapper exists just to rename `build_dsl_graph*`, `infer_signature`, or re-export core APIs
- any surviving facade must state the invariant it protects and be narrowed to the smallest visibility that works

Acceptance:
- `gunbc-dag/src/dsl_builder.rs` is deleted
- `gunbc-dag/src/fs_env.rs`, `gunbc-dag/src/dry_run.rs`, `gunbc-dag/src/dsl_registry.rs`, and `gunbc-dag/src/resolve.rs` are deleted
- `rg -n 'pub fn build_.*graph|pub fn .*signature' gunbc-dag/src` returns only justified compiler/runtime entrypoints, not convenience wrappers
- generated code no longer imports repo-local wrapper symbols like `build_build_graph`, `build_pragma_graph`, or `build_docgen_graph`

### Rename

`gunbc-dag` → `gunbc-app` (**done**). It's the application entry point, not a DAG definition crate.

- **Acceptance**: `Cargo.toml` `[package] name = "gunbc-app"`. Directory renamed. All `Cargo.toml` dependency references updated. `grep -r 'gunbc-dag' Cargo.toml */Cargo.toml` returns 0.

### Output directory: single source of truth

Currently output paths are specified in three places:
1. `core/ir/src/workspace_layout.rs` — hardcoded `"target/codegen/bin"` constants
2. `codegen_cli.rs` — calls `WorkspaceLayout` with string fallback
3. `gunbc-app/Cargo.toml` — `[[bin]]` entries: `path = "../target/codegen/bin/*/main.rs"`

Plus `dsl/config/codegen_paths.dag` which declares intent but isn't consumed.

- **Final state**: `dsl/config/codegen_paths.dag` is the source of truth. `WorkspaceLayout` derives from it. `Cargo.toml` `[[bin]]` entries are generated by codegen from the same source.
- **Acceptance**: `grep -r '"target/codegen/bin"' core/ir/` returns 0 (hardcoded constants deleted). `WorkspaceLayout::new()` reads from compiled DSL config. One declaration, three consumers.

### Testgen engine

Testgen is split between `core/codegen/src/testgen/` (14,400 LOC library) and `gunbc-app/src/testgen_dag/` (1,500 LOC orchestration) due to a real circular dependency with testgen-registry.

**Question**: Is the meta-circular testgen (testgen is itself a DAG that generates tests) the right design? Or should `daglang compile --emit=tests` be a compiler mode? The latter would eliminate the testgen DAG, the circular dependency, and the 1,500 LOC orchestration layer entirely.

- **Acceptance (if compiler mode)**: `gunbc-app/src/testgen_dag/` **deleted** (~1,500 LOC). `daglang compile --emit=tests` produces test files. Testgen-registry circular dependency eliminated.

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

## 2026-03-05 Rebaseline: Large Sweeps

The immediate file-by-file cleanup was useful, but the next phase should be
executed as a few larger sweeps with explicit end states.

### What we're trying to accomplish

1. **No app policy in compiler crates**: `core/*` should own language/compiler/runtime infrastructure, not gunbc repository conventions.
2. **No runtime interpretation of lowered DSL**: the lowerer emits executable IR/SubDags directly; runtime stops delegating back into evaluator code.
3. **No profile/config leakage into user-facing tools**: workflows declare semantic requirements, not concrete profile names or local fallback trees.
4. **No runtime Rust adapters just to feed DSL**: registry discovery, struct-to-`Value` shaping, and artifact rendering become compile-time artifacts or real DSL modules.
5. **One source of truth per concern**: output paths, build targets, auth models, and generated bin layout are each declared once and derived everywhere else.

### Sweep plan

| Sweep | Goal | Deletes / Moves | Acceptance |
|-------|------|------------------|------------|
| **Sweep A: Auth/Profile Deletion** | Remove profiles as a user/runtime concept; replace with structural auth modeling | `available_profiles`, `--profile` CLI plumbing, `dsl/profiles/*`, workflow-local credential helpers | `rg -n 'available_profiles|--profile|dsl/profiles/' core gunbc-dag dsl -g'*.rs' -g'*.dag'` returns only justified compatibility residue |
| **Sweep B: Runtime Bridge Deletion** | Lowerer emits final runtime shape; resolver stops interpreting DSL fragments | Bridges 1, 2, 3, 8, 9 | `rg -n 'DeclaredOutputCallableOp|FnBodyCallableOp|CollectionDelegate|add_fs_env_root_node|GenericFilePrepareOp|GenericFileParseOp' core/` returns 0 |
| **Sweep C: Makegen/App Policy Extraction** | Move gunbc repository build policy out of `core/codegen` | Remaining `core/codegen/src/makegen/{model,registry,gitignore,shared}.rs` app policy | no `compile_data_from_module(...build_targets.dag|gitignore.dag)` or runtime makegen discovery in `core/codegen` |
| **Sweep D: Extern Collapse** | Delete app externs that only exist because artifacts/features are missing | `DiscoverToolsOp`, bootstrap render externs, `render_tree`, `build_snapshot_content`, CI/infra discovery helpers | `extern_ops.rs` contains only irreducible app bindings, each justified by a missing compiler feature |
| **Sweep E: Output/Layout Unification** | Single source of truth for codegen output paths and app identity | hardcoded `target/codegen/*` constants, duplicated bin layout, temporary crate naming | `dsl/config/codegen_paths.dag` drives all output/layout consumers |

### 2026-03-05 Progress Update

1. **Sweep A (partial)**: repo-facing `--profile`/`available_profiles` plumbing is removed from tool discovery, generated CLIs, makegen projections, and active runtime diagnostics. Provider auth modules now live in `dsl/extdeps/github/auth.dag` and `dsl/extdeps/llm/auth.dag`. The deleted repo binding modules are `dsl/profiles/gist.dag`, `dsl/profiles/sdlc.dag`, and `dsl/shared/credentials.dag`. Remaining residue is compiler-internal: lowerer profile types, compatibility fixtures under `dsl/profiles/`, and historical design/docs.
2. **Sweep C (partial)**: repo makegen policy moved from `core/codegen/src/makegen/*` into `gunbc-dag/src/makegen/*`, and the handwritten Rust Justfile renderer plus profile-specific tool projection were deleted. Remaining debt is runtime artifact discovery/rendering (`DiscoverToolsOp`, `render_makefile_from_dsl_discovery`, and Rust-side `compile_data_from_module(...)` loaders in app code).
3. **Sweep E (partial)**: `core/ir` no longer exports hardcoded `CODEGEN_OUT_DIR`, `CODEGEN_BIN_DIR`, `CODEGEN_LIB_DIR`, or `CODEGEN_STAMP_PATH`. Remaining duplication is localized to `WorkspaceLayout`, generated-bin `Cargo.toml` entries, and fallback path strings.

### Recommended execution order

1. **Sweep B** first. It deletes the biggest category of “runtime compiler debt” and removes the bug class behind the gist postmortem.
2. **Sweep A** next. Once the runtime no longer interprets lowered DSL, auth modeling can move to structural provider models without preserving `--profile`.
3. **Sweep C** after A/B. Makegen is still one of the most obvious places where gunbc repo policy leaks into `core/codegen`.
4. **Sweep D** after C. Artifact emission and DSL features can then delete most of `extern_ops.rs` instead of refining it.
5. **Sweep E** last. Output-path unification and crate renaming are cleanup, but they will stick better after the app/compiler boundary is stable.

### Execution rule

Do not schedule the next phase as isolated files. Each change should be
attached to one of the sweeps above and carried through until the sweep's
acceptance grep/test criteria are satisfied.

---

## Execution Order

| Priority | Items | Effort | Deleted | Proves |
|----------|-------|--------|---------|--------|
| **Done** | Delete bridges 4, 5, 10 | — | `OutputPathMetadataOp` (7 LOC), `PipelineDispatchOp` (85 LOC), `strip_pipeline_nodes()` (22 LOC), 6 redundant builder functions (~185 LOC) | Dead code removal |
| **Done** | Bridge 2b (default params) | — | N/A (new feature) | `make gist` Bug 2 fixed. Default values flow through call chains. |
| **Now** | Bridge 1 + Bridge 2 | 3-5 days | `DeclaredOutputCallableOp` (9 LOC), `FnBodyCallableOp` (86 LOC), `LoweredFnBody` type, `fn_body` field on `LoweredOp::Callable` | `make gist` Bug 1 fixed. Match arms properly guard transport nodes. Also P0-5 (`make install`). |
| **Now** | Bridge 11 (hermeticity) | 2-3 days | N/A (new feature) | Effect classification structural. Test scope classification reliable. Parallel with 1+2. |
| **Next** | Bridge 3 + Bridge 8 | 3-5 days | `CollectionDelegate` (17 LOC), `evaluate_collection()` (50 LOC), `LoweredOp::Collection` variant, `add_fs_env_root_node()` (14 LOC), `DslFsEnvOp` | Evaluator delegation eliminated. |
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
