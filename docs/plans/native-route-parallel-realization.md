# Native-route parallel realization (Phase 1 model)

Durable record of the Phase 1 model sent to `eager-raven-113` and accepted with four binding conditions (dashboard GO, 2026-09-14). This document is the artifact to keep. Implementation is not landed under operator wind-down.

Independent program beside A (#11224, frozen, untouched) and B (#11217, royal-hawk owns B and the producer counts). Nothing here may make B's AFTER measurement harder to interpret. A's frozen subject is never altered or delayed.

Gatekeeper: eager-raven-113. Host for later builds/experiments: srv1 (`ssh srv1-lan`; `systemd-run --user --scope -p MemoryMax=24G`; `CTRL_BUILD_MODE=local`). srv2 is reserved for A. Receipt class 3/4 (native driver/observer) → pre-landing exact-head receipt when a later change actually lands code.

## Not credited

Do not claim later:

- `3x`
- `five minutes`
- the serial projection
- semantic equivalence of fierce-lark's merged shard surface

The shard experiment refuted identity volume, memory pressure, allocator return, and host-load tail as causes. The five-shard signature is not yet module-caused (rotation discriminator first; fierce-lark runs it on srv1).

## Homes (cite, do not fork)

Homes accepted as written, not forked:

- `std.realization` — `Placement.LocalChildProcess` (fifth arm). Serial `SourceRootEvalDriver` remains `LocalInProcess`. A prepare worker is a local OS process with its own address space, not a filesystem cache, not a remote host, not an accelerator. Do not nickname Placement as `WorkerKind`. Do not put the arm in `gunbc.compute`. Worker `RealizedStep` = `ExecuteEffect` + `LocalChildProcess` + `Recompute`. Parent merge stays `LocalInProcess` plus `ReadEffect` over worker artifacts (condition 3). Slice 1 does not take a Share cache obligation across siblings.
- `std.realization_width` — envelope is `process_memory_aware_spawn_width` (cores ∩ memory ∩ pids). This is the bound fold, not a second knob. Do not mint `NATIVE_SHARD_COUNT` / `--width`. `NATIVE_SHARD_INDEX` is an observation label only. **Precondition, fail-closed (landing side chat, 2026-09-15):** that helper is not the envelope on its own — it returns `conservative_fallback_width` when the memory or PID observation is zero, and clamps to one worker when the memory or PID budget fits none, so it can answer with a width outside the observed envelope or admit one worker when physical fit is zero. This realization may call it only after a fail-closed contract has established nonzero, readable CPU, memory and PID inputs and capacity for at least one worker; missing inputs or zero fit produce a named refusal (`NoFeasibleRealization`), and the helper's conservative-fallback and minimum-one arms are not admissible answers here. If a stricter existing operation with exactly those semantics is found at implementation time, bind that instead of wrapping this one.
- `std.decision` — width *choice* is `select_realization` with a named `DecisionSubject` and the two funded axes from `width_fold_objective_goals` (wall, peak memory; both LowerIsBetter). A Pareto front is not a winner. Requested N above the envelope is `NoFeasibleRealization` (later named `PrepareWidthRefusedAboveEnvelope` in the prepare-shard contract), never a silent clamp. Width is not computed inside `std.goal_assessment` / `ensure`.
- `gunbc.compute.work_request` / `work_provider_local` — **do not extend** `WorkOperation`. Prepare shards are native-driver work (`NativeTestContext` / `native_test_prepare_module`), not cargo `CompileEntry`.
- Instrument authority = `scratch/deep-cat-655-span-bisect@5b41958d07d9` (`[native-module-residual]`). Reuse; do not re-derive. Do not retouch `[native-prepare-split]`. Named children subtracted from module parent: prepare, eval, row_accumulation, arm_exit, release, meter. Resolve and infer are reported and nested inside prepare; they are not subtracted. Residual uses checked arithmetic and refuses on negative / missing / duplicate / unmatched. Prepare parent = fixed + Σ module parents + residual; if parent is not nested, refuse.

## Slice 1 construction (approved shape; not landed)

One immutable context in the parent (`NativeTestContext` via `native_test_context_from_ingest`). Fork after this. Context is read-only for workers.

Complete module manifest from that context + `universe.modules`. Assignment: **contiguous slices** over stable module-identity order (condition 4). The same function is used for W=1 and W=N.

Each worker (`LocalChildProcess`) prepares only its assigned modules and writes one artifact.

Named merge step: `prepare_shard_merge`. Law: bidirectional identity+payload join against serial (arm i). Added ∪ removed ∪ payload-changed ∪ duplicate ⇒ refuse the equivalence claim. Completeness is the join, not a count equality. Parent exclusive rows gain a named merge span; it is not stuffed into prepare residual.

## Binding conditions (eager-raven-113 GO)

1. **Acceptance is three arms, not two.** On one pinned quiet-host subject measure:
   - (i) in-process serial (today's production `SourceRootEvalDriver`);
   - (ii) W=1 via `LocalChildProcess` + `prepare_shard_merge` (placement price: fork, serialization, merge);
   - (iii) W=N.
   Report all three with total CPU, aggregate memory, process count, parent/worker partitions, merge span named. `(ii)−(i)` is the placement price; `(iii)−(ii)` is the width effect. Merge-law join runs against (i).
2. **Compose on the row-set reshape (#11374).** Add one variant + one roster entry on the keyed exclusive-row collection. Do not add a field to the old literal `NativeDriverExclusiveRows` struct. Build on a branch that includes #11374's head; rebase when it lands on main.
3. **Worker transport:** artifacts under `RUNNER_TEMP`, one file per worker, content-identified by digest; parent `ReadEffect`; missing / malformed / digest mismatch **refuses** (no partial merge). Digest join is part of the merge law.
4. **Assignment:** contiguous slices over stable module-identity order (printable interval; rotation can move whole sets). Same function for W=1 and W=N.

Everything else as in the model, including: no second width knob; `NATIVE_SHARD_INDEX` is an observation label; slice 2 declarations land with slice 1 but implementation waits on slice 1's green three-arm receipt; receipt class 3/4; srv1 only.

RED list for a first implementation push (not this wind-down PR): merge-law added/removed/changed; residual refuse arms; width > envelope refuses; missing worker artifact refuses.

## Slice 2 — modeled with slice 1, implemented after proof

Context map/reduce: partition ingest reads by source-root file identity. Mappers produce partial fold state; reduce concatenates roots, merges indices fail-closed on duplicate module names, concatenates file_refusals. Placement: same `LocalChildProcess` arm. Width: same envelope + `select_realization`. Merge name: `context_shard_reduce`. The reduce is not a fallback to serial context. Implementation waits on slice 1's merge law + instrument + three-arm receipt existing and green.

## Honesty vs a later implementation attempt

A construct attempt existed on `session/crisp-hawk-831` / #11378 and is **not** the landed program. Known gap against this model: children that re-exec and rebuild context are not fork-after-context; `(ii)−(i)` would be contaminated until workers inherit the parent's context (COW fork or a serialized context artifact) without re-ingest.

## Where bytes would live (when implementation is authorized again)

- `dag/std/realization.dag` — `LocalChildProcess`
- exhaustive `Placement` matches (accelerator demo and peers)
- `std.compiler_entry` keyed exclusive roster — `ExclusivePrepareShardMerge` / wire name `prepare_shard_merge`
- `gunbc.native_prepare_shard` — contract (envelope, selection, assignment, merge, artifacts, residual law, slice 2 standing)
- `v1.compiler.emit_rust` `SourceRootEvalDriver` — production default remains in-process; child path is opt-in via placement + selected width (selection receipt, not a second knob)
- witnesses: `dag/test/claim/native_prepare_shard_witness_test.dag` and exclusive-key exhaustive matches including `ExclusivePrepareShardMerge => 0`
