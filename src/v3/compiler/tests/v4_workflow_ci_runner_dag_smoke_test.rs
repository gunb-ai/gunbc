//! Standalone lean test target for the CI model/YAML binding smoke.
//!
//! The binding assertions are pure tokenize/parse + `include_str!` structural
//! checks over `src/v4/workflow/ci.dag`, `.github/workflows/ci.yml`, and the
//! `dsl/gunbc/ci_github_actions_workflow.dag` carrier — they run no pipeline and
//! gain nothing from the consolidated `integration` binary's cross-test
//! `cached_compile_to_dag` sharing. Hoisting the file into its own target lets
//! the required-CI `ci_floor` binding-smoke step compile only `v3_compiler`'s lib
//! plus this one module, instead of the full 138-module integration binary.
//!
//! The physical source stays under `integration/` so every `include_str!`
//! relative path and the `v4_workflow_ci_runner_dag_smoke_test::` test-name
//! prefix are preserved verbatim.

#[path = "integration/v4_workflow_ci_runner_dag_smoke_test.rs"]
mod v4_workflow_ci_runner_dag_smoke_test;
