# Before trusting an empty sweep, break something the sweep should catch

**The rule, and it is the whole brief.** An empty result has two readings — *nothing is wrong* and
*the instrument cannot see* — and they are **byte-identical at the output**. Reading the result more
carefully does not separate them. One command does: plant the defect the sweep exists to find, and
see whether it goes red.

This is a companion to [instrument traps](instrument-traps-2026-08-24.md), whose observation that *a
false absence is worse than a false green* — because an absence terminates the investigation — this
answers with the one case that has a cheap mechanical remedy. The planted red is what stops the
investigation terminating.

**The receipt is that the rule caught its own author.** The first version of this brief was written
from a sweep that came back empty, generalised from a single planted red, and stated a conclusion
that was wrong. What corrected it was running more planted reds instead of shipping. That is a
stronger receipt than a rule nobody tested on themselves, and it is why the history is kept here
rather than tidied away.

## The specimen

A lane was dispatched to finish an *incomplete-header sweep* over `src/v2/compiler`, on the premise
that a partial import header is **already broken** — that a module carrying one refuses when compiled
alone, as `v2.compiler.program_assembly` did in gunbc#9083.

Compiled entry-scoped, every module in the universe reports zero blocking errors. Read forward, the
premise was simply already satisfied and there was nothing to do.

**The first planted red appeared to confirm that reading, and it was the unlucky arm.** Deleting one
used import from `v2.compiler.compile` changed nothing. Deleting *the entire header* also changed
nothing. Two greens in a row support a tidy conclusion — headers contribute nothing, the instrument
is blind — and that conclusion was written up.

**It was wrong, and the next planted reds said so.** Dropping imports one at a time, three of nine
refuse immediately, by name and location:

    drop v2.compiler.emit               function 'emit' not found in scope
    drop v2.compiler.eval               function 'eval_node' not found in scope
                                        function 'inputs_root_only' not found in scope
    drop v2.compiler.program_assembly   function 'assemble_program_from_ingest' not found in scope

The header is load-bearing. One arm had said otherwise, and one arm was generalised from.

## What the sweep was actually measuring

Pushed further, the arms resolve into a single predicate. Measured on one binary, one file, each
state applied and reverted in sequence:

| header | blocking |
|---|---|
| complete | none |
| **missing one entry** | **refuses, located and named** |
| **absent entirely** | none |
| **one entry naming a symbol the file never uses** | **refuses everywhere at once** |

The last row is the one that settles it. Adding a *single inert line* — an import of a module the
file references nowhere — turns a green compile into dozens of unresolved types and functions.
Deleting that same line, leaving no header at all, returns it to green.

**So the regime is decided by the presence of any import line, not by its contents.** A module that
declares no imports is resolved against the whole pool and gets everything. A module that declares
even one is resolved against its declared closure and gets exactly what it named — which is why a
complete header passes, an incomplete one refuses, and *no* header passes for an entirely different
reason than the first.

That is the shape the original brief named and the first write-up walked past. The empty sweep was
not blindness: **the tree is currently clean**, the last broken instance having been repaired by
gunbc#9083. The instrument sees the class fine.

**One boundary on that table, because the third row is where the risk now lives.** *Green* in the
whole-pool row means resolution **succeeded**, never that it was **correct**. Where a name has more
than one declarer in the pool, binding the wrong one is green at every aggregate grain — a blocking
count cannot distinguish *found the right name* from *found a name*. Establishing which provider was
bound needs a site whose **wrong** bind cannot typecheck: match on a sibling arm the other candidate
does not have, and an identity question becomes a colour question. Inferring it instead from emitted
artifacts is unsound in one direction, because emission is known to re-decide some calls from their
leaf spelling, which erases exactly the difference being looked for — a difference in the emitted
bytes proves divergence, but agreement proves nothing.

## The corollary the specimen produced, which is stronger than "check your probes"

Six separate times in one session, an instrument returned **empty** and the empty was not a
measurement:

- a required output flag was missing, so every invocation was an argument error;
- a target selection reported a different class entirely;
- an output filter truncated results ahead of their summary line;
- a fixture was authored where the runner could not see it, so the subject never existed;
- a filter matched neither the diagnostic nor the refusal the run actually produced;
- a malformed fixture was refused, loudly and with a location, by a compiler doing its job — and the
  same filter dropped the refusal.

Every one failed silently toward good news, and none was visible in the output. The pattern under
them is not carelessness, and naming it as such would miss the mechanism:

> **An instrument that cannot represent the answer returns empty, and empty is indistinguishable
> from "nothing was there."**

That is a property of building instruments faster than they can be validated, which is the normal
condition of investigative work. It is why the remedy has to be structural — plant the red — rather
than attentional.

**The last one is worth separating, because it nearly became a false finding.** The malformed
fixture blanked, and the blankness was read as evidence of a *grammar ambiguity* — a nullary variant
in scrutinee position being silently reinterpreted as a record-literal constructor, which would be a
serious defect: two readings of one text with no refusal on either side. Re-run with the filter
removed, the compiler says:

    module index refused: 1 unparseable .dag source(s)
      <probe>.dag:110-112: expected expression, found FatArrow

**It refuses — typed, located, fail-closed.** There is no silent reinterpretation and no ambiguity;
the substrate behaved exactly as DESIGN §5 requires, and the only defect was the filter standing
between the refusal and the reader. A blank output is consistent with *the compiler said nothing
wrong* and with *the compiler refused and I could not hear it*, and those have opposite owners —
one is a defect in the language, the other a defect in the instrument. **Attributing an instrument's
silence to its subject is how a clean mechanism acquires a bug report it did not earn.**

**A corollary for filters specifically, since four of the six were filters — and it names the
mechanism rather than the symptom:**

> **The filter is the instrument.** A grep for `^error|blocking` is not a *view* of the output; it is
> a *hypothesis about* the output, and it discards exactly the cases where the hypothesis is wrong.

Every unexpected shape — a refusal, an argument error, a crash — arrives as silence, which is why
this is not fixed by looking harder at what came back. Read the raw tail first; filter only once you
know what the instrument says when it fails. The same class covers any flag that suppresses a
channel you are about to draw a conclusion from: a quiet push that hides a rejection leaves you
claiming content that was never delivered.

## Why this is not a story about carelessness

Three earlier clean-looking zeroes preceded any real measurement: a missing required output flag that
made every invocation an argument error, a target selection that reported unrelated
transport-realization gaps instead of the class under study, and an output filter that truncated
results ahead of their summary line. **Each failed silently in the direction of good news.** None was
visible in the output; each was found by breaking something on purpose.

Then the fourth failure was subtler than all of them, and it is the one worth transferring: **the
planted red fired correctly and the generalisation from it was still wrong.** A single arm establishes
a single fact. Where the mechanism has more than one regime, one arm samples one regime and says
nothing about the others.

## What follows for work planned on an empty result

An empty sweep is not a finding until a planted red has failed to survive it — and one planted red
licenses one conclusion, not a law. Until then it licenses nothing: not *the class is clean*, not
*the repair already landed*, and above all not a repair PR built to close it. **A repair whose
subject produced no red before and produces none after is unverifiable by construction**: permanently
green, carrying no information, and worse than absent, because it is afterwards cited as coverage of
a class it never touched (DESIGN §4b).

In this instance the planted reds bought two things. They stopped a 47-module repair PR that would
have been unfalsifiable and that collided with a corpus-wide cut deleting the same headers. And when
the write-up over-generalised anyway, they caught that too.

## Appendix — the sweep, as text rather than as a committed process

The instrument this brief is about was originally committed as `sweep22.sh` at the repository root. It
is reproduced here instead, and the move is not tidiness: main carries **zero** `.sh` files anywhere
outside `.githooks/`, so a repo-root script would have been the first, re-opening a class the tree
closed deliberately (`gunbc#9132` bankrupted the last of them, and DESIGN.md's dead-citation cleanup
turns on that emptiness being a fact). A hand-authored investigative script with no modeled authority
and no dissolution trigger is the §6 tell for *manual application committed as source*; a fenced block
is prose, which is what this always was.

```sh
set -u
echo "MARKER_HEAD=$(git rev-parse --short HEAD)"
echo "MARKER_SRC_DIGEST=$(md5sum < src/v1/05_emit_rust.dag)"
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
BIN=./target/release/gunbc
echo "MARKER_BIN_DIGEST=$(md5sum < $BIN)"
rm -rf /tmp/cand
set +e
$BIN compile --source-root dag --source-root src/v2 \
    --entry src/v2/compiler/03_ingest.dag --output-dir /tmp/cand --target rust \
    > /tmp/emit.log 2>&1
echo "MARKER_EMIT_RC=$?"
set -e
tail -2 /tmp/emit.log
echo "MARKER_FILES=$(find /tmp/cand -name '*.rs' | wc -l)"
```

**Two things to fix before anyone re-runs it,** because copying it as-is reproduces the failure mode the
brief is about. `set -u` without `set -e` means a failing step before the compile continues silently;
and `set +e` around the emit is what allows a non-zero `MARKER_EMIT_RC` to be printed and then walked
past. An instrument whose own failure arm continues is exactly the shape that makes an empty result and
a broken result render identically.

**What is worth keeping is the `MARKER_` discipline itself.** Printing the head, the source digest and
the *binary* digest beside the result is what lets a later reader tell whether two runs are comparable
at all — the same obligation that now applies to every self-host measurement in this repository, where
an unproven compiler identity voids the result outright. This sweep had that instinct before the rule
was written down.
