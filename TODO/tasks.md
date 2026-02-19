# Task Sheet — Dependency-Ordered, Parallelizable

**Last updated**: 2026-02-18
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Archive**: Completed items in `TODONE/`. Original TODO details preserved in git history.

**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

### Conventions

- **Definition of Done**: each task is done when code compiles, tests pass, and clippy is clean.
- **Cutover tasks** (M1-M3) additionally require: golden/snapshot test, failure-path test,
  build/Makefile target updated, CI job proves the new binary executes.
- **Deletion tasks** (T4, T5, T7) additionally require: `rg` search for straggler references,
  at least one end-to-end `--dry-run` per migrated tool.
- **Cross-backend tasks** (B1, B2) additionally require: non-Rust renderer compiles after change.
- **Code TODO/HACK comments** must reference a task ID (e.g., `TODO(SD1): ...`) so orphans
  are discoverable via grep.
- **Active Docs invariant**: every path in the task sheet must exist; no doc under
  `TODO/TODONE/` may appear in active sections.

---

## Active Cleanup (Thread Follow-up)

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **C1** | Resolver hardening: fail fast for unknown modules and unknown service prepare/parse callables; remove silent literal decode fallback | — | S | 2026-02-18 | 2026-02-18 |
| **C2** | Lowering hardening: wire `content_upsert(path: ...)` literal/source args into prepare nodes and shrink CI node-id path mapping accordingly | C1 | S | 2026-02-18 | 2026-02-18 |
| **C3** | Exec-runtime hardening: support lowering-generated `call_literal_source::*` and `call_param_source::*` nodes in layer-1 codegen/runtime classification (no unresolved node fallback) | C1, C2 | M | 2026-02-18 | 2026-02-18 |
| **C4** | Makegen path regression cleanup: restore real-mode/custom-output behavior in daglang compile-run tests and remove temporary literal-node override workaround | C2, C3 | S | 2026-02-18 | 2026-02-18 |
| **C5** | Makegen mock cleanup: remove legacy prepare-node path mock fallbacks and require callable/param-source path wiring | C2, C3, C4 | S | 2026-02-19 | 2026-02-19 |
| **C6** | Consolidation sweep: centralize transport-call analysis in emit backends and shared authenticated REST request helpers across GCP services | — | M | 2026-02-19 | 2026-02-19 |

---

## Critical Blocker: DynOp Type-Dispatch Elimination

**Why first**: 1,350 lines of zero-logic boilerplate. Adding a new tool costs ~60 lines across 4-5 files.
DSL-compiled `.dag` files exist but can't replace hand-built Rust graph builders without `DynOp`.
Every "wire DSL binary into build system" task (B1.1b/c, B1.3b) is blocked on this.

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **T1** | Add `DynOp` to `core/exec` (~20 lines: `Arc<dyn Executable + Send + Sync>`, Clone via Arc) | — | S | 2026-02-18 | 2026-02-18 |
| **T2** | Central resolver `gunbc-dag/src/resolve.rs`: `LoweredOp` → existing domain ops via `DynOp::new()` | T1 | M | 2026-02-18 | 2026-02-18 |
| **T3** | DSL builder per module (~15 lines each): compile `.dag` → `resolve_lowered_dag()` → `Dag<DynOp>` | T2 | M | 2026-02-18 | 2026-02-18 |
| **T4** | Delete manual `graph.rs` builders (pragma, codegen, makegen, ci, bootstrap, build, docgen — ~4,000 lines) | T3 | L | 2026-02-18 | 2026-02-18 |
| **T5** | Delete boilerplate layer: `WorkspaceOp` enum + 16 `From` impls + 10 converter fns + `FileOpsGraph<T>` (~600 lines) | T4 | M | 2026-02-18 | 2026-02-18 |
| **T6** | Delete/simplify `daglang-exec-bridge`: `ResolvedOp` + `RuntimeOpId` + duplicated handlers (~300 lines) | T2 | M | 2026-02-18 | 2026-02-18 |
| **T7** | Update callers: bin/*.rs, graph_mock.rs, parity tests, integration tests, resource registry tests | T4, T5 | L | 2026-02-18 | 2026-02-18 |
| **T8** | Lib crate cleanup: DynOp for DepsGraphOp, ClippyGraphOp, GistGraphOp, GcpGraphOps, ReviewGraphOp, LlmGraphOp (~300 lines) | T5 | M | 2026-02-18 | 2026-02-18 |

**Net**: ~5,950 lines deleted, ~300 added.

---

## Wave 1 — Ready Now (no blocker dependency)

These can start immediately, in parallel with the DynOp work above.

### 1A: DSL Migration Cutover

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **M1** | Pragma: verify generated binary produces identical output to hand-built | T3 | S | 2026-02-18 | 2026-02-18 |
| **M2** | Pragma: wire DSL binary into build system (replace hand-built pragma binary) | M1, T4 | M | 2026-02-18 | 2026-02-18 |
| **M3** | Codegen: verify generated binary matches hand-built codegen behavior | T3 | S | 2026-02-18 | 2026-02-18 |

### 1B: Bridge & Codegen Hygiene

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **B1** | `codegen_bridge` optional handling: replace Rust-specific `Option<T>` wrapping (line 170) with abstract `Optional<T>` prefix so non-Rust renderers don't get leaked Rust types | — | S | 2026-02-18 | 2026-02-18 |
| **B2** | Bridge naming invariant: add test asserting `BridgeModule` names pass through as canonical identifiers and renderers apply per-language casing (not bridge) | — | S | 2026-02-18 | 2026-02-18 |
| **B3** | Remove `body.contains("inputs.get(")` string inspection in `rust_exec_runtime.rs:831` — always name param `inputs` (fragile heuristic for unused-var warning) | — | S | 2026-02-18 | 2026-02-18 |
| **B4** | Add inline `// Raw because: ...` comments at each `Item::Raw` site in `rust_exec_runtime.rs` (5 sites — comments exist at call sites but not at definitions) | — | S | 2026-02-18 | 2026-02-18 |

### 1C: Code Quality

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **Q1** | Replace `panic!()` with `Err()` in `lib/transport/src/cli.rs` Result-returning fns (§17.6) | — | S | 2026-02-18 | 2026-02-18 |
| **Q2** | Replace `.expect()` with `?` in graph builders (~70 sites) (§17.2) | — | M | 2026-02-18 | 2026-02-18 |
| **Q3** | Replace `.is_none()`+`.unwrap()` with `ok_or_else` in `core/ir/src/builder.rs` (§17.3) | — | S | 2026-02-18 | 2026-02-18 |
| **Q4** | Introduce `ParamType` enum for CLI param types (§17.4) | — | S | 2026-02-18 | 2026-02-18 |
| **Q5** | `HashSet` for set operations in `lib/primitives/src/collection.rs` + `core/ir/src/value.rs` (§17.5) | — | S | 2026-02-18 | 2026-02-18 |
| **Q6** | `&PathBuf` → `&Path` in function signatures (§17.7) | — | S | 2026-02-18 | 2026-02-18 |
| **Q7** | `push_str(&format!())` → `write!()` (~100 sites) (§17.8) | — | L | 2026-02-18 | 2026-02-18 |
| **Q8** | `Cow<'static, str>` for struct fields with literal values (~80 allocs) (§17.9) | — | M | 2026-02-18 | 2026-02-18 |
| **Q9** | `BlobHandleError::new(&str)` → `impl Into<String>` (§17.10) | — | S | 2026-02-18 | 2026-02-18 |
| **Q10** | Minor nits: `.to_string_lossy().into_owned()`, `BTreeSet` for dedup, O(n^2) GitLab stages, dead statement, char_indices, `/root` fallback, execute_run dedup, MapToGcp dedup, lossy gist placeholder (§17.11-17.19) | — | M | 2026-02-18 | 2026-02-18 |

### 1D: Seed Policy Follow-ups

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **S1** | Extend seed matrix: define matrix for scenario context | — | S | 2026-02-18 | 2026-02-18 |
| **S2** | Extend seed matrix: define matrix for live-flow context | — | S | 2026-02-18 | 2026-02-18 |
| **S3** | Seed matrix enforcement tests | S1, S2 | S | 2026-02-18 | 2026-02-18 |
| **S4** | Unknown semantic carriers fail closed (strict, no silent fallback) | — | S | 2026-02-18 | 2026-02-18 |

### 1E: Debt & Consolidation

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **D1** | Extract `StableHashOp` to `lib/primitives`, add `DeduplicateOp` | — | M | 2026-02-18 | 2026-02-18 |
| **D2** | Review hand-written tests for redundancy with testgen (Pattern 1, 5 from §7) | — | M | 2026-02-18 | 2026-02-18 |
| **D3** | Design hermeticity annotation for `Shell` transport (producer-level, not variant-level) | — | S | 2026-02-18 | 2026-02-18 |

---

## Wave 2 — After DynOp or Wave 1 Deps

### 2A: System Model Refactor (Dag<TypeOp>)

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **R1** | Design: map `SystemModel` fields to `Dag<TypeOp>` nodes (behaviors as typed sub-DAGs) | — | M | 2026-02-18 | 2026-02-18 |
| **R2** | Register system behavior type DAGs in `TypeRegistry` | R1 | L | 2026-02-18 | 2026-02-18 |
| **R3** | Replace `derive_contract_test_specs()` with predicate-driven derivation from type DAG `Validate` nodes | R2 | M | 2026-02-18 | 2026-02-18 |
| **R4** | Replace `validate_store_behavior_mapping()` with structural DAG equivalence check | R2 | M | 2026-02-18 | 2026-02-18 |
| **R5** | Replace `rust_type_for_type_id()` with `PortType`-based derivation | R2 | S | 2026-02-18 | 2026-02-18 |
| **R6** | Cross-provider coercion test: GcpSecretPayload / AwsSecretValue via DAG walk | R2 | S | 2026-02-18 | 2026-02-18 |

### 2B: Structural Derivation

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **SD1** | Replace hardcoded tool/binary lists (5+ files) with inventory-derived registries | T8 | M | 2026-02-18 | 2026-02-18 |
| **SD2** | Consider `Box<dyn Executable>` for workspace DAG dispatch | T5 | S | 2026-02-18 | 2026-02-18 |
| **SD3** | Eliminate manual `From` impls for `WorkspaceOp` (9 impls + ~15 match arms) | T5 | M | 2026-02-18 | 2026-02-18 |

### 2C: Workflow Registry

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **W1** | Define `WorkflowSpec` type with entry points, deps, resources | T4 | M | 2026-02-18 | 2026-02-18 |
| **W2** | Register all existing workflows (build, test, codegen, testgen, pragma, etc.) | W1 | M | 2026-02-18 | 2026-02-18 |
| **W3** | Generate Makefile targets from registry | W2 | M | 2026-02-18 | 2026-02-18 |
| **W4** | Fast-path freshness: integrate git HEAD + dirty state into workflow execution | W1 | S | 2026-02-18 | 2026-02-18 |

### 2D: Codegen Quality

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **CQ1** | Map `ObligationCategory` variants to canonical kind strings | — | S | 2026-02-18 | 2026-02-18 |
| **CQ2** | Replace prefix-heuristic branches in `canonical_kind_from_shape` with obligation lookups | CQ1 | M | 2026-02-18 | 2026-02-18 |
| **CQ3** | Verify parity snapshots unchanged | CQ2 | S | 2026-02-18 | 2026-02-18 |
| **CQ4** | Propagate obligation lookups to lower_go.rs, lower_c.rs, lower_rust.rs, dag_mermaid.rs (residual prefix heuristics). **Note**: requires plumbing obligation metadata through CodeIR `Expr::Call` — obligation is on `LoweredOp` but lost when emitted to CodeIR. dag_mermaid needs it on `NodeTopology`. | CQ2 | L | 2026-02-18 | 2026-02-18 |

### 2E: Contract / CLI Guardrails

Design: `docs/design/integration-testgen.md`. Tier 0 (Makefile contract) done in `gunbc-dag/tests/cli_contract.rs`.

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **CT1** | Tier 1: per-tool `--dry-run` contract tests (validate parsed inputs match CLI args) | — | M | 2026-02-18 | 2026-02-18 |
| **CT2** | Add `--print-inputs json` flag to generated CLIs for machine-readable input echo | CT1 | S | 2026-02-18 | 2026-02-18 |
| **CT3** | Generated per-tool contract test harness via testgen obligation | CT1 | M | 2026-02-18 | 2026-02-18 |

---

## Wave 3 — Build-On

### 3A: Domain Completion

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **E1** | Provider-granted scope verification (E1.3b) | — | M | 2026-02-18 | 2026-02-18 |
| **E2** | `make gist-recent` works without hidden hardcoded defaults (E1.5a) | E1 | M | 2026-02-18 | 2026-02-18 |
| **E3** | WIF Bootstrap DAG — idempotent setup flow (E2.2b) | — | L | 2026-02-18 | 2026-02-18 |
| **E4** | Unified infra CLI: bootstrap, plan, apply, spec, graph (E2.7a) | E3 | M | 2026-02-18 | 2026-02-18 |
| **E5** | Enhanced login flow: verify ADC, SA impersonate, direnv (E2.7b) | E3 | M | 2026-02-18 | 2026-02-18 |
| **E6** | Status/health check: auth, projects, SA, secrets (E2.7c) | E3 | S | 2026-02-18 | 2026-02-18 |

### 3B: Cross-Language & Test

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **CL1** | Audit generated Rust for remaining clippy issues (D3.3a) | T3 | M | 2026-02-18 | 2026-02-18 |
| **CL2** | Audit generated Go for golint/govet issues (D3.3b) | — | M | 2026-02-18 | 2026-02-18 |
| **CL3** | Document IR modeling gaps discovered and fix (D3.3c) | CL1, CL2 | M | 2026-02-18 | 2026-02-18 |

### 3C: Consolidation

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **CO1** | `ToolGraphOp<D>` generic wrapper or GraphOp wrapper enum unification (moot: DynOp migration solved this) | SD3 | M | 2026-02-18 | 2026-02-18 |
| **CO2** | Split `MergeOutputs` dedup from cardinality handling | — | M | 2026-02-18 | 2026-02-18 |
| **CO3** | Probe-observer analysis single-source bundle (consolidation §17.A) | — | M | 2026-02-18 | 2026-02-18 |
| **CO4** | Seed policy ownership in IR types, not testgen whitelist (consolidation §17.B) | — | M | 2026-02-18 | 2026-02-18 |
| **CO5** | Live-secret requirements as generated workflow metadata (consolidation §17.C) | — | M | 2026-02-18 | 2026-02-18 |
| **CO6** | Execution trace inputs for coercion/assertion observability (consolidation §17.D) | — | M | 2026-02-18 | 2026-02-18 |
| **CO7** | Add `ValueKind` enum on `Value` so `types_compatible` becomes `TypeId backing accepts ValueKind` without manufacturing type-name strings (eliminates `mock_value_type_name` smell) | — | M | 2026-02-18 | 2026-02-18 |

---

## Wave 4+ — Horizon / Deferred

These require DSL language features that don't exist yet, or are low priority.
Design drafts: `docs/design/horizon/README.md`.

### Needs DSL Reactive/Metaprogramming

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **H1** | Display orchestration DSL migration (channel-driven event loop, timer ticks) | DSL reactive primitives | XL | 2026-02-18 | |
| **H2** | Testgen dynamic targets (N upsert chains per DagSpecDef via inventory) | DSL metaprogramming | L | 2026-02-18 | 2026-02-18 |
| **H3** | Makegen tool registry (procedural target gen from #[tool_target]) | DSL metaprogramming | L | 2026-02-18 | 2026-02-18 |
| **H4** | Loop extra inputs passthrough (for body needs non-element context) | DSL for-loop enhancement | M | 2026-02-18 | 2026-02-18 |

### Low Priority / Design-Gated

| ID | Task | Deps | Size | Started | Done |
|----|------|------|------|---------|------|
| **H5** | Fermi guard live tests (blocked on GCP WIF + codegen for secrets) | E3 | M | 2026-02-18 | 2026-02-18 |
| **H6** | Cardinality compositional modeling (non-blocking) | — | L | 2026-02-18 | 2026-02-18 |
| **H7** | Resource abstraction trait for DAG-native resource management | design decision | L | 2026-02-18 | 2026-02-18 |
| **H8** | Rendering workflows as DAGs: Makefile generation (when adding Justfile) | second format consumer | L | 2026-02-18 | 2026-02-18 |
| **H9** | Rendering workflows as DAGs: CI workflow generation (when adding second provider) | second CI provider | L | 2026-02-18 | 2026-02-18 |
| **H10** | Compute stack service interfaces (GCE, Cloud Run, LB, GCS bucket) | E3 | XL | 2026-02-18 | 2026-02-18 |
| **H11** | DAG typing hardening: typed node I/O wrappers + semantic carrier refinements | — | L | 2026-02-18 | 2026-02-18 |
| **H12** | `make test-integration` / `make test-external` Makefile targets | — | S | 2026-02-18 | 2026-02-18 |

---

## Parallelization Guide

```
         ┌─ T1→T2→T3→T4→T5→T7 (DynOp critical path)
         │       └→T6          (exec-bridge cleanup, parallel with T3+)
         │       └→T8          (lib crate cleanup, after T5)
         │
SESSION  ├─ B1-B4              (bridge hygiene, fully independent)
         │
         ├─ Q1-Q10             (code quality, fully independent)
         │
         ├─ S1-S4              (seed policy, fully independent)
         │
         ├─ D1-D3              (debt, fully independent)
         │
    ─────┤ (Wave 2 starts when T4+ complete)
         │
         ├─ R1→R2→R3,R4,R5,R6 (system model refactor)
         │
         ├─ SD1-SD3            (structural derivation, after T5/T8)
         │
         ├─ W1→W2→W3,W4       (workflow registry)
         │
         ├─ CQ1→CQ2→CQ3       (codegen quality)
         │
    ─────┤ (Wave 3)
         │
         ├─ E1→E2              (credential completion)
         ├─ E3→E4,E5,E6        (GCP CLI/UX)
         ├─ CL1,CL2→CL3       (cross-language audit)
         └─ CO1-CO7            (consolidation, mostly independent)
```

**Maximum parallelism at any point**: 5+ independent swimlanes.
**Critical path**: T1 → T2 → T3 → T4 → T5 → T7 (DynOp, ~2 weeks).
**Highest-ROI quick wins**: B1, B3, Q1, Q3, Q4, Q6 (all S-sized, zero deps).

---

## Completed Work Summary

All completed items are archived in `TODONE/` with dates. Key milestones:

- **Track A (DSL Core)**: DONE — 4-target codegen (Rust/Go/C/MIPS), exec-runtime, cross-language parity
- **Track C5 (Type Coercion)**: DONE — DAG-walk coercion, dual encoding elimination, stress tests
- **Track C7 (Workspace Model)**: DONE — WorkspaceLayout, glob derivation, parent() elimination
- **Track C1 (Platform)**: DONE — Arch/Vendor/Os/AbiEnv/TargetTriple/ExecutionEnv
- **Track C2 (Browser)**: DONE — cross-platform browser utility
- **Track C4 (Transport DAG)**: DONE — typed ports, behavioral specs, tests
- **Track D1 (Logging)**: DONE — unified DisplayConfig, secret redaction, stderr capture, failure-first, grouped progress
- **Track C6.1-C6.4**: DONE — system model types, GCP/AWS/transport models, contract tests
- **Track C6.5f-g**: DONE — model data distributed to owning crates via inventory
- **Track B workflow audit A-C**: DONE — purity, resource declarations, test registry
- **P3 (ValueBacking)**: DONE — centralized type→Value backing in core/ir
- **25/35 hacks resolved**, **18/18 consolidation items §9-15 resolved**
