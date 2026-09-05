//! Discriminating RED for Phase C (sharp-bee-290 msg_6317aebe):
//! undetermined-carrier `none`/`None` must REFUSE, never emit the fail-open
//! Rust `None` default. CardOptional resolves type-directed to `None`.
//!
//! SCAFFOLD (DESIGN §7 HAND-RUST GATE — explicit deferral / RetainedNonMigratable):
//! Lane: Value::Null split (`gunbc.plans.value_null_split`; ROADMAP fail-closed
//! meta band — "Value::Null split (the deep root)").
//! Why host-Rust: calls `emit_none_keyword_for_resolved_type` on the v1 seed
//! emitter surface; a `.dag` floor witness cannot import the v1 compiler crate
//! (same class as `e0308_mechanical_trio_retained`).
//! Dissolution: delete this module when the refuse is pinned by a floor-enrolled
//! emit-fixture witness that reaches the seed emitter, or when `src/v1` deletes
//! at v2 self-host (ROADMAP hand-MAINTAINED→zero terminal).
//! Checkable receipt: `dag/test/retirement/none_undetermined_carrier_refuse_retained.dag`.

use std::rc::Rc;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_emit::emit_keyword;
use v1_compiler::v1_compiler_emit_rust::emit_none_keyword_for_resolved_type;
use v1_compiler::v1_std_core::{
    no_span, Cardinality, Connective, ExprData, InferredNode, Node, NodeOccurrenceIdentity,
};

fn node_with_cardinality(cardinality: Cardinality) -> Rc<Node> {
    let span = no_span();
    Rc::new(Node {
        name: "T".to_string(),
        ident: None,
        span: span.clone(),
        ident_span: Some(span),
        children: Rc::new(im::Vector::new()),
        connective: Connective::NoConnective,
        params: Rc::new(im::Vector::new()),
        inferred: None,
        return_cardinality: cardinality,
        uses: Rc::new(im::Vector::new()),
        body: None,
        transport: None,
        properties: Rc::new(im::Vector::new()),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        expr_data: Rc::new(ExprData::NoExprData),
        occurrence_identity: Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
    })
}

#[test]
fn undetermined_carrier_refuses_not_null_default() {
    let out = emit_none_keyword_for_resolved_type(None);
    assert!(
        out.contains("undetermined carrier"),
        "expected typed refuse mentioning undetermined carrier, got: {out}"
    );
    assert_ne!(
        out, "None",
        "must not emit Rust None when carrier is undetermined"
    );
    assert!(
        out.starts_with("panic!("),
        "refuse should surface as emit_error_expr panic!(...), got: {out}"
    );
}

#[test]
fn card_optional_emits_none() {
    let rt = node_with_cardinality(Cardinality::CardOptional);
    let resolved = Some(Rc::new(InferredNode::Resolved { node: rt }));
    let out = emit_none_keyword_for_resolved_type(resolved);
    // The spelling is the Rust keyword row's (`extdeps.languages.rust.emit`, key `null`), read
    // through the same producer the emitter uses; a literal here would be a second copy of it.
    assert_eq!(
        out,
        emit_keyword("null".to_string(), RenderTarget::Rust),
        "CardOptional must type-direct to the Rust null keyword row, got: {out}"
    );
}

#[test]
fn non_optional_resolved_carrier_refuses() {
    let rt = node_with_cardinality(Cardinality::Required);
    let resolved = Some(Rc::new(InferredNode::Resolved { node: rt }));
    let out = emit_none_keyword_for_resolved_type(resolved);
    assert!(
        out.contains("non-optional resolved carrier"),
        "expected typed refuse for non-optional carrier, got: {out}"
    );
    assert_ne!(out, "None");
}
