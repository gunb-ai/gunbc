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

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_syntax::ast::{
    Expr, Field, Item, Literal, Param, ProvidesClause, Refinement, SourceFile, Stmt,
    TypeBody, TypeExpr, UsesClause, PipelineDef,
};
use daglang_syntax::ast_utils::{
    resource_type_name, service_call_lookup_keys,
    should_track_call_name as should_validate_call_name, type_expr_to_string, walk_stmts,
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
        stage_names: Vec<String>,
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
    ArityMismatch {
        name: String,
        expected: usize,
        got: usize,
    },
    /// Duplicate top-level item name in a module.
    DuplicateDefinition { module: String, name: String },
    /// Duplicate stage name in a pipeline.
    DuplicatePipelineStage { pipeline: String, stage: String },
    /// Duplicate `after` dependency in a stage header.
    DuplicatePipelineStageDependency {
        pipeline: String,
        stage: String,
        dependency: String,
    },
    /// Unknown `after` dependency in a stage header.
    UnknownPipelineStageDependency {
        pipeline: String,
        stage: String,
        dependency: String,
    },
    /// Stage depends on itself via `after`.
    PipelineStageSelfDependency { pipeline: String, stage: String },
    /// Stage `when` condition did not infer to a boolean expression.
    PipelineStageWhenTypeMismatch {
        pipeline: String,
        stage: String,
        got: String,
    },
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
    AmbiguousCallTarget { caller: String, callee: String },
    /// Call expression target cannot be resolved to a callable contract.
    UnresolvedCallTarget { caller: String, callee: String },
    /// Service call expression used wrong number of arguments.
    ServiceCallArityMismatch {
        caller: String,
        service_call: String,
        expected: usize,
        got: usize,
    },
    /// Service call target could not be resolved to a known service operation contract.
    UnresolvedServiceCall {
        caller: String,
        service_call: String,
    },
    /// Service call target matches multiple possible service operation contracts.
    AmbiguousServiceCall {
        caller: String,
        service_call: String,
    },
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
            Self::DuplicatePipelineStage { pipeline, stage } => {
                write!(f, "duplicate stage `{stage}` in pipeline `{pipeline}`")
            }
            Self::DuplicatePipelineStageDependency {
                pipeline,
                stage,
                dependency,
            } => write!(
                f,
                "duplicate stage dependency `{dependency}` in pipeline `{pipeline}` stage `{stage}`"
            ),
            Self::UnknownPipelineStageDependency {
                pipeline,
                stage,
                dependency,
            } => write!(
                f,
                "unknown stage dependency `{dependency}` in pipeline `{pipeline}` stage `{stage}`"
            ),
            Self::PipelineStageSelfDependency { pipeline, stage } => write!(
                f,
                "stage `{stage}` in pipeline `{pipeline}` cannot depend on itself"
            ),
            Self::PipelineStageWhenTypeMismatch {
                pipeline,
                stage,
                got,
            } => write!(
                f,
                "stage `{stage}` in pipeline `{pipeline}` has non-bool `when` condition (got `{got}`)"
            ),
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
            ),        }
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
    let pattern_callable_names = collect_pattern_callable_names(&graph.modules);
    let service_call_registry = collect_service_call_contracts(&graph.modules);
    let interface_registry = collect_interfaces(&graph.modules);
    let resource_type_registry = collect_resource_types(&graph.modules);
    let resource_capability_registry = collect_resource_capabilities(&graph.modules);
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
        pattern_callable_names: &pattern_callable_names,
        service_call_registry: &service_call_registry,
        interface_registry: &interface_registry,
        resource_type_registry: &resource_type_registry,
        resource_capability_registry: &resource_capability_registry,
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
        let (signatures, sig_errors) = collect_signatures(&module, &context, &module_name);
        errors.extend(sig_errors);
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
    pattern_callable_names: &'a HashSet<String>,
    service_call_registry: &'a ServiceCallRegistry,
    interface_registry: &'a InterfaceRegistry,
    resource_type_registry: &'a ResourceTypeRegistry,
    resource_capability_registry: &'a ResourceCapabilityRegistry,
    allow_unresolved_references: bool,
}

fn collect_signatures(
    module: &ResolvedModule,
    context: &TypecheckContext<'_>,
    module_name: &str,
) -> (Vec<TypedItemSignature>, Vec<TypeError>) {
    let mut errors = Vec::new();
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
        pattern_callable_names: context.pattern_callable_names,
        service_call_registry: context.service_call_registry,
        interface_registry: context.interface_registry,
        resource_type_registry: context.resource_type_registry,
        resource_capability_registry: context.resource_capability_registry,
        allow_unresolved_references: context.allow_unresolved_references,
    };
    let pipeline_param_bindings = collect_pipeline_param_bindings(module);

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
                errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                let item_known_types = extend_known_types(&module_known_types, &def.type_params);
                errors.extend(validate_params(
                    &def.name,
                    &def.params,
                    &item_known_types,
                    context.generic_arity_registry,
                ));
                // Handle anonymous record return types: `fn foo() -> { field: Type }`
                let (return_contract, outputs) = match &def.return_type {
                    TypeExpr::Record(fields) => {
                        for field in fields {
                            errors.extend(validate_type_expr(
                                &field.ty,
                                &item_known_types,
                                context.generic_arity_registry,
                                &format!("{}.{}", def.name, field.name),
                            ));
                        }
                        (
                            ReturnContract::record(field_signature_map(fields)),
                            fields
                                .iter()
                                .map(|f| TypedBinding {
                                    name: f.name.clone(),
                                    ty: type_expr_to_string(&f.ty),
                                })
                                .collect(),
                        )
                    }
                    _ => {
                        errors.extend(validate_type_expr(
                            &def.return_type,
                            &item_known_types,
                            context.generic_arity_registry,
                            &format!("{}.return", def.name),
                        ));
                        (
                            ReturnContract::single(type_expr_to_string(&def.return_type)),
                            vec![TypedBinding {
                                name: "return".to_string(),
                                ty: type_expr_to_string(&def.return_type),
                            }],
                        )
                    }
                };
                errors.extend(validate_callable_body(
                    &def.name,
                    &def.params,
                    return_contract,
                    &[],
                    CallableBodyRef {
                        stmts: &def.body.stmts,
                        is_lossy: def.body.lossy,
                    },
                    &body_context,
                ));
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
                errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                let item_known_types = extend_known_types(&module_known_types, &def.type_params);
                errors.extend(validate_params(
                    &def.name,
                    &def.params,
                    &item_known_types,
                    context.generic_arity_registry,
                ));
                errors.extend(validate_outputs(
                    &def.name,
                    &def.outputs,
                    &item_known_types,
                    context.generic_arity_registry,
                ));
                errors.extend(validate_uses_clauses(
                    &def.name,
                    &def.uses,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                ));
                errors.extend(validate_provides_clauses(
                    &def.name,
                    &def.provides,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                ));
                errors.extend(validate_use_provide_binding_conflicts(
                    &def.name,
                    &def.uses,
                    &def.provides,
                ));
                errors.extend(validate_callable_body(
                    &def.name,
                    &def.params,
                    ReturnContract::record(field_signature_map(&def.outputs)),
                    &def.uses,
                    CallableBodyRef {
                        stmts: &def.body.stmts,
                        is_lossy: def.body.lossy,
                    },
                    &body_context,
                ));
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
                errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                let item_known_types = extend_known_types(&module_known_types, &def.type_params);
                errors.extend(validate_params(
                    &def.name,
                    &def.params,
                    &item_known_types,
                    context.generic_arity_registry,
                ));
                errors.extend(validate_outputs(
                    &def.name,
                    &def.outputs,
                    &item_known_types,
                    context.generic_arity_registry,
                ));
                errors.extend(validate_uses_clauses(
                    &def.name,
                    &def.uses,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                ));
                errors.extend(validate_callable_body(
                    &def.name,
                    &def.params,
                    ReturnContract::record(field_signature_map(&def.outputs)),
                    &def.uses,
                    CallableBodyRef {
                        stmts: &def.body.stmts,
                        is_lossy: def.body.lossy,
                    },
                    &body_context,
                ));
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
                errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                errors.extend(validate_service_interface_conformance(
                    def,
                    context.interface_registry,
                ));
                signatures.push(TypedItemSignature::Service {
                    name: def.name.clone(),
                    operations: def.operations.len(),
                });
            }
            Item::ResourceDef(def) => {
                errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                errors.extend(validate_resource_interface_conformance(
                    def,
                    context.interface_registry,
                ));
                signatures.push(TypedItemSignature::Resource {
                    name: def.name.clone(),
                    implements: def.implements.clone(),
                });
            }
            Item::InterfaceDef(def) => {
                errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                signatures.push(TypedItemSignature::Interface {
                    name: def.name.clone(),
                    capabilities: def.capabilities.len(),
                });
            }
            Item::PipelineDef(def) => {
                errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                errors.extend(validate_pipeline_def(
                    def,
                    &pipeline_param_bindings,
                    &body_context,
                ));
                signatures.push(TypedItemSignature::Pipeline {
                    name: def.name.clone(),
                    stages: def.stages.len(),
                    stage_names: def.stages.iter().map(|stage| stage.name.clone()).collect(),
                });
            }
            // Test and fixture definitions are handled by the test lowering
            // pass, not the standard typecheck/lower pipeline.
            Item::TestDef(_) | Item::FixtureDef(_) => {}
            // Project/SDLC blocks are not typechecked yet in this pass
            Item::ProjectDef(_)
            | Item::FeatureDef(_)
            | Item::TaskDef(_)
            | Item::DesignDef(_)
            | Item::ComponentDef(_)
            | Item::EnvironmentDef(_)
            | Item::ProfileDef(_)
            | Item::ParamDecl(_)
            | Item::DataDef(_) => {}
            Item::ExternFuncDecl(def) => {
                errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                for field in &def.inputs {
                    errors.extend(validate_type_expr(
                        &field.ty,
                        &module_known_types,
                        context.generic_arity_registry,
                        &format!("{}.{}", def.name, field.name),
                    ));
                }
                for field in &def.outputs {
                    errors.extend(validate_type_expr(
                        &field.ty,
                        &module_known_types,
                        context.generic_arity_registry,
                        &format!("{}.{}", def.name, field.name),
                    ));
                }
            }
            Item::ExternAssetDecl(def) => {
                errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                errors.extend(validate_type_expr(
                    &def.ty,
                    &module_known_types,
                    context.generic_arity_registry,
                    &def.name,
                ));
            }
        }
    }

    (signatures, errors)
}

fn collect_pipeline_param_bindings(module: &ResolvedModule) -> HashMap<String, ValueType> {
    let mut bindings = HashMap::new();
    for item in &module.ast.items {
        if let Item::ParamDecl(decl) = &item.node {
            bindings.insert(
                decl.name.clone(),
                ValueType::Named(type_expr_to_string(&decl.ty)),
            );
        }
    }
    bindings
}

fn validate_pipeline_def(
    def: &PipelineDef,
    param_bindings: &HashMap<String, ValueType>,
    body_context: &BodyInferenceContext<'_>,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let pipeline_name = def.name.clone();
    let mut seen_stage_names = HashSet::new();
    let mut all_stage_names = HashSet::new();

    for stage in &def.stages {
        if !seen_stage_names.insert(stage.name.clone()) {
            errors.push(TypeError::DuplicatePipelineStage {
                pipeline: pipeline_name.clone(),
                stage: stage.name.clone(),
            });
        }
        all_stage_names.insert(stage.name.clone());
    }

    let empty_bound_services = BoundServiceCallRegistry::default();
    let empty_param_callable_contracts = HashMap::new();
    let infer_context = ExprInferenceContext {
        record_type_registry: body_context.record_type_registry,
        callable_registry: body_context.callable_registry,
        service_call_registry: body_context.service_call_registry,
        bound_service_registry: &empty_bound_services,
        param_callable_contracts: &empty_param_callable_contracts,
    };

    for stage in &def.stages {
        let mut seen_dependencies = HashSet::new();
        for dependency in &stage.after {
            if !seen_dependencies.insert(dependency.clone()) {
                errors.push(TypeError::DuplicatePipelineStageDependency {
                    pipeline: pipeline_name.clone(),
                    stage: stage.name.clone(),
                    dependency: dependency.clone(),
                });
                continue;
            }
            if dependency == &stage.name {
                errors.push(TypeError::PipelineStageSelfDependency {
                    pipeline: pipeline_name.clone(),
                    stage: stage.name.clone(),
                });
                continue;
            }
            if !all_stage_names.contains(dependency) {
                errors.push(TypeError::UnknownPipelineStageDependency {
                    pipeline: pipeline_name.clone(),
                    stage: stage.name.clone(),
                    dependency: dependency.clone(),
                });
            }
        }

        if let Some(condition) = &stage.when {
            let (inferred, infer_errors) =
                infer_expr_type(condition, param_bindings, &infer_context);
            errors.extend(infer_errors);
            let is_bool = matches!(
                inferred,
                ValueType::Named(ref name) if strip_generic_params(name) == "Bool"
            );
            if !is_bool && !matches!(inferred, ValueType::Unknown) {
                errors.push(TypeError::PipelineStageWhenTypeMismatch {
                    pipeline: pipeline_name.clone(),
                    stage: stage.name.clone(),
                    got: inferred.display_name().unwrap_or_else(|| "Unknown".to_string()),
                });
            }
        }
    }

    errors
}

fn extend_known_types(base: &HashSet<String>, additional: &[String]) -> HashSet<String> {
    let mut known = base.clone();
    known.extend(additional.iter().cloned());
    known
}

fn collect_known_types(modules: &[ResolvedModule]) -> HashSet<String> {
    let mut known = builtin_type_names();
    for module in modules {
        let module_prefix = module.module_path.join(".");
        for item in &module.ast.items {
            match &item.node {
                Item::TypeDef(def) => {
                    known.insert(def.name.clone());
                    known.insert(format!("{module_prefix}.{}", def.name));
                }
                Item::ResourceDef(def) => {
                    let config_name = format!("{}.Config", def.name);
                    known.insert(config_name.clone());
                    known.insert(format!("{module_prefix}.{config_name}"));
                }
                _ => {}
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
                Item::ResourceDef(def) => {
                    let name = format!("{}.Config", def.name);
                    let full_name = format!("{module_prefix}.{name}");
                    registry.full.insert(full_name, 0);
                    registry
                        .short
                        .entry(name)
                        .and_modify(|existing| {
                            if let Some(current) = existing {
                                if *current != 0 {
                                    *existing = None;
                                }
                            }
                        })
                        .or_insert(Some(0));
                    continue;
                }
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
            match &item.node {
                Item::TypeDef(def) => {
                    let daglang_syntax::ast::TypeBody::Record(fields) = &def.body else {
                        continue;
                    };
                    let signature = field_signature_map(fields);
                    let full_name = format!("{module_prefix}.{}", def.name);
                    registry.full.insert(full_name.clone(), signature.clone());
                    registry.full.entry(def.name.clone()).or_insert(signature);
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
                Item::ResourceDef(def) if !def.config.is_empty() => {
                    let signature = field_signature_map(&def.config);
                    let config_name = format!("{}.Config", def.name);
                    let full_name = format!("{module_prefix}.{config_name}");
                    registry.full.insert(full_name.clone(), signature.clone());
                    registry.full.insert(config_name.clone(), signature);
                    registry
                        .short
                        .entry(config_name)
                        .and_modify(|existing| {
                            if let Some(current) = existing {
                                if current != &full_name {
                                    *existing = None;
                                }
                            }
                        })
                        .or_insert(Some(full_name));
                }
                _ => {}
            }
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
            match &item.node {
                Item::FnDef(def) => register_callable_contract(
                    &mut callables,
                    def.name.clone(),
                    CallableContract {
                        arity: required_param_arity(&def.params),
                        params: def.params.iter().map(|param| param.name.clone()).collect(),
                        output: ValueType::Named(type_expr_to_string(&def.return_type)),
                    },
                ),
                Item::FuncDef(def) => register_callable_contract(
                    &mut callables,
                    def.name.clone(),
                    CallableContract {
                        arity: required_param_arity(&def.params),
                        params: def.params.iter().map(|param| param.name.clone()).collect(),
                        output: if def.outputs.len() == 1 && def.outputs[0].name == "return" {
                            ValueType::Named(type_expr_to_string(&def.outputs[0].ty))
                        } else {
                            ValueType::Record(field_signature_map(&def.outputs))
                        },
                    },
                ),
                Item::PatternDef(def) => register_callable_contract(
                    &mut callables,
                    def.name.clone(),
                    CallableContract {
                        arity: required_param_arity(&def.params),
                        params: def.params.iter().map(|param| param.name.clone()).collect(),
                        output: if def.outputs.len() == 1 && def.outputs[0].name == "return" {
                            ValueType::Named(type_expr_to_string(&def.outputs[0].ty))
                        } else {
                            ValueType::Record(field_signature_map(&def.outputs))
                        },
                    },
                ),
                Item::ExternFuncDecl(def) => register_callable_contract(
                    &mut callables,
                    def.name.clone(),
                    CallableContract {
                        arity: required_field_arity(&def.inputs),
                        params: def.inputs.iter().map(|field| field.name.clone()).collect(),
                        output: if def.outputs.len() == 1 && def.outputs[0].name == "return" {
                            ValueType::Named(type_expr_to_string(&def.outputs[0].ty))
                        } else {
                            ValueType::Record(field_signature_map(&def.outputs))
                        },
                    },
                ),
                Item::TypeDef(def) => {
                    if let TypeBody::Sum(variants) = &def.body {
                        for variant in variants {
                            register_callable_contract(
                                &mut callables,
                                variant.name.clone(),
                                CallableContract {
                                    arity: required_field_arity(&variant.fields),
                                    params: variant
                                        .fields
                                        .iter()
                                        .map(|field| field.name.clone())
                                        .collect(),
                                    output: ValueType::Named(def.name.clone()),
                                },
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }
    for (name, contract) in builtin_callable_contracts() {
        callables.entry(name).or_insert(Some(contract));
    }
    callables
}

fn collect_pattern_callable_names(modules: &[ResolvedModule]) -> HashSet<String> {
    modules
        .iter()
        .flat_map(|module| module.ast.items.iter())
        .filter_map(|item| match &item.node {
            Item::PatternDef(def) => Some(def.name.clone()),
            _ => None,
        })
        .collect()
}

fn required_param_arity(params: &[Param]) -> usize {
    params
        .iter()
        .filter(|param| param.default.is_none())
        .count()
}

fn required_field_arity(fields: &[Field]) -> usize {
    fields
        .iter()
        .filter(|field| field.default.is_none())
        .count()
}

fn callable_contract_max_arity(contract: &CallableContract) -> usize {
    contract.arity.max(contract.params.len())
}

fn service_contract_max_arity(contract: &ServiceCallContract) -> usize {
    contract.arity.max(contract.params.len())
}

fn register_callable_contract(
    callables: &mut HashMap<String, Option<CallableContract>>,
    name: String,
    contract: CallableContract,
) {
    callables
        .entry(name)
        .and_modify(|existing| {
            if existing.is_some() {
                *existing = None;
            }
        })
        .or_insert(Some(contract));
}

fn builtin_callable_contracts() -> Vec<(String, CallableContract)> {
    vec![
        (
            "map".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["f".to_string()]),
                output: ValueType::Named("List".to_string()),
            },
        ),
        (
            "filter".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["predicate".to_string()]),
                output: ValueType::Named("List".to_string()),
            },
        ),
        (
            "filter_map".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["f".to_string()]),
                output: ValueType::Named("List".to_string()),
            },
        ),
        (
            "flat_map".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["f".to_string()]),
                output: ValueType::Named("List".to_string()),
            },
        ),
        (
            "max_by".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["f".to_string()]),
                output: ValueType::Named("Any".to_string()),
            },
        ),
        (
            "starts_with".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["prefix".to_string()]),
                output: ValueType::Named("Bool".to_string()),
            },
        ),
        (
            "any".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["predicate".to_string()]),
                output: ValueType::Named("Bool".to_string()),
            },
        ),
        (
            "append".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["items".to_string()]),
                output: ValueType::Named("List".to_string()),
            },
        ),
        (
            "render_cytoscape_html".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["snapshot".to_string()]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "render_mermaid_markdown".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["snapshot".to_string()]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "join".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["separator".to_string()]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "repeat".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["n".to_string()]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "count".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                output: ValueType::Named("Int".to_string()),
            },
        ),
        (
            "all".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["predicate".to_string()]),
                output: ValueType::Named("Bool".to_string()),
            },
        ),
        (
            "sort_by".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["key_fn".to_string()]),
                output: ValueType::Named("List".to_string()),
            },
        ),
        (
            "first".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                output: ValueType::Named("Any".to_string()),
            },
        ),
        (
            "last".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                output: ValueType::Named("Any".to_string()),
            },
        ),
        (
            "ends_with".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["suffix".to_string()]),
                output: ValueType::Named("Bool".to_string()),
            },
        ),
        (
            "to_bytes".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::from(["value".to_string()]),
                output: ValueType::Named("Bytes".to_string()),
            },
        ),
        (
            "to_json".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::from(["value".to_string()]),
                output: ValueType::Named("Json".to_string()),
            },
        ),
        (
            "hash".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::from(["value".to_string()]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "eq".to_string(),
            CallableContract {
                arity: 2,
                params: HashSet::from(["a".to_string(), "b".to_string()]),
                output: ValueType::Named("Bool".to_string()),
            },
        ),
        (
            "replace_section".to_string(),
            CallableContract {
                arity: 2,
                params: HashSet::from(["section".to_string(), "replacement".to_string()]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "render_test_listings".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["sources".to_string()]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "render_graph_structure".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["sources".to_string()]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "render_source_artifacts".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["sources".to_string()]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "build_token".to_string(),
            CallableContract {
                arity: 5,
                params: HashSet::from([
                    "payload".to_string(),
                    "scheme".to_string(),
                    "header_name".to_string(),
                    "source_id".to_string(),
                    "required_scopes".to_string(),
                ]),
                output: ValueType::Named("AccessToken".to_string()),
            },
        ),
        (
            "generate".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "now".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "compute_topology_diff".to_string(),
            CallableContract {
                arity: 2,
                params: HashSet::from(["current".to_string(), "base".to_string()]),
                output: ValueType::Named("DagDiff".to_string()),
            },
        ),
        (
            "render_annotated_mermaid".to_string(),
            CallableContract {
                arity: 3,
                params: HashSet::from([
                    "diff".to_string(),
                    "topology".to_string(),
                    "title".to_string(),
                ]),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "detect_runtime".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                output: ValueType::Named("CloudRuntime".to_string()),
            },
        ),
        (
            "chars".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["s".to_string()]),
                output: ValueType::Named("List<Char>".to_string()),
            },
        ),
        (
            "code_point".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["c".to_string()]),
                output: ValueType::Named("Int".to_string()),
            },
        ),
        (
            "sum".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                output: ValueType::Named("Int".to_string()),
            },
        ),
        (
            "contains".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["item".to_string()]),
                output: ValueType::Named("Bool".to_string()),
            },
        ),
        (
            "split".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["delimiter".to_string()]),
                output: ValueType::Named("List".to_string()),
            },
        ),
        (
            "zip".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["other".to_string()]),
                output: ValueType::Named("List".to_string()),
            },
        ),
    ]
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
                    arity: required_field_arity(&operation.inputs),
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
                keys.insert(format!(
                    "{}.{}.{}",
                    module_name, service.name, operation.name
                ));
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
struct ResourceCapabilityRegistry {
    full: HashMap<String, HashMap<String, CapabilityContract>>,
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
    pattern_callable_names: &'a HashSet<String>,
    service_call_registry: &'a ServiceCallRegistry,
    interface_registry: &'a InterfaceRegistry,
    resource_type_registry: &'a ResourceTypeRegistry,
    resource_capability_registry: &'a ResourceCapabilityRegistry,
    allow_unresolved_references: bool,
}

#[derive(Debug, Clone, Default)]
struct BoundServiceCallRegistry {
    by_binding: HashMap<String, BoundServiceCallBinding>,
}

#[derive(Debug, Clone)]
enum BoundServiceCallBinding {
    Resolved(HashMap<String, ServiceCallContract>),
    Deferred,
}

#[derive(Debug, Clone)]
enum BoundServiceCallResolution {
    Resolved(ServiceCallContract),
    MissingCapability,
    Deferred,
    NotBound,
}

struct ExprInferenceContext<'a> {
    record_type_registry: &'a RecordTypeRegistry,
    callable_registry: &'a HashMap<String, Option<CallableContract>>,
    service_call_registry: &'a ServiceCallRegistry,
    bound_service_registry: &'a BoundServiceCallRegistry,
    param_callable_contracts: &'a HashMap<String, CallableContract>,
}

struct CallableBodyRef<'a> {
    stmts: &'a [Stmt],
    is_lossy: bool,
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
            let contract = InterfaceContract {
                type_params: interface.type_params.clone(),
                capabilities,
            };
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
                    if existing.is_some() {
                        *existing = None;
                    }
                })
                .or_insert_with(|| Some(full));
        }
    }
    registry
}

fn collect_resource_capabilities(modules: &[ResolvedModule]) -> ResourceCapabilityRegistry {
    let mut registry = ResourceCapabilityRegistry::default();
    for module in modules {
        let module_name = module.module_path.join(".");
        for item in &module.ast.items {
            let Item::ResourceDef(resource) = &item.node else {
                continue;
            };
            let capabilities = resource
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
            registry
                .full
                .insert(format!("{module_name}.{}", resource.name), capabilities);
        }
    }
    registry
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InterfaceContract {
    type_params: Vec<String>,
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
        "Secret".to_string(),
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
        ("Secret".to_string(), 0),
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
) -> Vec<TypeError> {
    if !seen_items.insert(item_name.to_string()) {
        vec![TypeError::DuplicateDefinition {
            module: module_name.to_string(),
            name: item_name.to_string(),
        }]
    } else {
        Vec::new()
    }
}

fn validate_params(
    item_name: &str,
    params: &[Param],
    known_types: &HashSet<String>,
    generic_arity_registry: &GenericArityRegistry,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for param in params {
        if !seen.insert(param.name.clone()) {
            errors.push(TypeError::DuplicateParameter {
                item: item_name.to_string(),
                param: param.name.clone(),
            });
        }
        errors.extend(validate_type_expr(
            &param.ty,
            known_types,
            generic_arity_registry,
            &format!("{}.{}", item_name, param.name),
        ));
    }
    errors
}

fn validate_outputs(
    item_name: &str,
    outputs: &[Field],
    known_types: &HashSet<String>,
    generic_arity_registry: &GenericArityRegistry,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let mut seen = HashSet::new();
    for output in outputs {
        if !seen.insert(output.name.clone()) {
            errors.push(TypeError::DuplicateOutputField {
                item: item_name.to_string(),
                field: output.name.clone(),
            });
        }
        errors.extend(validate_type_expr(
            &output.ty,
            known_types,
            generic_arity_registry,
            &format!("{}.{}", item_name, output.name),
        ));
    }
    errors
}

fn validate_uses_clauses(
    item_name: &str,
    uses: &[UsesClause],
    registry: &ResourceTypeRegistry,
    allow_unresolved_references: bool,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let mut seen_bindings = HashSet::new();
    for usage in uses {
        if !seen_bindings.insert(usage.binding.clone()) {
            errors.push(TypeError::DuplicateUsesBinding {
                item: item_name.to_string(),
                binding: usage.binding.clone(),
            });
        }
        if !allow_unresolved_references {
            let resource_type = resource_type_name(&usage.resource_type);
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
    errors
}

fn validate_provides_clauses(
    item_name: &str,
    provides: &[ProvidesClause],
    registry: &ResourceTypeRegistry,
    allow_unresolved_references: bool,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let mut seen_bindings = HashSet::new();
    for provided in provides {
        if !seen_bindings.insert(provided.binding.clone()) {
            errors.push(TypeError::DuplicateProvidesBinding {
                item: item_name.to_string(),
                binding: provided.binding.clone(),
            });
        }
        if !allow_unresolved_references {
            let resource_type = resource_type_name(&provided.resource_type);
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
    errors
}

fn validate_use_provide_binding_conflicts(
    item_name: &str,
    uses: &[UsesClause],
    provides: &[ProvidesClause],
) -> Vec<TypeError> {
    let mut errors = Vec::new();
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
    errors
}

fn collect_param_callable_contracts(params: &[Param]) -> HashMap<String, CallableContract> {
    params
        .iter()
        .filter_map(|param| {
            parse_function_type_callable_contract(&param.ty)
                .map(|contract| (param.name.clone(), contract))
        })
        .collect()
}

fn parse_function_type_callable_contract(ty: &TypeExpr) -> Option<CallableContract> {
    let raw = type_expr_to_string(ty);
    let compact = raw
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    if !compact.starts_with("fn(") {
        return None;
    }
    let close_paren = find_matching_paren(&compact, 2)?;
    let args = &compact[3..close_paren];
    let output = compact
        .get(close_paren + 1..)?
        .strip_prefix("->")
        .filter(|text| !text.is_empty())?
        .to_string();
    Some(CallableContract {
        arity: parse_function_type_arity(args),
        params: HashSet::new(),
        output: ValueType::Named(output),
    })
}

fn find_matching_paren(text: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in text.char_indices().filter(|(idx, _)| *idx >= open_index) {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_function_type_arity(args: &str) -> usize {
    if args.is_empty() {
        return 0;
    }
    let mut arity = 1usize;
    let mut paren_depth = 0usize;
    let mut angle_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    for ch in args.chars() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ',' if paren_depth == 0
                && angle_depth == 0
                && bracket_depth == 0
                && brace_depth == 0 =>
            {
                arity += 1;
            }
            _ => {}
        }
    }
    arity
}

fn validate_callable_body(
    caller: &str,
    params: &[Param],
    return_contract: ReturnContract,
    uses: &[UsesClause],
    body: CallableBodyRef<'_>,
    body_context: &BodyInferenceContext<'_>,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let bound_service_registry = build_bound_service_call_registry(uses, body_context);
    let param_callable_contracts = collect_param_callable_contracts(params);
    let infer_context = ExprInferenceContext {
        record_type_registry: body_context.record_type_registry,
        callable_registry: body_context.callable_registry,
        service_call_registry: body_context.service_call_registry,
        bound_service_registry: &bound_service_registry,
        param_callable_contracts: &param_callable_contracts,
    };
    let mut calls = Vec::new();
    collect_calls_from_stmts(body.stmts, &mut calls);
    for call in calls {
        let contract = match param_callable_contracts.get(&call.callee) {
            Some(contract) => contract,
            None => match body_context.callable_registry.get(&call.callee) {
                Some(Some(contract)) => contract,
                Some(None) => {
                    if !body_context.allow_unresolved_references {
                        errors.push(TypeError::AmbiguousCallTarget {
                            caller: caller.to_string(),
                            callee: call.callee.clone(),
                        });
                    }
                    continue;
                }
                None => {
                    if !body_context.allow_unresolved_references {
                        errors.push(TypeError::UnresolvedCallTarget {
                            caller: caller.to_string(),
                            callee: call.callee.clone(),
                        });
                    }
                    continue;
                }
            },
        };
        let is_pattern_callable = body_context.pattern_callable_names.contains(&call.callee);
        if !is_pattern_callable {
            let max_arity = callable_contract_max_arity(contract);
            if call.arg_count < contract.arity || call.arg_count > max_arity {
                let expected = if call.arg_count < contract.arity {
                    contract.arity
                } else {
                    max_arity
                };
                errors.push(TypeError::CallArityMismatch {
                    caller: caller.to_string(),
                    callee: call.callee.clone(),
                    expected,
                    got: call.arg_count,
                });
            }
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
            if !is_pattern_callable && !contract.params.contains(&named) {
                errors.push(TypeError::UnknownCallArgument {
                    caller: caller.to_string(),
                    callee: call.callee.clone(),
                    argument: named,
                });
            }
        }
    }

    let mut service_calls = Vec::new();
    collect_service_calls_from_stmts(body.stmts, &mut service_calls);
    for call in service_calls {
        let service_call_name = call.path.join(".");
        let contract =
            match resolve_service_call_contract(&call.path, body_context.service_call_registry) {
                ServiceCallResolution::Resolved(contract) => Some(contract),
                ServiceCallResolution::Ambiguous => {
                    if !body_context.allow_unresolved_references {
                        errors.push(TypeError::AmbiguousServiceCall {
                            caller: caller.to_string(),
                            service_call: service_call_name.clone(),
                        });
                    }
                    None
                }
                ServiceCallResolution::Missing => {
                    match resolve_bound_service_call_contract(&call.path, &bound_service_registry) {
                        BoundServiceCallResolution::Resolved(contract) => Some(contract),
                        BoundServiceCallResolution::MissingCapability
                        | BoundServiceCallResolution::NotBound => {
                            if !body_context.allow_unresolved_references {
                                errors.push(TypeError::UnresolvedServiceCall {
                                    caller: caller.to_string(),
                                    service_call: service_call_name.clone(),
                                });
                            }
                            None
                        }
                        BoundServiceCallResolution::Deferred => None,
                    }
                }
            };
        let Some(contract) = contract else {
            continue;
        };
        let max_arity = service_contract_max_arity(&contract);
        if call.arg_count < contract.arity || call.arg_count > max_arity {
            let expected = if call.arg_count < contract.arity {
                contract.arity
            } else {
                max_arity
            };
            errors.push(TypeError::ServiceCallArityMismatch {
                caller: caller.to_string(),
                service_call: service_call_name.clone(),
                expected,
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
    let mut saw_explicit_return = false;
    let mut trailing_expr_type = None;
    let mut trailing_expr = None;
    for stmt in body.stmts {
        match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                let (inferred, infer_errors) =
                    infer_expr_type(expr, &local_bindings, &infer_context);
                errors.extend(infer_errors);
                local_bindings.insert(name.clone(), inferred);
                trailing_expr_type = None;
                trailing_expr = None;
            }
            Stmt::Node(ns) => {
                let (inferred, infer_errors) =
                    infer_expr_type(&ns.expr, &local_bindings, &infer_context);
                errors.extend(infer_errors);
                local_bindings.insert(ns.name.clone(), inferred);
                trailing_expr_type = None;
                trailing_expr = None;
            }
            Stmt::Expr(expr) => {
                trailing_expr = Some(expr);
                let (inferred, infer_errors) =
                    infer_expr_type(expr, &local_bindings, &infer_context);
                errors.extend(infer_errors);
                trailing_expr_type = Some(inferred);
            }
            Stmt::Return(fields) => {
                saw_explicit_return = true;
                trailing_expr_type = None;
                trailing_expr = None;
                errors.extend(validate_return_stmt(
                    caller,
                    &return_contract,
                    fields,
                    &local_bindings,
                    &infer_context,
                ));
            }
        }
    }
    if !body.is_lossy && !saw_explicit_return {
        if let ReturnContract::Single { ty } = &return_contract {
            let inferred = match trailing_expr {
                Some(expr) => {
                    let (val, infer_errors) = infer_expr_type_for_expected_named_record(
                        expr,
                        ty,
                        &local_bindings,
                        &infer_context,
                    );
                    errors.extend(infer_errors);
                    val
                }
                None => trailing_expr_type.unwrap_or_else(|| ValueType::Named("Unit".to_string())),
            };
            let mismatches = push_type_mismatch_if_needed(ty, &inferred);
            if !mismatches.is_empty() {
                eprintln!("[DEBUG callable_body] caller={caller:?} return_ty={ty:?} inferred={inferred:?}");
            }
            errors.extend(mismatches);
        }
    }
    errors
}

fn validate_return_stmt(
    caller: &str,
    return_contract: &ReturnContract,
    fields: &[(String, Expr)],
    local_bindings: &HashMap<String, ValueType>,
    infer_context: &ExprInferenceContext<'_>,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    match return_contract {
        ReturnContract::Single { ty } => {
            if fields.len() != 1 {
                eprintln!("[DEBUG validate_return] caller={caller:?} expected single return type={ty:?} but got {len} fields", len=fields.len());
                errors.push(TypeError::TypeMismatch {
                    expected: ty.clone(),
                    got: "Record".to_string(),
                });
                return errors;
            }
            let (inferred, infer_errors) = infer_expr_type_for_expected_named_record(
                &fields[0].1,
                ty,
                local_bindings,
                infer_context,
            );
            errors.extend(infer_errors);
            let mismatches = push_type_mismatch_if_needed(ty, &inferred);
            if !mismatches.is_empty() {
                eprintln!("[DEBUG return_single] caller={caller:?} ty={ty:?} inferred={inferred:?} fields={:?}", fields.iter().map(|(n,_)| n.as_str()).collect::<Vec<_>>());
            }
            errors.extend(mismatches);
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
                let (inferred, infer_errors) = infer_expr_type(expr, local_bindings, infer_context);
                errors.extend(infer_errors);
                let mismatches = push_type_mismatch_if_needed(expected_ty, &inferred);
                if !mismatches.is_empty() {
                    eprintln!("[DEBUG return_record] caller={caller:?} field={field:?} expected_ty={expected_ty:?} inferred={inferred:?}");
                }
                errors.extend(mismatches);
            }
        }
    }
    errors
}

fn infer_expr_type_for_expected_named_record(
    expr: &Expr,
    expected_type: &str,
    local_bindings: &HashMap<String, ValueType>,
    infer_context: &ExprInferenceContext<'_>,
) -> (ValueType, Vec<TypeError>) {
    let Expr::Record(None, fields) = expr else {
        return infer_expr_type(expr, local_bindings, infer_context);
    };

    let Some(expected_fields) =
        resolve_record_fields(expected_type, infer_context.record_type_registry)
    else {
        return infer_expr_type(expr, local_bindings, infer_context);
    };

    let mut errors = Vec::new();
    let mut inferred_fields = HashMap::new();
    let mut compatible = true;
    for (name, value_expr) in fields {
        let (inferred, val_errors) = infer_expr_type(value_expr, local_bindings, infer_context);
        errors.extend(val_errors);
        let inferred_name = inferred.display_name().unwrap_or_else(|| "Any".to_string());
        inferred_fields.insert(name.clone(), inferred_name.clone());
        let Some(expected_field_ty) = expected_fields.get(name) else {
            errors.push(TypeError::NoSuchField {
                ty: expected_type.to_string(),
                field: name.clone(),
            });
            compatible = false;
            continue;
        };
        if !gunbc_ir::type_registry::TypeRegistry::with_core_types()
            .is_compatible(&normalize_type_id(&inferred_name), &normalize_type_id(expected_field_ty))
        {
            eprintln!("[DEBUG field_mismatch] expected_type={expected_type:?} field={name:?} expected_field_ty={expected_field_ty:?} inferred={inferred_name:?}");
            errors.push(TypeError::TypeMismatch {
                expected: expected_field_ty.clone(),
                got: inferred_name,
            });
            compatible = false;
        }
    }

    let value = if compatible {
        ValueType::Named(expected_type.to_string())
    } else {
        ValueType::Record(inferred_fields)
    };
    (value, errors)
}

fn infer_expr_type(
    expr: &Expr,
    local_bindings: &HashMap<String, ValueType>,
    infer_context: &ExprInferenceContext<'_>,
) -> (ValueType, Vec<TypeError>) {
    let mut errors = Vec::new();
    let value = match expr {
        Expr::Literal(literal) => match literal {
            daglang_syntax::ast::Literal::Int(_) => ValueType::Named("Int".to_string()),
            daglang_syntax::ast::Literal::Float(_) => ValueType::Named("Float".to_string()),
            daglang_syntax::ast::Literal::String(_) => ValueType::Named("String".to_string()),
            daglang_syntax::ast::Literal::Bool(_) => ValueType::Named("Bool".to_string()),
            daglang_syntax::ast::Literal::None => ValueType::Named("Unit".to_string()),
        },
        Expr::Ident(name) => local_bindings
            .get(name)
            .cloned()
            .or_else(|| {
                infer_context
                    .param_callable_contracts
                    .get(name)
                    .filter(|contract| callable_contract_max_arity(contract) == 0)
                    .map(|contract| contract.output.clone())
            })
            .or_else(|| {
                infer_context
                    .callable_registry
                    .get(name)
                    .and_then(|entry| entry.as_ref())
                    .filter(|contract| callable_contract_max_arity(contract) == 0)
                    .map(|contract| contract.output.clone())
            })
            .unwrap_or(ValueType::Unknown),
        Expr::FieldAccess(base, field) => {
            let (base_type, base_errors) = infer_expr_type(base, local_bindings, infer_context);
            errors.extend(base_errors);
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
                ValueType::Named(name) => {
                    match resolve_record_fields(&name, infer_context.record_type_registry) {
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
                    }
                }
                ValueType::Unknown => ValueType::Unknown,
            }
        }
        Expr::Call(name, args) => {
            for (_, arg) in args {
                let (_, arg_errors) = infer_expr_type(arg, local_bindings, infer_context);
                errors.extend(arg_errors);
            }
            if let Some(contract) = infer_context.param_callable_contracts.get(name) {
                contract.output.clone()
            } else {
                infer_context
                    .callable_registry
                    .get(name)
                    .and_then(|entry| entry.as_ref())
                    .map(|contract| contract.output.clone())
                    .unwrap_or(ValueType::Unknown)
            }
        }
        Expr::ServiceCall(path, args) => {
            for (_, arg) in args {
                let (_, arg_errors) = infer_expr_type(arg, local_bindings, infer_context);
                errors.extend(arg_errors);
            }
            match resolve_service_call_contract(path, infer_context.service_call_registry) {
                ServiceCallResolution::Resolved(contract) => ValueType::Record(contract.outputs),
                ServiceCallResolution::Ambiguous => ValueType::Unknown,
                ServiceCallResolution::Missing => {
                    match resolve_bound_service_call_contract(
                        path,
                        infer_context.bound_service_registry,
                    ) {
                        BoundServiceCallResolution::Resolved(contract) => {
                            ValueType::Record(contract.outputs)
                        }
                        BoundServiceCallResolution::MissingCapability
                        | BoundServiceCallResolution::Deferred
                        | BoundServiceCallResolution::NotBound => ValueType::Unknown,
                    }
                }
            }
        }
        Expr::BinOp(lhs, op, rhs) => {
            let (lhs_ty, lhs_errors) = infer_expr_type(lhs, local_bindings, infer_context);
            errors.extend(lhs_errors);
            let (rhs_ty, rhs_errors) = infer_expr_type(rhs, local_bindings, infer_context);
            errors.extend(rhs_errors);
            match op {
                daglang_syntax::ast::BinOp::Eq
                | daglang_syntax::ast::BinOp::Ne
                | daglang_syntax::ast::BinOp::Lt
                | daglang_syntax::ast::BinOp::Gt
                | daglang_syntax::ast::BinOp::Le
                | daglang_syntax::ast::BinOp::Ge
                | daglang_syntax::ast::BinOp::And
                | daglang_syntax::ast::BinOp::Or => ValueType::Named("Bool".to_string()),
                daglang_syntax::ast::BinOp::NullCoalesce => lhs_ty,
                _ => match (lhs_ty, rhs_ty) {
                    (ValueType::Named(lhs), ValueType::Named(rhs))
                        if strip_generic_params(&lhs) == strip_generic_params(&rhs) =>
                    {
                        ValueType::Named(lhs)
                    }
                    _ => ValueType::Unknown,
                },
            }
        }
        Expr::UnaryOp(op, inner) => {
            let (inner_ty, inner_errors) = infer_expr_type(inner, local_bindings, infer_context);
            errors.extend(inner_errors);
            match op {
                daglang_syntax::ast::UnaryOp::Not => ValueType::Named("Bool".to_string()),
                daglang_syntax::ast::UnaryOp::Neg => inner_ty,
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    let (_, inner_errors) = infer_expr_type(inner, local_bindings, infer_context);
                    errors.extend(inner_errors);
                }
            }
            ValueType::Named("String".to_string())
        }
        Expr::Record(type_name, fields) => {
            if let Some(name) = type_name {
                for (_, value) in fields {
                    let (_, val_errors) = infer_expr_type(value, local_bindings, infer_context);
                    errors.extend(val_errors);
                }
                ValueType::Named(name.clone())
            } else {
                ValueType::Record(
                    fields
                        .iter()
                        .map(|(name, expr)| {
                            let (val, val_errors) =
                                infer_expr_type(expr, local_bindings, infer_context);
                            errors.extend(val_errors);
                            (
                                name.clone(),
                                val.display_name().unwrap_or_else(|| "Any".to_string()),
                            )
                        })
                        .collect(),
                )
            }
        }
        Expr::Match(scrutinee, arms) => {
            let (_, scr_errors) = infer_expr_type(scrutinee, local_bindings, infer_context);
            errors.extend(scr_errors);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    let (_, guard_errors) = infer_expr_type(guard, local_bindings, infer_context);
                    errors.extend(guard_errors);
                }
                let (_, body_errors) = infer_expr_type(&arm.body, local_bindings, infer_context);
                errors.extend(body_errors);
            }
            ValueType::Unknown
        }
        Expr::If(cond, then_expr, else_expr) => {
            let (_, cond_errors) = infer_expr_type(cond, local_bindings, infer_context);
            errors.extend(cond_errors);
            let (then_ty, then_errors) = infer_expr_type(then_expr, local_bindings, infer_context);
            errors.extend(then_errors);
            let else_ty = else_expr.as_ref().map(|otherwise| {
                let (ty, else_errors) = infer_expr_type(otherwise, local_bindings, infer_context);
                errors.extend(else_errors);
                ty
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
        Expr::For(binding, iterable, passthrough, body) => {
            let (_, iter_errors) = infer_expr_type(iterable, local_bindings, infer_context);
            errors.extend(iter_errors);
            let mut loop_scope = local_bindings.clone();
            // Element type inference is not modeled yet; make loop binding available in body.
            loop_scope.insert(binding.clone(), ValueType::Unknown);
            for name in passthrough {
                let passthrough_ty = local_bindings
                    .get(name)
                    .cloned()
                    .unwrap_or(ValueType::Unknown);
                loop_scope.insert(name.clone(), passthrough_ty);
            }
            let (_, body_errors) = infer_expr_type(body, &loop_scope, infer_context);
            errors.extend(body_errors);
            ValueType::Unknown
        }
        Expr::Pipe(lhs, rhs) => {
            let (_, lhs_errors) = infer_expr_type(lhs, local_bindings, infer_context);
            errors.extend(lhs_errors);
            let (rhs_val, rhs_errors) = infer_expr_type(rhs, local_bindings, infer_context);
            errors.extend(rhs_errors);
            rhs_val
        }
        Expr::Lambda(_, body) => {
            let (val, body_errors) = infer_expr_type(body, local_bindings, infer_context);
            errors.extend(body_errors);
            val
        }
        Expr::List(items) => {
            for item in items {
                let (_, item_errors) = infer_expr_type(item, local_bindings, infer_context);
                errors.extend(item_errors);
            }
            ValueType::Named("List".to_string())
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                let (_, key_errors) = infer_expr_type(key, local_bindings, infer_context);
                errors.extend(key_errors);
                let (_, val_errors) = infer_expr_type(value, local_bindings, infer_context);
                errors.extend(val_errors);
            }
            ValueType::Named("Map".to_string())
        }
        Expr::Guarded(inner, guard) => {
            let (_, inner_errors) = infer_expr_type(inner, local_bindings, infer_context);
            errors.extend(inner_errors);
            let (_, guard_errors) = infer_expr_type(guard, local_bindings, infer_context);
            errors.extend(guard_errors);
            ValueType::Unknown
        }
        Expr::After(inner, _) => {
            let (val, inner_errors) = infer_expr_type(inner, local_bindings, infer_context);
            errors.extend(inner_errors);
            val
        }
        Expr::Return(fields) => ValueType::Record(
            fields
                .iter()
                .map(|(name, expr)| {
                    let (val, val_errors) = infer_expr_type(expr, local_bindings, infer_context);
                    errors.extend(val_errors);
                    (
                        name.clone(),
                        val.display_name().unwrap_or_else(|| "Any".to_string()),
                    )
                })
                .collect(),
        ),
    };
    (value, errors)
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

fn push_type_mismatch_if_needed(expected: &str, inferred: &ValueType) -> Vec<TypeError> {
    let Some(got) = inferred.display_name() else {
        return Vec::new();
    };
    if !gunbc_ir::type_registry::TypeRegistry::with_core_types()
        .is_compatible(&normalize_type_id(&got), &normalize_type_id(expected))
    {
        eprintln!("[DEBUG type_mismatch] expected={expected:?} got={got:?} inferred={inferred:?} backtrace:");
        // Print a short backtrace to identify call site
        let bt = std::backtrace::Backtrace::force_capture();
        let bt_str = format!("{bt}");
        for line in bt_str.lines().take(20) {
            eprintln!("  {line}");
        }
        vec![TypeError::TypeMismatch {
            expected: expected.to_string(),
            got,
        }]
    } else {
        Vec::new()
    }
}

/// Normalize a DSL type name to a `TypeId` by stripping generic parameters
/// and module-qualified prefixes.
fn normalize_type_id(name: &str) -> gunbc_ir::TypeId {
    let base = name.split('<').next().unwrap_or(name).trim();
    let short = base.rsplit('.').next().unwrap_or(base);
    gunbc_ir::TypeId::from(short)
}

/// Strip generic parameters from a type name (e.g., `List<String>` → `List`).
fn strip_generic_params(name: &str) -> &str {
    name.split('<').next().unwrap_or(name).trim()
}

fn resolve_record_fields(
    ty: &str,
    registry: &RecordTypeRegistry,
) -> Option<HashMap<String, String>> {
    let canonical = strip_generic_params(ty).to_string();
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
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let Some(implemented) = resource.implements.as_deref() else {
        return errors;
    };
    let interface_contract = match resolve_interface_contract(implemented, interface_registry) {
        InterfaceResolution::Resolved(contract) => contract,
        InterfaceResolution::Ambiguous => {
            errors.push(TypeError::AmbiguousInterface {
                implementor: resource.name.clone(),
                interface: implemented.to_string(),
            });
            return errors;
        }
        InterfaceResolution::Missing => {
            errors.push(TypeError::UnresolvedInterface {
                implementor: resource.name.clone(),
                interface: implemented.to_string(),
            });
            return errors;
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
    for (capability_name, required_contract) in &interface_contract.capabilities {
        let Some(provided_contract) = provided_capabilities.get(capability_name) else {
            errors.push(TypeError::MissingCapability {
                resource: resource.name.clone(),
                interface: interface_name.clone(),
                capability: capability_name.clone(),
            });
            continue;
        };
        errors.extend(validate_capability_contract(
            &resource.name,
            &interface_name,
            capability_name,
            provided_contract,
            required_contract,
            &interface_contract.type_params,
        ));
    }
    errors
}

fn validate_capability_contract(
    implementor: &str,
    interface: &str,
    capability: &str,
    provided: &CapabilityContract,
    required: &CapabilityContract,
    generic_params: &[String],
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    errors.extend(validate_signature_map(
        implementor,
        interface,
        capability,
        "input",
        &provided.inputs,
        &required.inputs,
        generic_params,
    ));
    errors.extend(validate_signature_map(
        implementor,
        interface,
        capability,
        "output",
        &provided.outputs,
        &required.outputs,
        generic_params,
    ));
    errors
}

fn validate_signature_map(
    implementor: &str,
    interface: &str,
    capability: &str,
    direction: &str,
    provided: &HashMap<String, String>,
    required: &HashMap<String, String>,
    generic_params: &[String],
) -> Vec<TypeError> {
    let mut errors = Vec::new();
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
        if generic_params
            .iter()
            .any(|generic| expected_ty == generic || expected_ty.contains(generic))
        {
            continue;
        }
        let stripped_provided = provided_ty.split(" @").next().unwrap_or(provided_ty).trim();
        let stripped_expected = expected_ty.split(" @").next().unwrap_or(expected_ty).trim();
        if stripped_provided != stripped_expected {
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
    errors
}

fn validate_service_interface_conformance(
    service: &daglang_syntax::ast::ServiceDef,
    interface_registry: &InterfaceRegistry,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    let Some(implemented) = service.implements.as_deref() else {
        return errors;
    };
    let interface_contract = match resolve_interface_contract(implemented, interface_registry) {
        InterfaceResolution::Resolved(contract) => contract,
        InterfaceResolution::Ambiguous => {
            errors.push(TypeError::AmbiguousInterface {
                implementor: service.name.clone(),
                interface: implemented.to_string(),
            });
            return errors;
        }
        InterfaceResolution::Missing => {
            errors.push(TypeError::UnresolvedInterface {
                implementor: service.name.clone(),
                interface: implemented.to_string(),
            });
            return errors;
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
    for (capability_name, required_contract) in &interface_contract.capabilities {
        let Some(provided_contract) = provided_operations.get(capability_name) else {
            errors.push(TypeError::MissingOperation {
                service: service.name.clone(),
                interface: interface_name.clone(),
                operation: capability_name.clone(),
            });
            continue;
        };
        errors.extend(validate_capability_contract(
            &service.name,
            &interface_name,
            capability_name,
            provided_contract,
            required_contract,
            &interface_contract.type_params,
        ));
    }
    errors
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
    let canonical = strip_generic_params(implemented).to_string();
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
    strip_generic_params(name).to_string()
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
    let Some(keys) = service_call_lookup_keys(call_path) else {
        return ServiceCallResolution::Missing;
    };
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

fn build_bound_service_call_registry(
    uses: &[UsesClause],
    body_context: &BodyInferenceContext<'_>,
) -> BoundServiceCallRegistry {
    let mut registry = BoundServiceCallRegistry::default();
    for usage in uses {
        let resource_type = resource_type_name(&usage.resource_type);
        let binding =
            match resolve_resource_type_name(&resource_type, body_context.resource_type_registry) {
                ResourceTypeResolution::Resolved(resolved_type) => {
                    if let Some(interface_contract) =
                        body_context.interface_registry.full.get(&resolved_type)
                    {
                        let capabilities = interface_contract
                            .capabilities
                            .iter()
                            .map(|(name, contract)| {
                                (
                                    name.clone(),
                                    ServiceCallContract {
                                        arity: contract.inputs.len(),
                                        params: contract.inputs.keys().cloned().collect(),
                                        outputs: contract.outputs.clone(),
                                    },
                                )
                            })
                            .collect::<HashMap<_, _>>();
                        BoundServiceCallBinding::Resolved(capabilities)
                    } else if let Some(resource_capabilities) = body_context
                        .resource_capability_registry
                        .full
                        .get(&resolved_type)
                    {
                        let capabilities = resource_capabilities
                            .iter()
                            .map(|(name, contract)| {
                                (
                                    name.clone(),
                                    ServiceCallContract {
                                        arity: contract.inputs.len(),
                                        params: contract.inputs.keys().cloned().collect(),
                                        outputs: contract.outputs.clone(),
                                    },
                                )
                            })
                            .collect::<HashMap<_, _>>();
                        BoundServiceCallBinding::Resolved(capabilities)
                    } else {
                        BoundServiceCallBinding::Deferred
                    }
                }
                ResourceTypeResolution::Ambiguous | ResourceTypeResolution::Missing => {
                    BoundServiceCallBinding::Deferred
                }
            };
        registry.by_binding.insert(usage.binding.clone(), binding);
    }
    registry
}

fn resolve_bound_service_call_contract(
    call_path: &[String],
    registry: &BoundServiceCallRegistry,
) -> BoundServiceCallResolution {
    if call_path.len() != 2 {
        return BoundServiceCallResolution::NotBound;
    }
    let binding = &call_path[0];
    let capability = &call_path[1];
    let Some(binding_contracts) = registry.by_binding.get(binding) else {
        return BoundServiceCallResolution::NotBound;
    };
    match binding_contracts {
        BoundServiceCallBinding::Resolved(capabilities) => capabilities
            .get(capability)
            .cloned()
            .map(BoundServiceCallResolution::Resolved)
            .unwrap_or(BoundServiceCallResolution::MissingCapability),
        BoundServiceCallBinding::Deferred => BoundServiceCallResolution::Deferred,
    }
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
    walk_stmts(stmts, &mut |expr| {
        if let Expr::Call(name, args) = expr {
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
        }
    });
}

fn collect_service_calls_from_stmts(stmts: &[Stmt], calls: &mut Vec<BodyServiceCall>) {
    walk_stmts(stmts, &mut |expr| {
        if let Expr::ServiceCall(path, args) = expr {
            calls.push(BodyServiceCall {
                path: path.clone(),
                arg_count: args.len(),
                named_args: args
                    .iter()
                    .filter_map(|(name, _)| name.clone())
                    .collect::<Vec<_>>(),
            });
        }
    });
}

fn validate_type_expr(
    ty: &TypeExpr,
    known_types: &HashSet<String>,
    generic_arity_registry: &GenericArityRegistry,
    context: &str,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
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
            if let Some(expected) = resolve_generic_arity(name, generic_arity_registry, known_types)
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
                errors.extend(validate_type_expr(
                    arg,
                    known_types,
                    generic_arity_registry,
                    context,
                ));
            }
        }
        TypeExpr::Optional(inner) => {
            errors.extend(validate_type_expr(
                inner,
                known_types,
                generic_arity_registry,
                context,
            ));
        }
                    "content" | "brand" | "non_empty" | "pattern" | "file_types" => {
                        match process_supported_annotation(annotation) {
                            Ok(processed) => match processed {
                                ProcessedAnnotation::PredicateContent(encoding) => {
                                    debug_assert!(!encoding.is_empty());
                                }
                                ProcessedAnnotation::TypeOpBrand(brand) => {
                                    debug_assert!(!brand.is_empty());
                                }
                                ProcessedAnnotation::PredicateNonEmpty => {}
                                ProcessedAnnotation::PredicateMatches(regex) => {
                                    debug_assert!(!regex.is_empty());
                                }
                                ProcessedAnnotation::FileTypes(mapping) => {
                                    if mapping.is_empty() {
                                        errors.push(TypeError::UnsatisfiableRefinement {
                                            ty: type_expr_to_string(inner),
                                            constraint: "@file_types must define at least one extension or default mapping".to_string(),
                                        });
                                    }
                                }
                            },
                            Err(constraint) => {
                                errors.push(TypeError::UnsatisfiableRefinement {
                                    ty: type_expr_to_string(inner),
                                    constraint,
                                });
                            }
                        }
                    }
                    // Known-but-deferred: these annotations are valid but not yet
                    // lowered to TypeOp. They will be migrated in future modeling
                    // phases (M8 + MetadataPayload extensions for AccessMode/ServiceMode).
                    "json" | "format" | "invariant" | "contract"
                    | "tool" | "mode" => {}
                    other => {
                        errors.push(TypeError::UnknownAnnotation {
                            context: context.to_string(),
                            annotation: other.to_string(),
                        });
                    }
                }
            }
        }
        TypeExpr::Refined(inner, refinements) => {
            errors.extend(validate_type_expr(
                inner,
                known_types,
                generic_arity_registry,
                context,
            ));
            for refinement in refinements {
                match refinement {
                    Refinement::Range { min, max } => {
                        let min_val = min.as_ref().and_then(extract_int_literal);
                        let max_val = max.as_ref().and_then(extract_int_literal);
                        if let (Some(mn), Some(mx)) = (min_val, max_val) {
                            if mn > mx {
                                errors.push(TypeError::UnsatisfiableRefinement {
                                    ty: type_expr_to_string(inner),
                                    constraint: format!("range min {mn} exceeds max {mx}"),
                                });
                            }
                        }
                    }
                    Refinement::Content(enc) => {
                        if canonical_content_encoding(enc).is_none() {
                            errors.push(TypeError::UnsatisfiableRefinement {
                                ty: type_expr_to_string(inner),
                                constraint: format!(
                                    "unknown content encoding `{enc}` — expected one of: Text, UTF8, ASCII, Latin1, Binary, Unknown"
                                ),
                            });
                        }
                    }
                    Refinement::Brand(name) => {
                        if name.trim().is_empty() {
                            errors.push(TypeError::UnsatisfiableRefinement {
                                ty: type_expr_to_string(inner),
                                constraint: "brand requires a non-empty name".to_string(),
                            });
                        }
                    }
                    Refinement::Pattern(regex) => {
                        if regex.trim().is_empty() {
                            errors.push(TypeError::UnsatisfiableRefinement {
                                ty: type_expr_to_string(inner),
                                constraint: "pattern requires a non-empty regex".to_string(),
                            });
                        }
                    }
                    Refinement::NonEmpty
                    | Refinement::Format(_)
                    | Refinement::Predicate(_)
                    | Refinement::RawBody
                    | Refinement::FileTypes(_) => {}
                }
            }
        }
        TypeExpr::Record(fields) => {
            for field in fields {
                errors.extend(validate_type_expr(
                    &field.ty,
                    known_types,
                    generic_arity_registry,
                    &format!("{context}.{}", field.name),
                ));
            }
        }
    }
    errors
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

/// Extract a string value from an expression (string literal or identifier).
fn expr_as_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(Literal::String(s)) => Some(s.clone()),
        Expr::Ident(name) => Some(name.clone()),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProcessedAnnotation {
    PredicateContent(String),
    TypeOpBrand(String),
    PredicateNonEmpty,
    PredicateMatches(String),
    FileTypes(BTreeMap<String, String>),
}



fn expr_as_string_list(expr: &Expr) -> Result<Vec<String>, String> {
    let Expr::List(items) = expr else {
        return Err("expected list of string extensions".to_string());
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let value = expr_as_string(item)
            .ok_or_else(|| "file extension list items must be strings/identifiers".to_string())?;
        out.push(value);
    }
    Ok(out)
}

fn validate_file_extension(ext: &str) -> Result<(), String> {
    if !ext.starts_with('.') || ext.trim().len() < 2 {
        return Err(format!(
            "invalid file extension `{ext}` — expected dot-prefixed suffix like `.rs`"
        ));
    }
    Ok(())
}

fn canonical_content_encoding(raw: &str) -> Option<String> {
    match raw {
        "Text" | "UTF8" | "ASCII" | "Latin1" | "Binary" | "Unknown" => {
            Some(raw.to_string())
        }
        _ => None,
    }
}


fn extract_int_literal(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Literal(Literal::Int(value)) => Some(*value),
        _ => None,
    }
}

fn should_validate_named_type(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '.'))
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

    fn ann(name: &str, args: Vec<Expr>) -> Annotation {
        Annotation {
            name: name.to_string(),
            args,
        }
    }

    #[test]
    fn process_supported_annotation_content_maps_to_predicate_content() {
        let processed = process_supported_annotation(&ann("content", vec![Expr::Ident("UTF8".into())]))
            .expect("content annotation should process");
        assert_eq!(
            processed,
            ProcessedAnnotation::PredicateContent("UTF8".to_string())
        );
    }

    #[test]
    fn process_supported_annotation_brand_requires_name() {
        let err = process_supported_annotation(&ann("brand", vec![]))
            .expect_err("brand without name should fail");
        assert!(err.contains("@brand requires exactly one name argument"));
    }

    #[test]
    fn process_supported_annotation_non_empty_rejects_args() {
        let err = process_supported_annotation(&ann(
            "non_empty",
            vec![Expr::Literal(Literal::String("oops".into()))],
        ))
        .expect_err("non_empty should reject arguments");
        assert!(err.contains("@non_empty does not accept arguments"));
    }

    #[test]
    fn process_supported_annotation_file_types_maps_extensions() {
        let processed = process_supported_annotation(&ann(
            "file_types",
            vec![Expr::Record(
                None,
                vec![
                    (
                        "text".to_string(),
                        Expr::List(vec![Expr::Literal(Literal::String(".rs".into()))]),
                    ),
                    (
                        "binary".to_string(),
                        Expr::List(vec![Expr::Literal(Literal::String(".png".into()))]),
                    ),
                    ("default".to_string(), Expr::Ident("Binary".into())),
                ],
            )],
        ))
        .expect("file_types should process");

        let ProcessedAnnotation::FileTypes(mapping) = processed else {
            panic!("expected file_types annotation mapping");
        };
        assert_eq!(mapping.get(".rs").map(String::as_str), Some("Text"));
        assert_eq!(mapping.get(".png").map(String::as_str), Some("Binary"));
        assert_eq!(mapping.get("*").map(String::as_str), Some("Binary"));
    }

    #[test]
    fn process_supported_annotation_file_types_rejects_non_dot_extensions() {
        let err = process_supported_annotation(&ann(
            "file_types",
            vec![Expr::Record(
                None,
                vec![(
                    "text".to_string(),
                    Expr::List(vec![Expr::Literal(Literal::String("rs".into()))]),
                )],
            )],
        ))
        .expect_err("invalid extension should fail");
        assert!(err.contains("invalid file extension"));
    }

    // Test infrastructure: filesystem access for test fixtures
    #[allow(clippy::disallowed_methods)]
    #[test]
    fn typecheck_accepts_makegen_module() {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl/tools/makegen.dag");
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
        let errors = typecheck_module_graph(graph).expect_err("duplicate outputs should fail");
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
        assert!(errors.iter().any(
            |error| matches!(error, TypeError::UndefinedType(msg) if msg.contains("MissingType"))
        ));
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
    fn duplicate_interface_definition_is_reported_and_causes_ambiguous_implements() {
        let graph = module_graph_from_sources(&[(
            "duplicate_interface_definition.dag",
            r#"module sample.dup
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("duplicate interface definitions should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.dup" && name == "Storage"
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousInterface { implementor, interface }
                if implementor == "FsStorage" && interface == "Storage"
        )));
    }

    #[test]
    fn strict_mode_duplicate_interface_definition_also_reports_ambiguous_implements() {
        let graph = module_graph_from_sources(&[(
            "duplicate_interface_definition_strict.dag",
            r#"module sample.dup
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
"#,
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for duplicate interface definitions");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.dup" && name == "Storage"
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousInterface { implementor, interface }
                if implementor == "FsStorage" && interface == "Storage"
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
    fn call_with_too_many_args_is_reported() {
        let graph = module_graph_from_sources(&[(
            "arity_overflow.dag",
            "module sample.calls\nfn fmt(value: String) -> String { value }\nfn run() -> String { fmt(\"a\", \"b\") }",
        )]);
        let errors = typecheck_module_graph(graph).expect_err("too many call args should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::CallArityMismatch {
                caller,
                callee,
                expected,
                got
            } if caller == "run" && callee == "fmt" && *expected == 1 && *got == 2
        )));
    }

    #[test]
    fn strict_mode_allows_call_omitting_defaulted_params() {
        let graph = module_graph_from_sources(&[(
            "defaulted_call.dag",
            r#"module sample.calls
fn greet(name: String, punctuation: String = "!") -> String {
  name
}
fn run() -> String {
  greet(name: "hi")
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("defaulted callable params should be optional at call sites");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_allows_pattern_calls_with_extra_wiring_named_args() {
        let graph = module_graph_from_sources(&[(
            "pattern_wiring_call.dag",
            r#"module sample.calls
pattern ensure(should_act: Bool = true) -> { acted: Bool } {
  return { acted: should_act }
}
fn run() -> Bool {
  let result = ensure(check: true, action: false)
  result.acted
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("pattern calls should allow extra named wiring arguments");
        assert_eq!(typed.modules.len(), 1);
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
    fn strict_mode_duplicate_callable_definition_also_reports_ambiguous_call_target() {
        let graph = module_graph_from_sources(&[(
            "sample/main.dag",
            r#"module sample.main
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }"#,
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for duplicate callable definition");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.main" && name == "helper"
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousCallTarget { caller, callee }
                if caller == "run" && callee == "helper"
        )));
    }

    #[test]
    fn relaxed_mode_duplicate_callable_definition_suppresses_ambiguous_call_target() {
        let graph = module_graph_from_sources(&[(
            "sample/single.dag",
            r#"module sample.single
fn helper() -> String { "a" }
fn helper() -> String { "b" }
fn run() -> String { helper() }"#,
        )]);
        let errors = typecheck_module_graph(graph)
            .expect_err("relaxed mode should still fail for duplicate callable definition");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.single" && name == "helper"
        )));
        assert!(!errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousCallTarget { caller, callee }
                if caller == "run" && callee == "helper"
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
    fn strict_mode_accepts_collection_intrinsic_call_targets() {
        let graph = module_graph_from_sources(&[(
            "sample/intrinsics.dag",
            r#"module sample.intrinsics
type Stage {
  success: Bool,
  skipped: Bool,
  name: String
}
fn summarize(stages: List<Stage>) -> Int {
  let passed = stages |> filter(s => s.success) |> count()
  let labels = stages |> map(s => s.name) |> join(",")
  let done = labels |> ends_with("ok")
  passed
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("collection intrinsics should be recognized in strict mode");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_accepts_std_helper_intrinsic_call_targets() {
        let graph = module_graph_from_sources(&[(
            "sample/helpers.dag",
            r#"module sample.helpers
type DocgenSources {}

fn run(sources: DocgenSources, payload: String) -> String {
  let a = "template" |> replace_section("section", "value")
  let b = render_test_listings(sources: sources)
  let c = render_graph_structure(sources: sources)
  let d = render_source_artifacts(sources: sources)
  let e = compute_topology_diff(current: "{}", base: "{}")
  let f = render_annotated_mermaid(diff: e, topology: "{}", title: "title")
  let g = detect_runtime()
  let h = generate()
  let i = now()
  let j = build_token(
    payload: payload,
    scheme: "Bearer",
    header_name: "Authorization",
    source_id: "source",
    required_scopes: ["gist"]
  )
  a
}
"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("std helper intrinsics should be recognized in strict mode");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_accepts_generic_fn_type_params() {
        let graph = module_graph_from_sources(&[(
            "sample/generic_fn.dag",
            r#"module sample.generic
fn identity<T>(value: T) -> T {
  value
}
fn relay<T>(value: T) -> T {
  identity(value: value)
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("generic fn type parameters should be treated as known types");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_accepts_generic_pattern_type_params() {
        let graph = module_graph_from_sources(&[(
            "sample/generic_pattern.dag",
            r#"module sample.generic
pattern passthrough<T: Serializable>(value: T) -> { value: T } {
  return { value: value }
}
fn relay<T>(value: T) -> T {
  let result = passthrough(value: value)
  result.value
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("generic pattern type parameters should be treated as known types");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_accepts_untyped_record_literal_for_named_return() {
        let graph = module_graph_from_sources(&[(
            "sample/records.dag",
            r#"module sample.records
type StageResult {
  success: Bool,
  skipped: Bool
}
fn result() -> StageResult {
  { success: true, skipped: false }
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("record literals should satisfy named-record return contracts");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_accepts_resource_config_named_type_returns() {
        let graph = module_graph_from_sources(&[(
            "sample/resources.dag",
            r#"module sample.resources
resource GcsBucket {
  config {
    name: String,
    project: String
  }
}
fn gcp_dev_storage() -> GcsBucket.Config {
  { name: "gunbc-dev-artifacts", project: "gunbai-auto" }
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("resource config named types should be recognized in strict mode");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_accepts_secret_builtin_type() {
        let graph = module_graph_from_sources(&[(
            "sample/secret.dag",
            r#"module sample.secret
fn identity(value: Secret) -> Secret {
  value
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("Secret should be recognized as builtin type");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_accepts_function_typed_parameter_call_targets() {
        let graph = module_graph_from_sources(&[(
            "sample/callback.dag",
            r#"module sample.callback
fn apply(value: Int, callback: fn(Int) -> Int) -> Int {
  callback(value)
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("function-typed parameters should be callable in strict mode");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn function_typed_parameter_call_reports_arity_mismatch() {
        let graph = module_graph_from_sources(&[(
            "sample/callback_arity.dag",
            r#"module sample.callback
fn apply(callback: fn(Int) -> Int) -> Int {
  callback()
}"#,
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("callback calls should enforce function-type arity");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::CallArityMismatch {
                caller,
                callee,
                expected,
                got
            } if caller == "apply" && callee == "callback" && *expected == 1 && *got == 0
        )));
    }

    #[test]
    fn strict_mode_accepts_sum_variant_constructor_call_targets() {
        let graph = module_graph_from_sources(&[(
            "sample/constructors.dag",
            r#"module sample.constructors
type CloudConfig
  = GcpConfig { project: String, region: String }
  | AwsConfig { region: String }

fn make_gcp() -> CloudConfig {
  GcpConfig(project: "gunbc", region: "us-central1")
}
"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("sum variant constructors should resolve as callable targets");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn sum_variant_constructor_call_reports_arity_mismatch() {
        let graph = module_graph_from_sources(&[(
            "sample/constructor_arity.dag",
            r#"module sample.constructors
type CloudConfig
  = GcpConfig { project: String, region: String }

fn make_gcp() -> CloudConfig {
  GcpConfig(project: "gunbc")
}
"#,
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("variant constructor calls should enforce field arity");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::CallArityMismatch {
                caller,
                callee,
                expected,
                got
            } if caller == "make_gcp" && callee == "GcpConfig" && *expected == 2 && *got == 1
        )));
    }

    #[test]
    fn strict_mode_accepts_zero_arity_variant_as_identifier_value() {
        let graph = module_graph_from_sources(&[(
            "sample/variant_ident.dag",
            r#"module sample.variant
type Environment = Dev | Ci
fn env() -> Environment {
  Dev
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("zero-arity variants should be inferred from identifier expressions");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_skips_missing_tail_mismatch_for_lossy_fn_body() {
        let graph = module_graph_from_sources(&[(
            "sample/lossy_match.dag",
            r#"module sample.lossy
type CloudConfig
  = GcpConfig { project: String }
  | AwsConfig { account: String }
type CloudProvider = Gcp | Aws

fn provider_of(config: CloudConfig) -> CloudProvider {
  match config {
    GcpConfig { ... } => Gcp
    AwsConfig { ... } => Aws
  }
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("lossy fn bodies should not emit missing-tail unit mismatch diagnostics");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn unknown_named_call_argument_is_reported() {
        let graph = module_graph_from_sources(&[(
            "unknown_arg.dag",
            "module sample.calls\nfn fmt(value: String) -> String { value }\nfn run() -> String { fmt(text: \"ok\") }",
        )]);
        let errors = typecheck_module_graph(graph).expect_err("unknown named argument should fail");
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
    fn service_call_with_too_many_args_is_reported() {
        let graph = module_graph_from_sources(&[(
            "service_arity_overflow.dag",
            r#"module sample.services
service FsStorage {
  operation read(path: String) -> { body: String }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read(path: "/tmp", extra: "value")
  return { ok: true }
}"#,
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("too many service call arguments should fail");
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
                && *got == 2
        )));
    }

    #[test]
    fn strict_mode_allows_service_call_omitting_defaulted_inputs() {
        let graph = module_graph_from_sources(&[(
            "service_default_input.dag",
            r#"module sample.services
service FsStorage {
  operation read(path: String, recursive: Bool = false) -> { ok: Bool }
}
func run() -> { ok: Bool } {
  let response = FsStorage.read(path: "/tmp")
  return { ok: response.ok }
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("service inputs with defaults should be optional at call sites");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn uses_bound_resource_capability_call_typechecks_and_infers_outputs() {
        let graph = module_graph_from_sources(&[(
            "resource_bound_service_call.dag",
            r#"module sample.resources
resource Filesystem {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("resource-bound capability calls should typecheck in strict mode");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn uses_bound_resource_capability_call_reports_arity_mismatch() {
        let graph = module_graph_from_sources(&[(
            "resource_bound_service_call_arity.dag",
            r#"module sample.resources
resource Filesystem {
  capability read {
    input { path: String }
    output { body: String }
  }
}
func run() -> { ok: Bool } uses fs: Filesystem {
  let response = fs.read()
  return { ok: true }
}"#,
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("resource-bound capability calls should enforce arity");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::ServiceCallArityMismatch {
                caller,
                service_call,
                expected,
                got
            } if caller == "run"
                && service_call == "fs.read"
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
    fn strict_mode_duplicate_service_definition_also_reports_ambiguous_service_call() {
        let graph = module_graph_from_sources(&[(
            "sample/main.dag",
            r#"module sample.main
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}"#,
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for duplicate service definition");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.main" && name == "FsStorage"
        )));
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousServiceCall {
                caller,
                service_call
            } if caller == "run" && service_call == "FsStorage.read"
        )));
    }

    #[test]
    fn relaxed_mode_duplicate_service_definition_suppresses_ambiguous_service_call() {
        let graph = module_graph_from_sources(&[(
            "sample/single.dag",
            r#"module sample.single
interface Storage {
  capability read {
    input { path: String }
    output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}"#,
        )]);
        let errors = typecheck_module_graph(graph)
            .expect_err("relaxed mode should still fail for duplicate service definition");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.single" && name == "FsStorage"
        )));
        assert!(!errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousServiceCall {
                caller,
                service_call
            } if caller == "run" && service_call == "FsStorage.read"
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
        let errors = typecheck_module_graph(graph).expect_err("ambiguous interface should fail");
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
        let errors = typecheck_module_graph(graph).expect_err("ambiguous interface should fail");
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
    fn strict_mode_accepts_uses_resource_type_with_runtime_config_suffix() {
        let graph = module_graph_from_sources(&[(
            "sample/main.dag",
            r#"module sample.main
resource Filesystem {}
func run() -> { ok: Bool } uses fs: Filesystem(mode: ReadWrite) {
  return { ok: true }
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("configured resource type should resolve in strict mode");
        assert_eq!(typed.modules.len(), 1);
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
    fn strict_mode_duplicate_resource_definition_also_reports_ambiguous_used_resource_type() {
        let graph = module_graph_from_sources(&[(
            "sample/main.dag",
            r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource {
  return { ok: true }
}"#,
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for duplicate resource definitions");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.main" && name == "SharedResource"
        )));
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
    fn relaxed_mode_duplicate_resource_definition_suppresses_ambiguous_used_resource_type() {
        let graph = module_graph_from_sources(&[(
            "sample/single.dag",
            r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } uses fs: SharedResource {
  return { ok: true }
}"#,
        )]);
        let errors = typecheck_module_graph(graph)
            .expect_err("relaxed mode should still fail for duplicate resource definition");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.single" && name == "SharedResource"
        )));
        assert!(!errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousUsedResourceType {
                item,
                binding,
                resource_type,
            } if item == "run" && binding == "fs" && resource_type == "SharedResource"
        )));
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
    fn strict_mode_accepts_provides_resource_type_with_runtime_config_suffix() {
        let graph = module_graph_from_sources(&[(
            "sample/main.dag",
            r#"module sample.main
resource ArtifactStore {}
func run() -> { ok: Bool } provides out: ArtifactStore(kind: temporary) {
  return { ok: true }
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("configured provided resource type should resolve in strict mode");
        assert_eq!(typed.modules.len(), 1);
    }

    #[test]
    fn strict_mode_accepts_annotated_provides_resource_type_reference() {
        let graph = module_graph_from_sources(&[(
            "sample/main.dag",
            r#"module sample.main
resource ArtifactStore {}
func run() -> { ok: Bool } provides out: ArtifactStore @test_integration {
  return { ok: true }
}"#,
        )]);
        let typed = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("annotated provided resource type should resolve in strict mode");
        assert_eq!(typed.modules.len(), 1);
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
    fn relaxed_mode_duplicate_resource_definition_suppresses_ambiguous_provided_resource_type() {
        let graph = module_graph_from_sources(&[(
            "sample/single.dag",
            r#"module sample.single
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource {
  return { ok: true }
}"#,
        )]);
        let errors = typecheck_module_graph(graph)
            .expect_err("relaxed mode should still fail for duplicate resource definition");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.single" && name == "SharedResource"
        )));
        assert!(!errors.iter().any(|error| matches!(
            error,
            TypeError::AmbiguousProvidedResourceType {
                item,
                binding,
                resource_type,
            } if item == "run" && binding == "out" && resource_type == "SharedResource"
        )));
    }

    #[test]
    fn strict_mode_duplicate_resource_definition_also_reports_ambiguous_provided_resource_type() {
        let graph = module_graph_from_sources(&[(
            "sample/main.dag",
            r#"module sample.main
resource SharedResource {}
resource SharedResource {}
func run() -> { ok: Bool } provides out: SharedResource {
  return { ok: true }
}"#,
        )]);
        let errors = typecheck_module_graph_with_options(
            graph,
            TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect_err("strict mode should fail for duplicate resource definitions");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicateDefinition { module, name }
                if module == "sample.main" && name == "SharedResource"
        )));
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
    fn implicit_type_mismatch_in_fn_return_is_reported() {
        let graph = module_graph_from_sources(&[(
            "implicit_type_mismatch_fn_return.dag",
            r#"module sample.types
fn run() -> String { 42 }"#,
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("implicit return type mismatch should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::TypeMismatch { expected, got }
                if expected == "String" && got == "Int"
        )));
    }

    #[test]
    fn missing_tail_expression_in_fn_return_is_reported_as_unit_mismatch() {
        let graph = module_graph_from_sources(&[(
            "missing_tail_expression_fn_return.dag",
            r#"module sample.types
fn run() -> String { let x = 42 }"#,
        )]);
        let errors = typecheck_module_graph(graph)
            .expect_err("missing tail expression should fail for non-unit return type");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::TypeMismatch { expected, got }
                if expected == "String" && got == "Unit"
        )));
    }

    #[test]
    fn missing_tail_expression_is_allowed_for_unit_return_type() {
        let graph = module_graph_from_sources(&[(
            "missing_tail_expression_unit_return.dag",
            r#"module sample.types
fn run() -> Unit { let x = 42 }"#,
        )]);
        let typed = typecheck_module_graph(graph)
            .expect("unit return type should allow no tail expression");
        assert_eq!(typed.modules.len(), 1);
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

    #[test]
    fn pipeline_unknown_stage_dependency_is_reported() {
        let graph = module_graph_from_sources(&[(
            "pipeline_unknown_dep.dag",
            r#"module sample.pipeline
pipeline ci {
  stage build [after missing] {}
}"#,
        )]);
        let errors =
            typecheck_module_graph(graph).expect_err("unknown pipeline stage dependency should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::UnknownPipelineStageDependency {
                pipeline,
                stage,
                dependency,
            } if pipeline == "ci" && stage == "build" && dependency == "missing"
        )));
    }

    #[test]
    fn duplicate_pipeline_stage_is_reported() {
        let graph = module_graph_from_sources(&[(
            "pipeline_duplicate_stage.dag",
            r#"module sample.pipeline
pipeline ci {
  stage build {}
  stage build {}
}"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("duplicate pipeline stage should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::DuplicatePipelineStage { pipeline, stage }
                if pipeline == "ci" && stage == "build"
        )));
    }

    #[test]
    fn pipeline_when_condition_must_be_bool() {
        let graph = module_graph_from_sources(&[(
            "pipeline_when_type_mismatch.dag",
            r#"module sample.pipeline
pipeline ci {
  stage build [when 42] {}
}"#,
        )]);
        let errors = typecheck_module_graph(graph).expect_err("non-bool when should fail");
        assert!(errors.iter().any(|error| matches!(
            error,
            TypeError::PipelineStageWhenTypeMismatch { pipeline, stage, got }
                if pipeline == "ci" && stage == "build" && got == "Int"
        )));
    }
}
