//! daglang-typecheck: Type checking and interface resolution.
//!
//! Validates that all types are well-formed, all references resolve to
//! compatible types, refinement constraints are satisfiable, and
//! `implements` clauses are fulfilled.
//!
//! # Pipeline position
//!
//! ```text
//! ResolvedAST → [daglang-typecheck] → TypedAST
//! ```
//!
//! # Key responsibilities
//!
//! - Record and sum type validation
//! - Refinement type constraint checking (`@range`, `@pattern`, etc.)
//! - Generic type instantiation (`List<T>`, `Map<K,V>`, `Queue<T>`)
//! - Interface conformance (`resource X implements Y` — all capabilities present)
//! - `CloudConfig` sum type → provider resolution at compile time
//! - `@contract` annotation validation (behavioral specs are well-typed)
//! - Subtyping via the bounded lattice (§4.1.4 of dsl-design.md)

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_syntax::ast::{Expr, Field, Item, Param, SourceFile, Stmt, TypeExpr, UsesClause};

/// A typechecked project snapshot.
#[derive(Debug)]
pub struct TypedProject {
    pub modules: Vec<TypedModule>,
}

#[derive(Debug, Clone, Copy)]
pub struct TypecheckOptions {
    pub allow_unresolved_imports: bool,
}

impl Default for TypecheckOptions {
    fn default() -> Self {
        Self {
            allow_unresolved_imports: true,
        }
    }
}

/// A typechecked module.
#[derive(Debug)]
pub struct TypedModule {
    pub path: PathBuf,
    pub module_path: Vec<String>,
    pub imports: Vec<Vec<String>>,
    pub ast: SourceFile,
    pub signatures: Vec<TypedItemSignature>,
}

/// A normalized signature captured from a top-level item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedItemSignature {
    Type {
        name: String,
    },
    Fn(TypedCallableSignature),
    Func(TypedCallableSignature),
    Pattern(TypedCallableSignature),
    Service {
        name: String,
        operations: usize,
    },
    Resource {
        name: String,
        implements: Option<String>,
    },
    Interface {
        name: String,
        capabilities: usize,
    },
    Pipeline {
        name: String,
        stages: usize,
    },
}

/// A normalized callable signature for fn/func/pattern items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedCallableSignature {
    pub name: String,
    pub params: Vec<TypedBinding>,
    pub outputs: Vec<TypedBinding>,
}

/// A single typed binding in a callable signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedBinding {
    pub name: String,
    pub ty: String,
}

/// Errors during type checking.
#[derive(Debug)]
pub enum TypeError {
    /// A type name was used but not defined.
    UndefinedType(String),
    /// A field was accessed on a type that doesn't have it.
    NoSuchField { ty: String, field: String },
    /// Type mismatch in assignment or call.
    TypeMismatch { expected: String, got: String },
    /// A resource doesn't implement all capabilities of its interface.
    MissingCapability {
        resource: String,
        interface: String,
        capability: String,
    },
    /// A refinement constraint is unsatisfiable.
    UnsatisfiableRefinement { ty: String, constraint: String },
    /// Generic type parameter count mismatch.
    ArityMismatch { name: String, expected: usize, got: usize },
    /// Duplicate top-level item name in a module.
    DuplicateDefinition { module: String, name: String },
    /// Duplicate parameter name in a callable signature.
    DuplicateParameter { item: String, param: String },
    /// Duplicate output field name in a callable signature.
    DuplicateOutputField { item: String, field: String },
    /// Import target does not exist in the available module graph.
    UnresolvedImport { module: String, target: String },
    /// Resource/service declares an interface that cannot be resolved.
    UnresolvedInterface {
        implementor: String,
        interface: String,
    },
    /// Service omits an operation required by its interface.
    MissingOperation {
        service: String,
        interface: String,
        operation: String,
    },
    /// Implementor signature does not match interface contract.
    InterfaceSignatureMismatch {
        implementor: String,
        interface: String,
        capability: String,
        detail: String,
    },
    /// Call expression used wrong number of arguments.
    CallArityMismatch {
        caller: String,
        callee: String,
        expected: usize,
        got: usize,
    },
    /// Call expression used an unknown named argument.
    UnknownCallArgument {
        caller: String,
        callee: String,
        argument: String,
    },
    /// `uses` clause references an unknown resource/interface type.
    UnknownUsedResourceType {
        item: String,
        binding: String,
        resource_type: String,
    },
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndefinedType(name) => write!(f, "undefined type `{name}`"),
            Self::NoSuchField { ty, field } => {
                write!(f, "type `{ty}` has no field `{field}`")
            }
            Self::TypeMismatch { expected, got } => {
                write!(f, "type mismatch: expected `{expected}`, got `{got}`")
            }
            Self::MissingCapability {
                resource,
                interface,
                capability,
            } => write!(
                f,
                "resource `{resource}` is missing capability `{capability}` for interface `{interface}`"
            ),
            Self::UnsatisfiableRefinement { ty, constraint } => {
                write!(f, "unsatisfiable refinement on `{ty}`: {constraint}")
            }
            Self::ArityMismatch {
                name,
                expected,
                got,
            } => write!(
                f,
                "generic arity mismatch for `{name}`: expected {expected}, got {got}"
            ),
            Self::DuplicateDefinition { module, name } => {
                write!(f, "duplicate definition `{name}` in module `{module}`")
            }
            Self::DuplicateParameter { item, param } => {
                write!(f, "duplicate parameter `{param}` in `{item}`")
            }
            Self::DuplicateOutputField { item, field } => {
                write!(f, "duplicate output field `{field}` in `{item}`")
            }
            Self::UnresolvedImport { module, target } => {
                write!(f, "unresolved import `{target}` in module `{module}`")
            }
            Self::UnresolvedInterface {
                implementor,
                interface,
            } => write!(
                f,
                "`{implementor}` references unresolved interface `{interface}`"
            ),
            Self::MissingOperation {
                service,
                interface,
                operation,
            } => write!(
                f,
                "service `{service}` is missing operation `{operation}` for interface `{interface}`"
            ),
            Self::InterfaceSignatureMismatch {
                implementor,
                interface,
                capability,
                detail,
            } => write!(
                f,
                "`{implementor}` does not match `{interface}.{capability}` contract: {detail}"
            ),
            Self::CallArityMismatch {
                caller,
                callee,
                expected,
                got,
            } => write!(
                f,
                "call arity mismatch in `{caller}` for `{callee}`: expected {expected}, got {got}"
            ),
            Self::UnknownCallArgument {
                caller,
                callee,
                argument,
            } => write!(
                f,
                "unknown named argument `{argument}` in call to `{callee}` within `{caller}`"
            ),
            Self::UnknownUsedResourceType {
                item,
                binding,
                resource_type,
            } => write!(
                f,
                "unknown used resource type `{resource_type}` for binding `{binding}` in `{item}`"
            ),
        }
    }
}

/// Typecheck a discovered module graph and produce typed module signatures.
pub fn typecheck_module_graph(graph: ModuleGraph) -> Result<TypedProject, Vec<TypeError>> {
    typecheck_module_graph_with_options(graph, TypecheckOptions::default())
}

/// Typecheck a discovered module graph with explicit options.
pub fn typecheck_module_graph_with_options(
    graph: ModuleGraph,
    options: TypecheckOptions,
) -> Result<TypedProject, Vec<TypeError>> {
    let known_types = collect_known_types(&graph.modules);
    let callable_registry = collect_unique_callables(&graph.modules);
    let interface_registry = collect_interfaces(&graph.modules);
    let resource_type_registry = collect_resource_types(&graph.modules);
    let available_modules = graph
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect::<HashSet<_>>();
    let mut errors = Vec::new();
    let mut typed_modules = Vec::with_capacity(graph.modules.len());
    let context = TypecheckContext {
        known_types: &known_types,
        callable_registry: &callable_registry,
        interface_registry: &interface_registry,
        resource_type_registry: &resource_type_registry,
        allow_unresolved_references: options.allow_unresolved_imports,
    };

    for module in graph.modules {
        let imports = module
            .ast
            .imports
            .iter()
            .map(|import| import.node.path.segments.clone())
            .collect::<Vec<_>>();
        let module_name = module.module_path.join(".");
        if !options.allow_unresolved_imports {
            for import in &imports {
                let target = import.join(".");
                if !available_modules.contains(&target) {
                    errors.push(TypeError::UnresolvedImport {
                        module: module_name.clone(),
                        target,
                    });
                }
            }
        }
        let signatures = collect_signatures(&module, &context, &module_name, &mut errors);
        typed_modules.push(TypedModule {
            path: module.path,
            module_path: module.module_path,
            imports,
            ast: module.ast,
            signatures,
        });
    }

    if errors.is_empty() {
        Ok(TypedProject {
            modules: typed_modules,
        })
    } else {
        Err(errors)
    }
}

struct TypecheckContext<'a> {
    known_types: &'a HashSet<String>,
    callable_registry: &'a HashMap<String, Option<CallableContract>>,
    interface_registry: &'a InterfaceRegistry,
    resource_type_registry: &'a ResourceTypeRegistry,
    allow_unresolved_references: bool,
}

fn collect_signatures(
    module: &ResolvedModule,
    context: &TypecheckContext<'_>,
    module_name: &str,
    errors: &mut Vec<TypeError>,
) -> Vec<TypedItemSignature> {
    let mut module_known_types = context.known_types.clone();
    for import in &module.ast.imports {
        if let Some(bindings) = &import.node.bindings {
            for binding in bindings {
                module_known_types.insert(binding.clone());
            }
        }
    }

    let mut seen_items = HashSet::new();
    let mut signatures = Vec::new();

    for item in &module.ast.items {
        match &item.node {
            Item::TypeDef(def) => {
                if !seen_items.insert(def.name.clone()) {
                    errors.push(TypeError::DuplicateDefinition {
                        module: module_name.to_string(),
                        name: def.name.clone(),
                    });
                }
                signatures.push(TypedItemSignature::Type {
                    name: def.name.clone(),
                });
            }
            Item::FnDef(def) => {
                record_duplicate_item_name(module_name, &def.name, &mut seen_items, errors);
                validate_params(&def.name, &def.params, &module_known_types, errors);
                validate_type_expr(
                    &def.return_type,
                    &module_known_types,
                    &format!("{}.return", def.name),
                    errors,
                );
                let outputs = vec![TypedBinding {
                    name: "return".to_string(),
                    ty: type_expr_to_string(&def.return_type),
                }];
                validate_callable_body(
                    &def.name,
                    &def.body.stmts,
                    context.callable_registry,
                    errors,
                );
                signatures.push(TypedItemSignature::Fn(TypedCallableSignature {
                    name: def.name.clone(),
                    params: def
                        .params
                        .iter()
                        .map(|param| TypedBinding {
                            name: param.name.clone(),
                            ty: type_expr_to_string(&param.ty),
                        })
                        .collect(),
                    outputs,
                }));
            }
            Item::FuncDef(def) => {
                record_duplicate_item_name(module_name, &def.name, &mut seen_items, errors);
                validate_params(&def.name, &def.params, &module_known_types, errors);
                validate_outputs(&def.name, &def.outputs, &module_known_types, errors);
                validate_uses_clauses(
                    &def.name,
                    &def.uses,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                    errors,
                );
                validate_callable_body(
                    &def.name,
                    &def.body.stmts,
                    context.callable_registry,
                    errors,
                );
                signatures.push(TypedItemSignature::Func(TypedCallableSignature {
                    name: def.name.clone(),
                    params: def
                        .params
                        .iter()
                        .map(|param| TypedBinding {
                            name: param.name.clone(),
                            ty: type_expr_to_string(&param.ty),
                        })
                        .collect(),
                    outputs: def
                        .outputs
                        .iter()
                        .map(|field| TypedBinding {
                            name: field.name.clone(),
                            ty: type_expr_to_string(&field.ty),
                        })
                        .collect(),
                }));
            }
            Item::PatternDef(def) => {
                record_duplicate_item_name(module_name, &def.name, &mut seen_items, errors);
                validate_params(&def.name, &def.params, &module_known_types, errors);
                validate_outputs(&def.name, &def.outputs, &module_known_types, errors);
                validate_uses_clauses(
                    &def.name,
                    &def.uses,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                    errors,
                );
                validate_callable_body(
                    &def.name,
                    &def.body.stmts,
                    context.callable_registry,
                    errors,
                );
                signatures.push(TypedItemSignature::Pattern(TypedCallableSignature {
                    name: def.name.clone(),
                    params: def
                        .params
                        .iter()
                        .map(|param| TypedBinding {
                            name: param.name.clone(),
                            ty: type_expr_to_string(&param.ty),
                        })
                        .collect(),
                    outputs: def
                        .outputs
                        .iter()
                        .map(|field| TypedBinding {
                            name: field.name.clone(),
                            ty: type_expr_to_string(&field.ty),
                        })
                        .collect(),
                }));
            }
            Item::ServiceDef(def) => {
                record_duplicate_item_name(module_name, &def.name, &mut seen_items, errors);
                validate_service_interface_conformance(def, context.interface_registry, errors);
                signatures.push(TypedItemSignature::Service {
                    name: def.name.clone(),
                    operations: def.operations.len(),
                });
            }
            Item::ResourceDef(def) => {
                record_duplicate_item_name(module_name, &def.name, &mut seen_items, errors);
                validate_resource_interface_conformance(def, context.interface_registry, errors);
                signatures.push(TypedItemSignature::Resource {
                    name: def.name.clone(),
                    implements: def.implements.clone(),
                });
            }
            Item::InterfaceDef(def) => {
                record_duplicate_item_name(module_name, &def.name, &mut seen_items, errors);
                signatures.push(TypedItemSignature::Interface {
                    name: def.name.clone(),
                    capabilities: def.capabilities.len(),
                });
            }
            Item::PipelineDef(def) => {
                record_duplicate_item_name(module_name, &def.name, &mut seen_items, errors);
                signatures.push(TypedItemSignature::Pipeline {
                    name: def.name.clone(),
                    stages: def.stages.len(),
                });
            }
        }
    }

    signatures
}

fn collect_known_types(modules: &[ResolvedModule]) -> HashSet<String> {
    let mut known = builtin_type_names();
    for module in modules {
        let module_prefix = module.module_path.join(".");
        for item in &module.ast.items {
            if let Item::TypeDef(def) = &item.node {
                known.insert(def.name.clone());
                known.insert(format!("{module_prefix}.{}", def.name));
            }
        }
    }
    known
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CallableContract {
    arity: usize,
    params: HashSet<String>,
}

fn collect_unique_callables(
    modules: &[ResolvedModule],
) -> HashMap<String, Option<CallableContract>> {
    let mut callables = HashMap::<String, Option<CallableContract>>::new();
    for module in modules {
        for item in &module.ast.items {
            let (name, params) = match &item.node {
                Item::FnDef(def) => (&def.name, &def.params),
                Item::FuncDef(def) => (&def.name, &def.params),
                Item::PatternDef(def) => (&def.name, &def.params),
                _ => continue,
            };
            let contract = CallableContract {
                arity: params.len(),
                params: params.iter().map(|param| param.name.clone()).collect(),
            };
            callables
                .entry(name.clone())
                .and_modify(|existing| {
                    if existing.is_some() {
                        *existing = None;
                    }
                })
                .or_insert_with(|| Some(contract));
        }
    }
    callables
}

#[derive(Debug, Clone, Default)]
struct InterfaceRegistry {
    full: HashMap<String, InterfaceContract>,
    short: HashMap<String, Option<InterfaceContract>>,
}

#[derive(Debug, Clone, Default)]
struct ResourceTypeRegistry {
    full: HashSet<String>,
    short: HashMap<String, Option<String>>,
}

fn collect_interfaces(modules: &[ResolvedModule]) -> InterfaceRegistry {
    let mut registry = InterfaceRegistry::default();
    for module in modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::InterfaceDef(interface) = &item.node else {
                continue;
            };
            let mut capabilities = HashMap::<String, CapabilityContract>::new();
            for capability in &interface.capabilities {
                capabilities.insert(
                    capability.name.clone(),
                    CapabilityContract {
                        inputs: field_signature_map(&capability.inputs),
                        outputs: field_signature_map(&capability.outputs),
                    },
                );
            }
            let contract = InterfaceContract { capabilities };
            let full_name = format!("{module_name}.{}", interface.name);
            registry.full.insert(full_name, contract.clone());

            registry
                .short
                .entry(interface.name.clone())
                .and_modify(|existing| {
                    if existing.is_some() {
                        *existing = None;
                    }
                })
                .or_insert_with(|| Some(contract.clone()));
        }
    }
    registry
}

fn collect_resource_types(modules: &[ResolvedModule]) -> ResourceTypeRegistry {
    let mut registry = ResourceTypeRegistry::default();
    for module in modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let name = match &item.node {
                Item::InterfaceDef(interface) => interface.name.as_str(),
                Item::ResourceDef(resource) => resource.name.as_str(),
                _ => continue,
            };
            let full = format!("{module_name}.{name}");
            registry.full.insert(full.clone());
            registry
                .short
                .entry(name.to_string())
                .and_modify(|existing| {
                    if let Some(current) = existing {
                        if current != &full {
                            *existing = None;
                        }
                    }
                })
                .or_insert_with(|| Some(full));
        }
    }
    registry
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InterfaceContract {
    capabilities: HashMap<String, CapabilityContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CapabilityContract {
    inputs: HashMap<String, String>,
    outputs: HashMap<String, String>,
}

fn builtin_type_names() -> HashSet<String> {
    HashSet::from([
        "Any".to_string(),
        "Unit".to_string(),
        "Bool".to_string(),
        "Int".to_string(),
        "Float".to_string(),
        "String".to_string(),
        "Bytes".to_string(),
        "Json".to_string(),
        "Record".to_string(),
        "List".to_string(),
        "Map".to_string(),
        "Option".to_string(),
        "Result".to_string(),
        "Queue".to_string(),
        "Self".to_string(),
    ])
}

fn record_duplicate_item_name(
    module_name: &str,
    item_name: &str,
    seen_items: &mut HashSet<String>,
    errors: &mut Vec<TypeError>,
) {
    if !seen_items.insert(item_name.to_string()) {
        errors.push(TypeError::DuplicateDefinition {
            module: module_name.to_string(),
            name: item_name.to_string(),
        });
    }
}

fn validate_params(
    item_name: &str,
    params: &[Param],
    known_types: &HashSet<String>,
    errors: &mut Vec<TypeError>,
) {
    let mut seen = HashSet::new();
    for param in params {
        if !seen.insert(param.name.clone()) {
            errors.push(TypeError::DuplicateParameter {
                item: item_name.to_string(),
                param: param.name.clone(),
            });
        }
        validate_type_expr(
            &param.ty,
            known_types,
            &format!("{}.{}", item_name, param.name),
            errors,
        );
    }
}

fn validate_outputs(
    item_name: &str,
    outputs: &[Field],
    known_types: &HashSet<String>,
    errors: &mut Vec<TypeError>,
) {
    let mut seen = HashSet::new();
    for output in outputs {
        if !seen.insert(output.name.clone()) {
            errors.push(TypeError::DuplicateOutputField {
                item: item_name.to_string(),
                field: output.name.clone(),
            });
        }
        validate_type_expr(
            &output.ty,
            known_types,
            &format!("{}.{}", item_name, output.name),
            errors,
        );
    }
}

fn validate_uses_clauses(
    item_name: &str,
    uses: &[UsesClause],
    registry: &ResourceTypeRegistry,
    allow_unresolved_references: bool,
    errors: &mut Vec<TypeError>,
) {
    if allow_unresolved_references {
        return;
    }
    for usage in uses {
        let resource_type = canonical_type_name(&type_expr_to_string(&usage.resource_type));
        if resolve_resource_type_name(&resource_type, registry).is_none() {
            errors.push(TypeError::UnknownUsedResourceType {
                item: item_name.to_string(),
                binding: usage.binding.clone(),
                resource_type,
            });
        }
    }
}

fn validate_callable_body(
    caller: &str,
    stmts: &[Stmt],
    callable_registry: &HashMap<String, Option<CallableContract>>,
    errors: &mut Vec<TypeError>,
) {
    let mut calls = Vec::new();
    collect_calls_from_stmts(stmts, &mut calls);
    for call in calls {
        let Some(Some(contract)) = callable_registry.get(&call.callee) else {
            continue;
        };
        if call.arg_count != contract.arity {
            errors.push(TypeError::CallArityMismatch {
                caller: caller.to_string(),
                callee: call.callee.clone(),
                expected: contract.arity,
                got: call.arg_count,
            });
        }
        for named in call.named_args {
            if !contract.params.contains(&named) {
                errors.push(TypeError::UnknownCallArgument {
                    caller: caller.to_string(),
                    callee: call.callee.clone(),
                    argument: named,
                });
            }
        }
    }
}

fn validate_resource_interface_conformance(
    resource: &daglang_syntax::ast::ResourceDef,
    interface_registry: &InterfaceRegistry,
    errors: &mut Vec<TypeError>,
) {
    let Some(implemented) = resource.implements.as_deref() else {
        return;
    };
    let Some(interface_contract) = resolve_interface_contract(implemented, interface_registry)
    else {
        errors.push(TypeError::UnresolvedInterface {
            implementor: resource.name.clone(),
            interface: implemented.to_string(),
        });
        return;
    };
    let provided_capabilities = resource
        .capabilities
        .iter()
        .map(|capability| {
            (
                capability.name.clone(),
                CapabilityContract {
                    inputs: field_signature_map(&capability.inputs),
                    outputs: field_signature_map(&capability.outputs),
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let interface_name = canonical_interface_name(implemented);
    for (capability_name, required_contract) in interface_contract.capabilities {
        let Some(provided_contract) = provided_capabilities.get(&capability_name) else {
            errors.push(TypeError::MissingCapability {
                resource: resource.name.clone(),
                interface: interface_name.clone(),
                capability: capability_name,
            });
            continue;
        };
        validate_capability_contract(
            &resource.name,
            &interface_name,
            &capability_name,
            provided_contract,
            &required_contract,
            errors,
        );
    }
}

fn validate_capability_contract(
    implementor: &str,
    interface: &str,
    capability: &str,
    provided: &CapabilityContract,
    required: &CapabilityContract,
    errors: &mut Vec<TypeError>,
) {
    validate_signature_map(
        implementor,
        interface,
        capability,
        "input",
        &provided.inputs,
        &required.inputs,
        errors,
    );
    validate_signature_map(
        implementor,
        interface,
        capability,
        "output",
        &provided.outputs,
        &required.outputs,
        errors,
    );
}

fn validate_signature_map(
    implementor: &str,
    interface: &str,
    capability: &str,
    direction: &str,
    provided: &HashMap<String, String>,
    required: &HashMap<String, String>,
    errors: &mut Vec<TypeError>,
) {
    for (field, expected_ty) in required {
        let Some(provided_ty) = provided.get(field) else {
            errors.push(TypeError::InterfaceSignatureMismatch {
                implementor: implementor.to_string(),
                interface: interface.to_string(),
                capability: capability.to_string(),
                detail: format!("missing {direction} field `{field}`"),
            });
            continue;
        };
        if provided_ty != expected_ty {
            errors.push(TypeError::InterfaceSignatureMismatch {
                implementor: implementor.to_string(),
                interface: interface.to_string(),
                capability: capability.to_string(),
                detail: format!(
                    "{direction} field `{field}` expected `{expected_ty}` but found `{provided_ty}`"
                ),
            });
        }
    }
}

fn validate_service_interface_conformance(
    service: &daglang_syntax::ast::ServiceDef,
    interface_registry: &InterfaceRegistry,
    errors: &mut Vec<TypeError>,
) {
    let Some(implemented) = service.implements.as_deref() else {
        return;
    };
    let Some(interface_contract) = resolve_interface_contract(implemented, interface_registry)
    else {
        errors.push(TypeError::UnresolvedInterface {
            implementor: service.name.clone(),
            interface: implemented.to_string(),
        });
        return;
    };
    let provided_operations = service
        .operations
        .iter()
        .map(|operation| {
            (
                operation.name.clone(),
                CapabilityContract {
                    inputs: field_signature_map(&operation.inputs),
                    outputs: field_signature_map(&operation.outputs),
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let interface_name = canonical_interface_name(implemented);
    for (capability_name, required_contract) in interface_contract.capabilities {
        let Some(provided_contract) = provided_operations.get(&capability_name) else {
            errors.push(TypeError::MissingOperation {
                service: service.name.clone(),
                interface: interface_name.clone(),
                operation: capability_name,
            });
            continue;
        };
        validate_capability_contract(
            &service.name,
            &interface_name,
            &capability_name,
            provided_contract,
            &required_contract,
            errors,
        );
    }
}

fn field_signature_map(fields: &[Field]) -> HashMap<String, String> {
    fields
        .iter()
        .map(|field| (field.name.clone(), type_expr_to_string(&field.ty)))
        .collect()
}

fn resolve_interface_contract(
    implemented: &str,
    registry: &InterfaceRegistry,
) -> Option<InterfaceContract> {
    let canonical = canonical_type_name(implemented);
    if let Some(contract) = registry.full.get(&canonical) {
        return Some(contract.clone());
    }
    let short = canonical.rsplit('.').next().unwrap_or(canonical.as_str());
    registry.short.get(short).and_then(|entry| entry.clone())
}

fn canonical_interface_name(name: &str) -> String {
    canonical_type_name(name)
}

fn resolve_resource_type_name(
    resource_type: &str,
    registry: &ResourceTypeRegistry,
) -> Option<String> {
    if registry.full.contains(resource_type) {
        return Some(resource_type.to_string());
    }
    let short = resource_type.rsplit('.').next().unwrap_or(resource_type);
    registry.short.get(short).and_then(|entry| entry.clone())
}

fn canonical_type_name(name: &str) -> String {
    name.split('<').next().unwrap_or(name).trim().to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BodyCall {
    callee: String,
    arg_count: usize,
    named_args: Vec<String>,
}

fn collect_calls_from_stmts(stmts: &[Stmt], calls: &mut Vec<BodyCall>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                collect_calls_from_expr(expr, calls);
            }
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    collect_calls_from_expr(expr, calls);
                }
            }
        }
    }
}

fn collect_calls_from_expr(expr: &Expr, calls: &mut Vec<BodyCall>) {
    match expr {
        Expr::Call(name, args) => {
            if should_validate_call_name(name) {
                calls.push(BodyCall {
                    callee: name.clone(),
                    arg_count: args.len(),
                    named_args: args
                        .iter()
                        .filter_map(|(name, _)| name.clone())
                        .collect::<Vec<_>>(),
                });
            }
            for (_, arg) in args {
                collect_calls_from_expr(arg, calls);
            }
        }
        Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                collect_calls_from_expr(arg, calls);
            }
        }
        Expr::FieldAccess(base, _) => collect_calls_from_expr(base, calls),
        Expr::BinOp(lhs, _, rhs) => {
            collect_calls_from_expr(lhs, calls);
            collect_calls_from_expr(rhs, calls);
        }
        Expr::UnaryOp(_, inner) => collect_calls_from_expr(inner, calls),
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_calls_from_expr(inner, calls);
                }
            }
        }
        Expr::Record(_, fields) => {
            for (_, value) in fields {
                collect_calls_from_expr(value, calls);
            }
        }
        Expr::Match(scrutinee, arms) => {
            collect_calls_from_expr(scrutinee, calls);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_calls_from_expr(guard, calls);
                }
                collect_calls_from_expr(&arm.body, calls);
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_calls_from_expr(cond, calls);
            collect_calls_from_expr(then_expr, calls);
            if let Some(otherwise) = else_expr {
                collect_calls_from_expr(otherwise, calls);
            }
        }
        Expr::For(_, iterable, body) => {
            collect_calls_from_expr(iterable, calls);
            collect_calls_from_expr(body, calls);
        }
        Expr::Pipe(lhs, rhs) => {
            collect_calls_from_expr(lhs, calls);
            collect_calls_from_expr(rhs, calls);
        }
        Expr::Lambda(_, body) => collect_calls_from_expr(body, calls),
        Expr::List(items) => {
            for item in items {
                collect_calls_from_expr(item, calls);
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                collect_calls_from_expr(key, calls);
                collect_calls_from_expr(value, calls);
            }
        }
        Expr::Guarded(inner, guard) => {
            collect_calls_from_expr(inner, calls);
            collect_calls_from_expr(guard, calls);
        }
        Expr::After(inner, _) => collect_calls_from_expr(inner, calls),
        Expr::Return(fields) => {
            for (_, value) in fields {
                collect_calls_from_expr(value, calls);
            }
        }
        Expr::Literal(_) | Expr::Ident(_) => {}
    }
}

fn should_validate_call_name(name: &str) -> bool {
    !matches!(name, "<expr>" | "as" | "with" | "fn")
}

fn validate_type_expr(
    ty: &TypeExpr,
    known_types: &HashSet<String>,
    context: &str,
    errors: &mut Vec<TypeError>,
) {
    match ty {
        TypeExpr::Named(name) => {
            if should_validate_named_type(name) && !known_types.contains(name) {
                let tail = name.rsplit('.').next().unwrap_or(name);
                if !known_types.contains(tail) {
                    errors.push(TypeError::UndefinedType(format!("{name} (in {context})")));
                }
            }
        }
        TypeExpr::Generic(name, args) => {
            if should_validate_named_type(name) && !known_types.contains(name) {
                let tail = name.rsplit('.').next().unwrap_or(name);
                if !known_types.contains(tail) {
                    errors.push(TypeError::UndefinedType(format!("{name} (in {context})")));
                }
            }
            for arg in args {
                validate_type_expr(arg, known_types, context, errors);
            }
        }
        TypeExpr::Optional(inner) => validate_type_expr(inner, known_types, context, errors),
        TypeExpr::Annotated(inner, _) => validate_type_expr(inner, known_types, context, errors),
    }
}

fn should_validate_named_type(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '.'))
}

fn type_expr_to_string(expr: &TypeExpr) -> String {
    match expr {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::Generic(name, args) => format!(
            "{name}<{}>",
            args.iter()
                .map(type_expr_to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpr::Optional(inner) => format!("{}?", type_expr_to_string(inner)),
        TypeExpr::Annotated(inner, annotations) => format!(
            "{} {}",
            type_expr_to_string(inner),
            annotations
                .iter()
                .map(|annotation| format!("@{}", annotation.name))
                .collect::<Vec<_>>()
                .join(" ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn module_graph_from_sources(sources: &[(&str, &str)]) -> ModuleGraph {
        let modules = sources
            .iter()
            .map(|(path, source)| {
                let ast = daglang_syntax::parser::parse(source).expect("source should parse");
                let module_path = ast
                    .module_path
                    .as_ref()
                    .map(|module| module.node.segments.clone())
                    .expect("module declarations are required in tests");
                ResolvedModule {
                    path: PathBuf::from(path),
                    ast,
                    module_path,
                    dependencies: Vec::new(),
                }
            })
            .collect();
        ModuleGraph { modules }
    }

    #[test]
    fn typecheck_accepts_makegen_module() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../dsl/tools/makegen.dag");
        let source = fs::read_to_string(file).expect("should read makegen source");
        let graph = module_graph_from_sources(&[("dsl/tools/makegen.dag", &source)]);
        let typed = typecheck_module_graph(graph).expect("makegen should typecheck");

        assert_eq!(typed.modules.len(), 1);
        assert_eq!(typed.modules[0].module_path.join("."), "tools.makegen");
        assert!(typed.modules[0]
            .signatures
            .iter()
            .any(|signature| matches!(signature, TypedItemSignature::Fn(_))));
        assert!(typed.modules[0]
            .signatures
            .iter()
            .any(|signature| matches!(signature, TypedItemSignature::Func(_))));
    }

    #[test]
    fn duplicate_param_names_are_reported() {
        let graph = module_graph_from_sources(&[(
            "dup_params.dag",
            "module sample.dup\nfn bad(a: String, a: Int) -> String { a }",
        )]);
        let errors = typecheck_module_graph(graph).expect_err("duplicate params should fail");
        assert!(errors
            .iter()
            .any(|error| matches!(error, TypeError::DuplicateParameter { item, param } if item == "bad" && param == "a")));
    }

    #[test]
    fn undefined_types_are_reported() {
        let graph = module_graph_from_sources(&[(
            "unknown_type.dag",
            "module sample.unknown\nfn run(input: MissingType) -> String { \"ok\" }",
        )]);
        let errors = typecheck_module_graph(graph).expect_err("unknown type should fail");
        assert!(errors
            .iter()
            .any(|error| matches!(error, TypeError::UndefinedType(msg) if msg.contains("MissingType"))));
    }

    #[test]
    fn strict_mode_reports_unresolved_imports() {
        let graph = module_graph_from_sources(&[(
            "missing_import.dag",
            "module sample.main\nimport missing.dep\nfn run() -> Unit {}",
        )]);
        let options = TypecheckOptions {
            allow_unresolved_imports: false,
        };
        let errors = typecheck_module_graph_with_options(graph, options)
            .expect_err("strict mode should fail on unresolved import");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnresolvedImport { module, target }
                if module == "sample.main" && target == "missing.dep"
        )));
    }

    #[test]
    fn call_arity_mismatch_is_reported() {
        let graph = module_graph_from_sources(&[(
            "arity_mismatch.dag",
            "module sample.calls\nfn fmt(value: String) -> String { value }\nfn run() -> String { fmt() }",
        )]);
        let errors = typecheck_module_graph(graph).expect_err("call arity mismatch should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::CallArityMismatch {
                caller,
                callee,
                expected,
                got
            } if caller == "run" && callee == "fmt" && *expected == 1 && *got == 0
        )));
    }

    #[test]
    fn unknown_named_call_argument_is_reported() {
        let graph = module_graph_from_sources(&[(
            "unknown_arg.dag",
            "module sample.calls\nfn fmt(value: String) -> String { value }\nfn run() -> String { fmt(text: \"ok\") }",
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("unknown named argument should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnknownCallArgument {
                caller,
                callee,
                argument
            } if caller == "run" && callee == "fmt" && argument == "text"
        )));
    }

    #[test]
    fn resource_missing_interface_capability_is_reported() {
        let graph = module_graph_from_sources(&[(
            "missing_capability.dag",
            r#"module sample.resources
interface ObjectStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
resource Disk implements ObjectStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}"#,
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("missing interface capability should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::MissingCapability {
                resource,
                interface,
                capability
            } if resource == "Disk" && interface == "ObjectStorage" && capability == "write"
        )));
    }

    #[test]
    fn unresolved_interface_on_resource_is_reported() {
        let graph = module_graph_from_sources(&[(
            "missing_interface.dag",
            "module sample.resources\nresource Disk implements MissingStorage {}",
        )]);
        let errors = typecheck_module_graph(graph).expect_err("unknown interface should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnresolvedInterface { implementor, interface }
                if implementor == "Disk" && interface == "MissingStorage"
        )));
    }

    #[test]
    fn service_missing_interface_operation_is_reported() {
        let graph = module_graph_from_sources(&[(
            "missing_operation.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
  capability write {
    input { path: String, body: String }
    output { ok: Bool }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("missing operation should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::MissingOperation {
                service,
                interface,
                operation
            } if service == "FsStorage" && interface == "Storage" && operation == "write"
        )));
    }

    #[test]
    fn unresolved_interface_on_service_is_reported() {
        let graph = module_graph_from_sources(&[(
            "missing_service_interface.dag",
            "module sample.services\nservice FsStorage implements MissingStorage { operation read(path: String) -> { body: String } }",
        )]);
        let errors = typecheck_module_graph(graph).expect_err("unknown interface should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnresolvedInterface { implementor, interface }
                if implementor == "FsStorage" && interface == "MissingStorage"
        )));
    }

    #[test]
    fn resource_capability_signature_mismatch_is_reported() {
        let graph = module_graph_from_sources(&[(
            "resource_sig_mismatch.dag",
            r#"module sample.resources
interface ObjectStorage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
resource Disk implements ObjectStorage {
  capability read {
    input { path: Int }
    output { body: String }
  }
}"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("signature mismatch should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::InterfaceSignatureMismatch {
                implementor,
                interface,
                capability,
                detail,
            } if implementor == "Disk"
                && interface == "ObjectStorage"
                && capability == "read"
                && detail.contains("input field `path` expected `String` but found `Int`")
        )));
    }

    #[test]
    fn service_operation_signature_mismatch_is_reported() {
        let graph = module_graph_from_sources(&[(
            "service_sig_mismatch.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: Int }
}"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("signature mismatch should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::InterfaceSignatureMismatch {
                implementor,
                interface,
                capability,
                detail,
            } if implementor == "FsStorage"
                && interface == "Storage"
                && capability == "read"
                && detail.contains("output field `body` expected `String` but found `Int`")
        )));
    }

    #[test]
    fn strict_mode_reports_unknown_used_resource_type() {
        let graph = module_graph_from_sources(&[(
            "unknown_uses.dag",
            "module sample.uses\nfunc run() -> { ok: Bool } uses fs: MissingResource { return { ok: true } }",
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for unknown used resource type");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnknownUsedResourceType {
                item,
                binding,
                resource_type,
            } if item == "run" && binding == "fs" && resource_type == "MissingResource"
        )));
    }

    #[test]
    fn relaxed_mode_allows_unknown_used_resource_type() {
        let graph = module_graph_from_sources(&[(
            "unknown_uses_relaxed.dag",
            "module sample.uses\nfunc run() -> { ok: Bool } uses fs: MissingResource { return { ok: true } }",
        )]);
        let typed = typecheck_module_graph(graph).expect("relaxed mode should allow unknown uses");
        assert_eq!(typed.modules.len(), 1);
    }
}
