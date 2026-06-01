//! **Layer:** boundary - Python leaf-model L1 static and L2 cross-target receipts.
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag`,
//! `src/v4/extdeps/typecheckers/{pyright,mypy}.dag`, and
//! `src/v4/test/claim/language_model/python_l1_static.dag`.
//! Host runners: `scripts/v4-leaf-model-python-l1-{static,mypy-static}-verify.sh`
//! and `scripts/v4-leaf-model-python-l2-cross-target-parity-verify.sh`.
//!
//! **P5 receipt:** temporary hand-Rust boundary ratchet until T-22 modeled
//! `run_target_verification` executes the static and cross-target receipts from `.dag`.

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");
const PYRIGHT_DAG: &str = include_str!("../../../../v4/extdeps/typecheckers/pyright.dag");
const MYPY_DAG: &str = include_str!("../../../../v4/extdeps/typecheckers/mypy.dag");
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/language_model/python_l1_static.dag");
const WORKSHEET: &str =
    include_str!("../../../../../docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md");

const L1_STATIC_HAPPY: &str = "def l1_test() -> int:\n    return 0\n";
const L1_STATIC_FALSIFICATION: &str = "def l1_test() -> int:\n    return \"not an int\"\n";
const L2_EXPECTED_STDOUT: &str = "r2a:3:true\nr3:x\n";

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

fn extract_fixture_source(dag_text: &str, data_name: &str) -> String {
    let needle = format!("data {data_name}: String = \"");
    let line = dag_text
        .lines()
        .find(|line| line.starts_with(&needle))
        .unwrap_or_else(|| panic!("missing {data_name} in fixture dag"));
    let rest = line
        .strip_prefix(&needle)
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or_else(|| panic!("malformed string literal for {data_name}"));
    unescape_dag_string_literal(rest)
}

#[test]
fn v4_leaf_model_python_l1_static_fixture_sources_match_dag_authority() {
    assert_eq!(
        extract_fixture_source(FIXTURE_DAG, "python_l1_static_happy_fixture_source"),
        L1_STATIC_HAPPY
    );
    assert_eq!(
        extract_fixture_source(FIXTURE_DAG, "python_l1_static_falsification_fixture_source"),
        L1_STATIC_FALSIFICATION
    );
}

#[test]
fn v4_leaf_model_python_l1_static_profiles_are_distinct_authorities() {
    assert!(PYRIGHT_DAG.contains("data pyright_profile_l1: PyrightConfig"));
    assert!(PYRIGHT_DAG.contains("pyright_diag_report_return_type"));
    assert!(MYPY_DAG.contains("data mypy_profile_l1: MypyConfig"));
    assert!(MYPY_DAG.contains("mypy_diag_return_value"));
    assert!(FIXTURE_DAG.contains("fn python_l1_static_fixture_pair()"));
    assert!(FIXTURE_DAG.contains("fn python_l1_static_mypy_fixture_pair()"));
    assert!(FIXTURE_DAG.contains("tool_profile_ref: pyright_profile_l1_id"));
    assert!(FIXTURE_DAG.contains("tool_profile_ref: mypy_profile_l1_id"));
}

#[test]
fn v4_leaf_model_python_l1_static_claim_wires_blocking_verdicts() {
    assert!(CLAIM_DAG.contains("claim_python_l1_static_fixture_pair_wired"));
    assert!(CLAIM_DAG.contains("BlockingForRung"));
    assert!(CLAIM_DAG.contains("StaticAnalysisRejected"));
    assert!(CLAIM_DAG.contains("pyright_diag_report_return_type"));
}

#[test]
fn v4_leaf_model_python_l2_cross_target_parity_sources_match_dag_authority() {
    let python = extract_fixture_source(FIXTURE_DAG, "python_l2_parity_python_source");
    let rust = extract_fixture_source(FIXTURE_DAG, "python_l2_parity_rust_source");
    let go = extract_fixture_source(FIXTURE_DAG, "python_l2_parity_go_source");
    let expected = extract_fixture_source(FIXTURE_DAG, "python_l2_parity_expected_stdout");
    assert!(python.contains("def r2a(a: int, b: int)"));
    assert!(rust.contains("fn r2a(a: i32, b: i32)"));
    assert!(go.contains("func r2a(a int, b int)"));
    assert_eq!(expected, L2_EXPECTED_STDOUT);
}

#[test]
fn v4_python_rca_worksheets_name_static_vs_behavioral_split() {
    assert!(WORKSHEET.contains("Worksheet A - pyright L1 Static Structural"));
    assert!(WORKSHEET.contains("Worksheet B - mypy L1 Static Structural"));
    assert!(WORKSHEET.contains("Worksheet C - L2 Cross-Target Behavioral Parity"));
    assert!(WORKSHEET.contains("R2b arbitrary precision is not asserted as cross-target equality"));
}
