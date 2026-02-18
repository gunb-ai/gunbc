# URGENT: DSL Migration Checklist

**Status**: Active
**Date**: 2026-02-17
**Last reconciled**: 2026-02-18
**Priority**: High
**DSL Alignment**: Primary DSL migration backlog
**Track**: B — Migration Targets

Hand-rolled patterns that should migrate to daglang as the compiler matures.

## Ready Now (DSL has the primitives)

- [ ] **Pragma graphs** (`gunbc-dag/src/pragma/graph.rs`) — 3 parallel content
      upsert chains. Express as `pattern` invocations with service calls.
- [ ] **Transport triplets** (all binaries) — prepare/execute/parse 3-node pattern.
      DSL already supports via service call lowering.
- [ ] **Codegen graph** (`gunbc-dag/src/codegen/graph.rs`) — staged pipeline:
      exists check → conditional codegen → stamp. DSL `if` in `func` bodies.
- [ ] **Conditional execution / skip semantics** — content upsert "compare" step
      skips write when content matches. Needs `[skip_if]` or equivalent DSL syntax.

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
