//! **Stage 3 — Typecheck**: Transforms a `ModuleGraph` into a
//! `TypedProject` (typed AST + type registry).
//!
//! # Pipeline position
//!
//! - **Before**: [`daglang-resolve`] has built a dependency-ordered `ModuleGraph`
//! - **After**: [`daglang-lower`] lowers the typed AST to `Dag<LoweredOp>`
//!
//! # Sequential steps
//!
//! 1. Validate record and sum type definitions
//! 2. Check refinement constraints (`@range`, `@pattern`, etc.)
//! 3. Instantiate generic types (`List<T>`, `Map<K,V>`, `Queue<T>`)
//! 4. Verify interface conformance (`resource X implements Y`)
//! 5. Resolve `contract` declarations and subtyping via bounded lattice
//! 6. Produce `TypedProject` with fully resolved type information
//!
//! # Purity
//!
//! Pure — no side effects. Operates entirely on the in-memory module graph.
//!
//! # Failure
//!
//! Returns `Vec<TypeError>` wrapped in `Verdict<TypedProject>` when
//! validation fails.

use std::collections::{HashMap, HashSet};

use daglang_contract::{Diagnostic, DiagnosticContext, FileId, LocatedSpan};
use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_syntax::ast::{
    Expr, Field, ForBody, Item, Literal, ModulePath, Param, PipelineDef, ProvidesClause,
    Refinement, Stmt, TypeBody, TypeExpr, UsesClause,
};
use daglang_syntax::ast_utils::{
    resource_type_name, service_call_lookup_keys, type_expr_to_string, walk_stmts,
    walk_stmts_with_expr_identities, ExprIdentity,
};
use gunbc_ir::{TypeRegistry, BUILTIN_TYPES};

/// A typechecked project snapshot over a resolved module graph.
#[derive(Debug)]
pub struct TypedProject<'a> {
    graph: TypedProjectGraph<'a>,
    typed_modules: Vec<TypedModule>,
    pipeline_params: Vec<PipelineParam>,
    dsl_type_registry: TypeRegistry,
    available_profiles: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TypecheckOptions {
    pub allow_unresolved_imports: bool,
}

/// A typechecked module.
#[derive(Debug)]
pub struct TypedModule {
    pub graph_index: usize,
    pub signatures: Vec<TypedItemSignature>,
    callable_body_metadata: HashMap<String, TypedCallableBodyMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineParam {
    pub name: String,
    pub type_id: String,
    pub default_value: Option<String>,
}

#[derive(Debug)]
enum TypedProjectGraph<'a> {
    Borrowed(&'a ModuleGraph),
    Owned(ModuleGraph),
}

#[derive(Debug, Clone, Copy)]
pub struct TypedModuleRef<'project, 'graph> {
    resolved: &'project ResolvedModule,
    pub graph_index: usize,
    pub signatures: &'project [TypedItemSignature],
    callable_body_metadata: &'project HashMap<String, TypedCallableBodyMetadata>,
    _graph: std::marker::PhantomData<&'graph ModuleGraph>,
}

impl<'a> TypedProject<'a> {
    pub fn graph(&self) -> &ModuleGraph {
        match &self.graph {
            TypedProjectGraph::Borrowed(graph) => graph,
            TypedProjectGraph::Owned(graph) => graph,
        }
    }

    pub fn module_count(&self) -> usize {
        self.typed_modules.len()
    }

    pub fn pipeline_params(&self) -> &[PipelineParam] {
        &self.pipeline_params
    }

    pub fn dsl_type_registry(&self) -> &TypeRegistry {
        &self.dsl_type_registry
    }

    pub fn available_profiles(&self) -> &[String] {
        &self.available_profiles
    }

    pub fn modules(&self) -> impl ExactSizeIterator<Item = TypedModuleRef<'_, 'a>> + '_ {
        self.typed_modules.iter().map(move |typed| TypedModuleRef {
            resolved: &self.graph().modules[typed.graph_index],
            graph_index: typed.graph_index,
            signatures: &typed.signatures,
            callable_body_metadata: &typed.callable_body_metadata,
            _graph: std::marker::PhantomData,
        })
    }

    pub fn module(&self, index: usize) -> Option<TypedModuleRef<'_, 'a>> {
        self.typed_modules.get(index).map(|typed| TypedModuleRef {
            resolved: &self.graph().modules[typed.graph_index],
            graph_index: typed.graph_index,
            signatures: &typed.signatures,
            callable_body_metadata: &typed.callable_body_metadata,
            _graph: std::marker::PhantomData,
        })
    }
}

impl<'project, 'graph> TypedModuleRef<'project, 'graph> {
    pub fn imports(&self) -> impl Iterator<Item = &'project ModulePath> + 'project {
        self.resolved
            .ast
            .imports
            .iter()
            .map(|import| &import.node.path)
    }

    pub fn callable_body_metadata(&self, name: &str) -> Option<&TypedCallableBodyMetadata> {
        self.callable_body_metadata.get(name)
    }
}

impl<'project, 'graph> std::ops::Deref for TypedModuleRef<'project, 'graph> {
    type Target = ResolvedModule;

    fn deref(&self) -> &Self::Target {
        self.resolved
    }
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

/// A normalized callable signature for fn/func/pattern/extern items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedCallableSignature {
    pub name: String,
    pub params: Vec<TypedBinding>,
    pub outputs: Vec<TypedBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesizedAnonymousRecordType {
    pub name: gunbc_ir::types::TypeId,
    pub fields: Vec<(String, gunbc_ir::code_ir::IrType)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedForLoopBinding {
    pub name: String,
    pub ir_type: gunbc_ir::code_ir::IrType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedForLoopScope {
    element_binding: TypedForLoopBinding,
    passthrough_bindings: Vec<TypedForLoopBinding>,
}

impl TypedForLoopScope {
    pub fn binding_ir_type(&self, name: &str) -> Option<&gunbc_ir::code_ir::IrType> {
        if self.element_binding.name == name {
            return Some(&self.element_binding.ir_type);
        }
        self.passthrough_bindings
            .iter()
            .find(|binding| binding.name == name)
            .map(|binding| &binding.ir_type)
    }

    fn resolved_bindings(
        &self,
        outer_bindings: &HashMap<String, gunbc_ir::code_ir::IrType>,
    ) -> HashMap<String, gunbc_ir::code_ir::IrType> {
        let mut bindings = outer_bindings.clone();
        bindings.remove(&self.element_binding.name);
        bindings.insert(
            self.element_binding.name.clone(),
            self.element_binding.ir_type.clone(),
        );
        for binding in &self.passthrough_bindings {
            bindings.insert(binding.name.clone(), binding.ir_type.clone());
        }
        bindings
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ForLoopScopeValueBinding {
    name: String,
    value_type: ValueType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ForLoopScopeContract {
    element_binding: ForLoopScopeValueBinding,
    passthrough_bindings: Vec<ForLoopScopeValueBinding>,
}

impl ForLoopScopeContract {
    fn local_bindings(
        &self,
        outer_bindings: &HashMap<String, ValueType>,
    ) -> HashMap<String, ValueType> {
        let mut bindings = outer_bindings.clone();
        bindings.remove(&self.element_binding.name);
        bindings.insert(
            self.element_binding.name.clone(),
            self.element_binding.value_type.clone(),
        );
        for binding in &self.passthrough_bindings {
            bindings.insert(binding.name.clone(), binding.value_type.clone());
        }
        bindings
    }

    fn typed_scope(
        &self,
        iterable_ir_type: &gunbc_ir::code_ir::IrType,
        resolved_bindings: &HashMap<String, gunbc_ir::code_ir::IrType>,
    ) -> TypedForLoopScope {
        let element_ir_type = for_loop_element_ir_type(iterable_ir_type)
            .unwrap_or_else(|| value_type_to_ir_type(&self.element_binding.value_type));
        let passthrough_bindings = self
            .passthrough_bindings
            .iter()
            .map(|binding| TypedForLoopBinding {
                name: binding.name.clone(),
                ir_type: resolved_bindings
                    .get(&binding.name)
                    .cloned()
                    .unwrap_or_else(|| value_type_to_ir_type(&binding.value_type)),
            })
            .collect();

        TypedForLoopScope {
            element_binding: TypedForLoopBinding {
                name: self.element_binding.name.clone(),
                ir_type: element_ir_type,
            },
            passthrough_bindings,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypedCallableBodyMetadata {
    anonymous_record_targets: HashMap<ExprIdentity, gunbc_ir::types::TypeId>,
    anonymous_record_field_types: HashMap<ExprIdentity, Vec<(String, gunbc_ir::code_ir::IrType)>>,
    conflicted_anonymous_records: HashSet<ExprIdentity>,
    synthesized_anonymous_record_types: Vec<SynthesizedAnonymousRecordType>,
    expr_ir_types: HashMap<ExprIdentity, gunbc_ir::code_ir::IrType>,
    for_loop_scopes: HashMap<ExprIdentity, TypedForLoopScope>,
}

impl TypedCallableBodyMetadata {
    pub fn anonymous_record_target(
        &self,
        expr_identity: ExprIdentity,
    ) -> Option<&gunbc_ir::types::TypeId> {
        if self.conflicted_anonymous_records.contains(&expr_identity) {
            return None;
        }
        self.anonymous_record_targets.get(&expr_identity)
    }

    pub fn anonymous_record_targets(
        &self,
    ) -> impl Iterator<Item = (ExprIdentity, &gunbc_ir::types::TypeId)> + '_ {
        self.anonymous_record_targets
            .iter()
            .filter(|(expr_identity, _)| !self.conflicted_anonymous_records.contains(expr_identity))
            .map(|(expr_identity, target)| (*expr_identity, target))
    }

    pub fn synthesized_anonymous_record_types(&self) -> &[SynthesizedAnonymousRecordType] {
        &self.synthesized_anonymous_record_types
    }

    pub fn expr_ir_types(
        &self,
    ) -> impl Iterator<Item = (ExprIdentity, &gunbc_ir::code_ir::IrType)> + '_ {
        self.expr_ir_types
            .iter()
            .map(|(expr_identity, ir_type)| (*expr_identity, ir_type))
    }

    pub fn for_loop_scope(&self, expr_identity: ExprIdentity) -> Option<&TypedForLoopScope> {
        self.for_loop_scopes.get(&expr_identity)
    }

    fn is_empty(&self) -> bool {
        self.anonymous_record_targets.is_empty()
            && self.conflicted_anonymous_records.is_empty()
            && self.synthesized_anonymous_record_types.is_empty()
            && self.expr_ir_types.is_empty()
            && self.for_loop_scopes.is_empty()
    }

    fn annotate_anonymous_record_target(&mut self, expr_identity: ExprIdentity, target: &str) {
        let target = gunbc_ir::types::TypeId::from(target);
        if self.conflicted_anonymous_records.contains(&expr_identity) {
            return;
        }
        match self.anonymous_record_targets.get(&expr_identity) {
            Some(existing) if existing != &target => {
                self.anonymous_record_targets.remove(&expr_identity);
                self.conflicted_anonymous_records.insert(expr_identity);
            }
            Some(_) => {}
            None => {
                self.anonymous_record_targets.insert(expr_identity, target);
            }
        }
    }

    fn annotate_anonymous_record_field_types(
        &mut self,
        expr_identity: ExprIdentity,
        fields: &HashMap<String, ValueType>,
    ) {
        let mut next_fields = fields
            .iter()
            .map(|(name, ty)| (name.clone(), value_type_to_ir_type(ty)))
            .collect::<Vec<_>>();
        next_fields.sort_by(|left, right| left.0.cmp(&right.0));
        self.anonymous_record_field_types
            .entry(expr_identity)
            .and_modify(|existing| merge_record_ir_fields(existing, &next_fields))
            .or_insert(next_fields);
    }

    fn finalize_anonymous_record_types(&mut self, callable_name: &str) {
        let mut unresolved_expr_ids = self
            .anonymous_record_field_types
            .keys()
            .copied()
            .filter(|expr_identity| {
                !self.conflicted_anonymous_records.contains(expr_identity)
                    && !self.anonymous_record_targets.contains_key(expr_identity)
            })
            .collect::<Vec<_>>();
        unresolved_expr_ids.sort();

        let mut typed_shape_names: HashMap<Vec<(String, String)>, String> = HashMap::new();
        let mut synthesized_types = Vec::new();
        let pascal_callable =
            capitalize_first_char(&callable_name.replace('_', " ")).replace(' ', "");

        for expr_identity in unresolved_expr_ids {
            let Some(field_types) = self.anonymous_record_field_types.get(&expr_identity) else {
                continue;
            };
            let Some(typed_shape) = field_types
                .iter()
                .map(|(field_name, ir_type)| {
                    Some((field_name.clone(), ir_type_to_type_id(ir_type)?))
                })
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let mut typed_shape = typed_shape;
            typed_shape.sort_by(|left, right| left.0.cmp(&right.0));

            let type_name = if let Some(existing) = typed_shape_names.get(&typed_shape) {
                existing.clone()
            } else {
                let next_name = if typed_shape_names.is_empty() {
                    format!("__{pascal_callable}State")
                } else {
                    format!("__{pascal_callable}State{}", typed_shape_names.len())
                };
                typed_shape_names.insert(typed_shape, next_name.clone());
                synthesized_types.push(SynthesizedAnonymousRecordType {
                    name: gunbc_ir::types::TypeId::from(next_name.clone()),
                    fields: field_types.clone(),
                });
                next_name
            };
            self.anonymous_record_targets
                .insert(expr_identity, gunbc_ir::types::TypeId::from(type_name));
        }

        self.synthesized_anonymous_record_types = synthesized_types;
    }

    fn annotate_expr_ir_type(
        &mut self,
        expr_identity: ExprIdentity,
        ir_type: gunbc_ir::code_ir::IrType,
    ) {
        self.expr_ir_types.insert(expr_identity, ir_type);
    }

    fn annotate_for_loop_scope(
        &mut self,
        expr_identity: ExprIdentity,
        scope: TypedForLoopScope,
    ) {
        self.for_loop_scopes.insert(expr_identity, scope);
    }
}

/// A single typed binding in a callable signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedBinding {
    pub name: String,
    pub ty: gunbc_ir::types::TypeId,
}

/// Errors during type checking.
#[derive(Debug)]
pub enum TypeError {
    /// A type name was used but not defined.
    UndefinedType(String),
    /// A type expression could not be resolved into a supported registry shape.
    UnresolvableType { ty: String, context: String },
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
    /// Service config declares an unrecognized auth scheme.
    InvalidAuthScheme { service: String, scheme: String },
    /// if/else branches produce incompatible types.
    BranchTypeMismatch {
        then_type: String,
        else_type: String,
    },
    /// Match arms produce incompatible types.
    MatchArmTypeMismatch {
        first_type: String,
        mismatched_type: String,
    },
    /// Match on a sum type does not cover all variants.
    NonExhaustiveMatch {
        scrutinee_type: String,
        missing_variants: Vec<String>,
    },
    /// `with(iterable, { binding: binding })` references a missing outer binding.
    UnknownForLoopPassthroughBinding { binding: String },
}

/// Non-fatal diagnostic emitted when the typechecker encounters an
/// expression whose type cannot be fully determined but is not an error.
///
/// Warnings are collected alongside `TypeError`s and surfaced to the
/// caller for reporting. They do NOT block compilation.
#[derive(Debug)]
pub enum TypecheckWarning {
    /// The typechecker could not infer a concrete type for an expression
    /// and fell back to `Inferred` (treated as compatible with any type).
    InferredType { context: String, hint: String },
}

impl std::fmt::Display for TypecheckWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InferredType { context, hint } => {
                write!(f, "type not inferred for {context}: {hint}")
            }
        }
    }
}

/// A type error enriched with source location information.
///
/// Wraps `TypeError` with mandatory source identity for user-facing diagnostics.
/// The driver uses this to populate `Diagnostic.file` and resolve `line:col` from
/// `ResolvedModule.source`.
#[derive(Debug)]
pub struct SpannedTypeError {
    pub error: TypeError,
    pub file_id: FileId,
    pub file: std::path::PathBuf,
    pub module: String,
    /// Byte offset span of the AST item that caused this error.
    pub span: daglang_syntax::span::Span,
}

impl SpannedTypeError {
    fn primary_span(&self) -> LocatedSpan {
        LocatedSpan {
            file: self.file_id,
            span: daglang_contract::Span {
                start: self.span.start,
                end: self.span.end,
            },
            label: "here".to_string(),
        }
    }

    /// Convert to a `Diagnostic` with file location.
    pub fn to_diagnostic(&self) -> daglang_contract::Diagnostic {
        self.error
            .to_located_diagnostic(self.primary_span())
            .with_file(self.file.clone())
    }

    /// Convert to a `Diagnostic` with file and resolved line:col.
    pub fn to_diagnostic_with_source(&self, source: &str) -> daglang_contract::Diagnostic {
        let (line, col) = daglang_contract::byte_to_line_col(source, self.span.start);
        self.to_diagnostic().with_line_col(line, col)
    }
}

impl TypeError {
    /// Stable, grep-able error code for this variant (CP-59).
    pub fn code(&self) -> &'static str {
        match self {
            Self::UndefinedType(..) => "TC001",
            Self::NoSuchField { .. } => "TC002",
            Self::TypeMismatch { .. } => "TC003",
            Self::MissingCapability { .. } => "TC004",
            Self::UnsatisfiableRefinement { .. } => "TC005",
            Self::ArityMismatch { .. } => "TC006",
            Self::DuplicateDefinition { .. } => "TC007",
            Self::DuplicatePipelineStage { .. } => "TC008",
            Self::DuplicatePipelineStageDependency { .. } => "TC009",
            Self::UnknownPipelineStageDependency { .. } => "TC010",
            Self::PipelineStageSelfDependency { .. } => "TC011",
            Self::PipelineStageWhenTypeMismatch { .. } => "TC012",
            Self::DuplicateParameter { .. } => "TC013",
            Self::DuplicateOutputField { .. } => "TC014",
            Self::UnresolvedImport { .. } => "TC015",
            Self::UnresolvedInterface { .. } => "TC016",
            Self::AmbiguousInterface { .. } => "TC017",
            Self::MissingOperation { .. } => "TC018",
            Self::InterfaceSignatureMismatch { .. } => "TC019",
            Self::CallArityMismatch { .. } => "TC020",
            Self::UnknownCallArgument { .. } => "TC021",
            Self::DuplicateCallArgument { .. } => "TC022",
            Self::AmbiguousCallTarget { .. } => "TC023",
            Self::UnresolvedCallTarget { .. } => "TC024",
            Self::ServiceCallArityMismatch { .. } => "TC025",
            Self::UnresolvedServiceCall { .. } => "TC026",
            Self::AmbiguousServiceCall { .. } => "TC027",
            Self::UnknownServiceCallArgument { .. } => "TC028",
            Self::DuplicateServiceCallArgument { .. } => "TC029",
            Self::UnknownUsedResourceType { .. } => "TC030",
            Self::AmbiguousUsedResourceType { .. } => "TC031",
            Self::DuplicateUsesBinding { .. } => "TC032",
            Self::DuplicateProvidesBinding { .. } => "TC033",
            Self::UseProvideBindingConflict { .. } => "TC034",
            Self::UnknownProvidedResourceType { .. } => "TC035",
            Self::AmbiguousProvidedResourceType { .. } => "TC036",
            Self::InvalidAuthScheme { .. } => "TC037",
            Self::BranchTypeMismatch { .. } => "TC038",
            Self::MatchArmTypeMismatch { .. } => "TC039",
            Self::NonExhaustiveMatch { .. } => "TC040",
            Self::UnresolvableType { .. } => "TC041",
            Self::UnknownForLoopPassthroughBinding { .. } => "TC042",
        }
    }

    /// Help text with fix suggestions for common errors (CP-50).
    pub fn help(&self) -> Option<String> {
        match self {
            Self::UndefinedType(name) => Some(format!(
                "check spelling of `{name}` — common types: String, Int, Bool, List<T>, Map<K,V>, Option<T>"
            )),
            Self::UnresolvableType { ty, .. } => Some(format!(
                "ensure `{ty}` is defined and uses supported shapes; maps must be `Map<String, T>`"
            )),
            Self::TypeMismatch { expected, got } => Some(format!(
                "change argument type to `{expected}` or add a conversion from `{got}`"
            )),
            Self::ArityMismatch {
                name,
                expected,
                got,
            } => Some(format!(
                "`{name}` expects {expected} type parameter(s), got {got}"
            )),
            Self::UnresolvedImport { target, .. } => Some(format!(
                "`{target}` not found — check the module path and ensure the .dag file exists"
            )),
            Self::UnresolvedCallTarget { callee, .. } => Some(format!(
                "`{callee}` is not defined — check spelling or add an import"
            )),
            Self::CallArityMismatch {
                callee,
                expected,
                got,
                ..
            } => Some(format!(
                "`{callee}` expects {expected} argument(s), got {got} — check parameter names"
            )),
            Self::UnknownCallArgument { argument, callee, .. } => Some(format!(
                "remove `{argument}:` from call to `{callee}` or check parameter names"
            )),
            Self::DuplicateDefinition { name, .. } => Some(format!(
                "rename one of the `{name}` definitions — each name must be unique within a module"
            )),
            Self::InvalidAuthScheme { scheme, .. } => Some(format!(
                "change `{scheme}` to one of: BearerToken, Basic, ApiKey, Header(\"...\"), None"
            )),
            Self::UnresolvedServiceCall { service_call, .. } => Some(format!(
                "`{service_call}` not found — check service import and operation name"
            )),
            Self::NoSuchField { ty, field } => Some(format!(
                "type `{ty}` has no field `{field}` — check field names in the type definition"
            )),
            Self::BranchTypeMismatch { then_type, else_type } => Some(format!(
                "if/else branches must produce the same type — `{then_type}` vs `{else_type}`"
            )),
            Self::MatchArmTypeMismatch { first_type, mismatched_type } => Some(format!(
                "all match arms must produce the same type — first arm is `{first_type}`, found `{mismatched_type}`"
            )),
            Self::NonExhaustiveMatch { missing_variants, .. } => Some(format!(
                "add arms for: {} — or add a `_ => ...` wildcard arm",
                missing_variants.join(", ")
            )),
            Self::UnknownForLoopPassthroughBinding { binding } => Some(format!(
                "define `{binding}` before the loop or remove it from `with(iterable, {{ ... }})`"
            )),
            _ => None,
        }
    }

    /// Convert to the shared compiler diagnostic shape.
    pub fn to_diagnostic(&self) -> Diagnostic {
        let mut diagnostic =
            Diagnostic::new(self.code(), self.to_string()).with_context(self.diagnostic_context());
        if let Some(help) = self.help() {
            diagnostic = diagnostic.with_help(help);
        }
        diagnostic
    }

    /// Convert to the shared compiler diagnostic shape with mandatory source location.
    pub fn to_located_diagnostic(&self, primary: LocatedSpan) -> Diagnostic {
        let mut diagnostic = Diagnostic::located(self.code(), self.to_string(), primary)
            .with_context(self.diagnostic_context());
        if let Some(help) = self.help() {
            diagnostic = diagnostic.with_help(help);
        }
        diagnostic
    }

    fn diagnostic_context(&self) -> DiagnosticContext {
        match self {
            Self::TypeMismatch { expected, got } => DiagnosticContext::TypeMismatch {
                expected: expected.clone(),
                got: got.clone(),
            },
            Self::UndefinedType(name)
            | Self::UnresolvableType { ty: name, .. }
            | Self::UnresolvedImport { target: name, .. }
            | Self::UnresolvedInterface {
                interface: name, ..
            }
            | Self::UnresolvedCallTarget { callee: name, .. }
            | Self::UnresolvedServiceCall {
                service_call: name, ..
            }
            | Self::UnknownCallArgument { argument: name, .. }
            | Self::UnknownServiceCallArgument { argument: name, .. }
            | Self::UnknownUsedResourceType { binding: name, .. }
            | Self::UnknownProvidedResourceType { binding: name, .. }
            | Self::UnknownForLoopPassthroughBinding { binding: name } => {
                DiagnosticContext::Missing {
                    kind: "declaration",
                    name: name.clone(),
                    available: Vec::new(),
                }
            }
            Self::DuplicateDefinition { name, .. }
            | Self::DuplicatePipelineStage { stage: name, .. }
            | Self::DuplicatePipelineStageDependency {
                dependency: name, ..
            }
            | Self::DuplicateParameter { param: name, .. }
            | Self::DuplicateOutputField { field: name, .. }
            | Self::DuplicateCallArgument { argument: name, .. }
            | Self::DuplicateServiceCallArgument { argument: name, .. }
            | Self::DuplicateUsesBinding { binding: name, .. }
            | Self::DuplicateProvidesBinding { binding: name, .. } => {
                DiagnosticContext::Duplicate {
                    name: name.clone(),
                    first: None,
                }
            }
            _ => DiagnosticContext::Note(String::new()),
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UndefinedType(name) => write!(f, "undefined type `{name}`"),
            Self::UnresolvableType { ty, context } => {
                write!(f, "type `{ty}` cannot be resolved in `{context}`")
            }
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
            ),
            Self::InvalidAuthScheme { service, scheme } => write!(
                f,
                "service `{service}` declares unknown auth scheme `{scheme}` \
                 (valid: BearerToken, Basic, ApiKey, Header(\"...\"), None)"
            ),
            Self::BranchTypeMismatch {
                then_type,
                else_type,
            } => write!(
                f,
                "if/else branch type mismatch: then-branch is `{then_type}`, else-branch is `{else_type}`"
            ),
            Self::MatchArmTypeMismatch {
                first_type,
                mismatched_type,
            } => write!(
                f,
                "match arm type mismatch: first arm is `{first_type}`, found arm with `{mismatched_type}`"
            ),
            Self::NonExhaustiveMatch {
                scrutinee_type,
                missing_variants,
            } => write!(
                f,
                "non-exhaustive match on `{scrutinee_type}`: missing {}",
                missing_variants.join(", ")
            ),
            Self::UnknownForLoopPassthroughBinding { binding } => {
                write!(f, "unknown for-loop passthrough binding `{binding}`")
            }
        }
    }
}

/// Typecheck a discovered module graph and produce typed module signatures.
pub fn typecheck_module_graph<'a>(
    graph: &'a ModuleGraph,
) -> Result<TypedProject<'a>, Vec<TypeError>> {
    typecheck_module_graph_with_options(graph, TypecheckOptions::default())
}

/// Typecheck an owned module graph and retain it inside the typed overlay.
pub fn typecheck_owned_module_graph(
    graph: ModuleGraph,
) -> Result<TypedProject<'static>, Vec<TypeError>> {
    typecheck_owned_module_graph_with_options(graph, TypecheckOptions::default())
}

/// Typecheck an owned module graph with explicit options and retain it inside
/// the typed overlay.
pub fn typecheck_owned_module_graph_with_options(
    graph: ModuleGraph,
    options: TypecheckOptions,
) -> Result<TypedProject<'static>, Vec<TypeError>> {
    let metadata = typecheck_graph_modules(&graph, &options)?;
    Ok(TypedProject {
        graph: TypedProjectGraph::Owned(graph),
        typed_modules: metadata.typed_modules,
        pipeline_params: metadata.pipeline_params,
        dsl_type_registry: metadata.dsl_type_registry,
        available_profiles: metadata.available_profiles,
    })
}

/// Typecheck a discovered module graph with explicit options.
///
/// Borrows the graph (CP-43) — callers retain access after typechecking.
pub fn typecheck_module_graph_with_options<'a>(
    graph: &'a ModuleGraph,
    options: TypecheckOptions,
) -> Result<TypedProject<'a>, Vec<TypeError>> {
    let metadata = typecheck_graph_modules(graph, &options)?;
    Ok(TypedProject {
        graph: TypedProjectGraph::Borrowed(graph),
        typed_modules: metadata.typed_modules,
        pipeline_params: metadata.pipeline_params,
        dsl_type_registry: metadata.dsl_type_registry,
        available_profiles: metadata.available_profiles,
    })
}

/// Typecheck with source-located errors.
///
/// Returns `SpannedTypeError` on failure, which carries the file path and module
/// name for each error. Use `SpannedTypeError::to_diagnostic_with_source()` to
/// resolve byte offsets into line:col using `ResolvedModule.source`.
pub fn typecheck_module_graph_located<'a>(
    graph: &'a ModuleGraph,
    options: TypecheckOptions,
) -> Result<TypedProject<'a>, Vec<SpannedTypeError>> {
    let metadata = typecheck_graph_modules_spanned(graph, &options)?;
    Ok(TypedProject {
        graph: TypedProjectGraph::Borrowed(graph),
        typed_modules: metadata.typed_modules,
        pipeline_params: metadata.pipeline_params,
        dsl_type_registry: metadata.dsl_type_registry,
        available_profiles: metadata.available_profiles,
    })
}

struct TypedProjectMetadata {
    typed_modules: Vec<TypedModule>,
    pipeline_params: Vec<PipelineParam>,
    dsl_type_registry: TypeRegistry,
    available_profiles: Vec<String>,
}

fn typecheck_graph_modules(
    graph: &ModuleGraph,
    options: &TypecheckOptions,
) -> Result<TypedProjectMetadata, Vec<TypeError>> {
    typecheck_graph_modules_spanned(graph, options)
        .map_err(|spanned_errors| spanned_errors.into_iter().map(|se| se.error).collect())
}

fn typecheck_graph_modules_spanned(
    graph: &ModuleGraph,
    options: &TypecheckOptions,
) -> Result<TypedProjectMetadata, Vec<SpannedTypeError>> {
    let known_types = collect_known_types(&graph.modules);
    let generic_arity_registry = collect_generic_arities(&graph.modules);
    let record_type_registry = collect_record_types(&graph.modules);
    let variant_parents = collect_variant_parents(&graph.modules);
    let _sum_type_variants = collect_sum_type_variants(&graph.modules);
    let callable_registry = collect_unique_callables(&graph.modules);
    let pattern_callable_names = collect_pattern_callable_names(&graph.modules);
    let service_call_registry = collect_service_call_contracts(&graph.modules);
    let interface_registry = collect_interfaces(&graph.modules);
    let resource_type_registry = collect_resource_types(&graph.modules);
    let resource_capability_registry = collect_resource_capabilities(&graph.modules);
    let available_modules = graph
        .modules
        .iter()
        .map(|module| module.module_path.clone())
        .collect::<HashSet<ModulePath>>();
    let mut errors: Vec<SpannedTypeError> = Vec::new();
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
        variant_parents: &variant_parents,
        allow_unresolved_references: options.allow_unresolved_imports,
    };

    for (graph_index, module) in graph.modules.iter().enumerate() {
        let module_name = module.module_path.as_dotted();
        let module_file_id = file_id_for_graph_index(graph_index);
        let module_file = module.path.clone();
        if !options.allow_unresolved_imports {
            for import in &module.ast.imports {
                if !available_modules.contains(&import.node.path) {
                    errors.push(SpannedTypeError {
                        error: TypeError::UnresolvedImport {
                            module: module_name.clone(),
                            target: import.node.path.as_dotted(),
                        },
                        file_id: module_file_id,
                        file: module_file.clone(),
                        module: module_name.clone(),
                        span: import.span,
                    });
                }
            }
        }
        let (signatures, callable_body_metadata, sig_errors) =
            collect_signatures(module, module_file_id, &context, &module_name);
        errors.extend(sig_errors);
        typed_modules.push(TypedModule {
            graph_index,
            signatures,
            callable_body_metadata,
        });
    }

    let (dsl_type_registry, registry_errors) = collect_dsl_type_registry(&graph.modules);
    errors.extend(registry_errors);

    errors
        .is_empty()
        .then_some(TypedProjectMetadata {
            typed_modules,
            pipeline_params: collect_pipeline_params(&graph.modules),
            dsl_type_registry,
            available_profiles: collect_available_profiles(&graph.modules),
        })
        .ok_or(errors)
}

#[derive(Debug, Clone)]
struct RegistryUnresolved {
    ty: String,
    context: String,
}

#[derive(Debug, Clone)]
struct RegistryTypeDef<'a> {
    def: &'a daglang_syntax::ast::TypeDef,
    file_id: FileId,
    file: std::path::PathBuf,
    module: String,
    span: daglang_syntax::span::Span,
}

fn collect_pipeline_params(modules: &[ResolvedModule]) -> Vec<PipelineParam> {
    let mut params = Vec::new();
    for module in modules {
        for item in &module.ast.items {
            if let Item::ParamDecl(decl) = &item.node {
                params.push(PipelineParam {
                    name: decl.name.clone(),
                    type_id: type_expr_to_string(&decl.ty),
                    default_value: decl.default.as_ref().and_then(expr_to_default_string),
                });
            }
        }
    }
    params
}

fn collect_dsl_type_registry(modules: &[ResolvedModule]) -> (TypeRegistry, Vec<SpannedTypeError>) {
    let mut registry = TypeRegistry::with_defaults();
    let mut registry_errors = Vec::new();

    // Collect all type definitions across modules for two-pass registration.
    let mut all_type_defs: Vec<RegistryTypeDef<'_>> = Vec::new();
    for (graph_index, module) in modules.iter().enumerate() {
        for item in &module.ast.items {
            if let Item::TypeDef(def) = &item.node {
                all_type_defs.push(RegistryTypeDef {
                    def,
                    file_id: file_id_for_graph_index(graph_index),
                    file: module.path.clone(),
                    module: module.module_path.as_dotted(),
                    span: item.span,
                });
            }
        }
    }

    // Pass 1: Register every type name with an identity placeholder.
    // This ensures forward references resolve to a known type instead of
    // triggering the fallback code path.
    for def in &all_type_defs {
        registry.register(
            def.def.name.as_str(),
            gunbc_ir::type_lib::identity(&def.def.name),
        );
    }

    // Build a dependency graph for topological ordering (Pass 2).
    // Alias types depend on their base type; Record types depend on their
    // field types. Sorting ensures base types register before derived types.
    let type_names: std::collections::HashSet<&str> =
        all_type_defs.iter().map(|d| d.def.name.as_str()).collect();
    let mut deps: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for def in &all_type_defs {
        let mut my_deps = Vec::new();
        match &def.def.body {
            TypeBody::Alias(type_expr) => {
                collect_type_deps_from_expr(type_expr, &type_names, &mut my_deps);
            }
            TypeBody::Record(fields) => {
                for field in fields {
                    collect_type_deps_from_expr(&field.ty, &type_names, &mut my_deps);
                }
            }
            TypeBody::Sum(variants) => {
                for variant in variants {
                    for field in &variant.fields {
                        collect_type_deps_from_expr(&field.ty, &type_names, &mut my_deps);
                    }
                }
            }
        }
        deps.insert(def.def.name.as_str(), my_deps);
    }
    let plain_defs: Vec<&daglang_syntax::ast::TypeDef> =
        all_type_defs.iter().map(|d| d.def).collect();
    let type_info_by_name: std::collections::HashMap<&str, &RegistryTypeDef<'_>> = all_type_defs
        .iter()
        .map(|def| (def.def.name.as_str(), def))
        .collect();
    let ordered = topological_sort_types(&plain_defs, &deps);

    // Pass 2: Re-register with resolved structural DAGs in topological order.
    for def in ordered {
        let mut unresolved = Vec::new();
        register_type_def(def, &mut registry, &mut unresolved);
        if let Some(info) = type_info_by_name.get(def.name.as_str()) {
            registry_errors.extend(unresolved.into_iter().map(|issue| SpannedTypeError {
                error: TypeError::UnresolvableType {
                    ty: issue.ty,
                    context: issue.context,
                },
                file_id: info.file_id,
                file: info.file.clone(),
                module: info.module.clone(),
                span: info.span,
            }));
        }
    }

    // Pass 3: Register inline record types from fn/func parameter and return types.
    // These aren't top-level type definitions but appear as anonymous structural
    // types in callable signatures. Signature validation already reports bad
    // inner types, so this pass registers them best-effort without emitting
    // duplicate diagnostics.
    for module in modules {
        for item in &module.ast.items {
            match &item.node {
                Item::FnDef(def) => {
                    for param in &def.params {
                        register_inline_records(&param.ty, &mut registry);
                    }
                    register_inline_records(&def.return_type, &mut registry);
                }
                Item::FuncDef(def) => {
                    for param in &def.params {
                        register_inline_records(&param.ty, &mut registry);
                    }
                    for output in &def.outputs {
                        register_inline_records(&output.ty, &mut registry);
                    }
                }
                Item::PatternDef(def) => {
                    for param in &def.params {
                        register_inline_records(&param.ty, &mut registry);
                    }
                    for output in &def.outputs {
                        register_inline_records(&output.ty, &mut registry);
                    }
                }
                _ => {}
            }
        }
    }

    (registry, registry_errors)
}

/// Recursively walk a type expression and register any inline record types.
fn register_inline_records(ty: &TypeExpr, registry: &mut TypeRegistry) {
    match ty {
        TypeExpr::Record(_) => {
            let mut ignored = Vec::new();
            resolve_field_type_dag_or_identity(ty, registry, &mut ignored, "<inline record>");
        }
        TypeExpr::Generic(_, args) => {
            for arg in args {
                register_inline_records(arg, registry);
            }
        }
        TypeExpr::Function(params, output) => {
            for param in params {
                register_inline_records(param, registry);
            }
            register_inline_records(output, registry);
        }
        TypeExpr::Optional(inner) | TypeExpr::Refined(inner, _) => {
            register_inline_records(inner, registry);
        }
        TypeExpr::Named(_) | TypeExpr::AssociatedOutput(_) => {}
    }
}

/// Recursively extract dependency type names from a TypeExpr.
///
/// Descends into Generic args, Optional inners, and Refined bases to find
/// all referenced type names that exist in `type_names`. This ensures
/// dependencies like `List<Bit>` correctly extract `Bit` instead of the
/// stringified `"List<Bit>"`.
fn collect_type_deps_from_expr<'a>(
    expr: &TypeExpr,
    type_names: &std::collections::HashSet<&'a str>,
    deps: &mut Vec<&'a str>,
) {
    match expr {
        TypeExpr::Named(name) => {
            if let Some(&dep) = type_names.get(name.as_str()) {
                deps.push(dep);
            }
        }
        TypeExpr::AssociatedOutput(_) => {}
        TypeExpr::Generic(_, args) => {
            for arg in args {
                collect_type_deps_from_expr(arg, type_names, deps);
            }
        }
        TypeExpr::Function(params, output) => {
            for param in params {
                collect_type_deps_from_expr(param, type_names, deps);
            }
            collect_type_deps_from_expr(output, type_names, deps);
        }
        TypeExpr::Optional(inner) => {
            collect_type_deps_from_expr(inner, type_names, deps);
        }
        TypeExpr::Refined(inner, _) => {
            collect_type_deps_from_expr(inner, type_names, deps);
        }
        TypeExpr::Record(fields) => {
            for field in fields {
                collect_type_deps_from_expr(&field.ty, type_names, deps);
            }
        }
    }
}

/// Topological sort of type definitions. Falls back to original order for cycles.
fn topological_sort_types<'a>(
    defs: &[&'a daglang_syntax::ast::TypeDef],
    deps: &std::collections::HashMap<&str, Vec<&str>>,
) -> Vec<&'a daglang_syntax::ast::TypeDef> {
    let name_to_def: std::collections::HashMap<&str, &'a daglang_syntax::ast::TypeDef> =
        defs.iter().map(|d| (d.name.as_str(), *d)).collect();
    let mut visited = std::collections::HashSet::new();
    let mut in_stack = std::collections::HashSet::new();
    let mut result = Vec::new();

    fn visit<'a>(
        name: &str,
        name_to_def: &std::collections::HashMap<&str, &'a daglang_syntax::ast::TypeDef>,
        deps: &std::collections::HashMap<&str, Vec<&str>>,
        visited: &mut std::collections::HashSet<String>,
        in_stack: &mut std::collections::HashSet<String>,
        result: &mut Vec<&'a daglang_syntax::ast::TypeDef>,
    ) {
        if visited.contains(name) {
            return;
        }
        if in_stack.contains(name) {
            return; // cycle — skip to break infinite recursion
        }
        in_stack.insert(name.to_string());
        if let Some(dep_list) = deps.get(name) {
            for dep in dep_list {
                visit(dep, name_to_def, deps, visited, in_stack, result);
            }
        }
        in_stack.remove(name);
        visited.insert(name.to_string());
        if let Some(def) = name_to_def.get(name) {
            result.push(*def);
        }
    }

    for def in defs {
        visit(
            def.name.as_str(),
            &name_to_def,
            deps,
            &mut visited,
            &mut in_stack,
            &mut result,
        );
    }
    result
}

/// Register a single type definition into the registry (Pass 2 worker).
fn register_type_def(
    def: &daglang_syntax::ast::TypeDef,
    registry: &mut TypeRegistry,
    unresolved: &mut Vec<RegistryUnresolved>,
) {
    match &def.body {
        TypeBody::Sum(variants) => {
            // Unit variants get "Unit" type, payload variants get resolved field DAGs.
            let resolved_variants: Vec<(&str, gunbc_ir::Dag<gunbc_ir::type_op::TypeOp>)> = variants
                .iter()
                .map(|variant| {
                    let dag = if variant.fields.is_empty() {
                        gunbc_ir::type_lib::unit()
                    } else if variant.fields.len() == 1 {
                        resolve_field_type_dag_or_identity(
                            &variant.fields[0].ty,
                            registry,
                            unresolved,
                            format!(
                                "type {}.{}.{}",
                                def.name, variant.name, variant.fields[0].name
                            ),
                        )
                    } else {
                        // Multi-field payload variant: wrap fields as an anonymous product.
                        let resolved_fields: Vec<(&str, gunbc_ir::Dag<gunbc_ir::type_op::TypeOp>)> =
                            variant
                                .fields
                                .iter()
                                .map(|f| {
                                    (
                                        f.name.as_str(),
                                        resolve_field_type_dag_or_identity(
                                            &f.ty,
                                            registry,
                                            unresolved,
                                            format!(
                                                "type {}.{}.{}",
                                                def.name, variant.name, f.name
                                            ),
                                        ),
                                    )
                                })
                                .collect();
                        gunbc_ir::type_lib::product_resolved(&variant.name, resolved_fields)
                    };
                    (variant.name.as_str(), dag)
                })
                .collect();
            registry.register(
                def.name.as_str(),
                gunbc_ir::type_lib::coproduct_resolved(def.name.as_str(), resolved_variants),
            );
        }
        TypeBody::Record(fields) => {
            let resolved_fields: Vec<(&str, gunbc_ir::Dag<gunbc_ir::type_op::TypeOp>)> = fields
                .iter()
                .map(|field| {
                    let dag = resolve_field_type_dag_or_identity(
                        &field.ty,
                        registry,
                        unresolved,
                        format!("type {}.{}", def.name, field.name),
                    );
                    (field.name.as_str(), dag)
                })
                .collect();
            registry.register(
                def.name.as_str(),
                gunbc_ir::type_lib::product_resolved(def.name.as_str(), resolved_fields),
            );
        }
        TypeBody::Alias(type_expr) => {
            let base_name = type_expr_to_string(type_expr);
            let predicates = collect_predicates_from_type_expr(type_expr);
            let brand_name = collect_brand_from_type_expr(type_expr);
            let base_dag_opt = registry.get_by_name(&base_name).cloned();

            // Build the inner DAG — embed base if available so
            // structural predicates (width, domain, etc.) are inherited.
            let inner_dag = if predicates.is_empty() {
                match base_dag_opt {
                    Some(dag) => dag,
                    None => {
                        unresolved.push(RegistryUnresolved {
                            ty: base_name.clone(),
                            context: format!("type {}", def.name),
                        });
                        gunbc_ir::type_lib::identity(&base_name)
                    }
                }
            } else {
                match base_dag_opt {
                    Some(dag) => gunbc_ir::type_lib::refined_with_base(&base_name, dag, predicates),
                    None => gunbc_ir::type_lib::refined(&base_name, predicates),
                }
            };

            // Wrap in a Brand node if the alias carries a brand refinement
            let final_dag = if let Some(bname) = brand_name {
                gunbc_ir::type_lib::branded(&bname, inner_dag)
            } else {
                inner_dag
            };

            registry.register(def.name.as_str(), final_dag);
        }
    }
}

/// Build a type DAG for a field's TypeExpr, preserving refinement predicates.
///
/// Handles structural containers (List, Optional, Set, Map) by recursing into
/// their type arguments and building structural DAGs instead of flattening to
/// identity strings. Returns `Err(type_name)` when a named type cannot be
/// resolved from the registry — callers decide whether to fail or warn.
fn resolve_field_type_dag(
    ty: &TypeExpr,
    registry: &mut TypeRegistry,
) -> Result<gunbc_ir::Dag<gunbc_ir::type_op::TypeOp>, String> {
    match ty {
        TypeExpr::Generic(name, args) => {
            match (name.as_str(), args.len()) {
                ("List", 1) => {
                    let elem = resolve_field_type_dag(&args[0], registry)?;
                    return Ok(gunbc_ir::type_lib::list(elem));
                }
                ("Option" | "Optional", 1) => {
                    let inner = resolve_field_type_dag(&args[0], registry)?;
                    return Ok(gunbc_ir::type_lib::optional(inner));
                }
                ("Set", 1) => {
                    let elem = resolve_field_type_dag(&args[0], registry)?;
                    return Ok(gunbc_ir::type_lib::set(elem));
                }
                ("Map", 2) => {
                    // Runtime Value::Map is string-keyed. Reject non-String
                    // key types at typecheck instead of deferring to runtime.
                    let key_name = type_expr_to_string(&args[0]);
                    if key_name != "String" {
                        // Non-String key types are not supported by Value::Map.
                        // Fall through to produce an error rather than silently
                        // miscompiling.
                    } else {
                        let key = resolve_field_type_dag(&args[0], registry)?;
                        let val = resolve_field_type_dag(&args[1], registry)?;
                        return Ok(gunbc_ir::type_lib::map(key, val));
                    }
                }
                _ => { /* fall through to string-based path */ }
            }
        }
        TypeExpr::Optional(inner) => {
            let inner_dag = resolve_field_type_dag(inner, registry)?;
            return Ok(gunbc_ir::type_lib::optional(inner_dag));
        }
        TypeExpr::Refined(inner, _) => {
            let base_dag = resolve_field_type_dag(inner, registry)?;
            let predicates = collect_predicates_from_type_expr(ty);
            if predicates.is_empty() {
                return Ok(base_dag);
            } else {
                let base_name = type_expr_to_string(inner);
                return Ok(gunbc_ir::type_lib::refined_with_base(
                    &base_name, base_dag, predicates,
                ));
            }
        }
        TypeExpr::Record(fields) => {
            // Inline record: desugar into a registered anonymous product type.
            // The structural name (e.g., "{key: String, value: String}") is
            // deterministic — identical inline records get the same type.
            let resolved_fields: Result<Vec<_>, String> = fields
                .iter()
                .map(|f| {
                    let dag = resolve_field_type_dag(&f.ty, registry)?;
                    Ok((f.name.as_str(), dag))
                })
                .collect();
            let resolved_fields = resolved_fields?;
            let name = type_expr_to_string(ty);
            let dag = gunbc_ir::type_lib::product_resolved(&name, resolved_fields);
            registry.register(name.as_str(), dag.clone());
            return Ok(dag);
        }
        _ => { /* Named — fall through */ }
    }

    // String-based path for Named types and unmatched Generic.
    let base_name = type_expr_to_string(ty);
    let predicates = collect_predicates_from_type_expr(ty);
    let base_dag_opt = registry.get_by_name(&base_name).cloned();
    if predicates.is_empty() {
        match base_dag_opt {
            Some(dag) => Ok(dag),
            None => Err(base_name),
        }
    } else {
        match base_dag_opt {
            Some(dag) => Ok(gunbc_ir::type_lib::refined_with_base(
                &base_name, dag, predicates,
            )),
            None => Ok(gunbc_ir::type_lib::refined(&base_name, predicates)),
        }
    }
}

/// Transitional wrapper: resolves a type DAG or falls back to identity internally.
///
/// Calls `resolve_field_type_dag` and, on failure, produces an identity placeholder
/// while recording the unresolved type and context. Typecheck converts those
/// recorded misses into diagnostics and fails the compile, so the placeholder is
/// only used to continue collecting additional errors in the same pass.
fn resolve_field_type_dag_or_identity(
    ty: &TypeExpr,
    registry: &mut TypeRegistry,
    unresolved: &mut Vec<RegistryUnresolved>,
    context: impl Into<String>,
) -> gunbc_ir::Dag<gunbc_ir::type_op::TypeOp> {
    match resolve_field_type_dag(ty, registry) {
        Ok(dag) => dag,
        Err(type_name) => {
            unresolved.push(RegistryUnresolved {
                ty: type_name.clone(),
                context: context.into(),
            });
            gunbc_ir::type_lib::identity(&type_name)
        }
    }
}

fn collect_available_profiles(modules: &[ResolvedModule]) -> Vec<String> {
    let mut profiles = std::collections::BTreeSet::new();
    for module in modules {
        for item in &module.ast.items {
            if let Item::ProfileDef(def) = &item.node {
                profiles.insert(def.name.clone());
            }
        }
    }
    profiles.into_iter().collect()
}

fn expr_to_default_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(Literal::String(value)) => Some(value.clone()),
        Expr::Literal(Literal::Int(value)) => Some(value.to_string()),
        Expr::Literal(Literal::Bool(value)) => Some(value.to_string()),
        Expr::Literal(Literal::Float(value)) => Some(value.to_string()),
        _ => None,
    }
}

/// Check match exhaustiveness for a single match expression (WS3-6).
///
/// Given a scrutinee type name and the set of variant patterns in match arms,
/// returns missing variants if the type is a known sum type and not all variants
/// are covered. Returns `None` if the type is unknown or all variants are covered.
///
/// This is opt-in validation — not enforced in the main typecheck path because
/// existing DSL code has intentional partial matches.
pub fn check_match_exhaustiveness(
    scrutinee_type: &str,
    matched_variants: &HashSet<String>,
    has_wildcard: bool,
    sum_type_variants: &HashMap<String, HashSet<String>>,
) -> Option<Vec<String>> {
    if has_wildcard {
        return None;
    }
    let all_variants = sum_type_variants.get(scrutinee_type)?;
    let missing: Vec<String> = all_variants
        .iter()
        .filter(|v| !matched_variants.contains(*v))
        .cloned()
        .collect();
    if missing.is_empty() {
        None
    } else {
        let mut sorted = missing;
        sorted.sort();
        Some(sorted)
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
    variant_parents: &'a HashMap<String, String>,
    allow_unresolved_references: bool,
}

fn extend_spanned_item_errors(
    out: &mut Vec<SpannedTypeError>,
    item_errors: Vec<TypeError>,
    file_id: FileId,
    file: &std::path::Path,
    module: &str,
    span: daglang_syntax::span::Span,
) {
    out.extend(item_errors.into_iter().map(|error| SpannedTypeError {
        error,
        file_id,
        file: file.to_path_buf(),
        module: module.to_string(),
        span,
    }));
}

fn collect_signatures(
    module: &ResolvedModule,
    file_id: FileId,
    context: &TypecheckContext<'_>,
    module_name: &str,
) -> (
    Vec<TypedItemSignature>,
    HashMap<String, TypedCallableBodyMetadata>,
    Vec<SpannedTypeError>,
) {
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
    let mut callable_body_metadata = HashMap::new();
    let data_bindings = collect_local_data_bindings(module);
    let body_context = BodyInferenceContext {
        record_type_registry: context.record_type_registry,
        callable_registry: context.callable_registry,
        data_bindings: &data_bindings,
        pattern_callable_names: context.pattern_callable_names,
        service_call_registry: context.service_call_registry,
        interface_registry: context.interface_registry,
        resource_type_registry: context.resource_type_registry,
        resource_capability_registry: context.resource_capability_registry,
        variant_parents: context.variant_parents,
        allow_unresolved_references: context.allow_unresolved_references,
    };
    let pipeline_param_bindings = collect_pipeline_param_bindings(module);

    for item in &module.ast.items {
        let item_span = item.span;
        let mut item_errors = Vec::new();
        match &item.node {
            Item::TypeDef(def) => {
                if !seen_items.insert(def.name.clone()) {
                    item_errors.push(TypeError::DuplicateDefinition {
                        module: module_name.to_string(),
                        name: def.name.clone(),
                    });
                }
                signatures.push(TypedItemSignature::Type {
                    name: def.name.clone(),
                });
            }
            Item::FnDef(def) => {
                item_errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                let item_known_types = extend_known_types(&module_known_types, &def.type_params);
                item_errors.extend(validate_params(
                    &def.name,
                    &def.params,
                    &item_known_types,
                    context.generic_arity_registry,
                    &def.type_params,
                ));
                // Handle anonymous record return types: `fn foo() -> { field: Type }`
                let (return_contract, outputs) = match &def.return_type {
                    TypeExpr::Record(fields) => {
                        for field in fields {
                            item_errors.extend(validate_type_expr(
                                &field.ty,
                                &item_known_types,
                                context.generic_arity_registry,
                                &def.type_params,
                                &format!("{}.{}", def.name, field.name),
                            ));
                        }
                        (
                            ReturnContract::record(field_signature_map(fields)),
                            fields
                                .iter()
                                .map(|f| TypedBinding {
                                    name: f.name.clone(),
                                    ty: gunbc_ir::types::TypeId::from(type_expr_to_string(&f.ty)),
                                })
                                .collect(),
                        )
                    }
                    _ => {
                        item_errors.extend(validate_type_expr(
                            &def.return_type,
                            &item_known_types,
                            context.generic_arity_registry,
                            &def.type_params,
                            &format!("{}.return", def.name),
                        ));
                        (
                            ReturnContract::single(type_expr_to_string(&def.return_type)),
                            vec![TypedBinding {
                                name: "return".to_string(),
                                ty: gunbc_ir::types::TypeId::from(type_expr_to_string(
                                    &def.return_type,
                                )),
                            }],
                        )
                    }
                };
                let body_analysis = analyze_callable_body(
                    &def.name,
                    &def.params,
                    return_contract,
                    &[],
                    CallableBodyRef {
                        stmts: &def.body.stmts,
                    },
                    &body_context,
                );
                item_errors.extend(body_analysis.errors);
                if !body_analysis.metadata.is_empty() {
                    callable_body_metadata.insert(def.name.clone(), body_analysis.metadata);
                }
                signatures.push(TypedItemSignature::Fn(TypedCallableSignature {
                    name: def.name.clone(),
                    params: def
                        .params
                        .iter()
                        .map(|param| TypedBinding {
                            name: param.name.clone(),
                            ty: gunbc_ir::types::TypeId::from(type_expr_to_string(&param.ty)),
                        })
                        .collect(),
                    outputs,
                }));
            }
            Item::FuncDef(def) => {
                item_errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                let item_known_types = extend_known_types(&module_known_types, &def.type_params);
                item_errors.extend(validate_params(
                    &def.name,
                    &def.params,
                    &item_known_types,
                    context.generic_arity_registry,
                    &def.type_params,
                ));
                item_errors.extend(validate_outputs(
                    &def.name,
                    &def.outputs,
                    &item_known_types,
                    context.generic_arity_registry,
                    &def.type_params,
                ));
                item_errors.extend(validate_uses_clauses(
                    &def.name,
                    &def.uses,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                ));
                item_errors.extend(validate_provides_clauses(
                    &def.name,
                    &def.provides,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                ));
                item_errors.extend(validate_use_provide_binding_conflicts(
                    &def.name,
                    &def.uses,
                    &def.provides,
                ));
                let body_analysis = analyze_callable_body(
                    &def.name,
                    &def.params,
                    ReturnContract::record(field_signature_map(&def.outputs)),
                    &def.uses,
                    CallableBodyRef {
                        stmts: &def.body.stmts,
                    },
                    &body_context,
                );
                item_errors.extend(body_analysis.errors);
                if !body_analysis.metadata.is_empty() {
                    callable_body_metadata.insert(def.name.clone(), body_analysis.metadata);
                }
                signatures.push(TypedItemSignature::Func(TypedCallableSignature {
                    name: def.name.clone(),
                    params: def
                        .params
                        .iter()
                        .map(|param| TypedBinding {
                            name: param.name.clone(),
                            ty: gunbc_ir::types::TypeId::from(type_expr_to_string(&param.ty)),
                        })
                        .collect(),
                    outputs: def
                        .outputs
                        .iter()
                        .map(|field| TypedBinding {
                            name: field.name.clone(),
                            ty: gunbc_ir::types::TypeId::from(type_expr_to_string(&field.ty)),
                        })
                        .collect(),
                }));
            }
            Item::PatternDef(def) => {
                item_errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                let item_known_types = extend_known_types(&module_known_types, &def.type_params);
                item_errors.extend(validate_params(
                    &def.name,
                    &def.params,
                    &item_known_types,
                    context.generic_arity_registry,
                    &def.type_params,
                ));
                item_errors.extend(validate_outputs(
                    &def.name,
                    &def.outputs,
                    &item_known_types,
                    context.generic_arity_registry,
                    &def.type_params,
                ));
                item_errors.extend(validate_uses_clauses(
                    &def.name,
                    &def.uses,
                    context.resource_type_registry,
                    context.allow_unresolved_references,
                ));
                let body_analysis = analyze_callable_body(
                    &def.name,
                    &def.params,
                    ReturnContract::record(field_signature_map(&def.outputs)),
                    &def.uses,
                    CallableBodyRef {
                        stmts: &def.body.stmts,
                    },
                    &body_context,
                );
                item_errors.extend(body_analysis.errors);
                if !body_analysis.metadata.is_empty() {
                    callable_body_metadata.insert(def.name.clone(), body_analysis.metadata);
                }
                signatures.push(TypedItemSignature::Pattern(TypedCallableSignature {
                    name: def.name.clone(),
                    params: def
                        .params
                        .iter()
                        .map(|param| TypedBinding {
                            name: param.name.clone(),
                            ty: gunbc_ir::types::TypeId::from(type_expr_to_string(&param.ty)),
                        })
                        .collect(),
                    outputs: def
                        .outputs
                        .iter()
                        .map(|field| TypedBinding {
                            name: field.name.clone(),
                            ty: gunbc_ir::types::TypeId::from(type_expr_to_string(&field.ty)),
                        })
                        .collect(),
                }));
            }
            Item::ServiceDef(def) => {
                item_errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                item_errors.extend(validate_service_interface_conformance(
                    def,
                    context.interface_registry,
                ));
                if let Some(ref scheme) = def.config.auth {
                    if !is_valid_auth_scheme(scheme) {
                        item_errors.push(TypeError::InvalidAuthScheme {
                            service: def.name.clone(),
                            scheme: scheme.clone(),
                        });
                    }
                }
                signatures.push(TypedItemSignature::Service {
                    name: def.name.clone(),
                    operations: def.operations.len(),
                });
            }
            Item::ResourceDef(def) => {
                item_errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                item_errors.extend(validate_resource_interface_conformance(
                    def,
                    context.interface_registry,
                ));
                signatures.push(TypedItemSignature::Resource {
                    name: def.name.clone(),
                    implements: def.implements.clone(),
                });
            }
            Item::InterfaceDef(def) => {
                item_errors.extend(record_duplicate_item_name(
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
                item_errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                item_errors.extend(validate_pipeline_def(
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
            // Project/profile blocks are not typechecked yet in this pass
            Item::ProjectDef(_)
            | Item::FeatureDef(_)
            | Item::TaskDef(_)
            | Item::DesignDef(_)
            | Item::ComponentDef(_)
            | Item::EnvironmentDef(_)
            | Item::ProfileDef(_) => {}
            Item::ParamDecl(decl) => {
                item_errors.extend(validate_type_expr(
                    &decl.ty,
                    &module_known_types,
                    context.generic_arity_registry,
                    &[],
                    &format!("param {}", decl.name),
                ));
            }
            Item::DataDef(def) => {
                item_errors.extend(validate_type_expr(
                    &def.ty,
                    &module_known_types,
                    context.generic_arity_registry,
                    &[],
                    &format!("data {}", def.name),
                ));
            }
            Item::ExternAssetDecl(def) => {
                item_errors.extend(record_duplicate_item_name(
                    module_name,
                    &def.name,
                    &mut seen_items,
                ));
                item_errors.extend(validate_type_expr(
                    &def.ty,
                    &module_known_types,
                    context.generic_arity_registry,
                    &[],
                    &def.name,
                ));
            }
        }
        extend_spanned_item_errors(
            &mut errors,
            item_errors,
            file_id,
            &module.path,
            module_name,
            item_span,
        );
    }

    (signatures, callable_body_metadata, errors)
}

fn collect_local_data_bindings(module: &ResolvedModule) -> HashMap<String, ValueType> {
    module
        .ast
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::DataDef(def) => Some((def.name.clone(), value_type_from_type_expr(&def.ty))),
            _ => None,
        })
        .collect()
}

fn file_id_for_graph_index(graph_index: usize) -> FileId {
    FileId(u32::try_from(graph_index).expect("module graph index should fit in u32"))
}

fn collect_pipeline_param_bindings(module: &ResolvedModule) -> HashMap<String, ValueType> {
    let mut bindings = HashMap::new();
    for item in &module.ast.items {
        if let Item::ParamDecl(decl) = &item.node {
            bindings.insert(decl.name.clone(), value_type_from_type_expr(&decl.ty));
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
        data_bindings: body_context.data_bindings,
        service_call_registry: body_context.service_call_registry,
        bound_service_registry: &empty_bound_services,
        param_callable_contracts: &empty_param_callable_contracts,
        variant_parents: body_context.variant_parents,
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
            if !is_bool && !inferred.is_inferred() {
                errors.push(TypeError::PipelineStageWhenTypeMismatch {
                    pipeline: pipeline_name.clone(),
                    stage: stage.name.clone(),
                    got: inferred.display_name(),
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
    let mut known: HashSet<String> = BUILTIN_TYPES.iter().map(|b| b.name.to_string()).collect();
    for module in modules {
        let module_prefix = module.module_path.as_dotted();
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
    for b in BUILTIN_TYPES {
        registry.full.insert(b.name.to_string(), b.arity);
        registry.short.insert(b.name.to_string(), Some(b.arity));
    }

    for module in modules {
        let module_prefix = module.module_path.as_dotted();
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
        let module_prefix = module.module_path.as_dotted();
        for item in &module.ast.items {
            match &item.node {
                Item::TypeDef(def) => {
                    let daglang_syntax::ast::TypeBody::Record(fields) = &def.body else {
                        continue;
                    };
                    let signature = field_value_type_map(fields);
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
                    let signature = field_value_type_map(&def.config);
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

/// Maps variant names to their parent sum type name (WS3-5).
///
/// For `type Result = Ok { value: T } | Err { error: E }`, returns
/// `{"Ok" => "Result", "Err" => "Result"}`.
fn collect_variant_parents(modules: &[ResolvedModule]) -> HashMap<String, String> {
    let mut parents = HashMap::new();
    for module in modules {
        for item in &module.ast.items {
            if let Item::TypeDef(def) = &item.node {
                if let daglang_syntax::ast::TypeBody::Sum(variants) = &def.body {
                    for variant in variants {
                        parents.insert(variant.name.clone(), def.name.clone());
                    }
                }
            }
        }
    }
    parents
}

/// Maps sum type names to their set of variant names (WS3-6).
///
/// For `type Color = Red | Blue | Green`, returns `{"Color" => {"Red", "Blue", "Green"}}`.
fn collect_sum_type_variants(modules: &[ResolvedModule]) -> HashMap<String, HashSet<String>> {
    let mut variants_map = HashMap::new();
    for module in modules {
        for item in &module.ast.items {
            if let Item::TypeDef(def) = &item.node {
                if let daglang_syntax::ast::TypeBody::Sum(variants) = &def.body {
                    let names: HashSet<String> = variants.iter().map(|v| v.name.clone()).collect();
                    variants_map.insert(def.name.clone(), names);
                }
            }
        }
    }
    variants_map
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CallableContract {
    arity: usize,
    params: HashSet<String>,
    param_order: Vec<String>,
    param_types: HashMap<String, ValueType>,
    output: ValueType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceCallContract {
    arity: usize,
    params: HashSet<String>,
    outputs: HashMap<String, ValueType>,
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
                        param_order: def.params.iter().map(|param| param.name.clone()).collect(),
                        param_types: def
                            .params
                            .iter()
                            .map(|param| (param.name.clone(), value_type_from_type_expr(&param.ty)))
                            .collect(),
                        output: value_type_from_type_expr(&def.return_type),
                    },
                ),
                Item::FuncDef(def) => register_callable_contract(
                    &mut callables,
                    def.name.clone(),
                    CallableContract {
                        arity: required_param_arity(&def.params),
                        params: def.params.iter().map(|param| param.name.clone()).collect(),
                        param_order: def.params.iter().map(|param| param.name.clone()).collect(),
                        param_types: def
                            .params
                            .iter()
                            .map(|param| (param.name.clone(), value_type_from_type_expr(&param.ty)))
                            .collect(),
                        output: if def.outputs.len() == 1 && def.outputs[0].name == "return" {
                            value_type_from_type_expr(&def.outputs[0].ty)
                        } else {
                            ValueType::Record(field_value_type_map(&def.outputs))
                        },
                    },
                ),
                Item::PatternDef(def) => register_callable_contract(
                    &mut callables,
                    def.name.clone(),
                    CallableContract {
                        arity: required_param_arity(&def.params),
                        params: def.params.iter().map(|param| param.name.clone()).collect(),
                        param_order: def.params.iter().map(|param| param.name.clone()).collect(),
                        param_types: def
                            .params
                            .iter()
                            .map(|param| (param.name.clone(), value_type_from_type_expr(&param.ty)))
                            .collect(),
                        output: if def.outputs.len() == 1 && def.outputs[0].name == "return" {
                            value_type_from_type_expr(&def.outputs[0].ty)
                        } else {
                            ValueType::Record(field_value_type_map(&def.outputs))
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
                                    param_order: variant
                                        .fields
                                        .iter()
                                        .map(|field| field.name.clone())
                                        .collect(),
                                    param_types: variant
                                        .fields
                                        .iter()
                                        .map(|field| {
                                            (
                                                field.name.clone(),
                                                value_type_from_type_expr(&field.ty),
                                            )
                                        })
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
    // Collection operation builtins — derived from centralized registry (S11).
    // Each CollectionKind carries its own typecheck contract metadata.
    use gunbc_ir::patterns::{non_collection_builtin_contracts, ALL_COLLECTION_OPS};

    let builtin_to_contract =
        |name: &str, bc: &gunbc_ir::patterns::BuiltinContract| -> (String, CallableContract) {
            (
                name.to_string(),
                CallableContract {
                    arity: bc.arity,
                    params: bc.params.iter().map(|s| s.to_string()).collect(),
                    param_order: bc.params.iter().map(|s| s.to_string()).collect(),
                    param_types: HashMap::new(),
                    output: ValueType::Named(bc.output_type.to_string()),
                },
            )
        };

    let mut contracts: Vec<(String, CallableContract)> = Vec::new();

    // Canonical collection ops (map, filter, fold, etc.)
    for kind in ALL_COLLECTION_OPS {
        let name = kind.from_name_reverse();
        let bc = kind.typecheck_contract();
        contracts.push(builtin_to_contract(name, &bc));
    }

    // Non-collection builtins (first, last, max_by, starts_with, etc.)
    for (name, bc) in non_collection_builtin_contracts() {
        contracts.push(builtin_to_contract(name, &bc));
    }

    // Non-collection builtins (standalone functions, render helpers, etc.).
    // eq, chars, code_point, and build_token are now DSL fn items
    // (std/patterns.dag, std/unicode.dag, gunbc/auth/patterns.dag).
    contracts.extend([
        (
            "render_cytoscape_html".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["snapshot".to_string()]),
                param_order: vec!["snapshot".to_string()],
                param_types: HashMap::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "render_mermaid_markdown".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["snapshot".to_string()]),
                param_order: vec!["snapshot".to_string()],
                param_types: HashMap::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "render_test_listings".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["sources".to_string()]),
                param_order: vec!["sources".to_string()],
                param_types: HashMap::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "render_graph_structure".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["sources".to_string()]),
                param_order: vec!["sources".to_string()],
                param_types: HashMap::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "render_source_artifacts".to_string(),
            CallableContract {
                arity: 1,
                params: HashSet::from(["sources".to_string()]),
                param_order: vec!["sources".to_string()],
                param_types: HashMap::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "generate".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                param_order: Vec::new(),
                param_types: HashMap::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "now".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                param_order: Vec::new(),
                param_types: HashMap::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "compute_topology_diff".to_string(),
            CallableContract {
                arity: 2,
                params: HashSet::from(["current".to_string(), "base".to_string()]),
                param_order: vec!["current".to_string(), "base".to_string()],
                param_types: HashMap::new(),
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
                param_order: vec![
                    "diff".to_string(),
                    "topology".to_string(),
                    "title".to_string(),
                ],
                param_types: HashMap::new(),
                output: ValueType::Named("String".to_string()),
            },
        ),
        (
            "detect_runtime".to_string(),
            CallableContract {
                arity: 0,
                params: HashSet::new(),
                param_order: Vec::new(),
                param_types: HashMap::new(),
                output: ValueType::Named("CloudRuntime".to_string()),
            },
        ),
    ]);

    contracts
}

fn collect_service_call_contracts(modules: &[ResolvedModule]) -> ServiceCallRegistry {
    let mut registry = ServiceCallRegistry::default();
    for module in modules {
        let module_name = module.module_path.as_dotted();
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
                    outputs: field_value_type_map(&operation.outputs),
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
    full: HashMap<String, HashMap<String, ValueType>>,
    short: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ValueType {
    Named(String),
    Generic(String, Vec<ValueType>),
    Record(HashMap<String, ValueType>),
    /// The typechecker could not determine a concrete type for this expression.
    ///
    /// This occurs in legitimate cases where inference is incomplete (e.g.,
    /// for-loop element types, deferred service calls, unresolved cross-module
    /// references). Unlike an error, `Inferred` means "some valid type exists
    /// but we lack information to name it." Downstream consumers treat it as
    /// compatible with any expected type.
    Inferred,
}

#[derive(Debug, Clone)]
enum ReturnContract {
    Single { ty: String },
    Record { fields: HashMap<String, String> },
}

#[derive(Debug, Default)]
struct CallableBodyAnalysis {
    errors: Vec<TypeError>,
    metadata: TypedCallableBodyMetadata,
}

type StableExprIdentities = HashMap<usize, ExprIdentity>;

impl ReturnContract {
    fn single(ty: String) -> Self {
        Self::Single { ty }
    }

    fn record(fields: HashMap<String, String>) -> Self {
        Self::Record { fields }
    }
}

fn stable_expr_identities(stmts: &[Stmt]) -> StableExprIdentities {
    let mut expr_identities = HashMap::new();
    walk_stmts_with_expr_identities(stmts, &mut |expr_identity, expr| {
        expr_identities.insert(expr as *const Expr as usize, expr_identity);
    });
    expr_identities
}

fn stable_expr_identity(expr: &Expr, expr_identities: &StableExprIdentities) -> ExprIdentity {
    *expr_identities
        .get(&(expr as *const Expr as usize))
        .expect("expression identity should exist for walked callable body")
}

fn strip_optional_type(ty: &str) -> &str {
    ty.strip_suffix('?').unwrap_or(ty)
}

fn record_target_type<'a>(ty: &'a str, registry: &RecordTypeRegistry) -> Option<&'a str> {
    let stripped = strip_optional_type(ty);
    resolve_record_fields(stripped, registry).map(|_| stripped)
}

#[derive(Clone, Copy)]
struct BodyInferenceContext<'a> {
    record_type_registry: &'a RecordTypeRegistry,
    callable_registry: &'a HashMap<String, Option<CallableContract>>,
    data_bindings: &'a HashMap<String, ValueType>,
    pattern_callable_names: &'a HashSet<String>,
    service_call_registry: &'a ServiceCallRegistry,
    interface_registry: &'a InterfaceRegistry,
    resource_type_registry: &'a ResourceTypeRegistry,
    resource_capability_registry: &'a ResourceCapabilityRegistry,
    variant_parents: &'a HashMap<String, String>,
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
    data_bindings: &'a HashMap<String, ValueType>,
    service_call_registry: &'a ServiceCallRegistry,
    bound_service_registry: &'a BoundServiceCallRegistry,
    param_callable_contracts: &'a HashMap<String, CallableContract>,
    variant_parents: &'a HashMap<String, String>,
}

struct CallableBodyRef<'a> {
    stmts: &'a [Stmt],
}

fn collect_interfaces(modules: &[ResolvedModule]) -> InterfaceRegistry {
    let mut registry = InterfaceRegistry::default();
    for module in modules {
        let module_name = module.module_path.as_dotted();
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
        let module_name = module.module_path.as_dotted();
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
    insert_default_resource_types(&mut registry);
    registry
}

fn insert_default_resource_types(registry: &mut ResourceTypeRegistry) {
    for name in ["Filesystem", "Network", "Clock", "AuthContext"] {
        let full = format!("std.resources.{name}");
        registry.full.insert(full.clone());
        registry
            .short
            .entry(name.to_string())
            .or_insert_with(|| Some(full));
    }
}

fn collect_resource_capabilities(modules: &[ResolvedModule]) -> ResourceCapabilityRegistry {
    let mut registry = ResourceCapabilityRegistry::default();
    for module in modules {
        let module_name = module.module_path.as_dotted();
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

// BuiltinType and BUILTIN_TYPES are imported from gunbc_ir (S12 consolidation).

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
    type_params: &[String],
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
            type_params,
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
    type_params: &[String],
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
            type_params,
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

fn positional_callable_param_name(index: usize) -> String {
    format!("__arg{}", index)
}

fn parse_function_type_callable_contract(ty: &TypeExpr) -> Option<CallableContract> {
    match ty {
        TypeExpr::Function(params, output) => {
            let param_order = params
                .iter()
                .enumerate()
                .map(|(index, _)| positional_callable_param_name(index))
                .collect::<Vec<_>>();
            let param_types = param_order
                .iter()
                .cloned()
                .zip(params.iter().map(value_type_from_type_expr))
                .collect::<HashMap<_, _>>();
            Some(CallableContract {
                arity: params.len(),
                params: HashSet::new(),
                param_order,
                param_types,
                output: value_type_from_type_expr(output),
            })
        }
        TypeExpr::Optional(inner) | TypeExpr::Refined(inner, _) => {
            parse_function_type_callable_contract(inner)
        }
        _ => None,
    }
}

fn call_arg_expected_record_type(
    contract: &CallableContract,
    arg_index: usize,
    arg_name: Option<&str>,
    record_type_registry: &RecordTypeRegistry,
) -> Option<String> {
    let ty = arg_name
        .and_then(|name| contract.param_types.get(name))
        .or_else(|| {
            contract
                .param_order
                .get(arg_index)
                .and_then(|name| contract.param_types.get(name))
        })?;
    let display = ty.display_name();
    record_target_type(&display, record_type_registry).map(str::to_string)
}

fn collect_constructor_targets_from_stmts<'a>(
    stmts: &'a [Stmt],
    local_bindings: &HashMap<String, ValueType>,
    binding_exprs: &HashMap<String, &'a Expr>,
    infer_context: &ExprInferenceContext<'_>,
    expr_identities: &StableExprIdentities,
    metadata: &mut TypedCallableBodyMetadata,
) {
    let mut scope_types = local_bindings.clone();
    let mut scope_exprs = binding_exprs.clone();
    for stmt in stmts {
        match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                collect_constructor_targets_from_expr(
                    expr,
                    &scope_types,
                    &scope_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
                let (inferred, _) = infer_expr_type(expr, &scope_types, infer_context);
                scope_types.insert(name.clone(), inferred);
                scope_exprs.insert(name.clone(), expr);
            }
            Stmt::Node(node_stmt) => {
                collect_constructor_targets_from_expr(
                    &node_stmt.expr,
                    &scope_types,
                    &scope_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
                let (inferred, _) = infer_expr_type(&node_stmt.expr, &scope_types, infer_context);
                scope_types.insert(node_stmt.name.clone(), inferred);
                scope_exprs.insert(node_stmt.name.clone(), &node_stmt.expr);
            }
            Stmt::Expr(expr) => collect_constructor_targets_from_expr(
                expr,
                &scope_types,
                &scope_exprs,
                infer_context,
                expr_identities,
                metadata,
            ),
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    collect_constructor_targets_from_expr(
                        expr,
                        &scope_types,
                        &scope_exprs,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn annotate_expr_with_expected_record<'a>(
    expr: &'a Expr,
    expected_type: &str,
    local_bindings: &HashMap<String, ValueType>,
    binding_exprs: &HashMap<String, &'a Expr>,
    infer_context: &ExprInferenceContext<'_>,
    expr_identities: &StableExprIdentities,
    metadata: &mut TypedCallableBodyMetadata,
    visiting: &mut HashSet<ExprIdentity>,
) {
    let Some(expected_type) = record_target_type(expected_type, infer_context.record_type_registry)
    else {
        return;
    };
    let expr_identity = stable_expr_identity(expr, expr_identities);
    if !visiting.insert(expr_identity) {
        return;
    }

    match expr {
        Expr::Record(None, fields) => {
            metadata.annotate_anonymous_record_target(expr_identity, expected_type);
            if let Some(expected_fields) =
                resolve_record_fields(expected_type, infer_context.record_type_registry)
            {
                for (field_name, field_expr) in fields {
                    if let Some(ValueType::Named(field_type_name)) = expected_fields.get(field_name)
                    {
                        annotate_expr_with_expected_record(
                            field_expr,
                            field_type_name,
                            local_bindings,
                            binding_exprs,
                            infer_context,
                            expr_identities,
                            metadata,
                            visiting,
                        );
                    }
                }
            }
        }
        Expr::Record(Some(record_name), fields) => {
            if let Some(expected_fields) =
                resolve_record_fields(record_name, infer_context.record_type_registry)
            {
                for (field_name, field_expr) in fields {
                    if let Some(ValueType::Named(field_type_name)) = expected_fields.get(field_name)
                    {
                        annotate_expr_with_expected_record(
                            field_expr,
                            field_type_name,
                            local_bindings,
                            binding_exprs,
                            infer_context,
                            expr_identities,
                            metadata,
                            visiting,
                        );
                    }
                }
            }
        }
        Expr::Ident(name) => {
            if let Some(bound_expr) = binding_exprs.get(name) {
                annotate_expr_with_expected_record(
                    bound_expr,
                    expected_type,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                    visiting,
                );
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_constructor_targets_from_expr(
                cond,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
            annotate_expr_with_expected_record(
                then_expr,
                expected_type,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
                visiting,
            );
            if let Some(otherwise) = else_expr {
                annotate_expr_with_expected_record(
                    otherwise,
                    expected_type,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                    visiting,
                );
            }
        }
        Expr::Match(scrutinee, arms) => {
            collect_constructor_targets_from_expr(
                scrutinee,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_constructor_targets_from_expr(
                        guard,
                        local_bindings,
                        binding_exprs,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
                annotate_expr_with_expected_record(
                    &arm.body,
                    expected_type,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                    visiting,
                );
            }
        }
        Expr::Block(stmts) => {
            let mut scope_types = local_bindings.clone();
            let mut scope_exprs = binding_exprs.clone();
            let mut trailing_expr = None;
            for stmt in stmts {
                match stmt {
                    Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                        collect_constructor_targets_from_expr(
                            expr,
                            &scope_types,
                            &scope_exprs,
                            infer_context,
                            expr_identities,
                            metadata,
                        );
                        let (inferred, _) = infer_expr_type(expr, &scope_types, infer_context);
                        scope_types.insert(name.clone(), inferred);
                        scope_exprs.insert(name.clone(), expr);
                        trailing_expr = None;
                    }
                    Stmt::Node(node_stmt) => {
                        collect_constructor_targets_from_expr(
                            &node_stmt.expr,
                            &scope_types,
                            &scope_exprs,
                            infer_context,
                            expr_identities,
                            metadata,
                        );
                        let (inferred, _) =
                            infer_expr_type(&node_stmt.expr, &scope_types, infer_context);
                        scope_types.insert(node_stmt.name.clone(), inferred);
                        scope_exprs.insert(node_stmt.name.clone(), &node_stmt.expr);
                        trailing_expr = None;
                    }
                    Stmt::Expr(inner) => {
                        collect_constructor_targets_from_expr(
                            inner,
                            &scope_types,
                            &scope_exprs,
                            infer_context,
                            expr_identities,
                            metadata,
                        );
                        trailing_expr = Some(inner);
                    }
                    Stmt::Return(fields) => {
                        for (_, inner) in fields {
                            annotate_expr_with_expected_record(
                                inner,
                                expected_type,
                                &scope_types,
                                &scope_exprs,
                                infer_context,
                                expr_identities,
                                metadata,
                                visiting,
                            );
                            collect_constructor_targets_from_expr(
                                inner,
                                &scope_types,
                                &scope_exprs,
                                infer_context,
                                expr_identities,
                                metadata,
                            );
                        }
                        trailing_expr = None;
                    }
                }
            }
            if let Some(trailing_expr) = trailing_expr {
                annotate_expr_with_expected_record(
                    trailing_expr,
                    expected_type,
                    &scope_types,
                    &scope_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                    visiting,
                );
            }
        }
        Expr::Guarded(inner, guard) => {
            collect_constructor_targets_from_expr(
                guard,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
            annotate_expr_with_expected_record(
                inner,
                expected_type,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
                visiting,
            );
        }
        Expr::After(inner, _) => {
            annotate_expr_with_expected_record(
                inner,
                expected_type,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
                visiting,
            );
        }
        _ => {}
    }

    visiting.remove(&expr_identity);
}

fn collect_constructor_targets_from_expr<'a>(
    expr: &'a Expr,
    local_bindings: &HashMap<String, ValueType>,
    binding_exprs: &HashMap<String, &'a Expr>,
    infer_context: &ExprInferenceContext<'_>,
    expr_identities: &StableExprIdentities,
    metadata: &mut TypedCallableBodyMetadata,
) {
    match expr {
        Expr::Literal(_) | Expr::Ident(_) => {}
        Expr::FieldAccess(base, _) | Expr::UnaryOp(_, base) => {
            collect_constructor_targets_from_expr(
                base,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
        }
        Expr::Call(name, args) if name == "fold" && args.len() >= 2 => {
            let init = args
                .iter()
                .find(|(n, _)| n.as_deref() == Some("init"))
                .or_else(|| args.get(1))
                .map(|(_, e)| e);
            let func = args
                .iter()
                .find(|(n, _)| n.as_deref() == Some("f"))
                .or_else(|| args.get(2))
                .map(|(_, e)| e);

            if let Some(init_expr) = init {
                let (acc_ty, _) = infer_fold_accumulator_type(
                    &args[0].1,
                    init_expr,
                    func,
                    local_bindings,
                    infer_context,
                );

                // Annotate init anonymous record with the merged accumulator field types.
                // The merged type refines incomplete init fields (e.g. empty list [] becomes
                // List<Span> after merging with the fold body).
                if let (Expr::Record(None, _), ValueType::Record(ref fields)) = (init_expr, &acc_ty)
                {
                    metadata.annotate_anonymous_record_field_types(
                        stable_expr_identity(init_expr, expr_identities),
                        fields,
                    );
                }

                // Recurse into fold lambda body with typed accumulator and element params
                // so nested anonymous records inherit the refined scope.
                if let Some(Expr::Lambda(params, body)) = func {
                    let (collection_ty, _) =
                        infer_expr_type(&args[0].1, local_bindings, infer_context);
                    let mut fold_scope_types = local_bindings.clone();
                    let mut fold_scope_exprs = binding_exprs.clone();
                    if let Some(param) = params.first() {
                        fold_scope_types.insert(param.clone(), acc_ty);
                        fold_scope_exprs.remove(param);
                    }
                    if let (Some(param), Some(elem_ty)) =
                        (params.get(1), collection_element_value_type(&collection_ty))
                    {
                        fold_scope_types.insert(param.clone(), elem_ty);
                        fold_scope_exprs.remove(param);
                    }
                    collect_constructor_targets_from_expr(
                        body,
                        &fold_scope_types,
                        &fold_scope_exprs,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
            }

            // Recurse into all non-lambda args for general annotation.
            for (_, arg_expr) in args {
                if matches!(arg_expr, Expr::Lambda(..)) {
                    continue;
                }
                collect_constructor_targets_from_expr(
                    arg_expr,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::Call(name, args) => {
            if name == "with" && args.len() == 2 {
                let (base_ty, _) = infer_expr_type(&args[0].1, local_bindings, infer_context);
                if let ValueType::Named(base_ty) = base_ty {
                    annotate_expr_with_expected_record(
                        &args[1].1,
                        &base_ty,
                        local_bindings,
                        binding_exprs,
                        infer_context,
                        expr_identities,
                        metadata,
                        &mut HashSet::new(),
                    );
                }
            } else {
                let contract = infer_context
                    .param_callable_contracts
                    .get(name)
                    .or_else(|| {
                        infer_context
                            .callable_registry
                            .get(name)
                            .and_then(|entry| entry.as_ref())
                    });
                if let Some(contract) = contract {
                    for (arg_index, (arg_name, arg_expr)) in args.iter().enumerate() {
                        if let Some(expected_ty) = call_arg_expected_record_type(
                            contract,
                            arg_index,
                            arg_name.as_deref(),
                            infer_context.record_type_registry,
                        ) {
                            annotate_expr_with_expected_record(
                                arg_expr,
                                &expected_ty,
                                local_bindings,
                                binding_exprs,
                                infer_context,
                                expr_identities,
                                metadata,
                                &mut HashSet::new(),
                            );
                        }
                    }
                }
            }
            for (_, arg_expr) in args {
                collect_constructor_targets_from_expr(
                    arg_expr,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::ServiceCall(_, args) => {
            for (_, arg_expr) in args {
                collect_constructor_targets_from_expr(
                    arg_expr,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::BinOp(lhs, _, rhs) => {
            collect_constructor_targets_from_expr(
                lhs,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
            collect_constructor_targets_from_expr(
                rhs,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_constructor_targets_from_expr(
                        inner,
                        local_bindings,
                        binding_exprs,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
            }
        }
        Expr::Record(type_name, fields) => {
            // For anonymous records, infer and annotate field types so downstream
            // stages can synthesize struct definitions without re-inference.
            if type_name.is_none() {
                let inferred_fields: HashMap<String, ValueType> = fields
                    .iter()
                    .map(|(name, field_expr)| {
                        let (ty, _) = infer_expr_type(field_expr, local_bindings, infer_context);
                        (name.clone(), ty)
                    })
                    .collect();
                metadata.annotate_anonymous_record_field_types(
                    stable_expr_identity(expr, expr_identities),
                    &inferred_fields,
                );
            }
            let record_fields = type_name
                .as_deref()
                .and_then(|name| resolve_record_fields(name, infer_context.record_type_registry));
            for (field_name, field_expr) in fields {
                if let Some(expected_fields) = &record_fields {
                    if let Some(ValueType::Named(field_type_name)) = expected_fields.get(field_name)
                    {
                        annotate_expr_with_expected_record(
                            field_expr,
                            field_type_name,
                            local_bindings,
                            binding_exprs,
                            infer_context,
                            expr_identities,
                            metadata,
                            &mut HashSet::new(),
                        );
                    }
                }
                collect_constructor_targets_from_expr(
                    field_expr,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::Match(scrutinee, arms) => {
            collect_constructor_targets_from_expr(
                scrutinee,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_constructor_targets_from_expr(
                        guard,
                        local_bindings,
                        binding_exprs,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
                collect_constructor_targets_from_expr(
                    &arm.body,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_constructor_targets_from_expr(
                cond,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
            collect_constructor_targets_from_expr(
                then_expr,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
            if let Some(otherwise) = else_expr {
                collect_constructor_targets_from_expr(
                    otherwise,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::For(binding, iterable, passthrough, body) => {
            collect_constructor_targets_from_expr(
                iterable,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
            let (iter_ty, _) = infer_expr_type(iterable, local_bindings, infer_context);
            let (loop_scope, _) =
                resolve_for_loop_scope_contract(binding, &iter_ty, passthrough, local_bindings);
            if let Some(loop_scope) = loop_scope {
                let loop_scope_types = loop_scope.local_bindings(local_bindings);
                let mut loop_scope_exprs = binding_exprs.clone();
                loop_scope_exprs.remove(binding);
                match body {
                    ForBody::Expr(body_expr) => collect_constructor_targets_from_expr(
                        body_expr,
                        &loop_scope_types,
                        &loop_scope_exprs,
                        infer_context,
                        expr_identities,
                        metadata,
                    ),
                    ForBody::Block(stmts) => collect_constructor_targets_from_stmts(
                        stmts,
                        &loop_scope_types,
                        &loop_scope_exprs,
                        infer_context,
                        expr_identities,
                        metadata,
                    ),
                }
            }
        }
        Expr::Lambda(params, body) => {
            let mut lambda_scope_types = local_bindings.clone();
            let mut lambda_scope_exprs = binding_exprs.clone();
            for param in params {
                lambda_scope_types.insert(param.clone(), ValueType::Inferred);
                lambda_scope_exprs.remove(param);
            }
            collect_constructor_targets_from_expr(
                body,
                &lambda_scope_types,
                &lambda_scope_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
        }
        Expr::List(items) => {
            for item in items {
                collect_constructor_targets_from_expr(
                    item,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                collect_constructor_targets_from_expr(
                    key,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
                collect_constructor_targets_from_expr(
                    value,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::Guarded(inner, guard) => {
            collect_constructor_targets_from_expr(
                inner,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
            collect_constructor_targets_from_expr(
                guard,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
            );
        }
        Expr::After(inner, _) => collect_constructor_targets_from_expr(
            inner,
            local_bindings,
            binding_exprs,
            infer_context,
            expr_identities,
            metadata,
        ),
        Expr::Return(fields) => {
            for (_, field_expr) in fields {
                collect_constructor_targets_from_expr(
                    field_expr,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::Block(stmts) => collect_constructor_targets_from_stmts(
            stmts,
            local_bindings,
            binding_exprs,
            infer_context,
            expr_identities,
            metadata,
        ),
    }
}

fn analyze_callable_body(
    caller: &str,
    params: &[Param],
    return_contract: ReturnContract,
    uses: &[UsesClause],
    body: CallableBodyRef<'_>,
    body_context: &BodyInferenceContext<'_>,
) -> CallableBodyAnalysis {
    let mut analysis = CallableBodyAnalysis::default();
    let expr_identities = stable_expr_identities(body.stmts);
    let bound_service_registry = build_bound_service_call_registry(uses, body_context);
    let param_callable_contracts = collect_param_callable_contracts(params);
    let infer_context = ExprInferenceContext {
        record_type_registry: body_context.record_type_registry,
        callable_registry: body_context.callable_registry,
        data_bindings: body_context.data_bindings,
        service_call_registry: body_context.service_call_registry,
        bound_service_registry: &bound_service_registry,
        param_callable_contracts: &param_callable_contracts,
        variant_parents: body_context.variant_parents,
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
                        analysis.errors.push(TypeError::AmbiguousCallTarget {
                            caller: caller.to_string(),
                            callee: call.callee.clone(),
                        });
                    }
                    continue;
                }
                None => {
                    if !body_context.allow_unresolved_references {
                        analysis.errors.push(TypeError::UnresolvedCallTarget {
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
                analysis.errors.push(TypeError::CallArityMismatch {
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
                analysis.errors.push(TypeError::DuplicateCallArgument {
                    caller: caller.to_string(),
                    callee: call.callee.clone(),
                    argument: named,
                });
                continue;
            }
            if !is_pattern_callable && !contract.params.contains(&named) {
                analysis.errors.push(TypeError::UnknownCallArgument {
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
                        analysis.errors.push(TypeError::AmbiguousServiceCall {
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
                                analysis.errors.push(TypeError::UnresolvedServiceCall {
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
            analysis.errors.push(TypeError::ServiceCallArityMismatch {
                caller: caller.to_string(),
                service_call: service_call_name.clone(),
                expected,
                got: call.arg_count,
            });
        }
        let mut seen_named = HashSet::new();
        for named in call.named_args {
            if !seen_named.insert(named.clone()) {
                analysis
                    .errors
                    .push(TypeError::DuplicateServiceCallArgument {
                        caller: caller.to_string(),
                        service_call: service_call_name.clone(),
                        argument: named,
                    });
                continue;
            }
            if !contract.params.contains(&named) {
                analysis.errors.push(TypeError::UnknownServiceCallArgument {
                    caller: caller.to_string(),
                    service_call: service_call_name.clone(),
                    argument: named,
                });
            }
        }
    }

    let mut local_bindings = params
        .iter()
        .map(|param| (param.name.clone(), value_type_from_type_expr(&param.ty)))
        .collect::<HashMap<_, _>>();
    let mut binding_exprs = HashMap::new();
    let mut saw_explicit_return = false;
    let mut trailing_expr_type = None;
    let mut trailing_expr = None;
    for stmt in body.stmts {
        match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                collect_constructor_targets_from_expr(
                    expr,
                    &local_bindings,
                    &binding_exprs,
                    &infer_context,
                    &expr_identities,
                    &mut analysis.metadata,
                );
                let (inferred, infer_errors) =
                    infer_expr_type(expr, &local_bindings, &infer_context);
                analysis.errors.extend(infer_errors);
                local_bindings.insert(name.clone(), inferred);
                binding_exprs.insert(name.clone(), expr);
                trailing_expr_type = None;
                trailing_expr = None;
            }
            Stmt::Node(ns) => {
                collect_constructor_targets_from_expr(
                    &ns.expr,
                    &local_bindings,
                    &binding_exprs,
                    &infer_context,
                    &expr_identities,
                    &mut analysis.metadata,
                );
                let (inferred, infer_errors) =
                    infer_expr_type(&ns.expr, &local_bindings, &infer_context);
                analysis.errors.extend(infer_errors);
                local_bindings.insert(ns.name.clone(), inferred);
                binding_exprs.insert(ns.name.clone(), &ns.expr);
                trailing_expr_type = None;
                trailing_expr = None;
            }
            Stmt::Expr(expr) => {
                collect_constructor_targets_from_expr(
                    expr,
                    &local_bindings,
                    &binding_exprs,
                    &infer_context,
                    &expr_identities,
                    &mut analysis.metadata,
                );
                trailing_expr = Some(expr);
                let (inferred, infer_errors) =
                    infer_expr_type(expr, &local_bindings, &infer_context);
                analysis.errors.extend(infer_errors);
                trailing_expr_type = Some(inferred);
            }
            Stmt::Return(fields) => {
                saw_explicit_return = true;
                trailing_expr_type = None;
                trailing_expr = None;
                analysis.errors.extend(validate_return_stmt(
                    caller,
                    &return_contract,
                    fields,
                    &local_bindings,
                    &binding_exprs,
                    &infer_context,
                    &expr_identities,
                    &mut analysis.metadata,
                ));
            }
        }
    }
    if !saw_explicit_return {
        if let ReturnContract::Single { ty } = &return_contract {
            if let Some(expr) = trailing_expr {
                annotate_expr_with_expected_record(
                    expr,
                    ty,
                    &local_bindings,
                    &binding_exprs,
                    &infer_context,
                    &expr_identities,
                    &mut analysis.metadata,
                    &mut HashSet::new(),
                );
            }
            let inferred = match trailing_expr {
                Some(expr) => {
                    let (val, infer_errors) = infer_expr_type_for_expected_named_record(
                        expr,
                        ty,
                        &local_bindings,
                        &infer_context,
                    );
                    analysis.errors.extend(infer_errors);
                    val
                }
                None => trailing_expr_type.unwrap_or_else(|| ValueType::Named("Unit".to_string())),
            };
            let mismatches = push_type_mismatch_if_needed(
                ty,
                &inferred,
                infer_context.variant_parents,
                infer_context.record_type_registry,
            );
            analysis.errors.extend(mismatches);
        }
    }
    analysis.metadata.finalize_anonymous_record_types(caller);
    let mut resolved_structural_bindings = body_context.data_bindings.clone();
    resolved_structural_bindings.extend(
        params
            .iter()
            .map(|param| (param.name.clone(), value_type_from_type_expr(&param.ty))),
    );
    let mut resolved_bindings = body_context
        .data_bindings
        .iter()
        .map(|(name, ty)| (name.clone(), value_type_to_ir_type(ty)))
        .collect::<HashMap<_, _>>();
    resolved_bindings.extend(params.iter().map(|param| {
        (
            param.name.clone(),
            value_type_to_ir_type(&value_type_from_type_expr(&param.ty)),
        )
    }));
    collect_resolved_expr_ir_types_from_stmts(
        body.stmts,
        &resolved_structural_bindings,
        &resolved_bindings,
        &infer_context,
        &expr_identities,
        &mut analysis.metadata,
    );
    analysis
}

fn collected_expr_ir_type(
    expr: &Expr,
    expr_identities: &StableExprIdentities,
    metadata: &TypedCallableBodyMetadata,
) -> Option<gunbc_ir::code_ir::IrType> {
    metadata
        .expr_ir_types
        .get(&stable_expr_identity(expr, expr_identities))
        .cloned()
}

fn collect_resolved_expr_ir_types_from_stmts(
    stmts: &[Stmt],
    structural_bindings: &HashMap<String, ValueType>,
    resolved_bindings: &HashMap<String, gunbc_ir::code_ir::IrType>,
    infer_context: &ExprInferenceContext<'_>,
    expr_identities: &StableExprIdentities,
    metadata: &mut TypedCallableBodyMetadata,
) {
    let mut structural_scope = structural_bindings.clone();
    let mut resolved_scope = resolved_bindings.clone();
    for stmt in stmts {
        match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                let (raw_ty, resolved_ir) = collect_resolved_expr_ir_types_from_expr(
                    expr,
                    &structural_scope,
                    &resolved_scope,
                    infer_context,
                    expr_identities,
                    metadata,
                );
                structural_scope.insert(name.clone(), raw_ty);
                resolved_scope.insert(name.clone(), resolved_ir);
            }
            Stmt::Node(node_stmt) => {
                let (raw_ty, resolved_ir) = collect_resolved_expr_ir_types_from_expr(
                    &node_stmt.expr,
                    &structural_scope,
                    &resolved_scope,
                    infer_context,
                    expr_identities,
                    metadata,
                );
                structural_scope.insert(node_stmt.name.clone(), raw_ty);
                resolved_scope.insert(node_stmt.name.clone(), resolved_ir);
            }
            Stmt::Expr(expr) => {
                let _ = collect_resolved_expr_ir_types_from_expr(
                    expr,
                    &structural_scope,
                    &resolved_scope,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    let _ = collect_resolved_expr_ir_types_from_expr(
                        expr,
                        &structural_scope,
                        &resolved_scope,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
            }
        }
    }
}

fn collect_resolved_expr_ir_types_from_expr(
    expr: &Expr,
    structural_bindings: &HashMap<String, ValueType>,
    resolved_bindings: &HashMap<String, gunbc_ir::code_ir::IrType>,
    infer_context: &ExprInferenceContext<'_>,
    expr_identities: &StableExprIdentities,
    metadata: &mut TypedCallableBodyMetadata,
) -> (ValueType, gunbc_ir::code_ir::IrType) {
    use gunbc_ir::code_ir::IrType;

    match expr {
        Expr::Literal(_) | Expr::Ident(_) => {}
        Expr::FieldAccess(base, _) | Expr::UnaryOp(_, base) => {
            let _ = collect_resolved_expr_ir_types_from_expr(
                base,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
        }
        Expr::Call(name, args) if name == "fold" && args.len() >= 2 => {
            let init = args
                .iter()
                .find(|(arg_name, _)| arg_name.as_deref() == Some("init"))
                .or_else(|| args.get(1))
                .map(|(_, value)| value);
            let func = args
                .iter()
                .find(|(arg_name, _)| arg_name.as_deref() == Some("f"))
                .or_else(|| args.get(2))
                .map(|(_, value)| value);

            let _ = collect_resolved_expr_ir_types_from_expr(
                &args[0].1,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
            if let Some(init_expr) = init {
                let _ = collect_resolved_expr_ir_types_from_expr(
                    init_expr,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
            match func {
                Some(Expr::Lambda(params, body)) => {
                    let (collection_ty, _) =
                        infer_expr_type(&args[0].1, structural_bindings, infer_context);
                    let (acc_ty, _) = init
                        .map(|init_expr| {
                            infer_fold_accumulator_type(
                                &args[0].1,
                                init_expr,
                                func,
                                structural_bindings,
                                infer_context,
                            )
                        })
                        .unwrap_or((ValueType::Inferred, Vec::new()));
                    let mut lambda_structural = structural_bindings.clone();
                    let mut lambda_resolved = resolved_bindings.clone();
                    if let Some(param) = params.first() {
                        lambda_structural.insert(param.clone(), acc_ty.clone());
                        lambda_resolved.insert(param.clone(), value_type_to_ir_type(&acc_ty));
                    }
                    if let (Some(param), Some(elem_ty)) =
                        (params.get(1), collection_element_value_type(&collection_ty))
                    {
                        lambda_resolved.insert(param.clone(), value_type_to_ir_type(&elem_ty));
                        lambda_structural.insert(param.clone(), elem_ty);
                    }
                    let _ = collect_resolved_expr_ir_types_from_expr(
                        body,
                        &lambda_structural,
                        &lambda_resolved,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
                Some(other) => {
                    let _ = collect_resolved_expr_ir_types_from_expr(
                        other,
                        structural_bindings,
                        resolved_bindings,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
                None => {}
            }
            let (raw_ty, _) = init
                .map(|init_expr| {
                    infer_fold_accumulator_type(
                        &args[0].1,
                        init_expr,
                        func,
                        structural_bindings,
                        infer_context,
                    )
                })
                .unwrap_or((ValueType::Inferred, Vec::new()));
            let resolved_ir = init
                .and_then(|init_expr| {
                    metadata
                        .anonymous_record_target(stable_expr_identity(init_expr, expr_identities))
                        .map(|target| IrType::Named(target.0.clone()))
                })
                .unwrap_or_else(|| value_type_to_ir_type(&raw_ty));
            if let Some(init_expr) = init {
                metadata.annotate_expr_ir_type(
                    stable_expr_identity(init_expr, expr_identities),
                    resolved_ir.clone(),
                );
            }
            metadata.annotate_expr_ir_type(
                stable_expr_identity(expr, expr_identities),
                resolved_ir.clone(),
            );
            return (raw_ty, resolved_ir);
        }
        Expr::Call(name, args) => {
            if matches!(name.as_str(), "map" | "filter" | "any") && args.len() >= 2 {
                let _ = collect_resolved_expr_ir_types_from_expr(
                    &args[0].1,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
                match &args[1].1 {
                    Expr::Lambda(params, body) => {
                        let (collection_ty, _) =
                            infer_expr_type(&args[0].1, structural_bindings, infer_context);
                        let mut lambda_structural = structural_bindings.clone();
                        let mut lambda_resolved = resolved_bindings.clone();
                        if let (Some(param), Some(elem_ty)) = (
                            params.first(),
                            collection_element_value_type(&collection_ty),
                        ) {
                            lambda_resolved.insert(param.clone(), value_type_to_ir_type(&elem_ty));
                            lambda_structural.insert(param.clone(), elem_ty);
                        }
                        let _ = collect_resolved_expr_ir_types_from_expr(
                            body,
                            &lambda_structural,
                            &lambda_resolved,
                            infer_context,
                            expr_identities,
                            metadata,
                        );
                    }
                    other => {
                        let _ = collect_resolved_expr_ir_types_from_expr(
                            other,
                            structural_bindings,
                            resolved_bindings,
                            infer_context,
                            expr_identities,
                            metadata,
                        );
                    }
                }
                for (_, arg_expr) in args.iter().skip(2) {
                    let _ = collect_resolved_expr_ir_types_from_expr(
                        arg_expr,
                        structural_bindings,
                        resolved_bindings,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
            } else {
                for (_, arg_expr) in args {
                    let _ = collect_resolved_expr_ir_types_from_expr(
                        arg_expr,
                        structural_bindings,
                        resolved_bindings,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
            }
        }
        Expr::ServiceCall(_, args) => {
            for (_, arg_expr) in args {
                let _ = collect_resolved_expr_ir_types_from_expr(
                    arg_expr,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::BinOp(lhs, _, rhs) => {
            let _ = collect_resolved_expr_ir_types_from_expr(
                lhs,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
            let _ = collect_resolved_expr_ir_types_from_expr(
                rhs,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    let _ = collect_resolved_expr_ir_types_from_expr(
                        inner,
                        structural_bindings,
                        resolved_bindings,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
            }
        }
        Expr::Record(_, fields) => {
            for (_, field_expr) in fields {
                let _ = collect_resolved_expr_ir_types_from_expr(
                    field_expr,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::Match(scrutinee, arms) => {
            let _ = collect_resolved_expr_ir_types_from_expr(
                scrutinee,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    let _ = collect_resolved_expr_ir_types_from_expr(
                        guard,
                        structural_bindings,
                        resolved_bindings,
                        infer_context,
                        expr_identities,
                        metadata,
                    );
                }
                let _ = collect_resolved_expr_ir_types_from_expr(
                    &arm.body,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            let _ = collect_resolved_expr_ir_types_from_expr(
                cond,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
            let _ = collect_resolved_expr_ir_types_from_expr(
                then_expr,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
            if let Some(otherwise) = else_expr {
                let _ = collect_resolved_expr_ir_types_from_expr(
                    otherwise,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::For(binding, iterable, passthrough, body) => {
            let (iter_ty, iter_ir_ty) = collect_resolved_expr_ir_types_from_expr(
                iterable,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
            let (loop_scope, _) = resolve_for_loop_scope_contract(
                binding,
                &iter_ty,
                passthrough,
                structural_bindings,
            );
            if let Some(loop_scope) = loop_scope {
                let loop_structural = loop_scope.local_bindings(structural_bindings);
                let typed_scope = loop_scope.typed_scope(&iter_ir_ty, resolved_bindings);
                metadata.annotate_for_loop_scope(
                    stable_expr_identity(expr, expr_identities),
                    typed_scope.clone(),
                );
                let loop_resolved = typed_scope.resolved_bindings(resolved_bindings);
                match body {
                    ForBody::Expr(inner) => {
                        let _ = collect_resolved_expr_ir_types_from_expr(
                            inner,
                            &loop_structural,
                            &loop_resolved,
                            infer_context,
                            expr_identities,
                            metadata,
                        );
                    }
                    ForBody::Block(stmts) => collect_resolved_expr_ir_types_from_stmts(
                        stmts,
                        &loop_structural,
                        &loop_resolved,
                        infer_context,
                        expr_identities,
                        metadata,
                    ),
                }
            }
        }
        Expr::Lambda(_, body) => {
            let _ = collect_resolved_expr_ir_types_from_expr(
                body,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
        }
        Expr::List(items) => {
            for item in items {
                let _ = collect_resolved_expr_ir_types_from_expr(
                    item,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                let _ = collect_resolved_expr_ir_types_from_expr(
                    key,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
                let _ = collect_resolved_expr_ir_types_from_expr(
                    value,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::Guarded(inner, guard) => {
            let _ = collect_resolved_expr_ir_types_from_expr(
                inner,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
            let _ = collect_resolved_expr_ir_types_from_expr(
                guard,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
        }
        Expr::After(inner, _) => {
            let _ = collect_resolved_expr_ir_types_from_expr(
                inner,
                structural_bindings,
                resolved_bindings,
                infer_context,
                expr_identities,
                metadata,
            );
        }
        Expr::Return(fields) => {
            for (_, field_expr) in fields {
                let _ = collect_resolved_expr_ir_types_from_expr(
                    field_expr,
                    structural_bindings,
                    resolved_bindings,
                    infer_context,
                    expr_identities,
                    metadata,
                );
            }
        }
        Expr::Block(stmts) => collect_resolved_expr_ir_types_from_stmts(
            stmts,
            structural_bindings,
            resolved_bindings,
            infer_context,
            expr_identities,
            metadata,
        ),
    }

    let expr_identity = stable_expr_identity(expr, expr_identities);
    let (raw_ty, _) = infer_expr_type(expr, structural_bindings, infer_context);
    let default_ir = value_type_to_ir_type(&raw_ty);
    let resolved_ir = match expr {
        Expr::Ident(name) => resolved_bindings
            .get(name)
            .cloned()
            .unwrap_or_else(|| default_ir.clone()),
        Expr::Record(None, _) => metadata
            .anonymous_record_target(expr_identity)
            .map(|target| IrType::Named(target.0.clone()))
            .unwrap_or_else(|| default_ir.clone()),
        Expr::Call(name, args) if name == "with" && !args.is_empty() => {
            collected_expr_ir_type(&args[0].1, expr_identities, metadata)
                .unwrap_or_else(|| default_ir.clone())
        }
        _ => default_ir.clone(),
    };
    metadata.annotate_expr_ir_type(expr_identity, resolved_ir.clone());
    (raw_ty, resolved_ir)
}

#[allow(clippy::too_many_arguments)]
fn validate_return_stmt(
    caller: &str,
    return_contract: &ReturnContract,
    fields: &[(String, Expr)],
    local_bindings: &HashMap<String, ValueType>,
    binding_exprs: &HashMap<String, &Expr>,
    infer_context: &ExprInferenceContext<'_>,
    expr_identities: &StableExprIdentities,
    metadata: &mut TypedCallableBodyMetadata,
) -> Vec<TypeError> {
    let mut errors = Vec::new();
    match return_contract {
        ReturnContract::Single { ty } => {
            if fields.len() != 1 {
                errors.push(TypeError::TypeMismatch {
                    expected: ty.clone(),
                    got: "Record".to_string(),
                });
                return errors;
            }
            annotate_expr_with_expected_record(
                &fields[0].1,
                ty,
                local_bindings,
                binding_exprs,
                infer_context,
                expr_identities,
                metadata,
                &mut HashSet::new(),
            );
            let (inferred, infer_errors) = infer_expr_type_for_expected_named_record(
                &fields[0].1,
                ty,
                local_bindings,
                infer_context,
            );
            errors.extend(infer_errors);
            let mismatches = push_type_mismatch_if_needed(
                ty,
                &inferred,
                infer_context.variant_parents,
                infer_context.record_type_registry,
            );
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
                annotate_expr_with_expected_record(
                    expr,
                    expected_ty,
                    local_bindings,
                    binding_exprs,
                    infer_context,
                    expr_identities,
                    metadata,
                    &mut HashSet::new(),
                );
                let (inferred, infer_errors) = infer_expr_type(expr, local_bindings, infer_context);
                errors.extend(infer_errors);
                let mismatches = push_type_mismatch_if_needed(
                    expected_ty,
                    &inferred,
                    infer_context.variant_parents,
                    infer_context.record_type_registry,
                );
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
        let inferred_name = inferred.display_name();
        inferred_fields.insert(name.clone(), inferred.clone());
        let Some(expected_field_ty) = expected_fields.get(name) else {
            errors.push(TypeError::NoSuchField {
                ty: expected_type.to_string(),
                field: name.clone(),
            });
            compatible = false;
            continue;
        };
        let expected_field_name = expected_field_ty.display_name();
        if !gunbc_ir::type_registry::TypeRegistry::with_core_types().is_compatible(
            &normalize_type_id(&inferred_name),
            &normalize_type_id(&expected_field_name),
        ) {
            errors.push(TypeError::TypeMismatch {
                expected: expected_field_name,
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

fn infer_block_expr_type(
    stmts: &[Stmt],
    local_bindings: &HashMap<String, ValueType>,
    infer_context: &ExprInferenceContext<'_>,
) -> (ValueType, Vec<TypeError>) {
    let mut scope = local_bindings.clone();
    let mut errors = Vec::new();
    let mut trailing_expr_type = ValueType::Named("Unit".to_string());

    for stmt in stmts {
        match stmt {
            Stmt::Let(name, expr) | Stmt::Assign(name, expr) => {
                let (inferred, stmt_errors) = infer_expr_type(expr, &scope, infer_context);
                errors.extend(stmt_errors);
                scope.insert(name.clone(), inferred);
                trailing_expr_type = ValueType::Named("Unit".to_string());
            }
            Stmt::Node(node_stmt) => {
                let (inferred, stmt_errors) =
                    infer_expr_type(&node_stmt.expr, &scope, infer_context);
                errors.extend(stmt_errors);
                scope.insert(node_stmt.name.clone(), inferred);
                trailing_expr_type = ValueType::Named("Unit".to_string());
            }
            Stmt::Expr(expr) => {
                let (inferred, stmt_errors) = infer_expr_type(expr, &scope, infer_context);
                errors.extend(stmt_errors);
                trailing_expr_type = inferred;
            }
            Stmt::Return(fields) => {
                let mut record = HashMap::new();
                for (field_name, expr) in fields {
                    let (inferred, stmt_errors) = infer_expr_type(expr, &scope, infer_context);
                    errors.extend(stmt_errors);
                    record.insert(field_name.clone(), inferred);
                }
                trailing_expr_type = ValueType::Record(record);
            }
        }
    }

    (trailing_expr_type, errors)
}

fn infer_fold_accumulator_type(
    collection_expr: &Expr,
    init_expr: &Expr,
    func: Option<&Expr>,
    local_bindings: &HashMap<String, ValueType>,
    infer_context: &ExprInferenceContext<'_>,
) -> (ValueType, Vec<TypeError>) {
    let (init_ty, mut errors) = infer_expr_type(init_expr, local_bindings, infer_context);
    let body_ty = match func {
        Some(Expr::Lambda(params, body)) => {
            let (collection_ty, collection_errors) =
                infer_expr_type(collection_expr, local_bindings, infer_context);
            errors.extend(collection_errors);
            let mut lambda_scope = local_bindings.clone();
            if let Some(param) = params.first() {
                lambda_scope.insert(param.clone(), init_ty.clone());
            }
            if let (Some(param), Some(elem_ty)) =
                (params.get(1), collection_element_value_type(&collection_ty))
            {
                lambda_scope.insert(param.clone(), elem_ty);
            }
            let (body_ty, body_errors) = infer_expr_type(body, &lambda_scope, infer_context);
            errors.extend(body_errors);
            Some(body_ty)
        }
        Some(other) => {
            let (body_ty, body_errors) = infer_expr_type(other, local_bindings, infer_context);
            errors.extend(body_errors);
            Some(body_ty)
        }
        None => None,
    };

    let value = body_ty
        .map(|body_ty| merge_value_types(init_ty.clone(), body_ty))
        .unwrap_or(init_ty);
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
            .or_else(|| infer_context.data_bindings.get(name).cloned())
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
            .unwrap_or(ValueType::Inferred),
        Expr::FieldAccess(base, field) => {
            let (base_type, base_errors) = infer_expr_type(base, local_bindings, infer_context);
            errors.extend(base_errors);
            match base_type {
                ValueType::Record(fields) => match fields.get(field) {
                    Some(ty) => ty.clone(),
                    None => {
                        errors.push(TypeError::NoSuchField {
                            ty: "Record".to_string(),
                            field: field.clone(),
                        });
                        ValueType::Inferred
                    }
                },
                ValueType::Named(name) => {
                    match resolve_record_fields(
                        strip_optional_type(&name),
                        infer_context.record_type_registry,
                    ) {
                        Some(fields) => match fields.get(field) {
                            Some(ty) => ty.clone(),
                            None => {
                                errors.push(TypeError::NoSuchField {
                                    ty: name,
                                    field: field.clone(),
                                });
                                ValueType::Inferred
                            }
                        },
                        None => ValueType::Inferred,
                    }
                }
                ValueType::Generic(_, _) | ValueType::Inferred => ValueType::Inferred,
            }
        }
        Expr::Call(name, args) if name == "concat" && args.len() >= 2 => args
            .iter()
            .map(|(_, arg)| infer_expr_type(arg, local_bindings, infer_context))
            .fold(ValueType::Inferred, |acc, (arg_ty, arg_errors)| {
                errors.extend(arg_errors);
                merge_value_types(acc, arg_ty)
            }),
        // list_push(list, item) → same type as the list argument
        Expr::Call(name, args) if name == "list_push" && args.len() == 2 => {
            let (list_ty, list_errors) = infer_expr_type(&args[0].1, local_bindings, infer_context);
            errors.extend(list_errors);
            let (_, item_errors) = infer_expr_type(&args[1].1, local_bindings, infer_context);
            errors.extend(item_errors);
            list_ty
        }
        Expr::Call(name, args) if name == "parse_int" && args.len() == 1 => {
            let (_, arg_errors) = infer_expr_type(&args[0].1, local_bindings, infer_context);
            errors.extend(arg_errors);
            ValueType::Named("Int?".to_string())
        }
        Expr::Call(name, args) if name == "map" && args.len() >= 2 => {
            let (collection_ty, collection_errors) =
                infer_expr_type(&args[0].1, local_bindings, infer_context);
            errors.extend(collection_errors);
            let elem_ty = match &args[1].1 {
                Expr::Lambda(params, body) if params.len() == 1 => {
                    let mut lambda_scope = local_bindings.clone();
                    if let Some(collection_elem_ty) = collection_element_value_type(&collection_ty)
                    {
                        lambda_scope.insert(params[0].clone(), collection_elem_ty);
                    }
                    let (body_ty, body_errors) =
                        infer_expr_type(body, &lambda_scope, infer_context);
                    errors.extend(body_errors);
                    body_ty
                }
                Expr::Lambda(_, body) => {
                    let (body_ty, body_errors) =
                        infer_expr_type(body, local_bindings, infer_context);
                    errors.extend(body_errors);
                    body_ty
                }
                other => {
                    let (body_ty, body_errors) =
                        infer_expr_type(other, local_bindings, infer_context);
                    errors.extend(body_errors);
                    body_ty
                }
            };
            ValueType::Generic("List".to_string(), vec![elem_ty])
        }
        Expr::Call(name, args) if name == "filter" && !args.is_empty() => {
            let (collection_ty, collection_errors) =
                infer_expr_type(&args[0].1, local_bindings, infer_context);
            errors.extend(collection_errors);
            if let Some((_, predicate_expr)) = args.get(1) {
                let (_, predicate_errors) =
                    infer_expr_type(predicate_expr, local_bindings, infer_context);
                errors.extend(predicate_errors);
            }
            collection_ty
        }
        Expr::Call(name, args) if name == "with" && !args.is_empty() => {
            let (base_ty, base_errors) = infer_expr_type(&args[0].1, local_bindings, infer_context);
            errors.extend(base_errors);
            if let Some((_, update_expr)) = args.get(1) {
                let (_, update_errors) =
                    infer_expr_type(update_expr, local_bindings, infer_context);
                errors.extend(update_errors);
            }
            base_ty
        }
        Expr::Call(name, args) if name == "fold" && args.len() >= 2 => {
            let init = args
                .iter()
                .find(|(arg_name, _)| arg_name.as_deref() == Some("init"))
                .or_else(|| args.get(1))
                .map(|(_, expr)| expr);
            let func = args
                .iter()
                .find(|(arg_name, _)| arg_name.as_deref() == Some("f"))
                .or_else(|| args.get(2))
                .map(|(_, expr)| expr);
            match init {
                Some(init_expr) => {
                    let (acc_ty, acc_errors) = infer_fold_accumulator_type(
                        &args[0].1,
                        init_expr,
                        func,
                        local_bindings,
                        infer_context,
                    );
                    errors.extend(acc_errors);
                    acc_ty
                }
                None => ValueType::Inferred,
            }
        }
        Expr::Call(name, args) if name == "any" || name == "contains" => {
            for (_, arg) in args {
                let (_, arg_errors) = infer_expr_type(arg, local_bindings, infer_context);
                errors.extend(arg_errors);
            }
            ValueType::Named("Bool".to_string())
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
                    .unwrap_or(ValueType::Inferred)
            }
        }
        Expr::ServiceCall(path, args) => {
            for (_, arg) in args {
                let (_, arg_errors) = infer_expr_type(arg, local_bindings, infer_context);
                errors.extend(arg_errors);
            }
            match resolve_service_call_contract(path, infer_context.service_call_registry) {
                ServiceCallResolution::Resolved(contract) => ValueType::Record(contract.outputs),
                ServiceCallResolution::Ambiguous => ValueType::Inferred,
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
                        | BoundServiceCallResolution::NotBound => ValueType::Inferred,
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
                _ => match (&lhs_ty, &rhs_ty) {
                    (ValueType::Named(lhs), ValueType::Named(rhs))
                        if strip_generic_params(lhs) == strip_generic_params(rhs) =>
                    {
                        lhs_ty
                    }
                    _ => {
                        let merged = merge_value_types(lhs_ty, rhs_ty);
                        if merged.is_inferred() {
                            ValueType::Inferred
                        } else {
                            merged
                        }
                    }
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
                            (name.clone(), val)
                        })
                        .collect(),
                )
            }
        }
        Expr::Match(scrutinee, arms) => {
            let (_scr_ty, scr_errors) = infer_expr_type(scrutinee, local_bindings, infer_context);
            errors.extend(scr_errors);
            let mut arm_types: Vec<ValueType> = Vec::new();
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    let (_, guard_errors) = infer_expr_type(guard, local_bindings, infer_context);
                    errors.extend(guard_errors);
                }
                let (arm_ty, body_errors) =
                    infer_expr_type(&arm.body, local_bindings, infer_context);
                errors.extend(body_errors);
                if !arm_ty.is_inferred() {
                    arm_types.push(arm_ty);
                }
            }
            // WS3-5: Check compatibility across arms
            if arm_types.len() >= 2 {
                let first = arm_types[0].display_name();
                for other in &arm_types[1..] {
                    let (compat, confident) = are_branch_types_compatible(
                        &first,
                        &other.display_name(),
                        infer_context.variant_parents,
                    );
                    if !compat && confident {
                        errors.push(TypeError::MatchArmTypeMismatch {
                            first_type: first.clone(),
                            mismatched_type: other.display_name(),
                        });
                        break;
                    }
                }
            }
            // WS3-6: Exhaustiveness checking infrastructure is available via
            // `check_match_exhaustiveness()`. Not enforced in the main typecheck
            // path because existing DSL code has intentional partial matches.
            //
            // S67: Return the unified arm type when all concrete arms agree,
            // rather than unconditionally returning Inferred.
            if let Some(first) = arm_types.first() {
                if arm_types.iter().all(|ty| ty == first) {
                    first.clone()
                } else {
                    ValueType::Inferred
                }
            } else {
                ValueType::Inferred
            }
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
                Some(ref otherwise) => {
                    // S67: Skip branch unification when either side is Inferred
                    // (insufficient information to compare).
                    if then_ty.is_inferred() || otherwise.is_inferred() {
                        // Prefer the concrete side if one exists.
                        if !then_ty.is_inferred() {
                            then_ty
                        } else if !otherwise.is_inferred() {
                            otherwise.clone()
                        } else {
                            ValueType::Inferred
                        }
                    } else {
                        let t = then_ty.display_name();
                        let e = otherwise.display_name();
                        let (compat, confident) =
                            are_branch_types_compatible(&t, &e, infer_context.variant_parents);
                        if compat {
                            let merged = merge_value_types(then_ty.clone(), otherwise.clone());
                            if !merged.is_inferred() {
                                merged
                            } else {
                                ValueType::Inferred
                            }
                        } else {
                            if confident {
                                errors.push(TypeError::BranchTypeMismatch {
                                    then_type: t,
                                    else_type: e,
                                });
                            }
                            ValueType::Inferred
                        }
                    }
                }
                // No else branch — expression type is the then-branch type
                // only in statement position; as an expression it's Inferred.
                None => ValueType::Inferred,
            }
        }
        Expr::For(binding, iterable, passthrough, body) => {
            let (iter_ty, iter_errors) = infer_expr_type(iterable, local_bindings, infer_context);
            errors.extend(iter_errors);
            let (loop_scope, loop_errors) =
                resolve_for_loop_scope_contract(binding, &iter_ty, passthrough, local_bindings);
            errors.extend(loop_errors);
            if let Some(loop_scope) = loop_scope {
                let loop_scope = loop_scope.local_bindings(local_bindings);
                let (_, body_errors) = match body {
                    ForBody::Expr(expr) => infer_expr_type(expr, &loop_scope, infer_context),
                    ForBody::Block(stmts) => {
                        infer_block_expr_type(stmts, &loop_scope, infer_context)
                    }
                };
                errors.extend(body_errors);
            }
            ValueType::Inferred
        }
        Expr::Lambda(_, body) => {
            let (val, body_errors) = infer_expr_type(body, local_bindings, infer_context);
            errors.extend(body_errors);
            val
        }
        Expr::List(items) => {
            let elem_ty = items
                .iter()
                .map(|item| infer_expr_type(item, local_bindings, infer_context))
                .fold(ValueType::Inferred, |acc, (item_ty, item_errors)| {
                    errors.extend(item_errors);
                    merge_value_types(acc, item_ty)
                });
            ValueType::Generic("List".to_string(), vec![elem_ty])
        }
        Expr::Map(entries) => {
            let mut key_ty = ValueType::Inferred;
            let mut value_ty = ValueType::Inferred;
            for (key, value) in entries {
                let (inferred_key_ty, key_errors) =
                    infer_expr_type(key, local_bindings, infer_context);
                errors.extend(key_errors);
                key_ty = merge_value_types(key_ty, inferred_key_ty);
                let (inferred_value_ty, value_errors) =
                    infer_expr_type(value, local_bindings, infer_context);
                errors.extend(value_errors);
                value_ty = merge_value_types(value_ty, inferred_value_ty);
            }
            ValueType::Generic("Map".to_string(), vec![key_ty, value_ty])
        }
        Expr::Guarded(inner, guard) => {
            let (_, inner_errors) = infer_expr_type(inner, local_bindings, infer_context);
            errors.extend(inner_errors);
            let (_, guard_errors) = infer_expr_type(guard, local_bindings, infer_context);
            errors.extend(guard_errors);
            ValueType::Inferred
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
                    (name.clone(), val)
                })
                .collect(),
        ),
        Expr::Block(stmts) => {
            let (val, block_errors) = infer_block_expr_type(stmts, local_bindings, infer_context);
            errors.extend(block_errors);
            val
        }
    };
    (value, errors)
}

impl ValueType {
    fn display_name(&self) -> String {
        match self {
            Self::Named(name) => name.clone(),
            Self::Generic(name, params) => format!(
                "{name}<{}>",
                params
                    .iter()
                    .map(ValueType::display_name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Record(_) => "Record".to_string(),
            Self::Inferred => "Any".to_string(),
        }
    }

    /// Returns `true` when the typechecker could not determine a concrete type.
    fn is_inferred(&self) -> bool {
        matches!(self, Self::Inferred)
    }
}

/// Check if two type names are compatible for branch unification (WS3-5).
///
/// Resolves variant names to their parent sum types (e.g., `Proceed` and
/// `TerminalFailed` are both `RetryDecision` variants → compatible).
/// Strips `?` for optionality (`T` and `T?` are branch-compatible).
///
/// Returns `(compatible, confident)`:
/// - `compatible=true`: types are known to be the same (after resolution)
/// - `compatible=false, confident=true`: types are clearly incompatible
///   (at least one is a known primitive and they differ)
/// - `compatible=false, confident=false`: types differ but both are DSL-defined;
///   don't emit errors (insufficient info for type system to judge)
fn are_branch_types_compatible(
    a: &str,
    b: &str,
    variant_parents: &HashMap<String, String>,
) -> (bool, bool) {
    let a_resolved = variant_parents.get(a).map(|s| s.as_str()).unwrap_or(a);
    let b_resolved = variant_parents.get(b).map(|s| s.as_str()).unwrap_or(b);
    // Strip trailing `?` for optionality — `T` and `T?` are branch-compatible.
    let a_base = a_resolved.trim_end_matches('?');
    let b_base = b_resolved.trim_end_matches('?');
    // Direct name equality (after variant + optionality resolution)
    if a_base == b_base {
        return (true, true);
    }
    let a_id = normalize_type_id(a_base);
    let b_id = normalize_type_id(b_base);
    let registry = gunbc_ir::type_registry::TypeRegistry::with_core_types();
    let a_known = registry.resolve_type(&a_id).is_some();
    let b_known = registry.resolve_type(&b_id).is_some();
    if a_known || b_known {
        // At least one type is a known primitive — use registry for compatibility
        let compat = registry.is_compatible(&a_id, &b_id) || registry.is_compatible(&b_id, &a_id);
        (compat, true)
    } else {
        // Both types are DSL-defined and unknown to the IR registry.
        // Can't confidently say they're incompatible.
        (false, false)
    }
}

fn push_type_mismatch_if_needed(
    expected: &str,
    inferred: &ValueType,
    variant_parents: &HashMap<String, String>,
    record_type_registry: &RecordTypeRegistry,
) -> Vec<TypeError> {
    // S67: Skip type mismatch when the inferred type is Inferred — the
    // typechecker lacks enough information to judge compatibility.
    if inferred.is_inferred() {
        return Vec::new();
    }
    let got = inferred.display_name();

    // v2 TC003 fix: bare enum variant is compatible with its parent sum type.
    // e.g. returning `UnterminatedString` where `StringScanResult` is expected.
    if let Some(parent) = variant_parents.get(&got) {
        if parent == expected {
            return Vec::new();
        }
    }

    // v2 TC003 fix: "Record"/"Map" is compatible with a known struct type.
    if (got == "Record" || got == "Map")
        && resolve_record_fields(expected, record_type_registry).is_some()
    {
        return Vec::new();
    }

    if !gunbc_ir::type_registry::TypeRegistry::with_core_types()
        .is_compatible(&normalize_type_id(&got), &normalize_type_id(expected))
    {
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
) -> Option<HashMap<String, ValueType>> {
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

/// Check whether an auth scheme string is in the recognized set.
fn is_valid_auth_scheme(scheme: &str) -> bool {
    matches!(scheme, "BearerToken" | "Basic" | "ApiKey" | "None") || scheme.starts_with("Header(")
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

fn field_value_type_map(fields: &[Field]) -> HashMap<String, ValueType> {
    fields
        .iter()
        .map(|field| (field.name.clone(), value_type_from_type_expr(&field.ty)))
        .collect()
}

fn value_type_from_type_expr(expr: &TypeExpr) -> ValueType {
    match expr {
        TypeExpr::Named(name) => ValueType::Named(name.clone()),
        TypeExpr::AssociatedOutput(base) => ValueType::Named(format!("{base}.Output")),
        TypeExpr::Generic(name, args) => ValueType::Generic(
            name.clone(),
            args.iter().map(value_type_from_type_expr).collect(),
        ),
        TypeExpr::Function(_, _) => ValueType::Named(type_expr_to_string(expr)),
        TypeExpr::Optional(_) => ValueType::Named(type_expr_to_string(expr)),
        TypeExpr::Refined(inner, _) => value_type_from_type_expr(inner),
        TypeExpr::Record(fields) => ValueType::Record(field_value_type_map(fields)),
    }
}

fn collection_element_value_type(ty: &ValueType) -> Option<ValueType> {
    match ty {
        ValueType::Generic(_, args) if !args.is_empty() => Some(args[0].clone()),
        _ => None,
    }
}

fn for_loop_element_value_type(ty: &ValueType) -> Option<ValueType> {
    match ty {
        ValueType::Generic(name, args) if name == "List" && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn for_loop_element_ir_type(
    ty: &gunbc_ir::code_ir::IrType,
) -> Option<gunbc_ir::code_ir::IrType> {
    match ty {
        gunbc_ir::code_ir::IrType::Generic(name, args) if name == "List" && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn resolve_for_loop_scope_contract(
    binding: &str,
    iterable_ty: &ValueType,
    passthrough: &[String],
    local_bindings: &HashMap<String, ValueType>,
) -> (Option<ForLoopScopeContract>, Vec<TypeError>) {
    let mut errors = Vec::new();
    let element_binding = if let Some(element_ty) = for_loop_element_value_type(iterable_ty) {
        Some(ForLoopScopeValueBinding {
            name: binding.to_string(),
            value_type: element_ty,
        })
    } else {
        if !iterable_ty.is_inferred() {
            errors.push(TypeError::TypeMismatch {
                expected: "List<T>".to_string(),
                got: iterable_ty.display_name(),
            });
        }
        None
    };
    let mut passthrough_bindings = Vec::with_capacity(passthrough.len());

    for name in passthrough {
        match local_bindings.get(name) {
            Some(passthrough_ty) => passthrough_bindings.push(ForLoopScopeValueBinding {
                name: name.clone(),
                value_type: passthrough_ty.clone(),
            }),
            None => errors.push(TypeError::UnknownForLoopPassthroughBinding {
                binding: name.clone(),
            }),
        }
    }

    let contract = element_binding.and_then(|element_binding| {
        if errors.is_empty() {
            Some(ForLoopScopeContract {
                element_binding,
                passthrough_bindings,
            })
        } else {
            None
        }
    });

    (contract, errors)
}

fn merge_value_types(left: ValueType, right: ValueType) -> ValueType {
    match (left, right) {
        (ValueType::Inferred, other) | (other, ValueType::Inferred) => other,
        (ValueType::Generic(left_name, left_args), ValueType::Generic(right_name, right_args))
            if left_name == right_name && left_args.len() == right_args.len() =>
        {
            ValueType::Generic(
                left_name,
                left_args
                    .into_iter()
                    .zip(right_args)
                    .map(|(left_ty, right_ty)| merge_value_types(left_ty, right_ty))
                    .collect(),
            )
        }
        (ValueType::Record(left_fields), ValueType::Record(right_fields))
            if left_fields.len() == right_fields.len()
                && left_fields
                    .keys()
                    .all(|name| right_fields.contains_key(name)) =>
        {
            ValueType::Record(
                left_fields
                    .into_iter()
                    .map(|(name, left_ty)| {
                        let right_ty = right_fields
                            .get(&name)
                            .cloned()
                            .unwrap_or(ValueType::Inferred);
                        (name, merge_value_types(left_ty, right_ty))
                    })
                    .collect(),
            )
        }
        (left_ty, right_ty) if left_ty == right_ty => left_ty,
        _ => ValueType::Inferred,
    }
}

fn value_type_to_ir_type(ty: &ValueType) -> gunbc_ir::code_ir::IrType {
    use gunbc_ir::code_ir::IrType;

    match ty {
        ValueType::Named(name) if name.ends_with('?') => {
            IrType::Optional(Box::new(value_type_to_ir_type(&ValueType::Named(
                name.trim_end_matches('?').trim().to_string(),
            ))))
        }
        ValueType::Named(name) => match name.as_str() {
            "Bool" => IrType::Bool,
            "Int" => IrType::Int,
            "String" => IrType::Str,
            "Unit" => IrType::Unit,
            _ => IrType::Named(name.clone()),
        },
        ValueType::Generic(name, params) => IrType::Generic(
            name.clone(),
            params.iter().map(value_type_to_ir_type).collect(),
        ),
        ValueType::Record(fields) => {
            let mut ir_fields = fields
                .iter()
                .map(|(name, field_ty)| (name.clone(), value_type_to_ir_type(field_ty)))
                .collect::<Vec<_>>();
            ir_fields.sort_by(|left, right| left.0.cmp(&right.0));
            IrType::Record(ir_fields)
        }
        ValueType::Inferred => IrType::Unknown,
    }
}

fn merge_ir_types(
    left: gunbc_ir::code_ir::IrType,
    right: gunbc_ir::code_ir::IrType,
) -> gunbc_ir::code_ir::IrType {
    use gunbc_ir::code_ir::IrType;

    match (left, right) {
        (IrType::Unknown, other) | (other, IrType::Unknown) => other,
        (IrType::Generic(left_name, left_args), IrType::Generic(right_name, right_args))
            if left_name == right_name && left_args.len() == right_args.len() =>
        {
            IrType::Generic(
                left_name,
                left_args
                    .into_iter()
                    .zip(right_args)
                    .map(|(left_ty, right_ty)| merge_ir_types(left_ty, right_ty))
                    .collect(),
            )
        }
        (IrType::Record(left_fields), IrType::Record(right_fields))
            if left_fields.len() == right_fields.len()
                && left_fields
                    .iter()
                    .map(|(name, _)| name)
                    .eq(right_fields.iter().map(|(name, _)| name)) =>
        {
            IrType::Record(
                left_fields
                    .into_iter()
                    .zip(right_fields)
                    .map(|((name, left_ty), (_, right_ty))| {
                        (name, merge_ir_types(left_ty, right_ty))
                    })
                    .collect(),
            )
        }
        (IrType::Optional(left_inner), IrType::Optional(right_inner)) => {
            IrType::Optional(Box::new(merge_ir_types(*left_inner, *right_inner)))
        }
        (IrType::Tuple(left_items), IrType::Tuple(right_items))
            if left_items.len() == right_items.len() =>
        {
            IrType::Tuple(
                left_items
                    .into_iter()
                    .zip(right_items)
                    .map(|(left_ty, right_ty)| merge_ir_types(left_ty, right_ty))
                    .collect(),
            )
        }
        (left_ty, right_ty) if left_ty == right_ty => left_ty,
        _ => IrType::Unknown,
    }
}

fn merge_record_ir_fields(
    existing: &mut [(String, gunbc_ir::code_ir::IrType)],
    next: &[(String, gunbc_ir::code_ir::IrType)],
) {
    if existing.len() != next.len()
        || !existing
            .iter()
            .map(|(name, _)| name)
            .eq(next.iter().map(|(name, _)| name))
    {
        return;
    }

    for ((_, existing_ty), (_, next_ty)) in existing.iter_mut().zip(next.iter()) {
        *existing_ty = merge_ir_types(existing_ty.clone(), next_ty.clone());
    }
}

fn ir_type_to_type_id(ir_type: &gunbc_ir::code_ir::IrType) -> Option<String> {
    match ir_type {
        gunbc_ir::code_ir::IrType::Named(name) => Some(name.clone()),
        gunbc_ir::code_ir::IrType::Generic(name, args) => {
            let rendered_args = args
                .iter()
                .map(ir_type_to_type_id)
                .collect::<Option<Vec<_>>>()?;
            Some(format!("{name}<{}>", rendered_args.join(", ")))
        }
        gunbc_ir::code_ir::IrType::Optional(inner) => {
            Some(format!("Optional<{}>", ir_type_to_type_id(inner)?))
        }
        gunbc_ir::code_ir::IrType::Bool => Some("Bool".to_string()),
        gunbc_ir::code_ir::IrType::Int => Some("Int".to_string()),
        gunbc_ir::code_ir::IrType::Str => Some("String".to_string()),
        gunbc_ir::code_ir::IrType::Tuple(items) => {
            let rendered_items = items
                .iter()
                .map(ir_type_to_type_id)
                .collect::<Option<Vec<_>>>()?;
            Some(format!("({})", rendered_items.join(", ")))
        }
        gunbc_ir::code_ir::IrType::Unit => Some("Unit".to_string()),
        gunbc_ir::code_ir::IrType::Record(_) | gunbc_ir::code_ir::IrType::Unknown => None,
    }
}

fn capitalize_first_char(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
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
                                        outputs: contract
                                            .outputs
                                            .iter()
                                            .map(|(k, v)| (k.clone(), ValueType::Named(v.clone())))
                                            .collect(),
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
                                        outputs: contract
                                            .outputs
                                            .iter()
                                            .map(|(k, v)| (k.clone(), ValueType::Named(v.clone())))
                                            .collect(),
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
            if is_internal_synthetic_call(name) {
                return;
            }
            calls.push(BodyCall {
                callee: name.clone(),
                arg_count: args.len(),
                named_args: args
                    .iter()
                    .filter_map(|(name, _)| name.clone())
                    .collect::<Vec<_>>(),
            });
        }
    });
}

fn is_internal_synthetic_call(name: &str) -> bool {
    matches!(name, "<expr>" | "as" | "with" | "fn")
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
    type_params: &[String],
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
        TypeExpr::AssociatedOutput(base) => {
            if !type_params.contains(base) {
                errors.push(TypeError::UndefinedType(format!(
                    "{base}.Output: `{base}` is not a type parameter (in {context})"
                )));
            }
        }
        TypeExpr::Generic(name, args) => {
            if name == "Map" && args.len() == 2 {
                let key_type = type_expr_to_string(&args[0]);
                if key_type != "String" {
                    errors.push(TypeError::UnresolvableType {
                        ty: type_expr_to_string(ty),
                        context: context.to_string(),
                    });
                }
            }
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
                    type_params,
                    context,
                ));
            }
        }
        TypeExpr::Function(params, output) => {
            for (index, param) in params.iter().enumerate() {
                errors.extend(validate_type_expr(
                    param,
                    known_types,
                    generic_arity_registry,
                    type_params,
                    &format!("{context}.param{}", index + 1),
                ));
            }
            errors.extend(validate_type_expr(
                output,
                known_types,
                generic_arity_registry,
                type_params,
                &format!("{context}.return"),
            ));
        }
        TypeExpr::Optional(inner) => {
            errors.extend(validate_type_expr(
                inner,
                known_types,
                generic_arity_registry,
                type_params,
                context,
            ));
        }
        TypeExpr::Refined(inner, refinements) => {
            errors.extend(validate_type_expr(
                inner,
                known_types,
                generic_arity_registry,
                type_params,
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
                        if let Err(constraint) = parse_surface_content_encoding(enc) {
                            errors.push(TypeError::UnsatisfiableRefinement {
                                ty: type_expr_to_string(inner),
                                constraint,
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
                    Refinement::Width(expr) => {
                        if let Some(v) = extract_int_literal(expr) {
                            if v < 1 || v > u16::MAX as i64 {
                                errors.push(TypeError::UnsatisfiableRefinement {
                                    ty: type_expr_to_string(inner),
                                    constraint: format!(
                                        "width({v}) out of range — must be 1..{}",
                                        u16::MAX
                                    ),
                                });
                            }
                        }
                    }
                    Refinement::Length(expr) => {
                        if let Some(v) = extract_int_literal(expr) {
                            if v < 0 {
                                errors.push(TypeError::UnsatisfiableRefinement {
                                    ty: type_expr_to_string(inner),
                                    constraint: format!("length({v}) must be non-negative"),
                                });
                            }
                        }
                    }
                    Refinement::NonEmpty
                    | Refinement::Format(_)
                    | Refinement::Predicate(_)
                    | Refinement::RawBody
                    | Refinement::FileTypes(_)
                    | Refinement::Signed(_)
                    | Refinement::Unsigned
                    | Refinement::Arithmetic
                    | Refinement::Domain(_) => {}
                }
            }
        }
        TypeExpr::Record(fields) => {
            for field in fields {
                errors.extend(validate_type_expr(
                    &field.ty,
                    known_types,
                    generic_arity_registry,
                    type_params,
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

fn parse_surface_content_encoding(raw: &str) -> Result<gunbc_ir::type_op::ContentEncoding, String> {
    gunbc_ir::type_op::SurfaceContentEncoding::parse(raw)
        .map(Into::into)
        .ok_or_else(|| {
            let expected = gunbc_ir::type_op::SurfaceContentEncoding::ALL
                .iter()
                .map(|encoding| encoding.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!("unknown content encoding `{raw}` — expected one of: {expected}")
        })
}

fn extract_int_literal(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Literal(Literal::Int(value)) => Some(*value),
        _ => None,
    }
}

/// Extract IR predicates from a type expression's refinements.
///
/// Walks through `Refined(inner, refinements)` wrappers and converts
/// each `Refinement` variant to its corresponding `Predicate`.
fn collect_predicates_from_type_expr(type_expr: &TypeExpr) -> Vec<gunbc_ir::type_op::Predicate> {
    let mut predicates = Vec::new();
    collect_predicates_recursive(type_expr, &mut predicates);
    predicates
}

fn collect_predicates_recursive(
    type_expr: &TypeExpr,
    predicates: &mut Vec<gunbc_ir::type_op::Predicate>,
) {
    if let TypeExpr::Refined(inner, refinements) = type_expr {
        collect_predicates_recursive(inner, predicates);
        for refinement in refinements {
            if let Some(pred) = refinement_to_predicate(refinement) {
                predicates.push(pred);
            }
        }
    }
}

/// Extract the brand name from a type expression's refinements (if any).
fn collect_brand_from_type_expr(type_expr: &TypeExpr) -> Option<String> {
    match type_expr {
        TypeExpr::Refined(inner, refinements) => {
            // Check current level first
            for r in refinements {
                if let Refinement::Brand(name) = r {
                    return Some(name.clone());
                }
            }
            // Recurse into inner
            collect_brand_from_type_expr(inner)
        }
        _ => None,
    }
}

fn refinement_to_predicate(refinement: &Refinement) -> Option<gunbc_ir::type_op::Predicate> {
    use gunbc_ir::type_op::Predicate;
    match refinement {
        Refinement::Pattern(regex) => Some(Predicate::Matches(regex.clone())),
        Refinement::Range { min, max } => {
            let min_val = min
                .as_ref()
                .and_then(extract_int_literal)
                .unwrap_or(i64::MIN);
            let max_val = max
                .as_ref()
                .and_then(extract_int_literal)
                .unwrap_or(i64::MAX);
            Some(Predicate::InRange {
                min: min_val,
                max: max_val,
            })
        }
        Refinement::NonEmpty => Some(Predicate::NonEmpty),
        Refinement::Content(enc) => parse_surface_content_encoding(enc)
            .ok()
            .map(Predicate::Content),
        Refinement::Width(expr) => extract_int_literal(expr).and_then(|v| {
            u16::try_from(v)
                .ok()
                .filter(|&w| w > 0)
                .map(Predicate::Width)
        }),
        Refinement::Length(expr) => extract_int_literal(expr).map(|v| Predicate::Length(v as u64)),
        Refinement::Signed(repr) => Some(Predicate::Signed(repr.clone())),
        Refinement::Unsigned => Some(Predicate::Unsigned),
        Refinement::Arithmetic => Some(Predicate::Arithmetic),
        Refinement::Domain(dom) => Some(Predicate::Domain(dom.clone())),
        // Brand is handled structurally (not as a predicate)
        Refinement::Brand(_) => None,
        // These are surface-level annotations, not type predicates
        Refinement::Format(_)
        | Refinement::Predicate(_)
        | Refinement::RawBody
        | Refinement::FileTypes(_) => None,
    }
}

fn should_validate_named_type(name: &str) -> bool {
    name.chars()
        .all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '.'))
}

#[cfg(test)]
mod tests;
