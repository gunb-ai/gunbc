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
use daglang_syntax::ast::{
    Expr, Field, Item, Param, ProvidesClause, SourceFile, Stmt, TypeExpr, UsesClause,
};

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
    /// Resource/service declares an interface that resolves ambiguously.
    AmbiguousInterface {
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
    /// Call expression reuses the same named argument multiple times.
    DuplicateCallArgument {
        caller: String,
        callee: String,
        argument: String,
    },
    /// Call expression target resolves to multiple callable contracts.
    AmbiguousCallTarget {
        caller: String,
        callee: String,
    },
    /// Call expression target cannot be resolved to a callable contract.
    UnresolvedCallTarget {
        caller: String,
        callee: String,
    },
    /// Service call expression used wrong number of arguments.
    ServiceCallArityMismatch {
        caller: String,
        service_call: String,
        expected: usize,
        got: usize,
    },
    /// Service call target could not be resolved to a known service operation contract.
    UnresolvedServiceCall { caller: String, service_call: String },
    /// Service call target matches multiple possible service operation contracts.
    AmbiguousServiceCall { caller: String, service_call: String },
    /// Service call expression used an unknown named argument.
    UnknownServiceCallArgument {
        caller: String,
        service_call: String,
        argument: String,
    },
    /// Service call expression reuses the same named argument multiple times.
    DuplicateServiceCallArgument {
        caller: String,
        service_call: String,
        argument: String,
    },
    /// `uses` clause references an unknown resource/interface type.
    UnknownUsedResourceType {
        item: String,
        binding: String,
        resource_type: String,
    },
    /// `uses` clause references an ambiguous short resource/interface type.
    AmbiguousUsedResourceType {
        item: String,
        binding: String,
        resource_type: String,
    },
    /// Duplicate `uses` binding within a callable declaration.
    DuplicateUsesBinding { item: String, binding: String },
    /// Duplicate `provides` binding within a callable declaration.
    DuplicateProvidesBinding { item: String, binding: String },
    /// A binding is declared in both `uses` and `provides`.
    UseProvideBindingConflict { item: String, binding: String },
    /// `provides` clause references an unknown resource/interface type.
    UnknownProvidedResourceType {
        item: String,
        binding: String,
        resource_type: String,
    },
    /// `provides` clause references an ambiguous short resource/interface type.
    AmbiguousProvidedResourceType {
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
            Self::AmbiguousInterface {
                implementor,
                interface,
            } => write!(
                f,
                "`{implementor}` references ambiguous interface `{interface}`"
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
            Self::DuplicateCallArgument {
                caller,
                callee,
                argument,
            } => write!(
                f,
                "duplicate named argument `{argument}` in call to `{callee}` within `{caller}`"
            ),
            Self::AmbiguousCallTarget { caller, callee } => write!(
                f,
                "ambiguous call target `{callee}` in `{caller}`"
            ),
            Self::UnresolvedCallTarget { caller, callee } => write!(
                f,
                "unresolved call target `{callee}` in `{caller}`"
            ),
            Self::ServiceCallArityMismatch {
                caller,
                service_call,
                expected,
                got,
            } => write!(
                f,
                "service call arity mismatch in `{caller}` for `{service_call}`: expected {expected}, got {got}"
            ),
            Self::UnresolvedServiceCall {
                caller,
                service_call,
            } => write!(
                f,
                "unresolved service call `{service_call}` in `{caller}`"
            ),
            Self::AmbiguousServiceCall {
                caller,
                service_call,
            } => write!(
                f,
                "ambiguous service call `{service_call}` in `{caller}`"
            ),
            Self::UnknownServiceCallArgument {
                caller,
                service_call,
                argument,
            } => write!(
                f,
                "unknown named argument `{argument}` in service call `{service_call}` within `{caller}`"
            ),
            Self::DuplicateServiceCallArgument {
                caller,
                service_call,
                argument,
            } => write!(
                f,
                "duplicate named argument `{argument}` in service call `{service_call}` within `{caller}`"
            ),
            Self::UnknownUsedResourceType {
                item,
                binding,
                resource_type,
            } => write!(
                f,
                "unknown used resource type `{resource_type}` for binding `{binding}` in `{item}`"
            ),
            Self::AmbiguousUsedResourceType {
                item,
                binding,
                resource_type,
            } => write!(
                f,
                "ambiguous used resource type `{resource_type}` for binding `{binding}` in `{item}`"
            ),
            Self::DuplicateUsesBinding { item, binding } => write!(
                f,
                "duplicate uses binding `{binding}` in `{item}`"
            ),
            Self::DuplicateProvidesBinding { item, binding } => write!(
                f,
                "duplicate provides binding `{binding}` in `{item}`"
            ),
            Self::UseProvideBindingConflict { item, binding } => write!(
                f,
                "binding `{binding}` is declared in both uses/provides in `{item}`"
            ),
            Self::UnknownProvidedResourceType {
                item,
                binding,
                resource_type,
            } => write!(
                f,
                "unknown provided resource type `{resource_type}` for binding `{binding}` in `{item}`"
            ),
            Self::AmbiguousProvidedResourceType {
                item,
                binding,
                resource_type,
            } => write!(
                f,
                "ambiguous provided resource type `{resource_type}` for binding `{binding}` in `{item}`"
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
    let generic_arity_registry = collect_generic_arities(&graph.modules);
    let record_type_registry = collect_record_types(&graph.modules);
    let callable_registry = collect_unique_callables(&graph.modules);
    let service_call_registry = collect_service_call_contracts(&graph.modules);
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
        generic_arity_registry: &generic_arity_registry,
        record_type_registry: &record_type_registry,
        callable_registry: &callable_registry,
        service_call_registry: &service_call_registry,
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
    generic_arity_registry: &'a GenericArityRegistry,
    record_type_registry: &'a RecordTypeRegistry,
    callable_registry: &'a HashMap<String, Option<CallableContract>>,
    service_call_registry: &'a ServiceCallRegistry,
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
    let body_context = BodyInferenceContext {
        record_type_registry: context.record_type_registry,
        callable_registry: context.callable_registry,
        service_call_registry: context.service_call_registry,
    };

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
                validate_params(
                    &def.name,
                    &def.params,
                    &module_known_types,
                    context.generic_arity_registry,
                    errors,
                );
                validate_type_expr(
                    &def.return_type,
                    &module_known_types,
                    context.generic_arity_registry,
                    &format!("{}.return", def.name),
                    errors,
                );
                let outputs = vec![TypedBinding {
                    name: "return".to_string(),
                    ty: type_expr_to_string(&def.return_type),
                }];
                validate_callable_body(
                    &def.name,
                    &def.params,
                    ReturnContract::single(type_expr_to_string(&def.return_type)),
                    &def.body.stmts,
                    &body_context,
                    context.allow_unresolved_references,
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
                validate_params(
                    &def.name,
                    &def.params,
                    &module_known_types,
                    context.generic_arity_registry,
                    errors,
                );
                validate_outputs(
                    &def.name,
                    &def.outputs,
                    &module_known_types,
                    context.generic_arity_registry,
                    errors,
                );
                validate_uses_clauses(
                    &def.name,
                    &def.uses,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                    errors,
                );
                validate_provides_clauses(
                    &def.name,
                    &def.provides,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                    errors,
                );
                validate_use_provide_binding_conflicts(&def.name, &def.uses, &def.provides, errors);
                validate_callable_body(
                    &def.name,
                    &def.params,
                    ReturnContract::record(field_signature_map(&def.outputs)),
                    &def.body.stmts,
                    &body_context,
                    context.allow_unresolved_references,
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
                validate_params(
                    &def.name,
                    &def.params,
                    &module_known_types,
                    context.generic_arity_registry,
                    errors,
                );
                validate_outputs(
                    &def.name,
                    &def.outputs,
                    &module_known_types,
                    context.generic_arity_registry,
                    errors,
                );
                validate_uses_clauses(
                    &def.name,
                    &def.uses,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                    errors,
                );
                validate_callable_body(
                    &def.name,
                    &def.params,
                    ReturnContract::record(field_signature_map(&def.outputs)),
                    &def.body.stmts,
                    &body_context,
                    context.allow_unresolved_references,
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

fn collect_generic_arities(modules: &[ResolvedModule]) -> GenericArityRegistry {
    let mut registry = GenericArityRegistry::default();
    for (name, arity) in builtin_type_arities() {
        registry.full.insert(name.clone(), arity);
        registry.short.insert(name, Some(arity));
    }

    for module in modules {
        let module_prefix = module.module_path.join(".");
        for item in &module.ast.items {
            let (name, arity) = match &item.node {
                Item::TypeDef(def) => (&def.name, def.params.len()),
                Item::InterfaceDef(def) => (&def.name, def.type_params.len()),
                _ => continue,
            };
            let full_name = format!("{module_prefix}.{name}");
            registry.full.insert(full_name.clone(), arity);
            registry
                .short
                .entry(name.clone())
                .and_modify(|existing| {
                    if let Some(current) = existing {
                        if *current != arity {
                            *existing = None;
                        }
                    }
                })
                .or_insert(Some(arity));
        }
    }
    registry
}

fn collect_record_types(modules: &[ResolvedModule]) -> RecordTypeRegistry {
    let mut registry = RecordTypeRegistry::default();
    for module in modules {
        let module_prefix = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::TypeDef(def) = &item.node else {
                continue;
            };
            let daglang_syntax::ast::TypeBody::Record(fields) = &def.body else {
                continue;
            };
            let signature = field_signature_map(fields);
            let full_name = format!("{module_prefix}.{}", def.name);
            registry.full.insert(full_name.clone(), signature);
            registry
                .short
                .entry(def.name.clone())
                .and_modify(|existing| {
                    if let Some(current) = existing {
                        if current != &full_name {
                            *existing = None;
                        }
                    }
                })
                .or_insert(Some(full_name));
        }
    }
    registry
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CallableContract {
    arity: usize,
    params: HashSet<String>,
    output: ValueType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceCallContract {
    arity: usize,
    params: HashSet<String>,
    outputs: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
struct ServiceCallRegistry {
    by_key: HashMap<String, Option<ServiceCallContract>>,
}

#[derive(Debug, Clone)]
enum ServiceCallResolution {
    Resolved(ServiceCallContract),
    Ambiguous,
    Missing,
}

#[derive(Debug, Clone)]
enum InterfaceResolution {
    Resolved(InterfaceContract),
    Ambiguous,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceTypeResolution {
    Resolved(String),
    Ambiguous,
    Missing,
}

fn collect_unique_callables(
    modules: &[ResolvedModule],
) -> HashMap<String, Option<CallableContract>> {
    let mut callables = HashMap::<String, Option<CallableContract>>::new();
    for module in modules {
        for item in &module.ast.items {
            let (name, params, output) = match &item.node {
                Item::FnDef(def) => (
                    &def.name,
                    &def.params,
                    ValueType::Named(type_expr_to_string(&def.return_type)),
                ),
                Item::FuncDef(def) => (&def.name, &def.params, ValueType::Record(field_signature_map(&def.outputs))),
                Item::PatternDef(def) => {
                    (&def.name, &def.params, ValueType::Record(field_signature_map(&def.outputs)))
                }
                _ => continue,
            };
            let contract = CallableContract {
                arity: params.len(),
                params: params.iter().map(|param| param.name.clone()).collect(),
                output,
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

fn collect_service_call_contracts(modules: &[ResolvedModule]) -> ServiceCallRegistry {
    let mut registry = ServiceCallRegistry::default();
    for module in modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::ServiceDef(service) = &item.node else {
                continue;
            };
            for operation in &service.operations {
                let contract = ServiceCallContract {
                    arity: operation.inputs.len(),
                    params: operation
                        .inputs
                        .iter()
                        .map(|field| field.name.clone())
                        .collect(),
                    outputs: field_signature_map(&operation.outputs),
                };
                let service_tail = service
                    .name
                    .rsplit('.')
                    .next()
                    .unwrap_or(service.name.as_str());
                let mut keys = HashSet::new();
                keys.insert(format!("{}.{}", service.name, operation.name));
                keys.insert(format!("{service_tail}.{}", operation.name));
                keys.insert(format!("{}.{}.{}", module_name, service.name, operation.name));
                for key in keys {
                    register_service_call_contract(&mut registry, key, contract.clone());
                }
            }
        }
    }
    registry
}

fn register_service_call_contract(
    registry: &mut ServiceCallRegistry,
    key: String,
    contract: ServiceCallContract,
) {
    registry
        .by_key
        .entry(key)
        .and_modify(|existing| *existing = None)
        .or_insert_with(|| Some(contract));
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

#[derive(Debug, Clone, Default)]
struct GenericArityRegistry {
    full: HashMap<String, usize>,
    short: HashMap<String, Option<usize>>,
}

#[derive(Debug, Clone, Default)]
struct RecordTypeRegistry {
    full: HashMap<String, HashMap<String, String>>,
    short: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ValueType {
    Named(String),
    Record(HashMap<String, String>),
    Unknown,
}

#[derive(Debug, Clone)]
enum ReturnContract {
    Single { ty: String },
    Record { fields: HashMap<String, String> },
}

impl ReturnContract {
    fn single(ty: String) -> Self {
        Self::Single { ty }
    }

    fn record(fields: HashMap<String, String>) -> Self {
        Self::Record { fields }
    }
}

#[derive(Clone, Copy)]
struct BodyInferenceContext<'a> {
    record_type_registry: &'a RecordTypeRegistry,
    callable_registry: &'a HashMap<String, Option<CallableContract>>,
    service_call_registry: &'a ServiceCallRegistry,
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

fn builtin_type_arities() -> HashMap<String, usize> {
    HashMap::from([
        ("Any".to_string(), 0),
        ("Unit".to_string(), 0),
        ("Bool".to_string(), 0),
        ("Int".to_string(), 0),
        ("Float".to_string(), 0),
        ("String".to_string(), 0),
        ("Bytes".to_string(), 0),
        ("Json".to_string(), 0),
        ("Record".to_string(), 0),
        ("List".to_string(), 1),
        ("Map".to_string(), 2),
        ("Option".to_string(), 1),
        ("Result".to_string(), 2),
        ("Queue".to_string(), 1),
        ("Self".to_string(), 0),
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
    generic_arity_registry: &GenericArityRegistry,
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
            generic_arity_registry,
            &format!("{}.{}", item_name, param.name),
            errors,
        );
    }
}

fn validate_outputs(
    item_name: &str,
    outputs: &[Field],
    known_types: &HashSet<String>,
    generic_arity_registry: &GenericArityRegistry,
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
            generic_arity_registry,
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
    let mut seen_bindings = HashSet::new();
    for usage in uses {
        if !seen_bindings.insert(usage.binding.clone()) {
            errors.push(TypeError::DuplicateUsesBinding {
                item: item_name.to_string(),
                binding: usage.binding.clone(),
            });
        }
        if !allow_unresolved_references {
            let resource_type = canonical_type_name(&type_expr_to_string(&usage.resource_type));
            match resolve_resource_type_name(&resource_type, registry) {
                ResourceTypeResolution::Resolved(_) => {}
                ResourceTypeResolution::Ambiguous => {
                    errors.push(TypeError::AmbiguousUsedResourceType {
                        item: item_name.to_string(),
                        binding: usage.binding.clone(),
                        resource_type,
                    });
                }
                ResourceTypeResolution::Missing => {
                    errors.push(TypeError::UnknownUsedResourceType {
                        item: item_name.to_string(),
                        binding: usage.binding.clone(),
                        resource_type,
                    });
                }
            }
        }
    }
}

fn validate_provides_clauses(
    item_name: &str,
    provides: &[ProvidesClause],
    registry: &ResourceTypeRegistry,
    allow_unresolved_references: bool,
    errors: &mut Vec<TypeError>,
) {
    let mut seen_bindings = HashSet::new();
    for provided in provides {
        if !seen_bindings.insert(provided.binding.clone()) {
            errors.push(TypeError::DuplicateProvidesBinding {
                item: item_name.to_string(),
                binding: provided.binding.clone(),
            });
        }
        if !allow_unresolved_references {
            let resource_type = canonical_type_name(&type_expr_to_string(&provided.resource_type));
            match resolve_resource_type_name(&resource_type, registry) {
                ResourceTypeResolution::Resolved(_) => {}
                ResourceTypeResolution::Ambiguous => {
                    errors.push(TypeError::AmbiguousProvidedResourceType {
                        item: item_name.to_string(),
                        binding: provided.binding.clone(),
                        resource_type,
                    });
                }
                ResourceTypeResolution::Missing => {
                    errors.push(TypeError::UnknownProvidedResourceType {
                        item: item_name.to_string(),
                        binding: provided.binding.clone(),
                        resource_type,
                    });
                }
            }
        }
    }
}

fn validate_use_provide_binding_conflicts(
    item_name: &str,
    uses: &[UsesClause],
    provides: &[ProvidesClause],
    errors: &mut Vec<TypeError>,
) {
    let used_bindings = uses
        .iter()
        .map(|usage| usage.binding.as_str())
        .collect::<HashSet<_>>();
    for provided in provides {
        if used_bindings.contains(provided.binding.as_str()) {
            errors.push(TypeError::UseProvideBindingConflict {
                item: item_name.to_string(),
                binding: provided.binding.clone(),
            });
        }
    }
}

fn validate_callable_body(
    caller: &str,
    params: &[Param],
    return_contract: ReturnContract,
    stmts: &[Stmt],
    body_context: &BodyInferenceContext<'_>,
    allow_unresolved_references: bool,
    errors: &mut Vec<TypeError>,
) {
    let mut calls = Vec::new();
    collect_calls_from_stmts(stmts, &mut calls);
    for call in calls {
        let contract = match body_context.callable_registry.get(&call.callee) {
            Some(Some(contract)) => contract,
            Some(None) => {
                if !allow_unresolved_references {
                    errors.push(TypeError::AmbiguousCallTarget {
                        caller: caller.to_string(),
                        callee: call.callee.clone(),
                    });
                }
                continue;
            }
            None => {
                if !allow_unresolved_references {
                    errors.push(TypeError::UnresolvedCallTarget {
                        caller: caller.to_string(),
                        callee: call.callee.clone(),
                    });
                }
                continue;
            }
        };
        if call.arg_count != contract.arity {
            errors.push(TypeError::CallArityMismatch {
                caller: caller.to_string(),
                callee: call.callee.clone(),
                expected: contract.arity,
                got: call.arg_count,
            });
        }
        let mut seen_named = HashSet::new();
        for named in call.named_args {
            if !seen_named.insert(named.clone()) {
                errors.push(TypeError::DuplicateCallArgument {
                    caller: caller.to_string(),
                    callee: call.callee.clone(),
                    argument: named,
                });
                continue;
            }
            if !contract.params.contains(&named) {
                errors.push(TypeError::UnknownCallArgument {
                    caller: caller.to_string(),
                    callee: call.callee.clone(),
                    argument: named,
                });
            }
        }
    }

    let mut service_calls = Vec::new();
    collect_service_calls_from_stmts(stmts, &mut service_calls);
    for call in service_calls {
        let service_call_name = call.path.join(".");
        let contract =
            match resolve_service_call_contract(&call.path, body_context.service_call_registry) {
            ServiceCallResolution::Resolved(contract) => contract,
            ServiceCallResolution::Ambiguous => {
                if !allow_unresolved_references {
                    errors.push(TypeError::AmbiguousServiceCall {
                        caller: caller.to_string(),
                        service_call: service_call_name,
                    });
                }
                continue;
            }
            ServiceCallResolution::Missing => {
                if !allow_unresolved_references {
                    errors.push(TypeError::UnresolvedServiceCall {
                        caller: caller.to_string(),
                        service_call: service_call_name,
                    });
                }
                continue;
            }
        };
        if call.arg_count != contract.arity {
            errors.push(TypeError::ServiceCallArityMismatch {
                caller: caller.to_string(),
                service_call: service_call_name.clone(),
                expected: contract.arity,
                got: call.arg_count,
            });
        }
        let mut seen_named = HashSet::new();
        for named in call.named_args {
            if !seen_named.insert(named.clone()) {
                errors.push(TypeError::DuplicateServiceCallArgument {
                    caller: caller.to_string(),
                    service_call: service_call_name.clone(),
                    argument: named,
                });
                continue;
            }
            if !contract.params.contains(&named) {
                errors.push(TypeError::UnknownServiceCallArgument {
                    caller: caller.to_string(),
                    service_call: service_call_name.clone(),
                    argument: named,
                });
            }
        }
    }

    let mut local_bindings = params
        .iter()
        .map(|param| {
            (
                param.name.clone(),
                ValueType::Named(type_expr_to_string(&param.ty)),
            )
        })
        .collect::<HashMap<_, _>>();
    for stmt in stmts {
        match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                let inferred = infer_expr_type(
                    expr,
                    &local_bindings,
                    body_context,
                    errors,
                );
                local_bindings.insert(name.clone(), inferred);
            }
            Stmt::Expr(expr) => {
                infer_expr_type(
                    expr,
                    &local_bindings,
                    body_context,
                    errors,
                );
            }
            Stmt::Return(fields) => {
                validate_return_stmt(
                    caller,
                    &return_contract,
                    fields,
                    &local_bindings,
                    body_context,
                    errors,
                );
            }
        }
    }
}

fn validate_return_stmt(
    caller: &str,
    return_contract: &ReturnContract,
    fields: &[(String, Expr)],
    local_bindings: &HashMap<String, ValueType>,
    body_context: &BodyInferenceContext<'_>,
    errors: &mut Vec<TypeError>,
) {
    match return_contract {
        ReturnContract::Single { ty } => {
            if fields.len() != 1 {
                errors.push(TypeError::TypeMismatch {
                    expected: ty.clone(),
                    got: "Record".to_string(),
                });
                return;
            }
            let inferred = infer_expr_type(
                &fields[0].1,
                local_bindings,
                body_context,
                errors,
            );
            push_type_mismatch_if_needed(ty, &inferred, errors);
        }
        ReturnContract::Record { fields: expected } => {
            for (field, expr) in fields {
                let Some(expected_ty) = expected.get(field) else {
                    errors.push(TypeError::NoSuchField {
                        ty: format!("{caller}.outputs"),
                        field: field.clone(),
                    });
                    continue;
                };
                let inferred = infer_expr_type(
                    expr,
                    local_bindings,
                    body_context,
                    errors,
                );
                push_type_mismatch_if_needed(expected_ty, &inferred, errors);
            }
        }
    }
}

fn infer_expr_type(
    expr: &Expr,
    local_bindings: &HashMap<String, ValueType>,
    body_context: &BodyInferenceContext<'_>,
    errors: &mut Vec<TypeError>,
) -> ValueType {
    match expr {
        Expr::Literal(literal) => match literal {
            daglang_syntax::ast::Literal::Int(_) => ValueType::Named("Int".to_string()),
            daglang_syntax::ast::Literal::Float(_) => ValueType::Named("Float".to_string()),
            daglang_syntax::ast::Literal::String(_) => ValueType::Named("String".to_string()),
            daglang_syntax::ast::Literal::Bool(_) => ValueType::Named("Bool".to_string()),
            daglang_syntax::ast::Literal::None => ValueType::Named("Unit".to_string()),
        },
        Expr::Ident(name) => local_bindings.get(name).cloned().unwrap_or(ValueType::Unknown),
        Expr::FieldAccess(base, field) => {
            let base_type = infer_expr_type(base, local_bindings, body_context, errors);
            match base_type {
                ValueType::Record(fields) => match fields.get(field) {
                    Some(ty) => ValueType::Named(ty.clone()),
                    None => {
                        errors.push(TypeError::NoSuchField {
                            ty: "Record".to_string(),
                            field: field.clone(),
                        });
                        ValueType::Unknown
                    }
                },
                ValueType::Named(name) => match resolve_record_fields(
                    &name,
                    body_context.record_type_registry,
                ) {
                    Some(fields) => match fields.get(field) {
                        Some(ty) => ValueType::Named(ty.clone()),
                        None => {
                            errors.push(TypeError::NoSuchField {
                                ty: name,
                                field: field.clone(),
                            });
                            ValueType::Unknown
                        }
                    },
                    None => ValueType::Unknown,
                },
                ValueType::Unknown => ValueType::Unknown,
            }
        }
        Expr::Call(name, args) => {
            for (_, arg) in args {
                infer_expr_type(arg, local_bindings, body_context, errors);
            }
            body_context
                .callable_registry
                .get(name)
                .and_then(|entry| entry.as_ref())
                .map(|contract| contract.output.clone())
                .unwrap_or(ValueType::Unknown)
        }
        Expr::ServiceCall(path, args) => {
            for (_, arg) in args {
                infer_expr_type(arg, local_bindings, body_context, errors);
            }
            match resolve_service_call_contract(path, body_context.service_call_registry) {
                ServiceCallResolution::Resolved(contract) => ValueType::Record(contract.outputs),
                ServiceCallResolution::Ambiguous | ServiceCallResolution::Missing => {
                    ValueType::Unknown
                }
            }
        }
        Expr::BinOp(lhs, op, rhs) => {
            let lhs_ty = infer_expr_type(lhs, local_bindings, body_context, errors);
            let rhs_ty = infer_expr_type(rhs, local_bindings, body_context, errors);
            match op {
                daglang_syntax::ast::BinOp::Eq
                | daglang_syntax::ast::BinOp::Ne
                | daglang_syntax::ast::BinOp::Lt
                | daglang_syntax::ast::BinOp::Gt
                | daglang_syntax::ast::BinOp::Le
                | daglang_syntax::ast::BinOp::Ge
                | daglang_syntax::ast::BinOp::And
                | daglang_syntax::ast::BinOp::Or => ValueType::Named("Bool".to_string()),
                _ => match (lhs_ty, rhs_ty) {
                    (ValueType::Named(lhs), ValueType::Named(rhs))
                        if canonical_type_name(&lhs) == canonical_type_name(&rhs) =>
                    {
                        ValueType::Named(lhs)
                    }
                    _ => ValueType::Unknown,
                },
            }
        }
        Expr::UnaryOp(op, inner) => {
            let inner_ty = infer_expr_type(inner, local_bindings, body_context, errors);
            match op {
                daglang_syntax::ast::UnaryOp::Not => ValueType::Named("Bool".to_string()),
                daglang_syntax::ast::UnaryOp::Neg => inner_ty,
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    infer_expr_type(inner, local_bindings, body_context, errors);
                }
            }
            ValueType::Named("String".to_string())
        }
        Expr::Record(type_name, fields) => {
            for (_, value) in fields {
                infer_expr_type(value, local_bindings, body_context, errors);
            }
            if let Some(name) = type_name {
                ValueType::Named(name.clone())
            } else {
                ValueType::Record(
                    fields
                        .iter()
                        .map(|(name, expr)| {
                            (
                                name.clone(),
                                infer_expr_type(expr, local_bindings, body_context, errors)
                                .display_name()
                                .unwrap_or_else(|| "Any".to_string()),
                            )
                        })
                        .collect(),
                )
            }
        }
        Expr::Match(scrutinee, arms) => {
            infer_expr_type(scrutinee, local_bindings, body_context, errors);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    infer_expr_type(guard, local_bindings, body_context, errors);
                }
                infer_expr_type(&arm.body, local_bindings, body_context, errors);
            }
            ValueType::Unknown
        }
        Expr::If(cond, then_expr, else_expr) => {
            infer_expr_type(cond, local_bindings, body_context, errors);
            let then_ty = infer_expr_type(then_expr, local_bindings, body_context, errors);
            let else_ty = else_expr.as_ref().map(|otherwise| {
                infer_expr_type(otherwise, local_bindings, body_context, errors)
            });
            match else_ty {
                Some(otherwise)
                    if then_ty.display_name().is_some()
                        && then_ty.display_name() == otherwise.display_name() =>
                {
                    then_ty
                }
                _ => ValueType::Unknown,
            }
        }
        Expr::For(_, iterable, body) => {
            infer_expr_type(iterable, local_bindings, body_context, errors);
            infer_expr_type(body, local_bindings, body_context, errors);
            ValueType::Unknown
        }
        Expr::Pipe(lhs, rhs) => {
            infer_expr_type(lhs, local_bindings, body_context, errors);
            infer_expr_type(rhs, local_bindings, body_context, errors)
        }
        Expr::Lambda(_, body) => infer_expr_type(body, local_bindings, body_context, errors),
        Expr::List(items) => {
            for item in items {
                infer_expr_type(item, local_bindings, body_context, errors);
            }
            ValueType::Named("List".to_string())
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                infer_expr_type(key, local_bindings, body_context, errors);
                infer_expr_type(value, local_bindings, body_context, errors);
            }
            ValueType::Named("Map".to_string())
        }
        Expr::Guarded(inner, guard) => {
            infer_expr_type(inner, local_bindings, body_context, errors);
            infer_expr_type(guard, local_bindings, body_context, errors);
            ValueType::Unknown
        }
        Expr::After(inner, _) => infer_expr_type(inner, local_bindings, body_context, errors),
        Expr::Return(fields) => ValueType::Record(
            fields
                .iter()
                .map(|(name, expr)| {
                    (
                        name.clone(),
                        infer_expr_type(expr, local_bindings, body_context, errors)
                        .display_name()
                        .unwrap_or_else(|| "Any".to_string()),
                    )
                })
                .collect(),
        ),
    }
}

impl ValueType {
    fn display_name(&self) -> Option<String> {
        match self {
            Self::Named(name) => Some(name.clone()),
            Self::Record(_) => Some("Record".to_string()),
            Self::Unknown => None,
        }
    }
}

fn push_type_mismatch_if_needed(expected: &str, inferred: &ValueType, errors: &mut Vec<TypeError>) {
    let Some(got) = inferred.display_name() else {
        return;
    };
    if !types_match(expected, &got) {
        errors.push(TypeError::TypeMismatch {
            expected: expected.to_string(),
            got,
        });
    }
}

fn types_match(expected: &str, got: &str) -> bool {
    if expected == got {
        return true;
    }
    let expected_canonical = canonical_type_name(expected);
    let got_canonical = canonical_type_name(got);
    expected_canonical == got_canonical
        || expected_canonical.rsplit('.').next() == got_canonical.rsplit('.').next()
}

fn resolve_record_fields(
    ty: &str,
    registry: &RecordTypeRegistry,
) -> Option<HashMap<String, String>> {
    let canonical = canonical_type_name(ty);
    if let Some(fields) = registry.full.get(&canonical) {
        return Some(fields.clone());
    }
    let short = canonical.rsplit('.').next().unwrap_or(canonical.as_str());
    let Some(Some(full_name)) = registry.short.get(short) else {
        return None;
    };
    registry.full.get(full_name).cloned()
}

fn validate_resource_interface_conformance(
    resource: &daglang_syntax::ast::ResourceDef,
    interface_registry: &InterfaceRegistry,
    errors: &mut Vec<TypeError>,
) {
    let Some(implemented) = resource.implements.as_deref() else {
        return;
    };
    let interface_contract = match resolve_interface_contract(implemented, interface_registry) {
        InterfaceResolution::Resolved(contract) => contract,
        InterfaceResolution::Ambiguous => {
            errors.push(TypeError::AmbiguousInterface {
                implementor: resource.name.clone(),
                interface: implemented.to_string(),
            });
            return;
        }
        InterfaceResolution::Missing => {
            errors.push(TypeError::UnresolvedInterface {
                implementor: resource.name.clone(),
                interface: implemented.to_string(),
            });
            return;
        }
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
    let interface_contract = match resolve_interface_contract(implemented, interface_registry) {
        InterfaceResolution::Resolved(contract) => contract,
        InterfaceResolution::Ambiguous => {
            errors.push(TypeError::AmbiguousInterface {
                implementor: service.name.clone(),
                interface: implemented.to_string(),
            });
            return;
        }
        InterfaceResolution::Missing => {
            errors.push(TypeError::UnresolvedInterface {
                implementor: service.name.clone(),
                interface: implemented.to_string(),
            });
            return;
        }
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
) -> InterfaceResolution {
    let canonical = canonical_type_name(implemented);
    if let Some(contract) = registry.full.get(&canonical) {
        return InterfaceResolution::Resolved(contract.clone());
    }
    let short = canonical.rsplit('.').next().unwrap_or(canonical.as_str());
    match registry.short.get(short) {
        Some(Some(contract)) => InterfaceResolution::Resolved(contract.clone()),
        Some(None) => InterfaceResolution::Ambiguous,
        None => InterfaceResolution::Missing,
    }
}

fn canonical_interface_name(name: &str) -> String {
    canonical_type_name(name)
}

fn resolve_resource_type_name(
    resource_type: &str,
    registry: &ResourceTypeRegistry,
) -> ResourceTypeResolution {
    if registry.full.contains(resource_type) {
        return ResourceTypeResolution::Resolved(resource_type.to_string());
    }
    let short = resource_type.rsplit('.').next().unwrap_or(resource_type);
    match registry.short.get(short) {
        Some(Some(resolved)) => ResourceTypeResolution::Resolved(resolved.clone()),
        Some(None) => ResourceTypeResolution::Ambiguous,
        None => ResourceTypeResolution::Missing,
    }
}

fn resolve_service_call_contract(
    call_path: &[String],
    registry: &ServiceCallRegistry,
) -> ServiceCallResolution {
    if call_path.len() < 2 {
        return ServiceCallResolution::Missing;
    }
    let Some(operation) = call_path.last() else {
        return ServiceCallResolution::Missing;
    };
    let service_name = call_path[..call_path.len() - 1].join(".");
    let short_service = call_path[call_path.len() - 2].clone();
    let keys = [
        format!("{service_name}.{operation}"),
        format!("{short_service}.{operation}"),
        call_path.join("."),
    ];
    let mut saw_ambiguous = false;
    for key in keys {
        if let Some(entry) = registry.by_key.get(&key) {
            match entry {
                Some(contract) => return ServiceCallResolution::Resolved(contract.clone()),
                None => saw_ambiguous = true,
            }
        }
    }
    if saw_ambiguous {
        ServiceCallResolution::Ambiguous
    } else {
        ServiceCallResolution::Missing
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct BodyServiceCall {
    path: Vec<String>,
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

fn collect_service_calls_from_stmts(stmts: &[Stmt], calls: &mut Vec<BodyServiceCall>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                collect_service_calls_from_expr(expr, calls);
            }
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    collect_service_calls_from_expr(expr, calls);
                }
            }
        }
    }
}

fn collect_service_calls_from_expr(expr: &Expr, calls: &mut Vec<BodyServiceCall>) {
    match expr {
        Expr::Call(_, args) => {
            for (_, arg) in args {
                collect_service_calls_from_expr(arg, calls);
            }
        }
        Expr::ServiceCall(path, args) => {
            calls.push(BodyServiceCall {
                path: path.clone(),
                arg_count: args.len(),
                named_args: args
                    .iter()
                    .filter_map(|(name, _)| name.clone())
                    .collect::<Vec<_>>(),
            });
            for (_, arg) in args {
                collect_service_calls_from_expr(arg, calls);
            }
        }
        Expr::FieldAccess(base, _) => collect_service_calls_from_expr(base, calls),
        Expr::BinOp(lhs, _, rhs) => {
            collect_service_calls_from_expr(lhs, calls);
            collect_service_calls_from_expr(rhs, calls);
        }
        Expr::UnaryOp(_, inner) => collect_service_calls_from_expr(inner, calls),
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_service_calls_from_expr(inner, calls);
                }
            }
        }
        Expr::Record(_, fields) => {
            for (_, value) in fields {
                collect_service_calls_from_expr(value, calls);
            }
        }
        Expr::Match(scrutinee, arms) => {
            collect_service_calls_from_expr(scrutinee, calls);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_service_calls_from_expr(guard, calls);
                }
                collect_service_calls_from_expr(&arm.body, calls);
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_service_calls_from_expr(cond, calls);
            collect_service_calls_from_expr(then_expr, calls);
            if let Some(otherwise) = else_expr {
                collect_service_calls_from_expr(otherwise, calls);
            }
        }
        Expr::For(_, iterable, body) => {
            collect_service_calls_from_expr(iterable, calls);
            collect_service_calls_from_expr(body, calls);
        }
        Expr::Pipe(lhs, rhs) => {
            collect_service_calls_from_expr(lhs, calls);
            collect_service_calls_from_expr(rhs, calls);
        }
        Expr::Lambda(_, body) => collect_service_calls_from_expr(body, calls),
        Expr::List(items) => {
            for item in items {
                collect_service_calls_from_expr(item, calls);
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                collect_service_calls_from_expr(key, calls);
                collect_service_calls_from_expr(value, calls);
            }
        }
        Expr::Guarded(inner, guard) => {
            collect_service_calls_from_expr(inner, calls);
            collect_service_calls_from_expr(guard, calls);
        }
        Expr::After(inner, _) => collect_service_calls_from_expr(inner, calls),
        Expr::Return(fields) => {
            for (_, value) in fields {
                collect_service_calls_from_expr(value, calls);
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
    generic_arity_registry: &GenericArityRegistry,
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
            if let Some(expected) =
                resolve_generic_arity(name, generic_arity_registry, known_types)
            {
                if expected != args.len() {
                    errors.push(TypeError::ArityMismatch {
                        name: name.clone(),
                        expected,
                        got: args.len(),
                    });
                }
            }
            if should_validate_named_type(name) && !known_types.contains(name) {
                let tail = name.rsplit('.').next().unwrap_or(name);
                if !known_types.contains(tail) {
                    errors.push(TypeError::UndefinedType(format!("{name} (in {context})")));
                }
            }
            for arg in args {
                validate_type_expr(arg, known_types, generic_arity_registry, context, errors);
            }
        }
        TypeExpr::Optional(inner) => {
            validate_type_expr(inner, known_types, generic_arity_registry, context, errors)
        }
        TypeExpr::Annotated(inner, annotations) => {
            validate_type_expr(inner, known_types, generic_arity_registry, context, errors);
            for annotation in annotations {
                if annotation.name != "range" {
                    continue;
                }
                let (min, max) = extract_range_bounds(&annotation.args);
                if let (Some(min), Some(max)) = (min, max) {
                    if min > max {
                        errors.push(TypeError::UnsatisfiableRefinement {
                            ty: type_expr_to_string(inner),
                            constraint: format!("range min {min} exceeds max {max}"),
                        });
                    }
                }
            }
        }
    }
}

fn resolve_generic_arity(
    name: &str,
    registry: &GenericArityRegistry,
    known_types: &HashSet<String>,
) -> Option<usize> {
    if let Some(arity) = registry.full.get(name) {
        return Some(*arity);
    }
    let short = name.rsplit('.').next().unwrap_or(name);
    if let Some(entry) = registry.short.get(short) {
        return *entry;
    }
    if known_types.contains(name) || known_types.contains(short) {
        return Some(0);
    }
    None
}

fn extract_range_bounds(args: &[Expr]) -> (Option<i64>, Option<i64>) {
    let mut min = None;
    let mut max = None;
    for arg in args {
        match arg {
            Expr::Record(_, fields) => {
                for (name, value) in fields {
                    match name.as_str() {
                        "min" => min = extract_int_literal(value),
                        "max" => max = extract_int_literal(value),
                        _ => {}
                    }
                }
            }
            _ => {
                if min.is_none() {
                    min = extract_int_literal(arg);
                } else if max.is_none() {
                    max = extract_int_literal(arg);
                }
            }
        }
    }
    (min, max)
}

fn extract_int_literal(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Literal(daglang_syntax::ast::Literal::Int(value)) => Some(*value),
        _ => None,
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
    fn duplicate_output_fields_are_reported() {
        let graph = module_graph_from_sources(&[(
            "dup_outputs.dag",
            r#"module sample.dup
func run() -> { ok: Bool, ok: Bool } {
  return { ok: true }
}
"#,
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("duplicate outputs should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateOutputField { item, field }
                if item == "run" && field == "ok"
        )));
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
    fn duplicate_definition_is_reported() {
        let graph = module_graph_from_sources(&[(
            "duplicate_definition.dag",
            r#"module sample.dup
fn run() -> Unit {}
func run() -> { ok: Bool } {
  return { ok: true }
}
"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("duplicate item name should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.dup" && name == "run"
        )));
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
    fn strict_mode_reports_ambiguous_call_target() {
        let graph = module_graph_from_sources(&[
            (
                "sample/one.dag",
                "module sample.one\nfn render(value: String) -> String { value }",
            ),
            (
                "sample/two.dag",
                "module sample.two\nfn render(value: String) -> String { value }",
            ),
            (
                "sample/main.dag",
                "module sample.main\nfn run() -> String { render(value: \"ok\") }",
            ),
        ]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for ambiguous callable target");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousCallTarget { caller, callee }
                if caller == "run" && callee == "render"
        )));
    }

    #[test]
    fn strict_mode_reports_unresolved_call_target() {
        let graph = module_graph_from_sources(&[(
            "sample/main.dag",
            "module sample.main\nfn run() -> String { missing(value: \"ok\") }",
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for unresolved callable target");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnresolvedCallTarget { caller, callee }
                if caller == "run" && callee == "missing"
        )));
    }

    #[test]
    fn relaxed_mode_allows_unresolved_call_target() {
        let graph = module_graph_from_sources(&[(
            "sample/main.dag",
            "module sample.main\nfn run() -> String { missing(value: \"ok\") }",
        )]);
        let typed = typecheck_module_graph(graph)
            .expect("relaxed mode should allow unresolved callable target");
        assert_eq!(typed.modules.len(), 1);
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
    fn duplicate_named_call_argument_is_reported() {
        let graph = module_graph_from_sources(&[(
            "duplicate_arg.dag",
            "module sample.calls\nfn fmt(value: String) -> String { value }\nfn run() -> String { fmt(value: \"a\", value: \"b\") }",
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("duplicate named argument should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateCallArgument {
                caller,
                callee,
                argument
            } if caller == "run" && callee == "fmt" && argument == "value"
        )));
    }

    #[test]
    fn service_call_arity_mismatch_is_reported() {
        let graph = module_graph_from_sources(&[(
            "service_arity_mismatch.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read()
  return { ok: true }
}"#,
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("service call arity mismatch should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::ServiceCallArityMismatch {
                caller,
                service_call,
                expected,
                got
            } if caller == "run"
                && service_call == "FsStorage.read"
                && *expected == 1
                && *got == 0
        )));
    }

    #[test]
    fn strict_mode_reports_unresolved_service_call() {
        let graph = module_graph_from_sources(&[(
            "service_unresolved_call.dag",
            r#"module sample.services
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}"#,
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for unresolved service call");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnresolvedServiceCall {
                caller,
                service_call
            } if caller == "run" && service_call == "MissingStorage.read"
        )));
    }

    #[test]
    fn strict_mode_reports_ambiguous_service_call() {
        let graph = module_graph_from_sources(&[
            (
                "sample/first.dag",
                r#"module sample.first
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
            ),
            (
                "sample/second.dag",
                r#"module sample.second
service SharedService {
  operation read(path: String) -> { body: String }
}"#,
            ),
            (
                "sample/main.dag",
                r#"module sample.main
func run(path: String) -> { body: String } {
  let response = SharedService.read(path: path)
  return { body: response.body }
}"#,
            ),
        ]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for ambiguous service call");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousServiceCall {
                caller,
                service_call
            } if caller == "run" && service_call == "SharedService.read"
        )));
    }

    #[test]
    fn relaxed_mode_allows_unresolved_service_call() {
        let graph = module_graph_from_sources(&[(
            "service_unresolved_call_relaxed.dag",
            r#"module sample.services
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}"#,
        )]);
        let typed = typecheck_module_graph(graph)
            .expect("relaxed mode should allow unresolved service call for lower-stage validation");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn unknown_named_service_call_argument_is_reported() {
        let graph = module_graph_from_sources(&[(
            "service_unknown_arg.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(file: path)
  return { body: response.body }
}"#,
        )]);
        let errors = typecheck_module_graph(graph)
            .expect_err("unknown named service call argument should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnknownServiceCallArgument {
                caller,
                service_call,
                argument
            } if caller == "run" && service_call == "FsStorage.read" && argument == "file"
        )));
    }

    #[test]
    fn duplicate_named_service_call_argument_is_reported() {
        let graph = module_graph_from_sources(&[(
            "service_duplicate_arg.dag",
            r#"module sample.services
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path, path: path)
  return { body: response.body }
}"#,
        )]);
        let errors = typecheck_module_graph(graph)
            .expect_err("duplicate named service call argument should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateServiceCallArgument {
                caller,
                service_call,
                argument
            } if caller == "run" && service_call == "FsStorage.read" && argument == "path"
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
    fn ambiguous_interface_on_resource_is_reported() {
        let graph = module_graph_from_sources(&[
            (
                "sample/first.dag",
                "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
            ),
            (
                "sample/second.dag",
                "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
            ),
            (
                "sample/main.dag",
                "module sample.main\nresource Disk implements Storage { capability read { input { path: String } output { body: String } } }",
            ),
        ]);
        let errors =
            typecheck_module_graph(graph).expect_err("ambiguous interface should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousInterface {
                implementor,
                interface
            } if implementor == "Disk" && interface == "Storage"
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
    fn ambiguous_interface_on_service_is_reported() {
        let graph = module_graph_from_sources(&[
            (
                "sample/first.dag",
                "module sample.first\ninterface Storage { capability read { input { path: String } output { body: String } } }",
            ),
            (
                "sample/second.dag",
                "module sample.second\ninterface Storage { capability read { input { path: String } output { body: String } } }",
            ),
            (
                "sample/main.dag",
                "module sample.main\nservice FsStorage implements Storage { operation read(path: String) -> { body: String } }",
            ),
        ]);
        let errors =
            typecheck_module_graph(graph).expect_err("ambiguous interface should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousInterface {
                implementor,
                interface
            } if implementor == "FsStorage" && interface == "Storage"
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
    fn strict_mode_reports_ambiguous_used_resource_type() {
        let graph = module_graph_from_sources(&[
            (
                "sample/one.dag",
                "module sample.one\nresource SharedResource {}",
            ),
            (
                "sample/two.dag",
                "module sample.two\nresource SharedResource {}",
            ),
            (
                "sample/main.dag",
                r#"module sample.main
func run() -> { ok: Bool } uses fs: SharedResource {
  return { ok: true }
}"#,
            ),
        ]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for ambiguous used resource type");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousUsedResourceType {
                item,
                binding,
                resource_type,
            } if item == "run" && binding == "fs" && resource_type == "SharedResource"
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

    #[test]
    fn strict_mode_reports_unknown_provided_resource_type() {
        let graph = module_graph_from_sources(&[(
            "unknown_provides.dag",
            "module sample.provides\nfunc run() -> { ok: Bool } provides out: MissingResource { return { ok: true } }",
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for unknown provided resource type");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnknownProvidedResourceType {
                item,
                binding,
                resource_type,
            } if item == "run" && binding == "out" && resource_type == "MissingResource"
        )));
    }

    #[test]
    fn strict_mode_reports_ambiguous_provided_resource_type() {
        let graph = module_graph_from_sources(&[
            (
                "sample/one.dag",
                "module sample.one\nresource SharedResource {}",
            ),
            (
                "sample/two.dag",
                "module sample.two\nresource SharedResource {}",
            ),
            (
                "sample/main.dag",
                r#"module sample.main
func run() -> { ok: Bool } provides out: SharedResource {
  return { ok: true }
}"#,
            ),
        ]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for ambiguous provided resource type");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousProvidedResourceType {
                item,
                binding,
                resource_type,
            } if item == "run" && binding == "out" && resource_type == "SharedResource"
        )));
    }

    #[test]
    fn duplicate_uses_binding_is_reported() {
        let graph = module_graph_from_sources(&[(
            "duplicate_uses.dag",
            r#"module sample.uses
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses fs: Storage uses fs: Storage {
  return { ok: true }
}"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("duplicate uses should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateUsesBinding { item, binding }
                if item == "run" && binding == "fs"
        )));
    }

    #[test]
    fn duplicate_provides_binding_is_reported() {
        let graph = module_graph_from_sources(&[(
            "duplicate_provides.dag",
            r#"module sample.provides
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage provides out: Storage {
  return { ok: true }
}"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("duplicate provides should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateProvidesBinding { item, binding }
                if item == "run" && binding == "out"
        )));
    }

    #[test]
    fn use_provide_binding_conflict_is_reported() {
        let graph = module_graph_from_sources(&[(
            "use_provide_conflict.dag",
            r#"module sample.conflict
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses io: Storage provides io: Storage {
  return { ok: true }
}"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("binding conflict should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UseProvideBindingConflict { item, binding }
                if item == "run" && binding == "io"
        )));
    }

    #[test]
    fn type_mismatch_in_fn_return_is_reported() {
        let graph = module_graph_from_sources(&[(
            "type_mismatch_fn_return.dag",
            r#"module sample.types
fn run() -> String { return 42 }"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("type mismatch should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::TypeMismatch { expected, got }
                if expected == "String" && got == "Int"
        )));
    }

    #[test]
    fn no_such_field_on_record_literal_is_reported() {
        let graph = module_graph_from_sources(&[(
            "no_such_field.dag",
            r#"module sample.fields
func run() -> { body: String } {
  let payload = { body: "ok" }
  return { body: payload.missing }
}"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("no such field should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::NoSuchField { ty, field } if ty == "Record" && field == "missing"
        )));
    }

    #[test]
    fn no_such_field_on_named_record_type_is_reported() {
        let graph = module_graph_from_sources(&[(
            "no_such_field_named_record.dag",
            r#"module sample.fields
type Payload { body: String }
fn run(input: Payload) -> String { input.missing }"#,
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("no such field on named record should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::NoSuchField { ty, field } if ty == "Payload" && field == "missing"
        )));
    }

    #[test]
    fn unsatisfiable_refinement_is_reported() {
        let graph = module_graph_from_sources(&[(
            "unsat_refinement.dag",
            r#"module sample.refinement
fn run(value: Int @range(min: 5, max: 1)) -> Int { value }"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("unsatisfiable range should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnsatisfiableRefinement { ty, constraint }
                if ty == "Int" && constraint.contains("min 5 exceeds max 1")
        )));
    }

    #[test]
    fn generic_arity_mismatch_is_reported() {
        let graph = module_graph_from_sources(&[(
            "generic_arity_mismatch.dag",
            r#"module sample.generics
fn run(items: Map<String>) -> Int { 1 }"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("generic arity mismatch should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::ArityMismatch {
                name,
                expected,
                got,
            } if name == "Map" && *expected == 2 && *got == 1
        )));
    }

    #[test]
    fn user_defined_generic_arity_mismatch_is_reported() {
        let graph = module_graph_from_sources(&[(
            "user_generic_arity_mismatch.dag",
            r#"module sample.generics
type Box<T> = T
fn run(value: Box<String, Int>) -> String { value }"#,
        )]);
        let errors = typecheck_module_graph(graph)
            .expect_err("user-defined generic arity mismatch should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::ArityMismatch {
                name,
                expected,
                got,
            } if name == "Box" && *expected == 1 && *got == 2
        )));
    }
}
