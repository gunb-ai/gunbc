//! **Layer:** integration
//!
//! T-16-C ratchet for `src/v4/test/fixture/task_manager_demo.dag`: TaskManager demo
//! wires `OpenApiOperationWireContractFacts` (method + path_template) through
//! `openapi_operation_wire_contract` into coordination `WireContract`, and exposes
//! canonical `RestRoute` receipts for Shape-B drift-lock.

use v3_compiler::parse_for_test;
use v3_compiler::tokenize_for_test;

const TASK_MANAGER_DAG: &str = include_str!("../../../../v4/test/fixture/task_manager_demo.dag");
const TASK_MANAGER_PATH: &str = "src/v4/test/fixture/task_manager_demo.dag";

#[test]
fn v4_test_fixture_task_manager_demo_dag_parses() {
    let tokens = tokenize_for_test(TASK_MANAGER_DAG, TASK_MANAGER_PATH)
        .unwrap_or_else(|e| panic!("{TASK_MANAGER_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, TASK_MANAGER_PATH)
        .unwrap_or_else(|e| panic!("{TASK_MANAGER_PATH}: parse: {e:?}"));
}

#[test]
fn v4_test_fixture_task_manager_demo_shape_b_route_receipt() {
    assert!(
        TASK_MANAGER_DAG.contains("OpenApiOperationWireContractFacts")
            && TASK_MANAGER_DAG.contains("openapi_operation_wire_contract(")
            && TASK_MANAGER_DAG.contains("rest_route_from_openapi_operation_wire_contract_facts(")
            && TASK_MANAGER_DAG.contains("task_manager_shape_b_routes")
            && TASK_MANAGER_DAG.contains("admitted_method: OpenApiPost")
            && TASK_MANAGER_DAG.contains("admitted_method: OpenApiPatch")
            && TASK_MANAGER_DAG.contains("admitted_method: OpenApiGet")
            && TASK_MANAGER_DAG.contains("after_slash: \"/tasks\"")
            && TASK_MANAGER_DAG.contains("after_slash: \"/tasks/{id}\"")
            && TASK_MANAGER_DAG.contains("path_parameters: [\"id\"]"),
        "TaskManager fixture must declare OpenAPI operation facts, derived WireContracts, and RestRoute drift-lock receipts"
    );
    assert!(
        !TASK_MANAGER_DAG.contains("WireContractFacts {")
            && !TASK_MANAGER_DAG.contains("exchange: RequestReply"),
        "TaskManager must not restate coordination WireContractFacts — single authority via openapi morphisms"
    );
}
