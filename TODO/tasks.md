# Task Sheet — Dependency-Ordered, Parallelizable

**Last updated**: 2026-02-21
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

### Archive Update (2026-02-21)

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
- Sprint 10 (all): `AI1`-`AI3`, `PR1`-`PR3`

Active IDs after archive: none (all lanes and Sprint 10 complete)

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
| F: Codegen-first SDLC | **DONE** | `CG1` superseded (SDLC modules are runtime-authored) |

---

## Sprint 10: Autonomous Implementation & Agent Integration — **DONE**

Archived to `TODO/TODONE/tasks-completed.md`. All 6 tasks (`AI1`-`AI3`, `PR1`-`PR3`) complete.

---

## Sprint 11: E2E Scenario Pipeline & Stage Execution — **DONE**

**Goal**: Make the SDLC pipeline execute the full stage progression (Idea -> Design -> DesignReview -> Accepted -> Implementation) with stage-based dispatch, concrete execution handlers for each transition, and a scenario intent YAML to drive the E2E test.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S11-1** | **Stage-based dispatch**: Refactor worker loop to route by `record.stage` via `execute_stage()` instead of unconditionally calling `execute_stage_idea_to_design`. | — | S | **DONE** |
| **S11-2** | **Design -> DesignReview handler**: `execute_stage_design_to_review()` extracts canonical design, runs review, persists review artifact, transitions stage label. | S11-1 | M | **DONE** |
| **S11-3** | **DesignReview -> Accepted handler**: `execute_stage_review_to_accepted()` checks `approved` flag from review artifact, transitions or blocks. | S11-2 | S | **DONE** |
| **S11-4** | **Accepted -> Implementation handler**: `execute_stage_accepted_to_implementation()` assembles `HandoffSpec`, dispatches to `AgentAdapter`, records in agent ledger. | S11-3 | M | **DONE** |
| **S11-5** | **Scenario intent YAML**: `TODO/feature-intent-markdown.yaml` with concrete criteria for the markdown report feature. | — | S | **DONE** |

---

## Sprint 12: E2E Pipeline Execution — Domain Interface Layer

**Design doc**: [docs/design/sdlc/e2e-gap-analysis.md](../docs/design/sdlc/e2e-gap-analysis.md)
**Goal**: Introduce the three-layer abstraction model (pipeline domain concepts -> domain interfaces -> infrastructure implementations) with compile-time deployment profile binding, enabling the SDLC pipeline to execute end-to-end without hand-written Rust orchestration or hardcoded transports.

### Phase 1: Domain Interface Layer (Gaps A, B)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-1** | **IssueProvider interface**: Define `interface IssueProvider` (discover, get, comment, set_labels, close). Refactor `services/github/issues.dag` into `resource GitHubIssueProvider implements IssueProvider`. Add `StubIssueProvider` for tests. | — | M | |
| **S12-2** | **ClaimStore interface**: Define `interface ClaimStore` (acquire, heartbeat, release). Implement `FileClaimStore` using `Filesystem` + `Clock`. Add `InMemoryClaimStore` for tests. Replace `services/sdlc/control_plane.dag` claim operations. | — | M | |
| **S12-3** | **OutcomeLedger interface**: Define `interface OutcomeLedger` (upsert, get). Implement `FileOutcomeLedger` using `Filesystem`. Add `InMemoryOutcomeLedger` for tests. Replace `services/sdlc/control_plane.dag` outcome operations. | S12-2 | S | |
| **S12-4** | **AgentProvider interface**: Define `interface AgentProvider` (spawn, poll, cancel). Refactor `services/agent/codex.dag` into `resource CodexAgentProvider implements AgentProvider`. Add `StubAgentProvider` for tests. | — | S | |
| **S12-5** | **Pipeline uses interfaces**: Update `dsl/pipelines/sdlc.dag` and `dsl/funcs/sdlc_worker.dag` to import domain interfaces instead of concrete services. | S12-1, S12-2, S12-3, S12-4 | M | |

### Phase 2: Compile-Time Profile Binding (Gaps C, D)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-6** | **Profile syntax in parser**: Add `profile` declaration and `bind` statement to `daglang-syntax` parser. | — | M | |
| **S12-7** | **Profile resolution in lowering**: When lowering `uses` declarations, resolve via active profile's bindings. Generate transport code for the concrete implementation. | S12-6 | L | |
| **S12-8** | **`--profile` CLI flag**: Add `--profile` to `daglang compile`. Create `unit_test`, `local`, `cloud_run` profile definitions. | S12-6, S12-7 | S | |
| **S12-9** | **Credential binding via profile**: Wire `credential: env(...)` and `credential: secret(...)` in profile bindings. Connect to existing `credential_chain` pattern for Secret Manager. | S12-7 | M | |

### Phase 3: Runtime Execution (Gaps F, G)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-10** | **SubDag node execution**: Replace `UnsupportedOp` for `SubDag` nodes in `resolve.rs` with recursive DAG resolution and execution. | — | M | |
| **S12-11** | **Pipeline node execution**: Replace `UnsupportedOp` for `Pipeline` nodes in `resolve.rs` with ordered stage sequence execution. | S12-10 | S | |
| **S12-12** | **Worker DAG invocation**: Wire `sdlc.rs` worker to load compiled pipeline, resolve via profile, and execute. Replace `mark_run_completed()` placeholder. | S12-5, S12-8, S12-10, S12-11 | M | |

### Phase 4: Stage Completion (Gaps H, I, J)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **S12-13** | **Code review stage**: Implement real code review in DSL (PR diff retrieval via `PullRequest.ListFiles`, LLM review, findings as PR comment). | S12-12 | M | |
| **S12-14** | **Acceptance testing stage**: Implement real acceptance testing in DSL (trigger CI or run `cargo test`/`cargo clippy` via shell service). | S12-12 | M | |
| **S12-15** | **Agent branch management**: Add git branch creation before `Codex.Spawn`, push after completion, deterministic branch naming (`sdlc/issue-{number}`). | S12-12 | S | |
| **S12-16** | **Agent polling in worker sweep**: Worker checks `agent_ledger` for in-flight runs, calls `AgentProvider.poll()` during regular sweep. | S12-12 | S | |
| **S12-17** | **Pipeline parameter injection**: Pipeline inputs (`owner`, `repo`, `run_key`) bound from profile or passed as DAG inputs at execution time. | S12-8 | S | |

---

## Cleanup: Eliminate Hardcoded Registration Lists

**Goal**: Replace manually maintained lists with discovery/derivation. Every time a new `.dag` module or tool is added, several Rust files require manual updates. These should either be auto-discovered from the filesystem, derived from the compiled DAG metadata, or eliminated entirely.

| ID | Task | Location | Problem | Fix | Size | Status |
|----|------|----------|---------|-----|------|--------|
| **CL1** | **Module order test fixture** | `daglang-cli/src/pipeline.rs:773-832` | 58 hardcoded module names in `expected_real_corpus_module_order()`. Breaks every time a `.dag` file is added/removed/renamed. | Replace with filesystem discovery: glob `dsl/**/*.dag`, extract module IDs, sort. The test asserts the compiler discovers the same set, not a hardcoded list. | S | |
| **CL2** | **Domain resolver dispatch** | `gunbc-dag/src/resolve.rs:625-645` | Match arms manually map `"tools.makegen"`, `"tools.build"`, etc. to resolver functions. New DSL modules require adding a match arm. | The `services.*` branch already uses generic dispatch via `resolve_service_transport()`. Extend this pattern: modules with `ServiceCallMetadata` use generic dispatch; remaining tool modules should derive their op mapping from the DSL definition (callable name -> op enum variant can be generated by a build script or macro from the `.dag` file). | L | |
| **CL3** | **`domain_passthrough_op!` macros** | `gunbc-dag/src/resolve.rs:136-283` | Each tool module has a handwritten macro invocation mapping callable names to enum variants (e.g., `"aggregate_results" => AggregateResults`). These duplicate information already present in the `.dag` files. | Short term: collapse into a single data-driven registry (map of `(module, callable_name) -> Box<dyn Executable>`). Long term: generate from DSL metadata -- the callable names are in the compiled DAG, so the resolver can look them up dynamically. | M | |
| **CL4** | **`WorkspaceBinary::ALL` array** | `gunbc-dag/src/binaries.rs:29-42` | 12-element `const ALL` array + match arms in `tool_name()`/`from_tool_name()`. New binaries require three manual edits. | Derive from `Cargo.toml` `[[bin]]` sections or from the filesystem (`gunbc-dag/src/bin/*.rs`). A build script can enumerate binaries and generate the enum. | S | |
| **CL5** | **`TOOL_WORKFLOWS` registry** | `gunbc-dag/src/workflow/spec_builders.rs:1445-1516` | 14 hardcoded `ToolWorkflowDescriptor` entries. New tool workflows need a manual entry. | Derive from DSL: each `tools.*.dag` that exports an entrypoint `func` is a tool workflow. The workflow spec builder can discover these from compiled DAG metadata instead of a static array. | M | |
| **CL6** | **Process unit registry** | `gunbc-dag/src/workflow/process_registry.rs:220-298` | Hardcoded CI and test-all workflow unit arrays. New CI steps need manual entries. | Derive from `pipelines/ci.dag`: the CI pipeline stages define the process units. The registry can be generated from the compiled pipeline DAG. | M | |
| **CL7** | **`MANUAL_TOOL_DEFS`** | `gunbc-dag/src/makegen/registry.rs:1713-1714` | 2 hardcoded manual tool definitions (`pragma`, `build`). | Investigate why these can't use the standard discovery path. If they need special treatment, document why; otherwise fold into the standard tool registry. | S | |
| **CL8** | **`std.resources` name match** | `gunbc-dag/src/resolve.rs:688-695` | Hardcoded resource names (`"Filesystem"`, `"Network"`, `"Clock"`, `"AuthContext"`). Adding a new resource to `std/resources.dag` requires a Rust match arm. | Derive from the compiled `std/resources.dag` metadata. The resource names are already in the DAG -- the resolver should read them from there. | S | |

### Priority

- **CL1** is the most fragile (58 entries, breaks on any module change). Fix first.
- **CL2 + CL3** are the largest impact (the resolver is the main bottleneck for adding DSL modules without touching Rust).
- **CL4-CL8** are smaller wins but compound over time.

### Architectural direction

All of these share a root cause: the Rust runtime has manually duplicated metadata that already exists in the DSL. The fix is always the same pattern: **read the metadata from the compiled DAG** instead of hardcoding it. This aligns with the codegen-first policy (mega design Section 5.3): "New SDLC behavior must be added to DSL/codegen first."

---

## Deferred

| ID | Task | Context | Size | Status |
|----|------|---------|------|--------|
| **DG1** | **Daggen (Dynamic DAG Generation)** | `needs_daggen()` returns false. Re-enable to scale the pipeline by dynamically generating steps based on git diffs. | L | **DEFERRED** |
| **S12-E** | **Multi-worker CAS** | Gap E: Implement `GcsClaimStore` with generation-based CAS (`x-goog-if-generation-match`). Not needed for single-worker local dev. | M | **DEFERRED** |

---

## Active Open Items (Deferred)

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
