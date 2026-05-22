//! **Layer:** integration
//!
//! T-19 witness-validity generator category structural guard. Pins:
//!   1. `src/v4/lens/testgen.dag` and `src/v4/test/claim/generated/witness_validity.dag`
//!      both tokenize and parse.
//!   2. `testgen_emit_witness_validity_claim` exists in `v4.lens.testgen` and returns
//!      `Outcome<TestClaim>`.
//!   3. The generated module imports the helper from `v4.lens.testgen` — i.e. rows
//!      route through the category helper, not through bespoke peer construction.
//!   4. The generated module contains at least three `data row_witness_validity_*` rows,
//!      and every row is constructed by calling the helper (no bare `EqualsClaim {` /
//!      `DiagnosticClaim {` / `CompilesClaim {` / `RoundTripClaim {` literals appear in
//!      the file body). This represents the tautology-skip discipline for the category:
//!      the oracle (`verify_witness`) — not the row author — decides claim polarity.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceItem, SurfaceType, TypeAngleArg};
use v3_compiler::tokenize_for_test;

const TESTGEN_DAG: &str = include_str!("../../../../v4/lens/testgen.dag");
const TESTGEN_PATH: &str = "src/v4/lens/testgen.dag";
const WITNESS_VALIDITY_DAG: &str =
    include_str!("../../../../v4/test/claim/generated/witness_validity.dag");
const WITNESS_VALIDITY_PATH: &str = "src/v4/test/claim/generated/witness_validity.dag";

#[test]
fn witness_validity_modules_tokenize_and_parse() {
    parse_module(TESTGEN_DAG, TESTGEN_PATH);
    parse_module(WITNESS_VALIDITY_DAG, WITNESS_VALIDITY_PATH);
}

#[test]
fn witness_validity_helper_returns_outcome_testclaim() {
    let testgen = parse_module(TESTGEN_DAG, TESTGEN_PATH);
    let rt = fn_return_type(&testgen, "testgen_emit_witness_validity_claim")
        .expect("testgen_emit_witness_validity_claim must be declared in v4.lens.testgen");
    assert!(
        type_is_outcome_named(rt, "TestClaim"),
        "testgen_emit_witness_validity_claim must return `Outcome<TestClaim>`; got {rt:?}"
    );
}

#[test]
fn witness_validity_module_imports_helper_from_lens_testgen() {
    let module = parse_module(WITNESS_VALIDITY_DAG, WITNESS_VALIDITY_PATH);
    let names = import_names_for_path(&module, &["v4", "lens", "testgen"]).expect(
        "witness_validity.dag must import from `v4.lens.testgen` (rows route through helper)",
    );
    assert!(
        names
            .iter()
            .any(|n| n == "testgen_emit_witness_validity_claim"),
        "witness_validity.dag must import `testgen_emit_witness_validity_claim`; got {names:?}"
    );
}

#[test]
fn witness_validity_module_has_at_least_three_rows_via_helper() {
    let helper_calls = WITNESS_VALIDITY_DAG
        .matches("testgen_emit_witness_validity_claim(")
        .count();
    assert!(
        helper_calls >= 3,
        "witness_validity.dag must contain ≥3 rows constructed via testgen_emit_witness_validity_claim; got {helper_calls}"
    );
    let row_data_decls = WITNESS_VALIDITY_DAG
        .matches("data row_witness_validity_")
        .count();
    assert!(
        row_data_decls >= 3,
        "witness_validity.dag must declare ≥3 `data row_witness_validity_*` rows; got {row_data_decls}"
    );
}

#[test]
fn witness_validity_module_authors_no_testclaim_literals() {
    for literal in [
        "EqualsClaim {",
        "DiagnosticClaim {",
        "CompilesClaim {",
        "RoundTripClaim {",
    ] {
        assert!(
            !WITNESS_VALIDITY_DAG.contains(literal),
            "witness_validity.dag must not author `{literal}` literals — claim polarity is decided by verify_witness inside the helper (tautology-skip discipline)"
        );
    }
}

fn parse_module(source: &str, file: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}

fn fn_return_type<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Option<&'a SurfaceType> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Fn {
            name: item_name,
            return_type,
            ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name,
            return_type,
            ..
        } => (item_name == name).then_some(return_type),
        _ => None,
    })
}

fn import_names_for_path<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
) -> Option<&'a [String]> {
    module.items.iter().find_map(|item| {
        let SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } = item
        else {
            return None;
        };
        if item_path.len() != path.len() {
            return None;
        }
        item_path
            .iter()
            .zip(path.iter())
            .all(|(a, &b)| a.as_str() == b)
            .then_some(names.as_slice())
    })
}

fn type_is_outcome_named(ty: &SurfaceType, inner_name: &str) -> bool {
    let SurfaceType::Parameterized { name, args, .. } = ty else {
        return false;
    };
    if name != "Outcome" || args.len() != 1 {
        return false;
    }
    let TypeAngleArg::TypeExpr { ty: inner } = &args[0] else {
        return false;
    };
    matches!(
        inner.as_ref(),
        SurfaceType::Named { name: n, .. } if n == inner_name
    )
}
