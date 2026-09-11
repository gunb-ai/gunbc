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

| Concept this plan needs | Existing home | Verdict |
| --- | --- | --- |
| "where do the bytes live" | `std.realization` `Placement` = `LocalInProcess` \| `LocalFilesystem` \| `RemoteNetwork` | **inhabit**. `std.materialization_ladder` already repaired a nickname minted beside this once; a second one is not available. |
| tier of a materialization | `std.materialization_ladder` `ProviderTier` = `ReferenceTier` \| `CopyTier` \| `MemoTier` \| `ArtifactTier` \| `CasTier`, derived from `ProviderAxes {placement, addressing, aliasing}` via `tier_axes` | **inhabit**. RAM-resident = `MemoTier`/`ReferenceTier` (`LocalInProcess`); per-host NVMe = `ArtifactTier` (`LocalFilesystem`); NFS or object origin = `CasTier` (`RemoteNetwork`). No ladder refactor is needed and none is proposed. |
| retention, capacity, replacement | `std.cache_interface` `ProviderRetention`, `CapacityPolicy`, `ReplacementStrategy`, `AtCapacityDisposition` | **inhabit**. |
| latency / consistency / locality class of a backend | `std.cache_interface` `CacheInterfaceFacts`, `ReadLatencyClass` (`InProcessNs` \| `LocalDiskUs` \| `LanMs` \| `WanTensMs`), `PersistenceLocality`, `ConsistencyModel`; the placement record is `extdeps.cache.catalog_placement` `CacheInterfaceCatalogPlacement` | **inhabit**, with one honest gap: see §6. |
| a bounded store over a provider, with eviction | `std.artifact_store` `ArtifactStore`, `store_over_provider`, `store_put`, `StorePutReceipt` | **inhabit** — this is the per-host NVMe cache, not a new thing. |
| content identity of the bytes | `std.content_hash` `ContentHash`; `std.materialization_provider` `ArtifactRequest` / `MaterializedArtifact` / `artifact_content_digest` | **inhabit**. Engram's identity is its digest; every route serves the same digest or is refused. |
| choosing among candidates | `std.decision` `select_realization`, `SelectionReceipt`, `HardConstraint`, `RealizationSelectionResult`; `std.pareto` `SelectionAxis` / `ParetoEntry` / `AxisGap` | **inhabit**. §3d forbids a second decision algebra and this plan declares no score, no weight, and no ranking function. |
| durable origin, read back and digest-verified | `gunbc.fabric_m0_origin_readback` `OriginReading` / `read_origin_staged_file` (sole constructor) | **inhabit**. Every route's chain begins at this origin. |
| citation standing of an upstream figure | `extdeps.external_authority` `CitedFigureStanding` = `CitedToAuthority` \| `TranscribedUncited` | **inhabit** (§4d). |

**Net new concepts proposed: three.** Each is listed with the home it inhabits in §9, and each exists because the census found no carrier for it, not because a new module wanted its own vocabulary.

## 3. The component: one identity, placement-independent

Engram is **one content-addressed immutable serving component**. RAM-residency, a per-host NVMe copy, an NFS-backed read and an object-store origin are four *access realizations of the same bytes*. They are not four artifacts, they do not get four identities, and a route change is not a checkpoint change.

This is the structural reason `engram_location = "ssd"` is forbidden: a location string attached to the component makes placement a property *of the artifact*, so two placements become two artifacts, and nothing can then state that they serve identical bytes. Identity is the digest; placement is a property of the *route*, which is a separate row joined to the component by that digest.

Component facts already live upstream in `extdeps.deepseek.deepseek_v4_1_flash` and mostly already exist: `deepseek_v4_1_flash_engram` (layer ids, n-gram order, heads, head dim, vocab), `..._safetensors_payload_footprint.engram_bytes`, and the publisher-standing row. What is missing there is **immutability and layout as declared facts** rather than as things a reader infers from "it's a checkpoint" — added as component rows in that module, not in a fabric module.

## 4. Access demand: the fact that decides everything, and the one nobody measures

**Never infer access cost from total file size.** This is the load-bearing methodological rule of the whole plan, and the reason the answer is not obvious.

Engram's per-token demand is derivable from `deepseek_v4_1_flash_engram`: 2 layers (`layer_ids: [1, 14]`) × 3 n-gram orders (`max_ngram_size: 4`, orders 2–4) × 8 heads (`n_heads: 8`) = **48 row reads per token**.

Row size is where the honesty boundary sits. `head_dim: 256` with an fp8 row plus an 8-byte scale gives 264 B, which is the figure the brief carries and which reproduces 48 × 264 = 12,672 B ≈ 12.375 KiB/token. **That row size is an inference, not a read fact**: it is consistent with the declared head dim and the checkpoint dtype, and it is not stated by the publisher. It lands as `TranscribedUncited` with a read obligation naming exactly what would ground it — the tensor dtype and per-row stride from the safetensors header for an `.engram.` tensor at this revision, which is a cheap read we have already done for the byte partition.

With that caveat stated once, the shape of the demand:

- **logical bytes/token: 12,672 B against a 203,073,076,240 B table — a ratio of 6.2 × 10⁻⁸.** Per token, one ten-millionth of the component is touched.
- **granularity: 264 B, sub-page.** So the *device* traffic is not the logical traffic: at 4 KiB page granularity a token costs 48 page reads = 196,608 B and 48 IOPS, a **15.5× read amplification**. Any model that prices the logical figure is wrong by that factor, which is precisely why granularity is a declared demand field rather than a derived one.
- **sparse, not sequential**, and reuse is workload-dependent — a token's rows are selected by its n-gram context, so hit locality is a property of the text, not of the model. **This is a typed gap, not a zero.** An assumed reuse rate is the one input that could flip the ordering, so it is declared unread rather than estimated.
- **concurrency**: a decode step does 48 × (batch size) gathers, issued together. Latency, not bandwidth, is the binding term, and it is a *queue-depth* question.
- **max admitted added latency** is a constraint the serving subject owns, not a storage fact.

Worked forward, per replica: at 400 tok/s aggregate the device sees ~78.6 MB/s and 19,200 IOPS; at 6,400 tok/s, ~1.26 GB/s and 307,200 IOPS. Both sit inside a single modern NVMe's random-read envelope — but that is a *comparison against a provider fact that must be measured on this fleet*, not a conclusion, and §7 keeps it that way.

## 5. Routes are chains, and NFS must say which role it plays

A route is an ordered chain from a durable origin to the bytes the runtime touches at token time. Four candidates:

- **A — origin → RAM-resident.** The status quo. Engram is a parameter; the tables occupy the unified pool.
- **B — origin → per-host NVMe (`ArtifactTier`) → runtime page cache.** Fill once per host; steady-state reads are local-disk latency; RAM cost is whatever the page cache actually retains, which is a *measured* number and not the table size.
- **C — origin/NFS → direct network-backed token-time access.** No local copy. NFS is on the serving path for every gather.
- **D — origin/NFS → per-host NVMe → page cache.** NFS is the fill source only; token-time reads never leave the host.

**C and D are different arms and the distinction is the point.** NFS carries three separable roles — correctness-bearing durable origin, site-level fill cache, and token-time serving path — and a route must state which it plays. The published third-party figures (2–3 ms/step on local NVMe, 5.9–7.8 ms/step on NFS) are exactly the C/B separation, and collapsing them into "NFS works" would price D at C's latency for no reason.

D is the *expected* leader. It stays `NeedsEvidence` until this fleet measures it, and the plan's value is destroyed if that expectation is written as the answer.

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

Result vocabulary: `EngramPlacementSelected { route, evidence, cost_receipt }` | `NeedsEvidence { missing }` | `Refused { violated }` — the three arms of `RealizationSelectionResult` as this subject spells them, not a fourth algebra.

## 8. The deduction, worked — including why it does not conclude today

The arithmetic that makes the ordering fall out, stated so it can be checked and so its rungs are visible.

Usable pool per host at the 82% memory fraction the canary route uses: 130,663,231,488 × 0.82 = 107,143,849,820 B (99.8 GiB).

- **Route A (all-resident).** Whole payload 510,286,023,000 B ÷ usable = 4.76 → **5 ranks minimum, so TP8**, since a tensor-parallel degree of 5 is not expressible for an 8-head layout.
- **Routes B/D (Engram off the pool).** Non-Engram payload 307,212,946,760 B ÷ usable = 2.87 → **3 ranks minimum, so TP4**.

### 8.1 The payoff is feasibility against the admissible host set, not a replica axis

An earlier draft of this section read the 8-vs-4 rank difference as "one replica versus two" and landed it on `ax_independent_replicas` and `ax_replica_capacity_share_lost`. **That reading is wrong and the correction matters more than the arithmetic.** The operator's standing ruling is **no redundancy**: replica count is pinned to 1 by a hard constraint in the in-flight #11013 (`constraint_no_redundancy`; not on `main` at this writing, so it is cited as in flight rather than as a resolvable declaration). Under that constraint there is no second replica for a placement decision to buy, and an axis that counts replicas reads identically for every candidate — a permanently-flat axis, which is a decoration, not a funded dimension.

With replica count fixed at 1, the difference is **whether the shape fits the hosts that are actually available at all**:

- Route A: TP8 × 1 — needs **all eight** Sparks, which means taking the serving cells and the RPC-reserved pair.
- Routes B/D: TP4 × 1 — fits the **four** Sparks that are admissible without disturbing either.

So placement is not trading a redundancy property against a latency property. It decides **feasibility inside a host set the fabric, not this decision, controls** — which is a hard constraint screened before Pareto, exactly where §7 puts things that can make a candidate unservable rather than merely expensive.

### 8.2 How "admissible hosts" enters, and the silent assumption it removes

This is the part the corrected reading forces into the open, because **`spark_fleet_snapshot` carries `host_count: 8` and no host identities.** A bare count cannot distinguish eight free hosts from eight hosts of which four are spoken for, so any fit test run against it silently assumes the whole fleet is available — generous in precisely the direction §5 forbids, on the screen whose job is ruling shapes out.

The fleet already says otherwise, in declarations that exist today:

- `gunbc.spark.cell_role` `spark_cell_role_assignments` assigns `SparkServingCell` to **srv5 and srv6**.
- **srv7/srv8** hold an undischarged llama.cpp RPC peer reservation (`gunbc.spark.llama_cpp_rpc_observed`), and `test.claim.spark_llama_cpp_rpc_witness` carries a disjointness wall forbidding them from entering the serving roster until a release receipt exists. That is a wall, not a preference: a candidate that binds them is refused.
- **srv9–srv12** carry no cell role and no reservation.

So the admissible set for a new serving shape today is four hosts, and it is four *because the fabric says so*, not because this document counted. The repair is to make the fit screen consume the **host binding** rather than a scalar: the count a candidate is screened against is derived from the roster minus role assignments minus undischarged reservations, with each exclusion naming its authority. A shape needing more admissible hosts than exist is `Refused` with the exclusions named — never silently fitted against 8.

Two consequences worth stating because they are the reason this is not a cosmetic change. First, the admissible count is **not a constant**: a release receipt on the RPC reservation moves it from four to six, and that is a legitimate future state which must move the decision rather than being baked in here. Second, it makes route A's cost concrete and attributable — A is not "expensive", it is *infeasible without displacing two named subjects*, and if it is ever taken, the displacement is what it costs.

Against that, route A's price for 189 GiB of the scarcest resource on the fleet buys 12.4 KiB/token of access. The RAM opportunity-cost axis of §7 is what states that; without it, A is simply "faster" and wins.

### 8.3 Why it still does not conclude

Missing, and tracked as the obligations §12 carries: this fleet's measured per-step added latency for B, C and D; the page-cache RAM a B/D route actually retains; the row-size ground read; the workload's reuse locality; and the §11 runtime question, which is a hard constraint and can refuse every non-resident route outright. Today the honest output is `NeedsEvidence`, and a plan that shipped `Selected` off published third-party figures would be the fabricated-plausible-output failure with a receipt attached.

**Rung honesty (§4b) for each claim above:** the byte partitions and fleet pool are read facts (`FootprintObservedOnFleet`, `torch.cuda` on a GB10). The 48-reads/token count is *derived from declared config*, which is as strong as the config. The 264 B row is an inference with a stated read obligation. The rank arithmetic is a consequence of the first two and the declared memory fraction. The role assignments and the RPC reservation are declared fleet state. The admissible-host derivation is **proposed, not built** — today no screen consumes it, which is why §14 lands it. The ordering of routes is **unestablished** and typed as such.

## 9. Homes, per new concept

| New concept | Home | Why not an existing carrier |
| --- | --- | --- |
| `ComponentAccessDemand` — bytes/reads per token, granularity, sparsity, reuse (typed gap), concurrency, admitted added latency | `gunbc.fabric.engram_materialization` | Nothing in the corpus states *how a consumer touches* bytes. `FrameDemand` names a computation's identity and nature; `ArtifactRequest` names inputs and outputs. Neither carries an access profile. This is the genuine gap. |
| `MaterializationRoute` — an ordered chain of hops, each a `(Placement, ProviderTier, retention, role)` over the existing ladder vocabulary, head-anchored at the M0 origin | `gunbc.fabric.engram_materialization` | The ladder models a *provider*; nothing models a **chain** of them, which is what a fill-path-plus-serving-path is. The hops are existing types; only the chain is new. |
| `RoutePerformanceReading` — measured p50/p95/p99, fill bandwidth, retained page-cache bytes, on this fleet at a named launch | `gunbc.fabric.engram_materialization` (receipt side) | §6: `ReadLatencyClass` is a backend taxonomy, deliberately not a fleet measurement. Merging them would put a reading inside an `extdeps` classification, a layer inversion. |

Component facts stay in `extdeps.deepseek.deepseek_v4_1_flash`. Backend/provider kind facts stay in `extdeps.cache` / `extdeps.storage`. Fleet readings are receipts in the observing layer, never properties authored into an upstream module (§3, external upstream decomposition).

The module is named for the domain, not the model: `engram_materialization` is the first *subject*, and the carriers are generic over component and route. If a second component (a vision tower, a draft model, a KV offload tier) asks the same question, it supplies its own demand row and consumes the same fold. That is the operator's "this is the general fabric-storage question" made structural rather than promised.

## 10. Integration: per-component placement without forking `ArtifactFootprint`

`gunbc.spark.serving_deployment_selection` carries `ArtifactFootprint` = `FootprintObservedOnFleet` | `FootprintReportedExternally` | `FootprintUnread`, and `weight_fit_of` divides it by tensor-parallel degree against the usable envelope. That vocabulary is correct and **is not forked**. The distinction it draws — read on this fleet vs cited vs unread — is exactly the distinction per-component bytes need.

The change is that the quantity fed to the fit screen becomes **the resident share implied by the selected route**, not the whole payload:

- `ModelArtifact` gains a component decomposition whose entries each carry their own `ArtifactFootprint` — the *same* type, one level down. The scalar whole-artifact footprint becomes the sum, so nothing that reads it today changes meaning.
- Each component carries a placement standing: resident, or placed by a route selected in `gunbc.fabric.engram_materialization`.
- `weight_fit_of` sums only the resident components. `FootprintUnread` on any resident component still yields `WeightFitUnderivable`, unchanged.
- A component placed on a non-resident route contributes **zero resident bytes and one hard constraint** — the runtime-capability constraint of §11 — so a route that the runtime cannot serve does not quietly shrink the footprint.

For #11013's candidate field: a candidate's `topology` is derived, not authored, so the Spark count **follows from** the placement rather than being chosen beside it. `EngramPlacement` is an input to candidate generation; the minimum rank count is a consequence of the resident sum (§8: 3 vs 5 → TP4 vs TP8); and with `constraint_no_redundancy` pinning replica count to 1, that rank count **is** the host demand. It is screened against the admissible set of §8.2, not against `spark_fleet_snapshot.host_count` — the one place this integration could otherwise reintroduce the eight-free-hosts assumption, since `fleet_fits` today compares `hosts_consumed` directly to that scalar. The candidate field does not gain a placement dimension to be searched independently: placement is selected by its own subject, and the deployment subject consumes the result. Two selections, two receipts, one direction of flow, no joint search space.

## 11. The open fact, as an obligation with a hard constraint attached

**Can vLLM's published `deepseekv41-flash-0909` image — `sha256:d84a1232…`, recorded at `gunbc.spark.v41_published_image_observation` `v41_published_arm64_digest` — serve Engram from a non-resident tier at all?**

This plan does **not** settle it, and says so rather than inferring an answer. What is known: `extdeps.deepseek.deepseek_v4_1_flash` `deepseek_v4_1_flash_config_declares_engram_off_accelerator` is `false` — off-accelerator placement is not a publisher config key, the reference implementation allocates the tables as parameters, and a third-party 4-Spark build **patched vLLM** to do it. A patch existing is positive evidence that the stock image does not.

It becomes a **hard constraint**, not an axis: if the runtime cannot serve the component from the selected tier, every non-resident route is `Refused` before Pareto, and the answer is A regardless of cost. Wiring it as an axis would let a cheap-but-unservable route rank.

The obligation is a capability probe against that exact digest, homed with the probes that already exist (`gunbc.spark.serving_runtime_capability_probe` `RuntimeCapabilityProbeRow`), answering one question: does this image expose a mechanism by which Engram tables are read from a non-resident tier at token time. Until that row exists the constraint reads unfunded, and per `std.decision` an unfunded constraint axis is a defect that produces `NeedsEvidence` — which is the correct output.

Note also that a GB10's host and GPU are **one unified pool**, so "host memory offload" adds no capacity here; the only routes that free bytes are ones that reach a *storage* tier. That is why B/C/D are the arms and a host-RAM arm is not.

## 12. Evidence loop

Predict, then measure the same subject, then join. Per route the selection predicts an amortized fill cost and a per-step added access cost; the first real load emits a receipt observing bytes moved, the route actually taken, measured access performance, and the serving behaviour that resulted — **joined to the exact launch**, at the profile grain `serving_deployment_selection` already established (artifact × topology × context ceiling × memory fraction × runtime revision × KV layout × speculative policy), because a receipt acquired under a different profile is not transportable.

The prediction and the measurement are separate rows with a stated relation, so a divergence is a visible fact rather than an overwritten estimate. Each carries its rung: the prediction is derived-from-declared-facts; the receipt is a fleet reading; the third-party figures are `CitedToAuthority` about someone else's hardware and can never be either.

## 13. Non-goals, and the walls that stay up

- No materialization-ladder refactor. The ladder's tiers already span this; §2 lists the mapping.
- No peer mesh. FABRIC-M0 holds: one durable origin, chains anchored there.
- No second decision algebra, no score, no weights (§3d).
- No `engram_location` string, anywhere, at any layer.
- No route declared the winner in this document. D is an expectation; the corpus will say `NeedsEvidence` until this fleet measures it.

## 14. Landing order, if approved

1. Component facts: immutability/layout rows and the row-size read obligation in `extdeps.deepseek.deepseek_v4_1_flash`.
2. `gunbc.fabric.engram_materialization`: `ComponentAccessDemand`, `MaterializationRoute` over the existing ladder vocabulary, the four candidate routes, and the selection consuming `select_realization`. Consumed by execution in the same change — the module produces a `RealizationSelectionResult` a witness exercises, not a declaration set nothing reads (§3c).
3. The runtime capability probe row against `sha256:d84a1232…`, which is what moves the hard constraint from unfunded to funded.
4. The **admissible host binding** (§8.2): the fit screen consumes a host set derived from the roster minus role assignments minus undischarged reservations, each exclusion naming its authority, replacing the comparison against `spark_fleet_snapshot.host_count`. This is separable from the rest and worth landing on its own merits — it removes a silent eight-free-hosts assumption that is wrong today independent of anything about Engram.
5. Component decomposition on `ModelArtifact` and the resident-sum change to `weight_fit_of`, with the existing witness rows unchanged in meaning.
6. `RoutePerformanceReading` and the receipt join, when a launch exists to read.

Steps 1–2 can land before any measurement exists; their correct output is `NeedsEvidence`, and that is the deliverable, not a placeholder.
