# Root cause, stated by the platform firmware — Mt. Collins unit 1 DRAM training

The console was never silent. The collector was broken for a week, and when repaired it printed
the answer on the first failing boot.

## The four captures, and what each one grounds

Review 64333 found that two of these were landed with nothing in this account reading them, which
is DESIGN section 3c's tell for an artifact as much as for a data row, and section 2's redundant
work. Each now states what it grounds, or it should not be here.

- **`artifacts/bmc/mtcollins1-sol-dram-training-2026-09-12.txt`**, sha256
  `e5044f71cde73fe9b2ea1e76c9b894c1ab2f007577b9cbb7e8d50650c2b78a4c`, 3764 bytes, captured over
  IPMI SOL across the failing boot of 2026-09-12T02:37:23Z. **Grounds the entire finding**: the
  firmware's own statement of the rule, the compared byte-6 values, and the 16-row per-DIMM
  population table.

- **`artifacts/bmc/mtcollins1-sel-full-2026-09-12T0355Z.txt`**, sha256
  `142ff5e254f50d138b3f744f844cfc6138ff75e2ee17d12556ae590ed5a5d31f`. **Grounds two things**: the
  controller's four-hour clock offset, readable from its first three lines; and the OEM payload
  records bracketing the SOL session, at records `518`, `51e` and `51f`.

- **`artifacts/bmc/mtcollins1-sel-full-2026-09-12.txt`**, sha256
  `8d93cb24349549ef4fefb884a56dbe9f72cc03b0530fa559afdca089b5250f89`. **Grounds the payload-family
  census** — the counts and co-emission structure that supported the socket-bit reading before the
  firmware stated it directly, including the same-second `0fde7033ff10`/`0fde703bff10` pair. It is
  also the capture that ENDS BEFORE the SOL boot, which is what made the first version of the
  co-timing claim in this document ungrounded; it is retained because that is a receipt, not an
  embarrassment to hide.

- **`artifacts/bmc/mtcollins1-controller-fru-2026-09-12.txt`**, sha256
  `dc63dabe07764dcf7688a74230e7fe1a6151ea384435f10460a1979eda637d3c`. **Grounds unit identity** —
  that every reading in this document is about `mtcollins1` and not some other Mt. Collins. It
  carries the controller's IPMI FRU board serial `02030A800TEXFT02L` and product serial
  `MXX2080619`, which the host reads independently from its own SMBIOS. Two transports, two agents,
  the same two serials; either alone identifies a reading rather than a unit. Its argv is on its
  first lines and `exit=0` on its last, so a missing read is distinguishable from a zero one.

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

`ERROR:   SK[0]` names socket 0 — from the host, which is the point. The three prior supports
were two declared crossovers, same-second co-emission, and vendor source.

**THE CO-TIMING CLAIM WAS UNGROUNDED WHEN FIRST WRITTEN AND REVIEW 64298 WAS RIGHT.** It asserted
that the OEM payload `0fde70331102` was "for this same boot" while the SEL artifact this probe
landed ENDED BEFORE that boot, containing no record of it at all. The reading existed only in a
session transcript, never in a committed artifact — which is the capture-discipline failure this
repository has a standing rule against, committed by the author of the rule.

It is now grounded, by a second SEL read taken for this purpose:
`artifacts/bmc/mtcollins1-sel-full-2026-09-12T0355Z.txt`, sha256
`142ff5e254f50d138b3f744f844cfc6138ff75e2ee17d12556ae590ed5a5d31f`, carrying its argv, the
controller's own clock reading, and `exit=0`. Record `51e` is the restart and `51f` is
`0fde70331102`.

**AND JOINING THE TWO ARTIFACTS REQUIRES A FACT NEITHER OF THEM STATES ALONE — the controller's
clock is wrong by exactly four hours while labelling itself UTC.** The measurement is the first
three lines of `mtcollins1-sel-full-2026-09-12T0355Z.txt`: line 1 is the host clock at capture,
line 2 is the argv that read the controller's clock, and line 3 is the controller's answer. Read
them as a pair — the seconds agree, the hours differ by four, and the controller's day is the
previous one.

The readings are deliberately NOT re-typed here. An earlier revision of this paragraph quoted a
host/controller pair taken from a live probe about half a minute before the committed capture, so
the numbers in the prose appeared nowhere in the artifact the prose cited — the same defect this
section exists to retract, committed inside the retraction. Review 64318 found it. DESIGN section 6
is the rule: name the instrument, never transcribe its output, because a transcribed number is
unreachable from the thing that owns it.

So every timestamp in every SEL capture from this unit is four hours behind real UTC, and reads
as a different calendar day. The reviewer's observation that the artifact contains no `09/12/2026`
rows is correct and is explained by this, not by the boot being absent.

Applying it: the SOL capture's own first line carries its start; SEL records `518`
(`S0/G0: working`), `51e` (`System Restart`) and `51f` (the `0fde70331102` payload) sit at lines
1308, 1314 and 1315 of that capture. Add four hours to their controller timestamps and they
bracket the SOL session — power-on just before it attached, restart under five minutes later. The
payload and the console text describe one boot.

This is a JOIN ACROSS TWO CLOCKS, one of which is known wrong, so it is an inference and not a
deduction. What would make it one: a shared identifier in both streams, which neither carries.

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

The class is `gunbc.recurring_failure_mode` `empty_capture_read_as_clean_result`, and the rule
that would have caught it is that row's own sixth instance: a verdict whose green arm is the
absence of bad lines cannot distinguish healthy from unread.

**THIS PR DOES NOT CONTAIN THAT FILING, and an earlier draft of this sentence claimed it did.**
Review 64298 found it: the carrier on this branch still ends at its sixth instance, so prose
asserting a filing that is not on the carrier is a parallel ledger (DESIGN section 3) and a
fabricated done-state (section 5). The seventh receipt is appended on the `session/eager-owl-205`
branch, PR #10965, at commit `606251b3fa2` — a different branch, which is exactly why the claim
did not hold here. It is deliberately not duplicated onto this branch: two lanes appending the
same row is the merge collision the one-class-per-file layout exists to avoid.

## Consequence for the fleet purchase

Every DIMM in a socket must match on SPD byte 6. Purchase one uniform part number, or segregate
package types by socket.

**A CLAIM THAT STOOD HERE IS WITHDRAWN, and it was the one most repeated to the operator.** The
text read "capacity, rank, width and speed being identical is NOT sufficient — they were identical
across the failing mixes." **They were not.** The capture that produced this refusal holds two
populations that differ on speed AND rank: the Micron rows read `16GB 2133 ECC 2R x4` and the SK
hynix rows read `16GB 2666 ECC 1R x4`. Only capacity and device width actually match.

The error was conflating two different comparisons. The two MICRON SIBLINGS
(`MTA36ADS2G72PZ-2G1A1` and `MTA36ASF2G72PZ-2G1A2`) genuinely are identical on every part-number
field except module options, and that is the clean discriminator for the package axis. The
CONFIGURATION THE FIRMWARE ACTUALLY REJECTED was Micron-against-SK-hynix, where speed and rank
differ as well. Extending the sibling comparison onto the captured mixture is the join defect this
document already retracts twice elsewhere, committed a third time in its own conclusion.

WHAT THE EVIDENCE STILL CARRIES, undiminished: the firmware named **byte 6** and no other field. It
did not cite the speed difference or the rank difference that were also present, which is what
keeps the package axis the operative one. But the rule may not be advertised as "everything else
matched", because in the observed rejection it did not.

The honest purchasing justification needs no overstatement: a directly observed, firmware-rejected
package-byte mixture exists on this platform, so procurement must avoid that configuration, while
qualification and applicability stay separate obligations. "One uniform part number per socket"
remains a sound CONSERVATIVE purchasing rule — and it must not be presented as a universal firmware
law, nor as evidence that any particular uniform population trains.
