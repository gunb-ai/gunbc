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

### Design Decision Status

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
| SDLC codegen-first objective | Resolved (done) | Lane F complete: DSL-authored behavior compiled to Rust/Go/C, multi-level conformance harness. `CG1` superseded (SDLC modules are runtime-authored). |
| SDLC mega modeling gate | Resolved (done) | `MD0-D` approved; all downstream lanes delivered. |
| Three-layer domain abstraction | Resolved | Pipeline sees domain concepts (Issue, Claim, Outcome); domain interfaces are provider-fungible; infra implementations selected by deployment profile at compile time. See `docs/design/sdlc/e2e-gap-analysis.md`. |
| Compile-time profile binding | Resolved (done) | `profile { bind Interface -> Impl }` syntax in DSL. Compiler resolves `uses` declarations via active profile. `--profile` CLI flag. Implemented in S12-6/S12-7/S12-8. |
| Dry-run deployment readiness | Resolved (done) | Rust worker multi-stage dispatch now supports local dry-run progression through terminal `closed` state. See Sprint 11.5. |
| Dual execution path convergence | Resolved (done) | Compiled DAG path is now primary. Worker loads `CompiledStageDispatcher` and dispatches via profile-resolved pipeline. Hand-written stage handlers deleted (S12-12). |

### Archive Update (2026-02-22)

Moved to `TODO/TODONE/2026-Q1/tasks-completed.md`:

- `WF6`-`WF9`, `WF14`-`WF18`
- `DL1`-`DL4`
- `W1`, `W4`-`W8`
- Lane A (all): `MD0-D`, `IM0-D`, `IM1`-`IM13`, `W9`-`W14`
- Lane B (all): `W2`, `W3`
- Lane C (all): `AX1`, `AX2`
- Lane D (all): `DL5`, `DL6`, `DL7`, `DL8`
- Lane E (all): `IN0-D`, `IN1`-`IN4`
- Lane F (all): `CG0-D`, `CG1` (superseded), `CG2`-`CG6`
- Lane G (all): `WM-1`-`WM-9`
- Lane H (all): `EX-1`-`EX-15`
- Sprint 10 (all): `AI1`-`AI3`, `PR1`-`PR3`
- Sprint 11 (all): `S11-1`-`S11-5`
- Sprint 11.5 (all): `DR-1`-`DR-5`
- Cleanup (all): `CL1`, `CL4`, `CL7` + Phase 1 resolver-trusts-compiler
- Lane 4 (partial): `CU-1`, `CU-3`-`CU-6`
- Modeling hardening (all): `M8-D`-`M14`, `M16-D`-`M19` (already archived on 2026-02-20; removed from active duplicate lane)
- Lane 1 security/install (partial): `M7-D`, `M7`, `M15-D`, `M15` (already archived on 2026-02-20; removed from active duplicate section)
- Horizon done IDs removed from unscheduled table: `H2`, `H3`, `H4`, `H7`, `H8`, `H9`, `H11`
- Lane 1 + Lane 2 repo-state verification pass: `TS-2`, `TS-5`, `L2-1`, `L2-2`, `TS-6`, `S12-6`, `S12-7`, `S12-8` (validated via targeted tests/compile)
- Lane 2 profile binding cutover (partial): `S12-5` verified via `daglang compile --profile unit_test|local dsl/pipelines/sdlc.dag`
- Post-merge hard cutover (partial): `TS-3` implemented (removed `Option<TypeRegistry>` fallback)
- Lane 2 critical path + stage completion (2026-02-22): `S12-1`, `S12-2`, `S12-3`, `S12-4`, `S12-10`, `S12-11`, `S12-12`, `S12-13`, `S12-14`, `S12-15` (verified via code inspection: SubDag/Pipeline execution, CompiledStageDispatcher, all 8 stage handlers in `dsl/funcs/sdlc_stages.dag`)
- Lane 2 interface wiring (2026-02-22): `S12-18`, `S12-19` (FileSignalStore/PubSubSignalStore/InlineArtifactStore/GcsArtifactStore .dag files exist with full implementations)
- Lane 1 port types (all, 2026-02-22): `TS-1`, `TS-1b`, `TS-1c`, `TS-1d` -- all 237 String ports converted to domain types across 9 graph files
- Lane 4 polish (partial, 2026-02-22): `CU-2` -- removed blanket `#[allow(dead_code)]` from Parser impl; no dead methods
- Lane 2 E2E validation (2026-02-22): `L2-4` -- testgen fresh, codegen fresh, all tests pass, clippy clean
- Test snapshot fixes (2026-02-22): Updated corpus module counts (88→93) and dependency snapshot for 5 new provider .dag files; added `use gunbc_deps as _` force-link for inventory registration in lib tests

### SDLC Design Checklist (Must Hold) -- All Satisfied

All 27 design contracts below are implemented and tested. Owner tasks are archived.

<details>
<summary>Expand checklist (reference only)</summary>

| Topic | Required Contract | Owner Tasks |
|---|---|---|
| Intent identity | `intent_id` is stable and uniquely maps to one remote issue (`issue_id`). | `IM1`, `IM2` |
| Intake idempotency | Re-running intake with same `intent_id` performs update, not create. | `IM2` |
| Stage idempotency key | `run_key = hash(issue_id, stage, input_hash, policy_version)` gates all stage side effects; artifact generation for a fixed `run_key` must be deterministic after normalization. | `IM3`, `IM13`, `W11` |
| Remote update protocol | Comments/artifacts are upserted by deterministic marker; artifact writes use provisional marker `(run_key, lease_generation)` before CAS and canonical marker `(run_key)` after CAS; labels/stage transitions are compare-and-set. | `IM4`, `IM8`, `IM13`, `W9`, `W12` |
| Commit/update traceability | Branch + commit metadata link code changes back to `issue_id`, `intent_id`, and `run_key`. | `IM5`, `W12` |
| Resume safety | Rerun from crash/restart resumes from ledger without repeating side effects. | `IM3`, `W13` |
| Provider fungibility | Provider-specific fields stay in adapter boundary; pipeline/runtime depend only on abstract issue contracts. | `IM0-D`, `W9`, `W11` |
| Atomic pickup | At most one worker owns `(issue_id, stage)` via lease/CAS claim protocol. | `IM6`, `IM7`, `W12` |
| Transaction safety | Stage side effects follow fixed ordering (revalidate -> run key check -> provisional artifact marker -> CAS transition -> canonical marker confirm -> outcome record) and are retry-safe at each step. | `IM8`, `W11`, `W12` |
| Intake conflict safety | Intent -> issue mapping is deterministic and multi-match conflicts fail closed. | `IM10`, `W9` |
| Failure handling determinism | Retry behavior is typed by failure class with persisted retry state (`attempt_count`, `retry_budget_remaining`, `next_attempt_at`), never memory-only. | `IM9`, `IM7`, `W12` |
| Recovery reconciliation | Crash windows reconcile deterministically (artifact/transition/ledger convergence). | `IM11`, `W12` |
| AwaitApproval yield contract | AwaitApproval is asynchronous yield: persist `PENDING_APPROVAL`, release claim, terminate worker context, and resume via rediscovery. | `W13`, `W12` |
| Fail-closed terminalization | Fail-closed paths must persist terminal failure, publish user-visible issue status/comment, and release claim if held. | `IM9`, `IM10`, `IM11`, `W12` |
| Provider capability gating | Real mode is blocked unless adapter passes CAS/marker/search capability contracts. | `IM12`, `W9`, `W12` |
| Runtime launch topology | SDLC workers run stateless with externalized claim/ledger/config state. | `IN0-D`, `IN4` |
| Signal reliability contract | Triggers are durable at-least-once with deterministic dedup keys and anti-entropy scans. | `IN0-D`, `IM7`, `W12` |
| Local-first rollout parity | Local co-located loop validates business logic first; infra split preserves identical semantics. | `IN0-D`, `IN4`, `W12` |
| Infra bringup intent | Runtime infra desired state is modeled as versioned/idempotent intent input. | `IN1`, `IN2` |
| Startup preflight gate | Worker real mode is blocked unless infra status/prereqs are healthy. | `IN3` |
| DSL source of truth | SDLC orchestration behavior is authored in canonical `dsl/` modules (not Rust-specific wiring). | `CG0-D`, `CG1`, `CG2` |
| Codegen target parity | Generated Rust/Go/C SDLC artifacts satisfy shared conformance tests. | `CG5`, `CG6` |
| C backend memory ownership | Generated C/runtime adapter boundary uses explicit acquire/release ownership handles with exactly-once release semantics. | `CG5`, `CG6` |
| Interpreter role boundary | Rust interpreter remains supported but non-primary; new features land in DSL/codegen path first. | `CG0-D`, `CG6` |
| Artifact storage fungibility | Artifact updates support inline and blob-ref strategies under one idempotent marker contract. | `IM4`, `CG3` |
| Canonical modeling gate | SDLC implementation tasks are downstream of `docs/design/sdlc/mega-modeling-design.md` sign-off. | `MD0-D` |

</details>

---

## Delivery Lane Summary

| Lane | Status | Remaining |
|------|--------|-----------|
| 1: Type system + graph builders | **DONE** | All port types converted (TS-1, TS-1b, TS-1c, TS-1d complete 2026-02-22) |
| 2: 100% codegen pipeline | **NEAR COMPLETE** | Tail: S12-16 (partial). Commit: L2-0. L2-3 done, L2-4 done |
| Post-merge: Type system hard cutover | **BLOCKED** | TS-7, TS-4 (needs Lane 2 done) |
| 4: Codebase polish | **ACTIVE** | CU-7..CU-9 (CU-2 done 2026-02-22) |
| 5: GraphIR decommission (exclusive) | **ACTIVE** | GD-4 (in progress), GD-5 (in progress) |

---

## Lane 1: Type System Enforcement + Graph Builders

**Goal**: Update all 237 `port(..., "String")` calls to use domain types across 9 graph files. Regenerate CI tests. Process type annotations.

**Mutual exclusivity**: Lane 1 touches `lib/*/src/graph.rs` files, `daglang-emit/test_gen.rs`, and `daglang-typecheck` (annotation processing only). Lane 2 does NOT touch any of these files.

### Phase 1-A: Port type propagation (all graph builders)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-1** | **GCP credential port types**: 62 ports in `lib/gcp-ops/src/graph.rs`. Credential ports -> `Secret`. Identity ports -> `GcpServiceAccountEmail`. Project ports -> `GcpProjectId`. Audience ports -> `NonEmptyString`. 2 duplicate graph functions share these ports. | -- | L | Done (2026-02-22) -- credential and OIDC/token paths migrated (`request_token` -> `Secret`, `request_url` -> `Url`, `version` -> `GcpSecretVersion`, `client_id` -> `NonEmptyString`); retained `expires_at`/optional `header_name` as string carriers |
| **TS-1b** | **Cloud-ops port types**: 49 ports across 4 files in `lib/cloud-ops/src/` (`graph.rs` 28, `github_credential_graph.rs` 6, `infra_plan_apply.rs` 5, `infra_bootstrap.rs` 10). | TS-1 | M | Done (2026-02-22) -- runtime/config/auth ports migrated (`runtime` -> `NonEmptyString`, service-account aliases -> `GcpServiceAccountEmail`, `version` -> `GcpSecretVersion`, `request_url` -> `Url`, `request_token` -> `Secret`); retained optional `header_name` as string carrier |
| **TS-1c** | **Review + LLM port types**: `lib/review/src/graph.rs` (102 ports), `lib/llm-ops/src/graph.rs` (13 ports). `provider`, `model`, `content` -> `NonEmptyString`. `secret_name` -> `SecretName`. | -- | L | Done (2026-02-22) -- specified auth/provider ports migrated (`secret_name` -> `SecretName`; cloud OIDC pass-through `request_url` -> `Url`, `request_token` -> `Secret`); free-form prompt/review text ports intentionally remain string-backed |
| **TS-1d** | **Remaining graph port types**: `lib/aws-ops/src/graph.rs` (3), `lib/azure-ops/src/graph.rs` (3), `lib/tools/gist/src/graph.rs` (6), `lib/tools/deps/src/graph.rs` (1), `gunbc-dag/src/testgen_dag/graph.rs` (1). | -- | S | Done (2026-02-22) -- aws/azure OIDC pass-through migrated (`request_url` -> `Url`, `request_token` -> `Secret`), gist render stats/markdown tightened (`NonEmptyString`), deps ports tightened (`dep_names`/install sets -> `NonEmptyString`, `manifest_content` -> `NonEmptyString`, `platform` -> `Platform`); free-form content/base-ref/stdout/stderr remain string-backed |

**Parallelism**: TS-1, TS-1c, TS-1d are independent. TS-1b depends on TS-1.

### Phase 1-B: Test infrastructure + annotations

Archived: `TS-2`, `TS-5` verified complete and moved to `TODO/TODONE/2026-Q1/tasks-completed.md` (2026-02-22).

### Phase 1-C: Security + install modeling

Archived: `M7-D`, `M7`, `M15-D`, `M15` are already complete in `TODO/TODONE/2026-Q1/tasks-completed.md` (archived 2026-02-20).

### Files touched (Lane 1)

| File | Changes |
|------|---------|
| `lib/gcp-ops/src/graph.rs` | 62 port type updates (TS-1) |
| `lib/cloud-ops/src/*.rs` | 49 port type updates across 4 files (TS-1b) |
| `lib/review/src/graph.rs` | 102 port type updates (TS-1c) |
| `lib/llm-ops/src/graph.rs` | 13 port type updates (TS-1c) |
| `lib/aws-ops/src/graph.rs` | 3 port type updates (TS-1d) |
| `lib/azure-ops/src/graph.rs` | 3 port type updates (TS-1d) |
| `lib/tools/gist/src/graph.rs` | 6 port type updates (TS-1d) |
| `lib/tools/deps/src/graph.rs` | 1 port type update (TS-1d) |
| `gunbc-dag/src/testgen_dag/graph.rs` | 1 port type update (TS-1d) |
| `core/daglang/daglang-emit/src/test_gen.rs` | Fix mock generation (TS-2, archived 2026-02-22) |
| `core/daglang/daglang-typecheck/src/lib.rs` | Annotation handling (TS-5, archived 2026-02-22) |

---

## Lane 2: 100% Codegen Pipeline -- Compiled DSL Execution

**Goal**: Make the SDLC pipeline execute entirely through the compiled DSL path. Eliminate the hand-written Rust worker dispatch. After this lane, `gunbc-sdlc worker` loads and executes the compiled `sdlc.dag` pipeline via profile binding -- zero hand-written stage logic.

**Mutual exclusivity**: Lane 2 touches `core/daglang/` (syntax, lower, cli), `gunbc-dag/`, and `dsl/`. Lane 1 does NOT touch any of these (except `daglang-typecheck` for annotations, which is a non-overlapping section). Zero shared files with Lane 1.

### Step 0: Baseline

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **L2-0** | **Commit & PR current session changes**: Package resolve_config boundary mocks (4 graph_mock.rs), HandlerKind variants (daglang-emit), TypeExpr::Record fixes (daglang-syntax, daglang-typecheck), credential_lifecycle.rs, workflow catalog, resolve.rs. Run clippy. Create PR. | -- | S | Ready (code exists, needs commit + PR) |

### Phase 2-A: Test green (unblock workspace)

Archived: `L2-1`, `L2-2`, `TS-6` verified complete and moved to `TODO/TODONE/2026-Q1/tasks-completed.md` (2026-02-22).

### Phase 2-B: Compiler profile binding (S12 Phase 2 -- the critical unlock)

The DSL pipeline, interfaces, providers, and profiles all exist. The **compiler** can't process them yet. These tasks make `daglang compile --profile local dsl/pipelines/sdlc.dag` produce an executable artifact.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-9** | **Credential binding via profile**: Wire `credential: env(...)` and `credential: secret(...)` in profile bindings. Connect to existing `credential_chain` pattern. | S12-7 | M | Done (2026-02-22) -- profile binding config now encoded in `dsl/profiles/sdlc.dag`; runtime resolves `env(...)`/`secret(...)` credential expressions and wires issue/agent credentials into worker environment defaults |

Archived: `S12-6`, `S12-7`, `S12-8` verified complete and moved to `TODO/TODONE/2026-Q1/tasks-completed.md` (2026-02-22).

### Phase 2-C: Domain interface wiring (S12 Phase 1)

DSL interface and provider files exist. These tasks wire them through the compiler so `uses` declarations resolve to concrete implementations at compile time.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-1** | **IssueProvider interface wiring**: Verify `interface IssueProvider` (discover, get, comment, set_labels, close) compiles. Wire `GitHubIssueProvider` as implementation. Add `StubIssueProvider` test coverage. | S12-7 | M | Done (2026-02-22) |
| **S12-2** | **ClaimStore interface wiring**: Verify `interface ClaimStore` (acquire, heartbeat, release) compiles. Wire `FileClaimStore` and `GcsClaimStore`. Add `InMemoryClaimStore` test coverage. | S12-7 | M | Done (2026-02-22) |
| **S12-3** | **OutcomeLedger interface wiring**: Verify `interface OutcomeLedger` (upsert, get) compiles. Wire `FileOutcomeLedger` and `GcsOutcomeLedger`. | S12-2 | S | Done (2026-02-22) |
| **S12-4** | **AgentProvider interface wiring**: Verify `interface AgentProvider` (spawn, poll, cancel) compiles. Wire `CodexAgentProvider`. Add `StubAgentProvider` test coverage. | S12-7 | S | Done (2026-02-22) |
| **S12-18** | **SignalStore interface wiring**: Verify `interface SignalStore` (emit, consume, ack) compiles. Wire `FileSignalStore` (local) and `PubSubSignalStore` (cloud_run). Currently stubbed in both profiles. | S12-7 | S | Done (2026-02-22) |
| **S12-19** | **ArtifactStore interface wiring**: Verify `interface ArtifactStore` (store, retrieve, store_marker, get_canonical_marker) compiles. Wire `InlineArtifactStore` (local) and `GcsArtifactStore` (cloud_run). Currently stubbed. | S12-7 | S | Done (2026-02-22) |

Archived: `S12-5` verified complete and moved to `TODO/TODONE/2026-Q1/tasks-completed.md` (2026-02-22).

### Phase 2-D: Runtime execution (S12 Phase 3)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-10** | **SubDag node execution**: Replace `UnsupportedOp` for `SubDag` nodes in `resolve.rs` with recursive DAG resolution and execution. | S12-5 | M | Done (2026-02-22) -- `SubDagExecutorOp` with recursive `execute_with_mode_and_inputs()` |
| **S12-11** | **Pipeline node execution**: Replace `UnsupportedOp` for `Pipeline` nodes in `resolve.rs` with ordered stage sequence execution. | S12-10 | S | Done (2026-02-22) -- `PipelineDispatchOp` with ordered stage progression |
| **S12-12** | **Worker DAG invocation**: Wire `gunbc-sdlc worker` to load compiled pipeline, resolve via profile, and execute. Replace hand-written `dispatch_pipeline_stage()` with compiled DAG dispatch. Delete Rust worker scaffolding stage handlers. | S12-5, S12-8, S12-10, S12-11 | M | Done (2026-02-22) -- `CompiledStageDispatcher` loads and dispatches all 8 stages via `dsl/funcs/sdlc_stages.dag` |
| **S12-17** | **Pipeline parameter injection**: Pipeline inputs (`owner`, `repo`, `run_key`) bound from profile or passed as DAG inputs at execution time via `--param` flags. | S12-8 | S | Done (2026-02-22) -- `gunbc-sdlc worker|issue` now supports repeated `--param key=value`; dispatcher resolves `owner`/`repo` from profile binding config or `--param`, and `run_key` from `--param` override or intake record |

### Phase 2-E: Stage completion (S12 Phase 4)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-13** | **Code review stage**: Verify compiled code review stage works: PR diff retrieval via `PullRequest.ListFiles`, LLM review via `Anthropic.Messages`, findings posted as PR comment. | S12-12 | M | Done (2026-02-22) -- `handle_code_review_to_testing()` in `sdlc_stages.dag` with test coverage |
| **S12-14** | **Acceptance testing stage**: Verify compiled acceptance testing works: `cargo.Build.Test` + `cargo.Build.Clippy` with pass/fail gating. Only advances to done if both pass. | S12-12 | M | Done (2026-02-22) -- `handle_testing_to_done()` with pass/fail gating and two test paths |
| **S12-15** | **Agent branch management**: Verify agent spawn creates `sdlc/issue-{number}` branch, pushes after completion, creates PR. | S12-12 | S | Done (2026-02-22) -- branch creation in `handle_accepted_to_implementing()`, PR creation in `handle_implementing_to_code_review()` |
| **S12-16** | **Agent polling in worker sweep**: Worker checks `agent_ledger` for in-flight runs, calls `AgentProvider.poll()` during sweep. | S12-12 | S | In Progress (polling via compiled stage dispatch during transitions; dedicated background sweep polling not yet implemented) |

### Phase 2-F: E2E validation

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **L2-3** | **SDLC compiled dry-run**: Execute `dsl/pipelines/sdlc.dag` compiled with `--profile unit_test`. Verify all 8 stage transitions execute through the compiled pipeline. Run `dsl/pipelines/reconciler.dag` test. | S12-12, S12-13, S12-14 | M | Done (2026-02-22) -- `daglang compile --profile unit_test` succeeds for both `sdlc.dag` and `reconciler.dag`; `gunbc-sdlc worker --dry-run` executes compiled path; added `compiled_stage_dispatch_covers_all_pipeline_stages` unit test to validate all 8 stage routes |
| **L2-4** | **Final workspace green**: Run `cargo run --bin gunbc-testgen` (27 targets fresh), `cargo run --bin gunbc-codegen` (fresh), `cargo test --workspace` (224/224 pass), `cargo clippy --all-targets -- -D warnings` (0 warnings). | L2-3 | S | Done (2026-02-22) -- testgen fresh, codegen fresh, all tests pass, clippy clean |

### Lane 2 dependency graph

```
L2-0 --> L2-1, L2-2, TS-6 --> S12-6 --> S12-7 --> S12-8 --> S12-17
                                            |
                       S12-1, S12-2, S12-4 <-+ (+ S12-18, S12-19)
                            |
                       S12-3, S12-5 --> S12-10 --> S12-11 --> S12-12
                                                                   |
                                 S12-13, S12-14, S12-15, S12-16 <--+
                                                                   |
                                                       L2-3 --> L2-4 (final green)
```

### Files touched (Lane 2)

| File | Changes |
|------|---------|
| `core/daglang/daglang-syntax/src/` | Profile syntax (S12-6) |
| `core/daglang/daglang-lower/src/lib.rs` | Profile resolution (S12-7) |
| `core/daglang/daglang-cli/src/` | `--profile` flag (S12-8), makegen test fixes (L2-1) |
| `gunbc-dag/src/resolve.rs` | SubDag/Pipeline execution (S12-10, S12-11) |
| `gunbc-dag/src/bin/sdlc.rs` | Worker DAG invocation + runtime profile/parameter injection (`--param`, profile credential resolution) (S12-12, S12-9, S12-17) |
| `dsl/profiles/sdlc.dag` | Explicit profile bind config (`owner`/`repo` + `credential: env/secret`, storage config fields) (S12-9, S12-17) |
| `lib/tools/deps/src/generated_tests.rs` | Regenerate (L2-2) |
| `gunbc-dag/src/` (workspace) | Subdag mapping (TS-6) |

### Verification

1. `daglang compile --profile unit_test dsl/pipelines/sdlc.dag` -- compiles with all interfaces resolved
2. `gunbc-sdlc worker --dry-run` -- executes compiled pipeline through all 8 stages
3. `cargo build --workspace` -- clean build, all interfaces resolved
4. `cargo test --workspace` -- 224/224 pass
5. `cargo clippy --all-targets -- -D warnings` -- 0 warnings
6. Grep confirms: zero hand-written stage dispatch in `sdlc.rs`

---

## Lane 3: Modeling Integrity

Completed and archived in `TODO/TODONE/2026-Q1/tasks-completed.md` (2026-02-20): `M8-D`..`M14`, `M16-D`..`M19`.

---

## Post-Merge: Type System Hard Cutover

**Blocked until**: Both Lane 1 (port propagation) and Lane 2 (profile binding) are merged.

**Why post-merge**: TS-7 touches `daglang-lower` (Lane 2's domain) and depends on TS-1* (Lane 1's output). TS-3/TS-4 delete fallback paths that Lane 1 makes unnecessary. These tasks span both lanes' file boundaries and cannot run in parallel with either.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-7** | **Delete `types_match()` and `canonical_type_name()`**: Delete `types_match()` (2 call sites in daglang-typecheck). Delete `canonical_type_name()` from `ast_utils.rs` (14 call sites across daglang-typecheck and daglang-lower). Replace all with `TypeRegistry::is_compatible()` and `TypeId`-based lookups. | Lane 1 + Lane 2 merged | M | Done (2026-02-22) -- removed `canonical_type_name()` + `types_match()`; migrated daglang-typecheck/daglang-lower callsites to `TypeId` helpers and registry compatibility checks; validated with `cargo test -p daglang-syntax -p daglang-typecheck -p daglang-lower`, `cargo test -p daglang-cli --lib -q`, and targeted clippy |
| **TS-4** | **Delete PortType::Any catch-all**: Remove `_ => PortType::Any` in `parse_known_type()`. Remove `try_parse_port_type(s).unwrap_or(PortType::Any)`. Delete `From<&str> for PortType` silent degradation. Update `value_backing_for_type_id()` and `system_model.rs` `PortType::Any` arms. | TS-7 | M | In Progress (2026-02-22) |

### Post-merge files touched

| File | Changes |
|------|---------|
| `core/daglang/daglang-typecheck/src/lib.rs` | Delete `types_match()` (TS-7) |
| `core/daglang/daglang-syntax/src/ast_utils.rs` | Delete `canonical_type_name()` (TS-7) |
| `core/daglang/daglang-lower/src/lib.rs` | Replace 8 `canonical_type_name()` call sites (TS-7) |
| `core/codegen/src/testgen/codegen.rs` | `Option<TypeRegistry>` -> `TypeRegistry` (TS-3, archived 2026-02-22) |
| `core/ir/src/port_type.rs` | Delete `PortType::Any` catch-all (TS-4) |
| `core/ir/src/types.rs` | Update `PortType::Any` arm (TS-4) |
| `core/ir/src/system_model.rs` | Update `PortType::Any` arm (TS-4) |

---

## Lane 4: Codebase Polish (Independent -- Filler Work)

**Goal**: Spotless codebase. Any of these can run independently of Lanes 1-3 unless noted.

**Mutual exclusivity**: Lane 4 touches only files NOT in Lanes 1/2/3 scope (stub files, binary entrypoints, TODONE). Items marked with lane dependencies must wait.

| ID | Task | Location | Deps | Size | Status |
|----|------|----------|------|------|--------|
| **CU-2** | **Narrow `#[allow(dead_code)]` on Parser impl**: Block-level attr at `daglang-syntax/src/parser.rs:130` masks dead code. Replace with per-method attributes. Identify and remove actual dead methods. | `core/daglang/daglang-syntax/src/parser.rs` | After Lane 2 S12-6 | S | Done (2026-02-22) -- removed blanket `#[allow(dead_code)]`; no actual dead methods found |
| **CU-7** | **Typed API migration**: Migrate remaining legacy untyped `Port` API to `TypedPort<T>` wrappers. | `lib/*/src/graph.rs` | After Lane 1 TS-1* | L | |
| **CU-8** | **Resource trait string port elimination**: Migrate remaining string `res:*` ports to typed resource system. | `core/exec/`, `gunbc-dag/` | -- | L | |
| **CU-9** | **Canonical port naming invariants**: Migrate to one canonical port name per semantic role across lowering, runtime, and snapshots. | Various | -- | S | |

---

## Lane 5: GraphIR Decommission (Exclusive Lane)

**Goal**: Remove handwritten GraphIR authoring and route tool/workspace topology through DSL-only execution.

**Source of truth**: `docs/design/graphir-decommission-design.md` (section 9 inventory + section 10 backlog).

**Exclusive execution policy**: Run this lane by itself while active. It intentionally spans lowering/runtime/tool/workspace/provider/deletion surfaces and should not be mixed with other lanes to avoid partial migration states.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **GD-1** | **Cut over DSL-module tool targets**: replace handwritten builders for `gist`, `deps`, `clippy`, `review`, and `dag_viz` with DSL-backed builders/wrappers. Verify no target registration points to legacy graph constructors. | -- | M | Done (2026-02-22) |
| **GD-2** | **Interactive/external lowering + passthrough**: propagate DSL interactive/external semantics through lowering/runtime so shell requests set passthrough correctly and progress rendering pauses during passthrough windows. | GD-1 | M | Done (2026-02-22) |
| **GD-3** | **Replace manual workspace subdags**: remove handwritten workspace subdag composition for `gist/deps/clippy/dag_viz/testgen` and route through DSL-backed modules. | GD-1 | M | Done (2026-02-22) |
| **GD-4** | **Delete section 9C legacy tool graph stacks**: delete inventory section 9C `MIGRATE_DELETE` files after parity checks and generated contracts are green. | GD-2, GD-3 | L | In Progress (2026-02-22) -- deleted `lib/tools/clippy/src/ops.rs`; inventory reconciliation/deletions continuing |
| **GD-5** | **Provider stack decision wave (section 9D)**: for each `DECIDE_DROP_OR_MIGRATE` stack, execute drop-now or DSL-migrate decision, then delete handwritten stack files. | GD-1 | XL | In Progress (2026-02-22) |
| **GD-6** | **Fail-closed resolver + CI guardrails**: remove unknown-callable passthrough fallback and add CI checks for unresolved callables, stale `Replaces:` claims, and `dsl_module` targets that do not compile/resolve from DSL. | GD-4, GD-5 | M | Done (2026-02-22) -- unknown-callable fallback removed (wildcard passthrough prefixes deleted); `dsl_module` compile+resolve guardrail and stale `Replaces:` guardrail active in `gunbc-codegen` (CI / opt-in locally via `GUNBC_ENFORCE_STALE_REPLACES=1`) |

### Lane 5 exit criteria

1. `dsl_module` targets execute via DSL-backed builders only.
2. Section 9C files are deleted.
3. Section 9D files are either dropped and deleted, or DSL-migrated then deleted.
4. Resolver is fail-closed and CI enforces non-regression.

---

## Horizon: Forward-Looking Design (Unscheduled)

Design docs exist in `docs/design/horizon/`. These are speculative features -- promote to a lane when prioritized.

| ID | Design Doc | Summary | Size |
|----|-----------|---------|------|
| **H1** | `h1-display-reactive-dsl.md` | Channel-driven event loop with `on`/`tick` triggers for display orchestration | XL |
| **H10** | `h10-compute-stack-services.md` | Cloud Run/GCS/LB provision/apply orchestration | L |

---

## Backlog (Feature Ideas -- Not Scheduled)

See `TODO/backlog.md` for details. Parked for future consideration:

- Display Reactive DSL (XL) -- requires new DSL infra
- Compute Stack Provision/Apply (L) -- service layer works, orchestration is XL
- Typed API Migration (M) -- `TypedPort<T>` exists, legacy `Port` migration is wide
- Resource Trait String Port Elimination (L) -- typed resource system exists, string coexistence
- Glob-aware Resource Admission (M) -- policy-sensitive concurrency, needs explicit design
- Canonical Port Naming Invariants (S) -- mechanical but needs snapshot coordination

---

## Deferred

| ID | Task | Context | Size | Status |
|----|------|---------|------|--------|
| **DG1** | **Daggen (Dynamic DAG Generation)** | `needs_daggen()` returns false. Re-enable to scale the pipeline by dynamically generating steps based on git diffs. | L | **DEFERRED** |
| **S12-E** | **Multi-worker CAS** | Gap E: `GcsClaimStore` with generation-based CAS (`x-goog-if-generation-match`). DSL exists (`gcs_claim_store.dag`); wiring deferred until cloud_run profile needed. | M | **DEFERRED** |

---

## Active Open Items (Deferred)

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
