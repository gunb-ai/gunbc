# Preregistered prediction — Mt. Collins unit 1, 2Rx4 group in socket 0

**Committed BEFORE the outcome is known.** Round 3. Directly comparable to round 2: same
population shape, same socket, only the identity of the four substituted modules differs.

## Configuration

12 original modules + **4 x Micron `2Rx4`** installed in SOCKET 0, odd connectors, 8 per
socket, 1DPC. The `2DRx4` quartet from round 2 is removed.

Round 2 for comparison: same shape, 4 x `2DRx4` in socket 0 -> restart 5m20s, `0fde70331102`.

## The prediction

**It will fail.** Both Micron quartets came from the same used-market purchase window, both
listings claim PC4-2133P, and the all-spares round that contained both groups failed. Nothing
currently distinguishes the two groups except a label reading we have not verified.

**Confidence: moderate-low.** No test has ever isolated the `2Rx4` group. If it trains, the
prediction is falsified and the finding is sharper than a confirmation would be.

## What each outcome earns

- **Fails** -> consistent with the Micron group being implicated as a group. Does NOT identify
  which property is responsible: vendor, speed, organization, provenance and individual health
  all remain confounded, because these modules differ from the originals on several axes at
  once.
- **Trains** -> the `2Rx4` group is functional in this position, and the responsible subset
  narrows to the `2DRx4` quartet specifically. That would make the module ORGANIZATION the
  leading hypothesis and make `MTA36ADS2G72PZ` the single most valuable thing to identify.
- **Fails with a DIFFERENT tail** than round 2's `1102` -> the tail discriminates something
  about the population. Worth recording carefully: the tail was `1102` for the all-spares
  round and for BOTH 2DRx4 positions, so a new value here would be its first variation under a
  module-only change.
- **Fails with the same `33 1102`** -> the payload does not discriminate module identity in
  this position, which bounds how much the record can ever localize.

## What no outcome earns

Four modules move together, so nothing here names one. A module merely untolerated by this
board is indistinguishable from a defective one. And the socket-field reading of byte 3
remains an INFERENCE, not a decode -- it may not be used to name a DIMM or a slot.
