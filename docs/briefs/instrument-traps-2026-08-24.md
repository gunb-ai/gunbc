# Instrument traps measured on 2026-08-24

Six ways an instrument in this repository answered a **narrower question than it was
asked** in one night, across two sessions and nine lanes. Each one is recorded with what
was measured, not with advice — the advice is always the same and always insufficient
("check your instrument"), and what actually transfers is the specific shape.

Every one of these produced output that was TRUE. None of them was a bug in the tool.
In each case the reader supplied a binding the output did not state.

---

## 1. `gh run rerun` is not a second observation

A rerun reuses the **original merge commit**, so it re-measures the same stale base and
reproduces the identical error. Only a new head recomputes the merge ref.

**Why it is expensive:** the failure looks exactly like a successful reproduction — same
error, same place, twice — which is normally the strongest confirmation signal available.
The defect defeats the instinct rather than requiring carelessness.

*Found by bold-raven-901, who asserted the wrong remedy on a PR and corrected it there.*

## 2. ctrl-build has TWO head behaviours and nothing in the output distinguishes them

- fetch-a-named-commit → `HEAD` genuinely is that commit
- apply-a-diff-onto-the-pushed-base → the echoed head is **main's**, not yours

Both were observed in this session, same tool, same day. The only distinguishing evidence
is the log preamble.

**The rule is not "remember which mode you are in."** It is: *never let a git-derived field
carry the attribution.* Assert the subject **in-tree** — e.g. a pair of counters proving the
tree contains the fix and does not contain the defect — so the run proves it measured what
you think it measured regardless of which commit it believes it is on.

*Found by lively-koi-546, whose `PROBEHEAD` column was main's sha on one run.*

## 3. `--depth=1` makes ancestry checks uninformative, not false

`git merge-base --is-ancestor` fails closed on a shallow clone, so a remote probe reporting
`ANCESTOR=absent` is describing the CLONE, not the history. Fails closed, so not dangerous —
but reporting it as a fact rather than an artifact is silence-as-zero.

## 4. `fn` is not the only declaration keyword — `func` is too

269 `func` against 36654 `fn`. A census pattern of `^fn ` undercounted a population by a
factor of 2.7, and **two sessions produced the same wrong number independently** from the
same defective pattern.

A neighbouring failure the same hour: a pattern with no END anchor counted
`witness_verdict_diagnostic_companion` (the derivation itself) and
`lens_verdict_diagnostic_locus_module` (substring only) as producers of a channel that has
none.

**Four defective greps in one night between two sessions.** The only one that reached nobody
was caught because the finding happened to be interesting enough to double-check before
sending — that is not a discipline anything can rely on. Match both keywords and anchor both
ends.

## 5. A dashboard-only review is invisible to `gh`

`dashboard-ops reviews <pr>` rows can carry `dashboard_only: true`. Those do **not** appear on
the GitHub PR — `gh pr view --json reviews` cannot see them at all.

So a lane checking readiness through `gh` sees `appr=1 rc=0` while the dashboard says
`rc=1`. This is why the working agreements name `dashboard-ops reviews` as the source of
truth, and it is not a preference.

**And the join still has to be done by hand:** `merge_criteria` has itself published
`1/1 approvals` under a `head_sha` that the only approving row did not sit on. Join
`reviews[].sha` against `merge_criteria.head_sha` yourself, every time.

**The instrument can also be BEHIND a push** — a join that passes right now may be joining an
approval against the previous commit. Confirm `head_sha` is the commit you mean.

## 6. Archival is hard-blocked on open PRs

`dashboard-ops archive <session>` returns `HTTP 409 archive-refused (reason: open-pr)`.

So closing lanes does **not** free child slots while their PRs are open. With manual
operator merges, subtree throughput is bounded by **merge rate**, not by lanes. "Close lanes
to free capacity" frees attention, which is real, but is not what the queue is short of.

## 7a. The side-chat read returns ONLY the other side — your own messages are never in it

`dashboard-ops side-chat` returns a `turns` array containing the assistant/operator turns.
**Your own posts do not appear in it, ever.**

So "my message is not in the thread" is *not* evidence that the send failed. It is what the
view always shows. This was measured the hard way: a post was reported as failed on exactly
that reasoning, twice, to a peer who then recorded the wrong lesson — while the operator's
next turn quoted the post's specific content back, including a commit sha only that message
had supplied.

**And a send can complete asynchronously long after its foreground call gives up.** A send
that produced no output, and appeared absent from a subsequent read, had in fact landed; the
token counter decremented later, which was the only true signal available and was initially
dismissed as stale.

**Confirmed structurally, not inferentially.** The first account of this reasoned from the
operator quoting back a commit sha only one message had supplied — sound, but it establishes
that *one* send landed. A second session then checked the endpoint itself: across three
separate captures of its own thread (3, 4 and 6 turns), **zero of its four posted messages
appeared in any `turns` array**, including one carrying `sent:true` with a counter decrement.
`role` is `None` on every entry, so there is no field to filter on. The array simply does not
contain your side.

Correct readings, in order of reliability: the **token counter decrementing**; the other
side **responding to your content**; the returned turns — which cannot answer the question at
all.

**The third is not weak, it is INCAPABLE, and the distinction decides behaviour.** A weak
signal invites more sampling; an incapable one must be abandoned. Six retries is what you do
to a weak signal — and six retries is exactly what was spent here before anyone asked whether
the instrument could answer at all. The action that survives every reading is the same: **never re-send on a missing
receipt**, and under the asynchronous-landing reading that rule gets *stronger*, because a
missing receipt now carries no information whatsoever about whether the send will land.

## 7. The side chat's read path needs a long BACKGROUND budget

Foreground calls (100s, 110s, 400s) return `HTTP 502: conversation fetch failed` or hang with
zero bytes and no stderr. One success in this fleet came from a **900s backgrounded** call.

**Two different sessions produced two different symptoms** for the same surface — a 502 in one,
zero-bytes-no-stderr in the other — so neither should generalise from its own.

Side chats are **per session**. One session cannot read another's, so relaying is not
available no matter whose instrument is healthy.

**The state to hold explicitly:** *never successfully read in this session* means you have no
baseline, so you cannot distinguish a channel that broke from one you never reached. Until a
read succeeds, the absence of a reply is not information — treating it as one is the
empty-observation narrow with your own observation as the victim.

## 8. A caveat can travel without changing anyone's behaviour

A measured figure (12278 declared `test fn`) was published together with an explicit caveat
that it was **not** the right denominator, naming the correct one (~10439 routed). The caveat
was received, agreed with in writing, and the caveated figure was then quoted — by both
readers, repeatedly, including into the one channel with a confirmed receipt, with the caveat
dropped in transit. The number was corrected by a third party who had to notice the error from
scratch.

**This is not a grep failure and no scan discipline prevents it.** The correction existed, was
correct, was acknowledged, and did nothing. What propagated was the figure; what did not
propagate was the *not-to-use-this* attached to it.

The mitigation that would have worked is structural rather than attentive: **do not publish a
number you have already established is the wrong one.** If the right denominator is not yet
measured, publish the gap, not the figure with a warning stapled to it. A caveat is a request
that every future reader do work; the readers here were two sessions that had just spent a
night refusing exactly that kind of request in code.

## 9. A timeout kills the payload and the shell reports success

`timeout N ctrl-build -- <payload>; echo done` exits **0** when the timeout fires. The shell
answers *did the last command in my pipeline succeed*, and you asked *did the payload
complete*. Those differ exactly when a timeout kills the thing you care about.

**The remedy is a planted completion marker, and it is better than vigilance** — the payload's
last statement echoes a sentinel, and its ABSENCE is what you read. That is positive evidence
you cannot fail to notice, rather than an omission you have to spot.

> **THIS CLASS HAS A CANONICAL HOME AND THIS ENTRY DEFERS TO IT: gunbc#9066**, authored by
> silent-gull-867, who hit it independently — a dispatch returning exit 0 with the payload
> never executed, the streamed log ending after `git apply`. Their write-up carries the
> distinguishing table (`EMITTED=0` means it ran and produced nothing; *no* `EMITTED=` line
> means it never ran, so the dispatch is **void, not negative**), places the class beside its
> neighbours (`cargo` exiting 0 without compiling, a pipe to `tail` masking a status, `grep -c`
> returning 0 for a missing file), and names how it differs from all of them: those are
> *corrupted* or *absent* statuses, this is an **honest** status answering a narrower question
> than the reader asks of it.
>
> That last formulation is the same one this document arrives at in "The shape they share",
> reached independently the same night from a different specimen. If #9066 lands, this entry
> should shrink to a pointer rather than restate it — two documents for one class is the §3
> violation these notes exist to catch.

## 10. ctrl-build cleans `target/` on every dispatch — so splitting an overrunning job is worse

Every remote dispatch runs `git clean -x -d --force`, which removes `target/`. So a cold
release build is paid **per dispatch**, not once.

This inverts the obvious remedy. A job that overruns the wall looks like it should be split
into N smaller dispatches; that costs **N cold builds** instead of amortising one, and usually
overruns worse. Either fit it in a single long dispatch or choose a different shape — do not
shard it.

Measured on a 70-module standalone-compile sweep: a cold build plus seventy compiles does not
fit a 55-minute wall, and no split of it would have.

---

## The shape they share

Not one of these is an instrument lying. Each answers a real question correctly, and the
question is narrower than the one the reader asked:

| you asked | it answered |
|---|---|
| does the defect still reproduce | does the base still fail |
| what commit did this measure | what commit does the runner believe it is on |
| is this an ancestor | can this clone see the history |
| how many companions exist | how many `fn`-declared companions exist |
| is this PR approved | is it approved *on GitHub* |
| is a slot free | is the lane closed |
| did the operator reply | did the fetch succeed |
| did my message send | did the last command in my pipeline exit 0 |

The transferable move is not vigilance. It is to **make the run assert its own subject**, so
that a narrower answer cannot be read as the wider one.
