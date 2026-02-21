# Completed Tasks — Archived from tasks.md

**Moved**: 2026-02-19

---

## Sprint 1: Get to Green (Complete 2026-02-19)

2984 passing, 0 failures.

| ID | Task | Status |
|----|------|--------|
| F-GCP | GCP prepare ops: graceful `unwrap_or("(unresolved)")` for missing `audience`/`project`/`secret`/`subject_token` inputs in `ServiceGcpStsExchangePrepareOp` and `ServiceGcpSecretManagerAccessVersionPrepareOp` (resolve.rs) | Done 2026-02-19 |
| F-OBS | Observability invariant: `auto_mock_spec` fallback NonEmpty matchers for `IdentityCallableOp` terminal nodes (mock_defaults.rs), testgen regenerated | Done 2026-02-19 |
| F1 | dag_viz: removed SubDag wrapper NodeExamples (`gist_upload`/`browser_open`) from `auto_mock_spec` — wrapper nodes don't exist in lowered DAG, so `execute_single_node` can't find them. Observability analysis runs on lowered DAG, so no matchers needed. | Done 2026-02-19 |

---

## Sprint 2: Review Findings + Polish (Landed 2026-02-19)

| ID | Task | Status |
|----|------|--------|
| R1 | Makegen transport port naming alignment (`response` across lowerer parity filter, exec runtime emitter, and makegen mocks). | Done 2026-02-19 |
| R3 | IR schema enriched with typed managed bindings (`Stmt::Bind` with explicit `BindIntent` and `BindTarget`), Go lowering/rendering migrated off string-encoded multi-bind syntax, and backend migration notes encoded in updated lowerers/renderers/tests across Go/C/MIPS. | Done 2026-02-19 |
| R4 | Go/C transport-statement lowering now isolates synthetic error-code/error bindings in lexical block scope, with structural regressions for repeated transport expressions and verified Go/C toolchain smoke compilation for the scoped outputs. | Done 2026-02-19 |
| R5 | MIPS lowering now routes returns through `JumpEpilogue`, temp allocation is fail-closed with explicit `LowerError` on exhaustion, and C block-scope locals are tracked with enter/exit visibility scopes to prevent leakage/aliasing across blocks. | Done 2026-02-19 |
| R6 | Cross-backend adversarial harness added in `daglang-emit`: shared fixture lowered across Go/C/MIPS, structural invariants asserted (Go/C scoped transport bindings, MIPS epilogue routing/no direct body `jr $ra`), and Go/C smoke compilation executed with hermetic temp caches; CI-oriented `make test*` commands now require backend toolchains (`GUNBC_REQUIRE_BACKEND_TOOLCHAINS=1`). | Done 2026-02-19 |
| R8 | `MethodMeta` request wiring centralized through shared `request_from_meta(_at)` helpers; GCP service methods migrated off duplicated endpoint strings and parity tests added to catch metadata/request drift. | Done 2026-02-19 |
| R9 | Infra CLI `parse_input_value` fail-closed parsing now type-driven via `ValueBacking` + compatibility checks; structured JSON/list/map/set parsing and incompatibility/unsupported tests added. | Done 2026-02-19 |
| R10 | `SystemModel` REST invocation paths now use named placeholders and validation enforces wildcard ban + placeholder↔required-input binding (with invalid-path regression tests). | Done 2026-02-19 |
| R11 | Strict parsing APIs (`try_parse`/`FromStr`) landed for `Arch`/`Vendor`/`Os`/`AbiEnv`/`ExecutionEnv` with tolerant `parse` retained for host detection paths; strict-vs-tolerant tests added. | Done 2026-02-19 |
| R12 | Mock default seeding now prefers refined typed GCP semantic aliases, with legacy port-name heuristics isolated behind an explicit compatibility fallback; rename-resilient typed seeding tests added. | Done 2026-02-19 |
| P1 | `daglang-derive` capture mode now derived structurally from `obligation` + `is_interactive` metadata (no blanket captured default logic). | Done 2026-02-19 |
| P2 | `daglang-derive` interactive node detection now uses structural `is_interactive: bool` on `LoweredOp::Callable` (no `name.contains("@interactive")`). | Done 2026-02-19 |
| P3 | `daglang-derive` resource usage derivation now uses `obligation` enum + `resource_target` metadata (no string prefix stripping). | Done 2026-02-19 |
| P4 | `daglang-cli check` no longer re-runs discovery/parse/typecheck after pipeline build; reuses build-stage module graph. | Done 2026-02-19 |
| P5 | GCP impersonation parse now surfaces `expires_at` output (and propagates empty string when skipped), with graph/output tests updated. | Done 2026-02-19 |
| P8 | Repeated GCP REST client constructors (`new` / `unauthenticated`) consolidated via shared helper macro across service clients. | Done 2026-02-19 |
| P9 | `content_upsert` source wiring deduplicated via shared helpers for resolved-source / param-source fanout paths. | Done 2026-02-19 |
| P10 | makegen compile tests now use a shared fixture object with automatic temp output cleanup via `Drop`. | Done 2026-02-19 |
| P11 (Sprint 2) | `build_workspace_dag_from_discovery(tool_names, pipeline_names)` extracted as a pure composition entrypoint; impure discovery wrapper delegates to it. | Done 2026-02-19 |

---

## Completed Near-Term Polish

| ID | Task | Status |
|----|------|--------|
| P7 | Remove `dedupe_release_resource_edges` (resolve.rs): tracked already-wired `(release_node, port)` pairs in lowerer via `wired_release_targets: HashSet`, seeded from lifecycle acquire→release edges. Removed workaround from resolve.rs. | Done 2026-02-17 |
| P11 | Auto-mock seeding for GCP service inputs: added `gcp_field_value()` in mock_defaults.rs for `audience`/`project`/`secret`/`subject_token`/`version`/`service_account`. Used in entrypoint ports, required terminal inputs, and optional terminal inputs. | Done 2026-02-17 |

---

## Completed Infrastructure (H2-H11)

| ID | Feature | Implementation | Status |
|----|---------|----------------|--------|
| H2 | Testgen dynamic targets | `iter_dag_specs()`, `#[testgen_target]` macro, 27 targets via inventory | Done |
| H3 | Makegen tool registry | `#[tool_target]` macro, inventory-driven `ToolRegistry` | Done |
| H4 | Loop extra inputs passthrough | `execute_loop_body` injects extras, DSL `with` clause parsed | Done |
| H7 | Resource abstraction trait | `Resource` trait, `AccessMode`, `ManagedResource`, capability validation | Done |
| H8 | Justfile renderer | `render_justfile()`, parity test with Makefile | Done |
| H9 | GitHub Actions renderer | DAG→`needs` mapping, YAML generation, GitLab CI too | Done |
| H11 | DAG typing hardening | `TypedPort<T>`, `TypedInput<T>`, `TypedOutput<T>`, `PortTypeTag` trait | Done |

---

## Completed Work Summary (2026-02-18)

### DynOp Type-Dispatch Elimination (T1-T8)
~5,950 lines deleted, ~300 added. `WorkspaceOp` enum, 16 `From` impls, 10 converter fns,
`FileOpsGraph<T>`, `ResolvedOp`, `RuntimeOpId` — all removed. Central `resolve.rs` replaces
hand-built graph builders for all 7 tool modules.

### Active Cleanup (C1-C6)
Resolver hardening, lowering hardening, exec-runtime literal/param source support,
makegen path regression fix, mock cleanup, transport-call consolidation.

### Wave 1 — DSL Migration & Quality
- **1A (M1-M3)**: Pragma and codegen DSL parity verified, pragma binary wired into build system
- **1B (B1-B4)**: Bridge hygiene — `Optional<T>` prefix, naming invariant test, string inspection removal
- **1C (Q1-Q10)**: 10 code quality items — panic→Err, expect→?, `ParamType` enum, `HashSet`, `&Path`, `write!()`, `Cow<'static, str>`, etc.
- **1D (S1-S4)**: Seed policy — scenario/live-flow matrices, enforcement tests, fail-closed carriers
- **1E (D1-D3)**: `StableHashOp` extraction, test redundancy review, hermeticity annotation design

### Wave 2 — System Model & Structure
- **2A (R1-R6)**: System model refactor — `Dag<TypeOp>`, TypeRegistry, contract derivation, store mapping, `PortType`, cross-provider coercion
- **2B (SD1-SD3)**: Structural derivation — inventory registries, `Box<dyn Executable>`, `From` impl elimination
- **2C (W1-W4)**: Workflow registry — `WorkflowSpec`, registration, Makefile generation, git freshness
- **2D (CQ1-CQ4)**: Codegen quality — obligation mapping, prefix-heuristic elimination, parity snapshots, CodeIR plumbing
- **2E (CT1-CT3)**: CLI contracts — `--dry-run` tests, `--print-inputs json`, testgen obligations

### Wave 3 — Domain & Consolidation
- **3A (E1-E6)**: Domain completion — scope verification, gist defaults, WIF bootstrap, infra CLI, login flow, health check
- **3B (CL1-CL3)**: Cross-language audit — generated Rust clippy clean, generated Go vet clean, IR gaps fixed
- **3C (CO1-CO7)**: Consolidation — DynOp made CO1 moot, MergeOutputs split, probe-observer bundle, seed policy IR, live-secret metadata, execution trace, ValueKind

### Wave 4+ Horizon (Completed)
- **H2-H12**: Testgen dynamic targets, makegen tool registry, loop extra inputs, Fermi guards, cardinality modeling, resource abstraction, workflow rendering (Makefile/CI), compute stack, DAG typing, integration test targets

### Pre-Sprint Tracks
- **Track A (DSL Core)**: 4-target codegen (Rust/Go/C/MIPS), exec-runtime, cross-language parity
- **Track C (Modeling)**: Type coercion, workspace model, platform, browser, transport DAG, system model
- **Track D (Logging)**: DisplayConfig, secret redaction, stderr capture, failure-first, grouped progress
- **Track B (Workflow Audit)**: Purity, resource declarations, test registry
- **P3 (ValueBacking)**: Centralized type→Value backing in core/ir
- **Architecture Debt A-C**: Infra extraction, mtime fast path, design fixes
- **25/35 hacks resolved, 18/18 consolidation items §9-15 resolved**
\n
## Archived 2026-02-20

### Sprint 2: Review Findings + Polish
| ID | Task | Status |
|----|------|--------|
| R2 | Wildcard resource semantics deferred, use coarse `file` locking | Done 2026-02-20 |
| R3 | Backend modeling enrichment RFC + IR schema update | Done 2026-02-19 |
| R4 | Go + C lowerer migration to modeled semantics | Done 2026-02-19 |
| R5 | MIPS control-flow + allocator fail-closed migration | Done 2026-02-19 |
| R6 | Holistic backend correctness harness | Done 2026-02-19 |
| R7 | Typed IAM policy domain model | Done 2026-02-19 |
| R8 | `MethodMeta` as execution source-of-truth | Done 2026-02-19 |
| R9 | Fail-closed CLI entrypoint input parsing | Done 2026-02-19 |
| R10 | Typed REST path-variable binding in `SystemModel` | Done 2026-02-19 |
| R11 | Strict platform parsing at boundaries | Done 2026-02-19 |
| R12 | Mock-default seeding by semantic kind | Done 2026-02-19 |
| P6 | `DeferredCallableOp` → per-module domain ops | Done 2026-02-20 |
| P12 | Move `resolve_infrastructure()` string-prefix matching up to lowering | Done 2026-02-20 |

### Sprint 5 & 5b: Workflow Execution Models & Minimization
| ID | Task | Status |
|----|------|--------|
| WF1-D | Workflow schema design spec | Done 2026-02-20 |
| WF1 | Minimum work-unit schema | Done 2026-02-20 |
| WF2-D | Mutual-exclusion/admission design spec | Done 2026-02-20 |
| WF2 | Mutual-exclusion claim model | Done 2026-02-20 |
| WF3-D | Key/ledger causality design spec | Done 2026-02-20 |
| WF3 | Deterministic materialization keys + miss reasons | Done 2026-02-20 |
| WF4-D | Downstream coordination design spec | Done 2026-02-20 |
| WF4 | Downstream coordination contract | Done 2026-02-20 |
| WF5 | Planner dry-run + execution plan explainability | Done 2026-02-20 |
| WF10-D | Control-token model | Done 2026-02-19 |
| WF11-D | Cached `result` persistence | Done 2026-02-19 |
| WF12-D | Changed-input routing authoritative semantics | Done 2026-02-19 |
| WF13-D | Conflict commutativity exceptions removed | Done 2026-02-19 |
| WF14-D | Compilation capability design spec | Done 2026-02-20 |
| WF15-D | Codegen capability design spec | Done 2026-02-20 |
| WF16-D | Gist base + mode capability design spec | Done 2026-02-20 |
| WF19-D | Generator + remaining tool capability design spec | Done 2026-02-20 |
| WF19 | Generator workflow capability port (bootstrap/makegen/pragma) | Done 2026-02-20 |
| WF20 | Remaining tool capability port (deps/dag-viz/dag-snapshot) | Done 2026-02-20 |
| WF21 | Makefile thinning for all tool targets | Done 2026-02-20 |
| WF22 | Capability minimization verification | Done 2026-02-20 |

### Sprint 6: Modeling Hardening
| ID | Task | Status |
|----|------|--------|
| M7-D | Secret redaction design spec | Done 2026-02-20 |
| M7 | Secret redaction by default | Done 2026-02-20 |
| M8-D | `TypeOp::Meta` design spec | Done 2026-02-20 |
| M8 | Semantically inert metadata op | Done 2026-02-20 |
| M9-D | Typed dependency marker design spec | Done 2026-02-20 |
| M9 | Typed dependency markers | Done 2026-02-20 |
| M10-D | Resource declaration + auto-wiring design spec | Done 2026-02-20 |
| M10 | Mandatory resource declarations + auto-wiring | Done 2026-02-20 |
| M11-D | Strict dry-run poisoning design spec | Done 2026-02-20 |
| M11 | Strict dry-run mode | Done 2026-02-20 |
| M15-D | Typed package-manager design spec | Done 2026-02-20 |
| M15 | Typed install planning | Done 2026-02-20 |
| M16-D | SystemModel/TransportBehavior unification design spec | Done 2026-02-20 |
| M16 | SystemModel/TransportBehavior unification | Done 2026-02-20 |
| M17-D | Global flattening + context-free identity design spec | Done 2026-02-20 |
| M17 | Global flattening + context-free identity | Done 2026-02-20 |
| M18-D | Single semantic authority/projection design spec | Done 2026-02-20 |
| M18 | Projection-only surfaces + drift enforcement | Done 2026-02-20 |
| M19-D | Formal non-redundancy proof design spec | Done 2026-02-20 |
| M19 | Formal non-redundancy proof harness | Done 2026-02-20 |

### Sprint 7: End-to-End Service Codegen from DSL
| ID | Task | Status |
|----|------|--------|
| SC1 | `ServiceOperationSpec` in the IR | Done 2026-02-20 |
| SC2 | Generic protocol interpreters (Rust exec-runtime) | Done 2026-02-20 |
| SC3 | Switch resolver + delete per-service Rust | Done 2026-02-20 |
| SC4 | LLM provider service definitions | Done 2026-02-20 |
| SC5 | Multi-language service emission (Go) | Done 2026-02-20 |
| SC6 | Multi-language service emission (C + MIPS) | Done 2026-02-20 |
| SC7 | New service smoke test (all languages) | Done 2026-02-20 |

## Archived 2026-02-21

### Workflow Planner Cutover + Universal/Gist Capabilities
| ID | Task | Status |
|----|------|--------|
| WF6 | Port `ci` to workflow planner (`gunbc-workflow ci`) | Done 2026-02-21 |
| WF7 | Port `test-all` to workflow planner (`gunbc-workflow test-all`) | Done 2026-02-21 |
| WF8 | Makefile thinning + strict cutover for `ci`/`test-all` | Done 2026-02-21 |
| WF9 | Latency SLO instrumentation + guardrails for planner workflows | Done 2026-02-21 |
| WF14 | Universal compilation capability implementation | Done 2026-02-21 |
| WF15 | Universal codegen capability implementation | Done 2026-02-21 |
| WF16 | Base gist workflow + snapshot mode | Done 2026-02-21 |
| WF17 | Gist diff mode augment over base workflow | Done 2026-02-21 |
| WF18 | Gist recent mode augment over base workflow | Done 2026-02-21 |

### Daglang CLI Hardening
| ID | Task | Status |
|----|------|--------|
| DL1 | `normalize_path_components` root-clamping fix | Done 2026-02-21 |
| DL2 | Parse-stop diagnostics normalization | Done 2026-02-21 |
| DL3 | Pipeline DAG/toposort cleanup in daglang CLI | Done 2026-02-21 |
| DL4 | Unified/explicit `.dag` directory behavior | Done 2026-02-21 |

### Dev Pipeline / Review
| ID | Task | Status |
|----|------|--------|
| W1 | `gunbc review` CLI binary entrypoint | Done 2026-02-21 |
| W4 | Abstract 4-dimension review DAG | Done 2026-02-21 |
| W5 | Coding review profile (`AGENT.md` + `clippy.toml` criteria loading) | Done 2026-02-21 |
| W6 | CI status injection into review context | Done 2026-02-21 |
| W7 | `gunbc pipeline` daily orchestration command | Done 2026-02-21 |
| W8 | Pipeline issue-context integration (`--issue`) | Done 2026-02-21 |

---

## Lane A: SDLC Delivery Lane (Complete 2026-02-21)

| ID | Task | Status |
|----|------|--------|
| MD0-D | SDLC mega modeling design gate: consolidated abstractions/invariants/layers/conformance into `docs/design/sdlc/mega-modeling-design.md` | Done 2026-02-21 |
| IM0-D | SDLC issue abstraction modeling: provider-agnostic issue contracts, adapter boundaries, idempotency keys, upsert protocols | Done 2026-02-21 |
| IM1 | Intent sheet contract: `intent_id`, objective, success criteria, constraints schema with template | Done 2026-02-21 |
| IM2 | Issue intake upsert flow: create-or-update one canonical issue per `intent_id` | Done 2026-02-21 |
| IM3 | Stage idempotency + resume keying: run/stage keys with duplicate-effect skip on replay | Done 2026-02-21 |
| IM4 | Idempotent remote update protocol: deterministic marker upsert + CAS transitions | Done 2026-02-21 |
| IM5 | Commit/update trace linkage: branch/commit metadata linked to `intent_id`/`issue_id`/run key | Done 2026-02-21 |
| IM6 | Claim/lease abstraction: atomic claim with lease expiry and heartbeat semantics | Done 2026-02-21 |
| IM7 | Async control loop: discover → claim → execute → release with bounded retries/backoff | Done 2026-02-21 |
| IM8 | Stage transaction executor: fixed step ordering with crash-safe replay | Done 2026-02-21 |
| IM9 | Failure taxonomy + retry policy: typed failure classes, persisted retry budget, terminal fail-closed | Done 2026-02-21 |
| IM10 | Intake conflict policy: fail-closed multi-match collision handling | Done 2026-02-21 |
| IM11 | Replay reconciliation loop: crash-window convergence, stale marker cleanup by lease generation | Done 2026-02-21 |
| IM12 | Provider capability gate: `ManagedIssueSearch` + `DeterministicIssueIdentity` checks, dry-run bypass | Done 2026-02-21 |
| IM13 | Artifact payload/reference contract: `Inline` vs `BlobRef`, content-hash equality, canonical markers | Done 2026-02-21 |
| W9 | GitHub Issues transport + adapter: provider-agnostic `TrackedIssue` mapping | Done 2026-02-21 |
| W10 | DesignOps module: `PrepareDesignPrompt` + `ParseDesignResponse` typed artifacts | Done 2026-02-21 |
| W11 | SDLC resolver wiring: pipeline→resolver→execution with typed transport + design ops | Done 2026-02-21 |
| W12 | `gunbc-sdlc` CLI binary: intake/worker dry-run + real mode with retry/reconciliation | Done 2026-02-21 |
| W13 | Approval gates: `AwaitApproval` yield semantics with claim release and rediscovery resume | Done 2026-02-21 |
| W14 | Metrics/monitoring: stage duration, LLM cost, approval latency in execution report | Done 2026-02-21 |

---

## Lane B: Review Credential Certification (Partial — 2026-02-21)

| ID | Task | Status |
|----|------|--------|
| W3 | Multi-provider operational verification: `--provider openai` and `--provider anthropic` with fail-closed cred errors | Done 2026-02-21 |

`W2` remains open (requires real-mode API key smoke test).

---

## Lane C: Planner/CI Additional (Complete 2026-02-21)

| ID | Task | Status |
|----|------|--------|
| AX1 | Bootstrap invariant CI gate: bootstrap-safe binaries compile without generated sources | Done 2026-02-21 |
| AX2 | Registry coupling hardening: contract-tested coupling between `default_registry()` and `derive_tool_defs()` | Done 2026-02-21 |

---

## Lane D: Daglang Convergence (Complete 2026-02-21)

| ID | Task | Status |
|----|------|--------|
| DL5 | Unify compile/pipeline overlap: shared `compile_from_module_graph_with_options` path, pipeline handles discovery | Done 2026-02-21 |
| DL6 | Manifest semantics clarity: split `dag manifest` into `dag progress` + `dag topology` (no flags, intent from command name) | Done 2026-02-21 |
| DL7 | Canonical IR CLI surface: `daglang compile --format canonical-json` with deterministic output + tests | Done 2026-02-21 |
| DL8 | Viz default decision: ASCII locked as default for `daglang viz`, documented + test-locked | Done 2026-02-21 |

---

## Lane E: Runtime Infra/Control-Plane (Complete 2026-02-21)

| ID | Task | Status |
|----|------|--------|
| IN0-D | Runtime/infra control-plane modeling: stateless worker topology, trigger/signal matrix, startup/drain semantics | Done 2026-02-21 |
| IN1 | Infra intent contract: versioned `InfraIntent` schema for runtime dependencies | Done 2026-02-21 |
| IN2 | Infra plan/apply coverage: drift-aware reconciliation contracts for SDLC runtime | Done 2026-02-21 |
| IN3 | Worker startup preflight gate: fail-closed readiness checks for infra components | Done 2026-02-21 |
| IN4 | Stateless deployment profile + drain: launch profile contracts, graceful drain/restart | Done 2026-02-21 |

---

## Lane F: Codegen-First SDLC (Complete 2026-02-21)

| ID | Task | Status |
|----|------|--------|
| CG0-D | Codegen-first architecture modeling: DSL-authored behavior compiled to Rust/Go/C boundary locked | Done 2026-02-21 |
| CG1 | Canonicalize SDLC DSL modules: **superseded** — `dsl/pipelines/sdlc.dag` and `dsl/tools/design.dag` removed; SDLC modules are runtime-authored, not DSL-discovered | Superseded 2026-02-21 |
| CG2 | Discovery-to-execution cutover: generic runtime wiring for discovered DSL modules | Done 2026-02-21 |
| CG3 | Control-plane DSL resources/services: claim lease store + stage outcome ledger as DSL interfaces | Done 2026-02-21 |
| CG4 | Infra intent reconcile in DSL: plan/apply/reconcile via compiled DSL orchestration | Done 2026-02-21 |
| CG5 | Generated target entrypoints (Rust/Go/C): runnable SDLC worker/infra entrypoints with C adapter boundary | Done 2026-02-21 |
| CG6 | Multi-level conformance + backend rotation harness: layered conformance suites, C sanitizer coverage, backend rotation | Done 2026-02-21 |

---

## Cleanup (2026-02-21)

| ID | Task | Status |
|----|------|--------|
| — | Remove `c_sanitizer_runtime_available` dead code in `codegen_parity.rs` | Done 2026-02-21 |
| — | Remove `tools.design` and `pipelines.sdlc` resolver registrations + dependent code after `.dag` file deletion | Done 2026-02-21 |
| — | Bulk rename `manifest`/`progress-manifest` → `progress`/`topology` across CLI, tests, docs | Done 2026-02-21 |

---

## Lane B: Review Credential (2026-02-21)

| ID | Task | Status |
|----|------|--------|
| W2 | Credential smoke test: run `gunbc-review` in real mode using `ANTHROPIC_API_KEY` against a small diff and verify structured findings output. | Done 2026-02-21 |

---

## Sprint 10: Autonomous Implementation & Agent Integration (2026-02-21)

### Phase 1: Implementation Handoff & GitOps

| ID | Task | Status |
|----|------|--------|
| AI1 | Handoff Contract: Typed `HandoffSpec`, `DesignArtifact`, `AgentConstraints` in `core/ir/src/transport/agent.rs`; `AgentAdapter` trait + `StubAgentAdapter` in `agent_adapter.rs`. | Done 2026-02-21 |
| AI2 | Agent Workspace Bootstrap: `gunbc-sdlc agent-spawn --intake-key` reads intake/artifact ledgers, validates Accepted stage, assembles `HandoffSpec`, dispatches to adapter, records in `agent-ledger.json`. | Done 2026-02-21 |
| AI3 | PR Automation: `PullRequestSpec` types + `gh pr create/comment/merge` request builders in `core/ir/src/transport/github/pull_request.rs`. | Done 2026-02-21 |

### Phase 2: SDLC Pull Request Validation

| ID | Task | Status |
|----|------|--------|
| PR1 | Diff Review Integration: `gunbc-sdlc validate-pr` runs diff review against PR branch, posts findings as PR comments. | Done 2026-02-21 |
| PR2 | CI/CD Aggregation: `validate-pr` runs `cargo test` + `cargo clippy` and aggregates pass/fail results. | Done 2026-02-21 |
| PR3 | Close Loop: `validate-pr` auto-transitions `Implementation` -> `Closed` when all checks pass, posts summary comment. | Done 2026-02-21 |
