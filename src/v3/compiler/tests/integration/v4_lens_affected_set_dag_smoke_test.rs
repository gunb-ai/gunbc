//! **Layer:** integration
//!
//! T-21 IRT-1: `src/v4/lens/affected_set.dag` — incremental re-exec frontier substrate;
//! `(Dag, Diff) -> Witness<ReExecFrontier>` via `re_exec_frontier_from_diff`, purity-aware
//! boundary prune, dimension propagation receipts, fail-closed whole-DAG gate.
//! Claims: `src/v4/test/claim/lens_affected_set/*.dag`.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full `compile_to_dag` import merge
//! deferred until cross-module v4 load lands (peer v4 smoke posture).
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-21 (IRT-1 held).
//!
//! **Wave-A W2 fold-delete:** IRT-1 leaf behavioral receipts migrated to
//! `src/v4/test/claim/lens_affected_set/sg_claims.dag` (consolidated roster). Remaining
//! module-authority and claim-wiring greps are **B-INTERIM** host-AST declaration-shape —
//! expressible in principle; host-AST until ctrl#1476 READ-axis reflection substrate lands;
//! TRIGGER: migrate to `.dag` witness when substrate exists (route substrate consumers to
//! sleek-carp-651).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const AFFECTED_SET_DAG: &str = include_str!("../../../../v4/lens/affected_set.dag");
const AFFECTED_SET_PATH: &str = "src/v4/lens/affected_set.dag";

const IRT1_LEAF_CLAIM_SUITE: (&str, &str) = (
    include_str!("../../../../v4/test/claim/lens_affected_set/irt1_leaf_claim_suite.dag"),
    "src/v4/test/claim/lens_affected_set/irt1_leaf_claim_suite.dag",
);

const IRT1_LEAF_CLAIMS: &[(&str, &str)] = &[
    (
        include_str!("../../../../v4/test/claim/lens_affected_set/irt1_boundary_prune_receipt.dag"),
        "src/v4/test/claim/lens_affected_set/irt1_boundary_prune_receipt.dag",
    ),
    (
        include_str!("../../../../v4/test/claim/lens_affected_set/irt1_dimension_seed_receipt.dag"),
        "src/v4/test/claim/lens_affected_set/irt1_dimension_seed_receipt.dag",
    ),
    (
        include_str!(
            "../../../../v4/test/claim/lens_affected_set/irt1_excluded_propagation_receipt.dag"
        ),
        "src/v4/test/claim/lens_affected_set/irt1_excluded_propagation_receipt.dag",
    ),
    (
        include_str!(
            "../../../../v4/test/claim/lens_affected_set/fail_closed_pending_escalation.dag"
        ),
        "src/v4/test/claim/lens_affected_set/fail_closed_pending_escalation.dag",
    ),
    (
        include_str!(
            "../../../../v4/test/claim/lens_affected_set/irt1_fail_closed_absorption_receipt.dag"
        ),
        "src/v4/test/claim/lens_affected_set/irt1_fail_closed_absorption_receipt.dag",
    ),
    (
        include_str!(
            "../../../../v4/test/claim/lens_affected_set/irt1_empty_diff_frontier_receipt.dag"
        ),
        "src/v4/test/claim/lens_affected_set/irt1_empty_diff_frontier_receipt.dag",
    ),
];

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn module_path(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<&str> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .unwrap_or_default()
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn {
            name: item_name, ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

#[test]
fn v4_lens_affected_set_module_authority_and_entrypoints() {
    let module = parse_module(AFFECTED_SET_DAG, AFFECTED_SET_PATH);
    assert_eq!(
        module_path(&module),
        vec!["v4", "lens", "affected_set"],
        "{AFFECTED_SET_PATH}: module path"
    );
    assert!(
        surface_declares_fn(&module, "re_exec_frontier_from_diff"),
        "{AFFECTED_SET_PATH}: re_exec_frontier_from_diff (IRT-1 canonical entry)"
    );
    assert!(
        surface_declares_fn(&module, "affected_set_from_diff"),
        "{AFFECTED_SET_PATH}: affected_set_from_diff (Wave-2-A edit/rebuild path)"
    );
    assert!(
        AFFECTED_SET_DAG.contains(") -> Witness<ReExecFrontier>"),
        "{AFFECTED_SET_PATH}: re_exec_frontier_from_diff must return Witness<ReExecFrontier>"
    );
    assert!(
        AFFECTED_SET_DAG.contains("IRT-1 whole/changed-subgraph gate"),
        "{AFFECTED_SET_PATH}: IRT-1 whole/changed-subgraph gate marker"
    );
    assert!(
        AFFECTED_SET_DAG.contains("FailClosed { reason: _ } => acc"),
        "{AFFECTED_SET_PATH}: seed_edit_fold_acc must short-circuit on FailClosed inline"
    );
    assert!(
        !AFFECTED_SET_DAG.contains("fn affected_fold_accepts_more_edits"),
        "{AFFECTED_SET_PATH}: no parallel ReExecFrontier variant-discriminator predicate"
    );
    assert!(
        surface_declares_fn(&module, "frontier_from_fold_acc"),
        "{AFFECTED_SET_PATH}: frontier_from_fold_acc"
    );
    assert!(
        surface_declares_fn(&module, "seed_edit_fold_acc"),
        "{AFFECTED_SET_PATH}: seed_edit_fold_acc (diff fold step)"
    );
}

#[test]
fn v4_lens_affected_set_irt1_leaf_claim_wiring() {
    let absorption = IRT1_LEAF_CLAIMS[4].0;
    assert!(
        absorption.contains("claim_irt1_fail_closed_absorption_receipt")
            && absorption.contains("seed_edit_fold_acc")
            && absorption.contains("re_exec_frontier_from_diff")
            && !absorption.contains("affected_fold_accepts_more_edits"),
        "irt1_fail_closed_absorption_receipt: canonical fold entrypoints only"
    );
    for (source, path) in IRT1_LEAF_CLAIMS {
        assert!(
            source.contains("ManualAnchorAbsent") && source.contains("EqualsClaim"),
            "{path}: focused leaf TestClaim wiring"
        );
        assert!(
            source.contains("affected_set_claim_failure_node"),
            "{path}: distinct failure sentinel on false branch"
        );
    }
}

#[test]
fn v4_lens_affected_set_irt1_leaf_claim_suite_wiring() {
    let (source, path) = IRT1_LEAF_CLAIM_SUITE;
    assert!(
        source.contains("data claim_affected_set_irt1_leaf_claim_suite: TestClaim"),
        "{path}: aggregate suite TestClaim row for subsumption MechanicalReverification"
    );
    for leaf_hold in [
        "irt1_boundary_prune_receipt_claim_holds",
        "irt1_dimension_seed_receipt_claim_holds",
        "irt1_excluded_propagation_receipt_claim_holds",
        "fail_closed_pending_escalation_claim_holds",
        "irt1_fail_closed_absorption_receipt_claim_holds",
        "irt1_empty_diff_frontier_receipt_claim_holds",
    ] {
        assert!(
            source.contains(leaf_hold),
            "{path}: suite must conjoin leaf hold fn {leaf_hold}"
        );
    }
}
