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

use std::collections::HashSet;
use std::path::PathBuf;

use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_syntax::ast::{Field, Item, Param, SourceFile, TypeExpr};

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
    let available_modules = graph
        .modules
        .iter()
        .map(|module| module.module_path.join("."))
        .collect::<HashSet<_>>();
    let mut errors = Vec::new();
    let mut typed_modules = Vec::with_capacity(graph.modules.len());

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
        let signatures = collect_signatures(&module, &known_types, &module_name, &mut errors);
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

fn collect_signatures(
    module: &ResolvedModule,
    known_types: &HashSet<String>,
    module_name: &str,
    errors: &mut Vec<TypeError>,
) -> Vec<TypedItemSignature> {
    let mut module_known_types = known_types.clone();
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
                signatures.push(TypedItemSignature::Service {
                    name: def.name.clone(),
                    operations: def.operations.len(),
                });
            }
            Item::ResourceDef(def) => {
                record_duplicate_item_name(module_name, &def.name, &mut seen_items, errors);
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
}
