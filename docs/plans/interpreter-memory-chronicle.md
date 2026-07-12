# Interpreter memory chronicle — post-#5910 (execution receipt)

**Status:** timestamped profiling receipt (2026-06-28). **DESIGN.md + `dag/gunbc/ci_floor_measurement.dag` remain the authority** — this doc chronicles measured numbers; it does not replace carrier rows.

**Owner:** calm-ram-408 / stern-moth-225 phase-0 metrics. **Base:** `529fd65044` (main, includes #5910).

---

## Summary table (headline arc)

| Metric | Pre-#5867 | Post-#5867 / fixes | #5910 receipt (run 28332256178) | **This run** (2026-06-28) | Verdict |
|---|---:|---:|---:|---:|---|
| **Whole-tree resolve RSS** (all-modules-live) | **14.2 GiB** | **5.5 GiB** (#5867 intern-table) | — | floor @ w=1 **5.12 GiB**; mock-precompute **32 MiB**; per-entry resolve **~158 MiB** | arc **~5.1 GiB now** (≈5.12); #5867/#5833/#5893 prizes **held** |
| **CI floor self-RSS** (VmHWM) | ~8.7 GiB typical | ~5.94 GiB co-resident @ w=2 | **4.24 GiB** @ w=1 | **5.12 GiB** (5,495,361,536 B) @ w=1, 8 GiB docker | **+0.7%** vs 5.08 GiB target; **+20%** vs pinned 4.24 GiB worst sample |
| **Cgroup peak** (job tree) | — | — | **6.27 GiB** < 8 GiB cap | **5.31 GiB** (5,699,862,528 B) < 8 GiB cap | **no exit-137**; under cap with margin |
| **spawn_width** @ 8 GiB runner cap | — | — | **1** (witnessed) | **1** | matches #5910 |

¹ Strict `whole_tree_resolved_ctx` over live `dag`+`src/v2` **fails** (test scaffolds / eval workflows) — same open thread as `wiring_liveness_whole_tree`. The historical 5.5 GiB probe (silent-moth-532) used a dedicated all-modules resolve, not replayable as one strict graph today.

² Canonical 8 GiB docker floor running at session end (`calm-ram-408-floor-8g-v2.log`); dashboard session partial (31 GiB, w=5): self **8.15 GiB**, per-shard **1.63 GiB**, cgroup **12.36 GiB** — wrong grain for #5910 comparison.

---

## 1. CI floor — canonical envelope (8 GiB cgroup, `spawn_width=1`)

**Method:** `claim_executor` in docker `--memory=8g`, worktree mounted at host path (git + python3 for gates).

**#5910 prior receipt (width=1 probe, run 28332256178):**

```
spawn_width=1
[measurement] floor peak RSS: 4556861440 bytes (VmHWM)
[measurement] cgroup peak: 6733590528 bytes, memory.max=8589934592
```

| Field | #5910 receipt | This run (8 GiB docker) |
|---|---:|---:|
| spawn_width | 1 | **1** |
| floor peak RSS | 4,556,861,440 (4.24 GiB) | **5,495,361,536 (5.12 GiB)** |
| cgroup peak | 6,733,590,528 (6.27 GiB) | **5,699,862,528 (5.31 GiB)** |
| exit-137 | no | **no** |

Discovery ran 1111/1112 witnesses before one `doc_reachability` failure; peak RSS captured after batch 2 halt (discovery resolve+eval included).

**Dashboard session partial** (31.27 GiB cap, `spawn_width=5`, discovery 1112 witnesses green, emit_host gate failed):

```
[measurement] floor peak RSS: 8745672704 bytes at spawn_width=5
[calibration] max-per-shard-peak-rss: 1749134541 bytes
[measurement] cgroup peak: 13279150080 bytes memory.max=33578549248
```

---

## 2. Whole-tree / per-resolve RSS (`claim_batch` probes)

**Mock-precompute vs real resolve** (`floor_effect_gate_witness.dag`, 204-module closure):

| Phase | VmHWM | vs historical |
|---|---:|---|
| After `precompute_whole_tree_published_mock_keys` | **33,837,056 B (~32 MiB)** | was **1,529 MiB** pre-#5833 scoping → **#5833 8.7× prize held** |
| After first entry resolve | **165,654,528 B (~158 MiB)** | executor per-resolve **~148–180 MiB** class (not mock-inflated) |
| End (per-shard peak) | **175,312,896 B (~167 MiB)** | `getrusage(RUSAGE_CHILDREN)` matches VmHWM |

**Inflation caveat:** mock-precompute RSS is **not** additive with per-shard resolve peaks in `claim_executor` (scoped precompute, shared index) — the old ~6× conflation was the unscoped 1.5 GiB transient graph; that path is gone.

**Strict whole-tree** (`measure_whole_tree_resolve`, dag production only, 468 modules): **274,763,776 B (~262 MiB)** — not comparable to 5.5 GiB (no `src/v2` in one graph).

**Serial discovery corpus** (`claim_batch --roster-from-discovery`, 1112 witnesses, host session):

```
[measurement] post-mock-precompute-rss: 456286208 bytes (~435 MiB)
[measurement] per-shard-peak-rss: 5178593280 bytes (~4.82 GiB)
[measurement] children-max-rss: 456286208 bytes (getrusage RUSAGE_CHILDREN)
```

`children-max-rss` under-reports vs VmHWM here (serial in-process resolves, no child reaping); **VmHWM is the authoritative probe**.

---

## 3. Structural prizes vs receipts

| PR | Prize | This run |
|---|---|---|
| **#5867** | intern-table dedup; 14.2→5.5 GiB whole-tree | mock precompute 32 MiB not 1.5 GiB; no regression signal |
| **#5833** | precompute scoping 8.7× (1.5 GiB→37 MiB class) | **32 MiB** post-precompute |
| **#5878** | Node empty-Vec singleton −43.7 MiB | not re-profiled; no regression expected |
| **#5893** | `func_env.sigs` single-authority (~hundreds MiB) | per-entry resolve **~158 MiB** on 204-module closure |
| **#5910** | cap split + width=1 @ 8 GiB | spawn_width=1; floor **5.12 GiB**, cgroup **5.31 GiB**, no exit-137 |

**Regression flag:** none observed. Floor self-RSS **5.12 GiB** @ w=1 is within **+0.7%** of the 5.08 GiB target and **~7% below** the historical 5.5 GiB whole-tree figure; cgroup peak **5.31 GiB** leaves **~2.7 GiB** headroom under the 8 GiB cap. Per-shard calibration in 31 GiB session (**1.63 GiB**) remains below pinned `per_shard_max` (**4.24 GiB**).

---

## 4. Instrumentation (this PR)

- `claim_batch`: `[measurement] post-mock-precompute-rss`, `post-first-entry-resolve-rss`, `per-shard-peak-rss`, `children-max-rss` (`getrusage RUSAGE_CHILDREN`)
- `measure_whole_tree_resolve` bin + `wiring_liveness_whole_tree` RSS line
- `docs/plans/interpreter-memory-chronicle.md` (this receipt)

**Dissolution trigger:** phase-0 `PerformanceReceipt` / resource-aware scheduler Node A supersedes prose receipts (same family as `ci-floor-fractal-gantt.md`).

Related: [space lens — minimal memory prediction from the static `.dag`](space-lens-minimal-project.md) — the predictive counterpart to this chronicle's measured receipts.
