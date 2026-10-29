# Integrated milk-crate cassette

**Goal:** stack serviceable compute cassettes without a permanent rack. **Method:** four integrated corner columns, broad compression-bearing faces, removable alignment keys and independent power/network drops. **Validation:** finalized geometry, complete placed-mass ledger, collision/extraction/bed-fit checks and qualified structural, thermal and electrical cases before populated use.

Architecture review [#13295](https://github.com/gunb-ai/gunbc/pull/13295), based on `session/cassette-joinery-review` (#13222), supersedes separate-frame #13283. It is not a print or live-service release; r06/r07 meshes remain historical.

## Structure and service

Pins retain alignment, never gravity load. Prefer open ribs/L-sections and gussets; section, layer direction, joint stiffness and creep remain unqualified. Existing rear arm sockets conflict with tray-corner posts. Move bearing sections outward or redesign sockets together: the review candidate is x=±142 mm, z=±159 mm with a 20 mm envelope (304×338 mm footprint), not released geometry. One authoritative `StackJoint`, actual sections and a continuous extraction sweep must replace `rack_assembly.fixed_parts`.

Supported middle removal is a proposed sequence:

1. Shut down/isolate the target; disconnect its DC and network.
2. Fit two floor-supported trestles outside extraction; their crosshead bears on the **upper cassette’s corner pads**, never tray/fan arms.
3. Lift only enough to disengage keys and restrain tipping. Upper drops need lift allowance, bend radius and strain relief; cables carry no load.
4. Withdraw forward while upper nodes retain independent drops; reinsert, reseat, transfer load, remove supports and reconnect.

Until load path, retention, lift and cable sweep qualify, demonstrate unloaded only. Without supports, service is LIFO.

## Harness and power

The [ASRock manual](https://download.asrock.com/Manual/ALTRAD8UD-1L2T.pdf) places three 8-pin inputs/five fan headers opposite rear I/O and permits DC-IN without the 4-pin signal lead. High-load input population and header current limits remain open.

- Each node gets an independent protected front drop, rated/keyed disconnect and strain relief; no power daisy-chain. Select pigtail contacts, crimps, wire and protection together, never by appearance.
- Independent rear data/management drops reserve plug/latch/bend clearance and escape beside fan apertures. Link count follows selected topology, not every RJ45 socket.
- Rear fans route to front headers along a side. The modeled 371 mm straight route plus 100 mm allowance exceeds a 400 mm lead; include extensions in BOM/mass. The cooler has its own lead; do not add its flow as an independent parallel rear fan.
- The power cassette owns mains/PE, distribution, branch protection, disconnects and cooling. `harness_census` is a lower-bound old LIFO/raceway screen, not a cut list or middle-service admission. Bind actual endpoints, topology, clips, extensions and station wiring before ordering.

[`cassette-server-psu-interface`](cassette-server-psu-interface.md) owns interchangeable mechanical inserts, PSU-specific adapters, AC derating, redundancy and [independent pairing](cassette-server-psu-interface.md). Assign each node one compatible protected output; check per-port/aggregate budgets independently of node/column count. Multiple cassettes may serve separate groups; no implicit output paralleling.

`node_power_feed` owns the voltage/contact calculations: round constant-power current upward at minimum voltage and refuse zero derating. The 8-circuit/18-AWG Molex 5556/5558 screening row uses the lower material rating, not the family maximum. `NodeFeedScreeningOnly` does not qualify actual headers/crimps/PCB, wire ampacity, current sharing, drop, protection, transients or board input population. ATX voltage is a direct-DC screening assumption. Supply selection still owes DC-return bonding and node bonding/ESD.

## Mass and mechanics

[`component_mass`](../../dag/product/printed_chassis/component_mass.dag) owns the component inventory, evidence classes and assumptions; `stack_mass_review` consumes actual srv3/srv4 memory populations. Count each cooler assembly/fan once. Use vendor bounds as bounds, nominal values as nominal, and board/DIMM planning ranges as assumptions; retailer shipping weights prove none of them. Printed mass must be revision/profile-bound slicer or measured part mass, not positive primitive volume. Historical r06 PLA job consumption includes possible support/purge and cannot become redesigned PETG part mass. Hardware, harness, corners, station and service loads remain explicit; no missing-as-zero. No operator weighing/disassembly is requested.

`placed_mass_model` folds mass and first moments. Every interface uses all components above it in that frame; the bottom station both adds mass and raises nodes. Check four support edges using actual CG; push uses force height, acceleration uses CG height. Under equal contact stiffness, reactions scale as `1 ± x/a ± z/b`; negative reactions require unilateral re-solving, not clamping. Quarter loads require centered symmetry. Incomplete ledgers refuse. Endpoint sweeps are sensitivity, not interval proof: heavier can increase restoring moment.

`stack_mechanics` remains the labeled uniform approximation (`StackScreeningOnly`, `max_screened_height`), with corrected push/station offsets and no bending-strength-as-compression substitution. **No safe height is established.** Qualify contact/orientation, joint shear/bending, buckling/end conditions, compression, measured-temperature creep, sliding/friction, cable pull, bump/tilt and supported-service loads. Rigid statics does not replace FEA/joint/creep tests. “10 high” and equal PLA/PETG height claims are withdrawn.

## Thermal and cooler variants

Match the exact SUNLU High Speed/Matte PETG SKU/lot to its [TDS](https://media.sunlu.com/prod/20260330/ccf53a55-5b58-49fa-b1d9-08f94c0a4b28.pdf?filename=TDS): HDT at specified stress/orientation is neither melt temperature nor a continuous loaded-part limit. Use the vendor-neutral filament row, not Bambu values. CPU junction temperature does not establish mount temperature; a measured 100°C mount exceeds this PETG’s HDT screen.

`exhaust_thermal` owns `Q/(rho V cp)` and `AssumedAir...` cases: fractions of catalog free-air flow are not PWM duty. Installed airflow needs a fan/system-curve intersection or measurement; unknown local conditions remain `InstalledThermalUnresolved`. The P8 ambient ceiling applies independently of filament. Measure inlet, fan inlet, hottest mount/joint under sustained load and a controlled fan-failure shutdown transient; radiation, conduction, recirculation and creep remain open. No PWM floor or PLA/PETG operating admission follows from free-air calculations.

`cooler_configuration.residential_cooler` selects [NH-U14S AMP-4926](https://www.noctua.at/en/products/nh-u14s-amp-4926/specifications); `stack_pitch_for_cooler` and `node_components_for_cooler` use its full envelope and mass including both fans once. Preserve W1 fleet observations and the three rear fans. No fixed 4U height or rack-unit rounding. Align perpendicular-to-socket airflow with exhaust. Contact-plane height is still a 30 mm review assumption; pose, clips, DIMM/connector/service clearance remain open. The envelope is not detailed CAD or proven fit. A taller/heavier variant needs its own pitch, harness and placed-load case; W1 qualification cannot transfer.

## Evidence and next cut

Run `dag/test/claim/printed_chassis_*_witness_test.dag` and `gunbc run --source-root dag --source-root src/v2 --entry dag/product/printed_chassis/stack_mass_review.dag`. The prior server-power/cooler witness run and combined report are historical receipts, not integrated-corner qualification. New geometry must replace the fixed-rack root, bind finalized parts to the ledger and pass collision/continuous-sweep/bed-fit checks before printing.

Vendor mass/geometry provenance: [Dynatron W1](https://www.dynatron.co/product-page/w1), [ARCTIC P8](https://www.arctic.de/media/84/14/68/1697784882/Spec_Sheet_P8_PWM_PST_EN.pdf), [Ampere Altra Max §4](https://amperecomputing.com/assets/Altra_Max_Rev_A1_DS_v1_25_20240130_73cfcc518a_4705c00046.pdf), [Samsung DIMM §18](https://download.semiconductor.samsung.com/resources/data-sheet/DS_64GB_TSV_DDR4_4Gb_D_die_RegisteredDIMM_Rev13.pdf). The 4 October 2026 research used indexed primary Ampere/Samsung excerpts where direct fetch failed; Samsung supplies geometry, not the assumed mass range.
