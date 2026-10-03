# Serving front door: closing the second door onto the serving groups

Item 3 of [dogfood-route-manual-interventions](dogfood-route-manual-interventions.md). The decision home is `gunbc.serving.serving_front_door`. The redemption chain's home is `product.capacity.redemption`, realized in `gunbc.fabric_event_log`.

## What landed in this change

- **The permit** (`ServingSeatPermitClaims`). It holds the claims a successful `gunbc.harness.harness_seat` `harness_bind_seat` already has:
  - the group, the exact seat partition and the co-tenancy class;
  - the structured seat reference (`HarnessSeatReference`: attempt = work identity, offer key = launch, stamp, round, head);
  - the `product.capacity.lease` grant identity and fence generation;
  - the caller principal and the lease term.
- **The signed representation is injective.** `serving_seat_permit_signing_input` length-prefixes every field, including the protocol, which is taken from the claim itself. As a result:
  - No content in one field can move a boundary into its neighbour.
  - A protocol rewritten after signing fails the MAC join.
  - Issuance refuses an unexpected protocol.
  - The permit header's decoder (owed under Proposal A) must consume exactly this encoding.
- **The ledger join is on the exact seat pool.** The door is configured with `HarnessSeatPoolIdentity` values minted by the seat authority (`harness_seat_pool_identity`: group, class, partition, ceiling, root). It then checks, against the ledger:
  1. The permit's partition equals the pool partition for its class. A prefix match is not accepted, so `group-b-not-a-seat` refuses. The selected pool must belong to the door's own group, and two configured pools for one class refuse rather than one being chosen.
  2. Exactly one acquisition of the reference exists, for exactly one seat.
  3. That acquisition was made by the grant event the permit names.
  4. The fence generation equals the acquisition's position in the chain.
  5. The partition folds under the pool's own root and ceiling.
  6. The seat has not been released.
  7. The recorded actor equals the authenticated caller.
  8. The recorded acquisition time plus the recorded term is still in the future.
- **Admission is a transition, not a read** (`product.capacity.redemption`):
  - **Redeem.** A seat moves `Unredeemed → InFlight { request, holder } → UseEnded`. The front door appends the redemption to the seat's redemption chain, beside its pool partition on the same fabric event log, by compare-and-set (`gunbc.fabric_event_log` `fabric_grant_redeem`). Only an admitted redemption forwards.
  - **Concurrent and replayed admissions.** A second admission of the same permit, concurrent or replayed, re-reads `InFlight` and refuses (`permit-already-redeemed`).
  - **Release while in flight.** Every seat release passes the redemption gate in `release_retry`, so a sender's release while a request is in flight refuses.
  - **Stream end.** The door ends the use (`fabric_grant_use_end`, exactly once per request) and then releases.
  - **Recovery.** A door that dies after redeeming leaves `InFlight` standing. The seat stays held and is charged to the redeemed request and proxy. It is ended only on *observed* quiescence (`front_door_recover`), never by a timer. Recovery releases only `UseEnded`; a seat that was never redeemed (`Unredeemed`) is left with its holder.
- **Minting.** `gunbc.harness.harness_cli` `harness_place_for` mints the permit after the seat is bound and records the principal it acts for as the acquisition actor; it no longer takes a free-text actor. If no permit can be minted, the seat is released and placement fails.
- **Counting.** `front_door_tally` keeps one count per refusal cause, plus the number of forwarded requests.
- **Router contract.** `gunbc.serving.router_realization` gains the forbidden arm `ForwardWithoutPermit`.
- **Witnesses** are in `test.claim.serving.serving_front_door_witness_test` (35 claims). They include:
  - the RED-first case;
  - each pool-join red (non-seat partition, another group's partition, class/partition mismatch, unserved class, zero amount, duplicate acquisition, wrong fence generation);
  - a protocol rewrite;
  - the ruling's outer-field collision;
  - all five redemption controls: two concurrent admissions give exactly one forward; a replay refuses; a sender release while the stream is in flight does not free the seat; stream end permits exactly one release; controller death leaves recoverable, charged state.

  Mutation runs disabled each new wall in turn, and its witnesses went red: the separator-joined signing input, the unsigned protocol, partition equality, the amount check, redemption ignoring state, and in-flight release.

## Proposal A: the forwarding realization (NOT applied; needs bold-bee-114's go)

**Constraint.** The interpreter is never on the token stream. Admission, which includes the redemption, runs **once per request**, and the body then streams without interpretation.

**Shape.** Use an auth-subrequest. A stock reverse proxy listens on the public `:30000`. For each request it makes one subrequest to a decision endpoint, which evaluates `front_door_admit_on_store` with the request's identity and the proxy's identity. Only a 2xx response streams to vLLM on loopback. The stream's end, whether complete, client-aborted or upstream-failed, must reach `front_door_stream_end` exactly once for that request. If the proxy cannot guarantee that hook, the seat stays `InFlight` and is recovered only on observed quiescence.

**Owed before Proposal A can be applied:**
1. **A measured admission latency budget.** Measure p50 and p99 of the decision endpoint, which now includes one compare-and-set, against group-b's time-to-first-token.
2. **The permit header wire form and its decoder.** It must decode exactly the length-prefixed signing encoding.
3. **The stream-end hook and the quiescence observation** that `front_door_recover` consumes. One candidate is vLLM's per-request state, observed through `gunbc.serving.engine_progress`.
4. **Tally persistence.**

**Consumption.** Until Proposal A lands, `front_door_admit_on_store`, `front_door_stream_end`, `front_door_recover` and `front_door_access_decision` have no production caller. This is a declared frontier, and its trigger is Proposal A's go. The redemption gate in `release_retry` *is* consumed today, by every harness release.

## Proposal B: the host change (NOT applied; needs bold-bee-114's go, because it takes group-b's traffic)

On each serving group's head (group-b: 192.168.1.236):
- vLLM binds to `127.0.0.1`.
- The front door takes `0.0.0.0:30000`.
- The permit key is materialized at `serving_seat_permit_key_path`, readable only by the door and the placement authority.

This is converged through the existing `gunbc.spark` serving deployment authority. The door and the loopback rebind move in one step.

**Owed population before Proposal B (rollout correction from the ruling).** Only `harness_place_cli` mints a permit today. The probe, worker, reviewer, auditor and supervisor paths in `gunbc.harness.harness_cli` acquire a seat and then call `harness_run_turn` with the URL, ignoring the seat record. Their acquisition actors are free-text role identities (`worker:<worktree>`), not the authenticated principal the door requires. Moving vLLM to loopback before they change would refuse the factory's own harness traffic. Each of these paths must do one of two things:
- **(a)** Mint and present a permit under a principal the door authenticates, record that principal as the actor, and end its use at the door.
- **(b)** Use an explicitly modeled internal route that a bypasser cannot reach.

Choosing between them is part of Proposal B's go.

## Credential: the permit key

- **Key id:** `serving-seat-permit-2026-10` (HMAC-SHA-256). With HMAC, the verifying key is also the issuing key, so custody is the boundary.
- **The ledger and the redemption chain are the authority; the MAC only protects the claims.** A holder of the real key cannot forward by minting:
  - A permit for a seat that does not exist, sits in another pool, belongs to another caller or another launch, claims another fence generation or stretches its own expiry is refused on what placement recorded.
  - A permit for a seat it does hold forwards at most one request at a time.
- **What the caller check can tell apart today.** There is one service principal (`principal:gunbc/workflows/fabric`). So the check separates humans from the service, but not one service sender from another. Per-sender principals are credential issuance, which belongs to the managed-identity row (#13068), and this change does not fork it.
- **Issuance, rotation and revocation drill** are owned by `credential-lifecycle-revocation` (gunb-ai/gunbc#13068). This key is registered there as a member that row must cover.

## Migration note: what the existing senders must change (stays manual)

- **ctrl dashboard router.**
  - Delete its load score and its host selection.
  - Per request, call `harness_place_cli` and present the permit, in the header form fixed under Proposal A item 2.
  - Do **not** release a seat that the door has redeemed. The door releases at stream end, and a sender release refuses while the request is in flight. A seat that was placed but never forwarded is still released with `harness_release_cli`.
  - Authenticate as Fabric service evidence.
- **ac-dialogue batch.**
  - Stop POSTing raw requests.
  - Take one seat per in-flight request and present that seat's permit.
  - A batch wanting N concurrent requests holds N seats. A permit is redeemable once, so one permit cannot carry two requests.
- **Which identities may request grants.** This stays an operator decision at placement.
