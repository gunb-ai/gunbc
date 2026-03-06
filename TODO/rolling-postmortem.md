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
| RP-001 | GitHub/LLM workflow callers now use provider-local auth modules, but the auth realization is still concrete workflow logic instead of structural provider modeling. | `dsl/extdeps/github/auth.dag`, `dsl/extdeps/llm/auth.dag`, and call sites in `dsl/tools/gist.dag` / `dsl/funcs/sdlc_worker.dag`. | Policy can still drift until auth becomes a modeled provider requirement rather than an effectful helper func. | Open (improved, shared helper deleted) |
| RP-002 | Interface-implementing services without explicit operation transport lower to `InterfaceStub`. | `core/daglang/daglang-lower/src/lib.rs` (`None if service.implements.is_some() => InterfaceStub`). | Real mode can fail late with a missing concrete binding error. | Open |
| RP-003 | DryRun auto-mocks `InterfaceStub` execute while Real mode hard-fails. | `core/resolve/src/service_ops/service_ops_impl.rs` (`InterfaceStubExecuteOp`). | DryRun success can mask Real mode binding regressions. | Open |
| RP-004 | Some gist end-to-end tests are ignored due pre-existing DryRun field-access gap. | `gunbc-dag/tests/gist_recent_regressions.rs` and `gunbc-dag/tests/for_loop_transport.rs` have ignored gist e2e tests. | Confidence gap for real command path; regressions can slip through. | Open |
| RP-005 | Stale test assumptions existed for CI DSL path (`pipelines` vs `workflows`). | `core/daglang/daglang-cli/tests/compile_commands.rs` failures referenced missing `dsl/pipelines/ci.dag`. | False-red / noisy signals reduce trust in test gate quality. | Open (partially remediated in prior branch work) |
| RP-006 | Long-running exhaustive auto-testgen validation made `make test-all` appear hung. | `gunbc_dag::testgen_dag::dag_test_discovery::comprehensive_auto_testgen_pipeline_validation` (~1398s debug runtime). | Developer feedback loop degraded; real failures masked by wall-clock cost. | Open (currently mitigated via `#[ignore]`) |
| RP-007 | Test size tiers were previously unclear for fast triage loops; explicit XS/S/M/L/XL targets now exist. | `dsl/config/build_targets.dag` and generated `Makefile` include `test-xs/s/m/l/xl` plus aliases. | Without this, developers overrun local loops and skip tests ad hoc. | Resolved |
| RP-008 | Test-runtime sizing still relies on coarse heuristics, not measured per-test budgets. | `core/test/src/fermi.rs` hardcoded cost timeouts + `gunbc-dag/src/testgen_dag/dag_test_discovery.rs` `profile_fermi_cost` heuristic mapping. | Misclassified tests can be unexpectedly slow or skipped, reducing confidence in target labels. | Open |
| RP-009 | CI YAML generation bypassed `config.ci` and hardcoded cache/env/trigger policy in `tools/cigen.dag`, including `target/` in cache paths. | `dsl/tools/cigen.dag` previously inlined GitHub/GitLab cache sections; `dsl/config/ci.dag` and `core/ir/src/transport/ci/render.rs` already modeled a smaller cache. | Policy drift produced oversized CI caches and runner disk exhaustion (`No space left on device`). | Resolved on branch; keep regression test |
| RP-010 | CI provider schema had been modeled anemically in DSL: `tools/cigen.dag` rendered YAML via string concatenation instead of building typed `Workflow`/`Pipeline` values from `extdeps.github_actions` / `extdeps.gitlab_ci`. | `dsl/extdeps/github_actions.dag`, `dsl/extdeps/gitlab_ci.dag`, `dsl/extdeps/github_actions_render.dag`, `dsl/extdeps/gitlab_ci_render.dag`, and `dsl/tools/cigen.dag`. | Static policy could drift into render helpers, and provider-specific invariants stayed weakly enforced. | Resolved on branch; keep ratchet tests and carry the same pattern into adjacent render lanes |
| RP-011 | CI discovery had crossed the DAG boundary as raw shell text (`tool_command`, `bootstrap_script`) rather than typed steps/commands. | `dsl/extdeps/ci_script.dag`; `dsl/tools/cigen.dag` `type CiDiscovery`; `gunbc-app/src/extern_ops.rs` `discover_ci_config`. | Step structure, command semantics, and freshness scope were stringly and harder to validate minimally. | Resolved on branch; keep typed-script regression tests and strengthen shell quoting separately if needed |
| RP-012 | CI rendering had duplicate policy surfaces in Rust and DSL. | `core/ir/src/transport/ci/render.rs` `CacheConfig::rust()` vs `dsl/config/ci.dag` + `dsl/tools/cigen.dag`. | Parallel render paths invited drift; the Rust renderer could silently diverge from generated CI policy. | Resolved on branch; CI YAML generation is now DSL-owned end-to-end |
| RP-013 | `content_upsert` special-case lowering kept per-call wiring but mislabeled `written` as compare `fresh`, creating contradictory status output and hiding the real freshness/write behavior. | `core/daglang/daglang-lower/src/lib.rs` special-case `expand_content_upsert_patterns()` / `wire_expansion_return_outputs()`. | Investigations get misled, callsite return values lie, and special-case lowering drifts away from the DSL pattern’s actual semantics. | Resolved on branch; keep regression test and continue collapsing special-case pattern logic |

## Incident Ledger

### Lane: Gist Reliability

1. Incident G1 (2026-02-27): unauthenticated gist request (401) + silent credential loss path.
2. Incident G2 (2026-03-05): Real mode interface-stub error despite green compile/test signals.
3. Incident G3 (2026-03-05): `test-all` perceived hang due exhaustive test runtime.

Canonical deep-dive docs:

1. `TODO/gist-auth-postmortem.md`
2. `TODO/gist-real-mode-confidence-postmortem.md`

Lane status snapshot:

1. Acute runtime workaround is still active, but it is now provider-local: GitHub/LLM auth resolve through `dsl/extdeps/*/auth.dag` instead of the deleted shared helper.
2. Confidence gap remains: concrete binding realization and DryRun/Real equivalence are not fully enforced by tests.

### Lane: Test Runtime Governance

1. Incident T1 (2026-03-05): single exhaustive auto-testgen validation dominated wall-clock test time.
2. Incident T2 (ongoing): Fermi sizing and timeout expectations are policy-driven heuristics, not continuously measured runtime contracts.

## Task Candidate Backlog

### P0 (confidence contract)

1. RC-P0-001: Add command-contract tests per first-class tool that mirror generated CLI invocation (mode, graph contract checks).
2. RC-P0-002: Add concrete-binding realization invariant tests for interface-using tools (starting with gist) that fail on unresolved `InterfaceStub` execute nodes.
3. RC-P0-003: Improve interface-stub runtime diagnostics to include binding context and lookup details.
4. RC-P0-004: Add measured runtime budget reporting for test targets (`test-xs/s/m/l/xl`) and fail when observed runtime materially exceeds declared budget bands.

### P1 (modeling and coverage hardening)

1. RC-P1-001: Replace the interim provider-local auth materialization with structural provider auth modeling once provider operation transport modeling is explicit and validated.
2. RC-P1-002: Unignore or replace ignored gist e2e tests with deterministic contract tests.
3. RC-P1-003: Expand testgen obligations for REST error status coverage and shell non-zero exit semantics.
4. RC-P1-004: Add stale-path/fixture drift checks for key compile-command tests.
5. RC-P1-005: Decompose monolithic exhaustive tests into bounded shards (or explicit integration workflows) so default test targets remain predictable and interactive.
6. RC-P1-006: Require explicit justification + annotation for any test expected to exceed normal local feedback budgets.
7. RC-P1-007: Apply the same typed-assembly plus leaf-serializer pattern from CI to adjacent render lanes (`makegen`, `justgen`, CLI gen, markdown, CI reports).
8. RC-P1-008: Keep CI YAML generation single-sourced in the DSL/config path and reject new Rust-side CI render surfaces.
9. RC-P1-009: Strengthen shared shell quoting/escaping for `ScriptLine::Command { argv }` now that CI discovery is structurally modeled.
10. RC-P1-010: Continue deleting `content_upsert` lowering special-cases in favor of one semantically aligned pattern-expansion path so callsite bookkeeping cannot drift from pattern meaning.

### P2 (workflow hygiene)

1. RC-P2-001: Define policy for long-running exhaustive tests (`ignored` + explicit target + runtime budget annotation).
2. RC-P2-002: Add periodic reconciliation pass: convert closed/obsolete workarounds into deletions or explicit retained design decisions.

## Closure Criteria

This rolling postmortem tracker is healthy when:

1. First-class command reliability has pre-runtime contract coverage.
2. DryRun vs Real mode confidence boundaries are explicit and tested.
3. Temporary hacks are either removed or codified as explicit design decisions.
4. Each open item has an owner, evidence, and closure criterion.
