//! Transport derivation: builds transport triplets as pure data.
//!
//! This module defines `TransportManifest` — the pure-data result of transport
//! derivation. The manifest contains all nodes, edges, and registry entries
//! needed to represent transport triplets in the DAG.
//!
//! Invariant: every service call site maps to exactly one transport triplet
//! (prepare → execute → parse).

use std::collections::HashSet;

use daglang_syntax::ast::{CapabilityDef, Item, OperationDef};
use daglang_syntax::ast_utils::type_expr_to_string;
use daglang_typecheck::TypedProject;
use gunbc_ir::resource::AccessMode;
use gunbc_ir::{Cardinality, Guard, Node, OperationKey, Port, PortName, Value};

use crate::{
    build_data_registry, derive_service_call_metadata, is_bound_interface_type_name,
    sanitize_identifier, validate_provider_config_fields, validate_rest_output_field_paths,
    CallableKind, LowerError, LoweredEndpoint, LoweredOp, ObligationCategory,
    ServiceCallMetadata, ServiceEndpointRegistry, ServiceOperationSpec, ServiceTransportClass,
    ServiceTransportEndpoint,
};

/// A deferred edge to be applied to the DagBuilder.
pub(crate) struct ManifestEdge {
    pub from_node: String,
    pub from_port: String,
    pub to_node: String,
    pub to_port: String,
}

/// Pure-data result of transport derivation.
///
/// Contains all the nodes, edges, and registry entries needed to represent
/// transport triplets in the DAG, without mutating any builder directly.
pub(crate) struct TransportManifest {
    pub nodes: Vec<Node<LoweredOp>>,
    pub edges: Vec<ManifestEdge>,
    pub registry: ServiceEndpointRegistry,
}

impl TransportManifest {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            registry: ServiceEndpointRegistry::default(),
        }
    }

    pub fn add_node(&mut self, node: Node<LoweredOp>) {
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) {
        self.edges.push(ManifestEdge {
            from_node: from_node.to_string(),
            from_port: from_port.to_string(),
            to_node: to_node.to_string(),
            to_port: to_port.to_string(),
        });
    }
}

// ============================================================================
// Prepare port builders
// ============================================================================

fn service_prepare_ports(operation: &OperationDef, metadata: &ServiceCallMetadata) -> Vec<Port> {
    // Carry (name, type, has_default) so we can set optional cardinality for defaulted inputs.
    let declared_inputs: Vec<(String, String, bool)> = match metadata.spec.as_ref() {
        Some(spec) if !spec.input_fields().is_empty() => spec
            .input_fields()
            .iter()
            .map(|field| (field.name.clone(), field.type_id.clone(), field.default.is_some()))
            .collect(),
        _ => operation
            .inputs
            .iter()
            .map(|field| {
                let ty = type_expr_to_string(&field.ty);
                (field.name.clone(), ty, field.default.is_some())
            })
            .collect(),
    };
    declared_inputs
        .into_iter()
        .map(|(name, ty, has_default)| {
            let cardinality = if has_default {
                Cardinality::ZERO_OR_ONE
            } else {
                Cardinality::ONE
            };
            Port::with_cardinality(name.as_str(), ty.as_str(), cardinality)
        })
        .collect()
}

fn capability_prepare_ports(
    capability: &CapabilityDef,
    metadata: &ServiceCallMetadata,
) -> Vec<Port> {
    // When a spec with explicit input fields is available (e.g., File operations),
    // use the spec's field declarations. Otherwise fall back to the capability's
    // declared inputs from the interface definition.
    let declared_inputs: Vec<(String, String, bool)> = match metadata.spec.as_ref() {
        Some(spec) if !spec.input_fields().is_empty() => spec
            .input_fields()
            .iter()
            .map(|field| (field.name.clone(), field.type_id.clone(), field.default.is_some()))
            .collect(),
        _ => capability
            .inputs
            .iter()
            .map(|field| {
                let ty = type_expr_to_string(&field.ty);
                (field.name.clone(), ty, field.default.is_some())
            })
            .collect(),
    };
    declared_inputs
        .into_iter()
        .map(|(name, ty, has_default)| {
            let cardinality = if has_default {
                Cardinality::ZERO_OR_ONE
            } else {
                Cardinality::ONE
            };
            Port::with_cardinality(name.as_str(), ty.as_str(), cardinality)
        })
        .collect()
}

// ============================================================================
// Transport triplet derivation
// ============================================================================

pub(crate) fn derive_service_transport_triplets(
    project: &TypedProject,
    required_calls: Option<&HashSet<String>>,
) -> Result<TransportManifest, LowerError> {
    let data_registry = build_data_registry(project);
    let mut manifest = TransportManifest::new();
    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Item::ServiceDef(service) = &item.node else {
                continue;
            };

            // SR-8: Validate provider-specific config fields against
            // schemas derived from dsl/std/provider_config.dag.
            validate_provider_config_fields(service, project)?;

            for operation in &service.operations {
                if let Some(required_calls) = required_calls {
                    let canonical = format!("{}.{}", service.name, operation.name);
                    let service_tail = service
                        .name
                        .rsplit('.')
                        .next()
                        .unwrap_or(service.name.as_str());
                    let short = format!("{service_tail}.{}", operation.name);
                    let module_scoped =
                        format!("{}.{}.{}", module_name, service.name, operation.name);
                    if !required_calls.contains(&canonical)
                        && !required_calls.contains(&short)
                        && !required_calls.contains(&module_scoped)
                    {
                        continue;
                    }
                }
                validate_rest_output_field_paths(service, operation)?;
                let service_metadata =
                    derive_service_call_metadata(service, operation, &data_registry);
                // RT4: Fail-closed when a service operation has no transport
                // block. Previously this silently created a triplet with no
                // spec, causing the executor to skip the operation.
                //
                // Exempt fully-abstract services: if NO operation in the
                // service has a transport block, the service is intended
                // for profile-based transport binding (e.g., infra/aws,
                // infra/azure providers). Also exempt interface implementors.
                if operation.transport.is_none() && service.implements.is_none() {
                    let service_has_any_transport =
                        service.operations.iter().any(|op| op.transport.is_some());
                    if service_has_any_transport {
                        return Err(LowerError::MissingTransport {
                            service: service.name.clone(),
                            operation: operation.name.clone(),
                        });
                    }
                }

                // SC-7: Validate auth_input references a real Secret-typed field.
                if let Some(ref auth_field_name) = service.config.auth_input {
                    if operation.transport.is_some() {
                        let matching_field =
                            operation.inputs.iter().find(|f| f.name == *auth_field_name);
                        match matching_field {
                            None => {
                                return Err(LowerError::InvalidAuthInput {
                                    service: service.name.clone(),
                                    operation: operation.name.clone(),
                                    field_name: auth_field_name.clone(),
                                    reason: format!(
                                        "field `{auth_field_name}` not found in operation inputs"
                                    ),
                                });
                            }
                            Some(field) => {
                                let field_type = type_expr_to_string(&field.ty);
                                if field_type != "Secret" {
                                    return Err(LowerError::InvalidAuthInput {
                                        service: service.name.clone(),
                                        operation: operation.name.clone(),
                                        field_name: auth_field_name.clone(),
                                        reason: format!(
                                            "field must be type `Secret`, found `{field_type}`"
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }

                let suffix = sanitize_identifier(&format!(
                    "{module_name}_{}_{}",
                    service.name, operation.name
                ));
                let prepare_id = format!("prepare_transport_{suffix}");
                let execute_id = format!("execute_transport_{suffix}");
                let parse_id = format!("parse_transport_{suffix}");
                let prepare_ports = service_prepare_ports(operation, &service_metadata);
                let prepare_inputs = prepare_ports
                    .iter()
                    .map(|port| port.name.0.clone())
                    .collect::<Vec<_>>();

                manifest.add_node(Node::opaque(
                    prepare_id.clone(),
                    prepare_ports,
                    vec![Port::scalar("request", "TransportRequest")],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::prepare::{}::{}",
                            service.name, operation.name
                        ),
                        obligation: ObligationCategory::ServiceTransportPrepare,
                        service_metadata: Some(Box::new(service_metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                ));
                let has_auth = matches!(
                    &service_metadata.spec,
                    Some(ServiceOperationSpec::Rest(spec)) if spec.auth_scheme.is_some()
                );
                let mut execute_inputs = vec![
                    Port::scalar("request", "TransportRequest"),
                    Port::resource("file", "FilesystemHandle", AccessMode::Read),
                ];
                if has_auth {
                    execute_inputs.push(Port::with_cardinality(
                        PortName::RESOURCE_CREDENTIAL,
                        "Credential",
                        Cardinality::ZERO_OR_ONE,
                    ));
                }
                let execute_node = Node::opaque(
                    execute_id.clone(),
                    execute_inputs,
                    vec![Port::scalar("response", "TransportResponse")],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::execute::{}::{}",
                            service.name, operation.name
                        ),
                        obligation: ObligationCategory::ServiceTransportExecute,
                        service_metadata: Some(Box::new(service_metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                )
                .with_input_guard("request", Guard::NotEq(Value::Skipped))
                .with_operation_key(OperationKey::new(&service.name, &operation.name));
                manifest.add_node(execute_node);
                let parse_outputs = if operation.outputs.is_empty() {
                    vec![Port::scalar("result", "Unit")]
                } else {
                    operation
                        .outputs
                        .iter()
                        .map(|field| {
                            let ty = type_expr_to_string(&field.ty);
                            Port::with_cardinality(
                                field.name.as_str(),
                                ty.as_str(),
                                Cardinality::ONE,
                            )
                        })
                        .collect::<Vec<_>>()
                };
                manifest.add_node(Node::opaque(
                    parse_id.clone(),
                    vec![Port::scalar("response", "TransportResponse")],
                    parse_outputs,
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::parse::{}::{}",
                            service.name, operation.name
                        ),
                        obligation: ObligationCategory::ServiceTransportParse,
                        service_metadata: Some(Box::new(service_metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                ));

                manifest.add_edge(
                    prepare_id.as_str(),
                    "request",
                    execute_id.as_str(),
                    "request",
                );
                manifest.add_edge(
                    execute_id.as_str(),
                    "response",
                    parse_id.as_str(),
                    "response",
                );

                let parse_output = operation
                    .outputs
                    .first()
                    .map(|field| field.name.clone())
                    .unwrap_or_else(|| "result".to_string());
                let operation_inputs = operation
                    .inputs
                    .iter()
                    .map(|field| field.name.clone())
                    .collect::<Vec<_>>();
                let endpoint = ServiceTransportEndpoint {
                    parse: LoweredEndpoint {
                        node_id: parse_id,
                        primary_output: parse_output,
                    },
                    prepare_node_id: prepare_id,
                    execute_node_id: execute_id,
                    prepare_inputs,
                    operation_inputs,
                    has_auth,
                    metadata: Some(service_metadata),
                };
                manifest.registry.register(
                    format!("{}.{}", service.name, operation.name),
                    endpoint.clone(),
                );
                let service_tail = service
                    .name
                    .rsplit('.')
                    .next()
                    .unwrap_or(service.name.as_str());
                manifest.registry.register(
                    format!("{service_tail}.{}", operation.name),
                    endpoint.clone(),
                );
                manifest.registry.register(
                    format!("{}.{}.{}", module_name, service.name, operation.name),
                    endpoint,
                );
            }
        }
    }
    Ok(manifest)
}

/// Register stub transport triplets for interfaces that lack profile bindings (IS-4).
///
/// When compiling without a profile, interface capabilities still need transport
/// triplets in the registry so `resolve_service_call_source` can find them. These
/// stubs use `ServiceTransportClass::InterfaceStub` and are DryRun-compatible;
/// real-mode execution will surface a "requires --profile" error at the resolver.
pub(crate) fn derive_interface_stub_transport_triplets(
    project: &TypedProject,
    stub_interfaces: &HashSet<String>,
) -> TransportManifest {
    let mut manifest = TransportManifest::new();
    if stub_interfaces.is_empty() {
        return manifest;
    }

    for module in &project.modules {
        let module_name = module.module_path.as_dotted();
        for item in &module.ast.items {
            let Item::InterfaceDef(interface) = &item.node else {
                continue;
            };

            if !is_bound_interface_type_name(stub_interfaces, &interface.name) {
                continue;
            }

            for capability in &interface.capabilities {
                let metadata = ServiceCallMetadata {
                    service: interface.name.clone(),
                    operation: capability.name.clone(),
                    transport: ServiceTransportClass::InterfaceStub,
                    idempotent: capability.idempotent,
                    readonly: capability.readonly,
                    spec: Some(ServiceOperationSpec::InterfaceStub {
                        interface: interface.name.clone(),
                        capability: capability.name.clone(),
                    }),
                };

                let suffix = sanitize_identifier(&format!(
                    "{module_name}_{}_{}",
                    interface.name, capability.name
                ));
                let prepare_id = format!("prepare_transport_{suffix}");
                let execute_id = format!("execute_transport_{suffix}");
                let parse_id = format!("parse_transport_{suffix}");

                // Prepare node: capability inputs → TransportRequest.
                let prepare_ports = capability_prepare_ports(capability, &metadata);
                let prepare_inputs = prepare_ports
                    .iter()
                    .map(|port| port.name.0.clone())
                    .collect::<Vec<_>>();

                manifest.add_node(Node::opaque(
                    prepare_id.clone(),
                    prepare_ports,
                    vec![Port::scalar("request", "TransportRequest")],
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::prepare::{}::{}",
                            interface.name, capability.name
                        ),
                        obligation: ObligationCategory::ServiceTransportPrepare,
                        service_metadata: Some(Box::new(metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                ));

                // Execute node: TransportRequest → typed capability outputs.
                // In DryRun, boundary mocks supply typed fields directly.
                // In Real mode, the execute op errors with "requires --profile".
                let typed_outputs = if capability.outputs.is_empty() {
                    vec![Port::scalar("result", "Unit")]
                } else {
                    capability
                        .outputs
                        .iter()
                        .map(|field| {
                            let ty = type_expr_to_string(&field.ty);
                            Port::with_cardinality(
                                field.name.as_str(),
                                ty.as_str(),
                                Cardinality::ONE,
                            )
                        })
                        .collect::<Vec<_>>()
                };
                let execute_node = Node::opaque(
                    execute_id.clone(),
                    vec![
                        Port::scalar("request", "TransportRequest"),
                        Port::resource("file", "FilesystemHandle", AccessMode::Read),
                    ],
                    typed_outputs.clone(),
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::execute::{}::{}",
                            interface.name, capability.name
                        ),
                        obligation: ObligationCategory::ServiceTransportExecute,
                        service_metadata: Some(Box::new(metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                )
                .with_input_guard("request", Guard::NotEq(Value::Skipped))
                .with_operation_key(OperationKey::new(&interface.name, &capability.name));
                manifest.add_node(execute_node);

                // Parse node: typed capability outputs → typed capability outputs (identity).
                manifest.add_node(Node::opaque(
                    parse_id.clone(),
                    typed_outputs.clone(),
                    typed_outputs,
                    LoweredOp::Callable {
                        module: module_name.clone(),
                        kind: CallableKind::Pattern,
                        name: format!(
                            "service_transport::parse::{}::{}",
                            interface.name, capability.name
                        ),
                        obligation: ObligationCategory::ServiceTransportParse,
                        service_metadata: Some(Box::new(metadata.clone())),
                        is_interactive: false,
                        resource_target: None,
                        fn_body: None,
                    },
                ));

                // Wire the triplet: prepare → execute → parse.
                manifest.add_edge(
                    prepare_id.as_str(),
                    "request",
                    execute_id.as_str(),
                    "request",
                );
                // Wire per-field edges from execute to parse.
                for field in &capability.outputs {
                    manifest.add_edge(
                        execute_id.as_str(),
                        field.name.as_str(),
                        parse_id.as_str(),
                        field.name.as_str(),
                    );
                }
                if capability.outputs.is_empty() {
                    manifest.add_edge(execute_id.as_str(), "result", parse_id.as_str(), "result");
                }

                // Register endpoints under multiple keys for flexible resolution.
                let parse_output = capability
                    .outputs
                    .first()
                    .map(|field| field.name.clone())
                    .unwrap_or_else(|| "result".to_string());
                let endpoint = ServiceTransportEndpoint {
                    parse: LoweredEndpoint {
                        node_id: parse_id,
                        primary_output: parse_output,
                    },
                    prepare_node_id: prepare_id,
                    execute_node_id: execute_id,
                    operation_inputs: prepare_inputs.clone(),
                    prepare_inputs,
                    has_auth: false,
                    metadata: Some(metadata),
                };
                let cap_key = format!("{}.{}", interface.name, capability.name);
                manifest
                    .registry
                    .register(cap_key.clone(), endpoint.clone());
                manifest
                    .registry
                    .register(format!("{module_name}.{cap_key}"), endpoint);
            }
        }
    }
    manifest
}
