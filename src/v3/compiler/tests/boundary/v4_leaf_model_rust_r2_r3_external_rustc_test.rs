//! **Layer:** boundary — rustc target exercise for v4 leaf-model R2a / R2b / R3-external.
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag`; claim wiring in
//! `rust_r2a.dag`, `rust_r2b.dag`, `rust_r3_external.dag`.
//! Host runners: `scripts/v4-leaf-model-rust-r2{a,b,r3-external}-verify.sh`.

use std::io::Write;
use std::process::Command;

use crate::common::RustcHarness;

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");

const R2A_HAPPY: &str = "pub fn r2a_test(a: i32, b: i32) -> (i32, bool) { (a + b, a < b) }\n";
const R2A_FALSIFICATION: &str = "pub fn r2a_test(a: i32) -> i32 { a.log2_exact() }\n";

const R2B_RUNTIME: &str =
    "pub fn r2b_test(a: i32, b: i32) -> i32 { a + b }\npub fn main() { let _ = r2b_test(i32::MAX, 1); }\n";

const R3_HAPPY: &str =
    "pub struct Symbol(pub String);\npub fn r3_test() -> Symbol { Symbol(\"x\".to_string()) }\n";
const R3_FALSIFICATION: &str = "pub type Symbol = String;\npub fn r3_test() -> Symbol { Symbol(\"x\") }\n";

fn unescape_dag_string_literal(body: &str) -> String {
    let mut out = String::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn extract_fixture_source(dag_text: &str, data_name: &str) -> Option<String> {
    let needle = format!("data {data_name}: String = \"");
    let line = dag_text.lines().find(|l| l.starts_with(&needle))?;
    let rest = line.strip_prefix(&needle)?.strip_suffix('"')?;
    Some(unescape_dag_string_literal(rest))
}

fn compile_lib(harness: &RustcHarness, label: &str, source: &str) -> (bool, String) {
    let tmp_dir = harness.next_child_dir();
    let src_path = tmp_dir.join(format!("{label}.rs"));
    let out_path = tmp_dir.join(format!("{label}.rlib"));
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write fixture source");

    let output = Command::new("rustc")
        .env_remove("RUSTC_BOOTSTRAP")
        .arg("--edition=2021")
        .arg("--crate-type")
        .arg("lib")
        .arg(&src_path)
        .arg("-o")
        .arg(&out_path)
        .output()
        .expect("invoke rustc");

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.status.success(), stderr)
}

fn compile_bin_and_run(
    harness: &RustcHarness,
    label: &str,
    source: &str,
    extra_rustc_args: &[&str],
) -> (bool, i32, String) {
    let tmp_dir = harness.next_child_dir();
    let src_path = tmp_dir.join(format!("{label}.rs"));
    let bin_path = tmp_dir.join(label);
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write fixture source");

    let mut compile = Command::new("rustc");
    compile
        .env_remove("RUSTC_BOOTSTRAP")
        .arg("--edition=2021")
        .arg("--crate-type")
        .arg("bin")
        .args(extra_rustc_args)
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path);
    let compile_out = compile.output().expect("invoke rustc");
    let mut stderr = String::from_utf8_lossy(&compile_out.stderr).into_owned();
    if !compile_out.status.success() {
        return (false, -1, stderr);
    }

    let run_out = Command::new(&bin_path).output().expect("run binary");
    stderr.push_str(&String::from_utf8_lossy(&run_out.stderr));
    (true, run_out.status.code().unwrap_or(-1), stderr)
}

#[test]
fn v4_leaf_model_rust_r2a_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "rust_r2a_happy_fixture_source").unwrap();
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "rust_r2a_falsification_fixture_source").unwrap();
    assert_eq!(happy, R2A_HAPPY);
    assert_eq!(falsification, R2A_FALSIFICATION);
}

#[test]
fn v4_leaf_model_rust_r2a_happy_fixture_rustc_accepts() {
    let harness = RustcHarness::new("v4_leaf_model_rust_r2a");
    let (ok, stderr) = compile_lib(&harness, "happy", R2A_HAPPY);
    assert!(ok, "rustc should accept R2a happy fixture; stderr:\n{stderr}");
}

#[test]
fn v4_leaf_model_rust_r2a_falsification_fixture_rustc_e0599() {
    let harness = RustcHarness::new("v4_leaf_model_rust_r2a");
    let (ok, stderr) = compile_lib(&harness, "falsification", R2A_FALSIFICATION);
    assert!(!ok, "falsification must not compile");
    assert!(stderr.contains("E0599"), "expected E0599; stderr:\n{stderr}");
}

#[test]
fn v4_leaf_model_rust_r2b_runtime_fixture_strings_match_dag_authority() {
    let source = extract_fixture_source(FIXTURE_DAG, "rust_r2b_runtime_fixture_source").unwrap();
    assert_eq!(source, R2B_RUNTIME);
}

#[test]
fn v4_leaf_model_rust_r2b_debug_overflow_panics_at_runtime() {
    let harness = RustcHarness::new("v4_leaf_model_rust_r2b_debug");
    let (compiled, exit, stderr) = compile_bin_and_run(&harness, "debug", R2B_RUNTIME, &[]);
    assert!(compiled, "debug build must compile; stderr:\n{stderr}");
    assert_ne!(exit, 0, "debug overflow must panic (non-zero exit); stderr:\n{stderr}");
}

#[test]
fn v4_leaf_model_rust_r2b_release_overflow_wraps_at_runtime() {
    let harness = RustcHarness::new("v4_leaf_model_rust_r2b_release");
    let (compiled, exit, stderr) =
        compile_bin_and_run(&harness, "release", R2B_RUNTIME, &["-C", "opt-level=2"]);
    assert!(compiled, "release build must compile; stderr:\n{stderr}");
    assert_eq!(exit, 0, "release overflow must wrap (exit 0); stderr:\n{stderr}");
}

#[test]
fn v4_leaf_model_rust_r3_external_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "rust_r3_external_happy_fixture_source").unwrap();
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "rust_r3_external_falsification_fixture_source")
            .unwrap();
    assert_eq!(happy, R3_HAPPY);
    assert_eq!(falsification, R3_FALSIFICATION);
}

#[test]
fn v4_leaf_model_rust_r3_external_happy_fixture_rustc_accepts() {
    let harness = RustcHarness::new("v4_leaf_model_rust_r3_external");
    let (ok, stderr) = compile_lib(&harness, "happy", R3_HAPPY);
    assert!(ok, "rustc should accept R3 happy fixture; stderr:\n{stderr}");
}

#[test]
fn v4_leaf_model_rust_r3_external_falsification_fixture_rustc_e0423() {
    let harness = RustcHarness::new("v4_leaf_model_rust_r3_external");
    let (ok, stderr) = compile_lib(&harness, "falsification", R3_FALSIFICATION);
    assert!(!ok, "falsification must not compile");
    assert!(stderr.contains("E0423"), "expected E0423; stderr:\n{stderr}");
}
