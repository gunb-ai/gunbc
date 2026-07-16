use std::rc::Rc;

use crate::v2_std_collection::Optional;
use crate::v2_std_node::Node;

pub fn parse_production_emitted_identity_optional(_node: Rc<Node>) -> Rc<Optional<String>> {
    Rc::new(None)
}
