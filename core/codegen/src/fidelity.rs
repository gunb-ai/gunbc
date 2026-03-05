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
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use daglang_derive::CallableProperties;
use daglang_lower::{CallableKind, LoweredFnBody, LoweredOp, ServiceTransportClass};
use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_syntax::ast::ModulePath;
use daglang_syntax::parser;
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

const STD_TYPES_SOURCE: &str = include_str!("../../../dsl/std/types.dag");
const STD_FERMI_SOURCE: &str = include_str!("../../../dsl/std/fermi.dag");
const STD_FIDELITY_SOURCE: &str = include_str!("../../../dsl/std/fidelity.dag");

/// Embedded stdlib evaluator host with one-time compilation cache.
struct StdLibHost {
    fns: HashMap<String, LoweredFnBody>,
}

impl StdLibHost {
    fn global() -> &'static Self {
        static HOST: OnceLock<StdLibHost> = OnceLock::new();
        HOST.get_or_init(|| StdLibHost {
            fns: compile_stdlib_fns(),
        })
    }

    fn eval_fn(&self, fn_name: &str, inputs: &HashMap<String, Value>) -> HashMap<String, Value> {
        let body = self
            .fns
            .get(fn_name)
            .unwrap_or_else(|| panic!("stdlib function `{fn_name}` not found"));
        daglang_lower::eval::evaluate_fn_body(body, inputs, &self.fns)
            .unwrap_or_else(|e| panic!("stdlib function `{fn_name}` failed to evaluate: {e}"))
    }
}

fn parse_embedded_module(
    path: &Path,
    source: &str,
) -> (ModulePath, Vec<ModulePath>, daglang_syntax::ast::SourceFile) {
    let ast = parser::parse_with_file_diagnostics(path, source).unwrap_or_else(|diags| {
        panic!("failed to parse embedded stdlib module {path:?}: {diags:?}")
    });
    let module_path = ast
        .module_path
        .as_ref()
        .map(|mp| mp.node.clone())
        .unwrap_or_else(|| {
            panic!("embedded stdlib module {path:?} is missing `module` declaration")
        });
    let imports = ast
        .imports
        .iter()
        .map(|imp| imp.node.path.clone())
        .collect::<Vec<_>>();
    (module_path, imports, ast)
}

fn build_embedded_stdlib_graph() -> ModuleGraph {
    let modules = vec![
        (PathBuf::from("<stdlib>/std/types.dag"), STD_TYPES_SOURCE),
        (PathBuf::from("<stdlib>/std/fermi.dag"), STD_FERMI_SOURCE),
        (
            PathBuf::from("<stdlib>/std/fidelity.dag"),
            STD_FIDELITY_SOURCE,
        ),
    ];

    let mut parsed = Vec::new();
    for (path, source) in modules {
        let (module_path, imports, ast) = parse_embedded_module(path.as_path(), source);
        parsed.push((path, module_path, imports, ast));
    }

    let mut index_by_module = HashMap::new();
    for (idx, (_, module_path, _, _)) in parsed.iter().enumerate() {
        index_by_module.insert(module_path.clone(), idx);
    }

    let mut resolved = Vec::new();
    for (path, module_path, imports, ast) in parsed {
        let mut dependencies = Vec::new();
        for import in imports {
            let dep = index_by_module.get(&import).copied().unwrap_or_else(|| {
                panic!(
                    "embedded stdlib import `{}` missing from host graph",
                    import.as_dotted()
                )
            });
            dependencies.push(dep);
        }
        resolved.push(ResolvedModule {
            path,
            ast,
            module_path,
            dependencies,
        });
    }

    ModuleGraph { modules: resolved }
}

fn compile_stdlib_fns() -> HashMap<String, LoweredFnBody> {
    let graph = build_embedded_stdlib_graph();
    let typed = daglang_typecheck::typecheck_module_graph(graph)
        .unwrap_or_else(|errs| panic!("embedded stdlib typecheck failed: {errs:?}"));
    let lowered = daglang_lower::lower_typed_project(&typed)
        .unwrap_or_else(|e| panic!("embedded stdlib lowering failed: {e}"));

    let mut fns = HashMap::new();
    for node in &lowered.nodes {
        match &node.body {
            // Legacy path: Callable with fn_body
            NodeBody::Opaque(LoweredOp::Callable {
                kind: CallableKind::Fn,
                name,
                fn_body: Some(body),
                ..
            }) => {
                fns.insert(name.clone(), *body.clone());
            }
            // Bridge 1: SubDag containing FnBodyCompute
            NodeBody::SubDag(inner, _) => {
                for inner_node in &inner.nodes {
                    if let NodeBody::Opaque(LoweredOp::Primitive {
                        kind: daglang_lower::PrimitiveOpKind::FnBodyCompute {
                            fn_body, ..
                        },
                        name,
                        ..
                    }) = &inner_node.body
                    {
                        fns.insert(name.clone(), *fn_body.clone());
                    }
                }
            }
            _ => {}
        }
    }
    fns
}

/// Convert a `ServiceTransportClass` to the DSL `TransportClass` variant name.
fn transport_class_to_dsl(tc: &ServiceTransportClass) -> Value {
    Value::Enum {
        ty: "TransportClass".to_string(),
        variant: match tc {
            ServiceTransportClass::LocalDirect => "LocalDirect",
            ServiceTransportClass::InterfaceStub => "InterfaceStub",
            ServiceTransportClass::ShellLocal => "ShellLocal",
            ServiceTransportClass::FileBoundary => "FileBoundary",
            ServiceTransportClass::RestNetwork => "RestNetwork",
            ServiceTransportClass::Unknown => "Unknown",
        }
        .to_string(),
    }
}

fn enum_variant(value: &Value) -> Option<&str> {
    match value {
        Value::Enum { variant, .. } => Some(variant.as_str()),
        Value::Str(value) => Some(value.as_str()),
        _ => None,
    }
}

fn decode_test_class(value: Option<&Value>) -> TestClass {
    match value.and_then(enum_variant) {
        Some("Unit") => TestClass::Unit,
        Some("Hermetic") => TestClass::Hermetic,
        Some("Integration") => TestClass::Integration,
        other => panic!(
            "classify_transports returned invalid test_class: {:?}",
            other
        ),
    }
}

fn decode_fermi_cost(value: Option<&Value>) -> FermiCost {
    match value.and_then(enum_variant) {
        Some("Xs") => FermiCost::XS,
        Some("S") => FermiCost::S,
        Some("M") => FermiCost::M,
        Some("L") => FermiCost::L,
        Some("Xl") => FermiCost::XL,
        other => panic!("classify_transports returned invalid depth: {:?}", other),
    }
}

/// Classify a callable using DSL-evaluated `classify_transports()`.
///
/// Converts transport classes to DSL values and evaluates the full
/// classification pipeline in DSL — including fold-based max depth
/// and all-hermetic checks.
pub fn classify_callable(props: &CallableProperties) -> FidelityClassification {
    let stdlib = StdLibHost::global();

    let transports: Vec<Value> = props
        .transport_classes
        .iter()
        .map(transport_class_to_dsl)
        .collect();

    let mut inputs = HashMap::new();
    inputs.insert("transports".to_string(), Value::List(transports));

    let result = stdlib.eval_fn("classify_transports", &inputs);

    let test_class = decode_test_class(result.get("test_class"));
    let fermi_cost = decode_fermi_cost(result.get("depth"));
    let hermetic_value = result.get("hermetic");
    let hermetic = match hermetic_value {
        Some(Value::Bool(b)) => *b,
        other => panic!("classify_transports returned invalid hermetic: {:?}", other),
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
/// This means a module's classification is the **join** (least
/// upper bound) of its callables' individual classifications:
///
/// - If *any* callable uses `RestNetwork`, the whole module is classified as
///   `Integration` (requires network access).
/// - `idempotent` and `readonly` are only `true` when *all* callables are.
/// - Auth requirements propagate transitively: if callable A calls service S
///   which has `config { auth: BearerToken }`, the lowerer stamps A with the
///   auth transport class, and `classify_module` unions it into the aggregate.
///
/// Consequence: a single authenticated callable inflates the module to
/// `Integration` tier even if all other callables are pure. This is
/// intentionally conservative — test classification should use per-callable
/// properties for fine-grained budgeting.
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
                idempotent: true,
                readonly: true,
                service_operations: vec![],
            },
        );
        props_map.insert(
            "mod::shell_fn".to_string(),
            CallableProperties {
                transport_classes: vec![ServiceTransportClass::ShellLocal],
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
