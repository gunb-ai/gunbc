# Workspace runtime integration

This source is not a wet allocation receipt. No image was built, no privileged controller was
installed, no VM was started, and no serving checkout was modified by this lane.

The workspace path reuses the existing microVM controller. Its convergence consumer seam is
`gunbc.runner_microvm_slot_controller.run_workspace_slot`. The inputs are a sealed
`WorkspaceLaunchBinding`, existing network sanitation subject, administrator-installed release
binaries, controller invocation, authoritative FabricStorageBinding, and the selected host gateway.
It calls `run_controller_with_wait`; in-flight persistence, staging, VMM cgroup placement, teardown,
recovery and sanitation are the same operations CI uses. The workspace-specific callback publishes
live access observations and honors durable release intent and the absolute lease.

The existing slot main now reads an optional systemd invocation credential before dispatching.
Absent workspace credentials preserve the original CI path. Present credentials must match the
exact owner ledger generation and allocation, authorized operator, observed host/slot, installed
release and configured cell CAS root. The shared controller applies the committed plan locally,
reobserves the real hold and substrate, and recovers in-flight cleanup before considering a new
launch. These changes have passed syntax checks but await the updated runtime witness run.

The installed unit uses `LoadCredential=workspace-launch:gunbc-workspace-%i`. Convergence publishes
JSON data at `/etc/credstore/gunbc-workspace-<slot>`; systemd snapshots it into the invocation's
credentials directory. It is never interpreted as environment settings. Images resolve beneath
`release_dir/images/workspace/<sha256>/`, keeping the existing installed-release custody.

## Runtime changes

- `runner_microvm_cell_readiness`: explicit GitHub runner versus workspace workload identity. Workspace
  identity carries bootstrap identity, image identity and absolute lease. No fabricated JIT registration.
- Readiness store codec persists the workload sum and still reads historical runner records that have
  a real `jit_registration_id`. The exact staged bootstrap path remains a durable cleanup subject.
- Shared launch plans carry a workload payload, committed disk size, cgroup properties and optional
  absolute lease. Shared staging writes and reads back `vm.json`, previously unwritten by this path.
- The shared jailer launch reads the clock after staging, then applies the remaining lease using the
  existing bounded command machinery. Failed clock reads and leases expired during staging refuse.
- `workspace_microvm` joins the actual cell snapshot, owner/Work, reservation generation and offer,
  memory grant, attempt, image pin and key reference. Machine memory is projected from the committed
  grant; CPU is projected from the admitted fixed profile. Disk is one 8 GiB disposable writable root.
- `workspace_microvm_access` joins the running systemd invocation, guest bootstrap identity and guest
  host key, then confirms that key through a real SSH key exchange from the selected host. It rechecks
  the live incarnation after the probe. A TCP port alone cannot establish readiness.
- The returned command installs the incarnation-specific known-host entry and uses strict host-key
  verification plus ProxyJump. No private key is sent to the guest or stored in the request.

## Image and deployment requirements

`workspace_guest_image` uses the existing pinned vendor base and kernel, without installing the
Actions runner. The base/kernel currently target aarch64; eligible supply must establish compatible
host architecture, KVM, network, capacity and installed controller prerequisites.

The unprivileged image builder extracts the pinned base, checks that SSH binaries already exist,
installs the workspace bootstrap/unit/config, and creates an 8 GiB ext4 image with the existing
userspace image builder. A base lacking SSH refuses. There is no unpinned package-install fallback.
The built image's measured SHA-256 must become the administrator-owned catalog pin before allocation.
Guest compatibility, networking and login still require a real boot test.

The demo guest key derives directly from gunbc.fleet_ssh_access.operator_ssh_principal; no user key
upload or per-request enrollment is required. The authorization policy binds the operator’s verified
Google subject to that fleet identity. Images still require administrator-owned installation and pins.
The guest bootstrap reads its identity and public key as data from a read-only block device; it never
sources that content as shell. It generates a fresh host key per incarnation and emits only the public
half through the invocation-bound console. The guest login user is `root`, with password authentication
disabled. The gateway account retains the user's existing host access; forwarding permission and actual
client login remain wet acceptance checks.

The owner ledger uses the existing FabricStorageBinding: local access only on the declared placement,
otherwise the existing served transport. Reservation cell CAS remains host-local: prepare, acquire and
release must execute on the selected host under its convergence lease. Path equality across hosts is
not shared storage. A successful release advances sanitized readiness through the actual freed CAS
generation so a subsequent acquisition is admissible; strict next-generation admission remains intact.

## Validation

`dag/test/claim/runner/workspace_microvm_witness_test.dag` covers exact machine-config sizing,
workspace recovery codec without GitHub fields, matching live SSH identity/key, old-key and foreign
bootstrap refusal, and bootstrap key record injection. Existing runner lifecycle/launch witnesses are
migrated to the explicit runner payload.

Bootstrap shell syntax checked with `/bin/sh -n`; exit 0. Compiler and witness verdicts are recorded
separately in the lane handoff. This document is not a claim that browser, privileged boot, cleanup,
lease expiry or subsequent reallocation acceptance passed.

## Userspace image build evidence

The shared pinned kernel/base authority now lives in `gunbc.microvm_guest_artifacts`; the existing runner image module re-exports it. This reduced workspace build source closure from 1,772 to 226 modules without changing artifact pins.

The DAG workspace builder completed under userspace fakeroot with a 2 GiB / zero-swap scope: exit 0, 73.84 seconds, peak RSS 1,523,712 KiB. Guest ownership is checked before assembly; plain unprivileged extraction is refused because it would copy UID 1000 into the guest. `debugfs` independently confirmed root-owned `/root`, SSH daemon, and generated unit in the resulting clean 8 GiB ext4.

Staged image SHA256: `97859b4f8e206e3398f02dc4070fad14cb5b7a7ec29e305a8895405750044305`. The build receipt is `target/workspace-image-demo/build-receipt.json`; source identities are in `target/workspace-image-closure/sources.json`. This artifact has not been administrator-installed or booted.

Gateway access still needs actual per-principal forwarding evidence: current srv2 policy prohibits TCP forwarding, so deriving an account and tailnet hostname does not establish a working ProxyJump path.

## Bounded validation and authority separation

The initial 1,859-module runtime witness load was killed at the enforced 6 GiB limit with swap disabled, before any witness verdict. This is not a passing suite. Existing attempt identity constructors and unit naming now live in `runner_microvm_attempt_identity`, with the old module re-exporting the same API. The readiness/store witness closure shrank from 1,784 to 257 modules; all 20 claims passed with exit 0 under a 2 GiB, zero-swap bound. These cover CI codec compatibility, workspace recovery without JIT fields, generation advancement on actual release, and quarantine/stale-generation refusals.

Two further existing authorities were separated without changing policy: pure floor resource requirements and buy terms moved from the workflow-emitting module into `fabric_floor_policy`, and the `CandidateRelease` record moved out of deployment observation into `live_deploy.candidate_identity`. Existing public imports re-export both. The controller closure fell from 1,850 to 829 modules and the combined runtime witness closure from 1,859 to 1,053. The combined 1,056-module witness load also hit the same enforced limit before verdicts. The exact systemd scope reported `oom-kill`; this was not an external operator cancellation. The production runtime was then validated in a smaller, explicit 845-module entry: it resolved and typechecked without diagnostics, with four of five claims passing. The positive readiness claim found a real runtime error: a record had been passed to the string-only structural hash function. The caller now serializes the existing canonical in-flight record before hashing; this fix awaits rerun. That run exited 1 normally, took 3:09.92, and peaked at 5,499,116 KiB with no swap. The 20 readiness/store claims were separately rerun after the commissioning variant was added and again passed with exit 0.

The final workspace image disables the pinned vendor MAC-derived `fcnet.service` startup link; the shared kernel `ip=` slot-network argument remains the sole address configuration. This was verified in the assembled ext4 before pinning it.

The corrected 846-module production runtime entry now passes all five workspace claims with clean exit 0 (2:59.82, peak RSS 5,531,768 KiB, zero swap). It explicitly imports the real slot controller and workspace preparation, tools, gateway, and convergence bridge. Snapshot: `target/workspace-shape-final-closure/sources.json`; log: `/tmp/workspace-runtime-shape-final-validation.log`. This validates exact guest sizing and attributed SSH-key/identity joins, not a booted VM or an established external SSH connection. Acquisition cancellation wet tests subsequently passed all three cases with clean exit 0 against temporary fabric storage (2:29.43, peak RSS 4,717,568 KiB, zero swap). The shared CI regression entry passed all 54 existing lifecycle, realization, and launch claims with clean exit 0 (2:22.59, peak RSS 4,527,780 KiB, zero swap); source manifest: `target/workspace-shared-runtime-closure/sources.json`, log: `/tmp/workspace-shared-runtime-validation.log`. These copied-closure test peaks are not measurements of the deployed full-source controller.
