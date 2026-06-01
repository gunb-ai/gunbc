//! **Layer:** integration
//!
//! **P2 / Practice 5 (single authority):** This harness proves `src/v4/lens/registry.dag`
//! parses and exposes the registry-backed query surface that `src/v4/workflow/ci.dag`
//! consumes for Lens-CI activation, while also preserving the staged T-23/P9 registry
//! parse-cleanliness checks. This local harness stays at parsed-shape level because
//! M1(2.8) still rejects v4 block-bodied functions in isolated `compile_to_dag` smokes; the
//! live CI step pairs it with
//! `v2-compiler compile` over a temporary `ci.dag` entry root plus `src/v4` dependency
//! root so lowering/inference of the workflow consumer interface is checked without using the known-hanging
//! `--target dag` path.
//!
//! **Note:** After P9 `#3503`, `registry.dag` imports `Symbol` and `List` from `v4.std.*` and
//! carries self-referential `Symbol` data rows. Isolated `compile_to_dag` does not resolve those
//! imports under M1(2.7) (same posture as `v4_bin_main_dag_smoke_test` / `v4_lens_testgen_dag_smoke_test`).
//!
//! **INVARIANTS §P5 Dispatch-Discipline Mechanism (b):** this path’s SG-0 census line + matching
//! `INVARIANTS.md` table row land in the same PR as the harness (home-of-record for the
//! hand-Rust receipt).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceExpr, SurfaceField, SurfaceItem, SurfaceRecordField, SurfaceType, SurfaceVariant,
    VariantPayload,
};
use v3_compiler::tokenize_for_test;

const REGISTRY_DAG: &str = include_str!("../../../../v4/lens/registry.dag");
const REGISTRY_PATH: &str = "src/v4/lens/registry.dag";
const STRUCTURAL_SIMILARITY_DAG: &str =
    include_str!("../../../../v4/lens/structural_similarity.dag");
const STRUCTURAL_SIMILARITY_PATH: &str = "src/v4/lens/structural_similarity.dag";
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

fn type_record_field_names<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
) -> Vec<&'a str> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord { name, fields, .. } if name == type_name => {
                Some(fields.iter().map(|field| field.name.as_str()).collect())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type record `{type_name}`"))
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

fn record_body_field<'a>(body: &'a SurfaceExpr, field_name: &str) -> &'a SurfaceExpr {
    let SurfaceExpr::Record { fields, .. } = body else {
        panic!("expected record body, got {body:?}");
    };
    fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|SurfaceRecordField { value, .. }| value)
        .unwrap_or_else(|| panic!("record body missing `{field_name}` field"))
}

fn variant_record_field<'a>(
    expr: &'a SurfaceExpr,
    target_name: &str,
    field_name: &str,
) -> &'a SurfaceExpr {
    let SurfaceExpr::VariantRecord { target, fields, .. } = expr else {
        panic!("expected `{target_name}` record, got {expr:?}");
    };
    assert_eq!(
        target, target_name,
        "expected `{target_name}` record target, got `{target}`"
    );
    fields
        .iter()
        .find(|field| field.name == field_name)
        .map(|SurfaceRecordField { value, .. }| value)
        .unwrap_or_else(|| panic!("`{target_name}` missing `{field_name}` field"))
}

fn ci_pipeline_jobs(body: &SurfaceExpr) -> &[SurfaceExpr] {
    let call_record = match body {
        SurfaceExpr::Call { target, args, .. } if target == "ci_pipeline_well_formed" => args
            .first()
            .unwrap_or_else(|| panic!("ci_pipeline_well_formed missing pipeline arg")),
        other => panic!("expected ci_pipeline_well_formed call, got {other:?}"),
    };
    let pipeline_expr = record_body_field(call_record, "p");
    let jobs_expr = variant_record_field(pipeline_expr, "CiPipeline", "jobs");
    let SurfaceExpr::List { elements, .. } = jobs_expr else {
        panic!("expected ci_pipeline.jobs list, got {jobs_expr:?}");
    };
    elements
}

fn ci_job_record_by_id<'a>(jobs: &'a [SurfaceExpr], job_id: &str) -> &'a SurfaceExpr {
    jobs.iter()
        .find(|job| {
            let SurfaceExpr::VariantRecord { target, fields, .. } = job else {
                return false;
            };
            if target != "CiJob" {
                return false;
            }
            fields.iter().any(|field| {
                field.name == "id"
                    && matches!(&field.value, SurfaceExpr::Var { name, .. } if name == job_id)
            })
        })
        .unwrap_or_else(|| panic!("missing CiJob `{job_id}`"))
}

#[test]
fn v4_lens_registry_dag_tokenizes_and_parses() {
    let module = parse_module(REGISTRY_DAG, REGISTRY_PATH);
    assert_eq!(
        module_path(&module),
        vec!["v4", "lens", "registry"],
        "{REGISTRY_PATH}: module path"
    );
    assert!(
        module_declares_type_sum_named(&module, "LensIdV0"),
        "{REGISTRY_PATH}: must declare LensIdV0 closed sum"
    );
    assert!(
        module_declares_type_sum_named(&module, "LensModulePathV0"),
        "{REGISTRY_PATH}: must declare LensModulePathV0"
    );
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
    assert_eq!(
        type_record_field_names(&module, "LensCiLiveWorkflowSignal"),
        vec!["smoke_step_name", "semantic_step_name", "semantic_target"],
        "{CI_PATH}: live workflow binding must not re-author ci_pipeline signal/job/policy facts"
    );
    let ci_pipeline = data_body(&module, "ci_pipeline");
    let lens_ci_job =
        ci_job_record_by_id(ci_pipeline_jobs(ci_pipeline), "lens_ci_registry_execution");
    assert_eq!(
        list_body_vars(variant_record_field(lens_ci_job, "CiJob", "needs")),
        vec!["v2_compile_src_v4"],
        "{CI_PATH}: Lens-CI execution must depend on the v2 compiler artifact job used by the live semantic step"
    );
}

#[test]
fn v4_lens_registry_t23_closed_lens_ids_present() {
    for id in [
        "Complexity",
        "Cost",
        "Parallelism",
        "EffectEnumeration",
        "Idempotency",
        "Provenance",
        "UnusedParameters",
        "StructuralResolution",
        "TableDecisionTree",
    ] {
        assert!(
            REGISTRY_DAG.contains(id),
            "{REGISTRY_PATH}: LensIdV0 arm `{id}` must appear in closed registry source"
        );
    }
}

#[test]
fn v4_lens_structural_similarity_dag_parses() {
    let module = parse_module(STRUCTURAL_SIMILARITY_DAG, STRUCTURAL_SIMILARITY_PATH);
    assert_eq!(
        module_path(&module),
        vec!["v4", "lens", "structural_similarity"],
        "{STRUCTURAL_SIMILARITY_PATH}: module path must match v4.lens.structural_similarity"
    );
    assert!(
        surface_declares_fn(&module, "structural_similarity_fact"),
        "{STRUCTURAL_SIMILARITY_PATH}: producer fn must be declared"
    );
    assert!(
        surface_declares_data(&module, "structural_similarity_empty_type_shape"),
        "{STRUCTURAL_SIMILARITY_PATH}: TypeShape.variant_set witness row must be declared"
    );
}

#[test]
fn v4_lens_registry_table_decision_tree_bound_to_module() {
    assert!(
        REGISTRY_DAG.contains("lens_id: TableDecisionTree")
            && REGISTRY_DAG.contains(r#"module_path: Bound { path: "v4.lens.table_decision_tree" }"#),
        "{REGISTRY_PATH}: TableDecisionTree must be Bound to v4.lens.table_decision_tree (L1.13.c substrate promotion)"
    );
}

#[test]
fn v4_lens_registry_structural_resolution_bound_to_module() {
    assert!(
        REGISTRY_DAG.contains("lens_id: StructuralResolution")
            && REGISTRY_DAG.contains(r#"module_path: Bound { path: "v4.lens.structural_resolution" }"#),
        "{REGISTRY_PATH}: StructuralResolution must be Bound to v4.lens.structural_resolution (T-13 registry fill)"
    );
}

#[test]
fn v4_lens_registry_p9_owned_fn_surface_present() {
    assert!(
        REGISTRY_DAG.contains("type LensOwnedFnV0")
            && REGISTRY_DAG.contains("data lens_owned_fn_registry_v0")
            && REGISTRY_DAG.contains("data symbol_llvm_instruction_cost")
            && REGISTRY_DAG.contains("owner_module_path: \"v4.lens.cost\""),
        "{REGISTRY_PATH}: P9 single-owner registry rows must remain in substrate home-of-record"
    );
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

fn module_declares_type_sum_named(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum {
                name: item_name, ..
            } if item_name == name
        )
    })
}
