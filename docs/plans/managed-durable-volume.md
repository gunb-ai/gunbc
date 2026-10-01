# Managed durable volume

The plan for the one storage product gunbc provides to an instance. The governing model is `product.fabric.durable_volume`, which carries the ack point and the writable-head epoch fence, with claims in `test.claim.durable_volume_witness`.

## Operator rulings this plan inherits (2026-10-01)

- **Launch may ship without it.** It may ship with strictly volatile local scratch plus BYO storage, shown plainly to customers. Durable volumes do not gate the first customer.
- **One storage SKU.** Any storage gunbc provides is one uniform managed durable volume. There is no persistent-local tier beside it (DESIGN §3: one name, one contract).
- **Local disk is never durable.** On a host it is cache and staging only.
- **Monthly cap.** A volume has a monthly read/write cap, weighted toward writes. The cap is part of the volume contract, metered, and enforced by throttle or refusal. It is priced against R2.

## The answer to "we can't use blob storage, we need a DB or something"

Correct: object storage alone cannot be the write path or the authority. It fails on two counts:

- **Latency.** A guest `fsync` would wait on several internet round trips to R2.
- **Cost.** Each flush would be at least one Class A operation, so cost would scale with the guest's fsync rate rather than with data volume.

R2 can hold the bodies. The write path and the authority have to be something else. The volume is therefore three parts with three different jobs.

| Part | Holds | Realization (candidate) | Moves when |
| --- | --- | --- | --- |
| 1. Write journal | acked guest writes not yet in R2 | replicated log on ≥ 2 fleet hosts, each replica fsynced | every guest flush |
| 2. Head / metadata store | volume version, chunk map, writer epoch, writer hold | strongly consistent CAS store (candidate: **etcd**, 3 members) | every chunk batch, every take-over |
| 3. Chunk bodies | immutable content-addressed chunks | **R2**, create-if-absent PUTs, in batches | every batch (seconds to minutes) |

### 1. Write journal

**Ack point:**
- **When a flush is acked:** a guest `FLUSH`/`FUA` completes when the flushed writes are appended and fsynced on **every** journal replica under the writer's current epoch.
- **Which ack is durable:** the replica-fsync ack, with readback of the appended record's identity. This is `volume_acknowledge` (#12955) with the journal append as the committed write.
- **Uncertain ack:** an append whose acknowledgement is uncertain is not acked until it is resolved by reading the replica tail.

**Placement:**
- **Replica count:** at least 2 replicas on different hosts, so the death of the guest's host leaves at least one copy of every acked write.
- **Two replicas (2-of-2):** writes stop when a replica is lost, until a replacement is seeded. That is consistent with the "may terminate, no automatic resume" default.
- **Three replicas (majority of 3):** writes keep going through one loss, at one more replica's disk and network.
- **Recommendation:** start at 2-of-2 on srv3+srv4 plus the guest host's local staging. Measure, then decide on 3.

**Fencing at the journal:**
- **Epoch check:** every append carries the writer epoch. A replica refuses an epoch lower than the highest it has accepted.
- **Take-over sequence:**
  1. the successor first advances the epoch in the head store (part 2);
  2. it then *seals* each reachable replica at the new epoch;
  3. it reads the longest acked tail.
- **Stale writer:** a writer from the killed host then cannot append. That is the model's `VolumeWriterFenced`, enforced by the replica rather than only by the head.

**Truncation:**
- **When:** a journal prefix is discarded only after the chunks it covers are in R2 and a head naming them has committed (part 2).
- **Consequence:** at every instant, an acked write is either in the journal or reachable from the committed head.

**Crash consistency and restore:**
- **State after a crash:** the volume is the committed head plus the journal tail replayed in order, which is exactly the acked prefix.
- **Restore:** a new realization acquires the hold, seals the journal at a new epoch, replays the tail over the head's chunk map, and serves. Reads fault chunks from R2 into the local cache.

### 2. Head / metadata store: are R2 conditional writes enough?

**What was read, 2026-10-01:**
- `developers.cloudflare.com/r2/api/s3/api/` lists `If-Match`, `If-None-Match`, `If-Modified-Since` and `If-Unmodified-Since` as **implemented** on PutObject.
- `developers.cloudflare.com/r2/reference/consistency/` says readers "will immediately see the latest object globally" after a write. It also says that when two clients write one key, "the last writer to complete wins". It does **not** state that a conditional PUT is evaluated atomically with the write.

**Verdict: not enough yet; not traced.** The header exists. What the head needs is that, of two concurrent `If-Match` PUTs on one ETag, exactly one commits. The docs don't promise that, and it hasn't been observed. M1 runs that race. If it holds over a large trial count, R2 could carry the head, which moves only once per batch. Even then, the writer hold and epoch need a store whose CAS is specified, not inferred.

**Candidate: etcd, 3 members on srv1/srv3/srv4.**
- **Why etcd:** it is a Raft-replicated key-value store whose transactions compare a key's revision before writing. That is the `std.durable_compare_and_set` shape, realized by an upstream modeled in `extdeps/` like R2 and curl are: a realization handler, not a replacement stack.
- **Not yet verified:** that claim about its transaction semantics is recalled from upstream docs, not read this session. M1 cites the upstream API before it is relied on.
- **Fallback:** the existing fabric DB file store (`gunbc.fabric_storage_file_store` on srv1). It is a working CAS, but one host and not replicated, so it would be the volume's single point of failure.

**Contents of the head:** the volume version, the chunk map (chunk index → chunk SHA-256), the writer epoch, and the writer hold (`std.durable_exclusive_hold`). The model's `VolumeHead` currently holds one extent; M2 replaces that with the chunk map.

### 3. Chunk bodies on R2

Chunks are fixed size and named by SHA-256 under `vol/<volume>/chunk/<sha256>`.

- **Write:** PUT create-if-absent (`If-None-Match: *`, implemented per the page above).
- **Integrity on write:** `Content-MD5`, the only body-integrity header R2 PutObject implements (`x-amz-checksum-*` is listed as not implemented). #12956 step 4 checks that R2 refuses a mismatch.
- **Identity on read:** verified against the SHA-256 after GET.
- **Batching:** a batch uploads every chunk dirtied since the last batch, then commits one head naming them. So R2 operations scale with distinct dirty chunks per batch interval, not with guest fsyncs.
- **Garbage collection:** chunks no committed head names are collected by mark from live heads, never by age. Deletes are free.

## Cost against the write-weighted monthly cap

R2 prices were read from `developers.cloudflare.com/r2/pricing/` on 2026-10-01 (Standard storage):

| Item | Price |
| --- | --- |
| Class A (PutObject, CopyObject, multipart) | $4.50 per million |
| Class B (GetObject, HeadObject) | $0.36 per million |
| Storage | $0.015 per GB-month |
| DeleteObject | free |
| Egress | free |

They need a row in `product.supplier.cloudflare_r2` before the cap consumes them.

| Part | Driver | Cost per month (formula) |
| --- | --- | --- |
| Journal | guest bytes flushed × replicas | our disk endurance and LAN bandwidth; no vendor per-operation price. Metered as bytes written. |
| Head store | batches + take-overs | our hosts. With one batch per 10 s, ≈ 2.6×10⁵ head commits/month per volume. |
| R2 chunks | distinct dirty chunks per batch | Class A: $4.50 × (Σ over batches of dirty chunks) / 10⁶. Storage: $0.015 × GB stored. |
| R2 reads | cache-miss chunks | Class B: $0.36 × misses / 10⁶ |
| Head on R2 (only if M1 qualifies it) | batches | ≈ 2.6×10⁵ Class A ≈ $1.17 per volume-month at one batch per 10 s |

**Write weighting falls out of the prices.** One Class A costs 12.5× one Class B, and every written byte is also paid in journal replicas. Bounds:

- **Worst case:** write amplification is bounded by chunk size ÷ smallest write, per batch. A guest that dirties every chunk in every batch pays one Class A per chunk per batch.
- **Effect of batching:** a longer batch interval is the lever that lowers the bill. Durability is unaffected because the journal already holds the acked data.

**Cap model (a contract term, M5):**
- **Meters:** per volume per month, journal bytes written, R2 Class A operations and R2 Class B operations.
- **Budget:** priced as one budget in dollars from the cited rows.
- **At the soft cap:** throttle. The batch interval widens and guest flush admission is rate-limited, so guests see latency, not errors.
- **At the hard cap:** a typed refusal that reaches the guest as a failed write. It is never silent below-floor delivery (DESIGN §4b).
- **Contract terms:** both limits are named terms of the one volume product.

**Latency per write class (all unmeasured; M1 measures them):**
- **Guest flush:** a LAN round trip to the replicas plus a replica fsync.
- **Batch upload:** off the guest's path.
- **Read miss:** one R2 GET.

## Precedents

None are cited. No precedent system's documentation was read in this session, so none is offered as support. The design above is derived from the model's own ack point and fence, plus the R2 pages named above.

## Milestones, sized

Sizes are bets in focused engineer-weeks.

| | Milestone | Size | Gate |
| --- | --- | --- | --- |
| M0 | Pure model: ack point, epoch fence, host-kill and stale-writer claims (#12955). R2 object proof probe (#12956). | done, in review | operator runs `full_proof` |
| **M1** | **First executable milestone, measurement only.** Four measurements, listed below. Output: measured latency per write class, a verdict on R2 for the head, and cited price and etcd rows. | 1.5–2 wk | srv1/srv3/srv4 reach, GCP token |
| M2 | Model journal (append, seal, epoch refusal, truncation-after-head), chunk-map head, batch commit and cap meters in `.dag`, each with discriminating claims. Extends `product.fabric.durable_volume`. | 2–3 wk | M1 picks journal replica count, chunk size, batch interval |
| M3 | Realization: journal replica service, host volume agent serving virtio-blk, batch uploader, restore. | 4–6 wk | **byte-path decision (risk 1)** |
| M4 | Real host-kill proof on srv3/srv4. The guest writes and fsyncs. The guest host is killed (qemu, agent and journal replica, or power through the BMC). Restore elsewhere reads every acked block. The old writer is resumed and refused at the journal and at the head. | 1 wk | M3 |
| M5 | Cap metering and enforcement live, customer-visible usage, chunk garbage collection. | 1.5–2 wk | M2, M3 |

The four measurements in M1:

- **(a)** An R2 `If-Match` race: two concurrent conditional PUTs on one ETag, repeated about 1,000 times. Exactly one must commit and the other must get 412.
- **(b)** R2 PUT, GET and HEAD latency from srv3 at 4 KiB, 1 MiB and 4 MiB.
- **(c)** etcd 3-member transaction-CAS latency.
- **(d)** srv3↔srv4 round trip plus NVMe fsync latency for journal appends.

**Total: about 10–14 engineer-weeks from M1 start, if risk 1 resolves.**

## Risks and decisions needed

1. **Byte path in the interpreter (largest).**
   - **Problem:** the journal and the agent move and hash every byte. The interpreter is measured unfit for that: one interpreted SHA-256 block costs 244,396 eval steps (`docs/plans/fabric-storage.md`, Known limits).
   - **Options:**
     - (a) M3 waits for native emission of the agent closure;
     - (b) byte movers are cataloged upstream programs bound as realization handlers (nbdkit or vhost-user-blk, sha256sum), while every decision stays `.dag`: ack, fence, seal, truncate, cap.
   - **Recommendation:** (b). **Needs an operator ruling.**
2. **Adopting etcd** is a new fleet service on three hosts. It needs operator sign-off and an `extdeps` model.
3. **R2 conditional-PUT atomicity**: not traced until M1(a).
4. **R2 Content-MD5 refusal**: documented as implemented, not yet observed. #12956 step 4 observes it.
5. **The ntfy approval loop is not wired for R2 effects** (located in #12956). Live runs use an operator-supplied token until it is.
