# Executor preparation, drainage and installation checkpoint

Source work only. No production install, generation-zero readiness, reservation, VM, SSH or release/reuse acceptance occurred. The commissioning HOLD remains.

## Changes

The installed bounded helper now has a real CLI protocol and root-custodied plan reader. It checks the exact native/helper/unit bytes and loaded unit configuration, then joins the canonical commissioning credential, unique reviewed prestate row, plan/apply bundle hash and run-bound fleet receipt. Initial preparation reobserves the reviewed prestate under the executor's existing fleet lock before acquiring the slot mutation guard. Missing or duplicate prestate rows refuse.

An unbound interrupted submission first moves by protected CAS to `submission-fenced`. Losing that CAS to a controller winner does not request a stop. The native transport drains the known slot unit; the helper independently rereads quiescence and admission before rearming. The controller consumes a systemd-snapshotted submitter credential and must match it to the journal: an old already-created process cannot claim the replacement submission. Finalization removes both credentials before releasing the mutation guard. No invented attempt identity or readiness is used.

The existing controller installer now emits the native executor, bounded helper and modeled unit without starting or enabling them. Installer and commissioning readback share the same release-environment renderer. This fixes a real mismatch between the installed unit and the raw template previously used for comparison. Workspace installation no longer recreates the old CI reservation unit; withdrawal of an already-installed competing unit remains an explicit convergence obligation.

A sealed read-only commissioning basis derives identity from installed tree/binary, shared resource policy, pinned image and actual sanitation/history observations. A completed no-op requires coherent generation-zero readiness, matching committed provenance/executor record, the exact mutation-release generation, and both credentials absent. This observer is not yet a connected `FleetConvergeRequest` scope.

## Qualification

Existing compiler SHA-256: `717e9b4159dc7bc722e94a235325e6760f5acd246e63d8bed14b828dac970243`. Byte-identical dependency snapshots; not an exact-head compiler build or required CI floor.

- Four submission/fence controls passed in the 827-module dependency snapshot (824 demanded modules), 6 GiB/no swap; reported peak RSS 5,626,560 KiB. Effectful preparation and observation were typechecked, not executed against protected host state. Subsequent no-op tightening is qualified separately below. A plan-owned guard left before journal creation is normalized only while both protected journals remain absent; the actual acquired generation is retained for submission.
- Twenty-six native process-boundary controls passed, including drain/rearm ordering, failed stop, pending job, failed/malformed rearm, changed invocation, timeout and exclusion/cancellation. Helpers and systemctl are explicit executable doubles; this is not live systemd/controller acceptance.
- Combined installer/readback qualification is recorded below when complete.

## Required next connection

The remaining source blocker is the enclosing commissioning fleet scope and dispatcher, including root custody of the four reviewed input files and safe reattachment after cancellation. Do not run the ordinary locked apply script and then start this executor while retaining that lock: the executor deliberately takes the same lock, so that would deadlock. Do not let the generic writer publish a generation merely because `systemctl start` succeeded.

The dispatcher must preserve existing run/revision/host/receipt admission; publish input without replacing an active transaction; transfer execution to the exact installed host-owned executor; wait/read back its successful terminal and durable generation; and recover the same transaction after workflow loss. Only that complete connection permits a reviewed commissioning plan and bounded live acceptance. No slot is exposed as supply by this source checkpoint.

Installer integration: four controls passed in a 1,125-module snapshot under 12 GiB/no swap, reported peak RSS 8,246,900 KiB. These execute the real installation projection, shared unit renderer and seven reviewed-plan binding controls; the actual installation/readback functions are typechecked but not invoked against srv1. The finalizer's subsequent change is indentation only. This is narrower than the full required floor and does not qualify live actuation.

Read-only srv1 check: the commissioning executor unit is not installed (`LoadState=not-found`); the slot controller is loaded but failed, PID zero and no queued job. This does not establish sanitation, initial readiness or permission to retry. The observed controller invocation is retained in `srv1-unit-readback.txt`. No services were changed.

Final preparation rerun: four controls passed on the updated 824-module helper/controller/observer composition under 6 GiB/no swap, reported peak RSS 5,617,860 KiB. This includes exact released-generation and absent-credential no-op checks in the typechecked production observer. Those host observations were not wet-executed. Source manifests are unmodified dependency copies; no exact-head CI verdict is claimed.
