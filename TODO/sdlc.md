# Lane 3: SDLC Pipeline

> **Reference note (2026-03-05)**: Active SDLC planning now lives in `tasks.md`
> Phase H. This file is a branch-status/reference snapshot, not the source of
> truth for prioritization.

**Goal**: Run the SDLC pipeline end-to-end — from issue discovery through design, implementation, code review, testing, and close. This is the objective of all the compiler and infrastructure work.

**Design docs**:
- `docs/design/sdlc/domain-modeling-comprehensive.md` — entity/relationship/state machine model
- `docs/design/sdlc/execution-intent-binding-plan.md` — `SM-1` / `SM-2` design for reusable execution intent and binding/link modeling
- `docs/design/sdlc/e2e-gap-analysis.md` — historical profile-era gap analysis
- `docs/design/sdlc/production-gap-analysis.md` — historical blocker baseline before current compile/dry-run proof
- `docs/design/sdlc/scenario-readiness.md` — practical go/no-go modes for local and hosted rollout
- `docs/design/sdlc/ambient-intellectual-roadmap.md` — canonical end-to-end roadmap from production SDLC to ambient/intellectual/lifecycle-controlled operation
- `docs/design/sdlc/ambient-feedback-model.md` — canonical GitHub comment/review ingestion and durable feedback-obligation model for ambient SDLC
- `docs/design/modeling/intellectual-pipeline-kernel.md` — future domain-neutral inquiry kernel stress-tested against SDLC, ML, and architecture workflows

> **Planning relationship (2026-03-05)**: `tasks.md` Phase H is now the single
> active planning surface. This file remains useful as a status snapshot and
> reference input when updating that lane.

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
Bindings:     no-profile dry-run path proven; temporary local compatibility profile restored for real-mode proof
Pipeline:     11 stages with real logic (design, review, code review, testing)
Worker:       COMPLETE — discovery, claim, dispatch, record
Handlers:     8 per-stage functions with LLM + CI integration
DSL Tests:    10+ DSL-level tests
Rust Tests:   compile + dry-run proof in `gunbc-dag/tests/compile_commands.rs`; env-gated local live proof in `gunbc-dag/tests/sdlc_phase_live.rs`
```

Branch reality on 2026-03-05:

- `profiles.sdlc.local` exists only as a temporary unblocker for local real-mode SDLC proof and is currently pinned to `gunb-ai/integration_testing`.
- The long-term direction is still to remove profile concepts and replace them with domain-modeled concrete binding/link artifacts.
- Generated and user-facing CLIs no longer expose `--profile`; the temporary compatibility path is currently exercised through Rust tests via `BuildOpts.profile`.
- CI does not continuously prove local real mode; that proof remains env-gated and operator-controlled.

### Tonight's doc lane

1. Make the temporary-vs-target binding split explicit everywhere.
2. Treat current proof surfaces as the baseline: compile tests, worker dry-run, and the env-gated local live harness.
3. Demote older profile-era SDLC docs to historical/reference status instead of letting them silently drive current planning.
4. Flesh out the four operating modes we actually care about: local dev testing, local real testing, remote dev testing, and remote real runs.

### Execution mode map

| Mode | What it is | Current proof | Remaining must-have work |
|------|------------|---------------|--------------------------|
| Local dev testing | Developer-machine compile + dry-run with mocked boundaries and no external mutation | `make ci`; SDLC compile tests; `dispatch_sdlc_dry_run_completes_without_legacy_bindings` | Keep this path green while compiler cleanup lands; do not let real-mode work break no-profile dry-run |
| Local real testing | Developer-machine run against real GitHub/LLM with local file-backed state and explicit mutation opt-in | env-gated `s10_local_profile_binds_real_local_providers`; env-gated `s11_local_profile_design_stage_e2e` | Finish S-9 provider-op proof, make S-11 repeatable, then drive S-12 through S-15 locally before relying on hosted rollout |
| Remote dev testing | Hosted canary in non-prod infra against dev repo/queue with real cloud stores and bounded blast radius | structural wiring and env-gated hosted harnesses for S-16 through S-18 | Concrete hosted binding/link artifacts, single-worker canary proof, deploy/health/signal validation in non-prod project |
| Remote real runs | Hosted fleet processing the real queue/repo with production mutations | none yet | S-19 multi-worker contention proof, hosted rollback/drain procedure, observability, and clear operator ownership |

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

Phase 0 **complete**: the active SDLC .dag files compile through the Rust compiler, and worker dry-run proof passes in `gunbc-dag/tests/compile_commands.rs`. Local real-mode proof is unblocked again through the temporary `profiles.sdlc.local` compatibility path and env-gated tests in `gunbc-dag/tests/sdlc_phase_live.rs`. This is operationally useful, but not the final architecture.

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

**Goal**: SDLC worker dry-run completes end-to-end with auto-mocked boundaries.

> **Prerequisite**: S-4 is green. Do not start until compilation is proven.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 5 | S-5 | **Add SDLC to workflow catalog.** Register in `config/workflow_catalog.dag` so `gunbc sdlc` command works. | `gunbc sdlc --help` shows SDLC options. Entry exists in `dsl/config/workflow_catalog.dag`. | S | Done |
| 6 | S-6 | **Keep the no-profile compile path valid.** Verify the worker graph still compiles and no longer depends on deleted legacy bindings. | `build_dsl_graph("funcs/sdlc_worker.dag", ...)` succeeds in a Rust test, and active graph nodes route through provider auth modules rather than deleted profile-only bindings. | M | Done |
| 7 | S-7 | **Dry-run execution.** `dispatch_sdlc_dry_run_completes_without_legacy_bindings` completes the worker path with auto-mocked boundaries. | Rust integration test exits 0. No runtime panics or unresolved port errors. | L | Done |
| 8 | S-8 | **Fix dispatch runtime.** `sdlc_dispatch_runtime.dag` returns static records — wire to real stage handlers in `sdlc_stages.dag`. | `sdlc_dispatch_runtime.dag` imports and calls functions from `sdlc_stages.dag`. No static placeholder returns. `daglang check` passes on the file. | M | Done |

---

## Phase 2: Local Real Run

**Goal**: The current local compatibility binding path processes a real GitHub issue through design stage.

> **Prerequisite**: S-7 passes (dry-run completes all stages).

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 9 | S-9 | **Verify GitHub provider.** `IssueProvider` operations work against real GitHub API (discover, get, comment, set_labels) under the temporary local binding path. | Env-gated live proof exercises the active local SDLC path against GitHub and validates the resulting issue state. | M | In Progress (covered indirectly by `s11_local_profile_design_stage_e2e`; standalone provider-op sweep still absent on this branch) |
| 10 | S-10 | **Verify local binding wiring.** `profiles.sdlc.local` binds local providers and GitHub auth cleanly enough to compile and execute the worker path. | `build_dsl_graph(..., profile=profiles.sdlc.local)` succeeds in a Rust test, and the graph contains GitHub/file/codex provider nodes. The live harness fetches the GitHub token through the normal Secret Manager path before compiling. | M | Done (env-gated compile/binding test in `gunbc-dag/tests/sdlc_phase_live.rs`) |
| 11 | S-11 | **End-to-end local run.** Process one ephemeral issue from Idea → Design: create issue, claim, call LLM, post comment, advance labels, then close during cleanup. | Ephemeral issue has: (1) design comment from LLM, (2) `sdlc:design` label after one dispatch, (3) closed-state cleanup after the test run. Command exits 0. Outcome ledger has entry for the stage run. | L | In Progress (env-gated live e2e harness added; deletion is not part of the current GitHub Issues API path) |

---

## Phase 3: Full Pipeline

**Goal**: Complete pipeline: Idea → Design → DesignReview → Accepted → Implementing → CodeReview → Testing → Done.

> **Prerequisite**: S-11 passes (one stage works end-to-end locally).

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 12 | S-12 | **Agent provider wiring.** `codex.AgentProvider` spawns real agent for implementation stage. | Agent creates branch with implementation. Branch exists on remote. PR created. | L | In Progress (structural wiring + env-gated live harness added) |
| 13 | S-13 | **Code review wiring.** PR diff retrieval + LLM review produces approval/rejection. | Review comment posted on PR with structured findings. Label updated based on approval/rejection. | M | In Progress (structural wiring + env-gated live harness added; LLM content extraction fixed: `review_response.content \|> first() \|> .text`) |
| 14 | S-14 | **Testing stage wiring.** `cargo test` + `cargo clippy` execution with result parsing. | Test results parsed into structured outcome. Pass → advance to Done. Fail → record failure reason. | M | In Progress (structural wiring + env-gated live harness added; git checkout of PR branch added before test/clippy) |
| 15 | S-15 | **Multi-stage progression.** Issue moves through all stages without manual intervention. | Issue starts at `sdlc:idea`, ends at `sdlc:done`. Each stage has artifacts in outcome ledger. No manual label changes required. | XL | In Progress (env-gated live progression harness added; requires secrets + mutable test issue) |

---

## Phase 4: Production (hosted concrete bindings)

> **Prerequisite**: S-15 passes (full pipeline works locally).

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 16 | S-16 | **GCS provider verification.** Claims, outcomes, artifacts stored in GCS with generation-based CAS. | Integration test (gated by GCP credentials) exercises claim/release/outcome operations against real GCS. Generation-based CAS prevents double-writes. | L | In Progress (hosted-provider structural wiring + env-gated CAS/live harness added) |
| 17 | S-17 | **Cloud Run deployment.** Deploy worker to Cloud Run via `deploy.dag` infrastructure. | `gcloud run services describe gunbc-sdlc-worker` returns running service. Health check endpoint returns 200. | L | In Progress (env-gated deploy/health live harness added) |
| 18 | S-18 | **Signal delivery.** Pub/Sub signals trigger worker execution. | Publishing a test signal to the topic triggers a worker execution. Worker log shows signal received and processed. | M | In Progress (env-gated Pub/Sub + log verification harness added) |
| 19 | S-19 | **Multi-worker fleet.** Multiple workers process different issues concurrently without claim conflicts. | 3+ workers processing 5+ issues. No double-processing (CAS prevents). All issues reach expected stage. | L | In Progress (env-gated parallel CAS contention harness added) |

---

## Provider Implementation Matrix

| Interface | Dry-run / unit-test intent | Local compatibility path | Hosted target | Notes |
|-----------|------------------------|---------------------------|---------------|-------|
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
- **Concrete binding cleanup still pending**: the temporary `profiles.sdlc.local` compatibility path is still required for local real-mode proof. AUTH-4 and DM-3A track the compiler cleanup needed to remove it.

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

## Backlog (not Day 1)

These items are intentionally deferred until the core SDLC pipeline is
operational end-to-end. They improve trust and ambient operation, but they are
not part of the Day 1 activation gate.

| ID | Item | Size | Priority |
|----|------|------|----------|
| B1 | Ambient feedback loop for issue/PR comments and review threads. Ingest human feedback as durable obligations with typed signals, ledger-backed tracking, worker rediscovery, and explicit resolved/responded state. A stray reviewer comment should become a first-class work item, not best-effort text. | L | P2 | **In Progress**: AS2 (types), AS3 (interface+providers+profiles), AS4 (ingestion+classification), AS5 (response) DONE. AS6 (classification rules) DONE. Remaining: AS1 (signal-aware worker), AS7 (reports), AS8 (soak test). |
| B2 | Generalize SDLC into a reusable intellectual pipeline kernel. Treat software delivery as one specialization of a broader hypothesis -> execution -> evidence -> critique -> revision -> conclusion loop so the control plane can later support research, architecture, and other knowledge-work workflows without re-deriving the core orchestration model. Design reference: `docs/design/modeling/intellectual-pipeline-kernel.md`. | L | P2 | **In Progress**: IK1 (inquiry types), IK2 (kernel artifacts), IK3 (SDLC-kernel mapping), IK4 (ML exemplar), IK5 (intent expansion) DONE. Remaining: IK6 (runtime proof), IK7 (evidence typing), IK8 (migration doc). |
| B3 | Operational drain / disable / destroy contract. Model how to stop intake, stop signal ingress, stop claim acquisition, drain in-flight work, disable selected pipeline sections, and optionally deprovision infrastructure with explicit verify-absent semantics rather than ad-hoc deletes. Design reference: `docs/design/horizon/h12-managed-lifecycle-control.md`. | L | P1 | **In Progress**: LC1 (parser: `managed` keyword + AST + parser), LC2 (lifecycle types), LC3 (ensure_absent pattern) DONE. Remaining: LC4 (lower/IR), LC5 (codegen), LC6 (testgen), LC7 (SDLC application), LC8 (cleanup). |

### B1 Acceptance Shape

1. GitHub issue comments, PR comments, and review threads map to typed feedback events with stable idempotency keys.
2. Webhook/signal loss delays feedback handling but does not lose it; anti-entropy scans rediscover unresolved feedback.
3. Feedback is persisted as an outstanding obligation until the pipeline posts a linked response artifact or code change outcome.
4. The system can distinguish `seen`, `in_progress`, `addressed`, and `closed` states for each feedback item.
5. Human-visible pipeline responses link back to the originating comment/review so closure is auditable.

### B2 Acceptance Shape

1. Domain-neutral core concepts are explicit: problem statement, hypothesis/design, execution, evidence, critique, revision, conclusion.
2. SDLC stage names remain a specialization layer, not the canonical internal ontology.
3. GitHub issue/PR surfaces are adapters over the core model, not the model itself.
4. Retry, replay, claim, approval, feedback, and artifact contracts stay reusable across non-SDLC workflows.
5. At least one non-SDLC exemplar is modeled before calling the abstraction complete.

### B3 Acceptance Shape

1. Shutdown is modeled in layers: intake disable, signal ingress disable, worker drain, stage/lane disable, and infrastructure teardown are distinct operations.
2. Drain is durable and reversible: workers stop acquiring new claims, release worker-owned claims, and exit with machine-readable status.
3. Graceful and brutal destroy are distinct paths: graceful means `disable -> drain -> destroy -> verify_absent`, brutal means explicit immediate `destroy -> verify_absent`.
4. Destroy is never implicit. Destructive teardown requires explicit intent and runs only after drain/ownership preconditions pass unless brutal destroy is explicitly requested.
5. The DSL gains a first-class verify-absent/ensure-absent shape for codegen and infrastructure cleanup instead of relying on ad-hoc deletes.
6. The system can turn off an arbitrary section of the pipeline without corrupting ledgers, orphaning claims, or losing auditability.

## Program Extension Roadmap

The canonical roadmap for the next wave of work lives in:

- `docs/design/sdlc/ambient-intellectual-roadmap.md`

That document resolves the major design decisions now and breaks the work into
four tracks:

1. finish production SDLC,
2. ambient trusted SDLC,
3. intent-driven intellectual kernel,
4. language-level managed lifecycle control.

---

## Success Criteria

1. **Phase 0**: All SDLC .dag files compile. Rust integration test proves it. **This gates everything.**
2. **Phase 1**: `dispatch_sdlc` dry-run completes on the no-profile compile path with auto-mocked boundaries.
3. **Phase 2**: One real GitHub issue is processed through design stage via the temporary local compatibility binding path.
4. **Phase 3**: Full pipeline: Idea → Done without manual intervention.
5. **Phase 4**: Multi-worker fleet on Cloud Run processing issues concurrently.
