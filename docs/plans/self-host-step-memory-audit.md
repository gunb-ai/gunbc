# Self-host step memory audit

Subject: the slot memory peak of `gunbc test //gunbc/instruments:self-host` and of
`//gunbc/instruments:v2-native-cli`. This page carries NO measured figure (DESIGN §6): every number
lives in the receipt the instrument wrote, and the model reads it.

- Receipt: `tools/self_host_step_memory_receipt.tsv` (line kinds `bucket`, `inferred`, `sampler`, `phase`, documented in the model's header).
- Model and reader: `gunbc.self_host_step_memory_demand` (`dag/gunbc/floor/self_host_step_memory_demand.dag`). Re-derive the readout from the repo root, under a cgroup budget:
  `gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/floor/self_host_step_memory_demand.dag --function self_host_step_memory_readout`
- Instrument, buckets: `GUNBC_MEMORY_COMPOSITION=1 GUNBC_MEMORY_COMPOSITION_RECEIPT=<file> gunbc test //gunbc/instruments:self-host` (`src/v1/stage0/src/cli_run/memory_composition.rs`) appends one `bucket` line per released structure; the hook releases only after emission and only when the env is set.
- Instrument, phases: a 1 s sampler over `systemd-run --user --scope -p MemoryMax=22G -p MemorySwapMax=0` reading the scope's `memory.current`/`memory.peak` and each pid's `VmRSS`, cut at the step's own log seams (`[pre-entry]`, `compile.*`, cargo invocations); one `phase` line per seam with the seed RSS, the rustc RSS sum and the cgroup maximum.

## What the receipt shows (read it there)
- The step's high-water mark is held by the seed process in `compile.reconcile`, not by rustc or the product under test.
- The tree-scale structures (token pool, parsed heads, resolve index, path/graph caches) are `WholeTree`; the reconcile transient splits into a closure-independent part and a closure-proportional part by regression, and both are `inferred` lines: bets, not facts.
- Whether demand is strictly decreasing: no. Tree-scale structures grow with the module count and nothing shrinks by itself; each ender in `self_host_step_demand_trend` is a named change.

## Provenance and standing (DESIGN §4d)
The committed receipt was assembled from one srv1 run set (composition run, two cold runs) by the recipe above. `bucket` and `sampler`/`phase` lines are measured; `inferred` lines are a two-closure regression with a stated read obligation (tagged allocation attribution inside `compile.reconcile`). Growth rate over time is NOT in the receipt: it is one week of tree history and is a bet.

## Levers and recommendation
Levers and the slot/trip recommendation are the lane report's, not this page's, because the pool-release lever is a production-path change measured in its own PR with its own receipt (dissolution trigger: closure-scoped ingestion). This page does not restate it.

## Consumption (DESIGN §3c): declared frontier
No executing consumer exists on this base. The named consumers are slot sizing, via the cold-run receipt row of `gunbc.cutover_receipt` and the note of `gunbc.runner_slot_desired`, which take the readout as the carrier of the per-step memory figure. Trigger: those modules import `gunbc.self_host_step_memory_demand` on a base that carries them (`integration/v1-closeout` carries `gunbc.cutover_receipt` since #13664).
