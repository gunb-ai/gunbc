# Belt demotion: event-driven attempt obligations

Status: plan. This is the design for items #6 and #4 of [dogfood-route-manual-interventions](dogfood-route-manual-interventions.md), written before the implementation. The model's homes are the factory controller, which owns wake-ups and event delivery, and the factory attempt lifecycle. Any roadmap rows land under those homes. `factory-dogfood-route` (gunbc#13077) only consumes their receipts and carries a gating edge on them.

## The defect, re-derived (DESIGN §6b)

`gunbc.roadmap_belt_actuate` `belt_tick_for_instance` is a single process that runs, in order: launch admission, spawn, teardown, commit (`belt_commit_attempts_for_instance`), verify (`belt_verify_attempts_after_commit`), review and audit (`belt_review_attempts_after_verify`, `belt_audit_attempts_after_verify`), and publish (`belt_publish_attempts_for_instance`). After that, `gunbc.roadmap_belt_tick_cli` `belt_run_once_cli_in` writes the served observation.

The earliest unjustified boundary is not a slow stage. It is the claim, implicit in that one function, that these are one fact. They are separate facts:
- each attempt's commit, verify, review and publish is an obligation of that attempt alone;
- launch admission is an obligation of the frontier;
- the served observation is an obligation of the instance.

Fusing them into one sequence makes each one's latency every other one's latency. The srv2 timeout, where one SCM capture starved everything after it from 06:50 to 16:30 on 2026-10-03, is a consequence of that fusion. Making the tick resumable with checkpoints would keep the fusion, which the operator has ruled against.

## The model

### Obligation identity (keys, DESIGN §3b)

An obligation is `(kind, subject)`:
- `kind` is one of `Capture`, `Verify`, `Review`, `Publish`, `ResultReturn` (#13075), `PlacementSettle` (#13074), `DogfoodStartRecord` (#13077), `LaunchAdmission`, and `ServedObservation`.
- `subject` is the attempt identity (`gunbc.roadmap.roadmap_attempt_occurrence`) for per-attempt kinds, the frontier revision for `LaunchAdmission`, and the instance for `ServedObservation`.

The effect identity is derived from that pair and is never minted fresh. Re-running an obligation therefore re-observes its subject and decides Noop. Publish is the one obligation whose effect subject is not the attempt. There is one PR per logical work item (the roadmap node W). A later attempt updates that PR and does not open a second one.

- The attempt identity keys the publication operation and its receipt. It never keys the PR.
- So the ensure is "work item W has exactly one PR, at attempt A's head", idempotent per (W, A).
- After an uncertain remote success, the obligation observes W's PR before it creates one.

### Each obligation is one ensure (convergence, DESIGN §3d)

Each kind is a `gunbc.ensure` `ensure_decide` over:
- the attempt's evidence authority (its verification receipt, review receipt or publication receipt);
- policy `EnsureMayApply`;
- a plan that is the existing per-attempt body.

The per-attempt bodies already exist as their own entry points in `gunbc.roadmap_belt_tick_cli`: `belt_verify_one_cli`, `belt_review_head_cli`, `belt_integrate_one_cli` and `belt_escalate_one_cli`. The demotion reuses them. It does not write a second copy (§3). The batch functions `belt_*_attempts_*` reduce to "for each attempt, decide whether its obligation is outstanding". They stop doing the work themselves.

A failed obligation records a typed refusal under its identity and stops. It does not retry itself, and it does not block any other obligation. This follows the operator's 2026-10-03 manual-resume ruling: the line stops for that attempt only.

### Trigger edges (event-driven, item #4)

| event | obligation invoked |
|---|---|
| worker transient unit exits (systemd `ExecStopPost=` on the unit `gunbc.roadmap_dispatch_actuator` already mints) | `Capture` then `Verify` for that attempt |
| verification receipt written | `Review` and audit for that attempt |
| passing review receipt | `Publish`, then `ResultReturn` |
| lease grant or release (#13074) | `PlacementSettle` |
| acceptance or merge event on the roadmap event log | `LaunchAdmission` |
| any receipt written for the instance | `ServedObservation` (cheap, run on its own) |

Each edge runs as its own process, keyed by obligation identity. If the edge's command crashes, its unit records the failure, and the obligation is still outstanding the next time anything observes it.

### Deadlines

Obligations with a deadline, such as a lease expiry (`std.temporal_effect` `HeldLease`) or a seat grant boundary, schedule a one-shot wake-up (a transient `systemd-run --on-active=` timer) keyed to the obligation. No recurring timer is a product requirement, and none becomes a desired-state row.

### Anti-entropy, bounded and discovery-only

An event can be missed: a host reboots, a hook is lost, or a unit is collected. To cover that, `belt_discover_once` lists up to N outstanding obligations:
- it compares each attempt's evidence against the receipts each kind requires;
- it hands each outstanding obligation to its edge command as a separate unit, which returns at once;
- it never runs capture, verify, review or publish inline.

Its cost is the observation cost of the attempts it reads, not the cost of the work. Whether anything invokes it periodically is a deployment choice outside the model. The model does not require it.

### Exclusivity (leasing/locking, DESIGN §3b)

Two edge invocations for one obligation identity must not both apply. Each obligation holds a `std.durable_exclusive_hold` keyed by its identity for the length of its apply. If a second invocation finds the hold taken, it decides Noop with the reason "held by <holder>". Obligations on different identities never contend, and that independence is what makes the item #6 RED pass.

### Observability (process reporting, DESIGN §3b)

Each obligation writes its own receipt, a `std.temporal_effect` `EffectStepReceipt` keyed by obligation identity. `ServedObservation` folds the latest receipts, so the page never waits on a running obligation. A running obligation is reported as running. Its absence from the page is not the signal.

## REDs (written first)

1. **Item #6.** Fixture: two attempts, A and B. A's verify is held, its hold is taken and it is running. B has a passing review. Invoking `Publish(B)` and `ServedObservation` completes both, and neither one observes or awaits A. Today that is impossible, because publish runs inside the same function after every verify.
2. **Item #4.** With no belt timer in deployment membership, a worker unit's exit hook invokes `Capture` and `Verify` for exactly that attempt. The control fails if `ExecStopPost=` is removed from the minted unit.
3. **Idempotence and PR identity.**
   - Invoking `Publish(W, A)` twice yields Noop the second time. The Noop is decided from the observed PR, not a local flag.
   - `Publish(W, A2)` after `Publish(W, A1)` updates W's one PR to A2's head. It does not create a second PR.
   - When a create succeeded remotely but left no receipt, the next invocation observes the existing PR and does not create another.

## Sequencing with the open PRs

- #13074 (G5), #13077 (dogfood route), #13075 (G4) and #13111 (SCM) all edit `belt_actuate`. This work rebases onto each as it lands.
- Each of their new passes becomes an obligation kind in the table above. None of them stays an inline step.
- Landing order:
  1. the obligation identity type, the per-kind outstanding predicate, and the REDs;
  2. edge entry points and unit hooks;
  3. `belt_tick_for_instance` reduced to launch admission plus discovery, with the inline passes deleted in the same change (a replacement migration, §3; delete first);
  4. the belt timer removed from deployment membership.

## Rulings (bold-bee-114, 2026-10-03)

- **Q1: no `std.obligation` carrier.** An obligation is outstanding when the attempt's own durable evidence says so. The discover pass computes that, and a queue entry is just `(kind, attempt identity)`.
- **Q2: the exit hook appends an event and does not invoke work directly.** The event is durable before any work starts. It does not go to `gunbc.roadmap_event_log`, which is the issue event authority: it pushes a git commit with a compare-and-set per event, and it is the history the requester reads. `gunbc.fabric_event_log` does not fit either. Appending there needs a head read and a compare-and-set put through the fabric storage client, and the log is ordered as a chain rather than create-only per key. The hook may not run the gunbc interpreter, because that would cost a typecheck on every exit.

### The attempt-event spool (proposed minimal store)

- **Home.** An instance-local spool directory, homed in `gunbc.host_layout` beside the publication spool. It follows the same precedent: two consumers, and `gunbc.live_deploy.emit` creates the directory with the ownership it needs.
- **Hook.** The hook is a typed argv command with no shell: `ExecStopPost=/usr/bin/mkdir <spool>/%n.${INVOCATION_ID}.${SERVICE_RESULT}.${EXIT_CODE}.${EXIT_STATUS}`. It is minted in `.dag` as an `ArgvCommand` beside the `systemd_run_property` rows.
  - systemd expands `%n` and `${VAR}` in Exec argv, and it sets `INVOCATION_ID`, `SERVICE_RESULT`, `EXIT_CODE` and `EXIT_STATUS` for `ExecStopPost`. The extdeps model cites systemd.exec(5) and systemd.service(5) for these.
  - `mkdir` is atomic and create-only, so a repeated hook fails harmlessly.
  - The entry's name carries the exit facts. Anything that does not fit in a name, the edge reads from systemd or the journal by unit and invocation id. The hook never writes it.
  - Shell control text in a minted command is ruled out, because that is the class the shell-typed-invocation project is deleting.
- **Trigger.** A systemd `.path` unit (`DirectoryNotEmpty=<spool>`) in deployment membership starts the edge service. This trigger is an event, not a timer. The edge decodes each entry and runs `Capture` and `Verify` for that attempt while holding that attempt's exclusive hold.
- **Consumption.** An entry is moved to `done/` only after its obligation's receipt exists. If the edge crashes, the entry stays and is picked up by the next activation or by discovery.
