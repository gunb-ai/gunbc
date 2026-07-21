//! emit_imports increment-2: `build_field_type_map` walks the resolved field-type node
//! tree so `List<InnerStruct>` contributes both container and element names to
//! `field_import_surface_names`.

use crate::helpers::compile_dag_resolved;
use v1_compiler::v1_compiler_infer_emit_info::TypeRepr;

const FIXTURE: &str = concat!(
    "module emitinfo.field_surface\n",
    "import std.nat { Nat }\n\n",
    "type InnerEvidence {\n",
    "  tag: Nat\n",
    "}\n\n",
    "type OuterFacts {\n",
    "  evidence: List<InnerEvidence>\n",
    "}\n"
);

#[test]
fn field_import_surface_names_include_generic_list_element() {
    let resolved = compile_dag_resolved(FIXTURE);
    assert!(
        resolved.diagnostics.is_empty(),
        "fixture should resolve cleanly, got: {:?}",
        resolved
            .diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
    let graph = resolved
        .graph
        .as_ref()
        .expect("resolved graph should be present");
    let summary = graph
        .emit_graph_info
        .type_summaries
        .get("OuterFacts")
        .unwrap_or_else(|| panic!("OuterFacts summary missing"));
    let surfaces = summary.field_import_surface_names.clone();
    assert!(
        surfaces.iter().any(|n| n == "List"),
        "container head must be present, got: {surfaces:?}"
    );
    assert!(
        surfaces.iter().any(|n| n == "InnerEvidence"),
        "generic list element must be present in field_import_surface_names, got: {surfaces:?}"
    );
    assert_eq!(
        summary.field_type_map.get("evidence"),
        Some(&"List".to_string()),
        "field_type_map head unchanged"
    );
    assert!(
        matches!(*summary.repr, TypeRepr::StructRepr),
        "OuterFacts must be a struct summary"
    );
}
