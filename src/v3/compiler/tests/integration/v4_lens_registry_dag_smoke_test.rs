//! **Layer:** integration
//!
//! **P2 / Practice 5 (single authority):** This harness proves `src/v4/lens/registry.dag`
//! parses and exposes the registry-backed query surface that `src/v4/workflow/ci.dag`
//! consumes for Lens-CI activation. This local harness stays at parsed-shape level because
//! M1(2.8) still rejects v4 block-bodied functions in isolated `compile_to_dag` smokes; the
//! live CI step pairs it with `v2-compiler compile --source-root src/v4 --target rust` so
//! lowering/inference of the actual interface is checked without using the known-hanging
//! `--target dag` path.
//!
//! **INVARIANTS §P5 Dispatch-Discipline Mechanism (b):** this path’s SG-0 census line + matching
//! `INVARIANTS.md` table row land in the same PR as the harness (home-of-record for the
//! hand-Rust receipt).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceType, SurfaceVariant, VariantPayload,
};
use v3_compiler::tokenize_for_test;

const REGISTRY_DAG: &str = include_str!("../../../../v4/lens/registry.dag");
const REGISTRY_PATH: &str = "src/v4/lens/registry.dag";
const CI_DAG: &str = include_str!("../../../../v4/workflow/ci.dag");
const CI_PATH: &str = "src/v4/workflow/ci.dag";

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

fn surface_declares_data(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Data {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } => {
            item_path
                .iter()
                .map(String::as_str)
                .eq(path.iter().copied())
                && names.iter().any(|n| n == name)
        }
        _ => false,
    })
}

fn type_sum_variant<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
    variant_name: &str,
) -> &'a SurfaceVariant {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum { name, variants, .. } if name == type_name => {
                variants.iter().find(|variant| variant.name == variant_name)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing `{type_name}.{variant_name}` variant"))
}

fn record_payload_field<'a>(variant: &'a SurfaceVariant, field_name: &str) -> &'a SurfaceField {
    let VariantPayload::Record(fields) = &variant.payload else {
        panic!("variant `{}` must carry a record payload", variant.name);
    };
    fields
        .iter()
        .find(|field| field.name == field_name)
        .unwrap_or_else(|| panic!("variant `{}` missing `{field_name}` field", variant.name))
}

fn data_body<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a SurfaceExpr {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::Data {
                name: item_name,
                body: Some(body),
                ..
            } if item_name == name => Some(body),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing data body `{name}`"))
}

fn type_is_list_of(ty: &SurfaceType, element_name: &str) -> bool {
    match ty {
        SurfaceType::Parameterized { name, args, .. } if name == "List" && args.len() == 1 => {
            matches!(
                &args[0],
                v3_compiler::parse_surface::TypeAngleArg::TypeExpr { ty }
                    if matches!(ty.as_ref(), SurfaceType::Named { name, .. } if name == element_name)
            )
        }
        _ => false,
    }
}

fn list_body_vars(body: &SurfaceExpr) -> Vec<&str> {
    let SurfaceExpr::List { elements, .. } = body else {
        panic!("expected list body, got {body:?}");
    };
    elements
        .iter()
        .map(|element| match element {
            SurfaceExpr::Var { name, .. } => name.as_str(),
            other => panic!("expected list element var, got {other:?}"),
        })
        .collect()
}

#[test]
fn v4_lens_registry_dag_compiles() {
    let module = parse_module(REGISTRY_DAG, REGISTRY_PATH);
    assert!(
        surface_declares_data(&module, "lens_registry_v0"),
        "{REGISTRY_PATH}: registry list must be the LensIdV0 row authority"
    );
    assert!(
        surface_declares_fn(&module, "lens_registry_ids_resolve"),
        "{REGISTRY_PATH}: registry must expose the required-lens resolution query"
    );
    assert!(
        surface_declares_fn(&module, "lens_registry_bound_row_count"),
        "{REGISTRY_PATH}: registry must count bound rows for fail-closed singleton resolution"
    );
    assert!(
        surface_declares_fn(&module, "lens_registry_row_count"),
        "{REGISTRY_PATH}: registry must count all rows so duplicate bound/unbound ids reject"
    );
}

#[test]
fn v4_ci_workflow_consumes_lens_registry_for_lens_ci_signal() {
    let module = parse_module(CI_DAG, CI_PATH);
    assert!(
        import_includes_name(
            &module,
            &["v4", "lens", "registry"],
            "lens_registry_ids_resolve"
        ),
        "{CI_PATH}: Lens-CI command must consume the registry query, not a parallel list"
    );
    let lens_ci = type_sum_variant(&module, "CiCommand", "LensCiCommand");
    let required_lenses = record_payload_field(lens_ci, "required_lenses");
    assert!(
        type_is_list_of(&required_lenses.ty, "LensIdV0"),
        "{CI_PATH}: LensCiCommand.required_lenses must be List<LensIdV0>"
    );
    let required = list_body_vars(data_body(&module, "lens_ci_required_lenses"));
    assert_eq!(
        required,
        vec![
            "Complexity",
            "Cost",
            "Parallelism",
            "EffectEnumeration",
            "Idempotency",
            "UnusedParameters",
        ],
        "{CI_PATH}: Lens-CI required lenses must be registry ids"
    );
    assert!(
        surface_declares_data(&module, "lens_ci_registry_signal"),
        "{CI_PATH}: Lens-CI must expose a CI gate signal"
    );
}
