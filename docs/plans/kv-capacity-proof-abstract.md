# KV capacity proof: the model-agnostic abstract (for the DS4.1 lane)

Distilled from the GLM-5.3-Flash / group-B work (2026-09-24/25). Everything
here except section 4 is model-agnostic; section 4 is the worked GLM instance
to imitate. Governing doctrine: docs/plans/moa-memory-modeling.md.

## 1. The question the proof answers

Given a qualified subject — model, runtime, topology, parallelism, allocator,
workload, generation — decide, WITHOUT launching:

```text
CacheFitProved / CacheFitRefused / CacheFitUnestablished
```

for a workload intent (sequence_length × seats) against every rank's allocator
inventory. The whole-arm verdict (ArmMemoryFit*) is a CONJUNCTION of subproofs
of which this is one: static residency, cache capacity, prefill transient,
decode transient, load transient, supply reserve. CacheFitProved never implies
ArmMemoryFitProved while another subproof is open.

## 2. The one algebraic law

A hybrid model's persistent cache is NOT a fungible token inventory. It is a
sum over cache GROUPS with different growth shapes:

```text
PersistentCache(rank, length, seats) =
      seats × Σ(fixed-per-sequence groups)          (recurrent/state groups)
    + seats × Σ(per-token groups) × length           (full attention)
    + seats × Σ(windowed groups) × min(length, window)
    + seats × Σ(pooled-index groups) × ceil(length / pool_every)
    + allocator grouping / padding                   (per-group block rounding)
```

The four growth shapes (extdeps.vllm.kv_layout's KvGroupKind):

| shape | scales with | example |
|---|---|---|
| FullAttention | length × seats | dense attention layers |
| SlidingWindow | min(length, window) × seats | windowed attention |
| Mamba/KDA recurrent | seats ONLY | KDA/linear-attention state |
| KPoolIndex | ceil(length / pool_every) × seats | compressed sparse index |

Demand: `seats × cold_admission_blocks(exact_layout, length)` per group,
folded against `allocator_blocks` and `arena_bytes` on EVERY rank, joined by
the LIMITING rank (ranks are asymmetric — measure or derive per rank, never
average).

### The three forbidden moves

1. **No full-context-equivalent division.** The engine's printed "GPU KV cache
   size: N tokens" is a projection at one profile, not an inventory;
   `N / new_length` is a CacheCapacityCandidate, never a proof.
2. **No linear rescale of allocated arena.** "KV memory in use" is the
   RESIDUAL allocation (envelope − weights − activation − graphs), not demand
   of the configured seat.
3. **No negative or composite terms as credits.** A negative component
   observation is an attribution refusal (lower 0, upper open);
   `weights + non-torch` is a composite observation, not a placement receipt.

## 3. The proof machinery (all reusable)

- **MemoryProofSubject** (7 axes): model@checkpoint-digest, runtime@source-sha
  +image-digest, topology@host-census-revision, parallelism policy,
  allocator@configuration-digest, workload@intent-revision, generation.
- **MemoryTerm**: { component, placement_law, scaling_law, phase, lifetime,
  bound, derivation } — every byte names its mechanism.
- **Receipt-subject compatibility**: computed FROM the scaling law. Static
  residency transfers across workloads; per-sequence-fixed transfers across
  lengths at equal seat counts; every workload-sensitive term refuses across
  subjects; a TP4 receipt refuses TP2 at the topology axis. Generation is
  provenance, not a gate.
- **Measurement standings**: DerivedExact / DerivedConservativeInterval /
  MeasuredUpperBound / ObservationOnly / ModelFalsified. A point sample never
  becomes MeasuredUpperBound by being the largest seen once.
- **The allocation plan as the durable instrument**
  (ResolvedKvAllocationPlan): the engine emits its group roster — kinds,
  layer identities/counts, block sizes, page bytes, tokens-per-state,
  grouping/padding — and the plan MUST reproduce the engine's printed capacity
  as an independent control. After that, any workload point evaluates offline;
  wet runs become falsification, not sizing.
- **Discriminating controls** (witness these in any new instance):
  same arena + different roster → different capacity;
  missing group → Unestablished;
  pool_every 4→8 → derived capacity changes;
  cross-topology receipt → refused;
  cross-workload transient receipt → refused;
  negative observation → cannot improve headroom.

## 4. The GLM worked instance (what to swap for DS4.1)

GLM-5.3-Flash @ TP4 (GB10 unified pool, fp8 KV, 0.8567 envelope):

```text
architecture (from config.json, cited):
  45 layers = 34 KDA linear-attention + 11 sparse-attention (~3:1 interleave)
  KDA: 64 heads × dim 128 → recurrent state ∝ seats (≈1 MiB/layer/seq/rank)
  sparse: 512-dim MLA latent (∝ tokens), index_kpool 4 (∝ tokens/4), per-req tail
measured (bounded experiment, subject-bound per rank):
  pools 119.69–121.69 GiB; weights+nontorch 77.0–81.6 (composite, asymmetric);
  limiting rank TP3 (pool 119.69, arena 17.03 GiB)
engine control print: 1,805,689 tokens @ 64K profile → 27.55x at 65,536
candidate (NOT a verdict): floor(1,805,689 / 262,144) = 6 seats at 262K
open: 262K prefill transient (sparse-indexer buffer ≈ 40 × 262,144 × 132 B
      ≈ 1.289 GiB/rank, formula from the pinned runtime source)
```

For DS4.1, swap exactly: the architecture facts (layer census by attention
kind, head counts/dims, latent widths, any pooling), the pinned runtime
revision, the topology/host census, and the envelope policy. The fold, the
verdicts, the standings, the compatibility law, and the controls do not
change. DeepSeek-family specifics to resolve in the roster: MLA latent
dimensions, whether a pooled/index state exists, MoE expert placement under
the chosen EP policy (active-params is compute, NOT residency).

## 5. Module map (reuse, don't fork)

- `dag/extdeps/vllm/kv_layout.dag` — growth shapes, cold_admission_blocks,
  layout mint refusals. Add DS4.1's group roster HERE as cited architecture
  facts beside GLM's.
- `dag/extdeps/vllm/kv_group_metadata.dag` — the emission-gap model; the
  ResolvedKvAllocationPlan rung.
- `dag/gunbc/spark/arm_memory_fit.dag` — the three-arm verdicts, MemoryTerm
  census, receipt join, compatibility law.
- `dag/gunbc/spark/arm_memory_experiment_observed.dag` — the wet-bundle
  binding pattern (how receipts admit with standings).
- `dag/gunbc/spark/group_b_serving_capacity.dag` — the capacity-intent →
  derived-profile pattern (intent is the owner's decision; profile derives).
