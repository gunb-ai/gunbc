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
| SDLC runtime launch + infra control-plane model | In progress | `IN0-D` opened for stateless worker topology + infra bringup contracts. |
| SDLC codegen-first objective | In progress | `CG0-D` opened: behavior/modeling in DSL, compiled to Rust/Go/C. |
| SDLC mega modeling gate | In progress | `MD0-D` is canonical; implementation tasks are downstream. |

### Archive Update (2026-02-21)

Moved to `TODO/TODONE/tasks-completed.md`:

- `WF6`-`WF9`
- `WF14`-`WF18`
- `DL1`-`DL4`
- `W1`
- `W4`-`W8`

Active IDs after archive:

- `IM0-D`
- `MD0-D`
- `IM1`, `IM2`, `IM3`, `IM4`, `IM5`, `IM6`, `IM7`, `IM8`, `IM9`, `IM10`, `IM11`, `IM12`, `IM13`
- `IN0-D`, `IN1`, `IN2`, `IN3`, `IN4`
- `CG0-D`, `CG1`, `CG2`, `CG3`, `CG4`, `CG5`, `CG6`
- `W2`, `W3`
- `W9`, `W10`, `W11`, `W12`, `W13`, `W14`
- `AX1`, `AX2`
- `DL5`, `DL6`, `DL7`, `DL8`

### SDLC Design Checklist (Must Hold)

These design contracts are required to avoid duplicate issues/updates and non-restartable runs.

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

### Mega Modeling Gate

`MD0-D` is the canonical SDLC modeling review gate.

1. Canonical doc: `docs/design/sdlc/mega-modeling-design.md`
2. Rule: all implementation tasks (`IM1+`, `IN1+`, `CG1+`, `W*`) are downstream of `MD0-D` sign-off.

### Tonight Handoff Lanes (Open Work, Shared Design Gate)

Use these lanes to assign workers with minimal overlap. `MD0-D` is the shared upstream design gate; after approval, lanes are independently parallel.

| Lane | Task IDs | Preconditions | Primary Files/Areas | Done When | Verify | Status |
|---|---|---|---|---|---|---|
| A: SDLC delivery lane | `MD0-D` -> `IM0-D` -> `IM1` -> `IM2` -> `IM3` -> `IM4` -> `IM5` -> `IM6` -> `IM7` -> `IM8` -> `IM9` -> `IM10` -> `IM11` -> `IM13` -> `W9` -> `IM12` -> `W10` -> `W11` -> `W12` -> `W13` -> `W14` | `MD0-D` approved | `docs/design/sdlc/`, `TODO/`, `core/ir/src/transport/github/`, `lib/`, `gunbc-dag/src/resolve.rs`, `gunbc-dag/src/workflow/`, `gunbc-dag/src/bin/`, `dsl/pipelines/` | intake is idempotent + conflict-safe, stage execution is idempotent/resumable, async pickup is atomic, retry behavior is deterministic with persisted budget, crash reconciliation converges safely (including artifact marker windows), AwaitApproval yields by releasing claim, fail-closed routes are terminalized with user-visible status, signal triggers are durable/idempotent with anti-entropy fallback scans, provider capability gating is enforced, artifact payload/reference contract is implemented, and `gunbc-sdlc` runs issue lifecycle end-to-end in dry-run with metrics while preserving provider-agnostic boundaries | `cargo run -q --release -p gunbc-dag --bin gunbc-sdlc -- intake --intent TODO/issue-intent-template.yaml --dry-run`; `cargo run -q --release -p gunbc-dag --bin gunbc-sdlc -- worker --dry-run`; `cargo run -q --release -p gunbc-dag --bin gunbc-sdlc -- --issue 42 --dry-run` | **OPEN** |
| B: Review credential certification lane | `W2` -> `W3` | none | `gunbc-dag/src/bin/review.rs`, credential policy wiring, provider selection paths | real-mode review succeeds for Anthropic + OpenAI on same diff; failures are fail-closed with actionable errors | `gunbc-review -r . --provider anthropic`; `gunbc-review -r . --provider openai` | **OPEN** |
| C: Planner/CI additional lane | `AX1`, `AX2` | none | CI guardrails, makegen registry/tool discovery contracts | bootstrap invariant enforced in CI and registry coupling risk reduced | `make ci` + targeted contract tests | **OPEN** |
| D: Daglang convergence lane | `DL5`, `DL6`, `DL7`, `DL8` | none | `core/daglang/daglang-cli/` compile/pipeline/ui surfaces | daglang compile/check/model surfaces are consolidated and explicit | `cargo test -p daglang-cli` | **OPEN** |
| E: Runtime infra/control-plane lane | `MD0-D` -> `IN0-D` -> `IN1` -> `IN2` -> `IN3` -> `IN4` | `MD0-D` approved | `docs/design/sdlc/mega-modeling-design.md`, `TODO/infra-intent-template.yaml`, `gunbc-dag/src/bin/infra.rs`, `lib/cloud-ops/` | SDLC runtime launch is explicitly modeled, deployable ownership and trigger matrix are explicit, infra bringup is intent-driven/idempotent, and worker startup has fail-closed infra preflight semantics for stateless fleet operation | `cargo run -q --release -p gunbc-dag --bin gunbc-infra -- spec --env dev`; `cargo run -q --release -p gunbc-dag --bin gunbc-infra -- plan --env dev`; `cargo run -q --release -p gunbc-dag --bin gunbc-infra -- status --env dev` | **OPEN** |
| F: Codegen-first SDLC lane | `MD0-D` -> `CG0-D` -> `CG1` -> `CG2` -> `CG3` -> `CG4` -> `CG5` -> `CG6` | `MD0-D` approved | `dsl/pipelines/`, `dsl/tools/`, `dsl/services/`, `dsl/infra/`, `core/daglang/`, `gunbc-dag/src/workspace/subdags/`, generated target test harnesses | SDLC behavior/modeling is DSL-authored, emitted targets (Rust/Go/C) are executable with conformance parity, and Rust hand-written logic is reduced to generic adapters/runtime kernel | `cargo run -q --release -p daglang-cli -- compile dsl/pipelines/sdlc.dag`; `cargo test --workspace`; target-specific emitted artifact smoke checks | **OPEN** |

Handoff rules:

1. One worker owns one lane at a time.
2. Every PR title begins with primary task ID (example: `[W9] ...`).
3. Any behavioral change must include/adjust at least one regression test.
4. If a lane hits unresolved design ambiguity, open/update a `-D` design task first.

---

## Sprint 9: Tool Workflows & SDLC Pipeline (Next Sprint)

**Goal**: Complete the local AI-assisted SDLC pipeline and operationally certify review credentials, with intake/idempotency guarantees first.

### Phase 0: Issue Management Modeling + Intake

| ID | Task | Deps | Size |
|----|------|------|------|
| **MD0-D** | **SDLC mega modeling design gate**: consolidate canonical abstractions/invariants/layers/conformance and traceability matrix into one review doc. **Design doc**: `docs/design/sdlc/mega-modeling-design.md`. | — | M |
| **IM0-D** | **SDLC issue abstraction modeling section**: finalize provider-agnostic issue contracts, adapter boundary rules, idempotency keys, comment/label upsert protocol, and commit linkage invariants in `docs/design/sdlc/mega-modeling-design.md`. | MD0-D | M |
| **IM1** | **Intent sheet contract**: define intent schema and canonical fields (`intent_id`, objective, success criteria, constraints, owner, links), with template under `TODO/`. | IM0-D | S |
| **IM2** | **Issue intake upsert flow**: add intake command/path that creates or updates one canonical issue per `intent_id` (never duplicate create on rerun). | IM0-D, IM1 | M |
| **IM3** | **Stage idempotency + resume keying**: define and persist run/stage keys (`issue_id`, `stage`, `input_hash`, `policy_version`) and skip duplicate side effects on replay. | IM0-D, IM2 | M |
| **IM4** | **Idempotent remote update protocol**: comments/artifacts/labels are upserted/transitioned via deterministic markers and compare-and-set semantics. | IM0-D, IM2 | M |
| **IM5** | **Commit/update trace linkage**: enforce branch/commit metadata linking changes to `intent_id`, `issue_id`, and run key. | IM3 | S |
| **IM6** | **Claim/lease abstraction**: implement atomic claim protocol for `(issue_id, stage)` ownership with lease expiry and heartbeat semantics. | IM0-D, IM3 | M |
| **IM7** | **Async control loop**: implement worker tick loop (`discover -> claim -> execute -> release`) using `IM6`, with bounded retries/backoff, durable trigger handling, and anti-entropy rediscovery scans. | IM6 | M |
| **IM8** | **Stage transaction executor**: implement fixed step ordering (revalidate -> run key check -> provisional artifact marker upsert -> CAS transition -> canonical marker upsert/confirm -> outcome record) with crash-safe replay behavior. | IM3, IM4, IM7 | M |
| **IM9** | **Failure taxonomy + retry policy**: encode typed failure classes (`TransportTransient`, `StateConflict`, etc.), persist retry budget/timing state in outcome metadata, and enforce terminal fail-closed behavior for non-retryable outcomes. | IM0-D, IM7 | S |
| **IM10** | **Intake conflict policy**: implement fail-closed handling for multi-match `intent_id` collisions and deterministic intent->issue upsert resolution. | IM0-D, IM2 | S |
| **IM11** | **Replay reconciliation loop**: implement deterministic convergence logic for crash windows (`artifact written`, `transition applied`, `ledger missing`), including stale provisional-marker supersession/cleanup by lease generation, and duplicate-intake loser cleanup. | IM3, IM8, IM10 | M |
| **IM13** | **Artifact payload/reference contract**: implement provider-agnostic artifact model (`Inline` vs `BlobRef`) with deterministic normalization + content-hash equality rules, canonical marker collision policy, and GitHub adapter compatibility. | IM0-D, IM4 | M |

### Phase 0.5: Runtime Infrastructure and Launch Modeling

| ID | Task | Deps | Size |
|----|------|------|------|
| **IN0-D** | **Runtime/infra control-plane modeling section**: define SDLC launch topology (stateless worker fleet), deployable ownership for each `H*` graph, trigger/signal matrix with idempotency keys, startup/drain semantics, and infra interaction boundaries in `docs/design/sdlc/mega-modeling-design.md`. | MD0-D | M |
| **IN1** | **Infra intent contract**: define versioned `InfraIntent` schema for runtime dependencies (claim store, outcome ledger, secrets, metrics), with template under `TODO/`. | IN0-D | S |
| **IN2** | **Infra plan/apply coverage for SDLC runtime**: extend infra plan/apply modeling from secret-only provisioning to required runtime dependencies and drift-aware reconciliation contracts. | IN0-D, IN1 | M |
| **IN3** | **Worker startup preflight gate**: add fail-closed real-mode startup checks for infra readiness (`status` + required component checks + capability prerequisites). | IN1, IN2 | M |
| **IN4** | **Stateless deployment profile + drain semantics**: define and implement launch profile contracts for 5-10+ workers, including local co-located profile parity, graceful drain/restart behavior, and operational runbook checks. | IN0-D, IN3 | M |

### Phase 0.6: Codegen-First SDLC Cutover

| ID | Task | Deps | Size |
|----|------|------|------|
| **CG0-D** | **Codegen-first architecture modeling section**: lock boundary that SDLC behavior/modeling is authored in DSL and compiled to Rust/Go/C, with runtime kernel limited to generic engine/adapters in `docs/design/sdlc/mega-modeling-design.md`. | MD0-D | M |
| **CG1** | **Canonicalize SDLC DSL modules**: move/promote SDLC pipeline + design tool specs into runtime-discovered `dsl/` roots (`dsl/pipelines/sdlc.dag`, `dsl/tools/design.dag`) with equivalent semantics. | CG0-D | M |
| **CG2** | **Discovery-to-execution cutover for SDLC modules**: remove/manual-minimize per-module Rust mapping for SDLC paths so discovered DSL modules execute via generic runtime wiring. | CG1 | M |
| **CG3** | **Control-plane DSL resources/services**: model claim lease store and stage outcome ledger as DSL-visible interfaces/resources with CAS/heartbeat/outcome contracts. | CG0-D, IN0-D | M |
| **CG4** | **Infra intent reconcile in DSL**: express runtime infra intent plan/apply/reconcile flow in DSL and make `gunbc-infra` delegate to compiled DSL orchestration path. | CG3, IN1, IN2 | L |
| **CG5** | **Generated target entrypoints (Rust/Go/C)**: emit runnable SDLC worker/infra reconcile entrypoints from DSL artifacts for Rust first, then Go/C smoke execution parity; include explicit C adapter/runtime ownership-handle boundary for variable-length payloads; keep interpreter mode as supported non-primary execution option. | CG2, CG4 | L |
| **CG6** | **Multi-level conformance + backend rotation harness**: add layered conformance suites (DSL, IR, semantic, adapter, e2e parity), C memory-ownership sanitizer coverage, and non-prod backend rotation strategy across generated backends with interpreter differential checks. | CG5 | M |

### Phase 1: Review Credential Certification

**Credentials & Core Engine:**
| ID | Task | Deps | Size |
|----|------|------|------|
| **W2** | **Credential smoke test**: run `gunbc-review` in real mode using `ANTHROPIC_API_KEY` against a small diff and verify structured findings output. | — | S |
| **W3** | **Multi-provider operational verification**: verify real-mode `--provider openai` and `--provider anthropic` both work with env/policy credentials, with explicit fail-closed errors on missing creds. | W2 | S |

### Phase 2: SDLC Pipeline Delivery

| ID | Task | Deps | Size |
|----|------|------|------|
| **W9** | **GitHub Issues transport + adapter**: add issues transport module and provider-agnostic `TrackedIssue` mapping for create/get/update/comment/labels/list, strictly following `IM0-D` adapter boundary contracts and `IM10` intake conflict rules. | IM0-D, IM2, IM10 | M |
| **IM12** | **Provider capability gate + contract tests**: implement capability checks (`StageCas`, `CommentUpsertByMarker`, `ManagedIssueSearch`, `DeterministicIssueIdentity`) and block real mode when unmet. | IM0-D, W9 | S |
| **W10** | **DesignOps module**: implement `lib/design-ops/` with `PrepareDesignPrompt` and `ParseDesignResponse`, returning typed design artifacts. | IM1 | M |
| **W11** | **SDLC resolver wiring**: connect pipeline module to resolver/execution path with typed transport + design ops integration, using `IM3` run keys and `IM8` stage transaction contracts while preserving provider fungibility at orchestration boundaries. | IM0-D, IM8, W9, W10 | M |
| **W12** | **`gunbc-sdlc` CLI binary**: add entrypoint to run lifecycle for an issue ID with dry-run and real modes, including intake/update flows and async worker loop operations with `IM9` retry-class semantics, `IM11` reconciliation, explicit terminal fail-closed issue updates, `IM12` capability gate enforcement, and local co-located control-loop execution profile for business-logic testing. | IM7, IM8, IM9, IM11, IM12, W11 | M |
| **W13** | **Approval gates**: extend workflow planner + ledger with asynchronous `AwaitApproval` yield semantics (`PENDING_APPROVAL` persist, claim release, rediscovery-based resume). | IM3, W12 | L |
| **W14** | **Metrics/monitoring**: record stage duration, LLM cost, and approval latency in SDLC execution report. | W12 | M |

### Phase 3: Additional Parallel Work

| ID | Task | Deps | Size |
|----|------|------|------|
| **AX1** | **Bootstrap invariant CI gate**: add CI assertion that bootstrap-safe binaries compile without generated sources. (From deferred open item #2) | — | S |
| **AX2** | **Registry coupling hardening**: reduce or explicitly contract-test coupling between `default_registry()` and `gunbc_codegen::registry::derive_tool_defs()`. (From deferred open item #3) | — | M |
| **DL5** | **Unify compile/pipeline overlap**: remove duplicated compile-path logic between `compile.rs` and `pipeline.rs` through one shared context pipeline. | — | M |
| **DL6** | **Manifest semantics clarity**: rename or split `daglang manifest` so progress vs topology output is unambiguous. | DL5 | S |
| **DL7** | **Canonical IR CLI surface**: promote canonical IR JSON snapshot flow into `daglang compile --format canonical-json`. | DL5 | S |
| **DL8** | **Viz default decision**: finalize default output mode for `daglang viz` (ASCII vs Mermaid), document and lock with tests. | — | S |

---

## Active Open Items (Deferred)

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
