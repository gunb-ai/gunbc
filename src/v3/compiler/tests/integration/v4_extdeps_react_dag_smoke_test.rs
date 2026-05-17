//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/frameworks/react.dag` —
//! T-4.7 React framework substrate must lower+infer with **zero** module
//! diagnostics (same 0-diag gate as `v4_extdeps_typescript_dag_smoke_test`).

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const REACT_DAG: &str = include_str!("../../../../v4/extdeps/frameworks/react.dag");
const REACT_PATH: &str = "src/v4/extdeps/frameworks/react.dag";

/// Receipt: `useMemo` deps use omit-vs-present (`ReactHookInlineDependenciesArgument`),
/// not a bare `List<…>` that would conflate omitted second arg with explicit `[]`.
const USE_MEMO_DEPS_FIELD_SHAPE: &str = concat!(
    "| UseMemo {\n",
    "      factory_ref: ReactCrossDeclRef\n",
    "      dependencies: ReactHookInlineDependenciesArgument\n",
    "    }"
);

#[test]
fn v4_extdeps_react_dag_compiles() {
    assert!(
        REACT_DAG.contains(USE_MEMO_DEPS_FIELD_SHAPE),
        "{REACT_PATH}: expected UseMemo {{ … dependencies: ReactHookInlineDependenciesArgument … }}"
    );
    match compile_to_dag(REACT_DAG, REACT_PATH) {
        Ok(dag) => assert!(
            dag.diagnostics().is_empty(),
            "{REACT_PATH}: expected empty diagnostics, got {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(CompileError::Semantic(dag)) => panic!(
            "{REACT_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{REACT_PATH}: {other:?}"),
    }
}
