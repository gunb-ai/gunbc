# Mt. Collins GPU augmentation — program

**Status: contract phase, no geometry authored.** This is a SECOND product program with its own
authority (`product.mt_collins_gpu_augmentation`), deliberately not folded into
`product.printed_chassis`. It pressures different parts of the model than the ALTRAD8UD cassette:
mixed purchased and fabricated structure, a heavy high-value expansion card, PCIe signal integrity
and cable strain, independent power and protective earth, two interacting airflow domains, sustained
thermal evidence, and a prototype-material versus deployment-material distinction.

The standing rule that **cable routing and grounding apply at every controlled layer** carries over
unchanged.

## Architecture ruling: companion bay first, hood second

```
MtCollinsGpuServingUnit
├── original sealed 1U server (lid, fan wall, CPU/DIMM shrouds untouched)
├── adjacent 1U GPU companion bay
│     retention/load path · airflow · dedicated PSU unless the PDB is proven · PE bond
├── purchased qualified PCIe riser/cable
└── modeled cable, service and thermal interfaces
```

The companion bay is the better first experiment because it preserves the stock thermal system,
keeps GPU weight off the riser connector, allows a dedicated PSU and independent intake/exhaust, is
reversible, and separates the server's cooling result from the GPU enclosure's. A modified 2U lid or
hood is a legitimate *comparison* candidate, not the starting point. **Card orientation is not yet
admitted** — exact card dimensions and usable 2U geometry decide it, and a horizontal arrangement may
be the only viable one for a full-height card.

## Census ANSWERED — this program is experimental, not the critical path

**~8 of the 28 boxes are at least 2U, some 4U, and take a GPU with no fabrication at all.** So the
1U retrofit is an experiment to run if it is ever wanted, not the route to a served GPU. Everything
below stands as the design if the retrofit is attempted; none of it is on the critical path, and the
program should not consume printer time that the ALTRAD8UD cassette needs.

The live question moved with it: not "how do we get a GPU into a 1U" but **"what changes in the
ALTRAD8UD stackable chassis when one of its nodes carries a GPU"** — recorded in
[printed-chassis-program.md](printed-chassis-program.md) under the GPU-bearing node, because that is
a variant of the node contract rather than a second product.

## The census, retained for the reasoning

The brief's own MTC-GPU-5 lists "purpose-built 2U GPU server" as a comparison candidate. Per the
operator's hardware discussion, **the 2U Gigabyte G242-P31 is the Ampere platform built for exactly
this** — four dual-slot GPUs, front-to-back airflow, PSUs sized for it — while **a 1U Mt. Collins
cannot take a dual-slot full-height card at all**; the 1Us are CI boxes.

So the cheapest possible first action is an **exact-model inventory census of all 28 boxes**. If any
are the 2U variant, the companion-bay program is not the cheapest route to a served GPU — it is a
workaround for a chassis constraint that a machine already owned does not have. A program that can
be deleted by one inventory query should face that query before any geometry is modeled.

This is not a reason to discard the brief: the companion bay remains the answer if the fleet is
all-1U, and the ruling's decomposition stands either way.

## Design for the production process, from the first printed part

The operator's stated intent is **prototype by printing, produce by manufacturing** — specifically
laser-cut and bent sheet aluminium, which for a colo is worth more than its price: fire-rated by
nature, grounds to the shelf, no tooling, ~$40-60 at 20-100 units. Printing does not scale into
production here; it is the design loop.

Two consequences bind the geometry **now**, before any part is modeled:

- **Model bend-friendly, flat-ish panels** — uniform material thickness, bend reliefs, no geometry
  that only survives because a printer tolerates it. *A print that only works because printing
  tolerates anything is a prototype of nothing.*
- **Freeze before you tool.** The readiness signal is not "it works" but "the last three revisions
  were cosmetic."

This also splits process qualification into **two targets**: the printer–spool process (what the
hole-diameter coupon measures, for the prototype loop) and the sheet-metal vendor process (what the
production parts are actually made by). These are different `ProcessQualificationIdentity` subjects
and must not share one. The coupon program is not invalidated by this — it qualifies the loop that
produces the prototypes — but it does not qualify a production part.

## Phases

| Phase | Terminal evidence |
| --- | --- |
| **MTC-GPU-0** contract | Exact GPU SKU, count, bay-vs-hood candidate, rack allocation, prototype-vs-deployment target, power architecture. No generic "Blackwell", "x16 riser" or "2U hood" stands in for a selected product. |
| **MTC-GPU-1** measure | Chassis external/usable envelope; lid, shroud, riser, rear-slot geometry; GPU envelope, mass, bracket, connectors, airflow; PDB connectors and ratings; candidate PSU; rack depth and cable-service envelope — with uncertainty and provenance. |
| **MTC-GPU-2** PLA cold-fit | Rear pass-through/blanking adapter, riser strain relief, bracket-location gauge, cradle prototype, duct prototype. Surrogate mass before the real card sits on printed retention. Fit only — no power or thermal claims. |
| **MTC-GPU-3** powered bay | Independent card load path, admitted PSU, PE, fans/ducting, full cable routing. PCIe link trains at required width and generation, no relevant link errors, GPU idles safely. |
| **MTC-GPU-4** thermal admission | The isolated matrix below. Combined load within exact product limits, no unexplained host regression, no printed-part thermal failure. |
| **MTC-GPU-5** production verdict | Compare custom bay vs integrated hood vs commercial expansion chassis vs purpose-built 2U server on cost, rack occupancy, thermals, reliability, service time, fabrication effort. |

## Load path — printed parts are never the retention

The GPU's load terminates in the bay or rack, **never** in the PCIe edge connector, the riser card,
the flexible cable, the printed duct, or the original server lid.

```
rack rails / metal bay shell -> metal GPU bracket -> metal or reinforced cradle
                                                  -> optional printed alignment pads and duct
```

Printed: card-location adapters, anti-sag supports, blower intake ducting, cable strain relief,
connector guards, fan mounts, blanking pieces, geometric identifiers.
Metal: rack-width shell, primary retention, long beams, PSU enclosure, PE path, fire and impact
containment. A rack-width enclosure is not one A1-mini print in any case — this is another
"print the joints and ducts, buy or bend the straight metal" design.

## Cable and grounding — independent modeled routes

PCIe data (host riser slot → pass-through → cable → GPU edge) · GPU DC power (pod PSU or admitted PDB
→ auxiliary connectors) · AC (rack supply → bay PSU) · cooling/control (fan power, PWM/tach) ·
**protective earth** (AC PE → PSU enclosure → bay metal → rack bond) · functional/chassis reference,
separate from PE.

Each route carries endpoints, connector orientation, bend volume, moving/fixed classification, strain
relief, hot-surface separation and disconnect order. **The PCIe cable is never retention, never
chassis grounding, never an uncontrolled hinge, and never a service loop with an undefined bend
radius.** The rear pass-through must preserve the host pressure boundary as far as practical — a
large unblanked opening can disturb host airflow even with the lid on.

## Thermal — two coupled domains

```
HostCoolingDomain   front fan wall -> CPU/DIMM shrouds -> rear exhaust
GpuCoolingDomain    dedicated ambient intake -> GPU cooler/blower -> declared exhaust destination
```

Primary law: **`GpuCoolingDomain` must not materially impair `HostCoolingDomain`.**

Test sequence, each change isolated: stock sealed 1U under CPU/DIMM load (host baseline) → bay
installed, GPU unpowered (passive obstruction, recirculation) → GPU idle (fan/idle-air interaction) →
GPU load only → combined load (terminal case) → declared fan degradation or elevated ambient
(failure margin).

Observed: CPU temps and throttle state, DIMM temps, BMC-available VRM/board sensors, host fan speeds,
GPU core/hotspot/memory/fan/power/throttle reason, host inlet and exhaust, GPU inlet and exhaust,
printed-part surface temperature at the hottest locations.

Admission requires: no vendor limit exceeded ∧ no unexplained throttling ∧ host regression within a
declared budget ∧ GPU sustained at target power ∧ printed material within its admitted service
envelope ∧ no recirculation mode left unmeasured.

## Material policy

PLA **may** establish dimensional fit, card and bracket location, riser-cable path, interference and
service clearance, room-temperature duct topology, fan and connector placement.

PLA **may not** establish sustained structural retention near the GPU exhaust, long-term anti-sag,
deployment thermal durability, flame behaviour, or colo suitability.

A powered PLA prototype is defensible only when the PLA parts are not the sole card support, are not
the mains enclosure or PE path, and are temperature- and deformation-monitored under supervision,
with a secondary metal retention path if the plastic softens. **PETG is not automatically
sufficient** — measure actual part temperatures first.

## Facts that stay OPEN until measured

Exact GPU SKU, dimensions, mass, slot width, power, connector locations, airflow direction · rear
riser electrical width and generation · exact qualified riser cable and usable routing length
(signal integrity is not inferable from "Gen4 x16 cable") · PDB GPU-power capability · stock
lid/shroud thermal role · GPU-bay intake/exhaust relationship to server exhaust · available rack
depth and rear service clearance · printed-part operating temperatures · colo/site rules.

The 60-80 °C blower exhaust figure is a **hypothesis until measured** on the exact card and airflow
arrangement, and "under 80 °C for an hour" is **not** the terminal rule — the exact GPU's vendor
limits, throttle state, hotspot and memory limits, and the server's CPU/DIMM limits govern.

## Workload model stays separate

The KV-cache analysis is not mechanical authority. It determines a `SelectedGpuDeployment`
(gpu_sku, gpu_count, target active session population, parked session population, storage tier)
which this program **consumes**. The chassis program must not independently encode "two cards" or
"64 sessions". The load-bearing distinction is that **parked-session capacity is not simultaneous
active decoding capacity** — that may decide whether the augmentation is worth building at all, and
it must not silently decide card count before the active-fraction requirement is selected.

## Extraction discipline

This becomes the **second real consumer** that could justify extracting generic facilities currently
inside the ALTRAD8UD program — process qualification, model-to-CAD realization, generated-artifact
materialization, spatial measurement, cable routing, protective earth, thermal observation,
fabrication receipts. Extract when both products ask the **same typed question**, never because the
names look reusable.
