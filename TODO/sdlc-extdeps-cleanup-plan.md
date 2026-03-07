# SDLC DAG-First Cleanup Plan

## Goal

Make SDLC execution and testing DAG-first and extdeps-first:

- No SDLC-specific secret fallback logic in Rust.
- No duplicate SDLC flow definitions in multiple DAG layers.
- No SDLC provider transports living under `services/sdlc/providers/*` when an extdeps equivalent can own it.
- Remove stale Cloud Run + `gunbc-sdlc` artifacts that no longer match current binaries/runtime.

This plan is intentionally aggressive (non-prod repo).

---

## Execution Status (2026-03-05)

Completed in this branch:

1. Phase 1: dead-code purge and stale infra manifest deletion.
2. Phase 2: canonicalized away `dsl/pipelines/sdlc.dag` references from tests/snapshots.
3. Phase 3: moved active SDLC providers to `dsl/extdeps/sdlc/providers/*` and rebound profiles.
4. Phase 4: removed `gunbc_lib_cloud_ops` re-exports and direct cloud/gcp-ops deps from `gunbc-dag`.
5. Phase 5: removed S16-S19 cloud mutation live-test block; kept structural cloud wiring check.

Remaining intentional references:

1. `dsl/infra/sdlc/deploy.dag` still models Cloud Run service identity/name (`gunbc-sdlc-worker`).
2. Cloud provider crates may still appear transitively through `gunbc-lib-aws-ops` dependency graph.

---

## Current Catalog

### A) Active Rust surfaces still in the SDLC path

1. `gunbc-dag/src/workflow/catalog.rs` (DSL workflow loading/building, includes `sdlc` aliasing).
2. `gunbc-dag/src/workflow/spec_builders.rs` (`sdlc_workflow_spec()` wrapper).
3. `gunbc-dag/tests/compile_commands.rs` (SDLC compile + structural assertions).
4. `gunbc-dag/tests/sdlc_phase_live.rs` (live/local/cloud mutation tests, includes direct `gcloud` subprocess usage for S16-S19).
5. `gunbc-dag/src/lib.rs` re-exports cloud env helpers from `gunbc-lib-cloud-ops`.

### B) Active SDLC DAG provider modules (non-extdeps namespace)

Bound by `dsl/profiles/sdlc.dag` and/or `dsl/profiles/gist.dag`:

1. `services/sdlc/providers/github_issue_provider.dag`
2. `services/sdlc/providers/file_claim_store.dag`
3. `services/sdlc/providers/file_outcome_ledger.dag`
4. `services/sdlc/providers/file_signal_store.dag`
5. `services/sdlc/providers/inline_artifact_store.dag`
6. `services/sdlc/providers/codex_agent_provider.dag`
7. `services/sdlc/providers/gcp_credential_provider.dag`
8. `services/sdlc/providers/local_credential_provider.dag`
9. `services/sdlc/providers/gcs_claim_store.dag`
10. `services/sdlc/providers/gcs_outcome_ledger.dag`
11. `services/sdlc/providers/gcs_artifact_store.dag`
12. `services/sdlc/providers/pubsub_signal_store.dag`
13. `services/sdlc/providers/stub_providers.dag`
14. `services/sdlc/providers/stub_credential_provider.dag`

### C) Redundant or dead DAG modules (safe delete candidates)

No inbound refs outside own file:

1. `dsl/funcs/sdlc_dispatch_runtime.dag`
2. `dsl/funcs/sdlc_validation_runtime.dag`
3. `dsl/services/sdlc/providers/health_check.dag`
4. `dsl/services/sdlc/providers/llm_agent_provider.dag`
5. `dsl/services/sdlc/providers/rolling_deploy.dag`
6. `dsl/services/sdlc/providers/structured_logging.dag`
7. `dsl/profiles/local.dag` (comment-only placeholder, no `profile` block)
8. `dsl/profiles/cloud_run.dag` (comment-only placeholder, no `profile` block)
9. `dsl/profiles/unit_test.dag` (legacy standalone profile module; not referenced)
10. `dsl/cloud/gcp/credential.dag` (not referenced by active SDLC/gist path)
11. `dsl/pipelines/sdlc_ci.dag` (no inbound refs)

### D) Duplicate SDLC flow surfaces (choose one canonical path)

1. `dsl/workflows/sdlc.dag` + `dsl/funcs/sdlc_worker.dag` + `dsl/funcs/sdlc_stages.dag` (active runtime path).
2. `dsl/pipelines/sdlc.dag` (currently compile-tested but not used by workflow catalog runtime path).

### E) Stale SDLC binary/cloud artifacts

1. `infra/sdlc/Dockerfile` builds/runs `gunbc-sdlc` (bin no longer exists).
2. `infra/sdlc/cloud-run-service.yaml` command uses `gunbc-sdlc`.
3. `infra/sdlc/cloud-scheduler-job.yaml` targets old worker endpoint naming.
4. `lib/design-ops/src/lib.rs` test acceptance string references `--bin gunbc-sdlc`.

---

## Migration/Deletion Plan

## Phase 1: Immediate dead-code purge (low-risk)

Delete items in section C and stale infra in section E.

Acceptance:

1. `rg` finds zero refs to deleted modules.
2. `cargo test -p gunbc-dag --test compile_commands -- --nocapture` passes.
3. `cargo test -p gunbc-dag --test sdlc_phase_live -- --list` passes.

## Phase 2: Canonicalize SDLC DAG topology

Choose one canonical SDLC flow surface and delete the other:

1. Recommended canonical: `workflows/sdlc.dag` + `funcs/sdlc_worker.dag` + `funcs/sdlc_stages.dag` (currently what workflow planner resolves).
2. Delete `dsl/pipelines/sdlc.dag` and remove compile test assertions for it.

Acceptance:

1. `workflow_spec(\"sdlc\")` still builds and includes `sdlc.worker` + `sdlc.report`.
2. No remaining tests compile `pipelines/sdlc.dag`.

## Phase 3: Move active SDLC providers into extdeps ownership

Target: `profiles/sdlc.dag` binds interfaces to extdeps modules/adapters, not `services.sdlc.providers.*`.

Work:

1. GitHub issue provider:
   - Replace `services/sdlc/providers/github_issue_provider.dag` with extdeps adapter over `extdeps/github/issues.dag` service.
2. GCP credentials:
   - Move `GcpWifCredentialProvider` and `LocalCredentialProvider` out of `services/sdlc/providers/*` into extdeps/provider-neutral modules.
   - Keep gist + sdlc profiles bound to the shared module.
3. GCS/PubSub stores:
   - Add transport service sections to extdeps cloud modules where missing (`extdeps/cloud/gcp/storage.dag`, `extdeps/cloud/gcp/pubsub.dag`, optional `cloud_run.dag`).
   - Rebind `ClaimStore`, `OutcomeLedger`, `ArtifactStore`, `SignalStore` to extdeps-backed implementations.
4. Local file stores:
   - Move file-based claim/outcome/signal/artifact implementations into provider-neutral/extdeps-local modules.

Acceptance:

1. `profiles/sdlc.dag` imports zero `services.sdlc.providers.*` modules (except optional temporary stubs during migration branch).
2. All S9-S15 structural assertions updated to new extdeps node prefixes.
3. `cargo test -p gunbc-dag --test compile_commands -- --nocapture` passes.

## Phase 4: Remove cloud-ops/gcp-ops coupling from gunbc-dag crate surface

After Phase 3 rebinding:

1. Remove cloud env helper re-exports from `gunbc-dag/src/lib.rs`.
2. Remove direct `gunbc-lib-cloud-ops` / `gunbc-lib-gcp-ops` dependencies from `gunbc-dag/Cargo.toml` if no longer referenced.
3. Keep these crates only where still required by non-SDLC tools.

Acceptance:

1. `rg` in `gunbc-dag/src` shows no `gunbc_lib_cloud_ops`/`gunbc_lib_gcp_ops` use.
2. `cargo check -p gunbc-dag` passes.

## Phase 5: Tighten tests to DAG-first contract

1. Keep env-gated live tests for local profile (`GITHUB_TOKEN`, `CODEX_API_KEY`) only.
2. If cloud path is intentionally dropped, remove S16-S19 cloud mutation test block from `sdlc_phase_live.rs`.
3. If cloud path is retained, keep S16-S19 but ensure all transports are extdeps-owned modules.

Acceptance:

1. No Rust secret discovery/fallback logic.
2. Test gating aligns with profile contract only.

---

## Recommended Execution Order

1. Phase 1 in one PR.
2. Phase 2 in one PR.
3. Phase 3 split into provider families (GitHub, credentials, stores, local stores).
4. Phase 4 after Phase 3 lands.
5. Phase 5 final cleanup + test tightening.

---

## Quick Verification Commands

```bash
# Build/compile checks
cargo test -p gunbc-dag --test compile_commands -- --nocapture
cargo test -p gunbc-dag --test sdlc_phase_live -- --list

# Guardrails: ensure legacy namespaces are gone when migration completes
rg -n "services\\.sdlc\\.providers" dsl/profiles/sdlc.dag
rg -n "gunbc-sdlc" . --glob '!docs/**' --glob '!TODO/**' --glob '!site/**'
rg -n "profiles\\.(local|cloud_run|unit_test)" dsl --glob '!dsl/profiles/*.dag'
```
