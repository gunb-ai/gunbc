# Printer converge: legacy-hold fence vs. the root:root parent re-own (srv1)

Work item `node://adhoc-0dbfbe96-bf6`. Receipt: fleet-converge run `42692296757` (main `b882c902`,
mode=printer, host=srv1, printer-01). This document records the verified diagnosis, the repair
derivation, the implementation plan, and the two decisions that gate landing the code. It is NOT a
design authority; DESIGN.md is. It is a hand-off.

## Diagnosis (verified in code, not just the hypothesis)

Pass order — `gunbc.fleet.print_converge.converge_print`:
1. `ensure_printer_state_directories`
2. `printer_state_move` (the state-layout migration; `gunbc.fleet.printer_state_layout`)

`ensure_printer_state_directories` (print_converge.dag), in order:
1. `host_effect_realize_command_over_transport(... executor_privileged_operation_elevated_command(EnsureSupplementaryGroupMembership ...) ...)` — **elevated**.
2. `printer_state_parent_identity_leg_on_host(tree)` — the #15948 re-own of the printer-starts PARENT to
   `root:root 00755` via an **elevated** `install` (`gunbc.fleet.printer_state_parent_reown` /
   `_host`). Fires only while the parent reads the pinned prior identity; a non-matching identity ends the
   pass ("stop, not delete"). srv1 is now at `root:root 0755`, so the re-own has run (or the fold
   established it); the pass REACHED the layout step, so `ensure` passed and the parent reads the DECLARED
   root:root identity (provider-declared, non-adopting).
3. fold of `printer_state_directories` — each child directory, **including the parent**, ensured via
   `live_deploy_apply_ensure_directory_on_host` → `executor_privileged_operation_elevated_command(EnsureOwnedDirectory ...)` —
   **elevated**.

**Consequence:** whenever `printer_state_move` runs, the printer-starts parent is at the declared
`root:root 00755` identity — readable/traversable by the job user, **not writable** by it.

The failing link — `printer_state_move` → `legacy_writer_exclusion_causes` → `fence_taken` →
`printer_execution_hold_fence_begin` (`gunbc.fleet.printer_execution_hold`) →
`file_hold_acquire(root: printer_starts_path, ...)` (`gunbc.durable_exclusive_hold_file_store`) →
CAS commit → `Filesystem.WriteCreateNew` in the **parent** (unprivileged). The job user cannot write the
parent → `FileHoldAcquireStoreUnavailable` → `HoldFenceRefused { cause: "the old printer hold store is
unavailable; nothing was moved" }` → `PrinterStateMoveRefused { cause: "the legacy printer controller is
not excluded; nothing was moved: <fence cause>" }` → `ConvergePrintRefused { "Printer state layout not
converged: <cause>" }`. This is the receipt string verbatim. Nothing printed.

**Principals.** Every other write into the root-only parent runs ELEVATED through the grant projection:
the re-own (`install`), each child-dir create (`EnsureOwnedDirectory`), and the parent read
(`deploy_apply_directory_observe` / privileged `stat`/`ls`). The fence's create-new in the parent is the
**only** unprivileged write into that parent.

## Earliest unjustified boundary

The fence assumes the job user can still write the printer-starts parent. #15908/#15948 revoked that write
(correctly — it is what closes the symlink-swap hole). That assumption is the first link in the model that
is not justified by the rest of the model.

Candidate repairs considered (per DESIGN.md `earliest_unjustified_boundary`):
- **(a) move before the re-own** — RULED OUT. srv1's parent is ALREADY `root:root 0755` (stern-bat-401
  readback; and `ensure` re-establishes it every pass). Reordering cannot un-own it, and the required
  witness drives an already-root-owned parent, which stays red under (a).
- **(c) fence needs no parent write** — RULED OUT. The fence is load-bearing: it occupies the exact
  parent slot `fleet-print-controller.<n>` so a pre-lock legacy controller cannot re-acquire it; the
  generations are copied to the child store but the held head is intentionally LEFT held in the parent.
  Dropping the parent write deletes that exclusion — the legacy controller sees a free slot again.
- **(b) the fence's parent write goes through the privileged grant projection** — THE DERIVATION. It is
  what every other parent-internal write already does; it keeps the safety property; it adds no privilege
  beyond writing into the same root-only parent the pass already writes into elevated.

**fierce-bee-888 approved (b)** with constraints: keep the shared unprivileged `file_hold_acquire`
untouched; make the new op as narrow as the re-own leg (one exact path pattern, create-new only, carries
its own end); the grant exists only for this migration and its arm becomes unreachable once no host has
legacy hold entries in the parent (write the dissolution on the carrier, like `printer_state_parent_reown`).

## Implementation plan (option b) — READY, NOT YET LANDED

1. `gunbc.executor_privileged_operation`: add ONE narrow op that create-news the held generation file in
   the root-only parent (create-new only, one exact path shape, `RunsAsRoot`), with its `argv` + `principal`
   projection and a migration-scoped `dissolution` on its carrier (mirroring the re-own). Add its sudoers
   grant ROW ONLY WHEN APPROVED (see Blocker B).
2. `gunbc.durable_exclusive_hold_file_store`: ADD (do not modify `file_hold_acquire`) a publish-capable
   commit sibling that mints the plan through the existing (unprivileged) prepare/read — reads succeed on a
   0755 parent — and routes ONLY the final create-new write through an injected publisher, reusing the
   store's own slot/generation encoding and CAS precondition re-verification (do NOT reimplement the slot
   encoding outside the store: a slot written by one authority and read by the other decodes as garbage).
3. `gunbc.fleet.printer_execution_hold`: `printer_execution_hold_fence_begin` keeps the unprivileged
   `file_hold_acquire` and, on `FileHoldAcquireStoreUnavailable` (the parent is the declared root-only
   identity), performs the parent create-new through the store's publish-capable sibling with the privileged
   publisher. `file_hold_acquire` itself is unchanged.
4. `gunbc.fleet.printer_state_layout` / `print_converge`: thread the publisher (function value, like the
   re-own's `apply`) from the host entry down to the fence; default = the privileged projection; the wet
   witness injects a local stand-in (the re-own witness's `apply` pattern — no real `sudo` in wet tests).
5. Witness: `dag/test/claim/printer_state_layout_rooted_parent_witness.dag` (added here) — drives an
   already-root-owned (chmod 0555) parent with a residual legacy hold and asserts the move converges (RED
   on main), plus the store-unavailable fact pinned at the fence primitive. Green half exercises the
   injected publisher per the re-own DI convention.

## Revision after fierce-bee-888's two questions (repo-only evidence; srv1 untouched)

Q: does a runnable FLAT-layout printer controller still exist that could re-acquire `fleet-print-controller.<n>`
in the parent? **No**, per the repo's deployed-unit records:
- The current controller acquires its controller hold ONLY in the SPLIT store —
  `printer_execution_hold.dag:62` (`printer_execution_hold_path` = `execution-hold` member) and `:156-159`
  (`printer_execution_hold_store` -> `PrinterExecutionHoldStore { root: execution-hold }`); the runtime dance
  (`:338-360`) acquires at `store.root`, never the flat parent.
- The ONLY current acquirer of the flat parent slot is the migration fence itself (`:113`
  `file_hold_acquire(root: printer_starts_path, ...)`). No other non-test code acquires the flat slot.
- No print/fleet-controller `.service` or `.timer` exists anywhere in the repo (only
  `gunbc-site-pxe-edge(.http).service`, `gunbc-ghrunner`, `gunbc-microvm-slot@<slot>.service`, and a BMC
  `phosphor-pid-control.service`). `provisioning/srv1/` holds only `gunbc-ghrunner.sudoers`.
- The controller is per-actuation, not resident (`printer_workflow_run.dag:73-81` files an OPERATOR card to
  confirm "no fleet controller is driving the printer" before a hold is released).
=> srv1's `fleet-print-controller.1/.2` are inert pre-split residues; no runnable unit re-acquires them, and a
non-root principal cannot re-create them now the parent is root-only. **The fence protects against nothing that
can run.**

Revised repair (preferred; pending fierce-bee-888 go / operator confirmation of srv1 unit state):
**rm-after-copy.** Once the legacy generations are copied + byte-verified into the SPLIT store, retire the
legacy parent entries with the ALREADY-GRANTED privileged `rm` (`executor_privileged_operation.dag`
`RemoveDirectoryTree` -> `sudo rm -rf`, `:79/:172-174/:456`, RunsAsRoot), and drop the fence's parent
create-new for the root-only-parent case. This needs NO new sudoers grant, NO edit to the shared
`durable_cas_file_store` write path, and NO create-new primitive. It is symlink-safe: `unlink` does not follow
a symlink, so `rm` of a legacy entry removes the link, never its target.

Rejected realization (mv -n): `rename(2)` moves a symlink source AS a symlink, and the store's read-back
`Filesystem.Read` FOLLOWS it, so a symlinked staged source can "verify" while the persisted slot is a symlink —
re-opening the #15908 symlink hole. Non-job-writable staging is the only guard, but the unprivileged fence then
cannot populate it, so `mv -n` from staging is not sound without a second privileged act. mv -n is dropped.

Residual risk: dropping the fence's active exclusion is justified only by "no runnable flat-layout controller."
Repo records are the allowed lens; if srv1 runs a controller unit not captured in the repo/provisioning, the
protection is lost — the operator should confirm srv1 has no enabled printer-controller unit/timer before the
fence is dropped. Otherwise keep the fence and pursue a root-owned O_EXCL publish of model-derived bytes (the
security-owner's call).

## Blockers (both are above this session's authority; neither is guessable)

- **Blocker A — sudoers grant row (operator's call).** A new sudoers-granted privileged operation is the
  operator's call; fierce-bee-888 has put it to them. The plan above does NOT add the grant row. The op is
  inert until it lands; the on-host elevated path cannot be enabled without it. This is a draft PR by design.
- **Blocker B — the elevated create-new trust/staging decision (security owner).** The sudoable tool set
  here is `{install, rm, stat, ls, mv, sha256sum}`; none is a single-argv "create-new a file with these
  bytes" that does not overwrite. The re-own and `live_deploy_apply_ensure_directory_on_host` only
  `install -d` directories. `InstallFile`/`SystemdUnit` `install` a file from a STAGED source, and DESIGN
  forbids a job-writable staged source reaching a root-owned file (that is the escalation). For the fence
  the payload is a model-derived constant (`held|printer-state-layout-move`), and the store re-reads +
  byte-verifies after the write, so a tampered staged source is caught — but pinning that as the sanctioned
  realization (e.g. `mv -n` from a model-staged source + CAS read-back verification, RetireUnitFragment
  precedent) is a security-owner call, not a guess. This session will not land the shared-store publish seam
  without that decision.

## Live host

srv1 was NOT touched (no ssh writes, no fleet-converge dispatch). Recovering srv1 needs a LIVE step that
this session must not perform: it needs the privileged create-new of the fence generation in the root-owned
parent (i.e. the fix, once built and granted). Until then srv1 with legacy hold entries stays wedged. That
write-up is part of this hand-off and goes to the operator through fierce-bee-888.

## Reproduction

`dag/test/claim/printer_state_layout_rooted_parent_witness.dag`:
- `w_a_root_owned_parent_makes_the_plain_acquire_report_store_unavailable` — green today; pins the fact that
  the unprivileged acquire the fence uses cannot write a root-owned parent (the wedge). Stays green through
  the fix (the fix ADDS the privileged path; it does not widen the unprivileged acquire).
- `w_a_root_owned_parent_with_a_legacy_hold_still_converges_the_move` — the acceptance contract: RED on main
  (the move refuses), GREEN on the (b) fix (the move completes once the parent create-new goes through the
  privileged projection).
