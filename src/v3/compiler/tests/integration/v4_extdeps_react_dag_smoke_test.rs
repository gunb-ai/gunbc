//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/frameworks/react.dag` —
//! T-4.7 React framework substrate must lower+infer with **zero** module
//! diagnostics (same 0-diag gate as `v4_extdeps_typescript_dag_smoke_test`).
//!
//! **Shape (E2 post A-fold-delete):** `v4_extdeps_react_dag_compiles` is the **0-diag**
//! gate; **5 B-class declaration-shape receipts** stay here as B-INTERIM host-AST
//! (operator 2026-06-07: no standalone guard files). **7 A-class receipts** migrated
//! to discriminating `.dag` witnesses in
//! `src/v4/test/claim/extdeps_react/structural_receipts.dag` (mutation-proven).
//!
//! **P5 receipt (INVARIANTS §P5(b)):** Explicit deferral ROADMAP.md § "Nine lanes" row
//! **T-PB-B** / `pb_rust_tests_outside_residual_zero` (ROADMAP.md:74). B-INTERIM
//! dissolution trigger: ctrl#1476 READ axis / type-decl reflection substrate
//! (sleek-carp-651); migrate each receipt to `.dag` intrinsic witnesses, then delete
//! this file + SG-0 census row (full E2 clean).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::TypeConnective;
use v3_compiler::CompileError;

const REACT_DAG: &str = include_str!("../../../../v4/extdeps/frameworks/react.dag");
const REACT_PATH: &str = "src/v4/extdeps/frameworks/react.dag";

/// Pinned `react@19.2.0` **Hooks index** roster in `ReactHookSite`: **18** built-in
/// `use*` APIs + **`CustomHook`** (= **19** arms). `use(resource)` is **not** modeled here
/// — see `ReactUseCallSite` (react.dev/use: not a Hook; placement rules differ).
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

// B-INTERIM consumer 4 (+3): UseResource ∉ ReactHookSite; ReactUseCallSite 1-arm UseResource.
// P5 deferral: ROADMAP T-PB-B (ROADMAP.md:74); TRIGGER: ctrl#1476 READ axis.
fn assert_use_resource_is_react_use_call_site_not_hook_site(dag: &v3_compiler::Dag) {
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
        "UseResource must not be a `ReactHookSite` arm — it is the `use` API, not a Hooks-index hook"
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
        "ReactUseCallSite should be a single-variant carrier for `use(resource)` in this slice"
    );
    assert_eq!(
        use_variants[0].label, "UseResource",
        "ReactUseCallSite should carry `UseResource` as the `use(resource)` arm"
    );
}

// B-INTERIM consumer 2: ReactHookSite 19-arm roster (count + labels).
// P5 deferral: ROADMAP T-PB-B (ROADMAP.md:74); TRIGGER: ctrl#1476 READ axis.
fn assert_react_hook_site_roster_matches_pin(dag: &v3_compiler::Dag) {
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
        "ReactHookSite must have exactly {} arms under the react@19.2.0 pin",
        EXPECTED_REACT_HOOK_SITE_ARMS.len()
    );
    for arm in EXPECTED_REACT_HOOK_SITE_ARMS {
        let n = variants.iter().filter(|v| v.label == *arm).count();
        assert_eq!(
            n, 1,
            "ReactHookSite must declare exactly one `{arm}` arm (pin fidelity); matched {n}"
        );
    }
}

// B-INTERIM consumer 1: ReactElement partition arm-set (Host|Composite|Fragment, no Text).
// P5 deferral: ROADMAP T-PB-B (ROADMAP.md:74); TRIGGER: ctrl#1476 READ axis.
// SOLE authority for partition (no standalone guard file; no .dag mirror witness).
fn assert_react_element_partition_is_create_element_return_only(dag: &v3_compiler::Dag) {
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
            "ReactElement should include `{expected}` (createElement-returned object partition)"
        );
    }
    assert!(
        !variants.iter().any(|v| v.label == "Text"),
        "primitive `Text` must not be a `ReactElement` arm — use `ReactCreateElementChild::Text`"
    );
    assert_eq!(
        variants.len(),
        3,
        "ReactElement should carry exactly Host | Composite | Fragment at this substrate layer"
    );
}

// B-INTERIM consumer 5: ReactCreateElementChild::Text record field-set (key absence).
// TRIGGER: ctrl#1476 record field-label projection on READ axis.
fn assert_react_create_element_child_text_has_no_element_key_field(dag: &v3_compiler::Dag) {
    let create_element_child = dag
        .declaration_by_name("ReactCreateElementChild")
        .expect("ReactCreateElementChild should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &create_element_child.connective else {
        panic!(
            "ReactCreateElementChild: expected coproduct (Disj), got {:?}",
            create_element_child.connective
        );
    };
    let text = variants
        .iter()
        .find(|v| v.label == "Text")
        .expect("ReactCreateElementChild should include a Text arm for primitive child values");
    let payload = dag.declaration(text.ty);
    let TypeConnective::Conj { children } = &payload.connective else {
        panic!(
            "Text arm: expected record (Conj) payload, got {:?}",
            payload.connective
        );
    };
    assert!(
        !children.iter().any(|f| f.label == "key"),
        "ReactCreateElementChild::Text must not declare `key` — primitive text is not a createElement-returned element object"
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

// B-INTERIM consumer 6: ReactContextBinding record field-set (invented-field absence).
// TRIGGER: ctrl#1476 record field-label projection on READ axis.
fn assert_react_context_binding_fields_match_create_context_surface(dag: &v3_compiler::Dag) {
    let binding = dag
        .declaration_by_name("ReactContextBinding")
        .expect("ReactContextBinding should exist after compiling react.dag");
    let TypeConnective::Conj { children } = &binding.connective else {
        panic!(
            "ReactContextBinding: expected record (Conj), got {:?}",
            binding.connective
        );
    };
    let labels: Vec<&str> = children.iter().map(|f| f.label.as_str()).collect();
    assert!(
        labels.contains(&"context_ref"),
        "ReactContextBinding must declare `context_ref` (single createContext return object); got {labels:?}"
    );
    assert!(
        labels.contains(&"default_value_ref"),
        "ReactContextBinding must declare `default_value_ref`; got {labels:?}"
    );
    assert!(
        !labels.contains(&"context_pair_ref"),
        "ReactContextBinding must not declare invented `context_pair_ref`; got {labels:?}"
    );
}

#[test]
fn v4_extdeps_react_dag_compiles() {
    let _dag = react_extdeps_dag_or_panic();
}

#[test]
fn v4_extdeps_react_dag_react_hook_site_roster_matches_pin() {
    assert_react_hook_site_roster_matches_pin(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_use_resource_is_react_use_call_site_not_hook_site() {
    assert_use_resource_is_react_use_call_site_not_hook_site(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_react_element_partition_excludes_primitive_text() {
    assert_react_element_partition_is_create_element_return_only(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_create_element_child_text_has_no_element_key() {
    assert_react_create_element_child_text_has_no_element_key_field(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_context_binding_matches_create_context_surface() {
    assert_react_context_binding_fields_match_create_context_surface(&react_extdeps_dag_or_panic());
}
