# Harness work lifecycle (sketch → structure)

Status: DRAFT v2 — owner-approved direction, external review folded in (2026-09-20).
Born of the srv2-deploy dispatch experiment: infrastructure proven end-to-end; the
missing thing is an authoritative lifecycle for coding work, so the known-mandatory
journey is owned by the workflow and never depends on an agent remembering it.

Guiding line: **make the known obligations unavoidable, keep agent judgment
flexible, and make every claimed improvement visible in the real execution path.**

## Axiom

Work is **delivered** per the delivery contract declared at intake:

- a **code-change task** is delivered only when its submitted candidate is
  published to its PR;
- an **orientation task** delivers an orientation result;
- a **review task** delivers a review.

The worker may not redefine a code-change task as "analysis completed" to escape
publication; orientation and review processes are never forced to manufacture PRs.

The five evidence states are distinct and never collapse:

**Ready ≠ Verified ≠ Published ≠ Approved ≠ Merged**

## Anchors

| Anchor | Owner | Structural (workflow) | Judgment (worker) |
|---|---|---|---|
| Intake | roadmap authority | accepted task + delivery contract + context bundle + base sha | — |
| Orientation | workflow (centering contract) | task, source pointers, constraints, exemplars, verification route; a typed way to report missing context | understand the problem; name uncertainty |
| Scoped editing | worker | bound workspace + permitted effects; nothing outside the scope is reachable | implementation, local debugging, local test iteration |
| Candidate handoff | worker submits, workflow binds | the handoff operation CAPTURES the candidate; the harness materializes its commit identity and carries the acknowledgment onto exactly that candidate | readiness claim + explanation + limitations; or blocked/needs-context |
| Verification | workflow (belt) | declared oracle runs against the captured head in a detached checkout; receipt retained | interpreting failure (feeds the next editing pass) |
| Publication | workflow (helper, credentialed) | push captured head, create-or-update the one PR per logical work item, receipt bound to the OBSERVED PR | PR body explanation |
| Review | owner/fleet | feedback bound to the reviewed head; changes route back as a new candidate | approval/merge decision (human) |

## Rules

1. **Delivery obligation.** A code-change task is never reported complete while a
   publication obligation is outstanding. `Ready` (submitted) enables publication
   and leaves the obligation open; passing verification is not publication;
   approval and merge are later, separate states.
2. **Submission ≠ checkpoint, and binds the CAPTURED candidate.** The belt may
   commit a dirty tree as a checkpoint (preservation only, no readiness claim).
   A submission is a declared act whose handoff operation captures the candidate
   as it stands THEN; the acknowledgment binds to that capture — never to
   whatever a later belt tick sweeps up. Budget exhaustion without a submission =
   `Yielded`, not ready. "A later edit inherits nothing" means old
   candidate-specific evidence does not admit the new candidate; the old evidence
   is PRESERVED, current eligibility recomputed.
3. **Head binding.** Verification and review bind to an exact commit. Revisions
   update the SAME PR (the obligation belongs to the logical work item).
4. **Authority allocation is modeled and enforced by effects.** Who may commit,
   publish, approve is a model fact; the worker's available effects, credentials,
   the execution path, AND the rendered instruction all derive from it. Generating
   "the worker may not publish" enforces nothing by itself — the selected effects
   must make the restriction true; the prompt only explains it.
5. **The worker may iterate; the workflow owns obligations.** Local test/debug
   loops are the worker's business. The authoritative verification, its receipt,
   publication, review routing, and per-failure-class retry ownership are the
   workflow's. Failure classes route by owner: CandidateFault → editing
   (continuation with verdict); VerificationInfra → retry/repair the verifier;
   PublicationInfra → blocked-on-publication with a named owner and next action.
   Never fold all three back onto the worker.
6. **Typed blockers have owners.** `BlockedOnPublication(cause)` preserves the
   pending obligation across restart, owns retry/escalation, and reconciles
   uncertain remote success before creating anything: if the PR was created but
   the local process died before recording it, resumption DISCOVERS the existing
   PR — it never creates a duplicate.
7. **Tests are behavioral, at the real execution boundary.** The suite must show:
   terminal response without submission is not ready; submission + pass discharges
   into publication with no discretionary request; publication failure stays
   visible and resumable; uncertain remote success reconciles without a duplicate
   PR; a later edit cannot reuse a prior head's evidence; the worker cannot invoke
   an effect the model assigns to the harness. Marker/projection comparisons are
   secondary.
8. **Agent-facing documents are a justified argument, not just generated text.**
   Once documentation is an input to execution, its correctness is part of the
   work's correctness. Every consequential instruction must have traceable
   support: WHY it must happen (applicable requirement/dependency), WHO may/must
   (modeled authority allocation), WHEN it applies (task/role/stage/conditions),
   HOW (the actual operation and admitted route), WHAT ESTABLISHES it happened
   (observation bound to its subject), and WHAT IF it cannot (typed blocked/
   repair/retry/escalation). Explanatory prose may stay authored, but it may not
   introduce an unmodeled permission, obligation, exception, or success claim.
   Deriving prompt and docs from one source removes drift; it does NOT remove
   consistent-wrongness — composition is checked, not just generation.
9. **The composed context must be consistent, not just each document.** The load
   phase admits an applicable instruction set; individually valid documents can
   compose into contradiction (case on file: worker prompt forbids claim_batch,
   DESIGN.md teaches it — the repair is declaring the SCOPE distinction in the
   model: the verifier uses the batch route, the worker submits candidates and
   never runs authoritative verification; both documents then express it
   correctly). Ordering one instruction before another does not repair meaning.
   The check covers brief, orientation, stage instructions, available tools, and
   observations: "publish the PR" is not a plan for a worker with neither
   publication authority nor a handoff to its owner.
10. **Policy, observed fact, and plan never render as the same standing.**
    "Publication is required", "publication succeeded", and "we intend to
    implement publication" are three different premises. Stage instructions
    reflect what is established NOW, not the aspirational pipeline. (The
    experiment's own trace called oracle-green "the full loop" with the PRs
    unopened — that class of unsupported conclusion is what this rule exists to
    expose.)
11. **Contradiction reporting is a first-class, consumed route.** The worker
    contract: when applicable instructions conflict, or a required conclusion
    lacks its stated support, report the conflict BEFORE acting on anything that
    depends on resolving it — never silently choose a convenient interpretation
    or claim satisfaction. The report names the conflicting statements and their
    sources, the affected operation, and the missing decision/evidence; a
    suggested reconciliation is a proposal until the responsible authority
    accepts it. The workflow preserves the affected obligation, surfaces the
    blocker, and routes it to the owner of the contradictory instruction or
    missing capability — a warning buried in a transcript is not consumption.
    Two complementary protections: modeled contradictions are caught BEFORE
    dispatch where possible; ambiguities found in execution are returned during
    it. The agent is an additional reviewer of the supplied argument, not the
    sole consistency checker, and it must never absorb our defect and disguise
    it as successful execution.

## The load phase (required dependency of every model-backed session)

Before a model-backed process starts — worker, planner, or reviewer — the harness
resolves its declared context manifest against the applicable source snapshot,
assembles the required contents in the declared order, and supplies that assembly
to the ACTUAL model request.

Honest boundaries (conceptual, not new modules):

- **Context declared**: which sources and instructions are required.
- **Context assembled**: those contents were resolved from the source snapshot,
  in order.
- **Context supplied**: the real provider request consumed the assembly (verified
  against the request as sent, including provider rendering and tool definitions —
  not against an intermediate prompt string).
- **Orientation produced**: the task-specific interpretation produced its output.
  Load is the common foundation; orientation applies it to this task. Neither
  proves the agent understood anything.

The initial manifest: (1) full DESIGN.md (the canonical READING surface;
AGENTS.md/CLAUDE.md/README.md symlink to it), (2) genuinely additional standing
instructions — NOT a duplicate of the operational-essentials section already
inside (1), (3) the anchor-relevant context for this session. The reported file
size is a reason to measure token usage and context fit, not authorization to
substitute a smaller subset.

Ordering law: stable project prefix first, role- and task-specific material last,
wherever the provider request format permits.

## Alignment ladder (owner direction, 2026-09-21): cadenced alignment at every level

Alignment is a RECURRING workflow responsibility across three scopes: the root
principles, the parent project's purpose and approach, and the task's outcome.
Functional decomposition exists so this is checkable: a decomposition must
PRESERVE each task's contribution to the levels above it — every node carries not
just its own contract but the derived statement of what that contract contributes
to its parent's intent. Alignment checks at any level are readings against that
carried contribution, never a fresh interpretation invented at check time.

A budget is NOT a failure measure — it is the alignment cadence. When a turn
exhausts its budget the question is never "did it fail" but "is the work still
aligned." Orientation establishes the initial grounding; alignment MAINTAINS that
grounding as the work develops. Verification, publication, and review remain
distinct obligations — an alignment check never discharges them and they never
discharge it.

The levels — each a different authority, and a check at one level never
substitutes for another:

- **L0 root principles** — DESIGN.md (the axioms and what follows from them).
  Question: does the work's approach still conform (single authority, fail-closed,
  displaced cost, no workaround)? Carried by the load phase.
- **L1 project intent** — the decomposition node's declared purpose, approach, and
  this task's carried contribution to it. Question: does the work still serve the
  arc's intent, by its declared approach?
- **L2 task contract** — the node's boundary / red control / handback plus the
  pinned witness. Question: is the current diff advancing THIS contract, or
  drifting (scope creep, adjacent improvements, goalpost edits)?
- **L3 the live work** — the candidate and its submission against L2: the
  submission's explanation is the worker's own alignment claim; verification and
  goal audit are the independent ones.

Cadence points — each cheap, each evidence-based, each producing an alignment
receipt:

1. **Dispatch** — orientation + load phase (initial alignment).
2. **Each work interval / continuation turn** — a brief direction check. It
   CONSUMES the actual work and its observations (the current diff, the receipts
   produced so far, the worker's own account), never the worker's self-report
   alone. Its verdict is a MAGNITUDINAL standing — a Fermi estimate, order-of-
   magnitude and evidence-cited, never false precision (owner direction,
   2026-09-21): **ALIGNED (+0/−0)** — no measurable drift; **MISALIGNED
   (direction)** — a named drift with a named correction; **EXTREMELY
   MISALIGNED** — the work is off the task/project contract; **CATASTROPHICALLY
   MISALIGNED** — the work violates a level above it (a root principle, the
   project's declared approach). Each magnitude maps to exactly one action:
   ALIGNED → justified continuation (evidence named); MISALIGNED → concrete steer
   binding the next interval; EXTREMELY → escalate to the supervisor (plan-defect
   territory; the cadence interval drops to its floor); CATASTROPHIC → stop the
   line (the supervisor is convened immediately; work does not continue on the
   candidate). The outcome AFFECTS the next work performed: a check whose answer
   cannot change what happens next is ceremony, and ceremony here is worse than
   absence because it reads as diligence.
3. **Budget checkpoints** — the Yielded path: the supervisor's adjudication IS the
   alignment verdict (Nominal = aligned but slow; Plan defect = misaligned by plan;
   Stuck = misaligned by reality).
4. **Submission and review/audit** — the worker's claim, then the independent
   verdicts.

Rules:

- **A direction check is never pressure to manufacture completion.** The check
  measures alignment and direction, not proximity to done; a check that punishes
  an honest "not yet converging" teaches workers to fabricate convergence, which
  is the same failure as budget-as-failure wearing a friendlier face. "Aligned
  and still working" is a complete, acceptable answer.
- Alignment content is DERIVED from the modeled artifacts — DESIGN.md from
  gunbc.design_document, arc intent and task contribution from the node's declared
  fields and its parent's, the task contract from the node and its pinned
  contract — never hand-typed prose that can drift (the submission-schema defect
  is the standing specimen of prose drift).
- **The cadence is adaptive — a step function over verdicts (owner direction,
  2026-09-21).** Each level's interval starts at its magnitudinal base (scaled by
  the chain's depth — more levels above a task means more to stay aligned to),
  then steps with the VERDICT'S MAGNITUDE: ALIGNED widens that level's interval by
  a declared step (+2 turns), MISALIGNED shortens it by the same step (sooner
  re-check), EXTREMELY drops the interval to its floor outright, and
  CATASTROPHICALLY stops the line rather than tuning anything. A RoutedConflict
  (instructions disagree — not a worker signal) does not tune cadence at all. Every
  level's interval stays bounded: never rarer than its magnitudinal ceiling (root
  principles still re-check at their epoch regardless of streaks), never more
  frequent than a declared floor that keeps probe cost subordinate to work. The
  cadence state is per attempt-lineage, recorded on the alignment receipt (the
  interval used and the verdict that moved it), so a later reader can replay why
  the checks came when they did. The step reads only typed verdicts — never
  vibes — and verification/publication/review gates are untouched by it: cadence
  tunes HOW OFTEN we look, never what advancing requires.
- Alignment is not re-planning: the probe measures drift against existing
  contracts; re-deriving the plan is the supervisor's job on escalation.
- What it is not: not a per-round interrupt, not a second full orientation, and
  never a vibe check that can stall progress unbounded.

First mechanical slice: the continuation brief carries the projected three-level
preamble; the answer rides in the submission's explanation; the belt records the
cadence point. Deeper slices: alignment answers as a typed per-turn artifact
consuming the diff and receipts, not just self-report; supervisor verdicts citing
alignment receipts.

## Warm-prefix reuse

**Warm-prefix reuse is an optimization to qualify and measure, not a guarantee.**
vLLM prefix caching matches processed token prefixes of resident blocks; eviction
and data-parallel cache scoping mean actual reuse depends on the serving route
and cache state, and must be OBSERVED. A canonical shared prefix makes sibling
requests ELIGIBLE for reuse — that is the whole claim. Consequences:

- No warm-up agent, no conversation-fork mechanism: the first real request
  supplies the initial demand; reuse shares eligible prefix computation, never
  another session's mutable conversation or conclusions.
- **Missing required context blocks the request. A cache miss must never weaken
  the context contract** — the declared cold route is taken when admitted.
- Cache savings are reported only when measured; prefill savings do not bound
  generation cost.
- No "stability class" field until it has a concrete consumer; exact source
  identity + declared order are what matter now.

## Slice 3's authority shape (the extdeps distinction)

DESIGN.md is the reading surface, not a second command authority. The intended
derivation is:

**modeled operation + applicable policy → executable invocation, documentation,
and worker-facing explanation**

— not one paragraph copied into both DESIGN.md and the prompt while execution is
authored elsewhere. Batching rules, witness population selection, wet-execution
requirement, executable provenance, and memory-budget requirements govern the
chosen execution ROUTE; the docs and prompt explain that same route. External
interface shapes stay in extdeps; the harness lifecycle composition and policy
stay in the workflow; the lifecycle does not move into extdeps to look modeled.
Slice 3's acceptance is the real execution boundary: at least one control fails
when the production connection between model facts and the session request is
removed.

## Resource policy (subordinate, not organizing)

**Budgets flow down the decomposition DAG; they are never flat.** A root carries
a declared bandwidth (rounds/tokens/wall-clock — the total effort the owner
allocates to that arc). A decomposition node allocates fractions of its
bandwidth to its children; a leaf's worker budget is what its parent dealt it,
not a global constant. Escalation sessions run one to two magnitudes above the
leaf budget they supervise — the supervisor must be able to re-read the
checkpoint, adjudicate, and still have runway to act, which a same-size budget
cannot do. Effort expended is therefore proportional to the project's bandwidth
by construction: big arcs afford big attempts; a leaf of a small chore node
gets a small one. Every budget figure is a declared allocation on the node,
visible on the dashboard — never a literal buried in harness code.

Budget exhaustion is NEVER a bare "task failed". It produces a defined outcome:

1. **Checkpoint** — the attempt state (transcript, worktree, receipts-so-far) is
   preserved as a checkpoint. If the worker left a submission, it advances
   normally; without one the attempt is `Yielded`, not failed.
2. **Escalation** — a Yielded attempt escalates to the supervisor session for
   the node it was dispatched under (leaf → its parent decomposition node's
   supervisor; the root arc's supervisor is the top). The supervisor resumes
   from the checkpoint (the attempt's warm context/state — a KV checkpoint,
   not a cold re-read) and adjudicates one of three routes:
   - **Nominal** — the worker was progressing; issue a continuation turn with a
     renewed budget. (Cheap tasks legitimately finishing late take this route;
     attempt 1 of wave 1 would have.)
   - **Plan defect** — the brief/centering sent the worker wrong (scope too big,
     exemplar missing, contradictory instructions). Routes to the node's owner
     as a plan defect — the worker is not re-dispatched against a known-bad
     plan, and rule 11's contradiction machinery carries it.
   - **Genuinely stuck** — the task itself can't advance (missing capability,
     external blocker). Typed `Blocked` with the supervisor's stated cause.
3. **Supervisor verdicts are evidence**: the route, its basis, and the
   continuation count are recorded per attempt. Continuation caps remain
   tunable policy attached to the anchor; a cap hit escalates again rather than
   failing silently.

The supervisor is a modeled session role (like worker/reviewer/auditor), not a
human pager: it runs on the same harness, with its own guidance contract and the
load phase applied. Escalation depth is bounded by the decomposition DAG — a
supervisor that itself yields escalates to its parent, terminating at the root.

## Metrics

Outcome metrics: cost per delivered code-change result, publication success rate,
operator interventions per delivery, review rework per delivery. Rounds-per-diff
is retained as a DIAGNOSTIC (it is how orientation/repair cost movements are
observed); it is no longer a success metric.

## Substrate ruling (owner, 2026-09-20): gunbc SCM inside, git at the edge

The internal process substrate uses gunbc SCM (`dag/gunbc/scm/*`: object store,
write spine, staging, proposals) for everything it can cover: candidate capture,
the submission's head binding, attempt state identity, and the fanin merge of
worker candidates into the project's integration head (SCM proposals, typed
conflicts, modeled merge verdict — not textual three-way git merge). Git is used
ONLY at the final publish layer: translating the project's integration head to a
git push + GitHub PR. The repo being git-hosted and GitHub publication are the
edge; nothing internal speaks git where an SCM operation exists. Belt commit/
verify/publish seams sit behind an explicit interface so the SCM realization is
the authority and the git realization is edge-only.

## Delivery structure: recursive decomposition (owner ruling 2026-09-20)

There is no separate "project" mechanism. A project IS a node — one whose work is
the completion of its children. The work graph is a DAG with the same structure
as the substrate itself: any node may decompose into child nodes (recursively),
a child may serve more than one parent (DAG, not tree — the import graph's law,
acyclicity, is the only structural rule), and every dispatchable node must have
at least one parent. Totally flat items — no parent — are not dispatchable work;
dispatch admission refuses them with a typed cause.

Consequences:

- **Fanout**: dispatching a decomposition node dispatches its ready descendants
  (capacity-bounded), each attempt producing a verified candidate.
- **Fanin is a fold, recursively**: a decomposition node's own candidate is the
  integration of its children's verified candidates (SCM proposal merges, typed
  conflicts); its evidence is complete when every child is delivered. The fold
  bottoms out at leaf nodes (ordinary single-worker tasks).
- **Delivery obligation binds at the grain you publish**: a leaf is a
  decomposition of one (the degenerate case — per-attempt PR behavior falls out
  without special-casing); a batch like the shell→dag ten publishes ONE PR at
  the decomposition node's integration head; a whole arc could publish one PR
  at its root.
- Evidence states still bind per-candidate-head at every grain; a parent head is
  Published only when its integration carries every included child's
  verification receipt.

Slice 2's publication machinery is unaffected in mechanism — the obligation
subject is "the node whose delivery contract was dispatched", at any depth.

## Distribution layer (owner direction, 2026-09-20): stateless executors, fabric-resident state

Agents are stateless step functions over stored state; any conforming executor on
the fabric can run the next slice of any attempt. The worker's signature:

> input: (attempt identity, transcript head ref, workspace ref, budget slice)
> output: appended events + typed terminal (Continue | Yielded | Submitted | Blocked)

Nothing an attempt needs may live only in an executor's local filesystem. Attempt
state maps onto the fabric's storage tiers — each class into the store that is
already shaped for it, never a new per-lane persistence invention:

| State class | Fabric storage home | Status (2026-09-20) |
|---|---|---|
| Transcript events (append-only, ordered, replayed on resume) | Fabric event log — the transcript becomes a hash-chained event stream with a head ref | Log exists; attempt transcripts not yet written to it |
| Indexes over events (transcript heads, attempt-by-node, executor fencing) | The fabric DB (gunbc#11723 — shared dependency with the group-A seat pool) | In flight |
| Workspace / candidate / receipt objects (immutable, content-addressed) | SCM object store — the belt's candidate capture already writes here | Landed (e9f0683adb) |
| Bulk artifacts (worker logs, large dumps) | Artifact store (artifact_store_fs realization today; storage-tiering policy later) | Exists |
| Executor leases / seats | Fabric cells (fabric_cell_acquire) | Exists for fleet work, not yet for dispatch |

Rules:

1. **Exactly one active executor per attempt**, fenced by a fabric lease. Two
   executors continuing one transcript is the corruption case; the fence, not
   politeness, prevents it.
2. **The belt observes fabric state, not host state.** Today it polls
   attempt-state dirs on the dashboard host; the same facts read from the store
   make it restart-safe and host-independent (a belt on srv1 cannot see srv2's
   disk).
3. **Placement is a claim, not a hardcode.** Dispatch mints work + state refs;
   executors (systemd units today, microVMs later) claim seats through the cell
   machinery. A microVM worker is pool member N behind this seam — never a
   bespoke spawn path.
4. **Executor switches pay cold prefill, stated honestly.** Prefix caches are
   per serving engine; a resumed attempt on a different executor recomputes its
   prefill. The slice-3 stable load-phase prefix is what keeps that cheap; it is
   a cost to measure, not a reason to pin attempts to hosts.
5. **Transcripts compact.** Long attempts append a compaction event
   (summary + reference to the compacted range) rather than growing unboundedly;
   the resume reads the compaction head.
6. **Transcripts carry private facts.** Store placement and access for attempt
   state inherit the private-facts boundary that ctrl/ used to be; this is a
   placement decision, not an afterthought.

The supervisor/escalation design (Resource policy) rides this substrate: a
supervisor session is just another executor reading the same refs — "resume
from the KV checkpoint" is, mechanically, load transcript head + workspace ref +
a warm prefix.

Graduated path, each step independently useful:

1. **State into the store** — transcripts and receipts content-addressed
   alongside candidate capture; the belt dual-reads (store first, disk
   fallback).
2. **Checkpoint/resume on one host** — a budget-exhausted worker exits Yielded
   with a transcript head; a fresh unit resumes from the ref. (The continuation
   routing built 2026-09-20 is this step's decision layer.)
3. **Executor pool seam** — spawn targets a claimed seat instead of the
   dashboard host by name.
4. **Second executor + microVM** — another host or a microVM joins the pool;
   nothing above this line changes.

## Workflow execution (owner ruling, 2026-09-21): durable obligations, replaceable processes

Repair the current failure, but the global belt pass is NOT the permanent
architecture. The durable facts are: accepted work, each outstanding obligation
(capture, verification, publication, review, audit, alignment check), its state,
its due time, and its last result or located failure. Processes, systemd units,
timers, and page bodies are replaceable realizations over those facts — never
the facts themselves.

1. **Per-item advancement, per-operation persistence.** Each work item's
   operations advance when INDEPENDENTLY eligible — not when a global pass
   reaches them. Every operation's result AND its located failure persists at
   the moment it is produced, never at pass end. A blocked capture is an
   item-scoped fact by construction: it must never prevent unrelated
   verification, publication, or alignment from advancing. And a required
   operation never disappears as a legacy skip — an unperformable operation
   stays outstanding with a typed cause and an owner, because a skip that sinks
   an obligation is a silent drop wearing a receipt.
2. **Scheduling: wake-ups, due times, bounded reconciliation.** Targeted
   wake-ups (submission written, worker terminal, receipt landed) advance the
   affected item's next operation promptly — the timer is not the critical
   path. Due times live ON obligations (retry-after, the alignment cadence) so
   anything time-driven is data, not a loop's period. Bounded reconciliation
   (the sweep) is the RECOVERY backstop that re-derives from durable state —
   never the primary driver. Missing a notification or restarting a controller
   is a non-event: recovery replays from the durable obligations and no user
   action is ever required to resume.
3. **The page: fast, honest, decoupled.** Observations are decoupled from
   execution success — a failed, poisoned, or hung item still renders
   truthfully. Staleness is EXPLICIT (rendered-at and observed-at per region),
   never implied by a fresh-looking body.
4. **Acceptance demonstrations.** (a) One submitted change reaches its PR
   while another attempt is poisoned or hung. (b) Restart and executor-loss
   recovery through the same path — kill the controller mid-obligation and
   watch recovery resume it without user action.

5. **The contract (owner ruling, 2026-09-21): durable, versioned handoff of
   responsibility across execution boundaries.** This generalizes clauses 1-4
   beyond the belt - and it does NOT mean queueing every local function call.
   The authority is a modeled module in the repo (lands as
   `gunbc.workflow.durable_handoff` with its first real consumer); this document
   REFERENCES it and must never become the only place the rule exists. The
   contract:
   - **Inbound**: a handler performs bounded admission, then confirmed durable
     acceptance, and returns an operation reference - it finishes there. The
     accepted work, its required inputs, and its continuation survive the caller
     and the executing process.
   - **Processing**: consumes identified inputs and advances state
     conditionally; settlement records an attributable result, a refusal, or a
     durable transfer of responsibility.
   - **Three separate facts**: transport acknowledgment, execution-resource
     release, and historical retention are distinct - a delivery may be settled
     once durable responsibility transfers, but settlement never erases the
     obligation or the information recovery needs.
   - **Outbound**: the same contract applies to effects at the upstream
     boundary, with guarantees limited to what the upstream API actually
     provides - modeled as observed guarantees, never invented ones.
   - **Composition, not invention**: the contract composes the existing work,
     storage, identity, ownership, and observation authorities.
6. **The missing conformance question lands with the first real inbound
   consumer.** The §3b roster (gunbc.design_argument conformance_domains ->
   roadmap_review_criteria) has no row whose question is "does this boundary
   hand off responsibility durably - bounded admission, durable acceptance,
   conditional advance, honest settlement?" - so this week's failure classes had
   no mechanical reviewer: a pass-killing item failure, a hung tick freezing a
   pipeline, an undecidable artifact sinking a candidate, a fresh-looking body
   hiding stale observations. The boundary with neighbors: leasing (§3b) owns
   the lease as a PRIMITIVE; fabric/compute owns allocation; the new row owns
   the handoff architecture that uses them. Per §3b's own rule a row names homes
   that EXIST: the durable_handoff authority lands first with its executable
   controls, and the conformance row is added naming it plus the lease
   authorities - until then this is a modeling obligation, not a reviewer. These
   classes are its seed tells.

## The belt demoted (owner ruling, 2026-09-22): wake-up and anti-entropy only

The belt tick is NOT the workflow authority, NOT the unit of execution, and NOT
the source of current state. It is demoted to two duties: delivering wake-ups
and bounded anti-entropy repair. Every accepted issue, attempt, candidate, and
external observation has its OWN durable authority and version — that is where
state lives.

The working shape:

- An event or a due refresh recomputes the SMALLEST affected projection,
  conditionally publishes it (publish only when the version moved — never
  republish unchanged), and creates ONLY the next eligible obligation.
- Duplicate, lost, and out-of-order wake-ups are harmless: every consumer is
  idempotent and version-checked, so a wake-up is a hint, never a fact.
- The expensive operations (verification, escalation convening, integration,
  publication) are per-item obligations invoked on wake-up — never phases of a
  monolithic pass whose completion bound the corpus scale will always
  eventually break. The 90-minute tick that dies unfinished is the standing
  specimen.

## The frontend as an issue tracker (owner ruling, 2026-09-22)

**Issue intake is fabric-resident (owner ruling, 2026-09-23).** Created issues
never touch the git tree: content versions go to the SCM object store, intake and
state-change events to the fabric event log, and the current-state index
(issue id → current version + standing) to the fabric DB (the shared in-flight
dependency — group-A seats and attempt-state indexing consume the same
substrate). The roadmap projection consumes the fabric-resident intake as an
overlay onto the .dag-authored nodes; the two populations present identically
downstream. `POST /issues` is the first real inbound consumer of
`gunbc.workflow.durable_handoff`: bounded admission (fields valid, a parent
required — the no-flat-items ruling), durable acceptance, an issue reference
back.

The frontend IS an issue tracker over the same authorities — and the interface
itself is an external authority, modeled in extdeps with its real vocabulary:
**Google's Issue Tracker (Buganizer)** (`extdeps.google.issue_tracker` — Issue,
Component, Status, Assignee, CC, Star, Upvote, Hotlist, SavedSearch,
BookmarkGroup, BlockingRelation, and its stock sidebar views: Assigned to me,
Starred by me, Upvoted by me, CC'd to me, Collaborating, Reported by me, To be
verified, plus Hotlists and Browse components). The roadmap frontend is a
MAPPING from our domain authorities onto that modeled interface (interface in
extdeps; mapping and policy in the workflow) — never a homegrown tracker shape.

- **Components express domain placement; parent issues express decomposition;
  blocking edges express dependencies; attempts express execution history.**
- The default view is a searchable issue table. Each issue has a detail page
  and a separate Work tab. A project-tree view uses nested title-in-progress
  bars (the landed taskbar component, promoted to the tree grammar).
- **Four facts stay separate**: planning status, the current workflow
  obligation, the attempt outcome, and the delivery evidence. The page derives
  a concise PRESENT-TENSE explanation from them ("verifying candidate
  2ccf4dc72f", "blocked: turn 2's artifact undecodable; turn 3 re-declared") —
  it NEVER flattens them into one label like "Submission failed" plus a row of
  misleading pending stages. A pending stage names what it waits on, not just
  that it waits.

## Slices and their production exit criteria

| Slice | What must be true before calling it complete |
|---|---|
| 1. Orientation requirement (in flight: agent-13, roadmap_centering.dag) | The real dispatch path consumes the applicable orientation input; absent or inapplicable orientation cannot be bypassed by a presence flag. |
| 2. Handoff and delivery | A submitted candidate is captured, authoritatively verified, and published to its associated PR without another discretionary request. Yield, publication failure, restart, and uncertain remote success all remain truthful. |
| 3. Justified instruction contract + load phase | The real session request consumes the ordered, source-bound context. Consequential build/test and submission/delivery claims derive from the same modeled facts that govern execution (traceable support per rule 8); the composed instruction set is consistent per rule 9 (the claim_batch scope distinction is the demonstration case, declared in the model and expressed correctly in both surfaces); the worker has the rule-11 contradiction route and it is consumed. At least one control catches a DELIBERATELY INTRODUCED conflict. |
| 4. Evidence-driven refinement | Actual dispatch records show where cost and failures occur. Improvements are evaluated against those records, including unsuccessful attempts and review rework. Cache savings reported only when measured. |

Serialization note: `roadmap_belt_actuate.dag` edits serialize behind agent-13;
preparation, source inspection, acceptance-case design, and non-conflicting work
do not.

## The review deliverable

The owner-facing PR contains the harness changes ACTUALLY EXERCISED (identifying
the source head that ran), links the resulting task PRs and evidence, and shows
at least one real dispatch reaching its task PR: this dispatch used this source
and context, submitted this candidate, verified this head, created/updated this
PR, reached this review state — plus which failure controls were exercised and
which capabilities remain unproven. A dashboard restart is an operational step,
not evidence. Plan text and dashboard status are not the deliverable.
