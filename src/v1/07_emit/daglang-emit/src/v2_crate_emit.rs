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
    // extdeps.languages.rust.emit → rust_emit (language + leaf to avoid collision)
    if let Some(rest) = path.strip_prefix("extdeps.languages.") {
        let parts: Vec<&str> = rest.split('.').collect();
        if parts.len() >= 2 {
            return format!("{}_{}", parts[0], parts[parts.len() - 1]).replace('-', "_");
        }
    }
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

    // P1a: Build cross-module call graph from DSL AST for SCC classification.
    // This determines which functions need stacker wrapping (recursive/TCO only).
    let call_graph: HashMap<String, HashSet<String>> = modules
        .iter()
        .flat_map(|(_, sf)| {
            sf.items.iter().filter_map(|item| match &item.node {
                Item::FnDef(fd) => {
                    let rust_name = type_codegen::to_snake_case(&fd.name);
                    let callees = fn_codegen::collect_fn_callees(fd);
                    Some((rust_name, callees))
                }
                _ => None,
            })
        })
        .collect();

    // Guard: assert no cross-module function name collisions in the flat call graph.
    // Full module-qualification is deferred; this catches collisions early.
    {
        let mut seen: HashMap<String, &str> = HashMap::new();
        for (dag_stem, sf) in modules {
            for item in &sf.items {
                if let Item::FnDef(fd) = &item.node {
                    let rust_name = type_codegen::to_snake_case(&fd.name);
                    if let Some(prev) = seen.get(&rust_name) {
                        panic!(
                            "function name collision: '{}' defined in both '{}' and '{}'; \
                             call graph requires unique names across modules",
                            rust_name, prev, dag_stem
                        );
                    }
                    seen.insert(rust_name, dag_stem);
                }
            }
        }
    }

    // P1a: Compute SCC-based function classification. TCO set starts empty;
    // the classification is conservative for now, so all recursive functions
    // stay in the stacker set. Once TCO-classified functions are threaded
    // through here, loop-lowered TCO bodies should skip stacker.
    let function_classes = fn_codegen::classify_functions(&call_graph, &HashSet::new());
    let needs_stacker: HashSet<String> = function_classes
        .iter()
        .filter(|(_, cls)| cls.needs_stacker())
        .map(|(name, _)| name.clone())
        .collect();

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

    // SG-10: Register v2 runtime functions that accept &str (impl AsRef<str>)
    // so call sites strip .to_string() from &str arguments. Without this,
    // passing source.to_string() to scan_while/skip_horizontal_ws copies
    // the entire source string per call — O(N) per token.
    let mut global_fn_str_params = global_fn_str_params;
    for (fn_name, param_idx) in [
        ("scan_while", 0usize),
        ("skip_horizontal_ws", 0),
        ("scan_to_eol", 0),
        ("scan_ident_rest", 0),
    ] {
        global_fn_str_params.insert((fn_name.to_string(), param_idx));
    }

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

    // Cross-module data names: all data declarations across all modules.
    // Needed so that imported data references (e.g., `rust_type_map` imported
    // from extdeps) are correctly emitted as SCREAMING_SNAKE_CASE statics.
    let global_data_names: HashSet<String> = modules
        .iter()
        .flat_map(|(_, sf)| {
            sf.items.iter().filter_map(|item| match &item.node {
                Item::DataDef(dd) => Some(dd.name.clone()),
                _ => None,
            })
        })
        .collect();
    let global_data_map_names: HashSet<String> = modules
        .iter()
        .flat_map(|(_, sf)| {
            sf.items.iter().filter_map(|item| match &item.node {
                Item::DataDef(dd) => {
                    if matches!(&dd.ty, daglang_syntax::ast::TypeExpr::Generic(name, _) if name == "Map")
                    {
                        Some(dd.name.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            })
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
            &global_data_names,
            &global_data_map_names,
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
        content.push_str(&render_rust::render_rust_source_selective_stacker(
            &source,
            &needs_stacker,
        ));
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

    // 9. Emit main.rs with compile subcommand for bootstrap (A5)
    files.push(GeneratedFile {
        rel_path: "src/main.rs".to_string(),
        content: emit_v2_main_rs(),
    });

    // 10. Emit Cargo.toml (standalone — not part of any workspace)
    let mut cargo_toml = render_rust::render_cargo_toml("v2-compiler", &[("stacker", "0.1")]);
    cargo_toml.push_str("clap = { version = \"4\", features = [\"derive\"] }\n");
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

/// A5: Generate main.rs with a `compile` subcommand for bootstrap.
///
/// The compile subcommand reads .dag files from a source directory, runs them
/// through the pipeline, and writes the compiled Rust output to a target
/// directory. File I/O stays in main.rs (Rust) — the pipeline remains pure.
fn emit_v2_main_rs() -> String {
    r#"//! Bootstrap CLI for the v2 DAG compiler.
//!
//! Generated by the v1 emitter. Provides a `compile` subcommand that reads
//! .dag source files and emits compiled Rust.

#![allow(unused_imports, dead_code)]

use std::rc::Rc;
use clap::{Parser, Subcommand};
use v2_compiler::{pipeline, v2_core};

#[derive(Parser)]
#[command(name = "v2-compiler", about = "v2 DAG compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile .dag source files to Rust
    Compile {
        #[arg(long)]
        source_dir: String,
        #[arg(long)]
        output_dir: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Compile { source_dir, output_dir } => {
            // Read all .dag files from source directory
            let mut sources: Vec<Rc<pipeline::SourceFile>> = Vec::new();
            let mut entries: Vec<_> = std::fs::read_dir(&source_dir)
                .unwrap_or_else(|e| panic!("failed to read source dir {}: {}", source_dir, e))
                .filter_map(|e| e.ok())
                .collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let path = entry.path();
                if path.extension().map(|e| e == "dag").unwrap_or(false) {
                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("failed to read {:?}: {}", path, e));
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    sources.push(Rc::new(pipeline::SourceFile {
                        path: filename,
                        content,
                    }));
                }
            }

            eprintln!("compiling {} .dag files from {}", sources.len(), source_dir);

            // Run the pipeline. Stage0 (this code, v1-emitted) wraps the sources
            // list in Rc::new() because v1 renders List<T> as Rc<Vec<Rc<T>>>.
            // Stage1 (v2-emitted) passes bare Vec<Rc<T>> because v2 renders
            // List<T> without the outer Rc. Each stage is internally consistent
            // with its own emitter's type representation.
            let result = pipeline::compile_sources(
                Rc::new(sources),
                v2_core::RenderTarget::Rust,
            );

            // Write output files
            std::fs::create_dir_all(format!("{}/src", output_dir))
                .unwrap_or_else(|e| panic!("failed to create output dir: {}", e));
            for file in result.files.iter() {
                let out_path = format!("{}/{}", output_dir, file.path);
                if let Some(parent) = std::path::Path::new(&out_path).parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                std::fs::write(&out_path, &*file.content)
                    .unwrap_or_else(|e| panic!("failed to write {}: {}", file.path, e));
            }

            // Report
            eprintln!("compiled: {} files emitted, {} diagnostics",
                result.files.len(), result.diagnostics.len());
            for (i, d) in result.diagnostics.iter().take(20).enumerate() {
                eprintln!("  [{}]: {:?}", i, d);
            }
            if result.files.is_empty() {
                eprintln!("error: no files emitted");
                std::process::exit(1);
            }
        }
    }
}
"#
    .to_string()
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
            let current_rust_mod = rust_mod_for_module_path(&current_module);
            if rust_mod != current_rust_mod {
                prelude.push_str(&format!("use crate::{}::*;\n", rust_mod));
            }
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
    global_data_names: &HashSet<String>,
    global_data_map_names: &HashSet<String>,
) -> code_ir::SourceFile {
    let mut ir_items: Vec<code_ir::Item> = Vec::new();

    // Collect data names for compile context — union of local and cross-module.
    // Cross-module data is needed so that imported data references (e.g., from
    // extdeps) are correctly emitted as SCREAMING_SNAKE_CASE statics.
    let mut data_names: HashSet<String> = global_data_names.clone();
    data_names.extend(items.iter().filter_map(|item| match &item.node {
        Item::DataDef(dd) => Some(dd.name.clone()),
        _ => None,
    }));

    // Collect data names that are Map types (need `&` reference instead of `.clone()`)
    let mut data_map_names: HashSet<String> = global_data_map_names.clone();
    data_map_names.extend(items.iter().filter_map(|item| match &item.node {
        Item::DataDef(dd) => {
            if matches!(&dd.ty, daglang_syntax::ast::TypeExpr::Generic(name, _) if name == "Map") {
                Some(dd.name.clone())
            } else {
                None
            }
        }
        _ => None,
    }));

    // Use cross-module enum_variants for correct variant resolution
    let enum_variants_map = all_enum_variants.clone();

    let mut ctx = fn_codegen::CompileContext {
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
        fn_param_types: std::collections::HashMap::new(),
        fn_param_name_indices: std::collections::HashMap::new(),
        optional_params: std::collections::HashSet::new(), // populated per-function in fndef_to_code_ir
        param_types: std::collections::HashMap::new(), // populated per-function in fndef_to_code_ir
        current_return_type: None,                     // populated per-function in fndef_to_code_ir
        current_return_ir_type: None,                  // populated per-function in fndef_to_code_ir
        ir_scope: std::collections::HashMap::new(),    // populated per-function in fndef_to_code_ir
        struct_field_ir_types: struct_field_ir_types.clone(),
        use_counts: std::collections::HashMap::new(), // populated per-function in compile_fn_body
        fold_accum_name: None,
        fold_accum_fresh_name: None,
        fold_accum_is_rc: false,
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
        match_bound_vars: std::collections::HashSet::new(),
    };
    ctx.set_fn_param_types(global_fn_param_types.clone());

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

    /// Bootstrap self-compile: runs the full pipeline using compile_sources
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
                let result = crate::pipeline::compile_sources(
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
                for (i, e) in errors.iter().enumerate() {{
                    eprintln!("  error[{{}}]: {{}} (module: {{:?}})", i, e.message, e.module_name);
                }}

                // Bootstrap ratchet: track error count but don't assert zero.
                // The v2 typechecker's incomplete inference produces false positives.
                // Self-compile succeeds if files are emitted and Rust compiles them.

                // Output-shape assertions
                assert!(result.files.len() >= 9,
                    "self-compile should produce at least 9 files, got {{}}",
                    result.files.len());

                // All files must have content
                assert!(result.files.iter().all(|f| !f.content.is_empty()),
                    "all self-compiled output files must have non-empty content");

                // Source count floor
                assert!(source_count >= 13,
                    "self-compile should process at least 13 sources, got {{}}",
                    source_count);

                // Diagnostic error ratchet (tracked, not yet tight)
                const SELF_COMPILE_ERROR_RATCHET: usize = 2700;
                assert!(error_count <= SELF_COMPILE_ERROR_RATCHET,
                    "self-compile error count regression: {{}} > {{}} ratchet",
                    error_count, SELF_COMPILE_ERROR_RATCHET);
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("self-compile-all test panicked");
    }}

    /// Bootstrap self-compile cargo check: runs the full pipeline, writes
    /// emitted files to a temp dir, and runs `cargo check` on them.
    #[test]
    #[ignore]
    fn self_compile_cargo_check() {{
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {{
                let sources: Vec<std::rc::Rc<crate::pipeline::SourceFile>> = vec![
{self_compile_source_files}                ];

                let result = crate::pipeline::compile_sources(
                    std::rc::Rc::new(sources),
                    crate::v2_core::RenderTarget::Rust,
                );

                assert!(!result.files.is_empty(), "self-compile produced no files");

                // Write emitted files to a temp directory
                let tmp_dir = std::env::temp_dir().join("v2-self-compile-check");
                let _ = std::fs::remove_dir_all(&tmp_dir);
                std::fs::create_dir_all(tmp_dir.join("src"))
                    .expect("failed to create temp src dir");

                for file in result.files.iter() {{
                    let dest = tmp_dir.join(&file.path);
                    if let Some(parent) = dest.parent() {{
                        std::fs::create_dir_all(parent).expect("failed to create parent dir");
                    }}
                    std::fs::write(&dest, &file.content)
                        .expect(&format!("failed to write {{}}", file.path));
                }}

                // Write a minimal Cargo.toml if not emitted
                let cargo_toml = tmp_dir.join("Cargo.toml");
                if !cargo_toml.exists() {{
                    std::fs::write(&cargo_toml,
                        "[package]\nname = \"v2-self-compile-check\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
                    ).expect("failed to write Cargo.toml");
                }}

                eprintln!("self-compile-cargo-check: wrote {{}} files to {{}}",
                    result.files.len(), tmp_dir.display());

                // Run cargo check
                let output = std::process::Command::new("cargo")
                    .arg("check")
                    .current_dir(&tmp_dir)
                    .output()
                    .expect("failed to run cargo check");

                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("cargo check stderr:\n{{}}", stderr);

                if !output.status.success() {{
                    panic!(
                        "cargo check failed on self-compiled output (dir: {{}}):\n{{}}",
                        tmp_dir.display(),
                        stderr
                    );
                }}

                let _ = std::fs::remove_dir_all(&tmp_dir);
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("self-compile-cargo-check test panicked");
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
        let expr_size = std::mem::size_of::<crate::v2_core::Expr>();
        assert!(
            node_size <= 176,
            "Node size regression: {{}} bytes (limit: 176). Check for unboxed rare fields.",
            node_size
        );
        assert!(
            expr_size <= 800,
            "Expr size regression: {{}} bytes (limit: 800). Node size likely regressed.",
            expr_size
        );
        // Print sizes for audit trail
        eprintln!("  Node: {{}} bytes", node_size);
        eprintln!("  Expr: {{}} bytes", expr_size);
    }}

    /// Profile the gist pipeline by stage: tokenize, parse, resolve.
    /// Reports per-file and per-stage wall-clock times.
    #[test]
    #[ignore]
    fn profile_gist_pipeline() {{
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {{
                use std::time::Instant;

                let sources: Vec<std::rc::Rc<crate::pipeline::SourceFile>> = vec![
{gist_resolve_sources}                ];

                eprintln!("\n=== GIST PIPELINE PROFILE ({{}} sources) ===\n", sources.len());

                // Stage 1: Tokenize each source individually
                let t_stage = Instant::now();
                let mut token_lists = Vec::new();
                for source in &sources {{
                    let t = Instant::now();
                    let tokens = crate::tokenize::tokenize(&source.content);
                    let elapsed = t.elapsed();
                    eprintln!("  tokenize {{:>40}}: {{:>8.2?}}  ({{:>5}} tokens, {{:>5}} chars)",
                        source.path, elapsed, tokens.len(), source.content.len());
                    token_lists.push(tokens);
                }}
                let tokenize_total = t_stage.elapsed();
                eprintln!("  TOKENIZE TOTAL: {{:?}}\n", tokenize_total);

                // Stage 2: Parse each token stream
                let t_stage = Instant::now();
                let mut modules = Vec::new();
                for (i, tokens) in token_lists.iter().enumerate() {{
                    let t = Instant::now();
                    let result = crate::parse::parse(tokens.clone());
                    let elapsed = t.elapsed();
                    let ok = result.module.is_some();
                    eprintln!("  parse   {{:>40}}: {{:>8.2?}}  (ok={{}})", sources[i].path, elapsed, ok);
                    if let Some(m) = result.module.clone() {{
                        modules.push(m);
                    }}
                }}
                let parse_total = t_stage.elapsed();
                eprintln!("  PARSE TOTAL:    {{:?}}\n", parse_total);

                // Stage 3: Resolve module graph
                let t_stage = Instant::now();
                let graph = crate::resolve::resolve_modules(std::rc::Rc::new(modules));
                let resolve_total = t_stage.elapsed();
                let errors: Vec<_> = graph.diagnostics.iter()
                    .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                    .collect();
                eprintln!("  RESOLVE TOTAL:  {{:?}}  ({{}} errors)\n", resolve_total, errors.len());

                eprintln!("=== SUMMARY ===");
                eprintln!("  Tokenize: {{:?}}", tokenize_total);
                eprintln!("  Parse:    {{:?}}", parse_total);
                eprintln!("  Resolve:  {{:?}}", resolve_total);
                eprintln!("  Total:    {{:?}}", tokenize_total + parse_total + resolve_total);
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("profile test panicked");
    }}

    /// Return current process RSS in bytes (macOS via mach_task_basic_info).
    /// Returns 0 on non-macOS platforms.
    fn get_rss_bytes() -> u64 {{
        #[cfg(target_os = "macos")]
        {{
            #[allow(non_camel_case_types)]
            #[repr(C)]
            struct mach_task_basic_info {{
                virtual_size: u64,
                resident_size: u64,
                resident_size_max: u64,
                user_time: [u64; 2],
                system_time: [u64; 2],
                policy: i32,
                suspend_count: i32,
            }}
            extern "C" {{
                fn mach_task_self() -> u32;
                fn task_info(
                    target_task: u32,
                    flavor: u32,
                    task_info_out: *mut mach_task_basic_info,
                    task_info_count: *mut u32,
                ) -> i32;
            }}
            const MACH_TASK_BASIC_INFO: u32 = 20;
            const MACH_TASK_BASIC_INFO_COUNT: u32 =
                (std::mem::size_of::<mach_task_basic_info>() / std::mem::size_of::<u32>()) as u32;
            let mut info: mach_task_basic_info = unsafe {{ std::mem::zeroed() }};
            let mut count = MACH_TASK_BASIC_INFO_COUNT;
            let kr = unsafe {{
                task_info(mach_task_self(), MACH_TASK_BASIC_INFO, &mut info, &mut count)
            }};
            if kr == 0 {{ info.resident_size }} else {{ 0 }}
        }}
        #[cfg(not(target_os = "macos"))]
        {{ 0 }}
    }}

    /// Format a byte count as a human-readable string (KB / MB / GB).
    fn format_bytes(bytes: u64) -> String {{
        if bytes >= 1_073_741_824 {{
            format!("{{:.1}} GB", bytes as f64 / 1_073_741_824.0)
        }} else if bytes >= 1_048_576 {{
            format!("{{:.1}} MB", bytes as f64 / 1_048_576.0)
        }} else if bytes >= 1024 {{
            format!("{{:.1}} KB", bytes as f64 / 1024.0)
        }} else {{
            format!("{{}} B", bytes)
        }}
    }}

    /// Profile the self-compile pipeline by stage with RSS checkpoints.
    /// Reports per-file and per-stage wall-clock times plus memory usage.
    #[test]
    #[ignore]
    fn profile_self_compile() {{
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {{
                use std::time::Instant;

                let sources: Vec<std::rc::Rc<crate::pipeline::SourceFile>> = vec![
{self_compile_source_files}                ];

                let source_count = sources.len();
                let rss_start = get_rss_bytes();
                eprintln!("\n=== SELF-COMPILE PIPELINE PROFILE ({{}} sources) ===", source_count);
                eprintln!("  RSS at start: {{}}\n", format_bytes(rss_start));

                // Phase 1: Tokenize each source individually
                let t_stage = Instant::now();
                let mut token_lists = Vec::new();
                let mut phase1_diags = 0usize;
                for source in &sources {{
                    let t = Instant::now();
                    let tokens = crate::tokenize::tokenize(&source.content);
                    let elapsed = t.elapsed();
                    eprintln!("  tokenize {{:>40}}: {{:>8.2?}}  ({{:>5}} tokens, {{:>6}} chars)",
                        source.path, elapsed, tokens.len(), source.content.len());
                    token_lists.push(tokens);
                }}
                let tokenize_total = t_stage.elapsed();
                let rss_after_tokenize = get_rss_bytes();
                eprintln!("  TOKENIZE TOTAL: {{:?}}  | RSS: {{}}  | diags: {{}}\n",
                    tokenize_total, format_bytes(rss_after_tokenize), phase1_diags);

                // Phase 2: Parse each token stream
                let t_stage = Instant::now();
                let mut modules = Vec::new();
                let mut phase2_diags = 0usize;
                for (i, tokens) in token_lists.iter().enumerate() {{
                    let t = Instant::now();
                    let result = crate::parse::parse(tokens.clone());
                    let elapsed = t.elapsed();
                    let ok = result.module.is_some();
                    if result.error.is_some() {{
                        phase2_diags += 1;
                    }}
                    eprintln!("  parse   {{:>40}}: {{:>8.2?}}  (ok={{}})",
                        sources[i].path, elapsed, ok);
                    if let Some(m) = result.module.clone() {{
                        modules.push(m);
                    }}
                }}
                let parse_total = t_stage.elapsed();
                let rss_after_parse = get_rss_bytes();
                eprintln!("  PARSE TOTAL:    {{:?}}  | RSS: {{}}  | diags: {{}}\n",
                    parse_total, format_bytes(rss_after_parse), phase2_diags);

                // Phase 3: Resolve module graph
                let t_stage = Instant::now();
                let graph = crate::resolve::resolve_modules(std::rc::Rc::new(modules));
                let resolve_total = t_stage.elapsed();
                let phase3_diags: usize = graph.diagnostics.iter()
                    .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                    .count();
                let rss_after_resolve = get_rss_bytes();
                eprintln!("  RESOLVE TOTAL:  {{:?}}  | RSS: {{}}  | diags: {{}}\n",
                    resolve_total, format_bytes(rss_after_resolve), phase3_diags);

                // Phase 4: Reconcile (typecheck)
                let t_stage = Instant::now();
                let typed = crate::reconcile::reconcile(graph);
                let reconcile_total = t_stage.elapsed();
                let phase4_diags: usize = typed.diagnostics.iter()
                    .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                    .count();
                let rss_after_reconcile = get_rss_bytes();
                eprintln!("  RECONCILE TOTAL: {{:?}}  | RSS: {{}}  | diags: {{}}\n",
                    reconcile_total, format_bytes(rss_after_reconcile), phase4_diags);

                // Phase 5: Emit (Rust target)
                let t_stage = Instant::now();
                let emit_result = crate::emit_rust::emit_rust(typed);
                let emit_total = t_stage.elapsed();
                let phase5_diags: usize = emit_result.diagnostics.iter()
                    .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                    .count();
                let emitted_files = emit_result.files.len();
                let emitted_bytes: usize = emit_result.files.iter()
                    .map(|f| f.content.len())
                    .sum();
                let rss_after_emit = get_rss_bytes();
                eprintln!("  EMIT TOTAL:     {{:?}}  | RSS: {{}}  | diags: {{}}\n",
                    emit_total, format_bytes(rss_after_emit), phase5_diags);

                // Summary
                let total = tokenize_total + parse_total + resolve_total
                    + reconcile_total + emit_total;
                let total_diags = phase1_diags + phase2_diags + phase3_diags
                    + phase4_diags + phase5_diags;
                eprintln!("=== SUMMARY ===");
                eprintln!("  Tokenize:   {{:?}}", tokenize_total);
                eprintln!("  Parse:      {{:?}}", parse_total);
                eprintln!("  Resolve:    {{:?}}", resolve_total);
                eprintln!("  Reconcile:  {{:?}}", reconcile_total);
                eprintln!("  Emit:       {{:?}}", emit_total);
                eprintln!("  Total:      {{:?}}", total);
                eprintln!("  Diagnostics: {{}}", total_diags);
                eprintln!("  Emitted: {{}} files, {{}}", emitted_files, format_bytes(emitted_bytes as u64));
                eprintln!("");
                eprintln!("=== RSS CHECKPOINTS ===");
                eprintln!("  Start:          {{}}", format_bytes(rss_start));
                eprintln!("  After tokenize: {{}}", format_bytes(rss_after_tokenize));
                eprintln!("  After parse:    {{}}", format_bytes(rss_after_parse));
                eprintln!("  After resolve:  {{}}", format_bytes(rss_after_resolve));
                eprintln!("  After reconcile:{{}}", format_bytes(rss_after_reconcile));
                eprintln!("  After emit:     {{}}", format_bytes(rss_after_emit));
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("profile_self_compile test panicked");
    }}

    /// Per-module reconcile profile: runs tokenize+parse+resolve then
    /// typecheck_module for each module individually with RSS+timing.
    /// Isolates which module causes OOM/timeout in the reconcile phase.
    #[test]
    #[ignore]
    fn profile_reconcile_per_module() {{
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {{
                use std::time::Instant;
                use std::collections::HashMap;

                let sources: Vec<std::rc::Rc<crate::pipeline::SourceFile>> = vec![
{self_compile_source_files}                ];

                eprintln!("\n=== PER-MODULE RECONCILE PROFILE ({{}} sources) ===", sources.len());

                // Phases 1-3: tokenize + parse + resolve (known safe, ~28MB)
                let t0 = Instant::now();
                let mut modules = Vec::new();
                for source in &sources {{
                    let tokens = crate::tokenize::tokenize(&source.content);
                    let result = crate::parse::parse(tokens);
                    if let Some(m) = result.module.clone() {{
                        modules.push(m);
                    }} else {{
                        eprintln!("  WARN: parse failed for {{}}", source.path);
                    }}
                }}
                let graph = crate::resolve::resolve_modules(
                    std::rc::Rc::new(modules)
                );
                let setup_time = t0.elapsed();
                let rss_baseline = get_rss_bytes();
                eprintln!("  Setup (tok+parse+resolve): {{:?}}  | RSS: {{}}", setup_time, format_bytes(rss_baseline));
                eprintln!("  Modules to reconcile: {{}}\n", graph.modules.len());

                // Phase 4: typecheck each module individually
                let mut mi_raw = HashMap::<String, std::rc::Rc<crate::reconcile::TypedModule>>::new();

                for resolved in graph.modules.iter() {{
                    let name = resolved.module.name.to_string();
                    let item_count = resolved.module.items.len();
                    let rss_before = get_rss_bytes();

                    // Print BEFORE typecheck so we know which module crashed on SIGKILL
                    eprint!("  {{:>35}} ({{:>3}} items) ... ", name, item_count);

                    let module_index = std::rc::Rc::new(mi_raw.clone());

                    // Sub-step 0: build_type_env_unresolved (merge + cycle detection only)
                    let t_unres = Instant::now();
                    let _unres = crate::reconcile::build_type_env_unresolved(
                        resolved.clone(),
                        module_index.clone()
                    );
                    let unres_elapsed = t_unres.elapsed();
                    let rss_after_unres = get_rss_bytes();
                    let unres_delta = rss_after_unres.saturating_sub(rss_before);

                    eprint!("cycles={{:>8.2?}}(+{{}}) ", unres_elapsed, format_bytes(unres_delta));

                    if unres_delta > 256 * 1024 * 1024 {{
                        eprintln!("");
                        panic!("ABORT: '{{}}' cycle detection grew RSS by {{}}", name, format_bytes(unres_delta));
                    }}

                    // Sub-step 1: build_type_env (includes topo_resolve_types)
                    let t_env = Instant::now();
                    let env_result = crate::reconcile::build_type_env(
                        resolved.clone(),
                        module_index.clone()
                    );
                    let env_elapsed = t_env.elapsed();
                    let rss_after_env = get_rss_bytes();
                    let env_delta = rss_after_env.saturating_sub(rss_before);
                    let env_errs: usize = env_result.diagnostics.iter()
                        .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                        .count();

                    eprint!("env={{:>8.2?}}(+{{}},e={{}}) ", env_elapsed, format_bytes(env_delta), env_errs);

                    if env_delta > 512 * 1024 * 1024 {{
                        eprintln!("");
                        panic!("ABORT: '{{}}' build_type_env grew RSS by {{}}", name, format_bytes(env_delta));
                    }}
                    if env_elapsed.as_secs() > 10 {{
                        eprintln!("");
                        panic!("ABORT: '{{}}' build_type_env took {{:?}}", name, env_elapsed);
                    }}

                    // Sub-step 2: full typecheck_module
                    let t_full = Instant::now();
                    let tc_result = crate::reconcile::typecheck_module(
                        resolved.clone(),
                        module_index
                    );
                    let full_elapsed = t_full.elapsed();
                    let rss_after = get_rss_bytes();
                    let delta = rss_after.saturating_sub(rss_before);
                    let diag_count: usize = tc_result.diagnostics.iter()
                        .filter(|d| matches!(d.severity, crate::v2_core::Severity::Error))
                        .count();

                    eprintln!("full={{:>8.2?}}  | RSS: {{}} (+{{}})  | errs: {{}}",
                        full_elapsed, format_bytes(rss_after), format_bytes(delta), diag_count);

                    // Guardrails: abort before OOM kills the system
                    if delta > 512 * 1024 * 1024 {{
                        panic!("ABORT: '{{}}' grew RSS by {{}} (>512MB)", name, format_bytes(delta));
                    }}
                    if full_elapsed.as_secs() > 10 {{
                        panic!("ABORT: '{{}}' took {{:?}} (>10s)", name, full_elapsed);
                    }}

                    let typed = tc_result.typed.clone();
                    mi_raw.insert(name, typed);
                }}

                let rss_final = get_rss_bytes();
                eprintln!("\n  RSS final: {{}} (from baseline: +{{}})",
                    format_bytes(rss_final),
                    format_bytes(rss_final.saturating_sub(rss_baseline)));
                eprintln!("=== DONE ===\n");
            }})
            .expect("failed to spawn thread")
            .join();
        result.expect("profile_reconcile_per_module panicked");
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
