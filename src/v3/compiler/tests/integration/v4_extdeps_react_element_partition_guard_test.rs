//! **Layer:** integration
//!
//! Host-AST interim guard for `ReactElement` createElement-return partition
//! (`src/v4/extdeps/frameworks/react.dag`).
//!
//! **Host-test preservation justification (W1-W4 qualifying bar — B-INTERIM,
//! DISTINCT from E1 permanent type-absence):** expressible-in-principle; host-AST
//! until compiler type-decl reflection substrate lands; **TRIGGER:** migrate to
//! `.dag` witness when the reflection-substrate (ctrl#1476 programmatic-access
//! READ axis) exists. Named consumer: `extdeps_react_element_partition` (E2).
//!
//! Reads the **compiled type-table** (`compile_to_dag` → `ReactElement` Disj
//! variant labels), not a hand-built mirror list. Behavioral partition
//! discrimination on constructed values rides other E2 `.dag` witnesses; this
//! host guard owns declaration arm-set == {Host, Composite, Fragment} with no
//! primitive Text arm until substrate reflection is available.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::TypeConnective;

const REACT_DAG: &str = include_str!("../../../../v4/extdeps/frameworks/react.dag");
const REACT_PATH: &str = "src/v4/extdeps/frameworks/react.dag";

fn react_extdeps_dag_or_panic() -> v3_compiler::Dag {
    match compile_to_dag(REACT_DAG, REACT_PATH) {
        Ok(dag) => dag,
        Err(e) => panic!("{REACT_PATH}: compile_to_dag: {e:?}"),
    }
}

/// Variant labels from the compiled `ReactElement` coproduct declaration (type-table authority).
fn react_element_disj_variant_labels(dag: &v3_compiler::Dag) -> Vec<String> {
    let react_element = dag
        .declaration_by_name("ReactElement")
        .expect("ReactElement should exist after compiling react.dag");
    let TypeConnective::Disj { variants } = &react_element.connective else {
        panic!(
            "ReactElement: expected coproduct (Disj), got {:?}",
            react_element.connective
        );
    };
    variants.iter().map(|v| v.label.clone()).collect()
}

fn assert_react_element_partition_is_create_element_return_only(dag: &v3_compiler::Dag) {
    let labels = react_element_disj_variant_labels(dag);
    for expected in ["Host", "Composite", "Fragment"] {
        assert!(
            labels.iter().any(|label| label == expected),
            "ReactElement should include `{expected}` (createElement-returned object partition); \
             got {labels:?} from compiled declaration"
        );
    }
    assert!(
        !labels.iter().any(|label| label == "Text"),
        "primitive `Text` must not be a `ReactElement` arm — use `ReactCreateElementChild::Text`; \
         got {labels:?} from compiled declaration"
    );
    assert_eq!(
        labels.len(),
        3,
        "ReactElement should carry exactly Host | Composite | Fragment at this substrate layer; \
         got {labels:?} from compiled declaration"
    );
}

#[test]
fn v4_extdeps_react_dag_react_element_partition_is_create_element_return_only() {
    assert_react_element_partition_is_create_element_return_only(&react_extdeps_dag_or_panic());
}
