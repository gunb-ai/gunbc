# Micro-VM launch displacement

**Analysis, not an executor design or schedule.** Price an atomic replacement of the per-attempt Firecracker launch on Mt. Collins. The finding is **medium, gap-intolerant**, with a missing host-boot execution lifecycle. This analysis does not choose an owner or authorize cutover.

**Citation standing at the analysis revision:** `gunbc.recurring_failure_mode.production_actuation_unmodeled_beside_a_modeled_probe` and the FCI-2 `gunbc.runner.runner_microvm_execution_authorization` are proposed on unmerged #10674, not corpus facts. The census and findings below do not depend on their landing. [Original analysis, search instruments and correction receipts](https://github.com/gunb-ai/gunbc/blob/3e5a0f1f0dbc62e0d3589df44146858d3147c9e9/docs/plans/microvm-launch-displacement-analysis.md).

## Goal and measured defect

`gunbc.fabric.fabric_ci_program` FCI-3 requires a persistent executor to **re-read the committed Grant before starting**: absent, corrupt or expired authorization produces zero target processes. Building a modeled executor beside the existing launcher leaves two authorities; it is useful only when it displaces that launcher under DESIGN §3.

The [absent-launch capture](../probes/mtcollins_jit_absent_launch_defect_2026-09-07.txt) records attempt `runner-canary-6-83b96b35` proceeding through credential drive, workspace and Firecracker launch with a zero-byte JIT credential. Refusal occurred inside the guest, after the forbidden target process existed. Firecracker exited 0 after guest panic: VMM exit status is not job success. The harness also misread a console drop as failed boot and power-cycled a host running a CI job.

The [bound-attempt capture](../probes/mtcollins_bound_attempt_host_capture_2026-09-07.txt) joins `runner-canary-8-1b3b5aea` to Actions run 34108061111 by a one-shot attempt label; a reused build label cannot establish that identity.

## Replacement boundary

| Steps | Existing owner at the census | Production route |
|---|---|---|
| 1. Host readiness; 2. guest-image build/digests | `runner_microvm_host_ready`; `runner_guest_image` | Already modeled: `fleet-converge.yml` runs `gunbc run` / `LocalExec` on the host. |
| 3. Media verification, jail chroot, kvm-owner | Partial: `runner_host_grants` | Hand-authored host boot tooling. |
| 4. Forwarding/nft/SNAT; 5. tap/address/MAC | `runner_microvm_network` | Facts modeled, production does not consume them. |
| 6. JIT mint/drive/ownership | `JitCredentialTransport` transport type only | No step authority. |
| 7. Workspace image, size/ownership | No authority | Hand-authored. |
| 8. Attempt cgroup limits | `runner_microvm_attempt` | Facts modeled, unconsumed. |
| 9. Jailer exec starts the VMM | No authority | The causation FCI-3 must replace. |
| 10. Census/teardown/receipts | Receipt modules | After-the-fact observation. |

The cut is **steps 3–9 in the HOST image's early boot**, not guest boot and not converge-time steps 1–2. Delete the displaced launcher and feeder steps in the same transition; Y cannot call X as a fallback. Observation remains a separate concern.

The executor must own these obligations:

1. Re-read committed authorization before any launch. Proposed FCI-2 `grant_authorizes_reservation` checks money fence and Work/Demand/Offer agreement; its proposed `seal_execution_binding` is an unconsumed frontier. Neither supplies the missing persistent reader.
2. Mint the one-shot credential only **after** that read authorizes the attempt.
3. Extend `gunbc.fabric.fabric_allocation_store_substrate` for durable grant state, reusing its presence/ownership/mode/prestate model. If a grant cannot inhabit it, report that finding; do not fork a store.

A JIT registration can be served once: the two paths cannot A/B the same credential, and a failed attempt consumes it. This justifies pricing the staged carve-out (shadow Y, then one transition), not keeping two launch authorities. Every required refusal must survive minimum Y.

## Host-boot placeability and recovered source

The proven converge route needs a self-hosted runner, repository checkout and built binary **on the host**. At step 9 the host has just booted from BMC virtual media onto tmpfs, about 17 seconds after kernel start. The runner is what step 9 creates, inside the guest. Reusing the converge route here is causally circular.

The replacement must instead be carried in the host image and invoked at boot. Two execution questions remain: can a `gunbc` binary run there on tmpfs, and what admission facts does the credential minter expose? This analysis did not have Mt. Collins access and does not claim either measured.

The source search found no repository build producing this host boot script. Its instrument searched the boot banner and `guest-boot-begin`, checked a known-positive gunbc hit, checked private-repository index reach with a universal word, and directly enumerated the tree where the search index was blind. Empty search results without the reachability control prove nothing; exact commands and dated results remain in the linked capture of this analysis.

**Recovery corrected loss, not build authority.** Seven source files (1,272 lines), including the `@@BUILD_ID@@` template `initramfs/init.recovered`, survived in `/var/tmp/gunbc-hostimage` on srv2 and are preserved as [frozen provenance](../recovered/mtcollins-host-image-source/README.md). They are readable X, not build inputs. The recovered directory's own dissolution condition governs its retention; deleting it does not displace the launcher inside the host image.

`runner_host_image_store` records a known, non-resilient store and a `first_reap` loss receipt. It does **not** establish location: the earlier srv1 `/srv/bmc` claim was false; the recovered export and seven digest ISOs were on srv2. Location requires a modeled read of BMC `remote/configurations`, not a copied literal. `runner_artifact_lifecycle` requires `DurablyMirrored` for repository-built host images.

Mt. Collins served micro-VM **canary** jobs, not the fleet's ordinary CI. The original all-host job census found srv1–srv4 only; retain that dated scope rather than saying every production job uses this script. The risk is that migration must pass through a launcher with no repository build. ctrl's version-controlled `jit-runner.sh` is the non-micro-VM sibling, not this producer; its earlier host-only drift caused an App-key rotation outage. Displacement is therefore neither a cross-repo negotiation nor a rename. Recovering X is done; modeling/building its replacement and cutting the image's live root remain.

## Admission axes and cutover bar

The capture alone names `signature unit-binding boot-binding attempt-binding expiry replay`; no corpus definitions establish all six meanings. `extdeps.github.actions_jit_runner` establishes one runner identity and an opaque credential blob. The host cannot parse it to prove binding: any binding must join minter records.

| Axis | Disposition and closing evidence |
|---|---|
| signature | **Required.** Establish authentic-and-present credential before step 9; the zero-byte capture had no admission verdict. The precise signature mechanism still needs its authority. |
| expiry | **Required.** An expired grant/credential must not create a guest that cannot register. |
| replay | **Deferred:** authority must decide whether upstream one-shot delivery already enforces it or host-side enforcement is owed. Do not duplicate an upstream authority. |
| unit-binding | **Deferred:** define the axis and checker; opaque bytes cannot discharge a mint-side obligation. |
| boot-binding | **Deferred:** define it and first settle whether it duplicates attempt-binding (attempt identity already changes each boot). Price no duplicate concept. |
| attempt-binding | **Deferred:** define the mint-side join; the capture's reused-build-label error makes exact attempt identity important for evidence as well as admission. |

The current sizing is two required whole-credential checks plus unresolved definitions, **conditional on the responsible authority confirming the other four are deferrable**. This analysis cannot grant that deferral itself. Like-for-like reproduction of all six missing checks is forbidden: it preserves the measured fail-open.
