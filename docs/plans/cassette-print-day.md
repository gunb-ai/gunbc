# Cassette print sequence, 4–5 October 2026

**Goal:** assemble one unpowered fit prototype from the r06 tray and later 140 mm rear assembly. This is a dated execution record, not today's queue or start authorization. Existing approval, fresh bed-clear/readiness, durable claims and no-replay rules remain in [print convergence](print-workflow-integration.md).

**Sequence:** tray panels P1/P2 then P3/P4; the real seam is the first fit checkpoint. Advance independent admitted work without synchronizing printers. The three-80-mm-fan queue was superseded by E1/E2, B140L/B140R, M140/G140 and stack joinery. Preserve reported completed/running instances through replanning; a new packing plan does not create replacement demand.

| Dated event | Evidence / remaining limit |
|---|---|
| Tray waves | Preparation under `print-prep-2026-10-04/cassette-wave-{1,2}`; execution under `/home/briansrls/print-run-2026-10-04/` on srv1. Exact-filename telemetry reconciled uncertain publications without replay. Reported completion did not establish fit. |
| Arms/crossmember | `cassette-wave-3` in those roots. Both starts were independently observed; E1/E2 later reported finished, awaiting inspection. Old launch scripts are receipts, not rerunnable jobs. |
| Socket cleanup damage | Integral rear sockets tore off: exactly P1/P2 became replacement demands. Separate S1/S2 retain the arms; through-bolts replace K1/K2. Do not duplicate unaffected parts or print those retired pins. |
| Replacement overnight plate | `artifacts/printing/socket-redesign-2026-10-04/` owns compared slices and start receipt for P1/P2, S1/S2 and K3/K4/K6/K7. One uncertain publish was reconciled as RUNNING; physical acceptance remained open. |
| Rear/upper-post plate | `artifacts/printing/rear-posts-2026-10-05/` owns preparation/start receipt for M140/G140, four upper posts and four pins. No replay; no completion/acceptance claim. |
| Lower-post trial | Orca emitted two plates; the one-plate check refused. No dispatch. Packing/process work remained. |

**Preserved preparation lessons:** apply the modeled orientation once at unit scale; include support/brim envelopes and independently read deposited paths. Removing brim alone did not make P4 fit; support-base expansion mattered. Leading-dot extrusion needed reader support; malformed first-layer timing was not a valid estimate. Slicer estimates exclude handling and do not promise a one-day completion. Upright panel experiments and support-free sockets did not qualify structural/thermal behavior; the later print-plane policy governs new requests.

**Assembly limits:** standoff support, underside clearance, actual fastener engagement, retention, loaded joints, power hardware and thermal suitability remained unresolved. M3/M4 length suggestions were geometry estimates, not verified hardware selections. The rear shoe's bolt-clearance reservation was checked geometrically, not physically. Manually prepared plates did not yet supply durable arbitrary-set orchestration or preparation enforcement at start; [batch planning](print-batch-orchestration.md) owns that migration. No print or procurement is authorized by retaining this record.

[Prior design and dated receipts](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/cassette-print-day.md).
