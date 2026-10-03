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
| **Intent** | The desired instance: class, CPU and memory requested independently, volume quota, endpoint name, placement constraints. | Requirements: `product.fabric.work` `ExecutionRequirements`. The desired record is a `std.fabric_storage` object under a per-instance head. | Conforms. Intent is an immutable object; "the current intent" is a head. No second desired-state table. |
| **Writer epoch** | Which realization may advance the workspace head and serve the endpoint. A monotone integer per workspace. | `std.temporal_effect` `LeaseEpoch` (`generation` = epoch, `owner_fingerprint` = realization). **The epoch is not a separate record:** it is carried by the `WorkspaceFence` entries on the workspace head, in `std.checkpointed_workspace` (still-dove-673, branch `workspace-revision-model`). | Conforms. There is no second epoch store. Consumer: `std.checkpointed_workspace` `workspace_commit_classify` (§3). |
| **Committed revision head** | The last committed checkpoint of the workspace, as a chain of `WorkspaceRevision` and `WorkspaceFence` entries. | `std.fabric_storage` `FabricHeadReading` / `FabricHeadAdvance`, as **one head per workspace**, `workspace.<id>.committed`. Its R2 realization is fabric-storage step 4B (ETag-conditioned put), stacked on gunbc#13080 `gunbc.cloudflare.r2_staged_transfer`. | Conforms. No per-epoch head namespace: a per-epoch namespace would make a stale commit unreadable but still *acknowledged* (§3). |
| **Host lease** | Which host holds a realization, until when, and as observed. | `std.temporal_effect` `HeldLease` (`HeldLeaseObservedState`: `LeaseAbsent` / `LeaseRunningExpected` / `LeaseRunningStale` / `LeasePortForeign` / `LeaseInaccessible`). The cell hold is `gunbc.fabric_control_plane` `CellReservation` / `reserve_selected_cell` / `end_cell_hold` over `std.durable_exclusive_hold`. | Conforms. The reconciler's observation *is* `HeldLeaseObservedState`, so detection reuses its verdict fold (`held_lease_observed_verdict`, `plan_for_held_lease_observation`). |
| **Endpoint binding** | Which realization the workspace's stable name routes to, and at which epoch. | No consumed home yet: the private rows `fabric-network-contract` and `instance-endpoint-continuity` own it. | **Modeling obligation (§6), not a conformance row.** This design only requires that the binding carry the epoch it was bound at (§3). Consumer: the overlay's route programming, named in `instance-endpoint-continuity`. |
| **Credentials per epoch** | Scoped storage write credential and overlay identity for one realization. | `std.effect_grant` `Grant` / `Envelope` (verb `Write`, namespace position = the epoch's chunk prefix). The minting pattern follows `gunbc.auth.authorization_pattern_selection`. | Conforms. **This is a cost and blast-radius control, not the fence.** Chunk writes go under `workspace/<id>/e<N>/`, so a revoked epoch-N grant stops a stale writer from spending money. Correctness never depends on revocation. |
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

**The fence is the same compare-and-set as the commit.** There is no separate epoch record read before the head advance. With "read record == N, then advance `e<N>/head`", a writer fenced *between* the two steps commits after the bump. If restore had already read that head, the stale commit returns success to the guest, an acknowledged barrier, and is then silently lost. A per-epoch namespace makes such a commit unreadable, not unacknowledged, and that breaks the contract's strongest promise.

The model, `std.checkpointed_workspace` (still-dove-673):

1. **One head per workspace:** `workspace.<id>.committed`.
2. **A fence is a head entry.** An epoch transition is the reconciler advancing that head to a `WorkspaceFence { epoch: N+1, parent: <last revision> }`. It does so through the same `FabricHeadAdvance` every writer uses, **before** any restore starts.
3. **Every commit expects the entry it built on.** A writer at epoch N advances with `ExpectHeadAt { object: <entry it built on> }`. If a fence landed in between, the advance is `FabricHeadMoved`. `workspace_commit_classify` classifies that as `WorkspaceCommitStaleEpoch` (observed entry epoch > writer epoch), and the guest never sees success. The fence and the commit linearize on one compare-and-set, so the gap no longer exists.
4. **Attach** reads the head and refuses unless the newest fence names this realization's epoch.
5. **Restore** walks the chain from the head past any fences to the last `WorkspaceRevision`, then advances from there under the new epoch.
6. **Coverage invariant.** The contract (private `volume-durability-contract`) sells one arm, the asynchronous checkpointed workspace: guest `fsync` is acknowledged locally, and durability is the manifest commit. `oldest_uncommitted_write_age` stays under the published window. A commit refused for unreachable storage is not retried silently: once the oldest uncommitted write would exceed the window, the instance enters the **declared storage outage** and refuses writes. An idle workspace is never refused. A commit refused as `WorkspaceCommitStaleEpoch` makes the realization quarantine itself.

Two defense-in-depth lines stand beside the fence; neither is the fence:

- **Per-epoch credentials** (state table): revocation is best-effort at a provider and bounds cost and blast radius only.
- **Traffic follows the binding.** The endpoint binding names `(instance, epoch)`, and the overlay routes only to the realization carrying the bound epoch. Its per-epoch overlay credential is revoked at the fence. A returning rack (demo stage 5) finds its binding gone, its grants dead, and its next commit `WorkspaceCommitStaleEpoch`; it quarantines, and the instance is destroyed.

**Can R2 conditional puts alone be the epoch authority? Yes.** The epoch authority *is* the workspace head, so R2 `If-Match` on that one object is the only linearization needed: commits and fences both use it, and creation is `If-None-Match: *`. Multi-provider placement does not break this, provided **B2, or any second backend, holds chunks only**, which are immutable and replicate freely. **The head stays in one place.** The fabric-architecture §2 hazard, a partitioned writer advancing a second provider's own head, arises only if a second provider holds a live head, so that is refused by design rather than fenced.

The residual requirement is to verify R2's conditional put as a strong read-after-write CAS against the cited doc (https://developers.cloudflare.com/r2/api/s3/extensions/ and the conditional-operations reference). Confidence is medium until verified.

A Durable Object is not needed for fencing. It stays as the fallback **only** if that verification fails, or if one transition must change the head and the endpoint binding atomically.

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
  - Single vendor for objects and control. That couples their failures, but either one down already stops commits, because the head lives in R2.

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

- **Phase 1 (minimal):** one head per workspace as an R2 object under step-4B conditional puts, carrying commits and fences alike (§3). One Cloudflare Worker is the reconciler, Cloudflare Queues carries "look at X", and a Cron Trigger runs the sweep. No DO and no new store.
- **Phase 2, only if needed:** a Durable Object per workspace holding the head. It is needed if R2's conditional-put semantics fail verification as a strong CAS, or if transitions need multi-key atomicity (epoch + binding in one step).

Prefer (a) over (b): it puts the control plane on the same vendor whose object store is already the commit dependency. Its correlated failure adds no new outage mode beyond "R2 down", which already stops commits.

## 6. When the control plane itself is down

**Requirement:** a control-plane outage degrades to "no *changes* of placement", never to data loss, split brain or a silent wrong answer.

| Still works | Stops |
| --- | --- |
| Running instances keep running and serving at their bound endpoint (bindings are programmed into the overlay, not looked up per packet). | Detection and recovery: a host that dies during the outage is not re-placed until the control plane returns. Stated in the guarantee set as a compound outage, never hidden. |
| Manifest commits continue **iff** R2 is up: the commit *is* the head compare-and-set and needs no reconciler. | With R2 unreachable, manifest commits refuse. `oldest_uncommitted_write_age` then grows, and at the window the instance enters the declared storage outage. Commits never proceed unchecked. |
| — | A host lost during that outage loses its pending writes. That is receipted as the contract's typed **compound failure** (sole volatile copy lost while coverage could not be kept), never as an ordinary recovery. |
| Restore by hand is possible from the last committed head, because heads and objects are plain storage. | Move-back and overflow placement. Cost drift accumulates; correctness does not. |
| Nothing can split brain: no actor except the reconciler bumps epochs, and it is down. | New instance creation. |

The rule is the fabric-storage rule applied one layer up: **an unreachable linearization point refuses, it does not fail over.** There is no "proceed without the fence" arm (DESIGN §5, no escape hatches).

## 7. Consumers and frontier

Every model named here is consumed by a named row; none lands before its consumer:

- **Fence-as-head-entry and stale-commit classification:** `std.checkpointed_workspace` `workspace_commit_classify` (still-dove-673). Rows: `checkpointed-workspace`, `instance-reconciliation`, `volume-durability-contract` (demo stages 1 and 5).
- **Per-epoch chunk credential prefix:** the storage grant minting under `instance-reconciliation`.
- **Reconciler fold over `HeldLeaseObservedState`** → `instance-host-evacuation` (demo stage 2).
- **Epoch-carrying endpoint binding** → `instance-endpoint-continuity` (demo stages 2, 5).
- **Overflow candidates in `select_supply`** → `alternate-target-adapter` (demo stage 4).

## 8. Open decisions (operator)

1. **Realization:** (a) Cloudflare Workers + Queues (+ DO later), recommended; or (b) AWS Lambda + DynamoDB + SQS.
2. **Epoch authority.** The recommendation is the workspace head itself, on R2 `If-Match`. This is minimal, but R2's availability bounds every commit. The alternative is a Durable Object holding the head. Gated on verifying R2's conditional put as a strong CAS.
3. **Where the epoch is checked:** settled by the model, not open. Fences and commits are the one head CAS; attach reads the head. Nothing is checked on guest `fsync`. (Kept as a numbered item so the list keeps its numbering.)
4. **Sweep period and lease term.** Together they bound detection time, and the demo receipts it. Drafted as same order as the published 60 s window, pending stage 0.
5. **Move-back hold horizon:** how long a cheaper owned slot must be expected to persist before a planned move is worth its downtime.
6. **Second storage provider (B2) timing.** B2 can join as a chunk replica at any time. It never holds a live head, so promotion means moving the one head, which is a planned operator action and not a failover.
