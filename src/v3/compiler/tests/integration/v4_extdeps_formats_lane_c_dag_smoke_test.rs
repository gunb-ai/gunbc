//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on Lane C remaining-format models (`csv`,
//! `json_schema`, `openapi`) — T-4.6 D2-resolver slice must lower+infer with
//! **zero** module diagnostics (same gate as `v4_extdeps_typescript_dag_smoke_test`).

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const CSV_DAG: &str = include_str!("../../../../v4/extdeps/formats/csv.dag");
const CSV_PATH: &str = "src/v4/extdeps/formats/csv.dag";

const JSON_SCHEMA_DAG: &str = include_str!("../../../../v4/extdeps/formats/json_schema.dag");
const JSON_SCHEMA_PATH: &str = "src/v4/extdeps/formats/json_schema.dag";

const OPENAPI_DAG: &str = include_str!("../../../../v4/extdeps/formats/openapi.dag");
const OPENAPI_PATH: &str = "src/v4/extdeps/formats/openapi.dag";

fn assert_zero_diagnostics(source: &str, path: &str) {
    match compile_to_dag(source, path) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{path}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{path}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{path}: {other:?}"),
    }
}

#[test]
fn v4_extdeps_formats_csv_dag_compiles() {
    assert_zero_diagnostics(CSV_DAG, CSV_PATH);
}

#[test]
fn v4_extdeps_formats_json_schema_dag_compiles() {
    assert_zero_diagnostics(JSON_SCHEMA_DAG, JSON_SCHEMA_PATH);
}

#[test]
fn v4_extdeps_formats_openapi_dag_compiles() {
    assert_zero_diagnostics(OPENAPI_DAG, OPENAPI_PATH);
}
