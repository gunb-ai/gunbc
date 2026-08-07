# Width-2 crossover over shared typed-module store

**Lane:** fixed width-2 discovery over the existing process-scoped `SharedTypecheckCaches` byte transport (pre-index module artifact reuse **parked** as contingency).

**Lane verdict (measured no-go, 2026-08-07):** width-2 fails memory bar (`memory.peak` 13.18 GiB > 13 GiB slot ceiling) and never completed the cohort (no speedup; early kills at ~334 s vs width-1 completion at 198 s). #7941 resource prediction consistent; resolved-graph pin mechanism neither confirmed nor refuted. Next term: shared immutable indexes / 518.5 MiB whole-pool heads (operator decision) or pre-index artifacts (parked).

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

## Cgroup ceilings (why the peak number matters)

| Environment | `memory.high` | `memory.max` | Notes |
|-------------|---------------|--------------|-------|
| **Runner slot (production)** | **13 GiB** | **14 GiB** | MemoryHigh throttles; MemoryMax is the hard cap. Serial floors measured 10.3–11.0 GiB peak — ~2–3 GiB headroom for a second worker's private index shell. |
| **This worktree session (measurement host)** | unlimited (`max`) | **31.27 GiB** | No throttling pressure; allocator has no reason to give memory back. |

Width-2 measured **`memory.peak = 13.18 GiB` on a host with >2× the production ceiling**. On an actual slot that peak crosses MemoryHigh immediately and sits **0.8 GiB under the hard cap** — so 13.18 GiB is a **lower bound** under production conditions, not a neutral figure measured with room to spare.

## Cohort A/B results (local, 2026-08-06)

| Width | Wall | Resolve | Outcome |
|-------|------|---------|---------|
| 1 (Serial) | **198,277 ms** (~3.3 min) | 96,327 ms | **Completed**; 1 pre-existing witness fail (`witness_observed_hostname_reads_typed_op_hermetic`); `private_fallback=12026` |
| 2 (ControlledWidthTwo) | **no completion** | — | Two early attempts: **exit 137 (SIGKILL) at ~334 s wall**; instrumented re-run: **18+ min, manually stopped (exit 143), still no completion summary** |

### Performance bar (speedup)

**No speedup ratio — width-2 never completed the 50-entry cohort on this host.**

| Comparison | Result |
|------------|--------|
| Width-1 wall | 198,277 ms — finished |
| Width-2 early kills | ~334 s wall — **1.69× slower than width-1's completion time, without finishing** |
| Width-2 instrumented re-run | 1,098 s wall before manual stop — still running, no `p1_cohort_probe: PASS` line |
| Cohort speedup ≥ 1.30× bar | **Not evaluable** (no width-2 completion); directional evidence is **anti-speedup** |

**Verdict:** width-2 is **both over the memory bar and not demonstrably faster** — a clean unambiguous no-go on both axes that were measured.

### Exit 137 cause analysis (measured, not inferred)

Exit 137 is SIGKILL. Three distinct causes matter; only cause 2 (own-cgroup OOM) supports "width 2 does not fit in the memory envelope" as an OOM kill.

| Cause | Description | Evidence |
|-------|-------------|----------|
| 1 | Harness wall-clock cap | First width-2 runs died at **~334 s**; width-1 completed in **~198 s**. Not a 600 s CI cap, but consistent with an external ~5 min session limit. |
| 2 | Own-cgroup OOM (`memory.events` `oom_kill` increments) | **Not proven** for the width-2 runs (see below). |
| 3 | Kill from outside cgroup (host limiter, governor) | Possible for the ~334 s kills; `memory.events` `max=0`, `high=0` throughout instrumented run. |

**Session cgroup readings** (`/sys/fs/cgroup`, this worktree session):

| Field | Value |
|-------|-------|
| `memory.max` | 33,578,549,248 bytes (**31.27 GiB**) |
| `memory.high` | `max` (unlimited) |
| `memory.peak` | 14,154,764,288 bytes (**13.18 GiB**) |
| `memory.events` | `low=0 high=0 max=0 oom=0 oom_kill=2 oom_group_kill=0` |

**Instrumented width-2 re-run (2026-08-06T22:52Z):** cgroup counters sampled every 5 s during `GUNBC_P1_COHORT_WIDTH=2 target/release/p1_cohort_probe`.

- Baseline `oom_kill=2` → post-run `oom_kill=2` (**Δ=0** during this workload).
- `memory.peak` rose from 12.66 GiB → **13.18 GiB** then held; **never approached `memory.max`**.
- Two worker threads active (TIDs at ~90% / ~74% CPU) for **18+ min** past the ~334 s kill wall of the earlier runs — **own-cgroup OOM did not recur at the earlier kill point**.
- Run manually stopped (SIGTERM) after receipt capture; not an OOM kill.

**Receipt sentence (honest):** width-2 terminated with **exit 137 at ~334 s wall on the first two attempts; no cgroup snapshot at kill time; own-cgroup `oom_kill` did not increment during the instrumented re-run; `memory.peak=13.18 GiB` against `memory.max=31.27 GiB` — exit 137 cause **undetermined** (cause 1 or 3 likely for the early kills; cause 2 **not demonstrated**).

### Resource bar vs #7941 (what we can claim)

The fleet resource bar is `memory.peak < 13 GiB`. Measured cgroup **`memory.peak=13.18 GiB` under width-2** — **fails the bar** even without an OOM kill. Receipt is **peak pressure**, not "OOM killed."

### #7941 prediction verdict (two parts, stated explicitly)

| Axis | Verdict |
|------|---------|
| **Resource (#7941 predicted underdelivery)** | **Consistent.** Shared `typed_module_cache` with per-worker private graph/index shells fails the 13 GiB peak bar (13.18 GiB measured). This supports the pre-registered prediction that this configuration underdelivers on memory — without claiming an OOM kill. |
| **Mechanism (resolved-graph pin)** | **Neither confirmed nor refuted.** Exit 137 cause is undetermined; own-cgroup `oom_kill` did not increment during the instrumented run. Peak pressure is consistent with the pin story but does not demonstrate it — a weaker, honest claim. |

**Do not read this receipt as "the resolved-graph pin was demonstrated."** The prediction is **consistent with** what was measured; that is not the same as mechanism proof.

**Next-term candidate (#7941, operator decision — not acted here):** `pool_parse` + `pool_bare_census` + `both_closure_edges` = **518.5 MiB** per worker, entry-count-independent, drop-order-stable — the only bounded duplication term identified. Pre-index module artifact reuse remains **parked** as contingency per lane scope.


## Counters

`shared_store_hit`, `shared_store_miss`, `shared_store_encode`, `shared_store_decode`, `private_store_fallback` — process-wide via `shared_typecheck_store_counters_snapshot()`.
