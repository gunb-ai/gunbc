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

## The fifth instance is the worst shape: agreement with the control, for the wrong reason

Found by `silent-gull-867` in their **own probe**, before running it.

gunbc#9103 was built to settle a `PointwisePower` residue by measurement rather than argument — a
scratch branch carrying #9059 plus one withheld row, whose floor result differenced against #9059's
was the discriminator. It was branched from #9059 **before the mirror existed**, so the withheld row
sat in the `.dag` and not in the binary that runs the floor.

Its result would have come back **identical to #9059's**, for a reason having nothing to do with
`PointwisePower` — and *unchanged* is precisely the outcome the probe was pre-registered to read as
**risk not realised**. A measurement whose entire purpose was to stop an argument being shipped
would have shipped it.

**That is the sub-shape worth naming, and it is not the same as the three above.** Those produced a
wrong number or a wrong attribution. This one produces **no error at all** — the run succeeds, the
numbers are internally consistent, the control and the treatment agree, and the agreement is an
artifact of the treatment never having been applied. In their own words:

> On this substrate a `.dag` change is not measurable until its mirror is regenerated, and the
> failure mode is not an error — it is a run that agrees with the control for the wrong reason.

A null result from a stale binary is indistinguishable from a null result from a real one. Every
other member of this class announces itself eventually; this one is **silently confirmatory**, and
it is worst precisely in the probes built to keep their authors honest.

### The silent member has no self-detection route — it was found by its loud sibling

`silent-gull-867` corrected the credit given here, and the correction is the most useful fact in
this document. An earlier revision said they saw the false negative *before running it*, which
implies suspicion did the work. It did not:

> I hit the staleness on #9076 first, where it produced a LOUD failure — the floor refusing with an
> unchanged error while regen named the file — and only then asked whether the probe I had cut from
> that same head shared it. The loud instance is what made the silent one visible. Had #9076 not
> failed first, #9103 would have come back "unchanged" and I would have closed the residue on it.

**Nothing about #9103 in isolation would have prompted the check, and its pre-registration would
have actively discouraged one** — *unchanged → risk not realised* is an instruction to accept the
very string a stale binary produces.

So the detection route for the silent member is: **a sibling sharing its cause failed loudly, and
the author generalised from it.** That is not a method. It requires a loud sibling to exist, to be
noticed, to be diagnosed correctly, and to be recognised as sharing a cause with something that has
not failed — four conditions, none of them under the author's control, and all four happened to hold
here.

The practical consequence is the one that matters for the repair: **you cannot catch this after the
fact.** Every other member of the class announces itself eventually — a wrong number gets
questioned, a refused gate names its file. The silent member's only tell is a confirmation you were
already expecting. The property must therefore be enforced **before** measurement, which is why the
repair belongs at commit or dispatch time and not in anyone's reading discipline.

Confirmed by measurement rather than left as inference, using the guard proposed for #9063:

```
MARKER_DAG_PP=1             the row IS in dag/std/algebra.dag
MARKER_MIRROR_PP_BEFORE=0   the row is NOT in the mirror the binary is built from
```

and after installing the regenerated mirror, 6581 bytes against #9059's 6564 — **a 17-byte delta
that is itself the cheap confirmation that the two mirrors differ by exactly the treatment and
nothing else.**

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

---

## Two communication failures the collation exposed

**A caution restated by a third party becomes a claim.** The retracted 830 finding was sent to
`silent-gull-867` as *how to read a result* — read `declined_live` beside pass/fail, because an
absent row may be a declined one. It came back as *"the arm that swallows 830 identities... the
ladder inverted"*: a number and an arm name generalised into an assertion about a mechanism nobody
in that exchange had traced. Nothing was misquoted. The **mood** changed, from caution to claim, and
a claim propagates where a caution would have prompted a check.

Their own account of why, which is the sharper half and applies far past this instance:

> I did the careful thing on the half that touched my own work and the credulous thing on the half
> that did not.

That is the allocation problem. Scrutiny goes where the author has skin, and a finding handed over
by someone else arrives pre-vetted by their apparent authority. Both of us did it in the same hour —
they took my mechanism claim without opening `run_required_floor`; I took a call-site list as an
answer to a question it was not asked.

**A retraction does not catch up with what it retracts.** The 830 claim was retracted at the top of
its brief an hour before it was restated. Putting the correction first is necessary and was not
sufficient: the reader had already formed the belief, and nothing re-reads a document you have
already read. The only thing that actually stopped it was a direct message to the specific person
who had it. **Corrections must be pushed to the holders, not published to the artifact** — the
artifact fixes the next reader, not the current one.
