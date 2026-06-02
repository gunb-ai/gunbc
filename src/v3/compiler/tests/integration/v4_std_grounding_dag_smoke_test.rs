//! **Layer:** integration
//!
//! G.0 receipt: `src/v4/std/grounding.dag` tokenizes and parses cleanly — Branch G.0 schema
//! carriers (per-language fact bundle keys, hollow-alias bar, SG evidence shapes, per-target receipt).
//! Parse smoke ratchets G.0 carriers including terminal `HollowAliasGovernanceBar` coproduct.
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
        GROUNDING_DAG.contains(
            "// 🟢 terminal — G.0 hollow-alias governance posture"
        ),
        "HollowAliasGovernanceBar must be a closed terminal coproduct (no configurable Bool policy)"
    );
    let module = grounding_surface_or_panic();
    assert_eq!(
        sum_variant_names(&module, "HollowAliasGovernanceBar"),
        vec!["HollowAliasRequiresNamedFieldsAndKernelAmbient"],
        "single mandatory governance posture — false cases unrepresentable"
    );
}

#[test]
fn v4_std_grounding_declares_a3b_testclaim_receipt_carriers() {
    let module = grounding_surface_or_panic();
    for name in [
        "HostVerdictSurfaceExecutionStatus",
        "PerTargetTestClaimReceipt",
    ] {
        assert!(
            surface_declares_type(&module, name),
            "grounding.dag must declare A.3b.3 carrier {name}"
        );
    }
}

#[test]
fn v4_std_grounding_execution_status_terminal_coproduct() {
    assert!(
        GROUNDING_DAG.contains("// 🟢 coproduct dissolution — terminal A.3b execution-provenance tag"),
        "HostVerdictSurfaceExecutionStatus must carry 🟢 terminal coproduct dissolution receipt"
    );
    let module = grounding_surface_or_panic();
    assert_eq!(
        sum_variant_names(&module, "HostVerdictSurfaceExecutionStatus"),
        vec!["RuntimeVerdicts", "AuthoringTimeVerdictSurface"],
        "two closed provenance arms — runtime vs authoring-time (RR-A §6 forbids the latter as terminal pass)"
    );
}

#[test]
fn v4_std_grounding_testclaim_receipt_single_target_authority() {
    let module = grounding_surface_or_panic();
    assert_eq!(
        type_record_field_names(&module, "PerTargetTestClaimReceipt"),
        vec!["host_run", "execution_status", "verdict_tally"],
        "TargetModel authority must be EmitHostRunReceipt.target only (P2); verdict surface is the folded VerdictTally"
    );
}

#[test]
fn v4_std_grounding_testclaim_terminal_gate_fails_closed_on_authoring_time() {
    // RR-A §4 R1/R6: only RuntimeVerdicts is a terminal CI pass; authoring-time fails closed.
    assert!(
        GROUNDING_DAG.contains("fn host_verdict_surface_runtime_authoritative("),
        "grounding.dag must expose the RuntimeVerdicts terminal-pass gate"
    );
    assert!(
        GROUNDING_DAG.contains("RuntimeVerdicts => true")
            && GROUNDING_DAG.contains("AuthoringTimeVerdictSurface => false"),
        "terminal gate must accept RuntimeVerdicts and fail closed on AuthoringTimeVerdictSurface"
    );
    // R1–R3 stay OPEN until the step-2 harness lands — dissolution mark must say so (no premature PROVEN).
    assert!(
        GROUNDING_DAG.contains("R1–R3")
            && GROUNDING_DAG.contains("step-2 harness")
            && GROUNDING_DAG.contains("does NOT itself run"),
        "A.3b.3 must mark R1–R3 OPEN until the Compiler Spine step-2 harness lands (no authoring-time co-authority)"
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
        "registry key must use TargetModel + ModelCorePrimitiveFactAxis (P2)"
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
            "// 🟢 coproduct dissolution — terminal G.0 evidence-family tag"
        ),
        "GroundingEvidenceSchema must carry 🟢 coproduct dissolution receipt (G.0 substrate discipline)"
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
        vec!["by_key"],
        "registry must be Map<PerLanguageFactBundleKey, Node> — not List (duplicate keys unrepresentable)"
    );
    assert!(
        GROUNDING_DAG.contains("fn insert_per_language_fact_bundle_entry("),
        "fail-closed registry insert rejects duplicate PerLanguageFactBundleKey"
    );
}

#[test]
fn v4_std_grounding_registry_entries_do_not_project_incomplete_primitive_bundles() {
    assert!(
        GROUNDING_DAG.contains("fact_axis: ModelCorePrimitiveFactAxis"),
        "illegal fact axes unrepresentable via model_core closed coproduct (P2)"
    );
    assert!(
        GROUNDING_DAG.contains("model_core_primitive_fact_axis_symbol"),
        "registry key can project the closed axis to model_core Symbol authority when an aggregation builder lands"
    );
    assert!(
        !GROUNDING_DAG.contains("fn primitive_fact_bundle_for_entry(")
            && !GROUNDING_DAG.contains("-> PrimitiveFactBundle"),
        "single fact-axis registry entries must not project directly to complete PrimitiveFactBundle carriers"
    );
    assert!(
        GROUNDING_DAG.contains("map_get(m: registry.by_key, key: key)"),
        "registry insert must use v4.std.collection map_get authority (B-LOOKUP-1 / P2)"
    );
    assert!(
        GROUNDING_DAG.contains("Present { value: _ }") && GROUNDING_DAG.contains("Absent =>"),
        "duplicate-key gate: Present => reject, Absent => accept insert"
    );
    assert!(
        GROUNDING_DAG.contains("Rejected { diagnostics: ds } => Rejected { diagnostics: ds }"),
        "map_get lookup failures propagate without collapsing to Absent (P3)"
    );
}
