//! Minimal heuristics for test classification + cost.
//!
//! Keep this intentionally simple so it's easy to replace later.

use gunbc_codegen::testgen::analyze::DagAnalysis;
use gunbc_ir::transport::TransportResponse;
use gunbc_ir::Value;
use gunbc_test::{FermiCost, MockSpec, TestClass};

pub fn infer_test_class(analysis: &DagAnalysis) -> TestClass {
    if analysis.transport_executors.is_empty() && analysis.tool_env_nodes.is_empty() {
        TestClass::Unit
    } else {
        TestClass::Hermetic
    }
}

pub fn infer_fermi_cost(class: TestClass) -> FermiCost {
    let _ = class;
    // Default to XS unless explicitly overridden by the test target.
    FermiCost::XS
}

pub fn infer_requires(spec: &MockSpec) -> Vec<String> {
    let mut reqs = Vec::new();
    let mut kinds = TransportKinds::default();
    for mock in &spec.transport_mocks {
        scan_transport_value(&mock.value, &mut kinds);
    }
    for mock in &spec.boundary_mocks {
        scan_transport_value(&mock.value, &mut kinds);
    }

    if kinds.file {
        reqs.push("fs".to_string());
    }
    if kinds.shell {
        reqs.push("shell".to_string());
    }
    if kinds.rest || kinds.http {
        reqs.push("http".to_string());
    }
    if kinds.tcp {
        reqs.push("tcp".to_string());
    }
    reqs
}

#[derive(Default)]
struct TransportKinds {
    file: bool,
    shell: bool,
    rest: bool,
    http: bool,
    tcp: bool,
}

fn scan_transport_value(value: &Value, kinds: &mut TransportKinds) {
    let response = match value {
        Value::Response(resp) => resp,
        _ => return,
    };
    match response {
        TransportResponse::File(_) => kinds.file = true,
        TransportResponse::Shell(_) => kinds.shell = true,
        TransportResponse::Rest(_) => kinds.rest = true,
        TransportResponse::Http(_) => kinds.http = true,
        TransportResponse::Tcp(_) => kinds.tcp = true,
    }
}
