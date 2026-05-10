# R3 SG-0 Census Trajectory Tracker

**Author**: deep-wolf-155 (PM)
**Authority scope**: PM-tier daily-cadence tracking artifact. Updated per-cycle (or per Director cycle absorption) until R3 close.
**Parent**: [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](r3-pb0-velocity-walk-2026-05-09.md) — load-bearing finding that motivated daily-tracking discipline.

---

## §0. What this tracks

Daily/per-cycle snapshot of `EXPECTED_HAND_AUTHORED_NON_TEST` + `EXPECTED_HAND_AUTHORED_TEST` + `EXPECTED_HAND_AUTHORED_FRAGMENTS` counts in `src/v3/compiler/tests/integration/sg0_census_test.rs`, against the R3-close target of **0 + 0 + 0** for §1.8 gates **#8** + **#84** (Pure-Bootstrap-Zero closure). Per ROADMAP.md:177 the SG-0 PR-window discipline names the delta surface as `EXPECTED_HAND_AUTHORED_*` ∪ fragments — fragments included in this tracker post-openai-pro REQUEST_CHANGES on PR #2361 (fragments are part of T-PB-A's non-test ratchet per `sg0_census_test.rs:597-598`).

## §1. Schema (per-snapshot row)

| Field | Type | Description |
|---|---|---|
| `date` | `YYYY-MM-DD` | Snapshot date (UTC) |
| `sha` | `git short-sha` | `origin/main` HEAD at snapshot time |
| `non_test` | `int` | `EXPECTED_HAND_AUTHORED_NON_TEST` count (gate #8 target=0) |
| `test` | `int` | `EXPECTED_HAND_AUTHORED_TEST` count (gate #84 target=0) |
| `fragments` | `int` | `EXPECTED_HAND_AUTHORED_FRAGMENTS` count (T-PB-A non-test ratchet; gate #8 also gates this to 0) |
| `total` | `int` | `non_test + test + fragments` |
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
fragments=$(git show origin/main:src/v3/compiler/tests/integration/sg0_census_test.rs | \
  awk '/^const EXPECTED_HAND_AUTHORED_FRAGMENTS/,/^\];/' | grep -E '"src/v3/' | wc -l)
total=$((non_test + test + fragments))
echo "$(date -u +%Y-%m-%d) | $sha | non_test=$non_test test=$test fragments=$fragments total=$total"
```

## §3. Snapshot history

| Date | sha | non_test | test | fragments | total | delta | bulk events | notes |
|---|---|---|---|---|---|---|---|---|
| 2026-04-30 | (HEAD~500) | 38 | 81 | 1 | 120 | — | — | retroactive baseline (fragments column added 2026-05-09 schema-fix; assumed 1 entry `parse_parser_body.txt` since landing) |
| 2026-05-02 | (HEAD~300) | 40 | 87 | 1 | 128 | +8 | — | retroactive |
| 2026-05-06 | (HEAD~150) | 46 | 89 | 1 | 136 | +8 | — | retroactive |
| 2026-05-07 | (HEAD~50) | 47 | 95 | 1 | 143 | +7 | — | retroactive |
| **2026-05-09** | **c25b2d8df** | **48** | **101** | **1** | **150** | **+7 (9-day total: +30)** | (none — Cluster M cold) | velocity-walk audit landed PR #2358; remediation program in flight |
| **2026-05-10** | **cea1fbe87** | **53** | **108** | **2** | **163** | **+13 (1-day; 10-day total: +43)** | 21 PRs landed in last cycle: R3 P0 dissolutions (#2490–#2499, #2504, #2526), gates #10/#33/#37/#40/#43/#72 (#2394/2433/2449/2450/2495/2505), audit #2501, u128 mirror sync, anthropic wire demo (#2506) | **anomaly: +13 in one day** (~4× 9-day avg of +3.3/day). Hypothesis: heavy R3-gate landing cycle added test scaffolds (anthropic_messages_*, e_p_producer_demo, e6_g1a_option3_static_lens) faster than P0 Class B/C/F/G dissolutions removed entries. Per-entry root-cause deferred (yesterday baseline SHA `c25b2d8df` not resolvable in current clone — need re-baseline next snapshot). Velocity tripwire (≥3:1 7-day) not yet tripped — need 7-day cumulative count after Cluster M Phase 1 lands |

## §4. Velocity-to-zero math

**Current state** (2026-05-10): 163 entries (53 non_test + 108 test + 2 fragments), growing +4.3/day over 10-day window (+13 spike on 2026-05-10).

**Required for R3 close**: 0 + 0 + 0.

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
- Current count vs target (150 → 0)
- Recent delta direction (growing / shrinking)
- Named bulk-dissolution events queued + landed
- Velocity-to-zero projection vs R3-close window

This artifact replaces lane-completion-proxy tracking with **honest trajectory tracking** for the PB-0 closure thesis.

---

**End of tracker.**
