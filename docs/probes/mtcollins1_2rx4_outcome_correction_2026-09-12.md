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

## ADDENDUM — a positive readback that is not telemetry

Obtained after the correction above was written, with srv1/srv2 access granted.

**The one-shot boot override was CONSUMED.** Before the 01:17:06Z reset, `chassis bootparam
get 5` read `Boot Flag Valid` / `BIOS EFI boot` / `Force Boot from CD/DVD`. At 01:33:16Z it
reads `Boot Flag Invalid` / `No override`.

Firmware clears a one-shot override WHEN IT READS IT to select a boot device. So this
configuration reached BOOT-DEVICE SELECTION.

WHY THIS IS BETTER EVIDENCE THAN THE WATTS: it is the same instrument used in the opposite
direction during the failing rounds, where the override remained VALID AND UNCONSUMED across
multiple restarts -- which was the evidence that firmware never reached boot selection there.
One discriminator, both polarities, observed on the same unit. It is a control-plane fact about
firmware behaviour rather than an aggregate sensor reading whose validity and freshness were
never established.

WHAT IT ESTABLISHES: POST completed and boot-device selection was reached. Together with
`Boot_Progress` asserted at 01:18:46Z and the `12c805000000`/`12c805004000` pair at 01:19:30Z
-- the pair's first appearance since the DIMM change, having been static at 49 occurrences all
evening.

WHAT IT STILL DOES NOT ESTABLISH: memory training completion as a terminal (no constructor for
it exists), that all sixteen modules were recognized, capacity, or any qualification. The census
image's output was NOT retrievable: /srv/bmc holds only ISOs with nothing written since
2026-09-10, the host NIC 28:c1:3c:8a:d6:4a appears in neither srv1's nor srv2's ARP table, and
SOL returned 86 bytes identical to every other capture tonight. The image reports over serial
only, and that channel is dead on this controller.

SO THE POPULATION-LEVEL READBACK REMAINS UNOBTAINED, and by a capability gap rather than a
choice: this platform exposes no per-DIMM telemetry out-of-band, and the in-band route requires
a console that does not work. Closing it needs either a working console, a diagnostic image that
writes to the share instead of the console, or host network reachability.
