# Plan — the operator fleet on the compute fabric (umbrella)

**Status:** draft for operator confirmation. **Date:** 2026-06-20.
**Scope:** the strategic umbrella tying together the in-flight threads. Two sub-plans nest under it:
- `docs/plans/compute-fabric-allocator.md` — the allocator invariant (#5390, in progress).
- `ACCESS_MODEL_PLAN.md` (worktree scratch) — visibility/ownership (PR #5415).

Integrated branch `fleet-on-fabric` = the access work (#5415) + the compute-fabric/CI work (#5390),
stacked, as the working base for everything below.

---

## 0. The goal (where we backed up to)

Model **all of the operator's compute — srv1, srv2, the MacBook Air, and future machines — as
first-class hosts on ONE compute fabric**, so that CI, agent sessions, and every other workload are
uniform *consumers* that ask for compute and are *placed* + *allocated* against real, grounded host
facts. Consolidate into the public **gunbc** repo (ditch **ctrl**); keep only operator-private
*values* in ctrl behind `SecretRef` / the visibility boundary; one **`operator_fleet.dag`** authority
that CI, sessions, and access all derive from. **CI is the first and most important consumer.**

## 1. Why (the strategic frame, §1–§3)

- **One authority, derive many.** Today the fleet is scattered: network topology in one file, hardware
  facts in BMC test fixtures, a *synthetic* single CI fleet offer, static runner labels. Each consumer
  (CI, sessions, docker) caps itself independently → the **625 GiB-on-125 GiB overcommit** that crashes
  srv2 hourly (see the allocator postmortem). The fix is one fabric every workload flows through.
- **Reduce convention to physics.** A machine's real CPU/RAM/arch (grounded via BMC/probe) is the
  truth; CI spawn-width, session caps, runner labels all *derive* from it instead of being hand-tuned
  constants.
- **This IS "ditch ctrl, get on gunbc full-time":** the fleet model + CI + control-plane consolidate
  into public gunbc; only operator-private *values* (distro/version, secrets, exact inventory) stay in
  ctrl behind the boundary.

## 2. The architecture (the DAG)

```
operator_fleet.dag   ── THE AUTHORITY (does not exist yet) ──
   { srv1, srv2, macbook, … } each a ComputeHost grounded via BMC/probe
   one source of truth: identity + CPU + RAM + arch + OS surface + network + baseboard
        │
        ▼
product.compute_fabric   ── the model (structurally complete today) ──
   ComputeHost / ComputeOffer (supply) · WorkDemand (demand) · satisfies() (match)
   + AllocationClass + admit(Σ receipts ≤ grantable) + place()      ← allocator (#5390, in progress)
        │   ComputeRequest{class} in  →  AllocationReceipt{host,class} out
        ▼
   derived consumers (all just ask for compute):
     · CI floor      — one request per unit; runner = the PLACED host (not a static label)
     · agent session — dashboard spawn requests before `docker run`  (closes the 625 GiB)
     · docker / ctrl — a wrapper that requests before running
     · access        — visibility/ownership of hosts + code (PR #5415; extends to resources ahead)
```

## 3. Current state (grounded — modeled vs missing)

- ✅ **Fabric structure complete:** `ComputeHost`/`ComputeOffer`/`WorkDemand`/`satisfies()` exist (`dsl/product/compute_fabric.dag`).
- ✅ **Network topology grounded:** srv1/srv2 IPs, BMC endpoints, reachability (`dsl/gunbc/operator_fleet_network.dag`).
- ✅ **Hardware facts available** from the BMC/Redfish seam: 128c/128t Ampere Altra, 128 GiB DDR4, ASRock ALTRAD8UD-1L2T (`dsl/test/claim/bmc_redfish_grounding_witness_test.dag`).
- ✅ **Allocator in progress (#5390):** `AllocationClass` + the `admit` sum-invariant + its own phased plan.
- ❌ **No `operator_fleet.dag` authority** — the "single source everything derives from" is aspirational; only the network file exists.
- ❌ **No per-host `ComputeHost`/`ComputeOffer` for srv1/srv2** — they are network endpoints, not placeable compute; CI uses one *synthetic* `gunbc_ci_fleet_offer` (a 64-thread stub).
- ❌ **CI runner selection is a static label** — `runs-on: [self-hosted, linux, arm64]` baked at YAML-emit, not placement-derived.
- ❌ **Heterogeneity hidden:** srv1 ≠ srv2 (different cgroup MemoryMax) treated as one pool; the MacBook Air (Darwin/arm64) needs `extdeps/cpu/apple.dag` rows that don't exist yet.

## 4. The threads (status)

| Thread | What | Status |
|---|---|---|
| **Allocator** | `admit` Σ≤RAM + fixed classes + `place()` | #5390 in progress — the OOM fix |
| **Fleet authority** | `operator_fleet.dag`: srv1/srv2 as real `ComputeHost`/`Offer`, BMC-grounded | **DONE** (this session) — 3 witnesses green; secrets as handles |
| **CI as placement** | CI emits `ComputeRequest`; runner = placed host, per-host width | depends on allocator (allocator-plan slice 2) |
| **Heterogeneity** | MacBook Air + future machines on the fabric (`extdeps/cpu/apple.dag`) | not started |
| **Access** | visibility/ownership of code (+ ahead: compute resources) | PR #5415 (this session) |
| **Control-plane** | rebuild the ctrl session dashboard as fabric consumers | the original goal (allocator-plan slice 4) |

## 5. Phased plan (bottom-up; nests the allocator plan)

1. **Allocator model** — `AllocationClass` + `admit(host, live, request)` invariant + discriminating witness. *(= allocator-plan slice 1, #5390, in progress.)*
2. **Fleet authority [DONE]** — `dsl/gunbc/operator_fleet.dag`: `srv1` + `srv2` as real `ComputeHost`/`ComputeOffer`, BMC-grounded (128c/128t Altra, 128 GiB), per-host `PlacementSupplyRow` feeding the allocator. **Practical access (skip-ctrl):** rotated BMC creds are `CredentialFlow.Stored` *handles* (never committed); operator-private distro left absent; operator owns the fleet (provider). **Receipt:** 3 witnesses green by execution (real 128-thread grounding; admit grants/refuses on real grantable); discriminating red. *Retires the synthetic offer next (slice 3).*
3. **Ground supply** — allocator's per-host `grantable` = the fleet authority's real `total_memory_bytes − reserves`. *(= allocator-plan slice 3.)*
4. **CI as consumer/placement** — `ci_floor_plan` emits `ComputeRequest{class}`; runner selection = the placed host (per-host labels), width from real per-host facts. *(= allocator-plan slice 2, now multi-host.)*
5. **Heterogeneity** — add `extdeps/cpu/apple.dag` (Apple M-series rows) + model the MacBook Air as a `ComputeHost` (`baseboard: none`, `Darwin` surface, Vm/container isolation). Prove a heterogeneous host on the *same* fabric + `satisfies()`.
6. **Complete the accounting** — route the dashboard session-spawn + a docker wrapper through `admit` (touches **ctrl**: private host facts + the spawn path). *The slice that actually stops the host crash.* *(= allocator-plan slice 4.)*
7. **Control-plane rebuild** — the session dashboard rebuilt as fabric consumers (a session = a `ComputeRequest`, placed + allocated). The original "rebuild ctrl in `.dag`" goal, now grounded on the fabric.

Slices 1–5 + 7 are public `.dag` in gunbc; slice 6 crosses the ctrl boundary.

## 6. Open questions / decisions

- **Consumer→fabric granularity** (allocator-plan open seam): CI one-request-per-unit vs one-per-floor-with-internal-width. Confirm.
- **`operator_fleet.dag` public/private split:** the host EXISTS + its BMC-grounded CPU/RAM/arch is publishable; distro/version + secrets + exact inventory stay ctrl-private (behind the visibility model). Where's the seam drawn?
- **Sequencing:** keystone first — do **slice 2 (fleet authority)** next so the allocator can ground on real per-host facts, or finish the allocator model (slice 1) standalone first?
- **Heterogeneity now or later:** model the MacBook Air early (forces the fabric to be honestly heterogeneous and catches mono-host assumptions), or after srv1/srv2 are solid?
- **Access ↔ fabric:** does the access model (#5415) extend to compute resources (who may *place on* / *own* a host), or stay code-only for now?
