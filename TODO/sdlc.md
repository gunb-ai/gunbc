# Lane 3: SDLC Pipeline

**Goal**: Run the SDLC pipeline end-to-end — from issue discovery through design, implementation, code review, testing, and close. This is the objective of all the compiler and infrastructure work.

**Design docs**:
- `docs/design/sdlc/domain-modeling-comprehensive.md` — entity/relationship/state machine model
- `docs/design/sdlc/e2e-gap-analysis.md` — gap resolution (all resolved via DSL)
- `docs/design/sdlc/production-gap-analysis.md` — activation blockers

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
Rust Tests:   ZERO — pipeline has never been compiled through the Rust compiler
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

### What's never been proven

The pipeline is the most ambitious .dag artifact in the repo but has **zero automated compilation proof**. Four known source bugs exist. No Rust integration test has ever compiled any SDLC .dag file.

---

## Phase 0: Prove It Compiles

**Goal**: Get `sdlc_worker.dag` through the compiler with zero errors. This is the prerequisite for everything else.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | S-1 | **Fix import bugs.** Wrong import in `sdlc_worker.dag:34` (`determine_stage` from wrong module). Wrong service names in `pipelines/sdlc.dag` (`Cargo` → `Build`, dotted `git.Core` selector). | All SDLC .dag files parse without import errors. | S | Open |
| 2 | S-2 | **Fix missing type imports in `stub_providers.dag`.** Uses `TrackedIssue`, `IssueEvent`, `Signal`, `SignalType` without imports. | `stub_providers.dag` parses and typechecks. | S | Open |
| 3 | S-3 | **Extract duplicate type definitions.** `CapabilityBehaviorContract` and `CapabilityFailureContract` duplicated across interface files. Extract to shared module. | Zero duplicate definitions. | S | Open |
| 4 | S-4 | **Add Rust compilation test.** `builds_sdlc_worker_dsl_graph()` and `builds_sdlc_stages_dsl_graph()` in `compile_commands.rs`. | SDLC .dag files compile through full pipeline (parse → typecheck → lower). | M | Open |

---

## Phase 1: Local Dry Run

**Goal**: `gunbc sdlc --profile unit_test` works end-to-end with stub providers.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 5 | S-5 | **Add SDLC to workflow catalog.** Register in `config/workflow_catalog.dag` so `gunbc sdlc` command works. | `gunbc sdlc --help` shows SDLC options. | S | Open |
| 6 | S-6 | **Wire profile binding.** Verify `unit_test` profile threads stub providers correctly through resolver. | Compilation with `--profile unit_test` produces executable DAG. | M | Open |
| 7 | S-7 | **Dry-run execution.** `gunbc sdlc --profile unit_test --dry-run` completes all 11 stages with mock data. | All stages execute, outcomes recorded, no runtime errors. | L | Open |
| 8 | S-8 | **Fix dispatch runtime.** `sdlc_dispatch_runtime.dag` returns static records — wire to real stage handlers in `sdlc_stages.dag`. | Dispatch routes to real handler functions. | M | Open |

---

## Phase 2: Local Real Run

**Goal**: `gunbc sdlc --profile local` processes a real GitHub issue through design stage.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 9 | S-9 | **Verify GitHub provider.** `IssueProvider` operations work against real GitHub API (discover, get, create, comment, set_labels). | GitHub operations return expected data. | M | Open |
| 10 | S-10 | **Verify credential wiring.** `GITHUB_TOKEN` flows through `local` profile credential provider to service operations. | Auth token reaches GitHub REST calls. | M | Open |
| 11 | S-11 | **End-to-end local run.** Process one issue from Idea → Design: discover issues, claim, call LLM, post comment, advance labels. | Issue has design comment and `sdlc:design-review` label. | L | Open |

---

## Phase 3: Full Pipeline

**Goal**: Complete pipeline: Idea → Design → DesignReview → Accepted → Implementing → CodeReview → Testing → Done.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 12 | S-12 | **Agent provider wiring.** `codex.AgentProvider` spawns real agent for implementation stage. | Agent creates branch with implementation. | L | Open |
| 13 | S-13 | **Code review wiring.** PR diff retrieval + LLM review produces approval/rejection. | Review comment posted on PR. | M | Open |
| 14 | S-14 | **Testing stage wiring.** `cargo test` + `cargo clippy` execution with result parsing. | Test results determine stage outcome. | M | Open |
| 15 | S-15 | **Multi-stage progression.** Issue moves through all stages without manual intervention. | Issue reaches `Done` with artifacts at each stage. | XL | Open |

---

## Phase 4: Production (cloud_run profile)

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 16 | S-16 | **GCS provider verification.** Claims, outcomes, artifacts stored in GCS with generation-based CAS. | Multi-worker safe operations. | L | Open |
| 17 | S-17 | **Cloud Run deployment.** Deploy worker to Cloud Run via `deploy.dag` infrastructure. | Worker runs as Cloud Run service. | L | Open |
| 18 | S-18 | **Signal delivery.** Pub/Sub signals trigger worker execution. | Worker responds to incoming signals. | M | Open |
| 19 | S-19 | **Multi-worker fleet.** Multiple workers process different issues concurrently without claim conflicts. | CAS prevents double-processing. | L | Open |

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

1. **Phase 0**: All SDLC .dag files compile. Rust integration test proves it.
2. **Phase 1**: `gunbc sdlc --profile unit_test --dry-run` completes all stages.
3. **Phase 2**: One real GitHub issue processed through design stage locally.
4. **Phase 3**: Full pipeline: Idea → Done without manual intervention.
5. **Phase 4**: Multi-worker fleet on Cloud Run processing issues concurrently.
