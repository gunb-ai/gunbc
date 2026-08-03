# Push-event affected-set baseline

Design note for work item `adhoc-b62a4d11-6a6`. Written to outlive the two PRs it
sequences (gunbc#7720 receipts, gunbc#7729 before-baseline), because the load-bearing
part is not either implementation — it is which claims here are **observed**, which are
**spec-grounded**, and which window each number came from. That distinction decays fastest
if it stays in a session's head, and every number below is short-window by construction.

## 1. The defect

`gunbc.diff_baseline` `resolve_diff_baseline` maps a `push` event to
`PushParent{origin/main}`. On a push **to** main, `actions/checkout` has already moved
`origin/main` to the pushed SHA, so the floor diffs the commit against itself and observes
zero changed paths. Every entry that is not independently fail-closed then skips —
including entries the merge itself modified.

Receipts:

- Two different main pushes report an identical `4118 unaffected` (runs `30774223741`,
  `30773369033`). A skip count cannot be a function of a diff it does not depend on.
- gunbc#7687 added four witnesses to
  `src/v2/test/claim/round_trip/dag_comment_wall_test.dag`. All four executed on its own
  PR run `30761160726`; **none** executed on the main-push run `30764935708` that merged
  them. No line named the entry either way — skips are counted, not narrated.
- The build log of that run shows the mechanism directly:
  `30710a9083..2ab7c1ee96  main -> origin/main`.

**The class.** This is the mirror of DESIGN §5's absorbing fallback. There, a mechanism
that cannot compute the affected set substitutes the *superset* — ⊤-as-answer conflated
with ⊤-as-ignorance. Here it substitutes the *empty set* — ⊥-as-answer conflated with
⊥-as-ignorance. It does not widen, it narrows to nothing, which is strictly worse: a widen
is merely expensive, a narrow is silently uncovered. The name **empty-observation narrow** is
proposed for DESIGN's recurring-failure-modes list by gunbc#7720, which is open at the time of
writing; until that merges the class has no entry in the canonical authority and this note is
the only place it is named.

**The tell that the observation is wrong rather than three local choices being reasonable:**
one observation, three consumers, three different copings — compile-clean *widens* to
whole-tree (fail-closed, correct), regen gates on `GITHUB_EVENT_NAME` (a workaround, and
its own comment admits the empty diff), the witness floor *skips* (fail-open).

## 2. Why the payload, not an env channel

Two shapes were considered:

- **(a)** `ci.yml` passes `GUNBC_CI_PUSH_BEFORE: ${{ github.event.before }}`.
- **(b)** Read `GITHUB_EVENT_PATH` and take the payload's `before` member.

**(b) is chosen, and the §3 argument decides it rather than the diff size.**
`GITHUB_EVENT_PATH` is the payload GitHub itself publishes and `before` is GitHub's own
name for the field. `GUNBC_CI_PUSH_BEFORE` would be a *second name* for that field, minted
by us, carried on a channel we own, requiring a generated-workflow edit to populate — a
nickname standing where the upstream authority was already available, and the duplication
would land in an emitted artifact where it is most expensive to unpick later. That (b) is
also the smaller diff, touching no generated artifact, is a consequence and not the reason.

The parse half already exists in the corpus: `extdeps/languages/json` means reading the payload
is a modeled ingest rather than a jq shell-out (a jq call here would be exactly the
shell-emission class the routing rules push back on). The read half does **not** exist — the one
process that read `GITHUB_EVENT_PATH` was retired, and the only surviving mention on main is
`tools.merge_admission_walk` `merge_admission_pr_number_deferred_note`, which records that
the retired cold stamp parsed the payload through jq shell and that the warm path deliberately
writes `Absent` rather than carry that transport into a runtime-present claim. That note is
better evidence than a live reader would be: its stated dissolve-on is *"a typed GitHub event
payload projection supplies the optional number"* — the repository has already declared the
carrier this note proposes to be the thing it is waiting for, and one projection discharges
both consumers.

## 3. The arms, and the evidence status of each

Measured read-only against the repo events API. **Window:
`2026-08-02T22:16:00Z .. 2026-08-03T03:26:13Z`, 166 push events.** The API retains only a
few hundred events and `before` is not recoverable once a push ages out, so this window
cannot be extended backwards.

| arm | evidence status |
| --- | --- |
| ordinary push — `before` is the prior tip | **observed**: 21/21 main pushes carry a populated, non-zero `before`; ancestor of head in 21/21; chains to the prior observed main head in 20/21 (the miss is the window boundary) |
| zero-SHA (branch creation) | **unobserved, spec-grounded** — see below |
| absent (`schedule`, `workflow_dispatch`, `merge_group`) | not observable from this instrument; those runs carry no push payload |
| non-ancestor (force-push) | **observed off-path**: 9 of 145 *branch* pushes; **0 of 21** main pushes |

**Zero-SHA is unobserved, not absent.** No all-zero `before` appeared, because branch
creation surfaces as a `CreateEvent` rather than a `PushEvent` — a `PushEvent` scan
structurally cannot see the case. The arm is therefore grounded in GitHub's published
payload semantics — which no carrier in this repository holds today; the note that grounds
the arm is written by gunbc#7729 as `extdeps.github.push_event` `push_event_note` and does not
exist until that lands — and **not** in this measurement. An arm that looks measured when it is only reasoned is worse than one
that says it is spec-grounded, because only the first stops anyone from checking.

**The non-ancestor arm is justified by locatability, not by under-selection and not by
rate.** Two claims that sound right and are not:

- *"A non-ancestor `before` yields a wrong affected set."* Not demonstrated. `git diff
  before..head` is a full tree comparison, so any path whose content differs is selected.
  After a force-push replacing X with X', paths they touch identically did not change and
  were covered by the run that verified X. It over-selects (reverts of discarded commits
  appear), which is wasteful, not unsafe.
- *"A discarded object will be missing, so git fails loudly."* False here: all **9 of 9**
  non-ancestor pushes in the window still have their `before` object present in a full
  clone.

What survives: when the object genuinely is gone, the path dies inside `git diff` with
"invalid ref or not a git repository", the least locatable diagnostic in the chain. That is
worth a typed arm. The 9-of-145 figure is a fact about *branch* pushes, which never reach
this arm — `ci.yml` has no push trigger outside main, so a force-push to a PR branch arrives
as `pull_request` `synchronize` and resolves through `MergeTarget`.

## 4. The chaining assumption — the deeper defect

Incremental push-diff selection is sound **if and only if every prior push was covered**.
Each run covers only its own delta, so the deltas must compose without gaps. They do not:

```
W green
push X, run cancelled
push X2
path P changed in W..X, and is byte-identical between X and X2
diff(X, X2) does not contain P
P was never covered, because X's run never completed
```

No force-push required — ordinary chained pushes and one incomplete run. This is silent
under-selection, and unlike the within-hop cases above it is real.

### 4.1 Run conclusion is not floor coverage — in both directions

The obvious measurement is wrong and its wrongness matters, because it would have driven
the design the other way.

At run level, **100 main push runs, window `2026-08-01T19:28:55Z .. 2026-08-03T02:55:01Z`**:
37 success, 36 failure, 20 cancelled, 7 in flight. By that measure 62 of 99 pushes had a
non-success predecessor, and for 40 of them no successful predecessor exists anywhere in
the window — implying a last-green baseline reaching back past 31 hours.

Sampling one level finer dissolves it:

- Of the 15 most recent **failure** runs, the floor job (`ci`) itself **succeeded in 6**,
  failed in 7, was skipped in 2. A red main run frequently means something other than the
  floor failed, and coverage happened.
- Of the 10 most recent **cancelled** runs, the floor job succeeded in **10 of 10**.
  Cancellation lands *after* the floor completes.

Both readings count holes that are not holes.

### 4.2 The right predicate, measured

Floor job conclusion, **last 40 main pushes, window
`2026-08-02T16:22:30Z .. 2026-08-03T02:55:01Z`**: floor green on 30, in-flight on 6, failed
on 4. **Predecessor floor not green: 9 of 39.** Walk-back distance to the nearest green
floor: 30 at distance 1, then 2 each at distances 2–5, 1 at distance 6. **Maximum 6.**

So a last-green baseline does **not** degenerate to the full corpus: it is the immediate
predecessor about 77% of the time and never reached back more than 6 pushes in this window.
The cost delta over a before-baseline is bounded and modest.

### 4.3 Two properties that must not be left implicit

**In-flight is not green.** Six of forty floors had not concluded. A floor still running has
covered nothing yet, so a baseline computed at that moment must treat in-flight as not-green.
Treating it as pending-and-probably-fine is the state-space conflation this lane exists to
name.

**"Floor job succeeded" means the job passed, not that the corpus executed.** Those coincide
**today only because main pushes select nothing** and therefore run the whole fail-closed
corpus. After the before-baseline change they stop coinciding — so the predicate a last-green
baseline depends on is one that the before-baseline change itself invalidates. A last-green
baseline must therefore be specified against *the floor having run and its selection having
covered that commit*, never against floor job conclusion. Naming this now is worth more than
the measurement, because the version discovered later is discovered by something being
silently uncovered.

## 5. Staging, and the condition on stage 1

**Stage 0 — receipts (gunbc#7720, landed/landing).** The empty-observation state is named,
located by the baseline it compared against, and carries its own frequency. Run disposition
untouched: the empty diff has no special run arm (operator ruling 2026-07-05).

**Stage 1 — before-baseline (gunbc#7729).** Push events resolve `PushBefore` from the
payload. Strictly better than self-compare.

> **Landing condition.** Stage 1 must emit a **typed, located, counted receipt** when it
> cannot establish that the predecessor commit's floor covered it. Not a refusal — the state
> occurs 9 times in 39 and halting main CI on it is not proportionate.
>
> This condition is not a hedge. Today the chaining gap is a hole in a floor nobody stands
> on: main pushes select nothing, so coverage is total and the gap is inert. The moment
> stage 1 lands, selection becomes real and the gap becomes real with it. Without a receipt
> the gap does not become *observable*, it becomes **active and silent** — a deficit whose
> frequency is zero by construction never ranks for fixing (§5), and that is a rung
> regression under §4b(3), which requires previous rung, temporary rung, reason, bounded
> population, and restoration trigger to be declared.
>
> - previous rung: coverage total (by accident of the defect)
> - temporary rung: coverage incremental, chaining unverified
> - reason: before-baseline is a strict improvement over self-compare and the last-green
>   observation is a larger change that should not be smuggled into it
> - bounded population: predecessor floor not green in 9 of 39; walk-back never past 6
> - restoration trigger: receipt count goes to zero when stage 2 lands
>
> The receipt does not need to *know* the predecessor floor was green. It needs to record
> that it **cannot** know — which at that layer is unconditionally true today, so the honest
> form is one counted row stating that this baseline assumes predecessor coverage and cannot
> verify it. Cheap to write, impossible to mistake for a proof, and it dissolves when stage 2
> lands.

**Stage 2 — last-green baseline.** Baseline becomes the last main commit whose floor ran and
whose selection covered it (§4.3, not floor job conclusion). Requires a fail-closed
run-conclusion observation that **refuses** when it cannot determine the answer — an
observation that quietly fell back to `before` would reintroduce the gap it exists to close.
Gated on the operator's cost call; §4.2 says the cost is bounded.

## 6. Dissolution

This note retires when stage 2 lands and its receipt count reaches zero. Until then the
authority for each fact is the carrier that holds it. Two exist on main today —
`gunbc.diff_baseline` and `v2.workflow.floor_diff_observe`; the third,
`extdeps.github.push_event`, is *proposed* by gunbc#7729 and is named here as the intended home
for the payload facts, not as a symbol anyone can resolve yet. This note is the sequencing
argument plus the evidence-status table, which no carrier owns.
