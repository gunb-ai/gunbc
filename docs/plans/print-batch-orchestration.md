# Desired-set printing and automatic plate packing

Requested after printer-01 completed the E1/E2 plate on 4 October 2026. This is the
workflow milestone. The explicit-plate preparation/controller foundation now has a
[modeled entrypoint](print-workflow-integration.md); the full desired-set scheduler and
release contract below remain incomplete. This document grants no print start.
The operator should request parts and quantities once, then clear beds when notified.

## Assembly convergence source

For cassette work, the desired multiset is derived from the selected assembly and
its manufacturable requirements in the [assembly convergence plan](cassette-assembly-requirements.md).
BATCH-1 must consume that source rather than independently authoring a second
cassette list. Standalone print requests remain explicit selected goals.
Assessment uses `std.goal_assessment`; unknown inventory or pending inspection
creates evidence/reconciliation work, not a replacement print. The planner still
needs durable reservations and the guarded start protocol: deriving a diff alone
does not make physical actuation idempotent. BATCH-5 inspection receipts feed back
into assembly assessment; FINISH never supplies physical acceptance.

## Contract

Input is a desired multiset of versioned part instances, compatible available printers,
and an admitted process/material policy. Output is an explainable packed plate plan,
per-printer schedule, remaining quantities and durable execution/handling receipts.
Rerunning the same desired set reconciles existing work instead of printing duplicates.

Use assembly part identities and content digests from the existing CAD/export path,
`manufacturing_manifest` for physical attribution, the fleet's printer identities,
and the reviewed shared start/approval/claim/reconciliation route from PR #13149.
Do not introduce another publisher or another printer inventory. `bed_partition` divides
an oversized design into manufacturable pieces; packing places those resulting pieces
on plates. Both consume the same bed/process allowances but answer different questions.

## Packing and scheduling

1. Expand quantities into stable required-instance slots, each bound to geometry revision,
   material/process and intended orientation. Reserve slots already assigned to an active,
   claimed, uncertain or completed-but-not-yet-inspected print. Accepted usable inventory
   satisfies matching slots. Rejecting a failed part creates an explicit replacement need.
2. Partition only by genuinely incompatible printer, nozzle, material and process constraints.
   Combine different parts and requested projects when those constraints and handling allow it.
3. Pack the remaining instance footprints deterministically. Evaluate allowed in-plane rotations;
   changing the load-bearing print orientation requires separate admission. Include supports,
   brim, object gaps, bed margins and printer exclusion zones. A whole-plate, layer-by-layer print
   avoids introducing sequential-print toolhead swept-volume assumptions.
4. Minimize required bed-clear cycles first, then use sliced duration to balance the two printers
   and respect assembly priorities. Do not make nominal area utilization the sole objective:
   a dense fragile plate that fails or mixes incompatible processes is worse.
5. Start with a deterministic rectangle packer over conservative oriented envelopes, using several
   orderings and scoring the resulting plans. Later contour nesting can improve density. Label
   the algorithm and packing quality; do not claim a globally optimal solution from a heuristic.
6. Slice candidates and independently read back complete deposited-toolpath bounds and separation,
   machine/nozzle, temperatures, material mapping, scale, supports, duration and material usage.
   If generated supports/brims exceed the reserved footprint, repack or split before admission.
   Preserve object/instance attribution rather than flattening unrelated parts into an anonymous STL.
7. Keep both queues prepared ahead. Reschedule only unclaimed work when a printer becomes free;
   never mutate the membership or digest of an approved/claimed/running plate.

The recent arms and crossmember plates illustrate the opportunity: their modeled envelopes
could be arranged as a 141.72 × 90 mm beam group and a rotated 112 × 50 mm arm group, with
10 mm between groups, giving approximately 141.72 × 150 mm before outer brim. This is an
unsliced packing candidate, not proof of admitted support/adhesion or a request to reprint them.

## Durable execution and physical handling

The batch reconciler owns stable required-instance slots. A plate attempt atomically reserves
its member slots across both printers before invoking the existing approval/start boundary.
A shared persistent lock/transaction prevents two schedulers from claiming the same instances.
Store plate membership, geometry/process/3MF digests, placement, printer, approval identity,
start claim and observations durably. A caller-selected output directory cannot reset identity.

Separate batch demand, plan revision, plate attempt and physical print instance. Plan IDs may
change when packing improves; demand-slot reservations and completed inventory must not reset.
A new plate attempt cannot reclaim an unresolved old attempt's slots merely by using a new ID.

Lifecycle: planned -> prepared -> awaiting bed-clear/approval -> claimed -> running-confirmed
-> completed-awaiting-removal/inspection -> accepted inventory. Upload/start uncertainty enters
reconciliation-required, retaining the claim and slot reservations. Explicit pause, failure,
partial failure and rejection have named outcomes; no automatic republish follows a timeout.
Completion alone neither proves bed clearance nor proves all objects are usable. Partial failure
can accept intact objects and replace only rejected instances, with an explicit operator record.

Use the existing fresh bed-clear ntfy approval as the operator's combined clear-and-continue
interaction for the next immutable plate. Notify completion and present the next packed plate
there. A1 minis still need physical part removal; this design does not assume automatic unloading
or infer a clear bed from FINISH, IDLE, a timer or an old confirmation. Approving a whole batch
must not silently authorize future uncleared beds.

## Implementation tasks

| Task | Deliverable | Acceptance |
| --- | --- | --- |
| BATCH-1 | Versioned desired-set and inventory reconciliation in DAG | Same request/restart/plan revision cannot duplicate accepted, active, claimed or uncertain slots; changed geometry/material cannot consume incompatible inventory. |
| BATCH-2 | Deterministic multi-object plate packer and review projection | Every required unreserved instance placed exactly once or explicitly refused; rotations, overlap, exclusions, incompatible processes and brim/support boundaries tested; report unused area and algorithm limits. |
| BATCH-3 | Multi-object CAD/slicer realization and readback | Preserve per-instance IDs and unit scale; fail/repack when actual toolpaths exceed a plate's admitted envelopes; sliced time/mass feed scheduling. |
| BATCH-4 | Durable two-printer batch controller over the existing guarded start route | Concurrent schedulers reserve once; crash recovery tested before/after upload, claim, publish and completion; lost ACKs reconcile exact filenames without replay. |
| BATCH-5 | Completion, partial acceptance and bed-clear continuation | One clear-and-continue interaction per plate; distinguish finished from usable/removed; replace only rejected items; no start from stale bed-clear evidence. |
| BATCH-6 | Shadow plan and one authorized packed live plate | Compare full remaining cassette plan with manual baseline, record plate count/handling visits/estimated duration, then exercise recovery without issuing duplicate physical commands. |

Deliver BATCH-1/2 as pure planning first; BATCH-3 produces reviewable slices; BATCH-4/5 are the
execution integration; BATCH-6 is the release gate. Do not label the feature implemented until
all six are complete. No extra gauges or speculative duplicate parts should fill a plate.

The subsequent socket-cleanup incident adds a process objective: compare structural
orientation candidates using sliced support mass, total material, duration, trapped
support accessibility and handling risk. Do not reduce infill or rotate loaded joints
solely to maximize nominal bed occupancy. Prefer longer useful multi-object plates
when compatible, as explicitly requested by the operator. Damaged P1/P2 become explicit
replacement demands; retain all other completed/active instance reservations.
