//! ctrl#1476 B5 — positional-Conj fold_list-by-construction detection (emit path).
//!
//! Grammar-relation / serialize_source token sequences on the emit path must lower via
//! `grammar_relation_tokens_node` (fold_list_node named-cons spine), not positional-Conj
//! `fold_list` edge builders.

const EMIT_PATH_DAGS: &[(&str, &str)] = &[
    (
        "src/v4/extdeps/languages/cpp.dag",
        include_str!("../../../../v4/extdeps/languages/cpp.dag"),
    ),
    (
        "src/v4/extdeps/languages/dag.dag",
        include_str!("../../../../v4/extdeps/languages/dag.dag"),
    ),
    (
        "src/v4/extdeps/languages/ecmascript.dag",
        include_str!("../../../../v4/extdeps/languages/ecmascript.dag"),
    ),
    (
        "src/v4/extdeps/languages/go.dag",
        include_str!("../../../../v4/extdeps/languages/go.dag"),
    ),
    (
        "src/v4/extdeps/languages/java.dag",
        include_str!("../../../../v4/extdeps/languages/java.dag"),
    ),
    (
        "src/v4/extdeps/languages/kotlin.dag",
        include_str!("../../../../v4/extdeps/languages/kotlin.dag"),
    ),
    (
        "src/v4/extdeps/languages/python.dag",
        include_str!("../../../../v4/extdeps/languages/python.dag"),
    ),
    (
        "src/v4/extdeps/languages/rust.dag",
        include_str!("../../../../v4/extdeps/languages/rust.dag"),
    ),
    (
        "src/v4/extdeps/languages/swift.dag",
        include_str!("../../../../v4/extdeps/languages/swift.dag"),
    ),
    (
        "src/v4/extdeps/languages/typescript.dag",
        include_str!("../../../../v4/extdeps/languages/typescript.dag"),
    ),
    (
        "src/v4/extdeps/languages/wasm.dag",
        include_str!("../../../../v4/extdeps/languages/wasm.dag"),
    ),
];

fn code_lines(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Detection: red if emit-path language models reintroduce positional-Conj fold_list builders.
#[test]
fn emit_path_language_models_do_not_define_positional_conj_fold_list_builders() {
    let fn_forbidden = [
        "_relation_token_edges",
        "_positional_edges_from_token_classes",
        "_positional_token_edge",
        "_mvp1_positional_token_edge",
    ];
    let call_forbidden = [
        "positional_edges_from_token_classes()",
        "_relation_token_edges(",
    ];
    for (label, source) in EMIT_PATH_DAGS {
        let body = code_lines(source);
        for suffix in fn_forbidden {
            if body.contains(&format!("fn {suffix}")) {
                panic!(
                    "positional-Conj fold_list bypass in {label}.\n\
                     forbidden: fn ...{suffix}\n\
                     Route token sequences through grammar_relation_tokens_node (ctrl#1476 B5)."
                );
            }
        }
        for needle in call_forbidden {
            assert!(
                !body.contains(needle),
                "positional-Conj fold_list bypass in {label}.\n\
                 forbidden pattern: {needle}\n\
                 Route token sequences through grammar_relation_tokens_node (ctrl#1476 B5)."
            );
        }
    }
}

const GRAMMAR_DAG: &str = include_str!("../../../../v4/std/grammar.dag");

#[test]
fn grammar_dag_exports_relation_tokens_node_chokepoint() {
    assert!(
        GRAMMAR_DAG.contains("fn grammar_relation_tokens_node<T>("),
        "grammar.dag must own the B5 chokepoint helper"
    );
    assert!(
        GRAMMAR_DAG.contains("fold_list_node(xs: tokens, item_node: token_to_node)"),
        "grammar_relation_tokens_node must delegate to fold_list_node"
    );
    assert!(
        GRAMMAR_DAG.contains("feature:B-POSITIONAL-CONJ-FOLD-LIST-1"),
        "B5 model mark must be present on the chokepoint"
    );
}
