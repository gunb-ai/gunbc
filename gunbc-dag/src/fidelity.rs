//! DSL-evaluated fidelity classification.
//!
//! Compiles `config/test_policy.dag`, extracts fn bodies and data
//! declarations, then evaluates `classify_from_facts()` to produce
//! test tier / depth / hermetic classification from structural graph facts.
//!
//! The Rust layer pre-aggregates transport classes (max depth, all-hermetic)
//! before calling DSL. This split avoids the lowerer limitation where `fold`
//! operations in fn bodies prevent `fn_body` extraction.
//!
//! Follows the proven pattern from `pragma/dsl_render.rs`.

use std::collections::HashMap;
use std::path::PathBuf;

use daglang_derive::CallableProperties;
use daglang_driver::{compile_from_context, DriverContext};
use daglang_lower::{CallableKind, LoweredFnBody, LoweredOp, ServiceTransportClass};
use gunbc_ir::node::NodeBody;
use gunbc_ir::Value;

/// Classification result for a callable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidelityClassification {
    pub tier: String,
    pub depth: String,
    pub hermetic: bool,
}

/// Compile `config/test_policy.dag` and extract fn bodies + data values.
fn compile_test_policy() -> (
    HashMap<String, LoweredFnBody>,
    HashMap<String, serde_json::Value>,
) {
    let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dsl");
    let dag_file = dsl_root.join("config/test_policy.dag");
    let context = DriverContext {
        roots: vec![dsl_root],
        target_file: Some(dag_file),
    };
    let output = compile_from_context(&context).expect("test_policy.dag should compile");

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

    (fns, output.data_values)
}

/// Ordinal for a transport class (matches DSL `transport_depth` ordering).
fn transport_depth_ordinal(tc: &ServiceTransportClass) -> u8 {
    match tc {
        ServiceTransportClass::LocalDirect | ServiceTransportClass::InterfaceStub => 0,
        ServiceTransportClass::ShellLocal | ServiceTransportClass::FileBoundary => 1,
        ServiceTransportClass::RestNetwork => 3,
        ServiceTransportClass::Unknown => 4,
    }
}

/// Depth string for a transport class.
fn transport_depth_str(tc: &ServiceTransportClass) -> &'static str {
    match tc {
        ServiceTransportClass::LocalDirect | ServiceTransportClass::InterfaceStub => "XS",
        ServiceTransportClass::ShellLocal | ServiceTransportClass::FileBoundary => "S",
        ServiceTransportClass::RestNetwork => "L",
        ServiceTransportClass::Unknown => "XL",
    }
}

/// Whether a transport class is hermetic (can be mocked in DryRun).
fn transport_is_hermetic(tc: &ServiceTransportClass) -> bool {
    matches!(
        tc,
        ServiceTransportClass::LocalDirect
            | ServiceTransportClass::InterfaceStub
            | ServiceTransportClass::ShellLocal
            | ServiceTransportClass::FileBoundary
    )
}

/// Classify a callable using DSL-evaluated policy.
///
/// Pre-aggregates transport facts in Rust (max depth, all-hermetic),
/// then evaluates `classify_from_facts()` from `config/test_policy.dag`.
pub fn classify_callable(props: &CallableProperties) -> FidelityClassification {
    let (fns, _data) = compile_test_policy();

    // Rust-side aggregation: max depth, all-hermetic
    let max_depth = props
        .transport_classes
        .iter()
        .max_by_key(|tc| transport_depth_ordinal(tc))
        .map(|tc| transport_depth_str(tc))
        .unwrap_or("XS");

    let hermetic = props
        .transport_classes
        .iter()
        .all(transport_is_hermetic);

    let has_transports = !props.transport_classes.is_empty();

    // DSL-side classification: tier from facts
    let mut inputs = HashMap::new();
    inputs.insert("max_depth".to_string(), Value::Str(max_depth.to_string()));
    inputs.insert("hermetic".to_string(), Value::Bool(hermetic));
    inputs.insert("has_transports".to_string(), Value::Bool(has_transports));

    let body = fns
        .get("classify_from_facts")
        .expect("classify_from_facts fn body should exist in test_policy.dag");
    let result = daglang_lower::eval::evaluate_fn_body(body, &inputs, &fns)
        .expect("classify_from_facts should evaluate");

    let tier = result
        .get("tier")
        .and_then(Value::as_str)
        .unwrap_or("Unit")
        .to_string();
    let depth = result
        .get("depth")
        .and_then(Value::as_str)
        .unwrap_or("XS")
        .to_string();
    let hermetic = result
        .get("hermetic")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(true);

    FidelityClassification {
        tier,
        depth,
        hermetic,
    }
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
        assert_eq!(result.tier, "Unit");
        assert_eq!(result.depth, "XS");
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
        assert_eq!(result.tier, "Hermetic");
        assert_eq!(result.depth, "S");
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
        assert_eq!(result.tier, "Integration");
        assert_eq!(result.depth, "L");
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
        assert_eq!(result.tier, "Integration");
        assert_eq!(result.depth, "L");
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
        assert_eq!(result.tier, "Hermetic");
        assert_eq!(result.depth, "XS");
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
        assert_eq!(result.tier, "Hermetic");
        assert_eq!(result.depth, "XS");
        assert!(result.hermetic);
    }
}
