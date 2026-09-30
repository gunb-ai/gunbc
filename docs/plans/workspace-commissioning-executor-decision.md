# Commissioning executor lifetime: lightweight executor selected

Decision: the operator explicitly selected a lightweight executor. The implementation is a Bash transport emitted from the existing orchestration authority, with a systemd service model in the existing controller slice. No additional resident DAG interpreter is introduced. Production installation and commissioning remain unperformed.

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

## Selected realization

`gunbc.workspace_commissioning_executor` emits the native transport through the existing orchestration Bash backend. `gunbc.workspace_commissioning_executor_unit` models its host-owned service in the existing controller slice. The service has `Restart=no`, `RemainAfterExit=yes`, and the existing control-group teardown intent. It is not enabled at boot.

The transport holds the existing fleet lock across prepare, start/reattach, exact-invocation wait, and finish. It preserves named systemd fields; missing/duplicate fields, a pending job, changed invocation, timeout and failed settlement refuse. Systemctl requests have bounded command lifetimes. There is no interpreter invocation in the polling loop. A readable controller success alone is not the executor terminal.

The bounded helper interface is intentionally not a new authorization authority:

| Call | Obligation before returning successfully |
| --- | --- |
| `prepare <executor-invocation>` | Verify installed execution and reviewed plan under the lock; recover the old actor and pending jobs; acquire/recover the mutation guard; persist submission intent before returning `start:<prior-invocation>`, or return `wait:<recorded-invocation>` / `complete:<recorded-invocation>`. |
| `rearm <executor-invocation>` | After CAS fencing and native blocking StopUnit, independently verify quiescence and fresh admission, snapshot the new submitter credential and CAS-publish its submission. |
| `bind <executor-invocation> <slot-invocation>` | Persist the actual slot invocation against the exact prepared journal head. |
| `finish <executor-invocation> <slot-invocation>` | Require strong controller/journal/readiness/artifact readbacks; remove and reread the credential; settle the mutation guard; publish/recover the exact fleet generation; return the committed terminal. |

The production helper, durable journal, fenced drainage/rearm and installer wiring now exist in source. The reviewed fleet commissioning scope and dispatcher remain to be connected; see the preparation checkpoint receipt. The native component and service model do **not** remove that gate. In particular, a fresh executor must not turn an unresolved earlier submission into another `start` simply because the unit currently looks inactive. Controls use explicit process-boundary doubles, not manufactured production permissions.

The temporary additional-interpreter-service alternative was not selected.

## Newly qualified observation component

`workspace_commissioning_start_observe` retains property names and requires exactly one Id, LoadState, ActiveState, MainPID, InvocationID and Job. Thus `Job=` is distinct from a missing property. A queued job, nonzero PID, unloaded unit, wrong unit identity, duplicate field or absent field cannot establish quiescence. This is observation only; no start permission is minted.

Four focused controls passed over a 129-module dependency snapshot under 6 GiB/no swap, peak RSS 695,868 KiB. The effectful observer typechecked but was not run against srv1-13. A read-only local systemctl comparison confirmed the named-output distinction; it is not commissioning evidence.

Upstream reference: systemd v255 [`systemctl-show.c`, Job property rendering](https://github.com/systemd/systemd/blob/v255/src/systemctl/systemctl-show.c#L1022-L1034). This code renders a nonzero job ID or an empty value for a known Job property. The repository's existing `systemctl_show_properties_command` preserves its name.

## Review and acceptance remain unchanged

The selected executor does not permit live commissioning before the complete transaction, exact-head qualification and reviewed plan exist. Required live proof remains initial commissioning, first VM and SSH, release/sanitation, larger-profile reuse on the same physical slot, and expiry.
