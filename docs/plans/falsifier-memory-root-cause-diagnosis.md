# Falsifier memory root-cause diagnosis

**Status:** diagnosis complete + P0 fix in PR #7813 (2026-08-04, session bold-eagle-790).
Width check and bisect executed per still-bat-561 mandate (2026-08-04). Authority:
`docs/plans/v1-run-stability-throughline.md`, `docs/plans/m2-floor-retention-measurement-receipt.md`,
`docs/plans/compile-clean-whole-tree-time-diagnosis.md`.

**One-line verdict:** Post-M2 (#7129), the falsifier no longer OOM-kills on fleet slots —
it **completes in ~2 h** but pins cgroup peak at the **15 GiB `memory.high` line**. The
resident footprint is **not a leak**; it is **same-process accumulation** from (a) a
**~9.5 GiB pre-floor baseline** (release build + selection-control step, not returned to
the kernel before the floor step starts) plus (b) a **~11.8 GiB discovery-corpus
steady-state plateau** during batch 1 whose dominant terms are **whole-pool heads**
(non-evictable by design) and **~779 concurrently retained typed modules** under schedule
retention. M2 schedule eviction **is working** (`schedule_evictions=2264`,
`retention_unknown=0` in batch 1). **Width is 1 throughout** (`forced_serial=0`) — this is
**not** the plural-worker outer-ring class. **Current falsifier redness is witness failure
(application-level), not memory or cgroup OOM** — wise-ram-22 owns witness content.

---

## 1. Evidence runs (fleet, 16 GiB slot / 15 GiB `memory.high`)

**Runner attribution rule:** there is no healthy reference host — read `actions-runner@srvN-NN`
from cgroup paths before comparing runs.

| Run | Host | SHA | Outcome | `floor_peak_pre` | `floor_peak_post` | Width | Governor | Primary failure |
|---|---|---|---|---:|---:|---|---|---|
| [30793942277](https://github.com/gunb-ai/gunbc/actions/runs/30793942277) | **srv3-01** | `19cc776d` | **success** | 9,748,836,352 | (success; post step not in retained log) | **1** | `forced_serial=0` | — |
| [30809682025](https://github.com/gunb-ai/gunbc/actions/runs/30809682025) | **srv4-05** | `19cc776d` | failure | 9,750,724,608 | 16,106,680,320 | **1** | `forced_serial=0` | witness redness (same SHA as last green) |
| [30824310833](https://github.com/gunb-ai/gunbc/actions/runs/30824310833) | srv2-05 | `76ef121ec4` | failure | 10,192,011,264 | (same class) | **1** | `forced_serial=0` | witness redness |
| [30935833427](https://github.com/gunb-ai/gunbc/actions/runs/30935833427) | srv2-03 | `64aed6007e` | failure | 10,219,753,472 | 16,106,897,408 | **1** | `forced_serial=0` | witness redness |

Contrast with pre-M2 baseline (`v1-run-stability-throughline.md` §1): step-cap TIME death,
`forced_serial=1`, `hard_backoffs=474`, cgroup peak censored at `memory.high` **with swap
thrash** — a different failure mode.

---

## 2. Memory timeline (run 30935833427, authoritative)

### 2a. Pre-floor baseline (~9.5 GiB before floor reset)

```
[calibration] floor_peak_pre reset=unsupported pre_peak=10219753472 scope=span cgroup=...actions-runner@srv2-03.service
```

The cgroup already holds **~9.5 GiB** from **Build release bins** (~3 min) and **Affected-set
selection control** (~10 min) before `claim_executor` starts. `memory.peak` reset is
unsupported (prior peak non-zero), so the floor measurement is **job-scoped span**, not a
clean floor-only reset.

### 2b. Batch 1 discovery corpus — steady-state plateau

Heartbeat samples (`memory.current`, batch 1, entries 11→912 of 918):

| Phase | Entry range | RSS | Notes |
|---|---:|---:|---|
| ramp | 11→199 | 7.2→11.2 GiB | typed_cache growing |
| **plateau** | 320→912 | **11.8 GiB** (±0.1) | M2 eviction ↔ new entries balance |
| batch boundary spike | batch 1→2 | **14.2 GiB** peak sample | transient overlap at schedule transition |

End-of-batch receipt:

```
[floor-drain] receipt: groups=918 typed_cache_peak=779 parse_cache_peak=805
  resolved_memo_peak=11 intern_table_peak=81993 evictions=0
  schedule_releases=2264 schedule_evictions=2264 retention_unknown=0
  peak_rss=9471655936
```

**Reading:** live RSS plateaus at **~11.8 GiB** while VmHWM (`peak_rss`) reports
**~8.8 GiB** — the heartbeat's `memory.current` is the honest live resident signal during
the sweep; VmHWM undercounts relative to cgroup accounting on this host.

### 2c. Later batches — eviction works, VmHWM is cumulative

| Batch | `typed_cache_peak` | `retention_unknown` | `peak_rss` (VmHWM) |
|---:|---:|---:|---:|
| 1 | 779 | 0 | 9.47 GiB |
| 5 | 94 | **25** | 10.28 GiB |
| 6 | 190 | 0 | 10.28 GiB |
| 8 (`source_root_ingest_gate`) | — | — | live 9.9→12.3 GiB over 16 min |

`typed_cache_peak` drops between batches (schedule retention evicting). VmHWM does not
decrease — it is a process-lifetime high-water mark.

### 2d. Cgroup pin at `memory.high`

```
[floor-memory] regime: memory.high=16106127360; memory.max=17179869184
[calibration] floor_peak_post=16106897408 floor_outcome=failure
```

**Arithmetic:** pre-floor **9.5 GiB** + floor transient spikes to **14.2 GiB** live → cgroup
peak **15.0 GiB** (within 7 MiB of `memory.high`). Swap stays **≤0.5 GiB**; no governor
distress. The pin is **reclaim-pressure proximity**, not OOM-kill.

---

## 3. Root-cause decomposition

| # | Mechanism | Evictable? | Measured share | Authority |
|---|---|---|---|---|
| **A** | **Pre-floor cgroup carry-in** — release build + selection-control binaries/indexes stay resident; `floor_peak_pre` cannot reset | no (separate steps, same cgroup) | **~9.5 GiB** at floor entry | run 30935833427 `floor_peak_pre` line |
| **B** | **Whole-pool heads** — bare/qualified census + heads-only parse snapshot required for reconcile (#6848/#6956) | **no** (by construction) | part of plateau; heads never in schedule refcount | `gunbc.executor_schedule_retention` `schedule_retention_heads_stay_resident` |
| **C** | **Schedule-retained typed modules** — ~779 modules concurrently resident while 918-entry batch 1 schedule still has downstream consumers | yes, and **does** evict (`schedule_evictions=2264`) | plateau **~11.8 GiB** live RSS | batch 1 `floor-drain` receipt |
| **D** | **`intern_table`** — 81,993 entries, process-lifetime | no current eviction path | counted in receipt; not separately attributed | `intern_table_peak=81993` |
| **E** | **`retention_unknown` modules** — provenance-gap retain (fail-open) | counted, not evicted | **25 modules**, batch 5 only; +~0.8 GiB VmHWM step | `schedule_retention_retain_on_unknown` |
| **F** | **Width=1 inline drain** — governor never admits plural workers | N/A (scheduling, not retention) | width>1 prize unmeasured | `m2-floor-retention-measurement-receipt.md` §4 |

**What this is NOT (retracted hypotheses):**

- **Not an M2 schedule-retention failure** — evictions fire, underflow refusals absent,
  `retention_unknown=0` across the dominant batch-1 path.
- **Not the pre-M1 inductive-field duplication class** — M1b/M1b-2 closed that axis (#6528).
- **Not current OOM/step-cap death** — runs complete; exit 1 is **witness redness**.

---

## 4. Relationship to historical "memory-dead falsifier"

`v1-run-stability-throughline.md` §1 M1 dial receipt: pre-M1 falsifier was **step-cap TIME
death** at swap speed with `forced_serial=1`. M1+M2 moved batch-1 live RSS from
**≥15 GiB censored / swap-thrash** to **~11.8 GiB plateau / swap≈0**. The falsifier is
**memory-alive** but **envelope-tight**: 9.5 GiB carry-in + 5.5 GiB floor headroom = pin at
`memory.high`.

---

## 5. Current failure mode (orthogonal to memory)

All four recent runs fail on **witness redness**, not memory:

1. **Batch 1:** `fn_wf_no_trigger_negative_control`
   (`dag/test/claim/generic_item_clone_bound_witness_test.dag`) — intermittent 1-of-6630
2. **Batch 6:** four `direct_rust_door` / `v2_emitter_direct_rust_door_*` long-lane witnesses
3. **Dark control:** `falsifier_red_control_holds` divergence logged (expected-red control)

Memory investigation does **not** unblock falsifier green — witness fixes do.

---

## 6. Fix lanes (ranked by displaced cost)

| Priority | Lane | Trigger | Expected effect | Status |
|---|---|---|---|---|
| **P0** | **Process split** — falsifier-build (compile + selection control + pack/upload) then falsifier floor job (download + verify + peak-bracketed floor) | falsifier-memory diagnosis 2026-08-04 | Recover ~9.5 GiB cgroup headroom — build carry-in no longer shares cgroup with floor | **LANDED (this PR)** |
| **P1** | **Closure-scoped `pool_parse`** (namespace §PR-5b / #6848 follow-on) | compile-clean diagnosis §5 | Shrink heads + reconcile baseline (~3.1 GiB compile-clean step; partial floor benefit) |
| **P2** | **Close `retention_unknown` provenance gaps** (25 modules, batch 5) | counted debt | Minor; fail-open retain is honest but bounded |
| **P3** | **Outer-ring `SpacePacked` eviction** | M2 width-1 banked, width>1 open | Host-budget eviction when schedule retention insufficient |
| **P4** | **Interface-summary retention** (S2a Inc-B) | if P0–P3 insufficient | Durable shrink of per-module typed payload |

**Not recommended:** raising `memory.high` (§5 guardrail in throughline — widening arm).

---

## 7. Dissolve trigger

This receipt dissolves when a fleet falsifier run records `floor_peak_post` **< 12 GiB**
with `floor_outcome=success` **and** batch 1 `peak_rss` receipt **< 8 GiB**, on the
standard 16 GiB slot without process-split scaffolding.

---

## 8. Width check (Lead One — executed 2026-08-04)

**Question:** Is the ~15 GiB pin the v1 run-stability lane's unfinished plural-worker outer
ring (width > 1)?

**Answer: NO.** Every examined falsifier cold-floor run logs:

```
claim_executor: batch N — … governor target_width=1
run_discovery_corpus: … (governor target_width=1)
run_discovery_corpus: width=1 inline drain — reusing process_shared_index
run_discovery_corpus: cross_worker_store withheld (governor target_width=1)
governor receipt — … forced_serial=0 hard_backoffs=0 …
```

This is the **same width=1 machinery** measured in the v1 run-stability lane (9.24 GiB
executor RSS / 10.7 GiB cgroup peak at width 1, #7129). The ~15 GiB pin is **not** a novel
width>1 defect — it is width=1 same-process accumulation (§3 mechanisms A–C) plus
pre-floor carry-in. Remedies for the plural-worker outer ring (M2 width>1 banked work) do
**not** apply here.

---

## 9. Bisect (Lead Two — executed 2026-08-04)

**Anchor:** last CompleteGreen = `19cc776d`, run [30793942277](https://github.com/gunb-ai/gunbc/actions/runs/30793942277), 2026-08-03T07:31:05Z, **srv3-01**.

**First failure after anchor:**

| Run | When | SHA | Host | vs anchor |
|---|---|---|---|---|
| [30809682025](https://github.com/gunb-ai/gunbc/actions/runs/30809682025) | +4h | **`19cc776d` (same)** | srv4-05 | **not a commit regression** |

The first post-green failure is on the **same commit** on a **different host**. Exit 1 is
**witness redness** (application-level fail-closed arms), not cgroup OOM or oomd kill:

- `falsifier_red_control_holds` divergence (expected-red control)
- `witness_committed_is_fixed_point` — eval budget exceeded (5 s fast-lane rule)
- batch-2 self-host behavioral witness timeouts (`self_host_*_behavioral_receipt_holds`)

**Reported upstream (wise-ram-22):** these are witness-content failures, not infrastructure
memory kills. The ~15 GiB cgroup pin co-occurs but is **not** the exit cause (`floor_outcome=failure` from witness batch failure, `forced_serial=0`).

**Commit-bound interval** (`19cc776d` → first failing *new* HEAD `76ef121ec4`, run
30824310833): subsequent failures remain witness-content class; no measured memory-regression
commit isolated in this interval. Corpus grew modestly (6507→6550→6630 discovery rows) but
pre-floor baseline and width=1 plateau class are stable across hosts.

**Bisect conclusion for F2 (infrastructure):** the memory envelope hypothesis (§3) **holds**
at width=1; redness is **orthogonal** (wise-ram-22). P0 process-split remains the
infrastructure fix for mechanism A (pre-floor carry-in).
