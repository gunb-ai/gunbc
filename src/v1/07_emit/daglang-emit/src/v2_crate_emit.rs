//! v2 compiler crate assembly — produces a complete Cargo crate from v2 .dag modules.
//!
//! Given parsed v2 module ASTs, this module:
//! 1. Emits types with recursive field boxing (Phase 4)
//! 2. Emits functions via fn_codegen
//! 3. Writes runtime shims (v2_rt module)
//! 4. Assembles a complete crate with Cargo.toml and lib.rs
//!
//! The emitted crate is written to `target/v2-compiler/`.
//!
//! ## TEMPORARY bootstrap scaffolding (remove after self-hosting)
//!
//! This module contains several categories of hardcoded knowledge that should
//! be derived from the .dag source files instead:
//!
//! - **`std_types_prelude()`**: Materializes types (`SourceSpan`, `BindingPower`,
//!   `ItemResult`, etc.) that the .dag source imports from `std.types` or uses as
//!   anonymous records. In a self-hosted compiler, these would come from the .dag
//!   type definitions themselves.
//!
//! - **`module_prelude()`**: Hardcodes cross-module `use` statements. Should be
//!   derived from `import` declarations in each .dag file.
//!
//! - **Hardcoded `struct_field_types` entries**: Manual registry of the materialized
//!   types' field layouts. Should be generated from the type definitions.
//!
//! - **`V2_MODULE_MAP`**: Hardcoded .dag stem → Rust module name mapping. Should
//!   be derived from module declarations.
//!
//! All of these exist because the v1 emitter's pipeline doesn't have a "resolve
//! imports" phase — it works on individual parsed modules without cross-module
//! knowledge. The v2 compiler's resolve phase handles this properly.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use daglang_syntax::ast::{Item, TypeDef, TypeBody};
use gunbc_ir::code_ir;

use crate::fn_codegen;
use crate::render_rust;
use crate::type_codegen;
use crate::v2_runtime_shim;

/// Mapping from v2 .dag file stems to Rust module names.
/// `core` is reserved in Rust, so 00_core.dag maps to `v2_core`.
const V2_MODULE_MAP: &[(&str, &str)] = &[
    ("00_core", "v2_core"),
    ("01_tokenize", "tokenize"),
    ("02_parse", "parse"),
    ("03_resolve", "resolve"),
    ("04_typecheck", "typecheck"),
    ("05_emit", "emit"),
    ("06_pipeline", "pipeline"),
];

/// A generated file with its path relative to the crate root and content.
#[derive(Debug)]
pub struct GeneratedFile {
    pub rel_path: String,
    pub content: String,
}

/// Assemble a complete Rust crate from parsed v2 module ASTs.
///
/// `modules` is a list of (dag_stem, parsed_items) pairs, where dag_stem
/// is e.g. "00_core" and parsed_items are the AST items from that file.
pub fn assemble_v2_crate(
    modules: &[(&str, &[daglang_syntax::span::Spanned<Item>])],
) -> Vec<GeneratedFile> {
    let mut files = Vec::new();

    // 1. Collect all type definitions across modules for recursive field analysis
    let all_type_defs: Vec<&TypeDef> = modules
        .iter()
        .flat_map(|(_, items)| {
            items.iter().filter_map(|item| match &item.node {
                Item::TypeDef(td) => Some(td),
                _ => None,
            })
        })
        .collect();

    let recursive_fields = type_codegen::compute_recursive_fields(&all_type_defs);

    // 2. Build variant_to_enum map for identifier resolution
    let variant_to_enum = build_variant_to_enum(&all_type_defs);

    // 3. Build struct_field_types map for record construction
    let mut struct_field_types = build_struct_field_types(&all_type_defs);
    // Add materialized types from std_types_prelude and extra helpers
    struct_field_types.insert("BindingPower".to_string(), {
        let mut m = HashMap::new();
        m.insert("left".to_string(), "i64".to_string());
        m.insert("right".to_string(), "i64".to_string());
        m
    });
    struct_field_types.insert("ItemResult".to_string(), {
        let mut m = HashMap::new();
        m.insert("item".to_string(), "Item".to_string());
        m.insert("diagnostics".to_string(), "Vec".to_string());
        m
    });
    struct_field_types.insert("VariantResult".to_string(), {
        let mut m = HashMap::new();
        m.insert("variant".to_string(), "Variant".to_string());
        m.insert("diagnostics".to_string(), "Vec".to_string());
        m
    });
    struct_field_types.insert("CapabilityResult".to_string(), {
        let mut m = HashMap::new();
        m.insert("capability".to_string(), "OperationDef".to_string());
        m.insert("diagnostics".to_string(), "Vec".to_string());
        m
    });
    struct_field_types.insert("SourceSpan".to_string(), {
        let mut m = HashMap::new();
        m.insert("start".to_string(), "i64".to_string());
        m.insert("end".to_string(), "i64".to_string());
        m
    });
    struct_field_types.insert("ParamResult".to_string(), {
        let mut m = HashMap::new();
        m.insert("param".to_string(), "Param".to_string());
        m.insert("diagnostics".to_string(), "Vec".to_string());
        m
    });
    struct_field_types.insert("OperationResult".to_string(), {
        let mut m = HashMap::new();
        m.insert("operation".to_string(), "OperationDef".to_string());
        m.insert("diagnostics".to_string(), "Vec".to_string());
        m
    });
    // Note: MatchPattern is defined in the .dag source — don't add it here.

    // 4. Build optional_fields map
    let optional_fields = build_optional_fields(&all_type_defs);

    // 5. Emit each module
    for (dag_stem, items) in modules {
        let rust_mod = match V2_MODULE_MAP.iter().find(|(stem, _)| stem == dag_stem) {
            Some((_, rust_name)) => *rust_name,
            None => continue,
        };

        let source = emit_module(
            items,
            &recursive_fields,
            &variant_to_enum,
            &struct_field_types,
            &optional_fields,
        );
        let mut content = module_prelude(dag_stem);
        content.push_str(&render_rust::render_rust_source(&source));
        files.push(GeneratedFile {
            rel_path: format!("src/{}.rs", rust_mod),
            content,
        });
    }

    // 6. Emit lib.rs with mod declarations and cross-module uses
    let lib_content = emit_lib_rs();
    files.push(GeneratedFile {
        rel_path: "src/lib.rs".to_string(),
        content: lib_content,
    });

    // 7. Emit v2_rt.rs runtime shims
    files.push(GeneratedFile {
        rel_path: "src/v2_rt.rs".to_string(),
        content: v2_runtime_shim::V2_RUNTIME_SOURCE.to_string(),
    });

    // 8. Emit Cargo.toml (standalone — not part of any workspace)
    let mut cargo_toml = render_rust::render_cargo_toml("v2-compiler", &[]);
    cargo_toml.push_str("\n[workspace]\n");
    files.push(GeneratedFile {
        rel_path: "Cargo.toml".to_string(),
        content: cargo_toml,
    });

    files
}

/// Write assembled crate files to disk under the given output directory.
pub fn write_crate(output_dir: &Path, files: &[GeneratedFile]) -> std::io::Result<()> {
    for file in files {
        let path = output_dir.join(&file.rel_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &file.content)?;
    }
    Ok(())
}

fn emit_lib_rs() -> String {
    let mut out = String::new();
    out.push_str("//! v2 DAG compiler — generated from .dag source files.\n\n");
    out.push_str("#![allow(unused_imports, unused_variables, unused_mut, dead_code, unreachable_patterns, clippy::all)]\n\n");

    // Module declarations
    for (_, rust_mod) in V2_MODULE_MAP {
        out.push_str(&format!("pub mod {};\n", rust_mod));
    }
    out.push_str("pub mod v2_rt;\n");

    out
}

/// Types imported from std.types that must be materialized in the generated crate.
/// These types come from `import std.types { ... }` in 00_core.dag but don't
/// exist in the generated crate unless we define them.
fn std_types_prelude() -> &'static str {
    r#"
/// Source span for diagnostic reporting.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceSpan {
    pub start: i64,
    pub end: i64,
}

/// Type alias for file paths.
pub type FilePath = String;

/// Non-empty string type (alias — validation not enforced at type level).
pub type NonEmptyStr = String;

/// Binding power for Pratt parser precedence levels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingPower {
    pub left: i64,
    pub right: i64,
}

/// Parser result types used in parse.dag.
/// These use wrapper functions for construction since Rust doesn't have
/// default field values.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemResult {
    pub item: Item,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariantResult {
    pub variant: Variant,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityResult {
    pub capability: OperationDef,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamResult {
    pub param: Param,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationResult {
    pub operation: OperationDef,
    pub diagnostics: Vec<Diagnostic>,
}

"#
}

/// Generate module-level prelude: use statements for cross-module types.
fn module_prelude(dag_stem: &str) -> String {
    let mut prelude = String::new();

    // Core module gets materialized std.types imports
    if dag_stem == "00_core" {
        prelude.push_str(std_types_prelude());
    }

    // All non-core modules import everything from v2_core
    if dag_stem != "00_core" {
        prelude.push_str("use crate::v2_core::*;\n");
    }

    // All modules get access to the runtime shims and std collections
    prelude.push_str("use crate::v2_rt;\n");
    prelude.push_str("use std::collections::HashMap;\n");
    // Map type alias is defined only in v2_core to avoid redefinition conflicts
    if dag_stem == "00_core" {
        prelude.push_str("pub type Map<K, V> = HashMap<K, V>;\n");
    }

    // Module-specific cross-imports
    match dag_stem {
        "02_parse" => {
            prelude.push_str("use crate::tokenize::*;\n");
        }
        "03_resolve" => {
            prelude.push_str("use crate::parse::*;\n");
        }
        "04_typecheck" => {
            prelude.push_str("use crate::parse::*;\n");
            prelude.push_str("use crate::resolve::*;\n");
        }
        "05_emit" => {
            prelude.push_str("use crate::parse::*;\n");
            prelude.push_str("use crate::typecheck::*;\n");
        }
        "06_pipeline" => {
            prelude.push_str("use crate::tokenize::*;\n");
            prelude.push_str("use crate::parse::*;\n");
            prelude.push_str("use crate::resolve::*;\n");
            prelude.push_str("use crate::typecheck::*;\n");
            prelude.push_str("use crate::emit::*;\n");
        }
        _ => {}
    }

    prelude.push('\n');
    prelude
}

fn emit_module(
    items: &[daglang_syntax::span::Spanned<Item>],
    recursive_fields: &HashSet<(String, String)>,
    variant_to_enum: &HashMap<String, String>,
    struct_field_types: &HashMap<String, HashMap<String, String>>,
    optional_fields: &HashMap<String, HashSet<String>>,
) -> code_ir::SourceFile {
    let mut ir_items: Vec<code_ir::Item> = Vec::new();

    // Collect data names for compile context
    let data_names: HashSet<String> = items
        .iter()
        .filter_map(|item| match &item.node {
            Item::DataDef(dd) => Some(dd.name.clone()),
            _ => None,
        })
        .collect();

    // Build enum_variants map from type defs
    let enum_variants_map: HashMap<String, HashSet<String>> = items
        .iter()
        .filter_map(|item| match &item.node {
            Item::TypeDef(td) => {
                if let TypeBody::Sum(variants) = &td.body {
                    let names: HashSet<String> = variants.iter().map(|v| v.name.clone()).collect();
                    Some((td.name.clone(), names))
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();

    let ctx = fn_codegen::CompileContext {
        data_names,
        optional_fields: optional_fields.clone(),
        variant_to_enum: variant_to_enum.clone(),
        struct_field_types: struct_field_types.clone(),
        enum_variants: enum_variants_map,
        boxed_fields: recursive_fields.clone(),
    };

    for item in items {
        match &item.node {
            Item::TypeDef(td) => {
                ir_items.extend(type_codegen::typedef_to_code_ir_boxed(td, recursive_fields));
            }
            Item::FnDef(fd) => {
                ir_items.extend(type_codegen::fndef_to_code_ir(fd, &ctx));
            }
            Item::DataDef(dd) => {
                // Collect struct TypeDefs for field-type resolution
                let struct_defs: Vec<&TypeDef> = items
                    .iter()
                    .filter_map(|i| match &i.node {
                        Item::TypeDef(td) => Some(td),
                        _ => None,
                    })
                    .collect();
                ir_items.extend(type_codegen::datadef_to_code_ir_with(dd, &struct_defs));
            }
            // Skip module/import/service/resource/interface/pipeline/extern declarations
            _ => {}
        }
    }

    code_ir::SourceFile {
        doc: vec![],
        items: ir_items,
    }
}

/// Build variant_name → enum_name map from type definitions.
/// For ambiguous variants (present in multiple enums), picks the first match.
/// This is imperfect but produces compilable code in most cases — the user
/// would need to explicitly qualify in the .dag source for true ambiguity.
fn build_variant_to_enum(type_defs: &[&TypeDef]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for td in type_defs {
        if let TypeBody::Sum(variants) = &td.body {
            for v in variants {
                // First definition wins. This means the order of type definitions
                // matters for ambiguous variants, which matches .dag file order.
                map.entry(v.name.clone()).or_insert_with(|| td.name.clone());
            }
        }
    }
    map
}

/// Build struct_name → { field_name → field_type_name } map.
fn build_struct_field_types(type_defs: &[&TypeDef]) -> HashMap<String, HashMap<String, String>> {
    let mut map = HashMap::new();
    for td in type_defs {
        match &td.body {
            TypeBody::Record(fields) => {
                let field_map: HashMap<String, String> = fields
                    .iter()
                    .map(|f| (f.name.clone(), type_expr_to_rust_name(&f.ty)))
                    .collect();
                map.insert(td.name.clone(), field_map);
            }
            TypeBody::Sum(variants) => {
                for v in variants {
                    if !v.fields.is_empty() {
                        let field_map: HashMap<String, String> = v
                            .fields
                            .iter()
                            .map(|f| (f.name.clone(), type_expr_to_rust_name(&f.ty)))
                            .collect();
                        map.insert(v.name.clone(), field_map);
                    }
                }
            }
            _ => {}
        }
    }
    map
}

/// Extract the simple type name from a TypeExpr.
fn type_expr_to_rust_name(expr: &daglang_syntax::ast::TypeExpr) -> String {
    match expr {
        daglang_syntax::ast::TypeExpr::Named(name) => name.clone(),
        daglang_syntax::ast::TypeExpr::Optional(inner) => type_expr_to_rust_name(inner),
        daglang_syntax::ast::TypeExpr::Generic(name, _) => name.clone(),
        daglang_syntax::ast::TypeExpr::Refined(inner, _) => type_expr_to_rust_name(inner),
        daglang_syntax::ast::TypeExpr::Record(_) => "Anonymous".to_string(),
    }
}

/// Build struct_name → { optional field names } map.
fn build_optional_fields(type_defs: &[&TypeDef]) -> HashMap<String, HashSet<String>> {
    let mut map = HashMap::new();
    for td in type_defs {
        match &td.body {
            TypeBody::Record(fields) => {
                let opt: HashSet<String> = fields
                    .iter()
                    .filter(|f| matches!(&f.ty, daglang_syntax::ast::TypeExpr::Optional(_)))
                    .map(|f| f.name.clone())
                    .collect();
                if !opt.is_empty() {
                    map.insert(td.name.clone(), opt);
                }
            }
            TypeBody::Sum(variants) => {
                for v in variants {
                    let opt: HashSet<String> = v
                        .fields
                        .iter()
                        .filter(|f| matches!(&f.ty, daglang_syntax::ast::TypeExpr::Optional(_)))
                        .map(|f| f.name.clone())
                        .collect();
                    if !opt.is_empty() {
                        map.insert(v.name.clone(), opt);
                    }
                }
            }
            _ => {}
        }
    }
    map
}
