# Lotes AZIFA072-P001CCS — LGA4926 socket body

The socket-body part for the LGA4926 designation, and the customer drawing that carries its
board land pattern. This closes the constituent that `extdeps.cpu_attachment.ilm4926` had left
unresolved since the module was written: Lotes AZIF0222 is the ILM + backplate assembly, and
this is the *different* orderable item that receives the processor and solders to the board.

## Provenance

Supplied by Lotes in direct response to the request recorded in
`lga4926_land_pattern_public_route` ("the socket-body part number that mates with AZIF0222, then
its customer drawing or recommended PCB layout"). Not access-gated; no confidentiality marking
appears in the document. This is the public-route resolution, not the NDA route — the geometry
below is carriable.

| | |
|---|---|
| Part number | `AZIFA072-P001CCS` |
| Drawing number | `GAP-AZIFA072` |
| Title | LGA4926 SOCKET |
| Revision | 2 (ECN N230045, 04/10'2023) |
| Prior revisions | 1 → CNDR2100660 (04/15'2022), N220063 (07/20'2022) |
| Sheets | 5, A4, units mm |
| Drawn / checked / approved | WU YONGQUAN / ZHIHUA HUI / GRANDO ZHANG |
| 3D | `AZIFA072_B_20220208 (LGA4926 Socket).STEP`, AP214, SolidWorks 2019, mm |

## Files

- `lga4926_pcb_pads.csv` — 4,926 PCB pad centres in mm, extracted from sheet 3 (below)
- `lga4926_pad_array.svg` — the same array rendered, for eyeball verification

**Neither the drawing nor the STEP solid is committed, and the drawing's absence is a policy
decision rather than a size one.** `product.altra_motherboard.attachment_stack` states it on
`RepositoryCarriageStanding`, whose deliberately absent fourth arm is `SourceExpressionMayBeCarried`:
copying a vendor's PDF, figure, prose or table layout is refused for **every** subject, not chosen
per subject. The operator's clearance covered confidentiality — there is no NDA on this part —
which is a different question from whether this repository carries a vendor's expression of its
own facts. An earlier revision of this directory committed the PDF; both conditions had to hold
and only the first had been checked.

What is carried here is what `NormalizedFactsMayBeCarried` admits: the normalized numerical facts,
extracted and cross-checked. To obtain the source documents, ask Lotes for **GAP-AZIFA072
revision 2** (the STEP is `AZIFA072_B_20220208 (LGA4926 Socket)` = socket body
`AZIFA072_B_220208(LGA4926 SKT)-1` + protective cap `SKT4926 CAP_A_20220107`).

## The land pattern

Extracted from the sheet-3 pad view, whose vector content draws every pad individually.

| Property | Value | Source |
|---|---|---|
| Pad count | **4,926** | drawing callout `4926 X` **and** the extracted element count, independently |
| Pad diameter | **Ø0.46 ±0.05 mm** | sheet-3 callout `4926 X Ø0.46 ±0.05 PCB PADS`; confirmed by the sheet-4 detail circle measuring 0.4599 mm at its stated 80:1 |
| Lattice | **hexagonal / staggered**, alternate rows offset 0.5 mm in X | extracted |
| Column pitch | **1.000 mm** (within a row) | extracted; fitted 0.99704 of nominal |
| Row pitch | **0.870 mm** | extracted; = 1.000 × cos 30° = 0.8660 nominal |
| Nearest-neighbour spacing | **1.00 mm** | derived: √(0.5² + 0.87²) |
| Array extent | **63.449 × 75.470 mm** | extracted |
| Rows | 84, in two banks of 42 | extracted |
| Columns | 64 max per row | extracted |
| Bank gap | 4.13 mm, no row between, banks in different X phase | extracted |
| Central void | ≈13.9 mm wide × ≈25 mm tall, centred | extracted |

`lga4926_pcb_pads.csv` carries all 4,926 centres, origin at the array centre, +X right / +Y up
in the STEP part frame (verified — see cross-check).

### Solder ball / termination

| Property | Value | Source |
|---|---|---|
| Solder ball diameter, before SMT | **Ø0.52 mm** | sheet-2 callout `4926 X Ø0.52 SOLDER BALLS BEFORE SMT`; the STEP models 650 spherical faces at exactly r = 0.26 |
| Ball centre height above PCB | 0.26 mm (before SMT) | STEP |

## Cross-check — two independent sources agree

The drawing's vector pad array and the STEP solid's solder-ball centres were extracted by
separate paths and compared. They are the same array:

- array extent from the PDF: 63.449 × 75.470 mm; from the STEP: 63.44 × 75.47 mm
- of the 650 ball positions the STEP models, **642 land within 0.25 mm** of an extracted pad
- median deviation **0.019 mm**, p95 0.051 mm, max 0.49 mm

This is a real cross-validation, not a restatement: nothing in the PDF extraction consulted the
STEP, or the reverse. The eight outliers sit at the array's chamfered corners, where the
per-row index fit accumulates the most rounding.

## Socket outline and stack-up

All Z given relative to the PCB pad plane (= the solder-ball tangent before SMT), from the STEP.

| Feature | Value |
|---|---|
| Housing bottom flange | 70.900 × 80.900 mm, at +0.18 mm |
| Housing top | 72.300 × 82.300 mm, at +4.68 mm |
| Widest plane | 77.418 × 82.300 mm, at +2.38 mm |
| Overall height above PCB, before SMT | **4.68 mm** |

The housing is symmetric about the pad-array centre in both X and Y. The widest plane is **not**
symmetric: it reaches 41.268 mm on +X against 36.150 mm on −X, a 5.118 mm protrusion on one side
only. That is the actuation side, and it is the drawing's `2X FINGER ACCESS`. A board keepout
taken from the symmetric outline will be 5.1 mm short on that edge.

## What is on each sheet

1. Socket housing, section A-A, `4926 X CONTACT TIPS`, `4926 X CONTACT POINTS AFTER SMT`,
   `2X FINGER ACCESS`, `STANDOFF GAP`, `4926 X SOLDER BALL POSITION`, before/after SMT states
2. `4926 X Ø0.52 SOLDER BALLS BEFORE SMT`, section B-B, `MARKING FOR Pin1`, contact tip free
   height, date code, `PICK AND PLACE TOOLING KEEP-IN ZONE`, housing/cap 1.1 / 1.2, 3D view 1:20
3. `SOCKET HOUSING OUTLINE`, `PACKAGE OUTLINE`, `4926 X Ø0.46 ±0.05 PCB PADS` — **the land pattern**
4. `STENCIL APERTURE` and `PCB PAD` details at 80:1 — **the paste layer**
5. Packaging: 4 sockets per tray, 40 per inner box, 160 per outer box

## Known gaps — read these before releasing a footprint

1. **Stencil aperture dimension not extracted.** Sheet 4 carries it, but its dimension values are
   drawn as vector outlines rather than text, and the only closed shape recoverable at 80:1 is the
   Ø0.46 pad itself. Read the aperture off the PDF by eye before releasing paste.
2. **Pin A1 not located.** Sheet 2 carries `MARKING FOR Pin1`; its position was not extracted.
   The array is not symmetric — the two banks have visibly different depopulation patterns — so
   orientation is recoverable from the pad map, but the A1 corner has not been *established*
   against the drawing and must not be guessed.
3. **Solder-mask openings unconfirmed.** Sheet 3's second view draws the same 4,926-position
   lattice with a larger circle measuring ≈0.73–0.77 mm. That is most likely the mask opening or
   the contact-tip footprint, but no callout was recovered attaching a name to it, so it is
   recorded as unidentified rather than assumed.
4. **A few rows near the bank boundary.** Row clustering merged a handful of rows whose drawn
   Y positions sit closer than the 0.87 mm pitch. The total is exactly 4,926 and no position is
   duplicated or dropped, so this affects which row a pad is *labelled* with, not where it is.
5. **The interior is not derivable from the STEP alone.** The solid models only the perimeter of
   the ball field — 650 of 4,926. A naive lattice fill of its outline gives 5,285, over by 359.
   The interior voids come from the drawing and only from the drawing.

## Method

Both extractions are reproducible from the drawing named above, which is not committed here (see
**Files**). The PDF's page content was decompressed,
its object streams expanded, per-font `ToUnicode` CMaps applied for text, and path operators
replayed under a tracked CTM for geometry; pads are the 9-vertex closed subpaths of the sheet-3
upper view, and scale was fixed by fitting the within-row column pitch to 1.000 mm and confirmed
independently against the 80:1 detail circle.
