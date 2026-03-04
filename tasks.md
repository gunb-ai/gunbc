# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)
**Archive**: `TODO/TODONE/tasks-archive-2026-03-02.md` (68 completed items from Lanes 1-3)
**Verified**: 2026-03-03 — all Done items pass `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Design doc**: [`docs/design/v4/compositional-type-coverage.md`](docs/design/v4/compositional-type-coverage.md) — vision, audit, gaps, workstreams WS-1 through WS-7

---

## Status Summary

```
Lane 1: Compiler Critical Path — 0 open (26/26 complete)
Lane 1: Binary Elimination    — 0 open (10/10 complete)
Phase 3: Purist Engine        — 0 open (4/4 complete)
Compiler Extensibility        — 5 items (CX-1 through CX-5)
Backlog                       — 7 items (low priority)
```

### Critical Path (Complete)

```
C10 (return expr) ──→ C10a (auth wiring) ──→ production gist/CI
      DONE                  DONE                  DONE
C24 (pure dataflow) ──→ C25 (service codegen) ──→ C28 (daggen AOT)
      DONE                    DONE                     DONE (Phase 1)
```

---

## Lane 1: Compiler Pipeline Refactor (Complete)

26 of 26 items complete. See archive for C1-C9, C11-C19, C21, C23, C26, C27.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | C10 | **Resolve ReturnExprCompute split-brain (RT4a/c).** Desugar complex returns (BinOp/If/Match/Pipe/...) into explicit DAG semantics. `resolve_return_expr_source` handles BinOp/If/Match/UnaryOp/Record/StringInterp/List/FieldAccess. Pipe/For/NullCoalesce tagged as PipeOp/ForOp. | ExprCompute reduced to evaluator-backed expressions only. | L | **Done** — 3 new PrimitiveOpKind variants (ListConstruct, PipeOp, ForOp), 4 new synthesis functions. |
| 2 | C10a | **`make gist` auth credential bridge fix.** RT1 (auth_input wiring) + RT2 (fail-closed enforcement) verified. | `make gist` no longer 401s. | M | **Done** — gist.dag wires `acquire_gcp_secret` → `auth_token` → BearerToken. `gist_recent_graph_wires_credential_to_gist_execute` test passes. |
| 3 | C22 | **Deductive Redundancy Elimination Phase 2.** `ExecutionLedger` in `core/exec/src/ledger.rs`. `ExecutionRecord`, `RedundancyViolation` types. `assert_no_redundant_operations()` in test lib. | Execution ledger catches dynamic redundancy. | L | **Done** — ledger.rs with 6 unit tests. |
| 4 | C24 | **Pure dataflow lowering.** 12 PrimitiveOpKind variants now handle all structural expression forms. Pipe/For use tagged evaluator. ListConstruct, StringInterpolate, GetField have dedicated synthesis. | ExprCompute reduced to evaluator-only expressions. | XL | **Done** — structural lowering for all expression categories except Lambda. |
| 5 | C25 | **Service-driven codegen.** All service operations use generic data-driven interpreters (`GenericRestPrepareOp`, `GenericShellPrepareOp`, etc.) parameterized by `ServiceOperationSpec`. Zero per-service DynOp types. | Zero handwritten `DynOp` for services. | XL | **Done** — all services use generic protocol interpreters parameterized by spec data. `extern_ops.rs` contains only 6 domain-specific ops (render_tree, discover_tools, etc.), not service ops. |

---

## Lane 1: Binary Elimination (Complete)

10 of 10 items complete.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | A2 | **Eliminate `deps_config.rs`.** Replaced with `dsl/tools/deps_config.dag`. Binary deleted from Cargo.toml. | DSL tool def with feature parity. | S | **Done** — `deps_config.dag` with DepsMode sum type, `[[bin]]` entry removed. |
| 2 | A3 | **Eliminate `pipeline.rs` binary.** Original binary already deleted. `daglang-cli/src/pipeline.rs` is compiler infrastructure. | No binary to eliminate. | M | **Done** — already eliminated. |
| 3 | A4 | **Eliminate `workflow.rs`.** Replaced with `dsl/tools/workflow.dag`. Binary deleted from Cargo.toml. | DSL tool def with feature parity. | L | **Done** — `workflow.dag` with WorkflowFormat sum type, `[[bin]]` entry removed. |

---

## Phase 3: The Purist Engine (Complete)

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 1 | C28 | **Daggen (AOT DAG Compilation) Phase 1.** Serde derives on all 25 lowered IR types (`LoweredOp`, `LoweredFnBody`, `LoweredExpr`, `PrimitiveOpKind`, `ServiceOperationSpec`, `PatternOp`, etc.). `serialize_lowered_dag()` / `deserialize_lowered_dag()` API. Round-trip + resolve tests. | XL | **Done (Phase 1)** — serialization infrastructure. Phase 2 (codegen integration + cache manager) remains for future work. |
| 2 | C29 | **Dynamic JSON-Path Output Mappings.** `GenericRestParseOp::execute` consumes `output_shape` when available. `extract_shape_field()` uses `OutputFieldExtraction` + `from_bridge_json_typed`. | REST parse uses declarative field extraction. | M | **Done** — output_shape consumed in GenericRestParseOp, fallback to output_fields. |
| 3 | C30 | **Strict Type-Aware JSON Bridging.** `extract_output_field()` delegates to `from_bridge_json_typed()` for all non-Secret/Bytes types. String-based type dispatch eliminated. | Type-aware JSON bridging for REST responses. | M | **Done** — single `from_bridge_json_typed` call replaces string-based switch. |
| 4 | CT-8 | **Wire Contract Test Generation.** `build_interface_contract_tests` and `build_response_contract_tests` generate actual test bodies with `#[ignore]`. `TestFn.attributes` field added. | Contract tests generated (not stubs). | M | **Done** — stubs replaced with real test bodies + `#[ignore]` attribute. |

---

## Backlog

### Compiler Extensibility (parallel list consolidation)

Audit 2026-03-03: Measured actual LOC at each dispatch site. Categorized as IDENTICAL (pure boilerplate),
DERIVED (table-shaped metadata), or UNIQUE (genuinely different logic). Only truly redundant code targeted.

| ID | What | Redundant / Total LOC | Size | Notes |
|----|------|:---------------------:|------|-------|
| CX-1 | **Pipe Method Registry.** Single `PipeMethodDef` table replaces metadata boilerplate across 10 sites (enum as_str/from_str, type inference, callable contracts, collection_op_kind, classify_collection, node labels). Eval logic (465 LOC in `eval_pipe_method`/`evaluate_collection`) stays — genuinely different per method. Also: Site 6 (`lib.rs:8399`) is a trivially eliminable re-implementation of `as_str()`. Sites 8+12 (node labels) are exact duplicates across crate boundaries. | 514 / 1,010 | M | Prerequisite for FC-CF2/CF3. 6 pipe methods have no eval impl (fall through to sibling-fn catch-all). |
| CX-2 | **Deduplicate `lower_expr`/`remap_expr_idents`.** 85% identical (297 LOC total). Only 2 real differences: FieldAccess flattening (`param.field` → `param__field` for port naming, unique to remap) and `variant_names` awareness (unique to lower_expr). **Latent bug**: `remap_expr_idents` lacks variant_names support — variant constructors in remapped expressions silently misclassified. | 220 / 297 | S | Fix latent bug + delete ~148 lines. Single parameterized function with `remap_field_access: bool` + `variant_names: Option<&HashSet>`. |
| CX-3 | **Type mapping consolidation.** Only 36 of 68 LOC across 5 sites could use existing `DslTypeMapping` table (3 Go/Rust type sites). 2 sites are genuinely different domains (pipe method return types, test matchers). | 36 / 68 | S | Low priority. Existing table pattern is correct — just 3 sites bypass it. Extend with refinement aliases (`NonEmptyStr`→`String`, `Url`→`String`) and `Char`→`char`. |
| CX-4 | **Callable Item group helper.** Only FuncDef=PatternDef is truly redundant (~70 LOC copy-paste across 5 lowerer helpers). FnDef genuinely differs (single return type vs outputs list, no uses/provides). Typecheck main validation (197 LOC) has real structural differences per item type. | 70 / 304 | S | `CallableItem` trait with `name()`, `params()`, `body()`, `uses()`, `provides()` methods. FnDef adapter returns `&[]` for uses. Won't touch typecheck main validation. |
| CX-5 | **Structural primitive `is_structural()`.** Same 11 `PrimitiveOpKind` variants in same order at 3 sites, each mapping to a "no-op" sentinel. 100% redundant. | 60 / 60 | S | `PrimitiveOpKind::is_structural()` method. Ensures new structural variants auto-propagate to all 3 sites. |

### Compiler Features (low priority — unblocked by CX-1)

| ID | Feature | Size | Notes |
|----|---------|------|-------|
| FC-CF2 | `skip(n)`: List\<T\> → List\<T\> | S | After CX-1: 1 table entry instead of 10 file edits |
| FC-CF3 | `enumerate()`: List\<T\> → List\<(Int, T)\> | M | After CX-1: 1 table entry instead of 10 file edits |

### Transport Completeness

| ID | Scope | Ops | Notes |
|----|-------|-----|-------|
| RF-TC4 | Stub providers (unit_test profile) | 28 | Consider `transport stub {}` marker |
| RF-TC5 | Infrastructure stubs (azure, aws, gcp-infra) | 140 | Defer until provisioning lane |

### Deleted Tests (re-add when root cause fixed)

| ID | Deleted Tests | Blocker | Status |
|----|---------------|---------|--------|
| RF-E5 | `makegen_runtime_differential_interpreter_vs_generated_rust_layer1` | FnBodyDelegate gap | Open |
| RF-E6 | `makegen_exec_runtime_e2e`, `pragma_exec_runtime_e2e`, `makegen_e2e_generated_binary`, `pragma_e2e_generated_binary` | Exec-runtime emitter | Open |

### Blue Backlog

| ID | Item | Size | Priority |
|----|------|------|----------|
| H10 | Compute stack orchestration: Cloud Run/GCS/LB lifecycle DAG builder | L | P2 |
| S12-E | Multi-worker CAS: GcsClaimStore with generation-based CAS | M | P2 |
| H1 | Display reactive DSL: channel-driven event loop | XL | P3 |

### C28 Phase 2 (Future)

| ID | Task | Size | Notes |
|----|------|------|-------|
| C28-P2 | **Daggen cache manager.** Content-hash cache key from DSL source files → `.dagbin` serialized DAGs → skip recompilation on cache hit. | M | Infrastructure ready (serde derives + API). |
| C28-P3 | **Daggen codegen integration.** Wire `CodegenSubcommand::Daggen` in codegen binary → serialize all tool DAGs at `make codegen` time → runtime loads from cache. | L | Eliminates runtime DSL parsing. |

---

## Observations

Active observations only. Resolved items archived.

| # | Smell | Observation | File | Date |
|---|-------|-------------|------|------|
| 3 | Static mapping table | Kitchen sink `default_rest_response()` grows per service type | `core/test/src/auto_mock.rs` | 2026-02-27 |

### Resolved Observations (archived 2026-03-03)

- `workflow_unit_commands()` string dispatch → **resolved** — uses `resolve_workflow_variant()` structured dispatch (2026-03-03)
- `gunbc-codegen cigen` drops GCP secrets → **resolved** — secrets correctly derived via testgen registry `iter_dag_specs()` (2026-03-03)
- `passthrough_fallback_value()` hard-coded port alias table → **deleted** (C15, commit 33513ac9)
- `looks_effectful_without_kind()` re-derives NodeKind from port strings → **deleted** (C18, commit 33513ac9)
- `classify_module()` inflated by transitive auth callables → **documented** in doc comment
- `from` path format split (`.` vs `/` separator) → **standardized** to `/` separator
- `IdentityCallableOp` overloaded for 2 roles → **split** into `OutputPathMetadataOp` + `ResourcePassthroughOp`
- `probe_best_response` pessimistic ordering → **reordered** to REST-first
- `@mock_response` type in AST, parser never populates → **deleted** (C8)
- `match field.type_id.as_str()` JSON→Value dispatch → **replaced** with `from_bridge_json_typed` (C30)
