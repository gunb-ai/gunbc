# The run did not measure what you think: ten failures in reading CI (2026-08-23, extended 2026-08-26)

**Subject:** attributing a CI result to a change. **Deliverable:** one class, ten measured
instances, and the check that closes each. **No repair is proposed here.**

> **Every run id cited below is attempt 1**, verified 2026-08-26. This matters and is not
> boilerplate: a re-run overwrites run-level fields, so an un-qualified run id can stop
> reproducing without anyone editing this file. See *The run-level object is a mutable
> projection* under instance 5. If a citation here ever fails to reproduce, check
> `run_attempt` before concluding the claim was wrong.

**Provenance.** Instances 1–4 were measured 2026-08-23, in four different lanes. Instances 5–6
were measured 2026-08-25, and one of them **falsified instance 4's own check** — which is why
they are folded in here rather than filed beside: two accounts of one fact is the failure this
document exists to describe. Instances 7–10 were measured 2026-08-26, across three further lanes.
Ten instances, six lanes, four days.

Every instance below cost real time, and most produced a *confident wrong attribution* rather
than an ambiguous one. That is what makes them one class rather than ten mistakes: in each case a
run was read as evidence about a change, and it was not evidence about that change.

> **This paragraph was itself a specimen, caught in review (`review 55981`, 2026-08-26).** It read
> *"Every instance below cost real time on 2026-08-23, in four different lanes … one class rather
> than four mistakes"* — true when the file held four instances, and left standing while six more
> were added beneath it across three later days. Nothing in the additions touched it, so nothing
> flagged it, and the headline provenance claim quietly became false while every instance under it
> stayed accurate. That is the stale-authority shape this file documents, occurring in the file
> that documents it: **an edit above or below a claim does not re-check the claim.** It is left
> recorded rather than silently corrected, because a document arguing that provenance decays is
> more credible carrying a receipt of its own decay than asserting it cleanly.

## The class

> **A run reports the ref it BUILT. It never reports what that ref was FOR, and it is rarely
> the ref you pushed.**

Between "a change" and "a verdict" sit several independent substitutions, each invisible in the
result. A green or red is equally plausible under all of them. Instances 5–6 add a second half to
the class: **even once the subject is established, the field you read may not be the field that
holds the verdict.**

## Instance 1 — `pull_request` builds the MERGE REF, not your head

The `witnesses` workflow triggers on `pull_request`, so GitHub builds a merge commit of the head
into the **current base tip**. A lane's floor run reported `head=cbe0c7253c`, which was neither
their SHA nor anything in their branch: `p1` was a main commit they had never merged, `p2` was
their head.

**Consequence, and it reached every open PR:** when main carries a defect, *every* PR run
inherits it, regardless of branch content. Two lanes independently concluded they had introduced
nine failing floor claims. The claims were `#8947`'s, which had landed **already red on its own
push run** — verified at `32610048803`, `event=push`, conclusion `failure`, touching exactly the
failing subjects.

**Check:** `gh api …/runs/<id> -q .event` — if `pull_request`, the built ref is a merge and main
is in it. Compare against a main run, not against your assumption.

## Instance 2 — a stale baseline inside a correct control

A lane ran a genuinely controlled comparison: main at `544b0feda` (`failed=0`) against their PR
run (`failed=9`), same phase, same whole-corpus scope. Sound method. **Their branch had already
absorbed a newer main**, so the two arms differed by their diff *and* by everything main landed
in between.

The tell is not in the numbers; both arms are real measurements. It is in the *interval*, and
the interval was never stated.

**Check:** name both refs and the interval between them in the receipt. If the interval is
non-empty, the comparison is not attributing to your diff alone. (Here the interval I proposed
for bisection turned out to be one commit changing one markdown file — measured by the lane
rather than obeyed, which is why it cost nothing.)

**Second receipt, different tool: the baseline moved between two commands (2026-08-26).** Adding
*this file* to a branch, a tree was built with `git read-tree origin/main` in one step and a
commit created with `git commit-tree -p origin/main` in the next. `origin/main` advanced in
between. Each command was individually correct and each read a *live* ref; the pair silently
disagreed, producing a commit whose tree was based on one main and whose parent was another — so
it reverted two commits' work while claiming to add one file. Both the local `git diff --stat`
against the base *and* `gh pr create` succeeded without complaint; the tell was the PR reporting
**19 changed files** where one was expected.

The repair is the general one for this class: **pin the ref once and use the pinned value
everywhere**, rather than re-reading a moving name at each step. A symbolic ref is not a
baseline; the SHA it happened to point at is.

## Instance 3 — the built head was a commit authored to be broken

A defect was attributed to a PR on the strength of run `32615991755`: receiver types lost to
`Primitive()`, with **0 occurrences on three main runs**. The absence-on-main check is exactly
right and was applied correctly.

The run built `18b5ddc626`, whose own message reads **"BISECT ARM (not a landing state)"**. It
was a deliberate revert arm, already reverted. On the lane's real head, occurrences: **0**.

A bisect arm, a deliberate RED control, a scratch revert and a merge ref are **indistinguishable
in `gh run view --json headSha`**. This is strictly worse than instance 1: misattributing main's
red wastes a lane's time, but attributing a defect to a lane *on the strength of a run they
authored to fail* tells them to stop doing the thing they were right to do.

**Check:** `git log -1 --format='%s' <built-sha>` before reading any run as evidence. One
command, and it was the missing step.

## Instance 4 — the cancelled run, which announces nothing

`concurrency: cancel-in-progress` is set for `pull_request`, keyed on PR number, so **every push
cancels the in-flight run by design**. One lane had five consecutive runs cancelled by its own
push cadence: two hours in which every head had **zero floor evidence** while the PR page looked
normal. A cancelled run is not a failed run, and the only tell is in `gh run list`.

With the floor at ~34 minutes single-threaded, any push cadence under ~45 minutes guarantees you
never observe a completed floor. **Re-measured 2026-08-25 and it has grown:** completed floors ran
`50m41s` (`32907586489`) and `48m28s` (`32908485619`), so the cadence that starves the evidence is
now under ~55 minutes. The figure is a property of the corpus, not of the workflow, and will keep
moving.

**Check:** read the run's `conclusion` explicitly. `cancelled` is not `failure` and is not
`success`; absence of a verdict is not a verdict.

> **CORRECTED 2026-08-25 — the check above is necessary and NOT sufficient, and it fails in the
> flattering direction.** See "Instance 5" below: a run whose `conclusion` is `cancelled` may
> contain a job that **failed terminally** before the cancellation arrived. Read jobs, not runs.


## Instance 5 — the run-level `conclusion` CONCEALS a terminal job failure (2026-08-25)

Instance 4's check reads the run's `conclusion`. That field reports only the **later** of several
job outcomes, so a job that failed terminally is overwritten by a cancellation that arrived
afterwards.

Receipt, run `32909985425` **attempt 1**:

```
build   COMPLETED     conclusion=FAILURE      23:35:43   <- real, terminal (rustfmt ETXTBSY)
floor   in_progress -> CANCELLED              23:42      <- killed by the author's own push
run-level conclusion:  cancelled
```

Both are true and **neither summarises the other**. The lane that found this had already been
told "a cancelled floor is not a red" — correct at *floor* grain, and at *run* grain it discards
a real defect. Had the run-level field been read, a genuine terminal failure would have been
closed as noise.

**The direction matters: the masking FLATTERS.** `cancelled` reads as no-verdict while a failure
sits underneath, so this error always resolves toward "nothing is wrong."

**Check:** `gh api …/runs/<id>/attempts/<n>/jobs` and read **`status` and `conclusion` per job,
naming the lane**. A cancelled run is not a red, but it is not necessarily a non-red either.

### The run-level object is a MUTABLE PROJECTION of the latest attempt

The same field that hides a job failure also hides an *earlier attempt*, and this one rots a
citation silently.

**A re-run overwrites run-level fields.** Measured: a failed run queried after re-running
reported the new attempt's `started_at` (22:12) where the true first-attempt time was 21:10. The
run-level object is not a record; it is a view of whichever attempt ran last. **The immutable
record is the attempt.**

That lands directly on the receipt above. It reproduces today because nobody has re-run
`32909985425` — run-level `run_attempt=1`, and `attempts/1` agrees. **If anyone ever re-runs it,
the run-level conclusion becomes the new attempt and the claim stops reproducing for a reader who
checks.** The citation would not become *wrong*; it would become *unverifiable*, with no edit to
this document. That is precisely the rot pointers are supposed to avoid, arriving through a
channel that looks immutable.

So: **cite `…/runs/<id>/attempts/<n>`, or write "attempt 1" beside the id.** It costs one word and
it is the only grain that stays checkable. `attempts/<n>/jobs` carries the per-job detail, so
nothing is given up by descending.

**One mutable projection, two things hidden under it:** the run-level *conclusion* masks a job
failure, and the run-level *timestamp* masks the earlier attempt. A reader who learns to descend
to jobs should learn to descend to attempts in the same breath.

#### A re-run of *failed jobs only* carries the passing jobs forward, with new ids

`gh run rerun <id> --failed` re-executes only what failed. The jobs that passed still appear in
the new attempt's job list, as `success` — with a **new job id** but the **original attempt's
timestamps**. Measured:

```
attempt 1  floor id=98007396821  started 23:40:44  completed 00:25:27
attempt 2  floor id=98018370339  started 23:40:44  completed 00:25:27   <- carried, not re-run
attempt 2  build id=98018369122  started 00:34:10  completed 00:53:50   <- actually re-run
```

The build in attempt 2 *started nine minutes after* the floor's recorded completion, which is
the tell. **The id says re-run; the timestamps say carried.** Two consequences, and the second
is the dangerous one: a reader auditing runner spend concludes a 45-minute lane was re-executed
when it was not, and — worse — a reader treating "attempt 2, all green" as one measurement is
reading a result partly measured *before* the re-run. Attempts share a commit so nothing can
have changed here, but the composite reading is still not what it appears to be.

**Check:** compare each job's `started_at` against the attempt's own start. A job that predates
its attempt was carried forward.

### The classification ladder

Four rungs, each learned by getting it wrong. Only after all four are excluded is a red yours:

| # | cause | tell |
|---|---|---|
| 1 | supersession | a newer run on the same **PR** — query by branch, not `head_sha` |
| 2 | runner kill | a step at `status=in_progress` on a **COMPLETED** job |
| 3 | runner setup | `Set up job` is the only named failure |
| 4 | concealed failure | run-level `cancelled` with a job-level **FAILURE** inside |

**Rung 4 must be checked even when rung 1 explains the run**, because supersession explains the
*cancellation* without explaining what was underneath it.

**One benign pattern, named so it is not chased:** `witnesses=failure` beside a cancelled floor is
the `always()` aggregator reporting *Both required lanes must have succeeded*. It is expected by
design. Anyone who adopts "read jobs, not runs" hits it immediately and it looks like a fifth
failure mode.

## Instance 6 — the supersession query that cannot see supersession (2026-08-25)

A lane measured 12 of 13 recent floor lanes cancelled across 8 branches and reported it as
environmental, explicitly **withholding an attribution rather than inventing one**. The
measurement was right. The framing was not: it is instance 4's mechanism at fleet scale.

They ruled out supersession by querying `runs?head_sha=X` and finding no newer run. **The
observation was true and the conclusion was false**, because the concurrency group is keyed on
**PR number** while the query projected onto `head_sha`. A newer run at a *different* head of the
*same* PR cancels the older one and is invisible to that query **by construction**.

That is the repository's index-domain congruence failure — `key(x) == key(y)` iff `x` and `y` are
interchangeable for the fact the index answers — arriving in a REST query rather than in a
resolver, on the same night the same class was being diagnosed in the compiler. **It is a receipt
that the class is general rather than a compiler quirk**, which is worth more than the incident.

**RESIDUE, DELIBERATELY LEFT OPEN.** The mechanism does not explain every instance. Run
`32908485619` (PR 9231) is the newest run on its PR, was cancelled after 48 minutes with nothing
to supersede it, and no host event correlates — three cancellations in that two-minute window ran
on three different hosts. No attribution is offered here, because a corrected account that reads
as *complete* sends the next person with an unsuperseded cancellation hunting for a push they
never made. **Mechanism confirmed, residue open.**

**Check:** query by **branch**, never by `head_sha`, and state which key the index uses.


## Instance 7 — the diagnostic span is BYTES and renders exactly like a line range (2026-08-26)

This one was found by the failure it describes, three hops deep, and it is the reason the other
six are worth writing down.

A lane reported a seed-parser defect: `expected expression, found Newline` pointing at the *next
declaration* rather than at the offending construct. It was relayed upward, sharpened at each
hop — *typed, located, and located wrong*; *worse than a missing feature*; then an admissibility
ruling under the v1 freeze saying someone should fix it. **The defect does not exist.** Two
independent probes, on different trees and different binaries, put the offset on the exact
character:

```
line 8 starts at byte 113, "  callee(xs:" = 12 chars
  -> newline terminating line 8 is byte 125
probe.dag:125-126: expected expression, found Newline
```

The locator is correct at character grain. What is *real* is why two people misread the same
number:

> **The diagnostic prints `file:START-END`, which is a BYTE SPAN and is indistinguishable from a
> line range.** Every neighbouring toolchain prints `file:line:col` in that position.

In an eleven-line probe, `125-126` is obviously not lines. **In a large file it is perfectly
plausible as one** — and the original report's `162-163` landed, read as lines, inside a
different declaration. That is precisely the innocent code the reader was sent to.

So the false report was never only a bad `awk` accumulator. The accumulator produced a wrong
attribution *and the raw output already looked like a line pointer agreeing with it*. Two
independent readings landed in the same wrong place, which is a property of the **rendering**,
not of either reader.

**The corrected finding is much smaller than the one that was endorsed, and it is true:** the
diagnostic is typed, correctly located, and renders its location in a format ambiguous with the
near-universal convention. An ergonomic defect, not a correctness one. It has now cost two regen
cycles and three hops of amplification.

**Check:** treat `file:N-M` from this compiler as **bytes** until proven otherwise. Convert with
a control — a probe whose answer you know — never with an accumulator you wrote in the same
sitting as the question.

### The escalation corollary

Three people amplified this, each adding confidence and none adding evidence. The rule the chain
forces:

> **A report from a lane is evidence, not a measurement.** Relaying it upward converts it into a
> claim in the relayer's name, and *that conversion is where verification belongs* — escalation
> strips the reporter's hedging, and the receiver cannot distinguish what was measured from what
> was inferred.

And its sharper half, which the third hop demonstrated by attaching a *ruling* to an unverified
report:

> **Each escalation hop raises confidence while lowering verifiability.** The receiver is
> progressively further from the instrument and progressively more likely to be the one people
> act on. So the duty to verify *increases with height*, exactly where the means to verify
> decrease — and exactly where a manager most wants to add value by ruling.

Every hop here had the means to check: one eleven-line file and one addition. Each did the
arithmetic *after* the retraction rather than before their own contribution.


## Instance 8 — a bounded query cannot report its own boundary (2026-08-26)

Two PRs were about to be reported as having **no run on their current head** — which reads as
*their greens are stale, the heads moved*, and would have sent two lanes to re-verify work that
was already green.

The runs existed. The query searched the **40 most recent** workflow runs and theirs had aged
out. Querying by **branch** found both, green, on their exact heads.

> **An absence produced by a bounded query is not an absence in the world.**

This is the empty-observation narrow — ⊥-as-ignorance rendered as ⊥-as-answer — arriving in the
tooling used to read CI rather than in CI itself.

**The repair generalises, and it is the reusable part.** A bounded query *cannot report its own
boundary*, so no amount of scrutinising its output finds the defect. Re-reading the same result
more carefully is guaranteed to fail. What works is **re-querying through a different index** —
by branch instead of by recency — because the second index has a different bound.

That is the same move as descending run → job → attempt: **change the projection; do not squint
at the current one.** Every masking failure in this document yields to a different projection and
none of them yield to closer reading of the same one.

**Check:** before reporting an absence, re-derive it through a second index whose bound differs.
If you cannot name the first query's bound, you have not established the absence.

**The stronger check, and the only one in this document that caught a live error rather than
explaining one after the fact: run a POSITIVE CONTROL.** Before trusting a zero, search the same
way for something you *know* is present. If the control also returns zero, the instrument is
wrong and the finding is void.

This is cheap, it is mechanical, and it does not depend on noticing anything. It fires on the
dangerous case — the zero that confirms what you expected — where surprise never will. Its
receipt: a lane was about to report that a complete `std` carrier had **no consumers**, on a
repo-wide grep returning one file. Grepping the same way for a symbol read minutes earlier with
their own eyes also returned nothing — because the worktree was **164 files behind `main`** and
did not contain the file at all. Every "whole tree" measurement that session had silently been
"my stale branch". The real answer was one consumer, not zero, and the correction arrived before
the claim was published rather than after.

Note what the control tests. Not the query — the *reachability of the subject*. Instance 8's
original repair (re-query through a second index) would not have caught this: both indexes were
bounded by the same missing files. **A second projection over the same absent subject reproduces
the absence perfectly.** The control is the only instrument here that distinguishes *the thing is
not there* from *I am not where the thing is*.

**A grep pattern is a bounded query too**, and the same night produced the receipt. A lane was
about to report that a gate did not print cargo's diagnostics, on the strength of grepping its
log and getting zero hits. The gate prints them — prefixed `cargo|`, which their pattern did not
match. In their words: *that zero is a claim about my instrument, not about the program.* The
bound need not be a `per_page` limit; a filter expression is a bound, and an empty result from
one establishes nothing until the filter itself is checked against a known-present line.

**A third receipt, two days later, adds the form that is hardest to see: a bound that is a
*window over correct output*.** Re-checking whether a lockfile drift still stood, a lane ran
`grep -A 12` against the package block and got zero. The block is longer than twelve lines. The
command's output was entirely correct; the tool answered exactly what was asked; the *question*
carried a bound that the answer could not mention. A `per_page` and a filter at least look like
bounds — `-A 12`, `head -n`, a `sed` range, a truncated log tail do not, because they read as
*how much output I want* rather than *how much of the subject I am willing to see*. The drift was
still there, confirmed by a full-block scan.

**And the detector that caught it was not the check above — it was surprise.** In that lane's own
words, they were about to report *already fixed on main* and stopped only because the answer was
unexpected. That is worth recording precisely because it is **not a method**. Surprise fires only
when you already hold a belief strong enough to be violated, which is exactly the case where you
needed the check least; the dangerous instances are the ones that confirm what you expected, and
those produce no surprise at all. So the receipt does not soften instance 8's rule — it shows
what stands in for the rule when the rule is skipped, and how thin it is. Re-deriving through a
second bound is the method; noticing that a zero felt wrong is luck that happened to be
available.

## Instance 9 — a failed job's log is unreadable while a sibling lane still runs (2026-08-26)

Observed directly while diagnosing a build-lane failure: the `build` job was
`completed/failure`, and fetching its log returned

```
run <id> is still in progress; logs will be available when it is complete
```

because the `floor` lane in the same run was still executing — and the floor runs ~50 minutes.

**So a terminal failure on one lane is undiagnosable for as long as an unrelated sibling keeps
running.** The two-lane split buys parallelism and pays for it in *diagnosis latency on the
failing half*, which is the half that matters. The verdict is available immediately at job grain;
only the evidence is withheld.

This is not an attribution failure — the subject is established and the field is honest. It is
listed here because it sets the floor on how fast any of the checks above can actually be
applied, and because the tempting response is to diagnose from the step name instead of waiting.
Ruling out one cause (steps 1–5 succeeded, so not the runner-setup class) **is not establishing
another**.

**Check:** read `status`/`conclusion` per job immediately; wait for the run to terminate before
reading the log; do not substitute the step name for the log.


## Instance 10 — discarding a LIVE instrument as stale (2026-08-26)

Every instance above is a variant of *trusting an instrument that did not measure what you
think*. This is its mirror, and it is worse.

A lane dispatched a measurement, then **killed it as obsolete** on the grounds that it was built
on a superseded patch set. It completed anyway — and the reasoning was wrong. The change they
believed invalidated it (hoisting a conjunct out of a branch arm) touches only the
`CarrierRefused` path; their plant reaches `Rendered`. **The measurement never traversed the
edited path**, so it was valid the whole time. It was also the only instrument that answered the
open question, and it was nearly thrown away.

**The two errors are not symmetric in cost.** Trusting a stale instrument produces a *wrong
answer*, which review can catch — every instance in this document was caught. Discarding a live
one produces *no answer*, and there is nothing left for review to look at. **The second failure
is silent by construction**, which is exactly the property that makes the empty-observation
narrow worse than the absorbing fallback.

There is also a standing pressure toward it: a document like this one trains suspicion of
instruments, and suspicion applied indiscriminately kills good measurements. Discipline about
*not trusting stale instruments* is not the same virtue as discipline about *establishing
staleness*.

**Check:** before discarding a measurement as stale, **name the path the change actually touches
and establish that your measurement traverses it.** One sentence. If it does not traverse the
changed path, the measurement is live regardless of when it was dispatched.

## Instance 11 — a commit on `main` is not the setting in effect in the job (2026-08-26)

The question was whether a known CI failure class had recurred *after* its repair landed. The
repair is `cache: false` on `setup-rust-toolchain`, commit `a6871631c6` (#9203), authored
2026-08-25 21:21 UTC. Three specimens are dated 23:33, 23:53, and 2026-08-26 00:40 UTC.

The available conclusion is immediate and wrong-shaped: every specimen postdates the repair,
therefore the repair does not work. The dates are correct. The inference joins two different
subjects — the state of `main` at a timestamp, and the configuration that a particular job
actually ran under — on a key (wall-clock order) that establishes neither. A job runs the
workflow file from the ref it was triggered on, not from `main`; a branch behind the repair, a
re-run of an older attempt, or a job carried forward from a prior attempt all break the join
silently, and all three break it in the direction that makes the repair look guilty.

The measurement that closes it is not a date at all. `setup-rust-toolchain` echoes its resolved
inputs into the job log, so the job states its own configuration:

```
gh api repos/gunb-ai/gunbc/actions/jobs/<job_id>/logs --allow-escape-sequences \
  | sed 's/\x1b\[[0-9;]*m//g' | grep -m1 -a 'cache: false'
```

Run against each of the three **failing build jobs** — `98002876566`, `98007396701`,
`98019746954` — all three print `cache: false` and `components: rustfmt`. The repair was live in
the jobs that failed, three for three, and the conclusion now rests on the jobs' own testimony
rather than on repository chronology.

Two things this instance is careful about, and they point opposite ways.

The date argument and the log argument reach the same verdict here, which is exactly why the
instance is worth recording: a check that agrees with the shortcut on the day you run it teaches
nothing, and gets dropped. Its value is the cases where they diverge, none of which announce
themselves.

And confirming recurrence establishes that the repair does not *prevent* the class — not that
the reasoning behind it was wrong. The carrier rejected a different repair (per-slot
`CARGO_HOME`) on cost, as a fix for cache-action cleanup deleting files from a shared
`~/.cargo`. The specimens here die inside `Install Rust toolchain`, before any cache interaction
occurs. That is a second mechanism sharing one symptom, and reading the recurrence as a refutation
of the recorded cost argument would be the same wrong join one layer up: same observable,
different subject.

### Receipt added 2026-08-26 01:45 UTC — the same check, run against a docs-only diff

The instance above was written from three specimens whose diffs all touched code, so "the change
is not the cause" rested on the configuration read rather than on the diff. A fourth specimen
removes even that dependency.

PR #9260 — **one markdown file, no Rust, no `.dag`, no yml** — failed its build. Run
`32919310791`, job `98029597779`, runner `srv3-19`. `cache: false` echoed at 01:32:39. At
01:45:00.538:

```
required-ci: regen refused: normalize emitted src/extdeps_languages_rust_derive_contracts.rs:
spawn rustfmt: Text file busy (os error 26) -- program /home/ghrunner/.cargo/bin/rustfmt was
resolved at admission from PATH, and ran there: `--version` was executed successfully before
this phase began. So it has been removed, replaced or made unusable while this run was executing;
this is not a missing or broken installation
```

Two other jobs on the **same host** ran `Install Rust toolchain`:

| job | slot | install window |
|---|---|---|
| `98031944651` | `srv3-14` | 01:44:59 → 01:45:00 |
| `98031944764` | `srv3-21` | 01:44:59 → 01:45:01 |

The failure instant falls inside both. Three slots on one host share `/home/ghrunner`.

Two things this receipt establishes that the date comparison could not. The diff is markdown, so
attributing the red to the change is not merely unproven but impossible — which is the instance's
point reached from the opposite direction: the first three specimens needed the job's own config to
exonerate the change, and this one needs nothing. And the binary replaced is **rustfmt**, under
**regen normalize** — so the class reaches the normalizer that the emitted stage0 artifact's fixed
point is computed against. `ETXTBSY` is the loud case, where the exec was caught in the act; the
quiet case differs only in whether the replacement lands *between* two reads rather than during
one, and requires no additional mechanism.

The generalisation for this brief: **a red on a diff that cannot produce it is a measurement of the
environment, not of the branch** — and it is only available to a reader who checked what the diff
contained before reading the failure as a verdict on it.


## Instance 12 — the aggregator renders a CANCELLED run as a FAILED one (2026-08-26)

The rollup on #9244 showed `witnesses: FAILURE`. The job's log:

```
##[error]a required lane did not succeed (build=cancelled floor=cancelled)
```

Neither lane executed a step. The run had been lawfully superseded by a push.

This one is **ours**, not GitHub's. The aggregator declares `always()` so that the required check
context is never reported as SKIPPED — a sound intent, since a skipped required check cannot gate.
The consequence is that it also runs when its lanes were *cancelled*, and reports that as a
failure. **"The run was cancelled" and "the run found a defect" are different states with opposite
owners and opposite repairs** — push again or ignore, versus fix your code — collapsed onto one
conclusion. It is the conflation this repository's own failure list names, in our own workflow.

It is also the mirror of instances 5–6: those resolve toward *nothing is wrong* and go
uninvestigated. This resolves toward *something is broken* and costs investigation instead. Two
sessions ran the same investigation on this PR within one hour before establishing it was noise.

**It compounds with a second reading error.** A check rollup lists *every* check run for a SHA, not
the latest per name. On #9244 at head `347bc88d795`:

| name | earlier | later |
|---|---|---|
| build | cancelled 01:03:33 | **success 01:16:24** |
| floor | cancelled 01:03:32 | **success 01:16:24** |
| witnesses | failure 01:16:19 | **success 02:06:33** |

Every latest attempt is green. Read without grouping by name and sorting by time, the rollup hands
you a red that a later run already replaced — and `dashboard-ops` reported `checks_state=failing`
for exactly that reason while the PR was in fact CLEAN.

The check that closes it:

```
gh api "repos/OWNER/REPO/commits/<sha>/check-runs?per_page=100" \
  -q '.check_runs[]|"\(.name) \(.status)/\(.conclusion) \(.started_at)"' | sort
```

Generalisation: **a check conclusion answers "what did this run report", never "did this run
execute"** — and a rollup answers "what has ever been reported for this SHA", never "what is the
current state". Both gaps are invisible at the summary level, and both were being read as verdicts.


### Instance 12, continued — the readiness endpoint reads the stale row, indefinitely

The half that makes instance 12 expensive is not the aggregator's conflation. A superseded failure
that *disappeared* would cost one confused glance. This one persists, and the dashboard's own
readiness endpoint reads it:

```
{"key":"checks","label":"Checks are not failing or pending","ok":false,"actual":"failing"}
```

`merge_criteria` is not taking latest-per-name, so on #9244 it reported `checks_state: "failing"`
while the PR was in fact `CLEAN` with every latest check green. **That field is what the working
agreement names as the source of truth for merge readiness** — so on any PR whose run was
superseded, the designated authority reports not-ready indefinitely, and no amount of waiting or
re-running clears it.

Two readers reached the same wrong conclusion from it inside one hour, and the failure mode is
self-reinforcing: a reviewer who checks readiness, sees `failing`, and stops has no reason to
suspect the field rather than the PR.

The generalisation this forces is sharper than instance 12's own: **an aggregate readiness verdict
inherits every conflation of the fields it reads, and reports them with more authority than any of
them carried.** The rollup at least shows both rows and lets a careful reader sort them; the
readiness field shows one boolean and hides that a choice was made.


### Instance 12, third reading — two fields of ONE response disagreeing about one fact

On #9251, the same `dashboard-ops reviews` payload reports both:

```
readiness.has_request_changes : True
merge_criteria.request_changes_count : 0
merge_criteria.requirements[request_changes] : ok=True  "none (stale, re-review pending: claude)"
```

`merge_criteria` is *correct* here — a `request_changes` posted on superseded SHA `50cbf333b`,
followed by an `approve` on the current head `f53e351b3`, is properly marked stale and does not
block. The top-level `readiness` block reports the same underlying review as a live blocker.

So the earlier lesson needs qualifying rather than repeating: the dashboard is not uniformly
staleness-blind. **One block of the response applies recency and another does not**, and they
disagree in the same payload. Reading the first field nearly produced a false blocker report on a
PR that is in fact clean — the opposite direction from #9244, where the readiness field was the one
that was wrong.

That is the more useful generalisation, and it is not "distrust the dashboard": **when one response
answers the same question twice, the answers are not redundant — one of them is derived under
different rules, and which is authoritative has to be established rather than assumed.**

A separate lag, distinguished rather than merged into the above: `merge_criteria.checks_state` read
`pending` on #9251 while GitHub reported three completed/success and **zero** non-completed check
runs on the head. The last of those finished ~1 minute earlier, so this is most likely snapshot lag
rather than the latest-per-name defect of #9244. Recording it as *undistinguished* — it would take a
second reading after the cache turns over to tell the two apart, and calling it either one now would
be exactly the unmeasured attribution this brief is about.


### The discriminator (deep-ant, measured 2026-08-26)

The `checks_state` question above resolved: it was **lag**, ~5 minutes, cleared unaided
(`pending` at 02:26:16 / 02:26:47 / 02:27:18, `passing` at 02:27:48; last check run completed
02:22:50). #9251 then read `ready=True` with no failing requirements.

What makes that interpretable is the check done *before* the wait: #9251's head carried exactly
three check runs, all SUCCESS, no cancelled generation and nothing non-completed — so there was no
stale artifact for the endpoint to be reading, and lag was the only remaining candidate. The poll
confirmed rather than discovered it.

So `checks_state` is wrong in **both** directions, with opposite handling:

| | cause | lifetime | action |
|---|---|---|---|
| stale-failing (#9244) | superseded generation in the rollup | **persists indefinitely** | take latest-per-name yourself |
| lag-pending (#9251) | snapshot not caught up | transient, ~5 min | wait; needs nothing |

They look alike at a glance and either produces a false blocker. **The discriminator is the
rollup, not the field: count the non-completed and non-latest runs.** A clean rollup means any
disagreement is lag. A rollup carrying a superseded generation means it is the defect, and it will
never clear.


---

## Instance 13 — attributing an outage to the wrong scope (2026-08-26)

Having found a review that crashed with `MODULE_NOT_FOUND` on the review host, I escalated it as
possibly affecting *every scheduled review in the fleet* — worth checking, I said, before it was
diagnosed one PR at a time.

That was an overclaim, and the check that refutes it is the same shape as every other entry here:
query the population rather than reasoning from the one specimen.

| PR | started | status | sha |
|---|---|---|---|
| #9244 | 01:17:43 | failed `MODULE_NOT_FOUND` | `347bc88d7` |
| #9260 | 01:19:48 | failed `MODULE_NOT_FOUND` | `fdb5c250f` |
| #9260 | 01:20:47 | **posted** `request_changes` | `fdb5c250f` |
| #9260 | 01:41:14 | **posted** `approve` | `777fe2d0e` |

A three-minute outage, not a fleet-wide breakage. #9260 is decisive because it carries a crash and
a success on either side of the boundary — same reviewer, same host, **same SHA**, 59 seconds
apart. That single pair rules out the systemic reading in a way that no amount of staring at
#9244's failure could.

The error is the mirror of instance 4: there, a missing observation was read as "nothing is
affected"; here, one observed failure was read as "everything is affected". Both substitute the
convenient scope for the measured one. And the practical cost was concrete — it produced advice to
another session ("do not push, the reviewer is broken") whose premise was false, on a PR that
repairs main's build.

**A failure's scope is a separate measurement from the failure.** One specimen establishes that a
thing can fail; only the population establishes how much is failing.


### A live confirmation of instance 5, on this brief's own PR

#9260 at head `777fe2d0e`, latest per name:

```
build      completed/failure     <- the rustup ETXTBSY
floor      completed/SUCCESS
witnesses  completed/failure     <- the always() arm, over the failed build
```

The **floor passed**. The run's conclusion is `failure`, and one of this session's two monitors
reported the run's conclusion while the other reported the floor check's — the same event rendering
as success and as failure depending on which level was queried. That is instance 5 exactly, observed
live rather than reconstructed, on the branch documenting it.

## Instance 14 — the face with no diagnostic at all

Found by witty-swift-77 on gunbc#9273; verified here before amplification; population measured
here. Recorded last because it is the one instance that does not present as an attribution
error at the *reading* layer — it is manufactured one layer down, in the evidence itself.

Instances 1–13 are all failures of a reader: a query on the wrong key, a bounded scan reporting
its boundary as an answer, a field consulted that answers a narrower question than the one asked.
Every one of them is repairable by reading more carefully, and every one leaves a discriminating
artifact in place for the careful reader to find. This one does not. The discriminating artifact
is destroyed before any reader arrives.

`v2.test.execution.emit_host_field_access_equals_eval` folds its run verdict to a `Bool`:

```
fn emit_field_eval_verdict_is_pass(run: TestClaimRun<Node, RuntimeValue>) -> Bool {
  match run.verdict {
    Pass                                             => true
    SemanticMismatch { actual: _, falsification: _ } => false
    BuildFailed      { actual: _, diagnostic: _ }    => false
    RunFailed        { actual: _, diagnostic: _ }    => false
    Deferred         { actual: _, diagnostic: _ }    => false
  }
}
```

A cargo spawn that never happened and a genuine emitter regression both render as
`returned Bool(false)`. The three faces of the shared-rustup class catalogued elsewhere in this
brief — ETXTBSY, ENOENT, exit 126 — are all errnos, and all three carry a sentence naming the
class. This face carries nothing. The usual tell (*it executed successfully moments before this
phase began, so it was there and then it was not*) is unavailable, because no diagnostic is
produced to hold it.

**The match is exhaustive and wrong, and the exhaustiveness is the concealment.** Five arms, no
wildcard: nothing is missing, no warning fires, and neither a reviewer reading the match nor a
wildcard-free lens has anything to catch. This is the corpus's own *total at the level examined,
blind one level down*, and it exhibits the tell that class names — `a bare binding in the arm
that carries the payload`. Each non-`Pass` arm binds `diagnostic: _`. The information needed to
identify the failure **exists in the verdict** and is discarded one line before it would be
reported.

**Population — and this brief must state its denominators, being a document about counts
detached from their subject.** Two independent measurements at `d311b1ceab`, answering different
questions and corroborating each other rather than competing:

| scope | grain | count |
|---|---|---|
| `src/` + `dag/`, `BuildFailed`/`RunFailed` arms | files / arms | **16 files, 70 arms** |
| `src/v2/test/`, exact literal `(BuildFailed\|RunFailed\|Deferred) { actual: _, diagnostic: _ } => false` | files / arms | **19 files, 108 arms** |
| `src/v2/test/`, arms binding `diagnostic: _` | arms | **41 arms** |
| `src/v2/`, fold-grain fns | folds / files | **23 folds, 13 files** |
| narrower root, fold-grain fns | folds / files | **24 folds, 11 files** (12 `is_pass`, 12 `is_semantic_mismatch`) |

**Three lanes measured this independently and got five different numbers, and none of them is
wrong.** Each names a different root and a different pattern; two of the three fold-grain figures
(23/13 and 24/11) differ only by root and land in the same place. No figure here is a census.
That the counts disagree while the shape does not is this brief's own subject arriving in its own
evidence — which is why every row above carries its root, and why the fold-grain rows are the
ones quoted downstream. **Fold grain is the one that matters**, measured by witty-swift-77: a repair edits
folds, not arms, so the scoping conversation is 24 folds — not 108 of anything. The family is
effectively every `emit_host_*_equals_eval` witness (`add`, `match`, `complement`, `fold_closure`,
`field_access`, `variant_construct`, `meet_join`, `loop`, `record_construct`, `emit_host_call`,
`classical_not_ingested`). So during an environmental storm — and calm-bee-813 measured 32% of
build failures in one hour as environmental — any of eleven witness files can go red as a bare
`Bool(false)` and read as an emitter regression.

**The twin fold does not escape it, and the argument that it does is the trap.** The obvious
narrowing is that only the `is_pass` direction misleads, because a build failure genuinely *is
not* a semantic mismatch, so `false` is the correct answer in `is_semantic_mismatch`. That is
true of the fold and false of the witness. A spawn failure yields `BuildFailed`, so
`is_semantic_mismatch` returns `false`, so a witness asserting *the refusal wall fires* returns
`false` — red, bare `Bool`, no diagnostic. Identical loss. Only the wrong conclusion differs:
`is_pass` false reads *the emitter produced wrong output*; `is_semantic_mismatch` false reads
*the refusal wall stopped firing*. Both send a lane into the compiler; neither carries the
diagnostic that would turn it around. witty-swift-77 caught this against their own finding,
having nearly published the halved population as good news — a narrowing that felt principled
and was a technicality.

The receipt is on the PR that surfaced it: gunbc#9273's two failures are one of each direction,
same window and same transport, and which one collapsed to `Bool(false)` versus threw a type
error was not a property of the fold.

What stopped the misattribution in the observed case was a sibling witness failing four seconds
later with a visible errno. witty-swift-77 named that correctly and against their own interest:
*pure luck of adjacency, not a property of the evidence*.

### Why this one belongs in this brief rather than beside the rustup root cause

The two are independent, and conflating them would repeat the brief's own subject. Repairing the
shared-rustup rewrite lowers the **frequency** of this misread; it does not touch the **provenance
loss**, which survives any environmental fix and reproduces exactly under the next storm from any
other cause. A fix that makes the symptom rare while leaving the evidence unable to distinguish
its own causes is the absorbing fallback wearing a repair's clothing.

### The generalisation, which supersedes this brief's earlier one

Instances 1–13 forced: *a verdict with no established subject is not a verdict.*

This instance forces the stronger form. There, the subject existed and the reader failed to
establish it. Here the subject is **unrecoverable in principle** from the artifact handed to the
reader, because the fold that produced it was not injective on the distinction that matters. So:

> **Evidence that cannot distinguish its own failure causes is not evidence of any of them.**

And the repair is correspondingly not a reading discipline but a carrier change: a stage that can
prevent a comparison from executing must participate in the verdict, so that *the comparison never
ran* and *the comparison disagreed* have no shared spelling. Every other instance in this brief
could have been prevented by a more careful reader. This one could not have been.

### The repair is not a blank page — the precedent is shipped and executing

**Correction, 2026-08-26 — an earlier revision of this section cited a symbol that does not
exist, and the correction belongs here rather than in a footnote, because this brief's whole
subject is claims detached from their referent.** That revision named `cargo_standing` in
gunbc#9273 as splitting a failed build carrying zero coded rustc diagnostics (`Unestablished`)
from one carrying them (`Refuted`). **No such symbol exists** — not in #9273, whose diff is
10,291 bytes and contains none of `standing`, `Unestablished`, or `Refuted`, and not anywhere in
the tree. The author of this brief relayed a mechanism he had verified together with a name he
had not, then repeated it to three parties, one of whom cited it back as established. The
mechanism was real; **the name is the part that travels**, and it travelled alone.

The real precedent is stronger than the invented one, and it is on `main` rather than in any PR.
`dag/tools/emission_entry_instrument.dag` defines `EmissionMeasurement` with **five** arms —
`EmissionSubjectUnestablished`, `EmissionUnreached`, `EmissionEmitRefused`,
`EmissionCargoUnreached`, `EmissionMeasured` — and states the principle in its own header:

> The measurement is COMPLETE whenever the instrument reached a verdict about the entry, **which
> includes an entry the compiler refused**. It is INCOMPLETE only where the instrument itself
> could not run. Conflating the two is how an instrument reports its own breakage as a finding
> about its subject.

`gunbc#9273` does contribute a real arm to this family — `EmitOutputTruncated` on
`EmitDiagnosticRead` in `gunbc.emit_diagnostic_observation`, separating *the host kept a bounded
tail so the marker was discarded* from *the compiler did not scope*. That is instrument-failure
versus subject-failure, authored from a measured receipt. It is simply about truncation, and
carries none of the vocabulary the retracted sentence attached to it.

**The retraction changes the design conclusion, which is why it is not a cosmetic fix.** The
invented citation described a *two-valued* split — host failed, subject failed — and the real
carrier is not two-valued at all. `emission_measurement_completed` returns `true` for
`EmissionEmitRefused` and `false` for `EmissionUnreached`: a subject's refusal is a **completed**
measurement. So the carrier separates two independent questions, and only the first is the
enumeration:

1. **Did the instrument reach a verdict?** — completeness of the *measurement*
2. **What is the verdict?** — including *nobody failed*

*Who failed* is not the top-level axis; it is a projection over arms that already answer (1).
Compressing five arms into the two-armed sentence this brief wanted is the same move that
produced the fake symbol — and had it been built, the carrier would have had no way to type a
result where the instrument ran, the subject is correct, and the population is *deliberately
incomplete*: nobody failed, so a who-failed enumeration must fabricate an answer. That failure
mode was caught by review before construction, not by the author.

So the carrier change has a worked shape available to it: a stage that can prevent a comparison
from executing must participate in the verdict, so that *never ran* and *ran and disagreed* have
no shared spelling. Note what this makes of the ladder — the repair is not a better diagnostic or
a more careful reader, both of which leave the bad state writable. It removes the spelling. The
distinction stops being something evidence must be trusted to preserve and becomes something the
carrier cannot lose.

## Instance 15 — the control that cannot flip for the right reason

Found by deep-ant-102 by execution, not by reading; verified here against the tree. Recorded
because it **widens instance 14's class past the mechanism instance 14 blamed**, which means the
repair instance 14 proposes would not have caught it.

Running the `00_compile` self-host behavioral receipt to termination (RED at cargo, 16 errors,
subject `d311b1ceab`), its discriminating RED is a conjunction — `dag/tools/self_host_curated_seed_linked_harness.dag`,
kernel `cssl_seed_linked_behavioral_receipt`:

```
    && build.success
    && pass_run.success && string_contains(s: pass_run.stdout, pattern: pass_marker)
    && (fault_run.success == false)
```

The last conjunct exists to prove the harness can tell a faulted run from a clean one. But when
cargo fails there is **no witness binary**, so the plain run and the `--inject-fault` run both
exit 127 — and `(fault_run.success == false)` is satisfied by *the binary does not exist* rather
than by *the fault was detected*.

**The receipt's outcome is still correct**, and that is the interesting part rather than a
mitigation. `build.success` and the `pass_marker` conjunct sit in the same chain and both go
false, so the receipt returns false as it should. What is broken is not the verdict but the
**evidence for the verdict**: the one arm whose job is to establish that this harness can
discriminate carries zero information in precisely the case where discrimination is in question.
A reader auditing *does this receipt have a discriminating control* counts it and finds one. It
is a control that cannot flip for the right reason.

### Why this is not instance 14 again

Instance 14's twenty-four folds **discard a payload that exists** — `diagnostic: _`, thrown away
one line before it would be reported. The proposed repair follows from that diagnosis: make the
carrier keep what the fold was dropping.

This instance has **no payload to discard**. `fault_run.success` is a boolean that never carried
a diagnostic; the predicate is simply true for two unrelated causes. Nothing was lost, because
nothing was ever held. So the fold repair — however correct for those twenty-four — does not
reach this, and a sweep that fixed every fold in the corpus would leave this control exactly as
hollow as it is now.

### The generalisation, corrected

Instance 14 proposed: *evidence that cannot distinguish its own failure causes is not evidence of
any of them.* That sentence survives this instance unchanged — it covers a hollow conjunct as
cleanly as a lossy fold. What does **not** survive is the diagnosis attached to it. Instance 14
located the defect in **non-injectivity of a fold over a payload**, and this instance is
non-injective with no fold and no payload. So the mechanism is one layer more general:

> **A predicate whose truth is reachable by a cause unrelated to what it asserts is not evidence,
> whether or not anything was discarded to make it so.**

The practical consequence is a scoping one, and it cuts against the tidier plan: the class is
**not** confined to folds and is therefore **not** closed by repairing folds. Any audit that
enumerates `match` arms will report full coverage while every conjunct of this shape stays
standing — which is instance 12's shape (a bounded query reporting its boundary as an answer)
reappearing inside the remediation rather than inside the diagnosis.

## The decision rule for gate-produced diffs — the one positive control in this document

Every other entry here is an error. This one is a rule that produced the right answer twice in
opposite directions in a single commit, and it is recorded because both of the obvious rules are
wrong and are wrong in *opposite* directions.

A build or a gate leaves diffs in your worktree that you did not type. Two tempting policies:

- **"Regenerate everything the gate touches."** Absorbs other authors' pending drift into your PR,
  hides it from the person who owns it, and makes your diff unreviewable.
- **"Touch nothing you did not author."** Leaves your own change's projection uncommitted, so the
  next consumer discovers it as a failure whose cause is invisible from where it fires.

neat-pike-374 hit both in one commit (`d00d5306f2a`) and applied a third rule:

> Is this the projection of an authority **this branch changed**?

`Cargo.lock` — yes: the dependency-tier move was their own #9243, and a lock disagreeing with the
manifests breaks `--locked` builds. **Committed.** `fleet-converge.yml` — no: pre-existing drift
from #9167. **Deliberately excluded**, and reported rather than silently absorbed.

**The distinction that decides it is authorship of the CAUSE, not whether the diff appeared in
your worktree.** Appearing in your worktree is a fact about what you happened to run; it carries
no information about who owns the fact. That is this brief's subject in its constructive form:
the same conflation between *an observation I made* and *a fact about the thing observed*, caught
before it produced an error rather than after.

The receipt on the excluded half is the part that makes it a rule rather than a preference. `main`
genuinely fails `cargo metadata --locked` (exit 101) while the repairing branch passes (exit 0) —
so the absorbed diff was load-bearing and the excluded one was somebody else's, and both calls
were checkable rather than tasteful. `fleet-converge.yml` remains drifted and unclaimed, which is
the correct outcome: visible, owned by its author, not laundered through an unrelated PR.

## Where the answer already lived — and why five lanes did not find it

The instances above describe a class: evidence that cannot distinguish its own failure causes.
Over one night, five lanes reached it independently from five subjects, and four of them began
designing a carrier for it. **A complete, correct, cited carrier for exactly this already existed
in `std`.** This section records where, so the sixth lane consumes it instead of re-deriving it.

### It exists, at two grains, in `dag/std/claim_evidence.dag`

```
type ClaimInformationState { support: Bool, challenge: Bool }
```

The two independent bits, literally — four states as their product: support only, challenge only,
both/conflicted, neither/insufficient. `claim_information_state()` derives it by folding evidence
link directions over a `ClaimAssessment`.

```
type ClaimRequirementReadiness<F, C, A, FidelityBoundary, Independence>
  = ClaimRequirementReady        { claim, supporting }
  | ClaimRequirementChallenged   { claim, challenging }
  | ClaimRequirementConflicted   { claim, supporting, challenging }
  | ClaimRequirementMissing      { claim }
  | ClaimRequirementEvidenceRefused { claim, supporting, challenging, refusals }
```

The named projection — the four states plus admission-refused as a distinct fifth — generic over
five type parameters, with a fold. **The worked instantiation is
`dag/gunbc/source_integration_landing_spine.dag`**, which binds concrete type arguments and folds
the result; it is the template a new consumer should copy. It was, at the time of writing, the
*only* consumer outside the declaring module.

The grounding is cited, not invented: `extdeps.assurance.claim_evidence`
`belnap_four_valued_information_authority`, and `docs/plans/dag-scm-design.md` states the
requirement in prose — *"preserves two independent bits — support and challenge: support only,
challenge only, both/conflicted, or neither/insufficient. This is the Belnap–Dunn shape … not
permission to reuse that logic's final type names before DFS."*

### Why the arity is not a design choice

Every lane that reached this class tried to pick an arity by counting the subjects it had met —
two, then three, then five. Each count was falsified by the next subject within the hour. The
arity is not ours to choose: **information states are four-valued because support and challenge
are two independent bits**, and a count of observed cases is convention standing where necessity
was available. The count is also unstable by construction, which is why it kept moving.

One consequence worth stating, because no lane derived it from cases: **told-both is real and
nobody invents it.** A collision where two records are *both legitimate* and the locator cannot
represent both is not a gap and carries no outstanding obligation — it is contradictory
information. Lanes reasoning from observed failures reach *absent* easily and *conflicted* never,
because a conflict does not look like a failure. That cell is the strongest argument for
realizing a grounded shape rather than enumerating one.

Distinguish it from what is *not* a value: a result that is complete and correct but deliberately
partial, and a set of identities planned and never reached, are **obligations carried alongside a
reached verdict**, not truth values of the claim. Two lanes described these in nearly identical
words — *a property of the population, not of the measurement or of any party* — precisely
because there was no slot for an obligation and it presented as a missing arm.

### Declared boundary — what this carrier does not reach

A four-valued *standing* carrier answers whether a claim is supported, challenged, both, or
neither. It says nothing about **which** fault a refusal represents when both alternatives are
subject faults. A seam whose two refusal paths both return `Bool` with one shared downstream
diagnostic loses *which arm refused*, one level below this distinction, and this carrier cannot
see it. That is a refusal vocabulary at the seam, not a standing. **Stated here so the carrier is
never cited as covering it** — a check credited with coverage it structurally cannot provide is
the coverage-by-illusion failure occurring at the mechanism built to prevent it.

### Why it was missed, which is the transferable part

Two independent lanes grepped for the concept and concluded it was absent:

```
Neither | NoInformation | Contradiction | FourValued | Belnap
ToldBoth | ToldNothing | NoInformation | FourValued
```

Both empty. Both read the empty as absence *of the concept*. But the design document had said in
advance that those names would not be there — and named the four states in prose one line above
the warning. The corpus did exactly what §3 asks (model the shape; do not adopt an upstream's
final type names before DFS), and both lanes penalised it for complying.

> **A grep proves the absence of a spelling. It never proves the absence of a concept.**

When the question is *does this concept exist*, the query must run over the concept's **structure**
— a coproduct with a support arm, a challenge arm, a both arm, a neither arm — or over the
**authority that would ground it**, following its imports to their declarations. Both lanes held
that thread and let go of it: `std.claim_evidence` imports
`belnap_four_valued_information_authority` twice, and neither opened the module doing the
importing. The citation was the one structural, resolvable edge available, and both measured
around it.

There is a second, sharper reason the grep could not win. The module *does* document itself —
`claim_evidence_boundary_note` states that `ClaimAssessment` is the four-valued information view
and that `ClaimInformationState` is the product of two independent bits, "so conflict and
insufficiency cannot collapse into chronology or a Boolean last observation." That is this class's
thesis, written in the module before any of these lanes existed. **It lives in a
`data … : NonEmptyStr` row** — prose the substrate cannot see (§4c). No lens reads it, no index
surfaces it, and grep finds it only if you already guessed the author's vocabulary. The knowledge
was not missing; it was in the one representation nothing can consume. That is what five
re-derivations cost, and the repair is discoverability, not construction.

## The shared shape

Instances 1–3 are *the subject was substituted*; instance 4 is *there was no subject*; instances
5–8 are *the subject was fine and the field or query you read was not the one holding the
answer*; instance 9 is *the answer exists and is withheld*; instance 10 is *the answer existed
and was thrown away*; instance 11 is *the answer was inferred from repository state instead of
read off the job*; instance 12 is *the answer conflated two states with opposite repairs, and the
readiness field that aggregated it inherited the conflation*; instance 13 is *one observed failure
was read as the scope of the failure*. All are the same underlying error — **treating a run as a measurement of a change without
establishing that it measured that change** — and all are cheap to close:

| establish | command |
|---|---|
| which ref was built | `gh api …/runs/<id> -q .head_sha` |
| whether it is a merge ref | `gh api …/runs/<id> -q .event` |
| what that ref was for | `git log -1 --format='%s' <sha>` |
| whether it reached a verdict | `gh api …/runs/<id>/attempts/<n>/jobs -q '.jobs[]\|"\(.name) \(.status)/\(.conclusion)"'` |
| what config the job actually ran under | `gh api …/jobs/<job_id>/logs --allow-escape-sequences \| sed 's/\x1b\[[0-9;]*m//g'` |
| the CURRENT state, not every state ever | `gh api …/commits/<sha>/check-runs -q '.check_runs[]\|"\(.name) \(.status)/\(.conclusion) \(.started_at)"' \| sort` |
| whether a failure is broad or local | query the whole review/run population, not the one specimen |

Tonight, each of the first four was skipped exactly once, by four different lanes, and each skip
produced a confident conclusion that was wrong. The fifth row was added 2026-08-26 (instance 11):
the first four establish *which change* a run measured, and it establishes *under what
configuration* — a question repository chronology looks able to answer and cannot.

The fourth row **was `-q .conclusion` until 2026-08-25**, when it was measured wrong — the
run-level field reports only the *later* of several job outcomes, so it conceals a terminal
failure behind a cancellation. Instance 5 carries the receipt.

## What the corpus already says, and why it did not prevent this

The repository's own failure list names the **empty-observation narrow** — ⊥-as-ignorance
rendered as ⊥-as-answer — and instance 4 is precisely that. Instances 1–3 are its sibling in a
position the list does not currently name: not the observation collapsing, but *the observation
being about a different subject than the one claimed*. A number with no ref is not a number; the
generalisation these instances force is that **a verdict with no established subject is not
a verdict**, and the subject of a CI run is not knowable from the run alone.

Instances 5–6 extend it in a direction the 2026-08-23 draft did not anticipate, and both are the
**flattering** direction: a concealed failure renders as a cancellation, and a supersession query
on the wrong key renders as "nothing superseded it." Neither produces an alarming wrong answer.
Both produce a reassuring one. That asymmetry is the reason this class keeps costing time —
an error that resolves toward *nothing is wrong* is not investigated.

Instances 12–13 close the symmetry, and are the reason the list is not simply "beware flattering
answers". Instance 12 resolves toward *something is broken* — it costs investigation rather than
concealing risk, which is the less dangerous direction and still expensive: two sessions ran the
same dead end on one PR inside an hour. Instance 13 is the author of this brief making the
mirror of instance 4's error in the same session it was written up — reading one observed failure
as the scope of the failure.

That last one is the honest summary of the whole document. Every entry here was written by someone
who already knew the class and made an instance of it anyway. The commands in the table are not a
reminder to be careful; they exist precisely because care does not survive contact with a plausible
answer.
