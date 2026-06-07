// Mechanism-decoupling control for the type-decl reflection intrinsics (ctrl#1484 pin:
// "prove the HOST did NOT pre-enumerate"). The reflection helpers compute arm/field labels
// SOLELY by walking the loaded program's type table (`InterpContext.modules`). With an EMPTY
// type table there is nothing to walk, so a load-bearing walk MUST return empty. If labels
// still came back with no table, that would expose a host fallback / pre-enumerated literal
// masquerading as .dag-native reflection. This complements the claim-run witness's echo
// controls (unknown-name→empty, mutate-real-decl→red) by testing the lookup MECHANISM, not
// just the lookup-miss path.
//
// This lives in an integration-test target (separate crate) on purpose: the lib's own
// `#[cfg(test)]` unit-test target is pre-existing-broken (src/compiler_tests.rs bit-rot), so
// an inline `#[cfg(test)] mod` would be dormant. Integration tests link the compiled lib
// without its cfg(test) modules, so this actually runs.

use std::collections::HashMap;
use std::rc::Rc;

use v2_compiler::v2_compiler_infer_emit_info::empty_emit_graph_info;
use v2_compiler::v2_compiler_infer_items::ResolvedGraph;
use v2_compiler::v2_interpreter::{
    reflect_arm_payload_field_labels, reflect_coproduct_arm_labels, reflect_record_field_labels,
    InterpContext,
};

fn empty_table_ctx() -> InterpContext {
    let graph = ResolvedGraph {
        modules: Rc::new(vec![]),
        item_registry: Rc::new(HashMap::new()),
        diagnostics: Rc::new(vec![]),
        emit_graph_info: empty_emit_graph_info(),
    };
    InterpContext::new(&graph, Rc::new(HashMap::new()), false)
}

#[test]
fn coproduct_arm_labels_empty_with_no_type_table() {
    let ctx = empty_table_ctx();
    assert!(
        reflect_coproduct_arm_labels(&ctx, "ReactElement").is_empty(),
        "reflection returned arms with no type table to walk — host pre-enumeration leak"
    );
}

#[test]
fn record_field_labels_empty_with_no_type_table() {
    let ctx = empty_table_ctx();
    assert!(
        reflect_record_field_labels(&ctx, "ReactContextBinding").is_empty(),
        "reflection returned fields with no type table to walk — host pre-enumeration leak"
    );
}

#[test]
fn arm_payload_field_labels_empty_with_no_type_table() {
    let ctx = empty_table_ctx();
    assert!(
        reflect_arm_payload_field_labels(&ctx, "ReactCreateElementChild", "Text").is_empty(),
        "reflection returned arm-payload fields with no type table to walk — host leak"
    );
}
