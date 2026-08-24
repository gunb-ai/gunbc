# Before trusting an empty sweep, break something the sweep should catch

**The rule, and it is the whole brief.** An empty result has two readings — *nothing is wrong* and
*the instrument cannot see* — and they are **byte-identical at the output**. Nothing about reading
the result more carefully separates them. One command does: plant the defect the sweep exists to
find, and see whether it goes red.

This is a companion to [instrument traps](instrument-traps-2026-08-24.md), whose diagnostic question
it answers in the one case where that question has a cheap mechanical answer. That brief observes
that *a false absence is worse than a false green*, because an absence terminates the investigation.
The planted red is what stops it terminating.

## The specimen

A lane was dispatched to finish an *incomplete-header sweep* over `src/v2/compiler`, on the premise
that a partial import header is **already broken** — that each module carrying one refuses when
compiled alone, in the manner of `v2.compiler.program_assembly` in gunbc#9083.

Compiled entry-scoped, every module in the universe reports zero blocking errors. Read forward, that
is a clean corpus and the brief's premise was simply already satisfied.

It is not. `v2.compiler.compile` carries a 33-line import header. Delete **the entire header** and
recompile with the same binary on the same entry: still zero blocking errors. The only effect is
additional non-blocking `unlisted import use` advisories. Rename one called function to a symbol that
exists nowhere, and the same instrument refuses immediately with a located
`function … not found in scope` — so it is not inert; it is **blind to the specific question the
sweep was asking.**

**What the sweep was measuring was closure membership, not headers.** A name resolves because
something in the compiled closure defines it, and the import statement contributes nothing to that
outcome. `sleek-ant-767` reached the same conclusion the same day from the construction side, on a
different instrument and a different fixture, by building the four admission cases rather than by
deleting one.

The premise was therefore not *satisfied* — it was **unmeasurable by the method chosen**, and
without the planted red the two are indistinguishable.

## Why this is not a story about carelessness

The same sweep produced three earlier clean-looking zeroes before any measurement existed at all: a
missing required output flag that made every invocation an argument error, a target selection that
reported unrelated transport-realization gaps instead of the class under study, and an output filter
that truncated results ahead of their summary line. **Each failed silently in the direction of good
news.** None was visible in the output; each was found by breaking something on purpose.

That is the point [instrument traps](instrument-traps-2026-08-24.md) makes about attention, arriving
from a different direction: no amount of care reads a distinction out of a channel that does not
carry it.

## What follows for work planned on an empty result

An empty sweep is not a finding until a planted red has failed to survive it. Until then it licenses
nothing — not "the class is clean", not "the repair already landed", and above all not a repair PR
built to close it. **A repair whose subject produced no red before and produces none after is
unverifiable by construction**: permanently green, carrying no information, and worse than absent,
because it is afterwards cited as coverage of a class it never touched (DESIGN §4b).

In this instance that is what the planted red bought: it converted a 47-module repair PR — which
would have been unfalsifiable, and which collided with a corpus-wide cut deleting the same headers —
into a one-line finding about the instrument.
