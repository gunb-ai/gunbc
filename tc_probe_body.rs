// THROWAWAY probe (NOT a committed test): same-text-refused FIELD AXIS.
// QUESTION (parent msg_a6dbe1c9 / msg_92f01032): node_type_shape prints two nodes
// IDENTICALLY, yet node_type_equals answers false. Which Node field is decisive in
// equality and invisible in shape? Hypothesis: `declaration` (None-vs-Present, the
// resolution asymmetry) reaching type_heads_agree, which -- when neither head is a
// declared carrier alias -- demands BOTH sides Present with equal module_path+decl_name.
// Fixtures use REAL table heads so node_is_element_collection classifies them:
// dag/std/algebra.dag carrier_container_algebra_rows gives FreeMonoid element_child_index
// = Some 0 and Map element_child_index = None (so Map carries two dimensions, matching
// the verbatim CI text Map(Primitive(K),Primitive(V))). TextJudgment::TextNotAsked on
// both sides, as the fold's rebind path passes it.
use im::{HashMap, Vector};
use std::rc::Rc;

use v1_compiler::std_decl_ref::{decl_ref, DeclarationRef};
use v1_compiler::v1_compiler_infer_types::{
    node_type_equals, node_type_shape, TextJudgment, TextNotAskedReason,
};
use v1_compiler::v1_std_core::{
    no_span, Cardinality, Connective, ExprData, Node, NodeOccurrenceIdentity,
    ParsedModuleItemKind, SourceSpan,
};

fn si() -> Rc<HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>> {
    Rc::new(HashMap::new())
}

fn node(name: &str, children: Vec<Rc<Node>>, decl: Option<Rc<DeclarationRef>>) -> Rc<Node> {
    let sp = Rc::new(SourceSpan { file: "tc.probe".to_string(), start: 0, end: 1 });
    Rc::new(Node {
        name: name.to_string(),
        ident: None,
        span: sp.clone(),
        ident_span: Some(sp),
        children: Rc::new(Vector::from(children)),
        connective: Connective::NoConnective,
        params: Rc::new(Vector::new()),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(Vector::new()),
        body: None,
        transport: None,
        properties: Rc::new(Vector::new()),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: ParsedModuleItemKind::NotAModuleItem,
        declaration_marker: v1_compiler::v1_std_core::DeclarationMarker::Unmarked,
        declaration: decl,
        expr_data: Rc::new(ExprData::NoExprData),
        occurrence_identity: Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
    })
}

fn leaf(name: &str) -> Rc<Node> {
    node(name, vec![], None)
}

fn row(label: &str, left: Rc<Node>, right: Rc<Node>) {
    let s = si();
    let shape_l = node_type_shape(left.clone(), s.clone());
    let shape_r = node_type_shape(right.clone(), s.clone());
    let eq = node_type_equals(
        left,
        right,
        s,
        Rc::new(TextJudgment::TextNotAsked {
            reason: TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes,
        }),
    );
    let same = shape_l == shape_r;
    let verdict = if same && !eq {
        "*** SAME-TEXT REFUSED (defect class) ***"
    } else if same && eq {
        "same-text ACCEPTED"
    } else {
        "distinct shapes"
    };
    println!("TC {label} | shapes_equal={same} | equals={eq} | {verdict}");
    println!("TC {label}   left={shape_l}  right={shape_r}");
}

#[test]
fn tc_same_text_refused_field_axis() {
    let _ = no_span();
    let element = leaf("element");
    // A. FreeMonoid<element>, declaration None on both. Must ACCEPT (pure control).
    row(
        "A_freemonoid_none_vs_none",
        node("FreeMonoid", vec![element.clone()], None),
        node("FreeMonoid", vec![element.clone()], None),
    );
    // B. THE HYPOTHESIS: identical print, declaration None vs Present.
    row(
        "B_freemonoid_none_vs_present",
        node("FreeMonoid", vec![element.clone()], None),
        node("FreeMonoid", vec![element.clone()], Some(decl_ref("std.core".into(), "FreeMonoid".into()))),
    );
    // C. Present vs Present, SAME module+name. Must ACCEPT.
    row(
        "C_freemonoid_same_decl",
        node("FreeMonoid", vec![element.clone()], Some(decl_ref("std.core".into(), "FreeMonoid".into()))),
        node("FreeMonoid", vec![element.clone()], Some(decl_ref("std.core".into(), "FreeMonoid".into()))),
    );
    // D. Present vs Present, DIFFERENT module_path, same carrier head: the review-80660
    // class. This refusal is CORRECT and must survive any fix (parent msg_92f01032).
    row(
        "D_freemonoid_cross_module",
        node("FreeMonoid", vec![element.clone()], Some(decl_ref("std.core".into(), "FreeMonoid".into()))),
        node("FreeMonoid", vec![element.clone()], Some(decl_ref("app.other".into(), "FreeMonoid".into()))),
    );
    // E. Map<K,V>: element_child_index None => both children are dimensions. Verbatim CI text.
    row(
        "E_map_none_vs_none",
        node("Map", vec![leaf("K"), leaf("V")], None),
        node("Map", vec![leaf("K"), leaf("V")], None),
    );
    row(
        "E_map_none_vs_present",
        node("Map", vec![leaf("K"), leaf("V")], None),
        node("Map", vec![leaf("K"), leaf("V")], Some(decl_ref("std.core".into(), "Map".into()))),
    );
    row(
        "E_map_cross_module",
        node("Map", vec![leaf("K"), leaf("V")], Some(decl_ref("std.core".into(), "Map".into()))),
        node("Map", vec![leaf("K"), leaf("V")], Some(decl_ref("app.other".into(), "Map".into()))),
    );
    // F. StructurallyGuaranteed arm: nested Product(Unit) inside Outcome. The verbatim CI
    // text Outcome(Product(Unit)) vs itself.
    row(
        "F_outcome_product_none_vs_none",
        node("Outcome", vec![node("Product", vec![leaf("Unit")], None)], None),
        node("Outcome", vec![node("Product", vec![leaf("Unit")], None)], None),
    );
    row(
        "F_outcome_product_none_vs_present",
        node("Outcome", vec![node("Product", vec![leaf("Unit")], None)], None),
        node("Outcome", vec![node("Product", vec![leaf("Unit")], None)], Some(decl_ref("std.core".into(), "Outcome".into()))),
    );
    // G. Does a differing occurrence identity (absent from shape) decide equality?
    let left = node("FreeMonoid", vec![element.clone()], None);
    let mut right_node = (*node("FreeMonoid", vec![leaf("element")], None)).clone();
    right_node.occurrence_identity = Rc::new(NodeOccurrenceIdentity::OccurrenceMinted {
        id: v1_compiler::std_occurrence_identity::OccurrenceId { value: 7 },
    });
    row("G_freemonoid_occurrence_diff", left, Rc::new(right_node));
    // H. Does a differing SPAN (absent from shape) decide equality?
    let mut spanned = (*node("FreeMonoid", vec![element.clone()], None)).clone();
    spanned.span = Rc::new(SourceSpan { file: "other.file".to_string(), start: 400, end: 480 });
    row(
        "H_freemonoid_span_diff",
        node("FreeMonoid", vec![element.clone()], None),
        Rc::new(spanned),
    );
    println!("TC_DONE");
}
