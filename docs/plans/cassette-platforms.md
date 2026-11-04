# One cassette design, multiple platform profiles

**Goal:** support ALTRAD8UD, NVIDIA P4261 GH200, Mt. Jade and Mt. Collins through one cassette generator (operator, 2026-10-04). **Method:** `cassette_profile` shapes, `cassette_profiles` rows and `cassette_layout` derivation; existing Altra geometry consumes the same model. **Validation:** shared assembly/collision/bed-fit witnesses plus mass/force/thermal screens, with unresolved outlines or holes refusing fabrication.

A baseboard cassette is mechanical support, not an electrical replacement PCB. Profiles may differ in footprint, mounting, retained assemblies, cooler, corridors and power. Cross-profile stacking must preserve supported interfaces/load paths; unsupported tray extensions do not solve width changes. `stack_mass_review` reports standings; upstream facts stay with vendor owners.

| Profile | Evidence and open inputs |
| --- | --- |
| ALTRAD8UD | Existing dimensions/prints/fan observations; standoff, underside, load and thermal obligations remain. |
| P4261 / 694-24261-000-100 K1 | Exact requested [listing identity](https://www.ebay.com/itm/157373033242), not authority for compatibility/pinout. Need manufacturer revision/outline/hole/keep-out drawings, full module/carrier/management BOM, power sequencing/connectors and cooling. No MGX enclosure or Jetson drawing substitution. |
| Jade | [OCP rev 1.0](https://www.opencompute.org/documents/open-compute-specification-mt-jade-rev-1-0-pdf-1) outline and provisional image candidates; verified mounting, retained assemblies and revision-specific interfaces remain. |
| Collins | Existing briefs/GSG and provisional reseller outline; exact board/chassis revision, mounting drawing and original power/management placement remain. |

Guide identities/hashes live in `extdeps.ampere.mt_{collins,jade}_getting_started_guide.subject`; [Ampere catalogue](https://amperecomputing.com/home/customer-reference-boards) supplies follow-up packages. The 2026-10-04 reads matched those hashes: Collins issue 1.05 Table 7/Figure 11/revision pages and Jade issue 1.00 rear interfaces/Figure 11 gave placement evidence, no dimensioned mounting map. Collins DVT BX/DX/EX, PVT IX/KX and MP 0B must match any later drawing. Jade’s later archived OCP pass recovered Figures 1/3 after the initial §12 fetch failed. Perspective screw rings are not fabrication coordinates; [mechanical references](cassette-board-mechanical-references.md) retain the provisional boundary. Keep publisher identities, not PDF binaries.

Prefer each transplant’s original functional assembly: socket backplates/load mounts; matched PSU/PDB/cage, enable/sense/standby/PE; management/sequencing/front-panel/interlocks; cooling, fan control/ducts and any liquid plumbing; risers/NIC/storage/retention and cable sweeps. A loose board does not prove completeness. Printed support replaces none of those electrical/bonding/cooling functions. Altra’s fan bank does not qualify GH200/dual-socket cooling; place every retained mass and qualify the larger span’s joint/creep loads.

Next: generalize fixed panels to one bed-constrained tile/edge graph, generating each shared joint once and deriving attachments; supply profile mounting/keep-out maps (never scale another board’s holes); place retained assemblies and derive routes/service volumes through common interfaces; realize through existing CadQuery. Preserve Altra as regression. `bed_partition.allocate_axis` owns feature/brim/clearance-aware cell counts; envelopes are not completed larger-board solids. No print or motherboard removal is requested.

Historical coordination: #13295 consolidates #13222 ancestry; #13283 extra-frame direction is superseded. Retain preparation/physical receipts; slicing #13223 and authorization #13149/approval-client remain separate operational changes.
