# Converging a selected cassette design to an accepted physical assembly

Status: migration in progress; preparation-bound inventory assessment, fenced inventory transitions and
inspection readback are implemented in shadow mode. See
[cassette-inventory-convergence-implementation.md](cassette-inventory-convergence-implementation.md)
for the implemented operations and remaining cutover work. This is not yet the
complete assembly admission mechanism. Active worktree:
`session/print-workflow-integration`, PR #13334, targeting main. This plan extends
CAS-3/5/6/8 and BATCH-1/5; it does not fork the cassette design or change active prints.
The earlier requirements/provider framing is retained as the assessment subject,
not a second convergence algebra.

## Current prototype scope

The operator specifies Ethernet and power as the external node connections, with
SSDs mounted on the motherboard. The baseline is one Ethernet connection and one
protected DC branch per node; the number of board-side power plugs still comes
from the existing power-feed calculation. Fan power/PWM and cooler fan wiring
remain internal connections and must be counted. Additional BMC Ethernet is an
explicit optional configuration, not a silently required second cable. On-board
SSDs add no external drive cable, but retain their clearance, retention, heat and
mass obligations.

Finish the existing fit prototype; no standoff retrofit is requested. Resting the
board on the tray for an unpowered layout review is a declared prototype state,
not satisfaction of the final board-support or powered-operation requirements.

## Existing authorities and gaps

| Authority | Retain | Change |
| --- | --- | --- |
| `assembly_geometry.AssemblyPart` | Geometry, identity, local/assembly frames, print plane | Attach typed interface contracts to the same component instance |
| `assembly_geometry.MechanicalInterface` | Explicit member relationships | Replace prose as the authority for executable requirements; derive display text |
| `cassette_profile.DesignObligation` | Outstanding profile-level information | Instantiate obligations against owners; do not delete unmatched obligations during migration |
| `server_power_interface.InterfaceObligation` | Electrical engineering requirements | Map to owned demands, including nongeometric electrical conditions |
| `server_power_interface.pairing_screen` | Named supply/branch allocation and capacity arithmetic | Feed it resolved assignments; do not duplicate its budget rules |
| `rack_assembly.hardware` clearance parts | Existing proposed reservations | Give each an owner and standing; a reservation is not a routed cable |
| `harness_census` | Census projection | Derive lengths and counts from selected components and routed connections |
| `assembly_review_checks.bore_has_mate` | Existing regression baseline | It checks infinite-axis coincidence and diameter only; add axial engagement and actual mate identity before calling it a mechanical fit |
| `exhaust_thermal` | Conditional thermal screens and measurement obligations | Keep installed airflow/temperature evidence distinct from geometric clearance |
| `process_geometry` / `deposited_geometry` | Print-plane admission and toolpath observations | Expose evidence through the shared report without weakening existing admission |

The harness and fan-header screen now receive selected fan facts explicitly; the
residential candidate carries directly cited P14 wiring facts. Deriving the
complete connection graph and routed harness remains a first-cut requirement.
`product.fabric.offer_route` provides a useful precedent for identity, missing
facts and ambiguity, but its provider-route policy is not a mechanical interface.
`std.resource_contract` describes compiler effects, not physical port capacity;
do not reuse its enums merely because they say capability/resource.

## Selection and convergence boundary

The loop is:

```text
intent + vendor facts + candidate evidence
  -> candidate production -> std.decision selection receipt
  -> versioned desired assembly and derived demand census
  -> goal-blind observation -> std.goal_assessment
  -> remedy selection/planning -> guarded actuation
  -> independent readback/inspection -> reassessment
```

Selection precedes each convergence scope. Board/fan/cooler choice, connection
selection, routing, inlet location and print orientation are explicit selected
realizations. Candidate producers enumerate alternatives; `std.decision` chooses
within the evidenced field and stated policy, producing `SelectedWithin` or its
actual unresolved/refused arms. Checks eliminate inadmissible candidates and
supply evidence; they do not silently choose the remaining policy. A Pareto front
is not a choice. If new evidence invalidates a choice, re-enter selection and
version the goal; do not quietly change it inside an actuator.

The desired state is derived from that selected configuration and intended use,
not from vendor facts alone. A board having two Ethernet ports does not require
both connected. The selected connection policy activates a port's demands; the
board authority supplies its physical/electrical facts. Selection and derived
requirements may constrain one another during candidate evaluation, but each
actuation consumes one fixed, identified selected goal.

`std.goal_assessment` owns remedy-free assessment now. Inhabit
`ObservationAttempt`, `GoalInspection` and `GoalAssessment`, including:

- `GoalSatisfied`: evidence establishes this scoped goal.
- `GoalDiverged`: a nonempty set of established differences.
- `GoalIndeterminate`: nonempty unknown facts **and any known differences**.
- `GoalAssessmentRefused`: the question is not valid/comparable (for example
  duplicate identities or an incompatible subject/revision).
- `ObservationRefused`: no usable snapshot; outside the assessment, not absence.

Use `assess_by_deviations_and_unknowns` where applicable. Display labels such as
open/violated are projections, not fresh competing verdict types. A known
violation must survive alongside unknown evidence. An open prerequisite refuses
an action that depends on it; it need not stop unrelated, independently admitted
work such as inspecting a different finished part.

`std.goal_assessment` deliberately owns no planner, actuator or post-actuation
cycle. DESIGN §3d and that module name the fleet admission spine as the first
consumer of the future shared effectful cycle. Bind cassette assessment now and
keep `gunbc.fleet.printer_batch` as the existing actuator; do not introduce a
cassette-local generic cycle. Before the cycle implementation, inspect
`gunbc.fleet_converge_plan` at `dag/gunbc/fleet/fleet_converge_plan.dag`. The shared
cycle lands with that spine as its required consuming integration, then the
cassette binds it in the same or a subsequent change. If it remains absent, mark
this portion open rather than declaring cassette-first completion.
`std.realization_reconcile` already classifies reported grounding, but does not
itself establish that a read happened or supply the missing guarded cycle.

## What counts as observation

Keep model/realization evidence and physical evidence separately attributed:

- Model consistency and independently read CAD/slicer artifacts establish claims
  about the design or generated instructions. A checked swept volume can satisfy
  a **modeled-clearance** goal under its stated inputs. It cannot establish the
  installed cable's placement or that the printed part survived.
- Physical inventory, accepted-part inspection, assembly observations and
  installed measurements establish claims about actual instances. Receipts bind
  subject, geometry/material/process compatibility, observation identity and time.
  A photo without sufficient detail leaves its affected claims indeterminate.

Do not force every design proposition to wait for a physical measurement; instead
make its subject and evidence requirements explicit. Conversely, never let a
rendered reservation or a successful command stand in for installed evidence.
Observers receive subject and independently scoped requests, not the goal.
Review capture and goal-derived-request paths too: an observer that only returns
wanted items cannot establish complete inventory or discover extra/mismatched
parts. Record snapshot coverage so an unread shelf is not reported empty.

## Desired demand, work and readback

Derive BATCH-1's versioned required-instance slots from the selected assembly's
manufacturable component occurrences referenced by the demand census. Several
demands satisfied by one component do not create several copies: occurrence
identity and selected quantity own multiplicity. Purchased parts, wiring, assembly and evidence collection
produce their appropriate work kinds; they are not all print jobs. Explicit
standalone print requests remain valid selected goals, but the cassette must not
maintain a second independently authored wanted-parts list.

The planner consumes assessment, inventory and outstanding attempts. Desired minus
accepted inventory is not sufficient: reserve instances already claimed, running,
uncertain, or completed awaiting inspection. Their unresolved state generates
reconciliation/inspection work, not replacement prints. A proven missing or
rejected instance can generate a replacement after remedy selection and admission.
A violation might instead require rerouting, purchasing, redesign or measurement.
No generic remediation follows from an indeterminate aggregate; separately scoped
work requires its own complete assessment and prerequisites.

Keep requirement-slot identity distinct from design revision, physical inventory
instance, plate plan and print attempt. Bind plans to the selected goal revision
and observed baseline; reject stale plans before new actuation. Geometry/process
changes do not erase old claims or silently reuse incompatible inventory.
Retain the existing controller hold, durable start claims, ntfy/bed-clear boundary
and uncertain-publication reconciliation. Derived plans alone do **not** guarantee
idempotent physical effects. BATCH-1's atomic multi-instance reservations remain
necessary; hashing a fresh plate or changing an output directory is not a replay
boundary.

After actuation, acquire readback of the exact subject independently of the
actuator's return. Printer `FINISH` means completed-awaiting-inspection. Acceptance
consumes an attributable physical inspection and satisfies only that instance's
applicable acceptance requirements. Assembly fit and powered-operation goals can
remain open. An uncertain acknowledgement retains its claim and is reconciled;
it is never a reason to resend automatically.

For the failed grille plate: **if** the posts/pins are accepted and M140/G140
rejected, the diff yields only two replacement needs. Recognizable parts in a
photo do not supply the missing acceptance receipts. In a repeated assessment
with the same goal, observations and reservations, no additional eligible work
appears. With new rejection evidence, only affected slots change. Progress is
conditional on evidence and achievable goals; this is not a guarantee that every
physical build eventually succeeds.

## One assembly report, checks for each domain

Use one owner-and-evidence relation, with a closed requirement payload whose
branches carry the actual domain quantities. Do not reduce all interfaces to
`kind + point + number`, or choose a provider by nearest distance.

Proposed carrier roles (names subject to the implementation's authority search):

- Component instance: stable identity, revision, selected configuration and rigid
  placement transform. Local connector positions and directions are transformed
  together. Exploded-view offsets never affect engineering checks.
- Port: owner, port identity, local mating frame, connector/contact contract,
  direction, capacity and source standing. A port is not automatically a provider
  for every demand of its kind.
- Demand: owner, lifecycle/use case, typed predicate and evidence needed. The
  owner may be a component, joint, route, assembly or fabrication instance.
- Binding: explicit named demand/provider endpoints, quantity and chosen route
  or joint. Ambiguous candidates remain ambiguous until an assignment is made.
- Desired assembly: selected revision plus derived owner-bound requirements and
  instance quantities; one authority for both review and requested work.
- Observed assembly: scoped model/artifact or physical snapshot with source,
  coverage, identity and standing; absence must be observed, not defaulted.
- Evaluation: the existing `std.goal_assessment` result with domain evidence,
  deviations, unknown facts and comparison refusals. Render missing evidence with
  a concrete closure trigger and deviations with located causes.

Preserve unknown source coordinates and provisional readings. Purchased-part
facts belong with their vendor authority when actually published. Coordinates
inferred from an image are repository observations with their own standing, not
new vendor facts. Installation direction is a product placement decision even
when the vendor supplies the fan's local airflow direction.

The assembly evaluator indexes placed owners/ports once, checks bindings,
accumulates shared capacity use, then evaluates local and cross-part constraints.
Candidate route search is a separate producer; `std.decision` selects a candidate
with a receipt, and the evaluator checks the selected result.
This is one report/fold boundary, not a claim that every constraint is a local
pairwise comparison or that a greedy single pass finds a valid layout. Stable
identities make findings independent of input ordering. Duplicate identities,
missing owners, multiply occupied exclusive ports and overdrawn shared capacity
produce explicit findings.

## Domain checks

**Mechanical:** declared bore/pin/fastener roles, mating axis and tolerance,
finite axial intervals, grip/engagement length, bearing faces, required contacts
and permitted contact pairs. Two distant coaxial bores do not prove attachment;
an arbitrary round solid is not automatically a compatible pin. Geometric contact
also does not prove joint strength or load capacity.

**Electrical and network:** connector family/keying/gender, pinout, rail/polarity,
current capacity, protective branch, and signal/protocol compatibility. Geometric
mating alone does not establish electrical compatibility. Board input population
and aggregate PSU/header current continue to use their existing authorities.
One external DC branch may require several board-side connections.

**Cable route:** explicit endpoints, connector-body/boot envelopes, centerline,
outer diameter, bend constraints, retention/strain relief, unplug/latch access and
service slack. Sweep the actual cable section and its connector/service envelopes
against physical solids, fan rotor space and other routes. Allow only declared
endpoint/contact exceptions; neither ignore all clearances nor reject intended
mating contacts. Minimum bend radius is checkable when the cable specification
and route geometry are known; absent data yields open, not an inherently
undecidable verdict. Derive cut length only when the routed path, termination and
slack allowances are all known; otherwise retain an estimate standing.

**Airflow:** transform the selected fan's inlet/outlet and flow direction, identify
rotor exclusion and inlet/exhaust space, and check route/structure obstruction.
Separate an unobstructed opening from delivered airflow: catalog free-air flow
does not prove installed flow through a grille, cooler and cable bundle. Preserve
thermal/pressure/fan-failure qualification as separate evidence requirements.

**Fabrication:** a plate/process instance realizes a component instance, with the
admitted orientation, support geometry, bed envelope and applicable process
evidence. Preserve the current off-plane refusal until a real qualified departure
producer exists; adding a prose reason does not satisfy stability. Retraction
observations describe demand on the process, but geometry alone cannot determine
a correct retraction distance or demonstrate adequate cooling.

**Service:** evaluate installed and removal configurations separately. A cable that
fits when stationary may tether a cassette during removal. Require named
disconnects, accessible latches, service motion and slack; do not infer a supported
middle-stack removal from the static assembly.

## First routing candidate to evaluate

Keep rear Ethernet near its board port, escaping sideways around the fan cradle
to a retained side route. Select a protected node DC inlet and route power along
an edge toward the board's front-edge inputs. Route the rear fan's power/PWM lead
to a compatible front-edge fan header, with a selected extension if needed.
Cooler leads stay clear of both fans. Left/right routing and DC inlet location
are candidate decisions selected with explicit policy and evidence receipts after
endpoint, clearance and service screening. No route is selected here, and no loose
cable is accepted through the fan rotor exclusion.

The existing 50 mm I/O reservation is only a proposed space budget. It cannot
stand in for connector bodies, cable bends or insertion/removal access. Use the
existing rear-fan prototype as a regression fixture, and show unresolved connector
locations rather than inventing exact coordinates to make a route pass.

## Migration and acceptance order

0. **Selection receipts and scopes.** Name the goal subject, intended prototype or
   operation scope, decision field, hard constraints, evidence and selection policy.
   Bind board/fan/cooler, connections, routes and fabrication choices through
   `std.decision`; carry unresolved choices without fabricated defaults. Give each
   convergence assessment its fixed selected goal and revision.
1. **Inventory and identity.** Attach every existing interface/obligation to its
   owner and use case. Inventory the selected board, P14, cooler, onboard storage,
   DC branch/adapter and Ethernet lead. Map every old obligation to an executable
   demand or an explicit open item, with no silently dropped rows.
2. **Join and assessment.** Bind named relationships and the report to
   `std.goal_assessment`, preserving observation failure, unknown facts, known
   deviations and malformed questions. Cut the
   mechanical review root over with controls for valid mates, distant coaxial
   bores, duplicate/ambiguous providers, missing owners and occupied ports.
   Preserve current geometry checks during replacement; retire the old matching
   path only when its replacement is used by the production root.
3. **Ethernet/DC/fan routes.** Place evidenced endpoints and proposed routes. Add
   swept-volume and insertion-access checks; feed resolved power assignments to
   `pairing_screen`. Replace P8-specific harness counting with the selected
   component/connection graph. No drive cable is added for onboard storage.
4. **Derived census and review.** Render routes, endpoints, airflow arrows,
   exclusion/service spaces and located open/violated demands in the existing
   review. Derive cable BOM/lengths from those same routes. Show the owner and
   evidence behind each finding; remove independently authored reservations/counts.
5. **Derived work and guarded actuation.** Make BATCH-1 consume the canonical
   assembly-derived instance slots, inventory and durable reservations. Remove its
   independent cassette demand authoring at cutover. Route print work through the
   existing batch controller, and represent assembly/inspection work explicitly.
   Test partial failure, stale revisions and uncertain starts without replay.
6. **Physical readback and reassessment.** Consume per-instance inspection receipts
   from BATCH-5; do not leave completion's inspection-pending state terminal to the
   overall assembly program. Reassess with new observations. Join current
   print-plane/support/readback evidence at its design/process scope. Separate
   view/fit-prototype, fabrication and powered-operation policies; board-support
   concessions cannot grant powered operation. Do not alter running jobs.

Use real assembled/sliced producers as integration controls, not only fixtures
that echo authored values. Required negative controls include reversed fan,
cable crossing rotor space, impossible bend, blocked latch, wrong power pinout,
overdrawn shared supply/header, insufficient lead length and an upright fan-grille
print without qualified process evidence. Change or remove a provider and verify
the downstream report, census and relevant admission change together.

Done means the existing cassette review and preparation roots consume this model;
a second obligation list or a standalone visualization does not complete it.
Delete prose requirements and old enum carriers only after their complete mapping
and consumer cutover. Keep explanatory prose as derived/help text. Do not add a
broad required-CI prefix or new job as part of this migration.

Convergence controls additionally distinguish unread inventory from known absence,
retain known violations beside unknowns, reject a wrong-revision inspection, and
show that FINISH or a successful start cannot mint physical acceptance. Concurrent
planners and restarts must retain reservations; partial acceptance must preserve
accepted siblings. The production route must consume inspection and reassess,
otherwise the new loop remains a plan rather than an implemented convergence path.

## Cassette completion task queue

Owner: cassette worktree / PR #13295. Target: main. User-approved sequence, 4 October 2026.
This is the current queue; dated gauge and rack-frame plans are historical. No new gauge sequence, weighing request or printer start is scheduled here.

5 October follow-up: the assembly convergence sections above
defines selection, derived demands, assessment, guarded actuation and independent
readback for CAS-3/5/6/8 and BATCH-1/5 on PR #13334. Current external
connections are Ethernet and power; storage is on-board. The existing fit prototype
does not require a standoff retrofit. The plan's owner-bound requirements, cable
routes and domain checks are not yet implemented.

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

### CAS-1 — Restore green CI

Correct the source reference in the Bambu filament scope and explicitly enumerate the closed variants in component mass, platform review and PSU capacity screening. Run the required floor and focused witnesses, fix subsequent findings, and require all GitHub checks on the pushed head. Do not weaken checks, register avoidable catch-all debt, or claim old-head results validate new code.

### CAS-2 — Shared cassette generation

One profile-driven generator supports ALTRAD8UD, GH200 P4261 / 694-24261-000-100 K1, Mt. Collins and Mt. Jade. Profiles own component/mounting evidence and retained power/management assemblies. The shared partition graph divides the board-plus-margin footprint into cells with one edge per adjacent pair, accounting for joint, rail/socket, handle, brim and bed-edge envelopes separately. The allocator derives minimum axis counts and balances cuts, allowing edge cells to be narrower. With the proposed 2 mm brim and 0.5 mm bed-edge clearance per side, Altra remains four equal cells; Jade and Collins use 3 × 4 planning cells. The old blanket 30 mm deduction produced 4 × 4 and is superseded.

Remaining: make the existing panel/joinery solid generator consume this graph, then parameterize supports, underside keep-outs, perimeter handles and sockets. Preserve the exact r06 Altra geometry as the printed compatibility fixture. The missing GH200 outline, provisional Collins outline and unverified mounting coordinates must not borrow Altra geometry or become printable through defaults. Accept when the same generator realizes both the Altra fixture and a larger synthetic profile, with bed/collision/mate checks and refusal controls.

### CAS-3 — Height-related rear cooling

User-selected policy: largest fitting standard fan, evaluating 140/120/92/80 mm candidates. Derive a single unobstructed opening after corner structure, connector and service reservations; do not add disconnected free areas. Check height, width, depth and individual-part print bed. Derive count and centered positions from width. Use vendor-specific hole patterns; never scale the 80 mm pattern.

The current selector retains the r06 80 mm assembly unchanged and reports an alternative rear bank in the stack review. ARCTIC publisher drawings were visually read: P14 125 mm pitch, P12 105 mm, F9 82.5 mm, all nominal 4.3 mm holes. These are manufacturer geometry, not printed-clearance compensation. Mount edge uses a proposed 6 mm margin each side. A conservative 40 mm fan-body reservation plus 20 mm mounting/service depth is a design allowance, not a vendor depth claim. Noctua variant height remains derived, not forced to 177.8 mm.

Remaining: generate removable, guarded rear modules and their attachment/airflow guides through the existing CAD path. A 231.84 × 190 mm unobstructed opening selects one 140 mm fan; a 310 × 132 mm opening selects two 120 mm fans. Geometric preference does not prove either bank is thermally adequate or quiet. Accept with fit-boundary, obstruction, depth/bed refusal and mount-solid checks, then airflow and header-current qualification in CAS-6.

### CAS-4 — Integrated milk-crate structure

Generate corner supports and stack keys as cassette-owned pieces, with no enclosing rack frame. Resolve interference with existing rear-arm sockets. Support interfaces and load paths must match before unlike profiles can stack. Minimize material and threaded joints; count assembly actions and fasteners in review output.

Model removal and cable swept volumes. A column carried by the removed cassette cannot remain supported by geometry that is leaving with it. Default service procedure: de-energize/disconnect the target and support or lift the units above before a middle pop. Do not claim independent drawer removal until a modeled alternate load path exists. Accept with mating/collision checks for two stacked units, a removal sequence and explicit supported/unsupported service states.

### CAS-5 — Modular power and harness placement

Keep a common stack attachment with PSU-specific replaceable carriers/backplanes. Default one PSU; retain the exact PSU/backplane/pinout/enable compatibility obligation. Reuse existing branch and aggregate budget checks. Native GH200/Collins/Jade power assemblies remain attached to their node profile where needed.

Place PSU, backplane, protected DC branches, mains enclosure/earth connection, connectors and strain relief. Cable census must follow actual endpoints, bends and service paths; front-edge board power and fan headers cannot disappear behind a rear-only route. Accept with cable/connector/service collisions, accessible disconnects, branch-load failures and retained-assembly envelopes accounted for.

### CAS-6 — Final physics and thermal qualification

Fold all installed components and generated/sliced part masses at their placements; retain researched ranges and missing-evidence standings. Include power cassette, wiring, joints and cooler transport loads. Re-evaluate corner reactions, buckling, tipping, interface shear and creep against final geometry and printed material/orientation. Preliminary Euler or rigid-body screens are not a certified stack limit.

Use selected fans' operating data with restriction/fan-failure cases, header-load limits and ambient/exhaust temperatures. CPU die temperature is not mount temperature. Generic PETG or Bambu PETG figures do not qualify SUNLU High Speed PETG. Keep material acceptance open until product-specific temperature/load evidence supports it. Accept with adverse controls and explicit remaining validation requirements; no invented measurements or operator weighing request.

### CAS-7 — Review and print release

Publish assembly and exploded per-piece rotation using the shared emitted model, including power/cable/clearance geometry. Show actual height, fan selection, BOM, assembly steps and qualification status. Compare with P1–P4 and the fitted fan mount; list reusable and superseded parts explicitly.

Only after geometry/solid/bed checks and appropriate material selection: export and slice remaining pieces for both A1 minis, record digests, mass and durations, and schedule a two-printer wave plan. Physical starts still require fresh bed-clear evidence and the reviewed authorization workflow. No new starts are implied by creating this queue.

### 140 mm joinery checkpoint — 4 October 2026

`rear_fan_assembly` now realizes the retained tray arms, two bed-sized crossmember halves, a keyed cradle, an integral-spacer guard and six broad-grip retaining pins. The P14 body uses its vendor 27 mm depth. Two diagonal M4 bolts capture plate, fan and guard; the modeled stack is 40 mm before washers/nuts, so final bolt length remains unresolved. The guard's nominal 7 mm openings are proposed geometry, not certified finger protection.

`stack_joinery` adds four cassette-owned clip shoes, hollow 20/12 mm posts split into lower/upper pieces, four middle-joint pins and keyed stack seats. No separate enclosing frame is present. The post wall area is 256 mm²; the shoe bearing area is only 127.75 mm². These must not be substituted for each other or for the old solid-pillar mechanics screen. Rear shoe relief clears the existing rear-arm pin heads. A middle pop explicitly requires independent support above it.

Validation: three new joinery witnesses passed. The shared CadQuery emitter produced 34 valid positive solids, including 28 connected printed solids, each under the 180 mm bed bound. Exact printed-solid intersection checks found no intersections above 0.01 mm³ within one cassette or between two at 230 mm pitch. These were manual realization checks, not a new durable CAD witness. P1–P4 are imported unchanged. The interactive review preserves piece isolation/rotation and adds a two-unit view. These results do not establish support-free slicing, clipping force, vibration retention, loaded lifting, creep, thermal suitability or a safe stack count.

Remaining before fabrication release: complete power and cable reservations, connect actual hollow/jointed support geometry to the load screen, qualify material/retention and cooling obligations, and slice selected parts. Shared larger-board panel-solid generation remains CAS-2 work; the profiles and partition graph alone do not make those boards printable. Required CI is still pending on the PR; focused local results are not a green merge gate.

### Provisional board references — 4 October 2026

The operator supplied a mechanical-source survey and authorized provisional inference. [Reference ledger](cassette-board-mechanical-references.md) records 13 Jade candidate mount positions derived from OCP Figure 1 and a separate provisional Collins outline (424.2 mm width × 512 mm depth). Collins now gets a 3 × 4 planning partition from the shared feature-aware allocator under the proposed process allowances. Hole-map completeness, feature identity and later verification remain explicit; no Jade pattern is substituted for Collins.

### Feature-aware cell allocation — 4 October 2026

`bed_partition` owns the axis allocation calculation, used both for board-only counts and actual panel-envelope planning. For two or more cells, capacity is first + last + (count − 2) × interior; this gives the minimum count directly. A logarithmic balancing search chooses cut widths, saturating constrained edge cells first. Integer remainders preserve exact coverage, and zero-width cells refuse. No repeated growing-list concatenation is used.

`tray_panel_features` owns the existing joint/rail/socket/handle dimensions; the retained Altra CAD and the allocator both read these values. This preserves the existing solids while removing the separate feature-budget estimate. `platform_projects_review` emits every cell's panel span and print envelope, including the proposed brim and bed-edge clearances.

| Profile | Planning grid | Column spans, mm | Row span, mm | Adjacent joins |
| --- | --- | --- | --- | --- |
| Altra | 2 × 2 | 135.92 / 135.92 | 147.35 | 4 |
| Collins | 3 × 4 | 148.5 / 151.85 / 151.85 | 135 | 17 |
| Jade | 3 × 4 | 148.5 / 153.75 / 153.75 | 126.84 | 17 |

This is 12 planned panels instead of 16 for Collins/Jade, and 17 adjacent joins instead of 24. It is the minimum axis-aligned grid under the declared side envelopes, not an unrestricted nesting optimum. A different brim, bed, feature envelope or board outline recomputes the count. Multi-cell solid realization remains open; no change to printer jobs or to fabrication admission follows from this plan.

Validation of this allocator change: six allocation checks, eight mechanical-reference/emitter checks and five platform checks passed. The combined review emitter succeeded. All 34 Altra part descriptions and its emitted CAD program are identical to the prior reviewed bundle, including P1–P4. Full required CI remains separate and pending.

### CAS-8 — Desired-set print orchestration

Requested next workflow milestone: [batch planning, dense plate packing and durable execution](print-batch-orchestration.md). BATCH-1 through BATCH-6 define the implementation and release checks. The [explicit-plate workflow foundation](print-workflow-integration.md) now owns preparation and fleet queue execution. Desired-set planning, accepted inventory, crash recovery and the live release checks remain outstanding; CAS-8 is not complete.
