# Lightweight commissioning executor controls

This is local component qualification, not an installed executor or commissioned slot.

The transport is emitted by `gunbc.workspace_commissioning_executor` through the existing orchestration Bash backend. Its systemd service model uses the existing controller slice, control-group teardown, `Restart=no`, and `RemainAfterExit=yes`. No additional resident DAG runtime is introduced. The process owns the existing fleet lock and calls a bounded helper only at prepare, invocation binding, and settlement boundaries.

## Evidence

- The emitter and service-model closure typechecked under 6 GiB/no swap using the recorded existing compiler binary. This is not a current-head compiler build or integrated CI floor.
- Two DAG controls passed: controller/readable refusal is not the fleet terminal; the service's lifetime is native and has the intended restart/teardown behavior.
- Executable controls run the **actual emitted script**, with explicit doubles for the helper and systemctl boundaries. They exercise fresh start, reattachment without another start, completed recovery, missing lock, missing/duplicate systemd fields, pending jobs, wrong unit, invocation changes, failed prepare/bind/start/settlement, readable refusal, timeout, lock contention and cancellation.
- A temporary user-manager service ran the same emitted script with systemd's actual InvocationID. Its dispatcher exited while it waited; the executor remained alive, then completed when the simulated controller became terminal. The service was stopped and removed from the manager afterward. Its MemoryPeak is recorded in the control output; it includes test doubles and is not a production controller measurement.
- The native waiting loop starts no DAG interpreter. Successful settlement requires a distinct fleet-generation-committed terminal, not merely the slot controller's commissioning receipt.

`artifacts.json` pins the emitted script, new source, control harness and compiler binary. `sources.json` records the final byte-identical dependency snapshot. Export logs record the actual source compilation. `witness.log` records the two unchanged model controls over the preceding snapshot; the final transport-only lock-open refinement is covered by re-export and executable controls.

## Deliberately unclaimed

The production helper and enclosing durable execution journal are **not yet wired**. A helper must validate the installed execution/plan, persist submission intent and actual slot invocation under CAS, establish old-actor/job recovery, perform the existing strong readbacks and credential cleanup, and commit/recover the fleet generation. No mocked helper result is a production start permit.

There is no new privileged grant or installer connection in this checkpoint. The fleet commissioning scope, installation, exact-head integrated qualification, read-only plan review and live acceptance remain required. No initial readiness, reservation, VM, SSH, release or same-slot reuse has occurred. Existing server and VM memory ceilings are unchanged; no runner services were altered.
