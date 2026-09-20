# Typed-module materialization across runs

> **Status:** DESIGN UNDER REVIEW (2026-09-20). Hand-authored, **not a projection**: no
> `gunbc.generated_artifact` row produces this file. On acceptance its declarations move into
> `gunbc.floor_materialization` (+ `std.materialization_ladder` if the frame lands there) and this
> file is replaced by that module's projection or deleted.
> **Authority it answers to:** `docs/dag-modeling/DESIGN.md`; the ladder's own admission rules in
> `std.materialization_ladder`.
> **Framing:** relief during the v1 → v2 migration. Not a parallelism project — see §7.

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

## 3. What already exists (this design adds one rung, it does not build a cache)

- **`gunbc.floor_materialization`** declares two providers, both scoped to the executor process
  with `scope_released_retention`:
  - `process_shared_index` over `union-index-build(dag,src/v2)`
  - `process_shared_typecheck_store` over `union-typecheck-store(dag,src/v2)`
- **`std.materialization_ladder`** already carries the vocabulary this needs:
  `Frame { name, kind }`, `ProviderTier` including `CasTier { keying: ContentKeyed }`,
  and **`persistent_retention(capacity)`** (`release_policy: ReleasedNever`).
- **`src/v1/stage0/src/shared_typecheck_store.rs`** ("S2a increment C") is a working **serde byte
  transport** for typed modules. It serializes `TypecheckModuleResult` as *authored module and type
  names*, explicitly so a decoding process can read an encoding process's snapshot against its own
  intern table. It was built for cross-worker sharing; cross-run is the same problem with a longer
  gap.
- **`SharedTypecheckStoreCounters`** already counts hits, misses and bytes.
- **`test.claim.typed_module_cache_capacity_witness`** already guards the cache's capacity.

What does *not* exist: any retention beyond process exit, and any content key.

## 4. The design

### 4.1 One new frame

```
data floor_runner_host_frame: Frame = Frame {
  name: "runner-host",
  kind: SharedStateFrame
}
```

`SharedStateFrame` is the honest kind: sibling processes and successive runs on one runner host
share the store's mutable state. It is broader than `floor_executor_process_frame()` and narrower
than the fleet — a host's store is never read by another host, so no cross-host coherence question
arises.

### 4.2 One new provider row

```
data floor_typecheck_store_persist_provider: CacheProvider = provider_row(
  id: "host_persisted_typecheck_store",
  scope: [floor_runner_host_frame],
  coverage: CoversIdentities { identities: [floor_typecheck_store_identity] },
  tier: CasTier { keying: ContentKeyed },
  retention: persistent_retention(capacity: CapacityBounded {
    limit: <entry-count limit, §8>,
    at_capacity: <eviction, §8>
  })
)
```

It covers the **same identity** as the in-process provider, `union-typecheck-store(dag,src/v2)`.
The in-process row stays exactly as it is; this row says where the value lives when the process
that computed it is gone.

**The ladder forces two properties, and that is the point of modelling it here rather than adding a
disk cache:** `retention_admission` refuses `ReleasedNever` with `CapacityUnbounded`
(`RetentionUnbounded`) and with `CapacityUnobserved` (`RetentionUnobserved`). A persistent provider
is therefore inadmissible unless it is **bounded** and its occupancy is **observed**. Both are
obligations this design must discharge, not features it may skip.

### 4.3 Why content keying is sound here

`FrameDemand.nature` for typed-module resolution is `PureComputation`: the same module text, the
same dependency closure and the same compiler produce the same typed result. A `ContentKeyed`
reuse of a pure computation is sound by construction, which is the argument the ladder wants — no
staleness envelope is being declared, because nothing is being read from the world.

The key is therefore everything the computation reads:

| Component | Why it is in the key | What breaks if it is out |
|---|---|---|
| Module authored identity | names the value | wrong module served |
| Digest of the module's own source bytes | the text typechecked | edits ignored |
| Digests of its resolved dependencies (transitive, as resolved) | typing depends on them | a dependency edit serves a stale type |
| Compiler identity — the `gunbc`/`claim_executor` binary digest | the typechecker *is* an input | a typechecker fix serves pre-fix types |
| Source-root set (`dag`, `src/v2`, …) | resolution is root-relative | a root change serves the wrong resolution |
| Store format version | decode compatibility | a format change decodes garbage |

### 4.4 Where it attaches in the code

At the seam the cross-worker store already uses — `index_insert_typed` writes through, a miss
decodes from the store — so there is no second cache layer and no new representation. The existing
transport is reused as-is; only its backing changes from an in-process map to a host directory of
content-addressed entries.

The counters already in `SharedTypecheckStoreCounters` become the observation the ladder demands,
emitted into the floor's measurement receipt so a run reports hits, misses, bytes and evictions.

## 5. Soundness, and how each failure mode ends

| Failure | Behaviour |
|---|---|
| Entry decodes wrongly / truncated | treated as a **miss**, recompute, count it. Never a partial decode used as a result. |
| Key collision | keyed by the repo's existing content hash; a collision is a hash break, not a cache bug. |
| Compiler changed | binary digest is in the key, so every entry misses. Correct, and honest: a `src/v1` Rust change gets no relief. |
| Corpus module changed | that module and its dependents miss; the rest hit. This is the common session PR. |
| Store corrupt / unreadable / disk full | fall back to full preparation, log it, continue. The gate's verdict never depends on the store being available. |
| A stale hit would change a verdict | §6 verify mode is the falsifier. |

## 6. The falsifier

A `verify` mode resolves a module **both** ways — from the store and from scratch — and refuses if
the typed results differ. It runs in a scheduled job, not per push, because it costs a full
preparation by definition. Without it, "the cache is sound" is a claim with no discriminating test,
which DESIGN §5 names spec-without-execution.

A second, cheaper check belongs in CI itself: the fold already prints a subject digest
(`digest=96e8b36e00b61c7a`). A warm run and a cold run of the same commit must print the same
digest.

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

## 8. What the reviewer has to decide

1. **Store location.** A per-host directory (candidate: alongside the existing
   `/var/lib/ctrl/…` host state), and whether ctrl's `ctrl-runner-reclaim.timer` prunes it or the
   store evicts for itself.
2. **The capacity limit and the at-capacity policy.** The ladder refuses the provider without them.
   Proposal: entry-count bound sized to one corpus plus headroom, least-recently-used eviction.
3. **Warm-seeding.** Whether a post-merge run on `main` populates the store so PR runs start warm,
   or PRs warm it themselves.
4. **Whether verify mode is a scheduled job** (§6) or an operator-invoked instrument.

## 9. Rollout, and the relief each step buys

| Step | Work | Effect |
|---|---|---|
| 1. Model rows: frame, provider, capacity, eviction, counters-as-observation | today | the authority exists; ladder admission passes or refuses loudly |
| 2. Persistent backing behind `GUNBC_TYPED_STORE_PERSIST`, default **off** | today | nothing changes until armed |
| 3. Measure on one host: cold vs warm fold, digests equal | tomorrow | the number this design is worth |
| 4. Arm in the workflow env | tomorrow | the relief lands in CI |
| 5. Widen `required_gate_prefixes` back toward the full roster | after | the claims dropped for cost come back |

**Expected relief, on the measured basis** (a PR touching `.dag` corpus modules only):

| | Now | With a warm store |
|---|---|---|
| Strict preparation | 8.3 min | ~1 min |
| Drift gate (same corpus work) | 6.3 min | ~2 min |
| Fold total | 21 min | ~12 min |
| Push total | ~32 min | ~20 min |

A PR that changes the v1 Rust compiler gets **no** relief, by §4.3. During v1 → v2 the bulk of the
churn is `.dag` corpus, which is exactly the case that hits.
