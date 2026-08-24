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

The transferable move is not vigilance. It is to **make the run assert its own subject**, so
that a narrower answer cannot be read as the wider one.
