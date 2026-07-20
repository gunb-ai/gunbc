# Cross-worker typecheck share — S2a increment C (the 52% prize)

> **Status:** design increment, 2026-07-14, session eager-otter-182. Prerequisites landed: union-resolve S1 (#6234), schedule-driven dispatch Inc A, interface/body split Inc B (#6543, `std.interface_summary` / `ModuleInterface`), resolve-split instrument (#6535). **Prize still OPEN** — Inc B was neutral on `typecheck_compute` wall time (201.6s vs 194.7s baseline, within noise) by design: it changes the *denomination* of cross-module consumption, not the *placement* of the store. This doc names the increment that closes the prize.
>
> DESIGN refs: §2 (one concept, every scale — the store is one authority, N workers are readers), §3 (single authority — `ModuleInterface` is the modeled cross-boundary grain; below-boundary body stays opaque), §5 (fail-closed — collision honesty and once-per-node extend to the process scope; no silent cross-worker divergence), §6 (priced in displaced cost; every scaffold carries a dissolution trigger), §7 (realization-scoped `Rc`→`Arc` migration with explicit dissolve-on, not a model fact).
>
> Parent lanes (referenced, not forked): [resolver-graph-major-design](resolver-graph-major-design.md) §7 move-2 staging, [interface-summary-declared-use-arity](interface-summary-declared-use-arity.md), [v1-run-stability-throughline](v1-run-stability-throughline.md) Track A receipt, [lens-input-authority-design](lens-input-authority-design.md) §1 per-shard W× residual.

---

## 0. The displaced cost (one line)

Whole-corpus floor runs spawn **W parallel workers** (falsifier receipt: `max_width_reached=9`), and each worker builds a **private** `MultiEntryIndex` with a cold `typed_module_cache` — so the widely shared std/spec prefix (~160 modules overlapping across witness closures) is **typechecked once per worker**, not once per process. The resolve-split instrument (#6535) attributes **52%** of whole-corpus resolve wall time to `typecheck_compute` (genuine cache misses). That term is the prize; everything else in the Track A table is owned by other lanes or is already within-worker memo.

| Fact | Value | Source |
|---|---|---|
| `typecheck_compute` share of resolve | **52%** (194.7s / 371.1s attributed) | v1-run-stability-throughline §1 Track A, #6535 |
| Post-Inc B neutral check | 201.6s typecheck (noise) | same receipt, 2026-07-14 |
| Floor worker width (cold falsifier) | 9 | M1 dial receipt §1 |
| Per-worker index build site | `build_multi_entry_index(&roots)` per shard thread | `cli_run.rs` worker spawn (~8091) |
| Within-worker once-per-node | **LANDED** (S1 + Inc A) | `union_resolve_typechecks_each_node_once` |
| Cross-worker share today | **absent** — `Rc` store is `!Send` | `cli_run.rs` worker comment (~8087) |

**Expected displacement (order-of-magnitude, not a done-bar):** if prefix modules are ~P of the union and workers are ~W wide, the redundant cold typecheck fraction is roughly `(W-1)/W × P/|union|` of the 52% term — dominated by the std/spec prefix every witness closure shares. The falsifier's 9-wide run implies up to **~8/9** of prefix typechecks are pure duplication today. Receipt the actual cut with the same resolve-split instrument, not this estimate.

---

## 1. Why Inc B was necessary but neutral

Inc B (`build_type_env` reads `ModuleInterface` / `interface.env` + `interface.cache`, never parent `TypeEnv` parent chains) delivered:

1. **Memory denomination** — cross-module consumption is interface-grain; the ~16.3 MiB/module whole-ancestry retention class is structurally unwritable at the import boundary (M2/M3 retention prizes).
2. **Store-value clarity** — the modeled handler product is `std.interface_summary`'s carrier (`ModuleInterface`), not an ad hoc `TypedModule` projection. `module_resolution_plan.dag`'s store-value note names this authority.
3. **Semantic firewall** — `transitive_interface_binding_test` proves binding transitivity through synthesized parents without leaking parent-chain objects.

It did **not** move the store across threads. Each worker still owns a private `RefCell<HashMap<…, Rc<TypecheckModuleResult>>>` inside its private index. Inc B shrinks *what* crosses a module boundary when shared; increment C moves *where* the boundary sits — from per-thread to per-process.

---

## 2. Current architecture (two share rungs, one gap)

```
claim_executor process
├── main / pump thread
│   └── process_shared_index (S1) — ONE index per thread for discovery + non-sharded paths
└── W worker threads (floor shards)
    ├── worker 1: build_multi_entry_index() → private typed_module_cache
    ├── worker 2: build_multi_entry_index() → private typed_module_cache
    └── …
        └── each: union-resolve S1 holds WITHIN the worker (once-per-node per shard)
```

**The gap:** S1 collapsed entry-major resolve to **one union per thread**; increment C collapses shard-major cold typecheck to **one union per process**. The `Rc`→`Arc` migration is the realization dissolve-on named in resolver-graph-major §5b and §7 S1 — not a new concept, the next rung on the same store.

**Not the gap (explicitly):**

| Term | Share | Owner |
|---|---|---|
| `reconcile_assembly` (29%) | within-worker schedule + assembly view | resolver-graph-major |
| `normalize` + `ownership` (15%) | within-worker per-module memo (`normalize_diag_cache`, `ownership_diag_cache`) | v1-run-stability-throughline §1 |
| `resolve` (1.3%) | n/a — disproven cut | closed |
| cross-run warm cache | S2b (gated #5941) | out of scope here |

---

## 3. Target increment — S2a move 2 increment C

**Name:** cross-worker typecheck share.

**Contract (operator S1 interim contract, extended one scope):**

1. **Minimum upper bound (process):** across all floor worker threads in one `claim_executor` process, `typecheck_compute_count` (summed with a process-wide atomic, or derived from resolve-split slots) **≤ distinct module count of the process union** — the shared std/spec prefix is paid once per process, not once per shard.
2. **No feature growth:** increment C is the `Rc`→`Arc` store migration + worker rewire only; it does not add persistence (S2b), determinism gates, or envelope knobs.
3. **Behavior preservation:** cross-worker share **==** private per-worker resolve, byte-identical claim outcomes — the existing every-order / warm-cold purity oracles extend to multi-worker shape.

**Model statement (one sentence):** the layer-2 store's value at the module node is `ModuleInterface` for cross-boundary readers; increment C realizes that store at **`SharedStateFrame` (process) `ReferenceTier`**, same cell as S1's index-build share (`floor_materialization.dag`), keyed by authored module name in S2a (content key remains S2b).

---

## 4. Mechanism

### 4.1 Store placement

Introduce a **process-scoped shared cache** created once before the worker pump starts:

```text
before worker loop:
  shared = process_shared_resolve_caches(source_roots)  // Arc<Mutex<SharedTypecheckCaches>>

per worker thread:
  index = build_multi_entry_index_with_shared_caches(roots, shared.clone())
  // index shell is built on-thread; only shared_caches Arc crosses threads
```

**C1 realization note (2026-07-14, implementation attempt):** `SharedTypecheckCaches` must use `std::collections::HashMap` shells (not `im_rc::HashMap`) and **`Arc` not `Rc` for every payload** — `Rc<T>` is `!Send`, so a `Mutex` holding `Rc`-backed typecheck results cannot be shared across OS threads even with exclusive locking. Increment C host wiring (`cli_run.rs` worker spawn) is **blocked on store-path `Rc`→`Arc` migration** for `TypecheckModuleResult` / nested infer carriers; the design and process-wide `typecheck_compute_count` (atomic) land in #6561; worker rewire follows in C1-host PR once Arc bridge exists. Eval stays on-thread (`Rc<ResolvedGraph>` unchanged).

### 4.2 `Rc`→`Arc` migration scope (seed boundary)

**In scope (store-carried, cross-thread):**

- `TypecheckModuleResult`, `TypedModule`, `ModuleInterface` payloads inserted into `typed_module_cache`
- `ParseResult` / `NewlineIndex` entries in `parse_cache` (parse is on the critical path to typecheck; sharing parse avoids duplicate front-end work when typecheck hits)
- `VariantExportSurface` map entries built during reconcile dispatch

**Out of scope (thread-local interpretation):**

- `ResolvedGraph` assembly products handed to `make_eval_context` — may remain `Rc` per worker if eval does not cross threads (today: eval stays on the worker thread that resolved)
- Ephemeral inference temporaries inside `typecheck_module` that never enter the store

**Authority edit:** start in hand-maintained `cli_run.rs` (store shell + worker spawn) + minimal `v1_rt` Arc bridges where the seed already centralizes `Rc` helpers. Full seed `Rc`→`Arc` is **not** required for increment C if only store insertion paths migrate — same discipline as other host-only S1 landings; regen scope is whatever the migrated call graph touches.

Inc B already projects `ModuleInterface` at typecheck exit (`build_module_interface`). The shared store inserts the full `TypecheckModuleResult` (cache-hit semantics require skipping entire `typecheck_module`), but cross-boundary *consumption* after Inc B reads only `typed.interface` — so the interface/body split is the semantic backstop that the shared payload is not an accidental whole-graph leak.

### 4.3 Concurrency rules

| Cache | Lock | Rule |
|---|---|---|
| `typed_module_cache` | `RwLock` | **Double-check:** read hit → return `Arc` clone; miss → compute outside lock → insert; racing writers compute duplicate work but must agree on bytes (purity oracle) |
| `module_source_identity` | `RwLock` | unchanged collision wall — mismatch is typed error, never silent serve |
| `parse_cache` | `RwLock` | same double-check pattern |
| `typecheck_compute_count` | atomic | bump only on genuine miss (§6.2 receipt) |

**Fail-closed:** a worker that cannot acquire or validate the shared store **refuses** — no `build_multi_entry_index` fallback arm (that would be the absorbing fallback: widen to private cold resolve and zero the deficit's frequency).

### 4.4 Worker integration site

Primary edit: `run_discovery_corpus_with_options` worker spawn (~`cli_run.rs` 8050–8120) — replace per-worker `build_multi_entry_index(&roots)` with `shared_index.clone()`.

Secondary: any other spawned shard path that still calls `build_multi_entry_index` for floor work (grep receipt: one site today; claim_executor `SharedClaims` units remain single-threaded by `Rc` eval context — out of scope unless they shard later).

### 4.5 Model / declaration (parallel, not blocking host proof)

Extend the materialization ladder declaration alongside `floor_materialization.dag` P1:

- **Provider id:** `process_shared_typecheck_store`
- **Frame:** `SharedStateFrame` (`claim_executor` process)
- **Tier:** `ReferenceTier`
- **Identity:** `union-typecheck-store(dag,src/v2)` — distinct from P1's `union-index-build` (index-build alone does not imply typecheck store share; S1 proved that)
- **Consumers:** floor worker threads (W readers)

Update `module_resolution_plan.dag` `module_resolution_store_value_note` from "until it lands" to "host store retains full `TypecheckModuleResult` at Arc grain for cache-hit semantics; modeled cross-boundary read is `ModuleInterface`."

---

## 5. Oracles & receipts

| Receipt | Discriminator | Extends |
|---|---|---|
| **Process once-per-node** | sum of worker `typecheck_compute` ≤ \|process module union\|; strict: == miss count on cold whole-corpus falsifier | §6.2 `union_resolve_typechecks_each_node_once` |
| **Cross-worker purity** | two workers, overlapping closures, byte-identical outcomes vs two private indexes | §6.1 every-order equivalence |
| **Collision honesty (process)** | same module name, two declaring files, co-resident across workers → typed error | §6.3 `source_identity_flags_coresidence_collision_but_allows_reexport` |
| **Track A wall time** | resolve-split `typecheck_compute` nanos down ≥(measured duplicate fraction); `reconcile_assembly` unchanged | #6535 instrument |
| **Governor health** | `forced_serial=0`, `hard_backoffs≈0` on falsifier cold run — shared retention must not force serial | M1 dial receipt |
| **Fingerprint stability** | `corpus_fingerprint` / `emit_graph_fingerprint` byte-identical | M1 oracles |

**RED controls:**

- Re-introduce per-worker `build_multi_entry_index` behind a test flag → process once-per-node receipt goes red (compute sum > distinct).
- Shared store serves stale interface after source mutation without identity miss → planted-staleness variant (process lifetime; cross-run staleness remains S2b).

---

## 6. Staging (two host sub-increments, one model row)

```
C1 — Arc store + worker rewire (host-only)
  │ process once-per-node + cross-worker purity tests green
  │ resolve-split receipt: typecheck_compute ↓
  ▼
C2 — ladder declaration + witness enrollment
  │ floor_materialization provider row + enrolled witness
  ▼
dissolve-on: S2b persistence re-keys the same store (content hash); S3 retires any residual per-process duplication to whole-run once
```

**C1 done-bar:** falsifier cold run, width > 1, resolve-split shows materially lower `typecheck_compute` with unchanged fingerprints and governor dial.

**C2 done-bar:** materialization provider row enrolled; regression of private per-worker index is countable (lens / witness), not convention.

---

## 7. Memory / governor interaction (honest trade)

Cross-worker share **increases co-resident retention**: one shared cache holds the union of modules touched by all active workers instead of each worker dropping its private cache at thread exit. This is the correct trade for the **time** axis (§1 A2): M1 already bought width at ~8.8 GiB peak; increment C trades duplicate compute for shared bytes.

**Mitigations already in tree / adjacent lanes:**

- Inc B reduced per-module interface retention (denomination for M2 strip).
- M2 roster-aware env strip (v1-run-stability-throughline M2) — if shared retention threatens governor width, M2 trigger is unchanged ("insufficient width/wall-clock gain").
- Governor remains the fail-closed backstop — no envelope raise.

Receipt both axes on the same falsifier run: `typecheck_compute` down, `peak_current` and `forced_serial` reported honestly.

---

## 8. Non-goals

- **S2b** cross-run Merkle persistence (gated #5941) — increment C is in-process only.
- **S3** cross-process shared store — different frame (`IsolatedChildrenFrame`), `resolved_graph_cache` tier.
- **Within-worker normalize/ownership memo** — already landed; not part of the 52% prize.
- **Eval memo / single-witness OOM** — eval-memo lane (#6441/#6469); explicitly out of v1-run-stability scope.
- **Envelope raises** — forbidden (v1-run-stability §4).

---

## 9. Open decisions for operator

1. **C1 landing gate:** host-only first (oracles green) before model declaration — recommended yes, same pattern as S1.
2. **Arc migration depth:** store-path-only (minimal seed touch) vs whole-seed `Rc`→`Arc` — recommend store-path-only for increment C; whole-seed is S2b/S7 frontier-eviction co-migration if needed. **Receipt (2026-07-20):** [Rc→Arc share spike](rc-to-arc-share-spike.md) — module-grain ratio **0.01** (decode ~100× slower than cold typecheck); increment C does not remove the ~55 s warm; lane dead for wall-clock unless transport changes.
3. **Memory trade acceptance:** proceed with shared retention on the falsifier evidence that width stays > 1; escalate to M2 only if governor receipts regress — recommend proceed (time prize dominates; M2 is pre-authorized).

---

## 10. Dissolution triggers

- Increment C dissolves into `floor_materialization`'s provider lattice when C2 enrolls — this doc's process-scope rows become redundant with the carrier.
- S2b re-keys the same store by `std.interface_summary.module_key` / content hash — increment C's name-keyed `Arc` store becomes the in-process backend S2b reads through.
- Terminal: module schedule is fully graph-major, store is content-keyed, per-process share is whole-run once (S3) — increment C's `SharedStateFrame` row persists only as the in-process leg of the realization tower.
