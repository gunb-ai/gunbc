# Lane 3: SDLC Pipeline

**Goal**: Run the SDLC pipeline end-to-end — from issue discovery through design, implementation, code review, testing, and close. This is the objective of all the compiler and infrastructure work.

**Design docs**:
- `docs/design/sdlc/domain-modeling-comprehensive.md` — entity/relationship/state machine model
- `docs/design/sdlc/e2e-gap-analysis.md` — gap resolution (all resolved via DSL)
- `docs/design/sdlc/production-gap-analysis.md` — activation blockers

> **Lesson**: Prove compilation before building infrastructure. The SDLC pipeline was built 3 times
> (Rust binary → DSL pipeline → deleted → rebuilt as 20 .dag files) with elaborate cloud infra
> (claims, CAS, GCS, Cloud Run) layered on top — but zero compilation proof ever existed.
> **Phase 0 is a hard gate.** No Phase 1 work until S-4 passes in CI.

---

## Current State

### Maturity Assessment

```
Interfaces:   7/7  COMPLETE — fully specified with behavioral contracts
Providers:    9/18 COMPLETE, 6/18 PARTIAL, 3/18 STUB
Profiles:     3/3  COMPLETE — unit_test, local, cloud_run
Pipeline:     11 stages with real logic (design, review, code review, testing)
Worker:       COMPLETE — discovery, claim, dispatch, record
Handlers:     8 per-stage functions with LLM + CI integration
DSL Tests:    10+ DSL-level tests
Rust Tests:   11 compile + 1 dry-run in `gunbc-dag/tests/compile_commands.rs`
```

### What works (on paper)

- **Issue management**: GitHub provider, 7 operations, REST transport
- **Claims**: File + GCS implementations, generation-based CAS, heartbeat/release
- **Outcomes**: File + GCS ledgers, idempotent upsert
- **Design stage**: LLM call → comment posting → label transitions
- **Design review**: LLM review → approval gate → conditional advancement
- **Code review**: Real PR diff retrieval + LLM review
- **Acceptance testing**: `cargo test` + `cargo clippy` with result parsing
- **Deployment infra**: GCS buckets, Cloud Run services, IAM roles, WIF credentials

### Compilation status

Phase 0 **complete**: all SDLC .dag files compile through the Rust compiler. 11 compilation tests + 1 dry-run execution test pass in `gunbc-dag/tests/compile_commands.rs`. Profile binding (unit_test, local, cloud_run) verified. Phase 1 (local dry run) complete. Phase 2 in progress.

---

## Phase 0: Prove It Compiles

**Goal**: Get `sdlc_worker.dag` through the compiler with zero errors. This is the prerequisite for everything else.

> **HARD GATE**: Phase 1 work MUST NOT start until S-4 is green in CI. Building features on
> unproven compilation is the #1 source of wasted work in this project's history.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | S-1 | **Fix import bugs.** Wrong import in `sdlc_worker.dag:34` (`determine_stage` from wrong module). Wrong service names in `pipelines/sdlc.dag` (`Cargo` → `Build`, dotted `git.Core` selector). | `daglang check dsl/pipelines/sdlc_worker.dag` and `daglang check dsl/pipelines/sdlc.dag` exit 0 with zero import errors. | S | Done |
| 2 | S-2 | **Fix missing type imports in `stub_providers.dag`.** Uses `TrackedIssue`, `IssueEvent`, `Signal`, `SignalType` without imports. | `daglang check dsl/profiles/stub_providers.dag` exits 0. All referenced types have explicit imports. | S | Done |
| 3 | S-3 | **Extract duplicate type definitions.** `CapabilityBehaviorContract` and `CapabilityFailureContract` duplicated across interface files. Extract to shared module. | `grep -c 'type CapabilityBehaviorContract' dsl/interfaces/` returns 1. `grep -c 'type CapabilityFailureContract' dsl/interfaces/` returns 1. Shared module exists (e.g., `dsl/interfaces/shared.dag`). | S | Done |
| 4 | S-4 | **Add Rust compilation test.** `builds_sdlc_worker_dsl_graph()` and `builds_sdlc_stages_dsl_graph()` in `compile_commands.rs`. | Two new `#[test]` functions in `gunbc-dag/tests/compile_commands.rs`. Both call `build_dsl_graph()` for SDLC modules and assert `Ok`. `cargo test -p gunbc-dag -- builds_sdlc` passes. **This is the gate for Phase 1.** | M | Done |

---

## Phase 1: Local Dry Run

**Goal**: `gunbc sdlc --profile unit_test` works end-to-end with stub providers.

> **Prerequisite**: S-4 is green. Do not start until compilation is proven.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 5 | S-5 | **Add SDLC to workflow catalog.** Register in `config/workflow_catalog.dag` so `gunbc sdlc` command works. | `gunbc sdlc --help` shows SDLC options. Entry exists in `dsl/config/workflow_catalog.dag`. | S | Done |
| 6 | S-6 | **Wire profile binding.** Verify `unit_test` profile threads stub providers correctly through resolver. | `build_dsl_graph_with_profile("pipelines.sdlc_worker", "unit_test")` succeeds in a Rust test. All 7 interface bindings resolve to stub impls. | M | Done |
| 7 | S-7 | **Dry-run execution.** `gunbc sdlc --profile unit_test --dry-run` completes all 11 stages with mock data. | Command exits 0. Output contains stage names for all 11 stages. No runtime panics or unresolved port errors. | L | Done |
| 8 | S-8 | **Fix dispatch runtime.** `sdlc_dispatch_runtime.dag` returns static records — wire to real stage handlers in `sdlc_stages.dag`. | `sdlc_dispatch_runtime.dag` imports and calls functions from `sdlc_stages.dag`. No static placeholder returns. `daglang check` passes on the file. | M | Done |

---

## Phase 2: Local Real Run

**Goal**: `gunbc sdlc --profile local` processes a real GitHub issue through design stage.

> **Prerequisite**: S-7 passes (dry-run completes all stages).

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 9 | S-9 | **Verify GitHub provider.** `IssueProvider` operations work against real GitHub API (discover, get, create, comment, set_labels). | Integration test (gated by `GITHUB_TOKEN` env var) exercises all 7 IssueProvider operations. Each returns expected data shape. | M | Done (env-gated live test in `gunbc-dag/tests/sdlc_phase_live.rs`) |
| 10 | S-10 | **Verify credential wiring.** `GITHUB_TOKEN` flows through `local` profile credential provider to service operations. | `build_dsl_graph_with_profile("pipelines.sdlc_worker", "local")` succeeds. Running with `GITHUB_TOKEN` set produces authenticated API calls (not 401). | M | Done (env-gated compile+auth test in `gunbc-dag/tests/sdlc_phase_live.rs`) |
| 11 | S-11 | **End-to-end local run.** Process one issue from Idea → Design: discover issues, claim, call LLM, post comment, advance labels. | Target issue has: (1) design comment from LLM, (2) `sdlc:design-review` label. Command exits 0. Outcome ledger has entry for the stage run. | L | In Progress (env-gated live e2e harness added; requires secrets + mutable test issue) |

---

## Phase 3: Full Pipeline

**Goal**: Complete pipeline: Idea → Design → DesignReview → Accepted → Implementing → CodeReview → Testing → Done.

> **Prerequisite**: S-11 passes (one stage works end-to-end locally).

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 12 | S-12 | **Agent provider wiring.** `codex.AgentProvider` spawns real agent for implementation stage. | Agent creates branch with implementation. Branch exists on remote. PR created. | L | In Progress (structural wiring + env-gated live harness added) |
| 13 | S-13 | **Code review wiring.** PR diff retrieval + LLM review produces approval/rejection. | Review comment posted on PR with structured findings. Label updated based on approval/rejection. | M | In Progress (structural wiring + env-gated live harness added) |
| 14 | S-14 | **Testing stage wiring.** `cargo test` + `cargo clippy` execution with result parsing. | Test results parsed into structured outcome. Pass → advance to Done. Fail → record failure reason. | M | In Progress (structural wiring + env-gated live harness added) |
| 15 | S-15 | **Multi-stage progression.** Issue moves through all stages without manual intervention. | Issue starts at `sdlc:idea`, ends at `sdlc:done`. Each stage has artifacts in outcome ledger. No manual label changes required. | XL | In Progress (env-gated live progression harness added; requires secrets + mutable test issue) |

---

## Phase 4: Production (cloud_run profile)

> **Prerequisite**: S-15 passes (full pipeline works locally).

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 16 | S-16 | **GCS provider verification.** Claims, outcomes, artifacts stored in GCS with generation-based CAS. | Integration test (gated by GCP credentials) exercises claim/release/outcome operations against real GCS. Generation-based CAS prevents double-writes. | L | In Progress (cloud_run structural wiring + env-gated CAS/live harness added) |
| 17 | S-17 | **Cloud Run deployment.** Deploy worker to Cloud Run via `deploy.dag` infrastructure. | `gcloud run services describe gunbc-sdlc-worker` returns running service. Health check endpoint returns 200. | L | In Progress (env-gated deploy/health live harness added) |
| 18 | S-18 | **Signal delivery.** Pub/Sub signals trigger worker execution. | Publishing a test signal to the topic triggers a worker execution. Worker log shows signal received and processed. | M | In Progress (env-gated Pub/Sub + log verification harness added) |
| 19 | S-19 | **Multi-worker fleet.** Multiple workers process different issues concurrently without claim conflicts. | 3+ workers processing 5+ issues. No double-processing (CAS prevents). All issues reach expected stage. | L | In Progress (env-gated parallel CAS contention harness added) |

---

## Provider Implementation Matrix

| Interface | unit_test | local | cloud_run | Notes |
|-----------|-----------|-------|-----------|-------|
| IssueProvider | stub (in-memory) | GitHub REST | GitHub REST | 7 operations |
| ClaimStore | stub (in-memory) | file (JSON + OS lock) | GCS (generation CAS) | Multi-worker via CAS |
| OutcomeLedger | stub (in-memory) | file (JSON) | GCS (JSON objects) | Idempotent upsert |
| AgentProvider | stub (no-op) | Codex CLI | Codex CLI | Shell-based |
| SignalStore | stub (in-memory) | file (JSON queue) | Pub/Sub | At-least-once delivery |
| ArtifactStore | stub (in-memory) | inline (issue comments) | GCS (objects) | Two-phase commit |
| CredentialProvider | stub (static) | local (env vars) | GCP WIF | Workload Identity |

## Known Compiler Limitations Impacting SDLC

- **Parameterized headers**: GCS CAS needs `x-goog-if-generation-match` injected from input fields. Compiler only supports static headers. Rust worker must construct at runtime.
- **Cross-callable data flow**: Service call arguments referencing callable parameters have limited wiring. Workaround: inline service calls into one func.
- **Same-module extern func wiring**: Calls to `extern func` from same module break codegen. Workaround: cross-module pattern.

## Transport Completeness (supporting work)

| ID | Scope | Ops | Notes |
|----|-------|-----|-------|
| RF-TC4 | Stub providers (unit_test profile) | 28 | Consider `transport stub {}` marker |
| RF-TC5 | Infrastructure stubs (azure, aws, gcp-infra) | 140 | Defer until provisioning lane |

## Future (post-pipeline)

| ID | Item | Size | Priority |
|----|------|------|----------|
| H10 | Compute stack orchestration: Cloud Run/GCS/LB lifecycle DAG builder | L | P2 |
| S12-E | Multi-worker CAS: GcsClaimStore with generation-based CAS | M | P2 |

---

## Success Criteria

1. **Phase 0**: All SDLC .dag files compile. Rust integration test proves it. **This gates everything.**
2. **Phase 1**: `gunbc sdlc --profile unit_test --dry-run` completes all stages.
3. **Phase 2**: One real GitHub issue processed through design stage locally.
4. **Phase 3**: Full pipeline: Idea → Done without manual intervention.
5. **Phase 4**: Multi-worker fleet on Cloud Run processing issues concurrently.
