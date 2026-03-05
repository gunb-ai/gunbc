# SDLC Scenario Readiness

Status: Draft  
Date: 2026-03-05  
Scope: Set realistic expectations for "can we use SDLC soon?" and define go/no-go gates.

## 1. Purpose

This document defines practical usage scenarios for the SDLC pipeline and the proof required for each one.  
It is not a design document. It is an activation checklist.

## 2. Current Baseline (Verified)

As of 2026-03-05:

- `make ci` passes in this branch (`tools.ci::ci.success: true`).
- SDLC compile and dry-run coverage exists in `gunbc-dag/tests/compile_commands.rs`, including:
  - `builds_sdlc_worker_dsl_graph`
  - `builds_sdlc_stages_dsl_graph`
  - `builds_sdlc_workflow_dsl_graph`
  - `builds_sdlc_worker_unit_test_profile_dsl_graph`
  - `builds_sdlc_worker_local_profile_dsl_graph`
  - `dispatch_sdlc_unit_test_profile_dry_run_completes`
- Env-gated local live tests exist in `gunbc-dag/tests/sdlc_phase_live.rs`:
  - `s9_issue_provider_live_operations_against_github`
  - `s10_local_profile_credential_wiring_compiles_and_authenticates`
  - `s11_local_profile_design_stage_e2e`
  - `s12_to_s15_local_pipeline_wiring_is_present`

What is still not continuously proven in CI:

- Mutable end-to-end local runs (`s11`) under real secrets.
- Cloud profile mutation path (claim/outcome/artifact/signal in hosted infra).
- Multi-worker cloud contention behavior.

## 3. Scenario Ladder

### Scenario A: Demo Safe (Now)

Goal: Prove SDLC is structurally valid and runnable in hermetic mode.

Profile:

- `profiles.sdlc.unit_test`

Required proof:

- `make ci` green.
- SDLC compile tests green.
- Unit-test dry-run dispatch completes.

Use case:

- Demos, refactors, non-mutation development.

### Scenario B: Local Pilot (Soonest useful mode)

Goal: Process one real GitHub issue through design flow with local profile.

Profile:

- `profiles.sdlc.local`

Required secrets/env:

- `GITHUB_TOKEN`
- `CODEX_API_KEY`
- `SDLC_GITHUB_OWNER`
- `SDLC_GITHUB_REPO`
- `SDLC_TEST_ISSUE_NUMBER`
- `SDLC_LLM_PROVIDER`
- `SDLC_LLM_MODEL`
- `SDLC_ALLOW_MUTATION=1`

Required proof:

- `s9`, `s10`, and `s11` pass in one controlled run.
- Issue receives expected design artifact/comment.
- Issue labels advance to design flow labels.
- Outcome artifacts are written under `target/sdlc/outcomes`.

Use case:

- Real pilot with one repository and explicit operator oversight.

### Scenario C: Local Flow Pilot (Multi-stage)

Goal: Validate the full local stage chain on a small issue set.

Profile:

- `profiles.sdlc.local`

Required proof:

- Repeated dispatch runs move issues across expected labels (`idea -> ... -> done`).
- No duplicate terminal transitions for the same `(issue, stage, run_key)`.
- Stage outcomes are recorded for each transition.

Use case:

- Team trial before any cloud rollout.

### Scenario D: Cloud Canary

Goal: Run one cloud worker with cloud profile and verify cloud-backed state.

Profile:

- `profiles.sdlc.cloud_run`

Required infra:

- GCP project with expected Secret Manager, GCS, and Pub/Sub resources.
- Credential flow for profile-bound `GcpWifCredentialProvider`.

Required proof:

- Worker compiles and runs with cloud profile.
- Claim/outcome/artifact operations persist in cloud stores.
- Signal emit/consume path works for canary traffic.
- No stuck claim lease beyond TTL during canary window.

Use case:

- First hosted environment validation.

### Scenario E: Cloud Team Beta

Goal: Operate multiple workers safely on live queue load.

Profile:

- `profiles.sdlc.cloud_run`

Required proof:

- Parallel workers do not double-process the same `(issue, stage)`.
- Conflict handling and retries are observable and bounded.
- Throughput and error rate stay within agreed SLO during trial window.

Use case:

- Pre-production confidence gate.

## 4. Recommended Near-term Target

If "use any time soon" means this month, the realistic target is:

1. Make Scenario B repeatable in one repository.
2. Then run Scenario C for a small issue batch.
3. Only then decide whether to invest in Scenario D immediately.

This keeps momentum while avoiding premature cloud hardening.

## 5. Go / No-Go Template

For each scenario, decide with three questions:

1. Reliability: Did all required proof checks pass twice in a row?
2. Safety: Is rollback/manual intervention clear for failure paths?
3. Cost: Is operator effort acceptable for current stage of adoption?

If any answer is "no", remain on the previous scenario and fix gaps there.

