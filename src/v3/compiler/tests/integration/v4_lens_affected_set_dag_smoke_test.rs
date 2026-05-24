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

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const AFFECTED_SET_DAG: &str = include_str!("../../../../v4/lens/affected_set.dag");
const AFFECTED_SET_PATH: &str = "src/v4/lens/affected_set.dag";
const IRT1_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/lens_affected_set/irt1_mechanical_reverification.dag");
const IRT1_CLAIM_PATH: &str =
    "src/v4/test/claim/lens_affected_set/irt1_mechanical_reverification.dag";
const FAIL_CLOSED_PENDING_ESCALATION_CLAIM_DAG: &str = include_str!(
    "../../../../v4/test/claim/lens_affected_set/fail_closed_pending_escalation.dag"
);
const FAIL_CLOSED_PENDING_ESCALATION_CLAIM_PATH: &str =
    "src/v4/test/claim/lens_affected_set/fail_closed_pending_escalation.dag";

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
fn v4_lens_affected_set_dag_tokenizes_and_parses() {
    let _ = parse_module(AFFECTED_SET_DAG, AFFECTED_SET_PATH);
    let _ = parse_module(IRT1_CLAIM_DAG, IRT1_CLAIM_PATH);
    let _ = parse_module(
        FAIL_CLOSED_PENDING_ESCALATION_CLAIM_DAG,
        FAIL_CLOSED_PENDING_ESCALATION_CLAIM_PATH,
    );
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
fn v4_lens_affected_set_irt1_claim_wiring() {
    assert!(
        IRT1_CLAIM_DAG.contains("claim_affected_set_irt1_mechanical_reverification")
            && IRT1_CLAIM_DAG.contains("irt1_mechanical_reverification_claim_holds")
            && IRT1_CLAIM_DAG.contains("seed_edit_fold_acc")
            && IRT1_CLAIM_DAG.contains("re_exec_frontier_from_diff")
            && !IRT1_CLAIM_DAG.contains("affected_fold_accepts_more_edits"),
        "{IRT1_CLAIM_PATH}: IRT-1 claim exercises canonical fold entrypoints only"
    );
}

#[test]
fn v4_lens_affected_set_fail_closed_pending_escalation_claim_wiring() {
    let module = parse_module(
        FAIL_CLOSED_PENDING_ESCALATION_CLAIM_DAG,
        FAIL_CLOSED_PENDING_ESCALATION_CLAIM_PATH,
    );
    assert_eq!(
        module_path(&module),
        vec![
            "v4",
            "test",
            "claim",
            "lens_affected_set",
            "fail_closed_pending_escalation"
        ],
        "{FAIL_CLOSED_PENDING_ESCALATION_CLAIM_PATH}: module path"
    );
    assert!(
        FAIL_CLOSED_PENDING_ESCALATION_CLAIM_DAG.contains("claim_fail_closed_pending_escalation")
            && FAIL_CLOSED_PENDING_ESCALATION_CLAIM_DAG.contains("frontier_from_fold_acc"),
        "{FAIL_CLOSED_PENDING_ESCALATION_CLAIM_PATH}: pending-escalation scaffold claim wiring"
    );
}
