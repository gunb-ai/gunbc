//! Helpers for resilient graph-mock specs on DSL-backed DAGs.

use gunbc_ir::transport::{ShellRequest, ShellResponse, TransportRequest, TransportResponse};
use gunbc_ir::{detect_boundaries, value_backing_for_type_id, Dag, Value, ValueBacking};
use gunbc_primitives::filename;
use gunbc_test::extract_mock_requirements;
use gunbc_test::MockSpec;

fn default_fs_handle() -> Value {
    let fs = filename::FilesystemHandle::cross_platform(filename::Scope::Write);
    fs.into()
}

fn default_value_for_type(type_id: &str) -> Value {
    match type_id {
        "TransportResponse" => {
            Value::Response(TransportResponse::Shell(ShellResponse::ok(String::new())))
        }
        "TransportRequest" => {
            Value::Request(TransportRequest::Shell(ShellRequest::new("true")))
        }
        "Secret" => Value::Secret(gunbc_ir::SecretString::new("mock")),
        "FilesystemHandle" => default_fs_handle(),
        _ => match value_backing_for_type_id(type_id) {
            ValueBacking::String => Value::Str("mock".to_string()),
            ValueBacking::Bool => Value::Bool(true),
            ValueBacking::Int | ValueBacking::Float => Value::Int(1),
            ValueBacking::Json => Value::Json(serde_json::json!({"mock": true})),
            ValueBacking::Map => Value::Map(std::collections::BTreeMap::new()),
            ValueBacking::List => Value::List(vec![Value::Str("mock".to_string())]),
            ValueBacking::Set => Value::Set(vec![Value::Str("mock".to_string())]),
            ValueBacking::Unit => Value::Unit,
            ValueBacking::Bytes => Value::List(vec![Value::Int(0)]),
        },
    }
}

/// Build a `MockSpec` by auto-filling required slots with type-compatible defaults.
pub fn auto_mock_spec<T: Clone>(dag: &Dag<T>, name: &str) -> MockSpec {
    let mut reqs = extract_mock_requirements(dag, name);
    let lowered =
        gunbc_exec::lower(dag).unwrap_or_else(|error| panic!("failed to lower {name}: {error}"));
    for node in &lowered.dag.nodes {
        reqs = reqs.skip_node_example(node.id.0.as_str());
    }
    loop {
        let missing: Vec<(String, String, String)> = reqs
            .missing_slots()
            .into_iter()
            .map(|slot| {
                (
                    slot.node_id.0.clone(),
                    slot.port_name.0.clone(),
                    slot.type_id.0.clone(),
                )
            })
            .collect();
        if missing.is_empty() {
            break;
        }
        for (node, port, type_id) in missing {
            let value = default_value_for_type(type_id.as_str());
            reqs = reqs
                .boundary(node.as_str(), port.as_str(), value)
                .unwrap_or_else(|error| {
                    panic!("failed to auto-fill mock slot {node}.{port} ({type_id}): {error}")
            });
        }
    }
    let mut spec = reqs.build_unchecked();
    let boundaries = detect_boundaries(&lowered.dag);
    for (node_id, port_name) in &boundaries.boundary_ports {
        if spec
            .get_boundary_mock(node_id.0.as_str(), port_name.0.as_str())
            .is_some()
        {
            continue;
        }
        let Some(node) = lowered.dag.get_node(node_id) else {
            continue;
        };
        let Some(port) = node.outputs.iter().find(|p| p.name == *port_name) else {
            continue;
        };
        spec = spec.boundary(
            node_id.0.as_str(),
            port_name.0.as_str(),
            default_value_for_type(port.type_id.0.as_str()),
        );
    }
    spec
}
