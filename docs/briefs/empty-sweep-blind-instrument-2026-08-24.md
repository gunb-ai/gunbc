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
