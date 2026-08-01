# Receipt — M2-aware floor-retention measurement (Dispatch J)

**Status:** measurement receipt, timestamped 2026-08-01. **DESIGN.md + `docs/plans/v1-run-stability-throughline.md` remain authority** — this is a dated receipt, not a fact ledger.

**Lane:** v1 run-stability · **Subject:** schedule-derived retention (#7129) on the real `claim_executor` floor path · **Deliverable:** measurement only (no retention mechanism changes).

**Main SHA measured:** `45e7a8caa457da236b1e5e5cef8f3672a555672b` (`Revalidate generated-artifact heal heads (#7544)`).

**Binary snapshot:** `target/release/claim_executor-m2-45e7a8c` (copied before probes; `cargo build` during a long run would invalidate resolve-cache content-addressing).

**Host regime (honesty):** dashboard session container, **uncapped** cgroup `memory.max` ≈ 31.27 GiB (`33578549248` B), `memory.high` absent. This is **not** the fleet 16 GiB slot with a 15 GiB `memory.high` throttle line. Comparisons to the M2 landed width-1 fleet receipt (9.24 GB RSS / 10.7 GB cgroup under 15 GiB) are **regime-relative** — width-1 peak observed here is **not** the throughline’s width > 1 exit metric and does not byte-match fleet cgroup physics.

**No shipped orchestration script** (operator precedent #7533 / valiant-lark: checked-in `run_matrix.sh` deleted — complete invocations live here as documentation, not as executable wrappers).

---

## 0 — Reproduction (verbatim invocations)

All commands run from the worktree root on the measured SHA. **Snapshot the binary before a long probe** — `cargo build` changes the executable bytes and invalidates resolve-cache content-addressing.

**1. Record SHA and build `claim_executor` (local; disable `sccache` if the host rustc ICEs under wrapper):**

```bash
git rev-parse HEAD
CTRL_BUILD_MODE=local RUSTC_WRAPPER= CARGO_BUILD_JOBS=4 \
  ctrl-build --local -- cargo build -p v1-compiler --bin claim_executor --release
```

**2. Snapshot binary (do not rebuild while the probe runs):**

```bash
cp -f target/release/claim_executor target/release/claim_executor-m2-45e7a8c
```

**3. Whole-corpus falsifier shape (primary probe — batch 1 `SelectionPredictOnly`):**

```bash
export GUNBC_FLOOR_DRAIN_RETENTION=1
export GITHUB_EVENT_NAME=schedule
LOG=docs/probes/m2_floor_retention_20260801T082411Z/falsifier_whole_corpus.log
mkdir -p "$(dirname "$LOG")"
target/release/claim_executor-m2-45e7a8c \
  --source-root dag \
  --source-root src/v2 \
  --plan-entry src/v2/workflow/ci_floor_plan.dag \
  --plan-function gunbc_falsifier_plan \
  2>&1 | tee "$LOG"
```

Stop after batch 1 completes (`✓ PASS [batch 1] discovery-corpus`) if later falsifier batches (wet self-host, etc.) are out of scope. This measurement stopped there.

**4. Main-push `SelectionApplied` shape (optional — not completed on this host):**

```bash
export GUNBC_FLOOR_DRAIN_RETENTION=1
export GITHUB_EVENT_NAME=push
LOG=docs/probes/m2_floor_retention_20260801T091619Z/main_push_floor.log
mkdir -p "$(dirname "$LOG")"
target/release/claim_executor-m2-45e7a8c \
  --source-root dag \
  --source-root src/v2 \
  --plan-entry src/v2/workflow/ci_floor_plan.dag \
  --plan-function gunbc_ci_floor_plan \
  2>&1 | tee "$LOG"
```

**5. Extract receipt lines from a completed log (read-only; not a rerun):**

```bash
grep -E '\[floor-drain\]|\[floor-memory\]|governor receipt|schedule-retention|FLOOR-BATCH|◷|cgroup peak' \
  docs/probes/m2_floor_retention_20260801T082411Z/falsifier_whole_corpus.log
```

**6. Cgroup regime readback (optional host context; not emitted by `claim_executor`):**

```bash
cat /sys/fs/cgroup/memory.max
cat /sys/fs/cgroup/memory.high 2>/dev/null || echo 'memory.high absent'
cat /sys/fs/cgroup/memory.current
cat /sys/fs/cgroup/memory.swap.current
# memory.events high only fires when memory.high is set — on uncapped hosts expect no throttle events:
grep -E '^high |^oom ' /sys/fs/cgroup/memory.events 2>/dev/null || true
```

Default `claim_executor` behavior is unchanged — no env disables M2 eviction unless `GUNBC_SCHEDULE_RETENTION_EVICT=0` (retain-all measurement pole; not used here).

---

## 1 — What was measured (and what was not)

| Probe | Plan function | Floor path? | Selection | Entry scale | Log |
|---|---|---|---|---|---|
| **falsifier_whole_corpus** | `gunbc_falsifier_plan` | yes (`claim_executor` → `ci_floor_plan.dag`) | `SelectionPredictOnly` (batch 1 — whole-corpus cold control) | **842 entries**, **5818 witness rows** | `docs/probes/m2_floor_retention_20260801T082411Z/falsifier_whole_corpus.log` |
| main_push_floor | `gunbc_ci_floor_plan` | yes | `SelectionApplied` (corpus batch when reached) | **not completed** — stopped during plan prelude (compile-clean install) | `docs/probes/m2_floor_retention_20260801T091619Z/main_push_floor.log` (partial) |

**Contrast explicitly excluded (valiant-lark / #7533 slice 2):** serial one-shared-index path without governor adaptive pool schedule arming — **not** the real floor path. Its 32 GiB cgroup OOM is an **instrument-path artifact**, not a floor fact.

**Affected-set ~600 (main-push scale):** not executed end-to-end on this host within the measurement window. The falsifier whole-corpus probe (**842 entries**) is a **strict superset** of the ~602-entry main-push corpus batch cited in `docs/plans/progress-observation-design.md` (batch 3 witness discovery). Peak memory on the superset bounds the subset; a dedicated ~600-entry `SelectionApplied` receipt remains optional follow-up if the operator wants exact parity.

---

## 2 — M2 mechanism receipts (batch 1, whole corpus)

**Arming (unconditional):**

```
[floor-drain] schedule-retention ARMED: entries=842 modules_refcounted=2094 evict_enabled=true
```

**End-of-batch aggregate (`floor-drain` receipt):**

| Field | Value |
|---|---|
| entry-groups | 842 |
| `schedule_evictions` | **2094** |
| `retention_unknown` | **0** |
| `typed_cache_peak` | 722 |
| `parse_cache_peak` | 719 |
| `resolved_memo_peak` | 11 |
| `typed_cache_evictions` (host cap backstop) | 0 |
| `peak_rss` (VmHWM sample) | **6730485760 B (6.27 GiB)** |
| `cap_entries` | 4000 |

**Per-entry eviction lines:** present throughout (example tail: `schedule_evictions=2094`, `graph_evictions` matched resolved-graph pin drops). **No** `SCHEDULE-RETENTION REFUSAL` / underflow / `RefcountUnderflow` lines in the log.

**Discovery summary:**

```
[measurement] discovery corpus: 5818 witness(es) (0 skipped, 905 deferred),
  resolve 1512066.148ms, evalu 84296.470ms, roster-closure 524 nodes (max shard)
```

**Batch outcome:** `✓ PASS [batch 1] discovery-corpus[2 root(s)+0 explicit, adaptive width] (5818 witnesses)`.

---

## 3 — Cgroup / memory / governor (batch 1)

**Cgroup regime disclosure:**

```
[floor-memory] regime: memory.high=none; memory.max=33578549248 (/sys/fs/cgroup/)
◷ governor adaptive width — budget=33578549248 source=cgroup memory.max max_width=128
```

**Floor-memory heartbeat (minute samples, batch 1 witness discovery):**

| Phase | `memory.current` (GiB) | swap (GiB) | pressure (avg10 bp) |
|---|---:|---:|---:|
| early (entry ~36/842) | 4.6–5.4 | 0.3–0.4 | 0 |
| mid (entry ~177–330) | 4.6–5.0 | 2.3–3.5 | 0–0.3 |
| late (entry ~600–801) | 5.8–6.0 | 3.5–3.6 | 0 |

Peak VmHWM in `floor-drain` group lines stabilized at **6730485760 B** once typed cache reached steady state (~entry 321+).

**`memory.events` high:** not observed on this uncapped host (`memory.high=none` — governor documents that events do not fire without a throttle line). **Not quotable** for fleet throttle-line behavior; fleet comparison uses peak RSS vs 15 GiB line.

**Governor width / distress:**

| Signal | Value |
|---|---|
| `target_width` (entire batch 1) | **1** |
| `max_width_reached` | **1** (inline drain latch — see §4) |
| `forced_serial` | **0** (no receipt line; no distress emissions) |
| `hard_backoffs` | **0** |
| `budget_exceeded` | **0** |

**Completion-within-step-budget:** batch 1 completed **PASS** without `FLOOR-BATCH-OVER-BUDGET` or step-cap kill. Wall clock ≈ **33 minutes** for batch 1 (resolve-dominated: **1512 s** measured).

---

## 4 — Width > 1 (honest accounting)

The throughline exit metric names **completion within step budget at governor width > 1**. On this architecture, the adaptive discovery pool **samples `target_width` once** at pool entry; when `target_width ≤ 1` it takes the **inline drain** branch and **does not** reach the governor AIMD controller that would admit plural workers (`cli_run.rs` documents this as an intentional latch until per-worker index sharing removes the Amdahl setup cost).

**Measured:** `max_width_reached=1`, `cross_worker_store withheld` — M2 schedule retention still **armed and evicting** on the width=1 inline path (the production serial-regime that fleet floors use when width cannot profitably grow).

**Do not misread:** width=1 here is **not** the pre-M2 “forced_serial=1 / 474 backoffs / swap-speed death” regime from §1 baseline — that was retention-throttled serial at the **memory.high** line. Here, width=1 is the **governor-correct** inline path with **M2 eviction active** and **~6.3 GiB peak** on 842 entries.

---

## 5 — Handback decision

**Scope honesty:** This receipt is an **incomplete width-1 measurement**. The throughline exit metric (§0 / §1) is **completion within the step budget at governor width > 1** — not width-1 peak RSS alone. Width > 1 was **not measured** here; main-push `SelectionApplied` was **not completed** (§6). **Do not record handback (a)** from this slice.

### (a) M2 path bounded comfortably at production scale → **NOT RECORDED**

Partial width-1 evidence (does not satisfy the authoritative exit metric):

1. **Whole-corpus falsifier shape** (842 entries / 5818 witnesses): **PASS**, peak RSS **6.27 GiB**, `schedule_evictions=2094`, `retention_unknown=0`, no retention refusals — **width 1 only**.
2. **Fleet envelope (regime-relative):** peak **< 16 GiB** with headroom vs the 15 GiB throttle line on an **uncapped** host — contrasts with pre-M2 censored ≥15 GiB peaks, but is **not** a width > 1 fleet receipt.
3. **M2 landed width-1 receipt** (throughline BANKED block): 9.24 GB / 10.7 GB — this measurement is **consistent** at the same width-1 regime.

**Width > 1 fleet bar remains NEXT** (throughline BANKED block): per-worker index sharing / outer-ring scheduling must be measured before handback (a) or the §0 exit metric can be claimed closed.

| Follow-up | Status after this receipt |
|---|---|
| width > 1 fleet receipt | **Still required** — authoritative success metric; not measured here. |
| main-push `SelectionApplied` (~602 entries) | **Not measured** — probe stopped in prelude (§6). |
| outer-ring `SpacePacked` eviction | **Open** — not exercised; schedule eviction active at width 1 only. |
| resident-accounting split | **Open** — no width > 1 receipt to attach accounting to. |

### (b) Capacity-bound → which follow-up — **NOT triggered at width 1**

No measured boundary on the **width-1** M2 floor path executed here. Does not adjudicate the width > 1 fleet bar.

---

## 6 — Retractions / limits (slice-2 discipline)

| Claim | Status |
|---|---|
| handback (a) “bounded comfortably” / follow-ups deprioritized | **RETRACTED** — incomplete width-1 slice; width > 1 bar remains authoritative (§5) |
| valiant-lark 32 GiB OOM as floor fact | **RETRACTED** (instrument path; stated in brief) |
| main-push ~602-entry `SelectionApplied` peak | **NOT MEASURED** (probe stopped in prelude; superset argument only) |
| `memory.events` high on fleet throttle line | **NOT MEASURED** (uncapped host) |
| `governor receipt` end-of-run line | **NOT CAPTURED** (falsifier plan stopped after batch 1; partial distress signals already §3) |

---

## 7 — Dissolve trigger

This receipt dissolves when either (1) a fleet-slot rerun lands post-`Rc→Arc` width un-latch with width > 1 receipts, or (2) the outer-ring `SpacePacked` lane produces a competing retention receipt that supersedes the M2-only story.
