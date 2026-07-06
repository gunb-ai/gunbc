use std::fs;
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{read_v2_file, resolve_imports_transitively, workspace_root};

fn claim_batch_exe() -> std::path::PathBuf {
    let ws = workspace_root();
    let release = ws.join("target/release/claim_batch");
    if release.is_file() {
        return release;
    }
    ws.join("target/debug/claim_batch")
}

fn hermetic_witness_temp_root() -> std::path::PathBuf {
    workspace_root().join("target").join(format!(
        "gunbc-claim-batch-hermetic-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

const HERMETIC_WITNESS_DAG: &str = r#"module test.claim_batch_hermetic

service test.Svc {
  config {
    endpoint: "https://unreachable.invalid.example"
  }
  operation Ping {
    output { token: String }
    transport rest { method: GET, path: "/ping" }
    response {
      200 => String
    }
    mock_response {
      200 => { token: "hermetic-claim-batch-token" }
    }
  }
}

fn hermetic_witness_holds() -> Bool {
  let r = test.Svc.Ping()
  r.token == "hermetic-claim-batch-token"
}
"#;

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
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
    let resolved = compile_to_resolved(Rc::new(sources.into()));
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
fn claim_batch_hermetic_flag_uses_mock_response_by_execution() {
    let exe = claim_batch_exe();
    assert!(
        exe.is_file(),
        "claim_batch binary missing at {}; build with `cargo build -p v1-compiler --bin claim_batch`",
        exe.display()
    );
    let root = hermetic_witness_temp_root();
    fs::create_dir_all(&root).expect("temp source root");
    let dag_path = root.join("hermetic_witness.dag");
    fs::write(&dag_path, HERMETIC_WITNESS_DAG).expect("write witness");

    let root_s = root.to_string_lossy();
    let entry_s = dag_path.to_string_lossy();
    let wet = {
        let mut cmd = Command::new(&exe);
        cmd.arg("--source-root")
            .arg(root_s.as_ref())
            .arg("--entry")
            .arg(entry_s.as_ref())
            .arg("--function")
            .arg("hermetic_witness_holds")
            .arg("--wet");
        cmd.output().expect("claim_batch --wet")
    };
    assert!(
        !wet.status.success(),
        "explicit --wet must fail without a live server; stderr={}",
        String::from_utf8_lossy(&wet.stderr)
    );

    let hermetic = {
        let mut cmd = Command::new(&exe);
        cmd.arg("--source-root")
            .arg(root_s.as_ref())
            .arg("--entry")
            .arg(entry_s.as_ref())
            .arg("--function")
            .arg("hermetic_witness_holds");
        cmd.output().expect("claim_batch default hermetic")
    };
    let _ = fs::remove_dir_all(&root);
    assert!(
        hermetic.status.success(),
        "claim_batch default Hermetic must pass via mock_response; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
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
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        v1_interpreter::ExecutionMode::Hermetic,
    );

    match v1_interpreter::run_in_context(&ctx, "witness", false) {
        Ok(Value::Str(s)) => assert_eq!(s, "dry-run-mock"),
        other => panic!("expected mock String, got {other:?}"),
    }
}
