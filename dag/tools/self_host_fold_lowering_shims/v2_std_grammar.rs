use std::rc::Rc;

use crate::v2_std_collection::Optional;
use crate::v2_std_node::{Connective, Node, NodeKind, Symbol};

pub fn node_atom_identity_optional(node: Rc<Node>) -> Rc<Optional<Symbol>> {
    match &*node.kind {
        NodeKind::TypeNode { connective } => match &**connective {
            Connective::Atom { identity } => Rc::new(Some(identity.clone())),
            _ => Rc::new(None),
        },
        _ => Rc::new(None),
    }
}
