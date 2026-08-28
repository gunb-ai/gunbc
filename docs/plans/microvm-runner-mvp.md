# Per-registration Firecracker microVMs for CI runners — MVP, measured

Status: **MVP proven by execution on srv4, 2026-08-28.** Nothing here is deployed, bound, or
reachable from any workflow. Operator direction: move all runners onto Firecracker.

## 1. Why, in one sentence

A runner slot's `Runner.Listener` can be orphaned by a restart and then delete the *live*
incarnation's credentials out of the shared slot directory, killing whatever job is running at its
next token refresh. A microVM per registration has no shared slot directory and no surviving
listener, so the defect has nowhere to live.

Root chain, confirmed at pid grain twice (see §7): reconciler declares a slot wedged → `systemctl
restart` → `KillMode=process` leaves the listener alive → orphan later runs *its* cleanup in the
shared directory → live job dies at its next re-auth.

## 2. What was proven, by execution

On srv4, `/var/tmp/fcmvp`, touching no CI configuration:

| fact | evidence |
|---|---|
| Firecracker v1.16.1 aarch64 runs on the host | `./firecracker --version` |
| Ubuntu 24.04.4 LTS guest boots | serial console reaches `ubuntu login:` |
| guest networking | tap `fcmvp0`, guest `172.16.0.2/30`, NAT via `enP3p3s0f1`; host→guest ping 2/2, 0.297 ms avg |
| actions-runner 2.337.0 registers **from inside the VM** | `√ Connected to GitHub` / `Listening for Jobs` |
| **boot → listening: 5.3 s** | `[2.806] starting single-job runner` → `[5.334] Listening for Jobs` |

The JIT config is minted on the host from the org GitHub App and handed to the guest on a second
block device (`/dev/vdb`); a one-shot systemd unit runs exactly one job and powers the VM off.

**Not yet proven:** a workflow job executed end-to-end, clean poweroff after it, and the floor's
working set inside a VM. Those are the next three measurements, not claims.

## 3. Host facts the design rests on

Measured 2026-08-28, read-only:

```
srv1  502 GiB RAM   362 avail   1.1T free   /dev/kvm  128 cores
srv4  502 GiB RAM   406 avail   215G free   /dev/kvm  128 cores
srv2  125 GiB RAM    79 avail               /dev/kvm  128 cores
srv3  UNVERIFIED — no SSH key; srv4→srv3 fails host-key verification
```

Kernel 6.8.0-138-generic. MIDR `0x413fd0c1` = **ARM Neoverse N1**, which Firecracker is
continuously tested on. **Zero tap devices on any host**; the only bridge is `docker0`. VM
networking is greenfield.

## 4. The shape: one VM per REGISTRATION, not per job body

`actions-runner@.service` already carries `ExecStart=/opt/actions-runner/jit-runner.sh`, and
`gunbc.runner_unit_file` **already renders that unit** (#9401). The change is to point `ExecStart`
at a wrapper that boots a microVM running the runner inside. Unit-per-slot, JIT registration,
labels, and cgroup caps are all preserved; one derived line changes.

**Rejected: adopting an orchestrator** (e.g. Hostinger fireactions, v2.0.6, actively maintained).
It would replace `jit-runner.sh`, the unit-per-slot model, and the JIT registration this repo
already models in `gunbc.runner_unit` — a second path beside an existing modeled route, which is
DESIGN §6's parallel-authority tell. We would delete our authority over five facts to buy pools we
did not ask for.

**Rejected as insufficient: containerizing the job body.**
`ctrl scripts/session-dashboard/host/ROOTLESS_DOCKER_CI_PLAN.md` (design-complete, awaiting
sign-off since 2026-06-10) isolates the *job body* and leaves the runner on the host. **It could not
have prevented either specimen in §7**, because the victim and the killer are both `Runner.Listener`
processes living outside the job body. This is a finding against that plan, not a note in this one:
it currently awaits sign-off as a fix for something it does not fix.

## 5. PREREQUISITE: #9401 must be deployed first, or the orphan just gets heavier

Under `KillMode=process` the unit's tracked child becomes the VMM. An orphaned VMM keeps its guest
alive with the same credentials in the same directory — the identical failure with a larger process
holding it. "Everything inside the VM dies together" is true and does not help if the VM is what
survives.

`gunbc.runner_unit` already carries the correct predicate:

```
lifecycle_intent_cannot_orphan(intent, listener_is_main_process) =
  kill_mode reaches whole cgroup AND (exit_type == Cgroup OR listener_is_main_process)
```

and the measurement that decides it: `jit-runner.sh` execs into GitHub's stock `run.sh`, which does
**not** exec, so the listener is a *grandchild* of MainPID. A VM wrapper has the same topology.
So the microVM design **depends on** `ExitType=cgroup` + `KillMode=control-group` (#9401, merged,
undeployed) rather than substituting for it.

## 6. What is genuinely unsolved

- **Cache strategy.** sccache over a host UDS and the shared `ctrl-jobserver` FIFO both assume a
  shared host. Per-VM isolation is exactly what breaks that assumption. Flagging early rather than
  discovering it at the end.
- **Memory sizing.** The floor's peak is a **single** observation (15.0 GiB, srv4-18). Sizing a hard
  16 GiB VM off one sample is 1.07× headroom, and a run peaking at 15.5 dies as an infrastructure
  failure that reads as a job failure. The peak *distribution* is required before any per-VM size is
  chosen — and if the honest size makes the current slot count not fit, **that is the finding**, not
  a number to round down to.
- **The cell resource boundary relocates.** Per-slot systemd slice caps currently bound the job on
  the host; under microVMs that boundary moves into the guest. A host slice capping a VMM caps a
  different thing than a slice capping a job. Naming it so it does not become two authorities for
  one fact.
- **srv3 is unmeasured** and is the discriminating host — it straddles the roster page boundary
  (58 of 149 registrations on page 1).

## 7. The defect this replaces, measured

n=2, pid grain. Discriminating test: a credentials removal by a pid **other** than the one running
the job, *during* the job. Same-pid-after-completion is lawful ephemeral teardown and appears on
every healthy run, which is why the log string alone proves nothing.

```
srv4-18 (job 98644348200)          srv4-04 (run 33130742525)
 19:24:04 WEDGED → restarted        00:35:24 WEDGED → restarted
 job pid 4054700 @ 19:25:48         job pid 1469326 @ 00:45:47
 pid 4024080 removes creds 19:29:24 pid 1425194 removes creds 00:56:52
 noticed 20:14:55  (45.5 min)       noticed 01:26:28  (29.6 min)
 cancelled 20:15:11                 cancelled 01:26:46
```

Latency is the time to the *next* token refresh, not a fixed interval — so the predicate contains
**no duration term**: a job dies iff an orphan deletes its credentials during the job AND a refresh
occurs before the job ends.

**Bound, stated rather than smoothed:** this does not explain the 385 `Canceled` events on srv4
since 2026-08-25, and srv4-08's 54-minute cancellation shows no orphan removal at all and remains
unexplained. Duration and job type are confounded in every specimen produced.

**Standing population** (2026-08-27T21:29Z): 522 org registrations against ~67 live slots, 447
offline. srv4 holds 331 and has **zero** rows on the reconciler's single unpaginated page, so every
srv4 slot reads `registration=absent` on every tick.

## 8. Migration ordering

1. **Deploy route first.** Nothing reaches a host while `deployed_tree_repository_transition` is
   `LegacyGitFileSync` and `apply` terminates that refusal in `exit_failure`. #9506 is merged but
   unbound with five declared prerequisites. Design against this as a constraint; a plan premised on
   the binding flip landing is premised on someone else's schedule.
2. **#9401 deployed** (§5), or the orphan becomes the VMM.
3. **Rootfs + kernel as a modeled artifact.** This *is* the dependencies-from-scratch program, not a
   sequel to it.
4. **srv1 and srv4 first** — 502 GiB each, so per-VM sizing has real slack. srv2 (125 GiB) runs 5
   slots today and is sized last. srv3 pending measurement.
5. **Delete `runner-liveness-reconcile.sh`.** Per-registration VMs have no long-lived listener to
   wedge, so the reconciler loses its subject. This retires an unversioned host script, its
   unpaginated roster read, and the stale-registration loop — a deletion, not a port.

## 9. Reproduction record

The MVP was produced by three scripts executed over SSH into `/var/tmp/fcmvp` on srv4. They are
recorded here **as a record, not as artifacts**: the 2026-08-24 operator ruling makes an ad-hoc
`.sh` in this repository §6 unmodeled realization, and the tree currently contains zero `.sh`
outside `.githooks`. The terminal form is a `.dag` authority rendering these the way
`gunbc.runner_unit_file` renders the unit; see the dissolution obligation in §10.

<details><summary>rootfs build (Ubuntu 24.04 noble arm64 → ext4)</summary>

```bash
truncate -s 8G rootfs.ext4 && mkfs.ext4 -q -F rootfs.ext4
mount -o loop rootfs.ext4 mnt
tar -xJf noble-server-cloudimg-arm64-root.tar.xz -C mnt
touch mnt/etc/cloud/cloud-init.disabled
rm -f mnt/etc/netplan/*.yaml
cat > mnt/etc/systemd/network/10-eth0.network <<'NET'
[Match]
Name=eth0
[Network]
Address=172.16.0.2/30
Gateway=172.16.0.1
DNS=1.1.1.1
NET
chroot mnt systemctl enable systemd-networkd
umount mnt
```
</details>

<details><summary>runner bake + one-shot unit</summary>

```bash
mount -o loop rootfs.ext4 mnt
mkdir -p mnt/opt/runner && tar xzf actions-runner-linux-arm64-2.337.0.tar.gz -C mnt/opt/runner
mount --bind /dev mnt/dev; mount -t proc proc mnt/proc; mount -t sysfs sys mnt/sys
chroot mnt apt-get install -y libicu74 liblttng-ust1 libkrb5-3 zlib1g curl ca-certificates git
chroot mnt useradd -m -s /bin/bash runner && chroot mnt chown -R runner:runner /opt/runner

# /usr/local/bin/runner-oneshot — one job, then power off
JIT=$(tr -d '\0\n' < /dev/vdb)
[ -n "$JIT" ] || { echo "EMPTY JIT CONFIG on /dev/vdb" >&2; poweroff -f; }
cd /opt/runner && runuser -u runner -- ./run.sh --jitconfig "$JIT"
poweroff -f
```
</details>

<details><summary>JIT mint + boot</summary>

```bash
# App JWT → installation token → org JIT config (same path jit-runner.sh uses today)
curl -X POST -H "Authorization: Bearer $INSTALL_TOKEN" \
  https://api.github.com/orgs/gunb-ai/actions/runners/generate-jitconfig \
  -d '{"name":"...","runner_group_id":1,"labels":["self-hosted","linux","arm64","fcmvp"],"work_folder":"_work"}' \
  | jq -r .encoded_jit_config > jitconfig.raw
truncate -s 64K jitconfig.raw          # second block device, read-only in guest

firecracker --api-sock fc.sock --config-file vmconfig.json
#   drives: rootfs.ext4 (vda, rw), jitconfig.raw (vdb, ro)
#   net:    tap fcmvp0, guest_mac AA:FC:00:00:00:01
#   machine: 4 vcpu, 4096 MiB
```
</details>

**Host state created on srv4 and not yet removed:** tap device `fcmvp0`, one iptables MASQUERADE
rule and two FORWARD rules scoped to `172.16.0.0/30`, and `/var/tmp/fcmvp`. Declared here because
host state no repository authority produced is exactly the class this fleet keeps losing track of.

## 10. Scaffold declaration

Everything in §9 is temporary by construction. **Dissolution obligation:** a `.dag` authority that
renders the rootfs build, the guest unit, and the boot configuration, consumed by the same
`gunbc.runner_unit` lifecycle that already renders `actions-runner@.service`. This document and its
reproduction record delete when that lands.

Landed under explicit operator direction (session chat, 2026-08-28): *"aim to get all of our runners
onto firecracker ASAP"* and *"push it to a separate PR"*. Recorded per DESIGN §5 — a dissolution
condition describes how debt ends and does not authorize creating it; the authorization is the
operator's, not the author's.
