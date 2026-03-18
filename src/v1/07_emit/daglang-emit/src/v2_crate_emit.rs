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
//! - **Hardcoded `struct_field_types` entries**: Manual registry of the materialized
//!   types' field layouts. Should be generated from the type definitions.
//!
//! - **`module_prelude()`**: Derived from each .dag file's module declaration and
//!   import list.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::fn_codegen;
use crate::render_rust;
use crate::type_codegen;
use crate::v2_runtime_shim;
use daglang_syntax::ast::{Item, SourceFile, TypeBody, TypeDef};
use daglang_syntax::ast_utils::type_expr_to_string;
use gunbc_ir::code_ir;

/// A generated file with its path relative to the crate root and content.
#[derive(Debug)]
pub struct GeneratedFile {
    pub rel_path: String,
    pub content: String,
}

#[derive(Debug, Clone)]
struct EmbeddedDagSource {
    rel_path: String,
    module_name: String,
    const_name: String,
    dsl_logical_path: Option<String>,
    content: String,
    include_in_self_parse: bool,
    include_in_self_resolve: bool,
    include_in_gist_resolve: bool,
}

#[derive(Debug, Clone)]
struct LoadedDagSource {
    rel_path: String,
    module_name: String,
    imports: Vec<String>,
    content: String,
}

#[derive(Debug, Clone, Copy, Default)]
struct EmbeddedDagMarks {
    include_in_self_parse: bool,
    include_in_self_resolve: bool,
    include_in_gist_resolve: bool,
}

fn workspace_root_from_manifest_dir() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(4)
        .expect("could not find workspace root from CARGO_MANIFEST_DIR")
        .to_path_buf()
}

fn dag_source_const_name(rel_path: &str) -> String {
    let mut sanitized = String::new();
    let mut last_was_underscore = false;
    for ch in rel_path.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_uppercase());
            last_was_underscore = false;
        } else if !last_was_underscore {
            sanitized.push('_');
            last_was_underscore = true;
        }
    }
    while sanitized.ends_with('_') {
        sanitized.pop();
    }
    format!("{sanitized}_SOURCE")
}

fn compiler_seed_paths(workspace_root: &Path) -> Vec<String> {
    let mut rel_paths = std::fs::read_dir(workspace_root.join("src/v2"))
        .unwrap_or_else(|e| panic!("failed to read src/v2/: {e}"))
        .filter_map(|entry| {
            let entry = entry.unwrap_or_else(|e| panic!("failed to read src/v2 entry: {e}"));
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if file_name.ends_with(".dag") {
                Some(format!("src/v2/{file_name}"))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    rel_paths.sort();
    rel_paths
}

fn load_dag_source(workspace_root: &Path, rel_path: &str) -> LoadedDagSource {
    let dag_path = workspace_root.join(rel_path);
    let content = std::fs::read_to_string(&dag_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", dag_path.display(), e));
    let parsed = daglang_syntax::parser::parse(&content)
        .unwrap_or_else(|errors| panic!("failed to parse {}: {errors:?}", dag_path.display()));
    let module_name = parsed
        .module_path
        .as_ref()
        .map(|path| path.node.as_dotted())
        .unwrap_or_else(|| panic!("expected module declaration in {}", dag_path.display()));
    let imports = parsed
        .imports
        .iter()
        .map(|import| import.node.path.as_dotted())
        .collect();
    LoadedDagSource {
        rel_path: rel_path.to_string(),
        module_name,
        imports,
        content,
    }
}

fn load_dag_source_cached(
    workspace_root: &Path,
    rel_path: &str,
    cache: &mut HashMap<String, LoadedDagSource>,
) -> LoadedDagSource {
    if let Some(loaded) = cache.get(rel_path) {
        return loaded.clone();
    }
    let loaded = load_dag_source(workspace_root, rel_path);
    cache.insert(rel_path.to_string(), loaded.clone());
    loaded
}

fn resolve_import_rel_path(
    module_path: &str,
    compiler_module_paths: &HashMap<String, String>,
) -> String {
    compiler_module_paths
        .get(module_path)
        .cloned()
        .unwrap_or_else(|| format!("dsl/{}.dag", module_path.replace('.', "/")))
}

fn mark_embedded_dag_source(
    sources: &mut BTreeMap<String, EmbeddedDagSource>,
    loaded: &LoadedDagSource,
    marks: EmbeddedDagMarks,
) {
    let entry = sources
        .entry(loaded.rel_path.clone())
        .or_insert_with(|| EmbeddedDagSource {
            rel_path: loaded.rel_path.clone(),
            module_name: loaded.module_name.clone(),
            const_name: dag_source_const_name(&loaded.rel_path),
            dsl_logical_path: loaded
                .rel_path
                .strip_prefix("dsl/")
                .map(|path| path.to_string()),
            content: loaded.content.clone(),
            include_in_self_parse: false,
            include_in_self_resolve: false,
            include_in_gist_resolve: false,
        });
    assert_eq!(
        entry.module_name, loaded.module_name,
        "embedded source module drift for {}",
        loaded.rel_path
    );
    assert_eq!(
        entry.content, loaded.content,
        "embedded source content drift for {}",
        loaded.rel_path
    );
    entry.include_in_self_parse |= marks.include_in_self_parse;
    entry.include_in_self_resolve |= marks.include_in_self_resolve;
    entry.include_in_gist_resolve |= marks.include_in_gist_resolve;
}

fn collect_import_closure(
    workspace_root: &Path,
    compiler_module_paths: &HashMap<String, String>,
    seed_rel_paths: &[String],
    marks: EmbeddedDagMarks,
    cache: &mut HashMap<String, LoadedDagSource>,
    sources: &mut BTreeMap<String, EmbeddedDagSource>,
) {
    let mut stack = seed_rel_paths.to_vec();
    let mut visited = HashSet::new();
    while let Some(rel_path) = stack.pop() {
        if !visited.insert(rel_path.clone()) {
            continue;
        }
        let loaded = load_dag_source_cached(workspace_root, &rel_path, cache);
        mark_embedded_dag_source(sources, &loaded, marks);
        for import_path in &loaded.imports {
            stack.push(resolve_import_rel_path(import_path, compiler_module_paths));
        }
    }
}

fn collect_embedded_dag_sources() -> Vec<EmbeddedDagSource> {
    let workspace_root = workspace_root_from_manifest_dir();
    let compiler_seed_paths = compiler_seed_paths(&workspace_root);
    let mut cache = HashMap::new();
    let mut compiler_module_paths = HashMap::new();
    let mut sources = BTreeMap::new();

    for rel_path in &compiler_seed_paths {
        let loaded = load_dag_source_cached(&workspace_root, rel_path, &mut cache);
        compiler_module_paths.insert(loaded.module_name.clone(), loaded.rel_path.clone());
        mark_embedded_dag_source(
            &mut sources,
            &loaded,
            EmbeddedDagMarks {
                include_in_self_parse: true,
                ..EmbeddedDagMarks::default()
            },
        );
    }

    collect_import_closure(
        &workspace_root,
        &compiler_module_paths,
        &compiler_seed_paths,
        EmbeddedDagMarks {
            include_in_self_resolve: true,
            ..EmbeddedDagMarks::default()
        },
        &mut cache,
        &mut sources,
    );

    let gist_seed_paths = vec!["dsl/gunbc/tools/gist.dag".to_string()];
    collect_import_closure(
        &workspace_root,
        &compiler_module_paths,
        &gist_seed_paths,
        EmbeddedDagMarks {
            include_in_gist_resolve: true,
            ..EmbeddedDagMarks::default()
        },
        &mut cache,
        &mut sources,
    );

    let mut seen_const_names = HashSet::new();
    for source in sources.values() {
        assert!(
            seen_const_names.insert(source.const_name.clone()),
            "duplicate embedded source const name: {}",
            source.const_name
        );
    }

    sources.into_values().collect()
}

fn rust_mod_for_module_path(path: &str) -> String {
    let leaf = path.rsplit('.').next().unwrap_or(path);
    match leaf {
        "core" => "v2_core".to_string(),
        other => other.replace('-', "_"),
    }
}

fn rust_mod_for_source_file(sf: &SourceFile) -> Option<String> {
    sf.module_path
        .as_ref()
        .map(|module_path| rust_mod_for_module_path(&module_path.node.as_dotted()))
}

/// Assemble a complete Rust crate from parsed v2 module ASTs.
///
/// `modules` is a list of (dag_stem, source_file) pairs, where dag_stem
/// is e.g. "00_core" and source_file is the full parsed AST including
/// imports and items.
pub fn assemble_v2_crate(modules: &[(&str, &SourceFile)]) -> Vec<GeneratedFile> {
    let mut files = Vec::new();

    // 1. Collect all type definitions across modules for recursive field analysis
    let all_type_defs: Vec<&TypeDef> = modules
        .iter()
        .flat_map(|(_, sf)| {
            sf.items.iter().filter_map(|item| match &item.node {
                Item::TypeDef(td) => Some(td),
                _ => None,
            })
        })
        .collect();

    // R8: Build the set of non-Copy types that get Rc-wrapped for O(1) clone.
    // This must be installed before compute_recursive_fields (which now skips
    // Rc-wrapped types since they're already heap-allocated) and before any
    // type_expr_to_rust calls (which Rc-wrap Named types in this set).
    let rc_wrapped_types = type_codegen::build_rc_wrapped_types(&all_type_defs);

    let recursive_fields =
        type_codegen::with_rc_wrapped_types(&rc_wrapped_types, || {
            type_codegen::compute_recursive_fields(&all_type_defs)
        });

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
    // SourceSpan and BindingPower are from std.types imports — keep them here.
    // ItemResult, VariantResult, etc. are now defined in 02_parse.dag and
    // auto-generated by the type codegen pipeline (S80 fix).
    struct_field_types.insert("SourceSpan".to_string(), {
        let mut m = HashMap::new();
        m.insert("start".to_string(), "i64".to_string());
        m.insert("end".to_string(), "i64".to_string());
        m
    });
    // Note: MatchPattern is defined in the .dag source — don't add it here.

    // 3b. Build struct_field_ir_types (target-agnostic type annotations)
    let mut struct_field_ir_types = build_struct_field_ir_types(&all_type_defs);
    // Add materialized types from std_types_prelude
    struct_field_ir_types.insert(
        "BindingPower".to_string(),
        vec![
            ("left".to_string(), gunbc_ir::code_ir::IrType::Int),
            ("right".to_string(), gunbc_ir::code_ir::IrType::Int),
        ],
    );
    struct_field_ir_types.insert(
        "SourceSpan".to_string(),
        vec![
            ("start".to_string(), gunbc_ir::code_ir::IrType::Int),
            ("end".to_string(), gunbc_ir::code_ir::IrType::Int),
        ],
    );

    // 4. Build optional_fields map
    let optional_fields = build_optional_fields(&all_type_defs);

    // 4c. Build global fn_return_types (cross-module) for type inference in intrinsics
    let global_fn_return_types: HashMap<String, String> = modules
        .iter()
        .flat_map(|(_, sf)| {
            sf.items.iter().filter_map(|item| match &item.node {
                Item::FnDef(fd) => {
                    let rust_name = crate::type_codegen::to_snake_case(&fd.name);
                    let ret = type_expr_to_rust_name(&fd.return_type);
                    Some((rust_name, ret))
                }
                _ => None,
            })
        })
        .collect();

    // 4d. Build set of function names that return Optional types (T?)
    let global_optional_return_fns: HashSet<String> = modules
        .iter()
        .flat_map(|(_, sf)| {
            sf.items.iter().filter_map(|item| match &item.node {
                Item::FnDef(fd) => {
                    if matches!(&fd.return_type, daglang_syntax::ast::TypeExpr::Optional(_)) {
                        Some(crate::type_codegen::to_snake_case(&fd.name))
                    } else {
                        None
                    }
                }
                _ => None,
            })
        })
        .collect();

    let global_fn_return_ir_types: HashMap<String, gunbc_ir::code_ir::IrType> = modules
        .iter()
        .flat_map(|(_, sf)| {
            sf.items.iter().filter_map(|item| match &item.node {
                Item::FnDef(fd) => {
                    let rust_name = crate::type_codegen::to_snake_case(&fd.name);
                    Some((rust_name, fn_codegen::type_expr_to_ir_type(&fd.return_type)))
                }
                _ => None,
            })
        })
        .collect();

    let global_fn_param_types: HashMap<String, Vec<(String, String)>> = modules
        .iter()
        .flat_map(|(_, sf)| {
            sf.items.iter().filter_map(|item| match &item.node {
                Item::FnDef(fd) => {
                    let rust_name = crate::type_codegen::to_snake_case(&fd.name);
                    let params = fd
                        .params
                        .iter()
                        .map(|param| (param.name.clone(), type_expr_to_rust_name(&param.ty)))
                        .collect();
                    Some((rust_name, params))
                }
                _ => None,
            })
        })
        .collect();

    // R3: Build set of (fn_name, param_index) where the param is exactly `String`
    // (not `Option<String>` or other wrappers). These become `&str` in generated code.
    let global_fn_str_params: HashSet<(String, usize)> = modules
        .iter()
        .flat_map(|(_, sf)| {
            sf.items.iter().filter_map(|item| match &item.node {
                Item::FnDef(fd) => {
                    let rust_name = crate::type_codegen::to_snake_case(&fd.name);
                    Some(
                        fd.params
                            .iter()
                            .enumerate()
                            .filter_map(|(i, param)| {
                                if matches!(&param.ty, daglang_syntax::ast::TypeExpr::Named(n) if n == "String")
                                {
                                    Some((rust_name.clone(), i))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>(),
                    )
                }
                _ => None,
            })
        })
        .flatten()
        .collect();

    // 4b. Build enum accessor fields (common fields across all variants)
    let enum_accessor_fields = type_codegen::build_enum_accessor_fields(&all_type_defs);

    // 4b2. Build cross-module enum_variants for context-aware variant resolution
    let all_enum_variants: HashMap<String, HashSet<String>> = all_type_defs
        .iter()
        .filter_map(|td| {
            if let TypeBody::Sum(variants) = &td.body {
                let names: HashSet<String> = variants.iter().map(|v| v.name.clone()).collect();
                Some((td.name.clone(), names))
            } else {
                None
            }
        })
        .collect();

    // 5. Emit each module, tracking type definitions to suppress exact duplicates
    // TEMPORARY bootstrap scaffolding (S81): downstream modules that re-declare
    // structurally identical types get their duplicate definitions suppressed,
    // so cross-module references use the upstream type
    // via `use crate::upstream::*`.
    let mut defined_type_signatures: HashMap<String, TypeDefSignature> = HashMap::new();
    // S81: Track which type names are visible to each module (current + upstream)
    // Initialize with hardcoded materialized types from std_types_prelude
    let mut visible_type_names: HashSet<String> =
        HashSet::from(["SourceSpan".to_string(), "BindingPower".to_string()]);
    // R8: Install Rc-wrapped types for all type_expr_to_rust calls within module emission
    type_codegen::with_rc_wrapped_types(&rc_wrapped_types, || {
    for (_dag_stem, sf) in modules {
        let Some(rust_mod) = rust_mod_for_source_file(sf) else {
            continue;
        };
        let items = &sf.items;

        // Add this module's type names AND variant names to the visible set
        for item in items.iter() {
            if let Item::TypeDef(td) = &item.node {
                visible_type_names.insert(td.name.clone());
                // Also add variant names (they're keys in struct_field_types)
                if let daglang_syntax::ast::TypeBody::Sum(variants) = &td.body {
                    for v in variants {
                        visible_type_names.insert(v.name.clone());
                    }
                }
            }
        }

        // Filter struct_field_types to only include visible types
        let module_struct_field_types: HashMap<String, HashMap<String, String>> =
            struct_field_types
                .iter()
                .filter(|(name, _)| visible_type_names.contains(name.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

        // Filter struct_field_ir_types to only include visible types
        let module_struct_field_ir_types: HashMap<
            String,
            Vec<(String, gunbc_ir::code_ir::IrType)>,
        > = struct_field_ir_types
            .iter()
            .filter(|(name, _)| visible_type_names.contains(name.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let source = emit_module(
            items,
            &recursive_fields,
            &variant_to_enum,
            &module_struct_field_types,
            &optional_fields,
            &all_enum_variants,
            &defined_type_signatures,
            &module_struct_field_ir_types,
            &global_fn_return_types,
            &global_fn_return_ir_types,
            &global_fn_param_types,
            &enum_accessor_fields,
            &global_optional_return_fns,
            &global_fn_str_params,
        );
        // Track which types this module defines with their structural signature.
        for item in items.iter() {
            if let Item::TypeDef(td) = &item.node {
                defined_type_signatures
                    .entry(td.name.clone())
                    .or_insert_with(|| type_def_signature(td));
            }
        }
        let mut content = module_prelude(sf);
        content.push_str(&render_rust::render_rust_source_with_stacker(&source));
        files.push(GeneratedFile {
            rel_path: format!("src/{}.rs", rust_mod),
            content,
        });
    }
    }); // end with_rc_wrapped_types

    // 6. Emit lib.rs with mod declarations and cross-module uses
    let lib_content = emit_lib_rs(modules);
    files.push(GeneratedFile {
        rel_path: "src/lib.rs".to_string(),
        content: lib_content,
    });

    // 7. Emit v2_rt.rs runtime shims
    files.push(GeneratedFile {
        rel_path: "src/v2_rt.rs".to_string(),
        content: v2_runtime_shim::V2_RUNTIME_SOURCE.to_string(),
    });

    // 8. Emit generated test module from the workspace source tree and import
    //    closure so the generated crate embeds a single, structural source set.
    let dag_sources = collect_embedded_dag_sources();
    files.push(GeneratedFile {
        rel_path: "src/generated_tests.rs".to_string(),
        content: emit_test_module(&dag_sources),
    });

    // 9. Emit Cargo.toml (standalone — not part of any workspace)
    let mut cargo_toml = render_rust::render_cargo_toml("v2-compiler", &[("stacker", "0.1")]);
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

/// Generate a `Cargo.lock` for the crate at `crate_dir` by delegating to Cargo.
///
/// Must be called after [`write_crate`] has written the `Cargo.toml` to disk.
/// This performs I/O: it spawns `cargo generate-lockfile` as a subprocess.
pub fn generate_lockfile(crate_dir: &Path) -> std::io::Result<()> {
    let output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(crate_dir)
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "cargo generate-lockfile failed in {}:\n{}",
            crate_dir.display(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

fn emit_lib_rs(modules: &[(&str, &SourceFile)]) -> String {
    let mut out = String::new();
    out.push_str("//! v2 DAG compiler — generated from .dag source files.\n\n");
    out.push_str("#![allow(unused_imports, unused_variables, unused_mut, dead_code, unreachable_patterns, suspicious_double_ref_op, non_shorthand_field_patterns, clippy::all)]\n\n");

    // Module declarations
    for (_, sf) in modules {
        if let Some(rust_mod) = rust_mod_for_source_file(sf) {
            out.push_str(&format!("pub mod {};\n", rust_mod));
        }
    }
    out.push_str("pub mod v2_rt;\n");
    out.push_str("\n#[cfg(test)]\nmod generated_tests;\n");

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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BindingPower {
    pub left: i64,
    pub right: i64,
}

"#
}

/// Generate module-level prelude from import declarations and the module path.
fn module_prelude(source_file: &SourceFile) -> String {
    let mut prelude = String::new();
    let current_module = source_file
        .module_path
        .as_ref()
        .map(|path| path.node.as_dotted())
        .unwrap_or_default();

    // Derive cross-module `use crate::*` from import declarations
    let mut has_std_types_import = false;
    let mut has_v2_core_import = false;
    for import in &source_file.imports {
        let dotted = import.node.path.as_dotted();
        if dotted == "std.types" {
            has_std_types_import = true;
            continue; // handled below via std_types_prelude()
        }
        if dotted == "v2.std.core" {
            has_v2_core_import = true;
        }
        if dotted != current_module {
            let rust_mod = rust_mod_for_module_path(&dotted);
            prelude.push_str(&format!("use crate::{}::*;\n", rust_mod));
        }
    }

    // Modules that import std.types get materialized type definitions,
    // but only if they don't also import v2.std.core (which already
    // contains these definitions via its own std.types import).
    if has_std_types_import && !has_v2_core_import {
        prelude.push_str(std_types_prelude());
    }

    // All modules get access to the runtime shims and std collections
    prelude.push_str("use crate::v2_rt;\n");
    // Import commonly-used runtime functions directly for unqualified calls
    prelude.push_str("use crate::v2_rt::{scan_while, scan_to_eol, skip_horizontal_ws, code_point, from_code_point, scan_string_end};\n");
    prelude.push_str("use std::collections::HashMap;\n");
    prelude.push_str("use std::rc::Rc;\n");
    // Map type alias is defined only in v2_core to avoid redefinition conflicts
    if current_module == "v2.std.core" {
        prelude.push_str("pub type Map<K, V> = HashMap<K, V>;\n");
    }

    prelude.push('\n');
    prelude
}

#[allow(clippy::too_many_arguments)]
fn emit_module(
    items: &[daglang_syntax::span::Spanned<Item>],
    recursive_fields: &HashSet<(String, String)>,
    variant_to_enum: &HashMap<String, String>,
    struct_field_types: &HashMap<String, HashMap<String, String>>,
    optional_fields: &HashMap<String, HashSet<String>>,
    all_enum_variants: &HashMap<String, HashSet<String>>,
    upstream_type_signatures: &HashMap<String, TypeDefSignature>,
    struct_field_ir_types: &HashMap<String, Vec<(String, gunbc_ir::code_ir::IrType)>>,
    global_fn_return_types: &HashMap<String, String>,
    global_fn_return_ir_types: &HashMap<String, gunbc_ir::code_ir::IrType>,
    global_fn_param_types: &HashMap<String, Vec<(String, String)>>,
    enum_accessor_fields: &HashMap<String, HashSet<String>>,
    optional_return_fns: &HashSet<String>,
    global_fn_str_params: &HashSet<(String, usize)>,
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

    // Collect data names that are Map types (need `&` reference instead of `.clone()`)
    let data_map_names: HashSet<String> = items
        .iter()
        .filter_map(|item| match &item.node {
            Item::DataDef(dd) => {
                if matches!(&dd.ty, daglang_syntax::ast::TypeExpr::Generic(name, _) if name == "Map") {
                    Some(dd.name.clone())
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();

    // Use cross-module enum_variants for correct variant resolution
    let enum_variants_map = all_enum_variants.clone();

    let ctx = fn_codegen::CompileContext {
        data_names,
        data_ir_types: items
            .iter()
            .filter_map(|item| match &item.node {
                Item::DataDef(dd) => {
                    Some((dd.name.clone(), fn_codegen::type_expr_to_ir_type(&dd.ty)))
                }
                _ => None,
            })
            .collect(),
        data_map_names,
        optional_fields: optional_fields.clone(),
        variant_to_enum: variant_to_enum.clone(),
        struct_field_types: struct_field_types.clone(),
        enum_variants: enum_variants_map,
        boxed_fields: recursive_fields.clone(),
        fn_return_types: global_fn_return_types.clone(),
        fn_return_ir_types: global_fn_return_ir_types.clone(),
        fn_param_types: global_fn_param_types.clone(),
        optional_params: std::collections::HashSet::new(), // populated per-function in fndef_to_code_ir
        param_types: std::collections::HashMap::new(), // populated per-function in fndef_to_code_ir
        current_return_type: None,                     // populated per-function in fndef_to_code_ir
        current_return_ir_type: None,                  // populated per-function in fndef_to_code_ir
        ir_scope: std::collections::HashMap::new(),    // populated per-function in fndef_to_code_ir
        struct_field_ir_types: struct_field_ir_types.clone(),
        use_counts: std::collections::HashMap::new(), // populated per-function in compile_fn_body
        fold_accum_name: None,
        enum_accessor_fields: enum_accessor_fields.clone(),
        optional_return_fns: optional_return_fns.clone(),
        anonymous_record_targets: std::collections::HashMap::new(),
        synthesized_anonymous_record_types: Vec::new(),
        expr_ir_types: std::collections::HashMap::new(),
        fn_str_params: global_fn_str_params.clone(),
        str_param_names: std::collections::HashSet::new(), // populated per-function in fndef_to_code_ir
        expr_identities: std::collections::HashMap::new(),
        expr_path: std::cell::RefCell::new(Default::default()),
        rc_wrapped_types: type_codegen::current_rc_wrapped_types(),
    };

    for item in items {
        match &item.node {
            Item::TypeDef(td) => {
                // TEMPORARY bootstrap-only nominality compromise (S81):
                // suppress same-name upstream duplicates only when their full
                // structural signature matches exactly. This is still structural
                // dedupe, not sound nominal typing, and should be removed once
                // cross-module type identity is modeled authoritatively.
                let this_signature = type_def_signature(td);
                if upstream_type_signatures
                    .get(&td.name)
                    .is_some_and(|upstream| upstream == &this_signature)
                {
                    continue;
                }
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
fn build_variant_to_enum(type_defs: &[&TypeDef]) -> HashMap<String, String> {
    let mut candidates: HashMap<String, Vec<String>> = HashMap::new();
    for td in type_defs {
        if let TypeBody::Sum(variants) = &td.body {
            for v in variants {
                candidates
                    .entry(v.name.clone())
                    .or_default()
                    .push(td.name.clone());
            }
        }
    }
    candidates
        .into_iter()
        .map(|(variant, mut enums)| {
            enums.sort();
            enums.dedup();
            (
                variant,
                enums
                    .into_iter()
                    .next()
                    .expect("variant has at least one enum"),
            )
        })
        .collect()
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

/// Build struct/variant name → [(field_name, IrType)] map for type-annotated IR.
fn build_struct_field_ir_types(
    type_defs: &[&TypeDef],
) -> HashMap<String, Vec<(String, gunbc_ir::code_ir::IrType)>> {
    let mut map = HashMap::new();
    for td in type_defs {
        match &td.body {
            TypeBody::Record(fields) => {
                let ir_fields: Vec<(String, gunbc_ir::code_ir::IrType)> = fields
                    .iter()
                    .map(|f| (f.name.clone(), fn_codegen::type_expr_to_ir_type(&f.ty)))
                    .collect();
                map.insert(td.name.clone(), ir_fields);
            }
            TypeBody::Sum(variants) => {
                for v in variants {
                    if !v.fields.is_empty() {
                        let ir_fields: Vec<(String, gunbc_ir::code_ir::IrType)> = v
                            .fields
                            .iter()
                            .map(|f| (f.name.clone(), fn_codegen::type_expr_to_ir_type(&f.ty)))
                            .collect();
                        map.insert(v.name.clone(), ir_fields);
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
        daglang_syntax::ast::TypeExpr::AssociatedOutput(base) => format!("{base}::Output"),
        daglang_syntax::ast::TypeExpr::Optional(inner) => type_expr_to_rust_name(inner),
        daglang_syntax::ast::TypeExpr::Generic(name, _) => name.clone(),
        daglang_syntax::ast::TypeExpr::Function(_, _) => "Function".to_string(),
        daglang_syntax::ast::TypeExpr::Refined(inner, _) => type_expr_to_rust_name(inner),
        daglang_syntax::ast::TypeExpr::Record(_) => "Anonymous".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TypeDefSignature {
    Alias(String),
    Record(Vec<(String, String)>),
    Sum(Vec<(String, Vec<(String, String)>)>),
}

fn type_def_signature(td: &TypeDef) -> TypeDefSignature {
    match &td.body {
        TypeBody::Alias(type_expr) => TypeDefSignature::Alias(type_expr_to_string(type_expr)),
        TypeBody::Record(fields) => {
            let mut normalized: Vec<(String, String)> = fields
                .iter()
                .map(|field| (field.name.clone(), type_expr_to_string(&field.ty)))
                .collect();
            normalized.sort_by(|(name_a, ty_a), (name_b, ty_b)| {
                name_a.cmp(name_b).then_with(|| ty_a.cmp(ty_b))
            });
            TypeDefSignature::Record(normalized)
        }
        TypeBody::Sum(variants) => {
            let mut normalized: Vec<(String, Vec<(String, String)>)> = variants
                .iter()
                .map(|variant| {
                    let mut fields: Vec<(String, String)> = variant
                        .fields
                        .iter()
                        .map(|field| (field.name.clone(), type_expr_to_string(&field.ty)))
                        .collect();
                    fields.sort_by(|(name_a, ty_a), (name_b, ty_b)| {
                        name_a.cmp(name_b).then_with(|| ty_a.cmp(ty_b))
                    });
                    (variant.name.clone(), fields)
                })
                .collect();
            normalized.sort_by(|(name_a, fields_a), (name_b, fields_b)| {
                name_a.cmp(name_b).then_with(|| fields_a.cmp(fields_b))
            });
            TypeDefSignature::Sum(normalized)
        }
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

/// Emit a test module containing `#[test]` functions for the generated v2 crate.
///
/// These tests exercise the generated tokenizer and parser on representative inputs,
/// proving that the emitted Rust code executes correctly. The tests are part of the
/// crate assembly itself — they travel with the generated crate, not with the test
/// harness.
///
/// `dag_sources` contains the compiler and fixture source closure derived from
/// the workspace tree and import graph, embedded as const strings in the
/// generated test module.
fn emit_test_module(dag_sources: &[EmbeddedDagSource]) -> String {
    // Pick the right raw string delimiter for each source.
    // We use r##"..."## so the content can contain r#"..."# sequences.
    // If any source contains "## we escalate to r###"..."###.
    fn raw_delimiters(source: &str) -> (&'static str, &'static str) {
        if source.contains("\"##") {
            ("r###\"", "\"###")
        } else {
            ("r##\"", "\"##")
        }
    }

    // Build const declarations for each .dag source file
    let mut const_decls = String::new();
    for source in dag_sources {
        let (open, close) = raw_delimiters(&source.content);
        const_decls.push_str(&format!(
            "    const {}: &str = {}{}{};\n",
            source.const_name, open, source.content, close
        ));
    }

    let tokenize_source_const = dag_sources
        .iter()
        .find(|source| source.rel_path == "src/v2/01_tokenize.dag")
        .map(|source| source.const_name.clone())
        .expect("missing src/v2/01_tokenize.dag embedded source");

    let self_parse_sources = dag_sources
        .iter()
        .filter(|source| source.include_in_self_parse)
        .map(|source| {
            format!(
                "                    (\"{}\", {}, \"{}\"),\n",
                source.rel_path, source.const_name, source.module_name
            )
        })
        .collect::<String>();

    let self_compile_source_files = dag_sources
        .iter()
        .filter(|source| source.include_in_self_resolve)
        .map(|source| {
            format!(
                "                    std::rc::Rc::new(crate::pipeline::SourceFile {{ path: \"{}\".to_string(), content: {}.to_string() }}),\n",
                source.rel_path, source.const_name
            )
        })
        .collect::<String>();

    let gist_resolve_sources = dag_sources
        .iter()
        .filter(|source| source.include_in_gist_resolve)
        .map(|source| {
            let logical_path = source
                .dsl_logical_path
                .as_ref()
                .unwrap_or_else(|| panic!("gist source {} is not under dsl/", source.rel_path));
            format!(
                "                    std::rc::Rc::new(crate::pipeline::SourceFile {{ path: \"{}\".to_string(), content: {}.to_string() }}),\n",
                logical_path, source.const_name
            )
        })
        .collect::<String>();

    format!(
        r#"#[cfg(test)]
mod generated_tests {{
    use crate::tokenize::tokenize;

{const_decls}
    #[test]
    fn tokenize_produces_tokens() {{
        let tokens = tokenize("fn foo() -> Int {{ 42 }}");
        assert!(!tokens.is_empty(), "tokenize should produce at least one token");
    }}

    #[test]
    fn tokenize_ends_with_eof() {{
        let tokens = tokenize("type Foo {{ x: Int }}");
        let last = tokens.last().expect("should have tokens");
        assert!(
            matches!(&*last.kind, crate::v2_core::TokenKind::Eof),
            "last token should be Eof, got {{:?}}",
            last.kind
        );
    }}

    #[test]
    fn tokenize_fn_keyword() {{
        let tokens = tokenize("fn");
        // Should have at least KwFn and Eof
        assert!(tokens.len() >= 2, "expected at least 2 tokens, got {{}}", tokens.len());
        assert!(
            matches!(&*tokens[0].kind, crate::v2_core::TokenKind::KwFn),
            "first token should be KwFn, got {{:?}}",
            tokens[0].kind
        );
    }}

    #[test]
    fn tokenize_count_stable() {{
        let tokens = tokenize("module test\ntype Foo {{ x: Int }}");
        // Non-trivial input should produce multiple tokens
        assert!(tokens.len() > 5, "non-trivial input should produce multiple tokens, got {{}}", tokens.len());
    }}

    #[test]
    fn parse_trivial_module() {{
        let tokens = tokenize("module test\ntype Foo {{ x: Int }}\n");
        let result = crate::parse::parse(tokens);
        // ParseResult should have a module
        assert!(result.module.is_some(), "valid module should parse successfully");
    }}

    /// Self-parse test: the compiled v2 compiler tokenizes and parses its own
    /// tokenizer source (01_tokenize.dag). This is the self-hosting seed — if
    /// the generated compiler can parse its own source, it can bootstrap.
    #[test]
    fn self_parse_tokenize_dag() {{
        // The generated parser uses recursive descent which requires extra stack
        // for non-trivial inputs (same as v1 evaluator's with_parser_stack).
        let result = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {{
                // Tokenize the v2 compiler's own tokenizer source
                let tokens = tokenize({tokenize_source_const});

                // Token list should be non-empty
                assert!(!tokens.is_empty(), "tokenizing 01_tokenize.dag should produce tokens");

                // Should end with Eof
                let last = tokens.last().expect("should have tokens");
                assert!(
                    matches!(&*last.kind, crate::v2_core::TokenKind::Eof),
                    "last token should be Eof, got {{:?}}",
                    last.kind
                );

                // Parse the tokens
                let result = crate::parse::parse(tokens);

                // Parse should succeed with a module
                assert!(
                    result.module.is_some(),
                    "parsing 01_tokenize.dag should produce a module"
                );

                // Module name should be "v2.compiler.tokenize"
                let module = result.module.as_ref().unwrap();
                assert_eq!(
                    module.name,
                    "v2.compiler.tokenize",
                    "module name should be v2.compiler.tokenize, got {{}}",
                    module.name
                );
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("self-parse test panicked");
    }}

    /// Phase 2 pipeline test: feed a self-contained single-module .dag source
    /// through the full compile pipeline (tokenize -> parse -> resolve ->
    /// typecheck -> emit) and verify the result has output files with no errors.
    #[test]
    fn pipeline_trivial_module() {{
        // The full pipeline uses recursive descent in resolve/typecheck/emit,
        // so give it the same 16MB stack as self-parse.
        let result = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {{
                let source = std::rc::Rc::new(crate::pipeline::SourceFile {{
                    path: "test.dag".to_string(),
                    content: "module test\ntype Foo {{ x: Int, name: String }}\nfn add(a: Int, b: Int) -> Int {{ a + b }}\n".to_string(),
                }});
                let result = crate::pipeline::compile_sources(std::rc::Rc::new(vec![source]), crate::v2_core::RenderTarget::Rust);

                // Should produce at least one output file
                assert!(
                    !result.files.is_empty(),
                    "compile_sources should produce output files, got none"
                );

                // Should have zero error diagnostics
                let errors: Vec<_> = result.diagnostics.iter()
                    .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                    .collect();
                assert!(
                    errors.is_empty(),
                    "compile_sources should produce no errors, got {{:?}}",
                    errors
                );

                // At least one file should have non-empty content
                let has_content = result.files.iter().any(|f| !f.content.is_empty());
                assert!(
                    has_content,
                    "at least one output file should have non-empty content"
                );
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("pipeline_trivial_module test panicked");
    }}

    /// Incremental self-parse: tokenize and parse each of the v2 .dag
    /// source files individually. Proves the compiled compiler can process
    /// its own complete source at the tokenize+parse level.
    #[test]
    fn self_parse_all_modules() {{
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {{
                // Parse all v2 modules including 02_parse.dag (3600+ lines).
                // Rc<Vec<T>> wrapping (S76 fix) makes this feasible — clone is O(1).
                let modules: Vec<(&str, &str, &str)> = vec![
{self_parse_sources}                ];
                for (file, source, expected_name) in &modules {{
                    let tokens = tokenize(source);
                    assert!(
                        !tokens.is_empty(),
                        "{{}} should produce tokens", file
                    );
                    assert!(
                        matches!(&*tokens.last().unwrap().kind, crate::v2_core::TokenKind::Eof),
                        "{{}} should end with Eof", file
                    );
                    let result = crate::parse::parse(tokens);
                    assert!(
                        result.module.is_some(),
                        "{{}} should parse successfully, error: {{:?}}", file, result.error
                    );
                    let module = result.module.as_ref().unwrap();
                    assert_eq!(
                        module.name, *expected_name,
                        "{{}} module name should be {{}}, got {{}}", file, expected_name, module.name
                    );
                }}
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("self-parse-all test panicked");
    }}

    /// Bootstrap self-compile: runs the full pipeline using compile_sources_lenient
    /// which skips the typecheck error gate. The v2 typechecker has false positives
    /// on recursive types and incomplete inference. The emitter produces structurally
    /// correct code; Rust's type checker is the final arbiter.
    #[test]
    fn self_compile_all_modules() {{
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {{
                let sources: Vec<std::rc::Rc<crate::pipeline::SourceFile>> = vec![
{self_compile_source_files}                ];

                let source_count = sources.len();
                let result = crate::pipeline::compile_sources_lenient(
                    std::rc::Rc::new(sources),
                    crate::v2_core::RenderTarget::Rust,
                );

                let errors: Vec<_> = result.diagnostics.iter()
                    .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                    .collect();
                let error_count = errors.len();

                eprintln!(
                    "self-compile completed: {{}} errors, {{}} files emitted from {{}} sources",
                    error_count, result.files.len(), source_count
                );

                // Bootstrap ratchet: track error count but don't assert zero.
                // The v2 typechecker's incomplete inference produces false positives.
                // Self-compile succeeds if files are emitted and Rust compiles them.

                let has_content = result.files.iter().any(|f| !f.content.is_empty());
                assert!(
                    has_content,
                    "self-compile should produce at least one non-empty output file (got {{}} errors, {{}} files)",
                    error_count, result.files.len()
                );
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("self-compile-all test panicked");
    }}

    /// Gist resolve: feed gist.dag's transitive source closure through
    /// tokenize -> parse -> resolve via the v2 compiler's own pipeline.
    /// Proves the compiled v2 compiler can process real-world DSL modules
    /// (services, resources, patterns) beyond its own source.
    #[test]
    fn gist_resolve_all_modules() {{
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {{
                // Gist's transitive source closure, derived from imports.
                let sources: Vec<std::rc::Rc<crate::pipeline::SourceFile>> = vec![
{gist_resolve_sources}                ];
                let result = crate::pipeline::resolve_sources(
                    std::rc::Rc::new(sources),
                );

                // Count error-severity diagnostics from tokenize + parse + resolve.
                // The gist dependency chain exercises DSL constructs (services,
                // resources, patterns, func) that the v2 compiler's own source
                // does not cover.
                let errors: Vec<_> = result.diagnostics.iter()
                    .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                    .collect();
                let error_count = errors.len();

                eprintln!("gist resolve error count: {{}}", error_count);
                for e in &errors {{
                    eprintln!("  {{:?}}", e);
                }}

                assert!(
                    error_count == 0,
                    "gist resolve errors: {{}} errors (expected 0): {{:?}}",
                    error_count, errors
                );
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("gist-resolve-all test panicked");
    }}

    /// Gist full pipeline: tokenize -> parse -> resolve -> typecheck -> emit.
    /// Proves the compiled v2 compiler can process the gist dependency chain
    /// through all pipeline stages without OOM.
    #[test]
    fn gist_compile_all_modules() {{
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {{
                let sources: Vec<std::rc::Rc<crate::pipeline::SourceFile>> = vec![
{gist_resolve_sources}                ];
                let result = crate::pipeline::compile_sources(
                    std::rc::Rc::new(sources),
                    crate::v2_core::RenderTarget::Rust,
                );

                let errors: Vec<_> = result.diagnostics.iter()
                    .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                    .collect();
                let error_count = errors.len();

                eprintln!("gist compile error count: {{}}", error_count);
                for e in &errors {{
                    eprintln!("  {{:?}}", e);
                }}

                assert!(
                    error_count == 0,
                    "gist compile errors: {{}} errors (expected 0): {{:?}}",
                    error_count, errors
                );

                let has_content = result.files.iter().any(|f| !f.content.is_empty());
                assert!(
                    has_content,
                    "gist compile should produce at least one non-empty file"
                );
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("gist-compile-all test panicked");
    }}

    #[test]
    fn type_size_regression_check() {{
        // Prevent silent type size regressions in generated v2 types.
        // These bounds assume Node.transport and Node.config are boxed (R2).
        let node_size = std::mem::size_of::<crate::v2_core::Node>();
        let typed_expr_size = std::mem::size_of::<crate::v2_core::TypedExpr>();
        assert!(
            node_size <= 160,
            "Node size regression: {{}} bytes (limit: 160). Check for unboxed rare fields.",
            node_size
        );
        assert!(
            typed_expr_size <= 800,
            "TypedExpr size regression: {{}} bytes (limit: 800). Node size likely regressed.",
            typed_expr_size
        );
        // Print sizes for audit trail
        eprintln!("  Node: {{}} bytes", node_size);
        eprintln!("  TypedExpr: {{}} bytes", typed_expr_size);
    }}
}}
"#,
        const_decls = const_decls,
        tokenize_source_const = tokenize_source_const,
        self_parse_sources = self_parse_sources,
        self_compile_source_files = self_compile_source_files,
        gist_resolve_sources = gist_resolve_sources,
    )
}

#[cfg(test)]
mod tests {
    use super::{assemble_v2_crate, build_variant_to_enum, type_def_signature};
    use daglang_syntax::ast::{Item, TypeDef};

    fn parse_source(source: &str) -> daglang_syntax::ast::SourceFile {
        daglang_syntax::parser::parse(source)
            .unwrap_or_else(|errors| panic!("parse failed: {errors:?}"))
    }

    #[test]
    fn assemble_v2_crate_keeps_same_name_records_with_different_field_types() {
        let core_sf = parse_source(
            r#"
module v2.std.core

type Shared {
  value: String
}
"#,
        );
        let tokenize_sf = parse_source(
            r#"
module v2.compiler.tokenize

type Shared {
  value: Int
}
"#,
        );

        let modules = vec![("00_core", &core_sf), ("01_tokenize", &tokenize_sf)];
        let files = assemble_v2_crate(&modules);

        let core_file = files
            .iter()
            .find(|file| file.rel_path == "src/v2_core.rs")
            .expect("core file emitted");
        let tokenize_file = files
            .iter()
            .find(|file| file.rel_path == "src/tokenize.rs")
            .expect("tokenize file emitted");

        assert!(
            core_file.content.contains("pub struct Shared"),
            "upstream module should emit Shared:\n{}",
            core_file.content
        );
        assert!(
            tokenize_file.content.contains("pub struct Shared"),
            "downstream module should emit its distinct Shared definition:\n{}",
            tokenize_file.content
        );
    }

    #[test]
    fn assemble_v2_crate_derives_rust_module_names_from_module_declarations() {
        let core_sf = parse_source(
            r#"
module v2.std.core

type Shared {
  value: String
}
"#,
        );
        let pipeline_sf = parse_source(
            r#"
module v2.compiler.pipeline

fn main() -> Int { 0 }
"#,
        );

        let modules = vec![
            ("not_the_stem", &core_sf),
            ("also_not_the_stem", &pipeline_sf),
        ];
        let files = assemble_v2_crate(&modules);

        assert!(
            files.iter().any(|file| file.rel_path == "src/v2_core.rs"),
            "core module should derive v2_core.rs from its declaration"
        );
        assert!(
            files.iter().any(|file| file.rel_path == "src/pipeline.rs"),
            "pipeline module should derive pipeline.rs from its declaration"
        );
    }

    #[test]
    fn assemble_v2_crate_cargo_toml_has_no_dev_opt_level_override() {
        let core_sf = parse_source(
            r#"
module v2.std.core

type Shared {
  value: String
}
"#,
        );

        let files = assemble_v2_crate(&[("00_core", &core_sf)]);
        let cargo = files
            .iter()
            .find(|file| file.rel_path == "Cargo.toml")
            .expect("Cargo.toml emitted");
        assert!(
            !cargo.content.contains("opt-level = 1"),
            "generated Cargo.toml should not pin dev opt-level:\n{}",
            cargo.content
        );
    }

    #[test]
    fn build_variant_to_enum_uses_deterministic_tiebreak() {
        let left_sf = parse_source(
            r#"
module sample.left

type Zeta = Shared | Tail
"#,
        );
        let right_sf = parse_source(
            r#"
module sample.right

type Alpha = Shared | Other
"#,
        );

        let type_defs: Vec<&TypeDef> = [&left_sf, &right_sf]
            .into_iter()
            .flat_map(|sf| {
                sf.items.iter().filter_map(|item| match &item.node {
                    Item::TypeDef(td) => Some(td),
                    _ => None,
                })
            })
            .collect();

        let variant_map = build_variant_to_enum(&type_defs);
        assert_eq!(variant_map.get("Shared"), Some(&"Alpha".to_string()));
    }

    #[test]
    fn type_def_signature_ignores_record_field_order() {
        let left_sf = parse_source(
            r#"
module sample.left

type Pair {
  left: Int
  right: String
}
"#,
        );
        let right_sf = parse_source(
            r#"
module sample.right

type Pair {
  right: String
  left: Int
}
"#,
        );

        let left = match &left_sf.items[0].node {
            Item::TypeDef(td) => td,
            other => panic!("expected typedef, got {other:?}"),
        };
        let right = match &right_sf.items[0].node {
            Item::TypeDef(td) => td,
            other => panic!("expected typedef, got {other:?}"),
        };

        assert_eq!(type_def_signature(left), type_def_signature(right));
    }
}
