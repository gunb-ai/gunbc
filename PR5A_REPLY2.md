Addressed review 95816 — commit `587282e`.

**Finding 1 — an unread leg was reported like a missing item (fixed).** An SSH/sudo failure or a non-zero `cat` used to collapse into `Absent`→`Missing` ("the drift the apply must write"), so `remote_build_serving_observe` could exit green with `terminal=drift` for a host it never read — and PR 5(b)'s apply would then WRITE items nobody observed. Violated fail_closed ("failing to compute the answer is a different outcome from computing a conservative one"). Now each read is an `ObservationAttempt` (`ObservationEstablished` for a value read — incl. `test` exit 0/1 as a *confirmed* present/absent, `test -f` exit 1 confirming `authorized_keys` is genuinely absent — or `ObservationRefused` for ssh/sudo/error). `serving_assess` folds established facts into **deviations** and refused legs into **unknowns** through `std.goal_assessment::assess_by_deviations_and_unknowns`; **unknowns win**, so a partly-unread host is `GoalIndeterminate` and the run **exits failure** (never reports a plan the apply could act on). A confirmed-absent dir/key stays `GoalDiverged` (runnable drift).

**Finding 2 — a second goal-assessment vocabulary (fixed).** `ServingItemVerdict` (Met/Drifted/Missing/Refused) was a rival to `std.goal_assessment::GoalAssessment`. Removed; the assessment is now a `GoalAssessment` (`GoalSatisfied`/`GoalDiverged`/`GoalIndeterminate`/`GoalAssessmentRefused`), mirroring `gunbc.machine_intake.host_resource_observation` (`TopologyFieldUnread{cause}` for an unread field → a `ServingUnknown`, never a deviation).

**Evidence (both witnesses green, `--claim-run`, exit 0).** The witness set now discriminates the exact failure the finding named:
- `an_unreadable_passwd_is_indeterminate_not_drift` and `an_unreadable_store_leg_is_indeterminate` — a refused leg yields `GoalIndeterminate`, NOT `GoalDiverged`, and `serving_assessment_runnable == false`.
- `confirmed_absent_store_dir_is_drift_not_indeterminate` / `confirmed_absent_authorized_keys_is_drift_not_indeterminate` — the paired control: a *confirmed* absence is still runnable drift (real drift not collapsed into indeterminate).
- `the_receipt_names_indeterminate_never_drift_for_an_unread_leg` — the receipt says indeterminate, never drift, for an unread host.
- setup witness 18/18, converge witness 3/3 (incl. `the_receipt_body_leaks_no_forced_command_material`). PASS lines in the body.

No workflow regen needed — the observe `--function` signature and the receipt path are unchanged.

— sent from loyal-bear-900
