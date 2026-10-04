# Provisional Jade and Collins mechanical references

Accepted for planning on 2026-10-04, following the operator-supplied research summary. Double-check board revision, outline axes, every mounting feature and underside keep-outs before committing standoff positions. These references unblock layout; they do not establish a complete mounting map.

## Sources and standing

- Jade: [OCP specification rev 1.0](https://www.opencompute.org/documents/open-compute-specification-mt-jade-rev-1-0-pdf-1), 2021-09-17, §7.2 p13: 428 mm wide × 479.36 mm deep. Retrieved from the [archived publisher document](https://web.archive.org/web/20220603132323id_/https://www.opencompute.org/documents/open-compute-specification-mt-jade-rev-1-0-pdf-1). PDF SHA256 `1ca9adc18543ef4686537aaafa386fa09075f75612dd3ea57c03b7d8d438f6ae`.
- Jade's p9 Figure 1 provides a cleaner top-down rendering than the annotated p13 Figure 3. The latter visibly has a different aspect ratio; neither is a dimensioned mechanical drawing. Candidate fasteners below were manually identified in Figure 1, not copied from a standard form factor. CPU socket fasteners are excluded. Feature identity and completeness remain unverified.
- Collins: [Portwell APTSA-627211-B4 datasheet](https://portwell.com/pdf/server/APTSA-627211-B4.pdf), as supplied in the operator's research: 512 × 424.2 mm. Direct retrieval failed in this session; this is **user-transcribed secondary evidence**, not a claimed independent read. Planning assigns 424.2 mm across the chassis and 512 mm front-to-rear, subject to confirmation. The reported unrelated copy/paste error reduces confidence in the sheet.
- The held Ampere Collins DVT/PVT/MP Getting Started Guide v1.05 p36 Figure 12 is cropped and obscured by cables/assemblies; it cannot establish full-board datums. Collins candidate coordinates therefore remain empty, rather than borrowing Jade's pattern. [Ampere document catalogue](https://amperecomputing.com/customer-connect/products/mt-collins) is the follow-up reference.

The reported dimensions make Collins **32.64 mm deeper and 3.8 mm narrower** than Jade under that axis assignment, not 84 mm longer. Public Collins OCP collateral was not located; this does not prove that none exists. Neither OCP NIC support nor an exhibition proves a published motherboard mechanical specification.

## Jade provisional candidate coordinates

Datum: left/front corner of the board's bounding rectangle; front is the fan edge at the top of Figure 1. Positive X goes right; positive depth goes toward rear I/O. The PCB is nonrectangular: the rectangle below is a scaling datum, not the actual routed outline.

Raster: full PDF page rendered at 2 pixels/point (1224 × 1584), PCB bounds x=394..830, y=594..1083, excluding connector projections. Each axis is scaled independently to the published outline. Coordinates are reproducible planning estimates; submillimetre display precision is not accuracy. No ±1–2 mm accuracy bound has been established, and hole diameters/threads are unknown.

| Candidate | From left, mm | From front, mm |
| --- | ---: | ---: |
| J01 | 100.1 | 9.8 |
| J02 | 302.3 | 9.8 |
| J03 | 10.8 | 230.4 |
| J04 | 419.2 | 232.3 |
| J05 | 94.2 | 259.8 |
| J06 | 264.1 | 259.8 |
| J07 | 348.5 | 269.6 |
| J08 | 143.3 | 282.3 |
| J09 | 103.1 | 303.9 |
| J10 | 82.5 | 447.0 |
| J11 | 102.1 | 444.1 |
| J12 | 167.9 | 458.8 |
| J13 | 338.7 | 444.1 |

## Model integration and later verification

`BoardOutline` now has an explicit `ProvisionalBoardOutline` arm. Collins uses it through the same layout/partition functions as the other boards: with 14 mm perimeter margin its tray planning envelope is 452.2 × 540 mm, allocated as 4 × 4 cells with the existing 180 mm bed / 30 mm feature budget. This does not generate drilling coordinates.

`board_mechanical_reference` owns the image-derived Jade candidates and source/datum/standing fields. `platform_projects_review` emits them alongside each platform, preserving the unverified status and empty Collins map. The separate mounting-coordinate and board-revision obligations remain open.

Later: reconcile every visible candidate against original mechanical CAD or a clear, calibrated board drawing; distinguish board fasteners from nearby hardware; account for obscured holes, nonrectangular outline, socket load paths and underside components. For Collins obtain a full top-down reference before deriving any coordinates. No operator measurement or disassembly is requested now.
