# gunbc Roadmap

## Architectural Thesis

**Node and DAG are the only compiler primitives.**

The compiler is a generic graph processor. It reads .dag source, builds
a graph of Nodes, applies structural rules, and emits target code. All
domain knowledge — types, cardinality, containers, optionality — lives
in .dag definitions, not in the compiler.

This thesis is implemented in three layers, each dissolving a category
of compiler knowledge:

| Layer | What dissolves | Compiler stops knowing | Status |
|-------|---------------|----------------------|--------|
| **L1: Types** | BuiltinTypeKind, Conj/Disj, node_is_*, type constructors | What Optional/List/Map/Int mean | **Active — 489 violations** |
| **L2: Expressions** | ExprData semantic knowledge | What if/for/match/let mean | Future — after bootstrap |
| **L3: Syntax** | Hardcoded parser branches | How to parse `if cond { body }` | Future — data-driven parser |

**Why L1 is urgent:** Every new feature that touches types adds more
`if n.name == "Map"` checks. Without a boundary, the violation count
climbs until the project loses all shape. L1 sets the direction.

**L2 and L3 are noted but not blocking.** L2 becomes practical once the
compiler is self-hosted and the expression walker is a shared fold. L3
is a parser generator — well beyond bootstrap scope but the right
long-term target.

---

## L1: Type Knowledge Dissolution

### Current violations (489 sites across 8 files, measured 2026-03-22)

| Category | Count | What the compiler "knows" |
|----------|-------|--------------------------|
| Connective field + Conj/Disj | 265 | Product vs coproduct semantics (every `connective` field ref) |
| Type constructors | 121 | `leaf_node()` 97, `optional_node()` 11, `container_node()` 7, `tuple_node()` 1, etc. |
| Type name comparisons | 59 | `.name == "Optional"`, `"Map"`, `"Dynamic"`, etc. |
| node_is_* predicates | 24 | `node_is_optional` 7, `node_is_map` 5, `node_is_dynamic` 5, etc. (calls only) |
| builtin_type_kind() | 20 | 14-branch if-else classifying types by name (calls only) |

### Strategy: properties-first migration

The compiler currently identifies types by name (`n.name == "Optional"`)
and by connective (`n.connective == Disj`). The migration replaces this
with property lookups on the Node graph. Types carry their own metadata
as .dag-defined properties; the compiler reads properties, not names.

**Step 1 — Define type properties in .dag:**
Types in `dsl/std/types.dag` carry properties that encode what the
emitter needs. The compiler discovers these by reading the graph.

**Step 2 — Replace compiler knowledge with property reads:**
Each `builtin_type_kind()` call, each `node_is_optional()` call, each
`n.name == "Map"` check becomes a property lookup. Incremental — one
type family at a time.

**Step 3 — Delete the primitives:**
Once all consumers read properties: delete `BuiltinTypeKind` enum,
`builtin_type_kind()`, all `node_is_*` predicates, type constructor
functions. Eventually remove `connective` field from Node (largest
change — 267 sites).

### Ratchet

Count of sites where the compiler uses type-specific knowledge.

```
Start:  489
Target: 0
```

Measured by: `rg -c 'connective|Conj|Disj' src/v2/*.dag` (265) +
`rg -c 'leaf_node\(|optional_node\(|container_node\(|tuple_node\(' src/v2/*.dag` (121) +
`rg -c '\.name == "(Optional|Map|List|Int|String|Bool|Dynamic|...)"' src/v2/*.dag` (59) +
`node_is_*` calls excluding definitions (24) +
`builtin_type_kind` calls excluding definition (20).

Note: categories overlap slightly (e.g., `connective: Some { value: Conj }`
counts in both connective and Conj). The total is approximate but directionally
correct — the goal is 0, not precise accounting.

### Acceptance

- `BuiltinTypeKind` enum deleted from `00_core.dag`
- `builtin_type_kind()` function deleted
- All `node_is_*` predicates deleted or replaced with property reads
- `optional_node()`, `container_node()`, `pair_node()` deleted
- `connective` field removed from Node (Conj/Disj dissolved)
- Zero type-name string matching in the compiler
- All type knowledge lives in `dsl/std/types.dag` or `dsl/extdeps/`
- Fixed point still holds

---

## Completed Milestones

| Milestone | Gate | Date |
|-----------|------|------|
| Self-compile pipeline | v2 processes own .dag through all 5 stages | 2026-03 |
| Bootstrap A5 | v1 → stage0 → stage1 (cargo check ✓) | 2026-03 |
| Fixed point A6 | stage1 output == stage2 output (byte-identical) | 2026-03 |
| A7 Phase 1 | Self-compile: 0 cargo check errors | 2026-03 |
| TypeExpr→Node | 8 TypeExpr variants deleted | 2026-03 |
| Expr→Node | 21 Expr variants deleted, ExprData discriminator on Node | 2026-03 |
| Transport dissolution | TransportBinding (4 variants) deleted | 2026-03 |
| Node/TypedNode unified IR | W1–W13 complete, 129 tests passing | 2026-03 |
| Performance audit | 50,000x improvement (tokenize+parse: 24ms) | 2026-03 |
| OOM fix | node_type_deps container-wrapped cycle detection | 2026-03 |

---

## End Goal

The compiler is a generic graph processor. It reads .dag source, builds
a graph of Nodes, applies structural rules defined in .dag, and emits
target code. Adding a type, expression, language, or transport means
editing .dag files — never compiler code.

Concrete acceptance:
- Zero type-world knowledge in the compiler (L1 complete)
- One shared emit walker drives all target languages via LanguageSpec
- Ownership and complexity proofs wired into compile pipeline
- At least one real program (gist) compiles and runs end-to-end
- v1 is archived
- All compiler-internal types are Node compositions

---

## Current Compositional State (2026-03-22 Audit)

This audit is not a new roadmap phase. It is a map of where the compiler
currently behaves like the `extdeps/` compositional model and where it still
collapses layer authority.

| Layer | Current state | Meaning for the next passes |
|------|------|------|
| `00_core.dag` | Strong foundation, mostly target-agnostic | `Node`/`ExprData`/transport modeling is the right base. Core now owns kernel-type authority and the shared self-call classifier; remaining ownership leakage sits downstream in reconcile/emit rather than on core types. |
| `01_tokenize.dag` | Mostly clean syntax leaf | Tokenization is structurally isolated; bootstrap-specific Rc commentary and `SourceRef` are still host-artifact leakage. |
| `02_parse.dag` | Strong compositional lowering | Service/resource syntax already dissolves into uniform `Node` structure and records facts like `namespace_root` structurally. |
| `03_resolve.dag` | Cleanest authority boundary | Pure import graph construction with almost no target leakage. Keep using this as the reference for stage boundaries. |
| `04_reconcile.dag` | Main structural hotspot (4871 LOC) | 5+ mixed concerns: type inference (~1500), type resolution (~800), method classification/call analysis (~650, 80+ string comparisons), emitter metadata prep (~300, IR/rendering conflation), type env management (~400). Target-agnostic but concept-overloaded. |
| `05_emit*.dag` | Partial extdeps-style composition | Shared emit (799 LOC) owns helpers/context but 0 expression dispatch. Rust (3634 LOC), Python (1202 LOC), Go (1226 LOC) each own full 22-arm ExprData dispatchers. 3 separate TCO walkers duplicate Let/If/Match/Block. Go TCO has `_` wildcard emitting `/* unhandled expr */`. |
| `07_complexity.dag` / `07_ownership.dag` | Good proof layers | Best examples of compositional modeling: proof objects, not runtime execution. complexity (1441 LOC) and ownership (307 LOC) both independently walk all ExprData variants — 3 total parallel walks including reconcile. Ownership is not wired into pipeline (complexity is). |
| `06_pipeline.dag` / `08_artifact.dag` / `09_trace.dag` | Narrowed to honest boundaries | `06_pipeline.dag` (177 LOC) owns compile flow but does not call ownership or artifact planning. `08_artifact.dag` (235 LOC) has real boundary verification logic but `Artifact.target` is still a `String`. `09_trace.dag` (221 LOC) is an external contract, not pipeline-wired. |

### Audit Reconciliation

- The v2 audit in `INVARIANTS.md` (repo root) is directionally correct, but several
  items in the roadmap need reinterpretation based on what has already landed.
- Theme 4 and Theme 6 are cross-cutting prerequisites that remove duplicate
  authority and dead branches before deeper semantic changes.
- P1.8 is complete: `classify_method_cost` deleted from complexity.
  `07_complexity.dag` has `intrinsic_method_cost_shape(...)`,
  `cost_of_expr(...)` reads reconcile-provided `method_semantics`,
  `receiver_size_var(...)` follows semantics instead of string names, and
  `04_reconcile.dag` is the single authority via
  `classify_reconciled_intrinsic_method`. No dual classification remains.
- P4.1: shared emit already imports language type/keyword/container data from
  `extdeps.languages.*`. Remaining duplication is per-target reserved-word/runtime
  tables, especially in the Python and Go renderers.
- Trace: `09_trace.dag` is an external runtime contract, not an in-compiler
  interpreter. Remaining decision is whether runtime adapters/source maps get
  wired into the pipeline or remain explicitly external.

### Structural Pass Order

These passes cut across phases. They should be executed in this order because each one
reduces the cost or risk of the next.

| Pass | Theme | What it changes first |
|------|------|------|
| S1 | Theme 4 | Done: `kernel_types` and `is_kernel_type` are single-authority in `00_core.dag` |
| S2 | Theme 6 | Done: pipeline owns compilation only, artifact is explicit-only, trace is an external contract |
| S3 | Theme 3 | Done: known-method resolution is centralized and complexity follows semantics; renderer/runtime cleanup remains |
| S3.5 | Theme 3/5 | Extract emitter metadata (EmitGraphInfo, TypeSummary, FieldSummary, MethodSemantics) from reconcile into a post-inference pass; reconcile should infer types, not prepare rendering data |
| S4 | Theme 5 | Move Rust-only ownership/render policy out of core + reconcile |
| S5 | Theme 1 | Fuse 3 duplicated `ExprData` walks (reconcile, complexity, ownership) into shared `fold_expr` with callbacks |
| S6 | Theme 2 | Shared emit dispatch with per-target leaves |
| S7 | Theme 7 | Final fabrication fallback cleanup and Dynamic-site audit; `04_reconcile.dag` has 80+ string literal comparisons across 7 dispatch patterns for method classification |

S1–S3 are done. The structural passes (S3.5–S7) continue as
cross-cutting work alongside Phases 2–5.

---

## Compositional Refactor Targets (post-M1)

These targets assume M1 renames are complete. Each entry references the
target DAG models in `src/v2/DESIGN.md` (Compositional stage targets) and
cross-references the S-passes/M-tracks that deliver them.

| ID | Stage | Current | Target | Cross-refs |
|----|-------|---------|--------|------------|
| R1 | `00_core.dag` | C — conflated kernel + emit types | A — kernel vocabulary only | S3.5, S4 |
| R2 | `01_tokenize.dag` | A — gold standard | A — no change | — |
| R3 | `02_parse.dag` | B+ — inherits core width | A — inherits R1 cleanup | R1 |
| R4 | `03_resolve.dag` | A — gold standard | A — no change | — |
| R5 | `04_infer.dag` | D — 5 concerns, 80+ string comparisons | B+ — data tables, no emit metadata | S3.5, S7 |
| R6 | `05_emit*.dag` | D — 3 parallel walkers | B+ — one fold, LanguageSpec-driven | S5, S6, M6, P4.2, P4.3 |
| R7 | `07_complexity.dag` | B+ — independent walker | A — fold consumer | S5 |
| R8 | `07_ownership.dag` | A- — unwired | A — wired into pipeline | M2 |
| R9 | `compile.dag` | B- — incomplete | A — complete orchestration | M1, M2, M3 |

#### R1. Core vocabulary scoping (00_core.dag)

What changes:

- Move `FieldSummary`, `TypeSummary`, `EmitGraphInfo`, `EmitInfoBuildState`
  out of core into the emit layer (they are produced and consumed only by emit)
- Move `RenderTarget` to `compile.dag` (orchestration concern)

Acceptance criteria:

- `00_core.dag` has zero types that only emit or pipeline orchestration consume
- No downstream module imports a rendering type from core
- All tests pass with types at their new locations

Cleanup / deletion:

- Delete re-exports or compatibility aliases for moved types
- Delete any comments in core referencing "emit info" or "render target"

#### R2–R4. Tokenize, parse, resolve — no refactor needed

These stages follow the gold-standard compositional pattern. `02_parse.dag`
import list shrinks automatically when R1 narrows core.

#### R5. Infer data tables (04_infer.dag)

What changes:

- Replace 80+ string literal method comparisons with a single data table:
  `data intrinsic_methods: Map<String, IntrinsicMethod>` (same pattern as
  tokenizer's `data keywords: Map<String, TokenKind>`)
- Extract EmitGraphInfo computation out of infer — either into emit itself
  or into a thin post-infer pass (S3.5)
- Output type becomes `InferredGraph` with no `emit_info` field

Acceptance criteria:

- Zero string literal method dispatch in infer — all via table lookup
- Infer output type contains no rendering metadata (no EmitGraphInfo,
  TypeSummary, or FieldSummary)
- Adding a new intrinsic method = one table entry, not new match arms
- All tests pass

Cleanup / deletion:

- Delete every `if method_name == "..."` string comparison for method
  classification (currently 80+ across 7 dispatch patterns)
- Delete `emit_info` field from infer output type
- Delete `EmitInfoBuildState` accumulator from infer
- Delete any helper in infer that builds TypeSummary or FieldSummary
  (moves to emit layer)

#### R6. Shared emit fold + target adapters (05_emit*.dag)

What changes:

- One ExprData fold in `05_emit.dag` replaces 3 per-target dispatchers
  (all three currently have 22-arm dispatchers)
- One TCO walker replaces 3 per-target TCO walkers
  (currently 3x duplication of Let/If/Match/Block)
- `LanguageSpec` from `dsl/std/languages.dag` parameterizes shared emit
- Per-target files shrink to irreducible rendering hooks

Acceptance criteria:

- No per-target file owns a full ExprData dispatcher
- No per-target file owns a separate TCO walker
- Python TCO silent fallthrough is eliminated
- Go TCO crash on unhandled expression is eliminated
- `classify_typed_item` remains the shared entry point (already wired)
- Adding a new ExprData variant = one match arm in shared emit, not N
  arms across N targets
- `LanguageSpec` is the source of truth for type maps, keywords,
  container templates — no inline duplicates in per-target files

Cleanup / deletion:

- Delete per-target ExprData dispatchers after shared fold lands
- Delete per-target TCO walkers after shared TCO implementation lands
- Delete `classify_typed_item` if still uncalled after refactor
- Delete inline keyword/type-map/reserved-word declarations that
  duplicate data in `LanguageSpec` or `extdeps/languages/*/emit.dag`
- Delete any triple-duplicated data (std.languages + extdeps + inline
  emit) — one source of truth only

#### R7. Complexity fold consumer (07_complexity.dag)

What changes:

- After shared `fold_expr` lands (S5), complexity becomes a fold consumer
  instead of maintaining its own ExprData walker
- Cost computation logic is unchanged — only the traversal method changes

Acceptance criteria:

- Complexity does not own an independent ExprData walk
- Cost algebra types and computation logic unchanged
- All complexity tests pass

Cleanup / deletion:

- Delete the standalone ExprData match in complexity (replaced by fold
  callback)

#### R8. Ownership pipeline wiring (07_ownership.dag)

What changes:

- `compile.dag` calls `analyze_ownership` alongside `build_complexity_report`
- Ownership proofs included in PipelineResult
- Emit can read proofs for Rust clone/borrow decisions

Acceptance criteria:

- `compile.dag` imports and calls ownership analysis
- PipelineResult includes ownership proofs
- Proofs are accessible to emit (at minimum for Rust target)
- All tests pass

Cleanup / deletion:

- Delete comments noting ownership is "not wired"
- Delete manual clone/borrow heuristics in emit that ownership proofs
  replace (if any exist)

#### R9. Pipeline completeness (compile.dag)

What changes:

- `Backend` enum defined here (moved from core's `RenderTarget`)
- Ownership analysis wired (R8)
- Artifact planning wired when M3 lands

Acceptance criteria:

- `Backend`/`RenderTarget` is defined in compile.dag, not in 00_core.dag
- All proof/analysis stages are called (complexity + ownership)
- Pipeline failure at any stage produces clear diagnostics, not silent skip

Cleanup / deletion:

- Delete `RenderTarget` from `00_core.dag` after move
- Delete comments noting ownership/artifact are "not wired"

#### Execution order

R5 is the bootstrap-critical refactor. The heuristics previously introduced
to reduce self-compile cargo check errors (Dynamic permissiveness,
fold-returns-Dynamic, var→func_env fallback, recursive field-access
resolution) were all symptoms of string-based method dispatch in reconcile.
R5 fixes the entire class at once via data tables — the same pattern that
makes tokenize gold-standard.

Priority order (bootstrap-integrated):

```
1. M1 (renames)        — mechanical prerequisite
2. R5 (data tables)    — holistic bootstrap fix: replaces 80+ string
   │                     comparisons with typed method contracts
   ├── R1 (core scoping) — can happen during or right after R5
   └── re-measure bootstrap errors (most should disappear)
3. R6 (shared emit fold) — fixes remaining emission divergence
   └── R7 (complexity fold consumer)
4. R8 + R9 (ownership wiring + pipeline completeness)
5. Resume bootstrap error reduction on correct infrastructure
```

R2–R4 are no-ops. R3 (parse) inherits R1's cleanup automatically.

---

## Architecture Migration Workboard

This board turns the current v2 gap analysis into executable migration
work. It is cross-cutting rather than phase-local: the point is to make
the target architecture in `src/v2/DESIGN.md` land incrementally without
creating another temporary system that has to be maintained forever.

The gap analysis is complete enough to schedule this work now. Future
audits may refine scope, but the main migration seams are already clear.

### Parallel tracks

| ID | Track | Status | Depends on | Can run in parallel with |
|----|------|--------|------------|---------------------------|
| M1 | Stage/module naming cleanup | Planned | none | M2, M4 |
| M2 | Compile bundle + projection contracts | Planned | none | M1, M3, M4 |
| M3 | Artifact planning above emit | Planned | none | M1, M2, M4 |
| M4 | Proof/obligation derivation contract | Planned | none | M1, M2, M3 |
| M5 | Generated tests as first-class projection | Planned | M3, M4 | M6 |
| M6 | Shared emit spine + target adapters | Planned | M3 | M5, M7 |
| M7 | DAG backend/runtime boundary | Planned | M2, M3, M4 | M6 |
| M8 | Mixed-backend artifact boundaries | Planned | M3, M5, M7 | none |

### M1. Stage/module naming cleanup

Current state:

- `04_reconcile.dag` still has the old name even though the intended
  role is infer/typecheck
- `06_pipeline.dag` is the compiler driver, not a sixth stage
- numbered/unnumbered responsibilities are still mixed in names and docs

Work:

- rename `04_reconcile.dag` -> `04_infer.dag`
- rename `06_pipeline.dag` -> `compile.dag`
- move `RenderTarget` enum out of `00_core.dag` into `compile.dag` or
  a targets metadata module (adding a new target should not require
  editing core)
- update imports, tests, docs, and bootstrap references
- ratchet the naming rule: numbered files are only core transformation
  stages

Acceptance criteria:

- all imports/tests/docs use `04_infer` (not `04_reconcile`) and `compile` (not `06_pipeline`)
- no numbered module remains that is not a core lowering stage
- the design/roadmap language no longer refers to the driver as "stage 6"

Deletion / cleanup:

- delete compatibility aliases or doc references for `06_pipeline`
- delete `04_reconcile` references once callers migrate

### M2. Compile bundle + projection contracts

Current state:

- compile interfaces are still mostly flat stage outputs or target-first
  entrypoints
- proofs/tests/reports are not yet surfaced as first-class outputs
- downstream components do not yet consume typed projections from one
  authoritative bundle
- `07_ownership.dag` is real and working (clean obligation model, no
  fallbacks) but `06_pipeline.dag` does not import or call it;
  complexity is wired, ownership is not

Work:

- define the authoritative compile result/bundle shape
- include typed graph, artifact plan, emitted runtime artifacts,
  validation artifacts, proof/report artifacts, obligations, and
  diagnostics in that bundle
- define typed projection/view contracts so downstream systems can peel
  off only the slices they need
- wire ownership analysis into the pipeline alongside complexity

Acceptance criteria:

- there is one authoritative compile bundle/result contract
- runtime/test/proof/report consumers can read typed views without
  depending on ad hoc side channels
- unsupported obligations are visible in the bundle/diagnostics, not
  silently dropped
- ownership analysis runs in the pipeline and its output is included in
  the compile bundle

Deletion / cleanup:

- delete hidden side-effect outputs that are not represented in the
  compile contract
- delete whole-system assumptions that compile only returns "files +
  diagnostics"

### M3. Artifact planning above emit

Current state:

- the top-level compile path still assumes one target for the whole
  compile
- `Artifact.target` is still stringly or under-modeled
- `artifact.dag` is not yet the selector of backends/boundaries for real
  compile flows

Work:

- move the canonical ordering to:
  `infer whole graph -> plan artifacts -> emit per artifact`
- make artifact target a typed backend, not a `String`
- keep the current single-target compile path only as a compatibility
  wrapper around a default one-artifact plan

Acceptance criteria:

- artifact planning runs after infer and before emit in the primary
  compile path
- `emit_artifact(...)` becomes the primary emit interface
- the single-target CLI/API path is implemented as a wrapper over a
  default artifact plan

Deletion / cleanup:

- delete the assumption that a whole project has exactly one target
- delete stringly artifact target fields after callers migrate

### M4. Proof/obligation derivation contract

Current state:

- `complexity.dag` and `ownership.dag` are real, but they still read
  partly like ad hoc analysis sidecars
- proofs, reports, and residual runtime obligations are not yet expressed
  through one shared contract

Work:

- define the shared vocabulary for projection roles and obligation
  outcomes
- make proof/analysis modules derive first-class outputs from the same
  typed graph as code emission
- require every proof family to end in one of three states:
  discharged statically, lowered to executable validation, or explicit
  `Unsupported`

Acceptance criteria:

- the proof/test/report model is documented as a shared projection model
- at least `complexity` and `ownership` fit that model explicitly
- unsupported proof obligations cannot disappear silently

Deletion / cleanup:

- delete doc language that treats proofs/tests as second-class cleanup
- delete future tendencies to smuggle rewrites into proof derivation
  modules

### M5. Generated tests as a first-class projection

Current state:

- generated tests still exist, but only through a narrow Rust-specific
  path
- test generation is not yet an artifact-level/shared contract

Work:

- preserve the current Rust `mock_response` path during migration
- derive runtime validation obligations from the typed graph plus
  boundary/fixture metadata
- surface generated tests as first-class artifact outputs rather than as
  a hidden emitter detail
- require each backend either to discharge its test obligations or to
  return explicit `Unsupported`

Acceptance criteria:

- Rust generated tests survive the emit refactor
- generated tests are represented in the artifact/compile contract
- at least one backend discharges hermetic generated tests through the
  shared contract
- missing backend validation support is explicit, not silent

Deletion / cleanup:

- delete Rust-only ownership of test generation once the shared contract
  exists
- delete stale claims that multi-backend generated tests already exist

### M6. Shared emit spine + target adapters

Current state:

- `05_emit.dag` has shared helpers/context, but Rust/Python/Go still own
  full expression and TCO walkers (all three: 22-arm ExprData dispatchers;
  Rust: 3634 LOC; Python: 1202 LOC; Go: 1226 LOC)
- 4 expression kinds (Let/If/Match/Block) are duplicated 3x in TCO
  walkers
- `classify_typed_item` exists in shared emit and is called by all
  three emitters
- Python TCO walker has no else arm (silent fallthrough); Go TCO walker
  has no wildcard (crash on unhandled expression)
- target policy is still split across shared emit and target files

Work:

- move traversal/dispatch into shared emit
- wire or replace the unused `classify_typed_item` in shared emit
- fix Python/Go TCO silent failure modes before or during extraction
- reduce target files to compiler-owned adapters under `src/v2/targets/*`
- keep `dsl/extdeps/languages/*` declarative only
- make adding a backend mean adding language facts plus an adapter, not a
  fourth whole emitter

Acceptance criteria:

- no target adapter owns a full whole-tree `ExprData` dispatcher
- no target adapter owns a separate whole-tree TCO walker
- shared emit can drive Rust/Python/Go through one traversal spine
- no silent fallthrough or crash path in TCO dispatch

Deletion / cleanup:

- delete per-target whole-expression dispatchers after shared dispatch
  lands
- delete duplicate TCO walkers after shared TCO dispatch lands

### M7. DAG backend/runtime boundary

Current state:

- DAG execution is now modeled correctly as downstream of compile, but no
  canonical DAG backend exists yet
- there is no runtime/interpreter consuming a v2 DAG artifact

Work:

- add `Dag` as a first-class backend
- define the canonical DAG artifact/bundle schema
- add a compiler-owned DAG target adapter
- keep execution in `runtimes/dag/*` or equivalent downstream runtime
  modules

Acceptance criteria:

- compiler can emit a canonical DAG artifact without embedding an
  interpreter in the core stages
- the DAG runtime boundary is explicit in code/docs
- interpretation/JIT remain runtime strategies over the same artifact

Deletion / cleanup:

- delete wording or interfaces that assume compile outputs are only
  native source files
- delete any design drift back toward an interpreter embedded in the
  compile stages

### M8. Mixed-backend artifact boundaries

Current state:

- the design now allows per-artifact backends, but the compiler does not
  yet really plan or validate mixed-backend boundaries
- backend mixing is still more conceptual than executable

Work:

- make artifact planning choose backend per artifact
- express cross-artifact boundaries explicitly and lower them through
  known boundary kinds
- generate boundary adapters/contracts/tests from those boundary plans

Acceptance criteria:

- at least one explicit boundary kind can generate both a runtime adapter
  and a hermetic validation artifact
- mixed-backend compilation works at artifact boundaries, not through ad
  hoc per-node backend mixing
- boundary compatibility remains a proofable property, not just a runtime
  hope

Deletion / cleanup:

- delete ad hoc direct backend-to-backend assumptions that bypass
  artifact boundaries
- delete stringly boundary target handling once typed backends land

### Suggested execution order

The tracks are parallelizable, but the least-wasteful order is:

1. M1 + M4
2. M2 + M3
3. M5 + M6
4. M7
5. M8

This order keeps interface and ownership decisions ahead of the bigger
refactors, preserves generated tests during the emit migration, and
avoids building a DAG backend on top of the wrong compile contract.

---

## Business Feature Track: Agent Workflow Vertical Slice

This track is intentionally parallel to compiler convergence. The goal
is to get one real agent integration working as soon as possible
without waiting for the full target
architecture to be complete.

The principle is:

- do not block on perfect compiler convergence before proving business
  value
- keep the first integration narrow, typed, and auditable
- use the integration work to pressure-test the compiler/runtime
  contracts, not to build a parallel ad hoc system
- do not solve the full agent platform first; land one minimal cloud
  agent happy path that is operationally real

### First task: cloud agent API integration

The first business task is deliberately small:

- model the cloud agent API in `.dag`
- model auth upsert for that provider as a workflow
- run one end-to-end happy path through that model:
  authenticate -> launch agent with simple prompt -> add follow-up ->
  delete agent
- observe what structural/compiler/runtime challenges the integration
  reveals

Today the preferred target is the Cursor cloud agent API / Composer 2
surface. The exact external API shape should be verified against the
current upstream docs when implementation starts; this roadmap item is
about the integration shape and the questions it should answer.

### AG1. Model the cloud agent API in `.dag`

Current state:

- there is no first-class `.dag` model yet for a cloud agent API
- agent integrations are still conceptual rather than encoded as typed
  compiler/runtime inputs and outputs

Work:

- define the narrowest useful `.dag` model for one cloud agent request
  lifecycle:
  credential/secret reference, request payload, agent/run handle,
  optional follow-up/session handle, result payload, and delete/cleanup
  operations
- define auth upsert for the provider as part of that lifecycle:
  detect missing credential, instruct an admin where to create it,
  reconcile it into secret storage, validate it, and return a ready
  handle
- keep the model transport-oriented and concrete rather than inventing a
  generic agent ontology up front
- make secret references and cleanup/deletion explicit in the model

Acceptance criteria:

- one cloud agent request/response lifecycle can be represented as a
  typed `.dag` program
- auth upsert for the provider can also be represented as a typed `.dag`
  workflow, not as undocumented setup glue
- credentials/secrets are modeled as explicit references/handles, not
  plain payload data
- cleanup/deletion is part of the model, not an out-of-band note

Deletion / cleanup:

- delete any prototype modeling that encodes secrets as ordinary payloads
- delete generic agent abstractions that are not required for the first
  integration

### AG2. Run one end-to-end happy path

Current state:

- no real cloud-agent-backed workflow is running through the v2 model yet

Work:

- wire one happy path from auth upsert -> request -> launch agent ->
  add follow-up -> delete agent
- support only the minimal state needed for follow-up/resume if the API
  actually requires it
- capture run metadata/audit output so the lifecycle is inspectable
- explicitly defer PR management/review flows until the base lifecycle is
  working cleanly

Acceptance criteria:

- one `.dag`-modeled auth-upsert workflow can guide/administer manual key
  provisioning into GCP Secret Manager, validate it, and return a ready
  handle
- one `.dag`-modeled workflow can authenticate and perform:
  launch -> follow-up -> delete
- the happy path is auditable end to end
- state persistence is either unnecessary or explicitly modeled as a
  minimal handle-based contract
- a cleanup/deletion path exists for any secret/state/run artifacts we
  create
- PR management/review automation is not required for this first slice

Deletion / cleanup:

- delete demo-only glue once the real happy path works

### Generated validation for the first workflow

The first workflow should carry generated validation from day 0. Keep it
narrow and tied to the exact lifecycle above.

Generated unit-style validation:

- auth upsert returns `NeedsManualProvision` when the Cursor key is
  missing
- auth upsert returns invalid/failed validation when `/v0/me` rejects the
  supplied key
- auth upsert returns a ready handle when the key exists and validates
- launch request shaping is correct for a simple prompt
- follow-up request shaping is correct for an existing agent handle
- delete request shaping is correct for an existing agent handle

Generated integration-style validation:

- auth upsert -> launch -> follow-up -> delete succeeds against mocked
  Cursor responses
- cleanup removes or invalidates any local state/handles created for the
  workflow
- follow-up after delete fails in a controlled/typed way
- repeated delete is either idempotent or produces an explicit expected
  error contract

Optional live/manual smoke:

- one ignored/manual test exercises the real Cursor API against a safe
  test repo and key

Review/acceptance bar for these generated tests:

- the unit tests must prove meaningful contract behavior, not merely that
  generated fields equal the same literals used to generate them
- the integration tests must validate an actual lifecycle boundary:
  auth validation, launch, follow-up, delete, or cleanup semantics
- at least one negative-path case must exist for auth validation and for
  post-delete behavior
- failures must be human-legible: a reviewer should be able to tell what
  contract regressed without reading generator internals
- if a generated test only reasserts something already proven
  structurally by the compiler, move that check into proof/compile-time
  validation instead of keeping a tautological runtime test

Reasoning/guarantee output required for this workflow:

- the workflow should emit a readable summary of:
  what is proven structurally, what is validated by generated tests, and
  what remains unsupported
- reviewers should not need to read generator internals to understand the
  coverage/guarantee split
- adding new generated tests later should read as an additive safety
  upgrade, not a patch required to keep unseen structures functioning

We should be happy with the first generated test set when:

- it covers the happy path and the most important failure edges
- each test is traceable to a concrete residual runtime obligation
- reviewers agree the tests would catch a realistic integration
  regression
- removing or breaking a key contract in the workflow would actually make
  at least one generated test fail
- a human can easily explain the guarantee ledger for the workflow:
  what is compile-time guaranteed, what is runtime-validated, and what is
  still unsupported

Out of scope for the first workflow:

- PR creation/review/follow-up management
- repository discovery/listing beyond what is required to launch one
  agent
- artifact download flows unless the happy path proves they are needed

### AG3. Record the integration challenges

Current state:

- we do not yet know which parts of the cloud agent API map cleanly into
  the current `.dag` model and which parts force design changes

Work:

- document the concrete friction points revealed by AG1/AG2
- classify each challenge as:
  model gap, compiler gap, runtime gap, secret/state-management gap, or
  upstream API mismatch
- feed the reusable parts back into the architecture/migration board

Examples of likely challenge areas:

- auth/secret acquisition and refresh
- auth upsert boundaries: what is manual provider provisioning vs what is
  automated in our system
- whether follow-up state is a hard API requirement or optional sugar
- async/polling/webhook vs synchronous completion
- file/tool attachments and result typing
- deletion semantics for agent runs, state, and stored artifacts

Acceptance criteria:

- there is a written challenge list from a real integration attempt
- each challenge is classified and attached to a concrete follow-up task
- the result informs the compiler roadmap instead of living as tribal
  knowledge

### Timing

AG1 can start after Phase 2 (gist e2e) — the compiler can emit real
programs at that point. Starting earlier risks building on an
unproven emit pipeline. AG1 modeling work (defining .dag types for the
cloud agent API) can start in parallel with Phase 2 if the modeling
doesn't require compilation.

### Relationship to compiler convergence

- AG1 should inform Phase 1 and Phase 4 rather than fork around them —
  if the cloud agent API exposes a type the compiler can't handle,
  that's a Phase 1 diagnostic fix, not a workaround
- AG2 should stay narrow and avoid forcing premature generality into the
  compiler
- AG3 should create concrete follow-up work for the relevant phase when
  the integration exposes real gaps

---

## Phase 1: Strict Soundness + Type Dissolution

Two interleaved goals:

**A. Diagnostics → 0** (`DIAG_RATCHET = 25` in `src/v2/tests/src/lib.rs`).
These are inference gaps — enumerate doesn't return `Tuple<Int, T>`,
fold doesn't infer accumulator type, map_insert loses key/value structure.
Fixing these is correctness work needed for gist (Phase 2).

**B. L1 dissolution** (489 type-knowledge violations → 0). This is the
architectural direction — compiler stops knowing what types mean. Each
type family migrates to .dag-defined properties.

**Why interleave:** Many diagnostic fixes require touching the same code
that dissolution targets (`infer_method_call_type_node`, `node_type_equals`,
`builtin_type_kind`). Doing inference fixes property-first (read .dag
properties instead of hardcoding) means each fix contributes to both goals.

**Bootstrap strategy:** Every step must preserve the fixed point
(stage1 == stage2 byte-identical). Type dissolution changes how the
compiler represents types internally, but the emitted output must be
identical. The approach is: add .dag properties → switch consumers to
read properties → delete old knowledge. The "switch consumers" step is
where the fixed point can break — verify after each consumer migration.

### Diagnostic fixes (Goal A)

These specific inference gaps produce the 25 remaining diagnostics.
Each fix should be done property-first when possible (contributing to
Goal B simultaneously).

| Fix | What | Status |
|-----|------|--------|
| Enumerate return type | Returns `List<Tuple<Int, T>>` via `tuple_node` | **Done** (quick-emu-93) |
| Fold accumulator | `fold_accumulator_type` parameter threads init-arg type | **Done** (quick-emu-93) |
| Callable/function-value type | `callable_node` + `callable_return_type` for function values | **Done** (quick-emu-93) |
| ErrorCategory enum | 7-variant structured error classification | **Done** (quick-emu-93) |
| map_insert/map_merge | `infer_builtin_call_type` still returns bare `leaf_node(name: "Map")` | Remaining |
| Chained field access | Depends on map fixes — once return types are real, field access resolves | Remaining |
| Tighten node_type_equals | Remove Dynamic==* and structural fallback — MUST BE LAST | Remaining |

**Files:** `04_reconcile.dag` (remaining fixes), `05_emit_rust.dag` (rendering)

### L1 dissolution by type family (Goal B)

### L1.1 — Optional/cardinality

**Current violations:** ~60 sites (node_is_optional, optional_node,
OptionalUnwrap, OptionalValue, Unit==Optional in node_type_equals,
Some/None synthesis, is_already_optional heuristic tree)

**Migration:** Optional becomes a .dag-defined type with properties.
The compiler reads properties to know "this type has optional
cardinality" — it never checks `n.name == "Optional"`. Emit reads
the same properties to decide how to render (Option<T> in Rust,
Optional[T] in Python, *T in Go).

The `is_already_optional` heuristic tree (v2: 30 lines, v1: 70 lines)
dissolves completely — return_type carries the property, emit reads it.

### L1.2 — Containers (List, Map, Set)

**Current violations:** ~40 sites (container_node, node_is_map,
node_is_container, BuiltinList/BuiltinMap/BuiltinSet, Map name
matching, map_node, pair_node)

**Migration:** Container types become .dag-defined with properties
encoding element type, key type, iteration semantics. The compiler
walks children structurally. The Map consistency issue (bare leaf vs
parameterized) dissolves — all Maps are just Nodes with properties.

### L1.3 — Primitives (Int, String, Bool, Float, Unit, Bytes, Json, Secret)

**Current violations:** ~30 sites (BuiltinInt/String/Bool/Float/Unit,
name matching for copy semantics, iteration, method availability)

**Migration:** Primitive types become .dag-defined with properties
encoding copy semantics, method availability, literal forms. Emit
reads properties to decide rendering.

### L1.4 — Connective dissolution (Conj/Disj)

**Current violations:** 265 sites. This is the largest change —
`connective` is a field on Node itself.

**Migration:** Product/coproduct semantics move to .dag-defined
properties or structural conventions. The `connective` field is
removed from Node. Pattern matching, type equality, emission all
switch to property reads.

**This is last** because it touches every Node construction site.
L1.1–L1.3 can proceed while connective is still a field.

### L1.5 — Delete remaining type primitives

**Acceptance (all must hold):**
- `BuiltinTypeKind` enum deleted
- `builtin_type_kind()` function deleted
- All `node_is_*` predicates deleted or replaced with property reads
- `optional_node()`, `container_node()`, `pair_node()` deleted
- `connective` field removed from Node
- Zero type-name string matching in compiler
- Fixed point holds

---

## Phase 2: Gist End-to-End — IN PROGRESS

**Gate:** `gist.dag` + 11 transitive deps → Rust → `cargo build` → `cargo run --
dry-run` → correct output.

**Prerequisite:** Phase 1 diagnostic fixes (Goal A) complete — the
remaining 25 diagnostics include inference gaps that will cause gist
compilation to emit wrong types.

**Status (2026-03-22):**
- P2.2: Done — emit_rust.dag has real transport call emission (reqwest,
  Command, auth injection, dry-run mocking).
- P2.3: Done — main.rs generation with workflow subcommands, clap args
  with defaults, function dispatch match arms.
- P2.1, P2.4, P2.5: Blocked on stage0 binary.

**Blocker:** The v1 interpreter cannot handle multi-module .dag files
through `compile_sources` (lambda scoping issue: "unbound variable: t").
This means gist E2E verification requires building and running the
stage0 binary (~2 min build). The interpreter limitation does not need
a fix — it will become irrelevant when v1 is retired (Phase 3).

**Unblocking path:** Build stage0 via `cargo test -p gunbc-dag-tests
v2_bootstrap_fixed_point -- --ignored` (produces the binary as a side
effect), then use that binary to compile gist. The ~2 min build time
is acceptable for CI; the interpreter path is only for fast local
iteration on individual files.

| ID | Item | Status |
|----|------|--------|
| P2.1 | Gist pipeline test | Partial — interpreter blocked, tests scaffolded |
| P2.2 | Service operation bodies | Done (pre-existing) |
| P2.3 | Main.rs workflow dispatch | Done (pre-existing) |
| P2.4 | Multi-module extdep imports | Needs stage0 verification |
| P2.5 | End-to-end build+run test | Needs stage0 binary |

**Compile bundle (what the compiler produces):**
```
output_dir/
├── Cargo.toml          (generated, with reqwest/tokio/clap deps)
├── src/
│   ├── main.rs         (CLI scaffold with workflow subcommands)
│   ├── lib.rs          (mod declarations for all modules)
│   ├── v2_rt.rs        (runtime functions: concat, char_at, etc.)
│   ├── gist.rs         (workflow functions)
│   ├── github_api.rs   (service structs + operation methods)
│   ├── git.rs          (git operations)
│   └── ...             (one .rs per input .dag module)
```
This is already what `06_pipeline.dag` produces via `CompileResult.files`.
The bundle is a self-contained Cargo crate that `cargo build` can compile
without external scaffolding (unlike today, where `v2_crate_emit.rs` in
v1 does post-processing).

**Files:** `05_emit_rust.dag`, `06_pipeline.dag`, `03_resolve.dag`,
`dsl/extdeps/languages/rust/runtime.dag`, `src/v2/tests/src/lib.rs`

---

## Phase 3: v1 Retirement + Pipeline Completion

**Gate:** v2 compiles everything v1 can. v1 is no longer needed for any
compilation path. S76–S81 bootstrap scaffolding is dead code.

**Prerequisite:** Phase 2 complete (gist builds and runs).

| ID | Item | What |
|----|------|------|
| P3.1 | Verify parity | Enumerate all .dag files v1 compiles. Verify v2 produces equivalent output. Port any remaining v1-only paths. |
| P3.2 | Runtime shim dissolution | 21 functions in `v2_runtime_shim.rs` → template strings in `dsl/extdeps/languages/rust/runtime.dag`. Update `emit_v2_rt_module()` to read from runtime.dag. Functions: concat, char_at, string_length, substring, string_contains, lookup, index_by, to_string, empty_map, map_insert, map_merge, list_concat, str_eq, scan_while, skip_horizontal_ws, scan_to_eol, scan_string_end, code_point, from_code_point, filesystem_read, Concat trait. |
| P3.3 | Scaffolding verification | S76–S81 are only called by `assemble_v2_crate()`. Once v2 self-compile and gist work without v1, mark `#[deprecated]`. |
| P3.4 | Wire ownership + complexity | `07_ownership.dag` (A-) and `07_complexity.dag` (B+) exist but `06_pipeline.dag` doesn't call them. Wire both into the compile pipeline so every compilation proves ownership safety and complexity bounds. The proof layers already exist — this is plumbing, not new analysis. Cross-ref: R8, R9, M2. |
| P3.5 | Archive v1 | Move `src/v1/` → `archive/v1/`. Update Cargo workspace. Update CLAUDE.md. |

---

## Phase 4: Generic Emitter + Language Extdeps

**Gate:** Adding a new target language = writing a language extdep.
Zero compiler changes required.

**Prerequisite:** Phase 3 complete (v1 retired).

| ID | Item | What |
|----|------|------|
| P4.1 | Import aliasing | Blocker: `05_emit.dag:578-594` duplicates language data inline because all three extdeps define same-named declarations and imports lack `as` aliasing. Add `import { name as alias }` to tokenizer, parser, resolver. |
| P4.2 | LanguageSpec wiring | `LanguageSpec` exists in `dsl/std/languages.dag` (1367 lines, comprehensive) but no emitter reads it. `reserved_words` and `type_map` are triple-duplicated: in `std.languages`, in `extdeps/languages/{lang}/emit.dag`, and inline in `05_emit_python.dag`/`05_emit_go.dag`. Add `load_language_spec(target) -> LanguageSpec`. Pass through emit functions. Delete duplicate declarations. Cross-ref: R6, M6. |
| P4.3 | Extract generic emit core | ~70% duplication across 3 emitter files (rust: 3634, python: 1202, go: 1226 lines). All three have identical 22-arm ExprData dispatchers. Extract shared skeleton parameterized by LanguageSpec. Per-language files shrink to irreducible transforms (Rust: ownership/clone/borrow; Python: exceptions/comprehensions; Go: multi-return/interfaces). Cross-ref: R6, S5, S6, M6. |
| P4.4 | `--target` CLI flag | `compile_sources` already takes `target: RenderTarget`. Wire through bootstrap main.rs Compile subcommand. |
| P4.5 | Validate equivalence | Self-compile + gist → same output with generic emitter. Fixed point holds. |

**Architecture:**
```
compiler core (language-agnostic)
    ↓ reads
LanguageSpec interface (.dag contract)
    ↓ filled by
language extdep (dsl/extdeps/languages/rust/)
    ↓ rendered by
thin semantic renderer (irreducible differences only)
    ↓ produces
target source files
```

---

## Phase 5: Convergence (L2 preparation)

**Gate:** One type (Node) flows through the entire pipeline. `04_infer.dag`
(renamed from `04_reconcile.dag`). Each dissolution step validated by
re-bootstrapping and proving stage1 == stage2.

**Prerequisite:** Phase 4 complete (generic emitter). L1 complete (no
type primitives).

Note: L1 (type dissolution) already delivers the largest part of
convergence. Phase 5 dissolves the remaining compiler-internal types
that are NOT type-world knowledge but pipeline-internal structure.

| ID | Item | What |
|----|------|------|
| P5.1 | Rename | `04_reconcile.dag` → `04_infer.dag`. Update all imports and test references. Re-bootstrap → fixed point. Cross-ref: M1. |
| P5.2 | Token dissolution | Token (`:30`) + TokenKind (77 variants, `:35-78`) → Node compositions. Largest dissolution. 4-step: add Node constructors → dual-write → migrate parser → delete types. |
| P5.3 | Module dissolution | Module (`:92`), Import (`:103`), ImportNames (`:99`) → Node Conj compositions. Update parser (produces) and resolver (consumes). |
| P5.4 | Diagnostic dissolution | Diagnostic (`:346`), Severity (`:353`), CompileResult (`:336`), TextFile (`:341`) → Node compositions. Update all producers/consumers. |
| P5.5 | Service types | ServiceConfig (`:293`), OperationDef (`:302`), CapabilityDef (`:316`) — may already dissolve during parsing. Verify; convert if not. |
| P5.6 | Semantic types | IntrinsicMethod (17 variants), BuiltinTypeKind (15 variants), VarBindingKind, FieldAccessStyle, etc. Closed enums stay as .dag type definitions producing Nodes. TransportKind already redundant. |

P5.2–P5.6 are independent; each must re-bootstrap and prove fixed point.

After convergence:
```
source → parse → resolve → infer → emit
           ↓        ↓        ↓       ↓
         Nodes    Nodes    Nodes   TextFiles
         (raw)  (imports  (types
                 linked)  filled)
```

---

## Ordering

```
Phase 1A: Diagnostic fixes ─────────────────────────────→ (0 diagnostics)
  enumerate, fold, map_insert, callable, tighten equality
                                                            │
Phase 1B: L1 dissolution (interleaved with 1A) ──────────│─→ (489→0 violations)
  L1.1 Optional ─┐                                        │
  L1.2 Containers ├→ L1.4 Connective → L1.5 Delete all   │
  L1.3 Primitives ┘                                       │
                                                            │
Phase 2: Gist e2e ←───────────────────────────────────────┘
  (requires 0 diagnostics; L1 can continue in parallel)
                    │
Phase 3: v1 retirement + pipeline completion ←────────────┘
  (requires gist working; L1 should be complete by now)
                    │
Phase 4: Generic emitter ←────────────────────────────────┘
  (requires v1 retired; reads LanguageSpec from .dag)
                    │
Phase 5: Convergence ←────────────────────────────────────┘
  (requires generic emitter; dissolves remaining types)

Business track (AG1-AG3): starts after Phase 2
  AG1 modeling can overlap with Phase 2 if no compilation needed
```

**Key insight:** Phase 1A (diagnostics) blocks Phase 2 (gist). Phase 1B
(L1 dissolution) does NOT block Phase 2 — L1 is the long-term direction
and can continue in parallel once diagnostics hit 0. Each L1 step must
preserve the fixed point.

**Interleaving strategy:** When fixing a diagnostic, do it property-first
when possible (e.g., make enumerate's return type discoverable via .dag
properties instead of hardcoding `Tuple<Int, T>`). This way each
diagnostic fix also reduces the L1 violation count.

**Cross-cutting work:** S3.5–S7, R1–R9, and M1–M8 are detailed
reference material above. They run alongside the phases:
- R5 (data tables) aligns with Phase 1A — same code, same fixes
- R6/M6 (shared emit) aligns with Phase 4
- R8/M2 (ownership wiring) aligns with Phase 3
- M1 (renames) aligns with Phase 5

---

## Backlog

Items not blocking active phases. Revisit when relevant.

### Language Features

| Item | What | Status |
|------|------|--------|
| General generic syntax | `type Foo<T> = ...` parameterized types | Deferred — special-cased Result/Option sufficient |
| Full linear type checking | Prove ownership flow statically | D-ownership landed, full proof deferred |
| Widen V5 | Non-takeable modified fields in functional record update | Deferred — conservative V5 covers hot paths |

### Compiler Improvements

| Item | What | Status |
|------|------|--------|
| Anonymous record target resolution | Ambiguous cases must fail closed | Deferred |
| Collection intrinsic semantics in shared IR | | Deferred |
| Generated self-hosting tests and stage contracts | | Deferred |
| TCO backend contract | No silent partial fallback | Deferred |
| SCC-aware return type resolution | Not yet blocking | Deferred |

### Invariant Violations

| Item | Violation | Status |
|------|-----------|--------|
| ~~Emission/complexity dual classification~~ | ~~`classify_method_cost` and `classify_reconciled_intrinsic_method` classify independently~~ | **Done** — `classify_method_cost` deleted. Single authority in reconcile; complexity reads `method_semantics` from reconcile output. |

### Root Cause B: Closed Sets as Strings

Mechanical enum conversions — no design ambiguity for B-1/2/3/5/6.
B-4 (method intrinsics) intersects with IntrinsicMethod enum in `05_emit.dag`.
See INVARIANTS.md for the full table.

### Root Cause C: Errors as Fabrications

37+ sites where errors propagate as valid-looking values. Design decision
(structural error variant) documented in INVARIANTS.md. ExprError handling
landed in complexity/ownership. Remaining: promote Warning→Error for semantic
errors, normalize error representation across all producer sites.

---

## Verification

| Gate | Command | When |
|------|---------|------|
| Unit tests | `cargo test --workspace --exclude gunbc-dag-tests` | After every change |
| Clippy | `cargo clippy --all-targets -- -D warnings` | After every change |
| 0 diagnostics | `cargo test -p gunbc-dag-tests v2_strict_compile_diagnostic_count -- --ignored` | End of Phase 1 |
| Fixed point | `cargo test -p gunbc-dag-tests v2_bootstrap_fixed_point -- --ignored` | After any .dag change |
| Gist pipeline | `cargo test -p gunbc-dag-tests v2_gist_full_pipeline -- --ignored` | End of Phase 2 |
| Gist e2e | `cargo test -p gunbc-dag-tests v2_gist_end_to_end -- --ignored` | End of Phase 2 |
