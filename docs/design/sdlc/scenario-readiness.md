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
  - `sdlc_worker_uses_provider_auth_modules_and_not_legacy_bindings`
  - `dispatch_sdlc_dry_run_completes_without_legacy_bindings`
- Env-gated local live tests exist in `gunbc-dag/tests/sdlc_phase_live.rs`:
  - `s10_local_profile_binds_real_local_providers`
  - `s11_local_profile_design_stage_e2e`

What is still not continuously proven in CI:

- Mutable end-to-end local runs (`s11`) under real secrets.
- Cloud profile mutation path (claim/outcome/artifact/signal in hosted infra).
- Multi-worker cloud contention behavior.

## 2.1 Credentialing Postmortem (No-Fallback Direction)

As of 2026-03-05, credentialing is not yet fully modeled end-to-end for SDLC:

- Active workflow callers now use provider-local auth modules in `dsl/extdeps/github/auth.dag` and `dsl/extdeps/llm/auth.dag`, not the deleted shared helper path.
- `profiles.sdlc.local` has been reintroduced only as a temporary compatibility path so the current branch can bind local SDLC providers for real-mode proof.
- That temporary profile is currently pinned to `gunb-ai/integration_testing`.
- The live harness fetches the GitHub token through the normal Secret Manager path, then injects it into the temporary profile’s `env("GITHUB_TOKEN")` binding. Codex auth remains environment-inherited rather than flowing through `CredentialProvider`.
- Profile `secret("...")` bindings still lower to literal `secret:<name>` references on `res:credential`; transport execution does not yet resolve those refs through `CredentialProvider`.

Conclusion:

- Do not treat the temporary local profile as the target architecture.
- Treat strict modeled credentialing as an explicit migration track: structural auth requirements plus concrete binding/link artifacts, with no workflow-local fallback logic.

## 3. Scenario Ladder

### Scenario A: Demo Safe (Now)

Goal: Prove SDLC is structurally valid and runnable in hermetic mode.

Profile:

- none on the current branch; the worker compile/dry-run proof uses the no-profile path

Required proof:

- `make ci` green.
- SDLC compile tests green.
- Worker dry-run dispatch completes with auto-mocked boundaries.

Use case:

- Demos, refactors, non-mutation development.

### Scenario B: Local Pilot (Soonest useful mode)

Goal: Process one real GitHub issue through design flow with local profile.

Profile:

- `profiles.sdlc.local`

Required secrets/env (current state, transitional):

- `ANTHROPIC_API_KEY` or `OPENAI_API_KEY` (matching `SDLC_LLM_PROVIDER`)
- `SDLC_LLM_PROVIDER`
- `SDLC_LLM_MODEL`
- `SDLC_ALLOW_MUTATION=1`
- working `gcloud` access to `gunbai-secrets/github-token`

Required proof:

- `s10` and `s11` pass in one controlled run.
- Ephemeral issue receives expected design artifact/comment.
- Ephemeral issue labels advance to design flow labels.
- Ephemeral issue is closed during test cleanup.
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

Current tactical note:

- The branch uses a temporary `profiles.sdlc.local` compatibility path to make Scenario B runnable now.
- Delete that path once compiler-side concrete binding/link cleanup lands.

## 5. Go / No-Go Template

For each scenario, decide with three questions:

1. Reliability: Did all required proof checks pass twice in a row?
2. Safety: Is rollback/manual intervention clear for failure paths?
3. Cost: Is operator effort acceptable for current stage of adoption?

If any answer is "no", remain on the previous scenario and fix gaps there.
