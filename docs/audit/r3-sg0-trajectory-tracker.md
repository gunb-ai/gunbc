# R3 SG-0 Census Trajectory Tracker

**Author**: deep-wolf-155 (PM)
**Authority scope**: PM-tier daily-cadence tracking artifact. Updated per-cycle (or per Director cycle absorption) until R3 close.
**Parent**: [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](r3-pb0-velocity-walk-2026-05-09.md) — load-bearing finding that motivated daily-tracking discipline.

---

## §0. What this tracks

Daily/per-cycle snapshot of `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_TEST` counts in `src/v3/compiler/tests/integration/sg0_census_test.rs`, against the R3-close target of **0 + 0** for §1.8 gates **#8** + **#84** (Pure-Bootstrap-Zero closure).

## §1. Schema (per-snapshot row)

| Field | Type | Description |
|---|---|---|
| `date` | `YYYY-MM-DD` | Snapshot date (UTC) |
| `sha` | `git short-sha` | `origin/main` HEAD at snapshot time |
| `non_test` | `int` | `EXPECTED_HAND_AUTHORED_NON_TEST` count (gate #8 target=0) |
| `test` | `int` | `EXPECTED_HAND_AUTHORED_TEST` count (gate #84 target=0) |
| `total` | `int` | `non_test + test` |
| `delta_vs_prev` | `int` | Net change vs prior snapshot |
| `bulk_events_landed` | `string[]` | Named bulk-dissolution events landed since prior snapshot (e.g., "Cluster M Phase 1 #85") |
| `notes` | `string` | One-line context (drift, stable, anomaly) |

## §2. Snapshot generation procedure

```bash
# At workspace root with origin/main fetched:
git fetch origin main
sha=$(git rev-parse --short origin/main)
non_test=$(git show origin/main:src/v3/compiler/tests/integration/sg0_census_test.rs | \
  awk '/^const EXPECTED_HAND_AUTHORED_NON_TEST/,/^\];/' | grep -E '"src/v3/' | wc -l)
test=$(git show origin/main:src/v3/compiler/tests/integration/sg0_census_test.rs | \
  awk '/^const EXPECTED_HAND_AUTHORED_TEST/,/^\];/' | grep -E '"src/v3/' | wc -l)
total=$((non_test + test))
echo "$(date -u +%Y-%m-%d) | $sha | non_test=$non_test test=$test total=$total"
```

## §3. Snapshot history

| Date | sha | non_test | test | total | delta | bulk events | notes |
|---|---|---|---|---|---|---|---|
| 2026-04-30 | (HEAD~500) | 38 | 81 | 119 | — | — | retroactive baseline |
| 2026-05-02 | (HEAD~300) | 40 | 87 | 127 | +8 | — | retroactive |
| 2026-05-06 | (HEAD~150) | 46 | 89 | 135 | +8 | — | retroactive |
| 2026-05-07 | (HEAD~50) | 47 | 95 | 142 | +7 | — | retroactive |
| **2026-05-09** | **c25b2d8df** | **48** | **101** | **149** | **+7 (9-day total: +30)** | (none — Cluster M cold) | velocity-walk audit landed PR #2358; remediation program in flight |

## §4. Velocity-to-zero math

**Current state** (2026-05-09): 149 entries, growing +3.3/day.

**Required for R3 close**: 0 + 0.

**Bulk-dissolution events on critical path** (per `r3-cluster-m-sequencing-plan-2026-05-09.md`):
1. **Cluster M Phase 1** (#85 + #86 substrate carriers) — ~0 immediate drop, enables Phase 2
2. **Cluster M Phase 2** (#87 cementing-test discipline) — ~20-25 test entries dissolve as cementing-test class migrates
3. **Cluster M Phase 3** (#84 bulk port) — ~50-65 remaining test entries dissolve
4. **PB-Runtime trampoline** (gate #71) — ~5 non-test entries dissolve
5. **T-LP-Retirement** (#5/#6/#7) — ~14 non-test entries dissolve
6. **T-V2-Retirement** (gate #42 partial) — ~10 non-test entries dissolve
7. **Cluster K complete** (T-Tier3 #1-#4) — ~5 non-test entries dissolve
8. **Tail (per-file/small-class)** — ~14 entries via various lane-Mgr partial events

## §5. Update cadence

- **Daily** (PM-recommended): one row per UTC day, appended to §3 snapshot history table
- **Per-cycle minimum**: every Director cycle absorption window (typically 1-2 days)
- **On bulk-dissolution event**: out-of-cadence row when a named event lands; document the event in `bulk events` column

## §6. Threshold alarms

Configure PM standing duty / dashboard:
- **Alarm 1**: 7-day net delta is `≥ +10` → trajectory-divergence flag (current state: +21 over last 7 days; alarm tripped)
- **Alarm 2**: 14-day net delta is `≥ 0` → bulk-dissolution-events-not-firing flag (current state: +30 over 9 days; alarm tripped on extrapolation)
- **Alarm 3**: 30-day net delta is `< -50` → progress-on-track flag (current state: not applicable; insufficient data)

When Alarm 1 or 2 trips, surface to Director cycle absorption + r3-program-plan §10 RED.

## §7. Relationship to R3 progress bars

When Director authors new R3-close progress bars (per operator ask 2026-05-09), this tracker becomes the data source for the SG-0 axis. The progress-bar visualization should reflect:
- Current count vs target (149 → 0)
- Recent delta direction (growing / shrinking)
- Named bulk-dissolution events queued + landed
- Velocity-to-zero projection vs R3-close window

This artifact replaces lane-completion-proxy tracking with **honest trajectory tracking** for the PB-0 closure thesis.

---

**End of tracker.**
