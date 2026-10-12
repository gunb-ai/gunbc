# Durable work handoff: the event-queue conformance pattern

> **Status: proposal for owner review, 2026-10-12, revised the same day with the owner's decisions (§11), the consolidation map (§7), and the interface agreed with the flex-inference lane (§5.9). The consolidation waves (§7) land before any new module. It proposes the model the 2026-09-21 owner ruling named and never landed (`gunbc.workflow.durable_handoff`), the host realization the services share, a §3b conformance row, and the order in which services move onto it.**

## 1. The question

Some work outlives the request or the process that created it: an attempt that needs capturing, a candidate that needs verifying, a publication answer that needs consuming, a compute reservation that needs recovering. Every such boundary has to answer four things:

1. **Who holds responsibility for the work right now?**
2. **What makes it advance?**
3. **What happens when advancing fails?**
4. **How does anyone know it has silently stopped?**

Today each service answers these with its own mechanism and its own vocabulary (§7). The belt is the most complete, and on 2026-10-12 it failed all four questions in one afternoon (§3). This document proposes one model, realized once per execution substrate and inhabited by every service, so the four answers are the same everywhere. It also proposes a conformance row so that a change which hands work across a boundary is reviewed against that model.

This is not a proposal to queue every local function call. The 2026-09-21 ruling draws that line: the contract applies where responsibility crosses an execution boundary, meaning a process, a unit, a host or a principal. Nor does a request being synchronous put it out of scope (owner, 2026-10-12). Most requests are synchronous at their interface. Whether one is answered in-process or handed off durably and awaited is a realization choice this corpus owns. The contract governs that realization, and the caller's interface stays synchronous.

**The queue decides nothing.** Every choice about the work — which item runs next, whether and when to retry, whether to wait, move or acquire supply, when to give up — is a selection under the declared objective of the demand the work serves (DESIGN §3d; §5.9). The queue holds what is owed and why it is waiting, wakes the select-ensure-observe loop, and records how each step ended. A mechanism that fixes a preference instead (oldest first, warm first, retry until success) is a policy leak (DESIGN §3).

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
- **Owner direction, 2026-10-12: every runtime decision is a selection under the demand's declared objective.** The acceptance test is the policy switch: with the work and the resources fixed, change only the end policy (for example debt payoff, where only throughput per dollar matters and delay and interruption are valued at $0, against latency-sensitive), and the decisions must change wherever there is a real tradeoff, with no code edit.

This document adds no rule above these. It supplies the model they call for and the realization that makes it true on a host.

## 3. Specimens: why now

All of these come from the srv1 lab, `srv1-lab-codex-feedback-v0`, on 2026-10-12, unless noted. Each is a gap in the queue contract, not in the work itself.

| Specimen | Which question failed | The gap |
| --- | --- | --- |
| A deploy changed what was owed for every attempt the lab had already settled: #13764 made an absent capture owed, not yielded. Nothing ran until an operator started discovery by hand. | What makes it advance? | A revision change is not an event. Outstanding work is derived from evidence under the running code, so new code can make work owed with no new evidence. |
| `install-owned` was captured at 18:09 and then stopped. Verify was owed, but nothing queued it. | What makes it advance? | Completing a step does not wake the next eligible step. In demand-engine terms, a completion does not re-evaluate dependents. |
| Discovery queues at most 8 attempts per pass (`belt_discovery_attempt_bound`), taken as a prefix in observation order with no cursor. The newest attempt was left out of the first pass. | What makes it advance? | No fairness rule. An item can starve behind older items that stay owed. |
| `install-owned`'s submission projects as **Refused**: "the belt has not selected a verification window for this head yet". Under the manual-resume ruling, Refused stops the line. | What happens when it fails? | "Not yet" is typed as "no". A waiting state recorded as a refusal is never revisited. The corpus has ten such places (§7, wave C). |
| The lab's publication helper unit has failed with `200/CHDIR` since 2026-10-11 04:10. Nothing on the page shows it. | How does anyone know? | A consumer that cannot start has no standing on any queue: `gunbc.live_deploy.unit_standing` covers the belt's units but omits the publication path and helper. |
| One discovery pass costs about 6 CPU-minutes and 13 GB, because it loads the whole corpus to read every attempt. | (cost) | Anti-entropy scales with history, not with outstanding work. |
| Earlier: srv2, 2026-10-03, one slow capture held every belt tick at its 90-minute bound for ten hours. Also the rung drops `publication_path_unbounded_retrigger` and `belt_liveness_publication_answers_unconsumed`. | All four | Fixed piecemeal, by vocabulary local to each case. |

## 4. One concept at two scales

The demand engine's ready queue (`v2.std.demand_engine`) and a service's work queue answer the same question at different scales. Each holds a set of keyed demands, some ready and some blocked on typed causes. A completion re-evaluates dependents, and a dispatch rule admits ready work up to the leases granted. DESIGN §2's horizontal rule applies: model it once and derive every use.

What the durable scale adds is exactly what a process boundary takes away:

- **Acceptance outlives the process.** A handed-off item survives the producer, the consumer and the host restarting.
- **Wake-ups can be lost, duplicated or reordered**, so a wake-up is a hint, and recovery re-derives from durable evidence.
- **Time is data.** Due times and retry-after are fields on an item, not the period of a loop.
- **Silence must be observable**, because no caller is waiting on a return value.

So the proposal is one vocabulary of standings and dispatch, `std.work_queue`, factored out of `v2.std.demand_engine` in M0 (owner, 2026-10-12) and imported by both the demand engine and `std.durable_handoff`. Durability adds fields and laws. It does not add a second algebra.

## 5. The model

### 5.1 The work item, and why outstanding is derived

An item is an **obligation**: a kind plus a subject. `gunbc.roadmap.roadmap_belt_obligation` already models this for the belt (`BeltAttemptObligationKind` over an attempt, and instance obligations over an instance). Every item also names the demand it serves (a `product.fabric.demand` `Demand` identity, §5.9); the queue never stores a valuation.

Whether an item is outstanding is **derived** from the subject's durable evidence (receipts, records), never stored separately (`belt_attempt_outstanding`; the bold-bee-114 Q1 ruling). That stays. It has one consequence the belt missed: the derivation runs **under a code revision**, so a revision change can make work owed without any new evidence. A revision change is therefore an event (§5.3).

The item's key is its identity: kind plus subject, content-addressed where the subject is. Every wake-up, queue entry, hold and receipt is keyed by it.

### 5.2 Standing: a closed set, and "not yet" is not "no"

Every item, at every moment, has exactly one standing. The set is closed, so there is nowhere for an item to be silently stuck:

| Standing | Meaning | Leaves by |
| --- | --- | --- |
| `Ready` | Owed, and every prerequisite holds | a worker claiming it, in the selected order (§5.4) |
| `Waiting { on }` | Owed, but blocked on a named future event, a due time, or an unread observation | that event, time or reading |
| `Running { hold }` | Claimed by a worker that holds the exclusive hold | the worker settling it, or its hold being recovered on observed holder death |
| `RetryDue { at, cause, attempt }` | A failure whose retry law admits another attempt no earlier than `at` | the due time, then the retry decision (§5.9) |
| `Stopped { cause, owner }` | The line stops: a refusal that needs a decision | an owner's action, recorded |
| `Finished { Completed }` | Nothing outstanding: the work is done | (terminal) |
| `Finished { Cancelled { by } }` | Nothing outstanding: an attributable cancellation ended it | (terminal) |
| `Finished { Exhausted { cause } }` | Nothing outstanding: the retry law's whole-history bound is spent | (terminal) |

- **`Waiting` vs `Stopped` is the distinction the `install-owned` specimen lacked.** A cause meaning "not yet" (no verification window selected, no capacity, no upstream answer, an observation not yet read) must name the event it waits on, and that event must be one of the wake-up sources in §5.3. A `Stopped` cause must name an owner. A cause that can name neither is a modeling gap, and the model refuses to construct it.
- **`Stopped` is the dead letter an owner can resume; `Finished { Exhausted }` is the one nobody resumes** (it is what `gunbc.serving.caller_backlog` `WorkRecoveryBoundExceeded`, in #13789, records). Both are listed, attributed and counted on the queue, never silent.
- **Why `Finished` and not `Settled`.** "Settle" already means four different things in the corpus (`PoolSettled`, `CellSettlement`, `SessionSettlement`, the belt's running-to-done move), and `v2.std.demand_engine` `demand_state_is_settled` counts a refusal as settled. Reusing the word would be a meaning fork (DESIGN §3).

**The arm-by-arm map from `v2.std.demand_engine` `DemandState`**, which M0 moves into `std.work_queue`:

| `DemandState` arm | Becomes | Why |
| --- | --- | --- |
| `DemandWaiting` | `Waiting { on: prerequisite }` | the same fact, with its event named |
| `DemandReady` | `Ready` | the same |
| `DemandRunning` | `Running { hold }` | the same, with the hold made explicit |
| `DemandAvailable { V }` | the engine keeps `Available { V }` as its own value-carrying wrapper; its standing is `Finished { Completed }` | the standing is value-free; releasing dependents is the engine's consequence of `Completed`, and the durable scale releases nothing in-process |
| `DemandRefused` | `Stopped { cause, owner }` | a refusal gains the cause and owner it lacks today |
| `DemandBlocked { cause }` | split: capacity and lookup causes become `Waiting { on }`; cycle, missing edge, key conflict and refused prerequisite become `Stopped { cause, owner }` | `DemandBlockedCause` mixes waits with permanent blocks today |
| `DemandLeasePending`, `DemandSuspended`, `DemandCommitPending` | deleted in M0 | no code constructs them; they return as `Waiting { on }` causes when the engine's M3a cut needs them |

### 5.3 Wake-ups: hints from a closed set of sources

A wake-up carries a key and nothing else. Any number of wake-ups for one key collapse into one queue entry, so duplicates are harmless. A lost wake-up is caught by resync (§5.7). A wake-up that arrives out of order is harmless, because the worker re-derives the item's state from evidence.

The sources are a closed set. Each specimen in §3 is a missing member:

| Source | Fires when | Belt today |
| --- | --- | --- |
| `Arrival` | New evidence lands: a worker exit, a submission, an upstream answer | exit spool (present); a publication answer starts discovery instead of enqueueing its attempt (wave D re-points it) |
| `Completion` | A step on the same item settles; re-evaluate the item and its dependents | **missing** |
| `RevisionChange` | The code that derives outstanding work changed: a deploy | **missing** |
| `DueTime` | An item's `Waiting` or `RetryDue` time arrives | **missing**: the belt has no timer, and no live due time exists anywhere in the corpus |
| `OperatorAction` | An owner resumes, re-drives or cancels | present (`belt_operator_follow_up_kinds`) |
| `Resync` | The bounded anti-entropy pass finds an owed item with no queue entry | present but unfair (discovery) |

Each source is a declared row. A service declares which sources its obligation kinds depend on, and the conformance review asks whether every state change that can make work owed has a source.

### 5.4 The queue

- **One entry per key, created only once.** Enqueue is idempotent and create-only: a second enqueue for a key already queued changes nothing. (The publication spool writes its request by overwrite today; wave D moves it to the same law.)
- **Order is a selection, not a rule.** Which `Ready` items take the next admissible leases is decision (a) of §5.9, made under the demand's objective. The queue supplies the candidates and the evidence (age, attempt history, estimated cost, estimated reconstruction) and never values them. A deterministic identity tie-break applies after the selected order. That makes `v2.std.demand_engine`'s present "stable order by identity" the case where the policy is indifferent, not a second rule beside the selection.
- **Starvation is a constraint the order must satisfy, not the order itself.** Every demand objective declares a maximum wait or explicitly declares none (`product.fabric.demand` `AdmissionTerms`, §5.9). A selection that would exceed a declared maximum wait is infeasible.
- **Concurrency is the leases granted, not a per-service number.** How much capacity to hold is supply selection (decision c, owned by the flex-inference lane); the queue reads how many leases it has.
- **The claim is an exclusive hold** (`std.durable_exclusive_hold`, fenced by `std.durable_compare_and_set` `CasGeneration`, the corpus's one fencing token after wave B). A hold is recovered only when its holder is observed gone (`durable_hold_recovery_assess`), never by age. The belt's "unit name is the hold" is one realization of this (§6).

**Progress law.** With one entry per key, a feasible selected order (every declared maximum wait respected) and bounded step time, an item that becomes `Ready` is claimed within its demand's declared maximum wait. A demand that declares no maximum wait has declared that waiting costs it nothing; the queue still reconsiders it at every capacity event, so it is never forgotten, only outranked.

### 5.5 The step, and its typed settlement

A worker does the same thing every time:

1. **Claim** the item under the hold.
2. **Re-derive** what is outstanding from evidence.
3. **Run the next eligible obligation** as an ensure, with that obligation's own retry law. Where the obligation involves a choice, the loop is select, then ensure, then independent observation (DESIGN §3d): `std.decision` makes the choice, and the queue only wakes the loop and records its settlement. The queue never selects.
4. **Persist the result** at the moment it is produced (the 2026-09-21 ruling, clause 1).
5. **Settle** with exactly one outcome:

| Outcome | Then |
| --- | --- |
| `Advanced` | Enqueue a `Completion` wake-up for the same key and its dependents |
| `Finished { Completed }` / `Finished { Cancelled { by } }` / `Finished { Exhausted { cause } }` | Done; a cancellation names who cancelled, an exhaustion names its cause |
| `Waiting { on }` | Park until that event, time or reading |
| `RetryAfter { at, cause }` | Only when the obligation's retry law admits a retry; `at` is the law's earliest time. Without a law, this becomes `Stopped`. Whether to retry then, retry elsewhere or stop is decision (b) of §5.9. |
| `Stopped { cause, owner }` | The line stops, visibly |
| `Transferred { to, ref }` | Responsibility moves durably to another queue: for example, the belt hands a publication request to the publication helper |

The three facts from the 2026-09-21 ruling stay separate:

- delivery acknowledgment (the wake-up or entry is consumed);
- resource release (the hold and the unit end);
- retention (receipts and history stay).

A settled step may release its hold. It never erases what recovery would need.

**The retry law.** A retry law is declared per obligation implementation (ensure closure). It decides only whether a retry is **admissible** and the **earliest** time one may run; which retry to take is a selection under the demand's policy (§5.9). It has one shape everywhere, taken from `gunbc.serving.caller_backlog` (#13789, not yet on main) and `gunbc.serving.caller_retry_contract`:

- The earliest time grows exponentially with jitter, and an upstream delay hint (a provider's retry-after) raises it but never lowers it.
- The bound runs over the item's whole history. A resume, a restart or a new endpoint does not reset it.
- Only outcomes of the real work count toward the bound. A succeeding probe never resets a failing series of real requests.
- Exhaustion is a persisted `Finished { Exhausted }`, and cancellation a persisted `Finished { Cancelled }`. Neither is ever inferred from silence.

**An unread observation is unknown, never empty.** A standing is derived from evidence. When the evidence cannot be read (an unreadable record, an inventory not observed), the item is not `Ready` and not `Finished`: it is `Waiting` on that observation, or `Stopped` if the unreadability is itself the defect. An empty fleet and an unobserved fleet are different facts.

### 5.6 Transfer between queues

A handoff to another queue is itself an item on the receiving queue: durable acceptance first, then the sender records `Transferred` with the receiver's reference. Today the publication spools do this implicitly, through files a separate principal answers. The corpus has exactly one typed two-phase acceptance, `gunbc.fleet_observation.capture_chunk` `LocalCommitStanding` / `DownstreamAcceptanceStanding`; wave D moves it into std as the transfer pair. The model makes the transfer a typed settlement, so the sender's view and the receiver's view can be joined. Any request with no answer and no standing becomes visible.

### 5.7 Resync: anti-entropy that is cheap and fair

Resync exists to catch lost wake-ups, not to drive work. Its laws:

- **It enqueues keys only. It never does the work.** This is unchanged from today's discovery.
- **It walks a cursor.** Each pass continues where the last one stopped, and wraps around. That makes it round-robin over the outstanding set, not a prefix of it. Fair discovery is a queue invariant; fair dispatch is the policy's (§5.4).
- **It reads an index of subjects that are not finished**, not the whole corpus. Its cost scales with outstanding work, not with history. The index is a projection of the same evidence, maintained by the same settlements.
- **What it finds is counted.** An owed item with no queue entry is a lost wake-up, recorded with its source class. A nonzero count is the instrument that says the event sources (§5.3) are incomplete. Today that knowledge exists only when someone notices.

### 5.8 Observability: a queue reports its own health

Each queue publishes one health reading, and the served page renders it:

- depth by standing (ready, waiting, running, retry-due, stopped);
- the age of the oldest owed item;
- stopped and exhausted items, each with its cause and owner;
- lost wake-ups found by resync, by source;
- consumer start failures.

**A consumer that cannot start is a `Stopped` standing on its queue**, with the unit failure as the cause. The publication helper's day of `200/CHDIR` would have been one red line on the page. Two existing readings become this one (wave D): `gunbc.roadmap_launch_deployment_observe` `event_backlog_judge` (`StaleBacklog`, today the exit spool only) and `gunbc.live_deploy.unit_standing`'s unit judgments, extended to every queue's consumer units.

### 5.9 Decisions the queue delegates: the agreed interface

Agreed with the flex-inference lane (deep-otter-797), which owns the objective model, on 2026-10-12:

- **Binding.** Every queue item names its `product.fabric.demand` `Demand` identity. The queue never stores a valuation.
- **The objective.** `Demand.admission` (`AdmissionTerms`) carries `objective: DemandObjective { profile, valuations }`. The profile is a named declaration (debt payoff, latency-sensitive). Each valuation prices one `std.decision` objective term in one unit of account: money; delay as money per second; interruption as money per event; accepted work as money per unit. A zero valuation is present, never dropped, and a missing term refuses as under-modeled.
- **Hard limits are not copied.** Budget caps stay in `BuyTerms`, and an optional maximum wait sits on `AdmissionTerms`. Each decision projects them into `std.decision` hard constraints when it builds its subject.
- **One entry point.** `std.decision` `select_realization`, unchanged, with a new `SelectionPolicy` arm, `ValuedObjective { identity, valuations }`. It runs after hard constraints and the Pareto front, picks the minimum valued sum among the survivors with the identity tie-break last, and never ranks on price before Pareto, so a dominated candidate never wins. The receipt's policy authority is the profile. Until the arm lands, decisions call with the existing policies and receive `SelectionNeedsPolicy`: there is no stub.
- **Who owns which decision.** The queue owns three decision subjects, each consuming the demand's objective through that entry point: (a) dispatch order among `Ready` items; (b) retry: retry at the law's earliest time or later, retry elsewhere (a re-selection), or stop; (d) expiry: keep waiting or cancel. The flex-inference lane owns (c): wait, move or acquire supply, as `select_supply` becomes an instance of `select_realization`. Expiry cancels only through a declared deadline valuation or a declared maximum wait, never through a default the queue picks.
- **The shared acceptance witness.** Two profiles over one fixture, with the work and the resources fixed. On a planted tradeoff, decisions (a), (c) and (d) flip with the profile. A third case, where one candidate dominates on every axis, must pick that candidate under both profiles, with receipts that still differ in policy authority, so a selector that flips everything cannot pass.

## 6. Where each fact lives (DESIGN §3: interface, realization, policy)

**Interface: two std homes.**

- **`std.work_queue`**, the vocabulary shared with the demand engine: item key, standing (§5.2), the dispatch rule (admit in the selected order, identity tie-break last), and completion re-evaluation. Factored out of `v2.std.demand_engine` in M0.
- **`std.durable_handoff`**, what durability adds: durable acceptance, wake-up sources, one entry per key, due times, the retry law, typed settlement including transfer, the resync cursor, and the health reading. It is agnostic: it names no host, unit or service.

They compose existing authorities rather than restating them (§3b, conformance):

| Need | Existing authority |
| --- | --- |
| Hold and recovery | `std.durable_exclusive_hold`, over `std.durable_compare_and_set` `CasExpectation` |
| Fencing | `std.durable_compare_and_set` `CasGeneration`, the only token after wave B |
| Step stall bound | `std.temporal_effect` `stall_budget_verdict` |
| Retry safety | `std.effects` `IdempotencyEvidence`; per-implementation retry law (ensure closure) |
| Convergence | `std.upsert_decision`, `std.realization_reconcile`, `std.goal_assessment` |
| Selection | `std.decision` `select_realization`, with the `ValuedObjective` policy arm (§5.9) |
| Due times | one instant type, chosen in wave E from the five in use |

**Host realization: `gunbc.durable_handoff_host`, one module shared by every host service.** It generalizes what the belt already built:

- **Entries:** atomic, create-only directory entries in a spool, as `gunbc.roadmap.roadmap_attempt_exit_spool` does today. Its stages (`AttemptExitSpoolStage`) and its claim and settle operations (`belt_exit_entries_claim_at`, `belt_exit_entries_settle_at`, `belt_exit_running_reconcile_at`) are the seed, generalized over an entry codec in wave D.
- **Wake-up delivery:** systemd path units, level-triggered (`DirectoryNotEmpty`, as the belt exit path unit already is), so a wake-up is never lost between a write and the trigger.
- **Workers:** transient units whose name is the hold, with a declared run bound per obligation kind (wave E splits today's single `belt_tick_run_bound`).
- **Time:** one timer per queue for due times and resync. Today the belt has none.
- **Placement:** workers run in a slice admitted for the work, not inside a serving slice (the `convergence_oneshot_in_a_serving_slice` lesson, #13795).

A fabric-store realization can follow later behind the same interface.

**Policy: per service, and only what is the service's.** Each service owns its obligation kinds and their evidence-dependency order (verify needs a capture, which is a dependency, not a preference); each kind's retry law (admissibility only); its wake-up sources; its run bounds; and its resync bound and cadence. These are rows, not code paths. Everything that trades one value against another — order, retry timing, waiting, capacity, expiry — is the demand's objective (§5.9), not the service's.

## 7. Consolidation before any new module

The owner's instruction (2026-10-12): drag every existing form of these concepts together now, before anything forks. Five surveys of main inventoried them; the waves below are the result. Each wave is a replacement migration under the doctrine: delete the extra spellings, fix forward onto the home, and let what breaks name what was load-bearing. Most of these roots sit outside the required gate, so each wave names its consumers before it deletes.

| Wave | Concept | Home | What becomes of the other spellings | Owner |
| --- | --- | --- | --- | --- |
| A | dead queue vocabulary | (deletion) | `gunbc.workflow.types` (a whole parallel queue vocabulary, no importers); `approval_broker` `WorkClaim` and `TakeoverAdmission`; `host_converge` `SpawnRetryBackstop`; the orphaned fleet-converge service and timer emitters and their cadence rows | this lane |
| B | exclusive possession: 30+ spellings, four concepts | occupancy `std.durable_exclusive_hold`; liveness `DurableHoldHolderLiveness` (process observer, plus one systemd-unit observer keyed by InvocationID that merges three); stale-holder refusal `ReleaseLaw`, moved to std; the one fencing token `CasGeneration` | the belt unit name, tmux sessions, flock, exclusive-create lease files and the git-ref lease become realizations; the plan folds of `std.temporal_effect`, `ProcessLeasePlan` and `DispatchLeasePlan` merge into one decision; CAS, the event chain, the capacity pool, single-use grants, rental leases and GitHub concurrency stay separate contracts. Six recoveries by age go to the operator for rulings. | child session |
| C | work standings and step outcomes: about 60 spellings | `std.work_queue` (§5.2) | `BeltPassOutcome`, `BeltObligationEnqueue` and `PublicationHelperDisposition` are deleted onto the shared settlement; domain records (`WorkflowSegmentState`, `CandidateHandoffState`, `AttemptState`, `CallerWorkPhase` and others) keep their records and derive the shared standing; the ten places that record "not yet" as a refusal, and the four that read a refusal or an unknown as waiting or done, are fixed | this lane |
| D | durable entries, spools and queue health | `gunbc.durable_handoff_host` and the std health reading | the belt exit spool becomes the first instance and the publication spool the second, under one create-only enqueue law; `event_backlog_judge` becomes the health reading; `unit_standing` covers every queue's consumer units; the `capture_chunk` acceptance pair moves to std; the publication answer path enqueues its attempt directly | child session |
| E | time and retry | one instant type; one retry law; per-kind run bounds; `stall_budget_verdict` | `harness_placement_rounds`, `WorkspaceConsumerRetry`, the publication helper's re-ask and the spark transfer restarts re-point to the retry law; `belt_tick_run_bound` splits per kind; the stall readings re-point; `std.types` `RetryCount` is deleted with its stage0 mirror | child session |
| F | the demand's objective reaching every decision | `std.decision` (`ValuedObjective`) and `product.fabric.demand` `DemandObjective` | the global `serving_delay_valuation` placeholder and the literal zeros passed as delay valuations are deleted; `select_supply` becomes an instance of `select_realization`; fixed preferences (warm first, first compare-and-set wins, roster order, fixed retries, fixed concurrency) become selections | flex-inference lane (deep-otter-797) |

Two smaller rulings travel with the waves: `gunbc.compute.attempt_lifecycle`'s recovery deadline only decides who may try (the compare-and-set decides who wins), and `gunbc.auth.approval_broker` `ExecutionObligation` has no executor today, so it is not a migration target until one exists.

## 8. The guarantee ladder for this domain

| Error class | Rung today | Ceiling | Next-rung trigger |
| --- | --- | --- | --- |
| Lost wake-up | below the ladder (silent; §3 specimens) | structurally guaranteed: the evidence write and its entry are one durable append | M1 counts resync findings (mechanically preventable); an outbox-style append makes it structural |
| Starvation | below the ladder (discovery prefix) | structurally guaranteed: no selected order may exceed a declared maximum wait (§5.4) | M0's dispatch subject with the maximum-wait constraint |
| "Not yet" typed as "no" | below the ladder (ten places, §7 wave C) | structurally impossible: `Waiting` requires a named source and `Stopped` requires an owner | M0's closed standing, then wave C's fixes |
| Silent consumer failure | below the ladder (publication helper) | structurally guaranteed: a consumer is a queue member with a standing | wave D |
| Two workers on one item | mitigatable (unit name) | structurally guaranteed: fenced claims | wave B's single fencing token, then M1's holds |
| Automatic retry of a non-idempotent step | mechanically preventable (ensure admission) | structurally impossible: `RetryAfter` requires a retry law | wave E |
| A mechanism fixing a preference that belongs to the demand | below the ladder (§7 wave F inventory) | structurally guaranteed: every runtime choice is a `select_realization` under the demand's objective, and the policy-switch witness holds | wave F plus decisions (a), (b) and (d) |
| Work in a slice whose bound belongs to others | mechanically preventable (#13795) | structurally guaranteed: worker placement is admitted | M1's host realization admits worker slices |

## 9. Milestones

**Waves A to E: consolidation, first.** Wave A (this lane) and waves B, D and E (one child session) run now; wave C follows A in this lane; wave F runs in the flex-inference lane on the agreed interface (§5.9). No new module lands before the wave that owns its concept.

**M0: the model, with no consumer change.**

- `std.work_queue`, factored out of `v2.std.demand_engine` `DemandState` by the §5.2 map, imported by the demand engine.
- `std.durable_handoff` with every law of §5, each witnessed on a planted RED:
  - two enqueues collapse into one entry;
  - a selected order that would exceed a declared maximum wait is infeasible;
  - a claim against a stale fence refuses;
  - `Waiting` without a source and `Stopped` without an owner cannot be built;
  - `RetryAfter` without a retry law refuses, and the whole-history bound survives a restart;
  - the resync cursor advances past a prefix that stays owed.
- The shared policy-switch witness of §5.9, as soon as `ValuedObjective` lands.

**M1: the belt is the first consumer, on the lab first.**

- `RevisionChange` fires on deploy; `Completion` fires after every step; a timer drives `DueTime` and resync.
- Resync walks a cursor over an index of unfinished subjects, replacing `belt_discovery_attempt_bound` prefix discovery.
- The verification-window "not yet" becomes `Waiting`.
- Decisions (a), (b) and (d) are made through `select_realization` under each item's demand.
- Queue health appears on the served page.
- **Acceptance:**
  - N lab dispatches run end to end with zero hand steps;
  - the 2026-09-21 ruling's clause 4 demonstrations pass: one attempt reaches its PR while another is stopped, and killing the controller mid-obligation resumes without user action;
  - resync finds zero lost wake-ups across those N runs, which is the evidence that the event sources are complete;
  - switching the lab's demand profile changes dispatch order on a planted tradeoff, with no code edit.

**M2: the publication helper becomes a queue.** The belt settles with `Transferred`, and the helper's start failure surfaces as `Stopped`. Both publication rung drops are retired by this milestone, by their own triggers.

**M3: the conformance row lands (§10), and the remaining services migrate one at a time.**

- Order of migration: compute recovery (`gunbc.compute.attempt_lifecycle`), the workspace consumer (`gunbc.workspace_request_consumer`), runner cells, and fleet converge. `gunbc.serving.caller_backlog` (#13789) inhabits the model by projection from its first landing: `CallerWorkRecord` is the item, its grant generation (`product.fabric.budget` `fence_admits`) is the hold fence, `WorkWaiting` derives `Ready`, `Waiting` or `Running`, `WorkCompleted` and `WorkCancelled` are two of the `Finished` terminals, and `WorkRecoveryBoundExceeded` is `Finished { Exhausted }` (agreed with its owner, deep-otter-797, 2026-10-12).
- Each migration is a replacement migration under the doctrine:
  - delete the service's own hold, retry and anti-entropy spellings;
  - fix forward onto the shared model;
  - census by deletion.

## 10. The proposed §3b conformance row

The row is added to `gunbc.design_argument` `conformance_domains` once M0's homes exist and M1 has consumed them. That follows §3b's rule that a row names homes that exist, and clause 6 of the 2026-09-21 ruling. Adding it needs operator sign-off, because it changes the reviewer roster.

### Durable handoff / work queues

**Owns.** Work whose responsibility crosses an execution boundary: how it is accepted, keyed, woken, claimed, advanced, retried, stopped, transferred, resynced and reported.

**Boundaries.**

- Leasing owns the hold primitive; this domain uses it.
- Fabric and compute own allocation.
- §3d selection precedes: a queue advances a chosen realization and never chooses one; its own decisions (order, retry, expiry) are selections under the demand's objective.
- Goal assessment checks the state an item reaches.

**Existing authorities.** `std.work_queue` and `std.durable_handoff` (new); `std.durable_exclusive_hold`; `std.durable_compare_and_set` `CasGeneration`; `std.effects` `IdempotencyEvidence`; `std.decision`; and `gunbc.durable_handoff_host`.

**Reviewer question.** Does this boundary hand off responsibility durably?

- Bounded admission and durable acceptance.
- A wake-up source for every state change that can make work owed.
- A closed standing for every item, in which "not yet" names its event and "stopped" names its owner.
- Order, retry timing, waiting and expiry decided under the demand's objective, never fixed in the mechanism.
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
- A fixed preference in a mechanism: oldest first, warm first, roster order, first compare-and-set wins, retry until success.
- A new spelling of hold, dedup, dead letter or due time.

## 11. Owner decisions and open questions

1. **Home names. Decided (owner, 2026-10-12).** `std.work_queue` (the vocabulary shared with the demand engine), `std.durable_handoff` (what durability adds) and `gunbc.durable_handoff_host` (the host realization). With it, the instruction that every existing form of each concept is consolidated first (§7).
2. **Shared vocabulary with the demand engine. Decided (owner, 2026-10-12): now.** M0 factors the standings and the dispatch rule into `std.work_queue`, by the §5.2 map, imported by `v2.std.demand_engine` and `std.durable_handoff`.
3. **Retry and stopping. Partly decided.** The owner expects most complex state-based work to move onto a convergence process. The flex-inference lane supplied the retry law's shape, and the agreed interface (§5.9) settles that the law decides only admissibility and the earliest time while the choice of retry is a selection under the demand's objective. Still open: whether stopping the line stays the default for a failure whose obligation declares no retry law.
4. **Scope. Decided (owner, 2026-10-12).** Synchronous requests are always valid, and most requests are logically synchronous. How one is processed mechanically, whether in-process or durably handed off and awaited, is an implementation detail this corpus owns. So the serving path is in scope as a realization question: `gunbc.serving.caller_retry_contract`'s stance that there is no server-side queue is a realization choice to revisit under this contract, not a scope exclusion. Local function calls stay out of scope.

## Dissolution trigger (DESIGN §6)

Delete this document when std.durable_handoff and its host realization carry these rules in their own declarations and witnesses, every service in section 9 has migrated onto them, and the section 10 conformance row has landed.
