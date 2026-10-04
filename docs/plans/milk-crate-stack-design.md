# Milk-crate stack: design, power topology, physics instruments

Operator direction, 4 October 2026, superseding the stacking-frame proposal of
[gunb-ai/gunbc#13283](https://github.com/gunb-ai/gunbc/pull/13283): the cassette IS the structure
(corner pillars on each cassette, no separate frame), removal is a literal stack pop, and the
power supply becomes a cassette in the same format that stacks with 1..N node cassettes and
feeds everything below or beside it. Cable management is a property of that arrangement, not a
step. Branch: `claude/eager-sagan-u4myup`, stacked on the r06 cassette model of
[gunb-ai/gunbc#13222](https://github.com/gunb-ai/gunbc/pull/13222).

## Two rulings this reverses in the program plan

`docs/plans/printed-chassis-program.md` fixed facts carried **service law: fixed frame, removable
cassette (R9)** and **PSU placement: cassette-resident**. Both are reversed here, in the plan, so the
model and the plan do not fork:

- **Service law:** LIFO column. Popping node k lifts every unit above it. The cost is derived
  (`product.printed_chassis.harness_census` `disconnect_sets_to_pop`), not argued.
- **PSU placement:** one **power cassette** per column (`product.printed_chassis.power_cassette`),
  the mains boundary of the column; node cassettes carry 12 V only.

## Review of #13283, carried forward

- Two authorities for what holds a cassette (the metal 2x2 rack in `rack_assembly.fixed_parts`
  stayed live beside the frame, and PSU/PDB/FEEDS were declared twice). **Owed with the pillar
  geometry change:** delete `fixed_parts` and the Bool-forked manifest in the same change that
  adds the pillars. This change adds no geometry, so it does not perform that cut; it records it.
- `stacking_dimensions_valid` was a decoration (its tests were true by the definitions of its
  inputs). The replacement is construction: the pitch is derived in
  `product.printed_chassis.stack_geometry` from the tallest and lowest cassette parts plus a
  declared clearance, and the pillar section is a named proposal row.
- The `rack-ground` interface was filtered out with nothing in its place. The protective-earth
  obligations are now rows on the power cassette (`power_cassette_earth_obligations`), and the
  argument that lets node cassettes drop out of the earth network is stated there: no mains part
  leaves the power cassette.
- Interference was sampled at seven positions in the Python adapter. For a pure translation the
  swept volume is each positive primitive stretched along the travel axis; that fold belongs in
  `.dag` and applies to the vertical lift of a pop. Not built here; named for the geometry change.

## What the manual settled (read first-party, v1.20, July 2025)

Rows in `extdeps.boards.asrock_rack`, each citing its section:

- DC-IN 12 V is supported on the 8-pin inputs; the 4-pin signal connector is **not used** in
  DC-IN mode (`asrock_altrad8ud_power_input_requirement`, section 2.8).
- "Under the lowest power supply, user can freely use ATX12V1, ATX12V2 or ATX12V3" and nothing
  above that: the population standing is now `PowerInputMinimumOneStated`, and how many inputs a
  given load needs is derived from connector ratings in `product.printed_chassis.node_power_feed`.
- Each 8-pin has four 12 V contacts (pins 5-8). ATX4PIN1 is PWROK / GND / 5VSB / PSON#.
- **Power inputs and all five fan headers are on the FRONT edge** (opposite the rear I/O), which
  is the fact the harness inherits: a rear raceway must cross the tray to reach them, and the rear
  fan bank's header cable crosses it the other way.
- Operating ambient 10-35 C; four rear RJ45 (2x 10GbE, 1GbE, IPMI); PSU SMBus header for one
  supply from one host.
- Standby under DC-IN is an inference typed as one (`asrock_altrad8ud_dc_in_standby_standing`):
  the board must derive 5VSB on-board; the receipt is a held board lighting SB_PWR1 with ATX4PIN1
  unplugged.

## Power topology, end to end

```
mains ─ IEC inlet ─ [power cassette: vendor-enclosed supply, PE bond, DC return bonded once,
                     distribution board, one protected isolatable 12 V branch per node]
          │ branch k (gauge, length derived from level distance)
          ▼
   node k DC inlet (rear or front: a selection the census prices)
          │ 8-pin pigtails, count derived from load vs connector rating
          ▼
   ATX12V1..3 on the board's front edge          fan bank ─PST chain─ one FAN header
```

`product.printed_chassis.node_power_feed` takes a node load (from `product.node_power_envelope`,
which refuses while the DIMM population is unread, or a declared bound), the Mini-Fit Jr rating,
a declared retained fraction, branch gauge and length, the station setpoint and the ATX12V +12 V
band, and answers `NodeFeedAdmitted` (inputs to populate, branch current, round-trip drop,
voltage at the connector) or a typed refusal: load beyond three inputs, unknown gauge, connector
below 11.40 V, setpoint above 12.60 V. The fan bank reaches the board on one header through PST
and the verdict is `FanHeaderLoadAgainstUnreadLimit`, because the manual states no header limit.

`product.printed_chassis.power_cassette` sizes the station with the one adequacy fold the node
model already has, derives wall draw at the supply's efficiency grade, and carries placement as a
parameter (top, bottom, beside). **Recommendation: bottom of column.** The heaviest unit sits
lowest (tipping), it is never lifted by a pop, and the cable lengths are symmetric with top.

`product.printed_chassis.harness_census` counts every cable per node (branch, pigtails, patch
leads, fan header cable, PST links) with lengths from the pitch, the level distance to the
station and switch, the tray crossing the inlet placement implies, and a service allowance; it
reports whether the fan's own 400 mm lead reaches the front header and how many disconnect sets
a pop costs. Run `dag/test/claim/printed_chassis_stack_feed_witness_test.dag` for the numbers.

## Physics instruments

`product.printed_chassis.stack_mechanics`: a mass roster as an identity join over the cassette's
parts (printed parts from modeled volume and the filament sheet's density, purchased parts from
vendor rows, board and DIMMs **refuse until weighed** unless a caller declares a figure by name),
per-level pillar load, bearing stress on the tray corner against a declared fraction of the
sheet's bending strength, Euler buckling of the pillar, static tip force against a declared
service push, and `max_admitted_height`. Creep at exhaust temperature is carried as an
unqualified boundary on every admission. Run
`dag/test/claim/printed_chassis_stack_mechanics_witness_test.dag`; under the witness's declared
board and DIMM masses tipping binds first and a bottom station admits a taller column.

`product.printed_chassis.exhaust_thermal`: exhaust rise = heat / (air density x bank flow x cp)
from `extdeps.physics.dry_air` and the P8's published airflow, against the filament sheet's HDT
with a declared margin. The die limit (100 C) is exposed beside it and is not an input: the mount
sees exhaust air. At full flow with a 35 C inlet the rise is single-digit kelvin and PLA Basic
keeps its margin; at 20 percent duty with full load it does not, so the BMC fan curve floor is a
safety input. Radiation from the fins, conduction through the standoffs, the cooler fan's
interaction with the bank, and HDT's stress dependence are named unmodelled couplings. Run
`dag/test/claim/printed_chassis_exhaust_thermal_witness_test.dag`.

## Cited rows added (each graded by how it was read)

`extdeps.printing.bambu_lab_filament` (PLA Basic, PETG HF sheets), `extdeps.connector.molex_mini_fit_jr`,
`extdeps.standards.awg`, `extdeps.standards.atx12v_psdg`, `extdeps.physics.dry_air`, airflow /
mass / current / PST on `extdeps.fans.arctic`, mass and fan duty points on
`extdeps.cooling.dynatron`. Vendor sites were refused by this session's egress; the figures came
through search relays and every such row carries `TranscribedUncited` with the read that retires it.

## Open obligations, in the order they block

1. Weigh the board and one DIMM; retire the witness's declared masses.
2. Pillar geometry: resolve the rear corner against the slide-in arm sockets, add the
   `StackJoint` row (one declaration, both sides), delete `fixed_parts`, add the vertical swept
   volume for a pop.
3. Read the fan header current limit from ASRock; until then a three-fan PST bank is unadmitted.
4. Confirm standby under DC-IN on a held board.
5. Select the DC inlet connector as a `std.decision` subject (XT60/XT90, Anderson, Mini-Fit Jr).
6. One read of each vendor page named in a `TranscribedUncited` row.
7. A loaded column held at the measured corner temperature, for creep.
