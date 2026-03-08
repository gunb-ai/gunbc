//! DAG test discovery and auto-testgen pipeline.
//!
//! Two discovery modes:
//!
//! 1. **Auto-discovery** (`discover_compilable_modules`): Scans all of `dsl/`
//!    recursively for `.dag` files with callable items (`func`, `fn`, `pattern`,
//!    `pipeline`). Every compilable module gets full testgen treatment via
//!    `auto_mock_spec()` — zero manual input.
//!
//! 2. **Test-block discovery** (`discover_dag_tests`): Legacy path that scans
//!    `dsl/tools/*.dag` for inline `test` blocks. Provides fixture overrides
//!    on top of auto-mocked defaults.
//!
//! Tier, hermetic, and fermi metadata are inferred from DAG topology by
//! `generate_target()`, not declared statically in annotations.

use crate::registry::TestgenTargetDef;
use daglang_emit::test_mock_emit::{TestFile, TERMINAL_NODE_SENTINEL};
use daglang_syntax::ast::{ExpectStmt, Expr, FixtureDef, Literal, TestDef};
use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};
use gunbc_resolve::{builder::build_dsl_graph, BuildOpts};
use gunbc_test::{auto_mock_failure_variants, auto_mock_spec};
use gunbc_test::{
    BoundaryMock, ExpectedOutput, FermiCost, LiveExpectedOutput, MockSpec, OutputMatcher,
    TestClass, TransportMock,
};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;

use super::mock_interpreter::{interpret_expr, is_transport_response};

fn build_gunbc_dsl_graph(
    relative_module: &str,
    opts: BuildOpts<'_>,
) -> Result<gunbc_resolve::DslGraphResult, BuilderError> {
    build_dsl_graph(relative_module, opts)
}

// ── Auto-discovery: any compilable .dag file ──────────────────────────

/// A compilable `.dag` module discovered by scanning `dsl/`.
#[derive(Debug, Clone)]
pub struct CompilableModule {
    /// Relative path from dsl root (e.g., "tools/bootstrap.dag").
    pub dsl_path: String,
    /// Dot-separated module name (e.g., "tools.bootstrap").
    pub module_name: String,
    /// Number of callable items in the module (func, fn, pattern, pipeline).
    ///
    /// Mirrors `module_has_callable_items()` in `daglang-driver/src/lib.rs` —
    /// the canonical set of item types that produce executable DAGs.
    /// If a new callable item type is added to the AST, both must be updated.
    pub callable_count: usize,
    /// Whether the module has inline `test` blocks.
    pub has_test_blocks: bool,
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

/// Rendered output for one auto-generated test module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedTestgenModule {
    pub content: String,
    pub path: String,
}

/// Discover all compilable `.dag` modules under `dsl_root`.
///
/// Scans recursively for `.dag` files containing callable items — `fn`, `func`,
/// `pattern`, or `pipeline` — that produce executable DAGs with nodes.
/// Pure-library modules with only types/data/services are excluded since
/// they're tested transitively when imported by a compilable module.
#[allow(clippy::disallowed_methods)] // Needs fs access for recursive .dag discovery
pub fn discover_compilable_modules(dsl_root: &Path) -> Vec<CompilableModule> {
    let mut modules = Vec::new();
    collect_dag_files(dsl_root, dsl_root, &mut modules);
    modules.sort_by(|a, b| a.dsl_path.cmp(&b.dsl_path));
    modules
}

/// Discover one compilable `.dag` module by relative path from `dsl_root`.
pub fn find_compilable_module(dsl_root: &Path, dsl_path: &str) -> Option<CompilableModule> {
    analyze_compilable_module(dsl_root, &dsl_root.join(dsl_path))
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
        if let Some(module) = analyze_compilable_module(base, &path) {
            out.push(module);
        }
    }
}

#[allow(clippy::disallowed_methods)]
fn analyze_compilable_module(base: &Path, path: &Path) -> Option<CompilableModule> {
    if path.extension().and_then(|e| e.to_str()) != Some("dag") {
        return None;
    }

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: cannot read {}: {e}", path.display());
            return None;
        }
    };
    let ast = match daglang_syntax::parser::parse(&source) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("warning: cannot parse {}: {e:?}", path.display());
            return None;
        }
    };

    // Count callable items — these are the item types that produce executable DAGs.
    // Mirrors `module_has_callable_items()` in `daglang-driver/src/lib.rs`.
    use daglang_syntax::ast::Item;
    let callable_count = ast
        .items
        .iter()
        .filter(|item| {
            matches!(
                item.node,
                Item::FnDef(_) | Item::FuncDef(_) | Item::PatternDef(_) | Item::PipelineDef(_)
            )
        })
        .count();

    if callable_count == 0 {
        return None;
    }

    let has_test_blocks = ast
        .items
        .iter()
        .any(|item| matches!(item.node, daglang_syntax::ast::Item::TestDef(_)));

    let rel_path = path
        .strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    Some(CompilableModule {
        module_name: rel_path
            .strip_suffix(".dag")
            .unwrap_or(&rel_path)
            .replace('/', "."),
        dsl_path: rel_path,
        callable_count,
        has_test_blocks,
    })
}

/// Run the auto-testgen pipeline on a single compilable module.
///
/// Pipeline: compile → auto_mock_spec → generate_target. Zero manual input.
/// Returns `Skipped` if compilation fails (graceful degradation).
///
pub fn auto_testgen_for_module(module: &CompilableModule, output_dir: &Path) -> AutoTestgenResult {
    let result = match build_gunbc_dsl_graph(&module.dsl_path, BuildOpts::default()) {
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

    // 2a. Derive failure-variant MockSpecs for error-path test generation (RV-4)
    let failure_variants = auto_mock_failure_variants(&spec);

    // 2b. Classify the module via DSL-evaluated fidelity policy
    let classification = crate::fidelity::classify_module(&result.callable_properties);
    let all_transport_classes: Vec<_> = result
        .callable_properties
        .values()
        .flat_map(|p| p.transport_classes.iter().cloned())
        .collect();
    let requires = crate::fidelity::requires_from_transport_classes(&all_transport_classes);

    // 3. Build TestgenTargetDef
    let output_path = output_path_for_module(output_dir, module);
    let module_test_name = format!("{}_generated_tests", module.module_name.replace('.', "_"));

    let dag_builder_call = format!(
        "gunbc_resolve::builder::build_dsl_graph_dag(\"{}\", gunbc_resolve::BuildOpts::default()).expect(\"graph should build\")",
        module.dsl_path,
    );
    let mock_spec_path = format!("gunbc_test::auto_mock_spec(&dag, \"{}\")", safe_name,);

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
        test_class: classification.test_class,
        fermi_cost: classification.fermi_cost,
        requires,
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
    let test_code = crate::testgen::generate_target_full(
        &target_def,
        result.dag,
        spec,
        Some(&result.dsl_type_registry),
        failure_variants,
    );

    AutoTestgenResult::Generated {
        target_def,
        test_code,
    }
}

/// Render generated test content for one module, including placeholder output
/// when structural compilation is not currently possible.
pub fn render_auto_testgen_for_module(
    module: &CompilableModule,
    output_dir: &Path,
) -> RenderedTestgenModule {
    match auto_testgen_for_module(module, output_dir) {
        AutoTestgenResult::Generated {
            target_def,
            test_code,
        } => RenderedTestgenModule {
            content: test_code,
            path: target_def.output_path.into_owned(),
        },
        AutoTestgenResult::Skipped { reason } => {
            let commented_reason = reason
                .lines()
                .map(|line| format!("// {line}"))
                .collect::<Vec<_>>()
                .join("\n");
            RenderedTestgenModule {
                content: format!(
                    "// Auto-testgen skipped for '{}':\n{commented_reason}\n",
                    module.module_name,
                ),
                path: output_path_for_module(output_dir, module),
            }
        }
    }
}

/// Compute the generated Rust test path for a compilable module.
pub fn output_path_for_module(output_dir: &Path, module: &CompilableModule) -> String {
    format!(
        "{}/generated_tests_{}.rs",
        output_dir.display(),
        module.module_name.replace('.', "_"),
    )
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
            // Skip tests marked with skip
            if test.skip {
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
    let mock_spec_path = format!("gunbc_test::auto_mock_spec(&dag, \"{test_name}\")");

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
        test_class: TestClass::Unit,
        fermi_cost: FermiCost::XS,
        requires: Vec::new(),
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
pub fn compile_dag_for_test(dsl_module: &str) -> Result<Dag<DynOp>, BuilderError> {
    build_gunbc_dsl_graph(dsl_module, BuildOpts::default()).map(|result| result.dag)
}

/// Derive the dag_builder_call Rust expression for a `.dag` module.
///
/// This is the expression emitted in generated test code so tests can rebuild
/// the DAG at test runtime.
pub fn dag_builder_call_for_module(dsl_module: &str) -> String {
    format!(
        "gunbc_resolve::builder::build_dsl_graph_dag(\"{dsl_module}\", gunbc_resolve::BuildOpts::default()).expect(\"graph should build\")"
    )
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

fn apply_expect<T>(spec: &mut MockSpec, expect: &ExpectStmt, module_prefix: &str, dag: &Dag<T>) {
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
                    "Secret" => OutputMatcher::IsSecret,
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

        // Should find at least the 6 tool modules + some std/extdeps with fn items
        assert!(
            modules.len() >= 6,
            "expected >=6 compilable modules, found {}",
            modules.len()
        );

        // All tools should be discovered
        let names: Vec<&str> = modules.iter().map(|m| m.module_name.as_str()).collect();
        assert!(
            names.contains(&"tools.bootstrap"),
            "missing tools.bootstrap"
        );

        // Pure library modules (std.types, std.symbols, etc.) should be excluded
        // because they have no func items
        assert!(
            !names.contains(&"std.types"),
            "std.types should be excluded (no func items)"
        );

        // Every module should have callable_count > 0
        for module in &modules {
            assert!(
                module.callable_count > 0,
                "{} has callable_count=0",
                module.module_name
            );
        }
    }

    #[test]
    fn auto_testgen_for_bootstrap_produces_tests() {
        let module = CompilableModule {
            dsl_path: "tools/bootstrap.dag".to_string(),
            module_name: "tools.bootstrap".to_string(),
            callable_count: 1,
            has_test_blocks: true,
        };
        let output_dir = std::path::Path::new("src/10_test/generated-tests/src");
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
                panic!("bootstrap should compile, but got: {reason}");
            }
        }
    }

    #[test]
    fn auto_testgen_includes_failure_variant_tests() {
        let module = CompilableModule {
            dsl_path: "tools/bootstrap.dag".to_string(),
            module_name: "tools.bootstrap".to_string(),
            callable_count: 1,
            has_test_blocks: true,
        };
        let output_dir = std::path::Path::new("src/10_test/generated-tests/src");
        let result = auto_testgen_for_module(&module, output_dir);
        match result {
            AutoTestgenResult::Generated { test_code, .. } => {
                // Bootstrap has transport boundaries, so failure variants should be generated.
                // Check for the RV-4 section header.
                assert!(
                    test_code.contains("RV-4: Failure Variant Tests"),
                    "generated code should contain failure variant test section"
                );
                // Check for at least one failure test function.
                assert!(
                    test_code.contains("test_failure_"),
                    "generated code should contain at least one failure variant test"
                );
                // Check that the test calls auto_mock_failure_variants at runtime.
                assert!(
                    test_code.contains("auto_mock_failure_variants"),
                    "generated code should call auto_mock_failure_variants"
                );
            }
            AutoTestgenResult::Skipped { reason } => {
                panic!("bootstrap should compile, but got: {reason}");
            }
        }
    }

    #[test]
    fn auto_testgen_skips_uncompilable_module() {
        let module = CompilableModule {
            dsl_path: "nonexistent/fake.dag".to_string(),
            module_name: "nonexistent.fake".to_string(),
            callable_count: 1,
            has_test_blocks: false,
        };
        let output_dir = std::path::Path::new("src/10_test/generated-tests/src");
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
}
