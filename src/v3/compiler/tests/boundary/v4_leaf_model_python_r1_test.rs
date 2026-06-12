//! **Layer:** boundary (TESTING.md § test layers — CPython exercise for v4 leaf-model python R1).
//!
//! Phase 1: python.dag claim R1 (int surface spelling) — fixture authority in
//! `src/v4/lens/leaf_model_verification.dag`; host runner
//! `scripts/v4-leaf-model-python-r1-verify.sh`.
//!
//! **P5 receipt:** matching `_internal/INVARIANTS_OPS.md` row +
//! `EXPECTED_HAND_AUTHORED_TEST` literal in `self_gen_census_test.rs` (same PR).
//! **Dissolution:** retire when T-22 eval + modeled `run_target_verification` owns
//! python3 invocation without this hand-Rust bridge.

use std::io::Write;
use std::process::Command;

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");
const CLAIM_DAG: &str = include_str!("../../../../v4/test/claim/language_model/python_r1.dag");

const HAPPY_FIXTURE: &str = "def r1_test() -> int:\n    return 0\n";
const FALSIFICATION_FIXTURE: &str = "def r1_test() -> i32:\n    return 0\n";

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

fn scratch_dir(label: &str) -> std::path::PathBuf {
    let suffix = std::process::id();
    std::env::temp_dir().join(format!("v4_leaf_model_python_r1_{label}_{suffix}"))
}

fn exercise_fixture(label: &str, source: &str) -> (i32, String) {
    let tmp_dir = scratch_dir(label);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).expect("create scratch dir");
    let src_path = tmp_dir.join(format!("{label}.py"));
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write fixture source");

    let py_compile = Command::new("python3")
        .args(["-m", "py_compile"])
        .arg(&src_path)
        .output()
        .expect("invoke python3 -m py_compile");
    if !py_compile.status.success() {
        let stderr = String::from_utf8_lossy(&py_compile.stderr).into_owned();
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return (py_compile.status.code().unwrap_or(1), stderr);
    }

    let exec = Command::new("python3")
        .arg(&src_path)
        .output()
        .expect("invoke python3 on fixture");
    let stderr = String::from_utf8_lossy(&exec.stderr).into_owned();
    let status = exec.status.code().unwrap_or(1);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    (status, stderr)
}

#[test]
fn v4_leaf_model_python_r1_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "python_r1_happy_fixture_source")
        .expect("python_r1_happy_fixture_source in lens/leaf_model_verification.dag");
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "python_r1_falsification_fixture_source")
            .expect("python_r1_falsification_fixture_source in lens/leaf_model_verification.dag");
    assert_eq!(happy, HAPPY_FIXTURE);
    assert_eq!(falsification, FALSIFICATION_FIXTURE);
    assert!(
        CLAIM_DAG.contains("python_r1_happy_fixture_source")
            && CLAIM_DAG.contains("python_surface_spelling_int"),
        "claim module must reference fixture + fact anchors"
    );
}

#[test]
fn v4_leaf_model_python_r1_happy_fixture_python_accepts() {
    let (status, stderr) = exercise_fixture("happy", HAPPY_FIXTURE);
    assert_eq!(
        status, 0,
        "python3 should accept R1 happy fixture; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_python_r1_falsification_fixture_python_name_error() {
    let (status, stderr) = exercise_fixture("falsification", FALSIFICATION_FIXTURE);
    assert_ne!(
        status, 0,
        "falsification fixture must not run clean (wrong int surface spelling i32)"
    );
    assert!(
        stderr.contains("NameError: name 'i32' is not defined"),
        "expected NameError on undefined i32 annotation; stderr:\n{stderr}"
    );
}
