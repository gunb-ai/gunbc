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

/// react.dev `useEffect(setup, …)` / `useLayoutEffect` / `useInsertionEffect` — **setup** is required;
/// cleanup is optional and modeled separately on-arm (`ReactEffectCleanupSite`).
fn assert_effect_hook_arms_require_setup_ref(dag: &v3_compiler::Dag) {
    let react_hook_site = dag
        .declaration_by_name("ReactHookSite")
        .expect("ReactHookSite should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &react_hook_site.connective else {
        panic!(
            "ReactHookSite: expected coproduct (Disj), got {:?}",
            react_hook_site.connective
        );
    };
    let cross_decl = dag
        .declaration_by_name("ReactCrossDeclRef")
        .expect("ReactCrossDeclRef should exist in this module");
    for arm in ["UseEffect", "UseLayoutEffect", "UseInsertionEffect"] {
        let v = variants
            .iter()
            .find(|v| v.label == arm)
            .unwrap_or_else(|| panic!("ReactHookSite should include {arm}"));
        let payload = dag.declaration(v.ty);
        let TypeConnective::Conj { children } = &payload.connective else {
            panic!(
                "{arm} arm: expected record (Conj) payload, got {:?}",
                payload.connective
            );
        };
        let setup = children.iter().find(|f| f.label == "setup_ref").unwrap_or_else(|| {
            panic!("{arm} payload should declare `setup_ref` (required setup function)")
        });
        let setup_ty = dag.declaration(setup.ty);
        assert_eq!(
            setup_ty.id,
            cross_decl.id,
            "{arm}.setup_ref must be `ReactCrossDeclRef`, got {:?}",
            setup_ty.name
        );
    }
}

/// createElement lifts `key` / `ref` out of `props` onto the element object (`element.key`,
/// `element.ref` per react.dev); host elements must carry them as fields, not only inside `props`.
fn assert_react_host_element_has_key_and_ref_fields(dag: &v3_compiler::Dag) {
    let host_el = dag
        .declaration_by_name("ReactHostElement")
        .expect("ReactHostElement should exist after compiling react.dag");
    let TypeConnective::Conj { children } = &host_el.connective else {
        panic!(
            "ReactHostElement: expected record (Conj), got {:?}",
            host_el.connective
        );
    };
    let opt_ref = dag
        .declaration_by_name("ReactOptRef")
        .expect("ReactOptRef should exist in this module");
    for label in ["key", "ref"] {
        let field = children
            .iter()
            .find(|f| f.label == label)
            .unwrap_or_else(|| {
                panic!("ReactHostElement should declare `{label}` (createElement lift)")
            });
        let ty = dag.declaration(field.ty);
        assert_eq!(
            ty.id, opt_ref.id,
            "ReactHostElement.{label} must be `ReactOptRef`, got {:?}",
            ty.name
        );
    }
}

fn assert_react_composite_element_has_key_and_ref_fields(dag: &v3_compiler::Dag) {
    let composite = dag
        .declaration_by_name("ReactCompositeElement")
        .expect("ReactCompositeElement should exist after compiling react.dag");
    let TypeConnective::Conj { children } = &composite.connective else {
        panic!(
            "ReactCompositeElement: expected record (Conj), got {:?}",
            composite.connective
        );
    };
    let opt_ref = dag
        .declaration_by_name("ReactOptRef")
        .expect("ReactOptRef should exist in this module");
    for label in ["key", "ref"] {
        let field = children
            .iter()
            .find(|f| f.label == label)
            .unwrap_or_else(|| {
                panic!("ReactCompositeElement should declare `{label}` (createElement lift)")
            });
        let ty = dag.declaration(field.ty);
        assert_eq!(
            ty.id, opt_ref.id,
            "ReactCompositeElement.{label} must be `ReactOptRef`, got {:?}",
            ty.name
        );
    }
}

fn assert_react_element_fragment_has_key_field(dag: &v3_compiler::Dag) {
    let react_element = dag
        .declaration_by_name("ReactElement")
        .expect("ReactElement should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &react_element.connective else {
        panic!(
            "ReactElement: expected coproduct (Disj), got {:?}",
            react_element.connective
        );
    };
    let fragment = variants
        .iter()
        .find(|v| v.label == "Fragment")
        .expect("ReactElement should include a Fragment arm");
    let payload = dag.declaration(fragment.ty);
    let TypeConnective::Conj { children } = &payload.connective else {
        panic!(
            "Fragment arm: expected record (Conj) payload, got {:?}",
            payload.connective
        );
    };
    let key_field = children
        .iter()
        .find(|f| f.label == "key")
        .expect("Fragment payload should declare `key` (keyed fragments)");
    let key_ty = dag.declaration(key_field.ty);
    let opt_ref = dag
        .declaration_by_name("ReactOptRef")
        .expect("ReactOptRef should exist in this module");
    assert_eq!(
        key_ty.id, opt_ref.id,
        "Fragment.key must be `ReactOptRef`, got {:?}",
        key_ty.name
    );
}

fn assert_react_element_text_has_no_element_key_field(dag: &v3_compiler::Dag) {
    let react_element = dag
        .declaration_by_name("ReactElement")
        .expect("ReactElement should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &react_element.connective else {
        panic!(
            "ReactElement: expected coproduct (Disj), got {:?}",
            react_element.connective
        );
    };
    let text = variants
        .iter()
        .find(|v| v.label == "Text")
        .expect("ReactElement should include a Text arm");
    let payload = dag.declaration(text.ty);
    let TypeConnective::Conj { children } = &payload.connective else {
        panic!(
            "Text arm: expected record (Conj) payload, got {:?}",
            payload.connective
        );
    };
    assert!(
        !children.iter().any(|f| f.label == "key"),
        "Text arm must not declare `key` — primitive text children are not createElement-returned element objects"
    );
    let value = children
        .iter()
        .find(|f| f.label == "text_value_ref")
        .expect("Text payload should declare `text_value_ref`");
    let value_ty = dag.declaration(value.ty);
    let cross_decl = dag
        .declaration_by_name("ReactCrossDeclRef")
        .expect("ReactCrossDeclRef should exist in this module");
    assert_eq!(
        value_ty.id, cross_decl.id,
        "Text.text_value_ref must be `ReactCrossDeclRef`, got {:?}",
        value_ty.name
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
            assert_effect_hook_arms_require_setup_ref(&dag);
            assert_react_host_element_has_key_and_ref_fields(&dag);
            assert_react_composite_element_has_key_and_ref_fields(&dag);
            assert_react_element_fragment_has_key_field(&dag);
            assert_react_element_text_has_no_element_key_field(&dag);
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{REACT_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{REACT_PATH}: {other:?}"),
    }
}
