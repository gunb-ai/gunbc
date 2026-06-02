//! **Layer:** integration
//!
//! T-33 receipt: `src/v4/std/model_core.dag` tokenizes and parses cleanly — Ratified Q1
//! `ModelCore` carrier (primitive fact bundles, algebra inhabitance, laws, effect/partiality
//! scaffolds). Full `compile_to_dag` on this module alone does not resolve `import v4.std.*`
//! peers under today's M1(2.7) single-file path (same posture as `v4_lens_testgen_dag_smoke_test`).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceType};
use v3_compiler::tokenize_for_test;

const MODEL_CORE_DAG: &str = include_str!("../../../../v4/std/model_core.dag");
const MODEL_CORE_PATH: &str = "src/v4/std/model_core.dag";
const ALGEBRA_DAG: &str = include_str!("../../../../v4/std/algebra.dag");
const ALGEBRA_PATH: &str = "src/v4/std/algebra.dag";

fn model_core_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(MODEL_CORE_DAG, MODEL_CORE_PATH)
        .unwrap_or_else(|e| panic!("{MODEL_CORE_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, MODEL_CORE_PATH)
        .unwrap_or_else(|e| panic!("{MODEL_CORE_PATH}: parse: {e:?}"))
}

fn algebra_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(ALGEBRA_DAG, ALGEBRA_PATH)
        .unwrap_or_else(|e| panic!("{ALGEBRA_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, ALGEBRA_PATH).unwrap_or_else(|e| panic!("{ALGEBRA_PATH}: parse: {e:?}"))
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

fn function_count(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> usize {
    module
        .items
        .iter()
        .filter(|item| match item {
            SurfaceItem::Fn {
                name: item_name, ..
            }
            | SurfaceItem::FnExternalBody {
                name: item_name, ..
            } => item_name == name,
            _ => false,
        })
        .count()
}

fn surface_declares_data(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
    ty_name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Data {
            name: decl_name,
            ty,
            ..
        } => decl_name == name && surface_type_name(ty) == ty_name,
        _ => false,
    })
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

fn type_record_fields<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a [SurfaceField] {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(fields.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type record {name}"))
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
        SurfaceType::Optional { inner, .. } => format!("?{}", surface_type_name(inner)),
        SurfaceType::Arrow { .. } => "fn".to_string(),
    }
}

#[test]
fn v4_std_model_core_dag_tokenizes_and_parses() {
    let module = model_core_surface_or_panic();
    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "std", "model_core"]],
        "T-33 authority module should remain v4.std.model_core"
    );
}

#[test]
fn v4_std_model_core_declares_ratified_q1_carriers() {
    let module = model_core_surface_or_panic();
    for name in [
        "ModelCore",
        "ModelCorePrimitiveFactAxis",
        "PrimitiveFactBundle",
        "AlgebraInhabitanceDecl",
        "AlgebraLawObligation",
        "EffectSignature",
        "ResourceAccess",
        "EffectSemanticsDecl",
        "PartialitySemanticsDecl",
    ] {
        assert!(
            surface_declares_type(&module, name),
            "T-33 must declare `{name}`"
        );
    }
}

#[test]
fn v4_std_model_core_bundles_five_facets() {
    let module = model_core_surface_or_panic();
    let fields: Vec<String> = type_record_fields(&module, "ModelCore")
        .iter()
        .map(|f| f.name.clone())
        .collect();
    assert_eq!(
        fields,
        vec![
            "primitives".to_string(),
            "inhabitance".to_string(),
            "laws".to_string(),
            "effects".to_string(),
            "partiality".to_string(),
        ],
        "ModelCore must bundle primitive/inhabitance/law/effect/partiality facets"
    );
    assert_eq!(
        surface_type_name(&type_record_fields(&module, "ModelCore")[0].ty),
        "List<PrimitiveFactBundle>"
    );
}

#[test]
fn v4_std_model_core_algebra_law_obligation_structural() {
    let module = model_core_surface_or_panic();
    let fields: Vec<(String, String)> = type_record_fields(&module, "AlgebraLawObligation")
        .iter()
        .map(|f| (f.name.clone(), surface_type_name(&f.ty)))
        .collect();
    assert_eq!(
        fields,
        vec![
            ("inhabitance_witness".to_string(), "Node".to_string()),
            ("law".to_string(), "Node".to_string()),
        ],
        "law obligations must reference canonical inhabitance via witness Node, not embed a parallel AlgebraInhabitanceDecl copy"
    );
}

#[test]
fn v4_std_model_core_effect_partiality_ops_use_node_authority() {
    let module = model_core_surface_or_panic();
    for (type_name, field_name, expected_type) in [
        ("PrimitiveOperationRef", "operation", "Node"),
        (
            "PrimitiveOperationRef",
            "effect_signature",
            "EffectSignature",
        ),
        ("PartialOperationDecl", "operation", "Node"),
    ] {
        let fields: Vec<(String, String)> = type_record_fields(&module, type_name)
            .iter()
            .map(|f| (f.name.clone(), surface_type_name(&f.ty)))
            .collect();
        let ty = fields
            .iter()
            .find(|(n, _)| n == field_name)
            .map(|(_, t)| t.as_str())
            .unwrap_or("missing");
        assert_eq!(
            ty, expected_type,
            "{type_name}.{field_name} must use its ModelCore substrate authority"
        );
    }
}

#[test]
fn v4_std_model_core_effect_signature_resource_access_shape() {
    let module = model_core_surface_or_panic();
    let signature_fields: Vec<(String, String)> = type_record_fields(&module, "EffectSignature")
        .iter()
        .map(|f| (f.name.clone(), surface_type_name(&f.ty)))
        .collect();
    assert_eq!(
        signature_fields,
        vec![("signature".to_string(), "Node".to_string())],
        "EffectSignature must own effect identity only; BoundaryAllowance.resource is the single resource authority"
    );
    let resource_fields: Vec<(String, String)> = type_record_fields(&module, "ResourceAccess")
        .iter()
        .map(|f| (f.name.clone(), surface_type_name(&f.ty)))
        .collect();
    assert_eq!(
        resource_fields,
        vec![("resource".to_string(), "Node".to_string())],
        "ResourceAccess must be the ModelCore-owned minimal resource carrier"
    );
}

#[test]
fn v4_std_model_core_primitive_fact_bundle_uses_axis_keyed_spec_facts() {
    let module = model_core_surface_or_panic();
    let bundle_fields: Vec<(String, String)> = type_record_fields(&module, "PrimitiveFactBundle")
        .iter()
        .map(|f| (f.name.clone(), surface_type_name(&f.ty)))
        .collect();
    assert_eq!(
        bundle_fields,
        vec![
            ("substrate_carrier".to_string(), "Node".to_string()),
            (
                "spec_facts".to_string(),
                "Map<Symbol, Node>".to_string(),
            ),
        ],
        "PrimitiveFactBundle.spec_facts must be axis-keyed Map<Symbol, Node> (duplicate axes structurally unrepresentable), not an opaque Node"
    );
    for axis in [
        "primitive_fact_axis_width",
        "primitive_fact_axis_signedness",
        "primitive_fact_axis_range",
        "primitive_fact_axis_encoding",
        "primitive_fact_axis_surface_spelling",
        "primitive_fact_axis_overflow_disposition",
    ] {
        assert!(
            surface_declares_data(&module, axis, "Symbol"),
            "declared axis Symbol `{axis}` is the canonical key for PrimitiveFactBundle.spec_facts"
        );
    }
    assert!(
        MODEL_CORE_DAG.contains("type ModelCorePrimitiveFactAxis")
            && MODEL_CORE_DAG.contains("= ModelCoreFactAxisWidth {}")
            && MODEL_CORE_DAG.contains("| ModelCoreFactAxisEncoding {}")
            && MODEL_CORE_DAG.contains("| ModelCoreFactAxisSurfaceSpelling {}")
            && MODEL_CORE_DAG.contains("fn model_core_primitive_fact_axis_symbol(axis: ModelCorePrimitiveFactAxis) -> Symbol"),
        "ModelCorePrimitiveFactAxis must be the typed canonical carrier for new grounding registries, with a single Symbol projection at the legacy spec_facts boundary"
    );
}

#[test]
fn v4_std_model_core_wave1_void_constructor_present() {
    let module = model_core_surface_or_panic();
    assert_eq!(
        function_count(&module, "wave1_void"),
        1,
        "wave1_void is a tracked 🟡 scaffold (feature:model-core-wave1-void-scaffold); not a silent default ModelCore"
    );
}

#[test]
fn v4_std_algebra_fold_list_node_pins_canonical_spine() {
    let module = algebra_surface_or_panic();
    assert_eq!(
        function_count(&module, "fold_list_node"),
        1,
        "fold_list_node is the single exported FreeMonoid-to-Node spine encoder"
    );
    assert!(
        surface_declares_data(&module, "fold_list_node_head", "Symbol"),
        "fold_list_node must own its canonical head edge label"
    );
    assert!(
        surface_declares_data(&module, "fold_list_node_tail", "Symbol"),
        "fold_list_node must own its canonical tail edge label"
    );
    assert!(
        ALGEBRA_DAG.contains("empty: Node { kind: TypeNode { connective: Conj }, children: [] }"),
        "empty-list encoding is fixed by the helper, not supplied by callers"
    );
    assert!(
        ALGEBRA_DAG.contains(
            "Edge { label: Named { name: fold_list_node_head }, target: item_node(item) }"
        ) && ALGEBRA_DAG
            .contains("Edge { label: Named { name: fold_list_node_tail }, target: tail_node }"),
        "cons encoding must use the canonical head/tail edge labels"
    );
}
