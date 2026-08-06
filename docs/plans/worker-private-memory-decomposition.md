# Worker private-memory decomposition — what a discovery worker holds that sharing does not cover

> **Status:** measurement receipt, session `lively-lynx-653`, work item `node://adhoc-2a689db3-964`, 2026-08-06.
> **Lane:** de-risking input to the width-2 crossover owned by `bright-koi-166`. This note does **not** run a
> width A/B, does not modify `DiscoveryWidthPolicy`, and does not touch the cross-worker store.
> **Harness:** `measure_worker_private_memory` (`src/v1/stage0/src/bin/measure_worker_private_memory.rs`),
> built with `--features interp_test_witness`. Lines are `[wpm] ...`.
> **It recommends no architecture.** It names the binding term and stops.

---

## 1. The question, and the answer

A runner slot is `MemoryHigh` 13 GiB / `MemoryMax` 14 GiB and a successful serial floor measures
10.3–11.0 GiB, so a second worker has roughly **2–3 GiB** of headroom. The cross-worker store
(`shared_typecheck_store.rs` `SharedTypecheckCaches`) shares **only** `typed_module_cache`. Everything
else on `MultiEntryIndex` is per worker. So: what is that per-worker remainder made of?

**The headline number.** On the 50-entry cohort, with schedule retention **armed** (the configuration a
real worker runs in), one worker's private live heap is **2.65 GiB** — 2711.2 MiB, against 3419.1 MiB RSS.

**The result that matters more than the number: the terms are mostly NOT separable.** Under the
separability control (§4), **79.3% of a worker — 2149.3 MiB — is one mutually-pinned bundle** whose
internal split is an artefact of drop order, not a fact. `resolved_graph_memo` alone attributes anywhere
from **262.6 MiB to 1863.8 MiB** depending only on when it is dropped. Reporting a per-term split inside
that bundle would be an invented number, so this note does not report one.

**The operative answer (§6): no, a second worker does not comfortably fit, and the term that would have
to be shared is not a single field — it is the bundle anchored on the live `ResolvedGraph`.** Sharing
`typed_module_cache` alone frees **somewhere between 83.8 and 1147.1 MiB** — a range, not a number,
because the same bytes are also reachable from privately-held resolved graphs — and the low end of that
range does not make a second worker fit.

---

## 2. Instrument, and what it can and cannot see

Two meters, reported side by side and **never blended**. Neither is corrected by the other, and no term
is sized by subtraction from RSS.

| Meter | What it is | Used for |
|---|---|---|
| **Live heap** | Counting global allocator (`CountingAlloc`): exact bytes live as requested from the allocator | **The only meter used for per-term attribution.** Immune to allocator retention, shared pages, and copy-on-write |
| **RSS / VmHWM** | `/proc/self/status` | Stage totals and the gap only. **Not attributable to a term** |

**Live heap does not see:** mmap'd file backing, thread stacks, allocator metadata and fragmentation.
That is why RSS runs above live heap at every stage — armed, **3419.1 MiB RSS vs 2711.2 MiB live**, a
**707.9 MiB (20.7%) gap** that belongs to the allocator and the process, not to any term. The gap is
reported, not distributed.

### 2.1 Attribution is by exclusive drop, not by shell sizing

The Rc→Arc spike receipt ([rc-to-arc-share-spike](rc-to-arc-share-spike.md) §2.2) records that per-field
shallow sizing under-counts parse trees, resolved graphs and typed payloads, and **must not be summed**
for a crossover. This harness therefore does not size shells. It clears one term and measures the
live-heap **release**.

The consequence is deliberate: a byte still reachable from another field (Rc-shared structure) is not
released, and so is **not attributed to the term dropped**. Shared structure surfaces as order
dependence, rather than as an invented split.

### 2.2 Separability is measured, not assumed

Drop order is a parameter (`--drop-order declared|reverse`). Two terms that share structure attribute
differently depending on which is dropped first, so the cohort was run in both orders and differenced.
**A term whose exclusive release is stable across both orders is separable; one that moves is reported
as not separable, never split by assumption.** §4 is that comparison, and it is the load-bearing section
of this note.

### 2.3 Honest limits

1. **Two configurations, both reported.** `--retention armed` drives the production drain step
   (`index_schedule_entry_completed`) on every entry-completion, arming through the loader's own closure
   authority (`entry_closure_paths_for_test`) — the same closure `index_arm_schedule_retention` uses, not
   a second adjacency. `--retention unarmed` is the retain-all pole. **Armed is the configuration a real
   worker runs in and is what §1's answer uses;** unarmed is reported in §5 because the gap between them
   is itself a finding.
2. **Resolve, not eval.** The warm drives `resolve_entry_with_index_for_discovery_corpus` — the worker's
   real per-entry resolve path and the `discovery_resolve_wall` term. Witness **evaluation** is not in
   this process, so eval-side residency is out of scope and not claimed either way. **This is why 2.65 GiB
   must not be read as a prediction of the 10.3–11.0 GiB serial floor**: that figure covers a full corpus
   and the eval side, and this one covers neither.
3. **Single thread, single index.** One worker by construction. Nothing here measures contention,
   allocator behaviour under two threads, or the width A/B — that is `bright-koi-166`'s measurement.
4. **50 entries, not the full corpus.** The cohort is the 50-entry roster from `p1_cohort_roster.txt`,
   the same population the P1 probe drives, so this decomposition and the width A/B read against one
   population. §5 separates the terms that grow with corpus size from the ones that do not — the
   distinction that lets anything here be extrapolated at all.
5. **`source_files` has no drop arm.** It is the root every other term borrows from, so its release is
   not expressible as a term drop; it sits inside the unattributed residue (41.8 MiB, stable in both
   orders).
6. **Repeatability.** Independent runs of the same configuration agree to **0.05–0.09%** on total private
   heap (armed 2711.2 / 2709.9 MiB; unarmed 6347.8 / 6341.9 MiB). Differences below ~0.1% in the tables
   below are run noise, not signal. Two earlier unarmed attempts were killed by external memory pressure
   before completing (`memory.events max 0` — the cgroup cap was never reached, so the kill came from
   outside the process); the runs reported here are the ones that ran to completion.

---

## 3. The decomposition (50-entry cohort, retention armed)

Construction stages, each delta the **marginal** retained heap given everything before it:

| Stage | Live heap | Δ | RSS |
|---|---|---|---|
| baseline | 0.0 MiB | — | 2.7 MiB |
| `index_shell` (`source_files` + `module_graph_facts`) | 40.3 MiB | +40.3 | 1077.2 MiB |
| `pool_parse` | 517.5 MiB | +477.2 | 1080.1 MiB |
| `pool_qualified_fill` | 571.8 MiB | +54.3 | 1082.4 MiB |
| `tree_bare_census` | 571.8 MiB | +0.0 | 1082.6 MiB |
| `pool_bare_census` | 775.6 MiB | +203.8 | 1082.9 MiB |
| `both_closure_edges` | 1061.5 MiB | +285.9 | 1236.6 MiB |
| warm ×50 entries (armed) | **2711.2 MiB** | +1649.6 | **3419.1 MiB** |

Construction is **1021.3 MiB** and the armed warm adds **1649.6 MiB**. Note the `index_shell` row: 40 MiB
of live heap against 1077 MiB of RSS, because `build_module_index` reads the whole tree's source text
through the page cache before the retained structures exist — a clean example of why the two meters are
never blended.

Population counts (reported so a term can be read per entry — **never** to derive bytes from counts):
`source_files` 3193, `pool_parse` nodes 3193, `source_hash_by_file` 2891, `typed_module_cache` 487,
`parse_cache` / `normalize_diag_cache` / `ownership_diag_cache` 483, `module_source_identity` 785,
`intern_table` strings 88837, `both_closure_edge` rows 4024, `tree_bare_census` 4,
`entry_closure_sources` 50, **`resolved_graph_memo` 1** (retention drops each completed entry's graph;
only the final entry's is still held).

---

## 4. Separability — the load-bearing result

Same armed cohort, both drop orders. A term is separable only if its exclusive release is stable.

| Term | declared | reverse | swing | verdict |
|---|---:|---:|---:|---|
| `pool_parse` | 295.9 | 297.0 | +1.1 | **separable** |
| `pool_bare_census` | 203.8 | 203.9 | +0.1 | **separable** |
| `both_closure_edges` | 18.8 | 18.8 | 0.0 | **separable** |
| unattributed residue | 41.8 | 41.8 | 0.0 | **stable** |
| `resolved_graph_memo` | 262.6 | 1863.8 | **+1601.2** | **not separable** |
| `typed_module_cache` | 1147.1 | 83.8 | **−1063.3** | **not separable** |
| `parse_cache` | 238.4 | 75.0 | −163.4 | **not separable** |
| `tree_bare_census` | 273.8 | 124.2 | −149.6 | **not separable** |
| `pool_qualified_fill` | 185.5 | 1.3 | −184.2 | **not separable** |
| `intern_table` | 43.1 | 0.0 | −43.1 | **not separable** |
| `normalize_diag` / `ownership_diag` / `source_hash_by_file` / `module_source_identity` / `entry_closure_sources` | ~0.5 total | ~0.5 total | ~0 | negligible either way |

Read as two groups, which is the only reading the data licenses:

| Group | Size | Character |
|---|---:|---|
| **Separable terms** (`pool_parse`, `pool_bare_census`, `both_closure_edges`) | **518.5 MiB** | Whole-pool heads. Fixed per worker; independent of entries served |
| **Unattributed residue** (incl. `source_files`) | **41.8 MiB** | Root structure |
| **One mutually-pinned bundle** (everything else) | **2149.3 MiB — 79.3% of the worker** | Split is order-dependent; **no per-term number inside it is real** |

**The same three terms are separable in the unarmed configuration, and the same bundle appears** — so
this is a property of the structure, not of one retention setting:

| Term | unarmed declared | unarmed reverse | verdict |
|---|---:|---:|---|
| `pool_parse` | 295.9 | 296.5 | **separable** |
| `pool_bare_census` | 203.9 | 203.9 | **separable** |
| `both_closure_edges` | 18.8 | 18.8 | **separable** |
| unattributed residue | 41.1 | 41.1 | **stable** |
| `resolved_graph_memo` | 2719.3 | **5494.2** | **not separable** |
| `typed_module_cache` | 2198.8 | **48.7** | **not separable** |
| `parse_cache` | 367.1 | 116.9 | **not separable** |
| `tree_bare_census` | 273.8 | 120.5 | **not separable** |
| `pool_qualified_fill` | 185.6 | 0.8 | **not separable** |
| `intern_table` | 43.1 | 0.0 | **not separable** |

Unarmed, the bundle is **5788 MiB — 91.2%** of the worker, and `resolved_graph_memo` dropped last
releases **5494.2 MiB**: with 50 graphs held, essentially the entire worker hangs off them.

**The mechanism is already documented in the code, and this measurement confirms it.** The
`resolved_graph_evictions` field comment in `cli_run.rs` `ScheduleRetention` states that each assembled
`ResolvedGraph` "strong-`Rc`-pins the very `TypedModule`s per-module eviction drops, so without this the
module bytes never free." That is exactly the swing observed: with the single retained graph dropped
**first**, it releases 262.6 MiB and `typed_module_cache` then releases 1147.1 MiB; with it dropped
**last**, it releases 1863.8 MiB and `typed_module_cache` releases 83.8 MiB. **One live `ResolvedGraph`
pins the typed modules, the parse entries, the qualified fill and the intern table behind it.** The
per-field question "how big is the typed cache" has no order-independent answer while a graph is live.

---

## 5. Armed vs unarmed, and what scales

Retention is worth **57.3%** of a worker on this cohort:

| | Private live heap | RSS |
|---|---:|---:|
| Retention **armed** (real worker) | **2711.2 MiB** | 3419.1 MiB |
| Retention **unarmed** (retain-all pole) | **6347.8 MiB** | 7035.6 MiB |

Both configurations were run in both drop orders; §4 carries the pair. The retain-all pole is dominated
by the same bundle, only more so (91.2% vs 79.3%), because 50 held graphs pin more than one does.

What this means for extrapolating past 50 entries — the distinction that governs a second worker:

* **Fixed, and paid in full by every worker.** The whole-pool heads — `pool_parse` 295.9,
  `pool_bare_census` 203.8, `both_closure_edges` 18.8, plus the census layers inside the bundle — do not
  depend on how many entries a worker serves. `gunbc.executor_schedule_retention`
  `schedule_retention_heads_stay_resident` declares them resident for the run's lifetime **by
  construction, never evicted**. A second worker duplicates them with **zero amortization**.
* **Grows with entries served.** The bundle's per-entry part: `typed_module_cache` (487 modules for 50
  entries) and the single live `ResolvedGraph`. A full-corpus worker holds more of this, not less.
* **Does not grow.** `resolved_graph_memo` **entry count is 1 under armed retention regardless of cohort
  size** — retention drops each completed entry's graph. Its *bytes* are large only because of what it
  pins, not because graphs accumulate.

The diag caches deserve an explicit dismissal, since the brief named them as candidates:
`normalize_diag_cache` and `ownership_diag_cache` are **0.1 MiB each** across 483 modules. They hold
`Rc<Vector<Rc<ErrorNode>>>` that are almost all empty. **They are not a memory term at all.**

### 5.1 The finding the starting list did not name

The brief's candidate list was parse structures, normalization, ownership, intern tables and index
shells. **None of those is the binding term.** The binding term is the **mutually-pinned bundle anchored
on the live `ResolvedGraph`** — 79.3% of the worker — and its existence, rather than its internal split,
is the fact a width decision has to price.

---

## 6. The operative answer

**Does a second worker plausibly fit in 2–3 GiB of headroom? Not comfortably, and not on the strength of
the sharing that exists today.**

| Configuration | Private per worker (50-entry cohort) |
|---|---:|
| Today — armed retention, private typed cache | **2711.2 MiB (2.65 GiB)** |
| Headroom available for a second worker | **2–3 GiB** |
| If `typed_module_cache` sharing removed its *declared-order* attribution (1147.1 MiB) | 1564.1 MiB (1.53 GiB) |
| If `typed_module_cache` sharing removed only its *reverse-order* attribution (83.8 MiB) | 2627.4 MiB (2.57 GiB) |

A worker sits **at the top of the headroom band before any sharing**, on a 50-entry cohort that is
smaller than a real floor run. And the effect of the sharing that exists cannot be stated as a number:
**it is the 83.8–1147.1 MiB range above**, because the bytes `typed_module_cache` holds are also reachable
from the resolved graph the worker keeps privately. At the low end of that range, arming the share
changes almost nothing.

**Which single term would have to be shared to make it fit?** The honest answer is that **no single field
is the unit**. The unit is the bundle — `resolved_graph_memo` together with `typed_module_cache`,
`parse_cache`, the census layers and the intern table — because a privately-held `ResolvedGraph` pins the
others regardless of whether the typed cache is nominally shared. **Sharing the typed cache while each
worker still assembles and holds its own resolved graphs is the configuration this measurement predicts
will underdeliver**, and that prediction is testable against `bright-koi-166`'s A/B: if width 2 fails on
memory with the share armed, this bundle is why.

Whether the bundle *can* be shared is not this note's question. It carries the same `Rc`→`Arc` `Send`
precondition already named as the width latch's dissolve-on in
[cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) open decision 2, and it is
a wider surface than that decision's `store-path-only` default contemplates. **Naming the term is where
this lane stops.** The architecture decision is the operator's and depends on `bright-koi-166`'s result
as well as this one.

---

## 7. Reproduce

```
cargo build --release -p v1-compiler --bin measure_worker_private_memory --features interp_test_witness
./target/release/measure_worker_private_memory --retention armed   --drop-order declared
./target/release/measure_worker_private_memory --retention armed   --drop-order reverse
./target/release/measure_worker_private_memory --retention unarmed --drop-order declared
```

Host for the numbers above: 125 GiB RAM, cgroup `memory.max` 31.25 GiB (never reached — peak RSS
6.87 GiB on the unarmed run, 3.34 GiB armed), aarch64. Load average was 15–45 during the runs; **wall
time is therefore not quoted and no timing claim is made from these runs.**

## 8. Dissolution

Delete this note when the width-2 crossover decision has been taken and cites either these terms or a
superseding measurement — or when the `ResolvedGraph`-anchored bundle becomes shared, at which point the
decomposition it reports no longer describes a worker.
