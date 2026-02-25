//! DSL-derived tool discovery.
//!
//! Replaces `derive_tool_defs()` (inventory-based) with discovery from
//! `@binary` annotations in DSL `.dag` files. Each `@binary func` in
//! `dsl/tools/*.dag` produces a [`ToolDef`] for CLI generation, Makefile
//! targets, and gitignore entries.
//!
//! Convention: binary name = func_name with `_` → `-`, unless overridden
//! by `@binary("name")`.
//!
//! Special case: testgen has a custom builder (no DSL module) and is
//! hardcoded as the sole exception.

use std::collections::HashSet;
use std::path::Path;

use daglang_driver::{compile_from_context, DriverContext};
use daglang_syntax::ast::{Expr, Item, Literal, TypeExpr};
use gunbc_cli::ParamType;
use gunbc_codegen::cli_gen::CliEntrypoint;
use gunbc_codegen::registry::ToolDef;
use gunbc_ir::{cargo, Cardinality, WorkspaceLayout};

/// A binary-producing func discovered from a `.dag` file.
#[derive(Debug)]
struct BinaryFunc {
    /// func name as written in DSL (e.g., "gist_snapshot")
    func_name: String,
    /// @binary("override") or None
    name_override: Option<String>,
    /// func params for CLI entrypoint derivation
    params: Vec<DslParam>,
}

/// A DSL func parameter, extracted from the AST.
#[derive(Debug)]
struct DslParam {
    name: String,
    type_id: ParamType,
    cardinality: Cardinality,
    default: Option<String>,
}

/// Discover tool definitions from DSL `@binary` annotations.
///
/// Scans `dsl/tools/*.dag` for `func` items with `@binary` annotations.
/// Each annotated func produces a [`ToolDef`] with:
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

    let mut tools = Vec::new();

    // Scan dsl/tools/*.dag
    let tools_dir = dsl_root.join("tools");
    if let Ok(entries) = std::fs::read_dir(&tools_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("dag") {
                continue;
            }
            if let Some(mut defs) = discover_from_dag_file(&dsl_root, &path) {
                tools.append(&mut defs);
            }
        }
    }

    // Special case: testgen (custom builder, no DSL module)
    tools.push(testgen_tool_def());

    tools.sort_by(|a, b| a.meta.tool_name.cmp(&b.meta.tool_name));
    tools
}

/// Parse a `.dag` file and produce `ToolDef`s for any `@binary` funcs.
#[allow(clippy::disallowed_methods)]
fn discover_from_dag_file(dsl_root: &Path, path: &Path) -> Option<Vec<ToolDef>> {
    let source = std::fs::read_to_string(path).ok()?;
    let ast = daglang_syntax::parser::parse(&source).ok()?;

    // Find @binary annotations on func items
    let mut binary_funcs = Vec::new();
    for item in &ast.items {
        if let Item::FuncDef(func) = &item.node {
            for ann in &func.annotations {
                if ann.name == "binary" {
                    let name_override = ann.args.first().and_then(|arg| {
                        if let Expr::Literal(Literal::String(s)) = arg {
                            Some(s.clone())
                        } else {
                            None
                        }
                    });
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
                    binary_funcs.push(BinaryFunc {
                        func_name: func.name.clone(),
                        name_override,
                        params,
                    });
                }
            }
        }
    }

    if binary_funcs.is_empty() {
        return None;
    }

    // Compile to get output_paths
    let rel_path = path
        .strip_prefix(dsl_root)
        .ok()?
        .to_string_lossy()
        .to_string();
    let module_name = rel_path
        .strip_suffix(".dag")?
        .replace('/', ".");

    let context = DriverContext {
        roots: vec![dsl_root.to_path_buf()],
        target_file: Some(path.to_path_buf()),
    };
    let compile_output = compile_from_context(&context).ok()?;

    let mut tools = Vec::new();

    for bf in &binary_funcs {
        let tool_name = bf
            .name_override
            .as_deref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| bf.func_name.replace('_', "-"));

        // Entry-point slicing: {module_name}::{func_name}
        let entry_node = format!("{}::{}", module_name, bf.func_name);
        let graph_builder_call = format!(
            "build_dsl_graph_for_entry(\"{}\", \"{}\")",
            rel_path, entry_node,
        );

        let description = humanize_tool_name(&tool_name);
        let mock_spec = format!(
            "gunbc_dag::mock_defaults::auto_mock_spec(&dag, \"{}\")",
            tool_name,
        );

        let entrypoints = derive_entrypoints(&bf.params);

        let mut tool = ToolDef::new(
            String::from("gunbc-dag"),
            tool_name.clone(),
            description,
            graph_builder_call,
            String::new(),
        )
        .returns_result()
        .mock_spec_call(mock_spec)
        .import("use gunbc_dag::dsl_builder::build_dsl_graph_for_entry;")
        .invocation(cargo::CargoInvocation::composed(&tool_name, "dag"));

        // Add outputs from compilation
        for output_path in &compile_output.output_paths {
            tool = tool.output(output_path.clone());
        }

        // Add entrypoints
        for ep in entrypoints {
            tool = tool.entrypoint(ep);
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
    fn discovers_binary_tools_from_dsl() {
        let tools = discover_tool_defs_from_dsl();

        let names: Vec<&str> = tools.iter().map(|t| t.meta.tool_name.as_ref()).collect();

        // All 7 @binary tools should be discovered
        assert!(names.contains(&"bootstrap"), "missing bootstrap");
        assert!(names.contains(&"deps"), "missing deps");
        assert!(names.contains(&"gist"), "missing gist");
        assert!(names.contains(&"gist-diff"), "missing gist-diff");
        assert!(names.contains(&"gist-recent"), "missing gist-recent");
        assert!(names.contains(&"makegen"), "missing makegen");
        assert!(names.contains(&"pragma"), "missing pragma");

        // Testgen (hardcoded)
        assert!(names.contains(&"testgen"), "missing testgen");
    }

    #[test]
    fn binary_tools_have_invocations() {
        let tools = discover_tool_defs_from_dsl();

        for tool in &tools {
            let name = tool.meta.tool_name.as_ref();
            if name == "testgen" {
                // Testgen has no invocation (custom binary)
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
        let gist_diff = tools.iter().find(|t| t.meta.tool_name == "gist-diff").unwrap();
        assert_eq!(gist_diff.entrypoints.len(), 2, "gist-diff should have 2 entrypoints");

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
        let pragma = tools.iter().find(|t| t.meta.tool_name == "pragma").unwrap();

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

    /// Contract test: compare DSL-derived tools against inventory-derived tools.
    ///
    /// Checks structural equivalence on key fields: tool_name, outputs, invocation.
    /// Entrypoints are expected to diverge (DSL derives from func params; inventory
    /// uses handcrafted JSON with service-level boundary params).
    #[test]
    fn dsl_derived_matches_inventory_derived() {
        let old = gunbc_codegen::registry::derive_tool_defs();
        let new = discover_tool_defs_from_dsl();

        // Collect names for comparison
        let old_names: HashSet<&str> = old.iter().map(|t| t.meta.tool_name.as_ref()).collect();
        let new_names: HashSet<&str> = new.iter().map(|t| t.meta.tool_name.as_ref()).collect();

        // The new system discovers @binary tools + testgen.
        // The old system also has non-binary tools (clippy, review, codegen).
        // Check that all NEW tools exist in the old system.
        for name in &new_names {
            assert!(
                old_names.contains(name),
                "DSL-derived tool '{}' not in inventory. New tools should be a subset.",
                name,
            );
        }

        // For tools in both systems, compare key fields
        for new_tool in &new {
            let name = new_tool.meta.tool_name.as_ref();
            let Some(old_tool) = old.iter().find(|t| t.meta.tool_name == name) else {
                continue;
            };

            // Outputs should match (from CompileOutput.output_paths vs ToolRegistration.outputs)
            let mut old_outputs = old_tool.outputs.clone();
            old_outputs.sort();
            let mut new_outputs = new_tool.outputs.clone();
            new_outputs.sort();
            assert_eq!(
                old_outputs, new_outputs,
                "{}: outputs diverge. old={:?}, new={:?}",
                name, old_outputs, new_outputs,
            );

            // Invocation presence should match
            assert_eq!(
                old_tool.invocation.is_some(),
                new_tool.invocation.is_some(),
                "{}: invocation presence diverges",
                name,
            );

            // If both have invocations, binary name should match
            if let (Some(old_inv), Some(new_inv)) =
                (&old_tool.invocation, &new_tool.invocation)
            {
                assert_eq!(
                    old_inv.binary, new_inv.binary,
                    "{}: binary name diverges",
                    name,
                );
            }
        }

        // Report tools only in old system (expected: clippy, review, codegen)
        let only_old: Vec<&&str> = old_names.difference(&new_names).collect();
        if !only_old.is_empty() {
            eprintln!(
                "Tools in inventory but not DSL-derived (expected): {:?}",
                only_old,
            );
        }
    }
}
