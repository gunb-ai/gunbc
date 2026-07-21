# Floor time +20min regression — diagnosis (bisected to #6848 namespace walks)

**Status:** LANDED (#6998 axis 2b + #6999 entry-closure memo, merge `dc2aa25684`); post-merge
validation read §5 below. Resolver semantics unchanged — perf-only caching/call-order.

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

### 1.4 Governor receipt — cap-saturated on capped hosts (16 GB `memory.high`)

Post-#6848 floor on cgroup-capped runners runs **at** the memory ceiling, not merely near it.
Representative runs (2026-07-21, `hard_backoffs=1 forced_serial=1` on every capped-host
floor observed today):

| run | host | budget | peak_current | peak RSS | batch-2 | batch-4 |
|---|---|---:|---:|---:|---:|---:|
| `29850515721` | srv1-04 | 16.1 GB | 16.10 GB (99.97%) | 13.4 GB | 591s | 1133s |
| `29855080611` | srv3-05 | 16.1 GB | 15.71 GB (97.5%) | 13.6 GB | 510s | 1125s |
| `29860090806` | srv1-02 | 16.1 GB | 15.08 GB (93.6%) | 13.6 GB | 573s | 1346s |

Contrast POST-regression `29819122813` on srv3-07: budget **96 GB** (`MemAvailable`),
peak 12.5 GB, **no** backoffs — same post-#6848 walk class, different host envelope.

Interpretation: #6956+#6972 brought peaks down from the 32 GB class, but the post-#6848
resolve walk keeps capped hosts **cap-saturated** — throttle cost on the time axis and one
bad allocation from exit-137. Pairs with §1.1–§1.3 and §5.2 as the memory-beside-time
residual (not addressed by #6998/#6999).

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

## 3. Fix (LANDED #6998 + #6999 — perf only, no binding change)

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

## 5. Post-landing validation read (merge `dc2aa25684`, 2026-07-21)

Compared batch walls using the §4 harness (`claim_executor: batch 2` → `PASS [batch 2]
discovery-corpus`; `claim_executor: batch 4` → last `PASS [batch 4]` gate). POST-FIX arm
uses run `29855080611` (`a475d2e8e0`, identical floor-time code to merge `dc2aa25684`);
post-merge main run `29860557952` was still in progress at write time.

### 5.1 Measured deltas (run ids)

| arm | run | host class | batch-2 | batch-4 | Δ₂ vs PRE | Δ₄ vs PRE |
|---|---|---:|---:|---:|---:|---:|
| PRE (ec22e8f) | `29763408563` | capped | **202s** | **342s** | — | — |
| PRE (07-19) | `29701881403` | capped | 226s | 165s | — | — |
| POST (#6848) | `29819122813` | 96 GB | **503s** | **1111s** | +301s | +769s |
| POST-FIX (#6998+#6999) | `29855080611` | 16 GB capped | **510s** | **1125s** | +308s | +783s |
| POST-FIX vs POST | — | — | **+7s (+1%)** | **+14s (+1%)** | — | — |

Total regression vs PRE ec22e8f: **+301s batch-2 + +769s batch-4 ≈ +18 min** (POST).
After #6998+#6999 on a comparable capped host: **~0% batch-wall recovery** (+7s/+14s,
within run-to-run noise).

### 5.2 Verdict — recovery vs residual

**Recovered (mechanism-level, not batch-wall-visible):**

- **Axis 2b (#6998):** `pool_qualified_fill` deferred past `try_reconcile_all_cache_hits` —
  removes whole-pool census on all-cache-hit reconciles. Oracle: `reconcile_defer_hot_hit_matches_cold_oracle`.
  Batch impact limited: discovery witnesses are predominantly cold typed-cache misses.
- **Entry-closure memo (#6999):** eliminates duplicate `extend_sources_to_both_closure_fixpoint`
  when the same entry is loaded twice in one worker (discovery → resolve on same path).
  Oracle: `entry_closure_sources_memo_reuses_name_derived_walk`. Batch-2 impact limited:
  discovery loads each entry file once per worker process — memo hit rate near zero on the
  ~2100-witness corpus path.

**Residual (dominant, ~18 min still owed vs PRE ec22e8f):**

| mechanism | est. batch cost | notes |
|---|---:|---|
| #6848 bare-reference fixpoint (once per entry) | ~5 min batch-2 | ~140ms/witness × ~2100; unchanged by memo |
| #6848 qualified-fill on reconcile miss | ~13 min batch-4 | deferral skips only all-hit path |
| Cap-saturation throttle (16 GB hosts) | unpriced additive | §1.4: `hard_backoffs=1 forced_serial=1`, peak 14–16 GB at budget |
| `UnlistedImportUse` advisory generation | typecheck overhead | §1.5: ~5k extra rows/run |

**Next scoped dispatch (not this lane):** attack the once-per-entry bare-reference fixpoint
(namespace-resolution-design §PR-5b residual) and/or cap-headroom (governor envelope vs walk
RSS); `rc_map_insert` quadratic (bold-crane-271) and advisory suppression are separate rows.

---

## 6. Provenance

- Log-diff receipts: bright-seal-219 (this session), by execution on CI logs.
- Post-landing validation §5: bright-seal-219, runs `29763408563` / `29819122813` / `29855080611` (2026-07-21).
- Memory-side bisection: eager-pike-178 / #6953 (`c10f4b091`).
- Parent coordination: sunny-wolf-225 mandate (msg_fdeee8c5); lane close validation (msg_1879f052).

Related: [floor-memory-pool-parse-regression-diagnosis.md](floor-memory-pool-parse-regression-diagnosis.md) ·
[namespace-resolution-design.md](namespace-resolution-design.md) §PR-5b ·
[v1-run-stability-throughline.md](v1-run-stability-throughline.md).
