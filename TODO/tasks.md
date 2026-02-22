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
| Dry-run deployment readiness | Open | Rust worker multi-stage dispatch enables local dry-run before compiled DAG path is ready. See Sprint 11.5. |
| Dual execution path convergence | Open | Rust worker path (scaffolding) vs compiled DAG path (target). Rust worker must not accumulate SDLC sequencing logic beyond what's needed for dry-run. |

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
| G: Workflow DSL migration | **ACTIVE** | WM-1..WM-9 (14 workflows, 69 process units, 16 builder fns) |
| H: DSL expression language | **ACTIVE** | EX-1..EX-15 (22 tool ops → 0; parser has syntax, gap is lowering + execution) |

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

## Sprint 11.5: Dry-Run Deployment Readiness

**Design doc**: [docs/design/sdlc/e2e-gap-analysis.md](../docs/design/sdlc/e2e-gap-analysis.md) (Section 7-8)
**Goal**: Enable a local dry-run deployment where the Rust worker progresses issues through the full stage chain (idea → done) using the existing ledger/claim/reconcile infrastructure. This is the bridge to Sprint 12's compiled-DAG execution path.

**Rationale**: Sprint 12's compiled DAG path requires Gaps A, B, C, F to all be resolved (domain interfaces, profile binding, SubDag/Pipeline execution). The Rust worker already has working ledger/claim/reconcile infrastructure; wiring multi-stage dispatch is the minimum unblocking work for a dry-run.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **DR-1** | **Wire stage dispatch into run_worker**: Replace direct `execute_stage_idea_to_design()` call (`sdlc.rs:1062`) with `execute_stage()` dispatcher that routes by `record.stage`. Sprint 11 delivered the dispatcher and per-stage handlers. | — | S | |
| **DR-2** | **Add remaining stage handlers**: Implement stub handlers for implementing→code-review, code-review→testing, testing→done, and done→close. Each handler updates `record.stage` and records an outcome. | DR-1 | S | |
| **DR-3** | **Verify stage ledger advancement**: Ensure each handler updates `record.stage` in the intake ledger so the next worker pass picks up from the correct stage. | DR-1, DR-2 | S | |
| **DR-4** | **Reconcile pipeline definitions**: Update `dsl/pipelines/sdlc.dag` to use `param` declarations matching the design version in `docs/design/sdlc/sdlc.dag`. Stub undefined functions (`generate_implementation_plan`, `get_pr_diff`). | — | S | |
| **DR-5** | **Integration test: multi-stage progression**: Test that runs `intake` + 8× `worker` and verifies the intake record reaches `stage: done` with correct execution report. | DR-1, DR-2, DR-3 | M | |

**Verification**: `cargo test --workspace` passes. `gunbc-sdlc intake --intent <path> && gunbc-sdlc worker` repeated 8 times results in a `done` stage with correct JSON report.

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

**Design doc**: [TODO/design-eliminate-registration-lists.md](design-eliminate-registration-lists.md)
**Goal**: Replace manually maintained lists with discovery/derivation. Every time a new `.dag` module or tool is added, several Rust files require manual updates. These should either be auto-discovered from the filesystem, derived from the compiled DAG metadata, or eliminated entirely.

### Phase 1 — Resolver trusts compiler — **DONE**

Implemented: default-passthrough resolver, `Option`-returning inventory resolvers, `InfraToolOp` deletion, behavioral test helpers. See design doc for details. CL2, CL3, CL8 are resolved.

### Phase 1 Remaining (small items)

| ID | Task | Location | Problem | Fix | Size | Status |
|----|------|----------|---------|-----|------|--------|
| **CL1** | **Module order test fixture** | `daglang-cli/src/pipeline.rs` | 58 hardcoded module names in `expected_real_corpus_module_order()`. Breaks every time a `.dag` file is added/removed/renamed. | Replace with filesystem discovery: glob `dsl/**/*.dag`, extract module IDs, sort. The test asserts the compiler discovers the same set, not a hardcoded list. | S | **DONE** |
| **CL4** | **`WorkspaceBinary::ALL` array** | `gunbc-dag/src/binaries.rs` | 12-element `const ALL` array + match arms. New binaries require three manual edits. | Derive from `Cargo.toml` `[[bin]]` sections or from the filesystem. | S | **DONE** |
| **CL7** | **`MANUAL_TOOL_DEFS`** | `gunbc-dag/src/makegen/registry.rs` | 2 hardcoded manual tool definitions (`pragma`, `build`). | Investigate why these can't use the standard discovery path. Fold in or document. | S | **DONE** |

### Lane G: Workflow DSL Migration (Phase 2)

**Design doc**: [TODO/design-eliminate-registration-lists.md](design-eliminate-registration-lists.md) — Changes 4-5
**Goal**: Migrate 14 Rust-constructed workflow specs to DSL pipeline files. Eliminate `TOOL_WORKFLOWS` (14 entries), workflow builder functions (16 functions), and `process_registry` (69 hardcoded entries).
**Prerequisite**: None (independent of Phase 3).

**Context**: `dsl/pipelines/ci.dag` (126 lines), `dsl/pipelines/sdlc.dag` (552 lines), and `dsl/pipelines/reconciler.dag` (258 lines) already demonstrate the target format. Each workflow is a `pipeline` with `stage` declarations, `[after ...]` dependencies, and `[when ...]` conditional guards.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **WM-1** | **Migrate `build-all` workflow to DSL**: Simplest workflow (1 core unit). Create `dsl/workflows/build-all.dag`. Verify the compiled DAG matches the Rust-constructed spec structurally. | — | S | **DONE** |
| **WM-2** | **Migrate `makegen` workflow to DSL**: 4 process units. Create `dsl/workflows/makegen.dag`. | — | S | **DONE** |
| **WM-3** | **Migrate `bootstrap` workflow to DSL**: 5 process units. Create `dsl/workflows/bootstrap.dag`. | — | M | **DONE** |
| **WM-4** | **Migrate `pragma` workflow to DSL**: 7 process units, most complex single-tool workflow. Create `dsl/workflows/pragma.dag`. | — | M | **DONE** |
| **WM-5** | **Migrate `deps` workflow to DSL**: 8 process units. Create `dsl/workflows/deps.dag`. | — | M | **DONE** |
| **WM-6** | **Migrate `gist` workflow family to DSL**: 3 variants (`gist-snapshot`, `gist-diff`, `gist-recent`) sharing 9 process units. Create `dsl/workflows/gist.dag` with parameterized mode. | — | M | **DONE** |
| **WM-7** | **Migrate `dag-viz` workflow family to DSL**: 3 variants (`dag-viz`, `dag-viz-diff`, `dag-viz-recent`) + `dag-snapshot`. 6-7 process units each. Create `dsl/workflows/dag-viz.dag`. | — | M | **DONE** |
| **WM-8** | **Derive process unit claims from DSL annotations**: The DSL already has `@file(READ/WRITE)` annotations and the compiler extracts `ResourceUsage`. Generate `UnitClaim` entries from compiled pipeline metadata instead of the hardcoded 69-entry `default_process_unit_registry()`. | WM-1..WM-7 | M | |
| **WM-9** | **Delete Rust workflow builders**: Remove `TOOL_WORKFLOWS` registry, all `*_workflow_spec()` builder functions, and `default_process_unit_registry()`. Wire workspace subdag discovery to load compiled DSL workflows. | WM-8 | M | |

**Parallelism**: WM-1 through WM-7 are fully independent — each workflow can be migrated by a separate worker. WM-8 and WM-9 depend on all migrations completing.

### Lane H: DSL Expression Language (Phase 3)

**Design doc**: [TODO/design-eliminate-registration-lists.md](design-eliminate-registration-lists.md) — Changes 6-11
**Goal**: Make the existing DSL expression features usable in function bodies so that all 22 custom tool op variants (across 5 modules) can be expressed in DSL. After this lane, zero tool `Executable` impls exist outside compiler/executor infrastructure. Rust is no longer an escape hatch for business logic.

**Principle**: Fix the data model first, then add minimal expression support. Most "string manipulation" disappears when data is properly structured. See design doc "DSL Language Features Required" for the full analysis.

**Key finding**: The parser already supports 21 expression types including `if/else`, `match`, `for`, all binary ops (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||), unary ops (!, -), list/map/record literals, string interpolation, lambdas, and pipe operators. The gap is in the **lowering and execution path** — ensuring these expressions compile to executable operations when used in function bodies. The compiler pipeline is: parser (2.8k LOC) → resolve → typecheck → lower (7.5k LOC) → emit (1.2k LOC+).

#### Phase 3a: Structured data at boundaries (eliminates ~45% of custom ops)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **EX-1** | **Structured transport responses**: Extend service call declarations with `@parse` annotations so the transport layer parses shell output into typed records. Eliminates all ad-hoc `.lines()/.trim()/.strip_prefix()` parsing in bootstrap (4 ops) and codegen (2 ops). | — | M | **DONE** |
| **EX-2** | **Structured path and glob types**: Make `FilePath` a proper structured type with segments (not a string alias). Add `GlobPattern` type. Path construction, joining, and pattern building become type-safe operations. Eliminates all path string manipulation in pragma and codegen (~8 string ops). | — | M | **DONE** |
| **EX-3** | **DSL data source declarations**: Add `data` blocks in DSL for declaring static typed configuration. Move clippy allowlist rules (8), dead code rules (5), allow lints (3), tool registry (12 tools), gitignore categories (14), and codegen path templates from Rust into `dsl/config/*.dag` files. Compiler resolves data references at compile time. No `dsl/config/` directory exists yet — create it. | — | M | **DONE** |

#### Phase 3b: Expression lowering (the real gap)

The parser has the syntax. The gap is making it executable. These tasks focus on the **lowering pass** (`daglang-lower/src/lib.rs`, 7.5k LOC) and **execution runtime** — ensuring parsed expressions in function bodies lower to `LoweredOp` nodes that the resolver can execute.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **EX-4** | **Lower collection method calls**: Parser has `List`, `Pipe`, `Lambda`, `Call`. Lowering must generate executable nodes for `.map(fn)`, `.filter(fn)`, `.sort()`, `.dedup()`, `.any(fn)`, `.all(fn)`, `.len()`, `.contains(item)`, `.join(sep)` when used in function bodies. Verify existing `CollectionOp` / `MapOp` / `FilterOp` etc. in `core/exec` are wired through. | — | M | **DONE** |
| **EX-5** | **Lower control flow in function bodies**: Parser has `If`, `Match`, `For`, `Let`, `BinOp`, `UnaryOp`. Verify the lowering pass emits correct `LoweredOp` graph structures for branching, iteration, and variable bindings within `fn` bodies. Existing `BranchOp`, `GuardOp`, `LoopOp` in `core/exec` may already cover this. | — | M | **DONE** |
| **EX-6** | **Lower string interpolation and formatting**: Parser has `StringInterp` with `Literal`/`Expr` parts. Ensure lowering emits nodes that evaluate interpolated expressions and concatenate results. This is the bridge to structured rendering. | — | S | **DONE** |
| **EX-7** | **Structured document rendering**: Add `render` functions producing typed document trees (`TextFile`/`Document`). Sections, lines, comments, and blank lines are structural blocks. Rendering engine handles formatting. Replaces all `format!()`/`write!()`/`.push_str()` in pragma (3 ops), makegen (2 ops), build (1 op), docgen (1 op). | EX-5, EX-6 | L | |
| **EX-8** | **End-to-end function body test**: Write a `.dag` file with a `fn` that uses `if/else`, `match`, `for`, list ops, string interpolation, and record construction in its body. Compile, resolve, and execute it. This is the integration gate proving the full pipeline works before migrating real ops. | EX-4, EX-5, EX-6 | S | |

#### Phase 3c: Custom op migration (the payoff — 22 ops across 5 modules)

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **EX-9** | **Migrate `tools.pragma` ops to DSL**: 3 ops (RenderClippy, RenderAllowlist, RenderLintPolicy). Depends on structured rendering (EX-7) and data sources (EX-3). | EX-3, EX-7 | M | |
| **EX-10** | **Migrate `tools.makegen` ops to DSL**: 3 ops (LoadRegistry, RenderMakefile, Entrypoint). Depends on data sources (EX-3) and rendering (EX-7). | EX-3, EX-7 | M | |
| **EX-11** | **Migrate `tools.bootstrap` ops to DSL**: 4 ops (PrepareScanWorkspace, ParseScanResult, GenerateMakefile, GenerateGitignore). Depends on structured transport (EX-1) and collections (EX-4). | EX-1, EX-4 | M | |
| **EX-12** | **Migrate `tools.codegen` ops to DSL**: 5 ops (PrepareCodegenExists, ParseCodegenExists, PrepareCodegenCommand, ParseCodegenResult, PrepareStampWrite). Depends on structured transport (EX-1), structured paths (EX-2), and control flow (EX-5). | EX-1, EX-2, EX-5 | M | |
| **EX-13** | **Migrate `tools.build` ops to DSL**: 7 ops (PrepareBuild, ParseBuild, PrepareTest, ParseTest, PrepareClippy, ParseClippy, Summary). Depends on control flow (EX-5) and rendering (EX-7). | EX-5, EX-7 | M | |
| **EX-14** | **Migrate remaining ops (ci, docgen, testgen, dag-viz) to DSL**: ~24 additional ops beyond the 5 core modules. These follow the same patterns as EX-9 through EX-13. Can be parallelized per module. | EX-1..EX-8 | L | |
| **EX-15** | **Delete custom resolver path**: After all custom ops are migrated, the `resolve_domain` match arms for custom modules become empty. Remove them — `resolve_domain` reduces to service transport + default passthrough. | EX-9..EX-14 | S | |

**Parallelism**: EX-1, EX-2, EX-3 are fully independent (data model fixes). EX-4, EX-5, EX-6 are independent of each other (lowering path work). EX-9 through EX-14 are independent per module once their expression dependencies are met — each module can be migrated by a separate worker. EX-8 is the integration gate before any migration begins.

**Success criteria**: After Lane H, adding a tool of any complexity requires **1 DSL file** and **0 Rust changes**. Custom tool `Executable` impls drop from 22 to 0 (46 total impls including infrastructure ops that stay in Rust by design).

---

## Deferred

| ID | Task | Context | Size | Status |
|----|------|---------|------|--------|
| **DG1** | **Daggen (Dynamic DAG Generation)** | `needs_daggen()` returns false. Re-enable to scale the pipeline by dynamically generating steps based on git diffs. | L | **DEFERRED** |
| **S12-E** | **Multi-worker CAS** | Gap E: Implement `GcsClaimStore` with generation-based CAS (`x-goog-if-generation-match`). Not needed for single-worker local dev. | M | **DEFERRED** |

---

## Active Open Items (Deferred)

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
