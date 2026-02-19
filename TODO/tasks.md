# Task Sheet — Dependency-Ordered, Parallelizable

**Last updated**: 2026-02-19
**Verification**: `cargo test --workspace` + `cargo clippy --all-targets -- -D warnings`
**Archive**: Completed items in `TODONE/`. Original TODO details preserved in git history.

**Sizing**: S (<1 day), M (1-3 days), L (3-5 days), XL (5+ days)

### Conventions

- **Definition of Done**: each task is done when code compiles, tests pass, and clippy is clean.
- **Code TODO/HACK comments** must reference a task ID (e.g., `TODO(F1): ...`) so orphans
  are discoverable via grep.
- **Active Docs invariant**: every path in the task sheet must exist; no doc under
  `TODO/TODONE/` may appear in active sections.

---

## Sprint 1: Get to Green

**Goal**: CI-green workspace. All test failures are generated-test structural drift
from the DSL roadmap sprint — the underlying code changes (resolver, lowering, CI binary)
are already committed and working.

### Phase 1: Regenerate (single root cause, ~30 failures)

All remaining failures are `ci::generated_tests::test_coercion_*` in gunbc-dag.
The CI DAG shape changed (literal_source nodes, GCP transport ops, CI binary simplification)
but the generated test harnesses still reference the old topology.

| ID | Task | Deps | Size |
|----|------|------|------|
| **F1** | Regenerate CI testgen: `make testgen` to rebuild generated coercion + dryrun tests for current CI DAG shape | — | S |
| **F2** | Verify `cargo test -p gunbc-dag ci::generated_tests` passes after regeneration | F1 | S |

### Phase 2: Acceptance & Guardrails

| ID | Task | Deps | Size |
|----|------|------|------|
| **F3** | Update `workflow_acceptance.rs` tests to match restructured `ci.rs` (auto-mocked spec, simplified path dispatch) | — | M |
| **F4** | Update `engine_execution_guardrails` allowlist for new execution helper surfaces | — | S |
| **F5** | Register new public graph builders in `resource_registry_coverage` test | — | S |

### Phase 3: Snapshot Refresh

| ID | Task | Deps | Size |
|----|------|------|------|
| **F6** | Regenerate `daglang-cli` snapshots if any compile/manifest/run golden files are stale after lowering changes | F1 | S |

---

## Near-Term: Code TODOs & DSL Compiler Polish

5 actionable TODO comments + 5 PR-review items. Zero HACK/FIXME.

### Design Decision: Node Metadata Classification (blocks P1-P3)

P1-P3 all replace string heuristics in `daglang-derive/src/lib.rs` with structural
classification on `LoweredOp`. The shared design question is: **what fields on
`LoweredOp::Callable` carry the metadata that `daglang-derive` currently extracts
from name strings?**

Current heuristics and their structural replacements:

| Derive function | Current heuristic | Available structural data | Missing |
|----------------|-------------------|--------------------------|---------|
| `derive_capture_modes()` (line 485) | Hardcoded `CaptureMode::Captured` for all nodes | `obligation: ObligationCategory` distinguishes transport (`ServiceTransport*`) from pure | `is_interactive` flag (for `Passthrough` mode); streaming marker (for `Streamed` mode) |
| `derive_interactive_nodes()` (line 495) | `name.contains("@interactive")` | Nothing — interactivity only exists as a name substring | `is_interactive: bool` on `LoweredOp::Callable` |
| `derive_resources()` (line 512) | Three `strip_prefix()` calls: `resource_lifecycle::acquire::`, `resource_lifecycle::release::`, `resource_provide::` | `obligation` already has `ResourceAcquire`, `ResourceRelease`, `ResourceProvide` variants | Resource name / binding name as a dedicated field (currently encoded in `name` string suffix) |

**Approach**: Extend `LoweredOp::Callable` with two fields during lowering:

```rust
Callable {
    module: String,
    kind: String,
    name: String,
    obligation: ObligationCategory,
    service_metadata: Option<ServiceMetadata>,
    is_interactive: bool,          // NEW — parsed from DSL `@interactive` attr
    resource_target: Option<String>, // NEW — resource name for lifecycle/provide nodes
}
```

Then all three derive functions become enum matches on `obligation` + field reads,
following the established `classify_obligation()` pattern in `daglang-lower:179`.
No string parsing in the derive phase.

### Design Decision: DeferredCallableOp Elimination Strategy (blocks P6)

P6 replaces `DeferredCallableOp` (identity passthrough) with per-tool domain ops.
The design question is: **what concrete `Executable` impl replaces each deferred
callable, and how should dry-run mode work without a passthrough fallback?**

Current deferred callables (from `resolve.rs` + `rust_exec_runtime.rs:306`):

| Module | Callables | What they actually do |
|--------|-----------|----------------------|
| `tools.build` | `build_all` | Orchestrates `cargo build` — prepare shell request, parse result |
| `tools.docgen` | `docgen`, `render_ab_workflows_doc` | Generate markdown docs from registry data |
| `tools.testgen` | `generate_tests`, `testgen` | Generate test harnesses from DagSpecDef |
| `tools.clippy` | `clippy_lint` | Prepare `cargo clippy` invocation, parse diagnostics |
| `tools.deps` | `render_deps_toml`, `select_platform_deps`, `deps_install`, `deps_generate` | Dependency resolution and rendering |
| `pipelines.ci` | all | CI stage orchestration (entrypoint + stage dispatch) |
| `shared.dag_util` | all | DAG construction helpers (pure combinators) |
| `shared.gist_modes` | all | Gist mode selection logic |
| `std.patterns` | all | Standard patterns (content_upsert, while, etc.) |

**Contrast with properly resolved examples** (PragmaOp, MakegenOp, BootstrapOp):
each has a domain enum with per-variant `execute()` containing real business logic,
and unknown callables return `Err(unknown_callable(...))`.

**Approach**: implement per-module `*Op` enums following the PragmaOp pattern.
For dry-run: resolved ops should check `ExecutionMode::DryRun` internally and
short-circuit with typed empty outputs (not generic identity passthrough). The
catch-all `_ => Ok(deferred_callable(...))` on line 872 becomes
`_ => Err(unknown_callable(...))` once all modules have resolution arms.

| ID | Task | Deps | Size | Source |
|----|------|------|------|--------|
| **P1** | `daglang-derive:485` — Derive capture mode from `obligation` + `is_interactive` field on `LoweredOp::Callable`, not hardcoded. Three modes: `ServiceTransport*` → `Captured`, `is_interactive` → `Passthrough`, streaming TBD. | — | M | `TODO(Phase 3)` |
| **P2** | `daglang-derive:495` — Replace `name.contains("@interactive")` with `is_interactive: bool` field on `LoweredOp::Callable`, parsed from DSL `@interactive` attribute during lowering in `daglang-lower`. | P1 | S | `TODO(Phase 3)` |
| **P3** | `daglang-derive:512` — Replace three `strip_prefix()` calls (`resource_lifecycle::acquire/release::`, `resource_provide::`) with `obligation` enum match + `resource_target: Option<String>` field. The `ObligationCategory::Resource*` variants already exist. | P1 | S | `TODO(Phase 3)` |
| **P4** | `daglang-cli/commands.rs:147` — Deduplicate `check_from_context` re-discovery/re-parse/re-typecheck with cached pipeline state | — | M | `TODO` |
| **P5** | `lib/gcp-ops/src/ops.rs:568` — Wire token expiry into output if callers need it | — | S | `TODO` |
| **P6** | `DeferredCallableOp` → per-module domain ops: implement `*Op` enums for each deferred module (see table above), replace catch-all passthrough with `Err(unknown_callable(...))`. Dry-run via `ExecutionMode` check inside each op, not via identity passthrough. ~15 modules, ~25 callables total. (`resolve.rs`, `rust_exec_runtime.rs:306`) | F1 | L | PR review |
| **P7** | Remove `dedupe_release_resource_edges` (resolve.rs:1281): duplicate edges arise when a callable both `uses` and `provides` the same resource — `add_used_resource_edges()` (lower:3550) and `add_provided_resource_nodes()` (lower:3622) both independently wire to the same `release_resource_*` node. Fix: track already-wired release targets in the lowerer's `ResourceLifecycleRegistry` and skip if `(release_node, "resource_handle")` pair already has an inbound edge. | — | S | PR review |
| **P8** | Consolidate repeated GCP service client constructors (`new`/`unauthenticated`) into a shared helper/macro across `lib/gcp-ops/src/services/*`. | — | S | PR review |
| **P9** | Deduplicate `content_upsert` source wiring in `core/daglang/daglang-lower/src/lib.rs` (content/path branches share nearly identical param/source edge logic). | — | M | PR review |
| **P10** | Consolidate makegen compile test setup/cleanup in `core/daglang/daglang-cli/src/compile/tests.rs` (temp output creation + teardown helpers) to reduce repetition and cleanup leaks. | — | S | PR review |

---

## Medium-Term: Horizon Features (Ready for Implementation)

Design docs in `docs/design/horizon/`. These are unblocked or have soft prerequisites.

### Typing & Resource Hardening

| ID | Task | Deps | Size | Design |
|----|------|------|------|--------|
| **HR1** | Typed node I/O wrappers (`TypedInput<T>`, `TypedOutput<T>`, `TypedPort<T>`) + fail-closed semantic carriers | — | L | `h11-dag-typing-hardening.md` |
| **HR2** | `Resource` trait: capability-oriented contract for acquisition, probing, release across ops | — | L | `h7-resource-abstraction-trait.md` |

### DSL Language Extensions

| ID | Task | Deps | Size | Design |
|----|------|------|------|--------|
| **HL1** | Loop extra inputs passthrough: body receives non-element context (repo, branch, policy) | — | M | `h4-loop-extra-inputs-passthrough.md` |
| **HL2** | Testgen dynamic targets: N upsert chains per DagSpecDef via inventory at codegen time | — | L | `h2-testgen-dynamic-targets.md` |
| **HL3** | Makegen tool registry: procedural target gen from `#[tool_target]` attributes | — | L | `h3-makegen-tool-registry.md` |

### Workflow Rendering

| ID | Task | Deps | Size | Design |
|----|------|------|------|--------|
| **HW1** | Justfile renderer: second format consumer for `WorkflowSpec`, validate portability | — | M | `h8-workflow-rendering-justfile.md` |
| **HW2** | GitHub Actions YAML renderer: jobs from targets, `needs` from deps, permissions from resources | HW1 | L | `h9-workflow-rendering-github-actions.md` |

---

## Long-Term: Design-Gated / Large Scope

These require significant new DSL primitives or infrastructure that doesn't exist yet.

| ID | Task | Deps | Size | Design |
|----|------|------|------|--------|
| **H1** | Display orchestration DSL migration: channel-driven event loop, timer ticks, reactive `on`/`tick` triggers | DSL reactive primitives (parser + IR + runtime scheduler) | XL | `h1-display-reactive-dsl.md` |
| **H10** | Compute stack service interfaces: Cloud Run, GCS, LB (MVP), GCE (phase 2) | Provider-neutral trait defs + GCP adapters | XL | `h10-compute-stack-services.md` |

---

## Parallelization Guide

```
         ┌─ F1→F2,F6              (testgen regen → verify + snapshots)
SPRINT 1 ├─ F3                     (CI acceptance, independent)
         ├─ F4, F5                 (guardrail fixes, independent)
         │
    ─────┤ (Sprint 2: polish, after green CI)
         │
         ├─ P7, P8, P10           (mechanical: edge dedup, GCP macro, test helpers)
         ├─ P9                     (lowerer source wiring dedup)
         ├─ P1→P2,P3              (LoweredOp metadata fields → structural classify)
         ├─ P4                     (CLI pipeline caching)
         ├─ P5                     (GCP token expiry)
         │
    ─────┤ (Sprint 3: DeferredCallableOp elimination)
         │
         ├─ P6                     (per-module domain ops, L — blocks on design)
         │
    ─────┤ (Horizon features, pick based on need)
         │
         ├─ HR1                    (typing hardening)
         ├─ HR2                    (resource trait)
         ├─ HL1, HL2, HL3         (DSL extensions, all independent)
         └─ HW1→HW2               (workflow rendering)
```

**Sprint 1 priority**: F1 (testgen regen) unblocks ~30 test failures in one step.
**Sprint 2 highest ROI**: P7, P8, P10 (mechanical, zero design risk).
**Sprint 3 design dependency**: P6 requires deciding dry-run strategy per-module.
**Horizon pick order**: HR1 or HL1 have highest leverage (typing safety / loop ergonomics).

---

## Completed Work Summary (2026-02-18)

All completed items are archived in `TODONE/` with dates. The following waves were
fully completed on 2026-02-18 as part of the DSL roadmap sprint:

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
