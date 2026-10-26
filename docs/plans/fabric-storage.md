# Fabric DB

The fabric DB is gunbc's own store for the fabric's durable state. The operator decided on 2026-09-19 to move the fabric event log off Git. Its first consumer, `gunbc.fabric.fabric_event_log`, carries capacity transactions, including serving seats and upstream quota.

**Target-policy supersession, 2026-10-23:** Brian replaced this plan's fixed one-host/one-root placement and R2-as-step-4B-durability-tier policy with contract-constrained arbitrage among this host's disk, another host's served disk, and R2. The authority for that target, the verified current baseline, interfaces, complete migration inventory and staffing is [`gunbc.plans.logical_work_realization::logical_work_realization_plan`](../../dag/gunbc/plans/logical_work_realization.dag). Cost and latency readings must be real and cited; missing evidence remains `SelectionNeedsEvidence`. No backend has a fixed role. One linearization point **per head**, durability, confidentiality, access and state continuity remain constraints. This design change does not move the incumbent store or authorize a deployment.

The September rollout and drain below are historical receipts, not instructions to discard the state of a new cohort. In particular, the still-Git-backed `roadmap_event_carrier` is a different authority from the already-migrated fabric event log and requires its own complete state transition.

## What it is

Three facts, kept apart (DESIGN §3):

- **Interface:** `std.fabric_storage`. It has immutable content-addressed objects and compare-and-set named heads, built from `std.content_hash` keys and `std.durable_compare_and_set` rather than new vocabulary.
  - **Links the store can see:** an object carries its links as data the store reads. That lets a closure read walk the whole chain where it is held.
  - **Refusals:** each has a typed arm:
    - `FabricStoreUnreachable`
    - `FabricStoreRefused` (the CAS store-failure vocabulary)
    - `FabricObjectMissing`
    - `FabricObjectCorrupt`
    - `FabricReplyUndecodable`
    - `FabricHeadNameRefused`
  - **Lost race:** a lost race is `FabricHeadMoved`, carrying both heads. It is an outcome, not a fault.
- **Realization:** three pieces.
  - `gunbc.fabric.fabric_storage_file_store`: heads are `gunbc.durable_cas_file_store` slots, and objects are write-once files beside them.
  - `gunbc.fabric.fabric_storage_serve`: the served endpoint, one `gunbc serve` process on the placed host, loopback-bound behind `tailscale serve`.
  - `gunbc.fabric.fabric_storage_client`: the caller-side binding. The current default placement is reached through an admitted local door/socket or HTTPS; separately admitted host-local file bindings also exist.
  - **Wire:** `gunbc.fabric.fabric_storage_wire` is the one text form every outcome crosses the hop in.
- **Incumbent policy:** `gunbc.fabric.fabric_storage_placement`, one placed default store on the prod-role host. This is the current deployment, not the target policy. Its replacement is the arbitrage program above; a changed quote alone never relocates existing mutable authority.

## Steps 1–3 (historical September rollout)

1. **Model:** the interface, as above.
2. **First realization:**
   - One writer on srv1. There is one linearization point by placement, and an unreachable placement refuses rather than failing over.
   - The store root is an *ensured* host directory (`gunbc.live_deploy.spec` `FabricDbStoreDirectory`). No retract removes it.
   - The endpoint's unit and tailscale route are owned by the srv1 live deployment.
3. **Cutover:**
   - `gunbc.fabric.fabric_event_log` runs on the fabric DB in one motion, and its git plumbing is deleted.
   - An event's parent is a store-held link, so a partition read is one closure and the full chain always arrives.
   - Consumers updated: the harness seat path, fabric quota, and both probes.

### Cutover and drain

Partitions start empty in the fabric DB, and nothing is imported from the git log. The new store never derives state from the one it replaced.

This is correct once every lease granted from the git log has expired:

- **Seats:** the longest term the fabric grants is the harness seat term, `gunbc.harness.harness_cli` `harness_seat_policy`, which is the request deadline plus the release allowance.
- **Quota:** quota leases are declared per call, and the one live caller (`gunbc.roadmap.roadmap_publish_observe`) takes 120 s.

After the deploy, wait at least the seat term before trusting a seat count. A seat held under the git log is invisible to the new partition, so for at most one term a pool could over-grant by the seats still held in the old log.

The git repository at `/opt/gunbc/fabric-event-log.git` on srv1 is left in place, read-only, as history. Nothing consults it.

### Access today (source-verified 2026-10-23)

The served deployment is loopback-bound behind `tailscale serve`; the local door/socket and HTTPS routes have distinct admission. `Tailscale-User-Login` has the standing declared by `extdeps.tailscale.identity`. The protected `/state` route requires one writer MAC for **every operation**, including reads. See `fabric_storage_binding`, `fabric_storage_client` and `fabric_storage_serve` for the executable policy.

The original unrestricted-append paragraph is superseded; it is not a current access claim. The remaining uncertainty, readiness and interface gaps are recorded in the new plan's verified-baseline section. Candidate pricing does not confer access or bypass the admitted route.

## Two planes

The fabric stores two kinds of thing. Their contracts remain distinct while placement is selected from requirements and evidence:

- **Metadata plane:** `std.fabric_storage`. Small structured immutable objects whose preimage travels inside one request, links the store walks, compare-and-set heads, durable history. An object body is `NonEmptyStr`; it does not grow a binary constructor, because fusing bulk bytes into this wire would push every existing client through a binary rewrite.
- **Blob plane:** `std.fabric_blob`. Opaque files that live on disk on both sides of a transfer, under an exact address (an admitted cryptographic key, not the content digest). A read stages the bytes in a directory the transfer minted and mints a sealed `FabricBlobReading` (SHA-256 and byte size derived from the staged file); whatever owns the bytes' meaning verifies them from that reading. Publication is create-if-absent; there is no listing, no prefix fallback, no overwrite and no delete on the publisher's surface. Objects are whole, up to a bound the placement declares, and an object over it refuses before upload.

`std.cache_interface` already distinguishes `StructuredArtifact` from `RawBytes`, `TarArchive` and `FileTree`; the metadata plane carries the first, the blob plane the others.

## Steps 4–5 (historical program, revised target above)

**4A. Whole-object R2 blob realization (the build cache is its first consumer).**

- The R2 staged-file transfer is `gunbc.cloudflare.r2_staged_transfer`, extracted below the durable origin's provenance. The durable origin (`gunbc.cloudflare.r2_origin_object`, which alone mints `OriginReading`) and the blob plane (`gunbc.fabric.fabric_blob_r2`, which mints `FabricBlobReading`) both consume it; neither shares the other's provenance carrier.
- Create-if-absent object writes are **not** missing: `extdeps.tools.curl` sends a signed `If-None-Match: *` and the transfer classifies stored / already present / refused / outcome-unknown. ETag-conditioned heads also now exist in `fabric_storage_r2_head::{r2_head_read,r2_head_advance}`. They are not yet a complete R2 realization behind the shared binding: history, missing-target refusal and other gaps are explicit fabric-integrator obligations in the new plan.
- Placement: the cache is a third private bucket, the `FabricCacheBlobs` purpose in `gunbc.cloudflare.r2_origin`, because its writer (one canonical publisher), its readers and its retention differ from the durable and boot buckets. It is a placement, not a third store: the interface is `std.fabric_blob`. Its layout, object bound and release are `gunbc.fabric.fabric_blob_placement`.
- Retention is physical release by age (`std.cache_interface` `ReleasedAfterInterval`), realized by an R2 lifecycle rule (`extdeps.cloudflare.r2_lifecycle`) that `gunbc.cloudflare.r2_bucket_ensure` converges with readback. It is not a freshness window: a present object stays a valid hit, and a released one is an established miss. It bounds object age only; peak bytes and cost stay unestablished until pack size and publication rate are measured.
- `fabric_blob_shell` retains an operation-plan shell realization and its evidence. The 2026-10-23 audit found no production caller of its get/put script producers in `dag/` or `src/v2/`; this historical intended route is not evidence of current workflow execution. The seed does have a native-product restore caller, inventoried in the new plan.

**4B fixed role is superseded.** R2 is an eligible realization for whichever logical contracts its evidence satisfies; it is not permanently a replica tier, and srv1 is not permanently the linearization host. Arbitrage preserves one linearization point per head, required history and authority continuity. Existing metadata objects or heads do not move until their cohort's admitted cut reconciles them.

**5. Remaining capability directions.** Multipart and chunked blobs, resumable transfer, replication with explicit head authority, failover preserving that authority, and collection of unreachable chunks remain capability questions. They are not fixed backend roles or permission to bypass the new plan's selection, retention, release and state-transition contracts.

The metadata interface already leaves room: object refs are `ContentHash` (a family change is a new mint, not a new type), heads are per name (so heads can shard by name), and every failure a replicated store adds (an unreachable replica, a stale read) has an existing arm to land in. The blob interface leaves the same room: an address and a reading do not change when an object is carried in chunks.

## Known limits of the first realization

- **Weak digest, and the strong one is built and measured, not missing:** object refs mint through the non-cryptographic structural family (`std.content_hash` `content_hash_of_value`). A collision is refused at put (`FabricObjectCorrupt`), never read through. The computing SHA-256 exists in the substrate — `extdeps.crypto.sha2` `sha256_hex`, FIPS 180-4, witnessed by `test.claim.sha256_fips_witness`. What does **not** exist is a mint over it: gunbc#11996 authored one in `std.content_hash`, bound `fabric_object_ref_of` to it, and **withdrew both on measurement**. Interpreted, every put, every verified get and every closure step pays a SHA-256, and the spark pair_serving real-execution claims that append and re-read chains through `gunbc.fabric.fabric_storage_file_store` ran two to three hundred times slower than on main, cancelling the required floor at its cap. The mint's own inhabitance claim then failed the same way for a second, independent reason: one interpreted SHA-256 over a single-block preimage performed 244,396 eval steps against the required floor's 72,300-step new-witness budget (`v2.workflow.required_floor` `claim_eval_step_budget_for_identity`), and one compression block is the floor of that cost — so the seam could not carry an executing claim either, and a declaration with no executing consumer is what DESIGN §3c refuses. The row is `gunbc.recurring_failure_mode` `fabric_object_identity_is_a_structural_locator` (rung 1, ceiling 3, the run and the differential cited by id). **The blocker is native emission, not strength:** `UInt32` is `Compose<UInt, MachineWidth<32>>`, a structural operand, and `std.operator_realization` `operator_realization_for` routes arithmetic on a structural operand to `structural_arithmetic_refusal`, so the closure runs only under the interpreter (`gunbc.recurring_failure_mode` `bounded_natural_arithmetic_evaluated_as_unbounded_int`). The trigger is stated at capability grain: native emission of that closure sufficient for every consumer of the file store to run at floor rate. The shell-out alternative was refused for a **structural** reason under either family: `extdeps.crypto.hash` `crypto.Sha256Sum.File` takes a *path*, while `fabric_object_preimage` produces bytes **in memory**, so reaching it means a temp-file write per object — the custody gap (hash one file, store another) that verification exists to close — and makes minting an effect. Minting stays pure, so `fabric_object_verified` stays a pure check. A collision remains a **content-identity** failure that can enable an isolation failure given authorization and sharing; tenant isolation stands on its own and never on the digest.
- **When the family does change, the parse changes with it and the stored population is measured first.** `fabric_object_ref_of_wire` admits exactly the family the mint produces, so the two move together (`gunbc.recurring_failure_mode` `a_carrier_swap_moves_the_mint_and_strands_the_parser`). Moving refs from fnv1a64 to SHA-256 makes any object *already stored* under the old family unreadable, and not loudly: `fabric_object_verified` re-derives identity with the new mint, `fabric_object_ref_eq` maps `ContentHashCrossFamilyIncomparable` to `false`, and the read returns `FabricObjectCorrupt` for bytes that are perfectly intact. **Observed 2026-09-21 on srv1** (`gunbc.roadmap.roadmap_dashboard_instance` `dashboard_instance_fabric_storage_root`): the store root was empty, so the population was zero at that reading; it must be re-measured when the trigger fires, and a conversion could not rewrite links alone — `gunbc.fabric.fabric_event_log` encodes the parent id **both** in the object's links and inside the serialized event body, so it has to go through the owning event codec and preserve the logical chain.
- **Head probe bound:** a head's generations are probed upward from 1 (`gunbc.durable_cas_file_store` `cas_max_generation_probe`). A partition with more appends than that bound refuses its head read, as `FabricStoreRefused` carrying `CasUnreadableObservationBoundExceeded`. The chain walk has the same bound (4096). Compaction, meaning a snapshot object and a head that starts from it, is the remedy for both, and it belongs to step 5.
