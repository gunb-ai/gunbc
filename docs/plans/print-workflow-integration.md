# Print requests: preparation, bed-clear, start and completion

A print is a **request converged over passes**, not one operation. `fleet-converge` with
`printer_request` `{"operation":"converge","request_id":…,"root":…,"recipe":…}` runs
`gunbc.fleet.printer_workflow_run` `run`, which calls `gunbc.fleet.print_converge`
`converge_print`. Each pass derives the request's standing from evidence it reads
(`gunbc.fleet.print_request` `PrintEvidence` → `PrintStanding` → `PrintAction`) and takes one action:

1. **No request record → prepare.** The root's committed plate plan
   (`artifacts/printing/<root>/plate.json`), the named print recipe
   (`product.printing.print_recipe`) and the source revision are prepared by
   `product.printed_chassis.print_workflow` `prepare_request_plate`. The pass writes the
   request record (`gunbc.print-request-record.v2`: request, source revision, artifact
   location, artifact sha256, attempt id) create-new and returns. It needs no credential and
   never contacts the printer.
2. **Recorded, unclaimed, printer ready → bed-clear, then start once.** The recorded artifact is
   copied and re-hashed against its recorded digest. The signed ntfy approval
   (`gunbc.fleet.printer_operator_approval`) is the human bed-clear acknowledgement; its subject
   binds the attempt, the artifact digest, the credential version resolved from the printer
   registry, and the observed printer-state fingerprint. `gunbc.fleet.printer_actuation`
   `start_authorized` is the only start authority: it re-reads the live report, re-derives the
   subject, writes the **attempt-keyed** start claim, uploads under an attempt-derived remote
   name, re-checks the state, and starts once.
3. **Claimed → observe the job.** Running is observed; a paused, failed or unidentifiable job
   stops for a person and is never resent. A finished job (`gcode_file`, else `subtask_name`,
   equal to the attempt's remote name, `FINISH`) is recorded once (`gunbc.print-completion.v2`)
   with the **same report that derived the standing**; a later pass accepts that record only if
   its identity fields match and its retained report still decodes as this job finished.
   Anything else is `CompletionRecordMismatched` and stops.

One request is one physical copy: a second copy, or a replacement, is a new request id.

Preparation recomputes on every request; there is no prepared-project store (DESIGN §2,
materialization ladder: no reuse obligation is declared). The Orca AppImage is acquired by
digest (`product.printing.slicer_acquisition`) and the recipe's machine, filament and process
profiles are flattened from that archive (`product.printed_chassis.print_preparation`
`stage_recipe_profiles`); no path or profile is caller-supplied.

**Physical limit of the bed-clear fingerprint.** The printer-state fingerprint is a state
digest, not a monotonic generation: it cannot see an ABA cycle back to an identical report (for
example, manually reprinting the identical last file during the approval window). Until the
printer exposes a stronger generation identifier, nobody operates the printer manually while an
approval is pending.

## Plate plans

A request plate plan is placements and support policy only: named component placements in
integer micrometres, orthogonal rotations in degrees, and explicit support-free parts. It has no
printer, credential version or attempt; the attempt is derived from request, artifact digest and
printer (`gunbc.fleet.print_request` `print_attempt_id`), and a plate plan that authors an
`attempt` is refused. `root` is the wire spelling of a
`product.printed_chassis.printable_assembly` `PrintedAssemblyRoot`; an unknown spelling refuses.
`product.printed_chassis.printable_assembly_selection` `select_printable_assembly` is the one
place a root becomes parts, keep-outs and assembly membership.

The older multi-plate queue entry (`print_workflow` `run`, plans with printer rows and per-plate
attempts) still prepares only and writes `prepared-queues.json`; it starts nothing.

## Required preparation stages

1. Decode the entire request, refusing unknown printers, duplicate attempts/placements,
   malformed fields, unsupported rotations, and nonexistent support-free members.
2. Select the root's assembly once, from the same producer as its review renderer.
   Execute the CadQuery adapter: source checks, valid connected printed solids,
   STL and STEP round trips, bed fit, and assembled printed-part intersections.
3. Check the root's declared `required_keepouts` against printed material. The
   residential cassette declares its rear-socket hardware allowances; the A16 duct
   declares its 8-pin power keep-out. The prepared output binds the root by the
   manifest digest, read back from one read of `cad/assembly.json`.
4. Snapshot the three slicer profiles. Read the checked per-part STEPs, verify their
   digests, apply the requested rigid orientations, check the packed solids, and emit a
   3MF with a separate named object for each requested instance.
5. Execute the pinned Orca private-copy route in a separate scratch directory for each plate. Orca currently auto-arranges the declared
   plate membership; supplied XY positions are a checked candidate, not a promise of
   final placement. An extra emitted plate is refused, not silently ignored.
6. Read back ZIP integrity, G-code MD5, exactly one plate, exact named membership, skipped
   flags, unit scale and rigid transforms, current PLA/process settings, AMS mapping,
   reviewed warnings, deposited XYZ bounds, and support-free members. The G-code reader
   handles leading-dot numbers and full-circle I/J arcs; unsupported radius arcs refuse.
7. Persist project/source/CAD/STEP/profile/tool/reader digests, realized placements,
   durations, mass, support fraction and per-feature/per-object extrusion readings.
   A preparation failure prevents the whole request from reaching approval.

The current process policy is the unpowered PLA prototype: A1 mini/0.4 mm, 220 °C nozzle,
65 °C textured bed, four walls, 20% infill, by-layer printing, AMS slot 1. Geometric
admission does not qualify loaded joints, creep, powered operation, or thermal behavior.

## Starting a print

See the request passes above: `gunbc.fleet.printer_actuation` `start_authorized` is the only
boundary that uploads to or starts a printer, holding the fleet controller hold and the
attempt-keyed claim under `/var/lib/gunbc/printer-starts`. No lost acknowledgement causes a resend.

## Validation

`test.claim.printer_batch_workflow_validation.validate_preparation` first checks real approval-request construction (including SHA-256, input binding and the durable claim path), executes the archive/G-code positive and negative controls, and calls
the production entrypoint. It cannot start a printer. The controls
include extra plates, wrong membership, scaled objects, bad checksums, out-of-bed XY/Z,
forbidden support, leading-dot motion words and full-circle arcs.

The recorded shadow plan produces two lower-post plates, approximately 4 h 05 min and
85 g each. The live printer/approval/controller route has not been exercised by this
change. Existing running prints were left alone. See the adjacent validation receipts
for the exact exercised source and artifact digests; CI is a separate gate.

Boundary tests run individually through the witness runner. The original aggregate
wrapper was rejected by the current compiler because tests cannot call other tests;
it was removed without removing any of the individual boundary tests.

The fast floor checks canonical approval serialization and claim-path ownership at
those interfaces. The real cryptographic construction is retained in
`printer_batch_workflow_validation.validate_approval_identity`, also consumed by
`validate_preparation`; it is integration validation, not a fast-floor verdict.
Plan admission rules each have their own counterexample and witness. This keeps
all rejection cases while avoiding one cumulative budget for unrelated cases.

## Start authority

There is one start path. Dispatch input validation
is not authorization: the common path verifies an unexpired ntfy grant, observes an explicit
IDLE or FINISH report with zero or the vendor-confirmed spurious 0500C011 code, and acquires a durable claim keyed by the print attempt before any
upload/start. Active/paused/unknown states, partial reports and other error codes refuse. Approval text requires the operator to
check the actual bed when approving. A second readiness observation and approval-expiry check occur
immediately before publishing the start.

Claims live in `/var/lib/gunbc/printer-starts` on the separately modeled printer LAN host. The
host check precedes the gate. The service account needs an owner-controlled, persistent directory
there before this path can operate; missing/unwritable storage refuses. Do not delete a claim to
retry an uncertain start.

One exact-version credential session covers preflight, upload, post-upload observation and start.
It is evidence of an existing-secret read, never PrinterCredentialProvisioned. Receipt source is
projected from the selected token source. The observed setup report is informational only.

The census distinguishes separately initiated operator requests from unattended provisioning.
Physical print starts select operator approval: the credential is read through the printer
federation (`gunbc.fleet.printer_federation`, pinned to `fleet-converge` at `main` in the
`printer-lan` environment) and the start is authorized only by the signed ntfy grant. No
operator-token or token-file entry exists; the token-file `mode=print` and the batch controller
were deleted (#14945).

Vendor-readiness correction (2026-10-04): Bambu Studio treats FINISH as a completed
job eligible for another print, not a cooling state. The bed may still hold a part in
either IDLE or FINISH, so fresh job-bound bed-clear approval remains mandatory.
The exact 0500C011 code is vendor-confirmed non-existent (BambuStudio issue 4495,
comment 2275068828); no broader nonzero-error exemption is introduced. Tests retain
refusal of the actual prior SD fault 83902511, adjacent codes, active/paused states,
missing/duplicate fields and string-typed errors. The durable claim and post-upload
readiness/approval-expiry checks remain unchanged.

### Automatic post-start reconciliation

The start path enters a read-only reconciliation after an acknowledged or uncertain
publish. The publisher is bounded by GNU timeout at 20 seconds
with a 2-second kill grace; every nonzero outcome remains uncertain, never permission to
republish. The cause of the observed connection drops is not established or claimed fixed.

Reconciliation reuses the credential session and makes at most eight fresh single-response
MQTT observations, each with the existing 15-second wait, separated by 5 seconds. It retains
each raw response (including incomplete responses) beside the attempt's durable start claim.
A report confirms running only if it contains the exact attempt-derived `gcode_file`,
`gcode_state: RUNNING`, and a nonblocking numeric `print_error` in the same JSON object.
No display-name fallback, cached report, merged delta, FINISH or unknown state can confirm.
The same project's pause/failure/blocking error stops reconciliation; wrong-file and partial
reports can consume the observation budget but never trigger another start.

The receipt records running-confirmed, expected-project-blocked or uncertain, the publisher
outcome, cleanup problems and the retained evidence path. Only confirmed running with no
cleanup failure returns success. Failure/uncertainty retains the claim and forbids replay.
The MQTT wait budget excludes CA acquisition and local filesystem runtime. This change
handles acknowledgement loss without manual telemetry checks; it does not promise that the
LAN or firmware cannot fail or that MQTT provides application-level exactly-once printing.

## Functional geometry, support geometry and print stability

The printer-02 incident on 2026-10-05 is recorded in `product.printing.failure_incidents`. Its upright M140/G140 rotations are now negative controls. Packing must preserve the modeled export plane; in-plane rotation is allowed. Off-plane orientation currently refuses because no build-stage qualification producer exists. This rule applies to every part, independently of its ID.

The preparation chain is:

1. Check functional CAD and export each part in its modeled print plane.
2. Admit placements that preserve that plane, then check transformed solids and bed fit.
3. Generate sacrificial supports using the pinned Orca generic automatic support algorithm, governed by `product.printing.process_geometry.prototype_support_policy`.
4. Read the actual archive settings back against the same policy. Extract generated support and support-interface motion into `<project>.supports.json`, independently of functional geometry, with per-object attribution when the slicer supplies it. Preserve unattributed paths as `outside-object`.
5. Bind that artifact to the project digest and record its SHA-256 in the readback. Record deposited XYZ bounds per object, including arc extrema. These describe commanded centerlines, not measured plastic or solid support volumes.
6. Only after preparation succeeds, request printer approval and use the existing exclusive controller, durable start claim, independent start reconciliation and monitoring.

### Remaining qualification work

Preserving a print plane does not prove adhesion or stiffness. A future generic build-stage assessment needs layer geometry, contact footprints, unsupported islands, growing-member slenderness, material/process properties, acceleration and nozzle-clearance evidence. Overhang support and lateral stabilization are separate obligations. No arbitrary height threshold is presented as a validated physical limit. Current artifacts provide evidence for this work, not an admission that it is solved.

A printer reporting completion still requires physical inspection. The current state/error telemetry cannot reliably detect spaghetti. Failure records retain observations separately from causal hypotheses and do not infer successful output from error code zero.

### Retraction, travel and cooling evidence

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

### Recovery plate plan

`artifacts/printing/fan-recovery-2026-10-05/plan.json` assigns only M140 to printer-01 and G140 to printer-02, each flat on its own plate. The four reported intact printer-01 pieces are not duplicated. The generated readbacks provide actual height, material, duration and support use before these plates reach approval.

### Declared runtime dependencies

The direct-host workflow now checks the Ubuntu 24.04 provider before CAD work. Orca's declared host-package roots include WebKitGTK 4.1 and JavaScriptCoreGTK 4.1, OpenGL, GLU and EGL. The OCP package roots supply GL and X11. Package installation is observed through dpkg's decoded status; CadQuery/OCP imports and exact Python distribution versions are checked; the digest-pinned Orca artifact must actually run its help command. Receipts retain those observations. Installed packages alone are not a runtime-success claim.

`product.printed_chassis.print_workflow.converge_runtime(receipt: ...)` is the explicit provisioner for the modeled printer LAN host. It uses the existing package observe/install/readback mechanism and the same requirement population as preparation. An unsupported OS or unreadable package observation refuses. Print preparation observes these dependencies and does not install packages implicitly.

The print runtime is one observed value. `product.printing.print_runtime`
`observe_print_runtime` checks the OS, packages, host architecture (`uname -m`), the CAD
interpreter and the pinned Orca artifact, writes one `runtime.json` receipt, and mints
`PrintRuntimeReady`. The CAD interpreter is not PATH `python3`: it is
`/var/lib/gunbc/print-runtime/cad/<spec sha256>/bin/python`, named by
`product.printing.cad_runtime_spec` (CPython tag, wheel platform, pinned distributions), and
it must answer the probe from that executable. The CAD check, plate packing and readback are
launched from that value only, and each plate readback records the runtime receipt's
sha256. Slicer profiles are snapshotted once per invocation under `<output>/inputs`.

`print_workflow.converge_runtime` (and so `provision_and_run`) converges that environment
after the host packages (`product.printing.cad_runtime_converge`): it acquires the pinned
micromamba by digest (`gunbc.url_file_acquisition`), takes an exclusive lock directory,
solves the pinned Python and pip from conda-forge into the digest-named prefix, installs the
pinned distributions with that prefix's own `python -m pip`, probes that interpreter, and
only then writes the ready marker that publishes it. A second run is a no-op. A prefix that
is unpublished or no longer matches is refused and left untouched; remove it to rebuild.
The transitive packages below the pins are not locked, so the result is an observed matching
environment, not a reproducible one. `/var/lib/gunbc/print-runtime` must be writable by the
workflow's user.

On 2026-10-05 a manual host-package install was mistakenly initiated before this declaration existed. It completed before the operator correction. The original failed preparation records remain under `/home/briansrls/print-run-2026-10-05/fan-recovery/prepared` on srv1. This is recorded as a process error; the later modeled readback does not retroactively make that earlier install modeled. Future provisioning goes through the declaration and convergence entry.

CadQuery 2.8.0's wheel metadata requires `cadquery-ocp>=7.9.3.1,<8.0`. The earlier no-VTK choice contradicted both that dependency and the actual working environment; the modeled pip realization now selects the pinned `cadquery-ocp` distribution. A no-VTK realization would need its own dependency-consistent resolution.

The ELF inspection after the first modeled readback found two further missing SONAMEs: `libSM.so.6` and `libICE.so.6`. Their providers, `libsm6` and `libice6`, are now declared too; the inspection is retained in the recovery artifacts. `print_workflow.provision_and_run` composes explicit package convergence with normal preparation and print approval in one invocation on the modeled LAN host. It validates the plan before convergence and stops on a convergence refusal.
