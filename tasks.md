# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)
**Archive**: `TODO/TODONE/tasks-archive-2026-03-02.md` (68 completed items from Lanes 1-3)

---

## Status Summary

```
Lane 1: Compiler Critical Path — 5 open (C10, C10a, C22-P2, C24, C25)
Lane 1: Binary Elimination    — 3 open (A2, A3, A4)
Phase 3: Purist Engine        — 4 open (C28, C29, C30, CT-8)
Backlog                       — 7 items (low priority)
```

### Critical Path

```
C10 (return expr) ──→ C10a (auth wiring) ──→ production gist/CI
        L                    M
C24 (pure dataflow) ──→ C25 (service codegen) ──→ C28 (daggen AOT)
        XL                    XL                      XL
```

---

## Lane 1: Compiler Pipeline Refactor (Open Items)

22 of 26 items complete. See archive for C1-C9, C11-C19, C21, C23, C26, C27.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | C10 | **Resolve ReturnExprCompute split-brain (RT4a/c).** Desugar complex returns (BinOp/If/Match/Pipe/...) into explicit DAG semantics. `resolve_return_expr_source` handles BinOp/If/Match/UnaryOp/Record but ExprCompute fallback remains for compound exprs. | Zero `ReturnExprCompute` in any compiled graph. No silent return-binding drops. | L | Partial |
| 2 | C10a | **`make gist` auth credential bridge fix.** Postmortem Option A/B/C. | `make gist` no longer 401s. | M | Open |
| 3 | C22 | **Deductive Redundancy Elimination Phase 2.** Phase 1 (compile-time `StaticFingerprint`) **done**: `stamp_static_fingerprints()` stamps transport nodes, `InputProvenance` + `StaticFingerprint` types in IR, `validate_fingerprint_uniqueness()` catches duplicates. Phase 2: test-time execution ledger — mock interceptor records `(OperationKey, Hash)` tuples, test runner asserts uniqueness per workflow. | Execution ledger catches dynamic redundancy (string interpolation, loop duplicates). | L | Phase 1 Done |
| 4 | C24 | **Pure dataflow lowering (kill `ExprComputeOp` + `__` hack).** Desugar `BinOp`, `If`, `Match`, `FieldAccess` into primitive structural DAG nodes. Step 1 done: `GetField` primitive op + 5 additional primitive ops. ~175 ExprCompute remain for compound expressions. **Design doc**: `docs/design/pure-dataflow-lowering.md`. | Zero `ExprComputeOp` in any compiled graph. `__` convention deleted. `referenced_vars` deleted. | XL | Partial |
| 5 | C25 | **Service-driven codegen (kill handwritten ops).** Compiler generates `DynOp` from service definitions (response blocks, exit blocks, transport class metadata). Delete `extern_ops.rs` dispatch table. **Design docs**: `docs/design/service-codegen.md`, `docs/design/pure-dataflow-lowering.md` §4. | Zero handwritten `DynOp` for services. `extern_ops.rs` derived from DSL. | XL | Open (needs C24) |

---

## Lane 1: Binary Elimination (Open Items)

7 of 10 items complete. See archive for A1, A5, A7-A11.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | A2 | **Eliminate `deps_config.rs`.** 117 LOC, actively referenced by makegen, CI pipeline, resource defs. Need DSL tool def for verify/ensure modes. | Generated binary has feature parity. `make deps-config` works. | S | Partial |
| 2 | A3 | **Eliminate `pipeline.rs` binary.** Original binary deleted. `daglang-cli/src/pipeline.rs` (3174 LOC) is compiler infrastructure (not the eliminated binary). No generated replacement tool yet. | Generated binary has feature parity. `make pipeline` works. | M | Partial |
| 3 | A4 | **Eliminate `workflow.rs`.** 346 LOC, still registered in Cargo.toml. Plan rendering logic needs DSL tool def. | Generated binary has feature parity. `make workflow` works. | L | Partial |

---

## Phase 3: The Purist Engine

C24 is the keystone dependency.

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 1 | C28 | **Daggen (AOT DAG Compilation).** Compile lowered DAGs into static `Dag<T>` Rust structs during `make codegen`. Zero DSL parsing at runtime. | XL | Pending | C24 |
| 2 | C29 | **Dynamic JSON-Path Output Mappings.** Extend JSONPath extraction to successful responses (`issue_id: "$.id"`). | M | Pending | TL-16 (done) |
| 3 | C30 | **Strict Type-Aware JSON Bridging.** Make `from_bridge_json` type-aware. Delete `__enum` JSON hacks. Emit structural `TypeShape` discriminant. Recent progress: `strip_optional_wrapper()`, `split_map_type_params()`, enum `ty` field cleanup. | M | Partial |
| 4 | CT-8 | **Wire Contract Test Generation.** Connect `StructuredContract` + `ProviderResponseContract` to testgen. `unimplemented!()` stubs placed. | M | Stub | CT-1 (done) |

---

## Backlog

### Compiler Features (low priority)

| ID | Feature | Size | Notes |
|----|---------|------|-------|
| FC-CF2 | `skip(n)`: List\<T\> → List\<T\> | S | Expressible via `fold` with index tracking |
| FC-CF3 | `enumerate()`: List\<T\> → List\<(Int, T)\> | M | Expressible via `fold` with counter accumulator |

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

---

## Observations

Active observations only. Resolved items archived.

| # | Smell | Observation | File | Date |
|---|-------|-------------|------|------|
| 1 | String dispatch | `match field.type_id.as_str()` for JSON→Value appears twice | `core/resolve/src/service_ops/service_ops_impl.rs` | 2026-02-26 |
| 2 | String dispatch | `workflow_unit_commands()` matches workflow name strings | `gunbc-dag/src/workflow/unit_commands.rs` | 2026-02-26 |
| 3 | Static mapping table | Kitchen sink `default_rest_response()` grows per service type | `core/test/src/auto_mock.rs` | 2026-02-27 |
| 4 | Inventory linkage gap | `gunbc-codegen cigen` drops GCP secrets | `gunbc-dag/src/ci/mod.rs` | 2026-02-26 |

### Resolved Observations (archived 2026-03-02)

- `passthrough_fallback_value()` hard-coded port alias table → **deleted** (C15, commit 33513ac9)
- `looks_effectful_without_kind()` re-derives NodeKind from port strings → **deleted** (C18, commit 33513ac9)
- `classify_module()` inflated by transitive auth callables → **documented** in doc comment
- `from` path format split (`.` vs `/` separator) → **standardized** to `/` separator
- `IdentityCallableOp` overloaded for 2 roles → **split** into `OutputPathMetadataOp` + `ResourcePassthroughOp`
- `probe_best_response` pessimistic ordering → **reordered** to REST-first
- `@mock_response` type in AST, parser never populates → **deleted** (C8)
