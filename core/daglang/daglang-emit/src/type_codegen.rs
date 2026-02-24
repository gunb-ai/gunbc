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
    match expr {
        TypeExpr::Named(name) => map_primitive(name),
        TypeExpr::Generic(name, args) => {
            let mapped = map_primitive(name);
            let arg_strs: Vec<String> = args.iter().map(type_expr_to_rust).collect();
            format!("{}<{}>", mapped, arg_strs.join(", "))
        }
        TypeExpr::Optional(inner) => {
            format!("Option<{}>", type_expr_to_rust(inner))
        }
        TypeExpr::Annotated(inner, _annotations) => type_expr_to_rust(inner),
        TypeExpr::Record(fields) => {
            let field_strs: Vec<String> = fields
                .iter()
                .map(|f| format!("{}: {}", f.name, type_expr_to_rust(&f.ty)))
                .collect();
            format!("{{ {} }}", field_strs.join(", "))
        }
    }
}

/// Map DSL primitive type names to Rust equivalents.
fn map_primitive(name: &str) -> String {
    match name {
        "Int" => "i64".to_string(),
        "Float" => "f64".to_string(),
        "Bool" => "bool".to_string(),
        "String" => "String".to_string(),
        "Char" => "char".to_string(),
        "List" => "Vec".to_string(),
        "Map" => "std::collections::HashMap".to_string(),
        other => other.to_string(),
    }
}

/// Check whether all variants of a sum type are simple (no fields).
fn is_simple_enum(variants: &[Variant]) -> bool {
    variants.iter().all(|v| v.fields.is_empty())
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

            let variant_strs: Vec<String> = variants
                .iter()
                .map(|v| format_variant(v))
                .collect();

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
                map_primitive(name)
            }
        }
        TypeExpr::Generic(name, args) => {
            let mapped = map_primitive(name);
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
                _ => "/* unsupported data value */".to_string(),
            };
            (
                format!("&[{elem_type}]"),
                format!("&[\n    {items}\n]"),
            )
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
        TypeExpr::Generic(name, args) if name == "List" && args.len() == 1 => {
            match &args[0] {
                TypeExpr::Named(n) => n.as_str(),
                _ => return vec![],
            }
        }
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
        TypeExpr::Annotated(inner, _) => type_expr_to_rust_name(inner),
        TypeExpr::Record(_) => "Anonymous".to_string(),
    }
}

/// Render a record expression for a data table entry,
/// using field type info to qualify enum variant references.
fn render_data_record(
    expr: &Expr,
    context_type: &str,
    field_types: &[(String, String)],
) -> String {
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

const STATIC_OPTS: RenderOpts = RenderOpts { static_context: true };
#[allow(dead_code)]
const RUNTIME_OPTS: RenderOpts = RenderOpts { static_context: false };

/// Render a DSL expression to a Rust expression string.
///
/// `context_type` is the Rust type name for the surrounding context,
/// used to qualify bare identifiers as enum variants.
fn render_expr_to_rust(expr: &Expr, context_type: &str, opts: RenderOpts) -> String {
    match expr {
        Expr::Literal(lit) => render_literal(lit, opts),
        Expr::Ident(name) => {
            // Bare identifiers in data contexts are enum variants.
            // `context_type` should be the enum name (e.g. "SymbolId").
            format!("{context_type}::{name}")
        }
        Expr::Record(maybe_name, fields) => {
            let type_name = maybe_name
                .as_deref()
                .unwrap_or(context_type);
            let field_strs: Vec<String> = fields
                .iter()
                .map(|(name, val)| {
                    let field_val = render_expr_to_rust(val, name, opts);
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
        _ => format!("/* unsupported expr: {expr:?} */"),
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
            '"' => { out.push_str("\\\""); i += 1; }
            '\n' => { out.push_str("\\n"); i += 1; }
            '\r' => { out.push_str("\\r"); i += 1; }
            '\t' => { out.push_str("\\t"); i += 1; }
            c if c.is_control() => {
                for byte in c.to_string().bytes() {
                    out.push_str(&format!("\\x{byte:02x}"));
                }
                i += 1;
            }
            c => { out.push(c); i += 1; }
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
pub fn fndef_to_code_ir(fd: &FnDef, data_names: &std::collections::HashSet<String>) -> Vec<code_ir::Item> {
    let params: Vec<(String, String)> = fd
        .params
        .iter()
        .map(|p| (p.name.clone(), type_expr_to_rust(&p.ty)))
        .collect();
    let ret = type_expr_to_rust(&fd.return_type);

    let body = if fd.body.stmts.is_empty() {
        vec![code_ir::Stmt::Expr(code_ir::Expr::MacroCall {
            name: "todo".to_string(),
            args: vec![code_ir::Expr::Str("generated from DSL".to_string())],
        })]
    } else {
        fn_codegen::reset_tmp_counter();
        let ctx = fn_codegen::CompileContext { data_names: data_names.clone() };
        fn_codegen::compile_fn_body(&fd.body, &ctx)
    };

    vec![code_ir::Item::Fn(code_ir::FnDef {
        name: to_snake_case(&fd.name),
        is_pub: true,
        params,
        return_type: Some(ret),
        body,
        doc: vec![],
        attributes: vec![],
    })]
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
/// ```ignore
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
        (TypeExpr::Generic(name, args), Expr::List(elems))
            if name == "List" && args.len() == 1 =>
        {
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
            let key_val = fields
                .iter()
                .find(|(n, _)| n == key_field)
                .map(|(_, v)| v);
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
    for item in items {
        if let daglang_syntax::ast::Item::DataDef(dd) = &item.node {
            data_names.insert(dd.name.clone());
        }
    }
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
                code_items.extend(fndef_to_code_ir(fd, &data_names));
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
    typed: &daglang_typecheck::TypedProject,
    module_filter: &[&str],
) -> String {
    use crate::render_rust::render_rust_source;

    // Pass 1: collect all TypeDefs and figure out which struct types
    // appear as elements in `data` lists (so we can make their String
    // fields `&'static str` for static-friendly structs).
    let mut type_defs: Vec<&TypeDef> = Vec::new();
    let mut static_struct_types: std::collections::HashSet<String> = std::collections::HashSet::new();
    for module in &typed.modules {
        let module_name = module.module_path.join(".");
        if !module_filter.is_empty() && !module_filter.contains(&module_name.as_str()) {
            continue;
        }
        for item in &module.ast.items {
            match &item.node {
                daglang_syntax::ast::Item::TypeDef(td) => {
                    type_defs.push(td);
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
                }
                _ => {}
            }
        }
    }

    // Also collect DataDefs for impl generation.
    let mut data_defs: Vec<&DataDef> = Vec::new();
    for module in &typed.modules {
        let module_name = module.module_path.join(".");
        if !module_filter.is_empty() && !module_filter.contains(&module_name.as_str()) {
            continue;
        }
        for item in &module.ast.items {
            if let daglang_syntax::ast::Item::DataDef(dd) = &item.node {
                data_defs.push(dd);
            }
        }
    }

    // Pass 2: generate all items, using type info for data tables.
    let mut all_items = Vec::new();
    for module in &typed.modules {
        let module_name = module.module_path.join(".");
        if !module_filter.is_empty() && !module_filter.contains(&module_name.as_str()) {
            continue;
        }

        for item in &module.ast.items {
            match &item.node {
                daglang_syntax::ast::Item::TypeDef(td) => {
                    if static_struct_types.contains(&td.name) {
                        all_items.extend(typedef_to_static_code_ir(td));
                    } else {
                        all_items.extend(typedef_to_code_ir(td));
                    }
                }
                daglang_syntax::ast::Item::DataDef(dd) => {
                    all_items.extend(datadef_to_code_ir_with(dd, &type_defs));
                }
                daglang_syntax::ast::Item::FnDef(fd) => {
                    let mut fn_data_names: std::collections::HashSet<String> = std::collections::HashSet::new();
                    for dd in &data_defs {
                        fn_data_names.insert(dd.name.clone());
                    }
                    all_items.extend(fndef_to_code_ir(fd, &fn_data_names));
                }
                _ => {}
            }
        }
    }

    // Pass 3: generate impl blocks from data table lookup patterns.
    // Detect DSL functions that are "lookup field in table" and generate
    // match-based impl methods instead of standalone todo!() stubs.
    for dd in &data_defs {
        let field_types = resolve_field_types_for_data(dd, &type_defs);
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
                &type_defs,
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
        all_items.insert(0, code_ir::Item::Raw(
            "#[inline]\npub fn code_point_i64(c: char) -> i64 { c as u32 as i64 }".to_string(),
        ));
    }

    let source = SourceFile {
        doc: vec!["Generated from DSL type definitions. Do not edit.".to_string()],
        items: all_items,
    };

    render_rust_source(&source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_syntax::ast::{DataDef, Expr, Field, FnDef, FnBody, Literal, Param, TypeBody, TypeDef, TypeExpr, Variant};

    #[test]
    fn simple_enum_generates_copy_hash() {
        let td = TypeDef {
            name: "SemanticColor".to_string(),
            params: vec![],
            body: TypeBody::Sum(vec![
                Variant { name: "Default".into(), fields: vec![] },
                Variant { name: "Success".into(), fields: vec![] },
                Variant { name: "Error".into(), fields: vec![] },
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
                    annotations: vec![],
                },
                Field {
                    name: "bold".into(),
                    ty: TypeExpr::Named("Bool".into()),
                    default: None,
                    annotations: vec![],
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
                assert_eq!(s.fields[0], ("color".into(), "Option<SemanticColor>".into(), true));
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
                    annotations: vec![],
                },
                Field {
                    name: "indent".into(),
                    ty: TypeExpr::Named("Int".into()),
                    default: None,
                    annotations: vec![],
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
                        annotations: vec![],
                    }],
                },
                Variant { name: "Point".into(), fields: vec![] },
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
                Field { name: "id".into(), ty: TypeExpr::Named("EntryKind".into()), default: None, annotations: vec![] },
                Field { name: "label".into(), ty: TypeExpr::Named("String".into()), default: None, annotations: vec![] },
            ]),
        };
        let dd = DataDef {
            name: "testData".to_string(),
            ty: TypeExpr::Generic("List".into(), vec![TypeExpr::Named("Entry".into())]),
            value: Expr::List(vec![
                Expr::Record(None, vec![
                    ("id".into(), Expr::Ident("Alpha".into())),
                    ("label".into(), Expr::Literal(Literal::String("first".into()))),
                ]),
            ]),
        };
        let items = datadef_to_code_ir_with(&dd, &[&entry_td]);
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Raw(s) => {
                assert!(s.contains("pub static TEST_DATA: &[Entry]"), "got: {s}");
                assert!(s.contains("id: EntryKind::Alpha"), "should resolve field type: {s}");
                assert!(s.contains(r#"label: "first""#), "static context uses &str: {s}");
                assert!(!s.contains("to_string"), "no to_string in static context: {s}");
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
                Param { name: "id".into(), ty: TypeExpr::Named("SymbolId".into()), default: None },
                Param { name: "tier".into(), ty: TypeExpr::Named("Tier".into()), default: None },
            ],
            return_type: TypeExpr::Named("String".into()),
            body: FnBody { stmts: vec![], lossy: false },
        };
        let items = fndef_to_code_ir(&fd, &std::collections::HashSet::new());
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Fn(f) => {
                assert_eq!(f.name, "resolve_symbol");
                assert!(f.is_pub);
                assert_eq!(f.params, vec![
                    ("id".to_string(), "SymbolId".to_string()),
                    ("tier".to_string(), "Tier".to_string()),
                ]);
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
            params: vec![
                Param { name: "x".into(), ty: TypeExpr::Named("Int".into()), default: None },
            ],
            return_type: TypeExpr::Named("Int".into()),
            body: FnBody {
                stmts: vec![
                    daglang_syntax::ast::Stmt::Expr(
                        Expr::BinOp(
                            Box::new(Expr::Ident("x".into())),
                            daglang_syntax::ast::BinOp::Add,
                            Box::new(Expr::Literal(Literal::Int(1))),
                        ),
                    ),
                ],
                lossy: false,
            },
        };
        let items = fndef_to_code_ir(&fd, &std::collections::HashSet::new());
        assert_eq!(items.len(), 1);
        match &items[0] {
            code_ir::Item::Fn(f) => {
                assert_eq!(f.name, "add_one");
                assert_eq!(f.body.len(), 1);
                assert!(matches!(f.body[0], code_ir::Stmt::TailExpr(code_ir::Expr::BinOp { .. })));
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
                Variant { name: "Red".into(), fields: vec![] },
                Variant { name: "Blue".into(), fields: vec![] },
            ]),
        };
        let mapping_td = TypeDef {
            name: "ColorMapping".to_string(),
            params: vec![],
            body: TypeBody::Record(vec![
                Field { name: "color".into(), ty: TypeExpr::Named("Color".into()), default: None, annotations: vec![] },
                Field { name: "code".into(), ty: TypeExpr::Named("String".into()), default: None, annotations: vec![] },
            ]),
        };
        let dd = DataDef {
            name: "mappings".to_string(),
            ty: TypeExpr::Generic("List".into(), vec![TypeExpr::Named("ColorMapping".into())]),
            value: Expr::List(vec![
                Expr::Record(None, vec![
                    ("color".into(), Expr::Ident("Red".into())),
                    ("code".into(), Expr::Literal(Literal::String("red_code".into()))),
                ]),
                Expr::Record(None, vec![
                    ("color".into(), Expr::Ident("Blue".into())),
                    ("code".into(), Expr::Literal(Literal::String("blue_code".into()))),
                ]),
            ]),
        };

        let struct_defs: Vec<&TypeDef> = vec![&color_td, &mapping_td];
        let item = impl_from_data_table(&dd, "color", "code", "code", &struct_defs);
        assert!(item.is_some(), "should generate impl block");
        match item.unwrap() {
            code_ir::Item::Raw(s) => {
                assert!(s.contains("impl Color"), "should impl on key type: {s}");
                assert!(s.contains("pub fn code(&self)"), "should have method: {s}");
                assert!(s.contains("Self::Red => \"red_code\""), "should have match arm: {s}");
                assert!(s.contains("Self::Blue => \"blue_code\""), "should have match arm: {s}");
            }
            _ => panic!("expected Raw"),
        }
    }

    #[test]
    fn escape_rust_string_handles_hex_escapes() {
        assert_eq!(escape_rust_string("\\x1b[0m"), "\\x1b[0m");
        assert_eq!(escape_rust_string("hello"), "hello");
        assert_eq!(escape_rust_string("a\"b"), "a\\\"b");
        assert_eq!(escape_rust_string("a\nb"), "a\\nb");
    }
}
