# Standing after six configurations — Mt. Collins unit 1 memory screen

Written so the next session does not re-derive tonight's eliminations. Every row here is
either a measured outcome or an explicitly labelled hypothesis.

## Configurations tested, with outcomes

| Population | Micron spares present | Outcome |
|---|---|---|
| Original 16, odd connectors, 8+8 | no | **Trains.** 12m30s clean, both sockets, 8 W each |
| 31 sticks (16 original + 15 spare) | yes | Restart ~5m11s, `0fde70330102` |
| 15 sticks (8 socket0 + 7 socket1) | yes | `SOCKET_1 Disabled`, continued degraded, `Boot_Progress` |
| 32 sticks, mismatched same-channel pairs | yes | Restart ~5m04s, `0fde70330502` |
| 16 spares, odd connectors, 8+8 | yes | Restart 4m50s, `0fde70331102` |
| 12 original + 4 x 2DRx4 in SOCKET 1 | yes | Restart 4m50s, `0fde703b1102` |
| 12 original + 4 x 2DRx4 in SOCKET 0 | yes | Restart 5m20s, `0fde70331102` |

## Eliminated as explanations

- **Population shape.** 16 modules / 8 per socket / odd connectors is the arrangement that
  TRAINS with the originals. Table 11 enumerates it; Table 10's per-socket channel counts are
  satisfied at 8.
- **The machine, its slots, and its channels.** Originals train in exactly these positions.
- **Seating.** All 32 were reseated with no change; four failures at 5m08/5m11/5m19/5m11s.
- **Per-socket capacity mismatch.** Ampere requires DIMMs on all populated channels of one
  socket to be identically sized. Operator confirms ALL modules are 16 GB, so the rule is
  satisfied and explains nothing.
- **Same-channel rank/width/density mixing.** Never engaged: every test since the 32-stick
  round has been 1DPC, one module per channel, so Mt. Jade's same-channel cells do not apply.
- **DDR4-2133 being unsupported on Altra.** RETRACTED. srv1/srv3/srv4 run 8 x 64 GiB
  M393A8G40D40-CRB, modelled at data_rate_mts 2133, and srv1 POSTed cleanly on that upgrade.
  The SoC supports 2133. (srv1 is an ASRock ALTRAD8UD-1L2T, a different board on the same SoC
  family; speed support is an SoC property so the evidence transfers.)

## RESOLVED — read mtcollins1_dram_training_root_cause_2026-09-12.md instead

The three hypotheses below were superseded on 2026-09-12 by a SOL capture in which the platform
firmware states the cause directly: `ERR: CHANNEL Mismatch Byte 6 SLOT0[00] EXP[91] @MCU[4]` /
`ERROR:   Non-identical DIMM mixture NOT supported!`. Every DIMM in a socket must agree on SPD
byte 6 (package type / die count / signal loading). Hypothesis 3 was the closest — it is a
cross-channel constraint — but the axis is PACKAGE TYPE, not rank.

Hypothesis 1 is dead: the firmware names a configuration rule, not a failing module.
Hypothesis 2 is WRONG AND STILL UNTESTED: a uniform population of 16 dual-die modules has never
been run, and the "16 spares" row in the table above was itself a mix of `ADS` and `ASF` parts,
so it is not evidence for it. The open prediction is that 16 identical `ADS` modules train.

The eliminations above all stand, and the table's outcomes are all still correct as observations.
What was wrong was the reading of them.

## Standing hypotheses (SUPERSEDED — kept because they were scored, not to be consumed)

1. A defective module among the Micron group.
2. The module ORGANIZATION is not tolerated on this board -- the label reads `2DRx4`, which is
   not standard DDR4 notation, and `extdeps` carries NO Micron authority at all.
3. A cross-channel interaction on the rank axis, which Mt. Jade marks `MixingUndetermined`.

## Evidence quality warnings

- The eBay listing for `MTA36ADS2G72PZ-2G1A1` says `2Rx4`; the physical label says `2DRx4`.
  **The listing is wrong about organization, so it is not evidence about speed either.** A
  speed explanation built on `PC4-2133P` from that listing was withdrawn on that basis, before
  the srv1 evidence independently killed it.
- `MTA36ADS2G72PZ` is not the common family; the standard Micron 16 GB 2Rx4 RDIMM is
  `MTA36ASF2G72PZ`. Unverified, flagged for catalog lookup.

## Fleet precedent worth carrying

srv2's 64 GiB install FAILED memory training and was reverted, while the identical Samsung
parts trained on srv1, srv3 and srv4. Modules working elsewhere and failing on one host is a
class this fleet has already seen once, recorded at `gunbc.host_memory_qualification` -- which
is the same module that carries the purchasing fail-open found tonight.

## What the OEM payload has earned, and only this

OBSERVED: byte 3 bit 3 changed in step with which socket held the intervened population,
across two controlled crossovers whose direction was declared in advance. The tail was `1102`
on BOTH sides of that swap.

INFERRED, NOT DEDUCED: that the bit encodes the socket and the tail encodes something else.
The companion modules moved with the quartet, so the bit may track a correlated property -- a
restriction carried by the group, a channel-group index, a first-failing-controller identity --
rather than the socket. Two consistent crossovers support the reading; they do not establish
it, and it may not be used as a premise.

Stage, status and tail interpretations have NO controlled support. No vendor mapping from
payload bytes to physical slots exists. This may not be used to name a DIMM.

## Next discriminator — WITHDRAWN, it was not a discriminator

10 originals + 6 x 1Rx4 from srvN is a uniform-monolithic population, which the operator
confirmed had been running for days. It could only confirm what was already known, and under the
byte-6 rule it is PREDICTED to train for a reason that tests nothing. Withdrawn.

The experiment that would still earn something: 16 IDENTICAL `ADS` modules alone, which tests
whether dual-die is supported as a uniform population — the one claim this investigation asserted
without ever testing.
