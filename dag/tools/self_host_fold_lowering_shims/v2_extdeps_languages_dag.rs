use std::rc::Rc;

use crate::v2_std_grammar::node_atom_identity_optional;
use crate::v2_std_node::{Connective, Edge, EdgeLabel, Node, NodeKind, Symbol};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum ParseSubtreeFind {
    ParseSubtreeAbsent,
    ParseSubtreeFound { captured: Rc<Node> },
}

pub fn parse_subtree_find_production_captured(root: Rc<Node>, emitted: Symbol) -> Rc<ParseSubtreeFind> {
    if let Some(id) = node_atom_identity_optional(root.clone()).as_ref().as_ref() {
        if id == &emitted {
            if let Some(captured) = parse_production_captured_child_optional(root.clone()) {
                return Rc::new(ParseSubtreeFind::ParseSubtreeFound { captured });
            }
        }
    }
    parse_subtree_find_production_captured_under_children(root, emitted)
}

fn parse_subtree_find_production_captured_under_children(
    root: Rc<Node>,
    emitted: Symbol,
) -> Rc<ParseSubtreeFind> {
    root.children.iter().fold(
        Rc::new(ParseSubtreeFind::ParseSubtreeAbsent),
        |acc, edge| match &*acc {
            ParseSubtreeFind::ParseSubtreeFound { .. } => acc,
            ParseSubtreeFind::ParseSubtreeAbsent => {
                parse_subtree_find_production_captured(edge.target.clone(), emitted.clone())
            }
        },
    )
}

fn parse_production_captured_child_optional(root: Rc<Node>) -> Option<Rc<Node>> {
    match &*root.kind {
        NodeKind::ComputationNode { .. } => root
            .children
            .iter()
            .find(|e| matches!(&*e.label, EdgeLabel::Positional))
            .map(|e| e.target.clone()),
        NodeKind::TypeNode { connective } => match &**connective {
            Connective::Conj => root
                .children
                .iter()
                .find(|e| matches!(&*e.label, EdgeLabel::Positional))
                .map(|e| e.target.clone()),
            _ => None,
        },
    }
}
