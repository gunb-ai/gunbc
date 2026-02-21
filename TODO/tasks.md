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

## Deferred

| ID | Task | Context | Size | Status |
|----|------|---------|------|--------|
| **DG1** | **Daggen (Dynamic DAG Generation)** | `needs_daggen()` returns false. Re-enable to scale the pipeline by dynamically generating steps based on git diffs. | L | **DEFERRED** |

---

## Active Open Items (Deferred)

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
