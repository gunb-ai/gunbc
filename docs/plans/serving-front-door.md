# Serving front door: closing the second door onto the serving groups

Item 3 of [dogfood-route-manual-interventions](dogfood-route-manual-interventions.md). Decision home: `gunbc.serving.serving_front_door`.

## What landed in this change

- **The permit** (`ServingSeatPermitClaims`). It holds the claims a successful `gunbc.harness.harness_seat` `harness_bind_seat` already has:
  - the admitted launch (`vllm_endpoint_process_launch_key`);
  - the group, the partition and the co-tenancy class (the seat entitlement);
  - the `product.capacity.lease` `LeaseGrant` reference, plus the fence grant and generation;
  - the caller principal and the work (attempt) identity.

  The expiry is `LeaseGrant.expires_at`. Revocation is the seat ledger: a released seat refuses at the door, whatever its permit says.
- **Integrity** comes from `extdeps.crypto.mac` over one signing input, using the separator from `gunbc.auth.approval_capability`. The MAC authenticates the claims. Currency is a separate, stateful read of the partition (`seat_grant_currency_of`).
- **Minting.** `gunbc.harness.harness_cli` `harness_place_for` mints the permit right after the seat is bound and prints it in the placement receipt. If no permit can be minted, the seat is released and placement fails. A seat with no permit would sit held for a full lease while the door refused every request on it.
- **The decision.** `front_door_admit` checks, in order:
  1. caller authentication;
  2. that a permit is present;
  3. the MAC join;
  4. protocol, group, launch, validity window and caller binding;
  5. the ledger.

  It has exactly one forwarding arm, and that arm requires both axes. `front_door_access_decision` projects the verdict onto `std.access` `AccessDecision`. This is a stated departure from building the verdict out of `decision_meet`: `decision_meet` erases which axis refused, and the counter needs that.
- **Counting.** `front_door_tally` keeps one count per refusal wire, plus the number of forwarded requests.
- **Router contract.** `gunbc.serving.router_realization` gains the forbidden arm `ForwardWithoutPermit`.
- **Witnesses** are in `test.claim.serving.serving_front_door_witness_test`. The RED-first case is a valid caller with no grant: it is refused and counted. A grant for another group, a grant for another launch, an expired grant, a released seat, a seat that was never admitted, rewritten claims and an unauthenticated caller all refuse. The positive control forwards. A mutation run weakened the expiry boundary and the counter, and the matching claims went red. A second mutation disabled the ledger-actor and ledger-term checks, and their two reds failed.

## Proposal A: the forwarding realization (NOT applied; needs bold-bee-114's go)

**Constraint.** The interpreter is never on the token stream. vLLM responses are mostly SSE streams, and the gunbc serve channel is request/response. A per-request `.dag` MAC verification on a TP=4 engine's hot path has an unmeasured cost. So admission runs **once per request**, and the body then streams without interpretation.

**Shape.** Use an auth-subrequest. A stock reverse proxy (for example nginx `auth_request`) listens on the public `:30000`. For each request it makes one subrequest to a decision endpoint that evaluates `front_door_admit_on_store` and answers 2xx or 403. Only on 2xx does it stream the request and response to vLLM on loopback, with buffering off. The decision endpoint emits the tally. A v2-emitted binary hosting the same fold is the alternative. The subrequest shape is preferred because the proxy then owns streaming and none of the streaming code is ours.

**Owed before Proposal A can be applied:**
1. **A measured admission latency budget.** Measure the decision endpoint's p50 and p99 against group-b's time-to-first-token. Name the budget before any traffic moves.
2. **The permit header wire form and its decoder** (`serving_seat_permit_header`). Today the receipt prints the permit's fields one at a time. The header needs one canonical encoding with exactly one decoding, so a permit cannot be authenticated under one reading and acted on under another (see the note in `gunbc.auth.approval_capability`). The decoder belongs to the realization that consumes it.
3. **Tally persistence.** Decide where the counts land (a fabric observation event, or the metrics the dashboard reads).

**Consumption.** Until Proposal A lands, `front_door_admit_on_store`, `seat_grant_currency_on_store` and `front_door_access_decision` have no production caller. This is a declared frontier, and its trigger is Proposal A's go.

## Proposal B: the host change (NOT applied; needs bold-bee-114's go, because it takes group-b's traffic)

On each serving group's head (group-b: 192.168.1.236):
- vLLM binds to `127.0.0.1` on an internal port.
- The front door takes `0.0.0.0:30000`.
- The permit key is materialized at `serving_seat_permit_key_path`, readable by the door and by the placement authority, and by no sender.

This is converged through the existing `gunbc.spark` serving deployment authority, not by hand. The cutover is atomic per group: the door and the loopback rebind move in one convergence step.

## Credential: the permit key

- **Key id:** `serving-seat-permit-2026-10` (HMAC-SHA-256). With HMAC, the verifying key is also the issuing key, so custody is the boundary. The key must not reach a sender.
- **The ledger is the authority; the MAC only protects the claims.** `front_door_admit_held` forwards only when everything it reads from what placement *recorded* holds, not from the permit:
  1. The reference was acquired, by the permit's grant event, in the named partition, and has not been released.
  2. The acquisition's recorded actor equals the authenticated caller. `harness_place_for` now records the principal it acts for as the actor, and no longer takes a free-text actor argument.
  3. The recorded reference decodes, through `harness_seat_reference_names` (the inverse of `harness_seat_reference`), to exactly the permit's work and **this door's** launch key, and the partition is one of the door's group's partitions.
  4. The recorded acquisition time plus its recorded term is still in the future.

  So a holder of the real key cannot get through by minting. A permit for a grant that doesn't exist, for another caller's live seat, for other work, for another launch, or with a stretched expiry each refuses (witnessed, with a mutation control).
- **What check 2 can tell apart today.** The authentication layer has exactly one service principal (`principal:gunbc/workflows/fabric`). So check 2 distinguishes a human session from the service, and one human from another, but not one service sender from another. Every service caller that passes authentication *and* holds the permit key passes check 2 for any service-held seat. Splitting that needs per-sender service principals, which is credential issuance and belongs to the managed-identity row (#13068). This change does not fork it. Until then, the permit key must reach only the placement authority and the door.
- **Issuance, rotation and revocation drill:** owned by `credential-lifecycle-revocation` (gunb-ai/gunbc#13068, the managed-identity row). This key is registered there as a member that row must cover. This change issues no credential.

## Migration note: what the existing senders must change (stays manual)

- **ctrl dashboard router.**
  - Delete its load score and its host selection; this is already required by `gunbc.serving.router_realization`.
  - Per request, call `harness_place_cli` and read the receipt.
  - Send the permit, in the header form fixed under Proposal A item 2, to the granted `url`.
  - Release with `harness_release_cli` using the receipt's capability when the proxied request ends.
  - Authenticate as Fabric service evidence.
- **ac-dialogue batch.**
  - Stop POSTing raw requests.
  - Take one seat per in-flight request (class `BatchQualityTolerant` when that entry exists; `harness_place_cli` binds `InteractiveQualitySensitive` today), present that seat's permit, and release on completion.
  - A batch that wants N concurrent requests holds N seats. It is bounded by the tolerant ceiling and never exceeds it by sharing one permit: each permit names exactly one reference.
- **Which identities may request grants.** This stays an operator decision at placement. The door only checks that the principal presenting a permit is the principal it was minted for. Today the only service principal is `principal:gunbc/workflows/fabric`, so the caller binding is coarse until managed identity gives each sender its own principal.
