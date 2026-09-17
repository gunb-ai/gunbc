# CI throughput subject — a census of the current tree

Stage 3 of [throughput-qualified convergence](throughput-qualified-convergence-design.md), which named
the CI side a frontier rather than a bound subject and required its census to read the CURRENT tree.
That instruction was specific and is honoured literally here: `docs/plans/ci-humming.md` describes the
June slot/cgroup program and an older control/apply architecture, and deriving axes from it would
model a fleet that has since been replaced.

This is a census, not a design. It answers one question — **which properties, if they change, make a
prior CI rate inapplicable** — and it answers it by naming carriers that exist today.

## The finding that motivates the rest

**Nothing in the runner subsystem measures a rate.** 76 modules under `dag/gunbc/runner/` model slot
identity, width admission, microVM sizing, placement, connectivity repair, and receipts for nearly
every step of an attempt's life. Searching them for a throughput quantity returns two hits, and
neither is one: `runner_unit_file` `InvocationLocalCargo { jobs_per_slot }` is a **cargo invocation
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

**3. There is no join from an attempt to the exact work it performed.** A rate needs an operation
ledger over homogeneous work (the parent design's point that a one-minute no-op and a fifteen-minute
clean build are not interchangeable units). The attempt receipts are rich, but the census found no
carrier binding an attempt to the exact Work identity and source tree it built, which is the
precondition for denominating any CI rate at all.

**4. Host class is implied by module paths rather than carried.** Mt Collins facts live in extdeps
briefs and the runner modules observe individual hosts; nothing names "this host belongs to class C"
in a way a measurement could cite.

## What this census does NOT do

It does not propose the CI subject type. Three of the four gaps above are modelling obligations
(DESIGN §6) rather than conformance rows (§3b), because a row whose home does not exist is an
obligation and not a domain — and coining a subject over carriers that do not exist would be the
re-invention DESIGN §2's test names. The order is: close the cache-state and work-identity gaps, then
assemble the subject from carriers that exist, then probe.

It also does not rank hosts, shapes or widths. The moment anything chooses among configurations,
`std.decision` is the home and §3d binds.

## Standing

A census, not enrolled in DESIGN, governing nothing. Its consumer is stage 3 of the parent design,
and its finding — that CI models its configuration thoroughly and its rate not at all — is the same
finding the parent made for serving, arrived at independently on the other side of the fleet.
