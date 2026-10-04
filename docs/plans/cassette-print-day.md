# First complete cassette: two-printer day plan

Planning source: cassette r06, source commit 389bb0588ee62f8f9de4276a6d72d36c23fe33c3.
Both A1 mini printers are available all day, per the operator. Use the previously confirmed
PLA in AMS slot 1 on each. Availability is not fresh bed-clear or idle evidence for a start.
This is a queue for one complete unpowered assembly, not a dispatch or print-admission receipt.
Keep the existing approval, fresh idle observation and durable start claim for every job.

## Historical r06 queue

Waves 3–5 below are superseded by the 140 mm continuation at the end of this document.
Do not dispatch the old three-80-mm-fan parts for the residential assembly.

| Wave | Printer-01 | Printer-02 | Assembly work after removal |
| --- | --- | --- | --- |
| 1 | P1: rear tray half | P2: other rear tray half | Check the real keyed seam and clamp access; retain both parts for the cassette. |
| 2 | P3: front tray half | P4: other front tray half | Join the four panels, check rails and handle; fit four M3 clamps when lengths are established. |
| 3 | M1: outer rear fan frame | M3: opposite outer rear fan frame | Clean supports from keys and airflow guides; inspect the mating surfaces. |
| 4 | M2 plus E1 and E2: center fan frame and both support arms | G1 plus K1–K5: first guard and all five retainers | Assemble the rear bank and slide the arms into the tray; check retention. |
| 5 | G2: second guard | G3: third guard | Fit three fans and guards with six M4 through-bolts once length/engagement is checked. |

Ten provisional plates cover all 17 instances exactly once. P1–P4 and M1–M3 are distinct;
E1/E2 share geometry, G1/G2/G3 share geometry, K1/K2 share geometry, and K3/K4 share geometry.
Do not substitute K5 for the other pins. No extra gauges or speculative spare sets are queued.

These are priorities, not synchronization barriers: advance either printer as soon as its bed is
cleared and its next admitted file is ready. If the other printer is still running, it need not
finish before this printer advances. Keep the remaining day as reprint/fit-correction capacity.
The real P1/P2 seam is the first assembly checkpoint; a failed seam pauses related tray parts for
correction, while independent fan/guard preparation can continue.

## Plate preparation

All placements below are candidates until checked in the slicer, including support and brim extent.
The modeled bed is 180 × 180 × 180 mm. Exported STLs already have their modeled print orientation;
import at 100% scale and do not apply the same orientation transform twice.

- P1/P2/P3/P4 each get a dedicated plate. Their modeled oriented footprints are approximately
  162.22 × 167.15, 142.22 × 167.15, 161.72 × 174.15, and 141.72 × 174.15 mm.
  P3/P4 plus a 2 mm brim occupy 178.15 mm along the long axis. The old 5 mm brim does not fit.
  Review the integral underside sockets and half-lap overhangs; support placement cannot be
  inherited from the gauge preset, which explicitly disables supports.
- M1/M3 each get a dedicated plate: approximately 112 × 122 mm, 105 mm tall in the modeled
  XZ print orientation. Review the fan-opening bridge, guides and keyed foot, with removable
  support that does not trap material in a mating socket. Allow support spread before admission.
- M2 is approximately 92 × 40 mm, 107 mm tall in its modeled orientation. Each arm is
  20 × 112 mm, 8 mm tall. A candidate arrangement places the two arms beside M2 with 6 mm
  object gaps; its unbrimmed bounding rectangle is approximately 144 × 112 mm.
  Split this plate if support extent or stability requires it; that adds one short job.
- G1/G2/G3 have 92 × 92 mm footprints and are 19 mm tall. Do not assume two guards fit on one
  180 mm plate: two axis-aligned footprints already span 184 mm before gaps or brims.
- Put the five retainers alongside G1, with sufficient separation for brims. Evaluate printing
  retainers grip-down so the broad head anchors the part, rather than balancing on a 4.1 mm shaft.
  This is an explicit candidate orientation change requiring slice preview, not an already
  verified alternative. If grouping compromises support or adhesion, split the retainers out.

Use the pinned Orca workflow and the existing A1 mini/material profiles. Establish a cassette
process profile explicitly: wall count, infill, top/bottom shells, supports and brim are still
preparation decisions. A successful slicer exit alone is insufficient. Read back the resulting
project/G-code for machine, nozzle, material/temperatures, AMS mapping, scale, bounds, supports,
time and material usage. Do not dispatch a gauge-profile slice of these parts by default.

## Timing and operator visits

The present evidence contains no sliced cassette project, so there is no defensible numerical
completion time or filament total yet. Two available printers do not establish that all ten
plates finish within one day. After slicing, assign longer jobs to the less-loaded printer
within the above assembly priorities, and publish each printer's cumulative ETA and filament
requirement. Add cooling, removal, support cleanup and bed-reset time between jobs; those are
operator-dependent and not included in slicer estimates.

Plan for five removal/reload rounds if the two queues stay roughly aligned; independently
finishing jobs can require up to ten visits. Leave a contingency window for a failed print or
joinery correction instead of filling the last available hour with optional duplicates.

## Ready-to-start boundary

The complete model and all 17 solid exports exist; source witnesses and CAD readback pass.
Current source-floor CI passed. The print queue itself has not been sliced or admitted.
Before the first dispatch: finish orientation/support review, generate and inspect the sliced
projects, complete the modeled review-to-print admission, and use the approved printer workflow.
Merging the CAD PR alone does not perform those steps or merge the separate printing/auth PRs.

Today's physical objective is one assembled, unpowered cassette. Board standoff thread/height,
underside clearances, fastener engagement, loaded-joint behavior and thermal suitability remain
unresolved. The fixed rack, shared PSU hardware, power distribution and board-support hardware
are outside this 17-piece printed set. Record actual assembly time and troublesome joints on
this first assembly so the next revision can reduce handling and fasteners further.

## Wave 1 preparation receipt

P1 and P2 were sliced through the digest-verified Orca 2.4.2 DAG route, using the
existing A1 mini 0.4 mm / textured PEI / PLA profiles and explicit cassette process
settings now in `slice_run.cassette_settings_members`. Four walls, 20% gyroid, five
top/bottom layers, 0.20 mm layer setting, 2 mm outer brim, normal automatic supports.
Readback confirms 220 C nozzle / 65 C textured bed, filament index 0 for AMS slot 1,
25 mm total model height, support use, proper unit-scale in-plane placement and in-bed
footprints. Orca's temperature advisory is retained (the existing bed profile is 65 C);
traditional timelapse warnings are retained and the start route does not request it.
Malformed `first_layer_time` metadata is not used for timing.

| Part | Estimated runtime | Estimated PLA | Project SHA-256 |
| --- | --- | --- | --- |
| P1 | 4 h 51 m 40 s | 156.51 g | f34eb44a56d7de5e5c1bb7e6617aac367aa4ec0bd53c33f7d75ac67786bf0720 |
| P2 | 4 h 35 m 50 s | 147.82 g | f10363214c47ae527d7e3055416868c011efe642fcba015e1beb492db1ba5cfc |

These are slicer estimates, not observed finish times. The first wave alone is almost
five hours; the complete ten-plate set is not promised for this day. Output files and
readback are in `/home/briansrls/print-prep-2026-10-04/cassette-wave-1` on srv2, staged
to `/home/briansrls/print-run-2026-10-04/cassette-wave-1` on srv1.
The live start uses reviewed printer source `6da087d2e30f789c59adf11287fe179fc41b0276`,
with a private persistent `/var/lib/gunbc/printer-starts` directory provisioned on srv1.
Physical start/finish standing must come from the corresponding workflow receipt and telemetry.

The operator launch environment also needs the workflow's three key-path bindings:
`GUNBC_APPROVAL_SUBMISSION_MAC_KEY_FILE=/etc/gunbc-roadmap/approval-submission-mac-key`,
`GUNBC_APPROVAL_CAPABILITY_MAC_KEY_FILE=/etc/gunbc-roadmap/approval-mac-key`, and
`GUNBC_APPROVAL_STORE_RECEIPT_MAC_KEY_FILE=/etc/gunbc-roadmap/approval-store-receipt-mac-key`.
Set `RUNNER_TEMP` to a mode-700 per-printer temporary directory for the signed request body.
Check those paths and the private operator token file before launch. Missing bindings caused
pre-filing refusals on the initial wave-1 attempts; neither had created a start claim or
contacted a printer to upload/start. Do not label a pre-filing attempt as awaiting approval.

Live wave-1 update: printer-02 was observed RUNNING the exact P2 digest at 1%, 270 min
remaining, with print_error 0. Its MQTT publisher lost the connection after dispatch; the
physical start was reconciled by the report, not replayed, and its durable claim is retained.
Printer-01 was withheld before upload on 83935249 / 0500C011. Bambu Studio's shipped table
has an empty description, and Bambu collaborator walterwongbbl identifies it as a non-existent
error in https://github.com/bambulab/BambuStudio/issues/4495#issuecomment-2275068828.
Printer source 54bd96e7e9 classifies that exact code, accepts IDLE/FINISH job readiness, and
retains fresh bed-clear approval, other-error refusals and no-replay claims. All six readiness/
approval witnesses passed. The P1 retry uses a new approval attempt under this revision.
FINISH is not a cooling state or proof of a part still being present: Bambu Studio's can_print
accepts IDLE and FINISH. Asking for a reboot solely to leave FINISH was unnecessary.

## P4 preparation — 4 October 2026

The operator reports P2 finished and printer-02's bed cleared, and authorizes the next segment. P4 is the next tray piece on that printer. It was sliced with the same pinned Orca workflow and machine/material profile. The initial 2 mm brim/support footprint exceeded the bed; removing the outer brim alone was insufficient. The accepted profile retains normal supports and sets `raft_first_layer_expansion=0` (this setting also affects the support base with zero raft layers).

The final project's SHA-256 is `14cebf1762970c67626859d9d5341f23ef322be37cff663a0c48ac08b7f74bab`. Estimated time: 3 h 33 m 42 s; PLA: 105.34 g. The 14 mm height agrees with source STL bounds. Independent deposited-path coordinates span X 0.811–179.190 mm and Y 18.073–160.650 mm; slicer bounding metadata is inconsistent and is not the sole bed-fit evidence. Unit scale, machine/nozzle, temperatures, supports, checksum and AMS index 0 passed readback. The previous thermal/timelapse warnings retain their documented disposition.

Artifacts are under `print-prep-2026-10-04/cassette-wave-2` locally and `print-run-2026-10-04/cassette-wave-2` on srv1. The reviewed source remains `source-readiness` from wave 1; attempt `cassette-r06-wave2-20261004-P4` preserves approval and durable no-replay guards. The approval and fresh readiness checks passed, the durable claim was written, and the project uploaded. The MQTT start publication lost its acknowledgement (exit 7); it was not replayed. The operator confirms printer-02 heating or calibrating P4. Read-only telemetry subsequently confirmed RUNNING, print_error 0, the exact P4 digest filename, 2% and 208 minutes remaining (layer_num 0). Printer-01 is reported finished and cooling; P3 preparation is authorized, but its bed-clear confirmation is still pending.

P3 is also sliced and checked with the same front-tray profile. Project SHA-256: `cd0e5071dd1f61ea99a1817eadf9cf7def2f1f5869f8730b14bbe0073bf1fc67`; estimated 3 h 43 m 47 s / 110.67 g PLA. Independent modal extrusion-path bounds are X 6.565–170.650 mm, Y 1.078–179.190 mm, including line starts/endpoints; no deposited arc moves occur in these slices. A 0.25 mm edge allowance is retained. The operator subsequently confirmed printer-01's bed clear and requested P3 start; attempt `cassette-r06-wave2-20261004-P3` was launched through the same guarded route.

P3 passed approval and fresh readiness, uploaded, and received one start publication. Independent telemetry confirms RUNNING the exact P3 digest at 0%, 223 minutes remaining, layer_num 0. Its print_error remains the vendor-documented non-existent 83935249 code admitted by the reviewed readiness classification; this is not a claim that the raw error field is zero. No start was replayed.

## 140 mm continuation after P3/P4

The operator reports P4 removed from printer-02 and P3 finished on printer-01.
All four tray panels have therefore been reported finished; assembly fit is not yet recorded.
Printer-01 cooling/removal confirmation remains pending. Next jobs are the two original
rear support arms E1/E2 on printer-01 and split crossmember B140L/B140R on printer-02,
from the reviewed residential 140 mm assembly. These supersede the old M1/M3 wave.

Both plates retain the assembly's unit-scale geometry. XZ print orientation puts their
8 mm thickness upright and the retaining holes vertical; the two arms occupy 50 × 112 mm
and the two crossmember halves 141.72 × 90 mm before brim. The arms have 10 mm separation,
as do the crossmember halves. Files are prepared in `cassette-wave-3` under the existing
local preparation root, through the pinned Orca DAG route and the supported cassette PLA
profile (four walls, 20% gyroid, five top/bottom layers, 2 mm brim). These are parts for
the unpowered fit assembly, not qualified structural or thermal components.

The subsequent queue is the M140 cradle, G140 guard, K1/K2/K3/K4/K6/K7 retainers, and
the four lower/upper corner-post pairs with four post retaining pins. Plate grouping,
orientation, material and slice readback for those later parts remain preparation work.
No old 80 mm fan frames or guards should be printed for this variant.

Both next plates passed archive/G-code checksum, unit-scale transform, A1 mini 0.4 mm,
PLA 220 C / textured-bed 65 C, AMS index 0, 8 mm height, and independent deposited-path
bounds checks. The existing 65 C bed-temperature advisory is retained for the unpowered
prototype. Automatic supports are enabled and reported in the slice.

| Printer | Plate | Slicer estimate | PLA | Project SHA-256 |
| --- | --- | --- | --- | --- |
| printer-01 | E1 + E2 | 56 m 27 s | 23.63 g | f1e5bcf8b2eb5574436ccd460f62ca15d43097829deb5163e6b3f30f27d21a3f |
| printer-02 | B140L + B140R | 1 h 52 m 46 s | 52.38 g | acaf7b25a4c49c3412043c2c20ae35e790d6f4ae3c9f0f6fb6ca70d55b7f7244 |

Files and unexecuted launch scripts are staged at
`/home/briansrls/print-run-2026-10-04/cassette-wave-3` on srv1. Printer PR #13149
remains open; the prior private token file is absent. A fresh private token file and
printer-01 removal confirmation have been requested. No approval request, upload or
start has been performed for these two jobs.
