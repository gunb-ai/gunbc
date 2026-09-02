# Printed node chassis — program plan

**Pinned plan. Updated 2026-09-02.** Printers land **2026-09-03**.

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
| Service law | fixed frame, removable cassette | operator requirement R9 |
| PSU placement | cassette-resident | side-chat ruling |
| External bundle | 2 x Ethernet + 1 x AC per node | operator |

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
| **PRINT-1** | TOOLCHAIN | Build envelope, filament classes, slicer platform claims as separate authorities, **and a concrete A1 mini product row that the coupon fit witness consumes** | reopened |
| **PRINT-2** | MEASURE | Measurement authority: absent refuses, duplicate-key conflict refuses, precision budget enforced | **done** |
| **PRINT-3** | CAL | Coupon authority: ladder as modeled experimental design, rungs and plate derived | **done** |
| **PRINT-4** | TOOLCHAIN | Realization contract v0: contract derives its own rungs; a wrong-step handler is caught and located | **done** |
| **PRINT-5** | TOOLCHAIN | Raw transport schema admission: unknown version, missing field and unknown operation all refuse before any geometry call | |
| **PRINT-6** | TOOLCHAIN | CadQuery handler + four-part authority wall; all four negative controls flip | |
| **PRINT-7** | TOOLCHAIN | STEP/3MF emitted and re-inspected; output conformance re-establishes envelope, holes, walls | |
| **PRINT-8** | TOOLCHAIN | Slicer profile bound or refused; per-printer manifests; ProcessQualificationIdentity minted, not branded | |
| **PRINT-9** | CAL | Coupons printed on BOTH printers; each printer+spool admitted or refused **independently** | needs printers |
| **PRINT-10** | FIT | Adjustable-standoff fixture; real board mounted unpowered; hole map measured back and frozen | needs printers + board |
| **PRINT-11** | CASSETTE | Structural cassette: rails, tray, handle, latch; carries node mass; no seam in layer-separation tension | needs PETG decision |
| **PRINT-12** | CASSETTE | PSU carrier and harnesses; every connector insertable; nothing side-loaded; **PE continuous with board, standoffs and cassette all removed** | needs PSU envelope |
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

**N = 17.** PRINT-0 through PRINT-4 are landed and executing (27 witnesses); PRINT-1 is REOPENED because the build envelope is authored inside the fit witness rather than owned by a printer product row, so that witness would stay green if the product authority changed or vanished. PRINT-5 through PRINT-7
are the pre-arrival critical path and depend on no operator input. PRINT-8 is the first step that
needs hardware.

### What blocks what

- **Nothing blocks PRINT-5..7.** These are the two-day cut.
- **Joint family** (dovetail / tongue-and-groove / keyed slide / pin) gates the
  ProductionInterfaceCoupon only — a later half of PRINT-8, not the MachineProcessCoupon.
- **PETG** gates PRINT-10 onward. PLA carries PRINT-8 and PRINT-9 and stops there: no PLA-to-PETG
  receipt carry.
- **Deployment envelope** gates PRINT-13..14 layout admission, nothing earlier.
- **Drive count** gates PRINT-11 completion only.

## The six projects

| Project | Scope | Terminal evidence | State |
|---|---|---|---|
| **CONTRACT** | Goals, non-goals, authority graph, refusal law, MVP boundary | No assumption represented as a default | deferred — no consumer yet |
| **MEASURE** | Board, cooler, PSU, fan, DIMM, cable, **deployment envelope**, uncertainty | Pack sufficient to generate CAL+FIT | **partial — carrier landed, roster empty** |
| **TOOLCHAIN** | Realization contract, CadQuery handler, authority wall, STEP/3MF, slicer env, printer nodes | Deterministic generation + executed no-parallel-authority controls | not started |
| **CAL+FIT** | Coupons, qualified tolerances, adjustable standoff fixture, real-board fit, hole-map feedback | One unpowered board fits; hole authority admitted | not started |
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
  is admitted through `admit_envelope`, which refuses an unsupported schema version, an unknown
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
  put the geometry on the accepted arm (`EnvelopeAdmitted { plate, features }`) and argued admission
  was unavoidable "because the refusal arms have no geometry to hand on" — a correct argument for an
  insufficient conclusion, since it rules out the raw envelope bypassing the judge and says nothing
  about a caller bypassing the raw envelope. A probe compiled the forgery from a foreign module.
  `AdmittedRealization` is now `sole_constructor`, so only `admit_envelope` can mint one; a foreign
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
  source writing the `AdmittedRealization` record literal from a foreign module, a green source with
  identical imports and no literal, and the red-minus-green subtraction as a third witness so the
  shared imported closure cancels rather than being asserted away. The `CensusNotRunnable` arm answers
  `-1`, so every assertion compares against a non-negative quantity and a harness that never ran
  satisfies nothing — which is exactly the vacancy that produced the near-false-finding below.

  **The refusal payload could carry a success, and the witnesses were where it showed.**
  `RealizationVerdict`'s refusal arm was typed `RealizationRefusedAtWire { admission: EnvelopeAdmission }`,
  and `EnvelopeAdmission` includes `EnvelopeAdmitted`. No route produces that state —
  `judge_realization` builds the refusal arm only on the three refusing branches — but reachability is
  not occupancy, and the state was writable. The tell was not in the contract at all: four witnesses
  carried a dead `EnvelopeAdmitted { admitted: _ } => false` arm purely to satisfy exhaustiveness over
  a case the route cannot produce, which is what a coproduct too wide for its position looks like from
  the consumer side. The refusal reasons are now their own closed coproduct `EnvelopeRefusal`, and
  `EnvelopeAdmission = EnvelopeAdmitted | EnvelopeRefused { refusal: EnvelopeRefusal }`. The class
  climbs from mechanically preventable to structurally impossible, and the four dead arms are deleted
  with it — §4b(4) retires the redundant production handling while every discriminating witness stays
  enrolled.

  **Still open, routed by the side chat and not yet answered:** `AdmittedRealization` is minted before
  semantic conformance runs, so the trusted-type boundary may sit one step too early. Two candidate
  cuts: a second sealed type downstream of `judge_realization` carrying admitted-and-conformant, or
  `AdmittedRealization` not existing until conformance has run. Decode-admitted and contract-conformant
  are genuinely different facts, so the first reading is not obviously wrong — but no consumer today
  distinguishes them, which is exactly the condition under which one of the two is redundant.

  **Probe discipline learned here, recorded because it nearly produced a false finding.** The first
  forgery probe returned exactly the baseline error count — consistent with "forgery permitted" — from
  a file the compiler had never read, because an added source root was silently ignored. Confirmation
  and vacancy produce the same number. Only a deliberate must-fail control in the same file
  distinguished them.

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

## Open decisions (operator) — none block the first print

- **Drive count** — a CASSETTE completion obligation. Blocks nothing before Thursday.
- **PETG** — blocks structural/powered parts, not CAL or unpowered FIT.
- **Deployment envelope** — **not on the pre-arrival critical path.** Its absence must refuse
  rack-layout admission later, not block calibration now.

## Next

1. ~~`CalibrationCouponAuthority`~~ — landed as `product.printed_chassis.coupon`.
2. `RealizationContract v0` — the transport-only, schema-bound feature graph the handler consumes.
3. CadQuery handler + four-part wall executed over v0, with all four negative controls.
4. STEP/3MF generation and conformance; bound slicer profile or refusal.
5. Per-printer manifests for node 1 and node 2; measurement/admission form ready for results.

**Deferred as definition-only residue until each has a consumer:** `CONTRACT` (needs the generator),
`DeploymentEnvelope` (needs the layout-comparison consumer). The design of both is pinned above.

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
