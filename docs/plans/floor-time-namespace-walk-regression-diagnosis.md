# Floor time +20min regression — diagnosis (bisected to #6848 namespace walks)

**Status:** diagnosis complete; axis 2b (reconcile deferral) on `main` (#6998); entry-closure memo in this PR (#6999).
Resolver semantics unchanged — perf-only caching/call-order.

**One-line verdict:** #6848's name-derived loader (`extend_sources_to_both_closure_fixpoint`
+ unconditional `pool_qualified_fill` before typed-cache short-circuit) added **~140ms per
floor entry-resolve** and **~13min of batch wall** on top of the memory regression already
priced in [floor-memory-pool-parse-regression-diagnosis.md](floor-memory-pool-parse-regression-diagnosis.md).
After #6956+#6972 fixed memory (peaks 32→12.5 GB), **time stayed** — this is walk-work, not
thrash.

---

## 1. Log-diff receipts (pre vs post, no profiler)

Compared **PRE** run `29701881403` (2026-07-19, 18.2 min wall) and **PRE-parent**
`29763408563` (`ec22e8fabb`, last green before #6848, 22.2 min) against **POST**
`29819122813` (`b5b9c53105`, 46.6 min, profiled).

### 1.1 Batch wall (where the +20min lives)

| batch | PRE 07-19 | PRE ec22e8f | POST b5b9c53 | Δ vs ec22e8f |
|---|---:|---:|---:|---:|
| 2 discovery (~2100 witnesses) | 226s | 202s | **503s** | **+301s (+5.0 min)** |
| 4 effectful gates | 165s | 342s | **1111s** | **+769s (+12.8 min)** |
| regen sub-plan | ~140s | ~148s | ~220s | +72s |

Total CI job wall: 18.2 → 22.2 → **46.6 min**. The **+24 min** post-#6848 class decomposes as
**~5 min discovery + ~13 min batch-4 gates** (remainder: regen + constant overhead).

Per-witness discovery amortized cost: **97ms → 236ms** (+139ms × ~2128 witnesses ≈ **296s**,
matching batch-2 delta).

### 1.2 Phase profile — resolve-before-frontend (`fe_begin`)

`[gantt] compile.frontend.begin t_ms=` is wall-ms since process start; the gap before the
first frontend mark is **entry resolve** (loader + pool census + parse closure).

| metric | PRE ec22e8f | POST b5b9c53 |
|---|---:|---:|
| `fe_begin` median | 35s | **138s** |
| `fe_begin` p95 | 310s | **503s** |
| `fe_begin` >300s count | 4 / 34 | **16 / 23** |

Resolve-before-frontend grew **~4× median** in shared-index discovery workers.

### 1.3 Floor materialization receipt — memo did NOT collapse

| field | PRE ec22e8f | POST b5b9c53 |
|---|---:|---:|
| `wasted_ms` | 22,715 | 23,132 |
| `memo_hits` | 1,821,178 | 1,859,716 |
| `memo_misses` | 678,664 | 646,975 |
| `keyed_calls` | 2,500,450 | 2,507,299 |

The +20min is **not** typed-cache memo collapse or materialization waste growth.

### 1.4 Governor receipt — cap-saturated on capped hosts (srv1-04, run 29850515721)

Post-#6848 floor on cgroup-capped runners now runs **at** the memory ceiling, not merely
near it. Run `29850515721` (PR #6999, merge of `origin/main` at `b339226`) on srv1-04:

| field | value |
|---|---|
| `budget` | 16,106,127,360 bytes (cgroup `memory.high`) |
| `peak_current` | 16,100,560,896 bytes (**99.97% of budget**) |
| `hard_backoffs` | 1 |
| `forced_serial` | 1 |
| `creep_backoffs` | 1 |
| `max_width_reached` | 1 |

Interpretation: #6956+#6972 brought peaks down from the 32 GB class, but the post-#6848
resolve walk keeps the floor **cap-saturated** even on 16 GB hosts — throttle cost on the
time axis, and one bad allocation from exit-137. Pairs with §1.1–1.3 as a memory-beside-time
receipt for this bisect lane (not a separate fix target in #6999).

### 1.5 Advisory diagnostic volume (typecheck overhead signal)

| | PRE 07-19 | POST |
|---|---:|---:|
| `advisory(typecheck): … unlisted import use` lines | **3** | **4,936** |

#6848's namespace resolution emits `UnlistedImportUse` for cross-module refs without import
lines (advisory, non-blocking). Floor still **generates** every row during typecheck even when
`GUNBC_ADVISORY_ROWS=1` collapses rendering — **~5k extra diagnostic constructions per run**.

---

## 2. Mechanism (code path)

Every floor entry resolve (`resolve_entry_with_parse_cache` → `load_sources_for_entry_with_pool`):

1. **`extend_sources_to_both_closure_fixpoint`** — NEW with #6848. Iterates
   `extend_with_bare_reference_closure` (text-scan stripped modules + `tree_bare_census_for_root`
   → `pool_parse` on first touch) and `extend_with_reference_closure` to fixpoint. Pre-#6848
   loader followed **import edges only** (`resolve_transitively`).

2. **`reconcile_with_typed_cache`** — builds `build_symbol_index_for_reconcile` →
   `pool_qualified_fill` → **`pool_parse`** **before** `try_reconcile_all_cache_hits`. Even a
   full typed-cache hit paid the whole-pool census first (fix axis 2b in memory diagnosis).

`pool_parse` is memoized per process; the per-entry cost is the **repeated bare-reference
fixpoint walk** and the **unconditional qualified-fill build on cache-hit misses at reconcile
entry** (closure census still built even when all modules hit cache).

---

## 3. Fix (this PR — perf only, no binding change)

1. **Entry-closure memo** on `MultiEntryIndex`: cache `load_sources_for_entry_with_pool`
   results keyed by normalized entry path. Witnesses sharing an entry file reuse one closure
   walk (claim_executor already amortizes resolve per entry; this eliminates duplicate walks
   when the loader is re-invoked on the same path within a process).

2. **Call-order**: move `build_symbol_index_for_reconcile` to **after**
   `try_reconcile_all_cache_hits` — full-cache-hit reconciles skip `pool_qualified_fill`
   entirely.

**Not in scope:** resolver semantics (§13 unique-on-chain — stern-owl/stern-newt lanes),
`rc_map_insert` quadratic (bold-crane-271), suppressing `UnlistedImportUse` generation (separate
if still needed after these two).

**Regression oracle:** existing census / closure witnesses + `diverge=0` harness unchanged;
no edits to `03_resolve` / `symbol_index_fill` / binding rules.

---

## 4. Reproduction (log-diff harness)

```bash
# download arms
gh run view 29701881403 --log > /tmp/pre.log
gh run view 29819122813 --log > /tmp/post.log
# batch wall
rg 'PASS \[batch|claim_executor: batch' /tmp/{pre,post}.log
# fe_begin distribution
rg -o 'compile\.frontend\.begin t_ms=\d+' /tmp/post.log | sed 's/.*=//' | sort -n | tail
# materialization
rg 'floor materialization:' /tmp/post.log
```

---

## 6. Residual (post-#6999) — thread-local index reset (this PR)

#6999 (§3) fixed the *within-instance* cost (entry-closure memo, reconcile call-order). It did
**not** touch the *cross-instance* cost: `resolve_entry_graph` routes each resolve through
`process_shared_index`, a `thread_local!` `MultiEntryIndex` keyed by `(thread, canonical
roots)` — the entry-closure memo and `pool_qualified_fill` cache only pay off for calls landing
on the **same thread**.

`claim_executor::run_walk` partitions batch units into `memo_units` (main thread,
`run_memo_shared_claims`, shared `walk_memo` across the whole run) vs `thread_units` (each
individually `thread::spawn`'d — cold `MultiEntryIndex` per spawn). Routing is decided by
`RunnableResourceProfile.heavy_whole_tree_resolve`, OR-merged across a `group_batch_units`
coalescing group. `DagCompileCleanGate`, `SourceRootIngestGate`, and `RegenVerifyGate` already
declared `heavy_whole_tree_resolve: true`; `SelfHostReadsRealBytesGate`,
`SelfHostStalenessGate`, and `EmitHostGate` (batch 4) did not — so #6999's caches were built and
then discarded on each of their freshly spawned threads, and batch-4 wall time stayed pinned at
~18.7min (1123s) after #6999 merged, unchanged from the ~1111s pre-#6999 figure and far off the
pre-#6848 baseline of 165–342s.

**Fix:** flip `heavy_whole_tree_resolve: false → true` for the three batch-4 gates in
`gate_runnable_profile` (`src/v2/workflow/ci_floor_plan.dag`) — aligning them with the
already-established pattern, not a new mechanism. This routes their resolves onto the
main-thread memo path; `floor_heavy_resolve_chain_resource_edges` already serializes all
`heavy_whole_tree_resolve` gates pairwise (`floor_plan_at_most_one_heavy_gate_resolve_per_batch`,
checked by `witness_plan_serializes_heavy_resolves`), so the three newly-heavy gates each land
in their own batch rather than co-residing — sequential on the main thread, sharing the process
`walk_memo`/`process_shared_index` cache instead of paying a cold walk each.

**Not in scope (unchanged from §3):** resolver semantics, `rc_map_insert` quadratic,
`UnlistedImportUse` suppression.

**Verification:** no test/lens found asserting `heavy_whole_tree_resolve: false` for these three
gates (`ci_floor_plan_witness_test.dag` checked); `runnable_excludes_corpus_co_residence` gates
on `profile.memory`, orthogonal to this flag. Falsifiable claim pending: a real CI floor run
must show batch-4 wall time drop from the ~18.7min baseline — not yet obtained at PR-open time.

## 7. Provenance

- Log-diff receipts: bright-seal-219 (this session), by execution on CI logs.
- Memory-side bisection: eager-pike-178 / #6953 (`c10f4b091`).
- Parent coordination: sunny-wolf-225 mandate (msg_fdeee8c5).
- §6 residual fix: proud-bear-438 (dashboard `adhoc-21c65e1a-2ff`), PR #7030.

Related: [floor-memory-pool-parse-regression-diagnosis.md](floor-memory-pool-parse-regression-diagnosis.md) ·
[namespace-resolution-design.md](namespace-resolution-design.md) §PR-5b ·
[v1-run-stability-throughline.md](v1-run-stability-throughline.md).
