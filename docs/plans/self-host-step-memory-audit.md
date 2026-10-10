# Self-host step memory audit (2026-10-10)

Subject: the 17.2 GB slot peak of `gunbc test //gunbc/instruments:self-host` (run 38016653087).
Model row (the authority; every number below is a row there, not a transcription):
`gunbc.floor.self_host_step_memory_demand`. Re-derive the readout:

    gunbc run --source-root dag --source-root src/v2 \
      --entry dag/gunbc/floor/self_host_step_memory_demand.dag \
      --function self_host_step_memory_demand_readout     # rc 0 green, run under a cgroup budget

## Instruments (measured)
- Phase attribution: a 22G/swap0 `systemd-run --user --scope` on srv1, cgroup memory.current/peak plus process-tree RSS sampled at a fixed beat; seams from the step's stderr log lines.
- Composition: env `GUNBC_MEMORY_COMPOSITION=1` (`cli_run/memory_composition.rs`) reads RSS, then releases structures in order with `malloc_trim` readback, separating live from freed-but-retained.
- Counterfactual: `GUNBC_MEMORY_COMPOSITION_DROP_POOL_BEFORE_RESOLVE=1` drops the whole-tree pool before closure resolution.

## Findings
1. Step-alone peak is 14.39 GB (self-host) and 14.23 GB (native-cli); the slot's 17.2 GB adds ~2.8 GB of co-resident runner/page-cache state.
2. The high-water mark is held by the **seed process in compile.reconcile**, not rustc (rustc ~6.3 GB, product ~20 MB). The cargo phase runs ~13.5 GB only because the seed's pool is still resident there.
3. Composition at the seed peak: whole-tree token pool 4.64 GB, pool heads 0.54 GB, name census / resolve index 1.82 GB (all measured, tree-scale: 8,196 pooled modules, 7,959 census-only); reconcile transient ~5.4 GB closure-independent and ~1.9 GB closure-proportional are **inferred** by regression (read obligation: tagged allocation attribution inside compile.reconcile).
4. **Counterfactual, measured:** dropping the pool before resolution lowered the cgroup peak from 14,393,454,592 to 10,485,690,368 bytes (-3.9 GB) with the step green (rc 0). Resolution does not need the token pool after the census.

## Trend: is it strictly decreasing?
No. Tree-scale buckets grow with the module count and the closure bucket with `v2.compiler.compile`'s size; nothing shrinks by itself. Demand is a staircase that is flat or rising between named changes (see `self_host_step_demand_trend`).

## Levers (displaced bytes per unit of change)
1. Release the pool before resolution: **-3.9 GB measured**, a small, local change (the counterfactual is already a one-call readback). Do this first.
2. Closure-scoped ingestion: up to ~12.4 GB of the 14.4 GB peak, large change; subsumes lever 1.
3. Locate the 5.4 GB closure-independent reconcile transient (step-zero measurement, then derive).
4. Native generation-two compiler: unmeasured; its own cold-run receipt (`gunbc.cutover_receipt`) is the instrument.

## Recommendation
- 22 GiB / 21 GiB High / 20 GiB trip hold today (peak 17.2 GB) but the observed growth (~120 modules/day, ~1.5 MB/module resident, one-week window, so a bet) exhausts the 20 GiB trip in ~24 days without lever 1. Land lever 1 first; it restores ~3.9 GB (~months of headroom) and the 22/21/20 envelope is then right for six months.
- The v2-native slot class starts from its own cold-run receipt; the native-cli step-alone 14.23 GB (same shape as self-host) is the only measured floor, so begin at the same 22 GiB and re-size from the first native receipt.
- `gunbc.compiler_gate_workflow` / `gunbc.emitted_subject_build_gate` are untouched here.

Measured vs inferred: findings 1, 2, 4 and bucket sizes in 3 first four are measured; the 5.4/1.9 GB split and the growth rate are inferred.
