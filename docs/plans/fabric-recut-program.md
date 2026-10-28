# Fabric recut

Replace the duplicated layer beneath `Demand`, `Work`, `Attempt`, `Offer`, `ExecutionGrant` and `Receipt`, preserving those product concepts and their hardened refusals. The fork census justified this program; the claim that #8413's branch was stale/non-mergeable did not. [Original measured bases, review rulings and correction receipts](https://github.com/gunb-ai/gunbc/blob/3e5a0f1f0dbc62e0d3589df44146858d3147c9e9/docs/plans/fabric-recut-program.md) records that evidence and obsolete cut inventories. This plan states the resulting construction; historical branch status is not present merge authority.

## Existing authorities and preservation bar

Reuse `std.materialization_provider`, `std.temporal_effect`, `std.effects`, `std.materialization_ladder`, resource envelopes, scoped-key relations and the encumbrance kernel rather than fabric-local copies. `std.access` is generic over its subject; no generic `AccessSubjectRef` existed in the measured tree. Fabric remaining generic in principal `P` is correct.

Preserve `MoneyRate`, `BudgetAccountId`, `Currency`/`Micro`, whole lease-identity comparison, full principal-scoped attempt identity, and cross-family fingerprint incomparability. The recut inherits reviewed behavior; it cannot trade a required refusal for a smaller replacement. `extdeps.accounting.encumbrance` (`settle_encumbrance` / `release_encumbrance` over `<Q,S>`) survives: product binding owns its temporal relation.

## Canonical lease, grant and receipt

The signed construction is:

```
LeaseEpoch { lease_key: NonEmptyStr, resource_fingerprint: ContentHash,
             owner_fingerprint: ContentHash, generation: Int }
HeldLease { epoch: LeaseEpoch, observed: HeldLeaseObservedState }
```

`LeaseEpoch` is the issued fence; `HeldLease` adds supplied observation. Grant, Receipt and reservation binding carry the epoch, never observation-bearing `HeldLease`. Updating observation preserves the entire epoch through upstream helpers. Do not mint an unvalidated `LeaseKey` alias: no grounded common syntax existed; a future branded key requires its validating mint.

Compare attempt → lease key → resource fingerprint → owner fingerprint → generation. `BoundToDifferentOwner` and `OwnerFingerprintIncomparable` are distinct. Time/expiry/renewal specialize the canonical lease, not its identity.

`MoneyLease` is not a lease: replace it with `ReservationLeaseBinding<R>` containing the **complete epoch derived from an admitted grant**. A refused reserve records nothing. Generation-only fencing or a `ReservationFence` rename leaves the fork.

Lease owner and receipt attester are independent principals. Receipt submission is an `std.access` request; its refusal projects `ReceiptFromUnauthorizedAttester`. The grant-bound fence is independently extracted as `admit_grant_bound_execution` over `GrantBoundExecutionRef`. Which attesters may submit each payload must be declared by the access profile, never inferred from a fixture's executor/owner coincidence or embedded in the epoch comparator.

`std.effect_grant.Envelope` owns reach and child attenuation (`envelope_bounded_by`); fabric duplicates no Verb, namespace position or child-boundedness. `std.access.AccessDecision` is the sole allow/deny result; only its `Permit` mints a positive grant.

A grant promises entitlement (vCPU/RAM/storage/network/isolation/authority), not runtime or variance. Receipts carry measured performance. A variance promise requires a future measured, priced execution class.

### Fourteen acceptance controls

1. `HeldLease` is exactly epoch + observed state; no other carrier repeats the four epoch fields.
2. Observation changes preserve epoch exactly.
3. Grant, Receipt and reservation binding contain epoch, never `HeldLease`.
4. Same generation/resource, different owner refuses.
5. Cross-family owner fingerprints refuse as incomparable.
6. Foreign unauthorized attester refuses despite perfect attempt/epoch.
7. An authorized attester different from owner succeeds.
8. Owner without attestation authorization refuses.
9. Settlement with correct generation but different lease/resource/owner refuses.
10. Refused reserve writes no binding.
11. Settlement/release advances encumbrance without mutating epoch.
12. Existing temporal-effect witnesses and `fleet_converge_plan`, `codex_supervised_turn`, `codex_app_server_press` and fleet CLI consumers pass the extraction.
13. No direct Grant constructor bypasses an admitting Permit.
14. Child envelopes cannot exceed parent.

## Identity and module ownership

`WorkContentKey { digest: ContentHash }` is an address for one canonical work contract, derived as contract → content hash → key, never caller-supplied. `ComputationIdentity.StructurallyIdentical` is a comparison qualification, not an address. Do not conflate them or substitute an unenforced branded serialization string. The work key and materialization request key are peers at different grains.

Use an acyclic ownership graph:

| Module | Owns | Forbidden dependency |
|---|---|---|
| `product.fabric.work` | Work, derived content key, work identity/tag and canonical constructor | scheduling quantities, suppliers, Demand |
| `product.fabric.budget` | BudgetAccountId, MoneyAccount, MoneyReservationRef, generic binding, currency wall, encumbrance projections | Demand, Offer, ExecutionGrant |
| `product.fabric.demand` | Demand/identity, admission/buy/satisfaction/delivery terms, commands/access profiles | — |
| `product.fabric.supply` | Offer/identity, quote, independently typed observed/quoted evidence, availability, accepted revision | demand merely to borrow a receipt string |
| `product.fabric.execution` | Attempt/lineage, Grant/access profile, accepted offer revision, atomic reservation bundle, epoch, envelope and fencing | — |
| `product.fabric.receipt` | generic Receipt<P,R>/identity, submission profile, authorization and result-independent acceptance | a closed payload coproduct |

Delete central `product.fabric.identity`: each owner defines its key/tag/principal specialization through `std.key_relation.ScopedKey`, not a freely constructible `(principal,key)` copy. Narrow provider imports follow these homes; adding a supplier or result must not edit unrelated demand or grant authorities.

## Replacement order and corrected boundaries

**A — canonical lease epoch first.** Migrate all live `HeldLease` consumers before downstream fencing builds on a temporary shape.

**B — retire `product.compute_fabric`.** Its surviving `Shape` and `HardRequirements` move to `product.fabric.work`; supply already imports work, so this adds no cycle. Delete the remainder and subject-only witnesses, classify compile refusals, repair stale prose, and preserve the thread-floor coverage named by `witness_hard_requirement_unmet_is_unmet` / the #5904 dissolution receipt. The original “only string references” census was false: unqualified witness calls were missed, and #8413 subsequently added real imports. Count qualified names, bare imports/calls and namespace projections. Conversely the fabricated `product.compute_fabric.Endpoint` in `grounding_lens` is a controlled namespace fixture, not a dangling real reference.

Do not expand thread-only `Shape` into an incomplete resource envelope merely to compile the cut. Future terminal resource references require their own grounded owners; the previously suggested `ExecutionClassRef` / `ResourceSupplyRef` did not exist at that measured base and are not permission to mint them inside deletion.

**C — close the CI workflow bypass.** `gunbc.ci.ci_runner_target` already owns runner-family selection and spec/memory projections. Point the CI emitter at `gunbc_ci_selected_runner_spec()`, correct its stale `ci_runner_target_mapping_note`, and preserve the physical placement consumers. `FleetSelfHosted` delegating to fleet facts is deliberate; the fault was the emitter bypassing the existing seam.

C requires byte-identical `witnesses.yml`, no **CI** production emitter calling `gunbc_ci_runner_spec()` directly, the selected-target witness green, and old direct projection marked for deletion with D. **Fleet convergence is excluded:** it must remain on the fleet when CI selection changes to a cloud runner. `fleet_converge_workflow` has separate host-pinned semantics and must not consume the CI selection merely because their current values match.

**D — decompose live `ComputeOffer`, then delete it.** This is consumer classification, not a type substitution. Terminal Offer has commercial terms, not physical host/OS/performance fields. Runner labels describe an **execution class/environment**, not a bare host or an offer. Reject a concrete-host field on `FleetSelfHosted`; an execution-class reference is the recommended eventual seam. A temporary gunbc projection may derive environment from present host authorities but cannot become permanent `runner_spec_from_host` product authority.

Provider-owned backing relates offers to realization independently:

```
OwnedSupplyBacking { id, physical: ComputeHost, execution: ExecutionSurface }
OfferBacking<P> { offer, offer_revision: ContentHash, backing }
```

Offer never enumerates backing kinds. Terms derive from or are proven against backing; changing backing preserves public execution class. Many-to-many backing and quoted fulfillment templates remain representable. Publication substantiates advertised shape/capability/trust/availability/quote; grant binds accepted offer identity+revision, exact reservation/executor/fulfillment; readback proves the materialized resource meets accepted obligations or refuses. Do not erase this relation when removing the wrong label consumer. Derive remaining capacity from grants, never store a competing remainder.

Classify every old consumer by actual field read: market → Offer; physical → backing/host (`ci_runner_placement`); environment → execution class (`runner_spec_from_offer`); performance → realization observations; settlement → accepted offer revision + grant + supplier billing evidence. Performance observations are append-only and keyed to exact realization/work-or-class/receipt, with derived profiles. The old measured population was empty and unconsumed; that does not authorize dropping future observation. Collect from day one; only an admitted variance-bound class makes it a contractual/fungibility fact.

D1 extracts backing and performance populations; D2 projects Offer from each owned backing (explicit zero quote, typed evidence, resource/capability profile, supplier/executor) with a relation witness; D3 migrates all five consumer classes; D4 deletes fused types, old fleet offers and obsolete projection, plus `CostEstimate` only if fully subsumed. No compatibility record survives. **D4 may delete `runner_spec_from_offer` only after fleet convergence has its own fleet-derived runner authority**: C deliberately leaves that consumer outside CI selection. Creating this authority is not C's scope.

**E — recut modules after supply is singular**, then build the first real Demand → eligible Offers → fungibility → affordability → selection → atomic reservations → Grant transition. Never land independently authoritative old/new supply carriers together. The historical #8413 plan preserved reviewed work and merged separately reviewable prerequisites into its branch; it is not authority to claim that branch remains open today.

## Validation instruments

Test the fourteen controls and each cut's production consumer, not only its new carrier. A declaration-name grep is insufficient reachability evidence; explicit fake fixtures must remain distinguishable from references.

Invoke an interpreter built from the evaluated tree by explicit path (`./target/release/gunbc`), in the same execution environment. The measured baked `/usr/local/bin/gunbc` resolved one source and produced plausible missing-type diagnostics while the tree-built binary evaluated the same witness. Bare `gunbc` inside a checkout reached that baked binary through a shim. Prove which binary ran; a pinned-binary red/green is not evidence about the change. Preserve refusal controls and byte-identical C output before claiming closure.
