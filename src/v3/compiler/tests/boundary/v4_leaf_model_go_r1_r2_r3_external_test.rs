//! **Layer:** boundary — Go target exercise for v4 leaf-model go R1 / R2a / R2b / R3-external.
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag`; claim wiring in
//! `go_r1.dag`, `go_r2a.dag`, `go_r2b.dag`, `go_r3_external.dag`.
//! Host runners: `scripts/v4-leaf-model-go-r{1,2a,2b,3-external}-verify.sh`.

use std::io::Write;
use std::process::{Command, Stdio};

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");

const R1_HAPPY: &str = "package leafmodel\n\nfunc r1() int { return 0 }\n";
const R1_FALSIFICATION: &str = "package leafmodel\n\nfunc r1() i32 { return 0 }\n";

const R2A_HAPPY: &str =
    "package leafmodel\n\nfunc r2a(a int, b int) (int, bool) { return a + b, a < b }\n";
const R2A_FALSIFICATION: &str =
    "package leafmodel\n\nfunc r2a(a int) int { return a.log2_exact() }\n";

const R2B_HAPPY: &str = "package main\n\nimport \"math\"\n\nfunc main() {\n    var x int64 = math.MaxInt64\n    got := x + 1\n    want := int64(math.MinInt64)\n    if got != want {\n        panic(\"silent signed wrap: expected MinInt64\")\n    }\n}\n";

const R2B_FALSIFICATION: &str = "package main\n\nimport \"math\"\n\nfunc main() {\n    var x int64 = math.MaxInt64\n    got := x + 1\n    want := int64(0)\n    if got != want {\n        panic(\"deliberately wrong expected wrap value\")\n    }\n}\n";

const R3_HAPPY: &str = "package leafmodel\n\nfunc r3() string { return \"x\" }\n";
const R3_FALSIFICATION: &str =
    "package leafmodel\n\nfunc r3() string { return string(\"x\", \"y\") }\n";

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

fn go_available() -> bool {
    Command::new("go")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .is_some_and(|status| status.success())
}

fn scratch_dir(label: &str) -> std::path::PathBuf {
    let suffix = std::process::id();
    std::env::temp_dir().join(format!("v4_leaf_model_go_{label}_{suffix}"))
}

fn exercise_go_build(label: &str, source: &str) -> (i32, String) {
    let tmp_dir = scratch_dir(label);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).expect("create scratch dir");
    let src_path = tmp_dir.join("fixture.go");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write fixture source");

    let out_path = tmp_dir.join("fixture.out");
    let output = Command::new("go")
        .env("GO111MODULE", "off")
        .args(["build", "-o", out_path.to_str().unwrap(), src_path.to_str().unwrap()])
        .output()
        .expect("invoke go build on fixture");
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let status = output.status.code().unwrap_or(1);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    (status, diagnostics)
}

fn exercise_go_run(label: &str, source: &str) -> (i32, String) {
    let tmp_dir = scratch_dir(label);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).expect("create scratch dir");
    let src_path = tmp_dir.join("main.go");
    std::fs::File::create(&src_path)
        .and_then(|mut f| f.write_all(source.as_bytes()))
        .expect("write fixture source");

    let output = Command::new("go")
        .env("GO111MODULE", "off")
        .args(["run", "main.go"])
        .current_dir(&tmp_dir)
        .output()
        .expect("invoke go run on fixture");
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let status = output.status.code().unwrap_or(1);
    let _ = std::fs::remove_dir_all(&tmp_dir);
    (status, diagnostics)
}

#[test]
fn v4_leaf_model_go_r1_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "go_r1_happy_fixture_source").unwrap();
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "go_r1_falsification_fixture_source").unwrap();
    assert_eq!(happy, R1_HAPPY);
    assert_eq!(falsification, R1_FALSIFICATION);
}

#[test]
fn v4_leaf_model_go_r1_happy_fixture_compiles() {
    if !go_available() {
        eprintln!("skipping Go toolchain boundary check: go not on PATH");
        return;
    }
    let (status, diagnostics) = exercise_go_build("r1_happy", R1_HAPPY);
    assert_eq!(
        status, 0,
        "go build should accept R1 happy fixture; diagnostics:\n{diagnostics}"
    );
}

#[test]
fn v4_leaf_model_go_r1_falsification_fixture_rejects_i32() {
    if !go_available() {
        eprintln!("skipping Go toolchain boundary check: go not on PATH");
        return;
    }
    let (status, diagnostics) = exercise_go_build("r1_falsification", R1_FALSIFICATION);
    assert_ne!(status, 0, "falsification must not compile");
    assert!(
        diagnostics.contains("undefined: i32") || diagnostics.contains("undefined type: i32"),
        "expected undefined i32 diagnostic; diagnostics:\n{diagnostics}"
    );
}

#[test]
fn v4_leaf_model_go_r2a_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "go_r2a_happy_fixture_source").unwrap();
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "go_r2a_falsification_fixture_source").unwrap();
    assert_eq!(happy, R2A_HAPPY);
    assert_eq!(falsification, R2A_FALSIFICATION);
}

#[test]
fn v4_leaf_model_go_r2a_happy_fixture_compiles() {
    if !go_available() {
        eprintln!("skipping Go toolchain boundary check: go not on PATH");
        return;
    }
    let (status, diagnostics) = exercise_go_build("r2a_happy", R2A_HAPPY);
    assert_eq!(
        status, 0,
        "go build should accept R2a happy fixture; diagnostics:\n{diagnostics}"
    );
}

#[test]
fn v4_leaf_model_go_r2a_falsification_fixture_rejects_method() {
    if !go_available() {
        eprintln!("skipping Go toolchain boundary check: go not on PATH");
        return;
    }
    let (status, diagnostics) = exercise_go_build("r2a_falsification", R2A_FALSIFICATION);
    assert_ne!(status, 0, "falsification must not compile");
    assert!(
        diagnostics.contains("log2_exact"),
        "expected missing log2_exact method diagnostic; diagnostics:\n{diagnostics}"
    );
}

#[test]
fn v4_leaf_model_go_r2b_runtime_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "go_r2b_happy_fixture_source").unwrap();
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "go_r2b_falsification_fixture_source").unwrap();
    assert_eq!(happy, R2B_HAPPY);
    assert_eq!(falsification, R2B_FALSIFICATION);
}

#[test]
fn v4_leaf_model_go_r2b_int64_silent_overflow_truncates_at_runtime() {
    if !go_available() {
        eprintln!("skipping Go toolchain boundary check: go not on PATH");
        return;
    }
    let (status, diagnostics) = exercise_go_run("r2b_happy", R2B_HAPPY);
    assert_eq!(
        status, 0,
        "go run should observe int64 silent wrap to MinInt64; diagnostics:\n{diagnostics}"
    );
}

#[test]
fn v4_leaf_model_go_r2b_falsification_fixture_panics_at_runtime() {
    if !go_available() {
        eprintln!("skipping Go toolchain boundary check: go not on PATH");
        return;
    }
    let (status, diagnostics) = exercise_go_run("r2b_falsification", R2B_FALSIFICATION);
    assert_ne!(
        status, 0,
        "falsification must panic on wrong expected wrap; diagnostics:\n{diagnostics}"
    );
    assert!(
        diagnostics.contains("deliberately wrong expected wrap value"),
        "expected deliberate panic message; diagnostics:\n{diagnostics}"
    );
}

#[test]
fn v4_leaf_model_go_r3_external_fixture_strings_match_dag_authority() {
    let happy = extract_fixture_source(FIXTURE_DAG, "go_r3_external_happy_fixture_source").unwrap();
    let falsification =
        extract_fixture_source(FIXTURE_DAG, "go_r3_external_falsification_fixture_source").unwrap();
    assert_eq!(happy, R3_HAPPY);
    assert_eq!(falsification, R3_FALSIFICATION);
}

#[test]
fn v4_leaf_model_go_r3_external_happy_fixture_compiles() {
    if !go_available() {
        eprintln!("skipping Go toolchain boundary check: go not on PATH");
        return;
    }
    let (status, diagnostics) = exercise_go_build("r3_happy", R3_HAPPY);
    assert_eq!(
        status, 0,
        "go build should accept R3 happy fixture; diagnostics:\n{diagnostics}"
    );
}

#[test]
fn v4_leaf_model_go_r3_external_falsification_fixture_rejects_conversion_arity() {
    if !go_available() {
        eprintln!("skipping Go toolchain boundary check: go not on PATH");
        return;
    }
    let (status, diagnostics) = exercise_go_build("r3_falsification", R3_FALSIFICATION);
    assert_ne!(status, 0, "falsification must not compile");
    assert!(
        diagnostics.contains("too many arguments")
            || diagnostics.contains("too many arguments in conversion"),
        "expected conversion arity diagnostic; diagnostics:\n{diagnostics}"
    );
}
