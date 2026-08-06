# Entry-view assembly memos — measured receipt

**Lane:** floor-prep-tax-program P1 follow-on (shared entry-view construction)
**Predecessor:** [fill-composition-overlay-direction-receipt](fill-composition-overlay-direction-receipt.md)
**Retained evidence:** `receipts/entry-view-assembly-direction/`
**Status:** **cohort verdict — MISS** (representative affected-floor run not started; cohort miss is decisive for this direction)

## Headline finding

**`root_env` hit at 92% (672 hits / 57 misses) and wall still moved only −1.5%.**

That is the load-bearing number. It does not say "wrong keys — try better keys." It says **entry-assembly memoization as a class is not where the time goes**: a cache that works, keyed correctly, at a 92% hit rate, and the eliminated work is still too small to move wall materially (+1.8% peak RSS for the retained state). This converges with earlier slice work: the term scaling with both entry count and closure size lives elsewhere (`load`, closure `symbol_index`), not in per-entry assembly env reuse.

The `closure_env` and `rewired` counters (0 hits / 50 misses each) are secondary — they show those particular memos do not repeat on this roster. They are not the reason to close the direction; `root_env` is.

## What this receipt is decisive about

| Claim | Supported? |
|---|---|
| Entry-view assembly memos are not worth shipping on this evidence | **yes** |
| Better memo keys would flip the verdict | **no** — `root_env` already hits at 92% |
| A representative affected-floor run would rescue the direction | **unknown, not measured** — cohort miss is decisive for memoization; floor bar not attempted |

## Bounds (read before citing numbers)

- **Roster:** fixed 50-entry post-#7836 cohort (`cohort.tsv`) — not whole-tree, not affected-set representative
- **Invocation:** one process, one `MultiEntryIndex`, interleaved `base-r1 / after-r1 / base-r2 / after-r2`
- **Tree:** `3b10d3b27`; one variable via `base-arm-revert.patch` (memos + receipt line only; post-#7836 fill composition retained)
- **Host:** BuildBuddy remote executor — local release `v1-compiler` link OOM'd under session cgroup cap (~32 GiB); fleet SIGKILL condition, not a per-session flake
- **Not run:** representative affected-floor run (≥10% / ≥3 min bar); cohort miss closes the direction before floor measurement
- **Binaries:** `beb49950…` (base) · `2edd2f57…` (after) — see `binary_sha256.txt`

## Mechanism (deleted — retained for receipt context only)

Three process-lifetime memos on `MultiEntryIndex`, keyed by semantic input identity:

| Memo | What it cached | Safety basis |
|---|---|---|
| `closure_assembly_env_memo` | `ClosureAssemblyEnv` per closure digest | Same digest ⇒ same closure census and symbol-index inputs |
| `root_assembly_env_memo` | per-root composed symbol index + variant-owner facts | Keyed by `(root, global_bare digest)`; overlay fill composition unchanged |
| `rewired_modules_memo` | import-string / type-env / func-env rewiring product per module digest | Same module content key ⇒ same rewiring product |

Instrumentation: `[entry-view-assembly]` line at end of `claim_batch` (after arm only).

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

**Memo population (after arm, both repros identical):**

```
closure_env keys=50 hits=0 misses=50
root_env    keys=57 hits=672 misses=57   ← 92% hit rate; wall −1.5%
rewired     keys=50 hits=0 misses=50
```

## Equivalence

All four arms: **48 PASS / 2 FAIL**, identical failure names in every arm (`artifact_fs_roundtrip_holds`, `build_artifact_corruption_probe_holds` — pre-existing hermetic refusals, same pair as the fill-composition receipt). Equal pass/fail counts alone would not establish validity; matching names across all four arms does.

## Benefit bar

| Criterion | Required | Observed | Verdict |
|---|---|---|---|
| Cohort wall materially below baseline | yes | −1.5% (~1.5 s on ~98 s mean) | **MISS** |
| Σ exclusive assembly materially below | implied | −4.7% (~1.2 s) | **MISS** |
| Peak RSS ≤ post-#7836 envelope (~5,803,012 kB) | yes | 6,081,740 kB (+4.8% vs envelope) | **MISS** |
| Semantic outcomes identical | yes | identical names all arms | PASS |
| Floor ≥10% or ≥3 min | yes | not run — cohort miss decisive | N/A |

Green CI is **not** the acceptance oracle; this receipt is.

## Verdict

**Close.** Process-lifetime memos deleted: +~280 MiB peak for −1.5% wall and −4.7% assembly — far below the ≥10% / ≥3-minute bar, and the discriminating evidence is that even a 92%-hit cache does not move wall enough to matter. Do not re-attempt entry-assembly memoization on better keys; redirect to wherever `load` and closure `symbol_index` scale.

**Disposition (2026-08-06):** `origin/main` integrated post-measurement; all three process-lifetime memos, receipt instrumentation, and `entry_view_assembly_memo_witness_test` **deleted** from the branch. Receipts retained; mechanism does not ship.

Raw arms: `remote_cohort_run3.log` (includes embedded TSV + `summary.json`).
