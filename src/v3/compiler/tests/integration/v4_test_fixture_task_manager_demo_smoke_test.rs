//! **Layer:** integration
//!
//! T-16-A+ ratchet for `src/v4/test/fixture/task_manager_demo.dag`: the
//! TaskManager omni-emission fixture must tokenize+parse cleanly and export
//! the Shape-B `task_to_sql_create_table` projection over sql.dag carriers
//! (fail-closed on unmapped domain scalars).
//!
//! Uses `parse_for_test` (not `compile_to_dag`): v4 cross-module fixtures
//! with record literals / block bodies remain M1(2.8) opaque to full lowering;
//! parse is the same gate used by coordination/openapi extdeps smoke tests.
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching
//! `EXPECTED_HAND_AUTHORED_TEST` line in `sg0_census_test.rs` + INVARIANTS
//! table row land in the same PR.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceType, TypeAngleArg};
use v3_compiler::tokenize_for_test;

const TASK_MANAGER_DEMO_DAG: &str =
    include_str!("../../../../v4/test/fixture/task_manager_demo.dag");
const TASK_MANAGER_DEMO_PATH: &str = "src/v4/test/fixture/task_manager_demo.dag";
const SQL_DAG: &str = include_str!("../../../../v4/extdeps/formats/sql.dag");
const SQL_DAG_PATH: &str = "src/v4/extdeps/formats/sql.dag";

fn task_manager_demo_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(TASK_MANAGER_DEMO_DAG, TASK_MANAGER_DEMO_PATH)
        .unwrap_or_else(|e| panic!("{TASK_MANAGER_DEMO_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, TASK_MANAGER_DEMO_PATH)
        .unwrap_or_else(|e| panic!("{TASK_MANAGER_DEMO_PATH}: parse: {e:?}"))
}

fn sql_dag_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(SQL_DAG, SQL_DAG_PATH)
        .unwrap_or_else(|e| panic!("{SQL_DAG_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, SQL_DAG_PATH).unwrap_or_else(|e| panic!("{SQL_DAG_PATH}: parse: {e:?}"))
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

fn type_sum_variants(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> Vec<String> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum {
                name: item_name,
                variants,
                ..
            } if item_name == name => Some(variants.iter().map(|v| v.name.clone()).collect()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type sum {name}"))
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered = args
                .iter()
                .map(|arg| match arg {
                    TypeAngleArg::TypeExpr { ty } => surface_type_name(ty),
                    TypeAngleArg::WidthNatLiteral { decimal, .. } => decimal.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered}>")
        }
        SurfaceType::Optional { inner, .. } => format!("?{}", surface_type_name(inner)),
        SurfaceType::Arrow { .. } => "fn".to_string(),
    }
}

fn record_field_type(fields: &[SurfaceField], name: &str) -> String {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| surface_type_name(&field.ty))
        .unwrap_or_else(|| panic!("missing field {name}"))
}

#[test]
fn v4_test_fixture_task_manager_demo_parses() {
    let _module = task_manager_demo_surface_or_panic();
}

#[test]
fn v4_test_fixture_task_manager_demo_task_field_authority_matches_task_record() {
    let module = task_manager_demo_surface_or_panic();
    let task_fields = type_record_fields(&module, "Task");
    assert_eq!(
        task_fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        ["id", "title", "status"]
    );
    assert_eq!(
        type_sum_variants(&module, "TaskManagerTaskField"),
        ["TaskFieldId", "TaskFieldTitle", "TaskFieldStatus"]
    );
    assert!(
        TASK_MANAGER_DEMO_DAG.contains("data task_manager_task_field_catalog: List<TaskManagerTaskField> = [")
            && TASK_MANAGER_DEMO_DAG.contains("TaskFieldId,")
            && TASK_MANAGER_DEMO_DAG.contains("TaskFieldTitle,")
            && TASK_MANAGER_DEMO_DAG.contains("TaskFieldStatus")
            && TASK_MANAGER_DEMO_DAG
                .contains("fn task_manager_task_fields_to_sql_columns(")
            && TASK_MANAGER_DEMO_DAG.contains(
                "o: task_manager_task_fields_to_sql_columns(fields: task_manager_task_field_catalog)"
            ),
        "SQL columns must fold over task_manager_task_field_catalog, not hand-list Task fields"
    );
}

#[test]
fn v4_test_fixture_task_manager_demo_sql_projection_uses_canonical_carriers() {
    let module = task_manager_demo_surface_or_panic();
    assert_eq!(
        type_sum_variants(&module, "TaskManagerSqlProjectableScalar"),
        ["ProjectTaskId", "ProjectString", "ProjectTaskStatus"]
    );
    assert_eq!(
        type_sum_variants(&module, "TaskManagerSqlScalarDispatch"),
        ["MappedProjectable", "UnmappedDomainScalar"]
    );

    let sql_module = sql_dag_surface_or_panic();
    let sql_table = type_record_fields(&sql_module, "SqlTableDefinition");
    assert_eq!(
        record_field_type(sql_table, "primary_key"),
        "?List<SqlIdentifier>"
    );
    assert!(
        TASK_MANAGER_DEMO_DAG.contains("primary_key: optional_present(value: [")
            && TASK_MANAGER_DEMO_DAG.contains("SqlTableDefinition {"),
        "fixture must populate SqlTableDefinition.primary_key via optional_present"
    );

    assert!(
        TASK_MANAGER_DEMO_DAG.contains("UnmappedDomainScalar { anchor } =>")
            && TASK_MANAGER_DEMO_DAG
                .contains("task_manager_reject_unmapped_domain_scalar(node: anchor)"),
        "UnmappedDomainScalar dispatch arm must route to fail-closed rejection"
    );
    assert!(
        TASK_MANAGER_DEMO_DAG
            .contains("fn task_to_sql_create_table() -> Outcome<SqlSchemaOperation>"),
        "fixture must export Task→SqlCreateTable Shape-B projection"
    );
}
