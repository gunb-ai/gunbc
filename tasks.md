# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Operating Model: Blue Team / Red Team

Two teams, two lanes each, never blocking each other.

```
  BLUE TEAM — Advance                        RED TEAM — Harden
  ─────────────────────────                  ────────────────────────
  Lane B1: SDLC Pipeline                    Lane R1: Structural Correctness
    RF-B1 → SDLC-1 → SDLC-2 →               RF-H1 → RF-H4 → RF-H2 →
    SDLC-3 → SDLC-4 → ─┐                    RF-H3 → RF-G-unblock →
                         ├→ SDLC-7 → 8        RF-A1 → RF-A2 → RF-A4
  Lane B2: SDLC Infra   │
    RF-B2 → SDLC-5 → ──┘                   Lane R2: Testing + Foundation
    SDLC-6 → ──────────┘                     BB-2 → BB-3 → BB-5 →
                                              FC-P7-c2 → FC-P7-d →
  ─ then ─                                    FC-CF5 → FC-CF6 →
  Lane B1: Cloud + Scale                      FC-P8-a → FC-P8-b → FC-P8-c
    SDLC-CD1:6 → DG1
  Lane B2: Agent Integration                ─ then ─
    SDLC-AG1:3 → webhook-driven             Lane R1: Typed Dispatch
    stage transitions                         RF-A5 → RF-A6 → RF-A8
                                            Lane R2: Code Hygiene
                                              RF-C1 → RF-C2 → RF-D eval
```

### Protocols

**Independence**: Lanes within a team touch different files. No merge
conflicts between lanes. Each lane can be worked by a separate agent.

**Scouting**: Every blue team PR includes a `Scouted:` line listing
red team opportunities discovered during implementation. Examples:
"string dispatch in catalog.rs" → RF-A10, "unwrap_or in provider
wiring" → new RF-H item. Red team triages and queues.

**Refill**: When a lane has <3 pending items, the worker proposes
new items from codebase observation or horizon scanning. Anemic
queues are a bug — the point is to never run out of work.

**Non-blocking**: Red never blocks blue. If red team cleanup is
needed for blue team progress, blue does the minimum fix inline
and red cleans up later.

---

# BLUE TEAM — Advance

## Lane B1: SDLC Pipeline

Registration, dispatch, validation, stage handlers. Touches
`gunbc-dag/src/workflow/`, `dsl/sdlc/`, `dsl/services/github/`.

### Transport Declarations — Active Services (RF-B1)

| ID | Scope | Ops Missing | Status | Notes |
|----|-------|-------------|--------|-------|
| RF-B1 | github/issues.dag (8), github/pull_request.dag (6), llm/openai.dag (2) | 16 | Pending | REST transport; need `config { endpoint, auth }` |

### Pipeline Activation

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| SDLC-1 | Register SDLC in workflow catalog + WorkspaceBinary dispatch. | M | Pending | — |
| SDLC-2 | Fill dispatch runtime: real stage transition logic via state machine. | M | Pending | SDLC-1 |
| SDLC-3 | Fill validation runtime: review_gate, ci_gate with real logic. | M | Pending | SDLC-2 |
| SDLC-4 | Complete testing→done handler (cargo test + clippy + conditional merge). | M | Pending | SDLC-1 |

### Convergence (needs both lanes)

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| SDLC-7 | Profile binding verification: compile all 3 profiles, hermetic e2e on unit_test. | M | Pending | SDLC-1:6 |
| SDLC-8 | Local profile e2e: real GitHub repo, idea → design → review flow. | L | Pending | SDLC-7 |

**Deliverable**: `gunbc sdlc --profile local --repo owner/name`

### B1 Horizon (after SDLC-8)

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| SDLC-CD1 | GCS SignalStore (PubSub-backed, at-least-once). | M | Pending | SDLC-8 |
| SDLC-CD2 | GCS ArtifactStore (GCS-backed, generation CAS). | M | Pending | SDLC-8 |
| SDLC-CD3 | GCP credential chaining (WIF OIDC exchange). | L | Pending | SDLC-8 |
| SDLC-CD4 | Cloud Run deployment DAG. | L | Pending | SDLC-CD1:3 |
| SDLC-CD5 | Multi-worker CAS stress test (3 workers, exactly-once). | M | Pending | SDLC-CD4 |
| SDLC-CD6 | CI integration (hermetic + cloud smoke). | M | Pending | SDLC-CD5 |
| DG1 | Daggen: re-enable `needs_daggen()` for dynamic DAG generation from git diffs. | L | Pending | SDLC-CD6 |

---

## Lane B2: SDLC Infrastructure

Providers, stores, resource implementations. Touches
`dsl/sdlc/providers/`, `gunbc-dag/src/sdlc/`, new provider crates.
**Independent from Lane B1** — can be built in parallel.

### Transport Declarations — Providers (RF-B2)

| ID | Scope | Ops Missing | Status | Notes |
|----|-------|-------------|--------|-------|
| RF-B2 | file stores (6), GCS stores (6), github_issue_provider (7), codex_agent (4), credential providers (4) | 27 | Pending | Mixed rest/shell/file/local |

### Provider Activation

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| SDLC-5 | Local SignalStore provider (file-based, satisfies signal_store.dag contracts). | M | Pending | — |
| SDLC-6 | Local ArtifactStore provider (file-based, content-hash keyed, two-phase commit). | M | Pending | — |

### B2 Horizon (after stores + convergence)

| ID | Task | Size | Status | Deps |
|----|------|------|--------|------|
| SDLC-AG1 | Agent provider: wire codex_agent.dag to LLM service for automated code review in review stage. | M | Pending | SDLC-8 |
| SDLC-AG2 | Credential provider: local keychain integration for GITHUB_TOKEN + LLM API keys. | M | Pending | SDLC-8 |
| SDLC-AG3 | Webhook-driven stage transitions: GitHub webhook → local listener → stage advance. | L | Pending | SDLC-AG1 |

---

## Blue Backlog

Not scheduled. Promote when horizon items are exhausted.

| ID | Item | Size | Priority | Notes |
|----|------|------|----------|-------|
| H10 | Compute stack orchestration: Cloud Run/GCS/LB lifecycle DAG builder. | L | P2 | `docs/design/horizon/h10-compute-stack-services.md` |
| S12-E | Multi-worker CAS: GcsClaimStore with generation-based CAS. DSL exists. | M | P2 | Deferred until cloud_run profile needed |
| H1 | Display reactive DSL: channel-driven event loop. | XL | P3 | No current use case. Review 2026-Q3, delete if not promoted. |

---

# RED TEAM — Harden

## Lane R1: Structural Correctness

Types over validation, enums over strings. Touches `core/ir/`,
`core/test/`, `gunbc-dag/src/fidelity.rs`, `gunbc-dag/src/resolve.rs`.
**No overlap with Lane R2 files.**

Queue ordered by danger (silent failures first, then mechanical cleanup):

| Order | ID | What | Size | Status | Deps |
|-------|----|------|------|--------|------|
| 1 | RF-H1 | Fidelity silent fallbacks → `.expect()` (short-term) or RF-G unblock (permanent). | S | Pending | — |
| 2 | RF-H4 | ResourceKind string dispatch → enum. Easiest win, local to resolve.rs. | S | Pending | — |
| 3 | RF-H2 | TestgenTargetDef Option fields → non-Option with defaults. | S | Pending | — |
| 4 | RF-H3 | TestClass/FermiCost parse → `FromStr` with `Result`. Overlaps RF-A8. | S | Pending | — |
| 5 | RF-E4 | Fidelity classification smoke test. Assert makegen→Hermetic/S, gist→Integration/L. | S | Pending | — |
| 6 | RF-G-unblock | `fold` extraction in evaluate_fn_body() — enables calling DSL classify_transports(). Deletes all RF-G1:6 shadows. | M | Pending | — |
| 7 | RF-A1 | NodeKind required on Node\<T\>. Remove Option, require in builders. | M | Pending | — |
| 8 | RF-A2 | Port namespace typing. PortCategory enum + methods on PortName. | M | Pending | — |
| 9 | RF-A4 | CallableClass enum in resolve.rs. Parse once, dispatch everywhere. | M | Pending | — |

### R1 Horizon (after A4)

| Order | ID | What | Size |
|-------|----|------|------|
| 10 | RF-A5 | TransportNodeKind enum (Prepare/Execute/Parse). | S |
| 11 | RF-A6 | String constants consolidation (__deps, __out:, res:file, tool:). | M |
| 12 | RF-A8 | `#[derive(StringEnum)]` macro for 15 enums, ~60 match blocks. | M |
| 13 | RF-A3 | ModulePath unification across 4 crates. | S |
| 14 | RF-A9 | Shared DslTypeMapping table for emit backends. | S |
| 15 | RF-A10 | Registry pattern for DAG tooling string dispatch (100+ arms). | L |

---

## Lane R2: Testing + Foundation

Black-box test generation, then extern bridge elimination. Touches
`core/codegen/src/testgen/`, `core/daglang/`, `gunbc-dag/src/extern_impls.rs`.
**No overlap with Lane R1 files.**

Queue ordered by dependency chain:

| Order | ID | What | Size | Status | Deps |
|-------|----|------|------|--------|------|
| 1 | BB-2 | Per-node test generation (Level 1a/1b). Pure nodes real exec, effectful DryRun. | M | Pending | BB-1 |
| 2 | BB-3 | Adjacent pair test generation (Level 2). Window tests for wiring bugs. | M | Pending | BB-2 |
| 3 | BB-5 | Cross-workflow consistency tests (Level 4). Same node, multiple workflows. | S | Pending | BB-2 |
| 4 | FC-P7-c2 | DSL Makefile assembly: import data, produce targets, wire to makegen output. | M | Pending | — |
| 5 | FC-P7-d | Delete 2 bootstrap extern impls. Parity golden tests. | M | Pending | FC-P7-c2 |
| 6 | FC-CF5 | Recursive types (self-referential type defs). | L | Pending | — |
| 7 | FC-CF6 | Recursive functions (self-calls in fn bodies). | L | Pending | FC-CF5 |
| 8 | FC-P8-a | Tree rendering in pure DSL. Delete RenderTreeOp. | L | Pending | FC-CF5, FC-CF6 |
| 9 | FC-P8-b | Snapshot content as MarkdownDoc. Delete BuildSnapshotContentOp. | M | Pending | FC-P8-a |
| 10 | FC-P8-c | Delete extern_impls.rs entirely. Zero extern func in any .dag file. | S | Pending | FC-P8-a, FC-P8-b |

### R2 Horizon (after FC-P8-c)

| Order | ID | What | Size |
|-------|----|------|------|
| 11 | RF-C1 | Split monolithic files (lower 11K, typecheck 5K, execute 4K). | L |
| 12 | RF-C2 | Unify passthrough op variants to single data-driven PassthroughOp. | S |
| 13 | RF-C3 | Error type consolidation (6 types → layered like ExecError). | M |
| 14 | RF-C4 | Test helper extraction (CompileTestHelper + MockFactory). | M |
| 15 | RF-D-eval | Scaffolding decision: delete RetryPolicy/ErrorMapping/ContractObligation/ResourceRequirement or wire through DSL. | S |
| 16 | RF-F-eval | Underused abstractions decision: delete algebra traits + render traits or find second consumer. | S |

---

## Red Backlog

Tracked for completeness. Not in any lane queue. Promote on discovery
of concrete business case or when horizon items are exhausted.

### Theme B: Remaining Transport Gaps (non-blocking)

| ID | Scope | Ops Missing | Notes |
|----|-------|-------------|-------|
| RF-B3 | **Stub providers**: stub_providers.dag (26), stub_credential_provider.dag (2) | 28 | Intentional — unit_test profile stubs. Consider `transport stub {}` marker. |
| RF-B4 | **Infrastructure stubs**: azure (43), aws (38), gcp-infra (59) | 140 | Dormant — defer until infrastructure provisioning lane opens. |

### Theme E: Pre-existing Test Gaps (non-fidelity)

| ID | Test | Root Cause |
|----|------|-----------|
| RF-E1 | `registry_exposes_required_claims` | Process unit "ci.build_compile" doesn't have `file:target` Write claim. |
| RF-E2 | `makegen_exec_runtime_e2e` (ignored) | Exec-runtime emit missing `LoadRegistry` handler + PureRender fn classification. |
| RF-E3 | `pragma_exec_runtime_e2e` (ignored) | `ContentUpsertOutputPath` nodes unclassified for exec-runtime emit. |

### Compiler Features (low priority)

| ID | Feature | Size | Notes |
|----|---------|------|-------|
| FC-CF2 | `skip(n)`: List\<T\> → List\<T\> | S | Expressible via fold+index. |
| FC-CF3 | `enumerate()`: List\<T\> → List\<(Int, T)\> | M | Expressible via fold+counter. |

---

## Reference: Theme Details

Detailed descriptions for items in lane queues. Consult when picking
up a task — the lane queue has the priority order, this section has
the context.

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

**RF-G-unblock** (in R1 queue position 6): Implement `fold` aggregation
in Rust's `evaluate_fn_body()` evaluator (it already handles other
collection ops). This directly enables calling the existing DSL fns.
Once done, delete:
- `transport_depth_ordinal()`, `transport_depth_str()` (RF-G1)
- `transport_is_hermetic()` (RF-G2)
- `classify_callable()` pre-aggregation (RF-G3)
- `test_policy.dag::classify_from_facts()` (RF-G4)
- `TestClass::parse()` / `FermiCost::parse()` round-trip (RF-G5)
- `test_policy.dag` shadow fns (RF-G6)

### Theme H: Structural Enforcement (parse, don't validate)

| ID | What | Current | Structural Fix |
|----|------|---------|---------------|
| RF-H1 | **Fidelity silent fallbacks**. `classify_callable()` parses DSL output strings into enums with `unwrap_or(Unit)` / `unwrap_or(XS)`. If DSL returns a typo or `None`, classification silently downgrades to Unit — tests are skipped at runtime with no error. | `TestClass::parse(str).unwrap_or(Unit)` in fidelity.rs:129 | **Eliminated by RF-G unblock**: `classify_transports()` returns typed `DerivedClassification`, no string round-trip. Short-term: change `unwrap_or` to `.expect("DSL classify_from_facts returned invalid tier")`. |
| RF-H2 | **TestgenTargetDef Option fields always populated**. `test_class: Option<TestClass>`, `fermi_cost: Option<FermiCost>`, `requires: Option<Vec<String>>` — every auto-testgen call site now fills `Some(...)` from fidelity. The `Option` only exists for legacy `DagSpecDef` path (which also never overrides). `generate_target_with_types()` does `unwrap_or(Unit)` on every field. | 6 Option fields in registry.rs | Make fields non-Option with `Default` impl. Callers construct with values; no unwrapping. `DagSpecDef.to_def()` fills from fidelity instead of leaving `None`. |
| RF-H3 | **TestClass/FermiCost parse returns Option not Result**. `parse(&str) -> Option<Self>` loses the invalid input. Callers chain `.unwrap_or(default)` which masks the error entirely. | fermi.rs:25-31, 56-64 | `FromStr` with `Result<Self, ParseError>`. Callers use `?` to propagate. Overlaps RF-A8 (derive macro). |
| RF-H4 | **ResourceKind string dispatch**. `ResourceAcquireOp { resource_kind: String }` matched at runtime. Unknown kinds fall through to `Value::Str("resource:{other}")` — wrong type, silent. | resolve.rs:365-386 | `ResourceKind` enum parsed once at resolve time. Match is exhaustive, no fallback arm. Overlaps RF-A4. |
| RF-H5 | **Port namespace string conventions**. ~30 `starts_with("res:")` / `"tool:"` / `"__deps"` checks across 8+ files. | Scattered | Already RF-A2 (`PortCategory` enum + methods on `PortName`). Listed for completeness. |

### Theme A: Typed Dispatch (full detail)

| ID | Pattern | Key Files | Notes |
|----|---------|-----------|-------|
| RF-A1 | **NodeKind on Node\<T\>**. `validate_node_kinds_for_interception()` is a runtime check that rejects `kind: None` nodes. Target: `Node::opaque()` requires `NodeKind`, eliminating `Option` and runtime check. | node.rs, execute.rs | Remove `Option<NodeKind>`, delete validation fn. |
| RF-A2 | **Port namespace typing**. 18+ `starts_with("res:")` / `"tool:"` / `"__out:"` checks. `is_user_param_port()` reimplemented 3×. | 6 files | `PortCategory` enum + methods on `PortName`. |
| RF-A3 | **Module path representation**. 4 crates use `Vec<String>` vs typed `ModulePath`. | 4 crates | Unify on `ModulePath`; add `From` impls. |
| RF-A4 | **Stringly-typed dispatch in resolve.rs**. 10+ string prefix matches for module/callable routing. | resolve.rs | `CallableClass` enum parsed once. |
| RF-A5 | **Transport node classification**. String-based prepare/execute/parse detection. | resolve.rs | `TransportNodeKind { Prepare, Execute, Parse }`. |
| RF-A6 | **String constants consolidation**. `"__deps"` (45×), `"__out:"` (6×), `"res:file"` (8×), `"tool:"` (15×). | 74+ sites | Central consts in `core/ir/src/signature.rs`. |
| RF-A7 | **Transport class heuristic shadows**. Rust replicas of DSL fns. | fidelity.rs | Delete via RF-G-unblock. |
| RF-A8 | **`as_str`/`parse` boilerplate**. 15 enums × 4 methods = ~60 match blocks. | 12 files | `#[derive(StringEnum)]` macro. |
| RF-A9 | **Emit backend type-name tables**. Same type mapping in 3 backends. | daglang-emit | Shared `DslTypeMapping` table. |
| RF-A10 | **String dispatch in DAG tooling**. 100+ match arms on string literals. | 5 files | Registry pattern or DSL data declarations. |

---

## Archive

NF-1 through NF-6 (compile+link hardening): complete 2026-02-25. Detail: `TODO/TODONE/tasks-completed.md`.
FC-NF7 (fn-level evaluation): complete 2026-02-25. `expr.rs` IR + `eval.rs` evaluator + `FnBodyDelegate`. `makegen/render.rs` deleted (~1200 lines). Makegen rendering is pure DSL.
FC-CL (dead code cleanup): complete 2026-02-25. Deleted `core/tool-registry` + `core/tool-registry-macros`, 14 orphaned spec builder fns, stale rules/comments.
FC-EG (enforcement gates): complete 2026-02-25. Import-direction lint, extern func count gate, format!/push_str boundary gate — all 3 automated ratchets in CI.
FC-P6-a:d (policy migration): complete 2026-02-26. `dsl_render.rs` evaluates `derive_*` DSL fns via `evaluate_fn_body()`. Allowlist + lint_policy migrated; clippy_toml blocked on FC-CF5. `all_extern_symbols()` 8→6.
FC-CF1 + FC-CF7 (split + zip): complete 2026-02-26. Both pipe methods across 4 compiler stages (typecheck, lower, eval, emit). 9 e2e tests.
FC-P7-a (build_workflows.dag): complete 2026-02-26. WorkflowSpec + MetaTarget types + data.
FC-P7-b (artifact emitter): complete 2026-02-26. `dag_emit.rs` emits valid `.dag` syntax. `CompileOutput::emit_artifact_dag()` for downstream introspection.
FC-P7-c1 (Makefile DSL types): complete 2026-02-26. Types already existed in `extdeps/make.dag`.
BB-0 (compositional type modeling): complete. All types in `core/test/src/corpus.rs` and `core/test/src/fidelity.rs`. 23 integration tests.
BB-1 (mock corpus builder): complete. `build_corpus()` in `core/codegen/src/testgen/mock_corpus.rs`. DryRun extraction + cross-workflow accumulation.
BB-4 (type-derived boundary values): complete. `enrich_corpus_with_type_witnesses()` with anchored mutation. MAX_EXAMPLES_PER_NODE=50.
BB-6 (transport fidelity ladders): complete. Canonical ladders for all 6 TransportKind variants. `node_max_fidelity()` transitive meet inference.
