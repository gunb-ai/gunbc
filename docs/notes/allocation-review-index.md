# Allocation, fabric storage, and approval integration review

This is a draft snapshot of the isolated `codex/allocation-demo` integration tree,
based on `e66a6a65ae5821c8c215bdfdc26b73dbbc1276d6`. It includes ports of the
session-bearing srv2 site's auth and tracker UI alongside the new implementation.
It is not deployment-ready. The serving checkout was not edited.

## Review areas

| Area | Main entry points | Evidence / remaining work |
| --- | --- | --- |
| Fabric storage and daily workspace migration | `dag/gunbc/fabric/fabric_storage_state_serve.dag`, `fabric_storage_state_auth.dag`, `fabric_storage_file_store.dag`; `dag/gunbc/roadmap/roadmap_event_carrier.dag`, `roadmap_event_migration.dag`, `roadmap_task_migration.dag` | Temporary-store CAS/retry, raw-history preservation, migration recovery/gating, and graph checks passed. See [migration receipt](allocation-demo-state-migration-receipt.md). No production import or cutover. |
| Allocation request and reservation | `dag/gunbc/workspace_allocation_admission.dag`, `workspace_allocation_reservation.dag`, `workspace_offer_producer.dag`, `workspace_prepare_convergence.dag` | Sized-request witnesses and combined preparation/controller typecheck passed. Eligible slot and authoritative commercial/commissioning inputs remain required. |
| Runtime, SSH readiness, release/recovery | `dag/gunbc/runner/workspace_microvm.dag`, `workspace_microvm_access.dag`, `runner_microvm_slot_controller.dag`; `dag/gunbc/workspace_allocation_lifecycle.dag`, `workspace_acquisition_invocation.dag` | Workspace runtime 5/5, existing CI lifecycle/launch 54/54, readiness 20/20, acquisition cancellation 3/3 passed. See [runtime handoff](workspace-runtime-handoff.md) and [lifecycle notes](allocation-lifecycle-integration.md). No VM boot or external SSH acceptance. |
| HTTP/session and page integration | `dag/gunbc/roadmap/workspace_allocation_http.dag`, `workspace_allocation_routes.dag`, `roadmap_auth_routes.dag`, `roadmap_task_routes.dag`; `src/v1/stage0/src/cli_run.rs` | Focused wire/admission and Rust transport tests passed. Integrated serve validation, complete mutable-task UI projection, CSS digest, and browser acceptance remain unfinished. |
| Approval to GCP / secret workflows | `dag/gunbc/auth/approved_gcp_access.dag`, `approved_secret_grants.dag`, `approval_run_identity.dag`; `dag/gunbc/fabric/fabric_storage_state_provision.dag` | Approval utility 3/3 and shared secret-grant checks 3/3 passed separately. Adapters exist for fleet accessors, Spark, and the GitHub App key. Credential producer, remaining consumers, broker enrollment, and live validation remain unconnected. See [approval notes](approval-gcp-access.md). |

## Deployment and integration limits

- The srv2 allocation page remains draft-only. No integration deployment, secret
  provisioning, privileged controller/image installation, or production migration
  was performed by this work.
- The built 8 GiB workspace image is a local artifact, not committed to Git.
  Its current SHA256 and build evidence are in the runtime handoff.
- Initial slot commissioning, real access-path observations, and the complete
  allocate → SSH → release → larger allocation → expiry acceptance loop remain.
- An elected acquisition actor whose socket operation may still be queued retains
  its obligation. Settlement needs the authoritative socket fencing/drain contract.
- Protected state storage must be reconciled with the separately owned restricted
  local socket work before deployment. This draft does not claim that integration.
- Migration cutover must quiesce the old Git writer, pin/recheck the final source,
  import/seed/verify, and then switch readers. A pinned-source audit is not a write
  freeze and does not preserve writes made after that pin by itself.
- Focused validation used byte-identical dependency snapshots with source hashes,
  bounded memory and zero swap. Those results are not full landing-suite results
  or measurements of deployed controller memory. Broader serve and secret-policy
  checks encountered unresolved dependencies; no full-suite pass is claimed.

The other notes contain chronological checkpoints; this index states the review
snapshot's overall status. No merge or deployment is requested by this draft.
