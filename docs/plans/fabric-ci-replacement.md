# Replacing GitHub Actions with the fabric: the plan

**Status: APPROVED IN DIRECTION, amended. Sign-off received 2026-08-19 — "I approve the one-PR,
no-shadow replacement direction, but not the plan exactly as written" — with eight blocking
corrections and sixteen acceptance conditions, recorded in §11–§13 below. Earlier sections are kept
as authored, with their defects registered rather than edited away.**

**Original framing:** Operator direction (2026-08-19): do the migration as one
PR, no long shadow process. That is the replacement doctrine's *default* — delete-first, one atomic
authority transition — rather than the gap-intolerant carve-out I had proposed. The plan below is
written to that ruling.

## 1. What is actually being replaced

GitHub Actions is a **fused authority**, and the recut program has just spent four cuts learning to
recognise one. It is simultaneously:

```
event source      — a push or PR opened/synchronised
allocator         — which machine, when, and how many at once
executor          — checkout, toolchain, build, run
status sink       — the red or green a human and the merge button read
log store         — where the output goes
concurrency       — cancel the superseded run
```

The fabric has authority over **allocation and execution**. It has no authority over webhooks or a
log UI, and pretending otherwise is how an MVP becomes a project. So this is the same move the side
chat ruled for `ComputeOffer` in §16 of the recut program: decompose the fused carrier, replace the
parts that have a home, and leave the rest cited rather than absorbed.

## 2. The contract to preserve, measured from `.github/workflows/witnesses.yml`

**The minimum replacement is not the smallest thing that runs the floor.** It must preserve every
refusal, or it has erased a correctness distinction rather than completed a migration. Measured
from the live workflow rather than remembered:

| behaviour | current | must survive |
| --- | --- | --- |
| triggers | `workflow_dispatch`, push to `main`, PR to `main` (`opened`, `synchronize`, `reopened`, `ready_for_review`) | yes |
| runner | `[self-hosted, linux, arm64]` | yes — via the Cut C seam, not a literal |
| timeout | 180 minutes | yes, as a typed bound |
| concurrency | group per PR number or run id; **`cancel-in-progress` for PRs only** | yes — main runs never cancel |
| checkout | `fetch-depth: 0` (full history) | yes |
| toolchain | `setup-rust-toolchain@v1.16.0`, `cache: false` | yes |
| build | `claim_executor`, `gunbc`, `v1_src_dag_parse` | yes |
| gate 1 | `v1_src_dag_parse` — src/v1 `.dag` sources parse, **a separate step that must pass** | yes |
| gate 2 | `claim_executor --required-floor --source-root dag --source-root src/v2` | yes |
| env | `GUNBC_EXPECTED_RED_ROSTER_JOIN=expected_red_roster_join.tsv` | yes |
| failure | non-zero exit on any step is red | yes |

Two of these are easy to lose silently and are called out for that reason: **the v1 parse gate is a
separate step** (it was invisible for 98 minutes once already, gunbc#8466 → #8519), and
**cancellation is asymmetric** — cancelling a main run would lose the only unconditional signal
that main is green.

## 3. Why this is the right work, not a side quest

`product.fabric.*` is 80 declarations across six modules with **no production consumer**. Every
guarantee it carries is currently type-level. This replacement makes the fabric load-bearing on the
one workload we already own end to end, which is the difference between a modeled market and a
market.

It is also the **forcing function for Cut D**. Cut D stalled on two authorities that do not exist:
an execution class, and a fleet-derived runner authority (§17). Both are exactly what a CI
replacement must construct anyway — "what kind of machine does this run on" *is* an execution
class. So this work does not compete with the recut program; it grounds the part of it that had no
consumer to justify its shape.

## 4. The terminal loop

```
poll GitHub                         extdeps.github.push_event / pulls
  → Work                            "required floor at tree T", keyed by WorkContentKey
  → Demand                          admission terms, budget, satisfaction requirement
  → match eligible Offers           fleet hosts, projected per Cut D's D2
  → ExecutionGrant                  accepted offer revision + reservation + executor + LeaseEpoch
  → Attempt                         checkout, toolchain, build, v1 parse gate, required floor
  → Receipt                         verdict, counts, evidence
  → CheckRun PATCH                  extdeps.github.checks
```

**Polling, not webhooks**, deliberately: no inbound networking, no webhook secret, no listener to
secure. The fabric does not care how the event arrived, and a poller is strictly less
infrastructure than an endpoint.

**The `LeaseEpoch` from Cut A is load-bearing here**, not decorative: two pollers, or a poller
restarted mid-run, must not both hold a grant on the same work. That is the fencing the epoch
exists for, and this is its first real consumer.

## 5. What the one PR contains

1. **`product.fabric` gains its executor** — the fold from Demand to Receipt. Types exist; the loop
   does not.
2. **Fleet hosts project Offers** (Cut D's D2), with an explicit zero quote — owned supply.
3. **An execution class** for `linux/arm64 self-hosted`, which is what `RunnerSpec` should have
   been derived from all along (§16), retiring Cut D's blocked precondition.
4. **The poller as a modeled systemd unit**, following `live_deploy_systemd_unit_for` — the only
   genuinely new infrastructure, and the tree has the pattern.
5. **Check Run reporting** via the already-modeled `extdeps.github.checks` POST/PATCH.
6. **`.github/workflows/witnesses.yml` deleted**, and `gunbc.witness_floor_workflow` with it.

## 6. Bootstrap and rollback, since there is no shadow

**The hazard is real and must be stated rather than mitigated by optimism: after this merges, the
system that gates merges is the system that just changed.** If it is broken, the fix cannot be
merged through it.

Three things make that acceptable rather than reckless:

- The **cutover PR itself is validated by Actions**, because the PR is still under the old regime.
  The last green Actions run is on the exact tree that contains the replacement.
- **Rollback is `git revert`**, and the operator merges manually today, so a revert can always be
  landed by a human who can see the check is not reporting. This is the actual escape hatch, and it
  is a person rather than a flag — which is the correct shape.
- The **fleet is already the execution substrate.** The runners are self-hosted on our hosts today,
  so this changes the control plane, not the machines.

## 7. What gets weaker — declared, not discovered

**Independence of the control plane.** Today a fleet outage costs execution but GitHub still
queues the work and reports the status. Afterwards, a fleet outage means *nothing notices the
commit at all*. That is a genuine reduction and it belongs on the ladder as a declared rung, with
its trigger: it climbs when the poller has a liveness signal whose absence is itself observable —
a check that goes red when no poll has happened, not merely green when one has.

**Log retention and the run UI** move from GitHub's storage to ours. The MVP keeps the Check Run
output as the human-readable surface; whole logs need a home before this is at parity, and naming
that gap is part of the plan rather than a follow-up someone discovers.

**Concurrency semantics are re-implemented rather than inherited**, which is where a subtle
regression is most likely — specifically the asymmetry in §2.

## 8. Open, for sign-off

1. **Scope of the first cut: main pushes only, or PRs too?** PRs bring cancellation, merge-queue
   semantics and per-PR concurrency. Main-only is dramatically smaller and still replaces a real
   thing — but it leaves PR gating on Actions, which means the workflow file cannot be deleted, and
   the doctrine's whole point is that the root goes in one motion. I do not think main-only is
   coherent with the no-shadow ruling. I want that checked rather than assumed.
2. **Where does the poller run?** A fleet host is the obvious answer and the wrong one if that host
   is also under test. Does the control plane need to be off-fleet to be honest?
3. **Is the execution class built here or in Cut D?** Building it here makes this PR bigger and
   unblocks Cut D; building it in Cut D blocks this. I lean toward here, because a consumer-less
   authority is what the recut program keeps refusing.
4. **What is the Work identity over?** The tree hash, so an identical tree does not re-run — which
   would be a real gain over Actions, or a real hazard if the floor is not actually a pure function
   of the tree. It reads pure; I have not proven it.

## 9. Two defects in this plan, registered before the sign-off returns

Both surfaced from the reviewer's working notes rather than the finished verdict, and both are
confirmed against the plan as written. Recorded now because one is an internal contradiction I
authored, and a plan that waits for permission to admit its own defect is doing the thing this
repository keeps charging elsewhere.

### `workflow_dispatch` is listed as preserved and cannot be preserved by the design

§2's table names `workflow_dispatch` among the triggers, marked **must survive: yes**. §4's loop
has **no manual trigger at all** — a poller notices commits; nobody can ask it for a run. So the
plan enumerates the contract correctly and then specifies something that cannot satisfy one row of
it. That is worse than omitting the row, because the table is what a reader would check the design
against.

Deleting the workflow deletes the only manual re-run mechanism we have, and manual re-run is not a
convenience: it is how a human recovers from an infrastructure flake without pushing an empty
commit. The replacement needs an explicit *demand-creation* surface — which in fabric terms is the
honest shape anyway, since `workflow_dispatch` **is** a Demand authored by a person rather than by
an event.

### Branch-head polling is a sampling, and sampling misses transitions

The plan says "poll GitHub for new commit on main". Polling a branch *head* observes the current
value of a mutable pointer, so two pushes between polls collapse into one observation and the
intermediate commit never receives a check. That is not a rare race: it is the ordinary case for a
merge followed quickly by another merge, and it fails **silently** — the missed commit simply has
no check rather than a red one.

This is the empty-observation narrow from DESIGN's failure-mode list, arriving through a different
door: "I sampled a pointer and saw one value" rendered as "there was one commit". The correct
construction is **reconciliation against durable state** — the set of commits that *should* carry a
verdict, differenced against the set that *do* — which is the shape the fleet spine already uses
(`Reconciliation<Intent, Evidence>`), not a poll-and-react loop. A reconciler that wakes up having
missed ten minutes converges; a poller that wakes up having missed ten minutes has lost the events.

**Consequence for §4:** the loop's first arm is wrong as drawn. It is not
`poll → new commit → Work`; it is `desired check targets ⊖ observed check runs → Work per
difference`. The trigger becomes a wake-up rather than an event, and correctness stops depending
on the polling interval.

### One framing to carry into the verdict

The reviewer's phrase for the risk is worth keeping: whether polling preserves every refusal
**without becoming a new ambient authority**. A poller acts without being asked — nothing grants it
the right to spend the fleet on a commit. In the fabric's own vocabulary that is a Demand with no
authenticated author, which is precisely the shape `std.access` exists to refuse. The reconciler
framing improves this too: a reconciler derives its work from declared desired state, and declared
desired state has an author.

## 10. Two blocking corrections — the plan tests the wrong commit, and the lease fences nothing

Both from the reviewer's working notes ahead of the verdict. Both confirmed. Both are mine, and the
first is one I had already written down and failed to apply.

### The PR subject is GitHub's synthetic merge commit, not the PR head

`actions/checkout@v5` on a `pull_request` event checks out **`refs/pull/N/merge`** — a commit
GitHub *constructs* by merging the PR head into the base. So today's CI does not test the branch;
it tests **the branch as merged into main**.

§4 of this plan says the Work is "required floor at tree T" derived from a polled commit. Applied
to PRs that would test the **head tree**, which is a different subject and a **strictly weaker
one**: a PR that is green in isolation and breaks when combined with main would pass. That is a
semantic-conflict class the current CI catches and the replacement as drawn would not — and losing
a refusal is exactly what §2 says disqualifies a minimum replacement.

It is worse than an oversight, because I have this written down. My own note
(`branch-tree-identical-ci-subject-is-not`) opens: *"run A evaluated the synthetic merge of the
branch into one main commit, run B into a later main commit"* — and its stated rule is to check the
**merge subject, not just head**, before treating two runs as comparable. I applied it to comparing
runs and did not apply it to defining the Work.

**Consequences the plan must now carry:**

- The Work identity is **not** the head tree. It is the merge result, which means the identity
  depends on **two** commits — PR head *and* base — so it changes when main moves even though the
  PR did not. That also refutes the §8 question-4 hope that "an identical tree does not re-run":
  the tree is not the input.
- Someone must **construct** the merge, since we would no longer be handed `refs/pull/N/merge`.
  That is a real operation with a real failure mode (conflicts), and a conflicted merge is a typed
  refusal, not an absent check.
- **`fetch-depth: 0` is load-bearing** and now visibly so: constructing a merge needs history.

### `LeaseEpoch` alone fences nothing without a durable compare-and-swap

§4 claims the epoch is load-bearing because "two pollers, or a poller restarted mid-run, must not
both hold a grant on the same work". The epoch is the right **token** and the sentence is
nonetheless wrong as an argument: a value does not exclude anyone. Two reconcilers can both read
epoch *N*, both believe they hold it, and both issue a grant.

Fencing requires a **durable single-writer transition** — a compare-and-swap on persisted state
where exactly one writer observes success at each epoch, and the loser refuses rather than
proceeding. Without it, `LeaseEpoch` is a label on a race.

This is the §5 trap in its purest form applied to my own design: I named a carrier and treated the
name as the guarantee, which is precisely what DESIGN §4b means by *richer type names are not
safety*. Cut A extracted the epoch as an immutable coordinate; **it never claimed to provide the
transition**, and I read the extraction as though it had.

**So the plan acquires a prerequisite it did not have:** durable state with an atomic transition.
That is the first genuinely new *stateful* infrastructure in this proposal — the poller was new
process, this is new persistence — and it needs to be named as such rather than absorbed into "the
reconciler keeps track". Where that state lives, and what makes its CAS atomic, is now an open
question ahead of the four in §8.

## 11. The merge gate is enforced by a ruleset, and the verdict's reading of it is wrong

The sign-off states there is *"currently no GitHub-enforced required-check gate on `main`: branch
data reports zero required contexts and enforcement off"*, and builds a failure-mode argument on it.
**Measured directly, that is false**, and the mechanism of the error is one this repository has a
name for.

```
GET /repos/gunb-ai/gunbc/branches/main/protection   →  403 Resource not accessible
GET /repos/gunb-ai/gunbc/rulesets                   →  "passing CI", enforcement: ACTIVE
GET /repos/gunb-ai/gunbc/rulesets/16178731          →  required_status_checks: [{ context: "witnesses" }]
                                                       plus deletion, non_fast_forward
                                                       conditions: ~DEFAULT_BRANCH
```

The gate exists, it is **active**, and it requires exactly the context `witnesses`. It is enforced
through a **ruleset**, not classic branch protection — a different API surface, which is why the
protection endpoint returns nothing useful. Reading "branch protection is empty" as "the branch is
unprotected" is the nearby-question failure: the endpoint answered a question adjacent to the one
asked, and answered it confidently.

**Three consequences, and the first is a bootstrap trap that would have bitten during the cutover.**

1. **Deleting `witnesses.yml` without addressing the ruleset makes every PR permanently
   unmergeable** — including the rollback PR. Nothing would produce a check named `witnesses`, so
   every PR sits at "Waiting for status to be reported" forever. The rollback path in §6 assumed a
   human could merge a revert; under this ruleset a human cannot, without bypass authority that has
   never been exercised.
2. **The good news is larger than the bad.** The required check carries **no `integration_id` pin**
   — the parameter is `{"context": "witnesses"}` and nothing else. So *any* source publishing a
   check named `witnesses` satisfies the rule. If the fabric's Check Run keeps that exact name, the
   gate transition is a **no-op**: no ruleset edit, no window in which the gate is absent, and the
   sign-off's concern about pinning the check to a GitHub App source becomes optional rather than
   load-bearing. **Keeping the context name is therefore a design constraint, not a preference.**
3. The failure mode the sign-off wanted is **already the one we have**: poller unavailable → no new
   check → PR unmergeable. That is fail-closed today, and the cutover must not weaken it. The
   sign-off's premise that the merge button is currently not fail-closed on CI is what would have
   licensed weakening it.

`deletion` and `non_fast_forward` are also active on the default branch, so §6's rollback story
needs a named bypass authority that has actually been tested — the sign-off's condition 16, arrived
at from the opposite direction.

## 12. The eight blocking corrections

§9 and §10 already registered three of these; they are restated here in one place with the
sign-off's sharper form.

1. **`LeaseEpoch` is not the serialization point.** It makes stale actuation decidable *after* a
   canonical Grant exists; it does not stop two ticks from both reading free capacity and minting
   different Grants against it. What is needed is a durable authority transition — *read canonical
   generation N, consume idempotent observations, derive, commit N+1 iff N is still current* — plus
   an **outbox** for GitHub mutations, because `CreateRun` succeeding and the process dying before
   recording the returned id produces a duplicate Check Run. The modeled `CheckRun` accepts an
   `external_id` on create but **does not retain it**, so there is no modeled exact join for
   recovery. That join is part of the work.
2. **`extdeps.github.push_event` cannot poll.** It parses a webhook payload from
   `GITHUB_EVENT_PATH`; once Actions is gone there is no such payload. Real polling operations are
   needed. Main: persist a **SHA cursor**, enumerate every unseen descendant in ancestry order,
   oldest first, and **refuse a non-descendant head as discontinuous history rather than silently
   resetting the cursor**. Not timestamps — commit times are not a reliable cursor.
3. **PR Work uses the synthetic merge subject** (§10), and PR head identity and execution subject
   stay **two separate facts** — the check correlates to the head, the run tests the merge. No
   head-tree fallback: an unusable merge ref is a typed `MergeSubjectUnavailable`
   (conflict / inaccessible / unobserved). A base-branch change that moves the merge subject
   naturally creates a new Demand, which is a gain.
4. **`workflow_dispatch` does not survive workflow deletion** (§9) — GitHub only provides it while
   the file exists on the default branch. It needs an explicit replacement creating a Demand with
   *requested reexecution, exact ref, new Attempt required, prior receipt not sufficient*. A Check
   Run "re-run" is not a substitute: re-requesting emits a webhook we would not receive.
5. **Check publication is a lifecycle, not a PATCH:** created `queued` when a Demand is admitted →
   `in_progress` when the Attempt begins → `completed` with a conclusion, and `cancelled` /
   `timed_out` as their own terminal arms. It needs a GitHub App identity with Checks write, whose
   token stays on the control side and **never enters the execution workspace**.
6. **Logs and the semantic result need an owned store in this cut** — deferring it was refused
   outright, and correctly: today's log surface cannot distinguish a complete semantic result from
   a cancelled prefix from a reader-truncated prefix. Retained ordered artifact, terminal manifest
   written last, manifest verifier, content-addressed blobs, `details_url` resolving to it. The
   terminal conclusion derives from the verified artifact plus process termination plus the two
   gate receipts — **never from scraping stdout**.
7. **Attempt precedes Grant.** My §4 loop had `Demand → Offer match → Grant → Attempt`, but the
   `ExecutionGrant` carrier *names the Attempt it authorizes*, so that ordering cannot be
   constructed. Attempt creation consumes no capacity; Grant issuance does.
8. **Source and toolchain materialization are real work**, not inherited. `FetchNoTags` does not
   ground an attempt worktree in a named mirror. The Rust closure is pinned at `1.93.0` with
   `clippy`/`rustfmt` and must be materialized and *verified*, not delegated to
   `setup-rust-toolchain` — that action is the old realization, not a product concept. Preserve
   Linux/AArch64 and the *reason* behind `cache: false` (no foreign post-job cleanup mutating a
   cache shared with a live Attempt); do **not** preserve the literal `self-hosted` label, which is
   a GitHub routing token whose subject disappears with Actions. `CARGO_TERM_COLOR=always` is in
   the workflow and missing from §2's table — my omission.

## 13. The four questions, answered

1. **PRs too.** Main-only is coherent only as a partial transition that retains a workflow and two
   execution authorities, contradicting the one-motion ruling — my reading was right. First cut
   covers every unseen main commit, every current PR merge subject targeting main, and manual
   reexecution. **No merge-queue semantics**: the current workflow has no `merge_group` trigger, so
   it is not in the measured contract. PR concurrency is expressed as a **state-reconciliation
   contract** — *every open PR targeting main has a terminal check for its current merge subject* —
   rather than replaying `opened`/`reopened`/`ready_for_review`/`synchronize`. One behaviour change
   is admitted deliberately: `ready_for_review` on an unchanged merge subject would no longer force
   a re-run. Draft PRs currently *do* run CI (no draft exclusion) and that stays.
2. **Off-fleet.** The control-plane principal and store must not be an Offer the control plane
   itself allocates; on-fleet placement lets one host failure remove observation, canonical state,
   allocation and execution together. Minimum footprint: one small independent node, persistent
   storage with snapshots, App key and token minting, a **one-shot tick** invoked by an external
   cadence — not a resident interpreted loop — an outbox publisher, and point-to-point authority to
   fleet executors. It runs no customer Work and never appears in the Offer roster. One node is a
   declared single point of failure whose climb is an external observer or a second replica.
3. **Execution class lands here, narrowly.** Environment and entitlement for the floor —
   Linux/AArch64, resource entitlement, source-materialization and toolchain-closure capability,
   current trust profile. It must **not** contain a `RunnerSpec`, a `self-hosted` label, a public
   SKU, a variance promise, or a physical host identity. The Offer-to-backing relation lands with
   it, because selection can otherwise choose an Offer it cannot actuate.
4. **Not the tree hash** — my §8 hope is refused. `WorkContentKey` is over source subject + CI
   contract digest + runtime closure + semantic environment + output contract, where source subject
   is the exact **commit** identity (for PRs, the merge commit plus head/base correlation). Commit
   rather than tree because the checkout materializes full history, so history may be a declared
   input until proven otherwise; and reuse is **disabled for the cutover** —
   `prior terminal receipt sufficient = no` — because witness budgets can depend on CPU and wall
   measurements, witnesses may consume Git identity, and the output contract includes the
   expected-red roster. **Dedup is not part of an authority cutover.**

## 14. What "no shadow" means, and the one thing I need re-ruled

The sign-off reads the operator's no-shadow instruction as **"no long-lived dual production
authority", not "no wet proof"**, and requires a bounded pre-merge canary: deploy the control plane
off-fleet, run the exact candidate merge subject through the fabric, publish a **non-required**
Check Run from the intended App, verify retained artifacts and teardown, disable intake, then merge
the one-motion cutover.

I think that reading is right and the distinction is real — my §6 pre-merge argument was too thin,
because a green Actions run proves the tree resolves and its witnesses pass, and proves **nothing**
about token minting, Check Run creation, durable state recovery, grant acceptance, materialization
outside Actions, or artifact publication.

**Operator ruling, 2026-08-19 — and it is better than either proposal:** *"wet proof before merge
can just come from the same PR that implements it — i.e. we make another job → confirm it works →
cutover/delete the first job."*

So the wet proof is **a second job in the existing workflow**, not a hand-deployed canary. That
resolves condition 15 and improves on it in three ways the canary could not:

- it is **automated and re-runs on every push to the PR**, so it cannot rot between the proof and
  the merge — a manually deployed canary proves a moment, a job proves the current commit;
- it needs no separate pre-merge deployment step and no human remembering to disable intake;
- the evidence is a **green CI run on a commit in the PR's own history**, citable by SHA, rather
  than a claim about something that happened on a control node.

The sequence within the one PR: add job B exercising the fabric path alongside job A (`witnesses`)
→ push, both run, B green → same PR then deletes job A, the workflow, and the emitter. The final
diff has no workflow at all; the wet proof lives in the PR's history, not its head.

**Verified as safe:** the required context is produced by the *job*, not the workflow — the live
check run on main is named `witnesses` from app `github-actions`, and the job id is `witnesses`. A
second job publishes its **own** check under its own name, which is not required and therefore
cannot disturb the gate. So the two-job phase is gate-neutral by construction.

**The naming constraint from §11 becomes load-bearing here.** The context that gates is the job
named `witnesses`; deleting that job is precisely what removes the gate. Since the required check
carries no `integration_id` pin, the fabric's Check Run named `witnesses` satisfies the same rule
from a different source — so the gate never has a gap. **Job B must therefore NOT be named
`witnesses`** during the two-job phase (that name is still the live gate), and the fabric claims it
only at the cutover commit.

## 15. The cutover PR cannot merge itself — the trap the two-job sequence walks into

The reviewer's rollback analysis notes, as an inference, that a `pull_request` workflow executes
from the **PR's merge commit** rather than from the base branch — which is why a rollback PR that
*restores* `witnesses.yml` might run the restored workflow and satisfy its own required check.

Run that inference in the other direction and it is not a convenience, it is a blocker:

```
the cutover PR DELETES .github/workflows/witnesses.yml
  → the merge commit contains no such workflow
  → Actions runs no job named `witnesses` for that PR
  → the required context `witnesses` is never reported
  → the ruleset holds the PR at "Waiting for status to be reported"
  → THE CUTOVER PR CANNOT MERGE
```

The two-job sequence is correct right up to its final commit and then blocks on itself. Job B
proves the fabric wet, job A is deleted — and deleting job A is exactly what withdraws the check
the ruleset is waiting for. The same property that makes the two-job proof work (a PR's workflows
run from its own merge subject) is what makes the deletion self-blocking.

**Three ways out, and they are not equally good.**

1. **The fabric publishes `witnesses` for the cutover PR's own merge subject.** The control plane
   is already deployed and observing by then — that is what the two-job phase established — so it
   can satisfy the gate for the very PR that installs it. This is the elegant arm: the cutover is
   gated by the system it is installing, which is the strongest possible wet proof, and the ruleset
   never changes. It requires the fabric to be *authoritative* for one PR before merge, which is a
   deliberate promotion of the canary rather than a shadow.
2. **Bypass.** Merge the cutover PR through a tested break-glass credential. This makes the
   reviewer's bypass drill **mandatory rather than prudent**, and it means the cutover's success
   depends on an authority we have never exercised.
3. **Two PRs** — one installs and proves the fabric while keeping job A, a second deletes job A
   once the fabric is already publishing `witnesses`. This is the safest and it is in tension with
   the one-motion ruling, though the *authority* still transitions in one motion; only the file
   deletion is separated.

**I have not tested the premise.** That a PR deleting a workflow causes it not to run for that PR
is documented GitHub behaviour and I am confident of it, but every confident structural claim in
this program that went untested today turned out wrong at least once, and this one decides the
shape of the final commit. The reviewer already proposed the machinery: a disposable branch with a
temporary active ruleset requiring a test context, and a PR that deletes the workflow producing it.
That test costs nothing and settles it.

**It also sharpens the canary naming.** The canary must publish `fabric-witnesses-canary`, never
`witnesses`, so the old gate stays unambiguously load-bearing and it stays provable which
implementation satisfied it. Arm 1 above is the one moment that rule is deliberately suspended, and
it should be suspended once, knowingly, at the cutover commit — not by having both producers emit
the same context throughout the canary.

## 16. Rollback, re-specified — and my claim about it was wrong

I wrote in §11 that `deletion` and `non_fast_forward` mean "a human cannot merge a revert". That
overstated it. `non_fast_forward` prevents **force** pushes, not ordinary fast-forward updates, and
`deletion` prevents deleting the ref. **Neither bans a normal push to `main`.** The operative
blocker is the required check alone — which is a narrower and more actionable finding than the one
I recorded.

What must actually be established before cutover:

- **Effective rules for `main`**, from the branch-rules endpoint — it returns every active rule
  applying to the branch including organisation-level ones, which neither classic branch protection
  nor the repository ruleset list gives.
- **`bypass_actors` on the ruleset**, fetched with credentials that can see them — GitHub omits the
  property from callers with insufficient ruleset access, so an empty reading is *not* evidence of
  no bypass. (Exactly the shape of the mistake that produced the retracted claim.)
- Bypass **mode** per actor: `always`, `pull_request`, or `exempt`. A `pull_request`-mode actor
  **cannot push directly** — it must open a PR and choose to bypass at merge. Being a repository
  administrator is not itself sufficient.

**Revised condition 16:** before cutover, at least one rollback route independent of the fabric is
executed end-to-end against an active equivalent ruleset, and a second independently shaped route
is prepared. Preferred pair: a rollback PR that restores and runs `witnesses.yml` as primary, and a
tested ruleset bypass by an independent credential as break-glass. The drill runs on a disposable
branch with its own temporary ruleset — negative control (ordinary identity refused), positive
control (break-glass succeeds and Rule Insights records a **Bypass**, an inspectable receipt rather
than a UI that looked like it would work) — then proves the drill branch's effective rules
equivalent to `main`'s.

**The rollback credential, patch, and ruleset snapshot must not live only on the fabric control-plane
machine.** That is the whole point of a break-glass path.

## 17. App pinning is deferred, deliberately

Agreed both ways: **do not pin the App during the cutover.** No ruleset mutation inside an already
large authority transition, no interval where old and new required-check configuration disagree,
and the Actions-produced `witnesses` continues to gate the cutover PR. The cost is real but
**pre-existing**: an unpinned context can be published by anyone with sufficient repository
permission.

Pinning is a later hardening cut, gated on the fabric App having produced `witnesses` on current
`main` and on every open PR's exact merge subject, its installation independently observed, the
rollback path not depending on that App being alive, and the current effective rules captured.
Then **update** the existing entry to add `integration_id` — never delete and recreate it. The
failure mode after pinning is not a protection gap but a fail-closed queue freeze if some open
subject lacks a fabric-authored check, which is why the pre-population is part of the cut.

## 18. Product direction reframes this PR, and corrects four things in it

From `swift-badger-524`, 2026-08-19. The reframe is not a relabelling: it changes what counts as
done, and two of the four corrections name defects in code already on this branch.

### The reframe

Not *"replace GitHub Actions with the fabric"* but: **build the first production CI binding, make
gunbc its first consumer, then cut gunbc's old executor authority over in one motion.** The test is
sharp — *you can successfully replace Actions and still have no saleable CI system.* The self-CI
cutover then wet-tests the exact path an external customer will use, which is a strictly stronger
proof than replacing our own workflow.

The ownership split that follows:

| layer | owns |
| --- | --- |
| `product.fabric` | provider-neutral negotiation, attempts, grants, receipts |
| GitHub binding | observations, subject correlation, demand commands, check projection |
| CI product layer | the public execution contract, isolation promise, billable receipt |
| `gunbc.*` | **only** the floor's program and contracts |

**This locates a defect in what I already built.** `gunbc.fabric_witness_run.authorize_floor_run`
takes a `Work` and an `Offer` and answers a fungibility question — *does this executor satisfy these
requirements* — which is **provider-neutral fabric logic sitting in a gunbc module**. `FloorRunRefusal`
is likewise a general fungibility refusal wearing a floor-specific name: nothing in
`ShapeNotCovered`, `TrustDomainMismatch` or `CapabilitiesNotOffered` is about the floor. The gunbc
module should keep `floor_work_contract` and `floor_execution_requirements` and nothing else.

### GitHub vocabulary must not leak into the fabric

**Operator ruling: there is no "main vs PR" in the fabric — it is just how compute is negotiated.**
§13's answer to Q1 is right in substance and wrong in vocabulary: *"every unseen main commit, every
current PR merge subject, manual reexecution"* are three ways **this binding** obtains desired
demands, and they must never become three modes inside the fabric. A fabric that names its
provider's event kinds cannot admit a provider it did not anticipate — the same argument
`FabricIdentity` already makes for keeping the principal type open.

### Self-CI is an ordinary priced demand

**Confirmed as a gap in this branch:** `authorize_floor_run` checks shape, capabilities and trust
and **does not consider price at all**, so gunbc's floor is currently an always-admitted path. That
is precisely the privileged arm the arbitrage thesis forbids — if self-CI bypasses the market, the
opportunity cost of running our own work is not a computable quantity and the arbitrage degrades
into a hand-maintained spreadsheet. Cheap now, expensive to retrofit.

Control-plane work is a separately privileged class, and that exclusion **must not leak to floor
runs**.

### Placement: the invariant is not "off-fleet"

§13's Q2 answer is superseded. The invariant is that **no single capacity, scheduler generation,
deployment, or failure domain may simultaneously remove observation, canonical state, allocation,
AND the only means of restoring them.** Meta-processes may run *on* the fabric, modeled as **control
work** — an execution class with reserved entitlement, its own trust domain, placement constraints
and receipts — distinct from customer work, which runs only from an admitted grant with no
control-plane credentials.

Four prerequisites, not follow-ups: **(a)** the reserved placement class; **(b)** a durable
single-writer compare-and-swap — *`LeaseEpoch` is a fence, not a serialization mechanism, and
replication without CAS makes the race worse rather than better*; **(c)** bootstrap independence —
whatever starts the control plane must not require the control plane, so an external one-shot
cadence that consults nothing; **(d)** rollout durability — a new version takes over from a **dead**
predecessor, never requiring a graceful handoff. First honest rung: one management node plus an
independent watchdog plus a tested restore path, declared as an SPOF.

### `WorkContentKey` bundles three concerns — split it

§13's Q4 answer conflated **identity**, **reuse policy**, and **correlation**. Reuse-disabled-for-
cutover is a **policy over identities**, not a property of identity; per §3 policy is a workflow
fact. Turning reuse on later must not mean editing what things *are*.

**The split is already available in the carriers** — verified rather than proposed:
`product.fabric.demand.SatisfactionRequirement { terminal_receipt_may_satisfy, new_attempt_required,
step_reuse_permitted }` lives on the **Demand**. So identity stays `WorkContentKey`, reuse policy
rides the Demand, and the cutover's "no reuse" is one Demand-side setting rather than a property
baked into what a Work *is*.

## 19. The arbitrage runs within an architecture, not across it

I asked for the arch-independent share of CI minutes, on the reasoning that it bounds the pool the
arbitrage operates over. **That framing was wrong and the number does not gate anything.**

The arbitrage runs **between suppliers at a given architecture**:

```
arm64 work   ->  our Ampere fleet   vs   rented ARM (Hetzner CAX)
amd64 work   ->  rented x86 (CPX/CCX) vs  any other x86 supplier
```

Every arm64 job — *including our entire floor, which is all arm64* — is already a live
own-vs-rented decision with two real offers. That is exactly the operator's arbitrage (run my CI on
my hardware, or sell that capacity and rent cheaper) and it requires **zero** architecture-independent
work to exist.

Architecture-independent work is a **second-order bonus pool** on top: it additionally lets a job
cross arch lines to whichever is cheapest. Probably small — my guess there was likely right — but it
is not the mechanism, and I had promoted a second-order term to the load-bearing one. Under that
reading the pool looked nearly empty; under the correct one it is 100% of our floor plus 100% of
arm64 customer work, which is measurable from our own usage rather than from a market statistic
nobody publishes.

**Open quantity, not a blocker:** the second-order cross-arch pool stays unmeasured. Not acquiring
it is deliberate — this is a revenue lane, not a research project.

**What it changes for the build:** the two-offer condition that makes pricing live rather than
formal is satisfiable *today* with own-fleet plus one rented ARM offer. That is the next supply
increment after this slice, and it is what turns the price check from a formality into a market.
