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
| Instrument | Already has toolchain filesystem probe | Mitigation lane (`warm-badger-62`, `strategy.private_witness_workflow`, private #46) — do not fork that authority here |
| M1 | **`dogfood-started` is this repo's CI** | Not the year-end exit |

**Sequence (shared capability first, then workflows):**

1. Placeability wet probe + host image from Y — **one host path**, both repos consume it.
2. **First workflow consumer: private**, one job with an **attempt-unique** `runs-on` label (the binding #10927 / bound-attempt receipts already named as missing). Reasons: smaller; already red on the deleter class; a failed cutover does not block public merges; the motivating incident is here. This is **not** M1 dogfood.
3. **Then public**, one lane at a time, still shadow-labelled until the operator cutover: flip `selected_ci_runner_target` / fleet labels only at the one transition. **That** start is `dogfood-started` and the two-week clock.

Instinct (private first) is right for **cure of today's red** and for **not taking public merge-admission down**. It is wrong if read as "private *is* M1". State both.

Capacity: one Mt. Collins canary is not two 90-minute public lanes plus private. Public cutover needs either serialized lanes, a second canary host, or an honest declared drop of parallelism. That is a cutover-decision input, not something this plan silently assumes.

## 4. What is lost, and the staged carve-out

**X (authority that must end in one motion):** the shared-filesystem self-hosted slot — persistent `jit-runner.sh` on srvN, jobs sharing a host kernel and a toolchain tree that another job can `unlink`. Also the recovered host-boot script once Y is PID1.

**Y:** per-job guest from `runner_attempt_launch` executed at host boot, image built from Y, JIT minted only after grant re-read.

**Gap-intolerant boundary:** CI must keep working. This is exactly #10927's staged carve-out: Y in shadow, **one** transition that switches the root and deletes X together. No "use Y when available". No dual `runs-on`.

**Lost on transition, and how it is covered:**

- **Shared toolchain / cargo caches across jobs on one host.** Covered by construction: each guest starts clean. Cost goes up; deleter class becomes unwritable. That is the point of `toolchain_filesystem_probe_dissolution_condition`.
- **`ci_isolate_toolchain_script` / start-end filesystem probe.** Retired when microVMs make the eviction class impossible — tell the mitigation lane (below).
- **Persistent runner registration.** Replaced by JIT per attempt. Covered only when mint HTTP + jail staging frontiers bind (`jit_mint_http_realization_frontier`, `jail_jit_device_staging_frontier`).
- **Slot RAM carve / `CARGO_BUILD_JOBS` from `ci_runner_target_memory_regime`.** A guest size is a different envelope (`gunbc.runner_microvm` size decision). The flip must project through `selected_ci_runner_target`, not a literal in YAML.
- **Canary-only capacity.** Covered by refusing to cut public until capacity is named, or by a declared §4b drop of dual-lane parallelism with a restoration trigger = a second canary host or serialized lanes that still meet the floor.

X may remain as an offline oracle (srv slots keep running **until** the switch). Y must not resolve through X.

## Mitigation lane — what retires their work

`warm-badger-62` owns `strategy.private_witness_workflow` and private #46. That lane types environmental CI failure and gives private the filesystem instrument public already has. **Mitigation.** This lane is **cure**.

**Retirement signal for the mitigation (delete when true, not before):**

1. Private and public required jobs `runs-on` a per-attempt microVM label, not `[self-hosted, linux, arm64]` fleet slots.
2. `toolchain_filesystem_probe_dissolution_condition` is eligible to bind on the microVM arm (shared-filesystem eviction class impossible).
3. Their typed environmental outcome may remain as a **classification** of BMC/host/JIT infrastructure failure — that is FCI-adjacent and is not dissolved by isolation. Only the **shared-FS deleter instrument** is dissolved.

Do not edit `strategy.private_witness_workflow` from this lane.

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

## What this document deliberately does not do

It does not change `selected_ci_runner_target`, private `runs-on`, fleet-converge, or recovered init. It does not exec jailer. It does not archive the mitigation lane. Those are the cutover, which this task forbids.
