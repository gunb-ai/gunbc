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
    RF-B1 → SDLC-1 → SDLC-2 →               RF-H4 → RF-H2 → RF-E4 →
    SDLC-3 → SDLC-4 → ─┐                    RF-G-unblock → RF-A1 →
                         ├→ SDLC-7 → 8        RF-A2a → RF-A4
  Lane B2: SDLC Infra   │
    RF-B2 → SDLC-5 → ──┘                   Lane R2: Testing + Foundation
    SDLC-6 → ──────────┘                     BB-2 → BB-3 → BB-5 →
                                              FC-P7-c2 → FC-P7-d →
  ─ then ─                                    FC-CF5 → FC-CF6 →
  Lane B1: Cloud + Scale                      FC-P8-a → FC-P8-b → FC-P8-c
    SDLC-CD1:6 → DG1
  Lane B2: Agent Integration                ─ then ─
    SDLC-AG1:3 → webhook-driven             Lane R1: Typed Dispatch
    stage transitions                         RF-A5 → RF-A6a → RF-A8
                                            Lane R2: Code Hygiene
                                              RF-A2b → RF-A6b → RF-C1 →
                                              RF-C2 → RF-D eval
```

### Protocols

**Independence**: Lanes within a team touch different files. No merge
conflicts between lanes. Each lane can be worked by a separate agent.

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
| SDLC-CD5 | Multi-worker CAS stress test (3 workers, exactly-once). Tests SDLC-CD1/CD2 stores, not S12-E. | M | Pending | SDLC-CD4 |
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

## Blue Unqueued

Raw observations from any worker. Not triaged, not sized.
Blue team promotes to backlog or lane queues during triage.

| Observation | Source | Date |
|-------------|--------|------|
| *(empty — add observations here)* | | |

---

## Blue Backlog

Triaged and sized. Promote to lane queues when horizon items are exhausted.

| ID | Item | Size | Priority | Notes |
|----|------|------|----------|-------|
| H10 | Compute stack orchestration: Cloud Run/GCS/LB lifecycle DAG builder. | L | P2 | `docs/design/horizon/h10-compute-stack-services.md` |
| S12-E | Multi-worker CAS: GcsClaimStore with generation-based CAS. DSL exists. Distinct from SDLC-CD5 (which stress-tests SignalStore/ArtifactStore). | M | P2 | Deferred until cloud_run profile needed |
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

### Remediation Ladder

When you find a smell, apply the **highest rung** that's feasible:

1. **Eliminate the representation** — the bad state can't be constructed
   (e.g., `NodeKind` required in constructor → no `Option` exists)
2. **Parse at the boundary** — raw input becomes a typed value once,
   all downstream code receives the type (e.g., `ResourceKind` enum
   parsed in resolve step, never a `String` again)
3. **Derive from source of truth** — delete the hand-maintained copy,
   generate from DSL/enum/data declaration (e.g., RF-G-unblock deletes
   6 Rust heuristic fns by calling DSL `classify_transports`)
4. **Centralize** — if elimination isn't possible yet, at least have
   one canonical impl that all call sites use (e.g., `PortCategory`
   enum + methods, used everywhere instead of ad-hoc `starts_with`)

Rung 4 is a **waypoint**, not a destination. If you centralize, file
a follow-up to eliminate.

---

## Lane R1: Structural Correctness

Types over validation, enums over strings. Touches `core/ir/`,
`core/test/`, `gunbc-dag/src/fidelity.rs`, `gunbc-dag/src/resolve.rs`.
**No overlap with Lane R2 files.** (RF-A2/RF-A6 split: R1 defines
types in `core/ir/`, R2 migrates call sites in `core/daglang/` + `core/codegen/`.)

Queue ordered by danger (silent failures first, then mechanical cleanup):

| Order | ID | What | Size | Status | Deps |
|-------|----|------|------|--------|------|
| 1 | RF-H4 | ResourceKind string dispatch → enum. Easiest win, local to resolve.rs. | S | **Done** | — |
| 2 | RF-H2 | TestgenTargetDef Option fields → non-Option with defaults. | S | **Done** | — |
| 3 | RF-E4 | Fidelity classification smoke test. makegen callable→Unit/XS, gist module→Integration/L. | S | **Done** | — |
| 4 | RF-G-unblock | `fold` extraction in evaluate_fn_body() — enables calling DSL classify_transports(). Deletes all RF-G1:6 shadows + fidelity silent fallbacks (was RF-H1). | M | Pending | — |
| 5 | RF-A1 | NodeKind required on Node\<T\>. Remove Option, require in builders. | M | Pending | — |
| 6 | RF-A2a | Port namespace typing: define `PortCategory` enum + methods on `PortName` in `core/ir/`. R1-scoped (definition only). | M | Pending | — |
| 7 | RF-A4 | CallableClass enum in resolve.rs. Parse once, dispatch everywhere. | M | Pending | — |

### R1 Horizon (after A4)

| Order | ID | What | Size |
|-------|----|------|------|
| 8 | RF-A5 | TransportNodeKind enum (Prepare/Execute/Parse). | S |
| 9 | RF-A6a | String constants: define central consts in `core/ir/src/signature.rs`. R1-scoped (definition only). | S |
| 10 | RF-A8 | `#[derive(StringEnum)]` macro for 15 enums (~60 match blocks). Includes TestClass/FermiCost `FromStr` (was RF-H3). | M |
| 11 | RF-A3 | ModulePath unification across 4 crates. | S |
| 12 | RF-A9 | Shared DslTypeMapping table for emit backends. | S |
| 13 | RF-A10 | Registry pattern for DAG tooling string dispatch (100+ arms). | L |

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
| 11 | RF-A2b | Port namespace typing: migrate `starts_with("res:")`/`"tool:"` call sites in `core/daglang/`, `core/codegen/testgen/` to `PortCategory`. | S |
| 12 | RF-A6b | String constants: migrate `__deps`/`res:file`/`tool:` references in `core/daglang/`, `core/codegen/testgen/` to central consts. | M |
| 13 | RF-C1 | Split monolithic files (lower 11K, typecheck 5K, execute 4K). | L |
| 14 | RF-C2 | Unify passthrough op variants to single data-driven PassthroughOp. | S |
| 15 | RF-C3 | Error type consolidation (6 types → layered like ExecError). | M |
| 16 | RF-C4 | Test helper extraction (CompileTestHelper + MockFactory). | M |
| 17 | RF-D-eval | Scaffolding decision: delete RetryPolicy/ErrorMapping/ContractObligation/ResourceRequirement or wire through DSL. | S |
| 18 | RF-F-eval | Underused abstractions decision: delete algebra traits + render traits or find second consumer. | S |

---

## Red Unqueued

Raw observations from any worker. Not triaged, not sized. Use the
smell catalog above to classify. Include file path + line if possible.

| Smell | Observation | File | Source | Date |
|-------|-------------|------|--------|------|
| Static mapping table | Three functions (`transport_depth_ordinal`, `transport_depth_str`, `transport_is_hermetic`) encode the same semantic mapping for `ServiceTransportClass`. Adding a variant requires updating all three. Consolidate into a single `TransportClassMetadata` struct or const array. | `gunbc-dag/src/fidelity.rs:60-89` | RF-H4 PR scout | 2026-02-26 |
| Heuristic reimplementation | `passthrough_fallback_value()` hard-codes a port alias table (`"result"→["input","value","content",...]`, `"return"→[11 aliases]`) to guess output port mappings. DSL callables should declare output port names explicitly; this table is a wiring-ambiguity bandaid. | `gunbc-dag/src/resolve.rs:95-162` | RF-H4 PR scout | 2026-02-26 |
| Heuristic reimplementation | `looks_effectful_without_kind()` re-derives NodeKind from port type strings (`"TransportRequest"`, `"ToolHandle"`, `starts_with("res:")`) to validate the lowerer. Becomes dead code once RF-A1 makes `kind: NodeKind` non-Option. Track as RF-A1 follow-up. | `core/exec/src/execute.rs:2064-2092` | RF-H4 PR scout | 2026-02-26 |
| Heuristic reimplementation | `classify_module()` aggregates ALL callables in the compiled output, including transitive auth callables from `std.patterns` (github_oidc, local_auth, metadata_oidc). Module-level classification is inflated beyond what the entry-point actually uses. Consider callable-scoped or entry-point-reachable classification. | `gunbc-dag/src/fidelity.rs:184-209` | RF-E4 impl | 2026-02-26 |
| Fallback arm | HTTP method `_ => RestRequest::post(&url)` — unknown methods silently become POST instead of failing. A typo in a DSL `@rest(PTCH, ...)` annotation would produce a POST request with no error. Parse HTTP method to an enum at boundary (lowerer or resolve step). | `gunbc-dag/src/resolve_service.rs:72-79` | R1 scout | 2026-02-26 |
| String dispatch | `match field.type_id.as_str()` for JSON→Value conversion appears twice (`parse_output_field` and `default_output_value`). Both use the same type string set (`"Secret"`, `"Int"`, `"Bool"`, `"Bytes"`, `"Json"`, fallback to String). A `TypeId` enum with `to_default_value()` and `parse_json()` methods would consolidate both match blocks. | `gunbc-dag/src/resolve_service.rs:291-335, 352-366` | R1 scout | 2026-02-26 |
| Validation at use site | `input_as_string()` returns `"(unresolved)"` for missing inputs — a magic string that silently flows into HTTP requests and shell commands as a real value. Should return `Result<String, ExecError>` when no default is provided. | `gunbc-dag/src/resolve_service.rs:634-641` | R1 scout | 2026-02-26 |
| String dispatch | `match self.spec.operation.as_str()` for file operations (`"READ"`, `"READ_BYTES"`, `"WRITE"`). Has an error arm for unknown ops (good), but the operation is still a runtime string. Could be a `FileOperation` enum parsed at resolve time. | `gunbc-dag/src/resolve_service.rs:933-948` | R1 scout | 2026-02-26 |
| String dispatch | `workflow_unit_commands()` matches workflow name strings to hand-written command builders (10 arms + error fallback). Related to RF-A10 (registry pattern for DAG tooling dispatch). Consider whether a registry-driven approach or DSL-declared command specs could replace this. | `gunbc-dag/src/workflow/unit_commands.rs:300-323` | R1 scout | 2026-02-26 |

---

## Red Backlog

Triaged and sized. Promote to lane queues when horizon items are exhausted.

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

**RF-G-unblock** (in R1 queue position 4): Implement `fold` aggregation
in Rust's `evaluate_fn_body()` evaluator (it already handles other
collection ops). This directly enables calling the existing DSL fns.
Once done, delete:
- `transport_depth_ordinal()`, `transport_depth_str()` (RF-G1)
- `transport_is_hermetic()` (RF-G2)
- `classify_callable()` pre-aggregation (RF-G3)
- `test_policy.dag::classify_from_facts()` (RF-G4)
- `TestClass::parse()` / `FermiCost::parse()` round-trip (RF-G5)
- `test_policy.dag` shadow fns (RF-G6)
- Fidelity silent fallbacks `unwrap_or(Unit)` / `unwrap_or(XS)` (was RF-H1)

Note: RF-A7 ("transport class heuristic shadows") was deleted as a
standalone item — it is a deliverable of this task, not independent work.

### Theme H: Structural Enforcement (parse, don't validate)

| ID | What | Current | Structural Fix |
|----|------|---------|---------------|
| RF-H2 | **TestgenTargetDef Option fields always populated**. `test_class: Option<TestClass>`, `fermi_cost: Option<FermiCost>`, `requires: Option<Vec<String>>` — every auto-testgen call site now fills `Some(...)` from fidelity. The `Option` only exists for legacy `DagSpecDef` path (which also never overrides). `generate_target_with_types()` does `unwrap_or(Unit)` on every field. | 6 Option fields in registry.rs | Make fields non-Option with `Default` impl. Callers construct with values; no unwrapping. `DagSpecDef.to_def()` fills from fidelity instead of leaving `None`. |
| RF-H4 | **ResourceKind string dispatch**. `ResourceAcquireOp { resource_kind: String }` matched at runtime. Unknown kinds fall through to `Value::Str("resource:{other}")` — wrong type, silent. | resolve.rs:365-386 | `ResourceKind` enum parsed once at resolve time. Match is exhaustive, no fallback arm. |

Deleted from this theme (subsumed by other tasks):
- RF-H1 → subsumed by RF-G-unblock (fidelity silent fallbacks eliminated when classify_transports returns typed DerivedClassification)
- RF-H3 → merged into RF-A8 (derive macro handles FromStr for all 15 enums including TestClass/FermiCost)
- RF-H5 → duplicate of RF-A2a/RF-A2b (PortCategory enum)

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
| RF-A8 | **`as_str`/`parse` boilerplate**. `#[derive(StringEnum)]` for 15 enums (~60 match blocks). Includes TestClass/FermiCost `FromStr` (was RF-H3). | 12 files | |
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
