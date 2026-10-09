# srv1 workspace VM bringup

Scope: the operator-owned workspace on `srv1-13`, using the existing commissioning,
allocation, controller and cleanup protocols. No customer create API, additional
hosts, CI-runner migration, or automatic expiry scheduling is part of this change.

## Observed gaps

The October 9, 2026 inspection found no successful workspace VM lifecycle receipt.
The September 24 controller invocation `c0fceefc7d9643b0aaff0617907b1335`
refused because it could not read the allocation store for `srv1-13`.
[Run 36054790165](https://github.com/gunb-ai/gunbc/actions/runs/36054790165)
collected that refusal; its green workflow conclusion is not a boot receipt.

| Boundary | Observed state | Required change |
| --- | --- | --- |
| Workflow dispatch | Main declares 28 inputs; GitHub accepts at most 25 | Keep printer selection separate and carry its five operation fields in one `printer_request` JSON input |
| Guest image | Builder exists; no image delivery in controller install | Build under fakeroot, archive kernel/rootfs/digest receipt, pin the measured rootfs, verify installed bytes and custody |
| Controller | Installed revision `cfb9ff80652fe0a74093f08cade64f59d9695266` | Install this PR through the existing controller installer |
| Slot budget | `fabric-cell-srv1-13.slice` has no installed unit file | Converge the modeled execution-cell boundary; a synthesized systemd slice is not proof |
| Commissioning | No commissioning or readiness record | Run reviewed commissioning plan/apply and verify its protected transaction |
| Protected state service | Running revision `5b96cab749088aa5cec71ce9ad52e1857724d252` predates the authenticated state route | Converge fabric storage through its existing deployment path |
| State-key custody | Key exists; operator can read it; `ghrunner` cannot | Preserve custody; exact installed helpers reconcile admitted intent and read the selected allocation |
| Gateway observation | Observer has no production caller; admission expires after 60 seconds | Bounded foreground operator refresh using the enrolled agent and host keys |
| Allocation dispatch | No workflow mode; release path came from an unrelated environment variable | Add `workspace_allocation_plan`, bind its installed release to the admitted run revision, reuse reviewed `apply` |
| Operator lifecycle | Request producer exists; release/observation need CLI entries | Add generation-checked release and owner-scoped JSON receipts under operator authority |

The historical refusal files are evidence. Do not delete them to manufacture an
empty commissioning prestate. Do not restore the old CI reservation service:
`srv1-13` is a workspace slot.

## Operation

1. Dispatch `fleet-converge`, `host=srv1`, `mode=workspace_image_build` at the
   candidate revision. The `workspace-guest-image` artifact contains only the
   kernel, rootfs and measured digest receipt. The filesystem build is not
   reproducible: pin that build's measured digest in `workspace_artifact_policy`.
2. Dispatch `microvm_controller_install` at the reviewed revision with
   `workspace_image_run_id` naming that build. The run ID locates bytes; it does
   not authorize them. The installer checks the reviewed kernel and rootfs pins
   after copying into the root-owned release directory, then independently reads
   back the image and installed executor.
3. Converge the execution-cell boundary and authenticated state service, then run
   `workspace_commissioning_plan` and its reviewed `apply` on srv1. The existing
   operator-local fleet plan/apply path is available for pre-merge validation;
   do not widen the main-only GCP login to run branch workflows.
4. Keep `workspace_gateway_refresh_operator_wet --arg seconds=3600` running in
   the operator context with the enrolled key selected in `SSH_AUTH_SOCK`. It
   refreshes forwarding evidence every 15 seconds and stops on refusal or at its
   deadline. It does not install a scheduler. As the enrolled operator on srv1, use `workspace_operator_request_wet` with
   `GUNBC_WORKSPACE_REQUEST_KEY`, `GUNBC_WORKSPACE_REQUEST_PROFILE` and
   `GUNBC_WORKSPACE_REQUEST_EXPIRES_AT`. Use a canonical UTC expiry within the
   policy's one-hour lease. The owner and public key come from installed policy.
5. Dispatch `workspace_allocation_plan` and review/apply its artifact. Observe
   through `workspace_operator_observe_wet` with the same key. Each operator
   command writes `target/workspace-operation.json`; a recorded request alone
   is not evidence that the guest is usable.
6. Authenticate SSH using the allocation's connection data and verified host key.
   Record the guest identity and a command executed in the guest.
7. Run `workspace_operator_release_wet` with the key and
   `GUNBC_WORKSPACE_RELEASE_GENERATION` from the observation. Reconcile with
   another allocation plan/apply, and verify released state and absent guest.
8. Request a second key, prove the same slot is reused with a new incarnation,
   and let its short lease expire. Explicitly dispatch allocation plan/apply to
   clean it up. No timer or scheduled expiry sweep is installed by this PR.

Workflow jobs receive no state MAC key and no grant to create or release an
operator request. Their installed helper can act only on already-admitted ledger
intent. Apply re-reads the exact request and generation carried in the reviewed
plan; it does not select the latest request again.

## Acceptance evidence

Keep the PR draft until all of the following have live receipts: image/controller
readback, commissioned slot, ready allocation, authenticated SSH, explicit release,
same-slot reuse with a new incarnation, expiry cleanup, and final released/ready
state. Record run IDs, revisions, allocation IDs and generations here as they are
observed. Never record private keys, tokens or signed state-request envelopes.

Local validation so far: the image builder produced an ext4 image whose `/root`
and `/usr/sbin/sshd` are owned by UID/GID 0, and the fresh compiler parsed the full
source tree. Live acceptance is pending.

The dispatch transport repair preserves the printer's existing domain validation.
Printer callers now supply `printer` and `printer_request`, for example
`{"operation":"observe"}` or
`{"operation":"print","project_path":"/absolute/project.3mf","project_sha256":"<reviewed digest>","credential_version":"<observed numeric version>","bed_clear":"true"}`.
No printer action is performed by VM bringup.
