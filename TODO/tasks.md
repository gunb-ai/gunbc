# Task Sheet — Dependency-Ordered, Parallelizable

**Last updated**: 2026-02-22
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Archive**: Completed items in `TODO/TODONE/tasks-completed.md`. Backlog in `TODO/backlog.md`.

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
| Compile-time profile binding | Open | `profile { bind Interface -> Impl }` syntax in DSL. Compiler resolves `uses` declarations via active profile. `--profile` CLI flag. |
| Dry-run deployment readiness | Resolved (done) | Rust worker multi-stage dispatch now supports local dry-run progression through terminal `closed` state. See Sprint 11.5. |
| Dual execution path convergence | Open | Rust worker path (scaffolding) vs compiled DAG path (target). Rust worker must not accumulate SDLC sequencing logic beyond what's needed for dry-run. |

### Archive Update (2026-02-22)

Moved to `TODO/TODONE/tasks-completed.md`:

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

### SDLC Design Checklist (Must Hold) — All Satisfied

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
| A: SDLC delivery | **DONE** | — |
| B: Review credential | **DONE** | — |
| C: Planner/CI | **DONE** | — |
| D: Daglang convergence | **DONE** | — |
| E: Runtime infra | **DONE** | — |
| F: Codegen-first SDLC | **DONE** | — |
| G: Workflow DSL migration | **DONE** | — |
| H: DSL expression language | **DONE** | — |
| 1: Type system + graph builders | **ACTIVE** | TS-1..TS-1d, TS-2, TS-5, M7, M15 |
| 2: 100% codegen pipeline | **ACTIVE** | L2-0..L2-4, TS-6, S12-1..S12-19 |
| Post-merge: Type system hard cutover | **BLOCKED** | TS-7, TS-3, TS-4a..TS-4d (needs both Lane 1 + Lane 2 done) |
| 3: Modeling integrity | **DONE** | All M7-M22 complete. GR-1..GR-4 graph.rs deletions blocked on Lane 2 |
| 4: Codebase polish | **ACTIVE** | CU-1..CU-10 |

---

## Lane 1: Type System Enforcement + Graph Builders

**Goal**: Update all 237 `port(..., "String")` calls to use domain types across 9 graph files. Regenerate CI tests. Process type annotations.

**Mutual exclusivity**: Lane 1 touches `lib/*/src/graph.rs` files, `daglang-emit/test_gen.rs`, and `daglang-typecheck` (annotation processing only). Lane 2 does NOT touch any of these files.

### Phase 1-A: Port type propagation (all graph builders)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-1** | **GCP credential port types**: 62 ports in `lib/gcp-ops/src/graph.rs`. Credential ports → `Secret`. Identity ports → `GcpServiceAccountEmail`. Project ports → `GcpProjectId`. Audience ports → `NonEmptyString`. 2 duplicate graph functions share these ports. | — | L | |
| **TS-1b** | **Cloud-ops port types**: 43 ports across 3 files in `lib/cloud-ops/src/` (`graph.rs` 28, `infra_plan_apply.rs` 5, `infra_bootstrap.rs` 10). `github_credential_graph.rs` deleted — 6 ports removed. | TS-1 | M | |
| **TS-1c** | **Review + LLM port types**: `lib/review/src/graph.rs` (102 ports), `lib/llm-ops/src/graph.rs` (13 ports). `provider`, `model`, `content` → `NonEmptyString`. `secret_name` → `SecretName`. | — | L | |
| **TS-1d** | **Remaining graph port types**: `lib/tools/deps/src/graph.rs` (1), `gunbc-dag/src/testgen_dag/graph.rs` (1). `aws-ops/graph.rs` (3), `azure-ops/graph.rs` (3), `gist/graph.rs` (6), `clippy/graph.rs` deleted — 15 ports removed; DSL handles typing now. | — | S | |

**Parallelism**: TS-1, TS-1c, TS-1d are independent. TS-1b depends on TS-1.

### Phase 1-B: Test infrastructure + annotations

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-2** | **Regenerate CI generated tests**: 2197 CI tests fail (`invalid 'items' input: expected StringList`). Fix `typed_mock_for_response` catch-all in `daglang-emit/test_gen.rs` (line 155). | — | M | |
| **TS-5** | **Process all annotations in typecheck**: `@content(encoding)` → `Predicate::Content`, `@brand(name)` → `TypeOp::Brand`, `@non_empty` → `Predicate::NonEmpty`, `@pattern(regex)` → `Predicate::Matches`, `@file_types` → extension→encoding map. | — | L | |

### Phase 1-C: Security + install modeling (from `TODO/modeling.md`)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **M7-D** | **Design: Secret redaction by default**: Capability-split `SecretValue` runtime + redacted render type. `Display`/`Debug`/`ToString` always redacted; plaintext extraction is explicit transport-boundary only. | — | S | |
| **M7** | **Secret redaction by default**: Implement per M7-D design. Audit all plaintext extraction callsites. Add clippy disallow rules for plaintext methods outside transport boundaries. Regression tests for no plaintext in renderers. | M7-D | M | |
| **M15-D** | **Design: Typed package manager modeling**: `PackageManagerId` typed parse + explicit `InstallPlan` policy model. Unknown IDs fail closed. | — | S | |
| **M15** | **Typed install planning**: Remove stringly/lossy installer bridging. Implement typed `PackageManagerId` with explicit selection policy. Adapter preserves required fields. Exhaustive tests. | M15-D | M | |

### Files touched (Lane 1)

| File | Changes |
|------|---------|
| `lib/gcp-ops/src/graph.rs` | 62 port type updates (TS-1) |
| `lib/cloud-ops/src/*.rs` | 43 port type updates across 3 files (TS-1b) — `github_credential_graph.rs` deleted |
| `lib/review/src/graph.rs` | 102 port type updates (TS-1c) |
| `lib/llm-ops/src/graph.rs` | 13 port type updates (TS-1c) |
| `lib/tools/deps/src/graph.rs` | 1 port type update (TS-1d) |
| `gunbc-dag/src/testgen_dag/graph.rs` | 1 port type update (TS-1d) |
| `core/daglang/daglang-emit/src/test_gen.rs` | Fix mock generation (TS-2) |
| `core/daglang/daglang-typecheck/src/lib.rs` | Annotation handling only (TS-5) |
| `core/ir/src/transport/` | SecretValue redaction (M7) |

---

## Lane 2: 100% Codegen Pipeline — Compiled DSL Execution

**Goal**: Make the SDLC pipeline execute entirely through the compiled DSL path. Eliminate the hand-written Rust worker dispatch. After this lane, `gunbc-sdlc worker` loads and executes the compiled `sdlc.dag` pipeline via profile binding — zero hand-written stage logic.

**Mutual exclusivity**: Lane 2 touches `core/daglang/` (syntax, lower, cli), `gunbc-dag/`, and `dsl/`. Lane 1 does NOT touch any of these (except `daglang-typecheck` for annotations, which is a non-overlapping section). Zero shared files with Lane 1.

### Step 0: Baseline

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **L2-0** | **Commit & PR current session changes**: Package resolve_config boundary mocks (4 graph_mock.rs), HandlerKind variants (daglang-emit), TypeExpr::Record fixes (daglang-syntax, daglang-typecheck), credential_lifecycle.rs, workflow catalog, resolve.rs. Run clippy. Create PR. | — | S | |

### Phase 2-A: Test green (unblock workspace)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **L2-1** | **Fix 4 failing daglang-cli makegen tests**: `resolve_lowered_dag_maps_makegen_nodes_to_dyn_ops` assertion fails on `LoadRegistry` op debug format. 3 `compile_resolve_execute_makegen` tests also fail. | L2-0 | S | |
| **L2-2** | **Deps generated test freshness**: `lib/tools/deps/src/generated_tests.rs` stale `FileResponse` struct (missing `bytes` field). Regenerate with `cargo run --bin gunbc-testgen`. | L2-0 | S | |
| **TS-6** | **Workspace subdag mapping**: 2 workspace subdag tests fail ("unmapped DSL pipeline modules: reconciler, sdlc"). Add module mappings or exclusions. | L2-0 | S | |

### Phase 2-B: Compiler profile binding (S12 Phase 2 — the critical unlock)

The DSL pipeline, interfaces, providers, and profiles all exist. The **compiler** can't process them yet. These tasks make `daglang compile --profile local dsl/pipelines/sdlc.dag` produce an executable artifact.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-6** | **Profile syntax in parser**: Add `profile` declaration and `bind` statement to `daglang-syntax`. Parse `profile local { bind IssueProvider -> GitHubIssueProvider { credential: env("...") } }`. | L2-1 | M | |
| **S12-7** | **Profile resolution in lowering**: When lowering `uses` declarations, resolve via active profile's bindings. Generate transport code for the bound concrete implementation. Thread profile through lowering context. | S12-6 | L | |
| **S12-8** | **`--profile` CLI flag**: Add `--profile` to `daglang compile` and all tool binaries. Load profile definitions from `dsl/profiles/`. Create `unit_test`, `local`, `cloud_run` profiles. | S12-6, S12-7 | S | |
| **S12-9** | **Credential binding via profile**: Wire `credential: env(...)` and `credential: secret(...)` in profile bindings. Connect to existing `credential_chain` pattern. | S12-7 | M | |

### Phase 2-C: Domain interface wiring (S12 Phase 1)

DSL interface and provider files exist. These tasks wire them through the compiler so `uses` declarations resolve to concrete implementations at compile time.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-1** | **IssueProvider interface wiring**: Verify `interface IssueProvider` (discover, get, comment, set_labels, close) compiles. Wire `GitHubIssueProvider` as implementation. Add `StubIssueProvider` test coverage. | S12-7 | M | |
| **S12-2** | **ClaimStore interface wiring**: Verify `interface ClaimStore` (acquire, heartbeat, release) compiles. Wire `FileClaimStore` and `GcsClaimStore`. Add `InMemoryClaimStore` test coverage. | S12-7 | M | |
| **S12-3** | **OutcomeLedger interface wiring**: Verify `interface OutcomeLedger` (upsert, get) compiles. Wire `FileOutcomeLedger` and `GcsOutcomeLedger`. | S12-2 | S | |
| **S12-4** | **AgentProvider interface wiring**: Verify `interface AgentProvider` (spawn, poll, cancel) compiles. Wire `CodexAgentProvider`. Add `StubAgentProvider` test coverage. | S12-7 | S | |
| **S12-5** | **Pipeline uses interfaces**: Verify `dsl/pipelines/sdlc.dag` and `dsl/funcs/sdlc_worker.dag` compile with all `uses` declarations resolved via profile binding. | S12-1, S12-2, S12-3, S12-4 | M | |
| **S12-18** | **SignalStore interface wiring**: Verify `interface SignalStore` (emit, consume, ack) compiles. Wire `FileSignalStore` (local) and `PubSubSignalStore` (cloud_run). Currently stubbed in both profiles. | S12-7 | S | |
| **S12-19** | **ArtifactStore interface wiring**: Verify `interface ArtifactStore` (store, retrieve, store_marker, get_canonical_marker) compiles. Wire `InlineArtifactStore` (local) and `GcsArtifactStore` (cloud_run). Currently stubbed. | S12-7 | S | |

### Phase 2-D: Runtime execution (S12 Phase 3)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-10** | **SubDag node execution**: Replace `UnsupportedOp` for `SubDag` nodes in `resolve.rs` with recursive DAG resolution and execution. | S12-5 | M | |
| **S12-11** | **Pipeline node execution**: Replace `UnsupportedOp` for `Pipeline` nodes in `resolve.rs` with ordered stage sequence execution. | S12-10 | S | |
| **S12-12** | **Worker DAG invocation**: Wire `gunbc-sdlc worker` to load compiled pipeline, resolve via profile, and execute. Replace hand-written `dispatch_pipeline_stage()` with compiled DAG dispatch. Delete Rust worker scaffolding stage handlers. | S12-5, S12-8, S12-10, S12-11 | M | |
| **S12-17** | **Pipeline parameter injection**: Pipeline inputs (`owner`, `repo`, `run_key`) bound from profile or passed as DAG inputs at execution time via `--param` flags. | S12-8 | S | |

### Phase 2-E: Stage completion (S12 Phase 4)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-13** | **Code review stage**: Verify compiled code review stage works: PR diff retrieval via `PullRequest.ListFiles`, LLM review via `Anthropic.Messages`, findings posted as PR comment. | S12-12 | M | |
| **S12-14** | **Acceptance testing stage**: Verify compiled acceptance testing works: `cargo.Build.Test` + `cargo.Build.Clippy` with pass/fail gating. Only advances to done if both pass. | S12-12 | M | |
| **S12-15** | **Agent branch management**: Verify agent spawn creates `sdlc/issue-{number}` branch, pushes after completion, creates PR. | S12-12 | S | |
| **S12-16** | **Agent polling in worker sweep**: Worker checks `agent_ledger` for in-flight runs, calls `AgentProvider.poll()` during sweep. | S12-12 | S | |

### Phase 2-F: E2E validation

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **L2-3** | **SDLC compiled dry-run**: Execute `dsl/pipelines/sdlc.dag` compiled with `--profile unit_test`. Verify all 8 stage transitions execute through the compiled pipeline. Run `dsl/pipelines/reconciler.dag` test. | S12-12, S12-13, S12-14 | M | |
| **L2-4** | **Final workspace green**: Run `cargo run --bin gunbc-testgen` (27 targets fresh), `cargo run --bin gunbc-codegen` (fresh), `cargo test --workspace` (224/224 pass), `cargo clippy --all-targets -- -D warnings` (0 warnings). | L2-3 | S | |

### Lane 2 dependency graph

```
L2-0 ──→ L2-1, L2-2, TS-6 ──→ S12-6 ──→ S12-7 ──→ S12-8 ──→ S12-17
                                            │
                       S12-1, S12-2, S12-4 ←┘ (+ S12-18, S12-19)
                            │
                       S12-3, S12-5 ──→ S12-10 ──→ S12-11 ──→ S12-12
                                                                   │
                                 S12-13, S12-14, S12-15, S12-16 ←─┘
                                                                   │
                                                       L2-3 ──→ L2-4 (final green)
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

1. `daglang compile --profile unit_test dsl/pipelines/sdlc.dag` — compiles with all interfaces resolved
2. `gunbc-sdlc worker --dry-run` — executes compiled pipeline through all 8 stages
3. `cargo build --workspace` — clean build, all interfaces resolved
4. `cargo test --workspace` — 224/224 pass
5. `cargo clippy --all-targets -- -D warnings` — 0 warnings
6. Grep confirms: zero hand-written stage dispatch in `sdlc.rs`

---

## Lane 3: Modeling Integrity (ACTIVE)

**Source**: `TODO/modeling.md` — 13 intake tasks for semantic-integrity hardening.
**Design-first policy**: Every M* task requires a paired M*-D design review before implementation.
**Status**: All 16 modeling tasks (M7-M22) COMPLETE. Graph.rs cleanup (GR-1..GR-4) blocked on Lane 2 S12 (profile binding).

### Lane 3-A: Graph semantics (M8 → M9 → M16)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **M8-D** | **Design: Separate metadata from validation** | — | S | ✅ Done |
| **M8** | **Semantically inert metadata op**: `TypeOp::Meta(MetadataPayload)`. Erasure-invariance test. Strict-mode guard. | M8-D | M | ✅ Done |
| **M9-D** | **Design: Typed dependency markers** | M8-D | S | ✅ Done |
| **M9** | **Typed dependency markers**: `DependencyKind` enum, `DependencyEdge` struct, round-trip tests. | M9-D, M8 | S | ✅ Done |
| **M16-D** | **Design: SystemModel/TransportBehavior unification** | M8-D | S | ✅ Done |
| **M16** | **Unify invocation contracts**: `ProtocolLayer` + `ProtocolStack` types in `contract.rs`. Bridge from `TransportBehavior`+`BehaviorScope`. **Unblocks**: graph.rs cleanup + cloud modeling. | M16-D, M8 | M | ✅ Done |

### Lane 3-A+ : Graph.rs Cleanup (Blocked on M16)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **GR-1** | **Delete `review/graph.rs`** (1,727 lines) | M16 | S | |
| **GR-2** | **Delete `llm-ops/graph.rs`** (267 lines) — only called by review's graph builder | GR-1, M16 | S | |
| **GR-3** | **Delete `cloud-ops/graph.rs`** (502 lines) — central dispatcher | M16 | S | |
| **GR-4** | **Delete `gcp-ops/graph.rs`** (1,674 lines) — called by cloud-ops dispatcher | GR-3, M16 | S | |

### Lane 3-B: Workflow execution safety (M10 → M11 → M12)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **M10-D** | **Design: Mandatory resource declarations + auto-wiring** | — | S | ✅ Done |
| **M10** | **Mandatory resource declarations**: `EffectKind` enum, `validate_effectful_declarations()`, auto-wiring helpers. | M10-D | L | ✅ Done |
| **M11-D** | **Design: Strict dry-run mode** | M10-D | S | ✅ Done |
| **M11** | **Strict dry-run in CI/testgen**: `DryRunStrictness::{Lenient,Strict}`, poison value model, fail-fast executor. | M11-D, M10 | M | ✅ Done |
| **M12-D** | **Design: Coercion proof nodes** | M11-D | S | ✅ Done |
| **M12** | **Coercion proof nodes**: `ShapeContract` assertion framework, shape/cardinality invariants. | M12-D, M11 | S | ✅ Done |

### Lane 3-C: Process contract drift (M13 → M14)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **M13-D** | **Design: Registry→CLI→Make contract tests** | — | S | ✅ Done |
| **M13** | **Contract tests**: Round-trip registry→CLI→Make parity tests. | M13-D | M | ✅ Done |
| **M14-D** | **Design: Single inventory authority** | M13-D | S | ✅ Done |
| **M14** | **Single inventory authority**: `inventory_is_single_authority` test, drift validation. | M14-D, M13 | M | ✅ Done |

### Lane 3-D: Global minimality proof (M17 → M18 → M19)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **M17-D** | **Design: Global flattening + context-free work identity** | — | M | ✅ Done |
| **M17** | **Global flattening**: `WorkIdentity`, `GlobalPlan`, `PlanStep`, flattening + merge. | M17-D | L | ✅ Done |
| **M18-D** | **Design: Single semantic authority / projection-only surfaces** | M17-D | S | ✅ Done |
| **M18** | **Projection-only surfaces**: `ProjectionSurface`, drift detection, canonical derivation. | M18-D, M17 | M | ✅ Done |
| **M19-D** | **Design: Formal non-redundancy proof harness** | M18-D | S | ✅ Done |
| **M19** | **Non-redundancy proof harness**: `NonRedundancyProof`, at-most-once, minimal-dirty-closure, single-writer checks. | M19-D, M17, M18 | M | ✅ Done |

### Lane 3-E: Repo self-model + codegen + annotations (M20, M21, M22)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **M20** | **Repository self-understanding model**: `CrateTier`, `CrateSpec`, generator edges, commit policies, toolchain requirements in `workspace_model.rs`. | — | L | ✅ Done |
| **M21** | **Structural primitives for codegen**: `CodegenTypeShape`, `ScalarKind`, `CodegenPlatformRepr`, `Platform` types in `contract.rs`. `TypeShape` extractor in `type_shape.rs`. | M8 | L | ✅ Done |
| **M22** | **Annotation-to-DAG modeling**: `ContractObligation` (Phase 1), `ErrorMapping` (Phase 2), `RetryPolicy` (Phase 3), `ResourceRequirement` (Phase 4), `@testgen_skip` emit wiring (Phase 5). | M8, M10, M12, M16 | L | ✅ Done |

### Lane 3 dependency graph

```
Lane 3-A:  M8 ✅ → M9 ✅ → M16 ✅ → [graph.rs cleanup: GR-1..GR-4]
                                      → [cloud gap: AWS services, Azure services, GitHub ops wiring]
Lane 3-B:  M10 ✅ → M11 ✅ → M12 ✅
Lane 3-C:  M13 ✅ → M14 ✅
Lane 3-D:  M17 ✅ → M18 ✅ → M19 ✅
Lane 3-E:  M20 ✅, M21 ✅, M22 ✅

3-A graph.rs cleanup is the critical path (blocked on M16 completion).
```

---

## Post-Merge: Type System Hard Cutover

**Blocked until**: Both Lane 1 (port propagation) and Lane 2 (profile binding) are merged.

**Why post-merge**: TS-7 touches `daglang-lower` (Lane 2's domain) and depends on TS-1* (Lane 1's output). TS-3/TS-4 delete fallback paths that Lane 1 makes unnecessary. These tasks span both lanes' file boundaries and cannot run in parallel with either.

**Motivation**: `port_type.rs` is a redundant, less-expressive shadow of `TypeRegistry` that hard-codes ~40 domain type mappings in a giant match statement. Every time the type system evolves, this shadow gets stale and causes bugs (e.g., `Credential` was mapped to `PortType::Secret` but serializes to `Value::Map`; `Platform` needed a special-case hack for dual backing). The `TypeRegistry` already has all this information in structured form — `port_type.rs` must be deleted.

### Phase PM-A: Make TypeRegistry the sole authority

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-7** | **Delete `types_match()` and `canonical_type_name()`**: Delete `types_match()` (2 call sites in daglang-typecheck). Delete `canonical_type_name()` from `ast_utils.rs` (14 call sites across daglang-typecheck and daglang-lower). Replace all with `TypeRegistry::is_compatible()` and `TypeId`-based lookups. | Lane 1 + Lane 2 merged | M | |
| **TS-3** | **Make TypeRegistry non-optional**: Change `Option<TypeRegistry>` → `TypeRegistry` in `core/codegen/src/testgen/codegen.rs`. Audit for other `Option<TypeRegistry>` patterns. This ensures testgen always has rich type info and never falls through to `PortType`-based guessing. | TS-7 | S | |

### Phase PM-B: Delete `port_type.rs` (file deletion)

**Goal**: Eliminate `core/ir/src/port_type.rs` entirely (355 lines). Three consumers to migrate:

| Consumer | Location | What it uses | Replacement |
|----------|----------|-------------|-------------|
| `value_backing_for_type_id()` | `types.rs:846` | `PortType::from()` → match on 9 variants | `TypeRegistry::value_backing()` (new method) |
| `rust_type_for_port_type()` | `system_model.rs:851` | `PortType::from()` → Rust type string | `TypeRegistry::base_type_name()` → same match |
| `PortType::from_registry()` | `port_type.rs:80` | Registry-aware resolution | Replace callers with direct `TypeRegistry` queries |

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-4a** | **Add `TypeRegistry::value_backing()` method**: New method on `TypeRegistry` that maps a `TypeId` → `ValueBacking` using `base_type_name()` chain. This replaces `value_backing_for_type_id()` which currently delegates to `PortType`. Handle parametric types (`List<T>`, `Set<T>`, `Map<K,V>`, `Optional<T>`) via expression parsing already in the registry. | TS-3 | M | |
| **TS-4b** | **Migrate `value_backing_for_type_id()` to registry**: Replace `PortType::from(type_id)` dispatch in `types.rs:846-890` with `TypeRegistry::value_backing()`. Thread `&TypeRegistry` through `value_compatible_with_type_id()` and its callers (`mock_requirements.rs:245`, `testgen/codegen.rs:121`). Delete `Platform` and similar special-case hacks in `value_compatible_with_type_id()` — the registry handles them. | TS-4a | M | |
| **TS-4c** | **Migrate `system_model.rs` and delete `port_type.rs`**: Replace `PortType::from(type_id)` in `rust_type_for_port_type()` (3 call sites at lines 908, 920, 929) with `TypeRegistry::base_type_name()`. Remove `pub use port_type::PortType` from `lib.rs`. Delete `core/ir/src/port_type.rs` (355 lines). Remove `mod port_type` from `lib.rs`. | TS-4b | S | |
| **TS-4d** | **Regression test: compound type mock compatibility**: Add test cases for all capability-marker types (`Credential`, `ToolHandle`, `FilesystemHandle`, `NetworkHandle`) and dual-backing types (`Platform`) in `test_value_compatible_with_type_id`. These are the types that `port_type.rs` got wrong. | TS-4b | S | |

### Post-merge dependency graph

```
Lane 1 + Lane 2 merged
        │
       TS-7 ──→ TS-3 ──→ TS-4a ──→ TS-4b ──→ TS-4c (port_type.rs DELETED)
                                       │
                                      TS-4d (regression tests)
```

### Post-merge files touched

| File | Changes |
|------|---------|
| `core/daglang/daglang-typecheck/src/lib.rs` | Delete `types_match()` (TS-7) |
| `core/daglang/daglang-syntax/src/ast_utils.rs` | Delete `canonical_type_name()` (TS-7) |
| `core/daglang/daglang-lower/src/lib.rs` | Replace 8 `canonical_type_name()` call sites (TS-7) |
| `core/codegen/src/testgen/codegen.rs` | `Option<TypeRegistry>` → `TypeRegistry` (TS-3), thread registry to `mock_types_compatible` (TS-4b) |
| `core/ir/src/type_registry.rs` | Add `value_backing()` method (TS-4a) |
| `core/ir/src/types.rs` | Migrate `value_backing_for_type_id()` + `value_compatible_with_type_id()` to use registry (TS-4b), delete special-case hacks |
| `core/ir/src/system_model.rs` | Replace `PortType::from()` with registry queries (TS-4c) |
| `core/ir/src/port_type.rs` | **DELETE** (TS-4c) |
| `core/ir/src/lib.rs` | Remove `mod port_type` + `pub use` (TS-4c) |
| `core/test/src/mock_requirements.rs` | Thread `&TypeRegistry` through `types_compatible()` (TS-4b) |

---

## Lane 4: Codebase Polish (Independent — Filler Work)

**Goal**: Spotless codebase. Any of these can run independently of Lanes 1-3 unless noted.

**Mutual exclusivity**: Lane 4 touches only files NOT in Lanes 1/2/3 scope (stub files, binary entrypoints, TODONE). Items marked with lane dependencies must wait.

| ID | Task | Location | Deps | Size | Status |
|----|------|----------|------|------|--------|
| **CU-1** | **Audit near-empty stub files**: 2 remaining: `gunbc-dag/src/policy/mod.rs`, `gunbc-dag/tests/common/mod.rs` (valid module tree structure — keep). ~~5 deleted~~: `c_ir.rs`, `go_ir.rs`, `register_ir.rs` (Batch 1d), `testgen/render.rs` (dead re-export), `cloud_env.rs` (inlined). `daglang-cli/src/lib.rs` is minimal but functional. | Various | — | S | ✅ Done |
| **CU-2** | **Narrow `#[allow(dead_code)]` on Parser impl**: Block-level attr at `daglang-syntax/src/parser.rs:130` masks dead code. Replace with per-method attributes. Identify and remove actual dead methods. | `core/daglang/daglang-syntax/src/parser.rs` | After Lane 2 S12-6 | S | |
| **CU-3** | **Factor common mock helpers**: 3 largest mock files (llm-ops 1043 lines, gist 643, review 620) share patterns. Extract to shared `gunbc-test::mock_helpers` module. | `lib/*/src/graph_mock.rs` | — | M | |
| **CU-4** | **Document side-effect imports**: ~16 `use ... as _;` imports across binary and test files. Add explanatory comments. | `gunbc-dag/src/bin/*.rs` | — | S | |
| **CU-5** | **Archive `design-eliminate-registration-lists.md`**: Phase 1 complete; Phases 2-3 covered by Lanes G/H (done). Move to `TODO/TODONE/`. | `TODO/` | — | S | |
| **CU-6** | **Organize TODONE by quarter**: 65 completed items in flat `TODO/TODONE/`. Create `TODONE/2026-Q1/` subdirectory. | `TODO/TODONE/` | — | S | |
| **CU-7** | **Typed API migration**: Migrate remaining legacy untyped `Port` API to `TypedPort<T>` wrappers. | `lib/*/src/graph.rs` | After Lane 1 TS-1* | L | |
| **CU-8** | **Resource trait string port elimination**: Migrate remaining string `res:*` ports to typed resource system. | `core/exec/`, `gunbc-dag/` | — | L | |
| **CU-9** | **Canonical port naming invariants**: Migrate to one canonical port name per semantic role across lowering, runtime, and snapshots. | Various | — | S | |
| **CU-10** | **TypeRegistry ↔ PortType drift audit**: Verify every domain type in `try_parse_port_type()` (40 mappings) has a consistent `TypeRegistry` registration with matching structural backing. Fix any remaining mismatches like the `Credential→Secret` bug. Stopgap until `port_type.rs` is deleted (TS-4c). | `core/ir/src/{port_type,type_registry}.rs` | — | S | ✅ Done |

---

## Backlog & Deferred

All unscheduled work is in `TODO/backlog.md` — prioritized (P1/P2/P3), reviewed
quarterly, P3 items deleted if not promoted within 2 quarters.

Design docs for backlog items remain in `docs/design/horizon/`.
