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
| Job selection | Controller install queues the shared credentialed job as well as its dedicated job | Derive shared job selection from the existing mode-to-job authority |
| Network receipt | The historical TAP/firewall installation exists, but the controller receipt is absent and the workflow observer entry was overwritten | Restore the observer, publish a root-owned initial receipt from live reads, and keep later sanitation observation separate |
| TAP IPv6 | Installed sysctl disables IPv6; networkd leaves it enabled on the live TAP | Disable networkd IPv6 autoconfiguration in the same generated TAP policy, then converge and read back |
| Controller | Installed revision `cfb9ff80652fe0a74093f08cade64f59d9695266` | Install this PR through the existing controller installer |
| Slot budget | `fabric-cell-srv1-13.slice` has no installed unit file | Converge the modeled execution-cell boundary; a synthesized systemd slice is not proof |
| Cell inventory | Read-only plan `local-2026-10-09T03:18:35Z` refuses foreign `fabric-cell-zp444.slice` | Establish ownership before retiring it; the empty transient slice has no unit file or remaining journal, and is not silently ignored |
| Commissioning | No commissioning or readiness record | Run reviewed commissioning plan/apply and verify its protected transaction |
| Protected state service | Running revision `5b96cab749088aa5cec71ce9ad52e1857724d252` predates the authenticated state route | Converge fabric storage through its existing deployment path |
| State-key custody | Key exists; operator can read it; `ghrunner` cannot | Preserve custody; exact installed helpers reconcile admitted intent and read the selected allocation |
| SSH access | Guest enrollment reused the personal operator key and required gateway evidence even for a client on srv1 | Enroll a dedicated guest key on srv1; model host-local SSH explicitly and retain gateway proof for gateway clients |
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
3. Converge the execution-cell boundary and authenticated state service. On srv1,
   the local administrator can run `runner_microvm_network_plan_local_wet`,
   review `target/microvm-network-local-plan.txt`, then run
   `runner_microvm_network_apply_local_wet` from
   `dag/gunbc/runner/runner_microvm_network_apply.dag`. Both require `host=srv1`,
   `expected_revision` equal to the clean checkout's HEAD, and effective UID 0.
   Run `runner_microvm_network_initialize_local_wet` from
   `dag/gunbc/runner/runner_microvm_network_observe.dag` with those same arguments
   to publish the controller receipt from live readback. This uses the installed
   root-owned receipt directory and refuses replacement of a different generation.
   These local entries require no SSH agent. Then run
   `workspace_commissioning_plan` and its reviewed `apply` on srv1. The existing
   operator-local fleet plan/apply path is available for pre-merge validation;
   do not widen the main-only GCP login to run branch workflows.
4. The installed operator enrollment uses a dedicated guest key whose private
   file stays on srv1 at `/home/briansrls/.ssh/gunbc-workspace-srv1-13`, owned by
   `briansrls` with mode `0600`. No agent forwarding is required. As the enrolled
   operator on srv1, use `workspace_operator_request_wet` with
   `GUNBC_WORKSPACE_REQUEST_KEY`, `GUNBC_WORKSPACE_REQUEST_PROFILE` and
   `GUNBC_WORKSPACE_REQUEST_EXPIRES_AT`. Use a canonical UTC expiry within the
   policy's one-hour lease. The owner and public key come from installed policy.
5. Dispatch `workspace_allocation_plan` and review/apply its artifact. Observe
   through `workspace_operator_observe_wet` with the same key. Each operator
   command writes `target/workspace-operation.json`; a recorded request alone
   is not evidence that the guest is usable.
6. Authenticate SSH from srv1 using the allocation's connection data and verified
   guest host key. The rendered command selects the dedicated identity file and
   disables the agent and agent forwarding. Record the guest identity and a command
   executed in the guest.
7. Run `workspace_operator_release_wet` with the key and
   `GUNBC_WORKSPACE_RELEASE_GENERATION` from `reservation.generation` in the
   observation (not the owner-ledger generation). Reconcile with
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

The host-local SSH revision passes all 28 focused witnesses across access policy,
owner enrollment, guest connection verification and allocation reconciliation.
The full tree (8,268 source files) parses, and formatting checks pass. An independent
`ssh -G` read on srv1 confirms the dedicated identity file, `IdentityAgent=none`,
`IdentitiesOnly=yes` and `ForwardAgent=no`. This checks client selection; it is not
a guest login receipt.

Local validation of the preceding revision: 44 focused witnesses passed across allocation dispatch, lifecycle,
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
dashboard readiness later passed through the existing HTTPS route. At 04:21 UTC the authenticated operator read returned `WorkspaceOperatorObserveRefused: request absent for operator`, proving the protected owner ledger is readable without creating an allocation. The preceding probe finished 19 seconds before the new storage listener opened and is not counted as acceptance.
The nested deployment/readback interpreters peaked above the original 24 GiB
scope bound, so that operation's cap was raised to 36 GiB with swap disabled.

The operator selected SSH from srv1. A dedicated guest-only ED25519 key was
created there, fingerprint `SHA256:sxhkzO98Tp3tSXi+nD2FJBfbRTnOYQn6QpuTlETnbQI`.
Only its public key is enrolled in source. It is not authorized for fleet-host
login or given to workflow jobs. Personal-agent forwarding was proposed earlier,
then withdrawn; no forwarded personal agent was used. Host-local access and
access through a gateway now have distinct policy and evidence types, so an
observation on srv1 cannot assert gateway forwarding permission.

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
window to let the services finish startup. Both timers were restored and read back active. The unused replacement
tailnet-door service is stopped after its socket-permission failure; its failed
cutover never changed the root route. An independent HTTPS `/healthz` read through
that existing route returns the approved revision and surface identity
`998c13f6282e889f`.

Controller installation [37884238413](https://github.com/gunb-ai/gunbc/actions/runs/37884238413)
was canceled before installation at `95037360ab1040b9de4efe4671bb53cad473cb34`
when the operator selected host-local SSH. The verified image artifact from run
`37882239644` remains the image to install. No commissioning or guest allocation
has been performed yet.

Controller installation [37887389027](https://github.com/gunb-ai/gunbc/actions/runs/37887389027)
selects revision `a85cc481af5b07dc89f270925bb55bc6f025c8cd` and image build
`37882239644`. Both srv1 operation checkouts have that revision. The first live
readback before installation found no controller process or queued start, and both
historical root-owned receipts are `ReservationNotObserved` refusals. They are
preserved. Installation and commissioning remain in progress.

The build job of `37887389027` succeeded. Its source tree is
`5dcdc0feb191c93ac957850344c88796c5651a39`, with request key
`release-bins-38e793db9ef32d629136be09416dabc9da34cdd3139f3541c9e025c146cb75a8`.
The general converge job also queued for this dedicated mode, occupying its shared
host-mutation concurrency key. Both queued jobs were canceled before starting;
there was no installation effect from the workflow. The general job condition now derives
from `fleet_converge_mode_job_id` to remove that second assignment.
The successful build artifact was downloaded and verified on srv1 using the exact
existing installer consumer script and the key recorded by the build job. All
16 binaries passed verification, and the image/kernel hashes still match. The
existing `microvm_controller_install_srv1_wet` entry is running as `ghrunner`
under an operator-local 24 GiB/no-swap scope. No workflow identity was fabricated.

The workflow-selection repair passes five focused witnesses. Regeneration succeeds,
and an independent check of the emitted YAML finds exactly one consumer job for
each of all 78 dispatch modes. The shared job selector is the only generated
workflow field that changed. All 8,269 source files parse.

The local `a85cc481af5` installation completed its mutations but returned
`executor scripts or loaded unit failed exact readback`. The final read attempted
to open root:root `0440` `/etc/sudoers.d/gunbc-workspace-commissioning` as
`ghrunner`; that account has no read permission or read grant. `visudo` accepts the
installed policy. Independent readback matches the compiler artifact's SHA-256
`16f75c38f7b5a3eb7c3580dbe9f6bae59b5997630d808bd3cc3b39ad49dd9930`, both image pins,
and the root-owned release/readiness/commissioning directories. The loaded slot
unit selects `a85cc481af5` and has no running process. Installation convergence is
not yet claimed: the readback needs an exact privileged read of its own policy.

The repair derives that read's argv and permission from the same privileged
operation, retaining root:root `0440` custody and exact content comparison. Seven
focused dispatch/allocation witnesses pass, including the exact policy-file grant
and the existing prohibition on workflow-created operator intent. The verified
`a85cc481af5` compiler binary can interpret this DAG-only repair; its build identity
remains `a85cc481af5`, separately from the new installed source revision. It is not
represented as a rebuilt compiler pack for the new tree.


The read-only commissioning preflight at `a85cc481af5` refused because
`/var/lib/gunbc/microvm-network/converged-slot-network.txt` and its parent directory
are absent. The network workflow still names `runner_microvm_network_observe_wet`,
but that function was overwritten when the same module was repurposed for
per-attempt sanitation. The previous implementation was recovered from source
revision `34ac2fefab51e912e9499dd23aa92bb3ebdac824`; sanitation keeps its own current
producer. The repair restores the workflow entry and adds an explicit local root
initializer which observes the host, publishes once with create-only semantics,
and verifies existing generations against fresh observations without overwriting.

The live `gunbc-tap13` holds `172.30.13.1/30`, and the existing microVM nft service
is active. However, `disable_ipv6` reads `0` despite the installed sysctl file
requesting `1`. systemd 255's [link IPv6 decision](https://github.com/systemd/systemd/blob/v255/src/network/networkd-link.c)
and [sysctl writer](https://github.com/systemd/systemd/blob/v255/src/network/networkd-sysctl.c)
show why the generated `.network` file must also disable link-local IPv6:
networkd re-enables IPv6 when its configuration calls for it. The local network
plan/apply entries use the existing staged-file and ordered-operation producers,
require root on the named host at the named revision, and use no forwarded agent.
Network changes have not yet been applied.

The `edad20b0ded` controller installer completed its mutations and installed the
exact policy read grant. A real `ghrunner` invocation can now read that policy,
and `visudo` accepts it, but exact adapter readback still refused. The next
boundary diagnosis identified `bounded_shell_host_drain`'s `trim_end` capture contract:
the final newline cannot survive a `cat` through that transport. Policy readback
now compares the actual file's SHA-256 with a digest of the expected bytes. The
exact grant becomes `sha256sum -- <policy path>`; it grants neither arbitrary
file reads nor the protected-state credential. The focused validation passes all
34 checks across network apply/convergence, observer production, policy readback
and allocation dispatch. The final network-path retry passes its 12 checks after
using the existing command executor and effective-UID reader. All 8,272 source
files parse, and formatting checks pass. Live reinstallation and network
convergence are next; no VM has been commissioned, allocated or booted.

Controller installation at `682c32644e84da04dd4e73d334413057a659dbf3` now exits
successfully, including exact executor/policy and guest-image readback. The network
plan at that revision also succeeds. Comparison with the installed files finds
only `LinkLocalAddressing=no` and `IPv6AcceptRA=no` added to the ten enrolled TAP
configurations; firewall, TAP addresses and IPv4 forwarding remain identical.
The reviewed apply refused before any network installation: its staging writer
used exclusive creation against existing root-owned staging files. Network
convergence is not yet claimed.

Pre-commissioning review found the native executor started its bind interpreter
alongside the controller in the same 24 GiB slice. The controller already binds
its journal identity before commissioning effects. The executor now waits for
that invocation to terminate before its independent bind/readback, keeping those
two preparations sequential without changing the resource ceiling or recovery
fences. This fixes the normal start overlap; recovery-path aggregate memory has
not been measured. No live commissioning OOM is claimed.

The manual transport experiment used `export_executor` from
`dag/test/claim/workspace_commissioning_executor_witness_test.dag` and the local
`target/srv1-bringup-evidence/executor-bind-order-controls.py` driver against
`target/workspace-commissioning-executor.sh`. The Python driver is local dev
tooling, excluded from the committed substrate by `gunbc.repo_workspace`; these
results are not claimed as an ongoing CI regression gate.
All eight revised-script cases pass: normal start, invocation replacement,
surviving PID, refused finish, wait, settle, complete and drain. The optional
`--reject-active-bind-script` control against the installed `682c32644e8`
predecessor refuses its premature bind, distinguishing the two schedules.

The local network staging writer now creates a private temporary inode in the
verified directory, fills it, and atomically replaces the exact staged destination.
The existing scope admission, root-custody checks and exact byte readback remain
required. The manual native filesystem control
`test.manual.runner_microvm_network_local_stage_test.local_stage_replaces_owned_bytes_wet`
passes creation, changed-content replacement, replay, 0600 mode, trailing-newline
preservation, out-of-scope refusal and scratch cleanup. The out-of-scope control
uses an existing allowed directory, so it distinguishes admission refusal from a
missing-directory failure. It runs as an ordinary user in a fresh scratch directory;
this is manual evidence, not an enrolled CI claim. The live network apply must
still pass with the repaired writer before commissioning.
