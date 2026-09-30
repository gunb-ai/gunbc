# Native generation publication and bounded executor adapters

Source checkpoint only. No executor installation, commissioning, initial readiness, allocation, VM or SSH acceptance occurred. HOLD remains.

## Implemented

The executor journal binds the commissioning identity separately from the enclosing reviewed fleet bundle digest. This avoids a circular hash: the bundle includes the commissioning credential. The undeployed journal codec is now v2; v1 does not acquire an invented bundle binding.

The bounded helper verifies its real systemd invocation and cgroup membership, binds the actual controller invocation through protected CAS, invokes the existing strong settlement/readback authority, then publishes the fleet generation as the existing lease owner. An identical competing journal publication is accepted only after protected readback. Completion is recorded last. A replay after legitimate slot reuse reads the historical completion rather than demanding the predecessor guest still exist.

Shared generation publication now retains immutable per-generation bundle receipts and atomically replaces the generation head under the existing flock. The ordinary fleet apply projection uses the same publisher. The receipt precedes head replacement, so an interrupted publication has a recoverable identity; another bundle cannot take that generation. Historical recovery does not roll back a newer head. No new resident service, allocation lock or fleet counter is introduced.

The existing generation path and writer identity are shared with the helper. Privileged command construction uses the existing sudo adapter; low-level constructor access is not widened. Fleet terminal vocabulary moved to a small shared module without changing its wire format, so decoding a receipt no longer imports the full planner.

## Qualification

All checks use byte-identical dependency snapshots and the existing allocation-vertical-resume binary, SHA-256 `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`. They are not an exact-head compiler build, required CI floor or privileged live acceptance.

- Journal identity/transition controls: 9 passed, 6 GiB/no swap.
- Native publication process controls: 11 passed, including competing processes, lost response, interruption before head replacement, pending-receipt conflict, historical recovery, malformed heads and nonadjacent generations. These run against temporary directories under a real flock, not host fleet state.
- Full helper composition plus executor-membership controls: 5 passed, 791-module snapshot, 6 GiB/no swap, reported maximum RSS 5,327,736 KiB. Effectful functions were typechecked; production stores and systemd services were not mutated.
- Existing fleet receipt controls: 17 passed in a 46-module snapshot, 6 GiB/no swap.
- The previous integrated fleet snapshot passed 87 controls with the new writer. Final rerun after shared-authority extraction is recorded below when complete.

## Remaining gate and rollout constraints

The helper still needs initial reviewed-plan admission, initial submission preparation, a CLI protocol adapter and installer wiring. The commissioning-only fleet scope is not connected. Recovery without a recorded controller invocation must establish fencing/drainage before permitting another start; observing an inactive unit is insufficient. No manual repair or fabricated readiness is proposed.

The generation protocol is not deployed. Old numeric-only readers refuse the new two-field head; the reviewed cutover must install compatible readers/writers before publishing it. Existing generic baseline gates remain in force: the new native publication recovery does not claim that the generic CLI automatically accepts every old plan after the fleet has advanced.

Production limits are unchanged. The integrated test uses the previously approved temporary 12 GiB/no-swap scope; no runner services were changed.

Recovery checkpoint: five controls passed in a 792-module snapshot, 6 GiB/no swap, reported maximum RSS 5,335,912 KiB. The production recovery adapter verifies the actual executor and the protected plan/bundle identity. A recorded invocation resumes waiting on that invocation; accepted readback and release resume settlement; completed publication resumes historical completion readback. An unbound submission explicitly refuses pending fencing/drainage. This is safe refusal, not a completed drainage mechanism.

The reviewed-byte adapter also joins the canonical commissioning directive to a unique human-plan row, the shared plan/apply bundle digest and the existing run-bound fleet receipt. It deliberately does not mint plan approval or a start permit: root-custodied artifact loading and independent live admission remain preparation obligations. The bundle algorithm is shared with the ordinary fleet planner, not copied into an allocation-specific hash authority.

Durability repair: recovery now syncs an already-existing receipt and its directory before head publication. An injected directory-sync failure leaves the head unchanged on both the original call and its retry; normal recovery then completes. This closes the interrupted-link-publication window rather than treating existence as durability.

Reviewed-byte controls: 5 passed in a 642-module snapshot, 6 GiB/no swap, maximum RSS 3,998,880 KiB. The positive join and changed-shell, duplicate-credential, foreign-host and wrong-scope controls execute the real canonical directive/receipt decoders and shared bundle hash. Logs and source-manifest digests are retained in this receipt directory.

Integrated fleet rerun: 87 passed, exit 0, 1,913-module snapshot under the approved 12 GiB/no-swap scope, maximum RSS 11471340 KiB. This snapshot includes the shared terminal/location/bundle authority extraction. The subsequent receipt-directory sync repair is separately exercised by the final eleven-control native export; no exact-head CI or production installation is claimed.
