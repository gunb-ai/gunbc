# Allocation lifecycle integration

The owner ledger is runtime state in the existing fabric DB protected state namespace, through `FabricStorageBinding` and `durable_cas_fabric_storage`. Request, release, observation, and immutable monetary release intent advance fabric heads; none requires a source merge. Schema and workflow changes remain deployed code. The selected host's existing local hold store remains its own authority: an identical path on another host is not the same store.

`workspace_record_request` retains idempotency history and allows one outstanding allocation per owner, including cleanup and acquisition uncertainty. `workspace_record_plan` persists the exact selected slot, root, predicted generation, grant, and monetary context before acquisition. `workspace_apply_recorded_plan` must win an owner CAS establishing `WorkspaceAcquiring` before touching the local hold. Retirement preserves that obligation. `workspace_settle_unstarted_retirement` can settle only a retired request that never acquired this obligation and has no reservation. An Acquiring record cannot be freed just because a hold read returns free.

`workspace_record_request_at` records the observed admission time, and `workspace_request_release_at` records the first observed retirement time. Both timestamps are metadata outside the immutable request; retries preserve them. Legacy records with missing timestamps explicitly retain unknown values. Expiry and controller stop use their observed clocks rather than inferring a release timestamp from the lease.

`workspace_budget_for_request` projects the advanced account from the last released reservation and its immutable release intent. It refuses outstanding or mismatched account histories and uses the approved initial account only before any reservation.

The existing lifecycle owns launch, stop, cleanup, and sanitation. `workspace_await_allocation` is its callback, not a second controller. It withdraws connection advertising on retirement, expiry, or failed readiness, and publishes only observations joined to the expected attempt. The page's connection projection expires after 30 seconds without fresh observation.

`workspace_complete_teardown` requires actual persisted sanitation for the reservation's cell and generation. Before freeing the local hold, `workspace_allocation_release` persists an immutable fabric release intent containing the reservation, fixed release timestamp, and advanced monetary account. Publication then requires a matching actual hold-ended receipt and readiness advancement from held to freed generation. `workspace_recover_release` reads the release intent, exact freed generation/release reference, and root-authoritative readiness. Uncertain or later-generation observations retain owner quota; recovery does not fabricate an effect receipt.

`workspace_record_first_ready` stores the earliest actual attributed SSH readiness clock under the exact reservation. Before common teardown, `workspace_capture_runtime_metrics` reads attempt and controller MemoryPeak/MemoryMax/MemorySwapMax and the controller ancestor slice, fences invocation identity before and after, and persists an immutable protected-fabric receipt. Missing readings remain null. The sample is explicitly before-teardown, not a final controller exit peak. Request-to-ready uses the immutable ready clock and original request time; release-to-clean uses actual Released observation and first retirement time. Unknown or backwards clocks do not become zero durations.

The workspace scope extends the existing fleet convergence request, member fingerprint, hashed plan/apply bundle, host lease, and receipt path. The baseline includes owner head version, full allocation, selected host, hold root, and installed controller release. Apply recovers its locator from the hash-admitted body and rereads the same owner version on the selected host. Under the existing host lease, its script stages the exact typed launch document, installs it at the selected slot's credential path, and starts the existing slot service. The controller's `LoadCredential` snapshot preserves the planned invocation data without interpreting it as environment assignments. The controller must match the authorized principal, owner revision, full request/incarnation, actual host/slot, configured hold root, and installed release before applying the reservation.

Workspace credential publication and slot start use one `workspace_launch_operations` value both in the apply script and in the exact sudoers roster. The roster derives only from commissioned workspace-purpose slots. The current empty designation produces no grants or eligible workspace offers.

For a prepared allocation, the local plan entry is `gunbc.fleet_converge_plan_cli.allocation_demo_plan_wet`. It reuses operator-local convergence provenance and reads `GUNBC_WORKSPACE_REQUEST_ID` and `GUNBC_WORKSPACE_CONTROLLER_RELEASE` as plan selectors. Apply uses the existing operator-local apply command and its expected bundle hash, not those environment fields. Initial selected-host preparation still needs its production supply/transport integration; the page must only record intent.

Validation is incomplete. The 108-witness pass reached typechecking of the migrated owner store, readiness release transition, and slot controller, but a `Present.profile` compiler diagnostic prevented witness evaluation. Later acquisition, release recovery, directive, and convergence edits have syntax probes only. The new temporary-fabric-store witness exercises retry, quota refusal, owner separation, unstarted release, reuse, and retained idempotency history without fleet effects. A pure race witness covers retirement winning before acquisition and preserving an already-established acquisition obligation.

Remaining acceptance work includes protected fabric writer authorization, real dedicated slot enrollment and observation, selected-host preparation, controller installation, actual login and SSH, complete sanitation, expiry with the browser closed, and reuse. No fleet mutation, deployment, or complete VM workflow is claimed by this handoff.

## Read-only srv1 commissioning evidence, 2026-09-27

SSH used the existing `briansrls@srv1` route with BatchMode and StrictHostKeyChecking enabled. The host reports hostname srv1. The modeled local allocation root `/var/lib/gunbc/fabric/allocation` and readiness root `/var/lib/gunbc/microvm-cell-readiness` are absent. The failed srv1-13 controller journal and receipt explicitly report `ReservationNotObserved` because the allocation store could not be read. Its installed release is still `cfb9ff80652fe0a74093f08cade64f59d9695266`.

Actions runners 01–12 were active; 13–50 were masked/inactive. Runner 01 reported a 26 GiB memory ceiling and 32 GiB swap ceiling. The controller ancestor slice reported a 24 GiB ceiling, zero swap, peak 14,734,024,704 bytes, and zero current tasks. The observed fabric-cell-zp444 slice had zero tasks but an infinite memory ceiling. `/dev/kvm` exists. Slot TAP interfaces including gunbc-tap13 existed with no carrier; this does not prove unheld TAP ownership or sanitation. No Firecracker/jailer process was observed by the read-only census.

No eligible workspace slot was proven. srv1-13 is a potential commissioning candidate because its runner is already withdrawn, but it still needs dedicated-purpose enrollment, host-budget admission, a bounded cell, authority-directory provisioning, actual sanitation/readiness, and current controller/image installation through modeled convergence. Neither a failed service nor free host RAM substitutes for those authorities. No host mutation was performed by this lane.

## Commissioning integration work in progress

`workspace_commissioning_admit` now distinguishes absent administrator provenance, unsound host budget, active/unknown previous purpose, missing obligation history, and incomplete initial sanitation. Its output binds the whole existing 28 GiB envelope, largest-profile CPU capacity, slot, planned artifact, and retained settled generation. No production observation currently supplies its approved provenance or complete census, and the workspace designation remains empty.

The existing `FabricExecutionCellsOnly` and `FabricAllocationStoreOnly` convergence scopes own cell and hold-directory provisioning. Initial readiness uses a new sealed permit and `WriteCommissioned`, which accepts only an absent readiness record and never resets existing history, in-flight recovery, or quarantine. The common sanitation authority now accepts cell-scoped observations without requiring an invented prior sandbox. Production commissioning still needs app-approved artifact provenance and an initial audit action in the existing controller's typed credential dispatch.

The unconditional network-observation refusal has been replaced by actual typed readers after quiescent stop and completed cleanup. The TUN reader scans per-thread `/proc` fdinfo and checks kernel attached queues through read-only ETHTOOL_GCHANNELS before and after. It refuses multi-queue mode because detached queues need additional proof; single-queue kernel counts cover references held outside installed process FDs. It never infers no holder from carrier state. The descriptor decoder's four witnesses passed on 13 byte-identical modules under 2 GiB with swap disabled. Full network producer type/wet validation is pending; its dependency closure fell from 1780 to 271 modules by moving the existing guest-network shape into a lightweight authority and preserving original reexports.

Acquisition actor binding (source, pending full validation): the owner Acquiring transition now
precedes an immutable protected-fabric journal keyed by the exact reservation plan; the journal
precedes the hold effect. Its invocation comes from INVOCATION_ID and must match manager readings
before/after, with the manager's MainPID equal to this interpreter's /proc/self/stat PID. A different
or unreadable actor may only observe the existing hold, never retry acquisition. No-hold Acquiring
still retains quota: a stopped controller does not prove a queued remote/socket CAS cannot commit.
Group A's hold authority needs a commit fence or drained-operation evidence before that uncertainty
can be settled. If the actor journal is still absent after a crash, its create-only CAS elects exactly one
invocation before any hold effect; an old delayed journal write cannot authorize a second actor.

Initial commissioning is an authorized convergence operation, not intrinsically a human approval
prompt. The existing effect authorization pattern must choose WIF/app authority. No source-only
AuthorizedPlan fixture may be promoted to production provenance. Real workspace commercial terms,
initial account and provider-use policy remain unenrolled; the compile floor's fixtures/account are
not substitutes. The request preparation bridge is ready to consume these inputs, but the production
CLI cannot manufacture them from an unplanned request.

Absent-actor retirement now uses an immutable cancellation tombstone, winning against actor election
before an exact owner CAS publishes Released. It requires the protocol marker created before a new
Recorded-to-Acquiring transition; legacy Acquiring rows without that marker retain their obligation.
An elected actor still requires hold authority fencing/drain evidence for no-hold settlement.
