# Belt demotion: event-driven attempt obligations

Status: implemented on gunbc#13125. This is the design for items #6 and #4 of [dogfood-route-manual-interventions](dogfood-route-manual-interventions.md), and it executes the owner ruling "The belt demoted: wake-up and anti-entropy only" (2026-09-22, [harness-work-lifecycle](harness-work-lifecycle.md)).

The model's homes are the factory controller, which owns wake-ups and event delivery, and the factory attempt lifecycle. `factory-dogfood-route` (gunbc#13077) only consumes their receipts and carries a gating edge on them.

## The defect, re-derived (DESIGN §6b)

`gunbc.roadmap_belt_actuate` `belt_tick_for_instance` was one process that ran, in order:
- launch admission, spawn and teardown;
- commit, then verify, then review and audit, then publish, then metering, over every attempt;
- then, in `gunbc.roadmap_belt_tick_cli`, the result return, the served observation, the dogfood start record and the snapshot census.

The earliest unjustified boundary was not a slow stage. It was the claim, implicit in that one function, that these are one fact. They are separate facts:
- each attempt's capture, verify, review, audit and publish is an obligation of that attempt alone;
- launch is an obligation of the frontier;
- each instance fold is an obligation of the instance.

Fusing them made every stage's latency every other stage's latency. On srv2, from 06:50 to 16:30 on 2026-10-03, one SCM capture held every tick at its 90-minute bound, and nothing behind it ran for any attempt.

## What was built

### Obligations (`gunbc.roadmap.roadmap_belt_obligation`)

- **Per-attempt kinds:** capture, verify, review, audit, publish.
- **Instance kinds:** launch, served observation, result return, metering, dogfood start record, snapshot census.
- **Outstanding is derived, never stored.** `belt_attempt_outstanding` reads the attempt's projected segments, so no second record repeats what the receipts say (ruling Q1).
- **A failed or refused stage is not outstanding.** The line stops for that attempt only, and the operator resumes it, per the 2026-10-03 manual-resume ruling.

### One unit per attempt, one per instance obligation

- **Why per attempt.** Every `gunbc run` loads the corpus, measured at about 60 s and about 8 GB peak RSS for a typecheck. A unit per (kind, attempt) would pay that once per kind.
- **What the attempt's unit does.** It is `gunbc-obligation-attempt-<node>-<attempt>`, running `belt_attempt_obligations_cli`. It runs the attempt's outstanding kinds in eligibility order (`belt_attempt_obligations_run`), re-observing after each one, and stops at the first that waits on something outside the process.
- **Each kind is still its own ensure** over the core the tick used to call per attempt. Each core is idempotent over unchanged evidence.
- **The unit name is the exclusive hold.** `belt_obligation_hold_decision` reads the manager for that one name: a loaded unit is held, an unreadable answer refuses, anything else starts. Another attempt's unit never reaches the decision. That is the #6 isolation.
- **Deadline.** `RuntimeMaxSec` is the obligation's deadline, and `CollectMode=inactive-or-failed` lets the name be reused after a failure.
- **Grain ruling.** bold-bee-114 agreed to the per-attempt grain on 2026-10-06.

### Exit records (`gunbc.roadmap.roadmap_attempt_exit_spool`)

- **Who writes them.** Worker, reviewer and auditor units end with a create-only exit record. Their `ExecStopPost` is a typed argv: `+mkdir -- <inbox>/<node>+<attempt>+<role>+${INVOCATION_ID}+${SERVICE_RESULT}+${EXIT_CODE}+${EXIT_STATUS}`.
  - No shell is involved. This follows bold-bee-114's ruling against shell control text in minted commands.
  - The `+` prefix runs only the hook outside the unit's sandbox, so a confined reviewer cannot forge records.
  - The stop-time variables and the command-line rendering are modeled in `extdeps.systemd.service_exec`.
- **RemainAfterExit is removed from worker, reviewer and auditor units.** This reverses the 2026-09-05 choice. That choice kept a successful exit readable as `active (exited)`, which is manager memory: it is lost on reboot, and while it stands the unit never stops, so no stop hook can fire at exit. The record replaces it.
- **The supervisor keeps RemainAfterExit.** Moving its convene gate onto the record is a follow-up.
- **Observer conditions (bold-bee-114):**
  - A collected unit is read from its record.
  - A collected unit with no record is unobserved, never "never ran". The start receipt (the G5 placement record for workers, the metering roster entry for reviewers and auditors) separates "never started" from "exited without a record".
  - A failed hook surfaces as that located unobserved arm.
  - One wet control reads the exit verdict from the spool alone.
- **Signals.** A signaled worker reads as the signal number, which is exactly what `ExecMainStatus` reported, so no consumer's decision moves. A distinct signaled arm on `WorkerProcessEvidence` is a follow-up.

### Events, not a timer (`gunbc.live_deploy`)

- **No belt timer.** The deployment member is `BeltEventPathUnit`, which installs:
  - the exit path unit, a `DirectoryNotEmpty=` level on the exit inbox, which starts the drain service (`belt_exit_drain_cli`);
  - the publication-answer path unit, a `PathChanged=` edge on the answer directory, which starts the discovery service (`belt_discover_once_cli`);
  - the spool stages: inbox, queued, done and refused.
- **The drain** hands each recorded exit to its attempt's unit. It moves the entry to `queued/` once handed off and to `refused/` when it cannot be handed off, so the path goes quiet. A worker exit also queues the launch obligation.
- **The attempt's run** moves its queued entries to `done/` only after its receipt is written.
- **Launch** is the demoted tick: admission, spawn, teardown and bounded discovery. It has one unit name per instance. The deployment's belt service only queues it (`belt_launch_enqueue_cli`).
- **Deadlines need no wake-up timer.** Every awaited unit (worker, reviewer, auditor) has its own run bound, and ending at that bound writes an exit record like any other ending. The deadline is therefore an event too.

### Discovery, bounded

`belt_discover_for_instance` reads every current attempt's evidence and hands at most `belt_discovery_attempt_bound` owed attempts to their units, plus the instance obligations a moved record makes owed. It never does the work, and nothing requires it to run on a cadence.

### Receipts and the page

- **Tick receipt v7:** spawn, teardown and discovery passes.
- **Per-attempt obligation receipt:** `receipts/obligations-<node>-<attempt>.json`, consumed by `/workflow.json`. A pending verification now shows that attempt's own verify step.

### The RLM launch receipt

bold-bee-114 ruled on 2026-10-06:
- **Timer row replaced.** The two event path units must be loaded, enabled, armed and aimed at the emitted target.
- **Tick freshness replaced by an event backlog row.** An exit-spool entry unconsumed past its stage's bound refuses with `StaleBacklog`, naming the oldest entry. An armed path unit proves the watcher is armed; the backlog row proves events are consumed.
- **Declared rung drop.** The publication-answer half has no consumption record yet, so it is declared as `gunbc.rung_drop` `belt_liveness_publication_answers_unconsumed`.

## REDs, and where they run

- **#6:** `test.claim.roadmap.roadmap_belt_obligation_witness_test` `a_long_verify_does_not_delay_another_attempts_publish_or_the_observation`.
- **#4, deployment half:** `the_deployment_wakes_the_drain_on_an_exit_and_installs_no_belt_timer` in the same file.
- **#4, hand-off half:** `test.claim.roadmap.roadmap_belt_exit_drain_wet_witness` `a_recorded_worker_exit_is_handed_to_its_attempt_and_leaves_the_inbox`. This runs a real spool on the local-repo wet lane.
- **Worker hook:** `test.claim.roadmap.dispatch_worker_confinement_witness_test` `the_worker_unit_stops_with_its_process_and_writes_its_exit_record`.

## Conformance (DESIGN §3b)

- **Leasing / locking:** the hold is the manager's unit-name uniqueness. It diverges from `std.durable_exclusive_hold`, for a stated reason: the subject is the manager's own unit population, and a second store would be a second authority for which unit is running.
- **Materialization / realization:** obligations reuse the existing per-attempt cores, with no second copy.
- **Process observability:** each obligation writes its own receipt, the page reads the attempt's own receipt, and the launch receipt carries liveness against events.

## Follow-ups, stated

- **One PR per work item.** Publication still keys the PR by the attempt branch. The publish obligation is idempotent per attempt (it observes before creating), not per work item. Raised with bold-bee-114 on 2026-10-06.
- **Publication helper.** It still runs on its own timer (`PublicationHelperTimerUnit`), under a separate principal.
- **Supervisor units** keep RemainAfterExit.
- **`WorkerProcessSignaled`.** The process model has no signaled arm yet.
- **Publication-answer consumption record.** This is the rung drop's restoration trigger.

## Rulings (bold-bee-114)

- **2026-10-03, Q1:** no `std.obligation` carrier.
- **2026-10-03, Q2:**
  - Exits are recorded in an instance-local spool, not `gunbc.roadmap_event_log` (the issue history) or `gunbc.fabric_event_log`, which needs a head compare-and-set per append.
  - The hook never runs the interpreter.
  - The hook is a typed `mkdir` argv, with no shell.
- **2026-10-03, homes and PR identity:**
  - Homes are the factory controller and the factory attempt lifecycle.
  - There is one PR per work item. Not yet built; see follow-ups.
- **2026-10-06, process grain:** per attempt.
- **2026-10-06, exit records:**
  - Reviewers and auditors get exit records in this PR.
  - The deadline is only a typed timeout, never the completion route.
- **2026-10-06, RemainAfterExit:** removed, with three conditions:
  - A unit gone with no record reads as unobserved.
  - A failed hook write is a located refusal.
  - One control produces the exit verdict from the spool alone.
- **2026-10-06, RLM receipt:** event path rows, plus the backlog row or a declared drop.
