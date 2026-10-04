# Printed node chassis — program plan

**Current review: 4 October 2026.** The operator requested fewer screws and faster assembly.
Revision 06 uses 17 printed pieces, four tray clamps, six fan/guard bolts and five broad-grip
retaining pins. See [cassette joinery review](cassette-joinery-review.md). Earlier counts below
are dated history. No further gauge sequence or cassette print is scheduled by this review.

**Pinned plan. Progress reconciled 2026-10-03.** Printers are in use; the operator has the
printed standoff gauge in hand. The operator reports that the existing gauge aligns with numbered holes 1 and 3.

## Current progress — 2026-10-03

Evidence: merged PRINT-N commit `b242faf7655` (#10680), current printed-chassis modules,
and the operator's 2026-10-03 report. Historical execution results are not tests rerun today.
This dated status supersedes the pre-arrival implementation narrative below.

- **CAD bootstrap and measured export profile landed.** PRINT-N records CadQuery 2.8.0 running
  through user-owned micromamba/conda-forge and measured tessellation observations in
  `product.printed_chassis.export_profile`. The old kernel blocker is resolved. Full STEP/3MF
  solid-readback acceptance is not established by that fact and remains to be audited.
- **Physical printing reached.** PRINT-N records printer-02 reporting `RUNNING` on the gauge
  project after FTPS delivery and MQTT start through the modeled path. The operator now confirms
  the gauge is in hand. No authenticated printer operation has been performed in this session.
- **Coupon feedback recorded.** `hole_fit_measurement.operator_ladder_reading_2026_09_04`
  records M3 interference / tight / free at 3.00 / 3.10 / 3.20 mm nominal rungs. Printer attribution
  remains absent; this is not universal hole compensation or qualification of both machines.
- **Initial pair fit observed, 2026-10-03.** The operator reports that both gauge holes align
  perfectly with numbered holes **1 and 3**, with the gauge underneath the motherboard and no
  screws installed. The supplied annotated image identifies these as **F and C**. This is an
  operator-reported alignment pass on that pair; it does not establish loaded support, exact
  tolerances, the other holes or the full FIT milestone. One board is sufficient for this check.

### Numbered reference and observed pair

The repo already cited the operator's Drive image in
`extdeps.boards.asrock_rack.asrock_altrad8ud_standoff_figure_authority`. A byte-for-byte local copy
is now [ALTRAD8UD_standoff_grid.png](../extdeps/asrock_rack/ALTRAD8UD_standoff_grid.png), retrieved
2026-10-03 from [the original Drive file](https://drive.google.com/file/d/1z36tmUWAwOwXWy6ASmaPWJNzKoZggBs8/view).
Use the image's numbers in operator instructions. Its annotated decimal coordinates are a visual
reference; they do not replace the standard's exact coordinate authority.

| Image hole | Model location |
|---|---|
| 1 | F |
| 2 | M |
| 3 | C |
| 4 | H |
| 5 | L |
| 6 | B |
| 7 | R |

**Observed:** 1–3 (F–C) aligns simultaneously underneath the board, without fasteners, per operator.
**Next candidate:** 3–4 (C–H). Other prepared checks are 4–5, 5–2, 3–6 and 4–7.
No new photo is required to identify the pair already reported against this reference.

### Prepared follow-on experiments — 2026-10-03

`product.printed_chassis.fit_gauges` selects five additional gauges using the existing standard
location authority, gauge admission and CadQuery emitter. They are spacing experiments, not confirmed
mounting locations or qualified structural parts. Start with C–H after the existing F–C check passes;
the others extend coverage to the remaining inferred holes. Each is a separate part so it can be
identified by its file and labeled before leaving the bed.

| Pair | Question answered |
|---|---|
| C–H | Does the rear-to-middle spacing agree on the C/H/L column? |
| H–L | Does the middle-to-front spacing agree on that column? |
| L–M | Does the front-row spacing agree? |
| C–B | Does the additional rear hole agree? |
| H–R | Does the outboard middle hole agree? |

J remains excluded and S remains unobserved. A connected set of pair distances does not fix every
relative angle; the assembled adjustable fixture still must confirm the full map. Do not install
extra standoffs merely because the standard has a named location.

**Measurements to gather before a mounting fixture:**

- Identify the seven candidate holes (F, C, B, H, R, L, M) on one physical board; record any absent
  or unexpected hole and pair mismatch. No need to disassemble several boards for this step.
- Actual standoff thread, height from support to board underside, and contact-pad/shoulder diameter;
  identify the screws being used and their usable engagement. Do not infer these from the M3 coupon.
- Highest underside protrusion and clearance around each support; note solder joints/components
  near the front overhang where a possible support would touch.
- Board edge dimensions and access around rear I/O, memory latches and power/storage connectors.

**Measurements that can follow, before cassette geometry:** cooler model and total installed height;
PSU model, actual envelope and mounting points; fan model and intended fasteners; intended drives and
expansion cards; loaded node mass; cable connector bodies and the space needed to plug/unplug them;
available deployment width/depth/height. Missing values stay open instead of becoming defaults. The current fleet inventory identifies Dynatron W1 coolers on srv3
and srv4; that does not identify the cooler on the board being checked, and its catalog row supplies
no measured installed envelope.

**Recovered operator setup reports:** Creality PLA, white on printer-01 and black on printer-02,
reported September 4–5; on October 3 the operator says both setups are identical. These reports are
retained in `gunbc.fleet.printer_setup_reports`, with source timestamps and a read-only `main`
projection. The historical AMS report did not name a printer. The operator explicitly confirmed AMS physical
slot 1 on both machines on October 3; that now governs the print command.
Nozzle/plate settings still need reconciliation against retained execution evidence. Recovered files disagree: `print-n/work/gauge_petg.3mf` specifies PETG with
textured PEI; `printn-slice/gauge_container.3mf` specifies PLA with a cool plate. Preset metadata does not supersede the operator’s filament reports. Do not reuse their machine G-code blindly. Final slicing must also check
placement including brim/skirt: the longest narrow gauge nearly spans the 180 mm bed. Keep model
scale at 100%; resolve any placement conflict through explicit orientation/adhesion settings, not
by shrinking a measuring instrument. The existing fan-mount experiment
is optional after the actual fan and fasteners are identified; it does not gate motherboard FIT.

### Preparation results — 2026-10-03

Generated **C–B, C–H, H–L, L–M and H–R** as individual STL and STEP files under
`/home/briansrls/print-prep-2026-10-03`. Generation ran through `fit_gauges.batch` with the
compiler built from this worktree; the generated Python contains the modeled geometry and export
settings. `geometry-readback.json` records hashes and executed results for all five parts:
valid single solids, expected dimensions and cylindrical hole locations/radii, STEP round-trip
agreement, closed manifold meshes, mesh bounds and volume agreement. These checks establish file
conformance for these gauges only, not physical fit or printer/material qualification.

The three `test.claim.fit_gauges_witness_test` checks executed and passed: every candidate fits
the actual modeled printer envelope; none requests J/S; an unknown pair cannot emit a substitute.
`witnesses.log` retains the run. No physical observation has been inferred from those software tests; the 1–3 fit is separately
recorded from the operator above.

**Ready for final slicing, not yet machine-ready G-code:** use C–H as the next candidate after the
existing F–C result. Use the recovered PLA reports; reconcile nozzle, plate and adhesion settings before
slicing. No upload or printer start was performed. Authenticated delivery/start was not revalidated;
the old `printer_credential_migration_run` includes destructive secret migration and is not an
appropriate routine readiness probe. A subsequent print must use an admitted credential path rather
than rerun that migration merely because it previously started a job.

### Requested two-printer batch — 2026-10-03

The operator requests concurrent printing on both printers. Prepare **printer-01: holes 3–4
(C–H)** and **printer-02: holes 4–5 (H–L)**. This checks two additional board spans; because the
parts differ, it is not a printer-to-printer calibration comparison. Source-identical STL copies and
hashes are staged under `/home/briansrls/print-prep-2026-10-03/two-printer-batch/`, one directory per
printer. Material/colour reports are now modeled: Creality PLA, white on printer-01 and black on
printer-02. Physical spool identity and current bed-clear state are not established by those reports.
The selected slicing profile is the A1 mini 0.4 mm nozzle, Generic PLA and Textured PEI plate;
this is a recorded process choice, not a claim of current printer telemetry.
The routine workload-identity entry point is implemented in the migration below; its deployment
remains pending. The operator-token batch route and live progress are recorded separately below.
No printer start is implied by staging the files. Label the finished gauges 3–4 and 4–5 before removing them.

### Printer authentication migration to the ntfy workflow

The operator requested migration on 2026-10-03. `gunbc.fleet.printer_federation_provision`
adds a dedicated `printer-lan` reader to the existing ntfy-approved GCP IAM estate. It requests
access only to the two rostered printer secrets; the print worker cannot create, rotate or delete
credentials or change IAM policy. The provider pins the repository, main workflow/ref, dispatch
and `printer-lan` environment through the shared OIDC claim authority.

`fleet-converge` mode `printer` has two operations:

1. `observe` resolves a printer's existing credential once and writes its exact version identity
   to a receipt. The receipt contains no access code. It performs no printer action.
2. `print` requires the selected printer, staged project path on srv1, expected SHA-256, numeric
   credential version and an explicit clear-bed report. It hashes a private copy before upload,
   uses the same version for FTPS and MQTT, and refuses a workflow rerun. A successful receipt
   means **start published**, not a claim that the machine reported RUNNING. Unknown outcomes
   require observation, never automatic replay.

The dedicated job runs on srv1, checks out the dispatch event SHA even if `expected_revision` is
supplied, and serializes starts per printer without cancelling an in-flight operation. It uses
`WorkloadIdentityToken`; no operator-token fallback and no destructive credential migration.
`gunbc.fleet.printer_host_tools.converge` is administrator preparation for the MQTT tool, using
the existing apt ensure and the tool's declared package. The worker checks the actual client
version before any upload. Mosquitto identifies its version through the documented `--help`
interface ([upstream manual](https://mosquitto.org/man/mosquitto_pub-1.html)).

**Deployment standing:** source changes are not evidence of installed IAM grants or a dispatched
print. The main-pinned workflow must land before it can federate. The existing IAM bootstrap must
cover the new target resources so its observer can plan and its approved executor can apply those
resource-local bindings. Then run the shared `gcp_iam_converge` workflow and approve its concrete
diff through ntfy, prepare the srv1 MQTT tool, and run credential observation. Final sliced projects
and live printer-state verification remain required before dispatching these gauges. The operator
reported both beds clear during this session; do not carry that report to an unrelated later print.

Validation: all seven printer workflow binding, admission, tool-version, OIDC, IAM approval and
start-replay regression tests passed. The canonical artifact gate regenerated the workflow; YAML
structure assertions and shell syntax checks passed. These are source checks, not live deployment
or printer observations.

### Operator-token batch execution — 2026-10-03

The operator supplied a one-hour GCP token and asked to print before the migration lands. For this
batch, `printer_operator_approval.run` uses the existing `OperatorSuppliedToken` capability and the
shared signed ntfy approval gate. It does not relabel that token as WIF, invent an Actions run, or
change IAM/secret versions. The permanent main-pinned GitHub route remains separate.

The approval binds printer, project SHA-256, exact credential version and batch attempt. After the
live approval it subscribes to a fresh report and requires IDLE/FINISH with zero print error, then
creates a persistent per-printer start marker before upload/start. A refused or ambiguous start is
never automatically replayed. `printer_report` reads telemetry through the same fleet identity,
exact credential and pinned CA; its report is current-state evidence, not causal acknowledgement.

Printer-01 credential version 1 was read successfully through the modeled client. Holes 3–4
were sliced successfully by Orca 2.4.2, with ZIP and embedded G-code checksum checks, one plate,
identity scaling, 15 layers/3 mm height, and bounds including the brim within 180 mm. Selected
settings: PLA, 220 °C nozzle, 65 °C textured bed, 0.20 mm layers, 2 mm outer brim. Estimated
19m22s and 6.99 g. Project SHA-256:
`28a8700489cf908edb9ed59c0aa871fe6903b4d558c3b64de6a7636af701580e`.

The isolated srv1 execution directory is `/home/briansrls/print-run-2026-10-03`; credentials live
only in owner-only private staging files and are not repository artifacts. Holes 4–5 also sliced
and passed the same checks: 15 layers, 13m48s, 3.67 g, SHA-256
`a8f434e80014b6a316d30665b372bd0df3e858a68a2ff04a5fe4f50031d68535`.

The shared approval client now follows the declared writer cutover to the approval broker on
8085; the old roadmap endpoint refused writes as designed. Both requests were filed and reached
approval polling. Four operator-route witnesses passed, including all three writer postures.
Fresh read-only reports showed printer-01 IDLE/error 0 and printer-02 IDLE/error 83902511.
Both reported PLA in AMS tray 0 (physical slot 1), while the existing start command selects the
external spool. The initial executions were stopped before upload to resolve that mismatch.
No start marker, physical start, or RUNNING observation was claimed at that point.

### Local preparation environment

The isolated worktree is `/home/briansrls/.worktrees/gunbc/3d-printing-2026-10-03`.
Local preparation artifacts are in `/home/briansrls/print-prep-2026-10-03`:

- `venv/`: CadQuery 2.8.0; resolved dependency versions in `requirements.lock`.
- `target/debug/gunbc`: compiler built from this worktree. Invocation requires an enforced cgroup
  memory budget; the preparation run uses a systemd scope with `MemoryMax=24G` and no swap.
- `mounting-check-map.svg`: schematic from the checked-out standard-location rows; all physical
  correspondences remain subject to the bench check.
- `verify-generated-gauges.py`: reads the emitted literal geometry instructions, runs them in
  CadQuery, checks the resulting solid and STEP round-trip dimensions/hole axes/volume, and checks
  STL bounds, closed manifold edges and volume. Its result only concerns generated-file conformance.
- `slicer-probe/DO_NOT_PRINT_old_settings.3mf`: tool-recovery probe using the old project settings;
  deliberately excluded from the next-print candidates.

The preserved OrcaSlicer 2.4.2 AppImage at `/tmp/orca.AppImage` matches the digest recorded in
`extdeps.printing.orca_slicer`. Its extracted CLI at `/tmp/orcax/squashfs-root/AppRun` now launches
after installing its missing OpenGL/WebKitGTK runtime dependencies. `--help` and model inspection
work. The modeled probe now uses the supported `--help` interface. `with_verified_orca` copies
and hashes the pinned AppImage in a private realization, runs probe and slice inside its callback,
and removes the realization before returning. `slice_run.run` uses it to produce the gauge project;
its success establishes slicing, not an observed physical print.

### Gauge observations to collect today

Record each board separately: stable label, model/revision, selected holes and gauge orientation.
The gauge checks **LocF–LocC**, modeled as offsets of **157.48 mm along the rear edge and
22.86 mm in depth**. These are model values, not measured dimensions of the printed part.
Establish the physical hole correspondence before interpreting a result; say whether checking board
holes directly or installed standoffs, since the latter also tests standoff placement.

Report whether both locations align simultaneously with the gauge resting naturally; if not,
which end aligns and the direction of the miss. Note obstructions or warp. Do not count alignment
obtained by bending the gauge or drawing it into place with screws as a pass. Record measured
values only when actually measured, with the instrument used. A photo can supplement the result.
An F–C pass supports that pair on that board, not the full mounting map or underside clearance.

### Next steps and dependencies

| Order | Work | Dependency / completion evidence |
|---|---|---|
| 1 | Record each board's F–C result | Operator observations; retain disagreements and distinguish board-hole alignment from standoff fit |
| 2 | Resolve misses; extend successful checks to remaining mounting locations | For a miss, distinguish orientation, printed-gauge dimensions and board/standoff placement before changing geometry. A pass permits additional pair gauges or an adjustable fixture, not freezing the entire map |
| 3 | Audit CAD, slicing and print evidence | Can proceed during board checks: locate generated artifacts, profile/toolchain identity, printer/material/nozzle binding and outstanding solid-readback controls; close PRINT-7/8 only against their full criteria |
| 4 | Complete process observations | Attribute the existing coupon only from retained evidence; otherwise print labeled replacements. Observe both printer/spool combinations independently and repeat attempts before claiming repeatability |
| 5 | Build an unpowered adjustable fit fixture | Confirm remaining mounting positions, standoff thread/height, underside keep-outs and connector access. PLA can serve this experiment without qualifying structural or powered use |
| 6 | Measure cassette inputs and choose joints/material | Can start now: actual cooler and PSU envelopes, node mass, drive requirement, fasteners, connector access and cable service space. Joint family and structural material precede their qualification prints |
| 7 | Prototype one removable cassette and one fixed bay | Requires fit and structural-process evidence; then PSU/cables, airflow and powered thermal validation under standing requirements, followed by 2x2 service demonstration |

The immediate milestone is a verified mounting map and an unpowered fit fixture. Full cassette
geometry depends on measured hardware envelopes and structural-process qualification.


Governing statement:

> Build a low-cost, space-efficient, thermally intentional system of independently removable
> ALTRAD8UD-1L2T server cartridges, using small-format printers for the custom geometry and
> conventional materials only where they are structurally superior.

The experiment succeeds when the model produces real printer artifacts, a board fits, a node runs, a
middle node is removable without disturbing neighbours, and the cost/space/thermal receipts decide
whether more printers are worth buying.

## Original motivations (the spine — every decision answers to these)

1. Cheaper than commodity chassis.
2. More space-efficient — this is not a storage chassis.
3. Deliberately directed airflow, not a case shaped for a different machine.
4. Forcing pressure on the spatial/fabrication model. This is a first-class goal, not a side effect.

## Fixed facts

| Fact | Value | Authority |
|---|---|---|
| Node subject | ASRock Rack ALTRAD8UD-1L2T | `extdeps.boards.asrock_rack` |
| Board outline | 267 x 244 mm | vendor manual §1.4, read first-party |
| Intended nodes | 8 (srv1-4 + 4 inbound) | operator |
| Printers | 2 x Bambu Lab A1 mini, scaling to N | operator order relay |
| Build volume | 180 x 180 x 180 mm | A1 mini spec table, operator relay |
| Fan form factor | 80 mm; **count derived**, not assumed | operator + derivation |
| Rack unit | 2x2 block, composable | side-chat ruling |
| Service law | Integrated corner-column stack; proposed temporary supports permit target-only removal, otherwise LIFO (4 October 2026, superseding R9) | operator ruling, [milk-crate stack design](milk-crate-stack-design.md) |
| PSU placement | one power cassette per column in the cassette format, the column's mains boundary (4 October 2026; supersedes the fixed-frame shared supply) | operator ruling, [milk-crate stack design](milk-crate-stack-design.md) |
| External bundle | 2 x Ethernet + 1 x protected, isolatable DC feed per node; AC at shared supply | operator acceptance, 4 October 2026 |

## Standing laws

- **Missing physical input policy: refuse.** No default, no nominal catalogue figure, no nearby standard.
- **Hidden CAD geometry authority: refuse.** The model owns dimensions *and* geometry-selection laws.
- **Printed polymer is never the mains enclosure and never a conductor in either grounding network.** Protective earth is bolted to metal and survives every removal the design invites; the board-to-chassis bond is functional only.
- **MVP thermal fixture choice carries no authority over deployment layout.** Using the 4U cooler
  first to reduce thermal uncertainty must not silently become the rack pitch.
- **Bay pitch derives from measured millimetres.** "2U" and "4U" are catalogue labels, not lengths.


## The numbered spine — PRINT-0 .. PRINT-15

The six projects are the working-memory units; these are the executable steps. Zero-indexed per the
RLM-N / MEMORY-0 convention already used in this directory. Each step names its terminal evidence,
because a step without one cannot be said to be done.

| Step | Project | Terminal evidence | State |
|---|---|---|---|
| **PRINT-0** | MEASURE | Board outline typed from the vendor manual (243.840 x 266.700 mm exact, axes corrected); micro-ATX lettered grid under its own standard authority; standoff placement derived from per-location evidence standings | **done** |
| **PRINT-1** | TOOLCHAIN | Build envelope, filament classes, slicer platform claims as separate authorities, **and a concrete A1 mini product row that the coupon fit witness consumes** | **done** (binding mutation-verified) |
| **PRINT-2** | MEASURE | Measurement authority: absent refuses, duplicate-key conflict refuses, precision budget enforced | **done** |
| **PRINT-3** | CAL | Coupon authority: ladder as modeled experimental design, rungs and plate derived | **done** |
| **PRINT-4** | TOOLCHAIN | Realization contract v0: contract derives its own rungs; a wrong-step handler is caught and located | **done** |
| **PRINT-5** | TOOLCHAIN | Raw transport schema admission: unknown version, missing field and unknown operation all refuse before any geometry call | **done** |
| **PRINT-6** | TOOLCHAIN | CadQuery handler + authority wall; sealed instruction plan; walls 1-3 execute, wall 4 (solid readback) is a declared boundary | handler/export profile landed; full readback closure unverified |
| **PRINT-7** | TOOLCHAIN | STEP/3MF emitted and re-inspected; output conformance re-establishes envelope, holes, walls | kernel blocker resolved; full acceptance to verify |
| **PRINT-8** | TOOLCHAIN | Slicer profile bound or refused; per-printer manifests; ProcessQualificationIdentity minted, not branded | live printing exercised; qualification closure to verify |
| **PRINT-9** | CAL | Coupons printed on BOTH printers; each printer+spool admitted or refused **independently** | coupon feedback recorded; independent qualification and repeatability pending |
| **PRINT-10** | FIT | Adjustable-standoff fixture; real board mounted unpowered; hole map measured back and frozen | gauge in hand; 1–3 (F–C) alignment pass reported; full fixture/map open |
| **PRINT-11** | CASSETTE | Structural cassette: rails, tray, handle, latch; carries node mass; no seam in layer-separation tension | needs PETG decision |
| **PRINT-12** | CASSETTE | Shared fixed-frame PSU/distribution and node harnesses; every connector insertable; nothing side-loaded; **PE continuous with board, standoffs and cassette all removed** | needs PSU envelope |
| **PRINT-13** | CASSETTE | 80 mm fan carrier + replaceable duct; powered thermal admitted against a bench baseline | needs cooler choice |
| **PRINT-14** | RACK | One fixed bay; cassette retained in service, removable after disconnect; empty bay structurally complete | |
| **PRINT-15** | RACK | Management-node mount (Raspberry-Pi class) on the rack, carried as a bay peer rather than an accessory | |
| **PRINT-16** | RACK | 2x2 block; an enclosed node extracted with neighbours untouched | |
| **PRINT-17** | VERDICT | Cost, volume, print hours, labour, reprint rate, thermal, service time -> buy-more-printers decision | |

**THE SEQUENCE IS INTEGERS, AND AN EARLIER REVISION BROKE THAT.** It carried PRINT-11b and PRINT-13b
while still calling itself N = 15, so the spine had two numbering authorities at once and the count
was wrong. Letter suffixes were how cable routing and the management mount got appended as
afterthoughts instead of being placed. Renumbered; new work takes the next integer.

**CABLE ROUTING IS NOT A STEP.** The operator's requirement is routing at EVERY controlled layer, so
it is a property each cassette and rack step must satisfy to be called done, not one step of its own.
It was PRINT-11b, which was exactly the bolt-on that requirement forbids. Every step from PRINT-11
onward now carries the cable obligation in its own terminal evidence: endpoints, bend and service
volume, clip positions, strain relief, moving-versus-fixed classification, and declared separation
from blades and hot surfaces.

**N = 17.** PRINT-0 through PRINT-6 are landed and executing. PRINT-1's reopening is CLOSED: the
build envelope is owned by `extdeps.printing.bambu_lab_a1_mini` and the coupon fit witness reads
`a1_mini_build_envelope` rather than an inline literal. That was verified by MUTATION rather than by
reading the import — shrinking the product row's x from 180 mm to 50 mm flips
`w_the_hole_ladder_coupon_fits_the_a1_mini` from green to red, so the binding is live and not
nominal. The check was run because the module's own annotation asserted "changing this row is what
moves that witness", and an annotation asserting a property the code does not have is exactly what
#10119 had to repair four times.

PRINT-5 and PRINT-6 landed with #10119. PRINT-N (#10680) subsequently resolved CAD bootstrap,
added measured export-profile work, and exercised physical printing. PRINT-7/8 still require an
evidence audit against their full criteria; job-start evidence alone does not close them.

### What blocks what

- **CAD bootstrap is resolved.** Full output-conformance and process-qualification evidence
  remain to be reconciled.
- **Joint family** (dovetail / tongue-and-groove / keyed slide / pin) gates the
  ProductionInterfaceCoupon only — a later half of PRINT-8, not the MachineProcessCoupon.
- **Structural material qualification** gates durable/powered cassette work. PLA can carry CAL
  and unpowered FIT; no PLA-to-PETG receipt carry.
- **Deployment envelope** gates PRINT-13..14 layout admission, nothing earlier.
- **Drive count** gates PRINT-11 completion only.

## The six projects

| Project | Scope | Terminal evidence | State |
|---|---|---|---|
| **CONTRACT** | Goals, non-goals, authority graph, refusal law, MVP boundary | No assumption represented as a default | deferred — no consumer yet |
| **MEASURE** | Board, cooler, PSU, fan, DIMM, cable, **deployment envelope**, uncertainty | Pack sufficient to generate CAL+FIT | **partial — coupon readings recorded; board-fit observations pending** |
| **TOOLCHAIN** | Realization contract, CadQuery handler, authority wall, STEP/3MF, slicer env, printer nodes | Deterministic generation + executed no-parallel-authority controls | partial — CAD/export and live printing exercised; full conformance open |
| **CAL+FIT** | Coupons, qualified tolerances, adjustable standoff fixture, real-board fit, hole-map feedback | One unpowered board fits; hole authority admitted | in progress — coupon feedback and gauge in hand |
| **CASSETTE** | Structure -> PSU/cables -> airflow -> powered thermal | Removable node operates and is serviceable | not started |
| **RACK+VERDICT** | One bay -> 2x2 block -> scale/economics | Middle-node service proven; buy-more-printers verdict | not started |

Consolidation from eleven stages must not collapse the internal gates. CASSETTE keeps
`structural -> PSU/cable -> airflow -> powered thermal`; RACK+VERDICT keeps
`single bay -> populated 2x2 -> economic verdict`.

## The CAD authority wall (TOOLCHAIN's load-bearing control)

CadQuery is a **realization handler**. It may map modeled operations to kernel APIs, reorder
provably-equivalent operations, heal representations, triangulate a determined solid, and implement
a *modeled* derivation whose identity and inputs the model already fixed.

It may **not** choose dimensions, offsets, clearances, wall thicknesses, radii, chamfers, feature
presence, hole patterns, joint topology, rib placement, segmentation boundaries, print orientation,
load-path geometry, duct path or cross-section, fallback values, or one construction algorithm over
another where the physical result differs.

A numeric-literal scan is **necessary but insufficient** — `fillet()` vs `chamfer()` carries no
literal. The wall is four-part:

- **A. Geometry-taint dataflow.** Any expression reaching a geometry-affecting call derives only
  from the typed contract, a modeled operation, or a proved non-semantic toolchain constant.
- **B. No-default completeness.** Unknown feature kinds, missing fields, unsupported derivations and
  schema skew all refuse. No Python defaults for geometry-affecting fields.
- **C. Realization trace.** Every realized feature maps to one modeled source; no unmodeled feature.
- **D. Output conformance.** Re-inspect the emitted STEP/3MF for envelope, hole centres, wall
  thickness, mating dimensions, clearances, build-envelope fit, collision/extraction envelopes.

**Negative control:** hard-code a plausible wall thickness in the handler and prove the wall rejects
it while the part still looks valid. A wall that never flips is a decoration.

Duct curvature is the boundary case: inlet + outlet + envelope + airflow obligation do **not**
determine a duct. The model must fix the family and the selection law (centerline derivation,
minimum bend radius, wall thickness, transition rule); the handler may implement that law and may
not choose it.

## Landed so far

- `extdeps.boards.asrock_rack` — typed 243840 x 266700 um outline, first-party manual read. The
  extents are NOMINAL: `asrock_altrad8ud_board_depth_nominal_exact` is the exact ARITHMETIC
  conversion of the vendor's published 9.6 x 10.5 in, not the as-built extent of any board here, and
  the rename exists to stop "exact" being read as zero physical tolerance by a clearance derivation.
  Mounting holes are a standard-location correspondence with a per-location standing, so an inferred
  hole and a physically confirmed one are different facts; `asrock_altrad8ud_open_mounting_unknowns`
  carries the five unmeasured ones with their discharge instrument.
- `extdeps.standards.micro_atx` — the nine microATX mounting locations own their own authority
  rather than sitting inside the ATX 2.2 module, which now carries only ATX 2.2 facts. Coordinate
  provenance is `OperatorRelayedDocument` from the shared `DimensionEvidence` carrier, not a prose
  string: a citation STATUS is a typed fact, and the string form was a nickname nothing could consume.
  **One structural over-claim is recorded and deliberately not half-fixed** — `extdeps_model_scope`
  cites the archived ATX 2.2 PDF as `first_citation` for the microATX locations, a machine-readable
  claim the provenance row denies, because this session's extraction recovered that PDF's text layer
  and not its mounting-location artwork. No microATX document is in hand, so the repair needs a
  first-party read or an `ExternalAuthority` arm that can carry a relayed-with-no-document chain;
  both are named as triggers rather than guessed at.
- `extdeps.printing.bambu_lab_a1_mini` — the printer as a PRODUCT ROW: build envelope, nozzle
  diameters, filament diameter, temperature ceilings, input voltage and frequency range, machine
  extents, and a ten-row material suitability roster. The fit witness reads the envelope from here
  instead of a literal it wrote itself, so it goes red if the printer authority vanishes.
- `extdeps.vendor.bambu_lab`, `extdeps.printing.fdm` — agnostic FDM shapes; build-envelope fit that
  reports every exceeded axis and never searches orientations.
- `extdeps.printing.bambu_studio` — spec-sheet and release-artifact platform claims carried as two
  authorities; the Linux disagreement modeled, not resolved.
- `product.printed_chassis.measurement` — SPATIAL-1 binding; absent refuses, duplicate-key conflict
  refuses with no precision or recency tie-break. Six witnesses green, two proven to flip.
- Compile-clean: 0 blocking errors in these files; the 57 remaining are pre-existing on main.
- `product.printed_chassis.coupon` — **TOOLCHAIN v0's first consumer.** The hole-diameter ladder as a
  modeled experimental design: base, step and rung count are authored, every rung and the plate width
  are DERIVED. `PlateDimensions` is its own type so a malformed plate has no zero-extent fallback to
  return. Micrometre-to-millimetre conversion rounds UP, because truncation reports a 180.5 mm part
  as fitting a 180 mm machine. Eight witnesses green; the conversion proven to flip in both the
  arithmetic and the fit path. **V0 is now closed at one operation** (`OpThroughHole`) — see the v0
  population section for why the second one was deleted rather than kept.
- `product.printed_chassis.realization_contract` — the transport wall. Raw JSON from the CAD handler
  is admitted through `decode_envelope`, which refuses an unsupported schema version, an unknown
  operation kind, and an envelope carrying no operations, each with a typed cause naming what it
  received. Conformance reports the EARLIEST divergence rather than the last, and a count mismatch is
  its own outcome rather than a silent zip truncation.

  **It compares WHOLE GEOMETRY, not the whole specimen, and the distinction is load-bearing for
  PRINT-5.** Compared: plate on three axes, operation count, then every operation field. NOT compared:
  specimen identity (carried, single-armed), revision (not a compared subject at all), and feature
  identity — operation *i* is "the *i*-th rung" by LIST POSITION, not by an identity the operation
  carries. A permutation is refused because position is compared, which is not the same as features
  having identities, and the difference reappears the moment two features share a geometry. PRINT-5
  must not read a conforming verdict as adjudicating identity or canonical ordering.

  **The admitted value is unforgeable, and an earlier revision only CLAIMED it was.** That revision
  put the geometry on the accepted arm (`EnvelopeDecodedV0 { plate, features }`) and argued admission
  was unavoidable "because the refusal arms have no geometry to hand on" — a correct argument for an
  insufficient conclusion, since it rules out the raw envelope bypassing the judge and says nothing
  about a caller bypassing the raw envelope. A probe compiled the forgery from a foreign module.
  `DecodedRealizationV0` is now `sole_constructor`, so only `decode_envelope` can mint one; a foreign
  module may name the arm and cannot obtain a value to put in it.

  **That wall now has an executing witness, and the claim that it could not have one was false.**
  An earlier revision of this plan said the RED was a compile failure and so could not be enrolled in
  a corpus that must compile, and named an expect-compile-refusal harness as the missing capability.
  That was DESIGN §4b's own distinction misapplied: a state unauthorable in the ACCEPTED CORPUS may
  still be perfectly authorable as SOURCE HANDED TO THE COMPILER BY A FIXTURE, and only the second
  boundary decides whether a check's RED exists. The harness was already here —
  `gunbc.compile_diagnostic_census`, whose `compile_dag_diagnostic_census` takes source as data and
  returns either an observed diagnostic population or a typed `CensusNotRunnable`; the seven-form
  precedent against a sealed fixture type is `test.claim.sole_constructor_completeness_audit_probe_test`.
  The finding was routed by the side chat, which cited the precedent rather than the claim.

  The witness is `test.claim.printed_chassis_admitted_realization_seal_witness_test`, built on the two
  forms that module's own annotation names as exact: a count scoped to diagnostic class AND
  `subject_name`, and a differential between two sources differing on one axis. It carries a red
  source writing the `DecodedRealizationV0` record literal from a foreign module, a green source with
  identical imports and no literal, and the red-minus-green subtraction as a third witness so the
  shared imported closure cancels rather than being asserted away. The `CensusNotRunnable` arm answers
  `-1`, so every assertion compares against a non-negative quantity and a harness that never ran
  satisfies nothing — which is exactly the vacancy that produced the near-false-finding below.

  **The refusal payload could carry a success, and the witnesses were where it showed.**
  `RealizationVerdict`'s refusal arm was typed `RealizationRefusedAtWire { admission: EnvelopeAdmission }`,
  and `EnvelopeAdmission` includes `EnvelopeDecodedV0`. No route produces that state —
  `judge_realization` builds the refusal arm only on the three refusing branches — but reachability is
  not occupancy, and the state was writable. The tell was not in the contract at all: four witnesses
  carried a dead `EnvelopeDecodedV0 { admitted: _ } => false` arm purely to satisfy exhaustiveness over
  a case the route cannot produce, which is what a coproduct too wide for its position looks like from
  the consumer side. The refusal reasons are now their own closed coproduct `EnvelopeRefusal`, and
  `EnvelopeAdmission = EnvelopeDecodedV0 | EnvelopeRefused { refusal: EnvelopeRefusal }`. The class
  climbs from mechanically preventable to structurally impossible, and the four dead arms are deleted
  with it — §4b(4) retires the redundant production handling while every discriminating witness stays
  enrolled.

  **The trusted-type boundary sat one step early, and the fix turned out to be the name.** The routed
  review's objection was that the admitted carrier is minted before conformance runs. I pushed back
  that decode-admitted and contract-conformant are two genuinely different facts and so the *stage*
  was not wrong. That was true and beside the point: the defect was that an unqualified capability
  name — "admitted" — was attached to the earlier fact, while it was also the only geometry-bearing
  value a handler could obtain. Renamed `DecodedRealizationV0`. The stage stayed.

  **What the pushback did surface is a second, real defect, deferred deliberately.** `judge_realization`
  returns a verdict and no geometry, so a handler must obtain geometry elsewhere and associate it with
  the verdict itself. That association is unbound: judge envelope A, retain envelope B's decoded
  geometry, hand B to the handler under A's conforming verdict. Every honest caller passes the same
  envelope twice and the invalid pairing stays writable — the same class as the refusal payload above,
  one level out. The close is a second sealed type returned *from* the judgment that established
  conformance, minted from the contract's own canonical specimen rather than from the transported
  values that happened to compare equal, so what a handler cuts is the model's geometry and the wire is
  reduced to an assertion that was checked. `AdmittedRealizationV0` is reserved for it and unspent.

  It is not built in this revision because nothing actuates geometry yet: with no handler the pairing
  has no site at which to go wrong, and minting a sealed carrier with zero consumers to hold a symbol
  is what §2 prices as redundant. So it is declared as `realization_v0_open_obligations` — a typed
  carrier and not an annotation, because PRINT-5 must not be able to reach a handler without answering
  it, and no `Accepted` program can read a comment.

  **The coupon could not be identified from its own geometry, and the fix is an authority change, not
  a handler change.** The ladder steps 0.10 mm across eleven rungs, so end to end the holes differ by
  1 mm and are visually interchangeable; rotated 180° the coupon reads as a valid coupon with its
  ordinals reversed, and every measurement is attributed to the wrong rung. Routed review found it,
  and also corrected its own first proposal — one obligation was doing two jobs:

  - **orientation** — which end of *this* coupon is the 3.00 mm end. A property of the canonical
    geometry, and now discharged by making that geometry asymmetric.
  - **print-instance attribution** — which printer and spool made *this* piece of plastic. Two
    correctly-oriented R1 coupons remain interchangeable. No geometry closes this, because the datum
    must be *identical* on both coupons or the two processes stop sharing a subject. It stays open,
    routed to the manufacturing manifest and a physical handling route.

  Holding them as one obligation is why the earlier route fold was near-vacuous — a single arm whose
  RED could not be authored. Splitting is what let one of them close. `HoleDiameterLadderR1` is the
  coupon *design* identity and never the identity of a printed instance.

  **The datum is its own field, not another entry in the feature list.** Appending one more
  `OpThroughHole` and remembering the last one is the datum makes "which hole is the datum" a
  positional convention — the class this module already deleted once — and it is worse here, because
  the datum exists precisely to remove an ambiguous reading. `HoleDiameterCouponGeometry` carries
  `plate`, a scalar `orientation_datum`, and `ladder_holes`, which makes four things structural: one
  datum exactly (zero and two are unwritable, not refused), the datum cannot become a twelfth rung,
  canonical geometry with no datum has no representation, and every consumer must name which
  population it means. The wire splits the same way — two *different* populations, not two parallel
  lists of one.

  **The near-miss worth recording is inside the derivation.** A first cut read rung zero's x back off
  the built feature list to make the correspondence structural. `.first()` returns an option, so it
  forced an `Absent` arm for a ladder that cannot be empty — and the arm answered with a fabricated
  datum at the margin. An absorbing fallback in miniature, inside the one function the whole
  orientation guarantee rests on. It now reads `hole_ladder_margin`, the same row the ladder fold's
  `i = 0` term reads: one row, two readers, no unreachable branch to fabricate in. The correspondence
  is asserted by `w_the_datum_is_at_rung_zero` rather than constructed, which is honestly one rung
  lower — mechanically preventable, not structurally impossible — with the trigger named.

  **Evidence, and it discriminates.** Six controls: canonical datum conforms; datum at the far end
  refuses naming `FieldCenterX` (the 180° reading itself); datum on the ladder centre-line refuses
  naming `FieldCenterY` (right x, so it passes only if the y is compared); the datum stays out of the
  measured population; it clears the ladder band and the plate edge; and it sits at rung zero's x.
  A mutation that compares the modeled datum against itself turns the two refusal controls **RED**
  while the positive control stays green — so they discriminate the wall rather than merely
  exercising it.

  **Probe discipline learned here, recorded because it nearly produced a false finding.** The first
  forgery probe returned exactly the baseline error count — consistent with "forgery permitted" — from
  a file the compiler had never read, because an added source root was silently ignored. Confirmation
  and vacancy produce the same number. Only a deliberate must-fail control in the same file
  distinguished them.

### PRINT-5 — the handler, and the one thing it does not yet do

The pairing discharge and the CadQuery handler both land: `AdmittedRealizationV0` is sealed and
minted only inside `judge_realization` from the **canonical** specimen, and `realize_admitted` takes
that capability and nothing else. Five witnesses green by execution — per-feature trace with the
datum as a distinct subject, uncentred plate, whole-program equality against a model-rebuilt
expectation, exact millimetre rendering on odd micrometres, and overtravel moving no measured
dimension.

**Actuation is registered but NOT materialized, and that is the honest state.**
`CouponCadQueryProgramArtifact` is a `gunbc.generated_artifact` variant generated through
`artifact_generate`, and it is `NotCommitted`. Since `main_wet` selects only committed artifacts,
**nothing writes the `.py` to disk today.** The remaining work is a typed workspace materializer and
a pinned CadQuery environment; using `main_wet` to keep an actuation claim alive would have been the
claim outrunning the execution.

The commit-policy question resolved against my first answer. I had registered it
`CommitRequired { consumer: FabricationToolchain }`, reasoning that `RepoConsumer` already spanned
more than git and CI. The correction, from review 5095012465: `RepoConsumer` answers the narrower
question *why must these bytes persist in a git checkout*, and a CAD kernel reading a materialized
workspace establishes no such requirement — one `FabricationToolchain` arm would have conflated the
program consumer, the solid consumer, the mesh consumer, the slicer and the printer. The arm becomes
justified only when a workflow requires an operator to obtain *this file* from a checkout, and then
it is narrow (`FabricationWorkstationCheckout`) with a receipt.

Committing it had already produced its own evidence: `.gitignore`'s blanket `*.py` swallowed the
path, so an artifact declared `CommitRequired` could never be committed and the drift gate would
have read an absent file indefinitely. A derived negation section in `gunbc.gitignore_emit` fixed
that and has been reverted with the policy — with no such artifact it is machinery whose
non-vacuity control names a path that no longer exists. **The latent class is real and now
unwalled**: a `CommitRequired` artifact whose path matches a developer ignore pattern is silently
uncommittable. Its trigger is the first such artifact; walling it before one exists would be a check
whose RED is unauthorable.

**Historical pre-PRINT-N state; the measured export profile subsequently landed.** Export was removed from the emitted program rather than profiled. The
`cq.exporters.export(result, "…stl")` line chose five things the model never stated — STL over STEP
or 3MF, a filename, a working-directory-relative destination, format-by-extension, and CadQuery's
default tessellation policy. The last is not neutral: STL is a mesh format, so every modeled circle
becomes a polygon whose fidelity is set by linear and angular tolerances, and the emitter was
choosing how round the holes this coupon exists to *measure* come out. The program now builds
`result` and stops. `CadQueryExportProfile` is a real obligation and is deliberately unauthored:
with no kernel here its tolerance fields would be literals with no oracle.

**Four defects the review found, three of which my own annotations had asserted away.** `zip_map`
truncates (`Empty` on either side) while the comment claimed it refuses a length mismatch — the
third appearance of the positional-parallel-list defect here, and the first where a comment supplied
the false guarantee; replaced by one row per substitution, which deletes the correspondence. The
capability wall leaked through `plate_line`/`cut_line`, callable on bare geometry, directly beneath a
header claiming there was "not even a private one" — the emitters now destructure geometry inside
the arms of a sealed `CadQueryInstructionV0` that only `cadquery_emission_plan` can mint.
`AdmittedRealizationV0` had no seal evidence at all while a witness annotation asserted its seal
held; its red/green/differential battery is now enrolled and green. And the overtravel annotation
described a doubled-overtravel diff that no body ever performed — the claim is narrowed to what
executes, with the solid readback recorded as an open obligation.

The discharged `RealizationV0Obligation` vocabulary is deleted. Keeping an empty roster was argued
on the grounds that it preserves a typed place for the next obligation, but nothing read it — no
gate required emptiness — so the forcing did not exist, and the discharged property is carried where
it executes, in the type of the accepting arm.

**Historical bootstrap probe, superseded by PRINT-N conda-forge bootstrap.** Wall 4 — re-reading the produced *solid*
for plate dimensions, datum location and the eleven ladder holes — is a §4b boundary obligation, not
a rung. Its trigger was written as "a runner with cadquery importable", which was too vague to act
on, so it was probed to termination:

- CadQuery **does** install from PyPI on this arm64 container (wheels exist for aarch64); the
  earlier note that this box has "no CAD kernel" was a missing-bootstrap observation, not an
  architecture wall. `pip` bootstraps into a venv via `get-pip.py` (the system interpreter is
  PEP-668 externally-managed, so `--user` refuses).
- `import cadquery` then fails on **`libGL.so.1`**, which is satisfiable by extracting
  `libgl1`/`libglx0`/`libglvnd0` plus the X client libs.
- Past that, `casadi` requires **`GLIBCXX_3.4.32`**, and a libstdc++ carrying it requires
  **`GLIBC_2.38`**. This container is **glibc 2.36**.

The probe then incorrectly concluded the trigger was **a base image with glibc ≥ 2.38 (or an x86_64 runner), not a pip install** —
and assembling one by hand-extracting `.deb`s onto `LD_LIBRARY_PATH` is where a probe turns into the
workaround §5 names, so it was stopped there rather than pushed through. The environment wall 4 needs
is a *pinned, identified* toolchain — Python version, CadQuery version, OCP closure, image digest,
install method — which is the same identity the qualification receipt has to carry anyway.

**What this costs tomorrow, stated plainly.** Without wall 4 the physical deviation measured on a
printed coupon is the sum of model→source, kernel, tessellation, slicer and printer–spool terms, and
the coupon exists to isolate the last. A print made before wall 4 closes therefore yields operational
evidence (adhesion, gross scale, obvious defects) and a valid **A-vs-B differential** between the two
printer–spool pairs — the upstream terms are common-mode across two prints of the same artifact and
cancel in the comparison — but it **cannot issue an absolute process-qualification receipt** against
the modeled nominal. Relative comparison survives; absolute qualification does not.

## The two-day cut (printers arrive 2026-09-03)

**Ruling: a CAL-driven vertical slice, not a general framework and not hand-authored coupon CAD.**

```
CalibrationCouponAuthority -> RealizationContract v0 -> four-part wall
  -> CadQuery handler -> STEP/3MF conformance -> bound slicer profile
  -> per-printer-node manifests -> physical coupon observations
```

The coupon is TOOLCHAIN's first real consumer. Governing rule:

> **Generalize TOOLCHAIN only one consumer ahead.** CAL determines v0; FIT determines the next
> expansion; CASSETTE the next. A new geometric operation arrives only with taint coverage,
> refusal behaviour, trace coverage and output conformance.

A wall with no handler is only a rule definition — the geometry-taint arm is not commissioned until
it observes real geometry calls, passes the lawful ones and rejects mutations.

### v0 operation population (closed; a feature belongs only if a chosen coupon consumes it)

box/prism · cylindrical through-hole · slot · linear or grid repetition · male/female clearance pair ·
wall or rib · rigid transforms · boolean union and subtraction

**`part/revision datum marking` was in this list and has been REMOVED from it, deliberately.** As
`OpDatumMark { text, at_x, at_y }` it fixed text and position and left FONT, GLYPH METRICS, STROKE
WIDTH, ALIGNMENT, ORIENTATION, DEPTH and ENGRAVED-VERSUS-EMBOSSED to the handler — the §3 tell in its
exact form, the handler choosing geometry the model had not fixed. It had already produced a silent
falsehood: `specimen_extent` assumed the mark was engraved and so assumed it could not enlarge the
plate, an assumption stated nowhere and enforced by nothing, and a handler that embossed would make
the envelope-fit answer wrong in the unsafe direction. Determining it fully means modelling fonts,
which is a domain rather than a field, imported to label a calibration coupon.

The need behind it is real and is tracked, not dropped: several coupons will exist physically and
must be told apart by hand. `coupon_v1_physical_identification_obligation` carries it as a V1
obligation with the route that looks right — identify the coupon by GEOMETRY, a coded through-hole or
notch pattern, which `OpThroughHole` already determines completely, checked by the same contract the
holes already pass through.

The ladder itself is a modeled experimental design. "It is only a calibration coupon" is not
permission for Python to choose test dimensions.

### CAL splits into two coupon classes

- **`MachineProcessCoupon`** — X/Y deviation, hole and slot deviation, sliding clearance, thin wall
  and rib, orientation anisotropy, first-layer behaviour, repeated-feature consistency. Depends on
  **no** server or site measurement. This is the first print.
- **`ProductionInterfaceCoupon`** — the exact joint family, insert/captive-nut geometry, fastener
  clearance, rail fit. Needs no server dimensions but **does** need the joint family and purchased
  hardware identities chosen. Do not invent an M3 pocket or dovetail angle to populate it.

### The slicer is a SECOND authority boundary

Scaling, dimensional compensation, line width, first-layer compensation, wall construction, layer
height and orientation all change the physical result. A `.3mf` carrying hidden hand-edited
compensation is as much a parallel authority as a Python literal. Qualification identity:

```
printer_node x firmware x material_product x material_spool x installed_nozzle
  x slicer_identity x slicer_profile x orientation x support_policy
  x coupon_revision x calibration_epoch
```

### Wall commissioning needs four negative controls, not one

1. **Numeric/dataflow** — a plausible Python-derived wall thickness. Taint must reject.
2. **Nonnumeric topology** — a fillet, chamfer or extra rib with no modeled feature identity. Trace
   or taint must reject.
3. **Default** — omit a required contract field and let a helper default take over. Completeness
   must reject.
4. **Output** — alter an exported hole or envelope after lawful realization. Conformance must reject.

### Material: PLA does not carry to PETG

No PLA-to-PETG receipt carry. Distinct admissions:
`ToolchainSmokeAdmitted` · `FitPrototypeProcessQualified` · `StructuralProcessQualified` ·
`ThermalServiceQualified`.

So the PETG decision blocks the **durable/powered CASSETTE path**, not the first CAL print or the
unpowered FIT fixture. Qualify each printer-spool combination independently; never infer that an
observed difference is the printer alone.

## Open decisions (operator) — structural and rack work

- **Drive count** — a CASSETTE completion obligation. Does not block gauge checks or the unpowered fixture.
- **PETG** — blocks structural/powered parts, not CAL or unpowered FIT.
- **Deployment envelope** — **not on the pre-arrival critical path.** Its absence must refuse
  rack-layout admission later, not block calibration now.

## Next

Follow the dated next-step table above. Board-specific gauge observations and remaining mounting
checks lead to the unpowered fixture; toolchain-evidence reconciliation and cassette measurements
can proceed independently. The former list predated the landed PRINT-N implementation.

**Deferred as definition-only residue until each has a consumer:** `CONTRACT` (needs the generator),
`DeploymentEnvelope` (needs the layout-comparison consumer). The design of both is pinned above.

## Supply identity — RULED, and it unblocks the filament slice

The open question was whether a spool is an inventory **lot** or a **PhysicalAsset instance**. Ruled:
**one supply instance per physical spool, allocated at durable physical individuation, each retaining
its source procurement lot.** The dependency runs one way only —

```
identity makes a later divergence attributable
NOT: a later divergence earns the object an identity
```

so the two day-one spools are separate instances *before* either print begins, even though they came
from one order, name one catalog product, and have no known difference. Waiting for measured
divergence would leave the first divergence with nowhere truthful to live.

Grains stay distinct rather than collapsing onto the lot: catalog product (what was sold),
manufacturer batch (what the maker says), procurement lot (what was bought), supply instance (which
spool fed this print), condition observation (instance x time). **One order is not one material
batch** — `ProcurementLotIdentity` and manufacturer batch standing stay independent, since one order
may span two production batches and two orders may share one. Moisture at print time is an
instance-and-time observation; a drying cycle mints a new condition observation, never a new spool
identity.

RFID and operator label are both **evidence routes, not identity kinds**. They may coexist; an RFID
read failure does not create a different spool, replacing a label does not create a different spool,
and disagreement must refuse the attribution rather than letting either route silently win. The RFID
UID is a machine-read manufacturer identifier, not a guaranteed globally unique spool serial, until
something establishes that.

The lot-to-spool relation must not be authored twice. The existing inventory event already relates
installed assets to the lot whose stock moved, so day one is one install event per spool with
`quantity: 1`. The generic rule refuses only when assets *exceed* quantity, which would admit
`quantity 2, assets [spool_a]`; an attribution-critical supply needs the stronger local law
`asset count == installed quantity AND the requested asset occurs exactly once`. The admitted supply
may carry the derived source lot, and must never accept an independently authored `source_lot`
without comparing it against the event.

Catalog boundary: `PhysicalAsset` requires a catalog `DeclarationRef` while `ProcurementLot` may
carry an unread catalog standing. Instance-per-spool must not force a fabricated exact filament
product — where only a package description is known, keep the unresolved standing and limit the
receipt to an end-to-end observation. Do not point a spool at the `Pla` polymer class as though a
class were a particular commercial product.

### Correction landed to the printer slice

The manifest annotation said the roster is filled by **reading the serial off each machine**. That
conflated two states the carrier already distinguishes: `PhysicalAsset.physical_serial` is optional,
so a printer present on the bench with an unread serial and a printer that has not arrived are
different facts, and collapsing them would report a machine the operator is standing in front of as
absent. A row is allocated at receipt and durable individuation — an operator-applied label suffices
— and the serial **corroborates** that binding rather than establishing it. If a serial is ever
required before attribution, that is an admission refusal arm, visible and countable, never silence.

### Two coupons per machine is two attempts, not two objects on one plate

`A1/B1` then `A2/B2`, each pair consuming the same machine artifact, supplies and nozzle unchanged
and no intervening calibration unless recorded. Two copies printed together answer **within-job /
bed-position** variation; two executions answer **print-to-print** variation. Both are useful and
they must not share one receipt name.

## On arrival (2026-09-03)

```
printer setup and identity observation
  -> same MachineProcessCoupon on each printer
  -> measure without silently averaging contradictions
  -> admit or refuse each process identity independently
  -> derive qualified FIT clearances
  -> generate adjustable FIT fixture
  -> mount the real board unpowered
```

The duplicate-measurement repair already has the right semantics for this: lookup refuses
contradiction. A future adjudication relation may supersede a measurement, but selection must never
be smuggled into ordinary resolution.

### The bench procedure, in the order it must physically happen

Written out because the ordering is the whole content: two steps here capture facts that **no later
measurement can reconstruct**, and both are cheap only while the parts are still on the machines.

1. **Label both printers before either is powered on.** A physical label, applied by hand — `A` and
   `B` is sufficient. This is the durable individuation the roster needs; the identity is allocated
   here, not derived from anything read later. Record the machine each label went on.
2. **Label both spools before either is loaded.** Same rule, `PLA-A` / `PLA-B`. Do this while they
   are still sealed and distinguishable as objects rather than as "the one in the left printer".
3. **Read the serials, if convenient — and do not wait on them.** The serial *corroborates* the
   binding made in step 1; it does not establish it. If a serial is unreadable, the printer is still
   registered. If an RFID tag reads, record it as evidence beside the label, never instead of it.
   Label and tag disagreeing is a refusal to resolve later, not a coin flip.
4. **Load spool A into printer A, spool B into printer B, and write down which went where.** This
   pairing is the treatment variable of the entire experiment. It is not recoverable from the plastic.
5. **Print the same coupon on both machines, concurrently.** Same generated artifact, same profile,
   same nozzle. Concurrency matters: it holds ambient conditions roughly fixed across the pair.
6. **Bind each coupon to its printer and spool BEFORE it leaves the bed.** Write on it, bag it and
   label the bag, or photograph it in place — any durable mark. **This is the irreversible step.**
   The two coupons are geometrically identical *by design*, so once both are off their beds and on
   the same table, nothing measured afterwards can tell them apart. Getting this wrong does not
   degrade the experiment; it voids it.
7. **Then repeat as a second attempt: A2 and B2**, same supplies, same nozzle, same profile, no
   intervening calibration unless it is recorded. A1/B1 versus A2/B2 answers *print-to-print*
   variation. Two copies on one plate would instead answer *bed-position* variation — a different
   question with a different receipt name, and conflating them is how a machine difference gets
   attributed to a corner of a build plate.

What is deliberately **not** on this list: any dimension read off a coupon, any compensation constant,
any judgement about which machine is better. Those are measurements, and they are all still available
tomorrow. Steps 1, 2, 4 and 6 are the only ones that expire.


## The floor-budget cluster, and why this program did not take the coverage loss

Adding this program's witnesses tipped a rotating subset of ten whole-corpus-reflection claims past
the floor's 500ms per-claim ceiling — claims that already sat at 415-436ms on main and were already
`[over-cost]` flagged. Two runs tipped DISJOINT subsets, so there was never a single slow witness to
chase.

The interim move was to lift those ten off the required floor into `test.claim.long.`, declared as a
§4b(3) rung drop. That was reverted before it was pushed, and the reason is worth keeping: a child
lane investigating the cause found `deduplicate_identities` building a COPIED ACCUMULATOR inside a
quadratic fold, over a population that is the corpus — roughly 16s of the 19s that one reflection
costs. Rewritten as a set-membership fold it drops the standing to ~4.8s.

So the ceiling was never the problem and the ten witnesses were never really the subject. Taking
them off the floor would have spent ten executing checks — including the one that EXECUTES the
compile-phase ratchet — to work around a cost-shape defect that §6 says is always fixed regardless of
realized n. The fix lands on main ahead of this program, and this program takes no drop.

The transferable rule: when a budget refusal names your change as the trigger, the trigger and the
cause are different questions. A rotating victim set is the tell that you have found neither.

## Grounding — R14, and why it is TWO networks rather than one path

**Corrected.** An earlier revision of this section said the earth connection is "part of the docking
interface" and that the earth path runs through the standoffs. That is wrong, and wrong in the
dangerous direction: it makes the safety path depend on a mechanism whose whole purpose is to be
disconnected by hand, routinely, by design. The requirement below replaces it.

There are two networks. They serve different purposes, they have different failure consequences, and
conflating them is what produced the earlier error.

**1. Protective earth (PE) — a safety network, and never load-bearing on anything removable.**
From the AC inlet's earth pin, through the PSU's own vendor enclosure, to a dedicated bonding point
on the rack's metal member, and from there to every accessible conductive part. It is bolted, not
docked. The witness is stated as three simultaneous conditions, because any one of them alone is a
state the rack will really be in:

> PE continuity holds with the motherboard removed, AND with every motherboard standoff removed, AND
> with the removable cassette undocked.

If pulling a node can open the earth network, the design is wrong regardless of how good the contact
is when it is seated.

**2. Board-to-chassis bond — a functional network, for EMI and reference, not for safety.**
Board mounting hole → metal standoff → cassette metal reference. This one legitimately breaks when
the board is removed, because that is what it is for. It may be a return and reference path; it may
never be the reason a chassis surface is safe to touch.

The consequences for the build:

- **Metal standoffs into a metal member.** The hybrid load path already buys aluminium extrusion or
  threaded rod — that member is the natural bonding conductor and is present for structural reasons
  anyway. It serves network 2, and is a *bonded branch of* network 1, not a segment of it.
- **Printed polymer is never a conductor in either network, never the mains enclosure, and never the
  sole earth path.** The PSU stays in its vendor enclosure.
- **The docking interface carries no PE obligation.** A dock is a connector; PE is a bolt.

The obligation this creates for MEASURE: the PSU's earth-stud or bonding-screw location, the rack
member's bonding point, and the standoff material and thread. None is measured yet.

The witness set, written now so the design is falsifiable before anything is printed:

| # | Condition | Required outcome |
|---|---|---|
| G1 | Motherboard absent | All accessible rack metal remains PE-bonded |
| G2 | Every motherboard standoff absent | PE continuity holds |
| G3 | One cassette extracted | PE continuity holds for the remaining rack |
| G4 | Board-hole bonding unknown | **No** board-to-chassis bond is claimed |
| G5 | Any printed polymer segment | Never carries PE continuity |

G4 is the one that is easy to skip: not knowing whether a mounting hole is bonded to board ground is
a reason to make no claim, not a reason to assume the convenient answer. A dedicated bonding stud,
conductor, terminal and tested connection are part of the rack design. Incidental contact through
rails or mounting screws is never the bond.

### The witnesses above are not sufficient, and the gap is a sequencing one

G1-G3 prove that the REMAINING rack stays bonded after something is removed. They say nothing about
whether the thing that was removed was correctly bonded while it was powered — a cassette can be
live, unbonded, and still leave a perfectly continuous rack behind it. Two ordering relations close
that, and both are about the service protocol rather than about geometry:

- **Power admission requires the bond.** `NodePowerAdmitted -> EveryRequiredAccessibleMetalPartPeBonded`.
  A node may not be energised unless every accessible conductive part it brings is already bonded.
- **Bond removal requires the power to be gone.** `PeBondMayBeRemoved -> AcDisconnected AND HazardousEnergyAbsent`.
  The earth connection is the last thing disconnected and the first thing connected.

This is why the bond is a captive bolt in the connect/disconnect sequence even though the docking
interface carries no PE obligation. "A dock is a connector, PE is a bolt" settles what the MECHANISM
is; it does not settle WHEN the bolt is made and broken, and the second question is the one that
decides whether a person servicing a live rack is safe.

### Front-edge support — admitted only under all five conjuncts

The 29.21 mm of board hanging past the last mounting row is a cassette input, not a description. A
support there is a *candidate*, and it is admitted only when all of the following hold, because each
one of them independently turns a helpful support into a defect:

1. The underside keep-out at the contact region is known.
2. The contact region is non-conductive.
3. The support does not obstruct the extraction path (R-middle-node-removal).
4. The support actually reacts connector insertion load — otherwise it is decoration.
5. The support does not become an **unintended electrical bond**, which is where this requirement
   meets the grounding correction above: a support that quietly bonds the board underside to the
   cassette creates a path nobody modelled and nobody tests.

## Cable routing — R12, at every layer

Every run gets endpoints, an access envelope, a minimum bend and service allowance, fixed clip
positions, strain relief, a moving-versus-fixed segment classification, and declared separation from
fan blades and hot surfaces. The service witness stays structural: after disconnecting the declared
external bundle and releasing the latch, the extraction path is collision-free and no neighbouring
cable or node has to move.

**PST changes the fan harness.** The ARCTIC P8 PWM PST carries a 4-pin connector AND a 4-pin socket,
so fans daisy-chain and fan count is decoupled from the board's five headers. That is a cardinality
fact only: a chain still owes admission against header continuous and startup current, connector and
wire rating, maximum chain length, failure isolation, and tach semantics. At the published 0.09 A a
three-fan chain draws 0.27 A and a five-fan chain 0.45 A steady — neither figure proves a chain safe,
because the header rating and startup behaviour are unmeasured. Do not assume every chained fan is
independently observable just because a downstream socket exists; tach forwarding needs manufacturer
authority or an executed electrical observation.

## The GPU-bearing node — how a card changes the cassette's structure

A node carrying a full-height dual-slot card is a **variant of the node contract**, not a second
product. What it changes is not "make it bigger": it converts two of the cassette's founding
assumptions.

**1. Distributed load becomes a concentrated cantilever.** The board is ~1 kg spread over standoffs;
a dual-slot card is ~1.5-2.5 kg hanging off one slot at one end. The existing standoff model cannot
carry it, and neither can the PCIe connector. The node needs a **second load termination** — a card
support spanning to the cassette frame, with the printed part as an alignment pad rather than the
retention. This is the same law the GPU augmentation program states, arriving inside the chassis:
*printed parts locate, metal retains.*

**2. The stack load path is sized by the heaviest node, not the average one.** "Remove a node from
the middle without lifting the others" means every shell carries the stack above it through its own
sidewalls. A GPU node is the heaviest member, so it sets the sidewall section for **every** node in a
mixed stack — or the model must carry a per-node mass and refuse a stacking order whose lower
members cannot carry what is above them. The second is the honest version and it is a real
refusal the current model cannot express.

**3. One airflow domain becomes two, and the second one exhausts sideways.** The cassette channels
80 mm fan air across the DIMMs. A blower card pulls its own air and exhausts out the bracket — which
in an OPEN, stacked chassis discharges into whatever is adjacent, including the intake of the
neighbouring node. **Inter-node recirculation is a failure mode the single-node airflow model does
not have.** It forces a declared exhaust destination per node and the rule that one node's exhaust is
never another's intake — which in practice means a consistent front-intake/rear-exhaust convention
and a duct from the blower to the rear face, even though the chassis is otherwise open.

**4. Grounding gets materially harder, not incrementally.** The standing requirement is a ground for
each board. A card bracket normally grounds through a metal chassis; in a printed, insulating
cassette there is **no such path** unless one is modeled. The GPU does not create this gap, it
exposes it — the bracket is simply the first component that assumes a conductive chassis and finds
none.

**5. Service becomes node-scoped.** A mid-stack node must extract with its card, cables and service
loops intact, so disconnect order and bend volumes become properties of the node, not the rack.

### The modeling consequence, and it is the operator's ruling applied

The shared authority is the **logical** structure — where load goes, where air goes, where fasteners
and interfaces sit. The **process** is an input to realization, not a fork of the design: PLA wants
thicker walls and rewards ribs; sheet metal is one folded surface that forbids enclosed ribs and
bosses and requires a bend radius and relief at every fold; molding wants uniform walls, draft and no
undercuts. So the process-specific part is **more than thickness**, and the difference must live in a
realization bound to a `FabricationProcess`, never in a hand-edited second model.

That is the same shape `gunbc.product.printed_chassis.realization_contract` already has for CadQuery,
and it says the emitter must not be forked per process: one logical model, N bound handlers, STL for
the printer and DXF for the laser shop derived from **one** source. Forking the emitter per process
is §3's fused-transport tell exactly.

### Open facts this cannot proceed without

Exact card envelope, mass, slot width and bracket geometry · blower exhaust direction and volume ·
node pitch (which decides upright versus flat-on-riser, and the riser makes it a cable problem) ·
aux power connector type and sustained draw against the node supply and stack distribution · whether
a mixed stack is allowed at all, since a uniform GPU stack and a mixed stack have different sidewall
answers.

### Batch follow-up: AMS and printer-02 storage

On 2026-10-03 the operator confirmed AMS physical slot 1 on both printers and identified
printer-02 error 83902511 (0x0500402f) as a microSD error. The operator was unsure whether
that card was in use. This LAN route uploads to SD storage and starts a `file:///sdcard/`
project, so the error remains a start blocker; it is not suppressed. No formatting or deletion
of printer storage is authorized or performed. The start payload now selects AMS and maps
the single project filament to tray 0; the signed operator request includes this feed choice.

The operator subsequently reported formatting printer-02’s card locally. A fresh no-error
report is still required before its start. Remote formatting was requested as a future capability;
the current modeled workflow has no format operation, and support on these printers is unverified.

At 22:01 UTC printer-02's AMS request was approved. Its fresh report admitted IDLE/error 0,
but FTPS upload returned curl exit 25 / server 550 before MQTT start. Its durable batch marker
is retained; there is no automatic replay. Printer-01's approved attempt received a partial
temperature-only report and correctly refused before upload. The observer now allows up to
eight fresh messages to obtain state and error together, without inventing missing fields.

The operator confirmed that printer-02 recovered and appeared functional after formatting.
A separate after-format attempt retains the original failed upload marker and requests new
approval. No MQTT start had been sent in the failed attempt. Status observation now subscribes
before publishing a read-only `pushing.pushall` snapshot request with Mosquitto request/response;
it still requires an explicit complete state/error report and cleans the private credential files.

Correction at 22:12 UTC: the operator reports printer-02 still shows SD card “error” in
settings despite clearing print_error. Stop its after-format attempt before actuation and
prioritize printer-01. A zero print_error is not proof of usable storage.

### Printer-01 start observed by operator

At approximately 22:15 UTC the full-snapshot gate admitted printer-01 and its holes 3–4
project uploaded successfully. MQTT publication did not return PUBACK. The operator then
confirmed the machine was heating or calibrating. The publisher had already exited with
connection-lost code 7 and cleaned its credential/CA directories before termination was attempted.
The transport outcome is UNKNOWN; the batch marker is retained and no start replay is permitted.
This is operator-observed preparation, not yet a telemetry-confirmed extrusion or completed print.
The selected feed is AMS physical slot 1; slice estimate 19m22s. Printer-02 remains held for
SD-card settings error, despite its zero print_error report.

At 22:19 UTC a fresh authenticated snapshot confirmed printer-01 `RUNNING`, `print_error=0`,
and subtask name `gunbc-28a8700489cf908edb9ed59c0aa871fe6903b4d558c3b64de6a7636af701580e.3mf`.
It reported 4%, 18 minutes remaining, layer 0/15 (preparation), nozzle 176 °C and bed 62 °C.
This reconciles the unknown MQTT transport result with observed execution of the intended gauge;
it does not establish completed extrusion or a finished part. No duplicate start was sent.

### Printer-02 recovery retry

The operator subsequently reported printer-02's card working and requested another attempt,
supplying a fresh one-hour token. The previously stopped after-format attempt had no start claim
and had sent no printer command. At approximately 22:31 UTC its new signed approval was observed,
the fresh full-state check admitted the printer, and the holes 4–5 file uploaded successfully.
AMS physical slot 1 remains selected. MQTT publication is awaiting reconciliation; it is not
replayed. The earlier FTP-550 attempt and its claim remain separate and retained.

At approximately 22:36 UTC, fresh telemetry confirmed printer-02 RUNNING/error 0 with the
expected `gunbc-a8f434e80014b6a316d30665b372bd0df3e858a68a2ff04a5fe4f50031d68535.3mf`
subtask, 38%, layer 0/15, and 8 minutes remaining. The upload/start route now has physical
execution evidence after the card recovery; this is not a long-term card-health qualification.

The operator reported printer-01 finished the holes 3–4 gauge, both holes line up well,
and the part was removed / bed cleared. Record C–H as an operator-reported alignment pass,
without inferring load-bearing qualification. Prepare holes 3–6 (C–B) as printer-01's next gauge.

The operator authorized the next printer-01 print after confirming C–H fit and a clear bed.
Holes 3–6 (C–B) sliced through the verified Orca entry with the same selected profiles.
Readback: SHA-256 `fb6d4bd1fcb23c07087b5575e6f5b20619f7ab8da03217828148d84cfcefc09f`,
15 layers / 3 mm, identity scaling, one plate, valid embedded G-code MD5 and ZIP integrity,
brim bounds [57.211684, 80.071684, 122.788316, 99.928316] mm, 12m10s / 2.59 g PLA.
The slicer metadata carries `bed_temperature_too_high_than_filament` at the upstream Generic PLA
textured-plate setting of 65 °C; this setting matches the successfully printed/aligned prior gauge.
It is retained as an observed warning, not described as warning-free. The new request binds this
project digest and a separate holes-3-6 attempt; printer-02's running job is not touched.

The other remaining candidates are also sliced and validated under
`/home/briansrls/print-prep-2026-10-03/next-batch/` (no start requested for either yet):

| Gauge | SHA-256 | Estimate | PLA |
|---|---|---|---|
| 5–2 (L–M) | `607601f18602ae273472654c01b9d34ca1674b7f6f5356c7136a382af41f545b` | 19m34s | 7.10 g |
| 4–7 (H–R) | `972ec6278c3334e070d4bd31884ce781211454c1c32296652e5f9a6ae4269252` | 13m26s | 3.42 g |

Both passed ZIP/G-code checksum, one-plate, 15-layer/3-mm, identity-transform, A1 mini/PLA/0.4-mm
profile, and 180-mm bed-bound checks. L–M including brim spans X=1.331684–178.668316 mm; no
scaling was applied. Both retain the same 65 °C bed-temperature warning described above.

### Operator direction: stop the pair-gauge sequence

The operator requested stopping further gauge printing and moving toward chassis/rack parts.
Do not enqueue the prepared 5–2 or 4–7 gauges. A fresh snapshot already confirmed the newly
started printer-01 3–6 job RUNNING/error 0 with its exact digest filename and 12 minutes remaining;
the operator explicitly chose to let that active print finish. No cancellation is requested.
The operator expects the remaining positions to be accurate; retain that as an expectation, not
a measured fit result. 1–3 and 3–4 have reported alignment passes.

Repository inspection found no printable full chassis/cassette/rack-bay geometry. Existing concrete
geometry covers coupons, standoff gauges and a fan-mount plate; rack_mount models occupancy of
catalog hardware, not printable rack construction. The proposed next geometry is a segmented,
adjustable motherboard carrier for unpowered assembly/fit, forming the first chassis prototype.
Remaining mounting positions can stay adjustable rather than requiring more pair gauges. Actual
standoff thread/height and underside clearance still determine the board-support interface;
PSU/cooler envelopes, structural material and load/joint checks remain for the powered cassette.

### Hosted concept workbench

The operator accepted the requirements and requested a hosted site for iterating on the chassis,
including independent 3D rotation of each piece. Published privately:
[Altra Chassis Workbench](https://altra-chassis-workbench.briansrls448156.chatgpt.site).
It contains an explicitly provisional four-panel carrier, segmented rails, handle and retaining
tabs, with a 2×2 fixed-frame study. Individual isolation/orbit, explosion, withdrawal and design
controls are available. There are no print controls or manufacturing exports.
The missing hardware dimensions remain unknown; the geometry is a visual proposal, not an
admitted fabrication result. Source project details are recorded in `sites/README.md`.
Desktop/mobile rendering and the principal interactions were exercised successfully.
All temporary copies of the operator's GCP token were removed after printing work ended.

### Cassette hardware and airflow study — 4 October 2026

Operator clarification: the target cooler is Dynatron (W1 in the preceding question), and PSUs
must be interchangeable rather than tied to one model. Roughly 400–600 W was an example range,
not a finalized electrical qualification. PSU compatibility must include accepted physical envelope,
mounting adapters, connector requirements, intake/exhaust clearance and service access; wattage
alone is insufficient. The inventory's unread PSU identities remain unread.

Concept 02 adds a rear cassette-resident adjustable metal PSU cradle, a variable PSU envelope,
Dynatron W1 geometry, DIMM clearance zones, one to three front 80 mm fans with individual carrier
feet, side airflow guides, a schematic internal harness corridor and front-to-rear direction arrows.
The operator proposed cassette-mounted fans. All these components travel with the cassette in the
2×2 frame view. Three fans is a layout default, not an established cooling requirement.

The [Dynatron manufacturer page](https://www.dynatron.co/product-page/w1), checked 4 October,
publishes active dimensions 141.1 × 84 × 72 mm and passive dimensions 116 × 84 × 72 mm.
The study depicts the active envelope with simplified fins/fan; active versus passive operation,
actual socket position/orientation and installed offset are not established by that depiction.
The P8 PWM PST uses its catalog 80 × 80 × 25 mm envelope; the existing 71.5 mm hole pitch and
4.4 mm clearance remain convention-unverified. Fan count, ducting and header/PST-chain electrical
admission still require evidence. Airflow arrows are not simulation results.

PSU slider defaults (150 × 85 × 150 mm), DIMM zones, the 50 mm rear service gap, adjustable cradle
interfaces and harness route are design proposals. They are not measured hardware or a claim that
all 400–600 W PSUs fit. No drive or expansion-card configuration has been invented. A height check
flags simple study-pitch conflicts; it does not prove full assembly clearance or powered suitability.

Source commit: `8c944389c0dbcb25639d4c6069587863ae0d0fd5` in the separate Sites repository.
Browser checks passed for selectable parts, PSU isolation, fan-count changes, height-conflict
feedback, airflow visibility, orbit, rack withdrawal, requirements and mobile overflow.
No additional gauge or chassis print was started.

### Shared power accepted — supersedes cassette-resident PSU

On 4 October 2026 the operator accepted shared power per four-node block. This replaces the earlier
cassette-resident PSU requirement and its per-node AC bundle. Eight nodes comprise two independently
powered four-node blocks. Each block has a fixed-frame 12 V supply system and four separately
protected, isolatable and disconnectable node feeds. Cassettes retain their board, cooler and fans.
The earlier roughly 400–600 W example concerned individual supplies and is not a shared-block rating.

[ASRock manual §2.8](https://download.asrock.com/Manual/ALTRAD8UD-1L2T.pdf) documents direct +12 V
DC input and distinguishes it from the ATX signal-adapter connection. This establishes a supported
input mode, not a qualified multi-node distribution system. Required power connections, independent
startup/shutdown behaviour, aggregate load/startup demand, branch fault protection, wire/connector
ratings, voltage drop, protective-earth continuity and service isolation still require engineering.
No connector pinout, fuse size, PSU wattage, redundancy or live hot-swap capability is selected here.
A nonredundant shared supply is a four-node failure domain; redundancy remains an open decision.

Concept 03 depicts one fixed top power shelf, one adjustable shared PSU space reservation, one
four-branch distribution placeholder and stationary branch harnesses. A DC disconnect placeholder
travels with each cassette. Withdrawal illustrates the node shut down and its branch isolated and
disconnected first. Top-shelf placement and all distribution dimensions are visual proposals. Shared
supply height is additional to the two node rows; cassette pitch is governed by cassette hardware.
Older Concept 02 notes above are retained as history and are superseded for power topology.

Concept 03 source: `7f77b6884f23af034fd1632e5918c3a21e988e5a` in the Sites repository.
Browser checks passed: shared PSU isolation, no PSU on the cassette-only view, fan-count controls,
height feedback, airflow visibility, rack withdrawal, requirements and mobile layout.

### Rear exhaust and first fan-mount print — 4 October 2026

The operator requested moving fans to the rear to keep hands at the front handle away from them,
and authorized modeling/printing a first part. Rear cassette-mounted exhaust is the new layout;
front-to-rear airflow remains. Rear I/O/cable access and a finger guard remain unresolved. Both
beds were reported clear; printer-01 is selected with the previously confirmed PLA in AMS slot 1.

The first real part reuses `fan_mount.arctic_p8_fan_mount`: 92 × 92 × 4 mm plate, 76 mm aperture,
71.5 mm hole pitch and 4.4 mm mounting holes. This is a useful fan mounting plate for unpowered
assembly fit, not another motherboard spacing gauge and not a complete cassette. Cassette
attachment and guarding have not been fabricated. Hole-pattern evidence remains convention-unverified.
`fan_mount_print.main` carries the existing admitted geometry through the existing CadQuery emitter
and exercised STL export settings; there are no independently authored CAD coordinates.

Generation and verification artifacts are in `/home/briansrls/print-prep-2026-10-04/rear-fan-mount`.
CadQuery 2.8.0 checks passed for one valid solid, dimensions, all five hole axes/radii, analytic volume,
STEP round-trip and a closed manifold STL. STL SHA-256:
`2b0c7df7e8cf79dc772a33ee6f62a6ccf9e5568ef07e0309545313480e35b4f8`.
Orca 2.4.2 sliced through `slice_run.run` at 0.20 mm, 20 layers, PLA/0.4 mm nozzle, 220 °C nozzle,
65 °C textured bed and 2 mm outer brim. Estimate 28m41s, 11.40 g. ZIP/G-code checksum, one plate,
identity scaling and 180 mm bed bounds passed; bounds including brim are 42.071684–137.928316 mm
on both axes. The retained bed-temperature warning matches the previously exercised 65 °C setting.
The slice_info first-layer-time metadata is malformed; G-code estimates 6m4s and that metadata is
not an admission input. Project SHA-256:
`71da943ff2ed73a6b47c54759ba288e82af7d084c269bf9a413d4d901cf5199c`.

Concept 04 is published with rear fans and downloadable generated STL/STEP, source
`c2a49273da70f20a23e2e044fe552dceb74b773f`. Browser interaction checks passed.
The operator-token/ntfy route is being used because PR #13149 remains unmerged. The request binds
printer-01, credential version 1, exact project digest and attempt `rear-fan-mount-20261004-01`.
A start result must be observed separately; preparation is not evidence of printing.

The first launch refused before submission because its submission-key environment variable was
missing; it filed no approval and created no start marker. Corrected operator launches on srv1
require these existing key paths (never their contents) plus the private token-file path:

- `GUNBC_APPROVAL_SUBMISSION_MAC_KEY_FILE=/etc/gunbc-roadmap/approval-submission-mac-key`
- `GUNBC_APPROVAL_CAPABILITY_MAC_KEY_FILE=/etc/gunbc-roadmap/approval-mac-key`
- `GUNBC_APPROVAL_STORE_RECEIPT_MAC_KEY_FILE=/etc/gunbc-roadmap/approval-store-receipt-mac-key`
- `RUNNER_TEMP` pointing to an owner-only staging directory.

The refusal log is retained as `print-01-preflight-refusal.log` under the srv1 run directory.
Retrying the same attempt before any request/start marker exists does not replay a printer start.

The corrected run passed the signed approval gate, observed the printer ready, claimed its durable
start marker, uploaded the exact project and sent one AMS-slot-1 start. MQTT publication lost its
connection without acknowledgement after two minutes, so the effect was classified UNKNOWN and
was not replayed. A separate modeled `printer_report.run` observation then returned RUNNING,
print_error 0, exact digest filename `gunbc-71da943ff2ed73a6b47c54759ba288e82af7d084c269bf9a413d4d901cf5199c.3mf`,
19% and 23 minutes remaining. This confirms the requested job active; layer_num was 0, so it is
not yet evidence of deposited layers or completed fit. The snapshot is retained locally and on srv1.
The operator independently reported probably heating. Printer-02 was not started. Temporary copies
of the operator token were deleted from both hosts.

### Complete logical cassette review — 4 October 2026

The operator reports that the printed rear fan plate fits. Record this as a physical fit
observation, not a caliper measurement or a promotion of the upstream conventional hole pattern.
The operator requests the whole cassette model now, review of all pieces together, then the first
cassette print. Stop the individual gauge sequence. No printer start is authorized by this review
artifact generation alone; the next print follows assembly review and the existing print workflow.

Review 05 replaces browser-authored mechanical geometry with `product.printed_chassis.cassette`
and `rack_assembly`. `assembly_geometry` carries typed box/cylinder CSG, micrometre coordinates,
part kind, fixed/moving scope, print orientation and mating-interface records. `cassette_review`
emits the full manifest. `review_realization` emits the generic CadQuery host adapter; it chooses
no mechanical dimensions and grants no fabrication admission. The old Site `concept.js` is removed.
Vendor board outline, fan plate/pattern, W1 envelope and P8 depth resolve from their existing owners.

One cassette has 49 printed instances / 29 distinct authored geometries: four vented tray panels,
four panel splices, four rails, two rail splices, a handle, two retaining stops, two rear extension
arms, two fan crossmember halves and a splice, three fitted-pattern fan plates, three feet, three
guards, two side airflow guides, four guide spacers and twelve guard spacers. The tray is
271.840 × 294.700 mm. Each piece fits the 180 mm A1 mini envelope in its declared orientation.
The two-level block retains a 140 mm bay pitch; this is a custom frame, not a claim of standard
2U/4U rack compliance. The fixed metal frame includes runner proposals. Shared 12 V supply,
four protected/isolatable feeds and distribution space remain fixed when a cassette is extracted.
Supply dimensions are explicitly space reservations; no supply wattage or circuit is selected.

The guides initially intersected the rail flanges and outer fan feet. The corrected model raises
the guide foot to 13 mm, adds front 8 mm / rear 4 mm spacers, and drills the outer foot for the shared
rear bolt stack. The front rests on the panel; the rear rests on the outer fan foot. This clears
the rail flange by 1 mm. Guard pads enclose the four fan fastener holes; upper 7 mm / lower 2 mm
spacers account for the 5 mm fan foot. Guard protection, fan bolt length and engagement still need
assembly review. All 56 distinct modeled fastener axes have a coaxial mate in another printed part.
This is geometric alignment evidence, not a fastener strength or engagement test.

Reproduction (from this worktree; use a memory-limited scope for the compiler):

```sh
systemd-run --user --scope -p MemoryMax=24G -p MemorySwapMax=0 gunbc run \
  --source-root dag --source-root src/v2 \
  --entry dag/gunbc/product/printed_chassis/cassette_review.dag --function main > model.json
systemd-run --user --scope -p MemoryMax=24G -p MemorySwapMax=0 gunbc run \
  --source-root dag --source-root src/v2 \
  --entry dag/gunbc/product/printed_chassis/review_realization.dag --function main > realize-review.py
python realize-review.py model.json fresh-output-directory
```

Use the exercised CadQuery 2.8.0 environment. Output includes bed-oriented STL for each printed
instance, local-coordinate STEP, cassette assembly STEP, viewer meshes, a manifest with digests,
parts/print-quantity/interface CSVs and one review ZIP. Generated artifacts and run logs are retained
under `/home/briansrls/print-prep-2026-10-04/cassette-r05`; the Site carries the review package.
The adapter refuses invalid/disconnected printed solids, cuts missing material, out-of-bed parts,
nonmanifold STL, STEP/STL bounds or volume divergence, unmatched fastener bores, orphan interface
members and intersections between printed parts. Kernel bounding boxes explicitly ignore cached
triangulation, while STL bounds allow the modeled tessellation deflection.

Remaining interfaces are deliberately represented as unresolved: board support thread/height and
underside keep-outs, loaded rail/joint/material behavior, real cooler pose and rear cable access,
fastener engagement, selected power/distribution hardware, thermal performance and unspecified
storage/expansion. No invented standoffs are provided. The first cassette is an unpowered assembly
fit, not a populated operational chassis. Review the complete parts and these interfaces together;
do not restart a gauge-by-gauge process.

Final exercised generation passed for all 49 printed pieces, 29 unique geometries, 56 matched
fastener axes and zero printed-part intersections. Negative controls separately refused an oversized
part, a disconnected solid, an unknown primitive, a displaced mating bore and an orphan interface.
The emitted adapter was the code actually executed. Local browser checks passed with the final
package; the Site shows the same generated mesh geometry as the downloadable CAD. No slicing,
approval request or printer start was performed in this review step. Reuse the already fitted fan
plate when scheduling quantities after review.

Review 05 published successfully at the existing private workbench, Site source `a9658fd05d27e83220c611b5fb00ac4397b74739`.
