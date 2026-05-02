# T-CostLens-Composition substrate-prep audit

**Lane:** R3 T-CostLens-Composition. **Authority:** PR #1263 cost-lens blocker history, cost-lens implications in `docs/design-numeric-construction.md`, and the current symbolic-cost substrate in `src/v3/lenses/cost.dag`.

This audit names the bottom-up substrate fact needed before cost-lens composition can honestly consume loop bounds after the numeric construction chain starts landing. It is docs-only: no `data cost_lens: Lens<SymbolicCost>`, no hand-Rust fold scaffold, no parser/literal/refinement work, and no numeric-consumer migration.

## Finding

The live symbolic-cost path is still honest because it receives the whole `LoopNode`:

- `src/v3/lenses/cost.dag` computes loop cost as `iterate(linear_at(l.source), body_cost(...))`.
- `LoopNode.source` is populated for both single-recursion `Cardinality` loops and mutual-recursion `Descent` loops.
- `LoopBound::Descent { cluster }` carries termination topology through `Dag.clusters`; it does not carry the runtime measure used for symbolic cost.

That is acceptable for the current DB-3 / DB-7 `AnalysisDimension<SymbolicCost>` path because `witness_of(d, behavior)` can inspect `Behavior::Loop(l)`. It is not acceptable as the future R3 `Lens<SymbolicCost>` composition boundary if that boundary remains `iterate: fn(C, LoopBound) -> C`: a source-less iterate function cannot recover the runtime measure from `LoopBound::Descent { cluster }` without rereading `LoopNode.source` or rewalking cluster topology.

## Required substrate fact

The next substrate precursor should make the loop-iteration measure a `LoopBound` fact:

```dag
type LoopBound
  = Cardinality { count: PortId }
  | Descent { cluster: ClusterId, measure: PortId }

fn loop_bound_measure(bound: LoopBound) -> PortId =
  match bound {
    Cardinality(c) => c.count
    Descent(d) => d.measure
  }
```

`measure` is the runtime port whose size bounds the iteration count. For current mutual-recursion lowering, that is the per-entry descent parameter already stored in `PendingMutualLoop.source`; today it is copied into `LoopNode.source`. The future shape moves that fact into `LoopBound::Descent` so `LoopBound` owns the cost projection and the cluster sidecar remains only the termination-topology authority.

## Why not implement it in this PR

This is not a tiny isolated carrier edit:

- `LoopBound` is reflected in `src/v3/std/substrate.dag`, Rust `dag.rs`, lowering, generated bootstrap snapshots, lens-generated Rust, emit dependency walks, and tests.
- Adding `Descent.measure` while leaving `LoopNode.source` as an independent authority would create a temporary duplicate measure fact unless the PR also ratchets equality and names the retirement path.
- Removing `LoopNode.source` in the same slice is broader: current emit, dimension, generated lens, and test-runner paths read it directly.

So the honest next PR is a focused substrate precursor, not this audit PR:

1. Add `measure: PortId` to `LoopBound::Descent`.
2. Add/ratchet `loop_bound_measure(bound) -> PortId`.
3. Populate `Descent.measure` from the same per-entry descent parameter currently used as `LoopNode.source`.
4. Add a ratchet that `LoopNode.source == Descent.measure` while `LoopNode.source` still exists for mutual-recursion descent loops. Do not assert that `Cardinality.count` equals `LoopNode.source`; cardinality loops may carry an explicit count port distinct from the iterated source.
5. Update the `LoopBound` green dissolution receipt in `src/v3/std/substrate.dag` and the mutual-recursion lowering design receipt in `docs/design-mutual-recursion-lowering.md` so they name the new measure facet and keep cluster topology distinct from cost measure.
6. In a later migration, retire direct `LoopNode.source` reads or make `source` derived from `LoopBound`.

That sequence preserves single authority while giving the cost lens a source-free projection point.

## 6Q audit

### Q1 - Cardinality invariants

**PASS.** The proposed fact adds one `PortId`, not a list. No empty/non-empty invariant is introduced.

### Q2 - Index/handle types

**PASS with existing caveat.** `PortId` is already the substrate handle used for runtime value ports. The new `measure` field must be populated by lowering from an existing valid parameter port; it should not be constructed from a raw integer outside the DAG builder.

### Q3 - Duplicated fact

**BLOCKER unless sequenced.** Adding `Descent.measure` duplicates `LoopNode.source` unless the same PR adds an equality ratchet and names the `LoopNode.source` retirement/derivation path. This is why the fact is not authored here.

### Q4 - Coproduct compression

**PASS.** `Cardinality` and `Descent` remain distinct loop-bound causes. The accessor projects only their shared cost-measure facet; it does not collapse termination topology into cardinality or erase the cluster witness.

### Q5 - Construction authority

**PASS if lowering owns construction.** Lowering already chooses the descent parameter while building `PendingMutualLoop`. That same lowering site should populate `Descent.measure`; consumers should read it through `loop_bound_measure`.

### Q6 - Representation duality

**BLOCKER unless direct `LoopNode.source` cost reads are retired.** The target state has one descent cost-measure representation: `loop_bound_measure(LoopBound)` for `Descent` reads, with `Cardinality.count` remaining the explicit cardinality bound. During migration, `LoopNode.source` is a compatibility field for descent loops with a ratcheted equality invariant, not a second authority.

## Decision

**STOP+PING for implementation; proceed only with this docs receipt.**

The required next substrate fact is crisp: `LoopBound::Descent` needs `measure: PortId`, and the shared accessor should be `loop_bound_measure(bound: LoopBound) -> PortId`. But implementing it safely is wider than a tiny precursor because `LoopNode.source` is still a live authority across emitted Rust and generated lens surfaces. The next worker should author the narrow substrate precursor with generated snapshots and descent equality ratchets, then a follow-up should retire direct `LoopNode.source` reads from cost composition.

## Non-goals

- Do not author `data cost_lens: Lens<SymbolicCost>`.
- Do not author a hand-Rust fold scaffold.
- Do not use `UnknownCost("mutual recursion descent cluster")` as a failure channel.
- Do not infer Descent cost by rescanning `Dag.clusters` or SCC topology.
- Do not edit numeric parser/literal/refinement syntax or migrate numeric consumers.
