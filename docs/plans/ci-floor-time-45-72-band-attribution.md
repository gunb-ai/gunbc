# CI floor time audit — attribute the 45–72 min band

**Status:** measurement receipt, 2026-07-23 (session vivid-fox-471). **DESIGN.md + carriers remain
authority** — this doc is a timestamped profiling receipt, not a fact ledger. Dissolves when
`realization_measurement_loop` Phase-0 lands a durable `.dag`-native Gantt carrier that supersedes
prose receipts (same trigger as [ci-floor-fractal-gantt.md](ci-floor-fractal-gantt.md)).

**One-line verdict:** The **ci job** (not whole workflow) clusters **44–55 min** on current main;
the **45–72 min workflow band** is mostly **build + ci + deploy_dashboard**. Inside the ci job,
**~35 min** is `claim_executor` floor time, dominated by **discovery resolve (~12 min wall,
~620s serial-sum)** and **self-host gate chain batches 5–6 (~14 min combined)**. The #6848
namespace-walk regression (+24 min vs PRE) is **partially recovered** on effectful gates (#7030,
−11 min) but **discovery resolve (+520s vs PRE)** and **new self-host enrollment (+14 min)**
keep the band ~25–30 min above the Jul-20 PRE baseline.

---

## 1. Band census (what “45–72 min” measures)

Measured over the last 60 `ci.yml` runs on **main** (2026-07-22/23 window).

| Grain | n | min | p25 | median | p75 | max | 45–72 band |
|---|---:|---:|---:|---:|---:|---:|---|
| **Workflow wall** (build+ci+deploy) | 52 | 23m | 37m | 55m | 64m | 98m | 29/52 (56%) |
| **ci job only** | 52 | 3m* | 20m | 44m | 48m | 55m | 21/52 (40%) |

\*Floor of 3m = fast-lane / early-fail runs.

**Interpretation:** Operator-facing “CI takes ~an hour” is the **workflow** number (median 55m).
Performance work should track the **ci job** (median 44m, p75 48m) — build is ~1m and deploy is
~2m and should not pollute floor attribution.

**Not the same as ~72 min emit:** `gunbc_ci_witness_corpus_only_batches_note` cites **~72 min** for
the whole-tree `--target dag` compile-clean gate as **pre-push infeasible** scope — that is
**emit wall inside `dag_compile_clean_gate_passes` on a cold whole-tree closure**, not the ci job
wall. On current main the compile-clean **batch wall is ~1 min** because CI runs import-closure
scoped compile (`tools.dag_compile_clean_scope`), not whole-tree emit every PR.

---

## 2. ci job decomposition (receipt runs)

Log-diff harness: parse `claim_executor` invocations by
`--plan-function gunbc_ci_regen_floor_batches` vs `gunbc_ci_floor_batches`; batch walls from
`claim_executor: batch N` → next batch start (or last `PASS [batch N]`).

| arm | run | ci job | overhead† | regen | floor | cap-sat |
|---|---|---:|---:|---:|---:|---|
| PRE-#6848 | `29763408563` | **17.9m** | ~7m | 2.5m | 10.8m | no |
| POST-#6848 | `29819122813` | **42.1m** | ~11m | 3.6m | 28.9m | no |
| POST-#6998/#6999 | `29855080611` | — | — | — | 28.9m‡ | **yes** |
| POST-#7030 | `29880571548` | — | — | 3.4m | 36.9m | no |
| main Jul-23 | `29970583893` | **47.7m** | ~10m | 3.0m | 35.0m | no |
| main Jul-23 | `29967907137` | **49.2m** | ~12m | 3.3m | 36.0m | no |
| main Jul-22 wide | `29961193892` | **48.7m** | ~10m | 3.0m | 36.5m | no |

†Overhead = ci job wall minus regen + floor executor spans (unpack release bins, plan prelude
~99s, artifact verify, yaml/selection prelude, post-floor steps).  
‡Failed run; batch-4 effectful wall still ~19m (pre-#7030 class).

**Current main typical ci job (~48m) ≈ 10m overhead + 3m regen + 35m floor.**

---

## 3. Floor executor batch attribution (current schedule)

Schedule after #7030 (heavy-resolve main-thread routing) + self-host gate enrollment. Representative:
run `29970583893` (ci job 47.7m, success).

| batch | lane | wall | % of floor | dominant mechanism |
|---:|---|---:|---:|---|
| 0 | cheap gates (layering / extdeps / drift) | <1m† | <3% | negligible scans (#7088 moves these before compile-clean on newer main) |
| 1 | `dag_compile_clean_gate_passes` | **0.8m** | 2% | import-closure scoped `.dag` compile (not whole-tree emit) |
| 2 | discovery corpus (hermetic, SelectionApplied) | **12.2m** | **35%** | **entry resolve** — see §4.1 |
| 3 | wet corpora (exec + bin witnesses) | **1.3m** | 4% | small explicit roster; resolve ~11s serial |
| 4 | effectful gates (emit_host + cheap scans) | **7.1m** | 20% | host effects; **recovered** from ~18.5m by #7030 |
| 5 | `source_root_ingest` + self-host chain | **10.9m** | **31%** | heavy whole-tree resolve + ingest host work |
| 6 | `self_host_reads_real_bytes` | **2.9m** | 8% | heavy resolve + filesystem read gate |
| — | regen sub-plan (separate invocation) | **3.0m** | — | regen_verify ~2m + staleness ~1m |

†On `29970583893` cheap gates still co-reside in batch 4; post-#7088 they move to batch 0
(seconds).

### 3.1 Discovery `[measurement]` line (batch 2)

| arm | resolve serial | eval serial | witnesses | skipped |
|---|---:|---:|---:|---:|
| PRE `29763408563` | **99s** | 15s | 2087 | 1652 |
| POST `29819122813` | **403s** | 15s | 2128 | 1755 |
| main `29970583893` | **620s** | 15s | 2204 | 1739 |

**Eval is not the story** on affected-set PRs (SelectionApplied skips ~79% of roster). **Resolve
serial-sum grew 6.3×** PRE→main and tracks batch-2 wall (~12 min). Per-witness amortized resolve
rose ~48ms → ~282ms (2204 witnesses), matching the #6848 bare-reference fixpoint class priced in
[floor-time-namespace-walk-regression-diagnosis.md](floor-time-namespace-walk-regression-diagnosis.md).

### 3.2 Governor / cap saturation

On **16 GiB `memory.high` capped hosts**, post-#6848 floors can run `forced_serial=1` with
`hard_backoffs=1` while RSS sits at 93–100% of budget (§1.4 of namespace-walk diagnosis).
This is an **additive throttle** on top of walk work — same resolve class, slower wall. Uncapped
hosts (MemAvailable budget) complete the same schedule without backoffs but **do not** erase the
resolve-serial inflation.

---

## 4. Mechanism ledger (cause → minutes → owner)

| # | mechanism | Δ vs PRE (~18m ci) | Δ vs POST-#6848 (~42m ci) | status | owner lane |
|---|---|---:|---:|---|---|
| A | #6848 bare-reference fixpoint (`extend_sources_to_both_closure_fixpoint`) — once per entry | **+5 min** floor batch-2 | ~0 (still dominant) | **open** | namespace-resolution §PR-5b |
| B | #6848 qualified-fill on reconcile miss | **+13 min** floor batch-4 (pre-#7030) | **−11 min** (#7030 heavy-resolve routing) | **partial** | #7030 landed; reconcile-miss path still open |
| C | `thread_local` cold `MultiEntryIndex` per spawned gate (#6999 didn't fix) | (folded into B) | **−11 min** | **landed #7030** | proud-bear-438 |
| D | Self-host gate enrollment batches 5–6 (`source_root_ingest`, `reads_real_bytes`) | **+14 min** | +14 min (new scope) | **by design** | self-host / module-identity |
| E | Cap-saturation throttle on 16 GiB runners | unpriced additive | widens band toward timeouts | **open** | v1-run-stability / governor envelope |
| F | `UnlistedImportUse` advisory generation (~5k rows/run) | typecheck overhead | same | **open** | namespace advisory suppression |
| G | ci overhead (bin unpack + plan prelude ~99s) | +3m | stable ~10m | baseline | CI substrate |

**Reconciliation to band:** PRE ci 17.9m → main ci 47.7m ≈ **+30 min** ≈ A (+5) + B net (+2 after
#7030) + D (+14) + G (+3) + E (variable). The POST-#6848 → main delta is mostly **D** (new gates)
with **B partially reversed**.

---

## 5. Historical contrast (effectful-gate recovery receipt)

Batch-4 effectful wall (first gate `emit_host_gate_passes` → last batch-4 pass):

| arm | batch-4 wall | notes |
|---|---:|---|
| PRE `29763408563` | **5.7m** | parallel groups, no self-host reads-bytes |
| POST `29819122813` | **18.5m** | cold per-thread index (#6848 + spawn routing) |
| POST `29855080611` | **18.8m** | #6998/#6999: ~0% recovery (by execution) |
| POST `29880571548` | **10.0m** | #7030: heavy-resolve flip — partial |
| main `29970583893` | **7.1m** | #7030 + schedule churn; near PRE+host-effects |

---

## 6. Reproduction

```bash
# ci job duration
gh run view <run_id> --json jobs | jq '.jobs[] | select(.name=="ci") | {startedAt,completedAt,conclusion}'

# floor batch walls + discovery measurement
gh run view <run_id> --log > /tmp/floor.log
rg 'claim_executor: batch|PASS \[batch|\[measurement\] discovery corpus:|\[governor\] receipt:' /tmp/floor.log

# compare arms
for r in 29763408563 29819122813 29970583893; do
  echo "=== $r ==="
  gh run view $r --json jobs | jq -r '.jobs[]|select(.name=="ci")|"ci job: \(.startedAt) -> \(.completedAt)"'
  rg 'discovery corpus: [0-9]+ witness' <(gh run view $r --log 2>/dev/null) | head -1
done
```

Python segmenter used for §2–3 tables: partition log on
`claim_executor" --source-root` + `--plan-function gunbc_ci_regen_floor_batches|gunbc_ci_floor_batches`.

---

## 7. Next scoped dispatch (not this lane)

Ordered by displaced ci job minutes on current main:

1. **Discovery resolve fixpoint (A)** — attack once-per-entry bare-reference walk
   (namespace-resolution-design §PR-5b residual). Target: **−5 to −8 min** floor batch-2.
2. **Self-host gate cost (D)** — per-gate measurement with `GUNBC_FLOOR_GANTT=1`; justify or
   narrow batches 5–6 if host work is redundant with batch-4 emit_host. Target: clarity first;
   reduction only if duplicated resolve.
3. **Cap saturation (E)** — headroom vs walk RSS on 16 GiB slots; pairs with #6848 residual.
4. **Advisory row suppression (F)** — if still hot after A.

---

## 8. Provenance

- Log-diff + ci job JSON: vivid-fox-471, by execution on fleet runs listed in §2–3 (2026-07-23).
- Parent mechanism docs: [floor-time-namespace-walk-regression-diagnosis.md](floor-time-namespace-walk-regression-diagnosis.md),
  [ci-floor-fractal-gantt.md](ci-floor-fractal-gantt.md), PR #7030 receipt.
- Related open threads: DESIGN.md floor shared-computation memoization M2; namespace-resolution §PR-5b.
