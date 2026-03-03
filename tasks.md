# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Operating Model: Three Lanes → Ship SDLC

All lanes oriented toward one goal: **SDLC runs reliably in production**.
Three parallel lanes, mutually independent, each covering a distinct
vertical slice of remaining work.

```
Lane 1: Compiler Critical Path
  Unblock SDLC execution — C10, C10a, then C24/C25/C26
  (C10:C26, A1:A4)

Lane 2: Service Contracts & Transport ✓
  All complete (TL-14, TL-15 done)

Lane 3: SDLC Ship
  Part B: Wire real transports → cloud profile works (SC-1:8)
  Part C: Harden for production — retry, observability, scale (SR-1:8)
```

### Critical Path

```
C10 (return expr) ──→ C10a (auth wiring) ──→ SC-1:6 (real transports) ──→ SC-8 (cloud e2e)
        L                    M                    6×M parallel                  M
                                                       │
                                              SR-1:4 (hardening, parallel)
                                                       │
                                              SR-5:7 (scale + deploy)
```

### Protocols

**Independence**: Lanes touch different files. No merge conflicts.
Lane 1: `core/daglang/`, `core/codegen/`, `core/exec/`, `core/resolve/`, `gunbc-dag/src/`.
Lane 2: `dsl/services/`, `lib/transport/`, `core/test/`, `core/ir/src/transport/`.
Lane 3: `dsl/services/sdlc/providers/`, `dsl/pipelines/`, `dsl/funcs/`, `lib/cloud-ops/`.

**Scouting**: Every PR includes a `Scouted:` line listing
opportunities for other lanes discovered during implementation.

**Non-blocking**: Lane 2 and 3 never block Lane 1. If cleanup is
needed for downstream progress, do the minimum fix inline.

---

# Lane 1: Compiler & Binary Elimination

## Philosophy

**Eliminate, Don't Relocate.** Every fix should push the problem
upstream — closer to the point of construction — so downstream code
can't encounter the bad state at all.

The test: *after your fix, can a future contributor reintroduce the
same class of bug?* If yes, you relocated it. If no, you eliminated it.

### Architectural Principles

1. **Pure functions, imperative shell.** Lowerer phases return typed data, not mutate shared state.
2. **Clear errors.** Every failure → typed error with span + stable error code. No panics on user input. No `_ => None` silent drops.
3. **Strong interfaces.** Pipe methods are an enum, not a string allowlist. Enums are values, not strings.
4. **Stdlib is a cached registry, not compile-on-demand.** No runtime `daglang_driver` calls from `core/codegen`. Embed sources, cache once.
5. **Minimal language core.** Lambdas: no capturing, no mutation, closed combinator set.
6. **Delete from app layer; generic infra may move to core but must shrink.**

---

## Part A: Compiler Pipeline Refactor

Restructure compiler into Google-style layer cake. Lowerer as pure functions.
Resolver fail-closed. Stdlib cached. Types strong. Testgen/resolve extracted to core/.

Target layout for `daglang-lower/src/`:
```
lib.rs        # public API + re-exports (~2k, down from 8.7k)
context.rs    # LoweringContext struct
callable.rs   # Phase 1: lower callables
scope.rs      # ScopedBody (replaces ad-hoc detect_*)
transport.rs  # Phase 3: derive transports → TransportManifest
wiring.rs     # Phases 4-6: derive edges → Vec<DerivedEdge>
resource.rs   # Phases 7-8: resource lifecycle
assembly.rs   # Final: assemble_dag(parts)
expr.rs       # LoweredExpr, LeafRef enum
eval.rs       # Pure evaluator
spec.rs       # Service operation specs
```

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | C1 | **Stdlib host + caching.** `OnceLock` cache for compiled fn bodies. `include_str!` for stdlib sources. Single `StdLibHost::eval_fn()` interface. | `classify_callable()` never calls `compile_from_context()`. No `../../dsl` paths. | M | **Done** |
| 2 | C2 | **Pipe methods first-class.** `PipeMethod` enum in syntax. Parser resolves `|> method()` to `PipeCall(PipeMethod, ...)`. Delete `should_track_call_name()` allowlist. | Allowlist deleted. `PipeMethod` has all 20 methods. | M | **Done** |
| 3 | C3 | **Typed enums end-to-end.** `Value::Enum { ty, variant }`. Delete `TestClass::parse()` / `FermiCost::parse()` round-trips. | Zero `parse()` on classification. Zero `unwrap_or()` in fidelity. | M | **Done** |
| 4 | C4 | **LoweringContext + dead code (staged).** Context struct grouping 8-11 params. Delete `#[allow(clippy::too_many_arguments)]`. Gated by C10. | Zero `too_many_arguments`. All `.dag` compile. | L | **Done** |
| 5 | C5 | **Integrate scope.rs.** Replace `detect_*_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` with `ScopedBody`. | `IfBranchSite` deleted. DAG parity. | M | **Done** |
| 6 | C6 | **Extract transport derivation.** `transport.rs` module. Returns `TransportManifest` (pure data). | `add_service_transport_triplets` returns data, not mutates builder. | M | **Done** |
| 7 | C7 | **Expr walker totality + typed leaf refs.** Explicit arms for all `Expr` variants. `LeafRef` enum. | Zero `_ => {}` in expr walkers. `PARAM_REF_SENTINEL` deleted. | M | **Done** |
| 8 | C8 | **Delete dead AST scaffolding.** `MockResponseDef`, `error_cases()`, `@retry`. | `MockResponseDef` deleted. `@retry` rejected by parser. | S | **Done** |
| 9 | C9 | **No panics, no silent parse.** `LowerError::InvalidTransportSpec` replaces `panic!`. | Zero `panic!` on user DSL. Parser test for `auth_input: "token"`. | S | **Done** |
| 10 | C10 | **Resolve ReturnExprCompute split-brain + completeness gate (RT4a/c).** Desugar complex returns (BinOp/If/Match/Pipe/...) into explicit DAG semantics. | Zero `ReturnExprCompute` in any compiled graph. No silent return-binding drops. | L | Partial |
| 11 | C10a | **`make gist` auth credential bridge fix.** Postmortem Option A/B/C. Blocks C11. | `make gist` no longer 401s. | M | Open |
| 12 | C11 | **Move resolve_service.rs to core/.** Physical move complete; app-specific dispatch consolidated into `extern_ops.rs`. | Moved code is simpler than source. No dropped registrations. | L | **Done** |
| 13 | C12 | **Move testgen to core/.** Core testgen library in `core/codegen/src/testgen/` (13.9k LOC). DAG integration in `gunbc-dag/src/testgen_dag/`. | Testgen works from `core/codegen`. Clean arch split. | M | **Done** |
| 14 | C13 | **Split mock_defaults.** Generic probing (~350) → `core/test/`. Delete GCP blob (~230). | `mock_defaults.rs` deleted. Auto-mock works from `core/test`. | S | **Done** |
| 15 | C14 | **REST status-code checking.** `GenericRestParseOp` checks status before field extraction. | 401 → structured error (not "field missing"). | M | **Done** (=SL-10) |
| 16 | C15 | **Fail-closed resolver audit.** Classify all `_ =>` fallbacks. Delete `passthrough_fallback_value()`. | Zero undocumented fallbacks. | M | **Done** |
| 17 | C16 | **Transport class in node metadata.** `ServiceTransportClass` in lowered nodes. | `from_node_context` reads metadata, not substrings. | S | **Done** |
| 18 | C17 | **Kill `propagate_to_param_sources`.** Fix boundary detection. | `propagate_to_param_sources` deleted. One port per input. | M | **Done** |
| 19 | C18 | **Executor dead code.** Delete `looks_effectful_without_kind()`. | Dead code deleted. `cargo clippy` clean. | S | **Done** |
| 20 | C19 | **Restore passthrough enforcement + diagnostics (RT4b).** Required outputs with no input → `ExecError` (not `Skipped`). | Missing passthrough ports are diagnosable. CI clean. | S | **Done** |
| 21 | C21 | **CLI generator: KEY=VALUE and multi-value flags.** For `Map<String, String>` params, generate `KEY=VALUE` parser. Unblocks A5. | `gunbc-infra --input project_id=foo` parses to map. | M | **Done** |
| 22 | C22 | **Deductive Redundancy Elimination (DRE).** Idempotency fingerprinting. Phase 1: compile-time `StaticFingerprint`. Phase 2: test-time execution ledger. See `docs/design/deductive-redundancy.md`. | Static fingerprint catches duplicate reads/writes at compile time. | L | Pending |
| 23 | C23 | **Hermetic AOT binaries (kill `CARGO_MANIFEST_DIR`).** 11 production files use `env!("CARGO_MANIFEST_DIR")` to read `.dag` files at runtime, hardcoding the developer's absolute path. Replace with `include_str!` to embed `.dag` sources at compile time, parsing AST purely in-memory. Bazel-style: binary runs on any machine. | Zero `env!("CARGO_MANIFEST_DIR")` in non-test code. Binaries run outside source tree. | L | **Done** |
| 24 | C24 | **Pure dataflow lowering (kill `ExprComputeOp` + `__` hack).** `ExprComputeOp` embeds a hidden AST interpreter in the executor. The lowerer rewrites `entry.kind` → `entry__kind`, forcing runtime Map flattening + `referenced_vars` pre-seeding with `Value::Skipped` to mask unbound variables. Desugar `BinOp`, `If`, `Match`, `FieldAccess` into primitive structural DAG nodes (`GetFieldNode`, `LogicalOrNode`, etc.). **Design doc**: `docs/design/pure-dataflow-lowering.md`. | Zero `ExprComputeOp` in any compiled graph. `__` convention deleted. `referenced_vars` deleted. | XL | Partial (step 1: `GetField` primitive op added — handles simple `param.field` projections. 175 ExprCompute nodes remain for compound expressions.) |
| 25 | C25 | **Service-driven codegen (kill handwritten ops).** The resolver hand-wires ~40 `DynOp` implementations for service operations (REST prepare/parse, shell prepare/parse, etc.). With response blocks (SL-7), exit blocks (SL-8), and transport class metadata (C16) in the IR, the compiler has enough information to generate these ops from service definitions. Delete `extern_ops.rs` dispatch table. **Design docs**: `docs/design/service-codegen.md` (protocol interfaces), `docs/design/pure-dataflow-lowering.md` §4 (migration path). | Zero handwritten `DynOp` for services. `extern_ops.rs` dispatch table derived from DSL. | XL | Open (needs C24) |
| 26 | C26 | **Incremental compilation.** Every `cargo run --bin gunbc-codegen-dag` recompiles all `.dag` files from scratch. With hermetic AOT binaries (C23) and static fingerprints (C22), the compiler can skip recompilation of unchanged modules. Hash `.dag` source → compare to cached IR → emit only changed artifacts. | Unchanged `.dag` files skip parse+lower+emit. 10x speedup on incremental edits. | **Done** |

**Remaining open**: C24 (XL — step 1 done, compound expression decomposition remains), C25 (XL — depends on C24)

---

## Part B: Binary & Workflow Elimination (-8.4k net LOC)

Delete 5 hand-written binaries and the Rust workflow subsystem. Replace with
DSL data. After: every binary generated from DSL.

**Prerequisite**: C20 (profile-aware CLI generation) — **Done**.
C21 unblocks A5 (multi-value flag support).

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | A7 | **Workflow catalog → DSL data.** `dsl/config/workflow_catalog.dag` with `data` for `WORKFLOW_VARIANTS`. | `catalog.rs` data section deleted. | M | **Done** |
| 2 | A8 | **Unit commands → DSL data.** `dsl/config/workflow_commands.dag` with per-workflow `{ program, args }`. | `unit_commands.rs` deleted. | M | **Done** |
| 3 | A9 | **Extract generic workflow to `core/workflow/`.** Move planner, executor, admission, coordination, slo, projection, proof, errors, schema, key (9 modules). | New `core/workflow/` crate. All tests pass. | L | **Done** |
| 4 | A10 | **Delete binary infrastructure.** Remove `BinaryArgs` from `gunbc-cli`. | `BinaryArgs` deleted. | S | **Done** |
| 5 | A1 | **Eliminate `sdlc.rs`.** Binary deleted. Stub exists in `target/codegen/bin/sdlc/main.rs`. Need full DSL tool def with param_source propagation. | Generated binary has feature parity. `make sdlc` works. | S | Partial (binary deleted, stub needs DSL tool def) |
| 6 | A2 | **Eliminate `deps_config.rs`.** Binary deleted. Stub exists. Need DSL tool def for verify/ensure modes. | Generated binary has feature parity. `make deps-config` works. | S | Partial (binary deleted, stub needs DSL tool def) |
| 7 | A3 | **Eliminate `pipeline.rs`.** Binary deleted. Stub exists. Need DSL tool def with `query_ci_status()`. | Generated binary has feature parity. `make pipeline` works. | M | Partial (binary deleted, stub needs DSL tool def) |
| 8 | A4 | **Eliminate `workflow.rs`.** Binary deleted. Stub exists. Need DSL tool def with plan rendering. | Generated binary has feature parity. `make workflow` works. | L | Partial (binary deleted, stub needs DSL tool def) |
| 9 | A5 | **Eliminate `infra.rs`.** Binary deleted. Full generated binary exists (159 LOC). 8 subcommands working. | All 8 subcommands work via generated binary. | L | **Done** |
| 10 | A11 | **Delete compensating tests.** 7 `workflow_*.rs` + `infra_cli.rs`. | Files deleted. `cargo test --workspace` passes. | S | **Done** (CT-6 restored tests) |

**Remaining open**: A1-A4 have binaries deleted but stubs need full DSL tool definitions for feature parity. A5 is **Done** (fully generated).

---

# Lane 2: Service Contracts & Transport

## Philosophy

Make services **correct by construction**: import typed domain models,
declare error responses, classify at the transport layer, generate compliance
tests. Follows the Protobuf/gRPC pattern: compiler generates configuration
code that links to a Target SDK.

---

## Part A: Transport Domain Modeling (TL Phase 2)

Phase 1 (Target SDK: TL-0:10) is **Done**. Phase 2 moves domain-specific
configuration (rate limits, error shapes) from Rust into `.dag` files.

**Design doc**: `docs/design/transport-primitives.md`

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | TL-7 | **Mock response synthesis.** `core/test/src/mock_synthesis.rs`. Generate mock responses from `OperationBehavior` data. Replace `default_rest_response()` kitchen sink. | L | **Done** |
| 2 | TL-11 | **DSL syntax for transport blocks.** Add `rate_limit {}`, `retry {}`, `error_shape {}`, `credential {}` blocks to grammar. | L | **Done** |
| 3 | TL-12 | **Lower transport blocks to IR.** Rate limit budgets → `RateLimitConfig`. Retry policies → `RetryConfig`. | M | **Done** |
| 4 | TL-13 | **Domain data migration.** Move hardcoded rate limits from Rust to `dsl/services/*.dag`. Delete provider-specific branches from `classify.rs`. | M | **Done** |
| 5 | TL-14 | **Multi-target emit.** Emit transport configuration per target language. Rust links to Target SDK. Go/Python stubs for future. | XL | **Done** |
| 6 | TL-15 | **Substrate cleanup.** `lib/transport/` becomes pure Target SDK. Delete `GITHUB_CORE_LIMIT` constants, `host.contains("github.com")` branches. | L | **Done** |
| 7 | TL-16 | **Dynamic JSON-path error shapes.** Lower `error_shape {}` blocks into JSON-path extraction rules in IR. Delete `ResponseProvider` enum, `infer_response_provider()`, hardcoded `parse_*_error` functions. Transport layer blindly executes JSON-path extractions. | L | **Done** |

**All TL tasks complete.**

---

## Part B: Service Layer Completion

Wire domain models (ED lane + Lane 4, both **Done**) into services: import
extdeps types, add `response {}` blocks, make the compiler enforce contracts.

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | SL-1 | **Wire extdeps → github services.** Import types from `extdeps/github/`. | M | **Done** |
| 2 | SL-2 | **Wire extdeps → llm services.** Import from `extdeps/llm/`. | S | **Done** |
| 3 | SL-3 | **Wire extdeps → gcp services.** Import from `extdeps/cloud/gcp/`. | M | **Done** |
| 4 | SL-4 | **Wire extdeps → shell/git/cargo services.** Import from `extdeps/git.dag`, `extdeps/cargo.dag`. | S | **Done** |
| 5 | SL-5 | **Enrich ED files with behavioral properties.** Add `OperationBehavior` data using DM-1 vocabulary. | L | **Done** |
| 6 | SL-6 | **`response` block parsing (PC-1).** Add `response { STATUS => TYPE }` syntax. | M | **Done** |
| 7 | SL-7 | **`response` blocks on all REST services (PC-3).** 29 operations. | L | **Done** |
| 8 | SL-8 | **`exit` blocks on all shell services (PC-4).** Exit code → output type mapping. | M | **Done** |
| 9 | SL-9 | **Lowerer: response → classify_response node (PC-5).** Compile entries to `ErrorMapping`. | M | **Done** |
| 10 | SL-10 | **GenericRestParseOp status checking (PC-6).** Route on status code before field extraction. | M | **Done** |
| 11 | SL-11 | **Completeness enforcement (PC-10).** Compiler requires ≥1 success + ≥1 error in `response {}`. | S | **Done** |

**All SL tasks complete.**

---

## Part C: Contract Testing & Compliance

Build the automated infrastructure that proves providers comply with interface
contracts. Every interface gets a generated test suite. Every provider runs it.

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | CT-1 | **Contract IR.** Parse `contract` declarations into `ContractObligation` structs. Sequence/idempotency/destructive obligation types. | L | **Done** |
| 2 | CT-2 | **Contract test generation.** For each interface with `contract`, testgen emits parameterized test suite. | L | **Done** |
| 3 | CT-3 | **Provider compliance wiring.** For each (profile, interface, provider) triple, instantiate CT-2. | M | **Done** |
| 4 | CT-4 | **Annotation cleanup (Category 3).** Delete metadata noise annotations (~30 uses). | S | **Done** |
| 5 | CT-5 | **`ProviderResponseContract` obligation (PC-7:9).** Per-status-code test generation, coverage validation. | L | **Done** |
| 6 | CT-6 | **Restore deleted tests.** 46 workflow contract tests restored across 8 files. All pass with C10/C19 fixes. | M | **Done** |
| 7 | CT-7 | **DSL-derive CI secrets (RF-INV2).** Replace inventory linkage with derivation from DSL service annotations. | M | **Done** |

**All CT tasks complete.**

---

# Lane 3: SDLC Production

## Philosophy

Take the hermetic SDLC pipeline to real cloud execution. Credential chaining,
Cloud Run deployment, real LLM agent, multi-worker CAS, CI integration.

Every item depends on the transport middleware (Lane 2 Phase 1 Target SDK — **Done**)
and service contracts (Lane 2 Part B) being in place.

## Part A: Cloud Infrastructure (SP-1:7) — DONE

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | SP-1 | **GCP credential chaining.** WIF OIDC → STS token exchange → impersonation → scoped access token. | L | **Done** |
| 2 | SP-2 | **Cloud Run deployment DAG.** Service creation, revision deployment, traffic migration. | L | **Done** |
| 3 | SP-3 | **Agent provider: real LLM.** Wire `codex_agent.dag` to Anthropic/OpenAI via transport pipeline. | M | **Done** |
| 4 | SP-4 | **Credential provider: local keychain.** Token storage for local profile. | M | **Done** |
| 5 | SP-5 | **Multi-worker CAS stress test.** 3 workers, exactly-once claim processing. GCS generation-based CAS. | M | **Done** |
| 6 | SP-6 | **CI integration.** Hermetic pipeline in CI (unit_test profile). Cloud smoke test (env-gated). | M | **Done** |
| 7 | SP-7 | **Webhook-driven stage transitions.** Cloud Run HTTP endpoint receives GitHub webhook events. | L | **Done** |

**All SP tasks complete.**

---

## Part B: SDLC Runs in Cloud

Wire real transport operations for cloud profile. After this part, `gunbc-sdlc --profile cloud_run`
executes against real GitHub, GCS, and LLM providers.

**Prerequisites**: C10 (complex return expr), C10a (auth credential wiring) — both Lane 1.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | SC-1 | **Wire GCS claim store transport.** Replace stub transport blocks in `gcs_claim_store.dag` with real `transport rest {}` blocks pointing to GCS JSON API. Wire `acquire` (conditional PUT with `ifGenerationMatch`), `heartbeat` (PATCH), `release` (DELETE). | `cloud_run` profile claim operations execute against real GCS. Generation-based CAS works. | M | **Done** |
| 2 | SC-2 | **Wire GCS outcome ledger transport.** Replace stubs in `gcs_outcome_ledger.dag` with real `transport rest {}`. Wire `upsert` (PUT) and `get` (GET) with JSON response parsing. | `cloud_run` profile outcome recording works against real GCS. | S | **Done** |
| 3 | SC-3 | **Wire GCS artifact store transport.** Replace stubs in `gcs_artifact_store.dag` with real `transport rest {}`. Wire `store` (PUT with content-hash path), `retrieve` (GET), `metadata` (HEAD). | `cloud_run` profile artifact operations work against real GCS. | M | **Done** |
| 4 | SC-4 | **Wire GitHub issue provider transport.** Replace stubs in `github_issue_provider.dag` with real `transport rest {}` blocks for `discover` (GET /issues), `get` (GET /issues/:id), `comment` (POST /issues/:id/comments), `set_labels` (PUT /issues/:id/labels), `close` (PATCH /issues/:id). | `cloud_run` + `local` profile issue operations work against real GitHub API. | M | **Done** |
| 5 | SC-5 | **Wire Pub/Sub signal store transport.** Replace stubs in `pubsub_signal_store.dag` with real `transport rest {}` for Pub/Sub API. Wire `emit` (POST /publish), `consume` (POST /pull), `ack` (POST /acknowledge). | `cloud_run` profile stage-transition signals flow through real Pub/Sub. | M | **Done** |
| 6 | SC-6 | **Wire credential providers.** Real transport for `gcloud auth print-access-token` (shell), Secret Manager `GET /secrets/:name/versions/latest:access` (REST). | `cloud_run` profile credential chain works end-to-end. No hardcoded tokens. | M | **Done** |
| 7 | SC-7 | **auth_input compile-time validation.** Validate that `config { auth_input: X }` references an actual field in the operation's `input {}` block. Validate field type is `Secret`. Emit compiler error if missing or wrong type. | `config { auth_input: nonexistent }` → compiler error with span. Zero silent credential skips. | S | **Done** |
| 8 | SC-8 | **Cloud profile e2e smoke test.** Integration test: create GitHub issue with `sdlc:idea` label → SDLC pipeline discovers, claims, executes design stage → verify outcome recorded in GCS. Env-gated (`GUNBC_CLOUD_E2E=true`). | `cargo test -- cloud_e2e` passes when env-gated. Exercises SC-1:6 end-to-end. | M | **Done** |

**All SC tasks complete.**

---

## Part C: SDLC Runs Reliably

Production hardening. After this part, SDLC runs unattended with
observability, retry resilience, and zero-downtime deploys.

| # | ID | What | Acceptance Criteria | Size | Status |
|---|-----|------|---------------------|------|--------|
| 1 | SR-1 | **Approval gate wiring.** `pending_approval` exit code 42 triggers claim release + yield. Approval signal (GitHub comment/label) resumes work on same issue. | Stage pauses at approval gate. Approval signal resumes within 60s. No duplicate work. | M | **Done** |
| 2 | SR-2 | **Retry budget persistence.** Retry count survives process restart. Store in outcome ledger alongside stage result. Enforce max retries per stage (configurable, default 3). | Failed stage retries up to budget. Budget persists across restarts. Budget=0 → terminal failure. | M | **Done** |
| 3 | SR-3 | **Structured execution logging.** JSON-structured log lines with `trace_id`, `issue_id`, `stage`, `node_id`, `duration_ms`. Cloud Logging compatible. | Every transport node emits structured log. Logs are queryable by `trace_id`. | M | **Done** |
| 4 | SR-4 | **Health check endpoint.** Cloud Run health check: `/health` returns 200 + worker status (idle/processing/draining). Graceful shutdown on SIGTERM (finish current stage, release claims). | Cloud Run readiness probe passes. SIGTERM → clean claim release within 30s. | S | **Done** |
| 5 | SR-5 | **Multi-worker scale test.** 5 workers, 20 concurrent issues, 10-minute run. Verify: no duplicate claims, no lost outcomes, all issues reach terminal state. | Zero claim conflicts. Zero lost outcomes. All 20 issues terminal within 10 min. | M | **Done** |
| 6 | SR-6 | **Rolling deploy with zero downtime.** Cloud Run traffic migration: deploy new revision → route 10% → validate → route 100%. Rollback on health check failure. | Deploy completes with zero dropped webhook events. Rollback tested. | M | **Done** |
| 7 | SR-7 | **Anti-entropy reconciliation.** Periodic scan (every 5 min): find issues with stale claims (no heartbeat > 2 min), orphaned outcomes (no matching claim), stuck stages. Auto-recover or alert. | Stale claims auto-released. Orphaned outcomes flagged. Stuck stages retried or escalated. | L | **Done** |
| 8 | SR-8 | **config.extra → typed provider config.** Parse unknown config values into `Expr` (not token-skipped strings). Move provider config schemas into `.dag` models. Validate config fields at compile time. | Zero `config.extra` entries. All provider config fields typed and validated. Config typo → compiler error. | M | **Done** |

**All SR tasks complete.**

---

# Phase 3: The Purist Engine (Aspirational)

Final stages of the compiler refactor. Eliminate all runtime interpretation,
achieve fully hermetic AOT compilation, and strong-type every boundary.

C24 (Pure Dataflow Lowering) is the keystone dependency — tracked in Lane 1 Part A above.
C27 and SR-8 cover related ground (typed config blocks).

| # | ID | Task | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 1 | C28 | **Daggen (AOT DAG Compilation).** Currently, generated CLI tools (`gunbc-sdlc`, etc.) parse and resolve `.dag` files dynamically at runtime via `build_dsl_graph_with_profile`. Implement "Daggen" to compile lowered DAGs directly into static `Dag<T>` Rust structs during `make codegen`. The final binaries should contain zero DSL parsing/resolution logic, becoming fully hermetic AOT executables. | XL | Pending | C24 |
| 2 | C29 | **Dynamic JSON-Path Output Mappings.** Just like `error_shape` (TL-16), extend JSONPath extraction to successful responses. Lower output mappings into extraction rules (e.g., `issue_id: "$.id"`) so the Rust runtime doesn't need hardcoded struct extraction logic. | M | Pending | TL-16 |
| 3 | C30 | **Strict Type-Aware JSON Bridging.** `value_bridge.rs` currently hijacks the JSON keys `__enum` and `__bytes` to reconstruct complex `Value` types. This is "in-band signaling" and could collide with actual API payloads. Make `from_bridge_json` type-aware by passing the expected `TypeId` (which is known statically from the port). Delete the `__enum` JSON dictionary hacks. **Consolidation scope (9 locations):** The hardcoded `PRIMITIVE_TYPE_IDS` list in `value_bridge.rs:143` is duplicated across the codebase — `type_registry.rs` has `value_backing()` (line 733, 8 primitives), `PRIMITIVE_BACKINGS` (line 746, 6 tuples), and `semantic_carrier_kind_for_type_id()` (line 983, 40+ type names); `service_ops_impl.rs` has REST response parsing (line 401) and skipped defaults (line 474); `auto_mock.rs:30` has mock value generation; `daglang-lower/lib.rs:6039` has shell output type dispatch. All duplicate knowledge the DSL type system already has. Fix: emit a structural `TypeShape` discriminant (`Enum \| Record \| Primitive \| Container`) from the compiler so downstream code dispatches on shape, not parallel string lists. | M | Pending | — |
| 4 | C27 | **Typed Config Blocks.** The parser currently skips unknown tokens in `config {}` blocks and stores them as stringified `extra: Vec<(String, String)>`. Replace this pragmatic fallback with strongly typed AST parsing for provider-specific config fields (e.g. `bucket: String`, `model: String`). | S | **Done** | — |
| 5 | CT-8 | **Wire Contract Test Generation.** Connect the new `StructuredContract` and `ProviderResponseContract` infrastructure in `core/ir/src/contract.rs` to `gunbc-testgen`. Emit S-Tier hermetic tests that mathematically prove every provider binding obeys the `.dag` behavioral contracts. | M | Pending | CT-1 |

---

# Backlog

Triaged items not yet assigned to a lane. Promote when lane queues thin.

## Compiler Features (low priority)

| ID | Feature | Size |
|----|---------|------|
| FC-CF2 | `skip(n)`: List\<T\> → List\<T\> | S |
| FC-CF3 | `enumerate()`: List\<T\> → List\<(Int, T)\> | M |

## Transport Completeness (remaining providers)

| ID | Scope | Ops | Notes |
|----|-------|-----|-------|
| RF-TC3 | GCS stores, github_issue_provider, credential providers | 17 | **Promoted to SC-1:6** |
| RF-TC4 | Stub providers (unit_test profile) | 28 | Consider `transport stub {}` marker |
| RF-TC5 | Infrastructure stubs (azure, aws, gcp-infra) | 140 | Defer until provisioning lane |

## Deleted Tests (re-add when root cause fixed)

Most workflow contract tests restored in CT-6 (46 tests across 8 files).

| ID | Deleted Tests | Blocker | Status |
|----|---------------|---------|--------|
| RF-E5 | `makegen_runtime_differential_interpreter_vs_generated_rust_layer1` | FnBodyDelegate gap | Open |
| RF-E6 | `makegen_exec_runtime_e2e`, `pragma_exec_runtime_e2e`, `makegen_e2e_generated_binary`, `pragma_e2e_generated_binary` | Exec-runtime emitter | Open |

## Blue Backlog

| ID | Item | Size | Priority |
|----|------|------|----------|
| H10 | Compute stack orchestration: Cloud Run/GCS/LB lifecycle DAG builder | L | P2 |
| S12-E | Multi-worker CAS: GcsClaimStore with generation-based CAS | M | P2 |
| H1 | Display reactive DSL: channel-driven event loop | XL | P3 |

---

# Unqueued Observations

Raw observations from any worker. Not triaged, not sized.

| Smell | Observation | File | Date |
|-------|-------------|------|------|
| Heuristic reimplementation | `passthrough_fallback_value()` hard-codes a port alias table | `gunbc-dag/src/resolve.rs` | 2026-02-26 |
| Heuristic reimplementation | `looks_effectful_without_kind()` re-derives NodeKind from port strings. Dead after RT6. | `core/exec/src/execute.rs` | 2026-02-26 |
| ~~Heuristic reimplementation~~ | ~~`classify_module()` inflated by transitive auth callables~~ — **RESOLVED**: documented transitive auth inflation in doc comment | `gunbc-dag/src/fidelity.rs` | 2026-02-26 |
| String dispatch | `match field.type_id.as_str()` for JSON→Value appears twice | `core/resolve/src/service_ops/service_ops_impl.rs` | 2026-02-26 |
| String dispatch | `workflow_unit_commands()` matches workflow name strings | `gunbc-dag/src/workflow/unit_commands.rs` | 2026-02-26 |
| Dead scaffolding | `@mock_response` type exists in AST, parser never populates it | `daglang-syntax/src/parser.rs` | 2026-02-27 |
| Static mapping table | Kitchen sink `default_rest_response()` grows per service type | `core/test/src/auto_mock.rs` | 2026-02-27 |
| ~~Dual convention~~ | ~~`from` path format split: `.` separator vs `/` separator~~ — **RESOLVED**: standardized `head.sha`→`head/sha`, `base.ref`→`base/ref` in pull_request.dag | `dsl/services/` | 2026-02-27 |
| ~~Heuristic reimplementation~~ | ~~`IdentityCallableOp` overloaded for 2 roles~~ — **RESOLVED**: split into `OutputPathMetadataOp` + `ResourcePassthroughOp` | `gunbc-dag/src/resolve.rs` | 2026-02-27 |
| ~~Pessimistic ordering~~ | ~~`probe_best_response` tries `[Shell, File, REST]` — REST majority tried last~~ — **RESOLVED**: reordered to REST-first in both `probe_best_response` and `default_value_for_slot` | `core/test/src/auto_mock.rs` | 2026-02-27 |
| Inventory linkage gap | `gunbc-codegen cigen` drops GCP secrets | `gunbc-dag/src/ci/mod.rs` | 2026-02-26 |

---

# Postmortems

## `make gist` 401 — Compounding Failures

**Symptom**: `make gist` returns 401 Unauthorized. `BearerToken (key: "", source: static)`.

Five compounding failures: (1) No credential wiring from operation inputs to execute node (ROOT CAUSE), (2) GenericRestPrepareOp doesn't propagate auth_token, (3) Execute node falls through silently, (4) Diagnostic source is misleading, (5) GenericShellParseOp was emitting Value::Str for Secret outputs (**Fixed**).

**Fix options**: Option A (auto-bridge in lowerer), Option B (explicit `auth_input` in DSL config), Option C (RestPrepareOp sets auth on request). Assigned as **C10a** — hard precondition for C11.

## `gunbc-ci` false failure — `overall_success: Skipped`

**Symptom**: `gunbc-ci` reports "required success check returned false" even when all stages succeed. Root cause: lowerer drops complex return expressions (`BinOp`, `If`, `Match`, `Pipe` → `_ => None`). Fix plan: **C10** (RT4a — complex return expr lowering), **C19** (RT4b — passthrough diagnostics), **C10** (RT4c — lowering completeness gate).

## Testgen Discovery Bug (BT-R1)

**Root cause**: `discover_compilable_modules()` used `func_count == 0` as pre-filter, silently dropping modules with `pipeline`, `pattern`, or `fn` items. **Fixed**: renamed to `callable_count`, broadened filter.

---

# Archive: Completed Lanes

## SDLC Activation (Blue Lane 1) — DONE

16 tasks (BT1:BT-E1). Full SDLC pipeline: compile, wiring, hermetic/integration tests, transport declarations, testgen, CLI entrypoint, signal/artifact store providers, execution gap fixes, transport deduplication.

## External Dependency Modeling (Blue Lane 2) — DONE

21 tasks (ED-1:21). All external system models: cloud core, GitHub (issues/PRs/gists), LLM (core/anthropic/openai), GCP (core/storage/pubsub/IAM/secret manager/cloud run/STS), Git, Cargo, AWS (core/s3/IAM/lambda/secrets manager/SQS), Azure (core/blob/identity/container apps/key vault/service bus).

## Registry & Extern Deletion (Red Worker B) — DONE

10 tasks (B1:B10). Gitignore, makegen registry, resource definitions, docgen targets, pragma.rs, extern_impls.rs, tool wrappers, embedded_assets.rs, compensating tests, makegen cleanup.

## Domain Model Foundation (Phase 2 Lane 4) — DONE

22 tasks (DM-1:22). Standard vocabulary (behavioral, rate limit, coordination, errors, capability). Secret providers (core, GCP, GitHub, env file, Vault). Coordination stores (core, GCS, PostgreSQL, SQLite). Tool lifecycle (Rust, GitHub CLI, package managers). Complementary models (devcontainers, LLM pricing, API ops). Interface enrichment with behavioral contracts.

## Transport Target SDK (Phase 2 Lane 5 Phase 1) — DONE

10 tasks (TL-0:10 except TL-7). Foundation types, rate limit middleware, retry middleware, response classification, credential middleware, virtual HTTP/shell backends, transport metrics, middleware composition, IR transport types.

## Historical Red Team Completions

| ID | What | Status |
|----|------|--------|
| C20 | CLI generator: profile, mode, subcommand support | Done |
| RT1:4 | Credential wiring, execute fail-closed, file transport, transport block validation | Done |
| RT5:8 | Fold extraction, NodeKind required, PortCategory, TransportRole | Done |
| RT9:12 | Virtual I/O DSL types, shell/HTTP/TCP registries | Done |
| RT17:28 | Port constants, StringEnum derive, ModulePath, DslTypeMapping, split monoliths | Done |
| RT-A1:A5 | Auth postmortem analysis tasks | Done |
| RT-I3:I6 | Credential chain integrity, shell exit, gist migration, e2e verify | Done |
| NF-1:6 | Compile+link hardening | Done |
| FC-NF7 | fn-level evaluation | Done |
| FC-CL | Dead code cleanup | Done |
| FC-EG | Enforcement gates | Done |
| FC-P6/P7 | Policy migration, build_workflows, artifact emitter, Makefile DSL types | Done |
| FC-CF1/CF7 | split + zip pipe methods | Done |
| BB-0:6 | Compositional type modeling, mock corpus, type witnesses, fidelity ladders | Done |
| RF-H2/H4 | TestgenTargetDef non-Option, ResourceKind enum | Done |
| RF-E4 | Fidelity smoke test | Done |
| CG-1:5 | DSL CI YAML generation (cigen) | Done |

## Design References

| Document | What |
|----------|------|
| `docs/design/sdlc/mega-modeling-design.md` | Canonical architecture: 9 boxes, core abstractions, contracts |
| `docs/design/sdlc/domain-modeling-comprehensive.md` | All domain objects, state machines, invariants |
| `docs/design/provider-contracts.md` | Provider response contracts, testgen obligations |
| `docs/design/contract-testing.md` | Contract IR, test generation, compliance |
| `docs/design/deductive-redundancy.md` | DRE: idempotency fingerprinting design |
| `docs/design/transport-primitives.md` | Transport domain modeling (Phase 2) |
| `docs/design/domain-model-porting.md` | Behavioral properties, secrets, coordination, tools, LLM pricing |
| `docs/design/mock-response-pipeline.md` | Mock response synthesis pipeline |
| `SPEC.md` | Formal IR specification |
| `docs/design/v4/dsl-design.md` | Full DSL language specification |
