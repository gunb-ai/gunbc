# Receipt — P1 retention vs drain cohort receipt (authoritative)

**Status:** measurement receipt, timestamped 2026-08-03. **DESIGN.md + `docs/plans/floor-prep-tax-program.md` (PR #7721) remain authority** — this is a dated receipt, not a fact ledger.

**Lane:** floor prep-tax program, P1 · **Subject:** does schedule-derived retention (#7129 eviction) cause the ~1s per-entry "tax" observed across the floor discovery path, or does the tax persist independent of eviction? · **Deliverable:** measurement only, ACCEPT/REJECT verdict against the parent-stated criterion. No retention-mechanism change lands in this PR.

**Main SHA measured:** `b53aec1dff6be7ccb28cb6683e88faa86d7a108c`.

**Vehicle:** new `p1_cohort_probe` binary (`src/v1/stage0/src/bin/p1_cohort_probe.rs`) — a thin CLI front end that calls the **same production entrypoint** `run_discovery_corpus_with_options` that `claim_executor` drives for the real floor path, with `DiscoveryWidthPolicy::Adaptive` (the governor-admitted width the fleet actually runs under — schedule-retention only arms inside this branch's width-1 special case), over an explicit fixed 50-entry cohort instead of a full-corpus scan. This is not a reimplemented/toy mechanism; it is the identical discovery-corpus/schedule-retention code `claim_executor` uses, scoped for direct entry-by-entry A/B comparability. `claim_batch` was investigated and ruled out as a vehicle: it does not call `run_discovery_corpus_with_options` at all (a different assembly-cost-decomposition harness — see `docs/plans/per-entry-assembly-decomposition-measurement.md`).

**Cohort:** 50 entries, `src/v1/stage0/src/bin/p1_cohort_roster.txt`, sourced verbatim from `docs/plans/receipts/entry-graph-union-slice2/receipt-post-merge-representative-50.json` (`summary.entry_timings[].entry`) — the same representative-50 module-sharing cohort an earlier slice-2 receipt already established as sharing the module universe.

**Host regime (honesty):** dashboard session container, uncapped-relative-to-fleet cgroup (see cgroup readback below); this is a regime-relative measurement, not a byte-match to the fleet width-1 receipt in `docs/plans/m2-floor-retention-measurement-receipt.md`.

---

## 0 — Reproduction (verbatim invocations)

```bash
git rev-parse HEAD   # b53aec1dff6be7ccb28cb6683e88faa86d7a108c
CTRL_BUILD_MODE=local RUSTC_WRAPPER= CARGO_BUILD_JOBS=4 \
  ctrl-build --local -- cargo build -p v1-compiler --bin p1_cohort_probe --release
cp -f target/release/p1_cohort_probe target/release/p1_cohort_probe-snapshot
# sha256: 54566635522137569193c1e56451ccc1a38c6e4e5595cc6fca57aee2cc26943a
```

**Mode A — default eviction ON (production default):**

```bash
export GUNBC_FLOOR_DRAIN_RETENTION=1
export GUNBC_P1_COHORT_RECEIPT=1
LOG=docs/probes/p1_retention_cohort_20260803T022255Z/mode_a_eviction_on.log
target/release/p1_cohort_probe-snapshot > "$LOG" 2>&1
```

**Mode B — retain-all measurement pole:**

```bash
export GUNBC_FLOOR_DRAIN_RETENTION=1
export GUNBC_P1_COHORT_RECEIPT=1
export GUNBC_SCHEDULE_RETENTION_EVICT=0
LOG=docs/probes/p1_retention_cohort_20260803T022255Z/mode_b_retain_all.log
target/release/p1_cohort_probe-snapshot > "$LOG" 2>&1
```

Raw logs and extracted `[p1-cohort]` lines are checked in under `docs/probes/p1_retention_cohort_20260803T022255Z/`.

---

## 1 — Mechanism receipts (both modes armed correctly)

Mode A:
```
[floor-drain] schedule-retention ARMED: entries=50 modules_refcounted=711 evict_enabled=true
```

Mode B:
```
[floor-drain] SCHEDULE-RETENTION EVICTION DISABLED (GUNBC_SCHEDULE_RETENTION_EVICT=0): arming + counting only, dropping NOTHING — the retain-all measurement pole (pre-M2 process-lifetime retention). schedule_evictions will be 0 BY CONSTRUCTION; unset the env to restore eviction.
[floor-drain] schedule-retention ARMED: entries=50 modules_refcounted=711 evict_enabled=false
```

Both modes armed over the identical 50-entry/711-module schedule. Mode B's loud diagnostic fired as required and `modules_evicted=0 graphs_evicted=0` held on every one of its 34 completed `[p1-cohort]` lines — confirms Mode B genuinely never evicted (not a partial/soft eviction).

---

## 2 — Completeness (honest accounting, not fabricated)

**Mode A completed all 50 entries** (414 witness rows, 1 pre-existing unrelated failure — `witness_observed_hostname_reads_typed_op_hermetic`, an environment-specific hostname-observation test, not a retention/timing concern; identical failure occurs in Mode B at the same entry, confirming it is unrelated to eviction mode).

**Mode B was killed by the OOM killer at entry 34/50** (`exit=137` / SIGKILL). Its `process_rss` climbed monotonically and unboundedly across every entry — 1.72 GB at entry 1 to 5.33 GB at entry 34, `cgroup_memory_current` reaching 8.07 GB — because retain-all, by construction, never frees a module. This is itself a first-class, expected finding of the retain-all pole (unbounded process-lifetime retention is exactly what M2's schedule-derived eviction exists to prevent), not an instrumentation bug. **Entries 35-50 have no Mode B data and none is fabricated or interpolated here.** All comparison below is restricted to the valid overlapping range, entries 1-34.

---

## 3 — Quantitative A/B comparison (entries 2..34, the valid overlapping range)

| | Mode A (eviction ON) | Mode B (retain-all) |
|---|---|---|
| mean `wall_ms` | 2045.4 | 1936.7 |
| mean `resolve_ms` | 1943.9 | 1891.6 |
| median `resolve_ms` | 1076 | 1054 |
| min `resolve_ms` | 869 | 908 |

Per-entry, the two modes track each other almost exactly (full table computed from the raw logs, entries 1-34):

```
entry  A_wall  B_wall  A_resolve  B_resolve  B_rss_MB
    1    2409    2247       2372       2236      1716
    2    1155    1024       1076       1019      1716
    3    1175    1009       1090       1001      1716
   10    1768    1750       1728       1735      2025
   11   13131   12397      12843      12186      2681   (heavy entry, both modes)
   18    3669    3393       3594       3363      3241
   32   11368   10737      10378       9911      5034   (heavy entry, both modes)
   34    1140     913       1070        908      5085
```

(Full 34-row table and extraction script output retained in the probe log directory.)

The heavy entries (11, 18, 21, 27, 29, 32) are heavy in **both** modes at nearly the same magnitude — eviction is not what is driving their cost, and retain-all does not relieve them either. The steady-state ~1-1.1s-per-entry baseline (entries 2-9, 12-17, 19-20, etc.) is likewise statistically indistinguishable between A and B; if anything Mode B trends marginally faster (noise-level, consistent with slightly fewer cache-eviction/re-resolve cycles, not a collapse).

**No entry in 2..34 shows the tax collapsing toward near-zero under Mode B.** Retaining every module for the full cohort life (growing to 5+ GB RSS) does not make entries 2..34 cheap.

---

## 4 — Verdict

**Parent's stated ACCEPT criterion:** *"ACCEPT iff entries 2..N sharing the module universe stop repaying ≈2s tax under Mode B."*

**REJECT.** Entries 2..34 do not stop repaying the tax under Mode B — mean/median `wall_ms` and `resolve_ms` are statistically indistinguishable between Mode A (eviction on) and Mode B (retain-all, arming confirmed, zero evictions confirmed, OOM-killed only after unbounded growth at entry 34). Disabling schedule-retention eviction entirely does not collapse or materially reduce the per-entry tax on this cohort.

Per the parent's own routing instruction, this receipt reports the result the A/B data shows, not a pre-decided answer (the earlier `claim_batch` framing that was retracted mid-task is not relied on here — this conclusion is derived solely from the floor-path A/B receipt above). **The ~1s-2s per-entry tax is not caused by schedule-derived module eviction/retention.** The redirect is to assembly/materialization reuse (as the parent's fallback routing anticipated, but arrived at here by measurement, not assumption) — consistent with the open thread in DESIGN.md's "duplicate work = the content-hash qualification of `Materialization`" and the per-entry assembly decomposition receipts already in `docs/plans/`.

**No retention mechanism change is implemented in this PR** — per the explicit brief, this receipt's job was discrimination, not a fix.

---

## 5 — Artifacts

- `src/v1/stage0/src/bin/p1_cohort_probe.rs`, `p1_cohort_roster.txt` — the probe vehicle (kept in tree; reusable for future retention receipts).
- `src/v1/stage0/src/cli_run.rs` — `[p1-cohort]` per-entry instrumentation, gated independently by `GUNBC_P1_COHORT_RECEIPT=1` (kept separate from the existing `GUNBC_FLOOR_DRAIN_RETENTION` switch — two distinct facts, two distinct toggles, DESIGN §3).
- `docs/probes/p1_retention_cohort_20260803T022255Z/` — raw Mode A/B logs and extracted `[p1-cohort]` line sets.
