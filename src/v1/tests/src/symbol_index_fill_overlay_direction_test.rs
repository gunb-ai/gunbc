//! Fill composition is direction-fixed: the CLOSURE overlays the shared underlay.
//!
//! `symbol_index_with_qualified_fill` and `symbol_index_with_bare_fill` (04_infer.dag,
//! `underlay_fill_direction_note`) denote one map — union of both key sets, closure value
//! wins on the intersection. The cost fix rewrote them from "walk the underlay's keys and
//! insert the misses into the closure" to "overlay the closure onto the underlay", which is
//! the same denotation only because the closure wins in BOTH readings. This witness is the
//! discriminating control on that clause: `probe.shared` / `ProbeShared` / `shared_service`
//! are present on both sides with different values, so a merge whose direction is flipped
//! (`map_merge(closure, underlay)`) reds on the winner assertion while still producing the
//! right key set. The union clause is carried by the closure-only and underlay-only keys,
//! and the untouched-axis clause by pointer identity with the input map.
//!
//! Size is derived from the boundary it crosses (DESIGN, witness-cost ruling): three keys
//! per axis — one collision plus one from each side — is the requirement plus one.

use std::sync::Arc as Rc;

use v1_compiler::std_induction::SubValueRelation;
use v1_compiler::v1_compiler_compile::{front_end_sources, SourceFile};
use v1_compiler::v1_compiler_infer::{
    symbol_index_with_bare_fill, symbol_index_with_qualified_fill,
};
use v1_compiler::v1_compiler_infer_env::{
    empty_symbol_index, symbol_index_insert, symbol_index_insert_service, GlobalBareLookupState,
    SymbolIndex, TypeBinding,
};
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::Node;

const CLOSURE_SRC: &str = r#"module probe.closure

type ProbeClosureType = ProbeClosureVariant
"#;

const UNDERLAY_SRC: &str = r#"module probe.underlay

type ProbeUnderlayType = ProbeUnderlayVariant
"#;

/// The module `Node` of a single-module source — a real resolved node, so the composed
/// index carries the same value shape production composes.
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
    let graph = frontend.graph.clone().expect("graph");
    graph
        .modules
        .iter()
        .next()
        .expect("single module")
        .module
        .clone()
}

fn unique_bare(module_path: &str, name: &str, resolved: Rc<Node>) -> Rc<GlobalBareLookupState> {
    Rc::new(GlobalBareLookupState::GlobalBareUniqueBinding {
        module_path: module_path.to_string(),
        binding: Rc::new(TypeBinding {
            name: name.to_string(),
            resolved,
            provenance: Rc::new(SubValueRelation::SubValueUnknown),
        }),
    })
}

fn bare_module_path(state: &Rc<GlobalBareLookupState>) -> String {
    match state.as_ref() {
        GlobalBareLookupState::GlobalBareUniqueBinding {
            module_path,
            binding: _,
        } => module_path.clone(),
        GlobalBareLookupState::GlobalBareAmbiguousBinding { .. } => {
            panic!("fixture builds only unique bindings")
        }
    }
}

/// closure side: `probe.shared` + `probe.closure_only` on every axis.
fn closure_index(node: Rc<Node>) -> Rc<SymbolIndex> {
    let with_entries = symbol_index_insert(
        symbol_index_insert(
            empty_symbol_index(),
            "probe.shared".to_string(),
            node.clone(),
        ),
        "probe.closure_only".to_string(),
        node.clone(),
    );
    let with_services = symbol_index_insert_service(
        symbol_index_insert_service(
            with_entries,
            "shared_service".to_string(),
            "probe.closure".to_string(),
            node.clone(),
        ),
        "closure_service".to_string(),
        "probe.closure".to_string(),
        node.clone(),
    );
    Rc::new(SymbolIndex {
        entries: with_services.entries.clone(),
        global_bare: v1_rt::rc_map_insert(
            v1_rt::rc_map_insert(
                with_services.global_bare.clone(),
                "ProbeShared".to_string(),
                unique_bare("probe.closure", "ProbeShared", node.clone()),
            ),
            "ProbeClosureOnly".to_string(),
            unique_bare("probe.closure", "ProbeClosureOnly", node),
        ),
        services: with_services.services.clone(),
    })
}

/// underlay side: `probe.shared` + `probe.underlay_only` on every axis, different values.
fn underlay_index(node: Rc<Node>) -> Rc<SymbolIndex> {
    let with_entries = symbol_index_insert(
        symbol_index_insert(
            empty_symbol_index(),
            "probe.shared".to_string(),
            node.clone(),
        ),
        "probe.underlay_only".to_string(),
        node.clone(),
    );
    let with_services = symbol_index_insert_service(
        symbol_index_insert_service(
            with_entries,
            "shared_service".to_string(),
            "probe.underlay".to_string(),
            node.clone(),
        ),
        "underlay_service".to_string(),
        "probe.underlay".to_string(),
        node.clone(),
    );
    Rc::new(SymbolIndex {
        entries: with_services.entries.clone(),
        global_bare: v1_rt::rc_map_insert(
            v1_rt::rc_map_insert(
                with_services.global_bare.clone(),
                "ProbeShared".to_string(),
                unique_bare("probe.underlay", "ProbeShared", node.clone()),
            ),
            "ProbeUnderlayOnly".to_string(),
            unique_bare("probe.underlay", "ProbeUnderlayOnly", node),
        ),
        services: with_services.services.clone(),
    })
}

#[test]
fn qualified_fill_unions_keys_and_closure_wins_the_collision() {
    let closure_node = module_node("dag/probe_closure.dag", CLOSURE_SRC);
    let underlay_node = module_node("dag/probe_underlay.dag", UNDERLAY_SRC);
    let closure = closure_index(closure_node.clone());
    let fill = underlay_index(underlay_node.clone());

    let composed = symbol_index_with_qualified_fill(closure.clone(), fill.clone());

    assert_eq!(
        composed.entries.len(),
        3,
        "entries must be the union of both sides; got keys={:?}",
        composed.entries.keys().collect::<Vec<_>>()
    );
    let shared = composed
        .entries
        .get("probe.shared")
        .expect("collision key survives");
    assert!(
        Rc::ptr_eq(shared, &closure_node),
        "closure must win the collision: probe.shared resolved to the underlay's node \
         ({} vs closure {}) — the fill overlay is composed in the wrong direction",
        shared.span.file,
        closure_node.span.file
    );
    assert!(
        composed.entries.contains_key("probe.closure_only"),
        "closure-only key must survive"
    );
    assert!(
        composed.entries.contains_key("probe.underlay_only"),
        "underlay-only key must be filled in"
    );
    assert!(
        Rc::ptr_eq(&composed.global_bare, &closure.global_bare)
            && Rc::ptr_eq(&composed.services, &closure.services),
        "qualified fill composes entries only; the bare and service axes stay the closure's"
    );
}

#[test]
fn bare_fill_unions_keys_and_closure_wins_the_collision() {
    let closure_node = module_node("dag/probe_closure.dag", CLOSURE_SRC);
    let underlay_node = module_node("dag/probe_underlay.dag", UNDERLAY_SRC);
    let closure = closure_index(closure_node);
    let tree = underlay_index(underlay_node);

    let composed = symbol_index_with_bare_fill(closure.clone(), tree.clone());

    assert_eq!(
        composed.global_bare.len(),
        3,
        "global_bare must be the union of both sides; got keys={:?}",
        composed.global_bare.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        bare_module_path(
            composed
                .global_bare
                .get("ProbeShared")
                .expect("collision key survives")
        ),
        "probe.closure",
        "closure must win the bare collision — the tree underlay is composed under it, \
         never over it"
    );
    assert!(
        composed.global_bare.contains_key("ProbeClosureOnly")
            && composed.global_bare.contains_key("ProbeUnderlayOnly"),
        "both one-sided bare keys must survive"
    );

    assert_eq!(
        composed.services.len(),
        3,
        "services must be the union of both sides; got keys={:?}",
        composed.services.keys().collect::<Vec<_>>()
    );
    assert_eq!(
        composed
            .services
            .get("shared_service")
            .expect("collision key survives")
            .module_path,
        "probe.closure",
        "closure must win the service collision"
    );
    assert!(
        composed.services.contains_key("closure_service")
            && composed.services.contains_key("underlay_service"),
        "both one-sided service keys must survive"
    );

    assert!(
        Rc::ptr_eq(&composed.entries, &closure.entries),
        "bare fill composes global_bare and services only; entries stay the closure's"
    );
}
