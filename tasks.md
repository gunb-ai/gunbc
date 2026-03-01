# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Operating Model: Three Lanes

Three parallel lanes, mutually independent, each covering a distinct
vertical slice of remaining work.

```
Lane 1: Compiler & Binary Elimination
  Red C + Red A — compiler refactor enables binary deletion
  (C1:C22, A1:A11)

Lane 2: Service Contracts & Transport
  Transport domain modeling + service layer + contract testing
  (TL-7,11:15, SL-1:11, CT-1:7)

Lane 3: SDLC Production
  Push hermetic pipeline to real cloud execution
  (SP-1:7)
```

### Protocols

**Independence**: Lanes touch different files. No merge conflicts.
Lane 1: `core/daglang/`, `core/codegen/`, `core/exec/`, `core/resolve/`, `gunbc-dag/src/`.
Lane 2: `dsl/services/`, `lib/transport/`, `core/test/`, `core/ir/src/transport/`.
Lane 3: `lib/cloud-ops/`, `lib/gcp-ops/`, `dsl/cloud/`, `dsl/workflows/`.

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

| # | ID | What | Acceptance Criteria | Size | Deps |
|---|-----|------|---------------------|------|------|
| 1 | C1 | **Stdlib host + caching.** `OnceLock` cache for compiled fn bodies. `include_str!` for stdlib sources. Single `StdLibHost::eval_fn()` interface. | `classify_callable()` never calls `compile_from_context()`. No `../../dsl` paths. | M | — |
| 2 | C2 | **Pipe methods first-class.** `PipeMethod` enum in syntax. Parser resolves `|> method()` to `PipeCall(PipeMethod, ...)`. Delete `should_track_call_name()` allowlist. | Allowlist deleted. `PipeMethod` has all 20 methods. | M | — |
| 3 | C3 | **Typed enums end-to-end.** `Value::Enum { ty, variant }`. Delete `TestClass::parse()` / `FermiCost::parse()` round-trips. | Zero `parse()` on classification. Zero `unwrap_or()` in fidelity. | M | — |
| 4 | C4 | **LoweringContext + dead code (staged).** Context struct grouping 8-11 params. Delete `#[allow(clippy::too_many_arguments)]`. Gated by C10. | Zero `too_many_arguments`. All `.dag` compile. | L | C1, C3 |
| 5 | C5 | **Integrate scope.rs.** Replace `detect_*_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` with `ScopedBody`. | `IfBranchSite` deleted. DAG parity. | M | — |
| 6 | C6 | **Extract transport derivation.** `transport.rs` module. Returns `TransportManifest` (pure data). | `add_service_transport_triplets` returns data, not mutates builder. | M | — |
| 7 | C7 | **Expr walker totality + typed leaf refs.** Explicit arms for all `Expr` variants. `LeafRef` enum. | Zero `_ => {}` in expr walkers. `PARAM_REF_SENTINEL` deleted. | M | — |
| 8 | C8 | **Delete dead AST scaffolding.** `MockResponseDef`, `error_cases()`, `@retry`. | `MockResponseDef` deleted. `@retry` rejected by parser. | S | — |
| 9 | C9 | **No panics, no silent parse.** `LowerError::InvalidTransportSpec` replaces `panic!`. | Zero `panic!` on user DSL. Parser test for `auth_input: "token"`. | S | — |
| 10 | C10 | **Resolve ReturnExprCompute split-brain + completeness gate (RT4a/c).** Desugar complex returns (BinOp/If/Match/Pipe/...) into explicit DAG semantics. | Zero `ReturnExprCompute` in any compiled graph. No silent return-binding drops. | L | — |
| 11 | C10a | **`make gist` auth credential bridge fix.** Postmortem Option A/B/C. Blocks C11. | `make gist` no longer 401s. | M | — |
| 12 | C11 | **Move resolve_service.rs to core/.** Physical move complete; now simplify: delete app-specific string dispatch, clean up inventory linkage. | Moved code is simpler than source. No dropped registrations. | L | C10a |
| 13 | C12 | **Move testgen to core/.** 5 files → `core/codegen/src/testgen/`. Delete gunbc-dag-specific assumptions. | `testgen_dag/` deleted from gunbc-dag. Testgen works from `core/codegen`. | M | — |
| 14 | C13 | **Split mock_defaults.** Generic probing (~350) → `core/test/`. Delete GCP blob (~230). | `mock_defaults.rs` deleted. Auto-mock works from `core/test`. | S | — |
| 15 | C14 | **REST status-code checking.** `GenericRestParseOp` checks status before field extraction. | 401 → structured error (not "field missing"). | M | — |
| 16 | C15 | **Fail-closed resolver audit.** Classify all `_ =>` fallbacks. Delete `passthrough_fallback_value()`. | Zero undocumented fallbacks. | M | — |
| 17 | C16 | **Transport class in node metadata.** `ServiceTransportClass` in lowered nodes. | `from_node_context` reads metadata, not substrings. | S | — |
| 18 | C17 | **Kill `propagate_to_param_sources`.** Fix boundary detection. | `propagate_to_param_sources` deleted. One port per input. | M | — |
| 19 | C18 | **Executor dead code.** Delete `looks_effectful_without_kind()`. | Dead code deleted. `cargo clippy` clean. | S | — |
| 20 | C19 | **Restore passthrough enforcement + diagnostics (RT4b).** Required outputs with no input → `ExecError` (not `Skipped`). | Missing passthrough ports are diagnosable. CI clean. | S | C4, C5, C7 |
| 21 | C21 | **CLI generator: KEY=VALUE and multi-value flags.** For `Map<String, String>` params, generate `KEY=VALUE` parser. Unblocks A5. | `gunbc-infra --input project_id=foo` parses to map. | M | — |
| 22 | C22 | **Deductive Redundancy Elimination (DRE).** Idempotency fingerprinting. Phase 1: compile-time `StaticFingerprint`. Phase 2: test-time execution ledger. See `docs/design/deductive-redundancy.md`. | Static fingerprint catches duplicate reads/writes at compile time. | L | — |

**Chain**: C1 → C3; C2; C10 (RT4a/c) → C4 → C5 → C6; C7; C8; C9; C10a → C11 → C14 → C15 → C19; C12; C13; C16; C17; C18; C21 (unblocks A5); C22

---

## Part B: Binary & Workflow Elimination (-8.4k net LOC)

Delete 5 hand-written binaries and the Rust workflow subsystem. Replace with
DSL data. After: every binary generated from DSL.

**Prerequisite**: C20 (profile-aware CLI generation) — **Done**.
C21 unblocks A5 (multi-value flag support).

| # | ID | What | Acceptance Criteria | Size | Deps |
|---|-----|------|---------------------|------|------|
| 1 | A7 | **Workflow catalog → DSL data.** `dsl/config/workflow_catalog.dag` with `data` for `WORKFLOW_VARIANTS`. | `catalog.rs` data section deleted. | M | — |
| 2 | A8 | **Unit commands → DSL data.** `dsl/config/workflow_commands.dag` with per-workflow `{ program, args }`. | `unit_commands.rs` deleted. | M | — |
| 3 | A9 | **Extract generic workflow to `core/workflow/`.** Move planner, executor, admission, coordination, slo, projection, proof, errors, schema, key (9 modules). | New `core/workflow/` crate. All tests pass. | L | — |
| 4 | A10 | **Delete binary infrastructure.** Remove `BinaryArgs` from `gunbc-cli`. | `BinaryArgs` deleted. | S | — |
| 5 | A1 | **Eliminate `sdlc.rs`.** Move param_source propagation. Delete binary. | `sdlc.rs` deleted. Generated binary works. | S | C20 ✓ |
| 6 | A2 | **Eliminate `deps_config.rs`.** | `deps_config.rs` deleted. `gunbc-deps-config --mode=ensure` works. | S | C20 ✓ |
| 7 | A3 | **Eliminate `pipeline.rs`.** Move `query_ci_status()` etc. to DSL. | `pipeline.rs` deleted. `gunbc-pipeline --depth 1` works. | M | C20 ✓ |
| 8 | A4 | **Eliminate `workflow.rs`.** Move plan rendering to DSL. | `workflow.rs` deleted. `gunbc-workflow plan` and `run` work. | L | C20 ✓ |
| 9 | A5 | **Eliminate `infra.rs`.** 8 subcommands → DSL. | `infra.rs` deleted. All 8 subcommands work. | L | C21 |
| 10 | A11 | **Delete compensating tests.** 7 `workflow_*.rs` + `infra_cli.rs`. | Files deleted. `cargo test --workspace` passes. | S | A1:A5 |

**Execution order**: A7 → A8 → A9 → A10 → A11 (parallel with A1-A5).
A1 → A2 → A3 immediately. A4 → A5 after C21.

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

| # | ID | What | Size | Deps |
|---|-----|------|------|------|
| 1 | TL-7 | **Mock response synthesis.** `core/test/src/mock_synthesis.rs`. Generate mock responses from `OperationBehavior` data. Replace `default_rest_response()` kitchen sink. | L | TL-3 ✓, TL-5 ✓ |
| 2 | TL-11 | **DSL syntax for transport blocks.** Add `rate_limit {}`, `retry {}`, `error_shape {}`, `credential {}` blocks to grammar. | L | TL-9 ✓ |
| 3 | TL-12 | **Lower transport blocks to IR.** Rate limit budgets → `RateLimitConfig`. Retry policies → `RetryConfig`. | M | TL-10 ✓, TL-11 |
| 4 | TL-13 | **Domain data migration.** Move hardcoded rate limits from Rust to `dsl/services/*.dag`. Delete provider-specific branches from `classify.rs`. | M | TL-12 |
| 5 | TL-14 | **Multi-target emit.** Emit transport configuration per target language. Rust links to Target SDK. Go/Python stubs for future. | XL | TL-13 |
| 6 | TL-15 | **Substrate cleanup.** `lib/transport/` becomes pure Target SDK. Delete `GITHUB_CORE_LIMIT` constants, `host.contains("github.com")` branches. | L | TL-14 |

**Chain**: TL-7 (independent); TL-11 → TL-12 → TL-13 → TL-14 → TL-15

---

## Part B: Service Layer Completion

Wire domain models (ED lane + Lane 4, both **Done**) into services: import
extdeps types, add `response {}` blocks, make the compiler enforce contracts.

| # | ID | What | Size | Deps |
|---|-----|------|------|------|
| 1 | SL-1 | **Wire extdeps → github services.** Import types from `extdeps/github/`. Replace inline type references. | M | ED ✓ |
| 2 | SL-2 | **Wire extdeps → llm services.** Import from `extdeps/llm/`. | S | ED ✓ |
| 3 | SL-3 | **Wire extdeps → gcp services.** Import from `extdeps/cloud/gcp/`. | M | ED ✓ |
| 4 | SL-4 | **Wire extdeps → shell/git/cargo services.** Import from `extdeps/git.dag`, `extdeps/cargo.dag`. | S | ED ✓, DM ✓ |
| 5 | SL-5 | **Enrich ED files with behavioral properties.** Add `OperationBehavior` data using DM-1 vocabulary. ~21 files enriched. | L | ED ✓, DM ✓ |
| 6 | SL-6 | **`response` block parsing (PC-1).** Add `response { STATUS => TYPE }` syntax. `Vec<ResponseEntry>` on `OperationDef`. Replaces dead `MockResponseDef`. | M | Lane 1 C8 |
| 7 | SL-7 | **`response` blocks on all REST services (PC-3).** 29 operations. Error shapes from `std/errors.dag`. | L | SL-6, SL-1:3 |
| 8 | SL-8 | **`exit` blocks on all shell services (PC-4).** Exit code → output type mapping. | M | SL-6 |
| 9 | SL-9 | **Lowerer: response → classify_response node (PC-5).** Compile entries to `ErrorMapping`. Generate classify_response node in transport DAG. | M | SL-6 |
| 10 | SL-10 | **GenericRestParseOp status checking (PC-6).** Route on status code before field extraction. Non-2xx → hard error. | M | SL-9 |
| 11 | SL-11 | **Completeness enforcement (PC-10).** Compiler requires ≥1 success + ≥1 error in `response {}`. | S | SL-6 |

**Chain**: SL-1:4 (parallel) → SL-5; SL-6 → SL-7:8, SL-9 → SL-10 → SL-11

---

## Part C: Contract Testing & Compliance

Build the automated infrastructure that proves providers comply with interface
contracts. Every interface gets a generated test suite. Every provider runs it.

| # | ID | What | Size | Deps |
|---|-----|------|------|------|
| 1 | CT-1 | **Contract IR.** Parse `contract` declarations into `ContractObligation` structs. Sequence/idempotency/destructive obligation types. | L | Lane 1 C done |
| 2 | CT-2 | **Contract test generation.** For each interface with `contract`, testgen emits parameterized test suite. | L | CT-1 |
| 3 | CT-3 | **Provider compliance wiring.** For each (profile, interface, provider) triple, instantiate CT-2. | M | CT-2 |
| 4 | CT-4 | **Annotation cleanup (Category 3).** Delete metadata noise annotations (~30 uses). | S | — |
| 5 | CT-5 | **`ProviderResponseContract` obligation (PC-7:9).** Per-status-code tests with mock body. Interface inheritance. | L | SL-9, CT-1 |
| 6 | CT-6 | **Restore deleted tests (RF-E5/E6).** Root causes fixed by Lane 1 C10/C19. | M | Lane 1 C10 |
| 7 | CT-7 | **DSL-derive CI secrets (RF-INV2).** Replace inventory linkage with derivation from DSL service annotations. | M | SL-6 |

**Chain**: CT-1 → CT-2 → CT-3 → CT-5; CT-4 (no deps); CT-6 (after C10); CT-7 (after SL-6)

---

# Lane 3: SDLC Production

## Philosophy

Take the hermetic SDLC pipeline to real cloud execution. Credential chaining,
Cloud Run deployment, real LLM agent, multi-worker CAS, CI integration.

Every item depends on the transport middleware (Lane 2 Phase 1 Target SDK — **Done**)
and service contracts (Lane 2 Part B) being in place.

| # | ID | What | Size | Deps |
|---|-----|------|------|------|
| 1 | SP-1 | **GCP credential chaining.** WIF OIDC → STS token exchange → impersonation → scoped access token. Uses `std/patterns.dag::credential_chain`. | L | ED ✓, DM ✓, TL-4 ✓ |
| 2 | SP-2 | **Cloud Run deployment DAG.** Service creation, revision deployment, traffic migration. Idempotent deploy. | L | SP-1, ED ✓ |
| 3 | SP-3 | **Agent provider: real LLM.** Wire `codex_agent.dag` to Anthropic/OpenAI via transport pipeline. | M | SL-2 |
| 4 | SP-4 | **Credential provider: local keychain.** Token storage for local profile. Encrypted file or OS keychain. | M | TL-4 ✓ |
| 5 | SP-5 | **Multi-worker CAS stress test.** 3 workers, exactly-once claim processing. GCS generation-based CAS. | M | SP-2, DM ✓ |
| 6 | SP-6 | **CI integration.** Hermetic pipeline in CI (unit_test profile). Cloud smoke test (env-gated). | M | SP-5 |
| 7 | SP-7 | **Webhook-driven stage transitions.** Cloud Run HTTP endpoint receives GitHub webhook events. | L | SP-2 |

**Chain**: SP-1 → SP-2 → SP-5 → SP-6; SP-3; SP-4; SP-7 (after SP-2)

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
| RF-TC3 | GCS stores, github_issue_provider, credential providers | 17 | Needed for cloud_run profile |
| RF-TC4 | Stub providers (unit_test profile) | 28 | Consider `transport stub {}` marker |
| RF-TC5 | Infrastructure stubs (azure, aws, gcp-infra) | 140 | Defer until provisioning lane |

## Deleted Tests (re-add when root cause fixed)

| ID | Deleted Tests | Blocker |
|----|---------------|---------|
| RF-E5 | `makegen_runtime_differential_interpreter_vs_generated_rust_layer1` | FnBodyDelegate gap |
| RF-E6 | `makegen_exec_runtime_e2e`, `pragma_exec_runtime_e2e`, `makegen_e2e_generated_binary`, `pragma_e2e_generated_binary` | Exec-runtime emitter |

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
| Heuristic reimplementation | `classify_module()` inflated by transitive auth callables | `gunbc-dag/src/fidelity.rs` | 2026-02-26 |
| String dispatch | `match field.type_id.as_str()` for JSON→Value appears twice | `core/resolve/src/service_ops/service_ops_impl.rs` | 2026-02-26 |
| String dispatch | `workflow_unit_commands()` matches workflow name strings | `gunbc-dag/src/workflow/unit_commands.rs` | 2026-02-26 |
| Dead scaffolding | `@mock_response` type exists in AST, parser never populates it | `daglang-syntax/src/parser.rs` | 2026-02-27 |
| Static mapping table | Kitchen sink `default_rest_response()` grows per service type | `core/test/src/auto_mock.rs` | 2026-02-27 |
| Dual convention | `from` path format split: `.` separator vs `/` separator | `dsl/services/` | 2026-02-27 |
| Heuristic reimplementation | `IdentityCallableOp` overloaded for 2 roles | `gunbc-dag/src/resolve.rs` | 2026-02-27 |
| Pessimistic ordering | `probe_best_response` tries `[Shell, File, REST]` — REST majority tried last | `core/test/src/auto_mock.rs` | 2026-02-27 |
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
