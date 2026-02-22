# Task Sheet — Dependency-Ordered, Parallelizable

**Last updated**: 2026-02-22
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

---

## Current Open Work

No active delivery-lane tasks remain in this sheet. Remaining items are unscheduled
(`H1`, `H10`) or explicitly deferred (`DG1`, `S12-E`).

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

---

## Backlog (Feature Ideas -- Not Scheduled)

See `TODO/backlog.md` for details. Parked for future consideration:

- Display Reactive DSL (XL) -- requires new DSL infra
- Compute Stack Provision/Apply (L) -- service layer works, orchestration is XL
- Glob-aware Resource Admission (M) -- policy-sensitive concurrency, needs explicit design

---

## Deferred

| ID | Task | Context | Size | Status |
|----|------|---------|------|--------|
| **DG1** | **Daggen (Dynamic DAG Generation)** | `needs_daggen()` returns false. Re-enable to scale the pipeline by dynamically generating steps based on git diffs. | L | **DEFERRED** |
| **S12-E** | **Multi-worker CAS** | Gap E: `GcsClaimStore` with generation-based CAS (`x-goog-if-generation-match`). DSL exists (`gcs_claim_store.dag`); wiring deferred until cloud_run profile needed. | M | **DEFERRED** |

---

## Active Open Items (Deferred)

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
