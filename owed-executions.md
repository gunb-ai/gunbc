# Owed executions: claims that landed on gunbc main without having run

Maintained by silent-bee-845, the daily integration manager. CI's required job (v2-bootstrap) only compiles the seed and runs the two emitted closures; it executes no `.dag` witness and no v1/stage0 unit test. Every item below landed on review plus a seed compile, and it is owed one real execution once an executor is named.

## Continuation stack (not landed; blocked on an executor)
- #15784 -> #15788 -> #15943 -> #15791 -> #15792 -> #15804: no witness has ever executed. #15791 holds REQUEST_CHANGES (review 92187) that only an executed run clears. #15784 still owes a supervisor inhabitance claim and a CLI verdict witness (reviews 91801, 91883).

## Landed restores (2026-11-09)
- #16125 (50f0644d): test.claim.host.host_effect_apply_witness. It is offline-local-only (excluded from CI enumeration). Its concern row host_effect_apply_witness_import_block_absent closes only when a scheduled route executes it.
- #16038 (9193e5fc): all 17 durable_attempt_controller claims, the attempt_lifecycle witnesses, and both terminal controls. attempt_lifecycle_wet_witness is excluded as LocalRepoWetLane, so the stale-owner-after-transfer CAS control has never run anywhere.
- #16020 (2f11e50b): harness_shell_job_tool_route, plus both wet witnesses (spawn/hold/reap/group-kill/backstop). The tool-route header cites "main seed refuses host_effect" as its NO_OBSERVATION trigger. That is STALE after #16099 and #16125: re-run before trusting or deleting it.
- #15710 (d65a1941): the stage0 Rust tests (record refusal before any request; URL credential-position refusal) and both redaction witnesses (heal_publication_refusal_redaction_witness; the six fake-secret claims in github_effect_perform_witness).
- #15648 (6498ee29): the pattern_binder_capture_native label (a plain-path regression guard only). The rc-ref branch rung is unclaimed; a recursive or shared-type fixture is owed once both legs can be built.
- #15687 (c5ec5901): the closure_emission and nested_arm_pattern_position claims (receipts were srv2 runs at the archived head and have not been re-executed).
- #15614 (b36c11f3): the touched claims and the witnesses of the repointed modules.
- #15567 (260cd3a1): printable_assembly_selection_witness, duct_card_catalog_witness, gpu_duct_witness, card_cooling_route_witness.
- #16042 (a2dddbe7): the substring seam is inert until heal publishes the primitive_substring HostRealizedSeam row from main.
- #15583 (6ea348f7): chunk-record optionality needs a real captured stream.
- #15894 (2e18b541): the service_operation_unrealizable claim.
- #15695 (7606bd82): the condition_position_record_parse claims.
- #15560 (e923d5c4): heal was RED at landing (two-generation retirement; operator default, msg_76ed4315). The next PR that touches generation inputs must heal green.

## Deferred by authors (recorded, not dropped)
- #16038: ControllerCapacityWaiting/Cancel silent overwrite; DurableHoldSubject's two wire forms; the witness lease() helper bypasses LeaseNotObserved. Remote-build PR 4/5 are tracked by concern controller_executor_fence_handoff.
- #16020: thread-bind admission (d080f31e) waits on #15804; hand-in coupling (main has no HandInTool).
- #15710: gh-api stderr passthrough; raw-body sites heal_publisher_provision::rest_refusal_text and printer_credential_migration (left alone).
