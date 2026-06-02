# CI Wave 1 — affected job binding smoke fix (2026-06-01)

## Per-PR dissolution gate (INVARIANTS.md P5 / template mechanism (b))

**Disposition (1) — fully retires:** the false-positive **512-byte substring window**
after `id: "affected"` inside
`v4_workflow_ci_wave1_generated_workflow_dag_matches_ci_yml_shape` in
`src/v3/compiler/tests/integration/v4_workflow_ci_runner_dag_smoke_test.rs`
(that window could not reliably reach job-level `continue_on_error` / YAML header
fields ~3.5KiB later in `dsl/gunbc/ci_github_actions_workflow.dag`, causing
spurious binding failures).

**Positive-Y replacement (same test, same `#[test]` count):** job-scoped
`workflow_dag_job_block` / `ci_yml_job_block` + fail-closed job-level
`continue_on_error` / header-only `continue-on-error` checks aligned with post-#4220
live `affected` component receipt (DAG: job-level field exactly `false`, not
satisfied by step-level `continue_on_error: false`; YAML: sliced job block omits
job-level `continue-on-error` anywhere, not merely in the pre-`steps:` header).

**Check:** `cargo test -p v3-compiler --test integration v4_workflow_ci_wave1_generated_workflow_dag_matches_ci_yml_shape -- --exact`
