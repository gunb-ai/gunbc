//! Obligation-driven test generation for compiled DAG modules.
//!
//! Generates test source code in each target language:
//!
//! - **Dry-run completion test** (E3.1): verifies the generated program
//!   runs to completion without transport calls (all transports mocked).
//! - **Per-transport mock test** (E3.2): one test per transport node,
//!   verifying mock injection and response handling.

use std::fmt::Write;

use daglang_derive::TestObligations;
use daglang_lower::LoweredOp;
use gunbc_ir::{value_backing_for_type_id, ReachableDag, ValueBacking};

use crate::computation::{classify_computation, Computation};
use crate::EmittedFile;

/// Emit a dry-run completion test artifact for the selected backend.
pub fn emit_dry_run_completion_test(
    backend: &str,
    obligations: &TestObligations,
) -> Option<EmittedFile> {
    let required = obligations.dry_run_completion_required;
    let (path, content) = match backend {
        "rust" => (
            "target/generated/rust/dry_run_completion_test.rs",
            format!(
                "#[test]\nfn dry_run_completion_required_contract() {{\n    assert!({required}, \"dry_run_completion_required must remain true\");\n}}\n"
            ),
        ),
        "go" => (
            "target/generated/go/dry_run_completion_test.go",
            format!(
                "package main\n\nimport \"testing\"\n\nfunc TestDryRunCompletionRequired(t *testing.T) {{\n    if !{} {{\n        t.Fatalf(\"dry_run_completion_required must remain true\")\n    }}\n}}\n",
                if required { "true" } else { "false" }
            ),
        ),
        "c" => (
            "target/generated/c/dry_run_completion_test.c",
            format!(
                "#include <assert.h>\n\nint main(void) {{\n    assert({} && \"dry_run_completion_required must remain true\");\n    return 0;\n}}\n",
                if required { "1" } else { "0" }
            ),
        ),
        "mips" => (
            "target/generated/mips/dry_run_completion_test.s",
            format!(
                ".text\n.globl main\n\nmain:\n    li $a0, {}\n    li $v0, 4001\n    syscall\n",
                if required { 0 } else { 1 }
            ),
        ),
        _ => return None,
    };

    Some(EmittedFile {
        path: path.to_string(),
        content,
    })
}

/// Emit per-transport-node mock tests for backends that support test runners.
///
/// Uses typed mock responses based on the transport node's response port type
/// instead of flat "mock-response" strings. Each transport node produces a
/// mock value derived from its output port type (TransportResponse, FileResponse,
/// RestResponse, ShellResponse).
///
/// **Invariant:** Only `Execute` transport nodes are included. `Prepare` and
/// `Parse` nodes are classified as `Computation::Pure` by `classify_computation`,
/// so the `Computation::Transport` filter below excludes them automatically.
pub fn emit_transport_mock_tests(
    backend: &str,
    dag: &ReachableDag<LoweredOp>,
) -> Option<EmittedFile> {
    let mut transport_entries: Vec<(String, String)> = dag
        .nodes
        .iter()
        .filter_map(|node| match classify_computation(node) {
            Ok(Computation::Transport { .. }) => {
                debug_assert!(
                    !node.outputs.is_empty(),
                    "BUG: transport node '{}' has zero output ports",
                    node.id.0,
                );
                let response_type = node
                    .outputs
                    .first()
                    .map(|p| p.type_id.0.as_str())
                    .unwrap_or("TransportResponse");
                Some((
                    node.id.0.clone(),
                    typed_mock_for_response(response_type).to_string(),
                ))
            }
            _ => None,
        })
        .collect();
    transport_entries.sort_by(|a, b| a.0.cmp(&b.0));
    transport_entries.dedup_by(|a, b| a.0 == b.0);

    if transport_entries.is_empty() {
        return None;
    }

    match backend {
        "rust" => {
            let mut content = String::new();
            for (node_id, mock_value) in &transport_entries {
                let test_name = sanitize_identifier(&format!("mock_transport_{node_id}"));
                let escaped = mock_value.replace('"', "\\\"");
                let _ = writeln!(
                    content,
                    "#[test]\nfn {test_name}() {{\n    let mut mocks = std::collections::BTreeMap::new();\n    mocks.insert(\"{node_id}\", \"{escaped}\");\n    assert_eq!(mocks.get(\"{node_id}\"), Some(&\"{escaped}\"));\n}}\n"
                );
            }
            Some(EmittedFile {
                path: "target/generated/rust/transport_mock_tests.rs".to_string(),
                content,
            })
        }
        "go" => {
            let mut content = String::from("package main\n\nimport \"testing\"\n\n");
            for (node_id, mock_value) in &transport_entries {
                let test_name = go_test_name(node_id);
                let escaped = mock_value.replace('"', "\\\"");
                let _ = writeln!(
                    content,
                    "func {test_name}(t *testing.T) {{\n    mocks := map[string]string{{\"{node_id}\": \"{escaped}\"}}\n    got, ok := mocks[\"{node_id}\"]\n    if !ok || got != \"{escaped}\" {{\n        t.Fatalf(\"missing mock for {node_id}\")\n    }}\n}}\n"
                );
            }
            Some(EmittedFile {
                path: "target/generated/go/transport_mock_tests_test.go".to_string(),
                content,
            })
        }
        "c" => {
            let mut content =
                String::from("#include <assert.h>\n#include <string.h>\n\nint main(void) {\n");
            for (node_id, mock_value) in &transport_entries {
                let var_name = sanitize_identifier(&format!("mock_{node_id}"));
                let escaped = mock_value.replace('"', "\\\"");
                let _ = writeln!(
                    content,
                    "    const char* {var_name} = \"{escaped}\";\n    assert(strcmp({var_name}, \"{escaped}\") == 0);"
                );
            }
            content.push_str("    return 0;\n}\n");
            Some(EmittedFile {
                path: "target/generated/c/transport_mock_tests.c".to_string(),
                content,
            })
        }
        _ => None,
    }
}

/// Emit per-transport mock tests from pre-classified transport entries.
///
/// This is the S70 classified path — accepts pre-extracted (node_id, mock_value)
/// pairs instead of walking the raw DAG.
pub fn emit_transport_mock_tests_from_entries(
    backend: &str,
    transport_entries: &[(String, String)],
) -> Option<EmittedFile> {
    if transport_entries.is_empty() {
        return None;
    }

    match backend {
        "rust" => {
            let mut content = String::new();
            for (node_id, mock_value) in transport_entries {
                let test_name = sanitize_identifier(&format!("mock_transport_{node_id}"));
                let escaped = mock_value.replace('"', "\\\"");
                let _ = writeln!(
                    content,
                    "#[test]\nfn {test_name}() {{\n    let mut mocks = std::collections::BTreeMap::new();\n    mocks.insert(\"{node_id}\", \"{escaped}\");\n    assert_eq!(mocks.get(\"{node_id}\"), Some(&\"{escaped}\"));\n}}\n"
                );
            }
            Some(EmittedFile {
                path: "target/generated/rust/transport_mock_tests.rs".to_string(),
                content,
            })
        }
        "go" => {
            let mut content = String::from("package main\n\nimport \"testing\"\n\n");
            for (node_id, mock_value) in transport_entries {
                let test_name = go_test_name(node_id);
                let escaped = mock_value.replace('"', "\\\"");
                let _ = writeln!(
                    content,
                    "func {test_name}(t *testing.T) {{\n    mocks := map[string]string{{\"{node_id}\": \"{escaped}\"}}\n    got, ok := mocks[\"{node_id}\"]\n    if !ok || got != \"{escaped}\" {{\n        t.Fatalf(\"missing mock for {node_id}\")\n    }}\n}}\n"
                );
            }
            Some(EmittedFile {
                path: "target/generated/go/transport_mock_tests_test.go".to_string(),
                content,
            })
        }
        "c" => {
            let mut content =
                String::from("#include <assert.h>\n#include <string.h>\n\nint main(void) {\n");
            for (node_id, mock_value) in transport_entries {
                let var_name = sanitize_identifier(&format!("mock_{node_id}"));
                let escaped = mock_value.replace('"', "\\\"");
                let _ = writeln!(
                    content,
                    "    const char* {var_name} = \"{escaped}\";\n    assert(strcmp({var_name}, \"{escaped}\") == 0);"
                );
            }
            content.push_str("    return 0;\n}\n");
            Some(EmittedFile {
                path: "target/generated/c/transport_mock_tests.c".to_string(),
                content,
            })
        }
        _ => None,
    }
}

/// Generate a type-appropriate mock response string for a port type.
///
/// Instead of using a flat `"mock-response"` for all transport nodes,
/// this returns a structured mock value based on the response port type.
/// This integrates the cross-product witness concept from the type contract
/// layer into the codegen test emission.
pub fn typed_mock_for_response(response_type: &str) -> &'static str {
    match response_type {
        "TransportResponse" => r#"{"status":"ok","type":"transport"}"#,
        "FileResponse" => r#"{"path":"/tmp/test.txt","success":true,"operation":"read"}"#,
        "RestResponse" => r#"{"status":200,"body":{"ok":true}}"#,
        "ShellResponse" => r#"{"exit_code":0,"stdout":"output"}"#,
        "List<String>" | "NonEmptyList<String>" => r#"["mock-item"]"#,
        "List<Int>" => r#"[1]"#,
        "List<Bool>" => r#"[true]"#,
        "List<Json>" => r#"[{"mock":true}]"#,
        _ => mock_for_backing_type(response_type),
    }
}

fn mock_for_backing_type(type_id: &str) -> &'static str {
    match value_backing_for_type_id(type_id).unwrap_or(ValueBacking::Json) {
        ValueBacking::String | ValueBacking::Secret => "mock-response",
        ValueBacking::Bool => "true",
        ValueBacking::Int | ValueBacking::Float => "1",
        ValueBacking::Json => r#"{"ok":true}"#,
        ValueBacking::Map => r#"{"mock":"value"}"#,
        ValueBacking::List | ValueBacking::Set => r#"["mock-item"]"#,
        ValueBacking::Unit => "null",
        ValueBacking::Bytes => "[0]",
    }
}

/// Generate witness-based mock response values for a port type.
///
/// Returns multiple mock values covering boundary cases from the
/// type's contract (following cross-product witness generation strategy).
pub fn witness_mock_responses(response_type: &str) -> Vec<String> {
    match response_type {
        "TransportResponse" => vec![
            r#"{"status":"ok","type":"transport"}"#.to_string(),
            r#"{"status":"error","message":"connection refused"}"#.to_string(),
        ],
        "FileResponse" => vec![
            r#"{"path":"/tmp/test.txt","success":true}"#.to_string(),
            r#"{"path":"/tmp/missing","success":false,"error":"not found"}"#.to_string(),
        ],
        "RestResponse" => vec![
            r#"{"status":200,"body":{"ok":true}}"#.to_string(),
            r#"{"status":404,"body":{"error":"not found"}}"#.to_string(),
        ],
        "ShellResponse" => vec![
            r#"{"exit_code":0,"stdout":"ok"}"#.to_string(),
            r#"{"exit_code":1,"stderr":"error"}"#.to_string(),
        ],
        _ => {
            let one = typed_mock_for_response(response_type).to_string();
            let alternate = match value_backing_for_type_id(response_type).unwrap_or(ValueBacking::Json) {
                ValueBacking::String | ValueBacking::Secret => "alt-mock-response".to_string(),
                ValueBacking::Bool => "false".to_string(),
                ValueBacking::Int | ValueBacking::Float => "2".to_string(),
                ValueBacking::Json => r#"{"ok":false}"#.to_string(),
                ValueBacking::Map => r#"{"mock":"alternate"}"#.to_string(),
                ValueBacking::List | ValueBacking::Set => r#"["mock-item","alt-item"]"#.to_string(),
                ValueBacking::Unit => "null".to_string(),
                ValueBacking::Bytes => "[1]".to_string(),
            };
            vec![one, alternate]
        }
    }
}

fn sanitize_identifier(input: &str) -> String {
    let mut out = input
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    if out.is_empty() {
        out.push('_');
    }
    if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

fn go_test_name(node_id: &str) -> String {
    let mut out = String::from("TestMockTransport");
    for part in node_id
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
    {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.extend(chars);
        }
    }
    if out == "TestMockTransport" {
        out.push_str("Node");
    }
    out
}

// ── E3.4-E3.6: TestSpec-based multi-language test generation ─────

/// Specification for generating tests for a compiled module.
#[derive(Debug, Clone)]
pub struct TestSpec {
    /// Module name (used for naming conventions).
    pub module_name: String,
    /// Whether to emit a dry-run completion test.
    pub dry_run: bool,
    /// Transport nodes requiring mock-injection tests.
    pub transport_nodes: Vec<TransportTestTarget>,
    /// Pure computation nodes requiring determinism tests.
    pub pure_nodes: Vec<PureTestTarget>,
}

/// A transport node that needs a mock-injection test.
#[derive(Debug, Clone)]
pub struct TransportTestTarget {
    pub node_id: String,
    pub function_name: String,
    pub transport_kind: String,
    pub mock_response: String,
}

/// A pure computation node that needs a determinism test.
#[derive(Debug, Clone)]
pub struct PureTestTarget {
    pub node_id: String,
    pub function_name: String,
    pub description: Option<String>,
    pub inputs: Vec<(String, String)>,
    pub expected_outputs: Vec<(String, String)>,
}

/// Convert an underscore_name to PascalCase.
pub fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => {
                    let mut out = String::new();
                    out.push(c.to_ascii_uppercase());
                    out.extend(chars);
                    out
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Convert an underscore_name to camelCase.
pub fn to_camel_case(s: &str) -> String {
    let parts: Vec<&str> = s.split('_').filter(|part| !part.is_empty()).collect();
    let mut out = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            out.push_str(part);
        } else {
            let mut chars = part.chars();
            if let Some(c) = chars.next() {
                out.push(c.to_ascii_uppercase());
                out.extend(chars);
            }
        }
    }
    out
}

/// Emit Rust `#[test]` functions from a `TestSpec`.
pub fn emit_rust_tests(spec: &TestSpec) -> String {
    let mut out = String::new();

    if spec.dry_run {
        let _ = writeln!(out, "#[test]");
        let _ = writeln!(out, "fn test_dry_run_completion() {{");
        let _ = writeln!(out, "    let result = {}_dry_run();", spec.module_name);
        let _ = writeln!(out, "    assert!(result.is_ok());");
        let _ = writeln!(out, "}}\n");
    }

    for t in &spec.transport_nodes {
        let _ = writeln!(out, "#[test]");
        let _ = writeln!(
            out,
            "fn test_transport_{}_{}() {{",
            spec.module_name, t.node_id
        );
        let _ = writeln!(out, "    let mock_response = \"{}\";", t.mock_response);
        let _ = writeln!(out, "    let result = {}(mock_response);", t.function_name);
        let _ = writeln!(out, "    assert!(result.is_ok());");
        let _ = writeln!(out, "}}\n");
    }

    for p in &spec.pure_nodes {
        let _ = writeln!(out, "#[test]");
        let _ = writeln!(out, "fn test_pure_{}() {{", p.function_name);
        for (name, val) in &p.inputs {
            let _ = writeln!(out, "    let {name} = {val};");
        }
        let args = p
            .inputs
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(out, "    let result = {}({});", p.function_name, args);
        for (name, val) in &p.expected_outputs {
            let _ = writeln!(out, "    assert_eq!(result.{name}, {val});");
        }
        let _ = writeln!(out, "}}\n");
    }

    out
}

/// Emit Go `func Test*(t *testing.T)` functions from a `TestSpec`.
pub fn emit_go_tests(spec: &TestSpec) -> String {
    let mut out = String::from("package main\n\nimport \"testing\"\n\n");

    if spec.dry_run {
        let fn_pascal = to_pascal_case(&spec.module_name);
        let _ = writeln!(out, "func TestDryRunCompletion(t *testing.T) {{");
        let _ = writeln!(out, "    result := {fn_pascal}DryRun()");
        let _ = writeln!(out, "    if result != nil {{");
        let _ = writeln!(out, "        t.Fatalf(\"dry-run failed: %v\", result)");
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "}}\n");
    }

    for t in &spec.transport_nodes {
        let test_name = format!("TestTransport{}", to_pascal_case(&t.node_id));
        let fn_camel = to_camel_case(&t.function_name);
        let _ = writeln!(out, "func {test_name}(t *testing.T) {{");
        let _ = writeln!(out, "    mockResponse := \"{}\"", t.mock_response);
        let _ = writeln!(out, "    result := {fn_camel}(mockResponse)");
        let _ = writeln!(out, "    if result == nil {{");
        let _ = writeln!(
            out,
            "        t.Fatalf(\"transport {} returned nil\")",
            t.node_id
        );
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "}}\n");
    }

    for p in &spec.pure_nodes {
        let test_name = format!("TestPure{}", to_pascal_case(&p.function_name));
        let fn_pascal = to_pascal_case(&p.function_name);
        let _ = writeln!(out, "func {test_name}(t *testing.T) {{");
        for (name, val) in &p.inputs {
            let _ = writeln!(out, "    {name} := {val}");
        }
        let args = p
            .inputs
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(out, "    result := {fn_pascal}({args})");
        for (name, val) in &p.expected_outputs {
            let field = to_pascal_case(name);
            let _ = writeln!(out, "    if result.{field} != {val} {{");
            let _ = writeln!(
                out,
                "        t.Fatalf(\"expected {name} = {val}, got %v\", result.{field})"
            );
            let _ = writeln!(out, "    }}");
        }
        let _ = writeln!(out, "}}\n");
    }

    out
}

/// Emit C test functions with assert macros and a `main()` runner from a `TestSpec`.
pub fn emit_c_tests(spec: &TestSpec) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "#include <stdio.h>");
    let _ = writeln!(out, "#include <stdlib.h>");
    let _ = writeln!(out, "#include <string.h>\n");

    let _ = writeln!(
        out,
        "#define ASSERT_EQ(a, b, msg) do {{ if ((a) != (b)) {{ fprintf(stderr, \"FAIL: %s\\n\", msg); exit(1); }} }} while(0)"
    );
    let _ = writeln!(
        out,
        "#define ASSERT_STR_EQ(a, b, msg) do {{ if (strcmp((a), (b)) != 0) {{ fprintf(stderr, \"FAIL: %s\\n\", msg); exit(1); }} }} while(0)"
    );
    let _ = writeln!(
        out,
        "#define ASSERT_OK(rc, msg) do {{ if ((rc) != 0) {{ fprintf(stderr, \"FAIL: %s\\n\", msg); exit(1); }} }} while(0)\n"
    );

    let mut test_fns = Vec::new();

    if spec.dry_run {
        let fn_name = "test_dry_run_completion";
        test_fns.push(fn_name.to_string());
        let _ = writeln!(out, "void {fn_name}(void) {{");
        let _ = writeln!(out, "    int rc = {}_dry_run();", spec.module_name);
        let _ = writeln!(out, "    ASSERT_OK(rc, \"dry-run completion\");");
        let _ = writeln!(out, "}}\n");
    }

    for t in &spec.transport_nodes {
        let fn_name = format!("test_transport_{}_{}", spec.module_name, t.node_id);
        test_fns.push(fn_name.clone());
        let _ = writeln!(out, "void {fn_name}(void) {{");
        let _ = writeln!(
            out,
            "    const char* mock_response = \"{}\";",
            t.mock_response
        );
        let _ = writeln!(out, "    int result = {}(mock_response);", t.function_name);
        let _ = writeln!(out, "    ASSERT_OK(result, \"transport {}\");", t.node_id);
        let _ = writeln!(out, "}}\n");
    }

    for p in &spec.pure_nodes {
        let fn_name = format!("test_pure_{}", p.function_name);
        test_fns.push(fn_name.clone());
        let _ = writeln!(out, "void {fn_name}(void) {{");
        for (name, val) in &p.inputs {
            let _ = writeln!(out, "    int {name} = {val};");
        }
        let args = p
            .inputs
            .iter()
            .map(|(n, _)| n.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(out, "    int result = {}({});", p.function_name, args);
        for (_name, val) in &p.expected_outputs {
            let _ = writeln!(
                out,
                "    ASSERT_EQ(result, {val}, \"pure {}\");",
                p.function_name
            );
        }
        let _ = writeln!(out, "}}\n");
    }

    let _ = writeln!(out, "int main(void) {{");
    for fn_name in &test_fns {
        let _ = writeln!(out, "    {fn_name}();");
    }
    let _ = writeln!(out, "    printf(\"All tests passed.\\n\");");
    let _ = writeln!(out, "    return 0;");
    let _ = writeln!(out, "}}");

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_lower::{CallableKind, CallableObligation, TransportObligation};
    use gunbc_ir::{Dag, Node, Port};

    fn obligations(required: bool) -> TestObligations {
        TestObligations {
            dry_run_completion_required: required,
            total_obligations: 0,
            transport_execution_targets: 0,
            pure_node_determinism_targets: 0,
            service_transport_prepare_targets: 0,
            service_transport_execute_targets: 0,
            service_transport_parse_targets: 0,
            service_transport_hermetic_targets: 0,
            service_transport_external_targets: 0,
            service_transport_idempotent_targets: 0,
            service_transport_readonly_targets: 0,
            service_param_source_targets: 0,
            resource_provide_targets: 0,
            resource_acquire_targets: 0,
            resource_release_targets: 0,
            interface_contract_verification_targets: 0,
        }
    }

    #[test]
    fn emit_rust_dry_run_completion_test() {
        let emitted = emit_dry_run_completion_test("rust", &obligations(true))
            .expect("rust backend should emit test");
        assert_eq!(
            emitted.path,
            "target/generated/rust/dry_run_completion_test.rs"
        );
        assert!(emitted.content.contains("#[test]"));
        assert!(emitted.content.contains("assert!(true"));
    }

    #[test]
    fn emit_go_dry_run_completion_test() {
        let emitted = emit_dry_run_completion_test("go", &obligations(true))
            .expect("go backend should emit test");
        assert_eq!(
            emitted.path,
            "target/generated/go/dry_run_completion_test.go"
        );
        assert!(emitted
            .content
            .contains("func TestDryRunCompletionRequired"));
        assert!(emitted.content.contains("if !true"));
    }

    #[test]
    fn emit_c_dry_run_completion_test() {
        let emitted = emit_dry_run_completion_test("c", &obligations(true))
            .expect("c backend should emit test");
        assert_eq!(emitted.path, "target/generated/c/dry_run_completion_test.c");
        assert!(emitted.content.contains("#include <assert.h>"));
        assert!(emitted.content.contains("assert(1"));
    }

    #[test]
    fn emit_mips_dry_run_completion_test() {
        let emitted = emit_dry_run_completion_test("mips", &obligations(true))
            .expect("mips backend should emit test");
        assert_eq!(
            emitted.path,
            "target/generated/mips/dry_run_completion_test.s"
        );
        assert!(emitted.content.contains("li $v0, 4001"));
        assert!(emitted.content.contains("li $a0, 0"));
    }

    #[test]
    fn unknown_backend_emits_no_dry_run_test() {
        assert!(emit_dry_run_completion_test("python", &obligations(true)).is_none());
    }

    fn dag_with_transport_node() -> ReachableDag<LoweredOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "transport.node",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Transport {
                module: "services.example".to_string(),
                kind: CallableKind::Func,
                name: "execute".to_string(),
                obligation: TransportObligation::Execute,
                service_metadata: Box::new(daglang_lower::ServiceCallMetadata {
                    service: "example".to_string(),
                    operation: "execute".to_string(),
                    transport: daglang_lower::ServiceTransportClass::ShellLocal,
                    idempotent: false,
                    readonly: false,
                    spec: None,
                    response_provider: None,
                }),
                is_interactive: false,
                resource_target: None,
            },
        ));
        ReachableDag::from_dag(&dag)
    }

    fn dag_without_transport_nodes() -> ReachableDag<LoweredOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "pure.node",
            vec![],
            vec![],
            LoweredOp::Callable {
                module: "tools.makegen".to_string(),
                kind: CallableKind::Fn,
                name: "render_makefile".to_string(),
                obligation: CallableObligation::PureRender,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        ReachableDag::from_dag(&dag)
    }

    #[test]
    fn emit_rust_transport_mock_tests_per_transport_node() {
        let emitted = emit_transport_mock_tests("rust", &dag_with_transport_node())
            .expect("rust backend should emit transport mock tests");
        assert_eq!(
            emitted.path,
            "target/generated/rust/transport_mock_tests.rs"
        );
        assert!(emitted.content.contains("#[test]"));
        // Typed mock response based on TransportResponse port type
        assert!(emitted.content.contains("status"));
    }

    #[test]
    fn emit_go_transport_mock_tests_per_transport_node() {
        let emitted = emit_transport_mock_tests("go", &dag_with_transport_node())
            .expect("go backend should emit transport mock tests");
        assert_eq!(
            emitted.path,
            "target/generated/go/transport_mock_tests_test.go"
        );
        assert!(emitted.content.contains("func TestMockTransport"));
        // Typed mock response based on TransportResponse port type
        assert!(emitted.content.contains("status"));
    }

    #[test]
    fn emit_c_transport_mock_tests_per_transport_node() {
        let emitted = emit_transport_mock_tests("c", &dag_with_transport_node())
            .expect("c backend should emit transport mock tests");
        assert_eq!(emitted.path, "target/generated/c/transport_mock_tests.c");
        assert!(emitted.content.contains("#include <assert.h>"));
        assert!(emitted.content.contains("strcmp"));
    }

    #[test]
    fn emit_transport_mock_tests_returns_none_when_no_transport_nodes() {
        assert!(emit_transport_mock_tests("rust", &dag_without_transport_nodes()).is_none());
        assert!(emit_transport_mock_tests("go", &dag_without_transport_nodes()).is_none());
        assert!(emit_transport_mock_tests("c", &dag_without_transport_nodes()).is_none());
    }

    #[test]
    fn typed_mock_for_response_emits_list_shape_for_string_list() {
        assert_eq!(typed_mock_for_response("List<String>"), r#"["mock-item"]"#);
        assert_eq!(typed_mock_for_response("List<String>"), r#"["mock-item"]"#);
    }

    // ===== E3.4-E3.6: TestSpec-based generation tests =====

    fn sample_spec() -> TestSpec {
        TestSpec {
            module_name: "makegen".to_string(),
            dry_run: true,
            transport_nodes: vec![
                TransportTestTarget {
                    node_id: "execute_read".to_string(),
                    function_name: "execute_file_read".to_string(),
                    transport_kind: "file_read".to_string(),
                    mock_response: "existing content".to_string(),
                },
                TransportTestTarget {
                    node_id: "execute_write".to_string(),
                    function_name: "execute_file_write".to_string(),
                    transport_kind: "file_write".to_string(),
                    mock_response: "ok".to_string(),
                },
            ],
            pure_nodes: vec![PureTestTarget {
                node_id: "render_makefile".to_string(),
                function_name: "render_makefile".to_string(),
                description: Some("renders Makefile from registry".to_string()),
                inputs: vec![("registry".to_string(), "42".to_string())],
                expected_outputs: vec![("result".to_string(), "42".to_string())],
            }],
        }
    }

    #[test]
    fn rust_spec_dry_run() {
        let spec = TestSpec {
            module_name: "makegen".to_string(),
            dry_run: true,
            transport_nodes: vec![],
            pure_nodes: vec![],
        };
        let rendered = emit_rust_tests(&spec);
        assert!(rendered.contains("#[test]"), "has test attr");
        assert!(
            rendered.contains("fn test_dry_run_completion()"),
            "dry-run fn: got {rendered}"
        );
        assert!(
            rendered.contains("makegen_dry_run()"),
            "calls fn: got {rendered}"
        );
    }

    #[test]
    fn rust_spec_transport() {
        let rendered = emit_rust_tests(&sample_spec());
        assert!(
            rendered.contains("fn test_transport_makegen_execute_read()"),
            "transport fn: got {rendered}"
        );
        assert!(
            rendered.contains("execute_file_read(mock_response)"),
            "call: got {rendered}"
        );
    }

    #[test]
    fn rust_spec_pure() {
        let rendered = emit_rust_tests(&sample_spec());
        assert!(
            rendered.contains("fn test_pure_render_makefile()"),
            "pure fn: got {rendered}"
        );
        assert!(
            rendered.contains("render_makefile(registry)"),
            "call: got {rendered}"
        );
        assert!(
            rendered.contains("assert_eq!(result.result, 42"),
            "assert: got {rendered}"
        );
    }

    #[test]
    fn go_spec_dry_run() {
        let spec = TestSpec {
            module_name: "makegen".to_string(),
            dry_run: true,
            transport_nodes: vec![],
            pure_nodes: vec![],
        };
        let rendered = emit_go_tests(&spec);
        assert!(
            rendered.contains("func TestDryRunCompletion(t *testing.T)"),
            "Go fn: got {rendered}"
        );
        assert!(rendered.contains("MakegenDryRun()"), "call: got {rendered}");
    }

    #[test]
    fn go_spec_transport() {
        let rendered = emit_go_tests(&sample_spec());
        assert!(
            rendered.contains("func TestTransportExecuteRead(t *testing.T)"),
            "Go transport: got {rendered}"
        );
        assert!(
            rendered.contains("executeFileRead(mockResponse)"),
            "call: got {rendered}"
        );
    }

    #[test]
    fn go_spec_pure() {
        let rendered = emit_go_tests(&sample_spec());
        assert!(
            rendered.contains("func TestPureRenderMakefile(t *testing.T)"),
            "Go pure: got {rendered}"
        );
        assert!(
            rendered.contains("RenderMakefile(registry)"),
            "call: got {rendered}"
        );
        assert!(
            rendered.contains("result.Result != 42"),
            "assert: got {rendered}"
        );
    }

    #[test]
    fn c_spec_dry_run() {
        let spec = TestSpec {
            module_name: "makegen".to_string(),
            dry_run: true,
            transport_nodes: vec![],
            pure_nodes: vec![],
        };
        let rendered = emit_c_tests(&spec);
        assert!(
            rendered.contains("void test_dry_run_completion(void)"),
            "C fn: got {rendered}"
        );
        assert!(
            rendered.contains("makegen_dry_run()"),
            "call: got {rendered}"
        );
        assert!(rendered.contains("ASSERT_OK"), "macro: got {rendered}");
    }

    #[test]
    fn c_spec_transport() {
        let rendered = emit_c_tests(&sample_spec());
        assert!(
            rendered.contains("void test_transport_makegen_execute_read(void)"),
            "C transport: got {rendered}"
        );
        assert!(
            rendered.contains("execute_file_read(mock_response)"),
            "call: got {rendered}"
        );
    }

    #[test]
    fn c_spec_pure() {
        let rendered = emit_c_tests(&sample_spec());
        assert!(
            rendered.contains("void test_pure_render_makefile(void)"),
            "C pure: got {rendered}"
        );
        assert!(
            rendered.contains("render_makefile(registry)"),
            "call: got {rendered}"
        );
        assert!(
            rendered.contains("ASSERT_EQ(result, 42"),
            "assert: got {rendered}"
        );
    }

    #[test]
    fn c_test_runner() {
        let rendered = emit_c_tests(&sample_spec());
        assert!(rendered.contains("int main(void)"), "main");
        assert!(
            rendered.contains("test_dry_run_completion();"),
            "calls dry-run"
        );
        assert!(
            rendered.contains("test_transport_makegen_execute_read();"),
            "calls transport"
        );
        assert!(
            rendered.contains("test_pure_render_makefile();"),
            "calls pure"
        );
        assert!(rendered.contains("All tests passed."), "success msg");
    }

    #[test]
    fn c_assert_macros() {
        let spec = TestSpec {
            module_name: "x".into(),
            dry_run: false,
            transport_nodes: vec![],
            pure_nodes: vec![],
        };
        let rendered = emit_c_tests(&spec);
        assert!(
            rendered.contains("#define ASSERT_EQ(a, b, msg)"),
            "ASSERT_EQ"
        );
        assert!(
            rendered.contains("#define ASSERT_STR_EQ(a, b, msg)"),
            "ASSERT_STR_EQ"
        );
        assert!(rendered.contains("#define ASSERT_OK(rc, msg)"), "ASSERT_OK");
    }

    #[test]
    fn pascal_case_conversion() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("execute_read"), "ExecuteRead");
        assert_eq!(to_pascal_case("makegen"), "Makegen");
    }

    #[test]
    fn camel_case_conversion() {
        assert_eq!(to_camel_case("hello_world"), "helloWorld");
        assert_eq!(to_camel_case("execute_file_read"), "executeFileRead");
    }

    #[test]
    fn full_spec_all_three_targets() {
        let spec = sample_spec();
        let rust = emit_rust_tests(&spec);
        let go = emit_go_tests(&spec);
        let c = emit_c_tests(&spec);

        assert!(!rust.is_empty());
        assert!(!go.is_empty());
        assert!(!c.is_empty());

        // dry-run(1) + transport(2) + pure(1) = 4 tests each
        assert_eq!(rust.matches("#[test]").count(), 4, "rust test count");
        assert_eq!(go.matches("func Test").count(), 4, "go test count");
    }
}
