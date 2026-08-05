//! Memo key completeness controls for PR B entry-view assembly (smart-badger-549 review).
//!
//! `global_bare_variant_base_memo` and `rewired_modules_memo` must not reuse results across
//! inputs that `build_global_bare_variant_locals` / the rewire passes would treat differently.

use im::HashMap;
use std::rc::Rc;

use v1_compiler::cli_run::{
    global_bare_semantic_digest_for_test, rewire_semantic_input_identity_for_test,
};
use v1_compiler::std_induction::SubValueRelation;
use v1_compiler::v1_compiler_compile::{front_end_sources, normalize_graph, SourceFile};
use v1_compiler::v1_compiler_infer::{build_global_bare_census, build_global_bare_variant_locals};
use v1_compiler::v1_compiler_infer_env::{GlobalBareCandidate, GlobalBareLookupState, TypeBinding};
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{
    build_newline_index, has_child_named, Connective, NewlineIndex, Node,
};

const DEFINER: &str = r#"module probe.def

type ProbeCurrency =
    ProbeEur
  | ProbeUsd

fn probe_minor_unit(c: ProbeCurrency) -> Int {
  match c {
    ProbeEur => 2
    ProbeUsd => 2
  }
}
"#;

fn src(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })
}

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

/// RED control: digest must distinguish the two predicates `build_global_bare_variant_locals`
/// actually branches on for the same `(name, module_path)` pair.
#[test]
fn global_bare_variant_base_digest_distinguishes_variant_eligibility() {
    let sources = Rc::new(
        vec![src("dag/probe_def.dag", DEFINER)]
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

    let eligible_state = global_bare
        .get("ProbeEur")
        .expect("census includes ProbeEur")
        .clone();
    let GlobalBareLookupState::GlobalBareUniqueBinding {
        module_path,
        binding,
    } = eligible_state.as_ref()
    else {
        panic!("ProbeEur must be a unique global-bare binding");
    };
    let binding_rc = binding.clone();
    let mut ineligible_binding = (*binding_rc).clone();
    let mut ineligible_owner = (*binding_rc.resolved).clone();
    ineligible_owner.connective = Connective::NoConnective;
    ineligible_binding.resolved = Rc::new(ineligible_owner);
    let ineligible_state = Rc::new(GlobalBareLookupState::GlobalBareUniqueBinding {
        module_path: module_path.clone(),
        binding: Rc::new(ineligible_binding),
    });

    let mut eligible_map: HashMap<String, Rc<GlobalBareLookupState>> = HashMap::new();
    eligible_map.insert("ProbeEur".to_string(), eligible_state);
    let mut ineligible_map = eligible_map.clone();
    ineligible_map.insert("ProbeEur".to_string(), ineligible_state);

    let digest_eligible =
        global_bare_semantic_digest_for_test(Rc::new(eligible_map.clone()), source_indices.clone());
    let digest_ineligible = global_bare_semantic_digest_for_test(
        Rc::new(ineligible_map.clone()),
        source_indices.clone(),
    );
    assert_ne!(
        digest_eligible, digest_ineligible,
        "digest must change when variant-eligibility changes for the same module_path"
    );

    let base_eligible =
        build_global_bare_variant_locals(Rc::new(eligible_map), source_indices.clone());
    let base_ineligible = build_global_bare_variant_locals(Rc::new(ineligible_map), source_indices);
    assert!(
        base_eligible.contains_key("ProbeEur"),
        "eligible owner must contribute ProbeEur"
    );
    assert!(
        !base_ineligible.contains_key("ProbeEur"),
        "ineligible owner must not contribute ProbeEur"
    );
}

/// Positive control: ambiguous bindings collapse to one tag and never contribute rows.
#[test]
fn global_bare_variant_base_digest_collapses_ambiguous_arm() {
    let si = source_indices_for("x.dag", "module x\n");
    let candidate = Rc::new(GlobalBareCandidate {
        module_path: "a.mod".to_string(),
        binding: Rc::new(TypeBinding {
            name: "Foo".to_string(),
            resolved: module_node("a.dag", "module a.mod\n"),
            provenance: Rc::new(SubValueRelation::SubValueUnknown),
        }),
    });
    let mut map_a: HashMap<String, Rc<GlobalBareLookupState>> = HashMap::new();
    map_a.insert(
        "Foo".to_string(),
        Rc::new(GlobalBareLookupState::GlobalBareAmbiguousBinding {
            candidates: Rc::new(im::vector![candidate.clone()]),
        }),
    );
    let mut map_b = map_a.clone();
    map_b.insert(
        "Foo".to_string(),
        Rc::new(GlobalBareLookupState::GlobalBareAmbiguousBinding {
            candidates: Rc::new(im::vector![candidate.clone(), candidate]),
        }),
    );

    let digest_a = global_bare_semantic_digest_for_test(Rc::new(map_a.clone()), si.clone());
    let digest_b = global_bare_semantic_digest_for_test(Rc::new(map_b.clone()), si.clone());
    assert_eq!(
        digest_a, digest_b,
        "ambiguous arm is key-complete at tag grain — candidate payloads do not affect the builder"
    );
    assert!(
        build_global_bare_variant_locals(Rc::new(map_a), si).is_empty(),
        "ambiguous bindings never insert into the variant base"
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

    // Discriminating arm: a different source hash in the content key must change identity even
    // when a caller might reuse an old NewlineIndex map keyed under a different path spelling.
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
