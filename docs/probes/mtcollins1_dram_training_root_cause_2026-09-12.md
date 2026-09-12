# Root cause, stated by the platform firmware — Mt. Collins unit 1 DRAM training

The console was never silent. The collector was broken for a week, and when repaired it printed
the answer on the first failing boot.

Artifact: `artifacts/bmc/mtcollins1-sol-dram-training-2026-09-12.txt`, sha256
`e5044f71cde73fe9b2ea1e76c9b894c1ab2f007577b9cbb7e8d50650c2b78a4c`, 3764 bytes, captured over
IPMI SOL across the failing boot of 2026-09-12T02:37:23Z.

## What the firmware says

```
ERR: CHANNEL Mismatch Byte 6 SLOT0[00] EXP[91] @MCU[4]
ERR: 84f05923
ERR: 84f05800
ERR: 8ff0c500
ERR: bff0c100
ERROR:   SK[0]: 00110010
ERROR:     MC[4]: 00f10081
ERROR:   Non-identical DIMM mixture NOT supported!
```

SPD byte 6 carries `SdramPackageType` (bit 7, monolithic vs non-monolithic), `DieCount`
(bits 6:4) and `SignalLoading` (bits 1:0) — JEDEC Annex L, and modelled in edk2
`MdePkg/Include/IndustryStandard/SdramSpdDdr4.h`. The `ADS` modules report `0x91`; the SK hynix
monolithic modules report `0x00`. The firmware took its expectation from MCU0..MCU3 (the four
`ADS` modules), reached MCU4 (the first SK hynix), found `0x00` against an expected `0x91`, and
refused.

**The rule is that every DIMM in a socket must AGREE on SPD byte 6.** It is a cross-channel
constraint, not a same-channel one, and at 1DPC it is the only mixing axis in play — which is
why every same-channel elimination this investigation made was correct and irrelevant.

## It explains all eight configurations, including the one that did not fit

| Configuration | byte 6 across the socket | Outcome |
|---|---|---|
| Original 16, all SK hynix | uniform `0x00` | trains |
| 4x `MTA36ASF2G72PZ` + 12 originals | uniform `0x00` — `ASF` is MONOLITHIC | reached boot-device selection, kernel executed |
| 4x `MTA36ADS2G72PZ` + 12 originals, socket 0 | `0x91` vs `0x00` | fails |
| 4x `MTA36ADS2G72PZ` + 12 originals, socket 1 | `0x91` vs `0x00` | fails |
| 31 / 32 / 15 sticks, and the "16 spares" round | mixed `ADS` + `ASF` | fails |

The `ASF` round succeeded because `ASF` is monolithic like the SK hynix parts, so byte 6 agrees
— NOT because `2Rx4` is supported while `2DRx4` is not. That distinction is the whole finding.

## Claims retracted by this capture

- **"The 2DRx4 organization is not tolerated on this board."** WRONG, and untested. A uniform
  population of 16 `ADS` modules has never been run. The "16 spares" round that was cited as
  evidence for it was itself a mix of `ADS` and `ASF`. The prediction this makes, and it is
  falsifiable: **16 identical `ADS` modules alone should train.**
- **"A defective module among the Micron group."** Dead. The firmware names a configuration
  rule and identifies the mismatching byte.
- **The die-density hypothesis** (4 Gb devices outside Ampere's stated 8/16 Gb support). Already
  retracted when the working part number arrived; this confirms it. Both parts are 4 Gb die and
  one boots a kernel.

## The socket bit, fourth confirmation and the first direct one

`ERROR:   SK[0]` names socket 0, and the OEM payload for this same boot is `0fde70331102` whose
byte 3 reads socket 0 under Ampere's documented `(channel << 4) | type | (socket << 3)`. The
three prior supports were two declared crossovers, same-second co-emission, and vendor source;
this is the host itself.

NOT PROMOTED: the firmware reports `MCU[4]` while the payload nibble reads channel 3. The
channel nibble is therefore NOT simply the MCU index, and no reading of it is earned here.

## Per-DIMM visibility existed all along

The same capture carries a full 16-row population table — `SK<socket> MC<mcu> S<slot>`, SPD
manufacturer bytes, capacity, speed, ECC, rank, width, RCD identity and part number. This is
the per-DIMM localization the BMC does not provide and that four sessions searched the
controller for. It is on the host console during early boot, which is the one window nobody had
successfully observed.

## Why it took a week

The SOL collector reported eight configurations as "console silent". Every capture was 86 bytes
of the instrument's own voice, byte-identical across seven materially different subjects,
because `ipmitool` was invoked with stdin not a terminal, warned, hit stdin EOF, and exited
`rc=0` after roughly three seconds while the wrapper slept out the window. Repaired — pty with
stdin held open — the same client stayed connected 645 seconds and captured the text above.

Filed as the seventh instance of `gunbc.recurring_failure_mode`
`empty_capture_read_as_clean_result`, which is the class, and the rule that would have caught it
is that row's own sixth instance: a verdict whose green arm is the absence of bad lines cannot
distinguish healthy from unread.

## Consequence for the fleet purchase

Every DIMM in a socket must match on SPD byte 6. Purchase one uniform part number, or segregate
package types by socket. Capacity, rank, width and speed being identical is NOT sufficient —
they were identical across the failing mixes.
