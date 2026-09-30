# Commissioning executor lifetime: decision needed

Standing: source/design investigation at allocation commissioning checkpoint `0622f9a1ed1`. Nothing installed, started, commissioned or reserved. The new named-property observer is a source component, not an executor.

## The boundary exposed

The reviewed fleet path runs `fleet_converge_locked_apply_script` under the workflow's process lifetime. Its inner script checks the fleet generation, executes `apply.sh`, and writes the next generation. The existing slot controller is independently owned by systemd. A start returning, or a workflow ending, does not mean that controller has completed.

The durable slot guard preserves safety on cancellation, but does not by itself establish which submitting process may still issue a start. In particular:

1. Persist start intent under the slot guard.
2. Submit `systemctl start`.
3. Lose the workflow before persisting the resulting InvocationID.
4. A retry sees the intent, but must distinguish a live submitter, a queued systemd job, a running invocation, and a submission that never happened.

A stopped/inactive unit is insufficient evidence to repeat the start. A readable receipt is insufficient evidence to declare success. The completed-controller recovery already implemented handles a later window; it does not settle this one.

## Required host-owned execution contract

Whichever realization is chosen must extend the existing fleet apply authority, not introduce another allocator or VM controller:

- Execute only the reviewed, revision-bound plan using the exact installed root-owned artifact.
- Own the existing host fleet-generation lock through reobservation, controller completion, independent readback, credential cleanup and generation publication.
- Record the plan, acquired mutation generation, submitting execution identity, pre-start slot invocation and selected new invocation durably.
- Survive workflow disconnection, or establish that the former submitting execution and its children are gone before recovery can issue a start.
- Observe named systemd properties, including Job, with missing/unreadable properties refusing. Settle queued jobs before treating a start as unsubmitted.
- Reattach to an existing invocation; never equate a retry with permission for another controller.
- Preserve recoverable held state after timeout, cancellation or uncertain readback.
- Reuse the existing slot controller as the sole commissioning/readiness writer.
- Keep the 12 GiB serving limit, 28 GiB cell envelope, and existing separately accounted controller budget unchanged.

## Realization choices

### Preserve the no-additional-interpreter-service constraint

Use a lightweight host-owned executor emitted from the existing fleet operation model. Its persistent process handles execution lifetime and exact-invocation waiting. It invokes bounded installed DAG operations for preparation and settlement; no interpreter is started on every poll. The dispatcher and privileged operations remain modeled, not an independently authored shell controller.

This requires implementing and qualifying that lightweight execution boundary. No such complete guarded commissioning executor was found in the current fleet apply path. The existing Spark transfer implementation demonstrates detached supervision, but its retry and publication semantics are specific to artifact transfer and cannot simply be reused as commissioning permission.

### Explicit temporary exception for the first commissioning run

Run the existing reviewed DAG apply logic inside a host-owned, one-shot systemd execution, from the same installed root-owned release. It would hold the fleet lock itself and start/wait/read back the existing slot controller. This is a temporary second interpreter process during commissioning, not another guest controller; it must fit the existing controller budget with no ceiling increase. Its combined peak and cancellation behavior are not yet qualified.

This option needs an explicit exception to the user's earlier instruction: “Do not ... add another interpreter-sized service for allocations.” It must not be selected silently just because it simplifies recovery.

## Newly qualified observation component

`workspace_commissioning_start_observe` retains property names and requires exactly one Id, LoadState, ActiveState, MainPID, InvocationID and Job. Thus `Job=` is distinct from a missing property. A queued job, nonzero PID, unloaded unit, wrong unit identity, duplicate field or absent field cannot establish quiescence. This is observation only; no start permission is minted.

Four focused controls passed over a 129-module dependency snapshot under 6 GiB/no swap, peak RSS 695,868 KiB. The effectful observer typechecked but was not run against srv1-13. A read-only local systemctl comparison confirmed the named-output distinction; it is not commissioning evidence.

Upstream reference: systemd v255 [`systemctl-show.c`, Job property rendering](https://github.com/systemd/systemd/blob/v255/src/systemctl/systemctl-show.c#L1022-L1034). This code renders a nonzero job ID or an empty value for a known Job property. The repository's existing `systemctl_show_properties_command` preserves its name.

## Review and acceptance remain unchanged

Neither option permits live commissioning before the complete transaction, exact-head qualification and reviewed plan exist. Required live proof remains initial commissioning, first VM and SSH, release/sanitation, larger-profile reuse on the same physical slot, and expiry.
