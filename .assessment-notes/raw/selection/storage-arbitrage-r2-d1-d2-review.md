# R2 / D1 / D2: selection and evidence review

Read-only repository audit and public-source reading, 2026-10-23. Proposed direction for PR14799; no implementation, live observation, purchase or deployment performed. Names explicitly described below as proposed are modeling deliverables, not existing APIs.

## Proposed replacement direction

Fabric storage chooses among an admitted store on the executing host, an admitted served store on another host, and an admitted R2 realization. These are alternatives for the same logical contract when their evidence establishes that they can satisfy it. Local disk is not assigned cost zero, served disk is not automatically the fallback, and R2 is not permanently assigned a durability-only role. A selected realization may have replicas only when the contract and its realization model require them; replication is not itself a fourth implicit choice or a reason to count R2's included durability twice.

The ordinary caller supplies its logical work and irreducible data. The responsible root owns the deployment context, contract, workload assumptions, service requirements, valuation policy and evidence sources. The fabric layer derives candidate-specific access, placement and prerequisites from that context. Current code still passes concrete realization dependencies internally: this plan does not claim that omitted-operand inference is implemented.

For allocated mutable state, pricing a different candidate does not move the authority. The selection subject includes the continuity commitment; a move requires an admitted transition that preserves state and prices that transition. An unavailable authoritative store does not become an empty cheaper store. A fresh isolated immutable snapshot is the first useful three-candidate exercise because it has no existing mutable authority to relocate.

## Required interface before implementation

| Boundary | Existing authority to reuse | Concrete modeling obligation |
| --- | --- | --- |
| Logical operation and result | `std.fabric_storage::{FabricObject,FabricObjectGet,FabricPut,FabricHeadExpectation,FabricHeadAdvance,FabricClosureRead}` | Preserve object/readback, uncertain publication and conditional-head semantics; do not add a second generic storage API. |
| Candidate discovery and binding | `gunbc.fabric.fabric_storage_placement`, `gunbc.fabric.fabric_storage_binding`, `gunbc.fabric.fabric_storage_client::FabricStorageBinding` | Declare the three candidate kinds and actual admitted subject identities. A local file route must not bypass the shared store's current socket/signed-route admission. Candidate availability does not confer authorization. |
| Nonnumeric contract eligibility | Domain projection owned with fabric placement/binding; `std.decision::DecisionConstraint` and `ConstraintDisposition` remain the generic subject accounting | Model candidate-specific established eligibility, established exclusion, and unresolved eligibility with typed causes and evidence. Cover operation support, consistency, durability/failure scope, retention, reachability, namespace/authority continuity and principal/access. Do not encode these as arbitrary 0/1 cost axes. |
| Numeric comparison | `std.decision::select_realization`, `std.pareto::{SelectionAxis,AxisReading,ParetoEntry,AxisGap}` | Project only dimensionally compatible, applicable readings for the declared workload. `HardConstraint` holds one numeric `bound_micro`, axis and denomination; it does not model tenancy, authorization or consistency. |
| Workload and SLO meaning | Existing measure/currency/time types; new domain fields with fabric policy as owner | A proposed storage workload/evidence carrier must state object sizes/counts, access counts, holding interval, origin/destination, operation scope, concurrency and warm/cold condition. A required latency metric additionally names statistic, sample population/window, bound, evidence rule and consequence. There is no current generic SLO carrier to cite as if complete. |
| Cost standing and provenance | `std.realization_schedule::{CostAxis,CostTimeAxis}`, `extdeps.external_authority::{CitedFigureStanding,ExternalAuthority,FactCitation}` | Preserve predicted/measured/unestablished and cited/uncited/observed distinctions separately. Neither carrier supplies workload applicability, a measurement window or immutable snapshot identity by itself. |
| Choice scope and receipt | `std.decision::{DecisionAssessment,DecisionScope,SelectionReceipt,RealizationSelectionResult}` | Close terms, parameters, constraints and all required term-variable sensitivity/coupling cells; carry distinct evidence identity for each immutable reading set. |
| Prerequisites and post-effect evidence | `gunbc.host_convergence_protocol::HostEffectDomainProjection`, `std.goal_assessment`, `gunbc.ensure`, existing authorized provider ensures | Observe and assess the selected context, apply only under the responsible root's authority, and independently read back. Selection is not readiness; command success is not convergence. |

An unresolved candidate must not silently disappear from the candidate population. Established incompatibility can exclude it, with the reason retained; absent applicability/access/consistency evidence cannot make a potentially better candidate lose by omission. Report that as `SelectionNeedsEvidence` with a domain cause projected to `CandidateStandingIncomplete`, or as subject closure obligations where the missing fact is shared by the decision. If a candidate is already conclusively excluded by a mandatory contract, irrelevant missing price data need not prevent choosing among the remaining eligible candidates.

The domain projection must close its disposition roster against the root-owned candidate roster, including exclusions. `establish_candidate_field` sees only the `ParetoEntry` field it receives; it cannot discover a candidate the projector dropped. This is the same completeness obligation, not a second selector.

## Exact selection boundaries found in code

`select_realization` takes `DecisionAssessment`, candidate-field identity, `List<ParetoEntry>`, funded axes, numeric hard constraints, constraint authority, policy and evidence-snapshot reference. Passing three prices is not enough: `decision_closure` checks physical-term population, causal couplings, sensitivities, parameters and constraints. Missing listed evidence defers; an incomplete structural join refuses. `ObjectiveTerm.ground` and every sensitivity coupling have `CitedFigureStanding`. An uncited declared assumption does not magically become a settled grounded decision.

`AxisReading` is a named axis plus `OrderedClosedInterval<Int>` called `interval_micro`. It has no built-in currency, latency statistic, workload or source. Those facts must be checked by the domain producer before projection and retained in the evidence. A point interval asserts an exact value; coarse readings should retain their justified interval. Overlap can remain undecided. Two incomparable feasible survivors require an applicable policy, such as existing `ValuedObjective`; do not manufacture a winner by list order or a backend priority. A missing policy remains `SelectionNeedsPolicy`.

`scopes_eq` compares subject reference, candidate-field reference and standing, constraint-authority reference, funded axes and `evidence_snapshot` reference. `receipts_eq` additionally compares survivors, policy reference and winner. It never compares reading values. Updating one `current_prices` declaration while retaining its reference cannot establish receipt inequality. The bounded first delivery can use immutable corpus-declared evidence sets, each with a distinct named declaration; changed prices, applicability assumptions or measured observations get a new identity. A later runtime-changing evidence snapshot must be represented at the decision authority rather than hidden behind one static `DeclarationRef`. This limitation must be explicit if runtime sampling is promised.

`std.cache_interface::ReadLatencyClass` and `read_latency_cost` are not production timing observations. The latter maps classes to fixed 1 ns, 1 microsecond, 1 ms and 10 ms values. Do not reuse those as actual disk/LAN/R2 latency. `CostTimeAxis` preserves scale and standing but is not an SLO contract.

`CitedToAuthority` cites an upstream claim; it is not a local measurement. `ObservedByReceipt` carries sealed `ObservedReceipt`, whose mint currently admits four named folds. A new storage latency receipt needs the same modeled, evidence-reading path and its admitted producer, not a hand-written observed reference. The current conditional-put race receipt establishes the stated race property, not general storage latency.

## Current primary-source price evidence

All URLs below were read on **2026-10-23**. Prices are inputs to a workload derivation, not whole-workload totals or actual fleet invoices.

| Input | Grounded current value | Required qualification |
| --- | --- | --- |
| R2 Standard stored capacity | USD 0.015 per GB-month | Separate storage from operations. |
| R2 Standard Class A | USD 4.50 per million requests | Includes object puts and lists; price the actual operation graph. |
| R2 Standard Class B | USD 0.36 per million requests | Includes gets and heads, including independent readback. |
| Direct R2 egress | No R2 egress charge | Other attached metered services can still charge. |
| R2 allowance/rounding | Monthly Standard allowance: 10 GB-month, 1 million A, 10 million B; billing rounds upward to billing units | Allowance and rounding are billing-context facts, not a fresh discount for every object. Storage uses average daily peaks over 30 days. |

Source: [Cloudflare R2 pricing](https://developers.cloudflare.com/r2/pricing/), page marked updated 2026-10-01. Infrequent Access has different prices and a minimum holding duration; do not silently substitute it for Standard.

R2 defines GB as 10^9 bytes and distinguishes it from GiB. This is a primary-source basis for revisiting the existing `r2_standard` row's `ByteBasisUnstated`; cite this definition alongside pricing rather than silently selecting a basis. [Cloudflare R2 limits](https://developers.cloudflare.com/r2/platform/limits/).

| Disk market input | Published monthly add-on price, excluding VAT | Applicability |
| --- | --- | --- |
| 22 TB SATA Enterprise drive | USD 43 / EUR 36 | Whole device, not per used logical TB. |
| 1 TB NVMe SSD | USD 36 / EUR 30 | Whole device, not evidence of a fleet machine's current cost. |

Source: [Hetzner dedicated-server add-ons](https://docs.hetzner.com/robot/dedicated-server/dedicated-server-hardware/price-server-addons/), page marked changed 2026-08-24. The vendor describes this as a planning guide; component availability depends on server and location. These quotes can ground an explicitly applicable rental or declared market-priced scenario. They cannot be relabeled as srv1's acquisition invoice or an automatically orderable configuration.

For actual this-host/other-host choices, bind the installed device and its real acquisition/rental evidence, or an explicitly admitted applicable replacement/rental quote. Account for useful capacity after required replication/reserve, utilization allocation, declared amortization interval and residual value, energy, host/service allocation and network charges. Do not double-charge a whole host and all its already-included disks. `std.measure::{MoneyOnce,MoneyPerMonth,AmortizationMonths}` and `extdeps.pricing.hetzner_dedicated::hetzner_listed_effective_monthly` are precedents for the units and declared holding interval, not a ready-made disk-cost model.

An existing idle disk may have zero *incremental rental cash payment* under a declared decision horizon. That is one cost term, not proof of zero total cost or zero latency. Whether the objective is incremental cash, allocated lifecycle cost, or both is an owning policy decision; changing it changes the decision subject. A missing acquisition price, utilization, useful-capacity allocation or relevant energy/network term remains evidence needed, not zero.

## Latency evidence and the missing measurement

[Cloudflare's R2 architecture documentation](https://developers.cloudflare.com/r2/how-r2-works/) identifies bucket geography, request origin and access pattern as performance inputs; cache-enabled reads can have different freshness behavior. It does not supply a fleet-origin put-plus-independent-get percentile for this workload.

[Cloudflare's Local Uploads announcement](https://developers.cloudflare.com/changelog/post/2026-02-03-r2-local-uploads/) reports a vendor cross-region upload test, with up to 75% lower TTLB and a graph described as roughly 2 seconds to 500 ms p50. This is a real vendor performance claim for that feature and test, not this pilot's readback latency, not its SLO, and not a numeric axis transferable without applicability evidence. No applicable numeric end-to-end latency guarantee was found for the three actual routes in this audit.

The bounded implementation evidence task is one retained workload-specific measurement for each admitted route: same object population, declared retention and operation semantics, same relevant origin/principal, completion through independent content readback, declared concurrency and cache condition, timings with sample count/window and retained failures/timeouts. A small measured corpus is enough to begin; no continuous production metering platform is required. Until a funded latency axis or mandatory latency bound has applicable evidence, return `SelectionNeedsEvidence`. Do not quietly remove the axis to make a choice.

## Workload normalization and coupling

Use one explicit decision horizon and one accepted unit, e.g. a complete captured-and-materialized snapshot of a stated input population over its required holding interval. Do not compare a raw disk GB-month with one R2 request. The current snapshot representation has body/path/entry objects and a manifest, so request counts are derived from the actual operations, including readback, retries and occupied-key handling; encoding can enlarge stored/transferred bytes. One logical file is not one PUT.

Compute provider cost from the account baseline with and without the workload when claiming incremental invoice cost. At minimum account for capacity-time, A/B operation counts, retention, free allowances, account-level rounding, relevant transfer/compute cost and any startup/transition allocation. Linear per-request quotients are rate-derived estimates, not exact invoice increments. For example the Class B quotient is 0.36 micro-USD per request, below integer micro-USD precision; retain a batch/rational computation and explicit rounding rather than truncating each get to zero.

The physical/coupling model must include every term that can change the ranking: host selection affects network and authorization route; object size/count and read/write mix affect billed capacity/requests and completion time; concurrency/load affects latency; replication/durability affects usable disk capacity and writes; setup and migration depend on prior state; shared account use affects allowance/rounding. Do not add speculative terms absent the decision, but do not declare invariant a term merely because it was not measured. Owner time valuation is needed if a `ValuedObjective` converts latency into money; it is policy, not a vendor fact.

## Reconciliation and document relationship

`gunbc.host_convergence_protocol::canonical_protocol_bindings` is the existing canonical stack. `HostEffectDomainProjection` supplies observe, assess and decide; `inspect_via_host_effect_projection` currently consumes observe/assess. Its `host_effect_decide_apply_frontier` explicitly awaits the fleet apply-side projection consumer. The storage delivery should bind its prerequisite domain to these existing boundaries and invoke existing authorized ensures, with real independent readback; it must neither claim the generic effect loop is complete nor block on finishing the whole fleet/compiler program. `std.goal_assessment` deliberately does not carry planner/admission/actuator/readback orchestration ahead of the fleet spine.

`docs/plans/fabric-storage.md` remains useful for the store's existing object/head semantics, access routes, metadata/blob-plane distinction and cutover obligations. Its September design fixes one srv1 head location and assigns R2 a later durability tier. The revised PR must explicitly supersede those fixed-role *target-policy* passages with contract-constrained selection among all three realizations. Preserve the incumbent deployment description and continuity obligations until implementation changes them. Its statement that ETag-conditioned R2 heads are missing is historical now that `fabric_storage_r2_head::{r2_head_read,r2_head_advance}` exist; it must not be the new program's capability premise. Do not create two competing storage programs by merely linking the new plan without stating which policy it replaces.

## Staffable acceptance evidence

1. A named three-candidate subject closes every candidate disposition, required contract, workload parameter and term/coupling cell. A failed mandatory access/consistency requirement excludes even the cheapest reading; absent eligibility produces an evidence need.
2. Real sourced price rows plus declared applicable workload/account/amortization inputs produce dimensional cost readings. A real retained route observation produces latency readings, or the choice remains honestly unresolved. A changed price/workload/observation set has a different receipt scope.
3. Controlled changes to otherwise feasible costs or latency valuations change the selected backend through `std.decision`, with the logical caller unchanged. The control may supply boundary values, but at least one inhabitance execution per admitted route retains real selection, prerequisites, execution and independent readback. No fixed-role dispatch qualifies as arbitrage.
4. Unknown publication, denied identity, failed prerequisite observation or missing post-effect readback cannot mint complete success. No live mutable store moves as a side effect of a cheaper quote.

## Review decisions versus missing facts

Set from source now: one selector; typed domain eligibility before numeric ranking; distinct immutable reading-set identity; actual route semantics; units/currency; no fixed backend priority; no fabricated latency; preserve allocation continuity; use the canonical host convergence composition without claiming its awaiting apply frontier is complete.

The responsible owner must declare the policy facts: cost horizon/basis and allocation, actual workload/retention, any latency bound and permitted evidence, valuation when cost and latency trade off. The model should expose these in root-owned declarations, not repeatedly question feature authors. Actual machine prices and route timings are acquisition tasks whose absent values remain `SelectionNeedsEvidence`; another fake numeric default cannot discharge them.

## Proposed ownership homes and four-seat review

Keep generic storage semantics in `std.fabric_storage`; candidate policy and subject context belong to `gunbc.fabric.fabric_storage_placement` and binding construction to `fabric_storage_binding`. A small proposed `gunbc.fabric.fabric_storage_selection` module can own the typed domain eligibility/workload/SLO-evidence projection and call `std.decision`; it must not become another ranking implementation. This is an explicit new model deliverable, not an already-existing profile constructor. Keep fleet price/timing observations in gunbc's observing layer. Cloudflare facts belong to its existing `extdeps.cloudflare.r2` upstream scope (or a narrowly scoped pricing submodule); Hetzner facts to its existing vendor pricing scope. `extdeps.pricing.object_storage` currently contains legacy multi-vendor facts: reuse its agnostic rate shapes and existing cited rows where appropriate, but do not use that legacy shape as permission to grow a fresh multi-vendor authority. Any new runtime evidence-identity capacity belongs to `std.decision`, not a wrapper local to storage.

The proposed four responsibility seats are workable if their handbacks are explicit. Fabric integration owns selection A/F and census V, including the complete decision subject, typed projection, immutable evidence identity, workload/SLO declaration and final reconciliation of every candidate disposition. Backend B1/B2 owns actual file/served/R2 capability, identity/auth, prerequisite reads and readback; it also owes the comparable real timing receipts and the applicability of each provider quote. Dispatch C1/C2 owns the complete six-consumer lifecycle cut, not only claim recording. State migration S/M owns scope, retained-state reconciliation, fencing/quiescence and the single authority transition across both isolated deployment contexts. The integrator accepts the evidence across the seams; price acquisition and route latency observations cannot fall between A and B. A missing numeric latency source means the measurement package is a dependency of a successful arbitrage acceptance, not an optional follow-up after announcing a winner. No fifth standing team is required, but the root must resolve cost-basis/SLO policy forks and review the completed combined design before landing.
