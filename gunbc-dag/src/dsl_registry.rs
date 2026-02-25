//! DSL-derived tool discovery.
//!
//! Discovers tool entrypoints from DSL `.dag` files using structural
//! inference: a `func` item with untapped input ports IS an entrypoint.
//! Each inferred entrypoint produces a [`ToolDef`] for CLI generation,
//! Makefile targets, and gitignore entries.
//!
//! Convention: tool name = func_name with `_` → `-`.
//!
//! Special case: testgen has a custom builder (no DSL module) and is
//! hardcoded as the sole exception.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use daglang_driver::{compile_from_context, DriverContext, InferredEntrypoint};
use daglang_syntax::ast::{Expr, Item, Literal, TypeExpr};
use gunbc_cli::ParamType;
use gunbc_codegen::cli_gen::CliEntrypoint;
use gunbc_codegen::registry::ToolDef;
use gunbc_ir::{cargo, Cardinality, WorkspaceLayout};

/// A DSL func parameter, extracted from the AST.
#[derive(Debug)]
struct DslParam {
    name: String,
    type_id: ParamType,
    cardinality: Cardinality,
    default: Option<String>,
}

/// Discover tool definitions from DSL entrypoint inference.
///
/// Scans `dsl/tools/*.dag` for `func` items with untapped inputs
/// (structurally inferred entrypoints). Each entrypoint produces a
/// [`ToolDef`] with:
/// - CLI entrypoints derived from func params (convention-based)
/// - Outputs from DSL compilation (`CompileOutput.output_paths`)
/// - Invocation as `cargo run -p gunbc-dag --bin gunbc-{name}`
/// - MockSpec as `auto_mock_spec(&dag, "{name}")`
///
/// Testgen is the sole hardcoded exception (custom builder, no DSL module).
#[allow(clippy::disallowed_methods)]
pub fn discover_tool_defs_from_dsl() -> Vec<ToolDef> {
    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .expect("workspace layout for DSL discovery");
    let dsl_root = layout.workspace_root.join("dsl");

    // Use BTreeMap for dedup by tool_name (later entries overwrite earlier,
    // so dedicated files like gist_diff.dag win over combined gist.dag).
    let mut tool_map: BTreeMap<String, ToolDef> = BTreeMap::new();

    // Scan dsl/tools/*.dag
    let tools_dir = dsl_root.join("tools");
    if let Ok(entries) = std::fs::read_dir(&tools_dir) {
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("dag"))
            .collect();
        paths.sort();

        for path in paths {
            if let Some(defs) = discover_from_dag_file(&dsl_root, &path) {
                for tool in defs {
                    tool_map.insert(tool.meta.tool_name.to_string(), tool);
                }
            }
        }
    }

    // Special case: testgen (custom builder, no DSL module)
    let testgen = testgen_tool_def();
    tool_map.insert(testgen.meta.tool_name.to_string(), testgen);

    tool_map.into_values().collect()
}

/// Parse a `.dag` file and produce `ToolDef`s for inferred entrypoints.
#[allow(clippy::disallowed_methods)]
fn discover_from_dag_file(dsl_root: &Path, path: &Path) -> Option<Vec<ToolDef>> {
    let source = std::fs::read_to_string(path).ok()?;
    let ast = daglang_syntax::parser::parse(&source).ok()?;

    // Compile to get inferred_entrypoints and output_paths
    let rel_path = path
        .strip_prefix(dsl_root)
        .ok()?
        .to_string_lossy()
        .to_string();
    let module_name = rel_path.strip_suffix(".dag")?.replace('/', ".");

    let context = DriverContext {
        roots: vec![dsl_root.to_path_buf()],
        target_file: Some(path.to_path_buf()),
    };
    let compile_output = compile_from_context(&context).ok()?;

    // Filter entrypoints to this module only
    let module_entrypoints: Vec<&InferredEntrypoint> = compile_output
        .inferred_entrypoints
        .iter()
        .filter(|ep| ep.module == module_name)
        .collect();

    if module_entrypoints.is_empty() {
        return None;
    }

    // Build a map of func name → AST params for param extraction
    let mut func_params: BTreeMap<String, Vec<DslParam>> = BTreeMap::new();
    for item in &ast.items {
        if let Item::FuncDef(func) = &item.node {
            let params = func
                .params
                .iter()
                .map(|p| {
                    let (type_id, cardinality) = map_type_expr(&p.ty);
                    let default = p.default.as_ref().and_then(expr_to_default_string);
                    DslParam {
                        name: p.name.clone(),
                        type_id,
                        cardinality,
                        default,
                    }
                })
                .collect();
            func_params.insert(func.name.clone(), params);
        }
    }

    let mut tools = Vec::new();

    for ep in &module_entrypoints {
        let tool_name = ep.func_name.replace('_', "-");

        let graph_builder_args = format!("\"{}\", Some(\"{}\")", rel_path, ep.func_name);

        let description = humanize_tool_name(&tool_name);
        let mock_spec = format!(
            "gunbc_dag::mock_defaults::auto_mock_spec(&dag, \"{}\")",
            tool_name,
        );

        // Get params from AST (lowered ports lose DSL type specifics)
        let entrypoints = func_params
            .get(&ep.func_name)
            .map(|params| derive_entrypoints(params))
            .unwrap_or_default();

        let mut tool = ToolDef::new(
            String::from("gunbc-dag"),
            tool_name.clone(),
            description,
            String::from("build_dsl_graph_for_entrypoint"),
            graph_builder_args,
        )
        .returns_result()
        .mock_spec_call(mock_spec)
        .import("use gunbc_dag::dsl_builder::build_dsl_graph_for_entrypoint;")
        .invocation(cargo::CargoInvocation::composed(&tool_name, "dag"));

        // Add outputs from compilation
        for output_path in &compile_output.output_paths {
            tool = tool.output(output_path.clone());
        }

        // Add entrypoints
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
        let short = param.name.chars().next().and_then(|c| {
            if used_shorts.insert(c) {
                Some(c)
            } else {
                None
            }
        });

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
fn map_type_expr(ty: &TypeExpr) -> (ParamType, Cardinality) {
    match ty {
        TypeExpr::Named(name) => {
            let type_id = match name.as_str() {
                "Bool" => ParamType::Bool,
                "Int" | "Integer" => ParamType::Int,
                // String, CommitSha, Url, FilePath, Platform, etc. → Str
                _ => ParamType::Str,
            };
            (type_id, Cardinality::ONE)
        }
        TypeExpr::Optional(inner) => {
            let (type_id, _) = map_type_expr(inner);
            (type_id, Cardinality::ZERO_OR_ONE)
        }
        TypeExpr::Generic(name, args) if name == "List" && !args.is_empty() => {
            let (type_id, _) = map_type_expr(&args[0]);
            (type_id, Cardinality::ZERO_OR_MORE)
        }
        _ => (ParamType::Str, Cardinality::ONE),
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

// ── Hardcoded exceptions ───────────────────────────────────────────

/// Testgen: custom builder, no DSL module.
fn testgen_tool_def() -> ToolDef {
    ToolDef::new(
        "gunbc-dag",
        "testgen",
        "Generate tests from DAG mock specifications",
        "build_testgen_graph_auto",
        "",
    )
    .returns_result()
    .mock_spec_call(r#"gunbc_dag::mock_defaults::auto_mock_spec(&dag, "testgen")"#)
    .import("use gunbc_dag::testgen_dag::build_testgen_graph_auto;")
    .output("**/generated_tests*.rs")
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_inferred_tools_from_dsl() {
        let tools = discover_tool_defs_from_dsl();

        let names: Vec<&str> = tools.iter().map(|t| t.meta.tool_name.as_ref()).collect();

        // All inferred entrypoints should be discovered
        assert!(names.contains(&"bootstrap"), "missing bootstrap");
        assert!(names.contains(&"gist-diff"), "missing gist-diff");
        assert!(names.contains(&"gist-recent"), "missing gist-recent");
        assert!(names.contains(&"makegen"), "missing makegen");
        assert!(names.contains(&"pragma"), "missing pragma");

        // Testgen (hardcoded)
        assert!(names.contains(&"testgen"), "missing testgen");
    }

    #[test]
    fn inferred_tools_have_invocations() {
        let tools = discover_tool_defs_from_dsl();

        for tool in &tools {
            let name = tool.meta.tool_name.as_ref();
            if name == "testgen" {
                continue;
            }
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
        let tools = discover_tool_defs_from_dsl();
        for tool in &tools {
            assert!(
                tool.meta.returns_result,
                "{} should have returns_result=true",
                tool.meta.tool_name,
            );
        }
    }

    #[test]
    fn entrypoints_derived_from_func_params() {
        let tools = discover_tool_defs_from_dsl();

        // gist_diff has: base_ref (CommitSha, default "HEAD~1"), public (Bool, default false)
        let gist_diff = tools
            .iter()
            .find(|t| t.meta.tool_name == "gist-diff")
            .unwrap();
        assert_eq!(
            gist_diff.entrypoints.len(),
            2,
            "gist-diff should have 2 entrypoints"
        );

        let base_ref = &gist_diff.entrypoints[0];
        assert_eq!(base_ref.port_name, "base_ref");
        assert_eq!(base_ref.type_id, ParamType::Str); // CommitSha → Str
        assert_eq!(base_ref.default_value.as_deref(), Some("HEAD~1"));

        let public = &gist_diff.entrypoints[1];
        assert_eq!(public.port_name, "public");
        assert_eq!(public.type_id, ParamType::Bool);
    }

    #[test]
    fn pragma_has_expected_outputs() {
        let tools = discover_tool_defs_from_dsl();
        let pragma = tools
            .iter()
            .find(|t| t.meta.tool_name == "pragma")
            .unwrap();

        // pragma produces 3 outputs via content_upsert
        assert!(
            !pragma.outputs.is_empty(),
            "pragma should have output paths from content_upsert",
        );
    }

    #[test]
    fn tool_names_sorted() {
        let tools = discover_tool_defs_from_dsl();
        let names: Vec<&str> = tools.iter().map(|t| t.meta.tool_name.as_ref()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted, "tools should be sorted by name");
    }

    #[test]
    fn tools_use_entrypoint_builder() {
        let tools = discover_tool_defs_from_dsl();
        for tool in &tools {
            if tool.meta.tool_name == "testgen" {
                continue;
            }
            assert_eq!(
                tool.meta.graph_builder_call.as_ref(),
                "build_dsl_graph_for_entrypoint",
                "{} should use build_dsl_graph_for_entrypoint",
                tool.meta.tool_name,
            );
        }
    }
}
