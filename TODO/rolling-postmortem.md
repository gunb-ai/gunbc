# Rolling Postmortem: Repo Reliability Reconciliation

> Canonical rolling tracker for reliability regressions, temporary hacks, and confidence gaps.
> Goal: if compile + tests are green, first-class commands should succeed with high probability.

## How To Use This Doc

1. Log incidents and hacks as soon as they are discovered.
2. Keep each item tied to concrete evidence (error text, file path, test name).
3. Convert open items into explicit task candidates with closure criteria.
4. Keep historical lane docs (for deep analysis), but track status here.

## Branch Cleanup Observations (2026-03-05)

| ID | Observation / Hack | Evidence | Risk | Status |
| --- | --- | --- | --- | --- |
| RP-001 | GitHub workflow callers still rely on a shared concrete credential helper instead of a modeled provider path. | `dsl/shared/credentials.dag` (`resolve_github_token`) and `dsl/tools/gist.dag` / `dsl/funcs/sdlc_worker.dag` call sites. | Profile selection is bypassed for GitHub auth and policy can still drift across environments. | Open (centralized interim workaround) |
| RP-002 | Interface-implementing services without explicit operation transport lower to `InterfaceStub`. | `core/daglang/daglang-lower/src/lib.rs` (`None if service.implements.is_some() => InterfaceStub`). | Real mode can fail late with profile-stub execution error. | Open |
| RP-003 | DryRun auto-mocks `InterfaceStub` execute while Real mode hard-fails. | `core/resolve/src/service_ops/service_ops_impl.rs` (`InterfaceStubExecuteOp`). | DryRun success can mask Real mode profile-binding regressions. | Open |
| RP-004 | Some gist end-to-end tests are ignored due pre-existing DryRun field-access gap. | `gunbc-dag/tests/gist_recent_regressions.rs` and `gunbc-dag/tests/for_loop_transport.rs` have ignored gist e2e tests. | Confidence gap for real command path; regressions can slip through. | Open |
| RP-005 | Stale test assumptions existed for CI DSL path (`pipelines` vs `workflows`). | `core/daglang/daglang-cli/tests/compile_commands.rs` failures referenced missing `dsl/pipelines/ci.dag`. | False-red / noisy signals reduce trust in test gate quality. | Open (partially remediated in prior branch work) |
| RP-006 | Long-running exhaustive auto-testgen validation made `make test-all` appear hung. | `gunbc_dag::testgen_dag::dag_test_discovery::comprehensive_auto_testgen_pipeline_validation` (~1398s debug runtime). | Developer feedback loop degraded; real failures masked by wall-clock cost. | Open (currently mitigated via `#[ignore]`) |
| RP-007 | Test size tiers were previously unclear for fast triage loops; explicit XS/S/M/L/XL targets now exist. | `dsl/config/build_targets.dag` and generated `Makefile` include `test-xs/s/m/l/xl` plus aliases. | Without this, developers overrun local loops and skip tests ad hoc. | Resolved |
| RP-008 | Test-runtime sizing still relies on coarse heuristics, not measured per-test budgets. | `core/test/src/fermi.rs` hardcoded cost timeouts + `gunbc-dag/src/testgen_dag/dag_test_discovery.rs` `profile_fermi_cost` heuristic mapping. | Misclassified tests can be unexpectedly slow or skipped, reducing confidence in target labels. | Open |

## Incident Ledger

### Lane: Gist Reliability

1. Incident G1 (2026-02-27): unauthenticated gist request (401) + silent credential loss path.
2. Incident G2 (2026-03-05): Real mode interface-stub error despite green compile/test signals.
3. Incident G3 (2026-03-05): `test-all` perceived hang due exhaustive test runtime.

Canonical deep-dive docs:

1. `TODO/gist-auth-postmortem.md`
2. `TODO/gist-real-mode-confidence-postmortem.md`

Lane status snapshot:

1. Acute runtime workaround is active: GitHub auth resolves through a shared credential helper, but still uses concrete env/gcloud lookup.
2. Confidence gap remains: profile-driven credential path and DryRun/Real equivalence are not fully enforced by tests.

### Lane: Test Runtime Governance

1. Incident T1 (2026-03-05): single exhaustive auto-testgen validation dominated wall-clock test time.
2. Incident T2 (ongoing): Fermi sizing and timeout expectations are policy-driven heuristics, not continuously measured runtime contracts.

## Task Candidate Backlog

### P0 (confidence contract)

1. RC-P0-001: Add command-contract tests per first-class tool that mirror generated CLI invocation (`--profile`, mode, graph contract checks).
2. RC-P0-002: Add profile-realization invariant tests for profile-bound tools (starting with gist) that fail on unresolved `InterfaceStub` execute nodes.
3. RC-P0-003: Improve interface-stub runtime diagnostics to include selected profile, discovered bindings, and lookup context.
4. RC-P0-004: Add measured runtime budget reporting for test targets (`test-xs/s/m/l/xl`) and fail when observed runtime materially exceeds declared budget bands.

### P1 (modeling and coverage hardening)

1. RC-P1-001: Replace the shared GitHub credential helper with a modeled provider path once provider operation transport modeling is explicit and validated.
2. RC-P1-002: Unignore or replace ignored gist e2e tests with deterministic contract tests.
3. RC-P1-003: Expand testgen obligations for REST error status coverage and shell non-zero exit semantics.
4. RC-P1-004: Add stale-path/fixture drift checks for key compile-command tests.
5. RC-P1-005: Decompose monolithic exhaustive tests into bounded shards (or explicit integration workflows) so default test targets remain predictable and interactive.
6. RC-P1-006: Require explicit justification + annotation for any test expected to exceed normal local feedback budgets.

### P2 (workflow hygiene)

1. RC-P2-001: Define policy for long-running exhaustive tests (`ignored` + explicit target + runtime budget annotation).
2. RC-P2-002: Add periodic reconciliation pass: convert closed/obsolete workarounds into deletions or explicit retained design decisions.

## Closure Criteria

This rolling postmortem tracker is healthy when:

1. First-class command reliability has pre-runtime contract coverage.
2. DryRun vs Real mode confidence boundaries are explicit and tested.
3. Temporary hacks are either removed or codified as explicit design decisions.
4. Each open item has an owner, evidence, and closure criterion.
