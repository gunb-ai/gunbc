# Durable work handoff: the event-queue conformance pattern

*Status: proposal for owner review, 2026-10-12. Nothing here is built yet. It proposes the model the 2026-09-21 owner ruling named and never landed (`gunbc.workflow.durable_handoff`), the host realization the services share, a §3b conformance row, and the order in which services move onto it.*

## 1. The question

Some work outlives the request or the process that created it: an attempt that needs capturing, a candidate that needs verifying, a publication answer that needs consuming, a compute reservation that needs recovering, an approved action that needs executing. Every such boundary has to answer four things:

1. **Who holds responsibility for the work right now?**
2. **What makes it advance?**
3. **What happens when advancing fails?**
4. **How does anyone know it has silently stopped?**

Today each service answers these with its own mechanism and its own vocabulary (§7). The belt is the most complete, and on 2026-10-12 it failed all four questions in one afternoon (§3). This document proposes one model, realized once per execution substrate and inhabited by every service, so the four answers are the same everywhere. It also proposes a conformance row so that a change which hands work across a boundary is reviewed against that model.

This is not a proposal to queue every local function call. The 2026-09-21 ruling draws that line: the contract applies where responsibility crosses an execution boundary, meaning a process, a unit, a host or a principal.

## 2. What is already ruled (not re-decided here)

- **Owner ruling, 2026-09-21: durable obligations, replaceable processes.** It sits in the deleted `docs/plans/harness-work-lifecycle.md`; read it with `git show fa44b981025^:docs/plans/harness-work-lifecycle.md`, section "Workflow execution".
  - The durable facts are the accepted work, each outstanding obligation, its state, its due time, and its last result or located failure. Processes, units, timers and page bodies are replaceable realizations over them.
  - Targeted wake-ups advance work. Due times live on obligations. Bounded reconciliation is the recovery backstop, never the primary driver.
  - An unperformable operation stays outstanding with a typed cause and an owner. It is never skipped.
  - Clause 5 is the contract: *durable, versioned handoff of responsibility across execution boundaries*, with inbound bounded admission and durable acceptance, conditional processing and attributable settlement. Three separate facts are kept apart: transport acknowledgment, execution-resource release, and historical retention. The contract is built by composing existing authorities, not by inventing new ones.
  - Clause 6: the §3b conformance question lands once the authority exists, with its first real consumer.
- **Owner ruling, 2026-09-22: the belt demoted.** The belt only delivers wake-ups and does bounded anti-entropy repair. A wake-up is a hint, never a fact, so every consumer must be idempotent and version-checked. An event recomputes the smallest affected projection and creates only the next eligible obligation.
- **`docs/plans/demand-engine-program.md` §2.5.** This is the same concept in-process: one live ready queue, a completion that immediately re-evaluates its direct dependents, one dispatch policy (stable order by identity, admit whatever can obtain a lease), and typed blocked states.
- **`docs/plans/ensure-closure-design.md`.** A retry law is declared per implementation, and admission refuses automatic retry without one. Idempotence belongs to each implementation, not to "ensure" as a category.
- **The 2026-10-03 manual-resume ruling**, as `gunbc.roadmap.roadmap_belt_obligation` cites it. A Failed or Refused segment stops the line, and an operator resumes it.

This document adds no rule above these. It supplies the model they call for and the realization that makes it true on a host.

## 3. Specimens: why now

All of these come from the srv1 lab, `srv1-lab-codex-feedback-v0`, on 2026-10-12, unless noted. Each is a gap in the queue contract, not in the work itself.

| Specimen | Which question failed | The gap |
|---|---|---|
| A deploy changed what was owed for every attempt the lab had already settled: #13764 made an absent capture owed, not yielded. Nothing ran until an operator started discovery by hand. | What makes it advance? | A revision change is not an event. Outstanding work is derived from evidence under the running code, so new code can make work owed with no new evidence. |
| `install-owned` was captured at 18:09 and then stopped. Verify was owed, but nothing queued it. | What makes it advance? | Completing a step does not wake the next eligible step. In demand-engine terms, a completion does not re-evaluate dependents. |
| Discovery queues at most 8 attempts per pass (`belt_discovery_attempt_bound`), taken as a prefix in observation order with no cursor. The newest attempt was left out of the first pass. | What makes it advance? | No fairness rule. An item can starve behind older items that stay owed. |
| `install-owned`'s submission projects as **Refused**: "the belt has not selected a verification window for this head yet". Under the manual-resume ruling, Refused stops the line. | What happens when it fails? | "Not yet" is typed as "no". A waiting state recorded as a refusal is never revisited. |
| The lab's publication helper unit has failed with `200/CHDIR` since 2026-10-11 04:10. Nothing on the page shows it. | How does anyone know? | A consumer that cannot start has no standing on any queue. |
| One discovery pass costs about 6 CPU-minutes and 13 GB, because it loads the whole corpus to read every attempt. | (cost) | Anti-entropy scales with history, not with outstanding work. |
| Earlier: srv2, 2026-10-03, one slow capture held every belt tick at its 90-minute bound for ten hours. Also the rung drops `publication_path_unbounded_retrigger` and `belt_liveness_publication_answers_unconsumed`. | All four | Fixed piecemeal, by vocabulary local to each case. |

## 4. One concept at two scales

The demand engine's ready queue (`v2.std.demand_engine`) and a service's work queue answer the same question at different scales. Each holds a set of keyed demands, some ready and some blocked on typed causes. A completion re-evaluates dependents, and a dispatch policy admits ready work up to capacity. DESIGN §2's horizontal rule applies: model it once and derive every use.

What the durable scale adds is exactly what a process boundary takes away:

- **Acceptance outlives the process.** A handed-off item survives the producer, the consumer and the host restarting.
- **Wake-ups can be lost, duplicated or reordered**, so a wake-up is a hint, and recovery re-derives from durable evidence.
- **Time is data.** Due times and retry-after are fields on an item, not the period of a loop.
- **Silence must be observable**, because no caller is waiting on a return value.

So the proposal is one vocabulary of standings and dispatch, with two realizations: in-process (the demand engine) and durable (§6). Durability adds fields and laws. It does not add a second algebra.

## 5. The model

### 5.1 The work item, and why outstanding is derived

An item is an **obligation**: a kind plus a subject. `gunbc.roadmap.roadmap_belt_obligation` already models this for the belt (`BeltAttemptObligationKind` over an attempt, and instance obligations over an instance).

Whether an item is outstanding is **derived** from the subject's durable evidence (receipts, records), never stored separately (`belt_attempt_outstanding`; the bold-bee-114 Q1 ruling). That stays. It has one consequence the belt missed: the derivation runs **under a code revision**, so a revision change can make work owed without any new evidence. A revision change is therefore an event (§5.3).

The item's key is its identity: kind plus subject, content-addressed where the subject is. Every wake-up, queue entry, hold and receipt is keyed by it.

### 5.2 Standing: a closed set, and "not yet" is not "no"

Every item, at every moment, has exactly one standing. The set is closed, so there is nowhere for an item to be silently stuck:

| Standing | Meaning | Leaves by |
|---|---|---|
| `Ready` | Owed, and every prerequisite holds | a worker claiming it |
| `Waiting { on }` | Owed, but blocked on a named future event or a due time | that event or time |
| `Running { hold }` | Claimed by a worker that holds the exclusive hold | the worker settling it, or its hold being recovered |
| `RetryDue { at, cause, attempt }` | A transient failure under a declared retry law | the due time |
| `Stopped { cause, owner }` | The line stops: a refusal that needs a decision, or retries exhausted | an owner's action, recorded |
| `Settled` | Nothing outstanding | (terminal) |

- **`Waiting` vs `Stopped` is the distinction the `install-owned` specimen lacked.** A cause meaning "not yet" (no verification window selected, no capacity, no upstream answer) must name the event it waits on, and that event must be one of the wake-up sources in §5.3. A `Stopped` cause must name an owner. A cause that can name neither is a modeling gap, and the model refuses to construct it.
- **`Stopped` is the dead letter.** The manual-resume ruling keeps it operator-owned. What changes is that it is listed, attributed and counted on the queue, never silent.
- The standing names line up with the demand engine's availability states (waiting, ready, lease pending, running, suspended, refused, blocked), and the model shares them rather than re-coining them (§8, open question 2).

### 5.3 Wake-ups: hints from a closed set of sources

A wake-up carries a key and nothing else. Any number of wake-ups for one key collapse into one queue entry, so duplicates are harmless. A lost wake-up is caught by resync (§5.7). A wake-up that arrives out of order is harmless, because the worker re-derives the item's state from evidence.

The sources are a closed set. Each specimen in §3 is a missing member:

| Source | Fires when | Belt today |
|---|---|---|
| `Arrival` | New evidence lands: a worker exit, a submission, an upstream answer | exit spool and answer spool (present) |
| `Completion` | A step on the same item settles; re-evaluate the item and its dependents | **missing** |
| `RevisionChange` | The code that derives outstanding work changed: a deploy | **missing** |
| `DueTime` | An item's `Waiting` or `RetryDue` time arrives | **missing**: the belt has no timer |
| `OperatorAction` | An owner resumes, re-drives or cancels | present (`belt_operator_follow_up_kinds`) |
| `Resync` | The bounded anti-entropy pass finds an owed item with no queue entry | present but unfair (discovery) |

Each source is a declared row. A service declares which sources its obligation kinds depend on, and the conformance review asks whether every state change that can make work owed has a source.

### 5.4 The queue

- **One entry per key.** Enqueue is idempotent, so a second enqueue for a key already queued changes nothing.
- **Order: due time, then enqueue time.** This is first-in-first-out among items that are due. Ordering by subject age or by observation order is not allowed.
- **Concurrency is bounded by declared capacity**, using the demand engine's dispatch rule: admit the next ready item whose worker can obtain its lease, and leave the others ready. The bound limits work running, never work discovered.
- **The claim is an exclusive hold** (`std.durable_exclusive_hold`, fenced by `std.temporal_effect` `LeaseEpoch`). A hold is recovered only when its holder is observed gone (`durable_hold_recovery_assess`), never by age. The belt's "unit name is the hold" is one realization of this (§6).

**Progress law.** With one entry per key, first-in-first-out order and bounded step time, an item that becomes `Ready` is claimed within (entries ahead of it ÷ concurrency) × step bound. Starvation becomes a property the model can state and a witness can check, instead of a live incident.

### 5.5 The step, and its typed settlement

A worker does the same thing every time:

1. **Claim** the item under the hold.
2. **Re-derive** what is outstanding from evidence.
3. **Run the next eligible obligation** as an ensure, with that obligation's own retry law.
4. **Persist the result** at the moment it is produced (the 2026-09-21 ruling, clause 1).
5. **Settle** with exactly one outcome:

| Outcome | Then |
|---|---|
| `Advanced` | Enqueue a `Completion` wake-up for the same key and its dependents |
| `Settled` | Done |
| `Waiting { on }` | Park until that event or time |
| `RetryAfter { at, cause }` | Only when the obligation's retry law admits a retry; the law declares the backoff and the attempt bound. Without a law, this becomes `Stopped`. |
| `Stopped { cause, owner }` | The line stops, visibly |
| `Transferred { to, ref }` | Responsibility moves durably to another queue: for example, the belt hands a publication request to the publication helper |

The three facts from the 2026-09-21 ruling stay separate:

- delivery acknowledgment (the wake-up or entry is consumed);
- resource release (the hold and the unit end);
- retention (receipts and history stay).

A settled step may release its hold. It never erases what recovery would need.

### 5.6 Transfer between queues

A handoff to another queue is itself an item on the receiving queue: durable acceptance first, then the sender records `Transferred` with the receiver's reference. Today the publication spools do this implicitly, through files a separate principal answers. The model makes the transfer a typed settlement, so the sender's view and the receiver's view can be joined. Any request with no answer and no standing becomes visible.

### 5.7 Resync: anti-entropy that is cheap and fair

Resync exists to catch lost wake-ups, not to drive work. Its laws:

- **It enqueues keys only. It never does the work.** This is unchanged from today's discovery.
- **It walks a cursor.** Each pass continues where the last one stopped, and wraps around. That makes it round-robin over the outstanding set, not a prefix of it.
- **It reads an index of subjects that are not settled**, not the whole corpus. Its cost scales with outstanding work, not with history. The index is a projection of the same evidence, maintained by the same settlements.
- **What it finds is counted.** An owed item with no queue entry is a lost wake-up, recorded with its source class. A nonzero count is the instrument that says the event sources (§5.3) are incomplete. Today that knowledge exists only when someone notices.

### 5.8 Observability: a queue reports its own health

Each queue publishes a standing through `std.observation`, and the served page renders it:

- depth by standing (ready, waiting, running, retry-due, stopped);
- the age of the oldest owed item;
- stopped items, each with its cause and owner;
- lost wake-ups found by resync, by source;
- consumer start failures.

**A consumer that cannot start is a `Stopped` standing on its queue**, with the unit failure as the cause. The publication helper's day of `200/CHDIR` would have been one red line on the page. `gunbc.roadmap_launch_deployment_observe` `event_backlog_judge` (`StaleBacklog`) is the existing seed of this and becomes one reading of it.

## 6. Where each fact lives (DESIGN §3: interface, realization, policy)

**Interface: a new home, `std.durable_handoff`.** It holds the types and pure folds above: item and key, standing, wake-up sources, entry, settlement outcome, dispatch order, resync cursor, and the queue-health reading. It is agnostic: it names no host, unit or service.

It composes existing authorities rather than restating them (§3b, conformance):

| Need | Existing authority |
|---|---|
| Hold and recovery | `std.durable_exclusive_hold`, over `std.durable_compare_and_set` `CasExpectation` |
| Fencing | `std.temporal_effect` `HeldLease`, `LeaseEpoch` |
| Step stall bound | `std.temporal_effect` `stall_budget_verdict` |
| Retry safety | `std.effects` `IdempotencyEvidence`; per-implementation retry law (ensure closure) |
| Convergence | `std.upsert_decision`, `std.realization_reconcile`, `std.goal_assessment` |
| Reporting | `std.observation` (`ProducerLiveness`, `observation_watchdog`) |
| Standings and dispatch | shared with `v2.std.demand_engine` (§8, open question 2) |

**Host realization: one module, shared by every host service.** It generalizes what the belt already built:

- **Entries:** atomic, create-only directory entries in a spool, as `gunbc.roadmap.roadmap_attempt_exit_spool` does today. Its stages (`AttemptExitSpoolStage`) and its claim and settle operations (`belt_exit_entries_claim_at`, `belt_exit_entries_settle_at`, `belt_exit_running_reconcile_at`) are the seed.
- **Wake-up delivery:** systemd path units, level-triggered (`DirectoryNotEmpty`, as the belt exit path unit already is), so a wake-up is never lost between a write and the trigger.
- **Workers:** transient units whose name is the hold, with a declared runtime bound (`belt_tick_run_bound` today).
- **Time:** one timer per queue for due times and resync. Today the belt has none.
- **Placement:** workers run in a slice admitted for the work, not inside a serving slice (the `convergence_oneshot_in_a_serving_slice` lesson, #13795).

A fabric-store realization can follow later behind the same interface.

**Policy: per service.** Each service owns:

- its obligation kinds and their eligibility order;
- each kind's retry law;
- its wake-up sources;
- its capacity;
- its resync bound and cadence.

These are rows, not code paths. A service that needs a new behavior adds a row or a policy value, and the queue does not fork.

## 7. Disposition of existing vocabulary

The survey found each queue concept spelled several ways across services. Under DESIGN §3 these are nicknames, to be consolidated as each service migrates (§9). They are not to be extended in the meantime.

| Concept | Home after this change | Spellings today |
|---|---|---|
| Exclusive hold | `std.durable_exclusive_hold` | unit name, flock, compute CAS slot, GitHub Actions concurrency group, `WorkClaim` (`gunbc.auth.approval_broker`) |
| Dedup key | the item key | mkdir name, `publication_spool_key`, content-hash event id, `idempotency_key`, `DemandIdentity` |
| Orphan recovery | `durable_hold_recovery_assess` | running-to-queued reconcile, `RecoveryTrigger` (`gunbc.compute.attempt_lifecycle`, which uses a deadline, i.e. recovery by age), `recovery_owed`, `TakeoverAdmission` |
| Dead letter | `Stopped { cause, owner }` | the spool's `refused` stage, the publication refused directory, cell quarantine, "stops the line", `WorkAbandoned` |
| Retry | `RetryDue` under a retry law | `WorkspaceConsumerRetry`, `AcquisitionLost`, `HarnessSeatPending`, `FailureClass.Retryable`, `std.types` `RetryCount` (zero consumers) |
| Deadline / due | item `at` | `belt_tick_run_bound`, `launch_deadline`, `execute_by`, `convergence_unit_start_timeout` |
| Anti-entropy | `Resync` | belt discovery, the retired fleet-converge timer, the disk-reclaim timer, the workspace pass, `ReconcileTick` |

**`gunbc.workflow.types` is deleted in M0.**
- It holds a full parallel queue vocabulary (`SignalType`, `Signal { idempotency_key }`, `FailureClass`, `StageOutcome { retry_budget_remaining, next_attempt_at }`), and nothing imports it.
- Its only other mention is a prose line in `gunbc.plans.host_effect_orchestration`, which already lists `RetryCount` as dead vocabulary to clear.
- A dormant module like this is an attractor. It would pull this work toward its names, so it goes first (the delete-first rule of the replacement migration doctrine).

## 8. The guarantee ladder for this domain

| Error class | Rung today | Ceiling | Next-rung trigger |
|---|---|---|---|
| Lost wake-up | below the ladder (silent; §3 specimens) | structurally guaranteed: the evidence write and its entry are one durable append | M1 counts resync findings (mechanically preventable); an outbox-style append makes it structural |
| Starvation | below the ladder (discovery prefix) | structurally guaranteed: the progress law of §5.4 holds by construction | M1's ordered queue with one entry per key |
| "Not yet" typed as "no" | below the ladder (`install-owned`) | structurally impossible: `Waiting` requires a named source and `Stopped` requires an owner | M0's closed standing |
| Silent consumer failure | below the ladder (publication helper) | structurally guaranteed: a consumer is a queue member with a standing | M2 |
| Two workers on one item | mitigatable (unit name) | structurally guaranteed: fenced claims | M1, holds through `std.durable_exclusive_hold` |
| Automatic retry of a non-idempotent step | mechanically preventable (ensure admission) | structurally impossible: `RetryAfter` requires a retry law | M0 |
| Work in a slice whose bound belongs to others | mechanically preventable (#13795) | structurally guaranteed: worker placement is admitted | M1's host realization admits worker slices |

## 9. Milestones

**M0: the model, with no consumer change.**
- `std.durable_handoff` with every law of §5, each witnessed on a planted RED:
  - two enqueues collapse into one entry;
  - first-in-first-out order holds when an older item stays owed;
  - a claim against a stale fence refuses;
  - `Waiting` without a source and `Stopped` without an owner cannot be built;
  - `RetryAfter` without a retry law refuses;
  - the resync cursor advances past a prefix that stays owed.
- Delete `gunbc.workflow.types`.
- Decide the shared vocabulary with `v2.std.demand_engine` (open question 2).

**M1: the belt is the first consumer, on the lab first.**
- `RevisionChange` fires on deploy; `Completion` fires after every step; a timer drives `DueTime` and resync.
- Resync walks a cursor over an index of unsettled subjects, replacing `belt_discovery_attempt_bound` prefix discovery.
- The verification-window "not yet" becomes `Waiting`.
- Queue health appears on the served page.
- **Acceptance:**
  - N lab dispatches run end to end with zero hand steps;
  - the 2026-09-21 ruling's clause 4 demonstrations pass: one attempt reaches its PR while another is stopped, and killing the controller mid-obligation resumes without user action;
  - resync finds zero lost wake-ups across those N runs, which is the evidence that the event sources are complete.

**M2: the publication helper becomes a queue.** The belt settles with `Transferred`, and the helper's start failure surfaces as `Stopped`. Both publication rung drops are retired by this milestone, by their own triggers.

**M3: the conformance row lands (§10), and the remaining services migrate one at a time.**
- Order of migration: compute recovery (`gunbc.compute.attempt_lifecycle`), approved-action execution (`gunbc.auth.approval_broker` `ExecutionObligation`), the workspace consumer (`gunbc.workspace_request_consumer`), runner cells, and fleet converge.
- Each migration is a replacement migration under the doctrine:
  - delete the service's own hold, retry and anti-entropy spellings;
  - fix forward onto the shared model;
  - census by deletion.

## 10. The proposed §3b conformance row

The row is added to `gunbc.design_argument` `conformance_domains` once M0's home exists and M1 has consumed it. That follows §3b's rule that a row names homes that exist, and clause 6 of the 2026-09-21 ruling. Adding it needs operator sign-off, because it changes the reviewer roster.

### Durable handoff / work queues

**Owns.** Work whose responsibility crosses an execution boundary: how it is accepted, keyed, woken, claimed, advanced, retried, stopped, transferred, resynced and reported.

**Boundaries.**
- Leasing owns the hold primitive; this domain uses it.
- Fabric and compute own allocation.
- §3d selection precedes: a queue advances a chosen realization and never chooses one.
- Goal assessment checks the state an item reaches.

**Existing authorities.** `std.durable_handoff` (new); `std.durable_exclusive_hold`; `std.temporal_effect` `LeaseEpoch`; `std.effects` `IdempotencyEvidence`; `std.observation`; and the host realization module.

**Reviewer question.** Does this boundary hand off responsibility durably?
- Bounded admission and durable acceptance.
- A wake-up source for every state change that can make work owed.
- A closed standing for every item, in which "not yet" names its event and "stopped" names its owner.
- Fair, bounded dispatch.
- Retries only under a declared law.
- Queue health that someone can see.

**Tells.**
- A loop with a sleep, or a tick that does work.
- A scan bounded by count, with no cursor.
- A refusal that really means "not yet".
- A failure with no owner.
- A consumer whose failure no page shows.
- Work that starts only when an operator runs something by hand.
- A retry with no retry law.
- Recovery by age.
- A new spelling of hold, dedup, dead letter or due time.

## 11. Open questions for the owner

1. **Home names.** The ruling named `gunbc.workflow.durable_handoff`. This proposes `std.durable_handoff` for the agnostic model plus a `gunbc` module for the host realization, following DESIGN §3's interface/realization split. Is that acceptable, or should the ruled name stand?
2. **Shared vocabulary with the demand engine.** Factor the standings and dispatch rule into one std home both import (M0), or let `std.durable_handoff` land first and have the demand engine adopt it at its M3a cut?
3. **Automatic retry.** The manual-resume ruling stops the line on Failed or Refused. This proposal admits `RetryAfter` only for transient causes under a declared retry law, with exhaustion going to `Stopped`. Does that refine the ruling, or contradict it?
4. **Scope edge.** Which boundaries are deliberately not queues? The ruling excludes local calls. Should synchronous request/response across a process (serving turns, `gunbc.serving.caller_retry_contract`) be in scope, given its stance that there is no server-side queue?
