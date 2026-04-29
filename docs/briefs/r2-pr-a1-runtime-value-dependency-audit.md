# R2 PR-A.1 Runtime Value Coproduct Dependency Audit

**Status:** BLOCKED dependency audit for the PR-A.1 implementation slice. This is not a carrier declaration PR.

**Parent authority:** [`r2-pr-a-runtime-value-model.md`](r2-pr-a-runtime-value-model.md), [`r2-evaluator-manager.md`](r2-evaluator-manager.md), and [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) section 3.2.

## Decision Needed

PR-A.1 must declare the observable runtime `Value` coproduct exactly as locked by PB-Runtime:

```dag
type Value
  = LiteralValue(LiteralBits)
  | RecordValue(List<NamedField>)
  | VariantValue { tag: DeclarationId, payload: Value }
  | NodeRef(NodeId)
  | CardinalityValue(LoopBound)
```

Current main already declares `type Value {}` in [`src/v3/spec/v3_l1.dag`](../../src/v3/spec/v3_l1.dag) as the L1 `Behavior` marker for `Behavior::Value`. The compiler still treats declarations as a flat name-keyed namespace for bootstrap lookup and same-rank duplicates fail closed. A second top-level `type Value = ...` under `src/v3/std/` would collide with the marker instead of creating a distinct runtime carrier authority.

PR-A.1 is therefore blocked on a substrate-owned name/identity resolution. The preferred resolution is to rename or split the existing L1 behavior marker before the runtime carrier lands, for example:

```dag
type ValueBehavior {}
```

The exact name is Substrate/Director-owned. The important invariant is that the runtime coproduct keeps the unqualified name `Value` from PB-Runtime section 3.2, and the L1 behavior marker stops occupying that flat name.

## Why Not `RuntimeValue`

`RuntimeValue` would avoid the compiler collision locally, but it would create a parallel value authority:

- PB-Runtime section 3.2 names the runtime result domain `Value`.
- PR-A.0 locks R2-Evaluator to that same structural model.
- PR-A.1 was dispatched to land that observable `Value` surface, not a namespaced substitute.

Using `RuntimeValue` would require amending the PB-Runtime and PR-A.0 design locks first. This audit does not make that amendment.

## Current Collision Surface

Known candidate consumers that must be updated or verified when the L1 marker is renamed/split:

- [`src/v3/spec/v3_l1.dag`](../../src/v3/spec/v3_l1.dag) declares `type Value {}` as the L1 marker.
- [`src/v3/compiler/src/dag.rs`](../../src/v3/compiler/src/dag.rs) caches `substrate_markers.value` through `declaration_by_name("Value")` and exposes `Dag::value_marker()`.
- [`src/v3/compiler/src/lower.rs`](../../src/v3/compiler/src/lower.rs) validates `BehaviorRealization.target` against name-keyed marker lookups including `declaration_by_name("Value")`.
- [`src/v3/compiler/tests/integration/m1_substrate_test.rs`](../../src/v3/compiler/tests/integration/m1_substrate_test.rs) expects `dag.value_marker()` in the bootstrap marker inventory.
- Docs that name the five L1 behavior markers as `Value / Transform / Branch / Loop / Bind` should be checked when the marker is renamed. The `Behavior` substrate variant name can remain `Value`; the collision is the marker declaration name, not the `Behavior` variant label.

This is the narrow observed surface from the PR-A.1 preflight. A substrate-owned marker rename PR should still run the full test suite and search for any additional string-keyed marker references.

## PR-A.1 Resume Gate

PR-A.1 can proceed once one of these is true:

1. The L1 behavior marker no longer owns the flat declaration name `Value`, and the marker cache/realization validation/tests point at the new marker name.
2. The compiler has a substrate-owned module-qualified declaration identity model that allows PB-Runtime `Value` and L1 marker `Value` to coexist without flat-name ambiguity.
3. PB-Runtime and PR-A.0 are explicitly amended to use a different observable carrier name.

Option 1 is the smallest dependency for the current slice. Option 2 is likely larger than PR-A.1. Option 3 is not recommended because it weakens the PB-Runtime/R2-Evaluator convergence lock.

Until one of these gates lands, PR-A.1 must not add `RuntimeValue`, `ClosureValue`, evaluator state carriers, or body evaluator logic.
