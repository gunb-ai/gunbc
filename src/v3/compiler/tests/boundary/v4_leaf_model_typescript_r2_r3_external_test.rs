//! **Layer:** boundary — tsc + Node target exercise for v4 leaf-model typescript R2a / R2b / R3-external.
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag`; claim wiring in
//! `typescript_r2a.dag`, `typescript_r2b.dag`, `typescript_r3_external.dag`.
//! Host runners: `scripts/v4-leaf-model-typescript-r2{a,b,r3-external}-verify.sh`.

use std::io::Write;
use std::process::Command;

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");

const TSC_PKG: &str = "typescript@5.9.2";

const R2A_HAPPY: &str =
    "function r2a_test(a: number, b: number): [number, boolean] { return [a + b, a < b]; }\n";
const R2A_FALSIFICATION: &str = "function r2a_test(a: number): number { return a.log2_exact(); }\n";

const R2B_RUNTIME_HAPPY: &str =
    "const ok = (2n ** 63n - 1n) + 1n === 2n ** 63n;\nif (!ok) { console.error(\"r2b bigint lane failed\"); process.exit(1); }\n";
const R2B_RUNTIME_FALSIFICATION: &str = "const bigintExact = (2n ** 63n + 1n) === 2n ** 63n;\nconst numberLane = (2 ** 63 + 1) === 2 ** 63;\nif (bigintExact || !numberLane) { console.error(\"expected bigint inequality and number-lane IEEE754 false positive\"); process.exit(1); }\n";

const R3_HAPPY: &str = "const s: symbol = Symbol(\"x\");\n";
const R3_FALSIFICATION: &str = "const s: symbol = new Symbol(\"x\");\n";

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
    std::env::temp_dir().join(format!("v4_leaf_model_typescript_r2_r3_{label}_{suffix}"))
}

fn exercise_tsc_fixture(label: &str, source: &str) -> (i32, String) {
    let tmp_dir = scratch_dir(label);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).expect("create scratch dir");
    let src_path = tmp_dir.join(format!("{label}.ts"));
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write fixture source");

    let output = Command::new("npx")
        .args([
            "-p", TSC_PKG, "tsc", "--strict", "--noEmit", "--target", "ES2022", "--module",
            "ES2022",
        ])
        .arg(&src_path)
        .output()
        .expect("invoke npx tsc");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let status = output.status.code().unwrap_or(1);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    (status, stderr)
}

fn exercise_node_fixture(label: &str, source: &str) -> (i32, String) {
    let tmp_dir = scratch_dir(label);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).expect("create scratch dir");
    let src_path = tmp_dir.join(format!("{label}.mjs"));
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write fixture source");

    let output = Command::new("node")
        .arg(&src_path)
        .output()
        .expect("invoke node on fixture");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let status = output.status.code().unwrap_or(1);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    (status, stderr)
}

#[test]
fn v4_leaf_model_typescript_r2a_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "typescript_r2a_happy_fixture_source").unwrap();
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "typescript_r2a_falsification_fixture_source").unwrap();
    assert_eq!(happy, R2A_HAPPY);
    assert_eq!(falsification, R2A_FALSIFICATION);
}

#[test]
fn v4_leaf_model_typescript_r2a_happy_fixture_tsc_accepts() {
    let (status, stderr) = exercise_tsc_fixture("r2a_happy", R2A_HAPPY);
    assert_eq!(
        status, 0,
        "tsc should accept R2a happy fixture; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_typescript_r2a_falsification_fixture_tsc_ts2339() {
    let (status, stderr) = exercise_tsc_fixture("r2a_falsification", R2A_FALSIFICATION);
    assert_ne!(status, 0, "falsification must not typecheck");
    assert!(
        stderr.contains("TS2339") && stderr.contains("log2_exact"),
        "expected TS2339 on log2_exact; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_typescript_r2b_runtime_fixture_strings_match_dag_authority() {
    let happy =
        extract_fixture_source(FIXTURE_DAG, "typescript_r2b_runtime_happy_fixture_source").unwrap();
    let falsification = extract_fixture_source(
        FIXTURE_DAG,
        "typescript_r2b_runtime_falsification_fixture_source",
    )
    .unwrap();
    assert_eq!(happy, R2B_RUNTIME_HAPPY);
    assert_eq!(falsification, R2B_RUNTIME_FALSIFICATION);
}

#[test]
fn v4_leaf_model_typescript_r2b_bigint_runtime_add_succeeds() {
    let (status, stderr) = exercise_node_fixture("r2b_happy", R2B_RUNTIME_HAPPY);
    assert_eq!(
        status, 0,
        "bigint add beyond 2**63 must succeed at runtime; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_typescript_r2b_number_lane_falsification_demonstrates_ieee754_divergence() {
    let (status, stderr) = exercise_node_fixture("r2b_falsification", R2B_RUNTIME_FALSIFICATION);
    assert_eq!(
        status, 0,
        "falsification must show bigint inequality while number lane false-positive-matches; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_typescript_r3_external_fixture_strings_match_dag_authority() {
    let happy =
        extract_fixture_source(FIXTURE_DAG, "typescript_r3_external_happy_fixture_source").unwrap();
    let falsification = extract_fixture_source(
        FIXTURE_DAG,
        "typescript_r3_external_falsification_fixture_source",
    )
    .unwrap();
    assert_eq!(happy, R3_HAPPY);
    assert_eq!(falsification, R3_FALSIFICATION);
}

#[test]
fn v4_leaf_model_typescript_r3_external_happy_fixture_tsc_accepts() {
    let (status, stderr) = exercise_tsc_fixture("r3_happy", R3_HAPPY);
    assert_eq!(
        status, 0,
        "tsc should accept R3 happy Symbol() factory fixture; stderr:\n{stderr}"
    );
}

#[test]
fn v4_leaf_model_typescript_r3_external_falsification_fixture_tsc_not_constructable() {
    let (status, stderr) = exercise_tsc_fixture("r3_falsification", R3_FALSIFICATION);
    assert_ne!(status, 0, "falsification must not typecheck");
    assert!(
        stderr.contains("TS7009") || stderr.contains("construct"),
        "expected TS7009 / not-constructable on `new Symbol`; stderr:\n{stderr}"
    );
}
