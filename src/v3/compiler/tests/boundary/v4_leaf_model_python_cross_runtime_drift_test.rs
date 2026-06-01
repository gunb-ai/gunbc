//! **Layer:** boundary — cross-runtime drift detection for v4 leaf-model python.
//!
//! PY-L2-CROSS-RUNTIME-DRIFT — the positive-divergence complement of the L2 cross-target
//! parity lane. The SAME modeled program (exact integer add at the fixed-width boundary,
//! `MAX + 1`) realized in each target's native integer DIVERGES: Python's `int` is arbitrary
//! precision (exact sum `9223372036854775808`), while Rust `i64` / Go `int64` are fixed-width
//! two's-complement (defined wraparound to `-9223372036854775808`). This receipt asserts the
//! drift POSITIVELY, so the verification system demonstrably DETECTS drift rather than treating
//! wraparound as parity.
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag` (`python_cross_runtime_drift_*`);
//! carrier `LeafModelCrossRuntimeDriftProbe` + `ValueDiff<String>` in
//! `src/v4/std/leaf_model_verification.dag`; claim wiring in `python_cross_runtime_drift.dag`;
//! host runner `scripts/v4-leaf-model-python-cross-runtime-drift-verify.sh`.

use std::io::Write;
use std::process::Command;

const LENS_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");
const LENS_DAG_PATH: &str = "src/v4/lens/leaf_model_verification.dag";
const STD_DAG: &str = include_str!("../../../../v4/std/leaf_model_verification.dag");
const STD_DAG_PATH: &str = "src/v4/std/leaf_model_verification.dag";
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/language_model/python_cross_runtime_drift.dag");
const CLAIM_DAG_PATH: &str = "src/v4/test/claim/language_model/python_cross_runtime_drift.dag";

const PYTHON_SOURCE: &str =
    "def add_exact(a: int, b: int) -> int:\n    return a + b\n\nif __name__ == \"__main__\":\n    print(add_exact((2 ** 63) - 1, 1))\n";
const RUST_SOURCE: &str = "fn add_exact(a: i64, b: i64) -> i64 { a.wrapping_add(b) }\nfn main() {\n    println!(\"{}\", add_exact(i64::MAX, 1));\n}\n";

const EXPECTED_ARBITRARY_PRECISION: &str = "9223372036854775808";
const EXPECTED_FIXED_WIDTH: &str = "-9223372036854775808";

fn unescape_dag_string_literal(body: &str) -> String {
    let mut out = String::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
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

/// Per-call-unique scratch dir. Rust tests run in parallel within one process and several tests
/// call `run_python` with the same label, so a pid+label path would race (one test's
/// `remove_dir_all` deletes another's fixture). A process-wide atomic counter makes every
/// invocation's directory unique — hermetic per TESTING.md.
fn scratch_dir(label: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let pid = std::process::id();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("v4_leaf_model_python_drift_{label}_{pid}_{seq}"))
}

/// Tokenize + parse a single `.dag` (full multi-module `compile_to_dag` is not hermetic per-file;
/// this catches surface/syntax regressions in the new drift carriers — same compromise as
/// `v4_bin_main_dag_smoke_test`).
fn tokenize_and_parse(dag_text: &str, dag_path: &str) {
    let tokens = v3_compiler::tokenize_for_test(dag_text, dag_path)
        .unwrap_or_else(|e| panic!("{dag_path}: tokenize: {e:?}"));
    let _module = v3_compiler::parse_for_test(&tokens, dag_path)
        .unwrap_or_else(|e| panic!("{dag_path}: parse: {e:?}"));
}

#[test]
fn v4_leaf_model_python_drift_dags_tokenize_and_parse() {
    tokenize_and_parse(STD_DAG, STD_DAG_PATH);
    tokenize_and_parse(LENS_DAG, LENS_DAG_PATH);
    tokenize_and_parse(CLAIM_DAG, CLAIM_DAG_PATH);
}

#[test]
fn v4_leaf_model_python_drift_fixture_strings_match_dag_authority() {
    let py = extract_fixture_source(LENS_DAG, "python_cross_runtime_drift_python_source").unwrap();
    let rs = extract_fixture_source(LENS_DAG, "python_cross_runtime_drift_rust_source").unwrap();
    let py_val =
        extract_fixture_source(LENS_DAG, "python_cross_runtime_drift_arbitrary_precision_value")
            .unwrap();
    let fw_val =
        extract_fixture_source(LENS_DAG, "python_cross_runtime_drift_fixed_width_value").unwrap();
    assert_eq!(py, PYTHON_SOURCE);
    assert_eq!(rs, RUST_SOURCE);
    assert_eq!(py_val, EXPECTED_ARBITRARY_PRECISION);
    assert_eq!(fw_val, EXPECTED_FIXED_WIDTH);
}

/// The modeled divergence is positive: the two `ValueDiff` arms differ.
#[test]
fn v4_leaf_model_python_drift_modeled_divergence_is_positive() {
    assert_ne!(
        EXPECTED_ARBITRARY_PRECISION, EXPECTED_FIXED_WIDTH,
        "drift receipt requires arbitrary-precision value != fixed-width value"
    );
}

fn run_python(source: &str) -> String {
    let tmp_dir = scratch_dir("py");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).expect("create scratch dir");
    let src_path = tmp_dir.join("fixture.py");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write python fixture");
    let out = Command::new("python3")
        .arg(&src_path)
        .output()
        .expect("invoke python3");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        out.status.success(),
        "python fixture must run clean; stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// rustc is MANDATORY (fail-closed): the cross-runtime drift claim *is* Python arbitrary
/// precision vs Rust fixed-width, and the host runner treats rustc as required for the same
/// reason. The repo's other rust boundary tests (`v4_leaf_model_rust_*`) likewise invoke rustc
/// unconditionally — it is the bootstrap toolchain, always present in a test environment.
/// Skipping the assertion when rustc is absent would let a green receipt never exercise the
/// drift pair, so we panic instead.
fn run_rust(source: &str) -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .expect("rustc must be available (mandatory for the Python-vs-Rust drift pair)");
    let tmp_dir = scratch_dir("rs");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).expect("create scratch dir");
    let src_path = tmp_dir.join("fixture.rs");
    let bin_path = tmp_dir.join("fixture_rs");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write rust fixture");
    let compile = Command::new("rustc")
        .env_remove("RUSTC_BOOTSTRAP")
        .arg("--edition=2021")
        .arg("-O")
        .arg(&src_path)
        .arg("-o")
        .arg(&bin_path)
        .output()
        .expect("invoke rustc");
    assert!(
        compile.status.success(),
        "rustc must accept the drift fixture; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&bin_path).output().expect("run rust binary");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

#[test]
fn v4_leaf_model_python_drift_python_realizes_arbitrary_precision() {
    let value = run_python(PYTHON_SOURCE);
    assert_eq!(
        value, EXPECTED_ARBITRARY_PRECISION,
        "Python int must realize the exact arbitrary-precision sum"
    );
}

#[test]
fn v4_leaf_model_python_drift_python_vs_rust_diverges() {
    let python_value = run_python(PYTHON_SOURCE);
    let rust_value = run_rust(RUST_SOURCE);
    assert_eq!(
        rust_value, EXPECTED_FIXED_WIDTH,
        "Rust i64 must wrap to i64::MIN at the fixed-width boundary"
    );
    assert_ne!(
        python_value, rust_value,
        "cross-runtime DRIFT: Python arbitrary precision must diverge from Rust fixed-width"
    );
}
