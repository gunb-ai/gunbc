# Tasks

**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

---

## Operating Model: Blue Team / Red Team

One blue lane (scenario-driven), two red lanes, never blocking each other.

```
  BLUE TEAM — Advance                     RED TEAM — Harden
  ────────────────────────────            ────────────────────────
  SDLC Activation (single lane):         Lane R1: Structural Correctness
    B-0 → B-1 → B-2 → B-3 →               RF-RG1 → RF-RG2 → RF-H4 →
    B-TC → B-4 → B-5 → B-6 →              RF-H2 → RF-G-unblock → RF-A1 →
    B-7 → B-8:13 → ...cloud                RF-A2a → RF-A4

                                          Lane R2: Testing + Foundation
                                            BB-2 → BB-3 → BB-5 →
                                            FC-P7-c2 → FC-P7-d →
                                            FC-CF5 → FC-CF6 →
                                            FC-P8-a → FC-P8-b → FC-P8-c
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

**DSL is comprehensive** (~3,600 lines across 20 .dag files):
- Pipeline: 11 stages, complete with service calls and LLM integration (`pipelines/sdlc.dag`, 551 lines)
- Stage handlers: 8/8 implemented with real logic (`funcs/sdlc_stages.dag`, 739 lines)
- Worker dispatch: full discover→claim→dispatch→record→release (`funcs/sdlc_worker.dag`, 381 lines)
- Dispatch policy: 6 stage transition fns (`funcs/sdlc_dispatch_runtime.dag`, 104 lines)
- Validation policy: 3 gate fns (`funcs/sdlc_validation_runtime.dag`, 59 lines)
- Interfaces: 7 with 24 capabilities (`interfaces/`)
- Providers: 9 implementations — stubs, file-based, GCS-based (`services/sdlc/providers/`)
- Profiles: 3 — unit_test (all stubs), local (GitHub + file), cloud_run (GCS + PubSub) (`profiles/sdlc.dag`)
- DSL-level tests: 10+ test blocks defined in .dag files

**What's missing** — Rust-side infrastructure to compile and execute:
- resolve.rs doesn't handle SDLC module paths (generalization needed — Red RF-RG2)
- Profile-aware compilation doesn't exist
- Transport declarations missing on GitHub/LLM services (compositional modeling — Red RF-TC1)
- No CLI entrypoint (catalog is manual — Red RF-RG1 eliminates, or Blue wires inline)

**Key dependency**: L0–L3 use stubs and don't need transport declarations.
L4+ needs transport on GitHub + LLM services — B-TC handles this in the Blue queue.

### Queue

| Order | ID | Task | Level | Size | Status | Deps |
|-------|----|------|-------|------|--------|------|
| 1 | B-0 | **Compile SDLC pipeline.** `build_dsl_graph_for_entry("pipelines/sdlc.dag", "...")` succeeds. May need minimal resolve.rs wiring for SDLC module paths. Profile-aware compilation for unit_test profile. | L0 | M | Pending | — |
| 2 | B-1 | **Hermetic scenario test.** unit_test profile, DryRun, full idea→done with stubs. Assert: stage transitions correct, claims acquired/released, outcomes recorded, labels changed. | L1 | M | Pending | B-0 |
| 3 | B-2 | **Per-stage handler tests.** Exercise all 8 handlers individually with mocked interfaces. Verify each handler's outputs, label transitions, service call arguments. | L2 | M | Pending | B-0 |
| 4 | B-3 | **Worker dispatch loop test.** Full discover→claim→dispatch→record→release cycle. Test paths: happy path, replay-skip (prior SUCCESS), retry (prior FAILED), claim conflict (another worker holds it). | L3 | S | Pending | B-2 |
| 5 | B-TC | **Transport declarations: GitHub + LLM services.** Add `transport rest { ... }` blocks to `github/issues.dag` (7 ops), `github/pull_request.dag` (7 ops), `llm/openai.dag` (2 ops). Compositional service-layer modeling — not SDLC logic, but on the critical path for real execution. | — | M | Pending | — |
| 6 | B-4 | **Local integration: single stage.** local profile, real GitHub API + file stores. Create a test issue with `sdlc:idea`, run worker, verify design comment posted and labels transitioned. | L4 | M | Pending | B-3, B-TC |
| 7 | B-5 | **Full local scenario.** Complete idea→done lifecycle on a test repo. Multiple worker invocations drive the issue through all stages. Verify: PR created, code review posted, tests run, PR merged, issue closed. | L5 | L | Pending | B-4 |
| 8 | B-6 | **Testgen integration.** Auto-generate per-node and per-pair tests for SDLC DAG nodes. Verify testgen handles profile-bound modules (interface→provider resolution). | L6 | M | Pending | B-0 |
| 9 | B-7 | **CLI entrypoint.** However entrypoints work by this point (generated binary or catalog), make `gunbc sdlc` run the pipeline with `--profile` and `--repo` args. | L7 | S | Pending | B-5 |

### Horizon (after B-7)

| ID | Task | Level | Size | Deps |
|----|------|-------|------|------|
| B-8 | GCS SignalStore (PubSub-backed, at-least-once) | L8 | M | B-7 |
| B-9 | GCS ArtifactStore (content-hash, generation CAS) | L8 | M | B-7 |
| B-10 | GCP credential chaining (WIF OIDC exchange) | L8 | L | B-7 |
| B-11 | Cloud Run deployment DAG | L8 | L | B-8:10 |
| B-12 | Multi-worker CAS stress test (3 workers, exactly-once) | L8 | M | B-11 |
| B-13 | CI integration (hermetic + cloud smoke) | L8 | M | B-12 |
| B-AG1 | Agent provider: wire codex_agent.dag to real LLM | L5 | M | B-5 |
| B-AG2 | Credential provider: local keychain for tokens | L5 | M | B-5 |
| B-AG3 | Webhook-driven stage transitions | L8 | L | B-AG1 |

**Deliverable**: `gunbc sdlc --profile local --repo owner/name` runs full lifecycle.
**Endstate**: SDLC on Cloud Run with GCS stores, PubSub signals, multi-worker CAS.

### Design References

| Document | What |
|----------|------|
| `docs/design/sdlc/mega-modeling-design.md` | Canonical architecture: 9 high-level boxes, core abstractions, canonical contracts, conformance model |
| `docs/design/sdlc/domain-modeling-comprehensive.md` | All domain objects, state machines, invariants |
| `docs/design/sdlc/e2e-gap-analysis.md` | Gap tracking (A–J, all resolved at DSL level) |
| `docs/design/sdlc/implementation-roadmap.md` | Task breakdown and dependency graph |

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

## Lane R1: Structural Correctness

Types over validation, enums over strings. Touches `core/ir/`,
`core/test/`, `gunbc-dag/src/resolve.rs`, `gunbc-dag/src/binaries.rs`,
`gunbc-dag/src/workflow/catalog.rs`.
**No overlap with Lane R2 files.**

Queue ordered by: manual registries first (pre-SDLC cleanup the user
explicitly flagged), then danger (silent failures), then mechanical cleanup.

| Order | ID | What | Size | Status | Deps |
|-------|----|------|------|--------|------|
| 1 | RF-RG1 | **Manual registry elimination.** `WorkspaceBinary` enum (13 entries, `binaries.rs`) is derivable from Cargo.toml `[[bin]]`. Workflow variant catalog (10 entries, `catalog.rs`) is derivable from DSL `pipeline` declarations + annotations for aliases/modes. Delete both manual tables, replace with auto-derivation. | M | Pending | — |
| 2 | RF-RG2 | **Resolve generalization.** `resolve.rs` hardcodes 3 module paths (`std.resources`, `tools.infra`, prefix `services.*`). New DSL modules (SDLC, future tools) must work without touching resolve.rs. Replace with metadata-driven dispatch or convention-based fallthrough. | M | Pending | — |
| 3 | RF-H4 | ResourceKind string dispatch → enum. Easiest win, local to resolve.rs. | S | Pending | — |
| 4 | RF-H2 | TestgenTargetDef Option fields → non-Option with defaults. | S | Pending | — |
| 5 | RF-E4 | Fidelity classification smoke test. Assert makegen→Hermetic/S, gist→Integration/L. | S | Pending | — |
| 6 | RF-G-unblock | `fold` extraction in evaluate_fn_body() — enables calling DSL classify_transports(). Deletes all RF-G1:6 shadows + fidelity silent fallbacks. | M | Pending | — |
| 7 | RF-A1 | NodeKind required on Node\<T\>. Remove Option, require in builders. | M | Pending | — |
| 8 | RF-A2a | Port namespace typing: define `PortCategory` enum + methods on `PortName` in `core/ir/`. | M | Pending | — |
| 9 | RF-A4 | CallableClass enum in resolve.rs. Parse once, dispatch everywhere. | M | Pending | — |

### R1 Horizon (after A4)

| Order | ID | What | Size |
|-------|----|------|------|
| 10 | RF-A5 | TransportNodeKind enum (Prepare/Execute/Parse). | S |
| 11 | RF-A6a | String constants: define central consts in `core/ir/src/signature.rs`. | S |
| 12 | RF-A8 | `#[derive(StringEnum)]` macro for 15 enums (~60 match blocks). Includes TestClass/FermiCost `FromStr`. | M |
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
| 11 | RF-A2b | Port namespace typing: migrate `starts_with("res:")`/`"tool:"` call sites to `PortCategory`. | S |
| 12 | RF-A6b | String constants: migrate `__deps`/`res:file`/`tool:` references to central consts. | M |
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
| *(empty — add observations here)* | | | | |

---

## Red Backlog

Triaged and sized. Promote to lane queues when horizon items are exhausted.

### Theme TC: Transport Completeness (compositional modeling)

Transport declarations belong on the **service layer**, not on SDLC.
The SDLC pipeline only sees interfaces. Each service operation needs a
`transport rest { ... }` or `transport shell { ... }` block so the
compiler can generate prepare→execute→parse triplets.

GitHub + LLM transport (16 ops) moved to Blue queue as B-TC (critical path for L4+).

| ID | Scope | Ops Missing | Notes |
|----|-------|-------------|-------|
| RF-TC3 | **SDLC providers**: file stores (6), GCS stores (6), github_issue_provider (7), codex_agent (4), credential providers (4) | 27 | Mixed rest/shell/file. Needed for real (non-stub) execution. |
| RF-TC4 | **Stub providers**: stub_providers.dag (26), stub_credential_provider.dag (2) | 28 | Intentional — unit_test profile stubs. Consider `transport stub {}` marker. |
| RF-TC5 | **Infrastructure stubs**: azure (43), aws (38), gcp-infra (59) | 140 | Dormant — defer until infrastructure provisioning lane opens. |

### Theme E: Deleted Tests (re-add when root cause fixed)

| ID | Deleted Tests | Root Cause | Blocker |
|----|---------------|-----------|---------|
| RF-E5 | `makegen_runtime_differential_interpreter_vs_generated_rust_layer1` (codegen_parity.rs) | FnBodyDelegate gap: interpreter produces raw `{header}{body}`, fn body evaluation only works via `shared.rs` direct path. | Interpreter needs fn body evaluation support. |
| RF-E6 | `makegen_exec_runtime_e2e_structural_verification` (daglang-driver), `pragma_exec_runtime_e2e_structural_verification` (daglang-driver), `makegen_e2e_generated_binary_produces_correct_makefile` (cli_commands), `pragma_e2e_generated_binary_produces_correct_config_files` (cli_commands) | Exec-runtime emitter missing: `LoadRegistry` handler, `PureRender` fn classification, `ContentUpsertOutputPath` classification. | `daglang-emit` exec-runtime backend needs node classification for all makegen/pragma node kinds. |
| — | `clippy_toml_dsl_produces_valid_output` (pragma_parity.rs) | Sum type variant tags lost during `build_data_values()` JSON serialization. | FC-CF5 (recursive types). Already tracked in R2 queue. |

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
- Fidelity silent fallbacks `unwrap_or(Unit)` / `unwrap_or(XS)`

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
