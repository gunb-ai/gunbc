# Dogfood route: the manual interventions, dispositioned

Status: plan, operator-agreed 2026-10-03. Members land under `factory-dogfood-route` (gunbc#13077) and are owned by its lane.

## Why this exists

The first end-to-end attempt on srv2 (attempt `01feb0b65dee1377`, node `shell-dag-live-deploy-restart-tailscale`) did its work: 40 steps on a group-b seat, ending at its planned alignment checkpoint (exit 3). It never reached a PR, because every belt tick after about 06:50 UTC hit its start timeout and was killed. Getting it that far took six kinds of hand intervention. The operator's go/no-go on external customers is their own confidence from dogfooding, and a route that needs a person to keep it moving is not yet that evidence.

This document records each intervention, what it exposed, what the system does about it automatically, what stays with the operator, and the test that must go red first.

## Three properties, not one

Each intervention was first read as "make it idempotent / retryable". That conflates three properties:

1. **Safe to retry**: repeating an operation cannot duplicate or corrupt its effect. Required everywhere.
2. **Convergent**: desired state and observed state decide Noop, Apply or Refuse. Required everywhere.
3. **Automatically repaired**: the controller chooses and applies the repair with no operator. A policy choice, and today the answer is **no**.

`gunbc.ensure` `EnsurePolicy` already carries the distinction. `EnsureObserveOnly` refuses a mismatch; `EnsureMayApply` applies an available plan. Convergence therefore does not imply self-healing. It can converge to a safe, visible state and stop there.

### Operator ruling (2026-10-03): manual resume, for now

A crash or failure is observed by a person, root-caused, then resumed by hand, until there is confidence in the automated remedy. This is DESIGN §5's factory model: the line stops, and the stop is analysed before restart. What the system owes is that the stop is loud and located, and that the operator's resume is a single retry-safe action.

### The incident shape every intervention below uses

```text
normal desired state
  -> fault observed
safe incident state
  evidence captured
  route or work withdrawn where necessary
  AwaitingOperatorDisposition
  -> operator records a disposition, keyed by incident identity
ResumeRequested | AbandonRequested | ClearRequested | QuarantineRequested
  -> ensure
Noop | Apply | Refuse
```

Every disposition effect is keyed by its incident identity, so running it twice is a no-op. A crash after the effect but before its receipt re-observes the subject and finds the effect already done.

The same rule applies to every outward effect on the route: "open the PR" means "ensure attempt X has exactly one PR" (create-only, keyed by attempt identity). Pushes, comments and R2 writes follow the same rule.

## The six interventions

### 1. Stale units cleared by hand, three times

**Exposed:** exited worker and supervisor units blocked redispatch until a teardown pass ran. With the timer off, they had to be cleared by hand, and the clearing left no record. #13066 records the same follow-up.

**Automatic now:**
- observe the terminal or lease-expired unit;
- bind it to one incident identity;
- preserve its status, journal, attempt, lease and process evidence;
- stop counting it as live;
- withhold any replacement attempt until a disposition is recorded.

A lease deadline (`std.temporal_effect` `HeldLease`) establishes `TerminalOrLeaseExpired`. It never by itself kills, deletes or restarts.

**Stays manual:** the root cause, and the choice of Resume, Abandon, Clear or Quarantine.

**Home:** the factory attempt lifecycle, consumed by `factory-dogfood-route`.

**RED first:** kill a worker mid-turn. The unit is reported stale under one incident identity, with evidence preserved, and no replacement spawns. Recording `ClearIncident` twice changes nothing the second time.

### 2. srv10 Wi-Fi restored by hand

**Operator ruling:** the Sparks will serve on Wi-Fi for a while, and that is a valid medium. Why the link dropped still needs debugging.

**Exposed:** a host lost its network and stayed routable.

**Automatic now:**
- a missing or stale health receipt withdraws the host from routing;
- link evidence is captured before reconnection obscures it: association state, addressing and routing state, relevant service and driver evidence, and timestamps.

A fresh health reading may say the host is technically available again, while a separate unresolved incident keeps it operator-held.

**Stays manual:** the diagnosis, and returning the host to service.

**Home:** fabric host health (`gunbc.fleet_health`) and network incident handling, with an edge into the route. The existing roadmap row "A host serves only on a fresh satisfied health receipt" is the admission half.

**RED first:** drop Wi-Fi while the serving process stays alive. The host leaves the routable population, the incident evidence survives reconnection, and the host does not return until the operator's disposition permits it.

### 3. Untracked senders on group-b, paused or killed by hand

**Exposed:** a second door. Seat admission (`gunbc.serving.turn_admission` `serving_group_admission`) is not the only path onto group-b.

**Operator ruling:** move the other senders onto seat admission, and add authentication to the Spark deployments through a real auth process, so every caller is forced onto the right path.

**Authentication alone does not close the door.** A caller with a valid general-purpose credential could still bypass the seat ledger. The forwarding boundary needs both:
- an authenticated caller identity; **and**
- a current serving grant naming the admitted serving launch, the granted route or group, the seat entitlement, the caller or work identity, and an expiry or revocation boundary.

A practical realization is a short-lived signed request permit minted by successful seat admission. A Spark-side front door verifies it before forwarding to vLLM, and refuses before the engine sees the request.

**Stays manual:** migrating the existing senders, and deciding which identities may request grants.

**Home:** managed identity owns the credentials (`credential-lifecycle-revocation`, gunbc#13068); serving and capacity own request admission and grant validation.

**RED first:** a request with a valid caller credential and no seat grant is refused at the front door, and the refusal is counted.

### 4. Belt timer started by hand

**Operator ruling:** event-driven wherever possible, not timer-driven.

**Automatic now:**
- events invoke the smallest affected obligation directly;
- deadline-bearing obligations schedule a one-shot wake-up;
- a bounded anti-entropy sweep discovers missed events and **queues** them. It never performs the work itself.

**Stays manual:** only typed refusals and incident dispositions.

**Home:** factory controller and event delivery. The timer is not a product requirement and does not become a desired-state row.

**RED first:** with no timer installed, a worker exit causes its capture to run.

### 5. #13066 deployed to srv2 by hand

**Exposed:** srv2 runs `cc5bbdcc` plus the #13066 commit, placed by hand. The model exists: `gunbc.live_deploy.desired` declares srv1-live, srv1-lab and srv2-deploy (`srv2_deploy_desired_deployment`) beside `repository_convergence`. What was missing is converging an admitted revision onto the host.

**Correction to the first reading:** a branch revision is valid when it has been explicitly admitted to a development environment. "Not main" is not inherently drift.

**Operator context:** srv1 and srv2 are both development environments. srv1 is closer to production. The model's names (srv1 "live" / production) do not match that framing.

**Automatic now:** each development environment's explicitly admitted revision is observed and converged into the running deployment, with readiness and member-identity readback.

**Stays manual:** selecting and admitting the revision for that environment.

**Home:** generic live-deploy convergence (`gunbc.live_deploy`).

**Operator ruling (2026-10-03): converges are triggered manually today, and automated later.** Until they are automated, the manual trigger is one retry-safe action, the converge entry point, never a hand-run sequence of steps. Running it twice against a converged host is a Noop. The deployment hand-placed on srv2 for #13066 is the case this rules out: the admitted revision goes onto the host through the converge, not around it.

**Still owed a decision:** which instance the model calls production. The naming decides which host a wrong-instance mistake reaches. The belt-tick module records one such near miss: a tick ran `git worktree add` into srv1's production tree.

**RED first:** admit a revision for srv2. Convergence brings the running deployment to it and reads it back. A running revision that differs from the admitted one is reported as drift.

### 6. Every belt tick times out (unsolved)

**Exposed:** `gunbc.roadmap_belt_actuate` `belt_run_once` is one monolithic pass that runs these stages in sequence:
1. spawn;
2. teardown;
3. launch;
4. commit;
5. verify;
6. review;
7. publish;
8. write the served observation.

Any slow stage blocks every stage behind it. The module's own note on `belt_observe_once_cli` says that when the monolithic pass is too slow to reach its observation write, the page freezes at the last completed tick's bytes. After about 06:50 UTC, no tick completed, so capture, verify and publish never ran for the attempt.

**Not the fix:** stage checkpoints that make the monolithic tick resumable. That preserves the structure that should go.

**Automatic now:** expensive work leaves the tick. Each durable obligation gets its own event-driven ensure and its own effect identity:
- a worker exit triggers capture and verify;
- a verify receipt triggers review;
- a passing review triggers publish.

The anti-entropy pass only discovers and queues a bounded number of outstanding obligations.

**Stays manual:** intervention only on a typed refusal.

**Home:** belt demotion and the factory lifecycle.

**RED first:** a verify that runs long does not delay publish of an unrelated attempt, or the served observation.

## Order

The two that matter most for the dogfood clock:
- **6 (belt):** it is why the first attempt has no PR.
- **3 (second door):** untracked traffic blocks admission.

Next, 1 (incident record for stale units) and 5 (admitted-revision convergence) turn the remaining hand steps into single recorded actions. Items 2 and 4 follow.

None of this counts toward the dogfood window by itself. The window starts with the first qualifying run under `factory-dogfood-route` that also carries the joined evidence receipt from `service-evidence-capture` (gunbc#13068).
