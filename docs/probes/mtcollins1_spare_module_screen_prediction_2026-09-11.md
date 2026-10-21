# Preregistered prediction — Mt. Collins unit 1, all-spare 16-DIMM screen

**Committed BEFORE the hardware was touched.** This file exists so the prediction can be
checked against the outcome by someone who does not trust the person who made it. Tonight's
earlier 32-DIMM test was NOT preregistered — the prediction was stated in conversation only,
which is why it is classified as the weaker thing it is.

## The configuration to be tested

Sixteen SPARE modules, one per channel, in the odd-numbered connectors
(`J1, J3, … J31`), eight per socket. None of the currently-installed sixteen.

Operator's spare inventory as reported: 8 × 1Rx4, 4 × 2Rx4, 4 × 2DRx4, 1 × 2Rx8
(17 modules; 16 of them to be used).

## Basis

- `extdeps.ampere.mt_collins_product_brief.memory_population` — Table 11 enumerates a
  16-DIMM population at connectors `[1,3,5,…,31]`; Table 10 gives supported channel counts
  per socket as 1, 2, 4, 6, 8. Eight per socket is supported.
- `extdeps.ocp.mt_jade.memory_mixing` — same-channel mixing constraints apply only where a
  channel holds two modules. At 1DPC there is no same-channel pair, so rank, width and
  density mixing rules are **not engaged**.
- Different-channel rank mixing reads `MixingUndetermined` (the specification's TBD), and
  this machine already runs a cross-channel 1R/2R mix in its working configuration.

## The prediction

**This configuration will complete firmware training and reach POST**, on the same evidence
signature as the working set: no `Sys_Boot_Restart` within the ~5m11s window that has
terminated every failing attempt tonight, both sockets asserting `S0/G0: working`, and
`MEM_Pwr` sustained on both sockets.

## What would falsify it

A restart inside the ~5-minute window, or an OEM payload in the `0fde7033…` family.

## What each outcome earns — stated in advance

- **Trains** → the sixteen spares are collectively functional in a supported population.
  It does NOT establish that any individual module is good, nor that they work at 2DPC.
- **Fails** → at least one spare is faulty, OR this particular cross-channel combination is
  not tolerated. It does NOT identify which module, and the platform exposes no per-DIMM
  telemetry to narrow it (verified: no DIMM FRUs, no per-slot sensors, 167 SDR entries all
  per-socket aggregates).

## Known weaknesses of this test, stated in advance

Sixteen modules change at once, so a failure localizes to nothing. It is a SCREEN, not a
diagnosis. It was chosen over a narrower test because the configuration shape is already
proven, which makes the modules the only variable.
