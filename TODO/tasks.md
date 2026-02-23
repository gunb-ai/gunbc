# Task Sheet — Dependency-Ordered, Parallelizable

**Last updated**: 2026-02-23 (Lane 7 added)
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Archive**: Completed items in `TODO/TODONE/2026-Q1/tasks-completed.md`. Backlog in `TODO/backlog.md`.

**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

### Conventions

- **Definition of Done**: each task is done when code compiles, tests pass, and clippy is clean.
- **Code TODO/HACK comments** must reference a task ID (e.g., `TODO(P1): ...`) so orphans
  are discoverable via grep.
- **Active Docs invariant**: every path in the task sheet must exist; no doc under
  `TODO/TODONE/` may appear in active sections.

---

## Delivery Lane Summary

| Lane | Status | Notes |
|------|--------|-------|
| 1: Type system + graph builders | **DONE** | Archived 2026-02-22 |
| 2: 100% codegen pipeline | **DONE** | Archived 2026-02-22 |
| 3: Modeling integrity | **DONE** | Archived 2026-02-20 |
| Post-merge: Type system hard cutover | **DONE** | Archived 2026-02-22 |
| 4: Codebase polish | **DONE** | Archived 2026-02-22 |
| 5: GraphIR decommission (exclusive) | **DONE** | Archived 2026-02-22 |
| 6A: Topology migration — cleanup + subdags | **ACTIVE** | Parallel with 6B |
| 6B: Topology migration — cloud/GCP/LLM graphs | **ACTIVE** | Parallel with 6A |
| 6C: Topology migration — review graph stack | **ACTIVE** | Depends on 6B |
| 6D: Topology migration — ops semantics | **DEFERRED** | Depends on 6B + 6C |
| 7: Review cleanup | **ACTIVE** | Parallel with 6A/6B/6C (disjoint scope) |

---

## Current Open Work

Lane 6 (Topology Migration) is active. Sub-lanes 6A, 6B, and 6C contain all
scheduled work. 6A and 6B are independent and may run in parallel. 6C depends
on 6B. 6D is deferred pending a DSL executable-semantics design decision.

Lane 7 (Review Cleanup) is active and fully parallel with Lane 6 — disjoint
file sets, no blocking dependencies.

---

## Lane 5: GraphIR Decommission (Exclusive Lane)

**Goal**: Remove handwritten GraphIR authoring and route tool/workspace topology through DSL-only execution.

**Source of truth**: `docs/design/graphir-decommission-design.md` (section 9 inventory + section 10 backlog).

**Exclusive execution policy**: Run this lane by itself while active. It intentionally spans lowering/runtime/tool/workspace/provider/deletion surfaces and should not be mixed with other lanes to avoid partial migration states.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **GD-1** | Cut over DSL-module tool targets. | -- | M | Done (2026-02-22) |
| **GD-2** | Interactive/external lowering + passthrough. | GD-1 | M | Done (2026-02-22) |
| **GD-3** | Replace manual workspace subdags. | GD-1 | M | Done (2026-02-22) |
| **GD-4** | Delete section 9C legacy tool graph stacks. | GD-2, GD-3 | L | Done (2026-02-22) |
| **GD-5** | **Provider stack decision wave (section 9D)**: execute drop-now or migrate-in-place decisions, remove redundant handwritten stacks, and lock final policy in design docs. | GD-1 | XL | Done (2026-02-22) -- drop-now complete for AWS/Azure + cargo ops; cloud infra helper stacks consolidated (`infra_graph.rs` + `secret_provision_graph.rs` deleted) and migrated APIs folded into active modules; remaining cloud/gcp/llm/review/clippy/deps/gist stacks explicitly retained as migrated active wrappers and validated |
| **GD-6** | Fail-closed resolver + CI guardrails. | GD-4, GD-5 | M | Done (2026-02-22) |

### GD-5 resolution (2026-02-22)

Final decision matrix:

| Stack | Decision | Execution |
|-------|----------|-----------|
| AWS/Azure provider stacks + cargo ops | Drop now | Deleted legacy `graph.rs` / `graph_mock.rs` / `ops.rs`; unsupported facades retained where needed |
| Cloud infra helper stacks | Migrate + delete redundant stacks | Deleted `lib/cloud-ops/src/infra_graph.rs` and `lib/cloud-ops/src/secret_provision_graph.rs`; moved `render_infra_spec_dot` into `infra_spec.rs` and secret provision builders into `infra_plan_apply.rs` |
| Cloud/GCP/LLM/Review active graph stacks | Migrate in place (retain) | Kept as active typed graph builders and generic-interpreter execution path; provider drop-now policy enforced via fail-closed config/runtime checks |
| Tool graph wrappers (`clippy`/`deps`/`gist`) | Drop now | Deleted handwritten `graph.rs`/`graph_mock.rs`, removed `pub mod` declarations, deleted dead test files; DSL-only execution path |

Verification after migration-wave closeout:

1. `cargo check -p gunbc-lib-cloud-ops`
2. `cargo test -q -p gunbc-lib-cloud-ops`
3. `cargo test -q -p gunbc-lib-llm-ops`
4. `cargo test -q -p gunbc-lib-review`
5. `cargo check -p gunbc-dag`
6. `cargo test -q -p gunbc-dag --test resource_registry_coverage`
7. `cargo run -q -p gunbc-dag --bin gunbc-testgen -- --dry-run`

### Lane 5 exit criteria

1. `dsl_module` targets execute via DSL-backed builders only. **(Done)**
2. Section 9C files are deleted. **(Done)**
3. Section 9D decision wave is complete (drop-now deletions executed; retained migrated wrappers explicitly documented and validated). **(Done)**
4. Resolver is fail-closed and CI enforces non-regression. **(Done)**

---

## Lane 6: Full Repo Migration — Rust to .dag

**Goal**: Migrate ALL handwritten Rust behavior, policy, config, and ops off Rust and onto `.dag`, leaving only the compiler/runtime/transport layer in Rust. ~47K LOC migrated or deleted across 3 parallel lanes.

**Execution constraint — no fallbacks**:
- No `#[cfg(feature = "legacy")]` toggles or "try DSL, fall back to Rust" paths.
- No temporary wrapper shims that keep both execution paths alive.
- No `TODO(HACK)` escape hatches. Each task either fully replaces the Rust with `.dag` or it is not done.
- If the DSL compiler needs new features to express something, that is a blocking prerequisite — not something to work around with Rust glue.

**Two fully parallel lanes**:

| Lane | Scope | LOC | Parallel |
|------|-------|-----|----------|
| **Lane A** | All `lib/` crate behavior + lib-dependent binaries | ~35,200 | A ∥ B |
| **Lane B** | All `gunbc-dag/src/` behavior + tool binaries | ~14,400 | A ∥ B |

**Shared infrastructure**: `gunbc-dag/src/dsl_builder.rs` and `gunbc-dag/src/resolve.rs` are append-only shared files. Commits to shared files are atomic per lane.

Sub-lane task definitions follow. Task IDs preserve their original numbering for traceability.

### Lane A: lib/ Crates + lib-Dependent Binaries (~35K LOC)

**Goal**: Migrate ALL handwritten Rust behavior in every `lib/` crate to `.dag`, then delete the binaries that depend on them. After this lane, `lib/` crates contain only `lib.rs` re-exports and `system_models.rs` registrations, and the 4 lib-dependent binaries are trivially generated entry points.

**Mutually exclusive scope**: ALL `lib/` crate source files (excluding `lib/transport/` and `lib/primitives/`), plus `gunbc-dag/src/bin/{review,pipeline,infra,sdlc}.rs`.

**Sub-lanes**: 6A-1 (tombstones), 6B (cloud/gcp/llm), 6C (review), 6E (git/blob/gist/markdown/design), 6G (clippy/deps), 6I-A (lib-dependent binaries)

---

#### Tombstone Cleanup (tasks 6A-*)

**Scope**: `lib/tools/{clippy,deps,gist}/src/graph*.rs`

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6A-1** | Delete tombstone `graph.rs`/`graph_mock.rs` in `lib/tools/{clippy,deps,gist}/src/` and remove `mod graph` / `mod graph_mock` declarations from their parent `lib.rs` files. | -- | S | **Done** |

| File | Action |
|------|--------|
| `lib/tools/clippy/src/graph.rs` | Delete |
| `lib/tools/clippy/src/graph_mock.rs` | Delete |
| `lib/tools/deps/src/graph.rs` | Delete |
| `lib/tools/deps/src/graph_mock.rs` | Delete |
| `lib/tools/gist/src/graph.rs` | Delete |
| `lib/tools/gist/src/graph_mock.rs` | Delete |

> Tasks 6A-2 and 6A-3 (subdag rewrites) are in **Lane B** below.

---

#### Cloud/GCP/LLM Full Stack (tasks 6B-*)

**Goal**: Migrate ALL handwritten Rust in `lib/gcp-ops/`, `lib/cloud-ops/`, and `lib/llm-ops/` to `.dag`. This includes graph builders, `Executable` ops, typed GCP REST service interfaces, cloud config/policy definitions, and LLM prepare/parse ops. After this lane, the only Rust remaining in these 3 crates is `lib.rs` re-exports and `system_models.rs` type registrations.

**Mutually exclusive scope**: `lib/gcp-ops/src/` (ALL files), `lib/cloud-ops/src/` (ALL files), `lib/llm-ops/src/` (ALL files), `dsl/cloud/`, `dsl/services/gcp/`, `dsl/config/gcp_project.dag`, `dsl/config/credential_policy.dag`

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6B-1** | Author `.dag` modules for GCP WIF credential graphs (3 runtime variants: GitHub Actions, Cloud Metadata, LocalDev). Must express runtime-conditional subdag composition — no Rust `match` dispatch fallback. Write per-variant `.dag` files if the compiler lacks branching, but each variant must be fully DSL-authored. | -- | L | Pending |
| **6B-2** | Author `.dag` modules for GCP Secret Manager upsert graphs (3 runtime variants). Same constraint as 6B-1. | 6B-1 | M | Pending |
| **6B-3** | Author `.dag` module for GCP discovery graph (463 LOC). Wire `build_gcp_discovery_graph_dsl()` in `dsl_builder.rs`. | 6B-1 | M | Pending |
| **6B-4** | Author `.dag` modules for cloud-ops facade: provider-neutral credential + upsert dispatch and GitHub credential graph (460 + 391 LOC). The DSL equivalent uses `dsl/interfaces/*.dag` provider abstractions or explicit per-provider modules. No Rust `CloudProviderKind` match fallback. | 6B-1, 6B-2 | M | Pending |
| **6B-5** | Author `.dag` module for LLM chat completion graph (268 LOC). Must compose cloud credential subdag from 6B-4 DSL output. | 6B-4 | M | Pending |
| **6B-6** | Delete all Rust graph/mock files. Remove `pub mod graph` / `pub mod graph_mock` / `pub mod discovery_graph` / `pub mod github_credential_graph` from parent `lib.rs` files. Update all downstream `use` imports to point to DSL-backed builders. | 6B-1..6B-5 | M | Pending |
| **6B-7** | Migrate `lib/gcp-ops/src/ops.rs` (2,403 LOC) + `lib/gcp-ops/src/discovery_ops.rs` (810 LOC) to generic interpreters. GCP ops are predominantly REST prepare/parse pairs — each maps to `RestPrepareOp`/`RestParseOp` parameterized by `ServiceOperationSpec` extracted from `.dag` service definitions. Author `dsl/cloud/gcp/services.dag` with per-operation specs. Delete per-op Rust structs once the generic interpreter handles them. | 6B-1, 6B-3 | L | Pending |
| **6B-8** | Migrate `lib/cloud-ops/src/ops.rs` (441 LOC) to generic interpreters. Provider-neutral dispatch ops become DSL-expressed routing or thin profile-bound adapters. Credential-policy ops that are pure config transforms become DSL `fn`. | 6B-4, 6B-7 | M | Pending |
| **6B-9** | Final graph+ops cleanup: delete emptied ops files, remove dead `mod` declarations, verify no Rust `Executable` impls remain in `lib/{gcp-ops,cloud-ops}/src/`. | 6B-6..6B-8 | S | Pending |
| **6B-10** | Migrate `lib/gcp-ops/src/services/` (11 files, 3,328 LOC) to DSL service declarations. Each Rust typed API interface (IAM, Secret Manager, WIF, Cloud Run, Compute Engine, Storage, Resource Manager, Load Balancer, local_auth) becomes a `service` block in `dsl/services/gcp/*.dag`. The `MethodMeta` structs map directly to `@rest(METHOD, path)` annotations. Delete Rust service files once specs compile. | 6B-7 | L | Pending |
| **6B-11** | Migrate `lib/cloud-ops/src/` config/policy files to DSL config: `project_spec.rs` (911 LOC) -> `dsl/config/gcp_project.dag`; `credential_policy.rs` (251 LOC) -> `dsl/config/credential_policy.dag`; `infra_spec.rs` (200 LOC) -> `dsl/config/infra_spec.dag`; `env_requirements.rs` (207 LOC) + `env_status.rs` (91 LOC) -> DSL config nodes. | 6B-4 | M | Pending |
| **6B-12** | Migrate `lib/cloud-ops/src/` infra DAG builders to DSL: `infra_bootstrap.rs` (1,141 LOC) -> expand `dsl/infra/gcp/bootstrap.dag`; `infra_plan_apply.rs` (490 LOC) -> expand `dsl/infra/gcp/plan_apply.dag`. These are manual `DagBuilder` graphs — same migration pattern as 6B-1..6B-4. | 6B-10, 6B-11 | L | Pending |
| **6B-13** | Migrate remaining `lib/cloud-ops/src/` behavior files: `config_loader.rs` (902 LOC) -> DSL config chain with profile resolution; `secret_cache.rs` (202 LOC), `secret_rotation.rs` (149 LOC), `secret_exports.rs` (112 LOC) -> DSL `fn`; `login_flow.rs` (180 LOC), `health_status.rs` (153 LOC), `project_registry.rs` (158 LOC), `config_resource.rs` (190 LOC) -> DSL config + `fn`. | 6B-11, 6B-12 | L | Pending |
| **6B-14** | Migrate `lib/llm-ops/src/lib.rs` (1,009 LOC) — LLM prepare/parse ops (PrepareChatRequest, ParseChatResponse, etc.) to service operations in `dsl/services/llm/*.dag`. These follow the transport pattern and map to `RestPrepareOp`/`RestParseOp` generic interpreters. | 6B-5 | M | Pending |
| **6B-15** | Full crate cleanup: delete all migrated files across `lib/{gcp-ops,cloud-ops,llm-ops}/src/`. Only `lib.rs` re-exports, `system_models.rs`, and `generated_tests*.rs` should remain. Remove dead `mod` declarations. | 6B-9..6B-14 | M | Pending |

#### 6B Deletion Manifest

| File | LOC | Action |
|------|-----|--------|
| `lib/gcp-ops/src/graph.rs` | 1,760 | Delete |
| `lib/gcp-ops/src/graph_mock.rs` | 452 | Delete |
| `lib/gcp-ops/src/discovery_graph.rs` | 463 | Delete |
| `lib/gcp-ops/src/ops.rs` | 2,403 | Delete (generic interpreters + `.dag` service specs) |
| `lib/gcp-ops/src/discovery_ops.rs` | 810 | Delete (generic interpreters + `.dag` service specs) |
| `lib/gcp-ops/src/services/cloud_run.rs` | 366 | Delete (-> `dsl/services/gcp/cloud_run.dag`) |
| `lib/gcp-ops/src/services/compute_engine.rs` | 402 | Delete (-> `dsl/services/gcp/compute_engine.dag`) |
| `lib/gcp-ops/src/services/iam.rs` | 517 | Delete (-> `dsl/services/gcp/iam.dag`) |
| `lib/gcp-ops/src/services/iam_policy.rs` | 109 | Delete (-> `dsl/services/gcp/iam_policy.dag`) |
| `lib/gcp-ops/src/services/load_balancer.rs` | 408 | Delete (-> `dsl/services/gcp/load_balancer.dag`) |
| `lib/gcp-ops/src/services/local_auth.rs` | 142 | Delete (-> `dsl/services/gcp/local_auth.dag`) |
| `lib/gcp-ops/src/services/mod.rs` | 258 | Delete |
| `lib/gcp-ops/src/services/resource_manager.rs` | 187 | Delete (-> `dsl/services/gcp/resource_manager.dag`) |
| `lib/gcp-ops/src/services/secret_manager.rs` | 276 | Delete (-> `dsl/services/gcp/secret_manager.dag`) |
| `lib/gcp-ops/src/services/storage.rs` | 243 | Delete (-> `dsl/services/gcp/storage.dag`) |
| `lib/gcp-ops/src/services/workload_identity.rs` | 420 | Delete (-> `dsl/services/gcp/workload_identity.dag`) |
| `lib/cloud-ops/src/graph.rs` | 460 | Delete |
| `lib/cloud-ops/src/github_credential_graph.rs` | 391 | Delete |
| `lib/cloud-ops/src/ops.rs` | 441 | Delete (generic interpreters + DSL `fn`) |
| `lib/cloud-ops/src/project_spec.rs` | 911 | Delete (-> `dsl/config/gcp_project.dag`) |
| `lib/cloud-ops/src/config_loader.rs` | 902 | Delete (-> DSL config chain) |
| `lib/cloud-ops/src/infra_bootstrap.rs` | 1,141 | Delete (-> `dsl/infra/gcp/bootstrap.dag`) |
| `lib/cloud-ops/src/infra_plan_apply.rs` | 490 | Delete (-> `dsl/infra/gcp/plan_apply.dag`) |
| `lib/cloud-ops/src/credential_policy.rs` | 251 | Delete (-> `dsl/config/credential_policy.dag`) |
| `lib/cloud-ops/src/env_requirements.rs` | 207 | Delete (-> DSL config) |
| `lib/cloud-ops/src/secret_cache.rs` | 202 | Delete (-> DSL `fn`) |
| `lib/cloud-ops/src/infra_spec.rs` | 200 | Delete (-> `dsl/config/infra_spec.dag`) |
| `lib/cloud-ops/src/config_resource.rs` | 190 | Delete (-> DSL config + `fn`) |
| `lib/cloud-ops/src/login_flow.rs` | 180 | Delete (-> DSL config + `fn`) |
| `lib/cloud-ops/src/project_registry.rs` | 158 | Delete (-> DSL config) |
| `lib/cloud-ops/src/health_status.rs` | 153 | Delete (-> DSL config + `fn`) |
| `lib/cloud-ops/src/secret_rotation.rs` | 149 | Delete (-> DSL `fn`) |
| `lib/cloud-ops/src/secret_exports.rs` | 112 | Delete (-> DSL `fn`) |
| `lib/cloud-ops/src/env_status.rs` | 91 | Delete (-> DSL config) |
| `lib/llm-ops/src/graph.rs` | 268 | Delete |
| `lib/llm-ops/src/graph_mock.rs` | 1,013 | Delete |
| `lib/llm-ops/src/lib.rs` | 1,009 | Shrink (only re-exports + `system_models` registration remain) |
| **Total** | **~18,135** | |

#### 6B Exit Criteria

1. Zero `graph.rs`, `graph_mock.rs`, `discovery_graph.rs`, `github_credential_graph.rs`, `ops.rs`, or `discovery_ops.rs` files in `lib/{gcp-ops,cloud-ops,llm-ops}/src/`.
2. Zero files in `lib/gcp-ops/src/services/` — entire directory deleted. All GCP service specs live in `dsl/services/gcp/*.dag`.
3. Zero behavior files in `lib/cloud-ops/src/` — `project_spec.rs`, `config_loader.rs`, `infra_bootstrap.rs`, `infra_plan_apply.rs`, `credential_policy.rs`, `env_requirements.rs`, `secret_cache.rs`, `infra_spec.rs`, `config_resource.rs`, `login_flow.rs`, `project_registry.rs`, `health_status.rs`, `secret_rotation.rs`, `secret_exports.rs`, `env_status.rs` all deleted.
4. `lib/llm-ops/src/lib.rs` contains only re-exports and `system_models` registration — zero `Executable` impls.
5. All 3 GCP runtime variants (GitHub/Metadata/Local) for both credential and upsert graphs compile from `.dag` files.
6. `dsl_builder.rs` has new builder functions for each migrated graph.
7. No Rust `match` fallback dispatch remains for graph construction — runtime variant selection is either expressed in DSL or handled by per-variant `.dag` files selected at compile time.
8. All downstream consumers (`lib/review/src/graph.rs`, `gunbc-dag/src/bin/review.rs`) that import these builders still compile (same public API, now backed by DSL).
9. Zero hand-written `Executable` impls in `lib/{gcp-ops,cloud-ops,llm-ops}/src/` — all service operations use generic interpreters parameterized by `ServiceOperationSpec`.
10. `dsl/services/gcp/*.dag` is the authoritative source for all GCP service operation specs.
11. `dsl/config/gcp_project.dag`, `dsl/config/credential_policy.dag`, `dsl/config/infra_spec.dag` are authoritative for cloud config/policy.

#### 6B Verification

```
cargo check -p gunbc-lib-gcp-ops
cargo test -q -p gunbc-lib-gcp-ops
cargo check -p gunbc-lib-cloud-ops
cargo test -q -p gunbc-lib-cloud-ops
cargo check -p gunbc-lib-llm-ops
cargo test -q -p gunbc-lib-llm-ops
cargo test -q -p gunbc-dag --test resource_registry_coverage
cargo run -q -p gunbc-dag --bin gunbc-testgen -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-infra -- spec
cargo clippy --all-targets -- -D warnings
```

---

#### Review Full Stack (tasks 6C-*)

**Goal**: Migrate ALL handwritten Rust in `lib/review/` to `.dag`. This includes graph builders, mocks, review ops (prompt building, response parsing), the 4-dimension review model, and review profiles. After this lane, the only Rust remaining in `lib/review/` is `lib.rs` re-exports and `system_models.rs` type registrations.

**Mutually exclusive scope**: `lib/review/src/` (ALL files), `gunbc-dag/src/bin/{review,pipeline}.rs` (import changes only), `dsl/tools/review.dag`, `dsl/config/review_profiles.dag`, `dsl/config/review_dimensions.dag`

**Depends on**: Lane 6B (review graphs compose cloud credential subdags from `lib/cloud-ops` and LLM subdags from `lib/llm-ops`)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6C-1** | Expand `dsl/tools/review.dag` (currently 29 LOC) to express all 4 review graph builders: (a) `review_phase` — blob acquisition + cloud credential chain + LLM triplet + response parsing; (b) `inline_review` — direct content + LLM; (c) `diff_review` — git diff triplet + LLM; (d) `dimension_diff_review` — parallel 4-dimension fan-out (coherence/quality/requirements/aspirational) with aspirational depending on prior findings, plus fan-in merge. Must import cloud credential and LLM modules from 6B DSL output. No `add_cloud_credential_chain` or `add_transport_triplet_named_with_passthrough` Rust helpers — topology expressed in DSL. | 6B-6 | L | Pending |
| **6C-2** | Wire `dsl_builder.rs` to compile each review graph variant. Add `build_review_phase_graph_dsl()`, `build_inline_review_graph_dsl()`, `build_diff_review_graph_dsl()`, `build_dimension_diff_review_graph_dsl()`. | 6C-1 | S | Pending |
| **6C-3** | Replace review `graph.rs` with thin DSL wrappers (same pattern as `gunbc-dag/src/bootstrap/graph.rs`). The public API (`build_review_phase_graph()`, etc.) stays but delegates to DSL builders. | 6C-2 | M | Pending |
| **6C-4** | Update `gunbc-dag/src/bin/review.rs` and `gunbc-dag/src/bin/pipeline.rs` to use DSL-backed review graph builders. No direct `DagBuilder` construction in binaries. | 6C-3 | S | Pending |
| **6C-5** | Delete `lib/review/src/graph.rs` and `lib/review/src/graph_mock.rs`. Remove `pub mod graph` / `pub mod graph_mock` from `lib/review/src/lib.rs`. Mocks are now auto-generated from `@auto_mock` test annotations in `.dag`. | 6C-3, 6C-4 | S | Pending |
| **6C-6** | Migrate `lib/review/src/lib.rs` (1,313 LOC) review ops: prompt building (assemble system/user prompts for each dimension), response parsing (extract structured review output from LLM response), and any `Executable` trait impls. Prompt-building ops become DSL `fn` with string interpolation. Parse ops become DSL `fn` or generic `RestParseOp` parameterized by `ServiceOperationSpec`. | 6C-1 | L | Pending |
| **6C-7** | Migrate `lib/review/src/dimension.rs` (968 LOC) — the 4-dimension review model (coherence, quality, requirements, aspirational). Dimension definitions, scoring, and merge logic become `dsl/config/review_dimensions.dag` (config) + DSL `fn` (scoring/merge). | 6C-6 | M | Pending |
| **6C-8** | Migrate `lib/review/src/profile.rs` (449 LOC) — review profiles (dimension opt-in/out, severity thresholds, LLM model selection) become `dsl/config/review_profiles.dag`. | 6C-7 | S | Pending |
| **6C-9** | Full crate cleanup: delete all migrated behavior files from `lib/review/src/`. Only `lib.rs` re-exports, `system_models.rs`, and `generated_tests*.rs` should remain. Remove dead `mod` declarations. | 6C-5..6C-8 | S | Pending |

#### 6C Deletion Manifest

| File | LOC | Action |
|------|-----|--------|
| `lib/review/src/graph.rs` | 1,794 | Delete |
| `lib/review/src/graph_mock.rs` | 585 | Delete |
| `lib/review/src/lib.rs` | 1,313 | Shrink (only re-exports + `system_models` registration remain) |
| `lib/review/src/dimension.rs` | 968 | Delete (-> `dsl/config/review_dimensions.dag` + DSL `fn`) |
| `lib/review/src/profile.rs` | 449 | Delete (-> `dsl/config/review_profiles.dag`) |
| **Total** | **~5,109** | |

#### 6C Exit Criteria

1. Zero `graph.rs` or `graph_mock.rs` in `lib/review/src/`.
2. Zero `dimension.rs` or `profile.rs` in `lib/review/src/` — all review model definitions live in `dsl/config/review_dimensions.dag` and `dsl/config/review_profiles.dag`.
3. `lib/review/src/lib.rs` contains only re-exports and `system_models` registration — zero `Executable` impls, zero prompt-building logic.
4. `dsl/tools/review.dag` fully expresses all 4 review graph variants — no Rust `DagBuilder` calls for review topology anywhere in the codebase.
5. `gunbc-review` and `gunbc-pipeline` binaries execute using DSL-compiled review graphs.
6. All review-related tests pass, including entrypoint/boundary detection tests, dimension opt-out tests, and transport subdag assertions (equivalent assertions must exist in `.dag` test blocks or in the wrapper module's tests).
7. `DimensionGraphConfigOps` (currently private to `graph.rs`) is expressed as DSL config nodes — not moved to `ops.rs`.

#### 6C Verification

```
cargo check -p gunbc-lib-review
cargo test -q -p gunbc-lib-review
cargo check -p gunbc-dag
cargo test -q -p gunbc-dag
cargo run -q -p gunbc-dag --bin gunbc-review -- -n
cargo run -q -p gunbc-dag --bin gunbc-pipeline -- --help
cargo test -q -p gunbc-dag --test resource_registry_coverage
cargo clippy --all-targets -- -D warnings
```

---

#### Pure lib/ Ops Crates (tasks 6E-*)

**Scope**: `lib/git-ops/`, `lib/blob/`, `lib/gist-ops/`, `lib/markdown/`, `lib/design-ops/`

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6E-1** | Migrate `lib/git-ops/src/lib.rs` (1,014 LOC) — git prepare/parse ops to service operations in `dsl/services/git.dag`. Each prepare/parse pair maps to `ShellPrepareOp`/`ShellParseOp` parameterized by `ServiceOperationSpec`. | -- | M | Pending |
| **6E-2** | Migrate `lib/gist-ops/src/lib.rs` (1,317 LOC) — gist prepare/parse ops to service operations in `dsl/services/github/gist.dag`. REST prepare/parse pairs mapping to `RestPrepareOp`/`RestParseOp`. | -- | M | Pending |
| **6E-3** | Migrate `lib/blob/src/lib.rs` (942 LOC) — content acquisition ops to service operations in `dsl/services/blob.dag`. | -- | M | Pending |
| **6E-4** | Migrate `lib/markdown/src/lib.rs` (484 LOC) — markdown rendering ops to DSL `fn` or `uses rust_fn`. Pure transforms with no transport interaction. | -- | S | Pending |
| **6E-5** | Migrate `lib/design-ops/src/lib.rs` (141 LOC) — design review ops to DSL `fn`. Smallest crate, pure transforms. | -- | S | Pending |
| **6E-6** | Full crate cleanup: delete all migrated behavior files across all 5 crates. Only `lib.rs` re-exports, `system_models.rs`, and `generated_tests*.rs` should remain. | 6E-1..6E-5 | S | Pending |

| File | LOC | Action |
|------|-----|--------|
| `lib/git-ops/src/lib.rs` | 1,014 | Shrink |
| `lib/gist-ops/src/lib.rs` | 1,317 | Shrink |
| `lib/blob/src/lib.rs` | 942 | Shrink |
| `lib/markdown/src/lib.rs` | 484 | Shrink |
| `lib/design-ops/src/lib.rs` | 141 | Shrink |
| **Total** | **~3,898** | |

---

#### Clippy + Deps Tool Internals (tasks 6G-*)

**Scope**: `lib/tools/clippy/src/{config.rs, policy.rs, lint.rs}`, `lib/tools/deps/src/*.rs` (non-graph)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6G-1** | Migrate `lib/tools/clippy/src/` internals (config.rs, policy.rs, lint.rs — 932 LOC). Clippy config types and lint policy become DSL config in `dsl/config/clippy.dag`. Policy enforcement becomes DSL `fn`. | -- | M | Pending |
| **6G-2** | Migrate `lib/tools/deps/src/` internals (7 files — 1,456 LOC). Manifest parsing, installer selection, platform detection become service operations in `dsl/services/deps.dag` + DSL config. | -- | L | Pending |
| **6G-3** | Full crate cleanup: delete all migrated files. Only `lib.rs` re-exports, `system_models.rs`, and graph tombstones should remain. | 6G-1, 6G-2 | S | Pending |

| File | LOC | Action |
|------|-----|--------|
| `lib/tools/clippy/src/{config,policy,lint}.rs` | ~932 | Delete |
| `lib/tools/deps/src/{tool_upsert,installer,upsert,platform,manifest,env,package_manager}.rs` | ~1,456 | Delete |
| **Total** | **~2,388** | |

---

#### lib-Dependent Binary Deletion (tasks 6I-A)

After lib/ crate migration is complete, delete the 4 binaries that primarily consume lib/ APIs.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6I-3** | Delete `review.rs` (279 LOC) and `pipeline.rs` (411 LOC). After 6C, review graph construction is DSL-compiled. Remaining CLI ceremony migrates to `.dag`. Replace with generated entry points. | 6C | S | Pending |
| **6I-7** | Delete `infra.rs` (1,035 LOC). After 6B, all infra DAG builders and cloud-ops are DSL-compiled. Migrate multi-command dispatch to `.dag` pipeline with command routing. Replace with generated entry point. | 6B | L | Pending |
| **6I-9** | Delete `sdlc.rs` (3,942 LOC). Migrate ALL remaining orchestration to `.dag` pipelines: ledger management becomes service operations via file I/O transport; intent validation becomes DSL `fn`; command dispatch becomes per-command `.dag` pipelines; `CompiledStageDispatcher` is already DSL-compiled — extend to cover full binary. Replace with generated entry point. | 6B, 6E | XL | Pending |

| File | LOC | Action |
|------|-----|--------|
| `gunbc-dag/src/bin/review.rs` | 279 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/pipeline.rs` | 411 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/infra.rs` | 1,035 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/sdlc.rs` | 3,942 | Delete (-> generated entry point) |
| **Total** | **5,667** | |

#### Lane A Exit Criteria

1. Zero `Executable` trait impls in ANY `lib/` crate (except `lib/transport/` and `lib/primitives/`).
2. Zero behavior files in any `lib/` crate beyond re-exports and `system_models`.
3. All service operations use generic interpreters parameterized by `ServiceOperationSpec` from `dsl/services/*.dag`.
4. All config/policy definitions live in `dsl/config/*.dag`.
5. Each `lib/` crate's `lib.rs` contains only re-exports and `system_models` registration.
6. `review.rs`, `pipeline.rs`, `infra.rs`, `sdlc.rs` are trivially generated entry points (≤30 LOC each).
7. Zero ledger management, intent validation, or orchestration logic in any Lane A binary.

#### Lane A Verification

```
cargo check --workspace
cargo test --workspace --lib
cargo test -q -p gunbc-dag --test resource_registry_coverage
cargo run -q -p gunbc-dag --bin gunbc-review -- -n
cargo run -q -p gunbc-dag --bin gunbc-pipeline -- --help
cargo run -q -p gunbc-dag --bin gunbc-infra -- spec
cargo run -q -p gunbc-dag --bin gunbc-sdlc -- help
cargo clippy --all-targets -- -D warnings
```

---

### Lane B: gunbc-dag/ Behavior + Tool Binaries (~14K LOC)

**Goal**: Migrate ALL handwritten behavior in `gunbc-dag/src/` to `.dag`, then delete the tool binaries. Includes workspace subdags, tool ops, makegen/policy/config, workflow commands, and 11 tool binary entry points. After this lane, `gunbc-dag/src/` contains only DSL wrappers, `uses rust_fn` rendering stubs, Category D residuals (~1K LOC), and trivially generated entry points.

**Mutually exclusive scope**: ALL `gunbc-dag/src/` non-binary behavior files, plus `gunbc-dag/src/bin/{build,bootstrap,ci,codegen,codegen_cli,docgen,pragma,makegen,testgen,deps_config,workflow}.rs`.

**Sub-lanes**: 6A-2/3 (subdags), 6D (tool ops), 6F (makegen/policy/config), 6H (workflow), 6I-B (tool binaries)

---

#### Workspace Subdag Rewrites (tasks 6A-2, 6A-3)

**Scope**: `gunbc-dag/src/workspace/subdags/{bootstrap,makegen}.rs`

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6A-2** | Replace `gunbc-dag/src/workspace/subdags/bootstrap.rs` (174 LOC) with DSL wrapper: call `build_dsl_graph("tools/bootstrap.dag")` and wrap as `Node::subdag("bootstrap", dag)`. Delete the manual `DagBuilder` body. Follow the pattern in `gunbc-dag/src/bootstrap/graph.rs`. | -- | S | **Done** |
| **6A-3** | Replace `gunbc-dag/src/workspace/subdags/makegen.rs` (140 LOC) with DSL wrapper: call `build_dsl_graph("tools/makegen.dag")` and wrap as `Node::subdag("makegen", dag)`. Delete the manual `DagBuilder` body. | -- | S | **Done** |

**Explicitly kept in Rust**: `languages.rs` (124 LOC) — compile-time metadata composition over `LanguageOp`; no DSL module exists.

---

#### Tool Ops to DSL (tasks 6D-*)

**Goal**: Migrate `Executable` trait implementations for tool-level ops from hand-written Rust structs to generic interpreters parameterized by `ServiceOperationSpec` (for service ops) and DSL `fn` definitions (for pure renders and config transforms). Cloud/GCP ops are handled by Lane 6B.

**Status**: **ACTIVE** — the prerequisite design decision is resolved: Strategy B (generic interpreters over `ServiceOperationSpec`, SC1-SC7) is implemented. The existing `RestPrepareOp`/`RestParseOp`/`ShellPrepareOp`/`ShellParseOp` generic interpreters in `gunbc-dag/src/resolve_service.rs` are the target runtime.

**Mutually exclusive scope**: `gunbc-dag/src/{bootstrap,build,ci,codegen,docgen,makegen,pragma}/ops.rs`, `dsl/services/tools/`

**Scope** (4,030 LOC):
- `gunbc-dag/src/ci/ops.rs` (2,115)
- `gunbc-dag/src/docgen/ops.rs` (664)
- `gunbc-dag/src/codegen/ops.rs` (365)
- `gunbc-dag/src/build/ops.rs` (300)
- `gunbc-dag/src/makegen/ops.rs` (244)
- `gunbc-dag/src/bootstrap/ops.rs` (226)
- `gunbc-dag/src/pragma/ops.rs` (116)

**Op categories** (each op in the files above falls into exactly one):

| Category | Ops | Migration Strategy | Example |
|----------|-----|-------------------|---------|
| **A: Service prepare/parse** | 39 | Replace with generic `ShellPrepareOp`/`ShellParseOp` (or `FilePrepareOp`/`FileParseOp`) parameterized by `ServiceOperationSpec` extracted from `.dag` service definitions. Rust struct deleted entirely. | `BuildOp::PrepareBuild`, `CIOp::PrepareTestCommand` |
| **B: Pure render** | 7 | Keep underlying Rust render function. Register from DSL via `uses rust_fn`. Delete `Executable` enum wrapper. | `PragmaOp::RenderClippy`, `MakegenOp::RenderMakefile` |
| **C: Config constant** | 1 | Replace with DSL config node (`dsl/config/*.dag`). Rust struct deleted. | `MakegenOp::LoadRegistry` |
| **D: Complex domain logic** | 5 | Stays as Rust function behind `uses rust_fn`. Function body stays; `Executable` wrapper simplified. | `CIOp::Report`, `CodegenOp::ParseCodegenExists` |

#### 6D Op Audit (Complete)

52 ops across 7 files. Each op is categorized exactly once.

**`pragma/ops.rs`** — 3 ops, 116 LOC → **delete entire file**

| Op | Cat | Migration |
|----|-----|-----------|
| `PragmaOp::RenderClippy` | B | `uses rust_fn "policy::pragma::clippy_renderer"`. Underlying render fn stays in `crate::policy::pragma`. |
| `PragmaOp::RenderAllowlist` | B | `uses rust_fn "policy::pragma::render_disallowed_methods_allowlist"` |
| `PragmaOp::RenderLintPolicy` | B | `uses rust_fn "policy::pragma::render_pragma_lint_policy"` |

**`bootstrap/ops.rs`** — 4 ops, 226 LOC → **delete entire file**

| Op | Cat | Migration |
|----|-----|-----------|
| `BootstrapOp::PrepareScanWorkspace` | A | `ShellPrepareOp` spec: `find crates -maxdepth 1 -mindepth 1 -type d` |
| `BootstrapOp::ParseScanResult` | A | `ShellParseOp` spec: outputs `crate_count: int`, `crate_names: str_list`. Custom line-parsing (strips `crates/` prefix, sorts). Needs parse-script or `uses rust_fn` for the prefix-stripping logic. |
| `BootstrapOp::GenerateMakefile` | B | `uses rust_fn "makegen::render::render_makefile"` via `ToolRegistry::default_registry()` |
| `BootstrapOp::GenerateGitignore` | B | `uses rust_fn "makegen::gitignore::render_gitignore"` via `default_build_config()` |

**`makegen/ops.rs`** — 3 ops, 244 LOC → **delete or shrink to Entrypoint only**

| Op | Cat | Migration |
|----|-----|-----------|
| `MakegenOp::LoadRegistry` | C | DSL config node in `dsl/config/tool_registry.dag`. Serializes `ToolRegistry::default_registry()` + `iter_dag_specs()` as JSON. |
| `MakegenOp::RenderMakefile` | B | `uses rust_fn "makegen::render::render_makefile"` (same Rust fn as bootstrap's) |
| `MakegenOp::Entrypoint` | D | Inspects `__deps` list for `TransportResponse::File(Write, success)`. `__deps` is a DAG-level mechanism not expressible as a service spec. Stays as `uses rust_fn`. |

**`build/ops.rs`** — 7 ops, 300 LOC → **delete entire file**

| Op | Cat | Migration |
|----|-----|-----------|
| `BuildOp::PrepareBuild` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().build.to_shell_request()` |
| `BuildOp::ParseBuild` | A | `ShellParseOp` spec: outputs `build_success`, `build_stdout`, `build_stderr` |
| `BuildOp::PrepareTest` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().test`. Prereq: `build_success`. |
| `BuildOp::ParseTest` | A | `ShellParseOp` spec: outputs `test_success`, `test_skipped`, `test_stdout`, `test_stderr` |
| `BuildOp::PrepareClippy` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().lint`. Prereq: `build_success`. |
| `BuildOp::ParseClippy` | A | `ShellParseOp` spec: outputs `clippy_success`, `clippy_skipped`, `clippy_stdout`, `clippy_stderr` |
| `BuildOp::Summary` | B | Pure bool aggregation + string report. `uses rust_fn` or DSL `fn` with conditionals. |

**`codegen/ops.rs`** — 5 ops, 365 LOC → **shrink to ParseCodegenExists only**

| Op | Cat | Migration |
|----|-----|-----------|
| `CodegenOp::PrepareCodegenExists` | A | `FilePrepareOp` spec: `FileRequest::glob("target/codegen/bin/**/main.rs")` |
| `CodegenOp::ParseCodegenExists` | D | **Design issue**: calls `TransportIo::new()` + `load_manifest_default()` — hidden I/O inside "pure" op. Must stay as `uses rust_fn` until manifest loading is extracted to a separate transport node. |
| `CodegenOp::PrepareCodegenCommand` | A | `ShellPrepareOp` spec: `cargo run -p gunbc-dag --bin gunbc-codegen -- codegen`. Prereq: `codegen_needed`. |
| `CodegenOp::ParseCodegenResult` | A | `ShellParseOp` spec: outputs `prep_success`, `codegen_ran`, `prep_message`. Skip-propagation on both `skip` and `response`. |
| `CodegenOp::PrepareStampWrite` | A | `FilePrepareOp` spec: `FileRequest::write(codegen_stamp_path(), "codegen ok\n")`. Prereq: `prep_success`. |

**`docgen/ops.rs`** — 3 ops, 664 LOC → **shrink to RenderAbWorkflowsDoc only**

| Op | Cat | Migration |
|----|-----|-----------|
| `DocgenOp::PrepareFileRead { path }` | A | `FilePrepareOp` spec: `FileRequest::read(path)`. Parameterized — one spec instance per path in the DAG. |
| `DocgenOp::ParseFileContent { path, allow_missing }` | A | `FileParseOp` spec: outputs `content: str`. Parameterized by `path` and `allow_missing`. |
| `DocgenOp::RenderAbWorkflowsDoc` | D | 500+ lines of template rendering (section replacement, code extraction, test collection). Stays as `uses rust_fn`. Takes 13 string inputs, produces `content` + `path`. |

**`ci/ops.rs`** — 27 ops, 2,115 LOC → **shrink to AggregateVerifyResults + Report only**

| Op | Cat | Migration |
|----|-----|-----------|
| `CIOp::ParseDepsExists` | A | `FileParseOp` spec: parses `Exists` response → `deps_exists`, `deps_checked`, `deps_installed`, `message` + status. |
| `CIOp::PrepareTestgenCommand` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().testgen`. Prereq: `prep_success`. |
| `CIOp::ParseTestgenResult` | A | `ShellParseOp` spec: outputs `testgen_success`, `testgen_stderr`, `testgen_stdout`. |
| `CIOp::PrepareBootstrapCommand` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().bootstrap`. Prereq: `prep_success`. |
| `CIOp::ParseBootstrapResult` | A | `ShellParseOp` spec: outputs `bootstrap_success`, `bootstrap_stderr`, `bootstrap_stdout`. |
| `CIOp::PreparePragmaCommand` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().pragma`. Prereq: `prep_success`. |
| `CIOp::ParsePragmaResult` | A | `ShellParseOp` spec: outputs `pragma_success`, `pragma_stderr`, `pragma_stdout`. |
| `CIOp::PrepareBuildCommand` | A | `ShellPrepareOp` spec: `cargo test --no-run` with `-D warnings` RUSTFLAGS. Prereqs: `prep_success`, `testgen_success`. |
| `CIOp::ParseBuildResult` | A | `ShellParseOp` spec: outputs `build_success`, `build_skipped`, `build_stdout`, `build_stderr`. |
| `CIOp::PrepareTestCommand` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().test`. Prereq: `build_success`. |
| `CIOp::ParseTestResult` | A | `ShellParseOp` spec: outputs `test_success`, `test_skipped`, `test_stdout`, `test_stderr`. |
| `CIOp::PrepareClippyLint` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().lint`. Prereqs: `build_success`, `pragma_success`. |
| `CIOp::ParseClippyLintResult` | A | `ShellParseOp` spec: outputs `lint_success`, `lint_skipped`, `lint_stdout`, `lint_stderr`. |
| `CIOp::PrepareGuardrailCheck` | A | `ShellPrepareOp` spec: `bash -lc "cargo test -p gunbc-dag --test resource_purity_checks --quiet"`. Prereqs: `testgen_success`, `pragma_success`. |
| `CIOp::ParseGuardrailResult` | A | `ShellParseOp` spec: outputs `guardrail_success`, `guardrail_stderr`, `guardrail_stdout`. |
| `CIOp::PrepareVerifyMakegenCheck` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().makegen_check`. Uses shared `verify_skip_reason()`. |
| `CIOp::ParseVerifyMakegenResult` | A | `ShellParseOp` spec: outputs `verify_makegen_success`, `verify_makegen_stderr`, `verify_makegen_stdout`. |
| `CIOp::PrepareVerifyDepsConfigCheck` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().deps_config_check`. |
| `CIOp::ParseVerifyDepsConfigResult` | A | `ShellParseOp` spec: outputs `verify_deps_config_*`. |
| `CIOp::PrepareVerifyBootstrapCheck` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().bootstrap_check`. |
| `CIOp::ParseVerifyBootstrapResult` | A | `ShellParseOp` spec: outputs `verify_bootstrap_*`. |
| `CIOp::PrepareVerifyTestgenCheck` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().testgen_check`. |
| `CIOp::ParseVerifyTestgenResult` | A | `ShellParseOp` spec: outputs `verify_testgen_*`. |
| `CIOp::PrepareVerifyPragmaCheck` | A | `ShellPrepareOp` spec: `BuildConfig::cargo().pragma_check`. |
| `CIOp::ParseVerifyPragmaResult` | A | `ShellParseOp` spec: outputs `verify_pragma_*`. |
| `CIOp::AggregateVerifyResults` | D | Iterates 5 verify results, aggregates into `verify_success` + structured `status()` output. `uses rust_fn`. |
| `CIOp::Report` | D | Aggregates 8 stage results with specialized extractors (`extract_build_errors`, `extract_lint_warnings`, `extract_test_failures`, `extract_verify_failures`) + structured rendering + truncation. `uses rust_fn`. |

#### 6D Audit Summary

| Category | Ops | Approx LOC Deleted | Migration Path |
|----------|-----|-------------------|----------------|
| **A: Service prepare/parse** | 39 | ~2,600 | `ServiceOperationSpec` in `dsl/services/tools/*.dag` → generic interpreters. Rust struct deleted entirely. |
| **B: Pure render** | 7 | ~350 | `uses rust_fn` pointing to existing render functions in policy/render modules. `Executable` enum deleted. |
| **C: Config constant** | 1 | ~50 | DSL config node in `dsl/config/*.dag`. Rust struct deleted. |
| **D: Complex domain** | 5 | ~1,030 (stays) | `uses rust_fn` with typed signatures. Function body stays in Rust. `Executable` enum simplified. |
| **Total** | **52** | **~3,000 deleted** | |

**Design notes for implementer**:
- All 39 Category A ops follow two patterns: (1) `ShellPrepareOp` — reads `BuildConfig::cargo()` fields + checks prerequisite bools → produces `TransportRequest` + `skip`; (2) `ShellParseOp`/`FileParseOp` — handles skip-propagation + extracts success/stdout/stderr from response. The generic interpreters need `prerequisites` and `skip_defaults` fields in `ServiceOperationSpec`.
- `BootstrapOp::ParseScanResult` (Cat A) has custom line-parsing logic (strip `crates/` prefix, sort). May need a parse-script field in the spec or a small `uses rust_fn` for the transform.
- `CodegenOp::ParseCodegenExists` (Cat D) has hidden I/O (`TransportIo::new()` + `load_manifest_default`). Ideally this would be refactored to load the manifest via a transport node, making it a pure parse op (A). But that's an optional improvement; it works as `uses rust_fn` today.
- `MakegenOp::Entrypoint` (Cat D) inspects `__deps` — a DAG-level mechanism. Not expressible as a service spec.
- The 5 Category D ops total ~1,030 LOC but their function bodies stay in Rust. The deleted code is only the `Executable` enum boilerplate and `Mockable` impls.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6D-1** | ~~Audit and categorize every `Executable` impl~~ | -- | -- | **Done** (see audit above) |
| **6D-2** | Migrate Category B ops (7 ops): register underlying Rust render functions from DSL via `uses rust_fn`. Covers: `PragmaOp::Render{Clippy,Allowlist,LintPolicy}`, `BootstrapOp::Generate{Makefile,Gitignore}`, `MakegenOp::RenderMakefile`, `BuildOp::Summary`. Delete the `Executable` enum variants and `Mockable` impls. Verify the DSL-registered functions produce identical output via existing tests. | -- | M | Pending |
| **6D-3** | Migrate Category C ops (1 op): `MakegenOp::LoadRegistry`. Author `dsl/config/tool_registry.dag` as the authoritative source for tool names, testgen targets, and registry JSON. Delete the Rust struct. | -- | S | Pending |
| **6D-4** | Migrate Category A ops in `bootstrap/ops.rs` (2 ops) and `build/ops.rs` (6 ops). Author `dsl/services/tools/bootstrap.dag` with shell specs for `find crates ...` and `dsl/services/tools/build.dag` with specs for `cargo build/test/clippy`. Each spec declares the command, args, env, prerequisites, and output keys. Delete per-op Rust structs. | -- | M | Pending |
| **6D-5** | Migrate Category A ops in `ci/ops.rs` (25 ops — largest batch). Author `dsl/services/tools/ci.dag` with shell specs for testgen, bootstrap, pragma, build, test, clippy, guardrails, and 5 verify checks. The generic `ShellPrepareOp`/`ShellParseOp` handles skip-propagation + prerequisite gating for all 25. Incrementally delete Rust structs as each spec is wired. | -- | L | Pending |
| **6D-6** | Migrate Category A ops in `codegen/ops.rs` (4 ops) and `docgen/ops.rs` (2 ops). Author `dsl/services/tools/codegen.dag` with file-glob + shell + file-write specs, and `dsl/services/tools/docgen.dag` with parameterized file-read specs (one per source file path). Delete per-op Rust structs. | -- | M | Pending |
| **6D-7** | Register Category D ops (5 ops) from DSL via `uses rust_fn`: `CodegenOp::ParseCodegenExists`, `DocgenOp::RenderAbWorkflowsDoc`, `MakegenOp::Entrypoint`, `CIOp::AggregateVerifyResults`, `CIOp::Report`. Ensure typed input/output signatures match. The function bodies stay in Rust (move from ops.rs to dedicated modules if needed). | 6D-2..6D-6 | S | Pending |
| **6D-8** | Final cleanup: delete emptied ops.rs files (`pragma`, `bootstrap`, `build` — fully Category A/B). Shrink remaining ops.rs files to only Category D registrations (`codegen`, `docgen`, `makegen`, `ci`). Remove dead `mod` declarations, unused imports, and orphaned `Mockable` impls. | 6D-2..6D-7 | S | Pending |

#### 6D Deletion Manifest

| File | LOC | Ops | Action |
|------|-----|-----|--------|
| `gunbc-dag/src/pragma/ops.rs` | 116 | 3B | **Delete entirely** — all 3 ops are Category B renders |
| `gunbc-dag/src/bootstrap/ops.rs` | 226 | 2A, 2B | **Delete entirely** — 2 service ops + 2 renders, no D residuals |
| `gunbc-dag/src/build/ops.rs` | 300 | 6A, 1B | **Delete entirely** — 6 service ops + 1 aggregator, no D residuals |
| `gunbc-dag/src/makegen/ops.rs` | 244 | 1B, 1C, 1D | **Shrink** — delete LoadRegistry (C) + RenderMakefile (B), keep Entrypoint (D, ~30 LOC) |
| `gunbc-dag/src/codegen/ops.rs` | 365 | 4A, 1D | **Shrink** — delete 4 service ops, keep ParseCodegenExists (D, ~90 LOC) |
| `gunbc-dag/src/docgen/ops.rs` | 664 | 2A, 1D | **Shrink** — delete 2 file-read service ops, keep RenderAbWorkflowsDoc (D, ~540 LOC) |
| `gunbc-dag/src/ci/ops.rs` | 2,115 | 25A, 2D | **Shrink** — delete 25 service ops + tests + mocks, keep AggregateVerifyResults + Report (D, ~400 LOC) |
| **Total** | **4,030** | **39A, 7B, 1C, 5D** | **~3,000 LOC deleted**, ~1,030 LOC stays as `uses rust_fn` |

#### 6D Exit Criteria

1. Every `Executable` impl in the 7 ops files is categorized and migrated per its category.
2. Zero Category A (service prepare/parse) ops remain as hand-written Rust — all use generic interpreters parameterized by `ServiceOperationSpec` from `dsl/services/tools/*.dag`.
3. Zero Category B (pure render) ops remain as hand-written Rust — all are DSL `fn` definitions.
4. Zero Category C (config constant) ops remain — all are DSL config nodes.
5. Category D ops (if any) are registered from DSL via `uses rust_fn` with explicit typed signatures — no `DagBuilder` topology in Rust.
6. All tool binaries (`gunbc-bootstrap`, `gunbc-build`, `gunbc-ci`, `gunbc-codegen`, `gunbc-docgen`, `gunbc-makegen`, `gunbc-pragma`) produce identical output before and after migration (verified by dry-run comparison).

#### 6D Verification

```
cargo check -p gunbc-dag
cargo test -q -p gunbc-dag
cargo test -q -p gunbc-dag --test resource_registry_coverage
cargo run -q -p gunbc-dag --bin gunbc-bootstrap -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-build -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-ci -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-testgen -- --dry-run
cargo clippy --all-targets -- -D warnings
```

---

#### makegen Behavior + Policy + Config (tasks 6F-*)

**Goal**: Migrate the tool registry, Makefile/Justfile/gitignore/CI rendering inputs, pragma policy definitions, resource definitions, and binary invocation mappings from handwritten Rust to `.dag`. This is the most impactful lane — `registry.rs` (2,355 LOC) is the single source of truth for all build/test/lint commands. Migrating it to DSL means `dsl/config/tool_registry.dag` becomes authoritative, eliminating Rust<->DSL synchronization bugs.

**Mutually exclusive scope**: `gunbc-dag/src/makegen/{registry.rs, render.rs, gitignore.rs, justfile.rs, ci_render.rs}`, `gunbc-dag/src/policy/pragma.rs`, `gunbc-dag/src/resources.rs`, `gunbc-dag/src/binaries.rs`, `dsl/config/tool_registry.dag`, `dsl/config/pragma_policy.dag`, `dsl/config/resources.dag`, `dsl/config/binaries.dag`, `dsl/config/gitignore_categories.dag`

**Parallel with**: 6A, 6B, 6C, 6E, 6G (no file overlap with 6D — 6D owns only `*/ops.rs`)

**Key dependency**: 6D's `dsl/services/tools/*.dag` should reference command specs from 6F's `dsl/config/tool_registry.dag` rather than hardcoding shell commands. 6F establishing the DSL config first makes 6D cleaner.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6F-1** | Migrate `makegen/registry.rs` (2,355 LOC) — `BuildConfig` + `ToolRegistry` to `dsl/config/tool_registry.dag` (currently an 18-line stub). This is the foundation: every tool binary name, test target, lint command, and cargo invocation becomes DSL-authoritative. Must include `iter_dag_specs()` output for testgen/CI. | -- | XL | Pending |
| **6F-2** | Migrate `policy/pragma.rs` (411 LOC) — pragma policy definitions (clippy config, lint policy, disallowed methods) to expand `dsl/config/pragma_policy.dag` (currently a 28-line stub). Policy types, severity levels, and allowlist entries become DSL config. | -- | M | Pending |
| **6F-3** | Migrate `resources.rs` (328 LOC) — `ResourceDef` constants (Makefile, .gitignore output paths, freshness markers) to `dsl/config/resources.dag`. | -- | S | Pending |
| **6F-4** | Migrate `binaries.rs` (204 LOC) — binary invocation mappings (tool name -> cargo binary name, args) to `dsl/config/binaries.dag`. | -- | S | Pending |
| **6F-5** | Migrate rendering inputs: `render.rs` (1,104 LOC), `gitignore.rs` (351 LOC), `justfile.rs` (395 LOC), `ci_render.rs` (192 LOC). Rendering functions stay as `uses rust_fn` but their config inputs (which tool targets to include, which sections to render, CI matrix entries) come from DSL config authored in 6F-1..6F-4. Delete the hardcoded Rust config; keep only the rendering logic behind `uses rust_fn`. | 6F-1..6F-4 | L | Pending |
| **6F-6** | Final cleanup: delete `registry.rs`, `policy/pragma.rs`, `resources.rs`, `binaries.rs`. Shrink rendering files to pure `uses rust_fn` stubs. Remove dead `mod` declarations. | 6F-5 | M | Pending |

#### 6F Deletion Manifest

| File | LOC | Action |
|------|-----|--------|
| `gunbc-dag/src/makegen/registry.rs` | 2,355 | Delete (-> `dsl/config/tool_registry.dag`) |
| `gunbc-dag/src/policy/pragma.rs` | 411 | Delete (-> `dsl/config/pragma_policy.dag`) |
| `gunbc-dag/src/resources.rs` | 328 | Delete (-> `dsl/config/resources.dag`) |
| `gunbc-dag/src/binaries.rs` | 204 | Delete (-> `dsl/config/binaries.dag`) |
| `gunbc-dag/src/makegen/render.rs` | 1,104 | Shrink (keep rendering fn behind `uses rust_fn`, delete config) |
| `gunbc-dag/src/makegen/gitignore.rs` | 351 | Shrink (keep rendering fn, delete config) |
| `gunbc-dag/src/makegen/justfile.rs` | 395 | Shrink (keep rendering fn, delete config) |
| `gunbc-dag/src/makegen/ci_render.rs` | 192 | Shrink (keep rendering fn, delete config) |
| **Total** | **~5,340** | |

#### 6F Exit Criteria

1. Zero `registry.rs`, `pragma.rs`, `resources.rs`, or `binaries.rs` in their current locations — all config/policy lives in `dsl/config/*.dag`.
2. `dsl/config/tool_registry.dag` is the single source of truth for all tool names, commands, and test targets — no `BuildConfig` or `ToolRegistry` Rust structs.
3. Rendering files (`render.rs`, `gitignore.rs`, `justfile.rs`, `ci_render.rs`) contain only `uses rust_fn` rendering logic — zero hardcoded config.
4. `dsl/config/pragma_policy.dag` is authoritative for all pragma policy.
5. All tool binaries still produce identical output (verified by dry-run comparison).

#### 6F Verification

```
cargo check -p gunbc-dag
cargo test -q -p gunbc-dag
cargo test -q -p gunbc-dag --test resource_registry_coverage
cargo run -q -p gunbc-dag --bin gunbc-makegen -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-pragma -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-bootstrap -- --dry-run
cargo clippy --all-targets -- -D warnings
```

#### Workflow Command Mappings (tasks 6H-*)

**Goal**: Migrate workflow unit-to-command mappings and SLO budget definitions from handwritten Rust to `.dag`. After this lane, the workflow planner consumes DSL-authored command specs and SLO budgets — zero hardcoded Rust config.

**Mutually exclusive scope**: `gunbc-dag/src/workflow/unit_commands.rs`, `gunbc-dag/src/workflow/slo.rs`, `dsl/config/slo.dag`, `dsl/config/workflow_commands.dag`

**Depends on**: 6F (command specs must be in DSL first — `unit_commands.rs` references tool names from `registry.rs`)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6H-1** | Migrate `unit_commands.rs` (628 LOC) — workflow unit-to-shell-command mappings. These map process unit names to the shell commands that execute them. After 6F, tool command specs live in `dsl/config/tool_registry.dag`; `unit_commands.rs` becomes a DSL module that references those specs. Author `dsl/config/workflow_commands.dag`. | 6F-1 | M | Pending |
| **6H-2** | Migrate `slo.rs` (303 LOC) — latency SLO budgets per workflow unit. Budget definitions become `dsl/config/slo.dag`. | -- | S | Pending |
| **6H-3** | Cleanup: delete `unit_commands.rs` and `slo.rs`. Remove `mod` declarations from parent. | 6H-1, 6H-2 | S | Pending |

#### 6H Deletion Manifest

| File | LOC | Action |
|------|-----|--------|
| `gunbc-dag/src/workflow/unit_commands.rs` | 628 | Delete (-> `dsl/config/workflow_commands.dag`) |
| `gunbc-dag/src/workflow/slo.rs` | 303 | Delete (-> `dsl/config/slo.dag`) |
| **Total** | **931** | |

#### 6H Exit Criteria

1. Zero `unit_commands.rs` or `slo.rs` in `gunbc-dag/src/workflow/`.
2. `dsl/config/workflow_commands.dag` is authoritative for unit-to-command mappings.
3. `dsl/config/slo.dag` is authoritative for SLO budgets.
4. Workflow planner produces identical plans before and after migration.

#### 6H Verification

```
cargo check -p gunbc-dag
cargo test -q -p gunbc-dag -- workflow
cargo run -q -p gunbc-dag --bin gunbc-workflow -- --plan ci --dry-run
cargo clippy --all-targets -- -D warnings
```

---

#### Tool Binary Deletion (tasks 6I-B)

After gunbc-dag/ behavior migration is complete, delete the 11 binaries that primarily consume tool DAGs.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **6I-1** | Delete thin binaries: `build.rs` (83), `codegen.rs` (112), `docgen.rs` (170), `pragma.rs` (219), `testgen.rs` (285). Replace each with a trivially generated entry point. | 6D, 6F | M | Pending |
| **6I-2** | Delete `bootstrap.rs` (380) and `deps_config.rs` (209). Migrate resource manifest ceremony, freshness logic to `.dag`. Replace with generated entry points. | 6D, 6F | M | Pending |
| **6I-4** | Delete `makegen.rs` (420). After 6F, registry loading and config are DSL-authoritative. Replace with generated entry point. | 6F | S | Pending |
| **6I-5** | Delete `ci.rs` (210). Bootstrap-safe constraint preserved — generated entry point compiles without generated sources. | 6D | S | Pending |
| **6I-6** | Delete `workflow.rs` (708). After 6H, command mappings and SLOs are DSL-authoritative. Migrate planner to `.dag`. Replace with generated entry point. | 6H | M | Pending |
| **6I-8** | Delete `codegen_cli.rs` (2,051). Migrate transaction-based code generation orchestration to `.dag` pipeline. Replace with generated entry point. | 6D, 6F | L | Pending |

| File | LOC | Action |
|------|-----|--------|
| `gunbc-dag/src/bin/build.rs` | 83 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/codegen.rs` | 112 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/docgen.rs` | 170 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/pragma.rs` | 219 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/ci.rs` | 210 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/deps_config.rs` | 209 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/testgen.rs` | 285 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/bootstrap.rs` | 380 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/makegen.rs` | 420 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/workflow.rs` | 708 | Delete (-> generated entry point) |
| `gunbc-dag/src/bin/codegen_cli.rs` | 2,051 | Delete (-> generated entry point) |
| **Total** | **4,847** | |

#### Lane B Exit Criteria

1. Zero manual `DagBuilder` calls in `gunbc-dag/src/workspace/subdags/`.
2. Every `Executable` impl in the 7 tool ops files is categorized and migrated per its category (see 6D audit).
3. Zero Category A/B/C ops remain as hand-written Rust. Category D ops (~1K LOC) stay as `uses rust_fn`.
4. Zero `registry.rs`, `policy/pragma.rs`, `resources.rs`, or `binaries.rs` — all config/policy in `dsl/config/*.dag`.
5. Zero `unit_commands.rs` or `slo.rs` — workflow config in `dsl/config/*.dag`.
6. 11 tool binaries are trivially generated entry points (≤30 LOC each).
7. All tool binaries produce identical output before and after migration (dry-run comparison).

#### Lane B Verification

```
cargo check -p gunbc-dag
cargo test -q -p gunbc-dag
cargo test -q -p gunbc-dag --test resource_registry_coverage
cargo run -q -p gunbc-dag --bin gunbc-bootstrap -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-build -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-ci -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-makegen -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-pragma -- --dry-run
cargo run -q -p gunbc-dag --bin gunbc-workflow -- --plan ci --dry-run
cargo clippy --all-targets -- -D warnings
```

---

### Lane 6 Summary

| Lane | Scope | Sub-lanes | LOC | Parallel |
|------|-------|-----------|-----|----------|
| **A** | All `lib/` crate behavior + 4 lib-dependent binaries | 6A-1, 6B, 6C, 6E, 6G, 6I-A | ~35,200 | A ∥ B |
| **B** | All `gunbc-dag/src/` behavior + 11 tool binaries | 6A-2/3, 6D, 6F, 6H, 6I-B | ~14,400 | A ∥ B |

**Total**: ~47,000 LOC of Rust behavior deleted or replaced by `.dag` files. ~1,000 LOC of Category D residuals stay as `uses rust_fn`.

**Mutual exclusivity**: Lane A owns all `lib/` files + `bin/{review,pipeline,infra,sdlc}.rs`. Lane B owns all `gunbc-dag/src/` non-binary files + `bin/{build,bootstrap,ci,codegen,codegen_cli,docgen,pragma,makegen,testgen,deps_config,workflow}.rs`. Zero overlap.

---

## Design Decision Status

All design decisions are resolved. Full table preserved for reference.

<details>
<summary>Expand design decisions (reference only)</summary>

| Decision | Status | Notes |
|---|---|---|
| Backend semantics encoded in IR | Resolved (done) | Applied in `R3`-`R6`. |
| External system semantics typed | Resolved (done) | Applied in `R7`-`R12`. |
| DeferredCallableOp elimination strategy | Resolved (done) | Implemented in `P6`/`P12`. |
| Runtime environment | Resolved | Local-first CLI, env creds + CI/cloud WIF path. |
| Abstract review model | Resolved | Four-dimension typed model with criteria-driven opt-in. |
| Workflow minimum unit + exclusive coordination | Resolved (done) | Canonicalized in WF design docs (`WF1-D`..`WF4-D`). |
| Control-token model | Resolved (done) | Keep completion-gated control; require explicit success guards for fail-fast functional paths. |
| Cached `result` persistence | Resolved (done) | Persist typed summary/reference by default; optional full payload in CAS. |
| Changed-input routing authority | Resolved (done) | Optimization hint only; non-authoritative for soundness. |
| Conflict commutativity exceptions | Resolved (done) | No commutativity exceptions in current phase. |
| Service codegen strategy | Resolved (done) | Strategy B implemented: generic interpreters over `ServiceOperationSpec` (SC1-SC3). |
| DSL as source of truth for services | Resolved (done) | `.dag` service definitions replace hand-written IR transport types (SC4-SC7). |
| Artifact dependency direction | Resolved (done) | Codegen outputs are compilation inputs. |
| Two-phase compilation | Resolved (done) | Bootstrap-safe binaries compiled without generated sources. |
| Daggen status | Deferred | `needs_daggen()` returns false. Workflow DAGs remain hand-authored in Rust. |
| SDLC pipeline architecture | Resolved | Issue-centric lifecycle with provider-agnostic types. |
| SDLC intake/idempotency-first rollout | Resolved | Intake + idempotency contracts are Phase 0 gates before stage automation. |
| SDLC runtime launch + infra control-plane model | Resolved (done) | Lane E complete: stateless worker topology, infra plan/apply, preflight gates, drain semantics. |
| SDLC codegen-first objective | Resolved (done) | Lane F complete: DSL-authored behavior compiled to Rust/Go/C, multi-level conformance harness. |
| SDLC mega modeling gate | Resolved (done) | `MD0-D` approved; all downstream lanes delivered. |
| Three-layer domain abstraction | Resolved | Pipeline sees domain concepts (Issue, Claim, Outcome); domain interfaces are provider-fungible; infra implementations selected by deployment profile at compile time. |
| Compile-time profile binding | Resolved (done) | `profile { bind Interface -> Impl }` syntax in DSL. Compiler resolves `uses` declarations via active profile. `--profile` CLI flag. |
| Dry-run deployment readiness | Resolved (done) | Rust worker multi-stage dispatch supports local dry-run progression through terminal `closed` state. |
| Dual execution path convergence | Resolved (done) | Compiled DAG path is now primary. Worker loads `CompiledStageDispatcher` and dispatches via profile-resolved pipeline. |

</details>

---

## SDLC Design Checklist (Must Hold) -- All Satisfied

All 27 design contracts are implemented and tested. Owner tasks are archived.

<details>
<summary>Expand checklist (reference only)</summary>

| Topic | Required Contract | Owner Tasks |
|---|---|---|
| Intent identity | `intent_id` is stable and uniquely maps to one remote issue (`issue_id`). | `IM1`, `IM2` |
| Intake idempotency | Re-running intake with same `intent_id` performs update, not create. | `IM2` |
| Stage idempotency key | `run_key = hash(issue_id, stage, input_hash, policy_version)` gates all stage side effects. | `IM3`, `IM13`, `W11` |
| Remote update protocol | Comments/artifacts upserted by deterministic marker; labels/stage transitions are compare-and-set. | `IM4`, `IM8`, `IM13`, `W9`, `W12` |
| Commit/update traceability | Branch + commit metadata link code changes back to `issue_id`, `intent_id`, and `run_key`. | `IM5`, `W12` |
| Resume safety | Rerun from crash/restart resumes from ledger without repeating side effects. | `IM3`, `W13` |
| Provider fungibility | Provider-specific fields stay in adapter boundary; pipeline depends only on abstract contracts. | `IM0-D`, `W9`, `W11` |
| Atomic pickup | At most one worker owns `(issue_id, stage)` via lease/CAS claim protocol. | `IM6`, `IM7`, `W12` |
| Transaction safety | Stage side effects follow fixed ordering and are retry-safe at each step. | `IM8`, `W11`, `W12` |
| Intake conflict safety | Intent -> issue mapping is deterministic and multi-match conflicts fail closed. | `IM10`, `W9` |
| Failure handling determinism | Retry behavior is typed with persisted retry state, never memory-only. | `IM9`, `IM7`, `W12` |
| Recovery reconciliation | Crash windows reconcile deterministically. | `IM11`, `W12` |
| AwaitApproval yield contract | AwaitApproval is asynchronous yield: persist, release claim, resume via rediscovery. | `W13`, `W12` |
| Fail-closed terminalization | Fail-closed paths persist terminal failure, publish status, release claim. | `IM9`, `IM10`, `IM11`, `W12` |
| Provider capability gating | Real mode blocked unless adapter passes capability contracts. | `IM12`, `W9`, `W12` |
| Runtime launch topology | SDLC workers run stateless with externalized state. | `IN0-D`, `IN4` |
| Signal reliability contract | Triggers are durable at-least-once with dedup keys and anti-entropy. | `IN0-D`, `IM7`, `W12` |
| Local-first rollout parity | Local loop validates business logic first; infra split preserves semantics. | `IN0-D`, `IN4`, `W12` |
| Infra bringup intent | Runtime infra desired state modeled as versioned/idempotent intent input. | `IN1`, `IN2` |
| Startup preflight gate | Worker real mode blocked unless infra prereqs are healthy. | `IN3` |
| DSL source of truth | SDLC behavior authored in canonical `dsl/` modules. | `CG0-D`, `CG1`, `CG2` |
| Codegen target parity | Generated Rust/Go/C artifacts satisfy shared conformance tests. | `CG5`, `CG6` |
| C backend memory ownership | C/runtime adapter uses explicit acquire/release ownership handles. | `CG5`, `CG6` |
| Interpreter role boundary | Rust interpreter supported but non-primary; new features land in DSL/codegen first. | `CG0-D`, `CG6` |
| Artifact storage fungibility | Artifact updates support inline and blob-ref under one idempotent marker contract. | `IM4`, `CG3` |
| Canonical modeling gate | SDLC tasks downstream of mega-modeling design sign-off. | `MD0-D` |

</details>

---

## Archive Update Log

Moved to `TODO/TODONE/2026-Q1/tasks-completed.md`:

- **2026-02-19**: Sprint 1, Sprint 2, Sprint 3
- **2026-02-20**: Lane 3 (all): `M8-D`..`M14`, `M16-D`..`M19`; Security/install: `M7-D`, `M7`, `M15-D`, `M15`
- **2026-02-22 (batch 1)**: `WF6`-`WF9`, `WF14`-`WF18`, `DL1`-`DL8`, `W1`-`W14`, Lane A-H (all), Sprint 10-11.5 (all), Cleanup (all), `CU-1`/`CU-3`-`CU-6`, `TS-2`/`TS-3`/`TS-5`/`TS-6`, `L2-1`/`L2-2`, `S12-5`-`S12-8`
- **2026-02-22 (batch 2 — full lane audit)**: Lane 1 (all): `TS-1`/`TS-1b`/`TS-1c`/`TS-1d`; Lane 2 (all): `L2-0`/`L2-3`/`L2-4`, `S12-1`-`S12-4`, `S12-9`-`S12-19`; Post-merge (all): `TS-4`/`TS-7`; Lane 4 (all): `CU-2`/`CU-7`-`CU-9`; Lane 5 (partial): `GD-1`-`GD-4`/`GD-6`
- **2026-02-22 (batch 3 — GD-5 closeout)**: Lane 5 completion: `GD-5`; section 9D policy finalized (drop-now + migrate-in-place), cloud helper stacks deleted (`infra_graph.rs`, `secret_provision_graph.rs`), and targeted verification pass recorded

---

## Horizon: Forward-Looking Design (Unscheduled)

Design docs exist in `docs/design/horizon/`. Speculative features — promote to a lane when prioritized.

| ID | Design Doc | Summary | Size |
|----|-----------|---------|------|
| **H1** | `h1-display-reactive-dsl.md` | Channel-driven event loop with `on`/`tick` triggers for display orchestration | XL |
| **H10** | `h10-compute-stack-services.md` | Cloud Run/GCS/LB provision/apply orchestration | L |
| **H12** | `h12-process-readiness-test-gate.md` | Process-level readiness gate: require fast tests to pass before side-effecting execution | M |

---

## Backlog (Feature Ideas -- Not Scheduled)

See `TODO/backlog.md` for details. Parked for future consideration:

- Display Reactive DSL (XL) -- requires new DSL infra
- Compute Stack Provision/Apply (L) -- service layer works, orchestration is XL
- Glob-aware Resource Admission (M) -- policy-sensitive concurrency, needs explicit design

---

## Lane 7: Review Cleanup

**Goal**: Eliminate fallbacks, stringly-typed hacks, and manually-maintained escape hatches identified during code review of the GraphIR-to-DSL migration diff.

**Source**: Review feedback (2026-02-22/23). Each task ID is referenced in code (e.g., `// RV-3`, `See \`RV-1\``).

**Parallelism**: Fully parallel with Lanes 6A–6C (disjoint file sets, no blocking deps). `RV-6` depends on `RV-1`.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **RV-1** | **Move `wire_missing_filesystem_resources` to lowering phase.** Currently a resolve-time fallback that auto-wires unconnected `FilesystemHandle` ports. Move to lowering (like `add_resource_lifecycle_nodes`) and make missing resource edges a compile error. | -- | M | Pending |
| **RV-2** | **Add `handler_hint` field to `LoweredOp::Callable`.** Replace module-name and string-prefix heuristics in `classify_handler` with an explicit `handler_hint: Option<HandlerHint>` produced during lowering from DSL annotations/obligation metadata. ~114 construction sites. | -- | L | Pending |
| **RV-3** | **Migrate credential wiring to `CredentialIntent` pipeline.** `wire_profile_credentials` uses `std::env::set_var` for `GITHUB_TOKEN`/`CODEX_API_KEY` behind a `Once` guard. Replace with the memory-safe `Credential` capability pipeline. | -- | M | Pending |
| **RV-4** | **Expose structured execution traces from generated runtimes.** Replace stdout-scraping node ID parsing with JSON trace events. Expose entrypoint and param-source IDs from compiler output so parity tests don't reverse-engineer naming. | -- | M | Pending |
| **RV-5** | **Migrate testgen to DAG-orchestrated execution.** Current `gunbc-testgen` uses imperative loop + `catch_unwind` + direct file I/O, bypassing the DAG engine. Rewrite to execute via `dsl/tools/testgen.dag`. | -- | L | Pending |
| **RV-6** | **Unify resource port naming convention.** Output ports use `file:write`/`file:read`, input ports use `res:file`. Converge on a single convention and eliminate the bridging pattern. | RV-1 | L | Pending |

#### Lane 7 Scope

| File | Task |
|------|------|
| `gunbc-dag/src/resolve.rs` | RV-1, RV-6 |
| `core/daglang/daglang-lower/src/lib.rs` | RV-1, RV-2, RV-6 |
| `core/daglang/daglang-emit/src/rust_exec_runtime.rs` | RV-2, RV-4 |
| `gunbc-dag/src/bin/sdlc.rs` | RV-3 |
| `core/daglang/daglang-cli/tests/codegen_parity.rs` | RV-4 |
| `gunbc-dag/src/bin/testgen.rs` | RV-5 |
| `core/ir/src/resource/mod.rs` | RV-6 |
| `lib/gist-ops/src/lib.rs`, `lib/review/src/graph.rs` | RV-6 |

#### Lane 7 Verification

```
cargo check --workspace
cargo test --workspace --lib
cargo clippy --all-targets -- -D warnings
```

---

## Deferred

| ID | Task | Context | Size | Status |
|----|------|---------|------|--------|
| **DG1** | **Daggen (Dynamic DAG Generation)** | `needs_daggen()` returns false. Re-enable to scale the pipeline by dynamically generating steps based on git diffs. | L | **DEFERRED** |
| **S12-E** | **Multi-worker CAS** | Gap E: `GcsClaimStore` with generation-based CAS (`x-goog-if-generation-match`). DSL exists (`gcs_claim_store.dag`); wiring deferred until cloud_run profile needed. | M | **DEFERRED** |

---

## Active Open Items (Deferred)

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
