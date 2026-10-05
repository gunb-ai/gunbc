# Cassette completion task queue

Owner: cassette worktree / PR #13295. Target: main. User-approved sequence, 4 October 2026.
This is the current queue; dated gauge and rack-frame plans are historical. No new gauge sequence, weighing request or printer start is scheduled here.

| ID | Task | Depends on | State |
| --- | --- | --- | --- |
| CAS-1 | Restore green required CI | — | Known declaration/catch-all failures corrected; validation pending |
| CAS-2 | Finish shared cassette generation | CAS-1 | Shared profiles and feature-aware minimum-cell allocation implemented; solid feature generalization open |
| CAS-3 | Size rear fans from cassette opening | CAS-2 | Selector, keyed 140 mm rear mount and guard modeled; geometry checks pass; cooling/retention qualification open |
| CAS-4 | Complete integrated stacking geometry | CAS-2, CAS-3 | Cassette-owned clip shoes, hollow posts and stack keys modeled; one/two-unit printed collision checks pass; load/retention qualification open |
| CAS-5 | Complete power cassette and harness layout | CAS-2, CAS-4 | Electrical/interface planning exists; physical placement open |
| CAS-6 | Validate final loads, airflow and materials | CAS-3, CAS-4, CAS-5 | Preliminary screens exist; final assembly qualification open |
| CAS-7 | Publish combined review and prepare remaining prints | CAS-6 | Tall-cassette joinery review updated; power layout and fabrication release remain open |
| CAS-8 | Desired-set print orchestration and plate packing | Existing CAD/slicing/start routes | Explicit-plate preparation/controller implemented for review; desired-set planning, recovery and live release outstanding |

## CAS-1 — Restore green CI

Correct the source reference in the Bambu filament scope and explicitly enumerate the closed variants in component mass, platform review and PSU capacity screening. Run the required floor and focused witnesses, fix subsequent findings, and require all GitHub checks on the pushed head. Do not weaken checks, register avoidable catch-all debt, or claim old-head results validate new code.

## CAS-2 — Shared cassette generation

One profile-driven generator supports ALTRAD8UD, GH200 P4261 / 694-24261-000-100 K1, Mt. Collins and Mt. Jade. Profiles own component/mounting evidence and retained power/management assemblies. The shared partition graph divides the board-plus-margin footprint into cells with one edge per adjacent pair, accounting for joint, rail/socket, handle, brim and bed-edge envelopes separately. The allocator derives minimum axis counts and balances cuts, allowing edge cells to be narrower. With the proposed 2 mm brim and 0.5 mm bed-edge clearance per side, Altra remains four equal cells; Jade and Collins use 3 × 4 planning cells. The old blanket 30 mm deduction produced 4 × 4 and is superseded.

Remaining: make the existing panel/joinery solid generator consume this graph, then parameterize supports, underside keep-outs, perimeter handles and sockets. Preserve the exact r06 Altra geometry as the printed compatibility fixture. The missing GH200 outline, provisional Collins outline and unverified mounting coordinates must not borrow Altra geometry or become printable through defaults. Accept when the same generator realizes both the Altra fixture and a larger synthetic profile, with bed/collision/mate checks and refusal controls.

## CAS-3 — Height-related rear cooling

User-selected policy: largest fitting standard fan, evaluating 140/120/92/80 mm candidates. Derive a single unobstructed opening after corner structure, connector and service reservations; do not add disconnected free areas. Check height, width, depth and individual-part print bed. Derive count and centered positions from width. Use vendor-specific hole patterns; never scale the 80 mm pattern.

The current selector retains the r06 80 mm assembly unchanged and reports an alternative rear bank in the stack review. ARCTIC publisher drawings were visually read: P14 125 mm pitch, P12 105 mm, F9 82.5 mm, all nominal 4.3 mm holes. These are manufacturer geometry, not printed-clearance compensation. Mount edge uses a proposed 6 mm margin each side. A conservative 40 mm fan-body reservation plus 20 mm mounting/service depth is a design allowance, not a vendor depth claim. Noctua variant height remains derived, not forced to 177.8 mm.

Remaining: generate removable, guarded rear modules and their attachment/airflow guides through the existing CAD path. A 231.84 × 190 mm unobstructed opening selects one 140 mm fan; a 310 × 132 mm opening selects two 120 mm fans. Geometric preference does not prove either bank is thermally adequate or quiet. Accept with fit-boundary, obstruction, depth/bed refusal and mount-solid checks, then airflow and header-current qualification in CAS-6.

## CAS-4 — Integrated milk-crate structure

Generate corner supports and stack keys as cassette-owned pieces, with no enclosing rack frame. Resolve interference with existing rear-arm sockets. Support interfaces and load paths must match before unlike profiles can stack. Minimize material and threaded joints; count assembly actions and fasteners in review output.

Model removal and cable swept volumes. A column carried by the removed cassette cannot remain supported by geometry that is leaving with it. Default service procedure: de-energize/disconnect the target and support or lift the units above before a middle pop. Do not claim independent drawer removal until a modeled alternate load path exists. Accept with mating/collision checks for two stacked units, a removal sequence and explicit supported/unsupported service states.

## CAS-5 — Modular power and harness placement

Keep a common stack attachment with PSU-specific replaceable carriers/backplanes. Default one PSU; retain the exact PSU/backplane/pinout/enable compatibility obligation. Reuse existing branch and aggregate budget checks. Native GH200/Collins/Jade power assemblies remain attached to their node profile where needed.

Place PSU, backplane, protected DC branches, mains enclosure/earth connection, connectors and strain relief. Cable census must follow actual endpoints, bends and service paths; front-edge board power and fan headers cannot disappear behind a rear-only route. Accept with cable/connector/service collisions, accessible disconnects, branch-load failures and retained-assembly envelopes accounted for.

## CAS-6 — Final physics and thermal qualification

Fold all installed components and generated/sliced part masses at their placements; retain researched ranges and missing-evidence standings. Include power cassette, wiring, joints and cooler transport loads. Re-evaluate corner reactions, buckling, tipping, interface shear and creep against final geometry and printed material/orientation. Preliminary Euler or rigid-body screens are not a certified stack limit.

Use selected fans' operating data with restriction/fan-failure cases, header-load limits and ambient/exhaust temperatures. CPU die temperature is not mount temperature. Generic PETG or Bambu PETG figures do not qualify SUNLU High Speed PETG. Keep material acceptance open until product-specific temperature/load evidence supports it. Accept with adverse controls and explicit remaining validation requirements; no invented measurements or operator weighing request.

## CAS-7 — Review and print release

Publish assembly and exploded per-piece rotation using the shared emitted model, including power/cable/clearance geometry. Show actual height, fan selection, BOM, assembly steps and qualification status. Compare with P1–P4 and the fitted fan mount; list reusable and superseded parts explicitly.

Only after geometry/solid/bed checks and appropriate material selection: export and slice remaining pieces for both A1 minis, record digests, mass and durations, and schedule a two-printer wave plan. Physical starts still require fresh bed-clear evidence and the reviewed authorization workflow. No new starts are implied by creating this queue.

## 140 mm joinery checkpoint — 4 October 2026

`rear_fan_assembly` now realizes the retained tray arms, two bed-sized crossmember halves, a keyed cradle, an integral-spacer guard and six broad-grip retaining pins. The P14 body uses its vendor 27 mm depth. Two diagonal M4 bolts capture plate, fan and guard; the modeled stack is 40 mm before washers/nuts, so final bolt length remains unresolved. The guard's nominal 7 mm openings are proposed geometry, not certified finger protection.

`stack_joinery` adds four cassette-owned clip shoes, hollow 20/12 mm posts split into lower/upper pieces, four middle-joint pins and keyed stack seats. No separate enclosing frame is present. The post wall area is 256 mm²; the shoe bearing area is only 127.75 mm². These must not be substituted for each other or for the old solid-pillar mechanics screen. Rear shoe relief clears the existing rear-arm pin heads. A middle pop explicitly requires independent support above it.

Validation: three new joinery witnesses passed. The shared CadQuery emitter produced 34 valid positive solids, including 28 connected printed solids, each under the 180 mm bed bound. Exact printed-solid intersection checks found no intersections above 0.01 mm³ within one cassette or between two at 230 mm pitch. These were manual realization checks, not a new durable CAD witness. P1–P4 are imported unchanged. The interactive review preserves piece isolation/rotation and adds a two-unit view. These results do not establish support-free slicing, clipping force, vibration retention, loaded lifting, creep, thermal suitability or a safe stack count.

Remaining before fabrication release: complete power and cable reservations, connect actual hollow/jointed support geometry to the load screen, qualify material/retention and cooling obligations, and slice selected parts. Shared larger-board panel-solid generation remains CAS-2 work; the profiles and partition graph alone do not make those boards printable. Required CI is still pending on the PR; focused local results are not a green merge gate.

## Provisional board references — 4 October 2026

The operator supplied a mechanical-source survey and authorized provisional inference. [Reference ledger](cassette-board-mechanical-references.md) records 13 Jade candidate mount positions derived from OCP Figure 1 and a separate provisional Collins outline (424.2 mm width × 512 mm depth). Collins now gets a 3 × 4 planning partition from the shared feature-aware allocator under the proposed process allowances. Hole-map completeness, feature identity and later verification remain explicit; no Jade pattern is substituted for Collins.

## Feature-aware cell allocation — 4 October 2026

`bed_partition` owns the axis allocation calculation, used both for board-only counts and actual panel-envelope planning. For two or more cells, capacity is first + last + (count − 2) × interior; this gives the minimum count directly. A logarithmic balancing search chooses cut widths, saturating constrained edge cells first. Integer remainders preserve exact coverage, and zero-width cells refuse. No repeated growing-list concatenation is used.

`tray_panel_features` owns the existing joint/rail/socket/handle dimensions; the retained Altra CAD and the allocator both read these values. This preserves the existing solids while removing the separate feature-budget estimate. `platform_projects_review` emits every cell's panel span and print envelope, including the proposed brim and bed-edge clearances.

| Profile | Planning grid | Column spans, mm | Row span, mm | Adjacent joins |
| --- | --- | --- | --- | --- |
| Altra | 2 × 2 | 135.92 / 135.92 | 147.35 | 4 |
| Collins | 3 × 4 | 148.5 / 151.85 / 151.85 | 135 | 17 |
| Jade | 3 × 4 | 148.5 / 153.75 / 153.75 | 126.84 | 17 |

This is 12 planned panels instead of 16 for Collins/Jade, and 17 adjacent joins instead of 24. It is the minimum axis-aligned grid under the declared side envelopes, not an unrestricted nesting optimum. A different brim, bed, feature envelope or board outline recomputes the count. Multi-cell solid realization remains open; no change to printer jobs or to fabrication admission follows from this plan.

Validation of this allocator change: six allocation checks, eight mechanical-reference/emitter checks and five platform checks passed. The combined review emitter succeeded. All 34 Altra part descriptions and its emitted CAD program are identical to the prior reviewed bundle, including P1–P4. Full required CI remains separate and pending.

## CAS-8 — Desired-set print orchestration

Requested next workflow milestone: [batch planning, dense plate packing and durable execution](print-batch-orchestration.md). BATCH-1 through BATCH-6 define the implementation and release checks. The [explicit-plate workflow foundation](print-workflow-integration.md) now owns preparation and fleet queue execution. Desired-set planning, accepted inventory, crash recovery and the live release checks remain outstanding; CAS-8 is not complete.

