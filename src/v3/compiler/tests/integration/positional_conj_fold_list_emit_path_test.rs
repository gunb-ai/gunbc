//! ctrl#1476 B5 — positional-Conj fold_list-by-construction detection (emit path).
//!
//! Grammar-relation / serialize_source token sequences on the emit path must lower via
//! `grammar_relation_tokens_node` (fold_list_node named-cons spine), not positional-Conj
//! `fold_list` edge builders.
//!
//! **P5 receipt (INVARIANTS.md §P5 Mechanism (b) — SG-0 `EXPECTED_HAND_AUTHORED_TEST`):**
//! explicit deferral to **ROADMAP.md** `### Nine lanes` row **T-PB-B** /
//! `pb_rust_tests_outside_residual_zero`; dissolves when modeled `TestClaim` exercises
//! emit-path grammar-relation token encode without this hand-Rust substring/parse ratchet.

use v3_compiler::parse_for_test;
use v3_compiler::tokenize_for_test;

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

fn defines_fn_with_suffix(body: &str, suffix: &str) -> bool {
    body.lines().any(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with("fn ") {
            return false;
        }
        let rest = trimmed.strip_prefix("fn ").unwrap_or("");
        rest.split(['(', '<', ' '])
            .next()
            .is_some_and(|name| name.ends_with(suffix))
    })
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
            if defines_fn_with_suffix(&body, suffix) {
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
const GRAMMAR_PATH: &str = "src/v4/std/grammar.dag";

/// Positive-side ratchet: emit-path language models must tokenize and parse (catches import-list
/// syntax regressions invisible to substring-only detection).
#[test]
fn emit_path_language_models_tokenize_and_parse() {
    for (path, source) in EMIT_PATH_DAGS {
        let tokens = tokenize_for_test(source, path)
            .unwrap_or_else(|e| panic!("{path}: tokenize failed: {e:?}"));
        parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse failed: {e:?}"));
    }
    let tokens = tokenize_for_test(GRAMMAR_DAG, GRAMMAR_PATH)
        .unwrap_or_else(|e| panic!("{GRAMMAR_PATH}: tokenize failed: {e:?}"));
    parse_for_test(&tokens, GRAMMAR_PATH)
        .unwrap_or_else(|e| panic!("{GRAMMAR_PATH}: parse failed: {e:?}"));
}

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
