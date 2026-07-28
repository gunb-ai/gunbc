# Recursive workflow advancement — design and completion plan

**Status:** DESIGN for review, 2026-07-28. This document records the incidents that led to
the portable Codex-dispatch and workflow-observation implementation, names the implementation's
honest current boundary, and specifies the remaining path to a minimum useful workflow.
Authority already present in `.dag` stays there; proposed carriers below are shapes to model and
witness, not permission to hand-code a second workflow engine. This document is bound to
`gunbc.workflow_reconcile.workflow_recursive_reconcile_note` in the doc graph and dissolves into
a registered `gunbc.plan.Plan` once advancement, the remaining receipt producers, and recursive
child realization have landed.

## 1. Operator outcome

From one roadmap item, an operator must be able to:

1. see the intended work and its current approved revision;
2. ask the system to advance the next realizable obligation;
3. watch environment, workspace, agent, verification, publication, review, and goal-audit facts
   appear from evidence;
4. inspect a precise refusal or failure without translating provider or host trivia;
5. change the goal deliberately without losing the original obligation; and
6. finish only when an antagonistic intent-versus-actual audit says the approved work is done.

The workflow is recursive. Any obligation may be realized directly or decomposed into another
planned workflow. “Build the frontend and backend” may therefore contain two child workflows,
and either child may decompose again. The same reconciliation law applies at every depth.

The system need not be stateless or distributed yet. It does need to make storage, host,
provider, and review transports explicit enough that moving from local files and tmux to
ephemeral workspaces, NFS/object storage, a table-backed coordinator, or an internal review API
does not change the meaning of a workflow fact.

## 2. Honest boundary of the current implementation

The current branch proves a useful foundation, not the whole business workflow.

| Segment | Current evidence | Honest projection |
| --- | --- | --- |
| Environment | admitted execution contract and exact host capabilities | Complete |
| Worktree | attempt state, branch, and worktree were created | Complete |
| Agent | bounded Codex events include `turn.completed`; retained process exited 0 | Complete |
| Verify | no authoritative validation receipt producer | Pending |
| Publish | no candidate/publication receipt producer | Pending |
| Review | no exact-candidate review receipt producer | Pending |
| Audit | no approved-intent versus actual audit receipt producer | Pending |

The live receipt that exposed the boundary was node `2-admission-model`, attempt
`189ec6faadc3e186`: the provider turn completed and the process exited successfully, while
Verify, Publish, Review, and Audit remained pending. That is the correct projection.

Clicking **Dispatch** again did not create a new attempt and did not advance verification. The
low-level actuator found the retained tmux container and returned `already_live`; the component
intentionally collapses both `spawned` and `already_live` acknowledgements to
`dispatch · accepted`. That acknowledgement is true at the session grain, but it is the wrong
primary control and the wrong caption at the workflow grain. This design removes that ambiguity.

## 3. Incident and gap ledger

This ledger separates an observed issue, what this branch has already corrected, and what still
owns the finish. “Landed” means modeled and witnessed on this branch; it does not claim that the
remaining workflow stages exist.

| Finding | Evidence / operator impact | Landed correction | Remaining obligation |
| --- | --- | --- | --- |
| Dashboard was unreachable from another LAN machine | `192.168.1.188` and then `192.168.1.185` timed out; a loopback or wrong-host address is not a shareable preview | Dashboard instances carry an explicit listen host/port and deployment posture; the lab was exposed through the host's reachable Tailscale name | Serve a modeled discovery/readback link in the UI instead of relying on a manually relayed address |
| One ambient repository path controlled dispatch | Observation refused because `git for-each-ref` tried `/opt/gunbc/gunbc`, which did not exist on the selected host | `HostDashboardInstance` owns source repository, remote, worktree root, state roots, port, and posture | Remove remaining srv1-specific defaults by passing instance identity through every workflow realizer |
| Safe iteration needed a separate lab | Manual changes risked the production dashboard and its state | The instance/apply model supports disjoint source, origin, worktrees, attempt/provider state, port, and observe/live posture; lab convergence was exercised | Make lab creation a first-class planned workflow and provide a visible production/lab identity |
| Absent paths were treated as exceptional command failures | A normal probe for a not-yet-created directory could refuse the apply before it had a chance to ensure it | Absence checks are modeled observations; parent directory ownership is preflighted before mutation | Keep probe absence, inaccessible state, conflict, and effect failure distinct in every new adapter |
| Deployment copied an incomplete hand-picked closure | New dashboard modules could exist in source but be absent from the served artifact | Complete dashboard source artifacts are rostered and modeled | Generalize artifact closure/readback so a new consumed module cannot be omitted silently |
| Parent ownership and cleanup were implicit | Apply could fail late on a root-owned parent; temporary cleanup could disagree with the apply contract | Parent ownership is preflighted and apply cleanup has an explicit contract | Provision incompatible parents through a separate privileged Ensure workflow, never an incidental web request |
| “Deployed” could mean the wrong source revision | File presence alone allowed false convergence against a stale source | Exact source revision is part of convergence and live lab readback receipts were recorded | Bind every later stage receipt to the exact candidate revision, not merely a branch name |
| Production health was a hidden precondition | One transient production `/healthz` failure correctly refused a lab apply before mutation, but the relationship was surprising | The pre-mutation refusal is loud and retry converged after recovery | Model the protection basis and its observation time; do not let an unrelated transient probe become an unexplained generic refusal |
| Claude availability was ambiguous | `claude auth status == false` is useful, but it is not evidence that the subscription quota is exhausted; no authenticated exhausted transcript was captured | Provider choice is explicit rather than “whichever CLI works”; unknown state does not fabricate a quota verdict | Add Claude auth/runtime event classification only from captured receipts; preserve unknown error grammar until witnessed |
| Codex OAuth and installation identity were finicky | Authentication, executable packaging, process name, state directory, and runtime success are different facts | Dedicated `CODEX_HOME`; modeled login-status and version probes; exact `codex-cli` identity; explicit noninteractive argv; live npm launcher receipt showing tmux observes `node` | Make provider provisioning/authentication its own Ensure workflow with the smallest unavoidable user interaction |
| Dependencies were discovered too late | An agent may need `cargo test`, rustc, rustfmt, git, tmux, jq, or a claim runner; discovering that inside the turn wastes the turn | `WorkItemExecutionContract` names semantic validation operations; task and platform capabilities derive mechanically; host realizations use `EnsurePolicy` and exact identity probes | Reuse the same contract to drive verification and provision missing capabilities before dispatch where policy permits |
| Session presence was mistaken for agent success | “session spawned/present” said nothing about provider readiness, progress, completion, or failure | Session/container, provider instance, provider event stream, and `WorkerProcessEvidence` are independent facts | Remove session-level Dispatch as the workflow's primary control |
| Stop and clear were conflated | A dead remain-on-exit pane showed Stop/refused; a red dotted Agent mark lacked a direct explanation | Running owned process → Stop; exited retained container → Clear; foreign/unobserved process → typed refusal; Agent state comes from provider/process evidence | Keep maintenance controls secondary and label every lamp with text/tooltips, not color or dots alone |
| A provider event could overwhelm observation | A Codex `item.completed` command event embedded about 154 KB of output in one JSONL record; another exceeded 200 KB; the old 64 KB line reader refused the stream | Raw JSONL is retained; modeled `/usr/bin/jq` validates each line and emits bounded structural envelopes before synchronous observation | Store raw/high-volume evidence behind references and keep all dashboard projections bounded |
| Malformed or contradictory events could imply false success | A terminal-looking nested field, malformed line, missing projector, zero exit without terminal event, or terminal success plus nonzero exit are unsafe | The event parser/projector and provider/process reconciliation fail closed with separate Failed and Refused outcomes | Apply the same contradiction laws to verify, publish, review, and audit receipts |
| Polling could race itself | Timer and post-dispatch refreshes could overlap and allow an older response to repaint newer facts | Each observation source is single-flight and coalesces one pending rerun | Add terminal-bounded polling once terminal attempt evidence is modeled |
| Workflow progress was invisible | The operator could not tell what clicking a task had accomplished | The row renders Env / Worktree / Agent / Verify / Publish / Review / Audit from a pure attempt evidence projection | Couple the primary control to a derived advancement plan |
| A stored stage enum would become rigid | Goals and decomposition change during real work; manual transition functions fork business meaning | `WorkflowFactTree`, its one catamorphism, and generic membership reconciliation derive remaining work and convergence from desired versus observed facts | Add dependency/readiness and realizer selection as modeled folds, not mutable `current_stage` |
| Goal drift could silently close partial work | Long turns lose the original purpose; a PR can close its item while only a fraction is implemented | Per-obligation `intent_revision`, Modified upserts, explicit Waived facts, and refusal to delete Ensured observations preserve drift honestly | Produce an antagonistic goal-audit receipt and make close depend on it |
| Publication and review have no producer | A local branch is not shareable work; future review may be GitHub or an internal API | The progress roster keeps both facts pending rather than guessing | Capture an immutable candidate, publish it, advertise an exact-head review target, and ingest the review verdict |
| Workspace/state storage is host-shaped | Worktrees, attempt state, raw events, and tmux are currently filesystem/process facts on one host | Instance boundaries name the roots instead of hiding them | Put storage and execution behind capability realizations so NFS, ephemeral volumes, tables, and object evidence can replace v0 adapters |
| Repeated Dispatch is idempotent at the wrong grain | A completed Agent plus retained session returns `already_live`, rendered as accepted, while Verify remains pending | The behavior is now understood and evidenced; no duplicate attempt was fabricated | Replace the row's primary button with `Advance`, derived from the next ready workflow obligation |

## 4. Root cause

The incidents are not independent UI bugs. One low-level actuator—“ensure a provider session for
this node”—was presented as if it controlled an end-to-end workflow. Facts from four grains were
then forced into one word:

- node eligibility;
- attempt identity;
- session/process lifecycle; and
- workflow obligation progress.

That is why `session · present`, `agent completed`, `dispatch · accepted`, and `Verify pending`
could all be true while the screen still felt contradictory.

The correction is not a larger imperative state machine. The correction is:

> A workflow is a desired recursive fact forest reconciled against observed evidence. An
> advancement control is a projection of the next admitted reconciliation obligation.

Session dispatch remains one realizer for one obligation. It no longer defines workflow
progress.

## 5. Foundation delivered by this branch

### 5.1 Portable host and apply boundary

`HostDashboardInstance` makes the deployment and execution context data: source repository,
origin/target identity, worktree/state/provider roots, network binding, and actuation posture.
The modeled apply preflights required capabilities and mutation boundaries, distinguishes normal
absence from refusal, realizes the complete source artifact set, and verifies exact-source
convergence. A lab can therefore be live and isolated without pretending to be production.

The remaining portability smell is any call site that reaches back to an srv1 constant instead
of consuming its instance. Those constants are acceptable v0 realizations; they are not
workflow authority.

### 5.2 Semantic dependencies through Ensure

`WorkItemExecutionContract` names validations such as a gunbc claim, a filtered Cargo test, or
formatting. It does not list incidental shell snippets. Required task capabilities derive from
those operations, and fixed dispatch mechanics derive separately from the platform.

`Ensure` stays abstract: it states a desired capability and a policy for realizing or observing
it. Git, tmux, jq, Cargo, rustc, rustfmt, and provider CLIs are realizations at a host boundary,
not special cases baked into the workflow. A missing dependency should therefore yield one of:

- already satisfied, with exact identity evidence;
- realizable by an admitted provisioning plan;
- requires bounded operator interaction, such as OAuth;
- refused, with the missing capability and policy reason.

It must never first appear as an agent's mysterious `command not found`.

### 5.3 Provider and process evidence

Codex is an explicit provider with a dedicated state root, login/version preflight, modeled
noninteractive `exec` arguments, workspace-write sandbox, never-ask approval, JSON events, and
persisted provider session. Complete raw JSONL remains evidence. A bounded jq projection feeds
the dashboard parser so command output volume cannot take down observation.

Provider event state and process state reconcile independently. A retained tmux session is not a
running process; a running process is not a completed provider turn; a completed provider turn
does not imply verification or publication.

Claude remains a provider shape, but its quota/error vocabulary is deliberately incomplete. An
unauthenticated status is evidence of unauthenticated state only. “Out of usage” becomes a typed
runtime/entitlement observation after an actual provider receipt demonstrates its grammar.

### 5.4 Recursive workflow facts

`WorkflowFactTree` carries a fact at every node and may decompose it into children.
`fold_workflow_fact_tree` is the single traversal. Membership, verdicts, observation tallies, and
remaining-work reports are algebras over that fold.

Desired and observed memberships reconcile generically:

- a missing or revised desired member is an upsert obligation;
- an unchanged member is monoid identity;
- removing an observed Ensured fact from intent refuses;
- an intentional scope removal remains as an explicit `WorkflowFactWaived`;
- changing `intent_revision` produces a Modified obligation without perturbing unchanged
  siblings.

The current seven-segment strip is a projection of that report. Pending means “no authoritative
observation,” not a stored pending receipt.

### 5.5 Honest UI observation

The dashboard independently observes sessions/processes and workflow attempts. It renders
provider activity, process exit, attempt branch, cleanup eligibility, and a seven-segment
workflow strip. Refusal is source-specific and loud. Pollers are single-flight.

This is sufficient to see what happened. It is not yet sufficient to ask the workflow what
should happen next.

## 6. Target model

### 6.1 Two structures, not one

Containment and dependency are different:

- **Containment tree:** why an obligation exists and how it decomposes. A parent workflow fact
  owns child workflow facts. The existing recursive fold covers this.
- **Dependency graph:** which facts must be satisfied or explicitly waived before an obligation
  is ready. Verification depends on an immutable candidate and agent completion; publication
  depends on successful verification; those are ordering edges, not parent/child claims.

Using a single ordered enum for both would make parallel frontend/backend children impossible
and would turn later goal revision into illegal state transitions.

The proposed desired carrier is conceptually:

```text
WorkflowObligationSpec
  key
  intent_revision
  dependencies: List<WorkflowFactKey>
  realization: DirectEffect | ChildWorkflow
  policy
```

The exact `.dag` shape should reuse existing `WorkflowFact`, capability, effect, plan, and
membership types rather than minting parallel vocabularies.

### 6.2 Readiness and advancement are derived

For each desired obligation, derive:

1. whether matching observed evidence at the same intent/candidate revision already satisfies it;
2. whether every dependency is satisfied or explicitly waived;
3. whether a matching realizer is already active;
4. whether the realizer's capabilities are admitted; and
5. whether policy permits automatic effect, operator-confirmed effect, or no effect.

The fold produces a ready set. v0 chooses the first ready obligation by a declared stable policy;
it does not infer order from UI position. Later planning supervision may choose among several
ready children without changing readiness semantics.

The server exposes a typed plan, conceptually:

```text
WorkflowAdvancePlan
  = WorkflowAlreadyConverged
  | WorkflowAdvanceActive { obligation, observation }
  | WorkflowAdvanceReady { obligation, realizer, confirmation_policy }
  | WorkflowAdvanceRefused { obligation?, reason }
```

The request carries at least workflow/node identity, attempt or candidate identity, approved
intent revision, and the obligation the operator saw. On POST, the server re-observes and
recomputes the plan:

- if the request is stale, it refuses with the changed fact;
- if the obligation is now satisfied, it returns an explicit no-op;
- if the matching realizer is active, it returns active evidence;
- only the current admitted plan may actuate.

The browser never chooses the next stage and never turns `already_live` into workflow
acceptance.

### 6.3 Idempotency is semantic

Idempotency keys bind:

```text
workflow identity
× obligation identity
× intent revision
× candidate revision where applicable
× realizer contract revision
```

Repeating a request with the same key returns the existing effect/receipt. A retry after a typed
failure creates a new realization attempt linked to the same obligation; it does not erase the
failed receipt. A goal revision changes semantic identity and therefore reconciles as Modified.

Attempt identity, session identity, branch identity, candidate commit, and review target are
related but not interchangeable.

### 6.4 Every realizer may recurse

A `MemberUpsert` can be discharged by a direct modeled effect or by a child planned workflow.
Examples:

- “ensure Codex authenticated” may spawn a provider-provisioning workflow;
- “implement frontend and backend” may spawn two implementation workflows;
- “address requested changes” may spawn a new implementation workflow bound to the reviewed
  candidate and comments;
- “run goal audit” may spawn a bounded independent review workflow.

The child publishes its evidence into the parent's observed fact forest. Green children do not
silently satisfy a pending parent: the parent still needs its own aggregation/acceptance receipt.

## 7. Receipt contracts for the minimum useful workflow

Each segment is a high-level fact and may decompose into the subfacts below. Every receipt binds
the workflow key, obligation key, intent revision, producer contract revision, timestamps, and
the exact subject it claims.

| Segment | Required observation | Realizer | Required refusal/failure distinctions |
| --- | --- | --- | --- |
| Environment | execution contract specified; all task/platform capabilities resolved with exact identity and Ensure receipt | observe or run admitted provisioning child workflow | missing, unprovisionable, interaction required, identity mismatch, probe inaccessible |
| Worktree | isolated worktree/branch exists at the admitted base revision; ownership and writable boundaries match instance | workspace effect | conflict, stale base, foreign worktree, create failure |
| Agent | provider configuration/auth admitted; raw event reference; bounded projected terminal event; compatible process exit | provider-turn child workflow | auth, entitlement/quota, provider refusal, malformed evidence, process failure, contradictory terminal facts |
| Verify | every validation from `WorkItemExecutionContract` ran in the attempt worktree against one immutable candidate revision; command/result/log references recorded | validation effect or child workflow | capability missing, command failure, timeout, stale candidate, incomplete validation set, unreadable evidence |
| Publish | candidate commit is immutable and reachable at an observed remote/artifact reference; readback equals the intended exact revision | publication adapter | dirty/uncommitted candidate, non-fast-forward/stale remote, auth, push failure, readback mismatch |
| Review | exact-candidate review target advertised and a decision observed; v0 adapter is a draft GitHub PR, future adapter may be an internal API | review adapter and, when needed, review child workflow | target missing, head changed, reviewer unavailable, changes requested, transport refusal |
| Audit | approved goal revision compared against diff, validation, publication, and review evidence; gaps and waivers enumerated; explicit verdict | independent/antagonistic audit child workflow | incomplete evidence, unapproved goal change, unmet obligation, invalid waiver, auditor refusal |

### 7.1 Candidate capture

Verification cannot be tied only to a mutable worktree. After agent completion, capture the
candidate identity:

- base revision;
- candidate tree/commit revision;
- dirty/untracked census;
- work item and intent revision;
- attempt(s) that contributed.

For v0, creating a local commit before Verify is recommended because it gives every downstream
receipt one immutable subject. If verification itself changes files, that produces a new
candidate and invalidates earlier verification by construction.

### 7.2 Publication and review

Publication and review are separate even when GitHub makes them feel like one click:

- Publication makes the exact candidate available outside the worker.
- Review advertises that candidate to an authority and records a decision.

The high-level Review fact may decompose into `ReviewTargetAdvertised` and `ReviewDecision`.
Opening a draft PR can produce the first child receipt; approval or requested changes produces
the second. An internal API can realize the same facts later.

Requested changes do not mutate the old attempt into running again. They create a new child
implementation workflow with the approved intent, reviewed candidate, comments, and remaining
reconciliation report as its brief.

### 7.3 Antagonistic goal audit

The audit must not trust the provider's final summary or the PR description. It compares:

- the original goal and every approved intent revision;
- acceptance/red controls and explicit waivers;
- actual candidate diff;
- validation receipts;
- publication and exact-head review receipts;
- remaining desired-versus-observed obligations.

Its result is at least:

```text
GoalSatisfied
GoalNeedsWork { gaps }
GoalChangeNeedsApproval { proposed_revision }
GoalAuditRefused { missing_or_contradictory_evidence }
```

Only `GoalSatisfied` at the current approved intent and candidate can close the work item.
`GoalNeedsWork` creates or exposes remaining obligations. A changed goal is not silently accepted
because the conversation drifted; a proposed revision must be approved, after which generic
reconciliation produces Modified/Added/Waived work.

This directly addresses the “PR merged, work item closed, only 25% finished” failure mode.

## 8. Provider and provisioning model

Authentication, entitlement, quota, runtime, and process state are independent provider facts:

```text
installation identity
authentication state
workspace trust / permission posture
entitlement or subscription state
quota / rate-limit state
runtime event outcome
persisted session/transcript reference
process lifecycle
```

`auth status == false` is valuable and should be shown, but it cannot be relabeled “out of
usage.” Conversely, an authenticated CLI may still emit an exhausted-quota runtime error. Each
provider adapter owns only error classes demonstrated by fixtures or live receipts. Unknown text
is retained and rendered as unknown/refused, never guessed into a stable type.

Provider setup is itself an Ensure reconciliation:

1. observe exact executable identity and state root;
2. observe authentication without exposing credentials;
3. if interaction is unavoidable, present the one exact login/device action to the operator;
4. resume observation after login;
5. prove a bounded non-mutating provider turn;
6. record the packaging-specific process fingerprint and event grammar.

The dashboard may advance a declared provisioning workflow. It must not opportunistically install
packages or rewrite a user's global auth state from a dispatch POST.

## 9. Storage and portability boundary

The workflow consumes capabilities, not local paths:

| Semantic store/capability | v0 realization | Future realization without semantic change |
| --- | --- | --- |
| source/candidate workspace | git worktree on host filesystem | ephemeral volume, NFS workspace, remote build workspace |
| small workflow facts and pointers | modeled attempt files/current pointer | transactional table |
| raw provider/validation evidence | append-only files | object store |
| active process/session | tmux on one host | container/job runtime |
| publication | git remote + GitHub | artifact service or internal change API |
| review | GitHub PR | internal escalation/review API |

Raw evidence and bounded observation remain separate in every realization. A fresh dashboard
process must reconstruct progress from durable facts; browser memory and a retained tmux pane are
never the authority.

## 10. UI contract

The primary control is **Advance**, with a label derived from the current plan:

| Derived situation | Primary presentation |
| --- | --- |
| no attempt; environment/workspace/agent plan admitted | `Start workflow` or the exact first obligation |
| Agent ready | `Start agent` |
| Agent active | `Agent working` (disabled, with activity) |
| Agent complete; Verify ready | `Run verification` |
| Verify complete; Publish ready | `Publish candidate` |
| Published; Review advertisement ready | `Open review` |
| Awaiting review decision | `Awaiting review` (disabled) |
| Review complete; Audit ready | `Run goal audit` |
| audit exposes gaps | `Continue work` with gap count |
| current approved workflow converged | `Complete` (no effect) |
| no realizer or unsafe plan | disabled/refusal with the exact missing capability or policy |

The compact seven-segment strip remains an observation projection. Every segment must expose a
text state—Pending, Active, Complete, Failed, Refused, or Waived—and a short evidence detail.
Color, border style, and dots are redundant channels, never the only explanation.

**Stop** and **Clear** remain secondary process-maintenance controls:

- Stop targets a currently running, ownership-proven process.
- Clear targets a retained exited session/container.
- Neither advances or rewinds workflow facts.
- Clearing runtime metadata must preserve immutable attempt, event, and stage receipts.

Low-level “dispatch another provider attempt” may remain in an advanced/retry disclosure when a
new agent realization is actually the plan. It is not the row's default action after Agent
completion.

## 11. Delivery plan

Each slice must be useful, model-first, and independently red-controlled.

### Slice A — advancement plan and truthful primary control

- Model dependency edges, realizer identity, ready-set fold, and `WorkflowAdvancePlan`.
- Serve the plan with attempt and intent revision.
- Replace the primary Dispatch button with the exact derived action.
- Initially reuse the existing dispatch realizer only when Agent is the ready obligation.
- Keep Stop/Clear secondary.

**RED:** with Agent Complete and Verify Pending, perturb retained session presence both ways. The
primary plan remains Verify; no arm can return `dispatch · accepted` or create an agent attempt.
A stale attempt/intent revision refuses before effect.

### Slice B — immutable candidate and verification

- Capture candidate identity.
- Project validation commands from the existing `WorkItemExecutionContract`.
- Run them in the exact attempt worktree/candidate environment.
- Persist bounded summaries plus raw log references.
- Reconcile the Verification fact.

**RED:** change one candidate byte after a green receipt; Verify becomes pending/stale. Remove one
declared validation; the completeness witness reds. A failed command cannot produce Complete.

This is the immediate next slice because the live attempt is already honestly waiting here.

### Slice C — publication

- Model a publication adapter and receipt.
- Require clean immutable candidate identity.
- Push an exact candidate reference and read it back.
- Make GitHub the first adapter without putting GitHub vocabulary in workflow facts.

**RED:** remote readback at a different revision refuses. A local branch name or successful push
exit without readback cannot satisfy Publication.

### Slice D — review advertisement and decision

- Open/bind a draft PR at the exact published candidate.
- Record advertisement separately from review decision.
- Observe exact-head approval, requested changes, closure, or transport refusal.
- Spawn a linked rework child workflow for requested changes.

**RED:** approval for an older head cannot satisfy the current Review fact. Rework cannot erase
the old review receipt.

### Slice E — goal audit and closure gate

- Model approved goal/acceptance revisions and waiver authority.
- Generate the intent-versus-actual audit input from facts, diff, and receipts.
- Run an independent antagonistic audit.
- Gate work-item closure on `GoalSatisfied`.

**RED:** delete or omit one accepted obligation while leaving the PR green; audit returns
NeedsWork and the item remains open. Unapproved goal shrink refuses.

### Slice F — recursive decomposition and planning supervision

- Let a realizer produce child planned workflows for implementation, provisioning, review
  rework, and audit.
- Aggregate child evidence into the parent without allowing green children to fabricate a green
  parent.
- Add a supervisor policy for several simultaneously ready obligations and proposed goal
  revisions.

**RED:** move an unchanged child to another parent; reconciliation emits one Modified upsert.
Plant a deep refusal; the parent cannot converge. Finish all children but omit the parent
acceptance receipt; the parent remains pending.

### Slice G — portability and provider parity

- Replace remaining srv1 ambient constants with instance/capability inputs.
- Add storage interfaces and a table/object-backed proof implementation when operationally
  needed.
- Complete Claude auth/trust/quota/runtime classification from real receipts.
- Add terminal-bounded observation and retention policy.
- Make lab provisioning and shareable endpoint discovery first-class.

**RED:** run the same desired workflow against two disjoint instances; no worktree, state,
provider home, port, or review identity may cross. An unknown provider error stays Unknown/
Refused.

## 12. Minimum useful workflow acceptance

This initiative is business-useful when all of the following are executable:

1. A roadmap item has an approved intent revision and explicit execution contract.
2. One primary control derives the next admitted obligation from desired versus observed facts.
3. Environment and workspace are ensured before provider execution, with actionable typed
   refusals.
4. Agent completion is reconstructed from durable provider/process evidence.
5. Verification runs the contract's complete validation set against an immutable candidate.
6. Publication exposes that exact candidate outside the worker and proves it by readback.
7. Review is advertised and decided against that exact candidate.
8. Goal audit compares approved intent with actual diff and receipts; incomplete work remains
   open.
9. A failure, refusal, retry, goal revision, waiver, or requested-change loop preserves history
   and reconciles without manual state repair.
10. A child workflow can satisfy a decomposed obligation while the parent retains its own
    acceptance fact.
11. Refreshing or restarting the dashboard reconstructs the same state from evidence.
12. A repeated or stale click is a typed no-op/refusal, never an unrelated provider dispatch.

## 13. Recommended v0 policy decisions

- Require an operator click before provider execution, publication, review advertisement, and
  goal closure. Pure observation and admitted local verification may later auto-advance.
- Use a draft GitHub PR as the first publication/review adapter; keep workflow facts transport
  neutral.
- Require explicit human approval for goal revision and waiver until a separate policy authority
  is modeled.
- Keep filesystem/tmux storage for v0, but introduce capability interfaces before a second
  backend is needed.
- Preserve complete raw evidence with bounded dashboard projections.
- Build Slice A, then Slice B. Do not add Publish/Review buttons that can only fabricate success.

## 14. Non-goals

- A globally distributed scheduler, transactional multi-host failover, or stateless workers.
- Automatic installation under privilege from an ordinary dashboard request.
- Guessing Claude quota semantics without a captured provider receipt.
- Treating an LLM summary, green PR check, pushed branch, or present session as goal completion.
- A hand-maintained transition function for every workflow shape.
- Automatic merge. Publication, review, audit, and merge policy remain separate facts.

## 15. This PR's completion claim

This PR may claim:

- portable, explicitly modeled dashboard/dispatch instances;
- exact dependency admission through execution contracts and Ensure;
- an operational Codex provider with captured/bounded evidence;
- correct separation of session, process, provider, cleanup, and workflow facts;
- recursive desired-versus-observed reconciliation;
- honest workflow progress through Agent completion; and
- this reviewed design and delivery sequence for the remaining workflow.

It may not claim end-to-end workflow execution. The deliberate close boundary is:

> Agent Complete; Verify, Publish, Review, and Audit Pending.

The next implementation must begin by coupling the primary control to a derived advancement plan,
then produce an exact-candidate verification receipt. That is the smallest slice that turns the
operator's latest click from an ambiguous session acknowledgement into real workflow progress.
