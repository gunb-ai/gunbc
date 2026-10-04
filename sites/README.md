# Chassis workbench

[Altra Chassis Workbench](https://altra-chassis-workbench.briansrls448156.chatgpt.site) is the private
assembly review site. Reuse project `appgprj_6ac187f2fe208191b1ec7ad6703d93e8` and the separate
Sites-managed checkout at `sites/chassis-workbench/`; do not create a second site.

Review 05 consumes meshes and metadata generated from `product.printed_chassis.cassette_review`.
Mechanical geometry, placements, print orientation and interfaces belong to `.dag`. The browser
only displays those meshes; the former `concept.js` and mechanical sliders are removed. View
controls change the camera, visibility, explosion and cassette withdrawal, never printable geometry.

- Carrier: all 49 printed instances, purchased hardware and labeled clearance reservations.
- All pieces: 29 unique authored print geometries with required quantities; isolate and rotate any.
- Rack: four cassettes in a fixed metal frame, fixed runners and one shared-power reservation.
- Downloads: individual STL/STEP, assembly STEP, logical manifest and a complete review ZIP with
  parts, print quantities and interface hardware lists.
- Assembly review: the passed fan fit, proposed joints and remaining hardware decisions.

The tray is 271.840 × 294.700 mm, split for the 180 mm A1 mini. The custom frame uses 140 mm bay
pitch; it is not a standard rack-unit compliance claim. Dynatron W1 and P8 envelopes come from
vendor authority. One fan plate has already physically fit; reuse it when counting the next prints.
Shared 12 V power remains fixed per four-node block with four isolatable/protected feeds. Capacity,
branch circuitry, connector choice and redundancy remain unresolved. Board supports are explicitly
unresolved rather than invented geometry. See `docs/plans/printed-chassis-program.md` for evidence,
reproduction, first-assembly scope and the existing fabrication/printing workflow.

CadQuery validation covers all printed solids, STL manifoldness/bounds/volume, STEP round-trip,
180 mm bed fit, 56 matched fastener axes and zero positive-volume intersections between printed
parts. It does not qualify strength, engagement, thermals or electrical operation. Local browser
checks cover generated mesh loading, roster, selection/isolation/keyboard rotation, rack extraction,
all-pieces view, filters, hardware/clearance visibility, downloads, interface table and mobile layout.
There are no printer controls, credentials, external application access or persistent user data.

Earlier visual-only source: concept 04 `c2a49273da70f20a23e2e044fe552dceb74b773f`.

Published review 05 Site source: `a9658fd05d27e83220c611b5fb00ac4397b74739` (owner-private audience unchanged).
