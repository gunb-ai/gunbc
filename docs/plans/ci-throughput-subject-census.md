# CI throughput comparability — a whole-path census of the current tree

Stage 3 of the throughput-qualified convergence design, which named the CI side a frontier rather than
a bound subject and required its census to read the CURRENT tree. **That parent does not exist in this
tree: it is open as gunbc#11540 and this census must land with or after it** — the link below resolves
only once that PR lands, and naming it here is the declared frontier's trigger (§3c) rather than a
citation that silently dangles: [throughput-qualified-convergence-design.md](throughput-qualified-convergence-design.md).
That instruction was specific and is honoured literally here: `docs/plans/ci-humming.md` describes the
June slot/cgroup program and an older control/apply architecture, and deriving axes from it would
model a fleet that has since been replaced.

This is a census, not a design. It answers one question — **which properties, if they change, make a
prior CI rate inapplicable** — and it answers it by naming carriers that exist today.

## The finding that motivates the rest

**No fold anywhere derives an aggregate CI rate** — completed homogeneous work per unit time, bound to
an admitted subject, a declared protocol and an observed realization. Durations DO exist and are
derived (`gunbc.superseded_run_starvation_census` `timestamp_diff_seconds`); a rate does not.

The runner modules under `dag/gunbc/runner/` model slot identity, width admission, microVM sizing,
placement, connectivity repair, and receipts for nearly every step of an attempt's life. The query
behind the claims below is named rather than its output copied (§6): a case-insensitive search of
those modules for a rate or duration vocabulary — `throughput`, `per_hour`, `jobs_per`, `duration`,
`elapsed`, `wall_clock` — re-derives what this census read, and a reader who runs it today sees
whatever is true today rather than what was true when this was written. **Naming the producer instead
of its count is not a formality here**: an earlier revision of this census copied a hit count out of a
narrower search and used it as the evidence for a claim this document has since retracted (see the
corrections below), so the copied number outlived the search that produced it by exactly the margin
that makes transcription dangerous.

What that search surfaces in the runner modules is not a rate: `runner_unit_file` `InvocationLocalCargo { jobs_per_slot }` is a **cargo invocation
parameter** — how many compile jobs one slot may spawn — and `runner_slot_allocation` carries a prose
aside about oversubscription degrading throughput. Both are inputs to a configuration; neither is an
observation of delivered work per unit time.

So the CI side is in exactly the state the parent design describes for serving: a configuration
derived with care, converged, verified, and never put to the question it was chosen to answer.

## Candidate axes, each with its carrier in the current tree

An axis is a property whose change makes a prior number inapplicable. Each row names where the
property lives today, so the eventual subject type is assembled from existing authorities rather than
re-coined.

| candidate axis | carrier in the tree today |
|---|---|
| host class | `gunbc.runner.runner_host_tpm_observation` and the Mt Collins extdeps briefs distinguish host populations; no single host-class carrier joins them yet |
| requested and observed slot width | `gunbc.runner_capacity_plan` `RunnerCapacityHostPlan { requested_width, observed_width }` |
| microVM shape | `gunbc.runner.runner_microvm` `RunnerMicroVmSize { vcpu_count, memory }`, with `GuestMemoryRequest` and `GuestResources` beside it |
| guest image | `gunbc.runner.runner_microvm` `RunnerGuestImage { distribution, release_series, arch, runner }` |
| actions-runner revision | `RunnerGuestImage.runner` (`ActionsRunnerRelease`) |
| cores per slot / jobs per slot | `gunbc.runner_slot_allocation` `gunbc_runner_cores_per_slot`, consumed by `runner_unit_file` `InvocationLocalCargo` |
| memory envelope per slot | `gunbc.memory_per_vcpu_convention`, with the microVM sizing relation |
| kernel and rootfs | `gunbc.runner.runner_microvm` config resolution (`kernel_path`, `rootfs_path`) |
| toolchain revision | not carried by the runner subsystem; the closest authority is the repo's own toolchain coherence module |
| cache topology and cache state | **no carrier** — see below |
| exact Work and source-tree identity | `gunbc.compute.work_request` `WorkOperation` is the nearest home; the join from a CI attempt to the exact tree it built is not modelled |
| dispatch policy | `gunbc.runner_attempt_launch` and the broker modules carry attempt admission; no policy-identity carrier |

## What the census found missing, which is the useful half

**1. Cache state has no carrier, and it is the axis most able to produce a meaningless comparison.**
`sccache` appears in the build environment and in `extdeps.cache` as a general notion, but nothing
models whether a given attempt ran against a warm or cold cache, or against which cache population. A
cold-cache run and a warm-cache run over the same tree on the same hardware differ by a large factor,
so without this axis two honest readings can disagree wildly and neither is wrong. Any CI floor set
before this exists is a floor over an unstated variable.

**2. The actions-runner revision already diverges across the fleet, and it is declared.**
`runner_microvm` states that the guest image pins 2.337.0 while the fleet's host slots pin 2.336.0 —
found in review rather than by anything in the repository, and resolved forward deliberately. This is
a live specimen of exactly what an axis is for: two attempts on "the same fleet" can differ on runner
revision depending on which population served them, so a rate compared across that boundary is
comparing two subjects.

**3. The rate INPUTS largely exist and nothing folds them — which an earlier revision of this census
missed, and the correction makes the CI probe cheaper rather than more expensive.**
`extdeps.github.workflow_runs` `WorkflowJobRun` carries `created_at`, `started_at`, `completed_at`,
`run_attempt`, `labels`, `status` and `conclusion`, and `gunbc.public_workload_census` already holds
observed execution rows with those timestamps populated. So the claim to make is narrower and sharper
than "nothing measures a rate": **every current consumer uses those timestamps as ORDERING
PREDICATES, never as an interval.** `gunbc.run_disposition` asks whether `run_started_at` is strictly
later than `created_at` to prove execution began; `gunbc.pr_base_freshness` asks whether a base commit
landed after a run started — both use the timestamps as ordering predicates rather than intervals.
That is a fact about THOSE TWO consumers and not about the tree, which is where an earlier revision of
this census overreached.

**And the sentence above was itself wrong when first written, which is the correction worth reading.**
A revision of this census asserted that "neither subtracts; no fold in the tree derives a duration, a
queue delay or a rate". That is false. `gunbc.superseded_run_starvation_census` declares
`timestamp_diff_seconds(end, start) -> DurationComputed { duration_seconds: Second }` and uses it for
dead time under hold and for group hold after jobs — real timestamp arithmetic into a typed
`std.measure` quantity, over live workflow-run and job API calls with paging and terminal
classification. It is an executing precedent for acquisition and interval derivation, not a declared
shape. The claim was over-stated from an incomplete grep and asserted as established, which is the
§4d failure of asserting as deduced what was only inferred.

The surviving claim is narrower and is the one that matters: **no fold derives an aggregate RATE** —
completed homogeneous work per unit time, bound to an admitted subject, a declared protocol and an
observed realization. Durations exist; a rate does not.

Two consequences. A CI rate needs a fold over a substrate that exists, with an interval precedent
already executing — a materially smaller job than this census first implied. And because that
substrate is a census of REAL production jobs, some CI rate questions can be answered without a
synthetic probe at all, as a reading over observed work carrying `WindowSharedWithDeclaredTraffic`
rather than an isolated window: a weaker standing, much cheaper, and for a trend possibly the right
instrument.

What remains genuinely absent is the **join from an attempt to the exact Work identity and source tree
it built**. A rate needs an operation ledger over homogeneous work (a one-minute no-op and a
fifteen-minute clean build are not interchangeable units), and job labels plus a run attempt do not
establish what was compiled. That, not the timestamps, is the precondition still missing.
(Correction raised by the side-chat review, 2026-09-17.)

**4. Host class is implied by module paths rather than carried.** Mt Collins facts live in extdeps
briefs and the runner modules observe individual hosts; nothing names "this host belongs to class C"
in a way a measurement could cite.

## Corrections after review (2026-09-17)

**The census searched the wrong closure.** It read the 76 modules under `dag/gunbc/runner/` and drew a
conclusion about CI, while the measurement substrate lives largely outside that subtree:
`extdeps.github.workflow_runs::WorkflowJobRun` (exact run attempt, status, conclusion, observed runner
labels, three timestamps, with exact-attempt and latest-attempt listings),
`gunbc.public_workload_census::ObservedExecution` (which joins a job to repository, workflow path,
workflow-blob SHA at the observed head, head SHA and an observed correctness standing — much closer to
a homogeneous operation ledger than this census allowed), and
`gunbc.superseded_run_starvation_census` (live API acquisition plus the interval derivation above). A
subtree is not a closure, and "I searched `runner/`" does not ground a claim about CI.

**A second run-record authority already exists.** `extdeps.github.actions_runs` is another
workflow-runs projection with run timestamps and a `gh run list` transport, and `workflow_runs`
already names the pair as a meaning fork awaiting consolidation. A CI probe must CHOOSE or consolidate
that authority; introducing a third run record would make the fork three-way (§3).

**The axis table conflates three layers.** Its test — does changing this make a prior rate
inapplicable — identifies a comparability dimension but does not decide where the dimension lives, and
the parent design already partitions subject / protocol / realization / window. Putting exact Work in
the subject makes the runner a new subject every commit; putting cache POPULATION in the subject makes
it change on every fill or eviction. Both are comparability facts and neither is stable configuration.

| dimension | layer |
|---|---|
| host hardware class; configured slot width and resource envelope | subject |
| runner agent, guest environment, toolchain identities | subject |
| cache implementation, namespace/topology, configured policy | subject |
| exact Work, source tree, workflow definition | protocol (operation identity) |
| offered concurrency, stopping rule, required warm/cold condition | protocol |
| exact cache population encountered; exact host, VM, runner and attempt population | realization |
| ambient or foreign work during the window | window/isolation standing |

So the cache finding splits three ways rather than being one missing axis: topology and policy are
subject, the required condition is protocol, the observed hit state is realization.

**The Work-identity gap is narrower than stated, and the repair is different.**
`gunbc.runner_attempt_launch::plan_attempt_launch` already RECEIVES exact Work, Demand and Offer
identities and calls `grant_authorizes_reservation` before planning the VM — so Work identity reaches
the launch planner. The loss is downstream: `LaunchAuthorized` retains the coarse `GrantAuthorization`
verdict and not the admitted identities, so later receipts cannot rejoin the attempt to the exact Work.
The missing piece is a BRIDGE — admitted fabric Work identity → launched microVM attempt → observed
`WorkflowJobRun` and terminal receipt — and the repair carries the existing admitted identity forward
rather than coining a second CI-work identity.

**Two "existing axes" are locators, not execution identities.** `RunnerGuestImage` carries
distribution, release series, architecture and actions-runner release; none of that content-identifies
the guest rootfs bytes, whatever its own comment calls itself. The launch planner likewise takes
`firecracker_binary`, `kernel_source` and `rootfs_source` as paths, and one path can name different
bytes across hosts or across time. So exact rootfs content identity, exact kernel content identity,
Firecracker/jailer release identity, and a host execution class covering host kernel, CPU class/policy
and storage realization are all REMAINING GAPS, not settled axes.

## What this census does NOT do

It does not propose the CI subject type. The gaps above are modelling obligations
(DESIGN §6) rather than conformance rows (§3b), because a row whose home does not exist is an
obligation and not a domain — and coining a subject over carriers that do not exist would be the
re-invention DESIGN §2's test names. The order is: close the cache-state and work-identity gaps, then
assemble the subject from carriers that exist, then probe.

It also does not rank hosts, shapes or widths. The moment anything chooses among configurations,
`std.decision` is the home and §3d binds.

## Standing

A census, not enrolled in DESIGN, governing nothing. Its consumer is stage 3 of the parent design —
which is gunbc#11540 and is NOT YET IN THE TREE, so this document's only consumer is a declared
frontier and its merge dependency is that PR,
and its finding — that CI models its configuration thoroughly and its rate not at all — is the same
finding the parent made for serving, arrived at independently on the other side of the fleet.
