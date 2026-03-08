//! DSL-derived tool discovery.
//!
//! Discovers tool entrypoints from DSL `.dag` files using structural
//! inference: a `func` item with untapped input ports IS an entrypoint.
//! Each inferred entrypoint produces a [`ToolDef`] for CLI generation,
//! tool discovery, and gitignore entries.
//!
//! Convention: tool name = func_name with `_` → `-`.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use crate::cli_gen::CliEntrypoint;
use crate::registry::{SubcommandDef, ToolDef};
use daglang_driver::{
    compile_from_context, compute_source_digest_for_context, CachedDiscoveryEntry,
    CachedEntrypoint, CachedFuncParam, DriverContext, InferredEntrypoint,
};
use daglang_syntax::ast::{Expr, Item, Literal, TypeExpr};
use gunbc_cli::ParamType;
use gunbc_ir::{cargo, Cardinality, WorkspaceLayout};

/// A DSL func parameter, extracted from the AST.
#[derive(Debug, Clone)]
struct DslParam {
    name: String,
    type_id: ParamType,
    cardinality: Cardinality,
    default: Option<String>,
}

/// Bump when cache format changes (e.g., new fields in CachedDiscoveryEntry).
/// Stale caches with a different version are discarded on load.
const CACHE_VERSION: u32 = 4;

/// Persistent discovery cache for incremental compilation (C26).
///
/// Stores per-module source digests and cached compilation metadata.
/// On cache hit (source digest matches), compilation is skipped entirely.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct DiscoveryCache {
    #[serde(default)]
    cache_version: u32,
    entries: BTreeMap<String, CachedDiscoveryEntry>,
}

impl DiscoveryCache {
    fn cache_path(workspace_root: &Path) -> PathBuf {
        workspace_root
            .join("target")
            .join("dag-cache")
            .join("discovery_cache.json")
    }

    /// Load cache from disk.
    ///
    /// Returns `Ok(None)` if file doesn't exist or cache version mismatches.
    /// Returns `Err` for read/parse failures.
    /// Build-time filesystem access (bootstrap exception).
    #[allow(clippy::disallowed_methods)]
    fn load(workspace_root: &Path) -> Result<Option<Self>, String> {
        let path = Self::cache_path(workspace_root);
        match std::fs::read_to_string(&path) {
            Ok(s) => {
                let cache = serde_json::from_str::<Self>(&s).map_err(|e| {
                    format!("discovery cache parse failed at {}: {e}", path.display())
                })?;
                if cache.cache_version != CACHE_VERSION {
                    return Ok(None);
                }
                Ok(Some(cache))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!(
                "cannot read discovery cache at {}: {e}",
                path.display()
            )),
        }
    }

    /// Persist cache to disk.
    ///
    /// Returns `Err` on create/serialize/write failures.
    /// Build-time filesystem access (bootstrap exception).
    #[allow(clippy::disallowed_methods)]
    fn save(&mut self, workspace_root: &Path) -> Result<(), String> {
        self.cache_version = CACHE_VERSION;
        let path = Self::cache_path(workspace_root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create cache directory {}: {e}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("cannot serialize discovery cache: {e}"))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("cannot write discovery cache to {}: {e}", path.display()))?;
        Ok(())
    }
}

/// Discover tool definitions from DSL entrypoint inference.
///
/// Scans `dsl/tools/*.dag` for `func` items with untapped inputs
/// (structurally inferred entrypoints). Each entrypoint produces a
/// [`ToolDef`] with:
/// - CLI entrypoints derived from func params (convention-based)
/// - Outputs from DSL compilation (`CompileOutput.output_paths`) plus
///   optional `data output_paths: List<String>` declarations for dynamic cases
/// - Invocation as `cargo run -p <tool-host-package> --bin gunbc-{name}`
/// - MockSpec as `auto_mock_spec(&dag, "{name}")`
///
/// Uses content-hash-based incremental caching (C26): unchanged modules
/// skip parse+typecheck+lower+emit entirely.
///
pub fn discover_tool_defs_from_dsl() -> Result<Vec<ToolDef>, String> {
    try_discover_tool_defs_from_dsl()
}

/// Fallible discovery entrypoint for callers that need structured errors.
#[allow(clippy::disallowed_methods)]
pub fn try_discover_tool_defs_from_dsl() -> Result<Vec<ToolDef>, String> {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .map_err(|e| format!("workspace layout for DSL discovery: {e}"))?;
    let dsl_root = layout.workspace_root.join("dsl");

    let mut cache = DiscoveryCache::load(&layout.workspace_root)?.unwrap_or_default();
    let mut cache_dirty = false;

    // Use BTreeMap for dedup by tool_name (later entries overwrite earlier,
    // so dedicated files like gist_diff.dag win over combined gist.dag).
    let mut tool_map: BTreeMap<String, ToolDef> = BTreeMap::new();

    // Scan dsl/tools/*.dag
    let tools_dir = dsl_root.join("tools");
    let entries = std::fs::read_dir(&tools_dir)
        .map_err(|e| format!("cannot read tools directory {}: {e}", tools_dir.display()))?;
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            format!(
                "cannot iterate tools directory entries in {}: {e}",
                tools_dir.display()
            )
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            paths.push(path);
        }
    }
    paths.sort();

    for path in paths {
        if let Some(defs) =
            discover_from_dag_file_cached(&dsl_root, &path, &mut cache, &mut cache_dirty)?
        {
            for tool in defs {
                tool_map.insert(tool.meta.tool_name.to_string(), tool);
            }
        }
    }

    // Persist cache if anything changed
    if cache_dirty {
        cache.save(&layout.workspace_root)?;
    }

    Ok(tool_map.into_values().collect())
}

/// Cache-aware discovery: check source digest before full compilation (C26).
///
/// Returns `Ok(Some(defs))` on success, `Ok(None)` if no entrypoints,
/// `Err(msg)` on read/parse/compile failure.
#[allow(clippy::disallowed_methods)]
fn discover_from_dag_file_cached(
    dsl_root: &Path,
    path: &Path,
    cache: &mut DiscoveryCache,
    cache_dirty: &mut bool,
) -> Result<Option<Vec<ToolDef>>, String> {
    let rel_path = path
        .strip_prefix(dsl_root)
        .map_err(|e| format!("path prefix: {e}"))?
        .to_string_lossy()
        .to_string();
    let module_name = rel_path
        .strip_suffix(".dag")
        .ok_or_else(|| format!("not a .dag file: {rel_path}"))?
        .replace('/', ".");

    let context = DriverContext {
        roots: vec![dsl_root.to_path_buf()],
        target_file: Some(path.to_path_buf()),
    };

    let source = std::fs::read_to_string(path).map_err(|e| format!("read: {e}"))?;
    let ast = daglang_syntax::parser::parse(&source).map_err(|e| format!("parse: {e:?}"))?;
    let func_params = extract_func_params_from_ast(&ast)?;
    if func_params.is_empty() {
        return Ok(None);
    }
    let success_ports = extract_success_ports_from_ast(&ast);

    // Compute source digest (cheap: module graph discovery + file hashing)
    let source_digest =
        compute_source_digest_for_context(&context).map_err(|e| format!("source digest: {e}"))?;

    // Cache hit: skip full compilation and reconstruct from cached metadata.
    if let Some(cached) = cache.entries.get(&module_name) {
        if cached.source_digest == source_digest {
            let cached_entrypoints: Vec<InferredEntrypoint> = cached
                .entrypoints
                .iter()
                .map(InferredEntrypoint::from)
                .collect();
            return Ok(build_tool_defs_from_cached_params(
                &rel_path,
                &module_name,
                &cached_entrypoints,
                &cached.output_paths,
                &func_params,
                &success_ports,
            ));
        }
    }

    // Cache miss: full compilation (build-time bootstrap exception)
    let compile_output = compile_from_context(&context).map_err(|e| format!("compile: {e}"))?;
    let output_paths = merge_output_paths(
        &compile_output.output_paths,
        &declared_output_paths(&compile_output.data_values)?,
    );

    let module_entrypoints: Vec<&InferredEntrypoint> = compile_output
        .inferred_entrypoints
        .iter()
        .filter(|ep| ep.module == module_name)
        .collect();

    if module_entrypoints.is_empty() {
        return Ok(None);
    }

    // Build ToolDefs from compilation output
    let result = build_tool_defs_from_cached_params(
        &rel_path,
        &module_name,
        &module_entrypoints
            .iter()
            .copied()
            .cloned()
            .collect::<Vec<_>>(),
        &output_paths,
        &func_params,
        &success_ports,
    );

    // Update cache with func params
    let cached_params = dsl_params_to_cached(&func_params);
    let entry = CachedDiscoveryEntry {
        source_digest,
        entrypoints: module_entrypoints
            .iter()
            .map(|ep| CachedEntrypoint::from(*ep))
            .collect(),
        output_paths,
        func_params: cached_params,
    };
    cache.entries.insert(module_name, entry);
    *cache_dirty = true;

    Ok(result)
}

/// Extract func parameters from a parsed AST.
fn extract_func_params_from_ast(
    ast: &daglang_syntax::ast::SourceFile,
) -> Result<BTreeMap<String, Vec<DslParam>>, String> {
    let mut func_params: BTreeMap<String, Vec<DslParam>> = BTreeMap::new();
    for item in &ast.items {
        if let Item::FuncDef(func) = &item.node {
            let mut params = Vec::with_capacity(func.params.len());
            for p in &func.params {
                let (type_id, cardinality) = map_type_expr(&p.ty).map_err(|e| {
                    format!(
                        "unsupported CLI type mapping for {}.{} parameter `{}`: {e}",
                        ast.module_path
                            .as_ref()
                            .map(|m| m.node.as_dotted())
                            .unwrap_or_default(),
                        func.name,
                        p.name
                    )
                })?;
                let default = p.default.as_ref().and_then(expr_to_default_string);
                params.push(DslParam {
                    name: p.name.clone(),
                    type_id,
                    cardinality,
                    default,
                });
            }
            func_params.insert(func.name.clone(), params);
        }
    }
    Ok(func_params)
}

fn extract_success_ports_from_ast(
    ast: &daglang_syntax::ast::SourceFile,
) -> BTreeMap<String, Option<String>> {
    let mut success_ports = BTreeMap::new();
    for item in &ast.items {
        if let Item::FuncDef(func) = &item.node {
            success_ports.insert(func.name.clone(), infer_success_port(&func.outputs));
        }
    }
    success_ports
}

fn infer_success_port(fields: &[daglang_syntax::ast::Field]) -> Option<String> {
    ["success", "overall_success"]
        .into_iter()
        .find(|candidate| {
            fields
                .iter()
                .any(|field| field.name == *candidate && is_bool_type(&field.ty))
        })
        .map(str::to_string)
}

fn is_bool_type(ty: &TypeExpr) -> bool {
    match ty {
        TypeExpr::Named(name) => name == "Bool",
        TypeExpr::Refined(inner, _) => is_bool_type(inner),
        _ => false,
    }
}

/// Convert DslParam map to cached format for serialization.
fn dsl_params_to_cached(
    params: &BTreeMap<String, Vec<DslParam>>,
) -> BTreeMap<String, Vec<CachedFuncParam>> {
    params
        .iter()
        .map(|(name, dsl_params)| {
            let cached = dsl_params
                .iter()
                .map(|p| CachedFuncParam {
                    name: p.name.clone(),
                    type_name: p.type_id.as_str().to_string(),
                    cardinality: encode_cached_cardinality(p.cardinality),
                    default: p.default.clone(),
                })
                .collect();
            (name.clone(), cached)
        })
        .collect()
}

fn declared_output_paths(
    data_values: &std::collections::HashMap<String, serde_json::Value>,
) -> Result<Vec<String>, String> {
    let Some(value) = data_values.get("output_paths") else {
        return Ok(Vec::new());
    };
    let serde_json::Value::Array(items) = value else {
        return Err("data output_paths must be a List<String>".to_string());
    };
    let mut output_paths = Vec::with_capacity(items.len());
    for item in items {
        match item {
            serde_json::Value::String(path) => output_paths.push(path.clone()),
            _ => return Err("data output_paths must contain only strings".to_string()),
        }
    }
    Ok(output_paths)
}

fn merge_output_paths(inferred: &[String], declared: &[String]) -> Vec<String> {
    let mut merged = BTreeSet::new();
    merged.extend(inferred.iter().cloned());
    merged.extend(declared.iter().cloned());
    merged.into_iter().collect()
}

fn encode_cached_cardinality(cardinality: Cardinality) -> String {
    let max = cardinality
        .max
        .map(|m| m.to_string())
        .unwrap_or_else(|| "*".to_string());
    format!("{}:{max}", cardinality.min)
}

fn dsl_graph_builder_adapter() -> String {
    // Returns a callable expression that takes (relative_module, bindings, opts).
    // The `{` ... `}` block avoids the clippy::redundant_closure_call lint
    // that triggers when a closure is immediately invoked.
    String::from("gunbc_resolve::builder::build_dsl_graph_dag")
}

fn dsl_graph_builder_args(rel_path: &str, entry_func: &str) -> String {
    format!(
        "\"{rel_path}\", gunbc_resolve::BuildOpts {{ entry_func: Some(\"{entry_func}\"), profile: None }}"
    )
}

fn extern_resolver_import() -> &'static str {
    ""
}

/// Build ToolDefs from pre-extracted func params (shared by cache-hit and cache-miss paths).
fn build_tool_defs_from_cached_params(
    rel_path: &str,
    module_name: &str,
    module_entrypoints: &[InferredEntrypoint],
    output_paths: &[String],
    func_params: &BTreeMap<String, Vec<DslParam>>,
    success_ports: &BTreeMap<String, Option<String>>,
) -> Option<Vec<ToolDef>> {
    if module_entrypoints.is_empty() {
        return None;
    }

    // RT63: When a module has multiple entrypoints, produce ONE ToolDef
    // with subcommand dispatch instead of N separate binaries.
    if module_entrypoints.len() > 1 {
        let module_tool_name = module_name
            .rsplit('.')
            .next()
            .unwrap_or("tool")
            .replace('_', "-");
        let description = humanize_tool_name(&module_tool_name);

        let mut subcommands = Vec::new();
        for ep in module_entrypoints {
            let subcmd_name = ep.func_name.replace('_', "-");
            let graph_builder_args = dsl_graph_builder_args(rel_path, &ep.func_name);
            let mock_spec = format!("gunbc_test::auto_mock_spec(&dag, \"{}\")", subcmd_name,);
            let entrypoints = func_params
                .get(&ep.func_name)
                .map(|params| derive_entrypoints(params))
                .unwrap_or_default();

            subcommands.push(SubcommandDef {
                name: subcmd_name.clone(),
                func_name: ep.func_name.clone(),
                description: humanize_tool_name(&subcmd_name),
                graph_builder_call: dsl_graph_builder_adapter(),
                graph_builder_args,
                returns_result: true,
                success_port: success_ports.get(&ep.func_name).cloned().flatten(),
                mock_spec_call: Some(mock_spec),
                entrypoints,
            });
        }

        let first_args = dsl_graph_builder_args(rel_path, &module_entrypoints[0].func_name);
        let mock_spec = format!("gunbc_test::auto_mock_spec(&dag, \"{}\")", module_tool_name,);

        let mut tool = ToolDef::new(
            String::from("crate"),
            module_tool_name.clone(),
            description,
            dsl_graph_builder_adapter(),
            first_args,
        )
        .returns_result()
        .mock_spec_call(mock_spec)
        .import(extern_resolver_import())
        .invocation(cargo::CargoInvocation::composed(
            &module_tool_name,
            "codegen",
        ));

        for output_path in output_paths {
            tool = tool.output(output_path.clone());
        }
        for subcmd in subcommands {
            tool = tool.subcommand(subcmd);
        }
        if !output_paths.is_empty() {
            tool = tool.enable_mode();
        }

        return Some(vec![tool]);
    }

    // Single entrypoint
    let mut tools = Vec::new();
    for ep in module_entrypoints {
        let tool_name = ep.func_name.replace('_', "-");
        let graph_builder_args = dsl_graph_builder_args(rel_path, &ep.func_name);
        let description = humanize_tool_name(&tool_name);
        let mock_spec = format!("gunbc_test::auto_mock_spec(&dag, \"{}\")", tool_name,);
        let entrypoints = func_params
            .get(&ep.func_name)
            .map(|params| derive_entrypoints(params))
            .unwrap_or_default();

        let mut tool = ToolDef::new(
            String::from("crate"),
            tool_name.clone(),
            description,
            dsl_graph_builder_adapter(),
            graph_builder_args,
        )
        .returns_result()
        .mock_spec_call(mock_spec)
        .import(extern_resolver_import())
        .invocation(cargo::CargoInvocation::composed(&tool_name, "codegen"));

        if let Some(success_port) = success_ports.get(&ep.func_name).cloned().flatten() {
            tool = tool.check_success(success_port);
        }

        for output_path in output_paths {
            tool = tool.output(output_path.clone());
        }
        if !output_paths.is_empty() {
            tool = tool.enable_mode();
        }
        for cli_ep in entrypoints {
            tool = tool.entrypoint(cli_ep);
        }

        tools.push(tool);
    }

    Some(tools)
}

// ── Entrypoint derivation ──────────────────────────────────────────

/// Derive CLI entrypoints from DSL func params using conventions.
///
/// Convention:
/// - `port_name` = param name
/// - `type_id` = mapped from DSL TypeExpr (String/Bool/Int)
/// - `short_flag` = first char of param name (skip on collision)
/// - `default` = from DSL default expression (literals only)
/// - `help` = humanized param name
/// - `make_var` = UPPER_SNAKE(param name), omitted for Bool params
/// - `cardinality` = from DSL type (List<T> → ZERO_OR_MORE, T? → ZERO_OR_ONE)
fn derive_entrypoints(params: &[DslParam]) -> Vec<CliEntrypoint> {
    let mut used_shorts = HashSet::new();
    let mut entrypoints = Vec::new();

    for param in params {
        let short =
            param.name.chars().next().and_then(
                |c| {
                    if used_shorts.insert(c) {
                        Some(c)
                    } else {
                        None
                    }
                },
            );

        let help = humanize_param_name(&param.name);

        let mut ep = CliEntrypoint::new(&param.name, param.type_id)
            .with_cardinality(param.cardinality)
            .help(help);

        if let Some(c) = short {
            ep = ep.short(c);
        }
        if let Some(ref d) = param.default {
            ep = ep.default(d);
        }
        // Bool params don't get make_var (they're flags, not Makefile variables)
        if param.type_id != ParamType::Bool {
            ep = ep.make_var(param.name.to_uppercase());
        }

        entrypoints.push(ep);
    }

    entrypoints
}

// ── Type mapping ───────────────────────────────────────────────────

/// Map a DSL TypeExpr to (ParamType, Cardinality).
fn map_type_expr(ty: &TypeExpr) -> Result<(ParamType, Cardinality), String> {
    match ty {
        TypeExpr::Named(name) => {
            let type_id = match name.as_str() {
                "Bool" => ParamType::Bool,
                "Int" | "Integer" => ParamType::Int,
                // String, CommitSha, Url, FilePath, Platform, etc. → Str
                _ => ParamType::Str,
            };
            Ok((type_id, Cardinality::ONE))
        }
        TypeExpr::Optional(inner) => {
            let (type_id, _) = map_type_expr(inner)?;
            Ok((type_id, Cardinality::ZERO_OR_ONE))
        }
        TypeExpr::Generic(name, args) if name == "List" => {
            let inner = args.first().ok_or_else(|| {
                "List type requires one type argument for CLI entrypoint mapping".to_string()
            })?;
            let (type_id, _) = map_type_expr(inner)?;
            Ok((type_id, Cardinality::ZERO_OR_MORE))
        }
        TypeExpr::Generic(name, _) => Err(format!(
            "generic type `{name}` is not supported for CLI entrypoint mapping"
        )),
        other => Err(format!(
            "type expression `{other:?}` is not supported for CLI entrypoint mapping"
        )),
    }
}

/// Extract a default value string from a DSL expression (literals only).
fn expr_to_default_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Literal(Literal::String(s)) => Some(s.clone()),
        Expr::Literal(Literal::Bool(b)) => Some(b.to_string()),
        Expr::Literal(Literal::Int(i)) => Some(i.to_string()),
        _ => None,
    }
}

// ── Naming conventions ─────────────────────────────────────────────

fn humanize_tool_name(name: &str) -> String {
    let mut words = name.split('-');
    let mut result = String::new();
    if let Some(first) = words.next() {
        result.push_str(&capitalize(first));
    }
    for word in words {
        result.push(' ');
        result.push_str(word);
    }
    result
}

fn humanize_param_name(name: &str) -> String {
    let mut words = name.split('_');
    let mut result = String::new();
    if let Some(first) = words.next() {
        result.push_str(&capitalize(first));
    }
    for word in words {
        result.push(' ');
        result.push_str(word);
    }
    result
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::disallowed_methods)] // Tests need filesystem access for scanning
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rust_files(&path, files);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    #[test]
    fn discovers_inferred_tools_from_dsl() {
        let tools = discover_tool_defs_from_dsl().expect("tool discovery should succeed");

        let names: Vec<&str> = tools.iter().map(|t| t.meta.tool_name.as_ref()).collect();

        // Single-func modules → standalone tools
        assert!(names.contains(&"bootstrap"), "missing bootstrap");
        assert!(names.contains(&"readme"), "missing readme");
    }

    #[test]
    fn inferred_tools_have_invocations() {
        let tools = discover_tool_defs_from_dsl().expect("tool discovery should succeed");

        for tool in &tools {
            let name = tool.meta.tool_name.as_ref();
            assert!(
                tool.invocation.is_some(),
                "{} should have an invocation",
                name,
            );
            let inv = tool.invocation.as_ref().unwrap();
            assert!(
                inv.binary.starts_with("gunbc-"),
                "{}: binary should start with gunbc-, got {}",
                name,
                inv.binary,
            );
        }
    }

    #[test]
    fn all_tools_return_result() {
        let tools = discover_tool_defs_from_dsl().expect("tool discovery should succeed");
        for tool in &tools {
            assert!(
                tool.meta.returns_result,
                "{} should have returns_result=true",
                tool.meta.tool_name,
            );
        }
    }

    #[test]
    fn tools_infer_success_ports_from_output_fields() {
        let tools = discover_tool_defs_from_dsl().expect("tool discovery should succeed");

        let codegen = tools
            .iter()
            .find(|t| t.meta.tool_name == "codegen")
            .unwrap();
        assert_eq!(codegen.meta.success_port.as_deref(), Some("success"));

        let build = tools
            .iter()
            .find(|t| t.meta.tool_name == "build-all")
            .unwrap();
        assert_eq!(build.meta.success_port.as_deref(), Some("overall_success"));
    }

    #[test]
    fn entrypoints_derived_from_func_params() {
        let tools = discover_tool_defs_from_dsl().expect("tool discovery should succeed");

        // bootstrap has func params → entrypoints should be derived
        let bootstrap = tools
            .iter()
            .find(|t| t.meta.tool_name == "bootstrap")
            .expect("bootstrap tool should exist");
        assert!(
            !bootstrap.entrypoints.is_empty(),
            "bootstrap should have at least one entrypoint"
        );
    }

    #[test]
    fn tool_names_sorted() {
        let tools = discover_tool_defs_from_dsl().expect("tool discovery should succeed");
        let names: Vec<&str> = tools.iter().map(|t| t.meta.tool_name.as_ref()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted, "tools should be sorted by name");
    }

    #[test]
    fn tools_use_entrypoint_builder() {
        let tools = discover_tool_defs_from_dsl().expect("tool discovery should succeed");
        for tool in &tools {
            assert_eq!(
                tool.meta.graph_builder_call.as_ref(),
                dsl_graph_builder_adapter(),
                "{} should use the direct gunbc_resolve builder adapter",
                tool.meta.tool_name,
            );
        }
    }

    #[test]
    fn clippy_tool_no_enable_mode() {
        let tools = discover_tool_defs_from_dsl().expect("tool discovery should succeed");
        let clippy = tools.iter().find(|t| t.meta.tool_name == "clippy-lint");
        // clippy-lint may or may not have outputs; if it exists without outputs, enable_mode should be false
        if let Some(tool) = clippy {
            if tool.outputs.is_empty() {
                assert!(
                    !tool.meta.enable_mode,
                    "clippy-lint has no outputs and should have enable_mode=false",
                );
            }
        }
    }

    #[test]
    fn derive_tool_defs_symbol_is_not_used_in_rust_sources() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let mut files = Vec::new();
        collect_rust_files(&workspace_root.join("src"), &mut files);

        let mut offenders = Vec::new();
        let needle = ["derive_tool_defs", "("].concat();
        for file in files {
            let Ok(source) = fs::read_to_string(&file) else {
                continue;
            };
            if source.contains(&needle) {
                offenders.push(file.display().to_string());
            }
        }

        assert!(
            offenders.is_empty(),
            "derive_tool_defs is removed; found stale references in: {:?}",
            offenders
        );
    }
}
