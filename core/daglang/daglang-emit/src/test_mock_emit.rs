//! Test mock emitter: generates `graph_mock.rs` Rust code from inline test blocks.
//!
//! Each `test` block in a `.dag` file becomes a function that returns
//! a `MockSpec`, decorated with `#[testgen_target]` and
//! `#[resource_test_target]` proc macros.
//!
//! Each `fixture` block becomes a helper function that applies shared mocks
//! to a `MockSpec`, callable from test functions.
//!
//! # Example
//!
//! ```dag
//! // bootstrap.dag (tests at bottom)
//! fixture cloud_env {
//!     mock cloud_env.config -> { project: "mock-project" }
//! }
//!
//! test bootstrap_dryrun : cloud_env {
//!     tier: Unit
//!     hermetic
//!     mock execute.response -> rest_response(200, { ok: true })
//!     expect result.ok == true
//! }
//! ```
//!
//! Generates:
//!
//! ```text
//! fn apply_cloud_env(spec: MockSpec) -> MockSpec {
//!     spec.boundary("cloud_env", "config", Value::Str(r#"{"project":"mock-project"}"#.into()))
//! }
//!
//! #[testgen_target(name = "bootstrap-dryrun", ...)]
//! pub fn bootstrap_dryrun_mock_spec() -> MockSpec {
//!     let dag = build_graph().expect("graph should build");
//!     let mut spec = auto_mock_spec(&dag, "bootstrap-dryrun");
//!     spec = apply_cloud_env(spec);
//!     spec = spec.transport_mock("execute", "response", ...);
//!     spec
//! }
//! ```

use daglang_syntax::ast::{
    ExpectStmt, Expr, FixtureDef, InputDecl, Item, Literal, MockDecl, SourceFile, TestDef,
};
use std::collections::BTreeMap;
use std::fmt::Write;

/// Sentinel node ID emitted for `expect result.*` assertions.
///
/// When a test uses `expect result.field ...`, the emitter cannot resolve
/// which DAG node is the terminal (it sees only the AST, not the lowered
/// DAG topology). Testgen resolves this sentinel to the actual terminal
/// node at code generation time.
///
/// Eliminating this sentinel entirely would require passing DAG topology
/// into the test emitter, which means lowering each module during test
/// extraction — a heavier operation reserved for a future refactor.
pub const TERMINAL_NODE_SENTINEL: &str = "__terminal__";

/// Configuration for test mock emission.
#[derive(Debug, Clone)]
pub struct TestEmitConfig {
    /// The Rust expression to build the DAG (e.g., `crate::build_bootstrap_graph().unwrap()`).
    pub dag_builder: String,
    /// The Rust path to the auto_mock_spec function.
    pub auto_mock_fn: String,
    /// The output file path for testgen (relative to workspace root).
    pub output_dir: String,
    /// The tool name for testgen target.
    pub tool_name: Option<String>,
    /// Optional Rust expression for the CLI signature.
    pub signature_fn: Option<String>,
}

/// A parsed test file containing fixtures and test cases.
#[derive(Debug)]
pub struct TestFile {
    /// Fixtures keyed by name. `BTreeMap` for deterministic emission order.
    pub fixtures: BTreeMap<String, FixtureDef>,
    pub tests: Vec<TestDef>,
}

impl TestFile {
    /// Extract fixtures and tests from a parsed source file.
    pub fn from_source(source: &SourceFile) -> Self {
        let mut fixtures = BTreeMap::new();
        let mut tests = Vec::new();
        for item in &source.items {
            match &item.node {
                Item::FixtureDef(f) => {
                    fixtures.insert(
                        f.name.clone(),
                        FixtureDef {
                            name: f.name.clone(),
                            mocks: f.mocks.clone(),
                        },
                    );
                }
                Item::TestDef(t) => {
                    tests.push(TestDef {
                        name: t.name.clone(),
                        fixture: t.fixture.clone(),
                        mocks: t.mocks.clone(),
                        inputs: t.inputs.clone(),
                        expects: t.expects.clone(),
                        tier: t.tier.clone(),
                        hermetic: t.hermetic,
                        skip: t.skip,
                        auto_mock: t.auto_mock,
                        mock_helpers: t.mock_helpers.clone(),
                    });
                }
                _ => {} // Ignore non-test items (types, imports, etc.)
            }
        }
        TestFile { fixtures, tests }
    }
}

/// Emit a complete `graph_mock.rs` file from a test file.
pub fn emit_test_mock_file(test_file: &TestFile, config: &TestEmitConfig) -> String {
    let mut out = String::new();

    // Header
    writeln!(
        out,
        "//! Generated from inline test blocks — do not edit by hand."
    )
    .unwrap();
    writeln!(out).unwrap();
    writeln!(out, "use gunbc_test::MockSpec;").unwrap();
    writeln!(out, "use gunbc_ir::Value;").unwrap();
    writeln!(out, "use gunbc_ir::transport::{{RestResponse, ShellResponse, FileResponse, FileOp, TransportResponse}};").unwrap();
    writeln!(out).unwrap();

    // Filter out tests marked with `skip`.
    let active_tests: Vec<&TestDef> = test_file.tests.iter().filter(|t| !t.skip).collect();

    // Emit Rust mock helper annotations as documentation comments.
    let rust_helpers: Vec<&str> = active_tests
        .iter()
        .filter_map(|t| t.mock_helpers.clone())
        .collect::<Vec<_>>()
        .into_iter()
        .map(|s| s.leak() as &str) // Static lifetime for dedup — emitter runs once.
        .collect();
    if !rust_helpers.is_empty() {
        let mut seen = std::collections::BTreeSet::new();
        for helper in &rust_helpers {
            if seen.insert(*helper) {
                writeln!(out, "// Rust mock helpers: {helper}").unwrap();
            }
        }
        writeln!(out).unwrap();
    }

    // Emit fixture helpers
    for fixture in test_file.fixtures.values() {
        emit_fixture_fn(&mut out, fixture);
    }

    // Emit test functions
    for test in &active_tests {
        emit_test_fn(&mut out, test, &test_file.fixtures, config);
    }

    out
}

/// Emit a fixture helper function.
fn emit_fixture_fn(out: &mut String, fixture: &FixtureDef) {
    let fn_name = &fixture.name;
    writeln!(out, "fn apply_{fn_name}(spec: MockSpec) -> MockSpec {{").unwrap();
    writeln!(out, "    let mut spec = spec;").unwrap();
    for mock in &fixture.mocks {
        emit_mock_apply(out, mock, "    ");
    }
    writeln!(out, "    spec").unwrap();
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
}

/// Emit a test mock spec function with testgen decorators.
fn emit_test_fn(
    out: &mut String,
    test: &TestDef,
    fixtures: &BTreeMap<String, FixtureDef>,
    config: &TestEmitConfig,
) {
    let fn_name = format!("{}_mock_spec", test.name);
    let test_name = test.name.replace('_', "-");

    // Fidelity system derives tier/hermetic from graph structure.
    // Manual tier hints are accepted but only affect the testgen attribute;
    // DSL tests always run against mocked graphs (hermetic by construction).
    let flow_flag = "flow_tests";
    let class_attr = match test.tier.as_deref() {
        Some("Unit") => ", class = \"unit\", fermi = \"XS\"",
        Some("Contract") => ", class = \"hermetic\", fermi = \"S\"",
        Some("Scenario") => ", class = \"hermetic\", fermi = \"S\"",
        Some("Integration") => ", class = \"integration\", fermi = \"M\"",
        _ => "",
    };

    let output_path = format!("{}/generated_tests_{}.rs", config.output_dir, test.name);
    let module_name = format!("{}_generated_tests", test.name);

    // Normalise builder expression once: strip any trailing .unwrap() / .expect(…)
    // and apply a uniform .expect("graph should build").
    let builder_base = config
        .dag_builder
        .strip_suffix(".unwrap()")
        .or_else(|| {
            config
                .dag_builder
                .strip_suffix(".expect(\"graph should build\")")
        })
        .unwrap_or(&config.dag_builder);
    let builder_expr = format!("{builder_base}.expect(\"graph should build\")");

    // resource_test_target decorator
    writeln!(
        out,
        "#[gunbc_testgen_registry_macros::resource_test_target("
    )
    .unwrap();
    writeln!(out, "    name = \"{test_name}\",").unwrap();
    writeln!(out, "    builder = \"{builder_expr}\"").unwrap();
    writeln!(out, ")]").unwrap();

    // testgen_target decorator
    writeln!(out, "#[gunbc_testgen_registry_macros::testgen_target(").unwrap();
    writeln!(out, "    name = \"{test_name}\",").unwrap();
    writeln!(out, "    output = \"{output_path}\",").unwrap();
    writeln!(out, "    module = \"{module_name}\",").unwrap();
    writeln!(out, "    builder = \"{builder_expr}\",").unwrap();
    if let Some(ref sig) = config.signature_fn {
        writeln!(out, "    signature = \"{sig}\",").unwrap();
    }
    if let Some(ref tool) = config.tool_name {
        writeln!(out, "    tool = \"{tool}\",").unwrap();
    }
    writeln!(out, "    {flow_flag}{class_attr}").unwrap();
    writeln!(out, ")]").unwrap();

    // Function body
    writeln!(out, "pub fn {fn_name}() -> MockSpec {{").unwrap();
    writeln!(out, "    let dag = {builder_expr};",).unwrap();
    writeln!(
        out,
        "    let mut spec = {}(&dag, \"{test_name}\");",
        config.auto_mock_fn
    )
    .unwrap();

    // Apply fixture if specified
    if let Some(ref fixture_name) = test.fixture {
        if fixtures.contains_key(fixture_name) {
            writeln!(out, "    spec = apply_{fixture_name}(spec);").unwrap();
        }
    }

    // Apply test-local mocks
    for mock in &test.mocks {
        emit_mock_apply(out, mock, "    ");
    }

    // Apply input declarations
    for input in &test.inputs {
        emit_input_apply(out, input, "    ");
    }

    // Apply expect assertions as expected_outputs / live_expected_outputs
    for expect in &test.expects {
        emit_expect_apply(out, expect, "    ");
    }

    writeln!(out, "    spec").unwrap();
    writeln!(out, "}}").unwrap();
    writeln!(out).unwrap();
}

/// Whether the expression is a transport response constructor.
///
/// Transport mocks are distinguished from boundary mocks by the **value type**,
/// not the port name. If the mock value is a transport response constructor
/// (`rest_response`, `shell_response`, `file_response`), it's a transport mock.
fn is_transport_response_value(expr: &Expr) -> bool {
    match expr {
        Expr::Call(name, _) => {
            matches!(
                name.as_str(),
                "rest_response" | "shell_response" | "file_response"
            )
        }
        Expr::Record(name, _) => name.as_deref().is_some_and(|n| {
            matches!(
                n,
                "rest_response"
                    | "RestResponse"
                    | "shell_response"
                    | "ShellResponse"
                    | "file_response"
                    | "FileResponse"
            )
        }),
        _ => false,
    }
}

/// Emit a mock application statement.
///
/// The mock kind (transport vs boundary) is determined by the value's type:
/// transport response constructors (`rest_response`, `shell_response`,
/// `file_response`) produce `spec.transport_mock(...)`, all other values
/// produce `spec.boundary(...)`.
fn emit_mock_apply(out: &mut String, mock: &MockDecl, indent: &str) {
    let node_id = mock.node_segments.join("/");
    let port = &mock.port;
    let value_expr = emit_value_expr(&mock.value);

    if is_transport_response_value(&mock.value) {
        writeln!(
            out,
            "{indent}spec = spec.transport_mock(\"{node_id}\", \"{port}\", {value_expr});"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "{indent}spec = spec.boundary(\"{node_id}\", \"{port}\", {value_expr});"
        )
        .unwrap();
    }
}

/// Emit an input application statement.
fn emit_input_apply(out: &mut String, input: &InputDecl, indent: &str) {
    let node_id = input.node_segments.join("/");
    let port = &input.port;
    let value_expr = emit_value_expr(&input.value);
    writeln!(
        out,
        "{indent}spec = spec.input_mock(\"{node_id}\", \"{port}\", {value_expr});"
    )
    .unwrap();
}

/// Emit an expect assertion as a live_expected_output or expected_output.
fn emit_expect_apply(out: &mut String, expect: &ExpectStmt, indent: &str) {
    match expect {
        ExpectStmt::Eq(lhs, rhs) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                let value = emit_value_expr(rhs);
                writeln!(
                    out,
                    "{indent}spec = spec.expected_output(\"{node}\", \"{port}\", {value});"
                )
                .unwrap();
            }
        }
        ExpectStmt::Contains(lhs, rhs) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                let substr = emit_string_literal(rhs);
                writeln!(
                    out,
                    "{indent}spec = spec.live_expected_output(\"{node}\", \"{port}\", gunbc_test::OutputMatcher::contains({substr}));"
                )
                .unwrap();
            }
        }
        ExpectStmt::Is(lhs, type_name) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                let matcher = match type_name.as_str() {
                    "String" => "gunbc_test::OutputMatcher::IsString",
                    "Bool" => "gunbc_test::OutputMatcher::IsBool",
                    "Int" => "gunbc_test::OutputMatcher::IsInt",
                    "NonEmpty" => "gunbc_test::OutputMatcher::NonEmpty",
                    _ => "gunbc_test::OutputMatcher::Any",
                };
                writeln!(
                    out,
                    "{indent}spec = spec.live_expected_output(\"{node}\", \"{port}\", {matcher});"
                )
                .unwrap();
            }
        }
        ExpectStmt::Truthy(lhs) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                writeln!(
                    out,
                    "{indent}spec = spec.live_expected_output(\"{node}\", \"{port}\", gunbc_test::OutputMatcher::NonEmpty);"
                )
                .unwrap();
            }
        }
        // For comparison operators, use NonEmpty as a baseline assertion
        ExpectStmt::Ne(lhs, _)
        | ExpectStmt::Lt(lhs, _)
        | ExpectStmt::Gt(lhs, _)
        | ExpectStmt::Le(lhs, _)
        | ExpectStmt::Ge(lhs, _) => {
            if let Some((node, port)) = extract_result_path(lhs) {
                writeln!(
                    out,
                    "{indent}spec = spec.live_expected_output(\"{node}\", \"{port}\", gunbc_test::OutputMatcher::NonEmpty);"
                )
                .unwrap();
            }
        }
    }
}

/// Extract (node_id, port_name) from a `result.field` expression.
///
/// `result.foo` maps to [`TERMINAL_NODE_SENTINEL`] with output port "foo".
/// The actual node ID resolution happens at testgen time when the DAG is available.
fn extract_result_path(expr: &Expr) -> Option<(String, String)> {
    match expr {
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(name) = base.as_ref() {
                if name == "result" {
                    return Some((TERMINAL_NODE_SENTINEL.to_string(), field.clone()));
                }
                // `node.port` — direct node reference
                return Some((name.clone(), field.clone()));
            }
            // `a.b.c` → node = a/b, port = c
            if let Some((parent_node, parent_port)) = extract_result_path(base) {
                return Some((format!("{parent_node}/{parent_port}"), field.clone()));
            }
            None
        }
        _ => None,
    }
}

/// Emit a Rust expression for a DSL value.
fn emit_value_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(lit) => match lit {
            Literal::String(s) => format!("Value::Str({}.to_string())", quote_rust_str(s)),
            Literal::Int(n) => format!("Value::Int({n})"),
            Literal::Float(f) => format!("Value::Float({f})"),
            Literal::Bool(b) => format!("Value::Bool({b})"),
            Literal::None => "Value::None".to_string(),
        },
        Expr::StringInterp(parts) => {
            let mut s = String::new();
            for part in parts {
                match part {
                    daglang_syntax::ast::StringPart::Literal(lit) => s.push_str(lit),
                    daglang_syntax::ast::StringPart::Expr(e) => {
                        s.push_str(&format!("{{{}}}", emit_inline_expr(e)));
                    }
                }
            }
            format!("Value::Str({}.to_string())", quote_rust_str(&s))
        }
        Expr::Record(name, fields) => emit_record_value(name.as_deref(), fields),
        Expr::Call(name, args) => {
            match name.as_str() {
                kind @ ("rest_response" | "shell_response" | "file_response") => {
                    emit_transport_response(kind, TransportFields::Positional(args))
                }
                "bytes" => {
                    if let Some((_, arg)) = args.first() {
                        let s = emit_string_literal(arg);
                        format!("Value::Str({s}.to_string())")
                    } else {
                        "Value::Str(String::new())".to_string()
                    }
                }
                _ => {
                    // Generic function call — emit as JSON
                    format!("Value::Str(\"{name}(...)\".to_string())")
                }
            }
        }
        Expr::Ident(name) => match name.as_str() {
            "true" => "Value::Bool(true)".to_string(),
            "false" => "Value::Bool(false)".to_string(),
            "none" => "Value::None".to_string(),
            _ => format!("Value::Str(\"{name}\".to_string())"),
        },
        Expr::List(items) => {
            let values: Vec<String> = items.iter().map(emit_value_expr).collect();
            format!("Value::List(vec![{}])", values.join(", "))
        }
        Expr::Map(entries) => {
            let mut json_parts = Vec::new();
            for (key, value) in entries {
                let k = emit_json_value(key);
                let v = emit_json_value(value);
                json_parts.push(format!("{k}: {v}"));
            }
            format!(
                "Value::Json(serde_json::json!({{{}}}))",
                json_parts.join(", ")
            )
        }
        Expr::UnaryOp(op, inner) => match op {
            daglang_syntax::ast::UnaryOp::Neg => {
                if let Expr::Literal(Literal::Int(n)) = inner.as_ref() {
                    format!("Value::Int(-{n})")
                } else {
                    emit_value_expr(inner)
                }
            }
            daglang_syntax::ast::UnaryOp::Not => {
                if let Expr::Literal(Literal::Bool(b)) = inner.as_ref() {
                    format!("Value::Bool({})", !b)
                } else {
                    emit_value_expr(inner)
                }
            }
        },
        other => {
            // Fail at compile time in the generated code so unhandled
            // expression variants surface immediately instead of silently
            // producing None.
            let variant = std::mem::discriminant(other);
            format!("compile_error!(\"test_mock_emit: unhandled DSL expression variant ({variant:?})\")")
        }
    }
}

/// Normalised field access for transport response constructors.
///
/// Both call-syntax `rest_response(200, { ok: true })` and record-syntax
/// `rest_response { status: 200, ok: true }` need to produce the same output.
/// This enum unifies positional and named access so a single emit function
/// handles both.
enum TransportFields<'a> {
    /// Positional arguments from `Expr::Call`.
    Positional(&'a [(Option<String>, Expr)]),
    /// Named fields from `Expr::Record`.
    Named(&'a [(String, Expr)]),
}

impl<'a> TransportFields<'a> {
    /// Get the first positional arg, or look up a named field.
    fn get(&self, position: usize, names: &[&str]) -> Option<&'a Expr> {
        match self {
            Self::Positional(args) => args.get(position).map(|(_, e)| e),
            Self::Named(fields) => fields
                .iter()
                .find(|(k, _)| names.iter().any(|n| k == n))
                .map(|(_, v)| v),
        }
    }

    /// Iterate named fields (for building the JSON body of a rest_response).
    /// For positional args, the body is at position 1; for named fields, it's
    /// everything that isn't in `exclude`.
    fn rest_body_json(&self, exclude: &[&str]) -> String {
        match self {
            Self::Positional(args) => args
                .get(1)
                .map(|(_, e)| emit_json_value(e))
                .unwrap_or_else(|| "{}".to_string()),
            Self::Named(fields) => {
                let parts: Vec<String> = fields
                    .iter()
                    .filter(|(k, _)| !exclude.iter().any(|ex| k == ex))
                    .map(|(k, v)| format!("\"{k}\": {}", emit_json_value(v)))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }
}

/// Emit a transport response expression (rest, shell, or file).
///
/// Handles both call-syntax and record-syntax via [`TransportFields`].
fn emit_transport_response(kind: &str, fields: TransportFields<'_>) -> String {
    match kind {
        "rest_response" | "RestResponse" => {
            let status = fields
                .get(0, &["status", "code"])
                .map(emit_int_value)
                .unwrap_or(200);
            let body = fields.rest_body_json(&["status", "code"]);
            format!(
                "Value::Response(TransportResponse::Rest(RestResponse::new({status}, serde_json::json!({body}))))"
            )
        }
        "shell_response" | "ShellResponse" => {
            let exit_code = fields
                .get(0, &["exit_code"])
                .map(emit_int_value)
                .unwrap_or(0);
            let stdout = fields
                .get(1, &["stdout"])
                .map(emit_string_literal)
                .unwrap_or_else(|| "\"\"".to_string());
            format!(
                "Value::Response(TransportResponse::Shell(ShellResponse {{ exit_code: {exit_code}, stdout: {stdout}.to_string(), stderr: String::new() }}))"
            )
        }
        "file_response" | "FileResponse" => {
            let path = fields
                .get(0, &["path"])
                .map(emit_string_literal)
                .unwrap_or_else(|| "\"\"".to_string());
            format!(
                "Value::Response(TransportResponse::File(FileResponse {{ path: {path}.to_string(), operation: FileOp::Read, success: true, content: None, exists: None, error: None, bytes: None }}))"
            )
        }
        _ => unreachable!("emit_transport_response called with unknown kind: {kind}"),
    }
}

/// Emit a record value expression.
///
/// Transport records delegate to [`emit_transport_response`]; everything else
/// becomes `Value::Json(serde_json::json!(...))`.
fn emit_record_value(name: Option<&str>, fields: &[(String, Expr)]) -> String {
    match name {
        Some(
            kind @ ("rest_response" | "RestResponse" | "shell_response" | "ShellResponse"
            | "file_response" | "FileResponse"),
        ) => emit_transport_response(kind, TransportFields::Named(fields)),
        _ => {
            let json_parts: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("\"{k}\": {}", emit_json_value(v)))
                .collect();
            let json = format!("{{{}}}", json_parts.join(", "));
            format!("Value::Json(serde_json::json!({json}))")
        }
    }
}

/// Emit a Rust string literal (properly escaped).
fn quote_rust_str(s: &str) -> String {
    format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\t', "\\t")
    )
}

/// Emit an integer from an expression.
fn emit_int_value(expr: &Expr) -> i64 {
    match expr {
        Expr::Literal(Literal::Int(n)) => *n,
        _ => 0,
    }
}

/// Emit a Rust string literal from an expression.
fn emit_string_literal(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::String(s)) => quote_rust_str(s),
        _ => "\"\"".to_string(),
    }
}

/// Emit a JSON-compatible value.
fn emit_json_value(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::String(s)) => quote_rust_str(s),
        Expr::Literal(Literal::Int(n)) => n.to_string(),
        Expr::Literal(Literal::Float(f)) => f.to_string(),
        Expr::Literal(Literal::Bool(b)) => b.to_string(),
        Expr::Literal(Literal::None) => "null".to_string(),
        Expr::Ident(name) => match name.as_str() {
            "true" => "true".to_string(),
            "false" => "false".to_string(),
            "none" | "null" => "null".to_string(),
            _ => quote_rust_str(name),
        },
        Expr::Record(_, fields) => {
            let parts: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("\"{k}\": {}", emit_json_value(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        Expr::List(items) => {
            let parts: Vec<String> = items.iter().map(emit_json_value).collect();
            format!("[{}]", parts.join(", "))
        }
        _ => "null".to_string(),
    }
}

/// Emit an inline expression (for string interpolation).
fn emit_inline_expr(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => name.clone(),
        Expr::FieldAccess(base, field) => {
            format!("{}.{field}", emit_inline_expr(base))
        }
        _ => "?".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_syntax::parser;

    #[test]
    fn parse_and_emit_simple_test() {
        let source = r#"
fixture cloud_base {
    mock cloud_env.config -> { project: "mock-project" }
}

test bootstrap_dryrun : cloud_base {
    tier: Unit
    hermetic
    mock execute.response -> rest_response(200, { ok: true })
    expect result.ok == true
}
"#;

        let ast = parser::parse(source).expect("should parse");
        let test_file = TestFile::from_source(&ast);

        assert_eq!(test_file.fixtures.len(), 1);
        assert_eq!(test_file.tests.len(), 1);
        assert_eq!(test_file.tests[0].name, "bootstrap_dryrun");
        assert_eq!(test_file.tests[0].fixture.as_deref(), Some("cloud_base"));
        assert_eq!(test_file.tests[0].mocks.len(), 1);
        assert_eq!(test_file.tests[0].expects.len(), 1);

        let config = TestEmitConfig {
            dag_builder: "crate::build_bootstrap_graph().unwrap()".to_string(),
            auto_mock_fn: "gunbc_test::auto_mock_spec".to_string(),
            output_dir: "gunbc-dag/src/bootstrap".to_string(),
            tool_name: Some("bootstrap".to_string()),
            signature_fn: Some("crate::bootstrap_signature()".to_string()),
        };

        let output = emit_test_mock_file(&test_file, &config);

        assert!(output.contains("fn apply_cloud_base(spec: MockSpec) -> MockSpec"));
        assert!(output.contains("pub fn bootstrap_dryrun_mock_spec() -> MockSpec"));
        assert!(output.contains("testgen_target"));
        assert!(output.contains("resource_test_target"));
        assert!(output.contains("apply_cloud_base(spec)"));
        assert!(output.contains("transport_mock"));
        // Builder should use .expect(), not .unwrap()
        assert!(output.contains(".expect(\"graph should build\")"));
        assert!(!output.contains(".unwrap()"));
    }

    #[test]
    fn parse_test_with_inputs() {
        let source = r#"
test dag_viz_snapshot {
    input render_snapshot.topology_json -> "{\"nodes\":[],\"edges\":[]}"
    mock execute.response -> rest_response(200, { ok: true })
    expect result.url is String
}
"#;

        let ast = parser::parse(source).expect("should parse");
        let test_file = TestFile::from_source(&ast);

        assert_eq!(test_file.tests.len(), 1);
        assert_eq!(test_file.tests[0].inputs.len(), 1);
        assert_eq!(
            test_file.tests[0].inputs[0].node_segments,
            vec!["render_snapshot"]
        );
        assert_eq!(test_file.tests[0].inputs[0].port, "topology_json");
    }

    #[test]
    fn parse_test_with_slashed_node_paths() {
        let source = r#"
test gist_upload_test {
    mock gist_upload/execute.response -> rest_response(200, { html_url: "https://gist.github.com/abc" })
    mock gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam.ok -> true
}
"#;

        let ast = parser::parse(source).expect("should parse");
        let test_file = TestFile::from_source(&ast);

        assert_eq!(test_file.tests[0].mocks.len(), 2);
        assert_eq!(
            test_file.tests[0].mocks[0].node_segments,
            vec!["gist_upload", "execute"]
        );
        assert_eq!(test_file.tests[0].mocks[0].port, "response");
        assert_eq!(
            test_file.tests[0].mocks[1].node_segments,
            vec![
                "gist_upload",
                "cloud_credential",
                "gcp_wif_secret",
                "parse_set_iam"
            ]
        );
        assert_eq!(test_file.tests[0].mocks[1].port, "ok");
    }

    #[test]
    fn transport_mock_detected_by_value_type_not_port_name() {
        let source = r#"
test value_type_detection {
    mock node_a.response -> { some: "boundary-data" }
    mock node_b.data -> rest_response(200, { ok: true })
}
"#;

        let ast = parser::parse(source).expect("should parse");
        let test_file = TestFile::from_source(&ast);

        let config = TestEmitConfig {
            dag_builder: "crate::build_graph()".to_string(),
            auto_mock_fn: "crate::auto_mock_spec".to_string(),
            output_dir: "test".to_string(),
            tool_name: None,
            signature_fn: None,
        };

        let output = emit_test_mock_file(&test_file, &config);

        // node_a.response has a record value (not transport) → boundary
        assert!(output.contains("spec.boundary(\"node_a\", \"response\""));
        // node_b.data has a rest_response value → transport_mock
        assert!(output.contains("spec.transport_mock(\"node_b\", \"data\""));
    }

    #[test]
    fn terminal_sentinel_used_for_result_paths() {
        let source = r#"
test sentinel_check {
    expect result.content is String
}
"#;

        let ast = parser::parse(source).expect("should parse");
        let test_file = TestFile::from_source(&ast);

        let config = TestEmitConfig {
            dag_builder: "crate::build_graph()".to_string(),
            auto_mock_fn: "crate::auto_mock_spec".to_string(),
            output_dir: "test".to_string(),
            tool_name: None,
            signature_fn: None,
        };

        let output = emit_test_mock_file(&test_file, &config);
        assert!(output.contains(TERMINAL_NODE_SENTINEL));
    }

    #[test]
    fn rust_mock_helpers_annotation_emitted_as_comment() {
        // mock_helpers has no typed DSL syntax (parser always returns None).
        // Construct TestFile directly to verify emit logic for mock_helpers.
        let test_file = TestFile {
            fixtures: BTreeMap::new(),
            tests: vec![TestDef {
                name: "with_helpers".to_string(),
                fixture: None,
                mocks: vec![MockDecl {
                    node_segments: vec!["execute".to_string()],
                    port: "data".to_string(),
                    value: Expr::Record(
                        None,
                        vec![("ok".to_string(), Expr::Literal(Literal::Bool(true)))],
                    ),
                }],
                inputs: vec![],
                expects: vec![],
                tier: None,
                hermetic: false,
                skip: false,
                auto_mock: false,
                mock_helpers: Some("gunbc_lib_review::graph_mock".to_string()),
            }],
        };

        let config = TestEmitConfig {
            dag_builder: "crate::build_graph()".to_string(),
            auto_mock_fn: "crate::auto_mock_spec".to_string(),
            output_dir: "test".to_string(),
            tool_name: None,
            signature_fn: None,
        };

        let output = emit_test_mock_file(&test_file, &config);
        assert!(output.contains("// Rust mock helpers: gunbc_lib_review::graph_mock"));
    }

    #[test]
    fn testgen_skip_excludes_test_from_emission() {
        let source = r#"
test normal_test {
    tier: Unit
    mock execute.data -> { ok: true }
}

test skipped_test {
    tier: Unit
    skip
    mock execute.data -> { ok: true }
}

test another_normal {
    tier: Unit
    mock execute.data -> { ok: false }
}
"#;
        let ast = parser::parse(source).expect("should parse");
        let test_file = TestFile::from_source(&ast);
        assert_eq!(test_file.tests.len(), 3, "all 3 tests parsed");

        let config = TestEmitConfig {
            dag_builder: "crate::build_graph()".to_string(),
            auto_mock_fn: "crate::auto_mock_spec".to_string(),
            output_dir: "test".to_string(),
            tool_name: None,
            signature_fn: None,
        };

        let output = emit_test_mock_file(&test_file, &config);
        assert!(
            output.contains("normal_test_mock_spec"),
            "normal test should be emitted"
        );
        assert!(
            !output.contains("skipped_test_mock_spec"),
            "skip-marked test should be excluded"
        );
        assert!(
            output.contains("another_normal_mock_spec"),
            "second normal test should be emitted"
        );
    }
}
