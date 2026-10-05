# Assembly requirements and their providers

Status: migration plan, not an implemented admission mechanism. Active worktree:
`session/print-workflow-integration`, PR #13334, targeting main. This plan extends
CAS-3/5/6/8; it does not fork the cassette design or change active prints.

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

The harness currently imports P8 lead/sharing facts while the residential assembly
selects P14. Removing that fixed-model dependency is a first-cut requirement.
`product.fabric.offer_route` provides a useful precedent for identity, missing
facts and ambiguity, but its provider-route policy is not a mechanical interface.
`std.resource_contract` describes compiler effects, not physical port capacity;
do not reuse its enums merely because they say capability/resource.

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
- Evaluation: `Satisfied` with named bindings and evidence; `Open` with missing
  evidence and a concrete closure trigger; or `Violated` with a located cause.
  An open item is not permission. A known collision cannot be hidden as open.

Preserve unknown source coordinates and provisional readings. Purchased-part
facts belong with their vendor authority when actually published. Coordinates
inferred from an image are repository observations with their own standing, not
new vendor facts. Installation direction is a product placement decision even
when the vendor supplies the fan's local airflow direction.

The assembly evaluator indexes placed owners/ports once, checks bindings,
accumulates shared capacity use, then evaluates local and cross-part constraints.
Candidate route search is a separate producer; the evaluator checks its result.
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
Cooler leads stay clear of both fans. The choice of left/right side and DC inlet
location is an output of endpoint, clearance and service checks, not yet a
verified layout. No loose cable is accepted through the fan rotor exclusion.

The existing 50 mm I/O reservation is only a proposed space budget. It cannot
stand in for connector bodies, cable bends or insertion/removal access. Use the
existing rear-fan prototype as a regression fixture, and show unresolved connector
locations rather than inventing exact coordinates to make a route pass.

## Migration and acceptance order

1. **Inventory and identity.** Attach every existing interface/obligation to its
   owner and use case. Inventory the selected board, P14, cooler, onboard storage,
   DC branch/adapter and Ethernet lead. Map every old obligation to an executable
   demand or an explicit open item, with no silently dropped rows.
2. **Join and findings.** Implement named bindings and the shared report. Cut the
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
5. **Fabrication and release.** Join the current print-plane/support/readback
   evidence into the report. Separate view/fit-prototype, fabrication and powered
   operation policies. Keep board supports open for this prototype, without
   allowing that concession to grant powered operation. Do not alter running jobs.

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
