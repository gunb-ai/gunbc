//! **Layer:** integration
//!
//! T-7 receipt: `src/v4/compiler/02_parse.dag` ParseTable memoization (`build_parse_table`,
//! `parse_production`) and `src/v4/test/claim/parse/grammar_validation.dag` parse-success claim
//! for a right-recursive grammar.
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS table row land in the same PR.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse gate; full T-22 claim execution deferred until eval
//! runner lands (same posture as peer v4 smoke tests).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceItem, SurfaceType};
use v3_compiler::tokenize_for_test;

const PARSE_DAG: &str = include_str!("../../../../v4/compiler/02_parse.dag");
const PARSE_DAG_PATH: &str = "src/v4/compiler/02_parse.dag";
const CLAIM_DAG: &str = include_str!("../../../../v4/test/claim/parse/grammar_validation.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/parse/grammar_validation.dag";
const ALGEBRA_DAG: &str = include_str!("../../../../v4/std/algebra.dag");
const ALGEBRA_PATH: &str = "src/v4/std/algebra.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
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

fn surface_declares_type_sum(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::TypeSum {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn surface_declares_test_claim_data(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Data {
            name: item_name,
            ty: SurfaceType::Named { name: ty_name, .. },
            ..
        } => item_name == name && ty_name == "TestClaim",
        _ => false,
    })
}

#[test]
fn v4_compiler_parse_table_dag_tokenizes_and_parses() {
    let _parse = parse_module(PARSE_DAG, PARSE_DAG_PATH);
    let _claim = parse_module(CLAIM_DAG, CLAIM_PATH);
    let _algebra = parse_module(ALGEBRA_DAG, ALGEBRA_PATH);
}

#[test]
fn v4_compiler_parse_table_entrypoints_and_claim_wiring() {
    let parse = parse_module(PARSE_DAG, PARSE_DAG_PATH);
    // P5 receipt: smoke-test assertion expansion for Lane A analyses fixes — token_position_indices
    // duplicate-append removal (Finding #6) and compile_ingest_staging mandatory-lens bypass (Finding #1).
    // No new Rust logic, no scaffold; these assertions verify the .dag source shape of T-7/T-36 scope.
    // Deferral: assertion surface retired when corresponding .dag assertions replace Rust smoke tests.
    for name in [
        "build_parse_table",
        "parse_production",
        "parse_table_empty",
        "token_position_indices",
    ] {
        assert!(
            surface_declares_fn(&parse, name),
            "{PARSE_DAG_PATH}: must declare {name}"
        );
    }
    assert!(
        !PARSE_DAG.contains("item: length(xs: tokens)"),
        "{PARSE_DAG_PATH}: token_position_indices fold must produce 0..N without an extra final append"
    );

    let algebra = parse_module(ALGEBRA_DAG, ALGEBRA_PATH);
    assert!(
        surface_declares_type_sum(&algebra, "ListTailResult"),
        "{ALGEBRA_PATH}: ListTailResult fail-closed tail projection"
    );

    let claim = parse_module(CLAIM_DAG, CLAIM_PATH);
    assert!(
        surface_declares_test_claim_data(&claim, "claim_right_recursive_parse_succeeds"),
        "{CLAIM_PATH}: parse-table success receipt claim"
    );
    assert!(
        surface_declares_fn(&claim, "validation_right_recursive_parse_fixture"),
        "{CLAIM_PATH}: fixture must call parse_production over memoized ParseTable path"
    );
    assert!(
        CLAIM_DAG.contains("parse_production("),
        "{CLAIM_PATH}: claim must exercise parse_production"
    );
}
