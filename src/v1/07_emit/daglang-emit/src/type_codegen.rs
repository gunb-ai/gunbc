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

use daglang_syntax::ast::{DataDef, Expr, FnDef, Literal, TypeBody, TypeDef, TypeExpr, Variant};
use daglang_syntax::span::Spanned;
use gunbc_ir::code_ir::{self, EnumDef, SourceFile, StructDef};

use crate::fn_codegen;

/// Default derives applied to every generated type.
const DEFAULT_DERIVES: &[&str] = &["Debug", "Clone", "PartialEq", "Eq"];

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
        TypeExpr::Named(name) => crate::type_mapping::resolve_and_emit(
            name,
            registry,
            crate::type_mapping::Backend::Rust,
        ),
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
                language_model::resolve_container(kind, &inner, key, model)
                    .unwrap_or_else(|| format!("{}<{}>", name, arg_strs.join(", ")))
            } else {
                let mapped = crate::type_mapping::resolve_and_emit(
                    name,
                    registry,
                    crate::type_mapping::Backend::Rust,
                );
                format!("{}<{}>", mapped, arg_strs.join(", "))
            }
        }
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

            vec![code_ir::Item::Enum(EnumDef {
                name: td.name.clone(),
                is_pub: true,
                derives,
                variants: variant_strs,
                doc: vec![],
            })]
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
                "List" => "Vec".to_string(),
                "Map" => "std::collections::HashMap".to_string(),
                "Set" => "std::collections::HashSet".to_string(),
                other => crate::type_mapping::resolve_and_emit(
                    other,
                    None,
                    crate::type_mapping::Backend::Rust,
                ),
            };
            let arg_strs: Vec<String> = args.iter().map(type_expr_to_static_rust).collect();
            format!("{}<{}>", mapped, arg_strs.join(", "))
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
        TypeExpr::Generic(name, args) if name == "Map" && args.len() == 2 => {
            let key_type = type_expr_to_rust(&args[0]);
            let val_type = type_expr_to_rust(&args[1]);
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

/// Get the simple type name (without Option wrapping) for field context resolution.
fn type_expr_to_rust_name(expr: &TypeExpr) -> String {
    match expr {
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::Optional(inner) => type_expr_to_rust_name(inner),
        TypeExpr::Generic(name, _) => name.clone(),
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
                    format!("({key_str}.to_string(), {val_str})")
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
    // Pre-pass: synthesize struct definitions for anonymous records (fold inits).
    let (synth_items, _name_map, new_field_types) =
        fn_codegen::synthesize_anonymous_structs(&fd.name, &fd.body, &ctx.struct_field_types);

    // Collect optional parameters (T? → Option<T>)
    let optional_params: std::collections::HashSet<String> = fd
        .params
        .iter()
        .filter(|p| matches!(&p.ty, TypeExpr::Optional(_)))
        .map(|p| p.name.clone())
        .collect();

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

    let ctx = {
        let mut augmented = ctx.clone();
        augmented.struct_field_types.extend(new_field_types);
        augmented.optional_params = optional_params;
        augmented.param_types = param_types;
        augmented.current_return_type = return_type_name;
        augmented.current_return_ir_type = Some(fn_codegen::type_expr_to_ir_type(&fd.return_type));
        augmented.ir_scope = ir_scope;
        std::borrow::Cow::Owned(augmented)
    };

    let mut params: Vec<(String, String)> = fd
        .params
        .iter()
        .map(|p| (p.name.clone(), type_expr_to_rust(&p.ty)))
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

    let mut items = synth_items;
    items.push(code_ir::Item::Fn(code_ir::FnDef {
        name: to_snake_case(&fd.name),
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
        optional_params: std::collections::HashSet::new(),
        param_types: std::collections::HashMap::new(),
        current_return_type: None,
        current_return_ir_type: None,
        ir_scope: std::collections::HashMap::new(),
        struct_field_ir_types: std::collections::HashMap::new(),
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

    // Pass 1: collect cloned AST items from matching modules.
    // TypedModuleRef temporaries from modules() don't live long enough
    // for cross-pass reference storage, so we clone the needed items.
    let mut type_defs: Vec<TypeDef> = Vec::new();
    let mut data_defs: Vec<DataDef> = Vec::new();
    let mut fn_defs: Vec<FnDef> = Vec::new();
    let mut static_struct_types: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for module in typed.modules() {
        let module_name = module.module_path.as_dotted();
        if !module_filter.is_empty() && !module_filter.contains(&module_name.as_str()) {
            continue;
        }
        for item in &module.ast.items {
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
                daglang_syntax::ast::Item::FnDef(fd) => {
                    fn_defs.push(fd.clone());
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

    for fd in &fn_defs {
        let mut fn_data_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for dd in &data_defs {
            fn_data_names.insert(dd.name.clone());
        }
        let mut opt_fields = std::collections::HashMap::new();
        let mut v2e = std::collections::HashMap::new();
        let mut ambig = std::collections::HashSet::new();
        let mut sft = std::collections::HashMap::new();
        let mut ev = std::collections::HashMap::new();
        for td in &type_defs {
            collect_optional_fields(td, &mut opt_fields);
            collect_variant_to_enum(td, &mut v2e, &mut ambig);
            collect_struct_field_types(td, &mut sft);
            collect_enum_variants(td, &mut ev);
        }
        let fn_ctx = fn_codegen::CompileContext {
            data_names: fn_data_names,
            data_map_names: std::collections::HashSet::new(),
            optional_fields: opt_fields,
            variant_to_enum: v2e,
            struct_field_types: sft,
            enum_variants: ev,
            boxed_fields: std::collections::HashSet::new(),
            fn_return_types: std::collections::HashMap::new(),
            optional_params: std::collections::HashSet::new(),
            param_types: std::collections::HashMap::new(),
            current_return_type: None,
            ir_scope: std::collections::HashMap::new(),
            struct_field_ir_types: std::collections::HashMap::new(),
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

use std::collections::{HashMap, HashSet};

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

    for type_name in all_type_names.iter() {
        if !visited.contains(type_name) {
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

    recursive_fields
}

/// Collect direct (non-List-wrapped) type references from a TypeExpr.
/// Only direct references need boxing; List<T> is already heap-allocated.
fn collect_direct_type_refs(
    ty: &TypeExpr,
    field_name: &str,
    known_types: &HashSet<String>,
    edges: &mut Vec<(String, String)>,
) {
    match ty {
        TypeExpr::Named(name) => {
            if known_types.contains(name) {
                edges.push((field_name.to_string(), name.clone()));
            }
        }
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
    use daglang_syntax::ast::{
        DataDef, Expr, Field, FnBody, FnDef, Literal, Param, Refinement, TypeBody, TypeDef,
        TypeExpr, Variant,
    };

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
    fn generic_list_maps_to_vec() {
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
                assert_eq!(s.fields[0].1, "Vec<Span>");
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
}
