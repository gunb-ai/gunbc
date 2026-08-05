//! Memo key completeness controls for PR B entry-view assembly.

use im::HashMap;
use std::rc::Rc;

use v1_compiler::cli_run::rewire_semantic_input_identity_for_test;
use v1_compiler::resolved_graph_cache::closure_content_digest;
use v1_compiler::v1_compiler_compile::{front_end_sources, normalize_graph, SourceFile};
use v1_compiler::v1_compiler_infer::{build_global_bare_census, build_global_bare_variant_locals};
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{
    authored_name_at, build_newline_index, has_child_named, NewlineIndex, Node,
};

const DEFINER_SINGLE_LINE: &str = r#"module probe.def

type ProbeCurrency = ProbeEur | ProbeUsd
"#;

const DEFINER_MULTILINE: &str = r#"module probe.def

type ProbeCurrency =
    ProbeEur
  | ProbeUsd
"#;

fn src(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })
}

fn module_node(path: &str, content: &str) -> Rc<Node> {
    let sources = Rc::new(
        vec![src(path, content)]
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

fn variant_base_for(
    content: &str,
) -> Rc<HashMap<String, Rc<v1_compiler::v1_compiler_infer_env::TypeBinding>>> {
    let sources = Rc::new(
        vec![src("dag/probe_def.dag", content)]
            .into_iter()
            .collect::<im::Vector<_>>(),
    );
    let frontend = front_end_sources(sources);
    let graph = frontend.graph.clone().expect("graph");
    let source_indices = frontend.newline_indices.iter().cloned().fold(
        v1_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
        |acc, si| v1_rt::rc_map_insert(acc, si.file.clone(), si),
    );
    let norm = normalize_graph(graph, source_indices.clone());
    let global_bare = build_global_bare_census(norm.graph.modules.clone(), source_indices.clone());
    build_global_bare_variant_locals(global_bare, source_indices)
}

/// RED control: owner-relevant closure content must change `closure_content_digest`
/// even when variant eligibility is unchanged — the coarse cache keys must not collide.
#[test]
fn closure_content_digest_distinguishes_owner_fact_for_same_eligibility() {
    let sources_a = vec![src("dag/probe_def.dag", DEFINER_SINGLE_LINE)];
    let sources_b = vec![src("dag/probe_def.dag", DEFINER_MULTILINE)];
    let digest_a = closure_content_digest(&sources_a);
    let digest_b = closure_content_digest(&sources_b);
    assert_ne!(
        digest_a, digest_b,
        "closure_content_digest must differ when owner-bearing source content differs"
    );

    let base_a = variant_base_for(DEFINER_SINGLE_LINE);
    let base_b = variant_base_for(DEFINER_MULTILINE);
    assert!(
        base_a.contains_key("ProbeEur") && base_b.contains_key("ProbeEur"),
        "both layouts must keep ProbeEur eligible"
    );
    let owner_a = base_a.get("ProbeEur").expect("ProbeEur").resolved.clone();
    let owner_b = base_b.get("ProbeEur").expect("ProbeEur").resolved.clone();
    assert_ne!(
        (
            owner_a.span.start,
            owner_a.span.end,
            authored_name_at(
                source_indices_for("dag/probe_def.dag", DEFINER_SINGLE_LINE),
                owner_a
            )
        ),
        (
            owner_b.span.start,
            owner_b.span.end,
            authored_name_at(
                source_indices_for("dag/probe_def.dag", DEFINER_MULTILINE),
                owner_b
            )
        ),
        "owner facts differ even though eligibility matches"
    );
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
