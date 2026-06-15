//! Marshaling-first probe (adhoc-99eb67ab-480): native compile_to_resolved → extract
//! InferredNode for stage-0 bare-Int MemorySpec; first gate before v4 InferredTree marshal.

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, SourceFile};
use v2_compiler::v2_std_core::{
    authored_name_at, Connective, InferredNode, ItemKind, Node,
};

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

fn find_memory_spec_type_item(graph: &v2_compiler::v2_compiler_infer_items::ResolvedGraph) -> Option<Rc<Node>> {
    for module in graph.modules.iter() {
        for item in module.items.iter() {
            if let Some(info) = graph.item_registry.get(&authored_item_name(item)) {
                if info.kind == ItemKind::TypeItem && info.name == "MemorySpec" {
                    return Some(item.clone());
                }
            }
        }
    }
    None
}

fn authored_item_name(item: &Rc<Node>) -> String {
    // Item nodes carry authored name on the module item record.
    let name = authored_name_at(
        Rc::new(std::collections::HashMap::new()),
        item.clone(),
    );
    if name.is_empty() {
        format!("{:?}", item.connective)
    } else {
        name
    }
}

#[test]
fn native_compile_to_resolved_terminates_on_bare_int_memory_spec() {
    let sources = bare_int_sources();
    let resolved = compile_to_resolved(Rc::new(sources));
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
fn native_memory_spec_type_carries_inferred_resolved_node() {
    let sources = bare_int_sources();
    let resolved = compile_to_resolved(Rc::new(sources));
    let graph = resolved.graph.as_ref().expect("graph");
    let item = find_memory_spec_type_item(graph).expect("MemorySpec type item");
    let inferred = item.inferred.clone().expect("inferred attachment");
    match (*inferred).clone() {
        InferredNode::Resolved { node } => {
            assert_eq!(
                node.connective,
                Connective::Conj,
                "MemorySpec inferred root should be Conj (record)"
            );
            assert!(
                !node.children.is_empty(),
                "MemorySpec should have ram_bytes field"
            );
        }
        other => panic!("expected InferredNode::Resolved, got {:?}", other),
    }
}
