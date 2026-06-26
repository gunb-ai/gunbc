# emit-host batch-isolation — keep rustc children off the wide discovery batch

Status: scoped, owner calm-carp-204, 2026-06-26. The rustc-children half of the
floor OOM (the other half is Layer A resolve de-merge, tidy-wren-707).

## Root cause (read from the live plan)

The floor `ci` job runs `claim_executor` over `gunbc_ci_spec.gates`
(`= gunbc_ci_floor_gates`). The batches are derived by
`parallel_executor_plan_from_dependencies` over `floor_dependencies`
(`src/v2/workflow/ci_floor_plan.dag`). A gate gets a *serialize-against-the-wide-
corpus* edge (`floor_corpus_serializes`, a `ResourceDependsOn`) **only if**
`gate_is_heavy_resolve(g) == true`. Today:

```
gate_is_heavy_resolve: SourceRootIngestGate => true, DslCompileCleanGate => true,
                       EmitHostGate => false, ...
```

So **`EmitHostGate` gets no serialize edge** and lands in the *same wide batch*
as the `RunnableDiscoveryBatch` corpus shards (~2 GiB each at `spawn_width`). But
`emit_host_gate_passes` spawns **child cargo/rustc processes** (~1.5 GiB), which
the 8 GiB cgroup counts. Concurrent peak = `width × resolve_shard + rustc_children`
> 8 GiB → exit 137. (`RustMonolithGate`, the other rustc gate, is **not** in the
floor — it's `gunbc_ci_rust_job_gates`, the separate `rust_tests` job — so the
floor's only host-compiler gate is `EmitHostGate`.)

`gate_is_heavy_resolve` is honestly named: it means *heavy whole-tree resolve*, a
**different cost axis** than *spawns a host compiler*. Overloading it would be a
§3 nickname. The two are distinct grounded contributors to one concept: **a gate
whose concurrent memory footprint is too large to overlap the wide corpus.**

## Fix (model-first, §2 decompose — not a third floating boolean)

One concept, two grounded contributors:

```
fn gate_spawns_host_compiler(g: Gate) -> Bool {   // grounded: emit_host_gate_passes execs rustc
  match g { EmitHostGate => true, _ => false }
}
fn gate_serializes_against_corpus(g: Gate) -> Bool {
  gate_is_heavy_resolve(g) || gate_spawns_host_compiler(g)
}
```

`floor_dependencies_with` uses `gate_serializes_against_corpus` in place of the
bare `gate_is_heavy_resolve`. Net: `EmitHostGate` gets a `floor_corpus_serializes`
edge → it lands in a batch with **no concurrent discovery shard**, so its rustc
peak no longer overlaps the wide corpus. Discovery corpus keeps running wide
(each shard ~2 GiB, shrinking further under Layer A).

## Witness (red-on-regression, by execution)

- **Positive:** the derived floor schedule places `EmitHostGate`'s runnable in a
  batch whose other members are not the `RunnableDiscoveryBatch` corpus (no
  host-compiler/wide-corpus co-residence).
- **Perturbation (must go RED):** with `gate_serializes_against_corpus` reverted
  to bare `gate_is_heavy_resolve`, `EmitHostGate` co-resides with the corpus batch.

## Relationship to the resource-aware scheduler (no fork)

This is a **pre-Node-B stepping stone**, explicitly marked to dissolve. The
durable model is the resource-aware scheduler (`docs/plans/resource-aware-scheduler.md`,
gentle-newt-542): per-`Runnable` `cost: CostEstimate { space: ByteSize }` (Node B),
with the scheduler packing batches so concurrent space ≤ budget (Node C). Once
that lands, `gate_serializes_against_corpus` dissolves into `cost.space > threshold`
— the boolean predicate is the honest interim until real per-Runnable ByteSize
costs exist. Carries a `Disposition: Scaffold` marker with that dissolution trigger.
Coordinated with gentle-newt-542 so `Runnable.cost` is not modeled twice.
