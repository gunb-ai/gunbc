# SDLC Production Gap Analysis

Status: Active
Date: 2026-03-03
Parent: [implementation-roadmap.md](implementation-roadmap.md)

## 1. Executive Summary

The SDLC pipeline has **3,616 lines of DSL code across 20+ files** — interfaces,
providers, profiles, a full 11-stage pipeline, a worker dispatch loop, and per-stage
handlers. The DSL compiler infrastructure (C24, C29, C30, CT-8) is production-grade.

**None of it has been proven to work end-to-end.**

There is no compilation test, no dry-run execution test, and no profile binding
verification. The SDLC pipeline is the most ambitious DSL artifact in the repo but
the only one with zero automated proof of correctness.

For comparison, every working tool (`makegen`, `pragma`, `gist`, `ci`, `clippy`,
`bootstrap`, `deps`, `codegen`, `infra`, `review`) has a `builds_*_dsl_graph()` test
in `dsl_builder.rs`. SDLC has none.

## 2. What Exists Today

### 2.1 DSL Artifacts (complete scaffolding)

| Layer | Files | Lines | Status |
|-------|-------|-------|--------|
| Interfaces (7) | `interfaces/*.dag` | ~373 | Fully specified — capabilities, behavioral contracts, failure modes |
| Providers (16) | `services/sdlc/providers/*.dag` | ~1,235 | Stubs through full REST/Shell impls |
| Profiles (3) | `profiles/sdlc.dag` | 109 | unit_test, local, cloud_run — all 7 interfaces bound |
| Pipeline | `pipelines/sdlc.dag` | 536 | 11 stages: fetch → close + report. DSL tests inline. |
| Worker | `funcs/sdlc_worker.dag` | 382 | Discovery → claim → dispatch → record loop. 4 DSL tests. |
| Stage handlers | `funcs/sdlc_stages.dag` | 701 | 8 handlers (idea→design through done). 5 DSL tests. |
| State machine | `std/state_machines.dag` | 201 | Ordinal, validation, label encode/decode, backward targets |
| Types | `std/types.dag` (SDLC section) | ~200 | IssueLifecycleStage, TrackedIssue, StageOutcome, etc. |
| Dispatch runtime | `funcs/sdlc_dispatch_runtime.dag` | 104 | **Hardcoded stubs** — each func returns a static record |
| Validation runtime | `funcs/sdlc_validation_runtime.dag` | 59 | Real logic — review_gate and ci_gate work |
| Workflow entry | `workflows/sdlc.dag` | 60 | Pipeline: compilation → codegen → intake → worker → report |

### 2.2 Rust Infrastructure

| Component | Status |
|-----------|--------|
| `workflow/catalog.rs` — sdlc.dag embedded as `WF_SDLC` | Present — included in embedded sources |
| `workflow_catalog.dag` — SDLC variant entry | **MISSING** — sdlc not registered |
| `InterfaceStub` transport class in lowerer | Working — generates stub ops for all capabilities |
| `strip_pipeline_nodes()` in dsl_builder | Working — strips `LoweredOp::Pipeline` before resolution |
| `build_dsl_graph_with_profile()` | Working — threads profile through compilation |
| `compile_lowered_with_profile()` | Working — `CompileOptions.profile` consumed |

### 2.3 Working Compilation Paths

The infrastructure exists to compile and resolve SDLC DAGs. These paths are proven
for other tools:

```
# Path A: func entrypoint (sdlc_worker, sdlc_stages)
build_dsl_graph_for_entrypoint("funcs/sdlc_worker.dag", Some("dispatch_sdlc"))

# Path B: full module (pipelines/sdlc.dag)
build_dsl_graph("pipelines/sdlc.dag")

# Path C: with profile (unit_test profile binding)
build_dsl_graph_with_profile("funcs/sdlc_worker.dag", "unit_test")
```

**These paths have never been called for SDLC files.**

## 3. Gap Inventory

Task IDs reference `tasks.md` § "SDLC Production Activation".

### Gap 0: Known DSL Source Bugs (MUST FIX BEFORE COMPILATION)

Four concrete bugs in the `.dag` source files prevent compilation:

**G0-a: Wrong import in `sdlc_worker.dag`** (task P0-1)
Line 34: `import funcs.sdlc_stages { determine_stage, execute_stage }`.
`determine_stage` is defined in `std.state_machines`, not `funcs.sdlc_stages`.
The stages module *imports* it but does not re-export it.

**G0-b: Wrong service name in `pipelines/sdlc.dag`** (task P0-2)
Line 36: `import services.cargo { Cargo }` — the service is `cargo.Build`, not `Cargo`.
Line 37: `import services.git { git.Core }` — dotted selector in braces is non-standard.
Call sites `Cargo.Test()`, `Cargo.Clippy()` (lines 406, 409) also wrong.

**G0-c: Missing type imports in `stub_providers.dag`** (task P0-3)
Uses `TrackedIssue`, `IssueEvent`, `Signal`, `SignalType` in operation signatures
without importing them.

**G0-d: Duplicate type definitions across interfaces** (task P0-4)
`CapabilityBehaviorContract` and `CapabilityFailureContract` are identically defined
in both `issue_provider.dag` and `claim_store.dag` (and likely `agent_provider.dag`,
`outcome_ledger.dag`, etc.). Should be extracted to a shared module.

### Gap 1: Zero Compilation Proof (CRITICAL)

Tasks: P1-1, P1-2, P1-3, P1-4.

No test calls `build_dsl_graph*()` on any SDLC `.dag` file. We don't know if
the SDLC modules compile through the lowerer, let alone resolve to `DynOp` or
execute in dry-run.

**Risk**: Type errors, import resolution failures, missing sum type variants,
lowerer limitations (cross-callable data flow, transport node deduplication) —
any of these could exist silently. Gap 0 items are known examples.

**Fix**: Add `builds_sdlc_worker_dsl_graph()` and `builds_sdlc_stages_dsl_graph()`
to `dsl_builder.rs`. This is the first thing to do — it tells us what actually breaks.

### Gap 2: Not in Workflow Catalog (BLOCKING)

Task: P3-1 or P3-2.

`dsl/config/workflow_catalog.dag` lists 9 workflows: ci, test-all, gist (3 modes),
bootstrap, makegen, pragma, deps, build-all. **SDLC is not listed.**

Without a catalog entry, `gunbc sdlc` cannot be invoked via the planner. The workflow
planner uses `resolve_workflow_variant("sdlc")` which returns `None`.

**Fix**: Add sdlc variant to `workflow_catalog.dag`. But this raises a question:
should SDLC use the planner path (which builds a `WorkflowSpec` from stage templates)
or the direct compilation path (which compiles `funcs/sdlc_worker.dag` as a func)?

**Decision needed**: The planner extracts stages from `pipeline` blocks in `.dag` files
and wires them via `ProcessUnitRef`. But the SDLC pipeline's real logic is in
`sdlc_worker.dag::dispatch_sdlc()` (a `func`, not a `pipeline`). The
`workflows/sdlc.dag` pipeline is a thin wrapper that calls `dispatch_sdlc()`. Two
options:

1. **Planner path**: Register in catalog, `workflows/sdlc.dag` pipeline stages map to
   process units, planner orchestrates compilation→codegen→intake→worker→report.
2. **Direct path**: Skip planner, compile `funcs/sdlc_worker.dag` directly as a
   func entrypoint, invoke via generated CLI binary (like `deps_config`, `workflow`).

Recommendation: **Direct path first** (simpler, proves the core logic), planner
path later for multi-profile deployment.

### Gap 3: Dispatch Runtime is Hardcoded (MODERATE)

Task: CL-1.

`sdlc_dispatch_runtime.dag` has 6 `func` items that each return a static record:
```
func dispatch_idea(...) -> { next_stage: "design", awaiting_approval: false, ... }
```

These are dummy stubs. The real dispatch logic lives in `sdlc_stages.dag::execute_stage()`
which does a full match-dispatch to per-stage handlers. The dispatch_runtime.dag is
**dead code** — the worker calls `execute_stage()`, not `dispatch_idea()`.

**Fix**: Delete `sdlc_dispatch_runtime.dag` — its functionality is already
superseded by `sdlc_stages.dag`. The validation_runtime.dag has real logic and
should be kept.

### Gap 4: Pipeline vs Func Architecture Split

Task: CL-7.

Two execution architectures coexist:

| Path | File | Approach |
|------|------|----------|
| Pipeline | `pipelines/sdlc.dag` | 11-stage `pipeline` block, inline service calls per stage |
| Worker func | `funcs/sdlc_worker.dag` + `sdlc_stages.dag` | Discovery loop, claim/release, match-dispatch to handlers |

The pipeline path is a monolithic 536-line pipeline with all logic inline. The worker
path is modular: worker does discovery/claim/record, stages do per-stage business logic.

**The worker path is architecturally correct** — it's the one with claim acquisition,
replay-skip, retry budget awareness, and the modular handler structure. The pipeline
path would need to be completely restructured to support these concerns.

**Recommendation**: The `pipelines/sdlc.dag` pipeline is aspirational documentation
(shows the full stage chain). The `funcs/sdlc_worker.dag` + `sdlc_stages.dag` pair
is the execution target. Promote the worker path; demote the pipeline to reference.

### Gap 5: Interface Resolution at Execution Time

Tasks: P4-1, P4-2, P4-3.

The 7 interfaces (IssueProvider, ClaimStore, OutcomeLedger, AgentProvider, SignalStore,
ArtifactStore, CredentialProvider) compile to `InterfaceStub` transport class by default.
In DryRun mode, `InterfaceStubExecuteOp` returns mock responses per the mock spec.

For **real execution** (local/cloud_run profiles), the lowerer needs to resolve
interface capabilities to concrete service operations via profile bindings. The
`CompileOptions.profile` field exists and threads through to lowering, but:

- Profile binding resolution in the lowerer (`resolve_profile_bindings()`) — needs verification
- Provider `.dag` files may not compile (REST config, auth wiring)
- Credential bridging (`env("GITHUB_TOKEN")`, `secret("github-token")`) — mechanism exists but untested for SDLC providers

**Fix**: Incremental — first get DryRun working (InterfaceStub), then test `unit_test`
profile, then `local` profile with real credentials.

### Gap 6: Cross-Module Func Calls in Worker

Tasks: P1-4, P2-1.

`sdlc_worker.dag` imports and calls `execute_stage()` from `sdlc_stages.dag`.
`execute_stage()` internally calls 8 per-stage handler funcs (same module).

Known lowerer limitations that may bite:
- **Cross-callable data flow**: Partially fixed — `wire_fn_call_arguments()` now
  handles ident/literal/field_access args. But complex expressions (match results,
  service call outputs) may not wire correctly.
- **Transport node deduplication**: One prepare/execute/parse triplet per service
  operation per module. Multiple stage handlers calling the same service (e.g.,
  `issues.comment()` called in 8 handlers) could hit duplicate scalar port conflicts.

**Fix**: Compile and diagnose. If transport deduplication hits, the workaround is
to consolidate service calls (already known pattern from docs/design/v4).

### Gap 7: Missing Agent Provider Execution

The `implementing` stage spawns an agent (`agents.spawn()`) and polls it
(`agents.poll()`). In DryRun, InterfaceStubExecuteOp returns mock responses.
For real execution:

- `codex_agent_provider.dag` — needs to map to actual Codex CLI or API calls
- `llm_agent_provider.dag` — alternative LLM-based implementation
- Agent lifecycle (spawn → poll → get_result) is inherently async/long-running

The SDLC worker's one-shot invocation model ("one stage transition per issue per
invocation") handles this: spawn returns immediately, next invocation polls.
But this has never been tested.

### Gap 8: Generated Tests Not Wired

Tasks: P6-1, CL-5.

`generated_tests_pipelines_sdlc.rs` (1.1MB, 30k+ lines) exists but is NOT in the
module tree. Zero tests execute. This is the largest generated test file in the repo
and provides zero value until included.

**Fix**: Add to module tree or regenerate + include. But first, the compilation
must work (Gap 1).

## 4. Execution Plan: SDLC to Production

### Phase 0: Prove Compilation (1 day)

| Step | Action | Expected outcome |
|------|--------|------------------|
| 0.1 | Add `builds_sdlc_worker_dsl_graph()` test | Compile `funcs/sdlc_worker.dag` → `dispatch_sdlc` entrypoint |
| 0.2 | Add `builds_sdlc_stages_dsl_graph()` test | Compile `funcs/sdlc_stages.dag` → `execute_stage` entrypoint |
| 0.3 | Add `builds_sdlc_pipeline_dsl_graph()` test | Compile `pipelines/sdlc.dag` (full pipeline) |
| 0.4 | Fix all compilation errors from 0.1-0.3 | Green tests |

**Outcome**: We know exactly what compiles and what doesn't.

### Phase 1: DryRun Worker Execution (2-3 days)

| Step | Action | Expected outcome |
|------|--------|------------------|
| 1.1 | Execute `dispatch_sdlc` in DryRun with mocked interfaces | Worker discovers issues, claims, dispatches, records |
| 1.2 | Execute `execute_stage` per-stage in DryRun | Each handler returns correct next_stage |
| 1.3 | Fix lowerer/resolver issues (transport dedup, data flow) | All 8 stage handlers execute |
| 1.4 | Delete `sdlc_dispatch_runtime.dag` (dead code) | Cleaner module set |

**Outcome**: `dispatch_sdlc()` runs end-to-end in DryRun with mock responses.

### Phase 2: Catalog + CLI Integration (1 day)

| Step | Action | Expected outcome |
|------|--------|------------------|
| 2.1 | Add `sdlc` to `workflow_catalog.dag` | `gunbc workflow plan sdlc` works |
| 2.2 | OR: Create `dsl/tools/sdlc.dag` with `func sdlc_run(...)` wrapper | `gunbc-sdlc --profile unit_test` works |
| 2.3 | Add generated binary entry to `Cargo.toml` | CLI entrypoint available |

**Outcome**: Invocable from command line.

### Phase 3: Profile Binding Verification (2-3 days)

| Step | Action | Expected outcome |
|------|--------|------------------|
| 3.1 | Compile with `--profile unit_test` | All interfaces resolve to stub providers |
| 3.2 | Execute unit_test profile hermetically | Full worker cycle with all-stub providers |
| 3.3 | Compile with `--profile local` | Interfaces resolve to file/GitHub providers |
| 3.4 | Fix credential bridging (`env("GITHUB_TOKEN")`) | Auth wiring works |

**Outcome**: `gunbc sdlc --profile unit_test` passes hermetically.

### Phase 4: Local Integration (2-3 days)

| Step | Action | Expected outcome |
|------|--------|------------------|
| 4.1 | Test against real GitHub repo (test repo, not production) | idea → design transition |
| 4.2 | Multi-invocation: idea → design → review → accepted | 3 stage transitions |
| 4.3 | Agent spawn (optional — can stub for initial release) | Implementation stage works |
| 4.4 | Full cycle: idea → done | End-to-end local validation |

**Outcome**: `gunbc sdlc --profile local --repo owner/name` processes real issues.

### Phase 5: Production Hardening (ongoing)

| Step | Action |
|------|--------|
| 5.1 | Wire generated tests into module tree |
| 5.2 | Cloud Run profile deployment (GCS providers) |
| 5.3 | Multi-worker CAS stress testing |
| 5.4 | Retry budget + backward transition e2e testing |

## 5. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Compilation fails immediately | High | Low | Phase 0 finds issues quickly |
| Transport node deduplication blocks multi-handler modules | Medium | High | Consolidate service calls per handler |
| Cross-callable data flow breaks worker→stage dispatch | Medium | High | Known limitation — workaround: inline |
| Profile binding resolution untested | Medium | Medium | Incremental: DryRun → unit_test → local |
| Agent lifecycle (spawn/poll) too complex for v1 | Low | Medium | Stub agents for v1, real agents for v2 |

## 6. What the Compiler Work (C24, C29, C30) Actually Enables

| Feature | SDLC benefit |
|---------|-------------|
| C24 (Pure Dataflow) | `MatchDispatch` handles `execute_stage()` match on `IssueLifecycleStage`. `StringInterpolate` handles comment body formatting. `ConditionalOp` handles `if claim.acquired { ... }`. These were previously `ExprCompute` blobs. |
| C29 (Output Shape) | GitHub REST responses extract fields by json_path. `pr_result.number`, `agent_result.exit_code` work correctly. |
| C30 (Type-Aware Bridging) | `IssueLifecycleStage` enum values round-trip through JSON correctly. Previously silently became strings. |
| CT-8 (Contract Tests) | Interface behavioral contracts generate real test obligations. |
| C22 (Redundancy Detection) | Multiple `issues.comment()` calls across stages — fingerprinting catches accidental duplication. |

**Bottom line**: The compiler work was necessary infrastructure. Without C24,
the SDLC worker's `match stage { ... }` dispatch would be an opaque ExprCompute
blob. Without C30, enum stage values would corrupt on JSON round-trip. But none
of this was validated against SDLC specifically.

## 7. Cleanup Inventory

Items that aren't blocking but should be addressed for hygiene. All tracked as
CL-* tasks in `tasks.md`.

| Item | File | Issue | Task |
|------|------|-------|------|
| Dead dispatch runtime | `funcs/sdlc_dispatch_runtime.dag` | Superseded by `sdlc_stages.dag::execute_stage()`. Nothing imports it. | CL-1 |
| Stale design copy | `docs/design/sdlc/design.dag` | Copy of `dsl/tools/design.dag` misplaced in docs dir. | CL-2 |
| Misleading doc status | `e2e-gap-analysis.md` | Header claims "Complete — All gaps resolved." Not true. | CL-3 |
| Stale roadmap claims | `implementation-roadmap.md` | testing→done "missing" (exists), validation_runtime "Stub" (has logic), provider count wrong. | CL-4 |
| Orphaned generated tests | 14+ `generated_tests_*sdlc*.rs` files | Not in module tree. Zero tests execute. Should delete and regenerate fresh. | CL-5 |
| Unbound provider files | 5 files in `services/sdlc/providers/` | `llm_agent_provider`, `rolling_deploy`, `health_check`, `structured_logging`, `local_credential_provider` — no profile binding. | CL-6 |
| Dual execution architecture | `pipelines/sdlc.dag` vs `sdlc_worker.dag` | Both implement the full SDLC flow differently. Worker path is correct. Pipeline should be demoted to reference. | CL-7 |

## 8. Relationship to Existing Docs

| Document | Role | Update needed? |
|----------|------|---------------|
| `implementation-roadmap.md` | Task list (SDLC-1 through SDLC-CD2) | Yes — Phase 0 must precede SDLC-1 |
| `e2e-gap-analysis.md` | Original gap inventory (A-J) | Outdated — many claims of "resolved" are aspirational |
| `domain-modeling-comprehensive.md` | Layer model + interface specs | No — still accurate as reference |
| `mega-modeling-design.md` | Architectural vision | No — aspirational target unchanged |
| This document | Current reality vs production | Canonical for "what do we actually need to do" |
