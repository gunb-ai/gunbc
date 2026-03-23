# gunbc Roadmap

## Architectural Thesis

**Node and DAG are the only compiler primitives.**

The compiler is a generic graph processor. It reads `.dag` source, builds
a graph of `Node`s, applies structural rules, and emits target code. All
domain knowledge, including types, cardinality, containers, optionality,
and target-language facts, should live in `.dag` definitions, not in the
compiler implementation.

This thesis dissolves compiler knowledge in three layers:

| Layer | What dissolves | Compiler stops knowing | Status |
|-------|----------------|------------------------|--------|
| **L1: Types** | `BuiltinTypeKind`, `Conj`/`Disj`, `node_is_*`, type constructors | What `Optional`, `List`, `Map`, `Int`, etc. mean | **Active — 489 violations** |
| **L2: Expressions** | `ExprData` semantic knowledge | What `if`, `for`, `match`, `let`, etc. mean | Future — after bootstrap and shared emit |
| **L3: Syntax** | Hardcoded parser branches | How to parse surface syntax like `if cond { body }` | Future — data-driven parser |

L1 is the urgent layer. Every new feature that touches types currently
adds more compiler-side knowledge and more string checks. L2 and L3 are
real targets, but they are not blocking bootstrap or the current
migration.

---

## How To Read This Roadmap

This file now has one canonical schedule and three supporting
decompositions.

- **Phases are the canonical execution order.** If another section seems
  to imply a different ordering, the phase plan wins.
- **`M*` tracks** describe cross-cutting architecture migrations that span
  more than one phase.
- **`R*` targets** describe the desired end state of specific compiler
  modules once the naming cleanup lands.
- **`S*` passes** describe technical refactors that cut across phases and
  tracks.

The repo is still in the middle of a rename/relocation cleanup. Some
sections refer to current filenames, and some refer to target filenames.
Use this map:

| Old file | Current file (M1 complete) | Meaning |
|----------|---------------------------|---------|
| `04_reconcile.dag` | `04_infer.dag` | Stage 4 is infer/typecheck, not "reconcile" |
| `06_pipeline.dag` | `compile.dag` | Compiler driver/orchestrator, not a sixth stage |
| `07_complexity.dag` | `complexity.dag` | Proof/report layer, not a numbered stage |
| `07_ownership.dag` | `ownership.dag` | Proof/obligation layer, not a numbered stage |
| `08_artifact.dag` | `artifact.dag` | Artifact planning layer, not a numbered stage |
| `09_trace.dag` | `trace.dag` | Runtime/debug contract, not a numbered stage |

M1 naming cleanup is complete. All files now use their target names.

---

## End Goal

The compiler is a generic graph processor. It reads `.dag` source, builds
a graph of `Node`s, applies structural rules defined in `.dag`, and emits
target artifacts. Adding a type, expression, language, transport, or
runtime contract should mean editing `.dag` files, not compiler code.

Concrete acceptance:

- Zero type-world knowledge in the compiler (L1 complete)
- One shared emit walker drives all target languages through a common
  compiler-owned spine
- Language-specific facts live in `dsl/extdeps/languages/*`; program-
  dependent lowering lives in compiler-owned adapters
- Ownership and complexity proofs are wired into the compile pipeline
- At least one real program (`gist`) compiles and runs end to end
- v1 is archived
- Compiler-internal structure converges onto `Node` compositions

---

## Completed Milestones

| Milestone | Gate | Date |
|-----------|------|------|
| Self-compile pipeline | v2 processes its own `.dag` through all 5 core stages | 2026-03 |
| Bootstrap A5 | v1 -> stage0 -> stage1 (`cargo check`) | 2026-03 |
| Fixed point A6 | stage1 output == stage2 output (byte-identical) | 2026-03 |
| A7 Phase 1 | Self-compile reached 0 `cargo check` errors | 2026-03 |
| TypeExpr -> Node | 8 `TypeExpr` variants deleted | 2026-03 |
| Expr -> Node | 21 `Expr` variants deleted, `ExprData` now lives on `Node` | 2026-03 |
| Transport dissolution | `TransportBinding` deleted | 2026-03 |
| Node/TypedNode unified IR | W1-W13 complete, 129 tests passing | 2026-03 |
| Performance audit | tokenize+parse down to ~24ms | 2026-03 |
| OOM fix | `node_type_deps` cycle detection stabilized | 2026-03 |
| M1 naming cleanup | All non-stage files renamed to target names | 2026-03 |
| Stage0 build | 18 build errors fixed, stage0 compiles cleanly | 2026-03 |
| Stage0 parse | 5 parser ambiguities fixed in v2 source | 2026-03 |
| Gist pipeline | 11-file gist closure compiles with 0 diagnostics | 2026-03 |
| V1 feature-gate | v1 crates gated behind `v1-bootstrap` cargo feature | 2026-03 |
| Diagnostic reduction | 395 → 197 via tuple naming, error cascade, branch compatibility | 2026-03 |
| Diagnostic ratchet 0 | 197 → 0 via 4 root-cause fixes (map types, data scope, lookup returns, cascade suppression) | 2026-03 |
| RenderTarget extraction | Moved from `00_core.dag` to `artifact.dag` (orchestration, not kernel) | 2026-03 |
| Emit metadata extraction | `emit_info` removed from ResolvedGraph; emit builds EmitGraphInfo locally | 2026-03 |

---

## Current State (2026-03-22 Audit, updated 2026-03-22)

### Compositional Audit

| Area | Current state | Meaning for the next passes |
|------|---------------|-----------------------------|
| `00_core.dag` | Strong foundation, mostly target-agnostic | `Node`/`ExprData`/transport modeling is the right base. Remaining ownership leakage is downstream, not on core types. |
| `01_tokenize.dag` | Mostly clean syntax leaf | Good example of a narrow stage boundary. |
| `02_parse.dag` | Strong compositional lowering | Service/resource syntax already dissolves into uniform `Node` structure. |
| `03_resolve.dag` | Cleanest authority boundary | Good reference stage for future stage boundaries. |
| `04_infer.dag` | Main structural hotspot (4871 LOC) | Mixed concerns: inference, type resolution, method classification, emit metadata prep, type env management. This is the Phase 1 hotspot. Renamed from `04_reconcile.dag`. |
| `05_emit*.dag` | Partial shared composition | `05_emit.dag` owns helpers/context but not tree traversal. Rust (3634 LOC), Python (1202 LOC), and Go (1226 LOC) still own full 22-arm `ExprData` dispatchers. TCO is duplicated 3x. `classify_typed_item` is already shared and called by all three emitters. Go main expression emission still ends in `_ => /* unhandled expr */`. |
| `complexity.dag` / `ownership.dag` | Good proof layers | complexity and ownership are both real and now pipeline-wired through `compile_sources`. Both still duplicate expression walking logic. Renamed from numbered files. |
| `compile.dag` / `artifact.dag` / `trace.dag` | Honest boundary shape, incomplete integration | `compile.dag` (formerly `06_pipeline.dag`) now returns complexity, ownership, and a default artifact plan, and emit dispatch now follows that plan. Artifact planning is still single-artifact compatibility mode, but `Artifact.target` is now typed as `RenderTarget`. `trace.dag` is correctly an external contract, not an interpreter stage. |

### Active Ratchets

#### Phase-Blocking Ratchet: Diagnostics

`src/v2/tests/src/lib.rs` enforces `DIAG_RATCHET = 0`. **PHASE 2 GATE MET.**

Journey: 2797 → 395 → 197 → 0. Root causes eliminated:
- RC-A: `method_receiver_element_node` for Map<K,V>, map/flat_map/fold return type propagation
- RC-B: Suppress variant/field lookup cascade diagnostics on error/leaf types
- RC-C: Imported data declarations added to scope via `merge_scope_from_imports`
- RC-D: `lookup` return type fixed from receiver to `Optional<element>`

#### Architectural Ratchet: L1 Type Knowledge Dissolution

Scripted audit via `scripts/l1-ratchet.sh`: **454 matched references
across 14 .dag files** as of 2026-03-22. Ratchet enforced at 454.
(Up from 440 due to diagnostic fixes using existing type vocabulary.)

| Category | Count | What the compiler still "knows" |
|----------|-------|----------------------------------|
| Connective field + `Conj` / `Disj` | 201 | Product vs coproduct semantics |
| Type constructors | 125 | `leaf_node`, `optional_node`, `container_node`, `tuple_node`, etc. |
| Type-name comparisons | 62 | `.name == "Optional"`, `"Map"`, `"Dynamic"`, etc. |
| `node_is_*` predicate calls | 43 | Type-specific dispatch helpers |
| `builtin_type_kind()` calls | 23 | Hardcoded builtin classification |

L1 acceptance:

- `BuiltinTypeKind` deleted
- `builtin_type_kind()` deleted
- `node_is_*` predicates deleted or replaced with property reads
- `optional_node()`, `container_node()`, `pair_node()` deleted
- `connective` field removed from `Node`
- Zero type-name string matching in the compiler
- Fixed point still holds

---

## Canonical Execution Order

Use this as the source of truth for sequencing.

| Order | Phase | What it does | Blocking gate |
|-------|-------|--------------|---------------|
| 1 | Phase 1 | Naming cleanup, bootstrap-critical inference cleanup, and the start of L1 dissolution | Diagnostics ratchet reaches 0; M1 naming map lands |
| 2 | Phase 2 | `gist` end-to-end through emitted Rust | `gist` builds and runs correctly |
| 3 | Phase 3 | Compile bundle, ownership/artifact wiring, and v1 retirement | v2 can compile everything v1 still matters for |
| 4 | Phase 4 | Shared emit spine, generated tests as projections, DAG backend boundary | New backend = language facts + compiler-owned adapter, with no shared-core changes |
| 5 | Phase 5 | Remaining convergence work after bootstrap shape is stable | One `Node`-centric internal model across compiler structure |

Important clarifications:

- Phase 1 is the only intentionally overlapping phase. **Diagnostics are
  blocking. L1 is not.** Once diagnostics hit 0, Phase 2 can start even
  if L1 is still being reduced.
- M1 belongs at the front of the roadmap, not at the end. The rest of the
  document uses the target names on purpose.
- `M*`, `R*`, and `S*` are support structures for this phase order, not
  competing schedules.

---

## Phase 1: Naming, Soundness, and L1 Type Dissolution

This phase combines the work from the rename/relocation effort and the
type-dissolution effort because both touch the same files and the same
stage boundaries.

Only diagnostics block Phase 2. L1 continues in parallel after that.

### Why Phase 1 Goes First

- The roadmap and `src/v2/DESIGN.md` already assume the target module
  names. Delaying M1 makes every later section harder to read.
- `04_reconcile.dag` is the bootstrap-critical hotspot. The remaining
  diagnostics and the highest-value L1 work both live there.
- Property-first fixes let one change improve correctness and reduce L1
  debt at the same time.

### Phase 1 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P1.1 | Naming cleanup (M1) | **Done** | All non-stage files renamed; RenderTarget moved to artifact.dag |
| P1.2 | Infer cleanup via data tables (R5, S3.5, S7) | **Partial** | Emit metadata extracted (emit_info removed from ResolvedGraph). Method handling deferred to P4. |
| P1.3 | Diagnostics ratchet -> 0 | **Done** | DIAG_RATCHET = 0. Four root-cause fixes: map types, data scope, lookup returns, cascade suppression. |
| P1.4 | L1 Optional/cardinality | Planned | Property-first optionality and cardinality |
| P1.5 | L1 Containers | Planned | `List` / `Map` / `Set` properties and structural traversal |
| P1.6 | L1 Primitives | Planned | `Int` / `String` / `Bool` / `Float` / `Unit` / `Bytes` / `Json` / `Secret` |
| P1.7 | L1 Connective dissolution | Planned | Last large L1 step; remove `connective` from `Node` |
| P1.8 | Delete residual type primitives | Planned | Delete remaining constructors, predicates, and builtin classifiers |

### P1.1 Naming Cleanup Scope

This is one mechanical batch. Do it once and let the rest of the roadmap
read naturally afterward.

- `04_reconcile.dag` -> `04_infer.dag`
- `06_pipeline.dag` -> `compile.dag`
- `07_complexity.dag` -> `complexity.dag`
- `07_ownership.dag` -> `ownership.dag`
- `08_artifact.dag` -> `artifact.dag`
- `09_trace.dag` -> `trace.dag`
- Move `RenderTarget` out of `00_core.dag` into `artifact.dag` (done;
  `artifact.dag` avoids circular import with `compile.dag`)
- Update imports, bootstrap references, tests, docs, and roadmap wording

Acceptance:

- No numbered file remains that is not a core lowering stage
- Docs/tests/imports use `04_infer` and `compile`
- The design language no longer refers to the driver as "stage 6"

### P1.2-P1.3 Diagnostic Fix Detail

These are the concrete remaining correctness items behind the current
diagnostic ratchet.

| Fix | Status | Notes |
|-----|--------|-------|
| Enumerate return type | Done | `List<Tuple<Int, T>>` now flows through inference |
| Fold accumulator threading | Done | `fold_accumulator_type` follows init-arg type |
| Callable/function-value type | Done | Callable type representation exists |
| Structured `ErrorCategory` | Done | Error classification moved off ad hoc strings |
| `map_insert` / `map_merge` result typing | Remaining | Still returns a bare `Map` leaf in the wrong places |
| Chained field access | Remaining | Depends on the map fixes to stop collapsing structure |
| Tighten `node_type_equals` | Remaining, last | Remove permissive `Dynamic` and structural fallbacks after inference stops fabricating them |

### P1.4-P1.8 L1 Family Order

#### Optional / Cardinality

Move optionality to `.dag`-defined properties. The compiler should read
properties, not `n.name == "Optional"`, and emitters should render from
those same properties.

#### Containers

Move `List`, `Map`, and `Set` behavior to structural properties. Fix the
current "bare leaf vs parameterized map" inconsistency as part of this
step.

#### Primitives

Move copy semantics, literal forms, method availability, and similar facts
for `Int`, `String`, `Bool`, `Float`, `Unit`, `Bytes`, `Json`, and
`Secret` into `.dag`.

#### Connective Dissolution

This is the largest L1 step and must go last. Remove `connective` from
`Node` only after optionality, containers, and primitives already read
properties instead of shape shortcuts.

#### Residual Primitive Deletion

After consumers have switched, delete the old constructors, builtin
classifiers, and predicate helpers.

### Phase 1 Exit Criteria

- `cargo test -p v2-compiler-tests v2_strict_compile_diagnostic_count -- --ignored` passes
- M1 naming cleanup is complete
- Fixed point still holds after every structural change
- Phase 2 may start once diagnostics hit 0, even if the L1 ratchet is not
  yet at 0

---

## Phase 2: Gist End to End

**Gate:** `gist.dag` plus its transitive dependencies compile to Rust,
`cargo build` succeeds, and the emitted program runs correctly in dry-run
mode.

### Current Status

- Service operation bodies are already real in `05_emit_rust.dag`
- `main.rs` workflow dispatch is already emitted
- The remaining blocker is verification through a built stage0 binary

### Why This Is Still Blocked

The v1 interpreter path cannot handle the full multi-module compile
through `compile_sources` because of lowered lambda scoping issues. That
means the real verification path is the stage0 binary, not the v1
interpreter.

The current acceptable path is:

1. Build stage0 via `v2_bootstrap_fixed_point`
2. Use the resulting binary to compile `gist`
3. Build and run the emitted Rust crate in dry-run mode

### Phase 2 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P2.1 | Gist pipeline test | **Done** | 11-file gist closure compiles with 0 diagnostics, 4 files emitted |
| P2.2 | Service operation bodies | Done | reqwest, `Command`, auth injection, dry-run mocking already landed |
| P2.3 | `main.rs` workflow dispatch | Done | Workflow subcommands and dispatch match arms already land |
| P2.4 | Multi-module extdep imports | **Done** | Verified via gist pipeline test; all 11 modules with transitive imports resolve |
| P2.5 | Emitted crate build/run | Needs verification | Test cleans up output; needs infrastructure to preserve and build emitted crate |

### Current Emitted Bundle Shape

Today the emitted Rust crate is already conceptually the right bundle:

```text
output_dir/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── v2_rt.rs
│   ├── gist.rs
│   ├── github_api.rs
│   ├── git.rs
│   └── ...
```

That bundle currently comes out of `06_pipeline.dag` plus the Rust
emitter. After M1 it should be understood as the output of `compile.dag`.

### Phase 2 Exit Criteria

- `cargo test -p v2-compiler-tests v2_gist_full_pipeline -- --ignored` passes
- The emitted gist crate builds and runs in dry-run mode
- No v1-only post-processing step is required to make the crate buildable

---

## Phase 3: Compile Contract, Pipeline Completion, and v1 Retirement

**Gate:** v2 compiles everything that still matters from v1, ownership is
pipeline-wired, artifact planning is real, and v1 is no longer on the
critical path.

This phase owns the compile contract work: M2, M3, and M4, plus R8 and
R9.

### Phase 3 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P3.1 | Verify parity with remaining v1 paths | **Done** | Two root causes identified: tuple field naming (fixed), if-branch type unification (fixed). Remaining 197 diagnostics are field access resolution issues. |
| P3.2 | Ownership wiring + authoritative compile bundle | In progress | `compile_sources` now returns `complexity`, `ownership`, and `artifact_plan`, and emit dispatch follows the planned artifact target; unsupported obligations/reporting still need consolidation |
| P3.3 | Artifact planning above emit | In progress | Default single-artifact planning now runs between infer and emit through the real artifact contract; real partitioning and per-artifact orchestration remain |
| P3.4 | Runtime shim dissolution | Planned | Move the remaining v1 runtime shim pieces into `.dag` runtime templates |
| P3.5 | Feature-gate v1 | **Done** | v1 crates gated behind `v1-bootstrap` feature; `cargo test -p v2-compiler-tests` runs 0 tests without feature |

### Key Decisions for Phase 3

- The compile result stops being just `files + diagnostics`
- Ownership becomes a first-class pipeline output, not a side analysis
- Artifact planning becomes part of the real compile flow, not a side
  module with stringly targets
- Unsupported proof or validation obligations must surface explicitly

### Phase 3 Exit Criteria

- The compile bundle has one authoritative typed shape
- ownership is included alongside complexity in the pipeline output
- artifact planning runs between infer and emit in the primary compile path
- v1 is no longer required for normal compilation

---

## Phase 4: Shared Emit, Projections, and Backend Boundaries

**Gate:** adding a new backend means writing language facts plus a
compiler-owned adapter, with no changes to the shared compiler core.

This phase owns M5, M6, and M7. M8 follows only after the Phase 4
contract is real.

### Design Rules for Phase 4

- Shared emit owns traversal
- Compiler-owned target adapters own program-dependent lowering
- `dsl/extdeps/languages/*` stays declarative
- Generated tests are first-class outputs, not Rust-only emitter details
- The DAG backend remains a compile target; execution stays in a runtime

### Phase 4 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P4.1 | `LanguageSpec` becomes the single authority | Planned | Shared emit already imports extdep language tables; remaining duplication must collapse into one contract |
| P4.2 | Shared emit fold + target adapters | Planned | Highest-risk refactor; Rust/Python/Go still own full tree dispatch today |
| P4.3 | Generated tests as first-class projection | Planned | Preserve the current Rust path while generalizing the contract |
| P4.4 | DAG backend/runtime boundary | Planned | Add canonical DAG artifact and keep execution downstream |
| P4.5 | Typed backend plumbing and CLI surface | Planned | Backend selection should stop being stringly |
| P4.6 | Equivalence validation | Planned | Self-compile and gist must still converge after shared emit lands |

### Current Phase 4 Risks

- Shared emit is still helper-only; traversal is still per target
- Go main expression emission still contains a placeholder fallback
- `LanguageSpec` exists, but emit does not yet read it as the single
  source of truth
- Generated tests are still mostly a Rust-specific path

### Phase 4 Exit Criteria

- No backend owns a whole-tree `ExprData` dispatcher
- No backend owns a separate whole-tree TCO walker
- `LanguageSpec` is the single authority for language facts
- Generated tests are first-class artifact outputs
- The DAG backend emits a canonical artifact without embedding an
  interpreter in the compiler stages

---

## Phase 5: Convergence (L2 Preparation)

**Gate:** one `Node`-centric internal model flows through the compiler,
with the naming cleanup already landed and the bootstrap architecture
stable enough to make the deeper dissolutions worth doing.

This phase is intentionally later. It should happen after the naming,
pipeline, and shared emit boundaries stop moving.

### Phase 5 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P5.1 | Token dissolution | Planned | Replace `Token` / `TokenKind` structures with `Node` compositions |
| P5.2 | Module/import dissolution | Planned | Dissolve `Module`, `Import`, and `ImportNames` into `Node` compositions |
| P5.3 | Diagnostic / compile-output dissolution | Planned | Dissolve `Diagnostic`, `Severity`, `CompileResult`, and `TextFile` where it is still valuable |
| P5.4 | Service/support type dissolution | Planned | Verify which service-layer types still need to move |
| P5.5 | Residual semantic enum cleanup | Planned | Move remaining compiler-only semantic types toward `.dag` or `Node`-based representation where appropriate |

### Phase 5 Exit Criteria

- Target filenames from M1 are fully normalized
- Compiler-internal structure is consistently `Node`-centric
- Each convergence step survives re-bootstrap and fixed-point verification
- The compiler is in a clean place to start real L2 work

---

## Cross-Cutting Reference

The sections below support the phase plan above. They do not override it.

### Structural Pass Order (`S*`)

| Pass | Primary phase | Meaning |
|------|---------------|---------|
| S1 | Done | Theme 4: `kernel_types` / `is_kernel_type` are single-authority in core |
| S2 | Done | Theme 6: pipeline owns compilation only; artifact/trace are honest side systems |
| S3 | Done | Theme 3: known-method resolution is centralized; complexity follows semantics |
| S3.5 | Phase 1 | Extract emit metadata out of infer/reconcile |
| S4 | Phase 1 | Move Rust-only ownership/render policy out of core + infer |
| S5 | Phase 4 | Fuse duplicated `ExprData` walks behind shared fold machinery |
| S6 | Phase 4 | Shared emit dispatch with per-target leaves |
| S7 | Phase 1 / Phase 4 | Remove fabrication fallbacks and finish residual string-keyed cleanup |

### Compositional Refactor Targets (`R*`)

These are written in post-M1 names.

| ID | Module | Current -> Target | Primary phase | Note |
|----|--------|-------------------|---------------|------|
| R1 | `00_core.dag` | C -> A | Phase 1 / 3 | Remove emit/pipeline-only types from core |
| R2 | `01_tokenize.dag` | A -> A | Done | No structural refactor required |
| R3 | `02_parse.dag` | B+ -> A | Phase 1 follow-through | Mostly inherits R1 cleanup |
| R4 | `03_resolve.dag` | A -> A | Done | No structural refactor required |
| R5 | `04_infer.dag` | D -> B+ | Phase 1 | Bootstrap-critical infer cleanup |
| R6 | `05_emit*.dag` | D -> B+ | Phase 4 | Shared traversal plus target adapters |
| R7 | `complexity.dag` | B+ -> A | Phase 4 | Convert complexity into a fold consumer |
| R8 | `ownership.dag` | A- -> A | Phase 3 | Wire ownership into the pipeline |
| R9 | `compile.dag` | B- -> A | Phase 3 | Complete orchestration and typed backend flow |

Practical notes:

- **R5 is the bootstrap-critical refactor.** It is the first high-value
  cleanup inside the current infer/reconcile hotspot.
- **R6 is the highest-risk refactor.** Do it only after the compile
  contract and naming cleanup are stable enough to support it.
- **R8 and R9 are Phase 3 work.** They should not wait for deep
  convergence.

### Architecture Migration Tracks (`M*`)

| ID | Track | Primary phase | Depends on | Outcome |
|----|-------|---------------|------------|---------|
| M1 | Stage/module naming cleanup | Phase 1 | none | Target filenames and stage naming are coherent |
| M2 | Compile bundle + projection contracts | Phase 3 | M1 | One authoritative compile result shape |
| M3 | Artifact planning above emit | Phase 3 | M2 | `infer -> plan -> emit` is real |
| M4 | Proof/obligation derivation contract | Phase 3 | none | Proofs/tests/reports share one contract and unsupported is explicit |
| M5 | Generated tests as first-class projection | Phase 4 | M3, M4 | Generated tests become artifact outputs, not a Rust side path |
| M6 | Shared emit spine + target adapters | Phase 4 | M3 | Shared traversal plus compiler-owned adapters |
| M7 | DAG backend/runtime boundary | Phase 4 | M2, M3, M4 | Canonical DAG artifact with runtime kept downstream |
| M8 | Mixed-backend artifact boundaries | Late Phase 4 / later | M3, M5, M7 | Typed boundary plans and generated validation across artifacts |

---

## Business Feature Track: Agent Workflow Vertical Slice

This track stays parallel to compiler convergence. Its job is to prove one
real business integration without forking the architecture.

### Guardrails

- Do not block all product value on perfect compiler convergence
- Keep the first integration narrow, typed, and auditable
- Use the first real integration to pressure-test compiler/runtime
  contracts
- Do not build a parallel ad hoc system around compiler gaps

### Preferred First Integration

The first target remains the Cursor cloud agent API / Composer 2 surface.
The exact upstream API shape must be re-verified against current docs when
implementation starts. This roadmap item is about the integration shape,
not freezing an external API contract in advance.

### Business Track Timing

- AG1 modeling can start once Phase 2 proves the compiler can emit a real
  program
- Modeling work can overlap with late Phase 2 if it does not need the full
  compile path yet
- AG2 and AG3 should not outrun the compiler contract they depend on

### AG Workboard

| ID | Item | Status | Acceptance |
|----|------|--------|------------|
| AG1 | Model the cloud agent API in `.dag` | Planned | One typed lifecycle covers credentials, request payload, agent/run handle, optional follow-up handle, result payload, and cleanup |
| AG2 | Run one end-to-end happy path | Planned | `auth upsert -> launch -> follow-up -> delete` works end to end and is auditable |
| AG3 | Record the integration challenges | Planned | Real friction points are written down, classified, and fed back into the main roadmap |

### Generated Validation Expectations

The first workflow should carry generated validation from day 0.

Generated unit-style validation:

- Missing key returns `NeedsManualProvision`
- Invalid key fails explicit validation
- Valid key returns a ready handle
- Launch, follow-up, and delete request shaping are correct

Generated integration-style validation:

- `auth upsert -> launch -> follow-up -> delete` succeeds against mocked
  responses
- Cleanup invalidates any local state/handles created for the workflow
- Follow-up after delete fails in a controlled, typed way
- Repeated delete is either idempotent or returns an explicit expected
  error

Review bar:

- Tests must prove meaningful contract behavior, not tautologies
- At least one negative-path case exists for auth validation and
  post-delete behavior
- Failures are human-legible without reading generator internals
- Anything already proven structurally by the compiler should move into
  compile-time proof, not remain as a tautological runtime test

Out of scope for the first workflow:

- PR creation/review/follow-up management
- Repository discovery beyond what the happy path needs
- Artifact download flows unless the happy path proves they are required

---

## Backlog

Items below are real, but they are not on the critical path for the
current phase order.

### Language Features

| Item | Why deferred |
|------|--------------|
| General generic syntax | Special-cased `Result` / `Option` is enough for bootstrap scope |
| Full linear type checking | Ownership proof work has started, but full proof remains beyond the current migration |
| Widen V5 | The conservative version covers current hot paths |

### Compiler Improvements

| Item | Why deferred |
|------|--------------|
| Anonymous record target resolution | Must fail closed, but is not blocking active phases |
| Collection intrinsic semantics in shared IR | Worth doing after shared emit is real |
| Generated self-hosting tests and stage contracts | Valuable once the compile contract settles |
| TCO backend contract | Should be cleaned up during/after shared emit extraction |
| SCC-aware return type resolution | Not currently blocking bootstrap |

### Open Invariant Follow-Ups

| Item | What remains |
|------|--------------|
| Residual closed sets represented as strings | Finish the infer/source classifier cleanup and remove the remaining leaf-level string dispatch |
| Fabricating fallbacks | Remove placeholder or error-masking paths such as the Go emitter wildcard and residual error-as-value sites |
| Error normalization | Promote semantic warning-as-error boundaries where appropriate and normalize producer sites |

---

## Verification

| Gate | Command | When |
|------|---------|------|
| Unit tests | `cargo test --workspace --exclude v2-compiler-tests` | After every change |
| Clippy | `cargo clippy --all-targets -- -D warnings` | After every change |
| V2 non-bootstrap | `cargo test -p v2-compiler-tests --features v1-bootstrap` | After every change |
| Diagnostics ratchet | `cargo test -p v2-compiler-tests --features v1-bootstrap v2_strict_compile_diagnostic_count -- --ignored` | End of Phase 1 |
| Fixed point | `cargo test -p v2-compiler-tests --features v1-bootstrap v2_bootstrap_fixed_point -- --ignored` | After any `.dag` change that affects bootstrap output |
| Gist pipeline | `cargo test -p v2-compiler-tests --features v1-bootstrap v2_gist_full_pipeline -- --ignored` | End of Phase 2 |
| L1 ratchet | `scripts/l1-ratchet.sh --check` | After any `.dag` change (goal: 0) |

Manual Phase 2 smoke still exists in addition to the automated test:
build the emitted gist crate and run it in dry-run mode. There is not yet
a dedicated `v2_gist_end_to_end` test in the tree, so the roadmap should
not pretend that one exists.
