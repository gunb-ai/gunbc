# srv-host instance slice — step receipts (2026-10-01)

Work item: srv-host instance slice create → connect → meter → destroy → cleanup on srv3/srv4
(strategy.year_end_plan instance-delivery-vertical exit). The deliverable is executed proof or a
located gap. **No step executed.** Each step below is a located blocker. Nothing was hand-built
around the real controllers, because the brief forbids a substitute.

## Host reach (executed)

| Host | Probe | Result |
|---|---|---|
| srv3 | ssh with the session fleet key, as `node` and as `briansrls` | **refused** (`Permission denied (publickey,password)`) — no route from this session |
| srv4 | ssh as `briansrls` with the fleet-automation key | reached: aarch64, 128 cpu, 502 GiB, `/dev/kvm` present, sudo -n works, `/var/lib/gunbc/fleet-converge` present; **no firecracker / cloud-hypervisor / qemu / virsh on PATH** |

## Step receipts

Controllers are the in-flight source on `codex/workspace-commissioning-production` (gunbc#12691),
which stacks on #12505, #12465, #12513 and #12638. All symbols below are cited on that branch.

| Step | Real controller | Status | Blocker |
|---|---|---|---|
| create | `gunbc.workspace_commissioning_dispatch` `workspace_dispatch_plan_cli` / `_stage_cli` / `_complete_cli`, then `runner.runner_microvm_slot_controller` `run_workspace_slot` and `runner.runner_microvm_lifecycle_realize` `launch_attempt`; image digest from `runner.workspace_guest_image` `verify_workspace_image` | missing (for srv3/srv4) | The slot controller admits only the srv1 slot (`srv1-13`). No srv3/srv4 slot identity, runtime install or capacity budget is modeled, and #12691's own plan says commissioning has not been applied anywhere. |
| connect (customer-held credential) | `runner.workspace_microvm_access` `workspace_connection_text` (prints an `ssh -J` line after a guest keyscan) | **missing** | Nothing installs a customer-held public key into the guest. The only `authorized_keys` writer is `fleet_host_key_enrollment`, and it writes fleet-automation keys. Guest login user is unmodeled. |
| workload | — | blocked | Blocked by create and connect. |
| meter | `gunbc.workspace_allocation_metrics` `workspace_capture_runtime_metrics` | partial | Collects raw systemd counters only. No join to `workspace_money_codec` or to any billing ledger. |
| destroy | `runner.runner_microvm_lifecycle_realize` `run_teardown` / `release_retained_unit`; `gunbc.workspace_allocation_release` `workspace_release_prepare` | not traced live | Never executed against a host. |
| cleanup readback | `runner.runner_microvm_lifecycle` `teardown_observations`; `runner.runner_microvm_lifecycle_realize` `observe_resource_absence` / `observe_slot_network_after_stop`; `gunbc.workspace_allocation_release` `workspace_release_read` | not traced live | Qualified only against local snapshots. No privileged host run. |

"Missing" means no producer exists in any branch. "Not traced live" means a producer exists in
source and has never run on a host.

## What unblocks the slice (in dependency order)

1. #12691 reaches its own live acceptance on srv1 (reservation, boot, SSH, release, same-slot reuse). That lane owns this; this slice consumes it and does not fork it.
2. A srv3/srv4 workspace target is modeled as fleet facts: slot identity, Firecracker/jailer install as a converge row, and a capacity budget. srv3 also needs a reach grant for sessions.
3. A customer-key provisioning step is added to the guest image or slot boot. This is a privileged-access concern for `gunbc.auth.authorization_pattern_selection` (DESIGN §3b).
4. The metering-to-money join is built.

Disposable storage only; no durability claim is made or implied.
