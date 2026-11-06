# Lotes AZIFA072 socket evidence

**Purpose:** preserve normalized PCB pad positions for the socket body that mates with the AZIF0222 ILM. `extdeps.cpu_attachment.lotes_azifa072` owns the part, drawing revision, geometry, provenance and release gaps; `azifa072_held_copies` locates the operator-held drawing and STEP.

**Files:** [lga4926_pcb_pads.csv](lga4926_pcb_pads.csv) contains drawing-derived centres in millimetres, origin at the array centre, +X right/+Y up in the STEP frame. [lga4926_pad_array.svg](lga4926_pad_array.svg) is an unreconciled visualization: its coordinates differ from the CSV, so it must not be treated as an exact rendering or an independent geometry check.

**Carriage:** `product.altra_motherboard.attachment_stack.RepositoryCarriageStanding` admits normalized facts, not vendor PDF/STEP expression. The confidentiality clearance does not override that distinction. Request GAP-AZIFA072 revision 2 and AZIFA072_B_20220208 from the holder or Lotes.

**Extraction evidence:** decompressed PDF object streams and font CMaps supplied text; CTM-tracked path replay supplied sheet-3 nine-vertex pad subpaths. Within-row pitch fitting fixed scale, checked against the sheet-4 detail. The separate STEP extraction agreed within 0.25 mm for 642 of its 650 modeled balls (median 0.019 mm, p95 0.051 mm, maximum 0.49 mm); corner fits account for the recorded outliers. This historical cross-check was not rerun during cleanup. Row clustering near the bank boundary affects row labels, not stored positions.

**Release:** consume `azifa072_open_gaps` and `azifa072_footprint_may_emit`; stencil aperture, A1 and mask interpretation remain unresolved. STEP's perimeter balls cannot reconstruct the interior voids. The asymmetric finger-access protrusion must survive keepout derivation. Reconcile the SVG against the held drawing and CSV before regenerating or retiring it.
