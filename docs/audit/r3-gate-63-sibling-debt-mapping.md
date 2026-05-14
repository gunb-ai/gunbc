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
| `dsl/gunbc/ci_emission.dag` | Allocated survivor: #99 / #100 | `WorkflowRuntime = YamlStatic \| BinaryShim` is #99 passing at HEAD. `project_github_actions` and `gunbc_ci_yml_workflow` are declared/text-ratcheted; full type-check/body completion remains row #100. |
| `dsl/gunbc/ci_github_actions_workflow.dag` | Allocated survivor: #100 | Generated/pinned GitHub Actions `Workflow` from `.github/workflows/ci.yml`; byte drift guard exists, but `.dag`-authoritative projection is row #100. |
| `src/v3/std/t_ci_workflow_as_data_demo.dag` | Pass-through | Authored modeled CI workflow and `demo_ci_modeled_timing_dimension_report` evaluator entrypoint. Gate #63 uses the bootstrap-shell receipt. |
| `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs` | Pass-through plus allocated sibling receipts | Default tests pin modeled workflow rows, CI DAG topology, command shape, gate #99 runtime enum, and the gate #63 evaluator receipt. Tests for projection/runtime surfaces remain row #99/#100 scope. |
| `.github/workflows/ci.yml` | Allocated survivor: #98 / #99 / #100 / #103 | Live CI transport remains YAML. It is explicitly tracked by T-WAD FULL R3 rows: hand-authority dissolution (#98), runtime enum/projection (#99/#100), and affected-set selection (#103). No unallocated Class 4 survivor. |
| `.github/workflows/ci-spot-rerun.yml` | Allocated survivor: #99 | Workflow-run retry scheduling is YAML transport logic. It is outside gate #63's modeled-CI evaluator receipt and belongs to workflow-runtime follow-on scope. |
| `.github/workflows/tier3-baseline-capture.yml` | Allocated survivor: #99/#100 | Manual benchmark capture workflow is GitHub Actions transport/scheduling logic. It is not part of the gate #63 representative CI workflow receipt and remains allocated to workflow-runtime/projection rows. |
| CI-referenced scripts | Pass-through or allocated to existing rows | Shell/Python scripts invoked by YAML are policy/build/test commands, not separate workflow authority unless they encode dispatch/projection. Workflow split/selection scripts are explicitly row #103 or existing CI-discipline gates; no unallocated Class 4 survivor found. |

## YAML Structural Facts

| YAML file | Structural facts | Classification |
|---|---|---|
| `.github/workflows/ci.yml` | Top-level `name`, `on.push`, `on.pull_request`, `permissions`, `concurrency`, `env`; jobs `fmt`, `ci`, `v3`, `self_host_ratchet`; job-level `if`, `runs-on`, `timeout-minutes`, `needs`, `continue-on-error`; step-level `uses`, `run`, `env`, and cache/setup directives. | Allocated to rows #98/#99/#100/#103. `dsl/gunbc/ci.dag` models the representative compiler gate DAG; full YAML authority dissolution/projection is outside gate #63. |
| `.github/workflows/ci-spot-rerun.yml` | Top-level `name`, `on.workflow_run`, `permissions`; job `rerun-once` with `if`, `runs-on`, `timeout-minutes`; step invokes `gh run rerun --failed`. | Allocated to row #99 workflow-runtime follow-on. |
| `.github/workflows/tier3-baseline-capture.yml` | Top-level `workflow_dispatch`, `permissions`, `concurrency`; jobs `capture-run` and `aggregate`; matrix strategy, artifact upload/download, benchmark and aggregation run steps. | Allocated to rows #99/#100 workflow-runtime/projection follow-on. |

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

## Result

- Unallocated Class 4 survivors: **0**
- Allocated survivors: rows **#98**, **#99**, **#100**, **#101/#102** where test-cost policy is involved, and **#103**
- Gate #63 is eligible for `CONSUMER_LANDED + PASSING` because the representative evaluator receipt runs by default and all sibling workflow/scheduling debt has explicit ledger allocation.
