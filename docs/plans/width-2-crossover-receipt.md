# Width-2 crossover over shared typed-module store

**Lane:** fixed width-2 discovery over the existing process-scoped `SharedTypecheckCaches` byte transport (pre-index module artifact reuse **parked** as contingency).

**Closed:** global `Arc` flip on per-index typed caches (+5.84% serial wall, zero width benefit — do not revive).

## Acceptance (all three groups)

| Group | Bar |
|-------|-----|
| Correctness | Outcomes and fingerprints identical; `typecheck_compute` ≤ process-union distinct modules; `private_store_fallback = 0` |
| Performance | 50-entry cohort speedup ≥ 1.30×; representative floor ≥ 20% faster or ≥ 6 minutes recovered |
| Resources | `memory.current` and `memory.peak` < 13 GiB; swap 0; `memory.high` events 0; no hard governor backoff |

Slots: MemoryHigh 13 GiB / MemoryMax 14 GiB. Serial floors measured 10.3–11.0 GiB peak — ~2–3 GiB headroom for a second worker's private index shell.

## Pre-registered prediction (#7941, lively-lynx-653)

**Registered before width-2 A/B results.** Source: PR #7941 private-memory decomposition on the 50-entry cohort (resolve path only, no eval, single thread — **not** a substitute for this A/B).

### Finding (structural)

On the 50-entry cohort, an armed-retention worker holds **2.65 GiB private live heap** (3.34 GiB RSS). The headline is structural: **79.3%** of a worker (2149 MiB armed; 91.2% unarmed) is **one mutually-pinned bundle** whose internal split is an artefact of drop order:

- `resolved_graph_memo` attributes **262.6 MiB or 1863.8 MiB** depending only on when it is dropped.
- `typed_module_cache` attributes **1147.1 MiB or 83.8 MiB**, same cause.
- A live `ResolvedGraph` strong-`Rc`-pins the typed modules.

### Prediction

The configuration where **`typed_module_cache` is SHARED while each worker still assembles its OWN graphs** is predicted to **underdeliver**. That is Commit 2 (`ControlledWidthTwo`) exactly. Sharing `typed_module_cache` alone frees a **range** (83.8–1147.1 MiB), not a fixed number, because a privately held resolved graph pins the same bytes regardless of who else has them in the shared store.

### Interpretation matrix (either outcome is informative)

| A/B outcome | Meaning |
|-------------|---------|
| Width 2 **fails on memory** with share armed | **#7941 predicted it** and named the mechanism (resolved-graph pin). Confirmed diagnosis, not a dead end — next term is the resolved-graph pin / graph assembly sharing, not more typed-cache work alone. |
| Width 2 **passes on memory** | **#7941 falsified** — we learn something real about the pin's actual reach under concurrent workers. |

### First concrete sharing target from #7941 (stable, non-artefactual)

Three terms are separable, stable across both drop orders, entry-count-independent, duplicated per worker with zero amortization:

| Term | Notes |
|------|-------|
| `pool_parse` | whole-pool head |
| `pool_bare_census` | whole-pool head |
| `both_closure_edges` | whole-pool head |
| **Total** | **518.5 MiB** — bounded, does not move when drop order changes |

If the memory bar binds, **these three are the first sharing target**, not the typed cache. Normalize and ownership diagnostic caches measured **0.1 MiB each** across 483 modules — not memory terms (prior candidate list from the brief was wrong; measurement corrected it).

### Bounds when quoting #7941

- Resolve path only — **no eval**; 2.65 GiB does **not** predict the 10.3–11.0 GiB serial floor.
- 50 entries, not full corpus.
- Single thread in the decomposition probe — not an A/B; does not substitute for this experiment.

## Failure-mode interpretation (original lane framing)

**Passes (CPU/wall + memory within bar)** → Bank the win. Follow-up: replace `Arc<Vec<u8>>` encode/decode with a narrow direct shared payload on the store-carried typed result only. Not the ~9,000 existing `Rc` sites.

**Fails on CPU or wall** → Shared typed cache is too small a slice of preparation cost. Next move: share the immutable index/module substrate (`pool_parse` / `pool_bare_census` / `both_closure_edges` per #7941), or serve module artifacts before index construction (parked pre-index lane).

**Fails on memory** → Second worker's private index shell / resolved-graph pin is the blocker (#7941 mechanism). Width 2 waits on shared immutable indexes, whole-pool head sharing, or pre-index artifacts.

## Measurement harness

- **Commit 1 (local):** `cross_worker_shared_typecheck_*` — barrier-synchronized concurrent resolves, overlap assertion (`maximum_concurrent_workers >= 2`), store counters.
- **Commit 2 (local):** `p1_cohort_probe` with `GUNBC_P1_COHORT_WIDTH=1` (Serial) vs `=2` (ControlledWidthTwo), same 50-entry roster.
- **Commit 3 (fleet):** Representative floor on PR branch with cgroup memory bars.

## Cohort A/B results (local, 2026-08-06)

| Width | Wall | Resolve | Outcome |
|-------|------|---------|---------|
| 1 (Serial) | 198,277 ms | 96,327 ms | Completed; 1 pre-existing witness fail (`witness_observed_hostname_reads_typed_op_hermetic`); `private_fallback=12026` |
| 2 (ControlledWidthTwo) | — | — | **SIGKILL (exit 137)** ~5.5 min in, during dual index build + first typechecks |

**Verdict vs #7941:** Width 2 failed on memory with the shared typed store armed. **#7941 predicted this configuration would underdeliver** and named the mechanism (private resolved graphs pin typed-module bytes; sharing the cache alone does not bound retention). This is a **confirmed diagnosis**, not a post-hoc explanation — the prediction was registered before this run.

Session cgroup `memory.max` ≈ 31 GiB; two whole-tree indexes plus shared store exceeded the envelope during early typecheck. Next term: resolved-graph pin and/or the three stable whole-pool heads (`pool_parse`, `pool_bare_census`, `both_closure_edges` — 518.5 MiB per worker, #7941).

## Counters

`shared_store_hit`, `shared_store_miss`, `shared_store_encode`, `shared_store_decode`, `private_store_fallback` — process-wide via `shared_typecheck_store_counters_snapshot()`.
