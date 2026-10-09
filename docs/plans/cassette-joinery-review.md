# Cassette joinery revision

Operator direction, 4 October 2026: fewer screws, fewer small attachment points, and shorter
assembly time. Replace the r05 screw-and-splice construction before scheduling cassette prints.

Proposed r06 assembly: four keyed half-lap tray panels with integral rails and split front handle;
two slide-in rear support arms; three interlocking fan-frame sections (outer guides integrated);
three guards with integral spacers/locating pins; five large removable retaining pins. Target
17 printed instances, four M3 panel-clamp screws and six M4 fan/guard screws, excluding motherboard
standoffs. The r05 design had 49 printed instances and 56 fastener axes. These are design targets,
not measured assembly times or confirmed fit.

Assembly order: mate the four panels and fit their four clamp screws; slide the two support arms
into the integral underside sockets and drop in two retainers; slide the center rear section into
the two outer sections and retain it with one pin; seat that bank on the arms and insert two pins;
fit fans and guards with two diagonal through-bolts per fan. The other two fan holes receive locating
pins. The keyed joints carry registration/shear; the four tray screws clamp the laps. No small
underside splice plates, separate rail splices, separate fan feet or loose guard spacers.

Nominal joint clearance: 0.25 mm per mating side (proposal, not measured printer compensation).
Board supports, loaded joint behavior and final hardware engagement remain explicit open interfaces.
Review one complete assembly; do not restart individual gauge printing.

Implementation notes for the revision:
- Preserve nominal board/tray outline, rear exhaust, shared fixed-frame power and custom bay pitch.
- Use 8 mm panels with 4 mm keyed laps. Panel print envelopes must include tongue, rail and handle.
- Rear arms slide in integral panel sockets; fixed rear bank remains split for the 180 mm bed.
- Derive both sides of each joint from one joint declaration; witness alignment and deliberate
  clearance mutations in `.dag`, alongside actual CAD readback.
- Extend the existing CadQuery realization path; remove the separate product-level Python
  geometric interpreter. Kernel observations are evidence, with validation policy modeled in DAG.

Validation performed for r06: CadQuery 2.8.0 realized all 17 printed pieces as single valid solids;
STL closure and volume/bounds checks, STEP round-trip, 180 mm bed fit, and printed-pair collision
checks passed. The review uses the shared cadquery_realization exact-dimension emitter. The source
manifest is bound to the emitted solid program. `.dag` witnesses cover source-model bed/mate/member
checks and negative controls. Geometry checks do not establish support-free printing, retained-pin
vibration performance, loaded joints, thermal adequacy or board support.

Remaining review work: kernel-observation policy still lives in the review packaging adapter;
move it behind a DAG-owned observed-result judgment and execute that pipeline in CI. The new source
checks and shared emitter remove the duplicate solid interpreter, but do not close that obligation.
The assembly remains review-only, with no fabrication capability. Assembly time must be measured
on the first full unpowered build; no invented time saving is claimed.

CI cost validation (2026-10-04): the closed-variant residue gate reports zero unrostered
cases after explicit constructor handling. Source admission indexes world-space bore lines and
part IDs instead of rescanning all primitives for every mounting hole. The map remains bounded
by this small assembly; persistent-map insertion is not claimed to be globally linear.
The whole-assembly and moved/missing/oversized controls remain full-model witnesses. Packaging
uses the same parameterized producer on a small fixture and includes duplicate-member refusal.
Direct claim_batch measurements put all six witnesses below the new-witness enrollment margin;
no ceiling, debt roster or exemption was changed. A complete bundle emission matched the r06
manifest and Python program exactly. Hosted floor validation must still confirm the committed
revision. Local full-floor execution is not a green receipt: this session host lacks the declared
browser-fixture runner premise.
