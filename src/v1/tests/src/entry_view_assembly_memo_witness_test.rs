//! Memo key completeness control for PR B `rewired_modules_memo`.
//!
//! `global_bare_variant_base_memo` was deleted after the representative 50-entry cohort
//! showed 1 cross-grain hit in 106 builds (smart-badger-549 item 2) — closure/root coarse
//! memos already retain the win; the inner digest recomputed eligibility on every miss.

use im::HashMap;
use std::rc::Rc;

use v1_compiler::cli_run::rewire_semantic_input_identity_for_test;
use v1_compiler::v1_compiler_compile::{front_end_sources, SourceFile};
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{build_newline_index, has_child_named, NewlineIndex, Node};

fn module_node(path: &str, content: &str) -> Rc<Node> {
    let sources = Rc::new(
        vec![Rc::new(SourceFile {
            path: path.to_string(),
            content: content.to_string(),
        })]
        .into_iter()
        .collect::<im::Vector<_>>(),
    );
    let frontend = front_end_sources(sources);
    frontend
        .graph
        .clone()
        .expect("graph")
        .modules
        .iter()
        .next()
        .expect("single module")
        .module
        .clone()
}

fn source_indices_for(path: &str, content: &str) -> Rc<HashMap<String, Rc<NewlineIndex>>> {
    Rc::new(HashMap::from_iter([(
        path.to_string(),
        build_newline_index(path.to_string(), content.to_string()),
    )]))
}

/// `rewire_semantic_input_identity` omits `source_indices` because content keys already pin
/// the per-file source hash that determines each `NewlineIndex`.
#[test]
fn rewire_semantic_input_identity_ignores_source_indices_path_representation() {
    let content_key = format!(
        "probe.mod\u{1f}{}",
        v1_rt::atom_identity_hash("module probe.mod\n".to_string())
    );
    let identity_a = rewire_semantic_input_identity_for_test(&[content_key.clone()]);
    let identity_b = rewire_semantic_input_identity_for_test(&[content_key]);
    assert_eq!(identity_a, identity_b);

    let other_key = format!(
        "probe.mod\u{1f}{}",
        v1_rt::atom_identity_hash("module probe.mod\n// touched\n".to_string())
    );
    let identity_other = rewire_semantic_input_identity_for_test(&[other_key]);
    assert_ne!(
        identity_a, identity_other,
        "content-key vector is the rewire memo subject — source_indices is derived, not independent"
    );

    let si_a = source_indices_for("/abs/probe/mod.dag", "module probe.mod\n");
    let si_b = source_indices_for("probe/mod.dag", "module probe.mod\n");
    let node = module_node("probe/mod.dag", "module probe.mod\n");
    assert!(
        has_child_named(node.clone(), "probe".to_string(), si_a.clone())
            == has_child_named(node.clone(), "probe".to_string(), si_b.clone()),
        "newline index path spelling must not change has_child_named for the same content"
    );
}
