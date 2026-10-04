# Stackable cassette frame review

Operator direction, 4 October 2026: combine milk-crate stacking with independently removable
cassettes, favoring joinery and short assembly time. Worktree: `cassette-stacking`; branch:
`session/cassette-stacking`, based on the r06 cassette model in PR #13222.

The r07 proposal retains all 17 r06 cassette print geometries and adds a separate frame per
cassette: six joined side sections, six cross-tie halves, and one broad-grip lift-out front stop.
Two clip-on front skids adapt the existing cassette to the lowered runners. The rear arm sockets
extend below the original tray rails; a runner directly under those rails interfered with them.
The revised runner top is at -15 mm, supporting the rear socket bottoms and the front skids.
This preserves P1/P2 already printing. No additional threaded fasteners are proposed.

Each frame has four integral locating feet and pockets. Broad corner shoulders carry vertical
load at a 158 mm stack pitch; feet provide alignment, not lifting locks. Nominal footprint is
345.84 by 450 mm. Shared power is reserved alongside the stack, not on the cassette or on top of
printed frame members. Supply selection, enclosure, distribution, protection and bonding remain
unresolved. The rear remains open for exhaust and disconnect access.

Side sections have paired rectangular tenons; cross-ties use end sockets and a center half-lap.
These are geometric proposals: positive retention against separation, racking, creep, material,
print orientation/support and loaded extraction need physical evaluation. No stack load rating,
maximum populated stack height or assembly-time saving is asserted. Do not lift a populated stack.
The viewer removes the front stop for withdrawal and depicts cables disconnected. The user must
support a withdrawn populated cassette until its loaded stability is qualified.

Geometry and layout are owned by `stacking_frame.dag` and `stacking_review.dag`. The existing
CadQuery emitter realizes both cassette and frame; the existing review adapter packages them.
The viewer consumes emitted meshes, placements, frame IDs and withdrawal travel. No new browser
mechanical geometry is authored. The default view is two stacked units, lower cassette withdrawn
150 mm; individual rotation, all-pieces view and STL/STEP downloads remain available.

Validation includes the DAG bed/member checks and negative controls for a missing corner,
wrong stack pitch and an interference-fit locator. CAD checks cover valid connected printed
solids, bed extents, STL manifoldness/bounds/volume, STEP round-trip, printed-part intersections
within one unit and between two seated units. Withdrawal checks sample seven positions over
0–150 mm, with the stop removed. They are not a continuous swept-volume proof or cable check.
Kernel-observation policy remains in the existing Python adapter, an acknowledged review debt;
source witnesses are not a claim of structural qualification.

Reproduction: run `product.printed_chassis.stacking_review` function `bundle`, split its manifest
and program into JSON/Python files, then execute the emitted program with CadQuery 2.8.0.
Run claims in `dag/test/claim/stacking_frame_witness_test.dag`. Local artifacts are under
`/home/briansrls/print-prep-2026-10-04/stack-r07`.

Printing status: operator confirmed printer-01 printing P1; telemetry confirmed printer-02
running P2. Both start publications lost their acknowledgement connections; neither was resent.
The expired temporary GCP token was removed from this machine and srv1. This stacking review
starts no additional prints and changes no active jobs.

Executed validation: all 32 printed instances / 25 distinct authored geometries passed the CAD
checks above. Both seated levels had zero positive-volume printed intersections. All seven
withdrawal samples passed after lowering the front cross-tie below the skids. All 17 original
cassette STL files are byte-identical to r06, including P1 and P2. The six existing cassette
witnesses passed; the four stacking witnesses cover the new source assembly and negative controls.
