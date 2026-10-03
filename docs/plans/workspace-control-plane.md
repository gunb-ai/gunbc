# Workspace control plane: off-rack, serverless, queue-driven

Design for the off-rack control plane of the recoverable workspace. Status: **design only. Nothing here is built, and no store is minted.**

The plan rows it serves live in the private `strategy.year_end_plan`:

- `control-plane-state-store` (its recovery half)
- `volume-durability-contract` (the one global epoch authority)
- `checkpointed-workspace`
- `instance-host-evacuation`
- `instance-reconciliation`
- `instance-endpoint-continuity`

The demo that exercises it is the private `docs/plans/2026-10-02-recoverable-workspace-demo.md`, stages 2–5. The state machine it realizes is the private `docs/plans/2026-09-25-fabric-architecture.md` §2 `resume(instance, target)`. The storage interface it stands on is [fabric-storage](fabric-storage.md), steps 4–5.

Operator rulings taken as inputs, not reopened here:

1. The off-rack control plane is serverless and low-QPS, with a queue.
2. Rack loss is a placement failure: restart elsewhere from the last committed revision.
3. Overflow is ordinary fabric placement by cost. Owned capacity has zero marginal cost and wins; OCI is the probable overflow target; work moves back when owned capacity returns.
4. R2 and B2 are fabric options.
5. CPU and memory are requested independently.
6. The operator is in the US northeast.

Claims are stated as **requirement and confidence**, never as dates. Every external figure carries its standing in the sense of DESIGN §4d (`extdeps.external_authority` `CitedFigureStanding`). A figure marked *transcribed* was copied from the vendor page named beside it, at authoring time, and owes a re-read before any decision is priced on it.

## 1. The state the control plane carries, and its existing homes

The control plane holds five kinds of fact. Every one already has a home in the corpus. The design adds **realizations and placements of those homes, not new vocabulary** (DESIGN §3b conformance: each row below conforms or names its divergence).

| State | Meaning | Home authority | Conformance |
| --- | --- | --- | --- |
| **Intent** | The desired instance: class, CPU and memory requested independently, volume arm, endpoint name, placement constraints. | Requirements: `product.fabric.work` `ExecutionRequirements`. The desired record is a `std.fabric_storage` object under a per-instance head. | Conforms. Intent is an immutable object; "the current intent" is a head. No second desired-state table. |
| **Writer epoch** | Which realization may write the volume head and serve the endpoint. A monotone integer per volume. | `std.temporal_effect` `LeaseEpoch` (`lease_key`, `resource_fingerprint`, `owner_fingerprint`, `generation`). It is advanced by compare-and-set: `std.durable_compare_and_set` `CasExpectation` / `cas_decide`. | Conforms. `generation` is the epoch number, and `owner_fingerprint` names the realization. **Frontier:** the epoch record is a `std.fabric_storage` head (`instance/<id>/epoch`) whose object carries the `LeaseEpoch`. Consumer: the reconciler (§2) and the volume commit path (§3). |
| **Committed revision head** | The last committed checkpoint of the volume: snapshot plus journal chunks. | `std.fabric_storage` `FabricHeadReading` / `FabricHeadAdvance` over content-addressed objects; R2 realization is fabric-storage step 4B (ETag-conditioned put, stacked on gunbc#13080 `gunbc.cloudflare.r2_staged_transfer`). | Conforms. **Divergence, stated:** the head is *namespaced by epoch* (`volume/<id>/e<N>/head`), the fabric-architecture §2 rule "each epoch writes into its own namespace". That is a head-naming policy over the same interface, not a second interface. |
| **Host lease** | Which host holds a realization, until when, and as observed. | `std.temporal_effect` `HeldLease` (`HeldLeaseObservedState`: `LeaseAbsent` / `LeaseRunningExpected` / `LeaseRunningStale` / `LeasePortForeign` / `LeaseInaccessible`). The cell hold is `gunbc.fabric_control_plane` `CellReservation` / `reserve_selected_cell` / `end_cell_hold` over `std.durable_exclusive_hold`. | Conforms. The reconciler's observation *is* `HeldLeaseObservedState`, so detection reuses its verdict fold (`held_lease_observed_verdict`, `plan_for_held_lease_observation`). |
| **Endpoint binding** | Which realization the workspace's stable name routes to, and at which epoch. | No consumed home yet: the private rows `fabric-network-contract` and `instance-endpoint-continuity` own it. | **Modeling obligation (§6), not a conformance row.** This design only requires that the binding carry the epoch it was bound at (§3). Consumer: the overlay's route programming, named in `instance-endpoint-continuity`. |
| **Credentials per epoch** | Scoped storage write credential and overlay identity for one realization. | `std.effect_grant` `Grant` / `Envelope` (verb `Write`, namespace position = the epoch's prefix); minting pattern per `gunbc.auth.authorization_pattern_selection`. | Conforms. A grant is rooted at `volume/<id>/e<N>/`, so `envelope_bounded_by` already makes an epoch-N grant unable to cover the epoch-N+1 prefix. |
| **Placement choice** | Where a realization goes and why. | `product.fabric.selection` `select_supply` → `std.decision` `SelectionReceipt`. | Conforms (§4). |

No parallel store: intent, epochs, heads and leases are all **objects and heads in one `std.fabric_storage` realization**. The control-plane service that §5 chooses is that realization's *linearization point for heads*. It is not a second database beside it.

## 2. The reconciler: level-triggered and idempotent

The reconciler is one pure fold, `reconcile(desired, observed) -> List<Action>`. It is wrapped by one effectful step that reads, decides, applies **at most one CAS-guarded transition**, and returns.

- **Level-triggered.** Its input is the whole current desired state and the whole current observation for one instance (heads plus `HeldLeaseObservedState` plus endpoint binding). It is never the event that woke it. A queue message carries only an instance id ("look at X"), so losing, duplicating or reordering messages costs latency and never correctness. A periodic sweep enqueues every instance, which bounds detection by the sweep period even if every edge-triggered message is lost.
- **Idempotent.** Each action is guarded by the expectation it was computed against:
  - an epoch bump is a CAS from `generation = N` to `N+1`;
  - a placement is a `CellReservation` keyed by `(instance, epoch)`, via `fabric_cell_allocation_key`;
  - an endpoint rebind names its target epoch.

  A retried or duplicated run observes the transition already made and computes `Noop`. This is `ensure` (DESIGN §3d): Noop, Apply or Refuse over a caller-supplied requirement, with independent readback. Selection happens before it, never inside it.
- **One transition per run.** `resume` from fabric-architecture §2 is driven as a sequence of states, not one long transaction:

  `Placed(e) → Fencing(e→e+1) → Fenced(e+1) → Restoring(e+1, target) → Realized(e+1) → Bound(e+1)`

  The current state is a function of the heads, never a flag. Each step is one queue delivery. A worker that dies mid-step leaves the heads unchanged or advanced, and the next delivery continues from what it reads. The receipt chain is `std.temporal_effect` `EffectStepReceipt` / `plan_next_step_from_prior_receipt_and_lease`. Stall detection is `EffectStallBudget`, which escalates as a typed outcome rather than retrying forever (DESIGN §5: refuse, never widen).
- **Detection.** A host lease reads as `LeaseRunningStale` or `LeaseInaccessible` past the lease term. That reading plus the instance's intent "should be running" yields `Fence`. There is no failure detector beyond lease expiry. A partitioned host that still runs is handled by fencing (§3), not by trying to tell "dead" from "unreachable". That is why the demo's harder case, rack alive but unreachable, needs no special arm.

## 3. Epoch fencing: a stale writer can neither commit nor receive traffic

**Requirement:** after the epoch authority records epoch N+1 for a volume, no action by a holder of epoch N reaches durable state or a client. **Confidence:** high for commit; medium for traffic, until the endpoint binding has a home.

A stale writer is held off by four lines. The first is the fence; the rest are defense in depth.

1. **The epoch authority is the fence.** Every commit is a head advance at `volume/<id>/e<N>/head`. A commit is *valid* only if the epoch record still reads `generation = N`. The reconciler bumps the record to N+1 **before** it starts restoring anywhere. Restore reads the last head committed under epoch N, then publishes epoch N+1's first head under `e<N+1>/`. Readers resolve the volume as "the head under the current epoch record", so nothing an epoch-N writer publishes afterward under `e<N>/` is ever read.
2. **Credentials are per epoch.** Epoch N's write grant (`std.effect_grant`, rooted at `e<N>/`) is revoked at the bump. Revocation is best-effort and asynchronous at a provider, so it is never the fence. It only stops a stale writer from spending money and confusing operators.
3. **Commit checks the epoch.** The host's commit path re-reads the epoch record before acknowledging a durable-arm `fsync`. If the record is not N, it refuses and the guest sees an I/O error. The durable arm therefore never acknowledges a write the authority will not read. On the asynchronous workspace arm, writes acknowledged inside the loss window may be lost. That is the arm's published contract, and the demo receipts it.
4. **Traffic follows the binding.** The endpoint binding names `(instance, epoch)`. The overlay routes only to the realization whose identity carries the bound epoch, and the per-epoch overlay credential is revoked like the storage grant. A returning rack (demo stage 5) finds its binding gone and its grants dead. Its local reconciler reads epoch ≠ N and quarantines the instance, then destroys it.

**Could R2 conditional puts alone be the epoch authority?** For a single provider, yes in principle. The epoch record is an R2 object; a bump is a put conditioned on the ETag last read (`If-Match`); creation is `If-None-Match: *`. That is exactly the CAS `std.durable_compare_and_set` needs, and fabric-storage step 4B already models it for heads.

It does not suffice for the architecture, for the reason fabric-architecture §2 already gives: **once B2 or a second R2 bucket can hold the live head, a conditional put at one provider cannot fence a writer at another.** The epoch record must live in *one* place that every provider's commit path consults.

There are two consistent choices:

- **(i) The epoch record is an R2 object, and R2 is that one place.** Every commit, including commits whose chunks land in B2, consults it. This is consistent. Its cost is that R2 is a hard dependency of every commit on every provider, and its availability bounds every volume's. R2 conditional-write semantics also have to be verified against the cited doc, as a strong read-after-write on the conditional path.
- **(ii) A serverless strongly-consistent store holds the epoch record**, with R2 holding objects and per-epoch heads.

Option (i) is the minimal design, and §5 recommends starting there. Option (ii) is the upgrade if the queue and reconciler need transactional state that R2 cannot express in one conditional put.

## 4. Cost-based placement and move-back

Placement is `product.fabric.selection` `select_supply` over a candidate field built from:

- owned cells (`gunbc.fabric_control_plane` `fabric_cell_roster_candidates`) at zero marginal cost;
- overflow offers (OCI Ampere drafted; AWS Graviton the alternative), as priced `CandidateOffer`s.

`ExecutionRequirements` carries CPU and memory as **independent** requirements. A target is admissible when both fit, never by a bundled shape. The result is a `SelectionReceipt`. A different field (rack lost, rack back) is a different decision, so staleness is mechanical (DESIGN §3d).

- **Rack loss** removes every owned candidate from the field, and selection lands on overflow. Same `resume`, a different target adapter.
- **Move-back** is not a special path. The sweep re-runs selection for every running instance. When the receipt's selected candidate differs from the current placement and the cost delta exceeds the move cost (the restore transfer plus the endpoint downtime, as a `delay_valuation`), the reconciler plans a planned `resume` to the cheaper target.
- **Thrash guard:** a move is planned only when the delta covers the move cost over a declared hold horizon (`GrantDuration`). This is a selection policy input, not a timer in the reconciler.
- **Overflow quota** is a candidate's availability observation (`AvailabilityObservation`). An unverified quota is `SelectionNeedsEvidence`, never an assumed admit. This follows the demo's "overflow quota" risk.

## 5. Realization choice

The load is tiny: one record per instance, transitions on failure or move, and a sweep. The figures below decide which choices are admissible, not which is cheapest; at this QPS every option costs a few dollars a month.

### (a) Cloudflare Workers + Durable Objects + Queues, beside R2

- **Consistency:**
  - A Durable Object is a single-threaded actor with transactional, strongly consistent storage. One DO per instance (or per volume) makes every epoch transition serial by construction, and the CAS is a plain read-compare-write inside the object.
  - Location hints (`enam`, eastern North America) place it near the operator.
  - Docs: https://developers.cloudflare.com/durable-objects/ ; https://developers.cloudflare.com/durable-objects/reference/data-location/
- **Queues:**
  - Delivery is at-least-once, which is what a level-triggered reconciler wants.
  - Consumers are Workers, and there are dead-letter queues.
  - Docs: https://developers.cloudflare.com/queues/reference/delivery-guarantees/
- **R2 is the same vendor and the same account.** The credential scope and the step 4B realization are already modeled in `extdeps.cloudflare.r2` / gunbc#13080. There is no egress charge from R2 (https://developers.cloudflare.com/r2/pricing/).
- **Price** (*transcribed*; owes a re-read of https://developers.cloudflare.com/workers/platform/pricing/, https://developers.cloudflare.com/durable-objects/platform/pricing/, https://developers.cloudflare.com/queues/platform/pricing/):
  - Workers Paid is $5/month, which includes Workers, DO and Queues allowances: about 10M Worker requests, 1M DO requests and 1M Queue operations per month.
  - This load sits inside the included tier.
- **Weakness:**
  - A DO's storage lives in one location. A Cloudflare-wide or DO-wide outage takes the control plane down; see §6 for what survives.
  - Single vendor for objects and control. That couples their failures, but either one down already stops commits under option (i).

### (b) AWS Lambda + DynamoDB conditional writes + SQS

- **Consistency:**
  - DynamoDB `PutItem`/`UpdateItem` with a `ConditionExpression` is a linearizable per-item CAS: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Expressions.ConditionExpressions.html
  - Strongly consistent reads within a region. Global tables are multi-region but **last-writer-wins**, so they are *not* a CAS across regions: https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/V2globaltables_HowItWorks.html
  - The fence therefore lives in one region (us-east-1 or us-east-2, for the operator).
- **SQS** standard queues are at-least-once; there is a Lambda event source and DLQs. Docs: https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/standard-queues.html
- **Price** (*transcribed*; owes a re-read of https://aws.amazon.com/dynamodb/pricing/on-demand/, https://aws.amazon.com/lambda/pricing/, https://aws.amazon.com/sqs/pricing/):
  - On-demand writes are on the order of a dollar per million (reduced in late 2024; re-read).
  - Lambda's free tier is 1M requests per month, and SQS's is 1M requests per month.
  - This load is near zero.
- **Weakness:**
  - A second vendor, a second credential pattern (`gunbc.auth` would need an AWS WIF row) and a second extdeps surface to model, for no consistency gain over a DO.
  - Egress from R2 is free, but every control-plane call crosses vendors.
  - Its real advantage, independence from Cloudflare, is worth only what §6 says survives anyway.

### Recommendation

**Confidence:** medium.

- **Phase 1 (minimal):** the epoch record and heads as R2 objects under step-4B conditional puts (option (i) of §3). One Cloudflare Worker is the reconciler, Cloudflare Queues carries "look at X", and a Cron Trigger runs the sweep. No DO and no new store.
- **Phase 2, only if needed:** a Durable Object per instance as the epoch authority (option (ii)). It is needed if R2's conditional-put semantics fail verification as a strong CAS, or if transitions need multi-key atomicity (epoch + binding in one step).

Prefer (a) over (b): it puts the control plane on the same vendor whose object store is already the commit dependency. Its correlated failure adds no new outage mode beyond "R2 down", which already stops commits.

## 6. When the control plane itself is down

**Requirement:** a control-plane outage degrades to "no *changes* of placement", never to data loss, split brain or a silent wrong answer.

| Still works | Stops |
| --- | --- |
| Running instances keep running and serving at their bound endpoint (bindings are programmed into the overlay, not looked up per packet). | Detection and recovery: a host that dies during the outage is not re-placed until the control plane returns. Stated in the guarantee set as a compound outage, never hidden. |
| Commits continue **iff** the epoch record is still readable (option (i): R2 up). The commit check reads the record, not the reconciler. | Option (ii) with the DO down: commit checks cannot read the epoch, so durable-arm commits **refuse** (typed storage outage, the demo's stage-5 arm). They never proceed unchecked. |
| Restore by hand is possible from the last committed head, because heads and objects are plain storage. | Move-back and overflow placement. Cost drift accumulates; correctness does not. |
| Nothing can split brain: no actor except the reconciler bumps epochs, and it is down. | New instance creation. |

The rule is the fabric-storage rule applied one layer up: **an unreachable linearization point refuses, it does not fail over.** There is no "proceed without the fence" arm (DESIGN §5, no escape hatches).

## 7. Consumers and frontier

Every model named here is consumed by a named row; none lands before its consumer:

- **Epoch record over `std.fabric_storage` heads** → `instance-reconciliation`, `volume-durability-contract`.
- **Per-epoch head namespace** → `checkpointed-workspace` (demo stage 1 commit/restore).
- **Reconciler fold over `HeldLeaseObservedState`** → `instance-host-evacuation` (demo stage 2).
- **Epoch-carrying endpoint binding** → `instance-endpoint-continuity` (demo stages 2, 5).
- **Overflow candidates in `select_supply`** → `alternate-target-adapter` (demo stage 4).

## 8. Open decisions (operator)

1. **Realization:** (a) Cloudflare Workers + Queues (+ DO later), recommended; or (b) AWS Lambda + DynamoDB + SQS.
2. **Epoch authority:**
   - option (i), an R2 object under conditional put (minimal; R2 bounds every commit's availability); or
   - option (ii), a Durable Object per instance from the start.

   The answer is gated on verifying R2's conditional put as a strong CAS.
3. **Commit-path epoch check frequency:** on every durable-arm `fsync` (safest, adds one read per flush), or cached with a lease shorter than the fence-to-restore delay (cheaper; a correctness argument is owed).
4. **Sweep period and lease term.** Together they bound detection time, and the demo receipts it. Drafted as same order as the published 60 s window, pending stage 0.
5. **Move-back hold horizon:** how long a cheaper owned slot must be expected to persist before a planned move is worth its downtime.
6. **Second storage provider (B2) timing.** Under option (i), adding B2 for chunks is safe; promoting B2 to hold the *live head* requires the epoch record to stay single-homed, as §3 states.
