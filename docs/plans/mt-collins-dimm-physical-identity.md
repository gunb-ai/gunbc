# Mt. Collins DIMM physical identity: documentary findings

Read 2026-09-11 for `node://adhoc-ca3ce4b5-182`. The upstream findings below use
only publisher documents. No machine was accessed. A later operator report is
carried separately in gunbc, as described below; it supplies no extdeps fact.
The starting citations were `docs/plans/altra-cross-channel-rank-mixing-survey.md`.

## What can be located without reading silkscreen

Ampere's **Mt. Collins GSG Issue 1.05 Figure 9 (p.11)** labels all 32 connectors and
places their banks around CPU0 and CPU1. **Figure 10 (p.13)** labels the processors
Socket0 and Socket1 in a chassis photograph, with the NVMe backplane and six fans
at the top and PCIe risers at the bottom. With that orientation, Socket0 is left
and Socket1 is right. They are side by side, not front and back.

The model carries the Figure 9 bank order in
`extdeps.ampere.mt_collins_getting_started_guide.dimm_layout.dimm_figure_banks`.
The processor grouping and channel identity are resolved by J-number against
`extdeps.ampere.mt_collins_product_brief.memory_population.mt_collins_channel_connector_pairs`;
no second channel table is authored. `gunbc.mt_collins_dimm_physical_identity.resolve_dimm_diagram_identity`
refuses missing or repeated keys and disagreements in processor socket.

The **document-derived positional reading** is: view from above with the front/backplane
away from you and the rear/riser end toward you; then use Figure 9's order within
each CPU's left and right banks. Joining that order with the existing Table 11
16-DIMM row selects alternating connectors starting at the **outermost connector
away from each CPU** on both sides. This is a spatial inference from Figures 9
and 10 plus Table 11, not a vendor sentence instructing an installation order,
a photograph of the operator's board, or a physical colour rule. The model exposes
the source bank order and population join rather than minting this inference as
an independently authored slot list.

## Colour: three distinct facts

1. **Drawing legend:** GSG Issue 1.05 Figure 8 (p.10), 1U User Guide Issue 1.00
   Figure 10 (p.39), and 2U User Guide Issue 1.00 Figure 19 (p.52) all use **yellow
   for slot#0, used at 1DPC**, and **blue for slot#1, added at 2DPC**. The numbered
   J annotations agree with the existing odd-J 1DPC population. The model derives
   each connector's legend role from the existing population-role join.
2. **Photographed plastic:** GSG Issue 1.05 Figure 12 (p.36) visibly contains blue
   and black connectors/latches. It is a cropped board-revision photograph, not
   an annotated complete DIMM colour map. This visual fact is carried at its
   actual grain in `motherboard_photo_colours`.
3. **Meaning of physical colour:** **not found in the inspected documents.**
   None establishes that physical blue means slot#0, slot#1, or the 1DPC set.
   The drawing's blue cannot be translated into the chassis's blue. A readable
   photograph identifying actual connector positions, or explicit vendor
   documentation of the physical colour scheme, is still needed for that answer.

This does not establish why a processor failed to train, and does not select a
hypothesis about the running machine.

## Silkscreen: not established for all connectors

GSG Figure 9 supplies **diagram annotations** J1–J32, not a silkscreen view.
Figures 10 and 12 show installed DIMMs and small/obscured board text. The 1U guide
Figures 11 (p.40), 24–25 (pp.52–53), and the 2U guide Figures 20 (p.53), 42–43
(pp.74–75) illustrate replacement without an all-connector readable silkscreen
map. No inspected source establishes the literal text printed beside **each**
DIMM connector. No row therefore claims that a connector's physical silkscreen
reads its J-number. The old table-only comment saying connectors were named
on the board is narrowed to what it actually knew: the guide's identifiers.

## Mt. Jade is not a physical orientation substitute

Mt. Jade GSG Issue 1.00 Figure 9 (p.10) explicitly marks chassis front/top and
rear/bottom, with **Socket1 left and Socket0 right**; Figure 10 (p.12) repeats
that grouping in a photograph. Its within-socket numbered bank pattern resembles
Collins, but its processor placement is reversed. No Jade orientation or DIMM
silkscreen spelling is imported into the Collins model. OCP Mt. Jade Rev 1.0 is
already a separate platform authority in `extdeps.ocp.mt_jade.subject`; this
change does not assert that a shared processor or channel topology makes their
physical boards interchangeable.

## Another figure boundary

The right-side MCU/channel leader labels in the topology cartoon run 4,5,6,7
against connector pairs displayed 16/15,14/13,12/11,10/9 (and the corresponding
socket-1 keys). The existing GSG Tables 12–13 authority assigns those pairs
MCU7,6,5,4. This change uses the cartoon only for its slot#0/slot#1 legend, and
Figure 9 for connector positioning. It does **not** replace the established
pair table with the cartoon's MCU labels. Resolving that upstream discrepancy
is outside this physical-identity change.

## Documents and bytes read

All four publisher downloads returned PDF bytes without credentials. Full text
was extracted with PyMuPDF; the cited topology, chassis and board-replacement
figures were visually inspected. Figure 12's embedded 1220×1626 photograph was
also inspected at native resolution. No document PDFs or photographs are committed.

| Document | Revision/date | Location and inspection scope | SHA-256 |
|---|---|---|---|
| Mt. Collins DVT/PVT/MP Getting Started Guide, AMP 2021-00511 | Issue 1.05, 2026-07-20 | [Ampere publisher](https://connect-admin.amperecomputing.com/api/secure-file-download/download-regular/?file=Mt_Collins_DVT_PVT_MP_GSG_v1_05_20260720_52a0348e87.pdf&type=technical-document&documentId=z6zenr250qdpa8eq4irolgds); Figures 7–10 pp.9–13, Figure 11 p.32, Figures 12–13 pp.36–37; Tables 11–13 pp.11–12 | `1c41b4f088fa9c8b760d21d57ae93b0d1a68ee67a740d4eb7ac893753fc580b3` |
| Mt. Collins 1U User Guide and Hardware Maintenance Manual | Issue 1.00, 2022-03-30 | [Ampere publisher](https://connect-admin.amperecomputing.com/api/secure-file-download/download-regular/?file=new-private-files/94/tech/Mt._Collins_1U_Users_Guide_v1.00_20220330.pdf&type=technical-document&doc_id=669); §4.2.7 Figures 10–11/Table 3 pp.39–40; Figures 24–25 pp.52–53 | `7f351cc3bf8a9e2d1e6809a9e5337e527feac61a8ff3c4e4d6514f5e59311e7c` |
| Mt. Collins 2U User Guide and Hardware Maintenance Manual | Issue 1.00, 2022-03-30 | [Ampere publisher](https://connect-admin.amperecomputing.com/api/secure-file-download/download-regular/?file=new-private-files/94/tech/Mt._Collins_2U_Users_Guide_v1.00_20220330.pdf&type=technical-document&doc_id=649); §4.2.7 Figures 19–20/Table 6 pp.52–53; Figures 42–43 pp.74–75 | `a8102a2a661fca508c079ba460da2f701360216517ee090a036e533bbf078bbf` |
| Same 2U guide, operator-supplied **third-party mirror** | Same printed issue/date | [Doslab Electronics mirror](https://repo.doslabelectronics.com/misc/Mt-Collins-2U-Users-Guide-v1-00-20220330.pdf); fetched with curl, HTTP 200, 16,148,131 bytes; byte-identical to Ampere's copy. Initial Python HTTP request returned 403; browser tool timed out. The successful download is the cited read, not those failed attempts. | `a8102a2a661fca508c079ba460da2f701360216517ee090a036e533bbf078bbf` |
| Mt. Jade PVT/DVT (NVMe) Getting Started Guide | Issue 1.00, 2022-04-12 | [Ampere publisher](https://connect-admin.amperecomputing.com/api/secure-file-download/download-regular/?file=new-private-files/94/tech/Altra_Family_Mt._Jade_PVT_DVT_NVMe_GSG_v1.00_20220412.pdf&type=technical-document&doc_id=631); Figures 8–10 pp.8–12 | `f6528f70dec97b8838bd67413566287fc7c465addbf07182e9bdd1d48874c69f` |

The 2U document has its own `extdeps.ampere.mt_collins_2u_user_guide.subject`
authority, including its publisher URL, separately named mirror URL, issue,
date, digest and Figure 19 citation. The mirror fetch result is a gunbc receipt,
not an Ampere-owned fact. It is not called a publisher locator.

## Consumption and remaining boundary

`gunbc.mt_collins_dimm_physical_identity.sixteen_dimm_diagram_identity` is an
executable inspection entry point into `population_diagram_identity`. It carries
ordered banks, canonical channel identity, drawing legend, chassis orientation,
2U guide citation/mirror provenance, and explicit physical identification gaps.
Unknown/duplicate connector references, socket disagreements and invalid population
rows refuse without returning partial slot instructions. The published 32-DIMM
row still has 31 keys; the projection reports the mismatch rather than silently
adding J32. The existing correction remains the authority for that separate inference.

The later **Mt. Collins rack-facing DIMM population projection** is declared by
`operator_projection_frontier` beside the join, including the capability required
to close it. This change lands the documentary model and executable join, not a
rack UI or a claim that physical identification gaps have been closed. That future
projection must preserve the distinction between figure-based positioning and
verified silkscreen/colour. A clear chassis photograph is the next evidence route
when the operator cannot read the markings.

## Later operator report: unit 1 only

Dashboard message `msg_ecd92656-ea35-4746-9e3f-f67d12706ec5`, relayed by
`eager-owl-205` on 2026-09-11, reports the operator's by-eye inspection of
**unit 1, a 1U chassis**: physical blue connectors are the odd-J set used at
1DPC; the silkscreen supplied no useful slot-identification information.
No photograph accompanied the report to this session. We did not independently
inspect the chassis, and the report is not an Ampere statement.

`gunbc.machine_intake_mtcollins1_dimm_connector_observation.mtcollins1_blue_connector_observation`
records that report, its date, observer, relay message and unit scope. The
reported set references the canonical 16-DIMM population rather than repeating
its J-number list. The upstream documentary findings above remain unchanged.

`mtcollins1_colour_meaning_divergence` is the third fact: it joins the reported
physical blue set to the documentary roles and finds that blue on this chassis
has a different role from blue in the schematic legend. This is evidence that
the diagram colour does not predict this chassis's colour, not a contradiction
in a vendor physical-colour rule (none was found).

`colour_evidence_for_unit` returns this evidence only for the existing unit-1
intake subject. Another unit receives `NoColourObservationForUnit`, and the
returned documentary projection still carries both upstream identification
gaps. The later rack UI must preserve date and by-eye/report provenance; this
receipt does not generalize to other units, the 2U chassis, or the fleet, and it
does not explain or validate processor training.
