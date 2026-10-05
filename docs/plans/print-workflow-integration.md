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

`test.claim.printer_batch_workflow_witness_test.validate_preparation` executes the archive/G-code positive and negative controls and calls
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
