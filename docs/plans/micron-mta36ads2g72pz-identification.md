# MTA36ADS2G72PZ-2G1A1: what Micron says it is, and what the Ampere documents say about it

Survey date: 2026-09-12. Lane: clever-heron-234, work item `node://adhoc-677e2a53-ae7`.
Landed authority: `extdeps.memory.micron` (rows `mta36ads2g72pz_2g1a1_catalog`,
`mta36asf2g72pz_2g1a2_catalog`), `extdeps.vendor.micron`, and the third value of
`extdeps.memory.types` `DramDieStacking`. This document carries the reading; the rows carry the
facts. Nothing observed on Mt. Collins unit 1 is authored into `extdeps`.

## Verdict, in the brief's order

1. **MTA36ADS2G72PZ-2G1A1 is a 16 GB, 2-rank, x4, DDR4-2133 CL15, ECC, registered,
   VERY-LOW-PROFILE (18.75 mm) RDIMM built from 18 DUAL-DIE packages of two 4 Gb dies each.**
   Every word of that is Micron's, from two documents (part-numbering decode and the SPD image
   Micron publishes for the part), and the two agree at byte grain. Status: obsolete.
2. **ADS versus ASF is `DS` = "VLP Dual-Die w/Temp Sensor" versus `SF` = "FBGA w/Temp
   Sensor".** Micron's DDR4 Module Part Numbering System (Rev. 05-Dec-2018) defines the
   module-options field as a two-letter code where the first letter is the PCB height class
   (`S` standard, `D` VLP, `H` 2U, `L` 4U) and the second the DRAM package (`F` FBGA
   single-die, `S` dual-die, `Q` quad-die, `E` octal-die). The letter difference IS the answer:
   the ADS part is the VLP dual-die build of the same 36-die, 2 Gig x 72, RDIMM, -2G1 family.
3. **"2DRx4" reads as "2 ranks, dual-die, x4", and that is verified, not adopted.** The
   operator's hypothesis was that a VLP module needs dual-die packages to reach 16 GB with x4
   devices in 18.75 mm. Micron's SPD byte 6 for the part is `0x91`: non-monolithic, 2 dies,
   MULTI LOAD STACK, i.e. the JEDEC class a DDP belongs to. Byte 128 (`0x04`, nominal height
   ≤ 19 mm) confirms the VLP outline. The catalogue's "Component Config: 2Gb x4" names the
   8 Gb dual-die package; "Number of Components: 36" counts dies, matching the `36` in the
   part number, whose field the numbering sheet labels "Number of Die".
4. **Ampere publishes nothing that admits or refuses a dual-die (multi-load stack) RDIMM.**
   Altra Rev A1 Datasheet Issue 1.55 §2.10 lists "RDIMMs, LRDIMMs, and Stacked (3DS) RDIMMs"
   and "8 Gb and 16 Gb DRAM devices"; the words DDP, dual-die, multi-load and 4 Gb do not
   appear in the document. Neither does the Mt. Collins GSG, the OCP Mt. Jade spec, or the
   AVL say anything about package construction. The OCP Table 3 "Different DRAM DIES" = Yes
   cells govern MIXING die types across modules; they do not say a given die construction is
   supported, and this survey did not let them. Note also that the datasheet's device list
   (8 Gb, 16 Gb) does not name the 4 Gb die that BOTH Micron quartets use, and the monolithic
   quartet trained; so "unlisted device density" is a fact about both quartets and cannot be
   what separates them. The corpus now models the DDP class as *not among the listed classes*
   (`altra_admits_module_class` answers false for Registered × DualDiePackage) and the
   controller module's own prose keeps that distinct from a withdrawal such as UDIMM's.
5. **Neither part is on the Ampere Altra Family AVL Issue 1.15 (December 2025).** Table 3's
   Micron rows are MTA18ASF2G72PDZ (16 GB 2Rx8), MTA18ASF2G72PZ-2G9E1 (16 GB 1Rx4),
   MTA36ASF4G72PZ (32 GB 2Rx4, three suffixes) and MTA36ASF8G72PZ-3G2E1 (64 GB 2Rx4) — all
   `SF`, all 8 Gb/16 Gb dies, all 2666 or faster. There is NO 16 GB 2Rx4 row from any vendor
   and no 2133 row at all, so the AVL is equally silent about the ADS part and the ASF part
   that trained. An AVL omission is not a prohibition and this survey does not read it as one.

**Firmware corroboration, received from hw-first-host after this survey was drafted (SOL capture
`artifacts/bmc/mtcollins1-sol-dram-training-2026-09-12.txt`, sha256 `e5044f71…8a4c`, on that
lane's branch).** The Altra DRAM-init firmware prints the ADS part as
`RDIMM[2c:80] 16GB 2133 ECC 2R x4 RCD[b3:80] 36ADS2G72PZ-2G1A1` — rank count 2, exactly as
Micron's byte 12 says, the `D` being a package letter and not a rank — and refuses the mixed
socket with `ERR: CHANNEL Mismatch Byte 6 SLOT0[00] EXP[91] @MCU[4]` followed by
`ERROR: Non-identical DIMM mixture NOT supported!`. Two things follow, both about the observing
layer and neither authored into `extdeps`: (a) the module's SPD byte 6 reads `0x91` on the
board, agreeing with Micron's published image; (b) what the firmware enforces on that socket is
**byte-6 agreement across the socket's channels**, i.e. a MIXING rule, so the failure with
4 × ADS + 12 monolithic is a mixed-population refusal and says nothing about a uniform ADS
population, which has not been run. The trained quartet's label reads `36ASF2G72PZ-2G1A2II`,
matching the catalogue SKU carried here plus process code `II`. The firmware's `RCD[b3:80]` /
`RCD[32:86]` per-module variation is the same variation Micron's record carries across process
codes (bytes 133–135), which is why register vendor is not a field of the catalogue row.

**What Micron's data leaves as the only differences between the two quartets:** package
construction (dual-die multi-load stack vs monolithic), PCB outline (18.75 vs 31.25 mm) and, with
the outline, the JEDEC raw-card reference design (SPD byte 130 `0x08` vs `0x01`) and the
module-attribute byte 131 (`0x05` vs `0x09`). Everything else — vendor, generation, capacity,
speed bin, CAS latency, ECC, buffering, device width, package-rank count, die density, DRAM
addressing — is byte-identical between the two SPD images. Which of those the Mt. Collins
training outcome tracks is not decided by any document read here; the firmware line above
names byte 6 directly, and that receipt belongs with the lane that ran the crossovers.

## Documents read

| Document | Revision | Locator | Digest / standing | What it establishes |
|---|---|---|---|---|
| Micron part catalogue record, MTA36ADS2G72PZ-2G1A1 | database record fetched 2026-09-12 | `www.micron.com/products/obsolete/obsolete-vlp-rdimm/part-catalog/part-detail/mta36ads2g72pz-2g1a1` (renders `…/_jcr_content.products.json/getproductinfo/-/-/-/en_US/-/mta36ads2g72pz-2g1a1`) | HTTP 200, no credentials, `"data-source": "DATABASE"`, `"sub-category": "obsolete-vlp-rdimm"` | Details table and full DDR4 SPD image (12 process codes JG/HK/JJ/JI/HG/HJ/II/HI/IG/IJ/IK/JK; they differ only at register-vendor bytes 133–135 and the bytes-128..253 CRC). |
| Micron part catalogue record, MTA36ASF2G72PZ-2G1A2 | same | `www.micron.com/products/obsolete/obsolete-rdimm/part-catalog/part-detail/mta36asf2g72pz-2g1a2` | HTTP 200 | The control's details table and SPD (process codes KI/KJ/KK/KG/II/IG/IJ/IK). The bare `mta36asf2g72pz-2g1` record answers with no process codes. |
| Micron DDR4 Module Part Numbering System | Rev. 05-Dec-2018 | `siewert-kau.info/micron/wp-content/uploads/2020/05/Nomenklatur_Micron.pdf` (distributor copy; `micron.com/numbering` → `assets.micron.com` answers "Request Rejected" to this session) | sha256 `7d52c2ad…5c6a2` | Field decode: family, die count, voltage, module options (`DS` = VLP Dual-Die w/Temp Sensor, `SF` = FBGA w/Temp Sensor), configuration, module type (`P` = 288-pin RDIMM), package code (`Z`), speed grade (`-2G1` = DDR4-2133, PC4-2133, 15-15-15, component -093E), die and PCB revision. |
| Micron 288-Pin DDR4 RDIMM Core Product Description | Rev. B 12/2021, CCM005-341111752-10541 | `media.digikey.com/pdf/Data%20Sheets/Micron%20Technology%20Inc%20PDFs/DDR4_SDRAM_RDIMM_Core_RevB_Dec2021.pdf` (DigiKey copy of Micron's PDF; Micron's own host rejects) | sha256 `ea054b3d…8abc` | Figure 1 RDIMM 31.25 ± 0.15 mm; Figure 2 VLP RDIMM 18.75 ± 0.15 mm; Table 12 module ↔ component speed grades; C0–C2 pin text distinguishing "a traditional DDP package, which uses CS1_n, CKE1, and ODT1 to control the second die" from single-load 4H/8H stacks. It says the per-part organization lives in the MPN-specific addendum. |
| Micron per-part addendum for MTA36ADS2G72PZ | — | **Not found.** Probed `…/data-sheet/modules/{vlp_rdimm,rdimm,parity_rdimm}/ads36a2gx72pz.pdf` and variants (404); search engines index no addendum for the ADS family; the catalogue page's download resource answers 404. | — | The SPD image is the strongest first-party organization statement reachable; the addendum would add placement and thermal figures, not organization. |
| Ampere Altra Rev A1 Datasheet | Issue 1.55, 2025-04-08 | `extdeps.cpu.ampere` `ampere_altra_rev_a1_datasheet_authority` (v1.55 locator) | 90 pages, read as text | §2.10 feature list; no DDP / dual-die / multi-load / 4 Gb text anywhere in the document. |
| Ampere Altra Family AVL | Issue 1.15, December 2025, AMP 2025-0116 | `extdeps.cpu.ampere` `ampere_altra_avl_authority` | sha256 `6199ae84…8d0d` (matches swift-bat-341's read) | Table 3 Micron rows listed above; no 16 GB 2Rx4 row; no 2133 row; §1.3 test suite is homogeneous 1DPC/2DPC. |
| OCP Mt. Jade Motherboard Spec | Rev 1.0 | `extdeps.ocp.mt_jade.memory_mixing` | already modelled | Table 3 is a MIXING matrix; "Different DRAM DIES" = Yes is not a construction-support statement. |
| swift-bat-341 survey | 2026-09-11 | [altra-cross-channel-rank-mixing-survey.md](altra-cross-channel-rank-mixing-survey.md) | — | Starting point for the AVL and the platform-guide reads; nothing there is contradicted here. |

### Reading the two SPD images side by side

| Byte | Meaning (JESD21-C Annex L) | ADS-2G1A1 | ASF-2G1A2 |
|---|---|---|---|
| 2 | DRAM device type | `0C` DDR4 | `0C` |
| 3 | Module type | `01` RDIMM | `01` |
| 4 | Density and banks | `84` 4 Gb/die, 4 BG × 4 banks | `84` |
| 5 | Addressing | `21` 16 rows, 10 columns | `21` |
| **6** | **SDRAM package type** | **`91` non-monolithic, 2 dies, multi-load stack** | **`00` monolithic** |
| 12 | Module organization | `08` 2 package ranks, x4 | `08` |
| 13 | Bus width | `0B` 64 + 8 ECC | `0B` |
| 18 / 125 | tCKmin / fine offset | `08` / `C2` → 938 ps (2133) | same |
| 24 | tAAmin | `6C` 13.5 ns → CL15 | same |
| **128** | **Nominal height** | **`04` ≤ 19 mm** | **`11` ≤ 32 mm** |
| 129 | Max thickness | `11` | `11` |
| **130** | **Raw card** | **`08`** | **`01`** |
| **131** | **RDIMM attributes** | **`05`** | **`09`** |
| 329–349 | Part number | `36ADS2G72PZ-2G1A1` | `36ASF2G72PZ-2G1A2` |

Raw card letters and bytes 131/133–135 are recorded as bytes, not decoded: the JEDEC raw-card
letter table and the JEP106 register-vendor table are not authorities the corpus carries yet,
and reading them from memory would be exactly the uncited transcription §4d forbids. A JEDEC
SPD authority module (`extdeps.memory.jedec` currently carries only the generation ceilings) is
the named follow-up that would make the decode above typed rather than prose.

## The OEM SEL family (second half)

Payloads observed on Mt. Collins unit 1, all manufacturer `3a cd 00`, record type `C0`, raw
bytes preserved exactly as the brief carries them:

```
0fde7033ff10   0fde703bff10        2026-08-19, same timestamp, adjacent record IDs
0fde70330102 (x19)   0fde70330502 (x2)
0fde70331102         0fde703b1102
```

### What is public

The only public serializer of a `3a cd 00` / `C0` OEM SEL record with this byte shape is
Ampere's OpenBMC layer, `meta-ampere/meta-common/recipes-ampere/host/ac01-boot-progress/dimm_train_fail_log.sh`
in `github.com/ampere-openbmc/openbmc` (read at `f68588807957ffbc07371634ed41e534e7b416c2`,
2026-01-22; the script has three commits in its history and was created 2023-04-15 already
carrying the constants below — there is no earlier revision with a different sensor number).
It emits, via `IpmiSelAddOem` with OEM id `0x3a 0xcd 0x00` and SEL type bytes
`0x00 0x00 0x00 0xC0`:

```
byte 0  SENSOR_TYPE_SYSTEM_FW_PROGRESS = 0x0F
byte 1  SENSOR_BOOT_PROGRESS           = 235 = 0xEB
byte 2  EVENT_DIR_ASSERTION | OEM_SENSOR_SPECIFIC = 0x00 | 0x70
byte 3  (channel << 4) | (socket << 3) | BOOT_SYNDROME_DATA(=4)
byte 4  syndrome bits 15:8   (the SoC-BMC "Boot Stage Error Syndrome" byte 1)
byte 5  syndrome bits  7:0   (byte 0: [1:0] failure type, [4:2] physical rank, [7:5] syndrome 0)
```

The syndrome bytes it copies are defined by the Ampere Altra Family SoC-BMC Interface
Specification, Issue 1.43 (2023-05-30), Table 16 registers `0xB4` (slot select) and `0xB5`
(Boot Stage Error Syndrome): failure type 1 = PHY training, 2 = DIMM training; syndrome 0 for
PHY = setup / write leveling / read gate leveling / read leveling / software training;
syndrome 0 for DIMM = VREFDQ / LRDIMM DB / LRDIMM DB software; byte 1 for write leveling =
slice number and per-nibble "no rising edge" bits. The specification is a `connect-admin`
download that answered HTTP 200 without credentials (sha256 `34ccdcd2…`, 56 pages). The
companion "Altra Family Monitoring Events" v0.60 (2023-06-05, sha256 `23d84c2c…`) is a
Redfish message registry and defines no SEL byte layout.

### What is not public, and therefore not decoded

- **Sensor `0xde` (222) is not in any public Ampere source.** `ampere-openbmc/openbmc`,
  `ampere-ipmi-oem`, and the Mt. Jade / Mt. Mitchell IPMI sensor YAMLs were searched; 222 and
  235 appear there only as ordinary temperature/current sensor numbers unrelated to boot
  progress. Mt. Collins runs AMI MegaRAC (`extdeps.ampere.mt_collins_product_brief.bmc`), not
  this OpenBMC layer, and AMI's Ampere port is not published. So the `0xde` serializer is
  **not found**.
- **Byte 3's low nibble is `3` where the public schema writes `4`**, and the sensor number
  differs, so the OpenBMC layout is a *related* schema and no byte of ours may be read through
  it as a decode. Under that layout byte 3 = `0x33` would be channel 3, socket 0; `0x3b`
  would be channel 3, socket 1 — which is consistent with the operator's inference that bit 3
  moved with the socket holding the intervened population across the two crossovers, and is
  still an inference: the companions moved with it.
- **The tail is ambiguous even under the cousin schema**, and the ambiguity is why it is not
  promoted. Reading `11 02` as `[syndrome hi][syndrome lo]` gives failure type 2 (DIMM
  training), rank 0, syndrome0 = 0 with byte 1 = `0x11`; reading it as `[lo][hi]` gives failure
  type 1 (PHY training), rank 4, syndrome0 = 0, byte 1 = `0x02` (write-leveling slice 2). The
  two readings name different training steps and different ranks. Only a decode of register
  `0xB5` on this platform, or the AMI serializer, settles it. The `ff 10` pair from 2026-08-19
  reads as failure type 3 (Reserved) under one ordering and failure type 0 (N/A — no failure)
  under the other, neither of which is a syndrome a training-failure record should carry, which
  is further evidence that the `0xde` family is not simply the `0xeb` family under another
  sensor number.

So the payloads stay raw. `gunbc.host_memory_qualification` `TrainingEvidenceStanding` already
has the right arms for what a receipt may claim: `AssociatedByIdenticalSignature` at best
(signature = the raw six bytes, source = the meta-ampere script as the related schema), never
`DecodedFromPlatformSyndrome`, until `0xB5` is read on the unit.

## What this lane did not land, and why

- **No `HostMemoryStanding` or `CausalExperimentReceipt` rows for the eight Mt. Collins
  configurations.** The brief summarises them (durations, wattage, temperature, payloads) but
  carries no instants, no DIMM masks, no slot locators; authoring receipts from a summary would
  fabricate `observed_at` values and put prose in the mask field — the exact defect that module's
  own notes record. The rows belong to the lane that holds the raw BMC log; they now have a
  catalog row to bind to (`ObservedDimmCataloged { catalog: mta36ads2g72pz_2g1a1_catalog }`
  reconciles, exercised by `test.claim.micron_dram_module`).
- **No typed SEL/OEM-record carrier.** One night's payloads under an unpublished schema are
  not an upstream authority. A carrier is warranted when either the AMI serializer or a `0xB5`
  read exists to ground it.
- **No `form_factor`/height field on `DramModuleCatalogRow`.** The VLP fact is carried in the
  row's annotation and here; the field lands with its first consumer (a chassis DIMM-clearance
  check), per §3c.
- **The Micron-hosted locators.** Both Micron PDFs are cited through distributor copies with
  digests because `assets.micron.com` rejects this session's fetches; re-cite to `micron.com`
  when reachable. The catalogue records ARE Micron-hosted.
