# Preregistered prediction — Mt. Collins unit 1, 2DRx4 group isolation

**Committed BEFORE the hardware is touched.** Round 1 of a bisection whose purpose is to
find which of the spare modules is responsible for the training failure recorded at
`0fde70331102`.

## Configuration to be tested

Sixteen modules in the odd connectors (`J1, J3, … J31`), eight per socket:

- **12 modules from the ORIGINAL set** — the population proven to train (12½ minutes clean,
  both sockets at 8 W, 2026-09-11 23:17–23:30).
- **4 × `2DRx4` spare modules** — the group whose part notation this corpus cannot identify.

Only those four modules differ from a known-good configuration.

## Revised inventory basis

Operator-reported spares, corrected 2026-09-12: 6 × 1Rx4, 4 × 2Rx4, 4 × 2DRx4, 1 × 2Rx8
(15 modules). An earlier report said 8 × 1Rx4; the corrected count is used here.

## Prior being deliberately NOT tested first

The operator reports the 6 × 1Rx4 spares ran for months in the srvN fleet hosts and is
confident they are good. That is a prior from a *different machine*, not a result on this
platform, and it is scheduled for a later round rather than assumed. It is recorded here so
that if a later round falsifies it, the prior is visible as something that was held and
displaced.

## The prediction

**This configuration will FAIL to train**, on the established signature: a
`Sys_Boot_Restart` within ~5 minutes of power-up and an OEM payload in the `0fde7033…`
family.

Basis: the `2DRx4` group is the least characterized, and a dual-die or otherwise unusual
organization is the most likely of the three spare groups to be either faulty or not
tolerated alongside the installed parts.

**Confidence is low.** This is a guess among three roughly equal candidates, recorded so
that being wrong costs something.

## What each outcome earns

- **Fails** → the responsible subset is within those four `2DRx4` modules. It does NOT
  establish that any individual one is faulty, nor that they are faulty rather than
  *untolerated in combination* with the installed parts.
- **Trains** → those four are compatible and functional in this population. The responsible
  subset lies among the 4 × 2Rx4, the 1 × 2Rx8, or the 6 × 1Rx4 the operator believes good —
  and the prediction above is falsified.

## Standing disjunction, carried forward from the prior round

The falsified prediction of 2026-09-11 established only that **population shape alone is not
sufficient**. It did NOT establish that supported shape is necessary, and it did NOT
establish that a module is faulty rather than a combination being untolerated. Both branches
remain open, and no round of this bisection may name "a bad module" until one is isolated
AND reproduces the failure alone.
