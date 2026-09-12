# CORRECTION to the round-3 outcome record

`9e74a6b5262` claimed the round-3 prediction was FALSIFIED because "the 2Rx4 group trains".
Three things in it overreach and are withdrawn here. The underlying observations stand.

## 1. "The group trains" is not established

WHAT WAS OBSERVED: no restart reported through 373 seconds; aggregate memory power 10.4 W /
9.6 W on the two sockets; DIMM group temperature 44 C / 38 C; `Boot_Progress` asserted; the
`12c805000000` + `12c805004000` pair emitted.

WHAT WAS NOT OBSERVED: a memory-training completion terminal. None exists to observe --
gunbc#11110's `pre_os_bringup_verdict` deliberately has NO training-completion constructor, so
"the group trains" is a stronger statement than any model here can produce. I substituted a
correlated signal for the proposition.

A restart-free interval is a RIGHT-CENSORED observation: it establishes a survival interval,
not an infinite lifetime and not a success event.

## 2. The prediction was scored against the wrong endpoint

The committed prediction at `a28843d3180` says "It will fail" and names TRAINING as its
falsifier. I scored it against "no restart within a window plus higher power readings". Those
are different propositions:

- "a restart will occur within interval I" is contradicted by an adequately covered I with no
  restart
- "this configuration will not reach the required boot/memory state" is contradicted only by
  POSITIVE evidence that it reached that state
- "this configuration will eventually fail" cannot be contradicted by any finite quiet interval

The honest scoring: the restart-based reading of the prediction is contradicted for a 373-second
interval. The training-based reading -- which is what the artifact actually declared -- is NOT
YET SCORED, because the observation it needs was never obtained.

## 3. "Only organization remains" does not follow

I wrote that vendor, provenance, purchase window and claimed speed grade were exonerated
because both quartets share them, leaving organization. That is wrong in two ways:

- COHORT IDENTITY IS NOT ONE VARIABLE. Swapping four modules for four others changes four
  individual physical objects along with any construction, firmware-description and condition
  attributes that differ. A working substitute narrows the failing CONFIGURATION; it does not
  isolate the property responsible.
- THE PROCUREMENT HISTORIES DIFFER. The two quartets came from DIFFERENT orders and DIFFERENT
  sellers. "Both from the used market" is a category, not a common lot, handling history, or
  individual condition.

The supportable statement: this exact replacement population reached the observed endpoints on
this unit; the population containing the `2DRx4` quartet remains associated with failure; the
causal difference is not isolated.

## 4. A real model gap, worth more than this incident

`extdeps.memory.types` defines `DramDieStacking = MonolithicDie | ThreeDimensionalStacked`,
and `DramModuleCatalogRow.die_stacking` requires one of them. That cannot represent
NON-3DS MULTI-DIE construction, which is a third thing.

Micron's module numbering system distinguishes these as SEPARATE FIELDS: the `DS` module option
denotes very-low-profile DUAL-DIE construction with a temperature sensor, while `P` denotes an
RDIMM and `PS` a 3DS master/slave RDIMM. `MTA36ADS2G72PZ-2G1A1` carries `DS` and `P` -- so it is
dual-die and is NOT 3DS. The `-2G1` grade denotes 2133 MT/s, which is the first independent
confirmation of that speed from the numbering system rather than from the seller's listing.

So the operator's hypothesis -- that these are low-profile parts and that is why the label reads
`2DRx4` -- is SUPPORTED by the manufacturer's own naming system. It remains unestablished that
dual-die construction is unsupported by Mt. Collins; what is established is that our catalog
shape is too coarse to ask the question without fabricating an answer. Forcing this part into
`ThreeDimensionalStacked` to construct a row would invent a physical fact.

## What still stands from round 3

The two tested cohorts produced DIFFERENT observed behaviour under a comparison that held the
population shape, socket, connectors and twelve companion modules constant. That contrast is
real and it is the most useful thing tonight produced. It is not a diagnosis.
