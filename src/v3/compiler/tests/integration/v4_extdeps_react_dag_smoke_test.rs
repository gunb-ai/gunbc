//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/frameworks/react.dag` —
//! T-4.7 React framework substrate must lower+infer with **zero** module
//! diagnostics (same 0-diag gate as `v4_extdeps_typescript_dag_smoke_test`).
//!
//! **Shape (E2 B-DELETE exemplar):** `v4_extdeps_react_dag_compiles` is the **0-diag**
//! gate; **7 A-class receipts** are discriminating `.dag` witnesses in
//! `src/v4/test/claim/extdeps_react/structural_receipts.dag` (mutation-proven).
//! **3 EXTERNAL-ORACLE** receipts (react.dev-cited React-spec fidelity) stay here;
//! **2 dual-representation** declaration-shape receipts deleted (no external oracle).
//!
//! **P5 receipt (INVARIANTS §P5(b)):** Explicit deferral ROADMAP.md § "Nine lanes" row
//! **T-PB-B** / `pb_rust_tests_outside_residual_zero` (ROADMAP.md:74). Dissolves when
//! oracle receipts migrate to cited `.dag` TestClaim execution or generated harness.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::TypeConnective;
use v3_compiler::CompileError;

const REACT_DAG: &str = include_str!("../../../../v4/extdeps/frameworks/react.dag");
const REACT_PATH: &str = "src/v4/extdeps/frameworks/react.dag";

/// EXTERNAL-ORACLE: Built-in Hooks index @ react@19.2.0 pin.
/// Authority: https://react.dev/reference/react/hooks (not our `react.dag` decl labels).
const REACT_DEV_HOOKS_ORACLE: &str = "https://react.dev/reference/react/hooks";

/// Hooks roster transcribed from `REACT_DEV_HOOKS_ORACLE` + `CustomHook` convention.
/// `use(resource)` is excluded — oracle: https://react.dev/reference/react/use .
const EXPECTED_REACT_HOOK_SITE_ARMS: &[&str] = &[
    "UseState",
    "UseReducer",
    "UseContext",
    "UseRef",
    "UseImperativeHandle",
    "UseEffect",
    "UseLayoutEffect",
    "UseInsertionEffect",
    "UseEffectEvent",
    "UseMemo",
    "UseCallback",
    "UseTransition",
    "UseDeferredValue",
    "UseId",
    "UseSyncExternalStore",
    "UseDebugValue",
    "UseActionState",
    "UseOptimistic",
    "CustomHook",
];

/// EXTERNAL-ORACLE: `use` API is not a Hook.
/// Authority: https://react.dev/reference/react/use
const REACT_DEV_USE_ORACLE: &str = "https://react.dev/reference/react/use";

/// EXTERNAL-ORACLE: `createElement` returns element objects; primitive text is a child.
/// Authority: https://react.dev/reference/react/createElement
const REACT_DEV_CREATE_ELEMENT_ORACLE: &str = "https://react.dev/reference/react/createElement";

/// Panics unless `react.dag` compiles with **zero** module diagnostics.
fn react_extdeps_dag_or_panic() -> v3_compiler::Dag {
    match compile_to_dag(REACT_DAG, REACT_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{REACT_PATH}: expected empty diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{REACT_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{REACT_PATH}: {other:?}"),
    }
}

/// EXTERNAL-ORACLE: `use` is not a Hook — model must place `use(resource)` on `ReactUseCallSite`.
fn assert_react_dev_use_is_not_hook_site_arm(dag: &v3_compiler::Dag) {
    let hook_site = dag
        .declaration_by_name("ReactHookSite")
        .expect("ReactHookSite should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &hook_site.connective else {
        panic!(
            "ReactHookSite: expected coproduct (Disj), got {:?}",
            hook_site.connective
        );
    };
    assert!(
        !variants.iter().any(|v| v.label == "UseResource"),
        "{REACT_DEV_USE_ORACLE}: `use` is not a Hook — model must not declare a \
         `UseResource` arm on `ReactHookSite`"
    );

    let use_call_site = dag
        .declaration_by_name("ReactUseCallSite")
        .expect("ReactUseCallSite should exist after compiling react.dag");
    let TypeConnective::Disj {
        variants: use_variants,
    } = &use_call_site.connective
    else {
        panic!(
            "ReactUseCallSite: expected coproduct (Disj), got {:?}",
            use_call_site.connective
        );
    };
    assert_eq!(
        use_variants.len(),
        1,
        "{REACT_DEV_USE_ORACLE}: `use(resource)` must be modeled on `ReactUseCallSite` \
         (single arm in this slice)"
    );
    assert_eq!(
        use_variants[0].label, "UseResource",
        "{REACT_DEV_USE_ORACLE}: `ReactUseCallSite` must carry `UseResource` for \
         `use(resource)`"
    );
}

/// EXTERNAL-ORACLE: Built-in Hooks index roster @ react@19.2.0 pin.
fn assert_react_dev_hooks_index_roster(dag: &v3_compiler::Dag) {
    let hook_site = dag
        .declaration_by_name("ReactHookSite")
        .expect("ReactHookSite should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &hook_site.connective else {
        panic!(
            "ReactHookSite: expected coproduct (Disj), got {:?}",
            hook_site.connective
        );
    };
    assert_eq!(
        variants.len(),
        EXPECTED_REACT_HOOK_SITE_ARMS.len(),
        "{REACT_DEV_HOOKS_ORACLE}: model must expose exactly {} built-in hook APIs \
         (+ CustomHook) under the react@19.2.0 pin",
        EXPECTED_REACT_HOOK_SITE_ARMS.len()
    );
    for arm in EXPECTED_REACT_HOOK_SITE_ARMS {
        let n = variants.iter().filter(|v| v.label == *arm).count();
        assert_eq!(
            n, 1,
            "{REACT_DEV_HOOKS_ORACLE}: hooks index lists `{arm}` — model must declare \
             exactly one matching arm; matched {n}"
        );
    }
}

/// EXTERNAL-ORACLE: `createElement` returns element objects (Host | Composite | Fragment).
/// Primitive text is modeled as `ReactCreateElementChild::Text`, not a `ReactElement` arm.
fn assert_react_dev_create_element_return_partition(dag: &v3_compiler::Dag) {
    let react_element = dag
        .declaration_by_name("ReactElement")
        .expect("ReactElement should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &react_element.connective else {
        panic!(
            "ReactElement: expected coproduct (Disj), got {:?}",
            react_element.connective
        );
    };
    for expected in ["Host", "Composite", "Fragment"] {
        assert!(
            variants.iter().any(|v| v.label == expected),
            "{REACT_DEV_CREATE_ELEMENT_ORACLE}: createElement-returned objects include \
             `{expected}` — model partition must expose this arm"
        );
    }
    assert!(
        !variants.iter().any(|v| v.label == "Text"),
        "{REACT_DEV_CREATE_ELEMENT_ORACLE}: primitive text is a child value, not a \
         createElement-returned element — model must not declare `Text` on `ReactElement`"
    );
    assert_eq!(
        variants.len(),
        3,
        "{REACT_DEV_CREATE_ELEMENT_ORACLE}: element partition is Host | Composite | Fragment \
         at this substrate layer"
    );
}

#[test]
fn v4_extdeps_react_dag_compiles() {
    let _dag = react_extdeps_dag_or_panic();
}

#[test]
fn v4_extdeps_react_dag_react_dev_hooks_index_roster() {
    assert_react_dev_hooks_index_roster(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_react_dev_use_is_not_hook_site_arm() {
    assert_react_dev_use_is_not_hook_site_arm(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_react_dev_create_element_return_partition() {
    assert_react_dev_create_element_return_partition(&react_extdeps_dag_or_panic());
}
