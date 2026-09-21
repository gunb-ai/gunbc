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

## Steps 4–5 (the program, not this change)

4. **R2 durability tier.** A second realization of the same interface, with objects replicated to Cloudflare R2. This needs:
   - conditional-write modeling in `extdeps.cloudflare.r2`: `If-None-Match: *` for write-once objects, and an ETag-conditioned put for a head. `extdeps.object_storage` has no conditional writes today, so R2 cannot hold a head until they are modeled.
   - a minted write credential with the scope that implies.

   The file store stays the linearization point. R2 holds objects, and eventually heads once a conditional put can serve as the compare-and-set.
5. **Colossus-like growth.** Chunked objects for large payloads, replication across hosts with an explicit metadata authority (which host linearizes which head), and failover that preserves one linearization point per head.

   The interface already leaves room: object refs are `ContentHash` (a family change is a new mint, not a new type), heads are per name (so heads can shard by name), and every failure a replicated store adds (an unreachable replica, a stale read) has an existing arm to land in.

## Known limits of the first realization

- **Cryptographic digest, interpreted only:** object refs mint through `std.content_hash` `content_hash_of_value_cryptographic` over `extdeps.crypto.sha2` (FIPS 180-4 SHA-256, witnessed by `test.claim.sha256_fips_witness_test`); `test.claim.fabric.fabric_storage_witness` `the_mint_computes_a_real_sha256_digest` pins the wire form against a digest computed outside this repository. The shell-out alternative was refused for a **structural** reason, not an economic one: `extdeps.crypto.hash` `crypto.Sha256Sum.File` takes a *path*, while `fabric_object_preimage` produces bytes **in memory**, so reaching it means a temp-file write per object — which manufactures the custody gap (hash one file, store another) that verification exists to close, and makes minting an effect. Minting stays pure, so `fabric_object_verified` stays a pure check. **The remaining limit is emission, not strength:** `UInt32` is `Compose<UInt, MachineWidth<32>>`, a structural operand, and `std.operator_realization` `operator_realization_for` routes arithmetic on a structural operand to `structural_arithmetic_refusal`, so this closure runs only under the interpreter (`gunbc.recurring_failure_mode` `bounded_natural_arithmetic_evaluated_as_unbounded_int`). A collision is refused at put (`FabricObjectCorrupt`), never read through — that arm is **not** retired now the digest is stronger, because under the old family a collision was an accident and under this one it is an attack. A collision remains a **content-identity** failure that can enable an isolation failure given authorization and sharing; tenant isolation stands on its own and never on the digest.
- **Minting can refuse, and callers see it:** the mint answers an Optional, and the refusal is propagated rather than absorbed — falling back to the structural digest would answer with a locator in exactly the case where identity could not be established. `fabric_storage_file_put` mints before touching the filesystem, so a refusal never leaves bytes on disk under no name; `fabric_storage_wire` `closure_stored` refuses a whole reply rather than filtering, because a reply one object short satisfies every check below it. The typed fault is `FabricObjectDigestUnavailable`, kept distinct from `FabricObjectCorrupt`: nothing-was-compared and the-bytes-disagree have different repairs.
- **Head probe bound:** a head's generations are probed upward from 1 (`gunbc.durable_cas_file_store` `cas_max_generation_probe`). A partition with more appends than that bound refuses its head read, as `FabricStoreRefused` carrying `CasUnreadableObservationBoundExceeded`. The chain walk has the same bound (4096). Compaction, meaning a snapshot object and a head that starts from it, is the remedy for both, and it belongs to step 5.
