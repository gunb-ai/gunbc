//! **Layer:** integration
//!
//! Smoke `compile_to_dag` on `src/v4/extdeps/frameworks/react.dag` —
//! T-4.7 React framework substrate must lower+infer with **zero** module
//! diagnostics (same 0-diag gate as `v4_extdeps_typescript_dag_smoke_test`).
//!
//! **Shape:** `v4_extdeps_react_dag_compiles` is the single **0-diag** gate; sibling
//! `#[test]`s each assert one structural receipt on a fresh compile (TESTING.md §4),
//! including the pinned **19-arm `ReactHookSite`** label roster plus separate
//! **`ReactUseCallSite`** for `use(resource)` (not a Hook — react.dev/use).

use v3_compiler::compile_to_dag;
use v3_compiler::dag::Field;
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

/// Compiler-visible receipt: `UseMemo` / `UseCallback` carry **required**
/// `dependencies: List<ReactCrossDeclRef>` (design-r4 canvas; pinned call shape).
/// Omitted-deps arity is **not** constructible on those arms — only hooks that use
/// `ReactHookInlineDependenciesArgument` admit omit-vs-present (`useEffect`, …).
fn assert_use_memo_use_callback_dependencies_are_required_lists(dag: &v3_compiler::Dag) {
    let react_hook_site = dag
        .declaration_by_name("ReactHookSite")
        .expect("ReactHookSite should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &react_hook_site.connective else {
        panic!(
            "ReactHookSite: expected coproduct (Disj), got {:?}",
            react_hook_site.connective
        );
    };
    let list_decl = dag
        .declaration_by_name("List")
        .expect("List should resolve for `List<ReactCrossDeclRef>` fields");
    let cross_decl = dag
        .declaration_by_name("ReactCrossDeclRef")
        .expect("ReactCrossDeclRef should exist in this module");
    for arm in ["UseMemo", "UseCallback"] {
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
        let deps_field = children
            .iter()
            .find(|f| f.label == "dependencies")
            .unwrap_or_else(|| {
                panic!("{arm} payload should declare `dependencies`");
            });
        let deps_ty = dag.declaration(deps_field.ty);
        let TypeConnective::Instantiation {
            template,
            arguments,
        } = &deps_ty.connective
        else {
            panic!(
                "{arm}.dependencies must be `List<ReactCrossDeclRef>` (Instantiation), got {:?}",
                deps_ty.connective
            );
        };
        assert_eq!(
            *template,
            list_decl.id,
            "{arm}.dependencies must instantiate `List`, got template decl {:?}",
            dag.declaration(*template).name
        );
        assert_eq!(
            arguments.len(),
            1,
            "{arm}.dependencies must be unary List<…>"
        );
        assert_eq!(
            arguments[0].value, cross_decl.id,
            "{arm}.dependencies must be `List<ReactCrossDeclRef>`"
        );
    }
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
        let setup = children
            .iter()
            .find(|f| f.label == "setup_ref")
            .unwrap_or_else(|| {
                panic!("{arm} payload should declare `setup_ref` (required setup function)")
            });
        let setup_ty = dag.declaration(setup.ty);
        assert_eq!(
            setup_ty.id, cross_decl.id,
            "{arm}.setup_ref must be `ReactCrossDeclRef`, got {:?}",
            setup_ty.name
        );
    }
}

/// `use(resource)` is the separate **`use` API** (react.dev/reference/react/use), not a
/// Hooks-index hook: unlike Hooks, it may appear in loops/conditionals. Substrate must not
/// fold it into `ReactHookSite` (Rules-of-Hooks / P2 placement).
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
    let opt_key = dag
        .declaration_by_name("ReactOptKey")
        .expect("ReactOptKey should exist in this module");
    let opt_ref = dag
        .declaration_by_name("ReactOptRef")
        .expect("ReactOptRef should exist in this module");
    let key_field = children
        .iter()
        .find(|f| f.label == "key")
        .expect("ReactHostElement should declare `key` (createElement lift)");
    assert_eq!(
        dag.declaration(key_field.ty).id,
        opt_key.id,
        "ReactHostElement.key must be `ReactOptKey`, got {:?}",
        dag.declaration(key_field.ty).name
    );
    let ref_field = children
        .iter()
        .find(|f| f.label == "ref")
        .expect("ReactHostElement should declare `ref` (createElement lift)");
    assert_eq!(
        dag.declaration(ref_field.ty).id,
        opt_ref.id,
        "ReactHostElement.ref must be `ReactOptRef`, got {:?}",
        dag.declaration(ref_field.ty).name
    );
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
    let opt_key = dag
        .declaration_by_name("ReactOptKey")
        .expect("ReactOptKey should exist in this module");
    let opt_ref = dag
        .declaration_by_name("ReactOptRef")
        .expect("ReactOptRef should exist in this module");
    let key_field = children
        .iter()
        .find(|f| f.label == "key")
        .expect("ReactCompositeElement should declare `key` (createElement lift)");
    assert_eq!(
        dag.declaration(key_field.ty).id,
        opt_key.id,
        "ReactCompositeElement.key must be `ReactOptKey`, got {:?}",
        dag.declaration(key_field.ty).name
    );
    let ref_field = children
        .iter()
        .find(|f| f.label == "ref")
        .expect("ReactCompositeElement should declare `ref` (createElement lift)");
    assert_eq!(
        dag.declaration(ref_field.ty).id,
        opt_ref.id,
        "ReactCompositeElement.ref must be `ReactOptRef`, got {:?}",
        dag.declaration(ref_field.ty).name
    );
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
    let opt_key = dag
        .declaration_by_name("ReactOptKey")
        .expect("ReactOptKey should exist in this module");
    assert_eq!(
        key_ty.id, opt_key.id,
        "Fragment.key must be `ReactOptKey`, got {:?}",
        key_ty.name
    );
}

fn assert_children_field_is_list_create_element_child(
    dag: &v3_compiler::Dag,
    record_name: &str,
    conj_children: &[Field],
) {
    let list_decl = dag
        .declaration_by_name("List")
        .expect("List should resolve for `List<ReactCreateElementChild>` fields");
    let create_element_child = dag
        .declaration_by_name("ReactCreateElementChild")
        .expect("ReactCreateElementChild should exist in this module");
    let ch_field = conj_children
        .iter()
        .find(|f| f.label == "children")
        .unwrap_or_else(|| {
            panic!("{record_name} should declare `children` (createElement child list)")
        });
    let ch_ty = dag.declaration(ch_field.ty);
    let TypeConnective::Instantiation {
        template,
        arguments,
    } = &ch_ty.connective
    else {
        panic!(
            "{record_name}.children must be `List<ReactCreateElementChild>` (Instantiation), got {:?}",
            ch_ty.connective
        );
    };
    assert_eq!(
        *template,
        list_decl.id,
        "{record_name}.children must instantiate `List`, got template decl {:?}",
        dag.declaration(*template).name
    );
    assert_eq!(
        arguments.len(),
        1,
        "{record_name}.children must be unary List<…>"
    );
    assert_eq!(
        arguments[0].value, create_element_child.id,
        "{record_name}.children must be `List<ReactCreateElementChild>`"
    );
}

fn assert_react_element_records_children_are_create_element_child_lists(dag: &v3_compiler::Dag) {
    for (record_name, preview) in [
        ("ReactHostElement", "host"),
        ("ReactCompositeElement", "composite"),
    ] {
        let decl = dag
            .declaration_by_name(record_name)
            .unwrap_or_else(|| panic!("{record_name} should exist after compiling react.dag"));
        let TypeConnective::Conj { children } = &decl.connective else {
            panic!(
                "{record_name}: expected record (Conj), got {:?} ({preview})",
                decl.connective
            );
        };
        assert_children_field_is_list_create_element_child(dag, record_name, children);
    }

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
    assert_children_field_is_list_create_element_child(dag, "Fragment", children);
}

// ReactElement partition arm-set (Host|Composite|Fragment, no Text): extracted to
// v4_extdeps_react_element_partition_guard_test.rs (B-INTERIM host-AST; see SG-0 census row).

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

fn assert_react_create_element_child_element_arm_wraps_react_element(dag: &v3_compiler::Dag) {
    let create_element_child = dag
        .declaration_by_name("ReactCreateElementChild")
        .expect("ReactCreateElementChild should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &create_element_child.connective else {
        panic!(
            "ReactCreateElementChild: expected coproduct (Disj), got {:?}",
            create_element_child.connective
        );
    };
    let element_arm = variants
        .iter()
        .find(|v| v.label == "Element")
        .expect("ReactCreateElementChild should include an Element arm wrapping `ReactElement`");
    let payload = dag.declaration(element_arm.ty);
    let TypeConnective::Conj { children } = &payload.connective else {
        panic!(
            "Element arm: expected record (Conj) payload, got {:?}",
            payload.connective
        );
    };
    let element_field = children
        .iter()
        .find(|f| f.label == "element")
        .expect("Element payload should declare `element`");
    let element_ty = dag.declaration(element_field.ty);
    let react_element_decl = dag
        .declaration_by_name("ReactElement")
        .expect("ReactElement should exist after compiling react.dag");
    assert_eq!(
        element_ty.id, react_element_decl.id,
        "ReactCreateElementChild::Element.element must name `ReactElement`, got {:?}",
        element_ty.name
    );
}

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
fn v4_extdeps_react_dag_use_memo_use_callback_dependencies_are_required_lists() {
    assert_use_memo_use_callback_dependencies_are_required_lists(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_effect_hooks_require_setup_ref() {
    assert_effect_hook_arms_require_setup_ref(&react_extdeps_dag_or_panic());
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
fn v4_extdeps_react_dag_host_element_declares_key_and_ref() {
    assert_react_host_element_has_key_and_ref_fields(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_composite_element_declares_key_and_ref() {
    assert_react_composite_element_has_key_and_ref_fields(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_fragment_arm_declares_key() {
    assert_react_element_fragment_has_key_field(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_element_children_are_list_create_element_child() {
    assert_react_element_records_children_are_create_element_child_lists(
        &react_extdeps_dag_or_panic(),
    );
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
fn v4_extdeps_react_dag_create_element_child_element_arm_wraps_react_element() {
    assert_react_create_element_child_element_arm_wraps_react_element(&react_extdeps_dag_or_panic());
}

#[test]
fn v4_extdeps_react_dag_context_binding_matches_create_context_surface() {
    assert_react_context_binding_fields_match_create_context_surface(&react_extdeps_dag_or_panic());
}
