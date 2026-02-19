# Completed Tasks — Archived from tasks.md

**Moved**: 2026-02-19

---

## Sprint 1: Get to Green (Complete 2026-02-19)

2984 passing, 0 failures.

| ID | Task | Status |
|----|------|--------|
| F-GCP | GCP prepare ops: graceful `unwrap_or("(unresolved)")` for missing `audience`/`project`/`secret`/`subject_token` inputs in `ServiceGcpStsExchangePrepareOp` and `ServiceGcpSecretManagerAccessVersionPrepareOp` (resolve.rs) | Done 2026-02-19 |
| F-OBS | Observability invariant: `auto_mock_spec` fallback NonEmpty matchers for `IdentityCallableOp` terminal nodes (mock_defaults.rs), testgen regenerated | Done 2026-02-19 |
| F1 | dag_viz: removed SubDag wrapper NodeExamples (`gist_upload`/`browser_open`) from `auto_mock_spec` — wrapper nodes don't exist in lowered DAG, so `execute_single_node` can't find them. Observability analysis runs on lowered DAG, so no matchers needed. | Done 2026-02-19 |

---

## Sprint 2: Review Findings + Polish (Landed 2026-02-19)

| ID | Task | Status |
|----|------|--------|
| R1 | Makegen transport port naming alignment (`response` across lowerer parity filter, exec runtime emitter, and makegen mocks). | Done 2026-02-19 |
| R5 | MIPS lowering now routes returns through `JumpEpilogue`, temp allocation is fail-closed with explicit `LowerError` on exhaustion, and C block-scope locals are tracked with enter/exit visibility scopes to prevent leakage/aliasing across blocks. | Done 2026-02-19 |
| R8 | `MethodMeta` request wiring centralized through shared `request_from_meta(_at)` helpers; GCP service methods migrated off duplicated endpoint strings and parity tests added to catch metadata/request drift. | Done 2026-02-19 |
| R9 | Infra CLI `parse_input_value` fail-closed parsing now type-driven via `ValueBacking` + compatibility checks; structured JSON/list/map/set parsing and incompatibility/unsupported tests added. | Done 2026-02-19 |
| R10 | `SystemModel` REST invocation paths now use named placeholders and validation enforces wildcard ban + placeholder↔required-input binding (with invalid-path regression tests). | Done 2026-02-19 |
| R11 | Strict parsing APIs (`try_parse`/`FromStr`) landed for `Arch`/`Vendor`/`Os`/`AbiEnv`/`ExecutionEnv` with tolerant `parse` retained for host detection paths; strict-vs-tolerant tests added. | Done 2026-02-19 |
| R12 | Mock default seeding now prefers refined typed GCP semantic aliases, with legacy port-name heuristics isolated behind an explicit compatibility fallback; rename-resilient typed seeding tests added. | Done 2026-02-19 |
| P1 | `daglang-derive` capture mode now derived structurally from `obligation` + `is_interactive` metadata (no blanket captured default logic). | Done 2026-02-19 |
| P2 | `daglang-derive` interactive node detection now uses structural `is_interactive: bool` on `LoweredOp::Callable` (no `name.contains("@interactive")`). | Done 2026-02-19 |
| P3 | `daglang-derive` resource usage derivation now uses `obligation` enum + `resource_target` metadata (no string prefix stripping). | Done 2026-02-19 |
| P4 | `daglang-cli check` no longer re-runs discovery/parse/typecheck after pipeline build; reuses build-stage module graph. | Done 2026-02-19 |
| P5 | GCP impersonation parse now surfaces `expires_at` output (and propagates empty string when skipped), with graph/output tests updated. | Done 2026-02-19 |
| P8 | Repeated GCP REST client constructors (`new` / `unauthenticated`) consolidated via shared helper macro across service clients. | Done 2026-02-19 |
| P9 | `content_upsert` source wiring deduplicated via shared helpers for resolved-source / param-source fanout paths. | Done 2026-02-19 |
| P10 | makegen compile tests now use a shared fixture object with automatic temp output cleanup via `Drop`. | Done 2026-02-19 |
| P11 (Sprint 2) | `build_workspace_dag_from_discovery(tool_names, pipeline_names)` extracted as a pure composition entrypoint; impure discovery wrapper delegates to it. | Done 2026-02-19 |

---

## Completed Near-Term Polish

| ID | Task | Status |
|----|------|--------|
| P7 | Remove `dedupe_release_resource_edges` (resolve.rs): tracked already-wired `(release_node, port)` pairs in lowerer via `wired_release_targets: HashSet`, seeded from lifecycle acquire→release edges. Removed workaround from resolve.rs. | Done 2026-02-17 |
| P11 | Auto-mock seeding for GCP service inputs: added `gcp_field_value()` in mock_defaults.rs for `audience`/`project`/`secret`/`subject_token`/`version`/`service_account`. Used in entrypoint ports, required terminal inputs, and optional terminal inputs. | Done 2026-02-17 |

---

## Completed Infrastructure (H2-H11)

| ID | Feature | Implementation | Status |
|----|---------|----------------|--------|
| H2 | Testgen dynamic targets | `iter_dag_specs()`, `#[testgen_target]` macro, 27 targets via inventory | Done |
| H3 | Makegen tool registry | `#[tool_target]` macro, inventory-driven `ToolRegistry` | Done |
| H4 | Loop extra inputs passthrough | `execute_loop_body` injects extras, DSL `with` clause parsed | Done |
| H7 | Resource abstraction trait | `Resource` trait, `AccessMode`, `ManagedResource`, capability validation | Done |
| H8 | Justfile renderer | `render_justfile()`, parity test with Makefile | Done |
| H9 | GitHub Actions renderer | DAG→`needs` mapping, YAML generation, GitLab CI too | Done |
| H11 | DAG typing hardening | `TypedPort<T>`, `TypedInput<T>`, `TypedOutput<T>`, `PortTypeTag` trait | Done |

---

## Completed Work Summary (2026-02-18)

### DynOp Type-Dispatch Elimination (T1-T8)
~5,950 lines deleted, ~300 added. `WorkspaceOp` enum, 16 `From` impls, 10 converter fns,
`FileOpsGraph<T>`, `ResolvedOp`, `RuntimeOpId` — all removed. Central `resolve.rs` replaces
hand-built graph builders for all 7 tool modules.

### Active Cleanup (C1-C6)
Resolver hardening, lowering hardening, exec-runtime literal/param source support,
makegen path regression fix, mock cleanup, transport-call consolidation.

### Wave 1 — DSL Migration & Quality
- **1A (M1-M3)**: Pragma and codegen DSL parity verified, pragma binary wired into build system
- **1B (B1-B4)**: Bridge hygiene — `Optional<T>` prefix, naming invariant test, string inspection removal
- **1C (Q1-Q10)**: 10 code quality items — panic→Err, expect→?, `ParamType` enum, `HashSet`, `&Path`, `write!()`, `Cow<'static, str>`, etc.
- **1D (S1-S4)**: Seed policy — scenario/live-flow matrices, enforcement tests, fail-closed carriers
- **1E (D1-D3)**: `StableHashOp` extraction, test redundancy review, hermeticity annotation design

### Wave 2 — System Model & Structure
- **2A (R1-R6)**: System model refactor — `Dag<TypeOp>`, TypeRegistry, contract derivation, store mapping, `PortType`, cross-provider coercion
- **2B (SD1-SD3)**: Structural derivation — inventory registries, `Box<dyn Executable>`, `From` impl elimination
- **2C (W1-W4)**: Workflow registry — `WorkflowSpec`, registration, Makefile generation, git freshness
- **2D (CQ1-CQ4)**: Codegen quality — obligation mapping, prefix-heuristic elimination, parity snapshots, CodeIR plumbing
- **2E (CT1-CT3)**: CLI contracts — `--dry-run` tests, `--print-inputs json`, testgen obligations

### Wave 3 — Domain & Consolidation
- **3A (E1-E6)**: Domain completion — scope verification, gist defaults, WIF bootstrap, infra CLI, login flow, health check
- **3B (CL1-CL3)**: Cross-language audit — generated Rust clippy clean, generated Go vet clean, IR gaps fixed
- **3C (CO1-CO7)**: Consolidation — DynOp made CO1 moot, MergeOutputs split, probe-observer bundle, seed policy IR, live-secret metadata, execution trace, ValueKind

### Wave 4+ Horizon (Completed)
- **H2-H12**: Testgen dynamic targets, makegen tool registry, loop extra inputs, Fermi guards, cardinality modeling, resource abstraction, workflow rendering (Makefile/CI), compute stack, DAG typing, integration test targets

### Pre-Sprint Tracks
- **Track A (DSL Core)**: 4-target codegen (Rust/Go/C/MIPS), exec-runtime, cross-language parity
- **Track C (Modeling)**: Type coercion, workspace model, platform, browser, transport DAG, system model
- **Track D (Logging)**: DisplayConfig, secret redaction, stderr capture, failure-first, grouped progress
- **Track B (Workflow Audit)**: Purity, resource declarations, test registry
- **P3 (ValueBacking)**: Centralized type→Value backing in core/ir
- **Architecture Debt A-C**: Infra extraction, mtime fast path, design fixes
- **25/35 hacks resolved, 18/18 consolidation items §9-15 resolved**
