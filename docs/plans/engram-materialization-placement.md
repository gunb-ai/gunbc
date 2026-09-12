# Engram placement as a fabric/materialization cost decision

**Status: plan. No `.dag` implementation lands from this document until it is approved.** The one non-prose change shipped with it is the `gunbc.doc_graph_roots` bind that makes this file reachable — a registration, not an implementation.

Operator ruling (2026-09-11): placement goes through a fabric/materialization process. **Model Engram access and cost and let the ordering fall out; do not declare it.** This is the general fabric-storage question — RAM, local NVMe, NFS, an object/CAS origin — and Engram is the first subject that makes it bite, not a special case with its own vocabulary.

The sentence this document exists to make unwritable is `engram_location = "ssd"`.

---

## 1. The question, stated so it has an answer

DeepSeek V4.1 Flash publishes a payload whose Engram tables are 203,073,076,240 B (`extdeps.deepseek.deepseek_v4_1_flash` `deepseek_v4_1_flash_safetensors_payload_footprint.engram_bytes`), against a backbone of 298,309,535,168 B. On a GB10 the unified pool is 130,663,231,488 B per host (`gunbc.spark.serving_deployment_selection` `spark_fleet_snapshot`). So the Engram tables alone are larger than one host's entire memory, and today the corpus has exactly one way to talk about that: a single scalar `ArtifactFootprint` on the whole checkpoint, which is either resident or the artifact does not fit.

That scalar cannot express the thing that is actually true — that **two components of one checkpoint have access profiles that differ by seven orders of magnitude**, and that the right home for their bytes therefore differs. The backbone is read densely every step. Engram is a sparse gather. A model that cannot say so will keep answering "how many Sparks" with an arithmetic that charges the scarcest resource on the fleet for bytes almost none of which are touched.

The decision this plan models is therefore **not** "where does Engram go". It is: *given a component's access demand and a set of access routes with measured facts, which route minimises total serving cost subject to the performance constraints* — with "cost" including the opportunity cost of the RAM a route consumes, because that is the term whose omission makes RAM look free.

## 2. What the corpus already carries (the §2 DFS, done first)

This was the failure mode the fabric already committed once — `docs/plans/fabric-concept-reconciliation.md` records four existing concepts re-minted on a clean-slate fabric branch, one of them dropping the field that was the security check. So the census comes before the vocabulary.

**An earlier revision of this table overstated four rows and the census is not closed.** Each correction is kept visible below rather than silently repaired, because the pattern — reading an authority's *shape* and assuming its *semantics* — is the thing a later reader needs to avoid.

| Concept this plan needs | Existing home | Verdict |
| --- | --- | --- |
| "where do the bytes live" | `std.realization` `Placement` = `LocalInProcess` \| `LocalFilesystem` \| `RemoteNetwork` \| **`LocalAccelerator`** | **inhabit**, with a scope gap declared. An earlier revision listed three arms and dropped `LocalAccelerator` — the arm most relevant to the all-resident route. On GB10 host and accelerator are one unified pool, so whether resident Engram is `LocalInProcess`, `LocalAccelerator`, or a topology relation between them is a real modelling question this plan does **not** settle; it is an obligation on step 2, not a silent omission. |
| tier of a materialization | `std.materialization_ladder` `ProviderTier` = `ReferenceTier` \| `CopyTier` \| `MemoTier` \| `ArtifactTier` \| `CasTier`, derived from `ProviderAxes {placement, addressing, aliasing}` via `tier_axes` | **inhabit — but a tier is not a locality.** `CasTier` means remote **and** key-addressed **and** content-keyed, with the aliasing `tier_axes` derives. An NFS pathname does not become CAS because the filesystem is remote. An earlier revision mapped "NFS or object origin = `CasTier`", which classifies by locality while silently inheriting keying semantics it never established. Each NFS route must now say whether its addressing is content-derived, pathname-derived, or supplied by an enclosing content-addressed namespace. |
| retention, capacity, replacement | `std.cache_interface` `ProviderRetention`, `CapacityPolicy`, `ReplacementStrategy`, `AtCapacityDisposition` — reached through `CacheProvider.retention` | **inhabit**, and not restated on the route (§9). |
| latency / consistency / locality class of a backend | `std.cache_interface` `CacheInterfaceFacts`, `ReadLatencyClass`, `PersistenceLocality`, `ConsistencyModel`; placement record `extdeps.cache.catalog_placement` `CacheInterfaceCatalogPlacement` | **inhabit**, with one honest gap: see §6. |
| a bounded store over a provider, with eviction | `std.artifact_store` `ArtifactStore`, `store_over_provider`, `store_put`, `StorePutReceipt` | **inhabit** — this is the per-host NVMe cache, not a new thing. |
| content identity of the bytes | `std.content_hash` `ContentHash`; `std.artifact_store` `ArtifactKey` | **inhabit at this level only.** An earlier revision also cited `std.materialization_provider` `ArtifactRequest` / `MaterializedArtifact` as an existing home. That is wrong: `ArtifactRequest` is a **closed coproduct** over `ParseModuleRequest`, `TypecheckModuleRequest` and `ResolveClosureRequest`, each fixing its own output roster and key derivation. Engram cannot inhabit it unchanged. Either a genuine generic component-materialization request is minted at the proper authority — which is new modelling with its own §3 justification, not a citation — or this plan uses the lower-level `ContentHash`/`ArtifactStore` vocabulary honestly. It now does the latter. |
| choosing among candidates | `std.decision` `select_realization`, `SelectionReceipt`, `HardConstraint`, `RealizationSelectionResult`; `std.pareto` `SelectionAxis` / `ParetoEntry` / `AxisGap` | **inhabit**, returned unchanged (§7). |
| durable origin, read back and digest-verified | `gunbc.fabric_m0_origin_readback` `OriginReading` / `read_origin_staged_file` (sole constructor) | **inhabit**. Every route's chain begins at this origin. |
| citation standing of an upstream figure | `extdeps.external_authority` `CitedFigureStanding` = `CitedToAuthority` \| `TranscribedUncited` | **inhabit** (§4d). |

**The net-new concept count is therefore open, not "exactly three".** §9 lists what is currently believed necessary; the corrections above add at least the component-identity trio of §3 and the two access carriers of §4, and the generic-request question is unresolved. A plan that reported a closed count here would be asserting the one thing this census just proved it had not checked.


## 3. The component: one identity — and what "the same digest" can actually mean

Engram is **one semantic serving component**. RAM-residency, a per-host NVMe copy, an NFS-backed read and an object-store origin are four *access realizations* of it. They are not four artifacts, they do not get four identities, and a route change is not a checkpoint change.

This is the structural reason `engram_location = "ssd"` is forbidden: a location string attached to the component makes placement a property *of the artifact*, so two placements become two artifacts, and nothing can then state that they serve the same thing.

**But an earlier revision said "every route serves the same digest", and that sentence was doing more work than it can bear.** If RAM, safetensors on disk, and an SSD row store repacked for gather have different physical encodings, they do not share one byte digest — they are one semantic component with *different realization digests*. Claiming a single digest across routes is either false, or it silently presupposes a canonical physical encoding that no one has defined. Three carriers, not one:

```
EngramComponentSubject        exact model revision + exact tensor population
MaterializedEngramArtifact    encoding/layout + physical content digest
FaithfulComponentRealization  the relation proving this artifact realizes that subject
```

Identity is the *subject*; each route carries a `MaterializedEngramArtifact` whose digest is its own, and the route is admissible only when a `FaithfulComponentRealization` relates the two. "Digest-verified at a declared point in the chain" (§5) then means verified against **that route's** artifact digest, with faithfulness to the subject established separately — which is the honest version of what the earlier sentence was gesturing at.

**And the subject does not exist yet.** `extdeps.deepseek.deepseek_v4_1_flash` `deepseek_v4_1_flash_engram` carries config dimensions and `..._safetensors_payload_footprint.engram_bytes` carries aggregate bytes. Neither is a digest over an exact tensor population. Producing one is step 1 work, not a citation.


## 4. Access demand: logical demand and physical access are two facts, not one chain

**Never infer access cost from total file size.** That rule survives. What does not survive is an earlier revision's single chain from config straight through to device IOPS, which asserted as derived a series of steps that are a *scenario under named assumptions*.

### 4.1 Logical demand — derivable, and derivable from more than the config

The reference implementation defines `n_hash_cols = (max_ngram_size - 1) * n_heads`. With the published configuration — `max_ngram_size: 4`, `n_heads: 8`, `layer_ids: [1, 14]` — that is 24 hash IDs per Engram layer and **48 logical hash IDs per token** across the two layers.

That figure is **implementation-plus-config derived**, not config-only as an earlier revision claimed. The arithmetic `2 × 3 × 8` happens to reach the same number, but it is not the reference's own formula, and a reader who believed the config alone established it would have no reason to re-check when either the config or the implementation moved.

### 4.2 Physical access — a route property, and currently unmeasured

An earlier revision continued: 48 rows × 264 B → 48 device pages → 196,608 B/token → 15.5× amplification → 19,200 IOPS at 400 tok/s → 307,200 at 6,400. **None of that is derived.** Each step assumes something not established here:

- **264 B is a summed logical payload, not a layout fact.** 256 fp8 values plus 8 scale entries — but the reference stores `weight` and `scale` as *separate tensors* with separate `F.embedding` operations. There is no evidence of one physically contiguous 264-byte record. A route could repack them into one, and that would be a **route-specific materialization format**, which is a different claim from an upstream layout property.
- **A sub-page payload does not imply one page read.** Value and scale may sit on different pages; a row may straddle a boundary; several requested rows may share a page; pages may already be in the page cache; the kernel or adapter may coalesce, batch or read ahead; the host's page size is not established here.
- **Tensor parallelism changes the demand.** The reference shards rows across `world_size`, maps off-rank IDs to local row zero, zeroes them afterward and `all_reduce`s. Masked local-row-zero reads and ownership routing are physical demand this model has not accounted for — and they are one of the couplings §7 now takes seriously.

So the split is structural, and the standing lives *in* the carrier rather than beside it:

```
ComponentLogicalAccessDemand    hash_ids_per_token = 48
                                evidence: derived from exact implementation + config

RoutePhysicalAccessStanding     physical reads, bytes transferred, unique pages,
                                page-cache hits/misses, coalescing/readahead,
                                per-rank ownership
                                evidence: Unobserved { obligation } | Measured { route, profile, receipt }
```

The `Unobserved` arm must project to an `AxisGap` in every selection projector. That is why the 264 B figure's standing and its number have to be **inseparable**: a bare `ByteSize` beside a separate prose standing row is exactly how an inference gets consumed as established, which is the failure §4d names and which this document has already committed twice.

What survives as useful is the **ratio — conditionally**. 48 logical row reads against a 203,073,076,240 B table is ≈ 6.2 × 10⁻⁸ of the component per token, but that figure is (48 × 264 B) / 203,073,076,240 and therefore rests on the **same 264 B payload assumption** §4.2 just declined to grant. It is a logical-byte *illustration* under that assumption, not an assumption-free fact: from "48 row reads" and a byte count alone, no ratio of bytes follows. It motivates asking the question; it cannot fund an axis.

Reuse locality is workload-dependent — a token's rows are selected by its n-gram context — and stays a typed gap rather than an estimate. Max admitted added latency is **not** a field here: it belongs to the serving subject and reaches this decision as context or a hard constraint (§7).


## 5. Routes are chains, and NFS must say which role it plays

A route is an ordered chain from a durable origin to the bytes the runtime touches at token time. Four candidates:

- **A — origin → RAM-resident.** The status quo. Engram is a parameter; the tables occupy the unified pool.
- **B — origin → per-host NVMe (`ArtifactTier`) → runtime page cache.** Fill once per host; steady-state reads are local-disk latency; RAM cost is whatever the page cache actually retains, which is a *measured* number and not the table size.
- **C — origin/NFS → direct network-backed token-time access.** No local copy. NFS is on the serving path for every gather.
- **D — origin/NFS → per-host NVMe → page cache.** NFS is the fill source only; token-time reads never leave the host.

**C and D are different arms and the distinction is the point.** NFS carries three separable roles — correctness-bearing durable origin, site-level fill cache, and token-time serving path — and a route must state which it plays. The published third-party figures (2–3 ms/step on local NVMe, 5.9–7.8 ms/step on NFS) are exactly the C/B separation, and collapsing them into "NFS works" would price D at C's latency for no reason.

D is the *expected* leader. It does not become the selected route until this fleet measures it, and the plan's value is destroyed if that expectation is written as the answer.

**FABRIC-M0 wall holds unchanged: one durable origin, no peer mesh.** A route may not name another serving host as a fill source. Every chain's head is the `gunbc.fabric_m0_origin_readback` origin, digest-verified at a declared point in the chain — which is itself a route fact, since verifying at fill and verifying at every token-time read are different costs and different guarantees.

## 6. Provider-route facts, and the one gap the census found

Per route, per provider hop: capacity; retention and replacement (`std.cache_interface`); fill bandwidth; steady-state latency and throughput; **p50/p95/p99 under the concurrency the workload actually generates**; RAM consumed by page cache; network bytes and contention; replication count; failure domain; recovery/refill cost after a host loss; and the point in the chain where the digest is verified.

The census gap: `CacheInterfaceFacts` carries `ReadLatencyClass` as a four-valued *class* (`InProcessNs` / `LocalDiskUs` / `LanMs` / `WanTensMs`). That is the right vocabulary for choosing a cache backend and it is **too coarse for this decision** — B and D both read `LocalDiskUs`, and the 2–3 ms vs 5.9–7.8 ms separation that decides C is invisible inside `LanMs`. This is a *scope* gap on an existing home, not a missing home, so the repair is to carry measured distributional readings on the route row (which is a receipt about this fleet) and to leave the class on the backend row (which is a property of the backend kind). The two are not the same fact and must not be merged: one is a taxonomy, the other is a measurement.

## 7. Selection: total serving cost, and the RAM term that makes it honest

No bespoke score. The route field is handed to `std.decision` `select_realization` over `std.pareto` axes exactly as `gunbc.spark.serving_deployment_selection` already does, with **every cost dimension declared as a funded axis or present as an `AxisGap`** — the discipline that module already enforces, and the reason an unmeasured route cannot win.

Cost terms, each an axis:

- amortized fill cost (origin → local tier, per host, per deployment lifetime)
- steady-state access cost at the workload's concurrency
- **RAM opportunity cost**: bytes of unified pool consumed, priced in what they displace — backbone weights, KV cache, concurrent sequences. This is the term whose absence makes A look free, and it is the reason the ordering is *deduced* rather than asserted.
- storage cost, network bytes, replication count
- churn / refill cost on host loss, and failure-domain breadth

Hard constraints, screened *before* Pareto (§3d: an attractive candidate violating a hard constraint is excluded, never ranked): fits the pool at all; digest verified at a declared point; added per-step latency within the serving subject's admitted envelope; the runtime can actually serve the component from that tier (§11).

**Result vocabulary: this subject returns `RealizationSelectionResult` itself.** An earlier revision proposed a three-arm spelling — selected / needs-evidence / refused — and called it "the three arms of `RealizationSelectionResult`". It has **six**: `SelectedWithin`, `CandidateFieldUnestablished`, `NoFeasibleRealization`, `SelectionNeedsEvidence`, `SelectionNeedsPolicy`, `RealizationSelectionRefused`. Collapsing them is not a spelling choice, it is a second algebra with fewer distinctions, which is exactly the §3d fork this plan claims elsewhere to avoid.

The dropped arm that bites hardest here is **`SelectionNeedsPolicy`**, and this subject is likely to reach it. B and D can both be *fully measured* and still non-dominated — D wins on fill (the site cache is already near the hosts) while B wins on network bytes and failure-domain breadth — and no further measurement resolves that. Under the three-arm vocabulary such a field has nowhere honest to go: it either reports `NeedsEvidence` for evidence that exists, or picks a winner off a front of two, which §3d names as the thing the law exists to prevent. `NoFeasibleRealization` is the second live arm — see §11 — and it is distinct from a refusal, because "every candidate violated a constraint" is not "the selection itself was defective".

So the subject adds no result type: it returns `RealizationSelectionResult<MaterializationRoute>` directly. A domain-specific spelling would be admissible only if its projection were total and distinction-preserving, and the simplest way to guarantee that is not to have one. The receipt is `SelectionReceipt`, unchanged.

### 7.1 Placement and topology are coupled — there is no context-free route winner

An earlier revision closed §10 with "two selections, two receipts, one direction of flow, no joint search space." **That factorization was asserted, not established, and it is wrong.** The plan's own objective terms depend on the serving topology and execution profile:

- fill cost is **per host**, so it scales with the participating host count;
- the Engram table is **row-sharded across ranks**, so per-rank demand and ownership routing depend on the degree (§4.2);
- local storage quantity and refill scope depend on how many hosts hold a copy;
- retained page-cache memory depends on route, concurrency and access pattern;
- failure-domain cost depends on host assignment;
- the route's latency requirement depends on the exact serving profile.

And the coupling runs the other way too: placement changes the resident-memory lower bound, and therefore which topologies can enter the field at all. Selecting one global route *before* topology can discard a route that is preferable at one topology and inferior at another, or price a route against no concrete host population. §3d requires the complete decision subject and its load-bearing variables; settling a coupled variable outside the candidate field needs an established invariance relation, and **this plan has none and does not claim one**.

Two shapes are sound. Either one complete candidate field over `(materialization route, topology, execution profile)`; or **route selection parameterized by each exact serving candidate**, producing a route result and receipt *for that candidate*, with the outer serving selection then ranking the composed candidates. This plan takes the second: it keeps the two authorities distinct — `gunbc.fabric.engram_materialization` still owns route selection, `serving_deployment_selection` still owns the deployment — without pretending a context-free route winner exists. The consequence is that "the selected route" is always *the route selected for a named serving candidate*, and a receipt that does not name one is incomplete.

## 8. The deduction, worked — including why it does not conclude today

The arithmetic that makes the ordering fall out, stated so it can be checked and so its rungs are visible.

Usable pool per host at the 82% memory fraction the canary route uses: 130,663,231,488 × 0.82 = 107,143,849,820 B (99.8 GiB).

**Under two stated assumptions — that loaded bytes equal publisher payload bytes, and that a non-resident route adds no resident working set (§10 shows the second is false as an assumption and must be measured):**

- **Route A (all-resident).** Whole payload 510,286,023,000 B ÷ usable = 4.76 → the first candidate degree **not ruled out by the lower bound** is TP8, since 5 is not expressible for an 8-head layout.
- **Routes B/D (Engram off the pool).** Non-Engram payload 307,212,946,760 B ÷ usable = 2.87 → the first degree not ruled out is TP4.

The phrasing is deliberate and matches the authority: `weight_fit_of`'s positive arm is named `WeightsNotRuledOutByLowerBound`, not "fits". This is a lower-bound screen against a **cited** payload (§8.3), so "Route A needs TP8" and "B/D fit four Sparks" are both overstatements. What the arithmetic supports is that the two routes are not ruled out at *different* degrees, which is enough to motivate the decomposition and not enough to conclude anything.

### 8.1 The payoff is feasibility against the admissible host set, not a replica axis

An earlier draft of this section read the 8-vs-4 rank difference as "one replica versus two" and landed it on `ax_independent_replicas` and `ax_replica_capacity_share_lost`. **That reading is wrong and the correction matters more than the arithmetic.** The operator's standing ruling is **no redundancy**: replica count is pinned to 1 by `gunbc.spark.serving_deployment_selection` `constraint_no_redundancy`. Under that constraint there is no second replica for a placement decision to buy, and an axis that counts replicas reads identically for every candidate — a permanently-flat axis, which is a decoration, not a funded dimension.

With replica count fixed at 1, the difference is **whether the shape fits the hosts that are actually available at all**:

- Route A: at the first degree not ruled out, TP8 × 1 — a host demand that `host_commitment` may or may not be able to meet (§8.2).
- Routes B/D: TP4 × 1 — a *smaller* host demand, which is the whole point of the comparison. **Whether that demand can be met is a separate question this document does not answer** (§8.2): investigating four-rank candidates does not establish that four hosts are available or acquired, and the resident working set (§10) and an actual fleet byte read (§8.3) both still stand between this and a feasibility claim.

So placement is not trading a redundancy property against a latency property. It decides **feasibility inside a host set the fabric, not this decision, controls** — which is a hard constraint screened before Pareto, exactly where §7 puts things that can make a candidate unservable rather than merely expensive.

### 8.2 How "admissible hosts" enters, and the silent assumption it removes

This is the part the corrected reading forces into the open, because **`spark_fleet_snapshot` carries `host_count: 8` and no host identities.** A bare count cannot distinguish eight free hosts from eight hosts of which four are spoken for, so any fit test run against it silently assumes the whole fleet is available — generous in precisely the direction §5 forbids, on the screen whose job is ruling shapes out.

The fleet already says otherwise — but **not for the reason an earlier revision of this section gave.** That revision named srv7/srv8 as held by an undischarged llama.cpp RPC peer reservation. On this revision that is false: `gunbc.spark.llama_cpp_rpc_observed` `llama_cpp_rpc_release_receipts` carries a receipt for **both** hosts (read 2026-09-05: no llama-server, no rpc-server, nothing on 30000 or 50052, 4 GB used of 121 on each), and `test.claim.spark_llama_cpp_rpc_witness` pins `length(llama_cpp_rpc_undischarged_peer_reservation_hosts()) == 0`. The reservation is discharged. That revision read a stale annotation in `gunbc.spark.cell_role` instead of the declarations the witness actually executes against, and the annotation's own sentence — that the count moves from four to six on a release receipt — described something that had *already happened*.

The correction matters more than the fix, because the wrong authority was hiding behind a right number. Applying the formula that revision stated — roster minus role assignments minus undischarged reservations — yields **six** admissible hosts, not the four it claimed. The count was right by accident; the formula that produced it was wrong, and a worker handed it would have built the wrong binding.

What actually binds srv5–srv8 today is a **live serving deployment**: `gunbc.spark.pair_serving_desired` `spark_pair_serving_groups = [FabricGroupA]`, and Group A is srv6 at the head with srv5, srv7 and srv8 as workers, tensor-parallel four, vLLM — pinned by `test.claim.spark_pair_serving_desired_witness` `group_a_is_four_ranks_headed_by_srv6_at_their_rail_addresses`. So:

- **srv5–srv8** are bound as the four ranks of Group A. `gunbc.spark.cell_role` `spark_cell_role_assignments` names only srv5 and srv6 as `SparkServingCell`, which is why reading the roles alone undercounts: **cell-role assignment lags the pair-serving desired state**, and that module's own annotation says roles land when pair-serving is promoted. The desired state is the authority; the role roster is downstream of it.

**The repair is to consume a shared authority, not to restate a subtraction.** An earlier revision of this section described a recipe — roster minus desired-deployment bindings minus role assignments minus undischarged reservations — and even re-deriving that live would still miss a category: it has no term for a serving-arm claim. Enumerating exclusion categories in this document is the defect, not the particular enumeration, because every consumer that enumerates them independently is a place the next category goes missing.

So the fit screen **consumes `gunbc.spark.host_commitment`** (in flight at this writing; not yet on `origin/main`, so cited as in flight rather than as a resolvable symbol). It joins the unit roster, cell roles, claimed serving groups and held reservations, and derives the admissible set in production — one authority, one place a new category lands, every consumer correct at once.

**And this document does not state the number, in any revision.** An earlier revision said "four hosts — srv9–srv12". That was already wrong within hours — srv9 took the GLM canary — and the count has moved again since. **The replacement is not a fresher number.** Writing today's figure would repeat the mistake in a newer costume and would be stale by the same mechanism; the count is whatever `host_commitment` derives when asked. That it changed twice while this document was being written is the argument, and whether the supply is sufficient is an operator decision this plan does not resolve. The count is whatever `host_commitment` derives when asked, and the durable lesson is the one §8.2 already contained and this section already broke — **the host list must not be carried here as data.** A shape needing more admissible hosts than exist fails a hard constraint and is excluded before Pareto; if that empties the field, the result is `NoFeasibleRealization` with the exclusions named (§7), never a silent fit against 8.

Separately from host fit, and whatever `host_commitment` makes available: route A's price for 189 GiB of the scarcest resource on the fleet buys 12.4 KiB/token of access. The RAM opportunity-cost axis of §7 is what states that; without it, A is simply "faster" and wins.

### 8.3 Why it still does not conclude

Missing, and tracked as the obligations §12 carries: this fleet's measured per-step added latency for B, C and D; the page-cache RAM a B/D route actually retains; the row-size ground read; the workload's reuse locality; and the §11 runtime question, which is a hard constraint and can refuse every non-resident route outright. Today the honest output is **`SelectionNeedsEvidence`** — the runtime axis is *declared and funded*, and what is missing is the candidate reading on it (§11.1). An earlier revision said `RealizationSelectionRefused`, conflating two different `std.decision` mechanisms; §11.1 now states the four states exactly, and the refusal arm is reserved for a defective selection input rather than an under-evidenced one.

No automatic progression follows from a probe existing. What the result becomes depends on **what the probe observes**, plus the remaining evidence and the other candidates — an instrument landing is not an answer arriving. A plan that shipped `SelectedWithin` off published third-party figures would be the fabricated-plausible-output failure with a receipt attached.

**Rung honesty (§4b) for each claim above.** The fleet pool is a read fact (`spark_fleet_snapshot.how`: `torch.cuda.get_device_properties` inside the serving container on a GB10). **The byte partitions are not.** An earlier revision of this paragraph called them `FootprintObservedOnFleet`; they are publisher figures summed from the safetensors index headers, and `extdeps.deepseek.deepseek_v4_1_flash` `deepseek_v4_1_flash_payload_footprint_standing` types them `CitedToAuthority` with an annotation that says so in capitals — "PUBLISHER-REPORTED, NOT FLEET-READ" — and a dissolution naming `FootprintObservedOnFleet` as a thing that does not yet exist for this revision. The only `FootprintObservedOnFleet` nearby is a **different artifact** (`artifact_deepseek_v4_flash_native`, 168 GB, V4 rather than V4.1). Claiming the stronger arm was rung inflation, in the paragraph whose job is rung honesty.

**And the correction has teeth, which is why it is not just a label fix.** `weight_fit_of` returns `WeightFitUnderivable` on `FootprintReportedExternally` — deliberately, so a number from someone else's deployment cannot support a feasibility verdict here. So §8's rank arithmetic, run through the screen that actually exists today, does **not** yield 5 ranks and 3 ranks: it yields *underivable*, for every route. The 5-vs-3 split is a consequence of publisher figures and is offered as motivation for the decomposition, not as a feasibility finding. A Spark read of the exact tensor byte spans can establish the fleet-read payload input. It does not establish loaded resident memory, route working set, or execution feasibility. The lower-bound screen's positive result remains `WeightsNotRuledOutByLowerBound`. That read would also ground §4's 264 B row, which is why §14 acquires both in one pass. The 48-reads/token count is **implementation-plus-config derived** (§4.1), not config-only — the reference's own `n_hash_cols` formula is half of it. The 264 B row is an inference with a stated read obligation. The rank arithmetic is a consequence of a cited figure and the declared memory fraction, so it inherits the weaker of the two. The pair-serving group binding and the role assignments are declared fleet state, **read at this revision** — §8.2 records an earlier revision of this section getting that read wrong, which is why the binding must derive them live rather than inherit this paragraph's transcription. The admissible-host derivation is **proposed, not built** — today no screen consumes it, which is why §14 lands it. The ordering of routes is **unestablished** and typed as such.

## 9. Homes, per new concept

| New concept | Home | Why not an existing carrier |
| --- | --- | --- |
| `EngramComponentSubject`, `MaterializedEngramArtifact`, `FaithfulComponentRealization` (§3) | subject in `extdeps.deepseek.deepseek_v4_1_flash`; artifact and realization relation in `gunbc.fabric.engram_materialization` | The corpus has `ContentHash` and `ArtifactKey` but nothing relating *one semantic component* to *several physical encodings of it*. `ArtifactRequest`/`MaterializedArtifact` cannot host this: they are a closed coproduct over compiler artifacts (§2). |
| `ComponentLogicalAccessDemand` — hash IDs per token, granularity, sparsity, reuse (typed gap), concurrency (§4.1) | `gunbc.fabric.engram_materialization` | Nothing in the corpus states *how a consumer touches* bytes. `FrameDemand` names a computation's identity and nature; neither it nor `ArtifactRequest` carries an access profile. **Max admitted added latency is not a field here** — an earlier revision put it here while §4 said it belongs to the serving subject, which made the fabric layer a second authority for serving policy. It arrives as decision context or a hard constraint. |
| `RoutePhysicalAccessStanding` — physical reads, bytes, unique pages, cache hits/misses, coalescing, per-rank ownership; `Unobserved` or `Measured` (§4.2) | `gunbc.fabric.engram_materialization` | Physical access is a property of a route on a profile, not of the component. Folding it into logical demand is what let an earlier revision present a scenario as derived. |
| `MaterializationRoute` — an ordered chain of hops, each identifying a **`CacheProvider` and a role**, head-anchored at the M0 origin | `gunbc.fabric.engram_materialization` | The ladder models a *provider*; nothing models a **chain** of them, which is what a fill-path-plus-serving-path is. An earlier revision stored `(Placement, ProviderTier, retention, role)` per hop — but `tier_axes` already *derives* placement from the tier and `CacheProvider` already carries retention, so restating them made contradictory hops writable (`ArtifactTier` beside `RemoteNetwork`). The hop now identifies the provider and derives the rest: construction rather than validation. |
| `RouteResidentMemoryStanding` — observed / bounded / unestablished (§10) | `gunbc.fabric.engram_materialization` | No existing carrier states how much RAM a *route* retains as working set, which is distinct from what a component occupies. |
| `RoutePerformanceReading` — measured p50/p95/p99, fill bandwidth, retained page-cache bytes, at a named launch | `gunbc.fabric.engram_materialization` (receipt side) | §6: `ReadLatencyClass` is a backend taxonomy, deliberately not a fleet measurement. Merging them would put a reading inside an `extdeps` classification, a layer inversion. |

**This roster is what is currently believed necessary, not a closed count** (§2). The generic component-materialization request question is unresolved, and the `LocalInProcess`/`LocalAccelerator` modelling of resident bytes on a unified pool is an open obligation.

Component facts stay in `extdeps.deepseek.deepseek_v4_1_flash`. Backend/provider kind facts stay in `extdeps.cache` / `extdeps.storage`. Fleet readings are receipts in the observing layer, never properties authored into an upstream module (§3, external upstream decomposition).

The module is named for the domain, not the model: `engram_materialization` is the first *subject*, and the carriers are generic over component and route. If a second component (a vision tower, a draft model, a KV offload tier) asks the same question, it supplies its own demand row and consumes the same fold. That is the operator's "this is the general fabric-storage question" made structural rather than promised.

## 10. Integration: per-component placement without forking `ArtifactFootprint`

`gunbc.spark.serving_deployment_selection` carries `ArtifactFootprint` = `FootprintObservedOnFleet` | `FootprintReportedExternally` | `FootprintUnread`, and `weight_fit_of` divides it by tensor-parallel degree against the usable envelope. That vocabulary is correct and **is not forked**. The distinction it draws — read on this fleet vs cited vs unread — is exactly the distinction per-component bytes need.

The change is that the quantity fed to the fit screen becomes **the resident share implied by the selected route**, not the whole payload:

- `ModelArtifact` gains a component decomposition whose entries each carry their own `ArtifactFootprint` — the *same* type, one level down. The scalar whole-artifact footprint becomes the sum, so nothing that reads it today changes meaning.
- Each component carries a placement standing: resident, or placed by a route selected in `gunbc.fabric.engram_materialization`.
- `weight_fit_of` sums only the resident components. `FootprintUnread` on any resident component still yields `WeightFitUnderivable`, unchanged.
- A component placed on a non-resident route contributes **its route's resident working set**, which is not zero and is not currently known. An earlier revision wrote "zero resident bytes", and that is a fail-open term of exactly the kind §5 forbids: routes B and D depend on a retained page cache *by construction*, and direct NFS may populate one too. Unknown working-set memory silently becoming zero deletes the precise RAM term this plan insists must be measured — and it is what would make TP4 look feasible. The fit path needs a route-specific standing:

  ```
  RouteResidentMemoryStanding
    = ResidentMemoryObserved   { bytes, receipt }
    | ResidentMemoryBounded    { upper_bound, authority }
    | ResidentMemoryUnestablished { obligation }
  ```

  The resident sum is static resident components **plus** each route's cache/buffer working set, and the `ResidentMemoryUnestablished` arm keeps the weight-fit and host-feasibility results **open** rather than optimistic.
- It also carries one hard constraint — the runtime-capability constraint of §11 — so a route the runtime cannot serve does not quietly shrink the footprint.

For #11013's candidate field, the flow is the §7.1 parameterized shape rather than the two-independent-selections one an earlier revision closed with. Each serving candidate — topology and execution profile — is the **context** a route selection runs against, producing a route result and `SelectionReceipt` *for that candidate*; the outer serving selection then ranks the composed candidates. So there is no free-floating "the selected route": a route result that does not name the serving candidate it was evaluated under is incomplete, because §7.1's couplings mean the same route can price differently at a different degree.

What follows from that, rather than from a global route choice: a candidate's resident sum includes its route's `RouteResidentMemoryStanding`, the rank count follows from that sum, and with `constraint_no_redundancy` pinning replica count to 1 the rank count **is** the host demand — screened against the admissible set of §8.2, never against `spark_fleet_snapshot.host_count`, which is the one place this integration could otherwise reintroduce the eight-free-hosts assumption, since `fleet_fits` today compares `hosts_consumed` directly to that scalar.

Two authorities remain distinct — `gunbc.fabric.engram_materialization` owns route selection, `serving_deployment_selection` owns the deployment — without either pretending the other's variable is settled.

## 11. The open fact, as an obligation with a hard constraint attached

**Can the vLLM image published as `deepseekv41-flash-0909` — pull reference `sha256:d84a1232…`, `gunbc.spark.v41_published_image_observation` `v41_published_arm64_digest` — serve Engram from a non-resident tier at all?** (On digest grain, see §11.1: the pull reference is not the execution subject.)

This plan does **not** settle it, and says so rather than inferring an answer. What is known: `extdeps.deepseek.deepseek_v4_1_flash` `deepseek_v4_1_flash_config_declares_engram_off_accelerator` is `false` — off-accelerator placement is not a publisher config key, and the reference implementation allocates the tables as parameters. A third-party 4-Spark build **patched vLLM** to do it, and an earlier revision read that patch as "positive evidence that the stock image does not". **That inference is withdrawn.** Someone patching *their* build observes nothing about the published digest this fleet would run: they may have started from a different base, needed unrelated changes, or not tried the stock path. The patch is a reason to ask the question; the two gates below establish the answer for the exact image and route.

It becomes a **hard constraint**, not an axis: if the runtime cannot serve the component from a non-resident tier, B, C and D are excluded before Pareto. Wiring it as an axis would let a cheap-but-unservable route rank.

**What that does not mean is "then the answer is A",** which an earlier revision said. Route A is subject to the same host-fit screen. A candidate requiring more hosts than `gunbc.spark.host_commitment` makes available is excluded. If the runtime and host-fit constraints together eliminate every candidate, the result is `NoFeasibleRealization` with the causes named — not A by elimination. A surviving candidate is one that passed every constraint, never the last one standing after the others were excluded; treating elimination as selection is how a screened-out candidate gets served anyway.

That state is not hypothetical, and it is the most useful thing this subject can say: it would mean the deployment does not fit this fleet under any placement, and the reply is to change a constraint — free admitted supply, patch the runtime, widen the envelope — rather than to pick a route. Which constraint is cheapest to move depends on what is committed at the time, and is not a property of any route.

### 11.1 A capability probe cannot discharge this constraint — two gates, not one

An earlier revision briefed a single `RuntimeCapabilityProbeRow` as the thing that "moves the hard constraint from unfunded to funded". **That module says otherwise about itself**, in `v41_prerequisites_are_not_feasibility`: a positive capability reading "rules the image IN as a candidate and establishes nothing about serving", and executable feasibility "belongs to `gunbc.spark.serving_deployment_selection` `RuntimeFeasibility`, whose witness is a launch receipt rather than a symbol lookup". Briefing the inventory as the gate would have used an authority against its own stated meaning — the plan citing a home while ignoring what the home says it is for.

So two gates, and only the second satisfies the constraint:

```
NonResidentEngramMechanismStanding      absent     -> excluded by the hard constraint
                                        unobserved -> SelectionNeedsEvidence
                                        present    -> prerequisite only; launch proof still required

RuntimeFeasibility (exact route × profile)   engine starts
                                             exact checkpoint loads
                                             the exact materialization route is observed
                                             a request completes
                                             resident-memory and access receipts recorded
```

The asymmetry is the useful part: *absence* is decisive and cheap, *presence* is not decisive at all.

**The four missing-evidence states, stated once so they stop drifting.** `std.decision` separates two mechanisms that an earlier revision of this plan collapsed: `unfunded_constraint_axis_defects` fires when a hard constraint names an axis **absent from the declared axes**, while `unread_axis_gaps` fires when a declared axis has **no candidate reading**. The first is a malformed selection input; the second is an evidence need. So:

| situation | result |
| --- | --- |
| constraint and axis declared, mechanism observation missing | `SelectionNeedsEvidence` |
| mechanism present, exact-route launch evidence missing | `SelectionNeedsEvidence` |
| a necessary mechanism established **absent** | exclude that candidate by the hard constraint; if the field empties, `NoFeasibleRealization` |
| a hard constraint naming an undeclared or unfunded axis | `RealizationSelectionRefused` — the selection input is defective |

**And the runtime axis is declared on purpose.** Omitting it so that an under-evidenced plan reports `RealizationSelectionRefused` would buy a louder-looking arm by making the selection genuinely malformed — preserving the six arms' spelling while destroying their meanings, which is the fork §3d forbids wearing the shape of compliance.

**Execution identity and observation lookup key are two different things, and an earlier revision merged them.** It said the capability rows are keyed to the observed local config digest. They are not, and the merged tree says so: `test.claim.spark.serving_runtime_capability_probe_witness` `the_published_ask_is_keyed_on_the_pull_pin_not_the_config_id` asserts that the published architecture observation is keyed on the **pull pin** `vllm/vllm-openai@sha256:d84a…`, that the config ID is a *different* hash, and — the load-bearing clause — that `architecture_observation_exists(image_reference: config_id)` is **false**.

So querying the pin-keyed inventory with a config ID returns a miss, and **reading that miss as "the image does not support this" would manufacture a measured negative out of a wrong lookup key.** The earlier text would have instructed a worker to do exactly that.

The relation to keep, all four terms:

```
requested repository@manifest-digest   the pull pin — the OBSERVATION LOOKUP KEY
inspect readback                       the step relating one to the other
observed local config ID               what the container actually is
exact container execution + receipt    the EXECUTION IDENTITY
```

A new non-resident-mechanism receipt may name the config ID it actually executed — it should — but it must retain the observed relationship back to the requested pin, and it must never query pin-keyed inventory by config ID. Both facts stay; neither substitutes for the other.

With the axis declared and the mechanism unobserved, today's result is `SelectionNeedsEvidence` naming that gap — the first row of the table above.

Note also that a GB10's host and GPU are **one unified pool**, so "host memory offload" adds no capacity here; the only routes that free bytes are ones that reach a *storage* tier. That is why B/C/D are the arms and a host-RAM arm is not.

## 12. Evidence loop

Predict, then measure the same subject, then join. Per route the selection predicts an amortized fill cost and a per-step added access cost; the first real load emits a receipt observing bytes moved, the route actually taken, measured access performance, and the serving behaviour that resulted — **joined to the exact launch**, at the profile grain `serving_deployment_selection` already established (artifact × topology × context ceiling × memory fraction × runtime revision × KV layout × speculative policy), because a receipt acquired under a different profile is not transportable.

The prediction and the measurement are separate rows with a stated relation, so a divergence is a visible fact rather than an overwritten estimate. Each carries its rung: the prediction is derived-from-declared-facts; the receipt is a fleet reading; the third-party figures are `CitedToAuthority` about someone else's hardware and can never be either.

## 13. Non-goals, and the walls that stay up

- No materialization-ladder refactor. The ladder's tiers already span this; §2 lists the mapping.
- No peer mesh. FABRIC-M0 holds: one durable origin, chains anchored there.
- No second decision algebra, no score, no weights (§3d).
- No `engram_location` string, anywhere, at any layer.
- No route declared the winner in this document. D is an expectation; the corpus will refuse or ask for evidence until this fleet measures, and may legitimately end at `SelectionNeedsPolicy` rather than a winner.

## 14. Landing order, if approved

1. Component facts in `extdeps.deepseek.deepseek_v4_1_flash`: the `EngramComponentSubject` digest over an exact tensor population (§3), immutability/layout rows, and the row-size read obligation. **One Spark read discharges three obligations** and should be a single pass: the `.engram.` row dtype and stride (grounding §4.2's 264 B and settling whether weight and scale are separately addressed), and the `.engram.`/backbone byte spans read on the fleet — turning `CitedToAuthority` into `FootprintObservedOnFleet`. **That read alone does not make §8's arithmetic a feasibility finding** — an earlier revision of this step said it would. A byte-span read establishes neither loaded resident memory, nor the route's working set (§10), nor that anything executes; the positive result stays `WeightsNotRuledOutByLowerBound`, which is a screen passed rather than a fit established.
2. `gunbc.fabric.engram_materialization`: the §9 carriers, the four candidate routes, and route selection **parameterized by a serving candidate** (§7.1), returning `RealizationSelectionResult<MaterializationRoute>`. Also settles the `LocalInProcess`/`LocalAccelerator` question for resident bytes on a unified pool (§2).

   **§3c: a witness is not the production consumer.** An earlier revision said this step may land because a witness exercises the fold, with the serving consumer arriving at step 5. A discriminating witness proves the fold's behaviour; it does not make a future consumer present. The plan already names the consumer, so the honest choices are to land route selection **together with** the `ModelArtifact`/candidate-field consumption (steps 2 and 5 merged), or to land the module as a **declared frontier** naming that exact later consumer and its dissolution trigger. Not "a witness reads it".
3. Both §11.1 gates, on the key relation §11.1 states: the `NonResidentEngramMechanismStanding` probe queried by the **pull pin**, which is the observation lookup key, with the observed local config ID carried on the receipt as execution identity; and, separately, the `RuntimeFeasibility` launch receipt, which is the only thing that can satisfy the hard constraint. **Do not query pin-keyed inventory by config ID** — §11.1 records why that manufactures a false negative.
4. The **admissible host binding** (§8.2): the fit screen **consumes `gunbc.spark.host_commitment`** — the shared authority joining unit roster, cell roles, claimed serving groups and held reservations — replacing the comparison against `spark_fleet_snapshot.host_count`. It does **not** re-implement a subtraction over those inputs: an independently enumerated recipe misses whatever category is added next, which is exactly how an earlier revision of this plan missed the serving-arm claim.

   **This step is NOT gated on this plan's approval, and it must not be sequenced behind the Engram work.** `fleet_fits` is generous today, on a live screen, for reasons that have nothing to do with Engram — this plan merely found it. Carried as a step in a plan about something else, it acquires the one property that makes a known defect permanent: it is real, it is written down, and it belongs to nobody.

   It has an owner: a dashboard work item labelled **DSV41-9**, created by the DSV41 manager session on 2026-09-11 and sequenced after **#11013**, which rewrites `gunbc.spark.serving_deployment_selection` and would otherwise collide. That label is a dashboard node, not a repository artifact, so it does not resolve in the tree — **#11013 is the anchor a reader can check**, in the same shape the corpus already cites DSV41-2 beside it. The honest description of the screen today is "wrong and owned", and the trigger that retires it is the binding landing — not this plan being approved, and not the Engram routes being measured.

   The exclusion terms that binding must consume are in §8.2, and its first obligation is to read them live. An earlier revision of §8.2 transcribed a state that had already changed, which is the specific mistake the binding exists to make unrepeatable.
5. Component decomposition on `ModelArtifact` and the resident-sum change to `weight_fit_of`, with the existing witness rows unchanged in meaning.
6. `RoutePerformanceReading` and the receipt join, when a launch exists to read.

Steps 1–2 can land before any measurement exists. Their correct output is whatever §11.1's four-state table gives for the evidence actually held — today `SelectionNeedsEvidence`, the runtime axis being declared and funded with its reading missing — and that is the deliverable, not a placeholder: a fold that answers honestly about what it cannot yet decide is the shippable unit here.

**A standing instruction for whoever edits this document next: a status claim is not a citation, and a merge silently falsifies it.**

Sentences of the form "in flight", "not yet on `main`", "cited as in flight rather than as a resolvable symbol" are claims about **corpus state at a moment**, and nothing re-derives them. Merging `main` into this branch — which this plan did, correctly, so that a witness it cites would resolve here — does not only make citations resolve. It falsifies every such claim in the document at the same time, three sections away, with no conflict and no diagnostic, because the merge touched no line of this file. And these claims read as *settled*, so a re-read skips them precisely when they have become wrong.

So: **re-check each one against the live PR state at the moment you edit, never against your memory of what was open when you wrote it** — and check them all, because they will not have moved together. At this writing #11013 and #11088 are merged and their symbols are cited plainly; #11107 is open and `host_commitment`'s in-flight wording is still true and must stay. Sweeping them uniformly trades one false status claim for another.

This is the same class as the §14 rule below — a restatement that nothing re-derives — which is why the two sit together.

**A note on how to read this section.** Three separate reviews found §14 restating a conclusion that the section above it had already corrected — steps 3 and the line above among them, each time the *landing order* preserving a reading the *argument* had abandoned. That is not coincidence: §14 is the part a worker executes, so a stale restatement here outranks the corrected prose everywhere else. The repair is that **§14 states no semantics of its own.** Every step names the section that owns its meaning and defers to it; where a step and a section disagree, the section wins and the step is the defect.
