# Receipt — P3 width-2 trial after bounded retention (16 GiB)

**Status:** measurement receipt (in progress). **Authority:** `docs/plans/v1-run-stability-throughline.md` §0 exit metric — completion within step budget at governor width > 1 under the fleet 16 GiB slot envelope.

**Lane:** v1 run-stability · **Subject:** M2 schedule retention (#7129) + controlled width-2 trial (`GUNBC_FLOOR_WIDTH_TRIAL=2`) · **Deliverable:** measurement only on the real `claim_executor` floor path.

**Prerequisite:** width-1 bounded retention at the fleet envelope (`GUNBC_MEMORY_BUDGET_BYTES=16106127360`, the runner-slot `memory.high` line) must PASS before the width-2 trial arm is interpreted.

**Host regime:** document cgroup `memory.max` / `memory.high` at run time. Fleet comparison uses peak RSS vs the 15 GiB throttle line and completion-within-batch-budget.

**No shipped orchestration script** (operator precedent #7533 / m2-floor-retention-measurement-receipt.md).

---

## 0 — Reproduction (verbatim invocations)

All commands from the worktree root. Snapshot `claim_executor` before a long probe.

**1. Build and snapshot:**

```bash
git rev-parse HEAD
ctrl-build -- cargo build -p v1-compiler --bin claim_executor --release
cp -f target/release/claim_executor target/release/claim_executor-p3-width2
```

**2. Phase A — bounded retention at fleet envelope (width 1, no trial env):**

```bash
export GUNBC_MEMORY_BUDGET_BYTES=16106127360
export GUNBC_FLOOR_DRAIN_RETENTION=1
export GITHUB_EVENT_NAME=schedule
LOG=docs/probes/p3_width2_$(date -u +%Y%m%dT%H%M%SZ)/phase_a_bounded_retention_16gib.log
mkdir -p "$(dirname "$LOG")"
target/release/claim_executor-p3-width2 \
  --source-root dag \
  --source-root src/v2 \
  --plan-entry src/v2/workflow/ci_floor_plan.dag \
  --plan-function gunbc_falsifier_plan \
  2>&1 | tee "$LOG"
```

Stop after batch 1 completes (`✓ PASS [batch 1] discovery-corpus`) unless the operator extends scope.

**3. Phase B — width-2 trial (only after Phase A PASS):**

```bash
export GUNBC_MEMORY_BUDGET_BYTES=16106127360
export GUNBC_FLOOR_DRAIN_RETENTION=1
export GUNBC_FLOOR_WIDTH_TRIAL=2
export GITHUB_EVENT_NAME=schedule
LOG=docs/probes/p3_width2_$(date -u +%Y%m%dT%H%M%SZ)/phase_b_width2_trial_16gib.log
mkdir -p "$(dirname "$LOG")"
target/release/claim_executor-p3-width2 \
  --source-root dag \
  --source-root src/v2 \
  --plan-entry src/v2/workflow/ci_floor_plan.dag \
  --plan-function gunbc_falsifier_plan \
  2>&1 | tee "$LOG"
```

**4. Extract receipt lines:**

```bash
grep -E '\[floor-drain\]|\[floor-memory\]|governor receipt|schedule-retention|FLOOR-BATCH|measurement width trial|max_width_reached|cross_worker_store' "$LOG"
```

---

## 1 — Phase A results (bounded retention, width 1)

| Field | Value |
|---|---|
| Main SHA | _pending_ |
| Budget armed | 16106127360 B (15 GiB `memory.high` via env) |
| Batch 1 outcome | _pending_ |
| `schedule_evictions` | _pending_ |
| `retention_unknown` | _pending_ |
| Peak RSS (VmHWM) | _pending_ |
| `max_width_reached` | _pending_ (expect 1) |
| Governor distress | _pending_ |

**Phase A gate:** PASS with peak < 16 GiB and zero retention refusals → Phase B authorized.

---

## 2 — Phase B results (width-2 trial)

| Field | Value |
|---|---|
| `GUNBC_FLOOR_WIDTH_TRIAL` | 2 |
| Trial arm logged | _pending_ |
| `max_width_reached` | _pending_ (expect ≥ 2) |
| `cross_worker_store` | _pending_ (expect armed, not withheld) |
| Batch 1 outcome | _pending_ |
| Peak RSS (VmHWM) | _pending_ |
| Wall clock (batch 1) | _pending_ |
| Governor distress | _pending_ |

---

## 3 — Handback

| Claim | Status |
|---|---|
| Bounded retention at fleet 16 GiB envelope (width 1) | _pending_ |
| Width > 1 fleet receipt (authoritative exit metric) | _pending_ — requires Phase B PASS at width ≥ 2 |
| Production un-latch | **NOT claimed** — `GUNBC_FLOOR_WIDTH_TRIAL` is measurement-only; dissolve-on remains shared index (Rc→Arc) per `cli_run.rs` |

---

## 4 — Dissolve trigger

This receipt dissolves when the throughline §0 exit metric is banked (two consecutive whole-corpus main runs + one falsifier cold run at width > 1 within step budget) or superseded by a fleet-slot rerun with production index sharing.
