# URGENT: DSL Migration Checklist

**Status**: Active
**Date**: 2026-02-17
**Last reconciled**: 2026-02-18
**Priority**: High
**DSL Alignment**: Primary DSL migration backlog
**Track**: B — Migration Targets

Hand-rolled patterns that should migrate to daglang as the compiler matures.

## Prerequisite: Type-Dispatch Boilerplate Elimination

> **See**: `TODO/TODO_URGENT_type_dispatch_boilerplate.md` for full audit and implementation plan.

The "Ready Now" migrations are blocked by **~1,350 lines of type-dispatch boilerplate**
(15 union enums, 16 `From` impls, 10 converter functions) that exist solely to satisfy
`Dag<T: Executable + Clone + Send + 'static>`. Deleting manual `graph.rs` files requires
replacing them with DSL-compiled `Dag<DynOp>`, which first requires introducing `DynOp` —
a type-erased `Arc<dyn Executable + Send + Sync>` wrapper in `core/exec`.

Without `DynOp`, each migrated module would still need its own `GraphOp` union enum and
converter functions, preserving the boilerplate the DSL was meant to eliminate.

**Fix**: `DynOp` + central resolver (~300 lines added) → delete ~5,650 lines of boilerplate.

## Ready Now (DSL has the primitives)

- [ ] **Pragma graphs** (`gunbc-dag/src/pragma/graph.rs`) — `dsl/tools/pragma.dag` is authored with 3 parallel `content_upsert` chains; runtime cutover from the hand-built Rust builder is still pending.
- [x] **Transport triplets** (all binaries) — audited/verified via daglang triplet derivation tests across workspace DSL tool modules; lowering preserves prepare→execute request wiring and execute→parse response wiring where parse stages exist.
- [ ] **Codegen graph** (`gunbc-dag/src/codegen/graph.rs`) — `dsl/tools/codegen.dag` is authored with staged conditional pipeline (exists check → conditional codegen → stamp); runtime cutover from the hand-built Rust builder is still pending.
- [x] **Conditional execution / skip semantics** — covered by existing
      `content_upsert` lowering (compare output `skip` is wired to execute transport `skip`); no new `[skip_if]` syntax required for this pattern.

## Needs DSL Work First

- [ ] **Display orchestration** (`core/exec/src/display.rs`) — channel-driven event
      loop with timer ticks. Needs reactive/streaming DSL primitives (`observe events`,
      `every 80ms`). Rendering IR exists (`Frame`, `FrameRenderer`, `OutputMedium`)
      but no DSL construct generates event loops yet.
- [ ] **Testgen dynamic targets** (`gunbc-dag/src/testgen_dag/graph.rs`) — N
      upsert chains, one per `DagSpecDef` discovered via inventory. Needs
      compile-time metaprogramming or inventory integration in DSL.
- [ ] **Makegen tool registry** — procedural target generation from `#[tool_target]`
      inventory. Same metaprogramming gap as testgen.
- [ ] **Loop extra inputs** — `for` loops where body needs non-element context
      (e.g., `repo_path`). DSL `for` lowering doesn't model passthrough inputs yet.

## DSL Maturity Snapshot (2026-02-17)

| Layer | Status |
|-------|--------|
| Syntax (types, fn, func, pattern, service, resource, interface, pipeline) | Stable |
| Lowering to GraphIR | Solid — patterns expand, services → triplets, resources → acq nodes |
| Type system (records, sums, interfaces, provider resolution) | Working |
| Pragmatic use (real tools using .dag files) | In progress — workspace tool composition discovers `dsl/tools/*.dag`; legacy Rust DAG implementations still exist for execution paths/parity |

The "Ready Now" items are the highest-ROI migration targets — pragma especially,
since it's 3 identical upsert chains that map directly to a `pattern` invocation.
