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

### 1.4 Advisory diagnostic volume (typecheck overhead signal)

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

## 5. Provenance

- Log-diff receipts: bright-seal-219 (this session), by execution on CI logs.
- Memory-side bisection: eager-pike-178 / #6953 (`c10f4b091`).
- Parent coordination: sunny-wolf-225 mandate (msg_fdeee8c5).

Related: [floor-memory-pool-parse-regression-diagnosis.md](floor-memory-pool-parse-regression-diagnosis.md) ·
[namespace-resolution-design.md](namespace-resolution-design.md) §PR-5b ·
[v1-run-stability-throughline.md](v1-run-stability-throughline.md).
