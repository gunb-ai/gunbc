# CI onto per-job microVMs — plan, not a cutover

*Subject: get gunbc and gunbc-private CI onto per-job Firecracker guests. This document does not switch `runs-on`, does not bind `host_boot_cutover_frontier`, and does not start the M1 dogfood clock. It answers the four questions the operator asked, with named instruments. Cutover is a later operator decision.*

*Governed by DESIGN §3 replacement-migration (gap-intolerant staged carve-out). Continues [microvm-launch-displacement-analysis](microvm-launch-displacement-analysis.md) and gunbc#10927 (`gunbc.runner_attempt_launch`).*

## Why this is already a plan item (say this first)

`strategy.year_end_plan` (gunbc-private) already carries:

- **`runner-canary`** — `InProgress`, M1. Exit: GitHub App + `workflow_job` webhook → JIT ephemeral runner in a **per-job microVM** on the canary host → one real job completes → metered receipt → environment destroyed. One real job has already completed on Mt. Collins (gunbc#10733). Three conjuncts remain (authenticated JIT push, metered vCPU-minutes, teardown bound to the attempt identity).
- **`dogfood-started`** — M1. Exit: **gunbc's own CI runs on the canary host through the runner-canary path** — *the two-week observation clock STARTS*. Depends on `hw-first-host`, `boot-minimum`, `fabric-m0`, `runner-canary`.

This is not an infrastructure detour competing with the year-end plan. **Public CI on our own microVM runners is the M1 dogfood deliverable.** Until that exit is true, the two-week clock cannot start, and M1b `dogfood-window` cannot start.

The 2026-09-10 motive is independent and already declared. gunbc-private PR #50 went red on "Build gunbc from the public seed" with exit 126 (~15s, command found but not executable). Public `gunbc.witness_floor_workflow` `toolchain_filesystem_probe_dissolution_condition` is the named cure for that shared-FS eviction class. **gunbc#10985** (still-bear-335) rewrites that row so the probe dissolves only on **both** conjuncts this plan already named — public `selected_ci_runner_target` / `gunbc_ci_selected_runner_spec` and private `witnesses_job()` `SelfHosted` custom labels, not the empty-custom fleet slot — and identifying a deleter on the shared slot is **not** a substitute. Typed env-vs-code (POSIX 126/127, BMC/host/JIT) does not dissolve here. This PR does not flip `runs-on`; #10985 does not either.

## 1. What must be true for `host_boot_cutover_frontier` to fire

Authority: `gunbc.runner_attempt_launch` `host_boot_cutover_frontier`. It is an unbound `DissolutionCondition`.

**The capability, not an artifact:** Y (`plan_attempt_launch` or a successor of the same root) **executes at host boot on tmpfs**, and **the host image is built from that model**, so the recovered `initramfs/init` path has **zero production consumers**. Sufficient for deleting `docs/recovered/mtcollins-host-image-source` and the frontier row in one motion.

**Not sufficient (explicit in the row):** the planner compiling; a `gunbc` binary in a checkout; Y sitting beside X; a witness; recovered init remaining as evidence.

**Shortest honest route:**

1. **Placeability of an amd64/aarch64-correct binary at boot** — `gunbc.runner.runner_host_image_placeability`. Closed is not answered: `HostShellAndBootImageAccessRefused` closes the work item without measuring the image. The wet probe (`host_image_placeability_wet_probe_frontier`) is still unbound.
2. **Carry that binary (or a smaller fold that execs the planned argv) in the host image** and invoke it as the per-attempt driver at the point X currently `exec`s jailer. The converge-time `gunbc run` channel that already displaced steps 1–2 **cannot** do this: it presupposes a self-hosted runner, a checkout, and a built binary, which is what step 9 creates (displacement analysis, reading half).
3. **Discriminating RED + positive control on the host-boot jailer-exec path** — empty/absent credential yields **zero** VMM processes (FCI-3); a live grant starts exactly one. VMM exit 0 stays `WorkNotEstablished` (`gunbc.runner_attempt_launch`).
4. **Host image rebuild from Y**, BMC redirect to that digest, recovered init has no production consumer. Then bind the frontier and delete recovered source in one motion. Gap-intolerant: JIT is one-shot, so Y is built in shadow (already) and the switch is one transition — not A/B on the same attempt.

Until step 4, reported rung stays the **minimum** across paths: `reported_attempt_launch_rung` = `Mitigatable`.

## 2. The four #10927 conditions, given a powered host

| # | Condition | Satisfiable now? |
|---|---|---|
| 1 | Frontier fires: Y execs at host boot, image built from Y | **No.** Planner is still shadow. |
| 2 | No production consumer of Y until the cut (`plan_attempt_launch` not imported by production) | **Still holds** (constraint, not a host-power fact). Must stay true until the one transition. |
| 3 | Displacement done / host-boot jailer-exec executed | **No.** Rung remains Mitigatable. |
| 4 | Plan-level refusals vs X; cutover judged against the "not yet represented" list | See below. Powering the host does not add constructors. |

**Condition 3, measured today (not transcribed from chat):**

- Instrument: `curl -sk --max-time 3 -o /dev/null -w '%{http_code}' https://192.168.1.228/` against `gunbc.machine_intake_mtcollins1_access_observation` `mtcollins1_endpoint`. Result at plan authoring: **HTTP 200**. The BMC answers. Powered-on is not a lie.
- `ping`/`ip`/`ipmitool` are not present in this session, so chassis power and neighbour state are **not** claimed.
- `host_image_placeability_standing` is still `PlaceabilityRefused { cause: HostShellAndBootImageAccessRefused }`. That arm **does not answer the image**. BMC HTTPS ≠ tmpfs exec of a carried `gunbc`.

**Condition 4 — what CI-on-microVMs actually needs** vs customer-job-on-microVMs.

Already on the plan as refusals (keep for CI): grant disagreement; prior resources; presented envelope standing (absent, malformed, signature reject, unit/boot/nonce/attempt binding, payload digest, floor, ceiling); unresolved VM config. CI jobs are untrusted relative to the host: **do not drop these to shrink the cut.**

Not yet represented; CI need vs customer need:

| Missing constructor | CI-on-microVMs | Customer-job |
|---|---|---|
| Live signature verification and fetch (mint is `jit_mint_http_realization_frontier`) | **Required before any production CI job on this path.** The receipted fail-open was `jitconfig-bytes=0` then launch anyway. gunbc#10923 is **merged**, but the named module `gunbc.github_effect_perform` still does not resolve in this tree; `gunbc.runner.runner_jit_perform` still performs no I/O. The frontier's trigger sentence is therefore **not satisfied by the merge alone**. | Same. |
| nft install/readback | **Required.** The guest must reach `github.com` (JIT register, checkout, API). A plan that cannot install or read back NAT is a guest that cannot run Actions. | Same. |
| mkfs failure | **Required.** Checkout and toolchain isolation live on the workspace image. | Same. |
| Live bidirectional cgroup placement census | **Defer for first CI dogfood.** Isolation is structurally the guest+jailer; a live census is evidence quality, not the class that made private exit 126. | Required before selling tenancy (`tenant-boundary-minimum`). |
| PID1 refusals (no boot secret, stage2 missing files) | **Required for host-image Y**, because that is PID1. Without them a bad image loops or launches unauthorized. | Same. |

Like-for-like reproduction of six absent admission checks is still forbidden (displacement analysis). CI does not get a weaker FCI-3.

## 3. Public and private are different problems — sequence

Both workflows today select `[self-hosted, linux, arm64]` (public via `gunbc.ci_runner_target` `gunbc_ci_selected_runner_spec` → fleet offer; private `.github/workflows/witnesses.yml` the same labels). That is the **shared-filesystem srv slot** class (`ctrl` `jit-runner.sh` on srv1–srv4). The canary guest is **Aarch64** (`gunbc.runner.runner_guest_network_receipt` `guest_userspace`). Architecture is not the blocker.

| | Public | Private |
|---|---|---|
| Size | Required floor: two parallel lanes, 90-minute jobs, clippy-all-targets + witnesses | Smaller witness roster; builds gunbc from **unpinned public main** |
| Merge consequence | Blocks every gunbc PR | Blocks private PRs (already largely red today) |
| Adjacent lanes | Filesystem probe already in `witness_floor_workflow`. Typed env-vs-code classification: **still-bear-335** (`adhoc-3b2f737a-7b8`). | Process count: **warm-badger-62** / private #46 — one `gunbc run` per roster row → one process for the enforced half. DESIGN §2, not §5. Do not fork `strategy.private_witness_workflow`. |
| M1 | **`dogfood-started` is this repo's CI** | Not the year-end exit |
| `runs-on` | `gunbc.ci_runner_target` `gunbc_ci_selected_runner_spec` | Not a YAML literal. `witnesses_job()` is `SelfHosted { labels: self_hosted_labels(kernel: Linux, arch: Aarch64, custom: []) }`. Per-attempt label goes in `custom`. #46 does not touch `witnesses_job()`. |

**Why order matters — two improvements can compose into a wash.** #46 removes 34 redundant world acquisitions, which *leaves the seed build as the dominant term*. A naive microVM flip that then compiles that seed with no reachable cache reintroduces cost on that same term. Isolation of the deleter class is still a win; the *job wall* can be a wash. That is the argument for sequencing, and it is stronger than "private is smaller".

**The bar is the whole job against the pre-#46 job wall and against the timeout that was cancelling runs, not the build step.** Producers, so the subtraction re-derives:

- Pre-#46 job wall: gunbc-private actions run [34462653642](https://github.com/gunb-ai/gunbc-private/actions/runs/34462653642), job `witnesses`, `started_at` 2026-09-10T09:48:16Z / `completed_at` 2026-09-10T10:43:57Z → **3341 s**. Private #46 names this run as the 52.8 / 55.7 minute roster-vs-job split.
- Post-#46 job wall (still on `session/warm-badger-62`; #46 is unmerged): run [34512318040](https://github.com/gunb-ai/gunbc-private/actions/runs/34512318040), job `witnesses`, `started_at` 2026-09-10T18:06:50Z / `completed_at` 2026-09-10T18:19:29Z → **759 s**. Same job, step `Build gunbc from the public seed`: 2026-09-10T18:07:04Z–18:10:06Z → **182 s**.
- Timeout-cancel class: run [34523487941](https://github.com/gunb-ai/gunbc-private/actions/runs/34523487941) `conclusion=cancelled`, `startedAt` 2026-09-10T19:57:52Z / `updatedAt` 2026-09-10T20:58:48Z → **~61 min**. Several siblings the same day have the same shape. That is the bar "near timeout", independent of whether the current workflow row says 60 or 90.
- Local-cold lower bound: BuildBuddy [1f689c7c-cb5f-4de9-b25e-c7d45a7f8a84](https://app.buildbuddy.io/invocation/1f689c7c-cb5f-4de9-b25e-c7d45a7f8a84) `elapsed_sec=238`.

Subtraction is `post_job − warm_seed + candidate_seed` against those two job walls:

| Seed-build cost | Implied whole-job (759 − 182 + seed) | vs 3341 s | vs ~61 min cancel |
|---|---|---|---|
| Warm 182 s (run 34512318040 build step) | 759 s | crushing win | not racing |
| Lower-bound local-cold 238 s (BuildBuddy invocation) | 815 s | crushing win | not racing |
| 2× that lower bound (476 s seed) | 1053 s | crushing win | not racing |

**Accepted-cold clears the bar.** Every row is a crushing win over 3341 s and none is near the ~61-minute cancel class. Guest egress and a prebuilt image are therefore **not on the critical path**. Carrying three arms forward after this subtraction would be redundant work.

The 3m 58s figure is a **lower bound, twice over**, not an estimate: (1) `CARGO_HOME` was not wiped, and private wipes `RUNNER_TEMP/cargo`; (2) amd64 BuildBuddy is not the Aarch64 fleet host — different ISA, different dependency set, different registry cache. The number can only be wrong *up*. Twice the lower bound still clears the bar, which is why accepted-cold is the working assumption rather than a guess that 4m will hold on arm64.

**What would make it a conclusion rather than a working assumption:** the same command on an **Aarch64 fleet host with `CARGO_HOME` wiped**. Unobtainable in this session: `ctrl-build --remote` is linux/amd64; a session-local release compile is forbidden (sessions.slice OOM, swap off); the Aarch64 fleet slots *are* the CI runners and are not a disposable probe; Mt. Collins guest exec is still `HostShellAndBootImageAccessRefused`. That measurement is the **trigger** that could reopen the cache question as a gate. Until it lands, accepted-cold stands.

**Sequence (shared capability first, then workflows):**

1. Placeability wet probe + host image from Y — **one host path**, both repos consume it. Accepted-cold is the seed-build assumption; do not wait on guest sccache.
2. **First workflow consumer: private**, attempt-unique label in `self_hosted_labels(…, custom: …)`. Not M1 dogfood.
3. **Then public**, one lane at a time. **That** start is `dogfood-started` and the two-week clock.

Capacity: one Mt. Collins canary is not two 90-minute public lanes plus private. Public cutover needs either serialized lanes, a second canary host, or an honest declared drop of parallelism.

**Coldness is a property of the cache's location, not the VM's lifetime.** That distinction still matters as an *optimization* (a reachable sccache still saves a few minutes). It is not a cutover gate while accepted-cold clears the job bar.

The three arms, after the subtraction:

| Arm | Standing |
|---|---|
| **Accepted local-cold** | **Working assumption for cutover.** Clears the 3341 s job wall (run 34462653642) and the ~61-minute cancel class (run 34523487941). Trigger that could demote it: Aarch64 + wiped `CARGO_HOME` seed build that pushes the *whole job* near that cancel class. Does not need guest egress. Does not silence unpinned public main. |
| **External shared cache** | **Blocked, off the critical path.** `gunbc.runner.runner_guest_egress_attempt` `egress_outcome` is `GuestEgressNotEstablished`. `gunbc.runner.runner_host_filtered_egress` still withholds `DefaultDenyEgress` from the Mt. Collins offer until a filtered-egress acceptance receipt. That is a **capability frontier this lane does not own**. A reader must not discover the block by trying to point the guest at srvN sccache. Reopens only if the Aarch64 cold job fails the 60-minute bar. |
| **Prebuilt in the image** | **Rejected for private.** Private builds from unpinned public main so a public authority change goes loud. A binary in the image silences that. Not required while accepted-cold holds. |

Named lower-bound dispatch (not a guest): `ctrl-build --remote -- bash -lc '… unset RUSTC_WRAPPER; CARGO_TARGET_DIR=$(mktemp -d); /opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc'`. Producer: BuildBuddy invocation [1f689c7c-cb5f-4de9-b25e-c7d45a7f8a84](https://app.buildbuddy.io/invocation/1f689c7c-cb5f-4de9-b25e-c7d45a7f8a84). Cargo `Finished release … in 3m 58s`; `elapsed_sec=238`; `rustc_wrapper_at_start=UNSET`.

**Complementary, not subsuming.** #46 stops the 3341 s job (run 34462653642) racing the cancel class of run 34523487941. MicroVMs stop the deleter class. Together they do not wash: even 1053 s after #46 is still a crushing win over 3341 s.

## 4. What is lost, and the staged carve-out

**X (authority that must end in one motion):** the shared-filesystem self-hosted slot — persistent `jit-runner.sh` on srvN, jobs sharing a host kernel and a toolchain tree that another job can `unlink`. Also the recovered host-boot script once Y is PID1.

**Y:** per-job guest from `runner_attempt_launch` executed at host boot, image built from Y, JIT minted only after grant re-read.

**Gap-intolerant boundary:** CI must keep working. This is exactly #10927's staged carve-out: Y in shadow, **one** transition that switches the root and deletes X together. No "use Y when available". No dual `runs-on`.

**Lost on transition, and how it is covered:**

- **Shared toolchain / cargo caches across jobs on one host.** Isolation makes the deleter class unwritable. Lost *local* reuse is accepted under the working assumption: 815–1053 s still clears 3341 s and the ~61-minute cancel class. External sccache is an optimization blocked on `GuestEgressNotEstablished`, not cover for the cut.
- **`ci_isolate_toolchain_script` / start-end filesystem probe.** Retired when microVMs make the eviction class impossible — that question belongs to **still-bear-335**, not to the process-count PR.
- **Persistent runner registration.** Replaced by JIT per attempt. Covered only when mint HTTP + jail staging frontiers bind (`jit_mint_http_realization_frontier`, `jail_jit_device_staging_frontier`).
- **Slot RAM carve / `CARGO_BUILD_JOBS` from `ci_runner_target_memory_regime`.** A guest size is a different envelope (`gunbc.runner_microvm` size decision). The flip must project through `selected_ci_runner_target`, not a literal in YAML.
- **Canary-only capacity.** Covered by refusing to cut public until capacity is named, or by a declared §4b drop of dual-lane parallelism with a restoration trigger = a second canary host or serialized lanes that still meet the floor.

X may remain as an offline oracle (srv slots keep running **until** the switch). Y must not resolve through X.

## Adjacent lanes — what this cure does and does not retire

Three lanes, three questions. An earlier revision of this plan routed the filesystem-instrument dissolution to warm-badger-62. That was wrong.

| Lane | Question | Relation to this plan |
|---|---|---|
| **still-bear-335** (`adhoc-3b2f737a-7b8`, gunbc#10985) | Typed env-vs-code; public filesystem probe | **Mitigation.** Probe dissolves only on this plan's two conjuncts (their rewrite of `toolchain_filesystem_probe_dissolution_condition`). Typed BMC/host/JIT vs code stays. Private probe copy is follow-on STEPS with the same bind rule. They do not edit `strategy.private_witness_workflow`. |
| **warm-badger-62** (private #46) | Process count: stop re-acquiring the composed world 34 times | **DESIGN §2.** MicroVMs do not replace it. After #46 the seed build dominates; whole-job arithmetic against runs 34462653642 and 34512318040 says even a cold seed still crushes the pre-#46 job wall. Do not edit `strategy.private_witness_workflow`. Per-attempt label: `self_hosted_labels(…, custom: …)`. |
| This plan | Per-job guest; FCI-3 before jailer; host image from Y | **Cure of the shared-FS eviction class.** Does not shorten the 3341 s roster-dominated job. Does not classify failures. |

**Retirement of the probe instrument (home: #10985's `toolchain_filesystem_probe_dissolution_condition`, bind when both hold):**

1. Public required jobs: `selected_ci_runner_target` / `gunbc_ci_selected_runner_spec` is a per-attempt microVM, not the empty-custom fleet slot.
2. Private required jobs: `witnesses_job()` `SelfHosted` `custom` labels, same, not empty-custom.
3. That selection makes shared-FS eviction impossible, so the row is eligible to bind. Identifying a deleter on the shared slot is not a substitute for (1)–(2).

**Not retired by this cure:** private #46's single-process roster; the timeout-cancel class (run 34523487941); typed env-vs-code classification.

## Trigger state (this document's standing)

| Trigger | State |
|---|---|
| `host_boot_cutover_frontier` | Unbound |
| `host_image_placeability_wet_probe_frontier` | Unbound; BMC HTTPS 200; standing still access-refused |
| `jit_mint_http_realization_frontier` | Unbound; #10923 merged; named performer module still absent |
| `jail_jit_device_staging_frontier` | Unbound |
| `toolchain_filesystem_probe_dissolution_condition` | Unbound. Proposed home of the two-conjunct bind is gunbc#10985; identifying a deleter on the shared slot is not a substitute. |
| `dogfood-started` | Not started (public CI still on srv slots) |
| This plan's cutover | **Not taken** — operator decision |
| Seed-build cache as a cutover gate | **Retired as a gate.** Working assumption: accepted-cold. Trigger to reopen: Aarch64 + wiped `CARGO_HOME` whole-job near the cancel class of run 34523487941. |
| Local-cold `cargo build --release -p v1-compiler --bin gunbc` (empty target dir, no `RUSTC_WRAPPER`) | **3m 58s lower bound** (amd64, `CARGO_HOME` not wiped), invocation `1f689c7c-cb5f-4de9-b25e-c7d45a7f8a84`. |
| Guest reachability of fleet sccache | **Blocked** on `GuestEgressNotEstablished` / withheld `DefaultDenyEgress`. Off this plan's critical path while accepted-cold holds. Not this lane's frontier. |

## What this document deliberately does not do

It does not change `selected_ci_runner_target`, private `witnesses_job()` labels, fleet-converge, or recovered init. It does not exec jailer. It does not land guest egress. It does not archive still-bear-335 or private #46. Those are the cutover, which this task forbids.
