# Functional geometry, support geometry and print stability

The printer-02 incident on 2026-10-05 is recorded in `product.printing.failure_incidents`. Its upright M140/G140 rotations are now negative controls. Packing must preserve the modeled export plane; in-plane rotation is allowed. Off-plane orientation currently refuses because no build-stage qualification producer exists. This rule applies to every part, independently of its ID.

The preparation chain is:

1. Check functional CAD and export each part in its modeled print plane.
2. Admit placements that preserve that plane, then check transformed solids and bed fit.
3. Generate sacrificial supports using the pinned Orca generic automatic support algorithm, governed by `product.printing.process_geometry.prototype_support_policy`.
4. Read the actual archive settings back against the same policy. Extract generated support and support-interface motion into `<project>.supports.json`, independently of functional geometry, with per-object attribution when the slicer supplies it. Preserve unattributed paths as `outside-object`.
5. Bind that artifact to the project digest and record its SHA-256 in the readback. Record deposited XYZ bounds per object, including arc extrema. These describe commanded centerlines, not measured plastic or solid support volumes.
6. Only after preparation succeeds, request printer approval and use the existing exclusive controller, durable start claim, independent start reconciliation and monitoring.

## Remaining qualification work

Preserving a print plane does not prove adhesion or stiffness. A future generic build-stage assessment needs layer geometry, contact footprints, unsupported islands, growing-member slenderness, material/process properties, acceleration and nozzle-clearance evidence. Overhang support and lateral stabilization are separate obligations. No arbitrary height threshold is presented as a validated physical limit. Current artifacts provide evidence for this work, not an admission that it is solved.

A printer reporting completion still requires physical inspection. The current state/error telemetry cannot reliably detect spaghetti. Failure records retain observations separately from causal hypotheses and do not infer successful output from error code zero.

## Recovery plate plan

`artifacts/printing/fan-recovery-2026-10-05/plan.json` assigns only M140 to printer-01 and G140 to printer-02, each flat on its own plate. The four reported intact printer-01 pieces are not duplicated. The generated readbacks provide actual height, material, duration and support use before these plates reach approval.
