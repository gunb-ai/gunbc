# Task Sheet — Dependency-Ordered, Parallelizable

**Last updated**: 2026-02-25
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

### Archive Update (2026-02-24)

Verified via codebase audit and marked done:
- `S12-9` (credential binding -- fully implemented: profile registry, credential parsing, wire_auth_credential_edges)
- `TS-4` (PortType::Any -- enum already eliminated, parse_known_type/try_parse_port_type gone)
- `GD-4` (legacy graph stacks -- zero lib/*/src/graph.rs files remain)
- `GD-5` (provider stack decisions -- N/A, design doc and markers no longer exist)
- `L2-0` (commit/PR -- workspace compiles clean on main)
- `S12-17` (pipeline parameter injection -- params declared in DSL with defaults, `--param` CLI wiring complete)
- `S12-16` (agent polling sweep -- transition polling via compiled dispatch; background sweep deferred to DAG rewrite)
- `TS-7` (delete types_match + canonical_type_name -- replaced with TypeRegistry::is_compatible and TypeId lookups)
- `GD-6` (fail-closed resolver -- PassthroughOp fallback removed; unresolved callables are compile errors)
- `L2-3`/`L2-4` (SDLC compiled dry-run + final green -- completed, workspace green)
- Golden snapshot fixes: all workflow fixture obligation counts regenerated, corpus module lists updated (87→103 files), representative_ast item lists updated
- `sdlc.rs` binary deleted -- will be rewritten in DAG later

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
- Lane 1 port types (partial, 2026-02-22): `TS-1c` (specified ports -- provider, model, content, secret_name -- all converted)
- Lane 1 port types (complete, 2026-02-22): `TS-1` (all 62 GCP ports), `TS-1b` (all cloud-ops ports), `TS-1c` (all review/LLM ports including auxiliaries), `TS-1d` (all gist/deps/remaining ports)
- Lane 4 codebase polish (2026-02-22): `CU-2` (parser dead_code narrowing)
- Lane 2 interface wiring (2026-02-22): `S12-18` (SignalStore), `S12-19` (ArtifactStore)
- Test snapshot fixes (2026-02-22): Updated corpus module counts (88→93) and dependency snapshot for 5 new provider .dag files
- Inventory force-link fix (2026-02-22): Added `use gunbc_deps as _` to `gunbc-dag/src/lib.rs` for `inventory` registration visibility in lib tests

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
| 1: Type system + graph builders | **DONE** | All TS-1/1b/1c/1d complete |
| 2: 100% codegen pipeline | **DONE** | All tasks complete; `sdlc.rs` deleted (rewrite in DAG later) |
| Post-merge: Type system hard cutover | **DONE** | TS-7 complete, TS-4 already done |
| 4: Codebase polish | Backlogged | CU-7..CU-9 moved to backlog |
| 5: GraphIR decommission (exclusive) | **DONE** | GD-6 complete (fail-closed resolver) |
| 6: Testgen auto-generation | **DONE** | TG-1..TG-5 |
| 7: Compile+link no-fallback hardening | Planned | NF-1..NF-6 |
| 8: Interface stub transport + per-profile live tests | Planned | IS-1..IS-8, PT-1..PT-6 |

---

## Lane 1: Type System Enforcement + Graph Builders

**Goal**: Update all 237 `port(..., "String")` calls to use domain types across 9 graph files. Regenerate CI tests. Process type annotations.

**Mutual exclusivity**: Lane 1 touches `lib/*/src/graph.rs` files, `daglang-emit/test_gen.rs`, and `daglang-typecheck` (annotation processing only). Lane 2 does NOT touch any of these files.

### Phase 1-A: Port type propagation (all graph builders)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-1** | **GCP credential port types**: 62 ports in `lib/gcp-ops/src/graph.rs`. Credential ports -> `Secret`. Identity ports -> `GcpServiceAccountEmail`. Project ports -> `GcpProjectId`. Audience ports -> `NonEmptyString`. 2 duplicate graph functions share these ports. | -- | L | Done (2026-02-22) -- all 62 ports converted: `expires_at`→NonEmptyString, `version`→GcpSecretVersion, `client_id`→NonEmptyString |
| **TS-1b** | **Cloud-ops port types**: 49 ports across 4 files in `lib/cloud-ops/src/` (`graph.rs` 28, `github_credential_graph.rs` 6, `infra_plan_apply.rs` 5, `infra_bootstrap.rs` 10). | TS-1 | M | Done (2026-02-22) -- all ports already typed; no remaining String ports |
| **TS-1c** | **Review + LLM port types**: `lib/review/src/graph.rs` (102 ports), `lib/llm-ops/src/graph.rs` (13 ports). `provider`, `model`, `content` -> `NonEmptyString`. `secret_name` -> `SecretName`. | -- | L | Done (2026-02-22) -- all ports converted including auxiliary: `question`, `answer`, `system_prompt`, `artifact`, `stats`, `dimension`, `depth`, `prior_findings`, `summary` → NonEmptyString |
| **TS-1d** | **Remaining graph port types**: `lib/aws-ops/src/graph.rs` (3), `lib/azure-ops/src/graph.rs` (3), `lib/tools/gist/src/graph.rs` (6), `lib/tools/deps/src/graph.rs` (1), `gunbc-dag/src/testgen_dag/graph.rs` (1). | -- | S | Done (2026-02-22) -- all ports converted: gist `result`/`markdown`/`contents`→NonEmptyString; deps `manifest_content`/`install_script`/`script`/`stdout`/`stderr`→NonEmptyString, `platform`→Platform |

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
| **L2-0** | **Commit & PR current session changes**: Package resolve_config boundary mocks (4 graph_mock.rs), HandlerKind variants (daglang-emit), TypeExpr::Record fixes (daglang-syntax, daglang-typecheck), credential_lifecycle.rs, workflow catalog, resolve.rs. Run clippy. Create PR. | -- | S | Done (2026-02-24) -- workspace compiles clean; changes already committed on main. |

### Phase 2-A: Test green (unblock workspace)

Archived: `L2-1`, `L2-2`, `TS-6` verified complete and moved to `TODO/TODONE/2026-Q1/tasks-completed.md` (2026-02-22).

### Phase 2-B: Compiler profile binding (S12 Phase 2 -- the critical unlock)

The DSL pipeline, interfaces, providers, and profiles all exist. The **compiler** can't process them yet. These tasks make `daglang compile --profile local dsl/pipelines/sdlc.dag` produce an executable artifact.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-9** | **Credential binding via profile**: Wire `credential: env(...)` and `credential: secret(...)` in profile bindings. Connect to existing `credential_chain` pattern. | S12-7 | M | Done (2026-02-24) -- profile registry (`daglang-lower:772-882`), credential parsing (`:950-987`), `wire_auth_credential_edges()` (`:5927-6007`). All 3 profiles use env()/secret(). |

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
| **S12-17** | **Pipeline parameter injection**: Pipeline inputs (`owner`, `repo`, `run_key`) bound from profile or passed as DAG inputs at execution time via `--param` flags. | S12-8 | S | Done (2026-02-24) |

### Phase 2-E: Stage completion (S12 Phase 4)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-13** | **Code review stage**: Verify compiled code review stage works: PR diff retrieval via `PullRequest.ListFiles`, LLM review via `Anthropic.Messages`, findings posted as PR comment. | S12-12 | M | Done (2026-02-22) -- `handle_code_review_to_testing()` in `sdlc_stages.dag` with test coverage |
| **S12-14** | **Acceptance testing stage**: Verify compiled acceptance testing works: `cargo.Build.Test` + `cargo.Build.Clippy` with pass/fail gating. Only advances to done if both pass. | S12-12 | M | Done (2026-02-22) -- `handle_testing_to_done()` with pass/fail gating and two test paths |
| **S12-15** | **Agent branch management**: Verify agent spawn creates `sdlc/issue-{number}` branch, pushes after completion, creates PR. | S12-12 | S | Done (2026-02-22) -- branch creation in `handle_accepted_to_implementing()`, PR creation in `handle_implementing_to_code_review()` |
| **S12-16** | **Agent polling in worker sweep**: Worker checks `agent_ledger` for in-flight runs, calls `AgentProvider.poll()` during sweep. | S12-12 | S | Done (2026-02-24) -- polling via compiled stage dispatch; `sdlc.rs` deleted, background sweep deferred to DAG rewrite |

### Phase 2-F: E2E validation

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **L2-3** | **SDLC compiled dry-run**: Execute `dsl/pipelines/sdlc.dag` compiled with `--profile unit_test`. Verify all 8 stage transitions execute through the compiled pipeline. Run `dsl/pipelines/reconciler.dag` test. | S12-12, S12-13, S12-14 | M | Done (2026-02-24) |
| **L2-4** | **Final workspace green**: Run `cargo run --bin gunbc-testgen` (27 targets fresh), `cargo run --bin gunbc-codegen` (fresh), `cargo test --workspace` (224/224 pass), `cargo clippy --all-targets -- -D warnings` (0 warnings). | L2-3 | S | Done (2026-02-24) -- 285 pass, 3 pre-existing failures, 0 clippy warnings |

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
| `gunbc-dag/src/bin/sdlc.rs` | Worker DAG invocation (S12-12) |
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
| **TS-7** | **Delete `types_match()` and `canonical_type_name()`**: Delete `types_match()` (2 call sites in daglang-typecheck). Delete `canonical_type_name()` from `ast_utils.rs` (14 call sites across daglang-typecheck and daglang-lower). Replace all with `TypeRegistry::is_compatible()` and `TypeId`-based lookups. | Lane 1 + Lane 2 merged | M | Done (2026-02-24) |
| **TS-4** | **Delete PortType::Any catch-all**: Remove `_ => PortType::Any` in `parse_known_type()`. Remove `try_parse_port_type(s).unwrap_or(PortType::Any)`. Delete `From<&str> for PortType` silent degradation. Update `value_backing_for_type_id()` and `system_model.rs` `PortType::Any` arms. | TS-7 | M | Done (2026-02-24) -- `PortType` enum already eliminated from codebase. `parse_known_type`/`try_parse_port_type` no longer exist. `core/ir/src/port_type.rs` does not exist. |

### Post-merge files touched

| File | Changes |
|------|---------|
| `core/daglang/daglang-typecheck/src/lib.rs` | Delete `types_match()` (TS-7) |
| `core/daglang/daglang-syntax/src/ast_utils.rs` | Delete `canonical_type_name()` (TS-7) |
| `core/daglang/daglang-lower/src/lib.rs` | Replace 17 `canonical_type_name()` call sites (TS-7) |
| `core/codegen/src/testgen/codegen.rs` | `Option<TypeRegistry>` -> `TypeRegistry` (TS-3, archived 2026-02-22) |

---

## Lane 4: Codebase Polish (Independent -- Filler Work)

**Goal**: Spotless codebase. Any of these can run independently of Lanes 1-3 unless noted.

**Mutual exclusivity**: Lane 4 touches only files NOT in Lanes 1/2/3 scope (stub files, binary entrypoints, TODONE). Items marked with lane dependencies must wait.

| ID | Task | Location | Deps | Size | Status |
|----|------|----------|------|------|--------|
| **CU-2** | **Narrow `#[allow(dead_code)]` on Parser impl**: Block-level attr at `daglang-syntax/src/parser.rs:130` masks dead code. Replace with per-method attributes. Identify and remove actual dead methods. | `core/daglang/daglang-syntax/src/parser.rs` | After Lane 2 S12-6 | S | Done (2026-02-22) -- removed blanket `#[allow(dead_code)]`; no dead methods found |
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
| **GD-4** | **Delete section 9C legacy tool graph stacks**: delete inventory section 9C `MIGRATE_DELETE` files after parity checks and generated contracts are green. | GD-2, GD-3 | L | Done (2026-02-24) -- zero `lib/*/src/graph.rs` files remain; all legacy tool graph stacks already deleted. Design doc also removed. |
| **GD-5** | **Provider stack decision wave (section 9D)**: for each `DECIDE_DROP_OR_MIGRATE` stack, execute drop-now or DSL-migrate decision, then delete handwritten stack files. | GD-1 | XL | N/A (2026-02-24) -- design doc and DECIDE_DROP_OR_MIGRATE markers no longer exist in codebase; legacy stacks already removed. |
| **GD-6** | **Fail-closed resolver + CI guardrails**: remove unknown-callable passthrough fallback and add CI checks for unresolved callables, stale `Replaces:` claims, and `dsl_module` targets that do not compile/resolve from DSL. | GD-4, GD-5 | M | Done (2026-02-24) -- PassthroughOp fallback removed; SDLC runtime callable evaluator added for `funcs.sdlc_*` modules |

### Lane 5 exit criteria

1. `dsl_module` targets execute via DSL-backed builders only.
2. Section 9C files are deleted.
3. Section 9D files are either dropped and deleted, or DSL-migrated then deleted.
4. Resolver is fail-closed and CI enforces non-regression.

---

## Lane 6: Testgen Auto-Generation

**Goal**: Any compilable `.dag` file gets full testgen treatment automatically — driven purely by types + DAG structure, zero manual input (no inline `test` blocks, no `MockSpec`, no `@mock_response` annotations).

**Principle**: The DAG structure knows which nodes are transport boundaries (they have `TransportRequest` inputs). The type system knows output shapes of every port. `auto_mock_spec()` already generates type-compatible mocks from nothing but a `Dag<T>`. The obligation model (4 buckets) is purely structural. None of this requires developer input.

**Pipeline** (all pieces exist, just need wiring):
```
compile .dag → auto_mock_spec() → obligation analysis → emit tests
```

**Design reference**: `docs/design/testgen.md`, `docs/design/v4/dsl-design.md` Appendix N

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TG-1** | **Universal DAG discovery**: `discover_compilable_modules()` scans all of `dsl/` for `.dag` files with `func` items. Filters out pure-library modules. Returns `CompilableModule { dsl_path, module_name, has_test_blocks, func_count }`. | -- | M | DONE |
| **TG-2** | **Auto-testgen pipeline**: `auto_testgen_for_module()` calls `build_dsl_graph(path)`, `auto_mock_spec(&dag, name)`, `generate_target()`. Tolerates compile failures (skip with warning). | TG-1 | M | DONE |
| **TG-3** | **Wire into testgen binary**: `build_testgen_graph_auto()` discovers all compilable modules, creates content upsert chains with `TestgenOp::AutoGenerate`. Registered as testgen tool builder. | TG-2 | M | DONE |
| **TG-4** | **Validate auto-mock equivalence**: 30 modules discovered, 21 generated (2,939 test functions), 9 skipped (pre-existing DSL resolver gaps, not auto-mock limitations). | TG-3 | L | DONE |
| **TG-5** | **Deprecate manual path**: Auto-discovery is primary. `#[testgen_target]` and inline test blocks are optional overrides. MockSpec enforcement panic retained as safety net (unreachable via auto-discovery). `docs/design/testgen.md` updated. | TG-4 | S | DONE |

### Lane 6 files touched

| File | Changes |
|------|---------|
| `gunbc-dag/src/testgen_dag/dag_test_discovery.rs` | `CompilableModule`, `AutoTestgenResult`, `discover_compilable_modules()`, `auto_testgen_for_module()`, comprehensive validation test (TG-1, TG-2, TG-4) |
| `gunbc-dag/src/testgen_dag/ops.rs` | `TestgenOp::AutoGenerate` variant with full pipeline in `execute()` (TG-3) |
| `gunbc-dag/src/testgen_dag/graph.rs` | `build_testgen_graph_auto()` — discovers + builds upsert chains (TG-3) |
| `gunbc-dag/src/testgen_dag/mod.rs` | Updated tool_target builder + exports (TG-3) |
| `docs/design/testgen.md` | Auto-generation as primary model, registry as legacy (TG-5) |

### Lane 6 verification (all passing)

- TG-1: `discover_compilable_modules()` returns 30 modules (>14)
- TG-2: `auto_testgen_for_module()` produces 335 test fns for `tools/makegen.dag` with zero manual input
- TG-3: `build_testgen_graph_auto()` builds graph with ≥14 generate nodes + upsert chains
- TG-4: 21/30 modules generate tests (2,939 test functions); 9 skipped (DSL resolver gaps)
- Clippy: `cargo clippy --all-targets -- -D warnings` = 0 warnings
- Tests: all testgen_dag tests pass (17/17)

---

## Lane 7: Compile+Link No-Fallback Hardening

**Goal**: Eliminate string-coupled/runtime fallback behavior by adopting compile+link semantics: extern symbol resolution, hard missing-symbol errors, and deterministic receipts.

**Design reference (source of truth)**: `docs/design/v4/domain-hard-error-no-fallback-plan.md`

**Scope note**: Keep this lane high-level until design review is finalized; do not implement ad-hoc shortcuts outside the design contract.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **NF-1** | **Extern DSL surface**: Add `extern func` and `extern asset` syntax/typechecking/lowering so runtime-provided behavior is explicit in DSL. | -- | L | Planned |
| **NF-2** | **Minimal symbol model**: Introduce canonical `SymbolId` + `NodeId` model and lower ops to `Intrinsic`/`Call`/`Extern`. | NF-1 | L | Planned |
| **NF-3** | **Link step + backend resolver contract**: Add linker stage that resolves extern funcs/assets through backend interfaces and emits hard missing-symbol errors. | NF-2 | L | Planned |
| **NF-4** | **Runtime/asset migration to extern symbols**: Convert existing runtime handler + embedded asset flows to extern symbol resolution. Remove hidden authority from CLI/emitter registries. | NF-3 | L | Planned |
| **NF-5** | **Delete fallback surfaces**: Remove passthrough controls/handlers, stub asset fallbacks, and module-name dispatch heuristics. | NF-4 | M | Planned |
| **NF-6** | **Determinism contract hardening**: Add compile receipt digests linked to emit-manifest and CI determinism gates (single-file, CI pipeline, directory compile) with deterministic diagnostic ordering. | NF-5 | M | Planned |

### Lane 7 exit criteria

1. No CLI/emitter/runtime fallback path remains for unresolved extern funcs/assets.
2. Runtime and embedded assets resolve through link-time extern symbol contracts.
3. Missing symbol failures are deterministic in both set and order.
4. Determinism receipts and emit manifests are stable across repeated runs.

---

## Lane 8: Interface Stub Transport + Per-Profile Live Tests

**Goal**: Unblock testgen for ~10 interface-using modules by generating stub transport from interface capability shapes (Part 1), then generate per-profile live integration tests (Part 2).

**Design reference (source of truth)**: `docs/design/interface-stub-transport.md`

### Part 1: Interface Stub Transport

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **IS-1** | **Add `InterfaceStub` to `ServiceTransportClass`**: New enum variant in `daglang-lower`. Audit all match arms for exhaustiveness. | -- | S | Planned |
| **IS-2** | **Add `add_interface_stub_transport_triplets()`**: Mirror resource capability transport pattern. Walk `InterfaceDef.capabilities`, create prepare/execute/parse triplets with `InterfaceStub` transport class. | IS-1 | M | Planned |
| **IS-3** | **Relax `enforce_profile_for_bound_uses()`**: Convert hard error to informational. Return `HashSet<String>` of interface types needing stubs. | IS-1 | S | Planned |
| **IS-4** | **Wire stubs into lowering flow**: Call `add_interface_stub_transport_triplets()` after service transport, merge into endpoint registry. | IS-2, IS-3 | S | Planned |
| **IS-5** | **Update `resolve_service_call_source()` fallback**: Try `cap_key` lookup when `active_profile_bindings` is `None`. Only error if stub lookup also fails. | IS-3 | S | Planned |
| **IS-6** | **Handle `InterfaceStub` in DynOp resolver**: `InterfaceStubPrepareOp`, `InterfaceStubExecuteOp` (errors in Real mode, auto-mocked in DryRun), `InterfaceStubParseOp`. | -- | M | Planned |
| **IS-7** | **Verify auto-mock compatibility**: Confirm stub execute nodes carry `ServiceTransportExecute` obligation for auto-mock. | IS-4 | S | Planned |
| **IS-8** | **Tests**: Lowerer test (no profile -> stub triplets), resolver test (Real mode error), integration (`make test-all`). | IS-4, IS-6, IS-7 | M | Planned |

### Part 1 dependency graph

```
IS-1 ──┬──> IS-2 ──> IS-4 ──> IS-7 ──> IS-8
       │              ^
IS-3 ──┘──> IS-5 ────/
IS-6 (parallel with IS-2..IS-5)
```

### Part 2: Per-Profile Live Tests

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **PT-1** | **Profile discovery module**: Scan `dsl/profiles/*.dag`, extract profile name, bound interfaces, env/secret requirements, inferred test class. | IS-8 | M | Planned |
| **PT-2** | **Augment `CompilableModule` with interface imports**: Add `interface_imports: HashSet<String>` populated from `import interfaces.*` in AST. | IS-8 | S | Planned |
| **PT-3** | **Add `LiveProfileTestConfig` to `TestgenTargetDef`**: `profile_name`, `test_class`, `fermi_cost`, `required`, `required_any_of`, `dag_builder_call`. | IS-8 | S | Planned |
| **PT-4** | **Add `build_dsl_graph_with_profile()`**: New compilation path threading `profile` through `CompileOptions` with `allow_placeholder_env`. | IS-8 | M | Planned |
| **PT-5** | **Generate per-profile test sections in codegen**: `build_per_profile_live_flow_sections()` — one `test_live_flow_{module}_{profile}()` per config, gated by env requirements. | PT-3, PT-4 | M | Planned |
| **PT-6** | **Wire profile discovery into auto-testgen pipeline**: `discover_profiles()` in graph build, `profiles_for_module()` per module, populate `live_profile_tests`. | PT-1, PT-2, PT-5 | M | Planned |

### Part 2 dependency graph

```
PT-1 ──> PT-6
PT-2 ──> PT-6
PT-3 ──> PT-5 ──> PT-6
PT-4 ──> PT-5
```

### Lane 8 exit criteria

1. All interface-using modules compile without `--profile` and produce valid DryRun-testable DAGs.
2. Testgen coverage increases from 21/30 to ~30/30 compilable modules.
3. Per-profile live tests appear in generated test files, gated by env requirements.
4. `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings` clean.

### Lane 8 files touched

| File | Changes |
|------|---------|
| `core/daglang/daglang-lower/src/lib.rs` | `InterfaceStub` variant, stub transport triplets, relaxed validation (IS-1..IS-5) |
| `gunbc-dag/src/resolve.rs` | Stub ops in DynOp resolver (IS-6) |
| `gunbc-dag/src/mock_defaults.rs` | Auto-mock verification (IS-7) |
| `gunbc-dag/src/testgen_dag/profile_discovery.rs` | New — profile scanning (PT-1) |
| `gunbc-dag/src/testgen_dag/dag_test_discovery.rs` | Interface imports on `CompilableModule` (PT-2) |
| `core/codegen/src/registry.rs` | `LiveProfileTestConfig` (PT-3) |
| `gunbc-dag/src/dsl_builder.rs` | Profile-aware compilation (PT-4) |
| `core/codegen/src/testgen/codegen.rs` | Per-profile test generation (PT-5) |
| `gunbc-dag/src/testgen_dag/{graph.rs, ops.rs}` | Pipeline wiring (PT-6) |

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
