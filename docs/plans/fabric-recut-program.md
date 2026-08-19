# Fabric recut: the carriers survive, the layer underneath is replaced

Successor to [fabric authority census](fabric-concept-reconciliation.md), which found the forks
one at a time. This document is the program that retires them together, and it exists because the
census's own remedy ("repair each fork in place") was refused on review: nine forks in one layer is
not a list of defects, it is the wrong layer.

## 1. The verdict

**#8413 does not land in its current carrier shape.** The six product concepts — `Demand`, `Work`,
`Attempt`, `Offer`, `ExecutionGrant`, `Receipt` — survive. They name fabric-level facts no existing
kernel collectively holds. What does not survive is the layer they are built on.

**What the verdict is NOT supported by.** The review that produced it argued the PR was stale at
`ed2d375` and non-mergeable, "which makes this a natural recut point rather than a reason to
preserve the branch's current layering". That premise is false: `fabric/terminal-contract` is at
`66dc3aab8`, `MERGEABLE`/`CLEAN`, and CI-green — its 44 earlier failures were entirely main's
(compute_board non-exhaustive matches fixed by #8427/#8419; 18 budget rows quarantined by #8457).
The recut is justified by the fork census alone. Recording the rejected support rather than
inheriting it, because a conclusion carried by a false premise decays the moment someone checks it.

## 2. The forks, each verified by execution against branch `66dc3aab8` and main `a2b33f577a`

| fabric carrier | existing authority | verified |
|---|---|---|
| `MaterializationReceipt` | `std.materialization_provider` | both present |
| `LeaseIdentity`, `MoneyLease` | `std.temporal_effect` `HeldLease` | all three present |
| `WorkKey` | `std.computation_identity` `ComputationIdentity` | both present |
| `ResumabilityTier` (3 sites) | `std.effects` + `std.materialization_ladder` | present |
| thread-only `Shape` | `ResourceEnvelope` (7 files on main) | both present |
| `FabricGrantEvidence` | five distinct evidence authorities | 5 fields, one type |
| `FabricIdentity<P,K>` | existing scoped-key relation | **unresolved, see §5** |
| `SettlementReceipt` | encumbrance ledger terminal state | present |
| new market vocabulary | `ComputeNeed` (2 files), `Opportunity` (6), `ComputeOffer` (10) | live on main |

**Corrected in the same review:** current main has no generic `AccessSubjectRef`. `std.access` is
generic over `S`, so the fabric staying generic in `P` is correct and is not a fork. An earlier
census entry claimed otherwise; it was wrong.

## 3. Measured blast radii, not estimated

- **`HeldLease`: 8 files** — the authority, 2 witnesses, and 5 production consumers
  (`codex_app_server_press`, `codex_supervised_turn`, `fleet_converge_plan`,
  `fleet_converge_plan_cli`, `plans/host_effect_orchestration`). Small enough that extracting the
  identity+ownership coordinate is tractable rather than a sweep.
- **Old market vocabulary: 18 file-mentions across three names.** This is the replacement
  population, and it is the one with a live consumer (below).

## 4. The lease decision, signed off: candidate (b)

Three candidates were put to review: (a) embed `HeldLease` whole in `FabricGrantContext`,
(b) extract the identity+ownership coordinate in `std.temporal_effect` and have both consume it,
(c) keep `LeaseIdentity` but add `owner_fingerprint`.

**(b), approved on review.** (c) makes the fork permanent. (a) fixes the fork and creates a fail-open field:
`HeldLease` carries `observed: HeldLeaseObservedState`, a supplied observation rather than a derived
verdict, and embedding it puts a staleness field nobody sets inside an authorization decision that
is immutable at issue time. (b) is the only option where "identity + ownership" becomes a named
thing rather than a field group two carriers each remember to spell. Time, expiry and renewal are
fabric specializations layered on the canonical lease, never a second identity.

**Corrected on review, and the correction matters for the design.** An earlier revision of this
document said `owner_fingerprint`'s absence *caused* the attester hole. It does not. **Lease owner
and receipt attester are independent identities**: the holder authorized to actuate a leased
resource is not necessarily the principal asserting a fact about it. They coincide in the first
vertical and diverge immediately after — a worker attests execution, an artifact publisher attests
retention, a host meter attests usage, a billing observer attests actual spend, a teardown
controller attests absence. Treating the fence as the attester check would bake that coincidence
in. So the lease decomposition and the attester authorization are **parallel repairs, not a
chain**, and the attester question is answered in §7 by `std.access`, never by a branch inside the
epoch comparator.

## 5. Open, and deliberately not decided by the author

1. **Cut order.** This is a replacement migration, so the root goes first and the deletion is the
   census. The old market vocabulary looks like the root, but it has a live consumer:
   `gunbc.runner_spec_from_offer` → `gunbc.witness_floor_workflow`, which **emits our own CI
   workflow**. So the cut cannot open by deleting `ComputeOffer`; a gap-intolerant boundary needs
   the staged form.
2. **Does `StructuralWorkKey` exist at all**, or is it `ComputationIdentity`'s structural member and
   therefore a rename of something already owned? Prefer deletion to renaming.
3. **Six modules or fewer.** The current split has `execution.dag` importing `identity`, `demand`
   and `supply`. If the layer underneath is replaced wholesale, the file boundaries are
   re-litigable at the same time and should be settled once.
4. **Who may attest a receipt.** `admit_receipt_for_grant` never reads `r.id`, whose principal is
   the attesting producer — so any principal can author a `SettlementReceipt` carrying
   `actual_spend` against another's grant. The fix is blocked on a question the fixtures expose
   rather than answer: the fixture grant has `executor: "srv3"` while its admissible receipt is
   attested by `"operator"`, so "authorized attester" is underdetermined between grantee, executor,
   and per-payload. Naming one unilaterally would be a guess wearing a wall's clothing.

## 6. What is already true and must be preserved by any recut

The branch is 16 commits of review-hardened work and was already converging on existing authorities
when it stopped: it consumes `MoneyRate` rather than forking a money-rate carrier, consumes
`BudgetAccountId` rather than minting a second account identifier, and imports `Currency`/`Micro`
after retracting a claim that they were undeclared. The acceptance fold already compares the whole
lease identity rather than the generation alone, compares the attempt as a full `FabricIdentity`
so a cross-tenant same-named attempt refuses, and refuses rather than reports difference on a
cross-family fingerprint comparison. **The recut inherits these; it does not restart from the
first commit.** A minimum replacement that loses any of these refusals has erased a correctness
distinction rather than completed a migration.


## 7. The signed-off construction

Reviewed and approved, with the named carrier being `LeaseEpoch` rather than `LeaseIdentity`,
because it includes the generation and so is an epoch rather than an identity.

```
type LeaseEpoch { lease_key: LeaseKey, resource_fingerprint: ContentHash,
                  owner_fingerprint: ContentHash, generation: Int }
type HeldLease  { epoch: LeaseEpoch, observed: HeldLeaseObservedState }
```

`LeaseEpoch` is *what was issued and what fences action*; `HeldLease` is *what was observed about
that issued epoch*. `ExecutionGrant`, `Receipt` and the reservation binding carry `LeaseEpoch` and
never `HeldLease`; only the observer and reconcile paths carry `HeldLease`. Upstream helpers
(`held_lease`, `held_lease_with_observed`, `held_lease_epoch`) exist so that updating an
observation cannot silently drop an ownership coordinate — `codex_supervised_turn` today rebuilds
all five fields merely to change `observed`, which is exactly the accident the helpers remove.

**The fence compares all epoch coordinates**, and because both fingerprints are `ContentHash`, each
comparison needs the cross-family incomparability result rather than Boolean equality — so
`BoundToDifferentOwner` and `OwnerFingerprintIncomparable` are distinct outcomes, and the order is
attempt → lease key → resource fingerprint → owner fingerprint → generation.

**`MoneyLease` is not a lease** — no leased resource, no owner, no lease key, no liveness, no
observation. It is a reservation-to-lease binding whose generation-only fence is insufficient
(two unrelated leases sit at generation 3). It becomes `ReservationLeaseBinding<R>` carrying the
complete `LeaseEpoch`, **derived from the admitted grant** rather than from independently supplied
fields, so a refused reserve records nothing. Renaming it to `ReservationFence` would keep the
partial fork and is explicitly rejected.

**Attester authorization is `std.access`**, projected as a receipt-submission request, with
`ReceiptFromUnauthorizedAttester` as the located projection of an access refusal rather than a
branch in the lease kernel. The lease comparison is extracted as `admit_grant_bound_execution`
over a `GrantBoundExecutionRef`, keeping fencing and authorization separable.

**The authority envelope is `std.effect_grant` `Envelope`**, which already owns effect reach over
namespace positions and already provides child attenuation via `envelope_bounded_by` (both verified
present on main). `product.fabric` duplicates no `Verb`, namespace position, or child-boundedness.
There remains no `FabricGrantAllowed | FabricGrantDenied` sum: `std.access.AccessDecision` is the
only allow/deny result, and a `Permit` is what mints the positive grant.

**Entitlement, per the operator ruling, gives a carrier boundary:** the `ExecutionGrant` carries
entitlement (vCPU, RAM, storage, network entitlement, isolation/authority envelope); the `Receipt`
carries measured performance and variance. The grant does not promise "completes in five minutes"
or "within 20% of dedicated" until a measured, priced variance-bound execution class exists.
Instrumented variance is evidence about an Offer and its realization, not part of authorization.

## 8. What survives that is NOT a fork

`dag/extdeps/accounting/encumbrance.dag` is new on this branch and absent from main — a generic,
cited accounting kernel with `settle_encumbrance` / `release_encumbrance` over `<Q, S>`. It is
endorsed unchanged. The recut is of the layer *beneath* the six concepts, and this kernel is not
part of it; the product binding owns the temporal relation and then invokes the kernel.

## 9. The fourteen controls that gate merge

Reviewer's conditions, recorded verbatim in substance so the acceptance bar is not renegotiated later:

1. `HeldLease` is exactly `LeaseEpoch + HeldLeaseObservedState`; no other carrier repeats the four epoch fields.
2. Changing `observed` preserves the epoch exactly.
3. `ExecutionGrant`, `Receipt` and reservation binding carry `LeaseEpoch`, never `HeldLease`.
4. Same generation and resource, different owner fingerprint → refuses.
5. Cross-family owner fingerprints → incomparability refusal.
6. A foreign receipt attester with otherwise perfect attempt and epoch coordinates → denied by `std.access`.
7. An attester different from the lease owner **succeeds** when explicitly authorized.
8. The lease owner **cannot** attest a result it was not authorized to attest.
9. Money settlement with correct generation but different lease key, resource or owner → refuses.
10. A refused money reservation writes no reservation binding.
11. Successful settlement or release advances the encumbrance and does not mutate the canonical epoch.
12. Existing `fleet_converge_plan`, `codex_supervised_turn`, `codex_app_server_press`, the fleet CLI and the temporal-effect witnesses continue to pass after the upstream extraction.
13. No direct `ExecutionGrant` constructor path bypasses an admitting `std.access.Permit`.
14. Child effect envelopes cannot exceed the parent grant envelope.

Controls 7 and 8 are the pair that matters most and neither is obvious: they are what stop the
implementation from silently re-equating lease owner with receipt attester after §4's correction.
Testing only "foreign attester refuses" would pass under exactly the conflation being removed.
