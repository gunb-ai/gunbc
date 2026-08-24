# Read a marker you authored, not the harness's status (2026-08-23)

**Class:** an instrument that did not run, reporting as a measurement that came back.
**Found by:** `silent-gull-867`, during the Set/Map carrier decomposition. Generalised to the fleet
at the request of `smart-ram-730` (emission convergence).

## The observation

A `ctrl-build --remote` dispatch returned **exit code 0 with the payload never executed**. The
streamed log ended after `git apply` finished listing its files; not one line of the command's own
output appeared. The wrapper reported success.

Reading that exit code would have produced a report of a green run from a dispatch that ran nothing.

## Why it is invisible

The wrapper's exit code describes the **dispatch**, not the payload. Between

- "the binary ran and printed nothing", and
- "the binary never ran",

there is no distinguishing signal in the status. They are byte-identical through it, and neither
errors. There is no failure arm anywhere: the absence of output *is* the only evidence, and absence
reads as a value.

This is the same shape as the empty-observation class DESIGN's recurring-failure list already
names — ⊥-as-answer conflated with ⊥-as-ignorance — moved one layer out, from the subject to the
tooling that measures the subject. The subject-level instance was live in the same session: an
emission probe returned `emitted files: 0` because the compiler had refused and written no output
directory at all, which is a fact about the instrument, not about the emission.

## The rule

**Author your own markers and read those. Never the harness's status.**

```bash
ctrl-build --remote -- bash -lc '
  set -x
  ./target/release/thing --flags > /tmp/a.log 2>&1; echo "RUN_EXIT=$?"
  echo "EMITTED=$(find /tmp/out -name "*.rs" 2>/dev/null | wc -l)"
  echo "HARD=$(grep -oE "produced [0-9]+ hard" /tmp/a.log | head -1)"
'
```

Then grep the transcript for `RUN_EXIT=`, `EMITTED=`, `HARD=`. The property that matters is that a
**missing** marker is distinguishable from a **zero** marker:

| you see | it means |
|---|---|
| `EMITTED=0` | the command ran and produced nothing |
| no `EMITTED=` line at all | the command never ran — the dispatch is void, not negative |

Without the marker those two collapse into one another, and the second silently becomes the first.

`set -x` is part of the rule, not decoration: it makes the transcript show which statements were
reached, so a truncated run is legible as truncated.

## Second clause: assert the SUBJECT, not only that you measured

The rule above guards *whether the command ran*. It does not guard *what it ran against*, and that
gap has its own live specimen from the same evening (`smart-ram-730`, confirming a fix on a merged
head): a `git merge --ff-only` had failed, so the dispatched branch did **not** contain the commit
under test. The head was unchanged, the payload executed normally, every execution marker was
present — and the run would have returned zero emitted files and read as **"the fix did not work"**.

The first clause cannot catch that one. The markers are there; the command did run. What is missing
is any assertion about the *subject*:

```bash
echo "HEAD_SHA=$(git rev-parse HEAD)"
echo "HAS_FIX=$(git merge-base --is-ancestor <fix-sha> HEAD && echo yes || echo no)"
```

Read those first. A dispatch that cannot state which tree it measured has not measured a tree — it
has measured *a* tree, and which one is exactly the fact in dispute when the result is surprising.

**The subject marker must be answerable where it runs.** An ancestry check is the obvious spelling
and it is the wrong one on these runners: they fetch with `--depth=1`, so there is no history for
`git merge-base --is-ancestor` to walk, and it reports **not an ancestor** for a commit that is
present. Measured here — a branch that demonstrably contained the fix (emission worked, 176 files)
reported `HAS_9027=0`. A subject marker that answers "no" because it *cannot* answer is the same
defect one level in, so prefer a **content** assertion over a history one:

```bash
echo "SUBJECT_HAS_TYPE=$(grep -c '^type FinitePowerSet' dag/std/algebra.dag)"
echo "SUBJECT_ALIAS=$(grep -oE '^type Set<element> = [A-Za-z]+' dag/std/types.dag)"
```

Those read the working tree the compiler will read, need no history, and say what the tree *is*
rather than where it came from.

The two clauses are complementary and neither implies the other:

| failure | execution markers | subject markers |
|---|---|---|
| dispatch never ran the payload | **absent** — caught | absent |
| dispatch ran against the wrong tree | present — *not* caught | **wrong** — caught |

A surprising result should send you to the subject markers before the conclusion. "The fix does not
work" and "the fix is not in this tree" produce identical output, and only one of them is about the
fix.

## Third clause: assert the subject STANDS ALONE

Clauses 1 and 2 guard that the command ran and what it ran against. Neither asks whether the thing
measured is **viable on its own** — and a comparative measurement can be perfectly valid, correctly
executed, on exactly the intended tree, and still never pose that question.

**Receipt (gunbc#8282, the namespace cut, abandoned 2026-08-24).** Every measurement taken on that
branch was comparative — conflict counts, import deltas, rehearsal tables — and **not one asked
whether the branch built on its own**. It had not built since 2026-08-22: breaking commit
`c98516772e7` dropped 36 emitted modules whose `.dag` authorities still exist, and 117 commits
landed on top of it over two days. Nobody was careless. Everybody was measuring, and every
measurement was relative to something else carrying the same defect.

The check is one dispatch: **build the head alone in a fresh worktree — no merge, no working-tree
patch, `git clean -x -d --force`.**

It has a positive arm, which is why this is a routine check rather than a cautionary tale. Run on a
different branch the same night, a fresh fetch + clean + head-alone build confirmed the current
mirrors built the post-relocation corpus and reached the executor — clearing that branch in a single
dispatch. The value is that **one dispatch distinguishes two otherwise identical-looking
situations**: *"my change is incompatible with main"* and *"my change cannot exist without itself"*
produce the same confusing regen failure.

**The signature to recognise, because it is what let #8282 run for two days: hand-restoration does
not converge, and looks like progress.** Measured there — head as-is 123 errors; plus 4 mirror files
122; plus all 36 dropped modules 271. Three rounds, monotonically worse, because each restored
module references further things the same commit dropped. **If your repair loop is going the wrong
way while you add more of the thing that seems missing, that is a broken seed, not a list of missing
files.**

## Fourth clause: two arms, two dispatches

Running a before/after comparison **in one dispatch, with a checkout between the arms**, is the
natural and efficient thing to do, and it silently destroys the comparison.

Measured here, while deliberately trying to avoid exactly this class: the mid-run
`git checkout <other-sha>` failed, its failure was swallowed, and the "before" arm therefore ran on
the **same tree** as the "after" arm. Both reported identical shas. Identical shas were the result
being sought, so the run looked like a clean pass — it was `measure() == measure()`, the tautological
control DESIGN's oracle rule already names, produced by a comparison that had quietly lost its
second operand.

**Run each arm as its own dispatch, on its own tree, each asserting its own subject.** A comparison
whose two arms share a process shares whatever went wrong in it, and the failure mode is silence: a
lost operand does not report as a lost operand, it reports as agreement.

## The three clauses

| | assert | catches | missed by the others |
|---|---|---|---|
| 1 | **that** you measured | payload never ran | 2 and 3 see markers and a real tree |
| 2 | **what** you measured | ran against the wrong tree | 1 sees every marker present |
| 3 | the subject **stands alone** | subject was never viable | 1 and 2 both pass — #8282 passed both for two days |
| 4 | the two arms are **two dispatches** | a comparison that lost an operand | 1–3 all pass per arm; the arms are simply the same arm |

Each catches what the others cannot, and none implies another.

**They are not four independent lessons.** They are four faces of one thing: *an instrument
reporting an answer to a question it did not ask.* Clause 1's wrapper answers "did the dispatch
succeed" when asked "did the payload run". Clause 2's ancestry check answers "can I see this in my
history" when asked "is this in my tree". Clause 3's comparison answers "do these differ" when asked
"does this work". Clause 4's single dispatch answers "are these equal" when it holds only one thing.
In every case nothing lied and nothing errored — the artifact was simply answering something
narrower than the reader was asking, which is why none of them has a failure arm and why each has to
be guarded by asserting the missing question rather than by checking for an error.

## Neighbours already recorded, and how this differs

- `cargo` exiting 0 without compiling is the same rule for a different harness.
- Piping to `tail` masking an exit code is a *corrupted* status; this is an *honest* status
  answering a narrower question than the reader asks of it.
- A grep returning 0 hits because the file does not exist is the same conflation at file grain.

The common repair in every case is identical: make the instrument state something only a real run
could state, and read that instead of a status.
