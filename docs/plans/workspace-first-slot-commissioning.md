# First workspace slot: production commissioning joins

Source audit: allocation integration `2ad001dc88dcd554e0de9a2dd2853ab3f77124b8`,
2026-09-29. This is an implementation plan, not an installation receipt.
The operator has authorized transferring existing capacity through convergence.
No cost observation is required. Preserve the 28 GiB cell envelope and separately
accounted controller; no memory ceiling increase is proposed.

## Current prerequisite standing

- Credential custody #12580 is merged. Run 36605269073 established the pinned
  state-writer key on srv1 with ownership, mode, content equality and staging
  readback. Remote srv2 delivery is being retried in run 36613639455, executing
  the same qualified b43704581 revision through #12646's runner-placement change.
  Its job now requests generic fleet labels; success/readback remains pending.
- Allocation full serve at ba0ca2fef starts within 12 GiB/no swap. Generated
  qualification at 304b89694 passed, but the integrated floor found stale fixtures
  and forbidden test imports. The source repair has 66 focused passes under
  6 GiB/no swap; the complete floor and updated page suite remain unqualified.
- Read-only srv1 observation finds the slot service failed since September 24,
  an inactive cell slice without a finite memory bound, and a separate active
  24 GiB/no-swap controller slice. Neither failure nor absent stores proves
  absence of prior attempts.

## Required production connection

`workspace_commissioning_admit` and `WriteCommissioned` currently have witness
callers only. `workspace_observe_commissioning_obligations` reads both stores but
has no production caller. Installing the controller does not close this gap:
`microvm_controller_install_intent` installs its release, VMM tools, receipt
folder and template unit; it deliberately starts no instance.

The commissioning transaction must use the existing fleet plan/apply boundary:

1. Bind the exact source, selected host and slot to the administrator-authorized
   plan. Withdraw competing launch entry points and establish their quiescence;
   do not infer withdrawal from the new source roster alone.
2. Observe allocation and readiness history, prior controller subjects and any
   outstanding acquisition actors. Missing stores remain unreadable until their
   lineage and initialization are established. Recover existing generations;
   never reset them to zero. Pending acquisition effects must be fenced or drained.
3. Install the allocation substrate and readiness substrate through modeled root
   custody, preserving existing records. The existing allocation-store convergence
   provisions the hold directory; it does not commission a clean cell.
4. Read back the full-slot budget and controller accounting. Install and observe
   the 28 GiB/no-swap cell configuration through convergence.
5. Reuse the lifecycle and slot-network observers to establish every initial
   sanitation fact for this slot. A stopped service or guessed attempt path is
   insufficient. Unknown prior cleanup subjects keep commissioning refused.
6. Call the existing admission with the bound plan and observed facts. Persist
   initial readiness through its store authority under the same exclusive
   commissioning transaction, preventing a competing controller from publishing
   between observation and write.
7. Persist/read back the commissioning provenance separately from the evolving
   readiness generation. Preparation on the second allocation must consume the
   original commissioning authority plus current readiness; it must not attempt
   initial commissioning again. Recover a crash between the two publications
   without overwriting a later incarnation.
8. Only then provide the resulting commissioned slot to
   `workspace_prepare_convergence_from_observations`, using actual image, tool,
   host, gateway and resource-budget policy evidence. Persist the selected plan
   and invoke the existing selected-host fleet convergence entry.

A first run may manually invoke preparation and selected-host convergence, as
requested. It may not supply fixture sanitation, synthesize empty history, or
bypass the durable reservation. Automatic website scheduling remains separate.

## Acceptance

Prove initial commissioning readback and restart recovery; conflicting prior
history and uncertain sanitation must refuse. Then record the real authenticated
request, prepare, reserve, boot, observe SSH and verify resources. Release and
reuse the same physical slot with the larger profile, and exercise prelaunch and
running expiry. Custody, installation, and source tests alone are not VM acceptance.

## Concurrent CI reservation lane

Merged PR #12256 (integrated at main 6571276ce43de56e414bcccfa9623129c7c4e2b4)
adds a CI reservation broker for the same srv1-13 slot. Its broker consumes
`fleet_cell_sanction`; this integration already changes that shared sanction to
customer-executable capacity for workspace-designated slots, so the compile-floor
requirements refuse it before reservation. A second purpose gate is unnecessary.
However, an older installed revision can still have the old designation. Observe
withdrawal/quiescence of that entry point before initial commissioning, and check
the shared refusal against the integrated source; the production broker control now passes. The operator does not
know whether another CI live run is planned. No reservation was performed here.
