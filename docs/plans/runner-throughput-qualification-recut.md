# Runner-throughput qualification — the recut

Subject: gunbc#11722 (HW1-QUAL, dashboard node `adhoc-e7a8de23-9fa`). Ruling: the operator's
SOURCE HOLD on `03175fd0` (2026-09-19 19:13Z, six items against DESIGN §3d), re-posted as a
side-chat ruling at 19:46Z; parent eager-owl-205's confirmation of 2026-09-20 02:3xZ that the
recut lands on #11722 and that two things stay constant: **the assessment CONSUMES a selected
`ExecutorSanction`/offer and a selected conserving host-plan receipt and never mints either**, and
**runner register / dispatch / deregister are modeled control-plane mutations bound like every
other controller write**. This document is the recut plan the meta-review 68822 asks for before
the next implementation round; the parent reads it before anything is built.

The hold is a recut and not a fix list because it condemns the DIRECTION of the module's central
edge: today `RunnerThroughputQualification → qualify_runner_throughput → ExecutorSanction /
SupplierOffer`, i.e. the assessment mints the permission and the advertised supply it exists to
assess. §3d: selection precedes convergence — the selected class, offer and requirements exist
BEFORE the assessment, and an independent observation says whether that selected candidate
delivers. Nothing imports either module, so the cost of the recut is zero consumers.

## 1. The hold, item by item, and what discharges each

| # | Hold item | Discharged by |
|---|---|---|
| 1 (P0) | The assessment mints the sanction/offer it should assess; `capacity_class_candidate` is caller-authored and never assessed. | `SelectedRunnerQualification` carries a selected `ExecutorSanction`, a selected `SupplierOffer` and `ExecutionRequirements` as INPUTS. This lane produces a delivery claim bound to that offer; `product.fabric.capacity_admission admit_delivered_offer` joins policy and qualification. `qualified_executor_sanction`, `capacity_class_candidate`, `qualified_supplier_offer` are DELETED. RED: an admitted throughput assessment paired with a `CustomerExecutableCapacity` sanction that was not independently selected cannot mint one — there is no function that takes an assessment and returns a sanction. |
| 2 (P0) | The receipt is caller-paired: `receipt_for_run` copies host/workload/envelope/backing from the subject; the effective cgroup bound is discarded; every admitted arm is directly authorable. | `ObservedRunnerQualificationRun` is a `sole_constructor` record minted only from a producer-owned run receipt that CARRIES host, program+revision, calibration specimen identity, image/runtime digests, slot unit name and incarnation, the EFFECTIVE systemd properties read back, the cgroup level with its `HostBudgetResolution`, and the workspace observation. The join proves the effective bound equals the selected slot bound. REDs: foreign host; same step count, different program; wrong cgroup / effective high; wrong workspace backing; a directly authored admitted disposition (unwritable — the admitted arm has no authorable constructor). |
| 3 (P0) | One internally chosen run (`slowest_run`) answers every axis. | A `SamplingPolicy` is an INPUT of the selection (`ExactlyOneReconciledRun` \| `ConservativePerAxis { minimum_runs }`); aggregation is per axis — minimum throughput and width, maximum peak and phase wall, unread dominating. `slowest_run`, `standing_speaks_before`, `measured_speaks_before`, `first_unreadable_run` are DELETED. RED: run A slowest and passes memory, run B faster but reaches `memory.high` → not admitted. Missing or duplicated required-phase readings refuse. |
| 4 (P0) | Slot width re-authors allocation policy inside the assessment (session 0, compile 0, floor-as-headroom, raw nproc; control-plane charge omitted). | The selection carries a **selected conserving host-plan receipt** (`FleetHostPlanDerived` plus its `HostAllocationVerdict` = `HostAllocationConserves`, with the fabric control-plane claim charged) as an INPUT; the assessment reads `runner_slice_cap` and the effective runner CPU cap off it and divides by the selected slot. `HostBudgetFacts` and the inline `fleet_host_plan_for` call are DELETED. RED: an unchargeable or oversubscribed plan derives no quantity. |
| 5 (P1) | `phase_wall` and `floor_eval_steps` are carried but never adjudicated; the phase roster can be incomplete. | The conclusion is NARROWED to what is assessed. The delivery claim asserts exactly two facts: interpreter rate ≥ the pin and one slot-memory envelope held. Per-phase walls are adjudicated only against `PhaseCeilings` supplied in the selection, and the observed roster must equal `required_ci_phases` (missing or duplicated → refuse). `floor_eval_steps` is carried as observation only and the claim's `assessed_facts` names what it does not cover. |
| 6 (P1) | The route says one stage writes a controller; registering / dispatching / deregistering a GitHub runner are control-plane mutations classified `NoControllerWrite`. | `StageEffect = BmcWrite { gate } \| GitHubControlPlaneWrite { authority: RouteLegStanding } \| GuestLocal \| ObservationOnly`. Every effectful leg is bound or `LegAwaitingAuthority`; `route_controller_writes_are_gated` is replaced by `route_effectful_legs_are_bound_or_declared`, which is red while any effectful leg is unbound — the route is not called executable until it is green. |

## 2. The four types (target shape)

Three live in `gunbc.runner_throughput_qualification` (the route stays in
`gunbc.runner_throughput_qualification_route`); the fourth is an arm on the fabric's existing
delivery-claim carrier. Names are the operator's. **One module was added after the cut for a
measured reason:** `gunbc.runner_throughput_selection` consumes the fleet authorities
(`runner_slot_allocation`, `fleet_host_budget`, `fabric_executor_class`) and projects them into the
selection carriers, because `product.fabric.node_qualification` imports the leaf and, on the
current seed, an import edge from that closure to `gunbc.build_cache_instance` (reached by all
three) breaks bare-name resolution in `gunbc.harness.harness_backend` with the module set
unchanged — bisected 2026-09-20 on #11722, reported to the parent lane as a seed defect.

**`SelectedRunnerQualification<P>`** — the selection, made BEFORE the assessment, by the fleet:

```
type SelectedRunnerQualification<P> {
  offer: SupplierOffer<P>                      // selected supply row: quantity_bound, shape, evidence
  executor_sanction: ExecutorSanction          // selected fleet-side class permission (gunbc.fabric_executor_class)
  requirements: ExecutionRequirements          // the work this offer is claimed to deliver (the floor's)
  workload: FloorWorkloadPin                   // program + revision + calibration specimen identity
  host_plan: SelectedHostPlan                  // FleetHostPlanDerived + HostAllocationConserves, control plane charged
  slot: SelectedSlot                           // RunnerSlotDesired + unit name + expected effective properties
  sampling: SamplingPolicy                     // ExactlyOneReconciledRun | ConservativePerAxis { minimum_runs }
  phase_ceilings: PhaseCeilings                // per RequiredCiPhase wall ceiling, or NoPhaseCeilings (then walls are observation only)
}
```

**`ObservedRunnerQualificationRun`** — `sole_constructor`, minted by the producer only. Carries
every identity the hold lists: host, program, revision, calibration specimen identity and its
`[witness]` line, image / runtime / toolchain digests, slot unit and incarnation, effective
`MemoryMax` / `MemoryHigh` / `TasksMax` / `CPUQuota` read back from the unit, cgroup level with
`HostBudgetResolution`, `memory.peak`, workspace observation (`WorkspaceBytesLocus` as READ, not
as declared), `[floor-phase]` rows, and the run coordinate (`WorkflowRunId`, attempt label,
`observed_at`). Nothing in it is copied from the selection.

**`RunnerThroughputAssessment<P>`** — `sole_constructor`, minted only by
`assess_runner_throughput(selection, runs)`, whose first act is the SUBJECT JOIN: every run's
host, program, revision, specimen identity, slot unit and effective bound must equal the
selection's, or the assessment refuses by name (`SubjectJoinRefused { axis, selected, observed }`).
Then per-axis standings under the sampling policy: `ThroughputStanding` (min rate vs pin),
`MemoryEnvelopeStanding` (max peak vs the EFFECTIVE high, which must equal the selected high),
`PhaseWallStanding` (roster completeness, max wall vs ceiling), `SlotWidthStanding` (host plan's
slice ÷ selected slot, min with effective CPU cap ÷ cores-per-slot). `assessed_facts` names the
two facts the claim covers.

**The fourth type IS the new arm on the existing carrier** (parent ruling 2026-09-20):
`product.fabric.node_qualification OfferDeliveryClaim<P>` gains
`OfferClaimsRunnerDelivery { offer: FabricIdentity<P, OfferKey>, assessment: RunnerThroughputAssessment<P> }`,
and `claimed_delivery_check` gains one dispatch arm: the named offer must equal the assessment's
selected offer, and the assessment's refusals surface as
`QualificationRefusal::RunnerThroughputDidNotHold { refusals }`. There is no standalone runner claim
type and no mirrored check — that would be option (b)'s shape surviving beside (a).

### The join point, and one edit outside the new modules

`product.fabric.capacity_admission admit_delivered_offer` takes
`product.fabric.node_qualification OfferDeliveryClaim<P>`, whose only positive arm is the Spark
group admission. Two honest options; the parent ruled (a) on 2026-09-20, and the departure is stated in the PR body:

- **(a) Extend the existing carrier** — `OfferDeliveryClaim<P>` gains
  `OfferClaimsRunnerDelivery { offer, assessment }` and `QualificationRefusal` gains
  `RunnerThroughputDidNotHold { refusals: List<ThroughputRefusal> }`; `claimed_delivery_check`
  dispatches. One delivery-claim authority, one admission fold, no new admission. This touches
  `product.fabric.node_qualification` (a §3b home) — the edit is an arm on its coproduct and an arm
  on its refusal list, nothing in its accelerator axes.
- (b) A parallel `RunnerOfferDeliveryClaim` consumed by a second `admit_delivered_runner_offer` —
  rejected: two delivery-claim vocabularies feeding one admission is the §3 fork the hold names.

## 3. Disposition of every existing declaration

`gunbc.runner_throughput_qualification`:

| declaration | disposition |
|---|---|
| `FloorWorkloadPin` | survives; gains `specimen: NonEmptyStr` (calibration specimen identity) |
| `WorkspaceBacking`, `WorkspaceBytesLocus`, `workspace_bytes_locus`, `workspace_backing_wire` | survive; the observed run carries the locus as READ, the selection carries the declared backing, the join compares them |
| `PhaseWall` | survives |
| `RateSpecimenReading`, `CgroupMemoryReading` | survive as fields of `ObservedRunnerQualificationRun` |
| `QualificationRun` | DELETED → `ObservedRunnerQualificationRun` (sole_constructor, identities added) |
| `HostBudgetFacts` | DELETED → `SelectedHostPlan` (a receipt, not facts to re-plan from) |
| `RunnerThroughputQualification` | DELETED → `SelectedRunnerQualification<P>` |
| `ThroughputStanding`, `MemoryEnvelopeStanding`, `RevisionStanding`, `SlotWidthStanding` | survive (per-axis, three-valued); `RevisionStanding` folds into the subject join |
| `RunnerThroughputReceipt` | DELETED → `RunnerThroughputAssessment<P>` (sole_constructor) |
| `ThroughputRefusal` | survives; gains `SubjectJoinRefused`, `PhaseRosterDidNotHold`, `PhaseWallDidNotHold` |
| `RunnerThroughputAdmission` | DELETED — admission is the fabric's (`CapacityAdmission<P>`) |
| `RouteLegStanding`, `leg_is_bound` | survive |
| `qualification_run_producer_frontier`, `qualification_admission_consumer_frontier` | survive, re-worded: the producer mints `ObservedRunnerQualificationRun`; the consumer is `admit_delivered_offer` with the runner claim |
| `qualify_throughput`, `qualify_memory_envelope`, `qualify_revision` | survive, re-homed under the per-axis aggregation |
| `qualify_slot_width` | survives, re-derived from `SelectedHostPlan` (no `fleet_host_plan_for` call inside) |
| `receipt_for_run` | DELETED → `assess_runner_throughput` |
| `throughput_refusal`, `memory_refusal`, `slot_width_refusal`, `receipt_refusals` | survive |
| `runs_at_pin`, `standing_speaks_before`, `measured_speaks_before`, `run_throughput_standing`, `slowest_run` | DELETED → per-axis aggregation under `SamplingPolicy` |
| `UnreadableRun`, `run_unreadable_reason`, `first_unreadable_run` | DELETED — unread dominates inside the memory axis aggregation |
| `qualify_runner_throughput` | DELETED → `assess_runner_throughput` |
| `qualified_executor_sanction` | DELETED (item 1) |
| `admitted_receipt`, `admission_refusals` | DELETED |
| `qualified_supplier_offer`, `slot_usable_threads` | DELETED (item 1: the offer is selected, not derived) |
| `nat_wire`, `*_wire`, `admission_wire` | survive as `assessment_wire` |

`gunbc.runner_throughput_qualification_route`:

| declaration | disposition |
|---|---|
| `StageControllerWrite`, `stage_controller_write`, `stage_writes_controller`, `route_controller_writes_are_gated`, `route_controller_write_count` | DELETED → `StageEffect`, `stage_effect`, `route_effectful_legs_are_bound_or_declared` (item 6) |
| `QualificationStage` | survives; `RegisterEphemeralRunnerSlot` / `DispatchFloorToSlot` / `DeregisterRunnerSlot` gain a `GitHubControlPlaneWrite { authority }` leg |
| `stage_egress` | survives |
| `cpu_quota_properties`, `ephemeral_slot_properties`, `ephemeral_slot_argv`, `ephemeral_slot_labels`, `ephemeral_slot_unit` | survive; the expected effective properties become the `SelectedSlot` the observed run is joined against |
| `QualificationInstrument`, `qualification_instrument_producer`, `qualification_instrument_wire`, `qualification_instruments` | survive |
| `mtcollins1_qualification_workspace_backing`, `mtcollins1_boot_leg`, `mtcollins1_image_leg` | survive |
| `mtcollins1_capacity_class_candidate` | DELETED (item 1) |
| `qualification_route`, `mtcollins1_qualification_route` | survive; take the selection rather than a bare workload |
| `route_registers_one_slot`, `route_deregisters_what_it_registered`, `route_dispatch_selector_names_the_attempt`, `route_boot_leg_is_bound` | survive |

Witness (`test.claim.runner_throughput_qualification_witness`): rewritten around the four types.
Fixtures supply a `SelectedRunnerQualification` and producer-shaped observed runs; every REDs row
of §1 gets a claim; the frontier claims stay expecting-red; the inhabitance claims for
`floor_execution_requirements`, `admit_delivered_offer` and `gunbc_runner_cores_per_slot` stay.

## 4. The REDs the recut must execute

1. Foreign host: observed run's host ≠ selection's host → `SubjectJoinRefused`.
2. Same eval-step count, different program/specimen → `SubjectJoinRefused` (the specimen identity is part of the join, not only the count).
3. Wrong cgroup / effective `memory.high` ≠ selected slot high → `SubjectJoinRefused`.
4. Wrong workspace backing (observed locus ≠ selected backing) → `SubjectJoinRefused`.
5. Directly authored admitted disposition → not writable: `RunnerThroughputAssessment` and `ObservedRunnerQualificationRun` are `sole_constructor`; the witness's positive control is the only way an admitted claim exists and it goes through `assess_runner_throughput`.
6. Two-run aggregation: A slowest passing memory, B faster reaching `memory.high` → not admitted (max peak axis).
7. Missing or duplicated required phase → `PhaseRosterDidNotHold`.
8. Sanction mismatch: assessment holds, selected sanction is `CustomerExecutableCapacity`, requirements are control-plane → `admit_delivered_offer` refuses through the existing `CapacityClassRefused` path; no function mints a sanction from an assessment.
9. Unchargeable / oversubscribed selected host plan → `SlotWidthUnderivable`, no quantity.
10. Carried forward: no run → refuse; stale revision → `SubjectJoinRefused { axis: Revision }`; unreadable bound → refuse; throughput below pin → refuse with the number; unbound effectful leg → route not executable.

## 5. Order of work (delete-first, §3 replacement migration)

1. Delete the condemned root in one motion: `RunnerThroughputQualification`, `RunnerThroughputReceipt`, `RunnerThroughputAdmission`, `qualify_runner_throughput`, `qualified_executor_sanction`, `qualified_supplier_offer`, the run-selection fold, `HostBudgetFacts`, `StageControllerWrite`. The witness refuses loudly; that is the census.
2. Land the four types and `assess_runner_throughput`; re-home the surviving standings and folds.
3. Extend `OfferDeliveryClaim` / `QualificationRefusal` (option (a), ruled) and add the dispatch arm to `claimed_delivery_check`; add the inhabitance claim that `admit_delivered_offer` refuses an unqualified runner claim by name and a claim riding another offer.
4. Recut the route's effect classification and bind or declare every effectful leg.
5. Rewrite the witness around §4; hermetic run scoped to the entry; parse sweep.
6. File the `gunbc.recurring_failure_mode` row `witness_oracle_transcribed_from_the_tree_it_measures` (done alongside this plan) and register this document in `gunbc.doc_graph_roots`.

Evidence bar unchanged: `v1_src_dag_parse` clean, every claim under the new-witness budget, the
required `witnesses` check green, the parent's read of this plan before step 1.

## 6. Retirement

This document retires when the three types and the `OfferClaimsRunnerDelivery` arm are on main
with `admit_delivered_offer` consuming it, the effectful route legs are bound, and the first
`ObservedRunnerQualificationRun` is minted from a real run on mtcollins1 — at which point the
frontier rows it re-words have dissolved and the plan describes a shipped system.
