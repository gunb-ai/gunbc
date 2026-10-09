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
| Deployment candidate | Compiler startup creates ignored `rung_drop/roster.dag`, which candidate admission rejects | Exclude that exact compiler-derived file alongside the existing failure-mode roster; preserve sibling and secret-file refusals |
| Workflow dispatch | Main declares 28 inputs; GitHub accepts at most 25 | Keep printer selection separate and carry its five operation fields in one `printer_request` JSON input |
| Guest image | Builder exists; no image delivery in controller install | Build under fakeroot, archive kernel/rootfs/digest receipt, pin the measured rootfs, verify installed bytes and custody |
| Controller | Installed revision `cfb9ff80652fe0a74093f08cade64f59d9695266` | Install this PR through the existing controller installer |
| Slot budget | `fabric-cell-srv1-13.slice` has no installed unit file | Converge the modeled execution-cell boundary; a synthesized systemd slice is not proof |
| Cell inventory | Read-only plan `local-2026-10-09T03:18:35Z` refuses foreign `fabric-cell-zp444.slice` | Establish ownership before retiring it; the empty transient slice has no unit file or remaining journal, and is not silently ignored |
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

Local validation: 44 focused witnesses passed across allocation dispatch, lifecycle,
launch directives, host offers, owner policy, gateway refresh, workflow dispatch and
deployment candidate admission, including three deployment-readiness refusal controls and two cutover recovery/refusal controls. The generated compiler-pair build step was also executed with controlled compiler outcomes: both output streams were retained, and exit codes 0 and 37 were preserved. The image builder produced an ext4 image whose
`/root` and `/usr/sbin/sshd` are owned by UID/GID 0. The fresh compiler parsed the
full source tree, and full generated-artifact regeneration completed successfully.
The combined workflow witness process needed a 24 GiB memory cap; its earlier
16 GiB run was killed by the cgroup limit, then passed with the larger cap.

[Image build 37878358778](https://github.com/gunb-ai/gunbc/actions/runs/37878358778)
at revision `521085ab254290029f8dfbaa312a218f84a6be7b` built its compiler successfully,
but packaging refused `BuildDiagnosticsMissing`: the compiler-pair build had not
produced the log required by its pack. The build dispatch now uses the existing
floor log-capture wrapper, preserving the compiler's exit status. [Retry 37882239644](https://github.com/gunb-ai/gunbc/actions/runs/37882239644)
at revision `5504b52cd0001115afcfb47a98e98b16c44e199c` succeeded. The downloaded
artifact's rootfs SHA-256 matches its receipt:
`d15a55780a707ec832a09e3599306254b8d7c428ff25ff621b427b826eeb0b27`.
That archived digest is now the reviewed image pin. The kernel matches
`cb1291c66bca75bc11cb9c8357fcef9965bb1786dffcb42a60923c3e0e49f319`;
independent ext4 inspection confirms `/root` and `/usr/sbin/sshd` belong to UID/GID 0.

The operator identified `fabric-cell-zp444.slice` as a disposable test slice and
authorized retirement. Immediately before stopping it, readback confirmed no unit
file, drop-ins, child units or processes. Its unit and cgroup are now absent.
Execution-cell plan `local-2026-10-09T03:37:32Z`, hash `76783579d393ed3d`,
advanced generation 18 to 19. Apply `local-2026-10-09T03:47:17Z` completed
`fully_applied`, receipt `34c5252f521bf5b3`. Independent systemd readback confirms
active `fabric-cell-srv1-13.slice`, persistent limits of 28 GiB for MemoryMax and
MemoryHigh, no swap, TasksMax 16384, and the modeled CPU settings. The cell and
attempt directories have the modeled custody. Commissioning, allocation and VM
boot remain unproven.

The read-only deployment probe admitted clean candidate
`19b12d88d6c0206c3f259cf85b0b664e65000b26` after the derived-roster repair. Its
release-member plan is: install executable
`sha256:5cbf98426843aebd1fad3c2f9cc52c6ba04ba6fbb71f6bbde1ea40eff8391543`,
publish that source revision, write `gunbc-roadmap.service`, reload systemd and
restart the roadmap service. The existing deployment additionally reinstalls its
fabric-storage, approval-broker, tailnet-door and timer members, which do not yet
have differential member identities. Consequently this prerequisite is a shared
production deployment with service interruption, not a slot-only mutation.
The operator approved this shared deployment, including temporary interruption.
The first operator-local attempt installed the generated `ghrunner` sudoers but
refused its exact `tailscale` grant probe under the operator account before other
mutations. The operator's broad sudo grant does not satisfy this exact grant-list
probe; the same probe admits the existing `ghrunner` grant. The retry uses the
existing `live_deploy_apply_srv1_wet` entry as `ghrunner`, without expanding the
operator grant roster. The failed attempt also exposed 720 unnecessary readiness
waits after refusal. Readiness polling now requires a converged mutation; cutover
recovery still runs so interrupted route changes can be rolled back.

The dispatch transport repair preserves the printer's existing domain validation.
Printer callers now supply `printer` and `printer_request`, for example
`{"operation":"observe"}` or
`{"operation":"print","project_path":"/absolute/project.3mf","project_sha256":"<reviewed digest>","credential_version":"<observed numeric version>","bed_clear":"true"}`.
No printer action is performed by VM bringup.

The approved deployment's tree sync and independent repository readback completed
at revision `19b12d88d6c0206c3f259cf85b0b664e65000b26`. Dashboard, approval broker,
tailnet door and fabric storage restarted between 04:11:48 and 04:12:04 UTC;
dashboard readiness is still being checked. At 04:21 UTC the authenticated operator read returned `WorkspaceOperatorObserveRefused: request absent for operator`, proving the protected owner ledger is readable without creating an allocation. The preceding probe finished 19 seconds before the new storage listener opened and is not counted as acceptance.
The nested deployment/readback interpreters peaked above the original 24 GiB
scope bound, so that operation's cap was raised to 36 GiB with swap disabled.

The enrolled operator key is the MacBook key, fingerprint
`SHA256:KEMuhG7TCEjoaF2Q8Xn/4jJd+5uZUggzycUYnLXQojI`. It is already authorized
on srv1. The local srv1/srv2 keys differ and no agent was selected. Gateway and
guest SSH acceptance therefore require the operator's forwarded agent; its socket
has been requested. No enrollment or authorized-key policy is being widened.

The deployment installed its members but returned a cutover refusal: the existing
`ghrunner` deployment entry cannot read the protected-state credential. Its final
cutover gate therefore could not read the journal and performed no route effect.
The dashboard's loopback `/healthz` independently reports revision `19b12d88d6c`;
the existing tailnet root route still targets that loopback backend. The new
parallel tailnet-door unit also failed binding `/opt/gunbc/tailnet-door.sock`:
its service user cannot create entries in root-owned `/opt/gunbc`. These are
shared-deployment gaps, not permission to distribute the state key or re-own the
installation root. Full shared-deployment convergence is not claimed.

During cold startup, the shared dashboard slice exceeded its 49 GiB memory-high
threshold (53 GiB hard cap) and spent about 60% of wall time stalled. Publication
and belt timers/helpers were temporarily stopped within the approved interruption
window to let the services finish startup. Their restoration remains required.
