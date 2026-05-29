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
use v3_compiler::tokenize_for_test;

const TASK_MANAGER_DEMO_DAG: &str =
    include_str!("../../../../v4/test/fixture/task_manager_demo.dag");
const TASK_MANAGER_DEMO_PATH: &str = "src/v4/test/fixture/task_manager_demo.dag";

fn task_manager_demo_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(TASK_MANAGER_DEMO_DAG, TASK_MANAGER_DEMO_PATH)
        .unwrap_or_else(|e| panic!("{TASK_MANAGER_DEMO_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, TASK_MANAGER_DEMO_PATH)
        .unwrap_or_else(|e| panic!("{TASK_MANAGER_DEMO_PATH}: parse: {e:?}"))
}

#[test]
fn v4_test_fixture_task_manager_demo_parses() {
    let _module = task_manager_demo_surface_or_panic();
}

#[test]
fn v4_test_fixture_task_manager_demo_exports_sql_create_table_projection() {
    let _module = task_manager_demo_surface_or_panic();
    assert!(
        TASK_MANAGER_DEMO_DAG.contains("fn task_to_sql_create_table() -> Outcome<SqlSchemaOperation>"),
        "fixture must export Task→SqlCreateTable Shape-B projection"
    );
    assert!(
        TASK_MANAGER_DEMO_DAG.contains("fn task_manager_reject_unmapped_domain_scalar("),
        "fixture must fail-closed on unmapped domain scalars"
    );
    assert!(
        TASK_MANAGER_DEMO_DAG.contains("type TaskManagerSqlProjectableScalar"),
        "fixture must enumerate projectable Task field scalars explicitly"
    );
}
