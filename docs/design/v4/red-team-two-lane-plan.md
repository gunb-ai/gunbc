# Red Team Two-Lane Plan

**Status**: Proposed
**Date**: 2026-02-28
**Goal**: Two parallel worker lanes, ~10k LOC each, primarily deletion

## Design Principles

1. **Lanes touch different files** — no merge conflicts
2. **Each lane is internally sequential** — clear dependency chain
3. **Deletion is the primary output** — every task should net-delete
4. **Consolidation before migration** — simplify before moving

## Codebase Inventory (gunbc-dag)

```
gunbc-dag/src/          22,767 lines
  bin/                   3,991   (7 hand-written binaries)
  workflow/              4,513   (workflow planner/executor/catalog/commands)
  makegen/               3,517   (Makefile generation + registry)
  testgen_dag/           2,177   (test generation engine)
  resolve.rs             2,355   (DAG resolver)
  resolve_service.rs     2,190   (service transport interpreter)
  extern_impls.rs          637   (extern bridge lookup)
  mock_defaults.rs         587   (auto-mock builder)
  dsl_registry.rs          579   (DSL tool discovery)
  policy/pragma.rs         546   (clippy policy rendering)
  dsl_builder.rs           401   (DSL graph builder)
  resources.rs             342   (resource definitions)
  (other small files)      ~932
gunbc-dag/tests/         5,887 lines
```

## What Makes Tasks Redundant

**Key insight**: many tasks exist because the codebase has three layers for the same
concept: (1) DSL declaration, (2) Rust registry that duplicates the DSL, (3) Rust
consumer that reads the registry. Deleting the Rust registry layer eliminates both
the registry maintenance AND the consumer complexity.

### Consolidation opportunities

| Consolidation | Tasks Made Redundant | LOC Eliminated |
|---------------|---------------------|----------------|
| **Delete all hand-written binaries** (RT58-66 as one push) | RT56, RT57 (testgen profile policy becomes moot when sdlc.rs is deleted) | ~2,600 |
| **Delete workflow subsystem** (RT71+78+79+81 merged) | RT29 (registry dispatch — catalog is the biggest string dispatch), RT87 (inventory linkage — workflow specs are main consumer) | ~4,500 |
| **Delete makegen Rust registry** (RT75+80 merged) | RT18 (bootstrap extern deletion — gitignore categories move to DSL), RT86 (cross-layer contract tests — layers collapse) | ~3,500 |
| **Delete extern_impls.rs + resolve.rs dispatch** (RT23+72 merged) | RT29 (registry dispatch — the other big string dispatch target), RT44 (evaluate_fn_body coverage — externs handled differently) | ~3,000 |
| **Delete dead AST scaffolding** (RT84 expanded) | RT13 (mock registry derivation — delete MockResponseDef instead of wiring it), RT-I1/I2 (superseded — delete the scaffolding entirely) | ~500 |

## Lane A: "Substrate Deletion" — Delete gunbc-dag Rust infrastructure

**Theme**: Everything that duplicates DSL metadata gets deleted. Rust code that is
purely a registry, mapping table, or manual catalog goes away. This is Streams B+C
from the migration queue plus binary elimination.

**Files touched**: `gunbc-dag/src/` (bins, workflow, makegen, policy, extern_impls,
mock_defaults, resources, embedded_assets, thin wrappers)

**Not touched**: `resolve.rs`, `resolve_service.rs`, `testgen_dag/`, `dsl_builder.rs`,
`dsl_registry.rs`, compiler crates (`core/daglang/`)

### Lane A Tasks

| # | What | Deletes | Net LOC |
|---|------|---------|---------|
| A1 | **Delete 5 hand-written binaries.** Replace `sdlc.rs`, `deps_config.rs`, `pipeline.rs`, `workflow.rs`, `infra.rs` with generated stubs (or extend CLI generator to cover their gaps inline). Move the ~3 genuinely needed features (profile flag, mode flag, subcommand dispatch) into the CLI generator as small extensions. Delete `BinaryArgs` from `gunbc-cli`. | `bin/sdlc.rs` (239), `bin/deps_config.rs` (209), `bin/pipeline.rs` (341), `bin/workflow.rs` (715), `bin/infra.rs` (1055) + `BinaryArgs` infra (~300) | **-2,860** |
| A2 | **Delete workflow catalog + unit_commands + spec_builders.** Move `WORKFLOW_VARIANTS` to `dsl/config/workflow_catalog.dag` as `data` declarations. Move unit commands to `dsl/config/workflow_commands.dag`. Delete spec_builders (thin wrappers). Process registry claims → derived from DSL `uses` annotations. | `workflow/catalog.rs` (576), `workflow/unit_commands.rs` (425), `workflow/spec_builders.rs` (93), `workflow/process_registry.rs` (316) | **-1,410** |
| A3 | **Delete workflow executor infrastructure.** Extract 7 generic modules (planner, executor, admission, coordination, slo, projection, proof) to `core/workflow` crate. gunbc-dag gets a thin 50-line adapter. | `workflow/planner.rs` (572), `workflow/executor.rs` (415), `workflow/admission.rs` (521), `workflow/coordination.rs` (185), `workflow/slo.rs` (303), `workflow/projection.rs` (111), `workflow/proof.rs` (91) → `core/workflow/` | **-2,200** (moved, ~200 remains) |
| A4 | **Delete makegen Rust registry.** Move `BuildConfig`, `MetaTarget`, manual `ToolInfo` entries, gitignore categories, resource targets to DSL data declarations. Keep only `ToolInfo::from_tool_def()` (DSL-derived) and Cargo command construction. | `makegen/registry.rs` (~1,800 of 2,217), `makegen/gitignore.rs` (372), `resources.rs` (342) | **-2,514** |
| A5 | **Delete extern_impls.rs + policy/pragma.rs + tool wrappers.** Move remaining extern bridge functions to DSL (shadow fn bodies already exist for pragma). Delete tool module thin wrappers. Delete embedded_assets. | `extern_impls.rs` (637), `policy/pragma.rs` (546), `bootstrap/mod.rs` + `build/mod.rs` + `codegen/mod.rs` + `deps_tool.rs` + `infra/mod.rs` + `gist.rs` + `embedded_assets.rs` (~145), `docgen/mod.rs` (94) | **-1,422** |
| A6 | **Delete compensating tests.** Tests that exist only because the Rust registries existed: workflow contract tests, workflow capability tests, tool registration tests that enforce enum↔DSL sync. | `tests/workflow_gist_contracts.rs`, `tests/workflow_tool_capability_contracts.rs`, `tests/workflow_global_dedup_contracts.rs`, `tests/workflow_executor_contracts.rs`, `tests/workflow_key_ledger_contracts.rs`, `tests/workflow_plan_cli_contracts.rs`, `tests/workflow_schema_contracts.rs` (~2,500) | **-2,500** |

**Lane A Total: ~12,900 LOC deleted**

### Lane A Dependency Chain

```
A1 (binaries) ───→ A2 (workflow data) ───→ A3 (workflow infra) ───→ A6 (tests)
                   A4 (makegen registry) ─┘
                   A5 (externs + policy) ─┘
```

A1 can start immediately. A2/A4/A5 can run in parallel after A1. A3 requires A2.
A6 requires A2+A3.

---

## Lane B: "Compiler Hardening" — Lowerer pure-function extraction + fail-closed enforcement

**Theme**: Make the compiler correct by construction. Extract lowerer phases into
pure functions, eliminate silent drops, add fail-closed enforcement at every boundary.
The result is a compiler that either produces a correct graph or a typed error — never
a silently incomplete graph.

**Files touched**: `core/daglang/` (lowerer, parser, typechecker, emitter),
`gunbc-dag/src/resolve.rs`, `gunbc-dag/src/resolve_service.rs`, `gunbc-dag/src/mock_defaults.rs`,
`core/exec/`

**Not touched**: `gunbc-dag/src/bin/`, `gunbc-dag/src/workflow/`, `gunbc-dag/src/makegen/`,
`gunbc-dag/src/policy/`

### Lane B Tasks

| # | What | Deletes | Net LOC |
|---|------|---------|---------|
| B1 | **Lowerer: extract LoweringContext + dead code deletion.** Create context struct grouping the 8-11 parameter tuples. Audit and delete all dead `_ => None` arms in wiring paths (RT82). Delete stale `#[allow(clippy::too_many_arguments)]` after context extraction. Delete `looks_effectful_without_kind()` (dead after RT6). | ~22 functions lose 3-5 params each. Dead match arms deleted. | **-800** |
| B2 | **Lowerer: integrate scope.rs, delete ad-hoc branch detection.** Replace `detect_if_branches_in_stmts`, `detect_match_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` with `ScopedBody` from scope.rs. Delete duplicate walk functions. | `lib.rs` ad-hoc detection (~300 lines) replaced by `scope.rs` (already written) | **-300** |
| B3 | **Lowerer: extract transport derivation.** Move `add_service_transport_triplets` + `clone_transport_triplet` + `ServiceTransportEndpoint` to `transport.rs` module. Return `TransportManifest` instead of mutating builder. | `lib.rs` transport section (~600 lines) → `transport.rs` (~500 lines) | **-100** (structure improvement) |
| B4 | **Parser/AST: delete dead scaffolding (RT84).** Delete `MockResponseDef` (parser stub — never populated), `error_cases()` trait (always empty), `@retry` annotation (unimplemented), `hermetic` field on `OperationDef` (orphaned). Delete `Mockable` trait's `error_cases()`. | AST fields, trait methods, parser dead branches | **-300** |
| B5 | **Resolver: fail-closed audit (RT88+RT89).** Add status-code checking to `GenericRestParseOp` (non-2xx → error before field extraction). Audit all `_ =>` fallback arms in resolve.rs and resolve_service.rs. Replace accidental fallbacks with typed errors. Delete `passthrough_fallback_value()` port alias table (heuristic reimplementation). Delete `default_rest_response()` kitchen-sink blob — replace with per-service response specs from DSL. | `resolve.rs` fallback table (~70 lines), `mock_defaults.rs` blob (~200 lines), various `_ =>` arms | **-500** |
| B6 | **Resolver: extract generic framework to core/ (RT67+RT72).** Move `resolve_service.rs` (2,190 lines — 100% generic) to `core/ir/src/transport/service_ops.rs`. Extract generic resolution framework from `resolve.rs` (~1,600 lines) to `core/resolve/`. Leave domain dispatch (~700 lines) in gunbc-dag. | `resolve_service.rs` (moved), `resolve.rs` (split) | **-3,790** (moved to core/) |
| B7 | **Testgen: move engine to core/ (RT73).** Move `dag_test_discovery.rs`, `graph.rs`, `mock_interpreter.rs`, `ops.rs`, `profile_discovery.rs` to `core/codegen/src/testgen/`. gunbc-dag keeps thin caller. | `testgen_dag/` (2,177 lines) → `core/codegen/src/testgen/` | **-2,177** (moved to core/) |
| B8 | **Mock defaults: split and simplify (RT68).** Move generic probing/builder framework (~350 lines) to `core/test/src/auto_mock.rs`. Delete GCP-specific field mapping blob — replace with DSL `data` declarations for mock response shapes. | `mock_defaults.rs` (~587 lines) → core (~350) + delete (~230) | **-580** |
| B9 | **Lowerer: RT4a/4b/4c completion + RT38/RT39/RT43.** Complete return expression compute wiring (param refs — already started). Add nested field access. Replace panics with LowerError. Add structured LowerWarnings to return type. | Various lowerer improvements | **-200** (net, after adding warnings infrastructure) |
| B10 | **Executor: delete dead heuristics.** Delete `looks_effectful_without_kind()` (dead after RT6). Delete credential expiry dead code paths that were never wired (RT91 analysis showed the plumbing exists but nothing calls it — delete the unused plumbing). | `core/exec/src/execute.rs` dead code, `core/ir/src/transport/credential.rs` unused paths | **-400** |

**Lane B Total: ~9,150 LOC deleted/moved**

### Lane B Dependency Chain

```
B1 (context extraction) ───→ B2 (scope.rs integration) ───→ B3 (transport extraction) ───→ B9 (RT4a completion)
B4 (dead scaffolding) ────────────────────────────────────────────────────────────────────┘
B5 (fail-closed audit) ──→ B6 (resolve extraction to core/)
                           B7 (testgen extraction to core/)
                           B8 (mock defaults split)
B10 (executor dead code) ─┘
```

B1 and B4 can start immediately in parallel. B5 can start immediately.
B6/B7/B8 can start after B5 (or in parallel with each other).

---

## No-Conflict Guarantee

| Resource | Lane A | Lane B |
|----------|--------|--------|
| `gunbc-dag/src/bin/` | **Owns** | No touch |
| `gunbc-dag/src/workflow/` | **Owns** | No touch |
| `gunbc-dag/src/makegen/` | **Owns** | No touch |
| `gunbc-dag/src/policy/` | **Owns** | No touch |
| `gunbc-dag/src/extern_impls.rs` | **Owns** | No touch |
| `gunbc-dag/src/resources.rs` | **Owns** | No touch |
| `gunbc-dag/src/resolve.rs` | No touch | **Owns** |
| `gunbc-dag/src/resolve_service.rs` | No touch | **Owns** |
| `gunbc-dag/src/mock_defaults.rs` | No touch | **Owns** |
| `gunbc-dag/src/testgen_dag/` | No touch | **Owns** |
| `core/daglang/` | No touch | **Owns** |
| `core/exec/` | No touch | **Owns** |
| `gunbc-dag/src/dsl_builder.rs` | Shared (read) | Shared (read) |
| `gunbc-dag/src/dsl_registry.rs` | Shared (read) | Shared (read) |
| `gunbc-dag/src/lib.rs` | Module decls only | Module decls only |
| `tasks.md` | Status updates only | Status updates only |

## Tasks Made Redundant By This Design

| Original Task | Why Redundant |
|---------------|---------------|
| RT29 (Registry pattern for string dispatch) | Lane A deletes catalog.rs + unit_commands.rs. Lane B extracts resolve.rs dispatch to core/. No string dispatch tables remain. |
| RT56/RT57 (Deterministic profile selection) | Lane A deletes sdlc.rs (the only consumer). CLI generator handles profile selection. |
| RT86 (Cross-layer contract tests) | Lane A collapses the layers. No registry↔CLI↔makegen drift possible when DSL is the single source. |
| RT87 (Inventory linkage verification) | Lane A deletes workflow specs (main inventory consumer). Lane B moves remaining to core/ with explicit deps. |
| RT44 (evaluate_fn_body coverage) | Lane B's B9 addresses the specific wiring gaps. The broader evaluate_fn_body limitation is a DSL evaluator maturity issue, not a cleanup task. |
| RT13 (Derive mock registries from @mock_response) | Lane B's B4 deletes MockResponseDef scaffolding entirely. Mock registries derive from `response {}` blocks (PC-1) which is a separate feature track. |
| RT18 (Delete bootstrap externs) | Lane A's A5 deletes all extern_impls.rs including bootstrap bridges. |

## Summary

| Lane | Theme | LOC Deleted | Files Deleted | Files Moved |
|------|-------|-------------|---------------|-------------|
| **A** | Substrate Deletion | ~12,900 | ~25 .rs files | 0 |
| **B** | Compiler Hardening | ~9,150 | ~5 .rs files | ~10 .rs files to core/ |
| **Total** | | **~22,050** | ~30 | ~10 |
