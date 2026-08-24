# Four lanes, one night, one class: a `.dag` edit reaches the executing binary only through regen

Measured 2026-08-24, 06:00–10:15Z, across four independent sessions that were not working together
and did not know they shared a failure until it was collated here.

## The class

`gunbc` is built from the Rust seed. A `.dag` authority edit changes what the seed *should* be; it
changes what the seed *is* only after regen emits the mirror and the mirror is committed. Between
those two moments the tree contains an authority and a realization that disagree, and **every
measurement taken through the binary answers a question about the old authority.**

## The four

| lane | authority edited | mirror | how it surfaced |
|---|---|---|---|
| gunbc#9063 | `src/v1/05_emit_rust.dag` (token attestation) | `v1_compiler_emit_rust.rs` never regenerated into the commit — the token commit touched **one file, 18 insertions** | found in review, by grepping the committed mirror for the new symbol |
| gunbc#9076 | `dag/std/algebra.dag` (`kernel_algebra_profile`) | `std_algebra.rs` still carried the seven-key map | CI: `regen FAIL generated surface drift: std_algebra.rs`, **17 minutes before** the floor refused for that exact reason |
| gunbc#9075 | `std.primitive_projection` | `std_primitive_projection.rs` drifted by generator-required expression braces | CI required-regen |
| gunbc#9058 | `.dag` only, evidence `.dag`-side | n/a | unaffected — the control case |

**Three of the four drew a conclusion from a measurement taken through a binary that lacked their
change**, and in two of them the wrong conclusion was specific and confident:

- #9063 attributed a 20-line improvement to a gate change that was not in the executing emitter.
  A producer-side explanation fits the same numbers and was not excluded.
- #9076 read an unchanged floor error as *the fix did not work* rather than *the fix has not
  arrived*.

## Why four, and why in one night

**The gate is correct.** Required-regen detects the drift and names the file. Nothing is broken
about the detection.

**It arrives too late to prevent the class.** The only place mirror drift is caught is CI, roughly
20–30 minutes after the edit, in a phase separate from the symptom it produces. By then the author
has usually already dispatched a measurement, read a number, and formed a conclusion.

Measured: `.githooks/pre-commit` and `.githooks/pre-push` run **`cargo fmt` and nothing else**.
There is no local regen check at all. So the feedback loop for *your authority edit has not reached
your binary* is one full CI round-trip, and the signal arrives beside — and sometimes long before —
the downstream symptom that actually gets the author's attention.

## What #9076 got right, and it is the most transferable thing here

Its ledger printed both failures, in order:

```
required-ci: regen first_generation_equal=false
required-ci: regen FAIL generated surface drift: std_algebra.rs
...            (17 minutes later)
required-ci: floor refused: ... 'Node(std.algebra.PartialFunction)'
               establishes no method surface
```

**Two failures in one ledger, and their order is the diagnosis.** That is a direct argument for the
independent-phases design: a line-stopping regen phase would have shown the drift and *hidden* the
floor error, leaving one fact instead of the relation between two — and the author would have had
no way to see that the second was caused by the first. Independent phases are usually justified as
completeness; here they were load-bearing for *attribution*.

## The property a repair must have

Stated as a property rather than a mechanism, deliberately — see the reviewer trap in
`instrument-traps-2026-08-24.md`:

> **An author who edits a `.dag` authority in the seed closure should learn, before they measure
> anything, that their change has not reached the binary.**

Candidate mechanisms to check rather than implement: `claim_executor --required-regen` in
`pre-push` (correct but possibly too slow to be tolerable); or a cheap syntactic pre-push check that
a commit touching a seed-closure `.dag` also touches its mirror (fast, but false-positives on
comment-only edits — loudly, which is the right direction, and confirmable by running regen).
Neither is specified here; the population and the property are.

## What is NOT claimed

That any of the four authors was careless. Three of them found their own drift, one within the same
ledger that reported it. The class is a feedback-latency defect, not an attention defect — and the
evidence for that is that it caught four people in four hours on four different files.
