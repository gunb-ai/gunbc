//! **Layer:** boundary - Python leaf-model L1 static structural receipts (pyright + mypy).
//!
//! Authority: `src/v4/lens/leaf_model_verification.dag`,
//! `src/v4/extdeps/typecheckers/{pyright,mypy}.dag`, and
//! `src/v4/test/claim/language_model/python_l1_static.dag`.
//! Host runner (Worksheet B, in-tree): `scripts/v4-leaf-model-python-l1-mypy-static-verify.sh`.
//! Pyright Worksheet A `.dag` authorities are on-tree; the pyright host runner path is not
//! (deleted `scripts/v4-leaf-model-python-l1-static-verify.sh` in #4252 script hygiene).
//!
//! **P5 receipt (`.github/PULL_REQUEST_TEMPLATE.md` Per-PR gate, disposition (2)):** same-PR
//! `EXPECTED_HAND_AUTHORED_TEST` census delta **171 → 172** paths (literal in
//! `self_gen_census_test.rs`; T-PB-B partition: module doc lines 9–10 +
//! `tests/boundary/README.md`). Workstream/lane (in-tree):
//! `docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md` Worksheet B (#4137 §11.8).
//! **Dissolution:** delete this path when host runners are superseded by modeled verification
//! over `src/v4/test/claim/language_model/python_l1_static.dag` (`src/v4/std/leaf_model_verification.dag`
//! T-22 `run_target_verification` landing notes).

const FIXTURE_DAG: &str = include_str!("../../../../v4/lens/leaf_model_verification.dag");
const PYRIGHT_DAG: &str = include_str!("../../../../v4/extdeps/typecheckers/pyright.dag");
const MYPY_DAG: &str = include_str!("../../../../v4/extdeps/typecheckers/mypy.dag");
const CLAIM_DAG: &str =
    include_str!("../../../../v4/test/claim/language_model/python_l1_static.dag");
const WORKSHEET: &str =
    include_str!("../../../../../docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md");

const L1_STATIC_HAPPY: &str = "def l1_test() -> int:\n    return 0\n";
const L1_STATIC_FALSIFICATION: &str = "def l1_test() -> int:\n    return \"not an int\"\n";

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
    assert!(CLAIM_DAG.contains("claim_python_l1_static_mypy_fixture_pair_wired"));
    assert!(CLAIM_DAG.contains("claim_python_l1_static_mypy_expected_verdicts_wired"));
    assert!(CLAIM_DAG.contains("BlockingForRung"));
    assert!(CLAIM_DAG.contains("StaticAnalysisRejected"));
    assert!(CLAIM_DAG.contains("pyright_diag_report_return_type"));
    assert!(CLAIM_DAG.contains("mypy_diag_return_value"));
    assert!(CLAIM_DAG.contains("python_l1_static_mypy_fixture"));
    assert!(CLAIM_DAG.contains("mypy_profile_l1_id"));
}

#[test]
fn v4_python_rca_worksheets_name_pyright_and_mypy_l1_static_lanes() {
    assert!(WORKSHEET.contains("Worksheet A - pyright L1 Static Structural"));
    assert!(WORKSHEET.contains("Worksheet B - mypy L1 Static Structural"));
    assert!(WORKSHEET.contains("python_l1_static_mypy_fixture"));
    assert!(WORKSHEET.contains("scripts/v4-leaf-model-python-l1-mypy-static-verify.sh"));
}
