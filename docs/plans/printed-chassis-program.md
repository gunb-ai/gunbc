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
- **Printed polymer is never the mains enclosure and never the sole protective-earth path.**
- **MVP thermal fixture choice carries no authority over deployment layout.** Using the 4U cooler
  first to reduce thermal uncertainty must not silently become the rack pitch.
- **Bay pitch derives from measured millimetres.** "2U" and "4U" are catalogue labels, not lengths.


## The numbered spine — PRINT-0 .. PRINT-15

The six projects are the working-memory units; these are the executable steps. Zero-indexed per the
RLM-N / MEMORY-0 convention already used in this directory. Each step names its terminal evidence,
because a step without one cannot be said to be done.

| Step | Project | Terminal evidence | State |
|---|---|---|---|
| **PRINT-0** | MEASURE | Board outline typed from the vendor manual; standoff map refused with a named trigger, not derived | **done** |
| **PRINT-1** | TOOLCHAIN | Printer, build envelope, filament classes, slicer platform claims as separate authorities | **done** |
| **PRINT-2** | MEASURE | Measurement authority: absent refuses, duplicate-key conflict refuses, precision budget enforced | **done** |
| **PRINT-3** | CAL | Coupon authority: ladder as modeled experimental design, rungs and plate derived | **done** |
| **PRINT-4** | TOOLCHAIN | Realization contract v0: contract derives its own rungs; a wrong-step handler is caught and located | **done** |
| **PRINT-5** | TOOLCHAIN | CadQuery handler + four-part authority wall; all four negative controls flip | **current** |
| **PRINT-6** | TOOLCHAIN | STEP/3MF emitted and re-inspected; output conformance re-establishes envelope, holes, walls | |
| **PRINT-7** | TOOLCHAIN | Slicer profile bound or refused; per-printer manifests; ProcessQualificationIdentity minted, not branded | |
| **PRINT-8** | CAL | Coupons printed on BOTH printers; each printer+spool admitted or refused **independently** | needs printers |
| **PRINT-9** | FIT | Adjustable-standoff fixture; real board mounted unpowered; hole map measured back and frozen | needs printers + board |
| **PRINT-10** | CASSETTE | Structural cassette: rails, tray, handle, latch; carries node mass; no seam in layer-separation tension | needs PETG decision |
| **PRINT-11** | CASSETTE | PSU carrier and harnesses; every connector insertable; nothing side-loaded | needs PSU envelope |
| **PRINT-12** | CASSETTE | 80 mm fan carrier + replaceable duct; powered thermal admitted against a bench baseline | needs cooler choice |
| **PRINT-13** | RACK | One fixed bay; cassette retained in service, removable after disconnect; empty bay structurally complete | |
| **PRINT-14** | RACK | 2x2 block; an enclosed node extracted with neighbours untouched | |
| **PRINT-15** | VERDICT | Cost, volume, print hours, labour, reprint rate, thermal, service time -> buy-more-printers decision | |

**N = 15.** PRINT-0 through PRINT-4 are landed and executing (18 witnesses). PRINT-5 through PRINT-7
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

- `extdeps.boards.asrock_rack` — typed 267 x 244 outline, first-party manual read; mounting holes
  refused with a named trigger rather than derived from the micro-ATX pattern (board is 23 mm wider
  than that pattern).
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
  arithmetic and the fit path.

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
wall or rib · part/revision datum marking · rigid transforms · boolean union and subtraction

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
