//! **Layer:** integration
//!
//! Branch H.7.2 source-authority shape contract plus B-min Bmin.4-Bmin.5
//! canonical `.dag` source parse/print law: the compiler surface must model
//! canonical `.dag` source round trips as source text plus normalized source AST/IR
//! equality, without treating `dag-artifact.json` or `--target dag` JSON IR as source
//! authority. This smoke is intentionally parser-level until substrate equality and
//! canonical `.dag` `TargetModel` execution land.
//!
//! **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — SG-0 `EXPECTED_HAND_AUTHORED_TEST`):**
//! explicit deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero` (ROADMAP.md:63); dissolves when a `.dag`
//! TestClaim or generated harness executes the Branch H.7.2 source-authority receipt directly.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const SOURCE_AUTHORITY_DAG: &str = include_str!("../../../../v4/compiler/source_authority.dag");
const SOURCE_AUTHORITY_PATH: &str = "src/v4/compiler/source_authority.dag";
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/round_trip/source_authority_contract.dag");
const CLAIM_PATH: &str = "src/v4/test/claim/round_trip/source_authority_contract.dag";

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

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::TypeRecord {
            name: item_name, ..
        }
        | SurfaceItem::TypeAlias {
            name: item_name, ..
        } => item_name == name,
        _ => false,
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

#[test]
fn v4_source_authority_contract_tokenizes_and_parses() {
    let _ = parse_module(SOURCE_AUTHORITY_DAG, SOURCE_AUTHORITY_PATH);
    let _ = parse_module(CLAIM_DAG, CLAIM_PATH);
}

#[test]
fn v4_source_authority_contract_uses_source_and_serializer_authorities() {
    let module = parse_module(SOURCE_AUTHORITY_DAG, SOURCE_AUTHORITY_PATH);
    assert_eq!(
        module_path(&module),
        vec!["v4", "compiler", "source_authority"],
        "{SOURCE_AUTHORITY_PATH}: module path"
    );
    assert!(
        surface_declares_type(&module, "SourceAuthorityRoundTripLaw"),
        "{SOURCE_AUTHORITY_PATH}: round-trip law type"
    );
    assert!(
        SOURCE_AUTHORITY_DAG.contains("source_law: SourceAstEqual"),
        "{SOURCE_AUTHORITY_PATH}: law must carry proven source equality payload"
    );
    assert!(
        SOURCE_AUTHORITY_DAG.contains("semantic_law: SemanticIrEqual"),
        "{SOURCE_AUTHORITY_PATH}: law must carry proven semantic equality payload"
    );
    assert!(
        SOURCE_AUTHORITY_DAG.contains("canonical_source_law: CanonicalDagSourceEmissionLaw"),
        "{SOURCE_AUTHORITY_PATH}: round-trip law must carry the B-min canonical source emission law"
    );
    assert!(
        surface_declares_type(&module, "NormalizedDagSourceAst"),
        "{SOURCE_AUTHORITY_PATH}: B-min law must name normalized .dag source AST explicitly"
    );
    assert!(
        SOURCE_AUTHORITY_DAG.contains("type NormalizedDagSourceAst {\n  tree: NormalizedTree")
            && SOURCE_AUTHORITY_DAG.contains("o: normalize(parse_tree: ast.tree)"),
        "{SOURCE_AUTHORITY_PATH}: normalized source AST must consume the compiler normalize stage"
    );
    assert!(
        surface_declares_type(&module, "NormalizedDagSourceParsePrintLaw"),
        "{SOURCE_AUTHORITY_PATH}: B-min law must model parse/print over normalized source AST"
    );
    assert!(
        surface_declares_type(&module, "DeterministicDagSourceSerializerLaw"),
        "{SOURCE_AUTHORITY_PATH}: Bmin.5 must model deterministic canonical source serialization"
    );
    assert!(
        !SOURCE_AUTHORITY_DAG.contains("type SourceAuthorityRoundTripReceipt"),
        "{SOURCE_AUTHORITY_PATH}: receipt carrier must not duplicate forgeable law fields"
    );
    assert!(
        SOURCE_AUTHORITY_DAG.contains(") -> Outcome<Witness<SourceAuthorityRoundTripLaw>>"),
        "{SOURCE_AUTHORITY_PATH}: public boundary must stay fail-closed over the round-trip law witness"
    );
    assert!(
        SOURCE_AUTHORITY_DAG.contains("Violates { diagnostic: d } =>\n      Rejected")
            && SOURCE_AUTHORITY_DAG.contains("Violates { diagnostic: d } =>\n          Rejected"),
        "{SOURCE_AUTHORITY_PATH}: equality witness violations must reject before accepted law output"
    );
    assert!(
        surface_declares_fn(&module, "source_authority_round_trip"),
        "{SOURCE_AUTHORITY_PATH}: round-trip entrypoint"
    );
    assert!(
        surface_declares_fn(&module, "canonical_dag_source_parse_print_law"),
        "{SOURCE_AUTHORITY_PATH}: Bmin.4-Bmin.5 entrypoint"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "compiler", "translate"],
            "target_serialize_source_from_model"
        ),
        "{SOURCE_AUTHORITY_PATH}: must use canonical source serializer"
    );
    assert!(
        import_includes_name(&module, &["v4", "compiler", "parse"], "parse"),
        "{SOURCE_AUTHORITY_PATH}: must reparse canonical source text"
    );
    assert!(
        !SOURCE_AUTHORITY_DAG.contains("dag-artifact.json"),
        "{SOURCE_AUTHORITY_PATH}: JSON debug artifact must not be source authority"
    );
    assert!(
        !SOURCE_AUTHORITY_DAG.contains("--target dag"),
        "{SOURCE_AUTHORITY_PATH}: target dag JSON IR equality must not be source authority"
    );
}

#[test]
fn v4_source_authority_claim_imports_contract_entrypoint() {
    let claim = parse_module(CLAIM_DAG, CLAIM_PATH);
    assert!(
        import_includes_name(
            &claim,
            &["v4", "compiler", "source_authority"],
            "source_authority_round_trip"
        ),
        "{CLAIM_PATH}: claim must import source_authority_round_trip"
    );
    assert!(
        import_includes_name(
            &claim,
            &["v4", "compiler", "source_authority"],
            "canonical_dag_source_parse_print_law"
        ),
        "{CLAIM_PATH}: B-min claim must import canonical_dag_source_parse_print_law"
    );
    assert!(
        CLAIM_DAG.contains("source_authority_round_trip_boundary_holds()"),
        "{CLAIM_PATH}: claim input must structurally depend on the source-authority boundary"
    );
    assert!(
        CLAIM_DAG.contains("canonical_dag_source_parse_print_boundary_holds()"),
        "{CLAIM_PATH}: B-min claim input must structurally depend on the parse/print law boundary"
    );
    assert!(
        CLAIM_DAG.contains("Accepted { value: witness, diagnostics: _ }")
            && CLAIM_DAG.contains("Holds { value: _law }"),
        "{CLAIM_PATH}: claim must project the round-trip witness to pass/fail claim input"
    );
    assert!(
        CLAIM_DAG.contains("claim_bmin_canonical_dag_source_parse_print_law"),
        "{CLAIM_PATH}: must declare the Bmin.4-Bmin.5 canonical source law claim"
    );
    assert!(
        !CLAIM_DAG.contains("dag-artifact.json") && !CLAIM_DAG.contains("--target dag"),
        "{CLAIM_PATH}: claim must not cite JSON IR source-fidelity evidence"
    );
}
