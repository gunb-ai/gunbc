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

## Provenance
The committed `tools/self_host_step_memory_receipt.tsv` is the unedited output of one instrumented run by this head's writer: `GUNBC_MEMORY_COMPOSITION=1 GUNBC_MEMORY_COMPOSITION_RECEIPT=<file> GUNBC_MEMORY_COMPOSITION_CONFIGURATION="pool=held_through_resolution;scope=srv1_systemd_scope_22g_swap0;run=smart-gull-336-20261010T1157Z" gunbc test //gunbc/instruments:self-host` on srv1 under `systemd-run --user --scope -p MemoryMax=22G -p MemorySwapMax=0` (exit 0). Every line carries that cell as its last column; bucket lines carry the full resident drop across release and `malloc_trim` and the trimmed figure beside it. Re-derive the readout with `gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/floor/self_host_step_memory_demand.dag --function self_host_step_memory_readout`.

## Levers and recommendation
Levers and the slot/trip recommendation are the lane report's, not this page's, because the pool-release lever is a production-path change measured in its own PR with its own receipt (dissolution trigger: closure-scoped ingestion). This page does not restate it.

## Consumption (DESIGN §3c)
Executing consumers in this change: `test.claim.floor.self_host_step_memory_demand_witness_test`, which runs `self_host_step_memory_readout_over` and `measured_lowering_holds` over supplied receipt rows (a complete receipt holds; a missing structure line, a malformed bytes cell, a resident-set bound violation and an unverifiable measured lowering refuse), and the readout entry itself over the committed receipt, `gunbc run --source-root dag --source-root src/v2 --entry dag/gunbc/floor/self_host_step_memory_demand.dag --function self_host_step_memory_readout`, whose exit each PR records. Declared frontier: a `gunbc test` label for the readout, which needs a `TargetProducer` arm in `gunbc.target_binding` and its seed arm (the trigger `gunbc.memory_composition_seed_growth` names); until then the readout is on no required path. `gunbc.runner_slot_desired` is on main and sizes the slot as a policy row; it does not import an effectful entry, so it is not a consumer of this module.
