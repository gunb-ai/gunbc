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

## Retraction, travel and cooling evidence

Preparation now preserves the complete `Metadata/project_settings.config` as
`sliced_project_settings` in its receipt. These are the slicer's raw authored keys
and values, including `nil` filament overrides; an absent key stays absent. This
does not invent defaults or implement a second profile-inheritance resolver.
Retraction settings remain profile facts, not newly imposed product overrides.

`product.printing.deposited_geometry.toolpaths` (the generated adapter function)
also emits `process_observation`. Each slicer-marked layer records negative-E
moves, retraction episodes, retracted/recovered filament, retracting XY moves,
nonextruding XY moves and chord distance, deposition runs after travel, deposited
Z bounds, and functional object labels. An episode starts when negative E creates
a previously absent recovery debt; split wipe moves therefore need not be counted
as separate retractions. Recovery debt survives layer boundaries and G92 resets.
G10/G11 commands are counted separately, without assuming firmware retraction
distances. Observation starts at the first `CHANGE_LAYER` marker with zero known
debt; startup extrusion is outside its scope.

The parser shares its coordinate/extrusion state and deposited-arc geometry with
the existing bounds reader. It does not run a second interpretation of G-code.
Its commanded-motion seconds sum path length / modal feed, including deposited
arcs and extrusion-only moves. Unknown feed or unhandled travel-arc lengths are
counted as unestimated moves. Acceleration, firmware limits, waits, dwell, and
synchronization are excluded: this is a comparison proxy, not actual layer time
or a claim that cooling targets were achieved. Measured and slicer-per-layer time
remain explicitly unavailable.

Object labels do not identify individual geometric islands. A deposition run
after travel is evidence of an interruption, not an island count. Positive E
with XY motion follows the existing commanded-deposition convention, including
any recovery combined with motion; it is not a physical extrusion measurement.
No red/green stringing or stability threshold is introduced by these observations.

`product.printed_chassis.print_preparation.observe_process_batch` accepts a JSON
list of archived project paths and a fresh output directory. It uses the same
archive-integrity checks and process observer as preparation, writes reports
named by project SHA-256, and never authorizes printing or overwrites a prior
receipt. This is the retrospective route for comparing failure and recovery jobs.
The integration controls in `test.claim.print_preparation_kernel_validation`
exercise both admission and retrospective output, raw settings preservation,
relative/absolute E, G92, split wipes, cross-layer recovery, modal feed, full-circle
arc timing, unavailable feed, and firmware-command reporting.

## Recovery plate plan

`artifacts/printing/fan-recovery-2026-10-05/plan.json` assigns only M140 to printer-01 and G140 to printer-02, each flat on its own plate. The four reported intact printer-01 pieces are not duplicated. The generated readbacks provide actual height, material, duration and support use before these plates reach approval.

## Declared runtime dependencies

The direct-host workflow now checks the Ubuntu 24.04 provider before CAD work. Orca's declared host-package roots include WebKitGTK 4.1 and JavaScriptCoreGTK 4.1, OpenGL, GLU and EGL. The OCP package roots supply GL and X11. Package installation is observed through dpkg's decoded status; CadQuery/OCP imports and exact Python distribution versions are checked; the digest-pinned Orca artifact must actually run its help command. Receipts retain those observations. Installed packages alone are not a runtime-success claim.

`product.printed_chassis.print_workflow.converge_runtime(receipt: ...)` is the explicit provisioner for the modeled printer LAN host. It uses the existing package observe/install/readback mechanism and the same requirement population as preparation. An unsupported OS or unreadable package observation refuses. Print preparation observes these dependencies and does not install packages implicitly.

On 2026-10-05 a manual host-package install was mistakenly initiated before this declaration existed. It completed before the operator correction. The original failed preparation records remain under `/home/briansrls/print-run-2026-10-05/fan-recovery/prepared` on srv1. This is recorded as a process error; the later modeled readback does not retroactively make that earlier install modeled. Future provisioning goes through the declaration and convergence entry.

CadQuery 2.8.0's wheel metadata requires `cadquery-ocp>=7.9.3.1,<8.0`. The earlier no-VTK choice contradicted both that dependency and the actual working environment; the modeled pip realization now selects the pinned `cadquery-ocp` distribution. A no-VTK realization would need its own dependency-consistent resolution.

The ELF inspection after the first modeled readback found two further missing SONAMEs: `libSM.so.6` and `libICE.so.6`. Their providers, `libsm6` and `libice6`, are now declared too; the inspection is retained in the recovery artifacts. `print_workflow.provision_and_run` composes explicit package convergence with normal preparation and print approval in one invocation on the modeled LAN host. It validates the plan before convergence and stops on a convergence refusal.
