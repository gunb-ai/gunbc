# Four-Worker Parallel Plan

**Status**: Active
**Date**: 2026-02-28
**Goal**: 4 workers with mutually exclusive file ownership, ~7-10k LOC each

## Current State

```
gunbc-dag/src/      22,767 lines  (target: ~5,000)
gunbc-dag/tests/     5,791 lines  (target: ~2,000)
core/daglang/       50,288 lines  (restructure lowerer, keep rest)
core/codegen/       17,152 lines  (extract stdlib host, move testgen in)
core/exec/          14,554 lines  (clean up dead code)
```

## File Ownership (Mutually Exclusive)

| File/Directory | Worker A | Worker B | Worker C | Worker D |
|----------------|----------|----------|----------|----------|
| `gunbc-dag/src/bin/sdlc.rs` | **OWNS** | | | |
| `gunbc-dag/src/bin/deps_config.rs` | **OWNS** | | | |
| `gunbc-dag/src/bin/pipeline.rs` | **OWNS** | | | |
| `gunbc-dag/src/bin/workflow.rs` | **OWNS** | | | |
| `gunbc-dag/src/bin/infra.rs` | **OWNS** | | | |
| `gunbc-dag/src/workflow/` (all 17 files) | **OWNS** | | | |
| `gunbc-dag/tests/workflow_*.rs` (7 files) | **OWNS** | | | |
| `gunbc-dag/tests/infra_cli.rs` | **OWNS** | | | |
| `gunbc-dag/src/makegen/` (all files) | | **OWNS** | | |
| `gunbc-dag/src/policy/` (all files) | | **OWNS** | | |
| `gunbc-dag/src/extern_impls.rs` | | **OWNS** | | |
| `gunbc-dag/src/resources.rs` | | **OWNS** | | |
| `gunbc-dag/src/embedded_assets.rs` | | **OWNS** | | |
| `gunbc-dag/src/docgen/` | | **OWNS** | | |
| `gunbc-dag/src/{bootstrap,build,codegen,infra,gist,deps_tool}.rs` | | **OWNS** | | |
| `gunbc-dag/tests/tool_registration.rs` | | **OWNS** | | |
| `gunbc-dag/tests/makefile_parity.rs` | | **OWNS** | | |
| `gunbc-dag/tests/extern_ratchet.rs` | | **OWNS** | | |
| `core/daglang/daglang-lower/` | | | **OWNS** | |
| `core/daglang/daglang-syntax/` | | | **OWNS** | |
| `core/daglang/daglang-emit/` | | | **OWNS** | |
| `core/daglang/daglang-typecheck/` | | | **OWNS** | |
| `core/daglang/daglang-driver/` | | | **OWNS** | |
| `core/codegen/src/fidelity.rs` | | | **OWNS** | |
| `gunbc-dag/src/resolve.rs` | | | | **OWNS** |
| `gunbc-dag/src/resolve_service.rs` | | | | **OWNS** |
| `gunbc-dag/src/mock_defaults.rs` | | | | **OWNS** |
| `gunbc-dag/src/testgen_dag/` | | | | **OWNS** |
| `core/exec/` | | | | **OWNS** |
| `core/codegen/src/testgen/` | | | | **OWNS** |

**Shared (read-only for all)**: `gunbc-dag/src/lib.rs` (module decls), `dsl_builder.rs`,
`dsl_registry.rs`, `dsl/` (DSL source files), `gunbc-dag/src/bin/ci.rs`,
`gunbc-dag/src/bin/codegen_cli.rs`, `tasks.md` (status updates only).

---

## Worker A: "Binary & Workflow Elimination"

**Theme**: Delete all hand-written binaries (except bootstrap exceptions) and the
Rust workflow subsystem. Replace with DSL data declarations and CLI generator extensions.

**Files owned**: `gunbc-dag/src/bin/{sdlc,deps_config,pipeline,workflow,infra}.rs`,
`gunbc-dag/src/workflow/` (17 files), `gunbc-dag/tests/workflow_*.rs` (7 files),
`gunbc-dag/tests/infra_cli.rs`

### Tasks

| # | Task ID | What | Delete | Add |
|---|---------|------|--------|-----|
| A1 | RT59 | **Profile-aware CLI generation.** Expose `available_profiles` in `CompileOutput`. CLI template generates `--profile` enum flag. | 0 | ~100 |
| A2 | RT58+RT60 | **Eliminate `sdlc.rs`.** Param source propagation moves to `detect_entrypoints()`. Delete handwritten binary. | -239 | ~20 |
| A3 | RT61 | **Eliminate `deps_config.rs`.** Add `--mode` flag support to CLI template. | -209 | ~10 |
| A4 | RT62 | **Eliminate `pipeline.rs`.** Move `query_ci_status()`, `query_pr_description()` to DSL func nodes. | -341 | ~30 |
| A5 | RT63+RT64 | **Eliminate `workflow.rs`.** Subcommand dispatch in CLI generator. Move `render_plan_text/json` to DSL. | -715 | ~50 |
| A6 | RT65 | **Eliminate `infra.rs`.** 8 subcommands → DSL. `KEY=VALUE` parsing + multi-value flags in template. | -1,055 | ~80 |
| A7 | RT78 | **Workflow catalog → DSL data.** Move `WORKFLOW_VARIANTS` + `default_claims_for_stage()` to `dsl/config/workflow_catalog.dag`. | -576 | ~80 |
| A8 | RT79 | **Unit commands → DSL data.** Move per-workflow command tables to `dsl/config/workflow_commands.dag`. | -425 | ~60 |
| A9 | RT71 | **Extract generic workflow to `core/workflow`.** Move planner, executor, admission, coordination, slo, projection, proof, errors, schema, key. gunbc-dag keeps 50-line adapter. | -3,100 | ~100 |
| A10 | — | **Delete compensating tests.** 7 workflow test files + infra_cli that exist for the deleted registries. | -1,900 | 0 |
| A11 | RT66 | **Delete handwritten binary infrastructure.** Remove `BinaryArgs` from `gunbc-cli`, clean up orphaned support code. | -300 | 0 |

**Worker A Total: -8,860 deleted, +530 added = -8,330 net**

### Dependency Chain
```
A1 (profile CLI gen) → A2 (sdlc.rs) → A3 (deps_config.rs) → A4 (pipeline.rs)
                       A5 (workflow.rs) ─→ A6 (infra.rs) ─→ A11 (cleanup)
A7 (catalog → DSL) → A8 (commands → DSL) → A9 (extract to core/) → A10 (delete tests)
```

### Endstate
- `gunbc-dag/src/bin/`: only `ci.rs` (209) + `codegen_cli.rs` (931) remain
- `gunbc-dag/src/workflow/`: deleted entirely (moved to `core/workflow/` as generic crate)
- All workflow data is in `dsl/config/*.dag`
- CLI generator handles profiles, modes, subcommands

---

## Worker B: "Registry & Extern Deletion"

**Theme**: Delete every Rust file that is a manual registry, data table, or extern
bridge. Replace with DSL data declarations. After this worker, adding a new tool
requires zero Rust changes.

**Files owned**: `gunbc-dag/src/makegen/` (6 files), `gunbc-dag/src/policy/` (2 files),
`gunbc-dag/src/extern_impls.rs`, `gunbc-dag/src/resources.rs`,
`gunbc-dag/src/embedded_assets.rs`, `gunbc-dag/src/docgen/`,
tool wrappers (`bootstrap/`, `build/`, `codegen/`, `infra/`, `gist.rs`, `deps_tool.rs`),
`gunbc-dag/tests/{tool_registration,makefile_parity,extern_ratchet}.rs`

### Tasks

| # | Task ID | What | Delete | Add |
|---|---------|------|--------|-----|
| B1 | RT75 | **Gitignore patterns → DSL data.** Move category definitions to `dsl/config/gitignore.dag`. | -372 | ~80 |
| B2 | RT80 | **Makegen registry → DSL data.** Move `BuildConfig`, `MetaTarget`, manual `ToolInfo` entries. Keep only `ToolInfo::from_tool_def()` + Cargo command construction. | -1,800 | ~200 |
| B3 | RT74 | **Resource definitions → DSL data.** Move `REPO_SOURCE_INPUT_GLOBS`, output paths to `dsl/config/resources.dag`. | -342 | ~60 |
| B4 | RT76 | **Docgen read targets → DSL data.** Move `DOCGEN_READ_TARGETS` to `dsl/tools/docgen.dag`. | -94 | ~20 |
| B5 | RT77 | **Delete `policy/pragma.rs`.** DSL rendering already works. Delete the Rust mirror. | -546 | 0 |
| B6 | RT23 | **Delete `extern_impls.rs`.** All extern bridges → DSL `extern func` or deleted. | -689 | ~20 |
| B7 | RT81 | **Delete tool module wrappers.** Replace 7 thin wrapper modules with generic lookup. | -165 | ~30 |
| B8 | — | **Delete `embedded_assets.rs`.** Dead after extern deletion. | -27 | 0 |
| B9 | — | **Delete compensating tests.** `tool_registration.rs` (registry sync), `makefile_parity.rs` (golden Makefile), `extern_ratchet.rs` (extern bridge count). | -1,091 | 0 |
| B10 | — | **Clean up `makegen/shared.rs` + `makegen/justfile.rs`.** Remove references to deleted registries. | -400 | ~50 |

**Worker B Total: -5,526 deleted, +460 added = -5,066 net**

### Dependency Chain
```
B1 (gitignore → DSL) → B2 (makegen registry → DSL) → B10 (shared cleanup)
B3 (resources → DSL) ─┘
B4 (docgen → DSL)
B5 (delete pragma.rs) → B6 (delete extern_impls) → B7 (delete wrappers) → B8 (embedded_assets) → B9 (tests)
```

### Endstate
- `gunbc-dag/src/makegen/`: only `registry.rs` (~400 lines, DSL-derived only) + `shared.rs` (~200)
- `gunbc-dag/src/policy/`: deleted entirely
- `gunbc-dag/src/extern_impls.rs`: deleted
- All tool/target/resource data lives in `dsl/config/*.dag`
- New tool = new `.dag` file, zero Rust

---

## Worker C: "Lowerer Pure-Function Refactor"

**Theme**: Transform the 8,670-line imperative lowerer into composable pure functions.
Kill the compile-then-eval anti-pattern. Make pipe methods and enums first-class.
Every phase independently testable.

**Files owned**: `core/daglang/` (all 5 compiler crates), `core/codegen/src/fidelity.rs`

### Tasks

| # | Task ID | What | Delete | Add |
|---|---------|------|--------|-----|
| C1 | RT93/B13 | **Stdlib host + caching.** Replace `compile_fidelity()` with `OnceLock` cache. Embed stdlib via `include_str!`. Single `StdLibHost::eval_fn()` interface. Delete per-module compile wrappers. | -100 | ~80 |
| C2 | RT42/B14 | **Pipe methods first-class in AST.** Delete `should_track_call_name()` string allowlist. `PipeMethod` enum at parse boundary. Exhaustive match in typechecker+lowerer. | -80 | ~60 |
| C3 | RT45+RT46/B15 | **Typed enums end-to-end.** Finish `Value::Enum { ty, variant }`. Delete `TestClass::parse()`/`FermiCost::parse()` round-trips. Replace `unwrap_or()` fallbacks with errors. | -60 | ~40 |
| C4 | B1 | **Extract `LoweringContext`.** Group the 8-11 parameter tuples into context struct. Thread `&LoweringContext` instead of N separate args. Delete 18 `#[allow(clippy::too_many_arguments)]`. | -200 | ~100 |
| C5 | B2 | **Integrate `scope.rs`, delete ad-hoc detection.** Replace `detect_if_branches_in_stmts`, `detect_match_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` with `ScopedBody`. | -300 | ~50 |
| C6 | B3 | **Extract transport derivation module.** `add_service_transport_triplets` → `transport.rs`. Return `TransportManifest` instead of mutating builder. | 0 | ~50 |
| C7 | RT82/B11 | **Expr walker totality.** Audit all `match expr {` for silent catch-alls. Recurse or diagnose — no `_ => {}`. | -100 | ~50 |
| C8 | B12 | **Typed leaf references.** Replace `PARAM_REF_SENTINEL` string encoding with `LeafRef` enum. Delete `split_once("__")` decoding. | -30 | ~50 |
| C9 | B4/RT84 | **Delete dead AST scaffolding.** `MockResponseDef` (stub), `error_cases()` (empty), `@retry` (unimplemented), `hermetic` on `OperationDef` (orphaned). | -200 | 0 |
| C10 | RT38+RT39 | **No panics, no silent parse.** Replace `panic!` in `derive_file_spec()` with `LowerError`. Replace silent `auth_input` parser advance with parse error. | -10 | ~30 |
| C11 | RT94/B17 | **Resolve ReturnExprCompute split-brain.** Lowerer desugars complex returns into explicit DAG nodes so emit path never encounters them. Delete `MetadataOnly` classification for `ReturnExprCompute`. | -50 | ~100 |

**Worker C Total: -1,130 deleted, +610 added = -520 net (but massive structural improvement)**

### Dependency Chain
```
C1 (stdlib host) → C3 (typed enums)
C2 (pipe method enum)
C4 (context) → C5 (scope.rs) → C6 (transport module)
C7 (expr totality) → C8 (typed leaf refs)
C9 (dead scaffolding)
C10 (panics+parse)
C11 (split-brain) — can start after C7
```

### Endstate
- Lowerer: 6+ modules instead of 1 monolith, all pure functions
- `LoweringContext` replaces 8-11 param threading
- Zero `_ => None` in wiring paths, zero panics on user input
- Pipe methods: enum, not string allowlist
- Enums: typed values, not string round-trips
- Stdlib: cached once, embedded, no disk I/O
- No ReturnExprCompute in emitted runtime path (desugared in lowerer)

---

## Worker D: "Resolver & Executor Hardening"

**Theme**: Extract generic infrastructure from gunbc-dag to core/ crates. Make
the resolver fail-closed. Clean up the executor. After this, gunbc-dag is just
a thin domain-specific adapter over core/ libraries.

**Files owned**: `gunbc-dag/src/resolve.rs`, `gunbc-dag/src/resolve_service.rs`,
`gunbc-dag/src/mock_defaults.rs`, `gunbc-dag/src/testgen_dag/` (5 files),
`core/exec/` (all files), `core/codegen/src/testgen/`

### Tasks

| # | Task ID | What | Delete | Add |
|---|---------|------|--------|-----|
| D1 | RT89/B5 | **REST status-code checking.** `GenericRestParseOp` checks status before field extraction. Non-2xx → error. Document intentional non-2xx ops (404 → `found: false`). | -20 | ~80 |
| D2 | RT88/B5 | **Fail-closed audit.** All `_ =>` fallback arms in resolve.rs + resolve_service.rs: classify as intentional vs accidental. Replace accidental with typed errors. | -100 | ~50 |
| D3 | — | **Delete `passthrough_fallback_value()`.** Port alias heuristic table (70 lines). Replaced by explicit wiring. | -70 | 0 |
| D4 | — | **Delete `default_rest_response()` blob.** Kitchen-sink mock response builder (200 lines). Replace with per-service DSL-declared response shapes. | -200 | ~30 |
| D5 | RT67/B6 | **Move `resolve_service.rs` to `core/`.** 2,190 lines, 100% generic (parameterized by `ServiceOperationSpec`). Target: `core/ir/src/transport/service_ops.rs` or `core/resolve/`. | -2,190 | ~20 (adapter) |
| D6 | RT72/B6 | **Split `resolve.rs`.** Generic resolution framework (~1,600 lines) → `core/resolve/`. Domain dispatch (~700 lines) stays in gunbc-dag. | -1,600 | ~50 (adapter) |
| D7 | RT73/B7 | **Move testgen to `core/codegen/src/testgen/`.** 5 files (2,177 lines). gunbc-dag keeps thin caller. | -2,177 | ~30 (caller) |
| D8 | RT68/B8 | **Split mock_defaults.** Generic probing/builder (~350 lines) → `core/test/src/auto_mock.rs`. Delete GCP blob (~230 lines). | -580 | ~20 (adapter) |
| D9 | B10 | **Executor dead code.** Delete `looks_effectful_without_kind()` (dead after RT6). Delete unwired credential expiry plumbing. | -400 | 0 |
| D10 | RT95/B16 | **Transport class in node metadata.** Store `ServiceTransportClass` during lowering. Registry gen reads metadata, not node-id substrings. | -40 | ~30 |
| D11 | RT96/B18 | **Kill `propagate_to_param_sources`.** Fix boundary detection. Param source nodes auto-fed by generated CLI. | -100 | ~30 |
| D12 | RT83 | **Restore passthrough enforcement.** Once lowerer wires dag_util if/else branches (Worker C), restore hard `ExecError` for required outputs with no input. | -5 | ~15 |

**Worker D Total: -7,482 deleted, +355 added = -7,127 net**

### Dependency Chain
```
D1 (REST status) → D2 (fail-closed audit) → D3 (delete fallback table) → D4 (delete mock blob)
D5 (move resolve_service) → D6 (split resolve.rs)
D7 (move testgen)
D8 (split mock_defaults)
D9 (executor dead code)
D10 (transport metadata)
D11 (kill propagate)
D12 (restore RT83) — blocked on Worker C finishing C5+C7
```

### Endstate
- `gunbc-dag/src/resolve.rs`: ~700 lines (domain dispatch only)
- `gunbc-dag/src/resolve_service.rs`: deleted (moved to `core/`)
- `gunbc-dag/src/mock_defaults.rs`: deleted (split to `core/test`)
- `gunbc-dag/src/testgen_dag/`: deleted (moved to `core/codegen`)
- All generic resolution/testgen/mock infrastructure lives in `core/`
- Executor: clean, no dead code
- Resolver: every path is fail-closed, no heuristic fallbacks

---

## Combined Endstate

| Area | Before | After | Delta |
|------|--------|-------|-------|
| `gunbc-dag/src/` | 22,767 | ~6,700 | **-16,067** |
| `gunbc-dag/tests/` | 5,791 | ~2,800 | **-2,991** |
| `core/workflow/` (new) | 0 | ~3,000 | +3,000 |
| `core/resolve/` (new) | 0 | ~1,600 | +1,600 |
| `core/daglang/daglang-lower/` | 15,033 | ~13,500 | -1,533 |
| `core/exec/` | 14,554 | ~14,100 | -454 |
| **Net** | | | **-16,445** |

### What becomes impossible after all 4 workers

- **Adding a tool requires Rust changes**: impossible (DSL-only, Worker B)
- **Silent wiring drops**: impossible (typed errors, Worker C)
- **Compiler panics on user input**: impossible (LowerError, Worker C)
- **Runtime compile-then-eval**: impossible (cached stdlib, Worker C)
- **Interpreted/emitted runtime diverge**: impossible (desugared in lowerer, Worker C)
- **Transport class from node-id substring**: impossible (metadata, Worker D)
- **Silent success on 401/error**: impossible (status checking, Worker D)
- **Manual param_source propagation**: impossible (boundary model, Worker D)

### Cross-worker dependency (only 1)

Worker D task D12 (restore RT83 enforcement) is blocked on Worker C tasks C5+C7
(scope.rs integration + expr walker totality). This is the only cross-worker
dependency. D12 should be the last task in Worker D's queue.
