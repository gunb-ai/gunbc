# Self-host step memory audit

Subject: the slot memory peak of `gunbc test //gunbc/instruments:self-host` and of
`//gunbc/instruments:v2-native-cli`. This page carries NO measured figure (DESIGN §6): every number
lives in the receipt the instrument wrote, and the model reads it.

- Receipt: `tools/self_host_step_memory_receipt.tsv` (line kinds `bucket` and `stage`, both written by the seed in one run; every line's last column is the configuration cell `GUNBC_MEMORY_COMPOSITION_CONFIGURATION`; the model selects on that column).
- Model and reader: `gunbc.self_host_step_memory_demand` (`dag/gunbc/floor/self_host_step_memory_demand.dag`). Re-derive the readout from the repo root, under a cgroup budget:
  `gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/floor/self_host_step_memory_demand.dag --function self_host_step_memory_readout`
- Instrument, buckets: `GUNBC_MEMORY_COMPOSITION=1 GUNBC_MEMORY_COMPOSITION_RECEIPT=<file> GUNBC_MEMORY_COMPOSITION_CONFIGURATION=<cell> gunbc test //gunbc/instruments:self-host` (`src/v1/stage0/src/cli_run/memory_composition.rs`) appends one `bucket` line per released structure; the hook releases only after emission and only when the env is set.
- Instrument, stages: the same hook appends one `stage` line (resident set and VmHWM) at each seam inside the emission; the model's readout checks the buckets against the post-emission stage line of the same run.
- DECLARED FRONTIER, not in the receipt: the cgroup-level phase attribution (a 1 s sampler over `systemd-run --user --scope -p MemoryMax=22G -p MemorySwapMax=0` cut at the step's log seams, covering cargo/rustc and the controls) and the two-closure regression that splits the reconcile transient. The seed cannot write either. Trigger: an instrument entry (an `instrument_targets` row or a seed-side sampler) that writes `phase` lines into the same receipt; until then the figures for cargo/rustc are not claimed by this page or the model.

- DECLARED FRONTIER, closure scale: the entry-closure ASTs, resolution and typecheck environments and emission buffers are NOT measured, so the model carries no `EntryClosure` structure and asserts no ordering between tree-scale and moment-scale sums (the earlier `tree <= moment` check was true by construction of the table and is deleted). Trigger: a seed-side release stage that drops one of those structures and writes its `bucket` line; its row then enters `self_host_step_memory_structures` with `scale: EntryClosure`, and the readout may then assert the tree/closure split because a receipt can violate it.
- Writer failure: the seed refuses (exit 2, located stderr line) when a resident-set reading, a trim reading or a receipt write fails; a missing or partial receipt is therefore never a measurement, and the reader refuses an absent or duplicated line.

## What the receipt shows (read it there)
- The tree-scale structures (token pool, parsed heads, resolve index, path/graph caches) are `WholeTree` and are read as `bucket` lines.
- Whether demand is strictly decreasing: no. Tree-scale structures grow with the module count and nothing shrinks by itself; each ender in `self_host_step_demand_trend` is a named change.
- Which phase holds the step's high-water mark, and the rustc/product shares, are the sampler frontier above and are not asserted here.

## Provenance and standing (DESIGN §4d)
The committed receipt is the unedited output of one instrumented run of `gunbc test //gunbc/instruments:self-host`; its configuration column names the pool policy, the scope and the run. This audit head's cell is `gunbc.self_host_step_memory_demand` `self_host_step_memory_receipt_configuration` (container, no cgroup peak: srv1 was unreachable). Growth rate over time is NOT in the receipt: it is one week of tree history and is a bet.

## Levers and recommendation
Levers and the slot/trip recommendation are the lane report's, not this page's, because the pool-release lever is a production-path change measured in its own PR with its own receipt (dissolution trigger: closure-scoped ingestion). This page does not restate it.

## Consumption (DESIGN §3c): declared frontier
No executing consumer exists on this base. The named consumers are slot sizing, via the cold-run receipt row of `gunbc.cutover_receipt` and the note of `gunbc.runner_slot_desired`, which take the readout as the carrier of the per-step memory figure. Trigger: those modules import `gunbc.self_host_step_memory_demand` on a base that carries them (`integration/v1-closeout` carries `gunbc.cutover_receipt` since #13664).
