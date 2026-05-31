//! **Layer:** boundary — CPython target exercise for v4 leaf-model python R2a / R2b / R3-external.
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag`; claim wiring in
//! `python_r2a.dag`, `python_r2b.dag`, `python_r3_external.dag`.
//! Host runners: `scripts/v4-leaf-model-python-r2{a,b,r3-external}-verify.sh`.

use std::io::Write;
use std::process::Command;

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");

const R2A_HAPPY: &str = "def r2a_test(a: int, b: int) -> tuple[int, bool]:\n    return (a + b, a < b)\n";
const R2A_FALSIFICATION: &str = "def r2a_test(a: int) -> int:\n    return a.log2_exact()\n\nif __name__ == \"__main__\":\n    r2a_test(1)\n";

const R2B_RUNTIME: &str = "def r2b_test() -> int:\n    return (2**63 - 1) + 1\n\nif __name__ == \"__main__\":\n    assert r2b_test() == 2**63\n";

const R3_HAPPY: &str = "class Symbol:\n    def __init__(self, value: str):\n        self.value = value\n\ndef r3_test() -> Symbol:\n    return Symbol(\"x\")\n\nif __name__ == \"__main__\":\n    r3_test()\n";
const R3_FALSIFICATION: &str = "class Symbol:\n    def __init__(self, value: str):\n        self.value = value\n\ndef r3_test() -> Symbol:\n    return Symbol(1, 2)\n\nif __name__ == \"__main__\":\n    r3_test()\n";

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
    std::env::temp_dir().join(format!("v4_leaf_model_python_r2_r3_{label}_{suffix}"))
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
fn v4_leaf_model_python_r2a_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "python_r2a_happy_fixture_source").unwrap();
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "python_r2a_falsification_fixture_source").unwrap();
    assert_eq!(happy, R2A_HAPPY);
    assert_eq!(falsification, R2A_FALSIFICATION);
}

#[test]
fn v4_leaf_model_python_r2a_happy_fixture_python_accepts() {
    let (status, stderr) = exercise_fixture("r2a_happy", R2A_HAPPY);
    assert_eq!(
        status, 0,
        "python3 should accept R2a happy fixture; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_python_r2a_falsification_fixture_python_attribute_error() {
    let (status, stderr) = exercise_fixture("r2a_falsification", R2A_FALSIFICATION);
    assert_ne!(status, 0, "falsification must not run clean");
    assert!(
        stderr.contains("AttributeError") && stderr.contains("log2_exact"),
        "expected AttributeError on log2_exact; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_python_r2b_runtime_fixture_strings_match_dag_authority() {
    let source = extract_fixture_source(FIXTURE_DAG, "python_r2b_runtime_fixture_source").unwrap();
    assert_eq!(source, R2B_RUNTIME);
}

#[test]
fn v4_leaf_model_python_r2b_arbitrary_precision_add_succeeds_at_runtime() {
    let (status, stderr) = exercise_fixture("r2b_runtime", R2B_RUNTIME);
    assert_eq!(
        status, 0,
        "arbitrary-precision add beyond 2**63 must succeed; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_python_r3_external_fixture_strings_match_dag_authority() {
    let happy =
        extract_fixture_source(FIXTURE_DAG, "python_r3_external_happy_fixture_source").unwrap();
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "python_r3_external_falsification_fixture_source")
            .unwrap();
    assert_eq!(happy, R3_HAPPY);
    assert_eq!(falsification, R3_FALSIFICATION);
}

#[test]
fn v4_leaf_model_python_r3_external_happy_fixture_python_accepts() {
    let (status, stderr) = exercise_fixture("r3_happy", R3_HAPPY);
    assert_eq!(
        status, 0,
        "python3 should accept R3 happy fixture; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_python_r3_external_falsification_fixture_python_type_error() {
    let (status, stderr) = exercise_fixture("r3_falsification", R3_FALSIFICATION);
    assert_ne!(status, 0, "falsification must not run clean");
    assert!(
        stderr.contains("TypeError") && stderr.contains("Symbol.__init__()"),
        "expected TypeError on Symbol.__init__ arity; stderr:\n{stderr}"
    );
}
