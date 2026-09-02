# Roadmap Launch MVP — Gated Serial Bootstrap

Project ID: RLM

Repository anchor: gunb-ai/gunbc

Baseline reviewed: main@5e80671ce4a5fa27ea52da29f36c15c64827785c

Terminal: edit the roadmap, deploy the exact accepted revision, launch one ready task from the daily workspace, observe exactly one worker complete it, verify and publish the result, accept it, and see its child become ready.

## 1. Terminal acceptance procedure

Starting from a clean revision R on main:

1. A two-node canary exists in roadmap authority: parent `roadmap-launch-canary` and child `roadmap-launch-canary-followup`.
2. The parent has an exact bounded execution contract and direct validation oracle; the child depends on the parent's accepted completion.
3. Required checks succeed and `refs/fleet/desired == R`.
4. Fleet convergence and dashboard deployment prove the production roadmap instance is running exactly R; the release identity and deployed tree digest are independently read back.
5. The daily workspace shows the parent as Ready, the child as Upcoming, and the exact parent blocker on the child.
6. Production launch mode is `ManualReady`: the timer may reconcile, verify, publish, and clean up, but may not start new work.
7. Clicking Launch on the parent creates exactly one attempt, one worktree, one provider process, and one durable attempt record.
8. A second click returns the same live attempt rather than spawning a duplicate.
9. Attempting to launch the child returns a typed dependency refusal before any worktree or provider effect.
10. The worker performs only the canary's exact change.
11. Verification evaluates the pinned oracle against the exact attempt head and records a passing receipt.
12. Publication offers that exact head with the correct base, node, attempt, and criteria identity and records a successful publication receipt.
13. The daily workspace shows the parent as completed/reviewable with observable activity and the published change identity.
14. After merge and explicit roadmap acceptance, the parent is Done, the child becomes Ready, and relaunching the parent is refused as already accepted.

No substitute terminal counts: a unit test, synthetic fixture, successful deploy alone, manually created branch, or manually edited host does not satisfy this procedure.

## 2. Current-state ruling

### Already usable

- The daily workspace renders roadmap lifecycle/scheduling and consumes observed attempt activity.
- The belt has real provider preflight, footprint admission, worktree/attempt/tmux spawn, exact-head verification, confined publication, and durable tick/attempt receipts.
- Both timer and manual dispatch refuse while repository transition is inhibited or desired/deployed revision standing is drifted or unobservable.
- Required CI and fleet-desired admission are green on the reviewed baseline; `refs/fleet/desired` equals main at that revision.
- Fleet convergence has typed plan/apply artifacts, fingerprints, lease/baseline checks, and wet live-deploy/readback machinery.

### Prevents the MVP today

- **No current exact-revision production receipt.** Repository evidence proves admission to `refs/fleet/desired`, not that srv1 is running that revision now.
- **Two conflicting launch authorities.** The periodic belt consumes the ready frontier, but manual dispatch intentionally permits a node-ID override without proving dependency readiness.
- **Unsafe activation posture.** The belt defaults to `Running` with capacity three, so a live canary added before a manual-only mode could be timer-launched before the operator test.
- **No wet end-to-end proof.** No single receipt chain covers browser launch → exactly one attempt → exact work → verification → publication → acceptance → child unlock.
- **Fleet autonomy remains incomplete.** `refs/fleet/desired` advances automatically, but production convergence/dashboard deployment are still explicit workflow-dispatch operations.

### Explicitly off the critical path

The compute-fabric / own-CI program is not required for this MVP. It now binds exact required build work but still has no execution grant → process → result path; the roadmap belt already has a separate production provider/tmux execution path. Do not couple the first roadmap-launch proof to Fabric CI.

## 3. Serial gate chain

Only one gate may be mutating at a time. A downstream lane may author inert types, fixtures, and falsifiers, but may not activate production rows, transports, current-state claims, or success constructors before its predecessor's frozen receipt is accepted.

### RLM-0 — Freeze the terminal and inert canary

**Question owned:** What exact observable procedure constitutes success?

**Build**

- Commit this project brief as the controlling scope.
- Define stable parent/child node identities.
- Define the canary execution contract: exact task text, exact allowed change, exact red control, and exact direct validation operation.
- Add synthetic authority fixtures proving parent-ready / child-blocked behavior.
- Keep the live canary out of production authority.

**Suggested canary**

A dedicated `.dag` module holds one sentinel value bound to a nonce, initially `pending:<nonce>`. The task may change only that value to `completed:<nonce>`. A direct validation claim is true only for the exact expected value and identity. The claim need not be enrolled in the global required floor; it is the attempt's pinned validation oracle.

**Exit receipt**

- Parent is ready in the synthetic graph.
- Child is blocked by the exact parent identity.
- The oracle is red on base, green on the one authorized mutation, and red on wrong/missing nonce variants.
- Criteria and authority digests are frozen.

**Required falsifiers**

- Delete the dependency edge: fixture must red.
- Change the child blocker identity: fixture must red.
- Use the wrong sentinel or nonce: validation must red.
- Let an unspecified execution contract become launchable: admission fixture must red.

**Stop condition:** Any ambiguity about task bounds, validation identity, or the acceptance event stops the chain.

### RLM-1 — One shared serial launch admission

**Question owned:** May this exact cause start this exact node now?

**Build**

Create one authoritative decision consumed by all three surfaces:

- daily-workspace action rendering,
- `POST /dispatch/{node_id}`,
- periodic belt spawning.

Recommended shape:

```text
roadmap_launch_admission_for_instance(instance, node, cause, observation)

LaunchCause = Operator | Timer
SpawnMode   = Paused | ManualReady | AutomaticReady

Admitted(launch_identity)
Refused(TransitionInhibited | RevisionUnobserved | RevisionDrift |
        NodeUnknown | DependenciesBlocked | ReviewRequired |
        Unplanned | ExecutionContractMissing | SizeMissing |
        AlreadyAccepted | AlreadyLive | NoCapacity |
        InstanceActuationRefused)
```

Admission must establish, in order:

1. production instance is actuatable;
2. repository transition is clear;
3. desired and deployed revision identities agree;
4. node exists and is not superseded, accepted, or review-only;
5. execution contract and sizing are complete;
6. graph dependencies are closed and the node is Ready;
7. no live attempt already owns the node;
8. capacity is available;
9. cause is permitted by mode.

Set production to `ManualReady` for the MVP. In that mode:

- operator launch is allowed only for Ready nodes;
- timer launch is refused;
- timer reconciliation, verification, publication, and cleanup continue.

Remove ordinary `AvailableAsOverride` launch behavior from blocked/upcoming/unplanned/review rows. A future break-glass path, if retained, needs a separate type, endpoint, authorization, and receipt, and is excluded from this project.

**Exit receipt**

The UI, manual route, and timer all return projections of the same decision identity and reason.

**Required falsifiers**

- Blocked child POST refuses before worktree/provider effects.
- Timer in `ManualReady` never starts a ready node.
- Removing the dependency check makes a mutation control red.
- Deployed/desired drift refuses both UI availability and backend actuation.
- Double dispatch resolves to `AlreadyLive`, not a second spawn.

**Frozen join:** admission contract digest + deployed revision containing it.

**Stop condition:** Any surface independently re-derives eligibility or preserves an unreceipted override.

### RLM-2 — Exact-revision fleet convergence and launch environment

**Question owned:** Is the production roadmap instance demonstrably capable of executing RLM-1 at the exact accepted revision?

**Build / execute**

Use the existing reviewed path rather than inventing a second deploy mechanism:

- merge RLM-1 and wait for required checks;
- prove `main == refs/fleet/desired == R`;
- run fleet-converge plan for srv1 and preserve plan identity, baseline, fingerprint, and lease epoch;
- run the authorized wet apply against that exact plan;
- run the existing dashboard deployment mode for the same revision if it is not already covered by the host plan;
- independently read back live release identity, deployed tree digest, readiness, transition state, service/timer state, mutable-root permissions, provider preflight, and attempt-ledger observability.

Create one `RoadmapLaunchDeploymentReceipt` joining:

- desired revision;
- plan artifact/fingerprint;
- candidate and deployed tree digests;
- apply receipt;
- readiness/readback receipt;
- transition-clear observation;
- roadmap service identity;
- belt service/timer mode;
- worktree/attempt/receipt-root footprint result;
- provider preflight result;
- `/workflow.json` observation standing.

No host hand edits count. If an operator must repair the host, model the operation and rerun this gate.

**Exit receipt**

All launch prerequisites are proven against one exact deployed revision, with `ManualReady` observed live.

**Required falsifiers**

- Deploy `R-1`: release equality refuses.
- Leave a transition marker: actuation refuses.
- Mutate deployed tree after apply: digest readback refuses.
- Make receipt/worktree root unwritable: footprint admission refuses.
- Remove provider credentials/executable: provider preflight refuses before spawn.
- Make attempt observation unreadable: daily activity remains unobservable and the gate refuses.

**Frozen join:** deployed revision + deployed tree digest + launch-deployment receipt identity.

**Stop condition:** If srv1's exact state cannot be proven, repair fleet/live-deploy authority here; do not bypass the revision gate.

#### RLM-2 measured closure and independent deployment blocker (2026-09-02)

The plan/apply phase is closed. Before the launch-environment convergence scope landed, a wet plan run completed green while its typed terminal was `PartiallyApplied|33`, so apply correctly refused. After that scope landed, independent plans at different revisions repeatedly produced `FullyApplied`, `timer-refusals=0`, member-set fingerprint `fffcbb4e3a5cb7ca` equal to the observed baseline, and an exact one-generation advance. A wet apply then durably advanced srv1 from generation 1 to generation 2 with plan receipt `c317d936ce8c19ea`, plan artifact hash `1c9a3ae156f9ad04`, and apply receipt `608a6808d244d97e`; its terminal was `fully_applied` and its locked apply exit code was zero. A later fresh plan observed generation 2 and proposed generation 3, independently corroborating that the durable advance landed. The applied generation-2 checkpoint remains valid evidence even though its exact revision expired before the revision-bound deployment phases; no rollback is owed.

That closure does **not** make the RLM-2 terminal reachable. The production dashboard workflow invokes `gunbc.live_deploy.apply.live_deploy_apply_srv1_transaction_wet`, whose transaction path consults `repository_transition_admission` before candidate observation or mutation. The bound `deployed_tree_repository_transition` is `LegacyGitFileSync`, and the enrolled admission deliberately returns `LegacyGitFileSyncNotAdmitted`; only the not-yet-built `GitNativeConvergence` realization is admitted. Therefore `DeploymentComplete` is unreachable at every revision, including on perfectly still main, until the Git-native repository transition is implemented and evidenced. Changing the binding, bypassing the admission, or weakening its refusal before that realization exists would fabricate capability rather than clear the gate.

**Untracked stall (DESIGN section 4b): complete transaction input projection.** The exact-revision retry history exposed a second missing capability, but a path list or module digest is not its trigger. The next rung requires a **complete typed projection** covering every semantic, code, payload, and history input to plan, apply, dashboard deployment, belt observation, and final receipt, with every phase carrying and joining the same projection receipt. The present deployment ships the tracked working tree nearly wholesale and also consumes commit identity and ancestry; among the 40 first-parent main commits measured on 2026-09-02, 36 changed deployed paths, four preserved the root tree, and zero were provably irrelevant once revision topology was included. The measurement producer enumerated main's first-parent commits, compared each commit with its first parent, and classified every changed path through `gunbc.live_deploy.deployed_tree_scope.path_is_deployed` and its declared excluded prefixes before joining the four identical-tree commits to the revision-topology inputs. Replacing revision equality with a narrower content digest would weaken the forward-only and cross-receipt guarantees, so exact-revision admission remains authoritative until the complete projection exists. The existing typed cross-lane stall roster is `gunbc.guarantee_rung_drop.all_guarantee_stalls`; this deployment-projection stall is not yet enumerated there, so a lane that does not read this controlling plan cannot discover it through that roster today, and the roster's executing witnesses establish properties of the rows present rather than completeness of the stall population.

**Scope correction.** The RLM-2 close sequence was scoped against a terminal that could not be reached: its 14-step acceptance procedure depended on an admitted Git-native repository transition that the predecessor chain did not enumerate. The completed plan/apply receipts remain the correct predecessor evidence and must not be discarded or replayed, but dashboard deploy, launch-deployment receipt, `DeploymentComplete`, and the frozen RLM-2 join remain blocked by the independent repository-transition wall. A future close attempt begins only after that predecessor lands; it does not rediscover the wall by dispatching another exact-revision plan.

### RLM-3 — Activate and serve the two-node canary

**Question owned:** Does the deployed daily workspace project the correct serial frontier before actuation?

**Build / execute**

- Add the parent and child rows to live roadmap authority.
- Merge and admit the exact authority revision.
- Converge/deploy it using the RLM-2 transaction.
- Observe both browser page and `/workflow.json`.

**Exit receipt**

- Parent is Ready and launchable.
- Child is Upcoming/blocked and names the parent as its exact blocker.
- Child has no ordinary launch action.
- No canary attempt exists.
- Page and JSON carry the same scheduling/activity decision identities.
- Belt remains `ManualReady`; no timer-created attempt appears.

**Required falsifiers**

- Child launch button or successful ordinary child POST is red.
- Page Ready / JSON blocked disagreement is red.
- Unobservable attempt ledger rendered as idle/no-attempt is red.
- Automatic timer spawn before operator action is red.

**Frozen join:** served release revision + parent/child authority digests + criteria digest.

### RLM-4 — One click produces exactly one attempt

**Question owned:** Does an admitted operator action produce one—and only one—durable execution attempt?

**Execute**

- Click Launch on the parent in the deployed daily workspace.
- Immediately click it again.
- Attempt ordinary launch of the blocked child.

**Exit receipt**

The first action joins all of:

- admitted launch identity;
- HTTP spawn result;
- node/attempt/base/provider identities;
- exactly one branch and worktree;
- exactly one tmux/provider process;
- durable attempt-state record;
- observable live activity in page and JSON.

The second parent action resolves to the existing attempt. The child resolves to `DependenciesBlocked` before any filesystem or provider effect.

**Required falsifiers**

- Two worktrees/processes for one node is red.
- Attempt record with wrong base/node/provider identity is red.
- Child worktree creation is red.
- Lost attempt-state persistence is red even if a process happens to run.

**Frozen join:** attempt key + base revision + criteria digest + provider identity.

### RLM-5 — Exact work verifies and publishes

**Question owned:** Did the launched attempt satisfy the exact task, and was that exact result offered through the confined publication boundary?

**Execute**

Allow the provider to complete. The belt may reconcile, verify, and publish in `ManualReady` mode.

**Exit receipt**

- Provider completion is observed for the exact attempt.
- Changed paths and base/head identities are within the declared window.
- Pinned validation oracle passes against the exact head in a detached worktree.
- Verification receipt binds node, attempt, base, head, criteria digest, oracle revision/source digest/entry/operation, and outcome.
- Publication helper receives the exact expected request and records a successful receipt for that head.
- Daily workspace shows terminal attempt success/reviewability and the publication identity.

**Required falsifiers**

- Wrong sentinel or nonce fails verification.
- Extra unauthorized path fails bounds/validation.
- Changed validation oracle/source identity fails pinning.
- Wrong base, head, node, attempt, or criteria identity refuses publication.
- Provider completion without a durable verification receipt does not appear successful.
- Publication token/helper failure remains a typed refusal and does not synthesize success.

**Frozen join:** attempt head + verification receipt + publication receipt.

### RLM-6 — Acceptance ratchet unlocks the child

**Question owned:** Does accepted completion, rather than merge or self-report, advance the serial roadmap frontier?

**Execute**

- Merge the canary result.
- Record the existing explicit roadmap acceptance event against the exact node/result identity.
- Merge/admit/converge/deploy the resulting authority revision.

**Exit receipt**

- Parent is Done by accepted evidence.
- Child is Ready because the exact parent acceptance dependency is closed.
- Parent ordinary launch is refused as already accepted.
- Child is now the sole canary launchable frontier item.
- The terminal receipt joins all prior frozen identities.

**Required falsifiers**

- Merged but unaccepted parent remains Review/active and child remains blocked.
- Acceptance for a different head/node/criteria identity does not unlock the child.
- Losing the acceptance evidence cannot leave the child green.
- A status carrier or renderer cannot substitute for the acceptance producer.

**Frozen join / terminal:** accepted parent identity + child-ready projection digest + exact deployed revision.

## 4. Ownership matrix

| Question | Sole authority / producer | Consumers that may only project it |
| --- | --- | --- |
| Which node is structurally ready? | roadmap graph/acceptance authority | page, manual route, timer |
| May this cause launch it now? | shared serial launch admission | UI action, POST handler, belt scheduler |
| What revision should production run? | `refs/fleet/desired` admission | fleet plan, live deploy, belt revision gate |
| What revision/tree does production run? | independent live-deploy readback receipt | launch admission, page health, manager |
| Can the host start the provider safely? | belt footprint/provider preflight | spawn actuator |
| What exactly ran? | durable attempt-state producer | activity renderer, verifier, manager |
| Did the task pass? | pinned validation/verification receipt | publication and workspace presentation |
| Was the exact head offered? | confined publication receipt | workspace presentation, manager |
| Is the roadmap item Done? | explicit roadmap acceptance authority | ready frontier, page, belt |

The manager must always name the producer it read. A renderer, status carrier, branch, PR, or consumer is not the producer of current standing.

## 5. Manager protocol

1. One manager owns RLM-0 through RLM-6 and is the only authority that opens the next gate.
2. One mutating worker is active at a time. Parallel workers may only prepare inert schema, synthetic fixtures, or falsifiers behind the current gate.
3. Every gate begins with its red controls already committed to the brief.
4. Every report names producer, source revision, exact instrument/command, host/instance, immutable identities, and class delta.
5. Identities decide; counts only summarize.
6. A typed refusal naming missing evidence is a valid gate result. A workaround is a stop signal.
7. No hand edits to generated output or the production host. Re-run declared generation/convergence transactions.
8. A lane may not grade its own success. The manager reads the authoritative receipt producer.
9. The next gate binds the previous gate's immutable revision/digest/receipt, never "latest."
10. Synthetic/lab evidence may prove models, but only the production instance supplies RLM-2 through RLM-6 terminal receipts.

## 6. Scope boundary

### Required for this MVP

- shared dependency-honest launch admission;
- explicit `ManualReady` mode;
- exact desired/deployed revision equality;
- existing fleet plan/apply and dashboard deployment path;
- independent release/tree/readiness readback;
- provider/footprint preflight;
- two-node live canary;
- exactly-once launch/deduplication;
- exact verification and confined publication;
- explicit acceptance and child unlock.

### Deferred until after the terminal

- compute-fabric execution grants and own-CI process execution;
- event-driven main/desired-ref → automatic production convergence;
- offline-host catch-up and whole-fleet proof;
- generalized automatic roadmap scheduling;
- parked ctrl spawner revival;
- multi-provider arbitration or generalized customer execution;
- blue/green deployment (currently retired by design).

## 7. Post-MVP fleet continuation

After RLM-6, the smallest honest continuation is:

- **FC-A:** desired-ref advance automatically requests a reviewed host convergence transaction.
- **FC-B:** every online host consumes the latest admitted desired revision without manual workflow dispatch.
- **FC-C:** an offline host catches up on return, with stale-plan and backward/unrelated revision refusal.
- **FC-D:** whole-fleet standing enumerates every host and every phase; absence is explicit, never silently omitted.
- **FC-E:** dashboard deployment becomes an ordinary member of the same desired-state convergence closure, eliminating the remaining separate dispatch.

Do not fold these into RLM unless the current manual plan/apply path cannot produce the exact RLM-2 receipt. First prove the product loop with the existing reviewed actuator, then automate that actuator without changing the product acceptance test.

## 8. First implementation slice

The first live PR after the inert brief should be RLM-1: shared serial launch admission + `ManualReady` — the narrowest change that makes a live canary safe.

It should not add the live canary, change fleet automation, or touch compute fabric. Its sole acceptance: UI, manual POST, and timer share one launch decision; blocked work cannot be launched through the ordinary path; production can reconcile completed attempts without autonomously starting new ones.

## Manager amendment — 2026-08-29 (RLM-0 review ruling)

Recorded by the RLM manager (session tidy-swift-334) on the side-chat REQUEST_CHANGES review 5059303095 of PR #9694; the only departure from the controlling artifact above.

**Child terminal state is structural Ready only.** The parent `roadmap-launch-canary` is the one execution canary. After the parent's explicit acceptance, the child `roadmap-launch-canary-followup` need only become structurally `SchedulingReady`; it needs neither a complete execution contract nor an admitted Launch action. The RLM-6 exit line "Child is now the sole canary launchable frontier item" is read under this ruling as "Child is the sole canary frontier item that is structurally Ready". This suffices to prove the acceptance ratchet; the parent proves the launch/execute/verify/publish transaction. An unspecified execution contract must still never launch — that refusal is RLM-1's admission, not the child's readiness.
