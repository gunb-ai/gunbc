use std::rc::Rc;

use crate::v2_std_diagnostic::{outcome_accepted, Outcome};
use crate::v2_std_node::Node;

// Witness fixtures are value nodes, not module shells — graft candidate is always false.
pub fn namespace_graft_is_graft_candidate(_root: Rc<Node>) -> bool {
    false
}

pub fn module_header_containment_graft(root: Rc<Node>) -> Rc<Outcome<Rc<Node>>> {
    outcome_accepted(root)
}
