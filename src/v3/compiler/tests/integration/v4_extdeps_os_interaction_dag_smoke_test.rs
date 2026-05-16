//! **Layer:** integration

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const FILE_SYSTEM_DAG: &str = include_str!("../../../../v4/extdeps/file_system.dag");
const FILE_SYSTEM_PATH: &str = "src/v4/extdeps/file_system.dag";
const PROCESS_DAG: &str = include_str!("../../../../v4/extdeps/process.dag");
const PROCESS_PATH: &str = "src/v4/extdeps/process.dag";

fn assert_v4_dag_compiles(source: &str, path: &str) {
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
fn v4_extdeps_file_system_dag_compiles() {
    assert_v4_dag_compiles(FILE_SYSTEM_DAG, FILE_SYSTEM_PATH);
}

#[test]
fn v4_extdeps_process_dag_compiles() {
    assert_v4_dag_compiles(PROCESS_DAG, PROCESS_PATH);
}
