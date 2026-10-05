# Arm recovery as convergence: observe the arm, plan at arm grain, delete the leftover classifier

Status: DESIGN, for agreement before code (operator-approved direction, 2026-10-05, relayed by
valiant-crab-775). Governs `gunbc.spark.native_serving_apply` and its launch/observe modes.

## 1. The defect, at its root

The native arm transaction keeps per-host files beside each unit (`.gunbc-transaction` fence,
`.gunbc-incumbent` backup, `.gunbc-had-no-unit` marker, `.gunbc-unresolved-rollback` marker), and
recovery classifies those files against a hand-enumerated list of leftover kinds:
`SparkNativeLeftoverDecision`, `SparkNativeLeftoverResolution`, `SparkNativeAdoption`, and
`LeftoversRecoverable`. Anything that is not on the list is refused. Three things are wrong with
this, and they share one root: **the files are standing in for an observation of the arm.**

- **The kinds multiply.** The number of states is roughly hosts × stages × failure points, and every
  enumerated kind is one more case a reviewer has to reason about. On 2026-10-05 (run 37254736596)
  three ranks had rolled back and were running while the head was fence-only and stopped. That
  matched no kind, so Group A stayed down until a human intervened.
- **Recovery is per-host.** No arm-level invariant ever sees the hosts together. Tonight's
  hand-recovery proved this: starting only the head (03:27Z) timed out at the rendezvous with 1 of 4
  ranks joined, because the workers had failed at 02:48Z. The arm came back only when all four were
  started together (about 03:45Z).
- **The fence's 2-hour horizon is a timer standing in for "is the writer dead".**
  `std.durable_exclusive_hold` already says why that is wrong: a hold is broken only because its
  holder was *observed* gone, never because it is old.

## 2. The replacement, in the existing homes

The replacement is one loop: **observe → assess against a desired arm → plan at arm grain →
actuate → readback**. Launch, rollback and recovery are three uses of it, differing only in which
arm is desired.

### 2.1 Observe each rank (new producer, existing carriers)

For every rank of the arm, one read returns:

- the unit file content identity (sha256 of the bytes);
- `LoadState`, `ActiveState`, `SubState` and `NRestarts`;
- the container incarnation: id, running state, restart count and start time, from the same
  `ContainerInspect` the readback already consumes;
- the rendezvous the container targets: master address and port, node rank and world size, read
  from its argv through the existing `observed_rank_incarnation`.

An unreadable field is an unknown, never an absence. The read is wrapped in
`std.goal_assessment` `ObservationAttempt`, so a failed read is `ObservationRefused`, never a state.

### 2.2 The desired arm

The desired arm is the per-rank planned unit text and container spec that the step already renders
(`SparkNativeHostStep`). There are two sources:

- **Launch:** the requested staircase step.
- **Recovery and rollback:** the last committed arm. Commit writes one record per arm on the head
  host (under the existing `/var/lib/gunbc` store root): the step identity and each rank's unit
  content hash. Recovery re-renders that step from source and **refuses** if a re-rendered hash
  differs from the recorded one, because a desired arm that cannot be reproduced is not desired, it
  is guessed.

This replaces the per-host `.gunbc-incumbent` backups. The record is the only file that recovery
reads.

### 2.3 Assess (std.goal_assessment, remedy-free)

`inspect_goal` with the goal being the desired arm. The deviations are per-rank and typed:

- unit content differs;
- unit not active;
- container absent or a different incarnation;
- rendezvous targets another master or world size;
- ranks are running but belong to different incarnation sets (started in different windows).

An unknown field gives `GoalIndeterminate`; an arm whose rank roster does not match the plan gives
`GoalAssessmentRefused`.

### 2.4 Plan at arm grain (std.upsert_decision)

The planner maps the assessment to `UpsertDecision<ArmConvergePlan>`:

- `GoalSatisfied` → `Noop`;
- `GoalIndeterminate` and `GoalAssessmentRefused` → `Refuse`, carrying the unknowns. Generic
  remediation is refused over indeterminate evidence, as `std.goal_assessment` already rules.
- `GoalDiverged` → `Apply` with **one** plan shape, whatever the deviations are:
  1. write the unit files that differ;
  2. stop every rank;
  3. clear the start limit on the loaded units (`spark_native_clear_start_limit`);
  4. start the head, then every worker, within the rendezvous window.

There is **no subset start**. Any deviation restarts the whole arm as one incarnation set. That is
the arm's invariant, and it is the reason no leftover kind needs to exist: a mixed arm is just
`GoalDiverged`.

Then the existing readback runs: field-for-field container comparison, the per-rank NCCL
announcement wait, and the bounded front door. That is `ensure`'s independent readback, unchanged.

Rollback is the same planner with desired = last committed arm. A failed rollback leaves nothing
special behind: the next run observes, assesses and converges. Every file other than
the committed record is then unread, and therefore harmless. The first converge deletes the legacy
fence, backup and marker files as a declared, one-time migration step (§4).

### 2.5 Writers are serialized by the job's concurrency group; there is no fence and no hold file

(Revised with valiant-crab-775, 2026-10-05, after the hold was found to be a second authority.)

The fence's only remaining job is to serialize writers, and an existing authority already does
that: the fleet-converge job's `concurrency` group, emitted from `ci_spec` through
`gunbc.fleet_converge_workflow`. GitHub releases the group exactly when the **job** ends, so
the group's own state is the holder-liveness read the hold would have needed. That covers the
parent's three conditions on the hold:

- a re-run attempt is a new job, queued behind the group;
- a hand run never holds the group;
- release follows the job's own state, not the run's conclusion.

A hold file would have been a second serialization of the same fact. It would also have needed a
GitHub token in the arm job and a store that two executors share; `durable_exclusive_hold_file_store`
is local-only.

So the per-host `.gunbc-transaction` fence is deleted, and every arm-writing mode joins its arm
group's concurrency group, `gunbc-arm-mutation-<group>`, following the existing
`gunbc-arm-mutation-glm-group-b-native`. The arm-writing modes are launch, the recovery converge,
adoption of an already-running arm as committed, and any rollback-only path. A witness over the
`ci_spec` rows asserts that every arm-writing mode maps to its arm's group, exhaustively and with no
default.

A killed run (step timeout, cancel) leaves no special state: the next run observes and converges.

**Measured dispatch fact.** With `cancel-in-progress: false`, a group still holds at most ONE
pending run, so a third dispatch cancels the queued middle one. That is safe, because a run
cancelled while pending never ran. The receipt and the dispatcher therefore report "cancelled while
pending" as NOT RUN, never as a failed launch.

**The honest residue: hand actuation is unserialized.** A person acting on the hosts directly holds
no group and is excluded by nothing. Their effect is caught only by the next observation, as
`GoalDiverged` (the realized arm differs from the desired one) or as an unknown (`Refuse`). No
mechanism here prevents it, and none is claimed.

## 3. What is deleted (the root cut, DESIGN §3)

The following go away, not kept beside the new path:

- **Types:** `SparkNativeLeftoverDecision`, `SparkNativeLeftoverResolution`, `SparkNativeAdoption`
  and their arms (`LeftoversAdoptable`, `LeftoversRecoverable`, `RecoverRolledBack`,
  `ResolveFenceOnly`, and the rest).
- **Leftover functions:** `spark_native_leftover_decision_of_leg`,
  `spark_native_leftover_observe_script`, `spark_native_leftover_resolve_script`.
- **Adoption and recovery:** `spark_native_adoption_of`, `spark_native_recovery_readback`,
  `spark_native_marker_clear_script`, and the incumbent/absent/unresolved suffix constants.
- **The admission verdict** `GroupArmLaunchAdmittedForAdoptionOnly` and the `ArmOwnFence` reading in
  `gunbc.spark.group_arm_launch`.
- **Witnesses:** the leftover and adoption witnesses in `native_serving_resumable_apply_witness_test`
  (about 20 claims). Their discriminating cases are re-expressed as assessment and plan witnesses.

Consumers enumerated before deletion: these live outside the required gate, so the substrate will
not report them. I will produce the list from `gunbc.module_impact_query` over the deleted
declarations, and post it in the PR before deleting anything.

## 4. On-host migration

Today Group A carries a legacy fence on srv6 and unresolved-rollback markers on srv5, srv7 and
srv8, while serving fp8-16 that was started by hand.

The first converge under the new path does the following:

1. Runs under the arm group's concurrency group.
2. Observes the arm against the requested step's render.
3. Adopting tonight's hand-started fp8-16 as committed is therefore just a launch of fp8-16 under
   the same mode. The assessment against fp8-16's render is either `GoalSatisfied` → `Noop`, or
   `GoalDiverged` → a whole-arm restart. Then the ordinary readback runs, and only its success
   writes the committed record. There is no separate adoption mode and nothing to enumerate.
4. Deletes the four legacy file kinds as one declared step.

## 5. Witnesses (supplied at the planner's interface, plus one inhabitance claim)

- **Tonight's lesson, as the RED:** observation = head unit inactive with no container; workers'
  units active, containers running, `NRestarts` > 0, rendezvous unjoined. The plan must be a
  whole-arm stop then start, head first. A head-only start is the red, and the planner has no
  constructor for one.
- **The mixed state of run 37254736596** (three ranks rolled back and running, head fence-only)
  plans a whole-arm converge. It is not refused.
- **An arm that is all running but split across incarnation sets** plans a whole-arm restart.
- **An unknown field** gives `Refuse` naming it, never `Apply`.
- **A satisfied arm** gives `Noop`, with no actuation.
- **Serialization:** every arm-writing mode maps to its arm group's concurrency group, read
  exhaustively from the `ci_spec` rows with no default.
- **Inhabitance:** the real per-rank observation producer, over a captured observe receipt from
  Group A, yields the same assessment the supplied cases assume.

## 6. Cut

The authority transition must be atomic, because the leftover classifier and the converge both
answer "what does this arm need". So it is one PR (on top of #13319) that lands the observation,
assessment, planner, committed record and concurrency-group keying, and deletes §3 in the same change.

If review finds it too large, the only admissible split is root-first:

1. **First PR:** the committed record and the concurrency-group keying, replacing the fence and
   backups; the classifier is deleted in that same PR.
2. **Second PR:** the planner.

Under that split, the interval between the two has no recovery path except refusal, and that is
stated rather than hidden.

The start-limit and transport-wait repairs on #13319 stand on their own and land first.
