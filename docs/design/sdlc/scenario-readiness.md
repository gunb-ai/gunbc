# SDLC Scenario Readiness

Status: Draft  
Date: 2026-03-05  
Scope: Set realistic expectations for "can we use SDLC soon?" and define go/no-go gates.

## 1. Purpose

This document defines practical usage scenarios for the SDLC pipeline and the proof required for each one.  
It is not a design document. It is an activation checklist.

`tasks.md` Phase H is the single active planning surface for SDLC on this branch.
Use this document as a scenario/readiness input when refining that lane.

If another SDLC doc still assumes profile-based architecture as the long-term
plan, treat that as historical unless it is explicitly reconciled in `tasks.md`.

## 2. Current Baseline (Verified)

As of 2026-03-05:

- `make ci` passes in this branch (`tools.ci::ci.success: true`).
- SDLC compile and dry-run coverage exists in `gunbc-app/tests/compile_commands.rs`, including:
  - `builds_sdlc_worker_dsl_graph`
  - `builds_sdlc_stages_dsl_graph`
  - `builds_sdlc_workflow_dsl_graph`
  - `sdlc_worker_uses_provider_auth_modules_and_not_legacy_bindings`
  - `dispatch_sdlc_dry_run_completes_without_legacy_bindings`
- Env-gated local live tests exist in `gunbc-app/tests/sdlc_phase_live.rs`:
  - `s10_local_profile_binds_real_local_providers`
  - `s11_local_profile_design_stage_e2e`

What is still not continuously proven in CI:

- Mutable end-to-end local runs (`s11`) under real secrets.
- Concrete binding/link replacement for the temporary local compatibility profile.
- Hosted mutation path (claim/outcome/artifact/signal in cloud infra).
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
- Do not plan around a user-facing `--profile` CLI coming back unless `tasks.md` changes; the current temporary path is compiler-internal and test-driven.

## 3. Operating Modes

These are the four modes that matter for planning and adoption. Do not collapse
them into one “SDLC works” bucket; each has different proof and different blockers.

| Mode | Runtime surface | Real effects | Binding path | Minimum proof | Current status |
|------|-----------------|--------------|--------------|---------------|----------------|
| Local dev testing | developer machine | none | no-profile compile path + auto-mocks | `make ci` + SDLC compile tests + worker dry-run | ready and should stay the default engineering path |
| Local real testing | developer machine | real GitHub/LLM mutations with explicit opt-in | temporary `profiles.sdlc.local` compatibility path | `s10` + `s11` | partially proven, still operator-driven |
| Remote dev testing | hosted worker in non-prod infra | real cloud mutations in dev/staging only | hosted concrete binding/link artifacts | S-16 through S-18 canary proof | not yet proven |
| Remote real runs | hosted fleet on real queue/repo | full live mutations | hosted concrete binding/link artifacts + fleet safety | S-19 + operational runbook | not ready |

### 3.1 Local Dev Testing

Goal: keep compiler and workflow development fast, hermetic, and safe by default.

Actually needed:

- no-profile compile path must keep working
- worker dry-run must complete with mocked boundaries
- SDLC compile tests stay in normal CI
- no real secrets or mutable external systems required

Use this for:

- compiler changes
- DSL refactors
- prompt/schema iteration
- most day-to-day development

### 3.2 Local Real Testing

Goal: prove the pipeline can mutate real systems from a developer machine before
investing in hosted rollout.

Actually needed:

- temporary `profiles.sdlc.local` compatibility path until concrete bindings replace it
- real GitHub token path
- real LLM API key path
- explicit mutation gate (`SDLC_ALLOW_MUTATION=1`)
- local file-backed claims/outcomes/signals/artifacts
- repeatable `s10` and `s11` runs against a controlled repo

Recommended scope:

- one dev/integration repo only
- one operator at a time
- ephemeral issue creation and cleanup

### 3.3 Remote Dev Testing

Goal: prove hosted execution in a bounded non-production environment.

Actually needed:

- hosted concrete binding/link artifacts for GitHub, GCS, Pub/Sub, and credentials
- non-production cloud project
- non-production repo or clearly segregated issue namespace
- single-worker canary first
- deploy, health, claim/outcome, and signal proof
- drain/rollback procedure before widening traffic

This mode should not target the real repo/queue first. Its job is to validate
hosted plumbing, not to prove full operational readiness.

### 3.4 Remote Real Runs

Goal: run the hosted worker fleet against the real queue with acceptable safety.

Actually needed:

- everything from remote dev testing
- S-19 multi-worker CAS/conflict proof
- clear ownership for rollout, drain, and incident response
- observability for claims, retries, stuck leases, and terminal failures
- bounded rollout plan (one worker, then small fleet, then normal capacity)

Until those are proven, “remote real” should be treated as not ready even if a
single hosted worker can run.

## 4. Recommended Near-term Order

If "use any time soon" means this month, the practical order is:

1. Keep local dev testing green at all times.
2. Make local real testing repeatable in one controlled repo.
3. Bring up remote dev testing as a single-worker hosted canary in non-prod.
4. Only after that, plan remote real runs.

This keeps momentum while avoiding premature hosted rollout.

Current tactical note:

- The branch uses a temporary `profiles.sdlc.local` compatibility path to make local real testing runnable now.
- Delete that path once compiler-side concrete binding/link cleanup lands.

## 5. Go / No-Go Template

For each mode, decide with three questions:

1. Reliability: Did all required proof checks pass twice in a row?
2. Safety: Is rollback/manual intervention clear for failure paths?
3. Cost: Is operator effort acceptable for current stage of adoption?

If any answer is "no", remain on the previous mode and fix gaps there.
