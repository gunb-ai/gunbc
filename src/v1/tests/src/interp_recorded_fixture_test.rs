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
    assert!(
        entry.is_file(),
        "witness dag must exist at {}",
        entry.display()
    );

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

    // Tamper: backdate recorded_at to force freshness-window expiry on replay.
    for path in fixture_files(&store_dir) {
        let bytes = fs::read(&path).expect("read fixture");
        let mut fixture: serde_json::Value = serde_json::from_slice(&bytes).expect("parse fixture");
        if let Some(obj) = fixture.as_object_mut() {
            obj.insert("recorded_at".to_string(), serde_json::json!(0u64));
        }
        fs::write(
            &path,
            serde_json::to_vec_pretty(&fixture).expect("serialize"),
        )
        .expect("write");
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
        combined.contains("expired recorded fixture")
            || combined.contains("refusing to replay stale value"),
        "expected staleness diagnostic, got:\n{combined}"
    );
}

#[test]
fn record_response_drift_for_same_input_hash_fails_closed() {
    let src = r#"module test.fixture_roundtrip

fn witness() -> Bool {
  let r = { success: true, bytes_written: 42, path: "/tmp/x", error: "" }
  r.success
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
    let val_a = v1_interpreter::run_in_context(&ctx, "witness", false).expect("witness runs");
    let store_dir = fixture_store_dir("response-drift");
    let store = RecordedFixtureStore::open(&store_dir);
    let empty_inputs = serde_json::json!([]);
    store
        .record(
            "Filesystem.Write",
            "0123456789abcdef",
            &empty_inputs,
            &val_a,
            &ctx,
            v1_interpreter::fixture_now_secs(&ctx).expect("clock"),
        )
        .expect("first record");

    let val_b_src = r#"module test.fixture_roundtrip

fn witness() -> Bool {
  let r = { success: false, bytes_written: 0, path: "/tmp/y", error: "changed" }
  r.success
}
"#;
    let sources_b = resolve_imports_transitively("test.dag", val_b_src);
    let resolved_b = compile_to_resolved(Rc::new(sources_b));
    assert_resolved_no_hard_errors(&resolved_b);
    let graph_b = resolved_b.graph.as_ref().expect("graph");
    let ctx_b = v1_interpreter::InterpContext::new(
        graph_b,
        resolved_b.source_indices.clone(),
        ExecutionMode::Wet,
    );
    let val_b = v1_interpreter::run_in_context(&ctx_b, "witness", false).expect("witness b");

    let err = store
        .record(
            "Filesystem.Write",
            "0123456789abcdef",
            &empty_inputs,
            &val_b,
            &ctx_b,
            v1_interpreter::fixture_now_secs(&ctx_b).expect("clock"),
        )
        .expect_err("same input_hash with different response must fail closed");
    assert!(
        err.to_string().contains("response drift"),
        "expected response drift diagnostic, got: {err}"
    );
    let _ = fs::remove_dir_all(&store_dir);
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
    let empty_inputs = serde_json::json!([]);
    store
        .record(
            "Filesystem.Write",
            "0123456789abcdef",
            &empty_inputs,
            &val,
            &ctx,
            v1_interpreter::fixture_now_secs(&ctx).expect("clock"),
        )
        .expect("record");
    let now = v1_interpreter::fixture_now_secs(&ctx).expect("clock");
    let fixture = store
        .lookup("Filesystem.Write", "0123456789abcdef", &empty_inputs, now)
        .expect("lookup");
    let back = v1_compiler::recorded_fixture::value_from_fixture_json(&fixture.response, &ctx)
        .expect("deserialize");
    assert_eq!(val, back, "fixture JSON round-trip must preserve Value");
    let _ = fs::remove_dir_all(&store_dir);
}

#[test]
fn hermetic_replay_rejects_corrupted_fixture_response() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("corrupt-response");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/filesystem_write_witness.dag");
    let target = "/tmp/gunbc_fs_write_witness.txt";

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "witness_write_then_read_roundtrip",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(record.status.success(), "record must capture write/read");

    // Tamper: mutate bytes_written in a recorded Write response fixture (nested tagged JSON).
    for path in fixture_files(&store_dir) {
        let bytes = fs::read(&path).expect("read fixture");
        let mut fixture: serde_json::Value = serde_json::from_slice(&bytes).expect("parse fixture");
        if let Some(response) = fixture.get_mut("response") {
            if let Some(fields) = response.get_mut("fields").and_then(|f| f.as_object_mut()) {
                if let Some(bw) = fields.get_mut("bytes_written") {
                    if let Some(val) = bw.get_mut("value") {
                        *val = serde_json::json!(99i64);
                    }
                }
            }
        }
        fs::write(
            &path,
            serde_json::to_vec_pretty(&fixture).expect("serialize"),
        )
        .expect("write tampered fixture");
    }

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "witness_write_then_read_roundtrip",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        !hermetic.status.success(),
        "corrupted fixture response must fail closed; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&hermetic.stdout),
        String::from_utf8_lossy(&hermetic.stderr)
    );
    assert!(
        combined.contains("FAIL") || combined.contains("deserialization"),
        "expected witness failure on corrupted fixture, got:\n{combined}"
    );
    let _ = target; // witness path (recorded under /tmp)
}

#[test]
fn hermetic_replay_uses_fixture_not_live_fs_after_mutation() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fixture-not-live");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/filesystem_write_witness.dag");
    let target = "/tmp/gunbc_fs_write_witness.txt";
    let payload = "hello from the v2 file transport\n";

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "witness_write_then_read_roundtrip",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(record.status.success(), "record must capture");

    // Mutate live filesystem AFTER record — hermetic must NOT observe this.
    fs::write(target, b"MUTATED-LIVE-FS-CONTENT").expect("mutate live file");

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "witness_write_then_read_roundtrip",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        hermetic.status.success(),
        "hermetic must replay recorded bytes, not live FS; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    let _ = payload; // original recorded payload
}

#[test]
fn filesystem_hermetic_without_fixture_store_fails_closed() {
    let ws = workspace_root();
    let entry = ws.join("dsl/test/claim/filesystem_write_witness.dag");
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
    ]);
    assert!(
        !hermetic.status.success(),
        "Filesystem op in Hermetic without fixture store must fail closed (no silent Unit)"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&hermetic.stdout),
        String::from_utf8_lossy(&hermetic.stderr)
    );
    assert!(
        combined.contains("no mock_response") || combined.contains("refusing to fabricate"),
        "expected fail-closed mock_response diagnostic, got:\n{combined}"
    );
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
