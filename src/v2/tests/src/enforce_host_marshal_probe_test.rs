//! Marshaling-first probe (adhoc-99eb67ab-480): native compile_to_resolved on stage-0
//! bare-Int MemorySpec — terminates and attaches InferredNode::Resolved.

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, SourceFile};
use v2_compiler::v2_compiler_infer_items::ItemKind;
use v2_compiler::v2_std_core::InferredNode;

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const BARE_INT_SOURCE: &str = "module stage0.real_source.unit_modeling.reject

type MemorySpec {
  ram_bytes: Int
}
";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

fn bare_int_sources() -> Vec<Rc<SourceFile>> {
    resolve_imports_transitively_with_source_roots(
        "stage0/memory_spec.dag",
        BARE_INT_SOURCE,
        &v4_source_roots(),
    )
}

fn memory_spec_item<'a>(
    graph: &'a v2_compiler::v2_compiler_infer_items::ResolvedGraph,
) -> &'a Rc<v2_compiler::v2_std_core::Node> {
    let info = graph
        .item_registry
        .values()
        .find(|info| info.kind == ItemKind::TypeItem && info.name == "MemorySpec")
        .expect("MemorySpec in item_registry");
    graph
        .modules
        .iter()
        .flat_map(|m| m.items.iter())
        .find(|item| {
            graph
                .item_registry
                .get(&item.name)
                .is_some_and(|i| i.kind == ItemKind::TypeItem && i.name == info.name)
        })
        .expect("type item with inferred")
}

#[test]
fn native_compile_to_resolved_terminates_on_bare_int_memory_spec() {
    let resolved = compile_to_resolved(Rc::new(bare_int_sources()));
    assert!(
        resolved.graph.is_some(),
        "expected resolved graph, diagnostics: {:?}",
        resolved
            .diagnostics
            .iter()
            .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn native_memory_spec_carries_inferred_resolved_node() {
    let resolved = compile_to_resolved(Rc::new(bare_int_sources()));
    let graph = resolved.graph.as_ref().expect("graph");
    let item = memory_spec_item(graph);
    assert!(
        !item.children.is_empty(),
        "MemorySpec type item should declare at least one field"
    );
    match (*item.inferred.clone().expect("inferred")).clone() {
        InferredNode::Resolved { .. } => {}
        other => panic!("expected InferredNode::Resolved, got {:?}", other),
    }
}
