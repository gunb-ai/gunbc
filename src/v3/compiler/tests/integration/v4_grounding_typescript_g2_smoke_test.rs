//! **Layer:** integration
//!
//! G.2 receipt: `grounding_typescript/*` T-38B claim family parses and declares the four-row
//! SG-1 / G.1.4 / SG-2 / SG-5 executable claim stack (parse-surface only).
//!
//! **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — SG-0 `EXPECTED_HAND_AUTHORED_TEST`):**
//! explicit deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:62); dissolves when substrate
//! `run_test_claim` executes the grounding_typescript roster without this hand-Rust parse smoke.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const SG_CLAIMS_DAG: &str =
    include_str!("../../../../v4/test/claim/grounding_typescript/sg_claims.dag");
const SG_CLAIMS_PATH: &str = "src/v4/test/claim/grounding_typescript/sg_claims.dag";

const SUBJECT_ROSTER_DAG: &str =
    include_str!("../../../../v4/test/claim/grounding_typescript/subject_roster.dag");
const SUBJECT_ROSTER_PATH: &str = "src/v4/test/claim/grounding_typescript/subject_roster.dag";

const FAMILY_RECEIPT_DAG: &str =
    include_str!("../../../../v4/test/claim/grounding_typescript/family_receipt.dag");
const FAMILY_RECEIPT_PATH: &str = "src/v4/test/claim/grounding_typescript/family_receipt.dag";

fn extract_fn_body<'a>(source: &'a str, fn_name: &str) -> &'a str {
    let marker = format!("fn {fn_name}(");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("{SG_CLAIMS_PATH}: missing `{marker}`"));
    let rest = &source[start + marker.len()..];
    let end = rest
        .find("\nfn ")
        .or_else(|| rest.find("\ndata "))
        .unwrap_or(rest.len());
    &source[start..start + marker.len() + end]
}

fn parse_surface(path: &str, source: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn surface_declares_data(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::Data { name: decl_name, .. } if decl_name == name
        )
    })
}

#[test]
fn v4_grounding_typescript_g2_modules_parse() {
    let sg = parse_surface(SG_CLAIMS_PATH, SG_CLAIMS_DAG);
    assert_eq!(
        sg.items
            .iter()
            .find_map(|item| match item {
                SurfaceItem::Module { path, .. } => {
                    Some(path.iter().map(String::as_str).collect::<Vec<_>>())
                }
                _ => None,
            })
            .expect("sg_claims module path"),
        vec!["v4", "test", "claim", "grounding_typescript", "sg_claims"]
    );
    parse_surface(SUBJECT_ROSTER_PATH, SUBJECT_ROSTER_DAG);
    parse_surface(FAMILY_RECEIPT_PATH, FAMILY_RECEIPT_DAG);
}

#[test]
fn v4_grounding_typescript_g2_declares_executable_claim_stack() {
    let sg = parse_surface(SG_CLAIMS_PATH, SG_CLAIMS_DAG);
    assert!(
        surface_declares_data(&sg, "run_ts_g2_sg1_atom_realizations"),
        "G.2 must wire SG-1 run_test_claim row"
    );
    assert!(
        surface_declares_data(&sg, "run_ts_g2_g14_fact_registry"),
        "G.2 must wire G.1.4 registry run_test_claim row"
    );
    assert!(
        surface_declares_data(&sg, "run_ts_g2_sg2_type_expression_projection"),
        "G.2 must wire SG-2 positive projection run_test_claim row"
    );
    assert!(
        surface_declares_data(&sg, "run_ts_g2_sg5_collection_absence"),
        "G.2 must wire SG-5 absence run_test_claim row"
    );
    let sg2_hold =
        extract_fn_body(SG_CLAIMS_DAG, "ts_g2_sg2_type_expression_projection_holds");
    assert!(
        sg2_hold.contains("Accepted { value: ProjectionPresent, diagnostics: _ }"),
        "SG-2 hold predicate must match ProjectionPresent (not import-only occurrence)"
    );
    assert!(
        SG_CLAIMS_DAG.contains("feature:G-2-grounding-evidence-bool-bridge")
            && SG_CLAIMS_DAG.contains("feature:T-22-outcome-to-bool-claim-bridge"),
        "G.2 claim predicates must carry predicate-dissolution disposition receipts"
    );
}
