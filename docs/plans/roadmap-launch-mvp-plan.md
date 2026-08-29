 # Roadmap Launch MVP — Gated Serial Bootstrap

Project ID: RLM · Repository anchor: gunb-ai/gunbc · Baseline reviewed: main@5e80671ce4a5fa27ea52da29f36c15c64827785c (re-verified by the manager on main@dd7bca774604c3f30119bc600295b7be14b69332, 2026-08-29: no roadmap/belt change between the two).

Terminal: edit the roadmap authority, deploy the exact accepted revision, launch one Ready task from the daily workspace, observe exactly one worker complete it, verify and publish the result, accept it, and see its child become Ready.

 ## 1. Terminal acceptance procedure

Starting from a clean revision R on main:

1. A two-node canary exists in roadmap authority: parent `roadmap-launch-canary`, child `roadmap-launch-canary-followup`.
2. The parent has an exact bounded execution contract and direct validation oracle; the child depends on the parent's accepted completion.
3. Required checks succeed and `refs/fleet/desired == R`.
4. Fleet convergence and dashboard deployment prove the production roadmap instance runs exactly R; release identity and deployed tree digest are independently read back.
5. The daily workspace shows the parent Ready, the child Upcoming, and the exact parent blocker on the child.
6. Production launch mode is `ManualReady`: the timer may reconcile, verify, publish, clean up, but may not start new work.
7. Clicking Launch on the parent creates exactly one attempt, one worktree, one provider process, one durable attempt record.
8. A second click returns the same live attempt.
9. Launching the child returns a typed dependency refusal before any worktree or provider effect.
10. The worker performs only the canary's exact change.
11. Verification evaluates the pinned oracle against the exact attempt head and records a passing receipt.
12. Publication offers that exact head with the correct base/node/attempt/criteria identity and records a publication receipt.
13. The daily workspace shows the parent completed/reviewable with observable activity and the published change identity.
14. After merge and explicit roadmap acceptance, the parent is Done, the child becomes Ready, and relaunching the parent is refused as already accepted.

No substitute terminal counts: a unit test, synthetic fixture, successful deploy alone, manually created branch, or hand-edited host does not satisfy this procedure.

 ## 2. Current-state ruling

Already usable: daily workspace renders lifecycle/scheduling and consumes observed attempt activity; the belt has provider preflight, footprint admission, worktree/attempt/tmux spawn, exact-head verification, confined publication, durable tick/attempt receipts; timer and manual dispatch refuse under transition-inhibited or revision drift; required CI and fleet-desired admission green on baseline; fleet convergence has typed plan/apply, fingerprints, lease/baseline checks, wet live-deploy/readback.

Prevents the MVP today: (1) no current exact-revision production receipt for srv1; (2) two conflicting launch authorities — the periodic belt consumes the Ready frontier while manual dispatch permits a node-ID override without dependency readiness (`AvailableAsOverride` in `gunbc.roadmap_presentation`); (3) unsafe activation posture — `gunbc.roadmap_belt` defaults `roadmap_belt_default_run_state = Running`, `roadmap_belt_default_capacity = 3`; (4) no wet end-to-end receipt chain; (5) fleet convergence/dashboard deploy remain explicit workflow_dispatch.

Off the critical path: the compute-fabric / own-CI program (#9682 binds required build Work but has no grant→executor→process→result path). The belt's provider/tmux path is the execution path for this MVP.

 ## 3. Serial gate chain

Only one gate may be mutating at a time. A downstream lane may author inert types, fixtures, and falsifiers, but may not activate production rows, transports, current-state claims, or success constructors before its predecessor's frozen receipt is accepted.

 ### RLM-0 — Freeze the terminal and inert canary
Question owned: what exact observable procedure constitutes success?
Build: commit this brief as controlling scope (enrolled in `gunbc.doc_graph_roots` with a dissolution row); define stable parent/child node identities; define the canary execution contract (exact task text, exact allowed change, exact red control, exact direct validation operation); add synthetic authority fixtures proving parent-Ready / child-blocked; keep the live canary OUT of production authority.
Suggested canary: a dedicated `.dag` module holds one sentinel bound to a nonce, initially `pending:<nonce>`; the task may change only that value to `completed:<nonce>`; a direct validation claim is true only for the exact expected value. It is the attempt's pinned validation oracle, not a global-floor row.
Exit receipt: parent Ready in the synthetic graph; child blocked by the exact parent identity; oracle red on base, green on the one authorized mutation, red on wrong/missing nonce; criteria and authority digests frozen.
Falsifiers: delete the dependency edge → fixture red; change the child blocker identity → red; wrong sentinel/nonce → validation red; an unspecified execution contract becoming launchable → admission fixture red.
Stop condition: any ambiguity about task bounds, validation identity, or the acceptance event stops the chain.

 ### RLM-1 — One shared serial launch admission
Question owned: may this exact cause start this exact node now?
Build one decision consumed by daily-workspace action rendering, `POST /dispatch/{node_id}`, and periodic belt spawning:
```
LaunchCause = Operator | Timer
SpawnMode   = Paused | ManualReady | AutomaticReady
LaunchAdmission = Admitted(launch_identity)
  | Refused(TransitionInhibited | RevisionUnobserved | RevisionDrift | NodeUnknown
          | DependenciesBlocked | ReviewRequired | Unplanned | ExecutionContractMissing
          | SizeMissing | AlreadyAccepted | AlreadyLive | NoCapacity | InstanceActuationRefused)
```
Order: instance actuatable → transition clear → desired==deployed → node exists, not superseded/accepted/review-only → contract and size complete → dependencies closed and Ready → no live attempt → capacity → cause permitted by mode.
Production = `ManualReady`: operator may start only Ready nodes; timer may not start new work; timer reconciliation/verification/publication/cleanup continue. Remove ordinary `AvailableAsOverride` from blocked/upcoming/unplanned/review rows; any break-glass path is a separate type/endpoint/authorization/receipt and is out of scope.
Exit: UI, manual route, timer return projections of one decision identity and reason.
Falsifiers: blocked child POST refuses before worktree/provider effects; timer in ManualReady never starts a Ready node; removing the dependency check reds a mutation control; drift refuses UI availability and backend actuation; double dispatch → AlreadyLive.
Frozen join: admission contract digest + deployed revision containing it.

 ### RLM-2 — Exact-revision fleet convergence and launch environment
Merge RLM-1; prove `main == refs/fleet/desired == R`; run the existing srv1 fleet-converge plan (keep plan identity, baseline, fingerprint, lease epoch); wet apply against that exact plan; existing dashboard-deploy mode where required; independently read back release identity, deployed tree digest, readiness, transition state, service/timer state, mutable-root permissions, provider preflight, attempt-ledger observability. One `RoadmapLaunchDeploymentReceipt` joins all of it. No host hand edits.
Falsifiers: deploy R-1 refuses equality; transition marker refuses actuation; post-apply tree mutation refuses digest readback; unwritable roots refuse footprint; missing provider refuses before spawn; unreadable attempt observation never renders idle.
Frozen join: deployed revision + tree digest + receipt identity.

 ### RLM-3 — Activate and serve the two-node canary
Add parent/child to live authority; merge/admit; converge via RLM-2; observe page and `/workflow.json`.
Exit: parent Ready with ordinary Launch; child Upcoming naming the exact parent blocker, no ordinary Launch; no canary attempt; page/JSON agree; belt ManualReady, no timer attempt.
Falsifiers: successful ordinary child launch; page/JSON disagreement; unobservable ledger rendered idle; timer spawn before operator click.

 ### RLM-4 — One click produces exactly one attempt
Click Launch on parent; click again; attempt child. First action joins admission identity + spawn result + node/attempt/base/provider identities + one branch/worktree + one process + durable record + live activity. Second click resolves to the existing attempt; child → DependenciesBlocked before any effect.
Falsifiers: two worktrees/processes; wrong identities in the record; child worktree; lost persistence.

 ### RLM-5 — Exact work verifies and publishes
Provider completes; belt reconciles/verifies/publishes in ManualReady. Verification receipt binds node, attempt, base, head, criteria digest, oracle revision/source digest/entry/operation, outcome; publication helper receives the exact request and records a receipt.
Falsifiers: wrong sentinel/nonce; extra path; changed oracle identity; wrong base/head/node/attempt/criteria refuses publication; completion without receipt not shown successful; helper failure never synthesizes success.

 ### RLM-6 — Acceptance ratchet unlocks the child
Merge the canary result; record the explicit roadmap acceptance event for the exact node/result identity; admit/converge/deploy.
Exit: parent Done by acceptance; child Ready; parent relaunch refused AlreadyAccepted; one terminal receipt joins every frozen identity.
Falsifiers: merged-but-unaccepted parent stays active and child stays blocked; acceptance for another identity does not unlock; losing acceptance evidence cannot leave the child green; no renderer/PR/status carrier substitutes for the acceptance producer.

 ## 4. Ownership matrix
Ready? → roadmap graph/acceptance authority (page, manual route, timer project it). May this cause launch now? → shared launch admission. What revision should production run? → refs/fleet/desired admission. What does production run? → independent live-deploy readback. Can the host spawn? → belt footprint/provider preflight. What ran? → durable attempt-state producer. Did it pass? → pinned verification receipt. Was the head offered? → confined publication receipt. Is it Done? → explicit roadmap acceptance authority.

 ## 5. Manager protocol
One manager owns RLM-0..6 and alone opens the next gate. One mutating worker at a time. Every gate begins with its red controls committed. Every report names producer, revision, instrument, host, immutable identities. Identities decide; counts summarize. A typed refusal is a valid result; a workaround is a stop signal. No hand edits to generated output or the production host. No lane grades itself. Successors bind exact SHA/digest/receipt, never "latest". Synthetic evidence ends at RLM-1.

 ## 6. Scope boundary
Required: shared dependency-honest launch admission; ManualReady; exact desired/deployed equality; existing fleet plan/apply + dashboard deploy; independent readback; provider/footprint preflight; two-node live canary; exactly-once launch; exact verification + confined publication; explicit acceptance + child unlock.
Deferred: Fabric CI execution grants; automatic desired-ref→host convergence; offline-host catch-up; whole-fleet proof; generalized automatic scheduling; parked ctrl spawner revival; multi-provider execution; blue/green (retired by design).

 ## 7. Post-MVP fleet continuation
FC-A desired-ref advance requests a reviewed host convergence → FC-B online hosts consume without manual dispatch → FC-C offline catch-up with stale/backward refusal → FC-D whole-fleet standing enumerates every host/phase → FC-E dashboard deploy joins the same closure. Must preserve the RLM terminal unchanged.

 ## 8. First implementation slice
After the inert RLM-0, the first live PR is RLM-1: shared serial launch admission + ManualReady. It does not add the live canary, change fleet automation, or touch compute fabric.
