# Fabric DB

The fabric DB is gunbc's own store for the fabric's durable state. The operator decided on 2026-09-19 to move off git: the fabric needs its own storage anyway and will grow it toward a Colossus-like shape, with Cloudflare R2 as a durability tier. Its first consumer is the fabric event log (`gunbc.fabric_event_log`), which carries every capacity transaction: serving seats and upstream quota.

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
  - `gunbc.fabric_storage_file_store`: heads are `gunbc.durable_cas_file_store` slots, and objects are write-once files beside them.
  - `gunbc.fabric_storage_serve`: the served endpoint, one `gunbc serve` process on the placed host, loopback-bound behind `tailscale serve`.
  - `gunbc.fabric_storage_client`: the caller-side binding. It runs in process on the placed host and makes one bounded HTTPS POST anywhere else.
  - **Wire:** `gunbc.fabric_storage_wire` is the one text form every outcome crosses the hop in.
- **Policy:** `gunbc.fabric_storage_placement`, one host and one root. Nothing downstream names the host.

## Steps 1–3 (this change)

1. **Model:** the interface, as above.
2. **First realization:**
   - One writer on srv1. There is one linearization point by placement, and an unreachable placement refuses rather than failing over.
   - The store root is an *ensured* host directory (`gunbc.live_deploy.spec` `FabricDbStoreDirectory`). No retract removes it.
   - The endpoint's unit and tailscale route are owned by the srv1 live deployment.
3. **Cutover:**
   - `gunbc.fabric_event_log` runs on the fabric DB in one motion, and its git plumbing is deleted.
   - An event's parent is a store-held link, so a partition read is one closure and the full chain always arrives.
   - Consumers updated: the harness seat path, fabric quota, and both probes.

### Cutover and drain

Partitions start empty in the fabric DB, and nothing is imported from the git log. The new store never derives state from the one it replaced.

This is correct once every lease granted from the git log has expired:

- **Seats:** the longest term the fabric grants is the harness seat term, `gunbc.harness.harness_cli` `harness_seat_policy`, which is the request deadline plus the release allowance.
- **Quota:** quota leases are declared per call, and the one live caller (`gunbc.roadmap_publish_observe`) takes 120 s.

After the deploy, wait at least the seat term before trusting a seat count. A seat held under the git log is invisible to the new partition, so for at most one term a pool could over-grant by the seats still held in the old log.

The git repository at `/opt/gunbc/fabric-event-log.git` on srv1 is left in place, read-only, as history. Nothing consults it.

### Access

The backend binds 127.0.0.1 and is reached only through `tailscale serve` on srv1's port 10000. So only tailnet members reach it, and the `Tailscale-User-Login` header is exactly as trustworthy as `extdeps.tailscale.identity` states.

**Declared gap:** no principal is refused. Any tailnet member that reaches the listener may append. Restricting appends needs a modeled roster of fabric writer principals, one sufficient for the handler to refuse a login outside it. That roster is the next rung.

## Two planes

The fabric stores two kinds of thing, and they get two interfaces over one placement policy and one transport family:

- **Metadata plane:** `std.fabric_storage`. Small structured immutable objects whose preimage travels inside one request, links the store walks, compare-and-set heads, durable history. An object body is `NonEmptyStr`; it does not grow a binary constructor, because fusing bulk bytes into this wire would push every existing client through a binary rewrite.
- **Blob plane:** `std.fabric_blob`. Opaque files that live on disk on both sides of a transfer, under an exact address (an admitted cryptographic key, not the content digest). A read stages the bytes in a directory the transfer minted and mints a sealed `FabricBlobReading` (SHA-256 and byte size derived from the staged file); whatever owns the bytes' meaning verifies them from that reading. Publication is create-if-absent; there is no listing, no prefix fallback, no overwrite and no delete on the publisher's surface. Objects are whole, up to a bound the placement declares, and an object over it refuses before upload.

`std.cache_interface` already distinguishes `StructuredArtifact` from `RawBytes`, `TarArchive` and `FileTree`; the metadata plane carries the first, the blob plane the others.

## Steps 4–5 (the program)

**4A. Whole-object R2 blob realization (the build cache is its first consumer).**

- The R2 staged-file transfer is `gunbc.cloudflare.r2_staged_transfer`, extracted below the durable origin's provenance. The durable origin (`gunbc.cloudflare.r2_origin_object`, which alone mints `OriginReading`) and the blob plane (`gunbc.fabric.fabric_blob_r2`, which mints `FabricBlobReading`) both consume it; neither shares the other's provenance carrier.
- Create-if-absent object writes are **not** missing: `extdeps.tools.curl` sends a signed `If-None-Match: *` and the transfer classifies stored / already present / refused / outcome-unknown. What is still missing is the ETag-conditioned put a head needs (4B).
- Placement: the cache is a third private bucket, the `FabricCacheBlobs` purpose in `gunbc.cloudflare.r2_origin`, because its writer (one canonical publisher), its readers and its retention differ from the durable and boot buckets. It is a placement, not a third store: the interface is `std.fabric_blob`. Its layout, object bound and release are `gunbc.fabric.fabric_blob_placement`.
- Retention is physical release by age (`std.cache_interface` `ReleasedAfterInterval`), realized by an R2 lifecycle rule (`extdeps.cloudflare.r2_lifecycle`) that `gunbc.cloudflare.r2_bucket_ensure` converges with readback. It is not a freshness window: a present object stays a valid hit, and a released one is an established miss. It bounds object age only; peak bytes and cost stay unestablished until pack size and publication rate are measured.
- A run that has not built `gunbc` reaches the blob plane through emitted workflow shell that realizes the same operation plans (key, size bound, conditional write, verdicts); it does not respell them.

**4B. R2 durability for the metadata plane.** Replicate immutable metadata objects to R2; the srv1 file store stays the head linearization point. R2 holds heads only once an ETag-conditioned put is modeled and can serve as the compare-and-set.

**5. Colossus-like growth.** Multipart and chunked blobs (lifting 4A's whole-object bound), chunk manifests, resumable transfer, replication across hosts with an explicit metadata authority (which host linearizes which head), failover that preserves one linearization point per head, and collection of unreachable chunks.

The metadata interface already leaves room: object refs are `ContentHash` (a family change is a new mint, not a new type), heads are per name (so heads can shard by name), and every failure a replicated store adds (an unreachable replica, a stale read) has an existing arm to land in. The blob interface leaves the same room: an address and a reading do not change when an object is carried in chunks.

## Known limits of the first realization

- **Weak digest, and the strong one is built and measured, not missing:** object refs mint through the non-cryptographic structural family (`std.content_hash` `content_hash_of_value`). A collision is refused at put (`FabricObjectCorrupt`), never read through. The computing SHA-256 exists in the substrate — `extdeps.crypto.sha2` `sha256_hex`, FIPS 180-4, witnessed by `test.claim.sha256_fips_witness_test`. What does **not** exist is a mint over it: gunbc#11996 authored one in `std.content_hash`, bound `fabric_object_ref_of` to it, and **withdrew both on measurement**. Interpreted, every put, every verified get and every closure step pays a SHA-256, and the spark pair_serving real-execution claims that append and re-read chains through `gunbc.fabric.fabric_storage_file_store` ran two to three hundred times slower than on main, cancelling the required floor at its cap. The mint's own inhabitance claim then failed the same way for a second, independent reason: one interpreted SHA-256 over a single-block preimage performed 244,396 eval steps against the required floor's 72,300-step new-witness budget (`v2.workflow.required_floor` `claim_eval_step_budget_for_identity`), and one compression block is the floor of that cost — so the seam could not carry an executing claim either, and a declaration with no executing consumer is what DESIGN §3c refuses. The row is `gunbc.recurring_failure_mode` `fabric_object_identity_is_a_structural_locator` (rung 1, ceiling 3, the run and the differential cited by id). **The blocker is native emission, not strength:** `UInt32` is `Compose<UInt, MachineWidth<32>>`, a structural operand, and `std.operator_realization` `operator_realization_for` routes arithmetic on a structural operand to `structural_arithmetic_refusal`, so the closure runs only under the interpreter (`gunbc.recurring_failure_mode` `bounded_natural_arithmetic_evaluated_as_unbounded_int`). The trigger is stated at capability grain: native emission of that closure sufficient for every consumer of the file store to run at floor rate. The shell-out alternative was refused for a **structural** reason under either family: `extdeps.crypto.hash` `crypto.Sha256Sum.File` takes a *path*, while `fabric_object_preimage` produces bytes **in memory**, so reaching it means a temp-file write per object — the custody gap (hash one file, store another) that verification exists to close — and makes minting an effect. Minting stays pure, so `fabric_object_verified` stays a pure check. A collision remains a **content-identity** failure that can enable an isolation failure given authorization and sharing; tenant isolation stands on its own and never on the digest.
- **When the family does change, the parse changes with it and the stored population is measured first.** `fabric_object_ref_of_wire` admits exactly the family the mint produces, so the two move together (`gunbc.recurring_failure_mode` `a_carrier_swap_moves_the_mint_and_strands_the_parser`). Moving refs from fnv1a64 to SHA-256 makes any object *already stored* under the old family unreadable, and not loudly: `fabric_object_verified` re-derives identity with the new mint, `fabric_object_ref_eq` maps `ContentHashCrossFamilyIncomparable` to `false`, and the read returns `FabricObjectCorrupt` for bytes that are perfectly intact. **Observed 2026-09-21 on srv1** (`gunbc.roadmap_dashboard_instance` `dashboard_instance_fabric_storage_root`): the store root was empty, so the population was zero at that reading; it must be re-measured when the trigger fires, and a conversion could not rewrite links alone — `gunbc.fabric_event_log` encodes the parent id **both** in the object's links and inside the serialized event body, so it has to go through the owning event codec and preserve the logical chain.
- **Head probe bound:** a head's generations are probed upward from 1 (`gunbc.durable_cas_file_store` `cas_max_generation_probe`). A partition with more appends than that bound refuses its head read, as `FabricStoreRefused` carrying `CasUnreadableObservationBoundExceeded`. The chain walk has the same bound (4096). Compaction, meaning a snapshot object and a head that starts from it, is the remedy for both, and it belongs to step 5.
