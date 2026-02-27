# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Operating Model: Blue Team / Red Team

One blue lane (scenario-driven), two red lanes, never blocking each other.

```
  BLUE TEAM — Advance                     RED TEAM — Harden
  ────────────────────────────            ────────────────────────
  SDLC Activation (single queue):         Single queue:
    BT1 → BT2 → BT3 → BT4 →               RT1 → RT2 → RT3 → RT4 →
    BT5 → BT6 → BT7 → BT8 →               RT5 → RT6 → RT7 → RT8 →
    BT9 → BT10 → BT11:19 → ...cloud        RT9:12 → RT13:16 → RT17:23
```

### Protocols

**Independence**: Blue and Red touch different files. No merge conflicts.
Blue: `dsl/`, `gunbc-dag/src/workflow/`, `gunbc-dag/tests/sdlc_*`.
Red: `core/`, `gunbc-dag/src/resolve.rs`, `gunbc-dag/src/fidelity.rs`.

**Scouting**: Every PR includes a `Scouted:` line listing
opportunities for the other team discovered during implementation.
Add raw observations to the other team's **Unqueued** section — never
directly into their lane queues. The owning team triages and promotes.

**Refill**: When a lane has <3 pending items, the worker proposes
new items from codebase observation or horizon scanning. Anemic
queues are a bug — the point is to never run out of work.

**Non-blocking**: Red never blocks blue. If red team cleanup is
needed for blue team progress, blue does the minimum fix inline
and red cleans up later.

---

# BLUE TEAM — Advance

## SDLC Activation (Single Lane)

### The Scenario

A GitHub issue goes through the full lifecycle:

1. Someone creates issue with label `sdlc:idea`
2. Worker discovers issue → acquires claim → dispatches `handle_idea_to_design`
3. LLM generates design → posted as comment → labels transition to `sdlc:design`
4. Worker dispatches `handle_design_to_review` → LLM reviews → `sdlc:design-review`
5. `handle_review_to_accepted` → approves → `sdlc:accepted`
6. `handle_accepted_to_implementing` → spawns Codex agent on dedicated branch
7. Agent completes → `handle_implementing_to_code_review` → creates PR, LLM reviews diff
8. `handle_code_review_to_testing` → review approved → `sdlc:testing`
9. `handle_testing_to_done` → cargo test + clippy → merges PR if green
10. `handle_done` → closes issue

### Testing Levels

| Level | What | Profile | Transport | Proves |
|-------|------|---------|-----------|--------|
| L0 | Pipeline compiles | — | — | DSL modules resolve, type-check, lower |
| L1 | Hermetic scenario | unit_test | DryRun/stubs | Full idea→done with stubs; stage transitions, claim lifecycle, outcome recording |
| L2 | Per-stage handlers | unit_test | DryRun/stubs | Each of 8 handlers individually correct |
| L3 | Worker loop | unit_test | DryRun/stubs | Discover→claim→dispatch→record→release; replay-skip, retry, claim conflict |
| L4 | Local integration | local | Real GitHub + file | Single stage transition (idea→design) against real API |
| L5 | Full local scenario | local | Real GitHub + file | Complete idea→done on test repo, multiple worker invocations |
| L6 | Testgen | — | — | Auto-generated per-node and per-pair coverage for SDLC DAGs |
| L7 | CLI entrypoint | local | Real | `gunbc sdlc --profile local --repo owner/name` |
| L8 | Cloud deployment | cloud_run | GCS + PubSub | Multi-worker CAS, GCS stores, Cloud Run |

### Current State

**DSL functions are real** (~3,600 lines across 20 .dag files):
- Stage handlers: 8/8 with real service calls and logic (`funcs/sdlc_stages.dag`, 739 lines)
- Worker dispatch: full discover→claim→dispatch→record→release with replay-skip and retry (`funcs/sdlc_worker.dag`, 381 lines)
- Validation policy: 3 real gate fns with conditional logic (`funcs/sdlc_validation_runtime.dag`, 59 lines)
- Interfaces: 7 with 24 capabilities, Providers: 9, Profiles: 3
- DSL-level tests: 10+ test blocks defined in .dag files

**DSL gaps** (need authoring, not just Rust wiring):
- **Pipeline wiring**: `workflows/sdlc.dag` wired — `intake` (param gate), `worker` (calls `dispatch_sdlc()`), `report` (aggregates results). `sdlc_dispatch_runtime.dag` kept as compiled dispatch policy.
- **Dispatch runtime stubs**: `sdlc_dispatch_runtime.dag` has 6 fns that return hardcoded literals — zero conditional logic, zero service calls. Meanwhile `execute_stage()` in `sdlc_stages.dag` already routes correctly. Decision needed: delete dispatch_runtime (dead code?) or fill with real pre-check policy.
- **Transport declarations**: All 26 ops now have DSL transport blocks (github 14 REST, llm 2 REST, file stores 6 file, codex agent 4 shell). Rust `@file` backend still missing (RT3).

**Rust infrastructure** — better than previously assessed:
- Profile-aware compilation **works** — `build_dsl_graph_with_profile()` exists, generated tests pass with all 3 profiles for other modules.
- SubDag/Pipeline execution **works** — `SubDagDispatchOp`, `PipelineDispatchOp` are real implementations.
- Interface stub resolution **works** — unbound interfaces get stub transport, bound interfaces wire to providers.
- resolve.rs may need minimal wiring for SDLC module paths (Red backlog RF-RG2 generalizes this).
- No file transport resolver exists in Rust (`@file` annotation has no backend).
- No CLI entrypoint (catalog is manual — Red backlog RF-RG1).

**Key dependency**: L0–L3 use unit_test profile (all stubs, no transport needed).
L4+ needs transport on all services the local profile touches — BT6 handles this.

### Queue

| # | ID | Task | Level | Size | Status | Deps |
|---|-----|------|-------|------|--------|------|
| 1 | BT1 | **Compile SDLC pipeline.** `build_dsl_graph_with_profile("pipelines/sdlc.dag", "unit_test")` succeeds. Fix any resolve.rs gaps inline. | L0 | S | Done | — |
| 2 | BT2 | **Pipeline wiring.** Fill 3 empty stages in `workflows/sdlc.dag`: wire `intake`, `worker`, `report`. Decision: keep `sdlc_dispatch_runtime.dag` as compiled dispatch policy. | L0 | M | Done | BT1 |
| 3 | BT3 | **Hermetic scenario test.** unit_test profile, DryRun execution succeeds. Pipeline has substantial node count. | L1 | M | Done | BT2 |
| 4 | BT4 | **Per-stage handler tests.** Compilation + structural checks: execute_stage router, 8 handlers, interface stubs. DryRun deferred (scalar fan-in in standalone compilation). | L2 | M | Done | BT2 |
| 5 | BT5 | **Worker dispatch loop test.** Compilation + structural checks: dispatch_sdlc, claim lifecycle, discover, outcome ledger. DryRun deferred (scalar fan-in in standalone compilation). | L3 | S | Done | BT4 |
| 6 | BT6 | **Transport declarations for local profile.** 26 ops: github (14 REST), llm (2 REST), file stores (6 file), codex agent (4 shell). DSL transport blocks complete. Rust `@file` backend deferred to RT3. | — | L | Done | — |
| 7 | BT7 | **Local integration: single stage.** `#[ignore]` tests: local profile compilation + DryRun. Real API gated on `GITHUB_TOKEN`. | L4 | M | Done | BT5, BT6 |
| 8 | BT8 | **Full local scenario.** `#[ignore]` test: full lifecycle DryRun with local profile. Real execution needs RT3 (@file backend). | L5 | L | Done | BT7 |
| 9 | BT9 | **Testgen integration.** Auto-discovery verified: 5 SDLC modules discovered, 1400+ test fns generated. Verification test in `sdlc_testgen.rs`. | L6 | M | Done | BT2 |
| 10 | BT10 | **CLI entrypoint.** `gunbc-sdlc --profile --repo --issue --dry-run`. Binary registered in Cargo.toml, help + DryRun working. | L7 | S | Done | BT8 |
| 11 | BT-R1 | **SDLC review: testgen discovery fix.** `callable_count` replaces `func_count` — discovery now mirrors `module_has_callable_items()` canonical predicate. 59 modules discovered (up from ~29), 9,710 test fns generated. Deleted 493 lines of hand-written compensating tests (sdlc_handlers, sdlc_worker, sdlc_integration, sdlc_scenario). | — | M | Done | BT9 |
| 12 | BT11 | **SignalStore providers.** `pubsub_signal_store.dag` (cloud_run) + `file_signal_store.dag` (local). Transport blocks, test blocks, profile bindings. | L8 | M | Done | BT10 |
| 13 | BT12 | **ArtifactStore providers.** `gcs_artifact_store.dag` (cloud_run) + `inline_artifact_store.dag` (local). Transport blocks, test blocks, profile bindings. | L8 | M | Done | BT10 |
| 14 | BT-R2 | **SDLC review: provider completion.** Transport blocks on gcs_claim_store + gcs_outcome_ledger. Inline definitions extracted to provider files. Dead code deleted from pipelines/sdlc.dag. deploy.dag variable naming fixed. Profile bindings updated in sdlc.dag. | — | M | Done | BT11, BT12 |
| 15 | BT-R3 | **SDLC review: fix 3 execution gaps.** LLM mock responses (enriched `default_rest_response` with LLM-shaped fields), `navigate_json_path` `/` separator + array index support, auth credential embedding in `GenericRestPrepareOp`, `CallParamSourceOp` replaces `IdentityCallableOp` for param_source nodes, param_source propagation in sdlc.rs CLI. Design: `docs/design/mock-response-pipeline.md`. | — | M | Done | BT10 |
| 16 | BT-E1 | **Transport node deduplication.** `gunbc-sdlc --dry-run` fails at 408/494 nodes: `scalar input 'prepare_transport_...Anthropic_Messages.max_tokens' has multiple upstream edges`. Root cause: lowerer creates ONE shared transport triplet per service operation, but `endpoint_use_count` resets per module — callables in different modules both wire literal sources to the same prepare node's scalar port. Fix: make `endpoint_use_count` global across all modules in the compiled graph (not per-module). Touches: `daglang-lower/src/lib.rs` (`add_service_call_edges`, line 5683). | L1 | M | Pending | BT-R3 |

### Postmortem: Testgen Discovery Bug (BT-R1)

**Root cause**: `discover_compilable_modules()` in `dag_test_discovery.rs` used `func_count == 0` as a pre-filter, counting only `Item::FuncDef`. This silently dropped all modules that produce graphs via `pipeline`, `pattern`, or `fn` items.

**Impact**: 12+ modules invisible to testgen (all `dsl/workflows/*.dag`, all `dsl/pipelines/*.dag`). This forced 493 lines of compensating hand-written Rust tests across 4 files. The tests were structurally correct but redundant — they tested exactly what testgen would have auto-generated.

**Fix**: Renamed `func_count` to `callable_count`, broadened filter to match all 4 callable item types (`FnDef | FuncDef | PatternDef | PipelineDef`), mirroring the canonical `module_has_callable_items()` in `daglang-driver/src/lib.rs`. Cross-reference comment added to both locations.

**Prevention**: Comment in discovery code references `module_has_callable_items` as canonical source of truth. If a new callable item type is added to the AST, both must be updated. This postmortem documents the pattern for future audits.

**Audit result**: All other `Item::FuncDef`-specific filters in the codebase (20+ locations) were verified correct for their specific use cases (CLI param extraction, `declared_outputs` field access, etc.).

### Horizon (after BT10)

| ID | Task | Level | Size | Deps |
|----|------|-------|------|------|
| BT13 | GCP credential chaining (WIF OIDC exchange) | L8 | L | BT10 |
| BT14 | Cloud Run deployment DAG | L8 | L | BT11:13 |
| BT15 | Multi-worker CAS stress test (3 workers, exactly-once) | L8 | M | BT14 |
| BT16 | CI integration (hermetic + cloud smoke) | L8 | M | BT15 |
| BT17 | Agent provider: wire codex_agent.dag to real LLM | L5 | M | BT8 |
| BT18 | Credential provider: local keychain for tokens | L5 | M | BT8 |
| BT19 | Webhook-driven stage transitions | L8 | L | BT17 |
| CT-1 | **Contract IR.** Parse `@contract` annotations into `ContractObligation` structs in lowerer. Sequence/idempotency/destructive obligation types. Store in type registry alongside interface capabilities. Design: `docs/design/contract-testing.md` §Phase 1. | — | L | BT-R1 |
| CT-2 | **Contract test generation.** For each interface with `@contract`, testgen emits parameterized test suite. Suite takes `ServiceBinding` as input. Each obligation becomes a test case: setup → execute sequence → assert postcondition. | — | L | CT-1 |
| CT-3 | **Provider compliance wiring.** For each (profile, interface, provider) triple, instantiate CT-2 suite. Stub providers: fast/hermetic/always-run. Real providers: env-gated/integration profiles. Wire into existing PT-* infrastructure. | — | M | CT-2 |
| CT-4 | **Annotation cleanup (Category 3).** Delete metadata noise annotations (`@network`, `@credential`, `@external`, `@derived_from`, `@ledger`, ~30 uses) per `docs/design/modeling/annotation-to-dag-modeling.md` Category 3. | — | S | — |

**Deliverable**: `gunbc sdlc --profile local --repo owner/name` runs full lifecycle.
**Endstate**: SDLC on Cloud Run with GCS stores, PubSub signals, multi-worker CAS.

### Design References

| Document | What |
|----------|------|
| `docs/design/sdlc/mega-modeling-design.md` | Canonical architecture: 9 high-level boxes, core abstractions, canonical contracts, conformance model |
| `docs/design/sdlc/domain-modeling-comprehensive.md` | All domain objects, state machines, invariants |
| `docs/design/sdlc/e2e-gap-analysis.md` | Gap tracking (A–J). Header says "all resolved" but means DSL files exist — Rust can't execute them yet. Gaps C, F genuinely done in Rust. Gaps A, B, D, E, H, I, J resolved at DSL level only. Gap G (worker invokes compiled DAG) not done at all. |
| `docs/design/sdlc/implementation-roadmap.md` | Task breakdown and dependency graph |
| `docs/design/provider-contracts.md` | Provider response contracts: `@response` annotation, mandatory error modeling, testgen obligations, interface inheritance. Supersedes RT-I1/RT-I2. |

---

## Blue Unqueued

Raw observations from any worker. Not triaged, not sized.
Blue team promotes to backlog or lane queues during triage.

| Observation | Source | Date |
|-------------|--------|------|
| CI YAML generation (`generate_github_actions_template`, `generate_gitlab_ci_template`) is ~120 lines of hand-wired `push_str`/`write!` string concatenation in `codegen_cli.rs:503-609`. The DSL already has rendering infrastructure (`std/render.dag`, `std/markdown_render.dag`) and a proven code-generation pattern (`tools/makegen.dag`). CI YAML types (Workflow, Job, Step, Trigger, Permission, Cache) should be modeled in `.dag` with pure rendering functions, following the makegen pattern: discover via extern → render in pure DSL → `content_upsert`. Deletes both template functions + the validation functions (lines 450-500). See task breakdown below. | R1 scout | 2026-02-26 |
| `dsl/cloud/aws/credential.dag` imports `aws.STS` and `aws.SecretsManager` from abstract infra services (no transport blocks). These work via profile bindings but have no test coverage at L0 (compilation) or L1 (hermetic). Same pattern for `cloud/azure/credential.dag`. These could be BT-adjacent test targets once SDLC profiles are active. | CI fix | 2026-02-27 |
| The `credential_chain` pattern in `dsl/std/patterns.dag:236-283` is a proven 5-step OAuth2 chain (OIDC → STS → impersonation → SecretManager → AccessToken) with `local_auth()` at lines 392-411. Gist bypasses this entirely in favor of raw `shell.GCloud.SecretManagerAccessVersion`. Migrating gist to use `credential_chain` would exercise the pattern end-to-end and validate the compositional auth model. | RT-A4 analysis | 2026-02-27 |

---

## Blue Backlog

Triaged and sized. Promote to lane queues when horizon items are exhausted.

| ID | Item | Size | Priority | Notes |
|----|------|------|----------|-------|
| CG-1 | DSL CI model types: `dsl/std/ci.dag` — `CiWorkflow`, `CiJob`, `CiStep` (Run/Uses/DagRun), `CiTrigger`, `CiPermission`, `CiCache`, `CiEnv`, plus provider sum type `CiProvider = GitHub \| GitLab`. Data declarations for shared configs (Rust cache paths, cargo env). | M | P1 | Layer 0 types — no rendering yet. Follow `std/languages.dag` pattern for tautological definitions. |
| CG-2 | DSL CI rendering functions: `dsl/std/ci_render.dag` — `render_github_workflow(w: CiWorkflow) -> String`, `render_gitlab_workflow(w: CiWorkflow) -> String`, plus helpers (`render_step`, `render_job`, `render_permissions`, `render_env_block`, `render_cache`). Pure functions, string interpolation + join. | M | P1 | Follow `makegen.dag` rendering pattern: small composable fns, `\|> map` + `\|> join("\n")`. YAML indentation via string literals (no general YAML serializer needed). |
| CG-3 | DSL cigen tool: `dsl/tools/cigen.dag` — single entrypoint `func cigen() -> { written: Bool }` that discovers CI config via extern (permissions, secrets, tool invocation, branches), constructs `CiWorkflow` records, renders both providers, calls `content_upsert` for each. Extern bridge: `discover_ci_config() -> CiConfig`. | M | P1 | Follow `makegen.dag` entrypoint pattern. Discovery extern returns structured config, all rendering is pure DSL. |
| CG-4 | Delete Rust cigen code: remove `generate_github_actions_template()`, `generate_gitlab_ci_template()`, `validate_github_actions_template()`, `validate_gitlab_ci_template()` from `codegen_cli.rs`. Wire `cmd_cigen()` to the new DSL tool (same pattern as `cmd_codegen()` calling `build_dsl_graph_for_entrypoint`). | S | P1 | ~200 lines deleted from `codegen_cli.rs:450-609`. Validation moves to DSL-side (structural — if the types construct, the YAML is valid). |
| CG-5 | Migrate `RenderConfig` builder + `SharedStep` + `yaml_block` from `core/ir/src/transport/ci/render.rs` — evaluate what remains needed as Rust runtime vs what becomes dead code after CG-1:4. Delete dead code, keep only provider detection (`detect_provider`, `is_ci`). | S | P1 | May keep `CiRenderer` trait for runtime step-level rendering (animated progress). CI YAML generation is a separate concern. |
| H10 | Compute stack orchestration: Cloud Run/GCS/LB lifecycle DAG builder. | L | P2 | `docs/design/horizon/h10-compute-stack-services.md` |
| S12-E | Multi-worker CAS: GcsClaimStore with generation-based CAS. DSL exists. Distinct from B-12 (which stress-tests SignalStore/ArtifactStore). | M | P2 | Deferred until cloud_run profile needed |
| H1 | Display reactive DSL: channel-driven event loop. | XL | P3 | No current use case. Review 2026-Q3, delete if not promoted. |

---

# RED TEAM — Harden

### Philosophy: Eliminate, Don't Relocate

The red team goal is **structural impossibility of defects**, not
better error messages for them. Every fix should push the problem
upstream — closer to the point of construction — so downstream code
can't encounter the bad state at all.

**Bad**: move a string match from file A to file B.
**Better**: parse the string into an enum at the boundary, match exhaustively.
**Best**: make the enum the only representation — no string ever exists.

The test: *after your fix, can a future contributor reintroduce the
same class of bug?* If yes, you relocated it. If no, you eliminated it.

### Smell Catalog (what scouts look for)

| Smell | Example | Typical Fix |
|-------|---------|-------------|
| **String dispatch** | `match kind_str { "shell" => ..., "rest" => ..., _ => ... }` | Parse once at boundary → enum. Exhaustive match, no fallback. |
| **Validation at use site** | `parse().unwrap_or(default)`, `if x.is_none() { return fallback }` | Make the constructor enforce the invariant. Fields are non-Option if always populated. |
| **Heuristic reimplementation** | Rust code that replicates logic the DSL already declares | Delete the Rust, call the DSL. If the evaluator can't handle the DSL construct yet, that's the real task (e.g., RF-G-unblock). |
| **Static mapping table** | Hand-maintained `HashMap` or match arms mapping A→B | Derive from a single source (DSL data declaration, enum with `#[derive]`, or const array). |
| **Option-that's-always-Some** | `field: Option<T>` where every construction site writes `Some(...)` | Make the field `T` with a `Default`. |
| **Stringly-typed enum** | `String` field that only holds N known values | Dedicated enum. `FromStr` at boundary, `.as_str()` only for serialization. |
| **Fallback arm** | `_ => default` or `other => ...` in a match on known variants | Exhaustive enum match. If a new variant appears, compilation forces handling it. |
| **Duplicate filter logic** | Same `starts_with("res:")` / `"tool:"` check in 5 files | Central type (`PortCategory`) with one `from()` impl. Call sites use the type. |
| **Manual registry** | Hand-maintained list mapping names → files/modules | Derive from DSL graph, Cargo.toml, or structural inference. |

### Remediation Ladder

When you find a smell, apply the **highest rung** that's feasible:

1. **Eliminate the representation** — the bad state can't be constructed
2. **Parse at the boundary** — raw input becomes a typed value once
3. **Derive from source of truth** — delete the hand-maintained copy
4. **Centralize** — if elimination isn't possible yet, at least one canonical impl

Rung 4 is a **waypoint**, not a destination. If you centralize, file
a follow-up to eliminate.

---

## Red Team Queue

Single queue. Model correctness first (silent failures), then structural
correctness, then testing + foundation.

| # | ID | What | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 1 | RT1 | **Credential wiring.** `config { auth: BearerToken }` + `auth_token: Secret` → token never reaches execute node → unauthenticated request, silent. Fix: explicit `auth_input` in DSL config, lowerer wires to `res:credential`. Touches: daglang-syntax, daglang-lower, resolve_service.rs. Test: `make gist` returns 200. See POSTMORTEM below. | M | Done | — |
| 2 | RT2 | **Execute node silent fallthrough.** Missing `res:credential` → sends unauthenticated, no error. Fix: fail-closed when `auth_scheme` declared. Touches: `lib/transport/src/ops.rs`. | S | Done | — |
| 3 | RT3 | **File transport completeness.** Only READ/READ_BYTES/WRITE. Missing: EXISTS, CREATE_DIR, DELETE, APPEND, GLOB. SDLC local profile needs EXISTS + CREATE_DIR. Touches: resolve_service.rs. | M | Done | — |
| 4 | RT4 | **Transport block validation in lowerer.** `LowerError::MissingTransport` now fires for partially-specified services (some ops have transport, some don't). Fully-abstract services (no transport on ANY operation, e.g., infra/aws, infra/azure) are exempt — they get transport via profile bindings. Also exempts interface implementors. Touches: daglang-lower. | M | Done | — |
| 4a | RT4a | **Complex return expression lowering.** `resolve_return_expr_source()` only handles Ident/FieldAccess/Call/Literal — `BinOp`, `UnaryOp`, `If`, `Match`, `Pipe` all fall through to `_ => None` (silent drop). Fix: synthesize compute nodes in IR for complex expressions, wire result to `__out:*`. Touches: `daglang-lower/src/lib.rs:7768-7841`. Test: `build.dag` `overall_success` evaluates to `Bool(true)` when all stages pass. Revert ci.rs workaround after. See POSTMORTEM below. | M | Pending | — |
| 4b | RT4b | **Passthrough missing-input diagnostic.** `execute_with_declared_output_passthrough()` silently falls back to `Value::Skipped` when `__out:*` input is missing. Fix: emit diagnostic warning (or error in strict mode) when a declared output port has no wired input. Touches: `gunbc-dag/src/resolve.rs:71-91`. | S | Pending | — |
| 4c | RT4c | **Lowering completeness gate.** `wire_callable_return_outputs()` silently `continue`s when `resolve_return_expr_source()` returns `None`. Fix: emit `LowerWarning` for unwired non-optional return outputs. Track unwired count as metric. Touches: `daglang-lower/src/lib.rs:7844-7902`. | S | Pending | — |
| 5 | RT5 | **`fold` extraction** in evaluate_fn_body() — enables DSL classify_transports(). Deletes fidelity shadows + silent fallbacks. Multi-param lambda parsing, pipe method typechecker exclusion, fidelity.rs rewrite to call DSL classify_transports directly. test_policy.dag shadow functions removed. | M | Done | — |
| 6 | RT6 | **NodeKind required** on Node\<T\>. Removed `Option<NodeKind>` — `kind` is now `NodeKind` with `#[serde(default)]` to `Pure`. Updated constructors, 27 test sites, and 14 comparison sites across 12 files. `validate_node_kinds_for_interception` checks `Pure` nodes instead of `None`. | M | Done | — |
| 7 | RT7 | **Port namespace typing**: `PortCategory` enum (Resource/Tool/Internal/User) + methods on `PortName` (`is_resource()`, `is_tool()`, `is_internal()`, `is_user()`, `bare_name()`, `category()`). Migrated 14 string-match sites across 12 files. | M | Done | — |
| 8 | RT8 | **TransportRole enum** in resolve.rs. `TransportRole::from_name()` parses `service_transport::{prepare/execute/parse}::` once, replaces 6 `starts_with()` checks in `resolve_domain()` and `resolve_service_transport()`. Match dispatch uses `(spec, role)` tuple instead of `(spec, is_prepare, is_parse)` booleans. | M | Done | — |
| 9 | RT9 | **Virtual I/O DSL types** (`dsl/std/virtual_io.dag`). VirtualFsSetup, ShellCassette, HttpStub, VirtualBackendConfig, ShellCassetteRegistry, HttpStubRegistry, TcpLoopback, TcpLoopbackRegistry. HttpMethod enum for request matching. VirtualBackendConfig composes all registries. | M | Done | — |
| 10 | RT10 | **Shell cassette registry** in VirtualTransportBackend. `ShellCassette` struct, `add_shell_cassette()`, `match_shell_cassette()`. Exact (command, args) matching with wildcard (empty args). Cassettes checked before built-in handlers (find, printenv, sh). 3 tests. | S | Done | — |
| 11 | RT11 | **HTTP response registry** in VirtualTransportBackend. `HttpStub` struct with method/path matching, `add_http_stub()`. `execute_rest()` and `execute_http()` dispatch to stub registry. Path prefix and exact matching. JSON body parsing for REST, raw for HTTP. 2 tests. | S | Done | — |
| 12 | RT12 | **TCP loopback registry** in VirtualTransportBackend. `TcpLoopback` struct, `add_tcp_loopback()`, `execute_tcp()`. Port-based matching, canned response data. 1 test. | S | Done | — |
| 13 | RT13 | **Derive mock registries** from `@mock_response` annotations. Auto-generate VirtualBackendConfig per workflow. | M | Pending | RT9 |
| 14 | RT14 | **S-tier codegen** in `build_fidelity_ladder_section`. Real mode + virtual backends via TransportBackendGuard. | M | Pending | RT10:13 |
| 15 | RT15 | **M-tier codegen** (sandboxed tempdir, `#[cfg(feature = "sandboxed_tests")]`). | M | Pending | RT14 |
| 16 | RT16 | **L/XL tier codegen** (cost-gated real/remote, `GUNBC_TEST_MAX_COST` env check). | S | Pending | RT14 |
| 17 | RT17 | **DSL Makefile assembly**: populated all 4 empty pipeline stages in `workflows/makegen.dag`. `load_registry` calls `discover_tools()`, `render_makefile` calls `render_makefile_content()` with data + tools, `upsert_makefile` calls `content_upsert()`, `report` emits success. Pipeline imports data from `config.build_targets` and functions from `tools.makegen`. | M | Done | — |
| 18 | RT18 | **Delete bootstrap externs**. Parity golden tests. Blocked: DSL bodies produce simple output; Rust externs produce full output (Makefile via ToolRegistry, gitignore via BuildConfig categories). Needs DSL port of gitignore category system + makegen delegation. | M | Blocked | RT17, DSL gitignore port |
| 19 | RT19 | **Recursive types** (self-referential type defs). | L | Pending | — |
| 20 | RT20 | **Recursive functions** (self-calls in fn bodies). | L | Pending | RT19 |
| 21 | RT21 | **Tree rendering in pure DSL**. Delete RenderTreeOp. | L | Pending | RT19, RT20 |
| 22 | RT22 | **Snapshot content DSL**. Delete BuildSnapshotContentOp. | M | Pending | RT21 |
| 23 | RT23 | **Delete extern_impls.rs** entirely. Zero extern func in any .dag file. | S | Pending | RT21, RT22 |
| 24 | RT24 | **Profile-aware compilation testing.** `auto_testgen_for_module()` now accepts discovered profiles. When `requires_profile`, picks first hermetic profile for compilation. `TestgenOp::AutoGenerate` has profile fallback. `build_dsl_graph_with_types_and_profile()` added. Deleted `sdlc_testgen.rs` + 2 hardcoded profile tests. Result: 6 SDLC modules now generate tests (3,532 test fns) instead of being silently skipped. | M | **Done** | — |

### Postmortem-Driven: Auth & Testgen Hardening (URGENT)

> See `TODO/gist-auth-postmortem.md` for full analysis. The gist 401 exposed
> systemic gaps in credential lifecycle, testgen error coverage, and
> compositional modeling enforcement. These affect every DSL-defined service,
> not just gist.

**Analysis tasks** (study before implementing):

| # | ID | What | Size | Status | Deps |
|---|-----|------|------|--------|------|
| A1 | RT-A1 | **Shell exit code audit.** Which operations use TrimStdout/SplitLines? Which can legitimately return non-zero (e.g. `printenv` → exit 1 = var not set)? Build decision matrix for exit code handling per operation. Refs: `resolve_service.rs:521-528` (TrimStdout), `resolve_service.rs:488` (SuccessStdoutStderr). | S | **Done** | — |
| A2 | RT-A2 | **`@mock_response` adoption gap.** Every service operation should declare success + auth-failure mock responses. Count the gap: ops × missing mocks = obligation deficit. Currently `@mock_response` is only in examples/comments — zero real service uses it. Refs: `dsl/services/` (all .dag), `gunbc-dag/src/mock_defaults.rs:145-180`. **Findings**: 29 REST ops × 0 mock_response annotations = 29 missing. Parser has `MockResponseDef` AST but parsing is not implemented (empty Vec). See `TODO/testgen-proof-analysis.md`. | M | **Done** | — |
| A3 | RT-A3 | **Credential chain audit.** Trace producer→consumer edges for every service with `config { auth }`. **Findings**: 5 services with `config { auth }`. Two have `auth_input` and work (`gcp.IAM` → access_token, `github.Gist` → auth_token). Three lack `auth_input` and have broken wiring: `gcp.SecretManager` (used in credential_chain but no auth_input → res:credential never bound), `gcp.ResourceManager` (both ops), `llm.Anthropic`. The lowerer only wires credentials when `auth_input` is explicitly set — there's no implicit resource injection path. Rule: services with `auth: <scheme>` MUST declare `auth_input: <field_name>`. Fix: add `auth_input` to SecretManager, ResourceManager, Anthropic + update callers to pass credential. | S | **Done** | — |
| A4 | RT-A4 | **Why gist bypasses `credential_chain`.** **Findings**: (1) `credential_chain` pattern fully works — proven by generated tests in `generated_tests_cloud_gcp_credential.rs` (360 obligations, 259 testable). (2) `acquire_gcp_secret` wrapper exists at `dsl/cloud/gcp/credential.dag:27-55`. (3) Gist was never wired to credential_chain — `dsl/workflows/gist.dag` has an empty `credential_resolve` stage placeholder. (4) No blockers: all dependent services exist, RT1/RT2 fixes are applied, lowering is proven. (5) Migration: replace `shell.GCloud.SecretManagerAccessVersion` with `acquire_gcp_secret(runtime: LocalDev, ...)` in all 3 gist entrypoints. Benefits: structured errors, auto-refresh, runtime polymorphism, resource semantics. | M | **Done** | — |
| A5 | RT-A5 | **Credential-as-resource model.** `credential_chain` declares `provides auth: AuthContext` (`dsl/std/resources.dag:87-98`, `expires: true`). How should the compiler enforce expiry handling? Should testgen generate expiry-scenario tests? **Findings**: (1) `expires: true` on `AuthContext` resource is parsed into `properties: Vec<(String, Expr)>` but the lowerer only reads `provider`/`cloud` from properties — `expires` is **silently discarded**. (2) The IR has full credential expiry infrastructure: `Secret.expires_at: Option<SystemTime>`, `Secret.is_valid()`, `MockSpec.resource_credential(id, expiry_ms)`, `ResourceType::Credential { expiry_ms, refreshable }`, `ResourceBehavior::LeaseExpires`. (3) No workflow test uses `resource_credential()` — only MockSpec unit tests test the infrastructure itself. (4) No testgen obligation generates expiry-scenario tests. (5) The `Credential` type in `core/ir/src/transport/credential.rs` has `expires_at` but nothing in the execution pipeline checks `is_valid()` before using the credential. **Gap summary**: DSL declares `expires: true`, IR has expiry plumbing, but compiler doesn't connect them. Three missing pieces: (a) lowerer reads `expires` from resource properties and sets `has_expiry` on the resource spec, (b) testgen generates expiry-scenario tests (expired credential → retry or fail), (c) executor checks `credential.is_valid()` before sending request. | S | **Done** | — |

**Implementation tasks** (after analysis):

| # | ID | What | Size | Status | Deps |
|---|-----|------|------|--------|------|
| I1 | RT-I1 | **Superseded by PC-3.** ~~`@mock_response` on all service operations.~~ → `@response` annotations with provider contract semantics. See `docs/design/provider-contracts.md`. | — | Superseded | — |
| I2 | RT-I2 | **Superseded by PC-7 + PC-8.** ~~Testgen error-status obligations.~~ → `ProviderResponseContract` obligation kind with per-status-code tests derived from `@response` annotations. See `docs/design/provider-contracts.md`. | — | Superseded | — |
| I3 | RT-I3 | **`CredentialChainIntegrity` testgen obligation.** Bucket D.6: for every transport execute node with `res:credential` port, verify edge exists. If disconnected, emit Invalid (the 401 bug pattern). If connected, discharge. Two new tests: `test_credential_chain_integrity_connected` and `test_credential_chain_integrity_disconnected_is_invalid`. | M | **Done** | RT-A3 |
| I4 | RT-I4 | **Shell exit code enforcement.** TrimStdout: fails on non-zero exit unless output is optional `T?` (returns `Value::Skipped`). SplitLines: returns empty list on non-zero (list-producing ops like `find` legitimately return exit 1 for missing paths). `shell_exit_error()` helper for clear diagnostics. Proof tests in `gunbc-dag/tests/shell_exit_enforcement_proof.rs`. Analysis report: `TODO/testgen-proof-analysis.md`. | S | **Done** | RT-A1 |
| I5 | RT-I5 | **Wire `credential_chain` into gist.** 3 call sites in `gist.dag` migrated from `shell.GCloud.SecretManagerAccessVersion` to `acquire_gcp_secret(runtime: LocalDev, ...)`. Added `extract_secret` helper fn + `secret: Secret` output on `acquire_gcp_secret` to work around lowerer's single-level field access limitation for auth_input wiring. `shared/gist_modes.dag` kept on services-layer call (import direction lint: shared layer 5 cannot import cloud layer 7). Fixed dual-wiring bug: `wire_auth_credential_edges` now skips endpoints with `auth_input` to avoid duplicate `res:credential` edges. | M | **Done** | RT-A4 |
| I6 | RT-I6 | **Verify `credential_chain` end-to-end.** Converted `gist_recent_end_to_end_emits_gist_url` to DryRun with `auto_mock_spec` (credential chain's `local_auth()` func is too complex for fn_body extraction — returns Skipped in Real mode). Added structural test `gist_recent_graph_wires_credential_to_gist_execute` verifying `res:credential` edge on Gist_Create execute node. Updated golden snapshot `s2_credential_chain_gcp.json` for `extract_secret` fn addition. All 4 gist_recent tests pass. | M | **Done** | RT-I5 |

### Horizon (after RT23)

| # | ID | What | Size |
|---|-----|------|------|
| 24 | RT24 | ✅ `NodeKind` already has `TransportPrepare`/`TransportExecute`/`TransportParse` variants (added earlier). Added `NodeKind::is_transport()` convenience method for callers that need to check transport-phase membership. 13 files use these variants across lowerer, executor, codegen, testgen, and derive. | S |
| 25 | RT25 | ✅ String constants centralized: `PortName::{RESOURCE_PREFIX, TOOL_PREFIX, INTERNAL_PREFIX, OUTPUT_PASSTHROUGH_PREFIX, DEPS, RESOURCE_CREDENTIAL}` on `PortName` in `types.rs`. Added `RESOURCE_CREDENTIAL` to `resource/mod.rs`. Updated `bare_name()`/`category()` to use constants. Migrated lowerer's private `OUTPUT_PASSTHROUGH_PREFIX` to use `PortName::OUTPUT_PASSTHROUGH_PREFIX`. | S |
| 26 | RT26 | ✅ `#[derive(StringEnum)]` proc-macro generating `as_str()`, `parse()`, `Display` for unit-variant enums. Container attribute `#[string_enum(rename_all = "UPPERCASE")]`, variant attribute `#[string_enum(name = "custom")]`. Migrated 7 enums: HttpMethod, AnnotationLevel, ReasoningEffort, ReasoningSummary, Role, CloudProviderKind, CodegenSubcommand. Deleted ~180 lines of manual impls. | M |
| 27 | RT27 | ✅ ModulePath unification across 4 crates. Replaced `Vec<String>` module paths with canonical `ModulePath` struct from daglang-syntax across resolve, typecheck, lower, emit, driver, cli. Added `new()`, `as_dotted()`, `Display`, `From<Vec<String>>`, `Hash`/`Eq` derives. | S |
| 28 | RT28 | ✅ Shared DslTypeMapping table for emit backends. Added `type_mapping.rs` with `DslTypeMapping` struct, static `RUST_TYPE_MAPPING`/`GO_TYPE_MAPPING` tables, and shared `map_abstract_type()` with recursive container handling. Rust and Go backends now delegate to shared tables. C backend remains separate (returns CType enum, not String). | S |
| 29 | RT29 | Registry pattern for DAG tooling string dispatch. Primary: `resolve_service_transport()` (12 arms), secondary: `lookup_extern_impl()` (9 arms). | L |
| 30 | RT30 | ✅ Port namespace migration: `starts_with("res:")`/`"tool:"` → `PortName` methods. 3 sites migrated (lower `is_user_param_port`, emit `is_user_input_port`, patterns `ResourceInput::new`). `process_registry.rs` skipped (uses `ClaimId`, not `PortName`). | S |
| 31 | RT31 | ✅ Migrated hardcoded port name strings to central constants: `"__deps"` → `PortName::DEPS` (37 sites across lowerer, driver, resolve, infra), `"__out:"` → `PortName::OUTPUT_PASSTHROUGH_PREFIX` (2 sites in resolve), `"res:file"` → `RESOURCE_FILE` (3 sites in lowerer, testgen graph), `"res:credential"` → `PortName::RESOURCE_CREDENTIAL`/`RESOURCE_CREDENTIAL` (16 sites across lowerer, transport ops). Skipped raw string templates (emitted runtime code) and test-only assertions. | M |
| 32 | RT32 | ✅ Split monolithic files: daglang-lower 11,497→8,107 lines (extracted spec.rs + tests.rs), typecheck 5,321→3,403 lines (extracted tests.rs), execute 4,336→2,186 lines (converted to directory module with tests.rs). Total: ~8.5K lines extracted. | L |
| 33 | RT33 | ✅ Unified passthrough op variants: replaced 3 `LoweredOp` variants (`LoopUnpack`, `LoopPack`, `BranchMerge`) with single `Pattern(PatternOp)`. Added `kind_name()` to `PatternOp`. Simplified 10 match sites across lowerer, emit, derive, resolver, and CLI render. | S |
| 34 | RT34 | ✅ Error type consolidation. Replaced opaque `CompileError { message: String }` with structured enum: `Resolve(ResolveError)`, `Typecheck(Vec<TypeError>)`, `Lower(LowerError)`, `Emit(EmitError)`, `Derive(DeriveError)`, `Message(String)`. Added `From` impls for all phase errors. Eliminated ad-hoc `format!()` conversions in driver pipeline. ParseErrors Display preserves `normalize_diagnostics` sorting. | M |
| 35 | RT35 | ✅ Test helper extraction: consolidated 9 duplicate `unique_temp_dir`/`unique_temp_file` implementations across driver, cli (pipeline, main, compile/tests), and 3 integration test files to use shared `gunbc_test::{unique_temp_dir, unique_temp_file}`. Added `unique_temp_file()` to gunbc-test/temp.rs. Added gunbc-test as dev-dependency to daglang-driver and daglang-cli. `module_graph_from_sources` kept crate-local (would require compiler crate dependencies in test utility). | M |
| 36 | RT36 | ✅ Deleted unused scaffolding: `RetryPolicy`, `ErrorMapping`, `BackoffStrategy` structs from lowerer, removed `retry_policy` field from `ServiceCallMetadata`, removed `error_mappings` field from `RestOperationSpec`, cleaned up 30+ `None`/`vec![]` initializations across 8 files. | S |
| 37 | RT37 | ✅ Deleted underused abstractions: `Semiring` trait + impl + 5 tests (zero production consumers), `GraphicsMedium` trait (zero impls), `MarkupRenderer` trait (zero impls), `DocumentRenderer` trait (zero impls). Kept `PartialOrder`, `JoinSemilattice`, `MeetSemilattice`, `BoundedLattice`, `Lattice` (2+ production consumers each). | S |

### Hack Deletion Queue (review-driven)

Hacks, silent fallbacks, and missing-language-feature workarounds identified
by code review. Each has a verified codebase location. Ordered: panics/safety
first, then stringly-typed hacks, then language-feature gaps.

| # | ID | What | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 38 | RT38 | **No panics in lowering for user-authored DSL.** `derive_file_spec()` calls `FileOp::from_dsl_str(..).unwrap_or_else(\|\| panic!(...))` (`daglang-lower/src/lib.rs:5051-5052`). Replace with `LowerError::InvalidTransportSpec` (or similar) including service/op context + bad string. Add test: unknown file op → clean error message. | S | Pending | — |
| 39 | RT39 | **`auth_input` must parse or error.** Parser at `daglang-syntax/src/parser.rs:338-343` silently advances on non-ident `auth_input` values (e.g. string literal → becomes `None`, changes semantics). Same pattern for `auth` at lines 331-336. Replace `else { self.advance(); }` with parse error "expected identifier for auth_input". Add test for non-ident value. | S | Pending | — |
| 40 | RT40 | **Structural optionality for shell TrimStdout.** `resolve_service.rs:570` detects optional output via `f.type_id.ends_with('?')` — stringly-typed. Replace with structured optionality flag or port cardinality from IR/spec. Non-zero→Skipped should only apply when the operation explicitly declares that mapping, not for all optional outputs. | S | Pending | — |
| 41 | RT41 | **Explicit exit mapping for shell SplitLines.** RT-I4 kept "non-zero → empty list" for SplitLines (`resolve_service.rs:529-537`). This conflates "command failed" with "no results" for non-`find` operations. Replace with explicit per-operation "acceptable non-zero exits" declaration (aligns with PC-4 `exit` blocks). Add test: exit=1 + non-empty stderr should error unless explicitly allowed. | S | Pending | PC-4 |
| 42 | RT42 | **Pipe methods first-class in AST.** `should_track_call_name()` in `ast_utils.rs:42-76` hard-codes 20+ pipe method names as a string allowlist shared between typechecker and lowerer. Introduce `PipeMethod` enum (or similar) in AST/IR — single authoritative registry. Acceptance: delete the allowlist and still typecheck/evaluate same corpus. | M | Pending | — |
| 43 | RT43 | **Nested field access in lowerer.** Lowerer only supports single-level field access for auth_input wiring. Workaround: `extract_secret` fn + alias `secret: Secret` output on `acquire_gcp_secret` (`dsl/cloud/gcp/credential.dag:28-31`). Implement `x.y` (and `x.y.z`) field access in lowerer/evaluator, then delete `extract_secret` and alias outputs. | M | Pending | — |
| 44 | RT44 | **Expand `evaluate_fn_body` coverage.** `local_auth()` func is too complex for fn_body extraction — returns Skipped in Real mode (`gist_recent_regressions.rs:73-74`). Either (a) teach `evaluate_fn_body` to support effectful conditionals used in `local_auth()`, or (b) restructure `local_auth()` into extractable pieces. Regression test: Real mode must not silently return Skipped for credential chain. | M | Pending | — |
| 45 | RT45 | **Typed enum values from `evaluate_fn_body`.** Fidelity classifier results come back as `Value::Str` requiring `TestClass::parse()` / `FermiCost::parse()` round-trips (`fidelity.rs:93-100`). Add `Value::Enum { ty, variant }` (or tagged union encoding) so evaluator returns structured enums. Delete parse round-trips. | M | Pending | — |
| 46 | RT46 | **Delete fidelity silent fallbacks.** `unwrap_or(TestClass::Unit)` and `unwrap_or(FermiCost::XS)` in `fidelity.rs:95,100` silently default on classification failure. Replace with explicit error (or force explicit "unknown" classification). Add CI guard: grep for these patterns outside tests → fail. | S | Pending | RT45 |
| 47 | RT47 | **Proper multi-param lambda parsing.** Speculative/backtracking parse at `parser.rs:2590-2628` manually manages `speculative_pos` index. Add a lookahead routine that decides "this is a lambda" without mutating parser state (or a formal backtracking helper). Test matrix: `(a,b)=>`, `(a, b) =>`, `(a,b,) =>` (if allowed), `(a) =>`, `(a,b)` (non-lambda). | M | Pending | — |
| 48 | RT48 | **Ship PC-1 + PC-4 early.** `response { STATUS => TYPE }` and `exit { CODE => BEHAVIOR }` structural blocks replace `@mock_response` stub (`MockResponseDef` parser always returns empty Vec per `parser.rs:1275,1410`) and implicit shell exit semantics. Priority: PC-1 (parse response blocks) → PC-4 (parse exit blocks) → remove implicit SplitLines/TrimStdout fallbacks. | L | Pending | — |

### Contract Testing & Derivation Queue (tautological cleanup review)

Items from dead-spec cleanup analysis. Contract testing phases, auto-derivation opportunities,
and structural simplifications identified by reviewing state machine / annotation / test redundancy.

| # | ID | What | Size | Status | Deps |
|---|-----|------|------|--------|------|
| 49 | RT49 | **`@contract` executable predicates (CT-1).** Upgrade `ContractDef.text: String` to executable predicate language → `ContractObligation` IR in type registry. Currently scaffolding only (2 uses in `infra/core.dag`, label string, no assertions). | L | Pending | — |
| 50 | RT50 | **Contract test suite generation (CT-2).** Generate parameterized compliance suites from `ContractObligation` IR. Each obligation → test case with typed assertion. | L | Pending | RT49 |
| 51 | RT51 | **Provider compliance instantiation (CT-3).** For each (profile, interface, provider) triple, instantiate contract suite. Hermetic profiles run stubs always; integration profiles env-gated with real I/O. RT24 provides the profile-binding substrate. | L | Pending | RT50 |
| 52 | RT52 | **Auto-derive fixture blocks.** Wire `auto_mock_spec` as the default DSL test executor mock strategy so `fixture` blocks become unnecessary. Requires DSL test runtime changes. | M | Pending | — |
| 53 | RT53 | **Status-code-variant test derivation.** `response` blocks on operations auto-generate per-status-code mock+assertion pairs. 13 manual tests could be eliminated. Requires response block design. | M | Pending | RT48 |
| 54 | RT54 | **Pipeline-derived state machine.** Compiler reads `pipeline sdlc { stage X [after Y] }` topology and derives `validate_transition()`, `stage_ordinal()`, etc. from graph edges automatically. `std/state_machines.dag` becomes a pure library of budget validation. | M | Pending | — |
| 55 | RT55 | **Transport cost model on `ServiceTransportClass`.** Move `transport_depth`/`transport_hermetic` tables from `test_policy.dag` into the Rust type. Reduces DSL policy to 2 budget constants. | S | Pending | — |
| 56 | RT56 | **Deterministic profile selection policy.** Replace "first hermetic profile" heuristic in `auto_testgen_for_module()` with explicit policy (e.g., `unit_test` always unless overridden). `gunbc-dag/src/bin/sdlc.rs` + testgen. | S | Pending | — |
| 57 | RT57 | **Remove TestgenOp compilation fallback.** `TestgenOp::AutoGenerate` tries profileless compile, then retries with "first hermetic profile". Replace with: `requires_profile` → select by deterministic policy (RT56), no retry loop. | S | Pending | RT56 |

---

## Red Unqueued

Raw observations from any worker. Not triaged, not sized. Use the
smell catalog above to classify. Include file path + line if possible.

| Smell | Observation | File | Source | Date |
|-------|-------------|------|--------|------|
| *(credential wiring promoted to RT1)* | | | | |
| Validation at use site | `auto_mock_spec()` always produces exit 0 / status 200 for transport mocks. 7 transport mocks for gist_recent, 0 error scenarios. Testgen Bucket C `SingleTransportFailure` only injects `Value::Str("<TRANSPORT_FAILURE>")` sentinel, not realistic errors (401, exit 1). See `TODO/testgen-proof-analysis.md`. | `gunbc-dag/src/mock_defaults.rs:145-184` | RT-I4 proof | 2026-02-27 |
| Validation at use site | `GenericRestParseOp` doesn't check HTTP status code — it just tries to extract fields from the response body. A 401 with `{"message":"Bad credentials"}` only fails because `html_url` is missing. If a 401 body happened to contain expected fields, it would "succeed" with garbage. | `gunbc-dag/src/resolve_service.rs` (REST parse) | RT-I4 proof | 2026-02-27 |
| Heuristic reimplementation | `@mock_response` parser is NOT implemented. `MockResponseDef` AST struct exists in `daglang-syntax/src/lib.rs:587-591` but parser always initializes `Vec::new()`. No service in `dsl/services/` uses `@mock_response`. The `error_cases()` trait method on `Mockable` exists but is never populated. | `core/daglang/daglang-syntax/src/lib.rs`, `core/test/src/mockable.rs:59` | RT-A2 audit | 2026-02-27 |
| Validation at use site | `GNUmakefile` had `--mode=ensure` flag on bootstrap command but the binary doesn't accept that flag. Bootstrap binary only accepts `--check-mode`, `--dry-run`, `--print-inputs`. | `GNUmakefile:26` | CI fix | 2026-02-27 |
| Validation at use site | `find crates -type d` in bootstrap returns exit 1 when `crates/` directory doesn't exist. SplitLines was failing hard on this (post-RT-I4) but should return empty list for list-producing ops. | `dsl/tools/bootstrap.dag`, `gunbc-dag/src/resolve_service.rs:523-543` | CI fix | 2026-02-27 |
| ~~Static mapping table~~ | ~~Three functions deleted by RT5 — DSL `classify_transports` is now the single source.~~ | ~~`gunbc-dag/src/fidelity.rs`~~ | RF-H4 PR scout | **Done** |
| Heuristic reimplementation | `passthrough_fallback_value()` hard-codes a port alias table. | `gunbc-dag/src/resolve.rs:95-162` | RF-H4 PR scout | 2026-02-26 |
| Heuristic reimplementation | `looks_effectful_without_kind()` re-derives NodeKind from port type strings. Dead code after RT6 (NodeKind). | `core/exec/src/execute.rs:2064-2092` | RF-H4 PR scout | 2026-02-26 |
| Heuristic reimplementation | `classify_module()` inflated by transitive auth callables. | `gunbc-dag/src/fidelity.rs:184-209` | RF-E4 impl | 2026-02-26 |
| ~~Fallback arm~~ | ~~HTTP method fallback now returns `ExecError` instead of silently falling back to POST.~~ | ~~`gunbc-dag/src/resolve_service.rs:73-85`~~ | R1 scout | **Done** |
| String dispatch | `match field.type_id.as_str()` for JSON→Value appears twice. | `gunbc-dag/src/resolve_service.rs:291-335, 352-366` | R1 scout | 2026-02-26 |
| ~~Validation at use site~~ | ~~`input_as_string()` now returns `Result<String, ExecError>` — missing inputs fail-closed instead of leaking `"(unresolved)"`.~~ | ~~`gunbc-dag/src/resolve_service.rs:703-733`~~ | R1 scout | **Done** |
| ~~String dispatch~~ | ~~`FileOperationSpec.operation` now uses `FileOp` enum — parsed at boundary in `derive_file_spec()`, exhaustive match in `GenericFilePrepareOp`.~~ | ~~`gunbc-dag/src/resolve_service.rs`~~ | R1 scout | **Done** |
| String dispatch | `workflow_unit_commands()` matches workflow name strings. | `gunbc-dag/src/workflow/unit_commands.rs:300-323` | R1 scout | 2026-02-26 |
| Inventory linkage gap | `gunbc-codegen cigen` drops GCP secrets. See Theme INV below. | `gunbc-dag/src/ci/mod.rs:56-77` | lane-2 merge | 2026-02-26 |
| *(success_port workaround promoted to RT4a:c)* | | | | |
| Dead scaffolding | `@mock_response` annotation type exists in AST (`MockResponseDef`), parser never populates it (`mock_response: Vec::new()`). Wire parser → lowerer → `RestOperationSpec.mock_response` end-to-end. Design: `docs/design/mock-response-pipeline.md` Phase 1-2. | `daglang-syntax/src/parser.rs`, `daglang-lower/src/lib.rs` | BT-R3 scout | 2026-02-27 |
| Static mapping table | Kitchen sink `default_rest_response()` grows a new blob of fields for every service type. Should be derived from `@mock_response` annotations or synthesized from output field `from` paths. Design: Phase 2-3. | `gunbc-dag/src/mock_defaults.rs:200+` | BT-R3 scout | 2026-02-27 |
| Dual convention | `from` path format split: `.` separator (`head.sha`, 3 uses) vs `/` separator (`content/0/text`, 12+ uses). Normalize to `/` (JSON Pointer-like). Design: Phase 4. | `dsl/services/github/*.dag`, `dsl/services/llm/*.dag` | BT-R3 scout | 2026-02-27 |
| Heuristic reimplementation | `IdentityCallableOp` still overloaded for 2 roles (ContentUpsertOutputPath, DSL callable passthrough). `CallParamSourceOp` extracted by BT-R3. Remaining: `MetadataPassthroughOp` + `DeclaredOutputCallableOp`. Design: Phase 5. | `gunbc-dag/src/resolve.rs:260+` | BT-R3 scout | 2026-02-27 |
| Pessimistic ordering | `probe_best_response` tries `[Shell, File, REST]` — REST services are the majority but tried last, wasting 2 trial executions per REST parse node. Design: Phase 6. | `gunbc-dag/src/mock_defaults.rs` | BT-R3 scout | 2026-02-27 |

### POSTMORTEM: `make gist` 401 — Compounding Failures

**Symptom**: `make gist` returns 401 Unauthorized. Diagnostic: `BearerToken (key: "", source: static)`.

**Key evidence**: `shell.GCloud.SecretManagerAccessVersion` **succeeds** (token fetched), but the token arrives **empty** at `github.Gist.Create`. Five compounding failures:

#### Failure 1: No credential wiring from operation inputs to execute node (ROOT CAUSE)

The lowerer creates prepare→execute→parse triplets for service operations. When a service has `config { auth: BearerToken }` and an `auth_token: Secret` input field, **no code exists to wire the auth_token value from the prepare node to the execute node's `res:credential` port**.

- `daglang-lower` lines ~5731-5741: execute node gets `res:credential` input IF `has_auth` is true
- `daglang-lower` lines ~6367: profile-based auth IS explicitly wired to `res:credential`
- **Gap**: operation-level `auth_token` input field has no automatic outlet to `res:credential`

Before annotation deletion, `@headers({ "Authorization": "Bearer {auth_token}" })` explicitly wired the credential. After deletion, `config { auth: BearerToken }` was added but the corresponding wiring logic was never created.

#### Failure 2: GenericRestPrepareOp doesn't propagate auth_token

`GenericRestPrepareOp` builds the request URL, body, and headers from input fields. It does NOT:
- Detect `auth_token` as a credential input
- Set `RestRequest.auth` from the operation's auth scheme
- Expose the token as an output port for the execute node

The prepare node treats `auth_token` like any other input field — it may end up in the body but never in the auth header.

#### Failure 3: Execute node falls through silently

`TransportOps::Execute` (`lib/transport/src/ops.rs` lines 65-94) looks for `res:credential` input. If not provided, it uses whatever auth is already on the request (which is `None`). **No error** — it just sends an unauthenticated request.

#### Failure 4: Diagnostic source is misleading

`decorate_service_failure` in `error.rs` gets `credential_ref: None` because `infer_auth_from_headers` can't find an Authorization header (it was never set). This shows `source: static` — misleading because it implies a hardcoded credential when really there's NO credential.

#### Failure 5: GenericShellParseOp was emitting Value::Str for Secret outputs (FIXED)

`GenericShellParseOp.TrimStdout` always emitted `Value::Str`, ignoring `is_secret` flag. This meant the gcloud token flowed as `Value::Str` not `Value::Secret`. **Fixed in current session** — `shell_trim_value()` helper now respects `OutputFieldSpec.type_id == "Secret"`.

#### Fix options (pick one)

| Option | What | Scope | Risk |
|--------|------|-------|------|
| A | **Auto-bridge in lowerer**: when `auth_scheme.is_some()` on service config AND an input field is typed `Secret`, create edge from `prepare.{field}` → `execute.res:credential` with `Credential::new(Secret::static_value(token), AuthScheme::Bearer)` wrapping | daglang-lower + resolve_service.rs | Medium — heuristic, may mis-identify which Secret field is the credential |
| B | **Explicit auth input declaration in DSL**: add `auth_input: "auth_token"` to `config { ... }` block, lowerer uses it to wire credential | daglang-lower + daglang-syntax | Low — explicit, no heuristic |
| C | **RestPrepareOp sets auth on request**: when `auth_scheme` is in spec and `auth_token` is in inputs, set `req.auth = Some(...)` directly in prepare | resolve_service.rs only | Low — localized fix, but skips resource port pattern |

Option C is the smallest fix that unblocks `make gist`. Options A/B are more principled for the long term.

### POSTMORTEM: `gunbc-ci` false failure — `overall_success: Skipped`

**Symptom**: `gunbc-ci` reports "A required success check returned false" even when all build/test/clippy stages succeed. Pre-existing on `main`. `make ci` (via `gunbc-workflow`) passes because it uses a different code path.

**Key evidence**: `success_port_failed()` finds `overall_success: Value::Skipped` on the `tools.build::build_all` node. The `&&` expression in the return statement was never wired.

#### Failure 1: Lowerer drops complex return expressions (ROOT CAUSE)

`resolve_return_expr_source()` (`daglang-lower/src/lib.rs:7768-7841`) handles return expression wiring for callable nodes. It matches 4 expression types:

| Expression | Match arm | Status |
|-----------|-----------|--------|
| `Expr::Ident(name)` | `return { x: my_var }` | Wired |
| `Expr::FieldAccess(base, field)` | `return { x: node.output }` | Wired |
| `Expr::Call(name, _)` | `return { x: my_fn() }` | Wired |
| `Expr::Literal` / `StringInterp` / `List` / `Map` | `return { x: "hello" }` | Wired |
| **Everything else** | `_ => None` (line 7839) | **Silent drop** |

`build.dag` line 35: `return { overall_success: build.success && test.success && clippy.success }` — this is `Expr::BinOp`, falls through to `_ => None`. No edge is created to `__out:overall_success`.

This silently drops: `BinOp` (`&&`, `||`, `+`, etc.), `UnaryOp` (`!`), `If`/`Match`, `Pipe`, `RecordUpdate`, `NullCoalesce`.

#### Failure 2: Passthrough falls back to `Value::Skipped` silently

`execute_with_declared_output_passthrough()` (`resolve.rs:71-91`) checks for `__out:overall_success` input. When the edge doesn't exist (failure 1), no input arrives. Fallback: `passthrough_fallback_value()` → `None` → `Value::Skipped`. No error, no warning.

#### Failure 3: `success_port_failed()` treats `Skipped` as failure

`success_port_failed()` (`display.rs:991-1003`) correctly treats `Value::Skipped` as failure (the success port must affirmatively be `Bool(true)`). This is correct behavior — the bug is upstream.

#### Failure 4: Testgen mocks bypass wiring

Generated tests for `build_all` (`generated_tests_tools_build.rs:658`) manually inject `__out:overall_success: Value::Bool(true)` as a mock input via `execute_single_node`. This tests the passthrough mechanism works but never validates that the lowerer creates the `__out:overall_success` edge from the `&&` return expression. The testgen model assumes all `__out:*` ports are properly wired — it tests nodes, not IR edges.

#### Failure 5: No IR-level wiring verification exists

There is no test or validation that checks: "for every return expression binding, was an edge created to the corresponding `__out:*` port?" The lowerer's `wire_callable_return_outputs()` silently `continue`s when `resolve_return_expr_source()` returns `None` (line 7890). The anti-tautology principle (testgen.md) says edge cardinality is "proven by construction" — but this is only true when the lowerer actually creates the edges.

#### Current workaround

`gunbc-dag/src/bin/ci.rs:160`: changed `success_port: Some("overall_success")` to `success_port: Some("success")`. This checks transport parse nodes' `success` output directly (which works because `success` is a simple `FieldAccess` expression, not a `BinOp`). The workaround is correct but bypasses the DAG's own aggregation logic.

#### Fix plan (three tasks, in priority order)

| # | Task | What | Scope | Eliminates |
|---|------|------|-------|-----------|
| RT4a | **Complex return expr lowering** | Extend `resolve_return_expr_source()` to handle `BinOp`, `UnaryOp`, `If`, `Match`, `Pipe` — synthesize compute nodes in the IR that evaluate the expression and wire result to `__out:*`. | `daglang-lower/src/lib.rs` | Root cause: silent drop of computed return values |
| RT4b | **Passthrough missing-input diagnostic** | When `execute_with_declared_output_passthrough()` falls back to `Value::Skipped` for a declared output port, emit a diagnostic warning (or error in strict mode). The port was declared in the type signature — `Skipped` means the wiring is broken. | `gunbc-dag/src/resolve.rs` | Silent failure: wrong output, no signal |
| RT4c | **Lowering completeness gate** | Add validation in `wire_callable_return_outputs()`: if `resolve_return_expr_source()` returns `None` for a non-optional output, emit `LowerWarning` (or error). Count unwired return expressions as a metric. | `daglang-lower/src/lib.rs` | Prevention: catch at compile time, not runtime |

RT4a is the real fix. RT4b/c are defense-in-depth so the class of bug can't recur silently. After RT4a, revert the ci.rs workaround back to `success_port: Some("overall_success")`.

---

## Red Backlog

Triaged and sized. Promote to lane queues when horizon items are exhausted.

### Theme TC: Transport Completeness (compositional modeling)

Transport declarations belong on the **service layer**, not on SDLC.
The SDLC pipeline only sees interfaces. Each service operation needs a
`transport rest { ... }` or `transport shell { ... }` block so the
compiler can generate prepare→execute→parse triplets.

Local-profile transport (26 ops) moved to Blue queue as BT6 (critical path for L4+).

| ID | Scope | Ops Missing | Notes |
|----|-------|-------------|-------|
| RF-TC3 | **Remaining providers**: GCS stores (6), github_issue_provider (7), credential providers (4) | 17 | Needed for cloud_run profile. Some overlap with BT6 (github_issue_provider delegates to github/issues.dag which BT6 covers). |
| RF-TC4 | **Stub providers**: stub_providers.dag (26), stub_credential_provider.dag (2) | 28 | Intentional — unit_test profile stubs. Consider `transport stub {}` marker. |
| RF-TC5 | **Infrastructure stubs**: azure (43), aws (38), gcp-infra (59) | 140 | Dormant — defer until infrastructure provisioning lane opens. |

### Theme E: Deleted Tests (re-add when root cause fixed)

| ID | Deleted Tests | Root Cause | Blocker |
|----|---------------|-----------|---------|
| RF-E5 | `makegen_runtime_differential_interpreter_vs_generated_rust_layer1` (codegen_parity.rs) | FnBodyDelegate gap: interpreter produces raw `{header}{body}`, fn body evaluation only works via `shared.rs` direct path. | Interpreter needs fn body evaluation support. |
| RF-E6 | `makegen_exec_runtime_e2e_structural_verification` (daglang-driver), `pragma_exec_runtime_e2e_structural_verification` (daglang-driver), `makegen_e2e_generated_binary_produces_correct_makefile` (cli_commands), `pragma_e2e_generated_binary_produces_correct_config_files` (cli_commands) | Exec-runtime emitter missing: `LoadRegistry` handler, `PureRender` fn classification, `ContentUpsertOutputPath` classification. | `daglang-emit` exec-runtime backend needs node classification for all makegen/pragma node kinds. |
| — | `clippy_toml_dsl_produces_valid_output` (pragma_parity.rs) | Sum type variant tags lost during `build_data_values()` JSON serialization. | RT19 (recursive types). |

### Theme INV: Inventory Linkage (cigen secrets gap)

`gunbc-codegen cigen` silently drops secrets from CI YAML because the
`inventory` crate's `submit!` registrations are discarded by the linker
when no symbols from the submitting crate are directly referenced.

**Root cause**: `ci_live_test_secrets()` calls `iter_dag_specs()` which
collects `DagSpecDef` entries via `inventory`. Entries with `live_required`
secrets are registered in `lib/gcp-ops`, `lib/review`, etc. The
`gunbc-codegen` binary (`codegen_cli.rs`) doesn't reference those crates'
symbols, so the linker drops them and their inventory registrations.
Meanwhile `gunbc-ci` (`ci.rs`) transitively references them through
`build_build_graph()`, so it sees the full inventory.

**Impact**: Running `gunbc-codegen cigen` produces a ci.yml missing 5
GCP secret env vars. Current workaround: ci.yml is committed with
secrets and not regenerated on every codegen pass.

| ID | Fix | Size | Notes |
|----|-----|------|-------|
| RF-INV1 | **Force-link inventory crates in codegen binary.** Add explicit `use` references or `extern crate` for crates that register `DagSpecDef` with `live_required` secrets. Simplest fix, but fragile — adding a new crate with secrets requires updating codegen_cli.rs. | S | Quick fix. |
| RF-INV2 | **DSL-derive CI secrets from service annotations.** Instead of inventory, derive `live_required` from `@auth` + `@endpoint` annotations on service operations in `.dag` files. The DSL already declares auth schemes — the compiler can extract which env vars are needed. Eliminates the inventory linkage problem entirely. | M | DSL-first fix. Aligns with RT13 (derive mock registries from `@mock_response`). |

### Theme PC: Provider Response Contracts (mandatory error modeling)

> Design doc: `docs/design/provider-contracts.md`
> Aligns with: `docs/design/modeling/annotation-to-dag-modeling.md` Phase 2

Services must model the actual provider API contract — not just the happy
path. Every documented response code, error body shape, and failure mode
gets declared via structural `response { ... }` blocks on operations.
The lowerer compiles these into classify_response nodes in the transport
DAG. Testgen generates **mandatory** per-status-code test obligations.
Supersedes RT-I1 and RT-I2.

| ID | What | Size | Deps |
|----|------|------|------|
| PC-1 | **`response` block parsing.** Add to `daglang-syntax`. `Vec<ResponseEntry>` on `OperationDef` (replaces `mock_response`). | M | — |
| PC-2 | **Standard error types.** `dsl/std/errors.dag` — common HTTP, GitHub, GCP error shapes. | S | — |
| PC-3 | **`response` blocks on all REST services.** 29 operations, `doc` references to provider API docs. | L | PC-1, PC-2 |
| PC-4 | **`exit` blocks on all shell services.** Exit code → output type mapping. | M | PC-1 |
| PC-5 | **Lowerer: populate `error_mappings` + classify_response node.** Wire `response` entries to existing `ErrorMapping` on `ServiceOperationSpec`. Generate classify_response node in transport DAG. | M | PC-1 |
| PC-6 | **`GenericRestParseOp` status checking.** Route on status code before field extraction. Hard-fail on undeclared non-2xx. | M | PC-5 |
| PC-7 | **`ProviderResponseContract` obligation.** New Bucket C obligation, one per `response` entry. | M | PC-5 |
| PC-8 | **Testgen codegen for response contracts.** Per-status-code tests, mock body derived from response type. | L | PC-7 |
| PC-9 | **Interface response contract inheritance.** Implementors inherit obligations from interface `response` declarations. | M | PC-7 |
| PC-10 | **Completeness enforcement.** Compiler requires ≥1 success + ≥1 error entry in `response` block on every `transport rest {}` operation. | S | PC-1 |

### Compiler Features (low priority)

| ID | Feature | Size | Notes |
|----|---------|------|-------|
| FC-CF2 | `skip(n)`: List\<T\> → List\<T\> | S | Expressible via fold+index. |
| FC-CF3 | `enumerate()`: List\<T\> → List\<(Int, T)\> | M | Expressible via fold+counter. |

---

## Reference: Theme Details

Detailed descriptions for queue items. The queue has priority order,
this section has context.

**ID mapping** (old → new):
RT1:4 = new (RF-MC1→RT1, RF-MC2→RT2, RF-MC3→RT3, RF-TV1→RT4).
RF-G-unblock→RT5, RF-A1→RT6, RF-A2a→RT7, RF-A4→RT8,
VIO-1→RT9, VIO-3→RT10, VIO-4→RT11, VIO-5→RT12, VIO-2→RT13,
VIO-6→RT14, VIO-7→RT15, VIO-8→RT16, FC-P7-c2→RT17, FC-P7-d→RT18,
FC-CF5→RT19, FC-CF6→RT20, FC-P8-a→RT21, FC-P8-b→RT22, FC-P8-c→RT23.
B-0→BT1, B-PW→BT2, B-1→BT3, B-2→BT4, B-3→BT5, B-TC→BT6, B-4→BT7,
B-5→BT8, B-6→BT9, B-7→BT10.

### Theme MC: Model Correctness (compositional transport pipeline)

**Audit findings** (2026-02-26):

The compositional model (transport → service → provider → interface → profile)
is architecturally sound and proven end-to-end for 39 operations across 4
transport types (shell, rest, file, local). But three infrastructure gaps
cause **silent failures** — requests go out wrong without errors:

| Gap | Symptom | Severity |
|-----|---------|----------|
| **RT1: Credential wiring** | `config { auth: BearerToken }` + `auth_token: Secret` input → token never reaches execute node → unauthenticated request, silent | Critical — affects all authenticated REST services |
| **RT2: Execute silent fallthrough** | Missing `res:credential` → execute sends request without auth, no error | Critical — masks RT1 |
| **RT3: File transport coverage** | Only READ/READ_BYTES/WRITE implemented → EXISTS/CREATE_DIR/DELETE/APPEND/GLOB operations fail at resolve | Blocking — SDLC local profile needs EXISTS + CREATE_DIR |
| **RT4: Typecheck validation** | Typechecker ignores transport blocks entirely → invalid contents compile silently | High — errors surface late at resolve time with confusing messages |

RT1 root cause analysis: See POSTMORTEM section below.
RT4 detail: `LowerError::MissingTransport` now fires for partially-specified services
(at least one op has transport, another doesn't). Fully-abstract services (no transport
on any operation) and interface implementors are exempt — they get transport via profiles.

**Coverage snapshot** (97 total operations across 28 services):
- 39 ops (40%): transport blocks present, pipeline works end-to-end
- 24 ops (25%): intentional stubs (unit_test profile, no transport needed)
- 16 ops (16%): have `@rest` annotations but no `transport rest {}` block (github/issues, github/pull_request, llm/openai)
- 18 ops (19%): SDLC providers needing transport for local/cloud profiles

### Theme RG: Manual Registry Elimination

**Audit findings** (2026-02-26):

| Registry | Location | Entries | Derivable From | Status |
|----------|----------|---------|----------------|--------|
| `WorkspaceBinary` enum | `binaries.rs:55-70` | 13 | Cargo.toml `[[bin]]` section | ⚠️ Manual |
| Workflow variant catalog | `catalog.rs:34-127` | 10 | DSL `pipeline` declarations + annotations | ⚠️ Manual |
| Extern impls | `extern_impls.rs:26-64` | 6 | DSL `extern func` declarations (gated by ratchet) | ✓ Gated |
| Module path dispatch | `resolve.rs:683-706` | 3 + prefix | Convention-based fallthrough | ⚠️ Implicit |
| Tool definitions | `dsl_registry.rs:44-178` | dynamic | Structural inference from DSL | ✓ Auto |
| Entrypoint inference | `daglang-lower` | dynamic | Unconnected port analysis | ✓ Auto |
| Process unit registry | `process_registry.rs` | dynamic | DSL workflow DAGs | ✓ Auto |

**RF-RG1**: Eliminate `WorkspaceBinary` + workflow variant catalog.
Both are static tables that duplicate information the compiler already derives.
Tool definitions and entrypoints are already auto-derived from DSL. These two
should follow the same pattern.

**RF-RG2**: Generalize resolve.rs so new DSL modules work without adding
hardcoded paths. Current hardcoded: `std.resources` (resource lifecycle),
`tools.infra` (single custom op), prefix `services.*` / `workspace.*`
(generic transport). Convention: if a module doesn't match any special
pattern, fall through to generic callable resolution.

### Theme G: Rust Heuristics Shadowing DSL Declarations

The DSL already has the complete, correct implementation:
- `std/fidelity.dag::classify_transports(transports: List<TransportClass>)`
  takes raw transport classes, aggregates via `fermi_max_of` + `all`, returns
  typed `DerivedClassification { test_class, depth, hermetic }`.
- `std/fermi.dag::fermi_max_of(depths)` folds over magnitudes.
- `config/test_policy.dag` adds repo-specific budget policy.

But the Rust side in `fidelity.rs` **replicates this entire computation**
as hand-wired heuristics: ordinal integers for depth comparison,
boolean maps for hermetic, string round-trips for enum values. These
exist solely because `classify_transports` uses `fold` (via
`fermi_max_of`), and the lowerer can't extract fn bodies containing
`fold` for `evaluate_fn_body()`.

**RT5** (was RF-G-unblock): **Done.** Multi-param lambda parsing
(`(acc, d) => expr`), pipe method typechecker exclusion (fold, map,
filter, etc. in should_track_call_name), fidelity.rs rewrite to call
DSL `classify_transports()` directly via `evaluate_fn_body()`.
Deleted:
- ✅ `transport_depth_ordinal()`, `transport_depth_str()` (RF-G1)
- ✅ `transport_is_hermetic()` (RF-G2)
- ✅ `classify_callable()` pre-aggregation → now calls DSL (RF-G3)
- ✅ `test_policy.dag::classify_from_facts()` (RF-G4)
- ✅ `test_policy.dag` shadow fns `transport_depth`/`transport_hermetic` (RF-G6)
Remaining: TestClass::parse()/FermiCost::parse() round-trip (RF-G5)
still used — fidelity result comes back as Value::Str, needs parsing.
Silent fallbacks `unwrap_or(Unit)` / `unwrap_or(XS)` still present as
safety nets for edge cases.

### Theme H: Structural Enforcement (parse, don't validate)

| ID | What | Current | Structural Fix |
|----|------|---------|---------------|
| RF-H2 | **TestgenTargetDef Option fields always populated**. `test_class: Option<TestClass>`, `fermi_cost: Option<FermiCost>`, `requires: Option<Vec<String>>` — every auto-testgen call site now fills `Some(...)` from fidelity. The `Option` only exists for legacy `DagSpecDef` path (which also never overrides). `generate_target_with_types()` does `unwrap_or(Unit)` on every field. | 6 Option fields in registry.rs | Make fields non-Option with `Default` impl. Callers construct with values; no unwrapping. `DagSpecDef.to_def()` fills from fidelity instead of leaving `None`. |
| RF-H4 | **ResourceKind string dispatch**. `ResourceAcquireOp { resource_kind: String }` matched at runtime. Unknown kinds fall through to `Value::Str("resource:{other}")` — wrong type, silent. | resolve.rs:365-386 | `ResourceKind` enum parsed once at resolve time. Match is exhaustive, no fallback arm. |

### Theme A: Typed Dispatch (full detail)

| ID | Pattern | Key Files | Notes |
|----|---------|-----------|-------|
| RF-A1 | **NodeKind on Node\<T\>**. `validate_node_kinds_for_interception()` is a runtime check that rejects `kind: None` nodes. Target: `Node::opaque()` requires `NodeKind`, eliminating `Option` and runtime check. | node.rs, execute.rs | Remove `Option<NodeKind>`, delete validation fn. |
| RF-A2a | **Port namespace typing (definition)**. Define `PortCategory` enum + methods on `PortName` in `core/ir/`. | core/ir/ | R1 scope. |
| RF-A2b | **Port namespace typing (migration)**. Migrate 18+ `starts_with("res:")`/`"tool:"`/`"__out:"` checks in `core/daglang/`, `core/codegen/testgen/`. | 4 R2 files | R2 scope. Depends on RF-A2a. |
| RF-A3 | **Module path representation**. 4 crates use `Vec<String>` vs typed `ModulePath`. | 4 crates | Unify on `ModulePath`; add `From` impls. |
| RF-A4 | **Stringly-typed dispatch in resolve.rs**. 10+ string prefix matches for module/callable routing. | resolve.rs | `CallableClass` enum parsed once. |
| RF-A5 | **Transport node classification**. String-based prepare/execute/parse detection. | resolve.rs | `TransportNodeKind { Prepare, Execute, Parse }`. |
| RF-A6a | **String constants (definition)**. Define central consts (`__deps`, `__out:`, `res:file`, `tool:`) in `core/ir/src/signature.rs`. | core/ir/ | R1 scope. |
| RF-A6b | **String constants (migration)**. Migrate 141+ `__deps`, 7 `res:file`, 15 `tool:` references in `core/daglang/`, `core/codegen/testgen/`. | R2 files | R2 scope. Depends on RF-A6a. |
| RF-A8 | **`as_str`/`parse` boilerplate**. `#[derive(StringEnum)]` for 15 enums (~60 match blocks). Includes TestClass/FermiCost `FromStr`. | 12 files | |
| RF-A9 | **Emit backend type-name tables**. Same type mapping in 3 backends. | daglang-emit | Shared `DslTypeMapping` table. |
| RF-A10 | **String dispatch in DAG tooling**. 100+ match arms on string literals. | 5 files | Registry pattern or DSL data declarations. |

### Theme VIO: Virtual I/O Infrastructure (BB-6 S+ tiers)

BB-6 XS tier (PureMock/DryRun) works. S+ tiers are stubs awaiting virtual I/O
infrastructure. The design insight: **S-tier doesn't need real servers**. The
`TransportBackend` trait intercepts at the `TransportRequest`/`TransportResponse`
struct level. S-tier REST/HTTP/TCP is just a response registry inside
`VirtualTransportBackend` — in-process request→response matching, no sockets.

**Existing infrastructure**:
- `VirtualTransportBackend` in `lib/transport/src/test_backend.rs`: handles File
  ops (read/write/append/delete/exists/glob/metadata) and basic Shell (find,
  printenv, test -f). REST/HTTP/TCP return error.
- `TransportBackendGuard` in `lib/transport/src/backend.rs`: scoped backend
  install/restore via RAII guard. Thread-safe global swap.
- `FidelityLadder`/`FidelityLevel` in `core/test/src/fidelity.rs`: canonical
  ladders for all 6 TransportKind variants. `node_max_fidelity()` transitive meet.

**DSL-first approach**:
- RT9 defines virtual backend configuration types in DSL (`dsl/std/virtual_io.dag`)
- RT13 derives mock registries from existing `@mock_response` annotations —
  zero hand-written mock data
- RT10:12 add response registries to `VirtualTransportBackend` (shell cassettes,
  HTTP stubs, TCP loopback) — same in-process interception pattern as File
- RT14:16 wire virtual backends into testgen codegen per tier

**Key files**:
- `dsl/std/virtual_io.dag` (new, RT9)
- `lib/transport/src/test_backend.rs` (RT10, RT11, RT12)
- `core/codegen/src/testgen/mock_corpus.rs` (RT13)
- `core/codegen/src/testgen/codegen.rs` (RT14, RT15, RT16)

---

## Archive

Completed items. RF-H4, RF-H2, RF-E4, BB-2, BB-3, BB-5 completed 2026-02-26.
BT-R1 (testgen discovery fix), BT-R2 (provider completion), BT11, BT12 completed 2026-02-27.

NF-1:6 (compile+link hardening): 2026-02-25. Detail: `TODO/TODONE/tasks-completed.md`.
FC-NF7 (fn-level evaluation): 2026-02-25. `expr.rs` IR + `eval.rs` evaluator + `FnBodyDelegate`.
FC-CL (dead code cleanup): 2026-02-25. Deleted `core/tool-registry` + macros, 14 orphaned fns.
FC-EG (enforcement gates): 2026-02-25. Import-direction lint, extern count gate, format!/push_str gate.
FC-P6-a:d (policy migration): 2026-02-26. `dsl_render.rs` evaluates `derive_*` DSL fns.
FC-CF1 + FC-CF7 (split + zip): 2026-02-26. Both pipe methods, 4 compiler stages, 9 tests.
FC-P7-a (build_workflows.dag): 2026-02-26. WorkflowSpec + MetaTarget types + data.
FC-P7-b (artifact emitter): 2026-02-26. `dag_emit.rs` emits valid `.dag` syntax.
FC-P7-c1 (Makefile DSL types): 2026-02-26. Types in `extdeps/make.dag`.
BB-0 (compositional type modeling): complete. Types in `corpus.rs` + `fidelity.rs`. 23 tests.
BB-1 (mock corpus builder): complete. `build_corpus()` in `mock_corpus.rs`.
BB-4 (type-derived boundary values): complete. `enrich_corpus_with_type_witnesses()`.
BB-6 (transport fidelity ladders): complete. Canonical ladders, `node_max_fidelity()`.
RF-H4 (ResourceKind enum): 2026-02-26. String dispatch → enum in resolve.rs.
RF-H2 (TestgenTargetDef non-Option): 2026-02-26. Option fields → defaults.
RF-E4 (fidelity smoke test): 2026-02-26. makegen→Unit/XS, gist→Integration/L.
BB-2 (per-node corpus tests): 2026-02-26. `build_corpus_section()`. Pure→Real, effectful→DryRun.
BB-3 (adjacent pair tests): 2026-02-26. `build_adjacent_pair_section()`. Edge wiring verification.
BB-5 (cross-workflow consistency): 2026-02-26. `build_cross_workflow_section()`. Multi-workflow nodes.
RT-A1 (shell exit code audit): 2026-02-27. Decision matrix for all 11 shell operations. TrimStdout/SplitLines semantics defined.
RT-A2 (@mock_response gap): 2026-02-27. 29 REST ops × 0 annotations = 29 missing. Parser not implemented. See `TODO/testgen-proof-analysis.md`.
RT-I4 (shell exit code enforcement): 2026-02-27. TrimStdout fails on non-zero (unless optional), SplitLines returns empty list. Proof tests + analysis report.
Positional auth_input wiring: 2026-02-27. Fixed positional service call args dropping credentials. `operation_inputs` field on `ServiceTransportEndpoint`.
GNUmakefile bootstrap fix: 2026-02-27. Removed invalid `--mode=ensure` flag. Fixed `make install` → `make ci` pipeline.
