//! N_v2 substrate measurement — v2 own emitter (`emit_for_target` via
//! `emit_compiler_import_closure_from_ingest`) on the scoped 00_compile compiler closure,
//! executed through the v1 interpreter with host manifest overlay.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use v1_compiler::cli_run::{
    discover_floor_corpus_rows, discover_source_root_reads_for_entry,
    emit_source_root_ingest_manifest, make_eval_context, parse_source_root_entry_admission,
    resolve_entry_graph, run_claim, ClaimOutcome,
};
use v1_compiler::v1_interpreter::{self, ExecutionMode, InterpContext, Value};

use crate::helpers::workspace_root;

fn cargo_binary() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

const COMPILER_ENTRY: &str = "src/v2/compiler/00_compile.dag";
const NV2_GATE_ENTRY: &str = "src/v2/compiler/self_host/compiler_closure_emit_from_ingest_gate.dag";
const EMIT_SOURCE_FN: &str = "compiler_closure_v2_emit_source_for_cargo_check";
const ACCEPT_FN: &str = "compiler_closure_v2_emit_from_scoped_ingest_accepts";

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("gunbc-{prefix}-{}-{}", std::process::id(), nanos))
}

fn decode_freemonoid_string(val: &Value, ctx: &InterpContext) -> String {
    fn codepoint(v: &Value) -> char {
        match v {
            Value::Int(n) => char::from_u32(*n as u32)
                .unwrap_or_else(|| panic!("codepoint {n} is not a valid char")),
            other => panic!("expected Int codepoint in String FreeMonoid, got {other:?}"),
        }
    }
    match val {
        Value::Str(s) => s.clone(),
        Value::List(items) => items.iter().map(codepoint).collect(),
        Value::Variant { .. } => {
            let mut out = String::new();
            let mut cur = val.clone();
            loop {
                match cur {
                    Value::Variant {
                        variant_name,
                        fields,
                        ..
                    } => {
                        if ctx.sym_eq(variant_name, "Empty") {
                            break;
                        }
                        if ctx.sym_eq(variant_name, "Cons") {
                            let head = ctx.field(&fields, "head").expect("Cons.head");
                            out.push(codepoint(head));
                            cur = ctx.field(&fields, "tail").expect("Cons.tail").clone();
                            continue;
                        }
                        panic!(
                            "unexpected FreeMonoid variant {}",
                            ctx.resolve(variant_name)
                        );
                    }
                    Value::Str(s) => {
                        out.push_str(&s);
                        break;
                    }
                    other => panic!("unexpected tail value {other:?}"),
                }
            }
            out
        }
        other => panic!("not a String FreeMonoid: {other:?}"),
    }
}

fn write_nv2_manifest(manifest_path: &Path) {
    let ws = workspace_root();
    let v2_root = ws.join("src/v2");
    let entry = ws.join(COMPILER_ENTRY);
    let roots = vec![v2_root.to_string_lossy().to_string()];
    let records = discover_source_root_reads_for_entry(
        &roots,
        entry.to_str().expect("entry utf8"),
        &["host_source_root_ingest_manifest.dag".to_string()],
    )
    .expect("discover scoped compiler closure reads");
    assert!(
        !records.is_empty(),
        "expected non-empty scoped compiler closure ingest"
    );
    eprintln!(
        "N_v2 manifest: scoped 00_compile closure ingest_read_count={}",
        records.len()
    );
    let entry_source =
        fs::read_to_string(ws.join(COMPILER_ENTRY)).expect("read compiler entry source");
    let admission =
        parse_source_root_entry_admission(&entry_source).expect("parse entry admission");
    emit_source_root_ingest_manifest(manifest_path, &records, Some(&admission))
        .expect("emit scoped ingest manifest");
}

fn nv2_eval_context(manifest_dir: &Path) -> Result<InterpContext, String> {
    let ws = workspace_root();
    let entry = ws.join(NV2_GATE_ENTRY);
    let roots = vec![
        ws.join("src/v2").to_string_lossy().to_string(),
        manifest_dir.to_string_lossy().to_string(),
    ];
    let (graph, source_indices) = resolve_entry_graph(&roots, entry.to_str().expect("entry utf8"))?;
    Ok(make_eval_context(
        &graph,
        source_indices,
        ExecutionMode::Wet,
    ))
}

fn nv2_blocked_fail(detail: String) -> ! {
    panic!("N_v2 BLOCKED_PENDING_COMPILED_BINARY — {detail}");
}

fn cargo_check_single_file_crate(source: &str, out_dir: &Path) -> (bool, usize, String) {
    fs::create_dir_all(out_dir.join("src")).expect("create src dir");
    fs::write(out_dir.join("src/lib.rs"), source).expect("write emitted source");
    fs::write(
        out_dir.join("Cargo.toml"),
        "[package]\nname = \"nv2_emit_receipt\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    let check = Command::new(cargo_binary())
        .arg("check")
        .arg("--manifest-path")
        .arg(out_dir.join("Cargo.toml"))
        .output()
        .expect("cargo check");
    let stdout = String::from_utf8_lossy(&check.stdout);
    let stderr = String::from_utf8_lossy(&check.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let error_count = combined.matches("error[").count() + combined.matches("error:").count();
    (check.status.success(), error_count, combined)
}

#[test]
#[ignore] // Boundary: N_v2 — fail-closed when run with --ignored; panics BLOCKED until substrate unblocked.
fn nv2_scoped_compiler_closure_substrate_cargo_check_error_count() {
    let temp = unique_temp_dir("nv2-substrate");
    fs::create_dir_all(&temp).expect("temp dir");
    let manifest_path = temp.join("v2-compiler-closure-ingest-manifest.dag");
    write_nv2_manifest(&manifest_path);
    let module_count = fs::read_to_string(&manifest_path)
        .expect("read manifest")
        .matches("file_path:")
        .count();

    let manifest_dir = manifest_path.parent().expect("manifest parent");
    let ctx = nv2_eval_context(manifest_dir).unwrap_or_else(|msg| {
        nv2_blocked_fail(format!("resolve failed ({module_count} modules): {msg}"));
    });

    match run_claim(&ctx, ACCEPT_FN) {
        ClaimOutcome::Pass => {}
        ClaimOutcome::Fail => {
            nv2_blocked_fail(format!(
                "{ACCEPT_FN} returned false ({module_count} modules)"
            ));
        }
        ClaimOutcome::NotBool { got } => {
            nv2_blocked_fail(format!("{ACCEPT_FN} returned non-Bool ({got})"));
        }
        ClaimOutcome::RuntimeError { message } => {
            nv2_blocked_fail(format!("{ACCEPT_FN} runtime error: {message}"));
        }
    }

    let value = v1_interpreter::run_in_context(&ctx, EMIT_SOURCE_FN, true).unwrap_or_else(|e| {
        nv2_blocked_fail(format!("run {EMIT_SOURCE_FN} failed: {e:?}"));
    });
    let source = decode_freemonoid_string(&value, &ctx);
    assert!(
        !source.is_empty(),
        "N_v2 substrate emit returned empty TargetSource"
    );

    let check_dir = temp.join("emit-crate");
    let (success, error_count, combined) = cargo_check_single_file_crate(&source, &check_dir);
    eprintln!(
        "N_v2 (v2 emit_for_target substrate): emit_for_target accepted; cargo check success={success} error_count={error_count} source_bytes={}",
        source.len()
    );
    eprintln!(
        "N_v2 headline: v2 emit_for_target on scoped 00_compile closure ({module_count} modules) → {error_count} cargo-check errors on emitted TargetSource"
    );
    if !success {
        eprintln!(
            "--- N_v2 cargo check output (tail) ---\n{}",
            tail_lines(&combined, 40)
        );
    }

    let _ = fs::remove_dir_all(&temp);
    assert!(
        error_count > 0 || success,
        "cargo check produced no diagnostics and did not succeed"
    );
}

// Manual RED->GREEN harness for the #5146-class resolve_expr_types O(2^depth)
// fix (bind-once). Generates the host_source_root_ingest manifest at increasing
// record counts N and times the gate resolve. PRE-FIX: 2^N — n=20 hung >800s.
// POST-FIX (bind-once in 04_resolve.dag + v1_compiler_infer_resolve.rs): linear,
// dominated by bounded front-end parse — n=20 ~87s, all ok=true. Not a CI gate
// (wall-clock); the structural regression guard is
// resolve_expr_types_has_no_redundant_child_retraversal (always-on).
//   NV2_SCALE_NS=2,5,10,20 cargo test -p v1-compiler-tests \
//     v2_compiler_closure_nv2_substrate_test::nv2_manifest_resolve_scaling_probe -- --ignored --nocapture
#[test]
#[ignore]
fn nv2_manifest_resolve_scaling_probe() {
    use std::time::Instant;
    let ws = workspace_root();
    let v2_root = ws.join("src/v2");
    let entry = ws.join(COMPILER_ENTRY);
    let roots = vec![v2_root.to_string_lossy().to_string()];
    let records = discover_source_root_reads_for_entry(
        &roots,
        entry.to_str().expect("entry utf8"),
        &["host_source_root_ingest_manifest.dag".to_string()],
    )
    .expect("discover");
    let entry_source = fs::read_to_string(ws.join(COMPILER_ENTRY)).expect("read entry");
    let admission =
        parse_source_root_entry_admission(&entry_source).expect("parse entry admission");
    let gate_entry = ws.join(NV2_GATE_ENTRY);
    let ns: Vec<usize> = std::env::var("NV2_SCALE_NS")
        .unwrap_or_else(|_| "2,5,10,20".to_string())
        .split(',')
        .map(|s| s.trim().parse().expect("usize"))
        .collect();
    for n in ns {
        let n = n.min(records.len());
        let temp = unique_temp_dir(&format!("nv2-scale-{n}"));
        fs::create_dir_all(&temp).expect("temp");
        let manifest_path = temp.join("v2-compiler-closure-ingest-manifest.dag");
        emit_source_root_ingest_manifest(&manifest_path, &records[..n], Some(&admission))
            .expect("emit");
        let bytes = fs::metadata(&manifest_path).map(|m| m.len()).unwrap_or(0);
        let manifest_dir = manifest_path.parent().expect("parent");
        let scale_roots = vec![
            v2_root.to_string_lossy().to_string(),
            manifest_dir.to_string_lossy().to_string(),
        ];
        let start = Instant::now();
        let result = resolve_entry_graph(&scale_roots, gate_entry.to_str().expect("entry utf8"));
        eprintln!(
            "SCALE n={n} bytes={bytes} resolve={:?} ok={}",
            start.elapsed(),
            result.is_ok()
        );
        let _ = fs::remove_dir_all(&temp);
    }
}

fn tail_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines
        .into_iter()
        .skip(text.lines().count().saturating_sub(n))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn source_root_ingest_symbol_leading_digit_gets_sr_prefix() {
    assert_eq!(
        v1_compiler::cli_run::source_root_ingest_artifact_id_for_path(
            "src/v2/compiler/00_compile.dag"
        ),
        "^sr_00_compile"
    );
}

#[test]
fn floor_discovery_enrollment_has_no_test_fn_hygiene_violations() {
    let ws = workspace_root();
    let roots = vec![
        ws.join("src/v2").to_string_lossy().to_string(),
        ws.join("dsl").to_string_lossy().to_string(),
    ];
    let scan_dirs = vec![
        ws.join("dsl/test/claim").to_string_lossy().to_string(),
        ws.join("src/v2/compiler/manual")
            .to_string_lossy()
            .to_string(),
    ];
    discover_floor_corpus_rows(&roots, &scan_dirs)
        .expect("floor discovery must not find test fn outside *_test.dag");
}

#[test]
fn manifest_entry_admission_qualified_name_is_well_formed() {
    let ws = workspace_root();
    let temp = unique_temp_dir("manifest-qn");
    fs::create_dir_all(&temp).expect("temp dir");
    let manifest_path = temp.join("manifest.dag");
    let entry_source = fs::read_to_string(ws.join(COMPILER_ENTRY)).expect("read entry");
    let admission = parse_source_root_entry_admission(&entry_source).expect("parse admission");
    let records = discover_source_root_reads_for_entry(
        &[ws.join("src/v2").to_string_lossy().to_string()],
        ws.join(COMPILER_ENTRY).to_str().expect("entry utf8"),
        &["host_source_root_ingest_manifest.dag".to_string()],
    )
    .expect("discover reads");
    emit_source_root_ingest_manifest(&manifest_path, &records[..1], Some(&admission))
        .expect("emit manifest");
    let manifest = fs::read_to_string(&manifest_path).expect("read manifest");
    assert!(
        manifest.contains(
            "QnCons { head: ^v2, tail: QnCons { head: ^compiler, tail: QnCons { head: ^compile, tail: QnEmpty } } }"
        ),
        "manifest admission subject QN malformed:\n{manifest}"
    );
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn parse_compiler_entry_admission_imports_compile_module() {
    let ws = workspace_root();
    let source = fs::read_to_string(ws.join(COMPILER_ENTRY)).expect("read entry");
    let admission = parse_source_root_entry_admission(&source).expect("parse admission");
    assert_eq!(admission.subject, vec!["v2", "compiler", "compile"]);
    assert!(
        admission
            .imports
            .iter()
            .any(|p| p == &["v2", "compiler", "emit"]),
        "expected compile module imports to include v2.compiler.emit: {:?}",
        admission.imports
    );
}

#[test]
#[ignore = "gap4 probe — prefix-scan first scoped-ingest Reject (manual)"]
fn probe_gap4_scoped_ingest_first_reject() {
    let ws = workspace_root();
    let v2_root = ws.join("src/v2").to_string_lossy().to_string();
    let records = discover_source_root_reads_for_entry(
        &[v2_root],
        COMPILER_ENTRY,
        &["host_source_root_ingest_manifest.dag".to_string()],
    )
    .expect("discover scoped compiler closure reads");
    assert!(
        !records.is_empty(),
        "expected non-empty scoped compiler closure ingest"
    );

    let entry_source = fs::read_to_string(ws.join(COMPILER_ENTRY)).expect("read entry");
    let admission =
        parse_source_root_entry_admission(&entry_source).expect("parse entry admission");

    let temp = unique_temp_dir("gap4-probe");
    fs::create_dir_all(&temp).expect("temp dir");

    let mut failures: Vec<String> = Vec::new();
    for (idx, rec) in records.iter().enumerate() {
        let manifest_path = temp.join(format!("manifest-{idx}.dag"));
        emit_source_root_ingest_manifest(&manifest_path, std::slice::from_ref(rec), Some(&admission))
            .expect("emit single-file manifest");

        let manifest_dir = manifest_path.parent().expect("manifest parent");
        let ctx = nv2_eval_context(manifest_dir).expect("resolve nv2 gate ctx");
        match run_claim(&ctx, "compiler_closure_scoped_ingest_parses") {
            ClaimOutcome::Pass => {}
            ClaimOutcome::Fail => {
                failures.push(format!(
                    "  [{idx}] file_path={} module_path={}",
                    rec.file_path, rec.module_path
                ));
            }
            ClaimOutcome::NotBool { got } => {
                failures.push(format!(
                    "  [{idx}] file_path={} module_path={} non-Bool={got}",
                    rec.file_path, rec.module_path
                ));
            }
            ClaimOutcome::RuntimeError { message } => {
                failures.push(format!(
                    "  [{idx}] file_path={} module_path={} runtime_error={message}",
                    rec.file_path, rec.module_path
                ));
            }
        }
    }

    let report = if failures.is_empty() {
        format!(
            "gap4 probe: all {} scoped ingest files passed in isolation (unexpected — full fold failed)",
            records.len()
        )
    } else {
        format!(
            "gap4 scoped ingest per-file Reject(s) ({}/{} fail):\n{}",
            failures.len(),
            records.len(),
            failures.join("\n")
        )
    };
    let report_path = std::env::temp_dir().join("gap4-scoped-ingest-probe.txt");
    fs::write(&report_path, &report).expect("write probe report");
    eprintln!("{report}");
    eprintln!("probe report: {}", report_path.display());
    if failures.is_empty() {
        panic!("{report}");
    }
    panic!("{report}");
}
