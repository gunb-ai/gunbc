//! DAG test discovery and auto-testgen pipeline.
//!
//! Two discovery modes:
//!
//! 1. **Auto-discovery** (`discover_compilable_modules`): Scans all of `dsl/`
//!    recursively for `.dag` files with `func` items. Every compilable module
//!    gets full testgen treatment via `auto_mock_spec()` — zero manual input.
//!
//! 2. **Test-block discovery** (`discover_dag_tests`): Legacy path that scans
//!    `dsl/tools/*.dag` for inline `test` blocks. Provides fixture overrides
//!    on top of auto-mocked defaults.
//!
//! Tier, hermetic, and fermi metadata are inferred from DAG topology by
//! `generate_target()`, not declared statically in annotations.

use crate::dsl_builder::{build_dsl_graph, build_dsl_graph_with_types};
use crate::mock_defaults::auto_mock_spec;
use daglang_emit::test_mock_emit::{TestFile, TERMINAL_NODE_SENTINEL};
use daglang_syntax::ast::{Annotation, ExpectStmt, Expr, FixtureDef, Literal, TestDef};
use gunbc_codegen::registry::TestgenTargetDef;
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};
use gunbc_test::{
    BoundaryMock, ExpectedOutput, LiveExpectedOutput, MockSpec, OutputMatcher, TransportMock,
};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use super::mock_interpreter::{interpret_expr, is_transport_response};

// ── Auto-discovery: any compilable .dag file ──────────────────────────

/// A compilable `.dag` module discovered by scanning `dsl/`.
#[derive(Debug, Clone)]
pub struct CompilableModule {
    /// Relative path from dsl root (e.g., "tools/bootstrap.dag").
    pub dsl_path: String,
    /// Dot-separated module name (e.g., "tools.bootstrap").
    pub module_name: String,
    /// Number of `func` items in the module.
    pub func_count: usize,
    /// Whether the module has inline `test` blocks.
    pub has_test_blocks: bool,
    /// Interface type names imported via `import interfaces.*`.
    pub interface_imports: HashSet<String>,
    /// Whether the module imports from `interfaces.*` (requires `--profile` to compile).
    pub requires_profile: bool,
}

/// Result of attempting auto-testgen on a module.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum AutoTestgenResult {
    /// Successfully generated test code.
    Generated {
        target_def: TestgenTargetDef,
        test_code: String,
    },
    /// Module failed to compile — skipped.
    Skipped { reason: String },
}

/// Discover all compilable `.dag` modules under `dsl_root`.
///
/// Scans recursively for `.dag` files that contain `func` items (compilation
/// units that produce DAGs with nodes). Pure-library modules with only
/// types/data/services are excluded — they're tested transitively when
/// imported by a compilable module.
#[allow(clippy::disallowed_methods)] // Needs fs access for recursive .dag discovery
pub fn discover_compilable_modules(dsl_root: &Path) -> Vec<CompilableModule> {
    let mut modules = Vec::new();
    collect_dag_files(dsl_root, dsl_root, &mut modules);
    modules.sort_by(|a, b| a.dsl_path.cmp(&b.dsl_path));
    modules
}

#[allow(clippy::disallowed_methods)]
fn collect_dag_files(base: &Path, dir: &Path, out: &mut Vec<CompilableModule>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(base, &path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("dag") {
            continue;
        }

        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let ast = match daglang_syntax::parser::parse(&source) {
            Ok(ast) => ast,
            Err(_) => continue,
        };

        // Count func items — only modules with funcs are compilable units
        let func_count = ast
            .items
            .iter()
            .filter(|item| matches!(item.node, daglang_syntax::ast::Item::FuncDef(_)))
            .count();

        if func_count == 0 {
            continue;
        }

        let has_test_blocks = ast
            .items
            .iter()
            .any(|item| matches!(item.node, daglang_syntax::ast::Item::TestDef(_)));

        // Collect interface type names from `import interfaces.*` statements.
        let interface_imports: HashSet<String> = ast
            .imports
            .iter()
            .filter(|import| {
                import
                    .node
                    .path
                    .segments
                    .first()
                    .is_some_and(|s| s == "interfaces")
            })
            .flat_map(|import| {
                import
                    .node
                    .bindings
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .cloned()
            })
            .collect();

        let requires_profile = !interface_imports.is_empty();

        // Build relative path from dsl root
        let rel_path = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let module_name = rel_path
            .strip_suffix(".dag")
            .unwrap_or(&rel_path)
            .replace('/', ".");

        out.push(CompilableModule {
            dsl_path: rel_path,
            module_name,
            func_count,
            has_test_blocks,
            interface_imports,
            requires_profile,
        });
    }
}

/// Run the auto-testgen pipeline on a single compilable module.
///
/// Pipeline: compile → auto_mock_spec → generate_target. Zero manual input.
/// Returns `Skipped` if compilation fails (graceful degradation).
pub fn auto_testgen_for_module(
    module: &CompilableModule,
    output_dir: &Path,
) -> AutoTestgenResult {
    // 1. Compile to Dag<DynOp> + DSL type registry
    let result = match build_dsl_graph_with_types(&module.dsl_path) {
        Ok(result) => result,
        Err(e) => {
            return AutoTestgenResult::Skipped {
                reason: format!("compile error: {e}"),
            };
        }
    };

    // 2. Auto-generate MockSpec from types + DAG structure
    let safe_name = module.module_name.replace('.', "-");
    let spec = auto_mock_spec(&result.dag, &safe_name);

    // 3. Build TestgenTargetDef
    let output_path = format!(
        "{}/generated_tests_{}.rs",
        output_dir.display(),
        module.module_name.replace('.', "_"),
    );
    let module_test_name = format!("{}_generated_tests", module.module_name.replace('.', "_"));
    let dag_builder_call = format!(
        "crate::dsl_builder::build_dsl_graph(\"{}\").expect(\"graph should build\")",
        module.dsl_path,
    );
    let mock_spec_path = format!(
        "crate::mock_defaults::auto_mock_spec(&dag, \"{}\")",
        safe_name,
    );

    let target_def = TestgenTargetDef {
        name: Cow::Owned(safe_name.clone()),
        output_path: Cow::Owned(output_path),
        module_name: Cow::Owned(module_test_name),
        mock_spec_path: Cow::Owned(mock_spec_path),
        dag_builder_call: Cow::Owned(dag_builder_call),
        signature_path: None,
        boundary_tests: true,
        chain_tests: true,
        flow_tests: true,
        live_flow_tests: false,
        window_max_nodes: None,
        test_class: None,
        fermi_cost: None,
        requires: None,
        secrets: None,
        live_test_class: None,
        live_fermi_cost: None,
        live_requires: None,
        live_required: None,
        live_required_any_of: None,
        tool_name: None,
        live_profile_tests: Vec::new(),
    };

    // 4. Generate test code via the shared codegen path with DSL type awareness
    let test_code = gunbc_testgen_registry::generate_target_with_types(
        &target_def,
        result.dag,
        spec,
        Some(&result.dsl_type_registry),
    );

    AutoTestgenResult::Generated {
        target_def,
        test_code,
    }
}

// ── Test-block discovery (legacy path) ────────────────────────────────

/// A discovered test target from a `.dag` file.
#[derive(Debug)]
pub struct DagTestTarget {
    /// Relative DSL module path (e.g., "tools/bootstrap.dag").
    pub dsl_module: String,
    /// Test name (e.g., "bootstrap_dryrun").
    pub test_name: String,
    /// Parsed test definition from the AST.
    pub test_def: TestDef,
    /// Fixtures from the same `.dag` file.
    pub fixtures: BTreeMap<String, FixtureDef>,
}

/// Discover all `.dag` files in `dsl/tools/` with inline test blocks.
///
/// Returns one `DagTestTarget` per non-skipped test block found.
#[allow(clippy::disallowed_methods)] // Needs fs access for .dag file discovery
pub fn discover_dag_tests(dsl_root: &Path) -> Vec<DagTestTarget> {
    let tools_dir = dsl_root.join("tools");
    let mut targets = Vec::new();

    let entries = match std::fs::read_dir(&tools_dir) {
        Ok(entries) => entries,
        Err(_) => return targets,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("dag") {
            continue;
        }

        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let ast = match daglang_syntax::parser::parse(&source) {
            Ok(ast) => ast,
            Err(_) => continue,
        };

        let test_file = TestFile::from_source(&ast);
        if test_file.tests.is_empty() {
            continue;
        }

        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let dsl_module = format!("tools/{file_stem}.dag");

        for test in &test_file.tests {
            // Skip tests marked with @testgen_skip(true)
            if find_annotation_bool(&test.annotations, "testgen_skip").unwrap_or(false) {
                continue;
            }

            targets.push(DagTestTarget {
                dsl_module: dsl_module.clone(),
                test_name: test.name.clone(),
                test_def: test.clone(),
                fixtures: test_file.fixtures.clone(),
            });
        }
    }

    targets.sort_by(|a, b| a.test_name.cmp(&b.test_name));
    targets
}

/// Build a `MockSpec` from a parsed test block by interpreting mock expressions at runtime.
///
/// 1. Auto-mocks all boundary/transport nodes
/// 2. Applies fixture mocks (if test references a fixture)
/// 3. Applies test-local mock overrides
/// 4. Applies input declarations
/// 5. Applies expect assertions
///
/// Mock node IDs from `.dag` files are unqualified (e.g., `load_manifest/execute`).
/// The lowered DAG uses module-qualified IDs (e.g., `tools.deps::load_manifest/execute`).
/// This function qualifies mock node IDs with the module prefix so they match.
pub fn build_mock_spec_from_test<T: gunbc_exec::Executable + Clone + Send>(
    dag: &Dag<T>,
    target: &DagTestTarget,
) -> MockSpec {
    let test_name = target.test_def.name.replace('_', "-");
    let mut spec = auto_mock_spec(dag, &test_name);
    let module_prefix = module_prefix_from_dsl_module(&target.dsl_module);

    // Apply fixture mocks
    if let Some(fixture_name) = &target.test_def.fixture {
        if let Some(fixture) = target.fixtures.get(fixture_name) {
            for mock in &fixture.mocks {
                apply_mock(&mut spec, mock, &module_prefix, dag);
            }
        }
    }

    // Apply test-local mocks
    for mock in &target.test_def.mocks {
        apply_mock(&mut spec, mock, &module_prefix, dag);
    }

    // Apply input declarations
    for input in &target.test_def.inputs {
        let raw_node_id = input.node_segments.join("/");
        let node_id = qualify_node_id(&raw_node_id, &module_prefix, dag);
        let value = interpret_expr(&input.value);
        spec = spec.input_mock(&node_id, &input.port, value);
    }

    // Apply expect assertions
    for expect in &target.test_def.expects {
        apply_expect(&mut spec, expect, &module_prefix, dag);
    }

    spec
}

/// Build a `TestgenTargetDef` for a discovered test target.
///
/// All override fields are set to `None` — tier, hermetic, fermi are inferred
/// from DAG topology by `generate_target()`.
pub fn build_testgen_target_def(
    target: &DagTestTarget,
    output_dir: &Path,
    dag_builder_call: &str,
) -> TestgenTargetDef {
    let test_name = target.test_def.name.replace('_', "-");
    let output_path = format!(
        "{}/generated_tests_{}.rs",
        output_dir.display(),
        target.test_def.name
    );
    let module_name = format!("{}_generated_tests", target.test_def.name);

    // mock_spec_path: use auto_mock_spec as the runtime function.
    // Test-specific overrides are baked into the MockSpec at generation time,
    // but flow tests calling this path will only get auto-mocked values.
    let mock_spec_path = format!(
        "crate::mock_defaults::auto_mock_spec(&dag, \"{test_name}\")"
    );

    TestgenTargetDef {
        name: Cow::Owned(test_name),
        output_path: Cow::Owned(output_path),
        module_name: Cow::Owned(module_name),
        mock_spec_path: Cow::Owned(mock_spec_path),
        dag_builder_call: Cow::Owned(dag_builder_call.to_string()),
        signature_path: None,
        boundary_tests: true,
        chain_tests: true,
        flow_tests: true,
        live_flow_tests: false,
        window_max_nodes: None,
        test_class: None,     // inferred from topology
        fermi_cost: None,     // inferred from topology
        requires: None,
        secrets: None,
        live_test_class: None,
        live_fermi_cost: None,
        live_requires: None,
        live_required: None,
        live_required_any_of: None,
        tool_name: None,
        live_profile_tests: Vec::new(),
    }
}

/// Compile a DAG from a `.dag` module path for testgen.
///
/// Uses the existing `build_dsl_graph` from `dsl_builder.rs`.
pub fn compile_dag_for_test(dsl_module: &str) -> Result<Dag<DynOp>, BuilderError> {
    build_dsl_graph(dsl_module)
}

/// Derive the dag_builder_call Rust expression for a `.dag` module.
///
/// This is the expression emitted in generated test code so tests can rebuild
/// the DAG at test runtime.
pub fn dag_builder_call_for_module(dsl_module: &str) -> String {
    // Map dsl module path to the corresponding Rust builder function.
    // These are defined in gunbc-dag/src/dsl_builder.rs.
    let stem = dsl_module
        .strip_prefix("tools/")
        .and_then(|s| s.strip_suffix(".dag"))
        .unwrap_or(dsl_module);

    match stem {
        "testgen" => "crate::testgen_dag::graph::build_testgen_graph_auto().expect(\"graph should build\")".to_string(),
        _ => format!("crate::dsl_builder::build_dsl_graph(\"{dsl_module}\").expect(\"graph should build\")"),
    }
}

// ── Internal helpers ────────────────────────────────────────────────

/// Derive the module prefix from a dsl_module path.
///
/// `"tools/deps.dag"` → `"tools.deps"` (used to qualify node IDs).
fn module_prefix_from_dsl_module(dsl_module: &str) -> String {
    dsl_module
        .strip_suffix(".dag")
        .unwrap_or(dsl_module)
        .replace('/', ".")
}

/// Qualify a node ID with the module prefix if the unqualified ID isn't in the DAG.
///
/// Tries unqualified first, then `{module_prefix}::{node_id}`.
fn qualify_node_id<T>(raw_node_id: &str, module_prefix: &str, dag: &Dag<T>) -> String {
    // Already qualified or exists as-is?
    if dag.nodes.iter().any(|n| n.id.0 == raw_node_id) {
        return raw_node_id.to_string();
    }
    // Try with module prefix
    let qualified = format!("{module_prefix}::{raw_node_id}");
    if dag.nodes.iter().any(|n| n.id.0 == qualified) {
        return qualified;
    }
    // Fall back to qualified (will be caught by validation later)
    qualified
}

fn apply_mock<T>(
    spec: &mut MockSpec,
    mock: &daglang_syntax::ast::MockDecl,
    module_prefix: &str,
    dag: &Dag<T>,
) {
    let raw_node_id = mock.node_segments.join("/");
    let node_id = qualify_node_id(&raw_node_id, module_prefix, dag);
    let value = interpret_expr(&mock.value);

    if is_transport_response(&mock.value) {
        spec.transport_mocks.push(TransportMock {
            node: node_id,
            port: mock.port.clone(),
            value,
        });
    } else {
        spec.boundary_mocks.push(BoundaryMock {
            node: node_id,
            port: mock.port.clone(),
            value,
            sequence: None,
        });
    }
}

fn apply_expect<T>(
    spec: &mut MockSpec,
    expect: &ExpectStmt,
    module_prefix: &str,
    dag: &Dag<T>,
) {
    match expect {
        ExpectStmt::Eq(lhs, rhs) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                let node = qualify_node_id(&node, module_prefix, dag);
                let value = interpret_expr(rhs);
                spec.expected_outputs.push(ExpectedOutput {
                    node,
                    port,
                    expected: value,
                });
            }
        }
        ExpectStmt::Contains(lhs, rhs) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                let node = qualify_node_id(&node, module_prefix, dag);
                let substr = expr_to_string(rhs);
                spec.live_expected_outputs.push(LiveExpectedOutput {
                    node,
                    port,
                    matcher: OutputMatcher::contains(&substr),
                });
            }
        }
        ExpectStmt::Is(lhs, type_name) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                let node = qualify_node_id(&node, module_prefix, dag);
                let matcher = match type_name.as_str() {
                    "String" => OutputMatcher::IsString,
                    "Bool" => OutputMatcher::IsBool,
                    "Int" => OutputMatcher::IsInt,
                    "NonEmpty" => OutputMatcher::NonEmpty,
                    _ => OutputMatcher::Any,
                };
                spec.live_expected_outputs.push(LiveExpectedOutput {
                    node,
                    port,
                    matcher,
                });
            }
        }
        ExpectStmt::Truthy(lhs) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                let node = qualify_node_id(&node, module_prefix, dag);
                spec.live_expected_outputs.push(LiveExpectedOutput {
                    node,
                    port,
                    matcher: OutputMatcher::NonEmpty,
                });
            }
        }
        // Comparison operators: use NonEmpty as a baseline
        ExpectStmt::Ne(lhs, _)
        | ExpectStmt::Lt(lhs, _)
        | ExpectStmt::Gt(lhs, _)
        | ExpectStmt::Le(lhs, _)
        | ExpectStmt::Ge(lhs, _) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                let node = qualify_node_id(&node, module_prefix, dag);
                spec.live_expected_outputs.push(LiveExpectedOutput {
                    node,
                    port,
                    matcher: OutputMatcher::NonEmpty,
                });
            }
        }
    }
}

/// Extract (node_id, port_name) from a `result.field` expression.
///
/// Mirrors `daglang_emit::test_mock_emit::extract_result_path`.
fn extract_result_path(expr: &Expr) -> Option<(String, String)> {
    match expr {
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(name) = base.as_ref() {
                if name == "result" {
                    return Some((TERMINAL_NODE_SENTINEL.to_string(), field.clone()));
                }
                return Some((name.clone(), field.clone()));
            }
            if let Some((parent_node, parent_port)) = extract_result_path(base) {
                return Some((format!("{parent_node}/{parent_port}"), field.clone()));
            }
            None
        }
        _ => None,
    }
}

fn find_annotation_bool(annotations: &[Annotation], name: &str) -> Option<bool> {
    annotations.iter().find(|a| a.name == name).and_then(|a| {
        a.args.first().and_then(|arg| match arg {
            Expr::Literal(Literal::Bool(b)) => Some(*b),
            Expr::Ident(s) if s == "true" => Some(true),
            Expr::Ident(s) if s == "false" => Some(false),
            _ => None,
        })
    })
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::String(s)) => s.clone(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_compilable_modules_finds_tools() {
        let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout");
        let dsl_root = layout.workspace_root.join("dsl");
        let modules = discover_compilable_modules(&dsl_root);

        // Should find at least all 14 tools + workflows + pipelines
        assert!(
            modules.len() > 14,
            "expected >14 compilable modules, found {}",
            modules.len()
        );

        // All tools should be discovered
        let names: Vec<&str> = modules.iter().map(|m| m.module_name.as_str()).collect();
        assert!(names.contains(&"tools.bootstrap"), "missing tools.bootstrap");
        assert!(names.contains(&"tools.makegen"), "missing tools.makegen");
        assert!(names.contains(&"tools.pragma"), "missing tools.pragma");

        // Non-tool compilable modules should be discovered (e.g., cloud credentials, funcs)
        let has_non_tools = names.iter().any(|n| !n.starts_with("tools."));
        assert!(has_non_tools, "only tool modules discovered — should include cloud/funcs/etc");

        // Pure library modules (std.types, std.symbols, etc.) should be excluded
        // because they have no func items
        assert!(
            !names.contains(&"std.types"),
            "std.types should be excluded (no func items)"
        );

        // Every module should have func_count > 0
        for module in &modules {
            assert!(
                module.func_count > 0,
                "{} has func_count=0",
                module.module_name
            );
        }
    }

    #[test]
    fn auto_testgen_for_makegen_produces_tests() {
        let module = CompilableModule {
            dsl_path: "tools/makegen.dag".to_string(),
            module_name: "tools.makegen".to_string(),
            func_count: 1,
            has_test_blocks: true,
            interface_imports: HashSet::new(),
            requires_profile: false,
        };
        let output_dir = std::path::Path::new("gunbc-dag/src");
        let result = auto_testgen_for_module(&module, output_dir);
        match result {
            AutoTestgenResult::Generated { test_code, .. } => {
                assert!(
                    test_code.contains("#[test]"),
                    "generated code should contain test functions"
                );
                assert!(
                    test_code.contains("test_dryrun_completion"),
                    "generated code should contain DryRun completion test"
                );
            }
            AutoTestgenResult::Skipped { reason } => {
                panic!("makegen should compile, but got: {reason}");
            }
        }
    }

    #[test]
    fn auto_testgen_skips_uncompilable_module() {
        let module = CompilableModule {
            dsl_path: "nonexistent/fake.dag".to_string(),
            module_name: "nonexistent.fake".to_string(),
            func_count: 1,
            has_test_blocks: false,
            interface_imports: HashSet::new(),
            requires_profile: false,
        };
        let output_dir = std::path::Path::new("gunbc-dag/src");
        let result = auto_testgen_for_module(&module, output_dir);
        assert!(
            matches!(result, AutoTestgenResult::Skipped { .. }),
            "nonexistent module should be skipped"
        );
    }

    #[test]
    fn dag_builder_call_known_modules() {
        let call = dag_builder_call_for_module("tools/bootstrap.dag");
        assert!(call.contains("build_dsl_graph"));
        assert!(call.contains("expect"));
    }

    #[test]
    fn dag_builder_call_unknown_module_falls_back() {
        let call = dag_builder_call_for_module("tools/unknown.dag");
        assert!(call.contains("build_dsl_graph"));
        assert!(call.contains("unknown.dag"));
    }

    #[test]
    fn comprehensive_auto_testgen_pipeline_validation() {
        let layout = gunbc_ir::WorkspaceLayout::from_env_manifest_dir()
            .or_else(|_| gunbc_ir::WorkspaceLayout::from_cargo_metadata())
            .expect("workspace layout");
        let dsl_root = layout.workspace_root.join("dsl");
        let output_dir = std::path::Path::new("/tmp/testgen_validation");

        // Phase 1: Discover all compilable modules
        let modules = discover_compilable_modules(&dsl_root);
        let total_discovered = modules.len();

        eprintln!("\n========================================");
        eprintln!("  Auto-Testgen Pipeline Validation");
        eprintln!("========================================\n");
        eprintln!("Discovered {} compilable .dag modules:\n", total_discovered);

        for (i, m) in modules.iter().enumerate() {
            eprintln!(
                "  {:>2}. {:<40} funcs={} tests={}",
                i + 1,
                m.module_name,
                m.func_count,
                if m.has_test_blocks { "yes" } else { "no" }
            );
        }

        // Phase 2: Run auto_testgen_for_module on each
        let mut generated = Vec::new();
        let mut skipped = Vec::new();

        for module in &modules {
            let result = auto_testgen_for_module(module, output_dir);
            match result {
                AutoTestgenResult::Generated { test_code, target_def: _ } => {
                    let test_fn_count = test_code.matches("#[test]").count();
                    generated.push((module.module_name.clone(), test_fn_count, test_code.len()));
                }
                AutoTestgenResult::Skipped { reason } => {
                    skipped.push((module.module_name.clone(), reason));
                }
            }
        }

        let total_generated = generated.len();
        let total_skipped = skipped.len();

        // Phase 3: Report
        eprintln!("\n--- Generated ({}) ---\n", total_generated);
        let mut total_test_fns = 0;
        let mut total_code_bytes = 0;
        for (name, test_fn_count, code_len) in &generated {
            total_test_fns += test_fn_count;
            total_code_bytes += code_len;
            eprintln!(
                "  [OK]  {:<40} {} test fns, {} bytes",
                name, test_fn_count, code_len
            );
        }

        if !skipped.is_empty() {
            eprintln!("\n--- Skipped ({}) ---\n", total_skipped);
            for (name, reason) in &skipped {
                eprintln!("  [SKIP] {:<40} {}", name, reason);
            }
        }

        eprintln!("\n--- Summary ---\n");
        eprintln!("  Total discovered:   {}", total_discovered);
        eprintln!("  Total generated:    {}", total_generated);
        eprintln!("  Total skipped:      {}", total_skipped);
        eprintln!("  Total test fns:     {}", total_test_fns);
        eprintln!("  Total code bytes:   {}", total_code_bytes);
        eprintln!("  Success rate:       {:.1}%", (total_generated as f64 / total_discovered as f64) * 100.0);
        eprintln!("\n========================================\n");

        // Assertions
        assert!(
            total_discovered > 14,
            "expected >14 compilable modules, found {}",
            total_discovered
        );
        assert!(
            total_generated > 0,
            "expected at least 1 module to generate tests"
        );
        // Every generated module should produce at least 1 #[test] fn
        for (name, count, _) in &generated {
            assert!(
                *count > 0,
                "{} generated 0 test functions",
                name
            );
        }
    }
}
