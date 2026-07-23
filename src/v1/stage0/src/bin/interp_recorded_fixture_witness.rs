#![allow(clippy::disallowed_macros)]

//! Host-physics floor witness for the v1 interpreter's recorded-fixture
//! record -> hermetic-replay -> fail-closed lifecycle (DESIGN §5 determinism /
//! safety axis). Migrated from `src/v1/tests/src/interp_recorded_fixture_test.rs`
//! (24 `#[test]` fns) under the test-migration lane: a pure `.dag` `test fn`
//! cannot exercise record/replay host effects, so the executing coverage lives
//! in this witness binary and is driven by the floor through a shell transport.
//!
//! Each check drives the real `claim_batch` binary with `--record` / `--hermetic`
//! / `--fixture-store`, mutates fixtures on disk, and asserts the fail-closed
//! diagnostics actually fire — green by execution, red on a real regression.
//! SCAFFOLD — dissolves when the hermetic-fixture record/replay lifecycle is
//! itself modeled + executed in the v2 `.dag` floor (open thread: v2.std.determinism).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output};
use std::rc::Rc;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::recorded_fixture::{value_from_fixture_json, RecordedFixtureStore};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v1_compiler::v1_interpreter::{self, ExecutionMode, Value};

// ---- helpers ported from the deleted test module -------------------------------

macro_rules! ensure {
    ($cond:expr, $($arg:tt)*) => {
        if !($cond) {
            return Err(format!($($arg)*));
        }
    };
}

fn claim_batch_exe() -> Result<PathBuf, String> {
    let ws = workspace_root();
    let release = ws.join("target/release/claim_batch");
    if release.is_file() {
        return Ok(release);
    }
    let debug = ws.join("target/debug/claim_batch");
    if debug.is_file() {
        return Ok(debug);
    }
    // Build it on demand so the witness is self-sufficient when run outside the
    // transport's pre-build step.
    let status = Command::new("cargo")
        .current_dir(&ws)
        .args([
            "build",
            "-p",
            "v1-compiler",
            "--release",
            "--bin",
            "claim_batch",
        ])
        .status()
        .map_err(|e| format!("failed to spawn cargo build for claim_batch: {e}"))?;
    if !status.success() {
        return Err("cargo build -p v1-compiler --bin claim_batch failed".to_string());
    }
    if release.is_file() {
        Ok(release)
    } else {
        Err(format!(
            "claim_batch binary still missing at {}",
            release.display()
        ))
    }
}

/// Per-check scratch dir (fixture store + any rewritten witness entry), homed
/// INSIDE the workspace under `target/`: the module universe is workspace-anchored
/// (`build_module_path_index` refuses paths outside `workspace_root()`), and
/// `resolve_transitively` refuses entries the facts pool does not declare — so a
/// rewritten entry must live somewhere a `--source-root` can cover. `target/` is
/// realization output: sibling roots never descend into it (`is_cargo_target_output_dir`),
/// so the unique dir is visible ONLY to the invocation that passes it as a root.
fn fixture_store_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target/interp-recorded-scratch")
        .join(format!(
            "{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ))
}

fn unique_fs_witness_entry(ws: &Path, scratch: &Path) -> Result<PathBuf, String> {
    let src = fs::read_to_string(ws.join("dag/test/claim/filesystem_write_witness.dag"))
        .map_err(|e| format!("read filesystem witness dag: {e}"))?;
    let live = scratch.join("fs_witness");
    let absent = scratch.join("fs_witness_absent_should_not_exist_42");
    let rewritten = src
        // The scratch copy is a SECOND declaring file for the same content; the
        // in-tree template keeps its name (one module, one authority — the
        // module-path collision wall refuses a same-name duplicate), so the copy
        // declares itself as the scratch variant.
        .replace(
            "module test.claim.filesystem_write_witness\n",
            "module test.claim.filesystem_write_witness_scratch\n",
        )
        .replace(
            "/tmp/gunbc_fs_witness_absent_should_not_exist_42",
            absent.to_str().ok_or("utf8 scratch path")?,
        )
        .replace(
            "/tmp/gunbc_fs_write_witness",
            live.to_str().ok_or("utf8 scratch path")?,
        );
    let entry = scratch.join("filesystem_write_witness.dag");
    fs::write(&entry, rewritten)
        .map_err(|e| format!("write rewritten filesystem witness dag: {e}"))?;
    Ok(entry)
}

fn closure_scale_witness_entry(ws: &Path, scratch: &Path) -> Result<PathBuf, String> {
    let src =
        fs::read_to_string(ws.join("dag/test/claim/filesystem_write_closure_scale_witness.dag"))
            .map_err(|e| format!("read closure-scale witness dag: {e}"))?;
    let live = scratch.join("fs_closure_scale_witness.txt");
    let rewritten = src
        .replace(
            "module test.claim.filesystem_write_closure_scale_witness\n",
            "module test.claim.filesystem_write_closure_scale_witness_scratch\n",
        )
        .replace(
            "/tmp/gunbc_fs_closure_scale_witness.txt",
            live.to_str().ok_or("utf8 scratch path")?,
        );
    let entry = scratch.join("filesystem_write_closure_scale_witness.dag");
    fs::write(&entry, rewritten)
        .map_err(|e| format!("write rewritten closure-scale witness dag: {e}"))?;
    Ok(entry)
}

fn run_claim_batch(args: &[&str]) -> Result<Output, String> {
    let exe = claim_batch_exe()?;
    let mut cmd = Command::new(&exe);
    cmd.current_dir(workspace_root());
    for arg in args {
        cmd.arg(arg);
    }
    cmd.output().map_err(|e| format!("claim_batch spawn: {e}"))
}

fn combined_output(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn collect_json_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, out);
        } else if path.extension().map(|e| e == "json").unwrap_or(false) {
            out.push(path);
        }
    }
}

fn fixture_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_json_files(dir, &mut out);
    out
}

fn assert_resolved_no_hard_errors(result: &ResolvedPipelineResult) -> Result<(), String> {
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    ensure!(
        msgs.is_empty() && result.graph.is_some(),
        "expected resolved graph, got diagnostics {:?} (graph present: {})",
        msgs,
        result.graph.is_some()
    );
    Ok(())
}

fn single_source(content: &str) -> Rc<im::Vector<Rc<SourceFile>>> {
    // The witness modules below have no imports, so import-transitive resolution
    // is exactly the entry file itself.
    Rc::new(im::vector![Rc::new(SourceFile {
        path: "test.dag".to_string(),
        content: content.to_string(),
    })])
}

fn write_http_pilot_fixture(store_dir: &Path, recorded_at: u64) -> Result<(), String> {
    let op_dir = store_dir.join("test__HttpPilot__GetPost");
    fs::create_dir_all(&op_dir).map_err(|e| format!("fixture op dir: {e}"))?;
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
    .map_err(|e| format!("write fixture: {e}"))
}

// ---- ported checks -------------------------------------------------------------

fn filesystem_write_witness_record_then_hermetic_replay_holds() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fs-write-record-replay");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = unique_fs_witness_entry(&ws, &store_dir)?;

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "filesystem_write_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(
        record.status.success(),
        "record capture must pass (wet I/O); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );
    ensure!(
        store_dir.join("Filesystem__Write").is_dir() || !fixture_files(&store_dir).is_empty(),
        "record must write fixture files under {:?}",
        store_dir
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "filesystem_write_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        hermetic.status.success(),
        "hermetic replay must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    Ok(())
}

fn filesystem_write_closure_scale_record_then_hermetic_replay_holds() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fs-closure-scale-record-replay");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = closure_scale_witness_entry(&ws, &store_dir)?;

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "filesystem_write_closure_scale_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(
        record.status.success(),
        "closure-scale record capture must pass: the Filesystem service must register \
         even when std.resources' resource Filesystem is in the same import closure; stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "filesystem_write_closure_scale_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        hermetic.status.success(),
        "hermetic replay of the closure-scale witness must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    Ok(())
}

fn hermetic_fixture_staleness_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fs-stale");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = unique_fs_witness_entry(&ws, &store_dir)?;

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "witness_read_absent_fails_closed",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(record.status.success(), "record must capture absent-read");

    for path in fixture_files(&store_dir) {
        let bytes = fs::read(&path).map_err(|e| format!("read fixture: {e}"))?;
        let mut fixture: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("parse fixture: {e}"))?;
        if let Some(obj) = fixture.as_object_mut() {
            obj.insert("recorded_at".to_string(), serde_json::json!(0u64));
        }
        fs::write(
            &path,
            serde_json::to_vec_pretty(&fixture).expect("serialize"),
        )
        .map_err(|e| format!("write: {e}"))?;
    }

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "witness_read_absent_fails_closed",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        !hermetic.status.success(),
        "stale fixture must fail closed, not replay stale value"
    );
    let combined = combined_output(&hermetic);
    ensure!(
        combined.contains("expired recorded fixture")
            || combined.contains("refusing to replay stale value"),
        "expected staleness diagnostic, got:\n{combined}"
    );
    Ok(())
}

fn record_response_drift_for_same_input_hash_fails_closed() -> Result<(), String> {
    let src = r#"module test.fixture_roundtrip

fn witness() -> Bool {
  let r = { success: true, bytes_written: 42, path: "/tmp/x", error: "" }
  r.success
}
"#;
    let resolved = compile_to_resolved(single_source(src));
    assert_resolved_no_hard_errors(&resolved)?;
    let graph = resolved.graph.as_ref().ok_or("graph")?;
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    let val_a = v1_interpreter::run_in_context(&ctx, "witness", false)
        .map_err(|e| format!("witness runs: {e:?}"))?;
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
            v1_interpreter::fixture_now_secs(&ctx).map_err(|e| format!("clock: {e}"))?,
        )
        .map_err(|e| format!("first record: {e}"))?;

    let val_b_src = r#"module test.fixture_roundtrip

fn witness() -> Bool {
  let r = { success: false, bytes_written: 0, path: "/tmp/y", error: "changed" }
  r.success
}
"#;
    let resolved_b = compile_to_resolved(single_source(val_b_src));
    assert_resolved_no_hard_errors(&resolved_b)?;
    let graph_b = resolved_b.graph.as_ref().ok_or("graph")?;
    let ctx_b = v1_interpreter::InterpContext::new(
        graph_b,
        resolved_b.source_indices.clone(),
        ExecutionMode::Wet,
    );
    let val_b = v1_interpreter::run_in_context(&ctx_b, "witness", false)
        .map_err(|e| format!("witness b: {e:?}"))?;

    let err = store.record(
        "Filesystem.Write",
        "0123456789abcdef",
        &empty_inputs,
        &val_b,
        &ctx_b,
        v1_interpreter::fixture_now_secs(&ctx_b).map_err(|e| format!("clock: {e}"))?,
    );
    let _ = fs::remove_dir_all(&store_dir);
    match err {
        Ok(()) => Err("same input_hash with different response must fail closed".to_string()),
        Err(e) => {
            ensure!(
                e.to_string().contains("response drift"),
                "expected response drift diagnostic, got: {e}"
            );
            Ok(())
        }
    }
}

fn hermetic_without_fixture_store_still_uses_mock_response_for_rest() -> Result<(), String> {
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
    let resolved = compile_to_resolved(single_source(src));
    assert_resolved_no_hard_errors(&resolved)?;
    let graph = resolved.graph.as_ref().ok_or("graph")?;
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Hermetic,
    );
    match v1_interpreter::run_in_context(&ctx, "witness", false) {
        Ok(Value::Str(s)) => {
            ensure!(s == "dry-run-mock", "expected dry-run-mock, got {s:?}");
            Ok(())
        }
        other => Err(format!("expected mock String, got {other:?}")),
    }
}

fn recorded_fixture_store_roundtrip_value() -> Result<(), String> {
    let src = r#"module test.fixture_roundtrip

fn witness() -> Bool {
  let r = { success: true, bytes_written: 42, path: "/tmp/x", error: "" }
  r.success && (r.bytes_written == 42)
}
"#;
    let resolved = compile_to_resolved(single_source(src));
    assert_resolved_no_hard_errors(&resolved)?;
    let graph = resolved.graph.as_ref().ok_or("graph")?;
    let ctx = v1_interpreter::InterpContext::new(
        graph,
        resolved.source_indices.clone(),
        ExecutionMode::Wet,
    );
    let val = v1_interpreter::run_in_context(&ctx, "witness", false)
        .map_err(|e| format!("witness runs: {e:?}"))?;
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
            v1_interpreter::fixture_now_secs(&ctx).map_err(|e| format!("clock: {e}"))?,
        )
        .map_err(|e| format!("record: {e}"))?;
    let now = v1_interpreter::fixture_now_secs(&ctx).map_err(|e| format!("clock: {e}"))?;
    let fixture = store
        .lookup("Filesystem.Write", "0123456789abcdef", &empty_inputs, now)
        .map_err(|e| format!("lookup: {e}"))?;
    let back = value_from_fixture_json(&fixture.response, &ctx)
        .map_err(|e| format!("deserialize: {e}"))?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(val == back, "fixture JSON round-trip must preserve Value");
    Ok(())
}

fn hermetic_replay_rejects_corrupted_fixture_response() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("corrupt-response");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = unique_fs_witness_entry(&ws, &store_dir)?;

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "witness_write_then_read_roundtrip",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(record.status.success(), "record must capture write/read");

    for path in fixture_files(&store_dir) {
        let bytes = fs::read(&path).map_err(|e| format!("read fixture: {e}"))?;
        let mut fixture: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("parse fixture: {e}"))?;
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
        .map_err(|e| format!("write tampered fixture: {e}"))?;
    }

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "witness_write_then_read_roundtrip",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        !hermetic.status.success(),
        "corrupted fixture response must fail closed; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    let combined = combined_output(&hermetic);
    ensure!(
        combined.contains("FAIL") || combined.contains("deserialization"),
        "expected witness failure on corrupted fixture, got:\n{combined}"
    );
    Ok(())
}

fn hermetic_replay_uses_fixture_not_live_fs_after_mutation() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fixture-not-live");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = unique_fs_witness_entry(&ws, &store_dir)?;
    let target = store_dir.join("fs_witness.txt");

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "witness_write_then_read_roundtrip",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(record.status.success(), "record must capture");

    fs::write(&target, b"MUTATED-LIVE-FS-CONTENT").map_err(|e| format!("mutate live file: {e}"))?;

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        store_dir.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "witness_write_then_read_roundtrip",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        hermetic.status.success(),
        "hermetic must replay recorded bytes, not live FS; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    Ok(())
}

fn filesystem_hermetic_without_fixture_store_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let scratch = fixture_store_dir("fs-no-store-fails-closed");
    fs::create_dir_all(&scratch).map_err(|e| format!("scratch dir: {e}"))?;
    let entry = unique_fs_witness_entry(&ws, &scratch)?;
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--source-root",
        scratch.to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "witness_read_absent_fails_closed",
        "--hermetic",
    ])?;
    ensure!(
        !hermetic.status.success(),
        "Filesystem op in Hermetic without fixture store must fail closed (no silent Unit)"
    );
    let combined = combined_output(&hermetic);
    let _ = fs::remove_dir_all(&scratch);
    ensure!(
        combined.contains("no mock_response") || combined.contains("refusing to fabricate"),
        "expected fail-closed mock_response diagnostic, got:\n{combined}"
    );
    Ok(())
}

fn filesystem_read_hermetic_without_fixture_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    // Contract after the checkout-input carve-out (§3 single-authority split): a hermetic
    // `filesystem_read` of a CHECKOUT path is a real input read (the commit is the input —
    // covered green by filesystem_read_record_then_hermetic_replay_holds, which round-trips
    // dag/std/filesystem.dag). A read of HOST STATE the carve-out does NOT confirm as a
    // committed input — here a `target/` build-artifact path — must still fail closed with no
    // fixture and no mock_response: no silent disk read of non-deterministic host state.
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        ws.join("dag/test/claim/filesystem_read_hermetic_witness.dag")
            .to_str()
            .unwrap(),
        "--function",
        "witness_read_host_state_fails_closed",
        "--hermetic",
    ])?;
    ensure!(
        !hermetic.status.success(),
        "filesystem_read of host state in Hermetic without fixture store must fail closed (no silent disk read)"
    );
    let combined = combined_output(&hermetic);
    ensure!(
        combined.contains("no mock_response")
            || combined.contains("refusing to fabricate")
            || combined.contains("refusing direct disk read"),
        "expected fail-closed hermetic diagnostic, got:\n{combined}"
    );
    Ok(())
}

fn filesystem_read_record_then_hermetic_replay_holds() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("fs-read-builtin-record-replay");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let dag_root = ws.join("dag");
    let entry = ws.join("dag/test/claim/filesystem_read_hermetic_witness.dag");
    let store_path = store_dir.to_str().unwrap().to_string();

    let common = |mode_flag: &str| -> Result<Output, String> {
        run_claim_batch(&[
            "--source-root",
            ws.to_str().unwrap(),
            "--source-root",
            dag_root.to_str().unwrap(),
            "--entry",
            entry.to_str().unwrap(),
            "--function",
            "witness_read_via_builtin_roundtrip",
            "--fixture-store",
            store_path.as_str(),
            mode_flag,
        ])
    };

    let record = common("--record")?;
    ensure!(
        record.status.success(),
        "record capture must pass (wet I/O via Filesystem.Read gate); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );
    ensure!(
        store_dir.join("Filesystem__Read").is_dir() || !fixture_files(&store_dir).is_empty(),
        "record must write fixture files under {:?}",
        store_dir
    );

    let hermetic = common("--hermetic")?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        hermetic.status.success(),
        "hermetic replay of filesystem_read must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    Ok(())
}

fn m4_governed_service_published_realizes_unpublished_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let common = |func: &str| -> Result<Output, String> {
        run_claim_batch(&[
            "--source-root",
            ws.to_str().unwrap(),
            "--source-root",
            ws.join("dag").to_str().unwrap(),
            "--entry",
            ws.join("dag/test/claim/m4_governed_service_witness.dag")
                .to_str()
                .unwrap(),
            "--function",
            func,
            "--hermetic",
        ])
    };

    let green = common("witness_published_realizes")?;
    ensure!(
        green.status.success(),
        "published op must realize hermetically on the claim_batch path; stderr={}",
        String::from_utf8_lossy(&green.stderr)
    );

    let red = common("witness_unpublished_fails_closed")?;
    let combined = combined_output(&red);
    ensure!(
        !red.status.success(),
        "unpublished op on a corpus-governed service must fail closed (non-zero exit); output:\n{combined}"
    );
    ensure!(
        combined.contains("not a published mock case"),
        "expected published-corpus refusal diagnostic, got:\n{combined}"
    );
    Ok(())
}

fn gcp_oauth_access_token_materializer_holds() -> Result<(), String> {
    let ws = workspace_root();
    let entry = ws.join("dag/test/claim/gcp_oauth_access_token_witness.dag");
    let store = ws.join("dag/test/fixture/gcp_oauth_access_token_store");
    let common = |func: &str| -> Result<Output, String> {
        run_claim_batch(&[
            "--source-root",
            ws.to_str().unwrap(),
            "--source-root",
            ws.join("dag").to_str().unwrap(),
            "--entry",
            entry.to_str().unwrap(),
            "--function",
            func,
            "--hermetic",
            "--fixture-store",
            store.to_str().unwrap(),
        ])
    };

    let green = common("gcp_oauth_access_token_materializer_green_holds")?;
    ensure!(
        green.status.success(),
        "materializer green witness must pass hermetically; stderr={}",
        String::from_utf8_lossy(&green.stderr)
    );

    let red = common("gcp_oauth_access_token_dispatch_discriminator_is_red_holds")?;
    ensure!(
        red.status.success(),
        "dispatch discriminator witness must pass (proves swapped arms are detectable); stderr={}",
        String::from_utf8_lossy(&red.stderr)
    );
    Ok(())
}

fn m4_universal_corpus_published_realizes_unpublished_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let common = |func: &str| -> Result<Output, String> {
        run_claim_batch(&[
            "--source-root",
            ws.to_str().unwrap(),
            "--source-root",
            ws.join("dag").to_str().unwrap(),
            "--entry",
            ws.join("dag/test/claim/m4_universal_corpus_witness.dag")
                .to_str()
                .unwrap(),
            "--function",
            func,
            "--hermetic",
        ])
    };

    let green = common("witness_universal_published_realizes")?;
    ensure!(
        green.status.success(),
        "published op must realize with whole-tree corpus governance; stderr={}",
        String::from_utf8_lossy(&green.stderr)
    );

    let red = common("witness_universal_unpublished_fails_closed")?;
    let combined = combined_output(&red);
    ensure!(
        !red.status.success(),
        "unpublished op must fail closed when corpus is outside entry closure; output:\n{combined}"
    );
    ensure!(
        combined.contains("not a published mock case"),
        "expected published-corpus refusal diagnostic, got:\n{combined}"
    );
    Ok(())
}

fn clock_now_record_then_hermetic_replay_holds() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("clock-record-replay");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = ws.join("dag/test/claim/clock_freshness_witness.dag");
    ensure!(
        entry.is_file(),
        "witness dag must exist at {}",
        entry.display()
    );

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "clock_freshness_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(
        record.status.success(),
        "Clock.Now record capture must pass (wet); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );
    ensure!(
        store_dir.join("Clock__Now").is_dir() || !fixture_files(&store_dir).is_empty(),
        "record must write Clock.Now fixture files under {:?}",
        store_dir
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "clock_freshness_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        hermetic.status.success(),
        "hermetic Clock.Now replay must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    Ok(())
}

fn hermetic_clock_fixture_staleness_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("clock-stale");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = ws.join("dag/test/claim/clock_freshness_witness.dag");

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "clock_freshness_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(record.status.success(), "record must capture Clock.Now");

    for path in fixture_files(&store_dir) {
        let bytes = fs::read(&path).map_err(|e| format!("read fixture: {e}"))?;
        let mut fixture: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("parse fixture: {e}"))?;
        if let Some(obj) = fixture.as_object_mut() {
            obj.insert("recorded_at".to_string(), serde_json::json!(0u64));
        }
        fs::write(
            &path,
            serde_json::to_vec_pretty(&fixture).expect("serialize"),
        )
        .map_err(|e| format!("write: {e}"))?;
    }

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "clock_freshness_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        !hermetic.status.success(),
        "stale Clock.Now fixture must fail closed, not replay stale value"
    );
    let combined = combined_output(&hermetic);
    ensure!(
        combined.contains("expired recorded fixture")
            || combined.contains("refusing to replay stale value"),
        "expected clock-path staleness diagnostic, got:\n{combined}"
    );
    Ok(())
}

fn env_get_record_then_hermetic_replay_holds() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("env-record-replay");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = ws.join("dag/test/claim/env_freshness_witness.dag");

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "env_freshness_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(
        record.status.success(),
        "shell.Env.Get record capture must pass (wet); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "env_freshness_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        hermetic.status.success(),
        "hermetic shell.Env.Get replay must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    Ok(())
}

fn hermetic_env_fixture_staleness_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("env-stale");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = ws.join("dag/test/claim/env_freshness_witness.dag");

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "env_freshness_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(record.status.success(), "record must capture shell.Env.Get");

    for path in fixture_files(&store_dir) {
        let bytes = fs::read(&path).map_err(|e| format!("read fixture: {e}"))?;
        let mut fixture: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("parse fixture: {e}"))?;
        if let Some(obj) = fixture.as_object_mut() {
            obj.insert("recorded_at".to_string(), serde_json::json!(0u64));
        }
        fs::write(
            &path,
            serde_json::to_vec_pretty(&fixture).expect("serialize"),
        )
        .map_err(|e| format!("write: {e}"))?;
    }

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "env_freshness_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        !hermetic.status.success(),
        "stale shell.Env.Get fixture must fail closed"
    );
    let combined = combined_output(&hermetic);
    ensure!(
        combined.contains("expired recorded fixture")
            || combined.contains("refusing to replay stale value"),
        "expected env-path staleness diagnostic, got:\n{combined}"
    );
    Ok(())
}

fn env_hermetic_without_fixture_store_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let entry = ws.join("dag/test/claim/env_freshness_witness.dag");
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "env_freshness_keystone_holds",
        "--hermetic",
    ])?;
    ensure!(
        !hermetic.status.success(),
        "shell.Env.Get in Hermetic without fixture store must fail closed"
    );
    let combined = combined_output(&hermetic);
    ensure!(
        combined.contains("no mock_response") || combined.contains("refusing to fabricate"),
        "expected fail-closed diagnostic for env without fixtures, got:\n{combined}"
    );
    Ok(())
}

fn diagnostic_redfish_record_then_hermetic_replay_holds() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("diagnostic-redfish-record-replay");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    let entry = ws.join("dag/test/claim/diagnostic_redfish_witness.dag");
    ensure!(
        entry.is_file(),
        "witness dag must exist at {}",
        entry.display()
    );

    let record = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "diagnostic_redfish_keystone_holds",
        "--record",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    ensure!(
        record.status.success(),
        "redfish.Http.GetChassisSensors record capture must pass (wet); stderr={}",
        String::from_utf8_lossy(&record.stderr)
    );

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "diagnostic_redfish_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        hermetic.status.success(),
        "hermetic redfish.Http.GetChassisSensors replay must pass from recorded fixtures; stderr={}",
        String::from_utf8_lossy(&hermetic.stderr)
    );
    Ok(())
}

fn diagnostic_redfish_hermetic_without_fixture_store_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let entry = ws.join("dag/test/claim/diagnostic_redfish_witness.dag");
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "diagnostic_redfish_keystone_holds",
        "--hermetic",
    ])?;
    ensure!(
        !hermetic.status.success(),
        "redfish.Http.GetChassisSensors in Hermetic without fixture store must fail closed"
    );
    let combined = combined_output(&hermetic);
    ensure!(
        combined.contains("no mock_response") || combined.contains("refusing to fabricate"),
        "expected fail-closed mock_response diagnostic, got:\n{combined}"
    );
    Ok(())
}

fn hermetic_http_pilot_fixture_staleness_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let store_dir = fixture_store_dir("http-pilot-stale");
    fs::create_dir_all(&store_dir).map_err(|e| format!("fixture dir: {e}"))?;
    write_http_pilot_fixture(&store_dir, 0)?;
    let entry = ws.join("dag/test/claim/http_pilot_rest_witness.dag");

    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "http_pilot_rest_keystone_holds",
        "--hermetic",
        "--fixture-store",
        store_dir.to_str().unwrap(),
    ])?;
    let _ = fs::remove_dir_all(&store_dir);
    ensure!(
        !hermetic.status.success(),
        "stale REST fixture must fail closed"
    );
    let combined = combined_output(&hermetic);
    ensure!(
        combined.contains("expired recorded fixture")
            || combined.contains("refusing to replay stale value"),
        "expected REST staleness diagnostic, got:\n{combined}"
    );
    Ok(())
}

fn hermetic_http_pilot_without_fixture_store_fails_closed() -> Result<(), String> {
    let ws = workspace_root();
    let entry = ws.join("dag/test/claim/http_pilot_rest_witness.dag");
    let hermetic = run_claim_batch(&[
        "--source-root",
        ws.to_str().unwrap(),
        "--source-root",
        ws.join("dag").to_str().unwrap(),
        "--entry",
        entry.to_str().unwrap(),
        "--function",
        "http_pilot_rest_keystone_holds",
        "--hermetic",
    ])?;
    ensure!(
        !hermetic.status.success(),
        "REST pilot in Hermetic without fixture store must fail closed (no inline mock)"
    );
    let combined = combined_output(&hermetic);
    ensure!(
        combined.contains("no mock_response") || combined.contains("refusing to fabricate"),
        "expected fail-closed diagnostic for REST without fixtures, got:\n{combined}"
    );
    ensure!(
        !combined.contains("PASS http_pilot_rest_keystone_holds"),
        "hermetic without store must not pass — proves inline stub is gone, got:\n{combined}"
    );
    Ok(())
}

// ---- runner --------------------------------------------------------------------

fn main() -> ExitCode {
    // Note: `http_pilot_rest_record_then_hermetic_replay_holds` is intentionally
    // omitted — it was `#[ignore]`d (wet-only live jsonplaceholder record→replay),
    // not part of the hermetic CI floor. All other 23 checks run.
    type Check = (&'static str, fn() -> Result<(), String>);
    let checks: &[Check] = &[
        (
            "filesystem_write_witness_record_then_hermetic_replay_holds",
            filesystem_write_witness_record_then_hermetic_replay_holds,
        ),
        (
            "filesystem_write_closure_scale_record_then_hermetic_replay_holds",
            filesystem_write_closure_scale_record_then_hermetic_replay_holds,
        ),
        (
            "hermetic_fixture_staleness_fails_closed",
            hermetic_fixture_staleness_fails_closed,
        ),
        (
            "record_response_drift_for_same_input_hash_fails_closed",
            record_response_drift_for_same_input_hash_fails_closed,
        ),
        (
            "hermetic_without_fixture_store_still_uses_mock_response_for_rest",
            hermetic_without_fixture_store_still_uses_mock_response_for_rest,
        ),
        (
            "recorded_fixture_store_roundtrip_value",
            recorded_fixture_store_roundtrip_value,
        ),
        (
            "hermetic_replay_rejects_corrupted_fixture_response",
            hermetic_replay_rejects_corrupted_fixture_response,
        ),
        (
            "hermetic_replay_uses_fixture_not_live_fs_after_mutation",
            hermetic_replay_uses_fixture_not_live_fs_after_mutation,
        ),
        (
            "filesystem_hermetic_without_fixture_store_fails_closed",
            filesystem_hermetic_without_fixture_store_fails_closed,
        ),
        (
            "filesystem_read_hermetic_without_fixture_fails_closed",
            filesystem_read_hermetic_without_fixture_fails_closed,
        ),
        (
            "filesystem_read_record_then_hermetic_replay_holds",
            filesystem_read_record_then_hermetic_replay_holds,
        ),
        (
            "m4_governed_service_published_realizes_unpublished_fails_closed",
            m4_governed_service_published_realizes_unpublished_fails_closed,
        ),
        (
            "gcp_oauth_access_token_materializer_holds",
            gcp_oauth_access_token_materializer_holds,
        ),
        (
            "m4_universal_corpus_published_realizes_unpublished_fails_closed",
            m4_universal_corpus_published_realizes_unpublished_fails_closed,
        ),
        (
            "clock_now_record_then_hermetic_replay_holds",
            clock_now_record_then_hermetic_replay_holds,
        ),
        (
            "hermetic_clock_fixture_staleness_fails_closed",
            hermetic_clock_fixture_staleness_fails_closed,
        ),
        (
            "env_get_record_then_hermetic_replay_holds",
            env_get_record_then_hermetic_replay_holds,
        ),
        (
            "hermetic_env_fixture_staleness_fails_closed",
            hermetic_env_fixture_staleness_fails_closed,
        ),
        (
            "env_hermetic_without_fixture_store_fails_closed",
            env_hermetic_without_fixture_store_fails_closed,
        ),
        (
            "diagnostic_redfish_record_then_hermetic_replay_holds",
            diagnostic_redfish_record_then_hermetic_replay_holds,
        ),
        (
            "diagnostic_redfish_hermetic_without_fixture_store_fails_closed",
            diagnostic_redfish_hermetic_without_fixture_store_fails_closed,
        ),
        (
            "hermetic_http_pilot_fixture_staleness_fails_closed",
            hermetic_http_pilot_fixture_staleness_fails_closed,
        ),
        (
            "hermetic_http_pilot_without_fixture_store_fails_closed",
            hermetic_http_pilot_without_fixture_store_fails_closed,
        ),
    ];

    let mut failures = 0usize;
    for (name, check) in checks {
        match check() {
            Ok(()) => println!("PASS {name}"),
            Err(msg) => {
                failures += 1;
                eprintln!("FAIL {name}: {msg}");
            }
        }
    }

    if failures == 0 {
        println!(
            "interp_recorded_fixture_witness: all {} checks passed",
            checks.len()
        );
        ExitCode::SUCCESS
    } else {
        eprintln!("interp_recorded_fixture_witness: {failures} check(s) failed");
        ExitCode::from(1)
    }
}
