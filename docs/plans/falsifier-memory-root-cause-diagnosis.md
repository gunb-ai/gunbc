# Falsifier reliability diagnosis

**Status:** updated 2026-08-05 (session bold-eagle-790). **Scope split (operator
ruling on #7813):** **7813A** (this PR, mergeable) = §5 build→floor job isolation
only. **7813B** (wise-ram-22 / swift-otter-728, NOT this PR) = §4 parent-first
bisect with memory.current/peak/events, oom_kill, oomd, termination signal, cgroup +
runner identity — do not treat §4a as completed root cause.

Operator correction (still-bat-561): **there is no memory event** — governor
receipts show `budget_exceeded=0 forced_serial=0` on every examined run; step 11
**completes** and exits 1 on witness redness. This doc separates two failure classes
previously conflated under "nine red windows." PR #7813A (build/floor job split) is
cgroup-headroom hygiene only, separate from witness containment (wise-ram-22).

**Rebase:** #7813A rebases onto main after #7812 (dark controls) merges; do not
compete on `falsifier_workflow.dag` until then.

---

## 1. Two failure classes (do not conflate)

| Class | Windows | Heads (approx) | Mode | Host behavior | Root cause lane |
|---|---|---|---|---|---|
| **A — BudgetExceeded** | 3 (2026-08-03 03:20–14:46) | `7024637b0`, `19cc776d4`, `76ef121ec4` | eval budget | **Host-sensitive** — `19cc776d4` PASSED srv3-01, FAILED srv4-05 | `witness_committed_is_fixed_point` in `generated_artifact_drift_test.dag` at 5004 ms vs 5000 ms fast-lane budget |
| **B — WitnessRed** | 7+ (2026-08-03 17:53 → present) | `44126ca1de` onward | structural | **Deterministic** — `772d92d18` failed identically on srv3-02 and srv4-01 | Five witnesses on main (see §2); bisect window `76ef121ec4..44126ca1de` |

Class A and Class B are **unrelated**. Neither is a cgroup OOM, oomd kill, or governor
budget_exceeded event.

---

## 2. Class B — current failing witnesses (main)

All present on origin/main; deterministic across four hosts:

| Witness | Entry |
|---|---|
| `fn_wf_no_trigger_negative_control` | `dag/test/claim/generic_item_clone_bound_witness_test.dag` |
| `witness_direct_rust_door_provenanced_artifact_holds` | `src/v2/test/claim/long/direct_rust_door_emission_witness_test.dag` |
| `witness_v2_emitter_direct_rust_door_closing_contract_holds` | `src/v2/test/claim/long/v2_emitter_direct_rust_door_acceptance_test.dag` |
| `witness_production_admissions_close_contract_holds` | `src/v2/test/claim/long/v2_emitter_direct_rust_door_contract_test.dag` |
| `witness_written_source_assessment_establishes_on_fixture` | same contract entry |

Falsifier step 11 runs ~1h39m–1h55m (cold whole-corpus), completes all eight receipt
phases, uploads receipts, then exits 1 because witnesses failed — **not killed**.

---

## 3. Governor receipt (no memory machinery engaged)

Verbatim pattern on examined failing runs (four hosts):

```
budget_exceeded=0 forced_serial=0 hard_backoffs=0 creep_backoffs=0
headroom_holds=0 ceiling_holds=0 max_width_reached=0
```

Floor peak RSS ~9.5 GiB against `memory.max=17179869184` (16 GiB). Width **1**
throughout (`governor target_width=1`). No oom_kill, no throttle, no forced
serialization. This is **not** the plural-worker outer-ring class from the v1
run-stability lane.

---

## 4. Class B bisect (`76ef121ec4..44126ca1de`) — **7813B / wise-ram-22 (WIP, not merge criterion)**

**NOT part of #7813A deliverable.** Interim parent-first bisect receipt below;
introduction commit not found. Continuation requires fresh binary at every head plus
memory.current, memory.peak, memory.events, oom_kill, oomd, termination signal, cgroup
and runner identity per operator ruling — memory.peak is historical max, not current
residency; positive OOM evidence required, never inferred from peak or exit code.

Seven commits between last BudgetExceeded window and first WitnessRed window:

1. `d16e17331c` — #7559 namespace occurrence identity
2. `b12027cd94` — #7728 disk-tier repeat-resolve
3. `0b6b99315f` — #7702 coordinator replay
4. `faa95d207e` — #7738 falsifier-is-flakey WIP
5. `4e089ef896` — #7725 P1 retention vs drain
6. `7237024035` — #7594 sole modeled publisher
7. `44126ca1de` — **#7682 Lane A R1 invert interpreter dispatch authority**

Method: run named witnesses per candidate head via
`claim_batch --entry … --functions …` (not full falsifier). Results in
§4 receipt below when bisect completes. Prior: #7682 given batch-1 witness +
direct_rust_door contract shape; **#7776 ruled out** (landed after first WitnessRed
window).

### 4a. Interim bisect receipt (executed 2026-08-04/05; **incomplete — search widening backward**)

Method: fresh `claim_batch` build at each candidate head (stale Aug-2 binary ruled out
for dispatch-layer changes). Parent-before-candidate test per still-bat-561.

| SHA | Commit | Fresh build | `fn_wf` | `closing_contract` | `production_admissions` | `written_source` |
|---|---|---|---|---|---|---|
| `76ef121ec4` | last BudgetExceeded window head (#7722) | yes | **FAIL** | **FAIL** | (not run) | (not run) |
| `7237024035` | parent of #7682 (#7594 publisher) | yes | **FAIL** | **FAIL** | **FAIL** | **FAIL** |
| `44126ca1de` | **#7682 dispatch authority** | yes | **FAIL** | **FAIL** | **FAIL** | **FAIL** |

**#7682 ruled OUT:** all four tested witnesses already fail at parent `7237024035` with a
binary built at that head. The first WitnessRed *falsifier window* coincides with #7682 as
HEAD, but witness redness predates it — likely enrollment or cadence visibility, not this
commit introducing the failure.

**`76ef121ec4` boundary:** `fn_wf` and `closing_contract` already **FAIL** with fresh
build — Class B witness redness predates the BudgetExceeded/WitnessRed window split.
Introduction is earlier than the seven-commit window.

**Next candidates** (after #7682 ruled out): `0b6b99315f`, `b12027cd94`, then remaining
commits before `7237024035`.

---

## 5. Infrastructure — build/floor job split (**7813A / PR #7813, mergeable**)

The monolith job carries ~9.5 GiB pre-floor residency (release build + selection
control) in the same cgroup as the floor peak measurement. Splitting into
`falsifier-build` → `falsifier` (mirrors ci build→ci) recovers cgroup headroom for
peak attribution. This does **not** fix Class B witness redness; it is orthogonal
cgroup hygiene.

---

## 6. Retracted claims

- **"Memory-alive / 15 GiB pin as failure mode"** — cgroup peak proximity is real but
  governor never engaged; exit 1 is witness redness.
- **"Width>1 plural-worker defect"** — falsified; width=1 on all examined runs.
- **"Bisect requires two-hour falsifier"** — direct witness runs per commit suffice.
