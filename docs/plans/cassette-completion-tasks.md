# Cassette completion task queue

Owner: cassette worktree / PR #13295. Target: main. User-approved sequence, 4 October 2026.
This is the current queue; dated gauge and rack-frame plans are historical. No new gauge sequence, weighing request or printer start is scheduled here.

| ID | Task | Depends on | State |
| --- | --- | --- | --- |
| CAS-1 | Restore green required CI | — | Known declaration/catch-all failures corrected; validation pending |
| CAS-2 | Finish shared cassette generation | CAS-1 | Shared profiles, layout and bed-allocation graph implemented; solid feature generalization open |
| CAS-3 | Size rear fans from cassette opening | CAS-2 | Selector, keyed 140 mm rear mount and guard modeled; geometry checks pass; cooling/retention qualification open |
| CAS-4 | Complete integrated stacking geometry | CAS-2, CAS-3 | Cassette-owned clip shoes, hollow posts and stack keys modeled; one/two-unit printed collision checks pass; load/retention qualification open |
| CAS-5 | Complete power cassette and harness layout | CAS-2, CAS-4 | Electrical/interface planning exists; physical placement open |
| CAS-6 | Validate final loads, airflow and materials | CAS-3, CAS-4, CAS-5 | Preliminary screens exist; final assembly qualification open |
| CAS-7 | Publish combined review and prepare remaining prints | CAS-6 | Tall-cassette joinery review updated; power layout and fabrication release remain open |

## CAS-1 — Restore green CI

Correct the source reference in the Bambu filament scope and explicitly enumerate the closed variants in component mass, platform review and PSU capacity screening. Run the required floor and focused witnesses, fix subsequent findings, and require all GitHub checks on the pushed head. Do not weaken checks, register avoidable catch-all debt, or claim old-head results validate new code.

## CAS-2 — Shared cassette generation

One profile-driven generator supports ALTRAD8UD, GH200 P4261 / 694-24261-000-100 K1, Mt. Collins and Mt. Jade. Profiles own component/mounting evidence and retained power/management assemblies. The shared partition graph divides the board-plus-margin footprint into cells with one edge per adjacent pair, reserving a declared bed budget for local features. Altra remains four cells; Jade requires 4 × 4 at 180 mm beds with a 30 mm feature allowance (the earlier 3 × 3 number was board-only).

Remaining: make the existing panel/joinery solid generator consume this graph, then parameterize supports, underside keep-outs, perimeter handles and sockets. Preserve the exact r06 Altra geometry as the printed compatibility fixture. Missing GH200/Collins outlines and unknown mounting coordinates must not borrow Altra geometry or become printable through defaults. Accept when the same generator realizes both the Altra fixture and a larger synthetic profile, with bed/collision/mate checks and refusal controls.

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
