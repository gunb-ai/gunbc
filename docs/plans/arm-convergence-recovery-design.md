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
special behind: the next run observes, assesses and converges. Every file other than the hold and
the committed record is then unread, and therefore harmless. The first converge deletes the legacy
fence, backup and marker files as a declared, one-time migration step (§4).

### 2.5 The fence only serializes writers (std.durable_exclusive_hold)

The fence becomes a `std.durable_exclusive_hold` slot per arm, stored through the existing
`gunbc.durable_exclusive_hold_file_store`. Acquire and release use compare-and-set on the
generation. **Holder liveness is observed, not timed.** The holder is the writer's identity: the
workflow run id and attempt. Its liveness is read from that run's status:

| run status | liveness verdict |
|---|---|
| `in_progress` | `HolderObservedLive` |
| `completed` | `HolderObservedDead`, with the run's conclusion as evidence |
| unreadable | `HolderLivenessUnobservable` → refuse |

Recovery of a dead holder's hold goes through `durable_hold_recovery_assess` →
`file_hold_recovery_assess` (an `admit_callers` row added for the arm converge).

**Choice to make:** the brief says "a lease with a deadline the holder renews". I recommend
observing the run instead. It answers "is the writer dead" directly, needs no renewal machinery, and
is the method `std.durable_exclusive_hold` asks for: observe the holder, never age. A renewed lease
still declares death by a clock: a stalled-but-live writer loses its hold, and a renewal loop is one
more effect that can fail. If you prefer the lease, `HeldLease` carries no deadline today. The
termed carrier is `gunbc.product.capacity.lease` `LeaseGrant`, and the hold's liveness verdict would
be minted from that grant's deadline. Either way, nothing reads a fixed horizon.

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

1. Acquires the new hold. No new slot exists yet, so it is absent.
2. Observes the arm. It is `GoalSatisfied` only if a committed record exists. None does yet: the
   hand start committed nothing.
3. Is therefore asked explicitly to converge to a named step. **Adopting the running arm as
   committed is a separate, declared operation**: observe, assess against the named step's render,
   and write the record only on `GoalSatisfied`.
4. Deletes the four legacy file kinds as one declared step, after the hold is held.

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
- **The hold:** a live holder refuses; a completed run lets the hold be recovered; an unreadable run
  refuses. No clock is read.
- **Inhabitance:** the real per-rank observation producer, over a captured observe receipt from
  Group A, yields the same assessment the supplied cases assume.

## 6. Cut

The authority transition must be atomic, because the leftover classifier and the converge both
answer "what does this arm need". So it is one PR (on top of #13319) that lands the observation,
assessment, planner, committed record and hold, and deletes §3 in the same change.

If review finds it too large, the only admissible split is root-first:

1. **First PR:** the hold and the committed record, replacing the fence and backups; the classifier
   is deleted in that same PR.
2. **Second PR:** the planner.

Under that split, the interval between the two has no recovery path except refusal, and that is
stated rather than hidden.

The start-limit and transport-wait repairs on #13319 stand on their own and land first.
