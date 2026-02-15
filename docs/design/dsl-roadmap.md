# DSL Roadmap: Staged Build + gunbc Migration

**Status**: Working Draft — February 2026
**Companion**: [`dsl-design.md`](./dsl-design.md)

---

## Diagnosis

The gunbc drift problem is **not** "we didn't model enough." It is:

> **We modeled the IR aggressively while other layers (spec / registry / discovery / emission / progress) were under-modeled**, so meaning leaked into glue, templates, string IDs, and ad-hoc rules — creating redundancy, rework pressure, and refactor churn.

"Drift" almost always means we've accidentally allowed **multiple sources of truth** — hand-written workflow logic here, a quasi-DSL there, ad-hoc conditionals somewhere else. The fix is to make the **DSL the single source of truth** for each discrete workflow, then unify execution + policy + modeling behind one engine.

The strategy: (1) inventory workflow contracts, (2) build the DSL in slices proven by real workflows, (3) migrate gunbc onto it with "map workflows → prove parity → then unify," (4) lock it down with guardrails so drift can't come back.

---

## Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    WORKFLOW CONTRACTS (Part 0)                           │
│                                                                         │
│  For each workflow (gist/ci/review/auth/…):                             │
│  ─ entry points, inputs, semantics, outputs, error model                │
│  ─ side effects, observability contract                                 │
│  ─ workflow matrix + golden fixtures = "non-drift harness"              │
├─────────────────────────────────────────────────────────────────────────┤
│                         DSL BUILD (Part 1)                              │
│                                                                         │
│  Phase 0         Phase 1          Phase 2          Phase 3    Phase 4   │
│  Scaffolding     Language Core    Services +       Comp +     Pipelines │
│                  + Discovery      Resources        Loops      + Backend │
│  ─ compiler      ─ types          ─ service decl   ─ for      ─ stages  │
│    entrypoint    ─ journeys       ─ @rest/@shell   ─ SubDag   ─ parall. │
│  ─ module graph  ─ patterns       ─ match/when     ─ scatter  ─ Go/Py   │
│  ─ parity        ─ resources      ─ resource LC    ─ TUI                │
│    harness       ─ manifest                                             │
├─────────────────────────────────────────────────────────────────────────┤
│                       MIGRATION (Part 2)                                │
│                                                                         │
│  Workstream A: Parity Harness (backbone — runs continuously)            │
│  Workstream B: Port workflows in scenario order (S1→S3→S2→S4→…)        │
│  Workstream C: Engine Unification (single engine consumes IR)           │
│                + gunbc becomes adapter layer (not second execution path) │
├─────────────────────────────────────────────────────────────────────────┤
│                     MODELING SWEEP (Part 3)                             │
│                                                                         │
│  Lens 1: Is the model closed?                                          │
│  Lens 2: Are semantics preserved through lowering?                      │
│  Lens 3: Does any cross-cutting concern lack a single home?             │
│  Lens 4: Is discovery single-source?                                    │
│  Lens 5: Anemia signals (DTOs, stringly-typed status, scattered policy) │
├─────────────────────────────────────────────────────────────────────────┤
│                       GUARDRAILS (Part 4)                               │
│                                                                         │
│  ─ Golden tests (DSL fixtures → IR snapshots → expected event streams)  │
│  ─ Dual-run / diff mode (old behavior vs new engine, compare outputs)   │
│  ─ Schema/typing CI (DSL must typecheck; invalid configs fail fast)     │
│  ─ IR stability checks (serialization snapshot tests)                   │
│  ─ Lint rules (forbid new ad-hoc execution paths outside the engine)    │
│  ─ Doc generation from DSL (eliminates doc drift)                       │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Part 0 — Workflow Contract Inventory

> **Before touching DSL code**, treat each workflow as a "product surface" with a contract. This becomes the hard target for the entire migration: once you refactor, you need to know whether you accidentally changed behavior.

### Why first

Contracts + fixtures give you a hard target. Without them, parity is an opinion. With them, parity is a CI check.

### Per-workflow contract

For each discrete workflow (gist, ci, review, auth, makegen, clippy, deps, ...):

| Contract dimension | What to capture |
|---|---|
| **Entry points** | How invoked: CLI, CI entrypoint, webhook, `make` target, other tool |
| **Inputs** | Required fields, defaults, validation rules, env vars consumed |
| **Semantics** | Ordering, retries, idempotency, concurrency, approval gates, permissions |
| **Outputs** | Emitted artifacts, events, statuses, exit codes |
| **Error model** | Retryable vs terminal, partial failure semantics, compensation behavior |
| **Side effects** | External calls, mutations, notifications, file writes |
| **Observability** | Traces, metrics, structured logs, correlation IDs |

### Deliverables

| Deliverable | Description |
|---|---|
| **Workflow matrix** | Rows = workflows, columns = triggers / inputs / steps / policies / outputs |
| **Golden fixtures** | Representative real configs + expected behavior snapshots per workflow |
| **Contract test suite** | Automated checks that behavior hasn't changed (entry: fixture → exit: expected outputs + side effects) |

### Workflow Matrix (starter)

| Workflow | Entry point | Key inputs | Pattern | Policies | Side effects | Key output |
|---|---|---|---|---|---|---|
| **makegen** | CLI, CI | tool registry | `content_upsert` | skip-if-unchanged | Makefile write | Makefile artifact |
| **gist** | CLI | branch, files, mode | composition + loop | — | GitHub API, file reads | Gist URL |
| **ci** | CLI, make | — | pipeline (staged) | retries, parallelism | build, test, lint | pass/fail + report |
| **review** | CLI | diff, model | service + credential | — | LLM API call | review markdown |
| **credential** | SubDag (internal) | runtime, project | `credential_chain` | token expiry | OIDC/STS/API calls | `Secret` token |
| **clippy** | CLI, CI | workspace | `upsert` | skip-if-installed | shell (rustup) | lint results |
| **auth** | CLI | provider | interactive | — | browser launch, OAuth | stored credential |
| **deps** | CLI, CI | manifest | `upsert` | skip-if-satisfied | shell (cargo/npm) | installed deps |

### Acceptance Gates

- [ ] Every workflow has a one-page contract document (or a row in the matrix with all columns filled)
- [ ] At least one golden fixture per workflow with expected outputs
- [ ] Contract tests runnable in CI (fixture → expected behavior, independent of whether DSL or builder produced the graph)

---

## Part 1 — Staged DSL Build

Each phase is anchored on a canonical workflow from the [scenario inventory](./dsl-design.md#j2-scenario-inventory). Each phase proves a language slice by compiling a real workflow end-to-end.

---

### Phase 0: Compiler Scaffolding

> **Goal**: Make the DSL a first-class producer of *gunbc's existing IR* (`Dag`/`Node`/`Port`/`Edge`), not a parallel system.

| Deliverable | Description |
|---|---|
| `core/daglang/` workspace area | New crate that discovers `.dag` files from a project manifest + filesystem layout |
| Module graph | Resolve imports, produce a dependency-ordered module graph |
| Compiler entrypoint API | `compile_project() -> { typed_ir, progress_manifest, test_obligations, tool_metadata }` |
| Parity harness | Framework to prove compiled `.dag` output is equivalent to existing hand-wired builders (see Part 2, Workstream A) |
| IR serialization + snapshot tests | Stable IR serialization format; snapshot tests detect accidental semantic changes |
| DSL versioning | `dsl_version` in project manifest so semantics can evolve safely |

**Acceptance Gates**

- [ ] "Compile-only" smoke test discovers `.dag` files and reports modules without executing anything
- [ ] Can compile one `.dag` file into a valid gunbc IR structure (even with stubby node bodies)
- [ ] Parity harness can canonicalize and diff two IR graphs
- [ ] IR snapshot test passes for at least one compiled `.dag` file

**Why first**: gunbc already has a strong IR and strong enforcement patterns ("everything is a DAG," I/O only at boundaries). The DSL should be a *front-end* that closes modeling gaps, not a replacement runtime.

---

### Phase 1: Language Core + Discovery + ProgressManifest

> **Proving workflow**: `makegen` (scenario S1 — simplest complete graph)

| Construct | What it covers |
|---|---|
| Minimal type system | Records, enums/sums, `T?`, `List<T>`, `Map<K,V>` — sufficient to typecheck `makegen` |
| `journey` syntax | Implicit edges via references |
| `pattern content_upsert` | Canonical render → read → compare → write chain |
| Resource declarations | `uses fs: Filesystem(mode: Write)` with compiler-inserted acquire/use wiring |
| ProgressManifest | Compiler-derived waves, topo depth, labels, SubDag boundaries |

**Deliverables**

| Deliverable | Artifact |
|---|---|
| `.dag` for makegen | `tools/makegen.dag` compiles to same shape as existing `build_makegen_graph()` |
| ProgressManifest | Generated for makegen DAG (waves, labels, boundaries) |
| Filesystem discovery | Module graph built from project manifest + filesystem scan |

**Acceptance Gates**

- [ ] **IR parity**: compiled IR equivalent to existing builder IR for `makegen` (nodes/edges/ports, normalized labels)
- [ ] **Test parity**: existing obligation/testgen model runs against compiled output — same 4 buckets, "DryRun completes" and "transport interceptable" tests pass
- [ ] **Discovery**: `makegen.dag` is auto-discovered without any manual registration

**Corresponds to**: [dsl-design.md Phase 1](./dsl-design.md#phase-1-language-core--module-discovery--progress-manifest), [Appendix A](./dsl-design.md#appendix-a-content-upsert-makegen)

---

### Phase 2: Services + Resources + Cloud Modeling

> **Proving workflow**: `acquire_gcp_secret` (scenario S2 — most complex graph, canonical stress test)

| Construct | What it covers |
|---|---|
| `service` declarations | `operation input/output` with transport annotations (`@rest`, `@shell`) |
| `match` / `when` | Runtime branching (GitHub Actions vs Metadata vs Local), guarded ports |
| `resource` with lifecycle | `Credential`, `Network` — acquire/use/release |
| Late-bound transport | Semantic metadata survives lowering (avoids "generic IR chokepoint") |

**Deliverables**

| Deliverable | Artifact |
|---|---|
| `.dag` for credential chain | `cloud/gcp/credential.dag` + `cloud/gcp/secret_manager.dag` (scenario S2) |
| Transport triplet emission | Service calls → prepare/execute/parse, without authors ever writing triplets |
| Semantics carrier | Service metadata (hermeticity, idempotency, permissions) survives to Derive/Validate |

**Acceptance Gates**

- [ ] **Classification**: calls classified (local git shell vs network REST) from service declarations, not from generic `TransportRequest` variants
- [ ] **Resource lifecycle**: acquisition/release nodes inserted by compiler; resource conflicts detected during validation
- [ ] **IR parity**: compiled credential chain matches existing `lib/gcp-ops/src/graph.rs` shape
- [ ] **Semantic preservation**: `@idempotent`, `@readonly`, `@permissions` annotations survive lowering and are accessible to test categorizer

**Corresponds to**: [dsl-design.md Phase 2](./dsl-design.md#phase-2-services--resources--cloud-modeling), [Appendix B](./dsl-design.md#appendix-b-cloud-credential-acquisition-gcp)

---

### Phase 3: Composition + Loops + TUI-Capable Progress

> **Proving workflow**: `gist_snapshot` (scenario S4 — multi-service composition with loops)

| Construct | What it covers |
|---|---|
| `for` loops | LoopBuilder equivalent |
| Journey composition | SubDag expansion (journey calls become SubDag nodes) |
| Scatter points | ProgressManifest includes loop expansion points for grouped counters |
| TUI progress | Manifest-driven rendering restores capabilities lost from the-gunbai |

**Deliverables**

| Deliverable | Artifact |
|---|---|
| `.dag` for gist | `tools/gist.dag` (snapshot/diff/recent) compiles and runs |
| Scatter progress | ProgressManifest includes scatter points; renderers show `read files [8/8]` |
| Static DAG viz | Visualize graph before execution (from manifest, not runtime) |

**Acceptance Gates**

- [ ] **Compression**: gist workflow expressed in ~80 lines of `.dag` (vs 1,449 lines of Rust builders)
- [ ] **Loop progress**: renderers display loop progress as grouped counter without manual configuration
- [ ] **Composition**: SubDag calls work for credential chain reuse within gist workflow
- [ ] **IR parity**: compiled gist graph matches existing builder shape for all 3 modes

**Corresponds to**: [dsl-design.md Phase 3](./dsl-design.md#phase-3-composition--tui-progress), [Appendix C](./dsl-design.md#appendix-c-service-composition-gist-snapshot)

---

### Phase 4: Pipelines + Stage Groups + Second Backend

> **Proving workflow**: CI pipeline (scenario S5 — largest composed graph, 133 obligations)

| Construct | What it covers |
|---|---|
| `pipeline` syntax | `stage`, `after`, `parallel`, `aggregate` |
| Stage groups | Derive stage groups into ProgressManifest |
| Second backend | Optional: Go or Python codegen backend |

**Deliverables**

| Deliverable | Artifact |
|---|---|
| `.dag` for CI | `pipelines/ci.dag` equivalent to current CI builder |
| Stage progress | Manifest-derived stage group progress |
| (Optional) Second backend | Same `.dag` → Go or Python output |

**Acceptance Gates**

- [ ] **Obligation parity**: CI obligation stats match expectations (133 obligations)
- [ ] **Bootstrap constraint**: CI entrypoint handles the "runs codegen, can't depend on generated code" constraint explicitly
- [ ] **Stage groups**: progress renderers display pipeline stages as collapsible sections
- [ ] **IR parity**: compiled CI graph matches existing 920-line builder shape

**Corresponds to**: [dsl-design.md Phase 4](./dsl-design.md#phase-4-pipelines--second-codegen-backend), [Appendix D](./dsl-design.md#appendix-d-ci-pipeline)

---

### Phase Summary

```
Phase   Proving Workflow          Language Slice                        Key Risk
─────   ─────────────────         ─────────────────────────────         ─────────────────────────
  0     (none — scaffolding)      Discover + Parse + Module Graph       IR integration boundary
  1     makegen (S1)              types, journey, pattern, resource     Pattern expansion fidelity
  2     acquire_gcp_secret (S2)   service, match/when, resource LC      Generic IR chokepoint
  3     gist_snapshot (S4)        for, composition, scatter progress    TUI renderer integration
  4     CI pipeline (S5)          pipeline, stage, parallel, aggregate  Bootstrap constraint
```

---

## Part 2 — Migration Plan (gunbc → DSL)

Strategy: **write DSL versions first, compile to gunbc IR, prove equivalence, then unify.**

---

### Workstream A: Parity Harness (migration backbone)

> Runs continuously throughout migration. Makes the "one big unify" safe and bounded.

For each workflow/tool being migrated:

```
1. Keep existing Rust builder  build_*_graph()
2. Add a .dag file and compile it to IR
3. Canonicalize both graphs:
   ├── Stable sort nodes/edges
   └── Normalize IDs if needed
4. Diff and assert parity:
   ├── Node count, edge count
   ├── Port types/cardinality compatibility
   ├── Boundary + entrypoint sets
   └── Transport boundary node set (execute nodes)
5. Require parity before cutover
```

**Key property**: you keep shipping while migrating. Both sources coexist until parity is proven.

---

### Workstream B: Port Workflows (scenario-ordered)

Port order follows the scenario inventory, because it progressively exercises the language surface while minimizing risk:

| Order | Scenario | DSL Phase | Key Constructs Exercised | Lines Today | Lines DSL (est.) |
|:---:|---|:---:|---|:---:|:---:|
| 1 | **S1** Makegen | Phase 1 | `pattern content_upsert`, resource decl | ~200 | ~25 |
| 2 | **S3** Tool install/upsert | Phase 1-2 | `pattern upsert`, shell boundaries | ~150 | ~30 |
| 3 | **S2** Credential chain | Phase 2 | `service`, `match`/`when`, `resource` lifecycle | ~1,688 | ~50 |
| 4 | **S4** Gist | Phase 3 | Composition, `for` loop, multi-service | ~1,449 | ~80 |
| 5 | **S6** Review | Phase 2-3 | External API service, credential reuse | ~1,376 | ~50 |
| 6 | **S7** Auth flow | Phase 3 | `@interactive` passthrough, browser/platform resource | ~400 | ~40 |
| 7 | **S5** CI pipeline | Phase 4 | `pipeline`, `stage`, `parallel`, `aggregate` | ~920 | ~60 |

**Why this order**: mirrors DSL phasing. You don't build pipeline syntax before service/resource semantics are stable. Each port builds on proven constructs from earlier ports.

---

### Workstream C: Engine Unification + One Big Unify

> **Precondition**: every major workflow has a `.dag` source of truth and parity is proven.

The big unlock is to unify the runtime around a **single engine that consumes IR**. If gist/ci/review currently each implement pieces of scheduling, retries, error handling, or policy enforcement differently — that's the drift. Centralize it.

#### C0: Engine Unification (the central thrust)

The DSL is a frontend. The engine is where drift currently hides.

**Pipeline** (single path for all workflows):

```
Parse DSL → AST
  → Validate + Typecheck → Typed AST
    → Compile → IR
      → Execute → Engine (scheduler / executor / policy / state machine)
```

**Engine responsibilities** (centralized, not per-workflow):

| Responsibility | What it replaces |
|---|---|
| Scheduling + dependency resolution | Per-workflow topo sort + manual dep wiring |
| Retries, backoff, idempotency, timeouts | Scattered retry logic across crates |
| Concurrency control (global + per-key) | Ad-hoc parallelism in SubDags |
| Approvals / gates / manual intervention | Workflow-specific gate implementations |
| Permission checks + authz model | Per-service permission checking |
| Secret + resource binding | Manual resource threading through edges |
| Error handling + run state transitions | Inconsistent error classification across workflows |
| Standardized telemetry | Per-workflow logging/tracing |

**IR primitives** the engine consumes (stable set that covers all workflows):

| Primitive | Covers |
|---|---|
| `Trigger` / `InvocationContext` | How the workflow was invoked (CLI, CI, webhook, SubDag call) |
| `Inputs` (typed) | Schema-validated entry points |
| `Graph` of `Nodes` with `Deps` | The DAG — nodes are steps, edges are data flow + ordering |
| `Policies` | Retries, concurrency, timeouts, approvals, permissions |
| `Resources` | Secrets, caches, credentials, filesystem, network |
| `Effects` | Notifications, writes, external calls — explicit, auditable |
| `StateMachine` | Run lifecycle with enforced transitions |
| `Telemetry` hooks | Correlation IDs, structured events, progress manifest |
| `ErrorTaxonomy` | Retryable vs terminal, with domain-specific classification |

#### C0-a: gunbc as Adapter Layer (not second execution path)

> Key principle: `gunbc` should **not** be a second execution path. It should route into the same engine so you don't maintain two semantics.

| gunbc becomes | Description |
|---|---|
| **Compatibility facade** | Translates old calls → new engine IR invocation |
| **Intentionally dumb** | Mapping + compatibility defaults + deprecation warnings |
| **Exhaustively tested** | Contract tests so it stays stable while internals change |
| **Not an execution path** | Routes into the engine; no independent scheduling/policy logic |

During migration, gunbc builders still work. After cutover, they become thin adapters that compile to the same IR the `.dag` files produce. Eventually, the adapters are deleted when all consumers use `.dag` directly.

#### C1: Discovery Unification (delete islands)

Replace registration islands with filesystem discovery + module graph.

| Delete | Replace with |
|---|---|
| `build_workspace_dag()` hardcoded lists | Module graph as workspace DAG |
| `inventory`-based tool/testgen islands | `.dag` discovery (or keep for legacy compat) |
| `GraphBuilderId::as_str()` dispatch | Module graph → resolved function pointers |
| `all_tools()` 360-line vec | Filesystem scan of `tools/*.dag` |

**Metric**: manual tool registrations → **0**, stringly ID references → **0**

#### C2: Emission Unification (collapse rendering systems)

The compiler's `Emit` phase + `CodegenBackend` becomes the single home for emission.

| Converge | Into |
|---|---|
| 13 rendering systems, 5 traits | Manifest-driven renderer trait |
| `format!()` constructing source code | `TestFile` IR + `TestRenderer` trait |
| CLI generation variants | `emit_cli()` from DAG entrypoint ports |
| Progress output modes | Manifest-driven frame building |

**Metric**: rendering systems without IR/trait → **0**, `format!()` constructing source → **0**

#### C3: Progress Unification (manifest-driven renderers)

`ProgressManifest` becomes the shared input to all renderers.

| From | To |
|---|---|
| gunbc's frame-based renderer (no scatter) | Manifest with scatter points, SubDag boundaries |
| the-gunbai's runtime-only TUI | Static + live + replay from same manifest |
| gunb.ai's manual progress groups | Emergent sections from SubDag boundaries |
| Service `@interactive` annotation | Compiles to `CaptureMode::Passthrough` |

#### C4: Transport and IR Cleanup

The "generic IR chokepoint" fixes, safe to do now because `.dag` files are the stable source of truth.

| Cleanup | Details |
|---|---|
| Decouple `TransportRequest`/`TransportResponse` from `Value` | Transport is its own type family |
| Eliminate `ValueExpr` | Codegen works from IR + types |
| Move transport out of `core/ir/src/transport/` | Into transport crate |
| Hermeticity classification | From service declarations, not transport variant inspection |

---

### Migration Timeline

```
         Part 0           Phase 0          Phase 1         Phase 2         Phase 3         Phase 4
        ┌──────────┐     ┌──────┐        ┌──────┐        ┌──────┐        ┌──────┐        ┌──────┐
        │Contracts │────▶│Scaff.│───────▶│Core  │───────▶│Svc + │───────▶│Comp +│───────▶│Pipe +│
        │+ Fixtures│     │      │        │      │        │Res   │        │Loop  │        │Stage │
        └──────────┘     └──────┘        └──────┘        └──────┘        └──────┘        └──────┘
                                                │               │               │               │
                                                ▼               ▼               ▼               ▼
Migration:                           ┌──────────────────────────────────────────────────────────────┐
 Workstream A                        │  Parity Harness (runs continuously — build once, use always) │
 (backbone)                          └──────────────────────────────────────────────────────────────┘
                                                │               │               │               │
                                                ▼               ▼               ▼               ▼
 Workstream B                        ┌─────────┐┌──────────────┐┌─────────────┐┌──────────────┐
 (port)                              │S1,S3    ││S2,S6         ││S4,S7        ││S5            │
                                     │makegen  ││credential    ││gist, auth   ││CI pipeline   │
                                     │clippy   ││review        ││             ││              │
                                     └─────────┘└──────────────┘└─────────────┘└──────────────┘
                                                                                        │
                                                                                        ▼
 Workstream C                                                                  ┌──────────────────┐
 (unify)                                                                       │ C0: Engine Unify │
                                                                               │ C0a: gunbc adapt │
                                                                               │ C1: Discovery    │
                                                                               │ C2: Emission     │
                                                                               │ C3: Progress     │
                                                                               │ C4: Transport/IR │
                                                                               └──────────────────┘
                                                                                        │
                                                                                        ▼
 Part 3: Modeling                                                              ┌──────────────────┐
 Sweep (during C0)                                                             │ Lens 1-5 sweep   │
                                                                               │ Anemia fixes     │
                                                                               │ Modeling upgrades│
                                                                               └──────────────────┘
                                                                                        │
                                                                                        ▼
 Part 4: Guardrails                                                            ┌──────────────────┐
 (lock it down)                                                                │ Golden tests     │
                                                                               │ IR snapshots     │
                                                                               │ Schema CI        │
                                                                               │ Lint rules       │
                                                                               │ Doc generation   │
                                                                               └──────────────────┘
```

---

## Part 3 — Modeling Sweep

Run the sweep against root-cause failure modes, not "hunt redundancy." Some apparent duplication is intentional (the design doc [explicitly corrects](./dsl-design.md#k4-what-is-not-a-failure-correcting-overstatements) overstated redundancy claims).

Do this sweep **during** the engine unification (Workstream C0) — because the new engine + IR is where domain concepts should become crisp.

---

### Lens 1: Is the Model Closed?

> If code can reach the environment implicitly, it leaks meaning outside the DAG and breaks DryRun reasoning.

**What to do during migration**: for each workflow, inventory any implicit env access and convert to:

| Implicit access | Convert to |
|---|---|
| `std::env::var()` | Explicit input ports on graph root |
| `SystemTime::now()` | `uses clock: Clock` resource declaration |
| `Platform::detect()` | `PlatformEnv` node with explicit env port |
| `FilesystemHandle::new()` | `uses fs: Filesystem(mode: ...)` resource declaration |

**Non-negotiable gates** (already enforced, keep during migration):
- [ ] Repo-wide purity checks
- [ ] Resource declaration audits
- [ ] Clippy guardrails preventing direct I/O outside transport/boundary crates

---

### Lens 2: Are Semantics Preserved Through Lowering?

> Distinctions that matter (hermetic vs non-hermetic shell) must not be erased when everything becomes `TransportRequest::Shell`.

**What to model**:

| Semantic | Source | Must survive to |
|---|---|---|
| Hermeticity | `@shell` without `@permissions` = hermetic | Test categorizer (integration vs external) |
| Idempotency | `@idempotent` on operation | Retry policy, cache invalidation |
| Read-only | `@readonly` on operation | Parallel execution safety |
| Permissions | `@permissions([...])` on operation | Auth validation, scope checking |

**Test**: after lowering, can the executor distinguish `git ls-files` (hermetic) from `gist create` (non-hermetic) without inspecting strings?

---

### Lens 3: Does Any Cross-Cutting Concern Lack a Single Home?

> If you find a third copy of something, resist "refactor now." Instead, decide where it belongs.

| Concern | Single home |
|---|---|
| Hashing | `std` patterns or `core/infra` |
| Freshness / skip-if-unchanged | `pattern content_upsert` (compiler expands) |
| Rendering / emission | Compiler `Emit` phase + `CodegenBackend` trait |
| Progress structure | Compiler `Derive` phase → `ProgressManifest` |
| Resource conflicts | Compiler `Validate` phase |
| Registry metadata | Compiler `Discover` phase (module graph) |
| Test obligations | Compiler `Derive` phase → `TestObligations` |
| Mock specs | Compiler `Derive` phase (from service declarations) |

---

### Lens 4: Is Discovery Single-Source?

> The filesystem discovery rules eliminate manual lists and "dag-viz can't see itself" blind spots.

**Migration consequence**:
- Treat module discovery as "the workspace DAG"
- Delete registration islands only **after** parity cutover (Workstream C1)
- **Verification**: every `.dag` file in `paths` directories must appear in the module graph, and the module graph must be the sole source for Makefile targets, CLI generation, and testgen registration

---

### Lens 5: Anemia Signals (what to hunt during the sweep)

> A good bar: if you can "read" a workflow run in code and see the invariants and transitions without hunting through helpers, the model is healthier.

**Red flags to look for**:

| Signal | What it looks like | Why it's drift |
|---|---|---|
| **DTO/dict workflows** | Workflow objects that are basically data bags with logic scattered across services | Invariants aren't co-located with the data they protect |
| **Stringly-typed status** | `"running"`, `"RUNNING"`, `"in_progress"` as interchangeable strings | No compiler-enforced state transitions |
| **Primitive-heavy signatures** | Functions taking 8+ primitives (`repo, sha, userId, …`) with implicit invariants | Missing value objects; callers can violate invariants |
| **`if workflow == "ci"` branching** | Workflow-specific conditionals scattered through shared code | Semantic drift — shared code knows too much about specific workflows |
| **Duplicated policy logic** | Retry rules implemented 3 different ways across crates | No single policy model |
| **Inconsistent error classification** | What's "retryable" differs by workflow with no shared taxonomy | Engine can't make consistent decisions |

**Modeling upgrades that pay off immediately** (do these during engine unification):

| Upgrade | What it replaces | Where it lives |
|---|---|---|
| **Explicit state machines** for runs/steps | Ad-hoc status strings + implicit transitions | Engine `StateMachine` primitive |
| **Value objects** for IDs, refs, artifact locators, durations | Raw strings, bare integers, unvalidated paths | IR types + DSL refinement types (`@pattern`, `@range`) |
| **Domain errors** with retryability + user-facing messages | Catch-all `anyhow::Error` or stringly-typed error kinds | Engine `ErrorTaxonomy` + DSL error model |
| **Aggregate boundaries** (e.g., `Run` as root, `StepAttempt` as entity, `Policy` as value object) | Flat structures with no ownership semantics | Engine IR primitives |
| **Policy as data** (retries/timeouts/concurrency/approvals) | Scattered config, per-workflow constants, hardcoded values | First-class `Policies` in IR, declared in DSL |
| **Effects as explicit outputs** (notifications, writes, external calls) | Implicit side effects buried in node implementations | Engine `Effects` primitive, auditable and testable |

---

## Part 4 — Guardrails Against Drift Re-Introduction

> Once unified, bake in drift prevention. The goal: it should be **harder to introduce drift than to avoid it**.

### Golden Tests (DSL fixtures → IR snapshots → expected event streams)

For each workflow:

```
.dag source  ──compile──▶  IR snapshot  ──execute──▶  expected event stream
     │                          │                           │
     │                          │                           │
  (checked in)            (snapshot test)            (golden fixture)
```

- DSL fixtures compile to IR snapshots; IR executes to expected event streams
- Any change to the DSL, compiler, or engine that alters output is caught
- Golden fixtures from Part 0 become the assertions here

### Dual-Run / Diff Mode (temporary, during migration)

While both old builders and `.dag` files coexist:

```
Input fixture
     │
     ├──▶ Old builder → IR → Execute → Output A
     │
     └──▶ .dag compile → IR → Execute → Output B
                                              │
                                        diff(A, B) == ∅
```

- Run both paths on the same fixture, compare outputs
- Any divergence is a bug (in the `.dag` file, the compiler, or the old builder)
- Remove once old builders are deleted (Workstream C)

### Schema / Typing CI

- DSL must typecheck: invalid `.dag` files fail CI
- Input schemas validated at compile time (or at least pre-run)
- Versioning: `dsl_version` in project manifest, so you can evolve semantics safely

### IR Stability Checks

- IR serialization snapshot tests (detect accidental semantic changes)
- If the IR representation of a workflow changes, the test fails — forces explicit acknowledgment
- Prevents "refactor that silently changes behavior" class of bugs

### Lint Rules

- Forbid new ad-hoc execution paths outside the engine
- Forbid direct I/O outside transport boundaries (existing clippy guardrails)
- Forbid `format!()` constructing source code (use `Emit` phase)
- Forbid manual tool registration (use filesystem discovery)

### Doc Generation from DSL

- Docs generated from schema + module definitions (eliminates doc drift)
- Workflow documentation is always accurate because it's derived from the source of truth
- API docs, CLI help text, and workflow descriptions all come from `.dag` + service declarations

---

## Definition of Done

The effort lands cleanly when all six criteria are met:

| # | Criterion | Measurable gate |
|:---:|---|---|
| 1 | **Every workflow authored in DSL** | gist, ci, review, auth, makegen, clippy, deps all have `.dag` sources that compile to the same IR |
| 2 | **One engine executes all workflows** | Policy enforcement (retries, concurrency, permissions, error handling) is centralized in the engine, not per-workflow |
| 3 | **gunbc routes into the engine** | gunbc is adapter-only with contract tests; no independent execution path |
| 4 | **Golden fixtures pass** | Behavior drift is either eliminated or explicitly versioned; contract tests green for all workflows |
| 5 | **Modeling sweep complete** | Fewer primitives/strings, explicit state machines, value objects, domain errors; anemia signals from Lens 5 resolved |
| 6 | **Meaningful code deletion** | A significant chunk of duplicated workflow-specific execution code is deleted (target: ~7,000+ lines of hand-wired builders → ~300 lines of pure logic) |

**Secondary criteria** (desirable but not blocking):

| # | Criterion | Measurable gate |
|:---:|---|---|
| 7 | **Test generation ratio** | 95%+ generated (up from 73%) |
| 8 | **Tool authoring cost** | ~20 lines in 1 `.dag` file (down from ~200 lines across 2 files) |
| 9 | **Zero manual registrations** | All 6 registration islands eliminated |
| 10 | **IR stability tests in CI** | Serialization snapshot tests for all workflows |

---

## Immediate Next Steps

Concrete actions that advance the roadmap without committing to a full rewrite.

### 0. Inventory workflow contracts (Part 0)

> The foundation everything else builds on. Do this first.

- [ ] Fill out the workflow matrix for all 7-8 discrete workflows (entry points, inputs, semantics, outputs, error model, side effects)
- [ ] Create at least one golden fixture per workflow (representative config + expected behavior snapshot)
- [ ] Wire golden fixture checks into CI (even if they just assert "old builder produces expected output" for now)

### 1. Add the parity harness (Workstream A)

> Turns "one big refactor" into a safe, bounded final step.

- [ ] Create `core/daglang/` workspace area
- [ ] Implement IR canonicalization (stable sort, ID normalization)
- [ ] Implement graph diff (node/edge count, port types, boundary sets)
- [ ] Wire into CI as an optional check

### 2. Start Phase 1 on makegen (S1)

> Exercises: pattern expansion (`content_upsert`), resource declaration, manifest derivation, minimal codegen. This is the "starting wedge" — the reference implementation that makes remaining ports mechanical.

- [ ] Write `tools/makegen.dag` (target: ~5 lines of authoring surface)
- [ ] Implement parser for minimal `journey` + `pattern` + `uses` syntax
- [ ] Implement `Lower` phase producing gunbc IR
- [ ] Run parity harness against existing `build_makegen_graph()`
- [ ] Verify golden fixture passes for compiled `.dag` output

### 3. Write `.dag` files early (even if compilation is partial)

> Mapping is the best way to surface modeling gaps.

- [ ] `tools/gist.dag` — composition + loop constructs
- [ ] `pipelines/ci.dag` — pipeline + stage constructs
- [ ] `tools/review.dag` — service + credential constructs
- [ ] `cloud/gcp/credential.dag` — resource lifecycle + branching

### 4. Keep existing invariants as gates

> The handbook's pipeline and invariants are exactly what to preserve while changing the authoring surface.

- [ ] I/O only at boundaries (no regression during migration)
- [ ] DryRun interception works for compiled `.dag` output
- [ ] Generated tests derived from DAG structure (4-bucket model)
- [ ] No hidden env access (resource declarations required)

---

## Appendix: Scenario → DSL Construct Matrix

For each workflow: what DSL constructs are required, what modeling gaps are likely to surface, what parity gates to run, and what gets deleted in the final unify.

### S1: Makegen (Content Upsert)

| Dimension | Details |
|---|---|
| **DSL constructs** | `type`, `pattern content_upsert`, `uses fs: Filesystem(mode: Write)`, `journey` |
| **Modeling gaps** | File resource mode (Write vs ReadWrite), skip-if-unchanged semantics at IR level |
| **Parity gates** | 8 nodes, 10 edges, ProgressManifest with 4 waves, DryRun completes |
| **Deletes in unify** | `gunbc-dag/src/makegen/graph.rs` (137 lines), makegen registration in `all_tools()` |

### S2: Credential Chain (GCP)

| Dimension | Details |
|---|---|
| **DSL constructs** | `service gcp.SecretManager`, `resource Credential`, `match runtime`, `when service_account` |
| **Modeling gaps** | Hermeticity classification, OIDC→STS→impersonation chain semantics, resource expiry |
| **Parity gates** | Transport triplets for all 8 API calls, branch coverage for 3 runtime variants, resource lifecycle nodes |
| **Deletes in unify** | `lib/gcp-ops/src/graph.rs` (1,688 lines), `SecretManagerService` trait (180 lines) |

### S3: Tool Install (Upsert)

| Dimension | Details |
|---|---|
| **DSL constructs** | `pattern upsert`, shell service calls, `when !check.exists` guard |
| **Modeling gaps** | Platform-specific tool paths, version resolution semantics |
| **Parity gates** | 3 nodes (check/create/resolve), guard branch coverage, skip-path test |
| **Deletes in unify** | Per-tool builder code, `#[tool_target]` proc macro dispatch |

### S4: Gist Snapshot (Composition)

| Dimension | Details |
|---|---|
| **DSL constructs** | `for file in files`, journey composition (SubDag), `service git.Core`, `service github.Gist` |
| **Modeling gaps** | Loop scatter points in ProgressManifest, multi-mode journey variants |
| **Parity gates** | 3 modes compile, scatter progress renders, SubDag boundaries match |
| **Deletes in unify** | `lib/tools/gist/` builder code (1,449 lines across 3 modes) |

### S5: CI Pipeline

| Dimension | Details |
|---|---|
| **DSL constructs** | `pipeline`, `stage`, `after`, `parallel`, `aggregate` |
| **Modeling gaps** | Bootstrap constraint (CI runs codegen, can't depend on generated code), stage group progress |
| **Parity gates** | 133 obligations match, stage ordering correct, parallel execution within stages |
| **Deletes in unify** | CI builder (920 lines), manual stage group wiring |

### S6: LLM Review

| Dimension | Details |
|---|---|
| **DSL constructs** | `service llm.OpenAI`, credential reuse (`credential_chain` SubDag), `@rest` for API |
| **Modeling gaps** | New service declaration pattern, API response parsing, streaming vs batch |
| **Parity gates** | Service call transport triplets, credential SubDag reuse, DryRun interception |
| **Deletes in unify** | `lib/review/` builder code (1,376 lines), LLM service trait |

### S7: Auth Flow (Interactive)

| Dimension | Details |
|---|---|
| **DSL constructs** | `@interactive`, `CaptureMode::Passthrough`, browser `resource`, platform detection |
| **Modeling gaps** | Progress pause/resume during OAuth, platform resource modeling, stdin passthrough |
| **Parity gates** | Interactive node correctly configured, progress display pauses, credential stored |
| **Deletes in unify** | Manual progress group declarations, passthrough wiring code |

---

## Appendix: Key Metrics (from internal docs)

Track these during migration to measure progress:

| Metric | Current | Target |
|---|---|---|
| Manual tool registrations | 6 islands | **0** (filesystem discovery) |
| Stringly `GraphBuilderId` references | ~20 | **0** (module graph) |
| Rendering systems without IR/trait | 13 systems, 5 traits | **0** (manifest-driven) |
| `format!()` constructing source code | Multiple crates | **0** (`TestFile` IR) |
| Hand-wired graph builder lines | ~7,000+ | **~300** (pure logic only) |
| Test generation ratio | 73% generated | **95%+** generated |
| Lines to add a new tool | ~200 across 2 files | **~20 in 1 `.dag` file** |

---

## References

- [DSL Design Document](./dsl-design.md) — full language specification and worked examples
- [Consolidation Plan](./consolidation-plan.md) — internal reconciliation and refactor tracking
- [Unified Registration](./unified-registration.md) — discovery unification design
- [Unified Emission](./unified-emission.md) — emission system consolidation design
- [Handbook](../handbook.md) — pipeline invariants and pattern catalog
