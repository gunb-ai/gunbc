# S0 (#14292) rework — merry-fox-82 side-chat HOLD, 2026-10-18T07:00Z

Head 7b293a23736 was tally-ready (review 81944 APPROVE, CLEAN, CI green) and is now
unfrozen for rework. merry-fox-82 is the merge authority; land asks go to them, never
the operator. Report: run with 2>&1, send the head, freeze. #14275 (bold, 10/10
M1–M11) and every other re-measure waits on this.

## ADMISSIBLE as-is (do not "fix")
- Rung-2 runtime route validation — but RouteAuthority is gone, so delete the PR-body
  claim that the illegal pairing fails to typecheck.
- S2 probe / S9 actuator frontiers with BoundaryInhabitation Uninhabited.

## Blocker 1 — FALSE READY
`network_entries_in` filters out non-network entries and THEN takes the head, so
BootOrder `[Hdd, Pxe]` gives Ready/Pxe when it needs preparation. Also BootOrder
`[Pxe]` with options `[UefiHttp]` gives Ready, because the decision only checks that
*SOME* network target exists — and `good_observation()` is exactly that inconsistent
case (`[PXE, NVMe0]` vs options `[http]`).

REPAIR: keep the original order; join the head entry to the enumerated, ENABLED option
by IDENTITY (`BootOptionReference`), never by kind or label. Fix the happy fixture.
Witness `[disk, PXE]` vs `[PXE, disk]`, plus an ordered option missing from the
enumeration.

Stage A design:
- BootOrder entries carry `reference` ONLY (kind arrives through the join ⇒ the false
  Ready becomes unwritable — DESIGN 5 "stronger than catching a wrong state is making
  it unwritable").
- `BootOption { reference, kind, label, enabled }`.
- `HeadJoinStanding = HeadJoinsEnabledOption | HeadJoinsDisabledOption
  | HeadOptionNotEnumerated { reference, enumerated_count } | HeadOrderEmpty
  | HeadEnumerationEmpty`.
- The "network available" classifier requires kind network AND enabled.
- `decide` / `ready_or_refuse_on_nics` take the OBSERVATION, not independent
  devices/options/nics.

## Blocker 2 — SEALS BYPASSED
`ready_evidence`, `ready_or_refuse_on_nics` (head Hdd + Unobserved axes gives Ready) and
`decide` (independent fields beside the observation) are unrestricted, and
`NetworkBootAttempt` is an ordinary record passable to
`firmware_network_boot_attempt_receipt`.

REPAIR: `sole_constructor` + `admit_callers` on the whole validated result and on every
value-returning path, so legitimate evidence cannot carry a contradictory conclusion.
Same for the attempt. Add outside-caller refusal probes, including reuse of legitimate
evidence with a swapped conclusion.

Stage B corpus idioms: `dag/test/claim/sole_constructor_completeness_audit_probe.dag`
(forms f1–f4 via `compile_dag_diagnostic_census`, class `SoleConstructorViolation`);
fixture `dag/test/fixture/sole_constructor_variant_probe/definer.dag`; union
`sole_constructor` precedent `gunbc.model.population:132`,
`gunbc.host.managed_host:186`; `admit_callers` exemplar `dag/extdeps/shell/exec.dag:43`
(no test declaration may be admitted — a witness goes through a production bridge).

## Blocker 3 — UNBOUND ROUTE
`route_for_blocking` takes only the blocker, so `PointUefiBootNext` always carries the
literal `"uefi-boot-next"` rather than the observed `BootOptionReference`. There is no
firmware-support evidence, and the observed bios is unread. NetworkStack and
NicBootSelection attribute names cite the generic ComputerSystem schema, which does not
establish them per firmware.

REPAIR: split the abstract mechanism catalog from a unit-bound route that carries the
observed option reference plus grounded mechanism support, or return an explicit
unresolved-binding refusal. Never let a missing option select `EnableNetworkStack`
without evidence. `PointUefiBootNext` reads back `BootNext` (not `UefiBootNext`);
`EnableNetworkStack`'s BootOrder readback goes AFTER the reset.

Stage C also: actually consult the observed `bios` (today `decide` takes it and never
reads it).

## Blocker 4 — DONE
`a_frontier_fires_only_for_the_declaration_it_names` was missing `&&` before
`declaration_ref_eq`, so the Fired assertions were discarded (a block returns its last
expression). Fixed in the tree; the mutation "dissolution_status ignores present_decls"
must show RED.

## Then
Full mutation suite (12+ rows incl. dissolution_status-ignores-present_decls RED and the
swap-conclusion probe RED), post the receipt, send the head to merry-fox-82, FREEZE.

Deferred, do not fold in: local branch `sb-unify` (Secure Boot carrier → extdeps) —
lands as a separate change after this merges (work item node://adhoc-5c725b77-b6a).
