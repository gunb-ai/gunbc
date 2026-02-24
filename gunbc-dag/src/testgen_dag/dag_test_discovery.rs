//! Direct .dag test block discovery and MockSpec construction.
//!
//! Scans `dsl/tools/*.dag` files at runtime, parses inline test/fixture blocks,
//! and builds `MockSpec` + `TestgenTargetDef` for each test — without generating
//! intermediate `graph_mock.rs` files.
//!
//! Tier, hermetic, and fermi metadata are inferred from DAG topology by
//! `generate_target()`, not declared statically in annotations.

use crate::dsl_builder::build_dsl_graph;
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
use std::collections::BTreeMap;
use std::path::Path;

use super::mock_interpreter::{interpret_expr, is_transport_response};

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
        "bootstrap" => "crate::dsl_builder::build_bootstrap_graph_dsl().expect(\"graph should build\")".to_string(),
        "clippy" => "crate::build_clippy_graph_dsl().expect(\"graph should build\")".to_string(),
        "codegen" => "crate::dsl_builder::build_codegen_graph_dsl().expect(\"graph should build\")".to_string(),
        "deps" => "crate::dsl_builder::build_deps_graph_dsl().expect(\"graph should build\")".to_string(),
        "gist_snapshot" => "crate::build_gist_snapshot_graph_dsl().expect(\"graph should build\")".to_string(),
        "gist" => "crate::build_gist_diff_graph_dsl().expect(\"graph should build\")".to_string(),
        "makegen" => "crate::dsl_builder::build_makegen_graph_dsl().expect(\"graph should build\")".to_string(),
        "pragma" => "crate::dsl_builder::build_pragma_graph_dsl().expect(\"graph should build\")".to_string(),
        "review" => "crate::build_review_graph_dsl().expect(\"graph should build\")".to_string(),
        "testgen" => "crate::testgen_dag::graph::build_testgen_graph_for_test().expect(\"graph should build\")".to_string(),
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
    fn dag_builder_call_known_modules() {
        let call = dag_builder_call_for_module("tools/bootstrap.dag");
        assert!(call.contains("build_bootstrap_graph_dsl"));
        assert!(call.contains("expect"));
    }

    #[test]
    fn dag_builder_call_unknown_module_falls_back() {
        let call = dag_builder_call_for_module("tools/unknown.dag");
        assert!(call.contains("build_dsl_graph"));
        assert!(call.contains("unknown.dag"));
    }
}
