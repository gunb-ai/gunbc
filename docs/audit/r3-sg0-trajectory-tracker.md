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

**Seven-day velocity-tripwire anchor (canonical).** Path-churn/tripwire rows use fixed **seven** UTC calendar days ending on snapshot UTC date **`D`** (seven days means from UTC day **`D−6`** through **`D`** inclusive). **Baseline** = latest `origin/main` commit strictly before **`D−6` 00:00:00 UTC**; **head** = **`origin/main`** at snapshot row SHA. Anchoring **`--before` on `D−7` UTC midnight** against an **EOD** snapshot dated **`D`** spans **approximately eight** calendar days — it mislabeled earlier prose as rolling **7d** and inflated introductory counts; **2026‑05‑11** rows normalize to **`4b156d839`** → **`eed86ffc9`**.

## §3. Snapshot history

| Date | sha | non_test | test | fragments | total | delta | bulk events | notes |
|---|---|---|---|---|---|---|---|---|
| 2026-04-30 | (HEAD~500) | 38 | 81 | 1 | 120 | — | — | retroactive baseline (fragments column added 2026-05-09 schema-fix; assumed 1 entry `parse_parser_body.txt` since landing) |
| 2026-05-02 | (HEAD~300) | 40 | 87 | 1 | 128 | +8 | — | retroactive |
| 2026-05-06 | (HEAD~150) | 46 | 89 | 1 | 136 | +8 | — | retroactive |
| 2026-05-07 | (HEAD~50) | 47 | 95 | 1 | 143 | +7 | — | retroactive |
| **2026-05-09 (mid-day)** | **c25b2d8df (stale)** | **48** | **101** | **1** | **150** | **+7 (9-day total: +30)** | (none — Cluster M cold) | velocity-walk audit landed PR #2358; remediation program in flight; row recorded mid-day, not EOD (see retroactive correction below) |
| **2026-05-09 EOD** | **eb2cc15cd** (last commit before 2026-05-10 UTC) | **53** | **107** | **2** | **162** | **+12 (retroactive correction; 9-day total: +42)** | 12 entries landed during 2026-05-09 evening: T-CostLens γ-ratification (#2283), R3 plan audit (#2501), 21-PR cycle (R3 P0 dissolutions + gate landings + audit + bridge retirements) | retroactive baseline correction; previous tracker row (150) was taken mid-day on 2026-05-09 before the evening cycle landed; honest EOD count is 162 |
| **2026-05-10 (00:38Z)** | **f1588bcc8** | **53** | **107** | **1** | **161** | **−1 (vs 2026-05-09 EOD)** | (none) — 1 fragment removed (`parse_corpus_manifest.txt`-class transient) between eb2cc15cd and f1588bcc8 | Director-receipted intermediate reading at gunbc#828 c#4414054598 (committed on `session/gentle-newt-665` branch as 88e6fca99; never merged to main due to session archival). Director framing: "Trajectory NOT yet inflected toward shrink — bulk events queued (gate #6 wise-crane-831 ACTIVE, F2 PR #2473, T-Tier3 D2a PR #2285, carve-promotion #81/#82/#83/#95) but pre-land at snapshot. Alarm 1 + Alarm 2 tripped on extrapolation against the (then-stale) 150 baseline." Honest delta vs corrected EOD baseline (162) is **−1** (a marginal shrinkage), not +11 |
| **2026-05-10 (later)** | **cea1fbe87** | **53** | **108** | **2** | **163** | **+2 (vs f1588bcc8 intermediate; +1 vs 2026-05-09 EOD)** | PR #2506 [codex] add anthropic wire demo (1 test entry + 1 fragment re-added: `anthropic_messages_wire_demo_test.rs` + manifest fragment) | **velocity is steady, not anomalous** (prior +13 framing was a baseline-comparison artifact; Director's +11 framing was the same artifact against the 150 baseline). True 1-day delta vs 2026-05-09 EOD is +1 entry net; cumulative 10-day total is +43 (+30 prior + +12 retroactive 2026-05-09 evening + +1 today). Velocity tripwire (≥3:1 introduction:dissolution, 7-day): **status pending/uncomputed** — 7-day cumulative ratio not yet computed in this snapshot; will be computed once Cluster M Phase 1 lands |
| **2026-05-11 (EOD)** | **eed86ffc9** | **50** | **116** | **2** | **168** | **+1 vs 31acf4357 (167)** | merge churn vs midday snapshot SHA: net **−3** non_test / **+4** test | **True rolling 7 calendar UTC days** ending **2026-05-11** (start **2026-05-05**): baseline `4b156d839` (last `origin/main` before **2026-05-05 00:00 UTC**). Σ-count **`137→168 (+31)`**. **Velocity tripwire** (`EXPECTED_HAND_AUTHORED_*` path-set intros:dissolves vs that baseline): **37:6 → 6.17:1 (≥3:1, TRIPPED)**. Midday causal story unchanged: four gate/T-Workflow-As-Data tests landed on the 31acf4357 ancestry (+4 vs cea1fbe87 on that lineage); §1.8 promoter sweeps are still separate bookkeeping. Bulk-port shrink not yet inflecting SG-0. |

## §4. Velocity-to-zero math

**Current state** (2026-05-11 EOD / `origin/main`): **168** entries (50 non_test + 116 test + 2 fragments). Midday row (31acf4357) captured +4 vs 2026-05-10 cea1fbe87 on that SHA; HEAD advanced to **eed86ffc9** with **net +1** on the Σ-count (+4 test churn vs −3 non_test net). **Seven calendar UTC days** ending **2026-05-11** (inclusive): baseline **`4b156d839`** (last `origin/main` commit strictly before **2026-05-05 00:00 UTC**); head **eed86ffc9**. Σ-count **`137 → 168` (+31)**. **Introduction:dissolution path ratio** over that window (unique paths added vs removed within the three `EXPECTED_HAND_AUTHORED_*` slices): **37 : 6** (**6.17:1**) — **still fires the ≥3:1 standing tripwire** until bulk-dissolution events dominate. Census still accumulating (Cluster M Phase 3 bulk port not yet kicked in).

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
- **Alarm 1**: 7-day net delta is `≥ +10` → trajectory-divergence flag (**2026-05-11 EOD**: Σ-count Δ vs true-7d baseline `4b156d839` (pre-**2026-05-05** UTC) = **`+31` ; tripped**)
- **Standing velocity tripwire (Mgr)**: rolling **7 calendar UTC days** **path introductions : path dissolutions** in `EXPECTED_HAND_AUTHORED_{NON_TEST,TEST,FRAGMENTS}` **`≥ 3:1` ⇒ RED** (**2026-05-11 EOD**: **`37 : 6` ⇒ 6.17:1; tripped** — procedure: sort-unique `"src/v3/..."` lines inside each const slice; `comm` add/remove vs baseline SHA **`4b156d839`**, snapshot head **`eed86ffc9`**; baseline encodes **start of UTC day May 5** vs **snapshot EOD May 11** inclusive).
- **Alarm 2**: 14-day net delta is `≥ 0` → bulk-dissolution-events-not-firing flag (current state: +30 over 9 days; alarm tripped on extrapolation)
- **Alarm 3**: 30-day net delta is `< -50` → progress-on-track flag (current state: not applicable; insufficient data)

When Alarm 1, Alarm 2, or the standing velocity tripwire fires, surface to Director cycle absorption + `r3-program-plan` §10 RED.

## §7. Relationship to R3 progress bars

When Director authors new R3-close progress bars (per operator ask 2026-05-09), this tracker becomes the data source for the SG-0 axis. The progress-bar visualization should reflect:
- Current count vs target (150 → 0)
- Recent delta direction (growing / shrinking)
- Named bulk-dissolution events queued + landed
- Velocity-to-zero projection vs R3-close window

This artifact replaces lane-completion-proxy tracking with **honest trajectory tracking** for the PB-0 closure thesis.

---

**End of tracker.**
