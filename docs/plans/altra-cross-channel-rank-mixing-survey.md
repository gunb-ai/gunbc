# Cross-channel 1R/2R DIMM mixing on Ampere Altra: published-documentation survey

Survey date: 2026-09-11. Lane: swift-bat-341, work item `node://adhoc-d3527d08-755`.

## Verdict

**Not resolved anywhere in published documentation.** No cited authority affirms cross-channel
1R/2R rank mixing on Altra / Altra Max, and none forbids it. The only cell that names the
configuration remains OCP Mt. Jade Rev 1.0 §7.4 Table 3, *Different Channel × Mixing 1R & 2R*
= **TBD** (`extdeps.ocp.mt_jade.memory_mixing` `mt_jade_mixed_dimm_support_table`). Every other
document read is silent on rank as a mixing axis.

What the survey DID establish, at its real strength:

1. **Ampere's own platform guides state a cross-channel CAPACITY rule, not a rank rule.**
   "DIMM modules on all channels on a socket must be identically sized" — Mt. Collins GSG Issue
   1.05 p.10 (already carried by `extdeps.ampere.mt_collins_product_brief.memory_population`) and,
   in identical wording, Mt. Jade PVT/DVT (NVMe) GSG Issue 1.00 p.11. "Sized" is capacity; a 16 GB
   1R and a 16 GB 2R module satisfy it. Neither sentence says anything about rank.
2. **No later revision of the OCP Mt. Jade specification is discoverable.** Search-engine index
   and the Internet Archive CDX index of `opencompute.org/documents/` both carry only
   `open-compute-specification-mt-jade-rev-1-0-pdf-1`. The OCP contribution database itself
   answers HTTP 403 to unauthenticated fetches (live and via WebFetch), so "no Rev 1.1 exists" is
   asserted only at the strength of two indexes, not of the database. The successor contribution
   (Mt. Mitchell v0.80, `extdeps.ocp.mt_mitchell`) is DDR5 and does not carry a DDR4 Table 3.
3. **Other Altra platform vendors publish board-level homogeneity guidance, none of it about rank
   across channels, and none of it a statement about the SoC.** See the table below.
4. **A firmware log line seen on the Ampere community forum is NOT evidence about rank.** The
   Altra DRAM init firmware prints `ERROR: Non-identical DIMM mixture NOT supported!` as its
   terminal line on a 2R×4 64 GB + 1R×8 16 GB cross-channel population (thread 564, ADP ES2,
   DRAM FW 211207) — but the specific error above it is `ERR: Mismatch Capacity MCU1`, i.e. the
   capacity rule in (1), and the same terminal line is printed on an ALTRAD8UD-1L2T with two
   IDENTICAL 1R×4 modules whose real fault was `Unsupported CL 22` / bad RAM (thread 3309, DRAM FW
   220414). The line is a generic memory-init failure summary, does not isolate rank, and is a
   forum observation rather than a publication. It is recorded here so nobody re-finds it and
   reads it as a prohibition.

Nothing below licenses the running configuration. Mt. Collins unit 1's clean EDAC under load is
a gunbc observation about one machine, not an upstream fact, and per the brief it is not minted
into `extdeps`. The purchase decision remains gated on an answer the vendor has not given.

## Documents read

Every row was fetched by this session on 2026-09-11 and read as text (pdf-parse, npm 1.1.1;
scrape verified against the Read-tool where available). Digests are sha256 of the fetched bytes.
Locators are the publisher's; `connect-admin.amperecomputing.com` downloads returned 200 with no
credentials.

### Ampere Computing (SoC vendor — statements about the SoC or its reference platforms)

| Document | Revision / date | Locator | sha256 | What it says about mixing |
|---|---|---|---|---|
| Altra Rev A1 Datasheet | Issue 1.50, 2024-01-30 | `amperecomputing.com/assets/Altra_Rev_A1_DS_v1_50_20240130_3375c3dec5_1c5d4604fa.pdf` | `6adeb858…cbbe9` | §2.10 *Eight DDR4-3200 SDRAM Memory Controllers*, p.10–11: up to 2DPC; x4/x8; 8 Gb/16 Gb devices; RDIMM/LRDIMM/3DS; 1/2/4/6/8 active channels; "Hashed memory interleave across active channels". **No mixing or rank rule.** |
| Altra Rev A1 Datasheet | Issue 1.55, 2025-04-08 | `connect-admin.amperecomputing.com/…?file=Altra_Rev_A1_DS_v1_55_20250408_e814f50701.pdf&type=technical-document&documentId=h51y6okvdig3i4u3lst6n120` | `3b14a78a…1a43d` | §2.10 identical to 1.50. **No mixing or rank rule.** |
| Altra Max Rev A1 Datasheet | Issue 1.25, 2024-01-30 | `amperecomputing.com/assets/Altra_Max_Rev_A1_DS_v1_25_20240130_73cfcc518a_4705c00046.pdf` | `9aaa3dfb…c923c` | §2.10 same feature list. **No mixing or rank rule.** |
| Altra Max Rev A1 Datasheet | Issue 1.30, 2025-04-08 | `connect-admin.amperecomputing.com/…?file=Altra_Max_Rev_A1_DS_v1_30_20250408_99487b8f45.pdf&type=technical-document&documentId=z7tgvww8kuqd9ofrrwq381df` | `5b0e7981…6ef89` | Same. **No mixing or rank rule.** |
| Altra Platform Hardware Design Specification | Issue 1.00, 2021-01-20 | `d1o0i0v5q5lp8h.cloudfront.net/ampere/live/assets/processor_families/design_specifications/Altra_Platform_HW_Design_Specification_v1.00_20210120.pdf` | `d564e2b4…4666` | §2.2 *Memory Subsystem* p.13–16: 1DPC/2DPC topology figures, signal connectivity (CS0–3 per channel, "X1 DIMM (Quad Ranks)" in the figures), §2.2.3 slot ordering. **No population or mixing rule of any kind.** |
| Altra Family Approved Vendor List | Issue 1.15, December 2025 (AMP 2025-0116) | `connect-admin.amperecomputing.com/…?file=Altra_Family_AVL_Dec_2025_v1_15_20251203_a11736f7be.pdf&type=technical-document&documentId=ppuyl31kktnze6gbof70xitc` | `6199ae84…7c69f` | §1.3 DDR AVL test suite tests each part at 1DPC (8/socket) and 2DPC (16/socket) — **homogeneous populations only**. Table 3 lists parts with ORGANIZATION (1Rx4, 2Rx8, 2Rx4, 4Rx4). No mixed-population row and no mixing note. SK hynix HMA82GR7CJR**4**N-VK (1Rx4) is listed; HMA82GR7CJR**8**N-VK (2Rx8, the unit-1 part per gunbc#11066) is not. |
| Mt. Collins DVT/PVT/MP Getting Started Guide | Issue 1.05, 2026-07-20 | `extdeps.ampere.mt_collins_getting_started_guide.subject` `mt_collins_gsg_authority` | `1c41b4f0…c580b3` (matches `mt_collins_gsg_content_digest`) | p.10, prose above Table 10: "memory on the Socket0 channels is not required to be the same size as the memory on the Socket1 channels. However, DIMM modules on all channels on a socket must be identically sized." Tables 10–13 are channel-count and slot-sequence tables. **Capacity rule only; silent on rank.** |
| Mt. Jade PVT/DVT (NVMe) Getting Started Guide | Issue 1.00, 2022-04-12 | `connect-admin.amperecomputing.com/…?file=new-private-files/94/tech/Altra_Family_Mt._Jade_PVT_DVT_NVMe_GSG_v1.00_20220412.pdf&type=technical-document&doc_id=631` | `f6528f70…4c69f` | p.11, prose above Table 11: same two sentences verbatim. **Capacity rule only; silent on rank.** |
| Mt. Collins 2U User Guide and Hardware Maintenance Manual | Issue 1.00, 2022-03-30 | `connect-admin.amperecomputing.com/…?file=new-private-files/94/tech/Mt._Collins_2U_Users_Guide_v1.00_20220330.pdf&type=technical-document&doc_id=649` | `a8102a2a…78bbf` | §4.2.7 *Memory Module Replacement* p.52, Table 6 channel configurations. **No mixing rule.** |
| Mt. Collins 1U User Guide and Hardware Maintenance Manual | Issue 1.00, 2022-03-30 | `…Mt._Collins_1U_Users_Guide_v1.00_20220330.pdf&type=technical-document&doc_id=669` | `7f351cc3…11e7c` | Same section shape. **No mixing rule.** |
| Mt. Collins 2U Product Brief | v0.50, 2021-02-23 | `…Ampere_Mt._Collins_2U_PB_v0.50_20210223.pdf&type=technical-document&doc_id=597` | `647f1cd8…90adc` | Feature list only. |
| Mt. Jade Product Brief | v0.65, 2020-11-02 | `…Mt._Jade_PB_v0.65_20201102.pdf&type=technical-document&doc_id=527` | `384f2023…b9e5` | Feature list only. |

The `altra-family-device-documentation` Customer Connect page lists, beyond the rows above, only the
SoC–BMC Interface Specification, Monitoring Events, OS Compatibility, product briefs and two
solution reference architectures — none is a memory population document.

### OCP (the platform specification that carries the TBD)

| Document | Revision | Locator | What it says |
|---|---|---|---|
| Mt. Jade Ampere Altra/AltraMax Motherboard Specification | Rev 1.0 | `extdeps.ocp.mt_jade.subject` (publisher 403; Internet Archive capture 20220603132323) | §7.4 Table 3: *Different Channel × Mixing 1R & 2R* = **TBD**; all six other different-channel cells Yes. Modelled at `extdeps.ocp.mt_jade.memory_mixing`. |
| Later Mt. Jade revision | — | Wayback CDX `url=www.opencompute.org/documents/*` filtered on jade/collins/ampere/altra/snow returns only the Rev 1.0 locator; search engines return only Rev 1.0; `opencompute.org/contributions` answers 403 unauthenticated. | **None found.** |
| Mt. Mitchell OCP Hardware Spec | v0.80 | `extdeps.ocp.mt_mitchell.subject` | DDR5 successor platform; no DDR4 mixing table. |
| OCP-Server mailing list (`ocp-all.groups.io/g/OCP-Server`) | — | Search endpoint redirects to a bot probe (302 → `/probe`; WebFetch 402). | **Not searchable from this session.** Nothing about the cell is indexed by search engines. |

### Other Altra platform vendors (BOARD-level guidance; evidence about the vendor's board, not the SoC)

| Vendor / board | Document | Locator | sha256 | What it says |
|---|---|---|---|---|
| Gigabyte R272-P30 (Altra, 1P, 16 DIMM 2DPC) | User Manual Rev 1.0 | `download.gigabyte.com/FileList/Manual/server_manual_e_R272-P30_v10.pdf` | (19.1 MB; not digested) | §3-5-1 p.26: "It is recommended that memory of the same capacity, brand, speed, and chips be used." §3-5-3 DIMM Population Table lists SRx4 and DRx8 16 GB RDIMMs as supported types. §3-5-4 channel-population table. **A recommendation, not a rule; rank is not named.** |
| ASRock Rack ALTRAD8UD-1L2T (Altra, 1P, 8 DIMM 1DPC) | User Manual v1.0, July 2023 | `download.asrock.com/Manual/ALTRAD8UD-1L2T.pdf` | `2160675a…3848` | §2.4 p.24: "For Eight channel configuration, it always needs to install identical (the same brand, speed, size and chip-type) DDR4 DIMM groups." Troubleshooting p.~89 repeats it. **Board-level identical-DIMM rule for the 8-channel configuration; "chip-type" is not rank; nothing about the SoC.** This is the board gunbc already models at `extdeps.boards.asrock_rack`. |
| Supermicro R12SPD-A / ARS-210M-NR (Altra Max, 1P, 16 DIMM 2DPC) | User Manual (read via manualslib.com mirror of MNL-2483; supermicro.com answers 403 to this session) | `manualslib.com/manual/3256479/Supermicro-Superserver-Ars-210m-Nr.html?page=41` | — | p.41 *General Guidelines for Optimizing Memory Performance*: "It's recommended to use DDR4 memory of the same type, size, and speed. Mixed DIMM speeds can be installed. However, all DIMMs will run at the speed of the slowest DIMM." **Recommendation; rank not named. Mirror, not publisher — re-read from `supermicro.com/manuals/motherboard/R12/MNL-2483.pdf` before citing.** |
| Wiwynn SV328R (Mt. Jade-derived 2P) | Datasheet 2021-11-04 | `wiwynn.com/hubfs/Datasheet-SV328R_211104_internal.pdf` | `b79b6bbc…d7649` | "32 DIMM slots (16 DIMM per processor); DDR4-3200, 8 channels per processor, RDIMM". **No population rules published.** |
| Foxconn/FII Mt. Collins | — | Only SPEC power results and the Ampere-hosted guides above are public. | Nothing beyond the Ampere GSG. |

### Community (observations, not publications)

| Source | What it shows |
|---|---|
| `community.amperecomputing.com/t/564` (2023-12, Altra Developer Platform ES2, DRAM FW 211207) | Cross-channel population of 2×(64 GB 2R×4) + 4×(16 GB 1R×8) refused at DRAM init: `ERR: Mismatch Capacity MCU1` then `ERROR: Non-identical DIMM mixture NOT supported!`. Capacity differed as well as rank, so rank is not isolated. |
| `community.amperecomputing.com/t/3309` (2026-01, ALTRAD8UD-1L2T, DRAM FW 220414) | Two identical 16 GB 1R×4 modules refused with `ERR: Unsupported CL 22 on DIMM` followed by the same `Non-identical DIMM mixture NOT supported!` line; resolved by replacing the RAM. Shows the line is a generic terminal summary. |
| Discourse search for "mixing DIMM", "1R 2R", "rank" | No thread addresses rank mixing on Altra. |

## What would resolve it

Only one of: (a) an Ampere document that names rank as a population axis (none of the public ones
does — the SoC datasheets stop at device width and density; the platform guides stop at capacity);
(b) a later OCP Mt. Jade revision filling the TBD cell, which no index shows existing; (c) an answer
from Ampere through a channel that responds. The DRAM init firmware evidently enforces some
identity rule across channels, but what it enforces is not published, and reading its refusals
off our own unit is observation, not documentation.

## Follow-ups this survey does not land

- The Altra datasheet §2.10 feature list and the Platform HW Design Specification §2.2 are SoC-level
  memory facts with no `extdeps` subject module yet; they are worth landing as
  `extdeps.ampere.altra_datasheet` / `…altra_platform_hw_design_specification` if a consumer appears
  (the "hashed interleave across active channels" and "1/2/4/6/8 active channels" rows are the
  candidates). Not landed here because this lane has no consumer for them and DESIGN §3c refuses
  dangling declarations.
- The Mt. Jade GSG Issue 1.00 carries the same capacity sentence as the Mt. Collins GSG; if the
  capacity rule is ever modelled as a rule (today it is prose in
  `extdeps.ampere.mt_collins_product_brief.memory_population`), it has two independent citations.
