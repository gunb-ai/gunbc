//! Smoke `compile_to_dag` on `src/v4/extdeps/typecheckers/pyright.dag` and
//! `src/v4/extdeps/typecheckers/mypy.dag` — PY-L1-STATIC-STRUCTURAL per-tool
//! static-analysis profiles (Arbiter ruling proud-fox-405, msg_41813c03).
//!
//! Each profile must lower+infer with zero module diagnostics and expose its
//! tool-id Symbol plus the per-tool diagnostic-rule namespace consumed by the
//! shared `TargetStaticAnalysisInvocation` / `TargetStaticAnalysisVerdict`
//! carriers in `v4.std.leaf_model_verification`.
//!
//! Single-file `compile_to_dag` cannot load `v4.std.*` peers; this harness lowers
//! the std prerequisite chain in dependency order (flat declaration table).

use v3_compiler::compile_to_dag_modules_in_order;
use v3_compiler::CompileError;

const NODE_DAG: &str = include_str!("../../../../v4/std/node.dag");
const ALGEBRA_DAG: &str = include_str!("../../../../v4/std/algebra.dag");
const LOGIC_DAG: &str = include_str!("../../../../v4/std/logic.dag");
const DIAGNOSTIC_DAG: &str = include_str!("../../../../v4/std/diagnostic.dag");
const WITNESS_DAG: &str = include_str!("../../../../v4/std/witness.dag");
const COLLECTION_DAG: &str = include_str!("../../../../v4/std/collection.dag");
const NAT_DAG: &str = include_str!("../../../../v4/std/nat.dag");
const TEXT_DAG: &str = include_str!("../../../../v4/std/text.dag");
const PYRIGHT_DAG: &str = include_str!("../../../../v4/extdeps/typecheckers/pyright.dag");
const MYPY_DAG: &str = include_str!("../../../../v4/extdeps/typecheckers/mypy.dag");

fn std_prefix() -> Vec<(&'static str, &'static str)> {
    vec![
        (NODE_DAG, "src/v4/std/node.dag"),
        (ALGEBRA_DAG, "src/v4/std/algebra.dag"),
        (LOGIC_DAG, "src/v4/std/logic.dag"),
        (DIAGNOSTIC_DAG, "src/v4/std/diagnostic.dag"),
        (WITNESS_DAG, "src/v4/std/witness.dag"),
        (COLLECTION_DAG, "src/v4/std/collection.dag"),
        (NAT_DAG, "src/v4/std/nat.dag"),
        (TEXT_DAG, "src/v4/std/text.dag"),
    ]
}

fn compile_or_panic(tail: (&'static str, &'static str)) -> v3_compiler::dag::Dag {
    let mut sources = std_prefix();
    sources.push(tail);
    match compile_to_dag_modules_in_order(&sources) {
        Ok(dag) => dag,
        Err(CompileError::Semantic(dag)) => panic!(
            "{}: semantic errors: {:?}",
            tail.1,
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{}: {other:?}", tail.1),
    }
}

#[test]
fn v4_extdeps_typecheckers_pyright_dag_compiles() {
    let dag = compile_or_panic((PYRIGHT_DAG, "src/v4/extdeps/typecheckers/pyright.dag"));
    assert!(
        dag.diagnostics().is_empty(),
        "pyright.dag: expected empty diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    for name in [
        "pyright_tool_id",
        "PyrightConfig",
        "pyright_profile_l1",
        "pyright_diag_report_return_type",
        "pyright_diagnostic_rules",
    ] {
        assert!(
            dag.declaration_by_name(name).is_some(),
            "pyright.dag should declare `{name}`"
        );
    }
}

#[test]
fn v4_extdeps_typecheckers_mypy_dag_compiles() {
    let dag = compile_or_panic((MYPY_DAG, "src/v4/extdeps/typecheckers/mypy.dag"));
    assert!(
        dag.diagnostics().is_empty(),
        "mypy.dag: expected empty diagnostics, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    for name in [
        "mypy_tool_id",
        "MypyConfig",
        "mypy_profile_l1",
        "mypy_diag_return_value",
        "mypy_diagnostic_codes",
    ] {
        assert!(
            dag.declaration_by_name(name).is_some(),
            "mypy.dag should declare `{name}`"
        );
    }
}
