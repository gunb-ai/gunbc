# Compiler Densification Roadmap

Status: Active
Created: 2026-03-04
Source: External review feedback + internal lane analysis

## 0. Framing

The project's north star is **"dense compiler, thin runtime"**: anything that changes
behavior must desugar into explicit nodes/edges/types *before* validation/execution.
The runtime executor should be dumb, predictable, and hard to misuse.

This document maps the feedback's priorities to existing task IDs and identifies gaps
where no task or design doc existed. It is organized by **what runtime bug class each
item eliminates** and **what it unlocks next**.

## 1. Current State (2026-03-04)

### Already done well

| Strength | Evidence |
|----------|----------|
| Visible I/O invariant | `TransportOps::Execute` is the only runtime I/O site. `clippy.toml` enforces. |
| Proof obligations → generated tests | Testgen encodes structural invalids as failing tests (panic on structural error). |
| Explicit resources/environment | `Port::resource()`, `ResourceInput`, `validate_resource_wiring_recursive()`. |
| Unified compilation entry point | `build_dsl_graph(module, resolver, BuildOpts)` in `core/resolve/src/builder.rs`. |
| Sibling fn threading | `collect_sibling_fn_bodies()` passes fn bodies through resolution. |
| Cardinality consolidation | Dual encoding substantially resolved in IR layer (assertion rejects container aliases). |
| Phase gates in process | "Prove compilation before infra" hard gate on SDLC Phase 0. |

### Remaining runtime leak-through

| Leak | Where | Bug class it causes |
|------|-------|---------------------|
| ExprComputeOp interpreter | `core/resolve/src/resolve.rs:756` | Hidden semantics, `__` convention, blocks optimizations |
| FnBodyCallableOp evaluator | `core/resolve/src/resolve.rs:173-258` | Match arms don't suppress unreachable transport (postmortem Bug 1) |
| CollectionDelegate | `core/resolve/src/resolve.rs:456-472` | Evaluator delegation at runtime |
| Hermeticity erasure | `ShellRequest` has no `semantics` field | Classification drift, test scope errors |
| ~22 hand-written DynOp adapters | `core/resolve/src/resolve.rs` | Scaling ceiling, per-service boilerplate |
| `evaluate_fn_body` at runtime | Called from 4 runtime locations | Interpreter hides semantics from graph |

## 2. Priority Map

Ordered by: **what downstream/runtime bug class it eliminates**.

### Priority 1: Kill the interpreter (Bridges 1+2+3)

**Task IDs**: Bridge 1, Bridge 2, Bridge 3 from `TODO/gunbc-app-simplification.md`;
C24 from `docs/design/pure-dataflow-lowering.md`

**Bug class eliminated**: Hidden semantics in ExprComputeOp. Match arms not guarding
transport nodes. `__` convention. Runtime evaluator doing "secret language stuff" the
graph can't see.

**Concrete fix**: The `make gist` postmortem (two compounding bugs from dual execution
model) is the case study. When fn bodies are lowered to SubDags, match arms become
guarded branches. Transport nodes inside non-matching arms produce `Value::Skipped`.

**Acceptance criteria** (from simplification doc):
- `DeclaredOutputCallableOp` struct **deleted**
- `FnBodyCallableOp` struct **deleted**
- `CollectionDelegate` struct **deleted**
- `evaluate_fn_body()` exists only in lowerer for compile-time evaluation
- Zero `ExprComputeOp` in any compiled graph

**Execution order** (matches `pure-dataflow-lowering.md` phasing):
1. Phase 1: Add 7 structural `PrimitiveOpKind` variants (M) — infrastructure only
2. Phase 2: Replace `synthesize_expr_compute` with `decompose_return_expr` (L) — incremental by category (P1→P4 tiers from census)
3. Phase 3: Delete legacy (S) — remove ExprCompute, remap_expr_idents, evaluate_fn_body
4. Bridge 1: Lowerer produces SubDag directly for fn/func/pattern items (M)
5. Bridge 2: All fn bodies lowered to SubDag at compile time (M)
6. Bridge 3: Collection ops as IR nodes, not evaluator delegation (M)

**Unlocks**: C25 (service-driven codegen), RF-E5/RF-E6 test restoration, `make gist` fix.

**Design doc**: `docs/design/pure-dataflow-lowering.md` (exists, comprehensive).

### Priority 2: Default parameter correctness (Bridge 2b)

**Task ID**: Bridge 2b from `TODO/gunbc-app-simplification.md`

**Status**: **DONE** (lowerer injects literal source nodes for omitted call args with
defaults; `daglang-lower/src/lib.rs:9164-9189`).

No further action needed. This was an "immediate Now" item and it's already landed.

### Priority 3: Hermeticity as structural, non-erasable property

**Task ID**: No existing task. Design doc exists: `docs/design/shell-hermeticity-annotation.md`

**Bug class eliminated**: Hermetic vs external shell commands are indistinguishable after
conversion to `TransportRequest::Shell`. Forces manual mappings. Weakens compile-time
classification. Test categorization unreliable.

**Acceptance criteria**:
- `ShellProducerSemantics` + `Hermeticity` types exist in `core/ir/src/transport/mod.rs`
- `ShellRequest.semantics` field populated by all known producers
- Testgen categorization uses `semantics.hermeticity` when present
- Strict mode available: reject `semantics=None` for workflows requiring hermetic classification

**Design doc**: `docs/design/shell-hermeticity-annotation.md` (exists, migration plan included).

**New task needed**: Create task in Lane 2 for implementation (see Actions below).

### Priority 4: Collapse remaining dual sources of truth

**Task IDs**: WS3-3 (`normalize_type_id` deletion), WS3-4 (optionality as DAG layer),
cardinality items from `TODO/type-system.md`

**Bug class eliminated**: Port cardinality vs type encoding drift. String-based references.
`type_optional: bool` legacy field.

**Current state**: Cardinality dual encoding is substantially resolved in IR core
(assertion rejects container aliases in ports). The remaining issue is `type_optional: bool`
on Port, which coexists with `Cardinality::ZERO_OR_ONE`. WS3-4 is the task to unify these.

**Acceptance criteria** (from type-system.md WS3-4):
- `Port::is_optional()` no longer calls `ends_with('?')`
- `grep "ends_with.*?" core/ir/src/` returns 0 for optionality checks
- Cardinality mismatch is a type error

**Design doc**: `docs/design/v4/compositional-type-coverage.md` (exists, WS-3 section).

### Priority 5: Service-driven codegen (C25)

**Task ID**: C25 from `docs/design/pure-dataflow-lowering.md` Phase 4

**Bug class eliminated**: Scaling ceiling of per-service DynOp adapters. ~22 hand-written
adapter structs in `resolve.rs`. Adding a new service requires Rust code.

**Acceptance criteria**:
- 3 generic protocol executables (REST, Shell, File)
- Zero per-service `Executable` adapter structs in `resolve.rs`
- Adding a new service requires only a `.dag` file

**Depends on**: Priority 1 (C24 pure dataflow lowering).

**Design doc**: `docs/design/service-codegen.md` (exists) + C25 section in
`docs/design/pure-dataflow-lowering.md`.

### Priority 6: Resource injection at lower time (Bridges 8+9)

**Task IDs**: Bridge 8, Bridge 9 from `TODO/gunbc-app-simplification.md`

**Bug class eliminated**: Filesystem resource injected at resolver time rather than
lowerer time. File transport goes through generic adapters instead of typed IR nodes.

**Acceptance criteria**:
- `add_fs_env_root_node()` deleted from `fs_env.rs`
- `GenericFilePrepareOp` and `GenericFileParseOp` deleted
- Resource acquisition nodes inserted by lowerer when `uses` declarations present
- File transport uses same prepare/execute/parse triplet as REST and Shell

**Design doc**: Covered in `TODO/gunbc-app-simplification.md` (Bridge 8, Bridge 9 sections).

## 3. What's NOT in existing task sheets (gaps identified by review)

### Gap A: No task for hermeticity implementation

The design doc (`shell-hermeticity-annotation.md`) exists with a migration plan but
there's no corresponding task in any lane. This needs to be added to Lane 2.

**Proposed task**: "Bridge 11: Shell hermeticity annotation"
- Add `ShellProducerSemantics` + `Hermeticity` to `core/ir/src/transport/mod.rs`
- Thread through `ShellRequest` builders
- Annotate known producers (git → Hermetic, gh gist → External, cargo → Hermetic)
- Add testgen categorization hook
- Effort: M
- Depends on: Nothing
- Parallel with: Bridges 1-3

### Gap B: No explicit "metadata erasure is semantics-preserving" invariant

The review identified this as a core design principle but it's not stated as an invariant
in `start-here.md`. The principle is implicit in Bridge 4 (OutputPathMetadataOp deleted)
and Bridge 10 (PipelineDispatchOp metadata-only) but should be explicit.

**Proposed addition to `start-here.md`**: Invariant I13 — "Deleting non-semantic metadata
from a compiled graph must not change observable behavior."

### Gap C: No task for `ReturnExprCompute` split-brain (from gap-analysis-tasks.md)

The `docs/review/gap-analysis-tasks.md` documents this as P0-5 / P1-1, which maps exactly
to C24 Phase 2 from the pure-dataflow-lowering design. However, the `TODO/gunbc-app-simplification.md`
references it as Bridge 1+2 without the P0-5 "make install fails" framing.

**Action**: The simplification doc already has the right acceptance criteria. The gap-analysis
P0-5 is the same work. No new task needed — just cross-reference.

### Gap D: Compile-time tool registry (Bridges 6+7)

The review correctly identifies this as a "larger design work" item. It's already tracked
in `TODO/gunbc-app-simplification.md` with the right "keep as-is, don't optimize" guidance.
The blocker is compiler artifact emission. No new doc needed.

## 4. Execution Order

Integrating with the existing three-lane structure from `tasks.md`:

```
Phase 0: Quick wins (parallel, 1-2 days)
├── Lane 2: Bridge 11 (hermeticity) — new, parallel with everything
├── Lane 1: WS1-6 remainder (LanguageId, GcpRegion)
├── Lane 1: WS1-7 (stub cleanup)
└── start-here.md I13 addition

Phase 1: Kill the interpreter (Lane 2, sequential, 2-3 weeks)
├── C24 Phase 1: Structural primitives (M)
├── C24 Phase 2: Expression decomposition P1+P2 tiers (L)
├── C24 Phase 2: Expression decomposition P3+P4 tiers (L)
├── C24 Phase 3: Delete legacy (S)
├── Bridge 1: SubDag direct lowering (M)
└── Bridge 2: FnBodyCallableOp deletion (M)

Phase 2: Widen the compiler (Lane 1+2, parallel, 2-3 weeks)
├── Lane 2: Bridge 3 (CollectionDelegate → IR nodes)
├── Lane 2: Bridge 8+9 (resource injection + file transport)
├── Lane 1: WS3-1 through WS3-4 (typechecker unification)
└── Lane 1: WS2-1 through WS2-4 (service type discipline)

Phase 3: Thin the runtime (Lane 2, 2-3 weeks)
├── C25: Service-driven codegen (3 generic protocol executables)
├── Lane 2: Rename crate, output dir consolidation
└── Lane 2: Bridges 6+7 (compile-time tool registry)

Phase 4: SDLC pipeline (Lane 3)
├── Already gated on Phase 0 compilation proof (DONE: S-1 through S-8)
├── Phase 2/3 local real run (in progress)
└── Phase 3/4 full pipeline + production
```

## 5. Invariant Checklist (for PRs touching the compiler)

Synthesized from the review's recommendations + existing repo invariants:

1. **Structural first**: If a feature changes behavior, it must desugar into DAG
   nodes/edges/types before validation. (I5, I10)
2. **No hidden I/O**: No `execute_transport()` outside `TransportOps::Execute`. (I3)
3. **Fail closed**: "We don't know" → obligation/test or compile error, not silent
   fallback. (I4)
4. **Single choke point**: All DSL compilation through `build_dsl_graph()`. (I8)
5. **Metadata erasure is safe**: Deleting non-semantic metadata must not change
   observable behavior. (proposed I13)
6. **Phase gates**: Prove compilation before building infrastructure. (Lane 3 lesson)
7. **No interpreter in runtime**: ExprComputeOp is the active elimination target.
   New code must not create new interpreter surfaces.

## 6. What this roadmap does NOT cover

- **SDLC pipeline feature work** (Lane 3 S-12 through S-19) — separate doc
- **External dependency modeling** (P5 from gap-analysis-tasks.md) — pure DSL authoring
- **Binary elimination** (P3 from gap-analysis-tasks.md) — blocked on C20
- **Backlog compiler features** (FC-CF2, FC-CF3, CX-1 through CX-5)
