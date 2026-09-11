# Where SERVING capacity belongs — a ruling

Question (operator, 2026-09-11, on the serving router: "this is supposed to be fabric work though"):
serving capacity is modeled in `gunbc.spark.*` + `gunbc.harness.*`, compute capacity in
`gunbc.fabric.*` + `gunbc.compute.*`. Both admit work to a scarce resource under a lease. Is that a
§3 fork?

**Verdict: not a fork, and not two unrelated subjects either. It is one shared authority already
consumed by two distinct subjects.** The premise that motivated the question — "neither references
the other for that purpose" — is false at symbol grain. The residual defect is a citation one, not
an architectural one: DESIGN §3b's `fabric / compute` conformance row names two *consumers* as the
domain's home authorities and omits the authorities both subjects actually inhabit.

## 1. The census

### The shared authority (already single)

`gunbc.product.capacity.*` and `gunbc.product.fabric.*` hold the admission concept once, generically:

- `product.capacity.lease` — `LeasePolicy`, `LeaseGrant`, `LeaseFence`, `fence_verdict`,
  `ReleaseLaw`, `capacity_after_expiry`. One lease algebra, no second one anywhere.
- `product.capacity.pool` / `pool_events` / `event_chain` — `Pool<Count, One>`, `SeatRequest`,
  `PoolAcquired`/`PoolReleased`, `PartitionId`, `HeadExpectation`, `ChainEvent`.
- `product.fabric.selection` — `select_supply<P>`, `CandidateOffer<P>`, the priced axes
  (`StartCostObservation`, `TransitionCostObservation`, `OperatingCostObservation`,
  `OpportunityCostObservation`), `OfferSelected` / `NoAdmissibleOffer` / `EqualCostTieUnresolved`,
  affinity-keyed rendezvous. Note the `<P>` — the selection fold is *already* parameterised over the
  provider subject, which is precisely the shape a shared authority takes.
- `product.fabric.work` — `ExecutionRequirements`, `ControlPlaneCapacity`, `TrustDomainRef`.
- `gunbc.fabric_event_log` — `fabric_seat_acquire`, `fabric_seat_observe`, `event_log_append`,
  `event_log_observe_head`. The compare-and-set seat race lives here, in a **fabric** module.

### Subject A — compute cells (`gunbc.fabric_*`, `gunbc.compute.*`)

`gunbc.fabric_control_plane` (`CellReservation`, `reserve_selected_cell`, `partition_cell_roster`,
`CellSlotState`, `end_cell_hold`, `free_cell_slot`), `gunbc.fabric_executor_class`
(`CapacityClassAdmission`, `ExecutorSanction`), `gunbc.fabric_quota` (`QuotaLease` over
`product.capacity.lease`), `gunbc.compute.work_request` (`WorkOperation`, `WorkSubject = ExactTree`,
`WorkOutcome`). Subject identity: an **exact source tree** to be built by some runner slot.

### Subject B — inference serving (`gunbc.harness.*`, `gunbc.serving.*`, `gunbc.spark.*`)

`gunbc.harness.harness_seat` (`harness_bind_seat`, `HarnessAcquisition`, `harness_release_fenced`,
`HarnessReleaseCapability`), `gunbc.serving.turn_admission` (`ServingCoTenancy`,
`ServingSeatOccupancy`, `serving_group_admission`, the cost-axis constructors),
`gunbc.spark.serving_offer` / `pair_serving_*` / `vllm_*`. Subject identity: a **turn** to be served
by a loaded model replica.

### Where they coincide — proven, not by word

`gunbc.harness.harness_seat` imports and calls, on the acquisition path:
`product.fabric.selection { select_supply, CandidateOffer, OfferSelected, NoAdmissibleOffer,
EqualCostTieUnresolved }`, `product.fabric.work { ExecutionRequirements, ControlPlaneCapacity }`,
`product.fabric.supply { EstimatedGrantDuration, AffinityKeyedRendezvous, PlacementAffinityKey }`,
`product.capacity.lease { LeasePolicy, LeaseGrant, GrantIdentity }`, `product.capacity.pool`,
`product.capacity.pool_events`, and `gunbc.fabric_event_log { fabric_seat_acquire }`. Its own comment
says it: *"product.fabric.selection ranks the survivors, and fabric_seat_acquire settles the race."*

So the vocabulary coincidence (lease / grant / release / admission / capacity class / reservation)
is not an alias. It is one set of declarations with two consumers. §2's decompose→map→reduce has
already run on this concept; there is nothing left to unify.

### Where they genuinely differ

1. **Settlement and charging.** `fabric_money_reservation_ref`, `broker_reserve_for_demand`,
   `reserve_priced_cell`, `admit_cell_settlement`, `settle_reserved_cell_at` exist only in the
   control plane. Serving *prices* (it supplies the same `product.fabric.selection` cost axes, with
   `serving_delay_valuation` as a `MoneyPerSecond`) but never reserves or settles money. This is a
   real asymmetry at the **billing** boundary, not at the selection boundary.
2. **Co-tenancy.** `serving_co_tenancy` / `AdmissibleForColdPrefill` /
   `InadmissibleActiveInteractiveTurn` encode a hazard with no compute-cell analogue: a cold prefill
   landing beside a live decode degrades an *already-admitted* tenant. A compute cell's slot is
   binary (held/free); a serving group's seat count is quality-conditioned
   (`harness_interactive_quality_seats`). Verified present and consumed — this is the sharpest
   distinguishing fact and it sits on the serving side of the shared pool, not beside it.
3. **Subject grain and refusal shape.** Compute admission refuses with
   `WorkInfrastructureRefusal`; serving admission refuses with `AdmissibilityUnread` /
   `SeatAcquireRefused`. Different observation boundaries, same lease algebra underneath.

### The "third home for the address fact" — dissolved

`gunbc.fleet.fleet_intent_network` is the host-address authority and is consumed by
`harness_seat`, `harness_backend`, `harness_wire` **and** `gunbc.fabric_event_log`. It is one home
for one fact (where a host answers), not a second home for capacity. An address is not a capacity.

## 2. The three-valued answer (DESIGN §3b)

**Two genuinely distinct subjects sharing one already-single authority.** Not a fork (no second
lease algebra, no second selection fold, no second seat race). Not a bare-alias coincidence (the
sharing is by import and call, not by name).

**Serving belongs INSIDE the `fabric / compute` conformance domain, not beside it.** The domain's
own question already reads "a machine, **a serving head**, a build slot, a budget" — serving is
already in scope by the question's text. What is wrong is the `homes` roster: it names
`gunbc.compute.work_request::WorkOperation`, `gunbc.compute.work_provider_local::ComputeLayout`,
`gunbc.fabric_control_plane::CellReservation`, `gunbc.fabric_executor_class::CapacityClassAdmission`
— four declarations, all of them **Subject A consumers**. A reviewer handed that roster and a
serving diff has nothing to conform it to, and would reasonably conclude serving is a separate
kingdom. That is how this question gets re-litigated.

## 3. Not a fork, so: what makes the subjects distinct, durably

The distinction that survives re-asking is **what the lease is over, and what the resource retains
between grants**:

- A **compute cell** is stateless between grants. Its slot carries no residue; admission is
  binary; the cost of a wrong placement is paid in money and wall-clock, which is why it settles.
- A **serving seat** is over a resource with *loaded state* (weights, KV cache). Admission is
  quality-conditioned because an admitted tenant's latency depends on who else is admitted, and
  there is no settlement because the resource is owned, not rented.

Everything else follows: co-tenancy exists on the serving side because state is retained; charging
exists on the compute side because capacity is rented.

**Where this is recorded so it stops being re-litigated:** the `conformance-compute` domain row in
`gunbc.design_argument`, extended to name the shared authorities and both subjects' entry points.
The homes roster is what a reviewer is handed, it is identity-hashed into the review criterion, and
it is projected into DESIGN.md §3b — so a future "is serving fabric?" is answered by reading the row
the reviewer already reads, not by a prose document. This ruling document is the *argument*; the row
is the *authority*.

## 4. What this ruling does NOT do

No migration is named because none is warranted — there is no X to uproot. Explicitly: the
replacement-migration doctrine does **not** apply here, and a proposal to "move serving into the
fabric control plane" would be a regression, because it would fuse a settling subject with a
non-settling one under one root and force serving to carry money reservation it has no payer for.

## 5. Findings, filed rather than parked

An earlier draft of this section carried two newly-discovered defect classes as prose bullets. That
was itself the failure DESIGN §4b names — "Membership is the directory of those files, not a second
hand-appended list" — and the bullets carried no rung, ceiling or trigger, so §4b(2) was
unsatisfied. They are now filed, and this section cites the symbols rather than restating them.

1. **`FabricGroup` carries three subjects on one cage discriminant.** Filed as a second form of
   `gunbc.recurring_failure_mode.meaning_fork` — the referent varying by *consumer* rather than by
   hidden state — with its invalid state, harm, recognition rule, rung (mitigatable), derived
   ceiling (structurally impossible) and next trigger on the row, and `FabricGroup`,
   `fabric_group_cage` and `harness_seat_partition` as its evidence. It was folded into that
   existing authority rather than minted as a new row, because §3 asks for one home per concept and
   `state_space_conflation` sets the corpus precedent for widening a class with a second form.

   Why it matters here: a sibling lane is enrolling a second serving group and wants
   model-agnostic routing, which is the "second model per cage" case arriving early. The silent arm
   is the dangerous one — a serving group that is not cage-shaped partitions the **seat pool** by
   the wrong key, and two tenants hold a lease each believes is exclusive.

2. **The word "fabric" meaning both compute fabric and network interconnect — withdrawn, not
   filed.** I proposed this as a §3 meaning fork and it does not qualify under that class's own
   scoping rule: one spelling in two explicitly distinct module scopes is legitimate reuse. The
   near-miss is recorded on the `meaning_fork` row so the next reader does not re-propose it. What
   is real at that boundary is finding 1 — a *type* crossing it — not the adjective sitting on both
   sides.

3. **Serving cost axes are asserted, not measured** — the already-rostered declared drop
   `gunbc.rung_drop.serving_cost_axes_asserted_unmeasured`. Named because this ruling leans on
   serving supplying the shared cost axes: it supplies their *shape* honestly and their *values*
   under that drop.
