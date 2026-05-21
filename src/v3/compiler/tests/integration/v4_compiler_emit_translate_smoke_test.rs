//! **Layer:** integration
//!
//! **T-10 / Wave-3-B:** `06_translate.dag` + `05_emit.dag` + `mvp1_rust_add_translate.dag`
//! tokenize/parse cleanly (M1(2.7) single-file path; full `compile_to_dag` deferred until
//! multi-module v4 load lands). Peers: `v4_bin_main_dag_smoke_test.rs`, `v4_extdeps_file_system_dag_smoke_test.rs`.
//!
//! **TESTING.md:** Assertions use `parse_for_test` surface AST (module path, import rows, `fn`
//! bodies) — not raw source `contains` probes — so helper renames/reordering preserve semantics.
//!
//! **ROADMAP:** `ROADMAP.md` § **Nine lanes** row **T-PB-B** / `pb_rust_tests_outside_residual_zero`;
//! **TASKS.md** T-10 (`src/v4/compiler/06_translate.dag`, `05_emit.dag`).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS §SG-0 hand-authored integration test receipts row
//! land in the same PR.
//!
//! **Dissolution:** remove when translate/emit/MVP-1 surfaces are exercised only by `.dag`
//! `TestClaim` rows / a generated harness without this per-file Rust probe (or when
//! `compile_to_dag` over v4 compiler modules resolves imports without substrate collision).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceExpr, SurfaceItem};
use v3_compiler::tokenize_for_test;

const TRANSLATE_DAG: &str = include_str!("../../../../v4/compiler/06_translate.dag");
const TRANSLATE_PATH: &str = "src/v4/compiler/06_translate.dag";
const EMIT_DAG: &str = include_str!("../../../../v4/compiler/05_emit.dag");
const EMIT_PATH: &str = "src/v4/compiler/05_emit.dag";
const MVP1_CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/manual/mvp1_rust_add_translate.dag");
const MVP1_CLAIM_PATH: &str = "src/v4/test/claim/manual/mvp1_rust_add_translate.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

#[test]
fn v4_translate_dag_tokenizes_and_parses() {
    let _module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
}

#[test]
fn v4_translate_dag_module_path_is_compiler_translate() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "compiler", "translate"]],
        "{TRANSLATE_PATH}: module authority path"
    );
}

#[test]
fn v4_translate_dag_imports_coercion_fold_and_fold_node() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "coercion"], "coercion_fold"),
        "{TRANSLATE_PATH}: must import coercion_fold from v4.std.coercion"
    );
    assert!(
        import_includes_name(&module, &["v4", "std", "node"], "fold_node"),
        "{TRANSLATE_PATH}: must import fold_node from v4.std.node"
    );
}

#[test]
fn v4_translate_dag_coerce_grounded_node_calls_coercion_fold() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    let body = fn_body(&module, "coerce_grounded_node")
        .unwrap_or_else(|| panic!("{TRANSLATE_PATH}: missing fn coerce_grounded_node"));
    assert!(
        expr_mentions_call(body, "coercion_fold"),
        "{TRANSLATE_PATH}: coerce_grounded_node must delegate to coercion_fold"
    );
}

#[test]
fn v4_translate_dag_translate_node_calls_fold_node() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    let body = fn_body(&module, "translate_node")
        .unwrap_or_else(|| panic!("{TRANSLATE_PATH}: missing fn translate_node"));
    assert!(
        expr_mentions_call(body, "fold_node"),
        "{TRANSLATE_PATH}: translate_node must traverse via fold_node"
    );
}

#[test]
fn v4_translate_dag_does_not_inline_find_witness_calls() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    for name in [
        "coerce_grounded_node",
        "translate_node",
        "translate",
        "translate_fold_init",
    ] {
        let body =
            fn_body(&module, name).unwrap_or_else(|| panic!("{TRANSLATE_PATH}: missing fn {name}"));
        assert!(
            !expr_mentions_call(body, "find_witness"),
            "{TRANSLATE_PATH}: fn {name} must not inline find_witness (Practice 11)"
        );
    }
}

#[test]
fn v4_emit_dag_tokenizes_and_parses() {
    let _module = parse_module(EMIT_DAG, EMIT_PATH);
}

#[test]
fn v4_emit_dag_module_path_is_compiler_emit() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "compiler", "emit"]],
        "{EMIT_PATH}: module authority path"
    );
}

#[test]
fn v4_emit_dag_imports_translate_stage() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        import_includes_name(&module, &["v4", "compiler", "translate"], "translate"),
        "{EMIT_PATH}: emit must import translate stage"
    );
}

#[test]
fn v4_emit_dag_emit_composes_through_translate() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    let body = fn_body(&module, "emit").unwrap_or_else(|| panic!("{EMIT_PATH}: missing fn emit"));
    assert!(
        expr_mentions_call(body, "translate"),
        "{EMIT_PATH}: emit must call translate before serialize_target"
    );
}

#[test]
fn v4_emit_dag_does_not_import_find_witness() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        !import_paths(&module)
            .iter()
            .any(|path| path.as_slice() == ["v4", "std", "find_witness"]),
        "{EMIT_PATH}: emit must not import find_witness (delegates via translate/coercion_fold)"
    );
}

#[test]
fn v4_mvp1_rust_add_claim_tokenizes_and_parses() {
    let _module = parse_module(MVP1_CLAIM_DAG, MVP1_CLAIM_PATH);
}

#[test]
fn v4_mvp1_rust_add_claim_imports_translate_and_emit() {
    let module = parse_module(MVP1_CLAIM_DAG, MVP1_CLAIM_PATH);
    assert!(
        import_includes_name(&module, &["v4", "compiler", "translate"], "translate"),
        "{MVP1_CLAIM_PATH}: claim must import translate stage"
    );
    assert!(
        import_includes_name(&module, &["v4", "compiler", "emit"], "emit"),
        "{MVP1_CLAIM_PATH}: claim must import emit stage"
    );
}

fn module_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Module { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn import_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Import { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        let SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } = item
        else {
            return false;
        };
        item_path.len() == path.len()
            && item_path
                .iter()
                .zip(path.iter())
                .all(|(a, &b)| a.as_str() == b)
            && names.iter().any(|n| n == name)
    })
}

fn fn_body<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Option<&'a SurfaceExpr> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Fn {
            name: item_name,
            body,
            ..
        } if item_name == name => Some(body),
        _ => None,
    })
}

fn expr_mentions_call(expr: &SurfaceExpr, name: &str) -> bool {
    match expr {
        SurfaceExpr::Call { target, args, .. } => {
            target == name || args.iter().any(|arg| expr_mentions_call(arg, name))
        }
        SurfaceExpr::PathCall { segments, args, .. } => {
            segments.last().is_some_and(|tail| tail == name)
                || args.iter().any(|arg| expr_mentions_call(arg, name))
        }
        SurfaceExpr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            expr_mentions_call(cond, name)
                || expr_mentions_call(then_branch, name)
                || expr_mentions_call(else_branch, name)
        }
        SurfaceExpr::Match {
            scrutinee, arms, ..
        } => {
            expr_mentions_call(scrutinee, name)
                || arms.iter().any(|arm| expr_mentions_call(&arm.body, name))
        }
        SurfaceExpr::Lambda { body, .. } => expr_mentions_call(body, name),
        SurfaceExpr::Operator { args, .. } => args.iter().any(|arg| expr_mentions_call(arg, name)),
        SurfaceExpr::List { elements, .. } => {
            elements.iter().any(|el| expr_mentions_call(el, name))
        }
        SurfaceExpr::Record { fields, .. } => fields
            .iter()
            .any(|field| expr_mentions_call(&field.value, name)),
        SurfaceExpr::Map { entries, .. } => entries
            .iter()
            .any(|entry| expr_mentions_call(&entry.value, name)),
        SurfaceExpr::Literal { .. } | SurfaceExpr::Var { .. } | SurfaceExpr::Path { .. } => false,
        SurfaceExpr::VariantRecord { fields, .. } => fields
            .iter()
            .any(|field| expr_mentions_call(&field.value, name)),
    }
}
