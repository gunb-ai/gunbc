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

- **`HeldLease`: 10 files** at main `8687f2a6a30` — the authority, 4 witnesses, and 5 production
  consumers. (This read 8 when written against an earlier base; two witness consumers landed
  between. Corrected by Cut A's re-measurement rather than left to float — a census with no base
  named is not a measurement.)
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

1. *(Resolved — see §10. There are two roots, not one.)*
2. *(Resolved — see §11. The carrier survives; deletion would have been wrong.)*
3. *(Resolved — see §12. Six modules, but not the present six.)*
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
type LeaseEpoch { lease_key: NonEmptyStr, resource_fingerprint: ContentHash,
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


## 10. Cut order, signed off: two roots, not one

My framing of "the old market vocabulary" as a single root was wrong, and bundling it with the
workflow consumer would have combined a dead root, a live supply authority and a wrong
product-interface dependency into one migration.

**X1 — `product.compute_fabric` is dead.** `Fabric`, `Shape`, `Program`, `ComputeNeed`,
`Opportunity`, `Connection = Bound | Pending | Unmet`, `connect`. Verified: its only non-plan-doc
mentions on main are **inside string literals** — `gunbc.host_standup` names
`product.compute_fabric.connect` in a `refusal:` message and an `authority_or_interim:` field,
which is prose, not a code dependency. So there is no production *decision* consumer and it is
deleted outright, needing no terminal market Y first — only that #8413 stop importing its `Shape`.

**The census, re-measured twice and corrected twice. The version before this one asserted a
structural fact that was false, and the method that produced it is the finding.**

*What I claimed:* every reference to `product.compute_fabric` is a string literal, so the deletion
refuses nothing and "the deletion is the census" does not apply here.

*What is true:* `dag/test/claim/host_standup_spine_witness_test.dag` declares
`compute_fabric_shape_authority_is_referenced_not_redeclared`, whose body is a direct call to the
module's own `witness_fabric_binds_need_to_opportunity()`, and
`compute_fabric_resource_witness_test.dag` calls it too. Both predate the base I measured. The
deletion **does** refuse, and the doctrine's guarantee partly holds.

*Why the method failed, which matters more than the claim:* I grepped the **module name**. That
witness calls a **bare function name** with no qualifier, so no amount of grepping for
`compute_fabric` could ever have found it. A `.dag` reference takes at least three forms —
qualified name, bare name, namespace projection — and checking only the first has cleared live
files wrongly before. The defect is not that I missed a file; it is that a single-form grep cannot
answer the question I asked of it, and I reported its output as a structural fact.

**#8413 merged to main as `08b6f7ea4d` while Cut B was in flight**, which adds three *real* imports
and changes the cut's shape:

```
dag/product/fabric/supply.dag:6                              import product.compute_fabric { Shape }
dag/product/fabric/work.dag:5                                import product.compute_fabric { Shape }
dag/test/claim/fabric_terminal_contract_witness_test.dag:7   import product.compute_fabric { Shape, HardRequirements }
```

**The resolution: `Shape` and `HardRequirements` move into `product.fabric.work`, and everything
else in `product.compute_fabric` is deleted.** This is the doctrine rather than a preference — the
dead module's only surviving load-bearing content *is* `Shape`, and a replacement migration moves
the surviving authority to its rightful owner instead of hunting for a substitute so the old file
can die with its contents intact. `Shape` belongs to work: it is what a `WorkSpec` requires and
what an `Offer` offers, and `work.dag` already destructures it in `shape_material` behind a
deliberate compile wall on axis growth.

The structural check that makes it free: **`supply.dag` already imports `product.fabric.work`**
(verified internal graph: `work <- identity`; `supply <- identity, demand, work`). No new edge, no
cycle. It also moves *toward* §12's layout, which assigns work the work-side vocabulary — and it
mints nothing and evolves nothing, so neither `ExecutionClassRef` nor `ResourceSupplyRef` is
needed. Those two were named in an earlier brief as the available alternatives; **neither exists on
main**, and naming unavailable carriers as the sanctioned path is how a brief pushes an
implementer toward minting one inside a deletion.

*The rest of the census:* five witnesses inside the production module; a separate witness file;
two `gunbc.host_standup` prose strings that become false on deletion; five plan documents, several
citing types the module no longer has (`WorkDemand`, `ParallelismShape`, `ResourceEnvelope`) which
its own `compute_fabric_resource_model_dissolution_receipt` records as dissolved in #5904; and
DESIGN §2, fixed separately in #8559. One coverage obligation must survive the cut rather than die
with it: that receipt names `witness_hard_requirement_unmet_is_unmet` as the thread-floor coverage
for the #5904 dissolution, so deleting the witness *and* the receipt recording it would retire a
coverage claim with nobody deciding to.

**Correction, kept rather than deleted — the `Endpoint` claim was also wrong.** An earlier version
recorded `grounding_lens_test.dag` as citing a non-existent `product.compute_fabric.Endpoint`, "a
fabricated citation, the §3 class". It is a **controlled fixture**: `product_namespace_fixture_decls()`
hand-builds two `concept_decl` rows to test that a `product.*` namespace is not layer-excluded, and
its sibling names `product.network_topology.Url`, which also does not exist. The test needs a
string *shaped like* a product qualified name, not one that resolves. Two false findings in one
section, both stated confidently, both about whether a string was a reference — recorded because a
plan that silently drops its own false claims teaches nothing.

**Do not evolve `Shape`.** Do not grow it into memory, storage, topology, network or isolation to
make anything compile. At the terminal-carrier grain use `ExecutionClassRef` / `ResourceSupplyRef`
until the resource-envelope replacement lands; putting a new incomplete resource record into #8413
merely to make the deletion compile is the scaffold arm.

**X2 — `gunbc.ci_fleet.ComputeOffer` is live**, and is the real replacement migration.

**The workflow edge is itself wrong**, which is the finding that reorders everything:

```
ComputeOffer -> runner_spec_from_offer -> RunnerSpec -> witnesses.yml
```

`runner_spec_from_offer` derives GitHub labels from the *physical host*, so the customer-facing
execution class currently depends on physical supply — every supply migration is a workflow
migration. `gunbc.ci_runner_target` already claims to be the single "which machine runs CI"
authority and is a smaller interception point (verified present, with consumers in
`ci_budget_tree` and `fleet_workflow_steps`), though it still delegates its fleet arm back to
`runner_spec_from_offer`.

### The order

- **Cut A — canonical lease epoch.** Land `LeaseEpoch` in `std.temporal_effect` and migrate all
  `HeldLease` consumers (10 files at `8687f2a6a30`). Upstream authority preparation, first, because Grant, Receipt,
  settlement, teardown and component fencing would otherwise all build against a temporary carrier.
- **Cut B — delete the dead root.** Delete `product.compute_fabric`, remove #8413's dependency on
  its thread-only `Shape`, drop the subject-only witnesses, and classify the compile refusals.
- **Cut C — point the live workflow emitter at the seam that already exists.** *(Rewritten after
  measurement. The original specification of this cut was premised on an edge that is not there.)*
- **Cut D — replace live `ComputeOffer`.** Migrate the remaining consumers to
  `product.fabric.supply.Offer`. Census by *declaration* name: `ci_budget_tree` (10),
  `ci_fleet` (7), `ci_runner_placement` (4), `runner_spec_from_offer` (2), `fleet_host_budget` (2),
  `ci_deploy_target_host` (2), plus plan prose.

### Cut C, as measured rather than as assumed

I wrote that "the customer-facing execution class currently depends on physical supply — every
supply migration is a workflow migration", and specified Cut C as *building* a seam. **The seam
already exists and is already the authority.** `gunbc.ci_runner_target` declares
`CiRunnerTarget = FleetSelfHosted | UbicloudRunner | GithubHostedRunner`, projects
`ci_runner_target_spec` and `ci_runner_target_memory_regime` from one selection, and its
`FleetSelfHosted` arm delegating to `runner_spec_from_offer` is **correct and deliberate** — a
self-hosted fleet runner's labels genuinely *are* facts about the physical fleet. That is not the
wrong edge; that is the right derivation.

**The actual defect is narrower and sharper: the one live workflow emitter bypasses the seam.**

```
witness_floor_workflow.dag:325   runner: gunbc_ci_runner_spec()        <- runner_spec_from_offer(gunbc_ci_fleet_offer)
ci_runner_target.dag:54          gunbc_ci_selected_runner_spec()       <- the seam's projection, UNUSED by any workflow
```

`ci_runner_target`'s own note says "ci_workflow reads the spec projection now" — but `ci_workflow`
was deleted in the floor cut, and the workflow that replaced it reads the un-seamed function. So
the bypass is most likely **collateral of the floor cut**, not an original decision: the consumer
that honoured the seam was deleted and its replacement was wired to the older call.

**Why this is an unusually safe cut, and why it must still be done.** The two are provably equal
today — `ci_runner_target_witness_test` asserts
`ci_runner_target_spec(target: FleetSelfHosted) == gunbc_ci_runner_spec()` — so pointing the
emitter at `gunbc_ci_selected_runner_spec()` is byte-identical in the emitted workflow, and the
existing witness is the control. What it buys is that the runner selection becomes a one-row edit
again, which is the entire reason `ci_runner_target` was built; and it removes one of the three
direct `runner_spec_from_offer` call sites ahead of Cut D. The other two
(`ci_runner_placement`, `ci_deploy_target_host`) are *legitimately* about physical placement and
stay.

**Correction of a live note is part of this cut**: `ci_runner_target_mapping_note` asserts a
present-tense consumer that no longer exists. A note claiming its seam is honoured, inside the
module whose seam is being bypassed, is the stale-present-tense class in the place most likely to
stop the next person from checking.

Cut C is what makes D cheaper: close the bypass before migrating the type behind it, so the
migration meets three call sites rather than four and the workflow is not one of them.

## 11. Resolved: the Work key is a distinct carrier, not a rename — `WorkContentKey`

I proposed deleting it in favour of `ComputationIdentity`'s structural member, on the standing
preference for deletion over renaming. That was refused, and the reason is a grain distinction I
had collapsed:

- `ComputationIdentity` / `StructurallyIdentical` is a **qualification result** — a verdict about a
  relationship between two things, produced by a comparison.
- A Work key is a **content address** — it answers "what exact canonical structural computation is
  this?" for one thing, with no second operand.

A verdict is not an address, so replacing the address with the verdict's member would have erased
the addressing capability rather than deduplicated it. §2's test applies in the other direction
here: net concepts must not *shrink* by conflation either.

**Decided representation:**

```
WorkContentKey { digest: ContentHash }
```

- Named `WorkContentKey`, **not** `StructuralWorkKey` — the noun is the content address, and
  "structural" would borrow the qualification vocabulary that caused the confusion.
- Wraps `ContentHash`; it is **not** a branded serialization string. Brands are unenforced in this
  substrate (`where`-refinement predicates defer at compile time), so a branded string is a name
  standing where a distinct carrier was available — §4b's "richer type names are not safety".
- **Derived, never caller-supplied**: canonical contract → content hash → key. A constructor that
  accepts a key from its caller concedes the mismatched state is writable.

It is additionally a legitimate **peer** of `materialization_provider.request_key` at a different
grain, not a fork of it: one addresses a work item's canonical contract, the other addresses a
materialization request. Peers at different grains are not nicknames.

## 12. Resolved: six modules, but not the present six

Both extremes were refused. One module carrying all six identities would fuse work identity,
requester policy, supplier statements, accounting, authorization, lease fencing, execution
occurrence and result evidence into a single file, so every downstream addition edits the same
authority. The present six-file split is also wrong — but its defect is not the count, it is
**mis-owned shared types creating backward imports**:

- `supply` imports `ObservationReceiptRef` from `demand`
- `budget` imports `BudgetAccountId` from `demand`
- `budget` imports `ReservationRef` from `execution`
- `execution` carries Attempt, Grant, Receipt, payloads *and* receipt admission at once
- `identity` is a central record every other authority depends on

The correct answer is **multiple modules with an acyclic authority graph** — the count is
incidental, the acyclicity is the reason.

```
          work          budget
            \            /
             \          /
               demand          supply
                    \          /
                     \        /
                      execution
                          |
                        receipt
```

| module | owns | must not import |
| --- | --- | --- |
| `product.fabric.work` | `Work`, `WorkContentKey`, work identity relation/tag, canonical `Work` constructor | scheduling quantities, supplier types, `Demand` |
| `product.fabric.budget` | `BudgetAccountId`, `MoneyAccount`, `MoneyReservationRef`, generic `ReservationLeaseBinding<R>`, the currency wall, encumbrance projections | `Demand`, `Offer`, `ExecutionGrant` |
| `product.fabric.demand` | `Demand`, demand identity, `AdmissionTerms`, `BuyTerms`, `SatisfactionRequirement`, `DeliveryRequirement`, demand commands and access profiles | — |
| `product.fabric.supply` | `Offer`, offer identity, `OfferQuote`, `ObservedSupplyEvidence`, `QuotedSupplyEvidence`, availability, accepted revision identity | `demand` (it must not import Demand merely to borrow a generic receipt string — observed and quoted supply carry their own typed evidence authorities) |
| `product.fabric.execution` | `Attempt`, `ExecutionAttemptLineage`, `ExecutionGrant`, the fabric grant `AccessRequest` profile, accepted offer revision, the atomic reservation bundle, `LeaseEpoch`, `ExecutionAuthorityEnvelope`, `GrantBoundExecutionRef`, the grant-bound fencing kernel | — |
| `product.fabric.receipt` | `Receipt<P, R>`, receipt identity, the receipt-submission `AccessRequest` profile, attester authorization, result-independent receipt acceptance | a closed payload coproduct — it does not define one |

**`product.fabric.identity` is deleted.** Each owning module defines its own key type, relation tag
and principal-scoped specialization, projected through `std.key_relation.ScopedKey` rather than
routed through another freely constructible `(principal, key)` record — which is §5's construction
point: a freely constructible identity record concedes the mismatched pairing is writable.

The payoff is narrow provider imports: an SCM binding imports Work and Demand; a compute supplier
binding imports Supply; a host realizer imports Execution; a result publisher imports Receipt; an
accounting actuator imports Budget. Adding GitLab does not import market allocation internals;
adding Hetzner does not import demand commands; adding a Receipt result does not edit
`ExecutionGrant`.

## 13. Cut E, and what #8413 may not do

Two additions to §10's order, both signed off:

- **Cut E — recut #8413's modules** into the §12 layout, after the supply authority is singular.
- **Then, and only then, the first real `Demand` → `ExecutionGrant` transition**: Demand plus
  eligible Offers → fungibility → affordability → selection → atomic reservations →
  `ExecutionGrant`. This is deliberately last: a market allocation fold written while two supply
  authorities are live would be written against whichever one the author had in hand.

**The merge constraint on #8413, stated as a refusal:** it must not merge while both `ComputeOffer`
and the new `Offer` are independently authoritative — that is the §3 dual-authority interval the
replacement doctrine forbids on main. Either the live supply replacement lands first and #8413
consumes the landed authority, or #8413 itself carries the complete replacement and deletes
`ComputeOffer`. There is no third arm where both stand.

**#8413 is preserved, not closed.** Cuts A–D land as separately reviewable changes and are merged
*into* the still-open #8413 branch, so its final diff is the product protocol rather than every
prerequisite migration absorbed into one unreviewable change. That is what answers the bankruptcy
concern without discarding 16 commits of review-hardened work (§6).

## 14. Two corrections from Cut A's implementation, kept so Cut B does not inherit them

**`LeaseKey` does not exist and must not be minted here.** §7 above originally spelled
`lease_key: LeaseKey`; there is no such type, and I had not checked. Cut A refused to mint it and
the refusal is correct: a bare `type LeaseKey = NonEmptyStr` carries no construction wall, which is
the hollow-alias mode §4b names — a richer type name is not safety until construction and
acceptance enforce the distinction. It would also be worse than ordinary debt at this location,
because minting an unenforced brand *inside* the authority a consolidation cut exists to establish
cements a nickname at exactly the place every downstream module will cite. There is additionally no
grounded syntactic law for these keys to refine on: they are host-scoped and attempt-scoped strings
built by two different consumers. **The field is `NonEmptyStr`.** A branded key may land later, but
only together with its validating mint — strictly additive to the record, blocked by nothing here.

**The per-witness run recipe depends on which binary you reach, and inside a gunbc checkout that
is deliberately not yours.** The brief told Cut A that
`gunbc run --source-root dag --source-root src/v2 --entry <claim file> --function <fn>` runs a
single witness, with the not-`ProcessExit` refusal carrying the Bool. Two wrong explanations were
proposed and both are refuted by execution — Cut A's "the route cannot reach std at all", and my
"it works for closure-local functions and fails on cross-module type reach". The second died on a
clean measurement: three different `--function` selections from one file produce *identical*
file-wide diagnostics naming lines in functions that were not selected, so `--function` selects
nothing about resolution. A base-drift explanation died the same way, unchanged across five main
commits spanning 129 commits.

**The variable is the binary.** `/usr/local/ctrl-build-shims/gunbc` carries an explicit
gunbc-checkout guard: when the cwd's origin is `gunb-ai/gunbc` it execs the *baked*
`/usr/local/bin/gunbc` rather than routing, on the reasoning that a gunbc-development session wants
its own interpreter rather than the pinned one. So a bare `gunbc` inside these worktrees reaches a
binary that does not match the tree it is being pointed at. Measured on one specimen, same file,
same function, same tree:

| binary | result |
| --- | --- |
| `/usr/local/bin/gunbc` (baked, what bare `gunbc` reaches) | `resolved 1 sources`, then `unresolved type 'NonEmptyStr'`, `'ContentHash'`, `'Timestamp'`, … |
| `./target/release/gunbc` (built from the tree) | evaluates, returns the value, refuses with not-`ProcessExit` |

The baked binary reports `resolved 1 sources` — it never walks the source roots at all, so every
type in the corpus is unresolved and the diagnostics land on whatever line mentions one first.
That is why a stale binary looks like a modeling error in the file under test.

**The rule for every cut in this program:** invoke the interpreter by explicit path
(`./target/release/gunbc`), never bare `gunbc`, and treat a receipt as trustworthy only from a
binary built from the tree being evaluated. A green or red from the baked binary is a fact about
the pin, not about the change — and its failure mode is a plausible-looking located diagnostic in
your own file, which is the fabricated-plausible-output shape aimed at the author's own instrument.
