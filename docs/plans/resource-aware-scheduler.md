# Resource-aware scheduler — larger allocations → larger throughput (always)

> Companion to [`realization-measurement-loop.md`](realization-measurement-loop.md) (Phase 0 space arm + Phase 1). That doc covers the full realization arc; this one tracks the four-node implementation the operator scoped 2026-06-25. Dispatched from session `calm-carp-204`. DESIGN refs: §1 (time = cost), §2 (one concept every scale — width IS a resource fit count), §3 (single authority — cost lives on `Runnable`, width derives from it), §5 (fail-closed — wrong width is silent over/under-use; construction makes over-commit unwritable).

---

## The invariant

`spawn_width = floor(available_memory / per_runnable_peak)` — derived, never hand-authored.
Host gets more memory → width goes up automatically → throughput goes up. Enforced by construction, not convention.

Today this is violated: `gunbc_ci_floor_per_shard_peak_rss_bytes()` reads a static data row (`gunbc_ci_floor_measured_peak`) derived by dividing a whole-run cgroup peak by a concurrency count. The division is coarse; the value is stale; the width calculation is a side-channel in `ci_floor_plan.dag`, not a property of the plan itself.

## Cost authority re-role (2026-06-25, operator design thread via sharp-stag-782)

**Derivation is the authority; measurement is the falsifier.**

Cost = `symbolic_cost(graph structure)` evaluated at a declared input envelope. Measuring where you can derive is the §5 fail-open smell — an unbounded input is the data-side of a non-terminating loop. So:

- `CostBasis = Predicted` — the AUTHORITY. Derived from graph structure + declared input envelope. This is what `spawn_width` and `ci_budget_tree` read.
- `CostBasis = Measured` — the FALSIFIER + calibration. `Measured.cost.space == Predicted.cost.space` must hold; a mismatch is a bug in the cost model. Also calibrates the host physical constants (allocator size-classes, `Value`-repr overhead) that the symbolic formula uses.

**Implication for this plan's nodes:**
- Node A (Rust plumbing, #5792): still lands as designed. `[measurement]` / `[calibration]` lines feed the **falsifier** role (§5 discriminating witness), not the scheduler directly.
- Node B (Runnable.cost): `CostAccount.basis = Predicted`; cost value comes from symbolic derivation (P2 in sharp-stag-782's roadmap `docs/plans/derived-cost-input-envelope-roadmap.md`), not from measurement.
- Node D (calibration loop): compares `Measured` to `Predicted`, flags mismatches. Physical-constant calibration updates the symbolic formula's input, not the cost row directly.

**BestEffort lease mode** (P4, calm-carp-204 ownership): `width = floor(budget / Predicted.cost.space)` — throughput-maximizing packing, no guarantee. Uses static data row as Predicted cost until P2 symbolic fold replaces it.

**Reserved lease mode** (P1/P2/P3/P6, sharp-stag-782 ownership): input-envelope-as-fact → symbolic-cost derivation → dissolve `min_bytes` → budget consumer.

---

## Current state (what exists vs. what's inert)

| Carrier | Status |
| --- | --- |
| `CostAccount<S> { time, space, power, basis }` in `std.realization_schedule` | **Always zero** — `cost_account_predicted_zero()` everywhere |
| `CostBasis = Measured \| Predicted` | Types exist, `Measured` never used |
| `shard_balance_slot_for_cost` in `std.realization_width` | Uses `CostAccount.time` to balance shards — cost is never populated |
| `WidthBoundedRealization { plan, spawn_width }` | Right carrier, produced by a side-channel not the scheduler |
| `memory_aware_spawn_width` | Formula correct; reads `per_shard_peak` from a static external data row |
| `PerformanceReceipt { wall_duration, sample_count, confidence }` in `compute_fabric` | Exists, not wired to `CostAccount.space` |
| `claim_batch.rs:74` `account_retained_memory()` | Emits interpreter heap bytes to stderr — never collected by `claim_executor` |
| `claim_batch.rs:30` `peak_rss_lines()` | Emits process `VmHWM` to stderr — never collected by `claim_executor` |

---

## Four nodes

### Node A — Measurement plumbing (v1 seed Rust)

**Scope:** make per-shard RSS observable as a structured output, collectable by `claim_executor`. The text lines are **transport** (a Lossless projection of `PerformanceReceipt.cost`, §4 one-grammar-both-directions), not the authority — `PerformanceReceipt` is.

- `claim_batch` emits a machine-readable line at process exit:
  `[measurement] per-shard-peak-rss: N bytes`
  (uses `/proc/self/status` VmHWM — this is the Space axis of a per-shard measured `CostAccount<Nano>`)
- `claim_executor` emits after the full walk:
  `[calibration] max-per-shard-peak-rss: N bytes at spawn_width=W`
  Current implementation: derives `per_shard = floor_rss / width` (a sound approximation); Node D refinement: parse per-shard lines from child stderr and take `max`.

**Authority chain (design-locked 2026-06-25 with sharp-stag-782):**
- `PerformanceReceipt.cost: CostAccount<Nano>` is the single measured authority (`cost.time` = wall_duration, `cost.space` = per-shard VmHWM, `cost.power` = unmeasured Watt(0), `basis = Measured`). Lives in `dsl/product/compute_fabric.dag`.
- `wall_duration` field removed from `PerformanceReceipt` — becomes a pure projection `fn performance_receipt_wall_duration(r) -> r.cost.time` (no stored state that can drift).
- `cache_state_summary` stays top-level on `PerformanceReceipt` — it is a measurement-context tag, not a cost axis. A cache-hit peak and a cold peak are different facts (§5 cache-impurity).
- `PerformanceReceiptSpaceContext { space: ByteSize, cache_state: NonEmptyStr? }` accessor gives sharp-stag-782's `ci_budget_tree` a paired (space, cache_state) without join risk.
- The `[calibration]` / `[measurement]` text lines are the Lossless wire projection of this receipt crossing the Rust→.dag boundary.
- **Shape PR:** nimble-tern-908 (work item `adhoc-e4d87b4e-49c`). Lands before any Rust emit format is finalized.

**Dissolution trigger:** when `Runnable` carries `CostAccount.space` (Node B) and the scheduler derives width from it, the `[calibration]` emit and `gunbc_ci_floor_measured_peak` dissolve into Phase 1.

**In-flight:** PR #5792 (gentle-newt-542), rust_tests=SUCCESS, ci=IN_PROGRESS (2026-06-25).

---

### Node B — Per-Runnable cost model (std substrate)

**Scope:** make `CostEstimate.space` a first-class substrate fact on `Runnable`.

- Add `cost: CostEstimate { space: ByteSize }` to `Runnable` in `std.realization_schedule`.
  (Already: `CostAccount<S>.space: ByteSize` exists as the roll-up; `CostEstimate` is the per-action input that feeds it.)
- Populate `cost` in `ci_floor_plan.dag`'s `gate_runnable()` from the measurement data rows (initially the values from Node A's calibration output; can be `predicted_zero` until Node D closes the feedback loop).
- `RealizationPlan.total` becomes a derived rollup via `schedule_critical_path_time` instead of `cost_account_predicted_zero()`.

**Depends on:** Node A (the measurements inform what value to put in the cost field).

Note: `realization-measurement-loop.md` Phase 0 covers the `CostAccount.time` arm. Node B extends the same Phase 0 to the `space` arm (per-shard RSS, not just wall-clock time). Same carrier, same phase, second axis.

---

### Node C — Resource-aware scheduler (std + workflow)

**Scope:** width derivation moves into the scheduler; the side-channel is deleted.

- `v2.workflow.scheduler` (or `WidthBoundedRealization`) takes a `memory_budget: ByteSize` and reads `max(runnable.cost.space)` from the plan to derive:
  `spawn_width = floor(memory_budget / max_runnable_space)`
- This makes `spawn_width` an output of `WidthBoundedRealization`, not a separately-computed side-channel in `ci_floor_plan.dag`.
- Delete `gunbc_ci_floor_spawn_width_for_budget` from `ci_floor_plan.dag`.
- Delete the separate `eval_spawn_width` call in `claim_executor.rs` — width comes from evaluating the plan function, not a second eval pass.

**Depends on:** Node B (scheduler reads cost from `Runnable`).

This is Phase 1 in `realization-measurement-loop.md` ("make parallelism visible + hardware-bounded width") — converges the width side-channel onto the same `PlacementSupplyRow` / `HardwareThreadCount` derivation the plan already models.

---

### Node D — Calibration loop (data layer)

**Scope:** measured per-shard data auto-updates per-Runnable cost estimates; static hand-authored data rows retire.

- After a run, the `[calibration]` output from Node A becomes the authority for `Runnable.cost.space` in Node B.
- Either: a CI step that reads the calibration line and rewrites `gunbc_ci_floor_measured_peak` (simple first form), or a `PerformanceReceipt`-typed store that the execution layer writes and the plan reads on next invocation (the proper long-term form — the resolved-graph cache from #5789 is the first building block of this pattern).
- Once landed: a host provisioning change (more memory) is reflected in `memory_budget` → scheduler derives higher `spawn_width` → throughput increases automatically. No manual data row edits required.

**Depends on:** Nodes A + B + C.

**Dissolution trigger:** `gunbc_ci_floor_measured_peak` and `gunbc_ci_floor_conservative_fallback_width` have no consumers outside `gunbc_ci_floor_per_shard_peak_rss_bytes()` and `gunbc_ci_floor_spawn_width_for_budget()` — both functions delete in Node C. The entire `ci_floor_measurement.dag` data section for concurrency/width becomes redundant once the scheduler self-calibrates.

---

## Dependency order

```
A (Rust plumbing, 1 PR)
  → informs B's cost values
B (std model, per-Runnable cost)
  → C depends on B
C (scheduler, width from plan — replaces side-channel)
  → D depends on A+B+C
D (calibration loop — data rows retire)
```

A and B can proceed concurrently; B can use provisional values until D closes the loop. C should land with B (a model with no consumer is a scaffold; model + scheduler land together). D is its own PR after A+B+C.

---

## What this is NOT

- Not a new caching mechanism — `realization-measurement-loop.md` owns the cache arc (Phase 0/P3 resolve-cache, Phase 2 cache kernel). This work is orthogonal: it's about *scheduling width*, not *whether to re-execute*.
- Not a placement mechanism — `compute-envelope-model.md` owns the BUILD phase fan-out / host packing / G1–G3 arc. This work covers the RUN phase (corpus shard width). Per that doc: "width-up is pids-safe on its own" — the two invariants don't multiply.

## Dissolution trigger (DESIGN §6)

Delete this doc when: (a) `gunbc_ci_floor_measured_peak` is retired (no consumers), (b) `spawn_width` is derived from the plan via `WidthBoundedRealization` rather than a side-channel, and (c) a host memory increase is provably reflected in higher width without any `.dag` data row edits — at which point the invariant is a by-construction property and this tracker is redundant.
