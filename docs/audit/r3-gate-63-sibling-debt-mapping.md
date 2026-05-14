# R3 Gate #63 Workflow Scheduling Closure Audit

**Gate:** `substrate_gap_workflow_scheduling_closed` (§1.8 row #63)
**Worker:** still-fox-187
**Date:** 2026-05-14
**Authority:** `docs/briefs/r3-substrate-gate-63-workflow-scheduling-worker.md`; canvas PR #2831 squash `89df284e3`; Director msg_804cdc93 relayed by PM msg_1e52a61b.

## Closure Receipts

Gate #63 closes on the §1.4 Class 4 conjunction:

1. **Representative execution receipt:** `t_ci_workflow_as_data_demo_test::ci_workflow_as_data_demo_timing_dimension_report_on_bootstrap_shell` runs by default and evaluates `demo_ci_modeled_timing_dimension_report` through the PB-1 bootstrap evaluator, producing `DimensionReport<TimingMeasurement>`. The prior planned-deferral ignore anchor is commit `73969f4a9`; HEAD no longer carries a `#[ignore]` for this receipt.
2. **Class bridge inventory:** the authority-surface enumeration and keyword cross-check below found **0 unallocated Class 4 survivors**. Remaining non-executing workflow/scheduling surfaces are Director-allocated to §1.8 rows #99 and #100.

## Director Allocation

Director msg_804cdc93 is the STRUCTURAL exception authority for the sibling failures:

> The sibling failures are NOT 'directly load-bearing for modeled as .dag data criterion' -- they're load-bearing for rows #99 + #100 substrate-shape closure paths, which have their own §1.8 ledger entries + closure scopes. Gate #63 closing via Candidate A does NOT hide substrate-debt because rows #99 + #100 carry that debt explicitly with their own DECLARED → CONSUMER_LANDED arc.

## Authority-Surface Inventory

| Surface | Classification | Receipt |
|---|---|---|
| `dsl/extdeps/github/actions.dag` | Pass-through substrate carrier surface | Declares the GitHub Actions model required by §4.4: `Workflow`, `WorkflowTrigger`, `WorkflowSecret`, `RunnerSpec` / `RunnerLabel`, `Step`, and `MatrixStrategy`. Consumers use these carriers directly. |
| `dsl/gunbc/ci.dag` | Pass-through for gate #63; allocated follow-on for projection | `CIWorkflowDag` is the provider-neutral CI workflow DAG authority: `CIPipeline`, `CIGate`, `CIGateEdge`, and the pinned `github_actions_workflow` carrier. Full projection from this authority is row #100. |
| `dsl/gunbc/ci_emission.dag` | Allocated survivor: #99 / #100 | Runtime-arm and projection facts are **not** gate #63 evidence. Verified current HEAD live enum is exactly `WorkflowRuntime = YamlStatic \| BinaryShim`; row #99 owns that enum surface. The YamlStatic arm, BinaryShim arm, and design-only PythonShim/InlineGunbc future-arm references are allocated below. `project_github_actions` and `gunbc_ci_yml_workflow` are row #100 / #98 follow-on scope. |
| `dsl/gunbc/ci_github_actions_workflow.dag` | Allocated survivor: #100 | Generated/pinned GitHub Actions `Workflow` from `.github/workflows/ci.yml`; byte drift guard exists, but `.dag`-authoritative projection is row #100. |
| `src/v3/std/t_ci_workflow_as_data_demo.dag` | Pass-through | Authored modeled CI workflow and `demo_ci_modeled_timing_dimension_report` evaluator entrypoint. Gate #63 uses the bootstrap-shell receipt. |
| `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs` | Pass-through plus allocated sibling receipts | Default tests pin modeled workflow rows, CI DAG topology, command shape, gate #99 runtime enum, and the gate #63 evaluator receipt. Tests for projection/runtime surfaces remain row #99/#100 scope. |
| `.github/workflows/ci.yml` | Allocated survivor: #98 / #99 / #100 / #103 | Live CI transport remains YAML. It is explicitly tracked by T-WAD FULL R3 rows: hand-authority dissolution (#98), runtime enum/projection (#99/#100), and affected-set selection (#103). No unallocated Class 4 survivor. |
| `.github/workflows/ci-spot-rerun.yml` | Allocated survivor: #99 | Workflow-run retry scheduling is YAML transport logic. It is outside gate #63's modeled-CI evaluator receipt and belongs to workflow-runtime follow-on scope. |
| `.github/workflows/tier3-baseline-capture.yml` | Allocated survivor: #99/#100 | Manual benchmark capture workflow is GitHub Actions transport/scheduling logic. It is not part of the gate #63 representative CI workflow receipt and remains allocated to workflow-runtime/projection rows. |
| CI-referenced scripts | Pass-through or allocated to existing rows | Shell/Python scripts invoked by YAML are policy/build/test commands, not separate workflow authority unless they encode dispatch/projection. Workflow split/selection scripts are explicitly row #103 or existing CI-discipline gates; no unallocated Class 4 survivor found. |

## Pass 1 Detailed Classifications

These tables expand the closed-surface checklist from the worker brief. "Pass-through" means the fact is a landed substrate/test fact that does not require a row #99/#100 runtime/projection closure before gate #63 can close. "Allocated survivor" means the fact is real Class 4 workflow/scheduling debt but has an explicit §1.8 home outside gate #63.

### `dsl/extdeps/github/actions.dag`

| Fact group | Classification | Row / receipt |
|---|---|---|
| `Workflow` top-level carrier (`name`, `on`, `concurrency`, `jobs`, `env`, `permissions`) | Pass-through substrate prerequisite | Satisfies §4.4 `Workflow<Trigger, Steps, Resources>` carrier mapping for gate #63; projection use remains #100. |
| `WorkflowTrigger`, `PullRequestActivity`, `DispatchInput`, `DispatchInputType` | Pass-through substrate prerequisite | Satisfies §4.4 trigger-event sum; no gate #63 survivor. |
| `WorkflowPermissions`, `PermissionLevel` | Pass-through platform substrate | YAML permission facts classify under runtime/projection rows only when emitted/ingested. |
| `WorkflowSecret`, `SecretScope` | Pass-through substrate prerequisite | Satisfies §4.4 `WorkflowSecret<Name>` mapping; attachment-site wiring remains outside gate #63. |
| `Job` | Pass-through platform substrate | Job shape supports modeled workflow data; provider-specific projection remains #100. |
| `RunnerSpec`, `RunnerLabel` | Pass-through substrate prerequisite | Satisfies §4.4 `RunnerResource<C>` mapping. |
| `Step`, `ShellType`, `ActionRef` | Pass-through substrate prerequisite | Satisfies §4.4 step-graph shape via `Step`; command semantics in repo CI scripts are classified separately below. |
| `ConcurrencySpec`, `CancelInProgressSpec`, `CancelInProgressWhenQueueMax` | Pass-through carrier; YAML instances allocated | Carrier is landed substrate; concrete YAML concurrency facts are allocated to #99/#100/#103 as listed in the YAML table. |
| `ArtifactOp`, `MissingFilesBehavior`, action refs | Pass-through platform substrate | Not gate #63 execution debt. |
| `MatrixStrategy` | Pass-through substrate prerequisite | Satisfies §4.4 `WorkflowMatrix<Axes>` mapping. |
| `CheckConclusion` and annotation carriers | Pass-through platform substrate | Not a gate #63 survivor. |

### `dsl/gunbc/ci.dag`

| Fact group | Classification | Row / receipt |
|---|---|---|
| `RatchetDirection`, `RatchetMetric`, `GateSource` | Pass-through CI gate substrate | Structural gate kind facts consumed by current tests; no unallocated survivor. |
| `CICommand` | Pass-through for gate #63; allocated for emitted command transport | `ci_workflow_as_data_demo_pins_interim_command_shape` pins current command coproduct; full emitted transport remains #99/#100/#103 where applicable. |
| `CIGate`, `CIPipeline`, `CIGateEdge` | Pass-through | Provider-neutral workflow DAG topology used by the representative gate #63 receipt. |
| `CIWorkflowDag` | Pass-through for modeled DAG; allocated for projection | Canonical workflow-as-dag carrier exists and is consumed by tests; `project_github_actions(CIWorkflowDag, WorkflowRuntime)` remains row #100. |
| Data rows `lint_gate`, `test_gate`, `l1_gate`, `compile_gates_gate`, `ci_pipeline`, `ci_workflow_dag` | Pass-through | Current modeled CI gate roster/topology; runtime dispatch and BinaryShim affected-set execution remain rows #99/#103. |

### `dsl/gunbc/ci_emission.dag`

| Fact group / arm | Classification | Row / receipt |
|---|---|---|
| `WorkflowRuntime` enum declaration | Allocated survivor | Row #99 owns the runtime-surface substrate. Gate #63 does not depend on runtime enum closure. |
| `YamlStatic` arm | Allocated survivor | Source contract and carrier equality now support row #59; `.dag`-authoritative emit-back and hand-authority dissolution remain rows #98/#100. |
| `BinaryShim` arm / `gunbc_ci_emission_binary_shim_workflow` | Allocated survivor | Row #99/#100 owns BinaryShim runtime/projection completion; row #103 owns affected-set selection feeding the runner. |
| PythonShim future-arm references | Allocated survivor | Design-only future runtime target named in row #99 scope comments; verified not present in the live `WorkflowRuntime` enum at HEAD and not gate #63 evidence. |
| InlineGunbc future-arm references | Allocated survivor | Design-only future runtime target named in row #99 scope comments; verified not present in the live `WorkflowRuntime` enum at HEAD and not gate #63 evidence. |
| `project_github_actions` | Allocated survivor | Row #100 owns the projection-function body/type-check closure. |
| `gunbc_ci_yml_workflow` | Allocated survivor | Row #98/#100 owns `.github/workflows/ci.yml` authority dissolution / emitted-artifact proof. |

### `dsl/gunbc/ci_github_actions_workflow.dag`

| Fact group | Classification | Row / receipt |
|---|---|---|
| Generated module header and source path | Allocated survivor | YAML-derived artifact; row #98/#100 own authority dissolution/projection direction. |
| `gunbc_ci_github_actions_workflow: Workflow` | Allocated survivor | Byte drift guard exists; not a gate #63 evaluator receipt. |
| Embedded triggers, permissions, concurrency, jobs, steps, cache/setup/action rows | Allocated survivor | These are concrete provider transport facts allocated to #98/#100. |

### `src/v3/std/t_ci_workflow_as_data_demo.dag`

| Fact group | Classification | Row / receipt |
|---|---|---|
| `modeled_gunbc_ci_workflow: Workflow` | Pass-through | Representative workflow data for gate #63 and gate #58 timing row. |
| `ci_demo_timing_read`, `ci_wad_modeled_workflow_timing_witness`, seed helpers | Pass-through | Evaluator-safe timing witness path for the representative receipt. |
| `DimensionReport<TimingMeasurement>` helpers and `demo_ci_modeled_timing_dimension_report` | Pass-through | Load-bearing gate #63 evaluator receipt. |
| `gate_58_ci_workflow_timing_row`, `gate_58_modeled_ci_timing_measurement`, `gate_58_apply_lens_self_application_pass` | Pass-through for self-application; not #63 debt | Gate #58 receipt, already ledger-owned separately. |

### `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs`

| Test / group | Classification | Row / receipt |
|---|---|---|
| `ci_workflow_as_data_demo_pins_modeled_workflow_row` | Pass-through | Confirms bootstrap modeled workflow row exists and no mirror CI DAG is authored. |
| `ci_workflow_as_data_demo_pins_structural_ci_dag_shape` | Pass-through | Confirms provider-neutral `CIWorkflowDag` topology. |
| `ci_workflow_as_data_demo_pins_interim_command_shape` | Pass-through with allocated transport follow-on | Pins current `CICommand` shape; emitted command/runtime closure is #99/#100/#103. |
| `ci_workflow_as_data_demo_uses_only_gunbc_ci_authority_topology` | Pass-through | Confirms no parallel topology authority for gate #63. |
| `gunbc_ci_github_actions_workflow_authority_compiles` | Allocated survivor | Provider transport compile receipt; projection closure remains #100. |
| `gunbc_ci_github_actions_workflow_dag_matches_yaml_generator_output` | Allocated survivor | YAML-authority drift guard; row #98/#100, not gate #63. |
| `gunbc_ci_emission_substrate_contract_is_present` | Allocated survivor | Text ratchet for #100 projection signature/binding. |
| `workflow_runtime_initial_enum_matches_t_wad_gate_99` | Allocated survivor | Row #99 runtime enum consumer. |
| `lens_self_application_demonstrated_*` / `recursive_flex_demonstration_landed` group | Pass-through or separately ledger-owned | Rows #57/#59, not gate #63 closure debt. |
| `ci_uses_affected_set_selection_*`, `workflow_no_path_regex_policy_ci_yml` | Allocated survivor | Row #103 affected-set/path policy. |
| `ci_workflow_as_data_demo_timing_dimension_report_on_bootstrap_shell` | Pass-through | Gate #63 representative evaluator receipt; default-running, no `#[ignore]`. |

## YAML Structural Facts

| YAML file | Structural facts | Classification |
|---|---|---|
| `.github/workflows/ci.yml` | Top-level `name`, `on.push`, `on.pull_request`, `permissions`, `concurrency`, `env`; jobs `fmt`, `ci`, `v3`, `self_host_ratchet`; job-level `if`, `runs-on`, `timeout-minutes`, `needs`, `continue-on-error`; step-level `uses`, `run`, `env`, and cache/setup directives. | Allocated to rows #98/#99/#100/#103. `dsl/gunbc/ci.dag` models the representative compiler gate DAG; full YAML authority dissolution/projection is outside gate #63. |
| `.github/workflows/ci-spot-rerun.yml` | Top-level `name`, `on.workflow_run`, `permissions`; job `rerun-once` with `if`, `runs-on`, `timeout-minutes`; step invokes `gh run rerun --failed`. | Allocated to row #99 workflow-runtime follow-on. |
| `.github/workflows/tier3-baseline-capture.yml` | Top-level `workflow_dispatch`, `permissions`, `concurrency`; jobs `capture-run` and `aggregate`; matrix strategy, artifact upload/download, benchmark and aggregation run steps. | Allocated to rows #99/#100 workflow-runtime/projection follow-on. |

### YAML Per-Fact Classification

| YAML file | Fact | Classification |
|---|---|---|
| `.github/workflows/ci.yml` | `name: ci` | Allocated survivor: #98/#100 (live YAML transport identity). |
| `.github/workflows/ci.yml` | `on.push.branches: [main]` | Allocated survivor: #98/#100 (provider trigger transport). |
| `.github/workflows/ci.yml` | `on.pull_request.branches/types` | Allocated survivor: #98/#100 (provider trigger transport). |
| `.github/workflows/ci.yml` | top-level `permissions` | Allocated survivor: #98/#100 (provider permission transport). |
| `.github/workflows/ci.yml` | top-level `concurrency` group/cancel policy | Allocated survivor: #98/#99/#100 (runtime scheduling transport). |
| `.github/workflows/ci.yml` | top-level `env` | Allocated survivor: #98/#100 (provider transport environment). |
| `.github/workflows/ci.yml` | job `fmt`: `if`, `runs-on`, `timeout-minutes`, checkout/setup-rust/fmt steps | Allocated survivor: #98/#100; command payload is pass-through policy, scheduling envelope is provider transport. |
| `.github/workflows/ci.yml` | job `ci`: `if`, `runs-on`, `timeout-minutes`, checkout/fetch/policy/setup/cache/regen/gate #103 steps | Allocated survivor: #98/#100/#103; explicit affected-set/path-regex steps are #103. |
| `.github/workflows/ci.yml` | job `v3`: `if`, fixed runner, `timeout-minutes`, checkout/setup/cache/build/test/clippy/ratchet steps | Allocated survivor: #98/#99/#100; full-suite split/zero-filter scheduling is runtime/test-gate transport, not gate #63. |
| `.github/workflows/ci.yml` | job `self_host_ratchet`: `if`, fixed runner, `needs: [v3]`, `continue-on-error`, main-only matrix steps | Allocated survivor: #98/#99/#100; dependency scheduling and main-only dispatch are workflow-runtime transport. |
| `.github/workflows/ci-spot-rerun.yml` | `name`, `on.workflow_run`, `permissions` | Allocated survivor: #99 (workflow-run retry runtime). |
| `.github/workflows/ci-spot-rerun.yml` | job `rerun-once`: `if`, `runs-on`, `timeout-minutes`, `gh run rerun --failed` step | Allocated survivor: #99 (retry scheduling behavior). |
| `.github/workflows/tier3-baseline-capture.yml` | `name`, `on.workflow_dispatch`, `permissions`, top-level `concurrency` | Allocated survivor: #99/#100 (manual workflow transport). |
| `.github/workflows/tier3-baseline-capture.yml` | job `capture-run`: matrix strategy, runner, timeout, checkout/setup/bench/stage/upload steps | Allocated survivor: #99/#100 (manual benchmark workflow scheduling). |
| `.github/workflows/tier3-baseline-capture.yml` | job `aggregate`: `needs: capture-run`, runner, timeout, checkout/download/aggregate/upload steps | Allocated survivor: #99/#100 (manual benchmark workflow scheduling). |

## CI-Referenced Script Classification

| Script | Classification |
|---|---|
| `scripts/check-pr-sg0-net-shrink-discipline.sh` and self-test | CI-discipline pass-through; not workflow scheduling authority. |
| `scripts/check-r4-carve-dissolution-discipline.sh` | CI-discipline pass-through; not workflow scheduling authority. |
| `scripts/check-fabrication-sentinels.sh` | CI-discipline pass-through. |
| `scripts/check-release-doc-authority.sh` / `scripts/test-check-release-doc-authority.sh` | CI-discipline pass-through. |
| `scripts/check-manager-brief-authority.sh` / `scripts/test-check-manager-brief-authority.sh` | CI-discipline pass-through. |
| `scripts/test-check-test-timeout.sh` / `scripts/check-test-timeout.sh` | Test-cost policy pass-through; row #101/#102 context where cost policy is involved. |
| `scripts/check-rust-toolchain-single-authority.sh` | Toolchain authority pass-through. |
| `scripts/check-workflow-path-regex-inventory.sh` | Allocated to row #103 affected-set/path-regex discipline. |
| `scripts/check-v3-full-suite-split-test-targets.sh` | Allocated workflow/test splitting bridge; row #99/#103 follow-on context, not gate #63. |
| `scripts/check-compiler-std-ratchet.sh` / `scripts/check-banked-dissolutions.sh` | CI-discipline pass-through. |
| `scripts/aggregate_tier3_baseline.py` | Tier-3 baseline capture command payload; workflow scheduling remains YAML allocated to #99/#100. |

The closed script set above was derived from every `run:` block in tracked `.github/workflows/*.yml` files. No tracked `.github/workflows/*.yaml` files exist at HEAD. `uses:` actions are provider action references, classified in the YAML table rather than as local script survivors.

## Sibling Failure Mapping

| Sibling failure from snappy-bear-502 audit | Owning row |
|---|---|
| `dsl/gunbc/ci_emission.dag` unresolved `CIWorkflowDag` | #100 `project_github_actions_landed` |
| `gunbc_ci_emission_binary_shim_workflow` opaque body | #99 `workflow_runtime_open_enum_landed` BinaryShim arm follow-on |
| PythonShim placeholder opaque body | #99 future runtime arm; design-only until substrate-prereq PR |
| `dsl/gunbc/ci_github_actions_workflow.dag` opaque body plus `concurrency` mismatch | #100 |
| `ci_workflow_as_data_demo_pins_*` topology/command sibling tests | #99/#100 topology/runtime/projection receipts |
| `gunbc_ci_emission_substrate_compiles` and `gunbc_ci_emission_authority_compiles` | #99/#100 substrate-shape close path |

## Keyword Cross-Check

Cross-check command:

```sh
git grep -nE "ci_workflow|ci_emission|ci_github_actions|github_actions_workflow|project_github_actions|workflow_runtime|WorkflowRuntime|workflow_as_data|workflow_scheduling|CIWorkflowDag|WorkflowTrigger|WorkflowStep|WorkflowSecret|MatrixStrategy|RunnerSpec|RunnerLabel|concurrency" \
  src/v3/ dsl/ .github/workflows/ scripts/ \
  | grep -v "^Binary file"
```

Findings were confined to the surfaces inventoried above plus generated bootstrap spans, existing SG-0 accounting, extdeps path facts, and the row #103 affected-set/runtime implementation (`src/v3/compiler/src/bin/gunbc_ci.rs`, `src/v3/compiler/src/gunbc_ci.rs`). Generated bootstrap spans are pass-through artifacts of the loaded `t_ci_workflow_as_data_demo.dag`; row #103 implementation is already ledger-owned. No additional unallocated Class 4 survivor was discovered.

### Keyword-Hit Path Classification

| Keyword-hit path | Classification |
|---|---|
| `dsl/config/gitignore.dag` | Pass-through: ignores workflow runtime state path; no scheduling authority. |
| `dsl/extdeps/cron_schedule_model.dag` | Pass-through external schedule carrier; used by `WorkflowTrigger::Schedule`. |
| `dsl/extdeps/github/actions.dag` | Pass-through substrate carrier surface; detailed above. |
| `dsl/extdeps/github/ci.dag` | Pass-through path-identity substrate for GitHub workflow files; projection use allocated to #100. |
| `dsl/gunbc/ci.dag` | Pass-through for modeled CI DAG; projection/runtime follow-on allocated above. |
| `dsl/gunbc/ci_emission.dag` | Allocated survivor: #99/#100; runtime/projection details above. |
| `dsl/gunbc/ci_github_actions_workflow.dag` | Allocated survivor: #98/#100. |
| `dsl/gunbc/test_node_wall_clock_ratchet.dag` | Allocated/pass-through: test-cost rows #101/#102; not unallocated Class 4 debt. |
| `dsl/std/languages.dag` | Pass-through prose/example hit on concurrency; no workflow scheduling authority. |
| `.github/workflows/ci.yml` | Allocated survivor: #98/#99/#100/#103; per-fact table above. |
| `.github/workflows/ci-spot-rerun.yml` | Allocated survivor: #99; per-fact table above. |
| `.github/workflows/tier3-baseline-capture.yml` | Allocated survivor: #99/#100; per-fact table above. |
| `scripts/ci-merge/sg0-pr-body-append.2371.txt` | Pass-through historical PR-body receipt; no live scheduling authority. |
| `scripts/workflow-path-regex-forbidden-substrings.txt` | Allocated survivor: #103 path-regex policy manifest. |
| `src/v3/compiler/build.rs` | Pass-through bootstrap file-ordering comment for timing/workflow modules. |
| `src/v3/compiler/Cargo.toml` | Pass-through build dependency on workflow DAG generator. |
| `src/v3/compiler/src/bin/gunbc_ci.rs` | Allocated survivor: BinaryShim runner entrypoint (#99/#103). |
| `src/v3/compiler/src/bootstrap_generated.rs` / `bootstrap_generated_without_parse_surface.rs` | Pass-through generated spans from loaded std workflow demo. |
| `src/v3/compiler/src/enforced_lens_application.rs` | Pass-through gate #58 timing enforcement consumer. |
| `src/v3/compiler/src/gunbc_ci.rs` | Allocated survivor: row #103 affected-set/runtime selection implementation. |
| `src/v3/compiler/tests/fixtures/t_gate_58_timing_enforcement_budget_violation.dag` | Pass-through gate #58 negative fixture. |
| `src/v3/compiler/tests/integration/parse_corpus_manifest.txt` | Pass-through corpus manifest entry for the std demo. |
| `src/v3/compiler/tests/integration.rs` | Pass-through test module registration. |
| `src/v3/compiler/tests/integration/sg0_census_test.rs` | Pass-through SG-0 accounting for existing workflow tests. |
| `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs` | Mixed pass-through/allocated; per-test table above. |
| `src/v3/compiler/tests/integration/t_gate_58_apply_lens_self_application_test.rs` | Pass-through gate #58 receipt. |
| `src/v3/SELF_HOSTING.md` | Pass-through prose hit; no live scheduling authority. |
| `src/v3/std/bootstrap_authority.dag` | Pass-through std file authority entry. |
| `src/v3/std/t_ci_workflow_as_data_demo.dag` | Pass-through gate #63 representative evaluator surface. |
| `src/v3/std/timing_lens.dag` | Pass-through timing/file-attachment carrier context (#55/#62). |

## Result

- Unallocated Class 4 survivors: **0**
- Allocated survivors: rows **#98**, **#99**, **#100**, **#101/#102** where test-cost policy is involved, and **#103**
- Gate #63 is eligible for `CONSUMER_LANDED + PASSING` because the representative evaluator receipt runs by default and all sibling workflow/scheduling debt has explicit ledger allocation.
