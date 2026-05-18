//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/cpp_abi.dag` — T-29's
//! C++ ABI / target data-model slice must lower+infer with zero module
//! diagnostics before `cpp.dag` fact-bundles consume it.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const CPP_ABI_DAG: &str = include_str!("../../../../v4/extdeps/cpp_abi.dag");
const CPP_ABI_PATH: &str = "src/v4/extdeps/cpp_abi.dag";

#[test]
fn v4_extdeps_cpp_abi_dag_compiles() {
    match compile_to_dag(CPP_ABI_DAG, CPP_ABI_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{CPP_ABI_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{CPP_ABI_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{CPP_ABI_PATH}: {other:?}"),
    }
}
