# By-Construction Reachability — Emit Only Reachable Code

**Status**: Draft  
**Date**: 2026-02-25  
**Depends on**: FC-14 (dead-path pruning emit variant)  
**Owned by**: FC-15

## Problem

FC-14 adds a pruning pass (`compute_reachable_node_ids`) that filters unreachable symbols before emission. This works, but is a **cleanup pass** — it fixes symptoms post-hoc rather than preventing them. The invariant "emit only reachable code" is enforced by runtime filtering, not by construction.

If a new emitter forgets to call the reachability filter, unreachable symbols silently appear in generated output. The invariant is opt-in, not structural.

## Goal

Make "emit only reachable code" a **compiler invariant** that is impossible to violate — by construction, not by cleanup.

**End state**: The IR/backend contracts structurally guarantee that only reachable symbols are emittable. An emitter that tries to iterate over unreachable nodes cannot access them.

## Invariant Specification

**INV-REACH**: For any emission target, the emitter receives only the reachable subgraph. Unreachable nodes are not present in the data structure the emitter iterates.

### Formal statement

```
∀ emitter E, ∀ dag D, ∀ node N ∈ emitted(E, D):
    N ∈ reachable(entrypoints(D), edges(D))
```

### Enforcement mechanism

The `Dag<T>` passed to emitters is pre-sliced to the reachable subgraph. The emitter never sees unreachable nodes.

## Failure Modes

1. **Entrypoint misclassification**: If entrypoint detection is wrong (e.g., resource nodes incorrectly classified as entrypoints), reachability is too broad. Mitigated by: entrypoint detection is well-tested and structural (no incoming edges = entrypoint).

2. **Edge omission**: If the lowerer fails to emit an edge for a real dependency, downstream nodes become unreachable and their code is silently dropped. Mitigated by: lowerer edge tests, dry-run execution catches missing wiring.

3. **Dynamic subgraphs**: Nodes generated at runtime (e.g., `SubDag` expansion) can't be statically analyzed. Mitigated by: SubDag bodies are recursively analyzed at their own level.

## Proof Obligations

1. **Completeness**: Every node that executes in a dry-run is present in the reachable set. (Test: compare dry-run execution log node IDs against reachable set.)

2. **Soundness**: No unreachable node appears in emitted output. (Test: emit a DAG with known unreachable nodes, verify they are absent from output.)

3. **Stability**: Reachability is deterministic — same DAG produces same reachable set. (Test: compute reachability twice, compare.)

## Migration Plan

### Phase 1 (FC-14, done): Pruning pass
- `compute_reachable_node_ids()` filters at emit time
- All emitters call the filter before iterating nodes
- Tests verify pruning works

### Phase 2 (this doc): Structural enforcement
- Add `ReachableDag<T>` wrapper type that only contains reachable nodes
- Emitters accept `&ReachableDag<T>` instead of `&Dag<T>`
- `ReachableDag::from_dag(dag)` performs the slice once
- The original `Dag<T>` cannot be passed to emitters (type mismatch)

### Phase 3: Remove pruning pass
- Once all emitters use `ReachableDag`, the per-emitter `compute_reachable_node_ids()` calls are redundant
- Remove them; the invariant is now by construction

## Contract Tests

1. `reachable_dag_excludes_unreachable_nodes`: Build a DAG with an unreachable node, create `ReachableDag`, verify unreachable node is absent.

2. `reachable_dag_preserves_all_reachable_nodes`: Build a fully-connected DAG, create `ReachableDag`, verify all nodes present.

3. `reachable_dag_preserves_edges`: Verify edges between reachable nodes are preserved and edges to/from unreachable nodes are removed.

4. `emitter_type_safety`: Verify that emitter functions accept `&ReachableDag<T>` and reject `&Dag<T>` at compile time.

5. `dry_run_parity`: Execute a DAG in dry-run, collect executed node IDs, verify they are a subset of `ReachableDag` node IDs.

## Design Decisions

### D1: Wrapper type vs. trait

Use a **wrapper type** (`ReachableDag<T>`) rather than a trait. The wrapper carries the sliced data directly, avoiding the overhead of virtual dispatch and making the API surface clear.

### D2: One-time computation

Reachability is computed once when `ReachableDag::from_dag()` is called. The result is cached in the wrapper. No per-emitter recomputation.

### D3: Edge preservation

Edges are filtered to only include those where both endpoints are in the reachable set. This ensures the reachable subgraph is a valid DAG.

## Vertical Slice: Rust Target

The first vertical slice enforces the invariant for the Rust target:

1. Add `ReachableDag<T>` to `core/ir`
2. Change `emit_rust_bundle()` to accept `&ReachableDag<LoweredOp>`
3. Remove the per-function `compute_reachable_node_ids()` call from `emit_rust_bundle()`
4. Add the contract tests listed above
5. Gate behind `--emit-reachable-only` flag (or always-on if tests pass)

Other targets (Go/C/MIPS) migrate in follow-up PRs.
