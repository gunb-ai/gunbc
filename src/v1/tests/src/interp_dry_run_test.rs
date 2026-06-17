//! Regression: `gunbc --dry-run run` must pass `cli.dry_run` into the interpreter
//! so REST service calls return modeled `mock_response` data instead of live HTTP.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{read_v2_file, resolve_imports_transitively};

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
}

fn resolve(src: &str) -> Rc<ResolvedPipelineResult> {
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    resolved
}

#[test]
fn run_subcommand_wires_cli_dry_run_flag() {
    let main_rs = read_v2_file("src/v1/stage0/src/main.rs");
    assert!(
        main_rs.contains("handle_run_with_options") && main_rs.contains("cli.dry_run"),
        "main.rs must forward global --dry-run into handle_run_with_options"
    );

    let emit_dag = read_v2_file("src/v1/05_emit_rust.dag");
    assert!(
        emit_dag.contains("handle_run_with_options") && emit_dag.contains("cli.dry_run"),
        "05_emit_rust.dag must emit cli.dry_run wiring for the Run subcommand"
    );
}

#[test]
fn interpreter_dry_run_returns_modeled_rest_mock_response() {
    let src = r#"module test.dry_run_mock

service test.Svc {
  config {
    endpoint: "https://api.example.com"
  }
  operation GetData {
    output { data: String }
    transport rest { method: GET, path: "/data" }
    response {
      200 => String
    }
    mock_response {
      200 => { data: "dry-run-mock" }
    }
  }
}

fn witness() -> String {
  let r = test.Svc.GetData()
  r.data
}
"#;
    let resolved = resolve(src);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(graph, resolved.source_indices.clone(), true);

    match v1_interpreter::run_in_context(&ctx, "witness", false) {
        Ok(Value::Str(s)) => assert_eq!(s, "dry-run-mock"),
        other => panic!("expected mock String, got {other:?}"),
    }
}
