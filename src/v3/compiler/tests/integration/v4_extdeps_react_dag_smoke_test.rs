//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/frameworks/react.dag` —
//! T-4.7 React framework substrate must lower+infer with **zero** module
//! diagnostics (same 0-diag gate as `v4_extdeps_typescript_dag_smoke_test`).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::TypeConnective;
use v3_compiler::CompileError;

const REACT_DAG: &str = include_str!("../../../../v4/extdeps/frameworks/react.dag");
const REACT_PATH: &str = "src/v4/extdeps/frameworks/react.dag";

/// Compiler-visible receipt: `UseMemo` carries `dependencies: ReactHookInlineDependenciesArgument`
/// (omit-vs-present), not a bare `List<ReactCrossDeclRef>` that would conflate omitted arity with `[]`.
fn assert_use_memo_dependencies_field_is_inline_deps_argument(dag: &v3_compiler::Dag) {
    let react_hook_site = dag
        .declaration_by_name("ReactHookSite")
        .expect("ReactHookSite should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &react_hook_site.connective else {
        panic!(
            "ReactHookSite: expected coproduct (Disj), got {:?}",
            react_hook_site.connective
        );
    };
    let use_memo = variants
        .iter()
        .find(|v| v.label == "UseMemo")
        .expect("ReactHookSite should include a UseMemo arm");
    let payload = dag.declaration(use_memo.ty);
    let TypeConnective::Conj { children } = &payload.connective else {
        panic!(
            "UseMemo arm: expected record (Conj) payload, got {:?}",
            payload.connective
        );
    };
    let deps_field = children
        .iter()
        .find(|f| f.label == "dependencies")
        .expect("UseMemo payload should declare a `dependencies` field");
    let deps_ty = dag.declaration(deps_field.ty);
    let inline_arg = dag
        .declaration_by_name("ReactHookInlineDependenciesArgument")
        .expect("ReactHookInlineDependenciesArgument should exist in this module");
    assert_eq!(
        deps_ty.id, inline_arg.id,
        "UseMemo.dependencies must name `ReactHookInlineDependenciesArgument` (same DeclarationId \
         as the top-level sum), got decl name={:?} id={:?} vs inline_arg id={:?}",
        deps_ty.name, deps_ty.id, inline_arg.id
    );
}

#[test]
fn v4_extdeps_react_dag_compiles() {
    match compile_to_dag(REACT_DAG, REACT_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{REACT_PATH}: expected empty diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            assert_use_memo_dependencies_field_is_inline_deps_argument(&dag);
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{REACT_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{REACT_PATH}: {other:?}"),
    }
}
