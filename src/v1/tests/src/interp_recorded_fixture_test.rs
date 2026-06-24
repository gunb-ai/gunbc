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

fn unique_fs_witness_entry(ws: &Path, scratch: &Path) -> PathBuf {
    let src = fs::read_to_string(ws.join("dsl/test/claim/filesystem_write_witness.dag"))
        .expect("read filesystem witness dag");
    let live = scratch.join("fs_witness");
    let absent = scratch.join("fs_witness_absent_should_not_exist_42");
    let rewritten = src
        .replace(
            "/tmp/gunbc_fs_witness_absent_should_not_exist_42",
            absent.to_str().expect("utf8 scratch path"),
        )
        .replace(
            "/tmp/gunbc_fs_write_witness",
            live.to_str().expect("utf8 scratch path"),
        );
    let entry = scratch.join("filesystem_write_witness.dag");
    fs::write(&entry, rewritten).expect("write rewritten filesystem witness dag");
    entry
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
    let entry = unique_fs_witness_entry(&ws, &store_dir);

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

// Regression: a large import closure that pulls BOTH the `extdeps.filesystem`
// `service Filesystem` and the `std.resources` `resource Filesystem` used to drop
// the service's operations at runtime (-> "unknown service operation:
// Filesystem.Write"), because runtime service-op registration gated on a
// name-keyed `item_registry` lookup that the same-named non-service entry won the
// merge of. The record phase below is wet (live I/O) and exercises that
// registration; pre-fix it fails at record time, post-fix it records then replays.
fn closure_scale_witness_entry(ws: &Path, scratch: &Path) -> PathBuf {
    let src =
        fs::read_to_string(ws.join("dsl/test/claim/filesystem_write_closure_scale_witness.dag"))
            .expect("read closure-scale witness dag");
    let live = scratch.join("fs_closure_scale_witness.txt");
    let rewritten = src.replace(
        "/tmp/gunbc_fs_closure_scale_witness.txt",
        live.to_str().expect("utf8 scratch path"),
    );
    let entry = scratch.join("filesystem_write_closure_scale_witness.dag");
    fs::write(&entry, rewritten).expect("write rewritten closure-scale witness dag");
    entry
}

#[test]
fn filesystem_write_closure_scale_record_then_hermetic_replay_holds() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fs-closure-scale-record-replay");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = closure_scale_witness_entry(&ws, &store_dir);

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "filesystem_write_closure_scale_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(
        record.status.success(),
        "closure-scale record capture must pass: the Filesystem service must register \
         even when std.resources' resource Filesystem is in the same import closure; stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "filesystem_write_closure_scale_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        hermetic.status.success(),
        "hermetic replay of the closure-scale witness must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
}

#[test]
fn hermetic_fixture_staleness_fails_closed() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fs-stale");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = unique_fs_witness_entry(&ws, &store_dir);

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
    let entry = unique_fs_witness_entry(&ws, &store_dir);
    let target = store_dir.join("fs_witness.txt");

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
    let _ = target;
}

#[test]
fn hermetic_replay_uses_fixture_not_live_fs_after_mutation() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fixture-not-live");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = unique_fs_witness_entry(&ws, &store_dir);
    let target = store_dir.join("fs_witness.txt");
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

    fs::write(&target, b"MUTATED-LIVE-FS-CONTENT").expect("mutate live file");

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
    let _ = payload;
}

#[test]
fn filesystem_hermetic_without_fixture_store_fails_closed() {
    let ws = workspace_root();
    let scratch = fixture_store_dir("fs-no-store-fails-closed");
    fs::create_dir_all(&scratch).expect("scratch dir");
    let entry = unique_fs_witness_entry(&ws, &scratch);
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
    let _ = fs::remove_dir_all(&scratch);
}

#[test]
fn filesystem_read_hermetic_without_fixture_fails_closed() {
    let ws = workspace_root();
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        ws.join("dsl/test/claim/filesystem_read_hermetic_witness.dag")
            .to_str()
            .expect("entry"),
        "--function",
        "witness_read_via_builtin_roundtrip",
        "--hermetic",
    ]);
    assert!(
        !hermetic.status.success(),
        "filesystem_read in Hermetic without fixture store must fail closed (no silent disk read)"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&hermetic.stdout),
        String::from_utf8_lossy(&hermetic.stderr)
    );
    assert!(
        combined.contains("no mock_response")
            || combined.contains("refusing to fabricate")
            || combined.contains("refusing direct disk read"),
        "expected fail-closed hermetic diagnostic, got:\n{combined}"
    );
}

#[test]
fn filesystem_read_record_then_hermetic_replay_holds() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fs-read-builtin-record-replay");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let dsl_root = ws.join("dsl");
    let entry = ws.join("dsl/test/claim/filesystem_read_hermetic_witness.dag");
    let store_path = store_dir.to_str().expect("store path");

    let common = |mode_flag: &str| -> std::process::Output {
        run_claim_batch(&[
            "--source-root",
            ws.to_str().expect("workspace"),
            "--source-root",
            dsl_root.to_str().expect("dsl root"),
            "--entry",
            entry.to_str().expect("entry"),
            "--function",
            "witness_read_via_builtin_roundtrip",
            "--fixture-store",
            store_path,
            mode_flag,
        ])
    };

    let record = common("--record");
    assert!(
        record.status.success(),
        "record capture must pass (wet I/O via Filesystem.Read gate); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );
    assert!(
        store_dir.join("Filesystem__Read").is_dir() || !fixture_files(&store_dir).is_empty(),
        "record must write fixture files under {:?}",
        store_dir
    );

    let hermetic = common("--hermetic");
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        hermetic.status.success(),
        "hermetic replay of filesystem_read must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
}

#[test]
fn m4_governed_service_published_realizes_unpublished_fails_closed() {
    let ws = workspace_root();
    let common = |func: &str| -> std::process::Output {
        run_claim_batch(&[
            "--source-root",
            ws.to_str().expect("workspace"),
            "--source-root",
            ws.join("dsl").to_str().expect("dsl root"),
            "--entry",
            ws.join("dsl/test/claim/m4_governed_service_witness.dag")
                .to_str()
                .expect("entry"),
            "--function",
            func,
            "--hermetic",
        ])
    };

    let green = common("witness_published_realizes");
    assert!(
        green.status.success(),
        "published op must realize hermetically on the claim_batch path; stderr={}",
        String::from_utf8_lossy(&green.stderr)
    );

    let red = common("witness_unpublished_fails_closed");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&red.stdout),
        String::from_utf8_lossy(&red.stderr)
    );
    assert!(
        !red.status.success(),
        "unpublished op on a corpus-governed service must fail closed (non-zero exit); output:\n{combined}"
    );
    assert!(
        combined.contains("not a published mock case"),
        "expected published-corpus refusal diagnostic, got:\n{combined}"
    );
}

#[test]
fn gcp_oauth_access_token_materializer_holds() {
    let ws = workspace_root();
    let entry = ws.join("dsl/test/claim/gcp_oauth_access_token_witness.dag");
    let store = ws.join("dsl/test/fixture/gcp_oauth_access_token_store");
    let common = |func: &str| -> std::process::Output {
        run_claim_batch(&[
            "--source-root",
            ws.to_str().expect("workspace"),
            "--source-root",
            ws.join("dsl").to_str().expect("dsl root"),
            "--entry",
            entry.to_str().expect("entry"),
            "--function",
            func,
            "--hermetic",
            "--fixture-store",
            store.to_str().expect("fixture store"),
        ])
    };

    let green = common("gcp_oauth_access_token_materializer_green_holds");
    assert!(
        green.status.success(),
        "materializer green witness must pass hermetically; stderr={}",
        String::from_utf8_lossy(&green.stderr)
    );

    let red = common("gcp_oauth_access_token_dispatch_discriminator_is_red_holds");
    assert!(
        red.status.success(),
        "dispatch discriminator witness must pass (proves swapped arms are detectable); stderr={}",
        String::from_utf8_lossy(&red.stderr)
    );
}

#[test]
fn m4_universal_corpus_published_realizes_unpublished_fails_closed() {
    let ws = workspace_root();
    let common = |func: &str| -> std::process::Output {
        run_claim_batch(&[
            "--source-root",
            ws.to_str().expect("workspace"),
            "--source-root",
            ws.join("dsl").to_str().expect("dsl root"),
            "--entry",
            ws.join("dsl/test/claim/m4_universal_corpus_witness.dag")
                .to_str()
                .expect("entry"),
            "--function",
            func,
            "--hermetic",
        ])
    };

    let green = common("witness_universal_published_realizes");
    assert!(
        green.status.success(),
        "published op must realize with whole-tree corpus governance; stderr={}",
        String::from_utf8_lossy(&green.stderr)
    );

    let red = common("witness_universal_unpublished_fails_closed");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&red.stdout),
        String::from_utf8_lossy(&red.stderr)
    );
    assert!(
        !red.status.success(),
        "unpublished op must fail closed when corpus is outside entry closure; output:\n{combined}"
    );
    assert!(
        combined.contains("not a published mock case"),
        "expected published-corpus refusal diagnostic, got:\n{combined}"
    );
}

fn fixture_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_json_files(dir, &mut out);
    out
}

#[test]
fn clock_now_record_then_hermetic_replay_holds() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("clock-record-replay");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/clock_freshness_witness.dag");
    assert!(
        entry.is_file(),
        "witness dag must exist at {}",
        entry.display()
    );

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "clock_freshness_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(
        record.status.success(),
        "Clock.Now record capture must pass (wet); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );
    assert!(
        store_dir.join("Clock__Now").is_dir() || !fixture_files(&store_dir).is_empty(),
        "record must write Clock.Now fixture files under {:?}",
        store_dir
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "clock_freshness_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        hermetic.status.success(),
        "hermetic Clock.Now replay must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
}

#[test]
fn hermetic_clock_fixture_staleness_fails_closed() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("clock-stale");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/clock_freshness_witness.dag");

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "clock_freshness_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(record.status.success(), "record must capture Clock.Now");

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
        "clock_freshness_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        !hermetic.status.success(),
        "stale Clock.Now fixture must fail closed, not replay stale value"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&hermetic.stdout),
        String::from_utf8_lossy(&hermetic.stderr)
    );
    assert!(
        combined.contains("expired recorded fixture")
            || combined.contains("refusing to replay stale value"),
        "expected clock-path staleness diagnostic, got:\n{combined}"
    );
}

#[test]
fn env_get_record_then_hermetic_replay_holds() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("env-record-replay");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/env_freshness_witness.dag");

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "env_freshness_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(
        record.status.success(),
        "shell.Env.Get record capture must pass (wet); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "env_freshness_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        hermetic.status.success(),
        "hermetic shell.Env.Get replay must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
}

#[test]
fn hermetic_env_fixture_staleness_fails_closed() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("env-stale");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/env_freshness_witness.dag");

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "env_freshness_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(record.status.success(), "record must capture shell.Env.Get");

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
        "env_freshness_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        !hermetic.status.success(),
        "stale shell.Env.Get fixture must fail closed"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&hermetic.stdout),
        String::from_utf8_lossy(&hermetic.stderr)
    );
    assert!(
        combined.contains("expired recorded fixture")
            || combined.contains("refusing to replay stale value"),
        "expected env-path staleness diagnostic, got:\n{combined}"
    );
}

#[test]
fn env_hermetic_without_fixture_store_fails_closed() {
    let ws = workspace_root();
    let entry = ws.join("dsl/test/claim/env_freshness_witness.dag");
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "env_freshness_keystone_holds",
        "--hermetic",
    ]);
    assert!(
        !hermetic.status.success(),
        "shell.Env.Get in Hermetic without fixture store must fail closed"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&hermetic.stdout),
        String::from_utf8_lossy(&hermetic.stderr)
    );
    assert!(
        combined.contains("no mock_response") || combined.contains("refusing to fabricate"),
        "expected fail-closed diagnostic for env without fixtures, got:\n{combined}"
    );
}

#[test]
fn diagnostic_redfish_record_then_hermetic_replay_holds() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("diagnostic-redfish-record-replay");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/diagnostic_redfish_witness.dag");
    assert!(
        entry.is_file(),
        "witness dag must exist at {}",
        entry.display()
    );

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "diagnostic_redfish_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(
        record.status.success(),
        "redfish.Http.GetChassisSensors record capture must pass (wet); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "diagnostic_redfish_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        hermetic.status.success(),
        "hermetic redfish.Http.GetChassisSensors replay must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
}

#[test]
fn diagnostic_redfish_hermetic_without_fixture_store_fails_closed() {
    let ws = workspace_root();
    let entry = ws.join("dsl/test/claim/diagnostic_redfish_witness.dag");
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "diagnostic_redfish_keystone_holds",
        "--hermetic",
    ]);
    assert!(
        !hermetic.status.success(),
        "redfish.Http.GetChassisSensors in Hermetic without fixture store must fail closed"
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

#[test]
#[ignore = "wet-only: live jsonplaceholder record→replay — not hermetic CI floor"]
fn http_pilot_rest_record_then_hermetic_replay_holds() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("http-pilot-record-replay");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    let entry = ws.join("dsl/test/claim/http_pilot_rest_witness.dag");
    assert!(
        entry.is_file(),
        "witness dag must exist at {}",
        entry.display()
    );

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "http_pilot_rest_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    assert!(
        record.status.success(),
        "REST wet record capture must pass (live jsonplaceholder); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );
    assert!(
        store_dir.join("test__HttpPilot__GetPost").is_dir()
            || !fixture_files(&store_dir).is_empty(),
        "record must write REST fixture files under {:?}",
        store_dir
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "http_pilot_rest_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        hermetic.status.success(),
        "hermetic REST replay must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
}

#[test]
fn hermetic_http_pilot_fixture_staleness_fails_closed() {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("http-pilot-stale");
    fs::create_dir_all(&store_dir).expect("fixture dir");
    write_http_pilot_fixture(&store_dir, 0);
    let entry = ws.join("dsl/test/claim/http_pilot_rest_witness.dag");

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "http_pilot_rest_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().expect("store path"),
    ]);
    let _ = fs::remove_dir_all(&store_dir);
    assert!(
        !hermetic.status.success(),
        "stale REST fixture must fail closed"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&hermetic.stdout),
        String::from_utf8_lossy(&hermetic.stderr)
    );
    assert!(
        combined.contains("expired recorded fixture")
            || combined.contains("refusing to replay stale value"),
        "expected REST staleness diagnostic, got:\n{combined}"
    );
}

#[test]
fn hermetic_http_pilot_without_fixture_store_fails_closed() {
    let ws = workspace_root();
    let entry = ws.join("dsl/test/claim/http_pilot_rest_witness.dag");
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().expect("workspace"),
        "--source-root",
        ws.join("dsl").to_str().expect("dsl root"),
        "--entry",
        entry.to_str().expect("entry"),
        "--function",
        "http_pilot_rest_keystone_holds",
        "--hermetic",
    ]);
    assert!(
        !hermetic.status.success(),
        "REST pilot in Hermetic without fixture store must fail closed (no inline mock)"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&hermetic.stdout),
        String::from_utf8_lossy(&hermetic.stderr)
    );
    assert!(
        combined.contains("no mock_response") || combined.contains("refusing to fabricate"),
        "expected fail-closed diagnostic for REST without fixtures, got:\n{combined}"
    );
    assert!(
        !combined.contains("PASS http_pilot_rest_keystone_holds"),
        "hermetic without store must not pass — proves inline stub is gone, got:\n{combined}"
    );
}

fn write_http_pilot_fixture(store_dir: &Path, recorded_at: u64) {
    let op_dir = store_dir.join("test__HttpPilot__GetPost");
    fs::create_dir_all(&op_dir).expect("fixture op dir");
    let fixture = serde_json::json!({
        "operation": "test.HttpPilot.GetPost",
        "input_hash": "b63a4282295b68bd",
        "inputs": [{
            "name": "post_id",
            "value": { "__tag": "Int", "value": 1 }
        }],
        "response": {
            "__tag": "Record",
            "__type": "GetPost",
            "fields": {
                "post_id": { "__tag": "Int", "value": 1 },
                "title": {
                    "__tag": "Str",
                    "value": "sunt aut facere repellat provident occaecati excepturi optio reprehenderit"
                }
            }
        },
        "recorded_at": recorded_at
    });
    fs::write(
        op_dir.join("b63a4282295b68bd.json"),
        serde_json::to_vec_pretty(&fixture).expect("serialize"),
    )
    .expect("write fixture");
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
