# Whole-tree compile-clean time diagnosis — memory-thrash, not compute

**Status:** diagnosis complete (2026-07-21, session lively-koi-497). Locate-only — no fix
landed. Connects to [v1 run-stability throughline](v1-run-stability-throughline.md) and
[floor memory pool_parse regression](floor-memory-pool-parse-regression-diagnosis.md).

**One-line verdict:** Whole-tree compile-clean is **~3 min of compute at ~4.5–6.5 GiB RSS**
when memory is available. The operator's 60+ min / 270-min-cap pain is a **memory-footprint
defect amplified into swap-thrash** on the **same-process floor** (batch-1 receipt + batch-2
discovery corpus), not inherent reconcile cost. The **dominant compile-clean reconcile RSS
driver today is #6848's whole-pool `pool_parse` path** (3.1× step vs pre-#6848); residual
M1 inductive-field duplication is **not** the current compile-clean footprint (pre-#6848
baseline was already 1.35 GiB post-M1).

---

## 1. Repro — whole-tree compile-clean (the CI path)

Build locally (not the PATH cargo shim):

```bash
CTRL_BUILD_WRAP_CARGO=0 /opt/cargo/bin/cargo build -p v1-compiler --release \
  --features text_lookup_work_counter \
  --bin claim_executor --bin compile_clean_diagnostic_histogram
```

**Floor receipt path (what CI batch-1 installs):**

```bash
GUNBC_CI_COMPILE_CLEAN_COLD_CONTROL=1 GUNBC_FLOOR_GANTT=1 \
  ./target/release/claim_executor \
    --source-root dag --source-root src/v2 \
    --plan-entry src/v2/workflow/ci_floor_plan.dag \
    --plan-function gunbc_ci_plan_artifact_batches
```

**Resolve oracle (same closure, `compile_to_resolved`, phase gantt marked):**

```bash
GUNBC_FLOOR_GANTT=1 ./target/release/compile_clean_diagnostic_histogram
```

Harness: `scripts/profile_whole_tree_compile_clean.sh` (runs both + prints a table).

---

## 2. Phase attribution (current main, by execution)

`compile_clean_diagnostic_histogram` on this tree (33 GiB cgroup, 2026-07-21):

| Phase | Wall | Share | RSS at reconcile |
|---|---:|---:|---:|
| parse (`frontend`) | 4.1 s | 2% | 687 MiB |
| normalize | 0.8 s | <1% | 687 MiB |
| **reconcile (typecheck-env)** | **156 s** | **79%** | **4518 MiB** |
| analyses | 0.5 s | <1% | 4518 MiB |
| **total** | **197 s** | | **4518 MiB peak** |

Floor receipt path (`claim_executor`, same tree): emit leg ~21 s; resolve/reconcile is
**unmarked** on the `floor_compile_clean_emit_ok_via_index` path (only `[gantt] compile.emit.*`
lines appear). Wall for receipt install ≈ 2.1 min after plan prelude.

**CI corroboration (fleet + dashboard):**

| Run | Slot | Compile-clean receipt | Peak RSS at receipt |
|---|---|---:|---:|
| 29828873976 | unconstrained (`memory.high=none`) | 12:19:05 → 12:21:46 (**2.7 min**) | ~6.3 GiB (`emit.done`) |
| 29834202745 | **15 GiB high / 16 GiB max** (srv2-05) | 13:27:21 → 13:29:49 (**2.5 min**) | ~6.2 GiB (`t=2m`) |

---

## 3. Footprint driver — bisect table (controlled pair)

Same harness (`compile_clean_diagnostic_histogram`), same tree, only binary differs:

| Arm | Commit | pool_parse | Wall | Reconcile RSS | Δ RSS vs pre-6848 |
|---|---|---|---:|---:|---:|
| **pre-#6848** | `ec22e8fabb` | none | **85 s** | **1352 MiB** | — |
| **#6848** | `c87d1b0d33` | full-body | **160 s** | **4174 MiB** | **+3.1×** |
| **#6956** | `d54574a5c0` | heads-only | **173 s** | **4374 MiB** | **+3.2×** |
| **main** | HEAD (post-#6956) | heads-only | **198 s** | **4518 MiB** | **+3.3×** |

**Reading:**

1. **#6848 introduced a step-change** in both time (~1.9×) and reconcile RSS (~3.1×). This
   is the whole-pool `pool_parse` → `pool_qualified_fill` → `build_symbol_index_for_reconcile`
   path (`cli_run.rs:6339–6514`), which parses **all ~2308 pool `.dag` modules** before
   reconcile regardless of entry closure size.

2. **#6956 heads-only did not materially shrink compile-clean RSS** (4374 vs 4174 MiB —
   within run variance). Heads-only cut the *memory doc's* full-body AST retention class;
   compile-clean reconcile still pays whole-pool **heads parse + census fill** on every run.

3. **Residual M1 inductive-field duplication is NOT the compile-clean footprint driver
   now.** Pre-#6848 reconcile RSS was already **1.35 GiB** (M1b/M1b-2 landed in #6528).
   The +3.1 GiB step is entirely #6848 namespace infrastructure, not re-opened env
   duplication.

4. Pool size today: **2308** `.dag` files under `dag/` + `src/v2/` (~5.0 MB source).

---

## 4. Thrash mechanism — where 46–60 min comes from

**CI slot ceiling** (`gunbc.runner_slot_allocation`): **16 GiB `memory.max`**, **15 GiB
`memory.high`** (reclaim throttle line), 32 GiB swap per slot.

### 4a. Compile-clean alone does NOT thrash on fleet

Fleet run **29834202745** (srv2-05, proper 15 GiB high):

- Receipt ok at **t=2.5 min**, RSS **~6.2 GiB**, **swap=0**.
- Batch 2 starts t=3 min; RSS **6.8 → 8.1 GiB** by t=6 min, still **swap=0**.

Compile-clean fits the 16 GiB slot with headroom.

### 4b. Whole-floor same-process accumulation DOES thrash

`claim_executor` installs the batch-1 receipt then runs batch-2 discovery corpus (**~2132
witnesses**) in the **same process**, reusing `process_shared_index` / `typed_module_cache`.
Retention grows monotonically — the v1-run-stability failure mode.

**Fleet run 29215148169** (2026-07-13, srv2-03, 15 GiB high — canonical receipt):

| Time | Event | RSS | Swap |
|---|---|---:|---:|
| t≈3 min | compile-clean receipt ok | ~6 GiB | 0 |
| **t=9 min** | RSS **pins at memory.high** | **15.0 GiB** (`16104591360 B`) | **943 MiB** |
| t=10 min | sustained thrash | 15.1 GiB | **2.1 GiB** |
| end | governor receipt | peak **= memory.high** | `forced_serial=1`, **`hard_backoffs=474`**, `budget_exceeded=1` |

**Dashboard / falsifier unconstrained runs** (`memory.high=none`, 33+ GiB budget) grow to
**28–43 GiB RSS** without OOM-kill — swap stays 0 on those boxes, but reconcile wall blows
out (falsifier run 29822303765: `reconcile.done` at **46 min**, **43 GiB RSS**). That is
**not** a compile-clean-isolated measurement; it is the **cumulative floor process**.

**Swap onset on unconstrained CI (29828873976):** first non-zero swap at **t=11 min**,
RSS **~9.5 GiB** (batch-2 well underway; compile-clean finished at t=2.7 min).

**Mechanism:** direct reclaim at `memory.high` → swap-backed allocation →
`forced_serial=1` governor → serial witness resolves at swap speed → step-cap death. Not
DRAM-bandwidth merge-wave contention (no evidence of concurrent floor jobs sharing a cgroup
in these logs; `high_events` / PSI track **this** job's reclaim pressure).

---

## 5. Mitigation headroom (quantified)

| Available memory | Compile-clean alone | Whole floor (batch 1 + 2) |
|---|---|---|
| **≥8 GiB** (est.) | **Fits** (~4.5 GiB peak) | **Fails** — needs typed-cache retention for full corpus |
| **16 GiB slot (fleet)** | **~2.5–3 min**, swap=0 | **Thrashes** at t≈9 min (pins 15 GiB high) |
| **33 GiB (dashboard)** | **~3.3 min** | Completes but RSS climbs to 28–43 GiB; slow reconcile tails |

**Interim wins (no footprint fix):**

- **Affected-set scoping** (already live): docs-only / scoped diffs skip batch-2 corpus.
- **Bigger runner** buys whole-floor headroom linearly until retention exceeds new high line
  (not a fix — shifts the wall).
- **Footprint fix lane:** (2a) lazy / closure-scoped `pool_parse` (namespace §PR-5b); (2b)
  M2 typed-cache eviction (#5886 shelved) for batch-2 accumulation.

---

## 6. Answers to the sharpened brief

| Question | Answer |
|---|---|
| **Dominant reconcile footprint driver?** | **#6848 whole-pool `pool_parse` path** (+3.1 GiB). Not residual inductive-field duplication (pre-#6848 was 1.35 GiB post-M1). #6956 heads-only: no meaningful RSS win on this path. |
| **RSS step at #6848 after #6956?** | **Yes: 1.35 → 4.2–4.5 GiB** (3.1–3.3×). Step landed at #6848; #6956 flat. |
| **Thrash mechanism?** | **memory.high reclaim + swap** when same-process floor RSS exceeds **15 GiB**. Compile-clean alone peaks ~6 GiB. Tipping point for whole floor on fleet: **~t=9 min**, RSS pins at **16106127360 B**. |
| **Mitigation headroom?** | Compile-clean needs **~5 GiB**; thrash-free whole floor needs **≫15 GiB** today (or eviction / process split). Observed thrash-free compile-clean up to **33 GiB** box. |

---

## 7. Provenance

- Bisect arms + histogram timings: executed 2026-07-21 on this tree (session lively-koi-497).
- Fleet receipts: GitHub Actions runs 29834202745, 29215148169, 29828873976.
- Prior art: [v1-run-stability-throughline](v1-run-stability-throughline.md) §0–§1 (M1 env
  duplication, governor `forced_serial=1`); [floor-memory-pool-parse-regression-diagnosis](floor-memory-pool-parse-regression-diagnosis.md) (memory bisect to #6848).
