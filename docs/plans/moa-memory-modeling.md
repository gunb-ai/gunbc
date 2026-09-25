# Mechanism-of-allocation memory modeling (owner directive 2026-09-25)

Amends the serving proof doctrine. Reading: **MoA = mechanism of
action/allocation** — which architectural feature allocates each byte, how that
quantity scales, which ranks hold it, during which phase it exists. The MoA
decomposition is the authority. Measurements serve only three purposes:

1. validate a derived bound;
2. fill a narrowly named term whose implementation exposes no derivation;
3. falsify the model when observation falls outside its claimed interval.

## The correction that prompted this

`floor(1,805,689 / 262,144) = 6` was on its way to `FitProved`. Withdrawn:

- The engine's printed "GPU KV cache size: N tokens" is a
  **full-context-equivalent projection**, not a fungible token inventory.
  `extdeps.vllm.kv_layout` already says so: hybrid layouts have
  length-dependent groups (full attention scales, windowed groups cap) plus
  fixed-per-sequence groups (KDA/Mamba state), and `cold_admission_blocks`
  refuses to linearly rescale a full-context figure.
- Six seats is therefore a **CacheCapacityCandidate**, not a proof. The cache
  subclaim proves only by direct group fold:
  `6 × cold_admission_blocks(exact_layout, 262144) ≤ allocator_blocks_on_limiting_rank`
  and independently
  `6 × cold_admission_bytes(exact_layout, 262144) ≤ admitted_kv_arena_bytes on EVERY rank`.
- The result vocabulary splits: `CacheFitProved / CacheFitRefused /
  CacheFitUnestablished`, and the whole arm stays `FitUnestablished` while any
  other subproof (prefill transient, decode transient, load transient, static
  residency, supply reserve) is open.

## GLM-5.3-Flash's four scaling laws (the MoA)

45-layer hybrid decoder: 34 KDA linear-attention layers + 11 sparse-attention
layers (interleaved ~3:1); 64 KDA heads × dim 128; index_kpool = 4; 512-dim
MLA latent; sparse MoE (~320B total, ~18B active — active-count is a COMPUTE
fact, not a residency fact; all experts are resident unless expert offload or
EP placement says otherwise).

1. **Weights: static, placement-dependent.** Derive from checkpoint manifest ×
   TP/EP/PP placement × loader transformation rules. The observed 77.0–81.6
   GiB/rank spread says placement is not an even quarter — build the exact
   rank-placement projection; do not enshrine measured constants.
2. **KDA state: ∝ sequences, NOT ∝ length × sequences.** Mamba-style recurrent
   state per layer per sequence per rank (illustratively 16 local heads ×
   128×128 × 4B = 1 MiB → ~34 MiB over 34 layers, before conv state and
   speculative additions). Six seats ≈ hundreds of MiB, not a linear rescale.
3. **Sparse MLA + KPool index: token-dependent.** 11 sparse layers: MLA latent
   cache (∝ tokens), KPool compressed index (∝ tokens/4), small per-request
   tail state (fixed). Persistent cache law:
   `seats × (KdaState + TailState) + seats × SparseMla(length) + seats × KPool(⌈length/4⌉) + allocatorGroupingAndPadding`.
4. **Prefill/decode workspaces: phase-specific transients.** Derivable, not
   discoverable: e.g. the upstream sparse-indexer prefill buffer carries 40
   entries per model token × 132 bytes (128-dim FP8 vector + scale) → at
   262,144 tokens ≈ 1.289 GiB per applicable rank — check against the exact
   deployed revision. **Read the allocator formula; never learn it from OOMs.**

## Admissibility of observations

- A resident-demand component can never be negative. TP1's `−2.54 GiB` graph
  figure is preserved raw but admitted as `GraphAttributionUnestablished`
  (lower bound 0, upper open) — the profiler interval includes a release.
  Never clamped silently, never a credit. A negative observation improving a
  verdict violates weaker-evidence-cannot-improve-admission.
- `weights + non-torch` is a COMPOSITE profile term (rank-specific
  model/runtime residency at that experiment phase), NOT a checkpoint-
  placement receipt. It does not separate checkpoint tensors from
  runtime-fixed, allocator, communication, or other non-Torch allocations.
- Measurement standings: `DerivedExact / DerivedConservativeInterval /
  MeasuredUpperBound / ObservationOnly / ModelFalsified`. A point sample never
  becomes MeasuredUpperBound merely by being the largest seen once.

## The unified proof subject

Every term binds to one exact qualified subject (qualified-subject-identity.md):

```text
MemoryProofSubject {
  model:      model:zai/GLM-5.3-Flash@checkpoint-digest
  runtime:    runtime:vllm@source-sha+image-digest
  topology:   topology:fleet/group-b/tp4@host-census-revision
  parallelism: parallelism:tp4/pp1/ep-policy/dcp-policy
  allocator:  allocator:vllm/hybrid-kv@configuration-digest
  workload:   workload:glm-serving/262144x6@intent-revision
  generation: generation:group-b/<launch-generation>
}
```

A 64K×1 receipt discharges only terms invariant across the move to 262K×6.

Every memory term names its mechanism:

```text
MemoryTerm { component, placement_law, scaling_law, phase, lifetime, bound, derivation }
```

Whole verdict = conjunction of subproofs: `StaticResidencyFit`,
`CacheCapacityFit`, `PrefillTransientFit`, `DecodeTransientFit`,
`LoadTransientFit`, `SupplyReserveFit`. `FitProved` only when every applicable
rank × phase proves.

## The allocation-plan projection (the durable instrument)

Patch or wrap the pinned engine to emit its allocation plan — introspection,
not tuning:

```text
ResolvedKvAllocationPlan {
  runtime revision / image digest, cache format, allocator block count,
  groups: [ identity, spec kind, layer identities, block size, page size bytes,
            tokens per state, max memory usage bytes,
            blocks required at the requested sequence length ],
  grouping/padding decision
}
```

The plan must reproduce the engine's printed capacity as an independent
control. Production admission then evaluates any workload point offline from
the group roster; wet execution becomes falsification + compatibility check,
not the sizing algorithm. (`kv_group_metadata` documented the emission gap:
layer counts and page bytes absent — this closes it at the source.)

## Discriminating controls (required witnesses)

```text
same arena bytes + different group roster      → different capacity
same printed token figure + changed KDA count  → no proof reuse
missing sparse-indexer tail group              → CacheFitUnestablished
index_kpool 4 → 8                              → derived capacity changes
TP4 receipt applied to TP2                     → refused
64K×1 activation receipt applied to 262K×6     → refused
negative component observation                 → cannot improve headroom
all cache groups prove, prefill open           → CacheFitProved + ArmMemoryFitUnestablished
```

## What remains legitimately measured

Foreign/host pool occupancy; driver allocations with no deterministic API;
fragmentation with no conservative construction bound; backend workspaces with
no formula nor sizing function; realized behavior that violates the model
(falsification). Everything else derives.

## Current honest standing (2026-09-25)

```text
Per-rank pool supply:                    observed
Rank-specific composite residency:       observed (composite, not placement)
KV arena by rank:                        observed
Limiting rank:                           TP3 / srv11 (that launch)
262K × 6 cache cost:                     CacheCapacityCandidate
262K × 6 prefill transient:              unestablished
Whole-arm verdict:                       FitUnestablished
```

262K is entirely plausible on this topology — the hybrid architecture exists
precisely to avoid dense KV growth on all 45 layers — but the durable answer
is an argument from architecture + pinned allocation laws + topology +
workload, not a fitted line through one experiment.
