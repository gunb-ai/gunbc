# Outcome — Mt. Collins unit 1, 2DRx4 socket swap (round 2)

Scores the prediction committed at `cd83f731921` BEFORE the hardware was touched. A prediction
that is never scored is a note, not a test.

## Observed

Power-up 00:16:55Z. Restart 00:22:15Z (5m20s). OEM payload `0fde7033 1102`.
Prior round, quartet in socket 1: `0fde703b 1102`.

## Prediction assessment

**MATCHED, both halves.** The prediction was a conjunction — that it would fail, and that the
payload would report `33` rather than `3b`. It failed, and the socket field followed the
quartet to socket 0.

## What the experiment DISTINGUISHED

The candidate socket field (bit 3 of payload byte 3) tracks the socket holding the moved
population, across two controlled interventions whose direction was declared in advance. That
is a second controlled confirmation of the socket-selector reading.

Separately: the TAIL is `1102` in both rounds, unchanged while byte 3 flipped. That is an
OBSERVATION, and the inference drawn from it is bounded accordingly.

WHAT IS OBSERVED: byte 3 changed between the two rounds; the tail did not.
WHAT IS INFERRED, AND ONLY INFERRED: that byte 3 carries the socket and the tail carries
something else. This does NOT follow deductively, because the companion modules changed
together with the socket (see below) -- so the bit may be tracking some other property that
moved with the intervention rather than the socket itself. A restriction attached to the
quartet, a channel-group index, or a first-failing-controller identity would each produce the
same movement.

Calling this "the socket lives in byte 3" would assert as deduced what is only inferred
(DESIGN section 4d). It is a SUPPORTED HYPOTHESIS with two consistent controlled crossovers,
and it must not become the next session's premise.

## What it did NOT distinguish — the correction that matters

An earlier decision table for this round claimed `33` would establish that "the fault follows
the modules". IT DOES NOT. A configuration restriction violated WHEREVER that quartet is
installed predicts exactly this outcome, identically to a defective member of the quartet.
The crossover cannot separate them.

Remaining indistinguishable:
- an individually defective module among the four
- a replacement/companion population interaction
- an applicable configuration restriction carried with the group

Further confound, also missed in the original table: the COMPANIONS CHANGED TOO. The originals
are a mixed cohort, so "four replacements + four socket-1 originals" and "the same four +
four socket-0 originals" are not one population relocated. The experimental unit for a clean
socket comparison is the complete eight-module population, which this round did not preserve.

## Hypothesis eliminated this round

Per-socket equal capacity. Ampere's published rule requires DIMMs on all populated channels
of one socket to be identically sized; the operator confirms ALL modules are 16 GB, so the
rule is satisfied and cannot explain any of the five failures. This was the best available
alternative explanation and it is dead.

## Standing after five configurations

Eliminated: population shape (16 / 8-per-socket / odd connectors is proven good with the
originals), the machine and its slots, per-socket capacity mismatch, and same-channel
rank/width/density rules (never engaged — every test has been 1DPC).

Still standing: a defective module; a cross-channel interaction on the rank axis the
specification marks TBD; or the 2DRx4 organization being unsupported on this platform.

## Narration errors retracted

- "0 W memory power is consistent with failing early rather than timing out" — those are not
  mutually exclusive, and the sensor's validity and freshness were never established. The
  readings are recorded as associated with outcomes, not as a training terminal.
- The three-outcome table was neither exhaustive nor equiprobable. Both socket variants,
  missing records, partial boot, and incomplete collection were all possible outcomes.

## The decoder's standing is unchanged by this

Two controlled confirmations of ONE FIELD. The stage, status and tail interpretations have no
such support, and no vendor mapping from payload bytes to physical slots exists. This may not
be used to name a DIMM.
