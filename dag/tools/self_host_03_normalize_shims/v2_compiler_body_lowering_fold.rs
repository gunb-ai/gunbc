use std::rc::Rc;

use crate::v2_std_collection::Optional;
use crate::v2_std_diagnostic::{Diagnostics, Outcome};
use crate::v2_std_node::{Node, Symbol};

pub fn body_lower_is_deferred_lower_emitted(emitted: Symbol) -> bool {
    emitted == "dag_surface_fn_body"
        || emitted == "dag_surface_pattern"
        || emitted == "dag_surface_match_arm"
        || emitted == "dag_surface_match_arm_stmt_body"
}

pub fn body_lower_finish(
    _n: Rc<Node>,
    folded: Rc<Node>,
    cd: Option<Rc<Diagnostics>>,
) -> Rc<Outcome<Rc<Node>>> {
    Rc::new(Outcome::Accepted {
        value: folded,
        diagnostics: cd.unwrap_or_else(|| Rc::new(Diagnostics::None)),
    })
}
