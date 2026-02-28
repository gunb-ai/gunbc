//! DSL-evaluated fidelity classification.
//!
//! Compiles `std/fidelity.dag` and evaluates `classify_transports()` to
//! produce test tier / depth / hermetic classification from structural graph
//! facts. All aggregation (max depth via fold, all-hermetic) now happens in
//! the DSL — the Rust side only converts `ServiceTransportClass` to DSL
//! values and interprets the result.
//!
//! Follows the proven pattern from `pragma/dsl_render.rs`.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use daglang_derive::CallableProperties;
use daglang_driver::{compile_from_context, DriverContext};
use daglang_lower::{CallableKind, LoweredFnBody, LoweredOp, ServiceTransportClass};
use gunbc_ir::node::NodeBody;
use gunbc_ir::Value;
use gunbc_test::{FermiCost, TestClass};

/// Classification result for a callable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidelityClassification {
    pub test_class: TestClass,
    pub fermi_cost: FermiCost,
    pub hermetic: bool,
}

/// Compile `std/fidelity.dag` and extract fn bodies (including transitive
/// imports from `std/fermi.dag`).
fn compile_fidelity() -> HashMap<String, LoweredFnBody> {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dsl");
    let dag_file = dsl_root.join("std/fidelity.dag");
    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(dag_file),
    };
    let output = compile_from_context(&context).expect("std/fidelity.dag should compile");

    let mut fns = HashMap::new();
    for node in &output.lowered_dag.nodes {
        if let NodeBody::Opaque(LoweredOp::Callable {
            kind: CallableKind::Fn,
            name,
            fn_body: Some(body),
            ..
        }) = &node.body
        {
            fns.insert(name.clone(), *body.clone());
        }
    }

    fns
}

/// Convert a `ServiceTransportClass` to the DSL `TransportClass` variant name.
fn transport_class_to_dsl(tc: &ServiceTransportClass) -> Value {
    Value::Str(match tc {
        ServiceTransportClass::LocalDirect => "LocalDirect",
        ServiceTransportClass::InterfaceStub => "InterfaceStub",
        ServiceTransportClass::ShellLocal => "ShellLocal",
        ServiceTransportClass::FileBoundary => "FileBoundary",
        ServiceTransportClass::RestNetwork => "RestNetwork",
        ServiceTransportClass::Unknown => "Unknown",
    }.to_string())
}

/// Classify a callable using DSL-evaluated `classify_transports()`.
///
/// Converts transport classes to DSL values and evaluates the full
/// classification pipeline in DSL — including fold-based max depth
/// and all-hermetic checks.
pub fn classify_callable(props: &CallableProperties) -> FidelityClassification {
    let fns = compile_fidelity();

    let transports: Vec<Value> = props
        .transport_classes
        .iter()
        .map(transport_class_to_dsl)
        .collect();

    let mut inputs = HashMap::new();
    inputs.insert("transports".to_string(), Value::List(transports));

    let body = fns
        .get("classify_transports")
        .expect("classify_transports fn body should exist in std/fidelity.dag");
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, &fns)
        .expect("classify_transports should evaluate");

    let test_class_value = result.get("test_class");
    let test_class = test_class_value
        .and_then(Value::as_str)
        .and_then(TestClass::parse)
        .unwrap_or_else(|| {
            panic!(
                "classify_transports returned invalid test_class: {:?}",
                test_class_value
            )
        });
    let depth_value = result.get("depth");
    let fermi_cost = depth_value
        .and_then(Value::as_str)
        .and_then(FermiCost::parse)
        .unwrap_or_else(|| {
            panic!(
                "classify_transports returned invalid depth: {:?}",
                depth_value
            )
        });
    let hermetic_value = result.get("hermetic");
    let hermetic = match hermetic_value {
        Some(Value::Bool(b)) => *b,
        other => panic!(
            "classify_transports returned invalid hermetic: {:?}",
            other
        ),
    };

    FidelityClassification {
        test_class,
        fermi_cost,
        hermetic,
    }
}

/// Derive `requires` tags from transport classes.
///
/// Maps each transport class to a human-readable requirement tag:
/// ShellLocal → "shell", FileBoundary → "fs", RestNetwork → "http".
pub fn requires_from_transport_classes(classes: &[ServiceTransportClass]) -> Vec<String> {
    let mut reqs = Vec::new();
    for tc in classes {
        match tc {
            ServiceTransportClass::ShellLocal => {
                if !reqs.contains(&"shell".to_string()) {
                    reqs.push("shell".to_string());
                }
            }
            ServiceTransportClass::FileBoundary => {
                if !reqs.contains(&"fs".to_string()) {
                    reqs.push("fs".to_string());
                }
            }
            ServiceTransportClass::RestNetwork => {
                if !reqs.contains(&"http".to_string()) {
                    reqs.push("http".to_string());
                }
            }
            ServiceTransportClass::LocalDirect
            | ServiceTransportClass::InterfaceStub
            | ServiceTransportClass::Unknown => {}
        }
    }
    reqs
}

/// Classify an entire module by aggregating all callables' properties.
///
/// Unions all transport classes across callables and classifies the aggregate.
pub fn classify_module(
    callable_properties: &BTreeMap<String, CallableProperties>,
) -> FidelityClassification {
    let mut all_classes = Vec::new();
    for props in callable_properties.values() {
        all_classes.extend(props.transport_classes.iter().cloned());
    }
    all_classes.sort();
    all_classes.dedup();

    let aggregate = CallableProperties {
        transport_classes: all_classes,
        permissions: callable_properties
            .values()
            .flat_map(|p| p.permissions.iter().cloned())
            .collect(),
        idempotent: callable_properties.values().all(|p| p.idempotent),
        readonly: callable_properties.values().all(|p| p.readonly),
        service_operations: callable_properties
            .values()
            .flat_map(|p| p.service_operations.iter().cloned())
            .collect(),
    };

    classify_callable(&aggregate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_derive::CallableProperties;
    use daglang_lower::ServiceTransportClass;

    #[test]
    fn classify_pure_callable_is_unit() {
        let props = CallableProperties {
            transport_classes: vec![],
            permissions: vec![],
            idempotent: true,
            readonly: true,
            service_operations: vec![],
        };
        let result = classify_callable(&props);
        assert_eq!(result.test_class, TestClass::Unit);
        assert_eq!(result.fermi_cost, FermiCost::XS);
        assert!(result.hermetic);
    }

    #[test]
    fn classify_shell_callable_is_hermetic() {
        let props = CallableProperties {
            transport_classes: vec![ServiceTransportClass::ShellLocal],
            permissions: vec![],
            idempotent: true,
            readonly: true,
            service_operations: vec![("git".to_string(), "status".to_string())],
        };
        let result = classify_callable(&props);
        assert_eq!(result.test_class, TestClass::Hermetic);
        assert_eq!(result.fermi_cost, FermiCost::S);
        assert!(result.hermetic);
    }

    #[test]
    fn classify_rest_callable_is_integration() {
        let props = CallableProperties {
            transport_classes: vec![ServiceTransportClass::RestNetwork],
            permissions: vec!["repo".to_string()],
            idempotent: false,
            readonly: false,
            service_operations: vec![("github".to_string(), "create_issue".to_string())],
        };
        let result = classify_callable(&props);
        assert_eq!(result.test_class, TestClass::Integration);
        assert_eq!(result.fermi_cost, FermiCost::L);
        assert!(!result.hermetic);
    }

    #[test]
    fn classify_mixed_transports_uses_max() {
        let props = CallableProperties {
            transport_classes: vec![
                ServiceTransportClass::ShellLocal,
                ServiceTransportClass::RestNetwork,
            ],
            permissions: vec![],
            idempotent: true,
            readonly: true,
            service_operations: vec![],
        };
        let result = classify_callable(&props);
        // RestNetwork dominates: Integration, L, not hermetic
        assert_eq!(result.test_class, TestClass::Integration);
        assert_eq!(result.fermi_cost, FermiCost::L);
        assert!(!result.hermetic);
    }

    #[test]
    fn classify_local_direct_is_hermetic_xs() {
        let props = CallableProperties {
            transport_classes: vec![ServiceTransportClass::LocalDirect],
            permissions: vec![],
            idempotent: true,
            readonly: true,
            service_operations: vec![],
        };
        let result = classify_callable(&props);
        assert_eq!(result.test_class, TestClass::Hermetic);
        assert_eq!(result.fermi_cost, FermiCost::XS);
        assert!(result.hermetic);
    }

    #[test]
    fn classify_interface_stub_is_hermetic_xs() {
        let props = CallableProperties {
            transport_classes: vec![ServiceTransportClass::InterfaceStub],
            permissions: vec![],
            idempotent: true,
            readonly: true,
            service_operations: vec![],
        };
        let result = classify_callable(&props);
        assert_eq!(result.test_class, TestClass::Hermetic);
        assert_eq!(result.fermi_cost, FermiCost::XS);
        assert!(result.hermetic);
    }

    #[test]
    fn classify_module_aggregates_callables() {
        let mut props_map = BTreeMap::new();
        props_map.insert(
            "mod::pure_fn".to_string(),
            CallableProperties {
                transport_classes: vec![],
                permissions: vec![],
                idempotent: true,
                readonly: true,
                service_operations: vec![],
            },
        );
        props_map.insert(
            "mod::shell_fn".to_string(),
            CallableProperties {
                transport_classes: vec![ServiceTransportClass::ShellLocal],
                permissions: vec![],
                idempotent: true,
                readonly: true,
                service_operations: vec![("git".to_string(), "status".to_string())],
            },
        );
        let result = classify_module(&props_map);
        // ShellLocal is the max → Hermetic, S
        assert_eq!(result.test_class, TestClass::Hermetic);
        assert_eq!(result.fermi_cost, FermiCost::S);
        assert!(result.hermetic);
    }

    #[test]
    fn requires_from_transport_classes_maps_correctly() {
        let classes = vec![
            ServiceTransportClass::ShellLocal,
            ServiceTransportClass::FileBoundary,
            ServiceTransportClass::RestNetwork,
            ServiceTransportClass::LocalDirect,
        ];
        let reqs = requires_from_transport_classes(&classes);
        assert!(reqs.contains(&"shell".to_string()));
        assert!(reqs.contains(&"fs".to_string()));
        assert!(reqs.contains(&"http".to_string()));
        assert_eq!(reqs.len(), 3);
    }
}
