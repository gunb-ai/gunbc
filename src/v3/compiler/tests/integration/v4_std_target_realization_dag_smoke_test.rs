//! **Layer:** integration
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + `_internal/INVARIANTS_OPS.md` row land in the same PR.
//! **This PR (+0 SG-0 paths):** same-file expansion for SG-5/SG-6 collection-bounded-lattice
//! worksheet closure (substrate on `main` #3957 / #4085) — three additional parse/tokenize
//! smokes only; no new hand-Rust logic beyond `parse_for_test` surface checks.
//!
//! SG-1 + SG-5/SG-6 receipt: `src/v4/std/target_model.dag` — `TargetAtomRealization` and
//! `TargetCollectionRealization` carriers; `bounded_lattice_completeness.dag` + `04_infer` gate;
//! Rust rows in `extdeps/languages/rust.dag`; `06_translate.dag` consumers.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceType};
use v3_compiler::tokenize_for_test;

const TARGET_MODEL_DAG: &str = include_str!("../../../../v4/std/target_model.dag");
const TARGET_MODEL_PATH: &str = "src/v4/std/target_model.dag";
const TRANSLATE_DAG: &str = include_str!("../../../../v4/compiler/06_translate.dag");
const TRANSLATE_PATH: &str = "src/v4/compiler/06_translate.dag";
const RUST_LANGUAGE_DAG: &str = include_str!("../../../../v4/extdeps/languages/rust.dag");
const RUST_LANGUAGE_PATH: &str = "src/v4/extdeps/languages/rust.dag";
const BOUNDED_LATTICE_COMPLETENESS_DAG: &str =
    include_str!("../../../../v4/std/bounded_lattice_completeness.dag");
const BOUNDED_LATTICE_COMPLETENESS_PATH: &str = "src/v4/std/bounded_lattice_completeness.dag";
const INFER_DAG: &str = include_str!("../../../../v4/compiler/04_infer.dag");
const INFER_PATH: &str = "src/v4/compiler/04_infer.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
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

fn surface_declares_data(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module
        .items
        .iter()
        .any(|item| matches!(item, SurfaceItem::Data { name: decl_name, .. } if decl_name == name))
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered = args
                .iter()
                .map(|arg| match arg {
                    v3_compiler::parse_surface::TypeAngleArg::TypeExpr { ty } => {
                        surface_type_name(ty)
                    }
                    v3_compiler::parse_surface::TypeAngleArg::WidthNatLiteral {
                        decimal, ..
                    } => decimal.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered}>")
        }
        other => format!("{other:?}"),
    }
}

fn type_record_fields(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Vec<(String, String)> {
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
                    .map(|f: &SurfaceField| (f.name.clone(), surface_type_name(&f.ty)))
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Import {
            path: import_path,
            names,
            ..
        } => {
            import_path.iter().map(String::as_str).collect::<Vec<_>>() == path
                && names.iter().any(|n| n == name)
        }
        _ => false,
    })
}

#[test]
fn v4_std_target_realization_dag_tokenizes_and_parses() {
    let _module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
}

#[test]
fn v4_std_target_realization_declares_target_atom_realization_carrier() {
    let module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
    assert!(
        surface_declares_type(&module, "TargetAtomRealization"),
        "{TARGET_MODEL_PATH}: must declare TargetAtomRealization"
    );
    let fields = type_record_fields(&module, "TargetAtomRealization");
    assert!(
        fields.iter().any(|(n, _)| n == "source_carrier"),
        "source_carrier must be Node-keyed authority"
    );
    assert!(
        fields
            .iter()
            .any(|(n, ty)| n == "type_form" && ty.contains("TargetTypeExpression")),
        "type_form must use SG-2 TargetTypeExpression {{ kind, node }} (M6 product; no parallel atom vocab)"
    );
    assert!(
        fields
            .iter()
            .any(|(n, ty)| n == "value_form" && ty.contains("TargetValueTemplate")),
        "value_form must be parametric TargetValueTemplate"
    );
    assert!(
        fields.iter().any(|(n, _)| n == "target_model"),
        "target_model must flow through catalog encode/decode"
    );
    assert!(
        fields.iter().any(|(n, _)| n == "constructor_form"),
        "constructor_form must not be dropped at catalog boundary"
    );
}

#[test]
fn v4_std_target_realization_declares_catalog_lookup() {
    let module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
    assert!(
        surface_declares_fn(&module, "target_atom_realization_lookup_in_catalog_node"),
        "catalog lookup must be structural over encoded row nodes"
    );
    assert!(
        surface_declares_fn(&module, "target_atom_type_spelling"),
        "type and value consumers share row authority via target_atom_type_spelling"
    );
    assert!(
        surface_declares_fn(&module, "target_atom_value_expression"),
        "value_form application must be row-driven"
    );
    assert!(
        surface_declares_fn(&module, "target_value_expression_node"),
        "TargetValueExpression must have a canonical Node projection (Practice 3)"
    );
}

#[test]
fn v4_translate_dag_imports_target_atom_realization_consumer() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_atom_realization_lookup_in_catalog_node"
        ),
        "{TRANSLATE_PATH}: translate must import catalog lookup from target_model"
    );
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_atom_realizations"
        ),
        "{TRANSLATE_PATH}: translate must read atom_realizations bundle edge"
    );
    assert!(
        surface_declares_fn(&module, "translate_target_atom_realization_for_carrier"),
        "{TRANSLATE_PATH}: must declare translate_target_atom_realization_for_carrier"
    );
    assert!(
        surface_declares_fn(&module, "translate_coerced_with_atom_realization"),
        "{TRANSLATE_PATH}: type+value emit must consult the same TargetAtomRealization row"
    );
    assert!(
        TRANSLATE_DAG.contains("translate_coerced_with_atom_realization("),
        "{TRANSLATE_PATH}: translate_fold_init must consult atom realization via translate_coerced_with_atom_realization (type shell via translate_coerced_shell inside)"
    );
    assert!(
        TRANSLATE_DAG.contains("translate_atom_realization_value_from_source("),
        "{TRANSLATE_PATH}: translate_fold_init must consult atom realization value path"
    );
    assert!(
        TRANSLATE_DAG.contains("value: row.type_form.node"),
        "{TRANSLATE_PATH}: type shell must apply row.type_form.node from catalog lookup"
    );
    assert!(
        TRANSLATE_DAG.contains("target_value_expression_node(expr: expr)"),
        "{TRANSLATE_PATH}: value realization must project TargetValueExpression onto emitted Node"
    );
    assert!(
        TARGET_MODEL_DAG.contains("source_atom_value_for_realization_row("),
        "{TARGET_MODEL_PATH}: translate must import row.value_form-driven source projection"
    );
    assert!(
        TARGET_MODEL_DAG.contains("ValueBoolLiteral") && TARGET_MODEL_DAG.contains("source_atom_bool("),
        "{TARGET_MODEL_PATH}: Bool rows must project literal witnesses, not type-carrier shells only"
    );
    assert!(
        TARGET_MODEL_DAG.contains("ValueCharUnicodeScalar")
            && TARGET_MODEL_DAG.contains("source_atom_char("),
        "{TARGET_MODEL_PATH}: Char rows must project unicode scalar witnesses"
    );
    assert!(
        TRANSLATE_DAG.contains("source_atom_value_for_realization_row("),
        "{TRANSLATE_PATH}: value path must call substrate source projection (no carrier-hash re-derive)"
    );
    assert!(
        TRANSLATE_DAG.contains("target_atom_realization_lookup_miss_diagnostic(source_carrier:"),
        "{TRANSLATE_PATH}: absent atom_realizations edge must decode as catalog lookup_miss"
    );
    assert!(
        TRANSLATE_DAG.contains("translate_target_atom_realization_for_carrier("),
        "{TRANSLATE_PATH}: translate must consult catalog by source_carrier (row-driven)"
    );
    assert!(
        TRANSLATE_DAG.contains("translate_outcome_is_catalog_lookup_miss("),
        "{TRANSLATE_PATH}: non-catalog coercions must pass through only on lookup miss"
    );
    assert!(
        TARGET_MODEL_DAG.contains("TargetAtomRealizationCatalogInvalid"),
        "{TARGET_MODEL_PATH}: malformed catalog rows must fail-closed decode"
    );
    assert!(
        TARGET_MODEL_DAG
            .contains("Rejected { diagnostics: _ } => TargetAtomRealizationCatalogInvalid"),
        "{TARGET_MODEL_PATH}: catalog lookup must not swallow row decode failures"
    );
}

#[test]
fn v4_rust_language_model_declares_target_atom_realization_rows() {
    let module = parse_module(RUST_LANGUAGE_DAG, RUST_LANGUAGE_PATH);
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_symbol"),
        "{RUST_LANGUAGE_PATH}: Symbol row is greenfield (no parallel std_projection sentinel)"
    );
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_bool"),
        "{RUST_LANGUAGE_PATH}: Bool TargetAtomRealization row coexists with rust_facts_bool"
    );
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_char"),
        "{RUST_LANGUAGE_PATH}: Char TargetAtomRealization row coexists with rust_facts_char"
    );
    assert!(
        surface_declares_data(&module, "rust_target_atom_realization_catalog"),
        "{RUST_LANGUAGE_PATH}: per-language catalog prepares Python/Go parallel rows"
    );
    assert!(
        RUST_LANGUAGE_DAG.contains("data rust_std_projection_bool:"),
        "rust_std_projection_bool sentinel must remain for rust_facts_bool std_projection"
    );
    assert!(
        RUST_LANGUAGE_DAG.contains("data rust_std_projection_char:"),
        "rust_std_projection_char sentinel must remain for rust_facts_char std_projection"
    );
    let bundle_core_start = RUST_LANGUAGE_DAG
        .find("fn rust_mvp1_target_model_bundle_core()")
        .expect("rust_mvp1_target_model_bundle_core");
    let bundle_core_end = RUST_LANGUAGE_DAG[bundle_core_start..]
        .find("fn rust_mvp1_binding_spellings()")
        .expect("rust_mvp1_binding_spellings after bundle_core");
    let bundle_core_body =
        &RUST_LANGUAGE_DAG[bundle_core_start..bundle_core_start + bundle_core_end];
    assert!(
        bundle_core_body.contains("target_model_edge_collection_realization"),
        "{RUST_LANGUAGE_PATH}: row/catalog host_bundle must include collection_realization so target_model_bundle_core(host) decode matches"
    );
}

// P5 receipt: same-path smoke expansion (SG-5 collection-bounded-lattice worksheet §6).
// Asserts `.dag` surface shape only — substrate edits landed #3957 / #4085 on `main`.
// Deferral: retired when `.dag` `TestClaim` / T-22 eval covers same facts (T-PB-B).
#[test]
fn v4_std_target_model_declares_target_collection_realization_carrier() {
    let module = parse_module(TARGET_MODEL_DAG, TARGET_MODEL_PATH);
    assert!(
        surface_declares_type(&module, "TargetCollectionRealization"),
        "{TARGET_MODEL_PATH}: SG-5 canonical collection realization carrier"
    );
    assert!(
        surface_declares_type(&module, "TargetCollectionReprKind"),
        "{TARGET_MODEL_PATH}: representation kind must be a coproduct (M6)"
    );
    assert!(
        surface_declares_type(&module, "RequiredTraitWitness"),
        "{TARGET_MODEL_PATH}: per-alternative trait witnesses must be typed"
    );
    assert!(
        surface_declares_fn(&module, "target_collection_select_choice"),
        "{TARGET_MODEL_PATH}: primary/alternatives selection must be substrate authority"
    );
}

#[test]
fn v4_translate_dag_imports_collection_realization_consumer() {
    let module = parse_module(TRANSLATE_DAG, TRANSLATE_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "std", "target_model"],
            "target_model_edge_collection_realization"
        ),
        "{TRANSLATE_PATH}: translate must read collection_realization bundle edge"
    );
    assert!(
        surface_declares_fn(&module, "project_set_collection_type_node"),
        "{TRANSLATE_PATH}: Set carrier projection must consume TargetCollectionRealization rows"
    );
    assert!(
        surface_declares_fn(&module, "collection_realization_from_target"),
        "{TRANSLATE_PATH}: collection rows must decode from TargetModel bundle"
    );
}

#[test]
fn v4_bounded_lattice_completeness_and_infer_gate_are_wired() {
    let bl_module = parse_module(
        BOUNDED_LATTICE_COMPLETENESS_DAG,
        BOUNDED_LATTICE_COMPLETENESS_PATH,
    );
    assert!(
        surface_declares_fn(&bl_module, "bounded_lattice_instance_completeness"),
        "{BOUNDED_LATTICE_COMPLETENESS_PATH}: SG-6 completeness classifier must live in cycle-breaker module"
    );
    let infer_module = parse_module(INFER_DAG, INFER_PATH);
    assert!(
        import_includes_name(
            &infer_module,
            &["v4", "std", "bounded_lattice_completeness"],
            "bounded_lattice_instance_completeness"
        ),
        "{INFER_PATH}: infer must import completeness from bounded_lattice_completeness.dag"
    );
    assert!(
        surface_declares_fn(&infer_module, "infer_bounded_lattice_consumer_gate"),
        "{INFER_PATH}: consumer sites must reject partial BoundedLattice references"
    );
    assert!(
        surface_declares_fn(&infer_module, "partial_bounded_lattice_instances_in_tree"),
        "{INFER_PATH}: partial instances must be collected before consumer gate"
    );
}
