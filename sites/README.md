# Chassis workbench

[Altra Chassis Workbench](https://altra-chassis-workbench.briansrls448156.chatgpt.site) is the private
assembly review site. Reuse project `appgprj_6ac187f2fe208191b1ec7ad6703d93e8` and the separate
Sites-managed checkout at `sites/chassis-workbench/`; do not create a second site.

Review 06 consumes meshes and metadata generated from `product.printed_chassis.cassette_review`.
Mechanical geometry, placements, print orientation and interfaces belong to `.dag`. The browser
only displays those meshes; the former `concept.js` and mechanical sliders are removed. View
controls change the camera, visibility, explosion and cassette withdrawal, never printable geometry.

- Carrier: all 17 printed instances, purchased hardware and labeled clearance reservations.
- All pieces: 12 unique authored print geometries with required quantities; isolate and rotate any.
- Rack: four cassettes in a fixed metal frame, fixed runners and one shared-power reservation.
- Downloads: individual STL/STEP, assembly STEP, logical manifest and a complete review ZIP with
  parts, print quantities and interface hardware lists.
- Assembly review: five assembly steps, four tray clamps, six fan/guard bolts, five broad-grip pins,
  the original fan fit and remaining hardware decisions. Motherboard standoffs are separate.

The tray is 271.840 × 294.700 mm, split for the 180 mm A1 mini. The custom frame uses 140 mm bay
pitch; it is not a standard rack-unit compliance claim. Dynatron W1 and P8 envelopes come from
vendor authority. The original fan plate physically fit; r06 retains its hole pattern in a new integral frame/foot.
Shared 12 V power remains fixed per four-node block with four isolatable/protected feeds. Capacity,
branch circuitry, connector choice and redundancy remain unresolved. Board supports are explicitly
unresolved rather than invented geometry. See `docs/plans/printed-chassis-program.md` for evidence,
reproduction, first-assembly scope and the existing fabrication/printing workflow.

CadQuery validation covers all printed solids, STL manifoldness/bounds/volume, STEP round-trip,
180 mm bed fit and zero positive-volume intersections between printed
parts. It does not qualify strength, engagement, thermals or electrical operation. Local browser
checks cover generated mesh loading, roster, selection/isolation/keyboard rotation, rack extraction,
all-pieces view, filters, hardware/clearance visibility, downloads, interface table and mobile layout.
There are no printer controls, credentials, external application access or persistent user data.

Earlier visual-only source: concept 04 `c2a49273da70f20a23e2e044fe552dceb74b773f`.

Historical review 05 Site source: `a9658fd05d27e83220c611b5fb00ac4397b74739` (owner-private audience unchanged).

Published review 06 Site source: `8dcdc73e2ed18f6498f63204796018727e3f923f` (owner-private).
Joinery/CAD draft: https://github.com/gunb-ai/gunbc/pull/13222 .
