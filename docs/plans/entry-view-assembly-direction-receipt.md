# Entry-view assembly memos — measured receipt

**Lane:** floor-prep-tax-program P1 follow-on (shared entry-view construction)
**Predecessor:** [fill-composition-overlay-direction-receipt](fill-composition-overlay-direction-receipt.md)
**Retained evidence:** `receipts/entry-view-assembly-direction/`
**Status:** **cohort verdict — MISS** (representative affected-floor run not started; cohort miss is decisive)

## Mechanism

Three process-lifetime memos on `MultiEntryIndex`, keyed by semantic input identity:

| Memo | What it caches | Safety basis |
|---|---|---|
| `closure_assembly_env_memo` | `ClosureAssemblyEnv` per closure digest | Same digest ⇒ same closure census and symbol-index inputs |
| `root_assembly_env_memo` | per-root composed symbol index + variant-owner facts | Keyed by `(root, global_bare digest)`; overlay fill composition unchanged |
| `rewired_modules_memo` | import-string / type-env / func-env rewiring product per module digest | Same module content key ⇒ same rewiring product |

Instrumentation: `[entry-view-assembly]` line at end of `claim_batch` (after arm only).

## Subject and arms

One tree (`3b10d3b27`), one variable: `base-arm-revert.patch` removes the three memos and receipt line only (post-#7836 fill composition retained). Built on BuildBuddy remote executor (local session cgroup cannot link release `v1-compiler`).

- **Roster:** same 50-entry post-#7836 cohort (`cohort.tsv`)
- **Invocation:** one process, one `MultiEntryIndex`, interleaved `base-r1 / after-r1 / base-r2 / after-r2`
- **Binaries:** `beb49950…` (base) · `2edd2f57…` (after) — see `binary_sha256.txt`

## Cohort result (derived from `summary.json` only)

| Receipt field | base r1 / r2 | after r1 / r2 | delta |
|---|---:|---:|---:|
| elapsed wall (ms) | 100,000 / 96,000 | 96,000 / 97,000 | **−1.5%** |
| Σ exclusive assembly (ms) | 29,189 / 27,205 | 26,887 / 26,873 | **−4.7%** |
| additive resolve, 50 resolves (ms) | 52,351 / 50,216 | 49,602 / 49,635 | **−3.2%** |
| peak RSS (kB) | 5,976,883 / 5,976,883 | 6,081,740 / 6,081,740 | **+1.8%** |

| Exclusive assembly row | base r1 / r2 | after r1 / r2 | delta |
|---|---:|---:|---:|
| root_symbol_index | 406.4 / 293.4 | 298.1 / 292.6 | **−15.6%** |
| symbol_index_merge | 342.1 / 292.2 | 293.8 / 284.4 | −8.8% |
| symbol_index (closure census) | 4,104.6 / 3,780.9 | 3,698.9 / 3,696.7 | −6.2% |
| root_variant_base | 4,322.0 / 4,098.4 | 4,099.5 / 4,013.0 | −3.7% |
| rewire_import_str | 6,547.2 / 6,042.7 | 5,967.3 / 6,006.8 | −5.1% |

**Memo population (after arm):**

```
closure_env keys=50 hits=0 misses=50
root_env keys=57 hits=672 misses=57
rewired keys=50 hits=0 misses=50
```

`closure_env` and `rewired` never hit on this cohort — each of the 50 entries presents a distinct key. `root_env` hits heavily (672 hits / 57 misses) but the saved work is small relative to total resolve cost dominated by `load` (~29s) and closure `symbol_index` (~3.7s).

## Equivalence

All four arms: **48 PASS / 2 FAIL**, identical failure names (`artifact_fs_roundtrip_holds`, `build_artifact_corruption_probe_holds` — pre-existing hermetic refusals, same as fill-composition receipt).

## Benefit bar

| Criterion | Required | Observed | Verdict |
|---|---|---|---|
| Cohort wall materially below baseline | yes | −1.5% (~1.5 s on ~98 s mean) | **MISS** |
| Σ exclusive assembly materially below | implied | −4.7% (~1.2 s) | **MISS** |
| Peak RSS ≤ post-#7836 envelope (~5,803,012 kB) | yes | 6,081,740 kB (+4.8% vs envelope) | **MISS** |
| Semantic outcomes identical | yes | identical | PASS |
| Floor ≥10% or ≥3 min (not measured) | yes | not run — cohort miss is decisive | N/A |

Green CI is **not** the acceptance oracle; this receipt is.

## Verdict

**Close or shrink.** The memos add process-lifetime retained state (+~280 MiB peak on this cohort) for ~1.5% wall and ~4.7% assembly — far below the ≥10% / ≥3-minute floor bar. The discriminating memo counters show the expensive keys (`closure_env`, `rewired`) do not repeat across this roster; only `root_env` repeats, and that row was already modest after fill-composition.

**Disposition (2026-08-06):** `origin/main` integrated post-measurement; all three process-lifetime memos, receipt instrumentation, and `entry_view_assembly_memo_witness_test` **deleted** from the branch. Receipts retained; mechanism does not ship.

Retained-for-promise is exactly the failure mode named in review: do not keep several process-lifetime memos on this evidence.

Raw arms: `remote_cohort_run3.log` (includes embedded TSV + `summary.json`).
