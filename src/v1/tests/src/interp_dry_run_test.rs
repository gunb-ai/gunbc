//! Regression: `gunbc --dry-run run` must pass `cli.dry_run` into the interpreter
//! so REST service calls return modeled `mock_response` data instead of live HTTP.
//! Hermetic `ExecutionMode` is the interpreter-side generalization of dry-run.

use std::fs;
use std::process::Command;
use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, Value};

use crate::helpers::{read_v2_file, resolve_imports_transitively, workspace_root};

fn cargo_binary() -> &'static str {
    if std::path::Path::new("/opt/cargo/bin/cargo").exists() {
        "/opt/cargo/bin/cargo"
    } else {
        "cargo"
    }
}

fn claim_batch_exe() -> std::path::PathBuf {
    workspace_root().join("target/debug/claim_batch")
}

fn ensure_claim_batch_built() {
    let exe = claim_batch_exe();
    if exe.exists() {
        return;
    }
    let output = Command::new(cargo_binary())
        .args([
            "build",
            "-p",
            "v1-compiler",
            "--bin",
            "claim_batch",
            "--quiet",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("spawn cargo build claim_batch");
    assert!(
        output.status.success(),
        "failed to build claim_batch: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn hermetic_witness_temp_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
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
fn claim_batch_hermetic_flag_uses_mock_response_by_execution() {
    ensure_claim_batch_built();
    let root = hermetic_witness_temp_root();
    fs::create_dir_all(&root).expect("temp source root");
    fs::write(root.join("hermetic_witness.dag"), HERMETIC_WITNESS_DAG).expect("write witness");

    let root_s = root.to_string_lossy();
    let wet = {
        let mut cmd = Command::new(claim_batch_exe());
        cmd.arg("--source-root")
            .arg(root_s.as_ref())
            .arg("--entry")
            .arg("hermetic_witness.dag")
            .arg("--function")
            .arg("hermetic_witness_holds");
        cmd.output().expect("claim_batch wet (default)")
    };
    assert!(
        !wet.status.success(),
        "default Wet must fail without a live server; stderr={}",
        String::from_utf8_lossy(&wet.stderr)
    );

    let hermetic = {
        let mut cmd = Command::new(claim_batch_exe());
        cmd.arg("--source-root")
            .arg(root_s.as_ref())
            .arg("--entry")
            .arg("hermetic_witness.dag")
            .arg("--function")
            .arg("hermetic_witness_holds")
            .arg("--hermetic");
        cmd.output().expect("claim_batch --hermetic")
    };
    let _ = fs::remove_dir_all(&root);
    assert!(
        hermetic.status.success(),
        "claim_batch --hermetic must pass via mock_response; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
}

#[test]
fn claim_batch_wires_execution_mode_flags() {
    let claim_batch = read_v2_file("src/v1/stage0/src/bin/claim_batch.rs");
    assert!(
        claim_batch.contains("\"--wet\"")
            && claim_batch.contains("\"--hermetic\"")
            && claim_batch.contains("let mut wet = true")
            && claim_batch.contains("ExecutionMode::Hermetic")
            && claim_batch.contains("ExecutionMode::Wet"),
        "claim_batch must default Wet (CI unchanged) and accept --wet/--hermetic"
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
