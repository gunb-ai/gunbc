//! Discriminating RED for Phase C (sharp-bee-290 msg_6317aebe):
//! undetermined-carrier `none`/`None` must REFUSE, never emit the fail-open
//! Rust `None` default. CardOptional resolves type-directed to `None`.
//!
//! Hand-authored stage0 integration test (not a `ct_` blob — no scaffold
//! roster row). Lives outside regen_stage0 so it is not wiped.

use std::rc::Rc;
use v1_compiler::v1_compiler_emit_rust::emit_none_keyword_for_resolved_type;
use v1_compiler::v1_std_core::{make_span, Cardinality, Connective, ExprData, InferredNode, Node};

fn node_with_cardinality(cardinality: Cardinality) -> Rc<Node> {
    let span = make_span(0, 0);
    Rc::new(Node {
        name: "T".to_string(),
        ident: None,
        span: span.clone(),
        ident_span: Some(span),
        children: Rc::new(Vec::new()),
        connective: Connective::NoConnective,
        params: Rc::new(Vec::new()),
        inferred: None,
        return_cardinality: cardinality,
        uses: Rc::new(Vec::new()),
        body: None,
        transport: None,
        properties: Rc::new(Vec::new()),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(ExprData::NoExprData),
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
    assert_eq!(
        out, "None",
        "CardOptional must type-direct to Rust None, got: {out}"
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
