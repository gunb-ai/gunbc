//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/languages/cpp.dag` — T-4 C++
//! LanguageModel D2-resolver slice must lower+infer with **zero** module diagnostics.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const CPP_DAG: &str = include_str!("../../../../v4/extdeps/languages/cpp.dag");
const CPP_PATH: &str = "src/v4/extdeps/languages/cpp.dag";

#[test]
fn v4_extdeps_cpp_dag_compiles() {
    match compile_to_dag(CPP_DAG, CPP_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{CPP_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{CPP_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{CPP_PATH}: {other:?}"),
    }
}
