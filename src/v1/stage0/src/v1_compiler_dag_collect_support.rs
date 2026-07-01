use crate::v1_compiler_emit::escape_json_string;
use crate::v1_rt;
use crate::v1_std_core::{
    make_error_node, CompilerDiagnostic, Connective, ErrorNode, ExprData, InferredNode, Node,
    SourceSpan,
};
use std::collections::HashMap;
use std::rc::Rc;

pub fn json_quote(s: String) -> String {
    v1_rt::concat(
        v1_rt::concat("\"".to_string(), escape_json_string(s)),
        "\"".to_string(),
    )
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DagCollectAcc {
    pub seen: Rc<HashMap<String, String>>,
    pub order: Rc<Vec<Rc<Node>>>,
    pub collision_errors: Rc<Vec<Rc<ErrorNode>>>,
}

pub fn inferred_fingerprint(value: Option<Rc<InferredNode>>) -> String {
    match value.as_deref().cloned() {
        None => "none".to_string(),
        Some(InferredNode::Resolved { node: _, .. }) => "Resolved".to_string(),
        Some(InferredNode::TypeVariable { id: id, .. }) => {
            v1_rt::concat("TypeVariable:".to_string(), id.clone())
        }
        Some(InferredNode::CompilerError { message: m, .. }) => {
            v1_rt::concat("CompilerError:".to_string(), m.clone())
        }
    }
}

pub fn expr_data_variant(data: Rc<ExprData>) -> String {
    match (*data).clone() {
        ExprData::NoExprData => "NoExprData".to_string(),
        ExprData::ExprLiteral { value: _, .. } => "ExprLiteral".to_string(),
        ExprData::ExprError { .. } => "ExprError".to_string(),
        ExprData::ExprVar {
            binding_kind: _, ..
        } => "ExprVar".to_string(),
        ExprData::ExprFieldAccess { summary: _, .. } => "ExprFieldAccess".to_string(),
        ExprData::ExprCall { .. } => "ExprCall".to_string(),
        ExprData::ExprMethodCall {
            method_semantics: _,
            ..
        } => "ExprMethodCall".to_string(),
        ExprData::ExprMatch => "ExprMatch".to_string(),
        ExprData::ExprIf => "ExprIf".to_string(),
        ExprData::ExprLet => "ExprLet".to_string(),
        ExprData::ExprRecordLit { parent_enum: _, .. } => "ExprRecordLit".to_string(),
        ExprData::ExprListLit => "ExprListLit".to_string(),
        ExprData::ExprBinOp { .. } => "ExprBinOp".to_string(),
        ExprData::ExprUnaryOp { op: _, .. } => "ExprUnaryOp".to_string(),
        ExprData::ExprLambda => "ExprLambda".to_string(),
        ExprData::ExprStringInterp => "ExprStringInterp".to_string(),
        ExprData::ExprBlock => "ExprBlock".to_string(),
        ExprData::ExprCast => "ExprCast".to_string(),
        ExprData::ExprForEach => "ExprForEach".to_string(),
        ExprData::ExprIndex => "ExprIndex".to_string(),
        ExprData::ExprSlice => "ExprSlice".to_string(),
        ExprData::ExprReturn => "ExprReturn".to_string(),
    }
}

pub fn dag_node_surface_fingerprint(node: Rc<Node>) -> String {
    dag_node_surface_fingerprint_rec(node)
}

fn dag_node_bag_hash(digests: Vec<String>) -> String {
    let mut sorted = digests;
    sorted.sort();
    let mut acc = v1_rt::atom_identity_hash("^dag_collect_bag_empty".to_string());
    for digest in sorted {
        acc = v1_rt::hash_combine(acc, digest);
    }
    acc
}

fn dag_node_seq_hash(digests: Vec<String>) -> String {
    let mut acc = v1_rt::atom_identity_hash("^dag_collect_seq_empty".to_string());
    for digest in digests {
        acc = v1_rt::hash_combine(acc, digest);
    }
    acc
}

fn child_subtree_hash(connective: Connective, digests: Vec<String>) -> String {
    match connective {
        Connective::Conj | Connective::Disj => dag_node_bag_hash(digests),
        Connective::Arrow | Connective::NoConnective => dag_node_seq_hash(digests),
    }
}

pub fn is_synthetic_span(span: &SourceSpan) -> bool {
    span.start == 0 && span.end == 0
}

fn dag_node_surface_leaf_mix(node: &Node) -> String {
    v1_rt::atom_identity_hash(v1_rt::concat(
        v1_rt::concat(
            v1_rt::concat(
                v1_rt::concat(
                    v1_rt::concat(node.name.clone(), "|".to_string()),
                    connective_name(node.connective.clone()),
                ),
                "|".to_string(),
            ),
            inferred_fingerprint(node.inferred.clone()),
        ),
        v1_rt::concat("|".to_string(), expr_data_variant(node.expr_data.clone())),
    ))
}

fn dag_node_surface_fingerprint_rec(node: Rc<Node>) -> String {
    let child_hashes: Vec<String> = node
        .children
        .iter()
        .map(|c| dag_node_surface_fingerprint_rec(c.clone()))
        .collect();
    let param_hashes: Vec<String> = node
        .params
        .iter()
        .map(|p| dag_node_surface_fingerprint_rec(p.clone()))
        .collect();
    let with_children = v1_rt::hash_combine(
        dag_node_surface_leaf_mix(&node),
        child_subtree_hash(node.connective.clone(), child_hashes),
    );
    v1_rt::hash_combine(with_children, dag_node_seq_hash(param_hashes))
}

pub fn dag_node_key_collision_error(key: String, span: Rc<SourceSpan>) -> Rc<ErrorNode> {
    {
        let synthetic = is_synthetic_span(&span);
        let detail = if synthetic {
            " (synthetic 0..0 span; provisional key cannot alias distinct scaffold nodes)"
                .to_string()
        } else {
            "".to_string()
        };
        make_error_node(
            Rc::new(CompilerDiagnostic::InternalError {
                message: v1_rt::concat(
                    v1_rt::concat(
                        "dag artifact: distinct nodes share identity key ".to_string(),
                        json_quote(key),
                    ),
                    detail,
                ),
                span: span.clone(),
            }),
            "".to_string(),
        )
    }
}

pub fn connective_name(value: Connective) -> String {
    match value {
        Connective::Conj => "Conj".to_string(),
        Connective::Disj => "Disj".to_string(),
        Connective::NoConnective => "NoConnective".to_string(),
        Connective::Arrow => "Arrow".to_string(),
    }
}

#[cfg(test)]
mod fingerprint_tests {
    use super::*;
    use crate::v1_std_core::{Cardinality, Connective, ExprData, Node, SourceSpan};
    use std::rc::Rc;

    fn synth_span() -> Rc<SourceSpan> {
        Rc::new(SourceSpan {
            file: "witness.dag".to_string(),
            start: 0,
            end: 0,
        })
    }

    fn shell_node(
        name: &str,
        connective: Connective,
        children: Vec<Rc<Node>>,
        params: Vec<Rc<Node>>,
    ) -> Rc<Node> {
        Rc::new(Node {
            name: name.to_string(),
            ident: None,
            span: synth_span(),
            ident_span: None,
            children: Rc::new(children),
            connective,
            params: Rc::new(params),
            inferred: None,
            return_cardinality: Cardinality::Required,
            uses: crate::v1_std_core::empty_node_list(),
            body: None,
            transport: None,
            properties: crate::v1_std_core::empty_node_list(),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
        })
    }

    #[test]
    fn recursive_fingerprint_distinguishes_same_named_child_subtrees() {
        let child_a_conj = shell_node("a", Connective::Conj, vec![], vec![]);
        let child_a_disj = shell_node("a", Connective::Disj, vec![], vec![]);
        let left = shell_node("wrap", Connective::Conj, vec![child_a_conj], vec![]);
        let right = shell_node("wrap", Connective::Conj, vec![child_a_disj], vec![]);
        let left_fp = dag_node_surface_fingerprint(left);
        let right_fp = dag_node_surface_fingerprint(right);
        assert_ne!(
            left_fp, right_fp,
            "recursive fingerprint must distinguish structurally different 0..0 subtrees with identical child names"
        );
    }

    #[test]
    fn recursive_fingerprint_distinguishes_param_order() {
        let p_x = shell_node("x", Connective::NoConnective, vec![], vec![]);
        let p_y = shell_node("y", Connective::NoConnective, vec![], vec![]);
        let left = shell_node(
            "fn",
            Connective::Arrow,
            vec![],
            vec![p_x.clone(), p_y.clone()],
        );
        let right = shell_node("fn", Connective::Arrow, vec![], vec![p_y, p_x]);
        assert_ne!(
            dag_node_surface_fingerprint(left),
            dag_node_surface_fingerprint(right),
            "params are positional — swapped order must not alias"
        );
    }

    #[test]
    fn recursive_fingerprint_distinguishes_arrow_child_order() {
        let c_x = shell_node("x", Connective::NoConnective, vec![], vec![]);
        let c_y = shell_node("y", Connective::NoConnective, vec![], vec![]);
        let left = shell_node(
            "arr",
            Connective::Arrow,
            vec![c_x.clone(), c_y.clone()],
            vec![],
        );
        let right = shell_node("arr", Connective::Arrow, vec![c_y, c_x], vec![]);
        assert_ne!(
            dag_node_surface_fingerprint(left),
            dag_node_surface_fingerprint(right),
            "Arrow children are ordered — swapped operands must not alias"
        );
    }

    #[test]
    fn recursive_fingerprint_conj_child_order_insensitive() {
        let c_a = shell_node("a", Connective::NoConnective, vec![], vec![]);
        let c_b = shell_node("b", Connective::NoConnective, vec![], vec![]);
        let left = shell_node(
            "bag",
            Connective::Conj,
            vec![c_a.clone(), c_b.clone()],
            vec![],
        );
        let right = shell_node("bag", Connective::Conj, vec![c_b, c_a], vec![]);
        assert_eq!(
            dag_node_surface_fingerprint(left),
            dag_node_surface_fingerprint(right),
            "Conj siblings are commutative — reorder must not false-split"
        );
    }

    #[test]
    fn recursive_fingerprint_disj_child_order_insensitive() {
        let c_a = shell_node("a", Connective::NoConnective, vec![], vec![]);
        let c_b = shell_node("b", Connective::NoConnective, vec![], vec![]);
        let left = shell_node(
            "bag",
            Connective::Disj,
            vec![c_a.clone(), c_b.clone()],
            vec![],
        );
        let right = shell_node("bag", Connective::Disj, vec![c_b, c_a], vec![]);
        assert_eq!(
            dag_node_surface_fingerprint(left),
            dag_node_surface_fingerprint(right),
            "Disj siblings are commutative — reorder must not false-split"
        );
    }
}
