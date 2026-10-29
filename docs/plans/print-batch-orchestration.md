# Desired-set printing

**Goal:** request parts once, then clear beds when notified. **Method:** deterministic plate packing and durable slot reconciliation through the existing guarded start route. **Validation:** BATCH-1–6 below, ending in one authorized packed live plate and recovery without duplicate commands. Requested after E1/E2 completed on 2026-10-04; the [explicit-plate foundation](print-workflow-integration.md) exists, but this full scheduler/release contract remains incomplete and grants no start.

Cassette demand derives from [selected assembly requirements](cassette-assembly-requirements.md), never a second authored list; standalone goals stay explicit. `std.goal_assessment` turns unknown inventory/pending inspection into evidence work, not replacement. Accepted compatible inventory satisfies slots; rejected objects create explicit replacement demand. FINISH never proves physical acceptance.

## Packing contract

Input: versioned part multiset, compatible available printers, admitted process/material policy. Output: explainable plates, schedules, remaining quantities and durable execution/handling receipts. Reuse CAD/export identities/digests, `manufacturing_manifest`, fleet printer identities and #13149’s approval/start/claim/reconciliation route. `bed_partition` splits oversized parts; packing arranges those pieces under the same allowances.

- Expand stable instance slots bound to geometry, material/process and orientation; reserve active/claimed/uncertain/completed-uninspected work. Partition only genuine printer/nozzle/material/process incompatibility; combine compatible projects/parts.
- Deterministically pack conservative oriented envelopes using several orderings; report algorithm/quality and unused area, never global optimality. Allowed in-plane rotations do not admit changed structural print orientation. Include support/brim/gaps/margins/exclusions; use whole-plate layer printing unless toolhead sweeps are separately modeled.
- Optimize bed-clear cycles first, then sliced duration across printers and assembly priority. Compare support mass, total material, duration, trapped-support access and handling risk; never sacrifice loaded-joint orientation/infill to area utilization. Prefer compatible useful multi-object plates; no speculative duplicate fillers.
- Independently read complete sliced toolpath bounds/separation, machine/nozzle, temperatures/material mapping, scale/supports/time/material. Repack/split overflow; preserve instance attribution, not anonymous merged STL. Earlier envelope arrangements were unsliced opportunities, not admissions or reprint requests.
- Prepare both queues; reschedule only unclaimed work. Approved/claimed/running plate membership and digest are immutable. Damaged P1/P2 are explicit replacements; other completed/active reservations remain.

## Execution and handling

A plate atomically reserves all its slots across both printers in shared persistent state before approval/start. Persist membership, geometry/process/3MF digests, placement, printer, approval, claim and observations. Caller output directories and new plan/attempt IDs cannot reset demand-slot identity, accepted inventory or unresolved reservations. Demand, plan revision, attempt and physical item are distinct.

Lifecycle: planned→prepared→awaiting bed-clear/approval→claimed→running-confirmed→completed-awaiting-removal/inspection→accepted. Upload/start uncertainty retains claims/reservations and requires reconciliation. Pause/failure/partial failure/rejection have named outcomes; no timeout republishes. Partial acceptance records intact/rejected objects explicitly, replacing only rejects.

Use fresh bed-clear ntfy approval as clear-and-continue for the next immutable plate. Completion notification presents that plate. A1 mini removal stays physical; FINISH, IDLE, timers, old confirmation or batch approval never establish future bed clearance.

| Task | Required evidence |
| --- | --- |
| BATCH-1: demand/inventory | Restart/replan cannot duplicate accepted/active/claimed/uncertain slots; incompatible revisions cannot satisfy demand. |
| BATCH-2: packer/review | Every unreserved slot exactly once or refused; rotation, overlap, exclusions, process and support/brim controls. |
| BATCH-3: realization/readback | Instance IDs/unit scale survive; toolpath envelope excess refuses/re-packs; sliced time/mass drive schedule. |
| BATCH-4: controller | Concurrent reservation once; crashes before/after upload, claim, publish and completion; lost ACK reconciles exact filenames without replay. |
| BATCH-5: handling | One fresh clear-and-continue per plate; finished differs from usable/removed; partial rejection replaces only rejected slots. |
| BATCH-6: release | Shadow full remaining cassette plan versus manual baseline (plates/visits/duration), then authorized packed live plate and no-duplicate recovery. |

Land pure BATCH-1/2, reviewable BATCH-3, execution BATCH-4/5, then BATCH-6. Completion requires all six.
