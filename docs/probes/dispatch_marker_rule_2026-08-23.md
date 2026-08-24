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

The two clauses are complementary and neither implies the other:

| failure | execution markers | subject markers |
|---|---|---|
| dispatch never ran the payload | **absent** — caught | absent |
| dispatch ran against the wrong tree | present — *not* caught | **wrong** — caught |

A surprising result should send you to the subject markers before the conclusion. "The fix does not
work" and "the fix is not in this tree" produce identical output, and only one of them is about the
fix.

## Neighbours already recorded, and how this differs

- `cargo` exiting 0 without compiling is the same rule for a different harness.
- Piping to `tail` masking an exit code is a *corrupted* status; this is an *honest* status
  answering a narrower question than the reader asks of it.
- A grep returning 0 hits because the file does not exist is the same conflation at file grain.

The common repair in every case is identical: make the instrument state something only a real run
could state, and read that instead of a status.
