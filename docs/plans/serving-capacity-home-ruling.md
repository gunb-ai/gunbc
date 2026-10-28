# Serving capacity belongs in the fabric domain

**Ruling:** compute cells and serving seats share priced selection, not occupancy. `gunbc.design_argument`'s `conformance-compute` row owns the reviewer-facing homes; this document preserves the argument, not another architecture.

| Boundary | Compute cell | Serving seat |
|---|---|---|
| Subject | Exact work on a stateless-between-grants cell | Turn on a loaded replica retaining weights/cache |
| Selection | Shared `product.fabric.selection.select_supply` | Same fold, including tie/refusal and cost axes |
| Occupancy | `CellReservation` / `std.durable_exclusive_hold`, file-backed CAS | `product.capacity` pool/events, `fabric_seat_acquire` |
| Lease | `product.fabric.execution.LeaseIdentity`: no deadline; supplied stale observation | `product.capacity.lease.LeaseGrant`: times, duration and release law |
| Admission/billing | Held/free; money reservation and settlement | Quality-conditioned co-tenancy; pricing without money settlement |

**Consequence:** retain both subjects and their actual homes. Serving belongs inside fabric/compute conformance; moving it wholesale into the settling control plane would impose money obligations without a payer. Upstream API quota consuming the serving lease algebra does not prove compute-cell occupancy uses it. Host addressing belongs to `fleet_intent_network`, not a third capacity authority.

**Validation:** read both producers and their live consumer joins before claiming unification; one consumer's imports prove only its path. No occupancy merger or replacement migration is authorized here. `gunbc.recurring_failure_mode.meaning_fork` owns the FabricGroup subject conflation and the rejected same-word/different-module near-miss; `gunbc.rung_drop.serving_cost_axes_asserted_unmeasured` owns the unmeasured cost-values limitation. Neither an address nor a module adjective establishes a capacity fork.

[Prior argument, rulings and dated receipts](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/serving-capacity-home-ruling.md).
