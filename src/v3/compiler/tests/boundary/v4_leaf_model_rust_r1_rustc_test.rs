//! **Layer:** boundary (TESTING.md § test layers — rustc target exercise for v4 leaf-model R1).
//!
//! Phase 1: rust.dag claim R1 (i32 surface spelling) — fixture authority in
//! `src/v4/lens/leaf_model_verification.dag`; host runner
//! `scripts/v4-leaf-model-rust-r1-verify.sh`.
//!
//! **P5 receipt:** matching `_internal/INVARIANTS_OPS.md` row +
//! `EXPECTED_HAND_AUTHORED_TEST` literal in `self_gen0_census_test.rs` (same PR).
//! **Dissolution:** retire when T-22 eval + modeled `run_target_verification` owns
//! rustc invocation without this hand-Rust bridge.

use std::io::Write;
use std::process::Command;

use crate::common::RustcHarness;

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");
const CLAIM_DAG: &str = include_str!("../../../../v4/test/claim/language_model/rust_r1.dag");

const HAPPY_FIXTURE: &str = "pub fn r1_test() -> i32 { 0i32 }\n";
const FALSIFICATION_FIXTURE: &str = "pub fn r1_test() -> i32 { \"string\" }\n";

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
        .expect("invoke rustc — install a rust toolchain to run this test");

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.status.success(), stderr)
}

#[test]
fn v4_leaf_model_rust_r1_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "rust_r1_happy_fixture_source")
        .expect("rust_r1_happy_fixture_source in lens/leaf_model_verification.dag");
    let falsification = extract_fixture_source(FIXTURE_DAG, "rust_r1_falsification_fixture_source")
        .expect("rust_r1_falsification_fixture_source in lens/leaf_model_verification.dag");
    assert_eq!(happy, HAPPY_FIXTURE);
    assert_eq!(falsification, FALSIFICATION_FIXTURE);
    assert!(
        CLAIM_DAG.contains("rust_r1_happy_fixture_source")
            && CLAIM_DAG.contains("rust_surface_spelling_i32"),
        "claim module must reference fixture + fact anchors"
    );
}

#[test]
fn v4_leaf_model_rust_r1_happy_fixture_rustc_accepts() {
    let harness = RustcHarness::new("v4_leaf_model_rust_r1");
    let (ok, stderr) = compile_lib(&harness, "happy", HAPPY_FIXTURE);
    assert!(
        ok,
        "rustc should accept R1 happy fixture; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_rust_r1_falsification_fixture_rustc_e0308() {
    let harness = RustcHarness::new("v4_leaf_model_rust_r1");
    let (ok, stderr) = compile_lib(&harness, "falsification", FALSIFICATION_FIXTURE);
    assert!(
        !ok,
        "falsification fixture must not compile (type mismatch probe)"
    );
    assert!(
        stderr.contains("E0308"),
        "expected rustc E0308 type mismatch; stderr:\n{stderr}"
    );
}
