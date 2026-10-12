# Checked preparation and fleet print queues

The entrypoint is `product.printed_chassis.print_workflow.run`. One invocation accepts
multiple printer queues, each containing multiple plate attempts. It owns CAD execution,
named-object realization, pinned Orca execution, sliced-file readback, approval filing,
start publication, acknowledgement reconciliation, and completion observation.

This is the explicit-plate execution foundation of CAS-8. It is **not** the completed
arbitrary desired-set optimizer, accepted-part inventory, or automatic crash recovery
specified in [print-batch-orchestration.md](print-batch-orchestration.md).

## Inputs and execution

[The exercised two-printer plan](../../artifacts/printing/workflow-integration-2026-10-05/prepare-plan.json)
is the input format. Each printer row has its fleet identity, numeric credential version,
and a list of plates. A plate has a stable `attempt`, named component placements in
integer micrometres, orthogonal rotations in degrees, and explicit support-free parts.
Attempt identities are unique across the entire fleet request. Keep those identities
when changing output directories or assigning work to another printer.

The CLI entry takes `plan_file`, `output`, `appimage`, `machine`, `filament`, `process`,
and `mode`. Paths are explicit tool/artifact inputs, not shell fragments.

- `mode=prepare` performs all preparation and writes `prepared-queues.json`; it does
  not acquire credentials, file approvals, contact printers, or start anything.
- `mode=print` performs the same preparation, initializes one pinned credential
  session per printer, acquires the fleet controller hold, then drives all queues.
  It currently uses the existing operator-token route on the modeled printer LAN host.
- Use a fresh absolute output directory. Existing realization/script/project outputs refuse;
  preparation does not silently reuse a possibly stale success file.

The CAD/Python environment and pinned Orca artifact must be available on the execution
host. This change does not provision those tools. The existing short-lived Actions
single-printer dispatch is not a supervisor for this day-long controller. Deploying a
supervised fleet entry and migrating that dispatch remain deployment work; do not run
this controller under that job's 15-minute step timeout.

## Required preparation stages

1. Decode the entire request, refusing unknown printers, duplicate attempts/placements,
   malformed fields, unsupported rotations, and nonexistent support-free members.
2. Derive the residential assembly once from the same producer as the review renderer.
   Execute the existing CadQuery adapter: source checks, valid connected printed solids,
   STL and STEP round trips, bed fit, and assembled printed-part intersections.
3. Check the modeled socket hardware keep-outs against printed material. Their proposed
   10 mm diameter × 6 mm depth allowances consume the socket's hole and placement data;
   they are no longer coordinates authored only in a preparation script.
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

## One fleet controller, independent printer progress

A fleet pass first files **all** ready approval requests, then advances each lane once.
Pending approval on one printer does not create a polling sleep before the other request
is sent. Publication returns to the controller; exact-file reconciliation happens on
later passes instead of holding up the second printer through a retry loop. The fold is
paced once per fleet pass, with a bounded observation budget.

The underlying upload/publish/read operations remain bounded synchronous operations.
This is overlapping printer lifetimes and interleaved observations, not parallel threads
for every network call or simultaneous CPU-heavy slicing.

The controller retains the initialized LAN credentials in memory. It does not fetch
Secret Manager again on each observation or require a fresh GCP token for every plate.
Every physical start still consumes a new job-bound approval and a fresh readiness
observation. FINISH neither accepts parts into inventory nor clears the bed. Completion
queues the next bed-clear approval for that printer; another printer can continue running.
Approval expiry/denial, pause/failure, or repeated unresolved observations place that lane
in attention. Existing 15-minute approval windows are retained.

There are three durable boundaries:

- The existing compare-and-set exclusive hold prevents competing fleet controllers.
  Single-printer starts also acquire it, so an old entrypoint cannot race the batch.
- A stable batch-attempt slot is independent of printer, output directory and sliced
  digest. Reassigning or re-slicing the same attempt cannot admit another physical start.
- The existing approval-derived start claim remains bound to the exact printer, bytes,
  credential version, feed and approval identity. No lost acknowledgement causes a resend.

Per-start receipts and observations live beside their claim under
`/var/lib/gunbc/printer-starts`. The timestamped fleet projection is
`batch-state.json`. The old single-printer receipt filename remains a
compatibility projection; the batch consumes the per-start records instead.

On uncertainty or process death, holds/claims remain. Same-owner text does not prove the
old process is dead and cannot reacquire its hold. Dead-holder recovery and resuming the
remaining inventory are still separate work; do not delete claims or invent new attempt
IDs to get past a refusal. Partial acceptance and replacement demands remain CAS-8 work.

## Validation

`test.claim.printer_batch_workflow_validation.validate_preparation` first checks real approval-request construction (including SHA-256, input binding and the durable claim path), executes the archive/G-code positive and negative controls, and calls
the production entrypoint with `mode=prepare`. It cannot start a printer. The controls
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

## Printer workflow review status

The printer start paths share one approval and claim implementation. Dispatch input validation
is not authorization: the common path verifies an unexpired ntfy grant, observes an explicit
IDLE or FINISH report with zero or the vendor-confirmed spurious 0500C011 code, and acquires a durable claim for the approved escalation ID before any
upload/start. Active/paused/unknown states, partial reports and other error codes refuse. Approval text requires the operator to
check the actual bed when approving. A second readiness observation and approval-expiry check occur
immediately before publishing the start.

Claims live in `/var/lib/gunbc/printer-starts` on the separately modeled printer LAN host. The
host check precedes the gate. The service account needs an owner-controlled, persistent directory
there before this path can operate; missing/unwritable storage refuses. Do not delete a claim to
retry an uncertain start. This review has not provisioned that directory or performed a live print.

One exact-version credential session covers preflight, upload, post-upload observation and start.
It is evidence of an existing-secret read, never PrinterCredentialProvisioned. Receipt source is
projected from the selected token source. The observed setup report is informational only.

The census distinguishes separately initiated operator requests from unattended provisioning.
Physical print starts select operator approval. Current code-enforced approval under a federated
read and the operator-token compatibility entry have explicit, bounded divergences; neither is
presented as a broker-minted printer credential. Token-file input does not establish credential
custody and is not authorization to relay tokens through chat.

Local approval, census and workflow-binding witnesses pass. CI is required and the PR remains
draft. CAD and slicer preparation are split out; approval-client cutover is prerequisite #13218.

Vendor-readiness correction (2026-10-04): Bambu Studio treats FINISH as a completed
job eligible for another print, not a cooling state. The bed may still hold a part in
either IDLE or FINISH, so fresh job-bound bed-clear approval remains mandatory.
The exact 0500C011 code is vendor-confirmed non-existent (BambuStudio issue 4495,
comment 2275068828); no broader nonzero-error exemption is introduced. Tests retain
refusal of the actual prior SD fault 83902511, adjacent codes, active/paused states,
missing/duplicate fields and string-typed errors. The durable claim and post-upload
readiness/approval-expiry checks remain unchanged.

### Automatic post-start reconciliation

Both the Actions and operator routes now enter the same read-only reconciliation after
an acknowledged or uncertain publish. The publisher is bounded by GNU timeout at 20 seconds
with a 2-second kill grace; every nonzero outcome remains uncertain, never permission to
republish. The cause of the observed connection drops is not established or claimed fixed.

Reconciliation reuses the credential session and makes at most eight fresh single-response
MQTT observations, each with the existing 15-second wait, separated by 5 seconds. It retains
each raw response (including incomplete responses) beside the durable escalation claim.
A report confirms running only if it contains the exact digest-derived `gcode_file`,
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

The two wave-three jobs were confirmed through the previous manual report route before this
change: both exact digest filenames RUNNING. No physical start was issued to test this change.

Validation for this change: five reconciliation witnesses, six approval/readiness witnesses,
and 24 publisher/no-replay witnesses passed locally. Both shared entry paths typecheck through
the common approval implementation. Live execution of the new post-start path remains for the
next authorized print; the current jobs used the previous revision.

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

On 2026-10-05 a manual host-package install was mistakenly initiated before this declaration existed. It completed before the operator correction. The original failed preparation records remain under `/home/briansrls/print-run-2026-10-05/fan-recovery/prepared` on srv1. This is recorded as a process error; the later modeled readback does not retroactively make that earlier install modeled. Future provisioning goes through the declaration and convergence entry.

CadQuery 2.8.0's wheel metadata requires `cadquery-ocp>=7.9.3.1,<8.0`. The earlier no-VTK choice contradicted both that dependency and the actual working environment; the modeled pip realization now selects the pinned `cadquery-ocp` distribution. A no-VTK realization would need its own dependency-consistent resolution.

The ELF inspection after the first modeled readback found two further missing SONAMEs: `libSM.so.6` and `libICE.so.6`. Their providers, `libsm6` and `libice6`, are now declared too; the inspection is retained in the recovery artifacts. `print_workflow.provision_and_run` composes explicit package convergence with normal preparation and print approval in one invocation on the modeled LAN host. It validates the plan before convergence and stops on a convergence refusal.
