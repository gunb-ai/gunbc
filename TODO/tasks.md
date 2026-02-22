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
| I: Type system enforcement | **ACTIVE** | TS-1..TS-1d, TS-2..TS-5, TS-7 (237 port updates, 3 deletions, 1 test fix) |
| J: Compiler test fix + pipeline execution | **ACTIVE** | J-0..J-4, TS-6 (4 test fixes, deps freshness, SDLC dry-run) |

---

## Lane I: Type System Enforcement — Hard Cutover

**Context**: Set-theoretic types-as-DAGs migration (Phases 1-6) substantially complete. IR foundation, DSL type compilation, TypeRegistry wiring, Bytes support, lattice boundary witnesses, and cross-product test generation all implemented. This lane completes the migration by eliminating every legacy escape hatch: `PortType::Any` catch-all, `types_match()` string heuristic, `canonical_type_name()` ad-hoc normalization, and `Option<TypeRegistry>` soft bypass. After this lane, every type mismatch is a compile error.

**Audit baseline** (2026-02-21): 237 `port(..., "String")` calls across 9 graph files. `types_match()` at 2 call sites + 14 `canonical_type_name()` call sites across 2 crates. `PortType::Any` catch-all at 2 sites in `port_type.rs`. `Option<TypeRegistry>` in `codegen.rs`.

**Mutual exclusivity**: Lane I touches `lib/*/src/graph.rs` files and `core/` compiler/IR crates. Lane J does NOT touch any of these files.

### Phase I-A: Port type propagation (all graph builders)

Every `port(..., "String")` that should be a domain type must be updated before the `PortType::Any` catch-all can be removed. The strict path (`try_parse_port_type`) already recognizes domain types — graph builders just aren't using them.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-1** | **GCP credential port types** | 62 ports in `lib/gcp-ops/src/graph.rs`. Credential ports (`access_token`, `subject_token`, `client_secret`, `refresh_token`) → `Secret`. Identity ports (`service_account`) → `GcpServiceAccountEmail`. Project ports → `GcpProjectId`. Audience ports → `NonEmptyString`. Requires registering `OptionalSecret` if any port is optional. 2 duplicate graph functions share these ports. | — | L | |
| **TS-1b** | **Cloud-ops port types** | 49 ports across 4 files in `lib/cloud-ops/src/` (`graph.rs` 28, `github_credential_graph.rs` 6, `infra_plan_apply.rs` 5, `infra_bootstrap.rs` 10). Same credential/identity patterns as TS-1. | TS-1 | M | |
| **TS-1c** | **Review + LLM port types** | `lib/review/src/graph.rs` (102 ports), `lib/llm-ops/src/graph.rs` (13 ports). Ports like `provider`, `model`, `content`, `question`, `answer` → `NonEmptyString` or domain types. `secret_name` → `SecretName`. `scheme`/`header_name` → `NonEmptyString`. | — | L | |
| **TS-1d** | **Remaining graph port types** | `lib/aws-ops/src/graph.rs` (3), `lib/azure-ops/src/graph.rs` (3), `lib/tools/gist/src/graph.rs` (6), `lib/tools/deps/src/graph.rs` (1), `gunbc-dag/src/testgen_dag/graph.rs` (1). Smaller scope, same patterns. | — | S | |

**Parallelism**: TS-1, TS-1c, TS-1d are independent. TS-1b depends on TS-1 (shares credential type decisions).

### Phase I-B: Delete legacy type comparison

`types_match()` (daglang-typecheck line 2555) creates a fresh `TypeRegistry::with_core_types()` on every call, never sees domain types, and falls back to short-name suffix matching (`rsplit('.').next()`) which can produce false positives (`foo.Bar` matches `baz.Bar`). `canonical_type_name()` (daglang-syntax/ast_utils.rs line 27) strips generic parameters via `split('<').next()` — loses type parameter information entirely. Both must be replaced by `TypeRegistry::is_compatible()` with a registry threaded through the context.

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-7** | **Delete `types_match()` and `canonical_type_name()`** | **Delete** `types_match()` (2 call sites: line 2227 in `infer_record_literal_type()`, line 2536 in `push_type_mismatch_if_needed()`). Add `TypeRegistry` as a field on the type checker context; replace both call sites with `registry.is_compatible()`. **Delete** `canonical_type_name()` from `ast_utils.rs` (14 call sites across daglang-typecheck and daglang-lower). The 8 call sites in daglang-lower (`insert_canonical_names`, `is_known_uses_type`, interface resolution at lines 600/611/4592/4608/4801/4815/4841/4851) need `TypeId`-based lookups instead of string splitting. Also delete `resolve_record_fields()` suffix matching (line 2583 `rsplit('.').next()`) — replace with registry lookup. | TS-1..TS-1d | M | |

### Phase I-C: Hard cutover — delete escape hatches

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-3** | **Make TypeRegistry non-optional** | `without_type_registry()` already deleted from builder. Remaining: change `type_registry: Option<TypeRegistry>` → `type_registry: TypeRegistry` in `core/codegen/src/testgen/codegen.rs` (line 235). Audit for any other `Option<TypeRegistry>` patterns. All callers must supply a registry. | TS-7 | S | |
| **TS-4** | **Delete PortType::Any catch-all** | Remove `_ => PortType::Any` in `parse_known_type()` (port_type.rs line 158). Remove `try_parse_port_type(s).unwrap_or(PortType::Any)` (line 216). Delete `From<&str> for PortType` impl that silently degrades unknowns to `Any` (line 141). Update `value_backing_for_type_id()` in `types.rs` (line 876, `PortType::Any =>` residual catch-all). Update `system_model.rs` (line 875, `PortType::Any => "gunbc_ir::Value"`). Either delete `PortType::Any` variant entirely or keep it as a non-wildcard (compatibility layer already restricts it — `Any` only matches `Any` per tests at line 230-234). | TS-1..TS-1d, TS-7 | M | |

### Phase I-D: Annotation processing

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-5** | **Process all annotations in typecheck** | `validate_type_expr` in daglang-typecheck skips non-`@range` annotations. Handle: `@content(encoding)` → `Predicate::Content`, `@brand(name)` → `TypeOp::Brand`, `@non_empty` → `Predicate::NonEmpty`, `@pattern(regex)` → `Predicate::Matches`, `@file_types { ... }` → extension→encoding map in TypeRegistry. | — | L | |

### Phase I-E: Test infrastructure

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **TS-2** | **Regenerate CI generated tests** | 2197 CI generated tests fail at runtime (`invalid 'items' input: expected StringList`). Pre-existing issue from collection dispatch changes. `CollectionDispatchOp` expects `StringList` format but mocks produce plain strings. Fix mock generation in `test_gen.rs` (`typed_mock_for_response` catch-all at line 155 returns `"mock-response"` for unknown types — should produce typed witnesses). | — | M | |

### Dependency graph

```
TS-1 ──┐
TS-1b ─┤ (TS-1b depends on TS-1 for type decisions)
TS-1c ─┼──→ TS-7 ──→ TS-3 ──→ (done)
TS-1d ─┘          └──→ TS-4 ──→ (done)

TS-5 ──→ (independent, can parallel with any phase)
TS-2 ──→ (independent, can parallel with any phase)
```

### Files touched

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
| `core/daglang/daglang-typecheck/src/lib.rs` | **Delete** `types_match()`, replace 2 call sites (TS-7). Annotation handling (TS-5) |
| `core/daglang/daglang-syntax/src/ast_utils.rs` | **Delete** `canonical_type_name()` (TS-7) |
| `core/daglang/daglang-lower/src/lib.rs` | Replace 8 `canonical_type_name()` call sites with TypeId lookups (TS-7) |
| `core/ir/src/port_type.rs` | **Delete** `_ => PortType::Any` catch-all, restrict/remove `PortType::Any` (TS-4) |
| `core/ir/src/types.rs` | Update `PortType::Any` arm in `value_backing_for_type_id()` (TS-4) |
| `core/ir/src/system_model.rs` | Update `PortType::Any` arm (TS-4) |
| `core/codegen/src/testgen/codegen.rs` | `Option<TypeRegistry>` → `TypeRegistry` (TS-3) |
| `core/daglang/daglang-emit/src/test_gen.rs` | Fix mock generation catch-all (TS-2) |

### Verification

1. `cargo build --workspace` — all crates compile with no `PortType::Any` fallback
2. `cargo test --workspace` — all tests pass including regenerated CI tests
3. `cargo clippy --all-targets -- -D warnings` — no warnings
4. Grep confirms: zero `types_match` call sites, zero `canonical_type_name` call sites, zero `Option<TypeRegistry>` patterns, zero `PortType::Any` catch-all arms

---

## Lane J: Compiler Test Fix + Pipeline Execution

**Context**: Last 2 sessions fixed testgen (27/27 targets), codegen (fresh), and workspace compilation. Four pre-existing test failures remain in `daglang-cli`, deps generated tests are stale, workspace subdag mapping is incomplete, and the SDLC pipeline hasn't been exercised end-to-end in dry-run mode. This lane clears all test failures and validates the SDLC pipeline.

**Mutual exclusivity**: Lane J touches `core/daglang/daglang-cli/` tests, `lib/tools/deps/` tests, `gunbc-dag/` workspace/sdlc, and `dsl/pipelines/`. Lane I does NOT touch any of these files.

### Step 0: Commit & PR baseline

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **J-0** | **Commit & PR current session changes** | Package all changes from last 2 sessions: resolve_config boundary mocks (4 graph_mock.rs files), HandlerKind variants (daglang-emit), TypeExpr::Record fixes (daglang-syntax, daglang-typecheck), credential_lifecycle.rs, workflow catalog, resolve.rs. Run `cargo clippy --workspace` before committing. Create PR against main. | — | S | |

### Phase J-A: Test green

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **J-1** | **Fix 4 failing daglang-cli makegen tests** | In `core/daglang/daglang-cli/src/compile/tests.rs`: `resolve_lowered_dag_maps_makegen_nodes_to_dyn_ops` assertion fails on `LoadRegistry` op debug format. 3 `compile_resolve_execute_makegen` tests also fail. Likely a `DynOp` display/debug format mismatch after the HandlerKind changes. | J-0 | S | |
| **J-2** | **Deps generated test freshness** | `lib/tools/deps/src/generated_tests.rs` has stale `FileResponse` struct (missing `bytes` field). Regenerate with `cargo run --bin gunbc-testgen`. Verify with `cargo test -p gunbc-deps`. | J-0 | S | |
| **TS-6** | **Workspace subdag mapping for reconciler/sdlc** | 2 workspace subdag tests fail: "unmapped DSL pipeline modules: reconciler, sdlc". Add module mappings in workspace subdag discovery or explicit exclusions. | J-0 | S | |

### Phase J-B: SDLC pipeline execution

| ID | Task | Deps | Size | Status |
|----|------|------|------|--------|
| **J-3** | **SDLC end-to-end dry-run** | Execute `dsl/pipelines/sdlc.dag` (555 lines) with `unit_test` profile. Use embedded test `sdlc_idea_to_design` (full mock chain). Verify all stages execute. Also run `dsl/pipelines/reconciler.dag` test `reconciler_converged_issues`. | J-1, J-2, TS-6 | M | |
| **J-4** | **Validate testgen + codegen freshness** | After all fixes, run full `cargo run --bin gunbc-testgen` and `cargo run --bin gunbc-codegen` to confirm all generated outputs are fresh. Run `cargo test --workspace` for final green. | J-3 | S | |

### Files touched

| File | Changes |
|------|---------|
| `core/daglang/daglang-cli/src/compile/tests.rs` | Fix 4 makegen test assertions (J-1) |
| `lib/tools/deps/src/generated_tests.rs` | Regenerate from testgen (J-2) |
| `gunbc-dag/src/` (workspace subdag) | Add reconciler/sdlc module mappings (TS-6) |

### Verification

1. `cargo test --workspace` — 224/224 pass (0 failures)
2. `cargo run --bin gunbc-testgen` — all 27 targets fresh
3. `cargo run --bin gunbc-codegen` — all outputs fresh
4. SDLC dry-run produces expected stage progression output

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

## Deferred

| ID | Task | Context | Size | Status |
|----|------|---------|------|--------|
| **DG1** | **Daggen (Dynamic DAG Generation)** | `needs_daggen()` returns false. Re-enable to scale the pipeline by dynamically generating steps based on git diffs. | L | **DEFERRED** |
| **S12-E** | **Multi-worker CAS** | Gap E: Implement `GcsClaimStore` with generation-based CAS (`x-goog-if-generation-match`). Not needed for single-worker local dev. | M | **DEFERRED** |

---

## Active Open Items (Deferred)

1. Resource wildcard pattern semantics remain explicitly deferred (`R2` + `backlog.md`).
