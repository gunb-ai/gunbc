# srv3 OS install actuation — operator runbook (prefix:os-install-actuated)

Operator-gated procedure to actuate the solve-driven NBD-proxy virtual-media install on srv3.
**Scope:** actuator prep + this runbook only — **NOT** srv3 OS install solved until seeded-ISO unattended boot is proven by execution.
Modeled seed-delivery gap (`gunbc.srv3_os_install_actuate_scope`) stays fail-closed on the **legacy solver plan**
(`gunbc.os_install.srv3_os_install_plan`, NoCloudNet); actuator prep binds **seeded delivery**
(`srv3_seeded_os_install_plan` + on-ISO `NoCloudLocal`) and the seeded ISO path authority. Unattended install is still
**NOT proven by execution** until boot-once + SOL receipt + OS-up witness green.
The **seeded-ISO pivot** (`gunbc.srv3_seeded_install_media`) embeds autoinstall user-data on media with GRUB `autoinstall ds=nocloud` (fully remote/CLI, no KVM). SOL console capture (`gunbc.srv3_sol_console_capture`) provides CLI observability during install.
Model authority: `gunbc.srv3_install_media_fetch`, `gunbc.srv3_seeded_install_media`, `gunbc.srv3_os_install_actuator_toolchain_ensure`, `gunbc.srv3_sol_console_capture`, `gunbc.srv3_os_install_actuate`,
`gunbc.srv3_bmc_credential_resolve`, `gunbc.srv3_os_install_actuate_workflow`, `gunbc.os_install_actuator_selection`, `gunbc.srv3_os_install_actuate_scope`,
`gunbc.nbd_proxy_virtual_media_install.srv3_os_install_actuator_plan`, `gunbc.srv3_boot_once_cd`.

**Precondition:** BMC Redfish reachable at **192.168.1.192** (HTTP 200). nbd-proxy ws-upgrade dry-run
confirmed **Present** on `/vm/0/0` (see [srv3 nbd-proxy ws-upgrade dry-run](srv3-nbd-proxy-ws-upgrade-dry-run.md)).
Actuator host is **derived** (`gunbc.os_install_actuator_selection.srv3_os_install_actuator_host` — today
srv1 via `OperatorHostPreference`); run serve/login commands from the selected actuator host or any host
with the toolchain grant (curl, websocat, nbdkit, socat).

## Hard constraints

1. **ISO must be fetched first** — run the modeled `srv3_install_media_fetch` on srv1 (step 0 below). Stock artifact path:
   `/var/lib/gunbc/artifacts/ubuntu-24.04.3-live-server-arm64.iso` (grounded on Ubuntu 24.04.3 point release + cited
   sha256 from [cdimage SHA256SUMS](https://cdimage.ubuntu.com/releases/24.04/release/SHA256SUMS)). Do not hand-fetch;
   the fetch receipt is the prereq read-back.
2. **Seeded ISO remaster before serve** — step 0b produces the actuator virtual-media path
   (`gunbc.srv3_os_install_actuate_scope.srv3_actuator_virtual_media_iso_install_path` →
   `.../ubuntu-24.04.3-live-server-srv3-seeded.iso`). `srv3_nbd_proxy_serve` binds this authority, not the stock path.
3. **`srv3_nbd_proxy_serve` is long-running** — runs foreground until Ctrl-C or installer disconnect. Open a second
   terminal for boot-once.
4. **Record receipts** — paste command output and router lease observation into the operator sign-off thread.

## Step 0a — actuator toolchain ensure (srv1)

websocat is **not** in Ubuntu Noble apt; the modeled ensure installs curl/nbdkit/socat via apt and websocat from the cited [vi/websocat v1.14.0](https://github.com/vi/websocat/releases/tag/v1.14.0) GitHub release binary to `/var/lib/gunbc/bin/websocat`. Add that directory to `PATH` for the serve session if `which websocat` does not find it after ensure:

```bash
export PATH="/var/lib/gunbc/bin:$PATH"

gunbc run --source-root dsl \
  --entry dsl/gunbc/srv3_os_install_actuator_toolchain_ensure.dag \
  --function srv3_os_install_actuator_toolchain_ensure
```

Expected stdout includes `OsInstallActuatorToolchainReceipt:` lines with `outcome=Present` or `outcome=Installed` for each tool.

## Step 0 — modeled ISO fetch (srv1 actuator host)

From the gunbc repo root on srv1:

```bash
GUNBC_ROOT="${GUNBC_ROOT:-$PWD}"
cd "$GUNBC_ROOT"

gunbc run --source-root dsl \
  --entry dsl/gunbc/srv3_install_media_fetch.dag \
  --function srv3_install_media_fetch
```

**Read-back (same path the fetch uses):**

```bash
test -f /var/lib/gunbc/artifacts/ubuntu-24.04.3-live-server-arm64.iso
sha256sum -c <<< '2ee2163c9b901ff5926400e80759088ff3b879982a3956c02100495b489fd555  /var/lib/gunbc/artifacts/ubuntu-24.04.3-live-server-arm64.iso'
```

Expected stdout includes `InstallMediaFetchReceipt:` with `outcome=AlreadyPresent` or `outcome=Fetched`, mirror used,
sha256, and bytes. On hash mismatch the modeled fetch **refuses** (no silent overwrite).

## Step 0b — seeded ISO remaster (srv1; xorriso ensure + remaster)

**Seeded path only** (fully remote/CLI unattended install). The remaster chain is fail-closed on xorriso: the modeled
`install_media_remaster_toolchain_ensure` verifies `xorriso --version` or runs `apt-get install -y xorriso` before
repack (srv1 was verified to lack xorriso/genisoimage/mkisofs — do not assume the toolchain is present).

```bash
gunbc run --source-root dsl \
  --entry dsl/gunbc/srv3_seeded_install_media.dag \
  --function srv3_seeded_install_media_toolchain_ensure

gunbc run --source-root dsl \
  --entry dsl/gunbc/srv3_seeded_install_media.dag \
  --function srv3_seeded_install_media_remaster
```

Expected stdout includes `InstallMediaRemasterToolchainReceipt: outcome=Present` or `outcome=Installed`, then
`InstallMediaRemasterReceipt:` with seeded ISO path
`/var/lib/gunbc/artifacts/ubuntu-24.04.3-live-server-srv3-seeded.iso`.

## Runnable prep (no BMC side effects except login)

**Run on srv1** (actuator host — `srv3_os_install_actuator_host`). BMC is always **192.168.1.192** (modeled `asrock_srv3_live_bmc_host`; not a host alias). srv2 is not used in this sequence.

From the gunbc repo root on srv1 (`cd` to your checkout — worktree or pinned tree):

```bash
GUNBC_ROOT="${GUNBC_ROOT:-$PWD}"   # export if not already in repo root
cd "$GUNBC_ROOT"

# Preflight: toolchain (ISO from step 0)
which curl websocat nbdkit socat jq sha256sum gunbc

# Emit the nbd-proxy serve script to /tmp/srv3_nbd_proxy_serve.sh (review before live)
gunbc run --source-root dsl \
  --entry dsl/gunbc/srv3_os_install_actuate.dag \
  --function srv3_os_install_actuate_emit_nbd_script
```

### BMCweb login — credential resolution (read before live)

**Modeled (`srv3_bmcweb_session_login`):** probes BMC with `bmc_probe_credential_phase`, resolves intent vs observation via `gunbc.srv3_bmc_credential_resolve`, then materializes password from factory login or `fetch_secret_ref_credential(bmc_srv3_admin_secret_ref)`. Password is written to **`/tmp/srv3_bmcweb_login_body.json`** (file on disk); curl posts it to `https://192.168.1.192/login`. On **success**, gunbc emits no password — only writes token to **`/tmp/srv3_bmcweb_token`**. On **failure**, stderr may include curl body — do not paste logs containing credentials.

Requires **gcloud auth** on the actuator host when BMC is in `RotatedCredentialActive` state.

```bash
gunbc run --source-root dsl \
  --entry dsl/gunbc/srv3_os_install_actuate.dag \
  --function srv3_bmcweb_session_login
```

## Live actuation sequence (operator-gated)

Run in **two terminals** on the actuator host.

### Terminal A — virtual media serve (held open)

```bash
gunbc run --source-root dsl \
  --entry dsl/gunbc/srv3_os_install_actuate.dag \
  --function srv3_nbd_proxy_serve
```

Leave running. The BMC should mount the ISO as virtual CD when the NBD handshake completes.

### Terminal B — boot once from CD + force restart (workflow escalation)

The modeled workflow (`gunbc.srv3_os_install_actuate_workflow`) places **`WorkflowOperatorApproval`** before **`WorkflowBootOnceCd`**. The default entrypoint `srv3_boot_once_cd` is **fail-closed** (`PendingApproval`). After explicit operator ack, use the granted entrypoint:

After serve is stable (give nbd-proxy ~10s to attach):

```bash
cd "$GUNBC_ROOT"
gunbc run --source-root dsl \
  --entry dsl/gunbc/srv3_boot_once_cd.dag \
  --function srv3_boot_once_cd_operator_approved
```

(`srv3_boot_once_cd` without approval refuses with `operator approval required before srv3-boot-once-cd`.)

**Boot-once semantics:** modeled `BootOverrideOnce` → Redfish wire **`"Once"`** (not `Continuous`). Applies to the **next boot only**; firmware clears the override after it is consumed. A failed install does **not** leave srv3 in a permanent CD-loop from Redfish — worst case is one retry if the host reboots back to virtual media before override clears. Post-success install reboots to disk (HDD/NVMe) and override is gone.

**Credentials:** same resolution path as login (`srv3_bmc_credential_resolve`); netrc written to `/tmp/bmc_onboard_netrc`.

Host should power-cycle and boot the Ubuntu installer from virtual media.

## Autoinstall timing and hang declaration

Modeled payload: `fully_automated: true`, hostname `srv3`, OpenSSH enabled. **Fail-closed seed gap (modeled, not solved):** `gunbc.srv3_os_install_actuate_scope` refuses seed delivery on the NbdProxy path — `NoCloudNet` seed at `install.srv3.lan/...` with `fleet_install_server_specs` empty. Stock ISO (`ubuntu-24.04.3-live-server-arm64.iso`) may boot **interactive** installer if autoinstall seed is not on-ISO — watch for subiquity waiting on network seed (hang risk). This PR does not paper the gap.

| Phase | Expected wall-clock (from boot-once) |
| ----- | ------------------------------------ |
| Virtual media attach + GRUB | 1–3 min |
| Installer/subiquity visible | 3–8 min |
| Autoinstall (if seed found) | 15–35 min |
| Post-install reboot | +3–5 min |
| Router DHCP option-12 `srv3` | within 2 min of reboot |

**Declare hang:** no installer UI progress for **15 min**; OR no router DHCP dynamic lease with hostname `srv3` **75 min** after boot-once. **Declare success (subsumption):** dynamic lease option-12 reads `srv3` (distinct from BMC static `bmc-altrad8ud-1l2t-3` @ .192).

## Post-install subsumption check (first network-identity witness)

After autoinstall completes and the host reboots, confirm srv3 OS appears on the LAN DHCP table with
option-12 hostname **srv3** (not merely the BMC static reservation `bmc-altrad8ud-1l2t-3` at .192).

Pre-install invariant (modeled): `srv3_pre_install_has_no_os_up_subsumption()` — only BMC static row.

Post-install target: dynamic lease row whose option-12 short hostname subsumes `operator_host_srv3`.

Router UI: Verizon CR1000A → Network → DHCP lease table. Record MAC, IPv4, and hostname column.

## Operator receipt template

```
srv3 os-install-actuated receipt
- date:
- operator:
- actuator host:
- iso path present: [yes | no]
- iso version + sha256: [e.g. 24.04.3 + checksum from SHA256SUMS]
- bmcweb login: [ok | fail]
- nbd serve held open: [yes | no]
- boot-once-cd: [ok | fail]
- router srv3 OS lease (option-12): [absent | srv3 | other]
- reading: [OsInstalled-pending | OsInstalled-confirmed]
```

## Related model files

- `dsl/gunbc/srv3_os_install_actuate.dag` — runnable prep + serve/login funcs
- `dsl/gunbc/srv3_boot_once_cd.dag` — Redfish boot-once CD + force restart
- `dsl/gunbc/network_identity_subsumption.dag` — DHCP option-12 subsumption checks
- `dsl/gunbc/host_standup.dag` — `prefix:os-install-actuated` spine gap
- `docs/plans/srv3-webui-kvm-virtual-media.md` — architecture B design
