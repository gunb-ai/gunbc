# Throughput-qualified convergence — one interface, one bound subject, one frontier

Standing: a design under revision, not a landed capability and not yet enrolled in DESIGN. It has been
through three review rounds (side-chat, 2026-09-17); the corrections are recorded in place rather than
absorbed, because several of them are the kind of error this document exists to make harder.

## The defect this exists to close

**A convergence today verifies its CONFIGURATION and stops. Neither subject verifies the RATE the
configuration was chosen to deliver.** The configuration is a proxy for the goal, and reading the goal
off the proxy is the gap.

Two live instances, each found by looking:

- **Serving, Group B, 2026-09-17.** A converge reported success and was independently verified on four
  hosts: image derived, pin correct, checkpoint loading, fp8 admission holding, KV law single-sourced.
  Every one of those is true and none is about collective transport. The arm was running every NCCL
  channel over `NET/Socket/0` with no IB device present, which the converge could report full
  agreement through because transport was not in its comparison.
- **CI.** `docs/plans/ci-humming.md` derives a runner-slot count from a memory budget across T0-T6 and
  never measures jobs per hour afterwards. T1 carries the right instinct one level down — a cap is not
  effective until the live cgroup is read back — and stops at configuration.

## Convergence and qualification are two facts, not one (corrected)

An earlier draft said "converged" should mean configuration agreed AND the rate was met. That is
wrong, and the error is worth keeping: a realization can match its desired image, checkpoint, argv,
transport plan, capabilities, devices and rank population exactly while missing a throughput floor.
That is not configuration drift, and redefining convergence to cover it destroys a distinction the
operator needs — *state agreement* and *service qualification* have different remedies.

`std.goal_assessment` already owns the remedy-free relation between a caller-supplied goal and an
independent observation, and deliberately keeps observation, assessment and remedy apart. So the
shape is a composition, not a redefinition:

    ConfigurationStanding = ConfigurationConverged | ConfigurationDiverged | ConfigurationIndeterminate
    ThroughputAssessment  = GoalSatisfied | GoalDiverged | GoalIndeterminate | GoalAssessmentRefused
    OperationalQualification
      = RealizationQualified { configuration_receipt, throughput_assessment }
      | ConfigurationNotConverged { .. }
      | ThroughputNotQualified { .. }

This also repairs the rung argument. A configuration convergence with no throughput result is a
legitimate value, not a defect. The invalid state to remove is the positive carrier
`RealizationQualified` minted with an absent assessment — and THAT can be made **structurally
impossible**, because a sole-constructor carrier cannot be built without its assessment. Enrolling
every production route remains mechanically preventable. Two different rungs for two different
claims, which is what §4b(1) asks for.

## Why this is one interface (DESIGN §2, horizontal)

"16 decode streams against a serving arm" and "N concurrent builds against a runner pool" share an
interface: within a declared window, apply a declared workload to a subject, derive a rate from a
counter the system itself keeps, and assess it against a floor that is not a copy of the last reading.

The partition matters more than the sharing, and DESIGN §3b's fabric row already states it for
capacity: **selection is shared; occupancy is not.**

    shared:           observation and assessment interface, rate derivation, goal assessment
    subject-specific: isolation acquisition, workload realization, counter producer,
                      subject identity, occupancy and release mechanism

So serving's termed `product.capacity.lease::LeaseGrant` is NOT the CI window's carrier. Compute
occupancy is a timeless `LeaseIdentity` settled by `file_compare_and_set`. An earlier draft of this
document cited `std.temporal_effect::HeldLease` for both, which carries an epoch and an observed state
and **no deadline at all**; a later draft then cited `LeaseGrant` for both, which crosses the very
boundary the roster draws. Each subject brings its own occupancy carrier.

## The parts, each with the failure it prevents

### 1. A quiet read is not a reserved window
A CAS stops two probes believing they hold one claim. It does not stop a production request arriving
after `num_requests_running == 0`, GitHub dispatching ordinary work after a queue check, or an
already-admitted operation becoming active mid-measurement. Isolation is therefore a standing the
subject-specific handler PRODUCES, not something the generic fold assumes:

    ProbeWindowStanding
      = WindowIsolated { grant, deadline, drain_receipt, exclusion_receipt }
      | WindowSharedWithDeclaredTraffic { traffic_receipt }
      | WindowIsolationUnread { cause }

Serving realizes it by route withdrawal or a dedicated endpoint, a drain, and a fence against
non-probe requests; CI by a dedicated runner population withdrawn from ordinary labels, drained, with
probe-only dispatch. Without an exclusion receipt the reading is still useful production telemetry —
it is simply not a controlled qualification, and must not be recorded as one.

### 2. The offered load is a policy, and the receipt constrains it rather than manufacturing it
Declaring the load is necessary; grounding the FIGURE is the rest. But an earlier correction here
introduced circularity — "concurrency derived from the admitted policy, and the policy owes a capacity
receipt" — which has the receipt manufacture the workload that then verifies it. The clean relation:

    QualificationProtocol        declares the offered load (e.g. 16 concurrent)
    ObservedCapacityReceipt      may admit or refuse that policy as safe or plausible
    ThroughputObservation        establishes the delivered rate AT that load

A contract may legitimately require 16 concurrent before anyone has found maximum capacity. And
`max_num_seqs` is a **scheduler ceiling, not a workload authority**: a launch configured for 16 may be
qualified at 8, 16 or another declared load depending on the contract being tested.

### 3. Backlog is what is observed; saturation is an inference, and capacity a further one
Two drafts got this wrong in opposite directions — first "queue depth > 0 means saturated", then "a
sweep establishes it". vLLM's own metric definitions treat running as current execution and waiting as
backpressure; neither defines a physical-saturation verdict. `16 running / 0 waiting` can be fully
occupied, and `16 running / 5 waiting` can be waiting on `max_num_seqs` or KV admission with no
hardware bottleneck identified. So:

    queue > 0  does NOT establish saturation
    queue == 0 does NOT establish unsaturation

The reading names only what was observed — `BacklogObserved` | `NoBacklogObserved` | `BacklogUnread` —
and a capacity inference is a separate operation with its own declared load progression and plateau
criterion. **For an operating floor, capacity need not be inferred at all:** *under protocol P at
offered load L, did the subject deliver at least floor F?* is answerable whether or not L saturated
anything.

*Correcting this document's own figures:* the 2026-09-17 socket readings retained no queue trace, so
they are `BacklogUnread` and `CapacityUnestablished`. The 8-to-16 scaling argument (aggregate 27.31 to
51.19 while per-stream moved 3.41 to 3.20) is useful causal reasoning and is **not** a queue
observation; assigning `UnsaturatedAtOfferedLoad` from it, as an earlier draft did, was an inference
reported as a reading.

### 4. Subject equality alone does not make two rates comparable
`serving_performance_subject` is a configuration fence, and its own source says launch identity is a
separate boundary — several launches may share one subject. The observation therefore carries more
than the subject:

    ThroughputObservation<Subject, Protocol, Realization, RateUnit> {
      subject            the effective configuration
      protocol           concurrency, work identity, sizes, stopping rule, warmup and cache rules
      realization        the exact serving incarnation or runner population
      window_standing    isolated, shared under declared traffic, or unread
      counter_producer   who kept the count
      counter_span       before, after, and continuity across the interval
      operation_ledger   every offered operation reached an admissible terminal outcome
      raw_carrier        the bytes the reading was taken from
      observed_at
    }

**Counter continuity** is its own field because a counter that reset, rebound or switched incarnation
mid-window yields a delta that is arithmetic rather than a measurement.

**Operations and rate units are different grains**, which an earlier draft conflated by counting
"attempted, completed and refused tokens". For serving the operation is a request and the unit is a
generated token — and a request can emit tokens and *then* fail, so the ledger cannot be denominated
in tokens. For CI the operation is an exact Work/job and the unit a completed work occurrence; "jobs
per hour" means something only over homogeneous work or a declared normalization, because a one-minute
no-op and a fifteen-minute clean build are not interchangeable units.

### 5. The rate is derived, never stored
`gunbc.harness.harness_throughput` already states this law for its own stream — counts and interval
are carried, the rate is a function of them, and storing it alongside would be a second representation
free to disagree with its inputs. Cited, not re-coined.

### 6. The floor is not a copy of the last reading
DESIGN §5: a merge-blocking literal needs a controlled fixture, an external or versioned authority, an
explicit policy budget, or a monotone debt contract. A floor pasted from this morning's run collapses
to `measure() == measure()`. So the floor is a declared operating budget someone owns, or a monotone
contract raised deliberately, with provenance either way. The verdict is `std.goal_assessment`.

## Conformance (DESIGN §3b)

Rows touched, with module names as the modules declare them (an earlier draft prefixed two of these
with `gunbc.`, which is the file path and not the module):

- **fabric / compute** — `product.capacity.lease::LeaseGrant`, `product.fabric.selection::select_supply`.
  The row already states the partial sharing this design inherits: shared selection, separate
  occupancy producers and linearizations.
- **leasing / locking / grants** — the serving window as a termed grant; the CI window through
  compute's own timeless identity. `std.durable_compare_and_set::CasExpectation` is the linearizer and
  is not by itself an exclusion receipt.
- **process observability / reporting** — `std.observation::ObservationEvent`,
  `std.observation::RecordedObservation`.
- **decision / selection** — deliberately NOT touched while the probe only assesses. The moment it
  ranks configurations, `std.decision` is the home and §3d binds.

## Evidence standing for readings already taken

An earlier draft argued the socket readings qualify as a receipt BECAUSE the arm is gone and the
question cannot be put again. That is wrong: irreproducibility establishes urgency to preserve
evidence, never provenance for it. The honest standing follows what was retained:

| retained | standing |
|---|---|
| exact subject, protocol, incarnation, raw counter and request traces, interval | subject-bound historical observation |
| summarized numbers with no raw producer carrier | historical reported reading |
| no queue trace | `BacklogUnread` |
| successor moved transport AND other axes | no transport-only attribution |
| no exact workload identity | protocol-unbound historical claim |

The 2026-09-17 socket figures are a **historical reported reading**: summarized aggregate and
per-stream rates, no retained raw carrier, no queue trace. They are worth recording as that, and they
are not a `RecordedObservation`. The arm they were taken against is additionally
`AdoptedOutOfBandRealization` — converged by a lane outside the modelled transaction — so a floor set
from them grounds a claim about the SUBJECT and never about a transaction that did not run.

## Staging

1. **Record the socket figures at their honest standing** — historical reported reading, `BacklogUnread`,
   `CapacityUnestablished`, `AdoptedOutOfBandRealization`.
2. **The serving probe** — extdeps rows for the completions route and the metrics counters; window,
   protocol, ledger, counter-span and rate folds; per-layer witnesses on supplied inputs with one
   inhabitance claim that runs the real endpoint.
3. **The CI subject is a FRONTIER, not a bound subject** — which is why this document's title says one
   bound subject and one frontier. `ci-humming.md` is the June slot/cgroup program and carries the
   older control/apply architecture; current main has a Firecracker/JIT runner-attempt planner working
   through fabric identities and execution grants. Stage 3 therefore begins with a producer-and-consumer
   census of the CURRENT tree, not the historical plan, covering at least: exact Work and source-tree
   identity, runner image/kernel/rootfs, microVM shape, host class, slot count and memory envelope,
   jobserver and resource controls, runner/JIT revision, toolchain revision, cache topology and cache
   state, and dispatch policy. Which of those invalidate a prior rate is decided by that census.
4. **The CI probe**, binding the same interface to dispatched builds.
5. **Qualification** — `OperationalQualification` composed from the two standings, with
   `RealizationQualified` unmintable without its assessment.

## Not yet done, stated plainly

This plan is **not enrolled in DESIGN**. DESIGN.md is generated from `gunbc.design_document` and is
never hand-edited; a plan is linked from the section that governs it. A file existing under
`docs/plans/` is not that linkage. Enrolment lands when the design settles, and until then this
document governs nothing.
