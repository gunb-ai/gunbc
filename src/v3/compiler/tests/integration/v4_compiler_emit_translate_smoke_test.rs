//! **Layer:** integration
//!
//! **T-10 / Wave-3-B:** `06_translate.dag` + `05_emit.dag` + `mvp1_rust_add_translate.dag`
//! tokenize/parse cleanly (M1(2.7) single-file path; full `compile_to_dag` deferred until
//! multi-module v4 load lands). Peers: `v4_bin_main_dag_smoke_test.rs`, `v4_extdeps_file_system_dag_smoke_test.rs`.
//!
//! **TESTING.md:** M1(2.7) `.dag` brace-bodied `fn` items surface as `FnExternalBody` (no
//! expression AST), so call-site contracts are checked via **import rows** and **declared `fn`
//! inventory** — not raw `str::contains` probes. Semantic substantiation deferred to T-22/T-14.
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
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const FIND_WITNESS_DAG: &str = include_str!("../../../../v4/std/find_witness.dag");
const FIND_WITNESS_PATH: &str = "src/v4/std/find_witness.dag";
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
fn v4_find_witness_dag_tokenizes_and_parses() {
    let _module = parse_module(FIND_WITNESS_DAG, FIND_WITNESS_PATH);
}

#[test]
fn v4_find_witness_dag_declares_find_witness_entrypoint() {
    let module = parse_module(FIND_WITNESS_DAG, FIND_WITNESS_PATH);
    assert!(
        surface_declares_fn(&module, "find_witness"),
        "{FIND_WITNESS_PATH}: must declare find_witness primitive"
    );
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
fn v4_translate_dag_imports_coercion_fold_delegate() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "coercion"], "coercion_fold"),
        "{TRANSLATE_PATH}: must import coercion_fold from v4.std.coercion (Practice 11)"
    );
}

#[test]
fn v4_translate_dag_imports_fold_node_traversal() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "node"], "fold_node"),
        "{TRANSLATE_PATH}: must import fold_node from v4.std.node"
    );
}

#[test]
fn v4_translate_dag_declares_coerce_grounded_node() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        surface_declares_fn(&module, "coerce_grounded_node"),
        "{TRANSLATE_PATH}: must declare coerce_grounded_node wrapper"
    );
}

#[test]
fn v4_translate_dag_declares_translate_node_and_translate() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        surface_declares_fn(&module, "translate_node"),
        "{TRANSLATE_PATH}: must declare translate_node fold entry"
    );
    assert!(
        surface_declares_fn(&module, "translate"),
        "{TRANSLATE_PATH}: must declare translate stage entry"
    );
}

#[test]
fn v4_translate_dag_imports_find_witness_types_not_inline_fn() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(&module, &["v4", "std", "find_witness"], "CandidateSet"),
        "{TRANSLATE_PATH}: may import find_witness carrier types"
    );
    assert!(
        !import_includes_name(&module, &["v4", "std", "find_witness"], "find_witness"),
        "{TRANSLATE_PATH}: must not import find_witness fn (delegates via coercion_fold)"
    );
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
        "{EMIT_PATH}: emit must import translate stage (serialize_target ∘ translate)"
    );
}

#[test]
fn v4_emit_dag_declares_emit_entrypoint() {
    let module = parse_module(EMIT_DAG, EMIT_PATH);
    assert!(
        surface_declares_fn(&module, "emit"),
        "{EMIT_PATH}: must declare emit entrypoint"
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
