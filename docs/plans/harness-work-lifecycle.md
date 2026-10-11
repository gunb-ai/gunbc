# Harness work lifecycle: one durable session per attempt, supervision as a transparent cadence

> **Status: design authority for a dispatched worker's session, draft for operator review.** It restores the document at the path the roadmap rows `harness-recursive-alignment` and `factory` and a dozen code annotations still cite; the previous version was deleted in fa44b981025 (2026-10-10, v1 closeout). It keeps that version's intent -- the alignment ladder (owner direction 2026-09-21) and durable obligations over replaceable processes (owner ruling 2026-09-21) -- and replaces its realization, which turned the alignment cadence into a process restart.
> **Operator rulings, 2026-10-11.** (1) The round boundary is a supervision cadence and is transparent to the worker: most of the time the worker never knows it happened. (2) Terminology: a turn is one model reply; a round is the larger stretch of turns.
> **Evidence.** The srv1 lab bring-up of 2026-10-10/11, findings ledger F61, F64, F65, F68, F71, F72, F73, F74 and F75, and the census of the current lifecycle taken the same day (section 4).

## 1. Terminology

- **Turn**: one model reply -- one provider request, its response, and the execution of the tool calls it names. The current code calls this a round (`round.usage`, `harness_post_round`).
- **Round**: the stretch of turns between two supervision observations; 40 turns by default (`gunbc.harness.harness_throughput` `harness_round_budget_steps`). The current code calls this a turn (`harness_run_turn`, `continuation_turn_cap`, the `continuation-turn` file).
- **Session**: the one conversation an attempt runs. Its transcript is a durable fact; the process running it is replaceable.
- **Attempt**: one dispatch of a node -- one worktree, one branch, one session.

The code is renamed with the restructure (slice S5), not before, so no intermediate state carries both meanings.

## 2. What the restart realization did

- At the end of every 40-turn stretch the harness exited 3, the worker's unit stopped and its process group was killed, and the belt spawned a NEW attempt: a new worktree, a new branch from the old one, a fresh conversation. Uncommitted work was lost (F61) until a carry was added; background test runs were killed mid-run (F74); compute reservations their clients held were orphaned in a two-slot pool (F75); commands and labels the worker had learned were relearned every stretch.
- The worker was told about the boundary after every turn (`gunbc.harness.harness_guidance` `RoundsRemaining`, `RoundsEndInAlignmentCheck`) and planned around it -- yielding with a plan to resume that no successor could carry out.
- Past `continuation_turn_cap` = 3 every stretch convened a supervisor session (20 to 30 minutes). Its Nominal could not change the budget or the next brief (F65), it certified a diff by reading rather than executing (F71), and its retained unit blocked the next dispatch (F72). Five laps on two lineages produced no hand-in.
- The belt cannot distinguish a checkpoint from a spent budget: both fold to `ProviderBudgetSpent` in `gunbc.roadmap.roadmap_provider_events` `provider_execution_fold_event`.
- The ladder's cadence machinery is written and unwired: `gunbc.roadmap.roadmap_alignment` `AlignmentCadenceState` (always empty), `alignment_next_interval`, `AlignmentVerdict`, and `gunbc.roadmap.roadmap_checkpoint_plan` reach only tests.

## 3. Requirements

1. **Transparent rounds.** A round boundary does not end, restart or inform the worker. Its conversation, worktree and the processes it started continue. The per-turn countdown notice is deleted.
2. **Supervision is an obligation, not a phase.** At each round boundary the harness persists a supervision observation (the round's turns, the diff, the receipts produced) and the belt's alignment obligation, due on that event, reads it out of band while the worker keeps running. Its verdict is the ladder's magnitude and maps to exactly one action: aligned, nothing, and the worker never knows; misaligned, a steer appended to the same conversation before the worker's next turn; extremely misaligned, the supervisor, with the session paused; catastrophically misaligned, stop. A check whose answer cannot change the next turn is not run.
3. **Durable session, replaceable process.** The transcript is persisted every turn (the wire log already writes the full request); a process that ends unexpectedly -- an OOM, a host restart, a deploy -- is replaced by one that resumes the same conversation in the same worktree. A fresh conversation is never substituted for a lost one: an unreadable transcript stops the attempt with a typed cause.
4. **The per-command bound stays.** `harness_default_duration_limit_seconds` returns control to the worker when a command hangs; a worker that wants to wait backgrounds the command and polls. Background processes live as long as the session.
5. **A session ends only by** hand-in (the submission file), stop (supervision or operator), an exhausted budget (escalated; a renewed budget continues the same session), or an unrecoverable fault.
6. **Context growth is handled inside the session** by the existing compaction, which a longer session will exercise for the first time.
7. **One unit per attempt for the session's life**, collected on any exit with the exit record as its evidence (F68), and no held resource outlives its owner (`gunbc.plans.convergence_coverage_program` cc-guaranteed-cleanup).
8. **Verdicts are executable; claims are executed.** A supervisor's verdict is one of the actions above, never advice the route cannot carry (F65). A supervisor's statement that work passes is not verification; verification stays the belt's obligation (F71).

## 4. What changes (census, 2026-10-11)

### Deleted: exists only for restart-per-stretch

- Harness: `harness_turn_checkpoint_due`, `harness_event_alignment_checkpoint_due`, `TurnAlignmentCheckpointDue` and its exit-3 arm in `harness_turn_exit`, `RoundsEndInAlignmentCheck` and the per-turn `RoundsRemaining` notice.
- Events: `HarnessAlignmentCheckpointDue` and its fold arm.
- Continuation lane (`gunbc.roadmap.roadmap_attempt_continuation`): `ContinueFromAttempt`, `continuation_turn_cap`, `ContinuationTurnRead`, `continuation_continue_next_turn`, `continuation_brief_prefix` and the stretch arms of `continuation_from_capture`.
- Spawn: `belt_continuation_carry`, `belt_attempt_state_initialize_after_carry`, `belt_origin_predecessor_key`, the continuation branch of `attempt_origin_base_ref`, `dispatch_continuation_alignment_preamble`; the `continuation-turn` file and the cross-attempt half of the alignment-binding gate.
- Supervisor plumbing keyed on the stretch: `AttemptSupervisorRole` turn, `MeteredSupervisor` turn, the `-tN` unit names.

### Reshaped

- The turn loop: the 40-turn count becomes a cadence that emits a supervision observation and continues, not a terminal.
- The wire log becomes the record a replacement process resumes from (the remaining work of the roadmap row `harness-conversation-log-forking`).
- Alignment: wire `AlignmentCadenceState`, `alignment_next_interval` and `AlignmentVerdict` as the alignment obligation's state and verdict.
- The supervisor is convened by the alignment obligation on an extremely-misaligned verdict, observes a paused live session, and has no stretch cap.
- Metering per session, not per stretch; the issue page shows supervision verdicts on the round strip.

### Kept

- Per-command and per-request bounds; compaction; seat bind and release; the exit record and unit lifecycle; the capture, verify, review, audit and publish obligations; `FreshFromBase`.

## 5. Open decisions

1. **Does the worker pause while a round's alignment check runs?** Proposed: no. It continues, and a steer lands before its next turn after the verdict; only an extremely- or catastrophically-misaligned verdict pauses or stops it.
2. **Where does a package-wide acceptance check run?** Proposed: in the belt's verify obligation after hand-in, which has no turn bound; the worker hands in when its node's own oracle holds. Today both the worker brief and verify name checks, and only verify runs where long work can finish.
3. **Review- and audit-requested changes**: continue the same session with the review as a steer, or start a fresh attempt. Proposed: the same session.
4. **Budget renewal**: who renews an exhausted session (the supervisor, the operator) and by how much.

## 6. Slices

Each slice is proven on the srv1 lab by a live lineage, with no operator hand step.

1. **S1 Resume.** A replacement process resumes the session from the wire log in the same worktree. Control: the worker is killed mid-session and continues with its conversation intact.
2. **S2 Transparent boundary.** The harness no longer exits at a round boundary; it persists the supervision observation and continues; the countdown notice is deleted. Control: a background job started in one round is read back in the next, and an aligned transcript carries no boundary text.
3. **S3 Alignment obligation.** The belt runs the check out of band on the observation; aligned does nothing; misaligned writes a steer the harness appends before the next turn. Control: a misaligned fixture produces a steer the worker receives; an aligned round produces none.
4. **S4 Escalation and stop**, and deletion of the continuation lane, the stretch cap, the carry and the `-tN` units.
5. **S5 Terminology** across the harness, events, metering and the page.
6. **S6** The acceptance placement of decision 2.

## 7. Controls

- RED for the design: a session whose round boundary kills a running background process, or whose transcript contains boundary text while its rounds were aligned.
- Positive control: the red control of the roadmap row `harness-recursive-alignment`, a lineage that crosses two round boundaries and a lab deploy and hands in one candidate, with no operator hand step.

## Dissolution trigger (DESIGN §6)

Delete this document when slices S1 to S5 are realized and their rules are carried by the harness and roadmap modules' own declarations and witnesses.
