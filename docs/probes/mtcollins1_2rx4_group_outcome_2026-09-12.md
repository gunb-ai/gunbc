# Outcome — round 3, Micron 2Rx4 group in socket 0

Scores the prediction committed at `a28843d3180` before the outcome was known.

## Observed

Power-up 01:06:30Z, both CPUs `S0/G0`. At 01:12:43Z — **6m13s, no restart**, no OEM record.
`CPU0_MEM_Pwr` 10.40 W, `CPU1_MEM_Pwr` 9.60 W, both climbing.

The failure window that terminated all seven prior failing configurations is ~5m04s–5m20s.
This cleared it by a minute, and the memory-power signature matches the ONE configuration that
trained (original 16: 8 W both sockets) rather than the failing ones (0–1.6 W).

## Prediction assessment

**FALSIFIED.** I predicted failure at moderate-low confidence, reasoning that both Micron
quartets shared purchase window, claimed speed grade, and provenance, and that the all-spares
round containing both had failed.

## What this establishes

The two Micron quartets behave DIFFERENTLY in the identical position. Round 2 put `2DRx4` in
socket 0 in this exact shape and it failed at 5m20s with `0fde70331102`; round 3 put `2Rx4`
there and it trained.

Because the two groups SHARE vendor, purchase window, used-market provenance and claimed speed
grade, those four properties are now **exonerated** — they cannot explain a difference they do
not carry. What distinguishes the groups is module ORGANIZATION, whatever `2DRx4` denotes.

## What it does NOT establish

- That any individual `2DRx4` module is defective. Four move together, and a group untolerated
  by this board is indistinguishable here from one containing a bad module.
- That the `2Rx4` quartet is qualified. It reached training in one attempt; it has not run a
  memory workload, has not been EDAC-checked under load, and has not been observed at 2DPC.
- That `2DRx4` is the CAUSE rather than a correlate of something else these four modules share.

## Consequence for the purchase

Half of the eight new Micron modules are usable in this machine now. The other four are not,
pending identification. This is a part-level answer rather than another elimination, and it
arrived from a falsified prediction rather than a confirmed one.

## The single most valuable remaining datum

`MTA36ADS2G72PZ-2G1A1` identified against Micron's own documentation — organization, die
construction, and what `2DRx4` denotes. `extdeps` carries no Micron authority at all, and the
standard 16 GB 2Rx4 RDIMM is `MTA36ASF2G72PZ`, so `ADS` is a different family.

## Unchanged

The byte-3 socket reading remains an INFERENCE, not a decode. No vendor mapping from payload
bytes to physical slots exists. Nothing here may name a DIMM or a slot.
