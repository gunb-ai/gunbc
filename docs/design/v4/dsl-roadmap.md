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
│                  + Discovery      Resources +      Loops +    + Backend │
│                                   Multi-Cloud      Providers            │
│  ─ compiler      ─ types          ─ service decl   ─ for      ─ stages  │
│    entrypoint    ─ funcs          ─ @rest/@shell   ─ SubDag   ─ parall. │
│  ─ module graph  ─ patterns       ─ match/when     ─ scatter  ─ Go/Py   │
│  ─ parity        ─ resources      ─ resource LC    ─ TUI                │
│    harness       ─ manifest       ─ interface      ─ AWS/Azure          │
│                                   ─ implements     ─ cross-provider     │
│                                   ─ collection ops                      │
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
| **credential (AWS)** | SubDag (internal) | runtime, role_arn | OIDC → STS | session expiry | AWS STS API calls | `AwsSessionCredentials` |
| **credential (Azure)** | SubDag (internal) | runtime, tenant_id | federated identity | token expiry | Azure AD API calls | `AzureAccessToken` |
| **infra bootstrap** | CLI | environment config | `resource` acquire | idempotent ensure | GCP/AWS/Azure APIs | `{ ready: Bool }` |
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

### Development philosophy: visualize before you implement

> Before writing code for any phase, you should be able to **see** the DAG shape you're targeting, the ProgressManifest it produces, and the modeling changes implied. Visualization is the feedback loop that lets you constantly reassess whether the IR, progress model, and rendering are complementary — before decisions are baked in.

This means:
1. **Visualization tooling lands in Phase 0** (before any workflow is compiled)
2. **Every phase starts by writing the `.dag` file and visualizing it** — confirm the DAG shape, the manifest, the test obligations, and the transport triplets *before* implementing the compiler pass that produces them
3. **Modeling changes are documented before code** — each phase has a "modeling preview" step: write the `.dag`, run `dag viz` / `dag expand` / `dag manifest`, compare to the existing builder output, identify the gaps, then implement

This inverts the usual "implement then visualize" flow. You see what you're building before you build it.

---

### Phase 0: Compiler Scaffolding + Visualization Tooling

> **Goal**: Make the DSL a first-class producer of *gunbc's existing IR* (`Dag`/`Node`/`Port`/`Edge`), and give yourself the tools to see what the compiler produces at every step.

| Deliverable | Description |
|---|---|
| `core/daglang/` workspace area | New crate that discovers `.dag` files from a project manifest + filesystem layout |
| Module graph | Resolve imports, produce a dependency-ordered module graph |
| Compiler entrypoint API | `compile_project() -> { typed_ir, progress_manifest, test_obligations, tool_metadata }` |
| Parity harness | Framework to prove compiled `.dag` output is equivalent to existing hand-wired builders (see Part 2, Workstream A) |
| IR serialization + snapshot tests | Stable IR serialization format; snapshot tests detect accidental semantic changes |
| DSL versioning | `dsl_version` in project manifest so semantics can evolve safely |
| **`dag viz` CLI** | **ASCII DAG visualization from compiled IR (static, pre-execution). Shows nodes, edges, ports, SubDag boundaries, waves.** |
| **`dag expand` CLI** | **Show lowered GraphIR: every Node, Edge, Port after pattern expansion and lowering.** |
| **`dag manifest` CLI** | **Show derived ProgressManifest: topology, SubDag boundaries, parallel groups, scatter points, labels.** |
| **`dag modules` CLI** | **Show the discovered module graph: all `.dag` files, their imports, dependency order.** |

**Visualization examples (what Phase 0 should produce)**:

`dag viz tools/makegen.dag`:
```
  fs_env ──────┐
               ├──→ render_makefile ──→ prepare_read ──→ execute_read
  load_registry┘                                              │
                                                              ▼
                                          prepare_write ←── compare
                                              │
                                              ▼
                                         execute_write
```

`dag manifest tools/makegen.dag`:
```
ProgressManifest:
  total_nodes: 8
  waves:
    [0] fs_env, load_registry
    [1] render_makefile, prepare_read
    [2] execute_read
    [3] compare, prepare_write
    [4] execute_write
  subdag_boundaries: (none)
  parallel_groups:
    [0] {fs_env, load_registry}
  scatter_points: (none)
  interactive_nodes: (none)
```

`dag expand tools/makegen.dag`:
```
Node  fs_env                    Opaque(FsEnv)           ports: [] → [FilesystemHandle]
Node  load_registry             Opaque(LoadRegistry)    ports: [] → [ToolRegistry]
Node  render_makefile           Fn(render_makefile)     ports: [ToolRegistry] → [String]
Node  prepare_read_makegen      Opaque(PrepareFileRead) ports: [FilesystemHandle] → [ReadSpec]
Node  execute_read_makegen      Transport(Execute)      ports: [ReadSpec] → [FileContent]
Node  compare_makegen_content   Fn(compare_content)     ports: [String, FileContent] → [Bool]
Node  prepare_write_makegen     Opaque(PrepareFileWrite) ports: [String, FilesystemHandle] → [WriteSpec]
Node  execute_makegen_transport Transport(Execute)      ports: [WriteSpec] → [Written]

Edge  fs_env.FilesystemHandle       → prepare_read_makegen.FilesystemHandle
Edge  fs_env.FilesystemHandle       → prepare_write_makegen.FilesystemHandle
Edge  load_registry.ToolRegistry    → render_makefile.ToolRegistry
Edge  render_makefile.String        → compare_makegen_content.String
Edge  render_makefile.String        → prepare_write_makegen.String
...
```

**Acceptance Gates**

- [x] "Compile-only" smoke test discovers `.dag` files and reports modules without executing anything — *`check` command runs pipeline to `Parse` and prints diagnostics*
- [x] Can compile one `.dag` file into a valid gunbc IR structure (even with stubby node bodies) — *`compile_from_context()` returns `Dag<LoweredOp>` via discover → typecheck → lower → derive → emit*
- [~] Parity harness can canonicalize and diff two IR graphs — *`compare_topology()` returns `ParityReport` with node/edge deltas; `compare_makegen_topology()` adds normalization rules. Needs expansion from topology-only to full IR shape (ports, node kinds, labels).*
- [ ] IR snapshot test passes for at least one compiled `.dag` file — **Missing: no insta/snapshot-style tests yet**
- [~] **`dag viz` produces ASCII graph for at least one `.dag` file** — *Implemented via Mermaid output (`to_mermaid`), not ASCII. Needs ASCII default with `--format mermaid` flag.*
- [~] **`dag expand` produces full node/edge/port listing matching the existing builder IR** — *Command exists with text listing of nodes/edges/ports. Needs golden tests for output stability and verification against builder IR.*
- [~] **`dag manifest` produces ProgressManifest matching expected topology** — *Command exists but manifest struct is `{total_nodes, total_edges, waves, entrypoint_nodes, boundary_nodes}` — does not match the roadmap contract (missing: topology list, labels, subdag_boundaries, parallel_groups, scatter_points, capture_modes, stage_groups, resources). See Bridge Milestone below.*
- [x] **`dag modules` shows the discovered module graph** — *`modules [dir]` runs pipeline to `Report` and prints dependency-ordered module listing*

**Status: ~75% complete.** Core compiler spine + CLI scaffolding are in place. Remaining gaps: viz format mismatch, manifest contract mismatch, IR snapshot tests, parity harness needs full IR shape comparison.

**Why first**: visualization is not a nice-to-have that ships later. It's the development tool that makes every subsequent phase implementable. If you can't see the DAG, the manifest, and the lowered IR, you're implementing blind. The design doc's Appendix M explicitly says "essential for trust and debugging from day one."

---

### Phase 0.5: Modeling Preview (before Phase 1 code)

> **Goal**: Write `.dag` files for the first few workflows, visualize them against the existing builders, and document the modeling changes *before* implementing the compiler passes.

For each workflow being targeted in Phases 1-2:

1. **Write the `.dag` file** (even if the compiler can't fully process it yet — the parser should handle the syntax)
2. **Run `dag viz`** on the existing builder IR (from the parity harness) to see the current shape
3. **Sketch `dag viz`** for the `.dag` version to see the target shape
4. **Compare the two** and document:
   - Nodes that map 1:1
   - Nodes the compiler will insert (resource acquisition, pattern expansion, transport triplets)
   - Modeling gaps (semantics that exist in the builder but not in the `.dag`, or vice versa)
   - Progress model differences (SubDag boundaries, parallel groups, scatter points)
5. **Write a one-page "modeling preview"** per workflow: current shape → target shape → gaps → plan

| Workflow | `.dag` file to write | Visualization comparison |
|---|---|---|
| makegen | `tools/makegen.dag` | 8 nodes, 10 edges — simplest; validates `content_upsert` pattern expansion |
| clippy | `tools/clippy.dag` | `upsert` pattern; validates guard/skip nodes |
| credential | `cloud/gcp/credential.dag` | 8 transport triplets; validates service → triplet expansion |
| gist | `tools/gist.dag` | Loop nodes, SubDag composition; validates nested manifest |

**Deliverables**:
- [x] `.dag` files written for at least makegen + one other workflow — *DSL corpus exists under `dsl/` with modules spanning tools, services, pipelines, infra, cloud, and examples*
- [ ] Side-by-side `dag viz` comparison (existing builder vs target `.dag` shape)
- [ ] Modeling preview document per workflow (gaps, insertions, manifest differences)
- [~] `dag show-triplets` works (shows service call → prepare/execute/parse expansion) — *Not a CLI command yet; triplet derivation exists in obligation counting logic*
- [~] `dag obligations` works (shows 4-bucket test obligations derived from DAG) — *`TestObligations` is derived and rendered by `dag manifest`, but not a standalone command; obligation counts are present but CLI UX needs work*

**Status: ~30% complete.** The corpus exists and obligation counting works, but the developer-facing CLI commands (`show-triplets`, `obligations`) and modeling preview documents are missing.

**Why this step**: you surface modeling gaps *before* you've committed to an implementation. If the `.dag` shape doesn't match the builder shape, or the manifest is missing information the renderer needs, you catch it here — not after weeks of compiler work.

---

### Bridge Milestone: Phase 0 → Phase 1 (parallel workstreams)

> **Goal**: Close the remaining Phase 0 acceptance gates and establish the shared contracts that Phase 1 depends on. Structured as 6 independent workstreams so multiple contributors can work in parallel with minimal merge conflicts.

**Context (February 2026 reconciliation)**: The compiler pipeline spine (discover → typecheck → lower → derive → emit) is functional and the CLI commands exist. However, several Phase 0 acceptance gates are only partially met, and the Phase 1 acceptance gates (makegen IR parity, manifest contract, test parity) depend on closing these gaps first.

The main structural mismatch is the **ProgressManifest**: the roadmap contract specifies `topology`, `labels`, `subdag_boundaries`, `parallel_groups`, `scatter_points`, `capture_modes`, `stage_groups`, and `resources`, but the current implementation has `{total_nodes, total_edges, waves, entrypoint_nodes, boundary_nodes}`. This must be resolved before Phase 1 gates can be evaluated.

#### Workstream A — Manifest Contract + Derive Correctness

**Goal:** Bring `daglang_derive::ProgressManifest` into exact alignment with the roadmap contract.

**Scope:** `daglang-derive` + `dag manifest` rendering.

| Deliverable | Details |
|---|---|
| Manifest struct expansion | Add `topology: Vec<TopologyNode>`, `labels: Map<NodeId, String>`, `subdag_boundaries`, `parallel_groups`, `scatter_points`, `interactive_nodes`, `capture_modes`, optional `stage_groups`, `resources` |
| Derivation from lowered DAG | Compute depth (wave) and parent for SubDag boundaries; derive labels from DSL identifiers; derive parallel groups (siblings at same depth with no ordering constraints); derive capture modes from annotations/op kinds |
| `dag manifest` JSON output | Add `--format json` (recommended for stability + snapshot tests); keep pretty text view layered on top of the contract object |

**Dependencies:** None. **PR strategy:** Land first — other workstreams consume the contract.

#### Workstream B — Visualization + `dag expand` UX

**Goal:** Make the Phase 0 CLI outputs match the acceptance gates.

**Scope:** `daglang-viz` / `core/ir/src/dag.rs` + CLI formatting.

| Deliverable | Details |
|---|---|
| `dag viz` ASCII default | Add ASCII rendering as default (roadmap asks for ASCII). Keep Mermaid behind `--format mermaid`. |
| `dag expand` stability | Ensure deterministic, full listing: nodes, input/output ports, edges, SubDag boundaries. Add golden tests. |

**Dependencies:** Can proceed in parallel; benefits from Workstream A's manifest for SubDag boundary display.

#### Workstream C — Parity Harness + IR Snapshot Tests

**Goal:** Finish the Phase 0 acceptance gate around parity + snapshots, and set up the Phase 1 parity target.

**Scope:** `daglang-lower` parity module + new snapshot test infrastructure.

| Deliverable | Details |
|---|---|
| Expand parity to full IR shape | Beyond topology: compare ports (names + type IDs), node kinds (at least as a tag), SubDag structure. |
| Canonical JSON serialization | Stable JSON format for lowered DAG + derived manifest (stable sort, ID normalization). |
| Snapshot tests | Per-module IR snapshots (at minimum for `tools/makegen.dag`). Detect accidental semantic changes. |

**Dependencies:** Minimal. Stable sorting/canonicalization rules are independent of other workstreams.

#### Workstream D — Discovery / Module-Path Consistency

**Goal:** Align discovery with Phase 1 "no manual lists", robust multi-root projects.

**Scope:** `daglang-resolve` + CLI.

| Deliverable | Details |
|---|---|
| Config-driven discovery | Project manifest behavior so roots are driven by config, not just `cwd/dsl`. |
| Module-path consistency enforcement | Strengthen `validate_module_path_consistency()` for whole-root compilation. |
| `dag modules` enrichment | Include dependency order, unresolved import diagnostics, cycle detection summaries. |

**Dependencies:** None. Mostly `daglang-resolve` + CLI.

#### Workstream E — Phase 0.5 "Model Preview" Commands

**Goal:** Land the Phase 0.5 developer-facing tools.

**Scope:** CLI + derive presentation.

| Deliverable | Details |
|---|---|
| `dag show-triplets <file.dag>` | Show pre/post lowering expansion (especially prepare/execute/parse triplets). |
| `dag obligations <file.dag>` | Output the 4-bucket obligations summary the roadmap calls for. |
| Modeling preview docs | "What `.dag` looks like vs what IR looks like" examples for makegen + one other workflow. |

**Dependencies:** Can proceed immediately — obligation counts already derived.

#### Workstream F — Phase 1 Gate: Makegen Parity

**Goal:** Satisfy Phase 1 acceptance: compiled makegen matches existing builder IR shape and tests line up.

**Scope:** Parity harness integration + execution path.

| Deliverable | Details |
|---|---|
| IR parity test | Compile `tools/makegen.dag` → lowered IR, build existing Rust makegen IR → compare using parity harness (Workstream C). |
| ProgressManifest parity | Ensure derived manifest matches the shared contract shape and has expected topology/labels/groups for makegen. |
| Execution path | Compile → execute IR directly with existing runtime (quickest path to end-to-end proof). |

**Dependencies:** Needs Workstream A (manifest contract) and Workstream C (parity + canonicalization).

#### PR / Merge Strategy

Stack PRs by "API provider → API consumers" to minimize conflicts:

1. **PR 1 (Workstream A skeleton):** new manifest contract structs + `dag manifest` JSON output
2. **PR 2 (Workstream C snapshots):** canonical JSON + snapshot infra using the new manifest
3. **PR 3 (Workstream B viz/expand):** ASCII viz + expand formatting consuming the manifest
4. **PR 4 (Workstream E commands):** show-triplets / obligations
5. **PR 5 (Workstream F parity):** parity harness comparing compiled makegen vs builder makegen
6. **PR 6 (Workstream D discovery):** config-driven discovery improvements

#### Bridge Milestone Acceptance Gate

The bridge is complete when:

- [ ] `dag viz` defaults to ASCII and is deterministic
- [ ] `dag manifest` emits the **contract ProgressManifest** (JSON)
- [ ] IR snapshot tests exist for `tools/makegen.dag`
- [ ] Parity harness can compare compiled makegen vs builder makegen and report diffs
- [x] `dag obligations` and `dag show-triplets` exist (even if obligations are initially "best effort")

This set unblocks Phase 1 without risking a giant "run makegen end-to-end" rewrite before the compiler outputs are stable.

---

### Execution Plan: Two-Worker Bridge → Phase 1

> **Context**: The Bridge Milestone's 6 workstreams have a real dependency graph. Two parallel workers can complete the bridge and Phase 1 without stepping on each other, because the work splits cleanly into "critical-path contracts" (manifest, parity, execution) and "code quality" (pure helpers, typed pipeline, structural obligation classification). Visualization tooling is explicitly deferred — the `.dag` source is self-documenting, and Mermaid output already exists for when a picture is needed. Each worker owns distinct crates and files.

#### Why this matters

The compiler pipeline *works* — it compiles `.dag` files to `Dag<LoweredOp>` with correct topology. But three things are missing before we can say "the DSL produces the same thing as the hand-wired builders":

1. **The manifest contract doesn't match the spec.** Renderers, tests, and parity comparisons all need the full `ProgressManifest` shape — topology, labels, SubDag boundaries, parallel groups. Without this, Phase 1's acceptance gates can't even be evaluated.
2. **Parity is topology-only.** We can diff node/edge counts, but not ports, node kinds, or SubDag structure. The parity harness needs to compare *full IR shape* to prove equivalence.
3. **No execution path.** The compiled `Dag<LoweredOp>` has the right shape but `LoweredOp` doesn't implement `Executable`. We need a dispatch layer that maps DSL-declared names to the existing concrete ops (`MakegenOp`, `TransportOps`, etc.) so the existing `execute_dag()` can run compiled output.

Each of these feeds the next. The manifest contract defines what "correct" looks like. The parity harness proves we match it. The execution path proves it actually runs. That's Worker 1's job.

Meanwhile, the compiler codebase itself has structural debt — impure helpers, dynamic pipeline execution, stringly-typed obligation classification, and large monolithic files. Cleaning this up now (while the crate APIs are still young) prevents the debt from compounding as Phase 1+ features land. That's Worker 2's job: make the codebase clean, modular, and honest before it gets bigger.

---

#### Worker 1: Critical Path — Contracts → Parity → Execution

**Mission**: Make the compiler's output *provably equivalent* to the hand-wired builders, then make it *executable*. This is the spine of the entire migration — everything in Part 2 (porting workflows) depends on this worker's output being correct and stable.

**Crates owned**: `daglang-derive`, `daglang-lower` (parity module), `daglang-emit`, `daglang-cli/src/compile.rs`

**Why you're first**: Nothing else matters if the compiled IR doesn't match the builder IR. Viz is nice. Commands are nice. But if the parity proof fails, we're building on sand. Your job is to make the foundation solid.

##### Step 1: Manifest Contract (Workstream A)

Expand `daglang_derive::ProgressManifest` to match the roadmap contract. The current struct has `{total_nodes, total_edges, waves, entrypoint_nodes, boundary_nodes}`. The contract requires:

| Field | Type | Derived from |
|---|---|---|
| `topology` | `Vec<TopologyNode>` | Node IDs + depth (wave index) |
| `labels` | `HashMap<NodeId, String>` | DSL identifiers (`module.name`) |
| `subdag_boundaries` | `Vec<SubDagBoundary>` | `NodeBody::SubDag` nodes in the lowered DAG |
| `parallel_groups` | `Vec<ParallelGroup>` | Siblings at same depth with no ordering edge |
| `scatter_points` | `Vec<NodeId>` | Loop expansion nodes (Phase 3, stub empty for now) |
| `interactive_nodes` | `Vec<NodeId>` | `@interactive` annotated nodes (Phase 3, stub empty) |
| `capture_modes` | `HashMap<NodeId, CaptureMode>` | Transport nodes → `Captured`, others → default |
| `stage_groups` | `Vec<StageGroup>` | Pipeline stages (Phase 4, stub empty) |
| `resources` | `HashMap<NodeId, Vec<ResourceUsage>>` | `uses`/`provides` clauses from typed project |

Add `dag manifest --format json` for stable, machine-readable output. Keep the existing text rendering as the default, layered on the contract object.

**Acceptance criteria**:
- [ ] `ProgressManifest` struct has all contract fields (Phase 3/4 fields can be empty `Vec`s)
- [ ] `derive_artifacts()` populates `topology`, `labels`, `parallel_groups` correctly for makegen
- [ ] `dag manifest tools/makegen.dag` produces the expected 8-node, 4-wave manifest
- [ ] `dag manifest --format json tools/makegen.dag` produces stable, parseable JSON
- [ ] All existing tests pass (the struct expansion must be backward-compatible)

##### Step 2: Parity Infrastructure (Workstream C)

Expand the parity harness beyond topology-only comparison. Currently `compare_topology()` counts nodes/edges and reports deltas. It needs to compare:

| Comparison | What to diff |
|---|---|
| Ports | Input/output port names and type IDs per node |
| Node kinds | At minimum a tag: `callable`, `transport`, `pattern-expanded`, `resource-lifecycle` |
| SubDag structure | Which nodes are inside SubDags, boundary nesting |
| Labels | DSL-derived labels match builder-derived labels |

Add canonical JSON serialization for the lowered DAG (stable sort by node ID, normalized edge ordering). This becomes the snapshot format.

Create IR snapshot tests: compile `tools/makegen.dag`, serialize to canonical JSON, compare against a checked-in snapshot. Any change to the compiler that alters the IR shape fails the test — forcing explicit acknowledgment.

**Acceptance criteria**:
- [ ] `compare_topology()` (or a new `compare_ir()`) diffs ports, node kinds, and labels — not just counts
- [ ] `ParityReport` includes per-node detail: which nodes differ and how
- [ ] Canonical JSON serialization for `Dag<LoweredOp>` is deterministic (same input → same bytes)
- [ ] At least one IR snapshot test exists for `tools/makegen.dag` and passes
- [ ] Snapshot test is in CI (fails if compiler changes alter makegen IR)

##### Step 3: Makegen Parity + Execution (Workstream F)

Wire the parity harness as a real test: compile `tools/makegen.dag` → lowered DAG, load the hand-wired `build_makegen_graph()` reference DAG, compare using the expanded parity harness. Get the delta to zero.

Then build the dispatch layer: a registry that maps `LoweredOp` descriptions to concrete `Executable` implementations. For makegen, this is ~8-10 entries:

| `LoweredOp::Callable` | Maps to |
|---|---|
| `tools.makegen::render_makefile` | `MakegenOp::RenderMakefile` |
| `tools.makegen::makegen` | `MakegenOp::Makegen` |
| `load_registry` | `MakegenOp::LoadRegistry` (or equivalent) |
| `fs_env` | Environment node producing `FilesystemHandle` |
| `prepare_read_makegen` | `PrepareFileReadOp` |
| `execute_read_makegen` | `TransportOps::Execute` |
| `compare_makegen_content` | `FreshnessStep` |
| `prepare_write_makegen` | `PrepareFileWriteOp` |
| `execute_makegen_transport` | `TransportOps::Execute` |

Write `fn resolve_dag(dag: Dag<LoweredOp>, registry: &OpRegistry) -> Result<Dag<Box<dyn Executable>>, ResolveError>` that walks the compiled DAG, replaces each `LoweredOp` node with its concrete `Executable`, and preserves all edges/ports.

Then execute: pass the resolved DAG to the existing `execute_dag()` in `core/exec`. Verify it produces the same Makefile output as running the hand-wired builder.

**Acceptance criteria**:
- [ ] Parity test: compiled makegen IR matches builder IR (zero delta in `ParityReport`)
- [ ] `OpRegistry` exists with entries for all makegen nodes
- [ ] `resolve_dag()` converts `Dag<LoweredOp>` → `Dag<Box<dyn Executable>>` for makegen
- [ ] End-to-end test: `compile → resolve → execute_dag()` produces valid Makefile output
- [ ] DryRun mode works: compiled makegen executes in DryRun with transport interception
- [ ] Existing `make test-all` still passes (no regressions)

##### Worker 1 Definition of Done

All three steps complete. The sentence "compile `tools/makegen.dag` and run it, producing the same Makefile as the hand-wired builder" is true and proven by tests.

---

#### Worker 2: Code Quality — Pure Helpers, Typed Pipeline, Modular CLI

**Mission**: Make the compiler codebase *clean and honest* before it gets bigger. Phase 1+ will add service transport, resource lifecycle, pattern expansion, and multi-backend emission — each touching multiple crates. If the foundation has impure helpers, dynamic typing where static typing would do, and 3000-line files, every future change is harder than it needs to be. Your job is to pay down the structural debt now, while the crate boundaries are still young.

**Crates owned**: `daglang-cli/src/pipeline.rs`, `daglang-cli/src/main.rs`, `daglang-cli/src/path_utils.rs`, `daglang-resolve/src/lib.rs`, `daglang-lower/src/lib.rs` (obligation classification only)

**Why you matter**: Worker 1 is building new capabilities (manifest contract, parity harness, execution dispatch). Every line they write inherits the code patterns that exist today. If helpers are impure, Worker 1 will write impure helpers. If the pipeline runner uses string-keyed HashMaps, new pipelines will too. You're setting the standard that all future code follows. Get it right now and everything downstream is cleaner.

##### Step 1: Pure Helper Libraries

The codebase has several helpers that call `std::env::current_dir()` internally, making them impure and environment-dependent. The fix is straightforward: effectful boundary at `main()`, pure core everywhere else.

**`path_utils.rs`**: `resolve_default_root()` and `normalize_cli_path()` both call `current_dir()`. Refactor them to take `cwd: &Path` as a parameter. Have `main.rs` call `current_dir()` once at startup and thread `cwd` down through command dispatch.

**`compile.rs`**: `build_context()` calls `resolve_default_root()` which calls `current_dir()`. After the `path_utils` refactor, `build_context()` should take `cwd: &Path` too.

**`pipeline.rs`**: `collect_dag_files()` constructs paths relative to an implicit cwd. Make the root path an explicit parameter.

The pattern: every function in the `daglang-cli` crate below `main()` should be a pure function of its arguments. I/O happens exactly at the boundary — `main()` reads the environment, commands write to stdout/stderr.

**Acceptance criteria**:
- [x] `path_utils::resolve_default_root(cwd: &Path)` — no `current_dir()` call
- [x] `path_utils::normalize_cli_path(cwd: &Path, path: &Path)` — no `current_dir()` call
- [x] `main.rs` calls `current_dir()` exactly once, threads `cwd` to all subcommands
- [x] `build_context()` takes `cwd: &Path`, not implicitly reading environment
- [x] No function in `path_utils.rs`, `compile.rs`, or `pipeline.rs` calls `std::env::current_dir()`
- [x] All existing tests pass (behavior is identical, only the call site of `current_dir()` moved)

##### Step 2: Typed Pipeline Runner

`pipeline.rs` runs the 4-stage compiler pipeline (discover → parse → build graph → report) through a `HashMap<String, PipeValue>` with string keys and 5 `take_*` extraction functions. This is dynamic typing in a statically typed language — a mismatch with the "imperative pure functions" style.

The `build_pipeline_dag()` function that constructs the pipeline as a `Dag<CompilerOp>` should stay — it powers `dag viz --self` and is structurally interesting. But *execution* should be a plain imperative function with typed locals:

```rust
fn run_pipeline(context: &PipelineContext, stop: PipelineStop) -> Result<PipelineResult, PipelineError> {
    let files = discover_dag_files(&context.roots)?;
    if stop == PipelineStop::Discover { return Ok(PipelineResult::Discovered(files)); }

    let (modules, diagnostics) = parse_all_files(&files)?;
    if stop == PipelineStop::Parse { return Ok(PipelineResult::Parsed(modules, diagnostics)); }

    let graph = build_module_graph(&modules, &context.roots)?;
    if stop == PipelineStop::Build { return Ok(PipelineResult::Built(graph, diagnostics)); }

    let report = format_module_report(&graph, &diagnostics);
    Ok(PipelineResult::Reported(report))
}
```

No HashMap, no string keys, no `remove()` semantics, no fan-out ambiguity. Each stage is a pure function call. The types enforce that you can't accidentally consume a value twice or forget to produce one.

**Acceptance criteria**:
- [x] `run_pipeline()` is an imperative function with typed locals — no `HashMap<String, PipeValue>`
- [x] `PipelineResult` is a typed enum (no string-keyed values)
- [x] The 5 `take_*` functions are deleted
- [x] `build_pipeline_dag()` still exists (for `dag viz --self`)
- [x] `dag check`, `dag modules`, and `dag compile` produce identical output
- [x] All existing pipeline tests pass

##### Step 3: Obligation Classification + CLI Commands

Obligation counts are currently derived from string prefix matching (`"service_transport::prepare::"`, port type equality `"TransportRequest"`). This is brittle — a rename in the lowerer silently breaks obligation counts with no compiler error.

Add a classification function on `LoweredOp` that returns a typed enum:

```rust
enum ObligationCategory {
    Callable,
    Pipeline,
    ServiceTransport,
    ResourceLifecycle,
    PatternExpanded,
    None,
}

fn classify_obligation(op: &LoweredOp) -> ObligationCategory { ... }
```

Centralize the string constants (or better, make classification structural — based on `LoweredOp` variant, not string content). Then rewrite `derive_test_obligations()` to use this classifier instead of ad-hoc string matching.

While touching the CLI, add two standalone commands that are cheap to implement (the data already exists in `derive_artifacts()`):

- `dag show-triplets <file.dag>` — print service call → transport triplet expansion (prepare/execute/parse nodes and edges)
- `dag obligations <file.dag>` — print the 4-bucket test obligation summary (currently embedded in `dag manifest` output, extract to standalone)

**Acceptance criteria**:
- [x] `ObligationCategory` enum exists with typed variants
- [x] `classify_obligation()` is a pure function on `LoweredOp` — no string prefix matching
- [x] `derive_test_obligations()` uses `classify_obligation()` internally
- [x] Obligation counts for `tools/makegen.dag` are unchanged (behavioral parity)
- [x] `dag show-triplets tools/makegen.dag` shows the content_upsert triplet expansion
- [x] `dag obligations tools/makegen.dag` shows the 4-bucket obligation summary
- [x] Both new commands support `--format json`

##### Worker 2 Definition of Done

The compiler codebase follows the "effectful boundary, pure core" principle. Every function below `main()` is a pure function of its arguments. The pipeline runner is typed and imperative. Obligation classification is structural, not stringly-typed. The two CLI commands (`show-triplets`, `obligations`) work and are useful for parity debugging.

**Deferred (explicitly not in scope)**:
- ASCII viz renderer — the `.dag` source is self-documenting; Mermaid exists for when you need a picture
- File splitting of `compile.rs` / `pipeline.rs` — do this when the files actually cause merge conflicts, not preemptively
- `daglang.toml` config-driven discovery — premature until we have more than one project consuming it
- Enriched `dag modules` (dependency counts, inline warnings) — nice-to-have, not blocking Phase 1

---

#### Coordination Protocol

The two workers share the `daglang-cli` crate but touch different files:

| File | Worker 1 | Worker 2 |
|---|---|---|
| `daglang-derive/src/lib.rs` | **owns** (manifest expansion) | reads (obligation refactor touches `derive_test_obligations`) |
| `daglang-lower/src/lib.rs` | **owns** (parity module) | reads (obligation classification on `LoweredOp`) |
| `daglang-emit/src/lib.rs` | **owns** (execution path) | — |
| `daglang-cli/src/compile.rs` | **owns** (resolve_dag, execution) | touches (cwd threading into `build_context`) |
| `daglang-cli/src/main.rs` | — | **owns** (cwd threading, new commands) |
| `daglang-cli/src/pipeline.rs` | — | **owns** (typed runner refactor) |
| `daglang-cli/src/path_utils.rs` | — | **owns** (pure helper refactor) |
| `daglang-resolve/src/lib.rs` | — | — (stable) |
| `daglang-syntax/` | — | — (stable, neither touches) |
| `daglang-typecheck/` | — | — (stable, neither touches) |

**Potential overlap**: Worker 2's `cwd` threading touches `compile.rs` (changing `build_context` signature), which Worker 1 also owns. Resolve this by having Worker 2 do the `cwd` refactor first (it's mechanical, no new logic), then Worker 1 builds on the new signature.

**Sync points**:
1. Worker 2 completes Step 1 (pure helpers + cwd threading) first. This changes function signatures in `path_utils.rs` and `compile.rs` that Worker 1 depends on. Merge this before Worker 1 starts Step 3.
2. Worker 2's Step 3 (obligation classification) touches `LoweredOp` typing in `daglang-lower`. Coordinate with Worker 1 if they're simultaneously changing lowering logic for parity.
3. After Worker 1 completes Step 3 (execution), Worker 2's `dag show-triplets` can optionally show dispatch mappings using the `OpRegistry`.

**Branch strategy**: Each worker gets their own feature branch off the current branch. Merge Worker 2 Step 1 first (signature changes), then both proceed independently. Worker 2 Step 3 and Worker 1 Step 2-3 should coordinate on `daglang-lower` if running concurrently.

---

### Phase 1: Language Core + Discovery + ProgressManifest

> **Proving workflow**: `makegen` (scenario S1 — simplest complete graph)

| Construct | What it covers |
|---|---|
| Minimal type system | Records, enums/sums, `T?`, `List<T>`, `Map<K,V>` — sufficient to typecheck `makegen` |
| `func` syntax | Implicit edges via references |
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

### Phase 2: Services + Resources + Multi-Cloud Modeling

> **Proving workflows**: `acquire_gcp_secret` (scenario S2), `GcsBucket implements ObjectStorage` (scenario S8 — infra as resources), `store_artifact(uses store: ObjectStorage)` (abstract interface usage)

| Construct | What it covers |
|---|---|
| `service` declarations | `operation input/output` with transport annotations (`@rest`, `@shell`) |
| `match` / `when` | Runtime branching (GitHub Actions vs Metadata vs Local), guarded ports |
| `resource` with lifecycle | `Credential`, `Network`, `GcsBucket`, `ManagedSecret` — acquire/use/release |
| `interface` declarations | Abstract infrastructure contracts (`ObjectStorage`, `Compute`, `SecretStore`, `Identity`, `Queue<T>`) with `@contract` behavioral annotations |
| `resource X implements Y` | Concrete resources implementing abstract interfaces (GCP provider) |
| `CloudConfig` sum type | Provider selection at compile time (`GcpConfig \| AwsConfig \| AzureConfig`) |
| Late-bound transport | Semantic metadata survives lowering (avoids "generic IR chokepoint") |
| Collection ops as IR nodes | `map`, `filter`, `fold` in `fn` bodies lower to `MapNode`, `FilterNode`, `FoldNode` — data-parallel execution |

**Deliverables**

| Deliverable | Artifact |
|---|---|
| `.dag` for credential chain | `cloud/gcp/credential.dag` (scenario S2) |
| Abstract infra interfaces | `infra/core.dag`: `ObjectStorage`, `Compute`, `SecretStore`, `Identity`, `Queue<T>` |
| GCP resource implementations | `infra/gcp/resources.dag`: `GcsBucket : ObjectStorage`, `CloudRunService : Compute`, etc. |
| Transport triplet emission | Service calls → prepare/execute/parse, without authors ever writing triplets |
| Semantics carrier | Service metadata (hermeticity, idempotency, permissions) survives to Derive/Validate |
| Collection IR lowering | `fn` body `|> map/filter/fold` lowers to IR-level collection nodes for data-parallel execution |

**Acceptance Gates**

- [ ] **Classification**: calls classified (local git shell vs network REST) from service declarations, not from generic `TransportRequest` variants
- [ ] **Resource lifecycle**: acquisition/release nodes inserted by compiler; resource conflicts detected during validation
- [ ] **IR parity**: compiled credential chain matches existing `lib/gcp-ops/src/graph.rs` shape
- [ ] **Semantic preservation**: `@idempotent`, `@readonly`, `@permissions` annotations survive lowering and are accessible to test categorizer
- [ ] **Interface resolution**: `uses store: ObjectStorage` resolves to `GcsBucket` when `CloudConfig = GcpConfig`
- [ ] **Contract verification**: `@contract` annotations on `ObjectStorage` generate behavioral tests for `GcsBucket`
- [ ] **Collection parallelism**: `list |> map(f)` inside `fn` compiles to `MapNode` in IR; executor can parallelize

**Corresponds to**: [dsl-design.md Phase 2](./dsl-design.md#phase-2-services--resources--cloud-modeling), [Appendix B](./dsl-design.md#appendix-b-cloud-credential-acquisition-gcp), [§7.6](./dsl-design.md#76-infrastructure-as-resources-multi-cloud)

---

### Phase 3: Composition + Loops + Multi-Provider + TUI Progress

> **Proving workflows**: `gist_snapshot` (scenario S4 — multi-service composition with loops), `cross_cloud_pipeline` (scenario S9 — cross-provider composition)

| Construct | What it covers |
|---|---|
| `for` loops | LoopBuilder equivalent |
| Func composition | SubDag expansion (func calls become SubDag nodes) |
| Scatter points | ProgressManifest includes loop expansion points for grouped counters |
| TUI progress | Manifest-driven rendering restores capabilities lost from the-gunbai |
| AWS provider resources | `S3Bucket : ObjectStorage`, `LambdaFunction : Compute`, `AwsSecret : SecretStore`, `AwsIamRole : Identity`, `SqsQueue : Queue<String>` |
| Azure provider resources | `BlobContainer : ObjectStorage`, `ContainerApp : Compute`, `KeyVaultSecret : SecretStore`, `AzureManagedIdentity : Identity`, `ServiceBusQueue : Queue<String>` |
| Cross-provider composition | Funcs that `uses` resources from different providers (e.g., GCS + S3 + SQS) |
| Multi-cloud credential chains | `cloud/aws/credential.dag` (OIDC → STS), `cloud/azure/credential.dag` (federated identity → AD) |

**Deliverables**

| Deliverable | Artifact |
|---|---|
| `.dag` for gist | `tools/gist.dag` (snapshot/diff/recent) compiles and runs |
| AWS + Azure resources | `infra/aws/resources.dag` (5 resources), `infra/azure/resources.dag` (5 resources) |
| AWS + Azure credentials | `cloud/aws/credential.dag`, `cloud/azure/credential.dag` |
| Cross-provider example | `examples/deployment.dag` `cross_cloud_pipeline` compiles and tests |
| Scatter progress | ProgressManifest includes scatter points; renderers show `read files [8/8]` |
| Static DAG viz | Visualize graph before execution (from manifest, not runtime) |

**Acceptance Gates**

- [ ] **Compression**: gist workflow expressed in ~80 lines of `.dag` (vs 1,449 lines of Rust builders)
- [ ] **Loop progress**: renderers display loop progress as grouped counter without manual configuration
- [ ] **Composition**: SubDag calls work for credential chain reuse within gist workflow
- [ ] **IR parity**: compiled gist graph matches existing builder shape for all 3 modes
- [ ] **Provider portability**: `store_artifact(uses store: ObjectStorage)` compiles against all 3 providers
- [ ] **Cross-provider auth**: each provider's credential chain resolves independently in a cross-provider func
- [ ] **Contract tests**: `@contract` behavioral tests pass for AWS and Azure implementations

**Corresponds to**: [dsl-design.md Phase 3](./dsl-design.md#phase-3-composition--tui-progress), [Appendix C](./dsl-design.md#appendix-c-service-composition-gist-snapshot), [§7.6](./dsl-design.md#76-infrastructure-as-resources-multi-cloud)

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
Phase   Proving Workflow          Deliverables                          Key Risk                  Status
─────   ─────────────────         ─────────────────────────────         ─────────────────────────  ──────
  0     (scaffolding)             Discover + Parse + Module Graph       IR integration boundary    ~75%
                                  + dag viz/expand/manifest/modules
  0.5   (modeling preview)        .dag files + side-by-side viz         Gaps found too late        ~30%
                                  + modeling preview docs
  0→1   (bridge milestone)        Manifest contract + ASCII viz         Contract mismatch          Not started
                                  + parity snapshots + model preview
                                  commands (6 parallel workstreams)
  1     makegen (S1)              types, func, pattern, resource        Pattern expansion fidelity Skeleton in place
                                  + plain/inline renderers
  2     acquire_gcp_secret (S2)   service, match/when, resource LC      Generic IR chokepoint
        GcsBucket:ObjectStorage   interface, implements, CloudConfig    Interface resolution
        (S8)                      collection ops as IR nodes
  3     gist_snapshot (S4)        for, composition, scatter progress    TUI renderer integration
        cross_cloud (S9)          + nested SubDag rendering             Cross-provider auth
                                  AWS + Azure providers
  4     CI pipeline (S5)          pipeline, stage, parallel, aggregate  Bootstrap constraint
                                  + JSONL renderer + Go backend
```

---

## Backend Architecture and Rendering Model

> Define all backends and the rendering structure **upfront** — not as an afterthought. Rendering is a first-class concern because it's how users understand what the system is doing. The nested composition model from the-gunbai must be preserved and enhanced.

### Two kinds of rendering

The system has two completely different rendering concerns that share the word "rendering" but are architecturally separate:

| Concern | What it is | Who does it | Where it lives |
|---|---|---|---|
| **Content rendering** | Producing artifacts: Makefiles, YAML, markdown, CLI output | Functors (`fn`) — pure transforms in the DAG | The `.dag` file, compiled by `CodegenBackend` |
| **Progress rendering** | Showing DAG execution: sections, spinners, status, error boxes | The framework — manifest-driven, per-mode renderers | The engine runtime, driven by `ProgressManifest` |

Content rendering is "just more functors" — no special system needed (this was Appendix G's insight: rendering doesn't need 13 systems, it needs typed pure functions). Progress rendering IS special — it's the framework's job, reading a compiler-derived manifest.

### CodegenBackend interface (updated for functor protocol)

Each codegen backend (Rust, Go, Python, TypeScript) implements one trait:

```
trait CodegenBackend {
  // Types
  fn emit_type(ty: &TypeDef) -> String                    // record, enum, alias
  fn emit_fn(f: &FnDef) -> String                         // pure functor → target language function

  // DAG wiring
  fn emit_transport(spec: &TransportSpec) -> String        // HTTP client, shell exec, file I/O
  fn emit_func(f: &FuncDef) -> String                       // DAG execution orchestrator
  fn emit_pipeline(p: &PipelineDef) -> String              // staged multi-func orchestrator

  // Testing
  fn emit_test(obligation: &TestObligation) -> String      // 4-bucket testgen
  fn emit_mock_spec(spec: &MockSpec) -> String             // from service declarations

  // Entrypoints
  fn emit_cli(entrypoints: &[Port]) -> String              // arg parsing from DAG entry ports
  fn emit_makefile_target(module: &Module) -> String        // per-tool make target

  // Progress (all renderers, manifest-driven)
  fn emit_progress_manifest(m: &ProgressManifest) -> String // static manifest for renderers
  fn emit_capture_buffer() -> String                        // per-node output capture
  fn emit_renderer(mode: RenderMode) -> String              // plain/inline/TUI/JSONL
}
```

**Note**: `emit_fn` emits a complete, compilable function (not a stub), because functor bodies are in the DSL. The design doc's compiler pipeline (§9) reflects this.

### ProgressManifest: the shared contract

All renderers read the same manifest. The manifest describes **what exists** (topology), not **how to display it** (rendering decisions). This is the key structural contract:

```
type ProgressManifest {
  // Topology
  total_nodes: Int
  topology: List<TopologyNode>

  // Labels (from DSL identifiers)
  labels: Map<NodeId, String>

  // Nested composition (the-gunbai's key capability)
  subdag_boundaries: List<SubDagBoundary>    // func calls → sections (patterns expand inline)
  parallel_groups: List<ParallelGroup>       // siblings at same depth → grouped counters
  scatter_points: List<NodeId>               // loop expansions → scatter groups [n/N]

  // Capture modes
  interactive_nodes: List<NodeId>            // @interactive → passthrough
  capture_modes: Map<NodeId, CaptureMode>    // Captured | Passthrough | Streamed

  // Stage groups (pipeline only)
  stage_groups: List<StageGroup>             // pipeline stages → collapsible sections

  // Resource context
  resources: Map<NodeId, List<ResourceUsage>>
}

type SubDagBoundary {
  node_id: NodeId
  label: String                              // "Authentication", "Fetching Secrets"
  inner_nodes: List<NodeId>                  // nodes inside — for expansion/collapse
  parent: NodeId?                            // for nesting: SubDag inside SubDag
}

type ParallelGroup {
  nodes: List<NodeId>
  depth: Int
  parent_subdag: NodeId?                     // which section this group belongs to
}
```

### Nested composition model (preserving the-gunbai)

This is the core rendering capability that gunbc lost and the DSL must restore. SubDags nest arbitrarily, and renderers must handle this:

```
func login() -> { ok: Bool } {
  auth = authenticate()          // SubDag → "› Authentication" section
  secrets = fetch_secrets(...)   // SubDag → "› Fetching Secrets" section
}

func authenticate() -> { token: Secret } {
  cache = clear_cache()          // node inside "Authentication"
  env = detect_env()
  tokens = check_tokens()
  cred = credential_chain(...)   // nested SubDag inside "Authentication"!
}
```

The manifest captures the nesting via `SubDagBoundary.parent`:

```
SubDagBoundary { label: "Authentication", parent: None }
SubDagBoundary { label: "credential_chain", parent: Some("Authentication") }
SubDagBoundary { label: "Fetching Secrets", parent: None }
```

Each renderer decides how to handle nesting:

| Renderer | How it handles nested SubDags |
|---|---|
| **plain** | Indented sections: `› Authentication` then `  › credential_chain` (deeper indent) |
| **inline** | Collapsed chips: `[✓ auth [✓ cred]] [◐ secrets]` — nested brackets |
| **TUI** | Expandable boxes: click to drill into a SubDag, see its inner nodes |
| **JSONL** | Flat events with `parent` field for consumers to reconstruct hierarchy |

### Four rendering backends

| Backend | Input | Output | When | Composition |
|---|---|---|---|---|
| **`plain`** | ProgressManifest + node state | Sections + status lines (gunb.ai style) | CI, non-TTY, piped | SubDags → `›` section headers; parallel → no special treatment; loops → `[n/N]` counter |
| **`inline`** | ProgressManifest + node state | Compact progress bar + chips (the-gunbai style) | Default TTY | SubDags → chips `[✓ auth]`; parallel → grouped in chip; loops → scatter count in chip |
| **`TUI`** | ProgressManifest + node state | Full DAG with boxes, edge pulses, wave layout (the-gunbai style) | Explicit opt-in | SubDags → expandable bordered boxes; parallel → same wave column; loops → expand/collapse |
| **`JSONL`** | ProgressManifest + node state | Structured event stream | Machine consumption | Flat events with `subdag_id` + `parent` fields; consumers reconstruct hierarchy |

### Rendering lifecycle (runtime protocol)

The engine drives renderers through a minimal event protocol:

```
type ProgressEvent
  = NodeStarted { id: NodeId, timestamp: Instant }
  | NodeSucceeded { id: NodeId, duration: Duration }
  | NodeFailed { id: NodeId, duration: Duration, captured_stderr: String }
  | NodeSkipped { id: NodeId, reason: SkipReason }
  | InteractivePause { id: NodeId }           // renderer clears, yields terminal
  | InteractiveResume { id: NodeId }          // renderer resumes display
  | ScatterProgress { id: NodeId, completed: Int, total: Int }  // loop iteration progress
```

Renderers implement:

```
trait ProgressRenderer {
  fn init(manifest: &ProgressManifest)        // receive static topology at start
  fn handle(event: ProgressEvent)             // update display per event
  fn finalize(summary: &RunSummary)           // show completion / error summary
}
```

The manifest is static (from compiler). Events are dynamic (from engine). Renderers combine both.

### Visual design specification (exact values, from gunb.ai → gunbc → DSL)

These are already proven and should be locked:

| Element | Value | Source |
|---|---|---|
| Section marker | `›` (U+203A) | gunb.ai |
| Spinner | Braille: `⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`, 80ms tick | gunb.ai |
| Colors | Success=`\033[38;5;34m`, Active=`\033[38;5;208m`, Error=`\033[38;5;196m`, Info=`\033[38;5;39m`, Dim=`\033[2m`, Calm=`\033[38;5;75m` | gunb.ai → gunbc |
| Status icons | `✓` success, `◐` running, `○` pending, `✖` failed, `◌` skipped | gunb.ai → gunbc |
| Error boxes | `╭─ Error: node-name ─╮` with captured stderr | gunb.ai |
| Preamble boxes | `╭─ tool-name ──╮` with description + args | gunb.ai |
| Completion animals | Random emoji on success | gunb.ai |
| Node line format | `   {icon} {name} ({duration})` — 3-space indent | gunb.ai |
| Duration format | `1ms`, `50ms`, `0.5s`, `3.4s`, `1m30s` | gunb.ai |
| Box border chars | `╭ ╮ ╰ ╯ │ ─`, width 60, min 40 | gunb.ai → gunbc |

### Terminal crate (harvestable from gunbc)

A single `terminal` crate containing ~2,271 lines harvested directly from gunbc:

| Component | Lines | Source | Status |
|---|---|---|---|
| `symbols.rs` (SemanticColor, SymbolId, tier resolution) | ~750 | gunbc | 95% standalone |
| `render_ir.rs` (frame rendering primitives) | ~580 | gunbc | Remove IR trait stubs |
| `box_draw.rs` (bordered boxes for errors/preamble) | ~427 | gunbc | Standalone |
| `frame_write.rs` (terminal write operations) | ~314 | gunbc | Standalone |
| `terminal.rs` (detection, viewport, capabilities) | ~200 | gunbc | Standalone |
| TUI module (ratatui + crossterm: edge pulses, DAG layout) | ~1,500 | the-gunbai | Optional `tui` feature flag |

### What the compiler emits per backend

For each `.dag` module, the `Emit` phase produces:

```
tools/makegen.dag
  │
  ├── types/          Type definitions (records, enums)
  ├── fn/             Pure functors (render_makefile, etc.)
  ├── transport/      Transport wiring (HTTP, shell, file)
  ├── func/           DAG orchestrator (topo-scheduled execution)
  ├── cli/            CLI entrypoint (arg parsing from func inputs)
  ├── test/           Test harness (4-bucket obligations)
  ├── mock/           MockSpec (from service declarations)
  ├── manifest/       ProgressManifest (static, from topology)
  └── makefile/       Makefile target (from module metadata)
```

The progress renderers are NOT per-module — they're framework code emitted once for the whole project, reading any module's manifest.

### Rendering phasing (when each capability lands)

| Phase | Rendering capability | What it proves |
|---|---|---|
| 0 | ProgressManifest type definition; terminal crate harvested | Structural foundation |
| 1 | `plain` renderer: sections from SubDag boundaries, status lines | gunb.ai-style output for `makegen` |
| 1 | `inline` renderer: compact bar + chips | the-gunbai-style default for TTY |
| 2 | `CaptureMode` on transport nodes; `@interactive` → passthrough | Auth flow terminal handling |
| 3 | Scatter groups from loops; nested SubDag rendering; `TUI` renderer | the-gunbai's nested composition restored for `gist` |
| 3 | Error boxes with captured stderr | gunb.ai's error display |
| 4 | Stage groups from pipelines; `JSONL` renderer | CI-scale progress with machine-consumable output |

---

## Key Decision: Typed Functor Protocol for Pure Logic

> The DSL includes a **constrained, typed functor protocol** for pure transforms — not a general-purpose language, not an "expression language + escape hatch," and not host-language stubs.

### The model: MapReduce, applied to DAGs

In MapReduce, you provide two functors (`map` and `reduce`) with specific typed signatures. The framework handles distribution, scheduling, fault tolerance. You write pure functions. The framework does everything else.

The DAG language follows the same model:

- The **shell** (`func`, `pipeline`, `service` calls) defines the DAG: ordering, I/O, resources, concurrency. This is the "framework."
- The **functors** (`fn`) are pure transforms with typed signatures. No I/O, no mutation, no side effects. This is "your code."
- The **compiler** handles everything else: transport wiring, test generation, progress manifests, multi-target emission, CLI entrypoints.

The constraint is the feature. If functors were arbitrary, the compiler couldn't reason about them — testgen breaks, multi-target emission breaks, refactoring becomes partial. By constraining functors to a small set of portable constructs, the compiler sees *everything*, which is what makes the "for free" features possible.

### Options considered

| Option | What the DSL covers | Verdict |
|---|---|---|
| **A: Structure-only** (stubs in host language) | DAG shape, types, services. Pure logic reimplemented per target language. | Rejected — "language-agnostic" promise is hollow; logic isn't portable. |
| **B: Expression language + escape hatch** | Structure + constrained expressions. `@custom` for complex cases. | Rejected — two half-languages, cliff problem, escape hatch becomes dominant. |
| **C: Unrestricted `fn` language** | Full computation language. Turing-complete. | Rejected — scope explosion; arbitrary functors undermine compiler guarantees. |
| **D: Typed functor protocol** | Constrained pure functions (~12 constructs). Intentionally limited to the intersection of mainstream languages. Compiler sees all code. | **Selected.** |

### Why constrained is better than arbitrary

| If functors are arbitrary... | If functors are constrained... |
|---|---|
| Compiler can't generate tests for function bodies | Compiler generates property-based tests from type signatures + function structure |
| Multi-target emission requires reimplementing complex logic per language | Every construct has a mechanical 1:1 translation to Rust/Go/Python/TS |
| Dead code detection is impossible (opaque bodies) | Compiler sees all paths, all references |
| Refactoring is partial (can't rename inside opaque bodies) | Refactoring is global across all backends |
| People write complex, untestable, unportable logic | The constraint forces simplicity — which is what makes "for free" work |

### The functor protocol: 12 constructs

The `fn` body language is the **semantic intersection** of Rust, Go, Python, and TypeScript at the pure-function level. Every construct has a direct, mechanical translation to all targets.

| # | Construct | Example |
|:---:|---|---|
| 1 | `let` bindings (immutable) | `let x = expr` |
| 2 | String interpolation | `"{branch}-snapshot.md"` |
| 3 | `match` (exhaustive) | `match scheme { Bearer => "Bearer {t}" }` |
| 4 | `if / else` | `if x > 0 { a } else { b }` |
| 5 | `for` (map sugar) | `for f in files { f.name }` |
| 6 | Pipe `\|>` | `list \|> filter(f => f.ok) \|> count` |
| 7 | Function calls | `join(targets, "\n")` |
| 8 | Record construction | `Report { passed, failed, total }` |
| 9 | Field access | `result.payload.name` |
| 10 | Arithmetic | `+ - * / %` |
| 11 | Comparison | `== != < > <= >=` |
| 12 | Boolean logic | `&& \|\| !` |

**Collection operations lower to IR nodes (data-parallel execution)**:

Collection operations (`map`, `filter`, `fold`, `flat_map`, `join`, etc.) inside `fn` bodies are **not** compiled as opaque function calls. The compiler lowers them to IR-level collection nodes (`MapNode`, `FilterNode`, `FoldNode`, `JoinNode`, etc.) whose inner transforms are scalar functions. This means:

- The executor can parallelize `MapNode` across workers
- `MapNode → FilterNode` can stream without materializing intermediates
- Adjacent trivial maps can be fused into single passes

Two kinds of parallelism are visible in the IR:
- **Task-parallel**: func-level `for` loops (each iteration has I/O)
- **Data-parallel**: `fn`-level `|> map/filter/fold` (each element is a pure transform)

This is the key design property that ensures "all programs can be efficiently parallelized for free."

**Not included** (and this is intentional):

| Excluded | Why |
|---|---|
| Mutation / `let mut` | Prevents portable compilation; forces value semantics |
| I/O of any kind | Purity enforced by grammar — there are no I/O primitives to call |
| General recursion | Totality: functors always terminate. Collection ops (`map`, `filter`, `fold`) cover the need. |
| Classes / traits / interfaces | Unnecessary complexity for pure transforms. Types are records + enums. |
| Generics beyond `List<T>`, `Map<K,V>`, `Option<T>` | Keep the type system simple. The std lib is generic; user functors typically aren't. |
| Closures as values | Lambdas appear only inline in `\|>` chains and `for`. Not assignable to variables. |
| Async / concurrency | The DAG shell handles this. Functors are synchronous and sequential. |
| Error handling syntax | The DAG propagates node failures. Functors return values, not `Result<T, E>`. |

### Concrete examples

**Render a Makefile** (makegen functor):

```
fn render_makefile(registry: ToolRegistry) -> String {
  let header = "# Generated by makegen"
  let targets = for tool in registry.tools {
    "{tool.name}:\n\t{tool.command}"
  }
  "{header}\n\n{targets |> join("\n\n")}\n"
}
```

**Format auth header** (credential chain functor):

```
fn format_auth_header(token: String, scheme: AuthScheme) -> String {
  match scheme {
    Bearer            => "Bearer {token}"
    Header { name }   => "{name}: {token}"
    Basic { username } => "Basic {base64("{username}:{token}")}"
  }
}
```

**Aggregate CI results** (CI functor):

```
fn aggregate_results(results: List<StepResult>) -> AggregateReport {
  let passed = results |> filter(r => r.success) |> count
  let failed = results |> filter(r => !r.success) |> count
  let failures = results
    |> filter(r => !r.success)
    |> map(r => FailureDetail { step: r.name, error: r.error })
  AggregateReport { passed, failed, total: results |> count, failures }
}
```

### Multi-target emission: mechanical translation

Every construct maps directly. No cleverness required.

| DSL | Rust | Go | Python |
|---|---|---|---|
| `let x = expr` | `let x = expr;` | `x := expr` | `x = expr` |
| `"{x}-{y}"` | `format!("{}-{}", x, y)` | `fmt.Sprintf("%s-%s", x, y)` | `f"{x}-{y}"` |
| `match e { A => ... }` | `match e { A => ... }` | `switch e { case A: ... }` | `match e: case A: ...` |
| `for x in list { body }` | `list.iter().map(\|x\| body).collect()` | `for _, x := range list { ... }` | `[body for x in list]` |
| `list \|> filter(f)` | `list.iter().filter(f).collect()` | `for range` + `if` | `[x for x in list if f(x)]` |
| `Record { a, b }` | `Record { a, b }` | `Record{A: a, B: b}` | `Record(a=a, b=b)` |

### Complete `.dag` file — shell + functors in one

```
module tools.gist

import services.git
import services.github.gist
import std.patterns { credential_chain }

// --- Functors: pure transforms (no I/O possible) ---

fn gist_filename(branch: String, base_ref: String?) -> String {
  match base_ref {
    Some(ref) => "{branch}-vs-{ref}.md"
    None      => "{branch}-snapshot.md"
  }
}

fn render_snapshot(files: List<FileContent>, branch: String) -> String {
  let header = "# Snapshot: {branch}"
  let sections = for f in files {
    "## {f.path}\n```\n{f.content}\n```"
  }
  "{header}\n\n{sections |> join("\n\n")}"
}

// --- Shell: DAG with I/O at boundaries ---

func gist_snapshot(base_ref: String?) -> { url: String }
  uses fs: Filesystem(mode: Read)
{

  branch = git.Core.CurrentBranch()
  files = git.Core.LsFiles()
  contents = for file in files.files { fs.read(path: file) }

  // Functor calls — compiler knows these are pure
  markdown = render_snapshot(files: contents, branch: branch.name)
  filename = gist_filename(branch: branch.name, base_ref: base_ref)

  // Service calls — compiler inserts transport triplets
  cred = credential_chain(runtime: detect_runtime())
  result = github.Gist.Create(
    description: "Snapshot from {branch.name}",
    files: { filename: markdown },
    credential: cred.credential
  )

  return { url: result.url }
}
```

### What the compiler gains from seeing functor bodies

Because functors are constrained and the compiler sees all code:

| Capability | How |
|---|---|
| **Property-based tests for pure functions** | Generate test inputs from type signatures; verify `render_snapshot` produces valid markdown for all `List<FileContent>` inputs |
| **Dead code detection** | `gist_filename` is never called → compiler warns |
| **Cross-backend equivalence tests** | Emit the same functor to Rust and Go; run both on same inputs; assert outputs match |
| **Inline optimization** | Trivial functors (single expression) inlined into DAG nodes |
| **Documentation generation** | Compiler generates docs showing what `render_snapshot` does, with example inputs/outputs from type-driven generation |
| **Mutation testing** | Systematically mutate functor bodies; verify tests catch the mutations |
| **Exhaustiveness checking** | `match` on `AuthScheme` missing a variant → compile error |

### Standard library (~30 functions, grows per phase)

| Category | Functions |
|---|---|
| **String** | `join`, `split`, `trim`, `contains`, `starts_with`, `ends_with`, `replace`, `to_upper`, `to_lower`, `regex_match` |
| **Collection** | `map`, `filter`, `fold`, `flat_map`, `count`, `sort_by`, `group_by`, `first`, `last`, `take`, `skip`, `any`, `all` |
| **Encoding** | `base64`, `url_encode`, `json_stringify`, `json_parse` |
| **Math** | `min`, `max`, `abs`, `round`, `floor`, `ceil` |
| **Formatting** | `pad_left`, `pad_right`, `truncate` |

### Impact on the design doc principles

**P9 revision**: "The language is total. `fn` functors are pure and operate on finite data (no general recursion, no I/O primitives). Funcs and pipelines are the imperative shell that sequences I/O through services and resources. Compilation always terminates."

**P10 stays**: "`.dag` files are the single source of truth for structure AND behavior. Codegen backends emit complete, runnable code in any target language."

**The 95/5 split becomes 100/0** for most workflows. `@custom` exists only for the rare case of host-language-specific SDKs with no REST/shell equivalent.

### Compiler cost (bounded)

| Component | Effort | Phase |
|---|---|---|
| Parser for `fn` bodies (12 constructs) | 2-3 weeks | Phase 0 |
| Type checker (records, enums, `List<T>`, `Option<T>`) | 4-6 weeks | Phase 0-1 |
| Rust codegen backend | 3-4 weeks | Phase 1 |
| `interface` / `implements` resolution | 2-3 weeks | Phase 2 |
| Collection ops → IR nodes (`MapNode`, `FilterNode`, `FoldNode`) | 2-3 weeks | Phase 2 |
| Go codegen backend | 3-4 weeks | Phase 4 |
| Standard library (~30 functions) | Ongoing | Incremental per phase |

### How it stages

| Phase | Functor features needed | Proving on |
|---|---|---|
| 0 | Parser, basic type checker, `let`, string interpolation | (scaffolding) |
| 1 | `match`, `if/else`, function calls, record construction | `render_makefile` (makegen) |
| 2 | Pipe `\|>`, `filter`, `map`, enum matching; **collection ops → IR nodes** (`MapNode`, `FilterNode`) | `format_auth_header` (credential), `storage_for_env` (infra) |
| 3 | `for`, `fold`, `flat_map`, richer std lib | `render_snapshot` (gist) |
| 4 | Go backend, `group_by`, `sort_by`, polish std lib | `aggregate_results` (CI) |

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
| 4 | **S8** Infra bootstrap (GCP) | Phase 2 | `interface`, `implements`, `resource` acquire, `CloudConfig` | N/A (new) | ~60 |
| 5 | **S4** Gist | Phase 3 | Composition, `for` loop, multi-service | ~1,449 | ~80 |
| 6 | **S6** Review | Phase 2-3 | External API service, credential reuse | ~1,376 | ~50 |
| 7 | **S7** Auth flow | Phase 3 | `@interactive` passthrough, browser/platform resource | ~400 | ~40 |
| 8 | **S9** Cross-cloud deploy | Phase 3 | Cross-provider `uses`, AWS/Azure resources, multi-credential | N/A (new) | ~40 |
| 9 | **S5** CI pipeline | Phase 4 | `pipeline`, `stage`, `parallel`, `aggregate` | ~920 | ~60 |

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
         Part 0           Phase 0        Phase 0.5        Phase 1         Phase 2         Phase 3         Phase 4
        ┌──────────┐     ┌──────┐       ┌──────────┐    ┌──────┐        ┌──────┐        ┌──────┐        ┌──────┐
        │Contracts │────▶│Scaff.│──────▶│Model     │───▶│Core  │───────▶│Svc + │───────▶│Comp +│───────▶│Pipe +│
        │+ Fixtures│     │+ Viz │       │Preview   │    │      │        │Res   │        │Loop  │        │Stage │
        └──────────┘     └──────┘       └──────────┘    └──────┘        └──────┘        └──────┘        └──────┘
                                                │               │               │               │
                                                ▼               ▼               ▼               ▼
Migration:                           ┌──────────────────────────────────────────────────────────────┐
 Workstream A                        │  Parity Harness (runs continuously — build once, use always) │
 (backbone)                          └──────────────────────────────────────────────────────────────┘
                                                │               │               │               │
                                                ▼               ▼               ▼               ▼
 Workstream B                        ┌─────────┐┌──────────────┐┌─────────────┐┌──────────────┐
 (port)                              │S1,S3    ││S2,S6,S8      ││S4,S7,S9    ││S5            │
                                     │makegen  ││credential    ││gist, auth   ││CI pipeline   │
                                     │clippy   ││review, infra ││cross-cloud  ││              │
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

> **Updated February 2026.** The compiler pipeline spine, CLI scaffolding, and DSL corpus are in place. The immediate priority is the **Bridge Milestone** (closing Phase 0 gaps and establishing shared contracts for Phase 1). See the Bridge Milestone section above for the full workstream breakdown.

### What's been accomplished

- [x] `core/daglang/` workspace area with crate split: `daglang-syntax`, `daglang-resolve`, `daglang-typecheck`, `daglang-lower`, `daglang-derive`, `daglang-emit`, `daglang-cli`
- [x] Compiler entrypoint: `compile_from_context()` does discover → typecheck → lower → derive → emit
- [x] CLI commands: `viz`, `expand`, `manifest`, `obligations`, `show-triplets`, `modules`, `check`, `compile`
- [x] Module graph: filesystem discovery, import resolution, dependency-ordered module listing
- [x] Parity harness (partial): `compare_topology()` + `compare_makegen_topology()` returning `ParityReport`
- [x] DSL corpus: `.dag` files for tools, services, pipelines, infra, cloud, examples
- [x] `TestObligations` derivation with 4-bucket obligation counting
- [x] `ToolMetadata` derivation (module-level callable/pipeline counts)

### Priority 1: Bridge Milestone (parallel workstreams)

> The #1 structural blocker is the ProgressManifest contract mismatch. Fix this first.

- [ ] **Workstream A (Manifest contract):** Expand `ProgressManifest` to match the roadmap contract; add `dag manifest --format json`
- [ ] **Workstream C (Parity + snapshots):** Canonical JSON serialization + IR snapshot tests for at least `tools/makegen.dag`
- [ ] **Workstream B (Viz + expand):** ASCII default for `dag viz`; golden tests for `dag expand`
- [x] **Workstream E (Model preview commands):** `dag show-triplets` and `dag obligations` as standalone commands
- [ ] **Workstream F (Makegen parity):** Compile `tools/makegen.dag` → compare against `build_makegen_graph()` using parity harness
- [ ] **Workstream D (Discovery):** Config-driven roots; enriched `dag modules` output

### Priority 2: Phase 1 makegen (S1)

> Once the bridge is crossed, makegen becomes the proving workflow.

- [x] Write `tools/makegen.dag` — *exists in DSL corpus*
- [x] Implement parser for `func` + `pattern` + `uses` syntax — *parser handles full language surface*
- [x] Implement `Lower` phase producing gunbc IR — *`daglang-lower` produces `Dag<LoweredOp>`*
- [ ] Run parity harness against existing `build_makegen_graph()` — *Workstream F*
- [ ] Verify golden fixture passes for compiled `.dag` output — *Workstream C*

### Priority 3: Workflow contracts (Part 0, ongoing)

> The foundation everything else builds on. Can proceed in parallel with bridge work.

- [ ] Fill out the workflow matrix for all 7-8 discrete workflows
- [ ] Create at least one golden fixture per workflow
- [ ] Wire golden fixture checks into CI

### Standing gates (must not regress)

- [x] I/O only at boundaries (clippy guardrails enforce transport-only I/O)
- [ ] DryRun interception works for compiled `.dag` output
- [ ] Generated tests derived from DAG structure (4-bucket model)
- [x] No hidden env access (resource declarations required; CI detection hardened against spurious env vars)

---

## Appendix: Scenario → DSL Construct Matrix

For each workflow: what DSL constructs are required, what modeling gaps are likely to surface, what parity gates to run, and what gets deleted in the final unify.

### S1: Makegen (Content Upsert)

| Dimension | Details |
|---|---|
| **DSL constructs** | `type`, `pattern content_upsert`, `uses fs: Filesystem(mode: Write)`, `func` |
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
| **DSL constructs** | `for file in files`, func composition (SubDag), `service git.Core`, `service github.Gist` |
| **Modeling gaps** | Loop scatter points in ProgressManifest, multi-mode func variants |
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

### S8: Infra Bootstrap (Multi-Cloud Interfaces)

| Dimension | Details |
|---|---|
| **DSL constructs** | `interface ObjectStorage { ... @contract }`, `resource GcsBucket implements ObjectStorage`, `CloudConfig` sum type, `resource` acquire blocks, `@mock_response` |
| **Modeling gaps** | Interface-to-implementation resolution at compile time, `@contract` → behavioral test generation, `ResourceFingerprint` for drift detection |
| **Parity gates** | Abstract interface resolves to concrete resource based on `CloudConfig`; `acquire` DAG produces correct check→create→resolve pattern; `@contract` tests generated and pass for GCP impl |
| **Deletes in unify** | N/A (new capability — no existing Rust equivalent) |

### S9: Cross-Cloud Deployment (Multi-Provider Composition)

| Dimension | Details |
|---|---|
| **DSL constructs** | Cross-provider `uses` (GCS + S3 + SQS in one func), AWS credential chain (OIDC → STS), Azure credential chain (federated identity → AD), provider-specific `resource` acquire blocks |
| **Modeling gaps** | Independent auth chain resolution per provider, cross-provider resource conflict detection, multi-provider test generation |
| **Parity gates** | Cross-provider func compiles; each provider's auth chain resolves independently; `@contract` tests pass for all 3 providers' `ObjectStorage` implementations; mock-based hermetic tests run for cross-provider DAGs |
| **Deletes in unify** | N/A (new capability) |

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
- [Unified Registration](../unified-registration.md) — discovery unification design
- [Unified Emission](../unified-emission.md) — emission system consolidation design
- [Handbook](../handbook.md) — pipeline invariants and pattern catalog
