# Converging a selected cassette design to an accepted physical assembly

Status: migration in progress; inventory assessment, fenced inventory transitions and
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
