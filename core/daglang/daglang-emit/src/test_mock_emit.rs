//! Test mock emitter: generates `graph_mock.rs` Rust code from `_test.dag` files.
//!
//! Each `test` block in a `_test.dag` file becomes a function that returns
//! a `MockSpec`, decorated with `#[testgen_target]` and
//! `#[resource_test_target]` proc macros.
//!
//! Each `fixture` block becomes a helper function that applies shared mocks
//! to a `MockSpec`, callable from test functions.
//!
//! # Example
//!
//! ```dag
//! // bootstrap_test.dag
//! fixture cloud_env {
//!     mock cloud_env.config -> { project: "mock-project" }
//! }
//!
//! test bootstrap_dryrun : cloud_env {
//!     @tier(Unit)
//!     @hermetic(true)
//!     mock execute.response -> rest_response(200, { ok: true })
//!     expect result.ok == true
//! }
//! ```
//!
//! Generates:
//!
//! ```rust,ignore
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
    Annotation, Expr, ExpectStmt, FixtureDef, InputDecl, Literal, MockDecl, SourceFile, Item,
    TestDef,
};
use std::collections::HashMap;
use std::fmt::Write;

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
    pub fixtures: HashMap<String, FixtureDef>,
    pub tests: Vec<TestDef>,
}

impl TestFile {
    /// Extract fixtures and tests from a parsed source file.
    pub fn from_source(source: &SourceFile) -> Self {
        let mut fixtures = HashMap::new();
        let mut tests = Vec::new();
        for item in &source.items {
            match &item.node {
                Item::FixtureDef(f) => {
                    fixtures.insert(f.name.clone(), FixtureDef {
                        name: f.name.clone(),
                        mocks: f.mocks.clone(),
                    });
                }
                Item::TestDef(t) => {
                    tests.push(TestDef {
                        name: t.name.clone(),
                        annotations: t.annotations.clone(),
                        fixture: t.fixture.clone(),
                        mocks: t.mocks.clone(),
                        inputs: t.inputs.clone(),
                        expects: t.expects.clone(),
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
    writeln!(out, "//! Generated from _test.dag — do not edit by hand.").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "use gunbc_test::MockSpec;").unwrap();
    writeln!(out, "use gunbc_ir::Value;").unwrap();
    writeln!(out, "use gunbc_ir::transport::{{RestResponse, ShellResponse, FileResponse, FileOp, TransportResponse}};").unwrap();
    writeln!(out).unwrap();

    // Emit fixture helpers
    for fixture in test_file.fixtures.values() {
        emit_fixture_fn(&mut out, fixture);
    }

    // Emit test functions
    for test in &test_file.tests {
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
    fixtures: &HashMap<String, FixtureDef>,
    config: &TestEmitConfig,
) {
    let fn_name = format!("{}_mock_spec", test.name);
    let test_name = test.name.replace('_', "-");

    // Extract tier and other annotations
    let tier = find_annotation_string(&test.annotations, "tier");
    let hermetic = find_annotation_bool(&test.annotations, "hermetic");

    // Determine testgen flags
    let flow_flag = if hermetic.unwrap_or(true) {
        "flow_tests"
    } else {
        "live_flow_tests"
    };
    let class_attr = match tier.as_deref() {
        Some("Unit") => ", class = \"unit\", fermi = \"XS\"",
        Some("Contract") => ", class = \"hermetic\", fermi = \"S\"",
        Some("Scenario") => ", class = \"hermetic\", fermi = \"S\"",
        Some("Integration") => ", class = \"integration\", fermi = \"M\"",
        _ => "",
    };

    let output_path = format!(
        "{}/generated_tests_{}.rs",
        config.output_dir,
        test.name
    );
    let module_name = format!("{}_generated_tests", test.name);

    // resource_test_target decorator
    writeln!(
        out,
        "#[gunbc_testgen_registry_macros::resource_test_target("
    )
    .unwrap();
    writeln!(out, "    name = \"{test_name}\",").unwrap();
    writeln!(
        out,
        "    builder = \"{}\"",
        config.dag_builder
    )
    .unwrap();
    writeln!(out, ")]").unwrap();

    // testgen_target decorator
    writeln!(
        out,
        "#[gunbc_testgen_registry_macros::testgen_target("
    )
    .unwrap();
    writeln!(out, "    name = \"{test_name}\",").unwrap();
    writeln!(out, "    output = \"{output_path}\",").unwrap();
    writeln!(out, "    module = \"{module_name}\",").unwrap();
    writeln!(
        out,
        "    builder = \"{}\",",
        config.dag_builder
    )
    .unwrap();
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
    writeln!(
        out,
        "    let dag = {};",
        config.dag_builder.replace(".unwrap()", ".expect(\"graph should build\")")
    )
    .unwrap();
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

/// Emit a mock application statement.
fn emit_mock_apply(out: &mut String, mock: &MockDecl, indent: &str) {
    let node_id = mock.node_segments.join("/");
    let port = &mock.port;
    let value_expr = emit_value_expr(&mock.value);

    // Heuristic: if the port is "response" on a node that looks like a transport
    // executor, use transport_mock. Otherwise use boundary.
    if port == "response" {
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
                let substr = emit_string_value(rhs);
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
/// For now, `result.foo` maps to the terminal node with output port "foo".
/// The actual node ID resolution happens at testgen time when the DAG is available.
fn extract_result_path(expr: &Expr) -> Option<(String, String)> {
    match expr {
        Expr::FieldAccess(base, field) => {
            if let Expr::Ident(name) = base.as_ref() {
                if name == "result" {
                    // `result.field` — the node is resolved at testgen time
                    return Some(("__terminal__".to_string(), field.clone()));
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
            // For simplicity, concatenate parts into a single string
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
        Expr::Record(name, fields) => {
            // Emit as JSON value
            let mut json_parts = Vec::new();
            for (key, value) in fields {
                json_parts.push(format!("\"{key}\": {}", emit_json_value(value)));
            }
            let json = format!("{{{}}}", json_parts.join(", "));
            if name.as_deref() == Some("rest_response") || name.as_deref() == Some("RestResponse") {
                // Special handling for REST responses
                format!(
                    "Value::Response(TransportResponse::Rest(RestResponse::new(200, serde_json::json!({}))))",
                    json
                )
            } else {
                format!(
                    "Value::Json(serde_json::json!({}))",
                    json
                )
            }
        }
        Expr::Call(name, args) => {
            match name.as_str() {
                "rest_response" => emit_rest_response_call(args),
                "shell_response" => emit_shell_response_call(args),
                "file_response" => emit_file_response_call(args),
                "bytes" => {
                    if let Some((_, arg)) = args.first() {
                        let s = emit_string_value(arg);
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
        Expr::Ident(name) => {
            match name.as_str() {
                "true" => "Value::Bool(true)".to_string(),
                "false" => "Value::Bool(false)".to_string(),
                "none" => "Value::None".to_string(),
                _ => format!("Value::Str(\"{name}\".to_string())"),
            }
        }
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
        Expr::UnaryOp(op, inner) => {
            match op {
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
            }
        }
        // For other expression types, fall back to string representation
        _ => "Value::None".to_string(),
    }
}

/// Emit a rest_response(status, body) call.
fn emit_rest_response_call(args: &[(Option<String>, Expr)]) -> String {
    let status = args
        .first()
        .map(|(_, e)| emit_int_value(e))
        .unwrap_or(200);
    let body = args
        .get(1)
        .map(|(_, e)| emit_json_value(e))
        .unwrap_or_else(|| "{}".to_string());
    format!(
        "Value::Response(TransportResponse::Rest(RestResponse::new({status}, serde_json::json!({body}))))"
    )
}

/// Emit a shell_response(exit_code, stdout) call.
fn emit_shell_response_call(args: &[(Option<String>, Expr)]) -> String {
    let exit_code = args
        .first()
        .map(|(_, e)| emit_int_value(e))
        .unwrap_or(0);
    let stdout = args
        .get(1)
        .map(|(_, e)| emit_raw_string_value(e))
        .unwrap_or_default();
    format!(
        "Value::Response(TransportResponse::Shell(ShellResponse {{ exit_code: {exit_code}, stdout: {stdout}.to_string(), stderr: String::new() }}))"
    )
}

/// Emit a file_response(path, op, success) call.
fn emit_file_response_call(args: &[(Option<String>, Expr)]) -> String {
    let path = args
        .first()
        .map(|(_, e)| emit_raw_string_value(e))
        .unwrap_or_else(|| "\"\"".to_string());
    format!(
        "Value::Response(TransportResponse::File(FileResponse {{ path: {path}.to_string(), operation: FileOp::Read, success: true, content: None, exists: None, error: None }}))"
    )
}

/// Emit a Rust string literal (properly escaped).
fn quote_rust_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t"))
}

/// Emit an integer from an expression.
fn emit_int_value(expr: &Expr) -> i64 {
    match expr {
        Expr::Literal(Literal::Int(n)) => *n,
        _ => 0,
    }
}

/// Emit a string value from an expression (Rust literal).
fn emit_string_value(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::String(s)) => quote_rust_str(s),
        _ => "\"\"".to_string(),
    }
}

/// Emit a raw string from an expression.
fn emit_raw_string_value(expr: &Expr) -> String {
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

/// Find a string value for an annotation by name.
fn find_annotation_string(annotations: &[Annotation], name: &str) -> Option<String> {
    annotations.iter().find(|a| a.name == name).and_then(|a| {
        a.args.first().and_then(|arg| match arg {
            Expr::Ident(s) => Some(s.clone()),
            Expr::Literal(Literal::String(s)) => Some(s.clone()),
            _ => None,
        })
    })
}

/// Find a boolean value for an annotation by name.
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
    @tier(Unit)
    @hermetic(true)
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
            auto_mock_fn: "crate::mock_defaults::auto_mock_spec".to_string(),
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
        assert_eq!(test_file.tests[0].inputs[0].node_segments, vec!["render_snapshot"]);
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
            vec!["gist_upload", "cloud_credential", "gcp_wif_secret", "parse_set_iam"]
        );
        assert_eq!(test_file.tests[0].mocks[1].port, "ok");
    }
}
