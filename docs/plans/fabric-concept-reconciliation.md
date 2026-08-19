# Fabric authority census: what the new carriers re-minted, and what they must consume

**Status: findings document. Verified claims are marked; inferences are marked separately. No code lands from it.** Operator directive 2026-08-18: reconcile the new fabric concepts against the existing corpus *before* they land, because the repository already carries realization, materialization, lease, ownership, reconcile, access and budget concepts, and an unlinked fabric is rework.

Subject: branch `fabric/terminal-contract` (`product.fabric.*`). Doctrine: DESIGN §2 (DFS the concept DAG before inventing vocabulary; net concepts must not grow by re-invention), §3 (single authority; nicknaming), §5 (fail-closed).

## 1. The class

The fabric was modelled as a clean slate and re-minted at least four concepts that already existed: a lease, an attempt, a step receipt, and content-key derivation over declared inputs. That is the §2 failure in its named form — **a fresh authority minted for a concept that already exists is a failed decomposition** — and in one case below the fork did not merely duplicate: **it dropped a field, and the dropped field was the security check.**

## 2. Verified by execution

Each of the following was read from the branch in this session.

### 2.1 `LeaseIdentity` is a fork of `std.temporal_effect` `HeldLease`

```
HeldLease                              LeaseIdentity
  lease_key            NonEmptyStr       lease_key            LeaseKey (branded NonEmptyStr)
  resource_fingerprint ContentHash       resource_fingerprint ContentHash
  generation           Int               generation           Int
  owner_fingerprint    ContentHash       -- absent --
  observed             HeldLeaseObservedState  -- absent --
```

Identical field names, identical types, strict subset. This is a re-declaration, not a specialization.

**The dropped `owner_fingerprint` is the cause of §2.2.** The existing carrier already modelled lease ownership; the fork discarded it; the receipt fence consequently had no owner to compare. **Authority: `HeldLease`.** The generic fencing kernel that `product.fabric.execution` should extract belongs on it, and `observed` is the arm that makes lease liveness expressible rather than assumed.

### 2.2 Receipt acceptance never checks the attester

`admit_receipt_for_grant` reads exactly `r.attempt`, `r.lease.lease_key`, `r.lease.resource_fingerprint` and the generation. It never reads `r.id`. A Receipt carrying a foreign `id.principal`, the victim's `attempt`, and the current lease is admitted.

The same function's own comment records this class being fixed once already for `attempt` — *"a cross-tenant fail-open in the exact carrier whose whole design puts the principal inside the identity so it cannot be dropped by accident. It was dropped by accident anyway, one field access at a time."* The class survived one field over.

### 2.3 The grant authorization wall is commentary

`product.fabric.execution` imports `std.types`, `std.measure`, `std.content_hash` and three sibling fabric modules. It does **not** import `std.access`. Nothing constructs an `AccessRequest`, calls a policy, or consumes a `Permit`, despite the module comment stating the grant "projects into `std.access`". `ExecutionGrant` is freely authorable.

### 2.4 `FabricGrantEvidence` is type-erased

Five fields, all `ObservationReceiptRef`: `fungibility_receipt`, `offer_authority`, `resource_conservation`, `budget_conservation`, `principal_authority`. Any one substitutes for any other and type-checks, so the record proves five references exist, not that five distinct facts were established. The repair is the real owning evidence carriers, never five hollow brands — that would be nicknaming the fix.

### 2.5 `ExecutionGrant` carries no authority envelope

Fields are identity, offer + quote, context, evidence, resource reservations, money reservation. Nothing states what the executor may do. Realization therefore has nowhere to read authority *from* and can only infer it from placement — the ambient-privilege shape the least-authority law rejects.

**The missing carrier already exists:** `std.effect_grant` `Envelope { frame, grants }` over `Grant { verb, root, binding, lifecycle }`, with `grant_covers`, `grant_coverage` and `CoveringGrant` already folding coverage over a namespace position. **Authority: `std.effect_grant`.** The fabric grant references an envelope; child components attenuate from it.

### 2.6 `std.access.AccessSubjectRef` does not exist

Grepped across `dag/` and `src/`: zero occurrences. `AccessRequest<S,A,O,C,E>` is fully generic with `subject: S` and no concrete subject carrier.

This matters because the review recommending it proposed *consuming an existing coordinate*; the true operation is **minting a new one**, which needs its own §3 justification. What exists is `gunbc.principal_projection` — `GunbcPrincipal`, `OidcProjectedPrincipal`, `PosixProjectedPrincipal`, `PrincipalProjection` — product-layer and evidence-derived, correct quarry but not consumable by a std-layer fabric as-is. **There is no provider-neutral principal carrier in `std` today**; supplying one is new modeling with a real design decision inside it.

### 2.7 `Offer` is not greenfield: its predecessor is live, consumed, and named in its own comments

`gunbc.ci_fleet` already declares:

```
type ComputeOffer {
  provider:         ProviderIdentity
  supply:           ComputeSupplyFacts
  available_window: AvailabilityWindow
  cost_quote:       CostEstimate?
}
```

against `product.fabric.supply` `Offer<P> { id, executor, shape, capabilities, trust_domain, quantity_bound, ready_at, quote, evidence }`. The correspondence is field-for-field: `provider`/`executor`, `supply`/`shape`+`capabilities`, `available_window`/`ready_at`, `cost_quote`/`quote`.

**This fork was known to its author.** The new module's comment reads: *"The predecessor model made cost_quote optional and every owned host used the absent arm, so a consumer reading it could not distinguish costs nothing from nobody priced this."* It names the predecessor's exact field. So `Offer` is not a new carrier landing beside nothing — it is **Y in a replacement migration whose X is still the production authority.**

**And X is load-bearing right now.** `gunbc.runner_spec_from_offer` derives fleet runner labels from the fleet offer, and its consumers include `gunbc.witness_floor_workflow` — the module that emits the workflow running this repository's CI today — plus `ci_runner_placement`, `ci_runner_target`, `ci_deploy_target_host`, `host_standup`, `fleet_converge_workflow` and `assimilate.bmc_token_federation`.

The consequence is a sequencing correction, not merely another row. Every prior statement in this conversation that #8413 is cheap to repair *because it has no production consumer* is true of the fabric **carriers** and false of the **concept**: compute supply already has an authority with live CI consumers. Under DESIGN §3 this is a replacement cut requiring a named root consumer, a consumer census with dispositions, and one transition — not a new model that lands and later absorbs the old one. The dual-authority interval the doctrine forbids is already open.

### 2.8 A third supply-and-selection vocabulary exists

`gunbc.dispatch_selection` carries `CodexOffer`, `ClaudeOffer`, `CursorOffer`, `ProviderOffer`, `ProviderInventory`, `ProviderSelectionRequest`, `ResolvedProviderSelection`, plus dispatch attempt, temporal-bounds and usage-observation carriers — a complete offer / inventory / selection-request / resolution model for routing work to AI providers.

Whether that is the same market relation as the fabric's at a different subject grain, or a genuine peer, is a verdict this document does not issue. It is recorded because a design that believes it is introducing the repository's first market vocabulary is working from a false premise: it would be the third.


### 2.9 The census method failed once, and the correction is recorded because it will recur

The first pass of this document searched for concepts it *already suspected* — realization, materialization, access, effect_grant, temporal_effect — and reported five forks. An independent census going the other direction, from the authority surface inward, found roughly twice as many. `dag/std/` holds **134 modules**; enumerating them is cheap and was not done.

**The rule: enumerate the authority surface, do not grep for the concepts you already have in mind.** A fork is by definition a concept you did not know was already modelled, so searching by remembered name is structurally unable to find the ones that matter.

Verified on the second pass, each mapping to a fabric carrier already known defective:

- **`std.key_relation` `ScopedKey<Relation, Scope, Key>`** against `FabricIdentity<P, K>`. Identity-under-a-relation as `(scope, key)` is the same relation as `(principal, key)`, and `ScopedKey` additionally carries a **phantom relation tag so unrelated relations cannot substitute** — precisely the property whose absence forces `fabric_identity_eq` to be hand-written and lets a receipt identity stand where an attempt identity belongs.
- **`std.computation_identity`** (`ComputationIdentity`, `IdentityUnknownCause`) against `WorkKey`. This module is **live, not planned**, so `WorkKey` forks an existing authority rather than anticipating a future one.
- **`std.materialization_ladder`** (`Frame`, `DemandNature`, `CacheKeying`, `CacheCoverage`, `ProviderTier`, `CacheProvider`) and **`std.effects`** (`EffectShape`, `IdempotencyEvidence`, `OperationEffect`) against `ResumabilityTier`. The tier is an authored verdict precomputing what those two derive.
- **`gunbc.fleet_container` `ResourceEnvelope`** against thread-only `Shape`.
- **`std.claim_evidence`** (`RecordedFact<F>`, `Claim<S,T,P,Scope,Bound>`, with W3C-PROV entity/activity/agent identifiers) against `FabricGrantEvidence` — the typed evidence authority the five-identical-strings record erased.
- **`std.pareto`** (`DominanceVerdict = Dominates | Dominated | Equivalent | Incomparable`, `AxisGoal`) against `SelectionPolicy`. Selection over price, delay, and later isolation and performance is multi-axis; the one-arm sum scalarizes it while the dominance authority already exists.
- **`std.realization_schedule`** (`CostBasis`, `CostAccount<S>`, `RealizationObjective`) against the fabric's cost and objective handling.
- **`MoneyLease { reference, generation }`** on the same branch — a **third** lease-fencing carrier beside `LeaseIdentity` and canonical `HeldLease`.

### 2.10 What this changes about the verdict

With §2.1–§2.9 taken together the finding is no longer a list of forks to repair. **The six product concepts — Demand, Work, Attempt, Offer, ExecutionGrant, Receipt — express real fabric-level facts the existing kernels do not collectively name, and they survive. The layer underneath them is substantially re-minted and does not.** The honest operation is therefore a **recut against current main**, not another commit on the existing carrier shape.


## 3. Name collisions

Two distinct types named `Grant`: `std.effect_grant` `Grant` (verb x namespace subtree) and `product.fabric.execution` `ExecutionGrant` (resource + money lease). They answer different questions and are legitimate peers, but the bare noun is now ambiguous in conversation and in grep. Likewise `Materialization` names a *strategy* in `std.realization` (`Recompute | Memoize | Share`) and a *receipt arm* in `ReceiptPayload`. Neither is a fork; both are nickname hazards worth resolving by naming rather than by merging.

## 4. Pairs awaiting a verdict

Parallel shapes where fork-versus-specialization is a judgment, not a measurement. Recorded so the decision is made deliberately rather than by default.

- **`Attempt` against `EffectAttemptIntent`** — `{ attempt_id, intent_hash, target, created_at }` versus `{ id, work, lineage, created_at }`.
- **`Receipt` against `EffectStepReceipt`** — the latter carries `applied`, `read_back_ok` and `observation_digest`. **`read_back_ok` is the independent-readback fact the teardown and boundary-proof obligations require, already modelled.** A fabric receipt that cannot express it will re-mint it.
- **`derive_work_key` against `std.materialization_provider` `request_key` / `declared_inputs_digest`** — two content-key derivations over declared inputs. The materialization kernel already owns request-key derivation with `DeclaredInput` and local-versus-full key separation.
- **`ReceiptPayload.MaterializationReceipt` against `MaterializedArtifact`** — a second materialization authority while the first sits at one of seven consumers wired. Likely resolution: the fabric becomes the eighth consumer and the arm disappears into the generic `Receipt<P, R>` result.
- **`WorkKey` against the `ComputationIdentity` lattice** (duplicate-work lane) — `WorkKey` is content-hash equality, exactly the `StructurallyIdentical` arm, with no `NormalizedIdentical`, `ExtensionallyIdentical{bound}` or `IdentityUnknown{cause}`. Either deduplication is a different question from duplicate-work qualification, or the fabric is forking the identity question while the other lane generalizes it.
- **`Offer` / reservations against the fleet-reconcile spine and `gunbc.ownership`** — `membership_reconcile` already diffs desired against observed by identity, and `Ownership` already makes removal the only ownership-consulting arm.
- **`ResumabilityTier` against `EffectShape` idempotency**, **`OfferEvidence` against `std.observation` / `ObservationVerdict`**, **`FabricGrantAction ExecuteAttempt` against `effect_grant` `Verb`**.

## 5. Consequences for the repair

The three-commit repair proposed in review is compatible with this census, with two amendments:

1. Its Commit A step "ground identities on `AccessSubjectRef`" names a nonexistent type (§2.6) and must be restated as new modeling with a justification.
2. Its Commit B step "extract `GrantBoundExecutionRef` and one fencing kernel" must extract onto `HeldLease` (§2.1), not onto the forked `LeaseIdentity` — otherwise the extraction cements the fork and the recovered `owner_fingerprint` never returns.

**None of the verified defects is exploitable today**, because the branch has no production consumer. That is precisely why this is the cheap moment: every one of them becomes a migration once a consumer exists.

## 6. What this document does not claim

Section 4 pairs are unverified judgments, not measurements. No verdict is issued on them here. The review's remaining findings — unauthorized `DemandCommand` transitions, `MoneyAccount` without a principal, `FabricIdentity`'s `P` proving no authentication — are reported but were not independently verified in this session. No repair is authored. The branch is not owned by this session, so routing these findings to its owner is a separate, unperformed action.
