# Tasks Archive — 2026-03-02

Archived from `tasks.md`. All items verified against acceptance criteria.

---

## Lane 1 Part A: Compiler Pipeline Refactor (Completed Items)

| # | ID | What | Size | Status | Verified |
|---|-----|------|------|--------|----------|
| 1 | C1 | **Stdlib host + caching.** `OnceLock` cache for compiled fn bodies. `include_str!` for stdlib sources. | M | Done | `classify_callable()` never calls `compile_from_context()`. No `../../dsl` paths. |
| 2 | C2 | **Pipe methods first-class.** `PipeMethod` enum in syntax. Parser resolves `|> method()` to `PipeCall(PipeMethod, ...)`. | M | Done | Allowlist deleted. `PipeMethod` has all 20 methods. |
| 3 | C3 | **Typed enums end-to-end.** `Value::Enum { ty, variant }`. Delete `TestClass::parse()` / `FermiCost::parse()` round-trips. | M | Done | Zero `parse()` on classification. Zero `unwrap_or()` in fidelity. |
| 4 | C4 | **LoweringContext + dead code (staged).** Context struct grouping 8-11 params. | L | Done | Zero `too_many_arguments`. All `.dag` compile. |
| 5 | C5 | **Integrate scope.rs.** Replace `detect_*_branches_in_stmts`, `IfBranchSite`, `MatchBranchSite` with `ScopedBody`. | M | Done | `IfBranchSite` deleted. DAG parity. |
| 6 | C6 | **Extract transport derivation.** `transport.rs` module. Returns `TransportManifest` (pure data). | M | Done | `add_service_transport_triplets` returns data, not mutates builder. |
| 7 | C7 | **Expr walker totality + typed leaf refs.** Explicit arms for all `Expr` variants. `LeafRef` enum. | M | Done | Zero `_ => {}` in expr walkers. `PARAM_REF_SENTINEL` deleted. |
| 8 | C8 | **Delete dead AST scaffolding.** `MockResponseDef`, `error_cases()`, `@retry`. | S | Done | `MockResponseDef` deleted (confirmed 2026-03-02). `@retry` rejected by parser. |
| 9 | C9 | **No panics, no silent parse.** `LowerError::InvalidTransportSpec` replaces `panic!`. | S | Done | Zero `panic!` on user DSL. Parser test for `auth_input: "token"`. |
| 10 | C11 | **Move resolve_service.rs to core/.** | L | Done | Moved code simpler than source. No dropped registrations. |
| 11 | C12 | **Move testgen to core/.** Core testgen library in `core/codegen/src/testgen/` (13.9k LOC). | M | Done | Testgen works from `core/codegen`. Clean arch split. |
| 12 | C13 | **Split mock_defaults.** Generic probing → `core/test/`. Delete GCP blob. | S | Done | `mock_defaults.rs` deleted. Auto-mock works from `core/test`. |
| 13 | C14 | **REST status-code checking.** `GenericRestParseOp` checks status before field extraction. | M | Done | 401 → structured error (not "field missing"). |
| 14 | C15 | **Fail-closed resolver audit.** Classify all `_ =>` fallbacks. Delete `passthrough_fallback_value()`. | M | Done | Zero undocumented fallbacks (confirmed deleted in 33513ac9). |
| 15 | C16 | **Transport class in node metadata.** `ServiceTransportClass` in lowered nodes. | S | Done | `from_node_context` reads metadata, not substrings. |
| 16 | C17 | **Kill `propagate_to_param_sources`.** Fix boundary detection. | M | Done | `propagate_to_param_sources` deleted. One port per input. |
| 17 | C18 | **Executor dead code.** Delete `looks_effectful_without_kind()`. | S | Done | Dead code deleted (confirmed deleted in 33513ac9). `cargo clippy` clean. |
| 18 | C19 | **Restore passthrough enforcement + diagnostics (RT4b).** | S | Done | Missing passthrough ports are diagnosable. CI clean. |
| 19 | C21 | **CLI generator: KEY=VALUE and multi-value flags.** | M | Done | `gunbc-infra --input project_id=foo` parses to map. |
| 20 | C23 | **Hermetic AOT binaries (kill `CARGO_MANIFEST_DIR`).** | L | Done | Zero `env!("CARGO_MANIFEST_DIR")` in non-test code. Binaries run outside source tree. |
| 21 | C26 | **Incremental compilation.** Hash-based skip for unchanged `.dag` files. | — | Done | `check_freshness_mtime()` + manifest-based caching in `core/infra/src/freshness.rs`. |
| 22 | C27 | **Typed Config Blocks.** Replace stringified `extra: Vec<(String, String)>` with typed AST parsing. | S | Done | `ProviderConfigField { name, ty: TypeExpr, default }` — fully typed. |

---

## Lane 1 Part B: Binary & Workflow Elimination (Completed Items)

| # | ID | What | Size | Status | Verified |
|---|-----|------|------|--------|----------|
| 1 | A7 | **Workflow catalog → DSL data.** `dsl/config/workflow_catalog.dag`. | M | Done | `catalog.rs` data section deleted. |
| 2 | A8 | **Unit commands → DSL data.** `dsl/config/workflow_commands.dag`. | M | Done | `unit_commands.rs` deleted. |
| 3 | A9 | **Extract generic workflow to `core/workflow/`.** 9 modules. | L | Done | New `core/workflow/` crate. All tests pass. |
| 4 | A10 | **Delete binary infrastructure.** Remove `BinaryArgs` from `gunbc-cli`. | S | Done | `BinaryArgs` deleted. |
| 5 | A5 | **Eliminate `infra.rs`.** Full generated binary (159 LOC), 8 subcommands. | L | Done | All 8 subcommands work via generated binary. |
| 6 | A11 | **Delete compensating tests.** 7 `workflow_*.rs` + `infra_cli.rs`. | S | Done | Files deleted. CT-6 restored tests. |
| 7 | A1 | **Eliminate `sdlc.rs`.** Binary deleted. Full DSL pipeline (45+ .dag files). | S | Done | No sdlc.rs source. Pipeline in `dsl/pipelines/sdlc.dag` (479 lines). |

---

## Lane 2: Service Contracts & Transport (All Complete)

### Part A: Transport Domain Modeling (TL Phase 2)

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | TL-7 | Mock response synthesis. `core/test/src/mock_synthesis.rs`. | L | Done |
| 2 | TL-11 | DSL syntax for transport blocks. `rate_limit {}`, `retry {}`, `error_shape {}`, `credential {}`. | L | Done |
| 3 | TL-12 | Lower transport blocks to IR. Rate limit budgets → `RateLimitConfig`. | M | Done |
| 4 | TL-13 | Domain data migration. Hardcoded rate limits → `dsl/services/*.dag`. | M | Done |
| 5 | TL-14 | Multi-target emit. Rust links to Target SDK. Go/Python stubs. | XL | Done |
| 6 | TL-15 | Substrate cleanup. `lib/transport/` becomes pure Target SDK. | L | Done |
| 7 | TL-16 | Dynamic JSON-path error shapes. `ResponseProvider` enum deleted. | L | Done |

### Part B: Service Layer Completion

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | SL-1 | Wire extdeps → github services. | M | Done |
| 2 | SL-2 | Wire extdeps → llm services. | S | Done |
| 3 | SL-3 | Wire extdeps → gcp services. | M | Done |
| 4 | SL-4 | Wire extdeps → shell/git/cargo services. | S | Done |
| 5 | SL-5 | Enrich ED files with behavioral properties. | L | Done |
| 6 | SL-6 | `response` block parsing (PC-1). | M | Done |
| 7 | SL-7 | `response` blocks on all REST services (PC-3). 29 operations. | L | Done |
| 8 | SL-8 | `exit` blocks on all shell services (PC-4). | M | Done |
| 9 | SL-9 | Lowerer: response → classify_response node (PC-5). | M | Done |
| 10 | SL-10 | GenericRestParseOp status checking (PC-6). | M | Done |
| 11 | SL-11 | Completeness enforcement (PC-10). | S | Done |

### Part C: Contract Testing & Compliance

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | CT-1 | Contract IR. `ContractObligation` structs. | L | Done |
| 2 | CT-2 | Contract test generation. Parameterized test suite per interface. | L | Done |
| 3 | CT-3 | Provider compliance wiring. Per (profile, interface, provider) triple. | M | Done |
| 4 | CT-4 | Annotation cleanup (Category 3). ~30 metadata noise annotations deleted. | S | Done |
| 5 | CT-5 | `ProviderResponseContract` obligation (PC-7:9). | L | Done |
| 6 | CT-6 | Restore deleted tests. 46 workflow contract tests across 8 files. | M | Done |
| 7 | CT-7 | DSL-derive CI secrets (RF-INV2). | M | Done |

---

## Lane 3: SDLC Production (All Complete)

### Part A: Cloud Infrastructure (SP-1:7)

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | SP-1 | GCP credential chaining. WIF OIDC → STS → impersonation → scoped access. | L | Done |
| 2 | SP-2 | Cloud Run deployment DAG. | L | Done |
| 3 | SP-3 | Agent provider: real LLM. | M | Done |
| 4 | SP-4 | Credential provider: local keychain. | M | Done |
| 5 | SP-5 | Multi-worker CAS stress test. | M | Done |
| 6 | SP-6 | CI integration. Hermetic pipeline + cloud smoke test. | M | Done |
| 7 | SP-7 | Webhook-driven stage transitions. | L | Done |

### Part B: SDLC Runs in Cloud (SC-1:8)

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | SC-1 | Wire GCS claim store transport. | M | Done |
| 2 | SC-2 | Wire GCS outcome ledger transport. | S | Done |
| 3 | SC-3 | Wire GCS artifact store transport. | M | Done |
| 4 | SC-4 | Wire GitHub issue provider transport. | M | Done |
| 5 | SC-5 | Wire Pub/Sub signal store transport. | M | Done |
| 6 | SC-6 | Wire credential providers. | M | Done |
| 7 | SC-7 | auth_input compile-time validation. | S | Done |
| 8 | SC-8 | Cloud profile e2e smoke test. | M | Done |

### Part C: SDLC Runs Reliably (SR-1:8)

| # | ID | What | Size | Status |
|---|-----|------|------|--------|
| 1 | SR-1 | Approval gate wiring. Exit code 42 → claim release + yield. | M | Done |
| 2 | SR-2 | Retry budget persistence. | M | Done |
| 3 | SR-3 | Structured execution logging. JSON + trace_id. | M | Done |
| 4 | SR-4 | Health check endpoint. `/health` + graceful SIGTERM. | S | Done |
| 5 | SR-5 | Multi-worker scale test. 5 workers, 20 issues, 10 min. | M | Done |
| 6 | SR-6 | Rolling deploy with zero downtime. | M | Done |
| 7 | SR-7 | Anti-entropy reconciliation. Stale claim auto-release. | L | Done |
| 8 | SR-8 | config.extra → typed provider config. Zero `config.extra` entries. | M | Done |

---

## Historical Completions (Summary)

| Group | Items | Status |
|-------|-------|--------|
| SDLC Activation (Blue Lane 1) | BT1:BT-E1 (16 tasks) | Done |
| External Dependency Modeling (Blue Lane 2) | ED-1:21 (21 tasks) | Done |
| Registry & Extern Deletion (Red Worker B) | B1:B10 (10 tasks) | Done |
| Domain Model Foundation (Phase 2 Lane 4) | DM-1:22 (22 tasks) | Done |
| Transport Target SDK (Phase 2 Lane 5 Phase 1) | TL-0:10 (10 tasks) | Done |
| Red Team (C20, RT1:28, RT-A1:A5, RT-I3:I6) | 39 tasks | Done |
| Foundation Close-Out (NF, FC, BB, RF, CG) | 28 tasks | Done |

---

## Postmortems

### `make gist` 401 — Compounding Failures

**Symptom**: `make gist` returns 401 Unauthorized. `BearerToken (key: "", source: static)`.

Five compounding failures: (1) No credential wiring from operation inputs to execute node (ROOT CAUSE), (2) GenericRestPrepareOp doesn't propagate auth_token, (3) Execute node falls through silently, (4) Diagnostic source is misleading, (5) GenericShellParseOp was emitting Value::Str for Secret outputs (**Fixed**).

**Fix options**: Option A (auto-bridge in lowerer), Option B (explicit `auth_input` in DSL config), Option C (RestPrepareOp sets auth on request). Assigned as **C10a**.

### `gunbc-ci` false failure — `overall_success: Skipped`

**Symptom**: `gunbc-ci` reports "required success check returned false" even when all stages succeed. Root cause: lowerer drops complex return expressions (`BinOp`, `If`, `Match`, `Pipe` → `_ => None`). Fix plan: **C10** (RT4a — complex return expr lowering), **C19** (RT4b — passthrough diagnostics), **C10** (RT4c — lowering completeness gate).

### Testgen Discovery Bug (BT-R1)

**Root cause**: `discover_compilable_modules()` used `func_count == 0` as pre-filter, silently dropping modules with `pipeline`, `pattern`, or `fn` items. **Fixed**: renamed to `callable_count`, broadened filter.

---

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
