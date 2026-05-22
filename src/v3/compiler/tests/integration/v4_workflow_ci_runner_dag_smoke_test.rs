//! **Layer:** integration
//!
//! T-21/T-24 Wave-0: `src/v4/workflow/ci_runner.dag` exposes affected-set-driven
//! `TestClaim` selection over `RerunNodeSet` (node-level rerun frontier). Receipt
//! claims live in `src/v4/test/claim/workflow/affected_set_ci_runner.dag`.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge
//! deferred until cross-module v4 load lands (same posture as peer v4 smoke tests).
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-21 + T-24.
//!
//! **Dissolution:** remove when `.dag` TestClaim execution covers these claims without
//! this hand-Rust parse harness.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const CI_RUNNER_DAG: &str = include_str!("../../../../v4/workflow/ci_runner.dag");
const CI_RUNNER_PATH: &str = "src/v4/workflow/ci_runner.dag";
const CLAIM_DAG: &str = include_str!("../../../../v4/test/claim/workflow/affected_set_ci_runner.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/workflow/affected_set_ci_runner.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn module_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<String>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Module(m) => Some(m.path.clone()),
            _ => None,
        })
        .collect()
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn(f) => f.name == name,
        _ => false,
    })
}

fn import_includes_path(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    symbol: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Import(i) => {
            i.path == path
                && i
                    .names
                    .iter()
                    .any(|n| n == symbol || n.starts_with(&format!("{symbol}(")))
        }
        _ => false,
    })
}

#[test]
fn v4_workflow_ci_runner_dag_tokenizes_and_parses() {
    let _module = parse_module(CI_RUNNER_DAG, CI_RUNNER_PATH);
}

#[test]
fn v4_workflow_ci_runner_module_authority_and_entrypoints() {
    let module = parse_module(CI_RUNNER_DAG, CI_RUNNER_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "workflow", "ci_runner"]],
        "{CI_RUNNER_PATH}: module authority path"
    );
    for name in [
        "ci_select_from_rerun_nodes",
        "ci_select_from_affected_set",
        "select_test_claims_for_rerun",
        "test_claim_in_rerun_frontier",
    ] {
        assert!(
            surface_declares_fn(&module, name),
            "{CI_RUNNER_PATH}: must declare {name}"
        );
    }
    assert!(
        import_includes_path(&module, &["v4", "lens", "affected_set"], "affected_set_rerun_nodes"),
        "{CI_RUNNER_PATH}: must import affected_set_rerun_nodes from T-21 lens"
    );
}

#[test]
fn v4_workflow_affected_set_ci_runner_claim_dag_tokenizes_and_parses() {
    let _module = parse_module(CLAIM_DAG, CLAIM_PATH);
}

#[test]
fn v4_workflow_affected_set_ci_runner_claim_wiring() {
    assert!(
        CLAIM_DAG.contains("ci_select_from_rerun_nodes")
            && CLAIM_DAG.contains("ci_select_from_affected_set")
            && CLAIM_DAG.contains("ci_runner_narrow_selection_claim")
            && CLAIM_DAG.contains("ci_runner_fail_closed_superset_claim"),
        "{CLAIM_PATH}: receipt claims must call ci_runner selection entrypoints"
    );
}
