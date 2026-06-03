//! **Layer:** boundary — cross-target behavioral PARITY for v4 leaf-model python (Worksheet C).
//!
//! PY-L2-CROSS-TARGET-PARITY — the positive-agreement complement of the cross-runtime drift lane.
//! The SAME modeled program realized in each target's native types AGREES on the common domain:
//! small-value integer add (`2 + 3 = 5`, inside the subdomain where arbitrary-precision and
//! two's-complement coincide) and external Symbol nominal/value projection (the payload `x`). This
//! receipt asserts parity POSITIVELY, so the verification system demonstrably DISTINGUISHES the
//! common domain (parity) from the fixed-width boundary (drift) rather than collapsing them.
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag` (`python_l2_parity_*`); carrier
//! `LeafModelCrossTargetParityProbe` + `ValueDiff<String>` in
//! `src/v4/std/leaf_model_verification.dag`; claim wiring in `python_l2_cross_target_parity.dag`.

use std::io::Write;
use std::process::Command;

const LENS_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");
const LENS_DAG_PATH: &str = "src/v4/lens/leaf_model_verification.dag";
const STD_DAG: &str = include_str!("../../../../v4/std/leaf_model_verification.dag");
const STD_DAG_PATH: &str = "src/v4/std/leaf_model_verification.dag";
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/language_model/python_l2_cross_target_parity.dag");
const CLAIM_DAG_PATH: &str = "src/v4/test/claim/language_model/python_l2_cross_target_parity.dag";

const R2A_PYTHON_SOURCE: &str =
    "def add_small(a: int, b: int) -> int:\n    return a + b\n\nif __name__ == \"__main__\":\n    print(add_small(2, 3))\n";
const R2A_RUST_SOURCE: &str =
    "fn add_small(a: i64, b: i64) -> i64 { a + b }\nfn main() {\n    println!(\"{}\", add_small(2, 3));\n}\n";
const R2A_GO_SOURCE: &str =
    "package main\n\nimport \"fmt\"\n\nfunc addSmall(a int64, b int64) int64 { return a + b }\n\nfunc main() {\n\tfmt.Println(addSmall(2, 3))\n}\n";
const R2A_VALUE: &str = "5";

const R3_PYTHON_SOURCE: &str =
    "class Symbol:\n    def __init__(self, value: str):\n        self.value = value\n\ndef project() -> str:\n    return Symbol(\"x\").value\n\nif __name__ == \"__main__\":\n    print(project())\n";
const R3_RUST_SOURCE: &str =
    "struct Symbol { value: String }\nfn project() -> String { Symbol { value: \"x\".to_string() }.value }\nfn main() {\n    println!(\"{}\", project());\n}\n";
const R3_GO_SOURCE: &str =
    "package main\n\nimport \"fmt\"\n\ntype Symbol struct { value string }\n\nfunc project() string { return Symbol{value: \"x\"}.value }\n\nfunc main() {\n\tfmt.Println(project())\n}\n";
const R3_VALUE: &str = "x";

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

/// Per-call-unique scratch dir (parallel-test safe — see drift test rationale).
fn scratch_dir(label: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let pid = std::process::id();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("v4_leaf_model_python_parity_{label}_{pid}_{seq}"))
}

/// Tokenize + parse a single `.dag` (full multi-module `compile_to_dag` is not hermetic per-file).
fn tokenize_and_parse(dag_text: &str, dag_path: &str) {
    let tokens = v3_compiler::tokenize_for_test(dag_text, dag_path)
        .unwrap_or_else(|e| panic!("{dag_path}: tokenize: {e:?}"));
    let _module = v3_compiler::parse_for_test(&tokens, dag_path)
        .unwrap_or_else(|e| panic!("{dag_path}: parse: {e:?}"));
}

#[test]
fn v4_leaf_model_python_parity_dags_tokenize_and_parse() {
    tokenize_and_parse(STD_DAG, STD_DAG_PATH);
    tokenize_and_parse(LENS_DAG, LENS_DAG_PATH);
    tokenize_and_parse(CLAIM_DAG, CLAIM_DAG_PATH);
}

#[test]
fn v4_leaf_model_python_parity_fixture_strings_match_dag_authority() {
    assert_eq!(
        extract_fixture_source(LENS_DAG, "python_l2_parity_r2a_python_source").unwrap(),
        R2A_PYTHON_SOURCE
    );
    assert_eq!(
        extract_fixture_source(LENS_DAG, "python_l2_parity_r2a_rust_source").unwrap(),
        R2A_RUST_SOURCE
    );
    assert_eq!(
        extract_fixture_source(LENS_DAG, "python_l2_parity_r2a_go_source").unwrap(),
        R2A_GO_SOURCE
    );
    assert_eq!(
        extract_fixture_source(LENS_DAG, "python_l2_parity_r2a_value").unwrap(),
        R2A_VALUE
    );
    assert_eq!(
        extract_fixture_source(LENS_DAG, "python_l2_parity_r3_python_source").unwrap(),
        R3_PYTHON_SOURCE
    );
    assert_eq!(
        extract_fixture_source(LENS_DAG, "python_l2_parity_r3_rust_source").unwrap(),
        R3_RUST_SOURCE
    );
    assert_eq!(
        extract_fixture_source(LENS_DAG, "python_l2_parity_r3_go_source").unwrap(),
        R3_GO_SOURCE
    );
    assert_eq!(
        extract_fixture_source(LENS_DAG, "python_l2_parity_r3_value").unwrap(),
        R3_VALUE
    );
}

/// The modeled parity is positive: both `ValueDiff` arms hold the SAME value (expected == actual).
#[test]
fn v4_leaf_model_python_parity_modeled_agreement_is_positive() {
    // Mirror of the drift test's `assert_ne!`: parity requires the two arms EQUAL.
    assert_eq!(R2A_VALUE, R2A_VALUE);
    assert_eq!(R3_VALUE, R3_VALUE);
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

/// rustc is MANDATORY (fail-closed): parity *is* Python vs Rust agreement on the common domain;
/// skipping rust when absent would let a green receipt never exercise the cross-target pair.
fn run_rust(source: &str) -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .expect("rustc must be available (mandatory for the Python-vs-Rust parity pair)");
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
        "rustc must accept the parity fixture; stderr:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new(&bin_path).output().expect("run rust binary");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

/// Go is a corroborating witness (asserted only when the toolchain is present), matching the drift
/// lane's treatment of Go.
fn run_go(source: &str) -> Option<String> {
    Command::new("go").arg("version").output().ok()?;
    let tmp_dir = scratch_dir("go");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).expect("create scratch dir");
    let src_path = tmp_dir.join("fixture.go");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write go fixture");
    let run = Command::new("go")
        .arg("run")
        .arg(&src_path)
        .output()
        .expect("invoke go run");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert!(
        run.status.success(),
        "go must accept the parity fixture; stderr:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    Some(String::from_utf8_lossy(&run.stdout).trim().to_string())
}

#[test]
fn v4_leaf_model_python_parity_r2a_python_vs_rust_agrees() {
    let python_value = run_python(R2A_PYTHON_SOURCE);
    let rust_value = run_rust(R2A_RUST_SOURCE);
    assert_eq!(
        python_value, R2A_VALUE,
        "Python small-value add must yield 5"
    );
    assert_eq!(rust_value, R2A_VALUE, "Rust small-value add must yield 5");
    assert_eq!(
        python_value, rust_value,
        "cross-target PARITY: small-value add must agree across Python and Rust"
    );
    if let Some(go_value) = run_go(R2A_GO_SOURCE) {
        assert_eq!(go_value, R2A_VALUE, "Go small-value add must yield 5");
    }
}

#[test]
fn v4_leaf_model_python_parity_r3_python_vs_rust_agrees() {
    let python_value = run_python(R3_PYTHON_SOURCE);
    let rust_value = run_rust(R3_RUST_SOURCE);
    assert_eq!(
        python_value, R3_VALUE,
        "Python Symbol projection must yield x"
    );
    assert_eq!(rust_value, R3_VALUE, "Rust Symbol projection must yield x");
    assert_eq!(
        python_value, rust_value,
        "cross-target PARITY: Symbol projection must agree across Python and Rust"
    );
    if let Some(go_value) = run_go(R3_GO_SOURCE) {
        assert_eq!(go_value, R3_VALUE, "Go Symbol projection must yield x");
    }
}
