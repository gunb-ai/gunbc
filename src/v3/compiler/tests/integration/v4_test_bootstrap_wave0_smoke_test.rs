//! **Layer:** integration
//!
//! Wave-0 ratchet for the early v4 test/bootstrap infrastructure lane.
//! This is a parse-surface receipt only: T-19/T-20 remain `.dag` authorities.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceModule};
use v3_compiler::tokenize_for_test;

const TESTGEN_DAG: &str = include_str!("../../../../v4/lens/testgen.dag");
const TESTGEN_PATH: &str = "src/v4/lens/testgen.dag";
const BOOTSTRAP_DAG: &str = include_str!("../../../../v4/workflow/bootstrap.dag");
const BOOTSTRAP_PATH: &str = "src/v4/workflow/bootstrap.dag";
const CI_DAG: &str = include_str!("../../../../v4/workflow/ci.dag");
const CI_PATH: &str = "src/v4/workflow/ci.dag";

#[test]
fn v4_wave0_testgen_declares_five_scheduling_arms() {
    let module = parse_module(TESTGEN_DAG, TESTGEN_PATH);

    assert_eq!(module_paths(&module), vec![vec!["v4", "lens", "testgen"]]);
    assert_eq!(
        type_sum_variants(&module, "TestgenConcept"),
        vec![
            "TypeConstruction",
            "AlgebraLaw",
            "DiagnosticExhaustiveness",
            "LensApplicability",
            "BidirectionalRoundtrip",
        ],
        "T-19 must keep exactly the five scheduling categories from TASKS.md"
    );
    assert_eq!(
        record_fields(&module, "Generator"),
        vec!["classification", "slot"],
        "Generator<C> stamps TestClassification beside the generated slot"
    );
}

#[test]
fn v4_wave0_bootstrap_declares_seed_once_chain() {
    let module = parse_module(BOOTSTRAP_DAG, BOOTSTRAP_PATH);

    assert_eq!(
        module_paths(&module),
        vec![vec!["v4", "workflow", "bootstrap"]]
    );
    assert_eq!(
        record_fields(&module, "BootstrapPlan"),
        vec!["seed", "self0", "self1", "fixpt"],
        "T-20 bootstrap must keep the seed-once/self-host/fixed-point stages structural"
    );
    assert_eq!(function_count(&module, "bootstrap_stage_output"), 1);
    assert_eq!(function_count(&module, "bootstrap_plan_well_formed"), 1);
    assert_eq!(data_count(&module, "bootstrap_plan"), 1);
}

#[test]
fn v4_wave0_ci_imports_bootstrap_authority() {
    let module = parse_module(CI_DAG, CI_PATH);

    let imported = import_names(&module, &["v4", "workflow", "bootstrap"])
        .expect("ci.dag must import the bootstrap authority module");
    assert!(
        imported.contains(&"v4_stage0_binary"),
        "CI seed job must reference the canonical stage0 symbol"
    );
    assert!(
        imported.contains(&"bootstrap_plan"),
        "CI must consume the validated bootstrap_plan value"
    );
    assert!(
        imported.contains(&"bootstrap_stage_output"),
        "CI must validate BootstrapStageCompile through bootstrap_stage_output"
    );
}

fn parse_module(source: &str, file: &str) -> SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}

fn module_paths(module: &SurfaceModule) -> Vec<Vec<&str>> {
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

fn type_sum_variants<'a>(module: &'a SurfaceModule, name: &str) -> Vec<&'a str> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum {
                name: item_name,
                variants,
                ..
            } if item_name == name => Some(
                variants
                    .iter()
                    .map(|variant| variant.name.as_str())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type sum {name}"))
}

fn record_fields<'a>(module: &'a SurfaceModule, name: &str) -> Vec<&'a str> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(field_names(fields)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type record {name}"))
}

fn field_names(fields: &[SurfaceField]) -> Vec<&str> {
    fields.iter().map(|field| field.name.as_str()).collect()
}

fn function_count(module: &SurfaceModule, name: &str) -> usize {
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

fn data_count(module: &SurfaceModule, name: &str) -> usize {
    module
        .items
        .iter()
        .filter(|item| match item {
            SurfaceItem::Data {
                name: item_name, ..
            } => item_name == name,
            _ => false,
        })
        .count()
}

fn import_names<'a>(module: &'a SurfaceModule, path: &[&str]) -> Option<Vec<&'a str>> {
    module.items.iter().find_map(|item| match item {
        SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } if item_path
            .iter()
            .map(String::as_str)
            .eq(path.iter().copied()) =>
        {
            Some(names.iter().map(String::as_str).collect())
        }
        _ => None,
    })
}
