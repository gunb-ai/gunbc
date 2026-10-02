# Executor wait bounds

The existing poll-count authority now also limits lock acquisition and each prepare/rearm/bind/finish helper call in seconds. Lock expiry exits 70; helper timeout fails through `set -e` and escalates termination after five seconds. Durable journal/mutation state is retained for recovery. These are per-operation limits, not one global transaction deadline. No service or memory limit is changed.

The actual model-emitted Bash passed 31 process controls. Five new cases cover unavailable lock, hanging prepare, rearm, bind and finish. They assert lock release and absence of successful publication; bind/finish retries use explicit journal-boundary doubles to exercise recovery without a second controller start. They do not prove live protected-journal behavior. Existing cancellation and exclusion controls still pass. Both executor DAG controls passed under 6 GiB/no swap; export also passed under that cap.

CI on predecessor 6798a868747 passed the prior import gate and then exposed stale test fixtures and forbidden test-to-test calls. This follow-up fills the roadmap fixture's attempt_stale field, handles WorkspaceSlotCommissioningOnly in two exhaustive matches, and extracts 13 shared assertion bodies into ordinary helpers. Every original named test remains. No assertions or checks are removed.

The shared repository's worktree metadata disappeared during this run. Source edits were preserved in an independent Git checkout at the same base. No live host state was modified. Integrated follow-up qualification remains pending.

## Integration results and escalation

All four commissioning dispatch/installer controls passed after the assertion-helper refactor under 12 GiB/no swap (pre-entry peak RSS 8,935,892 KiB). The roadmap entry now compiles and executes: 27 controls passed and 13 failed. `roadmap.log` preserves every result. Failures concern authority-row/dependency/detail rendering and client/CSS expectations; this checkpoint does not rewrite those assertions to match current output or claim the page qualified.

Live installation and commissioning remain blocked on integrated qualification. A further operational wiring check found the commissioning plan function only referenced by its declaration and the integration witness import, with no `fleet-converge` workflow choice exposing it. That connection must be resolved before a reviewed wet plan can run. Neither this receipt nor the process doubles establish initial readiness, reservation, VM boot, SSH, release or reuse.

The launch-environment scope entry passed all seven controls under 12 GiB/no swap. The full workflow-input entry was OOM-killed at the unchanged 12 GiB limit after eight passes (pre-entry peak RSS 11,947,192 KiB); it is incomplete, not green. The fresh-process invocation of the specific changed dashboard scope assertion also exited 137 under 12 GiB/no swap, after preparing 2,012 closure files (pre-entry peak RSS 11,954,560 KiB). Reducing assertion count alone does not solve this closure-size limit. It is not qualified.

The diagnostic page serialization succeeded. Its HTML has taskbar rows and lacks the legacy node-head/item-detail markup and requires-all sentence pinned by the failed witnesses; the CSS single-selector rules also differ. The compact diagnostic summary records these observations without committing the 1.1 MB HTML. This does not decide whether every removed display behavior is intentional; the page contract and its witnesses need reconciliation.
