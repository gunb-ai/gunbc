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

**A grep pattern is a bounded query too**, and the same night produced the receipt. A lane was
about to report that a gate did not print cargo's diagnostics, on the strength of grepping its
log and getting zero hits. The gate prints them — prefixed `cargo|`, which their pattern did not
match. In their words: *that zero is a claim about my instrument, not about the program.* The
bound need not be a `per_page` limit; a filter expression is a bound, and an empty result from
one establishes nothing until the filter itself is checked against a known-present line.

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

## The shared shape

Instances 1–3 are *the subject was substituted*; instance 4 is *there was no subject*; instances
5–8 are *the subject was fine and the field or query you read was not the one holding the
answer*; instance 9 is *the answer exists and is withheld*; instance 10 is *the answer existed
and was thrown away*; instance 11 is *the answer was inferred from repository state instead of
read off the job*. All are the same underlying error — **treating a run as a measurement of a change without
establishing that it measured that change** — and all are cheap to close:

| establish | command |
|---|---|
| which ref was built | `gh api …/runs/<id> -q .head_sha` |
| whether it is a merge ref | `gh api …/runs/<id> -q .event` |
| what that ref was for | `git log -1 --format='%s' <sha>` |
| whether it reached a verdict | `gh api …/runs/<id>/attempts/<n>/jobs -q '.jobs[]\|"\(.name) \(.status)/\(.conclusion)"'` |
| what config the job actually ran under | `gh api …/jobs/<job_id>/logs --allow-escape-sequences \| sed 's/\x1b\[[0-9;]*m//g'` |

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
