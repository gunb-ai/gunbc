//! **Layer:** integration
//!
//! T-16-A+ ratchet for `src/v4/test/fixture/task_manager_demo.dag`: the
//! TaskManager omni-emission fixture must compile with **zero** module
//! diagnostics and export the Shape-B `task_to_sql_create_table` projection
//! over sql.dag carriers (fail-closed on unmapped domain scalars).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching
//! `EXPECTED_HAND_AUTHORED_TEST` line in `sg0_census_test.rs` + INVARIANTS
//! table row land in the same PR.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const TASK_MANAGER_DEMO_DAG: &str =
    include_str!("../../../../v4/test/fixture/task_manager_demo.dag");
const TASK_MANAGER_DEMO_PATH: &str = "src/v4/test/fixture/task_manager_demo.dag";

fn task_manager_demo_dag_or_panic() -> v3_compiler::Dag {
    match compile_to_dag(TASK_MANAGER_DEMO_DAG, TASK_MANAGER_DEMO_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{TASK_MANAGER_DEMO_PATH}: expected empty diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{TASK_MANAGER_DEMO_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{TASK_MANAGER_DEMO_PATH}: {other:?}"),
    }
}

#[test]
fn v4_test_fixture_task_manager_demo_compiles() {
    let _dag = task_manager_demo_dag_or_panic();
}

#[test]
fn v4_test_fixture_task_manager_demo_exports_sql_create_table_projection() {
    let _dag = task_manager_demo_dag_or_panic();
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
