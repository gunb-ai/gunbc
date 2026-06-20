# Plan — compute as a uniform allocator (the fabric DAG)

**Status:** draft for operator confirmation. One seam is open (marked ⬚ below).
**Date:** 2026-06-20
**Author:** warm-crane-135
**Motivation:** the "CI is extremely flakey" / "srv2 crashing hourly" investigation
(`docs/postmortems/2026-06-20-ci-flakiness-fleet-overcommit.md`) bottomed out at one missing
invariant. This plan is the durable fix.

---

## 1. The problem, measured

A live census of one fleet host (125 GiB RAM, 128 cores), taken 2026-06-20 ~05:38:

- **20 agent-session containers**, each capped at **31.27 GiB**, plus the `system-actions-runner.slice`
  (CI floors) as a *separate* systemd slice, plus the kernel — all sharing **125 GiB physical**.
- Sum of container caps alone: **625 GiB on a 125 GiB host = 5× overcommit.**
- Survives only because sessions are *usually* idle (aggregate ~30 GiB at census time). A few
  simultaneously-busy sessions cross 125 GiB → **OS memory livelock = the hourly reboot.**

| consumer | how it launches today | memory cap | accounted against host total? |
|---|---|---|---|
| agent session (×20) | dashboard spawn → `docker run` | 31.27 GiB each | **NO** — per-container cap, no sum check |
| CI floor run | GitHub Actions → `system-actions-runner.slice` | slice MemoryMax | partial (its own slice, not vs sessions) |
| ctrl-build / cargo / sccache | under a session or the runner | inherited | **NO** |
| kernel + system | host | — | reserve only |

**The single missing invariant:** nothing enforces `Σ(live allocations on a host) ≤ physical RAM`.
Each consumer is independently capped; the *sum* is unbounded by construction (625 > 125). Every
symptom we chased — the per-run floor width (#5375), the per-host admission gap, the Arc 1 31 GiB
resolve OOM — is this one invariant seen from a different angle.

It also explains **"is srv1 even being used?"**: all 20 sessions are packed on one host. A placement
step would spread them; srv1 idle while srv2 carries everything *is* the missing placement.

---

## 2. The DAG we're building

```
        CONSUMERS (all just ask for compute — demand)
        ┌───────────────┬──────────────────┬─────────────────────┐
        │ CI floor unit │ agent session     │ docker / ctrl-build │
        └───────┬───────┴─────────┬─────────┴──────────┬──────────┘
                │                 │                    │
                ▼                 ▼                    ▼
              ComputeRequest { class }            (a canned WorkDemand)
                │                 │                    │
                └────────┬────────┴─────────┬──────────┘
                         ▼                  ▼
        COMPUTE FABRIC — the single allocator (product.compute_fabric)
        ┌──────────────────────────────────────────────────────────┐
        │ AllocationClass: Small 2c/2g · Medium 4c/4g ·             │
        │                  Large 8c/8g · XLarge 16c/16g (fixed)     │
        │ admit(host, live_receipts, request):                     │
        │     grant IFF Σ live_receipts + request ≤ grantable      │  ← THE invariant
        │ place(request): pick a host with room (else queue/refuse) │
        └───────────────────────────┬──────────────────────────────┘
                                     │ AllocationReceipt { host, slice, class }
                 ┌───────────────────┴───────────────────┐
                 ▼                                        ▼
            srv1 (grounded via BMC)                  srv2 (grounded via BMC)
            grantable = total_memory_bytes           grantable = total_memory_bytes
                        − reserves                               − reserves
            cgroup slices = granted receipts         cgroup slices = granted receipts
```

Reuses what already exists in `dsl/product/compute_fabric.dag`: `WorkDemand` / `ResourceEnvelope`
(demand), `ComputeOffer` / `ComputeHost` (supply), `satisfies(offer, demand) → ComputeLeaseEligibility`
(matching), `AllocationReceipt` (grant). The supply side is grounded by vivid-bee's BMC seam
(`BmcTelemetrySnapshot → bmc_ground_host_observations → BmcGroundedHostObservations` with
`processor_core_count`, `total_memory_bytes`) — **no hand-typed fleet rows.**

What is *new*: (a) `AllocationClass` (fixed sizes), (b) the `admit` sum-invariant, (c) routing **every**
consumer through the request path so the accounting is complete.

---

## 3. Design decisions

1. **CI is a pure consumer.** Today `ci_fleet.dag` + `ci_floor_plan.dag` compute spawn_width, memory
   budgets, the 14 GiB constant. All of that moves *into* the fabric. CI's job shrinks to emitting a
   `ComputeRequest { class }` per floor unit — it never sees a host, a width, or a byte. This deletes
   the entire spawn_width / per-unit / probe debate from CI.

2. **Fixed allocation classes (no demand/supply optimization yet).** `Small 2c/2g · Medium 4c/4g ·
   Large 8c/8g · XLarge 16c/16g`, each a canned `ResourceEnvelope`. A request names a class; the fabric
   guarantees that envelope and *caps* it. This is a strict simplification: probe-derived sizing
   (postmortem fix #2) can later replace the constants *behind the same `class` interface* without
   touching consumers.

3. **Completeness is the whole point.** The `admit` invariant only prevents overcommit if **every**
   workload is a receipt. The enforcement point is therefore where workloads actually launch:
   - **agent sessions:** the dashboard spawn path (today it `docker run`s a 31 GiB-capped container with
     no sum-check — this is the origin of the 625 GiB).
   - **docker / ctrl-build:** a ctrl wrapper that requests before running.
   - **CI floor:** already modeled; re-point to emit a request.

4. **Placement spreads load.** `place()` picks a host with room, so sessions fan across srv1+srv2
   instead of packing one. Fixes the srv1-idle imbalance for free.

---

## 4. Open seam (needs operator confirm) ⬚

- **(a)** Consumer→fabric granularity: does CI emit **one request per floor unit** (fabric decides
  count + placement), or one request for the whole floor with an internal width? The drawing assumes
  per-unit; confirm.
- **(b)** Enforcement point — **confirmed by evidence**: the dashboard session-spawn path is where the
  625 GiB of unbounded caps originate, so it is the right place to insert `admit`. (Plus a ctrl docker
  wrapper for non-session containers.)

---

## 5. Phased plan (smallest real slices, each independently verifiable §5)

1. **Model:** `AllocationClass` enum + 4 fixed `ResourceEnvelope`s + `admit(host, live_receipts,
   request) → AllocationReceipt?` (bin-pack fixed sizes onto `grantable`, refuse on overflow) in
   `compute_fabric.dag`. Discriminating witness: a 21st `Medium` on a full host is refused; the same on
   an empty host is granted. **No host change — pure model + witness.**
2. **CI as consumer:** re-point `ci_floor_plan` to emit `ComputeRequest { class }` instead of computing
   width. Proves CI-as-pure-consumer on the one consumer already modeled.
3. **Ground supply:** fill each host's `grantable` from the BMC seam (real `total_memory_bytes` −
   reserves), retiring the stub host facts.
4. **Complete the accounting (with operator/ctrl):** route the dashboard session-spawn and the docker
   wrapper through `admit`. This is the slice that actually stops the host crash — the 625 GiB becomes
   a bounded, placed, refused-when-full sum.

Slices 1–3 are public `.dag` in this repo; slice 4 touches ctrl (private host facts + the spawn path).

---

## 6. What this fixes (traceable to the postmortem)

- **srv2 hourly livelock:** `admit` refuses the allocation that would cross physical RAM; no more
  unbounded 625 GiB sum.
- **CI floor OOM contribution:** CI draws a bounded class like any consumer; a runaway resolve (Arc 1's
  31 GiB) is *capped and reported as one failed allocation*, not allowed to livelock the host. (It does
  not *prevent* the eager-materialization bug — that's eager-boar's content fix — but it **contains**
  it.)
- **srv1 idle:** `place()` spreads load across both hosts.
- **The 14 GiB constant / probe debate:** deferred — fixed classes sidestep it; probe-sizing slots in
  behind the same interface later.
