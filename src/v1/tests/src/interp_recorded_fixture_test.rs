//! Phase 2 hermetic rollout: record/replay faithfulness for service operations.
//!
//! RecordedFixture is keyed by content_hash(inputs). `--record` wet-captures;
//! `--hermetic --fixture-store` replays. Staleness is fail-closed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use v1_compiler::recorded_fixture::RecordedFixtureStore;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

use crate::helpers::{resolve_imports_transitively, workspace_root};

fn claim_batch_exe() -> PathBuf {
    let ws = workspace_root();
    let release = ws.join("target/release/claim_batch");
    if release.is_file() {
        return release;
    }
    ws.join("target/debug/claim_batch")
}

fn fixture_store_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "gunbc-recorded-fixture-{}-{}-{}",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

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

fn run_claim_batch(args: &[&str]) -> std::process::Output {
    let exe = claim_batch_exe();
    assert!(
        exe.is_file(),
        "claim_batch binary missing at {}; build with `cargo build -p v1-compiler --bin claim_batch`",
        exe.display()
    );
    let mut cmd = Command::new(&exe);
    cmd.current_dir(workspace_root());
    for arg in args {
        cmd.arg(arg);
    }
    cmd.output().expect("claim_batch")
}

#[test]
fn filesystem_write_witness_record_then_hermetic_replay_holds() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fs-write-record-replay");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/filesystem_write_witness.dag");
    assert!(entry.is_file(), "witness dag must exist at {}", entry.display());

    // Wet capture: record live Filesystem.Read/Write responses.
    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "filesystem_write_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(
        record.status.success(),
        "record capture must pass (wet I/O); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );
    assert!(
        store_dir.join("Filesystem__Write").is_dir() || !fixture_files(&store_dir).is_empty(),
        "record must write fixture files under {:?}",
        store_dir
    );

    // Hermetic replay from fixture store — no live I/O.
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "filesystem_write_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        hermetic.status.success(),
        "hermetic replay must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
}

#[test]
fn hermetic_fixture_staleness_fails_closed() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fs-stale");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/filesystem_write_witness.dag");

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "witness_read_absent_fails_closed",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(record.status.success(), "record must capture absent-read");

    // Tamper: rewrite every fixture's inputs_hash to force staleness on replay.
    for path in fixture_files(&store_dir) {
        let bytes = fs::read(&path).expect("read fixture");
        let mut fixture: serde_json::Value =
            serde_json::from_slice(&bytes).expect("parse fixture");
        if let Some(obj) = fixture.as_object_mut() {
            obj.insert(
                "inputs_hash".to_string(),
                serde_json::Value::String("deadbeefdeadbeef".to_string()),
            );
        }
        fs::write(&path, serde_json::to_vec_pretty(&fixture).expect("serialize")).expect("write");
    }

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "witness_read_absent_fails_closed",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        !hermetic.status.success(),
        "stale fixture must fail closed, not replay stale value"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&hermetic.stdout),
        String::from_utf8_lossy(&hermetic.stderr)
    );
    assert!(
        combined.contains("stale recorded fixture"),
        "expected staleness diagnostic, got:\n{combined}"
    );
}

#[test]
fn hermetic_without_fixture_store_still_uses_mock_response_for_rest() {
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
    let resolved = {
        let sources = resolve_imports_transitively("test.dag", src);
        let resolved = compile_to_resolved(Rc::new(sources));
        assert_resolved_no_hard_errors(&resolved);
        resolved
    };
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Hermetic,
    );

    match v1_interpreter::run_in_context(&ctx, "witness", false) {
        Ok(Value::Str(s)) => assert_eq!(s, "dry-run-mock"),
        other => panic!("expected mock String, got {other:?}"),
    }
}

#[test]
fn recorded_fixture_store_roundtrip_value() {
    let src = r#"module test.fixture_roundtrip

fn witness() -> Bool {
  let r = { success: true, bytes_written: 42, path: "/tmp/x", error: "" }
  r.success && (r.bytes_written == 42)
}
"#;
    let sources = resolve_imports_transitively("test.dag", src);
    let resolved = compile_to_resolved(Rc::new(sources));
    assert_resolved_no_hard_errors(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    let val = v1_interpreter::run_in_context(&ctx, "witness", false).expect("witness runs");
    let store_dir = fixture_store_dir("value-roundtrip");
    let store = RecordedFixtureStore::open(&store_dir);
    store
        .record("Filesystem.Write", "0123456789abcdef", &val, &ctx)
        .expect("record");
    let fixture = store
        .lookup("Filesystem.Write", "0123456789abcdef")
        .expect("lookup");
    let back = v1_compiler::recorded_fixture::value_from_fixture_json(&fixture.response, &ctx);
    assert_eq!(val, back, "fixture JSON round-trip must preserve Value");
    let _ = fs::remove_dir_all(&store_dir);
}

fn fixture_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_json_files(dir, &mut out);
    out
}

fn collect_json_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).expect("read fixture dir");
    for entry in entries {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, out);
        } else if path.extension().map(|e| e == "json").unwrap_or(false) {
            out.push(path);
        }
    }
}
