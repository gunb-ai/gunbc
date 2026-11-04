# Fabric storage

**Goal:** give fabric state one durable authority, beginning with the event log. `std.fabric_storage` owns immutable linked objects and CAS heads; `std.fabric_blob` owns bounded whole-file transfers. Metadata and bulk bytes remain separate interfaces over shared placement/transport policy.

**Method:** the file store, server, client and wire live in `gunbc.fabric.fabric_storage_*`; placement owns the host/root. An unreachable sole writer refuses rather than failing over. Linked closure reads carry the whole event chain; CAS loss carries both heads and is not a fault. Missing, corrupt, undecodable, unreachable and refused outcomes remain distinct.

The git-to-store cut starts empty only after old seat/quota leases drain; otherwise invisible old grants permit over-allocation. Retain the old log as unread history, never a fallback authority. The ensured store directory is not retracted. Tailnet reachability is not writer authorization: the missing principal roster remains an explicit admission gap.

**Blob contract:** exact admitted address, independently staged/hash-measured reading, create-if-absent, no listing/prefix fallback/overwrite/delete, and refusal above the placement bound. `gunbc.cloudflare.r2_staged_transfer` supplies transfer mechanics to origin and fabric consumers without merging their provenance. Cache placement/retention belong to `fabric_blob_placement` and the R2 lifecycle model. Age release is not freshness or a bound on peak cost. Emitted pre-compiler workflow operations must realize the same plans.

**Remaining work:** metadata replication retains one head linearization point until conditional ETag writes supply CAS. Later chunking/resume, replication, explicit head ownership, failover and unreachable-chunk collection must preserve that invariant. Generation/closure bounds refuse exhaustion; compaction needs a snapshot and new head.

**Identity gate:** the structural object locator is not a cryptographic isolation boundary. The withdrawn SHA-256 mint was blocked by measured interpreted cost and missing native structural-arithmetic emission, not a missing hash algorithm. Minting and verification stay pure; hashing a temporary file introduces custody/effect gaps. A replacement must move mint and parser together, remeasure stored population, and migrate both object links and event-body parent IDs through their owner codec. An earlier empty-store reading is not today's migration clearance. The recurring-failure rows `fabric_object_identity_is_a_structural_locator`, `bounded_natural_arithmetic_evaluated_as_unbounded_int` and `a_carrier_swap_moves_the_mint_and_strands_the_parser` retain the evidence and triggers.

**Validate:** exercise actual store consumers at admissible cost, CAS races, typed failures, linked closure integrity and identity migration; independently observe publication/readback and retention. This brief supplies no deployment or durability receipt.

[Prior design and dated receipts](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/fabric-storage.md).
