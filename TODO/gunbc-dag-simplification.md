# Lane 2: gunbc-dag Simplification

**Goal**: Reduce gunbc-dag to the minimum necessary Rust — thin wrappers, binary entrypoints, and extern operations blocked on compiler features. Everything expressible in .dag should be in .dag.

**Question for design review**: Is the current architecture overcomplicated? The crate has 5,265 LOC across 27 files. ~2,200 LOC is infrastructure that can't migrate. ~1,000 LOC is already-migrated thin wrappers. The real question is whether the remaining ~2,000 LOC (testgen engine, extern ops, resource defs) can be simplified or restructured.

---

## Current State

### What's already migrated (DSL core + thin Rust wrapper)

| Module | LOC | .dag equivalent |
|--------|-----|-----------------|
| `pragma/mod.rs` | 21 | `tools/pragma.dag` |
| `ci/mod.rs` | 93 | `pipelines/ci.dag` |
| `docgen/mod.rs` | 18 | `tools/docgen.dag` |
| `tool_graphs.rs` | 207 | `tools/{bootstrap,build,codegen,deps,infra,makegen}.dag` |
| `workflow/catalog.rs` | 442 | `config/workflow_catalog.dag` + `config/workflow_commands.dag` |
| `workflow/commands.rs` | 145 | `config/workflow_commands.dag` |
| `workflow/spec_builders.rs` | 95 | `config/workflow_catalog.dag` |
| **Total** | **1,021** | |

### Infrastructure (cannot migrate — binary/runtime plumbing)

| Module | LOC | Why it stays |
|--------|-----|--------------|
| `bin/ci.rs` | 267 | Bootstrap CI — runs before codegen generates other binaries |
| `bin/codegen_cli.rs` | 959 | Code generator — creates all other binaries, Cargo.toml manipulation |
| `resolve.rs` | 26 | App-specific `GunbcExternResolver` |
| `dsl_builder.rs` | 65 | Thin wrappers passing `GunbcExternResolver` |
| `tool_runner.rs` | 123 | Shared binary entry point helpers |
| `lib.rs` | 168 | Module aggregator |
| `testgen_dag/ops.rs` | 233 | `TestgenOp` executable — calls `build_dsl_graph_with_types()` at runtime |
| `workflow/capabilities.rs` | 231 | Codegen/compilation unit definitions |
| Re-export wrappers | 71 | `dsl_registry.rs`, `fs_env.rs`, `dry_run.rs`, `testgen_dag/mod.rs`, `workflow/mod.rs`, `fidelity.rs` |
| **Total** | **2,143** | |

### Partially migrated (DSL + Rust bridge)

| Module | LOC | What's in DSL | What's still in Rust |
|--------|-----|---------------|----------------------|
| `pragma/dsl_render.rs` | 112 | `config/clippy_policy.dag` (3 fn items) | `evaluate_fn_body()` bridge; clippy_toml blocked on FC-CF5 |
| `testgen_dag/dag_test_discovery.rs` | 929 | `config/test_policy.dag` (classification) | Test discovery, compilation, mock interpretation |
| `testgen_dag/graph.rs` | 313 | — | Dynamic DAG building (N upsert chains per target) |
| `resource_defs.rs` | 202 | `config/resources.dag` (globs, paths) | Resource trait definitions |
| **Total** | **1,556** | | |

### Extern operations (`extern_ops.rs` — 521 LOC)

11 extern operations, dispatch table at line 13-37. Status:

| # | Operation | LOC | Status | Blocker |
|---|-----------|-----|--------|---------|
| 1 | `render_tree` | 49 | Blocked | FC-CF6 (recursive fns) |
| 2 | `build_snapshot_content` | 46 | Blocked | Runtime string formatting |
| 3 | `discover_tools` | 29 | **Actionable** | Needs refactor (Plan A) |
| 4 | `discover_ci_config` | 37 | **Actionable** | Cross-module extern func calls |
| 5 | `render_bootstrap_makefile` | 17 | Blocked | DSL template layer not designed |
| 6 | `render_bootstrap_gitignore` | 17 | Blocked | DSL template layer not designed |
| 7 | `render_clippy_toml_content` | 11 | **Actionable** | FC-CF5 (partially done — VariantConstruct exists) |
| 8 | `render_disallowed_methods_allowlist_content` | — | **Done** | Evaluates DSL fn |
| 9 | `render_pragma_lint_policy_content` | — | **Done** | Evaluates DSL fn |
| 10 | `infra` | — | **Done** | Pure DSL (legacy dispatch point) |
| 11 | (deleted ops) | — | **Done** | — |

---

## Open Items

### Phase 1: Quick wins (no compiler changes needed)

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | DAG-1 | **Plan A: `discover_tools` refactor.** Consolidate registry filtering logic, eliminate ~95 LOC duplication with `registry_tools_to_value()`. Add parity test. | `discover_tools` uses consolidated path. | S | Open |
| 2 | DAG-2 | **Delete dead `infra` extern dispatch.** The `infra` entry in `extern_ops.rs` dispatches to pure DSL — the extern wrapper is unnecessary. | `infra` extern removed from dispatch table. | S | Open |
| 3 | DAG-3 | **Plan B: Branch body passthrough fix.** Add trivial `fn_body: return { result: input }` to branch/loop body ops. Add `validate_callable_output_wiring(dag)` in lowerer. | Branch bodies don't crash on missing `__out:result`. | S | Open |

### Phase 2: Unblock with small compiler fixes

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 4 | DAG-4 | **FC-CF5 completion: sum type variant tag access.** Unblocks `render_clippy_toml` migration (11 LOC extern → pure DSL). | `render_clippy_toml_content` extern deleted. | M | Open |
| 5 | DAG-5 | **Cross-module extern func call fix.** Lowerer currently breaks same-module calls. Fix unblocks `discover_ci_config` migration (37 LOC). | `discover_ci_config` extern deleted. | S | Open |

### Phase 3: Larger compiler features (deferred)

| # | ID | What | Blocker | LOC freed |
|---|-----|------|---------|-----------|
| 6 | DAG-6 | **Recursive fns (FC-CF6).** Unblocks `render_tree` (49 LOC). | Non-trivial type/lowering work | 49 |
| 7 | DAG-7 | **DSL template layer.** Unblocks `render_bootstrap_makefile` + `render_bootstrap_gitignore` (34 LOC). | Design needed | 34 |
| 8 | DAG-8 | **Runtime string formatting.** Unblocks `build_snapshot_content` (46 LOC). | Design needed | 46 |

### Phase 4: Structural simplification (needs design review)

| # | ID | What | Question |
|---|-----|------|----------|
| 9 | DAG-9 | **Testgen engine simplification.** `dag_test_discovery.rs` (929 LOC) + `graph.rs` (313 LOC) = 1,242 LOC of test discovery/compilation/mock interpretation. Is this overcomplicated? Can it be restructured? | Needs design review — the N-dynamic-DAG pattern may have a simpler DSL-native expression. |
| 10 | DAG-10 | **Resource defs simplification.** `resource_defs.rs` (202 LOC) loads from DSL but defines Rust traits. Can the Rust traits be eliminated? | Needs design review — depends on whether resource types need runtime Rust dispatch or can be pure DSL data. |

---

## Deleted Tests (re-add when root cause fixed)

| ID | Tests | Blocker |
|----|-------|---------|
| RF-E5 | `makegen_runtime_differential_interpreter_vs_generated_rust_layer1` | FnBodyDelegate gap |
| RF-E6 | `makegen_exec_runtime_e2e`, `pragma_exec_runtime_e2e`, `makegen_e2e_generated_binary`, `pragma_e2e_generated_binary` | Exec-runtime emitter |

## Future (post-simplification)

| ID | Task | Size | Notes |
|----|------|------|-------|
| C28-P2 | **Daggen cache manager.** Content-hash → `.dagbin` → skip recompilation. | M | Infrastructure ready. |
| C28-P3 | **Daggen codegen integration.** Serialize all tool DAGs at `make codegen` time. | L | Eliminates runtime DSL parsing. |

---

## Success Criteria

1. Zero extern operations that could be pure DSL (given current compiler features)
2. Testgen engine reviewed for simplification opportunities
3. `extern_ops.rs` dispatch table matches exactly the set of compiler-blocked operations
4. All soundness fixes (Plan A + Plan B) landed
5. Re-export wrappers reduced to minimum necessary
