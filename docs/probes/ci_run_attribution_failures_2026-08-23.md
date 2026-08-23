# The run did not measure what you think: four attribution failures in one night (2026-08-23)

**Subject:** attributing a CI result to a change. **Deliverable:** one class, four measured
instances, and the check that closes each. **No repair is proposed here.**

Every instance below cost real time on 2026-08-23, in four different lanes, and three of the
four produced a *confident wrong attribution* rather than an ambiguous one. That is what makes
them one class rather than four mistakes: in each case the run was read as evidence about a
change, and the run's subject was not that change.

## The class

> **A run reports the ref it BUILT. It never reports what that ref was FOR, and it is rarely
> the ref you pushed.**

Between "a change" and "a verdict" sit four independent substitutions, each invisible in the
result. A green or red is equally plausible under all of them.

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
never observe a completed floor.

**Check:** read the run's `conclusion` explicitly. `cancelled` is not `failure` and is not
`success`; absence of a verdict is not a verdict.

## The shared shape

Instances 1–3 are all *the subject was substituted*; instance 4 is *there was no subject*. Both
halves are the same underlying error — **treating a run as a measurement of a change without
establishing that it measured that change** — and both are cheap to close:

| establish | command |
|---|---|
| which ref was built | `gh api …/runs/<id> -q .head_sha` |
| whether it is a merge ref | `gh api …/runs/<id> -q .event` |
| what that ref was for | `git log -1 --format='%s' <sha>` |
| whether it reached a verdict | `gh api …/runs/<id> -q .conclusion` |

Four commands. Tonight, each of the four was skipped exactly once, by four different lanes, and
each skip produced a confident conclusion that was wrong.

## What the corpus already says, and why it did not prevent this

The repository's own failure list names the **empty-observation narrow** — ⊥-as-ignorance
rendered as ⊥-as-answer — and instance 4 is precisely that. Instances 1–3 are its sibling in a
position the list does not currently name: not the observation collapsing, but *the observation
being about a different subject than the one claimed*. A number with no ref is not a number; the
generalisation these four instances force is that **a verdict with no established subject is not
a verdict**, and the subject of a CI run is not knowable from the run alone.
