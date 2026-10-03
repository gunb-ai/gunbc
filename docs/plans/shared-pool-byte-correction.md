# The shared-pool byte correction (owner directive 2026-09-26)

Amends docs/plans/moa-memory-modeling.md. The 738/1035 block result is
coherent; the 125,010,432-byte result beside it was dimensionally wrong by
~106× and is withdrawn (it becomes a RED control).

## The defect

The allocation-plan fold computed:

```text
per-seat bytes = Σ group.blocks_at_requested_length × group.page_size_bytes
```

But `spec.page_size_bytes` is the page represented by that cache-group
specification — NOT the physical byte cost of taking one block ID from the
shared pool. At the pinned revision vLLM derives a separate physical pool
quantity (`_get_kv_cache_bytes_per_block`): for GLM-5.3-Flash the shared pool
block size comes from the packed MLA and index pages; `num_blocks =
available_memory // bytes_per_block`; the worker allocates one backing tensor
of `bytes_per_block × num_blocks`, and cache groups OVERLAY that allocation —
a block ID is owned by one group at a time. The check: 738/1035 blocks is
71.30% of the pool; against TP3's 17.29 GiB arena that is ~12.33 GiB of
block-equivalent capacity, not 0.116 GiB. The fold implied ~169 KB/block vs
the ~17.1 MiB/block the arena and inventory imply.

## The corrected model

```text
KvAllocatorCarve      — physical resident pool allocated at startup
KvAdmissionDemand     — logical blocks required by the promised population
```

(Calling both "KV bytes" is what enabled the mistake.)

The allocation plan must carry:

```text
allocator_blocks
physical_pool_bytes_per_block
allocated_pool_bytes
```

with the old per-group field renamed `group_spec_page_size_bytes` (and
`group_blocks_per_max_request`) so it cannot be read as the shared pool block
size. Conservation controls:

```text
allocated_pool_bytes == allocator_blocks × physical_pool_bytes_per_block
allocated_pool_bytes <= every rank's admitted KV arena
arena − allocated_pool_bytes < one physical pool block
  (modulo block override, null-block reservation, profile rounding)

request_blocks = Σ exact group blocks at requested length
request_block_equivalent_bytes = request_blocks × physical_pool_bytes_per_block
```

The 125,010,432 result is a RED control: 738/1035 blocks cannot occupy <1%
of the same pool.

## The verdict structure

The block wall is authoritative; the byte wall is a conservation CHECK on the
pool's construction, not an independent capacity answer:

```text
PoolConstructionProved { physical_pool_block_bytes, allocator_blocks,
                         allocated_pool_bytes, per-rank supply }
RequestBlockCostProved { exact group roster, blocks_per_request }
CacheAdmissionProved   { request_blocks <= usable_allocator_blocks }
```

`CacheFitProved` carries: limiting_rank, seats, length, blocks_per_seat,
blocks_used, blocks_reserved, hard_ceiling. If bytes appear, they are named
`block_equivalent_bytes` — never ordinary memory demand or physical headroom.

## The cache frontier (block wall)

| Seats | Blocks used | Blocks left | Reserve |
|---:|---:|---:|---:|
| 6 | 738 | 297 | 28.70% |
| 7 | 861 | 174 | 16.81% |
| 8 | 984 | 51 | 4.93% |
| 9 | 1,107 | −72 | REFUSED |

These are shares of the already-allocated block inventory, NOT additional
process-memory allocations (vLLM allocates the whole pool at startup).

## Reserve policy — hard ceiling ≠ guaranteed service

```text
CacheReservePolicy
  = NoOperationalReserve
  | ReserveBlocks { blocks }
  | ReserveFullLengthSeats { seats }
  | ReserveBasisPoints { basis_points }
```

Standings: hard cache ceiling 8; normal guaranteed seats selected by policy;
burst/cache-only ceiling 8. First production policy candidate
`ReserveFullLengthSeats { seats: 1 }` → **7 normal 262K seats**. Six remains
the conservative restoration point (28.7% reserve). Eight is not advertised
as normal guaranteed service until transient and SLO proofs support it.

## Mixed-length service (beyond 8 sessions)

The uniform "every seat may reach 262K" promise caps at 8. More ACTIVE
sessions are possible when not every session reserves 262K:

```text
SeatClass { max_context_tokens, guaranteed_count }
ServingCapacityIntent { guaranteed_classes, max_active_sequences,
                        max_batched_tokens, cache_reserve_policy }
```

Admission prices the actual reservation population through the same
allocation plan (fixed KDA per-sequence, length-growing MLA, pooled KPool,
fixed tail, grouping/padding) — NO second seat calculator. Until a real
request-admission boundary enforces the mixed budget, `max_num_seqs = N`
exposes N slots that may ALL submit maximum-length requests, and the safe
proof prices that maximum behavior.

## Cache fit is still not the whole arm

The bounded run established initialization + per-rank profiles + the plan; it
did NOT execute six simultaneous 262K requests. Before promoting seven or
eight: (1) pinned-source prefill derivation at the deployed revision;
(2) profile-shape binding (a 6-seat activation/graph receipt does not prove
an 8-seat profile); (3) a bounded falsification run at the derived worst
legal prefill/decode shape; (4) service capacity/SLO evidence (throughput,
latency, preemption, failure behavior). Final result shape:

```text
ServingCapacityPoint { workload_intent, cache_standing, prefill_standing,
                       decode_standing, static_residency_standing,
                       reserve_policy, slo_standing }
```

— a Pareto frontier, not one magic tuple.

## Operational sequence (owner-set)

1. Interrupt the byte-wall completion (done).
2. Land the shared-pool byte correction + RED control.
3. Recompute the exact cache frontier (6/7/8 by one fold).
4. Close the 262K prefill obligation for six seats.
5. Restore persistent serving at 262K × 6 with the planted-residue
   convergence proof and exact readback.
6. Evaluate 262K × 7 under one-full-seat reserve.
7. Retain 262K × 8 as hard/burst ceiling until phase + SLO evidence.
8. Add mixed-length token-weighted admission for stranded block capacity.
