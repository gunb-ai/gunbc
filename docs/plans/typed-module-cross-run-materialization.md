# Typed-module materialization across runs

> **Status:** DESIGN UNDER REVIEW (2026-09-20). Hand-authored, **not a projection**: no
> `gunbc.generated_artifact` row produces this file. On acceptance its declarations move into
> `gunbc.floor_materialization` (+ `std.materialization_ladder` if the frame lands there) and this
> file is replaced by that module's projection or deleted.
> **Authority it answers to:** `docs/dag-modeling/DESIGN.md`; the ladder's own admission rules in
> `std.materialization_ladder`.
> **Framing:** relief during the v1 → v2 migration. Not a parallelism project — see §7.
> **Revision 2 (2026-09-20):** integrated with the fabric store (`std.fabric_db`, which the operator
> would rather see named `fabric_storage`) instead of proposing a per-host directory; the frame is
> fabric-wide so warming is independent of branch and process; the key is a typed row; arming is
> derived with a measurement gate that can refuse the provider. What is left to decide is in §8.

---

## 1. The measurement this exists to answer

Required CI per push, after the 2026-09-19 consolidation (one job, `witnesses`):

| Step | Wall | Threads |
|---|---|---|
| Build compiler + witness executor | 2.9 min | parallel (cargo) |
| clippy | 1.4 min | parallel (cargo) |
| Nominal witnesses fold | 21 min | **1** |
| Generated-artifact drift gate | 6.3 min | **1** |

Inside the fold (local run, nominal roster, 6,242 files, 1,733-module closure):

| Phase | Wall |
|---|---|
| parse (.dag: src/v1, dag, src/v2) | ~2 min |
| namespace-wave admission | ~1.7 min |
| gate closure (×2) | 0.6 min |
| **strict preparation — resolve 1,733 modules** | **8.3 min** |
| discovery authority | 1.6 min |
| site projection | 1.6 min |
| reach probes + 400 claims | ~2 min (the claims themselves: 53 s) |

Peak RSS: fold 20.2 GiB, drift gate 12.9 GiB. CPU time equals wall time in every phase.

**The cost is preparation, not claims, and it is paid again on every process.** The drift gate is a
second process over the same corpus, so it pays a second time. Nothing survives exit: the floor's
teardown line names `process_resolve_index`, `process_resolve_store` and
`compile_dag_rust_emit_check_memo` being dropped.

## 2. Why now, and why it is framed at v1 → v2

The fold resolves `dag` **and** `src/v2` as one namespace, because the migration means both trees
are live at once. That is what makes the closure 1,733 modules and preparation 8.3 minutes. A
session PR typically changes a handful of `.dag` modules in that corpus, and every run re-resolves
the rest from scratch.

So the relief is structural to the migration rather than incidental to it: while two corpora are
resolved together, the unchanged one is re-derived on every push of the other.

## 3. What already exists (this design declares a rung; it builds no store)

- **The store is the fabric store** — `std.fabric_db`, which the operator would rather see named
  `fabric_storage` (rename noted, not taken here; the modules are cited under today's names). It is
  content-addressed by construction: `FabricObjectRef` IS a content hash, `FabricObject` the payload,
  with put, get, head read and head advance, and `FabricClosureRead` for reading a closure in one
  request rather than per object. `gunbc.fabric_db_placement` holds which host serves it ("one
  placement, not a replica set"); `fabric_db_file_store` and `fabric_db_serve` realize it.
- **Verification on read is already the store's rule**: `fabric_object_verified(asked, content)` is
  "the single authority every handler routes a fetched object through: a store that answered with
  other bytes is corrupt, never trusted."
- **`gunbc.floor_materialization`** declares two providers, both scoped to the executor process with
  `scope_released_retention`: `process_shared_index` over `union-index-build(dag,src/v2)` and
  `process_shared_typecheck_store` over `union-typecheck-store(dag,src/v2)`.
- **`std.materialization_ladder`** carries the vocabulary: `Frame { name, kind }`, `ProviderTier`
  including `CasTier { keying: ContentKeyed }`, and `persistent_retention(capacity)`.
- **`src/v1/stage0/src/shared_typecheck_store.rs`** ("S2a increment C") already serializes
  `TypecheckModuleResult` as authored module and type *names*, explicitly so a decoding process can
  read an encoding process's snapshot against its own intern table. Built for cross-worker; cross-run
  and cross-host are the same problem with a longer gap.
- **`SharedTypecheckStoreCounters`** already counts hits, misses and bytes;
  **`test.claim.typed_module_cache_capacity_witness`** already guards capacity.

What does not exist: a typed-module key, any retention beyond process exit, and a provider row that
names the fabric store as the place typed modules live.

## 4. The design

### 4.1 One new frame

```
data floor_fabric_runs_frame: Frame = Frame {
  name: "fabric-recurring-runs",
  kind: UnboundedSiblingsFrame
}
```

**The kind is load-bearing, and the first draft had it wrong** (review, 2026-09-20). The ladder's
plurality rules read:

> (1) Redundancy visible in the graph whose LCA frame is shared-state (one workspace/process —
> rewireable) = AUTHORED duplication = error; the fix is rewire/Share, **never a cache**. …
> (3) EXPECTED-emergent redundancy — a single demand today, but an enclosing frame declares
> re-demand (`ReplayedFrame` attempts>1 = retry/restart replay; **`UnboundedSiblingsFrame` = server
> loop / recurring CI invocations over time**) — obligates UP FRONT: prepare-before-demand, so
> checkpointing and **cross-run caches are derived, not bolted on** after the incident.

Recurring invocations are state (3): they execute apart by construction and cannot be rewired into
one another. Under `SharedStateFrame` the ladder would call them authored duplication and answer that
a cache is the wrong fix.

**The frame is the fabric, not one host** (operator ruling, 2026-09-20: warming must be independent
of branch and process). Every runner puts to and gets from one served store, so the re-demanding
siblings are every invocation anywhere — CI on any host, the merge queue, a session container, a
developer's own run. Nothing seeds anything and no branch is privileged: a module resolved once is
resolved for everyone who asks next.

### 4.2 One new provider row

```
data floor_typecheck_store_fabric_provider: CacheProvider = provider_row(
  id: "fabric_typecheck_store",
  scope: [floor_fabric_runs_frame],
  coverage: CoversIdentities { identities: [floor_typecheck_store_identity] },
  tier: CasTier { keying: ContentKeyed },
  retention: <the fork in §8.1>
)
```

It covers the **same identity** as the in-process provider, `union-typecheck-store(dag,src/v2)`. The
in-process row stays exactly as it is; this row says where the value lives when the process that
computed it is gone.

**`CasTier`, and the reason changed under review.** The reviewer correctly objected that a per-host
directory is `ArtifactTier` (`tier_axes`: `LocalFilesystem`) and not `CasTier` (`RemoteNetwork`) —
against the draft that proposed a directory. The store is now the fabric store, a served endpoint on
one host, which every other runner reaches over the network. `CasTier { keying: ContentKeyed }` is
the placement that actually obtains. Both tiers are `KeyAddressed`, so the keying below is unchanged.

### 4.3 The key is a typed row, not a table in this document

`FrameDemand.nature` for typed-module resolution is `PureComputation`: the same module text, the same
dependency closure and the same compiler produce the same typed result. Content-keyed reuse of a pure
computation is sound by construction — no staleness envelope is declared because nothing is read from
the world.

**The preimage is a declaration, so that "did we include X" is a field rather than a memory**
(operator ruling, 2026-09-20: these decisions belong in a conformant structure). The fabric store
keys by `FabricObjectRef`, the hash of a preimage; the preimage is this record:

```
type TypedModuleKeyPreimage {
  module_identity: NonEmptyStr        // which module's typed result this is
  module_source_digest: NonEmptyStr   // the text that was typechecked
  dependency_digests: List<NonEmptyStr> // the resolved closure it typed against
  compiler_identity: NonEmptyStr      // the typechecker IS an input
  source_roots: List<NonEmptyStr>     // resolution is root-relative
  store_format_version: Int           // decode compatibility
}
```

Every component is there because omitting it serves a wrong answer: without the source digest an edit
is ignored; without dependency digests a dependency's edit serves a stale type; without compiler
identity a typechecker fix serves pre-fix types; without the roots a root change serves the wrong
resolution; without the format version a format change decodes garbage. Adding a component later is a
diff that invalidates the store by construction, which is the property a prose table could not give.

**The cost profile that follows from `compiler_identity`**: a change under `src/v1` (the Rust
compiler) invalidates every entry, and a change to `.dag` corpus invalidates the changed modules and
their dependents. During v1 → v2 the bulk of the churn is corpus, which is the case that hits.

## 4.4 Where it attaches in the code

**The first draft's "reuse the existing seam" does not reach the fold, and that was its worst error**
(review, 2026-09-20). The shared store is armed only when the adaptive worker width exceeds one, and
`floor_materialization` says width 1 keeps the private per-index `Rc` cache. The read path says it:

```rust
let Some(store) = index.cross_worker_store.as_ref() else {
    shared_typecheck_store::record_private_store_fallback();
    return Ok(index.typed_module_cache.borrow().get(typed_key).cloned());
};
```

The nominal fold runs at width 1, so every read takes that fallback — there is a counter for it — and
a persistent backing behind the shared store would never be read or populated. No warm relief could
occur. **Arming must therefore be independent of worker width.**

**Two levels, and the store's "sole authority" rule has to relax.** The cross-worker store documents
that when armed it becomes the only typed-cache authority and reads decode bytes. At width 1 that
would replace a cheap `Rc` clone with a decode — now a *network get* — on thousands of reads, and
could make a warm run slower than a cold one. So:

| | Today at width 1 | This design |
|---|---|---|
| Read | private `Rc` map | L1 private `Rc` map; on an L1 miss, the fabric store |
| Write | private `Rc` map only | L1 as today, plus one put per newly resolved module |
| Fetch shape | — | one `FabricClosureRead` for the closure, not 1,733 round trips |
| Arming | `scheduled_width > 1` | derived (§4.5), never a flag someone remembers |

### 4.5 Arming is derived, and the gate that can refuse it is a row

**Whether to arm was decision 5 of the first draft; asking a reviewer was the error** (operator
ruling, 2026-09-20). The ladder already decides it: a `PureComputation` demand whose enclosing frame
declares re-demand obligates prepare-before-demand, and §4.1's frame declares exactly that. Arming is
the consequence; there is nothing left to choose.

What remains is empirical and belongs in the model as a refusable row rather than in someone's
judgement: **the provider is admitted only while its measured cold-run overhead is below its measured
warm-run saving.** Width 1 encodes nothing today, so this design adds put work on a cold run for
benefit that arrives on a later one, plus network latency on warm gets. The measurement in §9 step 3
produces both numbers, and a provider whose overhead exceeds its saving is refused by that row — not
kept alive by an argument.

## 5. Soundness, and how each failure mode ends

| Failure | Behaviour |
|---|---|
| Store answered with other bytes | the store's own rule refuses it: `fabric_object_verified` treats it as corrupt, never trusted. Recompute, count it. |
| Entry decodes wrongly / truncated | treated as a **miss**: recompute, count it. Never a partial decode used as a result. |
| Store unreachable, endpoint down, `FabricDbFault` | full preparation, logged. The gate's verdict never depends on the store being available — an unreachable store costs time, not correctness. |
| Compiler changed | `compiler_identity` is in the preimage, so every entry misses. Correct, and honest: a `src/v1` change gets no relief. |
| Corpus module changed | that module and its dependents miss; the rest hit. The common session PR. |
| A wrong KEY would return bytes that verify perfectly | **this is the gap verification cannot close** — the hash matches what was asked for; the question is whether the preimage covered everything that affects typing. §6 is the only check for it. |

## 6. The falsifier

A `verify` mode resolves a module **both** ways — from the store and from scratch — and refuses if
the typed results differ. It runs in a scheduled job, not per push, because it costs a full
preparation by definition. Without it, "the cache is sound" is a claim with no discriminating test,
which DESIGN §5 names spec-without-execution.

**The subject digest cannot serve as that check** (review, 2026-09-20). `PreparedSubject.subject_digest`
is computed by `subject_digest_for_closure` BEFORE strict resolution, over the closure's source
content and the compiler transform content — it hashes the *inputs*. Warm and cold runs of one commit
therefore print the same digest even if the store returned a wrong typed result, so comparing it
would be a check that cannot fail for the reason it is written: precisely the
spec-without-execution shape DESIGN section 5 forbids.

The CI check must compare **outputs**:
- a digest over the typed results of the resolved closure (the fold already holds every
  `TypecheckModuleResult` it produced), emitted into the measurement receipt; and
- the floor's existing receipts, which are output artifacts — `required_floor_disposition.tsv`,
  `required_floor_claim_cost.tsv`, `required_floor_cross_claim_demand.tsv`. A warm run and a cold
  run of one commit must produce byte-identical dispositions and per-claim verdicts.

The first is the direct falsifier; the second is available today with no new emission and would
catch any divergence that changes a claim's disposition.

## 7. Non-goals

- **Threading the resolver.** The interpreter is `Rc`-bound: 13,313 `Rc<` against 40 `Arc<` in
  `src/v1`, and even `parse_source` returns `Rc<ParseResult>`. `shared_typecheck_store.rs` names
  that conversion as its own dissolve-on ("store-path `Rc`→`Arc` on `TypecheckModuleResult`").
  Out of scope here, and not needed: this design removes the work rather than distributing it.
- **Re-sharding the fold across workers or processes.** Ruled out by the fold's own authority
  ("the single prepared subject is precisely what the fold exists to be"), and it would multiply
  preparation rather than divide it. Note the existing `CONTROLLED_WIDTH: usize = 2` sharded path
  in `run_discovery_corpus_with_options` belongs to the corpus-batch regime whose authority was
  superseded 2026-08-13 and deleted 2026-08-15.
- **Moving the fold to a hosted runner.** It peaks at 20.2 GiB; a standard hosted runner has 16 GB.
- **Building a store.** The fabric store exists and is content-addressed with verification on read;
  this design declares where typed modules live in it, and nothing more. A rename of `fabric_db` to
  `fabric_storage` is the operator's preference and a separate change.

## 8. What is left to decide

Four of the first draft's five questions dissolved: the store is the fabric store (§3), warming is
emergent and branch-independent (§4.1), the key is a declaration (§4.3), and arming is derived with a
refusable measurement gate (§4.5). Two remain, and the first is a genuine fork.

### 8.1 Retention — the fork

`std.fabric_db` states its divergence from `std.artifact_store` deliberately:

> that store is an evicting cache (least-recently-used packing to a byte budget). An object here is
> history a head may reach, so it is **never evicted** — retention is released-never, which
> `store_over_provider` deliberately does not realize.

So a bounded, evicting typed-module store is not what the fabric store is today, and the ladder still
refuses `ReleasedNever` with `CapacityUnbounded`. Three ways out, and this is the decision:

1. **Typed modules are history.** Keep them in the fabric store unevicted, and discharge the ladder's
   capacity obligation by bounding what is *put* (only modules in a gate closure) and observing
   growth. Simplest; unbounded in the long run.
2. **Typed modules are a cache, so they belong in `std.artifact_store`** with its LRU-to-a-byte-budget
   retention — the ~4,000 entries / ~14 GB already agreed — and the fabric store keeps history only.
   Honest about what they are; a second store in the picture.
3. **The fabric store grows a bounded namespace** whose objects are evictable, separated from
   head-reachable history. Most work, and the only option that leaves one store.

Option 2 matches the agreed sizing and each store's stated purpose; option 1 is the fastest to land.

### 8.2 Placement and reachability

The store has one placement by ruling, and the fold runs on srv1, srv3 and srv4. A warm get is a
network round trip from those hosts, and an unreachable placement degrades every fold to cold. §9
step 3 measures the latency; the question is whether one placement is acceptable for a CI-hot path or
whether the typed-module namespace wants a local read-through per host.

## 9. Rollout, and the relief each step buys

| Step | Work | Effect |
|---|---|---|
| 1. Model rows: frame, provider, `TypedModuleKeyPreimage`, the retention arm §8.1 picks, counters-as-observation | today | the authority exists; ladder admission passes or refuses loudly |
| 2. Put/get through `std.fabric_db` behind the L1 map, arming independent of width, closure read for the fetch | today | nothing changes until the gate row admits it |
| 3. Measure on one host: cold-armed-but-empty vs cold-disarmed (put cost), warm vs cold (saving and get latency), typed-output digest and the floor's receipts byte-identical | tomorrow | the numbers the §4.5 gate row consumes — and the number this design is worth |
| 4. The gate row admits the provider | tomorrow | the relief lands everywhere at once, since warming is fabric-wide |
| 5. Widen `required_gate_prefixes` back toward the full roster | after | the claims dropped for cost come back, against the cheaper curve |

No seeding step: warming is a property of the store, not of a branch or a job (§4.1).

**Expected relief, on the measured basis** (a PR touching `.dag` corpus modules only):

| | Now | With a warm store |
|---|---|---|
| Strict preparation | 8.3 min | ~1 min + get latency |
| Drift gate (same corpus work) | 6.3 min | ~2 min |
| Fold total | 21 min | ~12 min |
| Push total | ~32 min | ~20 min |

**And the roster curve this unlocks** (measured 2026-09-19): widening costs ~0.25 s and ~3.4 MiB per
module a prefix's closure pulls in, ~0.14 s per claim probed, and ~13 ms per claim executed. The
0.25 s/module term is preparation — the one a warm store removes — which is why the roster widens
after this lands rather than before.
