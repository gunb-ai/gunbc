# An instrument answering a question narrower than you asked (2026-08-23)

**Found by:** `silent-gull-867`, during the Set/Map carrier decomposition; clause 5 and clause 1's
precondition contributed by `quiet-pike-368`; clause 6 by `eager-lark-892` and `deep-ant-102`;
generalised to the fleet at the request of `smart-ram-730`.

## The rule, and why the techniques below are only its instances

**Every failure in this document is an instrument answering a question narrower than the reader was
asking.** Not one of them involves anything lying, erroring, or malfunctioning:

| the artifact answers | you asked | clause |
|---|---|---|
| did the *dispatch* succeed | did the *payload* run | 1 |
| can I see this in my *history* | is this in my *tree* | 2 |
| do these two *differ* | does this *work* | 3 |
| are these *equal* | (while holding one thing) | 4 |
| what is true of this *tree* | what did *I* do | 5 |
| were these two *built as named* | were the two things I compared *actually different* | 6 |
| (nothing — the artifact is well-formed) | *who is even able to check this?* | 1's precondition |
| this narrower thing is *green* | is the thing I care about green | the third leg |
| this check *failed* | did it fail, or was it *cancelled* | the third leg, in the tooling |
| I *applied* the treatment | did the treatment *reach* what I measured | the three below |
| these runs *differ* | does the *content* differ | the three below |
| this population is *N* | N *of what denominator* | the three below |

That is why **none of them has a failure arm**, and it is the whole difficulty: there is nothing to
catch, no error to check for, no status to inspect that would have been different. The absence of
the answer you wanted reads as the answer you wanted. So the guard is never "check for an error" —
it is always **assert the missing question**, explicitly, in a form the run itself has to produce.

The clauses below are the instances that have actually cost this repository time. They will go
obsolete as the tooling changes; the paragraph above will not. **If you are reading this to decide
what to do about a measurement you do not yet distrust, that paragraph is the part to apply.**

## A note on this document's own discoverability

`curated_cargo_probe_one.sh`'s header had already documented one of these traps — the 176-vs-177
emitted-file discrepancy, including that two sessions differenced the pair as a delta for forty
minutes at a prior ref. It was found, written down, and **sprung again the same night, on the people
running that very file**. A rule recorded where the reader will not be standing gets re-derived at
full price.

The structural version of the lesson, worth more than this document: `EMIT_COUNT_SRC`-style
provenance belongs **in the emitted line**, not in a header comment *about* the emitted line. A
number that states its own producer cannot be differenced against a number produced differently; a
comment explaining that they are different can be, and was.

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

### The marker must not match the request for the marker

Found by `smart-ram-730`, **inside this document's own remedy**, which
is the reason it is stated here rather than in a footnote.

`ctrl-build --remote` echoes the command it is about to run. So a transcript containing
`MARKER_ALL_DONE` contains it **twice**: once in the echoed command text, before anything executes,
and once if the payload actually reaches the end. A `grep -q MARKER_ALL_DONE` therefore succeeds
**because you asked for the marker**, not because anything produced it — and it succeeds identically
on a dispatch that ran nothing at all. The marker rule, defeated by the marker.

```bash
grep -q  MARKER_ALL_DONE   # matches the echoed command. Always true.
grep -qx MARKER_ALL_DONE   # whole-line match; the echo is one long line, so only real output matches
```

Anchoring works for the same reason: `grep -E "^MARKER_"` skips the echo, which carries the whole
script on a single line with the newlines escaped. **Every `EMITTED=`-style table in this document has
the same exposure** — read those markers anchored or whole-line, never as a bare substring.

This is the class one turn further in than the rest of the document: not an instrument answering a
narrower question, but *the check matching its own request*. The general form is that a transcript
containing both the instruction and the result cannot be searched for the result without excluding
the instruction.

### An exit code answers whether the LAST command succeeded

Two receipts, one night, and the pair is the point.

A 70-module sweep was given a wall-clock budget that could not fit a cold build plus the compiles.
`timeout` killed `ctrl-build`, a trailing `echo` ran, and **the shell returned exit 0**. The only
signal that anything had died was that the output carried no data rows — failure inferred from
missing content, by luck rather than by design, because no completion marker had been planted.

The same hour, on the same tooling: a dispatch printed its first marker (`SUBJ_PF_ROW=1`), then the
build line, then nothing. No `CI_EXIT=` marker — because the payload exceeded the dispatch wall. Read
as a *missing* marker, that is unambiguous: the payload did not finish. Read as an exit code, it was
a success.

> The shell tells you whether the last command in your pipeline succeeded. You asked whether the
> **payload** completed. Those differ exactly when a timeout kills the thing you care about.

Positive evidence beats an absence you have to notice. Plant a final marker, and read for it
anchored.

## Clause 1's precondition: only the party who knows the required answer can author the check

Contributed by `quiet-pike-368`, converged on with `smart-ram-730` from two failures in one night.
It sits **here**, before clause 2, rather than at the end, because it is not a sixth technique — it
is what makes clause 1 performable at all.

Clause 1 says a *missing* marker must be spelled differently from a *zero* marker. This says **who
is capable of making that distinction**:

> The check must be authored by whoever knows what the answer has to be. It cannot be performed by a
> reader of the output.

Without it, clause 1 reads as advice a careful downstream reader could follow. They cannot — they do
not hold the fact that makes the check possible.

**Two instances, and neither was caught by care.**

1. A candidate-union dump inside the emitter returned **empty for all five modules**. Empty-everywhere
   is spelled identically to the real answer being hunted (*absent from the union — nothing proposes
   these names*), and it was one step from being reported as that. It was caught only because one of
   the five was a **positive control**: a module where a specific name *must* appear, because the
   author's own change had just closed the error it causes. The cause was mundane — emitted files
   live under `<out>/src/`, the grep path did not exist, and a `2>/dev/null` ate the
   `No such file` that would have said so in one line.

2. A dashboard send **dropped the subject of a sentence**, twice in one night. It arrived as a
   well-formed sentence beginning mid-clause, and the reader reconstructed the missing phrase
   correctly — but only because that paragraph independently named the function, the data structure
   and the contrasting cases, so the phrase was over-determined by its neighbours. Remove that
   redundancy and reconstruction is not recovery: **the reader supplies what they already believed**,
   and it arrives back in the sender's voice as confirmation. In the live case that would have been a
   hypothesis the measurement had refuted an hour earlier. That is strictly worse than visibly
   garbled text, and it is invisible from both ends.

**What unifies them, and why it is one clause rather than two anecdotes: both artifacts were
well-formed.** A complete sentence; a complete empty result. Nothing errored, nothing was malformed,
no failure arm existed anywhere. Downstream holds only the artifact and the artifact is fine, so no
amount of downstream care reaches either. The detecting move is identical in both — *assert what must
be present, then check* — and in both cases the only party who can make that assertion is the one who
knows the intended content.

**The concrete form, because the abstract version gets nodded at:**

- spell a MISSING marker differently from an EMPTY one (`<no probe line>`, not an empty name set)
- print a **liveness count** beside every zero (`171 files carry probe lines`)
- never suppress stderr on an instrument — the `2>/dev/null` is the part that will actually stop the
  next author

Receipt from this very document's lane: a candidate-tree comparison printed `DIFFERS: <file>` per
mismatch and **nothing at all** when the file list was empty, so "the regen candidate matches the
committed mirror" and "`find` matched no files" rendered identically. The regen verdict said
`first_generation_equal=false` in the same output, which is the only reason the empty list was not
read as agreement.

## The third leg: assert what the instrument's GREEN actually entails

Clause 1 says assert **that** you measured. Clause 2 says assert **what** you measured. This is the
third leg, and it is the one that fires when both of those pass:

> A green that answers a narrower question is indistinguishable from the green you wanted.

Nothing here is a marker problem. The instrument ran, on the right tree, and returned an honest
success. The reader simply took it as covering a question it never asked.

**Three receipts, from three lanes, in one night:**

1. A `.dag` compile board run three times, green three times, read as evidence a carrier
   decomposition was complete. The board compiles **source**; it says nothing about whether the
   emitted **seed** still matches. `claim_executor --required-regen` asked the other question and
   answered `first_generation_equal=false` with ten drifted mirrors, immediately. Two instruments,
   two questions, and only one was run.
2. Whole-tree compile-clean read as evidence a module can be emitted **alone**. It cannot: the
   definers are in the pool only because some other module imported them, so the tree-wide green
   entails nothing about the single-module case.
3. `gh pr checks` rendering `fail` for a run whose job conclusion is `cancelled` — see the next
   section, where the same shape reaches the merge-readiness signal itself.

The repair is not a better marker. It is to name, before reading a green, **the exact proposition it
establishes**, and then check whether that is the proposition you need. If the two differ, the green
is not weak evidence for your question; it is no evidence at all.

## Where this reaches the tooling: CANCELLED renders as `fail`

Worth stating on its own because it manufactures reds for anyone stacking branches, and a
manufactured red gets chased.

`witnesses.yml` keys concurrency on the **resolved PR number**
(`witness-floor-${{ github.event.pull_request.number || github.run_id }}`, `cancel-in-progress` on
`pull_request`). GitHub attributes a run to a **branch**. So when branch B is based on branch A and
both have open PRs, a run created for A's PR can carry `head_branch: B` — and B's pushes cancel it.
A's checks then read **fail** indefinitely while nothing is wrong with A's diff.

The discriminators, because at a glance a cancelled run and a failed one are the same word:

| tell | cancelled-by-concurrency | genuine failure |
|---|---|---|
| job `conclusion` | `cancelled` | `failure` |
| `steps` | `[]` — never picked up a runner | populated |
| `runner_id` | `0` | real |
| lifetime | ~2 min from `started_at` | full duration |
| job `head_branch` | the CHILD's branch, not the PR's | the PR's own |

```bash
gh api repos/<owner>/<repo>/actions/runs/<id>/jobs \
  --jq '.jobs[]|"\(.name) \(.conclusion) steps=\(.steps|length) branch=\(.head_branch)"'
```

Recovery is to stop pushing the child, then rerun the parent's run. Reruns issued *while* the child
is still pushing get cancelled again — two attempts were burned confirming that.

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

## Fifth clause: is this defect MINE? — the reverse-patch control

*Contributed by `quiet-pike-368`, who was one night from rewriting a correct change to fix a defect
they did not cause.*

The clauses above all guard the run. This one guards the **attribution**. When your change is in the
compiler and the measurement is downstream of it, a red tells you nothing on its own: the tree you
measured contains your change **and** everything else that landed since your baseline. The failure
is not a wrong number — it is a **correct number attributed to the wrong cause**, with the author's
own plausible mechanism supplying the false explanation.

The control is both arms **inside one dispatch at one ref**, where the *before* arm is produced by
reverse-patching your own commit out of the worktree:

```bash
# locally: carry the patch in the script text
git show <your-commit> -- <files> | gzip -9 | base64 -w0

# in the dispatch, AFTER arm first, then:
echo "$REV_B64" | base64 -d | gunzip > /tmp/rev.patch
git apply -R /tmp/rev.patch && echo REVERT_APPLIED_OK || echo REVERT_FAILED
git status --porcelain          # PRINT THIS
# rebuild, then BEFORE arm
```

**Reverse-patch rather than `git checkout`**, because ctrl-build's remote does its own
`git checkout --force` before your script runs and the clone is grafted at your commit with a
depth-1 fetch — so a checkout-between-arms is either defeated or cannot reach the parent commit at
all. A patch carried in the script text survives whatever the runner did to the worktree.

Four things make it an instrument rather than a gesture, each learned by being burned:

1. **Print `git status` after the revert.** A silently-failed revert gives you two *after* arms
   reporting identical numbers, which reads exactly like "my change had no effect" — a false null
   with no failure arm. The `REVERT_APPLIED_OK` / `REVERT_FAILED` echo plus the porcelain listing is
   what makes the before arm provably *before*.
2. **Rebuild the binary between arms, and delete any binary-identity stamp.** The curated probe keys
   `gunbc` on `git rev-parse HEAD`; both arms are at the *same* HEAD by construction, so the stamp
   says "already built" and arm two silently reuses arm one's compiler — a false identical. This is
   the stale-binary class the probe's own header documents, arriving from a direction that header
   does not cover: the arms differ by a **worktree patch**, not by a commit.
3. **Revert the full set the binary is built FROM, not the set you edited by hand.** In a
   self-hosting tree the generated mirror is what the compiler is actually built from, so it must be
   in the reverse patch too. Receipt — one commit, two files, both required:

   ```
   src/v1/05_emit_rust.dag                      the authority
   src/v1/stage0/src/v1_compiler_emit_rust.rs   the mirror the binary compiles
   ```

   Reverse-patching only the `.dag` gives you a *before* arm whose `gunbc` was built from the
   **after** mirror: both arms then measure the change, the histogram rows come back identical, and
   it reads as "my change had no effect". That is the same false null as condition 1, arriving
   through a door `git status` does not close — the revert genuinely applied, the porcelain listing
   is honest, and the arm is still not a before arm. `git show <commit> --stat` is the roster; if a
   generated artifact is in it, it belongs in the reverse patch. The `.dag` is what the author
   thinks of as "my change"; the mirror is what the compiler thinks of as its source.
4. **Read the rows that did NOT move.** A delta on your target row is equally consistent with
   "fixed 8" and with "fixed 12, broke 4 elsewhere". Only the unchanged remainder discriminates.
   Receipt: `E0425` 24 → 16 while all seventeen other histogram rows stayed byte-identical.

**What it bought.** A board came back `EMIT_REFUSE`, 0 files, no cargo log. The obvious reading was
that the change had broken emit, and a plausible mechanism was ready to blame. The control returned
**identical refusals in both arms** — so it was not the author's. It was a trailing `//` block in a
file not even in the compiled closure, since fixed on main as #9027.

Note the failure this clause names passes clauses 1–4 completely: the run
executed, on the right tree, the subject stood alone, and the arms were genuinely two arms. What is
missing is any evidence about **whose** the difference is.

## The clauses

| | assert | catches | missed by the others |
|---|---|---|---|
| 1 | **that** you measured | payload never ran | 2 and 3 see markers and a real tree |
| 2 | **what** you measured | ran against the wrong tree | 1 sees every marker present |
| 3 | the subject **stands alone** | subject was never viable | 1 and 2 both pass — #8282 passed both for two days |
| 4 | the two arms are **two dispatches** | a comparison that lost an operand | 1–3 all pass per arm; the arms are simply the same arm |
| 5 | the difference is **yours** | a correct number blamed on the wrong change | 1–4 all pass; nothing about the run is wrong |
| 1p | **who can author the check** | a well-formed empty result read as a finding | every clause above assumes a checker who holds the required answer |
| 6 | the arms **actually differed** | an agreement that compared one thing to itself | 1–5 can all pass, including a *correct* provenance stamp |

Each catches what the others cannot, and none implies another.

**They are not five independent lessons.** The first four are faces of one thing: *an instrument
reporting an answer to a question it did not ask.* Clause 1's wrapper answers "did the dispatch
succeed" when asked "did the payload run". Clause 2's ancestry check answers "can I see this in my
history" when asked "is this in my tree". Clause 3's comparison answers "do these differ" when asked
"does this work". Clause 4's single dispatch answers "are these equal" when it holds only one thing.
In every case nothing lied and nothing errored — the artifact was simply answering something
narrower than the reader was asking, which is why none of them has a failure arm and why each has to
be guarded by asserting the missing question rather than by checking for an error.

Clause 5 is the same shape pointed at the *reader* rather than the instrument: the run answers "what
is true of this tree" when asked "what did I do", and the gap is filled by the author's own guess.
That is why its guard is a second arm rather than a better marker — no property of a single
measurement can say whose the difference is.

## Clause 6 — an agreeing pair is not a result until it carries its own proof of difference

Contributed by `eager-lark-892`, who ran it; the reporting-side statement is `deep-ant-102`'s,
relayed by `smart-ram-730`. It is not a sixth trick. **It is clause 4's own asymmetry moved one
layer down**, and filing it as a new technique is how it gets dropped the first time it is
inconvenient.

The asymmetry clause 4 rests on:

```
arms DIFFER -> the arms cannot have shared a binary -> self-proving, assert nothing
arms AGREE  -> the change was inert, OR both arms ran one thing, and the output does not say which
```

A **null is undecidable from its own output**, and the filter is vicious: a differing pair
advertises its own soundness, so the cases you would catch are exactly the ones that did not need
catching. A null is also the reading you are least inclined to interrogate, because it arrives as a
finding to explain rather than a fault to doubt.

**Binary provenance does not close it.** A provenance key answers *was this binary built from the
tree I named*. It never answers *were the two things I compared actually different*. Three things
defeat the first: a shared binary, a shared tree, and a key that is correct and blind — when both
arms run on one tree (the failed checkout inside a single dispatch, clause 4's live instance) the
key **rightly** returns the same value and cannot see the problem. That is `measure() == measure()`
with a stamp on it. As of `330f63c514d` the probe key includes `HEAD` plus a sha256 of
`git diff HEAD`, which closes the shared-binary half; it cannot close the shared-tree half, and
"the key is fixed" must not travel as "agreeing arms are safe".

**The control: diff the emitted artifact, not the board.** Compile the subject at each arm, emit to
two directories, diff the bytes.

```
boards AGREE + emitted bytes DIFFER    -> the compilers provably behaved differently. Real null.
boards AGREE + emitted bytes IDENTICAL -> genuine equivalence, OR one binary. Still undecidable;
                                          now go assert provenance per arm.
```

It works for exactly clause 4's reason, relocated: shared binaries cannot produce different bytes,
so an agreeing board over differing emitted bytes is self-proving by the same argument.

**Both arms of the control were observed**, which is what makes it a control rather than a
decoration — a check whose RED has never been produced is permanently green by construction and is
worse than absent, because it gets cited as coverage:

| pair | emitted bytes | boards | reading |
|---|---|---|---|
| `c07d13a49f` vs `974ac5d808` | 2 of 176 files differ | identical | real null: different compilers, unmoved board |
| `c07d13a49f` vs `98b18cdc81e` | 0 of 176 differ | identical | genuine equivalence for that entry's closure |

Two bounds, so it is not oversold: it proves the compilers **differ**, not that either is correct
(the 2-file diff sat under a repair that turned out not to move its target rows at all); and it is
scoped to **one entry's closure**, not to the two refs generally.

**Where the obligation sits.** Stated as a question the reader asks when a report arrives, this is
verification living in the reading frame — it works only while someone reads every report, remembers
the class, and recognises it in another author's phrasing, which is the structure that failed four
separate times in one night. So it belongs on the **producing** side:

> An agreeing arm-pair is not a result until it carries its own binary-provenance assertion, and
> preferably its own emitted-bytes diff. A differing pair is self-proving and needs nothing.

The difference between a guard and a habit is that an agreeing pair arriving *without* the assertion
is then visibly incomplete rather than quietly plausible.

**A third guard worth naming beside two-dispatch and the stamp**, and it is a design decision rather
than a footnote: run the arms as **sequential local dispatches from a fixed detached checkout**.
`ctrl-build --remote` checks out the pushed base and applies your diff as a patch, so a two-arm
dispatch from one branch has **one head and two trees** — the after-arm can be handed the
before-arm's binary. A sequential local run from a detached checkout cannot express that state.

Receipts: `gunbc#9019`'s `RESULT_FALSIFIED.md` and its controls directory.

## Three more, all from `quiet-pike-368`, all caught by an assertion rather than by care

Grouped because the pattern across them is one sentence: **a well-formed wrong number is the default
output of an under-specified instrument**, and in all three the only defence that worked was carrying
something that could contradict it.

### A treatment that cannot reach the instrument

Distinct from everything above: not a missing marker, not a narrower question. **Both arms are
honest and the comparison is meaningless.**

A two-arm test patched `src/v1/05_emit_rust.dag` and then measured `gunbc compile` output. Both arms
agreed on every figure — `EMIT_RC=0`, `FILES=176`, `LIVENESS_i64=848`, `BARE_INT_POSITIONS=47`, both
target sites byte-identical. The patch was inert because it **could never have applied**: `gunbc
compile` runs the *seed binary* built from `src/v1/stage0/src/*.rs`, and a `.dag` edit reaches an
artifact only through regen, which interprets the `.dag` to produce a candidate stage0.

What caught it was binary provenance: `md5sum target/release/gunbc` **identical** across arms, and
the after-build reporting `Finished … in 0.05s` following an `rm -f`. Without that, the patch would
have been reported as measured-inert.

> **Assert that the treatment reached the thing you measured — not merely that you applied it.**

### A digest that includes a path is a digest of the run, not of the content

A determinism study printed a per-run tree hash as
`find $DIR -name '*.rs' | sort | xargs md5sum | md5sum`, and all ten runs came back distinct. **That
counter is worthless**: `md5sum` echoes each *path* beside its digest, and `$DIR` was per-run
(`/tmp/n$i`), so the hash differs by construction on every run — *including under a perfectly
deterministic producer*. It fabricates exactly the positive result the study was looking for.

It was exposed by an internal contradiction between two instruments answering one question: a pair
showing **0 differing files** under `diff -rq` while its two "tree hashes" differed.

> **For cross-run content comparison use a path-independent instrument, and always carry a second
> instrument that CAN contradict the first.**

### When two counts of "the same" population differ, check the denominator before the test

Two sessions counted hand-maintained files in the v1 seed: 39 and 79. The natural suspect was the
*test* — whole-file `Source module:` versus first-five-lines — exactly the kind of difference that
yields two honest numbers. Measured across all 169 top-level files: **zero disagreement**, so the
test was not the separator.

The separator was the **denominator**. `src/v1/stage0/src` holds 209 `.rs` (169 top-level + 31
`bin/` + 6 `cli_run/` + 3 `module_path_index/`), and one glob — `src/v1/stage0/src/*.rs` — is
non-recursive. 31 + 39 + 9 = 79. The smaller number was right *and* partial.

Same shape as the digest above: `*.rs` silently means "top level only", answers a narrower question
than the reader assumes, and returns a well-formed number either way. **This is the more common
half**, because a test difference is visible in the code and a denominator difference is invisible in
both.

It pairs with a clause owned by `deep-ant-102`: before weighing which of two results dominates,
**intersect the populations** — an empty intersection means there is no dominance question to answer.
One failure from two sides: two measurements with no shared elements, and two measurements over
unequal denominators.

## Neighbours already recorded, and how this differs

- `cargo` exiting 0 without compiling is the same rule for a different harness.
- Piping to `tail` masking an exit code is a *corrupted* status; this is an *honest* status
  answering a narrower question than the reader asks of it.
- A grep returning 0 hits because the file does not exist is the same conflation at file grain.

The common repair in every case is identical: make the instrument state something only a real run
could state, and read that instead of a status.
