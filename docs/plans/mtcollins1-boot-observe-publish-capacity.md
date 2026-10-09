# mtcollins1 boot: observe and publish capacity; health separate from the boot verdict

Status: APPROVED by eager-owl-205 (2026-09-26) with the frontier answer below; work item adhoc-0e07aa33-b20.
Off the #12362 → publish → repin → boot critical path: nothing here lands before #12362.

## The defect, read as a chain (DESIGN §6b)

`gunbc.machine_intake_mtcollins1_boot_run` `mtcollins1_boot_qualifications` folds six
qualifications into ONE terminal verdict: `nproc`, `numa`, `boot-media`, `edac-before`,
`workload`, `edac-after`. Two of them (`nproc`, `numa`) compare the observation to an
INTENDED topology (`gunbc.machine_intake_mtcollins1_boot_milestone`
`mtcollins1_boot_topology_expectation`), so an intent mismatch is reported as "did not boot".
The earliest unjustified boundary is not the 160/80 literal; it is that the verdict type answers
four questions with one arm set. Patching the literal (the #12325 milestone + the rung drop
`mtcollins1_boot_accepts_socket1_absent`) was the symptom-link patch: it kept the conflation and
made the intended configuration a boot precondition.

## The four questions and their homes (DESIGN §3b — inhabit, do not mint)

| Question | Home (existing) | What changes |
|---|---|---|
| 1. Did it boot | `gunbc.machine_intake_mtcollins1_boot_run` terminal verdict | Verdict narrows to evidence the census image reached its interfaces: envelope closed/sections terminated, `boot-media` (the image we delivered is the one running). `nproc`/`numa`/`edac-*`/`workload` leave the verdict. |
| 2. What is available | `gunbc.compute.host_capacity` (`HostMemoryObservation`, the host `Pool<Memory, Kibi>` over `product.capacity.pool`), `product.fabric.envelope` `ResourceEnvelope`, `product.fabric.work` `HardRequirements { threads: HardwareThreadCount }` | A capture-sourced observation reading of CPU set (count + per-NUMA-node CPUs), and MemTotal/MemAvailable and `/dev/shm` size, each field `Observed \| Unknown{cause}` — malformed never becomes zero. Published regardless of (3). No new capacity vocabulary: threads are `HardwareThreadCount`, memory `Kibibyte`, the pool is `product.capacity.pool`. |
| 3. Health | `product.host_health` (`MemoryErrorCounters`, `CounterWindowStanding`, `HostHealthGoal`, assessed via `std.goal_assessment`) | EDAC before/after become a `CounterWindowStanding` over `MemoryErrorCounters`; workload digest stability a health signal; "observed topology vs intended (2 sockets × 80) and vs earlier observations" becomes a `GoalDiverged` deviation, never a boot refusal. Findings carry a consequence (restrict / quarantine), never delete the (2) observation. |
| 4. Eligibility | `product.fabric.supply` `offer_covers_shape` / `unmet_*_axis`, `product.fabric.capacity_admission` | Requirements are judged against the (2) allocatable envelope plus a health policy input from (3). No mtcollins1-specific eligibility code. |

Open question for the parent on (2): the capture is a BOOT-TIME observation on a machine not yet
enrolled as a fabric host (no `HostDashboardInstance`). I propose the observation lands as a
typed reading in machine_intake that is *shaped* as `ResourceEnvelope` + `HostMemoryObservation`
so enrolment consumes it unchanged; the actual pool publication is a declared frontier (§3c)
triggered by mtcollins1's fabric enrolment. If you want it published to a fabric partition now,
I need the enrolment home named.

## Memory test sizing (the #12362 residue)

Sizing follows the observation, never socket count, under an explicit policy row:
`reserve` (bytes held back from the test), `minimum_useful` (below which the test REFUSES as
"not run", not "passed"), and the size attempted = min(MemAvailable − reserve, /dev/shm free).
The receipt records exact bytes attempted and coverage = attempted / MemTotal. A bounded run is
typed `BoundedCoverage { attempted, of }` and can never read as full qualification; full
qualification is a separate arm requiring coverage at the declared threshold. The #12362
"one-socket 16 GiB profile" dissolves into this policy once #12362 has landed.

## The cut (DESIGN §3 replacement migration, delete-first)

The root is `gunbc.machine_intake_mtcollins1_boot_milestone`. Its consumers, enumerated by name
(this root is not in the required gate prefixes' blast radius alone, so named, not inferred):
`gunbc.machine_intake_mtcollins1_boot_run`, `test.claim.machine_intake.mtcollins1_boot_run_witness_test`,
`gunbc.rung_drop.mtcollins1_boot_accepts_socket1_absent` (+ `gunbc.rung_drop` roster,
`docs/design-rung-drops.md` projection).

One PR, one motion:
1. DELETE `mtcollins1_boot_milestone.dag` and the `nproc`/`numa` qualifications and their causes
   (`SocketPlacementMismatch`, `SocketPlacementUnread`, nproc mismatch) from the terminal verdict.
2. RETIRE the rung drop by construction: its subject ("boot qualification accepts socket 1 absent")
   no longer exists because the boot verdict no longer asks about sockets. The gate it lowered is
   not lost — it moves UP to health, where an unexpected reduction is a `GoalDiverged` finding
   against the intended 2-socket configuration. Nothing is weakened, so no new drop.
3. KEEP `mtcollins1_nproc = 160` untouched: a true receipt about the census boot it was read on;
   it becomes one of the "earlier observations" health compares against.
4. The numa-line parser (`numa_node_cpus_line`) moves into the capacity observation, not copied.
5. EDAC/workload move from verdict into the health assessment in `product.host_health`.

Witnesses: an 80-CPU single-socket capture BOOTS, publishes 80 threads, and carries a health
deviation; a malformed nproc section boots (if boot-media held) with threads `Unknown`, never 0;
a capture missing boot-media does not boot; EDAC UE>0 boots but health restricts; the route claim
exercises the real `mtcollins1_boot_terminal_verdict` path (§3 pairing obligation).

No live hardware is touched.

## Correction after reading the homes: in-band health is not `product.host_health`

`product.host_health` is built on a standing rule that its signals are read OUT OF BAND ("a count
the BMC does not expose is an unknown, never a host-OS read"). nproc, numactl and in-band EDAC
from the census image are host-OS reads. So the topology health assessment is a sibling
instance of the shared authority, `std.goal_assessment`, over an in-band subject, in
`gunbc.machine_intake_host_resource_observation`. It is a stated divergence from
`product.host_health`, not a silent fork. In-band EDAC follows the same route at the cut (below):
`product.host_health` `MemoryErrorCounters` / `counter_window_standing` are reused as values,
without the out-of-band epoch claim.

## Staging (parent ruling: nothing touches the #12362 → publish → repin → boot path until that boot has run)

- **PR 1 (this change).** Questions 2–4 as new modules:
  `gunbc.machine_intake_host_resource_observation` (observation → `host_topology_assessment` →
  `topology_health_admission` → `host_offered_shape` → `host_workload_eligibility` over
  `product.fabric.supply` `unmet_shape_axes`), and `gunbc.machine_intake_mtcollins1_topology_goal`.
  The numa reading has its own strict parser (node and CPU ids, duplicates refused). The milestone module's count-only parser is frozen and deleted at the cut.
- **PR 2 (the cut), after the #12362 boot has run.** Delete the milestone module and the
  nproc/numa verdict arms. Retire the rung drop. `mtcollins1_boot_run` feeds its envelope's
  sections into `host_resource_observation`, so the boot run becomes the production consumer.
  EDAC and workload move out of the verdict.
- **PR 3, after #12362 lands.** Memory-test sizing policy.
- **Declared frontier.** Pool publication into `product.capacity.pool` via
  `gunbc.compute.host_capacity`. Trigger: mtcollins1's fabric enrolment (a `HostDashboardInstance`
  naming it). No enrolment home is invented here.
