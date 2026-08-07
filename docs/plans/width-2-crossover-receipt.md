# Width-2 crossover over shared typed-module store

**Lane:** fixed width-2 discovery over the existing process-scoped `SharedTypecheckCaches` byte transport (pre-index module artifact reuse **parked** as contingency).

**Lane verdict (measured no-go, 2026-08-07, revised):** **performance no-go** for the current width-2 plus shared-JSON-store implementation. Width-1 finished the 50-entry cohort in **198 s**; width-2 never finished and was still burning CPU at **1,098 s** before manual stop. **Memory:** cgroup `memory.peak` **13.18 GiB** vs slot MemoryHigh **13 GiB** — a **soft-target miss** (~1.4%); behaviour under the actual capped envelope (MemoryMax 14 GiB, MemorySwapMax 32 GiB) is **unmeasured**. **Cause unapportioned** — only A vs D has been run; pending the 2×2 below. #7941 resolved-graph pin mechanism **neither confirmed nor refuted**. Do **not** start 518.5 MiB whole-pool-head sharing on this receipt alone.

**Closed:** global `Arc` flip on per-index typed caches (+5.84% serial wall, zero width benefit — do not revive).

## Acceptance (all three groups)

| Group | Bar |
|-------|-----|
| Correctness | Outcomes and fingerprints identical; `typecheck_compute` ≤ process-union distinct modules; `private_store_fallback = 0` |
| Performance | 50-entry cohort speedup ≥ 1.30×; representative floor ≥ 20% faster or ≥ 6 minutes recovered |
| Resources | `memory.current` and `memory.peak` < 13 GiB; swap 0; `memory.high` events 0; no hard governor backoff |

Slots: MemoryHigh **13 GiB** / MemoryMax **14 GiB** / MemorySwapMax **32 GiB**. Serial floors measured 10.3–11.0 GiB peak on width-1.

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

**Do not act on this number from this receipt.** The decomposition never established that these bytes own the wall-time regression, and on the JSON-decode hypothesis they would not. Operator decides the next lane.

Normalize and ownership diagnostic caches measured **0.1 MiB each** across 483 modules — not memory terms (prior candidate list from the brief was wrong; measurement corrected it).

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
- **Commit 3 (fleet):** Representative floor on PR branch with cgroup memory bars — **not run** per lane instruction.

## Cgroup ceilings (production policy vs measurement host)

| Environment | `memory.high` | `memory.max` | `memory.swap.max` | Notes |
|-------------|---------------|--------------|-------------------|-------|
| **Runner slot (production)** | **13 GiB** | **14 GiB** | **32 GiB** | MemoryHigh throttles reclaim; MemoryMax is the hard cap; swap available beyond that. |
| **This worktree session (measurement host)** | unlimited (`max`) | **31.27 GiB** | — | No throttling pressure; allocator has no reason to give memory back. **Capped behaviour unmeasured.** |

### What `memory.peak = 13.18 GiB` actually measures

- **Aggregate cgroup peak counter**, not an isolated measurement of two workers' live heap.
- Width-2 instrumented run: peak rose from **12.66 GiB → 13.18 GiB** (Δ **~0.5 GiB** on the counter) while two workers ran concurrently.
- Against production policy: **0.18 GiB over MemoryHigh** (soft reclaim threshold), **0.82 GiB under MemoryMax**, with swap available — a **soft-target miss (~1.4%)**, not demonstrated resource failure. No `high`, `max`, `oom`, or `oom_kill` counter movement during the instrumented run (`oom_kill` Δ=0).
- **Do not read 13.18 GiB as a lower bound on production need.** Under a real 13/14 GiB envelope, reclaim (and some swap) would reduce resident memory while increasing wall time. An uncapped peak is what the workload floats to when nothing pushes back.

## Cohort A/B results (local, 2026-08-06)

| Width | Wall | Resolve | Outcome |
|-------|------|---------|---------|
| 1 (Serial) | **198,277 ms** (~3.3 min) | 96,327 ms | **Completed**; 1 pre-existing witness fail (`witness_observed_hostname_reads_typed_op_hermetic`); `private_fallback=12026` |
| 2 (ControlledWidthTwo) | **no completion** | — | Two early attempts: **exit 137 (SIGKILL) at ~334 s wall**; instrumented re-run: **1,098 s, manually stopped (exit 143), still no completion summary** |

### Performance bar (speedup) — **no-go**

| Comparison | Result |
|------------|--------|
| Width-1 wall | 198,277 ms — finished |
| Width-2 early kills | ~334 s wall — **1.69× slower than width-1's completion time, without finishing** |
| Width-2 instrumented re-run | 1,098 s wall before manual stop — still running, no `p1_cohort_probe: PASS` line |
| Cohort speedup ≥ 1.30× bar | **Not evaluable** (no width-2 completion); directional evidence is **anti-speedup** |

**Verdict:** width-2 plus shared-JSON-store is a **performance no-go** on its own — slower and never finished. That finding does not require the memory number.

### Exit 137 cause analysis (measured, not inferred)

Exit 137 is SIGKILL. **Do not cite the ~334 s kills as resource evidence.** The same workload then lived **1,098 s** with no OOM event, which makes an external session or harness limit far more likely than memory for those early kills.

| Cause | Description | Evidence |
|-------|-------------|----------|
| 1 | Harness / session wall-clock cap | First width-2 runs died at **~334 s**; width-1 completed in **~198 s**. Instrumented run survived **18+ min** past that point. |
| 2 | Own-cgroup OOM (`memory.events` `oom_kill` increments) | **Not demonstrated** (`oom_kill` Δ=0 during instrumented run). |
| 3 | Kill from outside cgroup | Possible for the ~334 s kills; no snapshot at kill time. |

**Session cgroup readings** (`/sys/fs/cgroup`, this worktree session):

| Field | Value |
|-------|-------|
| `memory.max` | 33,578,549,248 bytes (**31.27 GiB**) |
| `memory.high` | `max` (unlimited) |
| `memory.peak` | 14,154,764,288 bytes (**13.18 GiB**) |
| `memory.events` | `low=0 high=0 max=0 oom=0 oom_kill=2 oom_group_kill=0` |

**Receipt sentence:** width-2 early attempts exited **137 at ~334 s** (cause undetermined, likely external limit); instrumented run showed **`oom_kill` Δ=0**, **`memory.peak` 13.18 GiB** on an uncapped host — **not** an OOM headline.

### Resource bar (soft-target miss, not resource no-go)

Acceptance bar asks `memory.peak < 13 GiB`. Measured **13.18 GiB** misses MemoryHigh by **0.18 GiB** while remaining **0.82 GiB under MemoryMax** with swap headroom. Swap-equals-zero is an optimization bar, not a safety law. **Behaviour under the capped slot envelope is unmeasured.**

### #7941 prediction verdict (two parts, stated explicitly)

| Axis | Verdict |
|------|---------|
| **Resource (#7941 predicted underdelivery)** | **Partially consistent** on peak pressure only — shared store with per-worker private shells produced a MemoryHigh soft miss on an uncapped host. **Not** a demonstrated resource failure or OOM. |
| **Mechanism (resolved-graph pin)** | **Neither confirmed nor refuted.** Exit-137 cause undetermined for early kills; `oom_kill` did not increment during the instrumented run. |

**Do not read this receipt as "the resolved-graph pin was demonstrated."**

## Leading hypothesis (JSON encode/decode, not memory)

The shared store holds **`Arc<Vec<u8>>` JSON**, not decoded results. Every insert is `serde_json::to_vec`; every shared read is `serde_json::from_slice` plus reallocation of decoded structures, with a per-key mutex around computation. The hot path went from map lookup plus clone an in-memory result to map lookup, clone byte handle, allocate, parse JSON, reconstruct. Serial receipt shows ~12,000 private-cache-path operations — that path is hot. Thousands of JSON reconstructions can cost more than a second worker saves.

## Next experiment: 2×2 matrix (5–10 entry cohort, not another 50-entry run)

Only **A vs D** has been measured. Apportion cause with four cells on a **small cohort**:

| Cell | Width | Cache |
|------|-------|-------|
| **A** | 1 | private decoded cache (baseline) |
| **B** | 1 | shared JSON store (isolates serialization/decode overhead) |
| **C** | 2 | private per-worker (isolates width, overlap, duplicate index build) |
| **D** | 2 | shared JSON store (current implementation) |

**Reading:**

- B much worse than A → JSON decode is the defect.
- C at or worse than A → dependency overlap or duplicate index construction leaves too little parallel work.
- C better than A but D much worse than C → shared-store lock/decode interaction is the defect.
- Only D pathological → interaction, not width or memory independently.

**Instrumentation (emit periodically and flush on interruption):** per-worker index-build wall; entry groups popped/completed; current entry per worker; shared hit/miss; JSON encode/decode count/bytes/nanos; per-key lock wait and compute-held nanos; `TYPECHECK_COMPUTE_COUNT`; `memory.current` / `memory.peak` / `swap.current` / `memory.events`. Counters that print only on completion are useless when the run never completes.

Pre-index module artifact reuse remains **parked** as contingency per lane scope.

## Counters

`shared_store_hit`, `shared_store_miss`, `shared_store_encode`, `shared_store_decode`, `private_store_fallback` — process-wide via `shared_typecheck_store_counters_snapshot()`.
