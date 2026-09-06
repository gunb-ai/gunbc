## The finding

FalsifierSelfHostWet is a DEAD CADENCE — falsifier.yml was deleted at 611fd02770/#8283 (2026-08-15). No workflow schedules it, no lane executes it, and witness_cadence_has_scheduled_route returns false for it. Main's own ci_layer_roots reason text admits "falsifier batch 2 no longer executes anything."

Two witnesses (namespace_import_closure_witness_test.dag and interpreter_dispatch_bijection_real_roster_witness_test.dag) were classified onto it and three assertions in witness_admission_test.dag were passing precisely BECAUSE the assert checked classification (FalsifierSelfHostWet => true) rather than execution — permanently green by construction, carrying no information.

This PR rehomes both witnesses to the live LocalRepoWetLane cadence (witness_cadence_has_scheduled_route returns true, enforced by v2.workflow.local_repo_wet_terminal's identity-grain join and finalization).

## Changes (9 files)

1. **ci_layer_roots.dag** — Reclassified 2 WitnessExclusionRows from FalsifierSelfHostWet to LocalRepoWetLane with excl_local_repo_wet_dissolve. Reasons document the rehome and name the re-derivation instrument. WET EXECUTION NOT YET MEASURED as of this commit — first execution is CI's local_repo_wet lane on this PR.

2. **local_repo_wet_terminal.dag** — Added WetScheduledClaim rows for both identities in local_repo_wet_schedule.

3. **witness_admission_test.dag** — Updated 2 inert assertions from FalsifierSelfHostWet => true to LocalRepoWetLane => true (can now go RED for a real reason). Renamed test functions to reflect the new cadence.

4. **floor_route_gap.dag** — Moved namespace_import_closure_receipt_holds from bare roster to LOCATED/expectation form (operation: Run, ground: NoMockResponse). interpreter_dispatch_bijection_real_roster_red_holds was already in expectations.

5. **wet_receipt_enrollment.dag** — Removed both identities from falsifier_self_host_wet_template_entries. Updated dispatch_bijection_real_roster_red_enrolled_on_falsifier_self_host_wet to return false (rehomed identity).

6. **witness_exclusion_reconciliation_test.dag** — Converted the enrollment assertion to a REGRESSION CONTROL (asserts the identity stays off the falsifier roster; green now, reds on re-enrolment).

7. **Prose updates** in namespace_import_closure_witness_test.dag, interpreter_dispatch_bijection_real_roster_witness_test.dag, namespace_import_closure_behavioral_transport.dag (nic_doc), wet_receipt_enrollment.dag (INTERIM home comment).

## What CI should show

- Both identities observed passed on the local-repo wet lane (planned=/executed=/verdict= lines, not standing=unobserved or standing=measurement_unreached)
- Route gap accounting shows enrolment beside a route (not a lowered count)
- Two identities gain an executing route, nothing else loses one

Dashboard node: adhoc-cf84ab88-8dc
