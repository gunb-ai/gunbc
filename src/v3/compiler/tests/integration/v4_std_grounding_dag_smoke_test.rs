//! **Layer:** integration
//!
//! G.0 receipt: `src/v4/std/grounding.dag` tokenizes and parses cleanly - Branch G.0 schema
//! carriers (per-language fact bundle keys, hollow-alias bar, SG evidence shapes, per-target receipt).
//! Parse smoke ratchets G.0 carriers including terminal `HollowAliasGovernanceBar` coproduct and
//! subject-level fact-bundle aggregation into `model_core`.
//! Full `compile_to_dag` on this module alone does not resolve `import v4.std.*` peers; cross-module
//! resolution is exercised by M1 v4 full-tree emit in CI `ci_floor` (same posture as
//! `v4_std_model_core_dag_smoke_test`).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem};
use v3_compiler::tokenize_for_test;

const GROUNDING_DAG: &str = include_str!("../../../../v4/std/grounding.dag");
const GROUNDING_PATH: &str = "src/v4/std/grounding.dag";

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

fn grounding_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(GROUNDING_DAG, GROUNDING_PATH)
        .unwrap_or_else(|e| panic!("{GROUNDING_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, GROUNDING_PATH)
        .unwrap_or_else(|e| panic!("{GROUNDING_PATH}: parse: {e:?}"))
}

fn type_record_field_names<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Vec<&'a str> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(
                fields
                    .iter()
                    .map(|SurfaceField { name, .. }| name.as_str())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

fn sum_variant_names<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Vec<&'a str> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum {
                name: item_name,
                variants,
                ..
            } if item_name == name => {
                Some(variants.iter().map(|v| v.name.as_str()).collect::<Vec<_>>())
            }
            _ => None,
        })
        .unwrap_or_default()
}

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum { name: decl_name, .. }
                | SurfaceItem::TypeRecord { name: decl_name, .. }
                | SurfaceItem::TypeAlias { name: decl_name, .. }
                | SurfaceItem::TypeAtom { name: decl_name, .. }
                if decl_name == name
        )
    })
}

#[test]
fn v4_std_grounding_dag_parses() {
    let module = grounding_surface_or_panic();
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "std", "grounding"]],
        "grounding.dag module path"
    );
}

#[test]
fn v4_std_grounding_declares_g0_carriers() {
    let module = grounding_surface_or_panic();
    for name in [
        "PerLanguageFactBundleKey",
        "HollowAliasGovernanceBar",
        "GroundingEvidenceSchema",
        "PerTargetGroundingReceipt",
    ] {
        assert!(
            surface_declares_type(&module, name),
            "grounding.dag must declare {name}"
        );
    }
}

#[test]
fn v4_std_grounding_hollow_alias_bar_terminal_coproduct() {
    assert!(
        GROUNDING_DAG.contains("// \u{1f7e2} terminal")
            && GROUNDING_DAG.contains("G.0 hollow-alias governance posture"),
        "HollowAliasGovernanceBar must be a closed terminal coproduct (no configurable Bool policy)"
    );
    let module = grounding_surface_or_panic();
    assert_eq!(
        sum_variant_names(&module, "HollowAliasGovernanceBar"),
        vec!["HollowAliasRequiresNamedFieldsAndKernelAmbient"],
        "single mandatory governance posture - false cases unrepresentable"
    );
}

#[test]
fn v4_std_grounding_receipt_single_target_authority() {
    let module = grounding_surface_or_panic();
    assert_eq!(
        type_record_field_names(&module, "PerTargetGroundingReceipt"),
        vec!["host_run"],
        "target authority must be EmitHostRunReceipt.target only (P2)"
    );
}

#[test]
fn v4_std_grounding_key_indexes_target_and_fact_axis() {
    let module = grounding_surface_or_panic();
    assert_eq!(
        type_record_field_names(&module, "PerLanguageFactBundleKey"),
        vec!["subject_carrier", "target", "fact_axis"],
        "registry key must use TargetModel + typed canonical fact axis (P2)"
    );
    assert!(
        GROUNDING_DAG.contains("fact_axis: ModelCorePrimitiveFactAxis"),
        "PerLanguageFactBundleKey.fact_axis must be typed, not a raw Symbol predicate/tag gate"
    );
}

#[test]
fn v4_std_grounding_entry_single_fact_authority() {
    let module = grounding_surface_or_panic();
    assert_eq!(
        type_record_field_names(&module, "PerLanguageFactBundleEntry"),
        vec!["key", "fact_value"],
        "fact payload must not be a free Map diverging from key.fact_axis (P2)"
    );
}

#[test]
fn v4_std_grounding_evidence_schema_terminal_coproduct() {
    assert!(
        GROUNDING_DAG.contains(
            "// \u{1f7e2} coproduct dissolution"
        ),
        "GroundingEvidenceSchema must carry coproduct dissolution receipt (G.0 substrate discipline)"
    );
    let module = grounding_surface_or_panic();
    assert_eq!(
        sum_variant_names(&module, "GroundingEvidenceSchema"),
        vec!["Sg1Evidence", "Sg1bEvidence", "Sg2Evidence", "Sg5Evidence"],
        "four closed SG family arms (SG-1 / SG-1b / SG-2 / SG-5)"
    );
    assert!(
        GROUNDING_DAG
            .contains("Sg1bEvidence { source_carrier: Node, boundary_site: FunctionBoundarySite }"),
        "Sg1b arm must include boundary_site (target_model signature-realization lookup key)"
    );
}

#[test]
fn v4_std_grounding_registry_keyed_map_authority() {
    let module = grounding_surface_or_panic();
    assert!(surface_declares_type(
        &module,
        "PerLanguageFactBundleRegistry"
    ));
    assert_eq!(
        type_record_field_names(&module, "PerLanguageFactBundleRegistry"),
        vec!["by_key", "entries"],
        "registry must retain duplicate-key authority and ordered entries for subject aggregation"
    );
    assert!(
        GROUNDING_DAG.contains("by_key: Map<PerLanguageFactBundleKey, Node>")
            && GROUNDING_DAG.contains("entries: List<PerLanguageFactBundleEntry>"),
        "registry stores keyed facts and the complete row set for model_core projection"
    );
    assert!(
        GROUNDING_DAG.contains("fn insert_per_language_fact_bundle_entry(")
            && GROUNDING_DAG.contains("feature:B-LOOKUP-1")
            && GROUNDING_DAG.contains("match map_get(registry.by_key, key)"),
        "fail-closed registry insert rejects duplicate PerLanguageFactBundleKey"
    );
}

#[test]
fn v4_std_grounding_primitive_fact_bundle_model_core_projection_aggregates_subject() {
    assert!(
        GROUNDING_DAG.contains("fn primitive_fact_bundle_for_subject(")
            && GROUNDING_DAG.contains("fn per_language_fact_bundle_spec_facts_for_subject("),
        "registry-to-model_core projection must build one PrimitiveFactBundle per subject x target"
    );
    assert!(
        GROUNDING_DAG.contains("fold_list(") && GROUNDING_DAG.contains("xs: registry.entries"),
        "PrimitiveFactBundle.spec_facts must aggregate all matching registry entries"
    );
    assert!(
        GROUNDING_DAG.contains("model_core_primitive_fact_axis_symbol(axis: entry.key.fact_axis)"),
        "spec_facts projection uses model_core Symbol authority at bundle boundary"
    );
    assert!(
        !GROUNDING_DAG.contains("fn primitive_fact_bundle_for_entry("),
        "per-entry projection would split a subject's axes across multiple PrimitiveFactBundle rows"
    );
}
