# CI onto per-job microVMs — plan, not a cutover

*Subject: get gunbc and gunbc-private CI onto per-job Firecracker guests. This document does not switch `runs-on`, does not bind `host_boot_cutover_frontier`, and does not start the M1 dogfood clock. It answers the four questions the operator asked, with named instruments. Cutover is a later operator decision.*

*Governed by DESIGN §3 replacement-migration (gap-intolerant staged carve-out). Continues [microvm-launch-displacement-analysis](microvm-launch-displacement-analysis.md) and gunbc#10927 (`gunbc.runner_attempt_launch`).*

## Why this is already a plan item (say this first)

`strategy.year_end_plan` (gunbc-private) already carries:

- **`runner-canary`** — `InProgress`, M1. Exit: GitHub App + `workflow_job` webhook → JIT ephemeral runner in a **per-job microVM** on the canary host → one real job completes → metered receipt → environment destroyed. One real job has already completed on Mt. Collins (gunbc#10733). Three conjuncts remain (authenticated JIT push, metered vCPU-minutes, teardown bound to the attempt identity).
- **`dogfood-started`** — M1. Exit: **gunbc's own CI runs on the canary host through the runner-canary path** — *the two-week observation clock STARTS*. Depends on `hw-first-host`, `boot-minimum`, `fabric-m0`, `runner-canary`.

This is not an infrastructure detour competing with the year-end plan. **Public CI on our own microVM runners is the M1 dogfood deliverable.** Until that exit is true, the two-week clock cannot start, and M1b `dogfood-window` cannot start.

The 2026-09-10 motive is independent and already declared. Public `gunbc.witness_floor_workflow` `toolchain_filesystem_probe_dissolution_condition` (emitted into `.github/workflows/witnesses.yml`) says to delete the start/end runner-filesystem instrument after the readings identify the deleter **or after per-job runner microVMs make the shared-filesystem eviction class impossible**. gunbc-private PR #50 went red today on "Build gunbc from the public seed" with exit 126 (~15s, command found but not executable). That is the same class. MicroVMs are the named cure; today's red is the trigger firing.

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

**Sequence (shared capability first, then workflows):**

1. **Measure cold vs warm for `cargo build --release -p v1-compiler --bin gunbc`** (same command private uses) with the cache *inside* an empty target dir and `RUSTC_WRAPPER` unset — then decide. One dispatch. If cold ≈ warm, the rest of this section is noise.
2. Placeability wet probe + host image from Y — **one host path**, both repos consume it.
3. **First workflow consumer: private**, attempt-unique label in `self_hosted_labels(…, custom: …)`, **only after** the cache-location question below has a number. Reasons that survive: already red on the deleter class; failed cutover does not block public merges. This is **not** M1 dogfood.
4. **Then public**, one lane at a time. **That** start is `dogfood-started` and the two-week clock.

Capacity: one Mt. Collins canary is not two 90-minute public lanes plus private. Public cutover needs either serialized lanes, a second canary host, or an honest declared drop of parallelism.

**Coldness is a property of the cache's location, not the VM's lifetime.** A per-attempt guest starts with an empty *local* disk. If the build cache lives *inside* that filesystem, every job is a full cold compile, permanently, and the ~3m warm private seed build (run 34512318040: checkout+toolchain 12s, build 3m 02s, roster 9m 18s; before #46 the roster was 52.8 of 55.7 min) becomes whatever cold actually is. If the cache is **external** and reachable over the network, a fresh VM can be warm on its first rustc. This fleet already has that shape in public tooling: sccache and the ctrl-build remote path exist so a build does not depend on local disk. The question to measure is therefore **whether the cache is reachable from inside the guest, and whether it is warm across VM instances** — not "is the VM cold".

The three arms answer **different questions**. They are not three interchangeable product choices:

| Arm | Question it answers | Constraint already in the corpus |
|---|---|---|
| **External shared cache** | VM cold, cache warm; cost is a network fetch, not a compile | Guest must **egress** to the cache. `gunbc.runner.runner_guest_egress_attempt` records a guest that did **not** obtain egress (bridge up, no address, no route off the segment). `runner_filtered_egress_receipt` is the later filtered path. A jailed CI guest **may not be permitted** to reach srvN sccache. If it cannot, that is the finding, and this arm is closed until egress is a granted effect, not a hope. |
| **Prebuilt in the image** | No compile in the job; build moves to image-build time | Private builds from **unpinned public main** so a public authority change goes loud. A binary baked into the image **silences that signal**. Do not adopt this arm without noticing it defeats the reason the private workflow tracks main. A public-CI artifact of the *same SHA the job just checked out* is a different construction (identity join, not a stale image); it still needs artifact-return, which the compute contract names as missing. |
| **Accepted local-cold** | Isolation with no reused compiler state | Honest iff measured. If cold is 4m and warm is 3m, the whole optimization is noise. |

**Measure cold first, then decide.** Named dispatch (not a guest): `ctrl-build --remote -- bash -lc '… unset RUSTC_WRAPPER; CARGO_TARGET_DIR=$(mktemp -d); /opt/cargo/bin/cargo build --release -p v1-compiler --bin gunbc'`. Producer: BuildBuddy invocation [1f689c7c-cb5f-4de9-b25e-c7d45a7f8a84](https://app.buildbuddy.io/invocation/1f689c7c-cb5f-4de9-b25e-c7d45a7f8a84). Result: cargo `Finished release … in **3m 58s**`; wrapper `elapsed_sec=238`; `rustc_wrapper_at_start=UNSET`; `Compiling` from `proc-macro2` through `v1-compiler`.

What this number **is**: local-compile-cold on an amd64 remote builder with an empty *target* dir. What it **is not**: a Firecracker guest, arm64 (private CI's `Aarch64` labels), a wiped `CARGO_HOME` (private wipes `RUNNER_TEMP/cargo` each job, so it also pays registry fetch), or a proof that guest-to-sccache egress works.

Against the warm 3m 02s on a fleet slot (run 34512318040), the delta is about a minute on a *different* architecture. That is **not** yet "cold ≈ warm, question is noise" for the actual CI hosts — it **is** enough to retire the fear of a 20-minute seed compile on this builder class. The remaining open measurement is **guest reachability of fleet sccache** (`runner_guest_egress_attempt` still `GuestEgressNotEstablished`). Do not flip `runs-on` until that is answered *or* accepted-local-cold is chosen knowing the arm64 number is still unmeasured.

**Complementary, not subsuming.** Private `timeout-minutes: 60` and a 53-minute roster is why main sat broken: two runs after 2026-09-08 **cancelled at 60 minutes**. MicroVM isolation stops the deleter class; it does not shorten a 53-minute job. #46's process-count cut stops racing that timeout. Neither covers the other; together they can still wash if the seed build goes fully cold.

## 4. What is lost, and the staged carve-out

**X (authority that must end in one motion):** the shared-filesystem self-hosted slot — persistent `jit-runner.sh` on srvN, jobs sharing a host kernel and a toolchain tree that another job can `unlink`. Also the recovered host-boot script once Y is PID1.

**Y:** per-job guest from `runner_attempt_launch` executed at host boot, image built from Y, JIT minted only after grant re-read.

**Gap-intolerant boundary:** CI must keep working. This is exactly #10927's staged carve-out: Y in shadow, **one** transition that switches the root and deletes X together. No "use Y when available". No dual `runs-on`.

**Lost on transition, and how it is covered:**

- **Shared toolchain / cargo caches across jobs on one host.** Isolation makes the deleter class unwritable (`toolchain_filesystem_probe_dissolution_condition`). What is lost is *local* reuse, not necessarily *all* reuse — cache location, not VM lifetime (§3). Until a local-cold number exists, "each guest starts clean" is not a priced cover.
- **`ci_isolate_toolchain_script` / start-end filesystem probe.** Retired when microVMs make the eviction class impossible — that question belongs to **still-bear-335**, not to the process-count PR.
- **Persistent runner registration.** Replaced by JIT per attempt. Covered only when mint HTTP + jail staging frontiers bind (`jit_mint_http_realization_frontier`, `jail_jit_device_staging_frontier`).
- **Slot RAM carve / `CARGO_BUILD_JOBS` from `ci_runner_target_memory_regime`.** A guest size is a different envelope (`gunbc.runner_microvm` size decision). The flip must project through `selected_ci_runner_target`, not a literal in YAML.
- **Canary-only capacity.** Covered by refusing to cut public until capacity is named, or by a declared §4b drop of dual-lane parallelism with a restoration trigger = a second canary host or serialized lanes that still meet the floor.

X may remain as an offline oracle (srv slots keep running **until** the switch). Y must not resolve through X.

## Adjacent lanes — what this cure does and does not retire

Three lanes, three questions. An earlier revision of this plan routed the filesystem-instrument dissolution to warm-badger-62. That was wrong.

| Lane | Question | Relation to this plan |
|---|---|---|
| **still-bear-335** (`adhoc-3b2f737a-7b8`) | Typed env-vs-code outcome; private copy of the public filesystem instrument | **Mitigation of the deleter class.** Dissolution of that *instrument* is the microVM arm of `toolchain_filesystem_probe_dissolution_condition`. Typed classification of BMC/host/JIT failure is **not** dissolved by isolation. |
| **warm-badger-62** (private #46) | Process count: stop re-acquiring the composed world 34 times | **DESIGN §2.** MicroVMs do not replace it. After #46 the seed build dominates; a naive microVM flip without reachable cache can wash that win. Do not edit `strategy.private_witness_workflow`. Per-attempt label: `self_hosted_labels(…, custom: …)`. |
| This plan | Per-job guest; FCI-3 before jailer; host image from Y | **Cure of the shared-FS eviction class.** Does not shorten a 53-minute roster. Does not classify failures. |

**Retirement signal for still-bear-335's filesystem instrument (delete when true, not before):**

1. Private and public required jobs select a per-attempt microVM label (`custom` on private; `selected_ci_runner_target` on public), not the empty-custom fleet slot.
2. `toolchain_filesystem_probe_dissolution_condition` is eligible to bind on the microVM arm.

**Not retired by this cure:** private #46's single-process roster; `timeout-minutes: 60` racing a long roster; typed env-vs-code classification.

## Trigger state (this document's standing)

| Trigger | State |
|---|---|
| `host_boot_cutover_frontier` | Unbound |
| `host_image_placeability_wet_probe_frontier` | Unbound; BMC HTTPS 200; standing still access-refused |
| `jit_mint_http_realization_frontier` | Unbound; #10923 merged; named performer module still absent |
| `jail_jit_device_staging_frontier` | Unbound |
| `toolchain_filesystem_probe_dissolution_condition` | Unbound |
| `dogfood-started` | Not started (public CI still on srv slots) |
| This plan's cutover | **Not taken** — operator decision |
| Local-cold `cargo build --release -p v1-compiler --bin gunbc` (empty target dir, no `RUSTC_WRAPPER`) | **3m 58s** on amd64 BuildBuddy, invocation `1f689c7c-cb5f-4de9-b25e-c7d45a7f8a84`. Not a guest, not arm64, `CARGO_HOME` not wiped. |
| Guest reachability of fleet sccache | Open; `runner_guest_egress_attempt` is `GuestEgressNotEstablished` |

## What this document deliberately does not do

It does not change `selected_ci_runner_target`, private `witnesses_job()` labels, fleet-converge, or recovered init. It does not exec jailer. It does not archive still-bear-335 or private #46. Those are the cutover, which this task forbids. The seed-build dispatch above does **not** establish guest-to-sccache reachability: BuildBuddy already has an external cache path by design; this run unset `RUSTC_WRAPPER` and still used that runner's `CARGO_HOME`.
