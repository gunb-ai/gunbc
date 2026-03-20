//! DSL → Rust code IR bridge.
//!
//! Converts DSL AST nodes into `code_ir::Item` so they can be rendered
//! to Rust source via the existing `render_rust` pipeline.
//!
//! ## Type mapping
//!   - `TypeBody::Record`  → `StructDef`  (all fields `pub`)
//!   - `TypeBody::Sum`     → `EnumDef`    (unit or struct variants)
//!   - `TypeBody::Alias`   → `Raw` type alias
//!
//! ## Data mapping
//!   - `data name: List<T> = [...]` → `pub static NAME: &[T] = &[...]`
//!   - `data name: T = {...}`       → `pub static NAME: T = T {...}`
//!
//! ## Fn mapping
//!   - `fn name(params) -> Ret`     → `pub fn name(params) -> Ret` signature

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use daglang_syntax::ast::{DataDef, Expr, FnDef, Literal, TypeBody, TypeDef, TypeExpr, Variant};
use daglang_syntax::span::Spanned;
use gunbc_ir::code_ir::{self, EnumDef, SourceFile, StructDef};

use crate::fn_codegen;

/// Default derives applied to every generated type.
const DEFAULT_DERIVES: &[&str] = &["Debug", "Clone", "PartialEq", "Eq"];

// ---------------------------------------------------------------------------
// R8: Rc-wrapped types — DAG value semantics as shared ownership
// ---------------------------------------------------------------------------

thread_local! {
    /// Set of type names that should be Rc-wrapped in generated code.
    /// Populated at crate assembly time in `assemble_v2_crate`.
    static RC_WRAPPED_TYPES: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Install the set of Rc-wrapped type names for the duration of `f`.
/// All calls to `type_expr_to_rust` within `f` will Rc-wrap Named types
/// whose names appear in this set.
pub fn with_rc_wrapped_types<R>(types: &HashSet<String>, f: impl FnOnce() -> R) -> R {
    RC_WRAPPED_TYPES.with(|cell| {
        let prev = cell.replace(types.clone());
        let result = f();
        cell.replace(prev);
        result
    })
}

/// Check whether a type name is in the current Rc-wrapped set.
pub fn is_rc_wrapped(name: &str) -> bool {
    RC_WRAPPED_TYPES.with(|cell| cell.borrow().contains(name))
}

/// Get a clone of the current Rc-wrapped types set (for populating CompileContext).
pub fn current_rc_wrapped_types() -> HashSet<String> {
    RC_WRAPPED_TYPES.with(|cell| cell.borrow().clone())
}

/// Build the set of non-Copy type names that should be Rc-wrapped.
///
/// This includes all generated struct types and all non-simple enum types,
/// excluding hardcoded/materialized types (SourceSpan, BindingPower).
pub fn build_rc_wrapped_types(type_defs: &[&TypeDef]) -> HashSet<String> {
    let hardcoded_exclude: HashSet<&str> =
        ["SourceSpan", "BindingPower", "FilePath", "NonEmptyStr"]
            .iter()
            .copied()
            .collect();
    type_defs
        .iter()
        .filter_map(|td| {
            if hardcoded_exclude.contains(td.name.as_str()) {
                return None;
            }
            match &td.body {
                TypeBody::Sum(variants) if is_simple_enum(variants) => None,
                TypeBody::Alias(_) => None,
                _ => Some(td.name.clone()),
            }
        })
        .collect()
}

type CallableReturnTypes = std::collections::HashMap<String, String>;
type CallableParamTypes = std::collections::HashMap<String, Vec<(String, String)>>;

fn register_unique_callable_type<T>(
    map: &mut std::collections::HashMap<String, Option<T>>,
    name: &str,
    value: T,
) {
    map.entry(name.to_string())
        .and_modify(|existing| *existing = None)
        .or_insert(Some(value));
}

fn collect_callable_type_maps_from_signatures<'a>(
    signatures: impl IntoIterator<Item = &'a daglang_typecheck::TypedItemSignature>,
) -> (CallableReturnTypes, CallableParamTypes) {
    let mut return_types = std::collections::HashMap::<String, Option<String>>::new();
    let mut param_types = std::collections::HashMap::<String, Option<Vec<(String, String)>>>::new();

    for signature in signatures {
        let callable = match signature {
            daglang_typecheck::TypedItemSignature::Fn(callable)
            | daglang_typecheck::TypedItemSignature::Func(callable)
            | daglang_typecheck::TypedItemSignature::Pattern(callable) => callable,
            _ => continue,
        };
        register_unique_callable_type(
            &mut param_types,
            &callable.name,
            callable
                .params
                .iter()
                .map(|binding| (binding.name.clone(), binding.ty.0.clone()))
                .collect(),
        );
        if let [binding] = callable.outputs.as_slice() {
            if binding.name == "return" {
                register_unique_callable_type(
                    &mut return_types,
                    &callable.name,
                    binding.ty.0.clone(),
                );
            }
        }
    }

    (
        return_types
            .into_iter()
            .filter_map(|(name, ty)| ty.map(|ty| (name, ty)))
            .collect(),
        param_types
            .into_iter()
            .filter_map(|(name, params)| params.map(|params| (name, params)))
            .collect(),
    )
}

fn collect_anonymous_record_targets(
    metadata: Option<&daglang_typecheck::TypedCallableBodyMetadata>,
) -> std::collections::HashMap<daglang_syntax::ast_utils::ExprIdentity, String> {
    metadata
        .into_iter()
        .flat_map(|metadata| metadata.anonymous_record_targets())
        .map(|(expr_identity, target)| (expr_identity, target.0.clone()))
        .collect()
}

fn collect_synthesized_anonymous_record_types(
    metadata: Option<&daglang_typecheck::TypedCallableBodyMetadata>,
) -> Vec<fn_codegen::SynthesizedAnonymousRecordType> {
    metadata
        .into_iter()
        .flat_map(|metadata| metadata.synthesized_anonymous_record_types().iter())
        .map(|synthesized| fn_codegen::SynthesizedAnonymousRecordType {
            name: synthesized.name.0.clone(),
            fields: synthesized.fields.clone(),
        })
        .collect()
}

fn collect_expr_ir_types(
    metadata: Option<&daglang_typecheck::TypedCallableBodyMetadata>,
) -> std::collections::HashMap<daglang_syntax::ast_utils::ExprIdentity, gunbc_ir::code_ir::IrType> {
    metadata
        .into_iter()
        .flat_map(|metadata| metadata.expr_ir_types())
        .map(|(expr_identity, ir_type)| (expr_identity, ir_type.clone()))
        .collect()
}

fn collect_data_ir_types<'a>(
    data_defs: impl IntoIterator<Item = &'a DataDef>,
) -> std::collections::HashMap<String, gunbc_ir::code_ir::IrType> {
    data_defs
        .into_iter()
        .map(|dd| (dd.name.clone(), fn_codegen::type_expr_to_ir_type(&dd.ty)))
        .collect()
}

/// Convert a DSL `TypeExpr` to a Rust type string.
fn type_expr_to_rust(expr: &TypeExpr) -> String {
    type_expr_to_rust_with_registry(expr, None)
}

/// Public version of type_expr_to_rust for use in v2_crate_emit.
pub fn type_expr_to_rust_pub(expr: &TypeExpr) -> String {
    type_expr_to_rust(expr)
}

/// Convert a DSL `TypeExpr` to a Rust type string, using the registry when
/// available for structural type resolution.
///
/// All named types are resolved through `resolve_and_emit` which uses the
/// structural path when a registry is available, falling back to identity-type
/// name-based mapping for opaque types.
pub fn type_expr_to_rust_with_registry(
    expr: &TypeExpr,
    registry: Option<&gunbc_ir::TypeRegistry>,
) -> String {
    match expr {
        TypeExpr::Named(name) => {
            let resolved = crate::type_mapping::resolve_and_emit(
                name,
                registry,
                crate::type_mapping::Backend::Rust,
            );
            // R8: Rc-wrap non-Copy generated types for O(1) clone
            if is_rc_wrapped(name) {
                format!("Rc<{}>", resolved)
            } else {
                resolved
            }
        }
        TypeExpr::AssociatedOutput(base) => {
            let resolved_base = crate::type_mapping::resolve_and_emit(
                base,
                registry,
                crate::type_mapping::Backend::Rust,
            );
            format!("{resolved_base}::Output")
        }
        TypeExpr::Generic(name, args) => {
            use crate::language_model::{self, ContainerKind};
            let model = language_model::model_for_backend(crate::type_mapping::Backend::Rust);
            let arg_strs: Vec<String> = args
                .iter()
                .map(|a| type_expr_to_rust_with_registry(a, registry))
                .collect();
            let container_kind = match name.as_str() {
                "List" => Some(ContainerKind::List),
                "Set" => Some(ContainerKind::Set),
                "Map" => Some(ContainerKind::Map),
                _ => None,
            };
            if let Some(kind) = container_kind {
                let inner = arg_strs.last().cloned().unwrap_or_default();
                let key = if arg_strs.len() > 1 {
                    Some(arg_strs[0].as_str())
                } else {
                    None
                };
                let resolved = language_model::resolve_container(kind, &inner, key, model)
                    .unwrap_or_else(|| format!("{}<{}>", name, arg_strs.join(", ")));
                // Wrap List and Map in Rc<> for O(1) clone (S76 fix)
                if kind == ContainerKind::List || kind == ContainerKind::Map {
                    format!("Rc<{}>", resolved)
                } else {
                    resolved
                }
            } else {
                let mapped = crate::type_mapping::resolve_and_emit(
                    name,
                    registry,
                    crate::type_mapping::Backend::Rust,
                );
                format!("{}<{}>", mapped, arg_strs.join(", "))
            }
        }
        TypeExpr::Function(params, output) => format!(
            "fn({}) -> {}",
            params
                .iter()
                .map(|param| type_expr_to_rust_with_registry(param, registry))
                .collect::<Vec<_>>()
                .join(", "),
            type_expr_to_rust_with_registry(output, registry)
        ),
        TypeExpr::Optional(inner) => {
            format!(
                "Option<{}>",
                type_expr_to_rust_with_registry(inner, registry)
            )
        }
        TypeExpr::Refined(inner, refinements) => {
            let mut props = gunbc_ir::StructuralProperties::default();
            for r in refinements {
                match r {
                    daglang_syntax::ast::Refinement::Width(daglang_syntax::ast::Expr::Literal(
                        daglang_syntax::ast::Literal::Int(w),
                    )) => props.width = Some(*w as u16),
                    daglang_syntax::ast::Refinement::Signed(_) => props.signed = Some(true),
                    daglang_syntax::ast::Refinement::Unsigned => props.signed = Some(false),
                    daglang_syntax::ast::Refinement::Arithmetic => props.arithmetic = true,
                    daglang_syntax::ast::Refinement::Domain(d) => props.domain = Some(d.clone()),
                    _ => {}
                }
            }
            if props.width.is_some()
                || props.signed.is_some()
                || props.domain.is_some()
                || props.arithmetic
            {
                return crate::type_mapping::emit_shape(
                    &gunbc_ir::TypeShape::Platform(props),
                    crate::type_mapping::Backend::Rust,
                );
            }
            type_expr_to_rust_with_registry(inner, registry)
        }
        TypeExpr::Record(fields) => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|f| {
                    format!(
                        "{}: {}",
                        f.name,
                        type_expr_to_rust_with_registry(&f.ty, registry)
                    )
                })
                .collect();
            format!("{{ {} }}", field_strs.join(", "))
        }
    }
}

/// Check whether all variants of a sum type are simple (no fields).
fn is_simple_enum(variants: &[Variant]) -> bool {
    variants.iter().all(|v| v.fields.is_empty())
}

/// Check whether a type expression is compatible with `Default` derive.
fn is_default_compatible(ty: &TypeExpr) -> bool {
    match ty {
        TypeExpr::Optional(_) => true,
        TypeExpr::Named(name) => name == "Bool",
        _ => false,
    }
}

/// Check whether all fields of a record are default-compatible.
fn all_fields_default_compatible(fields: &[daglang_syntax::ast::Field]) -> bool {
    fields.iter().all(|f| is_default_compatible(&f.ty))
}

/// Convert a single `TypeDef` into one or more code IR items.
pub fn typedef_to_code_ir(td: &TypeDef) -> Vec<code_ir::Item> {
    let derives: Vec<String> = DEFAULT_DERIVES.iter().map(|s| s.to_string()).collect();

    match &td.body {
        TypeBody::Record(fields) => {
            let struct_fields: Vec<(String, String, bool)> = fields
                .iter()
                .map(|f| (f.name.clone(), type_expr_to_rust(&f.ty), true))
                .collect();
            let mut derives = derives;
            if all_fields_default_compatible(fields) {
                derives.push("Default".to_string());
            }
            vec![code_ir::Item::Struct(StructDef {
                name: td.name.clone(),
                is_pub: true,
                derives,
                fields: struct_fields,
                doc: vec![],
            })]
        }
        TypeBody::Sum(variants) => {
            let mut derives = derives;
            if is_simple_enum(variants) {
                derives.push("Copy".to_string());
                derives.push("Hash".to_string());
            }

            let variant_strs: Vec<String> = variants.iter().map(format_variant).collect();

            let mut items = vec![code_ir::Item::Enum(EnumDef {
                name: td.name.clone(),
                is_pub: true,
                derives,
                variants: variant_strs,
                doc: vec![],
            })];

            // Generate accessor methods for fields common to all variants
            if let Some(impl_block) = enum_accessor_impl(&td.name, variants) {
                items.push(code_ir::Item::Raw(impl_block));
            }

            items
        }
        TypeBody::Alias(type_expr) => {
            let rust_type = type_expr_to_rust(type_expr);
            vec![code_ir::Item::Raw(format!(
                "pub type {} = {};",
                td.name, rust_type
            ))]
        }
    }
}

/// Compute fields that appear in every variant of a sum type.
/// Returns (field_name, rust_type_str) pairs for fields present in all variants with the same type.
pub fn common_enum_fields(variants: &[Variant]) -> Vec<(String, String)> {
    if variants.is_empty() {
        return vec![];
    }
    // Only consider variants that have fields (skip unit variants)
    let struct_variants: Vec<&Variant> = variants.iter().filter(|v| !v.fields.is_empty()).collect();
    if struct_variants.len() != variants.len() {
        return vec![]; // Not all variants have fields
    }
    // Start with fields from first variant
    let first_fields: HashMap<&str, &TypeExpr> = struct_variants[0]
        .fields
        .iter()
        .map(|f| (f.name.as_str(), &f.ty))
        .collect();
    // Keep only fields present in ALL variants with matching types
    first_fields
        .into_iter()
        .filter(|(name, ty)| {
            let ty_str = type_expr_to_rust(ty);
            struct_variants[1..].iter().all(|v| {
                v.fields
                    .iter()
                    .any(|f| f.name == *name && type_expr_to_rust(&f.ty) == ty_str)
            })
        })
        .map(|(name, ty)| (name.to_string(), type_expr_to_rust(ty)))
        .collect()
}

/// Build map from enum name → set of common field names across all type definitions.
pub fn build_enum_accessor_fields(type_defs: &[&TypeDef]) -> HashMap<String, HashSet<String>> {
    let mut map = HashMap::new();
    for td in type_defs {
        if let TypeBody::Sum(variants) = &td.body {
            let common = common_enum_fields(variants);
            if !common.is_empty() {
                map.insert(
                    td.name.clone(),
                    common.iter().map(|(name, _)| name.clone()).collect(),
                );
            }
        }
    }
    map
}

/// Generate accessor method `impl` block for an enum with common fields.
/// Returns cloned values (not references) to match the ownership semantics
/// of the DSL's field access, where `x.field` produces an owned value.
fn enum_accessor_impl(enum_name: &str, variants: &[Variant]) -> Option<String> {
    let common = common_enum_fields(variants);
    if common.is_empty() {
        return None;
    }
    let mut methods = Vec::new();
    for (field_name, rust_type) in &common {
        let arms: Vec<String> = variants
            .iter()
            .map(|v| {
                format!(
                    "{}::{} {{ {}, .. }} => {}.clone()",
                    enum_name, v.name, field_name, field_name
                )
            })
            .collect();
        methods.push(format!(
            "    pub fn {field_name}(&self) -> {rust_type} {{\n        match self {{\n            {arms}\n        }}\n    }}",
            arms = arms.join(",\n            ")
        ));
    }
    Some(format!(
        "impl {} {{\n{}\n}}",
        enum_name,
        methods.join("\n\n")
    ))
}

/// Like `typedef_to_code_ir` but maps `String` → `&'static str` for static data compatibility.
pub fn typedef_to_static_code_ir(td: &TypeDef) -> Vec<code_ir::Item> {
    let derives: Vec<String> = DEFAULT_DERIVES.iter().map(|s| s.to_string()).collect();

    match &td.body {
        TypeBody::Record(fields) => {
            let struct_fields: Vec<(String, String, bool)> = fields
                .iter()
                .map(|f| (f.name.clone(), type_expr_to_static_rust(&f.ty), true))
                .collect();
            vec![code_ir::Item::Struct(StructDef {
                name: td.name.clone(),
                is_pub: true,
                derives,
                fields: struct_fields,
                doc: vec![],
            })]
        }
        // Sum and Alias don't have String fields in practice; delegate.
        _ => typedef_to_code_ir(td),
    }
}

/// Convert DSL type expr to Rust, mapping String → &'static str.
/// NOTE: does NOT Rc-wrap types because static data requires Sync.
fn type_expr_to_static_rust(expr: &TypeExpr) -> String {
    match expr {
        TypeExpr::Named(name) => {
            if name == "String" {
                "&'static str".to_string()
            } else {
                crate::type_mapping::resolve_and_emit(
                    name,
                    None,
                    crate::type_mapping::Backend::Rust,
                )
            }
        }
        TypeExpr::Generic(name, args) => {
            let mapped = match name.as_str() {
                "List" => "Rc<Vec".to_string(),
                "Map" => "std::collections::HashMap".to_string(),
                "Set" => "std::collections::HashSet".to_string(),
                other => crate::type_mapping::resolve_and_emit(
                    other,
                    None,
                    crate::type_mapping::Backend::Rust,
                ),
            };
            let arg_strs: Vec<String> = args.iter().map(type_expr_to_static_rust).collect();
            if name == "List" {
                format!("{}<{}>>", mapped, arg_strs.join(", "))
            } else {
                format!("{}<{}>", mapped, arg_strs.join(", "))
            }
        }
        TypeExpr::Optional(inner) => {
            format!("Option<{}>", type_expr_to_static_rust(inner))
        }
        _ => type_expr_to_rust(expr),
    }
}

/// Format a variant for the enum definition string.
fn format_variant(v: &Variant) -> String {
    if v.fields.is_empty() {
        v.name.clone()
    } else {
        let field_strs: Vec<String> = v
            .fields
            .iter()
            .map(|f| format!("{}: {}", f.name, type_expr_to_rust(&f.ty)))
            .collect();
        format!("{} {{ {} }}", v.name, field_strs.join(", "))
    }
}

// ---------------------------------------------------------------------------
// DataDef → Rust static data
// ---------------------------------------------------------------------------

/// Convert a DSL `DataDef` to a Rust static item.
///
/// `data standard_symbols: List<SymbolEntry> = [...]`
///  → `pub static STANDARD_SYMBOLS: &[SymbolEntry] = &[...]`
pub fn datadef_to_code_ir(dd: &DataDef) -> Vec<code_ir::Item> {
    datadef_to_code_ir_with(dd, &[])
}

/// Convert data def to code IR with struct definitions for field-type resolution.
pub fn datadef_to_code_ir_with(dd: &DataDef, struct_defs: &[&TypeDef]) -> Vec<code_ir::Item> {
    let rust_name = to_screaming_snake(&dd.name);
    let (rust_type, rust_value) = match &dd.ty {
        TypeExpr::Generic(name, args) if name == "List" && args.len() == 1 => {
            let elem_type = type_expr_to_rust(&args[0]);
            // TEMPORARY bootstrap scaffolding (S81): In static context, List<String>
            // must emit &[&str] because string literals are &str, not String.
            let is_string_list = matches!(&args[0], TypeExpr::Named(n) if n == "String");
            let static_elem_type = if is_string_list {
                "&str".to_string()
            } else {
                elem_type.clone()
            };
            let elem_type_name = match &args[0] {
                TypeExpr::Named(n) => n.as_str(),
                _ => &elem_type,
            };
            let field_types = resolve_field_types(elem_type_name, struct_defs);
            let items = match &dd.value {
                Expr::List(elements) => elements
                    .iter()
                    .map(|e| render_data_record(e, &elem_type, &field_types))
                    .collect::<Vec<_>>()
                    .join(",\n    "),
                _ => "compile_error!(\"unsupported data value in type_codegen\")".to_string(),
            };
            (
                format!("&[{static_elem_type}]"),
                format!("&[\n    {items}\n]"),
            )
        }
        // Map<K, V> data: emit as a LazyLock static (HashMap can't be const)
        // R8: Don't Rc-wrap types in static context — Rc is not Sync.
        // Strip Rc wrapping from the resolved types.
        TypeExpr::Generic(name, args) if name == "Map" && args.len() == 2 => {
            let key_type = strip_rc_wrapper(&type_expr_to_rust(&args[0]));
            let val_type = strip_rc_wrapper(&type_expr_to_rust(&args[1]));
            let val_type_name = match &args[1] {
                TypeExpr::Named(n) => n.as_str(),
                _ => &val_type,
            };
            let value = render_expr_to_rust(&dd.value, val_type_name, STATIC_OPTS);
            // Emit as a LazyLock static with SCREAMING_SNAKE name so compile_ident's
            // `NAME.clone()` pattern works for both List statics and Map statics.
            return vec![code_ir::Item::Raw(format!(
                "pub static {rust_name}: std::sync::LazyLock<std::collections::HashMap<{key_type}, {val_type}>> = std::sync::LazyLock::new(|| {{\n    {value}\n}});"
            ))];
        }
        _ => {
            let rust_ty = type_expr_to_rust(&dd.ty);
            let value = render_expr_to_rust(&dd.value, &rust_ty, STATIC_OPTS);
            (rust_ty, value)
        }
    };
    vec![code_ir::Item::Raw(format!(
        "pub static {rust_name}: {rust_type} = {rust_value};"
    ))]
}

/// Build a field-name → Rust-type-name map from struct definitions.
fn resolve_field_types(struct_name: &str, struct_defs: &[&TypeDef]) -> Vec<(String, String)> {
    for td in struct_defs {
        if td.name == struct_name {
            if let TypeBody::Record(fields) = &td.body {
                return fields
                    .iter()
                    .map(|f| (f.name.clone(), type_expr_to_rust_name(&f.ty)))
                    .collect();
            }
        }
    }
    vec![]
}

/// Extract field types for a data table's element type.
fn resolve_field_types_for_data(dd: &DataDef, struct_defs: &[&TypeDef]) -> Vec<(String, String)> {
    let elem_type_name = match &dd.ty {
        TypeExpr::Generic(name, args) if name == "List" && args.len() == 1 => match &args[0] {
            TypeExpr::Named(n) => n.as_str(),
            _ => return vec![],
        },
        _ => return vec![],
    };
    resolve_field_types(elem_type_name, struct_defs)
}

/// R8: Strip `Rc<...>` wrapper from a Rust type string.
/// Used for static data tables where Rc is not Sync-compatible.
fn strip_rc_wrapper(rust_type: &str) -> String {
    if let Some(inner) = rust_type
        .strip_prefix("Rc<")
        .and_then(|s| s.strip_suffix('>'))
    {
        inner.to_string()
    } else {
        rust_type.to_string()
    }
}

/// Get the simple type name (without Option wrapping) for field context resolution.
fn type_expr_to_rust_name(expr: &TypeExpr) -> String {
    match expr {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::AssociatedOutput(base) => format!("{base}::Output"),
        TypeExpr::Optional(inner) => type_expr_to_rust_name(inner),
        TypeExpr::Generic(name, _) => name.clone(),
        TypeExpr::Function(_, _) => "Function".to_string(),
        TypeExpr::Refined(inner, _) => type_expr_to_rust_name(inner),
        TypeExpr::Record(_) => "Anonymous".to_string(),
    }
}

/// Render a record expression for a data table entry,
/// using field type info to qualify enum variant references.
fn render_data_record(expr: &Expr, context_type: &str, field_types: &[(String, String)]) -> String {
    match expr {
        Expr::Record(maybe_name, fields) => {
            let type_name = maybe_name.as_deref().unwrap_or(context_type);
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(name, val)| {
                    let field_type = field_types
                        .iter()
                        .find(|(n, _)| n == name)
                        .map(|(_, t)| t.as_str())
                        .unwrap_or(name.as_str());
                    let field_val = render_expr_to_rust(val, field_type, STATIC_OPTS);
                    format!("{name}: {field_val}")
                })
                .collect();
            format!("{type_name} {{ {} }}", field_strs.join(", "))
        }
        other => render_expr_to_rust(other, context_type, STATIC_OPTS),
    }
}

/// Options controlling expression rendering.
#[derive(Clone, Copy)]
struct RenderOpts {
    /// When true, string literals render as `"..."` (for static contexts)
    /// instead of `"...".to_string()` (for runtime contexts).
    static_context: bool,
}

const STATIC_OPTS: RenderOpts = RenderOpts {
    static_context: true,
};

/// Render a DSL expression to a Rust expression string.
///
/// `context_type` is the Rust type name for the surrounding context,
/// used to qualify bare identifiers as enum variants.
/// Capitalize the first character of a string (PascalCase heuristic for type names).
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn render_expr_to_rust(expr: &Expr, context_type: &str, opts: RenderOpts) -> String {
    match expr {
        Expr::Literal(lit) => render_literal(lit, opts),
        Expr::Ident(name) => {
            // Bare identifiers in data contexts are enum variants.
            // `context_type` should be the enum name (e.g. "SymbolId").
            format!("{context_type}::{name}")
        }
        Expr::Record(maybe_name, fields) => {
            let type_name = maybe_name.as_deref().unwrap_or(context_type);
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(name, val)| {
                    // FC-3: Use field name as context type only as a last resort.
                    // In most cases, the correct context type would come from type
                    // information. Since we don't have full type context here, we
                    // use the field name capitalized as a heuristic for enum type names
                    // (e.g., field "color" → context type "Color" for variant resolution).
                    // This is imperfect but better than raw field names as types.
                    let field_context = capitalize_first(name);
                    let field_val = render_expr_to_rust(val, &field_context, opts);
                    format!("{name}: {field_val}")
                })
                .collect();
            format!("{type_name} {{ {} }}", field_strs.join(", "))
        }
        Expr::List(elements) => {
            let items: Vec<String> = elements
                .iter()
                .map(|e| render_expr_to_rust(e, context_type, opts))
                .collect();
            if opts.static_context {
                format!("&[{}]", items.join(", "))
            } else {
                format!("vec![{}]", items.join(", "))
            }
        }
        Expr::StringInterp(parts) => {
            let mut s = String::new();
            for part in parts {
                match part {
                    daglang_syntax::ast::StringPart::Literal(lit) => s.push_str(lit),
                    daglang_syntax::ast::StringPart::Expr(_) => s.push_str("{}"),
                }
            }
            if opts.static_context {
                format!("\"{s}\"")
            } else {
                format!("\"{s}\".to_string()")
            }
        }
        Expr::Map(entries) => {
            let items: Vec<String> = entries
                .iter()
                .map(|(k, v)| {
                    let key_str = render_expr_to_rust(k, "String", opts);
                    let val_str = render_expr_to_rust(v, context_type, opts);
                    // In static context, string literal values are &str but HashMap<K, String>
                    // needs String values. Add .to_string() for String-typed values.
                    let val_expr = if context_type == "String" && opts.static_context {
                        format!("{val_str}.to_string()")
                    } else {
                        val_str
                    };
                    format!("({key_str}.to_string(), {val_expr})")
                })
                .collect();
            format!("HashMap::from([{}])", items.join(", "))
        }
        _ => format!(
            "compile_error!(\"unsupported expr in type_codegen: {}\")",
            format!("{:?}", expr)
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
        ),
    }
}

fn render_literal(lit: &Literal, opts: RenderOpts) -> String {
    match lit {
        Literal::Int(n) => n.to_string(),
        Literal::Float(f) => format!("{f:?}"),
        Literal::String(s) => {
            let escaped = escape_rust_string(s);
            if opts.static_context {
                format!("\"{escaped}\"")
            } else {
                format!("\"{escaped}\".to_string()")
            }
        }
        Literal::Bool(b) => b.to_string(),
        Literal::None => "None".to_string(),
    }
}

/// Escape a string for embedding in Rust source.
///
/// The DSL lexer preserves unrecognized escape sequences as literal chars
/// (e.g. `\x1b` → `\`, `x`, `1`, `b`). These are already valid Rust escape
/// syntax, so we pass `\x..` through unchanged.
fn escape_rust_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '\x1b' => {
                out.push_str("\\x1b");
                i += 1;
            }
            '\\' => {
                // Check if this is a \x.. hex escape preserved by the lexer.
                if i + 3 < chars.len() && chars[i + 1] == 'x' {
                    // Pass through \xHH sequences as-is (valid Rust).
                    out.push_str(&String::from_iter(&chars[i..i + 4]));
                    i += 4;
                } else {
                    out.push_str("\\\\");
                    i += 1;
                }
            }
            '"' => {
                out.push_str("\\\"");
                i += 1;
            }
            '\n' => {
                out.push_str("\\n");
                i += 1;
            }
            '\r' => {
                out.push_str("\\r");
                i += 1;
            }
            '\t' => {
                out.push_str("\\t");
                i += 1;
            }
            c if c.is_control() => {
                for byte in c.to_string().bytes() {
                    out.push_str(&format!("\\x{byte:02x}"));
                }
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

fn to_screaming_snake(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
    }
    result
}

// ---------------------------------------------------------------------------
// FnDef → Rust function signature
// ---------------------------------------------------------------------------

/// Convert a DSL `FnDef` to an abstract IR function definition.
///
/// If the function body contains statements, they are compiled to abstract IR
/// via `fn_codegen::compile_fn_body`. This produces target-agnostic IR that
/// flows through all backends (Rust, Go, C, MIPS).
///
/// `data_names` provides the set of `data` definition names visible in the
/// module so that identifier references can be mapped to SCREAMING_SNAKE_CASE.
pub fn fndef_to_code_ir(fd: &FnDef, ctx: &fn_codegen::CompileContext) -> Vec<code_ir::Item> {
    // S81: Collect parameter name → type name map for scrutinee type inference
    let param_types: std::collections::HashMap<String, String> = fd
        .params
        .iter()
        .map(|p| {
            let type_name = match &p.ty {
                TypeExpr::Named(n) => n.clone(),
                TypeExpr::Optional(inner) => match inner.as_ref() {
                    TypeExpr::Named(n) => n.clone(),
                    _ => type_expr_to_rust(&p.ty),
                },
                _ => type_expr_to_rust(&p.ty),
            };
            (p.name.clone(), type_name)
        })
        .collect();

    // Augment context with synthesized struct field types and optional params
    // so compile_expr can resolve anonymous records and prevent double-wrapping.
    // Extract the return type name for variant disambiguation
    let return_type_name = match &fd.return_type {
        TypeExpr::Named(n) => Some(n.clone()),
        TypeExpr::Optional(inner) => match inner.as_ref() {
            TypeExpr::Named(n) => Some(n.clone()),
            _ => None,
        },
        _ => None,
    };

    // Build ir_scope from function parameters
    let ir_scope: std::collections::HashMap<String, gunbc_ir::code_ir::IrType> = fd
        .params
        .iter()
        .map(|p| (p.name.clone(), fn_codegen::type_expr_to_ir_type(&p.ty)))
        .collect();

    // R3: Track which params are exactly String (now &str) — not Optional<String>
    let str_param_names: std::collections::HashSet<String> = fd
        .params
        .iter()
        .filter(|p| matches!(&p.ty, TypeExpr::Named(n) if n == "String"))
        .map(|p| p.name.clone())
        .collect();

    let mut analysis_ctx = ctx.clone();
    analysis_ctx.param_types = param_types.clone();
    analysis_ctx.str_param_names = str_param_names.clone();
    analysis_ctx.current_return_type = return_type_name.clone();
    analysis_ctx.current_return_ir_type = Some(fn_codegen::type_expr_to_ir_type(&fd.return_type));
    analysis_ctx.ir_scope = ir_scope.clone();

    let (synth_items, new_field_types, new_field_ir_types) =
        fn_codegen::materialize_synthesized_anonymous_record_types(
            &analysis_ctx.synthesized_anonymous_record_types,
        );

    // Collect optional parameters (T? → Option<T>)
    let optional_params: std::collections::HashSet<String> = fd
        .params
        .iter()
        .filter(|p| matches!(&p.ty, TypeExpr::Optional(_)))
        .map(|p| p.name.clone())
        .collect();

    let ctx = {
        let mut augmented = analysis_ctx;
        augmented.struct_field_types.extend(new_field_types);
        augmented.struct_field_ir_types.extend(new_field_ir_types);
        augmented.optional_params = optional_params;
        std::borrow::Cow::Owned(augmented)
    };

    let mut params: Vec<(String, String)> = fd
        .params
        .iter()
        .map(|p| {
            let ty = type_expr_to_rust(&p.ty);
            let ty = if ty == "String" {
                "&str".to_string()
            } else {
                ty
            };
            (p.name.clone(), ty)
        })
        .collect();
    let ret = type_expr_to_rust(&fd.return_type);
    let rename_todo_params = matches!(fd.name.as_str(), "box_top_line" | "box_bottom_line");

    let todo_body = vec![code_ir::Stmt::Expr(code_ir::Expr::MacroCall {
        name: "todo".to_string(),
        args: vec![code_ir::Expr::Str("generated from DSL".to_string())],
    })];
    let body = if fd.body.stmts.is_empty() {
        if rename_todo_params {
            for (name, _) in &mut params {
                if !name.starts_with('_') {
                    name.insert(0, '_');
                }
            }
        }
        todo_body
    } else {
        let compiled = fn_codegen::compile_fn_body(&fd.body, &ctx);
        if fn_codegen::body_has_empty_construct(&compiled) {
            if rename_todo_params {
                for (name, _) in &mut params {
                    if !name.starts_with('_') {
                        name.insert(0, '_');
                    }
                }
            }
            todo_body
        } else {
            compiled
        }
    };

    // Tail-call optimization: if all self-recursive calls are in tail position,
    // transform the function body to use a loop instead of recursion.
    let rust_fn_name = to_snake_case(&fd.name);
    let param_names: Vec<String> = fd.params.iter().map(|p| p.name.clone()).collect();
    let body = match fn_codegen::apply_tco(&rust_fn_name, &param_names, &body, &str_param_names) {
        Some(tco_body) => tco_body,
        None => body,
    };

    let mut items = synth_items;
    items.push(code_ir::Item::Fn(code_ir::FnDef {
        name: rust_fn_name,
        is_pub: true,
        params,
        return_type: Some(ret),
        body,
        doc: vec![],
        attributes: vec![],
    }));
    items
}

/// Build a map of struct_name → {optional field names} for `Some()` wrapping.
fn collect_optional_fields(
    td: &TypeDef,
    map: &mut std::collections::HashMap<String, std::collections::HashSet<String>>,
) {
    if let TypeBody::Record(fields) = &td.body {
        let opt: std::collections::HashSet<String> = fields
            .iter()
            .filter(|f| matches!(&f.ty, TypeExpr::Optional(_)))
            .map(|f| f.name.clone())
            .collect();
        if !opt.is_empty() {
            map.insert(td.name.clone(), opt);
        }
    }
}

fn collect_struct_field_ir_types(
    td: &TypeDef,
    map: &mut std::collections::HashMap<String, Vec<(String, gunbc_ir::code_ir::IrType)>>,
) {
    match &td.body {
        TypeBody::Record(fields) => {
            map.insert(
                td.name.clone(),
                fields
                    .iter()
                    .map(|field| {
                        (
                            field.name.clone(),
                            fn_codegen::type_expr_to_ir_type(&field.ty),
                        )
                    })
                    .collect(),
            );
        }
        TypeBody::Sum(variants) => {
            for variant in variants {
                if variant.fields.is_empty() {
                    continue;
                }
                map.insert(
                    variant.name.clone(),
                    variant
                        .fields
                        .iter()
                        .map(|field| {
                            (
                                field.name.clone(),
                                fn_codegen::type_expr_to_ir_type(&field.ty),
                            )
                        })
                        .collect(),
                );
            }
        }
        _ => {}
    }
}

/// Build a map of variant_name → enum_name for qualifying bare identifiers.
/// Ambiguous variants (same name in multiple enums) are excluded.
fn collect_variant_to_enum(
    td: &TypeDef,
    map: &mut std::collections::HashMap<String, String>,
    ambiguous: &mut std::collections::HashSet<String>,
) {
    if let TypeBody::Sum(variants) = &td.body {
        for v in variants {
            if ambiguous.contains(&v.name) {
                continue;
            }
            if let Some(existing) = map.get(&v.name) {
                if existing != &td.name {
                    ambiguous.insert(v.name.clone());
                    map.remove(&v.name);
                }
            } else {
                map.insert(v.name.clone(), td.name.clone());
            }
        }
    }
}

/// Build a map of enum_name → {variant names} for field-type disambiguation.
fn collect_enum_variants(
    td: &TypeDef,
    map: &mut std::collections::HashMap<String, std::collections::HashSet<String>>,
) {
    if let TypeBody::Sum(variants) = &td.body {
        let names: std::collections::HashSet<String> =
            variants.iter().map(|v| v.name.clone()).collect();
        map.insert(td.name.clone(), names);
    }
}

/// Build a map of struct_name → (field_name → type_name) for contextual variant resolution.
fn collect_struct_field_types(
    td: &TypeDef,
    map: &mut std::collections::HashMap<String, std::collections::HashMap<String, String>>,
) {
    if let TypeBody::Record(fields) = &td.body {
        let field_map: std::collections::HashMap<String, String> = fields
            .iter()
            .map(|f| (f.name.clone(), type_expr_to_rust_name(&f.ty)))
            .collect();
        if !field_map.is_empty() {
            map.insert(td.name.clone(), field_map);
        }
    }
}

pub fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}

// ---------------------------------------------------------------------------
// Impl generation from data tables
// ---------------------------------------------------------------------------

/// Generate a match-based `impl` method from a data table.
///
/// Given a data table like `ansi_mappings: List<AnsiMapping>` where
/// `AnsiMapping { color: SemanticColor, code: String }`, this generates:
///
/// ```text
/// impl SemanticColor {
///     pub fn ansi_code(&self) -> &'static str {
///         match self {
///             Self::Default => "\x1b[0m",
///             ...
///         }
///     }
/// }
/// ```
pub fn impl_from_data_table(
    data: &DataDef,
    key_field: &str,
    value_field: &str,
    method_name: &str,
    struct_defs: &[&TypeDef],
) -> Option<code_ir::Item> {
    let (elem_type_name, elements) = match (&data.ty, &data.value) {
        (TypeExpr::Generic(name, args), Expr::List(elems)) if name == "List" && args.len() == 1 => {
            let elem_name = match &args[0] {
                TypeExpr::Named(n) => n.as_str(),
                _ => return None,
            };
            (elem_name, elems)
        }
        _ => return None,
    };

    let field_types = resolve_field_types(elem_type_name, struct_defs);
    let key_type = field_types
        .iter()
        .find(|(n, _)| n == key_field)
        .map(|(_, t)| t.as_str())?;
    let value_type = field_types
        .iter()
        .find(|(n, _)| n == value_field)
        .map(|(_, t)| t.as_str())?;

    let rust_ret_type = if value_type == "String" {
        "&'static str"
    } else {
        value_type
    };

    let mut match_arms = Vec::new();
    for elem in elements {
        if let Expr::Record(_, fields) = elem {
            let key_val = fields.iter().find(|(n, _)| n == key_field).map(|(_, v)| v);
            let val_val = fields
                .iter()
                .find(|(n, _)| n == value_field)
                .map(|(_, v)| v);

            if let (Some(Expr::Ident(key)), Some(val)) = (key_val, val_val) {
                let rendered_val = render_expr_to_rust(val, value_type, STATIC_OPTS);
                match_arms.push(format!("            Self::{key} => {rendered_val},"));
            }
        }
    }

    if match_arms.is_empty() {
        return None;
    }

    let impl_block = format!(
        "impl {key_type} {{\n    pub fn {method_name}(&self) -> {rust_ret_type} {{\n        match self {{\n{}\n        }}\n    }}\n}}",
        match_arms.join("\n"),
    );

    Some(code_ir::Item::Raw(impl_block))
}

/// Collect all `TypeDef` items from a parsed DSL AST source file and
/// convert them to a Rust `SourceFile` ready for rendering.
pub fn typedefs_to_source_file(
    items: &[Spanned<daglang_syntax::ast::Item>],
    module_doc: &str,
) -> SourceFile {
    let mut data_names = std::collections::HashSet::new();
    let mut optional_fields = std::collections::HashMap::new();
    let mut variant_to_enum = std::collections::HashMap::new();
    let mut ambiguous = std::collections::HashSet::new();
    let mut struct_field_types = std::collections::HashMap::new();
    let mut struct_field_ir_types = std::collections::HashMap::new();
    let mut enum_variants = std::collections::HashMap::new();
    for item in items {
        match &item.node {
            daglang_syntax::ast::Item::DataDef(dd) => {
                data_names.insert(dd.name.clone());
            }
            daglang_syntax::ast::Item::TypeDef(td) => {
                collect_optional_fields(td, &mut optional_fields);
                collect_variant_to_enum(td, &mut variant_to_enum, &mut ambiguous);
                collect_struct_field_types(td, &mut struct_field_types);
                collect_struct_field_ir_types(td, &mut struct_field_ir_types);
                collect_enum_variants(td, &mut enum_variants);
            }
            _ => {}
        }
    }
    let ctx = fn_codegen::CompileContext {
        data_names,
        data_map_names: std::collections::HashSet::new(),
        optional_fields,
        variant_to_enum,
        struct_field_types,
        enum_variants,
        boxed_fields: std::collections::HashSet::new(),
        fn_return_types: std::collections::HashMap::new(),
        fn_param_types: std::collections::HashMap::new(),
        optional_params: std::collections::HashSet::new(),
        param_types: std::collections::HashMap::new(),
        current_return_type: None,
        current_return_ir_type: None,
        ir_scope: std::collections::HashMap::new(),
        struct_field_ir_types,
        use_counts: std::collections::HashMap::new(),
        fold_accum_name: None,
        enum_accessor_fields: HashMap::new(),
        data_ir_types: std::collections::HashMap::new(),
        fn_return_ir_types: std::collections::HashMap::new(),
        optional_return_fns: std::collections::HashSet::new(),
        fn_str_params: std::collections::HashSet::new(),
        str_param_names: std::collections::HashSet::new(),
        anonymous_record_targets: std::collections::HashMap::new(),
        synthesized_anonymous_record_types: Vec::new(),
        expr_ir_types: std::collections::HashMap::new(),
        expr_identities: std::collections::HashMap::new(),
        expr_path: std::cell::RefCell::new(Default::default()),
        rc_wrapped_types: std::collections::HashSet::new(),
        match_bound_vars: std::collections::HashSet::new(),
    };
    let mut code_items = Vec::new();
    for item in items {
        match &item.node {
            daglang_syntax::ast::Item::TypeDef(td) => {
                code_items.extend(typedef_to_code_ir(td));
            }
            daglang_syntax::ast::Item::DataDef(dd) => {
                code_items.extend(datadef_to_code_ir(dd));
            }
            daglang_syntax::ast::Item::FnDef(fd) => {
                code_items.extend(fndef_to_code_ir(fd, &ctx));
            }
            _ => {}
        }
    }
    SourceFile {
        doc: if module_doc.is_empty() {
            vec![]
        } else {
            vec![module_doc.to_string()]
        },
        items: code_items,
    }
}

/// Extract TypeDefs from a `TypedProject` and produce a rendered Rust source
/// string containing all generated types for the specified module paths.
pub fn generate_types_for_modules(
    typed: &daglang_typecheck::TypedProject<'_>,
    module_filter: &[&str],
) -> String {
    use crate::render_rust::render_rust_source;

    let (global_fn_return_types, global_fn_param_types) =
        collect_callable_type_maps_from_signatures(
            typed.modules().flat_map(|module| module.signatures.iter()),
        );

    // Pass 1: collect cloned AST items from matching modules.
    // TypedModuleRef temporaries from modules() don't live long enough
    // for cross-pass reference storage, so we clone the needed items.
    let mut type_defs: Vec<TypeDef> = Vec::new();
    let mut data_defs: Vec<DataDef> = Vec::new();
    let mut fn_defs: Vec<(usize, usize)> = Vec::new();
    let mut static_struct_types: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for module in typed.modules() {
        let module_name = module.module_path.as_dotted();
        if !module_filter.is_empty() && !module_filter.contains(&module_name.as_str()) {
            continue;
        }
        for (item_index, item) in module.ast.items.iter().enumerate() {
            match &item.node {
                daglang_syntax::ast::Item::TypeDef(td) => {
                    type_defs.push(td.clone());
                }
                daglang_syntax::ast::Item::DataDef(dd) => {
                    if let TypeExpr::Generic(name, args) = &dd.ty {
                        if name == "List" && args.len() == 1 {
                            if let TypeExpr::Named(elem) = &args[0] {
                                static_struct_types.insert(elem.clone());
                            }
                        }
                    }
                    if let TypeExpr::Named(name) = &dd.ty {
                        static_struct_types.insert(name.clone());
                    }
                    data_defs.push(dd.clone());
                }
                daglang_syntax::ast::Item::FnDef(_) => {
                    fn_defs.push((module.graph_index, item_index));
                }
                _ => {}
            }
        }
    }

    // Pass 2: generate all items, using type info for data tables.
    let type_def_refs: Vec<&TypeDef> = type_defs.iter().collect();
    let mut all_items = Vec::new();

    for td in &type_defs {
        if static_struct_types.contains(&td.name) {
            all_items.extend(typedef_to_static_code_ir(td));
        } else {
            all_items.extend(typedef_to_code_ir(td));
        }
    }

    for dd in &data_defs {
        all_items.extend(datadef_to_code_ir_with(dd, &type_def_refs));
    }

    for (module_index, item_index) in &fn_defs {
        let module = typed
            .module(*module_index)
            .expect("typed module should exist for collected fn");
        let item = module
            .ast
            .items
            .get(*item_index)
            .expect("collected fn item index should remain valid");
        let daglang_syntax::ast::Item::FnDef(fd) = &item.node else {
            panic!("collected fn item should still be a function");
        };
        let mut fn_data_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for dd in &data_defs {
            fn_data_names.insert(dd.name.clone());
        }
        let data_ir_types = collect_data_ir_types(data_defs.iter());
        let mut opt_fields = std::collections::HashMap::new();
        let mut v2e = std::collections::HashMap::new();
        let mut ambig = std::collections::HashSet::new();
        let mut sft = std::collections::HashMap::new();
        let mut sfit = std::collections::HashMap::new();
        let mut ev = std::collections::HashMap::new();
        for td in &type_defs {
            collect_optional_fields(td, &mut opt_fields);
            collect_variant_to_enum(td, &mut v2e, &mut ambig);
            collect_struct_field_types(td, &mut sft);
            collect_struct_field_ir_types(td, &mut sfit);
            collect_enum_variants(td, &mut ev);
        }
        let fn_ctx = fn_codegen::CompileContext {
            data_names: fn_data_names,
            data_ir_types,
            data_map_names: std::collections::HashSet::new(),
            optional_fields: opt_fields,
            variant_to_enum: v2e,
            struct_field_types: sft,
            enum_variants: ev,
            boxed_fields: std::collections::HashSet::new(),
            fn_return_types: global_fn_return_types.clone(),
            fn_return_ir_types: std::collections::HashMap::new(),
            fn_param_types: global_fn_param_types.clone(),
            optional_params: std::collections::HashSet::new(),
            param_types: std::collections::HashMap::new(),
            current_return_type: None,
            current_return_ir_type: None,
            ir_scope: std::collections::HashMap::new(),
            struct_field_ir_types: sfit,
            use_counts: std::collections::HashMap::new(),
            fold_accum_name: None,
            enum_accessor_fields: HashMap::new(),
            optional_return_fns: std::collections::HashSet::new(),
            fn_str_params: std::collections::HashSet::new(),
            str_param_names: std::collections::HashSet::new(),
            anonymous_record_targets: collect_anonymous_record_targets(
                module.callable_body_metadata(&fd.name),
            ),
            synthesized_anonymous_record_types: collect_synthesized_anonymous_record_types(
                module.callable_body_metadata(&fd.name),
            ),
            expr_ir_types: collect_expr_ir_types(module.callable_body_metadata(&fd.name)),
            expr_identities: std::collections::HashMap::new(),
            expr_path: std::cell::RefCell::new(Default::default()),
            rc_wrapped_types: std::collections::HashSet::new(),
            match_bound_vars: std::collections::HashSet::new(),
        };
        all_items.extend(fndef_to_code_ir(fd, &fn_ctx));
    }

    // Pass 3: generate impl blocks from data table lookup patterns.
    // Detect DSL functions that are "lookup field in table" and generate
    // match-based impl methods instead of standalone todo!() stubs.
    for dd in &data_defs {
        let field_types = resolve_field_types_for_data(dd, &type_def_refs);
        if field_types.len() < 2 {
            continue;
        }
        let key_field = &field_types[0].0;
        let key_type = &field_types[0].1;
        for (value_name, value_type) in field_types.iter().skip(1) {
            // Skip generating self-referential methods (e.g. fn id() -> SymbolId on SymbolId).
            if value_type == key_type {
                continue;
            }
            if let Some(item) = impl_from_data_table(
                dd,
                key_field,
                value_name,
                &to_snake_case(value_name),
                &type_def_refs,
            ) {
                all_items.push(item);
            }
        }
    }

    if all_items.is_empty() {
        return String::new();
    }

    // Emit built-in helper functions needed by generated code.
    let needs_char_funcs = all_items.iter().any(|item| {
        matches!(item, code_ir::Item::Fn(f) if f.body.iter().any(|s| {
            format!("{s:?}").contains("code_point_i64")
        }))
    });
    if needs_char_funcs {
        all_items.insert(
            0,
            code_ir::Item::Raw(
                "#[inline]\npub fn code_point_i64(c: char) -> i64 { c as u32 as i64 }".to_string(),
            ),
        );
    }

    // Replace resolve_symbol/symbol_color/ansi_code todo!() stubs with
    // proper implementations that delegate to generated impl methods.
    replace_builtin_stubs(&mut all_items);

    let source = SourceFile {
        doc: vec!["Generated from DSL type definitions. Do not edit.".to_string()],
        items: all_items,
    };

    render_rust_source(&source)
}

/// Replace `todo!()` stub functions with proper built-in implementations
/// that delegate to generated `impl` methods from data tables.
fn replace_builtin_stubs(items: &mut [code_ir::Item]) {
    for item in items.iter_mut() {
        if let code_ir::Item::Fn(f) = item {
            if is_todo_stub(&f.body) {
                if let Some(replacement) = builtin_body(&f.name) {
                    f.body = replacement;
                }
            }
        }
    }
}

fn is_todo_stub(body: &[code_ir::Stmt]) -> bool {
    matches!(body, [code_ir::Stmt::Expr(code_ir::Expr::MacroCall { name, .. })] if name == "todo")
}

fn builtin_body(name: &str) -> Option<Vec<code_ir::Stmt>> {
    match name {
        "resolve_symbol" => Some(vec![code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(
            "match tier {\n        \
             Tier::Emoji => id.emoji().to_string(),\n        \
             Tier::Unicode => id.unicode().to_string(),\n        \
             Tier::Ascii => id.ascii().to_string(),\n    \
             }"
            .to_string(),
        ))]),
        "symbol_color" => Some(vec![code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(
            "id.color()".to_string(),
        ))]),
        "ansi_code" => Some(vec![code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(
            "c.code().to_string()".to_string(),
        ))]),
        "truncate_text" => Some(vec![code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(
            "{\n        \
             let mut result = String::new();\n        \
             let mut used: i64 = 0;\n        \
             for c in text.chars() {\n            \
                 let w = char_width(c);\n            \
                 if used + w > max_width { break; }\n            \
                 result.push(c);\n            \
                 used += w;\n        \
             }\n        \
             result\n    \
             }"
            .to_string(),
        ))]),
        "truncate_spans" => Some(vec![code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(
            "{\n        \
             let mut kept = Vec::new();\n        \
             let mut remaining = budget;\n        \
             for span in spans {\n            \
                 if remaining <= 0 { break; }\n            \
                 let w = span_width(span.clone(), tier);\n            \
                 if w <= remaining {\n                \
                     kept.push(span);\n                \
                     remaining -= w;\n            \
                 } else {\n                \
                     let truncated = truncate_text(span.text.clone(), remaining);\n                \
                     kept.push(Span { text: truncated, style: span.style });\n                \
                     break;\n            \
                 }\n        \
             }\n        \
             kept\n    \
             }"
            .to_string(),
        ))]),
        "repeat_char" => Some(vec![code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(
            "c.repeat(n.max(0) as usize)".to_string(),
        ))]),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Recursive type detection → Box<> insertion (Phase 4)
// ---------------------------------------------------------------------------

/// Compute the set of (TypeName, FieldName) pairs where the field type
/// creates a recursive cycle and needs to be wrapped in `Box<>`.
///
/// Uses DFS cycle detection on a type reference graph built from the TypeDef
/// items. Only fields that directly reference a type in a cycle are boxed —
/// fields referencing types through `List<T>` are already heap-allocated
/// and don't need boxing.
pub fn compute_recursive_fields(
    type_defs: &[&daglang_syntax::ast::TypeDef],
) -> HashSet<(String, String)> {
    // Build adjacency: type_name → [(field_name, referenced_type_name)]
    let mut graph: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let all_type_names: HashSet<String> = type_defs.iter().map(|td| td.name.clone()).collect();

    for td in type_defs {
        let mut edges = Vec::new();
        match &td.body {
            TypeBody::Record(fields) => {
                for f in fields {
                    collect_direct_type_refs(&f.ty, &f.name, &all_type_names, &mut edges);
                }
            }
            TypeBody::Sum(variants) => {
                for v in variants {
                    for f in &v.fields {
                        let field_key = format!("{}::{}", v.name, f.name);
                        collect_direct_type_refs(&f.ty, &field_key, &all_type_names, &mut edges);
                    }
                }
            }
            TypeBody::Alias(_) => {}
        }
        graph.insert(td.name.clone(), edges);
    }

    // DFS cycle detection
    let mut recursive_fields = HashSet::new();
    let mut visited = HashSet::new();
    let mut on_stack = HashSet::new();

    let mut sorted_type_names: Vec<&String> = all_type_names.iter().collect();
    sorted_type_names.sort();
    for type_name in &sorted_type_names {
        if !visited.contains(*type_name as &str) {
            dfs_find_cycles(
                type_name,
                &graph,
                &mut visited,
                &mut on_stack,
                &mut Vec::new(),
                &mut recursive_fields,
            );
        }
    }

    // R2 size-based override removed: R8 Rc-wraps TransportBinding and
    // ServiceConfig, so these fields are already heap-allocated via Rc.
    // Box-wrapping on top of Rc is redundant and produces incorrect deref code.

    recursive_fields
}

/// Collect direct (non-List-wrapped, non-Rc-wrapped) type references from a TypeExpr.
/// Only direct references need boxing; List<T> and Rc-wrapped types are already heap-allocated.
fn collect_direct_type_refs(
    ty: &TypeExpr,
    field_name: &str,
    known_types: &HashSet<String>,
    edges: &mut Vec<(String, String)>,
) {
    match ty {
        TypeExpr::Named(name) => {
            // R8: Rc-wrapped types are heap-allocated, so they don't need Boxing
            if known_types.contains(name) && !is_rc_wrapped(name) {
                edges.push((field_name.to_string(), name.clone()));
            }
        }
        TypeExpr::AssociatedOutput(_) => {}
        TypeExpr::Function(_, _) => {}
        TypeExpr::Optional(inner) => {
            collect_direct_type_refs(inner, field_name, known_types, edges);
        }
        TypeExpr::Refined(inner, _) => {
            collect_direct_type_refs(inner, field_name, known_types, edges);
        }
        // Generic containers like List<T>, Map<K,V> are heap-allocated;
        // their contents don't need Boxing.
        TypeExpr::Generic(_, _) => {}
        TypeExpr::Record(_) => {}
    }
}

/// DFS helper for cycle detection. When a back-edge is found, all edges
/// on the cycle path are marked as needing Box<>.
fn dfs_find_cycles(
    node: &str,
    graph: &HashMap<String, Vec<(String, String)>>,
    visited: &mut HashSet<String>,
    on_stack: &mut HashSet<String>,
    path: &mut Vec<(String, String, String)>, // (from_type, field_name, to_type)
    recursive_fields: &mut HashSet<(String, String)>,
) {
    visited.insert(node.to_string());
    on_stack.insert(node.to_string());

    if let Some(edges) = graph.get(node) {
        for (field_name, target) in edges {
            if !visited.contains(target.as_str()) {
                path.push((node.to_string(), field_name.clone(), target.clone()));
                dfs_find_cycles(target, graph, visited, on_stack, path, recursive_fields);
                path.pop();
            } else if on_stack.contains(target.as_str()) {
                // Found a cycle — only mark edges that are WITHIN the cycle,
                // not edges that merely led to the cycle. The cycle starts
                // at `target` and includes all path edges from `target` onward.
                let cycle_start = target.as_str();
                let mut in_cycle = false;
                for (from, fname, to) in path.iter() {
                    if from == cycle_start {
                        in_cycle = true;
                    }
                    if in_cycle {
                        recursive_fields.insert((from.clone(), fname.clone()));
                    }
                    if in_cycle && to == cycle_start {
                        break;
                    }
                }
                // The current edge (node → target) closes the cycle
                if node == cycle_start || in_cycle {
                    recursive_fields.insert((node.to_string(), field_name.clone()));
                }
            }
        }
    }

    on_stack.remove(node);
}

/// Like `typedef_to_code_ir` but wraps recursive fields in `Box<>`.
pub fn typedef_to_code_ir_boxed(
    td: &daglang_syntax::ast::TypeDef,
    recursive_fields: &HashSet<(String, String)>,
) -> Vec<code_ir::Item> {
    let derives: Vec<String> = DEFAULT_DERIVES.iter().map(|s| s.to_string()).collect();

    match &td.body {
        TypeBody::Record(fields) => {
            let struct_fields: Vec<(String, String, bool)> = fields
                .iter()
                .map(|f| {
                    let rust_ty = type_expr_to_rust(&f.ty);
                    let needs_box = recursive_fields.contains(&(td.name.clone(), f.name.clone()));
                    let final_ty = if needs_box {
                        format!("Box<{}>", rust_ty)
                    } else {
                        rust_ty
                    };
                    (f.name.clone(), final_ty, true)
                })
                .collect();
            // All v2 structs get Default — this enables ..Default::default()
            // struct update syntax for missing fields. Since all field types
            // are either primitives, String, Vec, Option, Box, or other
            // generated structs (which also derive Default), this is sound.
            let mut derives = derives;
            derives.push("Default".to_string());
            vec![code_ir::Item::Struct(StructDef {
                name: td.name.clone(),
                is_pub: true,
                derives,
                fields: struct_fields,
                doc: vec![],
            })]
        }
        TypeBody::Sum(variants) => {
            let mut derives = derives;
            if is_simple_enum(variants) {
                derives.push("Copy".to_string());
                derives.push("Hash".to_string());
            }
            // Add a `#[default]` attribute on the first unit variant so enums
            // can derive Default — needed for struct fields that contain
            // enum types when the parent struct derives Default.
            // Rust only allows `#[default]` on unit variants (no fields).
            let default_variant_idx = variants.iter().position(|v| v.fields.is_empty());
            if default_variant_idx.is_some() {
                derives.push("Default".to_string());
            }

            let variant_strs: Vec<String> = variants
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let base = format_variant_boxed(v, &td.name, recursive_fields);
                    if Some(i) == default_variant_idx {
                        format!("#[default]\n    {base}")
                    } else {
                        base
                    }
                })
                .collect();

            let mut items = vec![code_ir::Item::Enum(EnumDef {
                name: td.name.clone(),
                is_pub: true,
                derives,
                variants: variant_strs,
                doc: vec![],
            })];

            // For enums without any unit variant, generate a manual impl Default
            // that constructs the first variant with all fields defaulted.
            if default_variant_idx.is_none() {
                let first = &variants[0];
                let default_fields: Vec<String> = first
                    .fields
                    .iter()
                    .map(|f| format!("{}: Default::default()", f.name))
                    .collect();
                items.push(code_ir::Item::Raw(format!(
                    "impl Default for {} {{\n    fn default() -> Self {{\n        {}::{} {{ {} }}\n    }}\n}}",
                    td.name,
                    td.name,
                    first.name,
                    default_fields.join(", ")
                )));
            }

            // Generate accessor methods for fields common to all variants
            if let Some(impl_block) = enum_accessor_impl(&td.name, variants) {
                items.push(code_ir::Item::Raw(impl_block));
            }

            items
        }
        TypeBody::Alias(type_expr) => {
            let rust_type = type_expr_to_rust(type_expr);
            vec![code_ir::Item::Raw(format!(
                "pub type {} = {};",
                td.name, rust_type
            ))]
        }
    }
}

/// Format a variant for enum definition, wrapping recursive fields in Box<>.
fn format_variant_boxed(
    v: &daglang_syntax::ast::Variant,
    type_name: &str,
    recursive_fields: &HashSet<(String, String)>,
) -> String {
    if v.fields.is_empty() {
        v.name.clone()
    } else {
        let field_strs: Vec<String> = v
            .fields
            .iter()
            .map(|f| {
                let rust_ty = type_expr_to_rust(&f.ty);
                let field_key = format!("{}::{}", v.name, f.name);
                let needs_box = recursive_fields.contains(&(type_name.to_string(), field_key));
                let final_ty = if needs_box {
                    format!("Box<{}>", rust_ty)
                } else {
                    rust_ty
                };
                format!("{}: {}", f.name, final_ty)
            })
            .collect();
        format!("{} {{ {} }}", v.name, field_strs.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_resolve::{ModuleGraph, ResolvedModule};
    use daglang_syntax::ast::{
        DataDef, Expr, Field, FnBody, FnDef, Item, Literal, Param, Refinement, Stmt, TypeBody,
        TypeDef, TypeExpr, Variant,
    };
    use daglang_typecheck::{TypedBinding, TypedCallableSignature, TypedItemSignature};
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    fn module_graph_from_sources(sources: &[(&str, &str)]) -> ModuleGraph {
        let modules = sources
            .iter()
            .map(|(path, source)| {
                let ast = daglang_syntax::parser::parse(source).expect("source should parse");
                let module_path = ast
                    .module_path
                    .as_ref()
                    .map(|module| module.node.clone())
                    .expect("module declarations are required in tests");
                ResolvedModule {
                    path: PathBuf::from(path),
                    ast,
                    module_path,
                    dependencies: Vec::new(),
                    source: source.to_string(),
                }
            })
            .collect::<Vec<_>>();
        let module_lookup = modules
            .iter()
            .enumerate()
            .map(|(index, module)| (module.module_path.as_dotted(), index))
            .collect::<HashMap<_, _>>();
        let mut modules = modules;
        for module in &mut modules {
            module.dependencies = module
                .ast
                .imports
                .iter()
                .filter_map(|import| module_lookup.get(&import.node.path.as_dotted()).copied())
                .collect::<Vec<_>>();
        }
        ModuleGraph { modules }
    }

    fn annotate_expr_ir_type(
        ctx: &mut fn_codegen::CompileContext,
        body: &FnBody,
        target: &Expr,
        ir_type: gunbc_ir::code_ir::IrType,
    ) {
        let mut expr_identity = None;
        daglang_syntax::ast_utils::walk_stmts_with_expr_identities(
            &body.stmts,
            &mut |identity, candidate| {
                if std::ptr::eq(candidate, target) {
                    expr_identity = Some(identity);
                }
            },
        );
        ctx.expr_ir_types.insert(
            expr_identity.expect("expected walked expression identity"),
            ir_type,
        );
    }

    fn compile_context_from_typechecked_metadata(
        source: &str,
        fn_name: &str,
    ) -> (FnDef, fn_codegen::CompileContext) {
        let graph = module_graph_from_sources(&[("sample/test.dag", source)]);
        let typed = daglang_typecheck::typecheck_module_graph_with_options(
            &graph,
            daglang_typecheck::TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("source should typecheck");
        let module = typed.module(0).expect("typed module should exist");
        let reparsed = daglang_syntax::parser::parse(source).expect("source should reparse");
        let fd = reparsed
            .items
            .iter()
            .find_map(|item| match &item.node {
                Item::FnDef(def) if def.name == fn_name => Some(def.clone()),
                _ => None,
            })
            .expect("expected function to exist in reparsed AST");
        let type_defs = reparsed
            .items
            .iter()
            .filter_map(|item| match &item.node {
                Item::TypeDef(td) => Some(td),
                _ => None,
            })
            .collect::<Vec<_>>();
        let data_defs = reparsed
            .items
            .iter()
            .filter_map(|item| match &item.node {
                Item::DataDef(dd) => Some(dd),
                _ => None,
            })
            .collect::<Vec<_>>();
        let (fn_return_types, fn_param_types) =
            collect_callable_type_maps_from_signatures(module.signatures.iter());
        let mut optional_fields = HashMap::new();
        let mut variant_to_enum = HashMap::new();
        let mut ambiguous_variants = HashSet::new();
        let mut struct_field_types = HashMap::new();
        let mut struct_field_ir_types = HashMap::new();
        let mut enum_variants = HashMap::new();
        for td in &type_defs {
            collect_optional_fields(td, &mut optional_fields);
            collect_variant_to_enum(td, &mut variant_to_enum, &mut ambiguous_variants);
            collect_struct_field_types(td, &mut struct_field_types);
            collect_struct_field_ir_types(td, &mut struct_field_ir_types);
            collect_enum_variants(td, &mut enum_variants);
        }

        (
            fd,
            fn_codegen::CompileContext {
                data_names: data_defs.iter().map(|dd| dd.name.clone()).collect(),
                data_ir_types: collect_data_ir_types(data_defs.iter().copied()),
                data_map_names: HashSet::new(),
                optional_fields,
                variant_to_enum,
                struct_field_types,
                enum_variants,
                boxed_fields: HashSet::new(),
                fn_return_types,
                fn_return_ir_types: HashMap::new(),
                fn_param_types,
                optional_params: HashSet::new(),
                param_types: HashMap::new(),
                current_return_type: None,
                current_return_ir_type: None,
                ir_scope: HashMap::new(),
                struct_field_ir_types,
                use_counts: HashMap::new(),
                fold_accum_name: None,
                anonymous_record_targets: collect_anonymous_record_targets(
                    module.callable_body_metadata(fn_name),
                ),
                synthesized_anonymous_record_types: collect_synthesized_anonymous_record_types(
                    module.callable_body_metadata(fn_name),
                ),
                expr_ir_types: collect_expr_ir_types(module.callable_body_metadata(fn_name)),
                expr_identities: HashMap::new(),
                expr_path: std::cell::RefCell::new(fn_codegen::ExprPath::default()),
                enum_accessor_fields: HashMap::new(),
                optional_return_fns: HashSet::new(),
                fn_str_params: HashSet::new(),
                str_param_names: HashSet::new(),
                rc_wrapped_types: HashSet::new(),
                match_bound_vars: HashSet::new(),
            },
        )
    }

    #[test]
    fn simple_enum_generates_copy_hash() {
        let td = TypeDef {
            name: "SemanticColor".to_string(),
            params: vec![],
            body: TypeBody::Sum(vec![
                Variant {
                    name: "Default".into(),
                    fields: vec![],
                },
                Variant {
                    name: "Success".into(),
                    fields: vec![],
                },
                Variant {
                    name: "Error".into(),
                    fields: vec![],
                },
            ]),
        };
        let items = typedef_to_code_ir(&td);
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Enum(e) => {
                assert_eq!(e.name, "SemanticColor");
                assert!(e.derives.contains(&"Copy".to_string()));
                assert!(e.derives.contains(&"Hash".to_string()));
                assert_eq!(e.variants, vec!["Default", "Success", "Error"]);
            }
            _ => panic!("expected Enum"),
        }
    }

    #[test]
    fn record_generates_struct_with_pub_fields() {
        let td = TypeDef {
            name: "SpanStyle".to_string(),
            params: vec![],
            body: TypeBody::Record(vec![
                Field {
                    name: "color".into(),
                    ty: TypeExpr::Optional(Box::new(TypeExpr::Named("SemanticColor".into()))),
                    default: None,
                    from_path: None,
                },
                Field {
                    name: "bold".into(),
                    ty: TypeExpr::Named("Bool".into()),
                    default: None,
                    from_path: None,
                },
            ]),
        };
        let items = typedef_to_code_ir(&td);
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Struct(s) => {
                assert_eq!(s.name, "SpanStyle");
                assert!(s.is_pub);
                assert_eq!(s.fields.len(), 2);
                assert_eq!(
                    s.fields[0],
                    ("color".into(), "Option<SemanticColor>".into(), true)
                );
                assert_eq!(s.fields[1], ("bold".into(), "bool".into(), true));
            }
            _ => panic!("expected Struct"),
        }
    }

    #[test]
    fn generic_list_maps_to_rc_vec() {
        let td = TypeDef {
            name: "Line".to_string(),
            params: vec![],
            body: TypeBody::Record(vec![
                Field {
                    name: "spans".into(),
                    ty: TypeExpr::Generic("List".into(), vec![TypeExpr::Named("Span".into())]),
                    default: None,
                    from_path: None,
                },
                Field {
                    name: "indent".into(),
                    ty: TypeExpr::Named("Int".into()),
                    default: None,
                    from_path: None,
                },
            ]),
        };
        let items = typedef_to_code_ir(&td);
        match &items[0] {
            code_ir::Item::Struct(s) => {
                assert_eq!(s.fields[0].1, "Rc<Vec<Span>>");
                assert_eq!(s.fields[1].1, "i64");
            }
            _ => panic!("expected Struct"),
        }
    }

    #[test]
    fn alias_generates_type_alias() {
        let td = TypeDef {
            name: "Width".to_string(),
            params: vec![],
            body: TypeBody::Alias(TypeExpr::Named("Int".into())),
        };
        let items = typedef_to_code_ir(&td);
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Raw(s) => {
                assert_eq!(s, "pub type Width = i64;");
            }
            _ => panic!("expected Raw"),
        }
    }

    #[test]
    fn variant_with_fields_generates_struct_variant() {
        let td = TypeDef {
            name: "Shape".to_string(),
            params: vec![],
            body: TypeBody::Sum(vec![
                Variant {
                    name: "Circle".into(),
                    fields: vec![Field {
                        name: "radius".into(),
                        ty: TypeExpr::Named("Float".into()),
                        default: None,
                        from_path: None,
                    }],
                },
                Variant {
                    name: "Point".into(),
                    fields: vec![],
                },
            ]),
        };
        let items = typedef_to_code_ir(&td);
        match &items[0] {
            code_ir::Item::Enum(e) => {
                assert_eq!(e.variants[0], "Circle { radius: f64 }");
                assert_eq!(e.variants[1], "Point");
                assert!(!e.derives.contains(&"Copy".to_string()));
            }
            _ => panic!("expected Enum"),
        }
    }

    #[test]
    fn data_list_generates_static_array() {
        let entry_td = TypeDef {
            name: "Entry".to_string(),
            params: vec![],
            body: TypeBody::Record(vec![
                Field {
                    name: "id".into(),
                    ty: TypeExpr::Named("EntryKind".into()),
                    default: None,
                    from_path: None,
                },
                Field {
                    name: "label".into(),
                    ty: TypeExpr::Named("String".into()),
                    default: None,
                    from_path: None,
                },
            ]),
        };
        let dd = DataDef {
            name: "testData".to_string(),
            ty: TypeExpr::Generic("List".into(), vec![TypeExpr::Named("Entry".into())]),
            value: Expr::List(vec![Expr::Record(
                None,
                vec![
                    ("id".into(), Expr::Ident("Alpha".into())),
                    (
                        "label".into(),
                        Expr::Literal(Literal::String("first".into())),
                    ),
                ],
            )]),
        };
        let items = datadef_to_code_ir_with(&dd, &[&entry_td]);
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Raw(s) => {
                assert!(s.contains("pub static TEST_DATA: &[Entry]"), "got: {s}");
                assert!(
                    s.contains("id: EntryKind::Alpha"),
                    "should resolve field type: {s}"
                );
                assert!(
                    s.contains(r#"label: "first""#),
                    "static context uses &str: {s}"
                );
                assert!(
                    !s.contains("to_string"),
                    "no to_string in static context: {s}"
                );
            }
            _ => panic!("expected Raw"),
        }
    }

    #[test]
    fn data_scalar_generates_static() {
        let dd = DataDef {
            name: "boxWidth".to_string(),
            ty: TypeExpr::Named("Int".into()),
            value: Expr::Literal(Literal::Int(60)),
        };
        let items = datadef_to_code_ir(&dd);
        match &items[0] {
            code_ir::Item::Raw(s) => {
                assert_eq!(s, "pub static BOX_WIDTH: i64 = 60;");
            }
            _ => panic!("expected Raw"),
        }
    }

    #[test]
    fn to_screaming_snake_converts_camel_case() {
        assert_eq!(to_screaming_snake("standardSymbols"), "STANDARD_SYMBOLS");
        assert_eq!(to_screaming_snake("boxWidth"), "BOX_WIDTH");
        assert_eq!(to_screaming_snake("ansiMappings"), "ANSI_MAPPINGS");
    }

    #[test]
    fn fndef_generates_function_definition() {
        let fd = FnDef {
            name: "resolve_symbol".to_string(),
            type_params: vec![],
            params: vec![
                Param {
                    name: "id".into(),
                    ty: TypeExpr::Named("SymbolId".into()),
                    default: None,
                },
                Param {
                    name: "tier".into(),
                    ty: TypeExpr::Named("Tier".into()),
                    default: None,
                },
            ],
            return_type: TypeExpr::Named("String".into()),
            body: FnBody { stmts: vec![] },
        };
        let ctx = fn_codegen::CompileContext::new();
        let items = fndef_to_code_ir(&fd, &ctx);
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Fn(f) => {
                assert_eq!(f.name, "resolve_symbol");
                assert!(f.is_pub);
                assert_eq!(
                    f.params,
                    vec![
                        ("id".to_string(), "SymbolId".to_string()),
                        ("tier".to_string(), "Tier".to_string()),
                    ]
                );
                assert_eq!(f.return_type.as_deref(), Some("String"));
            }
            other => panic!("expected Fn, got: {other:?}"),
        }
    }

    #[test]
    fn fndef_with_body_generates_compiled_ir() {
        let fd = FnDef {
            name: "add_one".to_string(),
            type_params: vec![],
            params: vec![Param {
                name: "x".into(),
                ty: TypeExpr::Named("Int".into()),
                default: None,
            }],
            return_type: TypeExpr::Named("Int".into()),
            body: FnBody {
                stmts: vec![daglang_syntax::ast::Stmt::Expr(Expr::BinOp(
                    Box::new(Expr::Ident("x".into())),
                    daglang_syntax::ast::BinOp::Add,
                    Box::new(Expr::Literal(Literal::Int(1))),
                ))],
            },
        };
        let ctx = fn_codegen::CompileContext::new();
        let items = fndef_to_code_ir(&fd, &ctx);
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Fn(f) => {
                assert_eq!(f.name, "add_one");
                assert_eq!(f.body.len(), 1);
                assert!(matches!(
                    f.body[0],
                    code_ir::Stmt::TailExpr(code_ir::Expr::BinOp { .. })
                ));
            }
            other => panic!("expected Fn, got: {other:?}"),
        }
    }

    #[test]
    fn typed_callable_signatures_drive_anonymous_record_call_args() {
        let signatures = [TypedItemSignature::Fn(TypedCallableSignature {
            name: "consume".to_string(),
            params: vec![TypedBinding {
                name: "cfg".to_string(),
                ty: gunbc_ir::types::TypeId::from("ConfigB"),
            }],
            outputs: vec![TypedBinding {
                name: "return".to_string(),
                ty: gunbc_ir::types::TypeId::from("String"),
            }],
        })];
        let (fn_return_types, fn_param_types) =
            collect_callable_type_maps_from_signatures(signatures.iter());
        let fd = FnDef {
            name: "make".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: TypeExpr::Named("String".to_string()),
            body: FnBody {
                stmts: vec![Stmt::Expr(Expr::Call(
                    "consume".to_string(),
                    vec![(
                        None,
                        Expr::Record(
                            None,
                            vec![(
                                "value".to_string(),
                                Expr::Literal(Literal::String("ok".to_string())),
                            )],
                        ),
                    )],
                ))],
            },
        };
        let mut ctx = fn_codegen::CompileContext::new();
        ctx.struct_field_types.insert(
            "ConfigA".to_string(),
            [("value".to_string(), "String".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.struct_field_types.insert(
            "ConfigB".to_string(),
            [("value".to_string(), "String".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.fn_return_types = fn_return_types;
        ctx.fn_param_types = fn_param_types;
        if let Stmt::Expr(Expr::Call(_, args)) = &fd.body.stmts[0] {
            annotate_expr_ir_type(
                &mut ctx,
                &fd.body,
                &args[0].1,
                gunbc_ir::code_ir::IrType::Named("ConfigB".to_string()),
            );
        }

        let items = fndef_to_code_ir(&fd, &ctx);
        let function = items
            .iter()
            .find_map(|item| match item {
                code_ir::Item::Fn(f) => Some(f),
                _ => None,
            })
            .expect("expected Fn item");
        match &function.body[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::Call { args, .. }) => match &args[0] {
                code_ir::Expr::Struct { name, .. } => assert_eq!(name, "ConfigB"),
                other => panic!("expected ConfigB struct arg, got: {other:?}"),
            },
            other => panic!("expected tail call body, got: {other:?}"),
        }
    }

    #[test]
    fn typed_callable_signatures_drive_with_base_record_updates() {
        let signatures = [TypedItemSignature::Fn(TypedCallableSignature {
            name: "make_state".to_string(),
            params: vec![],
            outputs: vec![TypedBinding {
                name: "return".to_string(),
                ty: gunbc_ir::types::TypeId::from("State"),
            }],
        })];
        let (fn_return_types, fn_param_types) =
            collect_callable_type_maps_from_signatures(signatures.iter());
        let fd = FnDef {
            name: "bump".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: TypeExpr::Named("State".to_string()),
            body: FnBody {
                stmts: vec![Stmt::Expr(Expr::Call(
                    "with".to_string(),
                    vec![
                        (None, Expr::Call("make_state".to_string(), vec![])),
                        (
                            None,
                            Expr::Record(
                                None,
                                vec![("pos".to_string(), Expr::Literal(Literal::Int(1)))],
                            ),
                        ),
                    ],
                ))],
            },
        };
        let mut ctx = fn_codegen::CompileContext::new();
        ctx.struct_field_types.insert(
            "State".to_string(),
            [
                ("pos".to_string(), "Int".to_string()),
                ("kind".to_string(), "String".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        ctx.struct_field_types.insert(
            "Position".to_string(),
            [("pos".to_string(), "Int".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.struct_field_ir_types.insert(
            "State".to_string(),
            vec![
                ("pos".to_string(), gunbc_ir::code_ir::IrType::Int),
                ("kind".to_string(), gunbc_ir::code_ir::IrType::Str),
            ],
        );
        ctx.struct_field_ir_types.insert(
            "Position".to_string(),
            vec![("pos".to_string(), gunbc_ir::code_ir::IrType::Int)],
        );
        ctx.fn_return_types = fn_return_types;
        ctx.fn_param_types = fn_param_types;
        if let Stmt::Expr(Expr::Call(_, args)) = &fd.body.stmts[0] {
            annotate_expr_ir_type(
                &mut ctx,
                &fd.body,
                &args[0].1,
                gunbc_ir::code_ir::IrType::Named("State".to_string()),
            );
            let mut expr_identity = None;
            daglang_syntax::ast_utils::walk_stmts_with_expr_identities(
                &fd.body.stmts,
                &mut |identity, candidate| {
                    if std::ptr::eq(candidate, &args[1].1) {
                        expr_identity = Some(identity);
                    }
                },
            );
            let expr_identity = expr_identity.expect("expected walked expression identity");
            ctx.anonymous_record_targets
                .insert(expr_identity, "State".to_string());
            ctx.expr_ir_types.insert(
                expr_identity,
                gunbc_ir::code_ir::IrType::Named("State".to_string()),
            );
        }

        let items = fndef_to_code_ir(&fd, &ctx);
        let function = items
            .iter()
            .find_map(|item| match item {
                code_ir::Item::Fn(f) => Some(f),
                _ => None,
            })
            .expect("expected Fn item");
        match &function.body[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::Struct { name, rest, .. }) => {
                assert_eq!(name, "State");
                assert!(rest.is_some(), "expected struct update rest");
            }
            other => panic!("expected struct update tail expr, got: {other:?}"),
        }
    }

    #[test]
    fn explicit_anonymous_record_targets_drive_let_bound_codegen() {
        let signatures = [TypedItemSignature::Fn(TypedCallableSignature {
            name: "consume".to_string(),
            params: vec![TypedBinding {
                name: "cfg".to_string(),
                ty: gunbc_ir::types::TypeId::from("ConfigB"),
            }],
            outputs: vec![TypedBinding {
                name: "return".to_string(),
                ty: gunbc_ir::types::TypeId::from("String"),
            }],
        })];
        let (fn_return_types, fn_param_types) =
            collect_callable_type_maps_from_signatures(signatures.iter());
        let fd = FnDef {
            name: "make".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: TypeExpr::Named("String".to_string()),
            body: FnBody {
                stmts: vec![
                    Stmt::Let(
                        "cfg".to_string(),
                        Expr::Record(
                            None,
                            vec![(
                                "value".to_string(),
                                Expr::Literal(Literal::String("ok".to_string())),
                            )],
                        ),
                    ),
                    Stmt::Expr(Expr::Call(
                        "consume".to_string(),
                        vec![(None, Expr::Ident("cfg".to_string()))],
                    )),
                ],
            },
        };
        let mut ctx = fn_codegen::CompileContext::new();
        ctx.struct_field_types.insert(
            "ConfigA".to_string(),
            [("value".to_string(), "String".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.struct_field_types.insert(
            "ConfigB".to_string(),
            [("value".to_string(), "String".to_string())]
                .into_iter()
                .collect(),
        );
        if let Stmt::Let(_, expr) = &fd.body.stmts[0] {
            let mut expr_identity = None;
            daglang_syntax::ast_utils::walk_stmts_with_expr_identities(
                &fd.body.stmts,
                &mut |identity, candidate| {
                    if std::ptr::eq(candidate, expr) {
                        expr_identity = Some(identity);
                    }
                },
            );
            ctx.anonymous_record_targets.insert(
                expr_identity.expect("expected walked expression identity"),
                "ConfigB".to_string(),
            );
            annotate_expr_ir_type(
                &mut ctx,
                &fd.body,
                expr,
                gunbc_ir::code_ir::IrType::Named("ConfigB".to_string()),
            );
        }
        ctx.fn_return_types = fn_return_types;
        ctx.fn_param_types = fn_param_types;

        let items = fndef_to_code_ir(&fd, &ctx);
        match &items[0] {
            code_ir::Item::Fn(f) => match &f.body[0] {
                code_ir::Stmt::Let {
                    ir_type: Some(gunbc_ir::code_ir::IrType::Named(ir_type)),
                    expr: code_ir::Expr::Struct { name, .. },
                    ..
                } => {
                    assert_eq!(name, "ConfigB");
                    assert_eq!(ir_type, "ConfigB");
                }
                other => panic!("expected let-bound ConfigB struct, got: {other:?}"),
            },
            other => panic!("expected Fn, got: {other:?}"),
        }
    }

    #[test]
    fn typechecked_anonymous_record_targets_survive_reparse_into_emit() {
        let source = r#"module sample.records
type ConfigB {
  value: String
}
fn consume(cfg: ConfigB) -> String {
  cfg.value
}
fn make() -> String {
  let cfg = { value: "ok" }
  consume(cfg)
}"#;
        let (fd, ctx) = compile_context_from_typechecked_metadata(source, "make");

        let items = fndef_to_code_ir(&fd, &ctx);
        let function = items
            .iter()
            .find_map(|item| match item {
                code_ir::Item::Fn(f) => Some(f),
                _ => None,
            })
            .expect("expected Fn item");
        match &function.body[0] {
            code_ir::Stmt::Let {
                expr: code_ir::Expr::Struct { name, .. },
                ..
            } => assert_eq!(name, "ConfigB"),
            other => panic!("expected let-bound ConfigB struct, got: {other:?}"),
        }
    }

    #[test]
    fn typechecked_fold_accumulator_types_survive_reparse_into_emit() {
        let source = r#"module sample.records
type Span {
  text: String
}
fn collect_text(spans: List<Span>) -> List<String> {
  let state = fold(spans,
    init: { texts: [] },
    f: (acc, span) => { texts: concat(acc.texts, [span.text]) }
  )
  state.texts
}"#;
        let (fd, ctx) = compile_context_from_typechecked_metadata(source, "collect_text");
        assert!(
            ctx.synthesized_anonymous_record_types
                .iter()
                .any(|ty| ty.name.starts_with("__CollecttextState")),
            "expected typecheck metadata to carry the synthesized accumulator type",
        );

        let items = fndef_to_code_ir(&fd, &ctx);
        let synthesized_name = items
            .iter()
            .find_map(|item| match item {
                code_ir::Item::Struct(code_ir::StructDef { name, .. })
                    if name.starts_with("__CollecttextState") =>
                {
                    Some(name.clone())
                }
                _ => None,
            })
            .expect("expected synthesized accumulator type to be emitted from typecheck metadata");
        let function = items
            .iter()
            .find_map(|item| match item {
                code_ir::Item::Fn(f) => Some(f),
                _ => None,
            })
            .expect("expected Fn item");
        match &function.body[0] {
            code_ir::Stmt::Let {
                ir_type: Some(gunbc_ir::code_ir::IrType::Named(ir_type)),
                ..
            } => assert_eq!(ir_type, &synthesized_name),
            other => panic!("expected synthesized accumulator let type, got: {other:?}"),
        }
    }

    #[test]
    fn to_snake_case_preserves_snake() {
        assert_eq!(to_snake_case("resolve_symbol"), "resolve_symbol");
        assert_eq!(to_snake_case("charWidth"), "char_width");
    }

    #[test]
    fn impl_from_data_table_generates_match() {
        let color_td = TypeDef {
            name: "Color".to_string(),
            params: vec![],
            body: TypeBody::Sum(vec![
                Variant {
                    name: "Red".into(),
                    fields: vec![],
                },
                Variant {
                    name: "Blue".into(),
                    fields: vec![],
                },
            ]),
        };
        let mapping_td = TypeDef {
            name: "ColorMapping".to_string(),
            params: vec![],
            body: TypeBody::Record(vec![
                Field {
                    name: "color".into(),
                    ty: TypeExpr::Named("Color".into()),
                    default: None,
                    from_path: None,
                },
                Field {
                    name: "code".into(),
                    ty: TypeExpr::Named("String".into()),
                    default: None,
                    from_path: None,
                },
            ]),
        };
        let dd = DataDef {
            name: "mappings".to_string(),
            ty: TypeExpr::Generic("List".into(), vec![TypeExpr::Named("ColorMapping".into())]),
            value: Expr::List(vec![
                Expr::Record(
                    None,
                    vec![
                        ("color".into(), Expr::Ident("Red".into())),
                        (
                            "code".into(),
                            Expr::Literal(Literal::String("red_code".into())),
                        ),
                    ],
                ),
                Expr::Record(
                    None,
                    vec![
                        ("color".into(), Expr::Ident("Blue".into())),
                        (
                            "code".into(),
                            Expr::Literal(Literal::String("blue_code".into())),
                        ),
                    ],
                ),
            ]),
        };

        let struct_defs: Vec<&TypeDef> = vec![&color_td, &mapping_td];
        let item = impl_from_data_table(&dd, "color", "code", "code", &struct_defs);
        assert!(item.is_some(), "should generate impl block");
        match item.unwrap() {
            code_ir::Item::Raw(s) => {
                assert!(s.contains("impl Color"), "should impl on key type: {s}");
                assert!(s.contains("pub fn code(&self)"), "should have method: {s}");
                assert!(
                    s.contains("Self::Red => \"red_code\""),
                    "should have match arm: {s}"
                );
                assert!(
                    s.contains("Self::Blue => \"blue_code\""),
                    "should have match arm: {s}"
                );
            }
            _ => panic!("expected Raw"),
        }
    }

    #[test]
    fn unsupported_data_value_produces_compile_error() {
        // DataDef with List type but non-List value hits unsupported data value path.
        let dd = DataDef {
            name: "bad".to_string(),
            ty: TypeExpr::Generic("List".into(), vec![TypeExpr::Named("Int".into())]),
            value: Expr::Literal(Literal::Int(42)), // not Expr::List
        };
        let items = datadef_to_code_ir(&dd);
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Raw(s) => {
                assert!(
                    s.contains("compile_error!"),
                    "expected compile_error! marker, got: {s}"
                );
                assert!(
                    !s.contains("/* unsupported"),
                    "should not produce silent comment"
                );
            }
            _ => panic!("expected Raw item"),
        }
    }

    #[test]
    fn unsupported_expr_in_render_produces_compile_error() {
        // Scalar DataDef with unsupported expr (e.g. Call) hits the catch-all.
        let dd = DataDef {
            name: "bad".to_string(),
            ty: TypeExpr::Named("Int".into()),
            value: Expr::Call("foo".into(), vec![]), // unsupported in render_expr_to_rust
        };
        let items = datadef_to_code_ir(&dd);
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Raw(s) => {
                assert!(
                    s.contains("compile_error!"),
                    "expected compile_error! marker, got: {s}"
                );
                assert!(
                    !s.contains("/* unsupported"),
                    "should not produce silent comment"
                );
            }
            _ => panic!("expected Raw item"),
        }
    }

    #[test]
    fn escape_rust_string_handles_hex_escapes() {
        assert_eq!(escape_rust_string("\\x1b[0m"), "\\x1b[0m");
        assert_eq!(escape_rust_string("hello"), "hello");
        assert_eq!(escape_rust_string("a\"b"), "a\\\"b");
        assert_eq!(escape_rust_string("a\nb"), "a\\nb");
    }

    #[test]
    fn refined_type_with_width_signed_produces_i64() {
        let expr = TypeExpr::Refined(
            Box::new(TypeExpr::Named("Int".to_string())),
            vec![
                Refinement::Width(Expr::Literal(Literal::Int(64))),
                Refinement::Signed(None),
                Refinement::Arithmetic,
            ],
        );
        assert_eq!(type_expr_to_rust(&expr), "i64");
    }

    #[test]
    fn refined_type_with_width_unsigned_produces_u8() {
        let expr = TypeExpr::Refined(
            Box::new(TypeExpr::Named("Int".to_string())),
            vec![
                Refinement::Width(Expr::Literal(Literal::Int(8))),
                Refinement::Unsigned,
                Refinement::Arithmetic,
            ],
        );
        assert_eq!(type_expr_to_rust(&expr), "u8");
    }

    #[test]
    fn refined_type_with_ieee754_produces_f32() {
        let expr = TypeExpr::Refined(
            Box::new(TypeExpr::Named("Float".to_string())),
            vec![
                Refinement::Width(Expr::Literal(Literal::Int(32))),
                Refinement::Domain("ieee754_binary32".to_string()),
                Refinement::Arithmetic,
            ],
        );
        assert_eq!(type_expr_to_rust(&expr), "f32");
    }

    #[test]
    fn refined_type_without_structural_predicates_strips() {
        let expr = TypeExpr::Refined(
            Box::new(TypeExpr::Named("String".to_string())),
            vec![Refinement::NonEmpty],
        );
        // Should fall through to stripping refinements
        assert_eq!(type_expr_to_rust(&expr), "String");
    }

    /// End-to-end regression: DSL source through typecheck and Rust emit for
    /// let-bound anonymous records and `fn(Check.Output)` signatures.
    #[test]
    fn e2e_let_bound_anonymous_record_and_fn_check_output_through_rust_emit() {
        let source = r#"module sample.regression
type Config {
  value: String
}
fn consume(cfg: Config) -> String {
  cfg.value
}
fn guarded<Check>(predicate: fn(Check.Output) -> Bool) -> String {
  let cfg = { value: "ok" }
  consume(cfg)
}"#;
        let graph = module_graph_from_sources(&[("sample/regression.dag", source)]);
        let typed = daglang_typecheck::typecheck_module_graph_with_options(
            &graph,
            daglang_typecheck::TypecheckOptions {
                allow_unresolved_imports: false,
            },
        )
        .expect("DSL with let-bound anonymous record and fn(Check.Output) should typecheck");

        let rust_source = generate_types_for_modules(&typed, &[]);

        // The Config struct should be emitted.
        assert!(
            rust_source.contains("struct Config"),
            "emitted Rust should contain Config struct: {rust_source}"
        );

        // The fn(Check.Output) -> Bool parameter should render with valid Rust
        // associated-type syntax (:: not .).
        assert!(
            rust_source.contains("fn(Check::Output) -> bool"),
            "emitted Rust should render fn(Check::Output) -> bool: {rust_source}"
        );

        // The let-bound anonymous record should resolve inside the function
        // body via typecheck metadata rather than falling back to compile_error!.
        assert!(
            rust_source.contains("let cfg = Config {"),
            "let-bound anonymous record should emit as a Config constructor in the body: {rust_source}"
        );
        assert!(
            !rust_source.contains("cannot resolve anonymous record type"),
            "production emit should not degrade to compile_error!: {rust_source}"
        );
    }
}
